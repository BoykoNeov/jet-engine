//! Rungs 64–71's panels — the bleed valve, its lag, the cascades and the reference splits
//! (slice AQ, first part).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line for
//! line, on rung 53's hardware ([`design13`], [`lp_map`], [`hp_map`]). **The march step is the trap
//! again**, read off each reader's own SIGNATURE: rung 64's two readers take their `ds = 0.005`
//! default; rungs 65–67 pass `ds=DS=0.005` EXPLICITLY over defaults of 0.005 (65) and 0.0025
//! (66/67); rung 67's `detector_sensitivity()` takes all its defaults (`ds = 0.0025`); rungs 68–71
//! pass no `ds`, so each reader's own default stands — 0.005 for most, but **0.002** for rung 69's
//! `reference_modes` and rung 71's `full_gains`. A flat Python key that lives in a Rust `Option`
//! is unwrapped: Python would raise where it is absent, so the golden proves it present.

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::bleed_transient::LeverArm;
use crate::cross_loop::{build_cross_loop_cascade, detector_sensitivity, OscRow};
use crate::cross_split::{build_cross_split_cascade, split_floor, split_gains, window_overlap};
use crate::engine::FlightCondition;
use crate::full_split::{
    band_containment, build_full_split_cascade, full_bill, full_gains, ic_contraction, window_law,
};
use crate::lagged_bleed::build_lagged_bleed;
use crate::limited_bleed::{build_limited_bleed, BleedLimiter};
use crate::map::ComponentMap;
use crate::pyf;
use crate::pyfmt::{py_list, py_tuple, Printer, PyFormat};
use crate::reference_split::{
    build_reference_split_cascade, reference_bill, reference_gains, reference_modes,
    StatorIncidenceLimiter,
};
use crate::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};
use crate::three_loop::{
    build_three_loop_cascade, cyclic_sensitivity, triple_bill, triple_gains, StatorLimiter,
    TripleRigArm,
};
use crate::two_lag::build_two_lag_cascade;
use crate::two_spool::TwoSpoolEngine;

/// `FLOOR = 0.55` — rung 36/41's imposed `phi_surge`.
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const B: f64 = 0.10;
const PHI: f64 = 0.80;

/// The transient builders' common signature: `Cls(design, flight, 1.0, map_lp=LP, map_hp=HP,
/// rho=1.0, **kw)`.
type Build = fn(TwoSpoolEngine, FlightCondition, f64, Option<ComponentMap>, Option<ComponentMap>,
                f64, &LeverArm) -> ScheduledStatorTransient;

fn machine(build: Build, d: &Design, arm: &LeverArm) -> ScheduledStatorCore {
    match build(design13(d), d.flight, 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("both maps are passed"),
    }
}

/// `(LO, HI)` at `r = 0.5`, `s_settle = 1.2` and the given `ds`.
fn ramp(ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r: 0.5, s_settle: 1.2, ds }
}

/// `print_bleed_limiter_table(flight)` — rung 64.
pub fn bleed_limiter_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nphi-REFERENCED BLEED LIMITER (rung 64): a limiter's LAW cannot buy PROTECTION,");
    p.print("only its PRICE. The ceiling is the lever's AUTHORITY; feedback buys the BILL.");

    let t = machine(build_limited_bleed, d, &LeverArm::default());

    p.print("\n  THE CEILING -- and the top row is an OPEN-LOOP law that reaches it:");
    // `authority_ceiling(flight, LO, HI, b_max=B)`: n_lo = 0.65, sm_over = 0.10, ds = 0.005.
    let c = t.authority_ceiling(flight, &ramp(0.005), B, 0.65, 0.10);
    p.print(pyf!("    {:<26} {:>9} {:>9} {:>14}", "law", "min phi", "sm", "b at that min"));
    let over_name = pyf!("FLOOR  phi_lim={:.4f}", c.phi_lim_over);
    for (r, name) in [(&c.shut, "valve SHUT".to_string()),
                      (&c.schedule, pyf!("SCHEDULE  b_max={:.2f}", B)),
                      (&c.full, pyf!("CONSTANT  b = {:.2f}", B)),
                      (&c.over, over_name)] {
        p.print(pyf!("    {:<26} {:9.6f} {:9.6f} {:14.6f}",
                     name, r.min_phi_lp, r.min_phi_lp / FLOOR - 1.0, r.b_at_min_lp));
    }
    p.print(pyf!("    The over-set floor SATURATES and is VIOLATED by {:+.4f}, and it", c.over_deficit));
    p.print(pyf!("    lands BELOW the fully-open march ({:+.3e}) -- so no law beats", c.over_vs_full));
    p.print("    b = b_max. The schedule's gap is PLACEMENT (it is not saturated at its own min).");

    p.print("\n  THE BILL -- three laws of ONE lever, matched to the SAME min phi, in rung 61's");
    p.print("  currency (the overspeed and the thrust, NOT the bleed integral):");
    // `matched_bill(flight, LO, HI, phi_target=PHI, b_cap=B)`: n_lo = 0.65, b_hi = 0.30.
    let m = t.matched_bill(flight, &ramp(0.005), PHI, B, 0.65, 0.30);
    p.print(pyf!("    match error {:.1e};  b* = {:.5f}, b_max* = {:.5f}", m.matched, m.b_star,
                 m.bmax_star));
    p.print(pyf!("    {:<24} {:>10} {:>12} {:>9} {:>9} {:>13}",
                 "law", "int b ds", "d nu_lp_end", "dF_end", "d int F", "d min phi_hp"));
    for (q, name) in [(&m.bill_constant, "1 constant   (blind)"),
                      (&m.bill_schedule, "2 schedule   (state-fed)"),
                      (&m.bill_floor, "3 phi FLOOR  (feedback)")] {
        p.print(pyf!("    {:<24} {:10.6f} {:+12.6f} {:+8.3f}% {:+8.3f}% {:+13.3e}",
                     name, q.b_int, q.d_nu_lp_end, q.thrust_end_pct, q.thrust_int_pct,
                     q.d_min_phi_hp));
    }
    p.print(pyf!("    The bill falls with the INFORMATION the law uses: floor/schedule = {:.4f} in",
                 m.b_ratio_sched));
    p.print("    bleed, and the closed loop's END thrust bill is machine-zero -- it");
    p.print("    self-releases, so it has left the machine by settle. The honest comparator is");
    p.print("    law 2 (a constant valve bleeds hardest where phi is already highest).");
    p.print("    The state-fed laws DEBIT the HP; the state-blind one CREDITS it.");
    p.print("    SCOPE: the valve is INSTANTANEOUS and unlagged; phi_lim rides on rung 36's");
    p.print("    disclaimed phi_surge and b_max is rung 42's valve size, so the MAGNITUDES are");
    p.print("    disclaimed and the ORDERING is the claim. See docs/rung64-spec.md.");
}

