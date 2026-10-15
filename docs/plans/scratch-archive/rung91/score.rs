//! Rung 91 scorer — measures every quantity the anchor (e46ab3d) pre-registered.
use turbojet::bleed_transient::LeverArm;
use turbojet::customer_bleed::CustomerBleed;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::turbine_cooling::{CooledResult, TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
                     eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None,
                     nozzle_convergent: true }
}
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
                       gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
                       hpr: 42.8e6, ..GasSpec::default() })
}
fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real()) }
fn de(cb: Option<CustomerBleed>, tc: Option<TurbineCooling>) -> TwoSpoolEngine {
    let mut e = design();
    if let Some(c) = cb { e = e.with_customer_bleed(c); }
    if let Some(t) = tc { e = e.with_turbine_cooling(t); }
    e
}
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn lp_shaped() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() } }
fn hp_shaped() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() } }
fn tilt() -> ComponentMap { ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() } }

const BAND: [f64; 12] = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 850.0, 800.0, 750.0, 700.0, 650.0];
// Anchor § 1's table (frozen at e46ab3d): [p, LP, HP] per stream.
const VANE_P: [f64; 12] = [0.751, 0.759, 0.767, 0.777, 0.788, 0.801, 0.817, 0.826, 0.836, 0.848, 0.861, 0.876];
const VANE_L: [f64; 12] = [-1.028, -0.955, -0.876, -0.790, -0.698, -0.598, -0.489, -0.432, -0.372, -0.309, -0.244, -0.177];
const VANE_H: [f64; 12] = [-0.239, -0.185, -0.125, -0.059, 0.015, 0.096, 0.187, 0.237, 0.289, 0.344, 0.402, 0.464];
const BLADE_P: [f64; 12] = [0.792, 0.800, 0.810, 0.821, 0.834, 0.850, 0.868, 0.879, 0.891, 0.905, 0.920, 0.938];
const BLADE_L: [f64; 12] = [-1.609, -1.536, -1.458, -1.373, -1.281, -1.182, -1.075, -1.018, -0.959, -0.898, -0.835, -0.769];
const BLADE_H: [f64; 12] = [-0.624, -0.562, -0.492, -0.415, -0.330, -0.234, -0.125, -0.066, -0.003, 0.065, 0.137, 0.215];

fn cm(cb: Option<CustomerBleed>, tc: Option<TurbineCooling>, ml: ComponentMap, mh: ComponentMap) -> TurbineCoolingMatcher {
    TurbineCoolingMatcher::new(de(cb, tc), flight(), 1.0, ml, mh, 0.0)
}
fn stream(vane: bool, e: f64) -> TurbineCooling {
    if vane { TurbineCooling::new(e, 0.0) } else { TurbineCooling::new(0.0, e) }
}

/// Richardson-extrapolated d ln(x)/d eps from one-sided differences at 0.01 and 0.02.
fn rich(x0: f64, x1: f64, x2: f64) -> f64 {
    2.0 * (x1 / x0).ln() / 0.01 - (x2 / x0).ln() / 0.02
}

