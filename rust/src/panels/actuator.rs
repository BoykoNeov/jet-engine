//! Rungs 72–77's panels — two loops on one actuator, the applied reference, the demand
//! coordinate, the declared anti-windup device, the sensed cap and the stiffness ledger (slice AQ,
//! second part).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line for
//! line, on rung 53's hardware. **The march step is read off each reader's own SIGNATURE** (none
//! of these panels passes `ds`): 0.005 for the trajectory readers (`authority_law`,
//! `handover_law`, `applied_bill`, `forcing_openloop`, `demand_law`, `windup_law` and every rung
//! 75–77 reader), **0.002** for the Jacobian readers (`shared_gains`, `shared_cells`,
//! `mask_discriminator`, `applied_gains`, `applied_cells`, `ref_discriminator`, `demand_gains`).
//! A Python `dict` keyed by a tuple or a formatted string is a Rust `Vec` searched by its key; a
//! readers' `Result`/`Option` is unwrapped where Python would have raised.

use super::cascades::{machine, B, FLOOR, HI, LO};
use super::Design;
use crate::anti_windup::{build_anti_windup_cascade, contraction_law, windup_bill, windup_gains,
                         WindupCell};
use crate::applied_reference::{
    applied_bill, applied_cells, applied_gains, build_applied_reference_cascade, handover_law,
    ref_discriminator,
};
use crate::bleed_transient::LeverArm;
use crate::demand_coordinate::{
    build_demand_coordinate_cascade, demand_gains, demand_law, forcing_openloop, windup_law,
    CoordRead, WindupCell as DemandCell,
};
use crate::fuel_transient::Authority;
use crate::limited_bleed::BleedLimiter;
use crate::map::ComponentMap;
use crate::pct;
use crate::pyf;
use crate::pyfmt::{py_list, py_tuple, Printer, PyFormat, PyRaw};
use crate::reference_split::StatorIncidenceLimiter;
use crate::sensed_cap::{build_sensed_cap_cascade, cap_bill, cap_gains, solve_gain, CapCell};
use crate::shared_actuator::{
    authority_law, build_shared_actuator_cascade, mask_discriminator, shared_cells, shared_gains,
};
use crate::stator_transient::ScheduledStatorCore;
use crate::stiffness_ledger::{
    build_stiffness_ledger_cascade, leg_slopes, set_point_gains, singular_limit, Leg,
};
use crate::three_loop::StatorLimiter;

use super::airflow::lp_map;

const TMAX: f64 = 1200.0;
/// Python's default `taus=(0.05, 0.05, 0.05, 0.05)`.
pub(crate) const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);

/// `str(tuple)` of a four-clock arm.
fn taus4((a, b, c, d): (f64, f64, f64, f64)) -> PyRaw {
    py_tuple(&[&a, &b, &c, &d])
}

/// `str(list)` of integers.
fn ints<T: PyFormat>(xs: &[T]) -> PyRaw {
    let v: Vec<&dyn PyFormat> = xs.iter().map(|x| x as &dyn PyFormat).collect();
    py_list(&v)
}

fn auth_str(a: Authority) -> &'static str {
    match a {
        Authority::Fuel => "fuel",
        Authority::Gov => "gov",
        _ => unreachable!("only a LIVE authority names a cell"),
    }
}

/// `bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=0.05)` plus, per `inc`, rung 69's incidence
/// stator or rung 68's `phi` stator — the `rig(phi_lim, inc)` of rungs 74–76.
fn rig_arm(lp: &ComponentMap, sm: f64, inc: bool) -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(lp, B, sm, Some(0.05))),
        stator_inc: inc.then(|| StatorIncidenceLimiter::from_margin(lp, 0.20, sm, Some(0.05))),
        stator_lim: (!inc).then(|| StatorLimiter::from_margin(lp, 0.20, sm, Some(0.05))),
        ..LeverArm::default()
    }
}

