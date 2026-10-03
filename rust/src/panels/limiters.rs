//! Rungs 46–52's panels — the fuel-side limiter family (slice AO).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line for
//! line. Every panel builds the same two-spool fuel transient (rung 43's plant on the flow/press
//! map pair, `rho = 1`); Python's keyword defaults are spelled positionally at each call, and
//! `min(traj, key=…)` / `max(traj, key=…)` keep Python's FIRST-on-tie rule (a strict compare).

use super::twospool::{cpg13, ts_design};
use super::Design;
use crate::fuel_transient::{AsymmetricLag, FuelLimiters, FuelPoint, SurgeLimiter, TwoSpoolFuelTransient};
use crate::gas::Gas;
use crate::map::ComponentMap;
use crate::pyf;
use crate::pyfmt::{Printer, PyFormat};
use crate::two_spool::Spool;

/// `LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)`.
fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::default() }
}

/// `HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)`.
fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::default() }
}

/// `TwoSpoolFuelTransient(design, flight, 1.0, map_lp=LP, map_hp=HP, rho=1.0)` on a fresh
/// `build_two_spool_turbojet(gas, 3.0, 6.0, …)`.
fn fuel_ft(gas: Gas, d: &Design) -> TwoSpoolFuelTransient {
    TwoSpoolFuelTransient::new(ts_design(gas, 3.0, 6.0, d), d.flight, 1.0, lp_map(), hp_map(), 1.0)
}

/// Python's `min(tj, key=…)`: the FIRST point at the minimum.
fn first_min(tj: &[FuelPoint], key: impl Fn(&FuelPoint) -> f64) -> &FuelPoint {
    let mut best = &tj[0];
    for p in &tj[1..] {
        if key(p) < key(best) {
            best = p;
        }
    }
    best
}

/// Python's `max(tj, key=…)`: the FIRST point at the maximum.
fn first_max(tj: &[FuelPoint], key: impl Fn(&FuelPoint) -> f64) -> &FuelPoint {
    let mut best = &tj[0];
    for p in &tj[1..] {
        if key(p) > key(best) {
            best = p;
        }
    }
    best
}

