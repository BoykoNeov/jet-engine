"""Rung 68 — THREE LOOPS ON ONE VARIABLE: a lagged STATOR limiter beside rung 65's lagged
VALVE and rung 52's lagged FUEL leg, all three holding `phi_lp` to the same `phi_lim`.

THE HEADLINE: `n` loops on one variable are ONE loop with all `n` RATES ADDED. `n` laws that
hold the same variable to the same set point have `dU_i/du_j = -phi_j/phi_i` UNIFORMLY — the
diagonal is not a special case — so `J = -D c r^T` is RANK ONE at every `n`, every plant,
every bandwidth: `n-1` zero eigenvalues and one root at `-sum 1/tau_i`. Rung 66's identity is
the n=2 case of that, not a property of pairs.

THE n>=3 CONTENT IS THE CYCLIC PRODUCT. Rung 66's three pairwise identities leave the 3x3 with
one free parameter, `x = R_q C_v V_g`, and `det = (x+1)^2/x` — so a block can be pairwise-
degenerate and still rank 2. Only `x` (predicted -1) tests JOINT collapse; `tr` is the
hardcoded diagonal and the second invariant is the pairwise result restated.

AND IT EXTENDS RUNG 64. `v_max` — the lever's AUTHORITY, which rung 64 made the ceiling on
protection — is EXACTLY inert on the triple and decisively binding on the same lever alone.
Authority is not a property of a lever; it is a property of the lever plus whatever else holds
the same variable.

THE TWO ARTIFACTS THAT WOULD HAVE COUNTERFEITED THE RUNG, and gates 6 and 7 exist for them:
a SATURATED loop costs the block a zero, so an unfiltered instrument reports a fully
INDEPENDENT triple (the inverse of rung 67's lesson); and rung 66's RK4 constant admits steps
at which this plant reports the floor EXACTLY HELD with a violation integral of zero —
perfect protection, counterfeited.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    ThreeLoopCascadeTransient, TwoLagCascadeTransient, CrossLoopCascadeTransient,
    BleedLimiter, StatorLimiter, SurgeLimiter, AsymmetricLag, BleedSchedule, StatorSchedule,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, PHI, V_MAX = 0.10, 0.80, 0.20
SM = PHI / FLOOR - 1.0
TAU, TAU_S = 0.05, 0.05                 # the valve's and the stator's clocks
TAU_ATT, TAU_REL = 0.05, 0.15           # rung 52's fast-attack / slow-release fuel leg

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _three(design, **kw):
    return ThreeLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _two(design, **kw):
    return TwoLagCascadeTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _keys(traj):
    return [tuple(p[k] for k in ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf"))
            for p in traj]


def _valve(tau=None):
    return BleedLimiter(phi_lim=PHI, b_max=B, tau=tau)


def _stator(tau=TAU_S, v_max=V_MAX):
    return StatorLimiter(phi_lim=PHI, v_max=v_max, tau=tau)


def _fuel():
    return SurgeLimiter.from_margin(LP, "lp", SM)


def _lag(att=TAU_ATT, rel=TAU_REL):
    return AsymmetricLag(tau_att=att, tau_rel=rel)


def _march(m, ds=DS, **kw):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, ds, **kw)[0]


@pytest.fixture(scope="module")
def triple(design):
    """THE rung-68 machine and its march, built once — five states, three clocks."""
    m = _three(design, bleed_lim=_valve(TAU), stator_lim=_stator())
    return m, _march(m, surge=_fuel(), lag=_lag())


# =============================================================================
# GATE 1 — THE REDUCE. The five-state integrator is entered ONLY when the
#          stator carries a clock; every inherited arm must reach the code path
#          it always did, bit-for-bit.
# =============================================================================

def test_reduce_no_stator_is_rung66_bit_for_bit(design):
    """`stator_lim=None` with both other clocks armed: rung 66's cascade, unchanged."""
    a = _march(_three(design, bleed_lim=_valve(TAU)), surge=_fuel(), lag=_lag())
    b = _march(_two(design, bleed_lim=_valve(TAU)), surge=_fuel(), lag=_lag())
    assert _keys(a) == _keys(b)
    assert "v" not in a[0], "rung 66's arm must not carry a fifth state"
    assert "g" in a[0] and "b" in a[0], "...but must still carry rung 66's four"


