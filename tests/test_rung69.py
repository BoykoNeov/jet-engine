"""Rung 69 — THE REFERENCE SPLIT: rung 68's SAME stator, referenced to INCIDENCE instead of to
`phi`, beside the SAME lagged valve (65) and the SAME lagged fuel leg (52). Five states, three
clocks, one lever, one physical wall at the design setting. **Only the COORDINATE moves.**

THE HEADLINE: a loop's COORDINATE, not its actuator, decides whether it adds a ZERO or a RANK.
Every row of the actuator block is a multiple of ITS OWN constraint's gradient, so
`rank M = dim span{grad c^(i)}` and **ZEROS = n - m**, with `m` the number of INDEPENDENT
CONSTRAINTS. The loop count never enters. Rung 66 (n=2,m=1) 1 zero; rung 67 (n=2,m=2) 0; rung
68 (n=3,m=1) 2; this rung (n=3,m=2) **1**.

AND IT CORRECTS HOW RUNG 68's DECOMPOSITION MUST BE READ. The two loops that still share `phi`
keep exactly PARALLEL rows, so `det J = 0` IDENTICALLY under both references — **`det` is
BLIND to the split**, and rung 68's `c0 = (x+1)^2/x...` does not survive it. What moves is the
SECOND invariant `c1`, by twelve orders of magnitude.

THE MODE THE SPLIT CREATES. The freed root does not land on the real axis: the surviving pair
is COMPLEX for some bandwidth **iff `k < 0`, i.e. iff the lever FIGHTS ITSELF across the two
walls**, and `zeta >= 1/sqrt(1-k)` for EVERY choice of the three clocks. One scalar `k` sets
the pairwise split, the cyclic product AND the damping floor — rung 67's `P` in a different
mechanism.

AND THE LEDGER'S WHOLE SIGN TABLE FLIPS: the same lever, same plant, same wall at the design
setting, protective or harmful in each currency according to which one its LOOP watches.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    ReferenceSplitTransient, ThreeLoopCascadeTransient,
    BleedLimiter, StatorLimiter, StatorIncidenceLimiter, SurgeLimiter, AsymmetricLag,
    BleedSchedule,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, PHI, V_MAX = 0.10, 0.80, 0.20
SM = PHI / FLOOR - 1.0
TAU, TAU_S = 0.05, 0.05
TAU_ATT, TAU_REL = 0.05, 0.15

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
T_C = LP.tan_beta1_crit()
M_LIM = T_C - 1.0 / PHI                      # THE SAME WALL, read at the design setting


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _split(design, **kw):
    return ReferenceSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _three(design, **kw):
    return ThreeLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _keys(traj):
    return [tuple(p[k] for k in ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf"))
            for p in traj]


def _valve(tau=None):
    return BleedLimiter(phi_lim=PHI, b_max=B, tau=tau)


def _inc(tau=TAU_S, v_max=V_MAX):
    return StatorIncidenceLimiter.from_margin(LP, v_max, SM, tau=tau)


def _phi_stator(tau=TAU_S, v_max=V_MAX):
    return StatorLimiter.from_margin(LP, v_max, SM, tau=tau)


def _fuel():
    return SurgeLimiter.from_margin(LP, "lp", SM)


def _lag(att=TAU_ATT, rel=TAU_REL):
    return AsymmetricLag(tau_att=att, tau_rel=rel)


def _march(m, ds=DS, **kw):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS if ds is None else ds, **kw)[0]


@pytest.fixture(scope="module")
def split(design):
    """THE rung-69 machine and its march — the incidence-referenced third loop."""
    m = _split(design, bleed_lim=_valve(TAU), stator_inc=_inc())
    return m, _march(m, surge=_fuel(), lag=_lag())


# =============================================================================
# GATE 1 — THE REDUCE, and THE BAND FLIP. Rung 69 changes a coordinate, so every
#          rung-68 arm must be reached bit-for-bit and the one-sided band must
#          run the OTHER way — the failure mode that raises nothing.
# =============================================================================

def test_reduce_no_incidence_stator_is_rung68_bit_for_bit(design):
    """`stator_inc=None` with a rung-68 phi stator armed: rung 68's own five-state cascade."""
    a = _march(_split(design, bleed_lim=_valve(TAU), stator_lim=_phi_stator()),
               surge=_fuel(), lag=_lag())
    b = _march(_three(design, bleed_lim=_valve(TAU), stator_lim=_phi_stator()),
               surge=_fuel(), lag=_lag())
    assert _keys(a) == _keys(b)
    assert "v" in a[0] and a[0]["v"] == 0.0
    assert min(p["v"] for p in a) < 0.0, "rung 68's band is the NEGATIVE one"


