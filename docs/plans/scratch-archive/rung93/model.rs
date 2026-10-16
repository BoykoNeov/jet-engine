//! Rung 93 anchor § 1 — THE MODEL (closed form + § 0's measured slopes), on EXISTING code only.
//! No metal device exists; this is the prediction instrument the device will be scored against.
use jet_rung93_probe::*;
use turbojet::turbine_cooling::{mixed_tt, TurbineCooling, TurbineCoolingMatcher};

const S: f64 = 150.0 * 5.0 / 9.0;        // Gauntner's safety margin, K
const PV: f64 = 0.30;                     // pattern factor, first stator
const PB: f64 = 0.13;                     // pattern factor, other rows
const KREL: f64 = 0.92;                   // rotor relative total temperature
const LV: f64 = 2210.0 * 5.0 / 9.0;       // 1985 vane limit, K
const LB: f64 = 2110.0 * 5.0 / 9.0;       // 1985 blade limit, K

/// Coolant share of the row's INLET GAS the stream must carry (endwall 4/3 included) so that a row
/// seeing gas `tg` (before safety and hot spot) with coolant `tc` sits at `lim`. 0 if uncooled is fine.
fn need(factor: f64, tg: f64, tc: f64, p: f64, lim: f64) -> f64 {
    let y = (1.0 + p) * (tg + S - tc) / (lim - tc) - 1.0;
    if y <= 0.0 { 0.0 } else { 4.0 / 3.0 * factor * 0.022 * y.powf(1.25) }
}

#[derive(Clone, Copy)]
struct Line { tc: f64, f: f64, st: f64 }

fn line(m: &TurbineCoolingMatcher, tt4: f64) -> Line {
    let r = m.try_match_point(&flight(), tt4).unwrap();
    let s = &r.base.base;
    let st = r.booking.map(|k| k.st_inlet).unwrap_or(s.performance.specific_thrust);
    Line { tc: s.station("3").tt, f: s.station("4").far, st }
}

/// Solve both rows at their limits: Tc(e1,e2) = tc0*(1 + a1 e1 + a2 e2) (a = 0: frozen, Model 0).
fn solve(factor: f64, tt4: f64, tc0: f64, f: f64, a1: f64, a2: f64) -> (f64, f64, f64, f64) {
    let g = cpg();
    let (mut e1, mut e2) = (0.0_f64, 0.0_f64);
    let mut tc = tc0;
    let mut t41 = tt4;
    for _ in 0..200 {
        tc = tc0 * (1.0 + a1 * e1 + a2 * e2);
        let gg = (1.0 - e1 - e2) * (1.0 + f);
        let n1 = need(factor, tt4, tc, PV, LV) * gg;
        let f41 = f * (1.0 - e1 - e2) / (1.0 - e2);
        t41 = mixed_tt(&g, gg, tt4, f, n1, tc, f41);
        let n2 = need(factor, KREL * t41, tc, PB, LB) * (gg + n1);
        if (n1 - e1).abs() < 1e-13 && (n2 - e2).abs() < 1e-13 { e1 = n1; e2 = n2; break; }
        e1 = n1; e2 = n2;
    }
    (e1, e2, tc, t41)
}

