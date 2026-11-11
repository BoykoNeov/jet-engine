//! How fine a step the two-speed clip needs: 3 cases x 3 flights x (7 governed laws at rung 106's edges + the open
//! ramp) x attack x release; steps DS/k, each k against 4k, the largest flow-coefficient gap on the DS grid. Also the
//! reduce check (equal constants 0.2 = the page's rung 106 / rung 47 march, bit for bit), reach (stops, min-fuel holds,
//! attack/release edge counts) and the time of each run.
use rung108_probe::*;
use std::sync::Mutex;
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, DS};

fn settings() -> Vec<(String, ControlsSettings)> {
    let mut v = Vec::new();
    for case in ["red", "sched", "floor"] {
        for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
            let base0 = ControlsSettings { t0, p0, m0, redline_on: true, gov_lag_on: true, gov_lag: 0.2, ..ControlsSettings::defaults() };
            let base = match case {
                "red" => base0,
                "sched" => ControlsSettings { accel_on: true, ..base0 },
                _ => ControlsSettings { floor_on: true, shape: 1, ..base0 },
            };
            v.push((format!("{case} {fname} open"), base));
            let gb = ControlsSettings { govern: true, ..base };
            let rho = gb.density_ratio();
            let laws: Vec<(&str, ControlsSettings)> = vec![
                ("thrust", ControlsSettings { hold: Hold::Thrust, ..gb }),
                ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..gb }),
                ("Tt4", ControlsSettings { hold: Hold::BurnerTemp, ..gb }),
                ("HPrate.1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1 / rho.min(1.0), ..gb }),
                ("HPprop10", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0 / rho.max(1.0), ..gb }),
                ("N1rate.15", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.15 / rho.min(1.0), ..gb }),
                ("N1prop7", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 7.0 / rho.max(1.0), ..gb }),
            ];
            for (l, s) in laws { v.push((format!("{case} {fname} {l}"), s)); }
        }
    }
    v
}

fn main() {
    let atts = [0.01, 0.02, 0.05, 0.1];
    let rels = [0.1, 0.8];
    let ks = [1usize, 2, 4, 8, 16];
    let jobs = Mutex::new(settings());
    let threads: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(8);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let Some((name, s)) = jobs.lock().unwrap().pop() else { break };
                let sv = match build(&s) {
                    Ok(v) => v,
                    Err(e) => { println!("{name}: REFUSED {}", e.0); continue; }
                };
                let same = same_as_reference(&sv, &march(&sv, 0.2, 0.2, DS));
                let full = (s.s_end() / DS).round() as usize + 1;
                for &ta in &atts {
                    for &tr in &rels {
                        let mut runs = Vec::new();
                        let mut secs = Vec::new();
                        for &k in &ks {
                            let t = Instant::now();
                            runs.push(march(&sv, ta, tr, DS / k as f64));
                            secs.push(t.elapsed().as_secs_f64());
                        }
                        let lens: Vec<String> = runs.iter().zip(&ks).map(|(r, k)| format!("{}", (r.len().max(1) - 1) / k + 1)).collect();
                        let gaps: Vec<String> = (0..3).map(|i| format!("{:.1e}", gap(&runs[i], ks[i], &runs[i + 2], ks[i + 2]))).collect();
                        let r4 = &runs[2];
                        println!("{name} same={same} att={ta} rel={tr} full={full} lens=[{}] gap(k v 4k,k=1,2,4)=[{}] att_edges={} rel_edges={} minfuel={} secs=[{}]",
                                 lens.join(","), gaps.join(","), attacks(r4), releases(r4), minfuel(r4),
                                 secs.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>().join(","));
                    }
                }
            });
        }
    });
}