def test_reduce_inherited_arms_bit_for_bit(design):
    """Rung 66's (no stator at all), rung 65's, rung 52's, rung 64's and rung 62's arms all
    leave through the same `super()`, so a rung-69 machine with no stator is every ancestor."""
    for kw, march_kw in (({"bleed_lim": _valve(TAU)}, {"surge": _fuel(), "lag": _lag()}),
                         ({"bleed_lim": _valve(TAU)}, {"surge": _fuel()}),
                         ({}, {"surge": _fuel(), "lag": _lag()}),
                         ({"bleed_lim": _valve()}, {}),
                         ({"bleed_sched": BleedSchedule(B, 0.65)}, {})):
        a = _march(_split(design, **kw), **march_kw)
        b = _march(_three(design, **kw), **march_kw)
        assert _keys(a) == _keys(b), kw
        assert "v" not in a[0], kw


def test_the_band_flips_and_an_out_of_band_start_is_refused(split):
    """`M_i` is INCREASING in `v` where `phi_lp` DECREASES, so the admissible band is
    `[0, +v_max]` — the MIRROR of rung 68's. Getting the orientation wrong returns a wrong
    regime label with nothing raising (rung 62's `_powers` trap, fifth reload), so the band is
    gated from BOTH sides."""
    m, traj = split
    assert all(p["v"] >= 0.0 for p in traj), "the incidence loop CLOSES the stators"
    assert max(p["v"] for p in traj) > 0.0, "...and it actually moved"
    with pytest.raises(AssertionError, match="stator POSITION"):
        _march(m, surge=_fuel(), lag=_lag(), v0=-0.05)        # rung 68's side is out of band
    assert _march(m, surge=_fuel(), lag=_lag(), v0=0.05)      # this one is not


def test_one_stator_one_reference(design):
    with pytest.raises(AssertionError, match="ONE reference"):
        _split(design, bleed_lim=_valve(TAU), stator_lim=_phi_stator(), stator_inc=_inc())


def test_one_physical_wall_is_enforced_not_one_float(design):
    """Across a change of coordinate, "one set point" can only mean ONE PHYSICAL WALL: the
    incidence floor must BE the valve's phi floor at the design setting. An offset here would
    confound the reference split with a set-point offset."""
    assert _inc().m_lim == pytest.approx(M_LIM, abs=1e-15)
    assert _inc().phi_lim_at(LP) == pytest.approx(PHI, rel=1e-15)
    with pytest.raises(AssertionError, match="ONE PHYSICAL WALL"):
        _split(design, bleed_lim=_valve(TAU),
               stator_inc=StatorIncidenceLimiter(m_lim=M_LIM + 0.01, v_max=V_MAX, tau=TAU_S))


def test_an_unlagged_incidence_stator_is_refused(design):
    with pytest.raises(AssertionError, match="INSTANTANEOUS"):
        StatorIncidenceLimiter(m_lim=M_LIM, v_max=V_MAX, tau=0.0)
    m = _split(design, bleed_lim=_valve(TAU), stator_inc=_inc(tau=None))
    assert "v" not in _march(m, surge=_fuel(), lag=_lag())[0]


def test_at_lever_keeps_the_reference(design):
    """THE SEVENTH instance of the trap rungs 61-68 each hit — and the second in a row where
    the signature GROWS, so 'silently swaps the REFERENCE' joins 'silently drops the loop'."""
    m = _split(design, bleed_lim=_valve(TAU), stator_inc=_inc())
    s = m.at_lever(bleed_lim=_valve(TAU), stator_inc=_inc())
    assert type(s) is ReferenceSplitTransient
    assert s.stator_inc is not None and s.stator_lim is None
    assert all(p["v"] >= 0.0 for p in _march(s, surge=_fuel(), lag=_lag()))


