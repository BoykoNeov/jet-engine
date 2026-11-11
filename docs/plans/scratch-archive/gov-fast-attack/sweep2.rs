//! Plan § 45 bar B2 — the step check ON THE SHIPPED ROUTE: the 63 settings of `sweep.rs` (seven laws at rung 106's
//! edges × schedule / floor / both + redline × design / thin / dense) × attack {0.02, 0.03, 0.05, 0.08, 0.12, 0.16, 0.19}
//! × release {0.1, 0.4, 1.6}: the page's own run (`sv.march()`, the fine step 0.005) against the shipped march at
//! 0.00125, the largest gap in either flow coefficient on the page run's points. Bar: every one < 1e-4.
use gov_fast_attack::march;
use std::sync::Mutex;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, DS};

fn settings() -> Vec<(String, ControlsSettings)> {
    let mut v = Vec::new();
    for case in ["sched", "floor", "both"] {
        for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
            let base0 = ControlsSettings { t0, p0, m0, govern: true, release_on: true, tau_att: 0.02, tau_rel: 0.4,
                                           ..ControlsSettings::defaults() };
            let base = match case {
                "sched" => ControlsSettings { accel_on: true, ..base0 },
                "floor" => ControlsSettings { floor_on: true, shape: 1, ..base0 },
                _ => ControlsSettings { accel_on: true, floor_on: true, redline_on: true, ..base0 },
            };
            let rho = base.density_ratio();
            let laws: Vec<(&str, ControlsSettings)> = vec![
                ("thrust", ControlsSettings { hold: Hold::Thrust, ..base }),
                ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..base }),
                ("Tt4", ControlsSettings { hold: Hold::BurnerTemp, ..base }),
                ("HPrate.1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1 / rho.min(1.0), ..base }),
                ("HPprop10", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0 / rho.max(1.0), ..base }),
                ("N1rate.15", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.15 / rho.min(1.0), ..base }),
                ("N1prop7", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 7.0 / rho.max(1.0), ..base }),
            ];
            for (l, s) in laws {
                v.push((format!("{case} {fname} {l}"), s));
            }
        }
    }
    v
}

fn main() {
    let atts = [0.02, 0.03, 0.05, 0.08, 0.12, 0.16, 0.19];
    let rels = [0.1, 0.4, 1.6];
    let jobs = Mutex::new(settings());
    let out = Mutex::new(Vec::<String>::new());
    let threads: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(8);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let Some((name, s0)) = jobs.lock().unwrap().pop() else { break };
                for &ta in &atts {
                    for &tr in &rels {
                        let s = ControlsSettings { tau_att: ta, tau_rel: tr, ..s0 };
                        let line = match build(&s) {
                            Err(e) => format!("{name} att={ta} rel={tr}: REFUSED {}", e.0),
                            Ok(sv) => {
                                assert_eq!(s.march_ds(), DS / 4.0);
                                let x = sv.march();
                                let y = march(&sv, DS / 16.0);
                                let full = s.expected_points();
                                let n = x.len().min((y.len() + 3) / 4);
                                let g = (0..n).map(|i| (x[i].phi_lp - y[4 * i].phi_lp).abs().max((x[i].phi_hp - y[4 * i].phi_hp).abs()))
                                    .fold(0.0, f64::max);
                                format!("{name} att={ta} rel={tr}: pts {}/{full} fine {}/{} gap {g:.2e}{}", x.len(), y.len(),
                                        4 * (full - 1) + 1, if g >= 1e-4 { " OVER" } else { "" })
                            }
                        };
                        println!("{line}");
                        out.lock().unwrap().push(line);
                    }
                }
            });
        }
    });
    let mut v = out.into_inner().unwrap();
    v.sort();
    std::fs::write("W:/temp/claude/gov-fast-attack/sweep2.txt", v.join("\n") + "\n").unwrap();
}
