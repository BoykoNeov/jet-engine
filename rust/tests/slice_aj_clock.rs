//! SLICE AJ step 2 — rung 81's five readers (`_central`, `_criterion_at`, `authority_clock`,
//! `_tau_f_inert`, `authority_mask`), **every returned value bit for bit, in Python's own key
//! order.**
//!
//! # WHAT IS PINNED
//!
//! Five readings on `tests/test_rung81.py`'s rig, each against
//! `oracle/probe_slice_aj_step2.py`'s output (`oracle/slice_aj_step2_pypy.tsv`, PyPy, the repo
//! venv): the two module fixtures `clock` and `mask` verbatim, plus three calls for branches the
//! fixtures never take —
//!
//! * `clock_latched` — a `demand-latched` grid: DEMAND-shaped points through `_criterion_at`'s
//!   ELSE branch (it tests the coordinate's NAME). Its predicted and measured labels disagree on
//!   most points, so the branch choice is loud here.
//! * `clock_single` — one `tau_f` in `clip`: `_tau_f_inert`'s `None` column and an EMPTY
//!   `control_clip_shared` (the control never meets `tau_f == 0.05`).
//! * `mask_clip` — the mask in `clip` at stride 2, where it is VACUOUS.
//!
//! Each reading is flattened to `path <TAB> token` lines — a dict writes `keys:N` then its items in
//! order, a list `len:N` then its items, a float its IEEE bits — and compared one for one. A field
//! missing, extra, renamed, or out of Python's order fails at its path, not only a value.
//! After each reading the caller's rig is read back (`lag_coord`, `_sm_air`): `split_march`'s
//! scope must have restored the knob.
//!
//! # UNREACHED ON THIS RIG — recorded, not implied by a green run (plan § 5.34.2)
//!
//! A riding point at a trajectory END (`n_edge > 0`), an ARRESTED row (`riding4_valid = False`,
//! `n_invalid > 0`), a skipped gains point (`switch` / `regime`), a `tie`/`dormant` label in a
//! cell, a quartic with other than ONE zero, `ever_two_authorities`, a row with no interior cell
//! (`agreement = None`), and `all_fuel`.

use turbojet::authority_clock::{
    authority_clock, authority_mask, CLOCK_COORDS, CLOCK_TAU_FS, CLOCK_TAU_GOVS, CLOCK_TAU_Q,
    CLOCK_TAU_S, MASK_CLOCKS, MASK_EVERY,
};
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_aj_step2_pypy.tsv");

// ---------------------------------------------------------------------------- the rig
//
// `tests/test_rung81.py`'s module constants.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const SM: f64 = 0.80 / FLOOR - 1.0;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const TT4_MAX: f64 = 1200.0;
const PHI_FUEL: f64 = 0.75;
const PHI_AIR: Option<f64> = Some(0.77);
// the readers' own defaults for the arguments no call passes
const R: f64 = 0.5;
const SETTLE: f64 = 1.2;
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

/// Python's `_rig`, with its four knob assignments. The class is `AuthorityClockTransient`
/// there; here it is a rung-80 core, because rung 81 adds no cell (plan § 5.34 (ii)).
fn rig() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
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

// ---------------------------------------------------------------------------- the flattening
//
// Every flattener lives in `tests/slice_aj_flat/mod.rs` since step 6: this file's gates are
// what VERIFY them, and `slice_aj_oracle.rs` reuses them, so no struct has two descriptions.

mod slice_aj_flat;
use slice_aj_flat::*;

// ---------------------------------------------------------------------------- the comparison

/// The oracle's lines for one reading: its root, its subtree, and its `@rig` readback.
fn expected(name: &str) -> Vec<(String, String)> {
    let dot = format!("{name}.");
    let at = format!("{name}@");
    let v: Vec<(String, String)> = ORACLE.lines()
        .map(|l| {
            let (p, t) = l.split_once('\t').expect("path<TAB>token");
            (p.to_string(), t.to_string())
        })
        .filter(|(p, _)| p == name || p.starts_with(&dot) || p.starts_with(&at))
        .collect();
    assert!(!v.is_empty(), "the oracle has no reading named {name:?}");
    v
}

fn check(name: &str, got: Flat) {
    let want = expected(name);
    let got = got.0;
    let bad: Vec<usize> = (0..want.len().min(got.len())).filter(|&k| got[k] != want[k]).collect();
    for &k in bad.iter().take(12) {
        eprintln!("{name}: line {k}: got {:?}, want {:?}", got[k], want[k]);
    }
    assert!(bad.is_empty() && got.len() == want.len(),
            "{name}: {} of {} lines differ (lengths got {}, want {})",
            bad.len(), want.len(), got.len(), want.len());
}

// ---------------------------------------------------------------------------- the gates

/// `test_rung81.py`'s `clock` fixture: the whole default grid, both coordinates, and the
/// shared-wall control.
#[test]
fn clock_fixture_bit_for_bit() {
    let m = rig();
    let r = authority_clock(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &CLOCK_TAU_FS,
                            &CLOCK_TAU_GOVS, CLOCK_TAU_Q, CLOCK_TAU_S, &CLOCK_COORDS, R, SETTLE,
                            DS, V_MAX, false);
    let mut o = Flat::default();
    clock(&mut o, "clock", &r);
    rig_after(&mut o, "clock", &m);
    check("clock", o);
}

/// `test_rung81.py`'s `mask` fixture: the mirror cell and rung 80's matched clocks, stride 1.
#[test]
fn mask_fixture_bit_for_bit() {
    let m = rig();
    let r = authority_mask(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &MASK_CLOCKS,
                           "demand", R, SETTLE, DS, V_MAX, false, MASK_EVERY)
        .expect("the mask's gains do not abort on this rig");
    let mut o = Flat::default();
    mask(&mut o, "mask", &r);
    rig_after(&mut o, "mask", &m);
    check("mask", o);
}

/// A `demand-latched` grid: DEMAND-shaped points through `_criterion_at`'s CLIP-form branch.
#[test]
fn latched_grid_takes_the_else_branch_bit_for_bit() {
    let m = rig();
    let r = authority_clock(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.05, 0.20],
                            &[0.05], CLOCK_TAU_Q, CLOCK_TAU_S, &["demand-latched"], R, SETTLE,
                            DS, V_MAX, false);
    let mut o = Flat::default();
    clock(&mut o, "clock_latched", &r);
    rig_after(&mut o, "clock_latched", &m);
    check("clock_latched", o);
}

/// One `tau_f`, `clip` only: the `None` inertness column and the EMPTY control census.
#[test]
fn single_column_is_none_bit_for_bit() {
    let m = rig();
    let r = authority_clock(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &[0.20], &[0.05],
                            CLOCK_TAU_Q, CLOCK_TAU_S, &["clip"], R, SETTLE, DS, V_MAX, false);
    let mut o = Flat::default();
    clock(&mut o, "clock_single", &r);
    rig_after(&mut o, "clock_single", &m);
    check("clock_single", o);
}

/// The mask in `clip`, stride 2 — every cell governor-held, so VACUOUS.
#[test]
fn clip_mask_at_stride_two_bit_for_bit() {
    let m = rig();
    let r = authority_mask(&m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &MASK_CLOCKS,
                           "clip", R, SETTLE, DS, V_MAX, false, 2)
        .expect("the mask's gains do not abort on this rig");
    let mut o = Flat::default();
    mask(&mut o, "mask_clip", &r);
    rig_after(&mut o, "mask_clip", &m);
    check("mask_clip", o);
}
