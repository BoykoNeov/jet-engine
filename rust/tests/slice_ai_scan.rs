//! SLICE AI step 2 — rung 79's §§ 1–4 readers, `coord_scan` and `coord_census`, **AND THE SCAN'S
//! TWO EXACT ZEROS ARE THE FALLBACK COMPARED WITH ITSELF.**
//!
//! # WHAT THE COUNTERS SAY THAT THE VALUES CANNOT
//!
//! Measured in Python before the port was written (`tests/test_rung79.py`'s rig, the same one
//! here): over the scan's 10 points, all 30 `phi` set-point solves AND all 30 incidence ones
//! short-circuit to the shipped `_surge_fuel`, which brackets its own hardcoded `phi` residual
//! whatever coordinate asked. So `w_inc == w_phi` bit for bit at every row, and `d_set` and D3's
//! `dwdq_err` are exact zeros BY SUBSTITUTION. Rung 79 § 5.1 records that mechanism for the MARCH;
//! its § 1 table still reads D3's zero as *"not small, exactly zero"*.
//!
//! A zero value-diff is two findings and only a counter splits them (slice AI's pre-flight), so
//! [`the_scan_is_python_bit_for_bit_and_every_incidence_solve_fell_back`] pins the counter vector
//! `[0, 0, 30, 30, 30, 30]` in the SAME test that runs the scan — reset, run, read, on this
//! thread, never through a shared cache (slice AH step 7).
//!
//! **What does measure the coordinate here: D2 alone** — the slope ratio, read straight off the
//! residual and never through a solve. § 4's census walks `Gi` for real (its incidence counters
//! stay at zero because it never SOLVES it,
//! [`the_census_is_python_bit_for_bit_and_walks_the_incidence_residual_directly`]), but the walk
//! reads only signs, so a census that walked `Gs` twice is bit-identical — injected, and invisible.
//!
//! # THE INJECTION SWEEP (plan § 5.33.2), PREDICTED BEFORE IT RAN, TWELVE OF TWELVE RIGHT
//!
//! Caught: the incidence `dw*/dq` leg or `w_inc` solved in `phi` (by the COUNTERS only — no value
//! moves); the perturbation leaked out of the freeze; `locate = false`; the incidence slope taken
//! off `Gs`; the freeze restored instead of clobbered. Survived, each for a stated reason: solve
//! order swapped; the incidence slope read at `w_inc` (it IS `w_phi` here); the census walking `Gs`
//! twice; two `f64::max` spellings (no `NaN` reached); a wrong per-row `default` (no empty zip).
//!
//! # THE EXPECTED VALUES ARE PYTHON's, AS BIT PATTERNS
//!
//! Every float below was printed by a Python probe of `coord_scan` / `coord_census` on the rig and
//! transcribed as its IEEE-754 bits. No golden file is read.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::residual_gauge::gauge_points;
use turbojet::state_coordinate::{
    build_state_coordinate_cascade, coord_at, coord_census, coord_counters, coord_scan,
    reset_coord_counters, CoordCounters, COORD_AT_DQ, COORD_AT_REL, COORD_CENSUS_WALK,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

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

/// Python's `_rig`. Its four knob assignments are not reproduced: `_ledger_march` builds its
/// marched machine with its own, which is why rung 78's `slice_ah_gauge.rs` omits them too — and
/// the bit-for-bit gates below would see it if that were wrong.
fn rig() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    };
    match build_state_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

fn f(bits: u64) -> f64 {
    f64::from_bits(bits)
}

// ---------------------------------------------------------------------------- Python's values

