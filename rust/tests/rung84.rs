//! RUNG 84 — **THE MARCHED MINIMUM'S STAIRCASE**: `docs/rung83-spec.md` § 8's first seam.
//!
//! **HEADLINE: a minimum over a MARCHED set is a reading on a MOVING GRID BOUNDARY, so the
//! residual carries the march's own SAWTOOTH** — rise and tread both scale with `ds`, and a
//! missing root is a crossing landing on a step. Rung 83's `argmin_moved` fired correctly and
//! reported a CONSEQUENCE: verdict CONFIRMED, reason CORRECTED.
//!
//! Ported from `tests/test_rung84.py`: **9 collected tests, all `slow` there** (the port has no
//! tier). **This file has 13**: the suite's 9, 1:1 in order, plus four of the ten driven voids
//! (plan § 5.34 (vi)), DECLARED below.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, **4 ADDED**, 0 COLLAPSED
//!
//! | # | `tests/test_rung84.py` | here |
//! |---|---|---|
//! | 1 | `reduce_the_reader_is_bit_for_bit_rung_82s_scan` | [`reduce_the_reader_is_bit_for_bit_rung_82s_scan`] |
//! | 2 | `p1_the_minimum_is_attained_at_the_windows_leading_point` | [`p1_the_minimum_is_attained_at_the_windows_leading_point`] |
//! | 3 | `p2_the_membership_term_is_exactly_zero_when_the_set_is_unchanged` | [`p2_the_membership_term_is_exactly_zero_when_the_set_is_unchanged`] |
//! | 4 | `p4_an_argmin_move_and_a_set_change_are_the_same_event` | [`p4_an_argmin_move_and_a_set_change_are_the_same_event`] |
//! | 5 | `p3_the_jump_count_times_the_step_is_a_constant_of_the_plant` | [`p3_the_jump_count_times_the_step_is_a_constant_of_the_plant`] |
//! | 6 | `p7_the_same_ramp_has_no_root_at_one_step_and_a_root_at_the_next` | [`p7_the_same_ramp_has_no_root_at_one_step_and_a_root_at_the_next`] |
//! | 7 | `p5_the_rise_and_the_branch_slope_are_what_make_lambda_ds_free` | [`p5_the_rise_and_the_branch_slope_are_what_make_lambda_ds_free`] |
//! | 8 | `p6_refuted_the_missing_root_does_not_return_when_the_step_is_refined` | [`p6_refuted_the_missing_root_does_not_return_when_the_step_is_refined`] |
//! | 9 | `the_threshold_move_between_steps_is_inside_the_sawtooth_bound` | [`the_threshold_move_between_steps_is_inside_the_sawtooth_bound`] |
//! | **+1** | **— none —** | [`void_v3_lattice_count_on_an_empty_window`] (`engine.py:23001`) |
//! | **+2** | **— none —** | [`void_v2_staircase_number_on_an_empty_window`] (`engine.py:23027`) |
//! | **+3** | **— none —** | [`void_v5_staircase_number_with_no_edge_move`] (`engine.py:23030`) |
//! | **+4** | **— none —** | [`void_v6_and_the_carried_v1_in_root_class`] (`engine.py:23061`) |
//!
//! # WHERE THE PORT DIFFERS FROM A LINE-BY-LINE COPY
//!
//! * **#1 is NOT a self-comparison here, so it is ported as written.** [`staircase_law::edge_read`]
//!   computes `h` through `scan_cells` and its OWN arithmetic (`g = h/kappa`, not rung 83's
//!   `F - tau`: plan § 5.34.4 (a)), so `h` agreeing with `threshold_law::scan`'s is a real check.
//!   `round(b["s_bind"], 9)` is [`staircase_law::round9`].
//! * **`round(root * f, 7)` is format-and-parse at 7 places**, the crate's spelling of Python's
//!   `round(x, n)`; all ten arguments are pinned to PyPy's bits before any read is taken.
//! * **`spacing = ds * (hi - lo) / ds_star`** associates left to right, as Python does, and is
//!   pinned to PyPy's bits at both steps.
//! * **`width = (hi - lo) / 2 ** 10`** divides by `1024.0` — Python's `2 ** 10` is the int 1024,
//!   converted exactly. The `r = 0.20` control asserts `moved == 0.0` EXACTLY.
//! * **`x["root_exists"]` on a VOID `root_class` is `None`**, so #8's sequence is compared as
//!   `Option<bool>`s: a void at any step fails it, as it does in Python.
//! * **#6 is STRICTER than Python on one clause, declared:** `coarse["d_membership"] != 0.0` is
//!   `True` on a `None` in Python, while `is_some_and(|d| d != 0.0)` fails on it. On this rig the
//!   jump always carries a membership term, so the two agree here.
//! * **`ia - im >= 1` on Python ints** is `i64` arithmetic on `edge_index`; a `None` index is
//!   Python's `TypeError`, here a panic by name.
//!
//! # THE FOUR ADDED GATES — driven at plan § 5.34 (vi)'s inputs, strings from the source
//!
//! Each string written from `engine.py`'s literal and Python's `%g`, then MEASURED on PyPy
//! (`W:\temp\claude\slice-aj-step5\probe_voids.py`, log opens `3.11.15 … [PyPy 7.3.23 …]`).
//!
//! * **+1, V3** at `r = 1.0`: no window, so no edge exists at all — yet the void says *an edge off
//!   the march grid*, which MISNAMES its cause (plan § 5.34 (vi)); reproduced verbatim, every
//!   Option `None`.
//! * **+2, V2** at `r = 1.0`: checked FIRST, before V5 (step 4's `sn_v2` kills the reverse order).
//! * **+3, V5** at `tau_lo = tau_hi = 0.0198`, `r = 0.25`.
//! * **+4, V6** at `eps = 1.0`, `n_bisect = 0`, `r = 0.25` — and `root_class` CARRIES `_bisect`'s
//!   V1 at `r = 1.0` (the V1 row of plan § 5.34 (vi) names both sites; `rung82.rs` +1 gates the
//!   `_bisect` one). At `r = 1.0` the `0.30` end's `h` is negative, so the straddle holds and
//!   this carry is blind to the order of `_bisect`'s checks exactly as `rung82.rs` +1 is.
//!
//! **What +3/+4 cannot see:** `%g` and Rust's `{}` both print `0.0198` and `1`, so a port
//! formatting these two with `{}` passes here. `slice_aj_staircase.rs`'s `sn_v5`
//! (`0.0198123456`) and `root_v6` (`0.123456789`) are the readings that pin `py_g`.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::staircase_law::{self, Kind, RootClass, RootOk, StaircaseNumber, EPS, N_BISECT};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::threshold_law::{self, ScanKw};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung84.py`'s module constants, verbatim.

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
const PHI_FUEL: f64 = 0.75;
const PHI_AIR: Option<f64> = Some(0.77);
/// Rung 82's shipped march step.
const DS: f64 = 0.005;
/// Rung 82's own bracket.
const BRACKET: (f64, f64) = (0.004, 0.30);

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

