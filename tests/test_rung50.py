"""Rung 50 — THE RELEASE EDGE, ISOLATED: the closing edge relocates BOTH spools' minima to
itself, and a limiter's immunity is TIMING, not clip SHAPE.

Rung 49 found that a limiter acts through BOTH edges and that they answer to different clocks,
but it could only move the release edge by moving phi_lim — which drags the engagement edge, the
window length AND the clip depth with it. So it hedged, correctly, as a WITHIN-FAMILY result,
and it left an open seam: "WHY rung 48's leg is immune to the release debit ... the clip SHAPE
is the obvious suspect, but it is NOT measured here."

This rung builds the instrument that decides both: a FORCED release time `s_off`, which slides
the closing edge ALONE, TWO-SIDED, with everything up to it bit-identical. It is an ISOLATION
DIAGNOSTIC in the project's own tradition (rung 34/40's freeze='lp' holds a spool's speed
against its own ODE), not a control law, and it is named as one.

THE HEADLINE: the release edge RELOCATES BOTH SPOOLS' MINIMA TO ITSELF — watched and unwatched,
both instrument families, grid-independently — whenever it lands at or after that spool's own
bare minimum. Three consequences: (1) the debit is RAMP-clocked, 2.75x larger at the ramp end
than at the unwatched spool's own minimum, so rung 49's within-family hedge LIFTS; (2) a limiter
forced to release early DEBITS THE SPOOL IT WATCHES, bounding rung 49's watched-side identity to
the unforced instrument; (3) THE SEAM CLOSES — rung 48's own leg, forced to release inside the
ramp, debits both spools exactly like the phi floor, so its immunity is TIMING, not SHAPE, and
rung 49 § 4's "the magnitude explanation does not transfer" was itself confounded (at FIXED
release time the debit is monotone in the deficit ACROSS both families).

Reduces: s_off=None never applies the gate (bit-for-bit rungs 43/45/46/47/48/49); s_off past the
natural release is float-for-float the unforced leg; s_off <= s_eng is float-for-float bare;
s_off without an armed leg ASSERTS; lp_disabled ASSERTS; the design run is bit-for-bit rung 6.
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

LO, HI, R, SETTLE, DS = 1000.0, 1400.0, 0.5, 2.0, 0.02
REDLINE = 1480.0
KEYS = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "mf")

# The bare march's raw surge minima at this config (rung 49's, verified in
# docs/plans/rung50-anchor-release-edge.md).
S_LP_STAR, S_HP_STAR = 0.240, 0.400                    # r = 0.5
S_LP_STAR_2, S_HP_STAR_2 = 0.320, 0.640                # r = 2.0
PHI_LIM = 0.7450                                       # the r=0.5 working floor
PHI_LIM_2 = 0.7725                                     # the r=2.0 floor (natural s_rel = 2.10)


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


def _sweep(s_offs, phi_lim=PHI_LIM, margin=None, r=R, settle=SETTLE, ds=DS, rho=1.0,
           spool="lp"):
    """Memoized within a worker — several gates read ONE sweep (each still asserts its own
    claim; the sweep is the shared, expensive measurement)."""
    key = (s_offs, phi_lim, margin, r, settle, ds, rho, spool)
    if key not in _SWEEPS:
        ft = _ft(rho=rho)
        surge = SurgeLimiter(spool=spool, phi_lim=phi_lim) if phi_lim is not None else None
        accel = ft.accel_schedule(FLIGHT, LO, HI, margin) if margin is not None else None
        _SWEEPS[key] = ft.release_sweep(FLIGHT, LO, HI, s_offs, surge=surge, accel=accel,
                                        r=r, s_settle=settle, ds=ds)
    return _SWEEPS[key]


# =============================================================================
# THE REDUCE SPINE
# =============================================================================

def test_reduce_s_off_none_never_gates_the_legs_bit_for_bit():
    """CONTRACT 1. `s_off=None` leaves rungs 43/45/46/47/48/49 bit-for-bit. Guaranteed at CODE
    level (`armed` short-circuits on `s_off is None`), which is what this gate witnesses: the
    five prior marches are reproduced through the NEW signature, byte-identically."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    acc = ft.accel_schedule(FLIGHT, LO, HI, 0.25)
    lim = SurgeLimiter(spool="lp", phi_lim=0.7500)
    end = R + 1.0
    for kw in (dict(), dict(Tt4_max=REDLINE), dict(Tt4_max=REDLINE, tau_gov=0.2),
               dict(accel=acc), dict(surge=lim), dict(accel=acc, surge=lim),
               dict(Tt4_max=REDLINE, tau_gov=0.2, accel=acc, surge=lim)):
        a = ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, **kw)
        b = ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, s_off=None, **kw)
        _same(a, b)
    # ... and the gate is not vacuous: the armed legs genuinely clip.
    assert any(p["mf"] < p["mf_sched"]
               for p in ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, surge=lim))


