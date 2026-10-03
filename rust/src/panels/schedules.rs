//! Rungs 57–63's panels — the stator schedule on the transient plant, its composites, and the
//! bleed beside it (slice AP, second half). Rung 59 prints no panel of its own.
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line for
//! line. **The march step is the trap here** (`tests/rung58.rs`'s header names it): each Python
//! reader carries its OWN `ds` default, so each call site picks its ramp off the SIGNATURE —
//! [`Ramp::new`] (`ds = 0.01`) for rung 57's three readers and rung 62's four, [`Ramp::fine`]
//! (`ds = 0.005`) for rung 58's composite / engagement and rung 63's retiming / dichotomy, and
//! `Ramp::new` again for rung 60, whose panel passes `ds=DS=0.01` over a 0.005 default. Python's
//! `neighbour={}` is `None` (`_isolating` reads `dict(neighbour or {})`).

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use crate::engine::FlightCondition;
use crate::fuel_transient::{Floor, SurgeLimiter};
use crate::pyf;
use crate::pyfmt::{Printer, PyFormat};
use crate::stator_bleed::{StatorBleedCore, Target};
use crate::stator_transient::{
    IncidenceLimiter, LadderAxis, Ramp, Regime, ScheduledStatorCore, ScheduledStatorTransient,
    StatorArm, StatorLeg, StatorSchedule,
};
use crate::two_spool::{Spool, TwoSpoolEngine};

const LO: f64 = 1000.0;
const HI: f64 = 1400.0;

/// `ScheduledStatorTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)`.
fn st(design: &TwoSpoolEngine, flight: &FlightCondition, arm: StatorArm) -> ScheduledStatorCore {
    match ScheduledStatorTransient::new(design.clone(), *flight, 1.0, Some(lp_map()),
                                        Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("both maps are passed"),
    }
}

/// `ScheduledBleedTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)`.
fn bt(design: &TwoSpoolEngine, flight: &FlightCondition, arm: &LeverArm) -> ScheduledStatorCore {
    match build_scheduled_bleed(design.clone(), *flight, 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                                arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("both maps are passed"),
    }
}

/// Python's `max(list)` / `min(list)`: the first value, replaced only on a strict compare.
fn py_max(xs: &[f64]) -> f64 {
    xs[1..].iter().fold(xs[0], |a, &x| if x > a { x } else { a })
}

fn py_min(xs: &[f64]) -> f64 {
    xs[1..].iter().fold(xs[0], |a, &x| if x < a { x } else { a })
}

/// Python's `regime` string (`engine.py`'s `floor_composite`).
fn regime_str(r: Regime) -> &'static str {
    match r {
        Regime::BothPinned => "both_pinned",
        Regime::ArmedClears => "armed_clears",
        Regime::Mixed => "mixed",
    }
}

