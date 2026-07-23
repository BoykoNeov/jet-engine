"""Rung 47 — THE LAGGED / ACTUATOR TOPPING GOVERNOR (tau_gov) on the two-shaft fuel path.

Rung 46 modelled the TIT topping governor as an idealised INSTANTANEOUS min-select fuel clip,
and closed on the concession: "a lagged governor is left open; a slow-enough governor would
smear the clip window and could reach EARLIER into the LP surge point." Rung 47 models the lag --
and REFUTES that hope.

`integrate_fuel(..., Tt4_max=..., tau_gov=...)` gives the governor a finite RESPONSE LAG (the
sensing / limiter-loop lag of a real TIT limiter, the DOMINANT lag in practice). The clip AMOUNT
(the fuel REDUCTION below the schedule) becomes a THIRD state `g` that relaxes toward the
instantaneous requirement with `tau_gov` (`_integrate_fuel_lagged`); the applied fuel is
`schedule - g`. `topping_relief(..., tau_gov=...)` differences bare vs topped-lagged.

THE HEADLINE (the refutation): a first-order lag is a TRAILING-edge tool -- it delays the
governor's action, it never ANTICIPATES. It cannot reach the EARLY LP surge minimum (s~0.24,
Tt4~1374, UPSTREAM of engagement). `relief_lp = 0` EXACTLY for every tau_gov at moderate r
(the topped march is bit-identical to bare up to the late engagement; the lag only ADDS fuel
after it). At the FAST ramp where rung 46's instantaneous governor DID reach the LP (the lever,
relief_lp>0), the lag ERODES that positive relief toward zero -- strictly WORSE than the ideal.
In NO regime does the lag reach the LP better than the instantaneous min-select. You cannot cure
a leading-edge problem with a trailing-edge tool.

THE COST OF REALISM: the lag DESTROYS rung 46's clean "the governor HOLDS the redline" (its gate
3). Tt4 OVERSHOOTS the redline by an amount growing with tau_gov (~55->191 K in-band at r=0.5,
~220->390 K at fast r=0.15) -- the classic topping overshoot -- and the HP rebate ERODES toward
zero. A real (lagged) governor is strictly worse than rung 46's idealisation: it breaks the TIT
hold, erodes the HP surge rebate, and still misses the LP.

THE SECONDARY (where the lag lives): the overshoot is NOT a property of any lag. A pure metering-
VALVE-position lag is INERT on the accel -- once the governor engages the binding topping command
RISES monotonically (nu up => airflow up => more fuel to hold the redline), so an instant-up valve
tracks it and never lags. The overshoot lives specifically in the sensing/limiter-LOOP lag (the
loop can't wind fuel down fast enough). WHERE the lag lives decides whether it even overshoots
(`topping_command_trace` gates the monotone command that makes the valve lag inert).

Reduces: tau_gov=None is the instantaneous rung-46 min-select (bit-for-bit); a redline above the
bare peak leaves the clip un-consulted (bit-for-bit rung 45); lp_disabled ASSERTS (the finding is
inherently two-shaft); decel never fires the clip (bit-for-bit rung 45); the default design run is
bit-for-bit rung 6.
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

# accel band + a redline in the GAP (above the 1400 endpoint, below the ~1670 bare peak)
LO, HI, REDLINE, R = 1000.0, 1400.0, 1480.0, 0.5
SETTLE = 2.0   # the surge min + Tt4 peak both live inside the ramp; a short settle suffices


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _ft(gas, ml=None, mh=None, rho=1.0, lp_disabled=False):
    return TwoSpoolFuelTransient(_design(gas), FLIGHT, 1.0, map_lp=ml, map_hp=mh,
                                 rho=rho, lp_disabled=lp_disabled)


# ======================================================================================
# SPINE — REDUCE gates (kept on every `pytest`, never slow-tagged)
# ======================================================================================

def test_reduce_tau_none_bit_for_bit_rung46():
    """tau_gov=None is the idealised instantaneous min-select -- byte-identical to the rung-46
    call (which never passed tau_gov). integrate_fuel and topping_relief both reduce."""
    ft = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0)
    mf0, mf1 = ft.fuel_for_Tt4(FLIGHT, LO), ft.fuel_for_Tt4(FLIGHT, HI)
    eq0 = ft.equilibrium(FLIGHT, LO)
    nu0 = (eq0["nu_lp"], eq0["nu_hp"])

    def sched(s):
        return mf0 + (mf1 - mf0) * min(1.0, s / R)

    pa = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, 0.02, Tt4_max=REDLINE)
    pb = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, 0.02, Tt4_max=REDLINE, tau_gov=None)
    assert len(pa) == len(pb)
    key = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "mf")
    for a, b in zip(pa, pb):
        assert tuple(a[k] for k in key) == tuple(b[k] for k in key)

    r46 = ft.topping_relief(FLIGHT, LO, HI, REDLINE, r=R, s_settle=SETTLE)
    rNone = ft.topping_relief(FLIGHT, LO, HI, REDLINE, r=R, s_settle=SETTLE, tau_gov=None)
    assert (r46["relief_lp"], r46["relief_hp"], r46["Tt4_peak_top"], r46["held"]) == \
           (rNone["relief_lp"], rNone["relief_hp"], rNone["Tt4_peak_top"], rNone["held"])
    assert r46["held"] and r46["overshoot"] <= 1e-6   # the instantaneous governor holds


def test_reduce_dormant_lag_bit_for_bit_rung45():
    """A redline above the bare peak leaves the clip un-consulted => the required clip stays 0 =>
    g stays 0 => the lagged march is the bare rung-45 march float-for-float, at any tau_gov."""
    ft = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0)
    mf0, mf1 = ft.fuel_for_Tt4(FLIGHT, LO), ft.fuel_for_Tt4(FLIGHT, HI)
    eq0 = ft.equilibrium(FLIGHT, LO)
    nu0 = (eq0["nu_lp"], eq0["nu_hp"])

    def sched(s):
        return mf0 + (mf1 - mf0) * min(1.0, s / R)

    bare = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, 0.02)
    huge = max(p["Tt4"] for p in bare) + 500.0
    lagged = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, 0.02,
                               Tt4_max=huge, tau_gov=0.3)
    assert len(bare) == len(lagged)
    key = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f")
    for a, b in zip(bare, lagged):
        assert tuple(a[k] for k in key) == tuple(b[k] for k in key)


def test_reduce_lp_disabled_and_tau_needs_redline_assert():
    """The lag is a governor lag: it asserts on the degenerate engine (the split is inherently
    two-shaft, rung 46's contract) and requires a redline (a lag with no governor is meaningless)."""
    ftd = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0, lp_disabled=True)
    with pytest.raises(AssertionError):
        ftd.integrate_fuel(FLIGHT, lambda s: 0.5, 1.0, 1.0, 0.05,
                           Tt4_max=REDLINE, tau_gov=0.2)
    ft = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0)
    with pytest.raises(AssertionError):
        ft.integrate_fuel(FLIGHT, lambda s: 0.5, 1.0, 1.0, 0.05, tau_gov=0.2)  # no Tt4_max