/// Python's `_rig(design)`. `StaircaseLawTransient` there; a rung-80 core here (plan § 5.34 (ii)).
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

/// `test_rung84.py`'s `_kw(r, ds=DS)`.
fn kw(f: &FlightCondition, r: f64, ds: f64) -> ScanKw<'_> {
    ScanKw {
        flight: f, tt4_lo: LO, tt4_hi: HI, tt4_max: TT4_MAX, phi_lim: PHI_FUEL, phi_air: PHI_AIR,
        tau_gov: 0.05, tau_q: 0.05, tau_s: 0.05, r, s_settle: 1.2, ds, v_max: 0.20,
        inc: false,
    }
}

/// Python's `round(x, 7)` — format-and-parse, the crate's spelling (`round6`, `round9`, …).
fn round7(x: f64) -> f64 {
    format!("{x:.7}").parse::<f64>().expect("a formatted finite double parses back")
}

/// `m.root_class(bracket=BRACKET, **_kw(r, ds))` — the reader's defaults `n_bisect = 10`,
/// `eps = 1e-7`.
fn root_class(m: &ScheduledStatorCore, r: f64, ds: f64) -> RootClass {
    let f = flight();
    staircase_law::root_class(m, BRACKET, N_BISECT, EPS, &kw(&f, r, ds))
}

fn ok(c: RootClass) -> RootOk {
    match c {
        RootClass::Ok(x) => x,
        RootClass::Void { void, .. } => panic!("`void is None` failed: {void}"),
    }
}

// ============================================================================== the reduce

