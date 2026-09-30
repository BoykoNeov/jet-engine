//! RUNG 82 — **THE THRESHOLD'S OWN LAW**: `docs/rung81-spec.md` § 8's first seam, answered NO.
//!
//! **HEADLINE: a forward reading inherits the SIGN of its own reference.** At one ramp, whether
//! the criterion's forward prediction lands above or below the true threshold follows the side
//! its REFERENCE march sat on — so it reports where the reader STARTED, and only the fixed point,
//! which has no reference, lands. And the criterion's terms are not independent coordinates: the
//! wall is the largest mover of `ċ_f`.
//!
//! Ported from `tests/test_rung82.py`: **16 collected tests, 12 of them `slow` there** (the port
//! has no tier). **This file has 18**: the suite's 16, 1:1 in order, plus two of the ten driven
//! voids (plan § 5.34 (vi)), DECLARED below.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, **2 ADDED**, 0 COLLAPSED, **1 SUBSTITUTED**
//!
//! | # | `tests/test_rung82.py` | here |
//! |---|---|---|
//! | 1 | `reduce_is_bit_for_bit_rung81` | [`reduce_the_scan_reads_split_marchs_own_march`] — **SUBSTITUTED**, below |
//! | 2 | `the_identity_control_reproduces_rung81s_own_cell` | [`the_identity_control_reproduces_rung81s_own_cell`] |
//! | 3 | `V1_no_four_loop_window_above_the_admissible_ramp` | [`v1_no_four_loop_window_above_the_admissible_ramp`] |
//! | 4 | `the_threshold_rises_monotonically_with_the_ramp` | [`the_threshold_rises_monotonically_with_the_ramp`] |
//! | 5 | `the_effective_clock_is_the_release_one_everywhere` | [`the_effective_clock_is_the_release_one_everywhere`] |
//! | 6 | `the_fixed_point_is_never_beaten_by_the_forward_reading` | [`the_fixed_point_is_never_beaten_by_the_forward_reading`] |
//! | 7 | `the_fixed_point_sits_below_the_measured_threshold` | [`the_fixed_point_sits_below_the_measured_threshold`] |
//! | 8 | `the_step_control_in_its_own_currency` | [`the_step_control_in_its_own_currency`] |
//! | 9 | `the_unrun_step_control_does_not_read_as_a_passed_one` | [`the_unrun_step_control_does_not_read_as_a_passed_one`] |
//! | 10 | `the_forward_reading_inherits_its_references_sign` | [`the_forward_reading_inherits_its_references_sign`] |
//! | 11 | `the_bisection_resolves_the_signs_it_is_asked_about` | [`the_bisection_resolves_the_signs_it_is_asked_about`] |
//! | 12 | `the_map_contracts_from_above_and_diverges_from_below` | [`the_map_contracts_from_above_and_diverges_from_below`] |
//! | 13 | `the_governor_clock_keeps_only_part_of_its_coefficient` | [`the_governor_clock_keeps_only_part_of_its_coefficient`] |
//! | 14 | `the_wall_moves_the_threshold_the_other_way` | [`the_wall_moves_the_threshold_the_other_way`] |
//! | 15 | `the_terms_do_not_separate_and_the_wall_moves_the_wrong_slope` | [`the_terms_do_not_separate_and_the_wall_moves_the_wrong_slope`] |
//! | 16 | `V3_censors_the_wall_on_both_sides` | [`v3_censors_the_wall_on_both_sides`] |
//! | **+1** | **— none —** | [`void_v1_a_bracket_end_with_no_window`] (`engine.py:22302`) |
//! | **+2** | **— none —** | [`void_v3_the_threshold_outside_the_bracket`] (`engine.py:22305`) |
//!
//! # WHERE THE PORT DIFFERS FROM A LINE-BY-LINE COPY
//!
//! * **#1 is SUBSTITUTED, for the reason `rung81.rs` #1 gives:** Python compares a
//!   `ThresholdLawTransient` march with an `AuthorityClockTransient` one — two classes — and the
//!   port has one `R80` core for both, so a verbatim copy compares `split_march` with itself. The
//!   gate instead requires [`threshold_scan`]'s reduction at FOUR DIFFERENT clocks
//!   `(0.12, 0.02, 0.05, 0.08)` to equal an independent `split_march` at the same four, read off
//!   the points' own fields — so a swapped pair among `tau_f`/`tau_gov`/`tau_q`/`tau_s`, or a
//!   dropped `r`/`ds`/wall, cannot pass. Python's `341` and its four compared keys are kept on the
//!   MATCHED march at rung 81's ramp, every point asserted a `Demand` point.
//! * **The fixtures TYPE the suite's literals, not the crate's defaults:** `law` runs
//!   `rs = (0.25, 0.35, 0.50)` and `ds_fine = None` (the crate's `LAW_RS` is five ramps and its
//!   `DS_FINE` runs the step control); `terms` runs three walls, not `TERMS_PHI_LIMS`' five.
//! * **`_bisect(lambda s: s["n_fuel"] > 0, …)` is the literal closure** `|s| s.n_fuel > 0`, not
//!   `key_fuel`.
//! * **`hi_wall.get("void")` is `Bisect::void()`; `hi_wall["below"]` is the `VoidStraddle`
//!   field** — a `KeyError` in Python on any other shape is a panic by name here.
//! * **`x is False` / `x is True` on an `Option<bool>`** is `== Some(false)` / `== Some(true)`,
//!   since Python's `None is False` is `False`.
//!
//! # THE TWO ADDED GATES — driven at plan § 5.34 (vi)'s inputs, strings from the source
//!
//! Both at `n = 0` (`p_drive.py`'s inputs; no bisection step runs), each string written from
//! `engine.py`'s literal and Python's `%g`, then MEASURED on PyPy
//! (`W:\temp\claude\slice-aj-step5\probe_voids.py`, log opens `3.11.15 … [PyPy 7.3.23 …]`).
//!
//! * **+1, V1** at `r = 1.0`: the `0.004` end has no window, the `0.30` end HAS one (step 3's
//!   finding) — so the void fires off the LOW end alone, and the ok ends are carried back.
//!   **It cannot see the ORDER of `_bisect`'s two checks**, and this was MISPREDICTED at step 5's
//!   sweep (injection I13, the straddle test moved above the window test, SURVIVED all four
//!   ported suites): here the straddle HOLDS (low end `n_fuel = 0`, high end all fuel), so a
//!   port testing V3 first falls through to the window test and returns the same V1. Only a
//!   bracket with BOTH ends empty sees the order; `slice_aj_threshold.rs`'s
//!   `v1_both_ends_empty_bit_for_bit` does, and killed I13 alone (plan § 5.34.5).
//! * **+2, V3** at `r = 0.35`, `phi_lim = 0.760`: `below` and not `above`. **Its message cannot
//!   pin `py_g`:** `%g` and Rust's `{}` both print `0.004` and `0.3`, so a port formatting with
//!   `{}` passes here. Step 4's `sn_v5`/`root_v6` readings are the ones that pin the formatter.

