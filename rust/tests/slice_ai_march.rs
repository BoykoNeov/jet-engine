//! SLICE AI step 3 — rung 79's § 5 and § 5.2, `coord_march` and `coord_forced`, **AND THE ONLY
//! SECTION ON THE PLANT ASKS THE KNOB THREE TIMES IN 1 366.**
//!
//! # WHAT PYTHON MEASURED BEFORE THE PORT WAS WRITTEN
//!
//! On `tests/test_rung79.py`'s rig (the same one here, with its four knob assignments — which
//! step 2 could omit and this step cannot: `accel_for` and `cap_march` build their rigs off the
//! CORE, through `at_lever`), the incidence march calls the phi leg 1 366 times and 1 363 of those
//! short-circuit to the shipped `_surge_fuel`. Three bracket the incidence residual, the cap moves
//! by exactly nothing (`d_max = 0.0`), and the reader's own guard reports `vacuous = True`, which
//! `tests/test_rung79.py` asserts as a disclosure. § 5.2 bypasses the short-circuit and finds the
//! coordinate's real footprint, `6.1e-15`, with the forced `phi` solve equal to the shipped one to
//! the bit — a COPY of `_surge_fuel`'s arithmetic, not a rederivation.
//!
//! # WHAT IS PINNED, AND WHY THE LOG IS DIGESTED
//!
//! Every key of both readings bit for bit, the counter vector in the SAME test that runs the march
//! (reset, run, read on this thread — slice AH step 7's rule), and the probe LOG itself as an
//! FNV-1a digest over every row's IEEE bits in order. Six of the march's keys are order statistics
//! or set sizes of that log, and none of them can see a row out of place; the digest can.
//!
//! # THE EXPECTED VALUES ARE PYTHON's, AS BIT PATTERNS
//!
//! Printed by a Python probe of `coord_march` / `coord_forced` on the rig (PyPy, the repo venv) and
//! transcribed as IEEE-754 bits. No golden file is read.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Floor, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::state_coordinate::{
    build_state_coordinate_cascade, coord_counters, coord_forced, coord_march_logged,
    coord_probe_armed, reset_coord_counters, with_probe, CoordCounters, CoordLogRow,
    PHI_REF_INCIDENCE, PHI_REF_PHI,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung79.py`'s module constants, and the reader defaults it takes by omission.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI_JAC: f64 = 0.80;
const SM: f64 = PHI_JAC / FLOOR - 1.0;
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

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// Python's `_rig`, **WITH its four knob assignments** — `m._lag_coord, m._ref_law,
/// m._windup_law, m._cap_law = "demand", "sched", "none", "solve"`. `ref_law` overrides the
/// builder's `"applied"`. Step 2's readers build their own marched machine and could omit these;
/// § 5's `accel_for` and `cap_march` build their rigs off this core through `at_lever`, which
/// copies every one.
fn rig() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    };
    let m = full_of(build_state_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm));
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.cap_law.set("solve");
    m
}

fn f(bits: u64) -> f64 {
    f64::from_bits(bits)
}

