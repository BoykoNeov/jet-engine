//! RUNG 1 — the ideal Brayton cycle: per-station checks and the whole-cycle validation table.
//!
//! Ported from `tests/test_stations.py` (each station verified the moment it lands) and
//! `tests/test_validation.py` (the whole-cycle target). Both transcribe values the spec already
//! provides — `SPEC.md` § Validation case — they do not derive them.
//!
//! `Gas::default()` IS the rung-1 cold-air-standard gas (hot section == cold, gamma = 1.4,
//! cp = 1004, R = 287), so this case doubles as the rung-2 reduce-to-ideal gate.

use turbojet::components::{ram_recovery, Burner, Compressor, Inlet, Nozzle, Turbine};
use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::{FlowState, Gas};

const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;
const MDOT: f64 = 1.0;      // specific quantities are per kg/s of air, so mdot is a free scale
const REL_TOL: f64 = 1e-3;  // "~0.1 %"

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn close(actual: f64, expected: f64) -> bool { close_rel(actual, expected, REL_TOL) }

fn close_rel(actual: f64, expected: f64, rel: f64) -> bool {
    (actual - expected).abs() <= rel * expected.abs()
}

/// The 0 -> 4 chain, shared by the per-station tests below. Each station test re-walks it so
/// that it verifies the station IN PLACE rather than against a stored fixture — the workflow
/// `docs/plans/rung1-plan.md` prescribes: derive -> code -> verify, one station at a time.
fn chain_to_4(gas: &Gas) -> (FlowState, FlowState, FlowState, FlowState, f64) {
    let engine = Engine::new(gas.clone(), vec![], 1.0);   // freestream needs no components
    let (state0, v0) = engine.freestream(&flight(), MDOT);
    let state2 = Inlet::new(1.0).apply(&state0, gas);
    let state3 = Compressor::new(PI_C, 1.0, None).apply(&state2, gas);
    let state4 = Burner::new(TT4, 1.0, 1.0).apply(&state3, gas);
    (state0, state2, state3, state4, v0)
}

// ======================================================================================
// Per-station checks (test_stations.py).
// ======================================================================================

/// Station 0: Tt0 = 286.1 K, pt0 = 80.19 kPa, V0 = 269.4 m/s.
#[test]
fn station0_freestream() {
    let gas = Gas::default();
    let engine = Engine::new(gas, vec![], 1.0);
    let (state0, v0) = engine.freestream(&flight(), MDOT);

    assert!(close(state0.tt, 286.1), "Tt0: got {}", state0.tt);
    assert!(close(state0.pt / 1000.0, 80.19), "pt0: got {}", state0.pt / 1000.0);
    assert!(close(v0, 269.4), "V0: got {v0}");
    assert_eq!(state0.mdot, MDOT);
    assert_eq!(state0.far, 0.0);
}

/// Ideal inlet: `Tt2 == Tt0`, `pt2 == pt0` (286.1 K, 80.19 kPa).
#[test]
fn station2_inlet() {
    let gas = Gas::default();
    let (state0, state2, _, _, _) = chain_to_4(&gas);

    // Spec table values...
    assert!(close(state2.tt, 286.1), "Tt2: got {}", state2.tt);
    assert!(close(state2.pt / 1000.0, 80.19), "pt2: got {}", state2.pt / 1000.0);
    // ...and the defining property: an ideal inlet PRESERVES the station-0 totals.
    assert!(state2.tt == state0.tt && state2.pt == state0.pt);
    assert!(state2.mdot == state0.mdot && state2.far == state0.far);
}

/// Ideal compressor: Tt3 = 552.4 K, pt3 = 801.9 kPa, and the PRIMARY HAND-CHECK —
/// `eta_th = 1 - Tt2/Tt3` must equal the closed form `1 - 1/pi_c^g`, both 0.4821.
///
/// The spec says: if those two disagree the compression leg is buggy, fix it before trusting
/// anything else. That check also lives in the whole-cycle test below, but there it is gated
/// behind the full engine wiring — so it runs HERE the moment the compressor lands, which is
/// the whole point of the per-station suite.
#[test]
fn station3_compressor() {
    let gas = Gas::default();
    let (_, state2, state3, _, _) = chain_to_4(&gas);

    // Spec table values — the only guard that catches a wrong pi_c or exponent in ABSOLUTE
    // terms (every in-component assert is structurally exact).
    assert!(close(state3.tt, 552.4), "Tt3: got {}", state3.tt);
    assert!(close(state3.pt / 1000.0, 801.9), "pt3: got {}", state3.pt / 1000.0);
    assert!(state3.mdot == state2.mdot && state3.far == state2.far);

    // Primary hand-check, two ways. Structurally exact (Tt3 == Tt2*pi_c^g makes Tt2/Tt3 ==
    // 1/pi_c^g to machine precision), so assert it TIGHT — a failure beyond float epsilon
    // means Tt3 was not computed via the isentropic relation.
    let eta_from_states = 1.0 - state2.tt / state3.tt;
    let eta_closed_form = 1.0 - 1.0 / turbojet::gas::powp(PI_C, gas.g_c());
    assert!((eta_from_states - eta_closed_form).abs() < 1e-9,
            "compression-leg bug: {eta_from_states} != {eta_closed_form}");
    // The 0.4821 target is rounded, so it stays at the ~0.1 % spec tolerance.
    assert!(close(eta_from_states, 0.4821), "eta_th: got {eta_from_states}");
}