/// `print_topping_governor_table(flight)` — rung 46.
pub fn topping_governor_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTIT topping governor (rung 46): clip fuel to hold Tt4 <= redline (the first fuel-side");
    p.print("FEEDBACK). Enforcing the TIT limit SPLITS the surge relief -- it rebates the late HP spool");
    p.print("but MISSES the early, binding LP one. Rung 35's coupled limits are SEQUENCED in time.");

    let (lp, hp) = (lp_map(), hp_map());
    let design = ts_design(Gas::thermally_perfect(), 3.0, 6.0, d);
    let ft = TwoSpoolFuelTransient::new(design.clone(), *flight, 1.0, lp, hp, 1.0);

    // --- THE MECHANISM: the LP surge min LEADS (low Tt4), the Tt4 peak LAGS.
    let (tj, _) = ft.core().fuel_ramp_march(flight, 1000.0, 1400.0, 0.5, 2.0, 0.02,
                                            &FuelLimiters::default());
    let lpm = first_min(&tj, |q| q.phi_lp);
    let hpm = first_min(&tj, |q| q.phi_hp);
    let pk = first_max(&tj, |q| q.tt4);
    p.print("\n  THE WINDOW (flow/press, accel Tt4 1000->1400, r=0.5). The limits are SEQUENCED:");
    p.print(pyf!("    LP surge min : s={:.2f}  Tt4={:6.1f}  (DURING the ramp, below any redline)",
                 lpm.s, lpm.tt4));
    p.print(pyf!("    HP surge min : s={:.2f}  Tt4={:6.1f}  (LATE, inside the clip window)",
                 hpm.s, hpm.tt4));
    p.print(pyf!("    Tt4 peak     : s={:.2f}  Tt4={:6.1f}", pk.s, pk.tt4));

    // --- THE SPLIT: enforce the redline, difference the surge object per spool.
    p.print("\n  THE SPLIT (redline Tt4_max=1480, in the gap above the 1400 endpoint). relief>0 = safer:");
    p.print(pyf!("  {:>12}{:>12}{:>12}{:>8}", "shape", "relief_lp", "relief_hp", "held"));
    for (name, ml, mh) in [("flow/press", lp, hp), ("hp-only", ComponentMap::flat(), hp)] {
        let rr = TwoSpoolFuelTransient::new(design.clone(), *flight, 1.0, ml, mh, 1.0)
            .topping_relief(flight, 1000.0, 1400.0, 1480.0, 0.5, 2.0, 0.02, None);
        p.print(pyf!("  {:>12}{:>12.5f}{:>12.5f}{:>8}", name, rr.relief_lp, rr.relief_hp,
                     rr.held.py_str()));
    }
    p.print("  => LP relief MACHINE-ZERO (the clip never reaches its early minimum), HP relief >0.");
    p.print("     Holds even on hp-only (LP flat, NO complex mode) -- the pure WINDOW mechanism.");

    // --- THE LEVER: fast ramp migrates the LP min INTO the clip window.
    p.print("\n  THE LEVER (Tt4_max=1440): a FASTER ramp lifts the LP surge min above the redline ->");
    p.print("  relief_lp switches ON (the governor becomes a modest LP-surge lever where it's needed):");
    let head: String = [0.5, 0.3, 0.15].iter()
        .map(|x: &f64| pyf!("{:>10}", format!("r={}", x.py_str()))).collect();
    p.print(pyf!("  {:>12}", "") + &head);
    let row: String = [0.5, 0.3, 0.15].iter()
        .map(|&r| pyf!("{:10.5f}",
                       ft.topping_relief(flight, 1000.0, 1400.0, 1440.0, r, 2.0, 0.02, None).relief_lp))
        .collect();
    p.print(pyf!("  {:>12}", "relief_lp:") + &row);
    p.print("  Tt4_max imposed -> no redline-level claim; the SPLIT sign is load-bearing. Reduce: dormant");
    p.print("  (redline above the peak) -> rung 45/43 bit-for-bit; decel never fires; cycle rung-6 exact.");
}

