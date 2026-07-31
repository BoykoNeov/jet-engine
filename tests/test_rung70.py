"""Rung 70 — THE GENERIC SPLIT: rung 47's `Tt4` topping GOVERNOR as the odd loop beside rung
65's `phi` valve and rung 68's `phi` stator. Rung 67's substitution, applied to rung 68's
triple. Five states, three clocks, `n = 3`, `m = 2` — **the same cell as rung 69, reached by a
different route**, so this is a controlled comparison at equal counts.

IT CLOSES TWO SEAMS AT ONCE, and rung 69 § 11 says they are one seam from two sides: rung 68's
*three loops on TWO variables*, and rung 69's *a plant with `pair_RV != pair_CV`*.

THE HEADLINE: **THE SPLIT BUYS THE RANK; THE RING NEEDS THE ODD CONSTRAINT TO BE A SECOND WALL
ON THE SAME LEVER.** Rung 69's ringing pair came from `k ~ -1.7`, which was ONE LEVER READING
TWO WALLS. Here the odd constraint sits on a different lever, both split pairs are cross-LEVER
gains, and the damping floor lands at ~0.99 — where rung 67 put it, by the same scalar.

AND THE IDENTITY MOVES RATHER THAN VANISHING. `pair_CV = 1` now (the valve and the stator share
`phi`); `pair_RC` and `pair_RV` split — and they come back with OPPOSITE SIGNS, which no single
scalar can summarise. The cyclic product equals `-pair_RC` and is structurally BLIND to
`pair_RV`, so rung 68's *quote `x`* and rung 69's *`x = -k`* both stop being complete.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    CrossSplitTransient, ReferenceSplitTransient, ThreeLoopCascadeTransient,
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
TT4_MAX = 1200.0                 # RUNG 67's imposed redline, VERBATIM (see the spec § 3)

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _cross(design, **kw):
    return CrossSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _ref(design, **kw):
    return ReferenceSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _three(design, **kw):
    return ThreeLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


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


@pytest.fixture(scope="module")
def cross(design):
    """THE rung-70 machine — the governor beside the valve and the phi stator."""
    return _cross(design, bleed_lim=_valve(), stator_lim=_phi_stator())


@pytest.fixture(scope="module")
def gains(cross):
    return cross.split_gains(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                             tau=TAU, tau_gov=TAU_GOV, tau_s=TAU_S, v_max=V_MAX, every=10)


# =============================================================================
# GATE 1 — THE REDUCE. Rung 70 substitutes ONE loop's SENSOR, so every ancestor
#          must still be reached BIT-FOR-BIT, and by DISPATCH rather than by a
#          numerical coincidence.
# =============================================================================

def test_reduce_no_governor_is_rung68_bit_for_bit(design):
    """`tau_gov=None` with a rung-68 phi stator: rung 68's own five-state cascade, untouched."""
    a = _march(_cross(design, bleed_lim=_valve(), stator_lim=_phi_stator()),
               surge=_fuel(), lag=_lag())
    b = _march(_three(design, bleed_lim=_valve(), stator_lim=_phi_stator()),
               surge=_fuel(), lag=_lag())
    assert _keys(a) == _keys(b)


def test_reduce_no_governor_incidence_is_rung69_bit_for_bit(design):
    """`tau_gov=None` with rung 69's INCIDENCE stator: rung 69's plant, untouched."""
    a = _march(_cross(design, bleed_lim=_valve(), stator_inc=_inc()),
               surge=_fuel(), lag=_lag())
    b = _march(_ref(design, bleed_lim=_valve(), stator_inc=_inc()),
               surge=_fuel(), lag=_lag())
    assert _keys(a) == _keys(b)


