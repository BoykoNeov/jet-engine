"""Rung-15 verification: the PDF THROUGH the finite quench — rung-13's resolved mixture-fraction
β-PDF carried through the rung-10/12 dwell chain, so the two mixing mechanisms finally COMBINE.

Rungs 12–13 kept the mechanisms ISOLATED by design. Rung 12 had the DWELL (an absolute,
off-optimum-growing core residence — a TIME mechanism; the over-penetration flank CLIMBS) but a
two-lump composition split. Rung 13 had the COMPOSITION variance (a mean-preserving β-PDF; the optimum
LOCATION pinned AT C_opt) but on the IDEAL bell — it dropped the quench, so its optimum collapsed to
≈0 and its far flank DESCENDED. Rung 15 is the additive COMBINATION:

    ⟨EI⟩₁₅(J) = EI_bulk_quench(τ_mean(J))     [term 1: the rung-11 mean-field FLOOR, present at all C]
              + D(u(C)) · ⟨EI_bell⟩(g(C))     [term 2: the rung-13 β-PDF integral × a rung-12 dwell]

with g(C)=min(g_max,k_g·|ln(C/C_opt)|) (rung-13 segregation), u(C)=|ln(C/C_opt)| (rung-12 unmixedness),
and the dwell factor D(u)=τ_res·(1+b_u·u)/τ_ref rescaling the reference-τ bell EI to the pocket's
lingering dwell (EI ∝ τ, dormant clamp). The result is distinguishable from BOTH parents: the ≈0
rung-13 floor BECOMES the finite bulk quench NO, and the descending far flank CLIMBS again — while the
NONLINEAR bell keeps the STOICH-MEAN SIGN REVERSAL a lumped-dwell rung-12-in-disguise cannot.

Gates (docs/rung15-spec.md), priority order:

1. reduce (LOAD-BEARING) — pdf_quench=None is code-path-identical rung 13 (all rung-15 fields None; the
   whole rung-1..14 suite stays green); and at C_opt (g→0) ⟨EI⟩ = the FINITE bulk quench NO
   (ei_no_quenched), NOT rung-13's ≈0 point value.
2. the FINITE floor (THE headline) — the optimum minimum is the mean-field bulk quench NO (≈1 g/kg),
   NOT ≈0. The ≈0 rung-13 floor becomes finite bulk NO.
3. the optimum PINNED AT C_opt, both flanks up, far flank CLIMBS — NOT rung-13's descent; the climb is
   the restored dwell (D(u) growth surviving J→∞); the over-flank is non-monotone (two mechanisms).
4. the optimum is AT the Holdeman group C_opt — J_min == J_opt=(C_opt·H/S)², shifting as (H/S)².
5. the STOICH-MEAN SIGN REVERSAL survives (THE discriminator) — term 2's nonlinear bell rises with g
   at a lean mean, FALLS at a stoich mean; a dwell-only construction cannot reverse.
6. g(C)/u(C) are the Holdeman kinks — 0 at C_opt, rising (kinked) on both flanks, symmetric in ln C.
7. cycle untouched — a pdf_quench zoned_nox call must not perturb station 4 (pure diagnostic).
8. require-mixing + mutual-exclusivity(pdf, unmixedness) + QuenchPDF positivity/range guards.

`_ei15` mirrors `Gas.zoned_nox`'s rung-15 math on a bell + quench trajectory built ONCE (both are
J-independent aside from τ_mean), and `test_zoned_nox_matches_ei15_helper` pins it to the production
path at one point — so the sweeps exercise the SAME arithmetic without rebuilding per J.

Run with `python tests/test_rung15.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, Unmixedness, MixingPDF, QuenchPDF, _F_STOICH, _HF_FUEL_DEFAULT,
    _equilibrium_composition, _primary_aft, _thermal_no, _quench_trajectory, _quench_no,
    _bell_interpolator, _beta_pdf_nodes_weights,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_PHI_P = 1.5              # the RQL rich primary (rungs 10–13)
_CE = 0.20               # jet entrainment constant — the anchored regime (main.py rung-11..13 panels)
# Coarse grids — the gates test SHAPE + DIRECTION (finite floor, pin, shift, climb, sign), not digits
# (project ethos; same rationale as rung 10..13). Small ngrid/nsteps keep the equilibrium-heavy quench
# interactive; the pin gate ties the helper to production at the SAME resolution.
_NG = 32                 # finite-quench trajectory resolution
_NSTEPS = 400            # RK4 steps for the bulk quench (term 1)
_NB, _NQ = 120, 160      # bell / β-PDF quadrature nodes


def _mix(J):
    return JetMixing(J=J, C_e=_CE, shape_n=2.0)


_DP_CACHE = None


def _design_point():
    """Build the equilibrium engine once and read the (derived) station-3/4 state + the primary pool
    and the shared τ-independent quench trajectory + the ideal bell (all J-independent) — cached.
    NO is trace → the cycle is bit-for-bit rung 6; every rung-15 gate uses the same design point."""
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
        hf = g.hf_fuel_molar if g.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
        far_p = _PHI_P * _F_STOICH
        alpha = far / far_p
        T_p = _primary_aft(far_p, p, Tt3, hf)
        comp_p = _equilibrium_composition(far_p, T_p, p)
        nox = _thermal_no(comp_p, T_p, p, _TAU, far_p)
        n0 = alpha * nox.x_no * sum(comp_p.values())
        tab = _quench_trajectory(comp_p, T_p, alpha, far, Tt3, p, ngrid=_NG)   # shared, τ-independent
        bell = _bell_interpolator(p, Tt3, hf, _TAU, n_bell=_NB)                # ideal bell, J-independent
        _DP_CACHE = dict(g=g, Tt3=Tt3, Tt4=Tt4, far=far, p=p, hf=hf, comp_p=comp_p, T_p=T_p,
                         alpha=alpha, n0=n0, tab=tab, bell=bell, xibar=far / (1.0 + far))
    return _DP_CACHE


def _floor(dp, J):
    """Term 1 — the rung-11 mean-field bulk quench EI at τ_mean=mixing.tau_q (the FINITE floor). Uses
    the shared trajectory + schedule, exactly as production's ei_no_quenched does."""
    m = _mix(J)
    return _quench_no(dp["comp_p"], dp["T_p"], dp["alpha"], dp["far"], dp["Tt3"], dp["p"],
                      dp["n0"], m.tau_q, nsteps=_NSTEPS, ngrid=_NG, tab=dp["tab"],
                      schedule=m.schedule)["ei"]


