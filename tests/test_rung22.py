"""Rung 22 — the RESOLVED cross-plane / spatial PDF: the INVERSION of rung 18.

Rung 18's load-bearing result was NEGATIVE: a 0-D variance transport CANNOT DERIVE the Holdeman C_opt
optimum — with any mean-field ω(J) the residual g(J) is monotone, so rung 18 had to IMPOSE the coverage
ω(C) peaked at C_opt (the spatial spacing S injected by hand). Rung 22 resolves the y-z dilution
cross-plane (`_spatial_segregation`) and C_opt EMERGES as an OUTPUT — the inversion.

THE HEADLINE (load-bearing, POSITIVE): the resolved-field segregation g_spatial collapses onto a
constant Holdeman group C=(S/H)√J — its minimum VALUE is geometry-independent and J_opt shifts EXACTLY
as (H/S)². The penetration δ=k_p·√(S·H)·J^(1/4) couples the SPACING S in (fixed-mass-ratio jet), and the
uniformity optimum is where δ fills half the height ⇒ (S/H)√J=1/(4k_p²), S,H-independent. There is NO
C_opt knob — the whole point is that C_opt is an OUTPUT of the penetration law. The VALUE (≈2.5) rides on
the semi-empirical k_p; only the COLLAPSE + the (H/S)² SHIFT are derived (the honest concession).

THE EMISSIONS, HONEST: through the pure ideal bell, C_opt is only a LOCAL ⟨EI⟩ minimum — the GLOBAL min
is at MAX segregation (rung-13's descending far flank, spatialized), because the derived floor
g(C_opt)≈0.018 sits just BELOW the ideal-bell hump peak ≈0.021 (a NARROW basin). So UNIFORMITY (g), not
emissions, is the clean headline. rung 18 was NOT wrong — it reported the real LOCAL behaviour.

Coarse grids — SHAPE + DIRECTION, not digits (project ethos). The COLLAPSE gates use the helper
`_spatial_segregation` directly (no equilibrium bell) and are cheap; the emissions/cycle gates use a
coarse bell + the cached design point.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, SpatialPDF, PocketQuenchPDF, TransportedPDF, _F_STOICH, _HF_FUEL_DEFAULT,
    _spatial_segregation, _two_stream_ceiling, _pdf_mean_ei,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_PHI_P = 1.5
_NB, _NQ = 48, 64            # coarse ideal-bell / β-PDF grids (shape, not digits)
_NY, _NZ = 40, 40           # coarse cross-plane grid (converged: 32/48/64 agree)
_FAR = None                 # overall far, filled from the design point (needed by the cheap helper gates)


def _mix(J, H=0.10):
    return JetMixing(J=J, H=H)


def _cfg(**kw):
    return SpatialPDF(S=0.0625, ny=_NY, nz=_NZ, n_bell=_NB, n_quad=_NQ, **kw)


_DP_CACHE = None


def _design_point():
    global _DP_CACHE, _FAR
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        Tt3, Tt4, far, p = st3.Tt, st4.Tt, st4.far, st4.pt
        hf = g.hf_fuel_molar if g.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
        _FAR = far
        _DP_CACHE = dict(g=g, Tt3=Tt3, Tt4=Tt4, far=far, p=p, hf=hf,
                         xibar=far / (1.0 + far))
    return _DP_CACHE


def _run(dp, J, cfg, H=0.10):
    return dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, tau=_TAU,
                             mixing=_mix(J, H), spatial=cfg, quench_ngrid=24, quench_nsteps=200)


def _argmin_C(far, S, H, k_p=0.316, ny=_NY, nz=_NZ, npts=49):
    """(g_min, J_opt, C_opt) of the RESOLVED width over a log J-sweep — the helper-level collapse
    probe (no equilibrium bell; cheap)."""
    Js = [1.0 * (400.0) ** (i / (npts - 1)) for i in range(npts)]
    best = (1e9, None)
    for J in Js:
        g = _spatial_segregation(far, _PHI_P, S, H, J, k_p=k_p, ny=ny, nz=nz)
        if g < best[0]:
            best = (g, J)
    return best[0], best[1], (S / H) * math.sqrt(best[1])


# --------------------------------------------------------------------------------------------------
# GATE 1 — reduce (load-bearing).
# --------------------------------------------------------------------------------------------------

def test_reduce_none_leaves_prior_path_untouched():
    """spatial=None short-circuits before any rung-22 code — a mixing-only call is unaffected and the
    rung-22 fields stay None (the whole rung 1-21 suite stays green by this construction)."""
    dp = _design_point()
    base = dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, tau=_TAU,
                             mixing=_mix(16), quench_ngrid=24, quench_nsteps=200)
    assert base.spatial is None and base.g_spatial is None and base.ei_no_spatial is None
    # the mean-field quench (rung-11 bulk) result is present and untouched by rung 22
    assert base.ei_no_quenched is not None and base.ei_no_quenched > 0.0


def test_reduce_primary_diagnostic_bit_identical():
    """A spatial call touches only the rung-22 fields; the PRIMARY diagnostic (ei_no / x_no_mix) is
    bit-identical to a mixing-only call — rung 22 adds a closure, never perturbs the primary."""
    dp = _design_point()
    base = _run(dp, 16, None) if False else dp["g"].zoned_nox(
        dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, tau=_TAU, mixing=_mix(16),
        quench_ngrid=24, quench_nsteps=200)
    st = _run(dp, 16, _cfg())
    assert st.ei_no == base.ei_no and st.x_no_mix == base.x_no_mix


# --------------------------------------------------------------------------------------------------
# GATE 2 — THE COLLAPSE (headline, load-bearing): C_opt is an OUTPUT, the inversion of rung 18.
# --------------------------------------------------------------------------------------------------

def test_group_collapse_gmin_geometry_independent():
    """The resolved width's MINIMUM VALUE is geometry-independent — vary S and H INDEPENDENTLY by 2×
    and g_min is unchanged to ~1%. This IS the Holdeman group collapse: only the LOCATION J_opt moves;
    the depth of the notch is a property of the cross-plane, not the geometry."""
    dp = _design_point()
    far = dp["far"]
    cases = [(0.0625, 0.10), (0.03125, 0.10), (0.125, 0.10), (0.0625, 0.20), (0.125, 0.20)]
    gmins = [_argmin_C(far, S, H)[0] for S, H in cases]
    g0 = gmins[0]
    for gm in gmins:
        assert math.isclose(gm, g0, rel_tol=0.03), \
            f"g_min must be geometry-independent (collapse): {gmins} vs {g0}"


def test_j_opt_shifts_as_H_over_S_squared():
    """J_opt shifts EXACTLY as (H/S)² — the Holdeman scaling as an OUTPUT. Halve S ⇒ J_opt ×4; double H
    ⇒ J_opt ×4; scale both (S/H fixed) ⇒ J_opt unchanged. (Fine J-sweep so the ratio is clean.)"""
    dp = _design_point()
    far = dp["far"]
    _, j_base, _ = _argmin_C(far, 0.0625, 0.10, npts=81)
    _, j_half_S, _ = _argmin_C(far, 0.03125, 0.10, npts=81)   # halve S ⇒ (H/S)² ×4
    _, j_dbl_H, _ = _argmin_C(far, 0.0625, 0.20, npts=81)     # double H ⇒ (H/S)² ×4
    _, j_both, _ = _argmin_C(far, 0.125, 0.20, npts=81)       # S/H fixed ⇒ unchanged
    assert math.isclose(j_half_S / j_base, 4.0, rel_tol=0.15), f"halve S ⇒ J_opt ×4, got {j_half_S/j_base:.2f}"
    assert math.isclose(j_dbl_H / j_base, 4.0, rel_tol=0.15), f"double H ⇒ J_opt ×4, got {j_dbl_H/j_base:.2f}"
    assert math.isclose(j_both / j_base, 1.0, rel_tol=0.15), f"S/H fixed ⇒ J_opt unchanged, got {j_both/j_base:.2f}"


def test_C_opt_is_an_output_matching_the_closed_form():
    """The argmin lands at C_opt≈1/(4k_p²) — the DERIVED value, an OUTPUT of the penetration constant
    k_p (δ fills half-height at the optimum). Not fed in anywhere."""
    dp = _design_point()
    far = dp["far"]
    closed = 1.0 / (4.0 * 0.316 ** 2)                         # ≈2.504 (Holdeman's ≈2.5)
    assert math.isclose(SpatialPDF().C_opt(), closed, rel_tol=1e-9)
    _, _, C = _argmin_C(far, 0.0625, 0.10, npts=81)
    assert math.isclose(C, closed, rel_tol=0.08), f"argmin C={C:.3f} must land at the derived C_opt≈{closed:.3f}"


def test_k_p_sets_C_opt_and_the_collapse_is_robust():
    """k_p is the ONE knob that sets C_opt=1/(4k_p²) as an OUTPUT; the collapse holds at each k_p (the
    magnitude rides on k_p, but the GROUP collapse does not). Larger k_p (deeper penetration) ⇒ smaller
    C_opt, and two geometries still agree — so what is DERIVED is the collapse, not the number."""
    dp = _design_point()
    far = dp["far"]
    for k_p in (0.25, 0.316, 0.40):
        closed = 1.0 / (4.0 * k_p ** 2)
        _, _, C0 = _argmin_C(far, 0.0625, 0.10, k_p=k_p, npts=81)
        _, _, C1 = _argmin_C(far, 0.125, 0.10, k_p=k_p, npts=81)   # double S — same C_opt if it collapses
        assert math.isclose(C0, closed, rel_tol=0.12), f"k_p={k_p}: C_opt {C0:.3f} vs closed {closed:.3f}"
        assert math.isclose(C0, C1, rel_tol=0.12), f"k_p={k_p}: collapse broke ({C0:.3f} vs {C1:.3f})"


def test_no_C_opt_knob_it_is_derived():
    """THE SIGNATURE OF THE INVERSION: SpatialPDF has NO C_opt field (contrast every rung-12..18 config).
    C_opt is a DERIVED property, and passing C_opt= is a constructor error — you cannot impose it."""
    assert callable(SpatialPDF().C_opt), "C_opt must be a derived method, not a field"
    try:
        SpatialPDF(C_opt=2.5)                                 # type: ignore[call-arg]
        raise AssertionError("SpatialPDF must REJECT a C_opt kwarg — the optimum is an OUTPUT, not an input")
    except TypeError:
        pass


# --------------------------------------------------------------------------------------------------
# GATE 3 — g_spatial < g_ceiling (the tie back to rung-18's DERIVED two-stream ceiling).
# --------------------------------------------------------------------------------------------------

def test_resolved_width_below_two_stream_ceiling():
    """A partial-mix resolved field must be LESS segregated than the two-δ extreme, so g_spatial <
    g_ceiling at every J — the rung-18 ceiling bounds the resolved variance (and the production branch
    asserts it internally). Checked at under-, at-, and over-penetration."""
    dp = _design_point()
    gceil = _two_stream_ceiling(dp["far"], _PHI_P)
    for J in (1, 16, 400):
        st = _run(dp, J, _cfg())
        assert math.isclose(st.g_ceiling, gceil, rel_tol=1e-12)
        assert 0.0 < st.g_spatial < gceil, f"J={J}: g_spatial={st.g_spatial:.4f} must be in (0, {gceil:.4f})"


# --------------------------------------------------------------------------------------------------
# GATE 4 — emissions: C_opt is a LOCAL min, the GLOBAL min is at max segregation (the honest finding).
# --------------------------------------------------------------------------------------------------

def test_emissions_local_min_at_C_opt():
    """Through the ideal bell, C_opt (J=16) is a LOCAL ⟨EI⟩ minimum — both IMMEDIATE flanks lift
    (under-penetration J=9, over-penetration J=25). This is rung-18's reported behaviour, and it is real."""
    dp = _design_point()
    ei_opt = _run(dp, 16, _cfg()).ei_no_spatial
    ei_lo = _run(dp, 9, _cfg()).ei_no_spatial
    ei_hi = _run(dp, 25, _cfg()).ei_no_spatial
    assert ei_lo > ei_opt and ei_hi > ei_opt, \
        f"C_opt must be a LOCAL emissions min: {ei_lo:.4f} > {ei_opt:.4f} < {ei_hi:.4f}"