/// `print_stator_schedule_table(flight)` — rung 57.
pub fn stator_schedule_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE STATOR SCHEDULE ON THE TRANSIENT PLANT (rung 57): the first lever that moves the");
    p.print("surge FLOOR *during* an accel. Rungs 46-52's levers all move the POINT, and every one");
    p.print("of them was credited by a CLOCK. This one is credited by a MAP:");
    p.print("   erosion = 1 - (net credit)/(pointwise credit)   <- the share the WORK channel eats");

    let v = 0.20;
    let design = design13(d);
    let mk = |arm: StatorArm| st(&design, flight, arm);

    let rs = [0.1, 0.25, 0.5, 1.0, 2.0];
    let rows: Vec<_> = rs.iter()
        .map(|&r| mk(StatorArm::constant(v, 0.0)).stator_credit(flight, &Ramp::new(LO, HI, r),
                                                                 Spool::Lp))
        .collect();

    p.print(pyf!("\n  A CONSTANT setting v = {:.2f} on the LP spool, over a 20x range of ramp rate r:", v));
    p.print(pyf!("    {:>5} {:>10} {:>9} {:>9} {:>9}", "r", "bare M_i", "credit", "credit/v",
                 "erosion"));
    for (r, q) in rs.iter().zip(&rows) {
        p.print(pyf!("    {:5.2f} {:+10.5f} {:9.5f} {:9.5f} {:9.4f}",
                     r, q.bare, q.credit, q.credit / v, q.erosion));
    }
    let bare: Vec<f64> = rows.iter().map(|q| q.bare).collect();
    let er: Vec<f64> = rows.iter().map(|q| q.erosion).collect();
    let cf = 1.0 - rows[0].closed_form;
    let err: Vec<f64> = er.iter().map(|e| (e - cf).abs()).collect();
    p.print(pyf!("    the MARGIN swings {:.1f} %; the EROSION moves {:.2f} points.",
                 100.0 * (py_max(&bare) - py_min(&bare)) / py_min(&bare),
                 100.0 * (py_max(&er) - py_min(&er))));
    p.print(pyf!("    rung 53's design-point closed form 1 - 1/(2+l) = {:.4f}  (max error {:.1f} %)",
                 cf, 100.0 * py_max(&err) / cf));
    p.print("    => the credit is a MAP property. A wall-moving lever has NO CLOCK.");

    let n_lo = mk(StatorArm::default()).fuel.inner.equilibrium(flight, LO).nu_lp;
    let sc = StatorSchedule::new(v, n_lo);
    let dec: Vec<_> = rs.iter()
        .map(|&r| mk(StatorArm::scheduled_lp(sc))
            .credit_decomposition(flight, &Ramp::new(LO, HI, r), Spool::Lp))
        .collect();
    p.print("\n  And a state-fed SCHEDULE v(n) (closed at low speed, 0 at design speed n=1):");
    p.print(pyf!("    {:>5} {:>9} {:>11} {:>10} {:>12} {:>10}",
                 "r", "FULL", "START-only", "RAMP-only", "share START", "FULL/RAMP"));
    for (r, q) in rs.iter().zip(&dec) {
        p.print(pyf!("    {:5.2f} {:+9.5f} {:+11.5f} {:+10.5f} {:12.3f} {:10.3f}",
                     r, q.full, q.start, q.ramp, q.share_start, q.self_cancel));
    }
    p.print(pyf!("    nu0_L rises {:.4f} -> {:.4f} when the stators close (rung 53: paid in SHAFT SPEED)",
                 dec[0].nu0_bare, dec[0].nu0_armed));
    p.print("    => the schedule READS that higher speed and opens back up: it SELF-CANCELS.");

    let a = mk(StatorArm::default())
        .arrow_toggle(flight, &Ramp::new(LO, HI, 0.5), v, Spool::Lp, None);
    p.print("\n  CORRECTING rung 53's P5, at ONE fixed transient state:");
    p.print(pyf!("    v_LP = {:.2f}  ->  d_phi_LP {:+.4e}   d_phi_HP {:+.4e}   d_Tt25 {:+.2f} K",
                 v, a.d_phi_lp, a.d_phi_hp, a.d_tt25));
    p.print("    rung 53 measured d_phi_HP == +0.000e+00 EXACTLY on the STEADY cascade and called");
    p.print("    the LP stator 'a pure-LP lever, bit-for-bit'. It is not, once the shaft speeds are");
    p.print("    STATES: the steady balance re-solved n_H and absorbed the Tt25 shift; rung 40 took");
    p.print("    that balance away. The break survives a FLAT-eta island, so it is NOT rung 53's");
    p.print("    eta-mediated arrow -- it is the ENERGY channel, and d_Tt25 names it.");
    p.print("\n  SCOPE: LP-side (the HP schedule reads shaft speed -- Tt25 is an output of the root");
    p.print("  it must be armed before); eta_c_at is stator-inert; no head-to-head against the");
    p.print("  fuel-side limiters (no common currency). See docs/rung57-spec.md § Concessions.");
}

