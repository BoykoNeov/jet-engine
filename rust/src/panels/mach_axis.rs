//! Rung 99's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_mach_axis_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung99.rs`. The thrust-inferring control on design-flight tables, flown at other speeds in the design
//! altitude's air: each end's steady miss with no speed correction (rung 98), on the speed grid, and with the thrust
//! recomputed for the flight speed.

use super::Design;
use crate::burner_hold::Hold;
use crate::mach_axis::SpeedCorr;
use crate::pyfmt::Printer;
use crate::sandbox_controls::{build, ControlsSettings, ThrustBy};

/// Both ends' (achieved − asked)/asked, %, or "refused".
fn misses(s: &ControlsSettings) -> String {
    match build(s) {
        Ok(sv) => {
            let (a, b) = (sv.hold_start.expect("a carried inferred run calibrates its ends"), sv.hold_end.expect("both ends"));
            format!("{:+.2}/{:+.2}", 100.0 * (a.achieved - a.target) / a.target, 100.0 * (b.achieved - b.target) / b.target)
        }
        Err(_) => "refused".to_string(),
    }
}

/// `print_mach_axis_table` — rung 99.
pub fn mach_axis_table(p: &mut Printer, _d: &Design) {
    p.print("\nTHE SPEED AXIS (rung 99): the control's tables made at Mach 0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5 and 3.0 in the");
    p.print("design altitude's air and blended at the speed flown, or the thrust recomputed for it. The thrust the governor");
    p.print("gives at the 1000 K / 1400 K ends over what the throttle asks, %, pressure ratio read, on the map curved along");
    p.print("flow at LP, along pressure at HP, flown in 250 K, 50 kPa air:");
    p.print(format!("\n  {:<10}{:>16}{:>16}{:>16}", "Mach", "not corrected", "speed grid", "recomputed"));
    // Rung 99 as it shipped: rung 100's reduce settings (no slow slices, rung 99's formula column).
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, t0: 250.0, p0: 50_000.0,
                               low_grid: false, formula_choked: true, ..ControlsSettings::defaults() };
    for m0 in [0.15, 0.5, 1.4] {
        let mut row = format!("  {m0:<10}");
        for c in [SpeedCorr::None, SpeedCorr::Grid, SpeedCorr::Formula] {
            row += &format!("{:>16}", misses(&ControlsSettings { m0, speed_corr: c, ..d }));
        }
        p.print(row);
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  (Mach 0.15 lies below the grid: the line through the Mach 0.3 and 0.6 tables is carried on.)",
    "  A table made at Mach 0.85 misses by 5-23 %. The grid leaves the bend of the thrust-against-speed curve between its",
    "  tables: short where the curve bends up, over where the nozzle's push bends it down, and more past the grid's end.",
    "  Recomputing the thrust for the flight speed leaves only the table's guess of the air swallowed - while the nozzle",
    "  is choked.",
];
