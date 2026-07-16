"""Rung 24 — the LOCALLY-RESOLVED MIXING TIME (the ceiling rungs 11–23 deferred BY NAME).

Rungs 11–23 all ran on ONE GLOBAL τ_mix=H/(C_e√J·U_c). Rung 23's §9 named the successor and its
hypothesis: a scalar-dissipation field giving each cell its OWN rate "could restore an off-optimum
dwell GROWTH that pins the emissions optimum non-circularly". Rung 24 ASKS that question — and the
answer is a SPLIT, whose HEADLINE is the NEGATIVE half:

    ω(y,z) = D_t·|∇ξ|²/var,  D_t = σ²/(2τ_mix)  [REUSED — no new constant]
    τ_cell = τ_mix·[1 − 1/E + 1/u],  u = ω·τ_mix = σ²|∇ξ|²/(2var)   ← τ_mix CANCELS: analytic, no nt
    ⟨τ⟩(J) = τ_mix(J) · F(C)                                        ← EXACT: scale × shape, separated

THE ROBUST LESSON (this file certifies ONLY these — see docs/rung24-spec.md):
  1. THE REDUCE — spatial_local=None ⇒ prior path untouched; g is IDENTICAL to rung-22's BY
     CONSTRUCTION (the same terminal field — 1e-12, not the <1% rung 23 needed for its re-derivation).
  2. THE FACTORIZATION — F is independent of τ_mix EXACTLY, and ⟨τ⟩ scales linearly in it. This is
     what makes "scale vs shape" a decomposition rather than a story.
  3. THE POSITIVE — F(C) is U-SHAPED with its minimum AT C_opt: the off-optimum dwell GROWTH rung 16
     IMPOSED (τ_res(1+b_u|ln(C/C_opt)|)), here DERIVED from gradients. NON-CIRCULAR: ⟨|∇ξ|²⟩ — which
     carries NO g algebraically — is MAXIMAL at C_opt (ω's 1/g factor cannot be what places it).
     The (H/S)² shift (rung-22's signature) is inherited by the DWELL.
  4. THE NEGATIVE (THE HEADLINE) — ⟨EI⟩(J) stays MONOTONE-decreasing on the real chemistry: F's ~39%
     U loses to τ_mix's ~20× swing. THE SCALE SWAMPS THE SHAPE. The emissions C_opt pin is STILL not
     recovered; rung 24 localizes the RATE, not the SCALE.
  5. NOT CLAIMED — the emissions global-min LOCATION (rung 16's GATE 3 and rung 23 both declined it;
     rung 24 declines it too, now having MEASURED why). Nor F's magnitude (rides on k_p/k_y/k_z), nor
     a global-argmin claim at S>H (the inherited rung-22 wrap-around; docs/rung24-spec.md §6).

Coarse grids — SHAPE + DIRECTION, not digits (project ethos). The geometry gates (1–3) are pure
field functionals and cost nothing; only GATE 4 pays for the per-pocket chemistry.
"""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, SpatialLocalPDF, SpatialPDF, SpatialDwellPDF,
    _F_STOICH, _HF_FUEL_DEFAULT, _spatial_local_field, _spatial_segregation,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_PHI_P = 1.5
_CE = 0.20
_U_C = 75.0
_H = 0.10
_S = 0.0625
_FAR = 0.02718          # the rung-16/22/23 design point (re-derived by _design_point below)
_NY = _NZ = 32
_NG = 24
_NSTEPS = 200
_NB, _NQ = 40, 160      # n_quad=160: 56 trips the KNOWN β-PDF guard at mid-J (rung-23 anchor §7)

_DP_CACHE = None


def _mix(J):
    return JetMixing(J=J, C_e=_CE, U_c=_U_C, shape_n=2.0)


def _cfg(**kw):
    d = dict(S=_S, ny=_NY, nz=_NZ, n_bell=_NB, n_quad=_NQ)
    d.update(kw)
    return SpatialLocalPDF(**d)


