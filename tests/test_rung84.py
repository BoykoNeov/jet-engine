"""Rung 84 — THE MARCHED MINIMUM'S STAIRCASE: `docs/rung83-spec.md` § 8's first seam.

Rung 83 § 8 asked WHICH RAMPS HAVE ROOTS AND AT WHICH `ds`, calling the `(r, ds)` map the thing
that decides *"whether rung 82's five-row table contains one discontinuity or several"*. The map is
here — and the mechanism under it turns the question over.

HEADLINE: **a minimum over a MARCHED set is not a minimum at all — it is an evaluation on a moving
GRID BOUNDARY, so the residual is a STAIRCASE whose rise and tread both scale with `ds`.** A `min`
over a FIXED finite set of continuous functions is continuous; it kinks at a handover and cannot
jump. Rung 83's jump is the four-loop window opening one march step earlier, and the entering point
binds immediately because the argmin IS the window's leading point.

CROSS-RUNG: rung 83's `argmin_moved` fired CORRECTLY and reported a CONSEQUENCE — an edge move
forces an argmin move, never the reverse. Verdict CONFIRMED, reason CORRECTED (rung 28's shape).

Scoring + tables: `docs/rung84-spec.md`; pre-registration: `docs/plans/rung84-anchor-staircase-law.md`.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    StaircaseLawTransient, ThresholdLawTransient, BleedLimiter, StatorLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, TT4_MAX = 1000.0, 1400.0, 1200.0
B, V_MAX, TAU, TAU_S = 0.10, 0.20, 0.05, 0.05
PHI_FUEL, PHI_AIR = 0.75, 0.77          # rung 80's walls, unchanged
DS = 0.005                              # rung 82's shipped march step
BRACKET = (0.004, 0.30)                 # rung 82's own

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


def _rig(design, cls=StaircaseLawTransient):
    sm = 0.80 / FLOOR - 1.0
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_lim=StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m


def _kw(r, ds=DS):
    return dict(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=PHI_FUEL,
                phi_air=PHI_AIR, tau_gov=0.05, tau_q=0.05, tau_s=0.05, s_settle=1.2,
                ds=ds, v_max=0.20, inc=False, r=r)


# --- THE REDUCE CONTRACT: an IDENTITY, because this rung adds nothing --------------------------

@pytest.mark.slow
def test_reduce_the_reader_is_bit_for_bit_rung_82s_scan(design):
    """Rung 84 adds no state, no knob and no constant, so its residual must be rung 82's EXACTLY.

    `edge_read` reaches the march through `_scan_cells` — `_threshold_scan`'s own body, extracted
    so this rung can see the point IDENTITIES that method reduces to counts. If the extraction
    moved anything, `h` moves, and `h` is what rungs 82 and 83 solve. Not "close": the same float.

    `s_bind` is compared through the same 9-place rounding `edge_read` keys its summands with —
    `_threshold_scan` returns the raw `s`, and asserting bit-equality between a rounded key and an
    unrounded float would be testing the rounding, not the plant."""
    m84, m82 = _rig(design), _rig(design, ThresholdLawTransient)
    for r in (0.25, 0.35):
        for tau in (0.03, 0.08):
            a = m84.edge_read(tau, **_kw(r))
            b = m82._scan(tau, **_kw(r))
            assert a["h"] == b["h"], (
                "rung 84 moved the residual at r=%g tau=%g: %r vs rung 82's %r"
                % (r, tau, a["h"], b["h"]))
            assert a["kappa_pure"] == b["kappa_pure"]
            assert a["kappa"] == (b["kappa"][0] if b["kappa_pure"] else None)
            assert a["n_scored"] + a["n_slope_excluded"] == b["n_scored"], (
                "the scored set and the slope exclusions no longer partition rung 82's cells")
            assert a["n_ride"] == b["n_riding4"]
            assert a["s_bind"] == round(b["s_bind"], 9), (
                "the binding point moved at r=%g tau=%g: %r vs %r"
                % (r, tau, a["s_bind"], b["s_bind"]))


# --- § 1.1 THE BOUNDARY READING: rung 82's `min` never binds away from the window's edge -------

@pytest.mark.slow
def test_p1_the_minimum_is_attained_at_the_windows_leading_point(design):
    """P1, measured 71/71 — and it is the premise every later section rests on.

    If the argmin is ALWAYS the window's first riding point, then `F(tau) = hat(s_edge(tau))` and
    the `min` rung 82 writes it as is decorative: the object rungs 82 and 83 have been solving is
    an evaluation on a moving discrete BOUNDARY. That is what makes § 1.2's `ceil` picture apply,
    and it is why an edge move and an argmin move are the same event (P4).

    The sample points straddle each ramp's own root (rung 83 § 4's table) so BOTH branches are
    covered rather than one side five times."""
    m = _rig(design)
    roots = {0.20: 0.01108, 0.25: 0.01975, 0.35: 0.03710, 0.50: 0.06109, 0.70: 0.09202}
    n_ok = n = 0
    for r, root in roots.items():
        for f in (0.7, 1.4):
            rd = m.edge_read(round(root * f, 7), **_kw(r))
            assert rd["window_open"], "V1: no four-loop window at r=%g" % r
            assert rd["edge_on_grid"], (
                "V3: the window edge %r is not a multiple of ds=%g at r=%g — § 1.2's `ceil` "
                "picture does not apply and no count may be quoted" % (rd["edge"], DS, r))
            n += 1
            n_ok += int(bool(rd["at_edge"]))
    assert n_ok == n, (
        "P1: the minimum bound away from the window's leading point at %d of %d points. The "
        "residual is then a genuine minimum and § 1.1's boundary reading is wrong." % (n - n_ok, n))


# --- § 2 THE CLASSIFIER IS EXACT — no threshold, in either direction ---------------------------

@pytest.mark.slow
def test_p2_the_membership_term_is_exactly_zero_when_the_set_is_unchanged(design):
    """P2, and the bar is `== 0.0` — an IDENTITY, not a tolerance.

    A minimum over a FIXED finite set of continuous functions is continuous, so on the points two
    marches SHARE the difference is the whole story and the membership term must vanish EXACTLY.
    This is the instrument upgrade over rung 83 § 3, whose classifier was a ratio its own
    docstring had to call "reported and never thresholded into a verdict here".

    BOTH DIRECTIONS ARE GATED. A test that only checked the zero would pass on a reader that
    always returned zero — rung 78's vacuity trap. So the jump's term is asserted NON-zero and
    dominant on the same rig, at rung 83 § 3.2's own two tau values."""
    m = _rig(design)
    # § 3.4's CROSSING — the sets agree, so the term is structurally zero
    a = m.edge_read(0.037000, **_kw(0.35))
    b = m.edge_read(0.037333, **_kw(0.35))
    c = m.classify(a, b)
    assert not c["set_changed"] and c["entered"] == [] and c["left"] == []
    assert c["d_membership"] == 0.0, (
        "P2: the sets are equal yet the membership term is %r — a min over a FIXED set is "
        "continuous, so this cannot be nonzero" % c["d_membership"])
    assert c["kind"] == "crossing" and c["sign_change"] and c["sign_change_common"]

    # § 3.2's JUMP — one point ENTERS, and it carries the sign change by itself
    a = m.edge_read(0.0197750, **_kw(0.25))
    b = m.edge_read(0.0197875, **_kw(0.25))
    c = m.classify(a, b)
    assert c["set_changed"] and c["left"] == [] and len(c["entered"]) == 1, (
        "the jump is supposed to be ONE point entering; got entered=%r left=%r"
        % (c["entered"], c["left"]))
    assert c["d_membership"] != 0.0
    assert abs(c["d_membership"]) > 100.0 * abs(c["d_smooth"]), (
        "P2: the membership term (%r) does not dominate the smooth term (%r) at rung 83 § 3.2's "
        "own jump" % (c["d_membership"], c["d_smooth"]))
    # AND THE VERDICT: the sign change does NOT survive restriction to the common points
    assert c["sign_change"] and not c["sign_change_common"] and c["kind"] == "jump", (
        "P2: rung 83 § 3.2's sign change survived restriction to the common set, so it is a "
        "root after all and the rung's headline is wrong")


