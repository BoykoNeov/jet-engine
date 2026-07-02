"""Rung-13 verification: the resolved mixing PDF — a mean-preserving β-PDF of mixture fraction
that replaces rung-12's parameterised SEGREGATION (w(C)→a continuous distribution), pinning the
Holdeman emissions optimum AT C_opt ((H/S)² shift) from composition variance alone, and making the
underlying lesson exact.

MECHANISM SEPARATION (the sharp teaching point). Rung-12's over-penetration CLIMB came from the
DWELL effect (an absolute, growing τ_core — a TIME mechanism). Rung 13 isolates the COMPOSITION
mechanism and drops the quench chain, so it structurally CANNOT reproduce that climb — it pins the
optimum LOCATION (min at C_opt) but the far-over-penetration flank DESCENDS (the humped ⟨EI⟩(g)).
Composition variance pins the optimum; the dwell effect makes the climb; combining them (the PDF
through the quench) is the rung-15 seam.

The lesson, framed CORRECTLY (not generic "convexity/Jensen"): the NO-vs-φ bell is convex on its
flanks but CONCAVE at the peak, so NO global convexity exists. What is true: NO is sharply PEAKED at
stoich, so spreading the local φ around a fixed mean (unmixedness) RAISES the mean NO whenever the
mean is OFF-stoich (our lean dilution mean), and REVERSES sign at a stoich mean. Segregation is the
β-PDF width g(C)=min(g_max, k_g·|ln(C/C_opt)|) — the SAME kinked Holdeman distance as rung-12's w.

    ⟨EI⟩(g) = ∫ EI_bell(φ(ξ)) · P_β(ξ; ξ̄, g) dξ ,   ξ̄ = the overall (lean) mixture fraction

Gates (docs/rung13-spec.md), priority order:

1. reduce (LOAD-BEARING, exact) — pdf=None is code-path-identical rung 12 (all rung-13 fields None;
   the whole rung-1..12 suite stays green); and g→0 is the well-mixed point value EI(φ_overall).
   (No bit-for-bit reduce to the two-stream model is claimed — it is a DIFFERENT closure.)
2. mean-preservation (THE deliverable) — ⟨ξ⟩≈ξ̄ and variance≈g·ξ̄(1−ξ̄) across the g-range (machine-
   precision in the singular a<1 regime via the u=ξ^a transform; <0.2% in the a≥1 regime).
3. the OPTIMUM (THE lesson) — ⟨EI⟩ is pinned to a sharp minimum AT C_opt (perfect mixing ⇒ ≈0);
   both immediate flanks lift by orders (segregation). ⟨EI⟩(g) is HUMPED (bimodal-ward descent) —
   so the far-over-penetration flank descends (the dwell climb is rung-12's, absent here).
4. the optimum is AT the Holdeman group C_opt — J_min == J_opt=(C_opt·H/S)², shifting as (H/S)².
5. the convexity jump is large + correct-signed for the lean mean AND reverses sign at a stoich mean.
6. g(C) is the segregation — 0 at C_opt, rising (kinked) on both flanks, symmetric in ln C, capped.
7. cycle untouched — a pdf zoned_nox call must not perturb station 4 (pure diagnostic).
8. require-mixing + mutual-exclusivity(unmixedness) + MixingPDF positivity/range guards.

The helper `_pdf_ei` mirrors `Gas.zoned_nox`'s PDF math on a bell built ONCE (EI(ξ) is J-independent),
and `test_zoned_nox_matches_pdf_helper` pins it to the production path at one point — so the sweeps
exercise the SAME arithmetic without rebuilding the equilibrium-heavy bell per J.

Run with `python tests/test_rung13.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, Unmixedness, MixingPDF, _F_STOICH, _HF_FUEL_DEFAULT,
    _bell_interpolator, _beta_pdf_nodes_weights, _ideal_bell_ei,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
# Coarse grids — the gates test SHAPE + DIRECTION (turn-up, pin, shift, sign), not digits (project
# ethos; same rationale as rung 10/11/12). n_bell≈120 / n_quad≈160 settle the shape.
_NG = 32          # finite-quench trajectory resolution (only the bulk reference uses it)
_NB, _NQ = 120, 160


_DP_CACHE = None


def _design_point():
    """Build the equilibrium engine once and read the (derived) station-3/4 state (cached).
    NO is trace → the cycle is bit-for-bit rung 6; every rung-13 gate uses the same design point."""
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        _DP_CACHE = (g, st3.Tt, st4.Tt, st4.far, st4.pt)
    return _DP_CACHE


_BELL_CACHE = {}


def _bell(g, Tt3, p):
    """The IDEAL bell EI(ξ), built ONCE (J-independent) and cached — the object a J-sweep reuses."""
    key = (round(Tt3, 6), round(p, 3))
    if key not in _BELL_CACHE:
        hf = g.hf_fuel_molar if g.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
        _BELL_CACHE[key] = _bell_interpolator(p, Tt3, hf, _TAU, n_bell=_NB)
    return _BELL_CACHE[key]


def _pdf_ei(bell, xibar, g_seg, n_quad=_NQ):
    """⟨EI⟩ over the β-PDF on a prebuilt bell — mirrors `Gas.zoned_nox`'s PDF math (a fast path for
    the J sweeps). g→0 short-circuits to the well-mixed point value, as production does."""
    if g_seg <= 1e-9:
        return bell(xibar)
    nodes, w = _beta_pdf_nodes_weights(xibar, g_seg, n_quad=n_quad)
    return sum(wi * bell(x) for wi, x in zip(w, nodes))


def _argmin(vals):
    return min(range(len(vals)), key=lambda i: vals[i])


def _j_opt(pdf):
    """The uniformity optimum J_opt where C=(S/H)√J_opt = C_opt (H=0.10, the JetMixing default)."""
    return (pdf.C_opt * JetMixing(J=1.0).H / pdf.S) ** 2


# --------------------------------------------------------------------------- #
# GATE 1 — reduce: pdf=None is code-path-identical rung 12; g→0 is the point.  #
# --------------------------------------------------------------------------- #
def test_reduce_pdf_none_is_rung12_path():
    # pdf=None (the default) must leave EVERY rung-13 field None and match the rung-12/11 call
    # bit-for-bit (the whole rung-1..12 suite staying green already pins the shared path).
    g, Tt3, Tt4, far, p = _design_point()
    for J in (9.0, 36.0):
        a = g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=J), quench_ngrid=_NG)
        b = g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=J), pdf=None, quench_ngrid=_NG)
        for s in (a, b):
            assert s.pdf is None and s.ei_no_pdf is None and s.g_seg is None and s.C_holdeman is None
        assert a.ei_no_quenched == b.ei_no_quenched and a.max_a_quench == b.max_a_quench


def test_reduce_g_to_zero_is_well_mixed_point_value():
    # g→0 ⇒ the β-PDF is a delta at ξ̄ ⇒ ⟨EI⟩ = EI(φ_overall), the well-mixed point value. This is
    # what the turn-up minimum pins to at C_opt (where g(C_opt)=0).
    g, Tt3, Tt4, far, p = _design_point()
    bell = _bell(g, Tt3, p)
    xibar = far / (1.0 + far)
    assert _pdf_ei(bell, xibar, 0.0) == bell(xibar)
    # a jet placed EXACTLY at C_opt has g=0, so the production ei_no_pdf is the point value too —
    # the EXACT ideal bell at the overall mean (production's delta short-circuit uses the exact
    # `_ideal_bell_ei`, not the interpolant, so compare to the exact value here).
    hf = g.hf_fuel_molar if g.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
    exact = _ideal_bell_ei(far, p, Tt3, hf, _TAU)
    pdf = MixingPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    s = g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=_j_opt(pdf)), pdf=pdf, quench_ngrid=_NG)
    assert s.g_seg == 0.0 and abs(s.C_holdeman - pdf.C_opt) < 1e-12
    assert s.ei_no_pdf == exact, f"at C_opt (g=0) production must return the exact point value {exact}"


def test_zoned_nox_matches_pdf_helper():
    # Pin the fast sweep helper to the PRODUCTION zoned_nox path (same n_bell/n_quad), so the sweep
    # gates below exercise the SAME arithmetic the production code does.
    g, Tt3, Tt4, far, p = _design_point()
    bell = _bell(g, Tt3, p)
    xibar = far / (1.0 + far)
    pdf = MixingPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    m = JetMixing(J=36.0)
    s = g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=m, pdf=pdf, quench_ngrid=_NG)
    g_seg = pdf.segregation(pdf.C(m))
    h = _pdf_ei(bell, xibar, g_seg)
    assert abs(s.ei_no_pdf - h) < 1e-9 * max(h, 1e-12)
    assert abs(s.C_holdeman - pdf.C(m)) < 1e-12 and abs(s.g_seg - g_seg) < 1e-12


# --------------------------------------------------------------------------- #
# GATE 2 — mean-preservation: the quadrature integrates at ξ̄ (the deliverable).#
# --------------------------------------------------------------------------- #
def test_quadrature_preserves_mean_and_variance():
    # The mean-preserving closure MUST integrate at ξ̄ — the u=ξ^a transform makes this machine-exact
    # in the singular a<1 regime, and the CENTERED window keeps the a≥1 (near-delta) regime <1% all
    # the way down to the delta floor. (The production asserts bind this every call; here we check it
    # directly across the g-range.) The SMALL-g values matter: production g(C)=k_g·|ln(C/C_opt)| → 0
    # continuously near C_opt, so a fine J-sweep through J_opt hits arbitrarily small g — the window
    # must stay resolved there or the standing assertion would crash a legitimate run.
    g, Tt3, Tt4, far, p = _design_point()
    xibar = far / (1.0 + far)
    for g_seg in (1e-6, 1e-4, 1e-3, 0.005, 0.01, 0.02, 0.05, 0.10, 0.20, 0.30):
        nodes, w = _beta_pdf_nodes_weights(xibar, g_seg, n_quad=_NQ)
        mean_xi = sum(wi * x for wi, x in zip(w, nodes))
        var_xi = sum(wi * (x - xibar) ** 2 for wi, x in zip(w, nodes))
        var_tgt = g_seg * xibar * (1.0 - xibar)
        assert abs(mean_xi - xibar) <= 0.01 * xibar, f"g={g_seg}: ⟨ξ⟩={mean_xi} vs ξ̄={xibar}"
        assert abs(var_xi - var_tgt) <= 0.05 * var_tgt, f"g={g_seg}: var {var_xi} vs {var_tgt}"


# --------------------------------------------------------------------------- #
# GATE 3 — the optimum: a sharp emissions minimum PINNED AT C_opt, flanks up.  #
# --------------------------------------------------------------------------- #
def test_emissions_minimum_pinned_at_c_opt_flanks_rise():
    # THE rung-13 lesson: composition variance pins the emissions minimum AT the Holdeman optimum.
    # At C_opt the jet is perfectly mixed (g=0 ⇒ delta ⇒ uniform lean ⇒ ≈0 NO); a small step to
    # EITHER flank segregates the mixture and lifts ⟨EI⟩ by ORDERS (the convexity jump, localized).
    #
    # This is NOT rung-12's "falls then rises" bowl: that over-penetration CLIMB came from the DWELL
    # effect (an absolute, growing τ_core), a TIME mechanism rung 13 deliberately drops. Composition
    # variance ALONE pins the optimum LOCATION; the far-over-penetration flank instead DESCENDS (the
    # humped ⟨EI⟩(g) — tested in `test_mean_ei_is_humped_in_g` below). Combining both mechanisms
    # (the PDF through the quench) is the rung-15 seam.
    g, Tt3, Tt4, far, p = _design_point()
    bell = _bell(g, Tt3, p)
    xibar = far / (1.0 + far)
    pdf = MixingPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    H = JetMixing(J=1.0).H
    J_opt = _j_opt(pdf)                                          # C = C_opt exactly (g=0)
    # bracket C_opt tightly on both flanks (under- and over-penetration).
    ei_opt = _pdf_ei(bell, xibar, pdf.segregation((pdf.S / H) * math.sqrt(J_opt)))
    ei_under = _pdf_ei(bell, xibar, pdf.segregation((pdf.S / H) * math.sqrt(J_opt / 1.3)))
    ei_over = _pdf_ei(bell, xibar, pdf.segregation((pdf.S / H) * math.sqrt(J_opt * 1.3)))
    assert ei_opt < 1e-3, f"at C_opt the jet is perfectly mixed ⇒ ⟨EI⟩≈0 (well-mixed lean), got {ei_opt}"
    assert ei_under > 1e3 * max(ei_opt, 1e-12) and ei_over > 1e3 * max(ei_opt, 1e-12), \
        f"both immediate flanks must lift ⟨EI⟩ by orders (segregation): under={ei_under}, over={ei_over}"


def test_mean_ei_is_humped_in_g():
    # ⟨EI⟩(g) is NON-monotone in the segregation: it PEAKS at moderate g (~0.02) and DESCENDS toward
    # high g. Real physics of a mixture-fraction PDF — at extreme segregation the β-PDF goes BIMODAL
    # (mass piles at pure-air ξ→0 and the rich cap, BOTH off the stoich peak), so ⟨EI⟩ falls again.
    # This is WHY the far-over-penetration flank descends (gate 3): a TESTED feature, not a surprise.
    g, Tt3, Tt4, far, p = _design_point()
    bell = _bell(g, Tt3, p)
    xibar = far / (1.0 + far)
    ei_lo = _pdf_ei(bell, xibar, 0.02)                          # near the hump peak
    ei_hi = _pdf_ei(bell, xibar, 0.30)                          # far into segregation (bimodal-ward)
    assert ei_lo > ei_hi, f"⟨EI⟩(g) must be humped (peak at low g, descending): ei(0.02)={ei_lo}, ei(0.30)={ei_hi}"


# --------------------------------------------------------------------------- #
# GATE 4 — the optimum is AT the Holdeman group C_opt, shifting as (H/S)².     #
# --------------------------------------------------------------------------- #
def test_optimum_is_at_holdeman_c_opt_and_shifts_as_H_over_S_squared():
    # The recovered optimum sits AT the Holdeman uniformity group: J_min == J_opt=(C_opt·H/S)². So
    # shrinking the spacing S moves the min up EXACTLY as (H/S)² — the group made literal (the kinked
    # g(C) pins the min at C_opt for every S, where g=0 ⇒ the well-mixed lean value ≈0).
    g, Tt3, Tt4, far, p = _design_point()
    bell = _bell(g, Tt3, p)
    xibar = far / (1.0 + far)
    H = JetMixing(J=1.0).H
    for S in (0.0625, 0.0500):
        pdf = MixingPDF(S=S, n_bell=_NB, n_quad=_NQ)
        J_opt = _j_opt(pdf)                                    # 16 (S=.0625), 25 (S=.05)
        Js = [J_opt / 4, J_opt / 2, J_opt, 2 * J_opt, 4 * J_opt]   # C = C_opt·{.5,.707,1,1.41,2}
        eis = [_pdf_ei(bell, xibar, pdf.segregation((S / H) * math.sqrt(J))) for J in Js]
        imin = _argmin(eis)
        assert Js[imin] == J_opt, f"S={S}: EI-min must sit AT J_opt={J_opt}, got J={Js[imin]}: {eis}"


# --------------------------------------------------------------------------- #
# GATE 5 — the convexity jump: large + correct-signed lean, reversing at stoich.#
# --------------------------------------------------------------------------- #
def test_convexity_jump_lean_and_sign_reversal_at_stoich():
    # For the LEAN mean, spreading raises ⟨EI⟩ by orders (the stoich-ward tail samples the bell peak
    # while EI(ξ̄) sits tiny in the lean wing). For a STOICH mean, spreading LOWERS ⟨EI⟩ (mass moves
    # OFF the peak) — the sign reversal that certifies the "peaked-at-stoich × off-stoich-mean"
    # framing over a loose "convexity ⇒ always raises" claim.
    g, Tt3, Tt4, far, p = _design_point()
    bell = _bell(g, Tt3, p)
    xibar_lean = far / (1.0 + far)
    far_st = _F_STOICH
    xibar_stoich = far_st / (1.0 + far_st)
    # lean: a big jump up
    assert _pdf_ei(bell, xibar_lean, 0.10) > 1e3 * bell(xibar_lean), \
        "lean mean: segregation must raise ⟨EI⟩ by orders (peaked bell × off-peak mean)"
    # stoich: spreading LOWERS it
    assert bell(xibar_stoich) > 5.0, "sanity: the stoich point value should be large (near the peak)"
    assert _pdf_ei(bell, xibar_stoich, 0.10) < bell(xibar_stoich), \
        "stoich mean: segregation must LOWER ⟨EI⟩ (sign reversal — moves mass off the peak)"


# --------------------------------------------------------------------------- #
# GATE 6 — g(C) is the segregation: 0 at C_opt, rising (kinked) on both flanks.#
# --------------------------------------------------------------------------- #
def test_segregation_is_kinked_zero_at_optimum():
    pdf = MixingPDF()
    assert pdf.segregation(pdf.C_opt) == 0.0, "g must be exactly 0 at C_opt (perfect mixing ⇒ delta)"
    lo, hi = pdf.segregation(pdf.C_opt / 1.3), pdf.segregation(pdf.C_opt * 1.3)
    assert lo > 0.0 and hi > 0.0, f"g must rise on BOTH flanks (under/over-penetration): {lo}, {hi}"
    # symmetric in ln C (an L1 |ln| distance).
    assert abs(pdf.segregation(pdf.C_opt / 1.4) - pdf.segregation(pdf.C_opt * 1.4)) < 1e-12
    # KINKED: non-zero slope just off C_opt (what pins the EI-min AT C_opt).
    assert pdf.segregation(pdf.C_opt * 1.05) > 0.0
    # capped at g_max.
    assert pdf.segregation(pdf.C_opt * 1e6) == pdf.g_max


# --------------------------------------------------------------------------- #
# GATE 7 — cycle untouched by a pdf quench.                                    #
# --------------------------------------------------------------------------- #
def test_cycle_untouched_by_pdf_call():
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)

    r1 = run()
    st3, st4 = r1.stations["3"], r1.stations["4"]
    far1, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    g.zoned_nox(far1, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=36.0),
                pdf=MixingPDF(n_bell=_NB, n_quad=_NQ), quench_ngrid=_NG)
    assert run().stations["4"].far == far1, "pdf call perturbed the cycle far — must stay rung-6"


# --------------------------------------------------------------------------- #
# GATE 8 — require-mixing + mutual-exclusivity + positivity guards.            #
# --------------------------------------------------------------------------- #
def test_pdf_requires_mixing():
    g, Tt3, Tt4, far, p = _design_point()
    try:
        g.zoned_nox(far, Tt3, Tt4, p, 1.5, pdf=MixingPDF(), quench_ngrid=_NG)  # no mixing
    except AssertionError:
        return
    raise AssertionError("pdf without mixing must be rejected (needs J and H for C)")


def test_pdf_unmixedness_mutually_exclusive():
    g, Tt3, Tt4, far, p = _design_point()
    try:
        g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=16.0),
                    pdf=MixingPDF(), unmixedness=Unmixedness(), quench_ngrid=_NG)
    except AssertionError:
        return
    raise AssertionError("pdf AND unmixedness together must be rejected (same physics, exclusive)")


def test_mixingpdf_positivity_guards():
    MixingPDF()                                          # defaults accepted
    MixingPDF(k_g=0.0)                                   # k_g=0 ⇒ g≡0 (well-mixed), allowed
    for bad in (dict(S=0.0), dict(S=-0.1), dict(C_opt=0.0), dict(k_g=-0.1),
                dict(g_max=0.0), dict(g_max=1.0), dict(g_max=1.5), dict(n_bell=1), dict(n_quad=0)):
        try:
            MixingPDF(**bad)
        except AssertionError:
            continue
        raise AssertionError(f"MixingPDF({bad}) should be rejected (positivity/range guard)")


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-13 gates passed")