def _field(J, S=_S, tau_mix=None, ny=_NY, nz=_NZ):
    """(g, tau_of_xi, F) straight from the helper — pure geometry, no engine, no chemistry."""
    m = _mix(J)
    return _spatial_local_field(_FAR, _PHI_P, S, _H, J, tau_mix if tau_mix is not None else m.tau_q,
                               ny=ny, nz=nz)


def _design_point():
    """Build the equilibrium engine ONCE (the cycle is bit-for-bit rung 6 — NO is a trace diagnostic)."""
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        _DP_CACHE = dict(g=g, Tt3=st3.Tt, Tt4=st4.Tt, far=st4.far, p=st4.pt)
    return _DP_CACHE


def _zoned(J, **kw):
    dp = _design_point()
    return dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], tau=_TAU,
                             phi_primary=_PHI_P, mixing=_mix(J), quench_ngrid=_NG,
                             quench_nsteps=_NSTEPS, **kw)


# --------------------------------------------------------------------------- #
# GATE 1 — the REDUCE: spatial_local=None is the prior path; g == rung 22's BY CONSTRUCTION.
# --------------------------------------------------------------------------- #
def test_reduce_spatial_local_none_is_prior_path():
    """spatial_local=None ⇒ the branch is never entered and every rung-24 field stays None."""
    s = _zoned(16, spatial_local=None)
    assert s.spatial_local is None
    for f in ("g_spatial_local", "f_shape", "tau_mean_local", "ei_no_spatial_local",
              "ei_no_spatial_local_meanfield", "ei_no_spatial_local_excess", "corr_ratio_local"):
        assert getattr(s, f) is None, f"{f} set despite spatial_local=None"


@pytest.mark.parametrize("J", [4, 16, 64])
def test_g_identical_to_rung22_by_construction(J):
    """The TERMINAL field is rung-22's EXACTLY — rung 24 changes only the PATH to it. So g must match
    `_spatial_segregation` to MACHINE precision, not to the <1% rung 23 needed (it re-derived the
    terminal field through a time development; rung 24 reuses it outright)."""
    g24, _, _ = _field(J)
    g22 = _spatial_segregation(_FAR, _PHI_P, _S, _H, J, ny=_NY, nz=_NZ)
    assert abs(g24 - g22) < 1e-12, f"g drifted from rung 22 at J={J}: {g24} vs {g22}"


def test_production_width_matches_spatial_pdf():
    """The shipped zoned_nox width == SpatialPDF's at the same grid (both ARE the terminal field)."""
    s = _zoned(16, spatial_local=_cfg())
    s22 = _zoned(16, spatial=SpatialPDF(S=_S, ny=_NY, nz=_NZ, n_quad=_NQ))
    assert abs(s.g_spatial_local - s22.g_spatial) < 1e-12


@pytest.mark.parametrize("J", [4, 16, 64])
def test_g_below_two_stream_ceiling(J):
    """The rung-18 two-stream ceiling still bounds the resolved field (carried from rungs 22/23)."""
    s = _zoned(J, spatial_local=_cfg())
    assert s.g_spatial_local < s.g_ceiling


# --------------------------------------------------------------------------- #
# GATE 2 — THE FACTORIZATION: τ_mix cancels out of the rate ⇒ ⟨τ⟩ = τ_mix·F EXACTLY.
# --------------------------------------------------------------------------- #
@pytest.mark.parametrize("J", [4, 16, 64])
def test_f_shape_is_independent_of_tau_mix(J):
    """u = ω·τ_mix = σ²|∇ξ|²/(2·var) — NO τ_mix. So F is a PURE FIELD FUNCTIONAL: scaling τ_mix by
    ×0.2/×5 must leave F bit-identical. This is what makes 'the scale swamps the shape' a
    DECOMPOSITION rather than a narrative — the two factors are genuinely separable."""
    base = _mix(J).tau_q
    _, _, f1 = _field(J, tau_mix=base)
    _, _, f2 = _field(J, tau_mix=0.2 * base)
    _, _, f3 = _field(J, tau_mix=5.0 * base)
    assert abs(f2 - f1) < 1e-12 and abs(f3 - f1) < 1e-12, f"F moved with τ_mix at J={J}"


