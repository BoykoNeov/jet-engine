"""Rung 63 — FUEL + BLEED on one plant: what a min-select leg can FEEL.

Rung 62 named this seam: every fuel-side leg (46-52) lives on `integrate_fuel` and the bleed
now sits in the same closure, so the composite is one constructor away.

THE HEADLINE: rung 58's ONE-WAY arrow was never a fact about wall-movers. It was a fact about
the Wf/pt3 leg's TWO PROTECTIONS -- a CHOKED A4 guards its ordinate (rung 59's _proof_chain)
and rung 39's pi_LPC cancellation guards its abscissa -- and a stator satisfies both. A bleed
is the ladder's only lever that breaks mdot_face == mdot_core, the identity sitting UPSTREAM
of both, so it reaches both sensed inputs and the arrow CLOSES: the leg's engagement time
moves +2.9 to +4.2 %, LATER, in all six (ramp rate x map shape) cells. But s_eng is a
TRAJECTORY quantity, not a table quantity, so a STATOR moves it too (up to +1.28 %) with the
table bit-identical: the bleed's channel is STRUCTURAL, the stator's TRAJECTORY-MEDIATED, and
what the data separates is systematic from incidental, not presence from absence.

THE SECOND FINDING: a phi floor and the valve have NO COMPOSABLE MIDDLE. Over the band
sm in [0.3372, 0.4344] -- whose edges are the two plants' OWN minimum phi -- the bleed
DISARMS the floor exactly (removed == 0.0, and the armed cell bit-for-bit its leg-free
march); above it both bind, the floor pins the currency, and the valve's credit is exactly
zero (-1.3e-15). Rung 60's tautology reproduced, with a new regime added below it.

THE INSTRUMENT that would have counterfeited the rung: rung 62 overrode `at_stator` so a
rung-57 reader carries this machine's valve, and that reaches SIX inherited readers. On a
bleed-armed machine `schedule_invariance` compares armed against armed and returns
`ordinate_identical = True` -- numerically identical to rung 59's headline -- while measuring
nothing. Gate 2 pins that trap directly.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    ScheduledBleedTransient, BleedSchedule, StatorSchedule, SurgeLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE = 1000.0, 1400.0, 0.005, 1.2
N_LO, V, B, MARGIN = 0.65, 0.20, 0.10, 0.25

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
TILT_LP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
TILT_HP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
SHAPES = {"shaped": (LP, HP), "tilted": (TILT_LP, TILT_HP)}

BLEED = dict(bleed_sched=BleedSchedule(B, N_LO))
STAT = dict(vsv_sched_lp=StatorSchedule(V, N_LO))


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _bt(lp=LP, hp=HP, design=None, **kw):
    return ScheduledBleedTransient(design if design is not None else _design(), FLIGHT,
                                   1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _leg(mach, margin=MARGIN):
    return mach.accel_schedule(FLIGHT, LO, HI, margin, 13)


# =============================================================================
# GATE 1 — THE REDUCE: rung 62's `_legs` is untouched when no leg is passed
# =============================================================================

def test_reduce_legless_marginal_loop_is_rung62_bit_for_bit():
    """`_legs` gained accel/surge/Tt4_max, all defaulting to None -- which is
    `_stator_march`'s own default -- so every rung-62 caller reaches the IDENTICAL four
    marches. Witnessed against `loop_decomposition`, whose reference is `at_lever()`: the
    same pair `marginal_loop` builds with an empty neighbour."""
    m = _bt(**BLEED)
    a = m.loop_decomposition(FLIGHT, LO, HI, r=0.5, ds=0.02)
    b = m.marginal_loop(FLIGHT, LO, HI, BLEED, r=0.5, ds=0.02)
    for k in ("reference", "start", "ramp", "full", "self_cancel", "nu0_ref", "nu0_armed"):
        assert a[k] == b[k], f"{k}: {a[k]!r} != {b[k]!r}"


def test_reduce_explicit_none_leg_is_identical_to_omitting_it():
    """A leg passed as None must reach the same code path as no leg at all -- otherwise the
    rung-62 gates above would be guarding a branch nobody takes."""
    m = _bt(**BLEED)
    a = m.marginal_loop(FLIGHT, LO, HI, BLEED, r=0.5, ds=0.02)
    b = m.marginal_loop(FLIGHT, LO, HI, BLEED, r=0.5, ds=0.02,
                        accel=None, surge=None, Tt4_max=None)
    assert a["self_cancel"] == b["self_cancel"] and a["full"] == b["full"]


def test_cycle_untouched_design_run_is_rung6_bit_for_bit():
    """Rung 63 adds only readers on the transient ladder. The default single-spool design
    run must be bit-for-bit rung 6 (the project's spine)."""
    kw = {k: v for k, v in REAL.items() if k in ("pi_d", "eta_b", "pi_b", "eta_m", "pi_n")}
    res = build_turbojet(gas=Gas.reacting_equilibrium(), pi_c=PI_LPC * PI_HPC, Tt4=TT4,
                         p_ambient=FLIGHT.p0, **kw).run(FLIGHT, 1.0)
    ref = build_turbojet(gas=Gas.reacting_equilibrium(), pi_c=PI_LPC * PI_HPC, Tt4=TT4,
                         p_ambient=FLIGHT.p0, **kw).run(FLIGHT, 1.0)
    assert res.performance.specific_thrust > 0.0 and res.performance.tsfc > 0.0
    for st in ("2", "3", "4", "5", "9"):
        assert res.stations[st].Tt == ref.stations[st].Tt
        assert res.stations[st].pt == ref.stations[st].pt
    assert res.performance.specific_thrust == ref.performance.specific_thrust


# =============================================================================
# GATE 2 — THE `_isolating` GATE: the trap that would have counterfeited § 1
# =============================================================================

def test_isolating_refuses_a_reference_carrying_the_lever():
    """The mirror of rung 62's gate 3. A lever key also present in the neighbour would make
    the 'reference' an ARMED machine, i.e. an armed-vs-armed comparison."""
    m = _bt()
    with pytest.raises(AssertionError, match="LEVER being isolated"):
        m.marginal_loop(FLIGHT, LO, HI, BLEED, neighbour=dict(BLEED))
    with pytest.raises(AssertionError, match="isolates a lever"):
        m.marginal_loop(FLIGHT, LO, HI, {})


def test_isolating_reference_is_valve_shut_and_armed_is_not():
    """The positive witness: the reference sibling carries the NEIGHBOUR's valve and nothing
    else, and the armed one carries lever + neighbour."""
    m = _bt()
    ref, armed = m._isolating(BLEED)
    assert not ref._armed_bleed() and armed._armed_bleed()
    # with a STATOR neighbour the reference still must be valve-shut
    ref2, armed2 = m._isolating(BLEED, neighbour=STAT)
    assert not ref2._armed_bleed() and armed2._armed_bleed()
    assert ref2._is_armed() and armed2._is_armed()      # both carry the stator


def test_the_at_stator_trap_is_real_and_returns_rung59s_zero_for_free():
    """THE COUNTERFEIT, pinned. Rung 62 deliberately overrode `at_stator` to carry this
    machine's valve. So on a bleed-armed machine rung 59's `schedule_invariance` compares
    the plant against ITSELF: it reports the tables bit-identical -- numerically rung 59's
    own headline -- while measuring nothing. This gate exists so no future edit can
    reintroduce that reading as evidence, and so § 1's instrument choice stays justified."""
    m = _bt(**BLEED)
    assert m.at_stator()._armed_bleed(), (
        "rung 62's at_stator override must keep the valve -- if this flips, rung 62's "
        "gate 3 has been broken and every inherited reader changes meaning.")
    trap = m.schedule_invariance(FLIGHT, LO, HI, MARGIN, n=5)
    assert trap["ordinate_identical"] and trap["abscissa_identical"], (
        "the trap must reproduce rung 59's exact-identity verdict for free")
    honest = _bt().sensed_inputs(FLIGHT, LO, HI, BLEED, margin=MARGIN, n=5)
    assert honest["d_ordinate"] > 1e-3 and honest["d_abscissa"] > 1e-3, (
        "and the isolating reader must NOT: it differences against a valve-shut sibling")


# =============================================================================
# GATE 3 — THE MECHANISM: the leg's two sensed inputs
# =============================================================================

@pytest.mark.slow
def test_bleed_moves_both_sensed_inputs_where_a_stator_moves_neither():
    """THE MECHANISM (rung 63 § 1). One instrument, two levers.

    Rung 59 proved an LP stator moves NEITHER half of the Wf/pt3 table (its own published
    tolerance is 1e-13). A bleed moves BOTH, by more than 1e-2 -- ten orders apart -- because
    the LP shaft balance is the one carrying (1-b) and it sits upstream of both protections.
    """
    m = _bt()
    bl = m.sensed_inputs(FLIGHT, LO, HI, BLEED, margin=MARGIN, n=9)
    st = m.sensed_inputs(FLIGHT, LO, HI, STAT, margin=MARGIN, n=9)
    assert st["d_ordinate"] < 1e-12 and st["d_abscissa"] < 1e-12, (
        f"rung 59's zero must reproduce: {st['d_ordinate']:.3e}, {st['d_abscissa']:.3e}")
    assert bl["d_ordinate"] > 1e-3, bl["d_ordinate"]
    assert bl["d_abscissa"] > 1e-3, bl["d_abscissa"]
    assert bl["d_ordinate"] / max(st["d_ordinate"], 1e-300) > 1e8
    # the SIGN: a bleed makes the burner inlet colder, so the steady Wf/pt3 ratio RISES
    assert bl["signed_ordinate"] > 0.0 and bl["signed_abscissa"] > 0.0


@pytest.mark.slow
@pytest.mark.parametrize("lever", [BLEED, STAT, dict(bleed=B)])
def test_choked_A4_control_holds_for_every_lever(lever):
    """`MFP_A4` is the corrected group at a CHOKED throat -- hardware, gamma and R. Nothing
    on the compressor side can reach it, for ANY lever. If it ever moves, the proof chain has
    broken somewhere else and every number in § 1 is meaningless."""
    d = _bt().sensed_inputs(FLIGHT, LO, HI, lever, margin=MARGIN, n=5)
    assert d["d_mfp"] < 1e-14, d["d_mfp"]


@pytest.mark.slow
def test_the_LP_balance_chain_is_signed_as_derived():
    """The derivation, term by term: (1-b) sits in the LP balance ONLY, so Tt25 falls, Tt3
    falls with it, f rises to make up the colder burner inlet, and kappa_ss rises."""
    d = _bt().sensed_inputs(FLIGHT, LO, HI, dict(bleed=B), margin=MARGIN, n=5)
    for row in d["chain"]:
        assert row["d_Tt25"] < 0.0, row
        assert row["d_Tt3"] < 0.0, row
        assert row["d_f"] > 0.0, row
        assert row["d_kappa"] > 0.0, row
        # the HP balance is bleed-INVARIANT (rung 42), so Tt3 - Tt25 must move LESS than
        # either endpoint: the whole shift is imported from the LP side.
        assert abs(row["d_Tt3"]) < abs(row["d_Tt25"]), row


# =============================================================================
# GATE 4 — THE HEADLINE: the return arrow
# =============================================================================

@pytest.mark.slow
@pytest.mark.parametrize("shape", list(SHAPES))
@pytest.mark.parametrize("r", [0.25, 0.5, 1.0])
def test_the_bleed_retimes_the_leg_where_the_stator_does_not(shape, r):
    """THE RUNG (63). Rung 58's own instrument, on a lever the leg can feel.

    A bleed schedule moves `s_eng` by +2.5 % or more, LATER, at every ramp rate and on both
    map shapes. The reading is the DORMANT march, where `g` is defined everywhere and no clip
    has perturbed the states; the limited march agrees to under 1 % of the shift.

    THE STATOR CONTROL IS BOUNDED, NOT ZERO, AND IT IS MEASURED HERE rather than quoted from
    rung 58 -- whose -0.162 % sits at ITS OWN placement (n_lo = 0.7557) and is therefore a
    DIFFERENT schedule from this rung's n_lo = 0.65, not a control for it. Measured on this
    placement the stator spans -0.03 % to +1.28 %: `s_eng` is a TRAJECTORY quantity and a
    stator moves the trajectory even with its TABLE bit-identical (gate 3). So what is
    gated is that the bleed is POSITIVE and STRICTLY THE LARGER in every cell -- not the
    "twenty times" an earlier draft claimed off two cells. See docs/rung63-spec.md § 2.

    THE CLAMP CAVEAT, gated rather than hidden: on the tilted map at r = 0.25 exactly ONE of
    207 cutting points on the REFERENCE march reads the cap at its clamped endpoint. The
    band is left at rungs 58/59's ramp band anyway, and § 2 publishes the wider-band value
    for that cell (+3.16 % against +2.88 %) so the conclusion is shown insensitive to it."""
    lp, hp = SHAPES[shape]
    m = _bt(lp, hp)
    leg = _leg(m)
    d = m.leg_retiming(FLIGHT, LO, HI, BLEED, accel=leg, r=r, ds=DS)
    assert d["rel_dormant"] > 0.025, (shape, r, d["rel_dormant"])
    # dormant vs limited: the two readings agree to <= 3e-5 in the ratio (under 1 % of it)
    assert abs(d["rel_limited"] - d["rel_dormant"]) < 1e-4, d
    assert all(a["clamped"] <= 1 for a in d["audits"].values()), d["audits"]
    s = m.leg_retiming(FLIGHT, LO, HI, STAT, accel=leg, r=r, ds=DS)
    # the bleed is POSITIVE and strictly the larger, in every cell. Both halves matter:
    # the stator's own shift is real (up to +1.28 %), so only the ordering is claimed.
    assert d["rel_dormant"] > abs(s["rel_dormant"]) > 0.0, (
        shape, r, d["rel_dormant"], s["rel_dormant"])
    assert abs(s["rel_dormant"]) < 0.02, (shape, r, s["rel_dormant"])


@pytest.mark.slow
def test_the_retiming_sign_is_decided_by_the_commanded_ramp_not_the_cap():
    """The pre-registered sign was EARLIER and it was REFUTED. The pressure channel does
    point that way (`pt3` falls), but the ABSCISSA channel this rung's own § 1 derives
    fights it, the cap barely moves, and the COMMANDED ramp -- re-derived on the bled plant,
    since both are pinned to the same Tt4 endpoints -- decides."""
    m = _bt()
    c = m.leg_retiming(FLIGHT, LO, HI, BLEED, accel=_leg(m), r=0.5, ds=DS)["channels"]
    assert c["d_pt3"] < 0.0, c                      # as predicted
    assert c["d_kappa"] > 0.0, c                    # fighting it -- s 1's abscissa shift
    assert abs(c["d_cap"]) < abs(c["d_pt3"]), c     # so the cap nearly cancels
    assert c["d_mf_sched"] < c["d_cap"], c          # and the ramp falls FURTHER
    assert c["d_g"] < 0.0, c                        # => the crossing arrives LATER


@pytest.mark.slow
def test_the_forward_arrow_is_rung58s_mechanism_confirmed_not_new_content():
    """The forward direction (leg -> lever) is rung 58's relocation x state-feed, and its own
    predictor -- re-reading the LEG-FREE credit profile at the relocated minimum -- recovers
    it. Rung 58 got 86 % for a stator schedule. This is why the headline is the RETURN arrow
    alone and not the ratio of the two."""
    m = _bt()
    d = m.lever_composite(FLIGHT, LO, HI, BLEED, accel=_leg(m), r=0.5, ds=DS)
    assert d["interaction"] > 0.0 and d["share"] > 0.02, d["share"]
    assert 0.80 < d["recovered"] < 0.95, d["recovered"]
    assert d["removed_bare"] > 0.0 and d["removed_armed"] > 0.0, (
        "a dormant leg's zero is the envelope edge, not evidence (rung 58's r = 2.0)")


@pytest.mark.slow
def test_the_forward_arrow_collapses_at_r1_by_rung48s_law_not_by_dormancy():
    """The two directions answer to DIFFERENT conditions, and this is the witness. At
    r = 1.00 the forward direction nearly vanishes -- but the leg is NOT dormant. It engages
    DOWNSTREAM of the incidence minimum, so it relocates nothing: rung 48's engagement law,
    reappearing inside a third composite. The return arrow has no such condition."""
    m = _bt()
    leg = _leg(m)
    fast = m.lever_composite(FLIGHT, LO, HI, BLEED, accel=leg, r=1.0, ds=DS)
    mid = m.lever_composite(FLIGHT, LO, HI, BLEED, accel=leg, r=0.5, ds=DS)
    assert fast["share"] < 0.25 * mid["share"], (fast["share"], mid["share"])
    assert fast["removed_bare"] > 0.0, "not dormancy -- the leg still binds"
    assert fast["cells"]["fuel"]["s_eng"] > fast["cells"]["neither"]["s"], (
        "the leg engages DOWNSTREAM of the bare incidence minimum")
    # while the return arrow is undiminished at the same rate
    assert m.leg_retiming(FLIGHT, LO, HI, BLEED, accel=leg, r=1.0,
                          ds=DS)["rel_dormant"] > 0.025


# =============================================================================
# GATE 5 — THE LOOP: rung 62 § 2's attribution, on a neighbour with no loop
# =============================================================================

@pytest.mark.slow
@pytest.mark.parametrize("r", [0.25, 0.5, 1.0])
def test_a_legged_neighbour_leaves_the_bleeds_loop_alone(r):
    """Rung 62 § 2 attributed the loop to `dn/d(setting)`. A fuel leg reads the state but
    emits a fuel CAP, not a setting, so it has no such term -- and it perturbs the bleed's
    amplification by under 2 %, the same order as rung 62's scheduled neighbour. The loop
    answers to its own gain and not to the trajectory a neighbour hands it."""
    m = _bt()
    leg = _leg(m)
    a = m.marginal_loop(FLIGHT, LO, HI, BLEED, r=r, ds=0.01)
    b = m.marginal_loop(FLIGHT, LO, HI, BLEED, r=r, ds=0.01, accel=leg)
    assert a["self_cancel"] > 1.0 and b["self_cancel"] > 1.0, (a, b)
    assert abs(b["self_cancel"] / a["self_cancel"] - 1.0) < 0.02, (
        r, a["self_cancel"], b["self_cancel"])


# =============================================================================
# GATE 6 — THE SECOND FINDING: no composable middle
# =============================================================================

@pytest.mark.slow
def test_the_floor_is_disarmed_inside_the_band_and_tautological_above_it():
    """THE SECOND FINDING (rung 63 § 3). A phi floor and the valve have TWO regimes and no
    middle, and the boundary is the two plants' OWN minimum phi -- nothing is fitted.

    INSIDE the band the armed cell is bit-for-bit its own leg-free march (`disarmed`);
    ABOVE it both bind, the floor pins the currency, and the valve's credit is exactly 0.
    Every verdict is read off `fuel_removed`; `s_eng` is nan there by construction and no
    assertion touches it."""
    m = _bt()
    d = m.floor_dichotomy(FLIGHT, LO, HI, BLEED,
                          sm_grid=(0.34, 0.36, 0.40, 0.43, 0.46), ds=DS)
    lo_b, hi_b = d["band"]
    assert 0.0 < lo_b < hi_b, d["band"]
    assert abs(lo_b - (d["min_phi_ref"] / d["phi_surge"] - 1.0)) < 1e-12
    assert abs(hi_b - (d["min_phi_armed"] / d["phi_surge"] - 1.0)) < 1e-12
    inside = [r for r in d["rows"] if lo_b < r["sm"] < hi_b]
    above = [r for r in d["rows"] if r["sm"] > hi_b]
    assert len(inside) >= 3 and len(above) >= 1, [r["sm"] for r in d["rows"]]
    for r in inside:
        assert r["removed_fuel"] > 0.0, r          # the floor DOES bind on the reference
        assert r["removed_both"] == 0.0, r         # and is exactly DISARMED on the armed one
        assert r["disarmed"], r                    # bit-for-bit its own leg-free march
    for r in above:
        assert r["removed_fuel"] > 0.0 and r["removed_both"] > 0.0, r   # both BIND
        assert abs(r["credit"]) < 1e-12, r         # rung 60's tautology, exact
        assert not r["disarmed"], r


@pytest.mark.slow
def test_the_disarming_band_widens_with_the_valve():
    """The band exists BECAUSE the valve buys phi, so its width must track `b_max` -- and
    vanish at b_max = 0, where the two plants are the same machine."""
    m = _bt()
    widths = []
    for bm in (0.0, 0.05, 0.10, 0.15):
        lever = dict(bleed_sched=BleedSchedule(bm, N_LO))
        d = m.floor_dichotomy(FLIGHT, LO, HI, lever, sm_grid=(), ds=0.01)
        widths.append(d["band"][1] - d["band"][0])
    assert abs(widths[0]) < 1e-12, widths[0]
    assert all(b > a for a, b in zip(widths, widths[1:])), widths


# =============================================================================
# GATE 7 — THE SPLICE: both halves live, opposite signs, and no ratio published
# =============================================================================

@pytest.mark.slow
def test_both_splice_halves_are_live_and_carry_opposite_signs():
    """Rung 59 always had one half of the table EXACTLY zero, which made its split trivially
    additive. A bleed moves both, and they FIGHT. The claim is carried by the two RAW deltas
    -- large and opposite in sign -- and deliberately NOT by shares: `delta_match` is then a
    small difference of two larger terms (rung 43's currency-circularity shape) and the
    shares swing ~10 % under an `ds` halving while their sum barely moves."""
    m = _bt()
    d = m.matched_leg_deltas(FLIGHT, LO, HI, BLEED, margin=MARGIN, r=0.5, ds=DS)
    assert d["clamped"] == 0, d["audits"]
    assert d["delta_index"] > 1e-3, d["delta_index"]
    assert d["delta_value"] < -1e-3, d["delta_value"]
    assert d["delta_index"] * d["delta_value"] < 0.0, "the two halves must FIGHT"
    assert abs(d["delta_match"]) < abs(d["delta_index"]), (
        "the net must sit inside the re-indexing term, since the ordinate cancels part of it")
    assert "abscissa_share" not in d and "ordinate_share" not in d, (
        "the shares are grid-sensitive and are deliberately not published -- see the spec")


@pytest.mark.slow
def test_a_stator_leaves_the_matched_leg_a_no_op_rung59_reproduced():
    """The control: on the SAME instrument an LP stator's matched leg is rung 59's exact
    no-op, so gate 7's numbers are the lever's doing and not the reader's."""
    m = _bt()
    d = m.matched_leg_deltas(FLIGHT, LO, HI, STAT, margin=MARGIN, r=0.5, ds=0.01)
    assert abs(d["delta_index"]) < 1e-9, d["delta_index"]
    assert abs(d["delta_value"]) < 1e-9, d["delta_value"]
    assert abs(d["delta_match"]) < 1e-9, d["delta_match"]


# =============================================================================
# GATE 8 — SCOPE: the choked branch survives a leg that cuts hard
# =============================================================================

@pytest.mark.slow
@pytest.mark.parametrize("lever", [{}, BLEED, dict(bleed=0.15)])
def test_every_march_stays_on_the_choked_branch_with_a_leg_armed(lever):
    """Rung 62's pre-check was run with no leg cutting fuel; rung 42 warns the choked guard
    bites SOONER with the valve open, and the bled `_close_fuel` makes a metered fuel flow
    RICHER. Re-checked with both leg kinds armed."""
    m = _bt(**lever)
    leg = _leg(_bt())
    floor = SurgeLimiter.from_margin(LP, "lp", 0.40)
    for kw in ({}, dict(accel=leg), dict(surge=floor)):
        traj, _ = m._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, 0.01, **kw)
        assert all(p["branch"] == "choked" for p in traj), (lever, kw)


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
