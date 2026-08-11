"""Rung 82 — THE THRESHOLD'S OWN LAW: `docs/rung81-spec.md` § 8's first seam.

Rung 81 derived a criterion for who holds the actuator and scored it as a LABEL predictor —
99.15 %. Its seam asked for the next step: turn it into a THRESHOLD predictor, quantitatively.
**The answer is no, and this rung is why.**

HEADLINE (§ 3a): **a forward reading inherits the sign of its own reference.** At one ramp, on
one plant, whether the criterion's forward prediction lands above or below the true threshold
follows the side its REFERENCE march sat on — 5 of 5. So it is not a prediction of the
threshold; it is a report on where the reader started. The fixed point, which has no reference,
lands to 2.7–9.4 %.

AND THE MECHANISM (§ 4–5): **the criterion's terms are not independent coordinates.** Every knob
reaches every term through the trajectory — the wall, which the criterion places in the
set-point term, is the largest mover of `ċ_f` in the rung (+144 %).

P1 is VOID BY ITS OWN BAR (a bisection width set by a loop count), P5 is refuted in BOTH halves,
and V5's registered form is the same self-referential error as P1 — see
`docs/plans/rung82-anchor-threshold-law.md` § 6a. Scoring + tables: `docs/rung82-spec.md`.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    ThresholdLawTransient, AuthorityClockTransient, BleedLimiter, StatorLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, TT4_MAX = 1000.0, 1400.0, 1200.0
B, V_MAX, TAU, TAU_S = 0.10, 0.20, 0.05, 0.05

PHI_FUEL, PHI_AIR = 0.75, 0.77          # rung 80's walls, unchanged
MATCHED = (0.05, 0.05, 0.05, 0.05)      # rung 81's clocks: the identity control
R81 = 0.5                               # rung 81's own ramp

# THE RAMP SET IS TRIMMED AGAINST THE SPEC's; THE RESOLUTION IS NOT, AND THAT IS THE LESSON.
# The spec's § 2 runs five ramps, and three certify every claim read off them. But the first
# version of this file ALSO cut `n_bisect` 10 → 7, and TWO gates flipped: at 7 the bracket is
# 2.3e-3, while § 3a's closest signed error is 9.2e-4 and § 2's fixed-point gap at `r = 0.35`
# is 1.4e-3. **The search was coarser than the effects it was measuring**, so the headline's
# 5-of-5 read 4-of-5 — not because the plant moved but because `τ*` itself was unresolved.
#
# Ten bisections give 2.89e-4, which clears the closest margin by 3.2×. `test_the_bisection_
# resolves_the_signs_it_is_asked_about` asserts that relation rather than trusting it, so a
# future trim cannot silently re-open the same hole.
RS = (0.25, 0.35, 0.50)
N_BISECT = 10

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


def _rig(design, cls=ThresholdLawTransient):
    sm = 0.80 / FLOOR - 1.0
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_lim=StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m


@pytest.fixture(scope="module")
def law(design):
    # `ds_fine=None` SKIPS the step control here — it is the most expensive leg in the reader
    # (every march twice as long) and it is gated on its own, at one ramp, below. The rows then
    # carry `ds_stable=None`, which the reader keeps distinct from False.
    return _rig(design).threshold_law(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                      phi_air=PHI_AIR, rs=RS, n_bisect=N_BISECT,
                                      ds_fine=None)


@pytest.fixture(scope="module")
def ref(design):
    return _rig(design).threshold_reference(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                            phi_air=PHI_AIR, n_bisect=N_BISECT)


@pytest.fixture(scope="module")
def terms(design):
    return _rig(design).threshold_terms(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                        phi_air=PHI_AIR, n_bisect=N_BISECT,
                                        phi_lims=(0.745, 0.750, 0.755))


# --- THE REDUCE: an IDENTITY, and it is the cheap gate ----------------------------------------

def test_reduce_is_bit_for_bit_rung81(design):
    """THE WHOLE CONTRACT OF A READER-ONLY RUNG. This rung adds no state, no knob and no
    constant, so at rung 81's clocks, walls and ramp its march must be `AuthorityClockTransient`'s
    TO THE LAST BIT. A rung-82 march that moved would be a rung-81 regression wearing a new
    class name.

    Compared on all four plant states at every point — not on a scalar. Rung 77's closure
    returned a perfect 1.000e+00 having outlived its state block, and this project's recorded
    way of being wrong is an invariance claim resting on one reduced number."""
    a = _rig(design, AuthorityClockTransient)
    b = _rig(design)
    ta = a._split_march(FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, "demand",
                        MATCHED, R81, 1.2, 0.005, V_MAX, False)[3]
    tb = b._split_march(FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, "demand",
                        MATCHED, R81, 1.2, 0.005, V_MAX, False)[3]
    assert len(ta) == len(tb) == 341
    diff = [(i, k) for i, (p, q) in enumerate(zip(ta, tb))
            for k in ("phi_lp", "Tt4", "b", "v") if p[k] != q[k]]
    assert not diff, f"rung-82 moved rung-81's march at {len(diff)} floats: {diff[:5]}"


def test_the_identity_control_reproduces_rung81s_own_cell(design):
    """RUNG 81 § 1's OWN TABLE CELL, and a grid that disagrees is not the shipped plant: 33
    four-loop points, ALL `gov`, at matched clocks on rung 80's walls."""
    s = _rig(design)._threshold_scan(
        flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=PHI_FUEL,
        phi_air=PHI_AIR, tau_f=0.05, tau_gov=0.05, tau_q=0.05, tau_s=0.05, r=R81,
        s_settle=1.2, ds=0.005, v_max=V_MAX, inc=False)
    assert s["n_riding4"] == 33, s["n_riding4"]
    assert s["n_fuel"] == 0 and s["n_gov"] == 33, (s["n_fuel"], s["n_gov"])
    assert s["riding4_valid"] and s["window_open"]


