//! The release knobs' step edges beside the governor: every law at rung 106's measured clean edge (HP rate 0.1, fan
//! rate 0.15, HP gain 10, fan gain 7 — density-scaled), the thrust / EPR / Tt4 holds, over attack × release on a fine
//! grid, at three flights. DS against DS/4, the largest gap in either flow coefficient. Prints, per law and flight,
//! every (attack, release) cell at or over 1e-4. Usage: edge <sched|floor|both>
use rung107_probe::*;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::AsymmetricLag;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, DS};

fn main() {
    let case: String = std::env::args().nth(1).unwrap_or("sched".into());
    let atts = [0.2, 0.4];
    let rels = [1.0, 1.6];
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
            ("HP rate .1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1 / rho.min(1.0), ..base }),
            ("HP prop 10", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0 / rho.max(1.0), ..base }),
            ("N1 rate .15", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.15 / rho.min(1.0), ..base }),
            ("N1 prop 7", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 7.0 / rho.max(1.0), ..base }),
        ];
        for (lname, s) in &laws {
            let sv = match build(s) {
                Ok(v) => v,
                Err(e) => {
                    println!("{case} {fname} {lname}: REFUSED {}", e.0);
                    continue;
                }
            };
            let (mut worst, mut stops) = (0.0f64, 0usize);
            let mut over = Vec::new();
            for &ta in &atts {
                for &tr in &rels {
                    let lag = AsymmetricLag::new(ta, tr);
                    let (x, sx) = march(&sv, lag, false, DS);
                    let (y, _) = march(&sv, lag, false, DS / 4.0);
                    if sx.is_some() {
                        stops += 1;
                    }
                    let n = x.len().min((y.len() + 3) / 4);
                    let dm = (0..n)
                        .map(|k| (x[k].phi_lp - y[4 * k].phi_lp).abs().max((x[k].phi_hp - y[4 * k].phi_hp).abs()))
                        .fold(0.0, f64::max);
                    worst = worst.max(dm);
                    if dm >= 1e-4 {
                        over.push(format!("{ta}/{tr}:{dm:.1e}"));
                    }
                }
            }
            println!("{case} {fname} (density {rho:.2}) {lname:12}: worst {worst:.2e}; stops {stops}; over 1e-4 (att/rel): [{}]",
                     over.join(" "));
        }
    }
}
