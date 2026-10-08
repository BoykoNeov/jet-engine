
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
