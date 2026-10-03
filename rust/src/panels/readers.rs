//! Rungs 78–84's panels — the residual gauge, the state coordinate, the split wall, and the
//! reader-only rungs 81–84 (slice AQ, third part).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line for
//! line, on rung 53's hardware and the rung-77 rig: `bleed_lim` and the `phi` stator from the
//! `phi = 0.80` margin, then four (rung 78: five) knobs set by PLAIN ASSIGNMENT. **Every reader
//! takes `ds = 0.005`** — its own default, or the `ds=0.005` rungs 83/84 spell in their `kw`.
//! Rung 79 builds a fresh machine per reader (`rig()`), and calls `coord_forced` TWICE, as the
//! Python does; the port keeps both calls.

use super::actuator::TAUS;
use super::airflow::lp_map;
use super::cascades::{machine, Build, B, FLOOR, HI, LO};
use super::Design;
use crate::authority_clock::{authority_clock, authority_mask};
use crate::bleed_transient::LeverArm;
use crate::corrector_law::{corrector_read, residual_shape};
use crate::engine::FlightCondition;
use crate::limited_bleed::BleedLimiter;
use crate::pct;
use crate::pyfmt::{py_dict, py_list, py_tuple, Printer, PyFormat};
use crate::residual_gauge::{build_residual_gauge_cascade, gauge_scan, root_census, GAUGE_SCAN_MULTS,
                            ROOT_CENSUS_MULTS};
use crate::state_coordinate::{build_state_coordinate_cascade, coord_census, coord_forced,
                              coord_march, coord_scan};
use crate::split_wall::{build_split_wall_cascade, split_arrest, split_gains as split_wall_gains,
                        split_liveness};
use crate::staircase_law::{classify, edge_read, root_class, Kind, RootClass};
use crate::stator_transient::ScheduledStatorCore;
use crate::three_loop::StatorLimiter;
use crate::threshold_law::{threshold_reference, ScanKw, ThresholdReference};

const TMAX: f64 = 1200.0;

/// The rung 77–84 rig: `bleed_lim=BleedLimiter.from_margin(LP, 0.10, sm, tau=0.05)` and
/// `stator_lim=StatorLimiter.from_margin(LP, 0.20, sm, tau=0.05)` at `sm = 0.80/FLOOR - 1`, then
/// `m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"`.
fn rig(build: Build, d: &Design) -> ScheduledStatorCore {
    let lp = lp_map();
    let sm = 0.80 / FLOOR - 1.0;
    let m = machine(build, d, &LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp, B, sm, Some(0.05))),
        stator_lim: Some(StatorLimiter::from_margin(&lp, 0.20, sm, Some(0.05))),
        ..LeverArm::default()
    });
    m.fuel.inner.lag_coord.set("demand");
    m.fuel.inner.ref_law.set("sched");
    m.fuel.inner.windup_law.set("none");
    m.fuel.inner.cap_law.set("solve");
    m
}

/// `str(list)`.
fn list<T: PyFormat>(xs: &[T]) -> String {
    let v: Vec<&dyn PyFormat> = xs.iter().map(|x| x as &dyn PyFormat).collect();
    py_list(&v).py_str()
}

