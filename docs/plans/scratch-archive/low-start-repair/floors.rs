// Throwaway: the lowest starting throttle the Controls page accepts, per map and flight speed,
// and the refusal just below it.
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings::defaults();
    println!("design flight: T0={} p0={} M0={}", d.t0, d.p0, d.m0);
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    for (name, t0, p0, m0) in [("design", d.t0, d.p0, d.m0), ("M1.6", d.t0, d.p0, 1.6), ("M2", d.t0, d.p0, 2.0),
                               ("M2.6", d.t0, d.p0, 2.6), ("SL std M0.85", t_sl, p_sl, 0.85), ("SL+40 M0.85", t_sl + 40.0, p_sl, 0.85)] {
        for (k, shape) in CONTROL_SHAPES.iter().enumerate() {
            let mut tt4 = 400.0;
            let mut last_err = String::new();
            let mut first = None;
            while tt4 <= 1000.0 + 1e-9 {
                match build(&ControlsSettings { shape: k, from: tt4, t0, p0, m0, ..d }) {
                    Ok(_) => { first = Some(tt4); break; }
                    Err(e) => last_err = e.0,
                }
                tt4 += 10.0;
            }
            let i = last_err.find("(The model says:").unwrap_or(0);
            println!("{name} {shape}: lowest start {first:?}; below it: {}", &last_err[i..]);
        }
    }
}