use std::sync::OnceLock;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Authority, FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::riding4_idx;
use turbojet::split_wall::{build_split_wall_cascade, split_march};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::threshold_law::{
    bisect, threshold_law, threshold_reference, threshold_scan, threshold_terms, Bisect,
    RefFull, RowOk, ScanKw, ThresholdLaw, ThresholdReference, ThresholdScan, ThresholdTerms,
};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung82.py`'s module constants, verbatim.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const TT4_MAX: f64 = 1200.0;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
/// Rung 80's walls, unchanged.
const PHI_FUEL: f64 = 0.75;
const PHI_AIR: Option<f64> = Some(0.77);
/// Rung 81's clocks: the identity control.
const MATCHED: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
/// Rung 81's own ramp.
const R81: f64 = 0.5;
/// The ramp set is trimmed against the spec's; the RESOLUTION is not (the suite's own comment).
const RS: [f64; 3] = [0.25, 0.35, 0.50];
const N_BISECT: usize = 10;

// The readers' unspelled defaults (`engine.py:22320`, `:22438`, `:22511`), typed.
const BRACKET: (f64, f64) = (0.004, 0.30);
const S_SETTLE: f64 = 1.2;
const DS: f64 = 0.005;

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

/// Python's `_rig(design)`. `ThresholdLawTransient` there; a rung-80 core here (plan § 5.34 (ii)).
fn rig() -> ScheduledStatorCore {
    let sm = 0.80 / FLOOR - 1.0;
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    };
    let m = match build_split_wall_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    };
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.cap_law.set("solve");
    m
}

/// The suite's `base = dict(...)` for `_threshold_scan` / `_bisect`, clocks 0.05.
fn kw(f: &FlightCondition, r: f64, ds: f64, phi_lim: f64) -> ScanKw<'_> {
    ScanKw {
        flight: f, tt4_lo: LO, tt4_hi: HI, tt4_max: TT4_MAX, phi_lim, phi_air: PHI_AIR,
        tau_gov: 0.05, tau_q: 0.05, tau_s: 0.05, r, s_settle: S_SETTLE, ds, v_max: V_MAX,
        inc: false,
    }
}

/// `lambda s: s["n_fuel"] > 0` — the suite's detector, literally.
fn fuel_held(s: &ThresholdScan) -> bool {
    s.n_fuel > 0
}

fn march(m: &ScheduledStatorCore, taus: (f64, f64, f64, f64), r: f64) -> (ScheduledStatorCore, Vec<FuelPoint>) {
    let (built, _, _, traj) = split_march(m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR,
                                          "demand", taus, r, S_SETTLE, DS, V_MAX, false);
    (built, traj)
}

// ============================================================================== the fixtures

fn law() -> &'static ThresholdLaw {
    static S: OnceLock<ThresholdLaw> = OnceLock::new();
    // `ds_fine=None` SKIPS the step control, as the suite's fixture does
    S.get_or_init(|| threshold_law(&rig(), &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &RS,
                                   0.05, 0.05, 0.05, 0.05, BRACKET, N_BISECT, S_SETTLE, DS, None,
                                   V_MAX, false))
}

fn reference() -> &'static RefFull {
    static S: OnceLock<ThresholdReference> = OnceLock::new();
    let r = S.get_or_init(|| threshold_reference(&rig(), &flight(), LO, HI, TT4_MAX, PHI_FUEL,
                                                 PHI_AIR, 0.35, &[0.02, 0.03, 0.05, 0.08, 0.12],
                                                 0.05, 0.05, 0.05, BRACKET, N_BISECT, S_SETTLE,
                                                 DS, V_MAX, false));
    match r {
        ThresholdReference::Full(x) => x,
        ThresholdReference::Void { void, .. } => panic!("`ref[\"void\"] is None` failed: {void}"),
    }
}

