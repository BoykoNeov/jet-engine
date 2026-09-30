//! RUNG 83 — **THE CORRECTOR'S OWN BAR**: `docs/rung82-spec.md` § 8's first seam, answered NO.
//!
//! **HEADLINE: a bracketing solve locates a SIGN CHANGE; a corrector needs a ROOT, and on a
//! residual built as a MINIMUM those are different objects.** At `r = 0.25` there is no root —
//! `g` steps across zero at an argmin handover. And the SIDE is free: `sign(h)` off ONE march
//! says which side of the root a reference sits on, correcting rung 82 § 6.
//!
//! Ported from `tests/test_rung83.py`: **8 collected tests, all `slow` there** (the port has no
//! tier). **This file has 12**: the suite's 8, 1:1 in order, plus four of the ten driven voids
//! (plan § 5.34 (vi)), DECLARED below.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, **4 ADDED**, 0 COLLAPSED
//!
//! | # | `tests/test_rung83.py` | here |
//! |---|---|---|
//! | 1 | `reduce_the_march_is_bit_for_bit_rung_82s` | [`reduce_the_march_is_bit_for_bit_rung_82s`] |
//! | 2 | `the_fixed_point_is_the_root_of_the_forward_readings_own_residual` | [`the_fixed_point_is_the_root_of_the_forward_readings_own_residual`] |
//! | 3 | `the_side_of_the_root_is_free_from_one_march` | [`the_side_of_the_root_is_free_from_one_march`] |
//! | 4 | `the_r025_sign_change_is_a_JUMP_and_no_root_exists_there` | [`the_r025_sign_change_is_a_jump_and_no_root_exists_there`] |
//! | 5 | `the_r035_sign_change_IS_a_crossing_and_the_contrast_is_the_finding` | [`the_r035_sign_change_is_a_crossing_and_the_contrast_is_the_finding`] |
//! | 6 | `the_secant_reaches_machine_precision_where_a_root_is_on_a_smooth_branch` | [`the_secant_reaches_machine_precision_where_a_root_is_on_a_smooth_branch`] |
//! | 7 | `the_secant_CANNOT_converge_at_r025_even_started_at_the_bisections_own_answer` | [`the_secant_cannot_converge_at_r025_even_started_at_the_bisections_own_answer`] |
//! | 8 | `a_degenerate_slope_voids_the_step_rather_than_clipping_it` | [`a_degenerate_slope_voids_the_step_rather_than_clipping_it`] |
//! | **+1** | **— none —** | [`void_v4_the_step_on_an_empty_window`] (`engine.py:22716`) |
//! | **+2** | **— none —** | [`void_v4_a_start_point_with_no_residual_stops_the_secant`] (`engine.py:22776`) |
//! | **+3** | **— none —** | [`void_v5_the_step_at_c_equal_one`] (`engine.py:22717`) |
//! | **+4** | **— none —** | [`void_s2_a_flat_pair_aborts`] (`engine.py:22784`) |
//!
//! # WHERE THE PORT DIFFERS FROM A LINE-BY-LINE COPY
//!
//! * **#1 is NOT a self-comparison here, so it is ported as written.** [`corrector_read`] is its
//!   own function that calls `threshold_law::scan` and carries the result; comparing its `scan`
//!   with a direct `scan` at the same `kw` checks that the corrector passes its arguments through
//!   unchanged. Python's `set(a) == set(b)` plus `a[k] == b[k]` for every key is one `Debug`
//!   equality of the two structs (shortest round-trip `f64`s, so equal strings are equal bits).
//! * **`T_START = (lo*hi) ** 0.5` is `powf(0.5)`**, not `sqrt` — the crate's power spelling —
//!   and both starts are pinned to PyPy's bits (`0x1.1bc779ef0d4e8p-5`, `0x1.62b9586ad0a22p-5`)
//!   so a changed spelling fails by name before the secant it seeds is scored.
//! * **`rd["tau_hat"] is None` on the step's VOID shape** is the [`CorrectorStep::Void`] variant;
//!   the ok shape's `void is None` is the `Ok` variant.
//!
//! # THE FOUR ADDED GATES — driven at plan § 5.34 (vi)'s inputs, strings from the source
//!
//! Each string written from `engine.py`'s literal, then MEASURED on PyPy
//! (`W:\temp\claude\slice-aj-step5\probe_voids.py`, log opens `3.11.15 … [PyPy 7.3.23 …]`).
//!
//! * **+1, V4** at `r = 1.0`, `tau = 0.05`, `c = 0.5`: no four-loop window, so no `F` — the void
//!   says *kappa impure*, which MISNAMES an EMPTY window (plan § 5.34 (vi)); reproduced verbatim.
//! * **+2, V4 at a start point**, twice. `p_drive.py`'s `(0.05, 0.06, cap = 1)` at `r = 1.0` fails
//!   at BOTH starts. **One arm is added beyond the pre-flight:** `(0.05, 0.30)`, whose SECOND start
//!   is valid (step 3: the `0.30` window is open at `r = 1.0`; PyPy `g = −0.14885…`). Python
//!   `break`s at the first failure, so the trace stays EMPTY — a port that `continue`d would march
//!   the second start. **This discharges step 4's coverage gap C13** (*"a start failure at the
//!   FIRST start"*, plan § 5.34.4 (b)'s UNREACHED list).
//! * **+3, V5** at `r = 0.25`, `tau = 0.0198`, `c = 1.0`: `forward` is carried (PyPy's bits),
//!   `tau_hat`/`correction` are not.
//! * **+4, S2** at `(0.0198, 0.0198, cap = 2)`, `r = 0.25`: two identical starts, two marches, a
//!   flat pair. **The one void here whose `%g` differs from Rust's `{}`** (`1e-12` vs `0.000000000001`).

