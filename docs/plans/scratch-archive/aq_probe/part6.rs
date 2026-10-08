
/// Rung 80's `phi_air` label: `"shared" if phi_air is None else "%.2f" % phi_air`.
fn air(pa: Option<f64>) -> String {
    match pa {
        None => "shared".to_string(),
        Some(x) => pct!("%.2f", x),
    }
}

/// `print_split_wall_table(flight)` — rung 80.
pub fn split_wall_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE SPLIT WALL (rung 80): a LEVEL split separates loops on the CONSTRAINT, never");
    p.print("  the two that share the ACTUATOR.");

    // THE FUEL WALL IS 0.75 AND BOTH EDGES OF ITS WINDOW ARE READ, NOT CHOSEN (rung 74's arrest
    // interval). Every reader: taus = (0.05,)*4, inc = False, ds = 0.005, v_max = 0.20.
    let phi_fuel = 0.75;
    let fresh = || rig(build_split_wall_cascade, d);

    // `split_liveness(..., phi_lim=PHI_FUEL)`: phi_airs = (None, 0.76, 0.77),
    // coords = ("demand", "clip").
    let lv = split_liveness(&fresh(), flight, LO, HI, TMAX, phi_fuel,
                            &[None, Some(0.76), Some(0.77)], &["demand", "clip"], TAUS, false, 0.5,
                            1.2, 0.005, 0.20);
    p.print("\n  s 2 -- THE CELL OPENS. `phi_air = shared` is rung 74's own result, reproduced");
    p.print("    as this table's baseline at IDENTICAL settings:");
    p.print("      coord   phi_air   valve moved   fuel cuts   ALL-FOUR riding");
    for r in &lv.rows {
        p.print(pct!("      %-7s %-9s %5d/%d      %4d/%d      %4d/%d",
                     r.coord, air(r.phi_air), r.valve_moved, r.npts, r.n_cut_fuel, r.npts,
                     r.n_riding4, r.npts));
    }
    p.print("    The `clip` rows are the POSITIVE CONTROL (they have four-loop cells at the");
    p.print("    shared wall, so zeros there would mean a broken reader, not a quiet plant).");
    p.print("    THE ANCHOR PREDICTED THE FUEL LEG WOULD GO DORMANT. It does not: its cut is");
    p.print("    evaluated at the SCHEDULED fuel, so a lever that raises the ACHIEVED phi only");
    let cuts: Vec<usize> = lv.rows.iter().filter(|r| r.coord == "demand").map(|r| r.n_cut_fuel)
        .collect();
    assert_eq!(cuts.len(), 3, "Python's `%` takes exactly three");
    p.print(pct!("    ERODES it (%d -> %d -> %d). Live on the counterfactual, not on the state.",
                 cuts[0], cuts[1], cuts[2]));

    // `split_arrest(..., phi_lim_lo=PHI_FUEL)`: walls = (0.7700, 0.7725, 0.7731, 0.7732, 0.7740,
    // 0.7800), phi_air_hi = 0.80, coord = "demand".
    let ar = split_arrest(&fresh(), flight, LO, HI, TMAX,
                          &[0.7700, 0.7725, 0.7731, 0.7732, 0.7740, 0.7800], phi_fuel, 0.80,
                          "demand", TAUS, false, 0.5, 1.2, 0.005, 0.20);
    p.print("\n  s 3 -- AND THE ARREST BELONGS TO NEITHER FLOOR, BUT TO THEIR COINCIDENCE:");
    p.print("      wall     shared (both)      air only          fuel only");
    for (i, w) in ar.walls.iter().enumerate() {
        let cells: Vec<String> = ["shared", "air", "fuel"].iter().map(|arm| {
            let row = &ar.arm(arm).rows[i].1;
            pct!("%-9s %7.1f", if row.arrested { "ARRESTED" } else { "marches" }, row.max_tt4)
        }).collect();
        p.print(pct!("      %-8.4f %s   %s   %s", w, cells[0], cells[1], cells[2]));
    }
    let shared = ar.arm("shared");
    p.print(pct!("    The SHARED column is rung 74's bracket to the digit (last march %.4f, first",
                 shared.last_march.expect("the shared arm marches somewhere")));
    p.print(pct!("    arrest %.4f, the free operating point 0.7731162133 INSIDE it) -- which is what",
                 shared.first_arrest.expect("the shared arm arrests somewhere")));
    p.print("    makes the other two columns readable. Neither SPLIT arm arrests anywhere, and");
    p.print("    the lift is still HAPPENING in both (phi(0) sits on 0.78 / 0.80 with the valve");
    p.print("    open), so the null is not a dormant knob. On a shared wall `the floor that");
    p.print("    lifts` and `the leg with no margin left` are ONE OBJECT; separating them is");
    p.print("    the only way to see that the lift which CAUSES the arrest also CURES it.");

    // `split_gains(..., phi_lim=PHI_FUEL, phi_airs=(None, 0.77), coord="clip")`: every = 5.
    let gn = split_wall_gains(&fresh(), flight, LO, HI, TMAX, phi_fuel, &[None, Some(0.77)], "clip",
                              TAUS, false, 0.5, 1.2, 0.005, 0.20, 5)
        .expect("rung 80's shipped gains read");
    p.print("\n  s 5 -- BUT THE FOURTH LOOP IS RIDING, NOT AUTHORITATIVE. Under min-select one");
    p.print("    fuel-side leg still holds the actuator and the other is MASKED, with its");
    p.print("    column at EXACTLY zero however far apart the walls are set:");
    for a in &gn.arms {
        // `a["authority"]` is Python's insertion-ordered `auth` dict, printed by `%s`.
        let keys: Vec<Option<&str>> = a.authority.iter().map(|(k, _)| k.map(|x| x.as_str()))
            .collect();
        let items: Vec<(&dyn PyFormat, &dyn PyFormat)> = keys.iter().zip(&a.authority)
            .map(|(k, (_, n))| (k as &dyn PyFormat, n as &dyn PyFormat)).collect();
        p.print(pct!("      phi_air %-6s interior %d   authority %s   max|mask leak| %s",
                     air(a.phi_air), a.n_interior, py_dict(&items), a.max_mask_leak));
    }
    p.print("    AND THE ZERO IS GUARDED, because an exact zero can mean *nothing was computed*:");
    p.print("    the shared arm holds a cell where the OTHER leg is masked, and on the identical");
    p.print(pct!("    code path it returns |cyclic| = %.6f -- rung 66's `two loops on one variable",
                 gn.control_nonzero.expect("the control cell exists")));
    p.print("    give exactly 1`. Every split arm differenced ALL of its points (skipped 0/0).");

    p.print("\n  WHAT THIS SAYS. The split delivers the seam's own object -- all four loops");
    p.print("    riding and strictly interior in `demand`, empty at every shared wall -- and NOT");
    p.print("    a fourth live loop: `n_live` is still <= 3, a SIXTH time. Five rungs failed at");
    p.print("    this seam because it was written in the RIDING noun and scored in the");
    p.print("    AUTHORITY one, and the two differ by exactly the leg `min` masks. Authority is");
    p.print("    decided on the ACTUATOR, so no CONSTRAINT-side knob can buy it. All three");
    p.print("    pre-registered predictions were REFUTED (anchor s 5a). Zero new constants.");
    p.print("    CORRECTS rung 74 s 2.2. See docs/rung80-spec.md.");
}

