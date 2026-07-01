"""Rung-8 verification: combustor zoning — the primary-zone NOx effect.

Gates (docs/rung8-spec.md § Verification gates), priority order:

1. reduce-to-rung-7 (LOAD-BEARING) — at α→1 (all air in the primary) the two-zone
   diagnostic collapses to rung-7's single mixed-out pool. TWO parts:
     (a) EXACT: zoned EI == thermal_nox(far, T_p, p) at the SAME primary AFT T_p (machine
         precision) — confirms far_p/α/the freeze scaling and that T_mix collapses to T_p.
     (b) PHYSICAL: T_p ≈ Tt4 and zoned EI is within a small factor of the rung-7 mixed-out
         thermal_nox(far, Tt4, p). The residual is a ~8 K scale-A/scale-B DATUM offset
         (`_h_molar_A` formation vs `_h_molar_B` 0K-sensible+HF298 — it does NOT cancel
         across combustion because moles change, and it SURVIVES η_b=1) PLUS a ~9 K η_b
         piece (more fuel → hotter true AFT). Both are dwarfed by NO's exp-in-T sensitivity
         at ~1500 K, so the EI ratio is O(1) not 1e-6 — see docs/plans/rung8-anchor-zoning
         § 3 (corrected). The cycle is bit-for-bit rung 6: zoning is a pure diagnostic and
         running it never touches the cycle far.
2. EI_NO lands in the ICAO band — primary φ_p = 0.9–1.0 gives single-digit-to-tens g/kg
   (order-of-magnitude landing zone), ~6 orders above the mixed-out ~zero.
3. mix-out T is split-independent and returns to Tt4 (the re-equilibration gate). A
   frozen-majors mix-out traps the dissociation energy and misses Tt4 (discriminating check).
4. NO-mole conservation through dilution — the mole FRACTION falls but EI (per kg fuel) is
   set in the primary and unchanged (concentration ≠ emission index).
5. T-sensitivity — EI_NO rises monotonically and >10× over φ_p 0.7 → 1.0.
6. φ_p ≤ 1 scope guard; the K-check binds at the hotter primary T (asserted every call).

Run with `python tests/test_rung8.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, _F_STOICH, _kcheck_ratio, _h_molar_A, _h_air_molar_A,
    _air_mole_fractions, _equilibrium_composition, _M_AIR, _M_CH2, _M_CH2_KG, _M_NO,
)

# Design point = main.py's (subsonic cruise) — the one the anchor's worked example uses;
# its Tt3 ≈ 583 K makes the near-stoich primary land in the ICAO band. Derived from a REAL
# equilibrium-engine run (never hardcoded): NO is trace, so the cycle is bit-for-bit rung 6.
_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3


def _close(a, b, rel=1e-9, abs_=0.0):
    return abs(a - b) <= rel * abs(b) + abs_


def _design_point(eta_b=0.99):
    """Build the equilibrium engine and read the (derived) station-3/4 state."""
    losses = dict(_LOSSES, eta_b=eta_b)
    g = Gas.reacting_equilibrium()
    r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **losses).run(_FLIGHT, 50.0)
    st3, st4 = r.stations["3"], r.stations["4"]
    return g, st3.Tt, st4.Tt, st4.far, st4.pt


# --------------------------------------------------------------------------- #
# GATE 1 — reduce-to-rung-7: exact (same T_p) + physical (≈ Tt4, O(1) factor). #
# --------------------------------------------------------------------------- #
def test_reduce_exact_same_Tp():
    # (a) At α→1 the primary far == the overall far; zoned NO is computed on that pool at
    # the primary AFT T_p. rung-7 thermal_nox(far, T_p, p) is the IDENTICAL computation, so
    # the two EI's match to machine precision — regardless of η_b. This certifies far_p, α,
    # and the mole-freeze scaling (not a tautology: a bug in any of them breaks it).
    for eta_b in (0.99, 1.0):
        g, Tt3, Tt4, far, p = _design_point(eta_b)
        z = g.zoned_nox(far, Tt3, Tt4, p, far / _F_STOICH, tau=_TAU)   # α = 1
        assert _close(z.alpha, 1.0, rel=1e-9), f"α should be 1 at φ_p=φ_overall, got {z.alpha}"
        ref = g.thermal_nox(far, z.T_primary, p, tau=_TAU)
        assert _close(z.ei_no, ref.ei_no, rel=1e-9), \
            f"exact reduce broke: zoned {z.ei_no} != rung-7@T_p {ref.ei_no} (η_b={eta_b})"


def test_reduce_physical_to_mixed_out():
    # (b) The PHYSICAL reduce: at α→1 the primary really is ~ the cycle's station-4 state.
    # T_p sits a few K above Tt4 (datum ~8 K + η_b ~9 K), and the mixed-out rung-7 EI is
    # within an O(1) factor — this is reduce-to-rung-7, not reduce-to-itself.
    g, Tt3, Tt4, far, p = _design_point(eta_b=1.0)
    z = g.zoned_nox(far, Tt3, Tt4, p, far / _F_STOICH, tau=_TAU)
    assert 0.0 < z.T_primary - Tt4 < 20.0, f"T_p {z.T_primary:.1f} not just above Tt4 {Tt4}"
    ei_mixed = g.thermal_nox(far, Tt4, p, tau=_TAU).ei_no
    ratio = z.ei_no / ei_mixed
    assert 0.5 < ratio < 3.0, f"α→1 EI ratio {ratio:.2f} vs mixed-out out of O(1) band"


def test_cycle_untouched_by_zoning():
    # The cycle is bit-for-bit rung 6: zoned_nox is a pure diagnostic — the equilibrium pool
    # never carries NO/N, and calling it does not perturb the station-4 far.
    g, Tt3, Tt4, far, p = _design_point()
    comp = _equilibrium_composition(far, Tt4, p)
    assert "NO" not in comp and "N" not in comp, "NO/N leaked into the equilibrium pool"
    g.zoned_nox(far, Tt3, Tt4, p, 0.9, tau=_TAU)
    far2 = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(
        _FLIGHT, 50.0).stations["4"].far
    assert _close(far, far2, rel=1e-12), "running zoned_nox perturbed the cycle far"


# --------------------------------------------------------------------------- #
# GATE 2 — EI_NO climbs into the ICAO band; mixed-out is ~6 orders lower.      #
# --------------------------------------------------------------------------- #
def test_ei_no_in_icao_band():
    g, Tt3, Tt4, far, p = _design_point()
    for phi_p in (0.9, 1.0):
        ei = g.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=_TAU).ei_no
        assert 5.0 < ei < 80.0, f"φ_p={phi_p}: EI_NO {ei:.2f} g/kg outside single-digit-to-tens"
    # The mixed-out station-4 number is ~zero, and the primary lifts it ~6 orders of magnitude.
    ei_primary = g.zoned_nox(far, Tt3, Tt4, p, 1.0, tau=_TAU).ei_no
    ei_mixed = g.thermal_nox(far, Tt4, p, tau=_TAU).ei_no
    assert ei_mixed < 1e-3, f"mixed-out EI_NO {ei_mixed:.2e} not ~zero"
    assert ei_primary / ei_mixed > 1e4, f"primary lift only {ei_primary/ei_mixed:.1e}× (< 1e4)"


# --------------------------------------------------------------------------- #
# GATE 3 — mix-out T split-independent, returns to Tt4; frozen-majors misses.  #
# --------------------------------------------------------------------------- #
def test_mixout_split_independent_returns_to_Tt4():
    g, Tt3, Tt4, far, p = _design_point()
    Tmix = [g.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=_TAU).T_mix
            for phi_p in (0.7, 0.8, 0.9, 1.0)]
    # α cancels analytically in the enthalpy balance -> T_mix is the SAME for every split,
    # to the bisection tolerance (a wrong basis / frozen composition would break this).
    for t in Tmix[1:]:
        assert _close(t, Tmix[0], rel=0.0, abs_=1e-3), f"T_mix not split-independent: {Tmix}"
    # And it returns to ≈ Tt4 (within the ~8 K datum + η_b gap): the re-equilibration gate.
    assert 0.0 < Tmix[0] - Tt4 < 30.0, f"T_mix {Tmix[0]:.1f} did not return to Tt4 {Tt4}"


def test_frozen_majors_mixout_misses_Tt4():
    # DISCRIMINATING check (anchor § 4): re-equilibrating the majors on mix-out releases the
    # stored dissociation energy so T_mix returns to Tt4. FREEZING the dissociated primary
    # composition (no recombination) traps that energy and lands at a DIFFERENT temperature,
    # missing Tt4 by ≫ the split-independence tolerance. This proves the re-equilibration is
    # real, not cosmetic.
    g, Tt3, Tt4, far, p = _design_point()
    phi_p = 1.0
    far_p = phi_p * _F_STOICH
    alpha = far / far_p
    z = g.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=_TAU)
    comp_p = _equilibrium_composition(far_p, z.T_primary, p)
    H_mix = (alpha * sum(comp_p[s] * _h_molar_A(s, z.T_primary) for s in comp_p)
             + (1.0 - alpha) * _h_air_molar_A(Tt3))
    # Frozen composite: primary dissociated products (× α) + dilution air, NO recombination.
    frozen = {}
    for s in comp_p:
        frozen[s] = frozen.get(s, 0.0) + alpha * comp_p[s]
    for s, x in _air_mole_fractions().items():
        frozen[s] = frozen.get(s, 0.0) + (1.0 - alpha) * x
    lo, hi = 500.0, 3200.0
    for _ in range(100):
        T = 0.5 * (lo + hi)
        Hp = sum(n * _h_molar_A(s, T) for s, n in frozen.items())
        lo, hi = (lo, T) if Hp > H_mix else (T, hi)
    T_frozen = 0.5 * (lo + hi)
    # Freezing traps the recombination energy in dissociated bonds -> substantially COOLER
    # than the re-equilibrated mix-out (which releases it into sensible heat).
    assert T_frozen < z.T_mix - 30.0, \
        f"frozen mix-out {T_frozen:.1f} not substantially cooler than re-eq {z.T_mix:.1f}"
    # And the re-equilibrated mix-out returns MUCH closer to Tt4 than the frozen one does —
    # only re-equilibration recovers the station-4 the cycle computed.
    assert abs(z.T_mix - Tt4) < abs(T_frozen - Tt4), \
        f"re-eq ({z.T_mix:.1f}) should be closer to Tt4 {Tt4} than frozen ({T_frozen:.1f})"


# --------------------------------------------------------------------------- #
# GATE 4 — NO-mole conservation through dilution (index ≠ concentration).      #
# --------------------------------------------------------------------------- #
def test_no_mole_conservation():
    g, Tt3, Tt4, far, p = _design_point()
    z = g.zoned_nox(far, Tt3, Tt4, p, 0.9, tau=_TAU)
    # Dilution drops the NO mole FRACTION (primary ppm -> mixed ppm)...
    assert z.ppm_mix < z.ppm_primary, f"dilution should lower NO fraction: {z.ppm_mix} vs {z.ppm_primary}"
    # ...but conserves NO MOLES, so EI (per kg fuel) computed from the DILUTED state equals
    # the primary EI. NO moles per mol total air = x_no_mix * ntot_mix; fuel mass per mol air
    # = far*(M_AIR/M_CH2)*M_CH2_KG. This is the clean concentration-vs-index separation.
    ntot_mix = sum(_equilibrium_composition(far, z.T_mix, p).values())
    n_no_total = z.x_no_mix * ntot_mix
    fuel_mass = far * _M_AIR / _M_CH2 * _M_CH2_KG
    ei_from_diluted = 1000.0 * (n_no_total * _M_NO) / fuel_mass
    assert _close(ei_from_diluted, z.ei_no, rel=1e-9), \
        f"EI not conserved through dilution: {ei_from_diluted} vs {z.ei_no}"


# --------------------------------------------------------------------------- #
# GATE 5 — T-sensitivity: EI_NO rises steeply and monotonically with φ_p.      #
# --------------------------------------------------------------------------- #
def test_temperature_sensitivity():
    g, Tt3, Tt4, far, p = _design_point()
    zs = [g.zoned_nox(far, Tt3, Tt4, p, phi_p, tau=_TAU) for phi_p in (0.7, 0.8, 0.9, 1.0)]
    afts = [z.T_primary for z in zs]
    eis = [z.ei_no for z in zs]
    assert all(b > a for a, b in zip(afts, afts[1:])), f"primary AFT not monotone in φ_p: {afts}"
    assert all(b > a for a, b in zip(eis, eis[1:])), f"EI_NO not monotone in φ_p: {eis}"
    assert eis[-1] / eis[0] > 10.0, f"EI_NO φ_p 0.7->1.0 rise {eis[-1]/eis[0]:.1f}× too weak (< 10×)"


# --------------------------------------------------------------------------- #
# GATE 6 — φ_p scope guard; the K-check binds at the primary T.                #
# NOTE: rung 8 held φ_p ≤ 1 (lean-stoich); rung 9 widened the guard to φ_p ≤ 2 #
# (rich RQL primary, below soot onset). So 1.2/1.5 are now VALID; the guard now #
# rejects only above the soot bound. Rich payoff gates live in test_rung9.py.  #
# --------------------------------------------------------------------------- #
def test_phi_primary_guard():
    g, Tt3, Tt4, far, p = _design_point()
    for bad in (2.5, 3.0):
        try:
            g.zoned_nox(far, Tt3, Tt4, p, bad, tau=_TAU)
        except AssertionError:
            continue
        raise AssertionError(f"φ_p={bad} > 2 should have been rejected (soot-bound scope)")


def test_kcheck_binds_at_primary_T():
    # The primary AFT (~2400 K) is inside the rung-7 K-check band; the thermo-kinetic ratio
    # (rate constants vs the a6/a7 thermo) still binds there. _thermal_no asserts it on every
    # zoned call; check the constant directly at a representative primary temperature.
    for T in (2200.0, 2350.0, 2450.0):
        r = _kcheck_ratio(T)
        assert 0.90 < r < 1.15, f"K-check ratio {r:.4f} at primary T={T} out of band"


def _run_all():
    """Dependency-free runner so `python tests/test_rung8.py` works."""
    failures = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            try:
                fn()
                print(f"PASS {name}")
            except Exception as e:  # noqa: BLE001 — harness reporting only
                failures += 1
                print(f"FAIL {name}: {type(e).__name__}: {e}")
    return failures


if __name__ == "__main__":
    sys.exit(1 if _run_all() else 0)