/// `print_shared_actuator_table(flight)` — rung 72.
pub fn shared_actuator_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTWO LOOPS ON ONE ACTUATOR (rung 72): a shared actuator adds a SWITCH BETWEEN");
    p.print("PLANTS, not a loop -- so `n` counts the loops holding AUTHORITY, not the states.");

    let sm = super::cascades::PHI / FLOOR - 1.0;
    let lp = lp_map();
    let t = machine(build_shared_actuator_cascade, d, &LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp, B, sm, Some(0.05))),
        ..LeverArm::default()
    });
    let clocks = [(0.05, 0.05, 0.05, 0.05), (0.20, 0.01, 0.50, 0.05)];

    p.print("\n  THE ONE MODELLING DECISION, DECLARED (there is no prior rung to inherit it from):");
    p.print("    mf = mf_sched - max(gf, gr)     MIN-SELECT in clip coordinates.  THE PLANT.");
    p.print("    mf = mf_sched - gf - gr         the SUM law.  AN INSTRUMENT, never the plant.");

    p.print("\n  AUTHORITY IS EXCLUSIVE, AND IT CHANGES HANDS ONCE -- INSIDE THE JOINT WINDOW:");
    let al = authority_law(&t, flight, LO, HI, TMAX, sm, &clocks, 0.5, 1.2, 0.005, 0.20);
    p.print(pyf!("    {:>10} {:>22} {:>16} {:>6} {:>5} {:>10}",
                 "stator", "clocks (f,g,q,s)", "joint window", "fuel", "gov", "hand-over"));
    for a in &al.arms {
        let lbl = if a.inc { "incidence" } else { "phi" };
        let w = pyf!("{:.3f}-{:.3f}", a.joint.lo.expect("the windows overlap"),
                     a.joint.hi.expect("the windows overlap"));
        let ho = match a.handovers.first() {
            Some(h) => pyf!("{:.3f}", h),
            None => "--".to_string(),
        };
        p.print(pyf!("    {:>10} {:>22} {:>16} {:6d} {:5d} {:>10}",
                     lbl, taus4(a.taus).py_str(), w, a.in_joint_fuel, a.in_joint_gov, ho));
    }
    p.print("    Both legs WANT a cut over ~94% of the march, so the masked one is RIDING and");
    p.print("    reaching NOTHING -- not dormant, and nowhere near a stop.");

    p.print("\n  WHY: `max()` IS FLAT IN THE MASKED CLIP, so its whole column is (-1, 0, 0, 0):");
    let g = shared_gains(&t, flight, LO, HI, TMAX, sm, TAUS, false, 0.5, 1.2, 0.002, 0.20, 2)
        .expect("rung 72's shipped gains read");
    p.print(pyf!("    F_r = {}   R_f = {}   pair_FR = {}      <- both legs solve from the SCHEDULED fuel",
                 g.worst_f_r.expect("rows were sampled"), g.worst_r_f.expect("rows were sampled"),
                 g.worst_pair_fr.expect("rows were sampled")));
    p.print(pyf!("    masked leg's C and V gains: {}   (live gains stay >= {:.1e}, so this is not a dead instrument)",
                 g.worst_mask_leak.expect("rows were sampled"),
                 g.min_live_gain.expect("rows were sampled")));
    p.print("    RUNG 66's MIRROR: two loops on one VARIABLE gave pair = 1 EXACTLY -- maximally");
    p.print("    REDUNDANT. Two loops on one ACTUATOR give 0 EXACTLY -- maximally EXCLUSIVE.");

    p.print("\n  SO THE FOUR CELLS ARE THE FOUR RUNGS -- one plant, selected at RUN TIME:");
    let c = shared_cells(&t, flight, LO, HI, TMAX, sm, &clocks, 0.5, 1.2, 0.002, 0.20, 2)
        .expect("rung 72's shipped cells read");
    p.print(pyf!("    {:>10} {:>6} {:>9} {:>4} {:>6} {:>19} {:>10}",
                 "stator", "holds", "is", "n", "zeros", "constraint reading", "poly gap"));
    for (k, par) in [((false, Authority::Fuel), "rung 68"), ((false, Authority::Gov), "rung 70"),
                     ((true, Authority::Fuel), "rung 69"), ((true, Authority::Gov), "rung 71")] {
        let dd = &c.cells.iter().find(|(key, _)| *key == k).expect("all four cells are seen").1;
        let lbl = if k.0 { "incidence" } else { "phi" };
        let naive: i64 = if !k.0 { 2 } else { 1 };
        p.print(pyf!("    {:>10} {:>6} {:>9} {:4d} {:6d} {:19d} {:10.1e}",
                     lbl, auth_str(k.1), par, dd.n, dd.zeros[0], naive, dd.gap));
    }
    p.print("    Each cell's characteristic POLYNOMIAL is the parent rung's own times");
    p.print("    (lam + 1/tau_masked), rebuilt from the SHIPPED rung-68/69/70/71 readers -- two");
    p.print("    independent instruments reaching ONE polynomial. Compared coefficient by");
    p.print("    coefficient, not root by root: in the rung-68 cell the parent has a DOUBLE zero");
    p.print("    root and a root match would report 4.6e-07, which is sqrt(precision) and not a");
    p.print("    disagreement (the two readers' base points agree to 0.0 exactly).");
    p.print("    THE CONSTRAINT READING IS WRONG BY ONE ON BOTH ARMS; the ACTUATOR reading is");
    p.print("    right on the phi arm and wrong on the incidence one -- right by COINCIDENCE.");
    p.print("    Rung 71 s 11 offered those two answers. The answer is NEITHER:");
    p.print("        zeros = n_live - m_live,   n_live = the loops holding AUTHORITY = 3, always");
    p.print("    AND THE RANK CHANGES AT THE HAND-OVER with no state, no gain, no clock moving.");

    p.print("\n  THE ISOLATION INSTRUMENT -- AND THE CONFOUND IT NEARLY SHIPPED WITH:");
    let md = mask_discriminator(&t, flight, LO, HI, TMAX, sm,
                                &[(0.05, 0.05, 0.05, 0.05), (0.05, 0.08, 0.05, 0.05),
                                  (0.02, 0.09, 0.05, 0.05)],
                                false, 0.5, 1.2, 0.002, 0.20, 4)
        .expect("rung 72's shipped discriminator reads");
    p.print(pyf!("    {:>16} {:>17} {:>12}", "clocks (f,g)", "min-select pole", "SUM pole"));
    for a in &md.arms {
        let tag = if a.matched { "  <- MATCHED" } else { "" };
        let (f0, g0, _, _) = a.taus;
        p.print(pyf!("    {:>16} {:17.2e} {:12.2e}{}",
                     py_tuple(&[&f0, &g0]).py_str(), a.law_max.worst_pole.expect("a pole is read"),
                     a.law_sum.worst_pole.expect("a pole is read"), tag));
    }
    p.print("    At tau_f = tau_g the SUM law has (1,-1,0,0) as an EXACT eigenvector with");
    p.print("    eigenvalue -1/tau, so the test passes under BOTH laws and separates nothing.");
    p.print("    The matched row is GATED beside the result: a discriminator quoted from it alone");
    p.print("    is a discriminator that never tested anything.");

    p.print("\n  SO `n = 4` IS A MIRAGE, AND THAT CLOSES A SEAM BY REFUTING ITS PREMISE:");
    p.print("    n_live is 3 at every instant, so a shared actuator collapses (4, m) to (3, m)");
    p.print("    plus a free pole. Rung 71 s 11 named TWO routes to n = 4; this one cannot get");
    p.print("    there, and rung 69 s 11's (a fourth LP lever) stays OPEN.");
    p.print("    SCOPE: the composition law is DECLARED, not derived, and the whole triangular");
    p.print("    structure rests on BOTH inherited legs computing at the SCHEDULED fuel -- an");
    p.print("    applied-fuel-referenced leg gives F_r != 0 and the block form is gone (the");
    p.print("    sharpest next seam). The incidence/governor cell is 1 point at matched clocks");
    p.print("    and is read on a WIDE-CELL clock arm; Tt4_max is rung 67's imposed value; all");
    p.print("    four tau are swept march coordinates. See docs/rung72-spec.md.");
}