/// `print_lagged_valve_table(flight)` — rung 65.
pub fn lagged_valve_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE LAGGED BLEED VALVE (rung 65): a lag repairs the SOLVE without removing the");
    p.print("DEGENERACY -- the redundancy of two loops on one variable is CONSERVED.");

    let ds = 0.005;
    let sm = PHI / FLOOR - 1.0;
    let tau = 0.05;
    let t = machine(build_lagged_bleed, d, &LeverArm::default());

    p.print("\n  BANDWIDTH, THE SECOND HARDWARE AXIS -- and it is PURE LOSS (a slower valve");
    p.print("  protects LESS *and* bleeds MORE), with rung 64's instantaneous valve as the top row:");
    let bc = t.bandwidth_ceiling(flight, &ramp(ds), PHI, B, &[0.4, 0.2, 0.1, 0.05, 0.02, 0.01]);
    p.print(pyf!("    {:>8} {:>12} {:>11} {:>9} {:>8} {:>11}",
                 "tau", "min phi_lp", "undershoot", "int b ds", "plateau", "dev vs r64"));
    p.print(pyf!("    {:>8} {:12.9f} {:11.3e} {:9.6f} {:8d} {:>11}",
                 "rung 64", bc.inst_min_phi, 0.0, bc.inst_b_int, bc.inst_plateau_pts, "--"));
    for row in &bc.rows {
        p.print(pyf!("    {:8.3f} {:12.9f} {:11.3e} {:9.6f} {:8d} {:11.3e}",
                     row.tau, row.min_phi_lp, row.undershoot, row.b_int, row.plateau_pts, row.dev));
    }
    let rows = &bc.rows;
    let worse = rows.windows(2).all(|w| w[0].undershoot < w[1].undershoot);
    let more = rows.windows(2).all(|w| w[0].b_int > w[1].b_int);
    p.print(pyf!("    Both currencies monotone in tau and the SAME way (protection {}, bleed {}):",
                 worse.py_str(), more.py_str()));
    p.print("    there is no trade to buy. And rung 64 s 4's DESTROYED argmin is RESTORED -- its");
    p.print(pyf!("    floor pins phi_lp over {} points (an interval, ~1/ds), every lagged march over 1.",
                 bc.inst_plateau_pts));
    p.print(pyf!("    The tau -> 0 arm of the reduce is MEASURED, not asserted: dev shrinks monotonically ({}).",
                 bc.dev_shrinks.py_str()));

    p.print("\n  THE DISCRIMINATOR -- rung 64 s 3 DERIVED that an instantaneous valve re-pins");
    p.print("  phi_lp at every trial fuel, deleting the fuel leg's plant. Here it is EXHIBITED,");
    p.print("  one bracket swept on both plants at the state where a fuel leg bites hardest:");
    let fracs = [1.0, 0.99, 0.98, 0.95, 0.90];
    let fa = t.fuel_authority(flight, &ramp(ds), sm, B, tau, &fracs);
    let fr: Vec<&dyn PyFormat> = fracs.iter().map(|x| x as &dyn PyFormat).collect();
    p.print(pyf!("    probe: s = {:.3f}, b = {:.6f}, phi_lp = {:.6f}, Wf x {}",
                 fa.at.s, fa.at.b, fa.at.phi_lp, py_tuple(&fr)));
    for (q, name) in [(&fa.inst, "INSTANTANEOUS (rung 64)".to_string()),
                      (&fa.lagged, pyf!("LAGGED  tau = {}", tau))] {
        p.print(pyf!("    {:<24} phi span {:11.4e}  monotone {:>5}  sign change {:>5}",
                     name, q.span, q.monotone.py_str(), q.sign_change.py_str()));
    }
    p.print("    The instantaneous span is ROUNDOFF (its monotonicity is meaningless), so the");
    p.print(pyf!("    ratio {:.1e} has a machine-zero denominator and only its ORDER is a", fa.ratio));
    p.print("    statement. Inside ONE derivative evaluation a lagged valve is a CONSTANT, so the");
    p.print("    fuel leg sees rung 42's imposed-valve plant and rung 49's own premise -- phi");
    p.print("    falls monotonically with fuel -- is restored verbatim.");

    p.print("\n  AND THE CONTINUUM SURVIVES ANYWAY -- THE RUNG. Wherever both loops ride, every");
    p.print("  (b, Wf) on phi_lp = phi_lim satisfies BOTH laws, so b_cmd == b and db/ds == 0:");
    // `marginal_mode(..., tau=TAU, taus=(0.2, 0.01), ds=DS)`: d_b0 = 0.01.
    let mm = t.marginal_mode(flight, &ramp(ds), sm, B, tau, &[0.2, 0.01], 0.01);
    p.print(pyf!("    {:<22} {:>9} {:>10} {:>14} {:>13} {:>10}",
                 "member", "b0", "drift", "|b_cmd-b|/tau", "fuel removed", "laws held"));
    for (cell, name) in [(&mm.natural, "natural (the EDGE)"), (&mm.moved_lo, "b0 - 0.01 (inside)"),
                         (&mm.moved_hi, "b0 + 0.01 (outside)")] {
        p.print(pyf!("    {:<22} {:9.6f} {:10.2e} {:14.2e} {:13.6e} {:10.2e}",
                     name, cell.b0, cell.drift, cell.dbds, cell.removed, cell.laws_held));
    }
    p.print("    b is a CONSTANT OF THE MOTION -- the drift column IS the claim (b_end = b0 to");
    p.print("    ~4e-16 over a whole march), so the family is one-parameter in b0 alone,");
    p.print(pyf!("    and its members withhold DIFFERENT fuel (ratio {:.3f} against the member",
                 mm.natural.removed / mm.moved_lo.removed));
    p.print("    0.01 below). The EDGE is derivable -- the valve's law is the SMALLEST position");
    p.print("    holding the floor, so ABOVE b_cmd(0) it is doing more than its law asks and");
    p.print("    CLOSES (the third row's drift), which is why the natural march looks unique.");
    p.print("    tau multiplies a machine zero and cannot reach the mode: withheld fuel is");
    p.print(pyf!("    tau-invariant to {:.1e} relative across a 20x range.", mm.tau_span_rel));
    p.print("    SCOPE: one anchor (rung 64's grid), phi_lim/b_max imposed or inherited, and the");
    p.print("    RK4 floor ds/tau <~ 2.78 is asserted in the plant -- a violation looks exactly");
    p.print("    like the finding 'a fast valve bleeds more'. See docs/rung65-spec.md.");
}