/// `print_authority_clock_table(flight)` — rung 81.
pub fn authority_clock_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE AUTHORITY CLOCK (rung 81): authority is decided by the LAG, not by the SET");
    p.print("  POINT -- so a leg that never holds the actuator has no clock at all.");

    // RUNG 80's OWN WALLS, unchanged.
    let (phi_fuel, phi_air) = (0.75, 0.77);
    let fresh = || rig(build_split_wall_cascade, d);

    // `authority_clock(..., tau_fs=(0.05, 0.20), tau_govs=(0.05,))`: tau_q = tau_s = 0.05,
    // coords = ("demand", "clip"), ds = 0.005.
    let g = authority_clock(&fresh(), flight, LO, HI, TMAX, phi_fuel, Some(phi_air), &[0.05, 0.20],
                            &[0.05], 0.05, 0.05, &["demand", "clip"], 0.5, 1.2, 0.005, 0.20, false);
    p.print("\n  s 1 -- WHO HOLDS THE ACTUATOR. One knob per axis: the VALVE clock is pinned at");
    p.print("    rung 80's 0.05 throughout, so nothing below can be the valve's doing.");
    p.print("      coord   tau_f   ALL-FOUR riding   fuel-held   gov-held   max Tt4");
    for r in &g.rows {
        p.print(pct!("      %-7s %5.2f   %8d          %6d      %6d     %7.1f",
                     r.coord, r.tau_f, r.n_riding4, r.n_fuel, r.n_gov, r.max_tt4));
    }
    p.print(pct!("    The `demand` tau_f = 0.05 row is RUNG 80's OWN CELL (%d four-loop points, every",
                 g.control_n_riding4[0]));
    p.print("    one held by the governor) -- the control that says this is the shipped plant.");
    p.print("    Slowing ONLY the fuel leg hands it the actuator, and the governor's own limit is");
    p.print("    STILL the more severe one there: the leg demanding the deeper cut is not the leg");
    p.print("    setting the fuel. That is the whole mechanism, and it is rung 74's `bill`.");

    p.print("\n  s 3 -- AND ITS CONSEQUENCE, WHICH IS THE HEADLINE. Compare each coordinate's two");
    p.print("    marches BIT-FOR-BIT (341 points x phi_lp/Tt4/b/v = 1364 floats):");
    for (k, v) in &g.tau_f_inert {
        let v = v.as_ref().expect("Python reads every entry's keys");
        p.print(pct!("      %-14s %4d of %d floats moved   %s",
                     k, v.n_differing, v.n_floats,
                     if v.march_identical { "IDENTICAL -- the clock reached nothing" }
                     else { "the clock is decisive" }));
    }
    p.print("    In `clip` at this wall the fuel leg is MASKED for the whole ramp, and its own");
    p.print("    time constant reaches NOTHING. That is rung 72's `min is flat in the masked");
    p.print("    leg` -- until now a zero in a Jacobian -- as an exact invariance of the plant.");
    p.print("    THAT ROW IS THE ENDPOINT PAIR OF A SIX-CLOCK COLUMN, not the claim: the spec");
    p.print("    measures it over six fuel clocks at THREE governor clocks (0 of 1364 floats in");
    p.print("    every clip column, 1304 of 1364 in every demand one) -- docs/rung81-spec.md s 3.");
    p.print("    AND THE CONTROL SAYS IT IS ABOUT MASKING, NOT ABOUT `clip`: at the SHARED wall,");
    p.print("    where that leg DOES take the actuator, the same sweep is live --");
    let taus: Vec<String> = g.control_clip_rows.iter().map(|x| pct!("%5.2f", x.tau_f)).collect();
    let fuel: Vec<String> = g.control_clip_rows.iter().map(|x| pct!("%5d", x.n_fuel)).collect();
    p.print(format!("      tau_f:       {}", taus.join("  ")));
    p.print(format!("      fuel-held:   {}", fuel.join("  ")));
    p.print("    and it falls the OTHER way, because a clip state lags its target from below: a");
    p.print("    slower leg cuts LESS. The sign of the clock's authority flips with the");
    p.print("    coordinate -- the same slowing that BUYS the actuator in `demand` SELLS it here.");

    // `authority_mask(..., phi_lim=PHI_FUEL, phi_air=PHI_AIR)`: clocks = ((0.20, 0.05, 0.05,
    // 0.05), (0.05, 0.05, 0.05, 0.05)), coord = "demand", ds = 0.005, every = 1.
    let mk = authority_mask(&fresh(), flight, LO, HI, TMAX, phi_fuel, Some(phi_air),
                            &[(0.20, 0.05, 0.05, 0.05), (0.05, 0.05, 0.05, 0.05)], "demand", 0.5,
                            1.2, 0.005, 0.20, false, 1)
        .expect("rung 81's shipped mask reads");
    p.print("\n  s 4 -- RUNG 72's BLOCK, READ ON THE OTHER SIDE OF THE SWITCH:");
    p.print("      clocks                    interior   authority   max|mask leak|   max|cyclic|");
    for a in &mk.arms {
        // `sorted(a["by_authority"].items())` — string keys, so a `None` among them would raise.
        let mut by: Vec<(&str, &crate::authority_clock::AuthStats)> = a.by_authority.iter()
            .map(|(k, v)| (k.expect("Python sorts str keys only").as_str(), v)).collect();
        by.sort_by(|x, y| x.0.cmp(y.0));
        let (c0, c1, c2, c3) = a.taus;
        let taus = py_tuple(&[&c0, &c1, &c2, &c3]);
        for (auth, dd) in by {
            p.print(pct!("      %-25s %6d     %-9s   %.3e        %.6f",
                         taus, dd.n, auth, dd.max_leak, dd.max_cyc));
        }
    }
    p.print("    The mask is SYMMETRIC: exactly one authority per point and a mask leak of zero");
    p.print("    on BOTH sides, so `n_live <= 3` a SEVENTH time -- this rung throws the switch,");
    p.print("    it does not remove it. The cyclic column is the discriminator, and both of its");
    p.print("    branches are in this one table: the three-phi-loop cycle runs THROUGH the fuel");
    p.print("    leg, so it is exactly 0 where that leg is masked and ~1 where the governor is.");
    p.print("    A reader returning only zeros would be indistinguishable from one measuring");
    p.print("    nothing; this one is not.");
    p.print("\n  HONEST SCOPE: one rig, one wall pair, one flight condition, one ramp. The");
    p.print("    criterion behind s 1 is QUASI-STEADY and misses near its own tie (9 of 1054 on");
    p.print("    the full grid, worst 11.8% from it) -- disclosed in docs/rung81-spec.md s 2,");
    p.print("    where the refined grid that FOUND those misses is also recorded: the first grid");
    p.print("    scored 506 of 506 and had no hard cases in it.");
}

