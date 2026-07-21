"""Rung 37 — THE TWO INTERNAL CLOCKS: volume-filling CONFIRMS, heat-soak CORRECTS.

Rungs 34-36 made the shaft the only dynamic element; rung 34 filed the omitted internal clocks as
one bundled concession ("no combustor volume-filling, no heat soak ... faster clocks below tau_spool,
they do not change the r framing"). Rung 37 tests both claims and they SPLIT (docs/rung37-spec.md):

  * VOLUME-FILLING (a combustor plenum, tau_fill << tau_spool) CONFIRMS: the r->0 peak surge
    excursion is unmoved (== rung-35 E0, INDEPENDENT of the fill clock). Its content is STRUCTURAL —
    the FIRST rung where compressor mass flow != NGV mass flow (the plenum stores the difference).
  * HEAT-SOAK (a metal state Tm, tau_soak ~ tau_spool) CORRECTS: a second STATE carries thermal
    memory, so E = E(r, theta0). Surge is PROTECTED (cold < hot-reslam < adiabatic; rung 34/35's
    adiabatic is the conservative WORST case); the cost is the accel-time LAG and the hot RESLAM.

Gates (docs/rung37-spec.md § Verification gates):
  1. REDUCE — both OFF => rung 35 bit-for-bit (dispatch, not re-solve; the inherited methods).
  2. PLENUM equilibrium == rung 35 (non-tautological: the back-pressure closure, a different path).
  3. PLENUM finding — r->0 peak == E0 independent of r_v; the mass-flow SPLIT is real (> 0).
  4. HEAT-SOAK equilibrium == rung 35 (transient-only: Q=0 at the fixed point).
  5. HEAT-SOAK finding — cold < hot-reslam < adiabatic, shape- and knob-robust.
  6. HEAT-SOAK — the accel-time LAG (cold slower than adiabatic; hot ~ adiabatic-fast).
  7. CYCLE UNTOUCHED — the default design run is bit-for-bit rung 6.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, ComponentMap, SpoolTransient, CombustorTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)
SURGE_SHAPES = [ComponentMap.surge_flow(), ComponentMap.surge_pressure(), ComponentMap.surge_tilted()]


def _build(cmap, **kw):
    gas = Gas.thermally_perfect()
    eng = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    return CombustorTransient(eng, FLIGHT, 1.0, comp_map=cmap, **kw)


# --------------------------------------------------------------------------- gate 1
def test_reduce_both_off_is_rung35_bit_for_bit():
    """GATE 1 — with the plenum and heat-soak OFF (defaults), a CombustorTransient IS rung 34/35:
    the inherited equilibrium_fuel / integrate_fuel never read the OFF knobs, so they equal a plain
    SpoolTransient's outputs bit-for-bit (dispatch, not re-solve)."""
    gas = Gas.thermally_perfect()
    cmap = ComponentMap.surge_flow()
    eng = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    ct = CombustorTransient(eng, FLIGHT, 1.0, comp_map=cmap)              # plenum/soak OFF
    st = SpoolTransient(eng, FLIGHT, 1.0, comp_map=cmap)
    for Tt4 in (1500.0, 1200.0, 900.0):
        mf = st._fuel_for_Tt4(FLIGHT, Tt4)
        a = ct.equilibrium_fuel(FLIGHT, mf)
        b = st.equilibrium_fuel(FLIGHT, mf)
        assert a["nu"] == b["nu"] and a["pi_c"] == b["pi_c"] and a["tau_t"] == b["tau_t"], \
            f"both-off CombustorTransient != rung 35 at Tt4={Tt4}"


