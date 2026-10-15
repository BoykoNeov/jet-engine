//! Rung 90 scorer — measures every quantity the anchor (504d428) pre-registered.
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::bleed_transient::LeverArm;
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};
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
fn de(cb: Option<CustomerBleed>) -> TwoSpoolEngine {
    match cb { Some(c) => design().with_customer_bleed(c), None => design() }
}
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn lp_shaped() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() } }
fn hp_shaped() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() } }
fn tilt() -> ComponentMap { ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() } }

const MODEL_LP: [f64; 12] = [-0.438, -0.379, -0.315, -0.246, -0.171, -0.089, 0.001, 0.049, 0.099, 0.153, 0.209, 0.268];
const MODEL_HP: [f64; 12] = [0.215, 0.240, 0.269, 0.301, 0.339, 0.382, 0.432, 0.460, 0.490, 0.522, 0.556, 0.594];
const BAND: [f64; 12] = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 850.0, 800.0, 750.0, 700.0, 650.0];

fn cm(cb: Option<CustomerBleed>, ml: ComponentMap, mh: ComponentMap, b: f64) -> CustomerBleedMatcher {
    CustomerBleedMatcher::new(de(cb), flight(), 1.0, ml, mh, b)
}

fn part_a() {
    for (name, ml, mh) in [("flat", ComponentMap::flat(), ComponentMap::flat()),
                           ("flow/press", lp_shaped(), hp_shaped())] {
        println!("\n=== PART A — {name}");
        let m0 = cm(None, ml, mh, 0.0);
        let m2 = cm(Some(CustomerBleed::share(0.02)), ml, mh, 0.0);
        let m5 = cm(Some(CustomerBleed::share(0.05)), ml, mh, 0.0);
        let b5 = TwoSpoolBleedMatcher::new(design(), flight(), 1.0, ml, mh, 0.05);
        let w5 = cm(Some(CustomerBleed::amount(0.05)), ml, mh, 0.0);
        println!("Tt4 | dlnphiL/dc meas model diff | dlnphiH/dc meas model diff | A5 offline dphiH% | A7 dF%(c) dF%(b) dTSFC%(c) dTSFC%(b) | A8 w: share  dlnphiL(w) dlnphiL(c.05)");
        for (i, &tt4) in BAND.iter().enumerate() {
            let r0 = match m0.try_match_point(&flight(), tt4) { Ok(r) => r, Err(e) => { println!("{tt4} c=0 FAIL {}", e.0); continue } };
            let p0 = &r0.base;
            let s = |r: Result<turbojet::customer_bleed::CustomerBleedResult, turbojet::gas::Abort>| r.ok();
            let (Some(r2), Some(r5)) = (s(m2.try_match_point(&flight(), tt4)), s(m5.try_match_point(&flight(), tt4))) else {
                println!("{tt4} c>0 FAIL"); continue };
            let dl = (r2.base.phi_lp / p0.phi_lp).ln() / 0.02;
            let dh = (r2.base.phi_hp / p0.phi_hp).ln() / 0.02;
            // A5: c=0.05 HP point vs c=0 line at the same x_H = Tt4/Tt25 (bisection on Tt4').
            let x_target = tt4 / r5.base.base.station("25").tt;
            let xh = |t: f64| -> Option<(f64, f64)> {
                m0.try_match_point(&flight(), t).ok().map(|r| (t / r.base.base.station("25").tt, r.base.phi_hp))
            };
            let (mut lo, mut hi) = (tt4 - 300.0, tt4 + 300.0);
            let mut off = f64::NAN;
            if let (Some((xl, _)), Some((xu, _))) = (xh(lo), xh(hi)) {
                if (xl - x_target) * (xu - x_target) < 0.0 {
                    let up = xu > xl;
                    for _ in 0..80 {
                        let mid = 0.5 * (lo + hi);
                        let Some((xm, _)) = xh(mid) else { break };
                        if (xm < x_target) == up { lo = mid } else { hi = mid }
                    }
                    if let Some((_, ph)) = xh(0.5 * (lo + hi)) {
                        off = 100.0 * (r5.base.phi_hp / ph - 1.0);
                    }
                }
            }
            // A7
            let st0 = p0.base.performance.specific_thrust; // c=0, b=0 -> core = inlet
            let tsfc0 = p0.base.performance.tsfc;
            let k5 = r5.booking.unwrap();
            let rb = b5.match_point(&flight(), tt4);
            let kb = rb.booking.unwrap();
            let (dfc, dfb) = (100.0 * (k5.st_inlet / st0 - 1.0), 100.0 * (kb.st_inlet / st0 - 1.0));
            let (dtc, dtb) = (100.0 * (k5.tsfc_inlet / tsfc0 - 1.0), 100.0 * (kb.tsfc_inlet / tsfc0 - 1.0));
            // A8
            let a8 = match w5.try_match_point(&flight(), tt4) {
                Ok(rw) => format!("{:.4} {:+.5} {:+.5}", rw.booking.unwrap().customer,
                                  (rw.base.phi_lp / p0.phi_lp).ln(), (r5.base.phi_lp / p0.phi_lp).ln()),
                Err(e) => format!("FAIL {}", e.0),
            };
            let fl = name == "flat";
            println!("{tt4:.0} | {dl:+.4} {} | {dh:+.4} {} | {off:+.3} | {dfc:+.3} {dfb:+.3} {dtc:+.3} {dtb:+.3} | {a8}",
                     if fl { format!("{:+.3} {:+.4}", MODEL_LP[i], dl - MODEL_LP[i]) } else { "-".into() },
                     if fl { format!("{:+.3} {:+.4}", MODEL_HP[i], dh - MODEL_HP[i]) } else { "-".into() });
        }
    }
}

fn machine(cb: Option<CustomerBleed>, ml: ComponentMap, mh: ComponentMap, arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(de(cb), flight(), 1.0, Some(ml.with_phi_surge(0.55)), Some(mh.with_phi_surge(0.55)), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn part_b() {
    println!("\n=== PART B — rung 64's rig, ds 0.005, Tt4 1000->1400, settle 1.2, b_max 0.10");
    let cust: [(&str, Option<CustomerBleed>); 3] =
        [("none", None), ("share.05", Some(CustomerBleed::share(0.05))), ("amount.05", Some(CustomerBleed::amount(0.05)))];
    for (shape, ml, mh) in [("shaped", lp_shaped(), hp_shaped()), ("tilted", tilt(), tilt())] {
        for r in [0.25, 0.5, 1.0] {
            let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds: 0.005 };
            for (cn, cb) in cust {
                let base = machine(cb, ml, mh, &LeverArm::default());
                let shut = base.bill_cell(&flight(), &ramp, false);
                let full = base.at_lever(&LeverArm::constant(0.10)).bill_cell(&flight(), &ramp, false);
                let mut line = format!("{shape} r={r} {cn:9} | shut minphiL {:.6} minphiH {:.6} npts {} | ceiling {:.6} npts {} |",
                                       shut.min_phi_lp, shut.min_phi_hp, shut.npts, full.min_phi_lp, full.npts);
                for sm in [0.30, 0.34, 0.38, 0.42] {
                    let lim = BleedLimiter::new((1.0 + sm) * 0.55, 0.10);
                    let c = base.at_lever(&LeverArm::floored(lim)).bill_cell(&flight(), &ramp, false);
                    line += &format!(" sm{sm:.2}: bint {:.6e} bpeak {:.4} npts {} minphiL {:.6} minphiH {:.6} |",
                                     c.b_int, c.b_peak, c.npts, c.min_phi_lp, c.min_phi_hp);
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
