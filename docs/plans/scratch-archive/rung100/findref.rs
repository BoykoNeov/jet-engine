// Find requests the fan-speed formula refuses in its own words (for a gate), and count them.
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ThrustBy};
fn main() {
    let (mut n, mut r) = (0, 0);
    for (t0, p0) in [(288.15, 101_325.0), (318.15, 101_325.0), (250.0, 50_000.0)] { for m0 in [0.05, 0.1, 0.2, 0.3] { for from in [600.0, 650.0, 700.0] { for cool in [0.0, 0.05] {
        let s = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_corr: SpeedCorr::Formula, from, t0, p0, m0,
                                   cool_vane: cool, cool_blade: cool, ..ControlsSettings::defaults() };
        n += 1;
        if let Err(e) = build(&s) { if e.0.contains("fan-speed formula") { r += 1; println!("{t0} {p0} M{m0} {from} cool {cool}: {}", e.0); } }
    } } } }
    println!("{r} of {n} refused by the fan formula");
}
