//! RUNG 79 — **THE STATE COORDINATE**: rung 78 § 9's fourth seam.
//!
//! ```text
//! Gi(w) = m_lim - M_i(w) = 1/phi(w) - 1/phi_lim = Gs(w)·h(w),   h = 1/(phi·phi_lim) > 0
//! ```
//!
//! **HEADLINE: a coordinate is a GAUGE on the set point and UNREACHABLE on the plant** — the
//! branch that makes a leg AUTHORITATIVE (`_cap_free`'s binding short-circuit) is the branch that
//! substitutes the original coordinate back in, via `_surge_fuel`'s own hardcoded `Gs`.
//!
//! Ported from `tests/test_rung79.py`: **25 collected tests, 1 of them `slow` there**
//! (`test_the_same_rig_DOES_march_when_the_wall_is_lowered`). **This file has 28**; the three
//! extra are DECLARED below rather than absorbed. Names are lower-cased where Python's carry
//! capitals (`the_min_never_flips_and_that_is_vacuous`, …); the map says which.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, **3 ADDED**, 0 COLLAPSED
//!
//! | # | `tests/test_rung79.py` | here |
//! |---|---|---|
//! | 1 | `the_incidence_residual_is_free_of_T_c_and_v` | [`the_incidence_residual_is_free_of_t_c_and_v`] |
//! | 2 | `the_multiplier_is_strictly_positive_so_signs_agree` | [`the_multiplier_is_strictly_positive_so_signs_agree`] |
//! | 3 | `reduce_phi_is_rung_78_by_dispatch` | [`reduce_phi_is_rung_78_by_dispatch`] — see below |
//! | 4 | `reduce_direction_one_the_knob_is_wired` | [`reduce_direction_one_the_knob_is_wired`] |
//! | 5 | `reduce_direction_two_the_set_point_does_not` | [`reduce_direction_two_the_set_point_does_not`] |
//! | 6 | `the_forced_bypass_reproduces_the_shipped_solve` | [`the_forced_bypass_reproduces_the_shipped_solve`] |
//! | 7 | `slope_scales_by_the_derived_factor` | [`slope_scales_by_the_derived_factor`] |
//! | 8 | `sensitivity_is_coordinate_invariant` | [`sensitivity_is_coordinate_invariant`] — PLUMBING, see below |
//! | 9 | `the_sensitivity_reading_is_not_trivially_zero` | [`the_sensitivity_reading_is_not_trivially_zero`] |
//! | 10 | `root_counts_are_equal_in_both_coordinates` | [`root_counts_are_equal_in_both_coordinates`] |
//! | 11 | `uniqueness_survives_the_coordinate` | [`uniqueness_survives_the_coordinate`] |
//! | 12 | `the_phi_leg_wins_the_inner_min_everywhere` | [`the_phi_leg_wins_the_inner_min_everywhere`] |
//! | 13 | `the_knob_was_live_on_the_plant_at_least_sometimes` | [`the_knob_was_live_on_the_plant_at_least_sometimes`] |
//! | 14 | `the_complementarity_is_exact` | [`the_complementarity_is_exact`] |
//! | 15 | `the_min_never_flips_AND_that_is_vacuous` | [`the_min_never_flips_and_that_is_vacuous`] |
//! | 16 | `the_gap_log_records_distinct_FLOATS_not_distinct_states` | [`the_gap_log_records_distinct_floats_not_distinct_states`] |
//! | 17 | `the_trajectory_and_the_schedule_do_not_move` | [`the_trajectory_and_the_schedule_do_not_move`] |
//! | 18 | `the_march_stands_still_AND_THAT_IS_THE_SCOPE_OF_SECTION_5` | [`the_march_stands_still_and_that_is_the_scope_of_section_5`] |
//! | 19 | `the_floor_is_armed_AT_the_initial_condition` | [`the_floor_is_armed_at_the_initial_condition`] |
//! | 20 | `the_same_rig_DOES_march_when_the_wall_is_lowered` (`slow`) | [`the_same_rig_does_march_when_the_wall_is_lowered`] |
//! | 21 | `gap_at_zero_margin_is_EXACTLY_zero` | [`gap_at_zero_margin_is_exactly_zero`] |
//! | 22 | `the_gap_residual_is_rung_77s_STIFFNESS` | [`the_gap_residual_is_rung_77s_stiffness`] |
//! | 23 | `incidence_times_a_live_rung_78_gauge_is_refused` | [`incidence_times_a_live_rung_78_gauge_is_refused`] |
//! | 24 | `the_carried_knob_survives_at_lever` | [`the_carried_knob_survives_at_lever`] |
//! | 25 | `the_probe_flag_is_written_on_the_class` | [`the_probe_flag_is_written_on_the_class`] |
//! | **+1** | **— none —** | [`the_forced_binding_walk_refuses_when_no_root_lies_below`] (`engine.py:21285`) |
//! | **+2** | **— none —** | [`the_forced_slack_walk_refuses_when_no_root_lies_above`] (`engine.py:21301`) |
//! | **+3** | **— none —** | [`the_forced_slack_walk_finds_pythons_root_in_both_coordinates`] (step 3's J6) |
//!
//! # WHERE THE PORT DIFFERS FROM A LINE-BY-LINE COPY, AND WHY
//!
//! * **#3's last line is VACUOUS in Python and is not copied.** `G.__code__.co_consts is not
//!   None` holds for every Python function. What the test's NAME claims — at `"phi"` the rung-79
//!   cell takes rung 78's body — is gated instead: on a valve-off rung-79 rig at a SLACK state the
//!   cell returns rung 78's float to the bit and moves no rung-79 counter, and at `"incidence"`
//!   the same call moves `hits`. The counters are what stop the equality from being a function
//!   compared with itself.
//! * **#8 and #9 carry the CORRECTED wording** (commit `9f1d11a`). Every incidence solve in the
//!   scan falls back to the `phi` solve, so `dwdq_err == 0` is the fallback compared with itself:
//!   #8 guards that the two paths are wired to one solve, not the invariance.
//! * **`isinstance` is held STRICTER than Python** (#24): the rebuilt machine's pointers must be
//!   rung 79's AND not rung 78's — `rung78.rs`'s lesson (b): under a table re-aimed at the parent,
//!   the positive half alone passes. A rung-80 subclass would pass Python's `isinstance`; here
//!   the rig is rung 79, so anything else is a leak.
//! * **#25's "fresh machine" has no Rust counterpart.** The probe flag is per-thread state, never
//!   a core field (plan § 5.33 (iv)), so every machine reads the same flag by construction.
//! * **The surge floors are Python's LITERAL `SurgeLimiter(spool="lp", phi_lim=PHI_JAC)`**, not
//!   `from_margin`.
//!
//! # `_a_cap` IS PORTED WITH ITS OUTER FREEZE (plan § 5.33 (vii))
//!
//! Python's helper freezes `(q, v)` on the marched machine, then solves and calls `_c_at` twice
//! inside that block. Each `_c_at` sets its own freeze and CLOBBERS it to `None` on exit, so the
//! first call is a NEST and the second re-freezes from `None`. Between the two nothing reads the
//! plant, so the outer freeze is value-invisible here — it is ported because the source has it,
//! and because it is the suite's only measured `_b_state`/`_v_state` nest.
//!
//! # THE THREE ADDED GATES — `forced_cap`'s two refusals and its SLACK arm
//!
//! Plan § 5.33 (vi) drove both refusals in Python and found no suite assertion; step 3's J6
//! (the slack walk growing by `shrink`) SURVIVED because the shipped `coord_forced` binds at all 10
//! points. A refusal gate cannot kill J6 — a walk in the wrong direction also refuses — so +3
//! pins the SLACK success: on a valve-off rung-79 rig at `(0.8, 0.9, 0.02)`, measured on PyPy by
//! `W:\temp\claude\slice-ai-step5\probe_py.py`, the two coordinates' roots differ by one ulp and
//! each equals `cap_free`'s slack arm to the bit.
//!
//! # COUNTERS
//!
//! Rung 79's six counters are THREAD-LOCAL (plan § 5.33 (iv)), so no gate here holds a lock;
//! every gate that reads them resets, runs and reads inside itself.