/// Rung 84's residual must be rung 82's EXACTLY — `h`, the purity, the partition of the scored
/// cells, the riding count and the binding point.
#[test]
fn reduce_the_reader_is_bit_for_bit_rung_82s_scan() {
    let (m, f) = (rig(), flight());
    for r in [0.25, 0.35] {
        for tau in [0.03, 0.08] {
            let a = staircase_law::edge_read(&m, tau, &kw(&f, r, DS));
            let b = threshold_law::scan(&m, tau, &kw(&f, r, DS));
            assert_eq!(a.h.map(f64::to_bits), b.h.map(f64::to_bits),
                       "rung 84 moved the residual at r={r} tau={tau}: {:?} vs {:?}", a.h, b.h);
            assert_eq!(a.kappa_pure, b.kappa_pure);
            assert_eq!(a.kappa, if b.kappa_pure { Some(b.kappa[0]) } else { None });
            assert_eq!(a.n_scored + a.n_slope_excluded, b.n_scored,
                       "the scored set and the slope exclusions no longer partition rung 82's cells");
            assert_eq!(a.n_ride, b.n_riding4);
            let sb = b.s_bind.expect("Python's `round(None, 9)` raises");
            assert_eq!(a.s_bind.map(f64::to_bits), Some(staircase_law::round9(sb).to_bits()),
                       "the binding point moved at r={r} tau={tau}: {:?} vs {sb}", a.s_bind);
        }
    }
}

// ============================================================================== § 1.1: the boundary reading

/// P1: the argmin is ALWAYS the window's first riding point, on both branches of every ramp.
#[test]
fn p1_the_minimum_is_attained_at_the_windows_leading_point() {
    let (m, f) = (rig(), flight());
    let roots = [(0.20, 0.01108), (0.25, 0.01975), (0.35, 0.03710), (0.50, 0.06109), (0.70, 0.09202)];
    // PyPy's `round(root * f, 7)`, by bits -- the arguments are pinned before any read is taken
    const PYPY: [[u64; 2]; 5] = [
        [0x3f7f_c4c1_6590_7d91, 0x3f8f_c4c1_6590_7d91],
        [0x3f8c_5048_16f0_068e, 0x3f9c_5048_16f0_068e],
        [0x3f9a_97e1_32b5_5ef2, 0x3faa_97e1_32b5_5ef2],
        [0x3fa5_e508_2cf5_2b91, 0x3fb5_e508_2cf5_2b91],
        [0x3fb0_7d6f_9767_9032, 0x3fc0_7d6f_9767_9032],
    ];
    let (mut n, mut n_ok) = (0, 0);
    for (k, (r, root)) in roots.into_iter().enumerate() {
        for (j, fac) in [0.7, 1.4].into_iter().enumerate() {
            let tau = round7(root * fac);
            assert_eq!(tau.to_bits(), PYPY[k][j], "round({root}*{fac}, 7) is not PyPy's");
            let rd = staircase_law::edge_read(&m, tau, &kw(&f, r, DS));
            assert!(rd.window_open, "V1: no four-loop window at r={r}");
            assert!(rd.edge_on_grid, "V3: the window edge {:?} is not a multiple of ds={DS} at r={r}",
                    rd.edge);
            n += 1;
            n_ok += rd.at_edge as usize;
        }
    }
    assert_eq!(n_ok, n, "P1: the minimum bound away from the window's leading point at {} of {n} \
                         points", n - n_ok);
}

// ============================================================================== § 2: the classifier

/// P2: the membership term is EXACTLY zero where the set is unchanged, and non-zero and dominant
/// at rung 83 § 3.2's own jump — both directions gated.
#[test]
fn p2_the_membership_term_is_exactly_zero_when_the_set_is_unchanged() {
    let (m, f) = (rig(), flight());
    // § 3.4's CROSSING
    let a = staircase_law::edge_read(&m, 0.037000, &kw(&f, 0.35, DS));
    let b = staircase_law::edge_read(&m, 0.037333, &kw(&f, 0.35, DS));
    let c = staircase_law::classify(&a, &b);
    assert!(!c.set_changed && c.entered.is_empty() && c.left.is_empty());
    assert_eq!(c.d_membership, Some(0.0),
               "P2: the sets are equal yet the membership term is {:?}", c.d_membership);
    assert!(c.kind == Some(Kind::Crossing) && c.sign_change && c.sign_change_common);

    // § 3.2's JUMP
    let a = staircase_law::edge_read(&m, 0.0197750, &kw(&f, 0.25, DS));
    let b = staircase_law::edge_read(&m, 0.0197875, &kw(&f, 0.25, DS));
    let c = staircase_law::classify(&a, &b);
    assert!(c.set_changed && c.left.is_empty() && c.entered.len() == 1,
            "the jump is supposed to be ONE point entering; got entered={:?} left={:?}",
            c.entered, c.left);
    let dm = c.d_membership.expect("a membership term");
    assert_ne!(dm, 0.0);
    let ds_ = c.d_smooth.expect("a smooth term");
    assert!(dm.abs() > 100.0 * ds_.abs(),
            "P2: the membership term ({dm:?}) does not dominate the smooth term ({ds_:?})");
    assert!(c.sign_change && !c.sign_change_common && c.kind == Some(Kind::Jump),
            "P2: rung 83 § 3.2's sign change survived restriction to the common set");
}

