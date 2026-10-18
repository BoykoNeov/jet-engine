// Rung 99 pre-anchor: do the scoring cells run? Flights at the design air (250 K, 50 kPa), Mach 0.15 / 0.5 / 1.4,
// moves 1000→1400 and 700→1400, every map. Prints y/n and counts only:
//  - the run's own ends solve; which are unchoked;
//  - the bracketing grid slices' 13 rows (or end pair below 0.3) solve, rise, unchoked count;
//  - rung 98's design-flight rows (what the formula reads) solve, unchoked count;
//  - the formula's nozzle constant: at choked flight points whose design rows are all choked, the G6 constant against
//    base.a8·(δ·p0_d − p0), worst relative difference (the formula's one ingredient beyond the table).
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::engine::FlightCondition;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face, Reading, Row, Table};
use turbojet::sandbox_controls::{ControlsSettings, CONTROL_SHAPES};

const GRID: [f64; 8] = [0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0];

fn bracket(m: f64) -> Vec<usize> {
    if let Some(j) = GRID.iter().position(|&x| x == m) { return vec![j]; }
    if m < GRID[0] { return vec![0, 1]; }
    let j = GRID.iter().rposition(|&x| x < m).unwrap();
    vec![j, j + 1]
}

fn main() {
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..ControlsSettings::defaults() };
        let sv = machine(&s);
        let fd = s.design_flight();
        let (tt2d, pt2d, v0d) = st2(&sv, &fd);
        let a8 = sv.core.fuel.inner.inner.base.a8;
        let eq = |f: &FlightCondition, t: f64| sv.core.fuel.inner.try_equilibrium(f, t, None).map(|x| x.0);
        for m in [0.05, 0.15, 0.5, 1.4] {
            let fl = FlightCondition::new(fd.t0, fd.p0, m);
            let (tt2, pt2, v0) = st2(&sv, &fl);
            let de = pt2 / pt2d;
            for (from, to) in [(1000.0, 1400.0), (700.0, 1400.0)] {
                let mut out = format!("{:10} M{m:<4} {from}->{to}:", CONTROL_SHAPES[shape]);
                let ends: Vec<_> = [from, to].iter().map(|&t| eq(&fl, t)).collect();
                if ends.iter().any(|e| e.is_err()) { println!("{out} own ends REFUSED"); continue; }
                let unch_ends: Vec<String> = [from, to].iter().zip(&ends).filter(|(_, e)| !choked(e.as_ref().unwrap()))
                    .map(|(t, _)| format!("{t:.0}")).collect();
                out += &format!(" ends ok (unchoked: {})", if unch_ends.is_empty() { "none".into() } else { unch_ends.join(",") });
                for &j in &bracket(m) {
                    let sl = FlightCondition::new(fd.t0, fd.p0, GRID[j]);
                    let (tt2s, pt2s, _) = st2(&sv, &sl);
                    let mut rows = Vec::new();
                    let mut unch = 0;
                    let mut fail = None;
                    for r in 0..13 {
                        let t = (from + (to - from) * r as f64 / 12.0) * tt2s / tt2;
                        match eq(&sl, t) { Ok(i) => { if !choked(&i) { unch += 1; } rows.push(Row::of(&i, pt2s)) } Err(_) => { fail = Some(t); break; } }
                    }
                    rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
                    let rise = rows.windows(2).all(|w| w[1].thrust > w[0].thrust && w[1].epr > w[0].epr && w[1].nu_lp > w[0].nu_lp && w[1].fuel > w[0].fuel);
                    out += &match fail { Some(t) => format!(" | slice M{} FAIL {t:.0}K", GRID[j]),
                                         None => format!(" | slice M{} built rise {rise} unchoked {unch}", GRID[j]) };
                }
                // rung 98's design-flight rows
                let mut rows = Vec::new();
                let mut unch = 0;
                let mut ok = true;
                for r in 0..13 {
                    let t = (from + (to - from) * r as f64 / 12.0) * tt2d / tt2;
                    match eq(&fd, t) { Ok(i) => { if !choked(&i) { unch += 1; } rows.push(Row::of(&i, pt2d)) } Err(_) => { ok = false; break; } }
                }
                if !ok { println!("{out} | design rows FAIL"); continue; }
                out += &format!(" | design rows unchoked {unch}");
                rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
                let all_choked = unch == 0;
                let tab = Table { rows, pt2: pt2d, v0: v0d };
                let want = a8 * (de * fd.p0 - fl.p0);
                let mut worst = 0.0f64;
                let mut n = 0;
                for j in 0..9 {
                    let t = from + (to - from) * j as f64 / 8.0;
                    let Ok(i) = eq(&fl, t) else { continue };
                    if !choked(&i) || !all_choked { continue; }
                    let e = epr(&i, pt2);
                    let ft = thrust(&i);
                    let r = ft - de * tab.inferred(Reading::Epr, e) - (de * v0d * tab.face_at_epr(e) - v0 * face(&i));
                    worst = worst.max(((r - want) / ft).abs());
                    n += 1;
                }
                out += &format!(" | formula constant vs a8·(δp0_d−p0): worst {worst:.1e} over {n} pts");
                println!("{out}");
            }
        }
    }
}
