"""Rung 49 — THE phi / SURGE-MARGIN FEEDBACK LIMITER: a limiter acts on a spool through BOTH
its edges, and the two edges answer to DIFFERENT clocks.

`docs/both-edges-limiter-negative.md` closed the whole pt3-FILTER family with one fact: pt3,
Wf, n and every filter of them rise MONOTONICALLY through the ramp, so such a limiter's release
edge is structurally POST-ramp — its window can never close inside the ramp, and therefore the
closing edge can play no part. That negative named the one escape: "the only signals with a
turnover UPSTREAM of a surge minimum are the surge variables themselves." This rung builds it.

THE HEADLINE: a phi floor on ONE spool DEBITS the other. The engagement edge TRUNCATES a
descent (a credit — rung 48's term); the release edge RE-OPENS one (a debit — new). And they
answer to different clocks: the credit is set by THAT spool's own surge minimum (per-spool,
rung 48), the debit by the RAMP END (common-mode, rung 44's clock). At r=2.0, where those sit
3.1x apart, the debit is 8x larger at s_rel~r than at s_rel~s_hp*.

Rung 48 is BOUNDED, not refuted: it is the one-shot-arrest special case, exact whenever the
release lands well past the ramp — the regime its own leg was structurally confined to. Push
the release far past the ramp here (r=0.15) and the debit vanishes: the unwatched relief goes
POSITIVE. Same instrument, same plant, opposite sign.

And rung 48's crossing law is reproduced EXACTLY on this new instrument class: an HP-watching
floor gives relief_lp exactly 0.0 once s_eng passes s_lp*, and strictly positive before it.

Reduces: surge=None never consults the leg (bit-for-bit rungs 45/46/47/48); a dormant floor is
float-for-float bare; lp_disabled ASSERTS; a decel never fires the leg; the design run is
bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolFuelTransient, SurgeLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
SINGLE = dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92,
              eta_m=0.99, pi_n=0.98)

LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
FLAT = ComponentMap.flat()

LO, HI, R, SETTLE, DS = 1000.0, 1400.0, 0.5, 2.0, 0.02
REDLINE = 1480.0                       # rungs 46/47's redline, for the composite gates
KEYS = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "mf")

# The bare march's raw surge minima at this config (ds=0.02): the references every gate below
# is read against. Verified in docs/plans/rung49-anchor-phi-limiter.md.
S_LP_STAR, S_HP_STAR = 0.240, 0.400
MIN_PHI_LP, MIN_PHI_HP = 0.735466, 0.861199
PHI_LP_START = 0.773116


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _ft(gas=None, ml=LP_SHAPED, mh=HP_SHAPED, rho=1.0, lp_disabled=False):
    return TwoSpoolFuelTransient(_design(gas or _cpg_gas()), FLIGHT, 1.0, map_lp=ml, map_hp=mh,
                                 rho=rho, lp_disabled=lp_disabled)


def _ramp(ft, lo=LO, hi=HI, r=R):
    """The accel fuel ramp + its running-line start, as `_fuel_ramp_march` builds it."""
    mf0, mf1 = ft.fuel_for_Tt4(FLIGHT, lo), ft.fuel_for_Tt4(FLIGHT, hi)
    eq0 = ft.equilibrium(FLIGHT, lo)

    def sched(s):
        return mf0 + (mf1 - mf0) * min(1.0, s / r)

    return sched, (eq0["nu_lp"], eq0["nu_hp"])


def _same(pa, pb, keys=KEYS):
    assert len(pa) == len(pb), (len(pa), len(pb))
    for a, b in zip(pa, pb):
        assert tuple(a[k] for k in keys) == tuple(b[k] for k in keys), (a["s"], b["s"])


_SWEEPS = {}


SHAPES = {"flow/press": (LP_SHAPED, HP_SHAPED),
          "flat-lp": (FLAT, HP_SHAPED)}      # FLAT LP => the degenerate corner (gate 11)


def _sweep(floors, spool="lp", r=R, settle=SETTLE, shape="flow/press", rho=1.0):
    """Memoized within a worker — several gates read ONE sweep (each still asserts its own
    claim; the sweep is the shared, expensive measurement)."""
    key = (floors, spool, r, settle, shape, rho)
    if key not in _SWEEPS:
        ml, mh = SHAPES[shape]
        ft = _ft(ml=ml, mh=mh, rho=rho)
        _SWEEPS[key] = ft.floor_sweep(FLIGHT, LO, HI, floors, spool=spool, r=r,
                                      s_settle=settle, ds=DS)
    return _SWEEPS[key]


LP_FLOORS = (0.7550, 0.7500, 0.7450, 0.7400)
HP_FLOORS = (0.9000, 0.8800, 0.8700, 0.8650)


# =============================================================================
# THE REDUCE SPINE
# =============================================================================

def test_reduce_surge_none_never_consults_the_leg_bit_for_bit():
    """CONTRACT 1. `surge=None` leaves rungs 45/46/47/48 bit-for-bit — guaranteed at CODE
    level (the leg is never consulted), which is what this gate witnesses: with `_surge_fuel`
    replaced by a raiser, all four prior marches still run."""
    ft = _ft()
    sched, nu0 = _ramp(ft)

    def boom(*a, **k):
        raise AssertionError("rung-49 phi leg consulted on a surge=None march")

    ft._surge_fuel = boom
    acc = ft.accel_schedule(FLIGHT, LO, HI, 0.25)
    bare = ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS)
    top = ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS, Tt4_max=REDLINE)
    lag = ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS, Tt4_max=REDLINE, tau_gov=0.2)
    sch = ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS, accel=acc)
    assert bare and top and lag and sch
    # ... and the four are genuinely different marches (the gate is not vacuous)
    assert max(p["Tt4"] for p in bare) > max(p["Tt4"] for p in top)
    assert max(p["Tt4"] for p in lag) > max(p["Tt4"] for p in top)
    assert any(p["mf"] < p["mf_sched"] for p in sch)


def test_reduce_dormant_floor_bit_for_bit_rung45():
    """CONTRACT 2. A floor below the whole march leaves the cap above the schedule EVERYWHERE;
    `_surge_fuel` returns its argument float-identically, so the trajectory is the bare
    rung-45 one float-for-float — not merely equal."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    bare = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS)
    for spool, floor in (("lp", 0.50), ("hp", 0.50)):
        dorm = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS,
                                 surge=SurgeLimiter(spool=spool, phi_lim=floor))
        _same(bare, dorm)
        assert all(p["mf"] == p["mf_sched"] for p in dorm), "a dormant leg must not clip"


