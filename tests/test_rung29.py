"""Rung 29 — THE SHIFTING TURBINE: is the frozen turbine EARNED?

Gates (named in docs/rung29-spec.md § Verification gates):

  1. REDUCE TO PRIOR (the spine) — the frozen branch of the bracket IS the shipped
     Turbine at eta_t=1, bit-for-bit. Structural: `Gas.shifting_turbine` takes the two
     `Turbine.apply` lines verbatim, so this gate certifies the delegation, and the
     independent work-limited SOLVER is checked against it separately (gate 2).
  2. THE SOLVER IS RIGHT — the independent `_work_limited_expand(shifting=False)`
     bisection agrees with that closed form. Without this, gate 1 is a tautology.
  3. CYCLE UNTOUCHED — calling the diagnostic does not perturb the cycle (bit-for-bit
     rung 6, the rungs-7+ invariant).
  4. THE VERDICT — the frozen turbine is EARNED at the design point and BITES HOT.
  5. THE INVERSION (ratio != energy) — the super-equilibrium RATIO falls with Tt4 while
     the shift it is supposed to justify RISES. The anti-correlation is the rung.
  6. DIRECTION — recombination reheats: the shifting exit is warmer and at higher
     pressure, at equal shaft work.
"""
import math
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import (  # noqa: E402
    Gas, _equilibrium_composition, _mix_mass_per_air, _work_limited_expand,
)
from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
REAL_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
                   eta_t=0.90, eta_m=0.99, pi_n=0.98)


def _run(Tt4, pi_c=PI_C):
    """Design-point run at Tt4 + the shaft-set delta_h the Engine hands its Turbine."""
    gas = Gas.reacting_equilibrium()
    r = build_turbojet(gas, pi_c, Tt4, FLIGHT.p0, **REAL_LOSSES).run(FLIGHT, 1.0)
    far = r.stations["4"].far
    delta_h = (gas.h_c(r.stations["3"].Tt) - gas.h_c(r.stations["2"].Tt)) / (
        REAL_LOSSES["eta_m"] * (1.0 + far))
    return gas, r, far, delta_h


_BRACKET_CACHE = {}


def _bracket(Tt4, pi_c):
    """The shifting-turbine bracket at (Tt4, pi_c), memoised (the pi_c gates re-read points)."""
    key = (Tt4, pi_c)
    if key not in _BRACKET_CACHE:
        gas, r, far, delta_h = _run(Tt4, pi_c)
        s4 = r.stations["4"]
        _BRACKET_CACHE[key] = (gas, far, gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h))
    return _BRACKET_CACHE[key]


def _completion(Tt4, pi_c):
    """Fraction of the entry radical inventory that equilibrium at the FROZEN exit state wants gone —
    how much of the pool the expansion actually asks for (docs/rung29-pi-c-margin.md)."""
    _, far, st = _bracket(Tt4, pi_c)
    c5 = _equilibrium_composition(far, st.T5_frozen, st.p5_frozen)
    inv5 = sum(c5.get(s, 0.0) for s in ("O", "H", "OH")) / sum(c5.values())
    return 1.0 - inv5 / st.radical_inventory


def test_reduce_to_prior_frozen_is_the_shipped_turbine():
    """GATE 1 — the bracket's frozen bound IS the shipped Turbine at eta_t=1, bit-for-bit."""
    for Tt4 in (1500.0, 1800.0, 2100.0, 2400.0):
        gas, r, far, delta_h = _run(Tt4)
        s4 = r.stations["4"]
        st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
        # The shipped path, recomputed here independently of gas.py's delegation.
        Tt5 = gas.T_from_h_t(gas.h_t(s4.Tt, far) - delta_h, far)
        pt5 = s4.pt * gas.pr_t(Tt5, far) / gas.pr_t(s4.Tt, far)
        assert st.T5_frozen == Tt5, f"Tt4={Tt4}: frozen bound != shipped turbine T (bit-for-bit)"
        assert st.p5_frozen == pt5, f"Tt4={Tt4}: frozen bound != shipped turbine p (bit-for-bit)"