def test_a_float_comparison_against_the_stop_is_not_the_regime(design):
    """Rung 68's trap, mirrored: `v > 0` is TRUE for a saturated incidence stator and for a
    riding one alike, so a reader that infers the regime from the float would admit both."""
    m = _split(design, bleed_lim=None, stator_inc=_inc(v_max=0.02))
    t = _march(m)
    sat = [p for p in t if p["v_regime"] == "saturated"]
    rid = [p for p in t if p["v_regime"] == "riding"]
    assert sat and rid
    assert all(p["v"] > 0.0 for p in sat) and all(p["v"] > 0.0 for p in rid)
    assert {p["v_regime"] for p in t} <= {"dormant", "riding", "saturated"}


# =============================================================================
# GATE 2 — THE PAIRWISE SPLIT. Which pairs keep rung 66's identity reads off
#          WHICH LOOPS SHARE A CONSTRAINT.
# =============================================================================

@pytest.fixture(scope="module")
def gains(split):
    m, _ = split
    return m.reference_gains(FLIGHT, LO, HI, sm=SM)


def test_the_shared_pair_survives_and_the_split_pairs_do_not(gains):
    """`pair_RC` — fuel and valve, still both on `phi` — must stay at 1 to the root-finders'
    floor under BOTH references, while `pair_RV` and `pair_CV` move to `k` under the split
    only. That contrast at ONE base point on ONE trajectory is the whole measurement."""
    assert gains["n_riding"] >= 40, gains["n_riding"]
    assert len(gains["rows"]) >= 6 and not gains["skipped"]
    assert gains["worst_RC_inc"] < 1e-8, gains["worst_RC_inc"]
    assert gains["worst_RC_phi"] < 1e-8, gains["worst_RC_phi"]
    for x in gains["rows"]:
        i, p = x["inc"], x["phi"]
        # the rung-68 reference at the SAME point: every pair at 1, cyclic at -1
        for kk in ("pair_RC", "pair_RV", "pair_CV"):
            assert abs(p[kk] - 1.0) < 1e-6, (x["s"], kk, p[kk])
        assert abs(p["cyclic"] + 1.0) < 1e-6
        # the split: RC survives, RV and CV do not, and the cyclic product FLIPS SIGN
        assert abs(i["pair_RV"] - x["k"]) < 0.01 * abs(x["k"])
        assert abs(i["pair_CV"] - x["k"]) < 0.01 * abs(x["k"])
        assert i["pair_RV"] < -1.5 and i["pair_CV"] < -1.5, (x["s"], i["pair_RV"])
        assert i["cyclic"] > 1.5, (x["s"], i["cyclic"])


def test_the_two_split_pairs_take_the_SAME_value_and_that_is_a_measurement(gains):
    """`pair_RV == pair_CV` is NOT general to a split — it holds iff the odd constraint depends
    on `(g, q)` ONLY THROUGH the shared one, which `M_i = T_c - 1/phi + v` does. So this
    equality measures that the two walls differ by exactly the LEVER'S OWN direct channel, and
    it is what gives `k` its closed form `(phi_v/phi^2)/psi_v`."""
    assert gains["worst_pair_gap"] < 0.01, gains["worst_pair_gap"]
    lo, hi = gains["k_range"]
    assert -2.1 < lo < hi < -1.5, gains["k_range"]


def test_the_evaluation_manifold_is_forced_and_the_alternative_is_reported(gains):
    """There is no point where all three constraints hold (`phi = phi_lim` and `M_i = m_lim`
    force `v = 0`, the dormant stop), so the base MUST be the SHARED constraint's manifold —
    rung 68's instrument unchanged. Read at the STATOR's own root instead, `pair_RC` degrades
    by orders. Reported, never gated on, and this gate is what keeps that honest."""
    assert gains["worst_RC_own"] > 1e-3, gains["worst_RC_own"]
    assert gains["worst_RC_own"] > 1e5 * gains["worst_RC_inc"]


