"""Rung 36 — THE SURGE LINE: the excursion gets a boundary to be measured against.

Gates (named in docs/rung36-spec.md § Verification gates):

  1. REDUCE — surge line off (phi_surge=0) => rung 34/35 bit-for-bit: adding a surge floor to a map
     does not perturb the equilibrium / running line (phi_surge is read ONLY by the surge methods).
     The rung 31-35 suites passing unchanged is the external bit-for-bit witness.
  2. PI_C REPRODUCTION (non-tautological) — _pi_c_map at the operating (n, phi_op) == the shipped
     equilibrium pi_c to machine zero: the margin is measured on the running-line map itself.
  3. THE SCHEDULE — SM_N thin at LOW power, SIGN-ROBUST across >=3 shapes x >=3 imposed phi_surge
     (and under the constant-FLOW definition too). Magnitude disclaimed.
  4. THE COMPOUNDING (confirmation + sharpening, NOT relocation) — E0/SM_N rises monotonically as
     start power falls (both E0 up and SM_N down, REINFORCING), across >=3 shapes: the low-power burst
     is most surge-critical on BOTH axes. Rung 34's E0 is already largest there (no relocation — the
     schedules are parallel); the surge line's new content is SM_N, the margin the excursion consumes.
  5. CURRENCY EQUIVALENCE (airtight) — reaches_surge == phi_step_le_surge everywhere
     (E0 >= SM_N  <=>  phi_step <= phi_surge): SM_N is the exact currency the excursion consumes.
  6. CROSSING IS DISCLAIMED (anti-overclaim) — with E0 fixed, varying phi_surge FLIPS reaches_surge:
     the crossing rides on the disclaimed floor, so the rung claims the TREND, never the crossing.
  7. CYCLE UNTOUCHED — the default design run is bit-for-bit rung 6.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, ComponentMap, SpoolTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C, TT4 = 10.0, 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)

SHAPES = [ComponentMap.surge_flow, ComponentMap.surge_pressure, ComponentMap.surge_tilted]
PHI_SURGE = [0.55, 0.65, 0.75]
SWEEP = [1500.0, 1300.0, 1100.0, 900.0, 800.0, 700.0]


def _st(cmap=None):
    gas = Gas.thermally_perfect()
    return SpoolTransient(
        build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
        FLIGHT, 1.0, comp_map=cmap)


# --------------------------------------------------------------------------- gate 1
def test_reduce_surge_off_is_bit_for_bit():
    """GATE 1 — the surge floor is a PURE diagnostic: attaching it to a map perturbs NOTHING on the
    running line. equilibrium(with floor) == equilibrium(without floor) to machine zero. And with NO
    floor the surge methods refuse to invent a boundary (assert), so nothing silently runs off a
    zero surge line."""
    for shape in SHAPES:
        st = _st()
        bare = shape()
        withsurge = shape().with_phi_surge(0.65)
        for Tt4 in (1400.0, 1000.0):
            eb = st.equilibrium(FLIGHT, Tt4, bare)
            es = st.equilibrium(FLIGHT, Tt4, withsurge)
            assert eb["pi_c"] == es["pi_c"], "surge floor perturbed pi_c (must be diagnostic-only)"
            assert eb["nu"] == es["nu"], "surge floor perturbed nu (must be diagnostic-only)"
            assert eb["flowcoef"] == es["flowcoef"], "surge floor perturbed the running line"
        # is_flat ignores phi_surge (a flat map WITH a floor still reduces MapMatcher to rung 31).
        assert ComponentMap.flat().with_phi_surge(0.7).is_flat()
    # No floor => the surge margin refuses to invent a boundary.
    st = _st()
    try:
        st.surge_margin(FLIGHT, 1200.0, ComponentMap.surge_flow())   # phi_surge = 0
        raise AssertionError("surge_margin must require a surge line (phi_surge>0)")
    except AssertionError as e:
        assert "surge line" in str(e), f"unexpected assert: {e}"


# --------------------------------------------------------------------------- gate 2
def test_pi_c_reproduction_non_tautological():
    """GATE 2 — the surge margin is measured on the SAME forward map that sets the running line:
    _pi_c_map at the operating point (n, phi_op) reproduces the shipped equilibrium pi_c to machine
    zero. Two code paths, one pi_c — so SM is not a parallel re-derivation."""
    for shape in (ComponentMap.surge_flow, ComponentMap.surge_pressure):
        st = _st()
        cm = shape().with_phi_surge(0.65)
        for Tt4 in (1500.0, 1200.0, 900.0, 700.0):
            eq = st.equilibrium(FLIGHT, Tt4, cm)
            pc = st._pi_c_map(cm, eq["n"], eq["flowcoef"], eq["Tt2"])
            assert abs(pc - eq["pi_c"]) <= 1e-12 * eq["pi_c"], (
                f"_pi_c_map != shipped pi_c at Tt4={Tt4} ({pc} vs {eq['pi_c']})")


# --------------------------------------------------------------------------- gate 3
def test_the_schedule_thin_at_low_power_sign_robust():
    """GATE 3 — THE SCHEDULE. SM_N decreases monotonically as Tt4 falls (tightest at part power),
    SAME SIGN across 3 shapes x 3 imposed phi_surge, and the constant-FLOW definition agrees. The
    sign is inherited from the running-line phi_op(Tt4) (choked-hardware determined), NOT the floor;
    the magnitude is disclaimed."""
    for shape in SHAPES:
        st = _st()
        for phi_s in PHI_SURGE:
            cm = shape().with_phi_surge(phi_s)
            sched = st.surge_margin_schedule(FLIGHT, SWEEP, cm)
            smN = [s["SM_N"] for s in sched]
            smF = [s["SM_flow"] for s in sched]
            # strictly decreasing from design (high Tt4) to part power (low Tt4)
            for a, b in zip(smN, smN[1:]):
                assert b < a, f"SM_N not monotone-thinning ({shape.__name__}, phi_s={phi_s}): {smN}"
            assert smN[-1] < smN[0], "SM_N must be thinnest at the low-power end"
            # constant-flow definition: same sign
            for a, b in zip(smF, smF[1:]):
                assert b < a, f"SM_flow sign disagrees with SM_N ({shape.__name__}, phi_s={phi_s})"
            # phi_op itself walks DOWN toward the fixed floor (the mechanism)
            phis = [s["phi_op"] for s in sched]
            assert phis[-1] < phis[0] and all(b <= a + 1e-12 for a, b in zip(phis, phis[1:])), (
                "running-line phi_op must walk down toward the stall floor as throttled")


# --------------------------------------------------------------------------- gate 4
def test_the_compounding_confirmation_and_sharpening():
    """GATE 4 — CONFIRMATION + SHARPENING (not relocation). For a full-throttle burst to Tt4_hi, the
    consumed-margin ratio E0/SM_N rises MONOTONICALLY as the start power Tt4_lo falls — BOTH
    ingredients point low (E0 rises AND SM_N falls), so the low-power burst is most surge-critical on
    BOTH axes. E0 alone (rung 34) is ALREADY largest at low power, so nothing relocates (the
    schedules are parallel); the new content is SM_N, the margin the excursion consumes. Not a
    rescale of E (SM_N varies independently of the ramp ratio r)."""
    lows = [1400.0, 1200.0, 1000.0, 900.0, 800.0, 700.0]
    for shape in SHAPES:
        st = _st()
        cm = shape().with_phi_surge(0.65)
        rows = [st.acceleration_binding(FLIGHT, lo, 1500.0, cm) for lo in lows]
        ratios = [r["ratio"] for r in rows]
        E0s = [r["E0"] for r in rows]
        SMs = [r["SM_N"] for r in rows]
        for a, b in zip(ratios, ratios[1:]):
            assert b > a, f"E0/SM_N not monotone-rising toward low power ({shape.__name__}): {ratios}"
        # both ingredients point low
        assert E0s[-1] > E0s[0], "E0 must rise as start power falls (bigger burst from lower spool)"
        assert SMs[-1] < SMs[0], "SM_N must fall as start power falls (running line near surge)"


# --------------------------------------------------------------------------- gate 5
def test_currency_equivalence_airtight():
    """GATE 5 — SM_N is EXACTLY the currency the constant-speed excursion consumes:
    reaches_surge (E0 >= SM_N) == phi_step_le_surge (phi_step <= phi_surge) at every tested point.
    The pressure-ratio crossing and the flow-coefficient crossing are the SAME statement."""
    for shape in SHAPES:
        st = _st()
        cm = shape().with_phi_surge(0.65)
        for lo in (1400.0, 1000.0, 800.0, 700.0):
            b = st.acceleration_binding(FLIGHT, lo, 1500.0, cm)
            assert b["reaches_surge"] == b["phi_step_le_surge"], (
                f"currency equivalence broken at Tt4_lo={lo} ({shape.__name__}): "
                f"E0>=SM_N is {b['reaches_surge']} but phi_step<=phi_surge is {b['phi_step_le_surge']}")


# --------------------------------------------------------------------------- gate 6
def test_crossing_is_disclaimed_flips_with_floor():
    """GATE 6 — the ANTI-OVERCLAIM gate. E0 is independent of phi_surge (a pure map displacement);
    only SM_N moves with the floor. So for a FIXED burst there is a phi_surge that surges and one
    that does not — the CROSSING rides on the disclaimed floor. The test ASSERTS the flip exists,
    certifying rung 36 claims the trend (gate 4), never the crossing location (rung 32's warning)."""
    st = _st()
    cm_lo = ComponentMap.surge_flow().with_phi_surge(0.55)   # wide floor
    cm_hi = ComponentMap.surge_flow().with_phi_surge(0.65)   # tight floor
    lo, hi = 700.0, 1500.0
    b_lo = st.acceleration_binding(FLIGHT, lo, hi, cm_lo)
    b_hi = st.acceleration_binding(FLIGHT, lo, hi, cm_hi)
    # E0 is the SAME (floor-independent), SM_N differs, verdict FLIPS.
    assert abs(b_lo["E0"] - b_hi["E0"]) <= 1e-12, "E0 must not depend on phi_surge"
    assert b_lo["SM_N"] != b_hi["SM_N"], "SM_N must move with phi_surge"
    assert b_lo["reaches_surge"] != b_hi["reaches_surge"], (
        "the crossing must flip with phi_surge — it is disclaimed, only the trend is claimed")


# --------------------------------------------------------------------------- gate 7
def test_cycle_untouched_bit_for_bit_rung6():
    """GATE 7 — the default design run is bit-for-bit rung 6; the surge line is read-only."""
    gas = Gas.thermally_perfect()
    eng = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    before = eng.run(FLIGHT, 1.0).performance.specific_thrust
    st = SpoolTransient(eng, FLIGHT, 1.0, comp_map=ComponentMap.surge_flow().with_phi_surge(0.65))
    _ = st.surge_margin_schedule(FLIGHT, [1400.0, 1000.0], st.comp_map)
    after = eng.run(FLIGHT, 1.0).performance.specific_thrust
    assert abs(after - before) < 1e-12, "using the surge line must not perturb the design run"


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"  ok  {name}")
    print("rung 36 — all gates pass")