def test_V1_no_four_loop_window_above_the_admissible_ramp(design):
    """V1, AND THE DISTINCTION IT PROTECTS. At `r = 1.0` there is no four-loop point at all, so
    that ramp does not carry a LARGE threshold — it carries NONE. A reader that reported a
    number there would be quoting `docs/rungs72-77-march-audit.md`'s frozen-plant census."""
    s = _rig(design)._threshold_scan(
        flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=PHI_FUEL,
        phi_air=PHI_AIR, tau_f=0.05, tau_gov=0.05, tau_q=0.05, tau_s=0.05, r=1.0,
        s_settle=1.2, ds=0.005, v_max=V_MAX, inc=False)
    assert s["n_riding4"] == 0 and not s["window_open"]
    # ... and the plant DID accelerate, so the empty window is not an arrest (V2 vs V1)
    assert s["riding4_valid"], "an arrested plant would make this V2, not V1"


# --- § 2: THE RAMP SWEEP ----------------------------------------------------------------------

@pytest.mark.slow
def test_the_threshold_rises_monotonically_with_the_ramp(law):
    """THE SEAM'S OWN QUESTION, ANSWERED IN ITS OWN DIRECTION: the threshold DOES move with the
    schedule's slope. It is the only half of the seam that survives — § 4–5 refute that it moves
    with `ċ_f/ċ_r`, which is what the seam actually named."""
    assert law["n_void"] == 0, [x.get("void") for x in law["rows"]]
    assert law["monotone_in_r"], law["thresholds"]
    lo, hi = law["thresholds"][0][1], law["thresholds"][-1][1]
    assert hi > 2.5 * lo, f"the ramp barely moved the threshold: {law['thresholds']}"


@pytest.mark.slow
def test_the_effective_clock_is_the_release_one_everywhere(law):
    """E5, AND THE 3x THIS WHOLE RUNG IS QUOTED AGAINST. Every binding point sits in RELEASE, so
    the swept knob and the criterion's active constant differ by exactly rung 52's factor. V4
    voids any row mixing regimes; none does, and that is asserted rather than assumed."""
    assert law["all_kappa_pure"], [x["kappa"] for x in law["rows"]]
    assert law["kappa_seen"] == [3.0], law["kappa_seen"]


@pytest.mark.slow
def test_the_fixed_point_is_never_beaten_by_the_forward_reading(law):
    """P3's SURVIVING HALF, and the comparison P1 was reaching for before its own bar voided it.

    Scored on the ERROR and not on bracket membership: the bracket is `2^-N` of the search
    interval, a width set by a loop count, so a gate written against it would pass or fail on
    `N_BISECT` rather than on the plant (anchor § 6a, P1)."""
    live = [x for x in law["rows"] if x["ok"]]
    assert len(live) == len(RS)
    assert law["p3_fwd_never_better"], [(x["r"], x["err_fixed"], x["err_fwd"]) for x in live]
    assert all(x["err_fixed"] < 0.12 for x in live), [(x["r"], x["err_fixed"]) for x in live]


@pytest.mark.slow
def test_the_fixed_point_sits_below_the_measured_threshold(law):
    """P2's SURVIVING HALF — rung 81 § 2's *"the criterion is early, never late"* transferred
    from the label to the threshold, for the reading that has no reference to inherit from."""
    live = [x for x in law["rows"] if x["ok"]]
    assert law["p2_fixed_early"] == [x["r"] for x in live], law["p2_fixed_early"]