/// `print_two_lag_cascade_table(flight)` — rung 66.
pub fn two_lag_cascade_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE TWO-LAG CASCADE (rung 66): two loops on ONE variable are ONE loop with the");
    p.print("RATES ADDED -- R_q * C_g == 1 is an IDENTITY, so det J == 0 and the pair is degenerate.");

    let ds = 0.005;
    let (sm, tau, tau_att) = (PHI / FLOOR - 1.0, 0.05, 0.05);
    let t = machine(build_two_lag_cascade, d,
                    &LeverArm::floored(BleedLimiter::with_tau(PHI, B, Some(tau))));

    p.print("\n  THE IDENTITY, MEASURED -- both cross-gains central-differenced at RIDING points");
    p.print("  (`required > 0` AND the valve strictly inside its stops), at three fuel clocks:");
    // `cascade_identity(..., tau=TAU, ds=DS)`: tau_atts = (0.005, 0.05, 0.5), rel_mult = 3.0,
    // n_sample = 12.
    let ci = t.cascade_identity(flight, &ramp(ds), sm, B, tau, &[0.005, 0.05, 0.5], 3.0, 12);
    p.print(pyf!("    {:>8} {:>22} {:>13} {:>13} {:>10} {:>12}",
                 "tau_att", "R_q*C_g lo..hi", "gain swing R", "gain swing C", "|lam| max",
                 "1/t_g+1/t_v"));
    for row in &ci.rows {
        p.print(pyf!("    {:8.3f} {:10.6f}..{:<10.6f} {:13.3f} {:13.3f} {:10.2f} {:12.2f}",
                     row.tau_att, row.prod_lo, row.prod_hi, row.gain_span_r, row.gain_span_c,
                     row.rho_max, row.rate_closed_form));
    }
    let mid = &ci.rows[1];
    p.print(pyf!("    gains at tau_att = {}:  R_q in [{:.4f}, {:.4f}],  C_g in [{:.4f}, {:.4f}]",
                 mid.tau_att, mid.r_q_lo, mid.r_q_hi, mid.c_g_lo, mid.c_g_hi));
    p.print("    The product holds to a few percent over 100x in the fuel clock while the gains");
    p.print("    themselves swing -- so it is a RECIPROCAL PAIR and not a constant plant, and both");
    p.print("    are strictly NEGATIVE (SUBSTITUTING loops). The spectrum is exactly");
    p.print(pyf!("    {{0, -(1/t_g + 1/t_v)}}: real at every sampled point ({}), and the",
                 ci.all_real.py_str()));
    p.print(pyf!("    non-zero root matches the SUM of the rates to {:.1e} relative.", ci.rho_err_max));

    p.print("\n  WHAT THE PAIR DELIVERS -- both cells LAGGED on purpose (an instantaneous control");
    p.print("  is a different plant, not a control), scored on the violation AREA int max(0,");
    p.print("  phi_lim - phi_lp) ds, which no initial condition can clamp:");
    let cb = t.cascade_bill(flight, &ramp(ds), sm, B, tau, tau_att, 3.0);
    p.print(pyf!("    {:>7} {:>13} {:>9} {:>14} {:>9}", "cell", "I (phi)", "credit", "min phi (s>0)",
                 "s at min"));
    for (k, c, credit) in [("bare", &cb.bare, None), ("fuel", &cb.fuel, Some(cb.credit_fuel)),
                           ("valve", &cb.valve, Some(cb.credit_valve)),
                           ("both", &cb.both, Some(cb.credit_both))] {
        let cr = match credit {
            None => "--".to_string(),
            Some(x) => pyf!("{:7.2f}%", x * 100.0),
        };
        p.print(pyf!("    {:>7} {:13.6e} {:>9} {:14.9f} {:9.4f}", k, c.i, cr, c.min_phi, c.s_at_min));
    }
    p.print(pyf!("    The pair beats both singles ({}) and is strongly SUB-ADDITIVE:",
                 cb.beats_both.py_str()));
    p.print(pyf!("    the two standalone credits sum to {:.2f}% and the pair delivers {:.2f}%.",
                 cb.sum_alone * 100.0, cb.delivered * 100.0));
    p.print(pyf!("    {:>12} {:>10} {:>9} {:>9}", "added LAST", "marginal", "alone", "erosion"));
    for (k, mg, al, er) in [("fuel", cb.marginal_fuel, cb.credit_fuel, cb.erosion_fuel),
                            ("valve", cb.marginal_valve, cb.credit_valve, cb.erosion_valve)] {
        p.print(pyf!("    {:>12} {:9.3f}% {:8.2f}% {:8.1f}x", k, mg * 100.0, al * 100.0, er));
    }
    p.print("    A WHOLE SECOND LIMITER on top of the stronger one buys almost nothing: with one");
    p.print("    effective actuator direction, the second loop buys the RATE, never the AUTHORITY.");
    p.print("    SCOPE: one anchor and one SET POINT -- both laws are referenced to the SAME");
    p.print("    phi_lim, which is what makes them redundant; the erosion MAGNITUDE is a");
    p.print("    measurement on this grid and the ordering is the claim. See docs/rung66-spec.md.");
}

