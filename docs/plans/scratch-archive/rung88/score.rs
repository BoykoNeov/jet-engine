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

fn main() {
    let ds: f64 = std::env::args().nth(1).map(|x| x.parse().unwrap()).unwrap_or(0.005);
    let fl = flight();
    let cells: [(&str, f64, f64, f64, bool); 21] = [
        ("shaped",0.25,0.34,0.2609,true),("shaped",0.25,0.38,0.3038,true),("shaped",0.25,0.42,0.3464,true),("shaped",0.5,0.38,0.2876,false),
        ("shaped",0.5,0.42,0.3447,true),("shaped",0.5,0.46,0.3896,true),("shaped",0.5,0.50,0.4344,true),("shaped",1.0,0.42,0.3223,true),
        ("shaped",1.0,0.46,0.3753,true),("shaped",1.0,0.50,0.4265,true),("shaped",1.0,0.54,0.4798,true),("tilted",0.25,0.34,0.2574,true),
        ("tilted",0.25,0.38,0.2993,true),("tilted",0.25,0.42,0.3424,true),("tilted",0.5,0.42,0.3391,true),("tilted",0.5,0.46,0.3848,true),
        ("tilted",0.5,0.50,0.4289,true),("tilted",1.0,0.42,0.3126,false),("tilted",1.0,0.46,0.3683,true),("tilted",1.0,0.50,0.4191,true),("tilted",1.0,0.54,0.4728,true)];
    let ms = bare(lp_map(), hp_map());
    let mt = bare(tilt_map(), tilt_map());
    let st = stat();
    // tallies: (hits, scored)
    let (mut h1, mut h2, mut h3, mut h4, mut h5) = ((0, 0), (0, 0), (0, 0), (0, 0), (0, 0));
    let mut h5_unbracketed = 0;
    let mut precond_bad = 0;
    for (shape, r, sm, smx, scored) in cells {
        let m = if shape == "shaped" { &ms } else { &mt };
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
        let t_c = m.design_map(Spool::Lp).tan_beta1_crit();
        let m_lim = t_c - 1.0 / phi_of(sm);
        let tag = format!("{shape} r={r} sm={sm:.2}{}", if scored { "" } else { " [band]" });
        // instantaneous references
        let mi = sibling(m, Some(&st), Some(BleedLimiter::new(phi_of(sm), BMAX).on_incidence()));
        let ti = march(&mi, &fl, &ramp, None).0; let ri = read(&mi, &fl, &ti); let bi = bleed(&mi, &ti);
        let mp = sibling(m, Some(&st), Some(BleedLimiter::new(phi_of(smx), BMAX)));
        let tp = march(&mp, &fl, &ramp, None).0; let bp = bleed(&mp, &tp);
        let b0p_inst = bp[0]; let b0i_inst = bi[0];
        println!("CELL {tag}: m_lim={m_lim:.6} inc inst Mi={:.6} bpk={:.4e} b0={b0i_inst:.4e} n={} | phi* sm*={smx} b0={b0p_inst:.4e} n={}",
                 ri.m_i, ri.b_peak, ti.len(), tp.len());
        let mut prev_mi = f64::INFINITY; let mut h3_ok = true;
        let mut h1_ok = true; let mut h2_ok = true; let mut h4_ok = true;
        for tau in TAUS {
            let ml = sibling(m, Some(&st), Some(BleedLimiter::with_tau(phi_of(sm), BMAX, Some(tau)).on_incidence()));
            let (tl, li): (Vec<FuelPoint>, LagRead) = run_lagged(&ml, &fl, &ramp);
            let mpl = sibling(m, Some(&st), Some(BleedLimiter::with_tau(phi_of(smx), BMAX, Some(tau))));
            let (tpl, lp) = run_lagged(&mpl, &fl, &ramp);
            let ni = tl.len().min(ti.len());
            let np = tpl.len().min(tp.len());
            let cmd_i: Vec<f64> = tl.iter().map(|p| valve_at(p).1).collect();
            let cmd_p: Vec<f64> = tpl.iter().map(|p| valve_at(p).1).collect();
            let bl_i: Vec<f64> = tl.iter().map(|p| valve_at(p).0).collect();
            let bl_p: Vec<f64> = tpl.iter().map(|p| valve_at(p).0).collect();
            let drift_i = trapv(&tl, &cmd_i, ni) - trapv(&ti, &bi, ni);
            let drift_p = trapv(&tpl, &cmd_p, np) - trapv(&tp, &bp, np);
            let dbi = trapv(&tl, &bl_i, ni) - trapv(&ti, &bi, ni);
            let dbp = trapv(&tpl, &bl_p, np) - trapv(&tp, &bp, np);
            let pre = li.clamp_hits == 0 && lp.clamp_hits == 0;
            if !pre { precond_bad += 1; }
            let bar1 = (0.05 * tau * ri.b_peak).max(1e-5);
            let ok1 = drift_i.abs() <= bar1;
            let strict = b0p_inst > b0i_inst;
            let ok2 = dbi < dbp;
            let ok3a = li.run.m_i < m_lim;
            let ok3b = li.run.m_i < prev_mi;
            prev_mi = li.run.m_i;
            let ok4 = li.cmd_rising_at_m_i == Some(true);
            if scored && pre {
                h1_ok &= ok1; if strict { h2_ok &= ok2; } h3_ok &= ok3a && ok3b; h4_ok &= ok4;
            }
            // H5: match the lagged phi valve to the lagged incidence valve's M_i
            let h5s = match matched_phi_lagged(m, &fl, &ramp, Some(&st), BMAX, Some(tau), li.run.m_i, 1e-12) {
                None => { if scored { h5_unbracketed += 1; } "unbracketed".to_string() }
                Some((phx, tx, rx)) => {
                    let n = tx.len().min(tl.len());
                    let bx: Vec<f64> = tx.iter().map(|p| valve_at(p).0).collect();
                    let (bb_x, bb_i) = (trapv(&tx, &bx, n), trapv(&tl, &bl_i, n));
                    let (f_x, f_i) = (thrust(&tx, n), thrust(&tl, n));
                    let ok = bb_x > bb_i && f_x < f_i;
                    let dmi = rx.m_i - li.run.m_i;
                    if scored && dmi.abs() < 1e-7 { h5.1 += 1; if ok { h5.0 += 1; } }
                    format!("matched phi_lim={phx:.5} (sm {:.4}) dMi={dmi:.1e} bint {bb_x:.4e}/{bb_i:.4e}={:.3} Fint phi-inc={:.3} n={n}/{}/{} {}",
                            phx / 0.55 - 1.0, bb_x / bb_i, f_x - f_i, tx.len(), tl.len(), if ok { "OK" } else { "FAIL" })
                }
            };
            println!("  tau={tau}: INC n={} Mi={:.6}@{:.3}(rising={:?}) dMi_vs_mlim={:.3e} b0={:.3e} bend={:.1e} clamp={} idres={:.1e} dbint={dbi:.3e} drift={drift_i:.3e} bar1={bar1:.1e} [H1 {}] [H3 {}] [H4 {}]",
                     tl.len(), li.run.m_i, li.run.s_at_m_i, li.cmd_rising_at_m_i, li.run.m_i - m_lim, li.b0, li.b_end, li.clamp_hits, li.identity_residual,
                     if ok1 { "ok" } else { "MISS" }, if ok3a && ok3b { "ok" } else { "MISS" }, if ok4 { "ok" } else { "MISS" });
            println!("           PHI* n={} Mi={:.6} b0={:.3e} bend={:.1e} clamp={} idres={:.1e} dbint={dbp:.3e} drift={drift_p:.3e} [H2 {}] | H5: {h5s}",
                     tpl.len(), lp.run.m_i, lp.b0, lp.b_end, lp.clamp_hits, lp.identity_residual,
                     if !strict { "n/a" } else if ok2 { "ok" } else { "MISS" });
        }
        if scored {
            h1.1 += 1; if h1_ok { h1.0 += 1; }
            if b0p_inst > b0i_inst { h2.1 += 1; if h2_ok { h2.0 += 1; } }
            h3.1 += 1; if h3_ok { h3.0 += 1; }
            h4.1 += 1; if h4_ok { h4.0 += 1; }
        }
    }
    println!("SUMMARY ds={ds}: H1 {}/{} cells  H2 {}/{} cells  H3 {}/{} cells  H4 {}/{} cells  H5 {}/{} (cell,tau) matched, {} unbracketed; runs breaking D1's precondition: {}",
             h1.0, h1.1, h2.0, h2.1, h3.0, h3.1, h4.0, h4.1, h5.0, h5.1, h5_unbracketed, precond_bad);
}
