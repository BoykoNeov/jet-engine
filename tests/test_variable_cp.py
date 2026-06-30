"""Rung-3 verification: variable cp(T), the thermally-perfect gas.

Six gates (docs/rung3-variable-cp.md § Verification gates), in priority order:

1. reduce-to-ideal (load-bearing) — a CPG Gas() reproduces the rung-1/2/2b tables
   TO THE DIGIT. This is owned by the EXISTING suites (test_validation, test_rung2,
   test_polytropic), which stay green untouched; a one-line guard here documents it.
2. round-trip inverses — T_from_h(h(T)) == T and T_from_pr(pr(T)) == T to ~1e-10,
   plus monotonicity of h and pr, across the working range (a standing assert too).
3. discriminating CPG-vs-integral check, run DUAL-SECTION — two distinct flat-cp
   polynomials through the integral path reproduce the rung-2 dual-cp turbojet to
   ~3e-4 (NOT 1e-9): proof the integral path is genuinely pr=exp(phi/R) AND that
   cold/hot are not confused (a routing bug would blow the gap wide open).
4. air-table isentropic anchor — isentropic compression of air, pi=10 from 300 K,
   lands at the gas-table ~574 K (vs the calorically-perfect 579 K).
5. external machinery anchors (SOURCED, docs/plans/rung3-anchor-cengel.md) — Cengel
   9-89 (T2s, T4s, cycle eta_th) and Mattingly Ex 2.7/2.8 (compression, nozzle),
   to ~0.15%. Topology caveat: Cengel is a POWER cycle, so these anchor the
   property+process MACHINERY (tested directly on the gas), not build_turbojet.
6. directional / gas-table effect — TPG losses move thrust/TSFC the right way, and
   TPG compression lands COOLER than CPG at the same design point (cp rises with T).

Run with `python tests/test_variable_cp.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.components import ram_recovery  # noqa: E402
from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import Gas  # noqa: E402


def _close(actual, expected, rel=1.5e-3):
    return abs(actual - expected) <= rel * abs(expected)


def _flat(cp, R):
    """A constant-cp polynomial (A_low == A_high == cp/R): a TPG section whose cp(T)
    happens to be flat, used to exercise the integral path against a known answer."""
    return ((cp / R, 0.0, 0.0, 0.0, 0.0), (cp / R, 0.0, 0.0, 0.0, 0.0))


# Rung-1 design point and the Mattingly Ex 7.1 case, reused below.
_FLIGHT_R1 = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_FLIGHT_MATT = FlightCondition(T0=216.7, p0=50_000.0, M0=2.0)
_MATT_COMMON = dict(pi_d=0.95 * ram_recovery(2.0), eta_c=0.8641, eta_b=0.98, pi_b=0.94,
                    eta_t=0.9099, eta_m=0.99, pi_n=0.96, p_exit=50_000.0 / 0.5)


# --- Gate 1: reduce-to-ideal still holds (owned by the other suites) -------------

def test_reduce_to_ideal_guard():
    """A CPG Gas() through the rung-3 code path still reproduces the rung-1 table.

    The full gate lives in test_validation/test_rung2/test_polytropic (untouched);
    this is a fast guard that the rung-3 refactor did not disturb the CPG branch.
    """
    r = build_turbojet(Gas(), pi_c=10.0, Tt4=1500.0, p_ambient=_FLIGHT_R1.p0).run(_FLIGHT_R1, 1.0)
    assert _close(r.stations["3"].Tt, 552.4)
    assert _close(r.stations["5"].Tt, 1239.7)
    assert _close(r.performance.specific_thrust, 816.6)
    assert _close(r.M9, 2.033)


# --- Gate 2: round-trip inverses + monotonicity ---------------------------------

def test_roundtrip_inverses_and_monotonicity():
    """T_from_h(h(T)) == T and T_from_pr(pr(T)) == T to ~1e-9; h, pr strictly up."""
    g = Gas.thermally_perfect()
    cold_Ts = [200.0, 250.0, 300.0, 500.0, 800.0, 1000.0, 1240.0, 1300.0]
    hot_Ts = [800.0, 1000.0, 1240.0, 1500.0, 1800.0, 2000.0]

    for T in cold_Ts:
        assert _close(g.T_from_h_c(g.h_c(T)), T, 1e-9), f"cold h round-trip at {T}"
        assert _close(g.T_from_pr_c(g.pr_c(T)), T, 1e-9), f"cold pr round-trip at {T}"
    for T in hot_Ts:
        assert _close(g.T_from_h_t(g.h_t(T)), T, 1e-9), f"hot h round-trip at {T}"
        assert _close(g.T_from_pr_t(g.pr_t(T)), T, 1e-9), f"hot pr round-trip at {T}"

    # Monotonicity (cp > 0 => h, pr strictly increasing): the well-posedness the
    # inverses rely on, across the join at 1000 K.
    for Ts, h, pr in ((cold_Ts, g.h_c, g.pr_c), (hot_Ts, g.h_t, g.pr_t)):
        for a, b in zip(Ts, Ts[1:]):
            assert h(b) > h(a) and pr(b) > pr(a), f"not monotone on [{a},{b}]"


# --- Gate 3: dual-section discriminating CPG-vs-integral check -------------------

def test_discriminating_dual_section_integral_path():
    """Flat-cp polynomials through the integral path reproduce the CPG turbojet to
    ~3e-4 (NOT 1e-9, NOT wildly off).

    The two sections carry DISTINCT flats (cold 1004/286.9, hot 1239/285.9, the
    rung-2 dual gas), so a routing bug that called pr_c where pr_t belongs would
    swap a 0.286 exponent for a 0.231 one and blow the gap to tens of percent. The
    measured gaps (Tt3 ~2e-4, F/m ~1.4e-4) prove BOTH: the integral path genuinely
    uses R/cp(T) (gap > 1e-9), and cold/hot are routed correctly (gap << 1%).
    """
    cpg = Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=1.3, cp_t=1239.0, R_t=285.9, hPR=42.8e6)
    flat_tpg = Gas(R_c=286.9, R_t=285.9, hPR=42.8e6,
                   cp_c_coeffs=_flat(1004.0, 286.9), cp_t_coeffs=_flat(1239.0, 285.9))

    rc = build_turbojet(cpg, 10.0, 1800.0, _FLIGHT_MATT.p0, **_MATT_COMMON).run(_FLIGHT_MATT, 1.0)
    rt = build_turbojet(flat_tpg, 10.0, 1800.0, _FLIGHT_MATT.p0, **_MATT_COMMON).run(_FLIGHT_MATT, 1.0)

    gap_Tt3 = abs(rt.stations["3"].Tt - rc.stations["3"].Tt) / rc.stations["3"].Tt
    gap_F = abs(rt.performance.specific_thrust - rc.performance.specific_thrust) / rc.performance.specific_thrust

    # The integral path is genuinely R/cp-based: the gap must EXIST (else it secretly
    # uses (gamma-1)/gamma and would match to 1e-9).
    assert gap_Tt3 > 3e-5, f"Tt3 gap {gap_Tt3:.1e} too small — integral path not exercised?"
    # ...but the routing is correct and the gap is the small rounded-R residual.
    assert gap_Tt3 < 1e-3, f"Tt3 gap {gap_Tt3:.1e} too large — cold/hot section confusion?"
    assert gap_F < 1e-3, f"F/m gap {gap_F:.1e} too large — section confusion?"


# --- Gate 4: air-table isentropic anchor (~574 K) -------------------------------

def test_air_table_isentropic_anchor():
    """Isentropic compression of air, pi=10 from 300 K, lands at the gas-table ~574 K.

    Datum-independent (a pr ratio), so immune to the table's enthalpy/entropy datum.
    The calorically-perfect answer is 300*10^0.2857 = 579.2 K — the ~5 K shortfall IS
    the variable-cp effect (cp rises with T, so less temperature for the same work).
    """
    g = Gas.thermally_perfect()
    T2 = g.T_from_pr_c(g.pr_c(300.0) * 10.0)
    assert _close(T2, 574.1, 2e-3), f"air-table T2: {T2}"             # gas table ~574.1
    assert T2 < 300.0 * 10.0 ** 0.2857, "variable cp must land below the CPG 579 K"


# --- Gate 5: external machinery anchors (Cengel 9-89, Mattingly 2.7/2.8) ---------

def test_cengel_9_89_machinery():
    """Cengel 9-89 (air, Table A-17): the rung-3 station-3/5 substate machinery.

    POWER cycle (turbine expands the full pi), so per the topology caveat this is
    tested on the GAS directly, not through build_turbojet. T2s/T4s are the exact
    rung-3 compressor/turbine substate equations on a single air section; the cycle
    eta_th is the same h-difference energetics (eta_c=0.83, eta_t=0.87, r_p=10).
    """
    g = Gas.thermally_perfect()
    T2s = g.T_from_pr_c(g.pr_c(295.0) * 10.0)
    T4s = g.T_from_pr_c(g.pr_c(1240.0) / 10.0)
    assert _close(T2s, 564.9), f"Cengel T2s: {T2s}"
    assert _close(T4s, 689.6), f"Cengel T4s: {T4s}"

    eta_c, eta_t = 0.83, 0.87
    h1, h3 = g.h_c(295.0), g.h_c(1240.0)
    h2 = h1 + (g.h_c(T2s) - h1) / eta_c
    h4 = h3 - eta_t * (h3 - g.h_c(T4s))
    eta_th = ((h3 - h4) - (h2 - h1)) / (h3 - h2)
    assert _close(eta_th, 0.3013, 2e-3), f"Cengel cycle eta_th: {eta_th}"


def test_mattingly_2_7_2_8_machinery():
    """Mattingly's OWN variable-cp examples (his Eq 2.53-2.58 gas-table method).

    2.7 = isentropic compression (the station-3 substate). 2.8 = isentropic nozzle:
    it exercises the station-9 pair TOGETHER — V2 from the ENTHALPY split and P2/P1
    from the pr ratio — and varies gamma over a wide range, covering the M9 blind
    spot the flat-cp gate-3 cannot see (docs/rung3-variable-cp.md gate 3).
    """
    g = Gas.thermally_perfect()
    # Ex 2.7: 293.15 K, pi = 15 -> 627.57 K.
    assert _close(g.T_from_pr_c(g.pr_c(293.15) * 15.0), 627.57), "Mattingly 2.7"

    # Ex 2.8: 3000 R, dh = 179.74 Btu/lbm -> 2377.7 R, P2/P1 = 0.3757.
    BTU_LBM, R_to_K = 2326.0, 1.0 / 1.8
    T1 = 3000.0 * R_to_K
    T2 = g.T_from_h_c(g.h_c(T1) - 179.74 * BTU_LBM)
    p_ratio = g.pr_c(T2) / g.pr_c(T1)                                 # = P2/P1 (pr ratio)
    assert _close(T2 / R_to_K, 2377.7), f"Mattingly 2.8 T2: {T2 / R_to_K} R"
    assert _close(p_ratio, 0.3757, 2e-3), f"Mattingly 2.8 P2/P1: {p_ratio}"


# --- Gate 6: directional + the gas-table effect ---------------------------------

def test_tpg_directional_and_gas_table_effect():
    """Losses move thrust/TSFC the right way on the TPG gas; and TPG compression
    lands COOLER than CPG at the same design point (the gas-table effect)."""
    gas = Gas.thermally_perfect()
    ideal = build_turbojet(gas, 10.0, 1500.0, _FLIGHT_R1.p0).run(_FLIGHT_R1, 1.0)
    lossy = build_turbojet(gas, 10.0, 1500.0, _FLIGHT_R1.p0,
                           pi_d=0.95, eta_c=0.88, eta_b=0.99, pi_b=0.95,
                           eta_t=0.90, eta_m=0.99, pi_n=0.98).run(_FLIGHT_R1, 1.0)
    assert lossy.performance.specific_thrust < ideal.performance.specific_thrust, "losses cut thrust"
    assert lossy.performance.tsfc > ideal.performance.tsfc, "losses raise TSFC"

    # Gas-table effect: at the same pi_c, variable-cp compression reaches a LOWER
    # Tt3 than constant-cp (cp climbs with T, so the same pressure work is a smaller
    # temperature rise). Compare ideal TPG vs ideal CPG at the rung-1 design point.
    cpg = build_turbojet(Gas(), 10.0, 1500.0, _FLIGHT_R1.p0).run(_FLIGHT_R1, 1.0)
    assert ideal.stations["3"].Tt < cpg.stations["3"].Tt, "TPG compression must land cooler"


def _run_all():
    """Dependency-free runner so `python tests/test_variable_cp.py` works."""
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