def _bell_pdf(dp, mean_xi, g_seg):
    """⟨EI_bell⟩ over the β-PDF on the prebuilt bell — mirrors _pdf_mean_ei's quadrature (g→0 delta)."""
    if g_seg <= 1e-9:
        return dp["bell"](mean_xi)
    nodes, w = _beta_pdf_nodes_weights(mean_xi, g_seg, n_quad=_NQ)
    return sum(wi * dp["bell"](x) for x, wi in zip(nodes, w))


def _ei15(dp, qp, J):
    """⟨EI⟩₁₅ = term1 (floor) + term2 (D(u)·⟨EI_bell⟩(g)) — the production rung-15 arithmetic."""
    C = qp.C(_mix(J))
    term2 = qp.dwell_factor(C, _TAU) * _bell_pdf(dp, dp["xibar"], qp.segregation(C))
    return _floor(dp, J) + term2


def _argmin(vals):
    return min(range(len(vals)), key=lambda i: vals[i])


def _j_opt(qp):
    """J_opt where C=(S/H)√J_opt = C_opt (H=0.10, the JetMixing default)."""
    return (qp.C_opt * JetMixing(J=1.0).H / qp.S) ** 2


# --------------------------------------------------------------------------- #
# GATE 1 — reduce: pdf_quench=None is rung 13; at C_opt (g→0) it is the floor. #
# --------------------------------------------------------------------------- #
def test_reduce_pdf_quench_none_is_rung13_path():
    # pdf_quench=None (the default) must leave EVERY rung-15 field None and match the rung-13/12/11
    # call bit-for-bit (the whole rung-1..14 suite staying green already pins the shared path).
    dp = _design_point()
    g = dp["g"]
    for J in (9.0, 36.0):
        a = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J), quench_ngrid=_NG)
        b = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                        pdf_quench=None, quench_ngrid=_NG)
        for s in (a, b):
            assert s.pdf_quench is None and s.ei_no_pdf_quench is None and s.ei_no_pdf_excess is None
        assert a.ei_no_quenched == b.ei_no_quenched and a.max_a_quench == b.max_a_quench