@pytest.mark.slow
def test_the_step_control_in_its_own_currency(design):
    """V5, RE-SCORED IN THE CURRENCY IT SHOULD HAVE CARRIED — and the registered form is NOT
    gated, because it compared a physical move against a bisection width (P1's error twice).

    Halving `ds` moves the threshold by under 1 %, two orders below the effect being measured,
    so the thresholds are RESOLVED. That is the claim worth protecting. Run at ONE ramp because
    the fine leg doubles every march; the spec's § 2 carries all five."""
    m = _rig(design)
    base = dict(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=PHI_FUEL,
                phi_air=PHI_AIR, tau_gov=0.05, tau_q=0.05, tau_s=0.05, s_settle=1.2,
                v_max=V_MAX, inc=False, r=0.25)
    coarse = m._bisect(lambda s: s["n_fuel"] > 0, 0.004, 0.30, N_BISECT,
                       **dict(base, ds=0.005))
    fine = m._bisect(lambda s: s["n_fuel"] > 0, 0.004, 0.30, N_BISECT,
                     **dict(base, ds=0.0025))
    assert not coarse.get("void") and not fine.get("void"), (coarse.get("void"),
                                                             fine.get("void"))
    rel = abs(fine["mid"] - coarse["mid"]) / coarse["mid"]
    assert rel < 0.01, f"threshold moved {rel:.3%} on a halved step: {coarse['mid']}/{fine['mid']}"


@pytest.mark.slow
def test_the_unrun_step_control_does_not_read_as_a_passed_one(law):
    """THE READER'S OWN HONESTY, GATED. The `law` fixture skips the step control, and a `None`
    that collapsed into the pass list would report a control that never ran as one that held —
    rung 78's *"`ok` is not a correctness guard"*, and the cheapest possible version of it."""
    assert law["ds_unrun"] == [x["r"] for x in law["rows"] if x["ok"]], law["ds_unrun"]
    assert law["ds_stable"] == [] and law["ds_unstable"] == []


# --- § 3a: THE HEADLINE -----------------------------------------------------------------------

@pytest.mark.slow
def test_the_forward_reading_inherits_its_references_sign(ref):
    """**THE HEADLINE.** At ONE ramp — so the reference's side is not collinear with `r`, which
    is what made § 2's 5-of-5 a confounded correlation — the sign of `forward − τ*` follows the
    side the REFERENCE march sits on, at every reference swept across the threshold.

    An ALL and never a count: a single row where the two part ends the causal reading."""
    assert ref["void"] is None, ref["void"]
    assert ref["n_live"] == len(ref["tau_refs"]), ref["n_live"]
    assert ref["sign_follows_reference"], ref["crossing"]
    # ... and BOTH sides are populated, or the law is vacuous rather than confirmed
    sides = {x["ref_above"] for x in ref["rows"]}
    assert sides == {True, False}, f"the sweep never crossed the threshold: {ref['crossing']}"


@pytest.mark.slow
def test_the_bisection_resolves_the_signs_it_is_asked_about(ref):
    """THE GUARD THE FIRST VERSION OF THIS FILE NEEDED AND DID NOT HAVE.

    Every sign in the headline is `forward − τ*`, and `τ*` is only known to a bracket. So the
    claim is only meaningful if the bracket is SMALLER than the smallest margin it decides. At
    `n_bisect = 7` it was not — the closest row's margin is 9.2e-4 against a 2.3e-3 bracket, and
    the headline read 4-of-5 for a purely numerical reason.

    Asserted as a RELATION rather than as two remembered constants, so a future trim of
    `N_BISECT` fails HERE, loudly, instead of silently re-opening the hole one gate along."""
    margins = [abs(x["fwd"] - ref["tau_star"]) for x in ref["rows"] if x["fwd"] is not None]
    assert margins
    assert ref["width"] < min(margins) / 2.0, (
        f"bracket {ref['width']:.3e} cannot resolve a margin of {min(margins):.3e} — "
        "every sign in this section is undecided at this resolution")


@pytest.mark.slow
def test_the_map_contracts_from_above_and_diverges_from_below(ref):
    """THE MECHANISM, AND IT REFUTES P3's GROWTH CLAUSE ON THE SIDE NOBODY EXPECTED. Read as an
    iteration, the forward map converges in one step from ABOVE the threshold — so the error
    SHRINKS as the reference moves further away — and diverges from BELOW, where it grows.

    This is rung 77's `1/(1−c)` with the SIGN of `c` deciding whether the reading is usable at
    all, and it is why § 2's two blown-up ramps are exactly the two whose reference sat low."""
    above = sorted((x for x in ref["rows"] if x["ref_above"]), key=lambda x: x["dist"])
    below = sorted((x for x in ref["rows"] if not x["ref_above"]), key=lambda x: x["dist"])
    assert len(above) >= 2 and len(below) >= 2
    assert ref["grows_above"] is False, [(x["dist"], x["err"]) for x in above]
    assert ref["grows_below"] is True, [(x["dist"], x["err"]) for x in below]
    # the asymmetry in one number: the worst reading from below beats nothing from above
    assert max(x["err"] for x in below) > 3.0 * max(x["err"] for x in above), (
        [x["err"] for x in below], [x["err"] for x in above])