# --------------------------------------------------------------------------- gate 2
def test_plenum_equilibrium_is_rung35():
    """GATE 2 — the PLENUM equilibrium (dnu/ds=0 AND dpt4/ds=0) reproduces rung 35's equilibrium_fuel
    via the BACK-PRESSURE closure (invert pi_c(m) for m) — a genuinely different code path than rung
    35's NGV-continuity root-find. Two closures, one operating point; and mdot_c = mdot_NGV holds at
    the fixed point (the decoupling closes)."""
    for cmap in SURGE_SHAPES:
        ct = _build(cmap, plenum_ratio=0.05)
        for Tt4 in (1400.0, 1100.0, 900.0):
            mf = ct._fuel_for_Tt4(FLIGHT, Tt4, cmap)
            a = ct.equilibrium_plenum(FLIGHT, mf, cmap)
            b = ct.equilibrium_fuel(FLIGHT, mf, cmap)
            assert abs(a["pi_c"] - b["pi_c"]) <= 1e-9 * b["pi_c"], f"plenum eq pi_c != rung35 at {Tt4}"
            assert abs(a["nu"] - b["nu"]) <= 1e-9, f"plenum eq nu != rung35 at {Tt4}"
            # mass balance closes at the fixed point (mdot_c + mdot_fuel == mdot_NGV):
            assert abs(a["mdot_c"] + mf - a["mdot_ngv"]) <= 1e-9 * a["mdot_ngv"], \
                f"plenum equilibrium mass balance not closed at {Tt4}"


# --------------------------------------------------------------------------- gate 3
def test_plenum_peak_is_E0_and_the_split_is_real():
    """GATE 3 — THE PLENUM FINDING. At r->0 (frozen spool) the plenum fills to its full quasi-steady
    pt4 before nu can move, so the peak surge excursion lands on rung-35's algebraic E0 to tolerance,
    INDEPENDENT of the fill clock r_v — the CONFIRMATION. The structural content is the mass-flow
    SPLIT the plenum stores: mdot_c != mdot_NGV during the fill (the first rung where they differ)."""
    for cmap in SURGE_SHAPES:
        peaks = []
        for r_v in (0.03, 0.1):
            ct = _build(cmap, plenum_ratio=r_v)
            res = ct.plenum_frozen_peak(FLIGHT, 1100.0, 1400.0, cmap)
            assert abs(res["peak_minus_E0"]) <= 1e-6, \
                f"plenum peak != E0 (peak-E0={res['peak_minus_E0']:.2e}) for {cmap}"
            assert res["split_max"] > 0.05, \
                f"the mass-flow split must be REAL (mdot_c != mdot_NGV); got {res['split_max']:.3e}"
            peaks.append(res["peak"])
        # the peak is INDEPENDENT of the fill clock (a frozen-spool map fact):
        assert abs(peaks[0] - peaks[1]) <= 1e-9, "plenum peak must not depend on r_v"


# --------------------------------------------------------------------------- gate 4
def test_heat_soak_equilibrium_is_rung35_transient_only():
    """GATE 4 — the HEAT-SOAK equilibrium reproduces rung 35 because at steady state Tm = Tt4_burner
    => Q = 0 => Tt4_turb = Tt4_burner: heat-soak NEVER moves the running line (a purely transient
    effect). The reduce is the fixed-point identity, not a knob->0 limit."""
    for cmap in SURGE_SHAPES:
        ct = _build(cmap, soak_gain=0.1, soak_ratio=3.0)
        for Tt4 in (1400.0, 1100.0):
            mf = ct._fuel_for_Tt4(FLIGHT, Tt4, cmap)
            a = ct.equilibrium_soak(FLIGHT, mf, cmap)
            b = ct.equilibrium_fuel(FLIGHT, mf, cmap)
            assert abs(a["pi_c"] - b["pi_c"]) <= 1e-9 * b["pi_c"], f"soak eq pi_c != rung35 at {Tt4}"
            assert abs(a["nu"] - b["nu"]) <= 1e-9, f"soak eq nu != rung35 at {Tt4}"