/// `print_cascade_a_table(flight)` — rung 67.
pub fn cascade_a_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nCASCADE A (rung 67): two loops on TWO variables. ONE SCALAR P = R_q * C_g < 0 sets");
    p.print("BOTH faces -- it ends the degeneracy AND opens a ringing window, then damps it.");

    let (tmax, ds) = (1200.0, 0.005);
    let (tau, tau_gov) = (0.05, 0.05);
    let t = machine(build_cross_loop_cascade, d,
                    &LeverArm::floored(BleedLimiter::with_tau(PHI, B, Some(tau))));

    p.print("\n  THE SCALAR -- the two cross-gains have OPPOSITE SIGNS (bleed makes it HOTTER,");
    p.print("  clipping fuel needs LESS bleed), so their product is negative and det J != 0:");
    // `cross_identity(..., tau=TAU, n_sample=8, ds=DS)`: tau_govs = (0.005, 0.05, 0.5).
    let ci = t.cross_identity(flight, &ramp(ds), tmax, tau, &[0.005, 0.05, 0.5], 8);
    p.print(pyf!("    {:>8} {:>8} {:>12} {:>9} {:>10} {:>9} {:>18}",
                 "tau_gov", "t_v/t_g", "P = R_q*C_g", "min R_q", "max C_g", "complex", "window rho"));
    for row in &ci.rows {
        p.print(pyf!("    {:8.3f} {:8.2f} {:12.6f} {:9.4f} {:10.4f} {:4d}/{:<4d} {:9.4f}..{:.4f}",
                     row.tau_gov, row.rho_clock, row.p_mid, row.r_q_lo, row.c_g_hi, row.n_complex,
                     row.n_sample, row.rho_lo.expect("the window opens on every row"),
                     row.rho_hi.expect("the window opens on every row")));
    }
    p.print(pyf!("    P is NEGATIVE everywhere ({}), and ~50x smaller in magnitude",
                 ci.all_negative.py_str()));
    p.print("    than cascade B's identically-1. The complex branch is a closed form in ONE");
    p.print("    coordinate -- rho + 1/rho < 2 + 4|P| -- so the edges are RECIPROCALS and the");
    p.print("    window is log-symmetric about matched clocks; the spectrum lands where it says.");
    p.print(pyf!("    R_q is not machine-zero ({:.2e}): a zero there would mean the", ci.r_q_min_abs));
    p.print("    governor is not sensing the live valve, i.e. two independent loops, not a cascade.");

    p.print("\n  ADMISSIBLE, BUT UNOBSERVABLE. The FREE response (natural minus a b0-offset march,");
    p.print("  so the common fuel ramp cancels) never rings -- inside the window or outside it:");
    // `oscillation_window(..., tau=TAU, rhos=(0.5, 1.0, 2.0), ds=DS)`: d_b0 = 0.005.
    let ow = t.oscillation_window(flight, &ramp(ds), tmax, tau, &[0.5, 1.0, 2.0], 0.005);
    p.print(pyf!("    P = {:.6f};  window {:.4f}..{:.4f},  zeta = {:.4f},  T/tau = {:.1f}",
                 ow.p, ow.window.rho_lo.expect("the window opens"),
                 ow.window.rho_hi.expect("the window opens"), ow.window.zeta,
                 ow.window.t_over_tau));
    p.print(pyf!("    {:>6} {:>8} {:>9} {:>18} {:>17}",
                 "rho", "tau_gov", "complex?", "sign changes q/g", "offset surviving"));
    for row in &ow.rows {
        let OscRow::Live(row) = row else { unreachable!("Python reads every row's keys") };
        p.print(pyf!("    {:6.2f} {:8.4f} {:>9} {:8d}/{:<9d} {:17.2e}",
                     row.rho, row.tau_gov, row.complex_predicted.py_str(), row.sign_changes_q,
                     row.sign_changes_g, row.survives));
    }
    p.print("    THE THRESHOLD IS TWO CROSSINGS, and it is a theorem rather than a tolerance: a");
    p.print("    sum of two decaying REAL exponentials has at most ONE zero, so a single crossing");
    p.print("    is admissible on the real branch and carries nothing. None of these reaches two.");
    p.print("    The SAME scalar that opens the window sets the damping -- zeta = 1/sqrt(1+|P|)");
    p.print("    and T/tau = 2pi/sqrt|P| contain NO time constant, so no bandwidth can make the");
    p.print("    mode visible. A null is worth nothing until the detector is shown to FIRE:");
    // `detector_sensitivity()`: Ps = (-0.02, -0.5, -3.0, -10.0), tau = 0.05, ds = 0.0025,
    // s_end = 1.7.
    let ds_ = detector_sensitivity(&[-0.02, -0.5, -3.0, -10.0], 0.05, 0.0025, 1.7);
    p.print(pyf!("    {:>7} {:>8} {:>8} {:>17} {:>13} {:>7}",
                 "|P|", "zeta", "T/tau", "periods in march", "sign changes", "rings"));
    for row in &ds_.rows {
        p.print(pyf!("    {:7.2f} {:8.4f} {:8.2f} {:17.2f} {:13d} {:>7}",
                     row.p.abs(), row.zeta, row.t_over_tau, row.periods, row.sign_changes,
                     row.rings.py_str()));
    }
    p.print("    So the plant's silence is a STATEMENT about |P| ~ 0.02 (the period is ~45 tau and");
    p.print("    the response is e^-45 by then), not about the counter.");

    p.print("\n  THE LEDGER cascade B could not build -- TWO currencies, so the 2x2 is SIGNED:");
    let cb = t.cross_bill(flight, &ramp(ds), tmax, tau, tau_gov);
    p.print(pyf!("    {:>7} {:>13} {:>10} {:>13} {:>11} {:>9}",
                 "cell", "I_T (Tt4)", "credit_T", "I_phi", "credit_phi", "max Tt4"));
    for (name, c, credit) in [("bare", &cb.bare, None),
                              ("governor", &cb.gov, Some((cb.credit_t_gov, cb.credit_phi_gov))),
                              ("valve", &cb.valve, Some((cb.credit_t_valve, cb.credit_phi_valve))),
                              ("both", &cb.both, Some((cb.credit_t_both, cb.credit_phi_both)))] {
        let (ct, cf) = match credit {
            None => ("--".to_string(), "--".to_string()),
            Some((t_, f_)) => (pyf!("{:8.2f}%", t_ * 100.0), pyf!("{:9.2f}%", f_ * 100.0)),
        };
        p.print(pyf!("    {:>7} {:13.6e} {:>10} {:13.6e} {:>11} {:9.2f}",
                     name, c.i_t, ct, c.i_phi, cf, c.max_tt4));
    }
    p.print("    OFF-DIAGONAL, and it has no cascade-B analogue: the valve DEBITS the temperature");
    p.print(pyf!("    ({:+.2f}%) while the governor CREDITS the surge margin ({:+.2f}%) --",
                 cb.valve_on_t * 100.0, cb.gov_on_phi * 100.0));
    p.print("    one loop helps the other, the other hurts it, and both signs are derivable from");
    p.print("    R_q > 0 and C_g < 0 before any march is run.");
    p.print(pyf!("    DIAGONAL: each loop keeps ~all of its own credit -- erosion {:.2f}x (governor)",
                 cb.erosion_gov));
    p.print(pyf!("    and {:.2f}x (valve), against rung 66's 38x on the SHARED", cb.erosion_valve));
    p.print("    variable. Same instrument, same phi_lim, opposite verdict: two loops on TWO");
    p.print("    variables buy AUTHORITY.");
    p.print("    SCOPE: one anchor; Tt4_max/phi_lim/b_max are imposed or inherited, the clocks are");
    p.print("    swept march coordinates, and P is measured on THIS plant -- the MAGNITUDES are");
    p.print("    disclaimed and the signs and the ordering are the claim. See docs/rung67-spec.md.");
}

/// Python's `max(d.values())` over a three-entry dict, in its insertion order.
fn max3((a, b, c): (f64, f64, f64)) -> f64 {
    [b, c].iter().fold(a, |m, &x| if x > m { x } else { m })
}