# =============================================================================
# GATE 3 — ZEROS = n - m. ONE zero here, TWO under rung 68's reference, on the
#          same rig and the same clock grid.
# =============================================================================

@pytest.mark.slow
def test_the_rank_is_the_constraint_count_not_the_loop_count(split):
    m, _ = split
    r = m.reference_modes(FLIGHT, LO, HI, sm=SM)
    assert len(r["arms"]) == 4
    for arm in r["arms"]:
        inc, phi = arm["refs"]["inc"], arm["refs"]["phi"]
        assert inc["rows"] and phi["rows"], arm["taus"]
        assert inc["skipped"] <= 2 and phi["skipped"] <= 2, arm["taus"]
        assert inc["zeros"] == [1], (arm["taus"], inc["zeros"])       # n - m = 3 - 2
        assert phi["zeros"] == [2], (arm["taus"], phi["zeros"])       # n - m = 3 - 1
        for x in phi["rows"]:
            assert x["zeta"] == pytest.approx(1.0), "rung 68's spectrum is REAL"


@pytest.mark.slow
def test_det_is_blind_to_the_split_and_c1_is_the_discriminator(split):
    """A reader that inherited rung 68's determinant test would report rank one and see
    NOTHING. Both invariants are read against the rate sum's own power, because 'zero' without
    its scale is not a measurement."""
    m, _ = split
    r = m.reference_modes(FLIGHT, LO, HI, sm=SM)
    for arm in r["arms"]:
        inc, phi = arm["refs"]["inc"], arm["refs"]["phi"]
        assert inc["max_c0_rel"] < 1e-8, (arm["taus"], inc["max_c0_rel"])
        assert phi["max_c0_rel"] < 1e-8, (arm["taus"], phi["max_c0_rel"])
        assert inc["min_c1_rel"] > 0.1, (arm["taus"], inc["min_c1_rel"])
        assert phi["min_c1_rel"] < 1e-10, (arm["taus"], phi["min_c1_rel"])
        assert inc["min_c1_rel"] / max(phi["min_c1_rel"], 1e-300) > 1e9


def test_a_determinant_provably_cannot_see_a_split():
    """THE GATE THAT KEEPS GATE 3's `c0` READING FROM BEING A COINCIDENCE. Hand-build the two
    blocks the algebra predicts and check `det == 0` in both while the RANK is 2 — so `det = 0`
    carries no information about the third row at all. Rung 68's own tautology-killer, one
    level up: there the danger was a measurement implied by the pairwise identities, here it is
    a measurement implied by nothing.

    Block A: a GENERIC second constraint (`pair_RV != pair_CV`).
    Block B: this plant's, where the odd constraint depends on (g,q) only through `phi` — then
             and only then do the two split pairs coincide at `k`."""
    def block(phi_g, phi_q, phi_v, psi_g, psi_q, psi_v):
        return [[-1.0, -phi_q / phi_g, -phi_v / phi_g],
                [-phi_g / phi_q, -1.0, -phi_v / phi_q],
                [-psi_g / psi_v, -psi_q / psi_v, -1.0]]

    def det(M):
        return (M[0][0] * (M[1][1] * M[2][2] - M[1][2] * M[2][1])
                - M[0][1] * (M[1][0] * M[2][2] - M[1][2] * M[2][0])
                + M[0][2] * (M[1][0] * M[2][1] - M[1][1] * M[2][0]))

    pg, pq, pv, phi = 2.0, 3.0, 5.0, 0.8
    A = block(pg, pq, pv, 1.0, 7.0, 1.0)                       # generic psi
    B = block(pg, pq, pv, pg / phi ** 2, pq / phi ** 2, pv / phi ** 2 + 1.0)   # this plant's
    def minors(r, t):
        """The three 2x2 minors of the 2x3 [r; t] — ALL of them zero iff the rows are
        parallel. Checking ONE would fail here for a reason that is itself the finding: in this
        plant rows 0 and 2 AGREE in the `(g, q)` columns up to scale (`psi` depends on them
        only through `phi`) and differ ONLY in the lever's own `v` column."""
        return [abs(r[i] * t[j] - r[j] * t[i]) for i, j in ((0, 1), (0, 2), (1, 2))]

    for M, name in ((A, "generic"), (B, "this plant")):
        assert abs(det(M)) < 1e-12, name
        # rank 2: rows 0 and 1 exactly parallel, row 2 NOT in their span
        assert max(minors(M[0], M[1])) < 1e-12, name
        assert max(minors(M[0], M[2])) > 1e-6, name
    assert minors(B[0], B[2])[0] < 1e-12, "the split is carried by the `v` column alone"
    assert minors(A[0], A[2])[0] > 1e-6, "...which is a property of THIS psi, not of a split"
    k = (pv / phi ** 2) / (pv / phi ** 2 + 1.0)
    assert B[0][2] * B[2][0] == pytest.approx(k)                # pair_RV
    assert B[1][2] * B[2][1] == pytest.approx(k)                # pair_CV — the SAME value
    assert B[0][1] * B[1][2] * B[2][0] == pytest.approx(-k)     # cyclic
    assert A[0][2] * A[2][0] != pytest.approx(A[1][2] * A[2][1])   # generic: NOT the same


