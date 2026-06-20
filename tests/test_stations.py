"""Per-station checks, filled in as each station is derived and implemented.

This complements tests/test_validation.py (the whole-cycle target, red until the
engine is fully wired). Here each station is verified the moment it lands, which
is the workflow in docs/plans/rung1-plan.md: derive -> code -> verify, one
station at a time. Station 0 is done as the worked example; add the rest below.

Run with `python tests/test_stations.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import Engine, FlightCondition  # noqa: E402
from turbojet.gas import Gas  # noqa: E402

GAS = Gas(gamma=1.4, cp=1004.0, R=287.0, hPR=42.8e6)
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
MDOT = 1.0
REL_TOL = 1e-3  # "~0.1%"


def _close(actual, expected, rel=REL_TOL):
    return abs(actual - expected) <= rel * abs(expected)


def test_station0_freestream():
    """Station 0: Tt0=286.1 K, pt0=80.19 kPa, V0=269.4 m/s (SPEC.md table)."""
    engine = Engine(GAS, components=[])  # freestream needs no components
    state0, V0 = engine.freestream(FLIGHT, mdot=MDOT)

    assert _close(state0.Tt, 286.1), f"Tt0: got {state0.Tt}"
    assert _close(state0.pt / 1000.0, 80.19), f"pt0: got {state0.pt / 1000.0}"
    assert _close(V0, 269.4), f"V0: got {V0}"
    assert state0.mdot == MDOT
    assert state0.far == 0.0


# --- TEMPLATE for the next stations (uncomment + fill in as you derive each) ---
#
# def test_station2_inlet():
#     """Inlet (ideal): Tt2 == Tt0, pt2 == pt0 (SPEC.md table: 286.1 K, 80.19 kPa)."""
#     engine = Engine(GAS, components=[])
#     state0, _ = engine.freestream(FLIGHT, mdot=MDOT)
#     state2 = Inlet().apply(state0, GAS)
#     assert _close(state2.Tt, 286.1)
#     assert _close(state2.pt / 1000.0, 80.19)
#
# ... and so on for compressor (552.4 K / 801.9 kPa), burner (f=0.02304),
# turbine (1239.7 K / 411.5 kPa), nozzle (M9=2.033, T9=678.8 K, V9=1061.6 m/s).


def _run_all():
    """Dependency-free runner so `python tests/test_stations.py` works."""
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
