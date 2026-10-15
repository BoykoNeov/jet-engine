// G2: on the SYMMETRIC slow valves re-tuned to m_lim (rung 88 sec. 4) — each valve's air A (over the common
// length), its opening and closing travel U/D, and rung 88's ratio R = A_phi/A_inc next to D_phi/D_inc and
// U_phi/U_inc: the arithmetic the two arms' bills move along. Symmetric only. Arg 1: ds.
use r89probe::*;
use turbojet::lagged_bleed::{first_closing_read, Census65};
use turbojet::lagged_incidence::{lag_read, matched_lagged, trapz, valve_at};
use turbojet::limited_bleed::{BleedLimiter, ValveReference};
use turbojet::stator_transient::Ramp;
use turbojet::three_levers::march;
use turbojet::two_spool::Spool;
use turbojet::valve_reference::sibling;

fn main() {
    let ds: f64 = std::env::args().nth(1).map(|x| x.parse().unwrap()).unwrap_or(0.005);
    let fl = flight();
    let ms = bare(lp_map(), hp_map());
    let mt = bare(tilt_map(), tilt_map());
    let st = stat();
    println!("cell tau | A_inc A_phi R | U_inc D_inc U_phi D_phi | D_phi/D_inc U_phi/U_inc | close-first(inc,phi) | sm_inc sm_phi");
    for (shape, r, sm) in CELLS {
        let m = if shape == "shaped" { &ms } else { &mt };
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
        let m_lim = m.design_map(Spool::Lp).tan_beta1_crit() - 1.0 / phi_of(sm);
        for tau in TAUS {
            let mut out = Vec::new();
            for (k, rf) in [ValveReference::Incidence, ValveReference::Phi].into_iter().enumerate() {
                let (x, _, rr) = matched_lagged(m, &fl, &ramp, Some(&st), BMAX, Some(tau), rf, m_lim, 1e-12).expect("bracketed");
                let v = BleedLimiter::with_tau(x, BMAX, Some(tau));
                let mm = sibling(m, Some(&st), Some(if k == 0 { v.on_incidence() } else { v }));
                Census65::reset();
                let t = march(&mm, &fl, &ramp, None).0;
                let fc = first_closing_read();
                let lr = lag_read(&mm, &fl, &t, Census65::take().clamp_hits);
                out.push((x / 0.55 - 1.0, t, lr, rr.s_at_m_i >= fc));
            }
            let n = out[0].1.len().min(out[1].1.len());
            let a: Vec<f64> = out.iter().map(|o| trapz(&o.1, n, |p| valve_at(p).0)).collect();
            println!("{shape} {r} {sm:.2} {tau} | {:.4e} {:.4e} {:.3} | {:.3e} {:.3e} {:.3e} {:.3e} | {:.3} {:.3} | {} {} | {:.4} {:.4}",
                     a[0], a[1], a[1] / a[0], out[0].2.rise, out[0].2.fall, out[1].2.rise, out[1].2.fall,
                     out[1].2.fall / out[0].2.fall, out[1].2.rise / out[0].2.rise, out[0].3, out[1].3, out[0].0, out[1].0);
        }
    }
}
