"""Rung 73 — THE APPLIED REFERENCE: rung 72 § 11's own sharpest seam, and the direct test of
the bound its § 6 put on its headline.

Rung 72's two fuel-side legs both compute their clip from the SCHEDULED fuel, which is what
makes `F_r = R_f = 0` exactly and the block triangular. Its § 11: *a leg that reads the applied
fuel gives `F_r != 0`, couples the two fuel rows, and destroys the block form — the spectrum
would then be genuinely four-dimensional.*

THE HEADLINE: **THE COUPLING IS REAL, AND IT LANDS IN THE WRONG COLUMN.** `F_r = −1` exactly, so
the premise HOLDS — but triangularity was never a property of the masked leg's ROW; it is a
property of its COLUMN, and `F_r` sits in the AUTHORITATIVE one. The masked column is zero under
EVERY reference, because `max()` is flat in the masked state. **Triangularity is a property of
MIN-SELECT alone.**

What the reference buys is the POLE: rung 72's free pole at `−1/tau_masked` moves to EXACTLY the
ORIGIN — a masked leg referenced to the applied fuel is a pure INTEGRATOR running open loop —
so every one of rung 72's four per-cell zero counts gains one, and `det J` dies in rung 71's
cell, the only full-rank plant in the family.

    | stator watches | fuel leg holds       | governor holds       |
    | `phi`          | rung 68 + a zero (3) | rung 70 + a zero (2) |
    | `M_i`          | rung 69 + a zero (2) | rung 71 + a zero (1) |
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    AppliedReferenceTransient, SharedActuatorTransient, FullSplitTransient,
    CrossSplitTransient, ReferenceSplitTransient, ThreeLoopCascadeTransient,
    CrossLoopCascadeTransient, BleedLimiter, StatorLimiter, StatorIncidenceLimiter,
    SurgeLimiter, AsymmetricLag,
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
TT4_MAX = 1200.0                 # RUNG 67's imposed redline, verbatim through rungs 70/71/72

# THE THREE CLOCK ARMS. Rung 72's two, plus a DEEP-CELL arm — the applied reference delays the
# hand-over, so rung 72's coverage does not transfer: at matched clocks the incidence/governor
# cell is EMPTY (0 points, against rung 72's 1), and rung 72's wide-cell arm reaches it with 4.
# The new arm is rung 72 § 2.3's own device pushed one notch (governor twice as fast, valve 1.6x
# slower). All four entries are swept march coordinates; no physical constant enters.
CLOCKS = ((0.05, 0.05, 0.05, 0.05), (0.20, 0.01, 0.50, 0.05), (0.20, 0.005, 0.80, 0.05))

# § 1.2's law: zeros = n_live − m_live + n_masked, i.e. rung 72's own counts EACH PLUS ONE
PREDICTED = {(False, "fuel"): 3, (False, "gov"): 2, (True, "fuel"): 2, (True, "gov"): 1}
RUNG72 = {(False, "fuel"): 2, (False, "gov"): 1, (True, "fuel"): 1, (True, "gov"): 0}

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _applied(design, **kw):
    return AppliedReferenceTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


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
# THE REDUCE SPINE — SIX arms. Five inherited by DISPATCH, one by the declared law.
#
# NOT MARKED `slow`, on rung 72's own reasoning (and rungs 69/70/71's): each runs two
# 341-point marches and is not free, but the reduce spine is the project's spine and
# `conftest.py` is explicit that `-m "not slow"` has no backstop. Every FINDING below IS
# marked. See docs/rung73-spec.md § 7.
# ======================================================================================

def test_reduces_to_rung72_under_the_scheduled_reference(design):
    """THE SIXTH ARM, and this rung's own: `_ref_law = 'sched'` makes `_reference` the identity,
    so the plant is rung 72 BIT-FOR-BIT. The hook is the only thing this rung adds to the
    march, so this is the arm that says so."""
    m = _applied(design, bleed_lim=_valve(), stator_inc=_inc_stator())
    a = m._with_ref("sched", _march, m, Tt4_max=TT4_MAX, tau_gov=TAU_GOV,
                    surge=_surge(), lag=_lag())
    b = _march(SharedActuatorTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                       bleed_lim=_valve(), stator_inc=_inc_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_surge(), lag=_lag())
    assert _keys(a) == _keys(b)


def test_the_scheduled_reduce_is_not_vacuous(design):
    """AND THE ARM ABOVE MUST BE A TEST, NOT A TAUTOLOGY. If `_reference` ignored `_ref_law` the
    reduce would still pass — it would compare rung 73 with rung 73 — so the same two marches
    under the APPLIED reference must DIFFER. Rung 72's `charpoly_selftest` discipline: a gate
    that has never failed on the bug it was written for is ceremony."""
    m = _applied(design, bleed_lim=_valve(), stator_inc=_inc_stator())
    a = _march(m, Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_surge(), lag=_lag())
    b = _march(SharedActuatorTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                       bleed_lim=_valve(), stator_inc=_inc_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_surge(), lag=_lag())
    assert _keys(a) != _keys(b)
    # and it differs in the PLANT, not only in the masked state
    assert max(abs(x["Tt4"] - y["Tt4"]) for x, y in zip(a, b)) > 1.0


def test_reduces_to_rung71_no_fuel_leg(design):
    a = _march(_applied(design, bleed_lim=_valve(), stator_inc=_inc_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    b = _march(FullSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                  bleed_lim=_valve(), stator_inc=_inc_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert _keys(a) == _keys(b)


def test_reduces_to_rung70_no_fuel_leg(design):
    a = _march(_applied(design, bleed_lim=_valve(), stator_lim=_phi_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    b = _march(CrossSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                   bleed_lim=_valve(), stator_lim=_phi_stator()),
               Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert _keys(a) == _keys(b)


def test_reduces_to_rung69_no_governor(design):
    """AND THIS ARM IS AN IDENTITY, NOT ONLY A DISPATCH. With ONE fuel-side leg armed the sole
    leg always holds authority, so `max(gf, gr) == g_own` everywhere and the applied reference
    IS the scheduled one — the reduce would hold even with the dispatch removed. Rung 71's
    *inherited identity* form, one rung on."""
    a = _march(_applied(design, bleed_lim=_valve(), stator_inc=_inc_stator()),
               surge=_surge(), lag=_lag())
    b = _march(ReferenceSplitTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                       bleed_lim=_valve(), stator_inc=_inc_stator()),
               surge=_surge(), lag=_lag())
    assert _keys(a) == _keys(b)


def test_reduces_to_rung68_no_governor(design):
    a = _march(_applied(design, bleed_lim=_valve(), stator_lim=_phi_stator()),
               surge=_surge(), lag=_lag())
    b = _march(ThreeLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                         bleed_lim=_valve(), stator_lim=_phi_stator()),
               surge=_surge(), lag=_lag())
    assert _keys(a) == _keys(b)


def test_reduces_to_rung67_no_stator_no_fuel_leg(design):
    ks = ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b")
    a = _march(_applied(design, bleed_lim=_valve()), Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    b = _march(CrossLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                         bleed_lim=_valve()), Tt4_max=TT4_MAX, tau_gov=TAU_GOV)
    assert _keys(a, ks) == _keys(b, ks)


def test_at_lever_and_the_rig_both_carry_the_reference(design):
    """THE ELEVENTH INSTANCE of the trap rungs 61–72 each hit, with a second head. Handing back
    the parent's class reports rung 73 while measuring rung 72; handing back the right class but
    dropping `_ref_law` does the same thing one level down, in every ledger cell."""
    m = _applied(design, bleed_lim=_valve())
    lv = m.at_lever(bleed_lim=_valve(), stator_inc=_inc_stator())
    assert type(lv) is AppliedReferenceTransient
    assert lv._ref_law == "applied"
    rig, _, _ = m._shared_rig(SM, TAU, TAU_S, V_MAX, TT4_MAX, inc=True)
    assert type(rig) is AppliedReferenceTransient and rig._ref_law == "applied"
    # and the restore-in-`finally` must reach the rig too (rung 62's reason, seventh reload)
    rig2, _, _ = m._with_ref("sched", m._shared_rig, SM, TAU, TAU_S, V_MAX, TT4_MAX, inc=True)
    assert rig2._ref_law == "sched" and m._ref_law == "applied"


# ======================================================================================
# THE INSTRUMENT, GATED AGAINST ITSELF — the bug here produced a PERFECT confirmation
# ======================================================================================

def test_the_reference_dispatch_is_live(design):
    """`_reference`'s first version applied reading B unconditionally, so `_with_ref('sched',.)`
    was a NO-OP and every A-vs-B reader differenced the plant against ITSELF. It did not fail:
    it returned `worst_delta_rest = 0.0` and `mask_leak = 0.0` — a perfect confirmation of this
    rung's headline from an instrument that had measured nothing. **That is the fifth instance
    of this family's shipped-instrument-agrees-with-itself pattern** (rung 67 gate 9, rung 71
    § 1.4, rung 72 § 4 and § 8's `_charpoly4`).

    So the bug is REBUILT and fed to the gate: the live gate is `moved_scaled == ±1`, and the
    broken version must fail it while still passing `worst_delta_rest == 0.0`."""
    class Broken(AppliedReferenceTransient):
        def _reference(self, req, g_own, gf, gr):     # the bug: no `_ref_law` dispatch
            clip = self._applied_clip(gf, gr)
            return req if clip == g_own else g_own + req - clip

        def at_lever(self, **kw):
            # `at_lever` names its class explicitly (the rung-61..72 trap's own fix), so the
            # probe has to re-bless the rig or it would test the SHIPPED class and pass.
            m = super().at_lever(**kw)
            m.__class__ = Broken
            return m

    m = Broken(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, bleed_lim=_valve())
    g = m.applied_gains(FLIGHT, LO, HI, TT4_MAX, SM, taus=CLOCKS[0], inc=False, ds=0.01,
                        every=8)
    assert g["rows"], "the broken-instrument probe needs at least one interior point"
    # the broken reader still passes the weak gate ...
    assert g["worst_delta_rest"] == 0.0
    # ... and FAILS the live one, which is why the live one is the gate
    assert all(abs(v) < 1e-9 for v in g["moved_scaled"]), g["moved_scaled"]


# ======================================================================================
# § 0 — THE HAND-OVER MOVES, AND THE MASKED LEG WINDS DOWN (anchor P9, § 0.2)
# ======================================================================================

@pytest.mark.slow
def test_the_handover_is_late_and_the_masked_leg_winds_down(design):
    """**The applied reference DELAYS the hand-over on every arm and at every clock**, and the
    sign is derivable: a masked governor referenced to the SCHEDULE races toward the clip the
    schedule would need — credit for a cut the fuel leg already made — while referenced to the
    APPLIED fuel it integrates the cut still OWED. The physically-correct governor is the
    SLOWER one.

    AND THE WINDUP CHECK, which was the feasibility gate: a masked integrator with only a floor
    under it is textbook min-select windup, and had it run away the hand-over would slam a
    wound-up clip onto the actuator (rung 72 § 4's SUM law died that way, at 84 points of 341).
    It winds DOWN instead — masked means `gr > gf ~ req_f`, so the integrand is negative."""
    h = _applied(design, bleed_lim=_valve()).handover_law(FLIGHT, LO, HI, TT4_MAX, SM,
                                                          clocks=CLOCKS)
    assert h["always_later"], [(a["inc"], a["taus"], a["delay"]) for a in h["arms"]]
    assert h["never_back"] and h["one_handover"] and h["full_march"]
    assert h["worst_dTt4"] > 0.0
    for a in h["arms"]:
        p = a["laws"]["applied"]
        assert p["n"] == a["laws"]["sched"]["n"], (a["taus"], p["n"])
        # NO WINDUP: the masked leg ends AT ITS FLOOR, and never exceeds the live clip's scale
        assert p["final_g_fuel"] == 0.0, (a["inc"], a["taus"], p["final_g_fuel"])
        assert p["max_masked"] < a["laws"]["sched"]["max_masked"], (a["inc"], a["taus"])
        # the IC is unchanged: both legs open dormant (rung 72's P9, inherited)
        assert p["ic_iters"] == 1 and p["ic_res"] == 0.0


# ======================================================================================
# § 1 — THE COUPLING IS REAL AND LANDS IN THE WRONG COLUMN (anchor P7, § 0.4)
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("inc", [False, True])
def test_the_masked_leg_couples_and_still_reaches_nothing(design, inc):
    """RUNG 72 § 11's PREMISE HOLDS AND ITS CONCLUSION DOES NOT, in four numbers:

        cross_masked ≈ −1     the masked leg DOES read the authoritative one — `F_r != 0`
        self_masked  ≈ +1     and its own state: it is an INTEGRATOR
        self_live    == 0.0   while the HOLDING leg's applied reference IS the scheduled one
        mask_leak    == 0.0   and the masked leg STILL reaches the plant through nothing

    THE TWO EXACT ZEROS ARE GATED AS EQUALITY AND THE TWO ONES ARE NOT, which is not a
    double standard: `self_live` is exact because the hook takes an explicit identity BRANCH,
    while `self_masked` is a central difference of a SUM (`gf ± dg + raw − gr`) and float
    addition does not distribute. An exact zero survives a difference quotient; an exact one
    does not."""
    g = _applied(design, bleed_lim=_valve()).applied_gains(
        FLIGHT, LO, HI, TT4_MAX, SM, taus=CLOCKS[0], inc=inc)
    assert g["rows"] and g["skipped"] == dict(switch=0, regime=0)
    assert g["self_live"] == [0.0]
    assert g["worst_mask_leak"] == 0.0
    assert all(abs(v - 1.0) < 1e-9 for v in g["self_masked"]), g["self_masked"]
    assert all(abs(v + 1.0) < 1e-9 for v in g["cross_masked"]), g["cross_masked"]
    # and the plant is not trivially decoupled — the LIVE gains are non-zero
    assert g["min_live_gain"] > 1e-4


@pytest.mark.slow
@pytest.mark.parametrize("inc", [False, True])
def test_only_two_entries_of_J_move_and_both_by_one_over_tau(design, inc):
    """THE ENTRYWISE `J(73) − J(72)`, at the SAME base points (rung 71's device, rung 72 § 4's:
    one law swapped, nothing else). **14 of the 16 entries are EXACTLY 0.0**, and the two that
    move — the masked leg's own diagonal and its cross-gain onto the AUTHORITATIVE axis — are
    both exactly `1/tau_masked`. That is the whole reach of the reference."""
    g = _applied(design, bleed_lim=_valve()).applied_gains(
        FLIGHT, LO, HI, TT4_MAX, SM, taus=CLOCKS[0], inc=inc)
    assert g["worst_delta_rest"] == 0.0, g["worst_delta_rest"]
    assert g["moved_scaled"], "nothing moved — the reference reader is dead (see § 8)"
    assert all(abs(abs(v) - 1.0) < 1e-9 for v in g["moved_scaled"]), g["moved_scaled"]
    # both signs are present: +1 on the diagonal, −1 on the cross-gain
    assert min(g["moved_scaled"]) < 0.0 < max(g["moved_scaled"])


# ======================================================================================
# § 2 — EVERY ZERO COUNT PLUS ONE, AND A DETERMINANT THAT DIES (anchor P1, P2, P3, P4)
# ======================================================================================

@pytest.mark.slow
def test_every_cell_is_its_rung72_parent_plus_one_zero(design):
    """**The plant is STILL rung 68/69/70/71 plus a pole — and the pole is now at the ORIGIN.**
    `zeros = n_live − m_live + n_masked`: one value per cell, all four cells, each exactly one
    above rung 72's own count."""
    c = _applied(design, bleed_lim=_valve()).applied_cells(FLIGHT, LO, HI, TT4_MAX, SM,
                                                           clocks=CLOCKS)
    assert c["all_four_cells"] and c["law_holds"], c["cells"]
    for k, d in c["cells"].items():
        assert d["zeros"] == [PREDICTED[k]], (k, d["parent"], d["zeros"], PREDICTED[k])
        assert PREDICTED[k] == RUNG72[k] + 1
        assert d["n"] >= 4 and d["n_parent"] == d["n"], (k, d["n"], d["n_parent"])
    # the RK4 floor, MEASURED rather than trusted (rung 65's retraction)
    assert c["worst_lam"] < 1.0, c["worst_lam"]


@pytest.mark.slow
def test_the_parent_polynomial_survives_with_the_pole_at_the_origin(design):
    """`p4(lam) = lam * p3(lam)`, `p3` rebuilt from the SHIPPED rung-68/69/70/71 readers.
    Coefficients, not roots — and the argument is STRONGER here than in rung 72, because the
    added root is exactly zero, so every cell has at least a DOUBLE zero root and a root match
    would resolve it only to sqrt(eps).

    **`gap` AND `null` ARE ONE NUMBER, NOT TWO.** The masked column's only non-zero entry is its
    own diagonal, and `a3` is minus the trace, so the `j = 1` term of `gap` reproduces `null`
    entry for entry. Quoting both as agreement would be this family's sixth
    instrument-agrees-with-itself; `gap_hi` (`j = 2, 3, 4`) is where the two INDEPENDENT readers
    actually meet, and it is gated separately."""
    c = _applied(design, bleed_lim=_valve()).applied_cells(FLIGHT, LO, HI, TT4_MAX, SM,
                                                           clocks=CLOCKS)
    assert c["worst_parent_gap"] < 1e-10, c["worst_parent_gap"]
    assert c["worst_parent_gap_hi"] < 1e-10, c["worst_parent_gap_hi"]
    # the two readers land on the SAME manifold base point — a mismatch there is ruled out
    assert c["worst_v_gap"] == 0.0
    # the zero EIGENVECTOR lies ON the masked axis: `A e_masked = 0`. THE GATED HALF of the
    # pole claim — the eigenVALUE is reported, never gated (rung 72 § 1.2's discipline).
    assert c["worst_null"] < 1e-10, c["worst_null"]


@pytest.mark.slow
def test_the_determinant_dies_in_the_one_cell_where_it_lived(design):
    """Rung 72 measured `det J = +5.9e4` in rung 71's cell — the only live determinant in the
    whole family — and `≈ 0` in the other three. Under the applied reference it is dead in ALL
    FOUR. **A reference is not a gain, not a clock and not a loop, and it changes the RANK.**"""
    c = _applied(design, bleed_lim=_valve()).applied_cells(FLIGHT, LO, HI, TT4_MAX, SM,
                                                           clocks=CLOCKS)
    rung71_cell = c["cells"][(True, "gov")]
    assert rung71_cell["parent"] == "rung 71"
    # normalised by the rate^4 the determinant scales with, it is eleven orders below rung 72's
    assert c["worst_det"] < 1e-3, c["worst_det"]
    assert rung71_cell["zeros"] == [1]


# ======================================================================================
# § 3 — THE ISOLATION INSTRUMENT: reading C moves the OTHER half (anchor P5)
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("inc", [False, True])
def test_the_two_readings_move_disjoint_halves_of_the_matrix(design, inc):
    """Reading C is the LITERAL reading of rung 72 § 11 (`req = mf_app − cap`, no increment): a
    well-posed proportional law with 2x droop, refused as the plant only because a leg that
    cannot reach its own floor measures a different object than rungs 46–72 did.

        B: the pole MOVES to the origin, the LIVE diagonal is unmoved, `M3` IS the parent's
        C: the pole STAYS at `−1/tau_m`, the LIVE diagonal moves by exactly −1, `M3` is not

    Two readings that agree on `F_r != 0` and disagree on everything it was supposed to imply.
    That is what makes the headline a measurement rather than a choice of law."""
    d = _applied(design, bleed_lim=_valve()).ref_discriminator(
        FLIGHT, LO, HI, TT4_MAX, SM, taus=CLOCKS[0], inc=inc)
    assert d["n"] > 0
    # C and rung 72 keep the free pole at −1/tau_masked; B does not
    assert d["worst_pole_C"] < 1e-9 and d["worst_pole_72"] < 1e-9
    assert d["best_pole_B"] > 1e-2, d["best_pole_B"]
    # the LIVE leg's own diagonal: EXACTLY 0 under B (the identity branch), EXACTLY −1 under C
    assert d["live_diag_B"] == [0.0] and d["live_diag_C"] == [-1.0]
    # AND THE COUNTS SEPARATE ALL THREE READINGS — differenced PER POINT, because this reader
    # spans both authority cells and their counts already differ by one under rung 72 alone, so
    # a pooled comparison would compare one cell against the other and say nothing. (The naive
    # "is there a root at the origin" test cannot separate them at all: rung 72 already has
    # zero roots in three of its four cells.)
    assert d["dzeros_B"] == [1], d["dzeros_B"]
    assert max(d["dzeros_C"]) <= 0, d["dzeros_C"]


# ======================================================================================
# § 4 — THE LEDGER: what the scheduled reference was quietly buying (anchor P6)
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("inc", [False, True])
def test_rung72_under_reported_its_own_peak_debit(design, inc):
    """Rung 72 § 5 reports the fuel leg's marginal peak `Tt4` debit as +0.29 K / +1.86 K and
    calls the `phi` credit the finding. **Under the correct reference the debit is 110x and 39x
    larger** — because the fuel leg's own authority window is EARLY, where the reference is the
    identity, while the governor's is LATE, where it is not, and a masked governor given credit
    for a cut it did not make takes the actuator too soon.

    The ordering is the claim and every magnitude is disclaimed; the gate is a 10x floor."""
    b = _applied(design, bleed_lim=_valve()).applied_bill(FLIGHT, LO, HI, TT4_MAX, SM,
                                                          taus=CLOCKS[0], inc=inc)
    assert b["debit_sched"] > 0.0 and b["debit_applied"] > 0.0
    assert b["debit_ratio"] > 10.0, (b["debit_sched"], b["debit_applied"])
    # the hand-over is later in the ledger's own full cell, too
    assert b["handover_applied"] > b["handover_sched"]
    # and `min phi` is UNMOVED, so the debit is not bought by moving the other currency
    assert b["phi_full_applied"] == b["phi_full_sched"]


# ======================================================================================
# THE REFUSALS (anchor P10)
# ======================================================================================

def test_refuses_the_applied_reference_on_top_of_the_sum_law(design):
    """TWO DECLARED LAWS AT ONCE. Under `sum` the hook never takes its identity branch, BOTH
    fuel rows gain a cross term and the block form goes — a fourth plant, whose result could be
    attributed to neither law. Rung 63's lesson in its plainest form."""
    m = _applied(design, bleed_lim=_valve(), stator_lim=_phi_stator())
    m._share_law = "sum"
    with pytest.raises(AssertionError, match="TWO declared"):
        _march(m, Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_surge(), lag=_lag())


def test_refuses_an_undeclared_reference(design):
    m = _applied(design, bleed_lim=_valve(), stator_lim=_phi_stator())
    m._ref_law = "whatever"
    with pytest.raises(AssertionError, match="DECLARED"):
        _march(m, Tt4_max=TT4_MAX, tau_gov=TAU_GOV, surge=_surge(), lag=_lag())


def test_the_rk4_floor_is_re_justified_and_still_armed(design):
    """THE SIXTH JUSTIFICATION, and the previous five do not carry: the masked leg's eigenvalue
    is exactly ZERO, which is neutrally stable, so 'the dominant root is below the rate sum' is
    no longer the sentence. The constant is unchanged and the message says why."""
    m = _applied(design, bleed_lim=_valve(0.005), stator_lim=_phi_stator(0.005))
    with pytest.raises(AssertionError, match="rung-73.*origin"):
        m._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.02, Tt4_max=TT4_MAX, tau_gov=0.002,
                        surge=_surge(), lag=_lag(0.002, 0.006))


