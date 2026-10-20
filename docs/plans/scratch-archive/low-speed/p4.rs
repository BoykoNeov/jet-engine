// Probe 4 (lowered-wall worktree): the design flight's low starts with the acceleration schedule ON — do they finish?
// And the remaining first-step stops: the start's mixture.
use turbojet::sandbox_controls::{controls, ControlsSettings, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings::defaults();
    for accel in [false, true] {
        for (k, shape) in CONTROL_SHAPES.iter().enumerate() {
            let mut line = String::new();
            let mut tt4 = 400.0;
            while tt4 <= 560.0 + 1e-9 {
                match controls(&ControlsSettings { shape: k, from: tt4, accel_on: accel, ..d }) {
                    Ok((sv, o)) => line += &format!(" {tt4:.0}:{}/{}p,f0={:.4},fend={:.4}", o.points.len(), sv.settings.expected_points(), sv.start.close.f, o.points.last().map(|p| p.f).unwrap_or(f64::NAN)),
                    Err(e) => line += &format!(" {tt4:.0}:refused({})", format!("{e:?}").chars().take(60).collect::<String>()),
                }
                tt4 += 20.0;
            }
            println!("accel {accel} | {shape}:{line}");
        }
    }
    for (m0, tt4) in [(1.6, 400.0), (1.6, 410.0), (2.0, 480.0), (2.0, 490.0)] {
        let (sv, o) = controls(&ControlsSettings { m0, from: tt4, ..d }).unwrap();
        println!("M{m0} {tt4}: start f={:.5}, {} points, {:?}", sv.start.close.f, o.points.len(),
            o.stop.as_ref().and_then(|s| s.failure.as_ref()).map(|f| f.message.clone()));
    }
}