use std::ptr::fn_addr_eq;
use std::sync::OnceLock;

use turbojet::bleed_transient::LeverArm;
use turbojet::demand_coordinate::cap_free;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AccelSchedule, Floor, FuelPoint, PointExtra, SurgeLimiter};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::residual_gauge::{accel_cap_fn, R78_TRIPLE};
use turbojet::sensed_cap::{accel_for, c_at, cap_march, C_AT_REL};
use turbojet::state_coordinate::{
    build_state_coordinate_cascade, coord_census, coord_counters, coord_forced, coord_march,
    coord_probe_armed, coord_scan, forced_cap, phi_cap, reset_coord_counters, with_probe,
    CoordCensus, CoordCounters, CoordForced, CoordMarch, CoordScan, COORD_AT_DQ,
    COORD_CENSUS_WALK, FORCED_GROW, FORCED_N, FORCED_SHRINK, PHI_REF_INCIDENCE, PHI_REF_PHI, R79,
    R79_TRIPLE,
};
use turbojet::stator_transient::{IncidenceLimiter, ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{MarchedBleed, MarchedStator};

// ============================================================================== the grid
//
// `tests/test_rung79.py`'s module constants, verbatim.

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
const PHI_JAC: f64 = 0.80;
const MARGIN: f64 = 0.10;
/// The standstill section's own four (`test_rung79.py:281`), and the readers' defaults.
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const R: f64 = 0.5;
const S_SETTLE: f64 = 1.2;
const DS: f64 = 0.005;

// ------------------------------------------------ the readers' unspelled defaults (`engine.py`)
//
// `coord_scan` (`:21049`), `coord_census` (`:21084`), `coord_march` (`:21125`), `coord_forced`
// (`:21306`): `phi_lim = 0.80`, `margin = 0.10`, `inc = False`, `every = 8`.

const INC: bool = false;
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

fn suite_arm(sm: f64) -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    }
}

