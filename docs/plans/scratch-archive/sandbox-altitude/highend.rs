//! Advisor check: (a) the HIGHEST workable throttle (scan down from 1700 K) in cold air high up, per map;
//! (b) every "off the maps" and "does not settle" refusal of the seed-14 crash map: throttles, T0, lever.
use turbojet::atmosphere::standard;
use turbojet::sandbox::call;
use turbojet::sandbox_controls::*;
fn main() {
    let d = ControlsSettings::defaults();
    for (z, dt) in [(0.0, -40.0), (11000.0, -40.0), (11000.0, 0.0), (20000.0, -40.0)] {
        let (t, p) = standard(z);
        print!("{z:>6} m {dt:+}: ");
        for shape in 0..4 {
            let mut top = None;
            let mut why = String::new();
            for k in 0..=60 {
                let tt = 1700.0 - 10.0 * k as f64;
                match build(&ControlsSettings { t0: t + dt, p0: p, shape, from: 1000.0, to: tt, ..d }) {
                    Ok(_) => { top = Some(tt); break; }
                    Err(e) => if why.is_empty() { why = e.0.chars().take(90).collect(); },
                }
            }
            print!(" {} top {:?}{}", CONTROL_SHAPES[shape], top, if why.is_empty() { String::new() } else { format!(" ({why})") });
        }
        println!();
    }
}