# --- § 2 P4: the two flags are COEXTENSIVE, and by construction --------------------------------

@pytest.mark.slow
def test_p4_an_argmin_move_and_a_set_change_are_the_same_event(design):
    """P4 was registered predicting a COUNTER-EXAMPLE (argmin moves, set does not) and got ZERO —
    which the anchor pre-registered as the STRONGER outcome, so this gate asserts zero.

    With P1 holding, the argmin IS the window's leading point, so it can only move when the edge
    moves, and the edge can only move by a point entering. The three flags are one event. That is
    what makes rung 83's `argmin_moved` a CORRECT reading of a CONSEQUENCE: it fired at the right
    place for the wrong reason. Verdict CONFIRMED, reason CORRECTED — rung 28's shape."""
    m = _rig(design)
    sc = m.staircase_scan(0.0190, 0.0206, 9, **_kw(0.25))
    assert sc["all_at_edge"], "P1 failed inside P4's own ladder"
    assert sc["n_argmin_only"] == 0, (
        "P4: %d pair(s) moved the argmin without changing the set — an INTERIOR handover. The "
        "residual then kinks without jumping there, and rung 83's flag is a false positive rather "
        "than a proxy." % sc["n_argmin_only"])
    assert sc["n_set_only"] == 0
    assert sc["n_edge_moves"] == sum(1 for p in sc["pairs"] if p["argmin_moved"]) == \
        sum(1 for p in sc["pairs"] if p["set_changed"])
    assert sc["exact_zero_when_set_equal"] and sc["nonzero_when_set_differs"]
    assert sc["edge_monotone"], "V4: the edge is not monotone, so a two-march count is invalid"
    assert sc["all_on_grid"], "V3: an edge off the march grid"