/// `print_three_loop_table(flight)` — rung 68.
pub fn three_loop_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHREE LOOPS ON ONE VARIABLE (rung 68): `n` loops on one variable are ONE loop");
    p.print("with ALL `n` RATES ADDED -- J = -D c r^T is RANK ONE at every n.");

    let vm = 0.20;
    let sm = PHI / FLOOR - 1.0;
    let t = machine(build_three_loop_cascade, d, &LeverArm {
        bleed_lim: Some(BleedLimiter::with_tau(PHI, B, Some(0.05))),
        stator_lim: Some(StatorLimiter::new(PHI, vm, Some(0.05))),
        ..LeverArm::default()
    });
    // Every reader below takes its own defaults: ds = 0.005 and rung 68's six clocks.
    let (rp, arm) = (ramp(0.005), TripleRigArm::default());

    p.print("\n  WHAT IS ACTUALLY INDEPENDENT -- the three PAIRWISE products are rung 66's one");
    p.print("  identity three times; only the CYCLIC product tests JOINT collapse:");
    let g = triple_gains(&t, flight, &rp, sm, &arm, 35);
    p.print(pyf!("    {:>7} {:>11} {:>11} {:>11} {:>20}",
                 "s", "R_q*C_g", "R_v*V_g", "C_v*V_q", "CYCLIC R_q*C_v*V_g"));
    for row in &g.rows {
        let o = &row.on;
        p.print(pyf!("    {:7.4f} {:11.8f} {:11.8f} {:11.8f} {:20.10f}",
                     row.s, o.pair_rc, o.pair_rv, o.pair_cv, o.cyclic));
    }
    p.print(pyf!("    {} points with all three loops riding INTERIOR. Predicted -1 (three factors",
                 g.n_riding));
    p.print("    of -phi_j/phi_i). det = (x+1)^2/x, so det carries nothing the cycle does not.");
    // `deltas = (0.0, 1e-4, 1e-3, 1e-2, 3e-2)`.
    let s = cyclic_sensitivity(&t, flight, &rp, sm, &arm, &[0.0, 1e-4, 1e-3, 1e-2, 3e-2]);
    p.print(pyf!("    The DETECTOR, measured not asserted: floor {:.2e}, gain {:.2f} per unit",
                 s.floor, s.gain.expect("the detector has a gain")));
    p.print(pyf!("    off-manifold displacement -- it resolves delta >~ {:.0e}.",
                 s.resolves.expect("the detector resolves a delta")));

    p.print("\n  THE LEDGER -- every subset of the three loops, and the WALL each credit is");
    p.print("  measured against (the stator MOVES the phi wall and not the metal one):");
    let b = triple_bill(&t, flight, &rp, sm, &arm);
    p.print(pyf!("    {:>5} {:>13} {:>9} {:>15} {:>9}", "cell", "I (phi)", "credit", "I (incidence)",
                 "credit"));
    for k in ["bare", "F", "V", "S", "FV", "FS", "VS", "FVS"] {
        let c = b.cell(k);
        p.print(pyf!("    {:>5} {:13.6e} {:8.2f}% {:15.6e} {:8.2f}%", k, c.i, c.credit, c.i_inc,
                     c.credit_inc));
    }
    p.print(pyf!("    Three standalone credits sum to {:.1f}%; the TRIPLE delivers {:.1f}%.",
                 b.sum_singles, b.delivered));
    p.print(pyf!("    {:>12} {:>16} {:>9} {:>9} {:>18}",
                 "added LAST", "marginal (phi)", "alone", "erosion", "marginal (incid)"));
    let (mg, si, er, mi) = (b.marginal, b.singles, b.erosion, b.marginal_incidence);
    for (k, m, s1, e, i) in [("fuel", mg.0, si.0, er.0, mi.0), ("valve", mg.1, si.1, er.1, mi.1),
                             ("stator", mg.2, si.2, er.2, mi.2)] {
        p.print(pyf!("    {:>12} {:15.3f}% {:8.2f}% {:8.1f}x {:17.3f}%", k, m, s1, e, i));
    }
    p.print("    The stator PROTECTS phi and ERODES incidence -- rung 53's 'a margin is a");
    p.print("    DISTANCE' reaching the SIGN of a credit. And rung 66 s 9's guess that the");
    p.print("    third limiter would buy LEAST is wrong: the FUEL leg, added last, buys least.");
    p.print("    SCOPE: the phi-referenced stator loop moves the lever the ANTI-PHYSICAL way (a");
    p.print("    real VSV closes to protect; this one OPENS), all three tau are swept march");
    p.print("    coordinates, and phi_lim/b_max/v_max are imposed or inherited -- so the");
    p.print("    MAGNITUDES are disclaimed and the ORDERING is the claim. See docs/rung68-spec.md.");
}