fn terms() -> &'static ThresholdTerms {
    static S: OnceLock<ThresholdTerms> = OnceLock::new();
    S.get_or_init(|| threshold_terms(&rig(), &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR,
                                     &[0.745, 0.750, 0.755], 0.35, &[0.02, 0.05, 0.20], 0.05, 0.05,
                                     BRACKET, N_BISECT, S_SETTLE, DS, V_MAX, false))
}

fn live(l: &ThresholdLaw) -> Vec<&RowOk> {
    l.rows.iter().filter_map(|x| x.live()).collect()
}

// ============================================================================== the reduce

/// **SUBSTITUTED for Python's `reduce_is_bit_for_bit_rung81`** (header).
#[test]
fn reduce_the_scan_reads_split_marchs_own_march() {
    // Python's own shape, on the matched march at rung 81's ramp
    let (_, traj) = march(&rig(), MATCHED, R81);
    assert_eq!(traj.len(), 341);
    assert!(traj.iter().all(|p| matches!(p.extra, PointExtra::Demand { .. })),
            "DECLARED: the march did not run `demand` -- a clip march also returns 341 points");

    // the substitution: four DIFFERENT clocks, so a swapped pair is visible
    let f = flight();
    let k = ScanKw { tau_gov: 0.02, tau_q: 0.05, tau_s: 0.08, ..kw(&f, R81, DS, PHI_FUEL) };
    let s = threshold_scan(&rig(), 0.12, &k);
    let (built, traj) = march(&rig(), (0.12, 0.02, 0.05, 0.08), R81);
    let b_max = built.fuel.inner.lever.lim.expect("a valve").b_max;
    let ride = riding4_idx(&traj, b_max);
    let auth = |i: usize| match traj[i].extra {
        PointExtra::Demand { authority, .. } => authority,
        _ => unreachable!("every point is a Demand point"),
    };
    let max_tt4 = traj.iter().map(|p| p.tt4).fold(f64::NEG_INFINITY, f64::max);
    let min_phi = traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min);
    assert_eq!((s.tau_f, s.tau_gov, s.r, s.ds, s.phi_lim, s.phi_air),
               (0.12, 0.02, R81, DS, PHI_FUEL, PHI_AIR), "the echoed arguments");
    assert_eq!(s.npts, traj.len(), "npts");
    assert_eq!(s.max_tt4.to_bits(), max_tt4.to_bits(), "max Tt4");
    assert_eq!(s.min_phi.to_bits(), min_phi.to_bits(), "min phi");
    assert_eq!(s.n_riding4, ride.len(), "n_riding4");
    assert!(s.window_open, "an empty window would make the counts below vacuous");
    assert_eq!(s.n_fuel, ride.iter().filter(|&&i| auth(i) == Authority::Fuel).count(), "n_fuel");
    assert_eq!(s.n_gov, ride.iter().filter(|&&i| auth(i) == Authority::Gov).count(), "n_gov");
    let interior: Vec<usize> = ride.iter().copied().filter(|&i| 0 < i && i < traj.len() - 1).collect();
    assert_eq!(s.n_scored + s.n_slope_excluded, interior.len(), "the interior cells");
    let sb = s.s_bind.expect("a binding point");
    assert!(interior.iter().any(|&i| traj[i].s.to_bits() == sb.to_bits()),
            "s_bind {sb} is not an interior riding point of the march at these clocks");
}

