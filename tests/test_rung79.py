"""Rung 79 — THE STATE COORDINATE: rung 78 § 9's fourth seam.

Rung 78 re-wrote a leg's LAW and lost the root's UNIQUENESS. This rung re-writes a leg's STATE
COORDINATE — rung 60's incidence `M_i` for rung 49's `phi`:

    Gi(w) = m_lim - M_i(w) = 1/phi(w) - 1/phi_lim = Gs(w) * h(w),   h = 1/(phi*phi_lim) > 0

`T_c` and `v` CANCEL, and what is left is a STRICTLY POSITIVE multiplier.

HEADLINE: **a coordinate is a GAUGE on the set point and UNREACHABLE on the plant** — the branch
that makes a leg AUTHORITATIVE (`_cap_free`'s binding short-circuit) is the branch that
SUBSTITUTES THE ORIGINAL COORDINATE back in, via `_surge_fuel`'s own hardcoded `Gs`. So
{knob live} and {leg reaches applied fuel} are DISJOINT BY CONSTRUCTION.

BOUNDS rung 78 — uniqueness survives here because the multiplier is positive; rung 78's loss was
its AFFINE family's `1 - k*c` passing through zero.

Anchor + scoring: `docs/plans/rung79-anchor-state-coordinate.md`, `docs/rung79-spec.md` § 8.
"""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    StateCoordinateTransient, ResidualGaugeTransient, IncidenceLimiter, SurgeLimiter,
    BleedLimiter, StatorLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, TT4_MAX = 1000.0, 1400.0, 1200.0
B, V_MAX, TAU, TAU_S = 0.10, 0.20, 0.05, 0.05
PHI_JAC, MARGIN = 0.80, 0.10

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg():
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=(1.4 - 1.0) / 1.4 * 1004.0,
               gamma_t=g, cp_t=cp, R_t=(g - 1.0) / g * cp, hPR=42.8e6)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _rig(design, cls=StateCoordinateTransient):
    sm = PHI_JAC / FLOOR - 1.0
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_lim=StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m


@pytest.fixture(scope="module")
def scan(design):
    return _rig(design).coord_scan(FLIGHT, LO, HI, TT4_MAX)


@pytest.fixture(scope="module")
def census(design):
    return _rig(design).coord_census(FLIGHT, LO, HI, TT4_MAX)


@pytest.fixture(scope="module")
def forced(design):
    return _rig(design).coord_forced(FLIGHT, LO, HI, TT4_MAX)


@pytest.fixture(scope="module")
def march(design):
    return _rig(design).coord_march(FLIGHT, LO, HI, TT4_MAX)


# --- s 1: THE CANCELLATION, WHICH IS WHAT MAKES THE KNOB CONSTANT-FREE -----------------------

def test_the_incidence_residual_is_free_of_T_c_and_v(design):
    """`Gi = 1/phi - 1/phi_lim`: the blade metal and the stator setting CANCEL.

    Checked against rung 60's SHIPPED `IncidenceLimiter`, not against this rung's algebra — so
    the cancellation is a property of the two instruments agreeing, not of one restating itself.
    Swept over `v`, because a term that cancels must cancel at EVERY setting."""
    m = _rig(design)
    T_c = LP.tan_beta1_crit()
    surge = SurgeLimiter(spool="lp", phi_lim=PHI_JAC)
    for v in (-0.15, 0.0, 0.07, 0.20):
        lim = IncidenceLimiter.from_phi(LP, "lp", PHI_JAC, vsv=v)
        for phi in (0.60, 0.72, PHI_JAC, 0.95):
            M_i = T_c - (1.0 / phi - v)
            # rung 60's residual, built the long way ...
            long_way = lim.m_lim - M_i
            # ... and this rung's, with the cancellation already taken
            short_way = 1.0 / phi - 1.0 / surge.phi_lim
            assert abs(long_way - short_way) < 1e-12 * max(1.0, abs(short_way)), (
                v, phi, long_way, short_way)


