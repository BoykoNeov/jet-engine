"""Rung 18 — the TRANSPORTED-variance closure: what a 0-D variance equation CAN and CANNOT derive.

Rungs 12-17 IMPOSE the β-PDF width as a kinked g(C)=min(g_max, k_g·|ln(C/C_opt)|). Rung 18 solves
g(C) as the residual of a variance DECAY ODE dg/dt=−C_φ·ω(C)·g from a DERIVED two-stream ceiling,
fed through the rung-13 ideal bell.

THE LOAD-BEARING RESULT IS NEGATIVE (docs/rung18-spec.md): a 0-D transport CANNOT derive the C_opt
optimum — with MEAN-FIELD ω(J) the residual g(J) is monotone/flat (no interior optimum); an optimum
appears ONLY once ω is given a SPATIAL coverage ω(C=(S/H)√J) — the jet spacing S injected by hand
(rung-11's 'mean-field ⇒ no mixing optimum' seam). What transport LEGITIMATELY adds and this file
certifies: the DERIVED two-stream ceiling (exposing g_max=0.3 as ~4.4× too large), the RESIDUAL floor
(optimum elevated off the well-mixed value), and KINK-non-genericity (the smooth basin vs the corner).

Coarse grids — SHAPE + DIRECTION, not digits (project ethos).
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, MixingPDF, PocketQuenchPDF, TransportedPDF, _F_STOICH, _HF_FUEL_DEFAULT,
    _two_stream_ceiling, _transport_variance, _pdf_mean_ei, _ideal_bell_ei,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_PHI_P = 1.5
_CE = 0.20
_NB, _NQ = 48, 64            # coarse ideal-bell / β-PDF grids (shape, not digits)


def _mix(J):
    return JetMixing(J=J, C_e=_CE, shape_n=2.0)


def _j_opt(cfg):
    """J where C=(S/H)√J = C_opt (H=0.10, the JetMixing default) — the imposed coverage peak."""
    return (cfg.C_opt * JetMixing(J=1.0).H / cfg.S) ** 2


_DP_CACHE = None


def _design_point():
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
        hf = g.hf_fuel_molar if g.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
        _DP_CACHE = dict(g=g, Tt3=Tt3, Tt4=Tt4, far=far, p=p, hf=hf,
                         xibar=far / (1.0 + far))
    return _DP_CACHE


def _cfg(**kw):
    return TransportedPDF(S=0.0625, n_bell=_NB, n_quad=_NQ, n_ode=200, **kw)


def _run(dp, J, cfg):
    return dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, tau=_TAU,
                             mixing=_mix(J), transported=cfg, quench_ngrid=24, quench_nsteps=200)


# --------------------------------------------------------------------------------------------------
# GATE 1 — reduce (load-bearing).
# --------------------------------------------------------------------------------------------------

def test_reduce_none_leaves_prior_path_untouched():
    """transported=None short-circuits before any rung-18 code — a mixing-only call is unaffected
    and the rung-18 fields stay None (the whole rung 1-17 suite stays green by this construction)."""
    dp = _design_point()
    base = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, tau=_TAU,
                             mixing=_mix(16), quench_ngrid=24, quench_nsteps=200)
    assert base.transported is None and base.ei_no_transported is None and base.g_ceiling is None
    # the mean-field quench (term-1) result is present and untouched by rung 18
    assert base.ei_no_quenched is not None and base.ei_no_quenched > 0.0


def test_reduce_perfect_mixing_recovers_the_kinked_notch():
    """Da_opt→∞ (perfect best-jet mixing) ⇒ g(C_opt)→0 ⇒ ei_no_transported = the well-mixed point
    value (the rung-13 g→0 value). The kinked model IS the infinite-mixing limit of the transported."""
    dp = _design_point()
    point = _ideal_bell_ei(dp["far"], dp["p"], dp["Tt3"], dp["hf"], _TAU)   # well-mixed φ_overall value
    st = _run(dp, 16, _cfg(Da_opt=60.0))                                    # near-perfect mixing at C_opt
    assert st.g_transported < 1e-9, f"g(C_opt) should collapse, got {st.g_transported:.2e}"
    assert math.isclose(st.ei_no_transported, point, rel_tol=0.02), \
        f"perfect-mixing ei {st.ei_no_transported:.5f} vs point value {point:.5f}"


def test_reduce_zero_ceiling_is_point_value():
    """g_ceiling→0 (no injected segregation) ⇒ g≡0 ⇒ the well-mixed point value ∀J (helper-level:
    a two-stream ceiling shrinks to 0 as the primary approaches the overall mean)."""
    dp = _design_point()
    # a primary only marginally richer than the overall mean ⇒ a vanishing ceiling
    phi_ov = dp["far"] / _F_STOICH
    gc = _two_stream_ceiling(dp["far"], phi_ov * 1.02)
    assert 0.0 < gc < 5e-3, f"near-overall primary should give a tiny ceiling, got {gc:.4f}"
    g = _transport_variance(gc, 500.0, 2.5e-3, c_phi=2.0, nsteps=400)
    assert g < gc and g > 0.0


# --------------------------------------------------------------------------------------------------
# GATE 2 — THE NEGATIVE RESULT (the headline): 0-D transport cannot derive the optimum.
# --------------------------------------------------------------------------------------------------

def test_meanfield_omega_is_monotone_no_optimum():
    """A GENUINE variance ODE with any MEAN-FIELD ω(J) (const / √J / J), fixed τ, gives a
    monotone-or-flat g(J) — NO interior optimum. These curves ILLUSTRATE the structural argument
    (they do not carry it): an interior optimum needs C_φ·ω·τ non-monotone in J ⇒ ω(J) with an
    interior MAXIMUM ⇒ a PREFERRED LENGTH SCALE (a specific penetration). A mean-field ω(J) is built
    only from J/τ_q(J)/U_c/H — no spacing S — so it has no scale to single out a J, so it cannot peak.
    The optimum can enter ONLY via ω(C=(S/H)√J) (next test) — the spatial S. (Rung-11's own result.)"""
    gc = 0.0675
    Js = [4, 9, 16, 25, 49, 100, 225, 625]
    forms = {
        "const": lambda J: 250.0,
        "sqrtJ": lambda J: 250.0 * math.sqrt(J / 16.0),
        "linJ":  lambda J: 250.0 * (J / 16.0),
    }
    for name, om in forms.items():
        vals = [_transport_variance(gc, om(J), 2.5e-3, c_phi=2.0, nsteps=400) for J in Js]
        imin = min(range(len(vals)), key=lambda i: vals[i])
        flat = (max(vals) - min(vals)) <= 1e-4 * max(vals)
        assert flat or imin in (0, len(vals) - 1), \
            f"mean-field ω={name} produced an INTERIOR optimum at J={Js[imin]} — impossible in 0-D"


def test_spatial_coverage_omega_is_needed_for_the_optimum():
    """The interior optimum appears ONLY when ω depends on C=(S/H)√J — i.e. once the SPATIAL spacing
    S is injected. Same ODE, same fixed τ, ω peaked at C_opt ⇒ interior min at J_opt."""
    cfg = _cfg()
    Js = [4, 9, 16, 25, 49, 100, 225, 625]
    gc = 0.0675

    def gJ(J):
        C = (cfg.S / JetMixing(J=1.0).H) * math.sqrt(J)
        return _transport_variance(gc, cfg.coverage_omega(C), cfg.tau_mix, c_phi=cfg.C_phi, nsteps=400)

    vals = [gJ(J) for J in Js]
    imin = min(range(len(vals)), key=lambda i: vals[i])
    assert 0 < imin < len(vals) - 1, "spatial coverage ω(C) must give an INTERIOR optimum"
    assert Js[imin] == 16, f"the optimum must sit at J_opt=16 (C_opt), got J={Js[imin]}"


# --------------------------------------------------------------------------------------------------
# GATE 3 — the DERIVED ceiling (lead result).
# --------------------------------------------------------------------------------------------------

def test_derived_ceiling_from_phi_p():
    """g_ceiling = (ξ_p−ξ̄)/(1−ξ̄) from φ_p, to machine precision, >0, and < the rung-13 free g_max=0.3
    (~4.4× larger). A pure composition quantity — independent of J and C_e."""
    dp = _design_point()
    xibar = dp["xibar"]
    far_p = _PHI_P * _F_STOICH
    xi_p = far_p / (1.0 + far_p)
    expect = (xi_p - xibar) / (1.0 - xibar)
    assert math.isclose(_two_stream_ceiling(dp["far"], _PHI_P), expect, rel_tol=1e-12)
    assert 0.0 < expect < 0.3, f"derived ceiling {expect:.4f} must be < g_max=0.3"
    assert 0.3 / expect > 4.0, f"g_max=0.3 should be >4× the derived ceiling {expect:.4f}"
    # J/C_e independence: same across two jets
    a = _run(dp, 16, _cfg()).g_ceiling
    b = _run(dp, 100, _cfg()).g_ceiling
    assert math.isclose(a, b, rel_tol=1e-12) and math.isclose(a, expect, rel_tol=1e-9)


def test_rich_primary_required_for_ceiling():
    """A primary LEANER than the overall mean has no two-stream segregation (ceiling ∉ (0,1)) — the
    RQL geometry guard."""
    dp = _design_point()
    phi_ov = dp["far"] / _F_STOICH
    try:
        _two_stream_ceiling(dp["far"], phi_ov * 0.5)         # primary leaner than the mean
        raise AssertionError("leaner-than-mean primary should fail the ceiling guard")
    except AssertionError as e:
        assert "RQL geometry" in str(e) or "RICHER" in str(e)


# --------------------------------------------------------------------------------------------------
# GATE 4 — the RESIDUAL floor + min AT C_opt (elevated, not touching zero).
# --------------------------------------------------------------------------------------------------

def test_residual_floor_elevates_the_optimum():
    """g(C_opt)=g_ceiling·exp(−Da_opt) > 0 (perfect mixing never reached), so ei_no_transported at
    C_opt sits ABOVE the well-mixed point value — the elevated optimum, NOT the kink's touch-the-floor.
    And the minimum is AT C_opt (the imposed coverage pins the location; both immediate flanks up)."""
    dp = _design_point()
    point = _ideal_bell_ei(dp["far"], dp["p"], dp["Tt3"], dp["hf"], _TAU)
    st_opt = _run(dp, 16, _cfg())
    assert st_opt.g_transported > 1e-3, "residual floor g(C_opt) must be > 0 (no perfect mixing)"
    assert st_opt.ei_no_transported > 10.0 * max(point, 1e-9), \
        "the elevated optimum must sit well above the well-mixed point value"
    # min AT C_opt: both immediate flanks lift
    ei_lo = _run(dp, 9, _cfg()).ei_no_transported     # under-penetration
    ei_hi = _run(dp, 25, _cfg()).ei_no_transported    # over-penetration
    assert ei_lo > st_opt.ei_no_transported and ei_hi > st_opt.ei_no_transported, \
        f"min must be AT C_opt (J=16): {ei_lo:.4f} > {st_opt.ei_no_transported:.4f} < {ei_hi:.4f}"


# --------------------------------------------------------------------------------------------------
# GATE 5 — KINK-is-non-generic (smoothness): the transported basin is SMOOTH, the kink a CORNER.
# --------------------------------------------------------------------------------------------------

def test_transported_width_is_smooth_kink_is_a_corner():
    """The transported g(C) has both one-sided slopes → 0 at C_opt (smooth analytic min); the imposed
    kink is a CORNER (equal-and-opposite one-sided slopes ±k_g/C_opt)."""
    cfg = _cfg()
    gc = 0.0675
    C0 = cfg.C_opt
    eps = 1e-5

    def g_tr(C):
        return _transport_variance(gc, cfg.coverage_omega(C), cfg.tau_mix, c_phi=cfg.C_phi, nsteps=400)

    sr = (g_tr(C0 * (1 + eps)) - g_tr(C0)) / (eps * C0)
    sl = (g_tr(C0) - g_tr(C0 * (1 - eps))) / (eps * C0)
    assert abs(sr) < 1e-2 and abs(sl) < 1e-2, f"transported slopes must vanish at C_opt: L={sl}, R={sr}"

    # the kink: g=k_g·|ln(C/C_opt)| has one-sided slopes ±k_g/C_opt (a corner)
    kink = MixingPDF(S=cfg.S, C_opt=C0)
    ks_r = (kink.segregation(C0 * (1 + eps)) - kink.segregation(C0)) / (eps * C0)
    ks_l = (kink.segregation(C0) - kink.segregation(C0 * (1 - eps))) / (eps * C0)
    assert abs(ks_r - ks_l) > 1e-2, "the imposed kink must be a corner (nonzero one-sided slope jump)"


def test_emissions_basin_rounds_the_notch():
    """One step off J_opt the transported basin changes by O(1) (rounded); the kinked ideal-bell notch
    dives by ≫10³× (it touches the ≈0 well-mixed floor). The sharpness was the artifact."""
    dp = _design_point()
    cfg = _cfg()
    ei_opt = _run(dp, 16, cfg).ei_no_transported
    ei_off = _run(dp, 9, cfg).ei_no_transported
    assert 1.0 < ei_off / ei_opt < 3.0, f"transported basin should round (O(1) step), got {ei_off/ei_opt:.2f}"

    # the kinked g through the SAME ideal bell dives to the floor at C_opt (a notch)
    kink = MixingPDF(S=cfg.S, n_bell=_NB, n_quad=_NQ)
    g_opt = max(kink.segregation(kink.C(_mix(16))), 1e-12)
    g_off = kink.segregation(kink.C(_mix(9)))
    k_opt = _pdf_mean_ei(dp["far"], dp["Tt3"], dp["p"], dp["hf"], _TAU, g_opt, n_bell=_NB, n_quad=_NQ)
    k_off = _pdf_mean_ei(dp["far"], dp["Tt3"], dp["p"], dp["hf"], _TAU, g_off, n_bell=_NB, n_quad=_NQ)
    assert k_off / max(k_opt, 1e-12) > 1e3, "the kinked notch must dive ≫10³× one step off C_opt"


# --------------------------------------------------------------------------------------------------
# GATE 6 — cycle untouched (pure diagnostic).
# --------------------------------------------------------------------------------------------------

def test_cycle_untouched():
    """A transported call leaves the cycle far bit-identical — NO/N never enter the equilibrium
    solve, so the cycle stays bit-for-bit rung 6."""
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)

    far0 = run().stations["4"].far
    dp = _design_point()
    _run(dp, 16, _cfg())
    assert run().stations["4"].far == far0, "transported call perturbed the cycle far — must stay rung-6"


# --------------------------------------------------------------------------------------------------
# GATE 7/8 — guards.
# --------------------------------------------------------------------------------------------------

def test_requires_mixing():
    dp = _design_point()
    try:
        dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, transported=_cfg())
        raise AssertionError("transported without mixing should raise")
    except AssertionError as e:
        assert "REQUIRES a `mixing`" in str(e)


def test_at_most_one_closure():
    dp = _design_point()
    try:
        dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(16),
                          transported=_cfg(), pocket_quench=PocketQuenchPDF())
        raise AssertionError("two closures should raise the ≤1-of-five guard")
    except AssertionError as e:
        assert "AT MOST ONE" in str(e)


def test_positivity_guards():
    for bad in (dict(S=0.0), dict(C_opt=0.0), dict(C_phi=0.0), dict(Da_opt=0.0),
                dict(w_cov=0.0), dict(tau_mix=0.0), dict(n_ode=1)):
        try:
            TransportedPDF(**bad)
        except AssertionError:
            continue
        raise AssertionError(f"TransportedPDF({bad}) should be rejected")
    TransportedPDF()                       # defaults accepted


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-18 tests passed")