# --- § 3 THE LATTICE COUNT: `n_jumps * ds` is a PLANT number ----------------------------------

@pytest.mark.slow
def test_p3_the_jump_count_times_the_step_is_a_constant_of_the_plant(design):
    """P3 was registered as a RATIO ("doubles ±25% at each halving") and is REFUTED — measured
    3.00 / 1.67 / 2.00 / 1.90 over five steps. The bar was mis-specified in a way this project has
    now recorded three times: A SMALL INTEGER COUNT CANNOT CARRY A RATE. At counts of 1 and 3 the
    ±1 quantization inherent to counting grid crossings is ±100% and ±33%, so the first two ratios
    measure quantization, not the plant. The two ratios taken at counts ≥ 5 are 2.00 and 1.90.

    THE MECHANISM'S OWN INVARIANT IS `n_jumps * ds = Δs*`, the distance the four-loop window's
    opening travels across the τ window — a plant property with no `ds` in it. Each level pins it
    only to ± ds, so the test is that the levels' bands INTERSECT, which is the strongest thing a
    set of quantized counts can say. Measured over five steps spanning 16×, the intersection is
    [0.005625, 0.006250]; this gate runs the three cheapest of them.

    The two-march count is licensed by V4 (a monotone edge), which `test_p4_...` certifies against
    a ladder on this same ramp."""
    m = _rig(design)
    lo, hi = 0.016, 0.024
    band_lo, band_hi = 0.0, 1.0
    counts = []
    for ds in (0.005, 0.0025, 0.00125):
        c = m.lattice_count(lo, hi, **_kw(0.25, ds))
        assert c["void"] is None and c["on_grid"], "V3: %s" % c["void"]
        assert all(c["at_edge"]), "P1 failed at a counting endpoint"
        assert c["n_jumps"] >= 1
        counts.append(c)
        band_lo = max(band_lo, (c["n_jumps"] - 1) * ds)
        band_hi = min(band_hi, (c["n_jumps"] + 1) * ds)
    assert band_lo < band_hi, (
        "P3: the levels' bands do NOT intersect (%r) — `n_jumps * ds` is then not one plant "
        "number and § 1.2's lattice picture is wrong" % [c["ds_star"] for c in counts])
    assert [c["n_jumps"] for c in counts] == [1, 3, 5], (
        "the counts moved: %r" % [c["n_jumps"] for c in counts])
    # AND THE COUNT RISES AS THE STEP FALLS — the direction is not quantized away
    assert counts[0]["n_jumps"] < counts[1]["n_jumps"] < counts[2]["n_jumps"]


# --- § 4 THE MAP: root existence flips with the march step, and the classifier says so ---------