/// `print_lagged_governor_table(flight)` — rung 47.
pub fn lagged_governor_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nLagged topping governor (rung 47): give rung 46's governor a response lag tau_gov. A");
    p.print("first-order lag is a TRAILING-edge tool -- it REFUTES rung 46's 'a slow governor reaches");
    p.print("earlier into the LP surge point'. It breaks the redline hold and STILL misses the LP.");

    let ft = fuel_ft(cpg13(), d);
    // `("inst", t1, t2, t3)` each formatted `>10` — a str, then floats by their `str()`.
    let taus_head = |ts: [f64; 3]| -> String {
        pyf!("{:>10}", "inst") + &ts.iter().map(|t| pyf!("{:>10}", t)).collect::<String>()
    };

    // --- THE COST OF REALISM: overshoot grows, HP rebate erodes, relief_lp pinned at 0.
    p.print("\n  THE COST OF REALISM (flow/press, accel 1000->1400, r=0.5, redline 1480). Sweep tau_gov:");
    p.print(pyf!("  {:>12}", "tau_gov:") + &taus_head([0.05, 0.2, 0.8]));
    let (mut ov, mut rlp, mut rhp) = (Vec::new(), Vec::new(), Vec::new());
    for tau in [None, Some(0.05), Some(0.2), Some(0.8)] {
        let o = ft.topping_relief(flight, 1000.0, 1400.0, 1480.0, 0.5, 2.0, 0.02, tau);
        ov.push(o.overshoot);
        rlp.push(o.relief_lp);
        rhp.push(o.relief_hp);
    }
    let join = |vs: &[f64], spec: &str| -> String { vs.iter().map(|v| pyf!(spec, v)).collect() };
    p.print(pyf!("  {:>12}", "overshoot:") + &join(&ov, "{:10.1f}") + "   <- redline HOLD lost (grows)");
    p.print(pyf!("  {:>12}", "relief_lp:") + &join(&rlp, "{:10.5f}") + "   <- STILL 0 (misses early LP)");
    p.print(pyf!("  {:>12}", "relief_hp:") + &join(&rhp, "{:10.5f}") + "   <- HP rebate ERODES toward 0");

    // --- THE LEVER, LAGGED: at fast r the lag ERODES rung 46's positive LP relief, never enhances.
    p.print("\n  THE LEVER, LAGGED (r=0.15, redline 1440). rung 46's INSTANT governor reaches the LP");
    p.print("  (relief_lp>0); if a lag 'reached earlier' it would EXCEED that. It ERODES it instead:");
    p.print(pyf!("  {:>12}", "tau_gov:") + &taus_head([0.05, 0.2, 0.4]));
    let lev: String = [None, Some(0.05), Some(0.2), Some(0.4)].iter()
        .map(|&t| pyf!("{:10.5f}",
                       ft.topping_relief(flight, 1000.0, 1400.0, 1440.0, 0.15, 2.0, 0.02, t).relief_lp))
        .collect();
    p.print(pyf!("  {:>12}", "relief_lp:") + &lev + "   <- eroded toward 0, never enhanced");
    p.print("  => neutral at moderate r, strictly WORSE at fast r -- in NO regime does the lag reach the");
    p.print("     LP better than the ideal. You can't cure a leading-edge problem with a trailing-edge tool.");

    // --- THE SECONDARY: the overshoot lives in the LOOP lag, not the valve.
    let tc = ft.core().topping_command_trace(flight, 1000.0, 1400.0, 1480.0, 0.5, 2.0, 0.02);
    p.print(pyf!("\n  WHERE THE LAG LIVES: the binding topping command is monotone-rising ({}, {} pts).",
                 tc.monotone_nondecreasing.py_str(), tc.n_engaged));
    p.print("  So a metering-VALVE lag is INERT (instant-up tracks it); the overshoot lives in the");
    p.print("  sensing/LOOP lag. tau_gov=None -> rung 46 bit-for-bit; dormant/decel -> rung 45; cycle rung-6.");
}

/// `print_accel_schedule_table(flight)` — rung 48.
pub fn accel_schedule_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nWf/pt3 accel schedule (rung 48): the FEEDFORWARD leg -- cap fuel by the compressor");
    p.print("delivery pressure. It watches the INPUT, not the output, so it can engage EARLY. Sweeping");
    p.print("its margin m sweeps the ENGAGEMENT TIME across both surge minima -- and the relief per");
    p.print("spool switches on IFF the clip lands UPSTREAM of that spool's own minimum.");

    let ft = fuel_ft(cpg13(), d);
    let core = ft.core();

    // --- THE WINDOW: the bare ratio is already far above the steady line UPSTREAM of the LP min.
    let (tj, _) = core.fuel_ramp_march(flight, 1000.0, 1400.0, 0.5, 2.0, 0.02,
                                       &FuelLimiters::default());
    let acc0 = core.accel_schedule(flight, 1000.0, 1400.0, 0.0, 13);
    let s_lp = first_min(&tj, |q| q.phi_lp).s;
    let pi_b = core.inner.inner.base.pi_b;
    let ratio = |q: &FuelPoint| -> f64 {
        let i = core.instant_fuel(flight, q.nu_lp, q.nu_hp, q.mf);
        q.mf / acc0.cap(i.base.close.n_hp, i.base.close.pt4 / pi_b)
    };
    // `tj[:26:5]`
    let picks: Vec<&FuelPoint> = tj.iter().take(26).step_by(5).collect();

    p.print("\n  THE WINDOW (flow/press, accel 1000->1400, r=0.5). The bare (Wf/pt3)/kappa_ss ratio");
    p.print("  rises MONOTONICALLY and clears the steady line LONG before the LP surge min:");
    p.print(pyf!("  {:>12}", "s:") + &picks.iter().map(|q| pyf!("{:>9.2f}", q.s)).collect::<String>());
    p.print(pyf!("  {:>12}", "ratio:")
            + &picks.iter().map(|q| pyf!("{:>9.3f}", ratio(q))).collect::<String>()
            + &pyf!("   <- LP surge min at s={:.2f}", s_lp));

    // --- THE CROSSING: sweep m, watch each spool's relief die at ITS OWN minimum.
    let rows = core.engagement_sweep(flight, 1000.0, 1400.0, &[0.15, 0.35, 0.42, 0.45, 0.48],
                                     0.5, 4.0, 0.02, 13);
    p.print(pyf!("\n  THE CROSSING (s_lp*={:.2f}, s_hp*={:.2f}). relief>0 = safer; fuel_rm = fuel removed:",
                 rows[0].s_lp_bare, rows[0].s_hp_bare));
    p.print(pyf!("  {:>6}{:>8}{:>12}{:>12}{:>10}{:>10}",
                 "m", "s_eng", "relief_lp", "relief_hp", "fuel_rm", "nu_H end"));
    for x in &rows {
        p.print(pyf!("  {:6.2f}{:8.2f}{:12.6f}{:12.6f}{:10.5f}{:10.5f}",
                     x.margin, x.s_eng, x.relief_lp, x.relief_hp, x.fuel_removed, x.nu_hp_end));
    }
    p.print(pyf!("  => relief_lp dies EXACTLY as s_eng passes s_lp*={:.2f} -- while relief_hp is still",
                 rows[0].s_lp_bare));
    p.print(pyf!("     positive, dying only as s_eng reaches s_hp*={:.2f}. ONE rule, TWO crossings.",
                 rows[0].s_hp_bare));
    p.print("  NOT rung 44's ramp-rate lever: fuel is STILL being removed where the LP gets exactly");
    p.print("  nothing, the endpoint is unmoved, and one clip splits the two spools. m and phi_surge");
    p.print("  imposed -> no level claim. Reduces: accel=None -> rungs 45/46/47 bit-for-bit; a dormant");
    p.print("  schedule/decel -> rung 45; the two-leg min-select -> the single leg; cycle rung-6 exact.");
}

