//! SLICE AH step 4 — **RUNG 78 §§ 1–3: THE FREE DIAL, THE INVARIANT SENSITIVITY, AND THE SECOND
//! ROOT.**
//!
//! # WHAT THIS FILE GATES
//!
//! Step 1 shipped rung 78's plumbing (the knob, its two refusals, the four re-aimed pointers) and
//! steps 2–3 closed rung 77. This step ships the rung's first three sections:
//! [`gauge_scan`] (§ 1/§ 2 — `w*(k)`, `G_w(k)`, `dw*/dq(k)`) and [`root_census`] (§ 3 — the
//! collision), together with [`root_count`], [`accel_cap_fn`], [`gauge_points`] and the sixth
//! knob's **two** restore guards.
//!
//! Every numeric bar below is quoted from `tests/test_rung78.py`'s own `assert` lines —
//! `Gw_err < 1e-6`, `Gw_span` outside `(-1, 1)`, `c_err < 1e-8`, `w_move < 1e-9`,
//! `gain_move < 1e-6`, `n_excluded > 0`, `n_kept > n_excluded`, `excluded_worst > 1e-3`,
//! `G_at_w0 < 1e-14`, `max(n_roots) > 1`, `approach < 0.2` — **not from the narrative beside
//! them.** Step 3's failure was two bars typed from a § 4 narrative that the suite returns and
//! gates nowhere, so the asserts were read this time and the prose used only to cross-check.
//! The discrete counts (10 rows, 30 of 100 readings excluded, the excluded multiples, the
//! multi-root band) were measured by running the PYTHON reader before any Rust was written.
//!
//! # THE FILE'S OWN FINDING — **THE SLICE'S LEADING HAZARD, ON A SECOND VARIABLE, WHERE NO CENSUS
//! WAS LOOKING**
//!
//! Plan § 5.32 (i) opens on one hazard given three incompatible treatments inside these two
//! classes: the `_b_state`/`_v_state` freeze, nested at six sites, re-armed at one, deliberately
//! avoided at another. That census was scoped to the freeze pair — and **the sixth declared knob
//! has exactly the same shape, one section apart, unbooked**:
//!
//! | site | how `_gauge_k` is put back |
//! |---|---|
//! | `engine.py:20394` / `engine.py:20404` | hand-rolled, saves `prev` and restores `prev` |
//! | `engine.py:20508` / `engine.py:20514` | hand-rolled, restores the **literal `1.0`** |
//! | `engine.py:20705` / `engine.py:20714` | the declared helper `_with_gauge`, which saves `prev` |
//!
//! The class **declares** the helper that would make this structural and then does not use it in
//! either of the two sections that need it most, and one of those substitutes a clobber for a
//! restore. That is step 3's lesson promoted from a sentence to a pattern: the discipline is
//! per-call-site, and a class with three call sites gets three answers.
//!
//! **AND THE CLOBBER IS REACHABLE-WRONG INSIDE ONE CALL.** `_shared_rig` propagates the caller's
//! gauge onto the marched machine (`engine.py:20332`), so a census entered at a non-identity gauge
//! builds row 1's cap under that gauge and every later row under the identity — different plants
//! in one table. Measured in Python at `k = 2.5`, the gauge at each row's `_accel_cap_fn` is
//! `[2.5, 1.0, 1.0]` for `root_census` and `[2.5, 2.5, …]` for `gauge_scan`.
//! [`the_census_clobbers_the_gauge_where_the_scan_restores_it`] reproduces both sequences by hand
//! on the ported guards. It is invisible today only because every shipped caller enters at the
//! identity.
//!
//! # THE MIRROR OF STEP 3, AND WHY IT NEEDS A GATE RATHER THAN A NOTE
//!
//! `singular_limit` DEPENDS on a residual outliving its freeze block; `gauge_scan`'s `at` depends
//! on the opposite, and there the natural Rust shape — rebuild everything in a scope — is the
//! CORRECT one. So this port gets it right by accident, which is exactly when a gate is worth
//! most. The discriminator is in the arithmetic: a leaked freeze makes `at` ignore its `qq`, so
//! `at(q + dq)` and `at(q - dq)` would agree bit for bit and `direct` would be an exact `0.0`.
//! [`the_perturbed_solves_see_different_plants`] asserts it is not, at every gauge in every row.
//!
//! # AND THE TWO ARMS OF `root_count`'s FALLIBILITY ARE GATED BY CONSTRUCTION
//!
//! Python catches `AssertionError` on the initial walk and **not** in the bisection. Measured over
//! the two shipped grids the walk arm fires 6 171 of 39 930 and 5 980 of 40 100 points — a sixth
//! of every walk — while the bisection arm fires **0 of 25 200 and 0 of 9 720**. A zero is a
//! property of the grid, not of the code, so both arms are driven here by hand-built residuals
//! rather than left to the sweep: one that refuses on part of the range must be absorbed, one that
//! refuses at a bracket's midpoint must propagate.
//!
//! # THE GATES WERE MUTATED, AND THE FIRST ELEVEN COULD NOT SEE THREE OF SIX DEFECTS
//!
//! Six plausible slips were injected into the port one at a time and the whole file re-run:
//!
//! | injection | seen by |
//! |---|---|
//! | the walk PROPAGATES a refusal instead of absorbing it | 10 gates |
//! | the census's clobber TIDIED into a restore | 1 |
//! | the bisection budget halved (60 → 30) | 1 |
//! | the slope read at the SOLVED root instead of the anchor | **0 — then 1** |
//! | the exclusion reads only its own gauge, not its two neighbours | **0** |
//! | `mult = 0.0` divides instead of short-circuiting | **0** |
//!
//! The slope one is the repair: it was invisible because the cells where the two readings differ
//! are exactly the cells the exclusion drops, so no aggregate could ever contain one.
//! [`the_solver_lands_on_the_spurious_root_and_the_anchor_is_what_keeps_the_slope_honest`] now
//! bounds `gw` against `1 - k*c` over EVERY cell, dropped ones included — which holds precisely
//! because the anchor is a root at every gauge, and fails the moment the slope is read anywhere
//! else. **The first writing of that gate did not catch it either**: it demonstrated the physics
//! by calling `slope_at` directly and never went through the shipped reader.
//!
//! The other two are recorded rather than gated, because on this grid neither is observable:
//!
//! * the three-argument exclusion `max(nr, hi, lo) > 1` never disagrees with the CENTRE's own
//!   count: measured across the shipped sweep, `n_roots > 1` and the shipped `excluded` flag give
//!   the same answer at **100 of 100 cells**. (What was measured is the centre against the shipped
//!   three-way result; what the two neighbours individually return was not read.) So the two extra
//!   reads are a defence with no reader — slice AA's pattern — and inventing a gate for it would
//!   mean inventing a grid the rung does not run on;
//! * `0.0 / c` is exactly `0.0` for every finite positive `c`, so Python's falsy short-circuit is
//!   arithmetically a no-op here. It could only separate at `c = 0`, `±inf` or NaN, none of which
//!   `c_at` returns on this march.

