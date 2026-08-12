//! RUNG 2 — real components: reduce-to-ideal, directional checks, and the external anchor.
//!
//! Ported from `tests/test_rung2.py`. Three gates, in priority order
//! (`docs/rung2-spec.md` § Verification gates):
//!
//! 1. REDUCE-TO-IDEAL — collapse the dual gas with `unified()` and set every loss to ideal; the
//!    rung-1 table must come back to the digit. (`rung1.rs`'s validation case already exercises
//!    this via `Gas::default()`; here we prove the `unified()` collapse itself works from a
//!    genuinely dual gas.)
//! 2. DIRECTIONAL — turning losses ON must lower specific thrust, raise TSFC, and lower the
//!    real thermal efficiency.
//! 3. EXTERNAL ANCHOR — Mattingly *Elements of Propulsion* Example 7.1, to ~0.1-0.2 % (the book
//!    rounds its intermediates to 4 sig figs). The book inputs POLYTROPIC `e_c`, `e_t`; this
//!    gate uses ISENTROPIC `eta_c`, `eta_t`, so it converts — exact for a perfect gas.

use turbojet::components::ram_recovery;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{powp, Gas, GasSpec};

fn close(actual: f64, expected: f64) -> bool { close_rel(actual, expected, 1e-3) }

fn close_rel(actual: f64, expected: f64, rel: f64) -> bool {
    (actual - expected).abs() <= rel * expected.abs()
}

// --- Gate 1: unified() collapses a dual gas back to the rung-1 single gas -----------------

/// A genuinely dual gas, `unified()` and run fully ideal, reproduces rung 1.
///
/// The hot section is deliberately different (Mattingly's 1.3 / 1239 / 285.9); `unified()` must
/// collapse the WHOLE triple onto the cold defaults (1.4 / 1004 / 287) so the result is the
/// rung-1 gas exactly — which is what makes the rung-2 machinery reproduce the rung-1 table to
/// the digit.
#[test]
fn unify_reduces_to_rung1() {
    let dual = Gas::new(GasSpec {
        gamma_t: 1.3, cp_t: 1239.0, r_t: 285.9, ..GasSpec::default()
    });                                          // hot != cold
    let gas = dual.unified();
    assert!(gas.spec.gamma_t == gas.spec.gamma_c
            && gas.spec.cp_t == gas.spec.cp_c
            && gas.spec.r_t == gas.spec.r_c);

    let engine = build_turbojet(gas, 10.0, 1500.0, 50_000.0, Losses::default());  // all ideal
    let result = engine.run(&FlightCondition::new(250.0, 50_000.0, 0.85), 1.0);
    let perf = &result.performance;

    // Rung-1 expected table (SPEC.md § Validation case).
    assert!(close(result.station("2").tt, 286.1));
    assert!(close(result.station("3").tt, 552.4));
    assert!(close(result.station("3").pt / 1000.0, 801.9));
    assert!(close(result.station("4").far, 0.02304));
    assert!(close(result.station("5").tt, 1239.7));
    assert!(close(result.m9, 2.033));
    assert!(close(result.v9, 1061.6));
    assert!(close(perf.specific_thrust, 816.6));
    assert!(close(perf.tsfc, 2.821e-5));
    assert!(close(perf.eta_brayton, 0.4821));
    // The KE-based thermal efficiency in the ideal limit is 0.5477, NOT 0.4821 — a different
    // quantity (docs/rung2-spec.md § Performance).
    assert!(close(perf.eta_thermal, 0.5477));
}

// --- Gate 2: losses move the numbers the right way ----------------------------------------

/// `eta < 1` must lower specific thrust, raise TSFC, and lower the real thermal efficiency.
#[test]
fn losses_are_directional() {
    // A single gas, to isolate the efficiency effect from the dual-cp effect.
    let flight = FlightCondition::new(250.0, 50_000.0, 0.85);
    let ideal = build_turbojet(Gas::default(), 10.0, 1500.0, flight.p0, Losses::default())
        .run(&flight, 1.0).performance;
    let lossy = build_turbojet(Gas::default(), 10.0, 1500.0, flight.p0, Losses {
        pi_d: 0.95, eta_c: 0.88, eta_b: 0.99, pi_b: 0.95, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }).run(&flight, 1.0).performance;

    assert!(lossy.specific_thrust < ideal.specific_thrust, "losses must reduce specific thrust");
    assert!(lossy.tsfc > ideal.tsfc, "losses must raise TSFC");
    assert!(lossy.eta_thermal < ideal.eta_thermal, "losses must lower real thermal efficiency");
}

// --- Gate 3: external anchor — Mattingly Example 7.1 --------------------------------------

/// Reproduce Mattingly Example 7.1 (`docs/plans/rung2-anchor-mattingly.md`).
///
/// Inputs use the book's dual gas and component losses; the polytropic `e_c`, `e_t` are
/// converted to our isentropic `eta_c`, `eta_t`. The conversion for the turbine needs
/// `tau_t = Tt5/Tt4`, which is INDEPENDENT of `eta_t` (the shaft sets the drop without it), so
/// a provisional pass recovers `tau_t`, then the real pass runs.
#[test]
fn mattingly_example_7_1() {
    // Book gas: cold air / hot products, R = (gamma-1)/gamma * cp per section.
    let gas_spec = GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: 285.9,
        hpr: 42.8e6,
        ..GasSpec::default()
    };
    let (m0, t0) = (2.0, 216.7);
    let p0 = 50_000.0;              // absolute value is arbitrary (results are ratios)
    let p9 = p0 / 0.5;              // book P0/P9 = 0.5 -> under-expanded, P9 = 2*p0
    let flight = FlightCondition::new(t0, p0, m0);
    let (pi_c, tt4) = (10.0, 1800.0);

    // Inlet net recovery: pi_d = pi_d_max * ram_recovery(M0). At M0 = 2, eta_r = 0.925.
    let pi_d = 0.95 * ram_recovery(m0);
    assert!(close(pi_d, 0.87875));

    // Compressor: polytropic e_c = 0.9 -> isentropic eta_c (exact for a perfect gas).
    let gc = Gas::new(gas_spec.clone()).g_c();
    let e_c = 0.9;
    let eta_c = (powp(pi_c, gc) - 1.0) / (powp(pi_c, gc / e_c) - 1.0);

    let common = Losses {
        pi_d, eta_c, eta_b: 0.98, pi_b: 0.94, eta_m: 0.99, pi_n: 0.96, p_exit: Some(p9),
        ..Losses::default()
    };

    // Provisional pass (eta_t = 1) just to recover tau_t = Tt5/Tt4.
    let prov = build_turbojet(Gas::new(gas_spec.clone()), pi_c, tt4, p0, common)
        .run(&flight, 1.0);
    let tau_t = prov.station("5").tt / tt4;
    let e_t = 0.9;
    let eta_t = (1.0 - tau_t) / (1.0 - powp(tau_t, 1.0 / e_t));

    let result = build_turbojet(Gas::new(gas_spec), pi_c, tt4, p0, Losses { eta_t, ..common })
        .run(&flight, 1.0);
    let perf = &result.performance;

    // The book rounds intermediates to 4 sig figs; actual deviations are <= 0.015 %, so 0.05 %
    // keeps margin over that rounding while still catching real regressions.
    let tol = 5e-4;
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