def test_work_limited_solver_agrees_with_the_closed_form():
    """GATE 2 — the independent bisection reproduces the closed form, so gate 1 is not a tautology.

    The solver marches entropy and ABSOLUTE enthalpy over the mixture; the closed form goes through
    h_t/pr_t. They are different code paths onto the same physics, so agreement to ~1e-6 (the solver's
    own bisection tolerance) certifies the work-limited construction itself.
    """
    for Tt4 in (1500.0, 2100.0):
        gas, r, far, delta_h = _run(Tt4)
        s4 = r.stations["4"]
        comp4 = _equilibrium_composition(far, s4.Tt, s4.pt)
        m = _mix_mass_per_air(comp4)
        T5, p5, _ = _work_limited_expand(comp4, far, s4.Tt, s4.pt, delta_h * m, shifting=False)
        st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
        assert abs(T5 - st.T5_frozen) < 1e-6 * st.T5_frozen, \
            f"Tt4={Tt4}: work-limited solver {T5} != closed form {st.T5_frozen}"
        assert abs(p5 - st.p5_frozen) < 1e-6 * st.p5_frozen, \
            f"Tt4={Tt4}: work-limited solver {p5} != closed form {st.p5_frozen}"


def test_cycle_untouched():
    """GATE 3 — a shifting_turbine call is a pure OBSERVER: the cycle is bit-for-bit rung 6."""
    gas, r, far, delta_h = _run(1500.0)
    s4, s5 = r.stations["4"], r.stations["5"]
    before = (s5.Tt, s5.pt, r.V9, r.performance.specific_thrust)
    gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
    r2 = build_turbojet(Gas.reacting_equilibrium(), PI_C, 1500.0, FLIGHT.p0,
                        **REAL_LOSSES).run(FLIGHT, 1.0)
    after = (r2.stations["5"].Tt, r2.stations["5"].pt, r2.V9, r2.performance.specific_thrust)
    assert before == after, "the rung-29 diagnostic perturbed the cycle"


def test_verdict_earned_at_design_bites_hot():
    """GATE 4 — the headline. Even the MAXIMUM shift is negligible at the design point; it is not hot.

    Rate-independent: this is the instant-chemistry, reversible bound, so no tau_res and no knob can
    make the real turbine exceed it.
    """
    gas, r, far, delta_h = _run(1500.0)
    s4 = r.stations["4"]
    st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
    assert st.frozen_turbine_earned, "design point: freezing the turbine should be EARNED"
    assert abs(st.dT5_fraction) < 2e-4, \
        f"design-point bound drifted: dT5/T5 = {st.dT5_fraction:.3e} (expected ~1.1e-4)"

    gas, r, far, delta_h = _run(2400.0)
    s4 = r.stations["4"]
    st_hot = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
    assert not st_hot.frozen_turbine_earned, "Tt4=2400 K: the freeze should NOT be earned"
    assert st_hot.dT5_fraction > 1e-2, \
        f"hot bound too small: dT5/T5 = {st_hot.dT5_fraction:.3e} (expected ~1.9e-2)"
    # The bound grows by more than two orders across the band — the reason "earned at the design
    # point" cannot be quoted as "earned".
    assert st_hot.dT5_fraction / st.dT5_fraction > 100.0


def test_the_inversion_ratio_is_not_energy():
    """GATE 5 — THE RUNG. The super-eq RATIO and the shift it is meant to justify move OPPOSITE ways.

    Rungs 25-28 quote the ratio as evidence the entry is far from equilibrium. It is a correct measure
    of KINETIC super-equilibrium, but not of exploitable enthalpy: that scales with the absolute
    radical INVENTORY. Across the Tt4 band the ratio FALLS while the inventory (and the shift) RISE, so
    the ratio is loudest exactly where the shift is most negligible.
    """
    ratios, shifts, inventories = [], [], []
    for Tt4 in (1500.0, 1800.0, 2100.0, 2400.0):
        gas, r, far, delta_h = _run(Tt4)
        s4 = r.stations["4"]
        st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
        ratios.append(st.super_eq_ratio_max)
        shifts.append(st.dT5_fraction)
        inventories.append(st.radical_inventory)

    # Strictly monotone, in opposite directions — the anti-correlation is the claim.
    assert all(a > b for a, b in zip(ratios, ratios[1:])), \
        f"super-eq ratio should FALL with Tt4: {ratios}"
    assert all(a < b for a, b in zip(shifts, shifts[1:])), \
        f"the shift should RISE with Tt4: {shifts}"
    assert all(a < b for a, b in zip(inventories, inventories[1:])), \
        f"radical inventory should RISE with Tt4 (the real predictor): {inventories}"
    # And the divergence is large, not a marginal crossing: the ratio falls ~30x while the shift
    # rises ~150x, so no reading of the ratio recovers the shift.
    assert ratios[0] / ratios[-1] > 10.0
    assert shifts[-1] / shifts[0] > 100.0


