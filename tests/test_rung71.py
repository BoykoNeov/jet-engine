"""Rung 71 — THE FULL SPLIT: `n = m = 3`, ZERO zeros. Rung 69's move (swap ONE loop's
COORDINATE) applied to rung 70's plant — rung 68's `phi` stator becomes rung 69's INCIDENCE
stator, beside rung 47's `Tt4` governor and rung 65's `phi` valve. Three loops, THREE
constraints: the last unoccupied cell of rung 69 s 1's table, and rung 70's named strongest seam.

THE HEADLINE: **A CONSTRAINT CAN BE INDEPENDENT IN RANK AND REDUNDANT ON THE BAND.** The
Jacobian is full rank, and the third loop is live over 2 % of the march — because at the valve's
own set point `M_i = m_lim + v >= m_lim` for every admissible `v >= 0`, so the third constraint
is IMPLIED by the second's on the whole band. `zeros = n - m` counts GRADIENT DIRECTIONS, not
LIVE loops.

AND `det J`, NON-ZERO FOR THE FIRST TIME IN THIS FAMILY, FACTORS: `-(1-pair_RC)(1-pair_CV)` —
rung 67's non-degeneracy condition times rung 69's, one factor per rung — and it is BLIND to
`pair_RV`, the only gain this rung contains that no earlier one measured. Rung 69's damping
floor turns out to be the `c0 = 0` corner and does not survive; a Routh certificate replaces it.

AND IT CORRECTS RUNG 70 s 5: a loop is eroded by loops that push its constraint into the SLACK
region, not only by loops it SHARES a constraint with.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    FullSplitTransient, CrossSplitTransient, ReferenceSplitTransient,
    CrossLoopCascadeTransient, BleedLimiter, StatorLimiter, StatorIncidenceLimiter,
    SurgeLimiter, AsymmetricLag, BleedSchedule,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, PHI, V_MAX = 0.10, 0.80, 0.20
SM = PHI / FLOOR - 1.0
TAU, TAU_S, TAU_GOV = 0.05, 0.05, 0.05
TAU_ATT, TAU_REL = 0.05, 0.15
TT4_MAX = 1200.0                 # RUNG 67's imposed redline, VERBATIM (spec s 3)

# RUNG 69's published `k` band over its own riding arc (docs/rung69-spec.md s 0.2 / s 1.3).
# `pair_CV` here IS that scalar, on the same two loops — re-measured on a DIFFERENT trajectory,
# so the FORM and the BAND are gated and no tolerance the trajectory shift cannot justify is.
R69_K_LO, R69_K_HI = -2.05, -1.60

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _full(design, **kw):
    return FullSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _cross(design, **kw):
    return CrossSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _ref(design, **kw):
    return ReferenceSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _cross67(design, **kw):
    return CrossLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _keys(traj):
    return [tuple(p[k] for k in ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf"))
            for p in traj]


def _valve(tau=TAU):
    return BleedLimiter.from_margin(LP, B, SM, tau=tau)


def _phi_stator(tau=TAU_S, v_max=V_MAX):
    return StatorLimiter.from_margin(LP, v_max, SM, tau=tau)


def _inc(tau=TAU_S, v_max=V_MAX):
    return StatorIncidenceLimiter.from_margin(LP, v_max, SM, tau=tau)


def _fuel():
    return SurgeLimiter.from_margin(LP, "lp", SM)


def _lag(att=TAU_ATT, rel=TAU_REL):
    return AsymmetricLag(tau_att=att, tau_rel=rel)


def _march(m, ds=DS, **kw):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, ds, **kw)[0]


KW = dict(r=R, s_settle=SETTLE, tau=TAU, tau_gov=TAU_GOV, tau_s=TAU_S, v_max=V_MAX)


@pytest.fixture(scope="module")
def full(design):
    """THE rung-71 machine — the governor, the valve and the INCIDENCE stator."""
    return _full(design, bleed_lim=_valve(), stator_inc=_inc())


@pytest.fixture(scope="module")
def gains(full):
    return full.full_gains(FLIGHT, LO, HI, TT4_MAX, SM, **KW)


@pytest.fixture(scope="module")
def modes(full):
    return full.full_modes(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, v_max=V_MAX)


@pytest.fixture(scope="module")
def bill(full):
    return full.full_bill(FLIGHT, LO, HI, TT4_MAX, SM, ds=DS, **KW)


# =============================================================================
# GATE 1 — THE REDUCE. Rung 71 moves ONE loop's COORDINATE, so every ancestor
#          must still be reached BIT-FOR-BIT, and by DISPATCH. And the march is
#          REUSED rather than copied, which is itself gated.
# =============================================================================

def test_reduce_no_governor_is_rung69_bit_for_bit(design):
    """`tau_gov=None` with the incidence stator: rung 69's own five-state plant, untouched."""
    a = _march(_full(design, bleed_lim=_valve(), stator_inc=_inc()),
               surge=_fuel(), lag=_lag())
    b = _march(_ref(design, bleed_lim=_valve(), stator_inc=_inc()),
               surge=_fuel(), lag=_lag())
    assert _keys(a) == _keys(b)


