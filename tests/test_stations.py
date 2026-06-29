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

from turbojet.components import Burner, Compressor, Inlet, Turbine  # noqa: E402
from turbojet.engine import Engine, FlightCondition  # noqa: E402
from turbojet.gas import Gas  # noqa: E402

GAS = Gas(gamma=1.4, cp=1004.0, R=287.0, hPR=42.8e6)
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
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


def test_station2_inlet():
    """Inlet (ideal): Tt2 == Tt0, pt2 == pt0 (SPEC.md table: 286.1 K, 80.19 kPa)."""
    engine = Engine(GAS, components=[])
    state0, _ = engine.freestream(FLIGHT, mdot=MDOT)
    state2 = Inlet().apply(state0, GAS)

    # Spec table values...
    assert _close(state2.Tt, 286.1), f"Tt2: got {state2.Tt}"
    assert _close(state2.pt / 1000.0, 80.19), f"pt2: got {state2.pt / 1000.0}"
    # ...and the defining property: an ideal inlet preserves the station-0 totals.
    assert state2.Tt == state0.Tt and state2.pt == state0.pt
    assert state2.mdot == state0.mdot and state2.far == state0.far


def test_station3_compressor():
    """Compressor (ideal): Tt3=552.4 K, pt3=801.9 kPa (SPEC.md table), and the
    PRIMARY HAND-CHECK: eta_th = 1 - Tt2/Tt3 must equal the closed form
    1 - 1/pi_c^g, both 0.4821.

    The spec says: if those two disagree the compression leg is buggy, fix it
    before trusting anything else. That check lives in test_validation.py too,
    but there it is gated behind the (still-unimplemented) engine wiring -- so we
    run it HERE the moment the compressor lands, which is the whole point of this
    station per the spec's "fix it first" rule.
    """
    engine = Engine(GAS, components=[])
    state0, _ = engine.freestream(FLIGHT, mdot=MDOT)
    state2 = Inlet().apply(state0, GAS)
    state3 = Compressor(PI_C).apply(state2, GAS)

    # Spec table values -- the only guard that catches a wrong pi_c or exponent
    # in absolute terms.
    assert _close(state3.Tt, 552.4), f"Tt3: got {state3.Tt}"
    assert _close(state3.pt / 1000.0, 801.9), f"pt3: got {state3.pt / 1000.0}"
    assert state3.mdot == state2.mdot and state3.far == state2.far

    # Primary hand-check, two ways. Structurally exact (Tt3 == Tt2*pi_c^g makes
    # Tt2/Tt3 == 1/pi_c^g to machine precision), so assert it TIGHT -- a failure
    # beyond float epsilon means Tt3 was not computed via the isentropic relation.
    eta_from_states = 1.0 - state2.Tt / state3.Tt
    eta_closed_form = 1.0 - 1.0 / (PI_C ** GAS.g)
    assert abs(eta_from_states - eta_closed_form) < 1e-9, (
        f"compression-leg bug: {eta_from_states} != {eta_closed_form}"
    )
    # The 0.4821 target is rounded, so it stays at the ~0.1% spec tolerance.
    assert _close(eta_from_states, 0.4821), f"eta_th: got {eta_from_states}"


def test_station4_burner():
    """Burner (ideal): f=0.02304, pt4 == pt3 (801.9 kPa), Tt4=1500 K, and mass
    grows by the fuel: mdot4 == mdot3*(1+f) (SPEC.md table + § Conservation).

    The mdot assertion is the one that exercises the new mass-growth line -- the
    others pass even if that line were forgotten.
    """
    engine = Engine(GAS, components=[])
    state0, _ = engine.freestream(FLIGHT, mdot=MDOT)
    state2 = Inlet().apply(state0, GAS)
    state3 = Compressor(PI_C).apply(state2, GAS)
    state4 = Burner(TT4).apply(state3, GAS)

    # Spec table values.
    assert _close(state4.far, 0.02304), f"f: got {state4.far}"
    assert _close(state4.pt / 1000.0, 801.9), f"pt4: got {state4.pt / 1000.0}"
    assert state4.Tt == TT4, f"Tt4: got {state4.Tt}"
    # Defining properties: ideal burner holds pt; fuel mass joins the stream.
    assert state4.pt == state3.pt, "ideal burner: pt4 == pt3"
    assert _close(state4.mdot, state3.mdot * (1.0 + state4.far)), f"mdot4: got {state4.mdot}"


def test_station5_turbine():
    """Turbine (ideal, the keystone): Tt5=1239.7 K, pt5=411.5 kPa (SPEC.md table).

    The shaft balance is the physics under test: delta_Tt = (Tt3 - Tt2)/(1 + f),
    so Tt5 = Tt4 - delta_Tt. The ABSOLUTE spec values are the real guard here --
    every in-component assert is structurally exact (mass is trivially preserved;
    pt5 is derived from Tt5 so the isentropic leg holds for any delta_Tt). Dropping
    the (1 + f) factor, for instance, gives Tt5=1233.7 K (~0.5%), which 1239.7
    catches but the in-component asserts do not.
    """
    engine = Engine(GAS, components=[])
    state0, _ = engine.freestream(FLIGHT, mdot=MDOT)
    state2 = Inlet().apply(state0, GAS)
    state3 = Compressor(PI_C).apply(state2, GAS)
    state4 = Burner(TT4).apply(state3, GAS)

    # The engine owns this coupling (it holds Tt2, Tt3, f); compute it explicitly
    # here so the per-station test exercises the same delta_Tt Engine.run will pass.
    delta_Tt = (state3.Tt - state2.Tt) / (1.0 + state4.far)
    state5 = Turbine().apply(state4, GAS, delta_Tt)

    # Spec table values -- the PRIMARY guard (the only check that fails on a wrong
    # delta_Tt formula, e.g. a missing (1 + f)).
    assert _close(state5.Tt, 1239.7), f"Tt5: got {state5.Tt}"
    assert _close(state5.pt / 1000.0, 411.5), f"pt5: got {state5.pt / 1000.0}"
    assert state5.mdot == state4.mdot and state5.far == state4.far

    # Shaft balance, both sides. This is a CROSS-CHECK, not physics validation:
    # both sides trace back to the same delta_Tt, so it only confirms the turbine
    # APPLIED delta_Tt faithfully (didn't mangle it) -- the 1239.7 above is what
    # actually validates the (Tt3-Tt2)/(1+f) formula.
    compressor_work = state3.Tt - state2.Tt
    turbine_work = (1.0 + state5.far) * (state4.Tt - state5.Tt)
    assert abs(turbine_work - compressor_work) < 1e-9, (
        f"shaft does not close: turbine {turbine_work} != compressor {compressor_work}"
    )


# --- TEMPLATE for the next station (uncomment + fill in as you derive it) ---
#
# ... nozzle (M9=2.033, T9=678.8 K, V9=1061.6 m/s).


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