def test_reduce_release_relief_none_is_rung49_surge_relief_bit_for_bit():
    """CONTRACT 1b. The rung-50 FINDING method at `s_off=None` IS rung 49's finding method —
    the same two marches, the same reference-free surge object, float-for-float."""
    ft = _ft()
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    a = ft.release_relief(FLIGHT, LO, HI, None, surge=lim, r=R, s_settle=SETTLE, ds=DS)
    b = ft.surge_relief(FLIGHT, LO, HI, lim, r=R, s_settle=SETTLE, ds=DS)
    for k in ("s_eng", "s_rel", "relief_lp", "relief_hp", "fuel_removed", "nu_hp_end",
              "min_phi_lp_bare", "min_phi_hp_bare", "min_phi_lp_lim", "min_phi_hp_lim"):
        assert a[k] == b[k], (k, a[k], b[k])


def test_reduce_late_s_off_is_inert_and_early_s_off_is_bare_bit_for_bit():
    """CONTRACT 2/3. Forcing a release the leg would have made anyway is INERT (float-for-float
    the unforced leg); forcing one BEFORE the leg ever engages leaves the march float-for-float
    BARE. The two ends of the sweep are exact, not approximate."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    end = R + SETTLE
    bare = ft.integrate_fuel(FLIGHT, sched, nu0, end, DS)
    free = ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, surge=lim)
    # natural window at this floor is [0.12, 0.44]
    _same(ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, surge=lim, s_off=1.50), free)
    _same(ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, surge=lim, s_off=0.10), bare)
    assert any(p["mf"] < p["mf_sched"] for p in free)      # not vacuous


def test_reduce_s_off_without_an_armed_leg_asserts():
    """CONTRACT 4. `s_off` forces a min-select LEG to release — with none armed it is
    meaningless, and the rung-46/47 governor is deliberately out of scope."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    with pytest.raises(AssertionError):
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 0.5, DS, s_off=0.30)
    with pytest.raises(AssertionError):        # a redline is NOT an armed min-select leg
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 0.5, DS, Tt4_max=REDLINE, s_off=0.30)
    with pytest.raises(AssertionError):        # release_relief needs a leg too
        ft.release_relief(FLIGHT, LO, HI, 0.30, r=R, s_settle=SETTLE, ds=DS)


def test_reduce_lp_disabled_asserts():
    """CONTRACT 5. The finding is inherently two-shaft (BOTH spools' minima relocate), so
    lp_disabled is not a reduce axis for it — same contract as rungs 46/47/48/49."""
    ft = _ft(lp_disabled=True)
    sched, nu0 = _ramp(_ft())        # the ramp comes off the two-shaft plant (rung 49's move)
    with pytest.raises(AssertionError, match="inherently two-shaft"):
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 0.5, DS,
                          surge=SurgeLimiter(spool="lp", phi_lim=0.75), s_off=0.30)