# =============================================================================
# GATE 4 — THE DAMPING FLOOR. One scalar `k` sets the ring, and no bandwidth
#          can beat it.
# =============================================================================

@pytest.mark.slow
def test_the_damping_floor_is_bandwidth_independent_and_binds_at_A_equals_z(split):
    """`zeta = (A+z)/(2 sqrt(A z (1-k))) >= 1/sqrt(1-k)` by AM-GM, with equality at
    `A = 1/tau_g + 1/tau_q == 1/tau_s = z`. The grid straddles `A/z = 1` from 1 to 4."""
    m, _ = split
    d = m.damping_floor(FLIGHT, LO, HI, sm=SM)
    live = [x for x in d["rows"] if "zeta" in x]
    assert len(live) == len(d["rows"]) >= 6
    assert d["holds"], [(x["taus"], x["zeta"], x["floor"]) for x in live]
    assert d["worst_pred_err"] < 1e-3, d["worst_pred_err"]
    assert all(x["complex_pair"] for x in live), "k < 0 => the pair RINGS on this grid"
    at1 = [x for x in live if abs(x["A_over_z"] - 1.0) < 1e-12]
    assert len(at1) == 2
    for x in at1:
        assert x["zeta"] == pytest.approx(x["floor"], rel=1e-9)   # the floor is REACHED
    for x in live:
        if x["A_over_z"] > 1.5:
            assert x["zeta"] > 1.02 * x["floor"], (x["taus"], x["zeta"], x["floor"])
    # BANDWIDTH-INDEPENDENT: the two A/z == 1 arms differ 2x in every clock, same zeta
    assert at1[0]["zeta"] == pytest.approx(at1[1]["zeta"], rel=0.02)
    assert at1[0]["rate_sum"] != pytest.approx(at1[1]["rate_sum"], rel=0.1)


@pytest.mark.slow
def test_a_slow_enough_stator_takes_the_pair_back_onto_the_real_axis(split):
    """The window is REAL and has EDGES: `zeta < 1` needs `(A+z)^2 < 4Az(1-k)`, so a clock
    ratio far from 1 puts the pair back on the axis. Rung 68's own clock grid contains such an
    arm (`tau_g = 0.005` => `A/z = 11`), which is what makes 'complex' a measurement."""
    m, _ = split
    r = m.reference_modes(FLIGHT, LO, HI, sm=SM)
    by = {a["taus"]: a["refs"]["inc"] for a in r["arms"]}
    assert by[(0.05, 0.05, 0.05)]["all_complex"] is True
    assert by[(0.005, 0.05, 0.05)]["all_complex"] is False, "A/z = 11 is outside the window"
    assert by[(0.005, 0.05, 0.05)]["zeros"] == [1], "...but the RANK does not care"


# =============================================================================
# GATE 5 — THE LEDGER. The whole sign table flips with the reference.
# =============================================================================

@pytest.fixture(scope="module")
def bill(split):
    m, _ = split
    return m.reference_bill(FLIGHT, LO, HI, sm=SM)