/// `print_applied_reference_table(flight)` — rung 73.
pub fn applied_reference_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE APPLIED REFERENCE (rung 73): the coupling rung 72 predicted is REAL, and it");
    p.print("lands in the WRONG COLUMN -- triangularity is a property of MIN-SELECT alone.");

    let sm = super::cascades::PHI / FLOOR - 1.0;
    let clocks = [(0.05, 0.05, 0.05, 0.05), (0.20, 0.01, 0.50, 0.05), (0.20, 0.005, 0.80, 0.05)];
    let lp = lp_map();
    let t = machine(build_applied_reference_cascade, d, &LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp, B, sm, Some(0.05))),
        ..LeverArm::default()
    });

    p.print("\n  THE SEAM NAMES ONE PLANT; THE LADDER ADMITS THREE, because every cap here is a");
    p.print("  SET-POINT solve -- a function of (nu, q, v) and NOT of the fuel it was asked");
    p.print("  about -- so d(required)/d(mf) is a BRANCH INDICATOR in {0, 1}, not a gradient:");
    p.print("    A  only the dormancy test moves  -- not a plant; the guard half, in both B and C");
    p.print("    B  req = g_own + (mf_app - cap)  -- THE PLANT: it reaches its own set point");
    p.print("    C  req = mf_app - cap            -- a P-controller with 2x DROOP. The instrument");
    p.print("  C is not broken, it is DEGENERATE FOR THIS LADDER: a leg that cannot reach its own");
    p.print("  floor measures a different object than every currency in rungs 46-72 did.");

    p.print("\n  THE HAND-OVER GOES LATE ON EVERY ARM -- and the SIGN is derivable:");
    let h = handover_law(&t, flight, LO, HI, TMAX, sm, &clocks, 0.5, 1.2, 0.005, 0.20);
    p.print(pyf!("    {:>10} {:>24} {:>8} {:>8} {:>10} {:>10} {:>11}",
                 "stator", "clocks (f,g,q,s)", "hand 72", "hand 73", "maxTt4 72", "maxTt4 73",
                 "d(min phi)"));
    for a in &h.arms {
        let (s, ap) = (&a.sched, &a.applied);
        p.print(pyf!("    {:>10} {:>24} {:8.3f} {:8.3f} {:10.2f} {:10.2f} {:11.1e}",
                     if a.inc { "incidence" } else { "phi" }, taus4(a.taus).py_str(),
                     s.first_gov.expect("the governor takes over"),
                     ap.first_gov.expect("the governor takes over"), s.max_tt4, ap.max_tt4,
                     a.d_phi));
    }
    p.print("    A masked governor referenced to the SCHEDULE races toward the clip the SCHEDULE");
    p.print("    would need -- credit for a cut the FUEL LEG already made. Referenced to the");
    p.print("    APPLIED fuel it integrates the cut still OWED. The CORRECT governor is SLOWER,");
    p.print("    takes the actuator later, and lets Tt4 run up to 71 K further. And phi does not");
    p.print("    move at all: the reference is decisive in Tt4 and invisible in phi.");
    p.print("    NO WINDUP, which was the feasibility gate: masked means gr > gf ~ req_f, so the");
    p.print(pyf!("    integrand is NEGATIVE and the masked leg winds DOWN to its floor (final gf = {:.1f}).",
                 h.arms[0].applied.final_g_fuel));

    p.print("\n  THE COUPLING IS REAL -- AND THE MASKED COLUMN IS STILL ZERO:");
    let g = applied_gains(&t, flight, LO, HI, TMAX, sm, clocks[0], false, 0.5, 1.2, 0.002, 0.20, 2)
        .expect("rung 73's shipped gains read");
    p.print(pyf!("    cross_masked = {:+.9f}   <- rung 72 s 11's `F_r != 0`, HELD", g.cross_masked[0]));
    p.print(pyf!("    self_masked  = {:+.9f}   <- and it reads its OWN state: an",
                 g.self_masked[g.self_masked.len() - 1]));
    p.print("                                     INTEGRATOR, not a lag");
    p.print(pyf!("    self_live    = {}          <- EXACT: the HOLDING leg's applied", g.self_live[0]));
    p.print("                                     reference IS the scheduled one");
    p.print(pyf!("    mask_leak    = {}          <- EXACT: and the masked leg still",
                 g.worst_mask_leak.expect("rows were sampled")));
    p.print("                                     reaches the plant through NOTHING");
    p.print(pyf!("    14 of the 16 entries of J(73) - J(72) are EXACTLY {}, at the SAME",
                 g.worst_delta_rest.expect("rows were sampled")));
    p.print("    base points. The two that move are the masked leg's own DIAGONAL and its cross-");
    p.print("    gain onto the AUTHORITATIVE axis -- both exactly 1/tau_masked. `F_r` is a ROW");
    p.print("    entry in the LIVE column; triangularity lives in the MASKED one.");
    p.print("    (The two exact ZEROS are gated as equality and the two ONES are not: `self_live`");
    p.print("    takes an explicit identity BRANCH, while `self_masked` is a central difference of");
    p.print("    a SUM. An exact zero survives a difference quotient; an exact one does not.)");

    p.print("\n  SO EVERY CELL IS ITS RUNG-72 PARENT PLUS ONE ZERO, AND det J DIES EVERYWHERE:");
    let c = applied_cells(&t, flight, LO, HI, TMAX, sm, &clocks, 0.5, 1.2, 0.002, 0.20, 2)
        .expect("rung 73's shipped cells read");
    p.print(pyf!("    {:>10} {:>6} {:>9} {:>4} {:>9} {:>9} {:>9} {:>10}",
                 "stator", "holds", "is", "n", "zeros 72", "zeros 73", "gap_hi", "|det|"));
    for k in [(false, Authority::Fuel), (false, Authority::Gov), (true, Authority::Fuel),
              (true, Authority::Gov)] {
        let dd = &c.cells.iter().find(|(key, _)| *key == k).expect("all four cells are seen").1;
        let r72 = c.rung72.iter().find(|(key, _)| *key == k).expect("four parents").1;
        p.print(pyf!("    {:>10} {:>6} {:>9} {:4d} {:9d} {:9d} {:9.1e} {:10.1e}",
                     if k.0 { "incidence" } else { "phi" }, auth_str(k.1), dd.parent, dd.n, r72,
                     dd.zeros[0], dd.gap_hi, dd.det));
    }
    p.print("    zeros = n_live - m_live + n_masked. Rung 72's free pole at -1/tau_masked moves");
    p.print("    to EXACTLY the origin -- min-select windup, in the spectrum, as the integrator");
    p.print("    itself and no longer as its lag. RUNG 71's CELL IS THE ONE THAT MATTERS: its");
    p.print("    det J = +5.9e+04 was the only live determinant in the whole family, and a change");
    p.print("    that adds no loop, no gain, no clock and no state KILLS it.");
    p.print("    `gap_hi` is the parent-polynomial comparison with the TRACE coefficient left");
    p.print("    out -- because `gap` and the null residual are ONE number, not two: the masked");
    p.print("    column's only non-zero entry is its own diagonal, and a3 IS minus the trace.");

    p.print("\n  THE INSTRUMENT HAD TO BE WEAKENED FIRST, AND IT COST FIVE ORDERS OF MAGNITUDE:");
    p.print("    rung 72's `_jac4` WRITES -1/tau on the diagonal, so a pole claim read off it");
    p.print("    would be the shipped instrument agreeing with itself (rung 67 gate 9, rung 71");
    p.print("    s 1.4, rung 72 s 4). Here F_f and R_r are MEASURED -- and a measured diagonal");
    p.print("    carries the float cancellation of `gf + req - gf`, so the parent identity lands");
    p.print("    at 5e-12 where rung 72's constructed one reached 7e-17. The right trade: the");
    p.print("    alternative was a headline the instrument would have written itself.");

    p.print("\n  THE TWO READINGS MOVE DISJOINT HALVES OF THE SAME MATRIX:");
    let d0 = ref_discriminator(&t, flight, LO, HI, TMAX, sm, clocks[0], false, 0.5, 1.2, 0.002,
                               0.20, 4)
        .expect("rung 73's shipped discriminator reads");
    p.print(pyf!("    {:>10} {:>15} {:>17} {:>14} {:>12}",
                 "", "root at origin", "root at -1/tau_m", "live diagonal", "zeros vs 72"));
    p.print(pyf!("    {:>10} {:>15} {:17.1e} {:14.1f} {:>12}",
                 "rung 72", "(3 cells have)", d0.worst_pole_72.expect("rows were sampled"), 0.0,
                 "--"));
    p.print(pyf!("    {:>10} {:15.1e} {:17.1e} {:14.1f} {:>12}",
                 "B applied", d0.worst_origin_b.expect("rows were sampled"),
                 d0.best_pole_b.expect("rows were sampled"), d0.live_diag_b[0],
                 ints(&d0.dzeros_b).py_str()));
    p.print(pyf!("    {:>10} {:15.1e} {:17.1e} {:14.1f} {:>12}",
                 "C literal", d0.best_origin_c.expect("rows were sampled"),
                 d0.worst_pole_c.expect("rows were sampled"), d0.live_diag_c[0],
                 ints(&d0.dzeros_c).py_str()));
    p.print("    B moves the POLE and keeps the parent; C keeps the pole and moves the PARENT.");
    p.print("    AND THE OBVIOUS DISCRIMINATOR IS THE ONE THAT FAILS: `is there a root at the");
    p.print("    origin` cannot separate B from rung 72, which already has zero roots in three of");
    p.print("    its four cells. Only the COUNT, differenced PER POINT, does it -- pooled across");
    p.print("    the two authority cells it compares one cell against the other and says nothing.");

    p.print("\n  AND THE BILL -- WHICH A SPECTRAL READING SAYS SHOULD BE EMPTY:");
    for inc in [false, true] {
        let b = applied_bill(&t, flight, LO, HI, TMAX, sm, clocks[0], inc, 0.5, 1.2, 0.005, 0.20);
        p.print(pyf!("    {:>10} stator | fuel leg's peak Tt4 debit {:+7.3f} K (rung 72) -> {:+8.3f} K ({:.0f}x)",
                     if inc { "incidence" } else { "phi" }, b.debit_sched, b.debit_applied,
                     b.debit_ratio.expect("the rung-72 debit is non-zero")));
        p.print(pyf!("    {:>10}        | its marginal phi credit {:+.3e} -> {:+.3e}   min phi {:.6f} (UNMOVED)",
                     "", b.phi_marginal_sched, b.phi_marginal_applied, b.phi_full_applied));
    }
    p.print("    RUNG 72 UNDER-REPORTED ITS OWN PEAK DEBIT BY TWO ORDERS OF MAGNITUDE, and the");
    p.print("    mechanism is structural: the fuel leg's authority window is EARLY, where the");
    p.print("    reference is the identity; the governor's is LATE, where it is not. The error");
    p.print("    exists ONLY at rung 72 -- with one fuel-side leg the two references coincide");
    p.print("    exactly, so rungs 46-71 are untouched. On the phi arm the leg's marginal phi");
    p.print("    credit even CHANGES SIGN: a phi limiter that debits phi, through a governor it");
    p.print("    never touches (rung 49's both-edges law, reaching a currency the leg owns).");

    p.print("\n  SCOPE: the reference is DECLARED, not derived, as rung 72's composition law is;");
    p.print("    reading C is read at B's base points and NEVER MARCHED, so nothing here says");
    p.print("    what a 2x-droop control would DO; the {0,1} branch indicator is a property of");
    p.print("    THIS ladder's set-point solvers, not of limiters in general; the Tt4 numbers are");
    p.print("    two model references at one imposed Tt4_max on one ramp -- the ORDERING is the");
    p.print("    claim; the incidence/governor cell is EMPTY at matched clocks and is read on two");
    p.print("    wider clock arms, one of them new; Tt4_max is rung 67's imposed value and all");
    p.print("    four tau are swept march coordinates. See docs/rung73-spec.md.");
}