def test_the_multiplier_is_strictly_positive_so_signs_agree(design, scan):
    """D1's mechanism: `Gi = Gs * h` with `h > 0`, so the two residuals agree in SIGN
    everywhere — which is why a coordinate cannot create or destroy a root."""
    assert scan["n"] > 0
    for row in scan["rows"]:
        # both slopes measured at the SAME point; a positive multiplier cannot flip one
        assert row["slope_phi"] * row["slope_inc"] > 0.0, row


# --- THE REDUCE, BOTH DIRECTIONS (rung 73's discipline) -------------------------------------

def test_reduce_phi_is_rung_78_by_dispatch(design):
    """`_phi_ref = "phi"` must take the PARENT's `_cap_fuel`, not an algebraically-equal copy."""
    m = _rig(design)
    assert m._phi_ref == "phi", "the shipped coordinate is the default"
    assert (StateCoordinateTransient._cap_fuel
            is not ResidualGaugeTransient._cap_fuel), (
        "rung 79 declares its own `_cap_fuel`; if it did not there would be nothing to reduce")
    # and the residual at "phi" IS rung 49's expression
    surge = SurgeLimiter(spool="lp", phi_lim=PHI_JAC)
    G = m._phi_residual(FLIGHT, 1.0, 1.0, surge, "phi")
    assert G.__code__.co_consts is not None      # a closure over phi_lim, not over 1/phi_lim


def test_reduce_direction_one_the_knob_is_wired(scan):
    """Rung 73's discipline, half one: at `incidence` the SLOPE must MOVE, by the DERIVED
    factor `1/phi_lim**2`. A coordinate that changes nothing is not a coordinate."""
    assert abs(scan["predicted_ratio"] - 1.0) > 0.1, "the test would be vacuous at phi_lim = 1"
    assert scan["ratio_err"] < 1e-6, scan["ratio_err"]
    for row in scan["rows"]:
        assert abs(row["ratio"] - 1.0) > 0.1, ("the slope did not move", row)


def test_reduce_direction_two_the_set_point_does_not(forced):
    """Rung 73's discipline, half two: the SET POINT must not move, or it is a device.

    Measured on the FORCED solve (`coord_forced`), NOT on the plant. The plant's `0.0` is
    `_surge_fuel`'s, not the coordinate's (spec s 5.1), so gating on it would pass a knob that
    was never consulted."""
    assert forced["d_forced"] < 1e-12, forced["d_forced"]
    assert forced["d_forced"] > 0.0, (
        "the forced solve returned bit-identical roots in both coordinates — either the bypass "
        "is not bypassing, or the two residuals are the same object (spec s 5.2)")


def test_the_forced_bypass_reproduces_the_shipped_solve(forced):
    """NON-VACUITY for s 5.2: the bypass must land on the SAME set point the plant uses, or it
    is measuring a nearby problem and its `1e-15` says nothing about this one."""
    assert forced["d_shipped"] == 0.0, forced["d_shipped"]
    assert forced["n_binding"] == forced["n"], (
        "these points are not in the binding regime, so the bypass bypasses nothing")


# --- s 1-3: THE IDENTITY, CONFIRMED (declared UNSCORED in the anchor) ------------------------

def test_slope_scales_by_the_derived_factor(scan):
    """D2: `Gi'(w*) / Gs'(w*) == 1/phi_lim**2`, a factor with NO fitted content."""
    assert scan["ratio_err"] < 1e-6, scan["ratio_err"]


def test_sensitivity_is_coordinate_invariant(scan):
    """D3: `dw*/dq` does not move — rung 78's headline half, arriving on the STATE side."""
    assert scan["dwdq_err"] < 1e-9, scan["dwdq_err"]


def test_the_sensitivity_reading_is_not_trivially_zero(scan):
    """NON-VACUITY for D3: `dwdq_err == 0` proves invariance only if `dwdq` is itself NONZERO.
    Two identically-dead readings difference to zero and pass (rung 77 s 8's `1.000e+00`)."""
    for row in scan["rows"]:
        assert abs(row["dwdq_phi"]) > 1e-4, ("dw*/dq is dead; the invariance is vacuous", row)