def test_decel_lagged_bit_for_bit_rung45():
    """The topping governor is an ACCELERATION limiter. On a decel Tt4 undershoots, the clip
    never fires, g stays 0 => the lagged decel march equals the bare (rung 45) march at every
    shape and any tau_gov."""
    d = _design(_cpg_gas())
    for name, (ml, mh) in SHAPES.items():
        ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        bare = ft.phi_excursion_fuel(FLIGHT, HI, LO, r=R, s_settle=SETTLE)
        top = ft.phi_excursion_fuel(FLIGHT, HI, LO, r=R, s_settle=SETTLE,
                                    Tt4_max=REDLINE, tau_gov=0.3)
        assert top == bare, (name, "decel: clip must never fire, even lagged")


def test_cycle_untouched_by_lagged_governor_bit_for_bit_rung6():
    """Exercising the lagged governor must not perturb the default single-spool design run."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    ft = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED)
    ft.topping_relief(FLIGHT, LO, HI, REDLINE, r=R, s_settle=1.5, tau_gov=0.2)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


# ======================================================================================
# THE HEADLINE — the lag overshoots the redline, erodes the HP rebate, and STILL misses the LP
# ======================================================================================

def test_lagged_governor_overshoots_erodes_hp_and_misses_lp():
    """At every shape (incl. mode-free hp-only), a lagged governor (tau_gov=0.2):
      * OVERSHOOTS the redline (held False, overshoot>0) -- rung 46's gate-3 hold is LOST;
      * keeps relief_lp == 0 EXACTLY -- the lag is a trailing-edge tool, it cannot reach the
        early LP surge minimum (the refutation of rung 46's next-seam hope);
      * still gives relief_hp > 0 but ERODED below the instantaneous rebate (the clip is softer,
        later).
    hp-only (LP flat, no rung-40 complex mode) witnesses the refutation is the WINDOW/timing
    mechanism, not a mode artifact. Magnitudes disclaimed; the SIGNS are gated."""
    d = _design(_cpg_gas())
    for name, (ml, mh) in SHAPES.items():
        ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        inst = ft.topping_relief(FLIGHT, LO, HI, REDLINE, r=R, s_settle=SETTLE)
        lag = ft.topping_relief(FLIGHT, LO, HI, REDLINE, r=R, s_settle=SETTLE, tau_gov=0.2)
        assert not lag["held"] and lag["overshoot"] > 1.0, \
            (name, "lagged governor must OVERSHOOT the redline", lag["overshoot"])
        assert abs(lag["relief_lp"]) < 1e-9, \
            (name, "the lag still misses the early LP min (relief_lp==0)", lag["relief_lp"])
        assert 0.0 < lag["relief_hp"] < inst["relief_hp"], \
            (name, "HP rebate positive but ERODED vs the instantaneous governor",
             lag["relief_hp"], inst["relief_hp"])


def test_overshoot_grows_and_hp_erodes_monotone_in_tau():
    """The cost of the lag is monotone in tau_gov (flow/press, r=0.5): the redline overshoot GROWS
    and the HP rebate ERODES as the governor gets slower, while relief_lp stays pinned at 0. This
    is the inversion of rung 46's gate 3 ('the governor holds') resolved as a knob."""
    ft = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0)
    prev_ov, prev_hp = -1.0, 1.0
    for tau in (0.05, 0.1, 0.2, 0.4, 0.8):
        o = ft.topping_relief(FLIGHT, LO, HI, REDLINE, r=R, s_settle=SETTLE, tau_gov=tau)
        assert o["overshoot"] > prev_ov, ("overshoot must grow with tau", tau, o["overshoot"])
        assert o["relief_hp"] < prev_hp, ("HP rebate must erode with tau", tau, o["relief_hp"])
        assert abs(o["relief_lp"]) < 1e-9, ("relief_lp pinned at 0", tau, o["relief_lp"])
        prev_ov, prev_hp = o["overshoot"], o["relief_hp"]