def test_reduce_at_c_opt_is_finite_bulk_quench_no():
    # THE new reduce (vs rung-13's ≈0): at C_opt the jet is perfectly mixed (g=0 ⇒ term 2 → 0), so
    # ⟨EI⟩₁₅ = the FINITE mean-field bulk quench NO (ei_no_quenched), NOT ≈0. The residual term 2 =
    # D(0)·EI_bell(ξ̄) ≈ 1e-5 g/kg is negligible (<0.01% of the floor).
    dp = _design_point()
    qp = QuenchPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P,
                          mixing=_mix(_j_opt(qp)), pdf_quench=qp,
                          quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert s.g_seg == 0.0 and abs(s.C_holdeman - qp.C_opt) < 1e-12
    assert s.ei_no_quenched > 0.3, f"the bulk floor must be a FINITE non-trace value, got {s.ei_no_quenched}"
    rel = abs(s.ei_no_pdf_quench - s.ei_no_quenched) / s.ei_no_quenched
    assert rel < 1e-4, f"at C_opt ⟨EI⟩₁₅ must equal the finite bulk quench NO to <0.01%, rel={rel:.2e}"


def test_zoned_nox_matches_ei15_helper():
    # Pin the fast helper to the PRODUCTION zoned_nox path (same ngrid/nsteps/n_bell/n_quad), so the
    # sweep gates below exercise the SAME arithmetic the production code does.
    dp = _design_point()
    qp = QuenchPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    J = 36.0
    s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                          pdf_quench=qp, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    h = _ei15(dp, qp, J)
    assert abs(s.ei_no_pdf_quench - h) < 1e-9 * max(h, 1e-12), \
        f"helper {h} vs production {s.ei_no_pdf_quench}"
    assert abs(s.C_holdeman - qp.C(_mix(J))) < 1e-12 and abs(s.g_seg - qp.segregation(qp.C(_mix(J)))) < 1e-12


# --------------------------------------------------------------------------- #
# GATE 2 — the FINITE floor (headline): optimum minimum is the bulk NO, not 0. #
# --------------------------------------------------------------------------- #
def test_finite_floor_at_optimum_not_zero():
    # The rung-15 headline: carrying the PDF through the quench turns rung-13's ≈0 optimum floor into
    # the FINITE bulk quench NO. Contrast rung 13 at the SAME C_opt jet: its ei_no_pdf (ideal bell) is
    # ≈0, while rung-15's ei_no_pdf_quench is the finite mean-field value (orders larger).
    dp = _design_point()
    g = dp["g"]
    qp = QuenchPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    m = _mix(_j_opt(qp))
    s15 = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=m, pdf_quench=qp,
                      quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    s13 = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=m,
                      pdf=MixingPDF(S=0.0625, n_bell=_NB, n_quad=_NQ), quench_ngrid=_NG)
    assert s15.ei_no_pdf_quench > 0.3, f"rung-15 optimum floor must be finite bulk NO, got {s15.ei_no_pdf_quench}"
    assert s13.ei_no_pdf < 1e-3, f"rung-13 optimum (ideal bell) is ≈0, got {s13.ei_no_pdf}"
    assert s15.ei_no_pdf_quench > 1e3 * max(s13.ei_no_pdf, 1e-12), \
        "the ≈0 rung-13 floor must BECOME a finite (orders-larger) bulk NO in rung 15"


