// Rung 103 pre-anchor probe: the STEADY two-shaft map matcher (rung 39) below the nozzle's unchoke boundary.
// For each (engine, flight, map, Tt4):
//   OLD  — rung 39's shipped match (ok / its refusal);
//   PIN  — the same joint loop with the guard bypassed (the "old model": rung 38's pin, numbers it would have printed);
//   A    — rung-33-shaped: pi_LPT an OUTER unknown, the joint (f, pt4) loop closed at each trial; resid scanned for roots;
//   B    — rung 97's helper inside the joint loop (pt45 from the current iterate);
//   TR   — rung 40's transient equilibrium (rung 97's branch) at the same Tt4 — the independent reference.
use turbojet::components::{ram_recovery, try_choked_mfp, Nozzle};
use turbojet::engine::FlightCondition;
use turbojet::gas::{powp, Abort, FlowState, Gas};
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::control_maps;
use turbojet::two_spool::{hp_eta_loop_closed, lp_eta_loop_arrow, TwoSpoolCore, TwoSpoolMapCore, TwoSpoolMapMatcher};
use turbojet::two_spool_transient::TwoSpoolTransientCore;
use turbojet::unchoked_lp::{try_lp_turbine, PI_TOP};
use turbojet::atmosphere::standard;

#[derive(Clone, Copy, Debug, Default)]
struct Pt { pi_lpt: f64, pi_lpc: f64, pi_hpc: f64, nl: f64, nh: f64, f: f64, pt4: f64, mdot: f64, sp: f64,
            resid: f64, passes: usize, capped: bool, p9: f64, m9: f64 }

/// One pass of rung 39's map cascade with the LP turbine given by `lp` (pin, fixed pi, or helper).
fn cascade(core: &TwoSpoolMapCore, wgas: &Gas, tt2: f64, tt4: f64, f: f64, pt4: f64,
           lp: &dyn Fn(&TwoSpoolCore, &Gas, f64, f64, f64, f64) -> Result<(f64, f64, f64), Abort>)
    -> Result<(f64, f64, f64, f64, f64, f64, f64, f64, f64), Abort>
{
    let b = &core.base;
    let mfp4 = try_choked_mfp(wgas, tt4, f)?;
    let (mut eta_hpt, mut eta_lpt) = (b.eta_hpt, b.eta_lpt);
    for _ in 0..60 {
        let (pi_hpt, _tau_hpt, tt45) = b.try_solve_choked_turbine(wgas, tt4, f, b.a4, b.a45, 1.0, eta_hpt)?;
        let (pi_lpt, _tau_lpt, tt5) = lp(b, wgas, tt45, f, eta_lpt, pi_hpt * pt4)?;
        let dh_lpt = b.eta_m * (1.0 + f) * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
        let tt25 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_lpt);
        let dh_hpt = b.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
        let tt3 = wgas.t_from_h_c(wgas.h_c(tt25) + dh_hpt);
        let mh = core.map_hp();
        let ml = core.map_lp();
        let hp = hp_eta_loop_closed(wgas, tt4, f, tt25, tt3, mfp4, &mh, b.eta_hpc, b.a4, b.pi_b, core.mcorr_hp_d, core.tau_hpc_d)?;
        let lpl = lp_eta_loop_arrow(wgas, tt2, tt4, f, tt25, mfp4, hp.pi, &ml, b.eta_lpc, b.a4, b.pi_b, core.mcorr_lp_d, core.tau_lpc_d)?;
        let nl = lpl.n * powp(tt2 / core.tt2_d, 0.5);
        let nh = hp.n * powp(tt25 / core.tt25_d, 0.5);
        let t_hpt = mh.eta_t_at(b.eta_hpt, nh * powp(core.tt4_d / tt4, 0.5));
        let t_lpt = ml.eta_t_at(b.eta_lpt, nl * powp(core.tt45_d / tt45, 0.5));
        if (t_hpt - eta_hpt).abs() <= 1e-11 && (t_lpt - eta_lpt).abs() <= 1e-11 {
            return Ok((pi_hpt, tt45, pi_lpt, tt5, lpl.pi, hp.pi, tt3, nl, nh));
        }
        eta_hpt = t_hpt;
        eta_lpt = t_lpt;
    }
    Err(Abort("turbine eta loop".into()))
}