fn main() {
    if std::env::args().any(|a| a == "part2") { part2(); return; }
    if std::env::args().any(|a| a == "part3") { part3(); return; }
    let tt4s = [1300.0, 1400.0, 1500.0, 1600.0, 1700.0, 1800.0, 1900.0, 2000.0, 2100.0, 2200.0];
    for (name, ml, mh) in maps() {
        let mk = |e1: f64, e2: f64| {
            let eng = if e1 + e2 > 0.0 { design().with_turbine_cooling(TurbineCooling::new(e1, e2)) } else { design() };
            TurbineCoolingMatcher::new(eng, flight(), 1.0, ml.clone(), mh.clone(), 0.0)
        };
        let (m0, mv, mb, mvb, mvb2) = (mk(0.0, 0.0), mk(0.05, 0.0), mk(0.0, 0.02), mk(0.05, 0.02), mk(0.10, 0.05));
        println!("\n== {name}: slopes and the Tc linearisation's residual");
        println!("Tt4 | a1 a2 (rel Tc per share) | d1 d2 (rel F/m per share) | Tc(5,2) meas/model | Tc(10,5) meas/model");
        let mut slopes = Vec::new();
        for &t in &tt4s {
            let (l0, lv, lb) = (line(&m0, t), line(&mv, t), line(&mb, t));
            let a1 = (lv.tc / l0.tc - 1.0) / 0.05;
            let a2 = (lb.tc / l0.tc - 1.0) / 0.02;
            let d1 = (lv.st / l0.st - 1.0) / 0.05;
            let d2 = (lb.st / l0.st - 1.0) / 0.02;
            let (c1, c2) = (line(&mvb, t), line(&mvb2, t));
            let p1 = l0.tc * (1.0 + a1 * 0.05 + a2 * 0.02);
            let p2 = l0.tc * (1.0 + a1 * 0.10 + a2 * 0.05);
            println!("{t:.0} | {a1:.4} {a2:.4} | {d1:.4} {d2:.4} | {:.2}/{p1:.2} ({:+.2}) | {:.2}/{p2:.2} ({:+.2})",
                     c1.tc, c1.tc - p1, c2.tc, c2.tc - p2);
            slopes.push((t, l0, a1, a2, d1, d2));
        }
        for factor in [2.0, 1.5, 1.0, 0.8] {
            println!("\n-- {name}, FACTOR {factor}: the cooling that puts BOTH metals at their limits");
            println!("Tt4 | B Model0 e1 e2 tot | B Model1 e1 e2 tot (Tc, Tt41) | F/m_B model | A e1 e2 tot (Tc, Tt41) | F/m_A unc, model | TSFC_A unc");
            for (t, l0, a1, a2, d1, d2) in &slopes {
                let b0 = solve(factor, *t, l0.tc, l0.f, 0.0, 0.0);
                let b1 = solve(factor, *t, l0.tc, l0.f, *a1, *a2);
                let ra = design_at(*t).run(&flight(), 1.0);
                let st = |l: &str| ra.stations.iter().find(|(x, _)| *x == l).unwrap().1;
                let a = solve(factor, *t, st("3").tt, st("4").far, 0.0, 0.0);
                let fb = l0.st * (1.0 + d1 * b1.0 + d2 * b1.1);
                let fa_unc = ra.performance.specific_thrust;
                let fa = fa_unc * (1.0 + d1 * a.0 + d2 * a.1);
                println!("{t:.0} | {:.4} {:.4} {:.4} | {:.4} {:.4} {:.4} ({:.1}, {:.1}) | {fb:.1} | {:.4} {:.4} {:.4} ({:.1}, {:.1}) | {fa_unc:.1} {fa:.1} | {:.2}",
                         b0.0, b0.1, b0.0 + b0.1, b1.0, b1.1, b1.0 + b1.1, b1.2, b1.3,
                         a.0, a.1, a.0 + a.1, a.2, a.3, ra.performance.tsfc * 1e6);
            }
        }
    }
}

