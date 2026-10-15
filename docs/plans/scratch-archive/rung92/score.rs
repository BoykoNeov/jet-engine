//! Rung 92 scorer — measures every quantity the anchor (1773b7d) pre-registered.
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses, TwoSpoolMapResult};

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
fn de(so: Option<ShaftOfftake>) -> TwoSpoolEngine {
    let e = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, real());
    match so { Some(o) => e.with_shaft_offtake(o), None => e }
}
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn lp_shaped() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() } }
fn hp_shaped() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() } }
fn tilt() -> ComponentMap { ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() } }

const BAND: [f64; 12] = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 850.0, 800.0, 750.0, 700.0, 650.0];
// Anchor § 1's table (frozen at 1773b7d), flat.
const HPS_L: [f64; 12] = [-1.402, -1.360, -1.313, -1.263, -1.209, -1.149, -1.084, -1.049, -1.012, -0.974, -0.933, -0.890];
const HPS_H: [f64; 12] = [-0.902, -0.860, -0.813, -0.763, -0.709, -0.649, -0.584, -0.549, -0.512, -0.474, -0.433, -0.390];
const LPS_L: [f64; 12] = [-0.035, -0.019, -0.002, 0.017, 0.038, 0.060, 0.085, 0.098, 0.112, 0.126, 0.142, 0.158];
const LPS_H: [f64; 12] = [0.117, 0.099, 0.082, 0.065, 0.048, 0.032, 0.016, 0.009, 0.002, -0.004, -0.011, -0.016];
const HPW_L: [f64; 12] = [-1.402, -1.613, -1.862, -2.158, -2.513, -2.938, -3.450, -3.744, -4.067, -4.422, -4.809, -5.235];
const HPW_H: [f64; 12] = [-0.902, -1.020, -1.153, -1.304, -1.473, -1.660, -1.859, -1.960, -2.059, -2.151, -2.232, -2.294];
const LPW_L: [f64; 12] = [-0.093, -0.060, -0.006, 0.077, 0.205, 0.402, 0.705, 0.913, 1.173, 1.499, 1.910, 2.431];
const LPW_H: [f64; 12] = [0.306, 0.308, 0.303, 0.289, 0.260, 0.212, 0.137, 0.085, 0.023, -0.052, -0.143, -0.251];

fn so_of(hp: bool, d: Demand) -> ShaftOfftake { if hp { ShaftOfftake::hp(d) } else { ShaftOfftake::lp(d) } }
fn m(so: Option<ShaftOfftake>, ml: ComponentMap, mh: ComponentMap) -> ShaftOfftakeMatcher {
    ShaftOfftakeMatcher::new(de(so), flight(), 1.0, ml, mh, 0.0)
}
fn rich(x0: f64, x1: f64, x2: f64) -> f64 { 2.0 * (x1 / x0).ln() / 0.01 - (x2 / x0).ln() / 0.02 }

fn sens(hp: bool, power: bool, ml: ComponentMap, mh: ComponentMap, tt4: f64) -> Option<(f64, f64, TwoSpoolMapResult)> {
    let mk = |x: f64| if power { Demand::power(x) } else { Demand::share(x) };
    let r0 = m(None, ml, mh).try_match_point(&flight(), tt4).ok()?.base;
    let r1 = m(Some(so_of(hp, mk(0.01))), ml, mh).try_match_point(&flight(), tt4).ok()?.base;
    let r2 = m(Some(so_of(hp, mk(0.02))), ml, mh).try_match_point(&flight(), tt4).ok()?.base;
    Some((rich(r0.phi_lp, r1.phi_lp, r2.phi_lp), rich(r0.phi_hp, r1.phi_hp, r2.phi_hp), r0))
}

fn crossings(v: &[(f64, f64)]) -> Vec<f64> {
    let mut out = Vec::new();
    for w in v.windows(2) {
        let ((t0, y0), (t1, y1)) = (w[0], w[1]);
        if (y0 < 0.0) != (y1 < 0.0) { out.push(t0 + (t1 - t0) * y0 / (y0 - y1)); }
    }
    out
}

