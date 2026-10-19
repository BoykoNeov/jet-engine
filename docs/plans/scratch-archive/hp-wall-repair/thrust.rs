use turbojet::components::{ram_recovery, try_choked_mfp, Nozzle};
use turbojet::engine::FlightCondition;
use turbojet::gas::{powp, Abort, FlowState, Gas};
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::control_maps;
use turbojet::two_spool::{hp_eta_loop_closed, lp_eta_loop_arrow, TwoSpoolCore, TwoSpoolMapCore, TwoSpoolMapMatcher};
use turbojet::two_spool_transient::TwoSpoolTransientCore;
use turbojet::unchoked_lp::{try_lp_turbine, PI_TOP};
use turbojet::atmosphere::standard;
use turbojet::sandbox_controls::{ControlsSettings, CONTROL_SHAPES};
use turbojet::visuals::Json;
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
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let want: Vec<&str> = vec!["180", "281", "318", "371", "418", "547", "828", "906"];
    let bs = BladeSettings::defaults();
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if !want.contains(&f[0]) { continue; }
        let Json::Obj(top) = Json::parse(f[1]) else { panic!() };
        let c = top.iter().find(|(k, _)| k == "controls").unwrap().1.clone();
        let s = ControlsSettings::from_json(&c).unwrap();
        let reason = f[2];
        let at = reason.find("throttle (").map(|k| &reason[k + 10..]).unwrap();
        let tt4: f64 = at[..at.find(' ').unwrap()].parse().unwrap();
        let (ml, mh) = control_maps(CONTROL_SHAPES[s.shape]).unwrap();
        let mm = TwoSpoolMapMatcher::new(bs.design(), bs.flight(), 1.0, ml, mh);
        let fl = s.flight();
        let r = joint(mm.core(), &fl, tt4, &|b, g, tt45, f, eta, pt45| try_lp_turbine(b, g, tt45, f, eta, pt45, fl.p0));
        match r { Ok(p) => println!("{} {} {tt4} K M{:.2}: specific thrust {:.2} N/(kg/s), M9 {:.3}", f[0], CONTROL_SHAPES[s.shape], fl.m0, p.sp, p.m9),
                  Err(e) => println!("{} {tt4} K: ERR {}", f[0], &e.0[..e.0.len().min(80)]) }
    }
}
