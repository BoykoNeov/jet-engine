"""Rung-6 verification: high-temperature dissociation + the chemical-equilibrium solve.

Gates (docs/rung6-spec.md § Verification gates), priority order:

1. reduce-to-lower-rung (LOAD-BEARING) — Gas.reacting_equilibrium is a separate factory,
   so rungs 1-5 stay green untouched (run the rest of the suite). Here: the ANTI-SEAM /
   reduce-to-rung-5 cold-Tt4 limit — rung-6 cycle f == rung-5 Fork-B f to ~1e-6 as Tt4
   drops (dissociation -> 0). A CONSTANT ~1% offset would betray scale-A enthalpy leaking
   into the scale-B energy balance; instead the delta shrinks with dissociation.
2. the Kp/equilibrium physics anchor — methane-air stoich equilibrium AFT in the CEA band
   (2226 K; ours 2231.7); and the (p/p0)^dnu factor live (pressure suppresses dissociation).
3. formation + entropy self-checks — h(298.15)=dHf, s(298.15)=S298 per species.
4. equilibrium-AFT drop (TEST-ONLY, scale A) — (CH2)n stoich AFT drops into the real
   ~2250 K band, below rung-5's no-dissociation value, monotone in f.
5. station-4 delta bounded + the whole cycle runs (asserts pass); the burn-config guard.

Run with `python tests/test_rung6.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, _HF298, _S298, _HF_FUEL_DEFAULT, _M_AIR, _M_CH2, _F_STOICH, _Ru,
    _air_mole_fractions, _equil_solve, _equilibrium_composition, _h_molar_A,
    _s_molar, _sens_h, _sens_phi, _a6_of, _a7_of, _T_REF,
)

# A real (lossy, supersonic) design point (same as test_forkb) so the gates exercise
# the whole cycle at station-4 pressure (~13 atm) where dissociation is doubly suppressed.
_FLIGHT = FlightCondition(T0=216.7, p0=18_750.0, M0=2.0)
_DESIGN = dict(pi_c=10.0, Tt4=1800.0, p_ambient=18_750.0,
               pi_d=0.95, eta_c=0.90, eta_b=0.98, pi_b=0.95, eta_t=0.90,
               eta_m=0.99, pi_n=0.97)
_P1 = 101325.0     # 1 atm for the CEA anchor / AFT diagnostics


def _close(a, b, rel=1e-9, abs_=0.0):
    return abs(a - b) <= rel * abs(b) + abs_


# --- AFT helper (TEST-ONLY, SCALE A: the physically-correct formation datum that
#     matches CEA — the same two-context split rung 5 used, docs/rung6-anchor § 2b) ---
def _aft_equilibrium(bC, bH, nO2, hf_fuel, p, n_fuel=1.0):
    """Constant-p adiabatic flame temp with dissociation, reactants at 298.15 K.
    Air enthalpy at 298.15 is 0 on scale A (elements), so H_react = n_fuel*hf_fuel."""
    x = _air_mole_fractions()
    nN2, nAr = nO2 * x["N2"] / x["O2"], nO2 * x["Ar"] / x["O2"]
    H_react = n_fuel * hf_fuel
    lo, hi = 1000.0, 3200.0
    for _ in range(100):
        T = 0.5 * (lo + hi)
        comp = _equil_solve(bC, bH, 2.0 * nO2, nN2 + nAr, T, p)
        H_prod = (sum(comp[s] * _h_molar_A(s, T) for s in comp)
                  + nN2 * _h_molar_A("N2", T) + nAr * _h_molar_A("Ar", T))
        lo, hi = (lo, T) if H_prod > H_react else (T, hi)
    return 0.5 * (lo + hi)


def _aft_ch2(f, p, dissociate=True):
    """(CH2)n flame temp per mol air. dissociate=False -> complete combustion (rung-5)."""
    x = _air_mole_fractions()
    n_fuel = f * _M_AIR / _M_CH2
    H_react = n_fuel * _HF_FUEL_DEFAULT
    lo, hi = 800.0, 3200.0
    for _ in range(100):
        T = 0.5 * (lo + hi)
        if dissociate:
            comp = _equilibrium_composition(f, T, p)
        else:
            comp = {"CO2": n_fuel, "H2O": n_fuel, "O2": x["O2"] - 1.5 * n_fuel,
                    "N2": x["N2"], "Ar": x["Ar"]}
        H_prod = sum(comp[s] * _h_molar_A(s, T) for s in comp)
        lo, hi = (lo, T) if H_prod > H_react else (T, hi)
    return 0.5 * (lo + hi)


# --------------------------------------------------------------------------- #
# GATE 1 — anti-seam / reduce-to-rung-5 in the cold-Tt4 limit.                 #
# --------------------------------------------------------------------------- #
def test_reduce_to_rung5_cold_limit():
    D = dict(_DESIGN, Tt4=1000.0)                       # cold -> dissociation ~ 0
    fB = build_turbojet(Gas.reacting_forkb(), **D).run(_FLIGHT, 50.0).stations["4"].far
    fE = build_turbojet(Gas.reacting_equilibrium(), **D).run(_FLIGHT, 50.0).stations["4"].far
    assert _close(fE, fB, rel=1e-6), f"cold-limit seam: equil {fE} vs Fork B {fB}"
    # And the delta SHRINKS with Tt4 (a scale leak would be a constant ~1%): 1400 K > 1000 K.
    D2 = dict(_DESIGN, Tt4=1400.0)
    fB2 = build_turbojet(Gas.reacting_forkb(), **D2).run(_FLIGHT, 50.0).stations["4"].far
    fE2 = build_turbojet(Gas.reacting_equilibrium(), **D2).run(_FLIGHT, 50.0).stations["4"].far
    assert abs(fE2 - fB2) / fB2 > abs(fE - fB) / fB, "seam delta must grow with Tt4, not be constant"


# --------------------------------------------------------------------------- #
# GATE 2 — methane-air stoich equilibrium AFT vs CEA; pressure suppression.    #
# --------------------------------------------------------------------------- #
def test_methane_aft_equilibrium_anchor():
    # CH4 + 2 O2 -> ... (C=1, H=4, nO2=2). CEA/Turns ~2226 K; ours ~2231.7 (NO/N deferred).
    Tf = _aft_equilibrium(bC=1.0, bH=4.0, nO2=2.0, hf_fuel=-74600.0, p=_P1)
    assert 2210.0 < Tf < 2245.0, f"CH4-air equilibrium AFT {Tf} out of CEA band"


def test_pressure_suppresses_dissociation():
    # Stoich (CH2)n at fixed T=2300 K: CO/(CO+CO2) must FALL as pressure rises ((p/p0)^dnu).
    f = _F_STOICH * 0.999
    fracs = []
    for p_atm in (1.0, 5.0, 13.0):
        comp = _equilibrium_composition(f, 2300.0, p_atm * _P1)
        fracs.append(comp["CO"] / (comp["CO"] + comp["CO2"]))
    assert fracs[0] > fracs[1] > fracs[2], f"dissociation must fall with pressure: {fracs}"
    assert fracs[0] > 0.05, f"1 atm stoich should show real CO dissociation, got {fracs[0]}"


# --------------------------------------------------------------------------- #
# GATE 3 — formation + entropy self-checks (a6 from dHf, a7 from S298).        #
# --------------------------------------------------------------------------- #
def test_formation_and_entropy_self_check():
    for s in _HF298:
        # h(298.15) on scale A = dHf; s(298.15) = S298 (both exact by construction).
        h_abs = _Ru * (_sens_h(s, _T_REF) + _a6_of(s))
        s_abs = _s_molar(s, _T_REF)
        assert _close(h_abs, _HF298[s], rel=1e-9, abs_=1e-6), f"{s}: h(298)={h_abs}!={_HF298[s]}"
        assert _close(s_abs, _S298[s], rel=1e-9, abs_=1e-6), f"{s}: s(298)={s_abs}!={_S298[s]}"
    # The five dissociation species carry the expected formation signs; H2 is an element.
    assert _HF298["CO"] < 0 and _HF298["H2"] == 0.0
    assert _HF298["OH"] > 0 and _HF298["O"] > 0 and _HF298["H"] > 0


# --------------------------------------------------------------------------- #
# GATE 4 — equilibrium-AFT drop: (CH2)n stoich into the real ~2250 K band.     #
# --------------------------------------------------------------------------- #
def test_equilibrium_aft_drop():
    fs = (0.020, 0.030, 0.050, _F_STOICH * 0.999)
    equil = [_aft_ch2(f, _P1, dissociate=True) for f in fs]
    frozen = [_aft_ch2(f, _P1, dissociate=False) for f in fs]
    # Monotone in f.
    assert all(b > a for a, b in zip(equil, equil[1:])), equil
    # Stoich drops into the real band, strictly below rung-5's no-dissociation value.
    assert 2250.0 < equil[-1] < 2275.0, f"stoich equilibrium AFT {equil[-1]} out of band"
    assert equil[-1] < frozen[-1] - 80.0, f"dissociation must LOWER stoich AFT: {equil[-1]} vs {frozen[-1]}"


# --------------------------------------------------------------------------- #
# GATE 5 — station-4 delta bounded, the cycle runs, and the burn-config guard. #
# --------------------------------------------------------------------------- #
def test_station4_delta_bounded_and_cycle_runs():
    rB = build_turbojet(Gas.reacting_forkb(), **_DESIGN).run(_FLIGHT, 50.0)
    rE = build_turbojet(Gas.reacting_equilibrium(), **_DESIGN).run(_FLIGHT, 50.0)
    fB, fE = rB.stations["4"].far, rE.stations["4"].far
    # Bounded (not the digit — Tt3 from the compressor + eta_b=0.98 drift it); equilibrium
    # needs slightly MORE fuel (dissociated products retain chemical enthalpy).
    assert 0.0 < (fE - fB) / fB < 0.005, f"station-4 delta {100*(fE-fB)/fB:.3f}% out of bound"
    # The frozen station-4 mixture actually dissociated a little (trace CO/OH present).
    c = _equilibrium_composition(fE, 1800.0, rE.stations["4"].pt)
    assert c["CO"] > 0.0 and c["OH"] > 0.0, "expected trace dissociation products at station 4"


def test_burn_config_guard():
    # Reusing ONE Gas across two burn conditions with the SAME far but different (Tt4,pt4)
    # must trip the guard (no hidden state — pure function of far per fixed burn config).
    g = Gas.reacting_equilibrium()
    g.freeze_equilibrium(0.03, 1800.0, 1.3e6)
    tripped = False
    try:
        g.freeze_equilibrium(0.03, 1700.0, 1.3e6)     # different Tt4
    except AssertionError:
        tripped = True
    assert tripped, "burn-config guard must fire on a changed (Tt4,pt4)"


def _run_all():
    """Dependency-free runner so `python tests/test_rung6.py` works."""
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