# --------------------------------------------------------------------------- gate 5
def test_heat_soak_cold_below_hot_below_adiabatic():
    """GATE 5 — THE HEAT-SOAK FINDING (the load-bearing SIGN). The peak surge excursion obeys
    cold first-accel < hot reslam < adiabatic (rung 35): the cold metal's heat sink depresses
    Tt4_turb -> colder NGV -> more airflow -> AWAY from surge (channel a wins), and a hot reslam
    (bodie) recovers most of the adiabatic worst case. Shape- AND knob-robust; magnitudes disclaimed."""
    for cmap in SURGE_SHAPES:
        for G in (0.05, 0.15):
            for r_m in (1.0, 5.0):
                ct = _build(cmap, soak_gain=G, soak_ratio=r_m)
                # E_surge peaks EARLY (near nu0), so a short march captures it (keeps the sweep fast).
                ad = ct.adiabatic_excursion(FLIGHT, 1100.0, 1400.0, cmap, s_end=6.0)["E_surge"]
                cold = ct.soak_excursion(FLIGHT, 1100.0, 1400.0, "cold", cmap, s_end=6.0)["E_surge"]
                hot = ct.soak_excursion(FLIGHT, 1100.0, 1400.0, "hot", cmap, s_end=6.0)["E_surge"]
                assert cold < hot < ad, \
                    f"ordering broken for {cmap} G={G} r_m={r_m}: cold={cold} hot={hot} adiab={ad}"


# --------------------------------------------------------------------------- gate 6
def test_heat_soak_accel_time_lag():
    """GATE 6 — the PRIMARY heat-soak effect: the cold metal steals turbine work, so a cold
    acceleration reaches its target speed LATER than the adiabatic one (the thrust-response lag),
    and the lag grows with G; a hot reslam is ~ adiabatic-fast (the metal releases heat early)."""
    cmap = ComponentMap.surge_flow()
    lags = []
    for G in (0.05, 0.15):
        ct = _build(cmap, soak_gain=G, soak_ratio=3.0)
        ad = ct.adiabatic_excursion(FLIGHT, 1100.0, 1400.0, cmap)
        cold = ct.soak_excursion(FLIGHT, 1100.0, 1400.0, "cold", cmap)
        hot = ct.soak_excursion(FLIGHT, 1100.0, 1400.0, "hot", cmap)
        # cold accel does not reach target as fast as adiabatic (t_accel None means "even slower").
        assert cold["t_accel"] is None or cold["t_accel"] > ad["t_accel"], \
            f"cold accel must lag adiabatic at G={G}"
        # hot reslam is ~ adiabatic-fast (within a step or two), NOT lagging like cold:
        assert hot["t_accel"] is not None and hot["t_accel"] <= ad["t_accel"] + 0.2, \
            f"hot reslam should be ~adiabatic-fast at G={G}"
        lags.append(cold["t_accel"] if cold["t_accel"] is not None else 1e9)
    assert lags[1] >= lags[0], "the accel lag should grow with the heat-extraction gain G"


# --------------------------------------------------------------------------- gate 7
def test_cycle_untouched_bit_for_bit_rung6():
    """GATE 7 — the default design run is bit-for-bit rung 6; both effects are read-only extras."""
    gas = Gas.thermally_perfect()
    eng = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    before = eng.run(FLIGHT, 1.0).performance.specific_thrust
    ct = CombustorTransient(eng, FLIGHT, 1.0, comp_map=ComponentMap.surge_flow(),
                            plenum_ratio=0.05, soak_gain=0.1, soak_ratio=3.0)
    _ = ct.plenum_frozen_peak(FLIGHT, 1100.0, 1400.0)
    _ = ct.soak_excursion(FLIGHT, 1100.0, 1400.0, "cold")
    after = eng.run(FLIGHT, 1.0).performance.specific_thrust
    assert abs(after - before) < 1e-12, "the combustor-dynamics diagnostics must not perturb the design run"


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"  ok  {name}")
    print("rung 37 — all gates pass")
