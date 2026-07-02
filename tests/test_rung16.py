"""Rung 16 — the PDF THROUGH the finite quench, PER POCKET (retires rung-15's linearised dwell).

Rung 15 carried the composition β-PDF through the dwell as term 2 = D(u)·⟨EI_bell⟩(g): the
CONSTANT-T ideal bell scaled by a SCALAR dwell factor D(u)=τ_core/τ_ref — exact only while EI ∝ τ
(the dormant clamp), which IGNORES that a lingering pocket COOLS. Rung 16 carries EACH rich-of-mean
β-PDF pocket through its OWN finite quench (`_quench_no` at the dwell τ_core), so the dwell acts
INSIDE the cooling chemistry.

    ⟨EI⟩₁₆(J) = EI_bulk_quench(τ_mean(J))            [term 1: rung-11 mean-field floor — UNCHANGED]
              + ⟨EI_pocket_quench(ξ; τ_core(C))⟩_g   [term 2: PER-POCKET quench β-PDF integral]

THE ROBUST LESSON (this file certifies ONLY these — see docs/rung16-spec.md):
  1. SUBLINEAR DWELL (the mechanism): a lingering pocket cools, so term 2 grows SUBLINEARLY in
     τ_core (far-flank ratio ≈×1.29) vs rung-15's LINEAR D(u)·EI (×1.51 = the dwell ratio exactly).
  2. FAR-FLANK EROSION (the headline): the cooling-limited dwell erodes rung-15's over-penetration
     secondary basin ~18-32%, into NEAR-DEGENERACY with the sharp C_opt notch (which SURVIVES —
     the composition excess still → 0 at C_opt, both immediate flanks up).
  3. NOT CLAIMED: which of the two near-degenerate optima is GLOBALLY lowest — it flips sign across
     the β-PDF quadrature (~5%), the φ>2 tail treatment, and the C_e regime (2%→21% over 0.20→0.15).
     So NO gate asserts a global-min LOCATION (unlike rung 15's GATE 3); the near-degeneracy gate
     asserts the GAP has collapsed vs rung 15, NEVER which well wins.

Coarse grids — SHAPE + DIRECTION, not digits (project ethos). Per-pocket quench is ~n_bell× costlier
than rung 15's single bell, so the bank of per-pocket trajectories is built ONCE (τ_core-independent)
and the cheap NO-only RK4 is re-run per J; the helper is pinned to production at the SAME resolution.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, Unmixedness, MixingPDF, QuenchPDF, PocketQuenchPDF, _F_STOICH, _HF_FUEL_DEFAULT,
    _equilibrium_composition, _primary_aft, _thermal_no, _quench_trajectory, _quench_no,
    _ideal_bell_ei, _beta_pdf_nodes_weights,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_PHI_P = 1.5              # the RQL rich primary (rungs 10-15)
_CE = 0.20               # jet entrainment constant — the ANCHORED regime (rungs 11-15; test_rung15)
# Coarse grids — per-pocket quench is expensive; the bank is built ONCE, RK4 is NO-only after.
_NG = 24                 # finite-quench trajectory resolution (each point re-equilibrates the majors)
_NSTEPS = 200            # RK4 steps for the NO ODE (cheap once the trajectory tab is cached)
_NB, _NQ = 48, 64        # per-pocket ξ-grid / β-PDF quadrature nodes


def _mix(J):
    return JetMixing(J=J, C_e=_CE, shape_n=2.0)


def _j_opt(cfg):
    """J_opt where C=(S/H)√J_opt = C_opt (H=0.10, the JetMixing default)."""
    return (cfg.C_opt * JetMixing(J=1.0).H / cfg.S) ** 2


_DP_CACHE = None


def _design_point():
    """Build the equilibrium engine once; read station-3/4 + the rich primary pool + the shared
    mean-field trajectory (term 1) + the PER-POCKET trajectory bank (term 2, τ_core-independent, built
    ONCE over a fixed ξ-grid). NO is trace ⇒ the cycle is bit-for-bit rung 6."""
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
        hf = g.hf_fuel_molar if g.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
        # term 1 — the rich mean-field bulk pool + its shared trajectory
        far_p = _PHI_P * _F_STOICH
        alpha = far / far_p
        T_p = _primary_aft(far_p, p, Tt3, hf)
        comp_p = _equilibrium_composition(far_p, T_p, p)
        n0 = alpha * _thermal_no(comp_p, T_p, p, _TAU, far_p).x_no * sum(comp_p.values())
        tab = _quench_trajectory(comp_p, T_p, alpha, far, Tt3, p, ngrid=_NG)
        # term 2 — the per-pocket bank over a fixed ξ-grid (mirrors _pocket_quench_mean_ei exactly)
        xi_max = (2.0 * _F_STOICH) / (1.0 + 2.0 * _F_STOICH)
        xi_grid = [xi_max * (i + 0.5) / _NB for i in range(_NB)]
        bank = []
        for xi in xi_grid:
            fl = xi / (1.0 - xi)
            if fl < far or fl / _F_STOICH > 2.0 + 1e-9 or fl <= 0.0:
                bank.append(("b", _ideal_bell_ei(fl, p, Tt3, hf, _TAU)))         # lean/tail: rung-15 bell
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


def _term2_16(dp, cfg, J):
    """Term 2 — ⟨EI_pocket_quench(ξ; τ_core(C))⟩_g on the cached bank. Mirrors the production
    `_pocket_quench_mean_ei` (tail=0; g→0 ⇒ single pocket at ξ̄). Returns (excess, max_a)."""
    C = cfg.C(_mix(J))
    g = cfg.segregation(C)
    tau_core = cfg.core_dwell(C)
    vals, max_a = [], 0.0
    for kind, payload in dp["bank"]:
        if kind == "q":
            comp, T_pk, al, n0k, tabk = payload
            q = _quench_no(comp, T_pk, al, dp["far"], dp["Tt3"], dp["p"], n0k, tau_core,
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
        t = (xi - xg[lo]) / (xg[hi] - xg[lo])
        return vals[lo] + t * (vals[hi] - vals[lo])

    if g <= 1e-9:
        return qb(dp["xibar"]), max_a
    nodes, w = _beta_pdf_nodes_weights(dp["xibar"], g, n_quad=cfg.n_quad)
    return sum(wi * qb(x) for wi, x in zip(w, nodes)), max_a


def _ei16(dp, cfg, J):
    """⟨EI⟩₁₆ = term1 (floor) + term2 (per-pocket quench PDF integral)."""
    excess, _ = _term2_16(dp, cfg, J)
    return _floor(dp, J) + excess


def _cfg():
    return PocketQuenchPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)


def _argmin(vals):
    return min(range(len(vals)), key=lambda i: vals[i])


# --------------------------------------------------------------------------- #
# GATE 1 — reduces: pocket_quench=None is rung 15; at C_opt it is the floor.   #
# --------------------------------------------------------------------------- #
def test_reduce_pocket_quench_none_is_rung15_path():
    # pocket_quench=None (default) leaves EVERY rung-16 field None and matches the plain call
    # bit-for-bit (the whole rung 1-15 suite staying green pins the shared path).
    dp = _design_point()
    g = dp["g"]
    for J in (9.0, 36.0):
        a = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J), quench_ngrid=_NG)
        b = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                        pocket_quench=None, quench_ngrid=_NG)
        for s in (a, b):
            assert s.pocket_quench is None and s.ei_no_pocket_quench is None and s.ei_no_pocket_excess is None
        assert a.ei_no_quenched == b.ei_no_quenched and a.max_a_quench == b.max_a_quench


def test_reduce_at_c_opt_is_finite_bulk_quench_no():
    # At C_opt the jet is perfectly mixed (g=0 ⇒ term 2 → the single lean pocket at ξ̄ ≈ 0), so
    # ⟨EI⟩₁₆ = the FINITE mean-field bulk quench NO (ei_no_quenched), NOT ≈0. THE reduce that
    # separates rung 16 (a finite floor) from rung 13's ≈0 — same as rung 15's second reduce.
    dp = _design_point()
    cfg = _cfg()
    s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P,
                          mixing=_mix(_j_opt(cfg)), pocket_quench=cfg,
                          quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert s.g_seg == 0.0 and abs(s.C_holdeman - cfg.C_opt) < 1e-12
    assert s.ei_no_quenched > 0.3, f"the bulk floor must be a FINITE non-trace value, got {s.ei_no_quenched}"
    rel = abs(s.ei_no_pocket_quench - s.ei_no_quenched) / s.ei_no_quenched
    assert rel < 1e-3, f"at C_opt ⟨EI⟩₁₆ must equal the finite bulk quench NO to <0.1%, rel={rel:.2e}"


def test_zoned_nox_matches_ei16_helper():
    # Pin the fast (cached-bank) helper to the PRODUCTION zoned_nox path at the SAME resolution, so the
    # sweep gates below exercise the SAME arithmetic the production code does.
    dp = _design_point()
    cfg = _cfg()
    J = 36.0
    s = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                          pocket_quench=cfg, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    h = _ei16(dp, cfg, J)
    assert abs(s.ei_no_pocket_quench - h) < 1e-6 * max(h, 1e-12), \
        f"helper {h} vs production {s.ei_no_pocket_quench}"
    assert abs(s.C_holdeman - cfg.C(_mix(J))) < 1e-12 and abs(s.g_seg - cfg.segregation(cfg.C(_mix(J)))) < 1e-12


# --------------------------------------------------------------------------- #
# GATE 2 — the composition excess vanishes AT C_opt, both immediate flanks up. #
# --------------------------------------------------------------------------- #
def test_excess_vanishes_at_c_opt_flanks_up():
    # The C_opt notch SURVIVES the per-pocket quench: term 2 → 0 at C_opt (g→0, the single lean pocket
    # at ξ̄), and BOTH immediate flanks lift above the floor (segregation kicks in off-optimum).
    dp = _design_point()
    cfg = _cfg()
    J_opt = _j_opt(cfg)
    e_opt = _ei16(dp, cfg, J_opt)
    exc_opt, _ = _term2_16(dp, cfg, J_opt)
    assert exc_opt < 0.01 * _floor(dp, J_opt), f"term 2 must vanish AT C_opt, got {exc_opt}"
    e_under = _ei16(dp, cfg, J_opt / 1.7)
    e_over = _ei16(dp, cfg, J_opt * 1.7)
    assert e_under > e_opt and e_over > e_opt, \
        f"both immediate flanks must lift above the C_opt notch: under={e_under}, opt={e_opt}, over={e_over}"


# --------------------------------------------------------------------------- #
# GATE 3 — THE MECHANISM: per-pocket cooling makes term 2 SUBLINEAR in dwell.  #
# --------------------------------------------------------------------------- #
def test_sublinear_dwell_the_mechanism():
    # rung-15 term 2 = D(u)·⟨EI_bell⟩ scales EXACTLY with the dwell factor D (LINEAR); rung-16 term 2
    # carries the dwell INSIDE the cooling quench, so it grows SUBLINEARLY across the far flank.
    dp = _design_point()
    cfg = _cfg()
    qp = QuenchPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    g = dp["g"]

    def t2_15(J):
        return g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                           pdf_quench=qp, quench_ngrid=_NG, quench_nsteps=_NSTEPS).ei_no_pdf_excess

    def t2_16(J):
        return _term2_16(dp, cfg, J)[0]

    ratio_15 = t2_15(625.0) / t2_15(144.0)
    ratio_16 = t2_16(625.0) / t2_16(144.0)
    # rung 15's excess ratio IS the dwell-factor ratio (the linearisation, D∝(1+b_u·u)).
    D_ratio = qp.dwell_factor(qp.C(_mix(625.0)), _TAU) / qp.dwell_factor(qp.C(_mix(144.0)), _TAU)
    assert abs(ratio_15 - D_ratio) < 0.02 * D_ratio, \
        f"rung-15 term 2 must scale LINEARLY with dwell: ratio={ratio_15}, D_ratio={D_ratio}"
    assert ratio_16 < 0.95 * ratio_15, \
        f"rung-16 term 2 must be SUBLINEAR (cooling): ratio_16={ratio_16} vs ratio_15={ratio_15}"


# --------------------------------------------------------------------------- #
# GATE 4 — THE HEADLINE: the far over-penetration flank ERODES vs rung 15.     #
# --------------------------------------------------------------------------- #
def test_far_flank_erosion_vs_rung15():
    # The cooling-limited dwell drops EI on the whole over-penetration flank vs rung-15's linear climb.
    dp = _design_point()
    cfg = _cfg()
    qp = QuenchPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    g = dp["g"]
    for J in (144.0, 225.0, 400.0, 625.0):
        ei15 = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                           pdf_quench=qp, quench_ngrid=_NG, quench_nsteps=_NSTEPS).ei_no_pdf_quench
        ei16 = _ei16(dp, cfg, J)
        assert ei16 < 0.93 * ei15, f"J={J}: rung-16 must erode the far flank vs rung 15: EI16={ei16}, EI15={ei15}"


# --------------------------------------------------------------------------- #
# GATE 5 — the far-flank CLIMB flattens: rung-15's linear climb → rung-16 flat.#
# --------------------------------------------------------------------------- #
def test_far_flank_climb_flattens_vs_rung15():
    # The resolution-robust face of the erosion / near-degeneracy: rung-15's LINEAR dwell makes the far
    # over-penetration flank CLIMB (EI rises with J from 144→625), while rung-16's SUBLINEAR per-pocket
    # dwell FLATTENS it (the cooling saturates the re-making). Same two endpoints, OPPOSITE far-flank
    # slope. We assert the SLOPE CONTRAST — NEVER which well is the global min (that is within the
    # quadrature/tail/C_e ambiguity, so there is deliberately NO argmin assertion; contrast rung-15's
    # GATE 3). rung-15's climb is exactly what put its basin ABOVE C_opt; flattening it is what brings
    # the two into near-degeneracy at fine resolution.
    dp = _design_point()
    cfg = _cfg()
    qp = QuenchPDF(S=0.0625, n_bell=_NB, n_quad=_NQ)
    g = dp["g"]

    def ei15(J):
        return g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                           pdf_quench=qp, quench_ngrid=_NG, quench_nsteps=_NSTEPS).ei_no_pdf_quench

    lo, hi = 144.0, 625.0
    climb15 = ei15(hi) / ei15(lo) - 1.0
    climb16 = _ei16(dp, cfg, hi) / _ei16(dp, cfg, lo) - 1.0
    assert climb15 > 0.10, f"rung 15 far flank must CLIMB (linear dwell): climb15={climb15:.3f}"
    assert climb16 < 0.5 * climb15, \
        f"rung 16 far flank must FLATTEN (sublinear cooling): climb16={climb16:.3f} vs climb15={climb15:.3f}"


# --------------------------------------------------------------------------- #
# GATE 6 — clamp dormancy over the per-pocket streams (max_a < 1 everywhere).  #
# --------------------------------------------------------------------------- #
def test_clamp_dormant_over_pockets():
    # The per-pocket integrator is CLAMP-FREE (a super-eq pocket would roll over) — but DORMANT at this
    # design point: max_a < 1 across the sweep AND across every pocket (the difference is cooling, not
    # super-eq rollover). Production folds the pocket max_a into max_a_quench.
    dp = _design_point()
    cfg = _cfg()
    g = dp["g"]
    for J in (16.0, 144.0, 625.0):
        s = g.zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(J),
                        pocket_quench=cfg, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
        assert s.max_a_quench < 1.0, f"J={J}: clamp must stay dormant (max_a={s.max_a_quench})"


# --------------------------------------------------------------------------- #
# GATE 7 — cycle untouched by a pocket_quench call.                           #
# --------------------------------------------------------------------------- #
def test_cycle_untouched_by_pocket_quench_call():
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)

    r1 = run()
    st3, st4 = r1.stations["3"], r1.stations["4"]
    far1, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    g.zoned_nox(far1, Tt3, Tt4, p, _PHI_P, mixing=_mix(36.0),
                pocket_quench=PocketQuenchPDF(n_bell=_NB, n_quad=_NQ), quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert run().stations["4"].far == far1, "pocket_quench call perturbed the cycle far — must stay rung-6"


# --------------------------------------------------------------------------- #
# GATE 8 — require-mixing + ≤1-of-four mutual-exclusivity + positivity guards. #
# --------------------------------------------------------------------------- #
def test_pocket_quench_requires_mixing():
    dp = _design_point()
    try:
        dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P,
                          pocket_quench=PocketQuenchPDF(), quench_ngrid=_NG)   # no mixing
    except AssertionError:
        return
    raise AssertionError("pocket_quench without mixing must be rejected (needs J and H + τ_mean)")


def test_pocket_quench_mutually_exclusive_with_the_other_three():
    dp = _design_point()
    for extra in (dict(pdf=MixingPDF()), dict(unmixedness=Unmixedness()), dict(pdf_quench=QuenchPDF())):
        try:
            dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(16.0),
                              pocket_quench=PocketQuenchPDF(), quench_ngrid=_NG, **extra)
        except AssertionError:
            continue
        raise AssertionError(f"pocket_quench + {list(extra)[0]} must be rejected (same variance physics)")


def test_pocketquenchpdf_positivity_guards():
    PocketQuenchPDF()                                          # defaults accepted
    PocketQuenchPDF(k_g=0.0)                                   # k_g=0 ⇒ g≡0 (floor only), allowed
    PocketQuenchPDF(b_u=0.0)                                   # b_u=0 ⇒ flat dwell, allowed
    for bad in (dict(S=0.0), dict(S=-0.1), dict(C_opt=0.0), dict(tau_res=0.0), dict(k_g=-0.1),
                dict(b_u=-0.1), dict(g_max=0.0), dict(g_max=1.0), dict(n_bell=1), dict(n_quad=0)):
        try:
            PocketQuenchPDF(**bad)
        except AssertionError:
            continue
        raise AssertionError(f"PocketQuenchPDF({bad}) should be rejected (positivity/range guard)")


# --------------------------------------------------------------------------- #
# GATE 9 — g(C)/u(C)/τ_core(C) are the Holdeman kinks: 0 at C_opt, both flanks.#
# --------------------------------------------------------------------------- #
def test_kinks_zero_at_optimum():
    cfg = PocketQuenchPDF()
    assert cfg.segregation(cfg.C_opt) == 0.0 and cfg._u(cfg.C_opt) == 0.0, "g,u must be 0 at C_opt"
    lo, hi = cfg.segregation(cfg.C_opt / 1.3), cfg.segregation(cfg.C_opt * 1.3)
    assert lo > 0.0 and hi > 0.0, f"g must rise on BOTH flanks: {lo}, {hi}"
    assert abs(cfg.segregation(cfg.C_opt / 1.4) - cfg.segregation(cfg.C_opt * 1.4)) < 1e-12   # symmetric in ln C
    assert cfg.segregation(cfg.C_opt * 1.05) > 0.0                                            # KINKED slope
    assert cfg.segregation(cfg.C_opt * 1e6) == cfg.g_max                                      # capped
    # τ_core GROWS off-optimum (both flanks) and is τ_res at C_opt (the ABSOLUTE dwell, no τ_ref ratio).
    assert abs(cfg.core_dwell(cfg.C_opt) - cfg.tau_res) < 1e-12
    assert cfg.core_dwell(cfg.C_opt * 1.3) > cfg.core_dwell(cfg.C_opt)
    assert cfg.core_dwell(cfg.C_opt / 1.3) > cfg.core_dwell(cfg.C_opt)


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-16 gates passed")