# --- s 4: THE ROOT CENSUS -------------------------------------------------------------------

def test_root_counts_are_equal_in_both_coordinates(census):
    """D1: a strictly positive multiplier preserves the root SET pointwise."""
    assert census["counts_equal"], census["rows"]
    assert census["worst"] < 1e-9, census["worst"]


def test_uniqueness_survives_the_coordinate(census):
    """THE BOUND ON RUNG 78. Its gauge destroyed uniqueness; this one does not — because the
    multiplier is positive rather than affine-through-zero. `_root_count` is rung 78 s 3's own
    walk, which FOUND that second root, so `[1]` here is a measurement and not a blind spot."""
    assert census["n_roots"] == [1], census["n_roots"]


# --- s 5: THE PLANT, AND WHY IT CANNOT SEE THE KNOB -----------------------------------------

def test_the_phi_leg_wins_the_inner_min_everywhere(march):
    """P3: rung 78's s 5 died with `binds = 0` on the ACCEL leg. The leg that wins those points
    is the PHI leg, which is the one this rung moves."""
    assert march["hits"] > 0, "the re-coordinated branch never ran"
    assert march["binds"] == march["hits"], (march["binds"], march["hits"])


def test_the_knob_was_live_on_the_plant_at_least_sometimes(march):
    """s 5.1's counter, SPLIT BY COORDINATE. A single total cannot tell *3 slack states in each
    of 2 coordinates* from *6 in `phi` alone*, and only the first means the knob was exercised
    at all. If `br_inc` were 0, s 5 would have measured nothing about this rung."""
    assert march["br_inc"] > 0, (
        "the INCIDENCE residual was never bracketed on the plant — s 5 is fully vacuous")
    assert march["fb_inc"] > march["br_inc"], (
        "the short-circuit is supposed to DOMINATE here; if it does not, s 5.1 is wrong")


def test_the_complementarity_is_exact(march):
    """s 5.3 — THE RUNG. `_cap_free` brackets the coordinated residual iff the cap is AT OR
    ABOVE the schedule, which is exactly when `_applied_demand` throws the cap away. So the two
    sets are DISJOINT, and they PARTITION the calls."""
    assert march["n_both"] == 0, march["n_both"]
    assert march["n_live"] + march["n_reach"] == march["n_log"], (
        march["n_live"], march["n_reach"], march["n_log"])
    assert march["n_live"] > 0 and march["n_reach"] > 0, (
        "a partition with an empty side is not a partition — one of the two regimes never "
        "occurred, and the disjointness is then trivially true")


def test_the_min_never_flips_AND_that_is_vacuous(march):
    """P2 and P2n TOGETHER, and they must be asserted together.

    `flips = 0` on its own is worthless here: the coordinate moved the cap by EXACTLY zero on
    the plant, and the two legs are ~12.6% apart. The anchor registered this guard BEFORE the
    measurement and named this as the likely outcome; the gate records it as a VACUOUS hold so
    no later reader can quote `flips = 0` as evidence."""
    assert march["flips"] == 0, march["flips"]
    assert march["vacuous"] is True, (
        "the vacuity guard did not fire — if the gap has closed, P2 has become a real "
        "measurement and spec s 8 needs rescoring")
    assert march["gap_min"] > 1e3 * max(march["d_max"], 1e-15), (
        march["gap_min"], march["d_max"])


def test_the_gap_log_records_distinct_FLOATS_not_distinct_states(march):
    """CORRECTED AFTER SHIP (`docs/rung79-gap-margin.md` s 4.1). This counter's docstring used
    to read `n_distinct > 10` as refuting "ONE state logged 1366 times".

    **IT IS EXACTLY THAT.** The plant never leaves its initial state (see the standstill gates
    below); the 129 distinct `p_phi` values are float-level products of a bracketed solve whose
    START POINT `mf_sched` sweeps 1.478x while the state is constant to 1e-15. The counter
    distinguishes distinct FLOATS; the claim needed distinct STATES.

    A counter is only as good as the NOUN it counts -- "count, never eyeball" does not save you
    from counting the wrong thing. Kept as a plumbing check (the log is populated at all, which
    is what caught s 5.5's first instrument failure) and renamed so it cannot be re-read as a
    state count."""
    assert march["n_distinct"] > 10, march["n_distinct"]
    assert march["n_distinct_gap"] > 10, march["n_distinct_gap"]
    assert march["n_log"] > 100, march["n_log"]


