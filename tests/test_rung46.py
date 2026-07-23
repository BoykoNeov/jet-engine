"""Rung 46 — THE TIT TOPPING GOVERNOR on the two-shaft fuel path.

Rung 43/45 established a fuel accel makes Tt4 OVERSHOOT (rung 35 on two shafts), and every one
ended on the same concession: "Tt4 overshoot vs a redline is not claimed -- no TIT limit is
modelled." Rung 46 models it: `integrate_fuel(..., Tt4_max=...)` clips the metered fuel to hold
Tt4 <= Tt4_max (a min-select, the standard accel-schedule TIT limiter) -- the FIRST fuel-side
FEEDBACK in the ladder. `topping_relief` marches the same accel bare vs topped and differences the
surge object.

THE HEADLINE (the inversion) — the surge-relief SPLIT:
  The governor WORKS (pins Tt4 at the redline -- the effect added). But its surge-side consequence
  INVERTS "the two accel limits are coupled, so enforcing one relieves the other": enforcing the
  TIT redline rebates surge margin on the LATE, non-binding HP spool (relief_hp>0) but MACHINE-ZERO
  on the EARLY, binding LP spool (relief_lp=0). A two-shaft differential no single shaft can show.

THE MECHANISM (the why):
  The surge debit is paid on the EARLY-ramp fuel -- the LP surge minimum falls at Tt4~1374, DURING
  the ramp, below any valid redline, then self-recovers. The governor only trims LATE fuel (Tt4 >
  redline, near ramp end). It cannot refund a surge cost incurred UPSTREAM of its window. Rung 35's
  two limits are coupled in CAUSE but SEQUENCED in time; the governor acts on the trailing (TIT)
  limit and structurally misses the leading (surge) one. Rung 45's "fuel ENLARGES the surge
  approach" gets its punchline: the enlargement is deposited early, the governor is too late.

THE LEVER (the caveat): relief_lp=0 only at MODERATE r; in the fast-ramp limit (r<=0.3) the LP
  surge minimum migrates ABOVE the redline and relief_lp goes positive -- the governor becomes a
  modest LP-surge lever precisely where surge is most dangerous.

Gates (docs/rung46-spec.md § Verification gates):
  1. REDUCE — dormant (Tt4_max above the bare peak, or None) => integrate_fuel/phi_excursion_fuel
     bit-for-bit rung 45/43.
  2. REDUCE — lp_disabled ASSERTS (the split is inherently two-shaft).
  3+4+5. THE GOVERNOR HOLDS + THE SPLIT + THE MECHANISM — every shape incl hp-only: held,
     relief_lp~0 & relief_hp>0, and LP-min-Tt4 < redline < HP-min-Tt4.
  6. THE LEVER — relief_lp zero at moderate r, positive at r<=0.15.
  7. DECEL — clip never fires => bit-for-bit rung 45.
  8. CYCLE UNTOUCHED => rung 6.

Why no independent bare-math gate (rung 45's precedent): rung 46 marches rung 43's integrate_fuel
(anchored transitively to rung 40's steady manifold). The new content is the topping FEEDBACK
(gated by "Tt4 is HELD" + the dormant reduce) and the surge-relief-split SIGNS (shape-robust
directions, not magnitudes a bare-math replica would constrain).
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolFuelTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
SINGLE = dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92,
              eta_m=0.99, pi_n=0.98)

LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
TILTED = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)
FLAT = ComponentMap.flat()
SHAPES = {
    "flow/press": (LP_SHAPED, HP_SHAPED),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (TILTED, TILTED),
    "hp-only":    (FLAT, HP_SHAPED),   # LP FLAT => NO rung-40 complex mode (the discriminator)
}

# accel band + a redline in the GAP (above the 1400 endpoint, below the ~1645 bare peak)
LO, HI, REDLINE, R = 1000.0, 1400.0, 1480.0, 0.5
SETTLE = 2.0   # the surge min + Tt4 peak both live inside the ramp; a short settle suffices


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _tpg():
    return Gas.thermally_perfect()


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _ft(gas, ml=None, mh=None, rho=1.0, lp_disabled=False):
    return TwoSpoolFuelTransient(_design(gas), FLIGHT, 1.0, map_lp=ml, map_hp=mh,
                                 rho=rho, lp_disabled=lp_disabled)


# ======================================================================================
# GATE 1 — REDUCE: dormant governor => bit-for-bit rung 45/43
# ======================================================================================

def test_reduce_dormant_bit_for_bit_rung45():
    """A redline above the bare peak (and Tt4_max=None) leaves the clip un-consulted: the topped
    march is the bare rung-43 march float-for-float, and the rung-45 referenced excursion is
    identical armed-vs-bare (it never reads the redline)."""
    ft = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0)
    bare = ft.phi_excursion_fuel(FLIGHT, LO, HI, r=R, s_settle=SETTLE)
    huge = bare["Tt4_peak"] + 500.0
    assert ft.phi_excursion_fuel(FLIGHT, LO, HI, r=R, s_settle=SETTLE, Tt4_max=huge) == bare

    mf0, mf1 = ft.fuel_for_Tt4(FLIGHT, LO), ft.fuel_for_Tt4(FLIGHT, HI)
    eq0 = ft.equilibrium(FLIGHT, LO)
    nu0 = (eq0["nu_lp"], eq0["nu_hp"])

    def sched(s):
        return mf0 + (mf1 - mf0) * min(1.0, s / R)

    pa = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, 0.02)
    pb = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, 0.02, Tt4_max=huge)
    pc = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, 0.02, Tt4_max=None)
    assert len(pa) == len(pb) == len(pc)
    for a, b, c in zip(pa, pb, pc):
        key = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f")
        assert tuple(a[k] for k in key) == tuple(b[k] for k in key)   # dormant redline
        assert tuple(a[k] for k in key) == tuple(c[k] for k in key)   # Tt4_max=None


def test_reduce_lp_disabled_asserts_the_split_is_two_shaft():
    """The surge-relief SPLIT is inherently two-shaft: lp_disabled is not a reduce axis for a split
    BETWEEN spools, so the governor asserts on the degenerate engine (both via _fuel_ramp_march and
    directly in integrate_fuel). The Tt4_max=None dispatch to rung 35 is untouched."""
    ftd = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0, lp_disabled=True)
    with pytest.raises(AssertionError):
        ftd.topping_relief(FLIGHT, LO, HI, REDLINE)
    with pytest.raises(AssertionError):
        ftd.integrate_fuel(FLIGHT, lambda s: 0.5, 1.0, 1.0, 0.05, Tt4_max=REDLINE)


# ======================================================================================
# GATES 3+4+5 — THE GOVERNOR HOLDS, THE SPLIT, THE MECHANISM
# ======================================================================================

def test_governor_holds_and_the_surge_relief_split():
    """THE HEADLINE. Every shape incl. the mode-free hp-only:
      (3) the governor HOLDS Tt4 at the redline;
      (4) the SPLIT — relief_lp machine-zero, relief_hp strictly positive (a two-shaft
          differential; the exact LP zero is the clip never touching the LP surge minimum);
      (5) the MECHANISM — LP-min-Tt4 < redline < HP-min-Tt4, so the clip window excludes the
          (early) LP minimum and includes the (late) HP one.
    hp-only (LP FLAT, no complex mode) witnesses the split is the WINDOW mechanism, not a mode
    artifact. Magnitudes disclaimed; the differential's SIGN is gated."""
    gas = _tpg()
    d = _design(gas)
    for name, (ml, mh) in SHAPES.items():
        ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        R_out = ft.topping_relief(FLIGHT, LO, HI, REDLINE, r=R, s_settle=SETTLE)
        assert R_out["held"] and R_out["Tt4_peak_top"] <= REDLINE + 1e-6, \
            (name, "governor must hold Tt4 at the redline", R_out["Tt4_peak_top"])
        assert abs(R_out["relief_lp"]) < 1e-9, \
            (name, "LP (binding) surge relief must be machine-zero", R_out["relief_lp"])
        assert R_out["relief_hp"] > 1e-6, \
            (name, "HP (late) surge relief must be strictly positive", R_out["relief_hp"])

    # the mechanism (one shape — the ordering that gives the split its sign)
    ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
    tj, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, R, SETTLE, 0.02)
    lp_min_Tt4 = min(tj, key=lambda p: p["phi_lp"])["Tt4"]
    hp_min_Tt4 = min(tj, key=lambda p: p["phi_hp"])["Tt4"]
    assert lp_min_Tt4 < REDLINE < hp_min_Tt4, \
        ("window: LP-min-Tt4 < redline < HP-min-Tt4", lp_min_Tt4, REDLINE, hp_min_Tt4)


