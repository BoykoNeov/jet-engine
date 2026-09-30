//! SLICE AJ step 4, rung 84 — `edge_read`, `classify`, `staircase_scan`, `lattice_count`,
//! `staircase_number` and `root_class`, **every returned value bit for bit, in Python's own key
//! order.**
//!
//! # WHAT IS PINNED
//!
//! Twenty-two readings on `tests/test_rung84.py`'s rig, each against
//! `oracle/probe_slice_aj_step4.py`'s output (`oracle/slice_aj_step4_pypy.tsv`, PyPy, the repo
//! venv; rung 83's readings share the file and are gated in `slice_aj_corrector.rs`). The suite's
//! own calls where it has one — rung 83 § 3.2's and § 3.4's read pairs and their `classify`,
//! `test_p4`'s ladder, `test_p3`'s count at two steps, `test_p7`'s coarse `root_class` — and, for
//! branches the suite never takes:
//!
//! * `classify_rev` — the jump pair REVERSED: `left` non-empty, the sign change mirrored.
//! * `edge_v1` / `classify_v1` — the EMPTY window: no summands, every Option `None`, `kind = None`.
//! * `lattice_v3` — Python's misnamed `V3: an edge off the march grid` at `r = 1.0`.
//! * `sn_ok` / `sn_bare` / `sn_zero` — one edge move with a spacing, with none, and with `0.0`
//!   (Python's `if spacing` is TRUTHINESS: no tread, but `spacing = 0.0` reported).
//! * `sn_v5` at `0.0198123456` — `%g` prints `0.0198123`, `{}` would not; `sn_v2` at `r = 1.0`.
//! * `root_cross` — a CROSSING (`root_exists = True`, `d_membership = 0.0`) at the shipped step;
//!   `root_v1` — the bisection's V1 carried; `root_v6` — `eps = 0.123456789` (`%g`: `0.123457`).
//!
//! `summands`, Python's float-keyed dict, is flattened as a LIST of `[key, value]` pairs: the key
//! bits pinned, insertion order kept, and no float spelled into a path.
//!
//! # UNREACHED ON THIS RIG — recorded, not implied by a green run (plan § 5.34.4)
//!
//! `n_slope_excluded > 0` (so `n_scored` = the cell count here, and a port counting cells would
//! pass), a riding point at a trajectory END (so `edge` = the first scored cell here), a merged
//! `round(s, 9)` key within one march, an edge off the grid on an OPEN window, a non-monotone
//! edge, and a summand exactly `0.0`.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::staircase_law::{self, EPS, N_BISECT};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::threshold_law::{ScanKw, BRACKET};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_aj_step4_pypy.tsv");

// ---------------------------------------------------------------------------- the rig
//
// `tests/test_rung84.py`'s module constants (identical to `test_rung83.py`'s).

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
const DS: f64 = 0.005;
/// § 3.2's jump window, `r = 0.25`.
const JUMP_LO: f64 = 0.0197750;
const JUMP_HI: f64 = 0.0197875;
/// § 3.4's crossing window, `r = 0.35`.
const CROSS_LO: f64 = 0.037000;
const CROSS_HI: f64 = 0.037333;

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

/// Python's `_rig`, with its four knob assignments. The class is `StaircaseLawTransient` there;
/// here it is a rung-80 core, because rung 84 adds no cell (plan § 5.34 (ii)).
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

/// `test_rung84.py`'s `_kw(r, ds)`.
fn kw(f: &FlightCondition, r: f64, ds: f64) -> ScanKw<'_> {
    ScanKw {
        flight: f, tt4_lo: LO, tt4_hi: HI, tt4_max: TT4_MAX, phi_lim: PHI_FUEL, phi_air: PHI_AIR,
        tau_gov: 0.05, tau_q: 0.05, tau_s: 0.05, r, s_settle: 1.2, ds, v_max: 0.20,
        inc: false,
    }
}

// ---------------------------------------------------------------------------- the flattening
//
// Every flattener lives in `tests/slice_aj_flat/mod.rs` since step 6: this file's gates are
// what VERIFY them, and `slice_aj_oracle.rs` reuses them, so no struct has two descriptions.

mod slice_aj_flat;
use slice_aj_flat::*;

/// `test_p5`'s spacing formula at `ds = 0.005`: `ds·(hi - lo)/ds_star`.
const SPACING: f64 = 0.005 * (0.024 - 0.016) / 0.005938;

// ---------------------------------------------------------------------------- the comparison

/// The oracle's lines for one reading: its root, its subtree, and its `@rig` readback if any.
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