/// RUNG 81 § 1's OWN TABLE CELL: 33 four-loop points, ALL `gov`, at matched clocks.
#[test]
fn the_identity_control_reproduces_rung81s_own_cell() {
    let f = flight();
    let s = threshold_scan(&rig(), 0.05, &kw(&f, R81, DS, PHI_FUEL));
    assert_eq!(s.n_riding4, 33, "{}", s.n_riding4);
    assert!(s.n_fuel == 0 && s.n_gov == 33, "{:?}", (s.n_fuel, s.n_gov));
    assert!(s.riding4_valid && s.window_open);
}

/// V1: at `r = 1.0` there is no four-loop point at this `tau_f` — and the plant DID accelerate,
/// so the empty window is not an arrest.
#[test]
fn v1_no_four_loop_window_above_the_admissible_ramp() {
    let f = flight();
    let s = threshold_scan(&rig(), 0.05, &kw(&f, 1.0, DS, PHI_FUEL));
    assert!(s.n_riding4 == 0 && !s.window_open);
    assert!(s.riding4_valid, "an arrested plant would make this V2, not V1");
}

// ============================================================================== § 2: the ramp sweep

/// The threshold DOES move with the schedule's slope — the one half of the seam that survives.
#[test]
fn the_threshold_rises_monotonically_with_the_ramp() {
    let l = law();
    assert_eq!(l.n_void, 0, "{:?}", l.rows);
    assert!(l.monotone_in_r, "{:?}", l.thresholds);
    let (lo, hi) = (l.thresholds[0].1, l.thresholds[l.thresholds.len() - 1].1);
    assert!(hi > 2.5 * lo, "the ramp barely moved the threshold: {:?}", l.thresholds);
}

/// E5: every binding point sits in RELEASE — `kappa = 3` everywhere, none mixed.
#[test]
fn the_effective_clock_is_the_release_one_everywhere() {
    let l = law();
    assert!(l.all_kappa_pure, "{:?}", l.rows.iter().map(|x| x.kappa()).collect::<Vec<_>>());
    assert_eq!(l.kappa_seen, vec![3.0], "{:?}", l.kappa_seen);
}

/// P3's SURVIVING HALF, scored on the ERROR, never on bracket membership.
#[test]
fn the_fixed_point_is_never_beaten_by_the_forward_reading() {
    let l = law();
    let rows = live(l);
    assert_eq!(rows.len(), RS.len());
    assert!(l.p3_fwd_never_better,
            "{:?}", rows.iter().map(|x| (x.r, x.err_fixed, x.err_fwd)).collect::<Vec<_>>());
    assert!(rows.iter().all(|x| x.err_fixed < 0.12),
            "{:?}", rows.iter().map(|x| (x.r, x.err_fixed)).collect::<Vec<_>>());
}

/// P2's SURVIVING HALF — the fixed point is early at every live ramp.
#[test]
fn the_fixed_point_sits_below_the_measured_threshold() {
    let l = law();
    let want: Vec<f64> = live(l).iter().map(|x| x.r).collect();
    assert_eq!(l.p2_fixed_early, want, "{:?}", l.p2_fixed_early);
}