def test_cycle_untouched_by_the_forced_release_bit_for_bit_rung6():
    """CONTRACT 6. The rung-50 diagnostic is a separate entry point: the design-point run is
    bit-for-bit rung 6 across it."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    ft = _ft()
    ft.release_relief(FLIGHT, LO, HI, 0.30, surge=SurgeLimiter(spool="lp", phi_lim=PHI_LIM),
                      r=R, s_settle=SETTLE, ds=DS)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


# =============================================================================
# THE FINDINGS
# =============================================================================

R2_OFFS = (0.30, 0.66, 1.10, 1.56, 1.80, 2.06, 2.20)


def test_headline_the_release_edge_relocates_BOTH_minima_to_itself():
    """GATE 3 — THE HEADLINE. Whenever the DIVE BRANCH WINS on a spool, that spool's argmin phi
    sits AT the release point — for the WATCHED spool and the UNWATCHED one alike. Rung 49 saw
    only half of this (it measured the UNWATCHED minimum landing just after s_rel; the watched
    one was invisible because an LP floor's natural release always lands past the LP basin).

    "The dive branch wins" is the conjunction of two measurable preconditions, and BOTH bite in
    this sweep — which is what makes the gate the two-branch law of § 6 and not a slogan:

      (a) the release lands at or after that spool's OWN bare minimum. Released upstream of it
          the re-opened dive merges into the still-ongoing bare descent and bottoms in the bare
          basin instead (s_off=0.30: s_rel=0.28 < s_hp*=0.64, and s@min_hp is 0.56, not 0.28);
      (b) the dive actually beats rung 48's truncation branch, i.e. that spool's relief is
          NEGATIVE. Where the credit branch wins the minimum sits back at the arrest instead
          (s_off=2.20: relief_lp is +0.012 and s@min_lp is 1.60, far from the release).

    The anchor is `s_rel` (the last engaged point), not `s_off`: past the natural release the
    forcing is inert and the leg lets go on its own."""
    rows = _sweep(R2_OFFS, phi_lim=PHI_LIM_2, r=2.0)
    tol = 3 * DS
    hits = 0
    for x in rows:
        for key, star, rel in (("s_min_lp", S_LP_STAR_2, "relief_lp"),
                               ("s_min_hp", S_HP_STAR_2, "relief_hp")):
            if x["s_rel"] >= star and x[rel] < 0.0:
                assert abs(x[key] - x["s_rel"]) <= tol, (
                    x["s_off"], key, x[key], x["s_rel"], "must relocate TO the release point")
                hits += 1
    assert hits >= 8, (hits, "the gate must not be vacuous")
    # precondition (a) bites — released upstream of s_hp*, the HP keeps its bare basin
    early = rows[0]
    assert early["s_rel"] < S_HP_STAR_2 and early["relief_hp"] < 0.0 \
        and early["s_min_hp"] > early["s_rel"] + tol, early
    # precondition (b) bites — where the CREDIT branch wins, the minimum is NOT at the release
    late = rows[-1]
    assert late["relief_lp"] > 0.0 and late["s_min_lp"] < late["s_rel"] - tol, late


def test_discriminator_the_debit_is_RAMP_clocked_deconfounded():
    """GATE 4 — THE DISCRIMINATOR, on an axis that moves ONLY the release edge. rung 49 § 3
    measured this ordering by sweeping phi_lim, which drags s_eng and the clip depth along, and
    hedged it as WITHIN-FAMILY. Here s_eng is IDENTICAL in every row, so the hedge LIFTS.

    The debit deepens monotonically as the release walks THROUGH the unwatched spool's own
    minimum without noticing it, and peaks with the release just inside the RAMP END."""
    rows = _sweep(R2_OFFS, phi_lim=PHI_LIM_2, r=2.0)
    assert len({round(x["s_eng"], 6) for x in rows}) == 1, (
        "the engagement edge must be FIXED — that is the whole point of the axis")
    # monotone deepening straight THROUGH s_hp* = 0.640 without noticing it
    upto = [x for x in rows if x["s_off"] <= 1.10]
    mags = [-x["relief_hp"] for x in upto]
    assert all(b > a for a, b in zip(mags, mags[1:])), mags
    assert any(x["s_off"] < S_HP_STAR_2 for x in upto) and \
           any(x["s_off"] > S_HP_STAR_2 for x in upto), "must BRACKET s_hp*"
    at_star = min(rows, key=lambda x: abs(x["s_off"] - S_HP_STAR_2))
    peak = max(rows, key=lambda x: -x["relief_hp"])
    assert -peak["relief_hp"] > 2.5 * -at_star["relief_hp"], (at_star, peak)
    # the peak sits near the RAMP END, not at the unwatched spool's own minimum
    assert peak["s_off"] > 2.0 * S_HP_STAR_2, (peak["s_off"], "the peak is NOT at s_hp*")
    assert 0.6 * 2.0 <= peak["s_off"] <= 2.0, (peak["s_off"], "the peak is near the RAMP END")
    # ... and it collapses once the release goes past the ramp end
    past = [x for x in rows if x["s_off"] > 2.0]
    assert past and -past[-1]["relief_hp"] < 0.6 * -peak["relief_hp"], (peak, past)


def test_the_watched_spool_is_DEBITED_when_released_early_rung49_bounded():
    """GATE 5. Rung 49's gate 3 asserts relief_watched == phi_lim - min phi_bare identically and
    calls it definitional. It is — UNDER THE UNFORCED INSTRUMENT. Force the release early and it
    fails in the only direction that matters: the limiter leaves the spool it is PROTECTING
    worse off than no limiter at all.

    Rung 49 is BOUNDED, not corrected: as s_off runs past the natural release the identity comes
    straight back. Its "the exposed spool is the LATE one" is a statement about where the natural
    release LANDS, not about the spools."""
    rows = _sweep((0.16, 0.20, 0.26, 0.30, 0.36, 0.44, 0.60), phi_lim=PHI_LIM, r=R)
    assert any(x["relief_lp"] < 0.0 for x in rows), (
        [(x["s_off"], x["relief_lp"]) for x in rows],
        "an early release must DEBIT the watched spool")
    worst = min(rows, key=lambda x: x["relief_lp"])
    assert worst["s_off"] < S_HP_STAR, worst          # the damage is done EARLY
    # rung 49 recovered at the far end (its unforced instrument)
    free = rows[-1]
    assert free["relief_lp"] > 0.0, free
    assert abs(free["relief_lp"] - (PHI_LIM - free["min_phi_lp_bare"])) < 1e-5, free


def test_SEAM_rung48s_immunity_is_TIMING_not_clip_SHAPE():
    """GATE 6 — THE SEAM CLOSES. Rung 49's standing OPEN seam: "WHY rung 48's leg is immune to
    the release debit is an OPEN SEAM ... the clip SHAPE is the obvious suspect, but it is NOT
    measured here."

    Measured: rung 48's OWN leg, clip shape unchanged, forced to release inside the ramp DEBITS
    BOTH spools — with the same relocation signature as the phi floor. Left alone (natural
    release post-ramp) it delivers its rung-48 CREDIT. The immunity is TIMING."""
    rows = _sweep((0.30, 0.44, 0.50, 9.90), phi_lim=None, margin=0.25, r=R)
    forced = [x for x in rows if x["s_off"] < R + 1e-9]
    free = rows[-1]
    assert free["s_rel"] > R, (free["s_rel"], "rung 48's natural release must be POST-ramp")
    assert free["relief_lp"] > 0.0 and free["relief_hp"] > 0.0, (
        free, "unforced, rung 48's leg CREDITS both spools")
    for x in forced:
        assert x["relief_lp"] < 0.0 and x["relief_hp"] < 0.0, (
            x["s_off"], x["relief_lp"], x["relief_hp"],
            "forced inside the ramp, the SAME leg debits BOTH spools")


def test_SEAM_cross_regime_at_r2_and_rung48s_exact_zero_survives():
    """GATE 7. The seam closure OUT of rung 49's own s_hp*-vs-r confound (at r=0.5 those sit
    2.5 cells apart). At r=2.0, m=0.15 (the corrected band floor — m=0.25 never engages on so
    slow a ramp) the same inversion holds.

    And rung 48's EXACT ZERO survives the forcing untouched: s_eng=0.360 is downstream of
    s_lp*=0.320, and every release here lands past the LP basin, so relief_lp is exactly 0.0 in
    every row. The two laws coexist rather than compete."""
    rows = _sweep((0.66, 1.10, 1.80, 9.90), phi_lim=None, margin=0.15, r=2.0)
    assert all(x["relief_lp"] == 0.0 for x in rows), (
        [(x["s_off"], x["relief_lp"]) for x in rows], "rung 48's exact zero must SURVIVE")
    inside = [x for x in rows if x["s_off"] <= 1.10]
    assert all(x["relief_hp"] < 0.0 for x in inside), inside
    assert rows[-1]["relief_hp"] > 0.0, rows[-1]         # unforced => rung 48's credit


def test_the_deficit_factor_at_FIXED_release_rung49_section4_corrected():
    """GATE 8. Rung 49 § 4 refuted hand-back MAGNITUDE as the explanation, measuring it
    ANTI-correlated — but it swept magnitude and timing TOGETHER. Hold the release time fixed
    and the sign reverses: the debit is MONOTONE INCREASING in the deficit, and it is monotone
    ACROSS INSTRUMENT FAMILIES (two phi floors + rung 48's schedule, all released at the same
    s_rel). Rung 48's clip is not gentler per unit deficit — it is WORSE.

    Claimed as MONOTONE only: the functional form is measured, not derived (rung 49's own
    concession, carried)."""
    ft = _ft()
    acc = ft.accel_schedule(FLIGHT, LO, HI, 0.25)
    legs = [("phi 0.7450", dict(surge=SurgeLimiter(spool="lp", phi_lim=0.7450))),
            ("phi 0.7500", dict(surge=SurgeLimiter(spool="lp", phi_lim=0.7500))),
            ("rung48 m=0.25", dict(accel=acc))]
    out = [(n, ft.release_relief(FLIGHT, LO, HI, 0.44, r=R, s_settle=SETTLE, ds=DS, **kw))
           for n, kw in legs]
    # the release time is genuinely MATCHED across the three (that is the deconfounding)
    assert len({round(x["s_rel"], 6) for _, x in out}) == 1, [(n, x["s_rel"]) for n, x in out]
    rm = [x["fuel_removed"] for _, x in out]
    db = [-x["relief_hp"] for _, x in out]
    assert all(b > a for a, b in zip(rm, rm[1:])), rm     # deficits genuinely ordered
    assert all(b > a for a, b in zip(db, db[1:])), (rm, db,
                                                    "debit must ORDER WITH the deficit")


def test_not_the_ramp_rate_lever_the_non_tautology():
    """GATE 9. The deflation to exclude is "any clip removes fuel and slows the accel". Two
    measured exclusions:

      * fuel removal is MONOTONE in s_off while the debit is PEAKED — the largest removal is
        NOT the largest debit (19% more fuel removed for less than half the debit);
      * the endpoint is unmoved at rung 49's gate-10 settle."""
    rows = _sweep(R2_OFFS, phi_lim=PHI_LIM_2, r=2.0, settle=4.0)
    rm = [x["fuel_removed"] for x in rows]
    assert all(b > a for a, b in zip(rm, rm[1:])), rm
    peak = max(rows, key=lambda x: -x["relief_hp"])
    last = rows[-1]
    assert last["fuel_removed"] > peak["fuel_removed"], (peak, last)
    assert -last["relief_hp"] < 0.6 * -peak["relief_hp"], (
        peak, last, "MORE fuel removed must give a SMALLER debit")
    for x in rows:
        assert abs(x["nu_hp_end"] - x["nu_hp_end_bare"]) < 5e-4, (
            x["s_off"], x["nu_hp_end"], x["nu_hp_end_bare"], "the endpoint must be UNMOVED")


