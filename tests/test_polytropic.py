"""Rung-2b verification: polytropic efficiency as a first-class knob.

Five gates (docs/rung2b-polytropic.md § Verification gates), in priority order:

1. reduce-to-ideal — e_c = e_t = 1 collapses the polytropic path onto the rung-1
   table to the digit (the polytropic exponent gc/1 == gc, etc.).
2. equivalence (the STRONGEST gate) — a polytropic engine at e_c=e_t=0.9 and an
   isentropic engine at the CONVERTED eta_c, eta_t are algebraically identical, not
   merely close: every station Tt/pt and every performance number agrees to ~1e-9.
   Run on the full Mattingly dual-gas, lossy, under-expanded case, so it doubles as
   "the polytropic anchor matches the isentropic anchor to machine precision."
3. cross-check — exercised on every run INSIDE the components (implied eta ==
   closed-form conversion); here it rides along in gates 1, 2, 4.
4. polytropic-native external anchor — Mattingly Example 7.1 with e_c=e_t=0.9 fed
   DIRECTLY (no conversion, no provisional pass), matching the book.
5. asymmetry / directional — eta_c < e < eta_t at every pi_c > 1, both gaps grow
   with pi_c and vanish as pi_c -> 1 (the reheat/preheat lesson).

Run with `python tests/test_polytropic.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.components import Compressor, Turbine, ram_recovery  # noqa: E402
from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import Gas  # noqa: E402

REL_TOL = 1e-3


def _close(actual, expected, rel=REL_TOL):
    return abs(actual - expected) <= rel * abs(expected)


def _raises(fn):
    """True if fn() raises (dependency-free stand-in for pytest.raises)."""
    try:
        fn()
    except Exception:
        return True
    return False


# Mattingly Example 7.1 case (docs/plans/rung2-anchor-mattingly.md), reused by the
# equivalence and anchor gates. Book inputs POLYTROPIC e_c = e_t = 0.9.
_MATT_GAS = Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9,
                gamma_t=1.3, cp_t=1239.0, R_t=285.9, hPR=42.8e6)
_MATT_FLIGHT = FlightCondition(T0=216.7, p0=50_000.0, M0=2.0)
_MATT_PI_C, _MATT_TT4 = 10.0, 1800.0
_MATT_P9 = _MATT_FLIGHT.p0 / 0.5          # book P0/P9 = 0.5 -> under-expanded
_MATT_COMMON = dict(pi_d=0.95 * ram_recovery(2.0), eta_b=0.98, pi_b=0.94,
                    eta_m=0.99, pi_n=0.96, p_exit=_MATT_P9)


def _matt_polytropic():
    """The Mattingly case run with the POLYTROPIC knob fed directly (e = 0.9)."""
    engine = build_turbojet(_MATT_GAS, _MATT_PI_C, _MATT_TT4, _MATT_FLIGHT.p0,
                            e_c=0.9, e_t=0.9, **_MATT_COMMON)
    return engine.run(_MATT_FLIGHT, 1.0)


# --- Gate 1: e = 1 reduces to the rung-1 ideal table ----------------------------

def test_polytropic_reduces_to_ideal():
    """e_c = e_t = 1 (everything else ideal) reproduces the rung-1 table.

    The polytropic exponent gc/e_c collapses to gc at e_c = 1, and the turbine's
    (Tt5/Tt4)^(1/(e_t*gt)) collapses to the isentropic form at e_t = 1 — so the
    reduce-to-ideal gate is structurally untouched by the new knob.
    """
    gas = Gas()  # single rung-1 gas
    engine = build_turbojet(gas, pi_c=10.0, Tt4=1500.0, p_ambient=50_000.0,
                            e_c=1.0, e_t=1.0)
    result = engine.run(FlightCondition(T0=250.0, p0=50_000.0, M0=0.85), mdot=1.0)
    st, perf = result.stations, result.performance

    # Rung-1 expected table (SPEC.md § Validation case).
    assert _close(st["3"].Tt, 552.4)
    assert _close(st["3"].pt / 1000.0, 801.9)
    assert _close(st["4"].far, 0.02304)
    assert _close(st["5"].Tt, 1239.7)
    assert _close(result.M9, 2.033)
    assert _close(result.V9, 1061.6)
    assert _close(perf.specific_thrust, 816.6)
    assert _close(perf.tsfc, 2.821e-5)
    assert _close(perf.eta_brayton, 0.4821)
    assert _close(perf.eta_thermal, 0.5477)


# --- Gate 2: polytropic == converted-isentropic, to machine precision -----------

def test_polytropic_isentropic_equivalence():
    """A polytropic engine and the CONVERTED isentropic engine are identical to ~1e-9.

    Run on the full Mattingly case (dual gas, all losses, under-expanded nozzle) so
    this is also the "polytropic anchor matches the isentropic anchor to machine
    precision" check. The conversion is exact for a calorically perfect gas:
      eta_c = (pi_c^gc - 1)/(pi_c^(gc/e_c) - 1)
      eta_t = (1 - tau_t)/(1 - tau_t^(1/e_t)),  tau_t from the (knob-independent) shaft.
    """
    e_c = e_t = 0.9
    poly = _matt_polytropic()

    # Convert e -> eta. eta_c is closed-form in pi_c; eta_t needs tau_t, which the
    # shaft fixes independent of turbine efficiency, so read it from the poly run.
    gc = _MATT_GAS.g_c
    eta_c = (_MATT_PI_C ** gc - 1.0) / (_MATT_PI_C ** (gc / e_c) - 1.0)
    tau_t = poly.stations["5"].Tt / _MATT_TT4
    eta_t = (1.0 - tau_t) / (1.0 - tau_t ** (1.0 / e_t))

    iso = build_turbojet(_MATT_GAS, _MATT_PI_C, _MATT_TT4, _MATT_FLIGHT.p0,
                         eta_c=eta_c, eta_t=eta_t, **_MATT_COMMON).run(_MATT_FLIGHT, 1.0)

    # Every station total agrees to ~1e-9 relative (algebraic identity, not closeness).
    for label in poly.stations:
        sp, si = poly.stations[label], iso.stations[label]
        assert _close(sp.Tt, si.Tt, 1e-9), f"Tt[{label}] poly {sp.Tt} != iso {si.Tt}"
        assert _close(sp.pt, si.pt, 1e-9), f"pt[{label}] poly {sp.pt} != iso {si.pt}"
        assert _close(sp.far, si.far, 1e-9) if si.far else sp.far == si.far

    # And every headline performance number.
    pp, pi = poly.performance, iso.performance
    for attr in ("specific_thrust", "tsfc", "eta_brayton", "eta_thermal",
                 "eta_propulsive", "eta_overall"):
        a, b = getattr(pp, attr), getattr(pi, attr)
        assert _close(a, b, 1e-9), f"{attr}: poly {a} != iso {b}"
    assert _close(poly.V9, iso.V9, 1e-9) and _close(poly.M9, iso.M9, 1e-9)


# --- Gate 4: polytropic-native external anchor (Mattingly 7.1) ------------------

def test_polytropic_anchor_mattingly():
    """Mattingly Example 7.1 with e_c = e_t = 0.9 fed DIRECTLY — no provisional pass.

    The contrast with tests/test_rung2.py::test_mattingly_example_7_1 is the point:
    the isentropic anchor must run a provisional pass to recover tau_t before it can
    convert e_t -> eta_t; the polytropic knob needs neither conversion nor pass. One
    build, one run, same book numbers.
    """
    result = _matt_polytropic()
    st, perf = result.stations, result.performance

    tol = 5e-4  # book rounds intermediates to 4 sig figs; actual deviation <= 0.015%
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


# --- Gate 5: the asymmetry eta_c < e < eta_t, growing with pi_c -----------------

def _implied_etas(pi_c, e=0.9):
    """Implied isentropic (eta_c, eta_t) for a polytropic engine at this pi_c.

    eta_c is closed-form in pi_c; eta_t comes from tau_t, read off a real run (the
    shaft sets tau_t, so it is the same number the turbine's own cross-check uses).
    """
    gc = _MATT_GAS.g_c
    eta_c = (pi_c ** gc - 1.0) / (pi_c ** (gc / e) - 1.0)
    run = build_turbojet(_MATT_GAS, pi_c, _MATT_TT4, _MATT_FLIGHT.p0,
                         e_c=e, e_t=e, **_MATT_COMMON).run(_MATT_FLIGHT, 1.0)
    tau_t = run.stations["5"].Tt / _MATT_TT4
    eta_t = (1.0 - tau_t) / (1.0 - tau_t ** (1.0 / e))
    return eta_c, eta_t


def test_efficiency_asymmetry():
    """Same e for both, yet eta_c < e < eta_t — and both gaps grow with pi_c.

    The reheat/preheat lesson (docs/rung2b-polytropic.md § The asymmetry): diverging
    isobars make a compressor look WORSE than its per-stage eff and a turbine BETTER.
    The split is set by pressure ratio — it is why e exists as a separate knob.
    """
    e = 0.9
    pis = [1.001, 2.0, 10.0, 30.0]  # 1.001 stands in for the pi_c -> 1 limit
    gaps_c, gaps_t = [], []
    for pi_c in pis:
        eta_c, eta_t = _implied_etas(pi_c, e)
        # Ordering holds at every pi_c > 1 (strict once away from the limit).
        assert eta_c <= e + 1e-12, f"eta_c {eta_c} should be <= e at pi_c={pi_c}"
        assert eta_t >= e - 1e-12, f"eta_t {eta_t} should be >= e at pi_c={pi_c}"
        gaps_c.append(e - eta_c)
        gaps_t.append(eta_t - e)

    # Anchor point (pi_c = 10): the headline numbers from the doc.
    eta_c10, eta_t10 = _implied_etas(10.0, e)
    assert _close(eta_c10, 0.8641, 1e-3) and _close(eta_t10, 0.9099, 1e-3)
    assert eta_c10 < e < eta_t10

    # Both gaps vanish as pi_c -> 1 ...
    assert gaps_c[0] < 1e-3 and gaps_t[0] < 1e-3, "gaps must -> 0 as pi_c -> 1"
    # ... and grow monotonically with pi_c.
    assert gaps_c == sorted(gaps_c), f"compressor gap not monotonic in pi_c: {gaps_c}"
    assert gaps_t == sorted(gaps_t), f"turbine gap not monotonic in pi_c: {gaps_t}"


# --- Mutual exclusivity: the one new validation the knob needs ------------------

def test_knobs_are_mutually_exclusive():
    """A non-default isentropic eta alongside a polytropic e is contradictory."""
    assert _raises(lambda: Compressor(10.0, eta_c=0.88, e_c=0.9))
    assert _raises(lambda: Turbine(eta_t=0.90, e_t=0.9))
    assert _raises(lambda: build_turbojet(Gas(), 10.0, 1500.0, 50_000.0,
                                          eta_c=0.88, e_c=0.9))
    # But e with the DEFAULT eta is fine (the common case), and so is e = 1 (ideal).
    assert not _raises(lambda: Compressor(10.0, e_c=0.9))
    assert not _raises(lambda: Turbine(e_t=1.0))


def _run_all():
    """Dependency-free runner so `python tests/test_polytropic.py` works."""
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
