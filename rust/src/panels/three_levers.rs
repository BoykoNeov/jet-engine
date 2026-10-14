//! Rung 86's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_three_levers_table.txt`), a CHANGE detector only; its numbers are
//! checked by `tests/rung86.rs`. Ten cubes at `ds` = 0.005 (the gate's full grid is not repeated
//! here: it takes minutes).

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use crate::fuel_transient::AsymmetricLag;
use crate::pyfmt::Printer;
use crate::stator_transient::{Ramp, ScheduledStatorTransient, StatorArm, StatorSchedule};
use crate::three_levers::{cube, Cube, Limiter};
use crate::two_spool::Spool;

fn reached(c: &Cube) -> String {
    ["none", "B", "S", "SB"].iter().zip(c.reached).filter(|(_, r)| *r).map(|(n, _)| *n)
        .collect::<Vec<_>>().join(",")
}

/// `print_three_levers_table` — rung 86.
pub fn three_levers_table(p: &mut Printer, d: &Design) {
    p.print("\nFUEL + BLEED + STATOR (rung 86): the three levers on ONE plant -- rung 62's machine with");
    p.print("the LP stator schedule (0.20 below 0.65) and the bleed schedule (0.10 below 0.65), plus ONE");
    p.print("fuel limiter. Eight on/off cells, each the LP incidence-margin minimum; T is the THREE-WAY");
    p.print("term, what exists only with all three on: T = I_SB(limiter on) - I_SB(limiter off).");
    let m = match build_scheduled_bleed(design13(d), d.flight, 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                                       &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("both maps are passed"),
    };
    let stat = LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)));
    let bled = LeverArm::scheduled(BleedSchedule::new(0.10, 0.65));
    let lag = AsymmetricLag::new(0.02, 0.10);
    let rows: [(Limiter, f64); 10] = [
        (Limiter::Accel { margin: 0.25 }, 0.25),
        (Limiter::Accel { margin: 0.25 }, 0.5),
        (Limiter::Accel { margin: 0.25 }, 1.0),
        (Limiter::AccelLag { margin: 0.25, lag }, 0.5),
        (Limiter::Topping { tt4_max: 1480.0 }, 0.25),
        (Limiter::Topping { tt4_max: 1480.0 }, 0.5),
        (Limiter::Topping { tt4_max: 1480.0 }, 1.0),
        (Limiter::ToppingLag { tt4_max: 1480.0, tau_gov: 0.2 }, 0.5),
        (Limiter::IncidenceFloor { sm: 0.42 }, 0.5),
        (Limiter::PhiFloor { sm: 0.38 }, 0.5),
    ];
    p.print(format!("\n  {:<24}{:>5}{:>13}{:>13}{:>13}{:>8}  {:<14}{}", "limiter (shaped maps)", "r",
                    "I_SB off", "I_SB on", "T", "T/-I", "reaches", "runs"));
    for (lim, r) in rows {
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds: 0.005 };
        let c = cube(&m, &d.flight, &ramp, Spool::Lp, &stat, &bled, lim);
        let full = c.cells.iter().map(|x| x.read.npts).max().unwrap_or(0);
        let short: Vec<String> = c.cells.iter().enumerate().filter(|(_, x)| x.read.npts < full)
            .map(|(k, x)| format!("S{}B{}F{}:{}", k / 4, (k / 2) % 2, k % 2, x.read.npts)).collect();
        let runs = if short.is_empty() { "all full".to_string() } else { format!("short {}", short.join(" ")) };
        let label = match lim {
            Limiter::IncidenceFloor { sm } | Limiter::PhiFloor { sm } => format!("{} sm={sm:.2}", lim.key()),
            _ => lim.key().to_string(),
        };
        p.print(format!("  {label:<24}{r:>5.2}{:>13.4e}{:>13.4e}{:>13.4e}{:>8.3}  {:<14}{runs}",
                        c.i_sb[0], c.i_sb[1], c.t, c.t / -c.i_sb[0], reached(&c)));
    }
    p.print("\n  The acceleration schedule pulls the reading EARLIER, before the stator-bleed interaction");
    p.print("  has built up: T has the opposite sign to I_SB and is a fraction of it, computable from the");
    p.print("  limiter-free runs (rung 58's predictor recovers 71-93 %). The turbine limiter reaches only");
    p.print("  the runs whose minimum is hotter than its redline, so T is a ledger of WHERE the pair puts");
    p.print("  the minimum. The incidence floor is a ledger of its set point; the flow-coefficient floor");
    p.print("  fights the stator (which lowers the flow coefficient) and can starve the run -- the bleed");
    p.print("  rescues it.");
}