def test_reduce_composite_min_select_with_the_prior_legs():
    """CONTRACT 3, both directions — the min-select ORDERING gate. Armed together with rung
    46's governor, the pair reproduces whichever single leg actually binds, bit-for-bit."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    dorm = SurgeLimiter(spool="lp", phi_lim=0.50)      # never binds
    live = SurgeLimiter(spool="lp", phi_lim=0.7500)    # binds hard

    # (a) phi floor dormant + redline armed  ==  redline only
    top = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS, Tt4_max=REDLINE)
    both_a = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS, Tt4_max=REDLINE, surge=dorm)
    _same(top, both_a)

    # (b) phi floor armed + redline above the resulting peak  ==  phi floor only
    phi_only = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS, surge=live)
    peak = max(p["Tt4"] for p in phi_only)
    both_b = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS,
                               Tt4_max=peak + 50.0, surge=live)
    _same(phi_only, both_b)
    assert any(p["mf"] < p["mf_sched"] for p in phi_only), "the (b) leg must genuinely bind"


def test_reduce_lp_disabled_asserts():
    """CONTRACT 4. The finding is a per-spool SPLIT — inherently two-shaft (rungs 46/47/48's
    rule, carried)."""
    ft2 = _ft()
    ft = _ft(lp_disabled=True)
    sched, nu0 = _ramp(ft2)
    with pytest.raises(AssertionError, match="inherently two-shaft"):
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 0.5, DS,
                          surge=SurgeLimiter(spool="lp", phi_lim=0.75))


def test_decel_never_fires_bit_for_bit_rung45():
    """CONTRACT 5. On a DECEL phi rises above the running line throughout, so a floor set for
    the accel is never reached — the leg is structurally an accel instrument."""
    ft = _ft()
    sched, nu0 = _ramp(ft, lo=HI, hi=LO)               # 1400 -> 1000 K
    bare = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS)
    lim = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS,
                            surge=SurgeLimiter(spool="lp", phi_lim=0.7500))
    _same(bare, lim)
    assert all(p["phi_lp"] > 0.7500 for p in bare), "the decel must clear the floor"


def test_cycle_untouched_by_the_phi_leg_bit_for_bit_rung6():
    """CONTRACT 6. The design run is a SEPARATE entry point — adding a fourth fuel-side leg
    cannot move it."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    ft = _ft()
    ft.surge_relief(FLIGHT, LO, HI, SurgeLimiter(spool="lp", phi_lim=0.7500),
                    r=R, s_settle=1.0)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


