// Rung 99 pre-anchor given: can the two BRACKETING grid slices be built for a run flown elsewhere? Slices at the
// design altitude's air (250 K, 50 kPa) at Mach 0.3/0.6/0.85/1.2/1.6/2.0; the two bracketing (or, off the grid, the
// two end) slices, each on rows at Tt4·(Tt2_slice/Tt2). Prints only solve status, rise check, unchoked-row counts.
// No thrust, no reading. Rung 98's single design slice is printed beside it for comparison of REACH only.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::atmosphere::Ambient;
use turbojet::engine::FlightCondition;
use turbojet::inferred_thrust::Row;
use turbojet::sandbox_controls::{ControlsSettings, CONTROL_SHAPES};

fn grid() -> Vec<f64> {
    std::env::var("GRID").map(|g| g.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![0.3, 0.6, 0.85, 1.2, 1.6, 2.0])
}

fn bracket(m: f64) -> Vec<usize> {
    let g = grid();
    let n = g.len();
    if let Some(j) = g.iter().position(|&x| x == m) { return vec![j]; }
    if m < g[0] { return vec![0, 1]; }
    if m > g[n - 1] { return vec![n - 2, n - 1]; }
    let j = g.iter().rposition(|&x| x < m).unwrap();
    vec![j, j + 1]
}

fn main() {
    println!("# grid {:?}", grid());
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..ControlsSettings::defaults() };
        let sv = machine(&s);
        let fd = s.design_flight();
        let (tt2d, _, _) = st2(&sv, &fd);
        for (from, to) in [(1000.0, 1400.0), (700.0, 1400.0), (1400.0, 1000.0)] {
            let (mut n, mut own_bad, mut ok, mut ok98) = (0, 0, 0, 0);
            let (mut bad, mut rise_bad, mut unch_tabs) = (Vec::new(), Vec::new(), Vec::new());
            for z in [0.0, 3000.0, 5500.0, 8000.0, 11000.0, 15000.0] {
                for dt in [-30.0, 0.0, 30.0] {
                    for m in [0.05, 0.15, 0.3, 0.5, 0.85, 1.4, 1.6, 2.5, 3.0] {
                        n += 1;
                        let a = Ambient::from_altitude(z, dt);
                        let fl = FlightCondition::new(a.t0, a.p0, m);
                        let eq = |f: &FlightCondition, t: f64| sv.core.fuel.inner.try_equilibrium(f, t, None).map(|x| x.0);
                        if eq(&fl, from).is_err() || eq(&fl, to).is_err() { own_bad += 1; continue; }
                        let (tt2, _, _) = st2(&sv, &fl);
                        let tag = format!("{:.0}km{:+.0}K M{}", z / 1000.0, dt, m);
                        // rung 98's single design slice, reach only
                        if (0..13).all(|j| eq(&fd, (from + (to - from) * j as f64 / 12.0) * tt2d / tt2).is_ok()) { ok98 += 1; }
                        let mut failed = false;
                        let mut unch = 0;
                        for &j in &bracket(m) {
                            let sl = FlightCondition::new(fd.t0, fd.p0, grid()[j]);
                            let (tt2s, pt2s, _) = st2(&sv, &sl);
                            let k = tt2s / tt2;
                            let mut rows = Vec::new();
                            for r in 0..13 {
                                let t = (from + (to - from) * r as f64 / 12.0) * k;
                                match eq(&sl, t) {
                                    Ok(i) => { if !choked(&i) { unch += 1; } rows.push(Row::of(&i, pt2s)) }
                                    Err(_) => { bad.push(format!("{tag}[slice M{} {:.0}K]", grid()[j], t)); failed = true; break; }
                                }
                            }
                            if failed { break; }
                            rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
                            if !rows.windows(2).all(|w| w[1].thrust > w[0].thrust && w[1].epr > w[0].epr && w[1].nu_lp > w[0].nu_lp && w[1].fuel > w[0].fuel) {
                                rise_bad.push(format!("{tag}[slice M{}]", grid()[j])); failed = true; break;
                            }
                        }
                        if failed { continue; }
                        if unch > 0 { unch_tabs.push(format!("{tag}:{unch}")); }
                        ok += 1;
                    }
                }
            }
            println!("{:10} {from}->{to}: flights {n}, own ends refused {own_bad}, grid slices built {ok} (rung-98 design slice {ok98}), \
                      slice fails {}, rise fails {}, flights with unchoked slice rows {}",
                     CONTROL_SHAPES[shape], bad.len(), rise_bad.len(), unch_tabs.len());
            if !bad.is_empty() { println!("   slice fails: {}", bad.join(" ")); }
            if !rise_bad.is_empty() { println!("   rise fails: {}", rise_bad.join(" ")); }
            if !unch_tabs.is_empty() { println!("   unchoked rows (flight:count over both slices): {}", unch_tabs.join(" ")); }
        }
    }
}
