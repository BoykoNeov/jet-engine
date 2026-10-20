// Probe: rung 98's Mach-3 sea-level cell on the live page — the gap's two parts and their sum, at a few points.
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{controls, ControlsSettings, ThrustBy};
use turbojet::mach_axis::SpeedCorr;
fn main() {
    let d = ControlsSettings::defaults();
    let s = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, speed_corr: SpeedCorr::None,
                               t0: 288.15, p0: 101_325.0, m0: 3.0, ..d };
    let (sv, o) = controls(&s).unwrap();
    println!("shape {} from {} to {}; points {}", s.shape, s.from, s.to, o.points.len());
    let n = o.points.len();
    for k in [0, n / 4, n / 2, 3 * n / 4, n - 1] {
        let r = sv.read(&o.points[k]).unwrap();
        println!("point {k}: thrust {:.1} belief {:.1} gap {:.1} = ram part {:?} + nozzle part {:?}", r.thrust,
                 r.inferred.unwrap(), r.thrust - r.inferred.unwrap(), r.gap_ram, r.gap_nozzle);
    }
    let (a, b) = (sv.hold_start.unwrap(), sv.hold_end.unwrap());
    println!("start target {:.1} achieved {:.1}; end target {:.1} achieved {:.1}", a.target, a.achieved, b.target, b.achieved);
}
