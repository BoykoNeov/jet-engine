// G3: symmetric only. (a) exact monotone-segment count of each re-tuned valve's recorded position (one rise,
// one fall?); (b) the phi valve's FLOOR: its set point re-tuned so the minimum of M_i over the points BEFORE
// its first closing read equals m_lim (the infinitely-slow-close limit, D2's untouched part only);
// (c) the forecast R_f(tau_o, tau_c) of D4 at every off-diagonal pair, from the symmetric runs. Arg 1: ds.
use r89probe::*;
use turbojet::fuel_transient::FuelPoint;
use turbojet::lagged_bleed::{first_closing_read, Census65};
use turbojet::lagged_incidence::{lag_read, matched_lagged, trapz, valve_at};
use turbojet::limited_bleed::{BleedLimiter, ValveReference};
use turbojet::spool::{try_illinois, ILLINOIS_MAXIT};
use turbojet::stator_transient::{Ramp, ScheduledStatorCore};
use turbojet::three_levers::march;
use turbojet::two_spool::Spool;
use turbojet::valve_reference::{read, sibling};

fn segments(t: &[FuelPoint]) -> (usize, usize) {
    // count maximal strictly-rising and strictly-falling runs of the recorded position (flat steps skipped)
    let (mut rises, mut falls, mut last) = (0, 0, 0i8);
    for w in t.windows(2) {
        let d = valve_at(&w[1]).0 - valve_at(&w[0]).0;
        let sgn = if d > 0.0 { 1 } else if d < 0.0 { -1 } else { 0 };
        if sgn != 0 && sgn != last { if sgn > 0 { rises += 1 } else { falls += 1 } last = sgn; }
    }
    (rises, falls)
}

fn pre_close_min(m: &ScheduledStatorCore, ramp: &Ramp, x: f64, tt2: f64, t_c: f64) -> (f64, f64) {
    let mm = sibling(m, Some(&stat()), Some(BleedLimiter::with_tau(x, BMAX, Some(ramp_tau()))));
    Census65::reset();
    let t = march(&mm, &flight(), ramp, None).0;
    let fc = first_closing_read();
    let mut pre = f64::INFINITY;
    for p in &t { if p.s < fc { pre = pre.min(t_c - 1.0 / p.phi_lp + mm.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2))); } }
    (pre, fc)
}
thread_local! { static TAU: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) }; }
fn ramp_tau() -> f64 { TAU.with(|c| c.get()) }

fn main() {
    let ds: f64 = std::env::args().nth(1).map(|x| x.parse().unwrap()).unwrap_or(0.005);
    let fl = flight();
    let ms = bare(lp_map(), hp_map());
    let mt = bare(tilt_map(), tilt_map());
    let st = stat();
    let (tt2, _, _) = ms.fuel.inner.inlet(&fl);
    let mut not_single = 0; let mut nruns = 0;
    let mut rf_cross = 0; let mut rf_n = 0;
    for (shape, r, sm) in CELLS {
        let m = if shape == "shaped" { &ms } else { &mt };
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
        let t_c = m.design_map(Spool::Lp).tan_beta1_crit();
        let m_lim = t_c - 1.0 / phi_of(sm);
        // per tau_o: (A_inc, A_phi, D_inc, D_phi, both-min-first)
        let mut rows = Vec::new();
        for tau in TAUS {
            TAU.with(|c| c.set(tau));
            let mut a = [0.0; 2]; let mut d = [0.0; 2]; let mut first = [false; 2]; let mut sms = [0.0; 2];
            let mut trajs = Vec::new();
            for (k, rf) in [ValveReference::Incidence, ValveReference::Phi].into_iter().enumerate() {
                let (x, _, rr) = matched_lagged(m, &fl, &ramp, Some(&st), BMAX, Some(tau), rf, m_lim, 1e-12).expect("bracketed");
                let v = BleedLimiter::with_tau(x, BMAX, Some(tau));
                let mm = sibling(m, Some(&st), Some(if k == 0 { v.on_incidence() } else { v }));
                Census65::reset();
                let t = march(&mm, &fl, &ramp, None).0;
                let fc = first_closing_read();
                let lr = lag_read(&mm, &fl, &t, Census65::take().clamp_hits);
                let (ri, fa) = segments(&t);
                nruns += 1;
                if (ri, fa) != (1, 1) { not_single += 1; println!("  SEGMENTS {shape} {r} {sm} tau={tau} k={k}: rises={ri} falls={fa} b0={:.3e}", lr.b0); }
                d[k] = lr.fall; first[k] = rr.s_at_m_i < fc; sms[k] = x;
                trajs.push(t);
            }
            let n = trajs[0].len().min(trajs[1].len());
            for k in 0..2 { a[k] = trapz(&trajs[k], n, |p| valve_at(p).0); }
            // the phi valve's floor (only meaningful where its minimum is after its first close)
            let floor = if !first[1] {
                let shut = sibling(m, Some(&st), None);
                let lo = read(&shut, &fl, &march(&shut, &fl, &ramp, None).0).min_phi;
                let f = |x: f64| pre_close_min(m, &ramp, x, tt2, t_c).0 - m_lim;
                let (flo, fhi) = (f(lo), f(1.5));
                if flo < 0.0 && fhi > 0.0 {
                    let x = try_illinois(|x| Ok(f(x)), lo, 1.5, flo, fhi, 1e-12, ILLINOIS_MAXIT).unwrap();
                    let (_, fcx) = pre_close_min(m, &ramp, x, tt2, t_c);
                    format!("floor sm={:.4} (sym {:.4}, fc@{fcx:.4})", x / 0.55 - 1.0, sms[1] / 0.55 - 1.0)
                } else { format!("floor UNBRACKETED ({flo:.2e},{fhi:.2e})") }
            } else { String::from("min-first") };
            println!("{shape} {r} {sm:.2} tau={tau}: R={:.4} inc-first={} phi-first={} | phi {floor}", a[1] / a[0], first[0], first[1]);
            rows.push((tau, a, d, first[0] && first[1]));
        }
        for &(to, a, d, both) in &rows {
            if !both { continue; }
            let mut line = format!("  R_f {shape} {r} {sm:.2} tau_o={to}: R={:.4}", a[1] / a[0]);
            for tc in TAUS { if tc == to { continue; }
                let rf = (a[1] + (tc - to) * d[1]) / (a[0] + (tc - to) * d[0]);
                rf_n += 1; if (rf - 1.0) * (a[1] / a[0] - 1.0) <= 0.0 { rf_cross += 1; line += " CROSS"; }
                line += &format!(" {tc}:{rf:.4}"); }
            println!("{line}");
        }
    }
    println!("SEGMENTS: {not_single} of {nruns} runs are not exactly one rise and one fall");
    println!("R_f: {rf_cross} of {rf_n} (both-min-first cell, tau_o, tau_c) pairs put R_f across 1 from R(tau_o)");
}