fn joint(core: &TwoSpoolMapCore, fl: &FlightCondition, tt4: f64,
         lp: &dyn Fn(&TwoSpoolCore, &Gas, f64, f64, f64, f64) -> Result<(f64, f64, f64), Abort>) -> Result<Pt, Abort> {
    let b = &core.base;
    let pi_d = b.pi_d_max * ram_recovery(fl.m0);
    let (s0, v0) = b.try_freestream_for(fl)?;
    let (tt2, pt2) = (s0.tt, pi_d * s0.pt);
    let (mut f, mut pt4) = (b.f_design, b.pi_b * b.pi_hpc_design * b.pi_lpc_design * pt2);
    let mut last = None;
    let mut passes = 0;
    let mut capped = true;
    for k in 0..TwoSpoolCore::MAX {
        let owned = b.try_working_gas(f, tt4, pt4)?;
        let wgas = owned.as_ref().unwrap_or(b.gas());
        let c = cascade(core, wgas, tt2, tt4, f, pt4, lp)?;
        let pt4n = b.pi_b * c.5 * c.4 * pt2;
        let fnew = b.try_solve_f(c.6, pt4n, tt4)?;
        let done = (fnew - f).abs() <= TwoSpoolCore::TOL * (fnew + 1e-30) && (pt4n - pt4).abs() <= TwoSpoolCore::TOL * pt4n;
        last = Some(c);
        f = fnew;
        pt4 = pt4n;
        passes = k + 1;
        if done { capped = false; break; }
    }
    let c = last.unwrap();
    let owned = b.try_working_gas(f, tt4, pt4)?;
    let wgas = owned.as_ref().unwrap_or(b.gas());
    let mdot4 = b.a4 * pt4 * try_choked_mfp(wgas, tt4, f)? / powp(tt4, 0.5);
    // the nozzle at the solved LP exit
    let pt45 = c.0 * pt4;
    let ex = Nozzle::convergent(fl.p0, b.pi_n).try_apply(&FlowState { tt: c.3, pt: c.2 * pt45, mdot: 1.0, far: f }, wgas)?;
    let r = wgas.r_t_at(f);
    let supply = b.a45 * pt45 * try_choked_mfp(wgas, c.1, f)? / powp(c.1, 0.5);
    let demand = b.a8 * ex.p9 / (r * ex.t9) * ex.v9;
    let press = (1.0 + f) * r * ex.t9 * (1.0 - fl.p0 / ex.p9) / ex.v9;
    let sp = (1.0 + f) * ex.v9 - v0 + press;
    Ok(Pt { pi_lpt: c.2, pi_lpc: c.4, pi_hpc: c.5, nl: c.7, nh: c.8, f, pt4, mdot: mdot4 / (1.0 + f), sp,
            resid: (supply - demand) / supply, passes, capped, p9: ex.p9, m9: ex.m9 })
}