fn part_a() {
    for (name, ml, mh) in [("flat", ComponentMap::flat(), ComponentMap::flat()),
                           ("flow/press", lp_shaped(), hp_shaped())] {
        println!("\n=== PART A — {name}");
        let fl = name == "flat";
        let m0 = cm(None, None, ml, mh);
        let mc = turbojet::customer_bleed::CustomerBleedMatcher::new(de(Some(CustomerBleed::share(0.05)), None), flight(), 1.0, ml, mh, 0.0);
        for vane in [true, false] {
            let sname = if vane { "VANE" } else { "BLADE" };
            let (mp, mlp, mhp) = if vane { (VANE_P, VANE_L, VANE_H) } else { (BLADE_P, BLADE_L, BLADE_H) };
            let m1 = cm(None, Some(stream(vane, 0.01)), ml, mh);
            let m2 = cm(None, Some(stream(vane, 0.02)), ml, mh);
            let m5 = cm(None, Some(stream(vane, 0.05)), ml, mh);
            println!("-- {sname}: Tt4 | dlnpiHPT meas model diff | dlnphiL meas model diff | dlnphiH meas model diff \
                      | one-sided(0.02) L H | tauLPT rel | @0.05: Tt41 Tt45 dF%inlet dTSFC% | customer.05: dF% dTSFC%");
            for (i, &tt4) in BAND.iter().enumerate() {
                let r0 = match m0.try_match_point(&flight(), tt4) { Ok(r) => r, Err(e) => { println!("{tt4} base FAIL {}", e.0); continue } };
                let get = |m: &TurbineCoolingMatcher| -> Option<CooledResult> { m.try_match_point(&flight(), tt4).ok() };
                let (Some(r1), Some(r2)) = (get(&m1), get(&m2)) else { println!("{tt4} cooled FAIL"); continue };
                let b0 = &r0.base;
                let dp = rich(b0.base.pi_hpt, r1.base.base.pi_hpt, r2.base.base.pi_hpt);
                let dl = rich(b0.phi_lp, r1.base.phi_lp, r2.base.phi_lp);
                let dh = rich(b0.phi_hp, r1.base.phi_hp, r2.base.phi_hp);
                let (ol, oh) = ((r2.base.phi_lp / b0.phi_lp).ln() / 0.02, (r2.base.phi_hp / b0.phi_hp).ln() / 0.02);
                let tlpt = r2.base.base.tau_lpt / b0.base.tau_lpt - 1.0;
                let st0 = b0.base.performance.specific_thrust;
                let tsfc0 = b0.base.performance.tsfc;
                let a5 = match get(&m5) {
                    Some(r) => { let k = r.booking.unwrap();
                        format!("{:.1} {:.1} {:+.3} {:+.3}", k.tt41, k.tt45, 100.0 * (k.st_inlet / st0 - 1.0), 100.0 * (k.tsfc_inlet / tsfc0 - 1.0)) }
                    None => "FAIL".into(),
                };
                let c5 = match mc.try_match_point(&flight(), tt4) {
                    Ok(r) => { let k = r.booking.unwrap();
                        format!("{:+.3} {:+.3}", 100.0 * (k.st_inlet / st0 - 1.0), 100.0 * (k.tsfc_inlet / tsfc0 - 1.0)) }
                    Err(_) => "FAIL".into(),
                };
                let cmp = |meas: f64, model: f64| if fl { format!("{model:+.3} {:+.4}", meas - model) } else { "-".into() };
                println!("{tt4:.0} | {dp:+.4} {} | {dl:+.4} {} | {dh:+.4} {} | {ol:+.4} {oh:+.4} | {tlpt:+.1e} | {a5} | {c5}",
                         cmp(dp, mp[i]), cmp(dl, mlp[i]), cmp(dh, mhp[i]));
            }
        }
    }
}

fn machine(cb: Option<CustomerBleed>, tc: Option<TurbineCooling>, ml: ComponentMap, mh: ComponentMap, arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(de(cb, tc), flight(), 1.0, Some(ml.with_phi_surge(0.55)), Some(mh.with_phi_surge(0.55)), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn part_b() {
    println!("\n=== PART B — rung 64's rig, ds 0.005, Tt4 1000->1400, settle 1.2, b_max 0.10");
    let demands: [(&str, Option<CustomerBleed>, Option<TurbineCooling>); 4] = [
        ("none", None, None),
        ("vane.05", None, Some(TurbineCooling::new(0.05, 0.0))),
        ("blade.05", None, Some(TurbineCooling::new(0.0, 0.05))),
        ("cust.05", Some(CustomerBleed::share(0.05)), None),
    ];
    for (shape, ml, mh) in [("shaped", lp_shaped(), hp_shaped()), ("tilted", tilt(), tilt())] {
        for r in [0.25, 0.5, 1.0] {
            let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds: 0.005 };
            for (dn, cb, tc) in demands {
                let base = machine(cb, tc, ml, mh, &LeverArm::default());
                let shut = base.bill_cell(&flight(), &ramp, false);
                let full = base.at_lever(&LeverArm::constant(0.10)).bill_cell(&flight(), &ramp, false);
                let mut line = format!("{shape} r={r} {dn:8} | shut minphiL {:.6} minphiH {:.6} npts {} | ceiling {:.6} npts {} |",
                                       shut.min_phi_lp, shut.min_phi_hp, shut.npts, full.min_phi_lp, full.npts);
                for sm in [0.30, 0.34, 0.38, 0.42] {
                    let lim = BleedLimiter::new((1.0 + sm) * 0.55, 0.10);
                    let c = base.at_lever(&LeverArm::floored(lim)).bill_cell(&flight(), &ramp, false);
                    line += &format!(" sm{sm:.2}: bint {:.6e} bpeak {:.4} npts {} minphiL {:.6} |",
                                     c.b_int, c.b_peak, c.npts, c.min_phi_lp);
                }
                println!("{line}");
            }
        }
    }
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_else(|| "ab".into());
    if which.contains('a') { part_a(); }
    if which.contains('b') { part_b(); }
}