/// `print_phi_limiter_table(flight)` — rung 49.
pub fn phi_limiter_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nphi surge-margin limiter (rung 49): the first leg that watches the PROTECTED");
    p.print("variable. Its window CLOSES INSIDE the ramp -- the object no pt3 filter can build --");
    p.print("and that closing edge DEBITS the spool it is not watching.");

    let ft = fuel_ft(cpg13(), d);
    let core = ft.core();

    // --- THE SPLIT: one clip, two signs; and both window edges reported.
    let rows = core.floor_sweep(flight, 1000.0, 1400.0, &[0.7550, 0.7500, 0.7450, 0.7400],
                                Spool::Lp, 0.5, 2.0, 0.02);
    let (s_lp, s_hp) = (rows[0].s_lp_bare, rows[0].s_hp_bare);
    p.print(pyf!("\n  WATCHING THE LP (accel 1000->1400, r=0.5; bare minima s_lp*={:.2f}, s_hp*={:.2f}).",
                 s_lp, s_hp));
    p.print("  Every row engages UPSTREAM of s_hp* -- rung 48's law predicts a CREDIT on the HP:");
    p.print(pyf!("  {:>8}{:>7}{:>7}{:>9}{:>12}{:>12}{:>10}",
                 "phi_lim", "s_eng", "s_rel", "in ramp", "relief_lp", "relief_hp", "fuel_rm"));
    for x in &rows {
        p.print(pyf!("  {:8.4f}{:7.2f}{:7.2f}{:>9}{:12.6f}{:12.6f}{:10.5f}",
                     x.phi_lim, x.s_eng, x.s_rel, x.both_edges_inside_ramp.py_str(), x.relief_lp,
                     x.relief_hp, x.fuel_removed));
    }
    p.print("  => the WATCHED spool is credited and the OTHER one is DEBITED, by the same clip.");
    p.print("     The unwatched minimum relocates to just AFTER s_rel (not to s_eng): that is");
    p.print("     where the withheld fuel is handed back to a still-ramping plant.");

    // --- THE DISCRIMINATOR, in miniature: push the release far past the ramp and the sign flips.
    let fast = core.floor_sweep(flight, 1000.0, 1400.0, &[0.7500], Spool::Lp, 0.15, 2.0, 0.02)
        .remove(0);
    p.print(pyf!("\n  THE CLOCK: at r=0.15 the SAME floor releases at s_rel={:.2f} = {:.1f}x the ramp,",
                 fast.s_rel, fast.s_rel / 0.15));
    p.print(pyf!("  far past the minima -- and the unwatched relief FLIPS to {:+.6f}. The credit is",
                 fast.relief_hp));
    p.print("  clocked by the spool's OWN minimum (rung 48); the debit by the RAMP END (rung 44)");
    p.print("  -- a WITHIN-FAMILY result. So rung 48 is BOUNDED, not refuted: its credit term");
    p.print("  survives verbatim and an HP floor reproduces its exact zero. Its own leg is");
    p.print("  empirically immune to the debit; WHY is an open seam (the ratio does not transfer).");
    p.print("  phi_lim rides rung 36/41's imposed phi_surge -> signs, not magnitudes.");
    p.print("  Reduces: surge=None -> rungs 45/46/47/48 bit-for-bit; dormant floor/decel -> rung 45;");
    p.print("  the min-select composite -> the single binding leg; cycle rung-6 exact.");
}

