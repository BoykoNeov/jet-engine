//! Rung 98's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_corrected_tables_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung98.rs`. The thrust-inferring control on tables made at the design flight, flown elsewhere: the final end's
//! steady miss with both corrections, with the temperature one off, and with the pressure one off.

use super::Design;
use crate::burner_hold::Hold;
use crate::pyfmt::Printer;
use crate::sandbox_controls::{build, ControlsSettings, ThrustBy};

/// The final end's (achieved − asked)/asked, %, or the refusal's first words.
fn miss(s: &ControlsSettings) -> String {
    match build(s) {
        Ok(sv) => {
            let h = sv.hold_end.expect("a carried inferred run calibrates its ends");
            format!("{:+.2}", 100.0 * (h.achieved - h.target) / h.target)
        }
        Err(_) => "refused".to_string(),
    }
}

/// `print_corrected_tables_table` — rung 98.
pub fn corrected_tables_table(p: &mut Printer, _d: &Design) {
    p.print("\nTHE CORRECTED TABLES (rung 98): the control's tables made once at the design flight (250 K, 50 kPa, Mach 0.85)");
    p.print("and read elsewhere through the air's pressure and temperature. The thrust the governor gives at the 1400 K end,");
    p.print("over what the throttle asks, %, on the map curved along flow at LP, along pressure at HP:");
    p.print(format!("\n  {:<22}{:>10}{:>10}{:>10}{:>10}{:>10}{:>10}", "flight", "EPR", "no temp", "no press", "fan", "no temp",
                    "no press"));
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, ..ControlsSettings::defaults() };
    for (name, t0, p0, m0) in [("hot 290 K", 290.0, 50_000.0, 0.85), ("cold 216.65 K", 216.65, 50_000.0, 0.85),
                               ("thin 26.5 kPa", 250.0, 26_500.0, 0.85), ("Mach 0.5", 250.0, 50_000.0, 0.5),
                               ("Mach 1.4", 250.0, 50_000.0, 1.4)] {
        let mut row = format!("  {name:<22}");
        for by in [ThrustBy::Epr, ThrustBy::N1] {
            let s = ControlsSettings { thrust_by: by, t0, p0, m0, ..d };
            for c in [s, ControlsSettings { corr_t: false, ..s }, ControlsSettings { corr_p: false, ..s }] {
                row += &format!("{:>10}", miss(&c));
            }
        }
        p.print(row);
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  (EPR: thrust read from the pressure ratio; fan: from the LP speed; no temp / no press: that correction off.)",
    "  The pressure correction is exact here and the temperature one nearly so; the pressure ratio needs no temperature",
    "  correction at all, the fan speed misses by a quarter or more without it. No correction sees a change of flight",
    "  speed: the jet's push against the outside air and the drag of the air swallowed change, the engine's inside does not.",
];