fn set_knobs(m: &ScheduledStatorCore) {
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.cap_law.set("solve");
}

/// Python's `_rig(design)` — a rung-79 machine with the suite's valve and stator and its four
/// knob assignments.
fn rig() -> ScheduledStatorCore {
    let sm = PHI_JAC / FLOOR - 1.0;
    let m = full_of(build_state_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &suite_arm(sm)));
    set_knobs(&m);
    m
}

/// A rung-79 machine with NO lever — the only rig on which the phi leg can be SLACK
/// (`slice_ai_cells.rs`'s measured reason: the suite's valve holds `phi` at exactly its wall).
fn bare() -> ScheduledStatorCore {
    full_of(build_state_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &LeverArm::default()))
}

/// Python's LITERAL `SurgeLimiter(spool="lp", phi_lim=…)`.
fn surge_at(phi_lim: f64) -> Floor {
    Floor::Phi(SurgeLimiter::new(Spool::Lp, phi_lim))
}

/// The two frozen states of `slice_ai_cells.rs`, on [`bare`]: `phi(0.02) = 0.86` is above the
/// wall (SLACK), `phi(0.035) = 0.77` below it (BINDING).
const SLACK: (f64, f64, f64) = (0.8, 0.9, 0.02);

// ============================================================================== the fixtures
//
// Python's module-scoped fixtures. The readings are plain data, so each is computed once per
// binary; a CORE holds `Cell`s and cannot be shared, so `still` is rebuilt per test.

fn scan() -> &'static CoordScan {
    static S: OnceLock<CoordScan> = OnceLock::new();
    S.get_or_init(|| coord_scan(&rig(), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, INC,
                                R, S_SETTLE, DS, V_MAX, COORD_AT_DQ, EVERY))
}

fn census() -> &'static CoordCensus {
    static S: OnceLock<CoordCensus> = OnceLock::new();
    let (lo, hi, n) = COORD_CENSUS_WALK;
    S.get_or_init(|| coord_census(&rig(), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS,
                                  INC, R, S_SETTLE, DS, V_MAX, EVERY, lo, hi, n))
}

fn forced() -> &'static CoordForced {
    static S: OnceLock<CoordForced> = OnceLock::new();
    S.get_or_init(|| coord_forced(&rig(), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS,
                                  INC, R, S_SETTLE, DS, V_MAX, EVERY))
}

fn march() -> &'static CoordMarch {
    static S: OnceLock<CoordMarch> = OnceLock::new();
    S.get_or_init(|| coord_march(&rig(), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, INC,
                                 R, S_SETTLE, DS, V_MAX))
}

// ------------------------------------------------------------------ the standstill helpers

/// Python's `_march_at` — `coord_march`'s own march with the TRAJECTORY handed back.
fn march_at(phi_jac: f64, margin: f64)
    -> (ScheduledStatorCore, ScheduledStatorCore, AccelSchedule, Vec<FuelPoint>) {
    let mut m0 = rig();
    let sm = phi_jac / FLOOR - 1.0;
    if phi_jac != PHI_JAC {
        // the positive control needs its own floors
        m0 = m0.at_lever(&suite_arm(sm));
        set_knobs(&m0);
    }
    let fl = flight();
    let accel = accel_for(&m0, &fl, LO, HI, sm, TT4_MAX, TAUS, V_MAX, false, margin);
    let (m, _, _, traj) = cap_march(&m0, &fl, LO, HI, TT4_MAX, sm, TAUS, R, S_SETTLE, DS, V_MAX,
                                    false, "demand", "sched", "none", None, "solve", &accel, None);
    (m0, m, accel, traj)
}

fn still() -> (ScheduledStatorCore, ScheduledStatorCore, AccelSchedule, Vec<FuelPoint>) {
    march_at(PHI_JAC, MARGIN)
}