def test_reduce_inherited_arms_bit_for_bit(design):
    """Rung 65's arm (`lag=None`), rung 52's (no valve) and rung 64's (no clocks at all) all
    leave through the SAME `super().integrate_fuel`, so a rung-68 machine with no stator is
    every one of its ancestors."""
    for kw, march_kw in (({"bleed_lim": _valve(TAU)}, {"surge": _fuel()}),       # rung 65
                         ({}, {"surge": _fuel(), "lag": _lag()}),                # rung 52
                         ({"bleed_lim": _valve()}, {}),                          # rung 64
                         ({"bleed_sched": BleedSchedule(B, 0.65)}, {})):         # rung 62
        a = _march(_three(design, **kw), **march_kw)
        b = _march(_two(design, **kw), **march_kw)
        assert _keys(a) == _keys(b), kw
        assert "v" not in a[0], kw


def test_an_unlagged_stator_is_refused_not_silently_dropped(design):
    """A `StatorLimiter` without `tau` cannot be marched, and dropping it would make every
    reader report a third loop that never acted."""
    with pytest.raises(AssertionError, match="INSTANTANEOUS"):
        StatorLimiter(phi_lim=PHI, v_max=V_MAX, tau=None).__class__  # constructed below
        _stator(tau=0.0)
    m = _three(design, bleed_lim=_valve(TAU), stator_lim=_stator(tau=None))
    a = _march(m, surge=_fuel(), lag=_lag())
    assert "v" not in a[0], "an unlagged stator must not enter the five-state integrator"


def test_the_triple_is_the_only_five_state_path(triple):
    m, traj = triple
    for k in ("v", "v_cmd", "v_regime", "g", "required", "b", "b_cmd"):
        assert k in traj[0], k
    assert len(traj) == 341


def test_at_lever_returns_this_class_and_keeps_the_third_loop(design):
    """The SIXTH instance of the trap rungs 61-66 each hit — and the first where the
    signature GROWS, so the failure mode is also 'silently drops the third loop'."""
    m = _three(design, bleed_lim=_valve(TAU), stator_lim=_stator())
    s = m.at_lever(bleed_lim=_valve(TAU), stator_lim=_stator())
    assert type(s) is ThreeLoopCascadeTransient
    assert s.stator_lim is not None and s.stator_lim.tau == TAU_S
    assert "v" in _march(s, surge=_fuel(), lag=_lag())[0]


def test_one_set_point_is_enforced(design):
    """s 2's identity needs ONE SET POINT, not merely one variable: rung 66 measured a -2.5 %
    offset moving the product to 0.951. Two floors that disagree are a different rung."""
    with pytest.raises(AssertionError, match="ONE SET POINT"):
        _three(design, bleed_lim=_valve(TAU),
               stator_lim=StatorLimiter(phi_lim=0.78, v_max=V_MAX, tau=TAU_S))


def test_three_loops_on_two_variables_is_refused(design):
    """Rung 47's `tau_gov` watches `Tt4` — adding it here is THREE loops on TWO variables,
    which superposes rung 67's P<0 block onto this rank-one one. Rung 68's own next seam."""
    m = _three(design, bleed_lim=_valve(TAU), stator_lim=_stator())
    with pytest.raises(AssertionError, match="THREE loops on TWO variables"):
        _march(m, surge=_fuel(), lag=_lag(), Tt4_max=1200.0, tau_gov=0.05)


# =============================================================================
# GATE 2 — THE CYCLIC PRODUCT. The one quantity at n>=3 that the pairwise
#          identities do NOT force, and the reason a third loop had to be built
#          rather than argued.
# =============================================================================

