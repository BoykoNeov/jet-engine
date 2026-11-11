//! One thread: each governed setting at rung 106 (k=1, lag 0.2) and the two-speed clip (0.02, 0.2) at k=4; the open
//! ramp at k=1 both ways.
use rung108_probe::*;
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, DS};
fn main() {
    for case in ["red", "sched", "floor"] {
        for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
            let base0 = ControlsSettings { t0, p0, m0, redline_on: true, gov_lag_on: true, gov_lag: 0.2, ..ControlsSettings::defaults() };
            let base = match case { "red" => base0, "sched" => ControlsSettings { accel_on: true, ..base0 }, _ => ControlsSettings { floor_on: true, shape: 1, ..base0 } };
            let gb = ControlsSettings { govern: true, ..base };
            let rho = gb.density_ratio();
            for (l, s) in [
                ("open", base),
                ("thrust", ControlsSettings { hold: Hold::Thrust, ..gb }),
                ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..gb }),
                ("HPrate.1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1 / rho.min(1.0), ..gb }),
                ("HPprop10", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0 / rho.max(1.0), ..gb }),
                ("N1rate.15", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.15 / rho.min(1.0), ..gb }),
                ("N1prop7", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 7.0 / rho.max(1.0), ..gb }),
            ] {
                let Ok(sv) = build(&s) else { println!("{case} {fname} {l} REFUSED"); continue };
                let t = Instant::now(); let a = march(&sv, 0.2, 0.2, DS); let t106 = t.elapsed().as_secs_f64();
                let k = if l == "open" { 1.0 } else { 4.0 };
                let t = Instant::now(); let b = march(&sv, 0.02, 0.2, DS / k); let tnew = t.elapsed().as_secs_f64();
                println!("{case} {fname} {l} rung106={t106:.2}s n={} new={tnew:.2}s n={} ratio={:.1}", a.len(), b.len(), tnew / t106);
            }
        }
    }
}