/// `p[key]` on a demand-coordinate point — the keys the standstill gates read.
fn key(p: &FuelPoint, k: &str) -> f64 {
    match k {
        "nu_lp" => p.nu_lp,
        "nu_hp" => p.nu_hp,
        "mf" => p.mf,
        "mf_sched" => p.mf_sched,
        "phi_lp" => p.phi_lp,
        "Tt4" => p.tt4,
        "cap_fuel" => match p.extra {
            PointExtra::Demand { cap_fuel, .. } => cap_fuel,
            _ => panic!("`cap_fuel` is a demand-coordinate key -- this march did not run `demand`"),
        },
        _ => panic!("no key {k:?}"),
    }
}

/// Python's `_spread`. `max(abs(min(v)), 1e-30)` has an EXPRESSION first, so the fold is written
/// out rather than spelled `f64::max`.
fn spread(traj: &[FuelPoint], k: &str) -> f64 {
    let v: Vec<f64> = traj.iter().map(|p| key(p, k)).collect();
    let lo = v.iter().copied().fold(f64::INFINITY, |a, x| if x < a { x } else { a });
    let hi = v.iter().copied().fold(f64::NEG_INFINITY, |a, x| if x > a { x } else { a });
    let den = if 1e-30 > lo.abs() { 1e-30 } else { lo.abs() };
    (hi - lo) / den
}

fn bv(p: &FuelPoint) -> (f64, f64) {
    match p.extra {
        PointExtra::Demand { b, v, .. } => (b, v),
        _ => panic!("`b`/`v` are six-state keys -- this march did not run `demand`"),
    }
}

/// Python's `_a_cap` — the accel leg's set point at ONE frozen state, at an arbitrary `margin`,
/// **with its outer freeze** (module header).
fn a_cap(m0: &ScheduledStatorCore, m: &ScheduledStatorCore, p: &FuelPoint, margin: f64)
    -> (f64, f64, f64) {
    let fl = flight();
    let sm = PHI_JAC / FLOOR - 1.0;
    let accel = accel_for(m0, &fl, LO, HI, sm, TT4_MAX, TAUS, V_MAX, false, margin);
    let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let (q, v) = bv(p);
    let cap = accel_cap_fn(&m.fuel, &fl, a, h, &accel);
    let _sb = MarchedBleed::set(&m.fuel.inner, q);
    let _sv = MarchedStator::set(&m.fuel.inner, v);
    let g = |x: f64| -> Result<f64, Abort> { Ok(x - cap(x)?) };
    let w = cap_free(&g, ms, &|| m.fuel.try_sched_fuel(&fl, a, h, ms, &accel))
        .expect("the accel leg's fixed point solves at the standstill");
    let c_w = c_at(m, &fl, a, h, &accel, w, q, v, C_AT_REL).expect("rung 76's c at w");
    let c_mf = c_at(m, &fl, a, h, &accel, p.mf, q, v, C_AT_REL).expect("rung 76's c at mf");
    (w, c_w, c_mf)
}

// ============================================================================== § 1: the cancellation

/// `Gi = 1/phi − 1/phi_lim`: the blade metal and the stator setting CANCEL. Checked against rung
/// 60's SHIPPED `IncidenceLimiter`, swept over `v`.
#[test]
fn the_incidence_residual_is_free_of_t_c_and_v() {
    let lp = lp_map();
    let t_c = lp.tan_beta1_crit();
    let surge = SurgeLimiter::new(Spool::Lp, PHI_JAC);
    for v in [-0.15, 0.0, 0.07, 0.20] {
        let lim = IncidenceLimiter::from_phi(&lp, Spool::Lp, PHI_JAC, v);
        for phi in [0.60, 0.72, PHI_JAC, 0.95] {
            let m_i = t_c - (1.0 / phi - v);
            let long_way = lim.m_lim - m_i;
            let short_way = 1.0 / phi - 1.0 / surge.phi_lim;
            assert!((long_way - short_way).abs() < 1e-12 * 1.0_f64.max(short_way.abs()),
                    "{v} {phi} {long_way} {short_way}");
        }
    }
}

/// D1's mechanism: a positive multiplier cannot flip a slope's sign.
#[test]
fn the_multiplier_is_strictly_positive_so_signs_agree() {
    let s = scan();
    assert!(s.n > 0);
    for row in &s.rows {
        assert!(row.slope_phi * row.slope_inc > 0.0, "{row:?}");
    }
}

// ============================================================================== the reduce