def test_the_trajectory_and_the_schedule_do_not_move(march):
    """P4, and the carried-knob check. `sched_moved` guards the direction rung 78 lost sixteen
    times: `accel_for` builds on `_shared_rig`, which CARRIES `_phi_ref`, so a schedule that
    tracked the knob would make this section compare two schedules."""
    assert march["same_len"], "the two marches took different numbers of steps"
    assert march["worst"] < 1e-9, (march["worst"], march["where"])
    assert march["sched_moved"] == 0.0, march["sched_moved"]


# --- THE STANDSTILL: s 9's GAP SEAM, CHECKED AFTER SHIP -------------------------------------
# `docs/rung79-gap-margin.md`. These gates were added by a CORRECTION, not by this rung's
# anchor, and they are honest about that: nothing here was pre-registered.

TAUS, R, S_SETTLE, DS = (0.05, 0.05, 0.05, 0.05), 0.5, 1.2, 0.005


def _march_at(design, phi_jac=PHI_JAC, margin=MARGIN):
    """`coord_march`'s own march, with the TRAJECTORY handed back (the reader returns only
    aggregates, and the whole question here is whether the states move)."""
    m0 = _rig(design)
    sm = phi_jac / FLOOR - 1.0
    if phi_jac != PHI_JAC:                    # the positive control needs its own floors
        m0 = m0.at_lever(bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
                         stator_lim=StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S))
        m0._lag_coord, m0._ref_law = "demand", "sched"
        m0._windup_law, m0._cap_law = "none", "solve"
    accel = m0.accel_for(FLIGHT, LO, HI, sm, TT4_MAX, TAUS, V_MAX, False, margin)
    m, _, _, traj = m0._cap_march(FLIGHT, LO, HI, TT4_MAX, sm, TAUS, R, S_SETTLE, DS,
                                  V_MAX, False, "demand", "sched", "none", None, "solve",
                                  accel)
    return m0, m, accel, traj


def _spread(traj, key):
    v = [p[key] for p in traj if isinstance(p.get(key), float)]
    return (max(v) - min(v)) / max(abs(min(v)), 1e-30)


@pytest.fixture(scope="module")
def still(design):
    return _march_at(design)


def _a_cap(m0, m, p, margin):
    """The accel leg's set point at ONE frozen state, at an arbitrary `margin` -- rung 76's
    `solve` law, i.e. the FIXED POINT of `w = (1+margin)*kappa*pt3(w)`."""
    sm = PHI_JAC / FLOOR - 1.0
    accel = m0.accel_for(FLIGHT, LO, HI, sm, TT4_MAX, TAUS, V_MAX, False, margin)
    a, h, q, v, ms = p["nu_lp"], p["nu_hp"], p["b"], p["v"], p["mf_sched"]
    cap = m._accel_cap_fn(FLIGHT, a, h, accel)
    m._b_state, m._v_state = q, v
    try:
        w = m._cap_free(lambda x: x - cap(x), ms,
                        lambda: m._sched_fuel(FLIGHT, a, h, ms, accel))
        return w, m._c_at(FLIGHT, a, h, accel, w, q, v), m._c_at(
            FLIGHT, a, h, accel, p["mf"], q, v)
    finally:
        m._b_state, m._v_state = None, None


