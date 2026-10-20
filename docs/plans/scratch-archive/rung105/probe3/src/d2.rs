//! d2: why rung 78's `gauge_march` indexes an empty list under `hp-law-default` — rung 78's own rig
//! (tests/rung78.rs, verbatim), its first `cap_march`, and what the trajectory holds.
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::residual_gauge::build_residual_gauge_cascade;
use turbojet::sensed_cap::{accel_for, cap_march};
use turbojet::shared_actuator::riding4;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const PHI_JAC: f64 = 0.80;
const MARGIN: f64 = 0.10;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}
fn lp_map() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(FLOOR) }
fn hp_map() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(FLOOR) }
fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL) }
fn sm() -> f64 { PHI_JAC / FLOOR - 1.0 }
fn rig() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm(), Some(0.05))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm(), Some(0.05))),
        ..Default::default()
    };
    let m = match build_residual_gauge_cascade(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    m.fuel.inner.lag_coord.set("demand");
    m.fuel.inner.ref_law.set("sched");
    m.fuel.inner.windup_law.set("none");
    m.fuel.inner.cap_law.set("solve");
    m
}

fn main() {
    let law_default = cfg!(feature = "law");
    let core = rig();
    let fl = flight();
    let acc = accel_for(&core, &fl, 1000.0, 1400.0, sm(), 1200.0, TAUS, V_MAX, false, MARGIN);
    let (m0, _s, _l, traj0) = cap_march(&core, &fl, 1000.0, 1400.0, 1200.0, sm(), TAUS, 0.5, 1.2, 0.005,
                                        V_MAX, false, "demand", "sched", "none", None, "solve", &acc, None);
    let b_max = m0.fuel.inner.lever.lim.expect("armed").b_max;
    let ride = riding4(&traj0, b_max);
    println!("feature law={law_default}: trajectory {} points (rung 78 asserts 341), riding the valve {}", traj0.len(), ride.len());
    if let (Some(a), Some(z)) = (traj0.first(), traj0.last()) {
        println!("  first s={} nu=({:.5},{:.5})  last s={} nu=({:.5},{:.5})", a.s, a.nu_lp, a.nu_hp, z.s, z.nu_lp, z.nu_hp);
    }
    let (mut nmax_hp, mut tmax) = (0.0f64, 0.0f64);
    for p in &traj0 { nmax_hp = nmax_hp.max(p.nu_hp); tmax = tmax.max(p.tt4); }
    println!("  over the march: max nu_hp {nmax_hp:.6}, max Tt4 {tmax:.2} K");
    for k in [0usize, 40, 80, 120, 200, 340] {
        let p = &traj0[k];
        println!("  [{k}] s={:.3} nu=({:.6},{:.6}) Tt4={:.2} mf={:.6} mf_sched={:.6} pi_hpc={:.5}", p.s, p.nu_lp, p.nu_hp, p.tt4, p.mf, p.mf_sched, p.pi_hpc);
        println!("       extra {:?}", p.extra);
    }
}