/// `_phi_ref = "phi"` takes the PARENT's `_cap_fuel`. Python's last line (`co_consts is not None`)
/// is always true and is not copied; the dispatch it names is gated instead (module header).
#[test]
fn reduce_phi_is_rung_78_by_dispatch() {
    let m = rig();
    assert_eq!(m.fuel.inner.phi_ref.get(), PHI_REF_PHI, "the shipped coordinate is the default");
    assert!(!fn_addr_eq(R79_TRIPLE.cap_fuel, R78_TRIPLE.cap_fuel),
            "rung 79 declares its own `_cap_fuel`; if it did not there would be nothing to reduce");

    let b = bare();
    let fl = flight();
    let surge = surge_at(PHI_JAC);
    let call = |core: &ScheduledStatorCore, cell: &turbojet::three_loop::TripleHooks| {
        (cell.cap_fuel)(&core.fuel, &fl, SLACK.0, SLACK.1, SLACK.2, None, Some(&surge), None)
            .expect("the slack state solves")
    };
    reset_coord_counters();
    let via79 = call(&b, &R79_TRIPLE);
    let via78 = call(&b, &R78_TRIPLE);
    assert_eq!(via79.to_bits(), via78.to_bits(), "at `phi` the rung-79 cell is rung 78's body");
    assert_eq!(coord_counters(), CoordCounters::default(),
               "at `phi` the re-coordinated branch must not run at all");
    // ... and the control: the SAME call at incidence enters the branch, so the zero above is a
    // reading of the dispatch and not of a counter that never moves.
    b.fuel.inner.phi_ref.set(PHI_REF_INCIDENCE);
    let _ = call(&b, &R79_TRIPLE);
    assert_eq!(coord_counters().hits, 1, "the incidence branch did not run");
}

/// Half one: at incidence the SLOPE moves, by the DERIVED factor `1/phi_lim²`.
#[test]
fn reduce_direction_one_the_knob_is_wired() {
    let s = scan();
    assert!((s.predicted_ratio - 1.0).abs() > 0.1, "the test would be vacuous at phi_lim = 1");
    let e = s.ratio_err.expect("a non-empty scan");
    assert!(e < 1e-6, "{e}");
    for row in &s.rows {
        assert!((row.ratio - 1.0).abs() > 0.1, "the slope did not move: {row:?}");
    }
}

/// Half two: the SET POINT does not move — on the FORCED solve, not the plant, whose `0.0` is
/// `_surge_fuel`'s (spec § 5.1).
#[test]
fn reduce_direction_two_the_set_point_does_not() {
    let d = forced().d_forced.expect("a non-empty forced reading");
    assert!(d < 1e-12, "{d}");
    assert!(d > 0.0, "the forced solve returned bit-identical roots in both coordinates -- either \
                      the bypass is not bypassing, or the two residuals are the same object \
                      (spec § 5.2)");
}

/// NON-VACUITY for § 5.2: the bypass lands on the plant's own set point.
#[test]
fn the_forced_bypass_reproduces_the_shipped_solve() {
    let f = forced();
    assert_eq!(f.d_shipped, Some(0.0), "{:?}", f.d_shipped);
    assert_eq!(f.n_binding, f.n,
               "these points are not in the binding regime, so the bypass bypasses nothing");
}

// ============================================================================== §§ 1–3

/// D2: `Gi'(w*) / Gs'(w*) == 1/phi_lim²`, a factor with NO fitted content.
#[test]
fn slope_scales_by_the_derived_factor() {
    let e = scan().ratio_err.expect("a non-empty scan");
    assert!(e < 1e-6, "{e}");
}

/// **D3's PLUMBING, not D3.** Every incidence solve in this scan falls back to `_surge_fuel`
/// (spec § 1, the D3 row), so `dwdq_inc` IS the phi solve and this differences the fallback with
/// itself: it fails only if the two paths stop being wired to the same solve. The invariance
/// rests on the `h(w*)` algebra; D2 is the reading a wrong coordinate moves.
#[test]
fn sensitivity_is_coordinate_invariant() {
    let e = scan().dwdq_err.expect("a non-empty scan");
    assert!(e < 1e-9, "{e}");
}

/// The phi leg's `dw*/dq` is ALIVE — two dead readings would difference to zero and pass (rung
/// 77 § 8's `1.000e+00`). That is all it shows: both sides of D3 are the same fallback solve, so
/// a live reading compared with itself still differences to exactly zero.
#[test]
fn the_sensitivity_reading_is_not_trivially_zero() {
    for row in &scan().rows {
        assert!(row.dwdq_phi.abs() > 1e-4, "dw*/dq is dead; the invariance is vacuous: {row:?}");
    }
}

// ============================================================================== § 4

/// D1: a strictly positive multiplier preserves the root SET pointwise.
#[test]
fn root_counts_are_equal_in_both_coordinates() {
    let c = census();
    assert!(c.counts_equal, "{:?}", c.rows);
    let w = c.worst.expect("a non-empty census");
    assert!(w < 1e-9, "{w}");
}