/// `print_demand_coordinate_table(flight)` — rung 74.
pub fn demand_coordinate_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE DEMAND COORDINATE (rung 74): a coordinate on the lag is PURE BILL -- it cannot");
    p.print("touch the rank, and it moves the cut by the SCHEDULE'S OWN SLOPE.");

    let (phi_arrest, phi_both, phi_gov) = (0.80, 0.76, 0.70);
    let lp = lp_map();
    let rig = |phi_lim: f64, inc: bool| -> ScheduledStatorCore {
        machine(build_demand_coordinate_cascade, d, &rig_arm(&lp, phi_lim / FLOOR - 1.0, inc))
    };

    p.print("\n  THE ONE LINE THAT IS THE WHOLE RUNG. Every leg since rung 47 lags its CLIP; a");
    p.print("  fuel control lags its DEMAND -- the fuel it would allow -- and the lowest wins:");
    p.print("      CLIP    dg/ds = (required - g)/tau,  g >= 0,   mf = mf_sched - max(gf, gr)");
    p.print("      DEMAND  dw/ds = (cap      - w)/tau,  no floor, mf = min(mf_sched, wf, wr)");
    p.print("  Substituting w = mf_sched - g and cap = mf_sched - required:");
    p.print("      dg/ds = (required - g)/tau  +  d(mf_sched)/ds     <- a STATE-INDEPENDENT term");
    p.print("  so it appears in NO Jacobian. That is derivation, not measurement, and the panel");
    p.print("  below never reads it off a matrix this project wrote.");

    p.print("\n  THE SPECTRUM IS INVARIANT AND THE ENTRIES ARE NOT -- two Jacobians at ONE state,");
    p.print("  through two DIFFERENT sets of closures:");
    let g = demand_gains(&rig(phi_arrest, false), flight, LO, HI, TMAX, phi_arrest, TAUS, false, 0.5,
                         1.2, 0.002, 0.20, 4);
    p.print(pyf!("    interior points                          {:>12d}", g.n));
    p.print(pyf!("    characteristic polynomial, RELATIVE gap  {:12.2e}",
                 g.worst_poly_rel.expect("rows were sampled")));
    p.print(pyf!("    fuel<->non-fuel entries after a SIGN FLIP{:12.2e}",
                 g.worst_flip.expect("rows were sampled")));
    p.print(pyf!("    the OTHER off-diagonals                  {:12.1f}  <- EXACTLY 0",
                 g.worst_keep.expect("rows were sampled")));
    p.print(pyf!("    the four CYCLIC products                 {:12.2e}",
                 g.worst_pairs_gap.expect("rows were sampled")));
    p.print(pyf!("    entries that genuinely CHANGED SIGN      {:>12d}   (largest {:.0f})",
                 g.min_sign_changed.expect("rows were sampled"),
                 g.biggest_moved.expect("rows were sampled")));
    p.print("    The last row is the gate that matters: a port that silently did nothing would");
    p.print("    pass every other line (rung 73's `_reference` no-op returned a PERFECT");
    p.print("    confirmation having measured nothing).");
    p.print(pyf!("    AND THE MASKED COLUMN IS ZERO IN BOTH COORDINATES ({:.1f}):",
                 g.worst_mask_leak.expect("rows were sampled")));
    p.print("    `min()` is flat in the masked DEMAND exactly as `max()` was flat in the masked");
    p.print("    CLIP, so the block form survives and n_live is still <= 3. THE SEAM CLOSES BY");
    p.print("    REFUTATION -- the third rung running.");

    p.print("\n  THE FORCING, ISOLATED -- open loop, along ONE trajectory, because two plants that");
    p.print("  differ at all differ EVERYWHERE downstream:");
    let f = forcing_openloop(&rig(phi_both, false), flight, LO, HI, TMAX, phi_both, TAUS, false, 0.5,
                             1.2, 0.005, 0.20);
    p.print(pyf!("    schedule slope d(mf_sched)/ds = {:.6f} ,  tau = 0.05", f.slope));
    p.print(pyf!("    predicted steady offset  slope*tau = {:.4e}", f.predicted));
    p.print(pyf!("    MEASURED, late ramp                = {:.4e}   (ratio {:.4f}, worst point {:.1f}%)",
                 f.mean_delta_late.expect("the ramp has a late half"),
                 f.ratio_late.expect("the ramp has a late half"),
                 100.0 * f.worst_rel_late.expect("the ramp has a late half")));
    p.print(pyf!("    after the ramp stops   {:.3e} -> {:.2e}   <- a FORCING, not a gain",
                 f.delta_post_first.expect("the march settles"),
                 f.delta_post_last.expect("the march settles")));

    p.print("\n  SO THE LAG STOPS BREAKING THE REDLINE -- WHICH CORRECTS RUNG 47's CONCESSION:");
    p.print(pyf!("    {:>10} {:>8} {:>12} {:>10} {:>11} {:>10}",
                 "stator", "phi_lim", "clip maxTt4", "demand", "over(clip)", "over(dem)"));
    for inc in [false, true] {
        for phi in [phi_arrest, phi_both, phi_gov] {
            let dl = demand_law(&rig(phi, inc), flight, LO, HI, TMAX, 0.0, TAUS, &[phi], 0.5, 1.2,
                                0.005, 0.20);
            let a = dl.arms.iter().find(|x| x.inc == inc).expect("both arms are marched");
            let (CoordRead::Read(c), CoordRead::Read(m)) = (&a.clip, &a.demand) else {
                unreachable!("Python reads both coordinates' keys")
            };
            p.print(pyf!("    {:>10} {:8.2f} {:12.2f} {:10.2f} {:+11.2f} {:+10.2f}",
                         if inc { "incidence" } else { "phi" }, phi, c.max_tt4, m.max_tt4,
                         c.overshoot, m.overshoot));
        }
    }
    p.print("    A first-order lag tracks a SLOW target well and a RAMPING one poorly. In clip");
    p.print("    coordinates the target rides the SCHEDULE; in demand coordinates it rides the");
    p.print("    PLANT. Same leg, same clock -- and rung 47's *the cost of realism is that a");
    p.print("    lagged governor breaks the redline hold* is a property of the COORDINATE.");
    p.print("    AT phi_lim = 0.80 THE DEMAND PLANT DOES NOT ACCELERATE AT ALL (maxTt4 = Tt4_lo,");
    p.print("    min phi = 0.800000, both exact): the surge cap is AT the scheduled fuel from");
    p.print("    s = 0, so a leg that TRACKS it pins phi on the floor. The whole accel at that");
    p.print("    floor is powered by the clip coordinate's own tracking error.");

    p.print("\n  AND THE FLOOR CHANGES ADDRESS, WHICH DECIDES WHETHER THE PLANT EXISTS AT ALL:");
    let w = windup_law(&rig(phi_both, false), flight, LO, HI, TMAX, phi_both, TAUS, false, 0.5, 1.2,
                       0.005, 0.20);
    p.print(pyf!("    {:>16} {:>10} {:>7} {:>19}", "coordinate", "reference", "plant?",
                 "masked w/mf_sched"));
    for ((c, r), v) in [("demand", "sched"), ("demand", "applied"), ("demand-latched", "sched"),
                        ("demand-latched", "applied")].iter().zip(&w.cells) {
        let (exists, ratio) = match v {
            DemandCell::Present(x) => (true, pyf!("{:.4f}", x.max_masked_over_sched
                .expect("a present cell has a masked leg"))),
            DemandCell::Absent { .. } => (false, "--".to_string()),
        };
        p.print(pyf!("    {:>16} {:>10} {:>7} {:>19}", c, r, if exists { "yes" } else { "NO" },
                     ratio));
    }
    p.print("    The clip law floors the STATE (g >= 0); the demand law floors the COMPOSITION");
    p.print("    (mf <= mf_sched). Same admissible fuels, DIFFERENT plant -- a stop on the state");
    p.print("    resets the leg's memory and a stop on the output does not. So RUNG 73 s 0.2's");
    p.print("    *an applied-referenced leg is self-anti-winding under min-select -- a property");
    p.print("    of the COMPOSITION* is CORRECTED: it is a property of the coordinate's STOP.");
    p.print("    With the stop the masked leg parks at EXACTLY 1.0; without it there is no");
    p.print("    interior equilibrium and the march never starts. RUNG 52's max(0,.), inherited");
    p.print("    unexamined for 22 rungs, is this family's implicit ANTI-WINDUP DEVICE.");

    p.print("\n  RUNG 69, FROM THE OTHER SIDE: *a loop's COORDINATE decides whether it adds a ZERO");
    p.print("    or a RANK* -- that is the CONSTRAINT's coordinate. This is the STATE's, and it");
    p.print("    cannot touch the rank at all. A constraint's coordinate is GEOMETRY; a state's");
    p.print("    is BOOKKEEPING, and only one of them is in the Jacobian.");

    p.print("\n  SCOPE: phi_lim had to be SWEPT to get a comparable trajectory (the inherited");
    p.print("    floor arrests the demand plant) -- it is an imposed coordinate since rungs");
    p.print("    36/49, but this is the first rung whose ARMS depend on moving it; the unfloored");
    p.print("    cap is a set-point solve in a regime the family has never exercised (measured");
    p.print("    available at 341/341 anchor points, and it asserts rather than falling back);");
    p.print("    the demand plant is SCHED-referenced only, since demand x applied has no plant;");
    p.print("    the gains are read on the CLIP plant's states, because `_shared_rig` gives every");
    p.print("    leg one margin and a lowered floor makes the valve dormant; Tt4_max is rung 67's");
    p.print("    imposed value and all four tau are swept march coordinates.");
    p.print("    See docs/rung74-spec.md.");
}