def test_robustness_ds_convergence_of_the_relocation_and_the_debit():
    """GATE 10. The headline is a statement about WHERE a minimum sits, so it is the most
    grid-prone claim in the set. Measured at ds in {0.02, 0.01} with s_off ON the grid: the
    relocation offset is 0.000 at BOTH, and the depth converges to a few per cent (far tighter
    than rung 49's ~13% gate-12 drift, because a FORCED dive is anchored to an imposed s_off
    rather than to a solved edge).

    Checked at BOTH ramp rates, and deliberately including r=2.0 — those dives are ~8x deeper
    than the r=0.5 ones and are where a grid artifact would most plausibly hide. They converge
    BETTER (0.7% against 2%), not worse."""
    for offs, phi, r_, tol in (((0.30, 0.40, 0.44), PHI_LIM, R, 0.05),
                               ((1.10, 1.56), PHI_LIM_2, 2.0, 0.02)):
        a = _sweep(offs, phi_lim=phi, r=r_, ds=0.02)
        b = _sweep(offs, phi_lim=phi, r=r_, ds=0.01)
        for x, y in zip(a, b):
            assert abs(x["s_min_lp"] - x["s_off"]) <= 0.02 and \
                   abs(y["s_min_lp"] - y["s_off"]) <= 0.01, (r_, x["s_off"], x["s_min_lp"],
                                                             y["s_min_lp"])
            assert abs(y["relief_hp"] - x["relief_hp"]) < tol * abs(x["relief_hp"]), (
                r_, x["s_off"], x["relief_hp"], y["relief_hp"])


