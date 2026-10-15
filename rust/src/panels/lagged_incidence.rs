//! Rung 88's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_lagged_incidence_table.txt`), a CHANGE detector only; its numbers
//! are checked by `tests/rung88.rs`. One riding cell on the shaped maps at `ds` = 0.005 (the
//! 19-cell scoring grid takes minutes and is not repeated here).

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::bleed_transient::LeverArm;
use crate::lagged_bleed::build_lagged_bleed;
use crate::lagged_incidence::{matched_lagged, run_lagged, trapz, valve_at};
use crate::limited_bleed::{BleedLimiter, ValveReference};
use crate::pyfmt::Printer;
use crate::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use crate::two_spool::Spool;
use crate::valve_reference::sibling;

/// `print_lagged_incidence_table` — rung 88.
pub fn lagged_incidence_table(p: &mut Printer, d: &Design) {
    p.print("\nTHE SLOW VALVE ON A MOVING TARGET (rung 88): rung 87's bleed valve beside the LP stator");
    p.print("schedule, given rung 65's lag tau (the valve chases its instant opening). Shaped maps,");
    p.print("r = 0.5, set point sm = 0.42 (the incidence margin m_lim it is asked to hold).");
    let m: ScheduledStatorCore = match build_lagged_bleed(design13(d), d.flight, 1.0, Some(lp_map()), Some(hp_map()),
                                                          1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("both maps are passed"),
    };
    let stat = LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)));
    let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.005 };
    let (sm, b_max) = (0.42, 0.10);
    let phi_lim = 0.55 * (1.0 + sm);
    let m_lim = m.design_map(Spool::Lp).tan_beta1_crit() - 1.0 / phi_lim;
    let inst = sibling(&m, Some(&stat), Some(BleedLimiter::new(phi_lim, b_max).on_incidence()));
    let ti = crate::three_levers::march(&inst, &d.flight, &rp, None).0;
    let bi: Vec<f64> = ti.iter().map(|q| inst.b_at_point(&d.flight, q)).collect();
    let int_bi = {
        let mut a = 0.0;
        for k in 1..ti.len() { a += 0.5 * (ti[k].s - ti[k - 1].s) * (bi[k] + bi[k - 1]); }
        a
    };
    p.print("\n  1. The incidence valve, made slow. Its bleed bill is tau x the opening it starts with");
    p.print("     (here 0: the stators protect the start) plus a small drift; its protection falls.");
    p.print(format!("\n  {:>6}{:>12}{:>12}{:>12}{:>12}", "tau", "M_i - m_lim", "b(0)", "extra bleed", "drift"));
    for tau in [0.01, 0.05, 0.2] {
        let ml = sibling(&m, Some(&stat), Some(BleedLimiter::with_tau(phi_lim, b_max, Some(tau)).on_incidence()));
        let (tl, lr) = run_lagged(&ml, &d.flight, &rp);
        let n = tl.len().min(ti.len());
        let cmd = trapz(&tl, n, |q| valve_at(q).1);
        p.print(format!("  {tau:>6.2}{:>12.3e}{:>12.3e}{:>12.3e}{:>12.3e}", lr.run.m_i - m_lim, lr.b0, lr.int_b - int_bi,
                        cmd - int_bi));
    }
    p.print("\n  2. Both valves re-tuned, at each speed, to hold the SAME incidence margin m_lim:");
    p.print(format!("\n  {:>6}{:>10}{:>12}{:>10}{:>12}{:>9}{:>10}", "tau", "inc sm", "inc bleed", "phi sm", "phi bleed",
                    "ratio", "dF"));
    for tau in [None, Some(0.01), Some(0.05), Some(0.2)] {
        let fit = |rf| matched_lagged(&m, &d.flight, &rp, Some(&stat), b_max, tau, rf, m_lim, 1e-10)
            .expect("the panel's cell brackets");
        let ((xi, tin, _), (xp, tph, _)) = (fit(ValveReference::Incidence), fit(ValveReference::Phi));
        let n = tin.len().min(tph.len());
        let bleed = |t: &[crate::fuel_transient::FuelPoint], x: f64, inc: bool| {
            let v = BleedLimiter::with_tau(x, b_max, tau);
            let mm = sibling(&m, Some(&stat), Some(if inc { v.on_incidence() } else { v }));
            let b: Vec<f64> = t.iter().map(|q| mm.b_at_point(&d.flight, q)).collect();
            let mut a = 0.0;
            for k in 1..n { a += 0.5 * (t[k].s - t[k - 1].s) * (b[k] + b[k - 1]); }
            a
        };
        let (b_i, b_p) = (bleed(&tin, xi, true), bleed(&tph, xp, false));
        let df = trapz(&tph, n, |q| q.sp_thrust * q.mdot_air) - trapz(&tin, n, |q| q.sp_thrust * q.mdot_air);
        let tag = tau.map(|t| format!("{t:.2}")).unwrap_or_else(|| "inst".to_string());
        p.print(format!("  {tag:>6}{:>10.4}{b_i:>12.4e}{:>10.4}{b_p:>12.4e}{:>9.3}{df:>10.3}", xi / 0.55 - 1.0,
                        xp / 0.55 - 1.0, b_p / b_i));
    }
    p.print("\n  ratio = phi valve's bleed / incidence valve's; dF = phi's thrust integral minus incidence's.");
    p.print("  Instant, the incidence valve is the cheaper one (rung 87). Slow enough, the order REVERSES:");
    p.print("  the incidence valve waits until the margin is needed and then must open fast, so a lag costs");
    p.print("  it protection it has to buy back; the phi valve holds incidence margin it does not need, and");
    p.print("  that surplus absorbs the lag. Where the crossover sits in tau is a measurement");
    p.print("  on this rig's own time scale, not a statement about real valves.");
}