@pytest.mark.slow
def test_p7_the_same_ramp_has_no_root_at_one_step_and_a_root_at_the_next(design):
    """P7's decisive pair, and the whole map's content in two cells.

    `r = 0.25` is rung 83 § 3's ramp. At the shipped step its sign change is a JUMP — the exact
    classifier reproduces rung 83's own bisected answer, 0.019754, and says it is not a root. One
    halving later the SAME ramp's sign change is a CROSSING with the membership term exactly zero.

    ROOT EXISTENCE IS THEREFORE A PROPERTY OF THE PLANT **AND ITS RESOLUTION**, which is rung 83
    § 3.3's observation re-measured with a criterion instead of a ratio. The full map (five ramps ×
    both shipped steps, 1 of 10 without a root) is in `docs/rung84-spec.md`; gating all ten costs
    150 marches and adds no claim these two do not carry."""
    m = _rig(design)
    coarse = m.root_class(bracket=BRACKET, **_kw(0.25, 0.005))
    assert coarse["void"] is None, coarse["void"]
    assert coarse["kind"] == "jump" and coarse["root_exists"] is False, (
        "P7: r=0.25 at the shipped step classifies as %r — rung 83 § 3.2's missing root is gone"
        % coarse["kind"])
    assert coarse["set_changed"] and coarse["d_membership"] != 0.0
    assert abs(coarse["mid"] - 0.019754) < 1e-5, (
        "the bisection no longer lands on rung 83 § 3.2's answer: %r" % coarse["mid"])

    fine = m.root_class(bracket=BRACKET, **_kw(0.25, 0.0025))
    assert fine["void"] is None, fine["void"]
    assert fine["kind"] == "crossing" and fine["root_exists"] is True, (
        "P7: r=0.25 at ds=0.0025 classifies as %r — rung 83 § 3.3 says refining makes the root "
        "exist, and this is the measurement of that" % fine["kind"])
    assert fine["d_membership"] == 0.0, (
        "a crossing must have an EXACTLY zero membership term; got %r" % fine["d_membership"])
    # THE MOVE IS INSIDE THE SAWTOOTH BOUND § 1.3 derives, and that is the point of quoting it:
    # rung 82's V5 compares this move against a BISECTION width and so trips on a benign shift.
    assert abs(fine["mid"] - coarse["mid"]) < 0.48 * (0.005 + 0.0025), (
        "the root moved by %.3e, outside the sawtooth amplitude the mechanism allows"
        % abs(fine["mid"] - coarse["mid"]))


# --- § 5 THE STAIRCASE NUMBER: its FACTORS carry the claim, not its spread ---------------------

@pytest.mark.slow
def test_p5_the_rise_and_the_branch_slope_are_what_make_lambda_ds_free(design):
    """P5 is REFUTED (60.9% spread on the registered estimator) and this gate does NOT re-score it.

    What it gates is the reason: `Λ = (ĥ'/κ)·|ds*/dτ|/|dg/dτ|` has no `ds` in it because each
    factor is first order or flat, and each is measured on its own. The registered estimator
    divided by a tread taken from a count of 1 or 3 — ±100% and ±33% of pure quantization — which
    is P3's defect wearing P5's clothes (§ 0.1).

    SO THE ASSERTIONS ARE ON THE FACTORS. `|dg/dτ|` is a plant slope and must be flat across a 4×
    change of step; the rise must fall with the step. `spacing` is passed EXPLICITLY, which is the
    signature correction § 0.2 describes — a `staircase_number` that derived it internally is what
    refuted P5."""
    m = _rig(design)
    lo, hi = 0.016, 0.024
    ds_star = 0.005938                     # § 3's five-level intersection midpoint
    out = []
    for ds in (0.0025, 0.00125):
        # isolate ONE edge move by bisecting on the edge INDEX — an integer, so this is exact
        a, b = lo, hi
        ia = m.edge_read(a, **_kw(0.25, ds))["edge_index"]
        for _ in range(12):
            mid = 0.5 * (a + b)
            im = m.edge_read(mid, **_kw(0.25, ds))["edge_index"]
            if ia - im >= 1:
                b = mid
            else:
                a, ia = mid, im
            if b - a < 1e-5:
                break
        sn = m.staircase_number(a, b, spacing=ds * (hi - lo) / ds_star, **_kw(0.25, ds))
        assert sn["void"] is None, sn["void"]
        assert sn["lam"] is not None and sn["spacing"] is not None
        out.append(sn)
    # THE BRANCH SLOPE IS A PLANT NUMBER — flat across a 2x step change, not a march artifact
    s0, s1 = out[0]["dg_dtau"], out[1]["dg_dtau"]
    assert abs(s1 - s0) / s0 < 0.10, (
        "|dg/dtau| moved %.1f%% between steps — it is then not the branch slope § 1.3 divides by"
        % (100.0 * abs(s1 - s0) / s0))
    # AND THE RISE FALLS WITH THE STEP, which is what makes it the FIRST-ORDER term
    assert out[1]["rise"] < out[0]["rise"], (
        "the rise did not fall when the step halved (%r -> %r) — § 1.3's `ĥ'·ds` is wrong"
        % (out[0]["rise"], out[1]["rise"]))
    # WITHOUT a spacing the reader must return the FACTORS and NO `lam` — § 0.2's correction, so
    # that a caller cannot get a quantized number by accident
    bare = m.staircase_number(out[0]["tau_lo"], out[0]["tau_hi"], **_kw(0.25, 0.0025))
    assert bare["lam"] is None and bare["tread"] is None and bare["rise"] is not None


