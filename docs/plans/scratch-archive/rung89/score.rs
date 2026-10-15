// Rung 89 scoring of anchor section 3 (scratch). Arg 1: ds. Reads only; the bars were frozen in the anchor commit.
// Grid: 19 cells x (tau_o, tau_c) in TAUS^2; both valves re-tuned to m_lim. Writes grid_<ds>.tsv and prints H1-H5.
use r89probe::*;
use std::io::Write;
use turbojet::asymmetric_valve::{asym_valve, march_marked, margins, matched_asym, min_before, phi_floor};
use turbojet::lagged_incidence::{lag_read, trapz, valve_at};
use turbojet::limited_bleed::ValveReference;
use turbojet::stator_transient::Ramp;
use turbojet::two_spool::Spool;
use turbojet::valve_reference::sibling;

#[derive(Clone, Debug)]
struct Run { x: f64, s_min: f64, fc: f64, first: bool, n: usize, rise: f64, fall: f64, b_peak: f64, clamps: u64, resid: f64,
             post_slack: f64, traj: Vec<turbojet::fuel_transient::FuelPoint> }

fn main() {
    let ds: f64 = std::env::args().nth(1).map(|x| x.parse().unwrap()).unwrap_or(0.005);
    // arg 2 (optional): a comma list of cell indices into CELLS, so the grid can run as parallel pieces
    let pick: Option<Vec<usize>> = std::env::args().nth(2).map(|a| a.split(',').map(|x| x.parse().unwrap()).collect());
    let tag = std::env::args().nth(2).map(|a| format!("_{}", a.replace(',', "-"))).unwrap_or_default();
    let fl = flight();
    let ms = bare(lp_map(), hp_map());
    let mt = bare(tilt_map(), tilt_map());
    let st = stat();
    let mut tsv = std::fs::File::create(format!("W:/temp/claude/jet-rung89-probe/grid_{ds}{tag}.tsv")).unwrap();
    writeln!(tsv, "shape\tr\tsm\ttau_o\ttau_c\tvalve\tsm_set\ts_min\tfirst_close\tmin_first\tA\tthrust\tU\tD\tb_peak\tclamps\tresid\tpost_slack\tR").unwrap();
    // tallies (hit, scored)
    let mut h = std::collections::BTreeMap::<&str, (u32, u32)>::new();
    let mut tally = |k: &'static str, ok: bool| { let e = h.entry(k).or_insert((0, 0)); e.1 += 1; if ok { e.0 += 1; } };
    let mut notes: Vec<String> = Vec::new();
    for (ci, (shape, r, sm)) in CELLS.into_iter().enumerate() {
        if pick.as_ref().is_some_and(|p| !p.contains(&ci)) { continue; }
        let m = if shape == "shaped" { &ms } else { &mt };
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds };
        let m_lim = m.design_map(Spool::Lp).tan_beta1_crit() - 1.0 / phi_of(sm);
        // grid[i_o][i_c][valve]
        let mut grid: Vec<Vec<[Option<Run>; 2]>> = vec![vec![[None, None]; 5]; 5];
        for (io, &to) in TAUS.iter().enumerate() { for (ic, &tc) in TAUS.iter().enumerate() {
            for (k, rf) in [ValveReference::Incidence, ValveReference::Phi].into_iter().enumerate() {
                let Some((x, _, rr)) = matched_asym(m, &fl, &ramp, Some(&st), BMAX, to, tc, rf, m_lim, 1e-12) else {
                    notes.push(format!("UNBRACKETED {shape} {r} {sm} {to} {tc} k={k}")); continue; };
                let mm = sibling(m, Some(&st), Some(asym_valve(x, BMAX, to, tc, rf)));
                let (t, fc, clamps) = march_marked(&mm, &fl, &ramp);
                let lr = lag_read(&mm, &fl, &t, clamps);
                let mi = margins(&mm, &fl, &t);
                let pre = min_before(&mi, &t, fc);
                let post = t.iter().zip(&mi).filter(|(p, _)| p.s >= fc).map(|(_, &v)| v).fold(f64::INFINITY, f64::min);
                let b_peak = t.iter().map(|p| valve_at(p).0).fold(0.0, f64::max);
                grid[io][ic][k] = Some(Run { x, s_min: rr.s_at_m_i, fc, first: rr.s_at_m_i < fc, n: t.len(), rise: lr.rise, fall: lr.fall,
                    b_peak, clamps, resid: lr.identity_residual, post_slack: post - pre, traj: t });
            }
        }}
        // the phi floor per tau_o (close-first symmetric rows)
        let floors: Vec<Option<(f64, f64)>> = TAUS.iter().enumerate().map(|(io, &to)| {
            let d = grid[io][io][1].as_ref()?; if d.first { None } else { phi_floor(m, &fl, &ramp, Some(&st), BMAX, to, m_lim, 1e-12) } }).collect();
        for io in 0..5 { for ic in 0..5 {
            let (to, tc) = (TAUS[io], TAUS[ic]);
            let (Some(ri), Some(rp)) = (grid[io][ic][0].as_ref(), grid[io][ic][1].as_ref()) else { continue };
            let n = ri.n.min(rp.n);
            let a_i = trapz(&ri.traj, n, |p| valve_at(p).0); let a_p = trapz(&rp.traj, n, |p| valve_at(p).0);
            let f_i = thrust(&ri.traj, n); let f_p = thrust(&rp.traj, n);
            let rr = a_p / a_i;
            for (k, ru, a, f) in [(0, ri, a_i, f_i), (1, rp, a_p, f_p)] {
                writeln!(tsv, "{shape}\t{r}\t{sm}\t{to}\t{tc}\t{}\t{:.10}\t{:.4}\t{:.4}\t{}\t{:.6e}\t{:.6e}\t{:.6e}\t{:.6e}\t{:.6e}\t{}\t{:.2e}\t{:.3e}\t{:.6}",
                         if k == 0 { "inc" } else { "phi" }, ru.x / 0.55 - 1.0, ru.s_min, ru.fc, ru.first, a, f, ru.rise, ru.fall, ru.b_peak,
                         ru.clamps, ru.resid, ru.post_slack, rr).unwrap();
                if ru.clamps != 0 { notes.push(format!("CLAMP {shape} {r} {sm} {to} {tc} k={k}: {}", ru.clamps)); }
            }
            if io == ic { continue; }
            let di = grid[io][io][0].as_ref().unwrap(); let dp = grid[io][io][1].as_ref().unwrap();
            let nd = di.n.min(dp.n);
            let ad_i = trapz(&di.traj, nd, |p| valve_at(p).0); let ad_p = trapz(&dp.traj, nd, |p| valve_at(p).0);
            let r_d = ad_p / ad_i;
            // H1
            let h1 = (ri.x - di.x).abs() <= 1e-9 && ri.first;
            tally("H1", h1); if !h1 { notes.push(format!("H1 miss {shape} {r} {sm} ({to},{tc}): dphi={:.2e} first={} s_min={} fc={}", ri.x - di.x, ri.first, ri.s_min, ri.fc)); }
            // H2
            let pred = a_i - ad_i - (tc - to) * di.fall;
            let bar = (0.05 * (tc - to).abs() * di.b_peak).max(1e-5);
            tally("H2", pred.abs() <= bar); if pred.abs() > bar { notes.push(format!("H2 miss {shape} {r} {sm} ({to},{tc}): drift {pred:.2e} bar {bar:.2e}")); }
            let rf = (ad_p + (tc - to) * dp.fall) / (ad_i + (tc - to) * di.fall);
            if dp.first {
                // H3b (tc > to), H4a (tc > to)
                if tc > to {
                    let ok = (rp.x - dp.x).abs() <= 1e-9; tally("H3b", ok);
                    if !ok { notes.push(format!("H3b miss {shape} {r} {sm} ({to},{tc}): dphi={:.2e}", rp.x - dp.x)); }
                    let ok = (rr / rf - 1.0).abs() <= 0.02; tally("H4a", ok);
                    if !ok { notes.push(format!("H4a miss {shape} {r} {sm} ({to},{tc}): R={rr:.4} Rf={rf:.4}")); }
                    let ok = (rr - 1.0) * (r_d - 1.0) > 0.0; tally("H5b", ok);
                    if !ok { notes.push(format!("H5b miss {shape} {r} {sm} ({to},{tc}): R={rr:.4} Rdiag={r_d:.4}")); }
                }
            } else {
                // H4b
                if tc > to {
                    let (fx, _) = floors[io].expect("floor");
                    let mf = sibling(m, Some(&st), Some(asym_valve(fx, BMAX, to, to, ValveReference::Phi)));
                    let (tf, _, cf) = march_marked(&mf, &fl, &ramp);
                    let lf = lag_read(&mf, &fl, &tf, cf);
                    let nf = tf.len().min(di.n);
                    let rfl = (trapz(&tf, nf, |p| valve_at(p).0) + (tc - to) * lf.fall) / (trapz(&di.traj, nf, |p| valve_at(p).0) + (tc - to) * di.fall);
                    let ok = rfl * 0.98 <= rr && rr < rf; tally("H4b", ok);
                    if !ok { notes.push(format!("H4b miss {shape} {r} {sm} ({to},{tc}): Rfloor={rfl:.4} R={rr:.4} Rf={rf:.4}")); }
                    if r_d < 1.0 {
                        let ok = rr < 1.0; tally("H5b", ok);
                        if !ok { notes.push(format!("H5b miss {shape} {r} {sm} ({to},{tc}): R={rr:.4}")); }
                    } else if tc == 0.2 {
                        let ok = rr < 1.0; tally("H5a", ok);
                        notes.push(format!("H5a {} {shape} {r} {sm} ({to},{tc}): R={rr:.4} Rdiag={r_d:.4} phi sm {:.4} (sym {:.4}, floor {:.4})",
                                           if ok { "HIT" } else { "MISS" }, rp.x / 0.55 - 1.0, dp.x / 0.55 - 1.0, fx / 0.55 - 1.0));
                    }
                } else {
                    let ok = rr > rf; tally("H4b", ok);
                    if !ok { notes.push(format!("H4b(fast) miss {shape} {r} {sm} ({to},{tc}): R={rr:.4} Rf={rf:.4}")); }
                }
            }
            if tc < to {
                let ok = (rr - 1.0) * (r_d - 1.0) > 0.0; tally("H5c", ok);
                if !ok { notes.push(format!("H5c miss {shape} {r} {sm} ({to},{tc}): R={rr:.4} Rdiag={r_d:.4}")); }
            }
        }}
        // H3a per close-first tau_o: over tau_c ascending, phi set point non-increasing, >= floor, strictly decreasing above floor, == floor once min-first
        for io in 0..5 {
            let dp = grid[io][io][1].as_ref().unwrap(); if dp.first { continue; }
            let (fx, fres) = floors[io].unwrap();
            let xs: Vec<(f64, bool)> = (0..5).map(|ic| { let g = grid[io][ic][1].as_ref().unwrap(); (g.x, g.first) }).collect();
            let mut ok = true; let mut bind: Option<f64> = None;
            for ic in 0..5 {
                let (x, first) = xs[ic];
                if x < fx - 1e-9 { ok = false; }
                if first { if bind.is_none() { bind = Some(TAUS[ic]); } if (x - fx).abs() > 1e-9 { ok = false; } }
                else if bind.is_some() { ok = false; }
                if ic > 0 { let xp = xs[ic - 1].0; if xp > fx + 1e-9 { if !(x < xp) { ok = false; } } else if x > xp + 1e-9 { ok = false; } }
            }
            tally("H3a", ok);
            notes.push(format!("H3a {} {shape} {r} {sm} tau_o={}: floor {:.4} (resid {fres:.1e}) binds at tau_c={:?}; sm {:?}",
                               if ok { "HIT" } else { "MISS" }, TAUS[io], fx / 0.55 - 1.0, bind, xs.iter().map(|(x, f)| format!("{:.4}{}", x / 0.55 - 1.0, if *f { "*" } else { "" })).collect::<Vec<_>>()));
        }
        eprintln!("done {shape} {r} {sm}");
    }
    for n in &notes { println!("{n}"); }
    println!("TALLY (hit/scored):");
    for (k, (a, b)) in h { println!("  {k}: {a}/{b}"); }
}