/// THE BOUND ON RUNG 78: its gauge destroyed uniqueness; this one does not.
#[test]
fn uniqueness_survives_the_coordinate() {
    assert_eq!(census().n_roots, vec![1]);
}

// ============================================================================== § 5: the plant

/// P3: the leg that wins these points is the PHI leg, which this rung moves.
#[test]
fn the_phi_leg_wins_the_inner_min_everywhere() {
    let m = march();
    assert!(m.hits > 0, "the re-coordinated branch never ran");
    assert_eq!(m.binds, m.hits);
}

/// § 5.1's counter, SPLIT BY COORDINATE.
#[test]
fn the_knob_was_live_on_the_plant_at_least_sometimes() {
    let m = march();
    assert!(m.br_inc > 0,
            "the INCIDENCE residual was never bracketed on the plant -- § 5 is fully vacuous");
    assert!(m.fb_inc as i64 > m.br_inc,
            "the short-circuit is supposed to DOMINATE here; if it does not, § 5.1 is wrong");
}

/// § 5.3 — THE RUNG: the two sets are DISJOINT and they PARTITION the calls.
#[test]
fn the_complementarity_is_exact() {
    let m = march();
    assert_eq!(m.n_both, 0);
    assert_eq!(m.n_live + m.n_reach, m.n_log, "{} {} {}", m.n_live, m.n_reach, m.n_log);
    assert!(m.n_live > 0 && m.n_reach > 0,
            "a partition with an empty side is not a partition -- one of the two regimes never \
             occurred, and the disjointness is then trivially true");
}

/// P2 and P2n TOGETHER: `flips = 0` is recorded as a VACUOUS hold.
#[test]
fn the_min_never_flips_and_that_is_vacuous() {
    let m = march();
    assert_eq!(m.flips, 0);
    assert_eq!(m.vacuous, Some(true),
               "the vacuity guard did not fire -- if the gap has closed, P2 has become a real \
                measurement and spec § 8 needs rescoring");
    let d = m.d_max.expect("a non-empty log");
    let floor = if 1e-15 > d { 1e-15 } else { d };
    let g = m.gap_min.expect("armed rows exist");
    assert!(g > 1e3 * floor, "{g} {d}");
}

/// A counter of distinct FLOATS, not distinct STATES (`docs/rung79-gap-margin.md` § 4.1) —
/// kept as a plumbing check that the log is populated at all.
#[test]
fn the_gap_log_records_distinct_floats_not_distinct_states() {
    let m = march();
    assert!(m.n_distinct > 10, "{}", m.n_distinct);
    assert!(m.n_distinct_gap > 10, "{}", m.n_distinct_gap);
    assert!(m.n_log > 100, "{}", m.n_log);
}

/// P4, and the carried-knob check on the schedule.
#[test]
fn the_trajectory_and_the_schedule_do_not_move() {
    let m = march();
    assert!(m.same_len, "the two marches took different numbers of steps");
    assert!(m.worst < 1e-9, "{} {:?}", m.worst, m.where_);
    assert_eq!(m.sched_moved, 0.0);
}

// ============================================================================== the standstill

/// **THE CORRECTION, PINNED — THIS GATE BLESSES NOTHING.** `nu_lp`/`nu_hp` do not move by one
/// bit, so § 5's calls are all at ONE operating point, while the command ramps.
#[test]
fn the_march_stands_still_and_that_is_the_scope_of_section_5() {
    let (_, _, _, traj) = still();
    assert!(traj.len() > 300, "{}", traj.len());
    assert_eq!(spread(&traj, "nu_lp"), 0.0);
    assert_eq!(spread(&traj, "nu_hp"), 0.0);
    assert!(spread(&traj, "mf") < 1e-12, "{}", spread(&traj, "mf"));
    assert!(spread(&traj, "mf_sched") > 1.0, "{}", spread(&traj, "mf_sched"));
}

/// THE MECHANISM: the stator lifts the plant exactly onto the wall, so the phi leg's cap IS the
/// fuel already flowing. A limiter armed with ZERO initial margin has no transient.
#[test]
fn the_floor_is_armed_at_the_initial_condition() {
    let (_, _, _, traj) = still();
    assert!((traj[0].phi_lp - PHI_JAC).abs() < 1e-9, "{}", traj[0].phi_lp);
    assert!(spread(&traj, "phi_lp") < 1e-12, "phi never leaves the wall");
    assert!((key(&traj[0], "cap_fuel") - traj[0].mf).abs() < 1e-12 * traj[0].mf);
}