/// Ideal burner: f = 0.02304, pt4 == pt3, Tt4 = 1500 K, and mass grows by the fuel.
///
/// The mdot assertion is the one that exercises the mass-growth line — the others pass even if
/// that line were forgotten.
#[test]
fn station4_burner() {
    let gas = Gas::default();
    let (_, _, state3, state4, _) = chain_to_4(&gas);

    assert!(close(state4.far, 0.02304), "f: got {}", state4.far);
    assert!(close(state4.pt / 1000.0, 801.9), "pt4: got {}", state4.pt / 1000.0);
    assert_eq!(state4.tt, TT4, "Tt4: got {}", state4.tt);
    // Defining properties: an ideal burner holds pt; fuel mass joins the stream.
    assert_eq!(state4.pt, state3.pt, "ideal burner: pt4 == pt3");
    assert!(close(state4.mdot, state3.mdot * (1.0 + state4.far)), "mdot4: got {}", state4.mdot);
}

/// Ideal turbine, THE KEYSTONE: Tt5 = 1239.7 K, pt5 = 411.5 kPa.
///
/// The shaft balance is the physics under test: the turbine is handed an ENTHALPY drop
/// `delta_h = (h_c(Tt3) - h_c(Tt2))/(1 + f)` and `Tt5 = T_from_h_t(h_t(Tt4) - delta_h)`. The
/// ABSOLUTE spec values are the real guard — every in-component assert is structurally exact
/// (mass is trivially preserved; pt5 is derived from the substate so the isentropic leg holds
/// for ANY delta_h). Dropping the `(1 + f)` factor gives Tt5 = 1233.7 K (~0.5 %), which 1239.7
/// catches and the in-component asserts do not.
#[test]
fn station5_turbine() {
    let gas = Gas::default();
    let (_, state2, state3, state4, _) = chain_to_4(&gas);

    // The engine owns this coupling (it holds Tt2, Tt3, f); compute it explicitly here so the
    // per-station test exercises the same delta_h `Engine::run` will pass.
    let delta_h = (gas.h_c(state3.tt) - gas.h_c(state2.tt)) / (1.0 + state4.far);
    let state5 = Turbine::new(1.0, None).apply(&state4, &gas, delta_h);

    assert!(close(state5.tt, 1239.7), "Tt5: got {}", state5.tt);
    assert!(close(state5.pt / 1000.0, 411.5), "pt5: got {}", state5.pt / 1000.0);
    assert!(state5.mdot == state4.mdot && state5.far == state4.far);

    // Shaft balance, both sides — a GENUINE plumbing check, not a tautology. The two sides are
    // computed differently: compressor_work reads Tt3-Tt2 straight from the states, while
    // turbine_work comes from the turbine's OUTPUT (Tt5) and re-applies the (1+f) factor. So
    // this fires on a missing/wrong (1+f): dropping the /(1+f) gives Tt5 = 1233.7 ->
    // turbine_work 272.4 vs compressor_work 266.3, a 6.1 K residual. It is blind only to (a) a
    // uniformly-wrong f — the two (1+f) factors cancel, so the burner test guards f — and (b)
    // the pressure side, which the pt5 spec value guards.
    let compressor_work = state3.tt - state2.tt;
    let turbine_work = (1.0 + state5.far) * (state4.tt - state5.tt);
    assert!((turbine_work - compressor_work).abs() < 1e-9,
            "shaft does not close: turbine {turbine_work} != compressor {compressor_work}");
}

