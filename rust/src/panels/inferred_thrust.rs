//! Rung 96's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_inferred_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung96.rs`. The Controls engine's default governed slam, thrust read directly against each reading, and the
//! steady ends calibrated on the generator-off engine — the scored grid is the spec's.

use super::Design;
use crate::burner_hold::Hold;
use crate::pyfmt::Printer;
use crate::sandbox_controls::{controls, ControlsSettings, GeneratorMode, SpeedLaw, ThrustBy};

/// `print_inferred_table` — rung 96.
pub fn inferred_table(p: &mut Printer, _d: &Design) {
    p.print("\nA GOVERNOR THAT INFERS THRUST (rung 96): rung 95's governed thrust READ from the engine pressure ratio or the");
    p.print("fan (LP) speed, turned into thrust by a 13-row steady table. The Controls engine (pi 3 x 6, LP curved along flow,");
    p.print("HP along pressure), 1000 -> 1400 K over 0.5 tau; gaps are fractions of the thrust change.");
    p.print(format!("\n  {:<26}{:>9}{:>9}{:>9}{:>10}{:>10}{:>10}", "thrust read from", "peak Tt4", "phi_LP", "phi_HP",
                    "most over", "decel", "95% thr"));
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, ..ControlsSettings::defaults() };
    let rows: [(&str, ControlsSettings); 4] = [
        ("the thrust itself", d),
        ("the pressure ratio", ControlsSettings { thrust_by: ThrustBy::Epr, ..d }),
        ("the fan speed, rate 0.1", ControlsSettings { thrust_by: ThrustBy::N1, ..d }),
        ("the fan speed, prop. 20", ControlsSettings { thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, ..d }),
    ];
    for (name, s) in rows {
        // The most the engine gives over what the control believes over the ramp, accelerating and slowing.
        let gap = |s: &ControlsSettings, most: bool| -> Option<f64> {
            let (sv, o) = controls(s).ok()?;
            let step = (crate::governor::thrust(&sv.end) - crate::governor::thrust(&sv.start)).abs();
            let g = o.points.iter().filter(|x| x.s > 0.0 && x.s <= s.ramp + 1e-9).map(|x| {
                let r = sv.read(x).expect("a recorded point");
                (r.thrust - r.inferred.unwrap_or(r.thrust)) / step
            });
            Some(if most { g.fold(f64::NEG_INFINITY, f64::max) } else { g.fold(f64::INFINITY, f64::min) })
        };
        match controls(&s) {
            Ok((sv, o)) => {
                let pts = &o.points;
                let peak = pts.iter().map(|q| q.tt4).fold(f64::NEG_INFINITY, f64::max);
                let lo = |f: fn(&crate::fuel_transient::FuelPoint) -> f64| pts.iter().map(f).fold(f64::INFINITY, f64::min);
                let (f0, f1) = (crate::governor::thrust(&sv.start), crate::governor::thrust(&sv.end));
                let th: Vec<f64> = pts.iter().map(|q| sv.read(q).expect("a recorded point").thrust).collect();
                let x = f0 + 0.95 * (f1 - f0);
                let t95 = (1..th.len()).find(|&k| th[k - 1] < x && th[k] >= x)
                    .map(|k| pts[k - 1].s + (x - th[k - 1]) / (th[k] - th[k - 1]) * (pts[k].s - pts[k - 1].s));
                let decel = ControlsSettings { from: 1400.0, to: 1000.0, ..s };
                let fmt = |v: Option<f64>| v.map_or("-".to_string(), |v| format!("{v:+.4}"));
                p.print(format!("  {name:<26}{peak:>9.1}{:>9.4}{:>9.4}{:>10}{:>10}{:>10}", lo(|q| q.phi_lp), lo(|q| q.phi_hp),
                                fmt(gap(&s, true)), fmt(gap(&decel, false)),
                                t95.map_or("-".to_string(), |t| format!("{t:.3}"))));
            }
            Err(e) => p.print(format!("  {name:<26}{}", e.0)),
        }
    }
    p.print("\n  Calibrated on the engine with the generator off (power 0.02): each end's thrust over what was asked.");
    for (by, bname) in [(ThrustBy::Epr, "pressure ratio"), (ThrustBy::N1, "fan speed")] {
        for (g, gname) in [(GeneratorMode::Hp, "HP"), (GeneratorMode::Lp, "LP")] {
            let s = ControlsSettings { thrust_by: by, generator: g, ..d };
            match controls(&s) {
                Ok((sv, _)) => {
                    let (a, b) = (sv.hold_start.expect("a calibrated end"), sv.hold_end.expect("a calibrated end"));
                    p.print(format!("  {bname:<15} generator on {gname}: {:+.4} at 1000 K, {:+.4} at 1400 K",
                                    a.achieved / a.target - 1.0, b.achieved / b.target - 1.0));
                }
                Err(e) => p.print(format!("  {bname:<15} generator on {gname}: {}", e.0)),
            }
        }
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  (most over: the most thrust the engine gives over what the control believes on the acceleration; decel: the",
    "  same on the slowdown, negative; 95% thr: time to 95% of the thrust step, tau.) The pressure ratio reads the jet's",
    "  gross push exactly - both nozzles choked - but not the drag of the air swallowed: an engine that must also find",
    "  power to spare, to speed up or to drive a generator, burns hotter and swallows less air at a given pressure",
    "  ratio, so it gives MORE thrust than its control believes. The fan speed misses by far more.",
];