@pytest.mark.parametrize("J", [4, 16])
def test_tau_scales_linearly_in_tau_mix(J):
    """The other half of the factorization: with F fixed, every cell's τ is ∝ τ_mix exactly."""
    base = _mix(J).tau_q
    _, t1, _ = _field(J, tau_mix=base)
    _, t5, _ = _field(J, tau_mix=5.0 * base)
    xibar = _FAR / (1.0 + _FAR)
    for xi in (0.5 * xibar, xibar, 2.0 * xibar, 3.0 * xibar):
        assert abs(t5(xi) - 5.0 * t1(xi)) < 1e-15, f"τ(ξ={xi}) not linear in τ_mix at J={J}"


# --------------------------------------------------------------------------- #
# GATE 3 — THE POSITIVE: F(C) U-shaped with its min AT C_opt, and it is NOT the 1/g factor.
# --------------------------------------------------------------------------- #
def test_f_shape_is_u_shaped_with_minimum_at_c_opt():
    """THE DERIVED OFF-OPTIMUM DWELL GROWTH — the qualitative shape rung 16 IMPOSED as
    τ_core=τ_res(1+b_u|ln(C/C_opt)|), here derived from the plume's own gradients with NO C_opt,
    NO τ_res, NO b_u. At S=0.0625, H=0.10, C_opt=1/(4k_p²)≈2.5 ⇒ J_opt=16."""
    Js = [4, 9, 16, 36, 64]
    F = [_field(J)[2] for J in Js]
    assert Js[F.index(min(F))] == 16, f"argmin F at J={Js[F.index(min(F))]}, expected 16 (C_opt)"
    # BOTH flanks rise — a U, not a monotone trend (this is the whole disagreement with rung 23,
    # whose shared schedule gives F≈const).
    assert F[0] > F[2] and F[1] > F[2], "under-penetration flank does not rise"
    assert F[3] > F[2] and F[4] > F[2], "over-penetration flank does not rise"
    assert abs(_cfg().C_opt() - 2.5) < 0.02       # the DERIVED optimum, not a knob