/// Per row: `s, w_phi, slope_phi, slope_inc, ratio, dwdq_phi`. `w_inc` and `dwdq_inc` are not
/// listed because Python's are the SAME bits as `w_phi` and `dwdq_phi` at every row — the finding,
/// asserted as such below rather than hidden in a table.
const SCAN_ROWS: [[u64; 6]; 10] = [
    [0x3fbc28f5c28f5c2b, 0x3f874255cd19bc1a, 0x40277075617baba9, 0x40324fdbb3984a3d, 0x3ff8ffffff3af5b2, 0x3fafb19186ff5ecb],
    [0x3fc3333333333335, 0x3f88777729c30737, 0x4026e97296fb9c66, 0x4031e66184bcbe2c, 0x3ff8fffffe4c7be5, 0x3fb04414601c870c],
    [0x3fc851eb851eb855, 0x3f893f3d4404f6b7, 0x40267925aac7cee2, 0x40318ea56dc0b697, 0x3ff9000000787b66, 0x3fb0929f90ee4e04],
    [0x3fcd70a3d70a3d75, 0x3f89cbd2ac0ac240, 0x402614cb2e451f04, 0x4031403ebc6126fa, 0x3ff900000055b8ca, 0x3fb0d1f9a8ef1c38],
    [0x3fd147ae147ae14a, 0x3f8a3cf5fea74f6e, 0x4025b60b1cbbaebd, 0x4030f638aee6efe3, 0x3ff9000000ab6dec, 0x3fb10ab448235fb0],
    [0x3fd3d70a3d70a3da, 0x3f8aa2a0e4118fe3, 0x402559e320a9ae52, 0x4030ae3970e42836, 0x3ff8ffffff0f975a, 0x3fb1411c7f4fe1f4],
    [0x3fd666666666666a, 0x3f8b045ac1918d64, 0x4024fefd7e7af2c2, 0x403067360c010978, 0x3ff9000001d0d305, 0x3fb17738f0e06896],
    [0x3fd8f5c28f5c28fa, 0x3f8b6595d6435909, 0x4024a4ca9cc08276, 0x403020be4a6b4251, 0x3ff8ffffffeebbb3, 0x3fb1adf767372238],
    [0x3fdb851eb851eb8a, 0x3f8bc7dc7aef2da6, 0x40244b14a98d7a1d, 0x402fb550492ef806, 0x3ff900000033f77e, 0x3fb1e5c28c414eac],
    [0x3fde147ae147ae1a, 0x3f8c2be371d645b0, 0x4023f1cb845d7bc8, 0x402f29cdfed21169, 0x3ff9000000000000, 0x3fb21ecbda93e834],
];
/// `1/(0.8·0.8)` in floats — **NOT `1.5625`**: `0.8·0.8` rounds below `0.64`.
const PREDICTED_RATIO: u64 = 0x3ff8ffffffffffff;
const RATIO_ERR: u64 = 0x3e3297cd10000000;

/// Per row: `w0` and the single located root (both coordinates — the SAME bits, measured).
/// `s` is the scan's, row for row.
const CENSUS_ROWS: [[u64; 2]; 10] = [
    [0x3f874255cd19bc1a, 0x3ff000000000000a],
    [0x3f88777729c30737, 0x3ff0000000000005],
    [0x3f893f3d4404f6b7, 0x3ff0000000000002],
    [0x3f89cbd2ac0ac240, 0x3feffffffffffffe],
    [0x3f8a3cf5fea74f6e, 0x3ff0000000000004],
    [0x3f8aa2a0e4118fe3, 0x3ff0000000000003],
    [0x3f8b045ac1918d64, 0x3ff0000000000002],
    [0x3f8b6595d6435909, 0x3ff0000000000004],
    [0x3f8bc7dc7aef2da6, 0x3ff0000000000005],
    [0x3f8c2be371d645b0, 0x3ff0000000000001],
];

// ---------------------------------------------------------------------------- §§ 1–3

/// **§§ 1–3, BIT FOR BIT, WITH THE COUNTERS THAT SAY WHAT TWO OF ITS ZEROS ARE.**
///
/// Python's six bars first — they are `test_rung79.py`'s own asserts — then every value as bits,
/// then the finding: 30 of 30 incidence solves answered by the fallback.
#[test]
fn the_scan_is_python_bit_for_bit_and_every_incidence_solve_fell_back() {
    let m = rig();
    reset_coord_counters();
    let sc = coord_scan(&m, &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE,
                        DS, V_MAX, COORD_AT_DQ, EVERY);
    let counts = coord_counters();

    // --- test_rung79.py's bars.
    assert_eq!(sc.n, 10);
    assert!((sc.predicted_ratio - 1.0).abs() > 0.1, "vacuous at phi_lim = 1");
    assert!(sc.ratio_err.unwrap() < 1e-6, "D2: {:e}", sc.ratio_err.unwrap());
    assert!(sc.dwdq_err.unwrap() < 1e-9, "D3: {:e}", sc.dwdq_err.unwrap());
    for row in &sc.rows {
        assert!(row.slope_phi * row.slope_inc > 0.0, "a positive multiplier: {row:?}");
        assert!((row.ratio - 1.0).abs() > 0.1, "the slope did not move: {row:?}");
        // THE LEAKED-FREEZE DISCRIMINATOR: a solve that ignored `q ± dq` would difference two
        // identical floats to an exact zero.
        assert!(row.dwdq_phi.abs() > 1e-4, "dw*/dq is itself dead: {row:?}");
    }

    // --- Python's values, as bits.
    assert_eq!(sc.predicted_ratio.to_bits(), PREDICTED_RATIO);
    assert_eq!(sc.ratio_err.unwrap().to_bits(), RATIO_ERR);
    for (row, want) in sc.rows.iter().zip(SCAN_ROWS.iter()) {
        let got = [row.s, row.w_phi, row.slope_phi, row.slope_inc, row.ratio, row.dwdq_phi];
        for (k, (g, w)) in got.iter().zip(want.iter()).enumerate() {
            assert_eq!(g.to_bits(), *w, "row s = {}: field {k} is {g:e}, Python {:e}", row.s, f(*w));
        }
    }

    // --- THE FINDING: both "exactly zero" readings are the fallback compared with itself.
    assert_eq!(counts, CoordCounters { fb_phi: 30, fb_inc: 30, calls_phi: 30, calls_inc: 30,
                                       ..Default::default() },
               "Python's measured vector [hits, binds, fb_phi, fb_inc, calls_phi, calls_inc] = \
                [0, 0, 30, 30, 30, 30]: every set-point solve in the scan short-circuited to \
                `_surge_fuel`, in BOTH coordinates");
    assert_eq!(sc.n_same_float, 10, "so every row's two set points are one float …");
    for row in &sc.rows {
        assert_eq!(row.w_inc.to_bits(), row.w_phi.to_bits());
        assert_eq!(row.dwdq_inc.to_bits(), row.dwdq_phi.to_bits());
        assert_eq!(row.d_set, 0.0);
    }
    assert_eq!((sc.d_set, sc.d_set_min, sc.dwdq_err), (Some(0.0), Some(0.0), Some(0.0)),
               "… and `d_set` and D3 are zeros BY SUBSTITUTION — rung 79 § 5.1's mechanism \
                reaching § 1's table, not evidence of invariance");
}