/// `print_release_edge_table(flight)` — rung 50.
pub fn release_edge_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nThe release edge, ISOLATED (rung 50): force the leg to let go at s_off, with the");
    p.print("engagement edge and the clip depth held FIXED -- the axis rung 49 could not build.");

    let ft = fuel_ft(cpg13(), d);
    let core = ft.core();

    // --- THE HEADLINE + THE DISCRIMINATOR: r=2.0 puts s_hp* and the ramp end 3.1x apart.
    let lim = SurgeLimiter::new(Spool::Lp, 0.7725);
    let rows = core.release_sweep(flight, 1000.0, 1400.0,
                                  &[0.30, 0.66, 1.10, 1.56, 1.80, 2.06, 2.20],
                                  Some(&lim), None, 2.0, 2.0, 0.02);
    let (s_lp, s_hp) = (rows[0].s_lp_bare, rows[0].s_hp_bare);
    p.print(pyf!("\n  WATCHING THE LP, phi_lim=0.7725, accel 1000->1400 at r=2.0 (bare minima s_lp*={:.2f}, s_hp*={:.2f};",
                 s_lp, s_hp));
    p.print("  the ramp end is 2.00, so the two candidate clocks sit 3.1x apart):");
    p.print(pyf!("  {:>7}{:>7}{:>7}{:>12}{:>12}{:>9}{:>9}{:>10}",
                 "s_off", "s_eng", "s_rel", "relief_lp", "relief_hp", "s@minLP", "s@minHP", "fuel_rm"));
    for x in &rows {
        p.print(pyf!("  {:7.2f}{:7.2f}{:7.2f}{:12.5f}{:12.5f}{:9.2f}{:9.2f}{:10.5f}",
                     x.s_off.expect("release_sweep passes s_off"), x.s_eng, x.s_rel, x.relief_lp,
                     x.relief_hp, x.s_min_lp, x.s_min_hp, x.fuel_removed));
    }
    p.print("  => s_eng is IDENTICAL in every row, so this axis moves ONLY the release edge.");
    p.print("     BOTH spools' minima sit AT the release point (the two exceptions are the law's");
    p.print("     preconditions: row 1 releases upstream of s_hp*, row 7's LP credit branch wins).");
    p.print("     The debit walks straight THROUGH s_hp* without noticing it and peaks with the");
    p.print("     release near the RAMP END -- so rung 49's within-family clock hedge LIFTS.");
    p.print("     And it is not a ramp-rate lever: the LAST row removes the MOST fuel for less");
    p.print("     than half the peak debit (monotone removal, PEAKED debit).");

    // --- THE SEAM: rung 48's own leg, clip shape unchanged, forced to release inside the ramp.
    let acc = core.accel_schedule(flight, 1000.0, 1400.0, 0.25, 13);
    let seam = core.release_sweep(flight, 1000.0, 1400.0, &[0.30, 0.44, 0.50, 9.90],
                                  None, Some(&acc), 0.5, 2.0, 0.02);
    p.print("\n  THE SEAM rung 49 left open -- rung 48's OWN leg (Wf/pt3, m=0.25) at r=0.5,");
    p.print("  clip SHAPE unchanged, only WHEN it lets go:");
    p.print(pyf!("  {:>7}{:>7}{:>12}{:>12}", "s_off", "s_rel", "relief_lp", "relief_hp"));
    for x in &seam {
        p.print(pyf!("  {:7.2f}{:7.2f}{:12.5f}{:12.5f}",
                     x.s_off.expect("release_sweep passes s_off"), x.s_rel, x.relief_lp, x.relief_hp));
    }
    p.print("  => left alone it releases POST-ramp and CREDITS both spools (rung 48's finding).");
    p.print("     Forced inside the ramp the SAME leg DEBITS both. Its immunity is TIMING, not");
    p.print("     clip SHAPE -- rung 49's named suspect is refuted and the seam closes.");
    p.print("  s_off is an ISOLATION DIAGNOSTIC (freeze='lp' tradition), not a control law: it");
    p.print("  licenses the reading of rungs 48/49's real legs. phi_lim/m ride rungs 36/41/48/49's");
    p.print("  imposed constants -> signs, ordering and ds-convergence, not magnitudes.");
    p.print("  Reduces: s_off=None -> rungs 43/45/46/47/48/49 bit-for-bit; a late s_off is inert;");
    p.print("  an s_off before s_eng is float-for-float BARE; cycle rung-6 exact.");
}