def test_gradients_locate_c_opt_the_g_free_witness():
    """THE CIRCULARITY KILL TEST (docs/plans/rung24-anchor-local-mixing-time.md §2). ω carries an
    explicit 1/g (var=g·ξ̄(1−ξ̄)) and rung 22 ALREADY mins g at C_opt — so "argmin F == argmin g" is
    a TELL, not a confirmation. The G-FREE WITNESS: ⟨|∇ξ|²⟩ contains no g algebraically, and it is
    MAXIMAL at C_opt. The gradients place the optimum; the 1/g coupling only amplifies it.

    (Physically: at C_opt the jet fills to mid-height so residual structure sits at the plume's own
    scale σ — fine ⇒ steep ⇒ fast ⇒ short dwell; off-optimum the air piles into WALL-SCALE slabs —
    coarse ⇒ shallow ⇒ slow ⇒ long dwell. A property of the FIXED-σ cartoon, not a general law.)"""
    xibar = _FAR / (1.0 + _FAR)
    far_p = _PHI_P * _F_STOICH
    xi_p = far_p / (1.0 + far_p)

    def mean_grad_sq(J, ny=_NY, nz=_NZ):
        """⟨|∇ξ|²⟩ of the terminal field — rebuilt here WITHOUT any variance normalization."""
        delta = 0.316 * math.sqrt(_S * _H) * J ** 0.25
        sig_y, sig_z = 0.28 * _H, 0.28 * _S
        ys = [(i + 0.5) * _H / ny for i in range(ny)]
        zs = [(j + 0.5) * _S / nz for j in range(nz)]
        ay = [sum(math.exp(-((y - c) ** 2) / (2 * sig_y ** 2))
                  for c in (-delta, delta, 2 * _H - delta, 2 * _H + delta)) for y in ys]
        az = [sum(math.exp(-((z - _S / 2 - m * _S) ** 2) / (2 * sig_z ** 2))
                  for m in (-1, 0, 1)) for z in zs]
        may, maz = sum(ay) / ny, sum(az) / nz
        ayh, azh = [a / may for a in ay], [a / maz for a in az]
        beta_bar = (xi_p - xibar) / xi_p

        def mean_at(s):
            return sum(xi_p * (1.0 - min(1.0, max(0.0, s * beta_bar * a * b)))
                       for a in ayh for b in azh) / (ny * nz)

        lo, hi = 0.0, 50.0
        for _ in range(60):
            s = 0.5 * (lo + hi)
            if mean_at(s) > xibar:
                lo = s
            else:
                hi = s
        s_star = 0.5 * (lo + hi)
        xi = [[xi_p * (1.0 - min(1.0, max(0.0, s_star * beta_bar * a * b))) for b in azh] for a in ayh]
        dy, dz = _H / ny, _S / nz
        tot = 0.0
        for i in range(ny):
            im, ip = max(0, i - 1), min(ny - 1, i + 1)
            for j in range(nz):
                jm, jp = (j - 1) % nz, (j + 1) % nz
                gy = (xi[ip][j] - xi[im][j]) / ((ip - im) * dy)
                gz = (xi[i][jp] - xi[i][jm]) / (2 * dz)
                tot += gy * gy + gz * gz
        return tot / (ny * nz)

    Js = [4, 9, 16, 36, 64]
    grads = [mean_grad_sq(J) for J in Js]
    assert Js[grads.index(max(grads))] == 16, (
        f"⟨|∇ξ|²⟩ is maximal at J={Js[grads.index(max(grads))]}, not at C_opt (J=16) — the kill test "
        f"FAILS and F's minimum would be the 1/g factor's doing, not the gradients'."
    )


def test_f_optimum_shifts_as_h_over_s_squared():
    """Rung-22's SIGNATURE, inherited by the DWELL: the optimum is a function of the Holdeman group
    C=(S/H)√J ALONE, so HALVING the spacing must QUADRUPLE J_opt (C_opt fixed at ≈2.5)."""
    for S, J_opt, Js in ((0.03125, 64, [16, 36, 64, 144, 256]),
                         (0.0625, 16, [4, 9, 16, 36, 64])):
        F = [_field(J, S=S)[2] for J in Js]
        got = Js[F.index(min(F))]
        assert got == J_opt, f"S={S}: argmin F at J={got}, expected {J_opt} (the (H/S)² shift)"
        assert abs((S / _H) * math.sqrt(got) - 2.5) < 0.02      # ... and it IS C=2.5 at both


# --------------------------------------------------------------------------- #
# GATE 4 — THE NEGATIVE HEADLINE: <EI>(J) stays MONOTONE on the REAL chemistry.
# --------------------------------------------------------------------------- #
def test_ei_stays_monotone_the_emissions_optimum_is_not_recovered():
    """THE HEADLINE, and it is a NEGATIVE (the rung-18 tradition). F(C) grows off-optimum — the
    derived dwell growth is REAL and in the RIGHT PLACE — but ⟨τ⟩ = τ_mix(J)·F(C) and τ_mix's ~20×
    1/√J swing SWAMPS F's ~39% U. So ⟨EI⟩ never turns back up: THE EMISSIONS C_opt PIN IS STILL NOT
    RECOVERED, and rung 23's conclusion survives — now for a measured reason rather than a guess.

    This is asserted on the ACTUAL per-pocket quench, not inferred from ⟨τ⟩: EI is nonlinear in τ
    and richness-weighted, so the inference had to be checked."""
    Js = [4, 9, 16, 36, 64]
    ei = [_zoned(J, spatial_local=_cfg()).ei_no_spatial_local for J in Js]
    for k in range(len(Js) - 1):
        assert ei[k] > ei[k + 1], (
            f"⟨EI⟩ turned up between J={Js[k]} and J={Js[k + 1]} ({ei[k]:.4f} → {ei[k + 1]:.4f}) — "
            f"if this fires, the locally-resolved rate DID pin the emissions optimum and rung 24's "
            f"headline negative is wrong."
        )
    assert min(ei) == ei[-1], "the ⟨EI⟩ minimum is not at the strongest jet — the negative fails"


