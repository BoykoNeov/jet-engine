// The Controls page's lowest starting throttle, per variant, map and flight, with the below-idle floor on.
use turbojet::sandbox_controls::{build, ControlsSettings, CustomerMode, LeverChoice, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    let only = std::env::args().nth(1).unwrap_or_default();
    for variant in ["plain", "bleed lever", "stator lever", "customer", "cooling"] {
        if !only.is_empty() && !only.contains(variant) { continue; }
        for (name, t0, p0, m0) in [("design", d.t0, d.p0, d.m0), ("M1.6", d.t0, d.p0, 1.6), ("M2", d.t0, d.p0, 2.0),
                                   ("M2.6", d.t0, d.p0, 2.6), ("SL std M0.85", t_sl, p_sl, 0.85), ("SL+40 M0.85", t_sl + 40.0, p_sl, 0.85)] {
            for (k, shape) in CONTROL_SHAPES.iter().enumerate() {
                let mut s = ControlsSettings { shape: k, t0, p0, m0, ..d };
                match variant {
                    "bleed lever" => s.lever = LeverChoice::Bleed,
                    "stator lever" => s.lever = LeverChoice::Stator,
                    "customer" => s.customer = CustomerMode::Share,
                    "cooling" => { s.cool_vane = 0.03; s.cool_blade = 0.02; }
                    _ => {}
                }
                let mut tt4 = 400.0;
                let mut last_err = String::new();
                let mut first = None;
                while tt4 <= 1100.0 + 1e-9 {
                    match build(&ControlsSettings { from: tt4, ..s }) {
                        Ok(sv) => { first = Some((tt4, sv.start.sp_thrust_inlet.unwrap_or(sv.start.sp_thrust))); break; }
                        Err(e) => last_err = e.0,
                    }
                    tt4 += 10.0;
                }
                let i = last_err.find("(The model says:").unwrap_or(0);
                let tail: String = last_err[i..].chars().take(140).collect();
                println!("{variant} | {name} | {shape}: lowest start {first:?}; below it: {tail}");
            }
        }
    }
}