def test_the_march_stands_still_AND_THAT_IS_THE_SCOPE_OF_SECTION_5(still):
    """**THE CORRECTION, PINNED.** `nu_lp`/`nu_hp` do not move by ONE BIT over the whole march,
    so s 5's "1366 calls across the accel" are 1366 calls at ONE OPERATING POINT.

    THIS GATE BLESSES NOTHING. It records s 5's scope condition. An edit that unpins this march
    must RESCORE s 5 (and rung 78's), not silently change what the section measured -- and the
    rig must NOT be re-tuned to unpin it: `PHI_JAC = 0.80` sitting exactly on the wall is what
    ss 1-4's CONSTRAINED linearisation requires, and `test_numeric_fingerprint.py` pins these
    numbers bit-exact."""
    _, _, _, traj = still
    assert len(traj) > 300, len(traj)
    assert _spread(traj, "nu_lp") == 0.0, _spread(traj, "nu_lp")
    assert _spread(traj, "nu_hp") == 0.0, _spread(traj, "nu_hp")
    assert _spread(traj, "mf") < 1e-12, _spread(traj, "mf")
    # ... while the COMMAND ramps. Without this the standstill could be an empty march.
    assert _spread(traj, "mf_sched") > 1.0, _spread(traj, "mf_sched")


def test_the_floor_is_armed_AT_the_initial_condition(still):
    """THE MECHANISM. Three phi floors sit on one wall at `phi_lim = 0.80`, and the FREE initial
    point is BELOW it (0.7731) -- the stator lifts the plant exactly onto the wall, so rung 49's
    leg reads a state already ON its floor and its cap IS the fuel already flowing.

    A limiter armed with ZERO INITIAL MARGIN has no transient."""
    _, _, _, traj = still
    assert abs(traj[0]["phi_lp"] - PHI_JAC) < 1e-9, traj[0]["phi_lp"]
    assert _spread(traj, "phi_lp") < 1e-12, "phi never leaves the wall"
    # and the cap IS the flowing fuel, which is why nothing accelerates
    assert abs(traj[0]["cap_fuel"] - traj[0]["mf"]) < 1e-12 * traj[0]["mf"]


@pytest.mark.slow
def test_the_same_rig_DOES_march_when_the_wall_is_lowered(design):
    """THE POSITIVE CONTROL, and without it the gate above would also pass a march broken for
    any other reason. Same rig, same code path, wall below the initial operating point."""
    _, _, _, traj = _march_at(design, phi_jac=0.75)
    assert _spread(traj, "nu_lp") > 1e-2, _spread(traj, "nu_lp")
    t4 = [p["Tt4"] for p in traj]
    assert max(t4) - min(t4) > 100.0, (min(t4), max(t4))