def test_the_inherited_refusals_are_still_armed(design):
    """Rung 72's own five stay live through this rung's `integrate_fuel` override."""
    m = _applied(design, bleed_lim=_valve(), stator_lim=_phi_stator())
    with pytest.raises(AssertionError, match="no set point"):
        _march(m, tau_gov=TAU_GOV, surge=_surge(), lag=_lag())
    # refused TWICE OVER, and the outer refusal is STRUCTURAL: `_stator_march` does not plumb
    # `s_off`/`tau_rel` at all (rung 71 § 8.2's reading, inherited through rung 72), so the
    # inner assert is reached directly because there is no other way to reach it.
    import inspect
    sig = inspect.signature(AppliedReferenceTransient._stator_march).parameters
    assert "s_off" not in sig and "tau_rel" not in sig, sorted(sig)
    with pytest.raises(AssertionError, match="FORCED release"):
        m.integrate_fuel(FLIGHT, lambda s: 1.0, (1.0, 1.0), 0.1, DS, Tt4_max=TT4_MAX,
                         tau_gov=TAU_GOV, surge=_surge(), lag=_lag(), s_off=0.3)


def test_the_reference_lives_in_one_place(design):
    """`_reference` is the ONE seat of the law, as `_applied_clip` is for the composition — so
    no reader can compose it differently from the march that produced its base point."""
    import inspect
    src = inspect.getsource(AppliedReferenceTransient)
    assert src.count("g_own + req - clip") == 1, (
        "the applied reference must appear exactly once in this class")
    # and the hook the parent calls is the parent's only seat for it
    psrc = inspect.getsource(SharedActuatorTransient._integrate_fuel_shared)
    assert psrc.count("self._reference(") == 4, (
        "the march must reach the reference ONLY through the hook: twice in `der` and twice "
        "in the initial-condition sweep, once per leg in each")


