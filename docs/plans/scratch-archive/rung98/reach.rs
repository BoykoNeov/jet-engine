// Rung 98 pre-anchor given G5: can the design-flight tables be BUILT for a run flown elsewhere? Rows at the
// corrected-equivalent throttles Tt4·(Tt2_d/Tt2) at the DESIGN flight. Prints only: the run's own ends solve (y/n),
// the 13 design-flight rows solve (y/n, first failing Tt4), and rung 96's rise check on them. No thrust, no reading.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::atmosphere::Ambient;
use turbojet::engine::FlightCondition;
use turbojet::inferred_thrust::Row;
use turbojet::sandbox_controls::{ControlsSettings, CONTROL_SHAPES};

fn main() {
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..ControlsSettings::defaults() };
        let sv = machine(&s);
        let fd = s.design_flight();
        let (tt2d, pt2d, _) = st2(&sv, &fd);
        for (from, to) in [(1000.0, 1400.0), (700.0, 1400.0), (1400.0, 1000.0)] {
            let (mut n, mut own_bad, mut ok, mut rows_bad, mut rise_bad) = (0, 0, 0, Vec::new(), Vec::new());
            for z in [0.0, 3000.0, 5500.0, 8000.0, 11000.0, 15000.0] {
                for dt in [-30.0, 0.0, 30.0] {
                    for m in [0.3, 0.85, 1.6, 2.5, 3.0] {
                        n += 1;
                        let a = Ambient::from_altitude(z, dt);
                        let fl = FlightCondition::new(a.t0, a.p0, m);
                        let eq = |f: &FlightCondition, t: f64| sv.core.fuel.inner.try_equilibrium(f, t, None).map(|x| x.0);
                        if eq(&fl, from).is_err() || eq(&fl, to).is_err() { own_bad += 1; continue; }
                        let (tt2, pt2, _) = st2(&sv, &fl);
                        let k = tt2d / tt2;
                        let _ = pt2;
                        let mut rows = Vec::new();
                        let mut fail = None;
                        for j in 0..13 {
                            let t = (from + (to - from) * j as f64 / 12.0) * k;
                            match eq(&fd, t) { Ok(i) => rows.push(Row::of(&i, pt2d)), Err(_) => { fail = Some(t); break; } }
                        }
                        let tag = format!("{:.0}km{:+.0}K M{}", z / 1000.0, dt, m);
                        if let Some(t) = fail { rows_bad.push(format!("{tag}(θ{:.2},{:.0}K)", tt2 / tt2d, t)); continue; }
                        rows.sort_by(|a, b| a.thrust.total_cmp(&b.thrust));
                        if !rows.windows(2).all(|w| w[1].thrust > w[0].thrust && w[1].epr > w[0].epr && w[1].nu_lp > w[0].nu_lp && w[1].fuel > w[0].fuel) {
                            rise_bad.push(tag); continue;
                        }
                        ok += 1;
                    }
                }
            }
            println!("{:10} {from}->{to}: flights {n}, own ends refused {own_bad}, table built {ok}, rows fail {}, rise fails {}",
                     CONTROL_SHAPES[shape], rows_bad.len(), rise_bad.len());
            println!("   rows fail: {}", rows_bad.join(" "));
            if !rise_bad.is_empty() { println!("   rise fails: {}", rise_bad.join(" ")); }
        }
    }
}