/// THE POSITIVE CONTROL: same rig, same code path, wall below the initial operating point.
/// `slow` in Python; the port has no tier (slice M step 5: measure, don't map).
#[test]
fn the_same_rig_does_march_when_the_wall_is_lowered() {
    let (_, _, _, traj) = march_at(0.75, MARGIN);
    assert!(spread(&traj, "nu_lp") > 1e-2, "{}", spread(&traj, "nu_lp"));
    let t4: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
    let (lo, hi) = t4.iter().fold((f64::INFINITY, f64::NEG_INFINITY),
                                   |(a, b), &x| (a.min(x), b.max(x)));
    assert!(hi - lo > 100.0, "{lo} {hi}");
}

/// § 9's predicted offset REFUTED at its own anchor: a standing plant is at steady state, and a
/// `margin = 0` schedule IS the steady-state fuel.
#[test]
fn gap_at_zero_margin_is_exactly_zero() {
    let (m0, m, _, traj) = still();
    let p = &traj[traj.len() / 2];
    let (w0, _, _) = a_cap(&m0, &m, p, 0.0);
    assert!((w0 - p.mf).abs() < 1e-11 * p.mf, "{w0} {}", p.mf);
}

/// **WHAT § 9's RESIDUAL IS**: `d ln(gap+1) / d ln(1+margin) = 1/(1 − c)` with `c` read at the
/// FIXED POINT — and read at the plant's fuel it must MISS (the non-vacuity control).
#[test]
fn the_gap_residual_is_rung_77s_stiffness() {
    let (m0, m, _, traj) = still();
    let p = &traj[traj.len() / 2];
    let d = 0.01;
    let (lo, _, _) = a_cap(&m0, &m, p, MARGIN - d);
    let (hi, _, _) = a_cap(&m0, &m, p, MARGIN + d);
    let (mid, c_cap, c_mf) = a_cap(&m0, &m, p, MARGIN);
    let slope = (hi.ln() - lo.ln()) / ((MARGIN + d).ln_1p() - (MARGIN - d).ln_1p());
    assert!((slope - 1.0 / (1.0 - c_cap)).abs() < 1e-4 * slope, "{slope} {c_cap}");
    let miss = (slope - 1.0 / (1.0 - c_mf)).abs() / slope;
    assert!(miss > 1e-3,
            "`c` read at the plant's fuel agrees just as well as `c` read at the fixed point -- \
             the identity is then insensitive to the evaluation point and measures nothing: {miss}");
    let g = march().gap_min.expect("armed rows exist");
    assert!(((mid / p.mf - 1.0) - g).abs() < 1e-9 * g, "{} {g}", mid / p.mf - 1.0);
}

// ============================================================================== the refusal

/// `engine.py:20910` — the two knobs re-write DIFFERENT legs' residuals. Needle: the message's
/// own phrase, not Python's `"REFUSED"`, which matches eight sites.
#[test]
fn incidence_times_a_live_rung_78_gauge_is_refused() {
    let m = rig();
    m.fuel.inner.phi_ref.set(PHI_REF_INCIDENCE);
    m.fuel.inner.gauge_k.set(2.0);
    let surge = surge_at(PHI_JAC);
    let e = (m.fuel.inner.triple_hooks.cap_fuel)(
        &m.fuel, &flight(), 1.0, 1.0, 0.01, None, Some(&surge), None)
        .expect_err("an incidence leg under a live gauge must refuse");
    assert!(e.0.contains("an INCIDENCE phi leg x a non-identity rung-78 GAUGE is REFUSED"), "{}", e.0);
}

/// THE SEVENTEENTH INSTANCE: `at_lever` must carry the COORDINATE and the class. Held STRICTER
/// than `isinstance` — the rebuilt tables are rung 79's AND not rung 78's.
#[test]
fn the_carried_knob_survives_at_lever() {
    let m = rig();
    m.fuel.inner.phi_ref.set(PHI_REF_INCIDENCE);
    let n = m.at_lever(&LeverArm {
        bleed_lim: m.fuel.inner.lever.lim,
        stator_lim: m.fuel.inner.stator.lim,
        ..Default::default()
    });
    let (nl, nt) = (&n.fuel.inner.lever_hooks, &n.fuel.inner.triple_hooks);
    assert!(fn_addr_eq(nl.at_lever, R79.at_lever) && fn_addr_eq(nt.cap_fuel, R79_TRIPLE.cap_fuel),
            "the rebuilt machine is not rung 79");
    assert!(!fn_addr_eq(nt.cap_fuel, R78_TRIPLE.cap_fuel)
                && !fn_addr_eq(nt.with_coord, R78_TRIPLE.with_coord),
            "the rebuilt machine carries rung 78's cells");
    assert_eq!(n.fuel.inner.phi_ref.get(), PHI_REF_INCIDENCE, "the coordinate was dropped");
    assert_eq!(n.fuel.inner.cap_law.get(), m.fuel.inner.cap_law.get());
    assert_eq!(n.fuel.inner.lag_coord.get(), m.fuel.inner.lag_coord.get());
}