# --- THE MARCH AUDIT: rung 79's gap seam, checked from the other end ------------------------
# `docs/rungs72-77-march-audit.md`. A CONFIRMATION's gate, not this rung's anchor.

@pytest.mark.slow
def test_this_rungs_march_MOVES_and_all_four_loops_are_live(design):
    """The applied reference does not change the answer rung 72's arm gives: at `phi_lim = 0.80`
    in the CLIP coordinate the plant accelerates, and all four loops act. Rungs 78/79 stand
    still at the same wall in the DEMAND coordinate (rung 74 s 2.2) -- the cell, not the rig."""
    m = _applied(design, bleed_lim=_valve())
    traj = m._shared_march(FLIGHT, LO, HI, TT4_MAX, SM, CLOCKS[0], R, SETTLE, DS,
                           V_MAX, False)[3]
    assert len(traj) > 300, len(traj)
    nu = [p["nu_lp"] for p in traj]
    assert (max(nu) - min(nu)) / min(nu) > 1e-2, (min(nu), max(nu))
    t4 = [p["Tt4"] for p in traj]
    assert max(t4) - min(t4) > 200.0, (min(t4), max(t4))
    b_max = m.bleed_lim.b_max
    assert sum(1 for p in traj if p["required"] > 0.0) > 300
    assert sum(1 for p in traj if 0.0 < p["b_cmd"] < b_max) > 50
    assert sum(1 for p in traj if p.get("v_regime") == "riding") > 50
    assert min(p["phi_lp"] for p in traj) > 0.78, min(p["phi_lp"] for p in traj)