def test_direction_recombination_reheats():
    """GATE 6 — at equal shaft work the shifting exit is WARMER and at HIGHER pressure.

    The chemical energy pays part of the shaft's bill, so less pressure drop is needed. (Also asserted
    inside `shifting_turbine` on every call — contract #4.)
    """
    for Tt4 in (1500.0, 1800.0, 2100.0, 2400.0):
        gas, r, far, delta_h = _run(Tt4)
        s4 = r.stations["4"]
        st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
        assert st.dT5 > 0.0, f"Tt4={Tt4}: shifting exit should be warmer"
        assert st.dp5_fraction > 0.0, f"Tt4={Tt4}: shifting exit should be at higher pressure"
        assert st.delta_h == delta_h


# ---------------------------------------------------------------------------------------------
# The pi_c margin — rung 29's "one design point" concession, re-checked on the axis it named.
# A CONFIRMATION plus a SHARPENING, not a rung: docs/rung29-pi-c-margin.md.
# ---------------------------------------------------------------------------------------------

PI_C_SCAN = (2.0, 5.0, 10.0, 20.0, 80.0)


def test_earned_at_design_is_pi_c_robust():
    """The verdict survives the pi_c axis, with margin — and the boundary stays far above design.

    "Earned at design" was shipped from a single pi_c=10. Over pi_c 2..80 the design-point bound
    never exceeds ~0.0107% (9x under the 1e-3 threshold), and the earned/not-earned boundary Tt4*
    is bracketed 1800 < Tt4* < 2200 everywhere -- measured where dT5 ~ 1 K, well clear of any
    solver-noise question, so "far above the design point" cannot drift into an artifact.
    """
    for pi_c in PI_C_SCAN:
        _, _, st = _bracket(1500.0, pi_c)
        assert st.frozen_turbine_earned, f"pi_c={pi_c}: design point should stay EARNED"
        assert abs(st.dT5_fraction) < 2e-4, \
            f"pi_c={pi_c}: design bound drifted: dT5/T5 = {st.dT5_fraction:.3e} (expected <=1.07e-4)"
        # The boundary bracket: still earned at 1800 K, no longer earned at 2200 K, at EVERY pi_c.
        assert _bracket(1800.0, pi_c)[2].frozen_turbine_earned, \
            f"pi_c={pi_c}: Tt4=1800 K should still be earned (Tt4* > 1800)"
        assert not _bracket(2200.0, pi_c)[2].frozen_turbine_earned, \
            f"pi_c={pi_c}: Tt4=2200 K should NOT be earned (Tt4* < 2200)"


def test_pi_c_channels_oppose():
    """The mechanism: raising pi_c CUTS the inventory but RAISES how much of it is spent.

    (a) higher pt4 suppresses dissociation (more moles on the dissociated side), so the entry
        radical inventory falls monotonically; (b) higher pi_c means a larger shaft-set delta_h,
        hence a deeper and colder expansion, so equilibrium at the exit asks for a monotonically
        larger FRACTION of that pool. Two opposed, comparable channels -- which is what makes the
        shift's pi_c dependence non-monotone.
    """
    inv = [_bracket(1500.0, p)[2].radical_inventory for p in PI_C_SCAN]
    comp = [_completion(1500.0, p) for p in PI_C_SCAN]
    assert all(a > b for a, b in zip(inv, inv[1:])), \
        f"entry radical inventory should FALL with pi_c (pressure suppresses dissociation): {inv}"
    assert all(a < b for a, b in zip(comp, comp[1:])), \
        f"completion should RISE with pi_c (deeper, colder expansion): {comp}"
    # Comparable magnitudes -- neither channel is a rounding correction to the other.
    assert 2.0 < inv[0] / inv[-1] < 6.0
    assert 2.0 < comp[-1] / comp[0] < 4.0


