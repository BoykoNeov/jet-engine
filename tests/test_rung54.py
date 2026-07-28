"""Rung 54 — THE STATOR-ROW THROAT: a constraint's severity is coordinate-dependent too.

Rung 53 refused the flow-CAPACITY half of the variable stator because it "needs a new
constant (area per unit setting)". The cascade cosine rule o/s = cos(alpha_1) derives the
area law's SHAPE off rung 53's OWN coordinate (v = tan alpha_1) with zero new constants; the
LEVEL still needs one disclosed number, the design capacity fraction C = MFP(M_th0)/MFP(1),
and every claim below is delivered as a THRESHOLD on it.

Gates (named in docs/rung54-spec.md § Verification gates):

   1. REDUCE — an INVARIANCE OVER C, stronger than rung 53's identity at a point: the throat
      enters NO solver, so EVERY matched field is bit-identical for EVERY capacity, at a
      MOVED stator, on both gases. Plus capacity=0 => rung 53 untouched, and the is_flat
      rule (capacity ignored like phi_surge, NOT like vsv).
   2. THE DERIVED AREA LAW — throat_ratio == cos(atan v) exactly, EXACTLY EVEN in v, == 1 at
      the design setting, and X == m at v = 0.
   3. THE ONE CONSTANT, DISCLOSED — with_capacity rejects C >= 1; design_throat_mach inverts
      MFP to the tabulated Machs; c_min is reported WITHOUT any constant and m_c == 1 - C*X.
   4. P1 — BIND, NEVER RELIEVE (the theorem): a throat that CHOKES leaves every matched
      number bit-identical, so the channel cannot buy back rung 53's overspeed.
   5. THE HEADLINE — severity is coordinate-dependent: the throat cuts the SETTING far more
      than the incidence MARGIN, across all five shapes and the throttle band.
   6. THE ARTIFACT IS NEVER THE CEILING — v_ch < v_edge on every shape x throttle at
      C >= 0.80, and C_edge < 0.90 on every shape (P-A2, the shape-robust SIGN claim).
   7. P-C2 — THE TURNING POINT IS REACHED, correcting rung 53's concession: an interior
      incidence peak on the tilted/steep shapes (and NOT on flow/press, where rung 53
      measured), and rung 53's P7 schedule ceases to EXIST inside the envelope.
   8. P-C3 — rung 54's root is immune to the non-monotone residual that defeats rung 53's
      doubling ladder, and AGREES with it wherever that ladder succeeds.
   9. THE RACE — X(v*) crosses the DESIGN loading (a constant-free boundary) inside the
      envelope on the default shape, and the HP never approaches its throat (the exposure
      split, inherited).
  10. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    VariableStatorMatcher,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55

# Rung 53's five disclosed shapes, verbatim.
SHAPES = {
    "flow/press": (ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
                   ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85),
                   ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)),
    "steep":      (ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2),
                   ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)),
    "flat-eta":   (ComponentMap(sigma=0.1, l=0.7), ComponentMap(sigma=0.1, l=1.0)),
}
LP = SHAPES["flow/press"][0].with_phi_surge(FLOOR)
HP = SHAPES["flow/press"][1].with_phi_surge(FLOOR)
THROTTLE = (1500.0, 1200.0, 1000.0, 800.0)
FIELDS = ("pi_lpc", "pi_hpc", "n_lp", "n_hp", "phi_lp", "phi_hp", "slip",
          "eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt", "tau_lpc", "tau_hpc",
          "tau_hpt", "tau_lpt", "mdot_air", "thrust", "N_lp_ratio", "N_hp_ratio")


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _vm(gas, ml=LP, mh=HP, vl=0.0, vh=0.0, design=None):
    return VariableStatorMatcher(design if design is not None else _design(gas),
                                 FLIGHT, 1.0, map_lp=ml, map_hp=mh, vsv_lp=vl, vsv_hp=vh)


def _shaped(gas, shape, C=0.0, vl=0.0, vh=0.0, design=None):
    ml, mh = (m.with_phi_surge(FLOOR) for m in SHAPES[shape])
    if C > 0.0:
        ml, mh = ml.with_capacity(C), mh.with_capacity(C)
    return _vm(gas, ml=ml, mh=mh, vl=vl, vh=vh, design=design)


# ======================================================================================
# GATE 1 — REDUCE: an INVARIANCE OVER C, not merely an identity at C = 0
# ======================================================================================

@pytest.mark.parametrize("vl,vh", [(0.0, 0.0), (0.30, 0.0), (0.0, 0.15), (0.20, 0.10)])
def test_reduce_every_matched_field_is_bit_identical_for_every_capacity(vl, vh):
    """THE RUNG'S STRONGEST CLAIM (P1). `v` enters the solve through solve_n alone (rung 53)
    and the throat enters NO solver, so X is a post-hoc functional of the SOLVED state. Then
    the capacity constant cannot move ANY matched number — for every C, not just C = 0, and
    at a MOVED stator. Rung 53 earned an identity at one setting; rung 54 earns invariance
    over a whole parameter."""
    gas = _cpg_gas()
    design = _design(gas)
    base = _vm(gas, vl=vl, vh=vh, design=design)
    ref = {T: base.match(FLIGHT, T) for T in THROTTLE}
    for C in (0.05, 0.30, 0.55, 0.80, 0.95, 0.999):
        m = _vm(gas, ml=LP.with_capacity(C), mh=HP.with_capacity(C),
                vl=vl, vh=vh, design=design)
        for T in THROTTLE:
            od, od0 = m.match(FLIGHT, T), ref[T]
            for f in FIELDS:
                assert getattr(od, f) == getattr(od0, f), (
                    f"capacity C={C} moved {f} at Tt4={T} (vsv={vl},{vh}): "
                    f"{getattr(od, f)!r} vs {getattr(od0, f)!r} — the throat entered a solver")


def test_reduce_holds_on_the_reacting_gas_too():
    """The invariance is structural, so it must not be a CPG accident."""
    gas = Gas.reacting_equilibrium()
    design = _design(gas)
    base = _vm(gas, vl=0.25, design=design)
    for C in (0.40, 0.85):
        m = _vm(gas, ml=LP.with_capacity(C), mh=HP.with_capacity(C), vl=0.25, design=design)
        for T in (1500.0, 1200.0):
            od, od0 = m.match(FLIGHT, T), base.match(FLIGHT, T)
            for f in FIELDS:
                assert getattr(od, f) == getattr(od0, f), f"C={C} moved {f} at Tt4={T}"


def test_reduce_capacity_is_not_part_of_flatness_but_vsv_still_is():
    """The phi_surge rule, read for the throat: a PURE DIAGNOSTIC that never touches
    psi/eta/the running line is not part of flatness, so a flat map WITH a throat model
    still reduces MapMatcher to rung 31. rung 53's vsv is the opposite case and stays so."""
    assert ComponentMap.flat().with_capacity(0.8).is_flat()
    assert ComponentMap.flat().with_phi_surge(0.6).with_capacity(0.8).is_flat()
    assert not ComponentMap.flat().with_vsv(0.1).is_flat()
    assert not ComponentMap.flat().with_capacity(0.8).with_vsv(0.1).is_flat()