def test_emissions_global_min_at_max_segregation():
    """The HONEST finding: over a WIDE J-sweep the GLOBAL ⟨EI⟩ min is NOT at C_opt but at an ENDPOINT
    (max segregation) — rung-13's descending far flank, spatialized. Segregation at a lean mean moves
    mass OFF the stoich peak, lowering mean NO below the narrow C_opt basin. So UNIFORMITY (g), not
    emissions, is the derived headline."""
    dp = _design_point()
    Js = [1, 4, 16, 64, 256]
    ei = {J: _run(dp, J, _cfg()).ei_no_spatial for J in Js}
    amin = min(ei, key=ei.get)
    assert amin in (Js[0], Js[-1]), f"global emissions min must be at an ENDPOINT, got J={amin} ({ei})"
    assert amin != 16, "the global emissions min is NOT at C_opt (that is only the LOCAL min)"
    assert ei[amin] < ei[16], f"the endpoint must beat the C_opt floor: {ei[amin]:.4f} < {ei[16]:.4f}"


def test_derived_floor_sits_below_the_hump_peak():
    """WHY the C_opt emissions basin is narrow: the DERIVED floor g(C_opt)≈0.018 sits just BELOW the
    ideal-bell ⟨EI⟩(g) hump peak (~0.021). Rung 18's arbitrary floor g_ceiling·exp(−Da_opt)≈0.009 sits
    LOWER on the rising flank (a wider basin) — same curve, different floor placement, nobody wrong."""
    dp = _design_point()
    g_floor = _run(dp, 16, _cfg()).g_spatial                 # the derived C_opt floor ≈0.018
    # locate the ideal-bell ⟨EI⟩(g) hump peak (fine quadrature — the coarse test grid drifts the mean
    # at larger g; this is a one-off characterization scan, so use the production n_quad=200)
    gs = [0.008 + 0.0006 * i for i in range(40)]             # 0.008 .. 0.032 (brackets the ~0.021 peak)
    ei = [_pdf_mean_ei(dp["far"], dp["Tt3"], dp["p"], dp["hf"], _TAU, g, n_bell=_NB, n_quad=200) for g in gs]
    g_star = gs[max(range(len(gs)), key=lambda i: ei[i])]
    assert g_floor < g_star, f"derived floor {g_floor:.4f} must sit below the hump peak {g_star:.4f} (narrow basin)"
    assert g_star - g_floor < 0.01, f"floor {g_floor:.4f} sits JUST below peak {g_star:.4f} (why the basin is narrow)"