def test_cyclic_product_is_minus_one_and_the_pairs_are_one(triple):
    m, _ = triple
    g = m.triple_gains(FLIGHT, LO, HI, sm=SM)
    assert g["n_riding"] >= 50, g["n_riding"]
    assert len(g["rows"]) >= 8
    for row in g["rows"]:
        on = row["on"]
        assert abs(on["cyclic"] + 1.0) < 1e-6, (row["s"], on["cyclic"])
        for k in ("pair_RC", "pair_RV", "pair_CV"):
            assert abs(on[k] - 1.0) < 1e-6, (row["s"], k, on[k])


def test_the_cyclic_product_is_not_implied_by_the_pairwise_ones():
    """THE GATE THAT KEEPS GATE 2 FROM BEING A TAUTOLOGY. With the three pairwise identities
    imposed exactly, the block still has a free parameter: build one whose pairs are all 1 and
    whose cyclic product is NOT -1, and check `det != 0`. If this ever passes with det == 0,
    the cyclic measurement above is measuring nothing."""
    a, c = -7.0e-2, -1.0 / 7.0e-2          # ac = 1
    b, e = 4.0e-2, 1.0 / 4.0e-2            # be = 1
    d, f = 2.0, 0.5                        # df = 1
    x = a * d * e                          # the cyclic product, free
    det = (-1.0 * ((-1.0) * (-1.0) - d * f) - a * (c * (-1.0) - d * e)
           + b * (c * f - (-1.0) * e))
    assert abs(x + 1.0) > 0.1, "this hand-built block must NOT be cyclically degenerate"
    assert abs(det) > 0.1, "...and must therefore be rank 3 despite all pairs being 1"
    assert det == pytest.approx((x + 1.0) ** 2 / x, rel=1e-9), (
        "det is a monotone re-expression of the cyclic product — which is why the spec quotes "
        "x and not det, tr or the second invariant")


def test_the_detector_resolves_far_below_what_it_claims(triple):
    """MEASURE THE DETECTOR, DO NOT ASSERT THE NULL. Displacing the stator off the shared
    manifold by `delta` must move the departure LINEARLY and far above the noise floor —
    otherwise `cyclic == -1` is a statement about the instrument, not the plant."""
    m, _ = triple
    s = m.cyclic_sensitivity(FLIGHT, LO, HI, sm=SM)
    assert s["floor"] < 1e-7, s["floor"]
    assert 1.0 < s["gain"] < 2.0, s["gain"]
    rows = {r["delta"]: r["dep"] for r in s["rows"] if r["dep"] is not None}
    assert abs(rows[1e-3]) > 100.0 * s["floor"], (rows[1e-3], s["floor"])
    # LINEARITY: a decade in delta is a decade in the departure
    assert abs(rows[1e-2] / rows[1e-3]) == pytest.approx(10.0, rel=0.05)
    assert abs(rows[1e-3] / rows[1e-4]) == pytest.approx(10.0, rel=0.05)


# =============================================================================
# GATE 3 — THE SPECTRUM: n-1 = 2 zeros, and the RATES ADD at n = 3.
# =============================================================================

@pytest.mark.slow
def test_two_zero_eigenvalues_and_the_rates_add(triple):
    """`tr J = -sum 1/tau_i` is the ODE's own diagonal and is NOT a measurement. What IS
    measured is that the other two roots vanish — equivalently that the second invariant
    (the three PAIRWISE identities, weighted) and the determinant (the CYCLIC one) are both
    zero. The dominant root then equals the rate sum as a consequence."""
    m, _ = triple
    r = m.triple_modes(FLIGHT, LO, HI, sm=SM)
    assert len(r["arms"]) == 4
    for arm in r["arms"]:
        assert arm["rows"], arm["taus"]
        assert arm["skipped"] <= 2, (arm["taus"], arm["skipped"])
        scale = abs(arm["rate_sum"])
        assert arm["worst_zero"] < 1e-4 * scale, (arm["taus"], arm["worst_zero"])
        for x in arm["rows"]:
            assert x["dom"] == pytest.approx(arm["rate_sum"], rel=1e-4), arm["taus"]
            assert abs(x["cyclic"] + 1.0) < 1e-6