def test_reduce_capacity_zero_leaves_rung_53_expressions_bit_for_bit():
    """C = 0 is 'no throat model' exactly as phi_surge = 0 is 'no surge line'."""
    m = LP.with_vsv(0.3)
    assert m.capacity == 0.0
    assert m.with_capacity(0.0) == m
    for meth, arg in (("psi", 0.8), ("phi_max", 0.1)):
        assert getattr(m, meth)(arg) == getattr(m.with_capacity(0.7), meth)(arg)
    assert m.phi_surge_at() == m.with_capacity(0.7).phi_surge_at()
    assert m.tan_beta1(0.8) == m.with_capacity(0.7).tan_beta1(0.8)
    with pytest.raises(AssertionError):
        m.capacity_margin(1.0)          # no throat model => the margin is not defined


# ======================================================================================
# GATE 2 — THE DERIVED AREA LAW (shape: zero new constants)
# ======================================================================================

@pytest.mark.parametrize("v", [-1.5, -0.6, -0.2, 0.0, 0.2, 0.6, 1.5, 3.0])
def test_throat_ratio_is_the_cascade_cosine_rule(v):
    """A_th(v)/A_th(0) = cos(alpha_1) with v = tan(alpha_1): o/s = cos(alpha) is the standard
    cascade throat relation, so the area law rides on rung 53's OWN coordinate."""
    got = ComponentMap(l=0.7).with_vsv(v).throat_ratio()
    assert got == pytest.approx(math.cos(math.atan(v)), rel=0.0, abs=1e-15)
    assert got == pytest.approx(1.0 / math.sqrt(1.0 + v * v), rel=0.0, abs=1e-15)


