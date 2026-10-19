// Rung 101 post-build, reported not scored (anchor § 3) and the § 4 cost trigger.
// (1) cost: each source against `table`, governed EPR and fan speed (rate law) at Mach 0.25 and 1.2 from 900 K, and the
//     lagged sensor, best of 9 (build + march + read every point).
// (2) the lagged sensor on the fan speed at τ 0.1 / 0.5 / 2; (3) the generator (calibrated) cells; (4) Mach 0.5;
// (5) Mach 2.0 from 1000 K; (6) the fan proportional law under the sensor. Start miss and the move's worst (F ≥ 20).
use std::time::Instant;
use turbojet::airflow_source::AirflowFrom;
use turbojet::burner_hold::Hold;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, GeneratorBy, GeneratorMode, SpeedLaw, ThrustBy};

fn run(s: &ControlsSettings) -> Result<(f64, f64), String> {
    let sv = build(s).map_err(|e| e.0)?;
    let (pts, _) = sv.march_marked();
    if pts.len() < s.expected_points() { return Err(format!("stopped at {} of {}", pts.len(), s.expected_points())) }
    let h = sv.hold_start.ok_or("no calibrated start")?;
    let mut mv = 0.0f64;
    for p in &pts {
        let r = sv.read(p).map_err(|e| e.0)?;
        let m = (r.thrust - r.inferred.unwrap()) / r.thrust;
        if r.thrust >= 20.0 && m.abs() > mv.abs() { mv = m }
    }
    Ok(((h.achieved - h.target) / h.target, mv))
}
fn best(s: &ControlsSettings) -> f64 {
    (0..9).map(|_| { let t = Instant::now(); let _ = run(s); t.elapsed().as_secs_f64() }).fold(f64::INFINITY, f64::min)
}
fn show(tag: &str, s: &ControlsSettings) {
    match run(s) {
        Ok((a, b)) => println!("{tag:58} start {:+.3}%  move {:+.3}%", 100.0 * a, 100.0 * b),
        Err(e) => println!("{tag:58} REFUSED {}", e.chars().take(160).collect::<String>()),
    }
}

fn main() {
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, speed_corr: SpeedCorr::Formula, t0: 250.0, p0: 50_000.0,
                               ..ControlsSettings::defaults() };
    let src = |s: ControlsSettings, a: AirflowFrom| ControlsSettings { airflow: a, ..s };
    println!("== (1) cost, best of 9 (s), from 900 K");
    for m0 in [0.25, 1.2] {
        for by in [ThrustBy::Epr, ThrustBy::N1] {
            let s = ControlsSettings { m0, from: 900.0, thrust_by: by, ..d };
            let t = best(&src(s, AirflowFrom::Table));
            let g = best(&src(s, AirflowFrom::Grid));
            let x = best(&src(s, AirflowFrom::Exhaust));
            let l = best(&ControlsSettings { exhaust_lag: true, ..src(s, AirflowFrom::Exhaust) });
            println!("M{m0} {by:?}: table {t:.3}  grid {g:.3} ({:.2}x)  sensor {x:.3} ({:.2}x)  sensor lagged {l:.3} ({:.2}x)",
                     g / t, x / t, l / t);
        }
    }
    println!("== (2) the lagged sensor on the fan speed, Mach 0.7 / 1.2 from 750 K");
    for m0 in [0.7, 1.2] {
        let s = ControlsSettings { m0, from: 750.0, thrust_by: ThrustBy::N1, ..d };
        show(&format!("M{m0} fan, table"), &src(s, AirflowFrom::Table));
        show(&format!("M{m0} fan, sensor"), &src(s, AirflowFrom::Exhaust));
        for tau in [0.1, 0.5, 2.0] {
            show(&format!("M{m0} fan, sensor τ {tau}"), &ControlsSettings { exhaust_lag: true, exhaust_tau: tau, ..src(s, AirflowFrom::Exhaust) });
        }
    }
    println!("== (3) the generator (calibrated ends), Mach 1.2 from 900 K");
    for (gn, gm, gb) in [("HP share 0.05", GeneratorMode::Hp, GeneratorBy::Share), ("LP share 0.05", GeneratorMode::Lp, GeneratorBy::Share),
                         ("HP power 0.02", GeneratorMode::Hp, GeneratorBy::Power)] {
        for by in [ThrustBy::Epr, ThrustBy::N1] {
            for a in AirflowFrom::ALL {
                let s = ControlsSettings { m0: 1.2, from: 900.0, thrust_by: by, generator: gm, generator_by: gb, ..d };
                show(&format!("gen {gn} {by:?} {}", a.key()), &src(s, a));
            }
        }
    }
    println!("== (4) Mach 0.5 from 700 K (the 0.3-0.6 gap)");
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        for a in AirflowFrom::ALL { show(&format!("M0.5 {by:?} {}", a.key()), &src(ControlsSettings { m0: 0.5, from: 700.0, thrust_by: by, ..d }, a)); }
    }
    println!("== (5) Mach 2.0 from 1000 K");
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        for a in AirflowFrom::ALL { show(&format!("M2.0 {by:?} {}", a.key()), &src(ControlsSettings { m0: 2.0, from: 1000.0, thrust_by: by, ..d }, a)); }
        show(&format!("M2.0 {by:?} exhaust τ1"), &ControlsSettings { exhaust_lag: true, ..src(ControlsSettings { m0: 2.0, from: 1000.0, thrust_by: by, ..d }, AirflowFrom::Exhaust) });
    }
    println!("== (6) the fan proportional law under the sensor");
    for m0 in [0.25, 1.2] {
        let s = ControlsSettings { m0, from: 900.0, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, ..d };
        show(&format!("M{m0} fan prop, table"), &src(s, AirflowFrom::Table));
        show(&format!("M{m0} fan prop, sensor"), &src(s, AirflowFrom::Exhaust));
    }
}
