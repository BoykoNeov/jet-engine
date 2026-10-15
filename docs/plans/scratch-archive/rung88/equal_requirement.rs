// Rung 88 scoring of anchor section 3 (scratch). Arg 1: ds. Reads only; the bars were frozen at 81831f4.
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::FuelPoint;
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::lagged_incidence::{matched_phi_lagged, run_lagged, trapz, valve_at, LagRead};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use turbojet::three_levers::march;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::valve_reference::{read, sibling, ValveRun};

const REAL: TwoSpoolLosses = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn cpg() -> Gas { Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
    gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() }) }
fn lp_map() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(0.55) }
fn hp_map() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(0.55) }
fn tilt_map() -> ComponentMap { ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }.with_phi_surge(0.55) }
fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL) }
fn bare(lp: ComponentMap, hp: ComponentMap) -> ScheduledStatorCore {
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c, _ => unreachable!() } }
const BMAX: f64 = 0.10;
fn stat() -> LeverArm { LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65))) }
fn phi_of(sm: f64) -> f64 { 0.55 * (1.0 + sm) }
const TAUS: [f64; 5] = [0.01, 0.02, 0.05, 0.1, 0.2];

/// Bleed at each point through the machine's own b_at_point (recorded if lagged, re-solved if not).
fn bleed(m: &ScheduledStatorCore, t: &[FuelPoint]) -> Vec<f64> { t.iter().map(|p| m.b_at_point(&flight(), p)).collect() }
fn trapv(t: &[FuelPoint], v: &[f64], n: usize) -> f64 {
    let n = n.min(t.len()); let mut a = 0.0;
    for i in 1..n { a += 0.5 * (t[i].s - t[i - 1].s) * (v[i] + v[i - 1]); } a }
fn thrust(t: &[FuelPoint], n: usize) -> f64 { trapz(t, n, |p| p.sp_thrust * p.mdot_air) }

use turbojet::lagged_incidence::matched_lagged;
use turbojet::limited_bleed::ValveReference;
use turbojet::lagged_bleed::Census65;
fn main() {
    let ds: f64 = std::env::args().nth(1).map(|x| x.parse().unwrap()).unwrap_or(0.005);
    let fl = flight();
    let cells: [(&str, f64, f64); 19] = [
        ("shaped",0.25,0.34),("shaped",0.25,0.38),("shaped",0.25,0.42),("shaped",0.5,0.42),("shaped",0.5,0.46),("shaped",0.5,0.50),
        ("shaped",1.0,0.42),("shaped",1.0,0.46),("shaped",1.0,0.50),("shaped",1.0,0.54),("tilted",0.25,0.34),("tilted",0.25,0.38),
        ("tilted",0.25,0.42),("tilted",0.5,0.42),("tilted",0.5,0.46),("tilted",0.5,0.50),("tilted",1.0,0.46),("tilted",1.0,0.50),("tilted",1.0,0.54)];
    let ms = bare(lp_map(), hp_map());
    let mt = bare(tilt_map(), tilt_map());
    let st = stat();
    for (shape, r, sm) in cells {
        let m = if shape == "shaped" { &ms } else { &mt };
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
        let m_lim = m.design_map(Spool::Lp).tan_beta1_crit() - 1.0 / phi_of(sm);
        println!("REQ {shape} r={r} sm={sm:.2} m_lim={m_lim:.6}");
        for tau in [None, Some(0.01), Some(0.02), Some(0.05), Some(0.1), Some(0.2)] {
            let mut line = format!("  tau={tau:?}:");
            let mut bints = [0.0; 2]; let mut fints = [0.0; 2]; let mut ns = [0usize; 2]; let mut ok = true;
            let mut trajs: Vec<Vec<FuelPoint>> = Vec::new(); let mut bs: Vec<Vec<f64>> = Vec::new();
            for (k, rf) in [ValveReference::Incidence, ValveReference::Phi].into_iter().enumerate() {
                match matched_lagged(m, &fl, &ramp, Some(&st), BMAX, tau, rf, m_lim, 1e-12) {
                    None => { ok = false; line += &format!(" {rf:?} UNBRACKETED |"); }
                    Some((x, t, rr)) => {
                        let v = BleedLimiter::with_tau(x, BMAX, tau);
                        let mm = sibling(m, Some(&st), Some(if k == 0 { v.on_incidence() } else { v }));
                        Census65::reset();
                        let t2 = march(&mm, &fl, &ramp, None).0;
                        let clamps = Census65::take().clamp_hits;
                        assert_eq!(t2.len(), t.len());
                        let b: Vec<f64> = t.iter().map(|q| mm.b_at_point(&fl, q)).collect();
                        let j = t.iter().position(|q| q.s == rr.s_at_m_i).unwrap();
                        let lagstr = match t[j].extra { turbojet::fuel_transient::PointExtra::Valve { b: bb, b_cmd } => format!(" b-bcmd@min={:.2e}", bb - b_cmd), _ => String::new() };
                        let sat = b.iter().filter(|&&x| x >= BMAX - 1e-12).count();
                        line += &format!(" {} sm {:.4} dMi {:.0e} min@s={:.3} v={:.4} b@min={:.4}{lagstr} sat={sat} clamp={clamps} n={} |",
                                         if k == 0 { "INC" } else { "PHI" }, x / 0.55 - 1.0, rr.m_i - m_lim, rr.s_at_m_i, rr.v_at_m_i, b[j], t.len());
                        ns[k] = t.len(); trajs.push(t); bs.push(b);
                    }
                }
            }
            if ok {
                let n = ns[0].min(ns[1]);
                for k in 0..2 {
                    let t = &trajs[k];
                    bints[k] = trapv(t, &bs[k], n);
                    fints[k] = thrust(t, n);
                }
                line += &format!(" ratio phi/inc={:.3} dF(phi-inc)={:.3}", bints[1] / bints[0], fints[1] - fints[0]);
            }
            println!("{line}");
        }
    }
}
