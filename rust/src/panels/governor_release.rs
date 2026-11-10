//! Rung 107's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_governor_release_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung107.rs`. The Controls engine's default slam with the acceleration schedule and rung 52's realistic release
//! (attack 0.2): the governed thrust against rung 52's open ramp at each release time — the scored grid is the spec's.

use super::Design;
use crate::burner_hold::Hold;
use crate::fuel_transient::{FuelPoint, PointExtra};
use crate::governor as gv;
use crate::pyfmt::Printer;
use crate::sandbox_controls::{controls, ControlsSettings, ControlsSolver, DS};

/// The thrust step's fraction at every recorded point.
fn fraction(sv: &ControlsSolver, pts: &[FuelPoint]) -> Vec<f64> {
    let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
    pts.iter().map(|q| (gv::thrust(&sv.reread(q).expect("a recorded point").base) - f0) / (f1 - f0)).collect()
}

/// The deepest cut the lag held (0 on a route without one).
fn deepest(pts: &[FuelPoint]) -> f64 {
    pts.iter().map(|q| if let PointExtra::Asym { g, .. } = q.extra { g } else { 0.0 }).fold(0.0, f64::max)
}

/// `print_governor_release_table` — rung 107.
pub fn governor_release_table(p: &mut Printer, _d: &Design) {
    p.print("\nTHE GOVERNOR BESIDE THE REALISTIC RELEASE (rung 107): rung 95's governor holds thrust along the move and");
    p.print("rung 52's acceleration-schedule limiter cuts its fuel fast and gives it back slowly - the cut a state,");
    p.print("measured off the governor's demand. The Controls engine, 1000 -> 1400 K over 0.5 tau, attack 0.2 tau, no");
    p.print("redline, the perfect gas.");
    p.print(format!("\n  {:<14}{:>11}{:>11}{:>12}{:>10}{:>10}", "release (tau)", "cut gov", "cut open", "thrust@0.5", "sag",
                    "95% thr"));
    let d = ControlsSettings { hold: Hold::Thrust, accel_on: true, tau_att: 0.2, ..ControlsSettings::defaults() };
    for rel in [None, Some(0.1), Some(0.2), Some(0.4), Some(0.8), Some(1.0)] {
        let gov = ControlsSettings { govern: true, release_on: rel.is_some(), tau_rel: rel.unwrap_or(d.tau_rel), ..d };
        let open = ControlsSettings { govern: false, release_on: true, ..gov };
        let name = rel.map_or("instant".to_string(), |t| format!("{t}"));
        match (controls(&gov), controls(&open)) {
            (Ok((sv, o)), Ok((_, q))) => {
                let fr = fraction(&sv, &o.points);
                let k = (sv.settings.ramp / DS).round_ties_even() as usize;
                let (mut hi, mut sag) = (f64::NEG_INFINITY, 0.0f64);
                for &v in &fr[k..] {
                    hi = hi.max(v);
                    sag = sag.max(hi - v);
                }
                let t95 = (1..fr.len()).find(|&j| fr[j - 1] < 0.95 && fr[j] >= 0.95).map(|j| {
                    let (a, b) = (&o.points[j - 1], &o.points[j]);
                    a.s + (0.95 - fr[j - 1]) / (fr[j] - fr[j - 1]) * (b.s - a.s)
                });
                let cut_open = if rel.is_some() { format!("{:.3e}", deepest(&q.points)) } else { "-".to_string() };
                p.print(format!("  {name:<14}{:>11.3e}{:>11}{:>12.4}{:>10.4}{:>10}", deepest(&o.points), cut_open, fr[k], sag,
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
    "\n  (cut: the deepest fuel cut the slow limiter held, kg/s, governed and on rung 52's open ramp with the same times;",
    "  thrust@0.5: the share of the thrust step at the end of the move; sag: the largest fall of that share below its",
    "  running peak from then on; 95% thr: time to 95% of the step, tau, '-' if not within the run.) Beside a governor the",
    "  limiter cuts more than twice as deep - the governor asks for more fuel - but the slow ATTACK lets fuel through",
    "  first: the engine runs ahead (over three quarters of the step by the move's end, against a third with the instant",
    "  limiter), then the governor pulls it back and the thrust SAGS. A slower release deepens the sag only a little. On",
    "  the open ramp, and with the instant limiter, the thrust never falls. Holding thrust only: under the other holds the",
    "  governor sags without any lag. The temperature limiter is off - the sag needs no slow temperature limiter (106).",
];