/// The `kw` dict rungs 83/84 spell: rung 80's walls and rung 81's clocks, at ramp `r` and step
/// `ds` (rung 83's `kw` carries `ds=0.005`; rung 84's call sites pass it).
fn scan_kw(flight: &FlightCondition, r: f64, ds: f64) -> ScanKw<'_> {
    ScanKw { flight, tt4_lo: 1000.0, tt4_hi: 1400.0, tt4_max: 1200.0, phi_lim: 0.75,
             phi_air: Some(0.77), tau_gov: 0.05, tau_q: 0.05, tau_s: 0.05, r, s_settle: 1.2, ds,
             v_max: 0.20, inc: false }
}

/// `print_threshold_law_table(flight)` — rung 82.
pub fn threshold_law_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE THRESHOLD'S OWN LAW (rung 82): a criterion read FORWARD inherits the sign of");
    p.print("  its own reference -- so it reports where the reader started, not where the");
    p.print("  threshold is.");

    let (phi_fuel, phi_air) = (0.75, 0.77);
    let m = rig(build_split_wall_cascade, d);
    // `threshold_reference(..., phi_lim=PHI_FUEL, phi_air=PHI_AIR)`: r = 0.35,
    // tau_refs = (0.02, 0.03, 0.05, 0.08, 0.12), the three clocks 0.05, bracket = (0.004, 0.30),
    // n_bisect = 10, ds = 0.005.
    let g = match threshold_reference(&m, flight, LO, HI, TMAX, phi_fuel, Some(phi_air), 0.35,
                                      &[0.02, 0.03, 0.05, 0.08, 0.12], 0.05, 0.05, 0.05,
                                      (0.004, 0.30), 10, 1.2, 0.005, 0.20, false) {
        ThresholdReference::Full(g) => g,
        ThresholdReference::Void { .. } => unreachable!("Python reads the full dict's keys"),
    };

    p.print(pct!("\n  s 3a -- ONE ramp (r = %.2f), ONE plant. The threshold is BISECTED on the plant",
                 g.r));
    p.print(pct!("    at tau_f* = %.5f (bracket %.2e wide), then the criterion is read FORWARD from",
                 g.tau_star, g.width));
    p.print("    five different reference marches -- the only thing changing is where it STARTED.");
    p.print("      tau_ref   reference is   forward tau_f*   signed error   |error|");
    for x in &g.rows {
        let fwd = x.fwd.expect("every reference reads forward");
        p.print(pct!("      %7.3f   %-12s   %13.5f   %+12.5f   %6.1f%%",
                     x.tau_ref, if x.ref_above { "ABOVE tau_f*" } else { "below tau_f*" }, fwd,
                     fwd - g.tau_star, 100.0 * x.err.expect("every reference reads forward")));
    }
    p.print(pct!("    THE SIGN FOLLOWS THE REFERENCE'S SIDE, %d of %d: a reference above the threshold",
                 g.n_agree, g.n_live));
    p.print("    UNDER-predicts, one below OVER-predicts. The forward reading is therefore not a");
    p.print("    prediction of the threshold -- it is a report on where the reader began.");

    p.print("\n    AND THE MAP CONTRACTS ON ONE SIDE ONLY. Read as an iteration tau_ref -> forward,");
    p.print(pct!("    the error SHRINKS with distance ABOVE the threshold (grows_above=%s) and GROWS",
                 g.grows_above));
    p.print(pct!("    BELOW it (grows_below=%s).", g.grows_below));
    p.print("    So starting FURTHER AWAY -- on the right side -- reads BETTER. That is rung 77's");
    p.print("    1/(1-c) with the SIGN of c deciding whether the reading is usable at all, and the");
    p.print("    threshold is the boundary between the two regimes.");
    p.print("    It also explains the ramp sweep the spec carries: the two ramps where the forward");
    p.print("    reading blew up (73.6%, 182.9%) are EXACTLY the two whose reference sat below.");

    p.print("\n  WHAT THIS BOUNDS. Rung 81's 99.15% is a LABEL score, and it holds because every");
    p.print("    input is read at the very point being labelled. Asked to predict ACROSS");
    p.print("    trajectories the criterion has no separation to exploit -- the spec measures the");
    p.print("    governor clock keeping 53% of its own coefficient and the surge floor, which the");
    p.print("    criterion puts in the SET-POINT term, moving the fuel-cap SLOPE by 144%.");
    p.print("    The seam asked to turn a label predictor into a quantitative one; the answer is");
    p.print("    no, and rung 81's result stands exactly where it was measured.");
    p.print("\n  HONEST SCOPE: one rig, one wall pair, one flight condition. The ramp window");
    p.print("    (r <= 0.85) and the wall range (0.745-0.755) are where the plant is MARCHABLE,");
    p.print("    not physical boundaries -- outside them there is no four-loop point at all, or");
    p.print("    the threshold leaves the bracket. Both are disclosed in docs/rung82-spec.md s 6.");
}