@pytest.mark.parametrize("v", [0.2, 0.6, 1.5, 3.0])
def test_throat_area_law_is_exactly_even_and_unity_at_design(v):
    """The GEOMETRIC cost is two-sided: cos is even, so the throat is maximal AT the design
    setting and closes whichever way the vane turns. (That the peak coincides with the design
    setting is INHERITED from rung 53's coordinate origin, not derived — see the spec's
    Concessions.) Any measured asymmetry must therefore come from elsewhere: gate 5b."""
    m = ComponentMap(l=0.7)
    assert m.with_vsv(v).throat_ratio() == m.with_vsv(-v).throat_ratio()
    assert m.with_vsv(v).throat_ratio() < 1.0
    assert m.throat_ratio() == 1.0
    assert m.throat_loading(0.83) == 0.83                      # X == m at the design setting


# ======================================================================================
# GATE 3 — THE ONE CONSTANT, DISCLOSED (and the escape from it)
# ======================================================================================

def test_capacity_constant_is_bounded_and_reads_as_a_design_throat_mach():
    """C >= 1 would mean the row is past choke at its own design point. And C is disclosed in
    units an engineer can judge: the design throat Mach, by inverting MFP(M)/MFP(1)."""
    for bad in (1.0, 1.2, -0.1):
        with pytest.raises(AssertionError):
            LP.with_capacity(bad)
    for C, M in ((0.70, 0.4583), (0.80, 0.5533), (0.90, 0.6782)):
        assert LP.with_capacity(C).design_throat_mach() == pytest.approx(M, abs=5e-4)
    # strictly increasing, so the inverse is well posed
    machs = [LP.with_capacity(C).design_throat_mach() for C in (0.5, 0.6, 0.7, 0.8, 0.9)]
    assert machs == sorted(machs)


def test_c_min_is_reported_without_any_constant_and_the_margin_uses_it():
    """The escape from the disclosed constant: c_min = 1/X is a DERIVED threshold, present
    whether or not a throat model is attached, and the row chokes iff C >= c_min."""
    gas = _cpg_gas()
    design = _design(gas)
    bare = _vm(gas, vl=0.4, design=design).throat_margin(FLIGHT, 1200.0)["lp"]
    assert bare["capacity"] == 0.0 and "m_c" not in bare
    X = bare["throat_loading"]
    assert bare["c_min"] == pytest.approx(1.0 / X, rel=1e-14)
    for C in (0.5, 0.8):
        r = _vm(gas, ml=LP.with_capacity(C), mh=HP.with_capacity(C), vl=0.4,
                design=design).throat_margin(FLIGHT, 1200.0)["lp"]
        assert r["throat_loading"] == pytest.approx(X, rel=1e-14)   # X is constant-free
        assert r["m_c"] == pytest.approx(1.0 - C * X, rel=1e-14)
        assert r["choked"] == (C >= r["c_min"])