# --------------------------------------------------------------------------- #
# GATE 3 — the optimum PINNED AT C_opt: both flanks up, far flank CLIMBS.      #
# --------------------------------------------------------------------------- #
def test_optimum_pinned_at_c_opt_flanks_up_far_flank_climbs():
    # THE rung-15 lesson: the finite floor sits AT C_opt, both immediate flanks lift, and — UNLIKE
    # rung 13 — the far over-penetration flank CLIMBS again (the dwell restored, surviving J→∞). The
    # over-flank is non-monotone (a shallow interior min ABOVE the floor) — the two-mechanism
    # signature (composition convexity jump near C_opt, dwell climb far out).
    dp = _design_point()
    qp = QuenchPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    J_opt = _j_opt(qp)                                       # 16
    ei_opt = _ei15(dp, qp, J_opt)
    ei_under = _ei15(dp, qp, J_opt / 1.7)                    # under-penetration flank
    ei_over = _ei15(dp, qp, J_opt * 1.7)                     # immediate over-penetration flank
    assert ei_under > ei_opt and ei_over > ei_opt, \
        f"both immediate flanks must lift above the C_opt floor: under={ei_under}, opt={ei_opt}, over={ei_over}"
    # the FAR over-flank CLIMBS (dwell restored) — not rung-13's descent to ≈0.
    ei_far1 = _ei15(dp, qp, J_opt * 9)                       # C ≈ 3·C_opt
    ei_far2 = _ei15(dp, qp, J_opt * 25)                      # C ≈ 5·C_opt
    assert ei_far2 > ei_far1, f"far over-flank must CLIMB (restored dwell): ei(9·J)={ei_far1}, ei(25·J)={ei_far2}"
    # and stays ELEVATED (never collapses toward 0 like rung 13) — well above a small fraction of floor.
    assert ei_far1 > 0.5 * ei_opt and ei_far2 > 0.5 * ei_opt, "far flank must stay elevated (dwell), not collapse"
    # the global minimum over a wide sweep is AT C_opt (the finite floor).
    Js = [J_opt / 4, J_opt / 1.7, J_opt, J_opt * 1.7, J_opt * 4, J_opt * 9, J_opt * 25]
    eis = [_ei15(dp, qp, J) for J in Js]
    assert Js[_argmin(eis)] == J_opt, f"global EI-min must sit AT J_opt={J_opt}: {list(zip(Js, eis))}"


# --------------------------------------------------------------------------- #
# GATE 4 — the optimum is AT the Holdeman group C_opt, shifting as (H/S)².     #
# --------------------------------------------------------------------------- #
def test_optimum_at_holdeman_c_opt_shifts_as_H_over_S_squared():
    # The finite-floor minimum sits AT the Holdeman uniformity group: J_min == J_opt=(C_opt·H/S)². So
    # shrinking the spacing S moves the min up EXACTLY as (H/S)² (the kinked g/u pin it at C_opt).
    dp = _design_point()
    for S in (0.0625, 0.0500):
        qp = QuenchPDF(S=S, n_bell=_NB, n_quad=_NQ)
        J_opt = _j_opt(qp)                                  # 16 (S=.0625), 25 (S=.05)
        Js = [J_opt / 4, J_opt / 1.7, J_opt, J_opt * 1.7, J_opt * 4]
        eis = [_ei15(dp, qp, J) for J in Js]
        assert Js[_argmin(eis)] == J_opt, f"S={S}: EI-min must sit AT J_opt={J_opt}, got {list(zip(Js, eis))}"


# --------------------------------------------------------------------------- #
# GATE 5 — the STOICH-MEAN SIGN REVERSAL (the discriminator vs a lumped dwell).#
# --------------------------------------------------------------------------- #
def test_stoich_mean_sign_reversal_the_discriminator():
    # Term 2 samples the NONLINEAR, peaked bell — so ⟨EI_bell⟩ RISES with g at a LEAN mean (the
    # stoich-ward tail reaches the peak) and FALLS with g at a STOICH mean (mass moves OFF the peak).
    # A dwell-only construction (variance riding the ~linear EI_quench) shows the WRONG sign and CANNOT
    # reverse — so this reversal certifies rung 15 is genuine composition work, not rung 12 in disguise.
    dp = _design_point()
    xibar_lean = dp["xibar"]
    xibar_stoich = _F_STOICH / (1.0 + _F_STOICH)
    # lean mean: segregation RAISES the bell integral by orders.
    assert _bell_pdf(dp, xibar_lean, 0.10) > 1e3 * dp["bell"](xibar_lean), \
        "lean mean: segregation must RAISE ⟨EI_bell⟩ by orders (peaked bell × off-peak mean)"
    # stoich mean: segregation LOWERS it (the sign reversal).
    assert dp["bell"](xibar_stoich) > 5.0, "sanity: the stoich point value should be large (near the peak)"
    assert _bell_pdf(dp, xibar_stoich, 0.10) < dp["bell"](xibar_stoich), \
        "stoich mean: segregation must LOWER ⟨EI_bell⟩ (sign reversal — the rung-12-in-disguise discriminator)"