/// `print_anti_windup_table(flight)` — rung 75.
pub fn anti_windup_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE DECLARED ANTI-WINDUP DEVICE (rung 75): decisive on the SPECTRUM, inert on the");
    p.print("RANK -- the exact INVERSE of rung 74's coordinate.");

    let (phi_jac, phi_both) = (0.80, 0.76);
    let lp = lp_map();
    let rig = |phi_lim: f64| -> ScheduledStatorCore {
        machine(build_anti_windup_cascade, d, &rig_arm(&lp, phi_lim / FLOOR - 1.0, false))
    };

    p.print("\n  THE ONE LINE THAT IS THE WHOLE RUNG. Rung 74 removed the state floor and found");
    p.print("  the masked leg had NOTHING in its path. This puts a RATE there instead of a wall:");
    p.print("      none   dw/ds = (target - w)/tau                          <- RUNG 74");
    p.print("      track  dw/ds = (target - w)/tau + (mf_app - w)/tau_t     <- back-calculation");
    p.print("  The added term is STATE-DEPENDENT, so unlike rung 74's forcing it IS in the");
    p.print("  Jacobian. That is derivation; the panel measures what it does there.");

    let t = rig(phi_jac);
    // `windup_gains(..., tau_ts=(0.05, 0.0125), refs=("applied", "sched"))`: ds = 0.005,
    // every = 8.
    let g = windup_gains(&t, flight, LO, HI, TMAX, phi_jac, TAUS, &[0.05, 0.0125],
                         &["applied", "sched"], false, 0.5, 1.2, 0.005, 0.20, 8);
    // `g["cells"].get("%s|%s" % (ref, tt))`, kept only where it carries a non-zero `n`.
    let cell = |rf: &str, tt: f64| g.cells.iter()
        .find(|((r, t_), _)| *r == rf && *t_ == tt)
        .and_then(|(_, c)| match c {
            WindupCell::Read(c) if c.n != 0 => Some(c),
            _ => None,
        });
    p.print("\n  THE MASKED LEG'S OWN DIAGONAL -- the one rung 73's APPLIED reference cancelled");
    p.print("  to exactly zero, which is why det J has been dead for two rungs:");
    p.print(pct!("    %-10s %-8s  %-14s %-14s  %-10s %s",
                 "reference", "tau_t", "diag (none)", "diag (track)", "det J", "zeros"));
    for rf in ["applied", "sched"] {
        for tt in [0.05, 0.0125] {
            let Some(c) = cell(rf, tt) else { continue };
            let det = if c.det0_alive < 1e-9 {
                pct!("dead->%.0f", c.det_alive.abs())
            } else {
                "alive".to_string()
            };
            p.print(pct!("    %-10s %-8.4f  %-14.4f %-14.4f  %-10s %d -> %d",
                         rf, tt, c.masked_diag0.0, c.masked_diag.0, det, c.zeros0[0], c.zeros[0]));
        }
    }
    p.print("  Under APPLIED the pole LEAVES THE ORIGIN and det J REVIVES; under SCHED the");
    p.print("  diagonal was already -1/tau, so the device merely ADDS THE RATES and nothing was");
    p.print("  ever dead. ONE mechanism, two faces.");

    let ratio = |rf: &str| g.ratios.iter().find(|r| r.ref_law == rf);
    if let (Some(_), Some(_)) = (ratio("applied"), ratio("sched")) {
        p.print("\n  AND det J FOLLOWS THE DIAGONAL EXACTLY, which is what block-triangularity");
        p.print("  MEANS (det J = masked diagonal x det of rung 71's untouched 3x3 block):");
        p.print(pct!("    %-10s %-22s %s", "reference", "diagonal ratio", "det J ratio"));
        for rf in ["applied", "sched"] {
            let r = ratio(rf).expect("checked above");
            p.print(pct!("    %-10s %-22.6f %.6f", rf, r.diag.0, r.det.0));
        }
        p.print("  4.000000 where the diagonal is -1/tau_t; 2.500000 where it is the two rates");
        p.print("  ADDED (100/40) -- rung 66's identity in a fifth shape, now on a DEVICE.");
    }

    p.print("\n  AND THE RANK DOES NOT MOVE. The term sits in the masked leg's ROW (it reads the");
    p.print("  authoritative leg through mf_app); the masked COLUMN stays zero because min() is");
    p.print("  still flat in what the masked leg holds:");
    for rf in ["applied", "sched"] {
        if let Some(c) = cell(rf, 0.05) {
            p.print(pct!("    %-10s mask_leak %.1e (none) -> %.1e (track)   track_leak %.1e",
                         rf, c.mask_leak0, c.mask_leak, c.track_leak));
        }
    }
    p.print("  So n_live is STILL <= 3 -- the FOURTH running -- and the device is the ZERO");
    p.print("  FUNCTION on whichever leg holds the actuator (track_leak = 0.0, exactly), which");
    p.print("  is what leaves rung 72's `one plant IS rungs 68-71 by AUTHORITY` untouched.");

    // `contraction_law(..., tau_ts=(0.4, 0.2, 0.1, 0.05))`: res0 = 2.898e-3, tol = 1e-12,
    // ic_cap = 400, ds = 0.005.
    let c = contraction_law(&rig(phi_both), flight, LO, HI, TMAX, phi_both, TAUS,
                            &[0.4, 0.2, 0.1, 0.05], 2.898e-3, 1e-12, 400, false, 0.5, 1.2, 0.005,
                            0.20);
    p.print("\n  AND RUNG 74's OWN RESIDUAL, EXPLAINED. Its joint IC sweep is a FIXED POINT");
    p.print("  ITERATION; the device changes its map's slope from 1 to sigma = tau_t/(tau+tau_t),");
    p.print("  so it converges in ceil(ln(tol/res0)/ln sigma) -- with res0 RUNG 74's OWN reported");
    p.print("  2.898e-3 and tol the inherited 1e-12. Zero fitted constants:");
    p.print(pct!("    %-9s %-10s %-12s %s", "tau_t", "sigma", "predicted", "measured"));
    for x in &c.rows {
        let measured = match x.measured {
            Some(m) => m.py_str(),
            None => "-".to_string(),
        };
        p.print(pct!("    %-9.4f %-10.6f %-12d %s", x.tau_t, x.sigma, x.predicted, measured));
    }
    p.print("  So rung 74's `no interior equilibrium` VERDICT stands (tau_t -> inf gives sigma =");
    p.print("  1 and w* -> inf) and its NUMBER was never a solver failing -- it was a contraction");
    p.print("  with ratio EXACTLY ONE. The exists/does-not-exist boundary is the 60-iteration cap");
    p.print("  cutting a geometric sequence, and the cap is raised in a READER, never in a plant.");

    // `windup_bill(..., tau_ts=(0.0125, 0.05, 0.0625, 0.075, 0.1))`: ref = "applied", ds = 0.005.
    let b = windup_bill(&rig(phi_both), flight, LO, HI, TMAX, phi_both, TAUS,
                        &[0.0125, 0.05, 0.0625, 0.075, 0.1], "applied", false, 0.5, 1.2, 0.005,
                        0.20);
    let hand = |h: Option<f64>| match h {
        Some(h) => pct!("%.3f", h),
        None => "-".to_string(),
    };
    p.print("\n  THE BILL -- and rung 47's headline concession, THIRD LAYER. Rung 47: a lagged");
    p.print("  governor breaks the redline hold. Rung 74: that is the COORDINATE. Rung 75: inside");
    p.print("  the demand coordinate it is a THRESHOLD ON tau_t:");
    p.print(pct!("    %-9s %-10s %-11s %-11s %s",
                 "tau_t", "tau_t/tau", "max Tt4", "over max", "hand-over"));
    for x in &b.rows {
        p.print(pct!("    %-9.5f %-10.3f %-11.2f %+11.2f %s",
                     x.tau_t, x.ratio, x.max_tt4, x.over, hand(x.handover)));
    }
    let a = &b.accident;
    p.print(pct!("    %-9s %-10s %-11.2f %+11.2f %s",
                 "ACCIDENT", "-", a.max_tt4, a.over, hand(a.handover)));
    p.print(pct!("  The threshold is bracketed at tau_t/tau_f in (%.2f, %.2f] -- a tracking clock no",
                 b.ratio_holds.expect("a clock holds"), b.ratio_breaks.expect("a clock breaks")));
    p.print("  slower than its own leg's. Rung 54's shape, now on a CLOCK. AND THE MAGNITUDE IS");
    p.print("  AGAINST THE ACCIDENT, not across the sweep: tau_t moves the peak 13 K and the");
    p.print("  hand-over two grid cells; the declared device beats rung 52's inherited stop by");
    p.print("  ~160 K and hands over ~0.36 earlier. The whole sweep lives inside a tenth of it.");

    p.print("\n  WHAT THIS RUNG IS. Rung 74 moved the BILL and not the spectrum; this moves the");
    p.print("  SPECTRUM and not the rank. Together they separate three things this family has");
    p.print("  been running together since rung 68: POLE LOCATION, `zeros`, and LIVE-LOOP COUNT.");
    p.print("  Rung 71 found that zeros counts GRADIENTS, not live loops; this is its converse --");
    p.print("  A POLE IS NOT A LOOP EITHER.");

    p.print("\n  SCOPE: tau_t is a NEW CONSTANT, the first new clock since rung 65, and it is not");
    p.print("    derived from anything shipped -- every finding above is a property of the SWEEP");
    p.print("    or a threshold on it, never of a chosen value; its fast end is GRID-LIMITED at");
    p.print("    tau_t >= 0.00625 by the inherited RK4 floor, so perfect tracking is unreachable");
    p.print("    here and is not claimed; `clip x track` is REFUSED (rung 52's max(0,.) is still");
    p.print("    there, so the cell would run two devices at once), which means the accident is");
    p.print("    measured only against the demand coordinate's own latch; the control row is ONE");
    p.print("    point (the accel arms almost immediately); the hand-over's tau_t-dependence is");
    p.print("    two grid cells, at the resolution limit; the Jacobians are read at the INHERITED");
    p.print("    floor and the trajectories at the lowered one, so no cell carries both; Tt4_max");
    p.print("    is rung 67's imposed value and all four tau are swept march coordinates.");
    p.print("    See docs/rung75-spec.md.");
}

