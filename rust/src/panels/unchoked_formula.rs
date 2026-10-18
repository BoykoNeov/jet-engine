//! Rung 100's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_unchoked_formula_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung100.rs`. The thrust-inferring control on design-flight tables, flown slowly from a low throttle where the
//! nozzle is not choked: the starting end's steady miss under rung 99's formula, rung 100's, and the speed grid.

use super::Design;
use crate::burner_hold::Hold;
use crate::mach_axis::SpeedCorr;
use crate::pyfmt::Printer;
use crate::sandbox_controls::{build, ControlsSettings, ThrustBy};

/// The starting end's (achieved − asked)/asked, %, or "refused".
fn miss(s: &ControlsSettings) -> String {
    match build(s) {
        Ok(sv) => {
            let a = sv.hold_start.expect("a carried inferred run calibrates its ends");
            format!("{:+.2}", 100.0 * (a.achieved - a.target) / a.target)
        }
        Err(_) => "refused".to_string(),
    }
}

/// `print_unchoked_formula_table` — rung 100.
pub fn unchoked_formula_table(p: &mut Printer, _d: &Design) {
    p.print("\nBELOW CHOKE (rung 100): the formula worked out at the reading - the jet's push from the pressure ratio, the LP");
    p.print("shaft rebalanced against the flight's air for the fan speed - and two slow tables (Mach 0.05, 0.15) in the speed");
    p.print("grid. The thrust the governor gives at the 700 K starting end over what the throttle asks, %, on the map curved");
    p.print("along flow at LP, along pressure at HP, flown in 250 K, 50 kPa air (Mach 0.1 lies between the slow tables):");
    p.print(format!("\n  {:<8}{:<16}{:>13}{:>13}{:>13}{:>13}", "Mach", "read", "rung 99", "rung 100", "grid 99", "grid 100"));
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, from: 700.0, t0: 250.0, p0: 50_000.0,
                               ..ControlsSettings::defaults() };
    for m0 in [0.1, 0.5, 1.4] {
        for (by, name) in [(ThrustBy::Epr, "pressure ratio"), (ThrustBy::N1, "fan speed")] {
            let s = ControlsSettings { m0, thrust_by: by, ..d };
            let f = ControlsSettings { speed_corr: SpeedCorr::Formula, ..s };
            p.print(format!("  {m0:<8}{name:<16}{:>13}{:>13}{:>13}{:>13}",
                            miss(&ControlsSettings { formula_choked: true, ..f }), miss(&f),
                            miss(&ControlsSettings { low_grid: false, ..s }), miss(&s)));
        }
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  The pressure ratio's push is now exact below choke: what is left is the table's guess of the air swallowed - small",
    "  at low speed, but at Mach 1.4 from 700 K larger than rung 99's, whose push and airflow came from the same unchoked",
    "  design row and cancelled. The fan speed's formula overshoots: it finds the hotter burner but reads the choked fan's",
    "  compressors. The slow tables bring the grid back, for both readings: an unchoked engine leaves the table's one",
    "  line, and the flight speed is the second number that finds it.",
];
