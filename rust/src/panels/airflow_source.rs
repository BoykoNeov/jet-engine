//! Rung 101's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_airflow_source_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung101.rs`. The thrust-inferring control on design-flight tables with rung 100's formula, its airflow from
//! three sources: the starting end's steady miss, and the worst gap between the thrust believed and the true thrust
//! along the move.

use super::Design;
use crate::airflow_source::AirflowFrom;
use crate::burner_hold::Hold;
use crate::mach_axis::SpeedCorr;
use crate::pyfmt::Printer;
use crate::sandbox_controls::{build, ControlsSettings, ControlsSolver, ThrustBy};

/// The starting end's (achieved − asked)/asked, %, or "refused".
fn start(s: &ControlsSettings) -> String {
    match build(s) {
        Ok(sv) => {
            let a = sv.hold_start.expect("a carried inferred run calibrates its ends");
            format!("{:+.2}", 100.0 * (a.achieved - a.target) / a.target)
        }
        Err(_) => "refused".to_string(),
    }
}

/// The move's worst (F − F̂)/F, %, over the points with F at least 20 N per kg/s — or "refused".
fn moved(s: &ControlsSettings) -> String {
    let Ok(sv) = build(s) else { return "refused".to_string() };
    let (pts, _) = sv.march_marked();
    let mut w = 0.0f64;
    for p in &pts {
        let (Ok(i), Ok(r)) = (sv.reread(p), sv.read(p)) else { return "refused".to_string() };
        let f = ControlsSolver::thrust(&i.base);
        let m = (f - r.inferred.expect("an inferred run believes a thrust")) / f;
        if f >= 20.0 && m.abs() > w.abs() {
            w = m;
        }
    }
    format!("{:+.2}", 100.0 * w)
}

/// `print_airflow_source_table` — rung 101.
pub fn airflow_source_table(p: &mut Printer, _d: &Design) {
    p.print("\nTHE AIRFLOW OFF THE LINE (rung 101): rung 100's formula with the air swallowed taken from its carried table,");
    p.print("the speed grid's tables, or an exhaust-temperature sensor on the nozzle's own flow (the fuel metered and the air");
    p.print("dumped taken off). The 700 K starting end's thrust over what the throttle asks, %, on the map curved along flow at");
    p.print("LP, along pressure at HP, flown in 250 K, 50 kPa air:");
    p.print(format!("\n  {:<8}{:<16}{:>13}{:>13}{:>13}", "Mach", "read", "table", "speed tables", "sensor"));
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, from: 700.0, t0: 250.0, p0: 50_000.0,
                               speed_corr: SpeedCorr::Formula, ..ControlsSettings::defaults() };
    let with = |s: ControlsSettings, a: AirflowFrom| ControlsSettings { airflow: a, ..s };
    for m0 in [0.1, 0.5, 1.4] {
        for (by, name) in [(ThrustBy::Epr, "pressure ratio"), (ThrustBy::N1, "fan speed")] {
            let s = ControlsSettings { m0, thrust_by: by, ..d };
            p.print(format!("  {m0:<8}{name:<16}{:>13}{:>13}{:>13}", start(&with(s, AirflowFrom::Table)),
                            start(&with(s, AirflowFrom::Grid)), start(&with(s, AirflowFrom::Exhaust))));
        }
    }
    p.print("\n  Along the move, the worst (true - believed)/true thrust, %, where the thrust is at least 20 N per kg/s, the");
    p.print("  sensor instant and with a lag of one HP-shaft time constant:");
    p.print(format!("\n  {:<8}{:<16}{:>13}{:>13}{:>13}{:>13}", "Mach", "read", "table", "speed tables", "sensor", "sensor, lag"));
    for m0 in [0.1, 1.4] {
        for (by, name) in [(ThrustBy::Epr, "pressure ratio"), (ThrustBy::N1, "fan speed")] {
            let s = ControlsSettings { m0, thrust_by: by, ..d };
            let lag = ControlsSettings { exhaust_lag: true, ..with(s, AirflowFrom::Exhaust) };
            p.print(format!("  {m0:<8}{name:<16}{:>13}{:>13}{:>13}{:>13}", moved(&with(s, AirflowFrom::Table)),
                            moved(&with(s, AirflowFrom::Grid)), moved(&with(s, AirflowFrom::Exhaust)), moved(&lag)));
        }
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  The speed tables take the pressure ratio's steady miss to almost nothing, but not the move's: an accelerating",
    "  engine passes less air than any steady table says. The sensor reads the air itself, so on the pressure ratio the",
    "  belief is the true thrust at every instant - until the probe is slow, and then the move's error comes back, larger.",
    "  On the fan speed the formula's push is itself a guess: the speed tables remove an airflow error that had cancelled",
    "  part of it, so the start gets worse; the sensor, fed the same guessed pressure, cancels it instead.",
];
