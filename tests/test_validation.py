"""Validation against the spec's expected-output table (SPEC.md § Validation case).

This test is written BEFORE the physics (TDD-red): with the components still
raising NotImplementedError it MUST fail. Once the stations are derived and
implemented it should pass to ~0.1%. It only transcribes the expected values the
spec already provides — it does not derive them.

Run with `python tests/test_validation.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import Gas  # noqa: E402

# --- Inputs (SPEC.md § Validation case) ---
# Gas() defaults ARE the rung-1 cold-air-standard gas (hot section == cold,
# gamma=1.4, cp=1004, R=287), so this case doubles as the rung-2 reduce-to-ideal
# gate (docs/rung2-spec.md § Verification gates).
GAS = Gas()
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
MDOT = 1.0  # specific quantities are per kg/s of air, so mdot is a free scale

REL_TOL = 1e-3  # "~0.1%"

# --- Expected outputs (SPEC.md § Validation case) ---
EXPECTED = {
    "Tt2_K": 286.1,
    "pt2_kPa": 80.19,
    "Tt3_K": 552.4,
    "pt3_kPa": 801.9,
    "far": 0.02304,
    "Tt5_K": 1239.7,
    "pt5_kPa": 411.5,
    "M9": 2.033,
    "T9_K": 678.8,
    "V9_ms": 1061.6,
    "V0_ms": 269.4,
    "specific_thrust": 816.6,
    "tsfc": 2.821e-5,
    "eta_brayton": 0.4821,   # rung-1 "eta_th" is the Brayton identity 1 - Tt2/Tt3
    "eta_propulsive": 0.4073,
    "eta_overall": 0.2231,
}


def _close(actual, expected, rel=REL_TOL):
    return abs(actual - expected) <= rel * abs(expected)


def test_validation_case():
    engine = build_turbojet(GAS, pi_c=PI_C, Tt4=TT4, p_ambient=FLIGHT.p0)
    result = engine.run(FLIGHT, mdot=MDOT)
    st = result.stations
    perf = result.performance

    actual = {
        "Tt2_K": st["2"].Tt,
        "pt2_kPa": st["2"].pt / 1000.0,
        "Tt3_K": st["3"].Tt,
        "pt3_kPa": st["3"].pt / 1000.0,
        "far": st["4"].far,
        "Tt5_K": st["5"].Tt,
        "pt5_kPa": st["5"].pt / 1000.0,
        "M9": result.M9,
        "T9_K": result.T9,
        "V9_ms": result.V9,
        "V0_ms": result.V0,
        "specific_thrust": perf.specific_thrust,
        "tsfc": perf.tsfc,
        "eta_brayton": perf.eta_brayton,
        "eta_propulsive": perf.eta_propulsive,
        "eta_overall": perf.eta_overall,
    }
    for key, got in actual.items():
        assert _close(got, EXPECTED[key]), f"{key}: got {got!r}, expected ~{EXPECTED[key]!r}"


def test_primary_hand_check():
    """Thermal efficiency two ways must agree (SPEC.md § primary hand-check).

    eta_th = 1 - Tt2/Tt3 must equal the closed-form 1 - 1/pi_c^g. If these
    disagree the compression leg is buggy — fix it before trusting anything else.
    Both must equal 0.4821.
    """
    engine = build_turbojet(GAS, pi_c=PI_C, Tt4=TT4, p_ambient=FLIGHT.p0)
    result = engine.run(FLIGHT, mdot=MDOT)
    st = result.stations

    eta_from_states = 1.0 - st["2"].Tt / st["3"].Tt
    eta_closed_form = 1.0 - 1.0 / (PI_C ** GAS.g_c)
    assert _close(eta_from_states, eta_closed_form), (
        f"compression-leg bug: {eta_from_states} != {eta_closed_form}"
    )
    assert _close(eta_from_states, 0.4821)


def _run_all():
    """Dependency-free runner so `python tests/test_validation.py` works."""
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