/// Part 2 — the A/B crossing on a fine grid, and the knob form (shares in, Tt4_max out), Model 1.
pub fn part2() {
    let (ml, mh) = (maps()[0].1.clone(), maps()[0].2.clone());
    let mk = |e1: f64, e2: f64| {
        let eng = if e1 + e2 > 0.0 { design().with_turbine_cooling(TurbineCooling::new(e1, e2)) } else { design() };
        TurbineCoolingMatcher::new(eng, flight(), 1.0, ml.clone(), mh.clone(), 0.0)
    };
    let (m0, mv, mb) = (mk(0.0, 0.0), mk(0.05, 0.0), mk(0.0, 0.02));
    let slopes_at = |t: f64| {
        let (l0, lv, lb) = (line(&m0, t), line(&mv, t), line(&mb, t));
        (l0, (lv.tc / l0.tc - 1.0) / 0.05, (lb.tc / l0.tc - 1.0) / 0.02)
    };
    println!("\n== Part 2a: total coolant B(Model1) - A on a 10 K grid (flat; the maps do not enter Tc)");
    for factor in [2.0, 1.5, 1.0, 0.8] {
        let mut prev: Option<(f64, f64)> = None;
        let mut t = 1200.0;
        while t <= 2400.0 {
            let (l0, a1, a2) = slopes_at(t);
            let b = solve(factor, t, l0.tc, l0.f, a1, a2);
            let ra = design_at(t).run(&flight(), 1.0);
            let st = |l: &str| ra.stations.iter().find(|(x, _)| *x == l).unwrap().1;
            let a = solve(factor, t, st("3").tt, st("4").far, 0.0, 0.0);
            let d = (b.0 + b.1) - (a.0 + a.1);
            if let Some((tp, dp)) = prev {
                if dp.signum() != d.signum() {
                    let tc = tp + (t - tp) * dp / (dp - d);
                    let slope = (d - dp) / (t - tp);
                    println!("FACTOR {factor}: crossing at {tc:.1} K (B-A slope {slope:.3e} /K; B {:.4}, A {:.4})", b.0 + b.1, a.0 + a.1);
                }
            }
            prev = Some((t, d));
            t += 10.0;
        }
        println!("FACTOR {factor}: B-A at 1300 / 2400 ends of scan (last d {:+.5})", prev.unwrap().1);
    }

    println!("\n== Part 2b: the knob form, FACTOR 1.0, blade 0.02: each row's own Tt4 limit (Model 1), flat");
    let vane_lim = |e1: f64, e2: f64| -> f64 {
        // the Tt4 at which the vane metal reaches its limit, with Tc(T) and G(T) from the line
        let r = |t: f64| { let (l0, a1, a2) = slopes_at(t);
            let tc = l0.tc * (1.0 + a1 * e1 + a2 * e2);
            let gg = (1.0 - e1 - e2) * (1.0 + l0.f);
            metal(t, tc, 0.75 * e1 / gg, PV) - LV };
        bisect(r, 800.0, 2400.0)
    };
    let blade_lim = |e1: f64, e2: f64| -> f64 {
        let g = cpg();
        let r = |t: f64| { let (l0, a1, a2) = slopes_at(t);
            let tc = l0.tc * (1.0 + a1 * e1 + a2 * e2);
            let gg = (1.0 - e1 - e2) * (1.0 + l0.f);
            let f41 = l0.f * (1.0 - e1 - e2) / (1.0 - e2);
            let t41 = mixed_tt(&g, gg, t, l0.f, e1, tc, f41);
            metal(KREL * t41, tc, 0.75 * e2 / (gg + e1), PB) - LB };
        bisect(r, 800.0, 2400.0)
    };
    for k in 0..=10 {
        let e1 = 0.02 * k as f64;
        let (tv, tb) = (vane_lim(e1, 0.02), blade_lim(e1, 0.02));
        println!("vane {e1:.2}: vane limit {tv:.1} K, blade limit {tb:.1} K -> Tt4_max {:.1} ({})", tv.min(tb),
                 if tv < tb { "vane binds" } else { "blade binds" });
    }
    for e2 in [0.0, 0.01, 0.04] {
        for e1 in [0.0, 0.03, 0.06] {
            let (tv, tb) = (vane_lim(e1, e2), blade_lim(e1, e2));
            println!("vane {e1:.2} blade {e2:.2}: vane {tv:.1}, blade {tb:.1}");
        }
    }
}

fn bisect(f: impl Fn(f64) -> f64, mut lo: f64, mut hi: f64) -> f64 {
    let flo = f(lo);
    assert!(flo * f(hi) < 0.0, "no bracket");
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if f(mid) * flo > 0.0 { lo = mid; } else { hi = mid; }
    }
    0.5 * (lo + hi)
}

/// Gauntner's metal temperature for a row seeing gas `tg` with coolant `tc`, airfoil coolant/gas `wa`
/// (FACTOR 1.0): the hot spot, safety-raised, cooled to the effectiveness the coolant buys.
fn metal(tg: f64, tc: f64, wa: f64, p: f64) -> f64 {
    let t = tg + S;
    let hot = t + p * (t - tc);
    let y = (wa / 0.022).powf(0.8);
    let phi = y / (1.0 + y);
    hot - phi * (hot - tc)
}

