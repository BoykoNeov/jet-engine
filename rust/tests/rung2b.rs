//! RUNG 2b — polytropic efficiency as a first-class knob.
//!
//! Ported from `tests/test_polytropic.py`. Five gates, in priority order
//! (`docs/rung2b-polytropic.md` § Verification gates):
//!
//! 1. REDUCE-TO-IDEAL — `e_c = e_t = 1` collapses the polytropic path onto the rung-1 table to
//!    the digit (the polytropic exponent `gc/1 == gc`, etc.).
//! 2. EQUIVALENCE (THE STRONGEST GATE) — a polytropic engine at `e_c = e_t = 0.9` and an
//!    isentropic engine at the CONVERTED `eta_c`, `eta_t` are algebraically identical, not
//!    merely close: every station Tt/pt and every performance number agrees to ~1e-9. Run on
//!    the full Mattingly dual-gas, lossy, under-expanded case, so it doubles as "the polytropic
//!    anchor matches the isentropic anchor to machine precision".
//! 3. CROSS-CHECK — exercised on every run INSIDE the components (implied eta == closed-form
//!    conversion); here it rides along in gates 1, 2 and 4.
//! 4. POLYTROPIC-NATIVE EXTERNAL ANCHOR — Mattingly Example 7.1 with `e_c = e_t = 0.9` fed
//!    DIRECTLY: no conversion, no provisional pass.
//! 5. ASYMMETRY / DIRECTIONAL — `eta_c < e < eta_t` at every `pi_c > 1`, both gaps growing with
//!    `pi_c` and vanishing as `pi_c -> 1` (the reheat/preheat lesson).

use turbojet::components::{ram_recovery, Compressor, Turbine};
use turbojet::engine::{build_turbojet, EngineResult, FlightCondition, Losses};
use turbojet::gas::{powp, Gas, GasSpec};

fn close(actual: f64, expected: f64) -> bool { close_rel(actual, expected, 1e-3) }

fn close_rel(actual: f64, expected: f64, rel: f64) -> bool {
    (actual - expected).abs() <= rel * expected.abs()
}

// The Mattingly Example 7.1 case, reused by the equivalence, anchor and asymmetry gates. The
// book inputs POLYTROPIC e_c = e_t = 0.9.
const MATT_PI_C: f64 = 10.0;
const MATT_TT4: f64 = 1800.0;

fn matt_gas() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: 285.9,
        hpr: 42.8e6,
        ..GasSpec::default()
    })
}

fn matt_flight() -> FlightCondition { FlightCondition::new(216.7, 50_000.0, 2.0) }

fn matt_common() -> Losses {
    Losses {
        pi_d: 0.95 * ram_recovery(2.0), eta_b: 0.98, pi_b: 0.94, eta_m: 0.99, pi_n: 0.96,
        p_exit: Some(50_000.0 / 0.5),           // book P0/P9 = 0.5 -> under-expanded
        ..Losses::default()
    }
}

/// The Mattingly case run with the POLYTROPIC knob fed directly (e = 0.9).
fn matt_polytropic() -> EngineResult {
    build_turbojet(matt_gas(), MATT_PI_C, MATT_TT4, matt_flight().p0,
                   Losses { e_c: Some(0.9), e_t: Some(0.9), ..matt_common() })
        .run(&matt_flight(), 1.0)
}

// --- Gate 1: e = 1 reduces to the rung-1 ideal table ---------------------------------------

/// `e_c = e_t = 1` (everything else ideal) reproduces the rung-1 table.
///
/// The polytropic exponent `gc/e_c` collapses to `gc` at `e_c = 1`, and the turbine's
/// `(Tt5/Tt4)^(1/(e_t*gt))` collapses to the isentropic form at `e_t = 1` — so the
/// reduce-to-ideal gate is structurally untouched by the new knob.
#[test]
fn polytropic_reduces_to_ideal() {
    let result = build_turbojet(Gas::default(), 10.0, 1500.0, 50_000.0,
                                Losses { e_c: Some(1.0), e_t: Some(1.0), ..Losses::default() })
        .run(&FlightCondition::new(250.0, 50_000.0, 0.85), 1.0);
    let perf = &result.performance;

    // Rung-1 expected table (SPEC.md § Validation case).
    assert!(close(result.station("3").tt, 552.4));
    assert!(close(result.station("3").pt / 1000.0, 801.9));
    assert!(close(result.station("4").far, 0.02304));
    assert!(close(result.station("5").tt, 1239.7));
    assert!(close(result.m9, 2.033));
    assert!(close(result.v9, 1061.6));
    assert!(close(perf.specific_thrust, 816.6));
    assert!(close(perf.tsfc, 2.821e-5));
    assert!(close(perf.eta_brayton, 0.4821));
    assert!(close(perf.eta_thermal, 0.5477));
}

// --- Gate 2: polytropic == converted-isentropic, to machine precision ----------------------

