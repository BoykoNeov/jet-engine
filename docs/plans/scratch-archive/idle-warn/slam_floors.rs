// Probe: the Controls page's lowest start whose WHOLE default slam (to 1400 K) finishes, per variant, map and flight —
// beside the lowest start that only BUILDS, and how the slams between them stop. The live page's rule (warn).
use turbojet::sandbox_controls::{controls, ControlsSettings, CustomerMode, LeverChoice, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    for variant in ["plain", "bleed lever", "stator lever", "customer", "cooling"] {
        for (name, t0, p0, m0) in [("design", d.t0, d.p0, d.m0), ("M1.6", d.t0, d.p0, 1.6), ("M2", d.t0, d.p0, 2.0),
                                   ("M2.6", d.t0, d.p0, 2.6), ("SL std", t_sl, p_sl, 0.85), ("SL+40", t_sl + 40.0, p_sl, 0.85)] {
            for (k, shape) in CONTROL_SHAPES.iter().enumerate() {
                let mut s = ControlsSettings { shape: k, t0, p0, m0, ..d };
                match variant {
                    "bleed lever" => s.lever = LeverChoice::Bleed,
                    "stator lever" => s.lever = LeverChoice::Stator,
                    "customer" => s.customer = CustomerMode::Share,
                    "cooling" => { s.cool_vane = 0.03; s.cool_blade = 0.02; }
                    _ => {}
                }
                let (mut built, mut full, mut idle_up, mut stops) = (None, None, None, Vec::new());
                let mut tt4 = 400.0;
                while tt4 <= 1300.0 + 1e-9 {
                    if let Ok((sv, o)) = controls(&ControlsSettings { from: tt4, ..s }) {
                        built.get_or_insert(tt4);
                        let sp = sv.start.sp_thrust_inlet.unwrap_or(sv.start.sp_thrust);
                        if sp > 0.0 { idle_up.get_or_insert(tt4); }
                        if o.points.len() == s.expected_points() { full = Some(tt4); break; }
                        let m = o.stop.as_ref().and_then(|st| st.failure.as_ref()).map(|f| f.message.chars().take(40).collect::<String>()).unwrap_or_default();
                        stops.push(format!("{tt4:.0}:{}pts:{m}", o.points.len()));
                    }
                    tt4 += 10.0;
                }
                println!("{variant} | {name} | {shape}: builds from {built:?}, above idle from {idle_up:?}, whole slam from {full:?}; stops below: {}",
                         stops.join(" ; "));
            }
        }
    }
}