def test_throat_loading_equals_face_flow_times_secant():
    """X = m*sqrt(1+v^2): the face-referred corrected flow is NOT divided by the throat
    (annulus continuity keeps Vx independent of alpha_1), so phi is untouched and only the
    throat referral changes. That is the whole reason the channel is diagnostic-only."""
    gas = _cpg_gas()
    design = _design(gas)
    for v in (0.0, 0.35, 0.9):
        r = _vm(gas, vl=v, design=design).throat_margin(FLIGHT, 1200.0)["lp"]
        assert r["throat_loading"] == pytest.approx(
            r["m"] * math.sqrt(1.0 + v * v), rel=1e-13)
        assert r["m"] == pytest.approx(r["phi_op"] * r["n"], rel=1e-13)


# ======================================================================================
# GATE 4 — P1: BIND, NEVER RELIEVE
# ======================================================================================

def test_a_choked_row_still_leaves_every_matched_number_untouched():
    """The theorem's operational face. Pick a setting the row cannot pass (C*X > 1) and check
    the solve does not notice: the throat REMOVES SETTINGS FROM THE FEASIBLE SET, it does not
    change the map from setting to incidence. So no area law could buy back rung 53's
    overspeed — which REFUTES the expectation rung 53's own seam recorded."""
    gas = _cpg_gas()
    design = _design(gas)
    C = 0.95
    m = _vm(gas, ml=LP.with_capacity(C), mh=HP.with_capacity(C), vl=1.1, design=design)
    r = m.throat_margin(FLIGHT, 1500.0)["lp"]
    assert r["choked"] and r["m_c"] < 0.0, "pick a setting that actually chokes the row"
    bare = _vm(gas, vl=1.1, design=design)
    for f in FIELDS:
        assert getattr(m.match(FLIGHT, 1500.0), f) == getattr(bare.match(FLIGHT, 1500.0), f)


# ======================================================================================
# GATE 5 — THE HEADLINE: severity is coordinate-dependent
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("shape", list(SHAPES))
def test_headline_the_throat_cuts_the_setting_far_more_than_the_margin(shape):
    """RUNG 54's HEADLINE, and rung 53's law read one level up. Rung 53: a MARGIN is a
    distance, so it is coordinate-dependent. Rung 54: so is a CONSTRAINT'S SEVERITY. The
    throat truncates the stator hard in the lever's own coordinate and nearly not at all in
    the protected variable, because the coordinate's returns have already flattened."""
    gas = _cpg_gas()
    design = _design(gas)
    m = _shaped(gas, shape, C=0.90, design=design)
    seen = False
    for T in (1200.0, 1000.0, 800.0):
        a = m.authority_ceiling(FLIGHT, T, "lp")
        if a["v_ch"] is None:
            continue
        seen = True
        assert a["setting_cut"] > 0.10, (
            f"{shape} @{T}: the throat should bite the SETTING appreciably, "
            f"got {a['setting_cut']:.3f}")
        assert a["retained"] > a["setting_cut"], f"{shape} @{T}: severity not inverted"
        assert a["retained"] >= 0.78, (
            f"{shape} @{T}: retention {a['retained']:.3f} — the margin cost should stay "
            f"small even where the setting cost is large")
    assert seen, f"{shape}: the throat never bound anywhere — gate vacuous"


def test_headline_the_default_shape_numbers():
    """The spec's quoted case, pinned: at Tt4 = 1000, C = 0.90 the setting is cut ~30% and
    the margin ~4%."""
    m = _shaped(_cpg_gas(), "flow/press", C=0.90)
    a = m.authority_ceiling(FLIGHT, 1000.0, "lp")
    assert a["setting_cut"] == pytest.approx(0.304, abs=0.02)
    assert a["retained"] == pytest.approx(0.905, abs=0.02)
    assert a["m_i_usable"] / a["m_i_peak"] == pytest.approx(0.960, abs=0.02)


