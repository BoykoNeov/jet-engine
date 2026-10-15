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
use turbojet::three_levers::{ArmedLimiter, Limiter};
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
    let (mut n_on, mut n_onset_earlier, mut n_surplus, mut tot) = (0, 0, 0, 0);
    for (shape, r, sm) in cells {
        let m = if shape == "shaped" { &ms } else { &mt };
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
        let t_c = m.design_map(Spool::Lp).tan_beta1_crit();
        let m_lim = t_c - 1.0 / phi_of(sm);
        let (tt2, _, _) = m.fuel.inner.inlet(&fl);
        let mut bphi = Vec::new();
        for tau in [None, Some(0.01), Some(0.02), Some(0.05), Some(0.1), Some(0.2)] {
            let (xi, ti, ri) = matched_lagged(m, &fl, &ramp, Some(&st), BMAX, tau, ValveReference::Incidence, m_lim, 1e-12).unwrap();
            let (xp, tp, _) = matched_lagged(m, &fl, &ramp, Some(&st), BMAX, tau, ValveReference::Phi, m_lim, 1e-12).unwrap();
            let mk = |x: f64, inc: bool| { let v = BleedLimiter::with_tau(x, BMAX, tau); sibling(m, Some(&st), Some(if inc { v.on_incidence() } else { v })) };
            let (mi, mp) = (mk(xi, true), mk(xp, false));
            let bi: Vec<f64> = ti.iter().map(|q| mi.b_at_point(&fl, q)).collect();
            let bp: Vec<f64> = tp.iter().map(|q| mp.b_at_point(&fl, q)).collect();
            bphi.push(trapv(&tp, &bp, tp.len()));
            if let Some(t) = tau { if t >= 0.05 {
                tot += 1;
                let on_i = (0..bi.len()).find(|&k| bi[k] > 0.0).map(|k| ti[k].s).unwrap_or(f64::INFINITY);
                let on_p = (0..bp.len()).find(|&k| bp[k] > 0.0).map(|k| tp[k].s).unwrap_or(f64::INFINITY);
                if on_p < on_i { n_onset_earlier += 1; }
                if on_p <= on_i { n_on += 1; }
                // the phi valve's incidence surplus at the point where the slow incidence valve binds
                let j = tp.iter().position(|q| (q.s - ri.s_at_m_i).abs() < 1e-9).unwrap();
                let v = mp.v_of(Spool::Lp, tp[j].nu_lp, tp[j].nu_hp, Some(tt2));
                let surplus = t_c - 1.0 / tp[j].phi_lp + v - m_lim;
                if surplus > 0.0 { n_surplus += 1; }
                println!("DISC {shape} r={r} sm={sm} tau={t}: onset inc {on_i:.3} phi {on_p:.3} | phi surplus at inc's min (s={:.3}) {surplus:.3e}", ri.s_at_m_i);
            }}
        }
        println!("PHIBLEED {shape} r={r} sm={sm}: {}", bphi.iter().map(|b| format!("{:.4e}({:.3})", b, b / bphi[0])).collect::<Vec<_>>().join(" "));
        // rung 87's H5 with a slow valve: incidence valve beside a phi fuel floor at the same sm, stators on
        if ds == 0.005 {
            let bare_m = sibling(m, None, None);
            let lim = ArmedLimiter::on(&bare_m, &fl, &ramp, Spool::Lp, Limiter::PhiFloor { sm });
            let mut s = String::new();
            for tau in [None, Some(0.05), Some(0.2)] {
                let mm = sibling(m, Some(&st), Some({ let v = BleedLimiter::with_tau(phi_of(sm), BMAX, tau); v.on_incidence() }));
                let mm2 = sibling(m, Some(&st), Some({ let v = BleedLimiter::with_tau(phi_of(sm), BMAX, tau); v.on_incidence() }));
                let _ = mm2;
                let tr = march(&mm, &fl, &ramp, Some(&lim)).0;
                let cut = tr.iter().map(|p| p.mf_sched - p.mf).fold(0.0, f64::max);
                s += &format!(" tau={tau:?}: n={} maxcut={cut:.2e} |", tr.len());
            }
            println!("H5of87 {shape} r={r} sm={sm} full={}:{s}", ((r + 1.2) / ds).round() as usize + 1);
        }
    }
    println!("SUMMARY ds={ds}: tau>=.05 runs {tot}: phi onset earlier {n_onset_earlier}, earlier-or-same {n_on}; phi has incidence surplus at the incidence valve's min {n_surplus}");
}
