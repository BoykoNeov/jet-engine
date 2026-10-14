//! **THE SCHEDULE-CHECK CEILING, ON THE MARCHERS NO SHIPPED GRID REACHES** (2026-10-14, sandbox
//! plan § 14.5–14.6). Not a rung.
//!
//! Step A put rung 43's *"decide the min-select from the most fuel that solves"* into every
//! rung-65+ marcher, each in its own shape. A census of the whole gate forced ON found that the
//! shipped grids exercise rungs 43, 65, 68 and 72's branches — and NEVER rung 66's fuel leg, rung
//! 67's or rung 70's governor, or rung 74's demand cells (the last only inside a planted-defect
//! test). This file builds, for each of those, one operating point where the switch-OFF march
//! stops on the schedule check, and gates the switch on it:
//!
//! 1. the OFF march is SHORTER than the ON march;
//! 2. the points both reach are BIT-IDENTICAL — the switch acts only where OFF had no answer;
//! 3. OFF stopped on the schedule check: at the first point only ON reaches, the instant at the
//!    SCHEDULED fuel has no operating point (read with the live valve / stator states — the plant
//!    the march is on, not a shut-valve one);
//! 4. ON ends either at the ramp's end or on a NAMED physical limit — never silently.
//!
//! Each run takes a FRESH machine: one rung-70 machine reused for six marches in a row starts its
//! sixth off-equilibrium with the switch OFF or ON alike (pre-existing, not traced; sandbox plan
//! § 14.6), which would confound (2).

use turbojet::bleed_transient::LeverArm;
use turbojet::cross_loop::build_cross_loop_cascade;
use turbojet::cross_split::build_cross_split_cascade;
use turbojet::demand_coordinate::build_demand_coordinate_cascade;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::REF_LAW_DEFAULT;
use turbojet::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_lag::build_two_lag_cascade;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{MarchedBleed, MarchedStator};

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
/// The natural `b0` of the rung-65 valve at `phi_lim` = 0.80 (`tests/rung65.rs`'s `b_cmd(0)`).
const B_NAT: f64 = 0.03662636367018262;

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

fn full(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("the LP spool is never disabled"),
    }
}

fn sm_of(phi: f64) -> f64 { phi / FLOOR - 1.0 }
fn valve(sm: f64) -> BleedLimiter { BleedLimiter::from_margin_tau(&lp(), B, sm, Some(TAU)) }
fn floor(sm: f64) -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, sm)) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(0.05, 0.15) }
fn ramp(hi: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: hi, r: R, s_settle: SETTLE, ds: DS } }

/// The live actuator states a point carries — the plant the march is on there.
fn states(p: &FuelPoint) -> (Option<f64>, Option<f64>) {
    match p.extra {
        PointExtra::Cascade { b, .. } | PointExtra::CrossCascade { b, .. } => (Some(b), None),
        PointExtra::Triple { b, v, .. } | PointExtra::Demand { b, v, .. } => (Some(b), Some(v)),
        ref x => panic!("a point with no valve state: {x:?}"),
    }
}

