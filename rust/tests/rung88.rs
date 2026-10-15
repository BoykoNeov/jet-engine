//! RUNG 88 — the SLOW valve on a moving target: does a lag's cost depend on the wall it watches?
//! `docs/rung88-spec.md`; anchor `docs/plans/rung88-anchor-lagged-incidence.md` (§ refs are to it).
//!
//! Gate groups:
//!
//!   1. THE DEVICE — a lag keeps the valve's reference (it did not: `lagged` reset it to `φ`); with
//!      the stators off the lagged incidence valve IS the lagged `φ` valve bit for bit; the rung
//!      66–80 spelling still refuses an incidence valve.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::{r64_solve_b, BleedLimiter, ValveReference};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_levers::march;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::valve_reference::{incidence_valve, sibling};

// ---------------------------------------------------------------------------- rung 87's rig
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const DS: f64 = 0.005;
const B: f64 = 0.10;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

/// The bare machine, through RUNG 65's builder: rung 64's refuses a lagged valve, and with `τ = None`
/// the two march bit for bit (anchor G1).
fn gt() -> ScheduledStatorCore {
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(r: f64, ds: f64) -> Ramp {
    Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds }
}

fn phi_of(sm: f64) -> f64 {
    FLOOR * (1.0 + sm)
}

fn valve_bits(p: &FuelPoint) -> (u64, u64) {
    match p.extra {
        PointExtra::Valve { b, b_cmd } => (b.to_bits(), b_cmd.to_bits()),
        _ => panic!("a lagged march records the valve at every point"),
    }
}

// =============================================================================================
// GATE 1 — THE DEVICE
// =============================================================================================

/// § 1: `lagged` rebuilt through `with_tau`, which resets the reference to `φ` — a lagged incidence
/// valve came back a `φ` valve without a word. It keeps it now, and both spellings build one object.
#[test]
fn a_lag_keeps_the_reference() {
    let inc = incidence_valve(phi_of(0.42), B);
    let slow = inc.lagged(0.05);
    assert_eq!(slow.reference, ValveReference::Incidence);
    assert_eq!(slow.tau, Some(0.05));
    assert_eq!(slow, BleedLimiter::with_tau(phi_of(0.42), B, Some(0.05)).on_incidence());
    assert_eq!(BleedLimiter::new(phi_of(0.42), B).lagged(0.05).reference, ValveReference::Phi);
}

/// G1: with the stators off `v ≡ 0`, so the lagged incidence valve IS the lagged `φ` valve — every
/// point, every recorded position and every recorded command, bit for bit.
#[test]
fn the_lagged_incidence_valve_with_the_stators_off_is_the_lagged_phi_valve_bit_for_bit() {
    let m = gt();
    let rp = ramp(0.5, DS);
    let phi = BleedLimiter::with_tau(phi_of(0.42), B, Some(0.05));
    let (ta, _) = march(&sibling(&m, None, Some(phi)), &flight(), &rp, None);
    let (tb, _) = march(&sibling(&m, None, Some(phi.on_incidence())), &flight(), &rp, None);
    assert_eq!(ta.len(), tb.len());
    assert!(ta.len() > 300, "the march ran: {} points", ta.len());
    for (p, q) in ta.iter().zip(&tb) {
        assert!(p.s.to_bits() == q.s.to_bits() && p.nu_lp.to_bits() == q.nu_lp.to_bits()
                    && p.nu_hp.to_bits() == q.nu_hp.to_bits() && p.phi_lp.to_bits() == q.phi_lp.to_bits()
                    && p.mf.to_bits() == q.mf.to_bits() && p.sp_thrust.to_bits() == q.sp_thrust.to_bits(),
                "differs at s = {}", p.s);
        assert_eq!(valve_bits(p), valve_bits(q), "the recorded valve differs at s = {}", p.s);
    }
}

/// Rungs 66–80 solve every valve through `r64_solve_b`, which reads `phi_lim` as a flow coefficient —
/// so a LAGGED incidence valve reaching it is refused exactly as rung 87's instantaneous one was.
#[test]
fn the_rung66_to_80_spelling_still_refuses_an_incidence_valve() {
    struct S(f64);
    impl turbojet::limited_bleed::HasPhiLp for S {
        fn phi_lp(&self) -> f64 { self.0 }
    }
    let slow = incidence_valve(phi_of(0.42), B).lagged(0.05);
    assert!(std::panic::catch_unwind(|| r64_solve_b(&slow, |b| Ok(S(0.7 + b)))).is_err());
}