/// `print_release_rate_table(flight)` — rung 51.
pub fn release_rate_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nThe release RATE (rung 51): fade the clip over [s_off, s_off+tau_rel] instead of");
    p.print("dropping it. Rung 50 moved WHEN the fuel comes back; this moves HOW FAST.");

    let ft = fuel_ft(cpg13(), d);
    let core = ft.core();
    let lim = SurgeLimiter::new(Spool::Lp, 0.7725);
    // `K = dict(r=2.0, s_settle=2.0)`, `ds=0.02` by default.
    let rel = |s_off: f64, surge: Option<&SurgeLimiter>,
               accel: Option<&crate::fuel_transient::AccelSchedule>, tau_rel: Option<f64>| {
        core.release_relief(flight, 1000.0, 1400.0, Some(s_off), surge, accel, 2.0, 2.0, 0.02, tau_rel)
    };
    let show = |p: &mut Printer, tag: &str, x: crate::fuel_transient::ReleaseRelief| {
        p.print(pyf!("  {:<26}{:11.6f}{:12.5f}{:12.5f}{:9.2f}",
                     tag, x.fuel_removed, x.relief_lp, x.relief_hp, x.s_min_hp));
    };
    let head = || pyf!("  {:<26}{:>11}{:>12}{:>12}{:>9}",
                       "placement", "fuel_rm", "relief_lp", "relief_hp", "s@minHP");

    p.print("\n  THE BRACKET (phi floor 0.7725, accel 1000->1400 at r=2.0). The faded row's fuel");
    p.print("  is POINTWISE between the two hard rows, and so is its total removal:");
    p.print(head());
    show(p, "hard   s_off=1.56", rel(1.56, Some(&lim), None, None));
    show(p, "FADED  1.56, tau_rel=0.20", rel(1.56, Some(&lim), None, Some(0.20)));
    show(p, "hard   s_off=1.76", rel(1.76, Some(&lim), None, None));
    p.print("  => the two HARD rows AGREE (postponing a step release over that interval does");
    p.print("     essentially nothing) -- yet the faded row between them is ~1.5x SHALLOWER, on");
    p.print("     BOTH spools. No monotone functional of the fuel LEVEL, and no function of the");
    p.print("     TOTAL DEFICIT, can land outside a bracket it sits inside. The debit answers to");
    p.print("     the RATE => rung 50's deficit law is BOUNDED to the instantaneous hand-back.");

    p.print("\n  THE SCOPE (a negative, stated here and gated): at s_off=0.30 the same");
    p.print("  construction INTERPOLATES -- deep dives only. There, rate and deficit are not");
    p.print("  separable and nothing is claimed:");
    p.print(head());
    show(p, "hard   s_off=0.30", rel(0.30, Some(&lim), None, None));
    show(p, "FADED  0.30, tau_rel=0.20", rel(0.30, Some(&lim), None, Some(0.20)));
    show(p, "hard   s_off=0.50", rel(0.50, Some(&lim), None, None));

    let acc = core.accel_schedule(flight, 1000.0, 1400.0, 0.15, 13);
    p.print("\n  CROSS-FAMILY (rung 48's Wf/pt3 leg, m=0.15) -- the violation flips the SIGN:");
    p.print(head());
    show(p, "hard   s_off=1.10", rel(1.10, None, Some(&acc), None));
    show(p, "FADED  1.10, tau_rel=0.40", rel(1.10, None, Some(&acc), Some(0.40)));
    show(p, "hard   s_off=1.50", rel(1.50, None, Some(&acc), None));
    p.print("  => relief_lp is EXACTLY 0 throughout: rung 48's exact-zero law survives the rate");
    p.print("     axis, three rungs on. NOT claimed: that a slow hand-back is a way to BUILD");
    p.print("     immunity -- tau_rel fades a clip already forced off at an arbitrary time.");
    p.print("  Also corrected here: rung 50's precondition (a) ('the release must land at or");
    p.print("  after that spool's bare minimum') is MIS-STATED -- the crossover sits UPSTREAM of");
    p.print("  it, and rung 50's own table already violated the condition. Its relocation");
    p.print("  headline is untouched. NEXT SEAM: the asymmetric fast-attack/slow-release LAG,");
    p.print("  deferred because its release edge is EMERGENT and moves with the rate.");
    p.print("  Reduces: tau_rel=None or 0.0 -> rungs 43/45/46/47/48/49/50 bit-for-bit; tau_rel");
    p.print("  without s_off ASSERTS; a trigger past the natural release is inert; cycle rung-6.");
}