/// The OFF and ON marches of one operating point, each on a fresh machine.
fn pair(mk: &dyn Fn() -> ScheduledStatorCore, leg: &StatorLeg<'_>, sc: &MarchScope, rp: &Ramp)
    -> (Vec<FuelPoint>, Vec<FuelPoint>) {
    let go = |on: bool| {
        let m = mk();
        m.fuel.inner.below_ceiling.set(on);
        m.stator_march_scoped(&flight(), rp, None, leg, sc).0
    };
    (go(false), go(true))
}

/// Gates 1–3; returns the ON march for gate 4.
fn gate(name: &str, mk: &dyn Fn() -> ScheduledStatorCore, leg: &StatorLeg<'_>, sc: &MarchScope,
        rp: &Ramp) -> Vec<FuelPoint> {
    let (off, on) = pair(mk, leg, sc, rp);
    assert!(off.len() < on.len(), "{name}: OFF {} points is not shorter than ON {}", off.len(), on.len());
    for (i, (a, b)) in off.iter().zip(&on).enumerate() {
        let ka = [a.s, a.nu_lp, a.nu_hp, a.phi_lp, a.phi_hp, a.tt4, a.mf].map(f64::to_bits);
        let kb = [b.s, b.nu_lp, b.nu_hp, b.phi_lp, b.phi_hp, b.tt4, b.mf].map(f64::to_bits);
        assert_eq!(ka, kb, "{name}: point {i} (s = {}) differs between OFF and ON", a.s);
    }
    let p = &on[off.len()];
    let m = mk();
    let (b, v) = states(p);
    let _sb = b.map(|b| MarchedBleed::set(&m.fuel.inner, b));
    let _sv = v.map(|v| MarchedStator::set(&m.fuel.inner, v));
    let e = m.fuel.try_instant_fuel(&flight(), p.nu_lp, p.nu_hp, p.mf_sched).err();
    assert!(e.is_some(),
            "{name}: the scheduled fuel solves at s = {}, the first point only ON reaches -- so \
             OFF did not stop on the schedule check", p.s);
    on
}

fn ran_to_end(name: &str, on: &[FuelPoint], rp: &Ramp) {
    let n_full = ((rp.r + rp.s_settle) / rp.ds).round() as usize + 1;
    assert_eq!(on.len(), n_full, "{name}: the ON march ends early at s = {}", on.last().unwrap().s);
}

/// RUNG 66 — the lagged fuel leg (`try_clip_caps`). A high floor and a low valve start: OFF stops
/// at 215 points; ON runs on until the valve is FULLY OPEN and `phi_lp` sits below the floor — no
/// actuator has authority left, the floor's own refusal.
#[test]
fn rung66_the_lagged_fuel_leg_runs_past_the_check() {
    let sm = sm_of(0.86);
    let mk = || full(build_two_lag_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0,
                     &LeverArm { bleed_lim: Some(valve(sm)), ..Default::default() }));
    let leg = StatorLeg { accel: None, surge: Some(floor(sm)), tt4_max: None };
    let sc = MarchScope { lag: Some(lag()), b0: Some(0.5 * B_NAT), ..MarchScope::DEFAULT };
    let rp = ramp(1400.0);
    let on = gate("rung 66", &mk, &leg, &sc, &rp);
    let end = on.last().unwrap();
    let (b, _) = states(end);
    assert!((b.unwrap() - B).abs() < 1e-12, "rung 66: ON ends with the valve at {b:?}, not open");
    assert!(end.phi_lp < 0.86, "rung 66: ON ends holding the floor ({})", end.phi_lp);
    assert!(on.len() < ((rp.r + rp.s_settle) / rp.ds).round() as usize + 1);
}

/// RUNG 67 — the lagged governor beside the valve (`try_topping_required_below`). A steep schedule
/// (to 1600 K) the 1100 K redline cuts deep: OFF stops at 96 points, ON runs the whole ramp.
#[test]
fn rung67_the_governor_runs_past_the_check() {
    let mk = || full(build_cross_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0,
                     &LeverArm { bleed_lim: Some(valve(sm_of(0.80))), ..Default::default() }));
    let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(1100.0) };
    let sc = MarchScope { tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT };
    let rp = ramp(1600.0);
    let on = gate("rung 67", &mk, &leg, &sc, &rp);
    ran_to_end("rung 67", &on, &rp);
}

/// RUNG 70 — the governor beside the valve AND the lagged `phi` stator (the five-state march, also
/// rung 71's). Same point as rung 67.
#[test]
fn rung70_the_governor_runs_past_the_check_beside_the_stator() {
    let sm = sm_of(0.80);
    let mk = || full(build_cross_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0,
                     &LeverArm {
                         bleed_lim: Some(valve(sm)),
                         stator_lim: Some(StatorLimiter::from_margin(&lp(), V_MAX, sm, Some(0.05))),
                         ..Default::default()
                     }));
    let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(1100.0) };
    let sc = MarchScope { tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT };
    let rp = ramp(1600.0);
    let on = gate("rung 70", &mk, &leg, &sc, &rp);
    ran_to_end("rung 70", &on, &rp);
}

/// RUNG 74 — the demand cells (`cap_free_gated`, through the core's carrier), four loops: the
/// lagged `phi` fuel leg, the governor, the valve and the stator. OFF stops at 80 points, ON runs
/// the whole ramp.
#[test]
fn rung74_the_demand_cells_run_past_the_check() {
    let sm = sm_of(0.80);
    let mk = || {
        let m = full(build_demand_coordinate_cascade(
            design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0,
            &LeverArm {
                bleed_lim: Some(valve(sm)),
                stator_lim: Some(StatorLimiter::from_margin(&lp(), V_MAX, sm, Some(0.05))),
                ..Default::default()
            }));
        m.fuel.inner.lag_coord.set("demand");
        m.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
        m
    };
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: Some(floor(sm)), tt4_max: Some(1200.0) };
    let sc = MarchScope { lag: Some(lag()), tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT };
    let rp = ramp(1600.0);
    let on = gate("rung 74", &mk, &leg, &sc, &rp);
    ran_to_end("rung 74", &on, &rp);
}
