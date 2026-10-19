// Rung 101 post-build smoke: each airflow source on the SHIPPED code, closed loop. Start miss, worst move miss
// (F ≥ 20), the sensor gate (|F − F̂| at every point), the thrust_by:true comparison, timing.
use std::time::Instant;
use turbojet::airflow_source::AirflowFrom;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, ThrustBy};

fn run(s: &ControlsSettings) -> Result<(ControlsSolver, Vec<(f64, f64)>, f64), String> {
    let t = Instant::now();
    let sv = build(s).map_err(|e| e.0)?;
    let (pts, _) = sv.march_marked();
    let fs: Vec<(f64, f64)> = pts.iter().map(|p| {
        let r = sv.read(p).expect("read");
        (ControlsSolver::thrust(&sv.reread(p).unwrap().base), r.inferred.unwrap_or(f64::NAN))
    }).collect();
    Ok((sv, fs, t.elapsed().as_secs_f64()))
}

fn main() {
    let d = ControlsSettings::defaults();
    for (m0, from) in [(1.4, 700.0), (0.1, 700.0), (0.5, 1000.0)] {
        for by in [ThrustBy::Epr, ThrustBy::N1] {
            for (name, air, lag) in [("table", AirflowFrom::Table, false), ("grid", AirflowFrom::Grid, false),
                                     ("exh", AirflowFrom::Exhaust, false), ("exh-lag1", AirflowFrom::Exhaust, true)] {
                let s = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, speed_corr: SpeedCorr::Formula,
                                           m0, from, t0: 250.0, p0: 50_000.0, airflow: air, exhaust_lag: lag, exhaust_tau: 1.0, ..d };
                match run(&s) {
                    Err(e) => println!("M{m0} {from} {:?} {name}: REFUSED {}", by, e.chars().take(120).collect::<String>()),
                    Ok((_sv, fs, dt)) => {
                        let (f0, h0) = fs[0];
                        let mv = fs.iter().filter(|q| q.0 >= 20.0).map(|q| (q.0 - q.1) / q.0).fold(0.0f64, |a, x| if x.abs() > a.abs() { x } else { a });
                        let gap = fs.iter().map(|q| (q.0 - q.1).abs()).fold(0.0f64, f64::max);
                        let tag = format!("M{m0} {from} {:?} {name:8}", by);
                        let mut extra = String::new();
                        if air == AirflowFrom::Exhaust && !lag && by == ThrustBy::Epr {
                            let st = ControlsSettings { thrust_by: ThrustBy::True, ..s };
                            if let Ok((_, ft, _)) = run(&st) {
                                let w = fs.iter().zip(&ft).map(|(a, b)| ((a.0 - b.0) / b.0).abs()).fold(0.0f64, f64::max);
                                extra = format!(" | vs thrust_by:true worst {w:.2e} (n {} / {})", fs.len(), ft.len());
                            }
                        }
                        println!("{tag} pts {:3} start {:+.4}% move {:+.3}% max|F−F̂| {gap:.2e} t {dt:.2}s{extra}",
                                 fs.len(), 100.0 * (f0 - h0) / f0, 100.0 * mv);
                    }
                }
            }
        }
    }
}