/// P4: an argmin move and a set change are ONE event — zero counter-examples on a ladder.
#[test]
fn p4_an_argmin_move_and_a_set_change_are_the_same_event() {
    let (m, f) = (rig(), flight());
    let sc = staircase_law::staircase_scan(&m, 0.0190, 0.0206, 9, &kw(&f, 0.25, DS));
    assert!(sc.all_at_edge, "P1 failed inside P4's own ladder");
    assert_eq!(sc.n_argmin_only, 0, "P4: {} pair(s) moved the argmin without changing the set",
               sc.n_argmin_only);
    assert_eq!(sc.n_set_only, 0);
    let n_arg = sc.pairs.iter().filter(|p| p.argmin_moved).count();
    let n_set = sc.pairs.iter().filter(|p| p.set_changed).count();
    assert!(sc.n_edge_moves == n_arg && n_arg == n_set, "{:?}", (sc.n_edge_moves, n_arg, n_set));
    assert!(sc.exact_zero_when_set_equal && sc.nonzero_when_set_differs);
    assert!(sc.edge_monotone, "V4: the edge is not monotone, so a two-march count is invalid");
    assert!(sc.all_on_grid, "V3: an edge off the march grid");
}

// ============================================================================== § 3: the lattice count

/// P3, REFUTED as a ratio and gated as the mechanism's own invariant: the levels' `n_jumps · ds`
/// bands INTERSECT, and the count rises as the step falls.
#[test]
fn p3_the_jump_count_times_the_step_is_a_constant_of_the_plant() {
    let (m, f) = (rig(), flight());
    let (lo, hi) = (0.016, 0.024);
    let (mut band_lo, mut band_hi) = (0.0_f64, 1.0_f64);
    let mut counts: Vec<i64> = Vec::new();
    let mut stars = Vec::new();
    for ds in [0.005, 0.0025, 0.00125] {
        let c = staircase_law::lattice_count(&m, lo, hi, &kw(&f, 0.25, ds));
        assert!(c.void.is_none() && c.on_grid, "V3: {:?}", c.void);
        assert!(c.at_edge.0 && c.at_edge.1, "P1 failed at a counting endpoint");
        let n = c.n_jumps.expect("a count");
        assert!(n >= 1);
        band_lo = band_lo.max((n - 1) as f64 * ds);
        band_hi = band_hi.min((n + 1) as f64 * ds);
        counts.push(n);
        stars.push(c.ds_star);
    }
    assert!(band_lo < band_hi, "P3: the levels' bands do NOT intersect ({stars:?})");
    assert_eq!(counts, vec![1, 3, 5], "the counts moved: {counts:?}");
    assert!(counts[0] < counts[1] && counts[1] < counts[2]);
}

// ============================================================================== § 4: the map

/// P7: `r = 0.25` has NO root at the shipped step and a root one halving later — and the move is
/// inside the sawtooth bound.
#[test]
fn p7_the_same_ramp_has_no_root_at_one_step_and_a_root_at_the_next() {
    let m = rig();
    let coarse = ok(root_class(&m, 0.25, 0.005));
    assert!(coarse.kind == Some(Kind::Jump) && !coarse.root_exists,
            "P7: r=0.25 at the shipped step classifies as {:?}", coarse.kind);
    assert!(coarse.set_changed && coarse.d_membership.is_some_and(|d| d != 0.0));
    assert!((coarse.mid - 0.019754).abs() < 1e-5,
            "the bisection no longer lands on rung 83 § 3.2's answer: {}", coarse.mid);

    let fine = ok(root_class(&m, 0.25, 0.0025));
    assert!(fine.kind == Some(Kind::Crossing) && fine.root_exists,
            "P7: r=0.25 at ds=0.0025 classifies as {:?}", fine.kind);
    assert_eq!(fine.d_membership, Some(0.0),
               "a crossing must have an EXACTLY zero membership term; got {:?}", fine.d_membership);
    let moved = (fine.mid - coarse.mid).abs();
    assert!(moved < 0.48 * (0.005 + 0.0025),
            "the root moved by {moved:.3e}, outside the sawtooth amplitude the mechanism allows");
}

