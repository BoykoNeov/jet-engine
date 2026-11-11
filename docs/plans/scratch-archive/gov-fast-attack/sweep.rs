//! For fast attacks beside the governor: the step check at finer steps. Steps DS/k for k in {1,2,4,8,16,32}; the gap
//! of each k against 4k (the project's "step four times finer" check), the largest flow-coefficient gap on the DS grid.
//! Every law at rung 106's per-law edge (as rung 107's edge.rs), three cases x three flights; attack x release grid.
//! Also: bit-equality of the k = 1 run with the view's own march at the shipped settings, and the time of each run.
//! Usage: sweep  (prints one line per setting x lag pair)
use gov_fast_attack::*;
use std::sync::Mutex;
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, DS};

fn settings() -> Vec<(String, ControlsSettings)> {
    let mut v = Vec::new();
    for case in ["sched", "floor", "both"] {
        for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
            let base0 = ControlsSettings { t0, p0, m0, govern: true, release_on: true, tau_att: 0.2, tau_rel: 0.4,
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
    let atts = [0.005, 0.01, 0.02, 0.05, 0.1];
    let rels = [0.1, 0.4, 0.8];
    let ks = [1usize, 2, 4, 8, 16, 32];
    let jobs = Mutex::new(settings());
    let out = Mutex::new(Vec::<String>::new());
    let threads: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(8);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let Some((name, s)) = jobs.lock().unwrap().pop() else { break };
                let mut sv = match build(&s) {
                    Ok(v) => v,
                    Err(e) => {
                        let line = format!("{name}: REFUSED {}", e.0);
                        println!("{line}");
                        out.lock().unwrap().push(line);
                        continue;
                    }
                };
                // Bit-equality of the probe's march at DS with the view's own, at the shipped (admitted) lag.
                let same = format!("{:?}", march(&sv, DS)) == format!("{:?}", sv.march());
                for &ta in &atts {
                    for &tr in &rels {
                        sv.settings.tau_att = ta;
                        sv.settings.tau_rel = tr;
                        let mut runs = Vec::new();
                        let mut secs = Vec::new();
                        for &k in &ks {
                            let t = Instant::now();
                            runs.push(march(&sv, DS / k as f64));
                            secs.push(t.elapsed().as_secs_f64());
                        }
                        let full = (sv.settings.s_end() / DS).round() as usize + 1;
                        let lens: Vec<String> = runs.iter().zip(&ks).map(|(r, k)| format!("{}", (r.len() - 1) / k + 1)).collect();
                        let gaps: Vec<String> = (0..4).map(|i| format!("{:.1e}", gap(&runs[i], ks[i], &runs[i + 2], ks[i + 2]))).collect();
                        let line = format!("{name} same={same} att={ta} rel={tr} full={full} lens=[{}] gap(k v 4k, k=1,2,4,8)=[{}] secs=[{}]",
                                           lens.join(","), gaps.join(","),
                                           secs.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>().join(","));
                        println!("{line}");
                        out.lock().unwrap().push(line);
                    }
                }
            });
        }
    });
    let mut v = out.into_inner().unwrap();
    v.sort();
    std::fs::write("W:/temp/claude/gov-fast-attack/sweep.txt", v.join("\n") + "\n").unwrap();
}
