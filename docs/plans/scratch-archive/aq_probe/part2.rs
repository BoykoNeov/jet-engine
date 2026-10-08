
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
