"""Rung-21 verification: super-equilibrium O through the IDEAL-BELL PDF integrals — discharging the
last equilibrium-O seam on the primary/PDF side.

Rung 20 lifted everything through `_quench_no` (bulk/core/per-pocket/clamp) but LEFT the three
ideal-bell composition integrals on equilibrium O and FORBADE combining them with super_eq_o, because
`ei_no_pdf_quench = term1 + term2` would be a HALF-LIFTED HYBRID (lifted term1 + eq-O term2). Rung 21
threads the SAME Westenberg m(T) through the ideal bell so BOTH terms lift — the hybrid dissolves and
super_eq_o combines with EVERY closure.

THE LOAD-BEARING RESULT (measured before the lesson; docs/plans/rung21-anchor-ideal-bell-lift.md): the
ideal-bell PDF lift is ≈×1.15 — the rung-20 INVERSION generalized to composition variance. The bell
EI(φ) is PEAKED near stoich (φ≈0.97, T≈2424 K) where m(T) is at its MINIMUM (~1.12); the β-PDF integral
is EI-weighted onto that peak, so it inherits the small lift — BELOW the primary ×1.28, and even
DECREASING with segregation g (more variance ⇒ more stoich-peak weight ⇒ smaller fractional lift). The
naive "the bell spans cool deep-lean pockets where m→1.9, so the lift is big" is WRONG: those pockets
carry negligible EI. NO forms where it is hottest, and m is smallest where it is hottest.

Gates (docs/rung21-spec.md), priority order:
1. reduce (LOAD-BEARING, exact) — super_eq_o=False is bit-for-bit rungs 13/15/18 (a defaulted kwarg).
2. the lift is MODEST & PEAK-CONCENTRATED — ⟨EI⟩_pdf lift ∈(1.10,1.20), strictly < the primary ×1.28
   and < the deep-lean point-value ×1.9.
3. the lift DECREASES with segregation g (the measured, counter-intuitive corollary).
4. the HYBRID RESOLVED — ei_no_pdf_quench composite lift ∈(1.10,1.20), BETWEEN its term1 (bulk) and
   term2 (ideal-bell) lifts; super_eq_o combines with pdf/pdf_quench/transported (no raise — the
   consciously inverted rung-20 forbid gate).
5. g→0 consistency — lifted ei_no_pdf at C_opt == the super-eq-O well-mixed point value.
6. shape preserved — eq-O and super-eq-O J-sweeps both minimise AT J_opt (a consistency lift, not a
   relocated feature).

Run with `python tests/test_rung21.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, MixingPDF, QuenchPDF, TransportedPDF, PocketQuenchPDF,
    _F_STOICH, _HF_FUEL_DEFAULT, _bell_interpolator, _beta_pdf_nodes_weights,
    _ideal_bell_ei, _pdf_mean_ei,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_PHI_P = 1.5
_NG = 24          # finite-quench trajectory resolution (only the bulk reference / pdf_quench use it)
_NB, _NQ = 120, 160

_DP_CACHE = None


def _design_point():
    """Build the equilibrium engine once and read the (derived) station-3/4 state (cached).
    NO is trace → the cycle is bit-for-bit rung 6; every rung-21 gate uses the same design point."""
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        _DP_CACHE = (g, st3.Tt, st4.Tt, st4.far, st4.pt)
    return _DP_CACHE


_BELL_CACHE = {}


def _bells(g, Tt3, p):
    """The eq-O and super-eq-O ideal bells EI(ξ), each built ONCE (J-independent) and cached."""
    key = (round(Tt3, 6), round(p, 3))
    if key not in _BELL_CACHE:
        hf = g.hf_fuel_molar if g.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
        eq = _bell_interpolator(p, Tt3, hf, _TAU, n_bell=_NB, super_eq_o=False)
        su = _bell_interpolator(p, Tt3, hf, _TAU, n_bell=_NB, super_eq_o=True)
        _BELL_CACHE[key] = (eq, su, hf)
    return _BELL_CACHE[key]


def _pdf_ei(bell, xibar, g_seg, n_quad=_NQ):
    """⟨EI⟩ over the β-PDF on a prebuilt bell (mirrors `_pdf_mean_ei`'s math). g→0 ⇒ the point value."""
    if g_seg <= 1e-9:
        return bell(xibar)
    nodes, w = _beta_pdf_nodes_weights(xibar, g_seg, n_quad=n_quad)
    return sum(wi * bell(x) for wi, x in zip(w, nodes))


def _argmin(vals):
    return min(range(len(vals)), key=lambda i: vals[i])


def _j_opt(S, C_opt=2.5):
    """The uniformity optimum J_opt where C=(S/H)√J_opt = C_opt (H=0.10, the JetMixing default)."""
    return (C_opt * JetMixing(J=1.0).H / S) ** 2


# --------------------------------------------------------------------------- #
# GATE 1 — reduce (LOAD-BEARING): super_eq_o=False is bit-for-bit rung 13/15/18. #
# --------------------------------------------------------------------------- #
def test_reduce_super_eq_o_false_is_bit_for_bit():
    # Helper level: the lifted bell integral with super_eq_o=False == the eq-O bell (a defaulted kwarg
    # gating a single m=1 multiply). Machine-exact.
    g, Tt3, Tt4, far, p = _design_point()
    eq, _su, hf = _bells(g, Tt3, p)
    xibar = far / (1.0 + far)
    for gseg in (0.0, 0.02, 0.1, 0.3):
        a = _pdf_mean_ei(far, Tt3, p, hf, _TAU, gseg, n_bell=_NB, n_quad=_NQ)
        b = _pdf_mean_ei(far, Tt3, p, hf, _TAU, gseg, n_bell=_NB, n_quad=_NQ, super_eq_o=False)
        assert a == b, f"super_eq_o=False must be bit-for-bit the default at g={gseg}: {a} vs {b}"

    # Production level: one pdf zoned_nox — default vs explicit super_eq_o=False are identical.
    m = JetMixing(J=36.0)
    d = g.zoned_nox(far, Tt3, Tt4, p, _PHI_P, mixing=m, pdf=MixingPDF(S=0.0625, n_bell=_NB, n_quad=_NQ),
                    quench_ngrid=_NG)
    e = g.zoned_nox(far, Tt3, Tt4, p, _PHI_P, mixing=m, pdf=MixingPDF(S=0.0625, n_bell=_NB, n_quad=_NQ),
                    super_eq_o=False, quench_ngrid=_NG)
    assert d.ei_no_pdf == e.ei_no_pdf, "production ei_no_pdf: super_eq_o=False must be bit-for-bit default"


# --------------------------------------------------------------------------- #
# GATE 2 — the lift is MODEST & PEAK-CONCENTRATED, below the primary.           #
# --------------------------------------------------------------------------- #
def test_lift_is_modest_peak_concentrated_below_primary():
    g, Tt3, Tt4, far, p = _design_point()
    eq, su, hf = _bells(g, Tt3, p)
    xibar = far / (1.0 + far)

    # ⟨EI⟩_pdf lift at a representative segregation (over-penetration flank).
    gseg = 0.05
    lift_pdf = _pdf_ei(su, xibar, gseg) / _pdf_ei(eq, xibar, gseg)
    assert 1.10 < lift_pdf < 1.20, f"ideal-bell PDF lift {lift_pdf:.3f} outside the measured (1.10,1.20)"

    # the PRIMARY lift: m(T_p) at φ_p=1.5 (the rung-19 headline), via the ideal bell at that φ.
    fl = _PHI_P * _F_STOICH
    lift_primary = (_ideal_bell_ei(fl, p, Tt3, hf, _TAU, super_eq_o=True)
                    / _ideal_bell_ei(fl, p, Tt3, hf, _TAU, super_eq_o=False))
    assert 1.25 < lift_primary < 1.30, f"primary lift {lift_primary:.3f} != rung-19 ~×1.28"
    assert lift_pdf < lift_primary, (
        f"the ideal-bell PDF lift ({lift_pdf:.3f}) must be BELOW the primary ({lift_primary:.3f}) — "
        "the bell peak is HOTTER than the φ_p=1.5 flame, so m is smaller there (the rung-20 inversion)"
    )

    # the deep-lean POINT VALUE (g→0) carries the LARGEST lift (cool flame ⇒ m large) but negligible EI.
    lift_point = _pdf_ei(su, xibar, 0.0) / _pdf_ei(eq, xibar, 0.0)
    assert lift_point > lift_primary > lift_pdf, (
        f"expected point ({lift_point:.3f}) > primary ({lift_primary:.3f}) > PDF ({lift_pdf:.3f}): "
        "the lean point value is largest, the EI-weighted PDF smallest"
    )


# --------------------------------------------------------------------------- #
# GATE 3 — the lift DECREASES with segregation g (the measured corollary).      #
# --------------------------------------------------------------------------- #
def test_lift_decreases_with_segregation():
    g, Tt3, Tt4, far, p = _design_point()
    eq, su, hf = _bells(g, Tt3, p)
    xibar = far / (1.0 + far)
    # A narrow PDF (small g) samples near the lean mean (cool ⇒ large m); a broad PDF (large g) pulls
    # mass onto the stoich peak (hot ⇒ small m). Use a wide g-span (0.005 vs 0.30) for a durable margin.
    lift_small = _pdf_ei(su, xibar, 0.005) / _pdf_ei(eq, xibar, 0.005)
    lift_large = _pdf_ei(su, xibar, 0.30) / _pdf_ei(eq, xibar, 0.30)
    assert lift_large < lift_small, (
        f"lift must DECREASE with segregation (more variance ⇒ more stoich-peak weight ⇒ smaller "
        f"fractional lift): g=0.30 → {lift_large:.4f} vs g=0.005 → {lift_small:.4f}"
    )


# --------------------------------------------------------------------------- #
# GATE 4 — the HYBRID RESOLVED: composite lift between its two terms; combines.  #
# --------------------------------------------------------------------------- #
def test_hybrid_resolved_and_combines():
    g, Tt3, Tt4, far, p = _design_point()
    m = JetMixing(J=36.0)

    def run(**kw):
        return g.zoned_nox(far, Tt3, Tt4, p, _PHI_P, mixing=m, quench_ngrid=_NG, **kw)

    qp = dict(pdf_quench=QuenchPDF(S=0.0625, n_bell=_NB, n_quad=_NQ))
    pd = dict(pdf=MixingPDF(S=0.0625, n_bell=_NB, n_quad=_NQ))
    # combining super_eq_o with the ideal-bell closures MUST NOT raise (the inverted rung-20 gate).
    a_eq, a_su = run(**qp), run(super_eq_o=True, **qp)
    p_eq, p_su = run(**pd), run(super_eq_o=True, **pd)
    run(super_eq_o=True, transported=TransportedPDF(S=0.0625, n_bell=_NB, n_quad=_NQ))  # no raise

    lift_composite = a_su.ei_no_pdf_quench / a_eq.ei_no_pdf_quench
    lift_bulk = a_su.ei_no_quenched / a_eq.ei_no_quenched     # term1 (rung-20 bulk)
    lift_bell = p_su.ei_no_pdf / p_eq.ei_no_pdf               # term2's ideal-bell integral
    assert 1.10 < lift_composite < 1.20, f"pdf_quench composite lift {lift_composite:.3f} outside (1.10,1.20)"
    # the composite sits BETWEEN its two terms — the measured proof both now carry m(T) (no hybrid).
    lo, hi = sorted((lift_bulk, lift_bell))
    assert lo - 1e-9 <= lift_composite <= hi + 1e-9, (
        f"composite lift {lift_composite:.4f} must sit between term1 (bulk {lift_bulk:.4f}) and term2 "
        f"(bell {lift_bell:.4f}) — that BOTH lift is what dissolves the rung-20 half-lifted hybrid"
    )


# --------------------------------------------------------------------------- #
# GATE 5 — g→0 consistency: lifted ei_no_pdf at C_opt == super-eq-O point value. #
# --------------------------------------------------------------------------- #
def test_g0_consistency_is_super_eq_point_value():
    g, Tt3, Tt4, far, p = _design_point()
    _eq, _su, hf = _bells(g, Tt3, p)
    pdf = MixingPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    J_opt = _j_opt(pdf.S, pdf.C_opt)                          # C = C_opt exactly ⇒ g = 0
    s = g.zoned_nox(far, Tt3, Tt4, p, _PHI_P, mixing=JetMixing(J=J_opt), pdf=pdf,
                    super_eq_o=True, quench_ngrid=_NG)
    assert s.g_seg == 0.0 and abs(s.C_holdeman - pdf.C_opt) < 1e-12
    point = _ideal_bell_ei(far, p, Tt3, hf, _TAU, super_eq_o=True)
    assert s.ei_no_pdf == point, (
        f"at C_opt (g=0) the lifted ei_no_pdf must be the SUPER-eq-O well-mixed point value {point}, "
        f"got {s.ei_no_pdf} (the g→0 delta limit must thread the same flag)"
    )


# --------------------------------------------------------------------------- #
# GATE 6 — SHAPE preserved: eq-O and super-eq-O both minimise AT J_opt.          #
# --------------------------------------------------------------------------- #
def test_shape_preserved_optimum_at_jopt():
    g, Tt3, Tt4, far, p = _design_point()
    eq, su, hf = _bells(g, Tt3, p)
    xibar = far / (1.0 + far)
    S, H = 0.0625, JetMixing(J=1.0).H
    pdf = MixingPDF(S=S)
    J_opt = _j_opt(S, pdf.C_opt)
    Js = [J_opt / 4, J_opt / 2, J_opt, 2 * J_opt, 4 * J_opt]   # C = C_opt·{.5,.707,1,1.41,2}
    for bell, tag in ((eq, "eq-O"), (su, "super-eq-O")):
        eis = [_pdf_ei(bell, xibar, pdf.segregation((S / H) * math.sqrt(J))) for J in Js]
        assert Js[_argmin(eis)] == J_opt, (
            f"{tag}: the ⟨EI⟩_pdf minimum must stay pinned AT J_opt={J_opt} (the lift is shape-"
            f"preserving, not a relocated optimum), got J={Js[_argmin(eis)]}: {eis}"
        )


if __name__ == "__main__":
    for name in list(globals()):
        if name.startswith("test_"):
            print(f"--- {name}")
            globals()[name]()
    print("rung-21 gates passed")