/// `print_sensed_cap_table(flight)` — rung 76.
pub fn sensed_cap_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE FUEL-DEPENDENT CAP (rung 76): a device in a leg's LAW reaches only the MASKED");
    p.print("leg; a device in the PLANT THE LEGS READ reaches only the AUTHORITATIVE one.");

    let margin = 0.10;
    let (phi_jac, phi_both) = (0.80, 0.76);
    let lp = lp_map();
    let rig = |phi_lim: f64| -> ScheduledStatorCore {
        machine(build_sensed_cap_cascade, d, &rig_arm(&lp, phi_lim / FLOOR - 1.0, false))
    };

    p.print("\n  THE TWO READINGS OF ONE CAP. Rung 48's leg states its law as an inequality ON");
    p.print("  THE FUEL, and a real limiter EVALUATES the right-hand side from the pt3 it SENSES:");
    p.print("      solve   cap = w*  with  w* = (1+m)*kappa(n_H(w*))*pt3(w*)   <- rungs 48-75");
    p.print("      sensed  cap(w) = (1+m)*kappa(n_H(w))*pt3(w)  at w = mf_app  <- AS WRITTEN");
    p.print("  The set-point solve was a MODELLING CHOICE, not the schedule. This rung adds NO");
    p.print("  constant at all -- `margin` is rung 48's own, and kappa is still derived.");

    let t = rig(phi_jac);
    // `cap_gains(..., phi_lim=PHI_JAC, margin=MARGIN)`: tau_t = 0.05, refs = ("sched", "applied"),
    // laws = ("none", "track"), ds = 0.005, every = 8.
    let g = cap_gains(&t, flight, LO, HI, TMAX, phi_jac, margin, TAUS, 0.05, &["sched", "applied"],
                      &["none", "track"], false, 0.5, 1.2, 0.005, 0.20, 8);
    // `live = {k: c for k, c in g["cells"].items() if c.get("n")}`, iterated as `sorted(live)`.
    let mut live: Vec<(&str, &crate::sensed_cap::CapCellRead)> = g.cells.iter()
        .filter_map(|(k, c)| match c {
            CapCell::Read(c) if c.n != 0 => Some((k.as_str(), &**c)),
            _ => None,
        })
        .collect();
    live.sort_by(|a, b| a.0.cmp(b.0));

    p.print("\n  THE AUTHORITATIVE DIAGONAL -- the one entry rungs 73, 74 AND 75 each report as");
    p.print("  *moved 0.0 relative*, because min-select masks a LAW but cannot mask a PLANT:");
    p.print(pct!("    %-22s %-8s  %-11s %-11s %-9s %s",
                 "cell", "c", "solve", "sensed", "moved", "vs (c-1)/tau"));
    for (k, c) in &live {
        p.print(pct!("    %-22s %-8.4f  %-11.4f %-11.4f %-9.4f %.1e",
                     k, c.c.0, c.auth_diag0.0, c.auth_diag.0, c.auth_moved, c.auth_err));
    }
    p.print("  `c = d(cap)/dw` is MEASURED, not derived -- a bracketing root-finder converges on");
    p.print("  a sign change whether or not G is monotone, so the shipped solve working buys");
    p.print("  *a root exists*, never c < 1.");

    p.print("\n  AND THE MASKED LEG IS UNTOUCHED, so the RANK does not move -- a FIFTH running:");
    p.print(pct!("    %-22s %-14s %-12s %s", "cell", "masked moved", "mask_leak", "zeros"));
    for (k, c) in &live {
        p.print(pct!("    %-22s %-14.1e %-12.1e %d -> %d",
                     k, c.masked_moved, c.mask_leak, c.zeros0.0, c.zeros.0));
    }
    p.print("  min() is FLAT in what the masked leg holds, so d(mf_app)/dw_masked = 0. The same");
    p.print("  flatness that gives rungs 72-76 their triangularity is what confines THIS rung's");
    p.print("  device to the authoritative leg -- and rung 75's to the masked one.");

    // `solve_gain(..., phi_lim=PHI_JAC, margin=MARGIN)`: ref = "sched", ds = 0.005, dq = 1e-5,
    // every = 8.
    let sg = solve_gain(&t, flight, LO, HI, TMAX, phi_jac, margin, TAUS, "sched", false, 0.5, 1.2,
                        0.005, 0.20, 1e-5, 8);
    let gain = sg.gain.expect("rows were sampled");
    p.print("\n  AND THE SET-POINT SOLVE WAS NEVER A RELOCATION OF THE CAP -- IT IS A GAIN ON IT.");
    p.print("  Differentiating the fixed point cap = cap_sensed(cap, q) in one line gives");
    p.print("  d(cap_solve)/dq = (d(cap_sensed)/dq)/(1 - c), so a limiter written as a SOLVE is a");
    p.print("  STIFFER limiter than the schedule it claims to implement:");
    p.print(pct!("    the two laws agree at the solve's own answer to   %.1e  (machine zero)",
                 sg.fixed_point.expect("rows were sampled")));
    p.print(pct!("    measured amplification                            %.5f .. %.5f",
                 gain.0, gain.1));
    p.print(pct!("    against 1/(1 - c), per point                      %.1e",
                 sg.gain_err.expect("rows were sampled")));
    p.print("  This was NOT predicted. It was written down to explain why det J's ratio misses");
    p.print("  1-c by 0.7%, and it is the strongest number in the rung.");

    // `cap_bill(..., phi_lim=PHI_BOTH, margin=MARGIN)`: tau_t = 0.05, ref = "sched",
    // law = "none", ds = 0.005, tail = 3.0.
    let b = cap_bill(&rig(phi_both), flight, LO, HI, TMAX, phi_both, margin, TAUS, 0.05, "sched",
                     "none", false, 0.5, 1.2, 0.005, 0.20, 3.0);
    p.print("\n  THE BILL. During the ramp mf_app < cap_solve, so the sensed cap is the LOWER one");
    p.print("  and the leg cuts HARDER -- rung 48's solve has been granting the engine the fuel");
    p.print("  it would be self-consistent WITH, which is more than its own schedule allows:");
    p.print(pct!("    peak Tt4    %.2f -> %.2f K   (%+.2f)",
                 b.max_tt4.0, b.max_tt4.1, b.max_tt4.1 - b.max_tt4.0));
    p.print(pct!("    min phi_lp  %.6f -> %.6f  (%+.2e)",
                 b.min_phi.0, b.min_phi.1, b.min_phi.1 - b.min_phi.0));
    p.print(pct!("    fuel burnt  %+.4f%%   cuts harder over the WHOLE ramp: %s",
                 100.0 * (b.fuel_int.1 / b.fuel_int.0 - 1.0), b.cuts_harder));

    p.print("\n  SCOPE: the knob reaches the ACCEL branch of _cap_fuel and NOTHING else -- the phi");
    p.print("    leg and the governor are floors on STATES and have no sensed form, so the");
    p.print("    governor's whole row is bit-identical in every cell (measured 0.0). `margin` is");
    p.print("    imposed (rung 48's own) and every structural entry is checked at three margins;");
    p.print("    the accel leg had never been armed in this family, so NOTHING here is");
    p.print("    differenced against a quoted rung-73/74/75 number -- the 2x2 is re-measured on");
    p.print("    one rig and rung 75's headline is REPRODUCED rather than cited. The masked-leg");
    p.print("    cell is UNREACHED and rung 48 says why: its leg is feedforward and fires early,");
    p.print("    the governor is feedback and fires late, so the leg that binds the cap is also");
    p.print("    the leg that holds the actuator (24/24 combinations). det J's ratio is 1-c only");
    p.print("    to 0.7%, and section 3 names the mechanism rather than tightening a tolerance.");
    p.print("    See docs/rung76-spec.md.");
}

