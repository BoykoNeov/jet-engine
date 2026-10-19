// Part C work census: lit LP solves, burns, lean walls — per reply.
use turbojet::afterburner::{take_census_c, AbMode};
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{call_op, ControlsSettings};
use turbojet::visuals::Json;
fn main() {
    let d = ControlsSettings::defaults();
    for (lab, s) in [("gov HP unlit", ControlsSettings { ab: Some(AbMode::Unlit), govern: true, hold: Hold::HpSpeed, ..d }), ("default held", ControlsSettings { ab: Some(AbMode::Lit), ..d }),
                     ("default x1.15", ControlsSettings { ab: Some(AbMode::Lit), a8_ratio: 1.15, ..d }),
                     ("gov HP held", ControlsSettings { ab: Some(AbMode::Lit), govern: true, hold: Hold::HpSpeed, ..d }),
                     ("gov HP x1.15", ControlsSettings { ab: Some(AbMode::Lit), a8_ratio: 1.15, govern: true, hold: Hold::HpSpeed, ..d })] {
        let req = Json::parse(&format!("{{\"op\":\"controls\",\"controls\":{}}}", s.to_json().dump_compact()));
        take_census_c(); turbojet::fuel_transient::counters::reset();
        let t = std::time::Instant::now();
        let _ = call_op("controls", &req);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        let (lp, burn, wall) = take_census_c(); let ic = turbojet::fuel_transient::counters::take().instant_calls;
        println!("{lab:14} {ms:6.1} ms  lit LP solves {lp:7}  burns {burn:8} ({:.1}/solve)  lean walls {wall:6}  instants {ic}", burn as f64 / lp.max(1) as f64);
    }
}