// ============================================================================== § 5: the staircase number

/// P5 REFUTED, and the REASON gated on the factors: the branch slope is flat across a 2× step and
/// the rise falls with it; without a spacing the reader returns the factors and NO `lam`.
#[test]
fn p5_the_rise_and_the_branch_slope_are_what_make_lambda_ds_free() {
    let (m, f) = (rig(), flight());
    let (lo, hi) = (0.016, 0.024);
    let ds_star = 0.005938; // § 3's five-level intersection midpoint
    let idx = |tau: f64, ds: f64| {
        staircase_law::edge_read(&m, tau, &kw(&f, 0.25, ds)).edge_index
            .expect("Python's `ia - im` on a None index raises")
    };
    // `spacing = ds * (hi - lo) / ds_star`, PyPy's bits at each step
    let pypy_spacing = [(0.0025, 0x3f6b_977f_0227_d5ed_u64), (0.00125, 0x3f5b_977f_0227_d5ed)];
    let mut out: Vec<turbojet::staircase_law::StaircaseOk> = Vec::new();
    for (ds, want) in pypy_spacing {
        let (mut a, mut b) = (lo, hi);
        let mut ia = idx(a, ds);
        for _ in 0..12 {
            let mid = 0.5 * (a + b);
            let im = idx(mid, ds);
            if ia - im >= 1 {
                b = mid;
            } else {
                (a, ia) = (mid, im);
            }
            if b - a < 1e-5 {
                break;
            }
        }
        let spacing = ds * (hi - lo) / ds_star;
        assert_eq!(spacing.to_bits(), want, "spacing at ds={ds} is not PyPy's");
        let sn = match staircase_law::staircase_number(&m, a, b, Some(spacing), &kw(&f, 0.25, ds)) {
            StaircaseNumber::Ok(x) => x,
            other => panic!("`void is None` failed: {other:?}"),
        };
        assert!(sn.lam.is_some() && sn.spacing.is_some());
        out.push(sn);
    }
    let (s0, s1) = (out[0].dg_dtau, out[1].dg_dtau);
    assert!((s1 - s0).abs() / s0 < 0.10,
            "|dg/dtau| moved {:.1}% between steps", 100.0 * (s1 - s0).abs() / s0);
    assert!(out[1].rise < out[0].rise,
            "the rise did not fall when the step halved ({} -> {})", out[0].rise, out[1].rise);
    match staircase_law::staircase_number(&m, out[0].tau_lo, out[0].tau_hi, None,
                                          &kw(&f, 0.25, 0.0025)) {
        StaircaseNumber::Ok(bare) => assert!(bare.lam.is_none() && bare.tread.is_none(),
                                             "a bare call returned a quantized number"),
        other => panic!("`bare[\"rise\"] is not None` failed: {other:?}"),
    }
}

// ============================================================================== § 6: P6 refuted

/// P6 REFUTED, gated AS refuted: existence is absent / present / present over the three coarsest
/// steps.
#[test]
fn p6_refuted_the_missing_root_does_not_return_when_the_step_is_refined() {
    let m = rig();
    let seq: Vec<Option<bool>> = [0.005, 0.0025, 0.00125].into_iter()
        .map(|ds| match root_class(&m, 0.25, ds) {
            RootClass::Ok(x) => Some(x.root_exists),
            RootClass::Void { .. } => None, // Python's `root_exists=None` on a void
        })
        .collect();
    assert_eq!(seq, vec![Some(false), Some(true), Some(true)],
               "the existence sequence moved: {seq:?}");
}

// ============================================================================== § 6.1: rung 82's V5 gets a scale