// ---------------------------------------------------------------------------- § 4

/// **§ 4, BIT FOR BIT — AND THE INCIDENCE COUNTERS DO NOT MOVE, because the census walks `Gi`
/// directly rather than solving it.** Only the anchor `w0` is a solve (in `phi`, and it falls
/// back). The located roots agree to the bit because the bisection reads only SIGNS and `h > 0`
/// makes the two signs one — which is also why nothing here could see `Gs` walked in `Gi`'s place.
#[test]
fn the_census_is_python_bit_for_bit_and_walks_the_incidence_residual_directly() {
    let m = rig();
    let (lo, hi, n) = COORD_CENSUS_WALK;
    assert_eq!((lo, hi, n), (0.2, 3.0, 400), "Python's `coord_census` defaults");
    reset_coord_counters();
    let ce = coord_census(&m, &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, false, R, SETTLE,
                          DS, V_MAX, EVERY, lo, hi, n);
    let counts = coord_counters();

    // --- test_rung79.py's bars.
    assert!(ce.counts_equal, "D1: equal counts");
    assert!(ce.worst.unwrap() < 1e-9, "D1: located roots agree");
    assert_eq!(ce.n_roots, vec![1], "the phi leg's root is unique on rung 78's window");

    // --- Python's values, as bits.
    assert_eq!(ce.n, 10);
    for ((row, want), scan_row) in ce.rows.iter().zip(CENSUS_ROWS.iter()).zip(SCAN_ROWS.iter()) {
        assert_eq!(row.s.to_bits(), scan_row[0], "the census reads the scan's points");
        assert_eq!(row.w0.to_bits(), want[0], "s = {}: w0", row.s);
        assert_eq!((row.n_phi, row.n_inc), (1, 1));
        assert_eq!(row.roots_phi.iter().map(|x| x.to_bits()).collect::<Vec<_>>(), vec![want[1]]);
        assert_eq!(row.roots_inc.iter().map(|x| x.to_bits()).collect::<Vec<_>>(), vec![want[1]]);
        assert_eq!(row.worst, 0.0);
    }
    assert_eq!(ce.worst, Some(0.0));

    assert_eq!(counts, CoordCounters { fb_phi: 10, calls_phi: 10, ..Default::default() },
               "Python's measured vector [0, 0, 10, 0, 10, 0]: ten anchor solves in `phi`, each \
                answered by the fallback, and NO incidence solve at all");
}

// ---------------------------------------------------------------------------- the freeze

/// **`_coord_at` CLOBBERS THE FREEZE, AS PYTHON's `finally` DOES** — both cells come back `None`,
/// not to what the caller had set. Manufactured: no shipped caller enters with a freeze up.
/// And the row it returns is the scan's first row to the bit, so the reader is `coord_at` mapped.
#[test]
fn coord_at_clears_both_frozen_cells_rather_than_restoring_them() {
    let (m, surge, _accel, pts) = gauge_points(
        &rig(), &flight(), LO, HI, TT4_MAX, MARGIN, TAUS, R, SETTLE, DS, V_MAX, false, PHI_JAC,
        EVERY);
    m.fuel.inner.b_state.set(Some(0.123));
    m.fuel.inner.v_state.set(Some(0.045));
    let row = coord_at(&flight(), &m, &pts[0], surge.as_ref().unwrap(), COORD_AT_DQ, COORD_AT_REL);
    assert_eq!((m.fuel.inner.b_state.get(), m.fuel.inner.v_state.get()), (None, None),
               "Python writes `None, None` in its `finally` — a clobber, not a restore");
    assert_eq!(row.w_phi.to_bits(), SCAN_ROWS[0][1]);
    assert_eq!(row.dwdq_phi.to_bits(), SCAN_ROWS[0][5]);
}
