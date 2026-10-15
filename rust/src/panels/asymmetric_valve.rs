//! Rung 89's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_asymmetric_valve_table.txt`), a CHANGE detector only; its numbers
//! are checked by `tests/rung89.rs`. One cell on the shaped maps at `ds` = 0.005 (the 19-cell,
//! 25-pair scoring grid takes many minutes and is not repeated here).

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::asymmetric_valve::{matched_asym, phi_floor};
use crate::bleed_transient::LeverArm;
use crate::lagged_bleed::build_lagged_bleed;
use crate::lagged_incidence::{trapz, valve_at};
use crate::limited_bleed::ValveReference;
use crate::pyfmt::Printer;
use crate::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use crate::two_spool::Spool;

/// `print_asymmetric_valve_table` — rung 89.
pub fn asymmetric_valve_table(p: &mut Printer, d: &Design) {
    p.print("\nTHE ASYMMETRIC VALVE (rung 89): rung 88's slow bleed valve beside the LP stator schedule,");
    p.print("opening at one time constant and closing at another. Shaped maps, r = 0.5, both valves");
    p.print("re-tuned at every pair to hold the same incidence margin m_lim (set point sm = 0.46).");
    let m: ScheduledStatorCore = match build_lagged_bleed(design13(d), d.flight, 1.0, Some(lp_map()), Some(hp_map()),
                                                          1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("both maps are passed"),
    };
    let stat = LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)));
    let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.005 };
    let b_max = 0.10;
    let m_lim = m.design_map(Spool::Lp).tan_beta1_crit() - 1.0 / (0.55 * 1.46);
    let taus = [0.01, 0.05, 0.2];
    let fit = |to: f64, tc: f64, rf| matched_asym(&m, &d.flight, &rp, Some(&stat), b_max, to, tc, rf, m_lim, 1e-10)
        .expect("the panel's cell brackets");
    p.print("\n  1. Who is cheaper: the phi valve's bleed over the incidence valve's (below 1: the phi valve).");
    p.print("     Rows: how fast it OPENS; columns: how fast it CLOSES (spool time constants).");
    p.print(format!("\n  {:>10}{:>10}{:>10}{:>10}", "open\\close", taus[0], taus[1], taus[2]));
    let mut sets = Vec::new();
    for to in taus {
        let mut line = format!("  {to:>10.2}");
        for tc in taus {
            let ((xi, ti, _), (xp, tp, _)) = (fit(to, tc, ValveReference::Incidence), fit(to, tc, ValveReference::Phi));
            let n = ti.len().min(tp.len());
            let ratio = trapz(&tp, n, |q| valve_at(q).0) / trapz(&ti, n, |q| valve_at(q).0);
            line += &format!("{ratio:>10.3}");
            sets.push((to, tc, xi / 0.55 - 1.0, xp / 0.55 - 1.0));
        }
        p.print(line);
    }
    p.print("\n  2. The set points each valve needs (sm), and the phi valve's FLOOR: the lowest any closing");
    p.print("     speed can buy it (its margin before it first closes, which no closing speed touches).");
    p.print(format!("\n  {:>6}{:>7}{:>10}{:>10}{:>10}", "open", "close", "inc sm", "phi sm", "floor"));
    let floors: Vec<String> = taus.iter().map(|&to| phi_floor(&m, &d.flight, &rp, Some(&stat), b_max, to, m_lim, 1e-10)
        .map(|(x, _)| format!("{:>10.4}", x / 0.55 - 1.0)).unwrap_or_else(|| format!("{:>10}", "-"))).collect();
    for (k, (to, tc, si, sp)) in sets.into_iter().enumerate() {
        p.print(format!("  {to:>6.2}{tc:>7.2}{si:>10.4}{sp:>10.4}{}", floors[k / taus.len()]));
    }
    p.print("\n  Each valve is priced by ONE edge. The incidence valve's set point is its OPENING speed's:");
    p.print("  its margin bottoms out before it ever closes, so how fast it closes only adds or saves the");
    p.print("  air it holds on the way down. The phi valve's margin bottoms out AFTER it starts closing, so");
    p.print("  its set point is its CLOSING speed's (0.3805-0.3812 down the 0.01 column, whatever the");
    p.print("  opening) - until the closing is slow enough that the minimum comes first, and it sits on its");
    p.print("  floor. (At an opening of 0.2 the symmetric valve's minimum already comes first: its 'floor'");
    p.print("  is its own set point.) So the closing speed moves the verdict BOTH ways: a slow close makes");
    p.print("  the phi valve the cheaper one at every opening speed; a fast close hands the incidence valve");
    p.print("  the slow openings too. Over the 19 scored cells (spec section 3) the incidence valve is cheaper");
    p.print("  in 68 of 95 runs closing at 0.01 and in 0 of 95 closing at 0.2 - against 54 and 11 of 95 when");
    p.print("  the OPENING is held at 0.01 or 0.2: the closing speed is the stronger dial.");
}