def test_pi_c_is_not_simply_protective():
    """FORBID the beta-style reading. Unlike rung 28's beta -- which fell monotonically in pi_c --
    the shift TURNS OVER: it rises from pi_c=2 to ~10 and falls again out to 80. Asserted at
    Tt4=1800, where the turnover is a 1.9x effect and no solver-noise question arises.
    """
    lo = _bracket(1800.0, 2.0)[2].dT5_fraction
    mid = _bracket(1800.0, 10.0)[2].dT5_fraction
    hi = _bracket(1800.0, 80.0)[2].dT5_fraction
    assert lo < mid, f"shift should RISE from pi_c=2 to 10 (so pi_c is not protective): {lo} {mid}"
    assert hi < mid, f"shift should FALL from pi_c=10 to 80 (an INTERIOR maximum): {hi} {mid}"
    assert mid / lo > 1.5, f"the low-side rise should be substantial, not marginal: {mid / lo}"


def test_inventory_alone_fails_on_the_pi_c_axis():
    """THE SHARPENING. Rung 29 proposed the absolute radical INVENTORY as the currency that the
    super-eq RATIO failed to be. On the Tt4 axis that reads correctly (inventory swings two orders
    and dominates). On the pi_c axis it does not: inventory FALLS while the shift RISES -- the same
    failure mode, now committed by the replacement. The complete currency is inventory x COMPLETION
    (the RECOMBINED inventory), which is what the two opposed channels above multiply out to.
    """
    _, _, lo = _bracket(1500.0, 2.0)
    _, _, hi = _bracket(1500.0, 10.0)
    assert hi.radical_inventory < lo.radical_inventory, "inventory should fall from pi_c=2 to 10"
    assert hi.dT5_fraction > lo.dT5_fraction, "yet the shift should RISE -- inventory alone fails"
    # The recombined inventory does track it: it rises over the same span, unlike the entry pool.
    rec_lo = lo.radical_inventory * _completion(1500.0, 2.0)
    rec_hi = hi.radical_inventory * _completion(1500.0, 10.0)
    assert rec_hi > rec_lo, f"recombined inventory should RISE with the shift: {rec_lo} {rec_hi}"


# ---------------------------------------------------------------------------------------------
# The M0 / flight-axis margin — rung 29's LAST "one design point" concession (after the pi_c
# check), re-checked on the axis it named. A CONFIRMATION plus a CORRECTION to the pi_c doc's
# unification framing, not a rung: docs/rung29-M0-margin.md.
#
# Verdict, opposite of pi_c: the shift is MONOTONE-PROTECTIVE in M0 (no interior turnover), the
# bracket's beta-like axis. Same INVENTORY x COMPLETION currency, read where it is lopsided
# (completion near-saturated). The discriminator between turnover (pi_c) and monotone (M0) is the
# delta_h SWING, not completion headroom -- proven by the pi_c=2 control. And the flight axis is
# double-edged: protective per point, yet ram heating shrinks the earned OPERATING band.
# ---------------------------------------------------------------------------------------------

M0_SCAN = (0.3, 0.85, 1.6, 2.5, 3.0)
_M0_CACHE = {}


def _run_M0(Tt4, M0, pi_c=PI_C):
    """A run at (Tt4, M0, pi_c), same ambient/losses -- or None if the cycle does not solve
    (the low-M0 ram edge and the high-Tt4 equilibrium-burner ceiling; neither is the turbine)."""
    try:
        gas = Gas.reacting_equilibrium()
        flight = FlightCondition(T0=FLIGHT.T0, p0=FLIGHT.p0, M0=M0)
        r = build_turbojet(gas, pi_c, Tt4, FLIGHT.p0, **REAL_LOSSES).run(flight, 1.0)
        far = r.stations["4"].far
        s2, s3, s4 = r.stations["2"], r.stations["3"], r.stations["4"]
        delta_h = (gas.h_c(s3.Tt) - gas.h_c(s2.Tt)) / (REAL_LOSSES["eta_m"] * (1.0 + far))
        return gas, far, s4.Tt, s4.pt, delta_h
    except Exception:
        return None


def _bracket_M0(Tt4, M0, pi_c=PI_C):
    key = (Tt4, M0, pi_c)
    if key not in _M0_CACHE:
        out = _run_M0(Tt4, M0, pi_c)
        if out is None:
            _M0_CACHE[key] = None
        else:
            gas, far, tt4, pt4, delta_h = out
            _M0_CACHE[key] = (gas, far, delta_h, gas.shifting_turbine(far, tt4, pt4, delta_h))
    return _M0_CACHE[key]