/// A polytropic engine and the CONVERTED isentropic engine are identical to ~1e-9.
///
/// Run on the full Mattingly case (dual gas, all losses, under-expanded nozzle), so this is
/// also the "polytropic anchor matches the isentropic anchor to machine precision" check. The
/// conversion is exact for a calorically perfect gas:
///
/// ```text
/// eta_c = (pi_c^gc - 1)/(pi_c^(gc/e_c) - 1)
/// eta_t = (1 - tau_t)/(1 - tau_t^(1/e_t)),  tau_t from the (knob-independent) shaft
/// ```
#[test]
fn polytropic_isentropic_equivalence() {
    let (e_c, e_t) = (0.9, 0.9);
    let poly = matt_polytropic();

    // Convert e -> eta. eta_c is closed-form in pi_c; eta_t needs tau_t, which the shaft fixes
    // independent of turbine efficiency, so read it off the polytropic run.
    let gc = matt_gas().g_c();
    let eta_c = (powp(MATT_PI_C, gc) - 1.0) / (powp(MATT_PI_C, gc / e_c) - 1.0);
    let tau_t = poly.station("5").tt / MATT_TT4;
    let eta_t = (1.0 - tau_t) / (1.0 - powp(tau_t, 1.0 / e_t));

    let iso = build_turbojet(matt_gas(), MATT_PI_C, MATT_TT4, matt_flight().p0,
                             Losses { eta_c, eta_t, ..matt_common() })
        .run(&matt_flight(), 1.0);

    // Every station total agrees to ~1e-9 relative — an algebraic identity, not closeness.
    for (label, sp) in &poly.stations {
        let si = iso.station(label);
        assert!(close_rel(sp.tt, si.tt, 1e-9), "Tt[{label}] poly {} != iso {}", sp.tt, si.tt);
        assert!(close_rel(sp.pt, si.pt, 1e-9), "pt[{label}] poly {} != iso {}", sp.pt, si.pt);
        if si.far != 0.0 {
            assert!(close_rel(sp.far, si.far, 1e-9), "far[{label}]");
        } else {
            assert_eq!(sp.far, si.far, "far[{label}]");
        }
    }

    // And every headline performance number.
    let (pp, pi) = (&poly.performance, &iso.performance);
    for (name, a, b) in [
        ("specific_thrust", pp.specific_thrust, pi.specific_thrust),
        ("tsfc", pp.tsfc, pi.tsfc),
        ("eta_brayton", pp.eta_brayton, pi.eta_brayton),
        ("eta_thermal", pp.eta_thermal, pi.eta_thermal),
        ("eta_propulsive", pp.eta_propulsive, pi.eta_propulsive),
        ("eta_overall", pp.eta_overall, pi.eta_overall),
    ] {
        assert!(close_rel(a, b, 1e-9), "{name}: poly {a} != iso {b}");
    }
    assert!(close_rel(poly.v9, iso.v9, 1e-9) && close_rel(poly.m9, iso.m9, 1e-9));
}

// --- Gate 4: polytropic-native external anchor (Mattingly 7.1) -----------------------------

/// Mattingly Example 7.1 with `e_c = e_t = 0.9` fed DIRECTLY — no provisional pass.
///
/// The contrast with `rung2.rs`'s `mattingly_example_7_1` is the point: the isentropic anchor
/// must run a provisional pass to recover `tau_t` before it can convert `e_t -> eta_t`; the
/// polytropic knob needs neither conversion nor pass. One build, one run, same book numbers.
#[test]
fn polytropic_anchor_mattingly() {
    let result = matt_polytropic();
    let perf = &result.performance;

    let tol = 5e-4;   // book rounds intermediates to 4 sig figs; actual deviation <= 0.015 %
    assert!(close_rel(result.v0, 590.0, tol), "V0: {}", result.v0);
    assert!(close_rel(result.station("4").far, 0.03567, tol), "f: {}", result.station("4").far);
    assert!(close_rel(result.m9, 2.253, tol), "M9: {}", result.m9);
    assert!(close_rel(result.t9, 833.4, tol), "T9: {}", result.t9);
    assert!(close_rel(result.v9, 1253.8, tol), "V9: {}", result.v9);
    assert!(close_rel(perf.specific_thrust, 806.9, tol), "F/mdot: {}", perf.specific_thrust);
    assert!(close_rel(perf.tsfc, 4.421e-5, tol), "TSFC: {}", perf.tsfc);
    assert!(close_rel(perf.eta_thermal, 0.4192, tol), "eta_T: {}", perf.eta_thermal);
    assert!(close_rel(perf.eta_propulsive, 0.7439, tol), "eta_P: {}", perf.eta_propulsive);
    assert!(close_rel(perf.eta_overall, 0.3118, tol), "eta_O: {}", perf.eta_overall);
}

// --- Gate 5: the asymmetry eta_c < e < eta_t, growing with pi_c ----------------------------