def test_reduce_no_stator_is_rung67_bit_for_bit(design):
    """A governor and a valve with NO stator is rung 67 — this class never intercepts a march
    it does not own, so cascade A is reached through the parent's own dispatch."""
    a = _march(_cross(design, bleed_lim=_valve()), Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    b = _march(_cross67(design, bleed_lim=_valve()), Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert _keys(a) == _keys(b)
    assert "v" not in a[0], "no stator armed => no fifth state"


def test_reduce_inherited_arms_bit_for_bit(design):
    """Rungs 66/65/64/62's arms all leave through the same `super()`."""
    for kw, march_kw in (({"bleed_lim": _valve()}, {"surge": _fuel(), "lag": _lag()}),
                         ({"bleed_lim": _valve()}, {"surge": _fuel()}),
                         ({}, {"surge": _fuel(), "lag": _lag()}),
                         ({"bleed_sched": BleedSchedule(B, 0.65)}, {})):
        a = _march(_cross(design, **kw), **march_kw)
        b = _march(_ref(design, **kw), **march_kw)
        assert _keys(a) == _keys(b), kw


def test_at_lever_returns_this_class(design):
    """THE EIGHTH INSTANCE of the trap rungs 61-69 each hit: the inherited sibling constructor
    hardcodes its own name, so a rung-70 machine would hand back a rung-69 one and every reader
    would measure rung 69's plant while reporting rung 70's."""
    m = _cross(design, bleed_lim=_valve()).at_lever(bleed_lim=_valve(),
                                                    stator_lim=_phi_stator())
    assert type(m) is CrossSplitTransient
    assert m.stator_lim is not None and m.stator_inc is None


# =============================================================================
# GATE 2 — THE REFUSALS. Each names a plant this rung is NOT, and each is a seam.
# =============================================================================

def test_an_incidence_stator_beside_the_governor_is_refused(design):
    """Three loops on THREE constraints — `n = m = 3`, ZERO zeros, the one cell of rung 69's
    table this ladder has never occupied. Rung 70's own next seam, refused not run."""
    m = _cross(design, bleed_lim=_valve(), stator_inc=_inc())
    with pytest.raises(AssertionError, match="n = m = 3"):
        _march(m, Tt4_max=TT4_MAX, tau_gov=TAU_GOV)


def test_the_fuel_leg_beside_the_governor_is_refused(design):
    """`n = 4, m = 2` — FOUR loops, two of them on the same actuator. Rung 68's own `tau_gov`
    assert exists because 'silently accepts it' is the failure mode; this is its mirror."""
    m = _cross(design, bleed_lim=_valve(), stator_lim=_phi_stator())
    with pytest.raises(AssertionError, match="n = 4, m = 2"):
        _march(m, Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_fuel(), lag=_lag())


def test_a_governor_with_no_set_point_is_refused(design):
    """`tau_gov` without `Tt4_max` would march as rung 68 while every reader reported rung 70 —
    a wrong-plant failure that no float would reveal."""
    m = _cross(design, bleed_lim=_valve(), stator_lim=_phi_stator())
    with pytest.raises(AssertionError, match="odd loop IS the redline"):
        _march(m, tau_gov=TAU_GOV)


def test_the_rk4_floor_fires_and_names_its_own_reason(design):
    """Rung 65 published a RETRACTION for an RK4 instability that looked like a physical
    finding, and rung 68 measured that at `ds` its own constant refuses the march counterfeits
    PERFECT PROTECTION. The guard is re-justified here on a third argument, so it must fire."""
    m = _cross(design, bleed_lim=_valve(), stator_lim=_phi_stator())
    with pytest.raises(AssertionError, match="rung-70: ds"):
        _march(m, ds=0.05, Tt4_max=TT4_MAX, tau_gov=TAU_GOV)


def test_all_three_windows_overlap(cross):
    """A GATE, NOT A REMARK. `Tt4_max` is inherited from rung 67, which chose it for overlap
    with ONE phi loop. A gain table over an empty intersection would report the pairwise
    algebra of loops that were never simultaneously live."""
    w = cross.window_overlap(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                             tau=TAU, tau_gov=TAU_GOV, tau_s=TAU_S, v_max=V_MAX)
    assert w["overlaps"], w
    assert w["joint"][2] >= 20, f"the joint window is too thin to sample: {w['joint']}"
    for leg in ("gov", "valve", "stator"):
        assert w[leg][2] > 0, f"{leg} never rides at all: {w[leg]}"


# =============================================================================
# GATE 3 — s 1: THE IDENTITY MOVED, and the two split pairs DIFFER. THE RUNG.
# =============================================================================

def test_the_shared_pair_is_now_CV_and_it_holds_to_the_floor(gains):
    """`pair_CV = C_v V_q = 1` EXACTLY: the valve and the stator solve the SAME constraint, so
    their rows are parallel and the implicit-function derivatives are reciprocal.

    WHICH PAIR KEEPS RUNG 66's IDENTITY IS A DIRECT READ OF WHICH LOOPS SHARE A CONSTRAINT —
    rung 69's statement, and here it moves from `(R,C)` to `(C,V)`."""
    assert gains["rows"], "no interior riding point"
    assert gains["worst_CV"] < 1e-8, gains["worst_CV"]
    # and NEITHER split pair is 1 — the identity did not merely spread
    assert gains["worst_RC_is_1"] > 0.9
    assert gains["worst_RV_is_1"] > 0.8


def test_the_two_split_pairs_are_DIFFERENT_and_that_is_the_rung(gains):
    """RUNG 69 s 1.1: `pair_RV = pair_CV` is NOT general to a split — it holds iff the odd
    constraint depends on the shared actuators ONLY through the shared constraint. At rung 69
    that held trivially and both split pairs collapsed onto ONE scalar `k`. Here they do not.

    AND THEY COME BACK WITH OPPOSITE SIGNS, which is stronger than the registered prediction:
    the odd constraint couples with opposite sign through the two shared actuators (bleed makes
    it hotter; a closed stator does not reach `Tt4` the same way). No single scalar can
    summarise that."""
    RC = gains["pair_RC"]
    RV = gains["pair_RV"]
    assert all(x < 0.0 for x in RC), RC
    assert all(x > 0.0 for x in RV), RV
    # separated by ORDERS above the instrument's own floor (`worst_CV`, ~1e-10)
    assert gains["min_pair_gap"] > 0.5, gains["min_pair_gap"]
    assert min(abs(a - b) for a, b in zip(RC, RV)) > 1e6 * max(gains["worst_CV"], 1e-16)


def test_the_cyclic_product_is_minus_pair_RC_and_BLIND_to_pair_RV(gains):
    """`x = R_q C_v V_g = -pair_RC` identically. Rung 68 said *quote `x`*; rung 69 said *`x`
    flips to `-k`*. Both were complete only because every split pair was one scalar. Here `x`
    reproduces ONE of the two and structurally cannot see the other — rung 68's own *check what
    is INDEPENDENT before quoting it*, in its second shape."""
    assert gains["worst_cyclic_is_RC"] < 1e-8, gains["worst_cyclic_is_RC"]
    # the thing `x` cannot see: it would have to differ from `-pair_RV` by a lot, and it does
    for row in gains["rows"]:
        g = row["gov"]
        assert abs(g["cyclic"] + g["pair_RV"]) > 0.1


def test_the_identity_MOVED_measured_on_one_trajectory(gains):
    """The contrast that makes 'moved' a measurement rather than a comparison of two rungs'
    tables: rung 68's FUEL leg is re-read at the IDENTICAL base points, and there `pair_RC` is
    1 to the differencing floor while under the governor it is ~ -0.018."""
    assert gains["worst_RC_fuel"] is not None
    assert gains["worst_RC_fuel"] < 1e-8, gains["worst_RC_fuel"]
    assert gains["worst_RC_is_1"] > 0.9


def test_a_zero_cross_gain_would_be_a_MISSING_coupling_not_a_weak_one(gains):
    """THE `_b_state`/`_v_state` BOUNDARY, asserted rather than inherited — rung 68 flags it as
    the one thing here that can go wrong without failing. `R_q != 0` and `R_v != 0` ONLY because
    the governor senses `Tt4` on the machine as the other two actuators actually are; drop the
    boundary and both are identically zero, the odd loop decouples, and every prediction in this
    rung would 'confirm' rung 68 instead. `split_gains` runs the check at every sampled point;
    this asserts it produced the contrast it claims."""
    assert gains["boundary"], "the boundary check never ran"
    for chk in gains["boundary"]:
        assert chk["dead"]["R_q"] == 0.0 and chk["dead"]["R_v"] == 0.0
        assert abs(chk["live"]["R_q"]) > 0.0 and abs(chk["live"]["R_v"]) > 0.0


def test_pair_RC_reproduces_rung67_P_the_negative_control(cross):
    """`pair_RC` HERE IS rung 67's `P = R_q C_g` — same governor, same valve, same shipped
    closures. The only difference is that a third loop is present and has moved the base point,
    so the two must agree in SIGN and ORDER OF MAGNITUDE.

    IT IS A CONTROL, NOT A FINDING: a departure beyond the base-point shift means the state
    boundary is wrong, not that the plant changed. It is therefore checked loosely and on
    purpose — a tight tolerance here would be asserting that a third loop changes nothing."""
    c = cross.rung67_control(FLIGHT, LO, HI, TT4_MAX, SM, tau=TAU, tau_gov=TAU_GOV,
                             tau_s=TAU_S, v_max=V_MAX, r=R, s_settle=SETTLE, ds=DS)
    assert c["both_negative"], c
    assert 0.5 < c["ratio"] < 2.0, c


# =============================================================================
# GATE 4 — s 2: ONE ZERO, `det` BLIND, and `c1` a CLOCK-WEIGHTED SUM.
# =============================================================================

@pytest.mark.slow
def test_the_rank_is_the_constraint_count_at_a_second_realization(cross):
    """`zeros = n - m` = 1 at `(n,m) = (3,2)` — the SAME cell as rung 69, reached without an
    incidence wall. Rung 69 established the law on one realization of the cell; this is the
    second, and it is the one where the odd constraint does NOT factor."""
    mo = cross.split_modes(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=0.002,
                           v_max=V_MAX, every=20)
    for arm in mo["arms"]:
        assert arm["rows"], arm["taus"]
        assert arm["zeros"] == [1], (arm["taus"], arm["zeros"])


@pytest.mark.slow
def test_det_is_blind_to_this_split_too_and_c1_is_the_discriminator(cross):
    """`c0 = det J = 0` under this split as well — the valve and the stator keep exactly
    parallel rows whatever the governor watches. **A reader that inherited rung 68's determinant
    test would report rank one and see nothing**, which is rung 69's correction re-confirmed on
    a plant its derivation does not cover.

    `c1` is the discriminator again, and the measured value matches the two-term closed form of
    s 1.4 — which is what says the two split pairs enter on DIFFERENT clock products."""
    mo = cross.split_modes(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=0.002,
                           v_max=V_MAX, every=20)
    for arm in mo["arms"]:
        assert arm["max_c0_rel"] < 1e-9, (arm["taus"], arm["max_c0_rel"])
        assert arm["min_c1_rel"] > 1e-2, (arm["taus"], arm["min_c1_rel"])
        assert arm["max_c1_err"] < 1e-7, (arm["taus"], arm["max_c1_err"])


@pytest.mark.slow
def test_the_clock_swap_kills_the_one_scalar_model(cross):
    """THE DISCRIMINATING TEST, and the only one here that a one-scalar plant fails.

    That `c1 != 0` is rung 69's result; that it moves across a clock grid proves nothing (the
    rate sum moves too); that it matches this rung's own formula validates the formula against
    itself. **Hold `tau_g` and exchange `(tau_q, tau_s)`:** rung 69's shape (`u == w`) makes
    `c1` SYMMETRIC in that exchange and therefore INVARIANT, while two terms change by
    `(u-w)(1/(tau_g tau_q) - 1/(tau_g tau_s))`.

    The null is built from THIS plant's own gains forced to one scalar, so the comparison is
    between two models of one measurement rather than between two plants. **Every `c1` here
    comes from the shipped `_invariants`** — the actual 3x3 Jacobian — so the agreement with
    § 1.4's closed form is a test of the algebra and not a formula agreeing with itself
    (rung 67 gate 9's tautology, which this gate was rewritten to avoid)."""
    sw = cross.c1_clock_swap(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                             v_max=V_MAX)
    # the one-scalar null is invariant under the swap — rung 69's shape, on rung 70's numbers
    assert abs(sw["one_scalar_null"]["ratio"] - 1.0) < 1e-12, sw["one_scalar_null"]
    assert abs(sw["null_delta"]) < 1e-9 * abs(sw["measured_delta"]), sw
    # and this plant is decisively NOT that
    assert abs(sw["held_gains"]["ratio"] - 1.0) > 0.05, sw["held_gains"]
    assert abs(sw["measured_delta"] / sw["predicted_delta"] - 1.0) < 1e-9, sw
    # the marched arms agree with the held-gains reading up to the plant's own drift
    assert abs(sw["marched_ratio"] / sw["held_gains"]["ratio"] - 1.0) < 0.05, sw


@pytest.mark.slow
def test_the_surviving_clock_product_names_which_loops_share(cross):
    """A FREE STRUCTURAL CHECK. Rung 69's two `c1` terms both carry `1/tau_s`, its ODD loop's
    clock; both of rung 70's carry `1/tau_g`, this rung's odd loop. The pair that SHARES
    contributes nothing to `c1`, so the surviving factor is always the odd loop's clock — the
    clock products are a read of which two loops share a constraint.

    Measured by holding `tau_q = tau_s` and moving `tau_g` alone: `c1` must scale as `1/tau_g`
    EXACTLY, which a `1/tau_s`-carrying model cannot do — with `tau_q = tau_s` held fixed,
    rung 69's form has one term independent of `tau_g` and so cannot halve.

    The `c1` values come from the shipped `_invariants` on ONE set of measured gains, so this
    is the Jacobian's own scaling and not a re-evaluation of § 1.4. The CONTROL is a hand-built
    rung-69 block (rung 69's own device for its determinant claim): there the SHARED pair is
    `(R,C)`, so `1/tau_s` survives instead and `c1` provably cannot halve when `tau_g` doubles.
    Note that forcing `u == w` would NOT be that control — which pair shares is what selects the
    clock, not whether the split pairs happen to be equal."""
    sw = cross.c1_clock_swap(FLIGHT, LO, HI, TT4_MAX, SM, tau_g=0.05, fast=0.05, slow=0.05,
                             r=R, s_settle=SETTLE, ds=DS, v_max=V_MAX)
    gg = sw["arms"]["fast_valve"]["gains"]
    assert abs(gg["pair_CV"] - 1.0) < 1e-8, "this plant's shared pair is (C,V)"
    c1 = [cross._invariants(gg, (tau_g, 0.05, 0.05))[1] for tau_g in (0.05, 0.10)]
    assert abs(c1[0] / c1[1] - 2.0) < 1e-9, c1

    # THE CONTROL: rung 69's arrangement — (R,C) share, so `c1 = (1-k)(1/tau_g + 1/tau_q)/tau_s`
    k = -1.7                                   # rung 69's own measured value, near enough
    r69 = dict(R_q=2.0, C_g=0.5,               # pair_RC = 1        (the SHARED pair)
               R_v=1.0, V_g=k,                 # pair_RV = k        (split)
               C_v=1.0, V_q=k)                 # pair_CV = k        (split)
    n1 = [cross._invariants(r69, (tau_g, 0.05, 0.05))[1] for tau_g in (0.05, 0.10)]
    assert abs(n1[0] / n1[1] - 2.0) > 0.5, n1
    assert abs(n1[0] / n1[1] - 4.0 / 3.0) < 1e-9, n1   # exactly what its own form predicts


# =============================================================================
# GATE 5 — s 3: THE FLOOR. An INFIMUM on a RAY, and P8's REFUTATION.
# =============================================================================

@pytest.mark.slow
def test_the_floor_holds_and_is_STRICT_at_every_admissible_bandwidth(cross):
    """`zeta >= 1/sqrt(1 - min(pair_RC, pair_RV))` over every bandwidth, and STRICTLY.

    RUNG 69's EQUALITY SET WAS A HYPERPLANE (`u == w` makes `b, c` enter only through `b+c`, so
    `a = b+c` attains it with all three clocks finite). Here it collapses to a RAY — one shared
    loop silenced AND `a` matched to the other — so the bound is an INFIMUM that no admissible
    triple reaches. The closed form is checked against the shipped cubic's own roots."""
    f = cross.split_floor(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                          v_max=V_MAX)
    live = [x for x in f["rows"] if "zeta" in x]
    assert len(live) >= 6, f["rows"]
    assert f["holds"] and f["strict"], f["tightest"]
    assert f["worst_pred_err"] < 1e-8, f["worst_pred_err"]


@pytest.mark.slow
def test_P8_REFUTED_the_ring_is_reachable_but_only_by_silencing_the_third_loop(cross):
    """PRE-REGISTERED P8 SAID *NO COMPLEX PAIR AT ANY BANDWIDTH*. **That is FALSE**, and the
    refutation is the better result.

    The floor is `~0.990 < 1`, so a complex pair is ADMITTED — and it is found, on the arm
    `tau_s = 40x` the others, i.e. the RAY that nearly silences the stator. So the honest
    sentence is not 'no ring' but: the ring is reachable only where the third loop is
    dynamically inert, and even there `zeta ~ 0.992` puts it back in rung 67's
    *admissible, unobservable* class.

    'Reachable' and 'reachable with three live loops' are different sentences, and this gate
    asserts BOTH: no complex pair on any arm with comparable clocks, one on the ray."""
    f = cross.split_floor(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                          v_max=V_MAX)
    live = [x for x in f["rows"] if "zeta" in x]
    comparable = [x for x in live if x["quiet_share"] > 0.05]
    ray = [x for x in live if x["quiet_share"] <= 0.05]
    assert not any(x["complex_pair"] for x in comparable), \
        [x["taus"] for x in comparable if x["complex_pair"]]
    assert any(x["complex_pair"] for x in ray), [x["taus"] for x in ray]
    # and even on the ray it is rung 67's unobservable mode, not rung 69's visible one
    for x in ray:
        if x["complex_pair"]:
            assert x["zeta"] > 0.98, x


@pytest.mark.slow
def test_the_floor_is_rung67s_damping_ratio_and_that_is_CONTINGENT(cross, gains):
    """**ON THIS PLANT** the floor reduces to rung 67's `zeta = 1/sqrt(1+|P|)`, because `min()`
    selects `pair_RC` — and it selects it only because `pair_RV` came back POSITIVE. So the
    invariance 'a third loop sharing the wall moves the achievable damping nowhere' is
    CONDITIONAL on that sign, not structural: had `pair_RV` been the more negative one, the
    floor would be set by a gain rung 67 never measured.

    The gate asserts the condition ALONGSIDE the consequence, so a plant that broke the sign
    would fail here rather than silently invalidating the identity."""
    assert all(x > 0.0 for x in gains["pair_RV"]), gains["pair_RV"]
    assert gains["worse_pair"] == min(min(gains["pair_RC"]), min(gains["pair_RV"]))
    assert gains["worse_pair"] in gains["pair_RC"], "the floor must be set by rung 67's pair"
    f = cross.split_floor(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                          v_max=V_MAX)
    lo, hi = f["floor_range"]
    P = abs(sum(gains["pair_RC"]) / len(gains["pair_RC"]))
    assert abs(0.5 * (lo + hi) - (1.0 + P) ** -0.5) < 5e-3, (lo, hi, P)


@pytest.mark.slow
def test_the_inherited_rk4_constant_is_conservative_and_that_is_MEASURED(cross):
    """The guard keeps rung 68's constant on a THIRD argument (the non-zero pair is real and
    dominated by the rate sum). Rung 65 published a retraction for a trusted stability argument,
    so `|lam|/sum` is measured along the arc rather than asserted."""
    f = cross.split_floor(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                          v_max=V_MAX)
    assert f["max_mod_ratio"] < 1.0, f["max_mod_ratio"]
    assert f["max_ds_lambda"] < 2.0, f["max_ds_lambda"]


# =============================================================================
# GATE 6 — s 4: THE LEDGER, in TWO currencies, with OPPOSITE-SIGN cross-credits.
# =============================================================================

@pytest.fixture(scope="module")
def bill(cross):
    return cross.split_bill(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                            tau=TAU, tau_gov=TAU_GOV, tau_s=TAU_S, v_max=V_MAX)


@pytest.mark.slow
def test_each_loop_delivers_on_its_OWN_currency(bill):
    """The governor owns `Tt4` and the airflow loops own `phi`. Rung 68's three loops shared ONE
    currency and could only erode each other; here each buys in its own coin."""
    c = bill["cells"]
    assert c["G"]["E"] < 0.5 * c["bare"]["E"], (c["G"]["E"], c["bare"]["E"])
    assert c["V"]["I"] < 0.2 * c["bare"]["I"], (c["V"]["I"], c["bare"]["I"])
    assert c["S"]["I"] < 0.2 * c["bare"]["I"], (c["S"]["I"], c["bare"]["I"])
    assert c["GVS"]["I"] < c["VS"]["I"] and c["GVS"]["E"] < c["VS"]["E"]


@pytest.mark.slow
def test_the_cross_credits_have_OPPOSITE_SIGNS_rung67s_object_with_a_third_loop(bill):
    """RUNG 67's cross-credit, and it survives the third loop: the VALVE debits the temperature
    (`R_q > 0` — bleed makes it hotter at fixed fuel) while the GOVERNOR credits the surge
    margin (`C_g < 0` — clipping fuel raises `phi_lp`). One loop helps the other; the other
    hurts it — an object a one-currency ledger structurally cannot hold."""
    assert bill["marginal_Tt4"]["valve"] < 0.0, bill["marginal_Tt4"]
    assert bill["marginal_phi"]["gov"] > 0.0, bill["marginal_phi"]


@pytest.mark.slow
def test_the_two_phi_loops_erode_each_other_and_the_governor_does_not(bill):
    """RUNG 68's erosion is a property of the SHARED constraint, so it must appear between the
    valve and the stator and NOT between either of them and the governor. Each `phi` loop's
    marginal contribution to the triple is a fraction of what it delivers alone."""
    c = bill["cells"]
    alone_v = c["bare"]["I"] - c["V"]["I"]
    alone_s = c["bare"]["I"] - c["S"]["I"]
    assert bill["marginal_phi"]["valve"] < 0.2 * alone_v, (bill["marginal_phi"], alone_v)
    assert bill["marginal_phi"]["stator"] < 0.2 * alone_s, (bill["marginal_phi"], alone_s)
    # the governor's own currency is NOT eroded by the pair it does not share with
    alone_g = c["bare"]["E"] - c["G"]["E"]
    assert bill["marginal_Tt4"]["gov"] > 0.9 * alone_g, (bill["marginal_Tt4"], alone_g)


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