/// `print_composite_minselect_table(flight)` — rung 58.
pub fn composite_minselect_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE COMPOSITE MIN-SELECT (rung 58): rung 57's lever and ONE fuel-side leg on ONE");
    p.print("plant. Four cells and their MIXED SECOND DIFFERENCE -- no ranking of the two levers,");
    p.print("which is why it dodges the head-to-head rung 57 declared trapped:");
    p.print("   dI = [M_i(both) - M_i(fuel)] - [M_i(stator) - M_i(neither)]");
    p.print("   M_i = T_c - (1/phi - v)   <- wall = the METAL, the ONE object all four cells share");

    let v = 0.20;
    let design = design13(d);
    let mk = |arm: StatorArm| st(&design, flight, arm);

    let acc = mk(StatorArm::default()).fuel.accel_schedule(flight, LO, HI, 0.25, 13);
    let sc = StatorSchedule::new(v, mk(StatorArm::default()).fuel.inner.equilibrium(flight, LO).nu_lp);
    let leg = StatorLeg { accel: Some(&acc), ..Default::default() };
    let ramp = Ramp::fine(LO, HI, 0.5);
    let sch = mk(StatorArm::scheduled_lp(sc)).composite_credit(flight, &ramp, Spool::Lp, &leg);
    let con = mk(StatorArm::constant(v, 0.0)).composite_credit(flight, &ramp, Spool::Lp, &leg);

    p.print("\n  The four cells (state-fed schedule, rung 48's Wf/pt3 leg, r = 0.5):");
    p.print(pyf!("    {:>8} {:>10} {:>7} {:>7} {:>10}", "cell", "M_i", "s*", "v(s*)", "M_phi"));
    let cells = &sch.cells;
    for (k, c) in [("neither", &cells.neither), ("stator", &cells.stator), ("fuel", &cells.fuel),
                   ("both", &cells.both)] {
        p.print(pyf!("    {:>8} {:10.6f} {:7.4f} {:7.4f} {:+10.6f}", k, c.m_i, c.s, c.v, c.m_phi));
    }
    p.print(pyf!("    stator credit  bare {:+.6f}  ->  with the fuel leg {:+.6f}   dI = {:+.6f} ({:+.2f} %)",
                 sch.credit_bare, sch.credit_fuel, sch.interaction, 100.0 * sch.share));

    let e = mk(StatorArm::scheduled_lp(sc)).engagement_shift(flight, &ramp, &leg);
    p.print("\n  ... and the CONVERSE, sub-grid: the fuel leg's own engagement time");
    p.print(pyf!("    s_eng  bare {:.7f}  ->  stator armed {:.7f}   ({:+.3f} %)",
                 e.bare_dormant, e.armed_dormant, 100.0 * e.rel_dormant));
    p.print(pyf!("    => the influence runs ONE WAY, by a factor of {:.0f}.",
                 (sch.share / e.rel_dormant).abs()));

    let con_tag = pyf!("constant {:.2f}", v);
    p.print("\n  WHERE it lives -- the same composite with a CONSTANT setting, which cannot self-feed:");
    p.print(pyf!("    {:>12} {:>10} {:>10} {:>8} {:>12}", "stator leg", "credit", "dI", "share",
                 "v(s*) ratio"));
    for (tag, q) in [("schedule", &sch), (con_tag.as_str(), &con)] {
        p.print(pyf!("    {:>12} {:10.6f} {:+10.6f} {:7.2f}% {:12.5f}",
                     tag, q.credit_bare, q.interaction, 100.0 * q.share, q.v_ratio));
    }
    p.print(pyf!("    the fuel leg RELOCATES the incidence minimum ({:.4f} -> {:.4f}) and a state-fed schedule is MORE CLOSED there.",
                 cells.neither.s, cells.fuel.s));
    p.print("    A constant setting is read at the same v wherever the minimum goes.");
    p.print(pyf!("    => the two levers DO NOT SUPERPOSE. (The {:.2f} % floor is REAL, not zero, and at r = 0.10 it flips SIGN.)",
                 100.0 * con.share));

    p.print("\n  AND THE OBVIOUS READING IS WRONG -- the near-miss, published with the result.");
    p.print("  dI is strongly ramp-rate-dependent, which invites 'a clock-free lever INHERITS");
    p.print("  its partner's clock'. But dI ANTI-correlates with the bare credit, so what a");
    p.print("  designer is handed -- credit_bare + dI -- is FLATTER in r than the bare credit:");
    p.print(pyf!("    {:>12} {:>12} {:>16}", "stator leg", "bare spread", "composed spread"));
    p.print(pyf!("    {:>12} {:>12} {:>16}", "schedule", "8.53 %", "6.80 %"));
    p.print(pyf!("    {:>12} {:>12} {:>16}", "constant", "3.11 %", "0.89 %"));
    p.print("    => rung 57 is CONFIRMED on the delivered credit. Only the DECOMPOSITION is");
    p.print("       clocked, and a decomposition is not a deliverable.");

    p.print("\n  PREDICTED from the two marches that never saw the fuel leg -- the credit is a");
    p.print("  PROFILE in s, and the leg changes only WHICH POINT of it is read:");
    for (tag, q) in [("schedule", &sch), (con_tag.as_str(), &con)] {
        p.print(pyf!("    {:>12}  measured {:+.6f}   predicted {:+.6f}   ({:.0f} % recovered)",
                     tag, q.interaction, q.predicted, 100.0 * q.predicted / q.interaction));
    }

    p.print("\n  AND A LEG THAT CANNOT COMPOSE AT ALL. Rung 49's phi floor must sit BELOW the");
    p.print("  machine's phi at s=0 and ABOVE its minimum. Those windows are DISJOINT:");
    for (tag, arm) in [("bare", StatorArm::default()), ("schedule", StatorArm::scheduled_lp(sc)),
                       ("constant", StatorArm::constant(v, 0.0))] {
        // `m._stator_march(flight, LO, HI, 0.5, 1.2, 0.005)` — every field spelled by the panel.
        let rp = Ramp { tt4_lo: LO, tt4_hi: HI, r: 0.5, s_settle: 1.2, ds: 0.005 };
        let (tr, _) = mk(arm).stator_march(flight, &rp, None, &StatorLeg::default());
        let phis: Vec<f64> = tr.iter().map(|q| q.phi_lp).collect();
        p.print(pyf!("    {:>9}  admissible floor  ({:.4f}, {:.4f})", tag, py_min(&phis),
                     tr[0].phi_lp));
    }
    p.print("    => rung 53 made a MARGIN coordinate-dependent; this is the same fact reaching a");
    p.print("       LIMITER'S SET POINT. Rung 48's leg composes because Wf/pt3 is stator-invariant.");

    p.print("\n  SCOPE: the fuel leg is ONE object derived on the BARE machine (rung 59 proves this");
    p.print("  IS the matched leg for an LP stator); LP-side, M_i only, one");
    p.print("  gas. See docs/rung58-spec.md § Concessions.");
}

