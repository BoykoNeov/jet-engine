// Rung 99 cost: best of 9, a governed EPR run / the schedule / the proportional law at Mach 1.4, each speed correction.
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{controls, ControlsSettings, SpeedLaw, ThrustBy};
fn main() {
    let d = ControlsSettings { m0: 1.4, ..ControlsSettings::defaults() };
    for (name, s) in [("governed EPR", ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d }),
                      ("schedule", ControlsSettings { accel_on: true, ..d }),
                      ("prop law", ControlsSettings { govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, ..d })] {
        let mut line = format!("{name:14}");
        for c in [SpeedCorr::None, SpeedCorr::Grid, SpeedCorr::Formula] {
            let s = ControlsSettings { speed_corr: c, ..s };
            let best = (0..9).map(|_| { let t = std::time::Instant::now(); controls(&s).unwrap(); t.elapsed().as_secs_f64() })
                .fold(f64::INFINITY, f64::min);
            line += &format!("  {}: {best:.3} s", c.key());
        }
        println!("{line}");
    }
}