@pytest.mark.slow
def test_the_stator_free_cells_are_identical_between_the_references(bill):
    """`bare`, `F`, `V` and `FV` carry no stator, so they CANNOT differ — a free check that the
    two ledgers come from one rig and are differenceable (rung 63's lesson)."""
    assert bill["common_max_rel"] == 0.0, bill["common"]


@pytest.mark.slow
def test_the_credit_sign_table_flips_with_the_reference(bill):
    """RUNG 53's *a margin is a DISTANCE*, one level up. Rung 68 showed a credit is meaningless
    without its WALL. Here the same lever on the same plant, against the same two walls, is
    protective or harmful according to which wall its LOOP watches — so a credit needs its
    loop's REFERENCE named too."""
    c = bill["stator_credit"]
    # phi-referenced (rung 68): protective in phi, HARMFUL in incidence
    assert c["phi"]["alone"] > 80.0 and c["phi"]["alone_inc"] < -40.0
    assert c["phi"]["marginal"] > 0.0 and c["phi"]["marginal_inc"] < 0.0
    # incidence-referenced (rung 69): the MIRROR, in every one of the four cells
    assert c["inc"]["alone"] < -80.0 and c["inc"]["alone_inc"] > 50.0
    assert c["inc"]["marginal"] < 0.0 and c["inc"]["marginal_inc"] > 0.0
    # and the triple delivers on the wall its third loop watches, in both
    assert bill["delivered"]["phi"] > bill["delivered"]["inc"]
    assert bill["delivered_inc"]["inc"] > bill["delivered_inc"]["phi"]
    assert bill["delivered_inc"]["inc"] > 99.0


@pytest.mark.slow
def test_the_incidence_stator_alone_is_worse_than_no_limiter_at_all_in_phi(bill):
    """The sharpest single number in the ledger: closing the stators LOWERS `phi`, so a loop
    that protects incidence drives the flow coefficient BELOW the bare march's own minimum."""
    inc = bill["inc"]["cells"]
    assert inc["S"]["min_phi"] < inc["bare"]["min_phi"]
    assert inc["S"]["credit"] < -100.0
    assert inc["S"]["v_max_used"] > 0.0 and inc["S"]["v_min"] < 1e-9    # the band is MIRRORED
    assert inc["S"]["v_saturated"], "and it is authority-limited — anchor s 0.2"


@pytest.mark.slow
def test_the_sign_table_is_grid_converged(split):
    """RUNG 65 PUBLISHED A RETRACTION for an RK4 artifact that read as a physical finding, and
    rung 68 published a `ds` table because of it. That table is NOT inherited here: this
    plant's dominant root is a lightly-damped COMPLEX pair, a different aliasing character from
    rung 68's real one. So the cells that carry § 4's sign table are re-run at half the step —
    including the smallest number in it, the incidence loop's own `phi` marginal, whose SIGN is
    the delicate one."""
    m, _ = split
    T_c, phi_lim = LP.tan_beta1_crit(), PHI

    def cell(ds, fuel, valve, stator):
        rig, surge, lag = m._triple_rig(SM, TAU, TAU_S, V_MAX, TAU_ATT, TAU_REL,
                                        fuel=fuel, valve=valve, stator=stator)
        t = _march(rig, ds=ds, surge=surge, lag=lag)
        return rig._violation(t, phi_lim, R), rig._violation_inc(t, M_LIM, T_c, R)

    out = {}
    for ds in (DS, DS / 2):
        out[ds] = {n: cell(ds, *a) for n, a in (("bare", (False, False, False)),
                                                ("FV", (True, True, False)),
                                                ("S", (False, False, True)),
                                                ("FVS", (True, True, True)))}
    for n in ("bare", "FV", "S", "FVS"):
        for i in (0, 1):
            assert out[DS / 2][n][i] == pytest.approx(out[DS][n][i], rel=5e-3), (n, i)

    def credits(o):
        cr = lambda c, i: 100.0 * (1.0 - o[c][i] / o["bare"][i])
        return (cr("S", 0), cr("S", 1),
                cr("FVS", 0) - cr("FV", 0), cr("FVS", 1) - cr("FV", 1))

    a, b = credits(out[DS]), credits(out[DS / 2])
    for x, y in zip(a, b):
        assert x * y > 0.0, "a SIGN must not depend on the grid"
        assert y == pytest.approx(x, rel=5e-3), (x, y)
    assert a[0] < -100.0 and a[1] > 50.0 and a[2] < 0.0 and a[3] > 0.0