def test_reduce_phi_stator_beside_the_governor_is_rung70_bit_for_bit(design):
    """A `phi` stator instead of the incidence one, with the governor armed: that is rung 70's
    plant exactly (`n = 3, m = 2`), and it must be reached through the parent's own path."""
    a = _march(_full(design, bleed_lim=_valve(), stator_lim=_phi_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    b = _march(_cross(design, bleed_lim=_valve(), stator_lim=_phi_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert _keys(a) == _keys(b)


def test_reduce_no_stator_is_rung67_bit_for_bit(design):
    """A governor and a valve with NO stator is rung 67 — this class never intercepts a march
    it does not own."""
    a = _march(_full(design, bleed_lim=_valve()), Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    b = _march(_cross67(design, bleed_lim=_valve()), Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert _keys(a) == _keys(b)
    assert "v" not in a[0], "no stator armed => no fifth state"


def test_reduce_inherited_arms_bit_for_bit(design):
    """Rungs 66/65/64/62's arms all leave through the same `super()`."""
    for kw, march_kw in (({"bleed_lim": _valve()}, {"surge": _fuel(), "lag": _lag()}),
                         ({"bleed_lim": _valve()}, {"surge": _fuel()}),
                         ({}, {"surge": _fuel(), "lag": _lag()}),
                         ({"bleed_sched": BleedSchedule(B, 0.65)}, {})):
        a = _march(_full(design, **kw), **march_kw)
        b = _march(_cross(design, **kw), **march_kw)
        assert _keys(a) == _keys(b), kw


def test_the_march_is_REUSED_and_not_copied(design):
    """**THE INTEGRATOR IS RUNG 70's, ENTERED RATHER THAN REFUSED.** Rungs 68/69/70 each
    shipped a sibling integrator because a STATE was being added; nothing is added here, so a
    copy would be 130 lines that could not differ — and `tests/test_numeric_fingerprint.py`
    does not watch this path, so the reuse is gated rather than argued.

    Rung 69 made five seams overridable (`_stator_leg`, `_clamp_v`, `_check_v0`, `_manifold_v`,
    `_solve_v`), each the IDENTITY of what it replaced, which is what lets ONE integrator run
    both plants."""
    assert "_integrate_fuel_cross_triple" not in FullSplitTransient.__dict__, (
        "rung-71 must not own a march: the only thing it changes is which limiter "
        "`_stator_leg` hands back")
    assert (FullSplitTransient._integrate_fuel_cross_triple
            is CrossSplitTransient._integrate_fuel_cross_triple)
    # and it really is entered — the fifth state is recorded and the plant is NOT rung 70's
    t = _march(_full(design, bleed_lim=_valve(), stator_inc=_inc()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert all(p["v"] >= 0.0 for p in t), "the INCIDENCE band is [0, +v_max]"
    assert max(p["v"] for p in t) > 0.0 and any(p["required"] > 0.0 for p in t)


def test_at_lever_returns_this_class(design):
    """THE NINTH INSTANCE of the trap rungs 61-70 each hit: the inherited sibling constructor
    hardcodes its own name, so a rung-71 machine would hand back a rung-70 one and every reader
    would measure a `phi` stator (`m = 2`) while reporting `m = 3`."""
    m = _full(design, bleed_lim=_valve()).at_lever(bleed_lim=_valve(), stator_inc=_inc())
    assert type(m) is FullSplitTransient
    assert m.stator_inc is not None and m.stator_lim is None


# =============================================================================
# GATE 2 — THE REFUSALS. Each names a plant this rung is NOT.
# =============================================================================

def test_the_fuel_leg_beside_the_governor_is_refused(design):
    """`n = 4, m = 3` — FOUR loops, two of them on the same actuator. Rung 68's `tau_gov`
    assert exists because 'silently accepts it' is the failure mode."""
    m = _full(design, bleed_lim=_valve(), stator_inc=_inc())
    with pytest.raises(AssertionError, match="n = 4, m = 3"):
        _march(m, Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_fuel(), lag=_lag())


def test_a_governor_with_no_set_point_is_refused(design):
    """`tau_gov` without `Tt4_max` would march as rung 69 while every reader reported rung 71 —
    a wrong-plant failure no float would reveal."""
    m = _full(design, bleed_lim=_valve(), stator_inc=_inc())
    with pytest.raises(AssertionError, match="odd loop IS the redline"):
        _march(m, tau_gov=TAU_GOV)


def test_forced_release_edges_and_an_instantaneous_valve_are_refused(design):
    """Rungs 50/51's forced edges are an isolation instrument for a leg that could not pin its
    own trigger; all three legs here pin their own. And rung 65 called the instantaneous valve
    limit SINGULAR, so an unlagged valve beside a lagged stator is a different plant.

    The forced edges are refused TWICE OVER, and the outer one is structural: `_stator_march` --
    the entry every reader in this family actually calls -- does not plumb `s_off`/`tau_rel`
    through at all, so they cannot reach a march on this ladder even by mistake. The assert in
    `integrate_fuel` is the inner guard for a caller that goes around it."""
    import inspect
    sig = inspect.signature(FullSplitTransient._stator_march).parameters
    assert "s_off" not in sig and "tau_rel" not in sig, sorted(sig)
    src = inspect.getsource(FullSplitTransient.integrate_fuel)
    assert "s_off is None and tau_rel is None" in src
    m2 = _full(design, bleed_lim=BleedLimiter.from_margin(LP, B, SM), stator_inc=_inc())
    with pytest.raises(AssertionError, match="INSTANTANEOUS valve"):
        _march(m2, Tt4_max=TT4_MAX, tau_gov=TAU_GOV)


def test_the_rk4_floor_fires_and_names_its_own_reason(design):
    """Rung 65 published a RETRACTION for an RK4 instability that read as a physical finding.
    The guard's constant survives a FOURTH time on a THIRD argument (no zero root at all, so
    the trace is shared three ways), so it must fire and say so."""
    m = _full(design, bleed_lim=_valve(), stator_inc=_inc())
    with pytest.raises(AssertionError, match="rung-71: ds"):
        _march(m, ds=0.05, Tt4_max=TT4_MAX, tau_gov=TAU_GOV)


# =============================================================================
# GATE 3 — s 0: **RANK INDEPENDENCE IS NOT CONSTRAINT INDEPENDENCE.** The headline.
# =============================================================================

def test_the_third_constraint_is_implied_by_the_second_on_the_whole_band(full):
    """**THE CONTAINMENT, EXACTLY, ON THE MARCHED TRAJECTORY.** At the valve's own set point

        phi = phi_lim  =>  M_i = T_c - 1/phi_lim + v = m_lim + v  >=  m_lim   for all v >= 0

    and the incidence band IS `[0, v_max]` (rung 69 s 0.1), so `{phi >= phi_lim}` intersected
    with the band sits INSIDE `{M_i >= m_lim}`. The slack minus `v` is `1/phi_lim - 1/phi`,
    which is `>= 0` there IDENTICALLY and `== 0` exactly where the valve pins `phi` on its
    floor — so the bound is tight and needs no tolerance.

    THE CONSEQUENCE IS THE RUNG: the stator is DORMANT at every point where the valve
    delivers, so it can only ride inside the valve's LAG."""
    bc = full.band_containment(FLIGHT, LO, HI, TT4_MAX, SM, ds=DS, **KW)
    assert bc["n_delivering"] > 250, bc
    assert bc["min_slack_delivering"] >= 0.0, bc
    assert bc["worst_slack_minus_v"] == 0.0, bc     # tight, and EXACTLY so
    assert bc["riding_while_delivering"] == 0, bc   # zero exceptions out of ~300 points
    # and the wall IS violated where the valve is failing — otherwise the loop is vacuous
    assert bc["min_slack_all"] < 0.0 and bc["n_riding"] > 0, bc


@pytest.mark.slow
def test_the_third_loops_window_is_the_SECOND_loops_lag(full):
    """**THE MECHANISM, MEASURED FROM BOTH SIDES.** If the containment is why the window is
    thin, then the stator's right edge must be a function of the VALVE's clock and not of its
    own. A one-sided sweep could not separate that from 'a slower loop rides longer', which is
    a different and much weaker statement.

    Measured: the edge marches 0.115 -> 0.365 monotonically over a 400x sweep of `tau_q`, and
    moves within a 1.3x band NON-monotonically over an equivalent sweep of `tau_s`."""
    wl = full.window_law(FLIGHT, LO, HI, TT4_MAX, SM, ds=DS, **KW)
    assert wl["q_monotone"], wl["edge_q"]
    assert wl["q_span"] > 2.5, wl["edge_q"]
    assert wl["s_span"] < 1.6, wl["edge_s"]
    assert wl["q_span"] > 2.0 * wl["s_span"], (wl["edge_q"], wl["edge_s"])
    es = wl["edge_s"]
    assert not all(es[i] <= es[i + 1] + 1e-12 for i in range(len(es) - 1)), (
        "the stator's own clock is not even a monotone influence — which is the point")
    # THE JOINT WINDOW IS THIN, AND THAT IS DISCLOSED RATHER THAN WORKED AROUND
    assert 0.0 < wl["joint_fraction"] < 0.05, wl["joint_fraction"]
    assert wl["base"]["n_interior"] >= 5, wl["base"]
    # AND THE TWO WINDOWS ARE NOT THE SAME NUMBER, which is gated because conflating them would
    # credit CONTAINMENT with narrowing that belongs to rung 67's imposed `Tt4_max`. The stator
    # rides over ~7.9 % of the march; the joint window is that intersected with a governor that
    # opens late, and its LEFT edge is the GOVERNOR's, not the stator's.
    b = wl["base"]
    assert b["stator"][2] / b["n"] > 2.0 * wl["joint_fraction"], b
    assert b["joint"][0] == b["gov"][0], (b["joint"], b["gov"])
    assert b["stator"][0] < b["gov"][0], (b["stator"], b["gov"])


def test_the_stator_quits_while_the_marched_phi_is_still_short(full):
    """AND THE TWO EDGES ARE NOT THE SAME NUMBER, which is stated rather than fudged.
    `_solve_v` tests dormancy on the COUNTERFACTUAL plant at `v = 0`, so the loop quits while
    the MARCHED `phi` is still below the floor by its own contribution — measured `dphi/dv`
    is about `-0.42` (rung 69 s 0.1), so the shortfall should be `~0.42 * v`."""
    wl = full.window_law(FLIGHT, LO, HI, TT4_MAX, SM, ds=DS, tau_qs=(TAU,), tau_ss=(TAU_S,),
                         **KW)
    short, v = wl["phi_short_at_off"], wl["v_at_off"]
    assert short > 0.0 and v > 0.0, wl
    assert 0.30 < short / v < 0.55, (short, v, short / v)


# =============================================================================
# GATE 4 — s 1: THREE pairs, ZERO identities, and the determinant FACTORS.
# =============================================================================

def test_rung66s_identity_appears_ZERO_times_for_the_first_time(gains):
    """Rung 66's `pair = 1` survived three times at rung 68 (`m = 1`), once at rung 69 and once
    at rung 70 (`m = 2`), and **zero times here** — it is a property of a SHARED constraint, and
    at `n = m` nothing is shared. The closest any pair comes to 1 is ~1.0, i.e. not close."""
    assert gains["rows"], gains["skipped"]
    assert gains["closest_to_1"] > 0.9, gains["closest_to_1"]


def test_both_cyclic_products_are_REDUNDANT(gains):
    """Rung 68 said *quote `x`*; rung 69 said it flips to `-k`; rung 70 found it BLIND to
    `pair_RV`. Here BOTH cyclic products collapse onto the pairs:

        y := R_v C_g V_q = -pair_RV                 exactly, at ANY base point
        x := R_q C_v V_g = -pair_RC * pair_CV

    so the three PAIRS are the complete independent set and neither cyclic is a measurement.
    Rung 68's *check what is INDEPENDENT before quoting it*, in its third shape."""
    assert gains["worst_y_is_RV"] < 5e-3, gains["worst_y_is_RV"]
    assert gains["worst_x_is_product"] < 5e-3, gains["worst_x_is_product"]
    # against the quantities they reproduce: the residual is a differencing floor, not a signal
    assert gains["worst_y_is_RV"] < 0.02 * min(abs(p) for p in gains["pair_RV"])


def test_the_full_rank_determinant_FACTORS_into_two_prior_rungs(gains):
    """**THE HEADLINE INVARIANT.** `det M = -(1 - pair_RC)(1 - pair_CV)` — rung 67's own
    non-degeneracy condition times rung 69's, ONE FACTOR PER RUNG. And it is therefore BLIND to
    `pair_RV`, the one gain this rung contains that no earlier rung has measured: it cancels
    exactly against the reverse cyclic product `y`.

    THIS IS NOT A TAUTOLOGY, and the distinction is the gate (rung 67 gate 9's retraction). The
    closed form uses FOUR of the six gains and asserts the other two drop out. `c1`'s closed
    form, by contrast, IS a re-expression of any matrix with `-1` on the diagonal and is
    reported by `full_modes`, never gated."""
    assert gains["worst_det_err"] < 5e-3, gains["worst_det_err"]
    assert gains["worst_det_err"] < 1e-2 * gains["det_scale"], gains


def test_the_determinant_provably_cannot_see_pair_RV(full):
    """AND IT IS SHOWN BY CONSTRUCTION, not only by measurement (rung 69's precedent).

    Hand-build the block from the six gains with `grad psi = sigma grad phi + e_v` imposed, then
    move `T_v` — which changes `R_v`, and through it `pair_RV` and `y`, and NOTHING else. The
    determinant must not move at all."""
    # `sig = 1.6`, `phi_v = -0.4` puts `k = sig phi_v/psi_v = -1.778` -- rung 69's own
    # measured band, so the constructed block is the shipped plant's shape and not an
    # arbitrary one.
    T_g, T_q, phi_g, phi_q, phi_v, sig = -3.0, 0.7, -0.9, 1.3, -0.4, 1.6
    psi_v = sig * phi_v + 1.0

    def block(T_v):
        return [[-1.0, -T_q / T_g, -T_v / T_g],
                [-phi_g / phi_q, -1.0, -phi_v / phi_q],
                [-sig * phi_g / psi_v, -sig * phi_q / psi_v, -1.0]]

    def det(M):
        return (M[0][0] * (M[1][1] * M[2][2] - M[1][2] * M[2][1])
                - M[0][1] * (M[1][0] * M[2][2] - M[1][2] * M[2][0])
                + M[0][2] * (M[1][0] * M[2][1] - M[1][1] * M[2][0]))

    d0 = det(block(0.5))
    for T_v in (-4.0, -0.2, 1.7, 9.0):
        M = block(T_v)
        assert abs(det(M) - d0) < 1e-12 * max(1.0, abs(d0)), T_v
        # ...while `pair_RV` and the reverse cyclic DO move, and stay each other's negative
        pair_RV = M[0][2] * M[2][0]
        y = M[0][2] * M[1][0] * M[2][1]
        assert abs(y + pair_RV) < 1e-12
    # and the closed form is the two prior rungs' conditions, multiplied
    M = block(0.5)
    assert abs(d0 + (1.0 - M[0][1] * M[1][0]) * (1.0 - M[1][2] * M[2][1])) < 1e-12


def test_the_rank_is_rung67s_own_non_degeneracy_condition(full):
    """**`m = 3` IS `pair_RC != 1`.** `span{grad phi, grad psi} = span{grad phi, e_v}`
    UNCONDITIONALLY — the lever's own `+1` in `psi_v` puts `e_v` in the span whatever the plant
    does — so the governor's gradient escapes that plane iff `T_g phi_q != T_q phi_g`.

    Built rather than argued (rung 69's precedent): force `grad T` INTO the plane and the same
    `n = 3` block must come back rank 2 with exactly one zero eigenvalue."""
    phi_g, phi_q, phi_v, sig = -0.9, 1.3, -0.4, 1.6
    psi_v = sig * phi_v + 1.0
    taus = (0.05, 0.05, 0.05)

    def spectrum(T_g, T_q, T_v):
        gg = dict(R_q=-T_q / T_g, R_v=-T_v / T_g,
                  C_g=-phi_g / phi_q, C_v=-phi_v / phi_q,
                  V_g=-sig * phi_g / psi_v, V_q=-sig * phi_q / psi_v)
        gg["pair_RC"] = gg["R_q"] * gg["C_g"]
        c2, c1, c0 = FullSplitTransient._invariants(gg, taus)
        roots = FullSplitTransient._cubic_roots_c(c2, c1, c0)
        rate = sum(1.0 / t for t in taus)
        return gg["pair_RC"], sum(1 for r in roots if abs(r) < 1e-8 * rate)

    # generic: OUT of the plane => pair_RC != 1 => rank 3, ZERO zeros
    p, z = spectrum(-3.0, 0.7, 0.5)
    assert abs(p - 1.0) > 0.5 and z == 0, (p, z)
    # forced INTO the plane (`T_q/T_g == phi_q/phi_g`) => pair_RC == 1 => rank 2, ONE zero
    T_g = -3.0
    p, z = spectrum(T_g, T_g * phi_q / phi_g, 0.5)
    assert abs(p - 1.0) < 1e-12 and z == 1, (p, z)


@pytest.mark.slow
def test_the_two_inherited_controls(full, gains):
    """**TWO CONTROLS, AND THEY ARE DIFFERENT KINDS — conflating them would be the error.**

    `pair_RC` is a NUMERICAL control: rows R and C are the same shipped closures rungs 67 and
    70 used, so it must reproduce rung 67's `P` up to the base-point shift the third loop
    induces. It is read against a genuinely separate rung-67 march (`cross_identity` on a
    STATOR-FREE rig) and reported as a ratio, never gated to a tolerance that shift cannot
    justify.

    `pair_CV` is a FUNCTIONAL-FORM control: it IS rung 69's `k` on rung 69's own two loops, but
    re-measured on a different trajectory. Its FORM and BAND are what is gated."""
    ref = full._full_rig(SM, TAU, TAU_S, V_MAX, TT4_MAX, stator=False).cross_identity(
        FLIGHT, LO, HI, TT4_MAX, tau=TAU, tau_govs=(TAU_GOV,))
    assert ref["all_negative"] and all(p < 0.0 for p in gains["pair_RC"])
    mid = sum(gains["pair_RC"]) / len(gains["pair_RC"])
    ratio = mid / (0.5 * (ref["prod_lo"] + ref["prod_hi"]))
    assert 0.5 < ratio < 2.0, (mid, ref["prod_lo"], ref["prod_hi"], ratio)
    # rung 69's `k`, on rung 69's own two loops
    assert all(R69_K_LO < p < R69_K_HI for p in gains["pair_CV"]), gains["pair_CV"]


@pytest.mark.slow
def test_the_cross_rung_identity_pair_RV_is_k_times_rung70s(gains):
    """`pair_RV(71) = pair_CV * pair_RV(70)` at an IDENTICAL base point, because
    `psi_g/psi_v = (phi_g/phi_v)(sigma phi_v/psi_v)`. Measured by reading rung 70's
    `phi`-referenced rig at THIS march's own points — rung 69's design, which differences two
    references on ONE trajectory rather than on two."""
    assert gains["worst_cross_rung"] is not None
    assert gains["worst_cross_rung"] < 0.02, gains["worst_cross_rung"]


def test_the_state_boundary_is_asserted_at_every_sampled_point(gains):
    """`R_q != 0` and `R_v != 0` ONLY because the governor senses `Tt4` on the machine as the
    other two actuators actually are. Drop the `_b_state`/`_v_state` boundary and both
    cross-gains are identically zero, the odd loop DECOUPLES, and `m` reads 2 by accident.
    Rung 70 built the broken version on purpose; it is checked here at every sampled point."""
    assert gains["boundary"], "the boundary check never ran"
    for c in gains["boundary"]:
        assert c["dead"]["R_q"] == 0.0 and c["dead"]["R_v"] == 0.0, c
        assert abs(c["live"]["R_q"]) > 0.0 and abs(c["live"]["R_v"]) > 0.0, c


# =============================================================================
# GATE 5 — s 2: ZERO zeros, `det J` ALIVE, and Routh non-trivial.
# =============================================================================

@pytest.mark.slow
def test_the_last_unoccupied_cell_has_ZERO_zeros(modes):
    """**THE RUNG.** `zeros = n - m = 0` at `(n, m) = (3, 3)` — the one cell of rung 69 s 1's
    table this ladder has never occupied, and the first plant in this family whose actuator
    block is INVERTIBLE."""
    assert modes["zeros_everywhere"] == [0], modes["zeros_everywhere"]
    for arm in modes["arms"]:
        assert arm["rows"], arm["taus"]
        assert arm["zeros"] == [0], (arm["taus"], arm["zeros"])
        # and the smallest root is not merely 'non-zero by the tolerance'
        assert arm["min_root_rel"] > 1e-2, (arm["taus"], arm["min_root_rel"])


@pytest.mark.slow
def test_the_invariants_at_full_rank(modes):
    """THREE readings on ONE materialisation of the spectrum, and they share a test for a
    MEASURED reason rather than a stylistic one: under xdist a module-scoped fixture is rebuilt
    PER WORKER, so every extra consumer of this reader can cost a whole re-run of it. Measured
    on the full gate: five consumers of a 20 s reader added 2:37 to a 2:59 suite.

    `c0` — `det J != 0` for the first time in this family, and it equals
    `-(1-pair_RC)(1-pair_CV)/prod(tau)`, FOUR of the six gains. Rung 68 quoted `det`; rungs 69
    and 70 both found it BLIND to their split. Here it is the only invariant that is alive AND
    still blind to something.

    ROUTH — at `m < n` a zero root plus a negative trace made stability automatic. At full rank
    it is a CONDITION, and the derivation leaves six unconditionally positive terms plus
    `(u + w + z - u z) a b c`, so **`u + w + z >= u z` is SUFFICIENT at EVERY bandwidth triple**
    — the first non-trivial stability certificate this family has had. The spectrum is checked
    stable arm by arm rather than inferred from the certificate: an assert nobody has run past
    is a tautology (rung 67 gate 9).

    RK4 — the inherited constant survives a FOURTH time on a THIRD argument (with no zero root
    the trace is shared three ways, so the dominant root sits strictly below the rate sum), and
    rung 65's retraction is why it is MEASURED rather than trusted."""
    assert modes["max_c0_err"] < 5e-3, modes["max_c0_err"]
    assert modes["min_routh"] > 0.0, modes["min_routh"]
    assert modes["all_stable"], modes
    for arm in modes["arms"]:
        for row in arm["rows"]:
            assert row["stable"], (arm["taus"], row["s"], row["roots"])
    assert modes["max_mod_ratio"] < 1.0, modes["max_mod_ratio"]
    assert modes["ds"] * modes["max_mod_ratio"] * 240.0 < 2.0, modes


@pytest.mark.slow
def test_rung69s_damping_floor_was_the_c0_EQUALS_0_CORNER(modes):
    """**RUNG 69's FLOOR DOES NOT SURVIVE FULL RANK, and the mechanism says why.** All three
    roots share ONE trace budget, `sum(lam) = -sum 1/tau_i`. At rung 69 the third root WAS the
    zero, so the pair took the whole budget and `zeta >= 1/sqrt(1-k)` followed by AM-GM. Here
    the third loop's own pole DRAINS it, so the pair's real part is smaller at comparable
    modulus and the bound has no reason to hold.

    IT DOES NOT. The grid shows all three regimes — arms below rung 69's line, arms above it,
    and arms with no complex pair at all — which is what 'the bound is removed, not replaced'
    has to look like. A single monotone trend, or a floor that survived, would refute the
    trace-budget mechanism."""
    assert modes["arms_below_r69"] >= 1, modes
    assert modes["arms_with_ring"] - modes["arms_below_r69"] >= 1, modes
    assert modes["arms_real"] >= 1, modes


def test_the_damping_reader_had_to_be_REBUILT_a_third_time(full):
    """**THE INSTRUMENT, AND ITS THIRD REBUILD IN FOUR RUNGS.** Rung 69 reads
    `-Re(dom)/|dom|`, exact for a complex DOMINANT pair and exactly 1.0 for any real root; rung
    70 reads both NON-ZERO roots magnitude-sorted, exact when exactly one root is zero. **Here
    no root is zero and the pair is not always the two largest**, so magnitude ordering can drop
    a pair MEMBER and keep the odd real root.

    Built as a difference on constructed spectra, so it does not depend on the plant."""
    # a real root SMALLER than the pair: both readers agree
    ok = [complex(-18.0, 0.0), complex(-21.0, 28.0), complex(-21.0, -28.0)]
    assert abs(FullSplitTransient._zeta_ring(ok)
               - CrossSplitTransient._zeta_pair(ok)) < 1e-12
    # a real root LARGER than the pair: rung 70's reader drops a pair member
    bad = [complex(-194.0, 0.0), complex(-23.0, 25.5), complex(-23.0, -25.5)]
    ring = FullSplitTransient._zeta_ring(bad)
    assert abs(ring - 23.0 / abs(complex(-23.0, 25.5))) < 1e-12
    assert abs(CrossSplitTransient._zeta_pair(bad) - ring) > 0.5
    # an entirely REAL spectrum: a reader that returns a number where there is no ring is
    # worse than one that returns nothing
    assert FullSplitTransient._zeta_ring(
        [complex(-20.0, 0.0), complex(-82.0, 0.0), complex(-138.0, 0.0)]) is None


# =============================================================================
# GATE 6 — s 3: THE FIXED POINT IS A POINT. Rung 69 s 6, at nullity ZERO.
# =============================================================================

@pytest.mark.slow
def test_the_s0_fixed_point_becomes_UNIQUE_at_full_rank(full):
    """**RUNG 69 s 6 CALLED A NULL SPACE A SHOCK ABSORBER; AT NULLITY ZERO THERE IS NOTHING TO
    ABSORB WITH, AND THE SWEEP REJECTS INSTEAD.**

    Rungs 68/69/70 all carry a null space, so their `s = 0` fixed points are a ONE-PARAMETER
    FAMILY and a Gauss-Seidel sweep lands on whichever member its ORDER selects. At `n = m`
    there is no null space and the fixed point is a POINT: every sweep order and every displaced
    start must land on the SAME `(g, q, v)`.

    RUNG 70's PLANT IS THE NEGATIVE CONTROL ON THE SAME RIG — its valve and stator SHARE `phi`,
    so `|C_v V_q| = 1` exactly and its sweep is marginal by construction. A contraction here
    that were not matched by a failure to contract there would be measuring the solver."""
    ic = full.ic_contraction(FLIGHT, LO, HI, TT4_MAX, SM, ds=DS, **KW)
    fu, sh = ic["full"], ic["shared"]
    assert fu["n_converged"] == fu["n"], fu
    assert fu["members"] == 1, fu
    assert fu["spread"] == {"g": 0.0, "q": 0.0, "v": 0.0}, fu["spread"]
    # THE CONTROL: the shared-constraint plant lands on a FAMILY from the same starts
    assert sh["members"] > 1, sh
    assert max(sh["spread"].values()) > 1e-3, sh["spread"]


# =============================================================================
# GATE 7 — s 4: THREE currencies, and rung 70 s 5's erosion law CORRECTED.
# =============================================================================

@pytest.mark.slow
def test_six_of_the_eight_ledger_cells_are_INHERITED_bit_for_bit(bill, design):
    """A FREE DIFFERENCEABILITY CHECK (rung 63's lesson). Every cell WITHOUT an incidence stator
    is a rung-70 march; every cell WITHOUT a governor is a rung-69 one. Only `GS` and `GVS` are
    new, so six of the eight must reproduce their ancestors' published integrals exactly — a
    drift in a cell that CANNOT have one would mean the rigs are not comparable."""
    c = bill["cells"]
    r70 = _cross(design, bleed_lim=_valve(), stator_lim=_phi_stator()).split_bill(
        FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS, tau=TAU,
        tau_gov=TAU_GOV, tau_s=TAU_S, v_max=V_MAX)["cells"]
    for name in ("bare", "G", "V", "GV"):          # no stator at all => rung 70's own cells
        assert c[name]["I"] == r70[name]["I"], name
        assert c[name]["E"] == r70[name]["E"], name
    # the two NEW cells are the ones that carry BOTH the governor and the incidence stator
    assert c["GS"]["I"] != r70["GS"]["I"] and c["GVS"]["I"] != r70["GVS"]["I"]


@pytest.mark.slow
def test_the_ledger_needs_THREE_currencies(bill):
    """Rung 66/68 had one, rung 70 had two; three loops on three walls need three. And the
    cross-credits keep rung 70's SIGNS: both airflow loops DEBIT the temperature while the
    governor CREDITS the surge margin."""
    c = bill["cells"]
    assert c["V"]["E"] > c["bare"]["E"] and c["S"]["E"] > c["bare"]["E"], bill["degrades"]
    assert c["G"]["I"] < c["bare"]["I"], c["G"]
    for k in ("phi", "Tt4", "inc"):
        assert bill["delivered"][k] > 0.5, bill["delivered"]


@pytest.mark.slow
def test_the_loop_that_does_NOT_watch_the_wall_protects_it_BETTER(bill):
    """**s 0's CONTAINMENT, READ IN THE LEDGER, AND THE SHARPEST SINGLE NUMBER HERE.** The
    VALVE — which cannot see `M_i` at all — delivers more incidence credit running alone than
    the INCIDENCE STATOR does, because holding `phi` on its floor implies the incidence wall
    with margin `v` while the reverse is not true."""
    assert bill["inc_credit_valve_alone"] > bill["inc_credit_stator_alone"], bill
    assert bill["inc_credit_valve_alone"] > 0.85, bill["inc_credit_valve_alone"]


@pytest.mark.slow
def test_rung70s_erosion_law_is_CORRECTED_by_a_second_channel(bill):
    """**RUNG 70 s 5: *a loop is eroded by the loops it shares a constraint with, and by no
    others.* NO TWO LOOPS SHARE HERE, AND THE STATOR IS ERODED ANYWAY** — it keeps a few per
    cent of its solo credit in its own currency, while the governor keeps ~100 % of its own.

    THE CORRECTION IS s 0's MECHANISM: erosion has a SECOND channel. A loop is eroded by any
    loop that pushes its constraint into the SLACK region, which is a statement about FEASIBLE
    SETS and not about gradients. Rung 70 could not see it because none of its loops could
    satisfy another's wall on its behalf.

    AND THE TWO READINGS ARE QUOTED TOGETHER (rung 58's *check the SUM, not the term*): the
    valve's `kept` exceeds 1 only because the stator running alone DEGRADES `phi` below the bare
    march (rung 69 s 4's own finding), so the valve is repairing damage rather than delivering
    protection. That confound is recorded, not hidden."""
    kept = bill["kept"]
    assert 0.8 < kept["gov"] < 1.3, kept          # unshared AND uneroded — rung 70's half
    assert kept["stator"] < 0.25, kept            # unshared and ERODED — the correction
    # the confound behind `kept["valve"] > 1`, recorded rather than explained away
    assert kept["valve"] > 1.0, kept
    assert "I" in bill["degrades"]["S"], bill["degrades"]
    assert "I" in bill["degrades"]["GS"], bill["degrades"]
