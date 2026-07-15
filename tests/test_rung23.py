"""Rung 23 — the DERIVED DWELL SPECTRUM through the per-pocket quench (the rung-16 analog of rung 22).

Rung 22 resolved the cross-plane and derived the β-PDF WIDTH g(C), but fed it through the per-pocket
quench with the IMPORTED rung-16 kinked SCALAR dwell τ_core(C) — which BAKES C_opt in. Rung 23 develops
the SAME cross-plane in TIME (`_spatial_dwell_field`, over rung-11's τ_mix=mixing.tau_q) so each pocket
carries its OWN dwell τ(ξ) from first principles (NO C_opt, NO τ_res, NO b_u):

    ⟨EI⟩₂₃(J) = ei_no_quenched(τ_mean)                       [term 1: rung-11 mean-field floor — UNCHANGED]
              + ⟨EI_pocket_quench(ξ; τ(ξ))⟩_g                 [term 2: PER-POCKET quench, DERIVED dwell τ(ξ)]

THE ROBUST LESSON (this file certifies ONLY these — see docs/rung23-spec.md):
  1. THE REDUCE — spatial_dwell=None ⇒ prior path untouched; the terminal (t=τ_mix) field reproduces
     rung-22's g_spatial (`_spatial_dwell_field` == `_spatial_segregation`, the consistency anchor).
  2. THE CORRELATION SIGN (the load-bearing positive) — the MATCHED-MEAN experiment (τ(ξ) spectrum vs a
     scalar dwell = ⟨τ⟩_PDF, isolating ONLY the correlation) gives corr_ratio>1: rich pockets dwell long
     ⇒ ADD NO, ONE-SIGNED across τ_mix ×0.2–×5 (formation-limited, max_a<1), CONCENTRATED under-penetration.
  3. NOT CLAIMED — the emissions global-min LOCATION. The derived τ FALLS off-optimum (∝1/√J), so it does
     NOT lift the over-penetration flank; but rung 16 already declined the global-min location (its GATE 3).
     Whether the emissions C_opt pin survives rides on the un-anchored τ_mix trend — NOT asserted either way.

Coarse grids — SHAPE + DIRECTION, not digits (project ethos). Per-pocket quench is expensive, so the bank
of per-pocket trajectories is built ONCE (τ-independent) and the cheap NO-only RK4 re-runs per dwell.
"""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, SpatialDwellPDF, SpatialPDF, PocketQuenchPDF, MixingPDF,
    _F_STOICH, _HF_FUEL_DEFAULT, _equilibrium_composition, _primary_aft, _thermal_no,
    _quench_trajectory, _quench_no, _ideal_bell_ei, _beta_pdf_nodes_weights,
    _spatial_dwell_field, _spatial_segregation, _two_stream_ceiling,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_PHI_P = 1.5
_CE = 0.20
_U_C = 75.0
_NG = 24
_NSTEPS = 200
_NB, _NQ = 40, 56
_NY = _NZ = 32
_NT = 24

_DP_CACHE = None


def _mix(J):
    return JetMixing(J=J, C_e=_CE, U_c=_U_C, shape_n=2.0)


def _cfg(**kw):
    d = dict(S=0.0625, ny=_NY, nz=_NZ, nt=_NT, n_bell=_NB, n_quad=_NQ)
    d.update(kw)
    return SpatialDwellPDF(**d)