# --- § 4–5: THE OTHER TWO KNOBS ---------------------------------------------------------------

@pytest.mark.slow
def test_the_governor_clock_keeps_only_part_of_its_coefficient(terms):
    """P4: THE SIGN HOLDS AND THE MAGNITUDE DOES NOT. `transfer` is the fraction of a
    FROZEN-TRAJECTORY coefficient that survives the plant's own response — this rung's headline
    measured on an independent knob.

    Gated as a BAND, not a value: the point is that it is neither 0 (the coefficient reaching
    nothing) nor 1 (the trajectory not responding), and pinning it tighter would gate a secant
    over a 10x span that the spec's own sub-interval table shows is not constant."""
    p4 = terms["p4"]
    assert p4 is not None and p4["rises"], p4
    assert 0.2 < p4["transfer"] < 0.8, p4["transfer"]
    assert p4["rel_err"] > 0.25, (
        f"P4's registered 25 % bar was MISSED at {p4['rel_err']:.1%}; a gate asserting it "
        "passed would ship the opposite of what was measured")


@pytest.mark.slow
def test_the_wall_moves_the_threshold_the_other_way(terms):
    """P5's DIRECTION, REFUTED AND GATED AS REFUTED. `φ_lim` is the fuel leg's OWN floor, so
    RAISING it makes that leg's cap MORE severe, which LOWERS the set-point gap and lowers the
    threshold. The anchor registered the opposite and is wrong."""
    p5 = terms["p5"]
    assert p5 is not None and terms["n_void"] == 0, terms["n_void"]
    assert p5["rises"] is False, p5["taus"]
    assert p5["monotone"], p5["taus"]
    assert p5["gap_falls"], p5["gaps"]


@pytest.mark.slow
def test_the_terms_do_not_separate_and_the_wall_moves_the_wrong_slope(terms):
    """**V7 FIRED, AND THE SECOND HALF IS THE FINDING.** The criterion's two terms are `gap` and
    `τ_gov·ċ_r`; a wall that reached only the first would leave the second alone. It does not —
    and the largest move in the table is `ċ_f`, a slope the wall reaches through NEITHER term.

    Scored on `τ_gov·ċ_r` and never on `ċ_f/ċ_r`: the ratio is not one of the criterion's terms,
    and withdrawing P5 on it would withdraw it on a quantity the derivation does not contain."""
    p5 = terms["p5"]
    assert p5["separates"] is False and p5["v7_withdrawn"] is True, p5
    assert p5["d_lag"] > p5["d_gap"] / 3.0, (p5["d_gap"], p5["d_lag"])
    assert p5["d_slope_f"] > p5["d_gap"], (p5["d_gap"], p5["d_slope_f"])


def test_V3_censors_the_wall_on_both_sides(design):
    """V3, BOTH ENDS — and this is why § 5 sweeps a range a thousandth wide. At `φ_lim = 0.740`
    the threshold sits ABOVE the bracket and at 0.760 BELOW it: a >20x swing in the threshold
    for 0.02 of wall. An endpoint is a CENSORED observation, never a value, and the reader must
    void rather than return the endpoint."""
    m = _rig(design)
    base = dict(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_air=PHI_AIR,
                tau_gov=0.05, tau_q=0.05, tau_s=0.05, s_settle=1.2, v_max=V_MAX,
                inc=False, r=0.35, ds=0.005)
    hi_wall = m._bisect(lambda s: s["n_fuel"] > 0, 0.004, 0.30, 4,
                        **dict(base, phi_lim=0.760))
    lo_wall = m._bisect(lambda s: s["n_fuel"] > 0, 0.004, 0.30, 4,
                        **dict(base, phi_lim=0.740))
    assert hi_wall.get("void") and hi_wall["below"], hi_wall.get("void")
    assert lo_wall.get("void") and lo_wall["above"], lo_wall.get("void")
    # ... and NEITHER is void for want of a window — that would be V1, a different reading
    assert hi_wall["at_lo"]["window_open"] and lo_wall["at_hi"]["window_open"]


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__, "-v"]))