# =============================================================================
# GATE 4 — WHAT THE TRIPLE DELIVERS. All three marginals, and both walls.
# =============================================================================

@pytest.fixture(scope="module")
def bill(triple):
    m, _ = triple
    return m.triple_bill(FLIGHT, LO, HI, sm=SM)


@pytest.mark.slow
def test_the_pair_beats_every_single_and_the_triple_beats_every_pair(bill):
    c = bill["cells"]
    for one in ("F", "V", "S"):
        assert c["FVS"]["I"] < c[one]["I"], one
    for two in ("FV", "FS", "VS"):
        assert c["FVS"]["I"] < c[two]["I"], two


@pytest.mark.slow
def test_strongly_subadditive_and_the_ordering_is_the_object(bill):
    """Rung 66 s 9 predicted the third limiter would buy LESS than the second's 1.59 %. It
    does not, and the reason is that credit is not a function of the rate sum: rung 66's own
    two marginals differed by 21x while BOTH doubled it. All three are quoted."""
    assert bill["sum_singles"] > 2.4 * bill["delivered"]
    m = bill["marginal"]
    assert m["fuel"] < m["stator"] < m["valve"], m
    e = bill["erosion"]
    assert max(e.values()) / min(e.values()) > 4.0, e     # 122x vs 10x, measured
    assert m["stator"] > 1.59, ("the seam's magnitude prediction is a MISS", m["stator"])


@pytest.mark.slow
def test_the_credit_flips_sign_between_the_two_walls(bill):
    """RUNG 53's *a margin is a DISTANCE*, landing on a ledger. The stator MOVES the `phi`
    wall and leaves the metal one alone, so a credit quoted without its wall is meaningless:
    the same loop is strongly protective in `phi` and actively harmful in incidence."""
    assert bill["marginal"]["stator"] > 0.0
    assert bill["marginal_incidence"]["stator"] < 0.0
    assert bill["cells"]["S"]["credit"] > 80.0
    assert bill["cells"]["S"]["credit_inc"] < 0.0
    # the valve, which does NOT move either wall, keeps its sign in both
    assert bill["marginal"]["valve"] > 0.0 and bill["marginal_incidence"]["valve"] > 0.0


# =============================================================================
# GATE 5 — v_max: INERT in company, BINDING alone. This EXTENDS rung 64.
# =============================================================================

@pytest.mark.slow
def test_authority_is_inert_on_the_triple_and_binds_on_the_lever_alone(design):
    """Rung 64: *a limiter's LAW cannot buy PROTECTION, only its PRICE — the ceiling is the
    lever's AUTHORITY.* That is a statement about a lever ALONE. Here the SAME ceiling is
    EXACTLY inert once two other loops hold the same variable, because they take up the demand
    before the stop is reached."""
    def run(v_max, valve):
        m = _three(design, bleed_lim=_valve(TAU) if valve else None,
                   stator_lim=_stator(v_max=v_max))
        t = _march(m, surge=_fuel() if valve else None, lag=_lag() if valve else None)
        return (m._violation(t, PHI, R), min(p["v"] for p in t),
                any(p["v_regime"] == "saturated" for p in t))

    trip = {vm: run(vm, True) for vm in (0.05, 0.10, 0.20)}
    alone = {vm: run(vm, False) for vm in (0.05, 0.10, 0.20)}
    # IN COMPANY: identical to the ROOT TOLERANCE across a 4x ceiling, and never on the stop.
    # NOT bit-for-bit, and the reason is disclosed rather than tuned away: `v_max` is one end
    # of `_solve_v`'s bracket, so it moves `_illinois`'s first secant and the converged root
    # lands ~1e-15 apart. That is the solver's own resolution, four orders below the 1.4x
    # effect the same ceiling has on the lever ALONE.
    assert trip[0.05][0] == pytest.approx(trip[0.20][0], rel=1e-12)
    assert trip[0.10][0] == pytest.approx(trip[0.20][0], rel=1e-12)
    assert not any(t[2] for t in trip.values())
    # ALONE: the ceiling is decisive, and it saturates
    assert alone[0.05][0] > 1.4 * alone[0.20][0]
    assert alone[0.05][2] and alone[0.10][2]
    # and a TIGHT enough ceiling reaches even the triple, so the inertness is a MEASUREMENT
    tight = run(0.02, True)
    assert tight[2] and tight[0] > 1.2 * trip[0.20][0]