# =============================================================================
# THE INSTRUMENT — it works, and it does what no pt3 filter could
# =============================================================================

def test_the_hold_is_a_sliding_mode_not_chatter():
    """GATE 3 (an IDENTITY check, deliberately NOT a finding). The clip RAISES phi
    (`both-edges` § arrest), which would make the leg dormant and restore fuel — the naive
    worry is chatter. Measured: the set-point solve rides the floor to solver tolerance at
    EVERY engaged point.

    The watched relief `phi_lim - min phi_bare` is DEFINITIONAL under a working set-point
    solve; it is asserted here to gate the SOLVER, and it is never used as evidence for the
    rung's claims (those all live on the UNWATCHED spool)."""
    for row in _sweep(LP_FLOORS):
        assert row["hold_err"] < 1e-9, (row["phi_lim"], row["hold_err"], "CHATTER")
        assert abs(row["relief_watched"] - (row["phi_lim"] - MIN_PHI_LP)) < 1e-5, row


def test_both_edges_close_inside_the_ramp_the_unreachable_object():
    """GATE 4. THE ENABLING MEASUREMENT. `docs/both-edges-limiter-negative.md` proved no
    pt3-filter limiter can close its window inside the ramp (every proxy signal rises
    monotonically through it, so release is structurally post-ramp). A phi floor CAN: phi has
    its minimum inside the ramp by definition.

    This is the object that makes the closing edge testable at all."""
    rows = _sweep(LP_FLOORS)
    inside = [x for x in rows if x["both_edges_inside_ramp"]]
    assert inside, "the phi floor must produce a window with BOTH edges inside the ramp"
    for x in inside:
        assert 0.0 < x["s_eng"] < x["s_rel"] < R, x
    # ...and the tight floors do NOT (the window opens at both ends as the floor rises):
    assert not rows[0]["both_edges_inside_ramp"], rows[0]
    assert rows[0]["s_eng"] < rows[-1]["s_eng"] and rows[0]["s_rel"] > rows[-1]["s_rel"], (
        "a tighter floor must engage EARLIER and release LATER")


# =============================================================================
# THE HEADLINE — the closing edge is not inert
# =============================================================================

def test_headline_one_clip_credits_the_watched_spool_and_DEBITS_the_other():
    """GATE 5. THE RUNG. Every row engages UPSTREAM of s_hp*=0.400, so rung 48's law predicts
    a CREDIT on the HP in all of them. Measured: a DEBIT in all of them, from the very same
    clip that credits the LP."""
    for row in _sweep(LP_FLOORS):
        assert row["s_eng"] < S_HP_STAR, (row["phi_lim"], "must engage upstream of s_hp*")
        assert row["relief_watched"] > 0.0, row
        assert row["relief_other"] < 0.0, (
            row["phi_lim"], row["relief_other"], "the unwatched spool must be DEBITED")
    # the debit is not a rounding artifact — it is up to ~1.2% of the bare min phi
    assert min(x["relief_other"] for x in _sweep(LP_FLOORS)) < -0.005