def test_gap_at_zero_margin_is_EXACTLY_zero(still):
    """s 9 predicted `gap ~ margin + 0.026`, the offset being `kappa(n_H)`'s drift. REFUTED at
    its own anchor: a STANDING plant is AT steady state, and rung 48's `margin = 0` schedule IS
    the steady-state fuel -- so the accel cap and the phi floor's cap are the same number.

    There is no constant offset to explain."""
    m0, m, _, traj = still
    p = traj[len(traj) // 2]
    w0, _, _ = _a_cap(m0, m, p, 0.0)
    assert abs(w0 - p["mf"]) < 1e-11 * p["mf"], (w0, p["mf"])


def test_the_gap_residual_is_rung_77s_STIFFNESS(still, march):
    """**WHAT s 9's RESIDUAL ACTUALLY IS.** `gap + 1 = a_cap/mf` (the plant is pinned, so the
    phi cap IS the flowing fuel), and `a_cap` is rung 76's `solve` law -- a FIXED POINT, not an
    evaluation. So

        d ln(gap+1) / d ln(1+margin)  =  1/(1 - c) ,   c = d(cap)/dw   [rung 77's own scalar]

    Two independent instruments: the SWEEP's slope, and the SHIPPED `_c_at`.

    THE EVALUATION POINT IS THE WHOLE TEST, and the second assert is its NON-VACUITY control.
    `c` must be read at the FIXED POINT -- the point whose response is being swept -- not at the
    plant's fuel, which is 12.6% away in `w`. An instrument that agreed at BOTH points would be
    insensitive to where it was read and would therefore be measuring nothing (rung 77 s 8's
    `1.000e+00`)."""
    m0, m, _, traj = still
    p = traj[len(traj) // 2]
    d = 0.01
    lo, _, _ = _a_cap(m0, m, p, MARGIN - d)
    hi, _, _ = _a_cap(m0, m, p, MARGIN + d)
    mid, c_cap, c_mf = _a_cap(m0, m, p, MARGIN)
    slope = ((math.log(hi) - math.log(lo))
             / (math.log1p(MARGIN + d) - math.log1p(MARGIN - d)))
    assert abs(slope - 1.0 / (1.0 - c_cap)) < 1e-4 * slope, (slope, c_cap)
    # ... and the control: read at the WRONG point it must MISS, by orders
    miss = abs(slope - 1.0 / (1.0 - c_mf)) / slope
    assert miss > 1e-3, (
        "`c` read at the plant's fuel agrees just as well as `c` read at the fixed point -- "
        "the identity is then insensitive to the evaluation point and measures nothing", miss)
    # AND THE GAP REALLY IS THAT RATIO, not a nearby one. Compared against the SHIPPED reader's
    # own `gap_min` rather than a hardcoded literal: this file is not the place for an
    # absolute-value golden (that is `test_numeric_fingerprint.py`'s job, on a CPython anchor),
    # and a cross-instrument agreement is the stronger claim anyway -- it says this frozen-state
    # reconstruction reproduces what `coord_march` reports, which is what s 9 quoted.
    assert abs((mid / p["mf"] - 1.0) - march["gap_min"]) < 1e-9 * march["gap_min"], (
        mid / p["mf"] - 1.0, march["gap_min"])


# --- THE REFUSAL ----------------------------------------------------------------------------

def test_incidence_times_a_live_rung_78_gauge_is_refused(design):
    """The two knobs re-write DIFFERENT legs' residuals; composing them is neither rung. Rung
    78 s 0.3's refusal of `sensed x gauge`, one knob over."""
    m = _rig(design)
    m._phi_ref, m._gauge_k = "incidence", 2.0
    surge = SurgeLimiter(spool="lp", phi_lim=PHI_JAC)
    with pytest.raises(AssertionError, match="REFUSED"):
        m._cap_fuel(FLIGHT, 1.0, 1.0, 0.01, None, surge)


def test_the_carried_knob_survives_at_lever(design):
    """THE SEVENTEENTH INSTANCE. Rung 78 lost the class and the gauge here; this rung would lose
    the COORDINATE on top of both, and every reader below would report rung 79 having marched
    rung 78."""
    m = _rig(design)
    m._phi_ref = "incidence"
    n = m.at_lever(bleed_lim=m.bleed_lim, stator_lim=m.stator_lim)
    assert isinstance(n, StateCoordinateTransient)
    assert n._phi_ref == "incidence", "the coordinate was dropped by `at_lever`"
    assert n._cap_law == m._cap_law and n._lag_coord == m._lag_coord


def test_the_probe_flag_is_written_on_the_class(design):
    """s 5.5's first instrument failure, gated. `self._coord_probe = True` creates an INSTANCE
    attribute, and `_cap_march` builds a NEW machine — so the marching object read the class
    default `False` and the log came back EMPTY while `hits`/`binds` reported a flawless pass.

    SCOPE: this checks the FLAG's storage, not the PLUMBING. It builds a fresh machine directly
    rather than going through `_cap_march` -> `_shared_rig` -> `at_lever`, so it would still pass
    if a future edit dropped the flag somewhere in that chain. The real regression guard for the
    chain is `n_log > 100` in `test_the_gap_log_records_distinct_states`; do not read this test
    as covering it."""
    m = _rig(design)
    seen = {}

    def peek():
        seen["probe"] = StateCoordinateTransient._coord_probe
        seen["fresh"] = _rig(design)._coord_probe

    _, log = m._with_probe(peek)
    assert seen["probe"] is True, "the probe flag was not set on the class"
    assert seen["fresh"] is True, (
        "a FRESHLY BUILT machine did not see the probe — the flag is on the instance again")
    assert StateCoordinateTransient._coord_probe is False, "the flag was not restored"
    assert log == []


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