/// `print_reference_split_table(flight)` — rung 69.
pub fn reference_split_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE REFERENCE SPLIT (rung 69): a loop's COORDINATE, not its actuator, decides");
    p.print("whether it adds a ZERO or a RANK -- ZEROS = n - m, with m the CONSTRAINT count.");

    let vm = 0.20;
    let sm = PHI / FLOOR - 1.0;
    let lp = lp_map();
    let t = machine(build_reference_split_cascade, d, &LeverArm {
        bleed_lim: Some(BleedLimiter::with_tau(PHI, B, Some(0.05))),
        stator_inc: Some(StatorIncidenceLimiter::from_margin(&lp, vm, sm, Some(0.05))),
        ..LeverArm::default()
    });
    let t_c = lp.tan_beta1_crit();
    p.print(pyf!("  ONE PHYSICAL WALL, TWO COORDINATES: phi_lim = {} is m_lim = T_c - 1/phi_lim = {:.6f}",
                 PHI, t_c - 1.0 / PHI));
    p.print("  at the DESIGN stator setting. dphi/dv = -0.423 but dM_i/dv = +0.335, so THIS loop");
    p.print("  CLOSES the stators -- the physical direction, and the opposite of rung 68's.");

    p.print("\n  WHICH PAIRS KEEP RUNG 66's IDENTITY READS OFF WHICH LOOPS SHARE A CONSTRAINT.");
    p.print("  Both references, the SAME base points on ONE trajectory:");
    // `reference_gains(..., sm=SM, every=20)`: ds = 0.005 and the six default clocks.
    let g = reference_gains(&t, flight, &ramp(0.005), sm, &TripleRigArm::default(), 20);
    p.print(pyf!("    {:>7} | {:>11} {:>10} {:>10} {:>10} | {:>11} {:>9} {:>9}",
                 "s", "R_q*C_g", "R_v*V_g", "C_v*V_q", "CYCLIC", "R_q*C_g", "R_v*V_g", "CYCLIC"));
    p.print(pyf!("    {:>7} | {:^45} | {:^33}",
                 "", "-- incidence-referenced (rung 69) --", "-- phi (rung 68) --"));
    for row in &g.rows {
        let (i, ph) = (&row.inc, &row.phi);
        p.print(pyf!("    {:7.4f} | {:11.8f} {:10.5f} {:10.5f} {:10.5f} | {:11.8f} {:9.5f} {:9.5f}",
                     row.s, i.pair_rc, i.pair_rv, i.pair_cv, i.cyclic, ph.pair_rc, ph.pair_rv,
                     ph.cyclic));
    }
    p.print(pyf!("    The SHARED pair holds at 1 to {:.1e} under BOTH. The split",
                 g.worst_rc_inc.expect("rows were sampled")));
    p.print(pyf!("    pairs BOTH go to k in [{:.4f}, {:.4f}] -- equal to {:.2f}%, which",
                 g.k_range.0.expect("rows were sampled"), g.k_range.1.expect("rows were sampled"),
                 100.0 * g.worst_pair_gap.expect("rows were sampled")));
    p.print("    measures that the two walls differ by exactly the LEVER'S OWN channel.");

    p.print("\n  THE SPECTRUM -- and the invariant rung 68 used CANNOT SEE THIS:");
    // `reference_modes(..., clocks=…, every=60)`: ds = 0.002, v_max = 0.20, tau_rel_mult = 3.0.
    let r = reference_modes(&t, flight, &ramp(0.002), sm,
                            &[(0.05, 0.05, 0.05), (0.05, 0.005, 0.05)], 0.20, 3.0, 60);
    p.print(pyf!("    {:>20} {:>5} {:>7} {:>11} {:>11} {:>8} {:>8}",
                 "taus (g,q,s)", "ref", "zeros", "|c0|/S^3", "|c1|/S^2", "pair", "zeta"));
    for arm in &r.arms {
        let (a0, a1, a2) = arm.taus;
        let taus = py_tuple(&[&a0, &a1, &a2]);
        for (rf, a) in arm.refs() {
            let zt = a.zeta_range.0.expect("a ring is found");
            let zs: Vec<&dyn PyFormat> = a.zeros.iter().map(|x| x as &dyn PyFormat).collect();
            p.print(pyf!("    {:>20} {:>5} {:>7} {:11.2e} {:11.2e} {:>8} {:8.4f}",
                         taus.py_str(), rf, py_list(&zs).py_str(),
                         a.max_c0_rel.expect("rows were sampled"),
                         a.min_c1_rel.expect("rows were sampled"),
                         if a.all_complex.expect("rows were sampled") { "complex" } else { "real" },
                         zt));
        }
    }
    p.print("    det J = 0 under BOTH -- the two loops still on `phi` keep exactly PARALLEL rows,");
    p.print("    whatever the third watches. `c1` moves by more than TEN orders. And the freed root does");
    p.print("    NOT land on the real axis: zeta >= 1/sqrt(1-k) for EVERY bandwidth, so ONE");
    p.print("    SCALAR sets the split, the cyclic product AND the ring (rung 67's P, new");
    p.print("    mechanism). The window has edges -- the fast-fuel arm above is back on the axis,");
    p.print("    and the RANK does not care.");

    p.print("\n  THE LEDGER -- and its WHOLE SIGN TABLE flips with the reference:");
    let b = reference_bill(&t, flight, &ramp(0.005), sm, &TripleRigArm::default());
    p.print(pyf!("    {:>28} {:>16} {:>22}", "stator credit", "phi-referenced", "incidence-referenced"));
    let (cp, ci) = (&b.stator_credit_phi, &b.stator_credit_inc);
    for (lbl, xp, xi) in [(" alone, in phi", cp.alone, ci.alone),
                          (" alone, in incidence", cp.alone_inc, ci.alone_inc),
                          (" marginal, in phi", cp.marginal, ci.marginal),
                          (" marginal, in incidence", cp.marginal_inc, ci.marginal_inc)] {
        p.print(pyf!("    {:>28} {:15.2f}% {:21.2f}%", lbl, xp, xi));
    }
    p.print(pyf!("    The sharpest number: the incidence loop ALONE takes min phi_lp to {:.6f},",
                 b.inc.cell("S").min_phi));
    p.print(pyf!("    BELOW the bare march's own {:.6f} -- measurably worse than no limiter at all,",
                 b.inc.cell("bare").min_phi));
    p.print("    in the currency it does not watch. Rung 53's 'a margin is a DISTANCE' one level");
    p.print("    up again: rung 68 showed a credit needs its WALL named, this one shows it needs");
    p.print("    its loop's REFERENCE named too.");
    p.print("    SCOPE: v_max = 0.20 is INHERITED from rungs 57/58 and the loop SATURATES on it");
    p.print("    over 84% of the ramp when it runs alone, so the S/FS cells are authority-limited");
    p.print("    by a ceiling chosen elsewhere; all three tau are swept march coordinates; and");
    p.print("    the two floors are matched at the DESIGN setting, so the references are compared");
    p.print("    at equal WALL and not at equal excursion. See docs/rung69-spec.md.");
}