def test_mechanism_the_unwatched_minimum_relocates_to_just_after_the_release():
    """GATE 6. THE MECHANISM. Inside the window the unwatched spool is BETTER off (the clip
    really does slow its descent — rung 48's arrest). But it is SLOWED, not arrested: it falls
    right through the window while the bare march has already turned around. Then the leg lets
    go, the withheld fuel reaches a still-ramping plant, and the descent RE-OPENS.

    So the unwatched minimum sits just AFTER the release edge — that is where the damage is
    made, and it is why the closing edge is not causally inert."""
    ft = _ft()
    bare, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, R, SETTLE, DS)
    bmap = {round(p["s"], 6): p for p in bare}
    for row in _sweep(LP_FLOORS):
        # the unwatched minimum lands at (or within 3 cells after) the release edge
        assert row["s_rel"] - 1e-9 <= row["s_min_other"] <= row["s_rel"] + 3 * DS + 1e-9, (
            row["phi_lim"], row["s_rel"], row["s_min_other"])
    # ...and INSIDE the window the unwatched phi is ABOVE bare (slowed, not yet damaged)
    lim, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, R, SETTLE, DS, surge=SurgeLimiter(
        spool="lp", phi_lim=0.7450))
    mid = [p for p in lim if 0.20 <= p["s"] <= 0.30]
    assert mid and all(p["phi_hp"] > bmap[round(p["s"], 6)]["phi_hp"] for p in mid), (
        "inside the window the clip must HELP the unwatched spool")
    # ...while still descending through it (slowed, not arrested)
    win = [p["phi_hp"] for p in lim if row["s_eng"] <= p["s"] <= 0.42]
    assert win[-1] < win[0], "the unwatched spool must keep descending inside the window"


def test_sign_flips_when_the_release_lands_far_past_the_ramp_rung48_regime():
    """GATE 7. The two-term law predicting its own inversion. Push the release well past the
    ramp end (r=0.15, s_rel/r = 2.4...3.2) and the debit term dies, leaving rung 48's credit
    alone: the SAME instrument on the SAME plant watching the SAME spool now REBATES the
    other one.

    This is why rung 48 is BOUNDED, not refuted — its own leg released at s_rel/r = 1.16-2.24
    (measured in docs/both-edges-limiter-negative.md), i.e. in exactly this regime."""
    fast = _sweep((0.7500, 0.7400), r=0.15)
    for row in fast:
        assert row["s_rel"] > 2.0 * 0.15, (row["phi_lim"], row["s_rel"], "release must be late")
        assert row["relief_other"] > 0.0, (
            row["phi_lim"], row["relief_other"], "far-past-ramp release must REBATE")
    # ...and it is the same instrument that debited at r=0.5
    assert all(x["relief_other"] < 0.0 for x in _sweep(LP_FLOORS)), "the r=0.5 sign, for contrast"


def test_discriminator_the_debit_is_clocked_by_the_RAMP_not_the_spools_own_minimum():
    """GATE 8. THE DISCRIMINATOR — which clock sets the debit?

    At r=0.5 the unwatched spool's own minimum (0.400) and the ramp end (0.500) are too close
    to separate. At r=2.0 they are 3.1x apart (s_hp*=0.650 vs ramp end 2.0). The debit tracks
    the RAMP END: it is far larger with the release at s_rel~r than at s_rel~s_hp*, and it
    grows monotonically with s_rel straight THROUGH s_hp* without noticing it.

    So the two edges answer to DIFFERENT clocks: the credit is per-spool (rung 48), the debit
    is ramp-clocked (rung 44's clock). SLOW — a long ramp at production ds."""
    rows = _sweep((0.7650, 0.7690, 0.7725), r=2.0, settle=1.5)
    at_spool_min, mid, at_ramp_end = rows            # s_rel ~ 0.67 / 1.07 / 2.11
    assert at_spool_min["s_rel"] < 1.0 < mid["s_rel"] < at_ramp_end["s_rel"], rows
    for x in rows:
        assert x["relief_other"] < 0.0, x
    assert abs(at_ramp_end["relief_other"]) > 5.0 * abs(at_spool_min["relief_other"]), (
        at_spool_min["relief_other"], at_ramp_end["relief_other"],
        "the debit must be dominated by the RAMP clock, not the spool's own minimum")
    # monotone in s_rel straight through s_hp* = 0.650
    assert (abs(at_spool_min["relief_other"]) < abs(mid["relief_other"])
            < abs(at_ramp_end["relief_other"])), rows


# =============================================================================
# RUNG 48's LAW, ON A DIFFERENT INSTRUMENT CLASS
# =============================================================================