# =============================================================================
# GATE 6 — THE SATURATION CONFOUND. A stop costs the block a zero, so the
#          unfiltered instrument reports a fully INDEPENDENT triple.
# =============================================================================

@pytest.mark.slow
def test_a_saturated_loop_costs_the_block_a_zero(triple):
    """The INVERSE of rung 67's lesson (*a zero cross-gain is saturation, never decoupling*):
    there a stop faked the absence of COUPLING in one entry; here it fakes the absence of
    REDUNDANCY in the whole block. This is why every reader filters on the REGIME LABEL."""
    m, _ = triple
    r = m.saturation_counterfeit(FLIGHT, LO, HI, sm=SM)
    assert r["n_saturated"] > 10 and r["n_riding"] > 10
    sat = [x for x in r["rows"] if x["regime"] == "saturated"][0]
    rid = [x for x in r["rows"] if x["regime"] == "riding"][0]
    assert sat["V_g"] == 0.0 and sat["V_q"] == 0.0, "a stop returns EXACT zeros, measured"
    assert sat["off_regime"], "and the filter must have flagged it"
    assert sat["n_zero"] == 0, "the unfiltered block reads as fully INDEPENDENT"
    assert rid["n_zero"] == 2 and not rid["off_regime"], "the riding one keeps both zeros"


def test_a_float_comparison_against_the_stop_is_not_the_regime(design):
    """The trap made concrete: `v < 0` is TRUE for a saturated stator and for a riding one
    alike, so a reader that infers the regime from the float would admit both."""
    m = _three(design, bleed_lim=_valve(TAU), stator_lim=_stator(v_max=0.02))
    t = _march(m, surge=_fuel(), lag=_lag())
    sat = [p for p in t if p["v_regime"] == "saturated"]
    rid = [p for p in t if p["v_regime"] == "riding"]
    assert sat and rid
    assert all(p["v"] < 0.0 for p in sat) and all(p["v"] < 0.0 for p in rid), (
        "if this ever fails the trap has gone away and this gate is dead weight")
    assert {p["v_regime"] for p in t} <= {"dormant", "riding", "saturated"}


# =============================================================================
# GATE 7 — THE RK4 FLOOR. Rung 66's constant admits steps at which this plant
#          counterfeits PERFECT PROTECTION.
# =============================================================================

def test_the_floor_is_tighter_than_rung_66s_and_it_fires(design):
    m = _three(design, bleed_lim=_valve(TAU), stator_lim=_stator())
    ds = 0.04                       # ds*(1/t_g+1/t_v) = 1.6 <= 2 (rung 66 ADMITS this)
    assert ds * (1.0 / TAU + 1.0 / min(TAU_ATT, TAU_REL)) <= 2.0
    assert ds * (1.0 / TAU + 1.0 / min(TAU_ATT, TAU_REL) + 1.0 / TAU_S) > 2.0
    with pytest.raises(AssertionError, match="RATES ADD"):
        _march(m, ds=ds, surge=_fuel(), lag=_lag())