/// `print_matched_floor_table(flight)` — rung 60.
pub fn matched_floor_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE MATCHED phi FLOOR (rung 60): rung 58's refused repair, built -- and it answers");
    p.print("the wrong question. A floor that binds holds its own coordinate AT the set point:");
    p.print("   leg floors phi   ->  M_i(both) - M_i(fuel) = [T_c - 1/phi_lim + v] - [..+0] = v");
    p.print("   leg floors M_i   ->  M_i(both) - M_i(fuel) = m_lim - m_lim               = 0");
    p.print("   (LP spool, r = 0.5, ds = 0.01 -- the spec's tables are the ds = 0.005 grid)");

    // `DS = 0.01`, passed EXPLICITLY at every call — over the readers' own 0.005 default.
    let ramp = |r: f64| Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: 1.2, ds: 0.01 };
    let design = design13(d);
    let mk = |arm: StatorArm| st(&design, flight, arm);

    p.print("\n  BOTH ENDS OF THE TAUTOLOGY, measured against the value they are DERIVED to take:");
    p.print(pyf!("    {:>22} {:>5} {:>13} {:>21} {:>9} {:>10}",
                 "leg", "v", "regime", "M_i(both)-M_i(fuel)", "derived", "residual"));
    for (tag, v, leg) in [
        (" incidence M=0.509", 0.10, Floor::Incidence(IncidenceLimiter::new(Spool::Lp, 0.509))),
        (" incidence M=0.518", 0.15, Floor::Incidence(IncidenceLimiter::new(Spool::Lp, 0.518))),
        ("   phi floor 0.750", 0.15, Floor::Phi(SurgeLimiter::new(Spool::Lp, 0.750))),
    ] {
        let q = mk(StatorArm::constant(v, 0.0)).floor_composite(flight, &ramp(0.5), &leg, Spool::Lp);
        p.print(pyf!("    {:>22} {:5.2f} {:>13} {:21.15f} {:9.3f} {:+10.1e}",
                     tag, v, regime_str(q.regime), q.credit_fuel, q.pinned_prediction,
                     q.pinned_residual));
    }
    p.print("    => a number reproduced to 1e-15 by an identity is not evidence about the machine.");
    p.print("       Re-referencing MOVES the tautology; only a leg that RELOCATES a minimum");
    p.print("       (rung 48's schedule) leaves the derivative to the plant.");

    p.print("\n  WHAT IT DOES BUY -- ADMISSIBILITY. The set-point bands, in both coordinates:");
    let q = mk(StatorArm::constant(0.20, 0.0)).set_point_bands(flight, &ramp(0.5), Spool::Lp);
    p.print(pyf!("    phi   gap {:+.6f}  = {:+6.1f} % of a band   admissible: {}",
                 q.gap_phi, 100.0 * q.gap_phi_bands, q.phi_admissible.py_str()));
    p.print(pyf!("    M_i   gap {:+.6f}  = {:+6.1f} % of a band   admissible: {}",
                 q.gap_m, 100.0 * q.gap_m_bands, q.m_admissible.py_str()));
    p.print(pyf!("    and the gap is an IDENTITY: credit {:.6f} - excursion {:.6f} = {:+.6f}  (residual {:+.1e})",
                 q.credit, q.excursion, q.criterion, q.identity_residual));

    p.print("\n  ... so composability has a CLOCK, and the clock is entirely the RAMP's:");
    let rates: Vec<(f64, StatorArm)> = [0.15, 0.35, 0.5, 1.0].iter()
        .map(|&r| (r, StatorArm::constant(0.20, 0.0))).collect();
    let rows = mk(StatorArm::default())
        .composability_ladder(flight, &ramp(0.5), LadderAxis::Rates(&rates), Spool::Lp);
    p.print(pyf!("    {:>6} {:>10} {:>10} {:>11}  composable", "r", "credit", "excursion",
                 "criterion"));
    for row in &rows {
        p.print(pyf!("    {:6.2f} {:10.6f} {:10.6f} {:+11.6f}  {}",
                     row.r, row.credit, row.excursion, row.criterion, row.m_admissible.py_str()));
    }
    let cr: Vec<f64> = rows.iter().map(|q| q.credit).collect();
    let ex: Vec<f64> = rows.iter().map(|q| q.excursion).collect();
    p.print(pyf!("    credit spread {:.2f} %  (rung 57: a wall-moving lever has NO clock)",
                 100.0 * (py_max(&cr) / py_min(&cr) - 1.0)));
    p.print(pyf!("    excursion spread {:.2f}x  -- the ramp's, and it is what crosses",
                 py_max(&ex) / py_min(&ex)));
    p.print("       the threshold with the lever's setting never touched.");

    p.print("\n  SCOPE: the body is a CONSTANT setting, where the matched floor is a scalar and");
    p.print("  there is no new plant; LP-side, M_i only, one gas. docs/rung60-spec.md.");
}