def _design_point():
    """Build the equilibrium engine once; read station-3/4 + the rich primary bulk trajectory (term 1)
    + the PER-POCKET trajectory bank (term 2, dwell-independent, built ONCE over a fixed ξ-grid). NO is
    trace ⇒ the cycle is bit-for-bit rung 6. Mirrors tests/test_rung16.py."""
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
        n0 = alpha * _thermal_no(comp_p, T_p, p, _TAU, far_p).x_no * sum(comp_p.values())
        tab = _quench_trajectory(comp_p, T_p, alpha, far, Tt3, p, ngrid=_NG)
        xi_max = (2.0 * _F_STOICH) / (1.0 + 2.0 * _F_STOICH)
        xi_grid = [xi_max * (i + 0.5) / _NB for i in range(_NB)]
        bank = []
        for xi in xi_grid:
            fl = xi / (1.0 - xi)
            if fl < far or fl / _F_STOICH > 2.0 + 1e-9 or fl <= 0.0:
                bank.append(("b", _ideal_bell_ei(fl, p, Tt3, hf, _TAU)))
                continue
            try:
                T_pk = _primary_aft(fl, p, Tt3, hf)
            except AssertionError:
                bank.append(("b", 0.0))
                continue
            al = far / fl
            comp = _equilibrium_composition(fl, T_pk, p)
            n0k = al * _thermal_no(comp, T_pk, p, _TAU, fl).x_no * sum(comp.values())
            tabk = _quench_trajectory(comp, T_pk, al, far, Tt3, p, ngrid=_NG)
            bank.append(("q", (comp, T_pk, al, n0k, tabk)))
        _DP_CACHE = dict(g=g, Tt3=Tt3, Tt4=Tt4, far=far, p=p, hf=hf, comp_p=comp_p, T_p=T_p,
                         alpha=alpha, n0=n0, tab=tab, xi_grid=xi_grid, bank=bank,
                         xibar=far / (1.0 + far))
    return _DP_CACHE


def _floor(dp, J):
    """Term 1 — the rung-11 mean-field bulk quench EI at τ_mean=mixing.tau_q (the FINITE floor)."""
    m = _mix(J)
    return _quench_no(dp["comp_p"], dp["T_p"], dp["alpha"], dp["far"], dp["Tt3"], dp["p"],
                      dp["n0"], m.tau_q, nsteps=_NSTEPS, ngrid=_NG, tab=dp["tab"],
                      schedule=m.schedule)["ei"]


def _term2(dp, g_seg, tau_of_xi):
    """Term 2 — ⟨EI_pocket_quench(ξ; tau_of_xi(ξ))⟩_g on the cached bank (mirrors _pocket_quench_mean_ei
    with the rung-23 tau_of_xi callable). Returns (excess, max_a)."""
    vals, max_a = [], 0.0
    for (kind, payload), xi in zip(dp["bank"], dp["xi_grid"]):
        if kind == "q":
            comp, T_pk, al, n0k, tabk = payload
            q = _quench_no(comp, T_pk, al, dp["far"], dp["Tt3"], dp["p"], n0k, tau_of_xi(xi),
                           nsteps=_NSTEPS, ngrid=_NG, tab=tabk)
            vals.append(q["ei"])
            max_a = max(max_a, q["max_a"])
        else:
            vals.append(payload)
    xg = dp["xi_grid"]

    def qb(xi):
        if xi <= xg[0]:
            return vals[0]
        if xi >= xg[-1]:
            return 0.0
        lo, hi = 0, len(xg) - 1
        while hi - lo > 1:
            mid = (lo + hi) // 2
            if xg[mid] <= xi:
                lo = mid
            else:
                hi = mid
        w = (xi - xg[lo]) / (xg[hi] - xg[lo])
        return vals[lo] + w * (vals[hi] - vals[lo])

    nodes, wts = _beta_pdf_nodes_weights(dp["xibar"], g_seg, n_quad=_NQ)
    return sum(wi * qb(x) for wi, x in zip(wts, nodes)), max_a


def _spectrum(dp, cfg, J, scale=1.0):
    """The derived τ(ξ) (scaled by `scale` for the τ_mix-sensitivity gate) + ⟨τ⟩_PDF (matched-mean)."""
    m = _mix(J)
    g_seg, tau0 = _spatial_dwell_field(dp["far"], _PHI_P, cfg.S, m.H, m.J, m.tau_q,
                                       k_p=cfg.k_p, k_y=cfg.k_y, k_z=cfg.k_z,
                                       ny=cfg.ny, nz=cfg.nz, nt=cfg.nt)
    tau_of = lambda xi: scale * tau0(xi)
    nodes, wts = _beta_pdf_nodes_weights(dp["xibar"], g_seg, n_quad=_NQ)
    tau_mean = sum(wi * tau_of(x) for wi, x in zip(wts, nodes))
    return g_seg, tau_of, tau_mean


def _corr_ratio(dp, cfg, J, scale=1.0):
    """The matched-mean isolation: term2(correlated τ(ξ)) / term2(scalar ⟨τ⟩). Returns (ratio, max_a)."""
    g_seg, tau_of, tau_mean = _spectrum(dp, cfg, J, scale=scale)
    e_corr, a_corr = _term2(dp, g_seg, tau_of)
    e_mean, a_mean = _term2(dp, g_seg, lambda xi, _t=tau_mean: _t)
    return e_corr / e_mean, max(a_corr, a_mean)