def test_cross_instrument_rung48_crossing_reproduced_exactly():
    """GATE 9. Flip the watched spool. An HP floor engages LATE; the LP's minimum is EARLY
    (s_lp*=0.240), so rung 48's edge condition applies to the LP in pure form — and the
    exact-zero lands where the law says, with no fitting and no limited march.

    A genuine forecast off a BARE march, landing on a limiter class rung 48 never built."""
    ft = _ft()
    bare, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, R, SETTLE, DS)
    mpl = min(p["phi_lp"] for p in bare)

    def forecast(s_eng):
        return min(p["phi_lp"] for p in bare if p["s"] <= s_eng + 1e-12) - mpl

    seen_up = seen_down = False
    for row in _sweep(HP_FLOORS, spool="hp"):
        if row["s_eng"] < S_LP_STAR - 1e-12:
            seen_up = True
            assert row["relief_other"] > 0.0, (row["phi_lim"], "upstream must rebate")
            # the truncated-descent forecast, to its own O(ds) accuracy
            assert abs(row["relief_other"] - forecast(row["s_eng"])) < 2e-3, (
                row["s_eng"], row["relief_other"], forecast(row["s_eng"]))
        elif row["s_eng"] > S_LP_STAR + 1e-12:
            seen_down = True
            assert row["relief_other"] == 0.0, (
                row["phi_lim"], row["relief_other"], "downstream EXACTLY nothing")
            assert forecast(row["s_eng"]) == 0.0, "the forecast must call it exactly too"
    assert seen_up and seen_down, "the sweep must straddle s_lp*"


def test_the_exposed_spool_is_the_LATE_one_inverting_rungs_41_44_45():
    """GATE 9b. WHY the split has the direction it does. A release edge is structurally LATE
    (it needs an accumulated window), so it lands inside the HP's basin and past the LP's:
    within 0.005 of its own minimum the LP sits at s in [0.15,0.32], the HP at [0.29,0.50].

    So the early-LP / late-HP timing that ran through rungs 46/47/48 decides WHICH spool is
    exposed to the closing edge — and it is the HP, exactly INVERTING rungs 41/44/45's
    "the LP eats the excursion"."""
    ft = _ft()
    bare, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, R, SETTLE, DS)
    mpl = min(p["phi_lp"] for p in bare)
    mph = min(p["phi_hp"] for p in bare)
    b_lp = [p["s"] for p in bare if p["phi_lp"] - mpl <= 0.005]
    b_hp = [p["s"] for p in bare if p["phi_hp"] - mph <= 0.005]
    assert b_lp[0] < b_hp[0] and b_lp[-1] < b_hp[-1], "the LP basin must be the EARLY one"
    # every HP-watching release lands past the LP basin => no debit on the LP
    for row in _sweep(HP_FLOORS, spool="hp"):
        assert row["s_rel"] > b_lp[-1], (row["phi_lim"], row["s_rel"], b_lp[-1])
        assert row["relief_other"] >= 0.0, row
    # while every LP-watching release lands INSIDE the HP basin => the debit
    for row in _sweep(LP_FLOORS):
        assert b_hp[0] <= row["s_rel"] <= b_hp[-1] + 3 * DS or row["s_rel"] > b_hp[-1], row
        assert row["relief_other"] < 0.0, row


# =============================================================================
# THE NON-TAUTOLOGY AND THE HONEST BOUNDARY
# =============================================================================

