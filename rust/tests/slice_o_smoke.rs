//! SLICE O STEP 1 — the smoke check: does a rung-61 core reproduce Python AT ALL?
//!
//! Sixteen values, dumped from PyPy as RAW BITS, on the one cell every downstream gate rides:
//! `(Tt4 = 1500, v = 0.20, spool = LP, target = phi)` on the suite's own map pair.
//!
//! **This is a GO/NO-GO, not the gate.** `slice_o_oracle.rs` is the gate; this exists so a
//! structural mistake in the two table pointers is caught before an oracle is written around it —
//! slice N's `slice_n_smoke.rs` precedent, and slice L's before that.

use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_bleed::{take_census, Compensating, StatorBleedCore, Target};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const FLOOR: f64 = 0.55;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg_gas(), 3.0, 6.0, 1500.0, 50_000.0, real())
}

fn maps() -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }.with_phi_surge(FLOOR),
     ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }.with_phi_surge(FLOOR))
}

fn sb(vl: f64, vh: f64, b: f64) -> StatorBleedCore {
    let (lp, hp) = maps();
    StatorBleedCore::new(design(), flight(), 1.0, lp, hp, vl, vh, b)
}

/// PyPy's bits, dumped at `M:\claud_projects\temp\slice_o` — never retyped from a decimal.
const PY: &[(&str, u64)] = &[
    ("b_star", 4595882203653729158),
    ("goal", 4607182418800017435),
    ("resid", 13663482151985741824),
    ("bare_m_i", 4605544746208246372),
    ("bare_phi", 4607182418800017435),
    ("bare_m_phi", 4601778099247172920),
    ("phi_comp", 4607182418799955121),
    ("n_comp", 4607680525799197967),
    ("thrust_comp", 4648448721920835895),
    ("thrust_bare", 4649651497634225895),
    ("thrust_stator", 4649622828732499147),
    ("d_m_i", 4596373779694078848),
    ("d_m_phi", 4588015747952444992),
    ("dn_comp", 4592634132650054224),
    ("d_f_comp", 13819268731958033320),
    ("d_phi_other_comp", 4580111770758969472),
];

fn want(name: &str) -> f64 {
    f64::from_bits(PY.iter().find(|(k, _)| *k == name).expect(name).1)
}

fn same(name: &str, got: f64) {
    let w = want(name);
    assert_eq!(got.to_bits(), w.to_bits(),
               "{name}: rust {got:?} ({:#x}) vs pypy {w:?} ({:#x})", got.to_bits(), w.to_bits());
}

#[test]
fn slice_o_smoke_compensating_bleed_is_bit_exact() {
    let m = sb(0.0, 0.0, 0.0);
    let c = m.compensating_bleed(&flight(), 1500.0, 0.20, Spool::Lp, Target::Phi);
    match c {
        Compensating::Solved { b_star, goal, resid, bare_m_i, bare_phi, bare_m_phi, .. } => {
            same("b_star", b_star);
            same("goal", goal);
            same("resid", resid);
            same("bare_m_i", bare_m_i);
            same("bare_phi", bare_phi);
            same("bare_m_phi", bare_m_phi);
        }
        other => panic!("expected a solved b*, got {other:?}"),
    }
}

#[test]
fn slice_o_smoke_compensated_point_is_bit_exact() {
    let m = sb(0.0, 0.0, 0.0);
    let p = m.compensated_point(&flight(), 1500.0, 0.20, Spool::Lp);
    let c = p.comp.expect("the LP spool compensates at Tt4 = 1500");
    same("thrust_bare", p.thrust_bare);
    same("thrust_stator", p.thrust_stator);
    same("phi_comp", c.phi_comp);
    same("n_comp", c.n_comp);
    same("thrust_comp", c.thrust_comp);
    same("d_m_i", c.d_m_i);
    same("d_m_phi", c.d_m_phi);
    same("dn_comp", c.dn_comp);
    same("d_f_comp", c.d_f_comp);
    same("d_phi_other_comp", c.d_phi_other_comp);
}

/// **THE STRUCTURAL HALF, WHICH NO VALUE ABOVE CAN SEE.** A sibling built through rung 53's
/// `at_setting` body comes back with the valve SHUT; rung 61's carries it. § 5.3 item 7 measured
/// the damage that failure does — 13–15 % on `φ` and `N`, 0.1 % on thrust — which is exactly why
/// this is asserted on the STATE and not on a number.
#[test]
fn at_setting_carries_the_bleed_and_rung53s_body_would_not() {
    let m = sb(0.0, 0.0, 0.10);
    assert_eq!(m.bleed(), 0.10);

    // Rung 61's override, at the concrete type…
    let sib = m.at_setting(0.20, 0.0);
    assert_eq!(sib.bleed(), 0.10, "at_setting dropped the valve position");
    assert_eq!(sib.core.vsv_lp, 0.20);

    // …and THROUGH THE TABLE, which is the copy rung 53's own readers reach.
    let dispatched = m.core.at_setting(0.20, 0.0);
    assert_eq!(dispatched.core.bleed, 0.10,
               "the DISPATCHED at_setting dropped the valve — rung 53's body ran, not rung 61's");
    assert_eq!(dispatched.core.hooks as *const _, m.core.core.hooks as *const _,
               "the sibling reverted the inner table, so its match is no longer rung 42's");
}

/// The census wiring, checked once so every later count can be trusted. One `compensating_bleed`
/// on the LP spool: § 5.11 (ii) predicts the cap dead and the interval arm dead.
#[test]
fn the_census_counts_and_both_dead_things_are_dead() {
    let m = sb(0.0, 0.0, 0.0);
    let _ = take_census();
    let _ = m.compensating_bleed(&flight(), 1500.0, 0.20, Spool::Lp, Target::Phi);
    let c = take_census();

    assert_eq!(c.feasible_calls, c.at_point_built, "one construction per trial, by construction");
    assert_eq!(c.feasible_none, 0,
               "§ 5.11 (i): the plant refuses NOTHING on this grid — {} refusals",
               c.feasible_none);
    assert_eq!(c.exit_tol, 1, "the bisection ends on _B_TOL");
    assert_eq!(c.exit_interval, 0, "the `hi - lo <= 1e-15` arm is DEAD (§ 5.11 (ii))");
    assert_eq!(c.exit_ran_out, 0, "_B_MAX was not exhausted");
    assert!(c.bisect_passes_max < StatorBleedCore::B_MAX as u64,
            "_B_MAX = {} is DEAD; the deepest bisection took {}",
            StatorBleedCore::B_MAX, c.bisect_passes_max);
    assert!(c.walk_steps_max as f64 <= StatorBleedCore::B_CAP / StatorBleedCore::B_STEP + 1.0);
}