# =============================================================================
# GATE 6 — AUTHORITY. Rung 64's ceiling, and the SIGN the split gives it.
# =============================================================================

@pytest.mark.slow
def test_authority_is_inert_in_company_and_buys_only_the_watched_wall(design):
    """RUNG 64: *a limiter's LAW cannot buy PROTECTION, only its PRICE — the ceiling is the
    lever's AUTHORITY*. Rung 68 EXTENDED that (inert in company, binding alone). Rung 69 gives
    it a SIGN: alone, more authority monotonically improves the wall the loop WATCHES and
    monotonically degrades the other — under BOTH references, in mirror image."""
    def run(v_max, ref, company):
        kw = ({"stator_inc": _inc(v_max=v_max)} if ref == "inc"
              else {"stator_lim": _phi_stator(v_max=v_max)})
        m = _split(design, bleed_lim=_valve(TAU) if company else None, **kw)
        t = _march(m, surge=_fuel() if company else None, lag=_lag() if company else None)
        return (m._violation(t, PHI, R), m._violation_inc(t, M_LIM, T_C, R),
                any(p["v_regime"] == "saturated" for p in t))

    trip = {vm: run(vm, "inc", True) for vm in (0.05, 0.10, 0.20, 0.40)}
    assert all(t[0] == pytest.approx(trip[0.20][0], rel=1e-11) for t in trip.values())
    assert not any(t[2] for t in trip.values())          # never reaches the stop in company

    VS = (0.05, 0.10, 0.20, 0.40)
    alone = {ref: {vm: run(vm, ref, False) for vm in VS} for ref in ("inc", "phi")}
    a_i, a_p = alone["inc"], alone["phi"]

    def monotone(seq, down):
        """Monotone to the SOLVER's resolution, not to the last bit. The phi arm PLATEAUS —
        it stops saturating at `v_max = 0.20`, so 0.20 and 0.40 agree to 1e-14 and their
        float order is noise. That plateau is itself the finding (rung 64's ceiling, located)
        and asserting a strict order on it would be asserting the noise."""
        return all((b - a) * (-1 if down else 1) >= -1e-9 * max(abs(a), abs(b))
                   for a, b in zip(seq, seq[1:]))

    #             watched wall: MONOTONE BETTER              other wall: MONOTONE WORSE
    assert monotone([a_i[v][1] for v in VS], down=True), [a_i[v][1] for v in VS]
    assert monotone([a_i[v][0] for v in VS], down=False), [a_i[v][0] for v in VS]
    assert monotone([a_p[v][0] for v in VS], down=True), [a_p[v][0] for v in VS]
    assert monotone([a_p[v][1] for v in VS], down=False), [a_p[v][1] for v in VS]
    assert a_i[0.05][1] / a_i[0.40][1] > 5.0        # and the effect is DECISIVE, both ways
    assert a_p[0.05][0] / a_p[0.40][0] > 5.0
    # WHERE EACH LEVER RUNS OUT: rung 68's is done at 0.20, this one is still starved there
    assert a_p[0.20][0] == pytest.approx(a_p[0.40][0], rel=1e-12)
    assert a_i[0.20][1] > 2.0 * a_i[0.40][1]


# =============================================================================
# GATE 7 — THE DISPLACEMENT. A degenerate plant cannot even be displaced.
# =============================================================================