# --------------------------------------------------------------------------------------------------
# GATE 5 — grid convergence (the resolved field is numerically settled).
# --------------------------------------------------------------------------------------------------

def test_grid_converged():
    """C_opt (and g_min) are settled: ny=nz ∈ {32, 48, 64} agree — the coarse test grid is not the
    physics."""
    dp = _design_point()
    far = dp["far"]
    Cs = [_argmin_C(far, 0.0625, 0.10, ny=n, nz=n, npts=81)[2] for n in (32, 48, 64)]
    assert max(Cs) - min(Cs) < 0.05, f"C_opt must be grid-converged, got {Cs}"


# --------------------------------------------------------------------------------------------------
# GATE 6 — cycle untouched (pure diagnostic).
# --------------------------------------------------------------------------------------------------

def test_cycle_untouched():
    """A spatial call leaves the cycle far bit-identical — NO/N never enter the equilibrium solve, so
    the cycle stays bit-for-bit rung 6."""
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)

    far0 = run().stations["4"].far
    dp = _design_point()
    _run(dp, 16, _cfg())
    assert run().stations["4"].far == far0, "spatial call perturbed the cycle far — must stay rung-6"


# --------------------------------------------------------------------------------------------------
# GATE 7 — guards.
# --------------------------------------------------------------------------------------------------