/// Part 3 — (a) TSFC on A from the model (burner air (1-eps), f unchanged; thrust via the d-slopes),
/// (b) REACHABILITY: does rung 91's matcher solve at Model 1's B shares (solve/refuse + speeds only),
/// (c) the knob-form crossover bisected.
pub fn part3() {
    let (ml0, mh0) = (maps()[0].1.clone(), maps()[0].2.clone());
    let mk = |e1: f64, e2: f64, ml: &turbojet::map::ComponentMap, mh: &turbojet::map::ComponentMap| {
        let eng = if e1 + e2 > 0.0 { design().with_turbine_cooling(TurbineCooling::new(e1, e2)) } else { design() };
        TurbineCoolingMatcher::new(eng, flight(), 1.0, ml.clone(), mh.clone(), 0.0)
    };
    let (m0, mv, mb) = (mk(0.0, 0.0, &ml0, &mh0), mk(0.05, 0.0, &ml0, &mh0), mk(0.0, 0.02, &ml0, &mh0));
    let tt4s = [1300.0, 1400.0, 1500.0, 1600.0, 1700.0, 1800.0, 1900.0, 2000.0, 2100.0, 2200.0];
    println!("\n== Part 3a: A's TSFC model = TSFC_unc * (1-eps)/(1 + d1 e1 + d2 e2), mg/N.s (flat d-slopes)");
    for factor in [2.0, 1.5, 1.0, 0.8] {
        let mut row = format!("FACTOR {factor}:");
        for &t in &tt4s {
            let (l0, lv, lb) = (line(&m0, t), line(&mv, t), line(&mb, t));
            let d1 = (lv.st / l0.st - 1.0) / 0.05;
            let d2 = (lb.st / l0.st - 1.0) / 0.02;
            let ra = design_at(t).run(&flight(), 1.0);
            let st = |l: &str| ra.stations.iter().find(|(x, _)| *x == l).unwrap().1;
            let a = solve(factor, t, st("3").tt, st("4").far, 0.0, 0.0);
            let tu = ra.performance.tsfc * 1e6;
            let tc = tu * (1.0 - a.0 - a.1) / (1.0 + d1 * a.0 + d2 * a.1);
            row += &format!(" {t:.0}: {tu:.2}->{tc:.2} ({:+.2})", tc - tu);
        }
        println!("{row}");
    }
    println!("\n== Part 3b: reachability of B at Model 1's shares (solve or refuse; speeds)");
    for (name, ml, mh) in maps() {
        for factor in [2.0, 1.5, 1.0, 0.8] {
            let mut row = format!("{name} F {factor}:");
            for &t in &tt4s {
                let (l0, lv, lb) = (line(&m0, t), line(&mv, t), line(&mb, t));
                let a1 = (lv.tc / l0.tc - 1.0) / 0.05;
                let a2 = (lb.tc / l0.tc - 1.0) / 0.02;
                let b = solve(factor, t, l0.tc, l0.f, a1, a2);
                if b.0 >= 0.25 || b.1 >= 0.25 { row += &format!(" {t:.0}:CAP({:.3},{:.3})", b.0, b.1); continue; }
                let m = mk(b.0, b.1, &ml, &mh);
                match m.try_match_point(&flight(), t) {
                    Ok(r) => row += &format!(" {t:.0}:ok N{:.2}/{:.2}", r.base.n_lp_ratio, r.base.n_hp_ratio),
                    Err(e) => row += &format!(" {t:.0}:REFUSED[{}]", &e.0[..e.0.len().min(60)]),
                }
            }
            println!("{row}");
        }
    }
    println!("\n== Part 3c: the knob-form crossover, bisected (FACTOR 1.0, blade 0.02, flat)");
    let slopes_at = |t: f64| {
        let (l0, lv, lb) = (line(&m0, t), line(&mv, t), line(&mb, t));
        (l0, (lv.tc / l0.tc - 1.0) / 0.05, (lb.tc / l0.tc - 1.0) / 0.02)
    };
    let g = cpg();
    let lims = |e1: f64, e2: f64| -> (f64, f64) {
        let rv = |t: f64| { let (l0, a1, a2) = slopes_at(t);
            let tc = l0.tc * (1.0 + a1 * e1 + a2 * e2);
            let gg = (1.0 - e1 - e2) * (1.0 + l0.f);
            metal(t, tc, 0.75 * e1 / gg, PV) - LV };
        let rb = |t: f64| { let (l0, a1, a2) = slopes_at(t);
            let tc = l0.tc * (1.0 + a1 * e1 + a2 * e2);
            let gg = (1.0 - e1 - e2) * (1.0 + l0.f);
            let f41 = l0.f * (1.0 - e1 - e2) / (1.0 - e2);
            let t41 = mixed_tt(&g, gg, t, l0.f, e1, tc, f41);
            metal(KREL * t41, tc, 0.75 * e2 / (gg + e1), PB) - LB };
        (bisect(rv, 800.0, 2400.0), bisect(rb, 800.0, 2400.0))
    };
    let x = bisect(|e1: f64| { let (v, b) = lims(e1, 0.02); v - b }, 0.02, 0.04);
    let (v, b) = lims(x, 0.02);
    println!("crossover vane share {x:.5} (Tt4_max there {v:.1} / {b:.1} K)");
    // its sensitivity: a 10 K shift in either limit moves the crossover by
    let (v1, b1) = lims(x + 0.001, 0.02);
    println!("d(vane lim)/de1 {:.0} K, d(blade lim)/de1 {:.0} K per unit share near the crossover",
             (v1 - v) / 0.001, (b1 - b) / 0.001);
}
