// Part C cost (addendum § 5.4): the page's reply (`call_op("controls")`), best of 9, lit vs the same request fitted-unlit.
use std::time::Instant;
use turbojet::afterburner::AbMode;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{call_op, ControlsSettings};
use turbojet::visuals::Json;

fn best(s: &ControlsSettings) -> f64 {
    let req = Json::parse(&format!("{{\"op\":\"controls\",\"controls\":{}}}", s.to_json().dump_compact()));
    (0..9).map(|_| { let t = Instant::now(); let _ = call_op("controls", &req); t.elapsed().as_secs_f64() * 1e3 }).fold(f64::INFINITY, f64::min)
}

fn main() {
    let d = ControlsSettings::defaults();
    let cases = [
        ("default", d),
        ("redline", ControlsSettings { redline_on: true, ..d }),
        ("accel", ControlsSettings { accel_on: true, ..d }),
        ("floor", ControlsSettings { floor_on: true, ..d }),
        ("release", ControlsSettings { floor_on: true, release_on: true, ..d }),
        ("gov HP", ControlsSettings { govern: true, hold: Hold::HpSpeed, ..d }),
        ("settle 6", ControlsSettings { settle: 6.0, ..d }),
    ];
    for (lab, s) in cases {
        let unlit = best(&ControlsSettings { ab: Some(AbMode::Unlit), ..s });
        let none = best(&s);
        print!("{lab:9} none {none:7.1} ms  unlit {unlit:7.1} ms");
        for (tl, th) in [("held", ControlsSettings { ab: Some(AbMode::Lit), ..s }), ("x1.15", ControlsSettings { ab: Some(AbMode::Lit), a8_ratio: 1.15, ..s }),
                         ("match", ControlsSettings { ab: Some(AbMode::Lit), a8_match: true, ..s })] {
            let t = best(&th);
            print!("  {tl} {t:7.1} ({:.2}x)", t / unlit);
        }
        println!();
    }
}
