// G1 (the decisive given): on the SYMMETRIC slow valves re-tuned to rung 87's m_lim (rung 88 sec. 4),
// where is each valve's M_i minimum against the first stage that reads the closing constant? Plus the
// edge travel U/D and the bill identity, the after-close margin slack, and onset. Arg 1: ds.
use r89probe::*;
use turbojet::lagged_bleed::{first_closing_read, Census65};
use turbojet::lagged_incidence::{lag_read, matched_lagged, valve_at};
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
    let (tt2, _, _) = ms.fuel.inner.inlet(&fl);
    let mut tally = std::collections::BTreeMap::<(String, u32), (u32, u32)>::new();
    for (shape, r, sm) in CELLS {
        let m = if shape == "shaped" { &ms } else { &mt };
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
        let t_c = m.design_map(Spool::Lp).tan_beta1_crit();
        let m_lim = t_c - 1.0 / phi_of(sm);
        println!("CELL {shape} r={r} sm={sm:.2} m_lim={m_lim:.6}");
        for tau in TAUS {
            for (k, rf) in [ValveReference::Incidence, ValveReference::Phi].into_iter().enumerate() {
                let name = if k == 0 { "INC" } else { "PHI" };
                let Some((x, _, rr)) = matched_lagged(m, &fl, &ramp, Some(&st), BMAX, Some(tau), rf, m_lim, 1e-12)
                else { println!("  tau={tau} {name}: UNBRACKETED"); continue; };
                let v = BleedLimiter::with_tau(x, BMAX, Some(tau));
                let mm = sibling(m, Some(&st), Some(if k == 0 { v.on_incidence() } else { v }));
                Census65::reset();
                let t = march(&mm, &fl, &ramp, None).0;
                let fc = first_closing_read();
                let clamps = Census65::take().clamp_hits;
                let lr = lag_read(&mm, &fl, &t, clamps);
                // per-point margin; min before / after the first closing read
                let mi: Vec<f64> = t.iter().map(|p| t_c - 1.0 / p.phi_lp + mm.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2))).collect();
                let (mut pre, mut post) = (f64::INFINITY, f64::INFINITY);
                for (i, p) in t.iter().enumerate() { if p.s < fc { pre = pre.min(mi[i]) } else { post = post.min(mi[i]) } }
                let before = rr.s_at_m_i < fc;
                let lag_int = lr.int_b - lr.int_b_cmd;
                let bill = tau * (lr.fall - lr.rise);
                // onset: first point with b > 1e-9 ; peak and where
                let onset = t.iter().find(|p| valve_at(p).0 > 1e-9).map(|p| p.s).unwrap_or(f64::NAN);
                let (mut bpk, mut spk) = (0.0f64, 0.0);
                for p in &t { let b = valve_at(p).0; if b > bpk { bpk = b; spk = p.s; } }
                println!("  tau={tau} {name}: sm={:.4} min@{:.3} firstclose@{} {} pre={:.6} post-slack={:+.2e} b0={:.3e} bpk={:.3e}@{:.3} bend={:.3e} U={:.3e} D={:.3e} lagint={:+.3e} tau(D-U)={:+.3e} resid={:.1e} onset={:.3} clamps={clamps} n={}",
                         x / 0.55 - 1.0, rr.s_at_m_i, if fc.is_finite() { format!("{fc:.4}") } else { "never".into() },
                         if before { "MIN-FIRST" } else { "CLOSE-FIRST" }, pre, post - pre, lr.b0, bpk, spk, lr.b_end, lr.rise, lr.fall,
                         lag_int, bill, lr.identity_residual, onset, t.len());
                let e = tally.entry((name.to_string(), (tau * 1000.0) as u32)).or_insert((0, 0));
                e.1 += 1; if before { e.0 += 1; }
            }
        }
    }
    println!("TALLY min-before-first-close (count/of) per valve, tau:");
    for ((n, t), (a, b)) in tally { println!("  {n} tau={:.3}: {a}/{b}", t as f64 / 1000.0); }
}