@pytest.mark.parametrize("shape", ["flat-eta", "flow/press", "steep"])
def test_the_measured_asymmetry_is_the_efficiency_islands(shape):
    """GATE 5b, an EXACT ZERO. The geometric cost is exactly even (gate 2), so any asymmetry
    in the MEASURED cost X(v) must enter through m — which moves only via the efficiency
    island. On a FLAT island rung 53's P5 pins m exactly, so X must be even BIT-FOR-BIT;
    on a shaped island it must not be (or the zero is vacuous)."""
    gas = _cpg_gas()
    rows = {r["vsv"]: r for r in _shaped(gas, shape).throat_sweep(
        FLIGHT, 1500.0, [-0.6, -0.4, -0.2, 0.2, 0.4, 0.6], "lp")}
    diffs = [rows[a]["throat_loading"] - rows[-a]["throat_loading"] for a in (0.2, 0.4, 0.6)]
    if shape == "flat-eta":
        assert all(d == 0.0 for d in diffs), f"flat island must be EXACTLY even, got {diffs}"
        assert all(rows[a]["m"] == rows[-a]["m"] for a in (0.2, 0.4, 0.6))
    else:
        assert all(d > 0.0 for d in diffs), f"shaped island should NOT be even, got {diffs}"
        assert max(diffs) > 1e-3, "asymmetry too small to be a real contrast"


# ======================================================================================
# GATE 6 — THE ARTIFACT IS NEVER THE CEILING (P-A2, the shape-robust SIGN claim)
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("shape", list(SHAPES))
def test_the_throat_binds_before_solve_n_bracket_on_every_shape(shape):
    """Rung 53 conceded its authority ceiling was `solve_n`'s speed-line bracket — "a
    map-validity edge", i.e. an ARTIFACT. Once the throat is modelled that artifact is never
    what stops the stator: v_ch < v_edge everywhere at C >= 0.80. The LEVEL of C_edge is
    disclaimed (P-A1 measured 0.63..0.78 across shapes — it is a threshold on an artifact and
    has no reason to be robust); the SIGN is the claim.

    THE LOAD-BEARING ASSERTION IS THE NEGATIVE ONE, `binds != "edge"`. `throat_before_edge`
    also holds 20/20 but is weaker and says something different: on `steep` the incidence peak
    is INSIDE the throat (v_peak < v_ch < v_edge), so there the PEAK binds and the throat is
    merely also present. The spec states it that way."""
    gas = _cpg_gas()
    design = _design(gas)
    # Pin the scan resolution: the published 20/20 is measured at this step, and its tightest
    # cell (steep @1500, v_ch 0.993 vs v_edge 1.12) is the first that would flip if it changed.
    assert VariableStatorMatcher._V_STEP == 0.04, (
        "the 20/20 claim in docs/rung54-spec.md is measured at _V_STEP = 0.04; re-measure it "
        "before changing the scan resolution")
    m = _shaped(gas, shape, C=0.80, design=design)
    for T in THROTTLE:
        a = m.authority_ceiling(FLIGHT, T, "lp")
        assert a["v_ch"] is not None, f"{shape} @{T}: throat unreachable within the scan"
        assert a["binds"] != "edge", (          # <- the claim
            f"{shape} @{T}: the ARTIFACT bound (v_edge={a['v_edge']:.3f})")
        assert a["throat_before_edge"], (       # <- the weaker corroboration
            f"{shape} @{T}: v_ch={a['v_ch']:.3f} did not beat v_edge={a['v_edge']:.3f}")
        assert a["c_edge"] < 0.90, f"{shape} @{T}: C_edge={a['c_edge']:.4f} (P-A2)"


# ======================================================================================
# GATE 7 — P-C2: THE TURNING POINT IS REACHED (rung 53's concession, corrected)
# ======================================================================================

