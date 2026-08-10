"""Rung 72 — TWO LOOPS ON ONE ACTUATOR: rung 52's `phi` FUEL leg armed BESIDE rung 47's `Tt4`
governor, so two limiters drive the SAME actuator. Six states, four clocks, four loops, THREE
actuators — `n = 4`, the last unoccupied SHAPE after rungs 68–71 filled every `(3, m)` cell, and
the seam rungs 70 § 6.1 / 71 § 11 both named.

THE HEADLINE: **A SHARED ACTUATOR ADDS A SWITCH BETWEEN PLANTS, NOT A LOOP.** Min-select makes
authority EXCLUSIVE, so the masked leg reaches the plant through a `max()` that is FLAT in it:
its column is `(−1, 0, 0, 0)`, the block is triangular, and this ONE plant IS rung 68, 69, 70 or
71 at every instant — polynomial for polynomial — plus a free pole at the masked leg's own clock.

    | stator watches | fuel leg holds | governor holds |
    | `phi`          | RUNG 68 (zeros 2) | RUNG 70 (zeros 1) |
    | `M_i`          | RUNG 69 (zeros 1) | RUNG 71 (zeros 0) |

So `zeros = n_live − m_live`, counting the loops that hold AUTHORITY — and the RANK CHANGES at
the hand-over with no state, no gain and no clock moving. Rung 71 § 11 asked whether `m` counts
constraints or actuators; the answer is NEITHER, and the incidence arm is what proves it (the
constraint reading is wrong by one on BOTH arms, the actuator reading right on the `phi` arm
only, by coincidence).

AND THE `(4, m)` CELLS ARE A MIRAGE under a shared actuator: `n_live` is 3 at every instant, so
min-select collapses them to `(3, m)` plus a pole. Occupying `n = 4` genuinely needs the fourth
LP lever rung 69 § 11 named and this plant does not have.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    SharedActuatorTransient, FullSplitTransient, CrossSplitTransient,
    ReferenceSplitTransient, ThreeLoopCascadeTransient, CrossLoopCascadeTransient,
    BleedLimiter, StatorLimiter, StatorIncidenceLimiter, SurgeLimiter, AsymmetricLag,
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
TT4_MAX = 1200.0                 # RUNG 67's imposed redline, verbatim through rungs 70/71

# the two clock arms: MATCHED (the base reading) and WIDE-CELL. The second exists because the
# incidence arm's GOVERNOR-authority cell — rung 71's own — holds 1 point of 35 at matched
# clocks: a fast governor and a slow fuel leg hand over EARLY, a slow valve keeps the stator
# riding LATE. All four are swept march coordinates (spec s 6).
CLOCKS = ((0.05, 0.05, 0.05, 0.05), (0.20, 0.01, 0.50, 0.05))

# s 1.3's law, per cell: zeros = n_live - m_live, n_live = 3 always
PREDICTED = {(False, "fuel"): 2, (False, "gov"): 1, (True, "fuel"): 1, (True, "gov"): 0}
PARENT = {(False, "fuel"): "rung 68", (False, "gov"): "rung 70",
          (True, "fuel"): "rung 69", (True, "gov"): "rung 71"}

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _shared(design, **kw):
    return SharedActuatorTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _valve(tau=TAU):
    return BleedLimiter.from_margin(LP, B, SM, tau=tau)


def _phi_stator(tau=TAU_S, v_max=V_MAX):
    return StatorLimiter.from_margin(LP, v_max, SM, tau=tau)


def _inc_stator(tau=TAU_S, v_max=V_MAX):
    return StatorIncidenceLimiter.from_margin(LP, v_max, SM, tau=tau)


def _surge():
    return SurgeLimiter.from_margin(LP, "lp", SM)


def _lag(att=TAU_ATT, rel=TAU_REL):
    return AsymmetricLag(tau_att=att, tau_rel=rel)


def _march(m, **kw):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, **kw)[0]


def _keys(traj, ks=("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b", "v")):
    return [tuple(p[k] for k in ks) for p in traj]


# ======================================================================================
# THE REDUCE SPINE — five arms, all by DISPATCH, all bit-for-bit (anchor P8)
#
# NOT MARKED `slow`, DELIBERATELY. Each of these runs two 341-point marches, so they are not
# free, and `conftest.py` is explicit that `-m "not slow"` has no backstop against an unmarked
# expensive test. They stay in the fast loop anyway because the reduce spine is the project's
# spine — and because rungs 69/70/71 leave their own (2, 4 and 4 tests) unmarked for exactly
# that reason. Every FINDING sweep below IS marked. See docs/rung72-spec.md s 7.
# ======================================================================================

def test_reduces_to_rung71_no_fuel_leg(design):
    """No fuel leg + an incidence stator + the governor IS rung 71, entry for entry. The
    dispatch never enters this rung's march for a plant it does not own."""
    a = _march(_shared(design, bleed_lim=_valve(), stator_inc=_inc_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    b = _march(FullSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                  bleed_lim=_valve(), stator_inc=_inc_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert _keys(a) == _keys(b)


def test_reduces_to_rung70_no_fuel_leg(design):
    a = _march(_shared(design, bleed_lim=_valve(), stator_lim=_phi_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    b = _march(CrossSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                   bleed_lim=_valve(), stator_lim=_phi_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert _keys(a) == _keys(b)


def test_reduces_to_rung69_no_governor(design):
    a = _march(_shared(design, bleed_lim=_valve(), stator_inc=_inc_stator()),
               surge=_surge(), lag=_lag())
    b = _march(ReferenceSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                       bleed_lim=_valve(), stator_inc=_inc_stator()),
               surge=_surge(), lag=_lag())
    assert _keys(a) == _keys(b)


def test_reduces_to_rung68_no_governor(design):
    a = _march(_shared(design, bleed_lim=_valve(), stator_lim=_phi_stator()),
               surge=_surge(), lag=_lag())
    b = _march(ThreeLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                         bleed_lim=_valve(), stator_lim=_phi_stator()),
               surge=_surge(), lag=_lag())
    assert _keys(a) == _keys(b)


def test_reduces_to_rung67_no_stator_no_fuel_leg(design):
    """A governor and a valve with NO stator and NO fuel leg is rung 67 — and it stays rung 67
    even though this rung's dispatch no longer asks for a stator, because it asks for BOTH fuel
    legs instead."""
    ks = ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b")
    a = _march(_shared(design, bleed_lim=_valve()), Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    b = _march(CrossLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                         bleed_lim=_valve()), Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert _keys(a, ks) == _keys(b, ks)


def test_at_lever_returns_this_class(design):
    """THE TENTH INSTANCE of the trap rungs 61–71 each hit: hand back the parent's class and
    every reader measures rung 71's plant while reporting rung 72's."""
    m = _shared(design, bleed_lim=_valve()).at_lever(bleed_lim=_valve(),
                                                     stator_inc=_inc_stator())
    assert type(m) is SharedActuatorTransient
    assert m.stator_inc is not None and m.bleed_lim is not None


# ======================================================================================
# THE INSTRUMENT — gated against itself, because a broken one looked plausible
# ======================================================================================

def test_charpoly_selftest():
    """`_charpoly4`'s first version had `A` where Faddeev–LeVerrier needs `M_{k-1}` and returned
    a WRONG polynomial with entirely plausible downstream numbers: a stable-looking spectrum, a
    determinant of 5.9e+05 and a root residual of 1e-09, because the root finder was faithfully
    solving the wrong polynomial. Nothing downstream could tell, so the polynomial is checked
    against an INDEPENDENT trace and cofactor determinant and against a triangular matrix whose
    spectrum is its own diagonal."""
    out = SharedActuatorTransient.charpoly_selftest()
    for name, d in out.items():
        assert d["trace_err"] < 1e-9, (name, d)
        assert d["det_err"] < 1e-9, (name, d)
        assert d["det_vs_a0"] < 1e-9, (name, d)
        assert d["resid"] < 1e-9, (name, d)
    # the triangular arm: the spectrum IS the diagonal, and it is REAL
    assert out["triangular"]["diag_err"] < 1e-9
    assert out["triangular"]["max_imag"] < 1e-9


def test_charpoly_selftest_catches_the_broken_recursion():
    """MEASURE THE DETECTOR'S SENSITIVITY, do not assert it. The self-test is only worth having
    if it FAILS on the bug it was written for, so the bug is rebuilt and fed to it."""
    def broken(A):
        n, c, M = 4, [1.0], None
        for k in range(1, n + 1):
            if k == 1:
                M = [row[:] for row in A]
            else:                      # the bug: `A` where the recursion needs `M`
                T = [[A[i][j] + (c[-1] if i == j else 0.0) for j in range(n)]
                     for i in range(n)]
                M = [[sum(A[i][t] * T[t][j] for t in range(n)) for j in range(n)]
                     for i in range(n)]
            c.append(-sum(M[i][i] for i in range(n)) / k)
        return c

    tri = [[-20.0, 7.0, -3.0, 9.0], [0.0, -25.0, 4.0, -6.0],
           [0.0, 0.0, -30.0, 8.0], [0.0, 0.0, 0.0, -50.0]]
    got = sorted(z.real for z in SharedActuatorTransient._quartic_roots_c(broken(tri)))
    diag = sorted(tri[i][i] for i in range(4))
    assert max(abs(a - b) for a, b in zip(diag, got)) > 1.0, (
        "the broken recursion must be CAUGHT by the triangular arm, or the self-test is "
        "ceremony")


# ======================================================================================
# s 0 — WHO HOLDS THE ACTUATOR (anchor § 0)
# ======================================================================================

@pytest.mark.slow
def test_authority_changes_hands_once_inside_the_joint_window(design):
    """The plant splits into a fuel-leg interval and a governor interval, with ONE hand-over,
    and it sits INSIDE the joint window on every arm — which is what lets s 2 measure a rank
    change on BOTH sides of it on ONE trajectory, with no second plant."""
    al = _shared(design, bleed_lim=_valve()).authority_law(
        FLIGHT, LO, HI, TT4_MAX, SM, clocks=CLOCKS)
    assert al["one_handover"], [a["handovers"] for a in al["arms"]]
    for a in al["arms"]:
        assert a["joint"][2] > 0, a["taus"]
        assert a["handover_inside"], (a["inc"], a["taus"], a["handovers"], a["joint"])
        # the masked leg is RIDING and reaching nothing, not dormant
        assert a["both_want"] > 0.5 * a["n"], a["both_want"]
    # the WIDE-CELL arm reaches BOTH authority cells inside the joint window, on both arms
    assert al["both_cells_everywhere"]


# ======================================================================================
# s 1 — THE FOUR EXACT ZEROS (anchor P4, P7)
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("inc", [False, True])
def test_the_two_legs_cannot_see_each_other(design, inc):
    """`F_r = R_f = 0` EXACTLY, so `pair_FR = 0` exactly — both legs solve from the SCHEDULED
    fuel (rungs 47/52's own discipline, inherited verbatim). Rung 66's two loops on one VARIABLE
    gave a pair product of exactly 1; two loops on one ACTUATOR give exactly 0. The two corners
    of one question."""
    g = _shared(design, bleed_lim=_valve()).shared_gains(
        FLIGHT, LO, HI, TT4_MAX, SM, inc=inc)
    assert g["worst_F_r"] == 0.0
    assert g["worst_R_f"] == 0.0
    assert g["worst_pair_FR"] == 0.0


@pytest.mark.slow
@pytest.mark.parametrize("inc", [False, True])
def test_the_masked_leg_reaches_the_plant_through_nothing(design, inc):
    """`C_masked = V_masked = 0` EXACTLY — not small. `max()` is FLAT in the masked clip, so the
    coupling is absent rather than weak. This is the GATED quantity; the free pole at
    `-1/tau_masked` follows from it ALGEBRAICALLY (the diagonal is `-1/tau_i` by construction)
    and is reported, never gated — rung 67 gate 9's retraction in a third shape.

    And the LIVE gains are checked non-zero on the same points, so 'exactly zero everywhere' is
    not being bought with a decoupled instrument."""
    g = _shared(design, bleed_lim=_valve()).shared_gains(
        FLIGHT, LO, HI, TT4_MAX, SM, inc=inc)
    assert g["worst_mask_leak"] == 0.0
    assert g["min_live_gain"] > 1e-6, g["min_live_gain"]
    assert g["by_authority"]["fuel"] > 0 and g["by_authority"]["gov"] > 0


# ======================================================================================
# s 2 — THE FOUR CELLS: ONE PLANT, FOUR PARENT RUNGS (anchor P1, P2, P3)
# ======================================================================================

@pytest.mark.slow
def test_the_four_cells_are_rungs_68_69_70_and_71(design):
    """THE RUNG. Every cell's zero count is `n_live - m_live`, and every cell's characteristic
    polynomial is the PARENT rung's own times `(lam + 1/tau_masked)` — rebuilt from the SHIPPED
    three-loop readers, so two independent instruments reach one polynomial.

    The comparison is on COEFFICIENTS, not roots: in the rung-68 cell the parent has a DOUBLE
    zero root and this rung a TRIPLE one, and a repeated root resolves only to the square root
    of the working precision (measured: individual roots at 3e-07 while every invariant sits at
    1e-13)."""
    c = _shared(design, bleed_lim=_valve()).shared_cells(
        FLIGHT, LO, HI, TT4_MAX, SM, clocks=CLOCKS)
    assert c["all_four_cells"], sorted(c["cells"])
    assert c["law_holds"], {k: d["zeros"] for k, d in c["cells"].items()}
    for key, d in c["cells"].items():
        assert d["zeros"] == [PREDICTED[key]], (key, PARENT[key], d["zeros"])
        assert d["n"] >= 5, (key, d["n"])
        assert d["gap"] < 1e-10, (key, PARENT[key], d["gap"])
    # the two readers land on the SAME manifold base point, so the match is not a coincidence
    # of two different points (the alternative hypothesis for the rung-68 cell's precision)
    assert c["worst_v_gap"] == 0.0


@pytest.mark.slow
def test_the_rank_changes_at_the_hand_over_with_nothing_moving(design):
    """THE DISCONTINUITY. On ONE trajectory the zero count is 2 before the hand-over and 1 after
    (`phi` arm), 1 and 0 (incidence arm) — with no state, no gain and no clock changing. No
    earlier rung in this family could exhibit it, because none had a quantity that could change
    without something moving."""
    c = _shared(design, bleed_lim=_valve()).shared_cells(
        FLIGHT, LO, HI, TT4_MAX, SM, clocks=CLOCKS)
    for inc in (False, True):
        lo = c["cells"][(inc, "gov")]["zeros"]
        hi = c["cells"][(inc, "fuel")]["zeros"]
        assert hi == [lo[0] + 1], (inc, hi, lo)


@pytest.mark.slow
def test_only_the_rung71_cell_has_a_live_determinant(design):
    """`det J` is non-zero in exactly ONE of the four cells — the incidence arm under governor
    authority, which is rung 71's own plant and the only full-rank one in the family. That is
    rung 71 s 1.3's factorisation surviving a rung that adds no factor: the masked leg
    multiplies it by `-1/tau_masked` and nothing else."""
    c = _shared(design, bleed_lim=_valve()).shared_cells(
        FLIGHT, LO, HI, TT4_MAX, SM, clocks=CLOCKS)
    live = [k for k, d in c["cells"].items() if d["zeros"] == [0]]
    assert live == [(True, "gov")], live


# ======================================================================================
# s 3 — THE ISOLATION INSTRUMENT, AND ITS OWN CONFOUND (anchor P5)
# ======================================================================================

@pytest.mark.slow
def test_the_free_pole_separates_the_laws_ONLY_at_unmatched_clocks(design):
    """THE CONFOUND IS GATED BESIDE THE RESULT. At `tau_f = tau_g` the SUM law has `(1,-1,0,0)`
    as an exact eigenvector with eigenvalue `-1/tau`, so the free-pole test passes under BOTH
    laws and separates nothing. Unmatch the two fuel clocks and min-select keeps the pole to
    1e-14 while SUM loses it by ten orders of magnitude.

    Gating the confound is the point: a discriminator quoted from the matched arm alone is a
    discriminator that never tested anything."""
    md = _shared(design, bleed_lim=_valve()).mask_discriminator(FLIGHT, LO, HI, TT4_MAX, SM)
    assert md["max_pole_unmatched"] < 1e-12, md["max_pole_unmatched"]
    assert md["sum_pole_unmatched"] > 1e-3, md["sum_pole_unmatched"]
    # THE CONFOUND: at matched clocks the SUM law passes the same test
    assert md["sum_pole_matched"] < 1e-12, md["sum_pole_matched"]


@pytest.mark.slow
def test_the_sum_law_gives_the_masked_leg_its_rank_back(design):
    """Restoring both legs' authority WITHOUT changing the loop count moves the zero count by
    exactly one — at FUEL-authority points, where min-select was masking a leg, and not at
    governor-authority ones on this arm. So the count is not blind to masking after all (the
    anchor's D5 said it would be; it is scored as refuted in the spec)."""
    md = _shared(design, bleed_lim=_valve()).mask_discriminator(FLIGHT, LO, HI, TT4_MAX, SM)
    for taus, z in md["zeros_max"].items():
        assert z["fuel"] == [2], (taus, z)
    for taus, z in md["zeros_sum"].items():
        assert z["fuel"] == [1], (taus, z)


# ======================================================================================
# s 4 — THE LEDGER (anchor P6)
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("inc", [False, True])
def test_the_masked_leg_still_buys_something(design, inc):
    """A spectral reading says a masked leg is coupled to nothing; the ledger says otherwise,
    because authority is a function of `s` and a leg masked LATE held the actuator EARLY. The
    fuel leg's marginal `phi` credit is POSITIVE — and tiny, ~0.1-1.2 % of its solo credit.

    ITS SOLO CELL IS QUOTED BESIDE THE RATIO AND THAT IS NOT A FORMALITY: rung 52's leg ALONE
    holds `max Tt4` at the initial 1000 K (it starves the accel outright, `E = 0`), so the
    `kept` denominator is taken on a trajectory no other cell shares. Rung 71 s 4's own lesson,
    with the confound larger here than there."""
    b = _shared(design, bleed_lim=_valve()).shared_bill(
        FLIGHT, LO, HI, TT4_MAX, SM, inc=inc)
    assert b["fuel_marginal_phi"] > 0.0
    assert 0.0 < b["kept"]["F"] < 0.10, b["kept"]["F"]
    assert b["phi_full"] > b["phi_no_fuel"]           # it does buy phi
    # AND IT DOES SPEND THE GOVERNOR'S CURRENCY, in OPPOSITE directions on the two readings:
    # the exceedance INTEGRAL improves, the PEAK gets worse. The anchor's P6 predicted the
    # peak unmoved; it is refuted, and the sign is the claim (magnitudes disclaimed).
    assert b["fuel_marginal_Tt4"] > 0.0               # integral: a credit
    assert b["Tt4_full"] > b["Tt4_no_fuel"]           # peak: a debit
    # the degenerate solo cell, recorded so the ratio above cannot be read without it
    assert b["cells"]["F"]["E"] == 0.0
    assert b["cells"]["F"]["max_Tt4"] == pytest.approx(LO, abs=1e-6)


# ======================================================================================
# THE REFUSALS (anchor P10)
# ======================================================================================

def test_refuses_tau_gov_without_a_set_point(design):
    m = _shared(design, bleed_lim=_valve(), stator_inc=_inc_stator())
    with pytest.raises(AssertionError, match="governor with no set point"):
        _march(m, tau_gov=TAU_GOV, surge=_surge(), lag=_lag())


def test_refuses_a_forced_release_edge(design):
    """Refused TWICE OVER, and the outer refusal is STRUCTURAL — rung 71 s 8.2's own reading,
    inherited: `_stator_march`, the entry every reader on this ladder actually calls, does not
    plumb `s_off`/`tau_rel` at all, so they cannot reach a march even by mistake. The assert in
    `integrate_fuel` is the inner guard for a caller that goes around it, and it is reached
    directly here because there is no other way to reach it."""
    import inspect
    sig = inspect.signature(SharedActuatorTransient._stator_march).parameters
    assert "s_off" not in sig and "tau_rel" not in sig, sorted(sig)
    m = _shared(design, bleed_lim=_valve(), stator_inc=_inc_stator())
    with pytest.raises(AssertionError, match="FORCED release edges"):
        m.integrate_fuel(FLIGHT, lambda s: 1.0, (1.0, 1.0), 0.1, DS,
                         Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_surge(), lag=_lag(),
                         s_off=0.3)


def test_refuses_an_instantaneous_valve(design):
    m = _shared(design, bleed_lim=BleedLimiter.from_margin(LP, B, SM),
                stator_inc=_inc_stator())
    with pytest.raises(AssertionError, match="INSTANTANEOUS valve"):
        _march(m, Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_surge(), lag=_lag())


def test_refuses_an_undeclared_composition_law(design):
    """The composition law on the shared actuator is this rung's ONE modelling decision, so it
    is declared and never inferred."""
    m = _shared(design, bleed_lim=_valve(), stator_inc=_inc_stator())
    m._share_law = "mean"
    with pytest.raises(AssertionError, match="composition law"):
        _march(m, Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_surge(), lag=_lag())


def test_the_rk4_floor_is_on_all_four_clocks(design):
    """The floor is re-justified a FIFTH time on a FOURTH argument: the masked leg's eigenvalue
    is exactly `-1/tau_f` and the other three share the remainder, so no root exceeds the rate
    sum. Re-stated rather than inherited, because rung 65 published a retraction for a trusted
    stability argument."""
    m = _shared(design, bleed_lim=_valve(), stator_inc=_inc_stator())
    with pytest.raises(AssertionError, match=r"FOUR actuator states"):
        m._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.05, Tt4_max=TT4_MAX,
                        tau_gov=TAU_GOV, surge=_surge(), lag=_lag())
    # and it ADMITS the grid every reader here runs on
    SharedActuatorTransient._rk4_floor_shared(0.002, 4.0 / 0.05)


def test_the_composition_law_lives_in_one_place(design):
    """`_applied_clip` is the plant's law AND every reader's, so a reader cannot compose the two
    clips differently from the march that produced its base point."""
    m = _shared(design, bleed_lim=_valve())
    assert m._applied_clip(0.3, 0.7) == 0.7
    assert m._with_share("sum", m._applied_clip, 0.3, 0.7) == pytest.approx(1.0)
    assert m._share_law == "max"          # restored in a `finally` (rung 62's reason)


def test_authority_labels_the_switch_itself(design):
    """A third regime label no prior rung needed: `dormant`, `tie` (the kink, where a central
    difference straddles two branches) and the holder's name."""
    a = SharedActuatorTransient._authority
    assert a(0.0, 0.0) == "dormant"
    assert a(1e-3, 1e-3) == "tie"
    assert a(2e-3, 1e-3) == "fuel"
    assert a(1e-3, 2e-3) == "gov"


# --- THE MARCH AUDIT: rung 79's gap seam, checked from the other end ------------------------
# `docs/rungs72-77-march-audit.md`. Added by a CONFIRMATION, not by this rung's anchor, and
# honest about that: nothing here was pre-registered.

@pytest.mark.slow
def test_this_rungs_march_MOVES_and_all_four_loops_are_live(design):
    """`docs/rung79-gap-margin.md` proved rungs 78/79's marches never leave their initial state
    and flagged that THIS RUNG SHARES THE RIG. It does not stand still.

    **AND THE GATE IS A COUNTER-EXAMPLE, NOT A LIVENESS ASSERTION.** Same rig, same wall
    (`phi_lim = 0.80`), same 341 steps as the arrested rows -- the ONLY difference is the
    coordinate, which is `clip` here and `demand` there (rung 74 s 2.2's arrest arm). That is
    what localises the arrest to the CELL rather than to the rig.

    The four loop counts are the SECOND vacuity mode (audit s 1): a plant that moves while the
    section's own loop does nothing is just as vacuous as a frozen one."""
    m = _shared(design, bleed_lim=_valve())
    traj = m._shared_march(FLIGHT, LO, HI, TT4_MAX, SM, CLOCKS[0], R, SETTLE, DS,
                           V_MAX, False)[3]
    assert len(traj) > 300, len(traj)
    nu = [p["nu_lp"] for p in traj]
    assert (max(nu) - min(nu)) / min(nu) > 1e-2, (min(nu), max(nu))
    t4 = [p["Tt4"] for p in traj]
    assert max(t4) - min(t4) > 200.0, (min(t4), max(t4))
    # ALL FOUR LOOPS ACT -- governor, valve interior, stator riding, and rung 49's phi leg,
    # the last by its observable signature: the droop is held far above the free one (0.7430
    # at the 0.70 arm, audit s 3) while still crossing the wall the clip coordinate tracks.
    b_max = m.bleed_lim.b_max
    assert sum(1 for p in traj if p["required"] > 0.0) > 300
    assert sum(1 for p in traj if 0.0 < p["b_cmd"] < b_max) > 50
    assert sum(1 for p in traj if p.get("v_regime") == "riding") > 50
    assert min(p["phi_lp"] for p in traj) > 0.78, min(p["phi_lp"] for p in traj)