/// Implied isentropic `(eta_c, eta_t)` for a polytropic engine at this `pi_c`.
///
/// `eta_c` is closed-form in `pi_c`; `eta_t` comes from `tau_t`, read off a real run — the
/// shaft sets `tau_t`, so it is the same number the turbine's own cross-check uses.
fn implied_etas(pi_c: f64, e: f64) -> (f64, f64) {
    let gc = matt_gas().g_c();
    let eta_c = (powp(pi_c, gc) - 1.0) / (powp(pi_c, gc / e) - 1.0);
    let run = build_turbojet(matt_gas(), pi_c, MATT_TT4, matt_flight().p0,
                             Losses { e_c: Some(e), e_t: Some(e), ..matt_common() })
        .run(&matt_flight(), 1.0);
    let tau_t = run.station("5").tt / MATT_TT4;
    let eta_t = (1.0 - tau_t) / (1.0 - powp(tau_t, 1.0 / e));
    (eta_c, eta_t)
}

/// Same `e` for both, yet `eta_c < e < eta_t` — and both gaps grow with `pi_c`.
///
/// The reheat/preheat lesson (`docs/rung2b-polytropic.md` § The asymmetry): diverging isobars
/// make a compressor look WORSE than its per-stage efficiency and a turbine BETTER. The split
/// is set by pressure ratio — which is why `e` exists as a separate knob at all.
#[test]
fn efficiency_asymmetry() {
    let e = 0.9;
    let pis = [1.001, 2.0, 10.0, 30.0];   // 1.001 stands in for the pi_c -> 1 limit
    let (mut gaps_c, mut gaps_t) = (Vec::new(), Vec::new());
    for pi_c in pis {
        let (eta_c, eta_t) = implied_etas(pi_c, e);
        // The ordering holds at every pi_c > 1 (strict once away from the limit).
        assert!(eta_c <= e + 1e-12, "eta_c {eta_c} should be <= e at pi_c={pi_c}");
        assert!(eta_t >= e - 1e-12, "eta_t {eta_t} should be >= e at pi_c={pi_c}");
        gaps_c.push(e - eta_c);
        gaps_t.push(eta_t - e);
    }

    // Anchor point (pi_c = 10): the headline numbers from the doc.
    let (eta_c10, eta_t10) = implied_etas(10.0, e);
    assert!(close_rel(eta_c10, 0.8641, 1e-3) && close_rel(eta_t10, 0.9099, 1e-3));
    assert!(eta_c10 < e && e < eta_t10);

    // Both gaps vanish as pi_c -> 1 ...
    assert!(gaps_c[0] < 1e-3 && gaps_t[0] < 1e-3, "gaps must -> 0 as pi_c -> 1");
    // ... and grow monotonically with pi_c.
    assert!(gaps_c.windows(2).all(|w| w[0] <= w[1]),
            "compressor gap not monotonic in pi_c: {gaps_c:?}");
    assert!(gaps_t.windows(2).all(|w| w[0] <= w[1]),
            "turbine gap not monotonic in pi_c: {gaps_t:?}");
}

// --- Mutual exclusivity: the one new validation the knob needs -----------------------------
//
// Python raises ValueError and the test catches it; Rust panics and `#[should_panic]` catches
// it. Deliberately NOT `catch_unwind`: silencing the panic output needs `panic::set_hook`,
// which is PROCESS-global while cargo runs these tests as parallel threads in one binary — a
// test that installs a hook would be racing its neighbours.

/// A non-default isentropic `eta_c` alongside a polytropic `e_c` is contradictory.
#[test]
#[should_panic(expected = "set eta_c (isentropic) OR e_c (polytropic), not both")]
fn compressor_knobs_are_mutually_exclusive() {
    Compressor::new(10.0, 0.88, Some(0.9));
}

#[test]
#[should_panic(expected = "set eta_t (isentropic) OR e_t (polytropic), not both")]
fn turbine_knobs_are_mutually_exclusive() {
    Turbine::new(0.90, Some(0.9));
}

/// The same contradiction reached through the factory rather than the component.
#[test]
#[should_panic(expected = "set eta_c (isentropic) OR e_c (polytropic), not both")]
fn build_turbojet_rejects_contradictory_knobs() {
    build_turbojet(Gas::default(), 10.0, 1500.0, 50_000.0,
                   Losses { eta_c: 0.88, e_c: Some(0.9), ..Losses::default() });
}

/// But `e` with the DEFAULT eta is fine (the common case), and so is `e = 1` (ideal).
///
/// The other half of the exclusivity gate, and the load-bearing half: three `should_panic`
/// tests alone would also pass if the constructors rejected EVERYTHING.
#[test]
fn the_valid_knob_combinations_are_accepted() {
    Compressor::new(10.0, 1.0, Some(0.9));
    Turbine::new(1.0, Some(1.0));
    Compressor::new(10.0, 0.88, None);
    Turbine::new(0.90, None);
    build_turbojet(Gas::default(), 10.0, 1500.0, 50_000.0,
                   Losses { e_c: Some(0.9), ..Losses::default() });
}