/// `print_stator_bleed_table(flight)` — rung 61.
pub fn stator_bleed_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nSTATOR + BLEED (rung 61): a compensating lever buys back the COORDINATE, not the");
    p.print("BILL. b* is the valve setting that restores phi_op to its bare value exactly.");

    let m = StatorBleedCore::new(design13(d), *flight, 1.0, lp_map(), hp_map(), 0.0, 0.0, 0.0);

    p.print("\n  THE HEADLINE (LP spool, fixed throttle). The phi-debit goes; the overspeed stays:");
    p.print(pyf!("  {:>6}{:>6}{:>9}{:>10}{:>10}{:>11}{:>10}{:>7}{:>9}",
                 "Tt4", "v", "b*", "phi bare", "phi comp", "dn stator", "dn comp", "kept",
                 "dF comp"));
    for (tt4, v) in [(1500.0, 0.10), (1500.0, 0.20), (1500.0, 0.30), (1300.0, 0.20),
                     (1100.0, 0.20), (1100.0, 0.30)] {
        let c = m.compensated_point(flight, tt4, v, Spool::Lp);
        let k = c.comp.expect("b* exists on every LP row of this grid");
        p.print(pyf!("  {:6.0f}{:6.2f}{:9.5f}{:10.5f}{:10.5f}{:+11.2%}{:+10.2%}{:7.0%}{:+9.2%}",
                     tt4, v, c.b_star.expect("b* exists"), c.phi_bare, k.phi_comp, k.dn_stator,
                     k.dn_comp, k.dn_comp / k.dn_stator, k.d_f_comp));
    }
    p.print("    => at v=0.30 the COMPENSATED point overspeeds the bare stator (kept > 100%):");
    p.print("       undoing the lever is strictly WORSE than leaving it alone.");

    p.print("\n  WHY -- the phi-drop was a REBATE. n solves tau_c = 1 + (tau_d-1)*psi(phi)*n^2,");
    p.print("  and psi = base(phi) - v(1+l)phi. Restoring phi gives base(phi)'s rebate back:");
    let (tt4, v) = (1500.0, 0.20);
    let c = m.compensating_bleed(flight, tt4, v, Spool::Lp, Target::Phi);
    p.print(pyf!("  {:>14}{:>9}{:>10}{:>10}{:>10}{:>10}", "cell", "b", "phi", "n", "psi", "tau_c"));
    for (tag, vv, bb) in [("bare", 0.0, 0.0), ("stator", v, 0.0),
                          ("compensated", v, c.b_star().expect("b* exists at 1500 K"))] {
        let sib = m.at_point(vv, 0.0, bb);
        let od = sib.core.core.match_point(flight, tt4);
        p.print(pyf!("  {:>14}{:9.5f}{:10.5f}{:10.5f}{:10.5f}{:10.5f}",
                     tag, bb, od.phi_lp, od.n_lp, sib.core.core.map_lp().psi(od.phi_lp),
                     od.base.tau_lpc));
    }
    p.print("    => the compensated point is MORE unloaded than the stator-only one.");

    p.print("\n  THE SEAM AS POSED, REFUTED: opening the valve SHRINKS the stator's authority");
    p.print("  (it pre-spends the same incidence budget) -- it does not take over after it:");
    p.print(pyf!("  {:>7}{:>10}{:>11}{:>11}{:>11}", "b", "v_edge", "M_i(v=0)", "M_i peak",
                 "span left"));
    for r in m.authority_with_bleed(flight, 1500.0, &[0.0, 0.05, 0.10, 0.15], Spool::Lp) {
        p.print(pyf!("  {:7.2f}{:10.4f}{:11.5f}{:11.5f}{:11.5f}",
                     r.bleed, r.v_edge, r.m_i_0, r.m_i_peak, r.span));
    }

    p.print("\n  AND IT IS SPOOL-DEPENDENT. The two levers do not span the same space: the");
    p.print("  stator acts on either spool, rung 42's valve on one. On the HP, b* never exists:");
    for r in m.compensability(flight, &[1500.0, 1300.0, 1100.0, 900.0], 0.20) {
        p.print(pyf!("    Tt4={:6.0f}  pi_HPC={:7.4f}   b*_LP={:.5f}   b*_HP = none ({})",
                     r.tt4, r.pi_hpc, r.b_lp.expect("b*_LP exists on every row"),
                     r.why_hp.expect("b*_HP never exists, so a reason is always given")));
    }

    p.print("\n  SCOPE: steady only; b* is a constructed point, not a control law; the compensable");
    p.print("  range is capped by THIS rung's own b<0.45 and no ceiling is claimed. Magnitudes");
    p.print("  ride on the two maps and the imposed floor. See docs/rung61-spec.md § Concessions.");
}

