// Rung 100 pre-anchor, unscored flights: the exact pressure-ratio push leaves only the airflow guess. At each run's
// calibrated START (rung 99's formula machinery, its carried table), F_true − (push(EPR) − V0·ṁ̂(EPR)) over F_true, and
// the airflow guess V0·(ṁ̂ − ṁ_face)/F alone. No rung-100 code.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::burner_hold::Hold;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face};
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ThrustBy, CONTROL_SHAPES};
fn main() {
    let v = variants();
    for (name, s) in [("plain", v[0].1), ("cool", v[6].1), ("cust", v[4].1)] {
        for shape in 0..4 {
            for &(t0, p0, m0) in &[(250.0, 50_000.0, 0.1), (250.0, 50_000.0, 0.3), (250.0, 50_000.0, 0.4), (288.15, 101_325.0, 0.1), (288.15, 101_325.0, 0.3)] {
                for from in [700.0, 1000.0] {
                    let st = ControlsSettings { shape, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, speed_corr: SpeedCorr::Formula, from, t0, p0, m0, ..s };
                    let Ok(sv) = build(&st) else { println!("{name} {} {t0} M{m0} {from}: refused", CONTROL_SHAPES[shape]); continue };
                    let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
                    let base = &sv.core.fuel.inner.inner.base;
                    let i = &sv.start;
                    let e = epr(i, t.pt2);
                    let g = 1.3f64;
                    let pt9 = base.pi_n * e * t.pt2;
                    let crit = (2.0 / (g + 1.0)).powf(g / (g - 1.0));
                    let push = if pt9 * crit > p0 { base.a8 * (pt9 * crit * (1.0 + g) - p0) }
                               else { let m2 = 2.0 / (g - 1.0) * ((pt9 / p0).powf((g - 1.0) / g) - 1.0); base.a8 * g * p0 * m2 };
                    let mh = t.face_at_epr(e);
                    let ft = thrust(i);
                    println!("{name:5} {:10} {t0:.0}K M{m0:<4} {from:<5} {} | new formula miss {:+.4}% | airflow guess {:+.4}% | rung 99 formula {:+.4}%",
                             CONTROL_SHAPES[shape], if choked(i) { "ch" } else { "UN" },
                             100.0 * (ft - (push - t.v0 * mh)) / ft, 100.0 * t.v0 * (mh - face(i)) / ft, 100.0 * (ft - t.inferred(turbojet::inferred_thrust::Reading::Epr, e)) / ft);
                }
            }
        }
    }
}