# --------------------------------------------------------------------------- #
# GATE 1 — the REDUCE: spatial_dwell=None is the prior path; terminal field == rung 22.
# --------------------------------------------------------------------------- #
def test_reduce_spatial_dwell_none_is_prior_path():
    dp = _design_point()
    g = dp["g"]
    for J in (9.0, 36.0):
        a = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J), quench_ngrid=_NG)
        b = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                        spatial_dwell=None, quench_ngrid=_NG)
        for s in (a, b):
            assert s.spatial_dwell is None and s.ei_no_spatial_dwell is None
            assert s.corr_ratio is None and s.tau_mean_dwell is None
        assert a.ei_no_quenched == b.ei_no_quenched and a.primary.ei_no == b.primary.ei_no


def test_terminal_field_reproduces_rung22():
    # The consistency anchor: `_spatial_dwell_field` at t=τ_mix == `_spatial_segregation` (rung 22).
    dp = _design_point()
    cfg = _cfg()
    for J in (4.0, 16.0, 100.0):
        m = _mix(J)
        g_dwell, _ = _spatial_dwell_field(dp["far"], _PHI_P, cfg.S, m.H, m.J, m.tau_q,
                                          k_p=cfg.k_p, k_y=cfg.k_y, k_z=cfg.k_z,
                                          ny=cfg.ny, nz=cfg.nz, nt=cfg.nt)
        g22 = _spatial_segregation(dp["far"], _PHI_P, cfg.S, m.H, m.J,
                                   k_p=cfg.k_p, k_y=cfg.k_y, k_z=cfg.k_z, ny=cfg.ny, nz=cfg.nz)
        assert abs(g_dwell - g22) < 1e-9, f"terminal field {g_dwell} != rung-22 {g22} at J={J}"


def test_production_g_matches_spatialpdf():
    # The production spatial_dwell width == SpatialPDF's width at the same grid (both = the terminal field).
    dp = _design_point()
    J = 16.0
    s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                          spatial_dwell=_cfg(), quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    g22 = SpatialPDF(S=0.0625, ny=_NY, nz=_NZ).segregation(_mix(J), dp["far"], _PHI_P)[0]
    assert abs(s.g_spatial_dwell - g22) < 1e-9


# --------------------------------------------------------------------------- #
# GATE 2 — the CORRELATION SIGN (matched-mean): corr_ratio > 1, one-signed across τ_mix.
# --------------------------------------------------------------------------- #
def test_correlation_adds_no_at_design_point():
    # The load-bearing positive: at the design point the correlated τ(ξ) makes MORE NO than the
    # matched-mean scalar ⟨τ⟩ — rich pockets dwell long ⇒ re-make more. corr_ratio > 1.
    dp = _design_point()
    s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(16.0),
                          spatial_dwell=_cfg(), quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert s.corr_ratio > 1.0, f"correlation must ADD NO (corr_ratio>1), got {s.corr_ratio}"
    assert s.ei_no_spatial_dwell > s.ei_no_spatial_dwell_meanfield
    # the correlation is the ONLY difference between the twins (same g, same ⟨τ⟩, same floor)
    assert abs((s.ei_no_spatial_dwell - s.ei_no_spatial_dwell_meanfield)
               - (s.ei_no_spatial_dwell_excess - (s.ei_no_spatial_dwell_meanfield - s.ei_no_quenched))) < 1e-9


def test_correlation_sign_one_signed_across_tau_mix():
    # THE BLOCKER, gated: the correlation SIGN survives τ_mix ×0.2–×5 (the pockets stay formation-limited,
    # max_a<1 — the Jensen concavity never wins). Certifies the SIGN, not the magnitude.
    dp = _design_point()
    cfg = _cfg()
    for J in (4.0, 16.0):
        for scale in (0.2, 1.0, 5.0):
            r, mx = _corr_ratio(dp, cfg, J, scale=scale)
            assert r > 1.0, f"corr_ratio {r} <= 1 at J={J}, scale={scale} (sign flipped)"
            assert mx < 1.0, f"max_a {mx} >= 1 at J={J}, scale={scale} (left the formation-limited regime)"


