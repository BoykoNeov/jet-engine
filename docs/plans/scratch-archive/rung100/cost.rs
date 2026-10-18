// Rung 100 cost: governed runs, best of 9, rung 100's settings against rung 99's (the reduce settings).
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{controls, ControlsSettings, ThrustBy};
fn best(s: &ControlsSettings) -> f64 { (0..9).map(|_| { let t = Instant::now(); controls(s).unwrap(); t.elapsed().as_secs_f64() }).fold(f64::MAX, f64::min) }
fn main() {
    for (m0, from) in [(0.1, 700.0), (1.4, 1000.0)] { for by in [ThrustBy::Epr, ThrustBy::N1] { for c in [SpeedCorr::Grid, SpeedCorr::Formula] {
        let s = ControlsSettings { m0, from, govern: true, hold: Hold::Thrust, thrust_by: by, speed_corr: c, ..ControlsSettings::defaults() };
        let old = ControlsSettings { low_grid: false, formula_choked: true, ..s };
        println!("Mach {m0} from {from} {by:?} {}: rung 99 {:.3} s, rung 100 {:.3} s", c.key(), best(&old), best(&s));
    } } }
}