def test_robustness_the_split_survives_rho():
    """GATE 10b. rho = tau_L/tau_H is rung 40's one parameter. The early-release debit on the
    WATCHED spool — the new sign — survives it."""
    for rho in (0.25, 4.0):
        rows = _sweep((0.26, 0.30, 0.36), phi_lim=PHI_LIM, r=R, rho=rho)
        assert any(x["relief_lp"] < 0.0 for x in rows), (
            rho, [(x["s_off"], x["relief_lp"]) for x in rows])


if __name__ == "__main__":
    for fn in (test_reduce_s_off_none_never_gates_the_legs_bit_for_bit,
               test_reduce_release_relief_none_is_rung49_surge_relief_bit_for_bit,
               test_reduce_late_s_off_is_inert_and_early_s_off_is_bare_bit_for_bit,
               test_reduce_s_off_without_an_armed_leg_asserts,
               test_reduce_lp_disabled_asserts,
               test_cycle_untouched_by_the_forced_release_bit_for_bit_rung6,
               test_headline_the_release_edge_relocates_BOTH_minima_to_itself,
               test_discriminator_the_debit_is_RAMP_clocked_deconfounded,
               test_the_watched_spool_is_DEBITED_when_released_early_rung49_bounded,
               test_SEAM_rung48s_immunity_is_TIMING_not_clip_SHAPE,
               test_SEAM_cross_regime_at_r2_and_rung48s_exact_zero_survives,
               test_the_deficit_factor_at_FIXED_release_rung49_section4_corrected,
               test_not_the_ramp_rate_lever_the_non_tautology,
               test_robustness_ds_convergence_of_the_relocation_and_the_debit,
               test_robustness_the_split_survives_rho):
        fn()
        print("PASS", fn.__name__)