/// § 6.1: the root's move between the two shipped steps is inside the sawtooth bound, larger than
/// a bisection width at `r = 0.35`, and EXACTLY zero at the `r = 0.20` control.
#[test]
fn the_threshold_move_between_steps_is_inside_the_sawtooth_bound() {
    let m = rig();
    let bound = 0.48 * (0.005 + 0.0025);
    let width = (BRACKET.1 - BRACKET.0) / 1024.0; // `/ 2 ** 10`: the int 1024, exact
    for (r, expect_move) in [(0.35, true), (0.20, false)] {
        let c = ok(root_class(&m, r, 0.005));
        let f = ok(root_class(&m, r, 0.0025));
        assert!(c.kind == Some(Kind::Crossing) && f.kind == Some(Kind::Crossing),
                "{:?} / {:?}", c.kind, f.kind);
        let moved = (f.mid - c.mid).abs();
        assert!(moved < bound, "r={r} moved {moved:.3e} between the shipped steps, outside the \
                                sawtooth bound {bound:.3e}");
        if expect_move {
            assert!(moved > width, "r={r} no longer trips rung 82's V5 (moved {moved:.3e} vs one \
                                    bisection width {width:.3e})");
        } else {
            assert_eq!(moved, 0.0, "r={r}'s control moved by {moved:.3e}");
        }
    }
}

// ============================================================================== +1 … +4: the voids

/// **+1 — V3, `engine.py:23001`**, at `r = 1.0`: the misnamed void of an EMPTY window.
#[test]
fn void_v3_lattice_count_on_an_empty_window() {
    let f = flight();
    let c = staircase_law::lattice_count(&rig(), 0.02, 0.021, &kw(&f, 1.0, DS));
    assert_eq!(c.void.as_deref(), Some("V3: an edge off the march grid"));
    assert!(!c.on_grid && c.at_edge == (false, false));
    assert!(c.edge_lo.is_none() && c.edge_hi.is_none() && c.index_lo.is_none()
            && c.index_hi.is_none() && c.n_jumps.is_none() && c.ds_star.is_none()
            && c.slope_s_star.is_none(), "no window, so no edge and no count");
}

/// **+2 — V2, `engine.py:23027`**, at `r = 1.0`.
#[test]
fn void_v2_staircase_number_on_an_empty_window() {
    let f = flight();
    assert_eq!(staircase_law::staircase_number(&rig(), 0.02, 0.021, None, &kw(&f, 1.0, DS)),
               StaircaseNumber::V2 { void: "V2: kappa impure at an end of the bracket".into(),
                                     tau_lo: 0.02, tau_hi: 0.021 });
}

/// **+3 — V5, `engine.py:23030`**, at `tau_lo = tau_hi = 0.0198`, `r = 0.25`.
#[test]
fn void_v5_staircase_number_with_no_edge_move() {
    let f = flight();
    assert_eq!(staircase_law::staircase_number(&rig(), 0.0198, 0.0198, None, &kw(&f, 0.25, DS)),
               StaircaseNumber::V5 { void: "V5: no edge move in [0.0198, 0.0198]".into(),
                                     tau_lo: 0.0198, tau_hi: 0.0198, edge_moved: false });
}

/// **+4 — V6, `engine.py:23061`**, at `eps = 1.0`, `n_bisect = 0`, `r = 0.25`; and the V1
/// `root_class` CARRIES from `_bisect` at `r = 1.0`.
#[test]
fn void_v6_and_the_carried_v1_in_root_class() {
    let (m, f) = (rig(), flight());
    assert_eq!(staircase_law::root_class(&m, BRACKET, 0, 1.0, &kw(&f, 0.25, DS)),
               RootClass::Void { void: "V6: bracket narrower than 1".into(), r: 0.25, ds: DS });
    assert_eq!(staircase_law::root_class(&m, BRACKET, 0, EPS, &kw(&f, 1.0, DS)),
               RootClass::Void { void: "V1: four-loop window empty at a bracket end".into(),
                                 r: 1.0, ds: DS });
    // THE ORDER: at `r = 1.0` with `eps = 1.0` the bisection voids AND its span (0.296) is under
    // `eps`, so both voids' conditions hold, and the CARRY comes first (measured on PyPy). The two
    // arms above each trip ONE, so neither sees a port that tested V6 first (injection I15).
    assert_eq!(staircase_law::root_class(&m, BRACKET, 0, 1.0, &kw(&f, 1.0, DS)),
               RootClass::Void { void: "V1: four-loop window empty at a bracket end".into(),
                                 r: 1.0, ds: DS });
}