/// § 5.5's first instrument failure. The flag is per-thread state, so "a freshly built machine
/// sees it" holds by construction (module header); the restore and the empty log are gated.
#[test]
fn the_probe_flag_is_written_on_the_class() {
    let ((armed, fresh), log) = with_probe(|| {
        let armed = coord_probe_armed();
        let _fresh = rig();
        (armed, coord_probe_armed())
    });
    assert!(armed, "the probe flag was not set");
    assert!(fresh, "building a machine inside the probe disarmed it");
    assert!(!coord_probe_armed(), "the flag was not restored");
    assert!(log.is_empty());
}

// ============================================================================== +1, +2, +3

/// **+1 — `engine.py:21285`.** A wall no fuel can reach from below (`phi_lim = 50`, so
/// `Gs > 0` everywhere): the BINDING walk finds no sign change. An `Abort`, in both coordinates.
#[test]
fn the_forced_binding_walk_refuses_when_no_root_lies_below() {
    let b = bare();
    let surge = surge_at(50.0);
    for coord in [PHI_REF_PHI, PHI_REF_INCIDENCE] {
        let e = forced_cap(&b.fuel, &flight(), SLACK.0, SLACK.1, SLACK.2, &surge, coord,
                           FORCED_GROW, FORCED_SHRINK, FORCED_N)
            .expect_err("no root below a wall at 50");
        assert!(e.0.contains("the forced bracket found no sign change below mf_sched"), "{}", e.0);
        assert!(e.0.contains(&format!("'{coord}' coordinate")), "{}", e.0);
    }
}

/// **+2 — `engine.py:21301`.** A wall no fuel can reach from above (`phi_lim = 1e-3`): the SLACK
/// walk ends without a positive reading — here on a failed evaluation, measured on PyPy at the
/// same step (`searched to 4.181503e-02`).
#[test]
fn the_forced_slack_walk_refuses_when_no_root_lies_above() {
    let b = bare();
    let surge = surge_at(1e-3);
    for coord in [PHI_REF_PHI, PHI_REF_INCIDENCE] {
        let e = forced_cap(&b.fuel, &flight(), SLACK.0, SLACK.1, SLACK.2, &surge, coord,
                           FORCED_GROW, FORCED_SHRINK, FORCED_N)
            .expect_err("no root above a wall at 1e-3");
        assert!(e.0.contains("the forced bracket found no sign change above mf_sched"), "{}", e.0);
        assert!(e.0.contains("searched to 4.181503e"), "a different walk length: {}", e.0);
    }
}

/// **+3 — `forced_cap`'s SLACK arm SUCCEEDS, in both coordinates, to Python's bit** (step 3's
/// J6, owed since the shipped reading binds at every point).
///
/// Measured on PyPy (`probe_py.py`): `Gs(0.02) = −0.0600`, so the walk goes UP; the two
/// coordinates' roots are ONE ULP apart (`…2fd2` in `phi`, `…2fd1` in incidence), and each is
/// `cap_free`'s slack arm to the bit — two instruction sequences, one arithmetic.
#[test]
fn the_forced_slack_walk_finds_pythons_root_in_both_coordinates() {
    let b = bare();
    let fl = flight();
    let surge = surge_at(PHI_JAC);
    let g0 = turbojet::state_coordinate::phi_residual(
        &b.fuel, &fl, SLACK.0, SLACK.1, &surge, Some(PHI_REF_PHI))(SLACK.2).expect("evaluates");
    assert!(g0 <= 0.0, "the state is not slack, so the slack arm is not what this reads: {g0}");
    for (coord, want) in [(PHI_REF_PHI, 0x3f9e5cb744aa2fd2_u64),
                          (PHI_REF_INCIDENCE, 0x3f9e5cb744aa2fd1_u64)] {
        let got = forced_cap(&b.fuel, &fl, SLACK.0, SLACK.1, SLACK.2, &surge, coord, FORCED_GROW,
                             FORCED_SHRINK, FORCED_N)
            .expect("the slack walk brackets a root");
        assert_eq!(got.to_bits(), want, "{coord}: got {got:e}, Python {:e}", f64::from_bits(want));
        let via_cap_free = phi_cap(&b.fuel, &fl, SLACK.0, SLACK.1, SLACK.2, &surge, Some(coord))
            .expect("cap_free's slack arm solves");
        assert_eq!(got.to_bits(), via_cap_free.to_bits(), "{coord}: forced vs cap_free");
    }
}