@pytest.mark.slow
def test_what_the_refusal_refuses_is_measured_not_trusted(design):
    """AN ASSERT NOBODY HAS RUN PAST IS A TAUTOLOGY (rung 67 gate 9). The guard is overridden
    to a no-op and the refused band measured: at `ds = 0.05` — which rung 66's own constant
    admits — the march reports `min phi_lp` EXACTLY at the floor and a violation integral of
    ZERO. It does not blow up like rung 65's retraction did; it counterfeits perfect
    protection, which is worse."""
    class Unguarded(ThreeLoopCascadeTransient):
        @staticmethod
        def _rk4_floor(ds, rate, n_states, tau_s):
            return None

    def run(cls, ds):
        m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                bleed_lim=_valve(TAU), stator_lim=_stator())
        t = _march(m, ds=ds, surge=_fuel(), lag=_lag())
        return m._violation(t, PHI, R), min(p["phi_lp"] for p in t)

    fine = run(ThreeLoopCascadeTransient, 0.003125)
    edge = run(ThreeLoopCascadeTransient, 0.03125)          # inside BOTH constants
    bad = run(Unguarded, 0.05)                              # inside rung 66's, outside this
    assert fine[0] > 0.0 and edge[0] > 0.0
    assert edge[0] < fine[0] and edge[0] > 0.9 * fine[0]    # degraded but still a number
    assert bad[0] == 0.0, "the counterfeit: no violation at all"
    assert bad[1] == pytest.approx(PHI, abs=1e-9), "...and the floor exactly held"


# =============================================================================
# GATE 8 — THE LIMITS. tau_s -> INFINITY removes the loop; tau_s -> 0 does NOT.
# =============================================================================

@pytest.mark.slow
def test_the_converging_limit_is_the_slow_one(design):
    """INVERTS every earlier lag in this family. Rungs 65/66 send a clock to ZERO to recover
    the instantaneous loop, so there the fast limit is the richer object. A third loop is an
    ADDITION, so only the SLOW limit removes it."""
    ref = _two(design, bleed_lim=_valve(TAU))
    t66 = _march(ref, surge=_fuel(), lag=_lag())
    I66 = ref._violation(t66, PHI, R)

    def I_at(tau_s, ds=DS):
        m = _three(design, bleed_lim=_valve(TAU), stator_lim=_stator(tau=tau_s))
        return m._violation(_march(m, ds=ds, surge=_fuel(), lag=_lag()), PHI, R)

    slow = [I_at(t) for t in (0.5, 2.0, 10.0, 500.0)]
    assert slow == sorted(slow), "monotone in tau_s"
    assert abs(slow[-1] / I66 - 1.0) < 1e-3, slow[-1]
    assert abs(slow[0] / I66 - 1.0) > 0.05, "...and not already there at tau_s = 0.5"
    assert I_at(0.02) < 0.7 * I66, "the FAST limit runs the other way — a different object"


# =============================================================================
# GATE 9 — THE INITIAL CONDITION is a FAMILY, and the member is DECLARED.
# =============================================================================

@pytest.mark.slow
def test_the_declared_start_is_rung_66s_member_and_the_family_is_real(triple):
    """Rung 66's joint solve converged in one iteration because ITS march opened dormant. That
    escape is gone at n=3 — the valve and the stator are both live at s=0 and they share the
    constraint — so the s=0 fixed points are a CURVE. From the DECLARED start every sweep
    order lands on the same member; the family shows up when the START moves, which is rung
    66 s 0's own diagnosis (non-uniqueness of the IC, not a stalled solve)."""
    m, traj = triple
    f = m.ic_family(FLIGHT, LO, HI, sm=SM)
    assert traj[0]["g"] == 0.0 and traj[0]["v"] == 0.0
    assert traj[0]["b"] == pytest.approx(0.036626, abs=1e-5)   # rung 66's own b0
    assert traj[0]["ic_iters"] == 1 and traj[0]["ic_res"] == 0.0
    assert f["order_members"] == 1, "the order is NOT the lever from the declared start"
    assert all(x["iters"] == 1 for x in f["by_order"].values())
    assert f["start_spread_I"] > 0.5, f["start_spread_I"]
    assert f["start_spread_withheld"] > 1.0, f["start_spread_withheld"]


def test_an_out_of_band_start_is_refused(triple):
    m, _ = triple
    with pytest.raises(AssertionError, match="stator POSITION"):
        _march(m, surge=_fuel(), lag=_lag(), v0=0.05)          # v0 > 0 is out of the band
    with pytest.raises(AssertionError, match="permutation"):
        _march(m, surge=_fuel(), lag=_lag(), ic_order="ggv")


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
