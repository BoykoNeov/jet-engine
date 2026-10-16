//! Rung 93 — scoring every pre-registered quantity (anchor § 3) with the shipped device.
use jet_rung93_probe::*;
use turbojet::metal_temperature::{self as mt, FixedEngine, MetalLimit, Row};
use turbojet::turbine_cooling::{mixed_tt, TurbineCooling, TurbineCoolingMatcher};

const S: f64 = 150.0 * 5.0 / 9.0;
fn need(factor: f64, tg: f64, tc: f64, p: f64, lim: f64) -> f64 {
    let y = (1.0 + p) * (tg + S - tc) / (lim - tc) - 1.0;
    if y <= 0.0 { 0.0 } else { 4.0 / 3.0 * factor * 0.022 * y.powf(1.25) }
}
/// The anchor's model (copied from model.rs, unchanged): (e1, e2).
fn model(factor: f64, tt4: f64, tc0: f64, f: f64, a1: f64, a2: f64) -> (f64, f64) {
    let g = cpg();
    let lv = mt::allowable(Row::Vane, 1985.0, 1e4);
    let lb = mt::allowable(Row::Blade, 1985.0, 1e4);
    let (mut e1, mut e2) = (0.0_f64, 0.0_f64);
    for _ in 0..200 {
        let tc = tc0 * (1.0 + a1 * e1 + a2 * e2);
        let gg = (1.0 - e1 - e2) * (1.0 + f);
        let n1 = need(factor, tt4, tc, 0.30, lv) * gg;
        let f41 = f * (1.0 - e1 - e2) / (1.0 - e2);
        let t41 = mixed_tt(&g, gg, tt4, f, n1, tc, f41);
        let n2 = need(factor, 0.92 * t41, tc, 0.13, lb) * (gg + n1);
        if (n1 - e1).abs() < 1e-13 && (n2 - e2).abs() < 1e-13 { e1 = n1; e2 = n2; break; }
        e1 = n1; e2 = n2;
    }
    (e1, e2)
}