/// `print_corrector_law_table(flight)` — rung 83.
pub fn corrector_law_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE CORRECTOR'S OWN BAR (rung 83): a bracketing solve locates a SIGN CHANGE; a");
    p.print("  corrector needs a ROOT -- and on a residual built as a MINIMUM those are");
    p.print("  different objects.");

    let m = rig(build_split_wall_cascade, d);
    let kw = |r: f64| scan_kw(flight, r, 0.005);

    p.print("\n  s 1  RUNG 82's TWO READINGS ARE ONE OBJECT. Its scan returns h = min[hat - tau_eff]");
    p.print("    and tau_hat_min = min[hat]. With tau_eff constant along the trajectory the two");
    p.print("    minima share an argmin, so h = kappa*(F - tau) with F = tau_hat_min/kappa -- rung");
    p.print("    82's own FORWARD reading. THE FIXED POINT IS THE ROOT OF THE FORWARD READING'S OWN");
    p.print("    RESIDUAL, so its bisection is literally solving F(tau) = tau.");
    let rd = corrector_read(&m, 0.08, &kw(0.5));
    p.print(pct!("      at r=0.50, tau_f=0.080:  h = %+.12e", rd.h.expect("the window is open")));
    p.print(pct!("                               tau_hat_min - kappa*tau = %+.12e",
                 rd.identity_pred.expect("the window is open")));
    p.print(pct!("      identical to the last bit: %s     (kappa = %.1f, exact)",
                 rd.exact, rd.kappa.expect("the window is open")));
    p.print("    A Newton step then reads tau_1 = tau_0 + (F - tau_0)/(1 - c), and KAPPA CANCELS:");
    p.print("    the correction is the forward reading's own miss over rung 77's 1/(1-c) -- that");
    p.print("    scalar's THIRD role, after 77's stiffness and 78's gauge.");

    p.print("\n  s 2  AND THE SIDE IS FREE. h > 0 iff F > tau iff tau is BELOW the root, so ONE");
    p.print("    march says which side a reference sits on. Rung 82 s 6 says the reader 'cannot");
    p.print("    know which side it is on without solving the problem it was trying to avoid'.");
    p.print("      r      tau_f    sign(h)      says          root is at   correct");
    for (r, root, taus) in [(0.25, 0.019754, [0.008, 0.050]), (0.35, 0.037098, [0.020, 0.120])] {
        for t in taus {
            let x = corrector_read(&m, t, &kw(r));
            let below = x.below_root.expect("the window is open");
            p.print(pct!("      %.2f   %.3f    %-9s    %-11s   %.6f     %s",
                         r, t, if x.h.expect("the window is open") > 0.0 { "positive" }
                         else { "negative" },
                         if below { "BELOW root" } else { "above root" }, root, below == (t < root)));
        }
    }
    p.print("    IT CAN. The side is one march; only the ROOT is a solve. CORRECTS rung 82 s 6.");

    p.print("\n  s 3  SO WHY DOES ONE STEP NOT WORK? Not accuracy, and not the slope. F(tau) is a");
    p.print("    MINIMUM over the scored trajectory points, so it JUMPS wherever the binding point");
    p.print("    changes hands -- and at one of rung 82's five ramps the residual's sign change IS");
    p.print("    such a handover. 21 marches at 1.25e-05 spacing, inside the bisection's own bracket:");
    let jump = residual_shape(&m, 0.0197750, 0.0197875, 2, &kw(0.25)).changes.remove(0);
    let cross = residual_shape(&m, 0.037000, 0.037333, 2, &kw(0.35)).changes.remove(0);
    p.print("      (this panel uses main.py's R_c=286.9, so the last digits differ from the");
    p.print("       spec's tables, which carry the tests' R_c -- that is the gas, not drift)");
    p.print("      ramp   tau window            g below      g above      binding pt   min|g|/step");
    for (nm, ch) in [("0.25", &jump), ("0.35", &cross)] {
        p.print(pct!("      %s   %.7f..%.7f  %+.4e  %+.4e  %-10s   %8.2f",
                     nm, ch.tau_lo, ch.tau_hi, ch.g_lo, ch.g_hi,
                     if ch.argmin_moved { "HANDS OVER" } else { "unchanged" }, ch.ratio));
    }
    p.print("    AT r=0.25 THE RESIDUAL DOES NOT APPROACH ZERO -- IT STEPS ACROSS IT. The smaller");
    p.print(pct!("    of the two is %.0fx the tau step, and the step coincides with the argmin handover.",
                 jump.ratio));
    p.print("    So on the shipped grid THERE IS NO FIXED POINT at that ramp: rung 82's 13-march");
    p.print("    answer, 0.019754, is the location of a DISCONTINUITY. At r=0.35 the same reader");
    p.print(pct!("    on the same plant finds a genuine crossing (ratio %.2f, no handover) -- and that",
                 cross.ratio));
    p.print("    is the one ramp of five where a corrector works at all.");

    p.print("\n  s 4  THE COST, PAIRED WITH WHAT IT BUYS. From the start rule fixed in advance");
    p.print("    (sqrt of rung 82's own bracket, then x1.25), the secant at r=0.35 drives the");
    p.print("    residual to ~1e-15 in 8 marches against 13 -- and its 0.10% 'error' is the");
    p.print("    BISECTION's, whose 13 marches only resolve the root to +-0.39%. Started AT rung");
    p.print("    82's own answer for r=0.25, the same secant OSCILLATES and never converges.");
    p.print("      Under the registered start, 1 of 5 ramps reaches 1%; the intervention that moves");
    p.print("      the start onto each root converts 3 of the 4 failures. The fourth is r=0.25,");
    p.print("      because there is nothing there to converge to.");

    p.print("\n  WHAT THIS BOUNDS. The thirteen marches were never buying resolution -- they buy AN");
    p.print("    ANSWER THAT EXISTS. Bisection reads only sign(h), which s 2 shows is free and is");
    p.print("    defined whether or not a root is there; a corrector reads the residual's VALUE and");
    p.print("    SLOPE, which presuppose one. The seam's single step is answered NO. Rung 78 found a");
    p.print("    residual's SLOPE is a GAUGE and its root's UNIQUENESS is not; its EXISTENCE is not");
    p.print("    either -- refining the march step to rung 82's own ds_fine CREATES the r=0.25 root,");
    p.print("    moves the answer by a full bracket width, and opens a new handover elsewhere.");
    p.print("\n  HONEST SCOPE: one rig, one coordinate, one flight condition. The jumping mechanism");
    p.print("    is general to rung 82's reader; that it leaves a MISSING ROOT at 1 of 5 ramps is");
    p.print("    measured here only, and which ramps have roots at which ds is a two-parameter");
    p.print("    sweep this rung does not run. docs/rung83-spec.md s 7.");
}