fn main() {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let (t_sl, p_sl) = standard(0.0);
    let flights = [("SL+40 M0.85", FlightCondition::new(t_sl + 40.0, p_sl, 0.85)),
                   ("SL std M0.85", FlightCondition::new(t_sl, p_sl, 0.85)),
                   ("SL std M0.30", FlightCondition::new(t_sl, p_sl, 0.30)),
                   ("design flight", fd)];
    println!("design flight T0={:.2} p0={:.0} M0={}", fd.t0, fd.p0, fd.m0);
    for shape in ["flow/press", "press/flow", "tilted", "flat-lp"] {
        let (ml, mh) = control_maps(shape).unwrap();
        let mm = TwoSpoolMapMatcher::new(bs.design(), fd, 1.0, ml, mh);
        let core = mm.core();
        let tr = TwoSpoolTransientCore::new(bs.design(), fd, 1.0, ml, mh, 1.0);
        for (fname, fl) in flights.iter() {
            println!("\n=== {shape} | {fname}");
            let mut tt4 = 550.0;
            while tt4 <= 1150.0 + 1e-9 {
                let old = core.try_match_point(fl, tt4);
                let oldw = match &old { Ok(_) => "ok".to_string(),
                    Err(e) if e.0.contains("UNCHOKED") => "UNCHOKED".into(), Err(e) => format!("ERR {}", &e.0[..e.0.len().min(60)]) };
                let pin = joint(core, fl, tt4, &|b, g, tt45, f, eta, _pt45| b.try_solve_choked_turbine(g, tt45, f, b.a45, b.a8, b.pi_n, eta));
                let hb = joint(core, fl, tt4, &|b, g, tt45, f, eta, pt45| try_lp_turbine(b, g, tt45, f, eta, pt45, fl.p0));
                let teq = tr.try_equilibrium(fl, tt4, None);
                print!("Tt4 {tt4:5.0} OLD {oldw:9}");
                match &pin { Ok(p) => print!(" | PIN pi_lpt {:.6} nl {:.6} sp {:7.2} p9/p0 {:.4} pass {}{}", p.pi_lpt, p.nl, p.sp, p.p9 / fl.p0, p.passes, if p.capped {"C"} else {""}),
                             Err(e) => print!(" | PIN ERR {}", &e.0[..e.0.len().min(50)]) }
                match &hb { Ok(p) => print!(" | B pi_lpt {:.6} nl {:.6} sp {:7.2} M9 {:.4} pass {}{} res {:.1e}", p.pi_lpt, p.nl, p.sp, p.m9, p.passes, if p.capped {"C"} else {""}, p.resid),
                            Err(e) => print!(" | B ERR {}", &e.0[..e.0.len().min(50)]) }
                match &teq { Ok((i, ex, it)) => print!(" | TR pi_lpt {:.6} nl {:.6} nh {:.6} sp {:7.2} {:?}/{}", i.pi_lpt, i.close.n_lp * powp(i.close.tt2 / core.tt2_d, 0.5), i.nu_hp, i.sp_thrust, ex, it),
                             Err(e) => print!(" | TR ERR {}", &e.0[..e.0.len().min(50)]) }
                if let (Ok(o), Ok(p)) = (&old, &pin) {
                    print!(" | OLD-PIN dpi_lpc {:.1e} dnl {:.1e}", o.base.pi_lpc - p.pi_lpc, o.n_lp_ratio - p.nl);
                }
                println!();
                // A: scan resid over pi_LPT at the unchoked cells (and one choked for contrast)
                if let (Err(_), Ok(p)) = (&old, &pin) {
                    let lo = p.pi_lpt;
                    let n = 24;
                    let mut prev: Option<f64> = None;
                    let mut roots = 0;
                    let mut line = String::new();
                    let mut caps = 0;
                    let mut maxpass = 0;
                    let mut errs = 0;
                    for k in 0..=n {
                        let pi = lo + (PI_TOP - lo) * k as f64 / n as f64;
                        match joint(core, fl, tt4, &|b, g, tt45, f, eta, _| { let (t, t5) = b.try_tau_of(g, tt45, f, pi, eta)?; Ok((pi, t, t5)) }) {
                            Ok(a) => {
                                if let Some(pr) = prev { if pr * a.resid < 0.0 { roots += 1; } }
                                prev = Some(a.resid);
                                if a.capped { caps += 1; }
                                maxpass = maxpass.max(a.passes);
                                if k % 4 == 0 { line += &format!(" {pi:.4}:{:+.3e}", a.resid); }
                            }
                            Err(_) => { errs += 1; if k % 4 == 0 { line += &format!(" {pi:.4}:ERR"); } }
                        }
                    }
                    println!("      A-scan roots {roots} caps {caps} maxpass {maxpass} errs {errs} |{line}");
                }
                tt4 += 50.0;
            }
        }
    }
}