fn main() {
    let only = std::env::args().nth(1).unwrap_or_default();
    let tt4s: Vec<f64> = (0..10).map(|k| 1300.0 + 100.0 * k as f64).collect();
    let factors = [2.0, 1.5, 1.0, 0.8];
    if only.is_empty() || only == "b" {
        // ---- M1/M2/M4 (B) per map
        for (name, ml, mh) in maps() {
            let fe = FixedEngine::new(design(), flight(), 1.0, ml, mh);
            let mk = |e1: f64, e2: f64| {
                let eng = if e1 + e2 > 0.0 { design().with_turbine_cooling(TurbineCooling::new(e1, e2)) } else { design() };
                TurbineCoolingMatcher::new(eng, flight(), 1.0, ml, mh, 0.0)
            };
            let (m0, mv, mb) = (mk(0.0, 0.0), mk(0.05, 0.0), mk(0.0, 0.02));
            for &factor in &factors {
                let lim = MetalLimit::gauntner_1985(factor);
                println!("\n== B {name} FACTOR {factor}: Tt4 | device e1 e2 tot (passes) | M1 de1 de2 | Model0 tot (M2) | F/m N_L N_H phi_L phi_H | metal-lim v/b");
                let mut prev_st = f64::NEG_INFINITY;
                let (mut m1_worst, mut m2_ok, mut m4_ok) = (0.0_f64, true, true);
                for &t in &tt4s {
                    let r0 = m0.match_point(&flight(), t);
                    let (rv, rb) = (mv.match_point(&flight(), t), mb.match_point(&flight(), t));
                    let tc0 = r0.base.base.station("3").tt;
                    let f0 = r0.base.base.station("4").far;
                    let a1 = (rv.base.base.station("3").tt / tc0 - 1.0) / 0.05;
                    let a2 = (rb.base.base.station("3").tt / tc0 - 1.0) / 0.02;
                    let m1 = model(factor, t, tc0, f0, a1, a2);
                    let m0t = { let x = model(factor, t, tc0, f0, 0.0, 0.0); x.0 + x.1 };
                    match fe.try_limited(&lim, &flight(), t) {
                        Ok(l) => {
                            let (d1, d2) = (l.vane - m1.0, l.blade - m1.1);
                            m1_worst = m1_worst.max(d1.abs()).max(d2.abs());
                            let tot = l.vane + l.blade;
                            if !(tot < m0t) { m2_ok = false; }
                            let k = l.point.booking.expect("cooled");
                            if !(k.st_inlet > prev_st) { m4_ok = false; }
                            prev_st = k.st_inlet;
                            let b = &l.point.base;
                            println!("{t:.0} | {:.5} {:.5} {tot:.5} ({}) | {d1:+.2e} {d2:+.2e} | {m0t:.5} | {:.2} {:.4} {:.4} {:.4} {:.4} | {:+.1e} {:+.1e}",
                                     l.vane, l.blade, l.passes, k.st_inlet, b.n_lp_ratio, b.n_hp_ratio, b.phi_lp, b.phi_hp,
                                     l.reading.vane - lim.vane, l.reading.blade - lim.blade);
                        }
                        Err(e) => { println!("{t:.0} | REFUSED {}", e.0); m4_ok = false; }
                    }
                }
                println!("SUMMARY B {name} F {factor}: M1 worst |de| {m1_worst:.2e} (bar 1.5e-3) | M2 all below Model0: {m2_ok} | M4 F/m monotone: {m4_ok}");
            }
        }
    }
    let (ml, mh) = (maps()[0].1, maps()[0].2);
    let fe = FixedEngine::new(design(), flight(), 1.0, ml, mh);
    if only.is_empty() || only == "a" {
        // ---- A per FACTOR (+ M3 crossing vs B flat on a 10 K grid, M5)
        for &factor in &factors {
            let lim = MetalLimit::gauntner_1985(factor);
            println!("\n== A FACTOR {factor}: Tt4 | e1 e2 tot (passes) | F/m TSFC_lim TSFC_unc gap | metal-lim v/b");
            let (mut prev_st, mut prev_gap) = (f64::NEG_INFINITY, f64::INFINITY);
            let (mut m4_ok, mut m5_ok) = (true, true);
            for &t in &tt4s {
                let a = mt::try_limited_design(&cpg(), 3.0, 6.0, t, 50_000.0, real(), &lim, &flight(), 1.0).expect("A");
                let u = design_at(t).run(&flight(), 1.0);
                let gap = a.run.tsfc_inlet * 1e6 - u.performance.tsfc * 1e6;
                if !(a.run.st_inlet > prev_st) { m4_ok = false; }
                if !(gap < 0.0 && gap < prev_gap) { m5_ok = false; }
                prev_st = a.run.st_inlet; prev_gap = gap;
                println!("{t:.0} | {:.5} {:.5} {:.5} ({}) | {:.2} {:.3} {:.3} {gap:+.3} | {:+.1e} {:+.1e}", a.vane, a.blade, a.vane + a.blade,
                         a.passes, a.run.st_inlet, a.run.tsfc_inlet * 1e6, u.performance.tsfc * 1e6,
                         a.reading.vane - lim.vane, a.reading.blade - lim.blade);
            }
            println!("SUMMARY A F {factor}: M4 F/m monotone: {m4_ok} | M5 TSFC below uncooled and gap widening: {m5_ok}");
            let mut prev: Option<(f64, f64)> = None;
            let mut crossings = Vec::new();
            let mut t = 1300.0;
            while t <= 2200.0 + 1e-9 {
                let b = fe.try_limited(&lim, &flight(), t).expect("B");
                let a = mt::try_limited_design(&cpg(), 3.0, 6.0, t, 50_000.0, real(), &lim, &flight(), 1.0).expect("A");
                let d = (b.vane + b.blade) - (a.vane + a.blade);
                if let Some((tp, dp)) = prev { if dp.signum() != d.signum() { crossings.push((tp + (t - tp) * dp / (dp - d), dp < d)); } }
                prev = Some((t, d));
                t += 10.0;
            }
            println!("M3 F {factor}: crossings (T, rising) {crossings:?}; B-A at 2200 {:+.5}", prev.unwrap().1);
        }
    }
    if only.is_empty() || only == "k" {
        // ---- M6 knob form: FACTOR 1.0, blade 0.02, flat
        let lim = MetalLimit::gauntner_1985(1.0);
        println!("\n== M6 knob: vane | Tt4_max binding | each row's own limit");
        let row_lim = |e1: f64, row: Row| -> f64 {
            let m = fe.matcher(TurbineCooling::new(e1, 0.02));
            let g = |t: f64| { let p = m.match_point(&flight(), t); let r = mt::read(&mt::MetalInputs::of_cooled(&p), &lim);
                               match row { Row::Vane => r.vane - lim.vane, Row::Blade => r.blade - lim.blade } };
            let (mut a, mut b) = (800.0, 2400.0);
            for _ in 0..60 { let mid = 0.5 * (a + b); if g(mid) < 0.0 { a = mid } else { b = mid } }
            a
        };
        let mut prev_b = f64::NEG_INFINITY; let mut blade_mono = true; let mut switches = Vec::new(); let mut prev_bind = None;
        for k in 0..=8 {
            let e1 = 0.02 * k as f64;
            let mx = fe.try_max_tt4(TurbineCooling::new(e1, 0.02), &lim, &flight(), 800.0, 2400.0);
            let (tv, tb) = (row_lim(e1, Row::Vane), row_lim(e1, Row::Blade));
            if !(tb > prev_b) { blade_mono = false; } prev_b = tb;
            match mx {
                Ok(m) => { println!("{e1:.2} | {:.3} {:?} | vane {tv:.2} blade {tb:.2}", m.tt4, m.binding);
                           if let Some(pb) = prev_bind { if pb != m.binding { switches.push(e1); } } prev_bind = Some(m.binding); }
                Err(e) => println!("{e1:.2} | REFUSED {} | vane {tv:.2} blade {tb:.2}", e.0),
            }
        }
        let (mut a, mut b) = (0.02, 0.04);
        for _ in 0..40 { let mid = 0.5 * (a + b); if row_lim(mid, Row::Vane) - row_lim(mid, Row::Blade) < 0.0 { a = mid } else { b = mid } }
        println!("M6: switches after {switches:?}; crossover share {:.5} (bar 0.0338 +- 0.0010); blade limit monotone in vane air: {blade_mono}", 0.5 * (a + b));
    }
}
