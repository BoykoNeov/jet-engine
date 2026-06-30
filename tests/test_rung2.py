"""Rung-2 verification: reduce-to-ideal, directional checks, and the external anchor.

Three gates, in priority order (docs/rung2-spec.md § Verification gates):

1. reduce-to-ideal — collapse the dual gas with unified() and set every loss to
   ideal; the rung-1 table must come back to the digit. (The primary rung-1 case
   in test_validation.py already exercises this via Gas() defaults; here we prove
   the unified() collapse itself works from a genuinely dual gas.)
2. directional — turning losses ON must lower specific thrust, raise TSFC, and
   lower the real thermal efficiency.
3. external anchor — Mattingly *Elements of Propulsion* Example 7.1, to ~0.1-0.2%
   (the book rounds its intermediates to 4 sig figs). See
   docs/plans/rung2-anchor-mattingly.md. The book inputs POLYTROPIC e_c, e_t; our
   code uses ISENTROPIC eta_c, eta_t, so the test converts (exact for a perfect
   gas).

Run with `python tests/test_rung2.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.components import ram_recovery  # noqa: E402
from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import Gas  # noqa: E402

REL_TOL = 1e-3


def _close(actual, expected, rel=REL_TOL):
    return abs(actual - expected) <= rel * abs(expected)


# --- Gate 1: unified() collapses a dual gas back to the rung-1 single gas -------

def test_unify_reduces_to_rung1():
    """A genuinely dual gas, unified() and run fully ideal, reproduces rung 1.

    The hot section is deliberately different (Mattingly's 1.3 / 1239 / 285.9);
    unified() must collapse the WHOLE triple onto the cold defaults (1.4 / 1004 /
    287) so the result is the rung-1 gas exactly — which is what makes the rung-2
    machinery reproduce the rung-1 table to the digit.
    """
    dual = Gas(gamma_t=1.3, cp_t=1239.0, R_t=285.9)  # hot != cold
    gas = dual.unified()
    assert gas.gamma_t == gas.gamma_c and gas.cp_t == gas.cp_c and gas.R_t == gas.R_c

    engine = build_turbojet(gas, pi_c=10.0, Tt4=1500.0, p_ambient=50_000.0)  # all ideal
    result = engine.run(FlightCondition(T0=250.0, p0=50_000.0, M0=0.85), mdot=1.0)
    st, perf = result.stations, result.performance

    # Rung-1 expected table (SPEC.md § Validation case).
    assert _close(st["2"].Tt, 286.1)
    assert _close(st["3"].Tt, 552.4)
    assert _close(st["3"].pt / 1000.0, 801.9)
    assert _close(st["4"].far, 0.02304)
    assert _close(st["5"].Tt, 1239.7)
    assert _close(result.M9, 2.033)
    assert _close(result.V9, 1061.6)
    assert _close(perf.specific_thrust, 816.6)
    assert _close(perf.tsfc, 2.821e-5)
    assert _close(perf.eta_brayton, 0.4821)
    # The KE-based thermal efficiency in the ideal limit is 0.5477, NOT 0.4821
    # (different quantity — see docs/rung2-spec.md § Performance).
    assert _close(perf.eta_thermal, 0.5477)


# --- Gate 2: losses move the numbers the right way ------------------------------

def test_losses_are_directional():
    """eta < 1 must lower specific thrust, raise TSFC, lower real thermal eff."""
    gas = Gas()  # single gas, isolate the efficiency effect from the dual-cp effect
    flight = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)

    ideal = build_turbojet(gas, pi_c=10.0, Tt4=1500.0, p_ambient=flight.p0)
    lossy = build_turbojet(
        gas, pi_c=10.0, Tt4=1500.0, p_ambient=flight.p0,
        pi_d=0.95, eta_c=0.88, eta_b=0.99, pi_b=0.95, eta_t=0.90, eta_m=0.99, pi_n=0.98,
    )
    ri = ideal.run(flight, 1.0).performance
    rl = lossy.run(flight, 1.0).performance

    assert rl.specific_thrust < ri.specific_thrust, "losses must reduce specific thrust"
    assert rl.tsfc > ri.tsfc, "losses must raise TSFC"
    assert rl.eta_thermal < ri.eta_thermal, "losses must lower real thermal efficiency"


# --- Gate 3: external anchor — Mattingly Example 7.1 ----------------------------

def test_mattingly_example_7_1():
    """Reproduce Mattingly Example 7.1 (docs/plans/rung2-anchor-mattingly.md).

    Inputs use the book's dual gas and component losses; the polytropic e_c, e_t
    are converted to our isentropic eta_c, eta_t. The conversion for the turbine
    needs tau_t = Tt5/Tt4, which is INDEPENDENT of eta_t (the shaft sets the drop
    without it), so a provisional pass recovers tau_t, then the real pass runs.
    """
    # Book gas: cold air / hot products, R = (gamma-1)/gamma * cp per section.
    gas = Gas(
        gamma_c=1.4, cp_c=1004.0, R_c=286.9,
        gamma_t=1.3, cp_t=1239.0, R_t=285.9,
        hPR=42.8e6,
    )
    M0, T0 = 2.0, 216.7
    p0 = 50_000.0                 # absolute value is arbitrary (results are ratios)
    p9 = p0 / 0.5                 # book P0/P9 = 0.5 -> under-expanded, P9 = 2*p0
    flight = FlightCondition(T0=T0, p0=p0, M0=M0)
    pi_c, Tt4 = 10.0, 1800.0

    # Inlet net recovery: pi_d = pi_d_max * ram_recovery(M0). At M0=2, eta_r=0.925.
    pi_d = 0.95 * ram_recovery(M0)
    assert _close(pi_d, 0.87875)

    # Compressor: polytropic e_c=0.9 -> isentropic eta_c (exact for a perfect gas).
    gc, e_c = gas.g_c, 0.9
    eta_c = (pi_c ** gc - 1.0) / (pi_c ** (gc / e_c) - 1.0)

    common = dict(pi_d=pi_d, eta_c=eta_c, eta_b=0.98, pi_b=0.94, eta_m=0.99,
                  pi_n=0.96, p_exit=p9)

    # Provisional pass (eta_t=1) just to recover tau_t = Tt5/Tt4.
    prov = build_turbojet(gas, pi_c, Tt4, p0, eta_t=1.0, **common).run(flight, 1.0)
    tau_t = prov.stations["5"].Tt / Tt4
    e_t = 0.9
    eta_t = (1.0 - tau_t) / (1.0 - tau_t ** (1.0 / e_t))

    result = build_turbojet(gas, pi_c, Tt4, p0, eta_t=eta_t, **common).run(flight, 1.0)
    st, perf = result.stations, result.performance

    # Book rounds intermediates to 4 sig figs; actual deviations are <= 0.015%, so
    # 0.05% keeps margin over that rounding while still catching real regressions.
    tol = 5e-4
    assert _close(result.V0, 590.0, tol), f"V0: {result.V0}"
    assert _close(st["4"].far, 0.03567, tol), f"f: {st['4'].far}"
    assert _close(result.M9, 2.253, tol), f"M9: {result.M9}"
    assert _close(result.T9, 833.4, tol), f"T9: {result.T9}"
    assert _close(result.V9, 1253.8, tol), f"V9: {result.V9}"
    assert _close(perf.specific_thrust, 806.9, tol), f"F/mdot: {perf.specific_thrust}"
    assert _close(perf.tsfc, 4.421e-5, tol), f"TSFC: {perf.tsfc}"
    assert _close(perf.eta_thermal, 0.4192, tol), f"eta_T: {perf.eta_thermal}"
    assert _close(perf.eta_propulsive, 0.7439, tol), f"eta_P: {perf.eta_propulsive}"
    assert _close(perf.eta_overall, 0.3118, tol), f"eta_O: {perf.eta_overall}"


def _run_all():
    """Dependency-free runner so `python tests/test_rung2.py` works."""
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