/// V5, RE-SCORED IN ITS OWN CURRENCY: halving `ds` moves the threshold by under 1 %.
#[test]
fn the_step_control_in_its_own_currency() {
    let (m, f) = (rig(), flight());
    let coarse = bisect(&m, fuel_held, 0.004, 0.30, N_BISECT, &kw(&f, 0.25, 0.005, PHI_FUEL));
    let fine = bisect(&m, fuel_held, 0.004, 0.30, N_BISECT, &kw(&f, 0.25, 0.0025, PHI_FUEL));
    assert!(coarse.void().is_none() && fine.void().is_none(), "{:?}", (coarse.void(), fine.void()));
    let (c, fm) = (coarse.ok().mid, fine.ok().mid);
    let rel = (fm - c).abs() / c;
    assert!(rel < 0.01, "threshold moved {:.3}% on a halved step: {c}/{fm}", 100.0 * rel);
}

/// THE READER'S OWN HONESTY: a step control that never ran is not reported as one that held.
#[test]
fn the_unrun_step_control_does_not_read_as_a_passed_one() {
    let l = law();
    let want: Vec<f64> = live(l).iter().map(|x| x.r).collect();
    assert_eq!(l.ds_unrun, want, "{:?}", l.ds_unrun);
    assert!(l.ds_stable.is_empty() && l.ds_unstable.is_empty());
}

// ============================================================================== § 3a: the headline

/// **THE HEADLINE**: at ONE ramp the sign of `forward − τ*` follows the reference's side, at
/// every reference, with BOTH sides populated.
#[test]
fn the_forward_reading_inherits_its_references_sign() {
    let r = reference();
    assert_eq!(r.n_live, r.tau_refs.len(), "{}", r.n_live);
    assert!(r.sign_follows_reference, "{:?}", r.crossing);
    let mut sides: Vec<bool> = r.rows.iter().map(|x| x.ref_above).collect();
    sides.sort_unstable();
    sides.dedup();
    assert_eq!(sides, vec![false, true], "the sweep never crossed the threshold: {:?}", r.crossing);
}

/// THE GUARD: the bracket is SMALLER than the smallest margin it decides — a relation, so a trim
/// of `N_BISECT` fails here.
#[test]
fn the_bisection_resolves_the_signs_it_is_asked_about() {
    let r = reference();
    let margins: Vec<f64> = r.rows.iter().filter_map(|x| x.fwd).map(|f| (f - r.tau_star).abs()).collect();
    assert!(!margins.is_empty());
    let least = margins.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(r.width < least / 2.0,
            "bracket {:.3e} cannot resolve a margin of {least:.3e} -- every sign in this section \
             is undecided at this resolution", r.width);
}

/// THE MECHANISM: the map contracts from ABOVE and diverges from BELOW.
#[test]
fn the_map_contracts_from_above_and_diverges_from_below() {
    let r = reference();
    // `sorted(..., key=lambda x: x["dist"])` -- STABLE
    let side = |up: bool| {
        let mut v: Vec<_> = r.rows.iter().filter(|x| x.ref_above == up).collect();
        v.sort_by(|a, b| a.dist.partial_cmp(&b.dist).expect("dist is not NaN"));
        v
    };
    let (above, below) = (side(true), side(false));
    assert!(above.len() >= 2 && below.len() >= 2);
    assert_eq!(r.grows_above, Some(false), "{:?}", above.iter().map(|x| (x.dist, x.err)).collect::<Vec<_>>());
    assert_eq!(r.grows_below, Some(true), "{:?}", below.iter().map(|x| (x.dist, x.err)).collect::<Vec<_>>());
    let worst = |v: &[&turbojet::threshold_law::RefRow]| {
        v.iter().map(|x| x.err.expect("Python's `max` over a None raises")).fold(f64::NEG_INFINITY, f64::max)
    };
    assert!(worst(&below) > 3.0 * worst(&above), "{} vs {}", worst(&below), worst(&above));
}

// ============================================================================== §§ 4–5: the other two knobs

/// P4: the SIGN holds and the MAGNITUDE does not — `transfer` in a band, and the registered 25 %
/// bar MISSED.
#[test]
fn the_governor_clock_keeps_only_part_of_its_coefficient() {
    let p4 = terms().p4.as_ref().expect("`p4 is not None`");
    assert!(p4.rises, "{p4:?}");
    let t = p4.transfer.expect("a transfer");
    assert!(0.2 < t && t < 0.8, "{t}");
    let e = p4.rel_err.expect("a relative error");
    assert!(e > 0.25, "P4's registered 25 % bar was MISSED at {:.1}%; a gate asserting it passed \
                       would ship the opposite of what was measured", 100.0 * e);
}