/// `print_cross_split_table(flight)` — rung 70.
pub fn cross_split_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE GENERIC SPLIT (rung 70): the split buys the RANK; the RING needs the odd");
    p.print("constraint to be a SECOND WALL ON THE SAME LEVER.");

    let (vm, tmax) = (0.20, 1200.0);
    let sm = PHI / FLOOR - 1.0;
    let lp = lp_map();
    let t = machine(build_cross_split_cascade, d, &LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp, B, sm, Some(0.05))),
        stator_lim: Some(StatorLimiter::from_margin(&lp, vm, sm, Some(0.05))),
        ..LeverArm::default()
    });
    p.print("  RUNG 67's SUBSTITUTION, APPLIED TO RUNG 68's TRIPLE: the odd loop's SENSOR moves");
    p.print(pyf!("  from `phi` to `Tt4` (rung 47's governor, Tt4_max = {:.0f} K -- rung 67's own", tmax));
    p.print("  imposed value). Same valve, same stator, same wall. n = 3, m = 2 -- RUNG 69's CELL.");

    // Every reader: r = 0.5, s_settle = 1.2, ds = 0.005, tau = tau_gov = tau_s = 0.05,
    // v_max = 0.20.
    let w = window_overlap(&t, flight, LO, HI, tmax, sm, 0.5, 1.2, 0.005, 0.05, 0.05, 0.05, 0.20);
    p.print(pyf!("\n  All three windows overlap on s in [{:.3f}, {:.3f}] ({} points, {:.1f}% of the march) -- a GATE,",
                 w.joint.0.expect("the windows overlap"), w.joint.1.expect("the windows overlap"),
                 w.joint.2, 100.0 * w.joint_fraction));
    p.print("  not a remark: a gain table over an empty intersection would report the algebra of");
    p.print("  loops that were never simultaneously live.");

    p.print("\n  THE IDENTITY MOVES -- and BOTH of rungs 68/69's readings stop being complete:");
    let g = split_gains(&t, flight, LO, HI, tmax, sm, 0.5, 1.2, 0.005, 0.05, 0.05, 0.05, 0.20, 20);
    p.print(pyf!("    {:>7} | {:>10} {:>10} {:>13} {:>10} | {:>10}",
                 "s", "R_q*C_g", "R_v*V_g", "C_v*V_q", "CYCLIC x", "R_q*C_g"));
    p.print(pyf!("    {:>7} | {:^47} | {:^10}",
                 "", "-- rung 70: the GOVERNOR is the odd loop --", "rung 68"));
    for row in &g.rows {
        let (a, f) = (&row.gov, &row.fuel);
        // the rung-68 contrast is only readable where ITS leg is interior too
        let fuel = if f.interior { pyf!("{:10.6f}", f.pair_rc) } else { pyf!("{:>10}", "off-regime") };
        p.print(pyf!("    {:7.4f} | {:10.6f} {:10.6f} {:13.10f} {:10.6f} | {}",
                     row.s, a.pair_rc, a.pair_rv, a.pair_cv, a.cyclic, fuel));
    }
    p.print(pyf!("    The SHARED pair is now (C,V) and holds at 1 to {:.1e}. At the SAME",
                 g.worst_cv.expect("rows were sampled")));
    p.print(pyf!("    base points rung 68's fuel leg still gives pair_RC = 1 to {:.1e} -- so the",
                 g.worst_rc_fuel.expect("rows were sampled")));
    p.print("    identity MOVED, measured on ONE trajectory. The two SPLIT pairs come back with");
    p.print("    OPPOSITE SIGNS, which no single scalar can summarise -- and the CYCLIC product");
    p.print(pyf!("    equals -pair_RC to {:.1e} while being BLIND to pair_RV.",
                 g.worst_cyclic_is_rc.expect("rows were sampled")));

    p.print("\n  THE SPECTRUM, and the FLOOR the split leaves behind:");
    let f = split_floor(&t, flight, LO, HI, tmax, sm,
                        &[(0.05, 0.05, 0.05), (0.05, 0.05, 0.10), (0.05, 0.05, 2.00)],
                        0.5, 1.2, 0.005, 0.20);
    p.print(pyf!("    {:>20} {:>12} {:>9} {:>9} {:>9}", "taus (g,q,s)", "quiet share", "zeta", "floor",
                 "pair"));
    for row in &f.rows {
        let Some(l) = &row.live else { continue };
        let (a0, a1, a2) = row.taus;
        p.print(pyf!("    {:>20} {:12.4f} {:9.5f} {:9.5f} {:>9}",
                     py_tuple(&[&a0, &a1, &a2]).py_str(), l.quiet_share,
                     l.zeta.expect("a live row prints its zeta"), l.floor,
                     if l.complex_pair { "COMPLEX" } else { "real" }));
    }
    p.print("    RUNG 69 FOUND A VISIBLE RING (zeta >= 0.61) because its `k ~ -1.7` was ONE LEVER");
    p.print("    READING TWO WALLS. Here the odd constraint sits on a DIFFERENT lever, both split");
    p.print("    pairs are cross-LEVER gains, and the floor lands at ~0.990 -- which is rung 67's");
    p.print("    own zeta = 1/sqrt(1+|P|), because min() selects pair_RC and pair_RC IS rung 67's");
    p.print("    P. A third loop that SHARES a constraint adds a zero and moves the achievable");
    p.print("    damping NOWHERE. The pre-registered 'no complex pair at ANY bandwidth' is");
    p.print("    REFUTED: the last row rings -- but only where the stator carries 1% of the rate");
    p.print("    sum, i.e. where the third loop is dynamically inert, and at zeta = 0.992 it is");
    p.print("    rung 67's admissible-but-unobservable mode again.");
    p.print("    SCOPE: that floor identity is CONTINGENT on pair_RV > 0 (had the stator's pair");
    p.print("    been the more negative one, min() would select a gain rung 67 never measured);");
    p.print("    Tt4_max/phi_lim/b_max/v_max are imposed or inherited; all three tau are swept");
    p.print("    march coordinates. ORDERINGS and SIGNS are the claims, MAGNITUDES are");
    p.print("    disclaimed. See docs/rung70-spec.md.");
}

