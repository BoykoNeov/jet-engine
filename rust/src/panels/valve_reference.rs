//! Rung 87's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_valve_reference_table.txt`), a CHANGE detector only; its numbers
//! are checked by `tests/rung87.rs`. Seven riding cells on the shaped maps at `ds` = 0.005 (the
//! scoring grid is not repeated here: it takes minutes).

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::bleed_transient::LeverArm;
use crate::limited_bleed::build_limited_bleed;
use crate::pyfmt::Printer;
use crate::stator_transient::{Ramp, ScheduledStatorTransient, StatorArm, StatorSchedule};
use crate::valve_reference::{incidence_valve, matched_phi, run, sibling};

/// `print_valve_reference_table` — rung 87.
pub fn valve_reference_table(p: &mut Printer, d: &Design) {
    p.print("\nTHE CONTROLLED VALVE BESIDE THE STATOR SCHEDULE (rung 87): rung 64's bleed valve, opening");
    p.print("just enough to hold an LP stall margin, beside the LP stator schedule (0.20 below 0.65).");
    p.print("It can watch the flow coefficient (phi) or the rotor INCIDENCE (the margin to the metal).");
    p.print("Each row: the incidence valve, then the phi valve MATCHED to the same incidence minimum;");
    p.print("bleed = time integral of the opening, dF = thrust integral against the valve-shut run.");
    let m = match build_limited_bleed(design13(d), d.flight, 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                                      &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("both maps are passed"),
    };
    let stat = LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)));
    p.print(format!("\n  {:>5}{:>6}{:>10}{:>12}{:>12}{:>12}{:>10}{:>12}{:>12}{:>8}", "r", "sm", "M_i held",
                    "inc bleed", "inc no-stat", "inc dF", "phi* sm", "phi bleed", "phi dF", "ratio"));
    for (r, sm) in [(0.5, 0.42), (0.5, 0.46), (0.5, 0.50), (1.0, 0.42), (1.0, 0.46), (1.0, 0.50), (1.0, 0.54)] {
        let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds: 0.005 };
        let phi_lim = 0.55 * (1.0 + sm);
        let iv = incidence_valve(phi_lim, 0.10);
        let shut = run(&sibling(&m, Some(&stat), None), &d.flight, &rp, None);
        let inc = run(&sibling(&m, Some(&stat), Some(iv)), &d.flight, &rp, None);
        let off = run(&sibling(&m, None, Some(iv)), &d.flight, &rp, None);
        let (phi, pr) = matched_phi(&m, &d.flight, &rp, Some(&stat), 0.10, inc.m_i, 1e-10)
            .expect("a riding cell brackets");
        p.print(format!("  {r:>5.2}{sm:>6.2}{:>10.5}{:>12.4e}{:>12.4e}{:>12.4}{:>10.4}{:>12.4e}{:>12.4}{:>8.3}",
                        inc.m_i, inc.b_int, off.b_int, inc.thrust_int - shut.thrust_int, phi / 0.55 - 1.0,
                        pr.b_int, pr.thrust_int - shut.thrust_int, pr.b_int / inc.b_int));
    }
    p.print("\n  At EQUAL protection the phi valve bleeds 1.3-3.5x more and costs more thrust: it holds a");
    p.print("  CONSTANT flow coefficient, so it over-protects early in the ramp where the stators are most");
    p.print("  closed, to be safe late where they have reopened. The incidence valve opens only as far as");
    p.print("  each moment needs -- and the stators pay part of its bill (it bleeds less with them than");
    p.print("  without), where the phi valve fights them (more). Beside a phi FUEL floor the incidence");
    p.print("  valve lets phi fall and the floor starves the run: watch the same wall with both.");
}
