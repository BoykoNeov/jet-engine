// Rung 98: a run's cost with the tables made at the design flight against at this flight, best of 9, interleaved.
use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::TableAt;
use turbojet::sandbox_controls::{controls, ControlsSettings, SpeedLaw, ThrustBy};
fn main() {
    let d = ControlsSettings { t0: 290.0, ..ControlsSettings::defaults() };
    for (name, s) in [("EPR governed", ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d }),
                      ("accel schedule", ControlsSettings { accel_on: true, ..d }),
                      ("prop speed law", ControlsSettings { govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, ..d })] {
        let (mut a, mut b) = (f64::MAX, f64::MAX);
        for _ in 0..9 {
            for (t, at) in [(&mut a, TableAt::Design), (&mut b, TableAt::Flight)] {
                let t0 = std::time::Instant::now();
                controls(&ControlsSettings { table_at: at, ..s }).unwrap();
                *t = t.min(t0.elapsed().as_secs_f64());
            }
        }
        println!("{name:15}: design-flight tables {a:.3} s, this-flight tables {b:.3} s");
    }
}