def test_scale_swamps_shape_quantified():
    """WHY the negative holds, as numbers — the whole rung in one test, on the FULL geometry sweep
    (pure field functionals: no chemistry, so the wide sweep is free).

    The load-bearing claim is NOT a magic margin — it is that the PRODUCT ⟨τ⟩=τ_mix(J)·F(C) still
    FALLS monotonically despite F's U. That is precisely what makes ⟨EI⟩ monotone (GATE 4), so it is
    what gets asserted; the swing ratio is reported alongside as the reason."""
    Js = [1, 4, 9, 16, 36, 64, 144, 400]
    F = [_field(J)[2] for J in Js]
    tmix = [_mix(J).tau_q for J in Js]
    tau = [t * f for t, f in zip(tmix, F)]
    assert Js[F.index(min(F))] == 16                       # F's U still mins at C_opt over the wide sweep
    for k in range(len(Js) - 1):                            # THE POINT: the product falls anyway
        assert tau[k] > tau[k + 1], (
            f"⟨τ⟩ turned UP between J={Js[k]} and J={Js[k+1]} ({tau[k]*1e3:.4f} → {tau[k+1]*1e3:.4f} ms) "
            f"— F's U would then be beating the 1/√J scale, and rung 24's negative would flip."
        )
    f_swing, t_swing = max(F) / min(F), max(tmix) / min(tmix)
    assert t_swing > 3.0 * f_swing, (                       # ~20× vs ~1.4× — the honest margin
        f"τ_mix swing ({t_swing:.2f}×) no longer dwarfs the F swing ({f_swing:.2f}×) — the "
        f"'scale swamps shape' explanation of the negative would need re-deriving."
    )


def test_local_rate_moves_ei_only_modestly_vs_rung23():
    """The whole locally-resolved upgrade is worth only a few % of ⟨EI⟩ — the honest scale of what
    localizing the RATE (but not the SCALE) buys. Certified as a BOUND, not a value."""
    for J in (4, 16):
        e24 = _zoned(J, spatial_local=_cfg()).ei_no_spatial_local
        e23 = _zoned(J, spatial_dwell=SpatialDwellPDF(S=_S, ny=_NY, nz=_NZ, nt=24,
                                                      n_bell=_NB, n_quad=_NQ)).ei_no_spatial_dwell
        assert abs(e24 / e23 - 1.0) < 0.10, f"J={J}: rung 24 moved ⟨EI⟩ {100*(e24/e23-1):.1f}% vs rung 23"


# --------------------------------------------------------------------------- #
# GATE 5 — what rung 24 DELIBERATELY does not claim (mirroring rungs 16 and 23).
# --------------------------------------------------------------------------- #
def test_does_not_claim_the_emissions_global_min_location():
    """Rung 16's GATE 3 declined the emissions global-min LOCATION; rung 23 declined it; rung 24
    declines it too — but now having MEASURED the reason (the shape grows, the scale wins). What IS
    pinned at C_opt is the WIDTH g (rung 22's uniformity result) and now the dwell SHAPE F — NOT
    ⟨EI⟩. This test asserts the DISAGREEMENT: the two optima are in DIFFERENT places, deliberately."""
    Js = [4, 9, 16, 36, 64]
    F = [_field(J)[2] for J in Js]
    ei = [_zoned(J, spatial_local=_cfg()).ei_no_spatial_local for J in Js]
    assert Js[F.index(min(F))] == 16, "F's min moved off C_opt"
    assert Js[ei.index(min(ei))] != 16, (
        "⟨EI⟩'s min landed AT C_opt — that would be the emissions pin rung 24 explicitly does NOT "
        "claim; if this fires, the negative headline needs re-deriving, not the test relaxing."
    )


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