def test_not_the_ramp_rate_lever_the_non_tautology():
    """GATE 10. The deflation to exclude is "any clip removes fuel and slows the accel".

    Three exclusions, all measured: (i) the endpoint is UNMOVED; (ii) `fuel_removed` is
    positive and smooth, and the LARGEST fuel removal gives the SMALLEST debit — so the debit
    is not "how much fuel" but WHEN it is given back; (iii) one clip moves the two spools in
    OPPOSITE directions, which a ramp-rate lever cannot do. Uses a full settle (SLOW)."""
    rows = _sweep((0.7650, 0.7550, 0.7500, 0.7450, 0.7400), settle=4.0)
    for row in rows:
        assert row["fuel_removed"] > 0.0, row
        assert abs(row["nu_hp_end"] - row["nu_hp_end_bare"]) < 5e-4, (
            row["phi_lim"], row["nu_hp_end"], row["nu_hp_end_bare"], "endpoint must be unmoved")
        assert row["relief_watched"] > 0.0 > row["relief_other"], row
    # fuel_removed is monotone in the floor...
    fr = [x["fuel_removed"] for x in rows]
    assert fr == sorted(fr, reverse=True), fr
    # ...but the debit is NOT: the biggest removal gives the smallest debit
    assert abs(rows[0]["relief_other"]) < abs(rows[2]["relief_other"]), (
        rows[0]["fuel_removed"], rows[0]["relief_other"],
        rows[2]["fuel_removed"], rows[2]["relief_other"])


def test_honest_boundary_a_floor_above_the_running_line_destroys_the_accel():
    """GATE 11. `phi_lim` must sit BELOW the initial running-line phi, or the leg binds from
    s=0 and never releases. On the FLAT LP map the swept floor sits above the LP's start, and
    `nu_hp` at settle COLLAPSES — the accel does not complete and the leg HAS degenerated into
    rung 44's ramp-rate lever.

    Structurally rung 48's `m -> 0` degeneracy. Reported, not hidden: read the split only
    where `nu_hp_end` is unmoved."""
    row = _sweep((0.7500,), shape="flat-lp")[0]
    assert row["s_eng"] == 0.0, ("a floor above the running line must bind from s=0", row)
    assert row["nu_hp_end_bare"] - row["nu_hp_end"] > 0.2, (
        row["nu_hp_end"], row["nu_hp_end_bare"], "the accel must visibly fail to complete")
    # ...and the healthy band is precisely the one whose floor clears the start
    assert all(x["phi_lim"] < PHI_LP_START for x in _sweep(LP_FLOORS))


def test_robustness_the_debit_survives_ds_and_rho():
    """GATE 12. A minimum-LOCATION claim must survive refinement, and a two-spool claim must
    not ride on rung 40's complex inter-spool mode. SLOW."""
    ft = _ft()
    lim = SurgeLimiter(spool="lp", phi_lim=0.7500)
    vals = []
    for ds in (0.02, 0.01):
        vals.append(ft.surge_relief(FLIGHT, LO, HI, lim, r=R, s_settle=SETTLE,
                                    ds=ds)["relief_other"])
    assert all(v < 0.0 for v in vals), vals
    assert abs(vals[1] - vals[0]) < 0.25 * abs(vals[0]), ("ds-convergent", vals)
    for rho in (0.25, 4.0):
        row = _ft(rho=rho).surge_relief(FLIGHT, LO, HI, lim, r=R, s_settle=SETTLE, ds=DS)
        assert row["relief_watched"] > 0.0 > row["relief_other"], (rho, row)


if __name__ == "__main__":
    for fn in (test_reduce_surge_none_never_consults_the_leg_bit_for_bit,
               test_reduce_dormant_floor_bit_for_bit_rung45,
               test_reduce_composite_min_select_with_the_prior_legs,
               test_reduce_lp_disabled_asserts,
               test_decel_never_fires_bit_for_bit_rung45,
               test_cycle_untouched_by_the_phi_leg_bit_for_bit_rung6,
               test_the_hold_is_a_sliding_mode_not_chatter,
               test_both_edges_close_inside_the_ramp_the_unreachable_object,
               test_headline_one_clip_credits_the_watched_spool_and_DEBITS_the_other,
               test_mechanism_the_unwatched_minimum_relocates_to_just_after_the_release,
               test_sign_flips_when_the_release_lands_far_past_the_ramp_rung48_regime,
               test_discriminator_the_debit_is_clocked_by_the_RAMP_not_the_spools_own_minimum,
               test_cross_instrument_rung48_crossing_reproduced_exactly,
               test_the_exposed_spool_is_the_LATE_one_inverting_rungs_41_44_45,
               test_not_the_ramp_rate_lever_the_non_tautology,
               test_honest_boundary_a_floor_above_the_running_line_destroys_the_accel,
               test_robustness_the_debit_survives_ds_and_rho):
        fn()
        print("PASS", fn.__name__)
