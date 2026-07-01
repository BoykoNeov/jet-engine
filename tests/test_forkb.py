"""Rung-5 verification: Fork B — formation-enthalpy bookkeeping (derived heat release).

Five gates (docs/rung5-fork-b.md § Verification gates), priority order:

1. reduce-to-ideal + reduce-to-rung-4 (LOAD-BEARING) — Fork B is a separate factory
   (Gas.reacting_forkb), so the CPG/frozen/reacting-Fork-A paths are untouched and the
   existing suites stay green. The exact-equivalence THEOREM: a reacting-Fork-B cycle
   reproduces the reacting-Fork-A cycle's f/Tt5/thrust/TSFC to machine precision,
   because the released chemical energy is IDENTICALLY f*LHV for complete combustion.
2. derived LHV = Mattingly hPR — hf_fuel = -34.99 kJ/mol => LHV = 42.8000 MJ/kg.
3. formation self-check — H(298.15) = dHf per species; elements land at h=0 at 298.15 K.
4. absolute-balance closure — the burner's Sigma N h(react) = Sigma N h(prod) + loss
   assert fires on every Fork-B run (checked here explicitly at the converged f), and
   the fuel enthalpy is a LIVE knob (a lower LHV needs more fuel: f rises).
5. AFT physical plausibility (test-only, no book digit) — no-dissociation flame temps
   are monotone in f and stoich ~ 2375 K; deliberately HIGH vs real ~2250 K because we
   do not yet model dissociation (that is rung 6). See docs/plans/rung5-anchor-formation.

Run with `python tests/test_forkb.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, _HF298, _HF_FUEL_DEFAULT, _M_CH2_KG, _M_AIR, _Ru, _SPECIES, _T_REF,
    _F_STOICH, _air_mole_fractions, _antideriv_h, _lhv_from_fuel,
    _products_composition,
)

# A real (lossy, supersonic) design point so the gates exercise the whole cycle.
_FLIGHT = FlightCondition(T0=216.7, p0=18_750.0, M0=2.0)
_DESIGN = dict(pi_c=10.0, Tt4=1800.0, p_ambient=18_750.0,
               pi_d=0.95, eta_c=0.90, eta_b=0.98, pi_b=0.95, eta_t=0.90,
               eta_m=0.99, pi_n=0.97)


def _close(a, b, rel=1e-9, abs_=0.0):
    return abs(a - b) <= rel * abs(b) + abs_


# --------------------------------------------------------------------------- #
# GATE 1 — reduce-to-rung-4: Fork B == reacting Fork A, EXACTLY (the theorem). #
# --------------------------------------------------------------------------- #
def test_forkb_equals_forka_exactly():
    gA = Gas.reacting(hPR=42.8e6)          # rung-4 Fork A, assumed hPR
    gB = Gas.reacting_forkb()              # rung-5 Fork B, DERIVED LHV (defaults to 42.8)
    rA = build_turbojet(gA, **_DESIGN).run(_FLIGHT, mdot=50.0)
    rB = build_turbojet(gB, **_DESIGN).run(_FLIGHT, mdot=50.0)
    # Machine-precision agreement on the load-bearing outputs.
    assert _close(rB.stations["4"].far, rA.stations["4"].far, rel=1e-12, abs_=1e-15)
    assert _close(rB.stations["5"].Tt, rA.stations["5"].Tt, rel=1e-12)
    assert _close(rB.performance.specific_thrust, rA.performance.specific_thrust, rel=1e-12)
    assert _close(rB.performance.tsfc, rA.performance.tsfc, rel=1e-12)


# --------------------------------------------------------------------------- #
# GATE 2 — the derived LHV reproduces Mattingly's assumed hPR = 42.8 MJ/kg.   #
# --------------------------------------------------------------------------- #
def test_derived_lhv_matches_mattingly_hpr():
    g = Gas.reacting_forkb()
    assert _close(g.lhv, 42.8e6, rel=1e-6), f"derived LHV {g.lhv} != 42.8 MJ/kg"
    assert _close(g.hPR, g.lhv, rel=1e-12), "Fork B hPR slot must hold the derived LHV"
    # The pinned fuel enthalpy is ~ -35 kJ/mol (the advisor's prediction).
    assert -36_000.0 < _HF_FUEL_DEFAULT < -34_000.0, _HF_FUEL_DEFAULT


# --------------------------------------------------------------------------- #
# GATE 3 — formation self-check: H(298.15) = dHf; elements h=0 at 298.15 K.   #
# --------------------------------------------------------------------------- #
def test_formation_self_check():
    for s, hf in _HF298.items():
        A_low = _SPECIES[s][1]
        a6 = hf / _Ru - _antideriv_h(A_low, _T_REF)                # derived formation const
        H_abs = _Ru * (_antideriv_h(A_low, _T_REF) + a6)           # absolute molar enthalpy
        assert _close(H_abs, hf, rel=1e-9, abs_=1e-6), f"{s}: H(298.15)={H_abs} != {hf}"
    # Elements sit at zero (the absolute datum), CO2/H2O carry their negative formation.
    assert _HF298["N2"] == _HF298["O2"] == _HF298["Ar"] == 0.0
    assert _HF298["CO2"] < 0.0 and _HF298["H2O"] < 0.0


# --------------------------------------------------------------------------- #
# GATE 4 — absolute-balance closure + the fuel enthalpy is a LIVE knob.       #
# --------------------------------------------------------------------------- #
def test_absolute_balance_closes_and_fuel_is_live_knob():
    # NB on what this assert can and cannot catch: the exact-equivalence theorem makes
    # the absolute balance ALGEBRAICALLY equal to the solver's Fork-A form (with hPR:=LHV),
    # so it can never expose a SOLVER error. It guards only the absolute-interface plumbing
    # (h_t_abs, hf_fuel_mass, _formation_products_mass) against sign/constant slips. That is
    # its job; it is not an independent energy check on the converged f. See the production
    # scale note (0K-sensible + formation) in docs/rung5-fork-b.md.
    g = Gas.reacting_forkb()
    # Re-close Sigma N h(react) = Sigma N h(prod) + loss at a hand set of states,
    # mirroring the burner's standing assert (eta_b=1 => no loss term).
    Tt3, Tt4 = 800.0, 1600.0
    f = 0.0
    for _ in range(100):                                            # rung-4 contraction
        h4 = g.h_t(Tt4, f)
        f = (h4 - g.h_c(Tt3)) / (g.hPR - h4)
    react = g.h_c_abs(Tt3) + f * g.hf_fuel_mass                     # per kg air
    prod = (1.0 + f) * g.h_t_abs(Tt4, f)
    # Normalize by the SENSIBLE product enthalpy (the burner's own tolerance basis):
    # the absolute enthalpies are small (formation cancels most of the sensible part),
    # so the identity's ~1e-6 rounding lands relative to the ~1.9 MJ sensible scale.
    scale = (1.0 + f) * g.h_t(Tt4, f)
    assert abs(react - prod) <= 1e-6 * scale, f"absolute balance open: {react} vs {prod}"

    # A LOWER-LHV fuel (more negative formation enthalpy) needs MORE fuel for the same
    # Tt4 => f rises. This proves the derived heat release actually drives the burner.
    g_lean = Gas.reacting_forkb(hf_fuel_molar=-50_000.0)            # LHV ~ 41.7 MJ/kg
    assert g_lean.lhv < g.lhv
    rHi = build_turbojet(g, **_DESIGN).run(_FLIGHT, mdot=50.0)
    rLo = build_turbojet(g_lean, **_DESIGN).run(_FLIGHT, mdot=50.0)
    assert rLo.stations["4"].far > rHi.stations["4"].far, "lower LHV must need more fuel"


# --------------------------------------------------------------------------- #
# GATE 5 — adiabatic flame temperature: physical plausibility (TEST-ONLY).    #
# The engine takes Tt4 as an input; AFT is computed here only as the physical  #
# sanity anchor + the rung-6 motivation. Uses absolute enthalpies across the   #
# 1000 K join (the burner never needs the join, so this lives in the test).    #
# --------------------------------------------------------------------------- #
def _h_molar_abs(species, T):
    A_low, A_high = _SPECIES[species][1], _SPECIES[species][2]
    a6 = _HF298[species] / _Ru - _antideriv_h(A_low, _T_REF)
    if T <= 1000.0:
        sens = _antideriv_h(A_low, T)
    else:
        sens = (_antideriv_h(A_low, 1000.0)
                + _antideriv_h(A_high, T) - _antideriv_h(A_high, 1000.0))
    return _Ru * (sens + a6)


def _flame_temp(f):
    comp = _products_composition(f)
    x = _air_mole_fractions()
    n_fuel = f * _M_AIR / (_M_CH2_KG * 1000.0)
    H_react = (sum(x[s] * _h_molar_abs(s, _T_REF) for s in x)
               + n_fuel * _HF_FUEL_DEFAULT)
    lo, hi = 300.0, 4000.0
    for _ in range(200):
        T = 0.5 * (lo + hi)
        H_prod = sum(comp[s] * _h_molar_abs(s, T) for s in comp)
        if H_prod > H_react:
            hi = T
        else:
            lo = T
    return 0.5 * (lo + hi)


def test_adiabatic_flame_temp_plausible():
    temps = [_flame_temp(f) for f in (0.020, 0.030, 0.050, _F_STOICH * 0.999)]
    # Monotone increasing with f.
    assert all(b > a for a, b in zip(temps, temps[1:])), temps
    # Stoichiometric no-dissociation value in the right (deliberately high) band.
    t_stoich = _flame_temp(_F_STOICH * 0.999)
    assert 2300.0 < t_stoich < 2450.0, f"stoich AFT {t_stoich} out of band"
    # It is HIGHER than the real dissociation-capped ~2250 K — that gap is rung 6.
    assert t_stoich > 2250.0


def _run_all():
    """Dependency-free runner so `python tests/test_forkb.py` works."""
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