@pytest.mark.slow
def test_a_shared_constraint_absorbs_a_displaced_start_and_a_split_one_cannot(split):
    """Rung 68's `s = 0` fixed points are a FAMILY, so displacing the stator's initial position
    just selects another member: the other two loops take it up EXACTLY and no tracking error
    survives. Under the split they cannot, and a fifth of the displacement survives as an error
    that then swings back. That is the rank difference showing up in the TRAJECTORY, not in a
    Jacobian — and it is why the ring is not separably observable without one."""
    m, _ = split
    r = m.ring_visibility(FLIGHT, LO, HI, sm=SM, disp=0.05)
    assert r["phi"]["displaced"]["survives"] < 1e-10, r["phi"]["displaced"]
    assert r["inc"]["displaced"]["survives"] > 0.1, r["inc"]["displaced"]
    assert r["inc"]["displaced"]["crossings"] >= 1
    # HONEST LIMIT: the ramp's own forcing reverses the error in the UNDISPLACED run too, so
    # a crossing count cannot separate the mode from the forcing. Reported, not claimed.
    assert r["inc"]["base"]["crossings"] >= 1


# =============================================================================
# GATE 8 — THE RK4 FLOOR: the CONSTANT survives, its REASON does not.
# =============================================================================

def test_the_floor_still_fires_and_its_message_names_the_new_reason(design):
    m = _split(design, bleed_lim=_valve(TAU), stator_inc=_inc())
    ds = 0.04                       # rung 66's two-clock constant ADMITS this
    assert ds * (1.0 / TAU + 1.0 / min(TAU_ATT, TAU_REL)) <= 2.0
    with pytest.raises(AssertionError, match="rank TWO"):
        _march(m, ds=ds, surge=_fuel(), lag=_lag())


@pytest.mark.slow
def test_the_inherited_constant_is_conservative_and_that_is_MEASURED(split):
    """Rung 68's constant survives on a DIFFERENT argument: the dominant root is no longer
    `-sum 1/tau` but a complex pair of modulus `sqrt(A z (1-k))`, bounded by `sqrt(1-k)/2`
    times that sum. An assert nobody has checked against the plant is a tautology (rung 67 gate
    9), and rung 65 published a retraction for exactly a trusted stability argument."""
    m, _ = split
    g = m.rk4_margin(FLIGHT, LO, HI, sm=SM)
    assert g["n"] >= 5
    assert g["max_mod"] < g["rate_sum"], (g["max_mod"], g["rate_sum"])
    assert g["max_ratio"] < g["max_bound"] + 1e-9, (g["max_ratio"], g["max_bound"])
    assert g["max_bound"] < 1.0, "the bound only holds while k >= -3"
    assert g["max_ratio"] > 0.7, "...and it is not a slack bound either"


# =============================================================================
# GATE 9 — THE INITIAL CONDITION. Removing a zero eigenvalue makes the plant
#          MORE sensitive to a moved start, not less: redundancy ABSORBS.
# =============================================================================

@pytest.mark.slow
def test_a_smaller_null_space_is_a_LARGER_ic_sensitivity(split):
    """PRE-REGISTERED THE OTHER WAY AND MISSED (anchor P9), and the miss is the content.

    Rung 68 measured its `s = 0` start-spread at 45.2 % (`I`) / 105.5 % (withheld fuel) and
    DECLINED to attribute the growth over rung 66's 84 % to its second zero eigenvalue. This
    rung supplies the counter-example: dropping the nullity from 2 to 1 makes both spreads
    GROW again. So the zero count and the IC sensitivity move in OPPOSITE directions, and a
    null space is a SHOCK ABSORBER — the redundant loops redistribute a moved start among
    themselves. GATE 7 is the same mechanism read at a single point."""
    m, _ = split
    f = m.ic_family(FLIGHT, LO, HI, sm=SM)
    assert f["order_members"] == 1, "the order is still NOT the lever from the declared start"
    assert all(x["iters"] == 1 for x in f["by_order"].values())
    assert f["start_spread_I"] > 0.452, f["start_spread_I"]          # rung 68's own number
    assert f["start_spread_withheld"] > 1.055, f["start_spread_withheld"]
    assert f["start_spread_I"] > 1.5 and f["start_spread_withheld"] > 2.5
    # the DECLARED member is still rung 66's, and the stator still opens at its dormant stop
    z = f["by_order"]["gqv"]
    assert z["g0"] == 0.0 and z["v0"] == 0.0
    assert z["b0"] == pytest.approx(0.036626, abs=1e-5)


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
