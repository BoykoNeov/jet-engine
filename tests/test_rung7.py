"""Rung-7 verification: thermal NOx — the extended Zeldovich mechanism + kinetics.

Gates (docs/rung7-spec.md § Verification gates), priority order:

1. reduce-to-lower-rung (LOAD-BEARING) — NO/N are a SUPERIMPOSED layer, never added to
   _equil_solve, so rungs 1-6 stay green untouched (run the rest of the suite) and the
   cycle is bit-for-bit rung 6 (the equilibrium composition carries no NO/N; the cycle
   far is unchanged).
2. THE K-CHECK — (k1f k2f)/(k1r k2r) == Kc(N2+O2<=>2NO)=exp(-dG0/RuT) from the existing
   _g_molar (a6+a7). Certifies the transcribed rate constants AND NO's thermochemistry
   jointly (rung-6-style joint certification). Measured ratio 1.035-1.044.
3. the tau->inf asymptote — kinetic NO -> the independently-computed equilibrium NO.
4. formation + entropy self-checks — h(298.15)=dHf, s(298.15)=S298 for NO/N; a6/a7 vs GRI.
5. magnitude + kinetic freezing — equilibrium NO in band; kinetic << equilibrium at tau=3 ms;
   characteristic NO time >> residence.
6. T-sensitivity — initial rate rises steeply (~exp(-38370/T)), monotone.
7. pressure independence — equilibrium NO carries no (p/p0) factor (Dnu=0).

Run with `python tests/test_rung7.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, _F_STOICH, _Ru, _T_REF, _HF298, _S298, _kcheck_ratio, _kp_no,
    _equilibrium_no_fraction, _equilibrium_composition, _sens_h, _s_molar,
    _a6_of, _a7_of, _SP_REACT,
)

_FLIGHT = FlightCondition(T0=216.7, p0=18_750.0, M0=2.0)
_DESIGN = dict(pi_c=10.0, Tt4=1800.0, p_ambient=18_750.0,
               pi_d=0.95, eta_c=0.90, eta_b=0.98, pi_b=0.95, eta_t=0.90,
               eta_m=0.99, pi_n=0.97)
_P1 = 101325.0
_FST = _F_STOICH * 0.999

# GRI-Mech 3.0 TABULATED a6/a7 (low range) — the independent cross-check target (§4).
_GRI_A6_LOW = {"NO": 9.84509964e3, "N": 5.61046370e4}
_GRI_A7_LOW = {"NO": 2.28061001e0, "N": 4.19390870e0}


def _close(a, b, rel=1e-9, abs_=0.0):
    return abs(a - b) <= rel * abs(b) + abs_


# --------------------------------------------------------------------------- #
# GATE 1 — reduce-to-rung-6: NO/N are a superimposed layer; the cycle is unchanged.
# --------------------------------------------------------------------------- #
def test_reduce_to_rung6_cycle_untouched():
    rE = build_turbojet(Gas.reacting_equilibrium(), **_DESIGN).run(_FLIGHT, 50.0)
    st4 = rE.stations["4"]
    # The equilibrium composition NEVER contains NO/N (they are not in _equil_solve).
    comp = _equilibrium_composition(st4.far, st4.Tt, st4.pt)
    assert "NO" not in comp and "N" not in comp, "NO/N leaked into the C/H/O equilibrium solve"
    assert set(comp) == set(_SP_REACT) | {"N2", "Ar"}, f"unexpected species in pool: {set(comp)}"
    # The station-4 far still matches rung-6 Fork B within the rung-6 bound (adding NO/N
    # data to the dicts did not perturb the cycle) — the reduce-to-rung-6 invariant.
    fB = build_turbojet(Gas.reacting_forkb(), **_DESIGN).run(_FLIGHT, 50.0).stations["4"].far
    assert 0.0 < (st4.far - fB) / fB < 0.005, "cycle far drifted — NO/N must not touch the cycle"


# --------------------------------------------------------------------------- #
# GATE 2 — the thermo-kinetic K-check (the load-bearing joint certification).
# --------------------------------------------------------------------------- #
def test_kcheck_rates_vs_thermo():
    for T in (1800.0, 2000.0, 2200.0, 2500.0):
        r = _kcheck_ratio(T)
        assert 0.95 < r < 1.10, f"K-check ratio {r:.4f} at T={T} out of [0.95, 1.10]"


# --------------------------------------------------------------------------- #
# GATE 3 — tau->inf asymptote: kinetic NO recovers the equilibrium NO.
# --------------------------------------------------------------------------- #
def test_tau_infinity_recovers_equilibrium():
    g = Gas.reacting_equilibrium()
    n = g.thermal_nox(_FST, 2300.0, _P1, tau=2.0)          # tau >> tau_NO (~90 ms)
    assert _close(n.x_no, n.x_no_eq, rel=1e-3), f"tau->inf: {n.ppm} vs eq {n.ppm_eq}"


# --------------------------------------------------------------------------- #
# GATE 4 — formation + entropy self-checks (NO/N); derived a6/a7 vs GRI-Mech.
# --------------------------------------------------------------------------- #
def test_formation_entropy_self_check_and_gri():
    for s in ("NO", "N"):
        h298 = _Ru * (_sens_h(s, _T_REF) + _a6_of(s))
        assert _close(h298, _HF298[s], rel=1e-9, abs_=1e-6), f"{s}: h(298)={h298}!={_HF298[s]}"
        assert _close(_s_molar(s, _T_REF), _S298[s], rel=1e-9, abs_=1e-6), f"{s}: s(298) off"
    # N is dead-on vs GRI; NO's a7 (entropy) is tight, its a6 carries the ΔHf° spread (<2%).
    for s in ("NO", "N"):
        a6dev = abs(_a6_of(s) - _GRI_A6_LOW[s]) / abs(_GRI_A6_LOW[s])
        a7dev = abs(_a7_of(s) - _GRI_A7_LOW[s]) / abs(_GRI_A7_LOW[s])
        assert a7dev < 0.005, f"{s}: a7 dev {a7dev:.4f} vs GRI too large"
    assert abs(_a6_of("N") - _GRI_A6_LOW["N"]) / _GRI_A6_LOW["N"] < 5e-4, "N a6 vs GRI"
    a6dev_no = abs(_a6_of("NO") - _GRI_A6_LOW["NO"]) / _GRI_A6_LOW["NO"]
    assert 0.005 < a6dev_no < 0.02, f"NO a6 dev {a6dev_no:.4f} — expected the ~1.2% ΔHf° spread"
    # NO carries a POSITIVE formation enthalpy (endothermic); N is very endothermic.
    assert _HF298["NO"] > 0 and _HF298["N"] > 4e5


# --------------------------------------------------------------------------- #
# GATE 5 — magnitude + kinetic freezing (kinetic NO frozen far below equilibrium).
# --------------------------------------------------------------------------- #
def test_magnitude_and_kinetic_freezing():
    g = Gas.reacting_equilibrium()
    n = g.thermal_nox(_FST, 2300.0, _P1, tau=3e-3)
    assert 2500.0 < n.ppm_eq < 3500.0, f"equilibrium NO {n.ppm_eq:.0f} ppm out of band"
    assert n.fraction_of_equil < 0.10, f"kinetic NO {n.fraction_of_equil:.3f} not frozen below eq"
    # Characteristic NO time >> combustor residence (~ms): the physics of the freezing.
    n21 = g.thermal_nox(_FST, 2100.0, _P1, tau=3e-3)
    assert n21.char_time > 0.100, f"tau_NO(2100K)={n21.char_time*1000:.0f} ms should be >> residence"
    # ABSOLUTE-MAGNITUDE lower bound (the one the other gates are blind to): in the first
    # 1 ms (<< tau_NO=89 ms, so growth is ~linear) the KINETIC rate deposits ~34.5 ppm at
    # 2300 K stoich — a TWO-SIDED band [10, 100] ppm/ms. A too-SLOW error (a concentration
    # units slip, a dropped factor) makes NO too small and would sail through every other
    # gate (K-check tests only ratios; tau->inf clamps to thermodynamic [NO]_e); this pins
    # the absolute kinetic magnitude. Order-of-magnitude literature, NOT a book digit
    # (docs/plans/rung7-anchor-nox.md § What stays un-anchored).
    ppm_1ms = g.thermal_nox(_FST, 2300.0, _P1, tau=1e-3).ppm
    assert 10.0 < ppm_1ms < 100.0, f"NO@1ms,2300K = {ppm_1ms:.1f} ppm out of magnitude band"


# --------------------------------------------------------------------------- #
# GATE 6 — T-sensitivity: the initial NO rate rises steeply and monotonically.
# --------------------------------------------------------------------------- #
def test_temperature_sensitivity():
    g = Gas.reacting_equilibrium()
    rates = [g.thermal_nox(_FST, T, _P1, tau=1e-3).initial_rate
             for T in (1800.0, 2000.0, 2200.0, 2400.0)]
    assert all(b > a for a, b in zip(rates, rates[1:])), f"initial rate not monotone in T: {rates}"
    assert rates[2] / rates[1] > 20.0, f"2200K/2000K rate ratio {rates[2]/rates[1]:.1f} too flat"


# --------------------------------------------------------------------------- #
# GATE 7 — equilibrium NO is pressure-independent (Dnu=0, no (p/p0) factor).
# --------------------------------------------------------------------------- #
def test_equilibrium_no_pressure_independent():
    # A genuinely LEAN mixture: O2 is dominated by excess air (not dissociation), so it
    # barely moves with pressure -> equilibrium NO is ~pressure-independent. Contrast
    # rung-6 dissociation (CO/(CO+CO2)), which falls sharply with pressure.
    T = 2000.0
    xs = []
    for p_atm in (1.0, 13.0):
        comp = _equilibrium_composition(0.030, T, p_atm * _P1)
        xs.append(_equilibrium_no_fraction(comp, T))
    assert abs(xs[0] - xs[1]) / xs[0] < 0.02, f"lean equilibrium NO drifted with p: {xs}"
    # And Kp_NO itself takes no pressure argument (structural: Dnu=0).
    assert _kp_no(2000.0) > 0.0


def _run_all():
    """Dependency-free runner so `python tests/test_rung7.py` works."""
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
