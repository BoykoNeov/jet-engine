//! The open ramp at one step: attack x release incl. short releases, 3 cases x 3 flights x 4 maps, k=1 vs k=4.
use rung108_probe::*;
use turbojet::sandbox_controls::{build, ControlsSettings, DS};
fn main() {
    let mut worst = std::collections::BTreeMap::<String, (f64, String)>::new();
    for case in ["red", "sched", "floor"] {
        for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
            for shape in 0..4 {
                let base0 = ControlsSettings { t0, p0, m0, shape, redline_on: true, gov_lag_on: true, gov_lag: 0.2, ..ControlsSettings::defaults() };
                let s = match case { "red" => base0, "sched" => ControlsSettings { accel_on: true, ..base0 }, _ => ControlsSettings { floor_on: true, ..base0 } };
                let Ok(sv) = build(&s) else { println!("{case} {fname} {shape} REFUSED"); continue };
                for ta in [0.02, 0.03, 0.05, 0.1, 0.2] {
                    for tr in [0.02, 0.05, 0.1, 0.2, 0.8] {
                        let a = march(&sv, ta, tr, DS); let b = march(&sv, ta, tr, DS / 4.0);
                        let full = (s.s_end() / DS).round() as usize + 1;
                        let g = gap(&a, 1, &b, 4);
                        let key = format!("att={ta} rel={tr}");
                        let e = worst.entry(key).or_insert((0.0, String::new()));
                        if g > e.0 { *e = (g, format!("{case} {fname} map{shape} len={}/{full} edges={}/{}", a.len(), attacks(&a), releases(&a))); }
                        if a.len() < full || attacks(&a) > 1 || minfuel(&a) > 0 { println!("NOTE {case} {fname} map{shape} {ta} {tr} len={} att={} min={}", a.len(), attacks(&a), minfuel(&a)); }
                    }
                }
            }
        }
    }
    for (k, (g, w)) in worst { println!("{k} worst={g:.1e} ({w})"); }
}