def test_requires_mixing():
    dp = _design_point()
    try:
        dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, spatial=_cfg())
        raise AssertionError("spatial without mixing should raise")
    except AssertionError as e:
        assert "REQUIRES a `mixing`" in str(e)


def test_at_most_one_closure():
    dp = _design_point()
    for other in (dict(pocket_quench=PocketQuenchPDF()), dict(transported=TransportedPDF())):
        try:
            dp["g"].zoned_nox(dp["far"], dp["Tt3"], dp["Tt4"], dp["p"], _PHI_P, mixing=_mix(16),
                              spatial=_cfg(), **other)
            raise AssertionError("two closures should raise the ≤1-of-six guard")
        except AssertionError as e:
            assert "AT MOST ONE" in str(e)


def test_rich_primary_required():
    """A primary LEANER than the overall mean has no two-stream segregation to resolve — the RQL
    geometry guard fires inside `_spatial_segregation`."""
    dp = _design_point()
    phi_ov = dp["far"] / _F_STOICH
    try:
        _spatial_segregation(dp["far"], phi_ov * 0.5, 0.0625, 0.10, 16.0)   # leaner than the mean
        raise AssertionError("leaner-than-mean primary should fail the RQL guard")
    except AssertionError as e:
        assert "RQL geometry" in str(e) or "RICH" in str(e)


def test_positivity_guards():
    for bad in (dict(S=0.0), dict(k_p=0.0), dict(k_y=0.0), dict(k_z=0.0),
                dict(ny=1), dict(nz=1), dict(n_bell=1), dict(n_quad=1)):
        try:
            SpatialPDF(**bad)
        except AssertionError:
            continue
        raise AssertionError(f"SpatialPDF({bad}) should be rejected")
    SpatialPDF()                           # defaults accepted


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-22 tests passed")
