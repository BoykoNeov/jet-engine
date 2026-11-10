//! Step check of the prototype governed release march: step DS against DS/4, the largest gap in either flow
//! coefficient over the points both reach, per law at its rung-95 page edge (and thrust / EPR), over the attack and
//! release times, at three flights. Usage: step <case: sched|floor>
use rung107_probe::*;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::AsymmetricLag;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, DS};

fn main() {
    let case: String = std::env::args().nth(1).unwrap_or("sched".into());
    let atts = [0.02, 0.04, 0.06, 0.1, 0.2];
    let rels = [0.02, 0.1, 0.4, 0.8];
    for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
        let base0 = ControlsSettings { t0, p0, m0, govern: true, ..ControlsSettings::defaults() };
        let base = match case.as_str() {
            "sched" => ControlsSettings { accel_on: true, ..base0 },
            "floor" => ControlsSettings { floor_on: true, shape: 1, ..base0 },
            "both" => ControlsSettings { accel_on: true, floor_on: true, redline_on: true, ..base0 },
            _ => panic!("case"),
        };
        let rho = base.density_ratio();
        let laws: Vec<(&str, ControlsSettings)> = vec![
            ("thrust", ControlsSettings { hold: Hold::Thrust, ..base }),
            ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..base }),
            ("Tt4", ControlsSettings { hold: Hold::BurnerTemp, ..base }),
            ("HP rate edge", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.04 / rho.min(1.0), ..base }),
            ("HP prop edge", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 50.0 / rho.max(1.0), ..base }),
            ("N1 rate edge", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.04 / rho.min(1.0), ..base }),
            ("N1 prop edge", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 50.0 / rho.max(1.0), ..base }),
            ("HP prop 20", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 20.0 / rho.max(1.0), ..base }),
            ("HP rate .1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1 / rho.min(1.0), ..base }),
            ("N1 rate .1", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.1 / rho.min(1.0), ..base }),
        ];
        for (lname, s) in &laws {
            let sv = match build(s) {
                Ok(v) => v,
                Err(e) => {
                    println!("{case} {fname} {lname}: REFUSED {}", e.0);
                    continue;
                }
            };
            let mut row = Vec::new();
            let mut worst = 0.0f64;
            for &ta in &atts {
                for &tr in &rels {
                    let lag = AsymmetricLag::new(ta, tr);
                    let (x, sx) = march(&sv, lag, false, DS);
                    let (y, _) = march(&sv, lag, false, DS / 4.0);
                    let n = x.len().min((y.len() + 3) / 4);
                    let dm = (0..n)
                        .map(|k| (x[k].phi_lp - y[4 * k].phi_lp).abs().max((x[k].phi_hp - y[4 * k].phi_hp).abs()))
                        .fold(0.0, f64::max);
                    worst = worst.max(dm);
                    let mark = if sx.is_some() { "s" } else { "" };
                    if dm >= 1e-4 {
                        row.push(format!("{ta}/{tr}:{dm:.1e}{mark}"));
                    } else if sx.is_some() {
                        row.push(format!("{ta}/{tr}:stop"));
                    }
                }
            }
            println!("{case} {fname} (density {rho:.2}) {lname:13}: worst {worst:.2e}; over 1e-4 or stopped (att/rel): [{}]", row.join(" "));
        }
    }
}