/// P5's DIRECTION, REFUTED AND GATED AS REFUTED: raising the wall LOWERS the threshold.
#[test]
fn the_wall_moves_the_threshold_the_other_way() {
    let t = terms();
    let p5 = t.p5.as_ref().expect("`p5 is not None`");
    assert_eq!(t.n_void, 0, "{}", t.n_void);
    assert!(!p5.rises, "{:?}", p5.taus);
    assert!(p5.monotone, "{:?}", p5.taus);
    assert!(p5.gap_falls, "{:?}", p5.gaps);
}

/// **V7 FIRED**: the wall reaches BOTH terms, and moves `ċ_f` most.
#[test]
fn the_terms_do_not_separate_and_the_wall_moves_the_wrong_slope() {
    let p5 = terms().p5.as_ref().expect("`p5 is not None`");
    assert!(!p5.separates && p5.v7_withdrawn, "{p5:?}");
    let (gap, lag, sf) = (p5.d_gap.expect("d_gap"), p5.d_lag.expect("d_lag"),
                          p5.d_slope_f.expect("d_slope_f"));
    assert!(lag > gap / 3.0, "{:?}", (gap, lag));
    assert!(sf > gap, "{:?}", (gap, sf));
}

/// V3, BOTH ENDS: at `0.760` the threshold sits BELOW the bracket, at `0.740` ABOVE it — and
/// neither is void for want of a window.
#[test]
fn v3_censors_the_wall_on_both_sides() {
    let (m, f) = (rig(), flight());
    let hi_wall = bisect(&m, fuel_held, 0.004, 0.30, 4, &kw(&f, 0.35, DS, 0.760));
    let lo_wall = bisect(&m, fuel_held, 0.004, 0.30, 4, &kw(&f, 0.35, DS, 0.740));
    let (Bisect::VoidStraddle { below, at_lo: hi_at_lo, .. },
         Bisect::VoidStraddle { above, at_hi: lo_at_hi, .. }) = (&hi_wall, &lo_wall) else {
        panic!("both walls must void on the straddle: {:?} / {:?}", hi_wall.void(), lo_wall.void());
    };
    assert!(*below, "{:?}", hi_wall.void());
    assert!(*above, "{:?}", lo_wall.void());
    assert!(hi_at_lo.window_open && lo_at_hi.window_open);
}

// ============================================================================== +1, +2: the voids

/// **+1 — V1, `engine.py:22302`**, at `r = 1.0`, `n = 0`: the LOW end has no window and the HIGH
/// end has one, so the void fires off the low end alone.
#[test]
fn void_v1_a_bracket_end_with_no_window() {
    let f = flight();
    let b = bisect(&rig(), fuel_held, 0.004, 0.30, 0, &kw(&f, 1.0, DS, PHI_FUEL));
    let Bisect::VoidEnd { void, lo, hi, at_lo, at_hi } = &b else {
        panic!("expected the bracket-end V1, got {b:?}");
    };
    assert_eq!(void, "V1: four-loop window empty at a bracket end");
    assert_eq!((*lo, *hi), (0.004, 0.30));
    assert!(!at_lo.window_open && at_hi.window_open,
            "the 0.30 end is OPEN at r = 1.0 (step 3's finding) -- measured on PyPy");
}

/// **+2 — V3, `engine.py:22305`**, at `r = 0.35`, `phi_lim = 0.760`, `n = 0`. `%g` of `0.004`
/// and `0.3` — which Rust's `{}` also prints, so this message does NOT pin `py_g` (header).
#[test]
fn void_v3_the_threshold_outside_the_bracket() {
    let f = flight();
    let b = bisect(&rig(), fuel_held, 0.004, 0.30, 0, &kw(&f, 0.35, DS, 0.760));
    let Bisect::VoidStraddle { void, below, above, .. } = &b else {
        panic!("expected the straddle V3, got {b:?}");
    };
    assert_eq!(void, "V3: threshold not strictly inside [0.004, 0.3]");
    assert!(*below && !*above, "the threshold sits BELOW the bracket at 0.760");
}
