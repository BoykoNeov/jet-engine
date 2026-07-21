"""Rung 35 — FUEL is the control; Tt4 is an OUTPUT (the fuel-metering picture).

Rung 34 commanded Tt4(t) by fiat. A real engine meters FUEL, and the turbine-inlet temperature
Tt4 falls out of the burner balance against the airflow the spool can *currently* pump. At a
frozen spool a fuel step drives the airflow DOWN (the NGV passes less corrected mass as Tt4 rises,
and (1+f) rises), so f = mdot_fuel/mdot_air SPIKES and Tt4 OVERSHOOTS its steady endpoint before N
catches up. That over-temperature is a SECOND acceleration limit (turbine life) that commanding
Tt4 structurally hides — and it AMPLIFIES the airflow deficit, so it also enlarges rung 34's surge
excursion. The two acceleration limits are COUPLED, not independent.

Gates (named in docs/rung35-spec.md § Verification gates):

  1. REDUCE — CONTROL-INVARIANCE (non-tautological). Commanding the fuel mdot_fuel = f_eq*mdot_air_eq
     of a Tt4-control running-line point reproduces that SAME point (nu, pi_c, tau_t, mdot_air) and
     returns Tt4_out == Tt4 — via the forward-burner + fuel closure, a genuinely DIFFERENT code path
     than the pinned-Tt4 closure. Two closures, one operating point (machine-zero at design).
  2. REDUCE — Tt4-CONTROL UNTOUCHED + CYCLE. The Tt4-control equilibrium still reduces to rung 32's
     MapMatcher (so rung 34 is bit-for-bit); the default design run is bit-for-bit rung 6.
  3. THE FINDING — fuel control ENLARGES the surge excursion (E_surge_fuel > E_Tt4), gap MAX at r->0
     and VANISHING as r->inf; SHAPE-ROBUST in sign across >=3 surge maps. AND the NEW axis: the TIT
     overshoot E_temp > 0, monotone-decreasing in r, its r->0 limit the ALGEBRAIC map property.
  4. INSTANT-LEVEL INVERSE — the forward burner Tt4(f) is the EXACT inverse of the burner f-solve
     (the fuel<->Tt4 analogue of rung 34's forward/backward map-inverse gate 6), and the fuel closure
     at a Tt4-point's fuel recovers that point at fixed nu.

Runtime: the finding gate integrates (ds=0.1, s_settle small) like rung 34's; the rest is
algebraic/equilibria only. The finding + gates run on the FAST thermally_perfect gas
(gas-independent dynamics, matching rungs 32-34); reacting-gas fuel control is deferred (the reduce
to rung 34 on the reacting gas is the Tt4-control path, untouched).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, MapMatcher, ComponentMap, SpoolTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)

SURGE_SHAPES = [ComponentMap.surge_flow(), ComponentMap.surge_pressure(), ComponentMap.surge_tilted()]


def _fast_transient(comp_map=None):
    """A SpoolTransient (+ a MapMatcher) on the FAST (thermally_perfect) gas."""
    gas = Gas.thermally_perfect()
    st = SpoolTransient(build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
                        FLIGHT, 1.0, comp_map=comp_map)
    mm = MapMatcher(build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
                    FLIGHT, 1.0)
    return st, mm


# --------------------------------------------------------------------------- gate 1
def test_reduce_control_invariance():
    """GATE 1 — the NON-TAUTOLOGICAL reduce. A steady point does not care whether it is named by
    its Tt4 or by its fuel flow. Command the fuel mdot_fuel = f_eq*mdot_air_eq of a Tt4-control
    running-line point; the fuel-control equilibrium must return the SAME (nu, pi_c, tau_t,
    mdot_air) and Tt4_out == Tt4 — through the forward-burner + fuel closure, which NEVER pins Tt4.
    Two genuinely different closures onto one operating point.
    """
    st, _ = _fast_transient(ComponentMap.surge_tilted())
    shape = ComponentMap.surge_tilted()
    for Tt4 in (1500.0, 1300.0, 1100.0):
        eqT = st.equilibrium(FLIGHT, Tt4, shape)
        mdot_fuel = eqT["f"] * eqT["mdot_air"]
        eqF = st.equilibrium_fuel(FLIGHT, mdot_fuel, shape)
        # Machine-zero at design; tight on the sweep (nested root-finds at 1e-11/1e-12).
        tol = 1e-9 if Tt4 == TT4 else 1e-6
        assert abs(eqF["nu"] - eqT["nu"]) < tol * eqT["nu"], (
            f"control-invariance nu at Tt4={Tt4}: {eqF['nu']} vs {eqT['nu']}")
        assert abs(eqF["pi_c"] - eqT["pi_c"]) < tol * eqT["pi_c"], "control-invariance pi_c"
        assert abs(eqF["tau_t"] - eqT["tau_t"]) < tol * eqT["tau_t"], "control-invariance tau_t"
        assert abs(eqF["mdot_air"] - eqT["mdot_air"]) < tol * eqT["mdot_air"], "control-invariance mdot"
        # Tt4 comes BACK OUT of the fuel closure (it was an output) at the commanded value.
        assert abs(eqF["Tt4"] - Tt4) < 1e-5, f"Tt4 must fall back out of the fuel closure: {eqF['Tt4']}"
    # At design the invariance is machine-zero (nu==1, pi_c==pi_c_design).
    eqF_d = st.equilibrium_fuel(FLIGHT, st._fuel_for_Tt4(FLIGHT, TT4, shape), shape)
    assert abs(eqF_d["nu"] - 1.0) < 1e-9 and abs(eqF_d["pi_c"] - PI_C) < 1e-7, (
        f"design fuel-control point must be nu=1, pi_c=10: nu={eqF_d['nu']}, pi_c={eqF_d['pi_c']}")


# --------------------------------------------------------------------------- gate 2
def test_reduce_Tt4_control_untouched_and_cycle():
    """GATE 2 — the Tt4-control path is UNTOUCHED (so rung 34 is bit-for-bit), and the cycle is
    bit-for-bit rung 6. The rung-35 fuel methods are a separate entry point; adding them must not
    move any steady number. Witness: the Tt4-control equilibrium still reduces to rung 32.
    """
    st, mm = _fast_transient(ComponentMap.surge_tilted())
    shape = ComponentMap.surge_tilted()
    for Tt4 in (1400.0, 1100.0):
        eq = st.equilibrium(FLIGHT, Tt4, shape)
        res = mm.match(FLIGHT, Tt4, shape)
        assert abs(eq["pi_c"] - res.pi_c) < 1e-6 * res.pi_c, (
            f"Tt4-control equilibrium must still == rung-32 MapMatcher at Tt4={Tt4}")
        assert abs(eq["nu"] - res.N_ratio) < 1e-6, "Tt4-control nu must still == rung-32 N_ratio"

    # Cycle bit-for-bit rung 6: constructing a SpoolTransient does not perturb the design run.
    gas = Gas.thermally_perfect()
    eng = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    before = eng.run(FLIGHT, 1.0).performance.specific_thrust
    _ = SpoolTransient(eng, FLIGHT, 1.0, comp_map=ComponentMap.surge_flow())
    after = eng.run(FLIGHT, 1.0).performance.specific_thrust
    assert abs(after - before) < 1e-12, "building a SpoolTransient must not perturb the design run"


# --------------------------------------------------------------------------- gate 3
def test_the_finding_fuel_enlarges_surge_and_the_TIT_overshoot():
    """GATE 3 — THE RUNG. Two claims on the SAME fuel trajectory:

    (a) THE CORRECTION — fuel control ENLARGES the surge excursion: E_surge_fuel > E_Tt4 at matched
        r, gap MAX at r->0 and VANISHING as r->inf, SHAPE-ROBUST in sign across >=3 surge maps.
        Rung 34 under-counted surge because commanding Tt4 suppressed the over-temperature that
        amplifies the airflow deficit — the surge and TIT limits are COUPLED.
    (b) THE NEW AXIS — the turbine-inlet-temperature overshoot E_temp > 0 (Tt4 floats above its
        steady endpoint), monotone-decreasing in r, its r->0 limit the ALGEBRAIC map property.
    """
    LO, HI = 1250.0, 1450.0

    # (a1) SHAPE-ROBUST sign at r->0 (algebraic, no integration) across the surge maps.
    for shape in SURGE_SHAPES:
        st, _ = _fast_transient(shape)
        E0_T = st.constant_speed_excursion(FLIGHT, LO, HI, shape)             # rung 34, Tt4 control
        cs = st.constant_speed_excursion_fuel(FLIGHT, LO, HI, shape)          # rung 35, fuel control
        assert E0_T > 0.0, f"the Tt4-control accel excursion must be positive: {E0_T} ({shape})"
        assert cs["E_surge0"] > E0_T + 1e-4, (
            f"fuel control must ENLARGE the surge excursion at r->0: {cs['E_surge0']} vs {E0_T} ({shape})")
        assert cs["E_temp0"] > 0.05, (
            f"the TIT overshoot must be a meaningful positive number: {cs['E_temp0']} ({shape})")

    # (a2)+(b) INTEGRATED: gap persists at finite r and SHRINKS toward r->inf; both axes monotone.
    st, _ = _fast_transient(ComponentMap.surge_flow())
    shape = ComponentMap.surge_flow()
    E0_T = st.constant_speed_excursion(FLIGHT, LO, HI, shape)
    cs = st.constant_speed_excursion_fuel(FLIGHT, LO, HI, shape)

    fast = st.ramp_excursion_fuel(FLIGHT, LO, HI, 0.3, shape, s_settle=4.0, ds=0.1)
    slow = st.ramp_excursion_fuel(FLIGHT, LO, HI, 3.0, shape, s_settle=4.0, ds=0.1)
    eT_fast = st.ramp_excursion(FLIGHT, LO, HI, 0.3, shape, s_settle=4.0, ds=0.1)["E"]
    eT_slow = st.ramp_excursion(FLIGHT, LO, HI, 3.0, shape, s_settle=4.0, ds=0.1)["E"]

    # The integrated excursions are bounded by their r->0 algebraic limits (the largest possible).
    assert fast["E_surge"] <= cs["E_surge0"] + 1e-6, "E_surge(r) must not exceed the r->0 limit"
    assert fast["E_temp"] <= cs["E_temp0"] + 1e-6, "E_temp(r) must not exceed the r->0 limit"
    # Both axes monotone-decreasing in r.
    assert fast["E_surge"] > slow["E_surge"], f"E_surge must fall with r: {fast['E_surge']} !> {slow['E_surge']}"
    assert fast["E_temp"] > slow["E_temp"], f"E_temp must fall with r: {fast['E_temp']} !> {slow['E_temp']}"
    # THE CORRECTION at finite r: fuel control still enlarges the surge excursion...
    assert fast["E_surge"] > eT_fast + 1e-4, (
        f"fuel control must enlarge the surge excursion at r=0.3: {fast['E_surge']} vs {eT_fast}")
    # ...and the gap VANISHES toward r->inf (both control modes track the running line quasi-statically).
    gap_fast = fast["E_surge"] - eT_fast
    gap_slow = slow["E_surge"] - eT_slow
    assert gap_slow < gap_fast, f"the surge-excursion gap must shrink with r: {gap_slow} !< {gap_fast}"
    assert gap_slow < 0.4 * gap_fast, f"the gap must nearly close at r=3: {gap_slow} vs {gap_fast}"


# --------------------------------------------------------------------------- gate 4
def test_forward_burner_inverse_and_fuel_closure_recovers_point():
    """GATE 4 — the INSTANT-LEVEL inverse (the fuel<->Tt4 analogue of rung 34's map-inverse gate 6).

    (a) The forward burner Tt4(f) is the EXACT inverse of the shipped burner f-solve: solving f from
        Tt4(f) recovers f to machine zero.
    (b) At a fixed shaft speed, closing the compressor with the FUEL of a Tt4-control instant recovers
        that instant's (Tt4, pi_c, mdot_air) — the two closures agree off the running line too.
    """
    st, _ = _fast_transient(ComponentMap.surge_tilted())
    shape = ComponentMap.surge_tilted()

    # (a) forward burner exact inverse of _solve_f (both on the non-equilibrium gas).
    for Tt3, f in ((650.0, 0.020), (700.0, 0.025), (600.0, 0.030)):
        Tt4 = st._tt4_from_f(Tt3, f)
        pt4 = 1.0e6                      # pt4 is inert for the non-equilibrium burner f-solve
        f_back = st._solve_f(Tt3, pt4, Tt4)
        assert abs(f_back - f) < 1e-10, f"Tt4(f) must invert the burner f-solve: {f_back} vs {f}"

    # (b) off-running-line agreement: take a Tt4-control INSTANT at a perturbed nu, read its fuel,
    # and close the compressor in fuel-control at the SAME nu -> same Tt4, pi_c, mdot_air.
    nu = 0.92
    inst = st._instant(FLIGHT, nu, 1350.0, shape)
    mdot_fuel = inst["f"] * inst["mdot_air"]
    instF = st._instant_fuel(FLIGHT, nu, mdot_fuel, shape)
    assert abs(instF["Tt4"] - 1350.0) < 1e-6, f"fuel closure Tt4 off-line: {instF['Tt4']}"
    assert abs(instF["pi_c"] - inst["pi_c"]) < 1e-8 * inst["pi_c"], "fuel closure pi_c off-line"
    assert abs(instF["mdot_air"] - inst["mdot_air"]) < 1e-8 * inst["mdot_air"], "fuel closure mdot off-line"


if __name__ == "__main__":
    test_reduce_control_invariance()
    test_reduce_Tt4_control_untouched_and_cycle()
    test_the_finding_fuel_enlarges_surge_and_the_TIT_overshoot()
    test_forward_burner_inverse_and_fuel_closure_recovers_point()
    print("rung 35: all gates pass")