# --- § 6 P6 REFUTED: refinement did NOT bring the absence back --------------------------------

@pytest.mark.slow
def test_p6_refuted_the_missing_root_does_not_return_when_the_step_is_refined(design):
    """P6 predicted existence would NOT be monotone in `ds`. It is: absent / present / present /
    present at r=0.25 over 0.005 / 0.0025 / 0.00125 / 0.000625.

    THIS GATE ASSERTS THE REFUTATION, not the prediction. §§ 1–5 derive that the shadow FRACTION
    is `ds`-invariant, which is a statement about a RATE; one ramp over four steps cannot test one,
    and the spec says so in § 6 rather than reading three re-rolls as a trend. A gate that asserted
    the prediction would be asserting something this rung measured to be false.

    The two coarsest steps are gated here; the two finest cost ~360 s and add no claim — their
    values are in § 6 and the probe JSON."""
    m = _rig(design)
    seq = [m.root_class(bracket=BRACKET, **_kw(0.25, ds))["root_exists"]
           for ds in (0.005, 0.0025, 0.00125)]
    assert seq == [False, True, True], (
        "the existence sequence moved: %r. § 6 records [False, True, True, True] over four steps; "
        "a change here is either the plant moving or P6 becoming testable." % seq)


# --- § 6.1 RUNG 82's V5 GETS A SCALE ----------------------------------------------------------

@pytest.mark.slow
def test_the_threshold_move_between_steps_is_inside_the_sawtooth_bound(design):
    """§ 6.1: rung 82's V5 voids a threshold that moves by more than its own BISECTION width, and
    trips at r=0.35 — which that rung reported as a category error it could not repair.

    § 1.2 supplies the missing scale: the root inherits the sawtooth, so it may shift by up to
    `(ĥ'/κ)·ds/|dg/dτ| ≈ 0.48·ds` per step for an entirely benign reason. Across the two shipped
    steps that is 3.6e-3 — TWELVE bisection widths. This gates the ramp V5 voids: its move is real,
    it is the largest of the five, and it is inside the bound at 40%.

    TWO RAMPS, not five: r=0.35 is the one V5 trips on and r=0.20 is the control that does not move
    at all. The other three are in § 6.1's table and cost 90 marches to re-measure."""
    m = _rig(design)
    bound = 0.48 * (0.005 + 0.0025)
    width = (BRACKET[1] - BRACKET[0]) / 2 ** 10
    for r, expect_move in ((0.35, True), (0.20, False)):
        c = m.root_class(bracket=BRACKET, **_kw(r, 0.005))
        f = m.root_class(bracket=BRACKET, **_kw(r, 0.0025))
        assert c["void"] is None and f["void"] is None
        assert c["kind"] == f["kind"] == "crossing"
        moved = abs(f["mid"] - c["mid"])
        assert moved < bound, (
            "r=%g moved %.3e between the shipped steps, outside the sawtooth bound %.3e — the "
            "residual then carries something § 1.2 does not describe" % (r, moved, bound))
        if expect_move:
            assert moved > width, (
                "r=%g no longer trips rung 82's V5 (moved %.3e vs one bisection width %.3e), so "
                "§ 6.1 has nothing to correct" % (r, moved, width))
        else:
            assert moved == 0.0, "r=%g's control moved by %.3e" % (r, moved)
