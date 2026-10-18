// Rung 98 pre-anchor, after the advisor's review:
//  (a) every scoring cell: do the design-flight rows (Tt4 in linspace(1000,1400,13)·Tt2_d/Tt2 — the set all three
//      tables use for the default move) solve, choked, and rise? y/n only.
//  (b) the cross-flight decomposition identity on STEADY points at each scoring flight, every variant: the SPREAD over
//      the run's throttles of  F_true − δ·F_tab(e) − [δ·V0_d·ṁ_tab(e) − V0·ṁ_face]  over the largest thrust. The
//      constant itself is NOT printed.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::engine::FlightCondition;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face, Reading, Row, Table};
use turbojet::sandbox_controls::{ControlsSettings, CONTROL_SHAPES};

fn main() {
    let flights = [("hot 290K", 290.0, 50_000.0, 0.85), ("cold 216.65K", 216.65, 50_000.0, 0.85),
                   ("thin 26.5kPa", 250.0, 26_500.0, 0.85), ("dense 80kPa", 250.0, 80_000.0, 0.85),
                   ("M0.5", 250.0, 50_000.0, 0.5), ("M1.4", 250.0, 50_000.0, 1.4)];
    println!("# (a) design-flight rows per scoring cell (default move)");
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..ControlsSettings::defaults() };
        let sv = machine(&s);
        let fd = s.design_flight();
        let (tt2d, pt2d, _) = st2(&sv, &fd);
        for (name, t0, p0, m) in flights {
            let fl = FlightCondition::new(t0, p0, m);
            let k = tt2d / st2(&sv, &fl).0;
            let mut rows = Vec::new();
            let mut fail = None;
            let mut unch = 0;
            for j in 0..13 {
                let t = (1000.0 + 400.0 * j as f64 / 12.0) * k;
                match sv.core.fuel.inner.try_equilibrium(&fd, t, None) {
                    Ok((i, _, _)) => { if !choked(&i) { unch += 1; } rows.push(Row::of(&i, pt2d)) }
                    Err(_) => { fail = Some(t); break; }
                }
            }
            rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
            let rise = rows.windows(2).all(|w| w[1].thrust > w[0].thrust && w[1].epr > w[0].epr && w[1].nu_lp > w[0].nu_lp && w[1].fuel > w[0].fuel);
            println!("(a) {:10} {:13} rows {:.0}–{:.0} K: {}", CONTROL_SHAPES[shape], name, 1000.0 * k, 1400.0 * k,
                     match fail { Some(t) => format!("FAIL at {t:.0} K"), None => format!("built, rise {rise}, unchoked rows {unch}") });
        }
    }
    println!("# (b) decomposition spread / max thrust over flight Tt4 in 1000..1400 (9 points, choked ones), every variant");
    let mut worst = 0.0f64;
    for shape in 0..4 {
        for (vname, v) in variants() {
            let s = ControlsSettings { shape, ..v };
            let sv = machine(&s);
            let fd = s.design_flight();
            let (tt2d, pt2d, v0d) = st2(&sv, &fd);
            for (name, t0, p0, m) in flights {
                let fl = FlightCondition::new(t0, p0, m);
                let (tt2, pt2, v0) = st2(&sv, &fl);
                let (k, de) = (tt2d / tt2, pt2 / pt2d);
                let mut rows = Vec::new();
                let mut ok = true;
                for j in 0..13 {
                    match sv.core.fuel.inner.try_equilibrium(&fd, (1000.0 + 400.0 * j as f64 / 12.0) * k, None) {
                        Ok((i, _, _)) => rows.push(Row::of(&i, pt2d)),
                        Err(_) => { ok = false; break; }
                    }
                }
                if !ok { println!("(b) {:10} {:18} {:13}: no table", CONTROL_SHAPES[shape], vname, name); continue; }
                rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
                let tab = Table { rows, pt2: pt2d, v0: v0d };
                let (mut lo, mut hi, mut fmax, mut n, mut sk) = (f64::INFINITY, f64::NEG_INFINITY, 0.0f64, 0, 0);
                for j in 0..9 {
                    let t = 1000.0 + 400.0 * j as f64 / 8.0;
                    let Ok((i, _, _)) = sv.core.fuel.inner.try_equilibrium(&fl, t, None) else { sk += 1; continue };
                    if !choked(&i) { sk += 1; continue; }
                    let e = epr(&i, pt2);
                    let ft = thrust(&i);
                    let r = ft - de * tab.inferred(Reading::Epr, e) - (de * v0d * tab.face_at_epr(e) - v0 * face(&i));
                    lo = lo.min(r); hi = hi.max(r); fmax = fmax.max(ft.abs()); n += 1;
                }
                let sp = if n > 1 { (hi - lo) / fmax } else { f64::NAN };
                if sp.is_finite() { worst = worst.max(sp); }
                println!("(b) {:10} {:18} {:13}: spread {:.1e} over {n} points (skipped {sk})", CONTROL_SHAPES[shape], vname, name, sp);
            }
        }
    }
    println!("(b) WORST spread {worst:.2e}");
}