@pytest.mark.slow
def test_the_incidence_peak_is_interior_on_some_shapes_and_not_others():
    """Rung 53 § Concessions: "The incidence benefit SATURATES in v and does not turn back
    ... (The apparent turning point that this algebra suggests is *not* reached.)" TRUE on
    the shape rung 53 measured, FALSE on others — the rung-28 shape, verdict kept and reason
    corrected. Asserted as a CONTRAST so neither half can be vacuous."""
    gas = _cpg_gas()
    design = _design(gas)
    flat = _shaped(gas, "flow/press", design=design).authority_ceiling(FLIGHT, 1000.0, "lp")
    assert not flat["peak_interior"], (
        "flow/press is where rung 53 measured: its walk must still run to the edge")
    for shape, min_drop in (("tilted", 5e-3), ("steep", 3e-2)):
        a = _shaped(gas, shape, design=design).authority_ceiling(FLIGHT, 1000.0, "lp")
        assert a["peak_interior"], f"{shape}: expected an INTERIOR incidence peak"
        assert a["v_peak"] < a["v_edge"], f"{shape}: peak not strictly inside the band"
        assert a["m_i_peak"] - a["m_i_edge"] > min_drop, (
            f"{shape}: the turn-back is immaterial ({a['m_i_peak']-a['m_i_edge']:.5f}) — "
            f"rung 53's concession would effectively stand")


@pytest.mark.slow
def test_rung_53s_schedule_ceases_to_exist_inside_the_envelope():
    """The consequence for rung 53's P7 payoff object. Where the incidence peak falls short
    of the DESIGN incidence there is no schedule at all — the stator cannot restore design
    incidence at any feasible setting. Rung 53 disclosed finite authority (verdict kept) but
    attributed it to the map-validity edge (reason corrected)."""
    gas = _cpg_gas()
    design = _design(gas)
    rows = {r["Tt4"]: r for r in _shaped(gas, "steep", design=design).schedule_throat(
        FLIGHT, [1200.0, 1000.0], "lp")}
    assert rows[1200.0]["exists"], "steep still has a schedule at Tt4=1200"
    assert not rows[1000.0]["exists"], "steep must lose its schedule by Tt4=1000"
    assert rows[1000.0]["vsv_star"] is None
    assert rows[1000.0]["tan_b1_min"] > rows[1000.0]["tan_b1_design"], (
        "the reason must be that design incidence is UNREACHABLE, not a solver failure")
    # and it survives where rung 53 measured, so the finding is a contrast not a breakage
    ok = _shaped(gas, "flow/press", design=design).schedule_throat(
        FLIGHT, [1200.0, 1000.0], "lp")
    assert all(r["exists"] for r in ok)


# ======================================================================================
# GATE 8 — P-C3: rung 54's root is immune to the ladder rung 53 relies on
# ======================================================================================

@pytest.mark.slow
def test_rung54_root_finds_a_schedule_rung53s_doubling_ladder_walks_over():
    """Rung 53's `incidence_schedule` justifies its doubling ladder with "the residual is
    monotone decreasing in v". Where the peak is interior that premise fails and the ladder
    steps OVER the root. Rung 54 brackets off a scan and is immune. (Rung 53's own published
    table is the flow/press shape, where the premise holds — so its numbers stand.)"""
    gas = _cpg_gas()
    design = _design(gas)
    m = _shaped(gas, "steep", design=design)
    row = m.schedule_throat(FLIGHT, [1200.0], "lp")[0]
    assert row["exists"] and row["vsv_star"] == pytest.approx(0.909, abs=0.01)
    assert row["tan_b1"] == pytest.approx(row["tan_b1_design"], abs=1e-9), (
        "the root must actually satisfy the design-incidence condition")
    # the ladder, on the same point, gives up
    with pytest.raises(AssertionError, match="does not bracket"):
        m.incidence_schedule(FLIGHT, [1200.0], spool="lp",
                             v_hi=0.98 * row["v_edge"])


