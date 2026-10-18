//! Rung 97's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_unchoked_lp_table.txt`), a CHANGE detector only; its numbers are checked by
//! `tests/rung97.rs`. The Controls engine's 700 K steady end where the nozzle is unchoked, against rung 38's pin (the
//! old model), at sea level on a +40 K and a standard day.

use super::Design;
use crate::atmosphere::standard;
use crate::matcher::Branch;
use crate::pyfmt::Printer;
use crate::sandbox_controls::{build, ControlsSettings, ControlsSolver, CONTROL_SHAPES};

/// `print_unchoked_lp_table` — rung 97.
pub fn unchoked_lp_table(p: &mut Printer, _d: &Design) {
    p.print("\nTHE UNCHOKED NOZZLE ON TWO SHAFTS (rung 97): below the unchoke boundary the LP turbine expands only as far as");
    p.print("still lets the nozzle pass the flow against the outside air. The 700 K steady end at sea level, Mach 0.85,");
    p.print("on the Controls engine, against rung 38's pin (the old model: the nozzle throat treated as choked).");
    p.print(format!("\n  {:<9}{:<12}{:>9}{:>9}{:>11}{:>11}{:>9}{:>9}{:>8}", "day", "map", "pi_LPT", "old", "LP speed",
                    "old", "thrust", "old", "unch."));
    let (t, p0) = standard(0.0);
    for (day, dt) in [("+40 K", 40.0), ("standard", 0.0)] {
        for (shape, map) in CONTROL_SHAPES.iter().enumerate() {
            let s = ControlsSettings { shape, t0: t + dt, p0, from: 700.0, to: 1400.0, ..ControlsSettings::defaults() };
            match build(&s) {
                Ok(sv) => {
                    let core = &sv.core.fuel.inner;
                    core.inner.base.lp_pinned.set(true);
                    let old = core.try_equilibrium(&sv.flight, 700.0, None).map(|x| x.0);
                    core.inner.base.lp_pinned.set(false);
                    let unch = sv.march().iter().filter(|x| x.branch != Branch::Choked).count();
                    let n = &sv.start;
                    match old {
                        Ok(o) => p.print(format!("  {day:<9}{map:<12}{:>9.4}{:>9.4}{:>11.5}{:>11.5}{:>9.2}{:>9.2}{:>8}",
                                                 n.pi_lpt, o.pi_lpt, n.nu_lp, o.nu_lp, ControlsSolver::thrust(n),
                                                 ControlsSolver::thrust(&o), unch)),
                        Err(e) => p.print(format!("  {day:<9}{map:<12}old model: {}", e.0)),
                    }
                }
                Err(e) => p.print(format!("  {day:<9}{map:<12}{}", e.0.chars().take(60).collect::<String>())),
            }
        }
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  (pi_LPT: the LP turbine's pressure ratio, exit over inlet; LP speed: fraction of design; thrust: N per kg/s of",
    "  design airflow; unch.: how many of the 700 -> 1400 K move's 126 points have the nozzle unchoked.) The compressors,",
    "  the HP nozzle guide vane and the HP turbine never read the LP turbine, so the change takes work off the LP shaft",
    "  alone: it settles slower, and the LP turbine expands less.",
];
