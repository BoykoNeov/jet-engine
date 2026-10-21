//! Rung 106's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_governor_lag_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung106.rs`. The Controls engine's default slam with the temperature limiter at 1480 K and its lag: the
//! governed thrust against rung 47's open ramp at each lag — the scored grid is the spec's.

use super::Design;
use crate::burner_hold::Hold;
use crate::fuel_transient::FuelPoint;
use crate::governor as gv;
use crate::pyfmt::Printer;
use crate::sandbox_controls::{controls, ControlsSettings, ControlsSolver};

/// The thrust step's fraction at every recorded point.
fn fraction(sv: &ControlsSolver, pts: &[FuelPoint]) -> Vec<f64> {
    let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
    pts.iter().map(|q| (gv::thrust(&sv.reread(q).expect("a recorded point").base) - f0) / (f1 - f0)).collect()
}

/// `print_governor_lag_table` — rung 106.
pub fn governor_lag_table(p: &mut Printer, _d: &Design) {
    p.print("\nTHE GOVERNOR BESIDE THE SLOW TEMPERATURE LIMITER (rung 106): rung 95's governor holds thrust along the move and");
    p.print("rung 47's limiter cuts its fuel with a lag - the cut a state, measured off the governor's demand. The Controls");
    p.print("engine, 1000 -> 1400 K over 0.5 tau, the limiter at 1480 K, the perfect gas.");
    p.print(format!("\n  {:<14}{:>11}{:>11}{:>12}{:>14}{:>10}", "lag (tau)", "peak gov", "peak open", "thrust@0.5", "lowest after",
                    "95% thr"));
    let d = ControlsSettings { hold: Hold::Thrust, redline_on: true, ..ControlsSettings::defaults() };
    for lag in [None, Some(0.1), Some(0.2), Some(0.4), Some(0.8)] {
        let gov = ControlsSettings { govern: true, gov_lag_on: lag.is_some(), gov_lag: lag.unwrap_or(d.gov_lag), ..d };
        let open = ControlsSettings { govern: false, ..gov };
        let name = lag.map_or("instant".to_string(), |t| format!("{t}"));
        match (controls(&gov), controls(&open)) {
            (Ok((sv, o)), Ok((_, q))) => {
                let peak = |x: &[FuelPoint]| x.iter().map(|q| q.tt4).fold(f64::NEG_INFINITY, f64::max);
                let fr = fraction(&sv, &o.points);
                let k = o.points.iter().position(|q| (q.s - sv.settings.ramp).abs() < 1e-9).expect("the move's end");
                let lowest = fr[k..].iter().cloned().fold(f64::INFINITY, f64::min);
                let t95 = (1..fr.len()).find(|&j| fr[j - 1] < 0.95 && fr[j] >= 0.95).map(|j| {
                    let (a, b) = (&o.points[j - 1], &o.points[j]);
                    a.s + (0.95 - fr[j - 1]) / (fr[j] - fr[j - 1]) * (b.s - a.s)
                });
                p.print(format!("  {name:<14}{:>11.1}{:>11.1}{:>12.4}{:>14.4}{:>10}", peak(&o.points), peak(&q.points), fr[k], lowest,
                                t95.map_or("-".to_string(), |t| format!("{t:.3}"))));
            }
            (a, b) => p.print(format!("  {name:<14}{}", a.err().or(b.err()).map(|e| e.0).unwrap_or_default())),
        }
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  (peak: highest turbine inlet temperature, K, governed and on rung 47's open ramp with the same lag; thrust@0.5: the",
    "  share of the thrust step at the end of the move; lowest after: its lowest share from then on; 95% thr: time to 95%",
    "  of the step, tau.) A slow limiter beside a governor overshoots its redline twice as far as on the open ramp - the",
    "  governor's demand rises faster - and stores a larger cut, which it pays back after the move: the thrust climbs, then",
    "  SAGS, and reaches its target later than with an instant limiter. On the open ramp the thrust never falls.",
];
