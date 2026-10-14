//! **A REUSED MACHINE MARCHES LIKE A FRESH ONE** (2026-10-14, sandbox plan § 14.6). Not a rung.
//!
//! A stator march writes the LIVE compressor maps as the stators move (rung 57's `_arm`, rung
//! 68's limiter position) and, by Python's shape, never restores them inside the march. Nothing
//! restored them AFTER it either, so one rung-70 machine reused for six marches started its sixth
//! off-equilibrium: march 5 left the LP stators at `v` = −8.3e-4, and march 6's `s = 0`
//! equilibrium was solved on that moved map. `FuelTransientCore::stator_march_scoped` now puts
//! both maps back to the AS-BUILT ones at the START of every march (only there: the map a march
//! leaves behind is Python's, and slice V gates it).
//!
//! 1. every march on one reused machine equals a fresh machine's march, bit for bit;
//! 2. the case is not vacuous: some march ends with the stators OFF zero and leaves a moved map;
//! 3. the reset is a NO-OP on a fresh machine — the as-built map it restores IS the live one,
//!    including a CONSTANT setting (not the design map), so no fresh-machine number can move.

use turbojet::bleed_transient::LeverArm;
use turbojet::cross_split::build_cross_split_cascade;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg,
};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const SM: f64 = 0.80 / FLOOR - 1.0;
/// The six redlines of the sequence that first showed the drift (sandbox plan § 14.6).
const REDLINES: [f64; 6] = [1200.0, 1200.0, 1100.0, 1100.0, 1050.0, 1050.0];

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL) }

/// Rung 70: the `Tt4` governor beside the `phi` valve and the lagged `phi` stator, with an
/// optional CONSTANT setting on the HP stators.
fn rung70(vsv_hp: f64) -> ScheduledStatorCore {
    let arm = LeverArm {
        stator: StatorArm { vsv_hp, ..Default::default() },
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp(), 0.10, SM, Some(0.05))),
        stator_lim: Some(StatorLimiter::from_margin(&lp(), 0.20, SM, Some(0.05))),
        ..Default::default()
    };
    match build_cross_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, &arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("the LP spool is never disabled"),
    }
}

fn march(m: &ScheduledStatorCore, redline: f64) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(redline) };
    let sc = MarchScope { tau_gov: Some(0.05), ..MarchScope::DEFAULT };
    let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1600.0, r: 0.5, s_settle: 1.2, ds: 0.005 };
    m.stator_march_scoped(&flight(), &ramp, None, &leg, &sc).0
}

fn keys(t: &[FuelPoint]) -> Vec<[u64; 7]> {
    t.iter().map(|p| [p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.tt4, p.mf].map(f64::to_bits))
        .collect()
}

fn v_end(t: &[FuelPoint]) -> f64 {
    match t.last().expect("an empty march").extra {
        PointExtra::Triple { v, .. } => v,
        ref x => panic!("a rung-70 point with no stator state: {x:?}"),
    }
}

/// Gates 1 and 2 on one machine; returns it for gate 3.
fn sequence(vsv_hp: f64) -> ScheduledStatorCore {
    let reused = rung70(vsv_hp);
    let mut moved = 0;
    for (k, &red) in REDLINES.iter().enumerate() {
        let t = march(&reused, red);
        assert_eq!(keys(&t), keys(&march(&rung70(vsv_hp), red)),
                   "march {} (redline {red} K) on the reused machine differs from a fresh one",
                   k + 1);
        if v_end(&t).abs() > 1e-6 {
            moved += 1;
            assert_ne!(reused.fuel.inner.inner.map_lp(), reused.arming().built_map(Spool::Lp),
                       "march {} ended off zero yet left the as-built LP map", k + 1);
        }
    }
    assert!(moved > 0, "no march left the stators off zero -- the sequence no longer exercises \
                        the carry-over it was built to catch");
    reused
}

/// Gate 3: what the reset restores is exactly what a fresh machine already holds.
fn reset_is_a_no_op_when_fresh(vsv_hp: f64) {
    let m = rung70(vsv_hp);
    let a = m.arming();
    assert_eq!(m.fuel.inner.inner.map_lp(), a.built_map(Spool::Lp));
    assert_eq!(m.fuel.inner.inner.map_hp(), a.built_map(Spool::Hp));
}

#[test]
fn a_reused_machine_marches_like_a_fresh_one() {
    sequence(0.0);
    reset_is_a_no_op_when_fresh(0.0);
}

#[test]
fn a_constant_setting_is_restored_as_built() {
    let vsv_hp = 0.05;
    sequence(vsv_hp);
    reset_is_a_no_op_when_fresh(vsv_hp);
    let a = rung70(vsv_hp).arming().clone();
    assert_ne!(a.built_map(Spool::Hp), a.design_map(Spool::Hp),
               "a non-zero constant setting built the design map -- this case would be vacuous");
}
