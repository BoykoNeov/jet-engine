// Rung 99 pre-anchor, after the advisor's review: the curve the grid interpolates, from the MODEL's own quantities.
// At a fixed corrected engine with a choked nozzle, net thrust over pt2 is φ(M) = C − a8·g(M) − ṁc·h(M), with
// g = p0/pt2 and h = V0/√Tt2 taken from the core's own inlet() at each flight, a8 the model's nozzle area, ṁc the
// corrected face airflow (face·√Tt2/pt2) at the flight's steady point. The grid's interpolation error at equal
// corrected state is then pt2·{−a8·[(1−w)g_a + w·g_b − g] − ṁc·[(1−w)h_a + w·h_b − h]} — the table's belief minus the
// truth, NEGLECTING the temperature residual. Printed per map, flight and end, as a fraction of the end's thrust, with
// its nozzle and drag parts. State + a8 only: no table is built or read.
// Also (H5) the start end at Mach 0.05 / 0.15 / 0.5, 700 K: pt9/p0, the second-order nozzle term of a choked-nozzle
// formula (γ 1.30 and 1.36) over the thrust, and the airflow-guess bound V0·ṁ_face·5 % over the thrust.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::engine::FlightCondition;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face};
use turbojet::sandbox_controls::{ControlsSettings, CONTROL_SHAPES};

const GRID: [f64; 8] = [0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0];

fn bracket(m: f64) -> (usize, usize) {
    if m < GRID[0] { return (0, 1); }
    if m > GRID[7] { return (6, 7); }
    let j = GRID.iter().rposition(|&x| x < m).unwrap();
    (j, j + 1)
}

fn main() {
    println!("# grid interpolation error at fixed corrected state: belief − truth over thrust (−ve of the miss); nozzle | drag parts");
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..ControlsSettings::defaults() };
        let sv = machine(&s);
        let fd = s.design_flight();
        let a8 = sv.core.fuel.inner.inner.base.a8;
        let gh = |m: f64| { let fl = FlightCondition::new(fd.t0, fd.p0, m); let (tt2, pt2, v0) = st2(&sv, &fl); (fd.p0 / pt2, v0 / tt2.sqrt()) };
        for m in [0.05, 0.15, 0.5, 0.9, 1.05, 1.1, 1.4, 3.2] {
            let (a, b) = bracket(m);
            let w = (m - GRID[a]) / (GRID[b] - GRID[a]);
            let ((ga, ha), (gb, hb), (g, h)) = (gh(GRID[a]), gh(GRID[b]), gh(m));
            let fl = FlightCondition::new(fd.t0, fd.p0, m);
            let (tt2, pt2, _) = st2(&sv, &fl);
            let mut out = format!("{:10} M{m:<4} (slices {}/{}, w {w:+.3}):", CONTROL_SHAPES[shape], GRID[a], GRID[b]);
            for t in [700.0, 1000.0, 1400.0] {
                let Ok((i, _, _)) = sv.core.fuel.inner.try_equilibrium(&fl, t, None) else { out += &format!(" {t:.0}K none;"); continue };
                let mc = face(&i) * tt2.sqrt() / pt2;
                let noz = -a8 * ((1.0 - w) * ga + w * gb - g) * pt2;
                let drag = -mc * ((1.0 - w) * ha + w * hb - h) * pt2;
                let f = thrust(&i);
                out += &format!(" {t:.0}K{}: {:+.3}% ({:+.3} | {:+.3});", if choked(&i) { "" } else { "(unch)" },
                                100.0 * (noz + drag) / f, 100.0 * noz / f, 100.0 * drag / f);
            }
            println!("{out}");
        }
    }
    println!("# H5 ingredients: the 700 K start, unchoked");
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..ControlsSettings::defaults() };
        let sv = machine(&s);
        let fd = s.design_flight();
        let a8 = sv.core.fuel.inner.inner.base.a8;
        let pi_n = sv.core.fuel.inner.inner.base.pi_n;
        for m in [0.05, 0.15, 0.5] {
            let fl = FlightCondition::new(fd.t0, fd.p0, m);
            let (_, pt2, v0) = st2(&sv, &fl);
            for t in [700.0, 750.0] {
                let Ok((i, _, _)) = sv.core.fuel.inner.try_equilibrium(&fl, t, None) else { continue };
                if choked(&i) { println!("{:10} M{m} {t:.0}K choked", CONTROL_SHAPES[shape]); continue; }
                let r = pi_n * epr(&i, pt2) * pt2 / fl.p0;
                let f = thrust(&i);
                let term = |gm: f64| {
                    let x = r * (2.0 / (gm + 1.0)).powf(gm / (gm - 1.0));
                    let m9sq = 2.0 / (gm - 1.0) * (r.powf((gm - 1.0) / gm) - 1.0);
                    a8 * (fl.p0 * gm * m9sq - ((1.0 + gm) * x * fl.p0 - fl.p0)) / f
                };
                println!("{:10} M{m} {t:.0}K: pt9/p0 {r:.4}; nozzle term (true − choked formula)/F: γ1.30 {:+.4}%, γ1.36 {:+.4}%; airflow bound V0·ṁ·5%/F {:.4}%",
                         CONTROL_SHAPES[shape], 100.0 * term(1.30), 100.0 * term(1.36), 100.0 * v0 * face(&i) * 0.05 / f);
            }
        }
    }
}