use turbojet::bleed_transient::LeverArm;
use turbojet::corrector_law::{
    corrector_read, corrector_secant, corrector_step, residual_shape, CorrectorStep, FLAT,
};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::threshold_law::{self, ScanKw};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung83.py`'s module constants, verbatim.

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
/// Rung 81's own ramp — the identity control.
const R81: f64 = 0.5;
/// § 3.2's jump window, `r = 0.25` — the spec's measured one, two marches.
const JUMP_LO: f64 = 0.0197750;
const JUMP_HI: f64 = 0.0197875;
/// § 3.4's crossing window, `r = 0.35`.
const CROSS_LO: f64 = 0.037000;
const CROSS_HI: f64 = 0.037333;
/// § 4's start rule's bracket — rung 82's own.
const BRACKET: (f64, f64) = (0.004, 0.30);

/// `T_START = (BRACKET[0] * BRACKET[1]) ** 0.5` — `pow`, the crate's spelling above a square.
fn t_start() -> f64 {
    (BRACKET.0 * BRACKET.1).powf(0.5)
}

/// `T_START1 = 1.25 * T_START`.
fn t_start1() -> f64 {
    1.25 * t_start()
}

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

/// Python's `_rig(design)`. `CorrectorLawTransient` there; a rung-80 core here (plan § 5.34 (ii)).
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

/// `test_rung83.py`'s `_kw(r)`.
fn kw(f: &FlightCondition, r: f64) -> ScanKw<'_> {
    ScanKw {
        flight: f, tt4_lo: LO, tt4_hi: HI, tt4_max: TT4_MAX, phi_lim: PHI_FUEL, phi_air: PHI_AIR,
        tau_gov: 0.05, tau_q: 0.05, tau_s: 0.05, r, s_settle: 1.2, ds: 0.005, v_max: 0.20,
        inc: false,
    }
}

// ============================================================================== the reduce

/// Rung 83 adds no state, no knob and no constant, so its march must be rung 82's EXACTLY — the
/// same struct, field for field, bit for bit.
#[test]
fn reduce_the_march_is_bit_for_bit_rung_82s() {
    let (m, f) = (rig(), flight());
    for tau in [0.03, 0.08] {
        let a = corrector_read(&m, tau, &kw(&f, R81)).scan;
        let b = threshold_law::scan(&m, tau, &kw(&f, R81));
        assert_eq!(format!("{a:?}"), format!("{b:?}"), "rung 83 moved the scan at tau={tau}");
    }
}

// ============================================================================== § 1.1: the identity

/// `h == tau_hat_min - kappa*tau` at ZERO tolerance; and `g`, `h` differ by `kappa` to within the
/// 4 ULP one divide and one multiply can explain.
#[test]
fn the_fixed_point_is_the_root_of_the_forward_readings_own_residual() {
    let (m, f) = (rig(), flight());
    for r in [0.25, R81] {
        for tau in [0.03, 0.08] {
            let rd = corrector_read(&m, tau, &kw(&f, r));
            assert!(rd.kappa_pure, "kappa impure at r={r} tau={tau} -- V4, not a result");
            assert!(rd.exact, "the identity is NOT exact at r={r} tau={tau}: h={:?} vs \
                               tau_hat_min-kappa*tau={:?}", rd.h, rd.identity_pred);
            let (h, k, g) = (rd.h.expect("h"), rd.kappa.expect("kappa"), rd.g.expect("g"));
            let err = (h - k * g).abs();
            assert!(err <= 4.0 * 2.221e-16 * h.abs(),
                    "the kappa round trip cost {err:.3e} at r={r} tau={tau} -- more than 4 ULP of h");
        }
    }
}

// ============================================================================== § 2: the side is free

/// `sign(h)` off ONE march puts `tau` on the right side of each ramp's sign flip — scored on BOTH
/// branches.
#[test]
fn the_side_of_the_root_is_free_from_one_march() {
    let (m, f) = (rig(), flight());
    let (mut seen_above, mut seen_below) = (0, 0);
    for (r, flip, taus) in [(0.25, 0.019754, [0.008, 0.014, 0.030, 0.050]),
                            (0.35, 0.037098, [0.020, 0.030, 0.080, 0.120])] {
        for tau in taus {
            let rd = corrector_read(&m, tau, &kw(&f, r));
            let below = rd.below_root.unwrap_or_else(|| panic!("no residual at r={r} tau={tau}"));
            assert_eq!(below, tau < flip, "sign(h) put tau={tau} on the wrong side of r={r}'s sign \
                                           flip {flip} (h={:?})", rd.h);
            seen_above += (tau > flip) as usize;
            seen_below += (tau < flip) as usize;
        }
    }
    assert!(seen_above >= 3 && seen_below >= 3,
            "one branch went untested ({seen_above} above, {seen_below} below)");
}

// ============================================================================== § 3: a sign change is not a root

/// § 3.2 — THE RUNG: at `r = 0.25` `g` steps across zero AT an argmin handover.
#[test]
fn the_r025_sign_change_is_a_jump_and_no_root_exists_there() {
    let (m, f) = (rig(), flight());
    let sh = residual_shape(&m, JUMP_LO, JUMP_HI, 2, &kw(&f, 0.25));
    assert_eq!(sh.n_changes, 1, "expected exactly one sign change, got {}", sh.n_changes);
    let ch = &sh.changes[0];
    assert!(ch.argmin_moved, "the sign change is NOT at an argmin handover (s {:?} -> {:?})",
            ch.s_lo, ch.s_hi);
    assert!(ch.ratio > 50.0, "min|g|/step = {:.1} -- a root may exist here after all", ch.ratio);
    assert!(ch.smallest_g > 1e-4, "smallest |g| = {:.3e}", ch.smallest_g);
}

/// § 3.4 — the control: at `r = 0.35` a SMOOTH crossing, and the contrast asserted as a relation.
#[test]
fn the_r035_sign_change_is_a_crossing_and_the_contrast_is_the_finding() {
    let (m, f) = (rig(), flight());
    let sh = residual_shape(&m, CROSS_LO, CROSS_HI, 2, &kw(&f, 0.35));
    assert_eq!(sh.n_changes, 1);
    let ch = &sh.changes[0];
    assert!(!ch.argmin_moved, "the r=0.35 crossing moved its argmin too (s {:?} -> {:?})",
            ch.s_lo, ch.s_hi);
    assert!(ch.ratio < 2.0, "min|g|/step = {:.2} at r=0.35 -- not a crossing either", ch.ratio);
    let jump = residual_shape(&m, JUMP_LO, JUMP_HI, 2, &kw(&f, 0.25));
    let jr = jump.changes[0].ratio;
    assert!(jr > 25.0 * ch.ratio, "the jump's ratio ({jr:.1}) is not decisively above the \
                                   crossing's ({:.2})", ch.ratio);
}

// ============================================================================== § 4: the iteration

/// § 4 — at `r = 0.35`, from the start rule fixed in advance, the secant reaches ~1e-15 in
/// ≤ 8 marches, unclamped.
#[test]
fn the_secant_reaches_machine_precision_where_a_root_is_on_a_smooth_branch() {
    assert_eq!(t_start().to_bits(), 0x3fa1_bc77_9ef0_d4e8, "T_START is not PyPy's float");
    assert_eq!(t_start1().to_bits(), 0x3fa6_2b95_86ad_0a22, "T_START1 is not PyPy's float");
    let (m, f) = (rig(), flight());
    let out = corrector_secant(&m, t_start(), t_start1(), 6, BRACKET, FLAT, &kw(&f, 0.35));
    assert_eq!(out.abort, None);
    assert_eq!(out.clamps, 0, "clamped {} times -- a clamped secant is a bisection in disguise",
               out.clamps);
    assert!(out.marches <= 8, "cost {} marches", out.marches);
    let g = out.final_g.expect("a final residual");
    assert!(g < 1e-12, "residual only reached {g:.3e}");
    assert!(out.converged);
}

/// § 3.2 + § 4's control — started AT rung 82's own 13-march answer the secant still fails, and it
/// OSCILLATES.
#[test]
fn the_secant_cannot_converge_at_r025_even_started_at_the_bisections_own_answer() {
    let (m, f) = (rig(), flight());
    let root = 0.019753906249999998; // rung 82's 13-march bisected mid at r = 0.25
    let out = corrector_secant(&m, root, 1.25 * root, 6, BRACKET, FLAT, &kw(&f, 0.25));
    assert_eq!(out.abort, None);
    assert!(!out.converged);
    let g = out.final_g.expect("a final residual");
    assert!(g > 1e-4, "the residual reached {g:.3e} -- if the secant converges here there IS a root");
    let pos = out.trace.iter().any(|x| x.g > 0.0);
    let neg = out.trace.iter().any(|x| !(x.g > 0.0));
    assert!(pos && neg, "the iteration stayed on one side -- not the oscillation § 4 saw");
}

// ============================================================================== the void guards

/// V5: `1/(1-c)` at `c -> 1` returns no number with a named void, never a clipped one.
#[test]
fn a_degenerate_slope_voids_the_step_rather_than_clipping_it() {
    let (m, f) = (rig(), flight());
    let rd = corrector_read(&m, 0.08, &kw(&f, R81));
    let CorrectorStep::Void { void, .. } = corrector_step(&rd, 1.0) else {
        panic!("`tau_hat is None` failed at c = 1");
    };
    assert!(void.contains("V5"), "{void}");
    let CorrectorStep::Ok { tau_hat, forward, correction, .. } = corrector_step(&rd, 0.044) else {
        panic!("the step at c = 0.044 voided");
    };
    assert!(((forward + correction) - tau_hat).abs() < 1e-15);
}

// ============================================================================== +1 … +4: the voids

/// **+1 — V4, `engine.py:22716`**, at `r = 1.0`: the misnamed void of an EMPTY window.
#[test]
fn void_v4_the_step_on_an_empty_window() {
    let (m, f) = (rig(), flight());
    let rd = corrector_read(&m, 0.05, &kw(&f, 1.0));
    assert!(!rd.window_open && rd.f.is_none(), "the premise: no window, no forward reading");
    assert_eq!(corrector_step(&rd, 0.5),
               CorrectorStep::Void { c: 0.5, forward: None, void: "V4: kappa impure".into() });
}

/// **+2 — V4 at a start point, `engine.py:22776`**: both starts empty; then the FIRST empty and
/// the second valid — Python `break`s, so nothing is marched (C13, header).
#[test]
fn void_v4_a_start_point_with_no_residual_stops_the_secant() {
    let (m, f) = (rig(), flight());
    let out = corrector_secant(&m, 0.05, 0.06, 1, BRACKET, FLAT, &kw(&f, 1.0));
    assert_eq!(out.abort.as_deref(), Some("V4: kappa impure at a start point"));
    assert!(out.trace.is_empty() && out.marches == 0 && out.tau.is_none() && out.final_g.is_none());
    assert!(!out.converged);

    let second = corrector_read(&m, 0.30, &kw(&f, 1.0));
    assert!(second.window_open && second.g.is_some(),
            "the premise: the 0.30 start IS valid at r = 1.0 (PyPy g = -0.14885...)");
    let out = corrector_secant(&m, 0.05, 0.30, 1, BRACKET, FLAT, &kw(&f, 1.0));
    assert_eq!(out.abort.as_deref(), Some("V4: kappa impure at a start point"));
    assert!(out.trace.is_empty() && out.marches == 0,
            "the secant marched the second start after the first failed -- Python `break`s there");
}

/// **+3 — V5, `engine.py:22717`**, at `r = 0.25`, `tau = 0.0198`, `c = 1.0`.
#[test]
fn void_v5_the_step_at_c_equal_one() {
    let (m, f) = (rig(), flight());
    let rd = corrector_read(&m, 0.0198, &kw(&f, 0.25));
    let fwd = rd.f.expect("a forward reading");
    assert_eq!(fwd.to_bits(), 0.017356640246581255_f64.to_bits(), "F is not PyPy's");
    assert_eq!(corrector_step(&rd, 1.0),
               CorrectorStep::Void { c: 1.0, forward: Some(fwd), void: "V5: |1-c| below 1e-12".into() });
}

/// **+4 — S2, `engine.py:22784`**: identical starts, two marches, a flat pair. `%g` of the
/// default `flat = 1e-12` — the one void message here Rust's `{}` would print differently.
#[test]
fn void_s2_a_flat_pair_aborts() {
    let (m, f) = (rig(), flight());
    let out = corrector_secant(&m, 0.0198, 0.0198, 2, BRACKET, FLAT, &kw(&f, 0.25));
    assert_eq!(out.abort.as_deref(), Some("S2: flat pair, |dg| < 1e-12"));
    assert_eq!((out.marches, out.clamps), (2, 0));
    assert_eq!(out.tau, Some(0.0198));
    assert!(!out.converged);
    assert_eq!(out.trace[0].g.to_bits(), out.trace[1].g.to_bits(), "a flat pair is ONE float twice");
}
