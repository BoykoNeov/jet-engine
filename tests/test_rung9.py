"""Rung-9 verification: rich primary / RQL — the rich-side of the NOx bell.

Rung 8 resolved a hot, near-stoichiometric primary and lifted EI_NO into the ICAO band,
but held the primary LEAN-to-stoich (φ_p ≤ 1) — it could only see the lean flank of the
NO-vs-φ bell. Rung 9 lets the primary run RICH (φ_p up to 2.0): the 8-species equilibrium
pool (CO/H2 already unknowns; reactions 1+2 span the water-gas shift) now carries MAJOR
CO/H2, set by a branched seed in `_equil_solve`. No new species, reactions, or datum — the
same extended-Zeldovich integrator on a rich pool. The payoff (RQL's whole reason to exist):
EI_NO forms a BELL that peaks near stoichiometric and FALLS steeply on the rich flank, so a
rich primary is a low-NOx regime. Mix-out is the IDEAL (infinitely-fast) quench — NO frozen
at the primary value; finite-rate quench is the next seam.

Gates (docs/rung9-spec.md), priority order:

1. reduce-to-rung-8 (LOAD-BEARING) — at φ_p ≤ 1 the rich branch is never taken; the lean
   `_equil_solve` seed is byte-identical, so the whole rung-1..8 suite is bit-for-bit and the
   rung-8 exact same-T_p identity still holds. Running zoned_nox (rich or lean) never touches
   the cycle far (bit-for-bit rung 6).
2. rich equilibrium is right — methane φ=1.05 AFT in the CEA band (~2231 K), the AFT peak sits
   slightly rich, CO/H2 are MAJOR and grow with φ, and the rich pool satisfies the water-gas-
   shift identity (a thermodynamic self-check on the branched solve).
3. the EI_NO BELL — peaks near stoich (φ_p ≈ 0.95–1.0) and falls steeply rich (EI(1.3) is
   <10% of the peak). THE rung-9 lesson: why RQL burns rich.
4. rich mix-out still returns to Tt4, split-independent across rich φ_p (re-equilibration gate,
   now releasing CO/H2 oxidation energy too).
5. soot-bound guard — φ_p ≤ 2.0 accepted, above rejected (the 5-species / no-C(s) basis).
6. the K-check + trace guard still bind at the (lower) rich primary T.

Run with `python tests/test_rung9.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, _Ru, _F_STOICH, _kcheck_ratio, _g_molar, _h_molar_A,
    _air_mole_fractions, _equil_solve, _equilibrium_composition, _thermal_no,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3


def _close(a, b, rel=1e-9, abs_=0.0):
    return abs(a - b) <= rel * abs(b) + abs_


def _design_point(eta_b=0.99):
    """Build the equilibrium engine and read the (derived) station-3/4 state. Same helper
    as test_rung8 — NO is trace, so the cycle is bit-for-bit rung 6."""
    losses = dict(_LOSSES, eta_b=eta_b)
    g = Gas.reacting_equilibrium()
    r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **losses).run(_FLIGHT, 50.0)
    st3, st4 = r.stations["3"], r.stations["4"]
    return g, st3.Tt, st4.Tt, st4.far, st4.pt


# --------------------------------------------------------------------------- #
# GATE 1 — reduce-to-rung-8: lean branch byte-identical; cycle untouched.      #
# --------------------------------------------------------------------------- #
def test_reduce_lean_branch_unchanged():
    # At φ_p ≤ 1 the primary far ≤ f_stoich, so bO ≥ 2bC+bH/2 (the full-oxidation O demand)
    # and `_equil_solve` takes the LEAN branch — byte-identical to rung 6/8. Confirm the
    # branch predicate holds for the whole lean range (the guarantee behind bit-for-bit).
    from turbojet.gas import _M_AIR, _M_CH2
    x = _air_mole_fractions()
    for phi in (0.4, 0.7, 0.9, 1.0):
        n_fuel = phi * _F_STOICH * _M_AIR / _M_CH2
        bC, bH, bO = n_fuel, 2.0 * n_fuel, 2.0 * x["O2"]
        assert bO >= 2.0 * bC + bH / 2.0 - 1e-15, f"φ={phi} should be lean-branch (bit-for-bit)"


def test_reduce_exact_same_Tp_still_holds():
    # The rung-8 exact reduce (α→1: zoned EI == thermal_nox at the same primary AFT T_p) must
    # survive rung 9 unchanged — the rich work must not perturb the lean path.
    for eta_b in (0.99, 1.0):
        g, Tt3, Tt4, far, p = _design_point(eta_b)
        z = g.zoned_nox(far, Tt3, Tt4, p, far / _F_STOICH, tau=_TAU)   # φ_p = φ_overall (α→1)
        direct = _thermal_no(_equilibrium_composition(far, z.T_primary, p),
                             z.T_primary, p, _TAU, far)
        assert _close(z.ei_no, direct.ei_no, rel=1e-9), \
            f"α→1 same-T_p identity broke: zoned {z.ei_no} vs direct {direct.ei_no}"


def test_cycle_untouched_by_rich_zoning():
    # Running zoned_nox at RICH φ_p is a pure diagnostic — the cycle far is bit-for-bit rung 6.
    g, Tt3, Tt4, far, p = _design_point()
    far_before = far
    for phi_p in (0.9, 1.2, 1.6, 2.0):
        g.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=_TAU)
    _, _, _, far_after, _ = _design_point()
    assert far_after == far_before, "rich zoning perturbed the cycle far (must stay rung-6)"


# --------------------------------------------------------------------------- #
# GATE 2 — rich equilibrium is correct: CEA methane anchor + WGS self-check.   #
# --------------------------------------------------------------------------- #
def _methane_aft(phi, p=101325.0):
    """Methane-air AFT with dissociation (scale A). CH4: bC=1, bH=4, stoich 2 O2."""
    x = _air_mole_fractions()
    HF_CH4 = -74600.0                     # J/mol (JANAF)
    nO2 = 2.0 / phi
    nN2, nAr = nO2 * x["N2"] / x["O2"], nO2 * x["Ar"] / x["O2"]
    H_react = HF_CH4
    lo, hi = 1000.0, 3200.0
    for _ in range(100):
        T = 0.5 * (lo + hi)
        comp = _equil_solve(1.0, 4.0, 2.0 * nO2, nN2 + nAr, T, p)
        H_prod = (sum(comp[s] * _h_molar_A(s, T) for s in comp)
                  + nN2 * _h_molar_A("N2", T) + nAr * _h_molar_A("Ar", T))
        lo, hi = (lo, T) if H_prod > H_react else (T, hi)
    return 0.5 * (lo + hi)


def test_methane_rich_aft_cea_anchor():
    # CEA equilibrium methane-air AFT: ~2224 K stoich, ~2231 K at φ=1.05, peak slightly rich
    # (Marzouk 2024, ETASR/arXiv 2503.11826). Ours is ~7 K high (NO/N + 5-species deferred,
    # same offset rung 6 noted). Anchor the rich point AND the rollover location.
    Tf_105 = _methane_aft(1.05)
    assert 2225.0 < Tf_105 < 2248.0, f"methane φ=1.05 AFT {Tf_105:.1f} out of CEA band"
    Ts = {phi: _methane_aft(phi) for phi in (0.95, 1.0, 1.05, 1.10, 1.30)}
    peak_phi = max(Ts, key=Ts.get)
    assert 1.0 <= peak_phi <= 1.08, f"AFT peak at φ={peak_phi} — should be slightly rich"
    assert Ts[1.30] < Ts[1.0], "rich flank must fall below stoich (AFT rollover)"


def test_rich_pool_is_CO_H2_major_and_wgs_consistent():
    # Rich equilibrium of the (CH2)n fuel: CO/H2 become MAJOR and grow with φ; and the pool
    # satisfies the water-gas shift CO + H2O ⇌ CO2 + H2 (Δν=0) with Kp from the same g0 the
    # solve uses — a thermodynamic self-check that the branched rich solve landed on the real
    # equilibrium (not just an atom-balanced point).
    p, T = 802664.8, 2200.0
    prev_co = -1.0
    for phi in (1.1, 1.4, 1.7):
        comp = _equilibrium_composition(phi * _F_STOICH, T, p)
        nt = sum(comp.values())
        assert comp["CO"] / nt > 0.02, f"φ={phi}: CO not major ({comp['CO']/nt:.4f})"
        assert comp["H2"] / nt > 0.005, f"φ={phi}: H2 not major ({comp['H2']/nt:.4f})"
        assert comp["CO"] > prev_co, "CO must grow with φ (richer)"
        prev_co = comp["CO"]
        # WGS: (n_CO2 n_H2)/(n_CO n_H2O) == exp(-ΔG0/RuT), ΔG0 = g_CO2+g_H2 - g_CO - g_H2O.
        dG = _g_molar("CO2", T) + _g_molar("H2", T) - _g_molar("CO", T) - _g_molar("H2O", T)
        import math
        Kp = math.exp(-dG / (_Ru * T))
        ratio = (comp["CO2"] * comp["H2"]) / (comp["CO"] * comp["H2O"])
        assert _close(ratio, Kp, rel=1e-6), f"φ={phi}: WGS off — {ratio:.4f} vs Kp {Kp:.4f}"


# --------------------------------------------------------------------------- #
# GATE 3 — the EI_NO bell: peaks near stoich, FALLS on the rich flank.         #
# --------------------------------------------------------------------------- #
def test_ei_no_bell_falls_on_rich_flank():
    g, Tt3, Tt4, far, p = _design_point()
    ei = {phi: g.zoned_nox(far, Tt3, Tt4, p, phi, tau=_TAU).ei_no
          for phi in (0.7, 0.9, 0.95, 1.0, 1.05, 1.1, 1.3, 1.5)}
    peak_phi = max(ei, key=ei.get)
    assert 0.9 <= peak_phi <= 1.05, f"EI_NO should peak near stoich, got φ={peak_phi}"
    # The rich flank collapses — a rich primary is low-NOx (RQL's reason to exist):
    assert ei[1.3] < 0.10 * ei[peak_phi], \
        f"rich flank must collapse: EI(1.3)={ei[1.3]:.3f} vs peak {ei[peak_phi]:.3f}"
    # monotone falling once past the peak:
    for a, b in ((1.05, 1.1), (1.1, 1.3), (1.3, 1.5)):
        assert ei[a] > ei[b], f"EI_NO must fall monotonically rich: EI({a}) !> EI({b})"
    # and the peak still lands in the ICAO band (single-digit-to-tens g/kg):
    assert 5.0 < ei[peak_phi] < 60.0, f"peak EI_NO {ei[peak_phi]:.2f} outside ICAO band"


# --------------------------------------------------------------------------- #
# GATE 4 — rich mix-out returns to Tt4, split-independent.                     #
# --------------------------------------------------------------------------- #
def test_rich_mixout_returns_to_Tt4_split_independent():
    g, Tt3, Tt4, far, p = _design_point()
    Tmix = [g.zoned_nox(far, Tt3, Tt4, p, phi, tau=_TAU).T_mix
            for phi in (0.9, 1.2, 1.5, 1.8, 2.0)]
    for Tm in Tmix:
        assert abs(Tm - Tmix[0]) < 1e-3, "T_mix must be split-independent across rich φ_p"
        assert abs(Tm - Tt4) < 0.02 * Tt4, f"rich mix-out {Tm:.1f} did not return to Tt4"


def test_rich_dilution_drops_fraction_not_index():
    # NO-mole conservation through the rich dilution: the mole FRACTION drops (dilution) but
    # EI (per kg fuel) is set in the primary — unchanged by mix-out.
    g, Tt3, Tt4, far, p = _design_point()
    z = g.zoned_nox(far, Tt3, Tt4, p, 1.1, tau=_TAU)
    assert z.ppm_mix < z.ppm_primary, "dilution must drop the NO mole fraction"
    assert z.ei_no == z.primary.ei_no, "EI is set in the primary (index ≠ concentration)"


# --------------------------------------------------------------------------- #
# GATE 5 — the soot-bound scope guard (φ_p ≤ 2.0).                             #
# --------------------------------------------------------------------------- #
def test_soot_bound_guard():
    g, Tt3, Tt4, far, p = _design_point()
    g.zoned_nox(far, Tt3, Tt4, p, 2.0, tau=_TAU)          # at the bound: accepted
    for bad in (2.2, 3.0):
        try:
            g.zoned_nox(far, Tt3, Tt4, p, bad, tau=_TAU)
        except AssertionError:
            continue
        raise AssertionError(f"φ_p={bad} > 2 should be rejected (soot / C(s) basis limit)")


# --------------------------------------------------------------------------- #
# GATE 6 — the K-check + trace guard bind at the (lower) rich primary T.       #
# --------------------------------------------------------------------------- #
def test_kcheck_and_trace_hold_at_rich_primary():
    g, Tt3, Tt4, far, p = _design_point()
    # A rich primary is COOLER (AFT rolls over) — down to ~1715 K at φ_p=2. _thermal_no asserts
    # both the K-check and the trace guard on every zoned call, so a passing rich sweep IS the
    # gate; also check the K-check constant directly across the rich primary band.
    for phi_p in (1.2, 1.6, 2.0):
        z = g.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=_TAU)
        r = _kcheck_ratio(z.T_primary)
        assert 0.90 < r < 1.15, f"K-check {r:.4f} at rich primary T={z.T_primary:.0f} out of band"
        assert z.primary.x_no_eq < 0.02, "NO must stay trace (decoupling) in the rich primary"


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-9 gates passed")