/// Rungs 62/63's grid: `N_LO, V, B = 0.65, 0.20, 0.10`.
const N_LO: f64 = 0.65;
const V: f64 = 0.20;
const B: f64 = 0.10;

/// `STAT = dict(vsv_sched_lp=StatorSchedule(V, N_LO))`.
fn stat_arm() -> LeverArm {
    LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(V, N_LO)))
}

/// `BLED = dict(bleed_sched=BleedSchedule(B, N_LO))`.
fn bled_arm() -> LeverArm {
    LeverArm::scheduled(BleedSchedule::new(B, N_LO))
}

/// `print_bleed_schedule_table(flight)` — rung 62.
pub fn bleed_schedule_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nBLEED SCHEDULE + STATOR SCHEDULE (rung 62): a state-fed schedule's LOOP has a");
    p.print("SIGN. FULL/RAMP < 1 is self-cancellation (rung 57); > 1 is self-AMPLIFICATION.");

    let design = design13(d);
    let (stat, bled) = (stat_arm(), bled_arm());
    let mk = |arm: &LeverArm| bt(&design, flight, arm);

    p.print("\n  Both loop-gain factors on the steady running line (neither reverses):");
    p.print(pyf!("    {:>6} {:>10} {:>10}", "Tt4", "dn_L/db", "dn_L/dv"));
    for row in mk(&LeverArm::default())
        .loop_factors(flight, &[1500.0, 1300.0, 1100.0, 900.0], 0.10, 0.20) {
        p.print(pyf!("    {:6.0f} {:+10.5f} {:+10.5f}", row.tt4, row.dn_db, row.dn_dv));
    }
    p.print("    (bleed LOWERS the speed its schedule reads, the stator RAISES it -- and both");
    p.print("     schedules open at LOW speed, so the two loop gains have opposite signs.)");

    p.print("\n  The loop, per lever, over a 4x ramp-rate range:");
    p.print(pyf!("    {:<22} {:>5} {:>10} {:>9} {:>9}", "lever", "r", "FULL/RAMP", "cmd RAMP",
                 "cmd FULL"));
    for (name, kw) in [(pyf!("stator  v_max={:.2f}", V), &stat), (pyf!("bleed   b_max={:.2f}", B), &bled)] {
        for r in [0.25, 0.50, 1.00] {
            let q = mk(kw).loop_decomposition(flight, &Ramp::new(LO, HI, r), Spool::Lp);
            p.print(pyf!("    {:<22} {:5.2f} {:10.4f} {:9.5f} {:9.5f}",
                         name, r, q.self_cancel, q.cmd_ramp, q.cmd_full));
        }
    }
    p.print("    The stator commands LESS of itself between the two legs; the bleed MORE.");

    p.print("\n  The two loops share ONE state, and they do not compose. The stator schedule's");
    p.print("  own surrender (1 - FULL/RAMP), by what is armed BESIDE it:");
    let t = mk(&LeverArm::default());
    let cmd = mk(&bled).commanded_level(flight, &Ramp::new(LO, HI, 0.25), Spool::Lp).at_min;
    p.print(pyf!("    {:<34} {:>12}", "neighbour", "surrendered"));
    let (matched, more) = (LeverArm::constant(cmd), LeverArm::constant(B));
    for (name, nb) in [("(none)".to_string(), None),
                       (pyf!("constant b = {:.4f}  (matched)", cmd), Some(&matched)),
                       (pyf!("constant b = {:.4f}  (MORE lever)", B), Some(&more)),
                       (pyf!("SCHEDULE  b_max = {:.2f}", B), Some(&bled))] {
        let q = t.marginal_loop(flight, &Ramp::new(LO, HI, 0.25), &stat, nb, Spool::Lp,
                                &StatorLeg::default());
        p.print(pyf!("    {:<34} {:+12.4f}", name, q.surrendered));
    }
    p.print("    The schedule does ~3x the damage of a strictly LARGER constant valve, so it");
    p.print("    is the LOOP and not the LEVEL. The mirror is inert: a stator schedule leaves");
    p.print("    the bleed's amplification alone to within 0.7 %. A ONE-WAY arrow.");

    p.print("\n  And rung 61's steady superposition does not survive the shaft balance's");
    p.print("  removal -- the same two devices, additive to 2.3 % steady:");
    p.print(pyf!("    {:>5} {:>10} {:>10} {:>10} {:>10} {:>12}",
                 "r", "c_stator", "c_bleed", "c_pair", "sum", "interaction"));
    for r in [0.25, 0.50, 1.00] {
        let q = t.pair_interaction(flight, &Ramp::new(LO, HI, r), &stat, &bled, Spool::Lp);
        p.print(pyf!("    {:5.2f} {:+10.5f} {:+10.5f} {:+10.5f} {:+10.5f} {:+12.3f}",
                     r, q.credit_a, q.credit_b, q.credit_pair, q.credit_sum,
                     q.interaction_frac));
    }
    p.print("    SCOPE: b(n_L) is an IMPOSED valve position, not a controlled one; n_lo is a");
    p.print("    placement and the MAGNITUDE rides on it (the sign does not). Every rung-57");
    p.print("    concession is inherited. See docs/rung62-spec.md.");
}