/// Ideal nozzle, fully expanded (`p9 = p0`): M9 = 2.033, T9 = 678.8 K, V9 = 1061.6 m/s. This is
/// where the cycle drops from totals to static.
///
/// The spec values are the PRIMARY guard. The in-component asserts are either
/// exact-by-construction (the static<->total isentropic leg, since M9/T9 are derived to satisfy
/// it) or loose-by-necessity (the energy split, ~0.05 % off because cp = 1004.0 disagrees with
/// gamma*R/(gamma-1) = 1004.5) — neither pins the absolute numbers, so the table does.
#[test]
fn station9_nozzle() {
    let gas = Gas::default();
    // Feed the nozzle the spec's station-5 totals directly, so this test stands alone (the
    // full 0->5 chain is already guarded by the tests above).
    let state5 = FlowState { tt: 1239.7, pt: 411_500.0, mdot: 1.0, far: 0.02304 };
    let exit = Nozzle::new(flight().p0, 1.0, None).apply(&state5, &gas);

    assert!(close(exit.m9, 2.033), "M9: got {}", exit.m9);
    assert!(close(exit.t9, 678.8), "T9: got {}", exit.t9);
    assert!(close(exit.v9, 1061.6), "V9: got {}", exit.v9);
    // Defining properties: an ideal nozzle conserves totals and moves no mass/fuel.
    assert!(exit.state.tt == state5.tt && exit.state.pt == state5.pt);
    assert!(exit.state.mdot == state5.mdot && exit.state.far == state5.far);
}

// ======================================================================================
// The whole-cycle validation table (test_validation.py).
// ======================================================================================

/// Every headline number of `SPEC.md` § Validation case, to ~0.1 %.
#[test]
fn validation_case() {
    let engine = build_turbojet(Gas::default(), PI_C, TT4, flight().p0, Losses::default());
    let result = engine.run(&flight(), MDOT);
    let perf = &result.performance;

    let expected: &[(&str, f64, f64)] = &[
        ("Tt2_K", result.station("2").tt, 286.1),
        ("pt2_kPa", result.station("2").pt / 1000.0, 80.19),
        ("Tt3_K", result.station("3").tt, 552.4),
        ("pt3_kPa", result.station("3").pt / 1000.0, 801.9),
        ("far", result.station("4").far, 0.02304),
        ("Tt5_K", result.station("5").tt, 1239.7),
        ("pt5_kPa", result.station("5").pt / 1000.0, 411.5),
        ("M9", result.m9, 2.033),
        ("T9_K", result.t9, 678.8),
        ("V9_ms", result.v9, 1061.6),
        ("V0_ms", result.v0, 269.4),
        ("specific_thrust", perf.specific_thrust, 816.6),
        ("tsfc", perf.tsfc, 2.821e-5),
        // rung 1's "eta_th" is the Brayton identity 1 - Tt2/Tt3.
        ("eta_brayton", perf.eta_brayton, 0.4821),
        ("eta_propulsive", perf.eta_propulsive, 0.4073),
        ("eta_overall", perf.eta_overall, 0.2231),
    ];
    for &(key, got, want) in expected {
        assert!(close(got, want), "{key}: got {got:?}, expected ~{want:?}");
    }
}

/// Thermal efficiency two ways must agree (`SPEC.md` § primary hand-check).
#[test]
fn primary_hand_check() {
    let gas = Gas::default();
    let g_c = gas.g_c();
    let engine = build_turbojet(gas, PI_C, TT4, flight().p0, Losses::default());
    let result = engine.run(&flight(), MDOT);

    let eta_from_states = 1.0 - result.station("2").tt / result.station("3").tt;
    let eta_closed_form = 1.0 - 1.0 / turbojet::gas::powp(PI_C, g_c);
    assert!(close(eta_from_states, eta_closed_form),
            "compression-leg bug: {eta_from_states} != {eta_closed_form}");
    assert!(close(eta_from_states, 0.4821));
}

// ======================================================================================
// `ram_recovery`, which rung 1 never leaves the eta_r = 1 branch of.
// ======================================================================================

/// The MIL-E-5008B correlation's three branches, and the one property rung 1 depends on:
/// eta_r == 1 EXACTLY at and below M0 = 1, so the reduce-to-ideal gate is untouched.
#[test]
fn ram_recovery_branches() {
    for m0 in [0.0, 0.5, 0.85, 1.0] {
        assert_eq!(ram_recovery(m0), 1.0, "subsonic recovery must be exactly 1 at M0={m0}");
    }
    // Supersonic: monotonically worse, and the rung-2 anchor value at M0 = 2.
    assert!(close_rel(ram_recovery(2.0), 0.925, 1e-3), "M0=2: {}", ram_recovery(2.0));
    let (a, b, c) = (ram_recovery(1.5), ram_recovery(3.0), ram_recovery(5.0));
    assert!(1.0 > a && a > b && b > c, "recovery must fall with Mach: {a} {b} {c}");
    // The hypersonic branch takes over above M0 = 5, and joins continuously enough that the
    // correlation does not jump — a transcription slip in either branch shows up as a step.
    let (lo, hi) = (ram_recovery(5.0), ram_recovery(5.0001));
    assert!((lo - hi).abs() < 5e-3, "the M0=5 branch join must not jump: {lo} vs {hi}");
}