def test_rung54_root_agrees_with_rung53_wherever_the_ladder_succeeds():
    """The other half of the contrast: no silent divergence from the shipped rung."""
    gas = _cpg_gas()
    design = _design(gas)
    m = _shaped(gas, "flow/press", design=design)
    for T in (1300.0, 1100.0):
        mine = m.schedule_throat(FLIGHT, [T], "lp")[0]["vsv_star"]
        theirs = m.incidence_schedule(FLIGHT, [T], spool="lp")[0]["vsv_star"]
        assert mine == pytest.approx(theirs, abs=1e-9), f"Tt4={T}: {mine} vs {theirs}"


# ======================================================================================
# GATE 9 — THE RACE, and the exposure split it inherits
# ======================================================================================

@pytest.mark.slow
def test_the_schedule_crosses_the_design_throat_loading_inside_the_envelope():
    """A CONSTANT-FREE boundary. As power falls the schedule's demand v* RISES while the flow
    m FALLS, so X(v*) is a race. Above the crossing the schedule asks LESS of the throat than
    the DESIGN point, so it is feasible for EVERY row whatever its C; below, feasibility
    becomes C-dependent. Bracketed, not pinned (the level rides on the disclosed shape)."""
    m = _shaped(_cpg_gas(), "flow/press")
    rows = {r["Tt4"]: r for r in m.schedule_throat(
        FLIGHT, [1200.0, 1000.0, 900.0, 870.0, 860.0, 800.0], "lp")}
    assert all(r["exists"] for r in rows.values())
    assert rows[870.0]["throat_loading"] < 1.0 < rows[860.0]["throat_loading"], (
        "the design-loading crossing must be bracketed by Tt4 = 870 / 860")
    # rung 53's ENTIRE published band sits above the crossing: inert there for any row
    for T in (1200.0, 1000.0):
        assert rows[T]["c_min"] > 1.0, (
            f"Tt4={T}: c_min={rows[T]['c_min']:.4f} — must exceed 1, i.e. no row can choke")
    # and the race has an interior minimum: the throttle wins, then the schedule does
    assert rows[1200.0]["throat_loading"] < rows[1000.0]["throat_loading"]
    assert rows[1200.0]["throat_loading"] < 1.0


@pytest.mark.slow
def test_the_capacity_ceiling_is_a_pure_lp_object():
    """The exposure split, INHERITED not new: rung 53's P7 needs v*_LP >> v*_HP, and the
    throat cost goes as sqrt(1+v^2), so the LP eats it quadratically faster. The HP's demand
    FALLS monotonically and never approaches its throat."""
    m = _shaped(_cpg_gas(), "flow/press")
    hp = [r for r in m.schedule_throat(FLIGHT, [1400.0, 1200.0, 1000.0, 800.0], "hp")]
    assert all(r["exists"] for r in hp)
    loads = [r["throat_loading"] for r in hp]
    assert loads == sorted(loads, reverse=True), f"HP demand should FALL, got {loads}"
    assert max(loads) < 1.0 and min(loads) < 0.75
    lp = {r["Tt4"]: r for r in m.schedule_throat(FLIGHT, [1400.0, 800.0], "lp")}
    assert lp[800.0]["throat_loading"] > lp[1400.0]["throat_loading"], "LP must turn back up"
    assert lp[800.0]["vsv_star"] > 3.0 * hp[-1]["vsv_star"]


# ======================================================================================
# GATE 10 — CYCLE UNTOUCHED
# ======================================================================================

def test_cycle_untouched_design_run_is_bit_for_bit_rung6():
    """Rung 54 adds a diagnostic field and pure read methods; the default single-spool design
    path must be untouched, as at every rung since 7."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, 1600.0, FLIGHT.p0, **dict(
        pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92, eta_m=0.99, pi_n=0.98))
    r = eng.run(FLIGHT, 1.0)
    assert r.performance.specific_thrust > 0.0
    # the new field is inert by default, and does not disturb rung 53's flatness rule
    assert ComponentMap().capacity == 0.0 and ComponentMap().is_flat()


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