use std::ptr::fn_addr_eq;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::PointExtra;
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::residual_gauge::{
    accel_cap_fn, build_residual_gauge_cascade, gauge_points, gauge_residual, gauge_root,
    gauge_scan, root_census, root_count,
    GaugeClobbered, GaugeRestored, GaugeScan, RootCensus, GAUGE_K_IDENTITY, GAUGE_ROOT_N, GAUGE_ROOT_REL,
    GAUGE_ROOT_TRUST, GAUGE_SCAN_DQ, GAUGE_SCAN_MULTS, ROOT_CENSUS_MULTS, ROOT_CENSUS_N, ROOT_COUNT_HI, ROOT_COUNT_LO,
    ROOT_COUNT_N, R78_TRIPLE,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::sensed_cap::{c_at, C_AT_REL};
use turbojet::demand_coordinate::cap_free;
use turbojet::stiffness_ledger::{slope_at, R77_TRIPLE, SLOPE_AT_REL};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{MarchedBleed, MarchedStator};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung78.py`'s module constants — identical to rung 77's, which is `_gauge_points`'
// whole point: § 1's `k = 1` column IS rung 77 § 1 and can be differenced against it.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const SM: f64 = PHI / FLOOR - 1.0;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const TT4_MAX: f64 = 1200.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const MARGIN: f64 = 0.10;
const EVERY: usize = 8;

// The bars `tests/test_rung78.py` states in its own `assert` lines.
const GW_ERR_BAR: f64 = 1e-6;
const C_ERR_BAR: f64 = 1e-8;
const W_MOVE_BAR: f64 = 1e-9;
const GAIN_MOVE_BAR: f64 = 1e-6;
const EXCLUDED_WORST_BAR: f64 = 1e-3;
const G_AT_W0_BAR: f64 = 1e-14;
const APPROACH_BAR: f64 = 0.2;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn arm() -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    }
}

fn gauge() -> ScheduledStatorCore {
    match build_residual_gauge_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm())
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

fn scan() -> GaugeScan {
    gauge_scan(&gauge(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R, SETTLE, DS,
               V_MAX, GAUGE_SCAN_DQ, EVERY, &GAUGE_SCAN_MULTS)
}

fn census() -> RootCensus {
    root_census(&gauge(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R, SETTLE, DS,
                V_MAX, EVERY, &ROOT_CENSUS_MULTS, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_CENSUS_N)
}

// ---------------------------------------------------------------------------- THE REDUCE

/// `_gauge_k = 1.0` must take the PARENT's `_cap_fuel` by DISPATCH, not by an algebraically-equal
/// copy — and the gauged residual at the identity must be the shipped EXPRESSION, with the anchor
/// absent from it entirely.
#[test]
fn the_identity_gauge_is_rung_77_by_dispatch_and_the_anchor_is_absent_from_it() {
    let c = gauge();
    assert_eq!(c.fuel.inner.gauge_k.get(), GAUGE_K_IDENTITY, "the identity gauge is the default");
    assert!(
        !fn_addr_eq(R78_TRIPLE.cap_fuel, R77_TRIPLE.cap_fuel),
        "rung 78 declares its own `cap_fuel`; if it did not there would be nothing to reduce");
    // The suite's third assert: at `k = 1` the gauged residual is the SHIPPED EXPRESSION, so the
    // anchor does not appear in it. A NONSENSE anchor must therefore be ignored, EXACTLY.
    let cap = |w: f64| -> Result<f64, Abort> { Ok(0.3 * w + 1.0) };
    let g1 = gauge_residual(GAUGE_K_IDENTITY, &cap, 999.0);
    for w in [0.5, 1.0, 2.0] {
        assert_eq!(g1(w).expect("this cap never refuses").to_bits(), (w - (0.3 * w + 1.0)).to_bits(),
                   "the identity gauge must be `w - cap(w)` bit for bit, with no anchor in it");
    }
}

// ---------------------------------------------------------------------------- § 1 / § 2

/// **P2**: `G_w == 1 - k*c`, spanning BOTH signs — the dial reaches zero and goes out the far
/// side. And the knob is WIRED: some gauge moved the slope off the `k = 1` column.
#[test]
fn the_slope_is_the_predicted_free_dial() {
    let s = scan();
    let moved = s.rows.iter().flat_map(|x| x.ks.iter().map(move |d| (x.gw1, d)))
        // Python's `d["mult"] not in (1.0,)`. INERT in both languages — `GAUGE_SCAN_MULTS` has
        // no `1.0` — so it reads as a guard and filters nothing. Kept because the source has it.
        .filter(|(gw1, d)| d.mult != 1.0 && (d.gw - gw1).abs() > 1e-3)
        .count();
    assert!(moved > 0, "no gauge changed the residual slope — the knob is not wired");
    assert_eq!(s.sign_change, Some(true), "the slope never changed sign — the dial missed zero");
    let e = s.gw_err.expect("the sweep keeps readings");
    assert!(e < GW_ERR_BAR, "Gw_err = {e:.3e}");
    let (lo, hi) = s.gw_span.expect("the sweep keeps readings");
    assert!(lo < -1.0 && hi > 1.0, "Gw_span = ({lo}, {hi})");
}

/// **NON-VACUITY**: without this the sweep could be measuring its own root finder. The shipped
/// `gauge_root` is a damped Newton and shares no convergence test with `c_at`, so agreement with
/// rung 76's `c` is what pins the instrument.
#[test]
fn the_k_one_column_is_rung_76s_c() {
    let e = scan().c_err.expect("the sweep has rows");
    assert!(e < C_ERR_BAR, "c_err = {e:.3e}");
}

/// **P1 and P3 — THE RUNG.** The set point does not move and neither does `dw*/dq`, including
/// where `G_w < 0`: the `c -> 1` singularity is REMOVABLE, so `1/(1-c)` is a GAUGE.
#[test]
fn the_set_point_and_the_sensitivity_are_both_gauge_invariant() {
    let s = scan();
    let w = s.w_move.expect("the sweep keeps readings");
    let g = s.gain_move.expect("the sweep keeps readings");
    assert!(w < W_MOVE_BAR, "w_move = {w:.3e}");
    assert!(g < GAIN_MOVE_BAR, "gain_move = {g:.3e}");
    assert_eq!(s.n_bad, 0, "a kept reading did not converge");
}

/// **A RUNG THAT DROPS POINTS MUST SAY WHAT IT DROPPED, AND THE DROPPED ONES MUST MATTER.**
/// Otherwise the exclusion is decoration and the hold is a choice of where to look.
///
/// The three bars are the suite's; the counts beside them were measured by running the PYTHON
/// reader, and they are what discriminates a correct [`root_count`] port — the P1/P2/P3 aggregates
/// are all computed over `keep` and would look clean with the exclusion mis-ported.
#[test]
fn the_exclusion_is_measured_and_not_free() {
    let s = scan();
    assert!(s.n_excluded > 0, "nothing was excluded; the sweep never entered the band");
    assert!(s.n_kept > s.n_excluded,
            "{} of {} readings dropped — that is not an exclusion", s.n_excluded,
            s.n_kept + s.n_excluded);
    let worst = s.excluded_worst.expect("something was excluded");
    assert!(worst > EXCLUDED_WORST_BAR,
            "the excluded points moved only {worst:.2e} — if they are harmless then excluding \
             them bought a hold this rung did not need, and § 1.2 is wrong");
    // Measured from the Python reader before any Rust existed.
    assert_eq!((s.n, s.n_excluded, s.n_kept), (10, 30, 70));
    assert_eq!(s.excluded_mults, vec![0.9, 1.05, 1.1],
               "the excluded set is exactly the three multiples straddling `k*c = 1`");
    // The refuted `1e-3` window would have called all three of these clean.
    for m in &s.excluded_mults {
        assert!((1.0 - m).abs() > 1e-3 || *m == 1.0,
                "{m} sits inside the anchor's refuted window, which is the point");
    }
}

/// **THE MIRROR OF STEP 3.** A leaked freeze would make `at` ignore its `qq`, so the two perturbed
/// solves would agree bit for bit and `direct` would be an exact zero. Here the natural Rust shape
/// is the correct one, so this is asserted rather than assumed.
#[test]
fn the_perturbed_solves_see_different_plants() {
    let s = scan();
    let seen: usize = s.rows.iter().map(|x| x.ks.len()).sum();
    assert_eq!((s.rows.len(), seen), (10, 100), "a vacuous sweep would pass every check below");
    for x in &s.rows {
        assert_ne!(x.base.to_bits(), 0f64.to_bits(),
                   "the k = 1 reference is an exact zero — the freeze leaked");
        for d in &x.ks {
            assert!(d.direct != 0.0 && d.direct.is_finite(),
                    "mult {} gave direct = {:?}", d.mult, d.direct);
        }
    }
}

// ---------------------------------------------------------------------------- § 3

/// The construction, CHECKED: `w0` is a root of `G_k` at EVERY gauge, and the walk always finds
/// it. This is algebra, and it is gated because it is the one thing that would make §§ 1–2
/// meaningless.
#[test]
fn the_construction_is_exact_on_the_plant() {
    let c = census();
    let g = c.g_at_w0.expect("the census has cells");
    assert!(g < G_AT_W0_BAR, "G_at_w0 = {g:.3e}");
    assert!(c.true_found, "the walk lost the true root");
}

/// **THE OTHER HALF OF THE HEADLINE**: a second root sweeps in, collides with the true one at
/// `k*c = 1`, and departs upward — so inside the band a solver returns *a* root, not *the* root.
#[test]
fn the_gauge_destroys_uniqueness() {
    let c = census();
    assert!(*c.n_roots.last().expect("the census has cells") > 1, "n_roots = {:?}", c.n_roots);
    assert_eq!(c.brackets, Some(true),
               "the multi-root band {:?} does not bracket the singular gauge — the collision is \
                then not at `k*c = 1` and § 3's mechanism is wrong", c.band);
    let a = c.approach.expect("some cell is multi-rooted");
    assert!(a < APPROACH_BAR, "approach = {a:.3e}");
    // Measured from the Python reader: the band, and that the walk resolves up to FOUR roots.
    assert_eq!(c.n, 10);
    assert_eq!(c.n_roots, vec![1, 2, 4]);
    assert_eq!(c.multi_mults, vec![0.9, 0.99, 1.01, 1.05, 1.1, 1.2]);
    assert_eq!(c.band, Some((0.9, 1.2)));
}

/// **§ 3's HEADLINE ON ONE CELL — AND THE ONLY GATE THAT CAN SEE WHY THE SLOPE IS READ AT THE
/// ANCHOR.** Six defects were injected into this port; three were invisible to every other gate in
/// this file, and reading the slope at the SOLVED root instead of the anchor was one of them,
/// because the cells where the two differ are exactly the cells the exclusion removes.
///
/// So the cell is rebuilt by hand. At the second riding point, gauge `1.1/c`, the residual has two
/// roots — the true one and one at `1.613·w0` — and the damped Newton started from `mf_sched`
/// converges **cleanly, reporting success**, onto the spurious one. The slope there is POSITIVE
/// where the prediction `1 - k*c` is negative: reading `G_w` at the solved root would report the
/// wrong SIGN, which is precisely what Python's comment says and what nothing else here tests.
#[test]
fn the_solver_lands_on_the_spurious_root_and_the_anchor_is_what_keeps_the_slope_honest() {
    let c = gauge();
    let fl = flight();
    let (m, _surge, accel, pts) = gauge_points(
        &c, &fl, LO, HI, TT4_MAX, MARGIN, TAUS, R, SETTLE, DS, V_MAX, false, PHI, EVERY);
    let p = &pts[1];
    let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let (q, v) = match p.extra {
        PointExtra::Demand { b, v, .. } => (b, v),
        PointExtra::Shared { b, v, .. } => (b, v),
        _ => panic!("the demand march carries `b`/`v` on every point"),
    };
    let _sb = MarchedBleed::set(&m.fuel.inner, q);
    let _sv = MarchedStator::set(&m.fuel.inner, v);
    let cap = accel_cap_fn(&m.fuel, &fl, a, h, &accel);
    let g0 = |w: f64| -> Result<f64, Abort> { Ok(w - cap(w)?) };
    let w0 = cap_free(&g0, ms, &|| m.fuel.try_sched_fuel(&fl, a, h, ms, &accel))
        .expect("the k = 1 anchor solves here");
    let cc = c_at(&m, &fl, a, h, &accel, w0, q, v, C_AT_REL).expect("rung 76's c");
    let _sb2 = MarchedBleed::set(&m.fuel.inner, q);
    let _sv2 = MarchedStator::set(&m.fuel.inner, v);

    let k = 1.1 / cc;
    let _gk = GaugeRestored::set(&m.fuel.inner, k);
    let big_g = gauge_residual(m.fuel.inner.gauge_k.get(), &*cap, w0);
    let roots = root_count(&*big_g, w0, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_COUNT_N, true)
        .expect("no refusal reaches the bisection here");
    assert_eq!(roots.len(), 2, "roots = {roots:?}");
    assert!((roots[0] - 1.0).abs() < 1e-9, "the true root survives: {roots:?}");
    assert!((roots[1] - 1.613).abs() < 5e-3, "the spurious root: {roots:?}");
    // The cell is selected by INDEX, so pin what makes it the right cell: if the march ever
    // shifts, this must fail loudly rather than quietly assert the same things about another one.
    assert!(roots[1] - 1.0 > 0.5,
            "pts[1] at mult 1.1 is chosen because its second root is FAR from the true one;              the grid has moved and this gate is now testing a different cell: {roots:?}");

    let (w, ok) = gauge_root(&*big_g, ms, GAUGE_ROOT_REL, GAUGE_ROOT_N, GAUGE_ROOT_TRUST);
    assert!(ok, "the solver REPORTS SUCCESS — that is what makes the second root dangerous");
    assert!((w / w0 - roots[1]).abs() < 1e-6,
            "it converged to the SPURIOUS root: w/w0 = {}, roots = {roots:?}", w / w0);

    let at_anchor = slope_at(&*big_g, w0, SLOPE_AT_REL).expect("the anchor is a root at every k");
    let at_root = slope_at(&*big_g, w, SLOPE_AT_REL).expect("and the solver's root is one too");
    assert!((at_anchor - (1.0 - k * cc)).abs() < 1e-6,
            "the anchor reports the PREDICTION: {at_anchor} vs {}", 1.0 - k * cc);
    assert!(at_anchor < 0.0 && at_root > 0.0,
            "the two slopes must differ in SIGN, else this gate is not the one that sees it:              anchor {at_anchor}, root {at_root}");
    drop(_gk);

    // AND THE CONSEQUENCE FOR THE SHIPPED READER, which is what actually catches a scan that read
    // the solved root instead: because the anchor is a root at EVERY gauge, `1 - k*c` holds at
    // every cell — INCLUDING the ones the sweep drops. The exclusion is about the SET POINT's
    // well-posedness, not about the slope, so `gw_err` is bounded off the kept set too.
    let sc = scan();
    let mut worst = (0.0f64, f64::NAN, false);
    for x in &sc.rows {
        for d in &x.ks {
            let ap = d.gw_pred.abs();
            let den = if 1e-30 > ap { 1e-30 } else { ap };
            let e = (d.gw - d.gw_pred).abs() / den;
            if e > worst.0 { worst = (e, d.mult, d.excluded) }
        }
    }
    assert!(worst.0 < GW_ERR_BAR,
            "the slope prediction must hold at every cell, dropped ones included; worst {:.3e}              at mult {} (excluded = {})", worst.0, worst.1, worst.2);
    assert!(sc.rows.iter().flat_map(|x| x.ks.iter()).any(|d| d.excluded),
            "and some cell must actually be excluded, or the sentence above is vacuous");
}

// ---------------------------------------------------------------------------- THE FILE'S FINDING

/// **THE SIXTH KNOB HAS TWO RESTORE POLICIES IN ONE CLASS, AND ONE OF THEM IS A CLOBBER.**
///
/// The two guards are pinned directly, and then the consequence is reproduced by hand on the
/// machine `root_census` actually marches: with the caller's gauge propagated onto it by
/// `_shared_rig`, the census's policy shows the outer gauge at row 1 and the IDENTITY at every
/// later row, while the scan's policy shows the outer gauge throughout.
#[test]
fn the_census_clobbers_the_gauge_where_the_scan_restores_it() {
    let c = gauge();
    let k = &c.fuel.inner.gauge_k;

    k.set(2.5);
    {
        let _g = GaugeRestored::set(&c.fuel.inner, 7.0);
        assert_eq!(k.get(), 7.0);
    }
    assert_eq!(k.get(), 2.5, "`gauge_scan`'s policy RESTORES what was there");

    k.set(2.5);
    {
        let _g = GaugeClobbered::set(&c.fuel.inner, 7.0);
        assert_eq!(k.get(), 7.0);
    }
    assert_eq!(k.get(), GAUGE_K_IDENTITY,
               "`root_census`'s policy clobbers to the literal identity — `engine.py:20514`");

    // The consequence, on the marched machine, over three rows.
    k.set(1.0);
    let fl = flight();
    let (m, _surge, accel, pts) = gauge_points(
        &c, &fl, LO, HI, TT4_MAX, MARGIN, TAUS, R, SETTLE, DS, V_MAX, false, PHI, 32);
    assert!(pts.len() >= 3, "the finding needs at least three rows; got {}", pts.len());
    let mut clobbered: Vec<f64> = Vec::new();
    let mut restored: Vec<f64> = Vec::new();
    for policy in 0..2 {
        m.fuel.inner.gauge_k.set(2.5);
        for p in pts.iter().take(3) {
            let (q, v) = match p.extra {
                PointExtra::Demand { b, v, .. } => (b, v),
                PointExtra::Shared { b, v, .. } => (b, v),
                _ => panic!("the demand march carries `b`/`v` on every point"),
            };
            let _sb = MarchedBleed::set(&m.fuel.inner, q);
            let _sv = MarchedStator::set(&m.fuel.inner, v);
            // What `_accel_cap_fn` would be built under, at the top of this row.
            let seen = m.fuel.inner.gauge_k.get();
            if policy == 0 { clobbered.push(seen) } else { restored.push(seen) }
            let _cap = accel_cap_fn(&m.fuel, &fl, p.nu_lp, p.nu_hp, &accel);
            if policy == 0 {
                let _g = GaugeClobbered::set(&m.fuel.inner, 3.0);
            } else {
                let _g = GaugeRestored::set(&m.fuel.inner, 3.0);
            }
        }
    }
    m.fuel.inner.gauge_k.set(GAUGE_K_IDENTITY);
    assert_eq!(clobbered, vec![2.5, 1.0, 1.0],
               "the census's clobber changes the plant between rows");
    assert_eq!(restored, vec![2.5, 2.5, 2.5], "the scan's restore does not");
}

// ---------------------------------------------------------------------------- `root_count`

/// **THE TWO FALLIBILITY ARMS, DRIVEN BY HAND.** The walk's `except AssertionError` fires on a
/// sixth of every shipped point; the bisection's never fires on any shipped grid, so the only way
/// to know the port kept Python's asymmetry is to build a residual that reaches it.
#[test]
fn the_walk_absorbs_a_refusal_and_the_bisection_propagates_one() {
    // A residual with one sign change at x = 1.5, refusing on [2.4, 3.0] — well clear of it.
    let absorbing = |x: f64| -> Result<f64, Abort> {
        if x >= 2.4 { Err(Abort("the model does not reach here".into())) } else { Ok(x - 1.5) }
    };
    let roots = root_count(&absorbing, 1.0, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_COUNT_N, true)
        .expect("a refusal on the WALK is absorbed into a `None`");
    assert_eq!(roots.len(), 1, "roots = {roots:?}");
    assert!((roots[0] - 1.5).abs() < 1e-12, "roots = {roots:?}");

    // The same sign change, but the residual refuses strictly INSIDE the bracket, which only the
    // bisection can reach: the grid points at 1.4875 and 1.5125 are both fine.
    let propagating = |x: f64| -> Result<f64, Abort> {
        if (x - 1.5).abs() < 1e-3 { Err(Abort("refused mid-bracket".into())) } else { Ok(x - 1.5) }
    };
    assert!(
        root_count(&propagating, 1.0, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_COUNT_N, true).is_err(),
        "a refusal in the BISECTION propagates — Python leaves that call uncaught");

    // `locate = false` returns bracket midpoints unrefined, which is why § 1 can afford the walk.
    let coarse = root_count(&absorbing, 1.0, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_COUNT_N, false)
        .expect("the walk still absorbs");
    assert_eq!(coarse.len(), 1);
    assert!((coarse[0] - 1.5).abs() > 1e-12, "unrefined, so it is NOT the exact root");
}

/// A residual with no sign change has no roots, and one that refuses everywhere has none either —
/// the empty-walk arm, which no shipped grid reaches.
#[test]
fn a_walk_that_finds_nothing_returns_nothing() {
    let flat = |_x: f64| -> Result<f64, Abort> { Ok(1.0) };
    assert!(root_count(&flat, 1.0, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_COUNT_N, true)
        .expect("no refusal").is_empty());
    let dead = |_x: f64| -> Result<f64, Abort> { Err(Abort("nothing is modelled".into())) };
    assert!(root_count(&dead, 1.0, ROOT_COUNT_LO, ROOT_COUNT_HI, ROOT_COUNT_N, true)
        .expect("every point becomes a `None`, and `None` pairs are SKIPPED").is_empty());
}