/// `print_residual_gauge_table(flight)` — rung 78.
pub fn residual_gauge_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE RESIDUAL GAUGE (rung 78): a residual's SLOPE is a GAUGE; its root's");
    p.print("UNIQUENESS is not.");

    let (margin, phi) = (0.10, 0.80);
    let t = rig(build_residual_gauge_cascade, d);
    // Rung 78 also assigns `t._tau_t = None` — the fifth knob, as rung 77 does.
    t.fuel.inner.tau_t.set(None);

    // `gauge_scan(...)`: ds = 0.005, dq = 1e-5, every = 8, `mults=None` -> the reader's own grid.
    let g = gauge_scan(&t, flight, LO, HI, TMAX, phi, margin, TAUS, false, 0.5, 1.2, 0.005, 0.20,
                       1e-5, 8, &GAUGE_SCAN_MULTS);
    // `root_census(...)`: every = 8, its own mults, lo = 0.2, hi = 3.0, n = 400.
    let cen = root_census(&t, flight, LO, HI, TMAX, phi, margin, TAUS, false, 0.5, 1.2, 0.005, 0.20,
                          8, &ROOT_CENSUS_MULTS, 0.2, 3.0, 400);

    p.print("\n  THE CONSTRUCTION -- and the set point is invariant by ALGEBRA, not tolerance:");
    p.print("    cap_k(w) = w0 + k*(cap(w) - w0)      w0 := the k=1 root, so cap(w0) = w0");
    p.print("    cap_k(w0) = w0 + k*0 = w0            for EVERY k          =>  G_k' = 1 - k*c");

    let r = &g.rows[0];
    p.print(pct!("\n  ONE RIDING POINT (s = %.3f, c = %.6f, so the singular gauge is k = %.4f):",
                 r.s, r.c, r.k_crit));
    p.print(pct!("    %7s %10s %13s %13s %11s %16s %11s",
                 "k*c", "k", "G_w", "1 - k*c", "w move", "dw*/dq", "gain move"));
    for dd in &r.ks {
        p.print(pct!("    %7.2f %10.4f %13.5e %13.5e %11.2e %16.8e %11.2e%s",
                     dd.mult, dd.k, dd.gw, dd.gw_pred, dd.w_move, dd.direct, dd.gain_move,
                     if dd.excluded { "   <- multi-rooted" } else { "" }));
    }

    let span = g.gw_span.expect("rows were sampled");
    p.print(pct!("\n  OVER ALL %d POINTS:  G_w spans %.4f .. %.4f (BOTH SIGNS), and matches 1-k*c",
                 g.n, span.0, span.1));
    p.print(pct!("    to %.2e.  The set point moves %.2e and dw*/dq moves %.2e.",
                 g.gw_err.expect("rows were sampled"), g.w_move.expect("rows were sampled"),
                 g.gain_move.expect("rows were sampled")));
    p.print("    So `1/(1-c)` is reachable, reaches INFINITY, and the plant does not notice:");
    p.print("    G_w and G_q carry the SAME vanishing factor, so their quotient is invariant.");

    p.print(pct!("\n  BUT THE ROOT'S UNIQUENESS DIES -- |G_k(w0)| = %.2e at every gauge (it is a",
                 cen.g_at_w0.expect("rows were sampled")));
    p.print("    root by construction), while a SECOND root collides with it at k*c = 1:");
    p.print(pct!("    %7s %7s   %s", "k*c", "roots", "locations (w/w0)"));
    for dd in &cen.rows[0].cells {
        let locs: Vec<String> = dd.roots.iter().map(|x| pct!("%.6f", x)).collect();
        p.print(pct!("    %7.2f %7d   %s", dd.mult, dd.n_roots, locs.join("  ")));
    }
    let band = cen.band.expect("a multi-root band exists");
    p.print(pct!("    multi-root band k*c in %s, which BRACKETS the singular gauge: %s",
                 py_tuple(&[&band.0, &band.1]), cen.brackets.expect("a multi-root band exists")));

    p.print("\n  WHAT THIS SAYS. Rung 77 s 3 called `c -> 1` one of two routes to a singular");
    p.print("    set-point solve and unreachable here. It is reachable in one line, and it is a");
    p.print("    REMOVABLE singularity -- a property of how the residual was WRITTEN. What the");
    p.print("    limit actually costs is WELL-POSEDNESS: inside the band a solver converges");
    p.print("    cleanly onto the WRONG root (ok=True, set point moved 61%). Rung 76 SURVIVES,");
    p.print("    because `solve` -> `sensed` MOVES the root (2.8e-02 .. 1.4e-01) -- a rewriting");
    p.print("    that moves the root is a DEVICE, one that preserves it is a COORDINATE.");
    p.print("    One swept knob, no constant. See docs/rung78-spec.md.");
}

