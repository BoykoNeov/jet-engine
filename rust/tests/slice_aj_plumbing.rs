//! SLICE AJ STEP 1 — **the two helpers rungs 81–84 need and the crate did not have**, gated
//! before any reader uses them (plan § 5.34 (v), § 5.34.1).
//!
//! 1. [`py_g`] — Python's `'%g' % x`. Rungs 82–84 build four returned messages with it
//!    (`engine.py:22305`, `:22784`, `:23030`, `:23061`). Gated on EVERY row of
//!    `rust/oracle/dump_py_g.py`'s output under both interpreters (10 511 values, PyPy and CPython
//!    byte-identical), and on P3's driven arguments written out, so the expected strings a reader
//!    will compare against are readable here and not only in a TSV.
//! 2. [`riding4_idx`] — rung 72's `_riding4` as TRAJECTORY INDICES, the port of rungs 81/82's
//!    `id(p)` round trip. **Not gated against [`riding4`]**: `riding4` is now built from it, so that
//!    comparison would be the function compared with itself. It is gated against Python instead —
//!    the indices `authority_clock`'s loop keeps (`engine.py:21940–21946`), on `test_rung81.py`'s
//!    rig, at three marches, measured on PyPy by `W:\temp\claude\slice-aj-step1\idx_probe.py`:
//!
//!    ```text
//!    ride = m._riding4(traj, mm.bleed_lim.b_max); seen = {id(p) for p in ride}
//!    idx  = [i for i, p in enumerate(traj) if id(p) in seen]
//!    ```
//!
//!    The probe also asserted that no trajectory repeats a point object (`len({id(p)}) ==
//!    len(traj)`), so on these marches the index set is the object set with no argument needed.
//!    The first march is rung 81's control diagonal, whose 33 four-loop points are rung 80 § 2's
//!    number and `test_rung81.py`'s control gate.
//!
//! The refactor of `riding4` itself is guarded by every existing `riding4` caller's gate in the
//! crate — they all see the same `Vec` they saw before.
//!
//! [`py_g`]: turbojet::demand_coordinate::py_g
//! [`riding4_idx`]: turbojet::shared_actuator::riding4_idx
//! [`riding4`]: turbojet::shared_actuator::riding4

use turbojet::bleed_transient::LeverArm;
use turbojet::demand_coordinate::py_g;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::riding4_idx;
use turbojet::split_wall::{build_split_wall_cascade, split_march};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== 1. `%g`

const ORACLE_PYPY: &str = include_str!("../oracle/py_g_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/py_g_cpython.tsv");

fn check_oracle(text: &str, who: &str) {
    let mut n = 0usize;
    let mut bad: Vec<String> = Vec::new();
    for line in text.lines() {
        let (bits, want) = line.split_once('\t').expect("a row is `bits<TAB>string`");
        let x = f64::from_bits(bits.parse::<u64>().expect("bits are a u64"));
        let got = py_g(x);
        if got != want {
            bad.push(format!("{bits} ({x:e}): Python {want:?}, py_g {got:?}"));
        }
        n += 1;
    }
    assert_eq!(n, 10_511, "{who}: the oracle lost or gained rows");
    assert!(bad.is_empty(), "{who}: {} of {n} differ; first: {:?}", bad.len(), &bad[..bad.len().min(5)]);
}

#[test]
fn py_g_matches_the_pypy_oracle() {
    check_oracle(ORACLE_PYPY, "PyPy");
}

#[test]
fn py_g_matches_the_cpython_oracle() {
    check_oracle(ORACLE_CPYTHON, "CPython");
}

/// P3's arguments, and the four messages the pre-flight DROVE in Python (plan § 5.34 (vi)), with
/// `root_class`'s shipped default `eps = 1e-7` — the two-digit exponent Rust's `{:e}` lacks.
#[test]
fn py_g_pins_the_driven_arguments() {
    for (x, want) in [(0.004, "0.004"), (0.3, "0.3"), (1e-12, "1e-12"), (0.0198, "0.0198"),
                      (1.0, "1"), (1e-7, "1e-07")] {
        assert_eq!(py_g(x), want, "py_g({x:e})");
    }
    assert_eq!(format!("V3: threshold not strictly inside [{}, {}]", py_g(0.004), py_g(0.3)),
               "V3: threshold not strictly inside [0.004, 0.3]");
    assert_eq!(format!("S2: flat pair, |dg| < {}", py_g(1e-12)), "S2: flat pair, |dg| < 1e-12");
    assert_eq!(format!("V5: no edge move in [{}, {}]", py_g(0.0198), py_g(0.0198)),
               "V5: no edge move in [0.0198, 0.0198]");
    assert_eq!(format!("V6: bracket narrower than {}", py_g(1.0)), "V6: bracket narrower than 1");
    assert_eq!(format!("V6: bracket narrower than {}", py_g(1e-7)),
               "V6: bracket narrower than 1e-07");
}

// ============================================================================== 2. `riding4_idx`
//
// `tests/test_rung81.py`'s module constants and `_rig`, verbatim — rung 80's rig, which is what
// a rung-81 machine IS (plan § 5.34 (ii)).

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
const PHI_AIR: f64 = 0.77;
const MATCHED: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const SLOW_FUEL: (f64, f64, f64, f64) = (0.20, 0.05, 0.05, 0.05);

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

fn rig() -> ScheduledStatorCore {
    let sm = 0.80 / FLOOR - 1.0;
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    };
    let m = match build_split_wall_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm) {
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

/// `test_rung81.py`'s `_march(m, taus, coord)` → `riding4_idx(traj, m.bleed_lim.b_max)`.
fn ride_idx(taus: (f64, f64, f64, f64), coord: &'static str) -> (usize, Vec<usize>) {
    let (m, _, _, traj) = split_march(&rig(), &flight(), LO, HI, TT4_MAX, PHI_FUEL, Some(PHI_AIR),
                                      coord, taus, 0.5, 1.2, 0.005, V_MAX, false);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    (traj.len(), riding4_idx(&traj, b_max))
}

#[test]
fn riding4_idx_keeps_the_indices_python_keeps() {
    // (march, coord, Python's kept indices — each a contiguous run, measured, not assumed)
    for (taus, coord, want) in [
        (MATCHED, "demand", (38..=70).collect::<Vec<usize>>()),
        (SLOW_FUEL, "demand", (31..=77).collect()),
        (SLOW_FUEL, "clip", (52..=74).collect()),
    ] {
        let (n, got) = ride_idx(taus, coord);
        assert_eq!(n, 341, "{coord} {taus:?}: the march is not rung 81's length");
        assert_eq!(got, want, "{coord} {taus:?}: riding4_idx disagrees with Python's id(p) set");
    }
}