# ======================================================================================
# THE LEVER, LAGGED — at fast r the lag ERODES rung 46's positive LP relief, never enhances it
# ======================================================================================

def test_fast_ramp_lp_relief_eroded_by_lag_never_enhanced():
    """The airtight half of the refutation. At the FAST ramp (r=0.15) rung 46's INSTANTANEOUS
    governor DOES reach the LP (relief_lp>0 -- the lever). If a lag could 'reach earlier into the
    LP surge point' (the rung-46 concession's hope), the lagged relief_lp would EXCEED the
    instantaneous. It does the opposite: the lag ERODES it toward zero, monotonically. So in
    BOTH regimes the lag reaches the LP no better than the ideal min-select -- neutral at
    moderate r, strictly worse at fast r."""
    ft = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0)
    red = 1440.0
    inst = ft.topping_relief(FLIGHT, LO, HI, red, r=0.15, s_settle=SETTLE)
    assert inst["relief_lp"] > 1e-3, ("rung 46 reaches the LP at fast r", inst["relief_lp"])
    prev = inst["relief_lp"]
    for tau in (0.05, 0.1, 0.2, 0.4):
        o = ft.topping_relief(FLIGHT, LO, HI, red, r=0.15, s_settle=SETTLE, tau_gov=tau)
        assert 0.0 < o["relief_lp"] < prev, \
            ("lag ERODES the LP relief, never enhances it", tau, o["relief_lp"], prev)
        assert o["overshoot"] > 100.0, ("and it overshoots hugely at fast r", tau, o["overshoot"])
        prev = o["relief_lp"]


# ======================================================================================
# THE SECONDARY — the overshoot lives in the LOOP lag, not the valve (the monotone command)
# ======================================================================================

def test_valve_lag_inert_topping_command_monotone():
    """A pure metering-VALVE-position lag is INERT on the accel because the binding topping
    command RISES monotonically (an instant-up valve tracks it with no lag). The topping-command
    trace over the engaged window is monotone non-decreasing -- so the topping OVERSHOOT lives in
    the sensing/limiter-LOOP lag (which lags the clip AMOUNT), not the valve. WHERE the lag lives
    decides whether it overshoots."""
    ft = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0)
    t = ft.topping_command_trace(FLIGHT, LO, HI, REDLINE, r=R, s_settle=SETTLE)
    assert t["n_engaged"] > 10, ("the clip must engage over a real window", t["n_engaged"])
    assert t["monotone_nondecreasing"], "the binding topping command must rise monotonically"
    assert t["engaged"][-1][1] > t["engaged"][0][1], "the command genuinely rises across the window"


if __name__ == "__main__":
    for fn in (test_reduce_tau_none_bit_for_bit_rung46,
               test_reduce_dormant_lag_bit_for_bit_rung45,
               test_reduce_lp_disabled_and_tau_needs_redline_assert,
               test_decel_lagged_bit_for_bit_rung45,
               test_cycle_untouched_by_lagged_governor_bit_for_bit_rung6,
               test_lagged_governor_overshoots_erodes_hp_and_misses_lp,
               test_overshoot_grows_and_hp_erodes_monotone_in_tau,
               test_fast_ramp_lp_relief_eroded_by_lag_never_enhanced,
               test_valve_lag_inert_topping_command_monotone):
        fn()
        print("PASS", fn.__name__)