/// `print_full_split_table(flight)` — rung 71.
pub fn full_split_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE FULL SPLIT (rung 71): a constraint can be INDEPENDENT IN RANK and REDUNDANT");
    p.print("ON THE BAND -- so `zeros = n - m` counts GRADIENTS, not LIVE loops.");

    let (vm, tmax) = (0.20, 1200.0);
    let sm = PHI / FLOOR - 1.0;
    let lp = lp_map();
    let t = machine(build_full_split_cascade, d, &LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp, B, sm, Some(0.05))),
        stator_inc: Some(StatorIncidenceLimiter::from_margin(&lp, vm, sm, Some(0.05))),
        ..LeverArm::default()
    });
    p.print("  RUNG 69's MOVE, APPLIED TO RUNG 70's PLANT: the stator's REFERENCE goes from `phi`");
    p.print("  to INCIDENCE, beside rung 47's governor and rung 65's valve. Three loops, THREE");
    p.print("  constraints -- n = m = 3, and the actuator block is INVERTIBLE for the first time.");

    p.print("\n  THE CONTAINMENT, and it needs no new constant:");
    p.print("    phi = phi_lim  =>  M_i = T_c - 1/phi_lim + v = m_lim + v >= m_lim  for all v >= 0");
    // Every reader: r = 0.5, s_settle = 1.2, tau = tau_gov = tau_s = 0.05, v_max = 0.20, and
    // ds = 0.005 — except `full_gains`, whose own default is 0.002.
    let bc = band_containment(&t, flight, LO, HI, tmax, sm, 0.5, 1.2, 0.005, 0.05, 0.05, 0.05,
                              0.20);
    p.print("    so {phi >= phi_lim} INTERSECT the band [0, v_max] sits INSIDE {M_i >= m_lim}.");
    p.print(pyf!("    Measured on the march: of {} points where the valve DELIVERS,", bc.n_delivering));
    p.print(pyf!("    the stator rides at {} of them, and the slack minus v bottoms out at",
                 bc.riding_while_delivering));
    p.print(pyf!("    exactly {:.1f} (it IS 1/phi_lim - 1/phi, zero where the valve pins the floor).",
                 bc.worst_slack_minus_v.expect("the valve delivers")));

    p.print("\n  SO THE THIRD LOOP'S WINDOW IS THE SECOND LOOP'S LAG -- swept from both sides:");
    let wl = window_law(&t, flight, LO, HI, tmax, sm, &[0.005, 0.05, 0.20, 0.50, 2.00],
                        &[0.005, 0.05, 0.20, 0.50], 0.5, 1.2, 0.005, 0.05, 0.05, 0.05, 0.20);
    p.print(pyf!("    {:>8} {:>14}   |   {:>8} {:>14}", "tau_q", "rides to s =", "tau_s",
                 "rides to s ="));
    for i in 0..wl.tau_qs.len().max(wl.tau_ss.len()) {
        let left = if i < wl.tau_qs.len() {
            pyf!("{:8.3f} {:14.3f}", wl.tau_qs[i], wl.edge_q[i].expect("the stator rides"))
        } else {
            pyf!("{:>23}", "")
        };
        let right = if i < wl.tau_ss.len() {
            pyf!("{:8.3f} {:14.3f}", wl.tau_ss[i], wl.edge_s[i].expect("the stator rides"))
        } else {
            String::new()
        };
        p.print(pyf!("    {}   |   {}", left, right));
    }
    p.print(pyf!("    Monotone in the VALVE's clock over {:.2f}x; a {:.2f}x NON-monotone band in its own.",
                 wl.q_span.expect("the edges exist"), wl.s_span.expect("the edges exist")));
    p.print(pyf!("    The stator rides over {:.1f}% of the march (rung 70's stator: 24.3%) -- THIS rung's number.",
                 (100 * wl.base.stator.2) as f64 / wl.base.n as f64));
    p.print(pyf!("    The JOINT window is thinner still, {:.1f}%, but that is the 7.9% intersected with a",
                 100.0 * wl.joint_fraction));
    p.print("    governor that does not engage until s = 0.105 -- rung 67's imposed Tt4_max, NOT");
    p.print("    containment. Containment owns where the stator's window ENDS; Tt4_max owns where");
    p.print("    the joint one STARTS.");

    p.print("\n  THE DETERMINANT IS ALIVE FOR THE FIRST TIME -- AND STILL BLIND TO ONE GAIN:");
    let g = full_gains(&t, flight, LO, HI, tmax, sm, 0.5, 1.2, 0.002, 0.05, 0.05, 0.05, 0.20, 3);
    p.print(pyf!("    {:>7} | {:>10} {:>10} {:>10} | {:>10} {:>14}",
                 "s", "R_q*C_g", "R_v*V_g", "C_v*V_q", "det M", "-(1-RC)(1-CV)"));
    for row in &g.rows {
        let a = &row.gains;
        p.print(pyf!("    {:7.4f} | {:10.6f} {:10.6f} {:10.6f} | {:10.6f} {:14.6f}",
                     row.s, a.pair_rc, a.pair_rv, a.pair_cv, row.det, row.det_pred));
    }
    p.print(pyf!("    NO pair is 1 (closest: {:.3f} away) -- rung 66's identity is a",
                 g.closest_to_1.expect("rows were sampled")));
    p.print("    property of a SHARED constraint and nothing is shared. `pair_RC` IS rung 67's P");
    p.print("    and `pair_CV` IS rung 69's k, so only the middle column is new -- and the");
    p.print(pyf!("    determinant FACTORS into the other two to {:.1e}, because pair_RV cancels",
                 g.worst_det_err.expect("rows were sampled")));
    p.print(pyf!("    against the reverse cyclic product (to {:.1e}). ONE FACTOR PER RUNG.",
                 g.worst_y_is_rv.expect("rows were sampled")));

    p.print("\n  AND THE s = 0 FIXED POINT BECOMES A POINT:");
    let ic = ic_contraction(&t, flight, LO, HI, tmax, sm,
                            &["gqv", "gvq", "qgv", "qvg", "vgq", "vqg"], &[0.0, 0.25, 0.6, 1.0],
                            0.5, 1.2, 0.005, 0.05, 0.05, 0.05, 0.20);
    for (dd, lbl) in [(&ic.full, "rung 71 (n = m = 3)"), (&ic.shared, "rung 70 (n = 3, m = 2)")] {
        p.print(pyf!("    {:>24}: {} starts x orders -> {} limit point(s), spread {:.3e}",
                     lbl, dd.n, dd.members, max3(dd.spread.expect("a start converged"))));
    }
    p.print("    Rung 69 s 6 called a null space a SHOCK ABSORBER. At nullity ZERO there is");
    p.print("    nothing to absorb with, and the sweep REJECTS a moved start instead.");

    p.print("\n  THE LEDGER, IN THREE CURRENCIES -- one per loop:");
    let b = full_bill(&t, flight, LO, HI, tmax, sm, 0.5, 1.2, 0.005, 0.05, 0.05, 0.05, 0.20);
    p.print(pyf!("    {:>5} {:>12} {:>10} {:>12} {:>9} {:>9}",
                 "cell", "I (phi)", "E (Tt4)", "M (incid.)", "min phi", "max Tt4"));
    for (k, c) in &b.cells {
        p.print(pyf!("    {:>5} {:12.4e} {:10.3f} {:12.4e} {:9.6f} {:9.2f}",
                     k, c.i, c.e, c.m, c.min_phi, c.max_tt4));
    }
    p.print("    Each loop's MARGINAL share of its OWN currency, against its SOLO one:");
    for (leg, kept) in [("gov", b.kept.gov), ("valve", b.kept.valve), ("stator", b.kept.stator)] {
        p.print(pyf!("      {:>7}: kept {:7.1f} %", leg, 100.0 * kept.expect("a solo credit exists")));
    }
    p.print("    RUNG 70 s 5 SAID *a loop is eroded by the loops it SHARES a constraint with, and");
    p.print("    by no others*. NO TWO LOOPS SHARE HERE and the stator keeps a few per cent --");
    p.print("    so erosion has a SECOND channel: a loop is eroded by any loop that pushes its");
    p.print("    constraint into the SLACK region. That is the containment again, integrated.");
    p.print(pyf!("    The sharpest number: the VALVE, which cannot see M_i at all, delivers {:.1f} %",
                 100.0 * b.inc_credit_valve_alone.expect("the bare cell has an incidence bill")));
    p.print(pyf!("    of the incidence credit running ALONE, against the incidence stator's own {:.1f} %.",
                 100.0 * b.inc_credit_stator_alone.expect("the bare cell has an incidence bill")));
    p.print("    SCOPE: the joint window is ~2% of the march and every gain table lives inside");
    p.print("    it; the CONTAINMENT is contingent on the two walls being MATCHED at the design");
    p.print("    setting (rung 69's choice) -- only the RANK half of the headline is general;");
    p.print("    Tt4_max is rung 67's imposed value, phi_lim/b_max/v_max rungs 64/57/58's;");
    p.print("    the determinant's FACTORING is contingent on grad(M_i) = sigma grad(phi) + e_v;");
    p.print("    all three tau are swept march coordinates. See docs/rung71-spec.md.");
}
