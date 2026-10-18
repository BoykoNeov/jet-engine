//! Flat-LP over Mach 1.6–2.6 from 800–1600 K starts (to = from + 200, capped 1700): does any throttle bring it back on the map?
use turbojet::sandbox_controls::{controls, ControlsSettings, ControlsSolver};
fn main() {
    let d = ControlsSettings::defaults();
    for m0 in [1.5, 1.6, 1.8, 2.0, 2.2, 2.6] {
        let mut line = format!("M {m0:>3}:");
        for k in 0..9 {
            let from = 800.0 + 100.0 * k as f64;
            let s = ControlsSettings { shape: 3, m0, from, to: (from + 200.0f64).min(1700.0), ..d };
            line += &match controls(&s) { Ok((sv, _)) => format!(" {from}:OK(F {:.0})", ControlsSolver::thrust(&sv.start)), Err(_) => format!(" {from}:no") };
        }
        println!("{line}");
    }
}