/// `print_state_coordinate_table(flight)` — rung 79.
pub fn state_coordinate_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTHE STATE COORDINATE (rung 79): a coordinate is a GAUGE the PLANT cannot REACH.");

    let phi = 0.80;
    // Every reader: margin = 0.10, ds = 0.005, every = 8 (where it takes one).
    let fresh = || rig(build_state_coordinate_cascade, d);

    p.print("\n  THE CANCELLATION -- which is why the knob costs ZERO new constants:");
    p.print("    Gs(w) = phi_lim - phi(w)                                  [rung 49, SHIPPED]");
    p.print("    Gi(w) = m_lim - M_i(w) = [T_c - 1/phi_lim + v] - [T_c - 1/phi(w) + v]");
    p.print("          = 1/phi(w) - 1/phi_lim                    <- T_c AND v CANCEL IDENTICALLY");
    p.print("          = Gs(w)*h(w),   h := 1/(phi(w)*phi_lim) > 0   STRICTLY, so NO DIAL");

    let s = coord_scan(&fresh(), flight, LO, HI, TMAX, phi, 0.10, TAUS, false, 0.5, 1.2, 0.005,
                       0.20, 1e-5, 8);
    p.print(pct!("\n  s 1-3 -- THE GAUGE at %d riding points. Declared UNSCORED in advance: this is",
                 s.n));
    p.print("    four lines of algebra, so a disagreement here would be a BUG, not a finding.");
    p.print(pct!("    %7s %13s %13s %11s %11s %11s %15s",
                 "s", "w* (phi)", "w* (inc)", "slope phi", "slope inc", "ratio", "dw*/dq move"));
    for r in s.rows.iter().take(5) {
        p.print(pct!("    %7.3f %13.9f %13.9f %11.5f %11.5f %11.7f %15.3e",
                     r.s, r.w_phi, r.w_inc, r.slope_phi, r.slope_inc, r.ratio,
                     (r.dwdq_inc - r.dwdq_phi).abs()));
    }
    p.print(pct!("    ... %d more rows.", s.n as i64 - 5));
    p.print(pct!("    The slope scales by the DERIVED factor 1/phi_lim^2 = %.6f to %.2e, `dw*/dq`",
                 s.predicted_ratio, s.ratio_err.expect("rows were sampled")));
    p.print(pct!("    moves %.3e and the set point is the SAME FLOAT at %d of %d -- but NEITHER is a",
                 s.dwdq_err.expect("rows were sampled"), s.n_same_float, s.n));
    p.print("    measurement: every incidence solve here falls back to the phi solve, so both");
    p.print("    compare the fallback with ITSELF. The invariance rests on the algebra (both");
    p.print("    halves carry the same h(w*)); the slope ratio is the only number above a wrong");
    p.print("    coordinate would move.");

    let c = coord_census(&fresh(), flight, LO, HI, TMAX, phi, 0.10, TAUS, false, 0.5, 1.2, 0.005,
                         0.20, 8, 0.2, 3.0, 400);
    p.print("\n  s 4 -- THE ROOT CENSUS, and the instrument has an INHERITED POSITIVE CONTROL:");
    p.print(pct!("    counts EQUAL at %d/%d, n_roots = %s, located roots agree to %.3e.",
                 c.rows.iter().filter(|x| x.n_phi == x.n_inc).count(), c.n, list(&c.n_roots),
                 c.worst.expect("rows were sampled")));
    p.print("    A positive multiplier cannot create or destroy a sign change, so EQUAL is the");
    p.print("    identity -- but nothing forces either count to be ONE. `_root_count` is rung");
    p.print("    78 s 3's own walk, and THERE it FOUND a second root sweeping in and colliding");
    p.print("    at k*c = 1. An instrument that has detected the thing it now reports absent is");
    p.print("    not a blind one, so `[1]` is a measurement -- the first count of the PHI leg's");
    p.print("    roots any rung has taken.");

    // s 5.2 is READ here and PRINTED below: its `d_forced` is the only honest scale against which
    // s 5's `gap` can be called wide, since s 5's own `d_max` is exactly zero.
    let f = coord_forced(&fresh(), flight, LO, HI, TMAX, phi, 0.10, TAUS, false, 0.5, 1.2, 0.005,
                         0.20, 8);
    let m = coord_march(&fresh(), flight, LO, HI, TMAX, phi, 0.10, TAUS, false, 0.5, 1.2, 0.005,
                        0.20);
    let gap_min = m.gap_min.expect("the march logged a gap");
    p.print("\n  s 5 -- THE PLANT, AND ALMOST NONE OF IT MEANS WHAT IT LOOKS LIKE:");
    p.print(pct!("    hits = %d, binds = %d, flips = %d, trajectory worst = %.3e, sched_moved = %.3e",
                 m.hits, m.binds, m.flips, m.worst, m.sched_moved));
    p.print("    -- a flawless-looking pass, and the guard registered BEFORE the run says");
    p.print(pct!("    VACUOUS = %s, on both REGISTERED grounds at once: d_max = %.3e (the",
                 m.vacuous.expect("the guard is read"), m.d_max.expect("the march logged a delta")));
    p.print(pct!("    coordinate moved the cap by EXACTLY nothing) AND gap_min = %.6f (the legs",
                 gap_min));
    p.print(pct!("    are %.0f orders wider apart than the largest float move s 5.2 can force out",
                 (gap_min / f.d_forced.expect("a forced solve binds")).log10()));
    p.print("    of the coordinate, so no `min` could have flipped).");

    p.print("\n  ... AND A THIRD GROUND, FOUND AFTER SHIP (docs/rung79-gap-margin.md):");
    p.print("    THE MARCH NEVER LEAVES ITS INITIAL STATE -- nu_lp and nu_hp move by ZERO");
    p.print("    bits over every step; only the COMMAND ramps (mf_sched, 1.478x). Three phi");
    p.print("    floors sit on ONE wall and the stator lifts the free start (phi = 0.7731)");
    p.print("    exactly onto it, so rung 49's leg binds at s = 0 with no authority left:");
    p.print(pct!("    A LIMITER ARMED WITH ZERO INITIAL MARGIN HAS NO TRANSIENT. So the %d calls",
                 m.n_log));
    p.print(pct!("    are %d calls at ONE operating point, and the %d distinct gaps are distinct",
                 m.n_log, m.n_distinct_gap));
    p.print("    FLOATS -- a solve whose START POINT moves -- never distinct STATES. That is");
    p.print("    the FOURTH vacuity trap and the only one that survived the ship, sitting");
    p.print("    inside the guard s 8.1 is proudest of. s 5.1-5.3 below are identities of a");
    p.print("    branch condition and SURVIVE INTACT; every reading phrased as being ACROSS");
    p.print("    THE ACCEL does not. And the gap itself is then explained: gap(margin=0) = 0");
    p.print("    EXACTLY (a standing plant IS at steady state, which is what rung 48's");
    p.print("    margin-0 schedule returns), with d ln(gap+1)/d ln(1+margin) = 1/(1-c) --");
    p.print("    RUNG 77's STIFFNESS, not the kappa drift s 9 guessed. The rig is NOT");
    p.print("    re-tuned: phi_lim ON the wall is what ss 1-4's constrained linearisation");
    p.print("    requires, so the standstill is RECORDED AND GATED instead.");

    p.print("\n  ... AND THE ATTRIBUTION, CORRECTED (docs/rungs72-77-march-audit.md):");
    p.print("    THE ARREST IS A CELL, `(demand, phi_lim = 0.80)` -- NOT the shared rig. Of");
    p.print("    the six rungs the gap doc declined to claim about, NO SHIPPED march");
    p.print("    stands still: rungs 72/73/77 march in CLIP (77 at 0.80, this wall, and");
    p.print("    it is the LIVELIEST march in the family), rungs 75/76 in `demand` at");
    p.print("    0.76, and RUNG 74 -- the only one with an arrested arm at all -- sites");
    p.print("    its own shipped arms at 0.76/0.70, which move. And that cell was found,");
    p.print("    mechanised and GATED at RUNG 74 s 2.2 -- `THE ARREST, disclosed, not");
    p.print("    tuned away` -- before rungs 78/79 were built. Neither spec CITED it");
    p.print("    (rung 78 s 5.3 now records it); s 5 above marched inside it, so the");
    p.print("    hits/binds counters there read ONE point 341 times. Every number stands,");
    p.print("    the credit does not. The audit's own finding: at `(demand, 0.76)` the");
    p.print("    VALVE and the STATOR are INERT 0/341, so rungs 75/76 read a TWO-loop");
    p.print("    plant -- and a liveness counter on a FROZEN plant reports the initial");
    p.print("    condition at FULL count (valve = 341/341 here): spreads are read FIRST.");

    p.print("\n  s 5.3 -- THE COMPLEMENTARITY, an IDENTITY of `_cap_free`'s branch condition:");
    p.print(pct!("    {the knob is live} = %d      {the leg reaches applied fuel} = %d",
                 m.n_live, m.n_reach));
    p.print(pct!("    INTERSECTION = %d, and %d + %d = %d: the two sets PARTITION the calls.",
                 m.n_both, m.n_live, m.n_reach, m.n_log));
    p.print("    `_cap_free` short-circuits to `_surge_fuel` iff the cap lies BELOW the");
    p.print("    schedule, i.e. iff the leg BINDS -- and `_surge_fuel` brackets its OWN");
    p.print("    hardcoded phi residual, SUBSTITUTING THE ORIGINAL COORDINATE BACK IN. It");
    p.print("    brackets the coordinated residual only when `_applied_demand`'s second min");
    p.print("    throws the cap away. So the knob is unreachable in BOTH regimes, for two");
    p.print("    different reasons, and there is no third regime. (`binds` is NOT the last");
    p.print(pct!("    selector -- %d/%d wins the INNER min and is still consistent with total",
                 m.binds, m.n_log));
    p.print(pct!("    masking; %d is the honest number.)", m.n_reach));

    let f = coord_forced(&fresh(), flight, LO, HI, TMAX, phi, 0.10, TAUS, false, 0.5, 1.2, 0.005,
                         0.20, 8);
    p.print("\n  s 5.2 -- THE SHORT-CIRCUIT BYPASSED, which is what makes s 5 readable at all:");
    p.print(pct!("    binding at %d/%d, d_forced max/med = %.3e / %.3e (a few ulp), same float %d",
                 f.n_binding, f.n, f.d_forced.expect("a forced solve binds"),
                 f.d_forced_med.expect("a forced solve binds"), f.n_same_float));
    p.print(pct!("    of %d -- and forced vs the SHIPPED solve = %.3e, so the bypass is measuring",
                 f.n, f.d_shipped.expect("a forced solve binds")));
    p.print("    the SAME set point and not a nearby one. The coordinate is root-preserving to");
    p.print("    ~1e-15 and the short-circuit then ROUNDS THAT TO EXACTLY ZERO. An exact");
    p.print("    invariance and an UNMEASURED one are indistinguishable in the plant, and here");
    p.print("    they differ by 1e-15.");

    p.print("\n  WHAT THIS SAYS. This BOUNDS rung 78 rather than correcting it: uniqueness");
    p.print("    survives THIS gauge because h > 0 strictly, while rung 78's loss belonged to");
    p.print("    its affine LAW-side family, whose `1 - k*c` passes through zero. `Cannot be");
    p.print("    driven singular` here is not *we swept and never reached zero* -- there is");
    p.print("    NOTHING TO SWEEP, because a coordinate must be invertible. The root-finder is");
    p.print("    itself the diagnostic: rung 78 needed a damped Newton, this rung reuses rung");
    p.print("    74's shipped bracket unchanged, with zero failures. And the masking is a");
    p.print("    THIRD mechanism -- distinct from rung 72's `min`-mask and rung 76's law-vs-");
    p.print("    plant split -- consequent on WHERE A SOLVER SHORT-CIRCUITS, which no rung had");
    p.print("    looked at. Zero new constants. See docs/rung79-spec.md.");
}

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