def test_correlation_concentrated_under_penetration():
    # The correlation is LARGEST under-penetration (long dwell + high ξ-τ correlation) and fades toward
    # C_opt — the certified SHAPE (the under-penetration concentration).
    dp = _design_point()
    cfg = _cfg()
    r_under, _ = _corr_ratio(dp, cfg, 4.0)     # C≈1.25, under-penetration
    r_opt, _ = _corr_ratio(dp, cfg, 16.0)      # C≈2.5, at C_opt
    assert r_under > r_opt > 1.0, f"under-penetration {r_under} must exceed C_opt {r_opt} > 1"


# --------------------------------------------------------------------------- #
# GATE 3 — the rung-18 tie + clamp dormancy + cycle-untouched.
# --------------------------------------------------------------------------- #
def test_g_below_two_stream_ceiling():
    dp = _design_point()
    for J in (1.0, 16.0, 400.0):
        s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                              spatial_dwell=_cfg(), quench_ngrid=_NG, quench_nsteps=_NSTEPS)
        assert s.g_spatial_dwell < s.g_ceiling, f"g {s.g_spatial_dwell} !< ceiling {s.g_ceiling} at J={J}"


def test_clamp_dormant_at_station4():
    dp = _design_point()
    for J in (4.0, 16.0, 100.0):
        s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                              spatial_dwell=_cfg(), quench_ngrid=_NG, quench_nsteps=_NSTEPS)
        assert s.max_a_quench < 1.0, f"clamp fired at station 4 (max_a={s.max_a_quench}) at J={J}"


def test_cycle_untouched():
    # The primary diagnostic is bit-identical to a mixing-only call (NO/N never enter the cycle solve).
    dp = _design_point()
    base = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(16.0),
                             quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(16.0),
                          spatial_dwell=_cfg(), quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert s.primary.ei_no == base.primary.ei_no and s.x_no_mix == base.x_no_mix
    assert s.ei_no_quenched == base.ei_no_quenched


# --------------------------------------------------------------------------- #
# GATE 4 — the NON-CIRCULARITY signature: NO C_opt knob; C_opt() is derived.
# --------------------------------------------------------------------------- #
def test_no_c_opt_knob():
    with pytest.raises(TypeError):
        SpatialDwellPDF(C_opt=2.5)          # the signature of the inversion — C_opt is NOT an input
    cfg = _cfg(k_p=0.316)
    assert abs(cfg.C_opt() - 1.0 / (4 * 0.316 ** 2)) < 1e-12
    assert abs(_cfg(k_p=0.20).C_opt() - 1.0 / (4 * 0.20 ** 2)) < 1e-12   # k_p SETS C_opt as an output


def test_helper_matches_production():
    # Pin the fast cached-bank helper to the PRODUCTION zoned_nox path at the SAME resolution.
    dp = _design_point()
    cfg = _cfg()
    J = 16.0
    s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                          spatial_dwell=cfg, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    g_seg, tau_of, tau_mean = _spectrum(dp, cfg, J)
    excess_corr, _ = _term2(dp, g_seg, tau_of)
    ei = _floor(dp, J) + excess_corr
    assert abs(s.ei_no_spatial_dwell - ei) < 1e-6 * max(ei, 1e-12), \
        f"helper {ei} vs production {s.ei_no_spatial_dwell}"
    assert abs(s.tau_mean_dwell - tau_mean) < 1e-9


# --------------------------------------------------------------------------- #
# GATE 5 — the guards.
# --------------------------------------------------------------------------- #
def test_requires_mixing():
    dp = _design_point()
    with pytest.raises(AssertionError):
        dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, spatial_dwell=_cfg())


def test_at_most_one_closure():
    dp = _design_point()
    with pytest.raises(AssertionError):
        dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(16.0),
                          spatial_dwell=_cfg(), pocket_quench=PocketQuenchPDF(S=0.0625))
    with pytest.raises(AssertionError):
        dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(16.0),
                          spatial_dwell=_cfg(), pdf=MixingPDF(S=0.0625))


def test_config_positivity():
    for bad in (dict(S=-1.0), dict(k_p=0.0), dict(k_y=-0.1), dict(ny=1), dict(nt=1)):
        with pytest.raises(AssertionError):
            _cfg(**bad)


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-q"]))