/// `print_fuel_bleed_table(flight)` — rung 63.
pub fn fuel_bleed_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nFUEL + BLEED (rung 63): rung 58's ONE-WAY arrow was a fact about the LEG's two");
    p.print("PROTECTIONS, not about the lever's kind. A bleed breaks both, and the arrow CLOSES.");

    let (stat, bled) = (stat_arm(), bled_arm());
    let t = bt(&design13(d), flight, &LeverArm::default());
    let leg = t.fuel.accel_schedule(flight, LO, HI, 0.25, 13);
    let levers = [(pyf!("LP stator  v_max={:.2f}", V), &stat),
                  (pyf!("bleed      b_max={:.2f}", B), &bled)];

    p.print("\n  The leg's two SENSED INPUTS, per lever (rung 59's table, differenced):");
    p.print(pyf!("    {:<24} {:>12} {:>12} {:>12}", "lever", "d_ordinate", "d_abscissa",
                 "d_MFP(A4)"));
    for (name, kw) in &levers {
        // Only the ramp's two throttle ends are read; `r`/`ds` are inert here.
        let q = t.sensed_inputs(flight, &Ramp::new(LO, HI, 0.5), kw, 0.25, 9, None);
        p.print(pyf!("    {:<24} {:12.3e} {:12.3e} {:12.3e}",
                     name, q.d_ordinate, q.d_abscissa, q.d_mfp));
    }
    p.print("    A choked A4 keeps MFP at machine zero for BOTH -- that control never moves.");
    p.print("    What moves is Tt25: only the LP shaft balance carries (1-b), so Tt3 falls,");
    p.print("    f rises, and BOTH halves of the schedule table shift. Ten orders apart.");

    p.print("\n  So the leg can FEEL a bleed and cannot feel a stator (sub-grid s_eng):");
    p.print(pyf!("    {:<24} {:>5} {:>10} {:>12} {:>9}", "lever", "r", "s_eng ref", "s_eng armed",
                 "shift"));
    let sl = StatorLeg { accel: Some(&leg), ..Default::default() };
    for (name, kw) in &levers {
        for r in [0.25, 0.50, 1.00] {
            let q = t.leg_retiming(flight, &Ramp::fine(LO, HI, r), kw, &sl, None);
            p.print(pyf!("    {:<24} {:5.2f} {:10.5f} {:12.5f} {:+8.3f}%",
                         name, r, q.ref_dormant, q.armed_dormant, 100.0 * q.rel_dormant));
        }
    }
    p.print("    The bleed is positive and the LARGER in every cell; the stator's own shift is");
    p.print("    trajectory-mediated (its TABLE is bit-identical) and spans -0.03 % to +1.28 %.");
    p.print("    LATER, not earlier -- a sign the pressure channel alone gets WRONG (spec § 2).");

    p.print("\n  And a phi FLOOR beside the valve has NO composable middle -- two regimes only,");
    p.print("  with the boundary at the two plants' OWN minimum phi:");
    let q = t.floor_dichotomy(flight, &Ramp::fine(LO, HI, 0.5), &bled, &[0.36, 0.40, 0.43, 0.46],
                              Spool::Lp, None);
    p.print(pyf!("    band in sm = [{:.4f}, {:.4f}]   min phi: ref {:.5f}, armed {:.5f}",
                 q.band.0, q.band.1, q.min_phi_ref, q.min_phi_armed));
    p.print(pyf!("    {:>5} {:>8} {:>14} {:>14} {:>11}", "sm", "phi_lim", "removed(fuel)",
                 "removed(both)", "credit"));
    for row in &q.rows {
        p.print(pyf!("    {:5.2f} {:8.5f} {:14.4e} {:14.4e} {:+11.3e}",
                     row.sm, row.phi_lim, row.removed_fuel, row.removed_both, row.credit));
    }
    p.print("    Inside the band the valve DISARMS the floor exactly (and the armed cell is");
    p.print("    bit-for-bit its own leg-free march); above it both bind, the floor pins the");
    p.print("    currency, and the valve's credit is exactly zero -- rung 60's tautology.");
    p.print("    SCOPE: only the FEEDFORWARD leg composes; the valve is an IMPOSED position;");
    p.print("    s_eng's magnitude rides on rung 48's disclaimed margin. docs/rung63-spec.md.");
}