/// `print_staircase_law_table(flight)` — rung 84.
pub fn staircase_law_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE MARCHED MINIMUM'S STAIRCASE (rung 84): a minimum over a MARCHED set is not a");
    p.print("  minimum at all -- it is a reading on a MOVING GRID BOUNDARY, so the residual");
    p.print("  carries the march's own SAWTOOTH.");

    let m = rig(build_split_wall_cascade, d);
    let kw = |r: f64, ds: f64| scan_kw(flight, r, ds);
    let kind = |k: Option<Kind>| k.map(Kind::as_str);

    p.print("\n  s 1  A MIN CANNOT JUMP. Rung 83 says F is a `min`, so it jumps at every handover.");
    p.print("    A minimum over a FIXED finite set of continuous functions is CONTINUOUS -- it");
    p.print("    KINKS where the argmin changes hands. A jump needs the SET to change. And the");
    p.print("    argmin is not interior at all: it is the four-loop window's FIRST marched point.");
    p.print("      ramp   tau        binding pt   window edge  on the grid   at the edge");
    for (r, tau) in [(0.25, 0.0197), (0.35, 0.0371), (0.70, 0.0920)] {
        let rd = edge_read(&m, tau, &kw(r, 0.005));
        p.print(pct!("      %.2f   %.5f    %-10s   %-11s  %-12s  %s",
                     r, tau, rd.s_bind, rd.edge, rd.edge_on_grid, rd.at_edge));
    }
    p.print("    Measured at 71 of 71 across five ramps, both shipped steps, both branches. So");
    p.print("    rung 82's `min` is DECORATIVE: the object rungs 82 and 83 solve is an evaluation");
    p.print("    on a boundary that sits on the march grid and moves in whole grid steps.");

    p.print("\n  s 2  WHICH MAKES THE CLASSIFIER AN IDENTITY, NOT A THRESHOLD. On the points two");
    p.print("    marches SHARE, the minimum is over a fixed set and so is continuous. Split the");
    p.print("    difference there (SMOOTH) from the rest (MEMBERSHIP) and no tolerance is needed:");
    p.print("      ramp   tau window             d(total)     SMOOTH       MEMBERSHIP   verdict");
    for (nm, r, lo, hi, ds) in [("0.25", 0.25, 0.0197750, 0.0197875, 0.005),
                                ("0.35", 0.35, 0.0370000, 0.0373330, 0.005)] {
        let c = classify(&edge_read(&m, lo, &kw(r, ds)), &edge_read(&m, hi, &kw(r, ds)));
        p.print(pct!("      %s   %.7f..%.7f  %+.4e  %+.4e  %+.4e  %s",
                     nm, lo, hi, c.d_full.expect("both reads score"),
                     c.d_smooth.expect("both reads score"),
                     c.d_membership.expect("both reads score"), kind(c.kind)));
    }
    p.print("    THE MEMBERSHIP TERM IS EXACTLY ZERO when the sets agree -- not small, zero. Rung");
    p.print("    83's jump is the window OPENING ONE MARCH STEP EARLIER, and 99.4% of its step is");
    p.print("    that one entering point; restricted to the shared points the sign change VANISHES.");
    p.print("    (This panel uses main.py's R_c=286.9, so last digits differ from the spec's tables.)");

    p.print("\n  s 3  SO ROOT EXISTENCE IS A PROPERTY OF THE PLANT **AND ITS RESOLUTION**. The same");
    p.print("    ramp, one halving of the march step apart:");
    p.print("      ramp   ds        bisected at   verdict     membership");
    for ds in [0.005, 0.0025] {
        // `root_class(bracket=(0.004, 0.30), ...)`: n_bisect = 10, eps = 1e-7.
        let RootClass::Ok(rc) = root_class(&m, (0.004, 0.30), 10, 1e-7, &kw(0.25, ds)) else {
            unreachable!("Python reads the bisection's keys")
        };
        p.print(pct!("      0.25   %-8g  %.6f      %-9s   %+.4e",
                     ds, rc.mid, kind(rc.kind), rc.d_membership.expect("both ends score")));
    }
    p.print("    Five ramps x both shipped steps: ONE cell of ten has no root, and it is rung 83's");
    p.print("    own. So rung 82's five-row table contains exactly ONE discontinuity -- the seam's");
    p.print("    question, answered.");

    p.print("\n  WHAT THIS CORRECTS. Rung 83's `argmin_moved` flag fired at the RIGHT PLACE for the");
    p.print("    WRONG REASON: an edge move FORCES an argmin move, never the reverse, and on this");
    p.print("    plant the two are one event (0 counter-examples in 40 pairs). Verdict CONFIRMED,");
    p.print("    reason CORRECTED -- rung 28's shape. And rung 82's ds control gets the SCALE it");
    p.print("    lacked: it voids a threshold that moves by more than a BISECTION width, but the");
    p.print("    sawtooth lets a root shift ~0.48*ds for an entirely benign reason -- twelve such");
    p.print("    widths. The row it voids moves 5 of them, well inside.");
    p.print("\n  HONEST SCOPE: one rig, one coordinate. THREE of seven registered predictions were");
    p.print("    REFUTED, two of them because a ratio of SMALL INTEGER COUNTS cannot carry a rate.");
    p.print("    The claim that refinement RELOCATES rather than removes a missing root was one of");
    p.print("    them: measured absent/present/present/present over four steps. The shadow FRACTION");
    p.print("    is ds-free by its factors; ten cells cannot test a rate. docs/rung84-spec.md s 8.");
}
