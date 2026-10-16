//! Rung 95's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_governor_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung95.rs`. The Controls engine's default slam, open loop against each governor, with and without the
//! temperature limiter — the scored grid is the spec's.

use super::Design;
use crate::burner_hold::Hold;
use crate::governor as gv;
use crate::pyfmt::Printer;
use crate::sandbox_controls::{controls, ControlsSettings, SpeedLaw};

/// `print_governor_table` — rung 95.
pub fn governor_table(p: &mut Printer, _d: &Design) {
    p.print("\nA GOVERNOR IN THE SLAM (rung 95): the throttle holds its quantity ALL ALONG the move - a straight line from");
    p.print("the start's value to the end's - and the governor's fuel is what the limiters cut. The Controls engine (pi 3 x 6,");
    p.print("LP curved along flow, HP along pressure), 1000 -> 1400 K over 0.5 tau, the perfect gas.");
    p.print(format!("\n  {:<26}{:>9}{:>9}{:>9}{:>10}{:>9}", "fuel from", "peak Tt4", "phi_LP", "phi_HP", "95% thr", "edge"));
    let d = ControlsSettings::defaults();
    let rows: [(&str, ControlsSettings); 5] = [
        ("the open-loop ramp", ControlsSettings { hold: Hold::Thrust, ..d }),
        ("governed thrust", ControlsSettings { hold: Hold::Thrust, govern: true, ..d }),
        ("governed HP speed, rate", ControlsSettings { hold: Hold::HpSpeed, govern: true, ..d }),
        ("governed HP speed, prop.", ControlsSettings { hold: Hold::HpSpeed, govern: true, speed_law: SpeedLaw::Prop, ..d }),
        ("governed burner temp.", ControlsSettings { govern: true, ..d }),
    ];
    for redline in [false, true] {
        if redline {
            p.print("  with the temperature limiter at 1480 K:");
        }
        for (name, s) in rows {
            let s = ControlsSettings { redline_on: redline, ..s };
            match controls(&s) {
                Ok((sv, o)) => {
                    let pts = &o.points;
                    let peak = pts.iter().map(|q| q.tt4).fold(f64::NEG_INFINITY, f64::max);
                    let lo = |f: fn(&crate::fuel_transient::FuelPoint) -> f64| pts.iter().map(f).fold(f64::INFINITY, f64::min);
                    let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
                    let th: Vec<f64> = pts.iter().map(|q| gv::thrust(&sv.reread(q).expect("a recorded point").base)).collect();
                    let x = f0 + 0.95 * (f1 - f0);
                    let t95 = (1..th.len()).find(|&k| th[k - 1] < x && th[k] >= x)
                        .map(|k| pts[k - 1].s + (x - th[k - 1]) / (th[k] - th[k - 1]) * (pts[k].s - pts[k - 1].s));
                    let edge = o.marks.iter().filter(|m| m.clamp != gv::Clamp::Root).count();
                    p.print(format!("  {name:<26}{peak:>9.1}{:>9.4}{:>9.4}{:>10}{:>9}", lo(|q| q.phi_lp), lo(|q| q.phi_hp),
                                    t95.map_or("-".to_string(), |t| format!("{t:.3}")),
                                    if o.marks.is_empty() { "-".to_string() } else { edge.to_string() }));
                }
                Err(e) => p.print(format!("  {name:<26}{}", e.0)),
            }
        }
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  (95% thr: time to 95% of the thrust step, tau, - if not within the run; edge: points the governor sat at an",
    "  edge of what the engine can burn.) Holding thrust along a fast move burns more fuel than the ramp at every point,",
    "  so the turbine runs hotter and both compressors come nearer stall; its hottest moment is the end of the move. With",
    "  the limiter on, the governor still reaches the thrust sooner. Governing the burner temperature IS the",
    "  temperature-commanded slam: no overshoot, and the slowest thrust.",
];