# --------------------------------------------------------------------------- #
# GATE 6 — g(C)/u(C) are the Holdeman kinks: 0 at C_opt, rising both flanks.   #
# --------------------------------------------------------------------------- #
def test_segregation_and_unmixedness_kinked_zero_at_optimum():
    qp = QuenchPDF()
    assert qp.segregation(qp.C_opt) == 0.0 and qp._u(qp.C_opt) == 0.0, "g,u must be 0 at C_opt"
    lo, hi = qp.segregation(qp.C_opt / 1.3), qp.segregation(qp.C_opt * 1.3)
    assert lo > 0.0 and hi > 0.0, f"g must rise on BOTH flanks: {lo}, {hi}"
    assert abs(qp.segregation(qp.C_opt / 1.4) - qp.segregation(qp.C_opt * 1.4)) < 1e-12   # symmetric in ln C
    assert qp.segregation(qp.C_opt * 1.05) > 0.0                                          # KINKED slope
    assert qp.segregation(qp.C_opt * 1e6) == qp.g_max                                     # capped
    # the dwell factor GROWS off-optimum (both flanks) and is τ_res/τ_ref at C_opt.
    assert abs(qp.dwell_factor(qp.C_opt, _TAU) - qp.tau_res / _TAU) < 1e-12
    assert qp.dwell_factor(qp.C_opt * 1.3, _TAU) > qp.dwell_factor(qp.C_opt, _TAU)
    assert qp.dwell_factor(qp.C_opt / 1.3, _TAU) > qp.dwell_factor(qp.C_opt, _TAU)


# --------------------------------------------------------------------------- #
# GATE 7 — cycle untouched by a pdf_quench call.                              #
# --------------------------------------------------------------------------- #
def test_cycle_untouched_by_pdf_quench_call():
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)

    r1 = run()
    st3, st4 = r1.stations["3"], r1.stations["4"]
    far1, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    g.zoned_nox(far1, Tt3, Tt4, p, _PHI_P, mixing=_mix(36.0),
                pdf_quench=QuenchPDF(n_bell=_NB, n_quad=_NQ), quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert run().stations["4"].far == far1, "pdf_quench call perturbed the cycle far — must stay rung-6"


# --------------------------------------------------------------------------- #
# GATE 8 — require-mixing + mutual-exclusivity + positivity guards.            #
# --------------------------------------------------------------------------- #
def test_pdf_quench_requires_mixing():
    dp = _design_point()
    try:
        dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P,
                          pdf_quench=QuenchPDF(), quench_ngrid=_NG)     # no mixing
    except AssertionError:
        return
    raise AssertionError("pdf_quench without mixing must be rejected (needs J and H + τ_mean)")


def test_pdf_quench_mutually_exclusive_with_pdf_and_unmixedness():
    dp = _design_point()
    for extra in (dict(pdf=MixingPDF()), dict(unmixedness=Unmixedness())):
        try:
            dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(16.0),
                              pdf_quench=QuenchPDF(), quench_ngrid=_NG, **extra)
        except AssertionError:
            continue
        raise AssertionError(f"pdf_quench + {list(extra)[0]} must be rejected (same variance physics)")


def test_quenchpdf_positivity_guards():
    QuenchPDF()                                             # defaults accepted
    QuenchPDF(k_g=0.0)                                      # k_g=0 ⇒ g≡0 (floor only), allowed
    QuenchPDF(b_u=0.0)                                      # b_u=0 ⇒ flat dwell, allowed
    for bad in (dict(S=0.0), dict(S=-0.1), dict(C_opt=0.0), dict(tau_res=0.0), dict(k_g=-0.1),
                dict(b_u=-0.1), dict(g_max=0.0), dict(g_max=1.0), dict(n_bell=1), dict(n_quad=0)):
        try:
            QuenchPDF(**bad)
        except AssertionError:
            continue
        raise AssertionError(f"QuenchPDF({bad}) should be rejected (positivity/range guard)")


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-15 gates passed")