/// `print_stiffness_ledger_table(flight)` — rung 77.
pub fn stiffness_ledger_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE STIFFNESS LEDGER (rung 77): a set-point solve's sensitivity is a FORCING OVER");
    p.print("A SLOPE, and 1/(1-c) is the SLOPE HALF of one leg.");

    let (margin, phi) = (0.10, 0.80);
    let lp = lp_map();
    let sm = phi / FLOOR - 1.0;
    let t = machine(build_stiffness_ledger_cascade, d, &rig_arm(&lp, sm, false));
    // `t._lag_coord, t._ref_law = "demand", "sched"` and
    // `t._windup_law, t._tau_t, t._cap_law = "none", None, "solve"` — plain assignments.
    t.fuel.inner.lag_coord.set("demand");
    t.fuel.inner.ref_law.set("sched");
    t.fuel.inner.windup_law.set("none");
    t.fuel.inner.tau_t.set(None);
    t.fuel.inner.cap_law.set("solve");

    // Every reader: taus = (0.05,)*4, inc = False, ds = 0.005, v_max = 0.20, every = 8; plus
    // dq = 1e-5 (set_point_gains) and spread = 0.10 (singular_limit).
    let s1 = leg_slopes(&t, flight, LO, HI, TMAX, phi, margin, TAUS, false, 0.5, 1.2, 0.005, 0.20,
                        8);
    let s2 = set_point_gains(&t, flight, LO, HI, TMAX, phi, margin, TAUS, false, 0.5, 1.2, 0.005,
                             0.20, 1e-5, 8);
    let s3 = singular_limit(&t, flight, LO, HI, TMAX, phi, margin, TAUS, false, 0.5, 1.2, 0.005,
                            0.20, 0.10, 8);

    p.print("\n  THE THREE RESIDUALS, and only ONE of them is dimensionless:");
    p.print("    accel (48)  G_a(w) = w - cap(w)            G_a' = 1 - c       [-]");
    p.print("    gov   (46)  G_g(w) = Tt4(w) - Tt4_max      G_g' = dTt4/dw     [K per kg/s]");
    p.print("    phi   (49)  G_s(w) = phi_lim - phi_lp(w)   G_s' = -dphi/dw    [phi per kg/s]");
    p.print("  `Tt4_max` and `phi_lim` are CONSTANTS -- no `1` to subtract from, so no `c`.");

    p.print(pct!("\n  %-6s %22s %18s %10s %16s", "leg", "G_w (raw)", "normalised", "1/|n|",
                 "dw*/dq [kg/s]"));
    p.print(format!("  {}", "-".repeat(76)));
    let (gw, nm, st, gn) = (s1.gw.expect("rows were sampled"), s1.norm.expect("rows were sampled"),
                            s1.stiff.expect("rows were sampled"), s2.gain.expect("rows were sampled"));
    for k in Leg::ORDER {
        let i = k as usize;
        p.print(pct!("  %-6s %10.4e..%10.4e %8.5f..%8.5f %5.3f..%5.3f %+8.2e..%+8.2e",
                     k.name(), gw[i].0, gw[i].1, nm[i].0, nm[i].1, st[i].0, st[i].1, gn[i].0,
                     gn[i].1));
    }

    p.print("\n  THE ACCEL COLUMN IS RUNG 76 SECTION 3's GAIN, digit for digit (1.22799..1.24573),");
    p.print("    read here by an INDEPENDENT route -- 1/G_a' from one residual, never solve_gain.");
    p.print(pct!("    Instrument: |(1 - G_a') - c| = %.2e against rung 76's own _c_at.",
                 s1.c_err.expect("rows were sampled")));
    p.print(pct!("  D1 `dw*/dq = -G_q/G_w` HOLDS per leg: worst relative %.2e (a differencing",
                 s2.ift_err.expect("rows were sampled")));
    p.print("    floor -- dq swept 3 decades gives a textbook central-difference V).");

    let order = s2.order.expect("rows were sampled");
    let names: Vec<&str> = order.iter().map(|l| l.name()).collect();
    let items: Vec<&dyn PyFormat> = names.iter().map(|x| x as &dyn PyFormat).collect();
    p.print(pct!("\n  THE ORDER, in the currency all three legs share: %s", py_tuple(&items)));
    p.print("    The phi leg is 45-64x the governor and 112-268x the accel leg -- by a wide");
    p.print("    margin the stiffest thing in this control system.");
    p.print("  AND THE VALVE'S SIGN SPLITS: accel and gov NEGATIVE, phi POSITIVE. Opening the");
    p.print("    bleed valve tightens both fuel-side caps and loosens the one that watches phi,");
    p.print("    so the lever that buys the phi leg its protection debits both others (rung 61's");
    p.print("    `buys the COORDINATE, not the BILL`, with a sign on it).");

    p.print("\n  THE SINGULAR LIMIT -- dw*/dq diverges iff G_w -> 0, and there are TWO routes:");
    p.print("    accel   c -> 1, the set point chases its OWN actuator     NOT reachable here");
    p.print("    phi     another lever PINS the variable it watches        REACHED (rung 64)");
    p.print("    gov     neither: nothing here pins Tt4 at a fixed fuel    NOT reachable");
    p.print("  RUNG 64 DERIVED its valve degeneracy and said so (`DERIVED, not measured`).");
    p.print(pct!("    MEASURED: |G_s'| open %.4f -> closed %.3e, and phi_lp is phi_lim to %.1e",
                 s3.phi_open.expect("rows were sampled"), s3.phi_closed.expect("rows were sampled"),
                 s3.phi_off.expect("rows were sampled")));
    p.print(pct!("    across +-10%% fuel. CONTROL: the governor read the same way moves %.1e",
                 s3.gov_rel.expect("rows were sampled")));
    p.print("    relative, not zero but 2% against 100% -- without it this is an artifact.");

    p.print("\n  SCOPE: this rung measures SOLVERS, not physics -- G_w is a property of how a");
    p.print("    limiter is WRITTEN, which is what rung 76 s 8 asked for. `dw*/dq` is read in");
    p.print("    ONE direction (the valve), so `stiffest leg` means stiffest TO THE VALVE, and");
    p.print("    each slope is read at its OWN leg's set point. Over 24 cells the raw ordering");
    p.print("    is NOT invariant (3 invert, all at margin 0.40) -- every one a DORMANT accel");
    p.print("    leg, and under rung 76 s 1.3's own switch guard the phi leg is top 24/24.");
    p.print("    `c` never exceeds 0.2234, which BOUNDS rung 76's `c -> 1` seam before it is");
    p.print("    built. No knob, no state, no constant, no plant code. See docs/rung77-spec.md.");
}