/// FNV-1a 64 over five little-endian words per row — `a_cap, p_phi, p_cap, mf_sched` as IEEE bits
/// and `used_fb` as `0`/`1` — exactly as the Python probe digests its log.
fn log_digest(log: &[CoordLogRow]) -> u64 {
    let mut v: u64 = 0xcbf2_9ce4_8422_2325;
    for r in log {
        for w in [r.a_cap.to_bits(), r.p_phi.to_bits(), r.p_cap.to_bits(), r.mf_sched.to_bits(),
                  u64::from(r.used_fb)] {
            for b in w.to_le_bytes() {
                v ^= u64::from(b);
                v = v.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    v
}

// ---------------------------------------------------------------------------- Python's values

const LOG_N: usize = 1366;
const LOG_FNV: u64 = 0xdeae_979b_79b0_7cde;
/// The log's first and last rows: `a_cap, p_phi, p_cap, mf_sched` bits, then `used_fb`. The first
/// row is a FALLBACK call whose cap equals its schedule; the last is one of the three live ones.
const LOG_FIRST: ([u64; 4], bool) =
    ([0x3f85c8378090017a, 0x3f8357a30fa9fb7d, 0x3f8357a30fa9fb7d, 0x3f8357a30fa9fb7d], false);
const LOG_LAST: ([u64; 4], bool) =
    ([0x3f85c8378090017a, 0x3f8357a30fa9fb76, 0x3f8357a30fa9fb76, 0x3f97f79abc10b213], true);

const GAP_MIN: u64 = 0x3fc02539be742030;
const GAP_MED: u64 = 0x3fc02539be742123;
const GAP_MAX: u64 = 0x3fc02539be7421b0;

/// Per row: `s, w_phi, w_inc, d_forced`, then `same_float`. `w_shipped` is not listed: Python's is
/// `w_phi`'s bits at every row (`d_shipped == 0.0`), asserted as such below.
const FORCED_ROWS: [([u64; 4], bool); 10] = [
    ([0x3fbc28f5c28f5c2b, 0x3f874255cd19bc1a, 0x3f874255cd19bc15, 0x3ccb841e3aec0deb], false),
    ([0x3fc3333333333335, 0x3f88777729c30737, 0x3f88777729c3073d, 0x3ccf63c0370c7f78], false),
    ([0x3fc851eb851eb855, 0x3f893f3d4404f6b7, 0x3f893f3d4404f6b9, 0x3cb44794cc27e4db], false),
    ([0x3fcd70a3d70a3d75, 0x3f89cbd2ac0ac240, 0x3f89cbd2ac0ac240, 0x0000000000000000], true),
    ([0x3fd147ae147ae14a, 0x3f8a3cf5fea74f6e, 0x3f8a3cf5fea74f73, 0x3cc864590834d2b5], false),
    ([0x3fd3d70a3d70a3da, 0x3f8aa2a0e4118fe3, 0x3f8aa2a0e4119011, 0x3cfba1ede89d743c], false),
    ([0x3fd666666666666a, 0x3f8b045ac1918d64, 0x3f8b045ac1918d58, 0x3cdc6d31750c53a9], false),
    ([0x3fd8f5c28f5c28fa, 0x3f8b6595d6435909, 0x3f8b6595d6435912, 0x3cd5063b20b39ea3], false),
    ([0x3fdb851eb851eb8a, 0x3f8bc7dc7aef2da6, 0x3f8bc7dc7aef2da7, 0x3ca26e1823c4e035], false),
    ([0x3fde147ae147ae1a, 0x3f8c2be371d645b0, 0x3f8c2be371d645b4, 0x3cc22ca7ca6747a7], false),
];
const D_FORCED: u64 = 0x3cfba1ede89d743c;
const D_FORCED_MED: u64 = 0x3ccb841e3aec0deb;

// ---------------------------------------------------------------------------- § 5

/// **§ 5, bit for bit, with the counters and the log read in the SAME test that runs the march.**
#[test]
fn the_march_is_python_bit_for_bit_and_three_calls_are_the_whole_knob() {
    let core = rig();
    reset_coord_counters();
    let (m, log) = coord_march_logged(
        &core, &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX);

    // THE COUNTERS -- the reader's own copy, and the thread's, which it must not have left moved.
    let c = CoordCounters { hits: 1366, binds: 1366, fb_phi: 1363, fb_inc: 1363, calls_phi: 1366,
                            calls_inc: 1366 };
    assert_eq!((m.hits, m.binds, m.fb_phi, m.fb_inc, m.calls_phi, m.calls_inc),
               (c.hits, c.binds, c.fb_phi, c.fb_inc, c.calls_phi, c.calls_inc));
    assert_eq!(coord_counters(), c, "the reader reads the counters LAST; nothing runs after it");
    assert_eq!((m.br_phi, m.br_inc), (3, 3),
               "THREE of 1 366 calls bracketed the incidence residual -- the whole of § 5's knob");

    // THE LOG, row for row.
    assert_eq!((m.n_log, m.n_armed, log.len()), (LOG_N, LOG_N, LOG_N));
    let row = |r: &CoordLogRow| ([r.a_cap.to_bits(), r.p_phi.to_bits(), r.p_cap.to_bits(),
                                  r.mf_sched.to_bits()], r.used_fb);
    assert_eq!(row(&log[0]), LOG_FIRST);
    assert_eq!(row(&log[LOG_N - 1]), LOG_LAST);
    assert_eq!(log_digest(&log), LOG_FNV, "some row of the probe log differs from Python's");

    // THE READING.
    assert_eq!((m.phi_lim, m.margin, m.inc, m.n, m.same_len), (PHI_JAC, MARGIN, false, 341, true));
    assert_eq!((m.worst.to_bits(), m.where_), (0, None), "P4: not one bit of the march moved");
    assert_eq!(m.sched_moved.to_bits(), 0, "the schedule is not a function of the coordinate");
    assert_eq!((m.n_distinct, m.n_distinct_accel, m.n_distinct_gap), (42, 16, 129));
    assert_eq!((m.n_live, m.n_reach, m.n_both), (3, 1363, 0),
               "§ 5.3: the two sets PARTITION the calls, and their intersection is empty");
    assert_eq!(m.flips, 0);
    assert_eq!((m.gap_min.map(f64::to_bits), m.gap_med.map(f64::to_bits),
                m.gap_max.map(f64::to_bits)),
               (Some(GAP_MIN), Some(GAP_MED), Some(GAP_MAX)));
    assert_eq!((m.d_max.map(f64::to_bits), m.d_med.map(f64::to_bits)), (Some(0), Some(0)));
    assert_eq!(m.vacuous, Some(true),
               "`d_max == 0` is the STRONGEST vacuity, and Python's first version reported it as \
                `None`");

    // THE SCOPES LEFT NOTHING BEHIND: four coordinate scopes and one probe, all restored.
    assert_eq!((core.fuel.inner.lag_coord.get(), core.fuel.inner.phi_ref.get()),
               ("demand", PHI_REF_PHI),
               "a coordinate scope restored the wrong field, or restored it to the wrong value");
    assert!(!coord_probe_armed(), "the probe flag was left up");
}

// ---------------------------------------------------------------------------- § 5.2

/// **§ 5.2, bit for bit — and it moves no rung-79 counter**, because it never calls `_phi_cap`.
#[test]
fn the_forced_reading_is_python_bit_for_bit_and_moves_no_counter() {
    let core = rig();
    reset_coord_counters();
    let fo = coord_forced(
        &core, &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX,
        EVERY);
    assert_eq!(coord_counters(), CoordCounters::default(),
               "the forced reading builds residuals and brackets them itself; `_phi_cap` is never \
                called, so no instrument moves");

    assert_eq!((fo.n, fo.n_binding, fo.n_same_float), (10, 10, 1));
    for (i, (r, (want, same))) in fo.rows.iter().zip(FORCED_ROWS.iter()).enumerate() {
        assert_eq!([r.s.to_bits(), r.w_phi.to_bits(), r.w_inc.to_bits(), r.d_forced.to_bits()],
                   *want, "row {i}");
        assert_eq!(r.same_float, *same, "row {i}");
        assert!(r.binding, "row {i}: every point binds on this rig");
        assert_eq!(r.w_shipped.to_bits(), r.w_phi.to_bits(),
                   "row {i}: the forced `phi` solve is `_surge_fuel`'s arithmetic, copied");
        assert_eq!(r.d_shipped.to_bits(), 0, "row {i}");
    }
    assert_eq!((fo.d_forced.map(f64::to_bits), fo.d_forced_med.map(f64::to_bits),
                fo.d_shipped.map(f64::to_bits)),
               (Some(D_FORCED), Some(D_FORCED_MED), Some(0)));
    assert!(f(D_FORCED) > 0.0 && f(D_FORCED) < 1e-12,
            "the coordinate's footprint is real and float-sized -- `test_rung79.py`'s two bounds");
}

// ---------------------------------------------------------------------------- the probe's scope

fn coord(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_state_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Step 1's two frozen states on a VALVE-LESS machine — see `slice_ai_cells.rs` for why the valve
/// rig cannot supply them.
const SLACK: (f64, f64, f64) = (0.8, 0.9, 0.02);
const BINDING: (f64, f64, f64) = (0.8, 0.9, 0.035);

fn cap_at(m: &ScheduledStatorCore, s: (f64, f64, f64)) -> f64 {
    let floor = Floor::Phi(SurgeLimiter::from_margin(&lp_map(), Spool::Lp, SM));
    (m.fuel.inner.triple_hooks.cap_fuel)(&m.fuel, &flight(), s.0, s.1, s.2, None, Some(&floor),
                                         None)
        .expect("both states are inside the plant's domain")
}

/// **`_with_probe` RESTORES THE ENCLOSING PROBE — flag AND log.**
///
/// A manufactured nest, because no shipped reader holds one: an outer probe logs a call, an inner
/// probe logs one of its own, and the outer logs a third. Python's restore-previous hands the
/// inner scope an EMPTY log and gives the outer back its own row, so the outer ends with TWO rows
/// and the inner with ONE. A restore-to-`None` loses the outer's first row (and the next append is
/// Python's `None.append`); a probe that did not reset the log gives the inner two.
#[test]
fn the_probe_restores_the_enclosing_flag_and_log() {
    let m = coord(&LeverArm::default());
    m.fuel.inner.phi_ref.set(PHI_REF_INCIDENCE);
    assert!(!coord_probe_armed());
    let ((inner, armed_inside), outer) = with_probe(|| {
        cap_at(&m, SLACK);
        let (_, inner) = with_probe(|| {
            cap_at(&m, BINDING);
        });
        let armed_inside = coord_probe_armed();
        cap_at(&m, SLACK);
        (inner, armed_inside)
    });
    assert!(armed_inside, "the inner probe put the OUTER flag down on its way out");
    assert_eq!(inner.len(), 1, "the inner scope must start from an EMPTY log");
    assert!(inner[0].used_fb, "… and its one row is the BINDING call, which fell back");
    assert_eq!(outer.len(), 2, "the outer scope lost its own row across the nest");
    assert!(!outer[0].used_fb && !outer[1].used_fb, "the outer's two rows are the SLACK calls");
    assert!(!coord_probe_armed(), "the outermost probe restored the flag");
}

/// **AN UNWIND RESTORES TOO** — Python's `finally`. A panicking body must not leave the flag up,
/// or every later incidence call on this thread would append to a log that is no longer there.
#[test]
fn the_probe_restores_on_unwind() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(AssertUnwindSafe(|| with_probe::<()>(|| panic!("inside the probe"))));
    std::panic::set_hook(prev);
    assert!(out.is_err());
    assert!(!coord_probe_armed(), "the unwind left the probe flag up");
    // … and the slot is back at `None`: a fresh probe must still start from an empty log.
    let (_, log) = with_probe(|| ());
    assert!(log.is_empty());
}
