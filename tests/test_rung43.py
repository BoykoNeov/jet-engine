"""Rung 43 — TWO-SHAFT FUEL METERING: the two spools sit at DIFFERENT points in ONE
overshoot loop.

Gates (named in docs/rung43-spec.md § Verification gates):

   1. REDUCE / NON-TAUTOLOGICAL — CONTROL-INVARIANCE. Feeding f_eq*mdot_air_eq of a
      rung-40 Tt4-control point to `equilibrium_fuel` reproduces that point to machine
      zero, via the forward-BURNER closure (a genuinely different code path). This is
      also the gate that empirically kills the withdrawn "fuel metering breaks rung 39's
      (dagger)" framing: same manifold, different knob.
   2. REDUCE — lp_disabled EXACT DISPATCH to rung 35's SpoolTransient fuel path (==).
   3. REDUCE — Tt4-control UNTOUCHED => rung 40 bit-for-bit (==); rung 40's own `_close`
      is literally unchanged, so the rung 31-42 suites pass unchanged.
   4. REDUCE — SETTLE: a fuel ramp marched long lands ON the target equilibrium.
   5. FINDING — THE MECHANISM: freezing EITHER spool WORSENS the overshoot (both sit in
      the one loop), with the asserted CONTRAST that the share trades with rho. Sign and
      existence only; no calibrated split.
   6. FINDING — THE CEILING: the LP-frozen march is rho-independent BIT-FOR-BIT (rho
      multiplies only the LP ODE), and X(rho) rises monotonically toward it.
   7. FINDING — rho-MONOTONICITY of the overshoot, >=3 shape pairs x 2 ramp durations.
      Sign only; magnitudes disclaimed.
   8. INHERITED — TIT-limited before surge (rung 35), re-measured on two shafts.
   9. THE WITHDRAWN CLAIMS, asserted as such (rung 40's gate-7 move): the best-fit
      effective-clock exponent DIFFERS across currencies (they read back their own
      denominator) and no currency collapses below ~10%, so "the overshoot collapses on
      an effective clock" cannot silently creep back.
  10. CYCLE UNTOUCHED — the default single-spool design run is bit-for-bit rung 6.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    SpoolTransient, TwoSpoolTransient, TwoSpoolFuelTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
SINGLE = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.92,
              eta_m=0.99, pi_n=0.98, nozzle_convergent=True)

LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
SHAPES = {
    "flow/press": (LP_SHAPED, HP_SHAPED),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85),
                   ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)),
}
LO, HI = 1250.0, 1450.0          # rung 35's own step — apples-to-apples


def _cpg_gas():
    """Self-consistent CPG dual gas (the rung 31/38/39/40 recipe)."""
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _ft(gas, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0):
    return TwoSpoolFuelTransient(_design(gas), FLIGHT, 1.0, map_lp=map_lp,
                                 map_hp=map_hp, rho=rho)


# --------------------------------------------------------------------------- gate 1
def test_reduce_control_invariance_is_rung40_point():
    """GATE 1 — a steady point is the SAME however it is named.

    NON-TAUTOLOGICAL: `equilibrium_fuel` reaches it through the forward BURNER
    (Tt4 an OUTPUT of f = mdot_fuel/mdot_air) and a 2-D Newton on the fuel closure —
    it never calls the Tt4-control path. Two closures, one point.

    This is also the empirical death of the withdrawn framing "fuel metering breaks
    rung 39's (dagger) cancellation and re-couples LP into the HP core": if the two
    controls land on the same manifold, the control knob cannot change the coupling.
    """
    ft = _ft(_cpg_gas())
    for Tt4 in (1500.0, 1300.0, 1100.0):
        eq = ft.equilibrium(FLIGHT, Tt4)
        mf = eq["f"] * eq["mdot_air"]
        fq = ft.equilibrium_fuel(FLIGHT, mf)
        assert abs(fq["nu_lp"] / eq["nu_lp"] - 1.0) < 1e-12, (Tt4, fq["nu_lp"])
        assert abs(fq["nu_hp"] / eq["nu_hp"] - 1.0) < 1e-12, (Tt4, fq["nu_hp"])
        assert abs(fq["Tt4"] / Tt4 - 1.0) < 1e-12, (Tt4, fq["Tt4"])
        assert abs(fq["pi_lpc"] / eq["pi_lpc"] - 1.0) < 1e-11, (Tt4, fq["pi_lpc"])
        assert abs(fq["pi_hpc"] / eq["pi_hpc"] - 1.0) < 1e-11, (Tt4, fq["pi_hpc"])
        # and the residuals really are zero (not merely the speeds agreeing)
        assert abs(fq["Phi_lp"]) < 1e-9 and abs(fq["Phi_hp"]) < 1e-9


# --------------------------------------------------------------------------- gate 2
def test_reduce_lp_disabled_is_rung35_fuel_path_bit_for_bit():
    """GATE 2 — EXACT DISPATCH. lp_disabled builds no two-shaft state at all; the fuel
    methods forward to the held rung-35 SpoolTransient, so the fields compare ==."""
    gas = _cpg_gas()
    single = build_turbojet(gas, PI_HPC, TT4, FLIGHT.p0, **SINGLE)
    st = SpoolTransient(single, FLIGHT, 1.0, comp_map=HP_SHAPED)
    deg = TwoSpoolFuelTransient(single, FLIGHT, 1.0, map_lp=LP_SHAPED,
                                map_hp=HP_SHAPED, lp_disabled=True)
    for Tt4 in (1500.0, 1300.0, 1150.0):
        mf = st._fuel_for_Tt4(FLIGHT, Tt4)
        a, b = st.equilibrium_fuel(FLIGHT, mf), deg.equilibrium_fuel(FLIGHT, mf)
        for k in ("nu", "pi_c", "Tt4", "mdot_air", "f", "tau_t", "sp_thrust"):
            assert a[k] == b[k], (Tt4, k, a[k], b[k])


# --------------------------------------------------------------------------- gate 3
def test_reduce_tt4_control_untouched_is_rung40_bit_for_bit():
    """GATE 3 — rung 40's Tt4 control is inherited UNCHANGED. Building and exercising
    the fuel control must not perturb it."""
    gas = _cpg_gas()
    design = _design(gas)
    t40 = TwoSpoolTransient(design, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED)
    ft = TwoSpoolFuelTransient(design, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED)
    ft.constant_speed_excursion_fuel(FLIGHT, LO, HI)          # exercise the new path
    for Tt4 in (1500.0, 1300.0, 1150.0):
        a, b = t40.equilibrium(FLIGHT, Tt4), ft.equilibrium(FLIGHT, Tt4)
        for k in ("nu_lp", "nu_hp", "pi_lpc", "pi_hpc", "Tt4", "mdot_air", "f",
                  "tau_hpt", "tau_lpt", "sp_thrust"):
            assert a[k] == b[k], (Tt4, k)


# --------------------------------------------------------------------------- gate 4
def test_reduce_settle_lands_on_the_equilibrium():
    """GATE 4 — the DYNAMICAL reduce: hold the fuel at its high value and march; the
    trajectory relaxes onto the matched two-shaft equilibrium."""
    ft = _ft(_cpg_gas())
    mf_hi = ft.fuel_for_Tt4(FLIGHT, HI)
    eq_hi, eq_lo = ft.equilibrium(FLIGHT, HI), ft.equilibrium(FLIGHT, LO)
    traj = ft.integrate_fuel(FLIGHT, lambda s: mf_hi,
                             (eq_lo["nu_lp"], eq_lo["nu_hp"]), 14.0, 0.02)
    last = traj[-1]
    assert last["s"] > 13.0, last["s"]
    assert abs(last["nu_lp"] / eq_hi["nu_lp"] - 1.0) < 1e-6, last["nu_lp"]
    assert abs(last["nu_hp"] / eq_hi["nu_hp"] - 1.0) < 1e-6, last["nu_hp"]
    assert abs(last["Tt4"] / HI - 1.0) < 1e-5, last["Tt4"]


# --------------------------------------------------------------------------- gate 5
def test_finding_mechanism_both_spools_relieve_the_overshoot():
    """GATE 5 — THE MECHANISM (the rung).

    f = mdot_fuel/mdot_air is set at the LP FACE, but the Tt4 it produces is metered
    back through the HP-FED NGV choke: the two spools sit at DIFFERENT points in the ONE
    overshoot loop. Freezing EITHER spool therefore makes the overshoot WORSE — neither
    is a bystander, which is WHY no single spool's clock can govern it.

    SIGN / EXISTENCE ONLY. d_lp and d_hp do not sum to the total and are NOT calibrated
    weights; only their positivity, and the DIRECTION in which the share trades with
    rho, are asserted.
    """
    gas = _cpg_gas()
    seen = []
    for name in ("flow/press", "tilted"):
        ml, mh = SHAPES[name]
        for rho in (0.5, 1.0, 2.0):
            ft = _ft(gas, ml, mh, rho)
            for r in (0.25, 1.0):
                fc = ft.freeze_channels(FLIGHT, LO, HI, r)
                assert fc["d_lp"] > 0.0, (name, rho, r, fc)
                assert fc["d_hp"] > 0.0, (name, rho, r, fc)
                seen.append((name, r, rho, fc["d_lp"], fc["d_hp"]))

    # THE CONTRAST: the share trades with rho — as the LP spool slows, the LP channel's
    # relief SHRINKS and the HP channel's GROWS. Asserted as a direction on the ratio,
    # never as weights.
    for name in ("flow/press", "tilted"):
        for r in (0.25, 1.0):
            row = sorted((rho, dl, dh) for (n, rr, rho, dl, dh) in seen
                         if n == name and rr == r)
            ratios = [dl / dh for (_, dl, dh) in row]
            assert ratios == sorted(ratios, reverse=True), (name, r, row)


# --------------------------------------------------------------------------- gate 6
def test_finding_lp_frozen_is_the_rho_free_ceiling():
    """GATE 6 — THE CEILING. rho multiplies ONLY the LP ODE (dnu_L/ds = Phi_L/rho), so
    rho -> infinity IS the LP-frozen system: the LP-frozen march is rho-independent
    BIT-FOR-BIT, and the measured overshoot rises monotonically toward it.

    This is what turns the rho-monotonicity (gate 7) from a bare sign into a BOUNDED
    claim: the worst TIT excursion a heavy LP spool can produce is computable without
    marching the LP spool at all."""
    gas = _cpg_gas()
    for name in ("flow/press", "tilted"):
        ml, mh = SHAPES[name]
        for r in (0.25, 1.0):
            # rho-freeness of the ceiling: bit-for-bit across very different rho
            ceil = [_ft(gas, ml, mh, rho).ramp_excursion_fuel(
                FLIGHT, LO, HI, r, freeze="lp")["X"] for rho in (1.0, 7.0, 50.0)]
            assert ceil[0] == ceil[1] == ceil[2], (name, r, ceil)
            # and X(rho) climbs toward it from BELOW, monotonically
            xs = [_ft(gas, ml, mh, rho).ramp_excursion_fuel(FLIGHT, LO, HI, r)["X"]
                  for rho in (1.0, 8.0, 32.0)]
            assert xs == sorted(xs), (name, r, xs)
            assert xs[-1] < ceil[0], (name, r, xs[-1], ceil[0])
            assert xs[-1] > 0.90 * ceil[0], (name, r, xs[-1], ceil[0])


# --------------------------------------------------------------------------- gate 7
def test_finding_overshoot_rises_monotonically_with_rho():
    """GATE 7 — a heavier LP spool worsens the TIT excursion, because the LP-FACE
    airflow lag is what spikes f. SIGN ONLY across >=3 shape pairs x 2 ramp durations;
    every magnitude rides on rho, the maps, the step and the band (disclaimed)."""
    gas = _cpg_gas()
    for name, (ml, mh) in SHAPES.items():
        for r in (0.25, 1.0):
            xs = []
            for rho in (0.25, 0.5, 1.0, 2.0, 4.0):
                e = _ft(gas, ml, mh, rho).ramp_excursion_fuel(FLIGHT, LO, HI, r)
                assert e["complete"], (name, r, rho)
                xs.append(e["X"])
            assert xs == sorted(xs), (name, r, xs)
            assert xs[-1] > xs[0], (name, r, xs)


# --------------------------------------------------------------------------- gate 8
def test_inherited_tit_limited_before_surge():
    """GATE 8 — INHERITED from rung 35, re-measured on two shafts (NOT this rung's
    finding): the TIT excursion dwarfs the surge excursion on both spools, so the
    acceleration is temperature-limited before it is surge-limited on these maps.

    Also witnesses the r -> 0 step being EXACTLY rho-free (a pure algebraic map
    property — rung 34/35's argument doubled), which is the r_eff -> 0 endpoint of the
    ramp family rather than a separate object.

    The multiple is DISCLOSED, not tuned: measured 4.41x (flow/press), 6.33x (press/flow),
    5.21x (tilted), so the gate asserts >4x. It is an ORDERING claim on these maps, in the
    rung-32/35 register — which limit binds first is map-dependent, and no TIT redline is
    modelled."""
    gas = _cpg_gas()
    for name, (ml, mh) in SHAPES.items():
        cs = _ft(gas, ml, mh).constant_speed_excursion_fuel(FLIGHT, LO, HI)
        assert cs["E_temp"] > 4.0 * max(cs["E_lp"], cs["E_hp"]), (name, cs)
        # exactly rho-free: both spools are frozen, so no clock can enter
        a = _ft(gas, ml, mh, 0.2).constant_speed_excursion_fuel(FLIGHT, LO, HI)
        b = _ft(gas, ml, mh, 5.0).constant_speed_excursion_fuel(FLIGHT, LO, HI)
        for k in ("Tt4_peak", "E_temp", "E_lp", "E_hp", "f"):
            assert a[k] == b[k], (name, k, a[k], b[k])


# --------------------------------------------------------------------------- gate 9
def test_withdrawn_no_effective_clock_ratio():
    """GATE 9 — THE WITHDRAWN CLAIM, asserted as such (rung 40's gate-7 move).

    Rung 43 deliberately claims NO effective clock ratio r_eff = r/rho^q. Two facts are
    asserted so the tempting reading cannot creep back:

      (a) the referenced currencies are CIRCULAR — the best-fit q READS BACK whichever
          spool sits in the denominator, so E_temp_H's q ~ 0 was never evidence that
          "the HP clock governs";
      (b) even on the spool-neutral X there is NO collapse — the best exponent cuts the
          spread ~4.9x against the q=0 endpoint but bottoms out near 14%, i.e. points a
          real effective clock would place on ONE curve still differ by a seventh.

    Together these kill both the "geometric-mean composite clock" reading and the
    "slow spool rate-limits it" reading. Note in particular that NOTHING about the
    exponent is currency-independent.
    """
    gas = _cpg_gas()
    ml, mh = SHAPES["flow/press"]
    pts = []
    for rho in (0.25, 1.0, 4.0, 8.0):
        ft = _ft(gas, ml, mh, rho)
        for r in (0.25, 0.5, 1.0, 2.0):
            e = ft.ramp_excursion_fuel(FLIGHT, LO, HI, r)
            if e["complete"]:
                pts.append((r, rho, e))
    assert len(pts) >= 12, len(pts)

    qH, _ = TwoSpoolFuelTransient.collapse_exponent(pts, "E_temp_H")
    qX, sX = TwoSpoolFuelTransient.collapse_exponent(pts, "X")
    qL, _ = TwoSpoolFuelTransient.collapse_exponent(pts, "E_temp_L")

    # (a) CIRCULARITY: the exponent tracks the denominator, HP -> none -> LP.
    assert qH < qX < qL, (qH, qX, qL)
    assert qL - qH > 0.3, (qH, qX, qL)

    # (b) NO COLLAPSE on the spool-neutral currency.
    assert sX > 0.10, sX

    # ... and the neutral currency's own best exponent is INTERIOR — it matches neither
    # single-spool clock (q=0 "HP governs" nor q=1 "LP governs"). This is the ONLY
    # exponent statement rung 43 makes, and it is made only on X; note it is NOT a
    # refutation of q=1 in general, since on X the q=0 fit is the worse of the two.
    assert 0.0 < qX < 1.0, qX


# -------------------------------------------------------------------------- gate 10
def test_cycle_untouched_rung6_bit_for_bit():
    """GATE 10 — the default single-spool design run is untouched by rung 43 (the
    rungs-7+ invariant): building AND marching the fuel transient must not perturb it."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    ft = _ft(_cpg_gas())
    ft.constant_speed_excursion_fuel(FLIGHT, LO, HI)
    ft.freeze_channels(FLIGHT, LO, HI, 0.25)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far
    assert a.stations["9"].pt == b.stations["9"].pt


# ------------------------------------------------------------------ scope / concession
def test_reacting_gas_fuel_control_is_refused():
    """CONCESSION (rung 35's, carried verbatim): the forward burner is built for the
    NON-equilibrium gas and must REFUSE an equilibrium one rather than mis-solve. The
    reacting reduce is the Tt4-control path, which still works."""
    ft = _ft(Gas.reacting_equilibrium())
    try:
        ft._tt4_from_f(700.0, 0.025)
        raise AssertionError("rung-43 forward burner accepted an equilibrium gas")
    except AssertionError as exc:
        assert "non-equilibrium" in str(exc), str(exc)
    # the reacting Tt4-control path (rung 40) is unaffected
    eq = ft.equilibrium(FLIGHT, 1400.0)
    assert eq["nu_lp"] > 0.0 and eq["pi_lpc"] > 1.0


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"  {name} OK")
