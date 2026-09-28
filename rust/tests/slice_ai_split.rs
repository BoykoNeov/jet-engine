//! SLICE AI step 4 — rung 80 whole: `_with_air`, `_split_march`, `_split_row` and the four
//! readers, **AND ON EVERY RUNG-80 READER RUNG 79's SIX COUNTERS STAY AT ZERO, SO THE FIELD
//! READBACK IS THE ONLY INSTRUMENT THAT SEES A COORDINATE DEFECT.**
//!
//! # WHAT PYTHON MEASURED BEFORE THE PORT WAS WRITTEN
//!
//! On `tests/test_rung80.py`'s rig, every reader at the settings its fixtures (and the
//! fingerprint's `kernel_r80`) use — and `split_saturation`, which no Python test calls, at its
//! eight default walls, the ones `docs/rung80-spec.md` § 4 reports. Rung 79's six class counters
//! move by `[0, 0, 0, 0, 0, 0]` around all five readings: rung 80 marches in `phi` reference
//! throughout, so rung 79's incidence branch is never entered. Every march's built machine carries
//! `(lag_coord, phi_ref) = (coord, "phi")`, and the caller's `_sm_air` is back at `None` after
//! each reader.
//!
//! # WHAT IS PINNED
//!
//! Every value of every reading, bit for bit, as word vectors (float → IEEE bits, `None` →
//! [`NONE`], bool → `0`/`1`, a label → [`code`]): the 38 table rows field by field, the 21
//! interior gain cells, and each reader's aggregates. Plus the readback above, on a direct
//! `split_march`, and `AirScope`'s restore on drop and on unwind.
//!
//! # THE EXPECTED VALUES ARE PYTHON's
//!
//! Printed by `W:\temp\claude\slice-ai-step4\probe_py.py` (PyPy, the repo venv) and spliced in by
//! `gen.py` beside it. No golden file is read.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::Authority;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::{
    build_split_wall_cascade, split_arrest, split_liveness, split_march, split_saturation,
    AirScope, SplitArrest, SplitCell, SplitGains, SplitLiveness, SplitRow, SplitSaturation,
    ARREST_WALLS, GAINS_EVERY, LIVENESS_COORDS, LIVENESS_PHI_AIRS, SATURATION_PHI_AIRS,
};
use turbojet::state_coordinate::{coord_counters, reset_coord_counters, CoordCounters};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung80.py`'s module constants, and the reader defaults it takes by omission.

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
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const TT4_MAX: f64 = 1200.0;
const PHI_FUEL: f64 = 0.75;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
/// `test_rung80.py`'s two gain fixtures' walls, and the fingerprint's.
const GAIN_AIRS: [Option<f64>; 3] = [None, Some(0.77), Some(0.80)];

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

/// Python's `_rig`, with its four knob assignments. A FRESH rig per reader, as the probe and the
/// fingerprint build one.
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

// ---------------------------------------------------------------------------- the encoding

/// `None`'s word — a signalling-NaN payload no reader produces.
const NONE: u64 = 0x7ff4_dead_beef_0001;

fn code(s: &str) -> u64 {
    match s {
        "clip" => 1,
        "demand" => 2,
        "fuel" => 3,
        "gov" => 4,
        "tie" => 5,
        "dormant" => 6,
        "shared" => 7,
        "air" => 8,
        _ => panic!("no code for {s:?}"),
    }
}

fn of(x: Option<f64>) -> u64 {
    x.map_or(NONE, f64::to_bits)
}

fn ol(x: Option<Authority>) -> u64 {
    x.map_or(NONE, |a| code(a.as_str()))
}

const ROW_FIELDS: [&str; 22] = [
    "coord", "phi_lim", "phi_air", "npts", "phi_lim_built", "phi_air_built", "phi0", "b0", "v0",
    "b0_frac", "min_phi", "max_tt4", "arrested", "riding4_valid", "n_cut_fuel", "n_cut_gov",
    "max_req_fuel", "max_req_gov", "valve_moved", "stator_moved", "n_riding4", "b_max_hit",
];

fn row_words(x: &SplitRow) -> Vec<u64> {
    vec![code(x.coord), x.phi_lim.to_bits(), of(x.phi_air), x.npts as u64, of(x.phi_lim_built),
         of(x.phi_air_built), x.phi0.to_bits(), x.b0.to_bits(), x.v0.to_bits(),
         x.b0_frac.to_bits(), x.min_phi.to_bits(), x.max_tt4.to_bits(), u64::from(x.arrested),
         u64::from(x.riding4_valid), x.n_cut_fuel as u64, x.n_cut_gov as u64,
         x.max_req_fuel.to_bits(), x.max_req_gov.to_bits(), x.valve_moved as u64,
         x.stator_moved as u64, x.n_riding4 as u64, u64::from(x.b_max_hit)]
}

const CELL_FIELDS: [&str; 13] = [
    "arm", "s", "phi", "authority", "masked", "mask_leak", "zeros", "c1", "c0", "cyc_fwd",
    "cyc_rev", "pair_rc", "pair_cv",
];

fn cell_words(i: usize, c: &SplitCell) -> Vec<u64> {
    vec![i as u64, c.s.to_bits(), c.phi.to_bits(), ol(c.authority), ol(c.masked), of(c.mask_leak),
         c.zeros as u64, c.c1.to_bits(), c.c0.to_bits(), c.cyc_fwd.to_bits(), c.cyc_rev.to_bits(),
         c.pair_rc.to_bits(), c.pair_cv.to_bits()]
}

fn seq<T>(xs: &[T], enc: impl Fn(&T) -> u64) -> Vec<u64> {
    let mut w = vec![xs.len() as u64];
    w.extend(xs.iter().map(enc));
    w
}

fn liveness_agg(r: &SplitLiveness) -> Vec<u64> {
    vec![u64::from(r.control_ok), u64::from(r.levers_woke), u64::from(r.fuel_off),
         r.four_live as u64, r.n_split as u64]
}

fn arrest_agg(r: &SplitArrest) -> Vec<u64> {
    let mut w = Vec::new();
    for (name, a) in &r.arms {
        w.push(code(name));
        w.extend(seq(&a.marched, |x| x.to_bits()));
        w.extend(seq(&a.arrested, |x| x.to_bits()));
        w.extend([of(a.last_march), of(a.first_arrest), u64::from(a.monotone)]);
    }
    w.extend([of(r.control_bracket.0), of(r.control_bracket.1)]);
    w.extend(seq(&r.owner, |s| code(s)));
    w
}

fn saturation_agg(r: &SplitSaturation) -> Vec<u64> {
    let mut w = vec![of(r.first_sat), of(r.last_march)];
    w.extend(seq(&r.cell, |x| x.to_bits()));
    w.push(u64::from(r.impossible));
    w
}

fn gains_agg(r: &SplitGains) -> Vec<u64> {
    let mut w = Vec::new();
    for a in &r.arms {
        w.extend([of(a.phi_air), a.n_riding as u64, a.n_sampled as u64, a.n_interior as u64,
                  a.skipped.0 as u64, a.skipped.1 as u64, a.authority.len() as u64]);
        for (k, v) in &a.authority {
            w.extend([ol(*k), *v as u64]);
        }
        w.extend(seq(&a.masked, |x| ol(*x)));
        w.push(of(a.max_mask_leak));
        w.extend(seq(&a.zeros, |x| *x as u64));
        w.push(of(a.max_cyc));
    }
    w.push(u64::from(r.vacuous));
    w.extend(seq(&r.n_interior, |x| *x as u64));
    w.extend([u64::from(r.all_differenced), u64::from(r.ever_two_authorities),
              of(r.control_nonzero), of(r.max_mask_leak)]);
    w
}

/// Compare word vectors row by row and field by field, naming the first field that differs.
fn check_rows<const N: usize>(what: &str, got: &[Vec<u64>], want: &[[u64; N]], fields: &[&str]) {
    assert_eq!(got.len(), want.len(), "{what}: row count");
    for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
        assert_eq!(g.len(), N, "{what}: row {i} width");
        for (j, (a, b)) in g.iter().zip(w.iter()).enumerate() {
            assert_eq!(a, b, "{what}: row {i} field `{}` -- got {a:#x} ({}), Python {b:#x} ({})",
                       fields[j], f64::from_bits(*a), f64::from_bits(*b));
        }
    }
}

fn check_agg(what: &str, got: &[u64], want: &[u64]) {
    assert_eq!(got, want, "{what}: aggregate words differ");
}

/// Reset, run, read — on this thread, in this test (slice AH step 7's rule).
fn counted<T>(f: impl FnOnce() -> T) -> (T, CoordCounters) {
    reset_coord_counters();
    let out = f();
    (out, coord_counters())
}

fn assert_rig_restored(core: &ScheduledStatorCore) {
    let t = &core.fuel.inner;
    assert_eq!((t.lag_coord.get(), t.phi_ref.get(), t.sm_air.get()), ("demand", "phi", None),
               "the reader left a knob on the caller's rig");
}

// ---------------------------------------------------------------------------- Python's values

// ---- GENERATED by W:\temp\claude\slice-ai-step4\gen.py from the Python probe's py.json -- do not edit by hand

const LIVENESS_ROWS: [[u64; 22]; 6] = [
    [0x2, 0x3fe8000000000000, 0x7ff4deadbeef0001, 0x155, 0x3fe8000000000000, 0x3fe8000000000000, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe8173be7a4254c, 0x4092b843d971e367, 0x0, 0x1, 0xf2, 0x13f, 0x3f7e5dbb0559149c, 0x3f84a5e020d8ae77, 0x0, 0x0, 0x0, 0x0],
    [0x2, 0x3fe8000000000000, 0x3fe851eb851eb852, 0x155, 0x3fe8000000000000, 0x3fe851eb851eb851, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe8308da73d5c15, 0x4092b8628bc39f95, 0x0, 0x1, 0xe9, 0x13e, 0x3f7d88f87b4d1b3c, 0x3f8496e3c701c65c, 0x103, 0x109, 0x1f, 0x0],
    [0x2, 0x3fe8000000000000, 0x3fe8a3d70a3d70a4, 0x155, 0x3fe8000000000000, 0x3fe8a3d70a3d70a4, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe87328b14a8200, 0x4092b872d4b2a4d9, 0x0, 0x1, 0xd7, 0x13e, 0x3f7c9feb7350d1ec, 0x3f848fad87e67df6, 0x120, 0x127, 0x21, 0x0],
    [0x1, 0x3fe8000000000000, 0x7ff4deadbeef0001, 0x155, 0x3fe8000000000000, 0x3fe8000000000000, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe7eafeecf49025, 0x4093feb2af740d44, 0x0, 0x1, 0xb6, 0x13f, 0x3f7340ab37b60a60, 0x3f83bdf97bd98b8a, 0xf5, 0xfb, 0x17, 0x0],
    [0x1, 0x3fe8000000000000, 0x3fe851eb851eb852, 0x155, 0x3fe8000000000000, 0x3fe851eb851eb851, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe82ad43afbaef4, 0x409403df9a6dadc4, 0x0, 0x1, 0xa0, 0x13e, 0x3f7338bc02ae5354, 0x3f83c545bd557f49, 0x111, 0x118, 0x13, 0x0],
    [0x1, 0x3fe8000000000000, 0x3fe8a3d70a3d70a4, 0x155, 0x3fe8000000000000, 0x3fe8a3d70a3d70a4, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe87328b14a8200, 0x4094050de9a9568f, 0x0, 0x1, 0x95, 0x13e, 0x3f731e9344c6c0d4, 0x3f83d1b1a16bcb13, 0x128, 0x12f, 0x17, 0x0],
];

const ARREST_ROWS: [[u64; 23]; 18] = [
    [0x3fe8a3d70a3d70a4, 0x2, 0x3fe8a3d70a3d70a4, 0x7ff4deadbeef0001, 0x155, 0x3fe8a3d70a3d70a4, 0x3fe8a3d70a3d70a4, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe8a44df41db215, 0x4092ac3b14551ab5, 0x0, 0x1, 0x153, 0x13f, 0x3f8ac92c7fb5a159, 0x3f8632e8381c7d0e, 0x0, 0x0, 0x0, 0x0],
    [0x3fe8b851eb851eb8, 0x2, 0x3fe8b851eb851eb8, 0x7ff4deadbeef0001, 0x155, 0x3fe8b851eb851eb8, 0x3fe8b851eb851eb8, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe8b864884477a9, 0x4090539f7fdbaeb4, 0x0, 0x1, 0x154, 0x140, 0x3f8c16a97051045a, 0x3f8669a335bc14ee, 0x0, 0x0, 0x0, 0x0],
    [0x3fe8bd3c36113405, 0x2, 0x3fe8bd3c36113405, 0x7ff4deadbeef0001, 0x155, 0x3fe8bd3c36113405, 0x3fe8bd3c36113405, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe8bd3cb31e5390, 0x408f490b18314156, 0x0, 0x1, 0x154, 0x140, 0x3f8c6480737c1818, 0x3f8676b987115385, 0x0, 0x0, 0x0, 0x0],
    [0x3fe8be0ded288ce7, 0x2, 0x3fe8be0ded288ce7, 0x7ff4deadbeef0001, 0x155, 0x3fe8be0ded288ce7, 0x3fe8be0ded288ce7, 0x3fe8be0ded288ce8, 0x3f1ef215c8f8a584, 0x0, 0x3f53574d9d9b6772, 0x3fe8be0ded288ce3, 0x408f400000000024, 0x1, 0x0, 0x154, 0x140, 0x3f8c66bf177a5173, 0x3f86774ab2e3ff1e, 0x0, 0x0, 0xd, 0x0],
    [0x3fe8c49ba5e353f8, 0x2, 0x3fe8c49ba5e353f8, 0x7ff4deadbeef0001, 0x155, 0x3fe8c49ba5e353f8, 0x3fe8c49ba5e353f8, 0x3fe8c49ba5e353fc, 0x3f54615100a551f4, 0xbca8791be8576ed7, 0x3f8979a540cea671, 0x3fe8c49ba5e353f5, 0x408f400000000002, 0x1, 0x0, 0x154, 0x140, 0x3f8c683d36dbc386, 0x3f867956c0cfcfc2, 0x0, 0x0, 0x2, 0x0],
    [0x3fe8f5c28f5c28f6, 0x2, 0x3fe8f5c28f5c28f6, 0x7ff4deadbeef0001, 0x155, 0x3fe8f5c28f5c28f6, 0x3fe8f5c28f5c28f6, 0x3fe8f5c28f5c28f9, 0x3f83b10f6a283647, 0xbcf10ac22dc4e882, 0x3fb89d5344b243d8, 0x3fe8f5c28f5c28f1, 0x408f400000000003, 0x1, 0x0, 0x154, 0x140, 0x3f8c735cec5b519f, 0x3f868895b0095ad4, 0x0, 0x0, 0x140, 0x0],
    [0x3fe8a3d70a3d70a4, 0x2, 0x3fe8000000000000, 0x3fe8a3d70a3d70a4, 0x155, 0x3fe8000000000000, 0x3fe8a3d70a3d70a4, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe87328b14a8200, 0x4092b872d4b2a4d9, 0x0, 0x1, 0xd7, 0x13e, 0x3f7c9feb7350d1ec, 0x3f848fad87e67df6, 0x120, 0x127, 0x21, 0x0],
    [0x3fe8b851eb851eb8, 0x2, 0x3fe8000000000000, 0x3fe8b851eb851eb8, 0x155, 0x3fe8000000000000, 0x3fe8b851eb851eb8, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe8856325b78ed7, 0x4092b870ca373538, 0x0, 0x1, 0xd3, 0x13e, 0x3f7c739c140f0478, 0x3f84909ddeb0fca4, 0x127, 0x12e, 0x24, 0x0],
    [0x3fe8bd3c36113405, 0x2, 0x3fe8000000000000, 0x3fe8bd3c36113405, 0x155, 0x3fe8000000000000, 0x3fe8bd3c36113405, 0x3fe8bd5e367dc510, 0x0, 0x0, 0x0, 0x3fe889c928c0bc99, 0x4092b87007dac3fa, 0x0, 0x1, 0xd3, 0x13e, 0x3f7c67b43b312ccc, 0x3f8490eac2ec0564, 0x128, 0x12f, 0x25, 0x0],
    [0x3fe8be0ded288ce7, 0x2, 0x3fe8000000000000, 0x3fe8be0ded288ce7, 0x155, 0x3fe8000000000000, 0x3fe8be0ded288ce7, 0x3fe8be0ded288ce8, 0x3f1ef215c8f8a584, 0x0, 0x3f53574d9d9b6772, 0x3fe88a95863c9399, 0x4092b8700480d0d0, 0x0, 0x1, 0xd2, 0x13e, 0x3f7c650237a30438, 0x3f8490f37bf45524, 0x154, 0x12f, 0x25, 0x0],
    [0x3fe8c49ba5e353f8, 0x2, 0x3fe8000000000000, 0x3fe8c49ba5e353f8, 0x155, 0x3fe8000000000000, 0x3fe8c49ba5e353f8, 0x3fe8c49ba5e353fc, 0x3f54615100a551f4, 0xbca8791be8576ed7, 0x3f8979a540cea671, 0x3fe89112521a084c, 0x4092b86fc4f4b765, 0x0, 0x1, 0xd2, 0x13e, 0x3f7c4e4d4ee11908, 0x3f8491613ff0e65f, 0x154, 0x12f, 0x26, 0x0],
    [0x3fe8f5c28f5c28f6, 0x2, 0x3fe8000000000000, 0x3fe8f5c28f5c28f6, 0x155, 0x3fe8000000000000, 0x3fe8f5c28f5c28f6, 0x3fe8f5c28f5c28f6, 0x3f83b10f6a28376b, 0xbce9c09ce4738175, 0x3fb89d5344b24545, 0x3fe8c1ba7428fe0e, 0x4092b85d3329b4c6, 0x0, 0x1, 0xcd, 0x13e, 0x3f7b174a417c575c, 0x3f849e57ee8583f4, 0x154, 0x12f, 0x21, 0x0],
    [0x3fe8a3d70a3d70a4, 0x2, 0x3fe8a3d70a3d70a4, 0x3fe999999999999a, 0x155, 0x3fe8a3d70a3d70a4, 0x3fe999999999999a, 0x3fe999999999999a, 0x3fa2c0b0d4450957, 0x0, 0x3fd770dd09564bac, 0x3fe963ecec59c712, 0x4092b7f8719929f2, 0x0, 0x1, 0x127, 0x13e, 0x3f7bf08b191644a4, 0x3f84da609c5ad6cb, 0x154, 0x130, 0x24, 0x0],
    [0x3fe8b851eb851eb8, 0x2, 0x3fe8b851eb851eb8, 0x3fe999999999999a, 0x155, 0x3fe8b851eb851eb8, 0x3fe999999999999a, 0x3fe999999999999a, 0x3fa2c0b0d4450957, 0x0, 0x3fd770dd09564bac, 0x3fe963ecec59c712, 0x4092b7f866cfec58, 0x0, 0x1, 0x129, 0x13e, 0x3f7d0fa71f5e774c, 0x3f84da659b48ab4f, 0x154, 0x130, 0x26, 0x0],
    [0x3fe8bd3c36113405, 0x2, 0x3fe8bd3c36113405, 0x3fe999999999999a, 0x155, 0x3fe8bd3c36113405, 0x3fe999999999999a, 0x3fe999999999999a, 0x3fa2c0b0d4450957, 0x0, 0x3fd770dd09564bac, 0x3fe963ecec59c712, 0x4092b7f8118e7269, 0x0, 0x1, 0x12a, 0x13e, 0x3f7d54cd4a62ab1c, 0x3f84da8d174e01e3, 0x154, 0x130, 0x27, 0x0],
    [0x3fe8be0ded288ce7, 0x2, 0x3fe8be0ded288ce7, 0x3fe999999999999a, 0x155, 0x3fe8be0ded288ce7, 0x3fe999999999999a, 0x3fe999999999999a, 0x3fa2c0b0d4450957, 0x0, 0x3fd770dd09564bac, 0x3fe963ecec59c712, 0x4092b7f7fa3040be, 0x0, 0x1, 0x12a, 0x13e, 0x3f7d606008797234, 0x3f84da97e9987e10, 0x154, 0x130, 0x27, 0x0],
    [0x3fe8c49ba5e353f8, 0x2, 0x3fe8c49ba5e353f8, 0x3fe999999999999a, 0x155, 0x3fe8c49ba5e353f8, 0x3fe999999999999a, 0x3fe999999999999a, 0x3fa2c0b0d4450957, 0x0, 0x3fd770dd09564bac, 0x3fe963ecec59c712, 0x4092b7f6eed362a2, 0x0, 0x1, 0x12a, 0x13e, 0x3f7dbd5c07a4fb48, 0x3f84db13b4247e09, 0x154, 0x130, 0x27, 0x0],
    [0x3fe8f5c28f5c28f6, 0x2, 0x3fe8f5c28f5c28f6, 0x3fe999999999999a, 0x155, 0x3fe8f5c28f5c28f6, 0x3fe999999999999a, 0x3fe999999999999a, 0x3fa2c0b0d4450957, 0x0, 0x3fd770dd09564bac, 0x3fe963ecec59c712, 0x4092b7d801f358a3, 0x0, 0x1, 0x133, 0x13f, 0x3f804a4e85a5d53b, 0x3f84e935d9844f71, 0x154, 0x133, 0x33, 0x0],
];

const SATURATION_ROWS: [[u64; 22]; 8] = [
    [0x2, 0x3fe8000000000000, 0x3fe8f5c28f5c28f6, 0x155, 0x3fe8000000000000, 0x3fe8f5c28f5c28f6, 0x3fe8f5c28f5c28f6, 0x3f83b10f6a28376b, 0xbce9c09ce4738175, 0x3fb89d5344b24545, 0x3fe8c1ba7428fe0e, 0x4092b85d3329b4c6, 0x0, 0x1, 0xcd, 0x13e, 0x3f7b174a417c575c, 0x3f849e57ee8583f4, 0x154, 0x12f, 0x21, 0x0],
    [0x2, 0x3fe8000000000000, 0x3fe999999999999a, 0x155, 0x3fe8000000000000, 0x3fe999999999999a, 0x3fe999999999999a, 0x3fa2c0b0d4450957, 0x0, 0x3fd770dd09564bac, 0x3fe963ecec59c712, 0x4092b7f8719929f2, 0x0, 0x1, 0xbf, 0x13e, 0x3f72aea7d4b82df8, 0x3f84da609c5ad6cb, 0x154, 0x130, 0x13, 0x0],
    [0x2, 0x3fe8000000000000, 0x3fea3d70a3d70a3d, 0x155, 0x3fe8000000000000, 0x3fea3d70a3d70a3d, 0x3fea3d70a3d70a3d, 0x3fafec6f4a0122e1, 0xbccd098af3b6c7d6, 0x3fe3f3c58e40b5cc, 0x3fea062276725b5b, 0x4092b74d6accaaa6, 0x0, 0x1, 0xa9, 0x13f, 0x3f64655989992690, 0x3f85143829ed6cbb, 0x154, 0x131, 0x4, 0x0],
    [0x2, 0x3fe8000000000000, 0x3feae147ae147ae1, 0x155, 0x3fe8000000000000, 0x3feae147ae147ae1, 0x3feae147ae147ae0, 0x3fb63e73c441bcd8, 0xbcc0946f69e555bc, 0x3febce10b5522c0e, 0x3fea9cc1651f9403, 0x4092b5eebb2cbc90, 0x0, 0x1, 0x17, 0x13f, 0x3f3fa4d361d1bd80, 0x3f8541351340a5fb, 0x154, 0x145, 0x8, 0x0],
    [0x2, 0x3fe8000000000000, 0x3feb333333333333, 0x155, 0x3fe8000000000000, 0x3feb333333333333, 0x3feb333333333334, 0x3fb9475a792bee6c, 0x0, 0x3fef99311776ea07, 0x3fead3f112350236, 0x4092b5dbec87bf6b, 0x0, 0x1, 0x0, 0x13e, 0xbf42e5e6c9c8d2c0, 0x3f852f21bcce25e0, 0x154, 0x154, 0x0, 0x0],
    [0x2, 0x3fe8000000000000, 0x3feb5c28f5c28f5c, 0x155, 0x3fe8000000000000, 0x3feb5c28f5c28f5c, 0x3feb5c28f5c28f5c, 0x3fb9999999999979, 0xbf8289ceec234e6e, 0x3fefffffffffffd7, 0x3feaf9513ad96203, 0x4092b5c0624e6ea2, 0x0, 0x1, 0x0, 0x13e, 0xbf521894e3982830, 0x3f85201e60378685, 0x132, 0x154, 0x0, 0x1],
    [0x2, 0x3fe8000000000000, 0x3feb851eb851eb85, 0x155, 0x3fe8000000000000, 0x3feb851eb851eb85, 0x3feb851eb851eb84, 0x3fb999999999999a, 0xbf94ebdeae271927, 0x3ff0000000000000, 0x3feb2118a58d9b41, 0x4092b58d67cf86cf, 0x0, 0x1, 0x0, 0x13e, 0xbf5aaba7c8815f50, 0x3f85103cceffda3c, 0x132, 0x154, 0x0, 0x1],
    [0x2, 0x3fe8000000000000, 0x3fec28f5c28f5c29, 0x155, 0x3fe8000000000000, 0x3fec28f5c28f5c29, 0x3fec28f5c28f5c26, 0x3fb999999999999a, 0xbfb07d709b1599ad, 0x3ff0000000000000, 0x3febbfc3a08294af, 0x4092b78271d44984, 0x0, 0x1, 0x0, 0x13c, 0xbf6dfd37ffbb3c30, 0x3f84d4122408272e, 0x131, 0x154, 0x0, 0x1],
];

const GAINS_CLIP_CELLS: [[u64; 13]; 10] = [
    [0x0, 0x3fc147ae147ae149, 0x3fe7eb3ea4d935f8, 0x3, 0x4, 0x0, 0x2, 0x3ec46d3baaaaaaab, 0x3dcdf4fb9c4a0000, 0xbfefffffffe9059d, 0xbfefffffffed24eb, 0x0, 0x3ff0000000043f24],
    [0x0, 0x3fc47ae147ae147d, 0x3fe7f728aab0294d, 0x4, 0x3, 0x0, 0x1, 0x40cd8ef547f7d4c5, 0xbee2ee77bcca7b7a, 0x8000000000000000, 0x0, 0xbf9069b0abe10113, 0x3ff0000000047d69],
    [0x0, 0x3fc7ae147ae147b1, 0x3fe804bfc70af65a, 0x4, 0x3, 0x0, 0x1, 0x40cd9604a3eba6b5, 0xbef9aff96f39e248, 0x8000000000000000, 0x0, 0xbf90c7a64fb31f06, 0x3ff0000000097ad4],
    [0x0, 0x3fcae147ae147ae5, 0x3fe80df45b790d6c, 0x4, 0x3, 0x0, 0x1, 0x40cd9c80a825104d, 0xbed47949c2b20d4a, 0x8000000000000000, 0x0, 0xbf9122a3627cfeb6, 0x3ff000000003f0cd],
    [0x1, 0x3fd0a3d70a3d70a6, 0x3fe8bbb9ef8783e2, 0x4, 0x3, 0x0, 0x1, 0x40cda34f99c71a65, 0xbef3c031c38c6c32, 0x8000000000000000, 0x0, 0xbf9124849eaf37da, 0x3ff0000000073192],
    [0x1, 0x3fd23d70a3d70a40, 0x3fe8bdd4a0e6fe0b, 0x4, 0x3, 0x0, 0x1, 0x40cda8a5bae4c833, 0xbef2bca60beb1ace, 0x8000000000000000, 0x0, 0xbf91711e0aa6f521, 0x3ff000000009d2b2],
    [0x1, 0x3fd3d70a3d70a3da, 0x3fe8bed751b92a6d, 0x4, 0x3, 0x0, 0x1, 0x40cdadbd240ab6bf, 0xbeebce5967d31fc8, 0x8000000000000000, 0x0, 0xbf91ba2372450d2a, 0x3ff00000000343d2],
    [0x1, 0x3fd570a3d70a3d74, 0x3fe8bf2ab6519fb6, 0x4, 0x3, 0x0, 0x1, 0x40cdb2a0ba4674e7, 0xbef31f8cfbaa2740, 0x8000000000000000, 0x0, 0xbf92002371f57fda, 0x3ff000000005693e],
    [0x1, 0x3fd70a3d70a3d70e, 0x3fe8bf122cd9b7ee, 0x4, 0x3, 0x0, 0x1, 0x40cdb757e55eab4d, 0xbed61add2f0bbf70, 0x8000000000000000, 0x0, 0xbf9243840fb83d2e, 0x3ff000000005105a],
    [0x2, 0x3fd999999999999e, 0x3fe9b471e75ea44c, 0x4, 0x3, 0x0, 0x1, 0x40cdc9e06a14cd01, 0xbef3a9f993ce75a0, 0x8000000000000000, 0x0, 0xbf944556124f2ac8, 0x3ff000000007ab7b],
];

const GAINS_DEMAND_CELLS: [[u64; 13]; 11] = [
    [0x1, 0x3fc851eb851eb855, 0x3fe8b0a1a7fe549d, 0x4, 0x3, 0x0, 0x1, 0x40b3b271a11318f6, 0xbecc6bfbec3d1eb8, 0x8000000000000000, 0x0, 0xbf90bdf45ee143e8, 0x3ff00000000065ae],
    [0x1, 0x3fcb851eb851eb89, 0x3fe8b4f60592d001, 0x4, 0x3, 0x0, 0x1, 0x40b3b5374d4c4dbc, 0xbed86cef90ac62c3, 0x8000000000000000, 0x0, 0xbf9102ece27b91d2, 0x3ff000000005ba10],
    [0x1, 0x3fceb851eb851ebd, 0x3fe8b66cdc7d837f, 0x4, 0x3, 0x0, 0x1, 0x40b3b7beb7ce754f, 0xbee7dea7c7f4c1c1, 0x8000000000000000, 0x0, 0xbf913d79b17d97d6, 0x3ff00000000a0637],
    [0x1, 0x3fd0f5c28f5c28f8, 0x3fe8b6cb0e9d7c86, 0x4, 0x3, 0x0, 0x1, 0x40b3ba25944476d8, 0xbec1cfdb960b0870, 0x8000000000000000, 0x0, 0xbf9173744317f98c, 0x3ff0000000028859],
    [0x1, 0x3fd28f5c28f5c292, 0x3fe8b6b7e85e38fd, 0x4, 0x3, 0x0, 0x1, 0x40b3bc77118b1d43, 0xbed12fbf89f8e897, 0x8000000000000000, 0x0, 0xbf91a6f15d2f8280, 0x3ff0000000043b70],
    [0x1, 0x3fd428f5c28f5c2c, 0x3fe8b6741a040c3d, 0x4, 0x3, 0x0, 0x1, 0x40b3beb7beebd94b, 0xbeb34759d08f20a4, 0x8000000000000000, 0x0, 0xbf91d8b6f1ccaf85, 0x3ff0000000017df2],
    [0x1, 0x3fd5c28f5c28f5c6, 0x3fe8b61a597683db, 0x4, 0x3, 0x0, 0x1, 0x40b3c0e9b6dede14, 0xbed0a268a266bcce, 0x8000000000000000, 0x0, 0xbf9209150f4c22cd, 0x3ff0000000061ab7],
    [0x2, 0x3fd428f5c28f5c2c, 0x3fe9ac3d42e83ce3, 0x4, 0x3, 0x0, 0x1, 0x40b3c6db7a5cef61, 0xbeb8504fca701c42, 0x8000000000000000, 0x0, 0xbf9396ed11baca33, 0x3ff000000000b197],
    [0x2, 0x3fd5c28f5c28f5c6, 0x3fe9abd8f03be434, 0x4, 0x3, 0x0, 0x1, 0x40b3c8f708ed1fe6, 0xbed592fb803bd94b, 0x8000000000000000, 0x0, 0xbf93c3c34cbb5f59, 0x3ff000000004d912],
    [0x2, 0x3fd75c28f5c28f60, 0x3fe9ab6f6be996fa, 0x4, 0x3, 0x0, 0x1, 0x40b3cb0616fefd1f, 0xbee352a17a87631b, 0x8000000000000000, 0x0, 0xbf93ef80de982df7, 0x3ff00000000949e1],
    [0x2, 0x3fd8f5c28f5c28fa, 0x3fe9ab042d8d4431, 0x4, 0x3, 0x0, 0x1, 0x40b3cd0930e5f1c3, 0xbeb364d38c1ef3aa, 0x8000000000000000, 0x0, 0xbf941a33b24cf2dc, 0x3ff0000000064568],
];

const LIVENESS_AGG: [u64; 5] = [
    0x1, 0x1, 0x0, 0x21, 0x4,
];

const ARREST_AGG: [u64; 39] = [
    0x7, 0x3, 0x3fe8a3d70a3d70a4, 0x3fe8b851eb851eb8, 0x3fe8bd3c36113405, 0x3,
    0x3fe8be0ded288ce7, 0x3fe8c49ba5e353f8, 0x3fe8f5c28f5c28f6, 0x3fe8bd3c36113405, 0x3fe8be0ded288ce7, 0x1,
    0x8, 0x6, 0x3fe8a3d70a3d70a4, 0x3fe8b851eb851eb8, 0x3fe8bd3c36113405, 0x3fe8be0ded288ce7,
    0x3fe8c49ba5e353f8, 0x3fe8f5c28f5c28f6, 0x0, 0x3fe8f5c28f5c28f6, 0x7ff4deadbeef0001, 0x1,
    0x3, 0x6, 0x3fe8a3d70a3d70a4, 0x3fe8b851eb851eb8, 0x3fe8bd3c36113405, 0x3fe8be0ded288ce7,
    0x3fe8c49ba5e353f8, 0x3fe8f5c28f5c28f6, 0x0, 0x3fe8f5c28f5c28f6, 0x7ff4deadbeef0001, 0x1,
    0x3fe8bd3c36113405, 0x3fe8be0ded288ce7, 0x0,
];

const SATURATION_AGG: [u64; 7] = [
    0x3feb5c28f5c28f5c, 0x3fec28f5c28f5c29, 0x3, 0x3feb5c28f5c28f5c, 0x3feb851eb851eb85, 0x3fec28f5c28f5c29,
    0x0,
];

const GAINS_CLIP_AGG: [u64; 58] = [
    0x7ff4deadbeef0001, 0x17, 0x5, 0x4, 0x0, 0x1,
    0x2, 0x3, 0x1, 0x4, 0x3, 0x2,
    0x3, 0x4, 0x0, 0x2, 0x1, 0x2,
    0x3fefffffffed24eb, 0x3fe8a3d70a3d70a4, 0x17, 0x5, 0x5, 0x0,
    0x0, 0x1, 0x4, 0x5, 0x1, 0x3,
    0x0, 0x1, 0x1, 0x0, 0x3fe999999999999a, 0x4,
    0x1, 0x1, 0x0, 0x0, 0x1, 0x4,
    0x1, 0x1, 0x3, 0x0, 0x1, 0x1,
    0x0, 0x0, 0x3, 0x4, 0x5, 0x1,
    0x0, 0x0, 0x3fefffffffed24eb, 0x0,
];

const GAINS_DEMAND_AGG: [u64; 50] = [
    0x7ff4deadbeef0001, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x7ff4deadbeef0001, 0x0, 0x7ff4deadbeef0001, 0x3fe8a3d70a3d70a4,
    0x21, 0x7, 0x7, 0x0, 0x0, 0x1,
    0x4, 0x7, 0x1, 0x3, 0x0, 0x1,
    0x1, 0x0, 0x3fe999999999999a, 0x13, 0x4, 0x4,
    0x0, 0x0, 0x1, 0x4, 0x4, 0x1,
    0x3, 0x0, 0x1, 0x1, 0x0, 0x1,
    0x3, 0x0, 0x7, 0x4, 0x1, 0x0,
    0x7ff4deadbeef0001, 0x0,
];

// ---- END GENERATED

// ---------------------------------------------------------------------------- the gates

/// § 2 — `split_liveness` at `test_rung80.py`'s fixture (`phi_lim = 0.75`, default walls and
/// coordinates): six rows bit for bit, the five flags, zero counters, the rig restored.
#[test]
fn liveness_is_pythons_bit_for_bit() {
    let core = rig();
    let (r, c) = counted(|| split_liveness(
        &core, &flight(), LO, HI, TT4_MAX, PHI_FUEL, &LIVENESS_PHI_AIRS, &LIVENESS_COORDS, TAUS,
        false, R, SETTLE, DS, V_MAX));
    check_rows("liveness", &r.rows.iter().map(row_words).collect::<Vec<_>>(), &LIVENESS_ROWS,
               &ROW_FIELDS);
    check_agg("liveness", &liveness_agg(&r), &LIVENESS_AGG);
    // The headline, spelled: the clip control fires, the split wakes the levers, and P1's
    // "the fuel leg switches off" is REFUTED (`fuel_off = false`) — the spec's § 2.
    assert!(r.control_ok && r.levers_woke && !r.fuel_off);
    assert_eq!(c, CoordCounters::default(), "rung 79's branch ran on a rung-80 reader");
    assert_rig_restored(&core);
}

/// § 3 — `split_arrest` at `phi_lim_lo = 0.75`: eighteen rows, three arms, the control bracket.
#[test]
fn arrest_is_pythons_bit_for_bit() {
    let core = rig();
    let (r, c) = counted(|| split_arrest(
        &core, &flight(), LO, HI, TT4_MAX, &ARREST_WALLS, PHI_FUEL, 0.80, "demand", TAUS, false,
        R, SETTLE, DS, V_MAX));
    let got: Vec<Vec<u64>> = r.arms.iter()
        .flat_map(|(_, a)| a.rows.iter().map(|(w, x)| {
            let mut v = vec![w.to_bits()];
            v.extend(row_words(x));
            v
        }))
        .collect();
    let mut fields = vec!["wall"];
    fields.extend(ROW_FIELDS);
    check_rows("arrest", &got, &ARREST_ROWS, &fields);
    check_agg("arrest", &arrest_agg(&r), &ARREST_AGG);
    // Rung 74's bracket on the shared arm, and NO split arm arrests — the rung's correction.
    assert_eq!(r.control_bracket, (Some(0.7731), Some(0.7732)));
    assert!(r.owner.is_empty());
    assert_eq!(c, CoordCounters::default());
    assert_rig_restored(&core);
}

/// § 4 — `split_saturation` at its EIGHT default walls. **No Python test calls this reader**, so
/// this gate and the fingerprint's four-wall arm are all that pin it.
#[test]
fn saturation_is_pythons_bit_for_bit() {
    let core = rig();
    let (r, c) = counted(|| split_saturation(
        &core, &flight(), LO, HI, TT4_MAX, PHI_FUEL, &SATURATION_PHI_AIRS, "demand", TAUS, false,
        R, SETTLE, DS, V_MAX));
    check_rows("saturation", &r.rows.iter().map(row_words).collect::<Vec<_>>(),
               &SATURATION_ROWS, &ROW_FIELDS);
    check_agg("saturation", &saturation_agg(&r), &SATURATION_AGG);
    // § 4's bracket [0.850, 0.855], and cells saturated AND marching exist — P2 refuted.
    assert_eq!((r.first_sat, r.impossible), (Some(0.855), false));
    assert!(!r.cell.is_empty());
    assert_eq!(c, CoordCounters::default());
    assert_rig_restored(&core);
}

fn gains(coord: &'static str) -> (SplitGains, CoordCounters, ScheduledStatorCore) {
    let core = rig();
    let (r, c) = counted(|| turbojet::split_wall::split_gains(
        &core, &flight(), LO, HI, TT4_MAX, PHI_FUEL, &GAIN_AIRS, coord, TAUS, false, R, SETTLE,
        DS, V_MAX, GAINS_EVERY).expect("no Abort on this rig"));
    (r, c, core)
}

fn cells_of(r: &SplitGains) -> Vec<Vec<u64>> {
    r.arms.iter().enumerate()
        .flat_map(|(i, a)| a.cells.iter().map(move |c| cell_words(i, c)))
        .collect()
}

/// § 5 in `clip` — the arm carrying the POSITIVE CONTROL (`|cyclic| ≈ 1` where the governor is
/// masked) and the one regime skip.
#[test]
fn gains_clip_is_pythons_bit_for_bit() {
    let (r, c, core) = gains("clip");
    check_rows("gains clip", &cells_of(&r), &GAINS_CLIP_CELLS, &CELL_FIELDS);
    check_agg("gains clip", &gains_agg(&r), &GAINS_CLIP_AGG);
    assert_eq!(r.max_mask_leak, Some(0.0), "the masked column must be EXACTLY zero");
    assert!(r.control_nonzero.is_some_and(|x| (x - 1.0).abs() < 1e-6));
    assert_eq!(c, CoordCounters::default());
    assert_rig_restored(&core);
}

/// § 5 in `demand` — the VACUOUS baseline (`n_riding = 0` at the shared wall), reported, not
/// averaged away; the `default=None` arms fire here and nowhere else.
#[test]
fn gains_demand_is_pythons_bit_for_bit() {
    let (r, c, core) = gains("demand");
    check_rows("gains demand", &cells_of(&r), &GAINS_DEMAND_CELLS, &CELL_FIELDS);
    check_agg("gains demand", &gains_agg(&r), &GAINS_DEMAND_AGG);
    assert!(r.vacuous);
    assert_eq!((r.arms[0].n_riding, r.arms[0].max_cyc, r.control_nonzero), (0, None, None));
    assert_eq!(c, CoordCounters::default());
    assert_rig_restored(&core);
}

/// **THE READBACK — the only instrument on this rung that sees a coordinate defect**, because the
/// counters above never move. `_split_march`'s rig must carry the coordinate it was asked for in
/// `lag_coord` (rung 74's plain ASSIGNMENT — a dispatch would write `phi_ref` on a rung-80
/// machine), `phi_ref` untouched, and the split margin; the caller's knob must be restored.
#[test]
fn split_march_writes_the_coordinate_not_the_reference_and_restores_the_knob() {
    let core = rig();
    for (coord, pa) in [("clip", Some(0.77)), ("demand", None), ("demand", Some(0.80))] {
        let (m, surge, _, traj) = split_march(
            &core, &flight(), LO, HI, TT4_MAX, PHI_FUEL, pa, coord, TAUS, R, SETTLE, DS, V_MAX,
            false);
        let t = &m.fuel.inner;
        let ps = FLOOR;
        assert_eq!((t.lag_coord.get(), t.phi_ref.get()), (coord, "phi"), "{coord} {pa:?}");
        assert_eq!(t.sm_air.get(), pa.map(|p| p / ps - 1.0), "{coord} {pa:?}");
        assert!(surge.is_some(), "the fuel leg is armed");
        assert_eq!(traj.len(), 341);
        assert_rig_restored(&core);
    }
}

/// `AirScope` restores what it DISPLACED — on drop, nested, and on an unwind (Python's `finally`).
#[test]
fn air_scope_restores_on_drop_nest_and_unwind() {
    let core = rig();
    {
        let outer = AirScope::set(&core, Some(0.5));
        assert_eq!(outer.displaced(), None);
        {
            let inner = AirScope::set(&core, Some(0.6));
            assert_eq!(inner.displaced(), Some(0.5));
            assert_eq!(core.fuel.inner.sm_air.get(), Some(0.6));
        }
        assert_eq!(core.fuel.inner.sm_air.get(), Some(0.5));
    }
    assert_eq!(core.fuel.inner.sm_air.get(), None);
    let hit = catch_unwind(AssertUnwindSafe(|| {
        let _a = AirScope::set(&core, Some(0.7));
        panic!("inside the scope");
    }));
    assert!(hit.is_err());
    assert_eq!(core.fuel.inner.sm_air.get(), None, "the unwind did not restore `_sm_air`");
}
