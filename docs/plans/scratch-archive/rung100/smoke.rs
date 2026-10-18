// Rung 100 smoke (post-build, not scored): formula EPR / fan at an unchoked start, new vs rung 99's column; time.
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ThrustBy, CONTROL_SHAPES};
fn main() {
    for shape in [0usize, 3] {
        for m0 in [0.05, 0.5, 1.4] {
            for by in [ThrustBy::Epr, ThrustBy::N1] {
                for ch in [true, false] {
                    let from = if m0 == 1.4 && shape == 3 { 1000.0 } else { 700.0 };
                    let s = ControlsSettings { shape, govern: true, hold: Hold::Thrust, thrust_by: by, speed_corr: SpeedCorr::Formula,
                                               formula_choked: ch, from, t0: 250.0, p0: 50_000.0, m0, ..ControlsSettings::defaults() };
                    let t = Instant::now();
                    match build(&s) {
                        Ok(sv) => {
                            let (a, b) = (sv.hold_start.unwrap(), sv.hold_end.unwrap());
                            let tb = t.elapsed().as_secs_f64();
                            let t2 = Instant::now();
                            let run = controls(&s);
                            let tr = t2.elapsed().as_secs_f64();
                            let (n, worst) = match &run {
                                Ok((sv, o)) => { let mut w = 0.0f64; for p in &o.points { let r = sv.read(p).unwrap(); if let Some(x) = r.gap_nozzle { w = w.max(x.abs()) } } (o.points.len(), w) }
                                Err(e) => { println!("   run refused: {}", e.0); (0, f64::NAN) }
                            };
                            println!("{:10} M{m0:<4} {from} {by:?} choked_col {ch:5}: miss start {:+.4}% end {:+.4}% | build {tb:.3}s run {tr:.3}s pts {n} | worst |rest| {worst:.2e}",
                                     CONTROL_SHAPES[shape], 100.0 * (a.achieved / a.target - 1.0), 100.0 * (b.achieved / b.target - 1.0));
                        }
                        Err(e) => println!("{:10} M{m0} {by:?} {ch}: REFUSED {}", CONTROL_SHAPES[shape], e.0),
                    }
                }
            }
        }
    }
}