def _completion_M0(Tt4, M0, pi_c=PI_C):
    gas, far, _, st = _bracket_M0(Tt4, M0, pi_c)
    c5 = _equilibrium_composition(far, st.T5_frozen, st.p5_frozen)
    inv5 = sum(c5.get(s, 0.0) for s in ("O", "H", "OH")) / sum(c5.values())
    return 1.0 - inv5 / st.radical_inventory


def test_M0_helper_reproduces_the_certified_flight_anchor():
    """Gate the new _bracket_M0 helper against the certified FLIGHT path -- my own gate-2 principle:
    an independent code path must reproduce the anchor, else the RELATIVE monotonicity gates below
    could read 'monotone' on wrong numbers (a wiring bug in FlightCondition/delta_h would survive
    them). _bracket_M0(1500, M0=0.85) builds the SAME flight condition (T0=250, p0=50 kPa, M0=0.85),
    pi_c and losses as the shipped _bracket(1500, 10), so the two must agree bit-for-bit AND hit the
    rung-29 anchor 0.01067%.
    """
    ref = _bracket(1500.0, PI_C)[2]        # certified FLIGHT path (M0=0.85)
    got = _bracket_M0(1500.0, 0.85)[3]     # the M0 helper at the same point
    assert got.dT5_fraction == ref.dT5_fraction, \
        f"M0 helper must reproduce the FLIGHT path bit-for-bit: {got.dT5_fraction} != {ref.dT5_fraction}"
    assert got.T5_frozen == ref.T5_frozen and got.T5_shifting == ref.T5_shifting, \
        "M0 helper must reproduce the FLIGHT exit state bit-for-bit"
    assert abs(got.dT5_fraction * 100 - 0.01067) < 5e-5, \
        f"must hit the rung-29 anchor 0.01067%: {got.dT5_fraction*100:.5f}%"


def test_earned_at_design_is_M0_robust():
    """The verdict survives the M0 axis, with MORE margin than pi_c -- and the worst case is
    low-M0 takeoff, not the design cruise point.

    Over M0 0.3..3.0 the design-point (Tt4=1500) bound never exceeds ~0.0113% (8.8x under the 1e-3
    threshold), and the earned/not-earned boundary is bracketed 1800 < Tt4* < 2200 everywhere.
    """
    for M0 in M0_SCAN:
        st = _bracket_M0(1500.0, M0)[3]
        assert st.frozen_turbine_earned, f"M0={M0}: design point should stay EARNED"
        assert abs(st.dT5_fraction) < 2e-4, \
            f"M0={M0}: design bound drifted: dT5/T5 = {st.dT5_fraction:.3e} (expected <=1.13e-4)"
        assert _bracket_M0(1800.0, M0)[3].frozen_turbine_earned, \
            f"M0={M0}: Tt4=1800 K should still be earned (Tt4* > 1800)"
        assert not _bracket_M0(2200.0, M0)[3].frozen_turbine_earned, \
            f"M0={M0}: Tt4=2200 K should NOT be earned (Tt4* < 2200)"


def test_M0_shift_is_monotone_protective():
    """THE DIFFERENTIATOR, sign-flipped from pi_c's turnover. Unlike pi_c -- whose shift humped with
    an interior maximum -- the M0 shift falls MONOTONICALLY (the bracket's beta-like axis). Asserted
    at a HOT Tt4=2100 (a 2.1x swing) so no low-Tt4 solver-noise question arises.
    """
    fr = [_bracket_M0(2100.0, M0)[3].dT5_fraction for M0 in M0_SCAN]
    assert all(a > b for a, b in zip(fr, fr[1:])), \
        f"shift must fall monotonically in M0 (protective, NOT a turnover): {fr}"
    assert fr[0] / fr[-1] > 1.8, f"the fall should be substantial across the axis: {fr[0]/fr[-1]}"