fn part_a() {
    for (name, ml, mh) in [("flat", ComponentMap::flat(), ComponentMap::flat()),
                           ("flow/press", lp_shaped(), hp_shaped())] {
        println!("\n=== PART A — {name}");
        let fl = name == "flat";
        let mut maxdev = [0.0f64; 4];
        let mut lps_h = Vec::new();
        let mut lps_l = Vec::new();
        println!("Tt4 | HPshare L (model,diff) H (model,diff) | LPshare L H | HPpower L H | LPpower L H \
                  | A6 HP<LP both? | @0.05: HPs dF% dTSFC% | LPs dF% dTSFC% | HPw.05 share dF% | LPw.05 share dF%");
        for (i, &tt4) in BAND.iter().enumerate() {
            let hs = sens(true, false, ml, mh, tt4);
            let ls = sens(false, false, ml, mh, tt4);
            let hw = sens(true, true, ml, mh, tt4);
            let lw = sens(false, true, ml, mh, tt4);
            let f = |x: Option<(f64, f64, TwoSpoolMapResult)>| x.map(|(a, b, _)| (a, b));
            let (hs, ls, hw, lw) = (f(hs), f(ls), f(hw), f(lw));
            let fmt = |x: Option<(f64, f64)>, ml: f64, mh: f64| match x {
                Some((a, b)) if fl => format!("{a:+.4} ({ml:+.3},{:+.4}) {b:+.4} ({mh:+.3},{:+.4})", a - ml, b - mh),
                Some((a, b)) => format!("{a:+.4} {b:+.4}"),
                None => "FAIL".into(),
            };
            if fl {
                if let Some((a, b)) = hs { maxdev[0] = maxdev[0].max((a - HPS_L[i]).abs()); maxdev[1] = maxdev[1].max((b - HPS_H[i]).abs()); }
                if let Some((a, b)) = ls { maxdev[2] = maxdev[2].max((a - LPS_L[i]).abs()); maxdev[3] = maxdev[3].max((b - LPS_H[i]).abs()); }
            }
            if let Some((a, b)) = ls { lps_l.push((tt4, a)); lps_h.push((tt4, b)); }
            let a6 = match (hw, lw) { (Some((a, b)), Some((c, d))) => format!("{}", a < c && b < d), _ => "FAIL".into() };
            // Reported: thrust/TSFC at 0.05, and the power mode's floating share.
            let r0 = m(None, ml, mh).match_point(&flight(), tt4);
            let st0 = r0.base.base.thrust / r0.base.base.mdot_air;
            let tsfc0 = r0.base.base.performance.tsfc;
            let rep = |so: ShaftOfftake| match m(Some(so), ml, mh).try_match_point(&flight(), tt4) {
                Ok(r) => { let k = r.booking.unwrap();
                    format!("sh {:.3}/{:.3} {:+.2} {:+.2}", k.share_hp, k.share_lp, 100.0 * (k.st_inlet / st0 - 1.0),
                            100.0 * (k.tsfc_inlet / tsfc0 - 1.0)) }
                Err(e) => format!("ERR {}", &e.0[..e.0.len().min(30)]),
            };
            println!("{tt4:.0} | {} | {} | {} | {} | {a6} | {} | {} | {} | {}",
                     fmt(hs, HPS_L[i], HPS_H[i]), fmt(ls, LPS_L[i], LPS_H[i]),
                     fmt(hw, HPW_L[i], HPW_H[i]), fmt(lw, LPW_L[i], LPW_H[i]),
                     rep(ShaftOfftake::hp(Demand::share(0.05))), rep(ShaftOfftake::lp(Demand::share(0.05))),
                     rep(ShaftOfftake::hp(Demand::power(0.05))), rep(ShaftOfftake::lp(Demand::power(0.05))));
        }
        if fl {
            println!("max |meas-model| share: HP L {:.4} H {:.4}; LP L {:.4} H {:.4}  (bar 0.01)", maxdev[0], maxdev[1], maxdev[2], maxdev[3]);
        }
        println!("LP-shaft HP crossings {:?} (A4 bar 783 ± 75)", crossings(&lps_h));
        println!("LP-shaft LP crossings {:?} (A5 bar 1290 ± 55)", crossings(&lps_l));
    }
}

fn machine(so: Option<ShaftOfftake>, ml: ComponentMap, mh: ComponentMap, arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(de(so), flight(), 1.0, Some(ml.with_phi_surge(0.55)), Some(mh.with_phi_surge(0.55)), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn part_b() {
    println!("\n=== PART B — rung 64's rig, ds 0.005, Tt4 1000->1400, settle 1.2, b_max 0.10");
    let demands: [(&str, Option<ShaftOfftake>); 3] = [
        ("none", None),
        ("HPw.02", Some(ShaftOfftake::hp(Demand::power(0.02)))),
        ("LPw.02", Some(ShaftOfftake::lp(Demand::power(0.02)))),
    ];
    for (shape, ml, mh) in [("shaped", lp_shaped(), hp_shaped()), ("tilted", tilt(), tilt())] {
        for r in [0.25, 0.5, 1.0] {
            let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds: 0.005 };
            for (dn, so) in demands {
                let base = machine(so, ml, mh, &LeverArm::default());
                let shut = base.bill_cell(&flight(), &ramp, true);
                let tr = shut.traj.as_ref().unwrap();
                let (n0, n1) = (tr[0].nu_hp, tr[tr.len() - 1].nu_hp);
                let target = n0 + 0.9 * (n1 - n0);
                let mut t90 = f64::NAN;
                for w in tr.windows(2) {
                    if w[0].nu_hp < target && w[1].nu_hp >= target {
                        t90 = w[0].s + (w[1].s - w[0].s) * (target - w[0].nu_hp) / (w[1].nu_hp - w[0].nu_hp);
                        break;
                    }
                }
                let full = base.at_lever(&LeverArm::constant(0.10)).bill_cell(&flight(), &ramp, false);
                let mut line = format!("{shape} r={r} {dn:7} | shut minphiL {:.6} minphiH {:.6} npts {} | nuH {n0:.5}->{n1:.5} t90 {t90:.4} | ceiling {:.6} npts {} |",
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
