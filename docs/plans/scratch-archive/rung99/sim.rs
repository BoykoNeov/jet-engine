// Rung 99 pre-anchor givens.
//  (a) Across the grid slices (250 K, 50 kPa, Mach 0.3..2.0), at the same corrected throttle T*·θ_j (T* 700..1400 at
//      the design flight's terms), how far does each NON-thrust corrected column differ from the Mach-0.85 slice's?
//      Split choked / unchoked; unchoked counts per slice. The thrust column is NOT printed (the predictions are about it).
//  (b) The rung-98 split identity (G6) for a table made at Mach 0.85 read at other flights: the constant is taken from
//      the choked points; COUNT the unchoked points where it fails by more than 1e−9 of the thrust. No values printed.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::engine::FlightCondition;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face, Reading, Row, Table};
use turbojet::sandbox_controls::{ControlsSettings, CONTROL_SHAPES};

const GRID: [f64; 6] = [0.3, 0.6, 0.85, 1.2, 1.6, 2.0];

fn main() {
    println!("# (a) worst |rel diff| vs the M0.85 slice at equal corrected throttle; columns {:?}", &NAMES[1..]);
    for shape in 0..4 {
        for (vname, v) in variants() {
            let s = ControlsSettings { shape, ..v };
            let sv = machine(&s);
            let fd = s.design_flight();
            let (tt2d, pt2d, _) = st2(&sv, &fd);
            let d2 = (tt2d, pt2d);
            let base: Vec<Option<[f64; 8]>> = (0..8).map(|j| {
                let t = 700.0 + 100.0 * j as f64;
                sv.core.fuel.inner.try_equilibrium(&fd, t, None).ok().map(|(i, _, _)| corrected(&sv, &fd, d2, &i))
            }).collect();
            for &m in &GRID {
                if m == 0.85 { continue; }
                let sl = FlightCondition::new(fd.t0, fd.p0, m);
                let th = st2(&sv, &sl).0 / tt2d;
                let (mut wc, mut wu) = ([0.0f64; 7], [0.0f64; 7]);
                let (mut nu, mut nf, mut nc) = (0, 0, 0);
                for j in 0..8 {
                    let t = (700.0 + 100.0 * j as f64) * th;
                    let (Some(b), Ok((i, _, _))) = (base[j], sv.core.fuel.inner.try_equilibrium(&sl, t, None)) else { nf += 1; continue };
                    let c = corrected(&sv, &sl, d2, &i);
                    let w = if choked(&i) { nc += 1; &mut wc } else { nu += 1; &mut wu };
                    for k in 0..7 { w[k] = w[k].max(rel(c[k + 1], b[k + 1])); }
                }
                let f = |w: &[f64; 7]| w.iter().map(|x| format!("{x:.1e}")).collect::<Vec<_>>().join(" ");
                println!("(a) {:10} {:18} M{m:<4} θ{th:.3}: choked {nc} [{}] | unchoked {nu} [{}] | failed {nf}",
                         CONTROL_SHAPES[shape], vname, f(&wc), if nu > 0 { f(&wu) } else { "-".into() });
            }
        }
    }
    println!("# (b) G6 split with a Mach-0.85 (design) table: unchoked points failing the choked constant by > 1e−9·F");
    let flights = [0.05, 0.15, 0.3, 0.5, 0.6, 1.2, 1.4, 1.6, 2.0, 2.5];
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..ControlsSettings::defaults() };
        let sv = machine(&s);
        let fd = s.design_flight();
        let (tt2d, pt2d, v0d) = st2(&sv, &fd);
        for &m in &flights {
            let fl = FlightCondition::new(fd.t0, fd.p0, m);
            let (tt2, pt2, v0) = st2(&sv, &fl);
            let (k, de) = (tt2d / tt2, pt2 / pt2d);
            let mut rows = Vec::new();
            let mut ok = true;
            for j in 0..13 {
                match sv.core.fuel.inner.try_equilibrium(&fd, (700.0 + 700.0 * j as f64 / 12.0) * k, None) {
                    Ok((i, _, _)) => rows.push(Row::of(&i, pt2d)),
                    Err(_) => { ok = false; break; }
                }
            }
            if !ok { println!("(b) {:10} M{m}: no design table", CONTROL_SHAPES[shape]); continue; }
            rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
            let tab = Table { rows, pt2: pt2d, v0: v0d };
            let (mut rc, mut ru) = (Vec::new(), Vec::new());
            let mut fmax = 0.0f64;
            let mut nf = 0;
            for j in 0..15 {
                let t = 700.0 + 50.0 * j as f64;
                let Ok((i, _, _)) = sv.core.fuel.inner.try_equilibrium(&fl, t, None) else { nf += 1; continue };
                let e = epr(&i, pt2);
                let ft = thrust(&i);
                let r = ft - de * tab.inferred(Reading::Epr, e) - (de * v0d * tab.face_at_epr(e) - v0 * face(&i));
                fmax = fmax.max(ft.abs());
                if choked(&i) { rc.push(r) } else { ru.push(r) }
            }
            let c = rc.iter().sum::<f64>() / rc.len().max(1) as f64;
            let spread = rc.iter().map(|r| (r - c).abs()).fold(0.0, f64::max) / fmax;
            let fails = ru.iter().filter(|r| ((*r - c) / fmax).abs() > 1e-9).count();
            println!("(b) {:10} M{m:<4}: choked {} (spread {spread:.1e}), unchoked {} of which fail {fails}, no point {nf}",
                     CONTROL_SHAPES[shape], rc.len(), ru.len());
        }
    }
}