/// `rig` is the core the reading marched on — `None` for a derived reading, which has no
/// readback in the oracle either.
fn check(name: &str, rig: Option<&ScheduledStatorCore>, mut got: Flat) {
    if let Some(m) = rig {
        let p = format!("{name}@rig");
        got.keys(&p, 2);
        got.s(&format!("{p}.lag_coord"), m.fuel.inner.lag_coord.get());
        got.of(&format!("{p}.sm_air"), m.fuel.inner.sm_air.get());
    }
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

fn one<T>(name: &str, m: Option<&ScheduledStatorCore>, x: &T, w: fn(&mut Flat, &str, &T)) {
    let mut o = Flat::default();
    w(&mut o, name, x);
    check(name, m, o);
}

// ---------------------------------------------------------------------------- the gates

/// Rung 83 § 3.2's jump pair (`r = 0.25`) — `test_p2`'s second half — and its `classify`,
/// forward and REVERSED.
#[test]
fn jump_pair_and_its_classification_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let a = staircase_law::edge_read(&m, JUMP_LO, &kw(&f, 0.25, DS));
    one("edge_jump_lo", Some(&m), &a, edge);
    let b = staircase_law::edge_read(&m, JUMP_HI, &kw(&f, 0.25, DS));
    one("edge_jump_hi", Some(&m), &b, edge);
    one("classify_jump", None, &staircase_law::classify(&a, &b), classified);
    one("classify_rev", None, &staircase_law::classify(&b, &a), classified);
}

/// Rung 83 § 3.4's crossing pair (`r = 0.35`) — `test_p2`'s first half.
#[test]
fn crossing_pair_and_its_classification_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let a = staircase_law::edge_read(&m, CROSS_LO, &kw(&f, 0.35, DS));
    one("edge_cross_lo", Some(&m), &a, edge);
    let b = staircase_law::edge_read(&m, CROSS_HI, &kw(&f, 0.35, DS));
    one("edge_cross_hi", Some(&m), &b, edge);
    one("classify_cross", None, &staircase_law::classify(&a, &b), classified);
}

/// The EMPTY window, and `classify` of it with itself — every Option `None`.
#[test]
fn empty_window_read_and_classification_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let a = staircase_law::edge_read(&m, 0.05, &kw(&f, 1.0, DS));
    one("edge_v1", Some(&m), &a, edge);
    one("classify_v1", None, &staircase_law::classify(&a, &a), classified);
}

/// `test_p4`'s ladder, verbatim.
#[test]
fn p4_ladder_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = staircase_law::staircase_scan(&m, 0.0190, 0.0206, 9, &kw(&f, 0.25, DS));
    one("scan_p4", Some(&m), &s, ladder);
}

/// `test_p3`'s count at the shipped step.
#[test]
fn p3_count_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let c = staircase_law::lattice_count(&m, 0.016, 0.024, &kw(&f, 0.25, 0.005));
    one("lattice_p3", Some(&m), &c, lattice);
}

/// `test_p3`'s count at the halved step.
#[test]
fn p3_count_at_the_halved_step_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let c = staircase_law::lattice_count(&m, 0.016, 0.024, &kw(&f, 0.25, 0.0025));
    one("lattice_fine", Some(&m), &c, lattice);
}

/// Python's misnamed `V3: an edge off the march grid`, at `r = 1.0`.
#[test]
fn count_void_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let c = staircase_law::lattice_count(&m, 0.004, 0.05, &kw(&f, 1.0, DS));
    one("lattice_v3", Some(&m), &c, lattice);
}

/// ONE edge move with a spacing — a `lam`.
#[test]
fn staircase_number_with_a_spacing_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, JUMP_LO, JUMP_HI, Some(SPACING), &kw(&f, 0.25, DS));
    one("sn_ok", Some(&m), &n, number);
}

/// No spacing — the FACTORS and no `lam`.
#[test]
fn staircase_number_bare_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, JUMP_LO, JUMP_HI, None, &kw(&f, 0.25, DS));
    one("sn_bare", Some(&m), &n, number);
}

/// `spacing = 0.0` — Python's truthiness: no tread, but the spacing reported.
#[test]
fn staircase_number_zero_spacing_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, JUMP_LO, JUMP_HI, Some(0.0), &kw(&f, 0.25, DS));
    one("sn_zero", Some(&m), &n, number);
}

/// V5 on a degenerate bracket, its message through `%g`.
#[test]
fn staircase_number_v5_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(
        &m, 0.0198123456, 0.0198123456, None, &kw(&f, 0.25, DS));
    one("sn_v5", Some(&m), &n, number);
}

/// V2 — an end of the bracket in the empty window.
#[test]
fn staircase_number_v2_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, 0.05, 0.30, None, &kw(&f, 1.0, DS));
    one("sn_v2", Some(&m), &n, number);
}

/// `test_p7`'s coarse call — a JUMP, no root.
#[test]
fn root_class_jump_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, N_BISECT, EPS, &kw(&f, 0.25, 0.005));
    one("root_jump", Some(&m), &r, root);
}

/// A CROSSING at the shipped step (`r = 0.35`).
#[test]
fn root_class_crossing_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, N_BISECT, EPS, &kw(&f, 0.35, 0.005));
    one("root_cross", Some(&m), &r, root);
}

/// The bisection's V1, carried.
#[test]
fn root_class_v1_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, N_BISECT, EPS, &kw(&f, 1.0, DS));
    one("root_v1", Some(&m), &r, root);
}

/// V6 with an `eps` whose `%g` (`0.123457`) is not Rust's `{}`.
#[test]
fn root_class_v6_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, 2, 0.123456789, &kw(&f, 0.35, DS));
    one("root_v6", Some(&m), &r, root);
}