def test_M0_channels_are_lopsided():
    """The mechanism: the SAME two channels as pi_c (inventory down, completion up), but LOPSIDED --
    completion is near-saturated, so inventory dominates and the axis is monotone. The RECOMBINED
    inventory (inv x completion) falls monotonically and tracks the shift.
    """
    inv = [_bracket_M0(1500.0, M0)[3].radical_inventory for M0 in M0_SCAN]
    comp = [_completion_M0(1500.0, M0) for M0 in M0_SCAN]
    assert all(a > b for a, b in zip(inv, inv[1:])), f"inventory should FALL with M0: {inv}"
    assert all(a < b for a, b in zip(comp, comp[1:])), f"completion should RISE with M0: {comp}"
    # Lopsided: inventory swings much harder than completion (that is WHY M0 is monotone).
    inv_swing = inv[0] / inv[-1]
    comp_swing = comp[-1] / comp[0]
    assert inv_swing > 3.0 * comp_swing, \
        f"inventory must dominate completion on M0 (lopsided): inv {inv_swing:.2f}x vs comp {comp_swing:.2f}x"
    rec = [i * c for i, c in zip(inv, comp)]
    assert all(a > b for a, b in zip(rec, rec[1:])), f"recombined inventory should track the shift: {rec}"


def test_delta_h_swing_not_headroom_is_the_discriminator():
    """THE CORRECTION to the pi_c doc's unification. The tempting reading -- 'M0 is monotone because
    completion is already saturated at pi_c=10' -- is wrong. Re-run the M0 sweep at pi_c=2, where
    delta_h is small and completion starts with headroom (~33%): it is STILL monotone. Headroom alone
    does not restore the turnover; the weak delta_h swing on M0 (a datum shift, not a work climb) is
    why. So the discriminator is the delta_h swing (the completion DRIVER), not the headroom.
    """
    # Completion genuinely has room at pi_c=2 (unlike pi_c=10's ~86%), so 'saturated' cannot explain it.
    comp_lo = _completion_M0(1500.0, 0.3, pi_c=2.0)
    assert comp_lo < 0.5, f"pi_c=2 should leave completion headroom at low M0: {comp_lo}"
    fr2 = [_bracket_M0(1500.0, M0, pi_c=2.0)[3].dT5_fraction for M0 in (0.3, 0.85, 1.6, 2.5)]
    assert all(a > b for a, b in zip(fr2, fr2[1:])), \
        f"even with completion headroom, the M0 sweep at pi_c=2 stays monotone (no turnover): {fr2}"
    # And the delta_h swing on M0 really is weak (a datum shift), so completion cannot outpace inventory.
    dh_lo = _bracket_M0(1500.0, 0.3)[2]
    dh_hi = _bracket_M0(1500.0, 3.0)[2]
    assert dh_hi / dh_lo < 4.0, \
        f"M0's delta_h swing must be weak vs pi_c's ~11x (why completion cannot win): {dh_hi/dh_lo:.2f}x"


def test_M0_envelope_band_squeeze():
    """The flight axis is DOUBLE-EDGED in a way pi_c is not: protective per point, yet ram heating
    lifts the burner-squeeze FLOOR faster than the boundary, shrinking the earned OPERATING band.
    Cheap point-checks of the three edges (no bisection):
      - floor RISES:   a low Tt4 runs at low M0 but not at high M0 (ram heats Tt3 toward Tt4);
      - Tt4* RISES:    a Tt4 above design is not-earned at low M0 but earned at high M0 (protective);
      - ceiling RISES: a high Tt4 fails the burner balance at design M0 but runs at high M0.
    """
    # floor rises: Tt4=1200 runnable at M0=0.3, not at M0=3.0.
    assert _run_M0(1200.0, 0.3) is not None, "Tt4=1200 should run at M0=0.3 (floor below it)"
    assert _run_M0(1200.0, 3.0) is None, "Tt4=1200 should NOT run at M0=3.0 (ram-lifted floor above it)"
    # Tt4* rises (protective): Tt4=1900 not-earned at M0=0.3, earned at M0=3.0.
    assert not _bracket_M0(1900.0, 0.3)[3].frozen_turbine_earned, \
        "Tt4=1900 should NOT be earned at M0=0.3 (Tt4* < 1900)"
    assert _bracket_M0(1900.0, 3.0)[3].frozen_turbine_earned, \
        "Tt4=1900 SHOULD be earned at M0=3.0 (Tt4* > 1900 -- protective)"
    # ceiling rises: Tt4=2500 fails at design M0=0.85, runs at M0=2.5.
    assert _run_M0(2500.0, 0.85) is None, "Tt4=2500 should fail the burner balance at M0=0.85"
    assert _run_M0(2500.0, 2.5) is not None, \
        "Tt4=2500 SHOULD run at M0=2.5 (higher pt4 suppresses dissociation, burner closes hotter)"


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"  ok  {name}")
    print("rung 29 — all gates green")