/// `print_asymmetric_lag_table(flight)` — rung 52.
pub fn asymmetric_lag_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nThe asymmetric fast-attack/slow-release LAG (rung 52): the clip AMOUNT a state with");
    p.print("TWO time constants. The physically-realisable limiter rungs 50/51 imitated by force.");

    let ft = fuel_ft(cpg13(), d);
    let core = ft.core();
    let lim = SurgeLimiter::new(Spool::Lp, 0.7725);
    // `lag_relief`'s default `eps=(0.05, 0.01)`; `K = dict(r=2.0, s_settle=2.0)`, `ds=0.02`.
    let eps = [0.05, 0.01];

    p.print("\n  THE TRIGGER PINS ITSELF (phi floor 0.7725, accel 1000->1400 at r=2.0,");
    p.print("  tau_att=0.02). Sweep the RELEASE constant over 20x -- the crossing does not move:");
    p.print(pyf!("  {:>9}{:>9}{:>12}{:>12}{:>12}{:>11}",
                 "tau_rel", "s_cross", "s_rel(.01)", "credit_LP", "debit_HP", "fuel_rm"));
    let mut base = None;
    for tr in [0.02, 0.10, 0.40] {
        let x = core.lag_relief(flight, 1000.0, 1400.0, AsymmetricLag::new(0.02, tr), Some(&lim),
                                None, 2.0, 2.0, 0.02, &eps);
        // `x['s_rel_0.01']` — the LAST point whose fractional clip is >= 0.01.
        let s_rel_01 = x.eps_edges.iter().find(|e| e.0 == 0.01).expect("eps 0.01").2;
        p.print(pyf!("  {:9.2f}{:9.2f}{:12.2f}{:12.6f}{:12.6f}{:11.5f}",
                     tr, x.s_cross, s_rel_01, x.relief_watched.expect("surge leg"),
                     x.relief_other.expect("surge leg"), x.fuel_removed));
        if base.is_none() {
            base = Some(x);
        }
    }
    let base = base.expect("three rows");
    p.print("  => s_cross IDENTICAL and credit_LP identical to MACHINE ZERO, while the hand-back");
    p.print("     itself stretches out. tau_rel is never READ before the crossing, so the whole");
    p.print("     pre-crossing march is bit-identical -- the leg pins its own trigger, which is");
    p.print("     the property rung 50 had to FORCE with s_off. Rung 51's reason 1 is REFUTED.");
    p.print("  => and the debit SHRINKS while fuel_rm RISES: more fuel removed, smaller debit.");
    p.print("     Rung 51's headline, on a realisable leg (and far enough out it flips to CREDIT).");

    p.print("\n  WHY THE CREDIT'S ZERO IS NOT A TAUTOLOGY: it needs the watched spool's own");
    p.print("  minimum to sit UPSTREAM of the crossing. It does -- the lag's undershoot is");
    p.print("  largest EARLY, while g is still climbing, so rung 48's arrest pins that minimum at");
    p.print(pyf!("  the engagement edge: s_min_LP={:.2f} vs s_cross={:.2f} (bare min at 0.32).",
                 base.s_min_lp, base.s_cross));
    p.print("  => A SELF-RELEASING LIMITER CANNOT DEBIT THE SPOOL IT WATCHES. Rung 50's");
    p.print("     watched-side debit is an ARTIFACT OF FORCING; rung 49's identity is RESTORED");
    p.print("     for every realisable leg, and rung 50's bound re-scoped to its own instrument.");

    let g = core.factorization_grid(flight, 1000.0, 1400.0, &[0.02, 0.20], &[0.02, 0.10, 0.40],
                                    Some(&lim), None, 2.0, 2.0, 0.02, &eps);
    p.print("\n  DO THE TWO CLOCKS FACTOR? (the premise a real fast-attack/slow-release limiter");
    p.print("  is DESIGNED on). Additive-separability residual on the DEBIT:");
    p.print(pyf!("  {:>9}", "tau_att")
            + &g.tau_rels.iter().map(|tr| pyf!("{:>12.2f}", tr)).collect::<String>());
    for (i, ta) in g.tau_atts.iter().enumerate() {
        p.print(pyf!("  {:9.2f}", ta)
                + &g.residual[i].iter().map(|v| pyf!("{:+12.6f}", v)).collect::<String>());
    }
    p.print(pyf!("  max residual {:.6f} vs max main effect {:.6f} -> {:.0%}",
                 g.max_residual, g.max_main_effect, g.max_residual / g.max_main_effect));
    p.print("  => the interaction is the SAME ORDER as the main effects (and 70% at r=0.5), while");
    p.print("     the credit spread is machine zero. THE TWO CLOCKS SEPARATE ONE WAY: tau_att owns");
    p.print("     the credit EXACTLY, the debit is irreducibly JOINT. The design premise is HALF");
    p.print("     TRUE -- and the half that fails is the PROTECTIVE one.");
    p.print("  Rung 51's reason 2 was FORM-dependent: an asymmetric-RATE lag switches on");
    p.print("  sign(required-g) and both branches share the vanishing numerator, so the RHS is a");
    p.print("  KINK not a jump -- RK4-legal, and rung 47's latch hazard does not recur. Reason 3");
    p.print("  STANDS (an exponential never completes): the release edge is DECLARED");
    p.print("  fractional-of-schedule and every debit is reported at TWO epsilons.");
    p.print("  phi_lim rides rungs 36/41/49's imposed constant -> signs, machine-zeros and the");
    p.print("  ds/tau convergences are the claims, not magnitudes. NEXT SEAM: the lag's SHAPE, and");
    p.print("  the two-lag cascade (tau_gov + lag) this rung refuses -- what a real FADEC runs.");
    p.print("  Reduces: lag=None -> rungs 45/46/47/48/49/50/51 bit-for-bit; lag+s_off/tau_rel");
    p.print("  ASSERTS (alternative instruments); lag+tau_gov ASSERTS; cycle rung-6 exact.");
}