# ======================================================================================
# GATE 6 — THE LEVER: relief_lp switches on in the fast-ramp limit
# ======================================================================================

def test_the_lever_fast_ramp_switches_on_lp_relief():
    """relief_lp is machine-zero at MODERATE r (the LP surge min sits below the redline) but goes
    strictly POSITIVE in the fast-ramp limit (r<=0.15): the LP surge minimum migrates above the
    redline into the clip window. The gated claim is 'zero at moderate r, positive fast', not an
    unconditional zero."""
    ft = TwoSpoolFuelTransient(_design(_tpg()), FLIGHT, 1.0,
                               map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
    Tt4_max = 1440.0
    slow = ft.topping_relief(FLIGHT, LO, HI, Tt4_max, r=0.5, s_settle=SETTLE)
    fast = ft.topping_relief(FLIGHT, LO, HI, Tt4_max, r=0.15, s_settle=SETTLE)
    assert abs(slow["relief_lp"]) < 1e-9, ("moderate r: LP relief zero", slow["relief_lp"])
    assert fast["relief_lp"] > 1e-4, ("fast r: LP relief positive", fast["relief_lp"])
    assert fast["relief_hp"] > slow["relief_hp"], "faster ramp => more HP rebate too"


# ======================================================================================
# GATE 7 — DECEL: clip never fires => bit-for-bit rung 45
# ======================================================================================

def test_decel_bit_for_bit_rung45():
    """The topping governor is an ACCELERATION-schedule limiter. On a decel Tt4 undershoots and
    never exceeds a redline above the endpoint, so the clip never fires and the topped decel march
    equals the bare (rung 45) decel march float-for-float, at every shape."""
    d = _design(_cpg_gas())
    for name, (ml, mh) in SHAPES.items():
        ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        bare = ft.phi_excursion_fuel(FLIGHT, HI, LO, r=R, s_settle=SETTLE)
        top = ft.phi_excursion_fuel(FLIGHT, HI, LO, r=R, s_settle=SETTLE, Tt4_max=REDLINE)
        assert top == bare, (name, "decel: clip must never fire")


# ======================================================================================
# GATE 8 — CYCLE UNTOUCHED => rung 6
# ======================================================================================

def test_cycle_untouched_by_topping_governor_bit_for_bit_rung6():
    """Exercising the governor must not perturb the default single-spool design run (rung 6)."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    ft = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED)
    ft.topping_relief(FLIGHT, LO, HI, REDLINE, r=R, s_settle=1.5)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


if __name__ == "__main__":
    for fn in (test_reduce_dormant_bit_for_bit_rung45,
               test_reduce_lp_disabled_asserts_the_split_is_two_shaft,
               test_governor_holds_and_the_surge_relief_split,
               test_the_lever_fast_ramp_switches_on_lp_relief,
               test_decel_bit_for_bit_rung45,
               test_cycle_untouched_by_topping_governor_bit_for_bit_rung6):
        fn()
        print("PASS", fn.__name__)
