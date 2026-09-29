//! RUNG 80 — **THE SPLIT WALL**: `docs/rung74-arrest-interval.md` § 8's seam.
//!
//! ```text
//! phi_lim = (1 + sm    )·phi_surge     the fuel leg
//! phi_air = (1 + sm_air)·phi_surge     the valve and the stator
//! ```
//!
//! **HEADLINE: a LEVEL split separates loops on the CONSTRAINT; it cannot separate the two that
//! share the ACTUATOR.** The split OPENS the four-loop cell in `demand`, but `min` still masks one
//! fuel-side leg with an EXACTLY zero column. And it CORRECTS rung 74: the arrest belongs to the
//! walls' COINCIDENCE, not to either floor.
//!
//! Ported from `tests/test_rung80.py`: **12 collected tests, none `slow`. This file has 16**; the
//! four extra are DECLARED below rather than absorbed.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, **4 ADDED**, 0 COLLAPSED
//!
//! | # | `tests/test_rung80.py` | here |
//! |---|---|---|
//! | 1 | `reduce_none_path_is_rung79_identical` | [`reduce_none_path_is_rung79_identical`] |
//! | 2 | `reduce_sm_air_equal_sm_is_bit_for_bit` | [`reduce_sm_air_equal_sm_is_bit_for_bit`] |
//! | 3 | `the_knob_is_loud` | [`the_knob_is_loud`] (`engine.py:21467`) |
//! | 4 | `shared_wall_control_reproduces_rung74_bracket` | [`shared_wall_control_reproduces_rung74_bracket`] |
//! | 5 | `neither_split_arm_arrests` | [`neither_split_arm_arrests`] |
//! | 6 | `the_lift_is_present_so_the_null_is_not_a_dormant_knob` | [`the_lift_is_present_so_the_null_is_not_a_dormant_knob`] |
//! | 7 | `clip_positive_control_fires` | [`clip_positive_control_fires`] |
//! | 8 | `split_opens_the_demand_four_loop_cell` | [`split_opens_the_demand_four_loop_cell`] |
//! | 9 | `fuel_leg_stays_live_and_erodes_monotonically` | [`fuel_leg_stays_live_and_erodes_monotonically`] |
//! | 10 | `one_authority_per_point_and_the_mask_is_exactly_zero` | [`one_authority_per_point_and_the_mask_is_exactly_zero`] |
//! | 11 | `the_zero_has_a_positive_control_and_an_empty_skip_count` | [`the_zero_has_a_positive_control_and_an_empty_skip_count`] |
//! | 12 | `demand_shared_wall_is_vacuous_and_says_so` | [`demand_shared_wall_is_vacuous_and_says_so`] |
//! | **+1** | **— none —** | [`the_split_control_refuses_a_legal_input_two_ulp_wide`] (`engine.py:21486`, P5) |
//! | **+2** | **— none —** | [`a_valveless_row_refuses_after_the_three_reads`] (`engine.py:21551`) |
//! | **+3** | **— none —** | [`the_arrest_walls_must_bracket_below`] (`engine.py:21647`, first clause) |
//! | **+4** | **— none —** | [`the_arrest_walls_must_bracket_above`] (`engine.py:21647`, second clause) |
//!
//! # WHERE THE PORT DIFFERS FROM A LINE-BY-LINE COPY
//!
//! * **#1's `repr(a) == repr(b)` is `Debug` equality** of the two [`CoordScan`]s. Rust's `Debug`
//!   of an `f64` is its shortest round-trip form, so equal strings are equal bits, `-0.0` included.
//! * **#2 also asserts the march ran `demand`** (every point a `Demand` point, and the built rig's
//!   `lag_coord`). Declared, and owed: `slice_af_dispatch.rs` found that a port writing the
//!   coordinate through rung 79's setter would march the `clip` arm here, and Python's
//!   `len == 341` passes on that too.
//! * **#3 also reads the caller's `sm_air` back after the refusal** — `_with_air`'s `finally`.
//! * **#3 also asserts the built rig is a RUNG-80 machine** (slice AJ step 1, plan § 5.34 (i)).
//!   Python's catcher is INCIDENTAL: `tests/test_rung80.py:126` reads the walls as
//!   `rig._walls_of(rig, surge)`, a method looked up ON THE REBUILT RIG, so a rig of rung 79's
//!   class dies there with `AttributeError` — which is how Python catches the deletion of rung
//!   80's `at_lever`. [`walls_of`] is a free function here and accepts any core, so without this
//!   assert the port passed that deletion 16/16 (slice AI step 7). The rig's TABLE is this crate's
//!   only spelling of its class. **The pointer compared is `shared_rig`, not `at_lever`:** the
//!   deletion re-fills `R80.at_lever` from `..R79`, so an `at_lever` compare would test the rig
//!   against the very function that built it and pass; a rung-79 rig carries `R79_TRIPLE`, whose
//!   `shared_rig` differs from [`R80_TRIPLE`]'s, and the deletion cannot move that reference.
//! * `pytest.approx(x, rel=r)` with no `abs` is `|a − x| ≤ r·|x|`, and is written so.
//!
//! # THE FOUR ADDED GATES — rung 80's other three refusals, every one written from the source
//!
//! Plan § 5.33 (vi) drove all four in Python; the suite asserts only `:21467` (#3).
//!
//! * **+1 is P5.** `engine.py:21486` is written to catch a WIRING bug — a split that never reached
//!   the plant — and it also fires on a LEGAL input: `1 + sm_air` carries four times `sm`'s ulp, so
//!   an `sm_air` one or two ulp above `sm` rebuilds the SAME float as the fuel wall. Measured on
//!   PyPy (`W:\temp\claude\slice-ai-step5\probe_py.py`) at `sm = 0.4545454545454546`: `+0` builds
//!   (the `sm_air == sm` exemption), `+1` and `+2` refuse, `+3` builds with the airflow wall at
//!   `0.8000000000000002`. The `+0` arm brackets the band from below.
//! * **+2** drives `_split_row` on a valve-less machine, and pins WHERE the assert sits: after the
//!   three reads. A point without `b` must die on its `KeyError`-shaped message, not the valve one.
//! * **+3 / +4** break each half of `split_arrest`'s `and` separately, so a dropped clause cannot
//!   survive behind the other.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;
use std::sync::OnceLock;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::PointExtra;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::SharedRigArm;
use turbojet::split_wall::{
    build_split_wall_cascade, split_arrest, split_liveness, split_march, split_row, walls_of,
    AirScope, SplitArrest, SplitGains, SplitLiveness, SplitRow, ARREST_WALLS, GAINS_EVERY,
    LIVENESS_COORDS, LIVENESS_PHI_AIRS, R80_TRIPLE, ROW_TOL,
};
use turbojet::state_coordinate::{build_state_coordinate_cascade, coord_scan, CoordScan, COORD_AT_DQ};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung80.py`'s module constants, verbatim.

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
/// Strictly inside rung 74's window — both edges READ OFF `docs/rung74-arrest-interval.md`.
const PHI_FUEL: f64 = 0.75;
const FREE_PHI0: f64 = 0.7731162133;

// ------------------------------------------------ the readers' unspelled defaults (`engine.py`)
//
// `split_liveness` (`:21583`), `split_arrest` (`:21623`), `split_gains` (`:21716`) and rung
// 79's `coord_scan` (`:21049`): `taus = (0.05,)*4`, `inc = False`, `r = 0.5`, `s_settle = 1.2`,
// `ds = 0.005`; `split_arrest`'s `phi_air_hi = 0.80`, `coord = "demand"`; `coord_scan`'s
// `phi_lim = 0.80`, `margin = 0.10`, `every = 8`.

const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const R: f64 = 0.5;
const S_SETTLE: f64 = 1.2;
const DS: f64 = 0.005;
const PHI_AIR_HI: f64 = 0.80;
/// The two gain fixtures' walls.
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

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// `_rig`'s margin — the SHARED 0.80 wall.
fn rig_sm() -> f64 {
    0.80 / FLOOR - 1.0
}

fn suite_arm() -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, rig_sm(), Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, rig_sm(), Some(TAU_S))),
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

/// Python's `_rig(design)` — rung 80.
fn rig() -> ScheduledStatorCore {
    let m = full_of(build_split_wall_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &suite_arm()));
    set_knobs(&m);
    m
}

/// Python's `_rig(design, StateCoordinateTransient)` — rung 79, same arms and knobs.
fn rig79() -> ScheduledStatorCore {
    let m = full_of(build_state_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &suite_arm()));
    set_knobs(&m);
    m
}

/// `m._shared_rig(sm, TAU, TAU_S, V_MAX, TT4_MAX)` — through the table, as Python dispatches it.
fn shared_arm(sm: f64) -> SharedRigArm {
    SharedRigArm { sm, tau: TAU, tau_s: TAU_S, v_max: V_MAX, tt4_max: TT4_MAX,
                   ..SharedRigArm::default() }
}

/// `pytest.approx(x, rel=r)` with no `abs`.
fn approx(a: f64, x: f64, rel: f64) -> bool {
    (a - x).abs() <= rel * x.abs()
}

/// Catch a panic and hand back its message. **No hook swap** — `take_hook`/`set_hook` are
/// process-wide, and in a parallel run one gate could restore a sibling's silent hook. The
/// harness already hides a passing test's panic output.
fn caught<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|e| {
        e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    })
}

fn panic_message<T>(f: impl FnOnce() -> T) -> Option<String> {
    caught(f).err()
}

// ============================================================================== the fixtures
//
// Python's module-scoped fixtures, each on a FRESH rig. The readings are plain data, so each is
// computed once per binary.

fn liveness() -> &'static SplitLiveness {
    static S: OnceLock<SplitLiveness> = OnceLock::new();
    S.get_or_init(|| split_liveness(&rig(), &flight(), LO, HI, TT4_MAX, PHI_FUEL,
                                    &LIVENESS_PHI_AIRS, &LIVENESS_COORDS, TAUS, false, R, S_SETTLE,
                                    DS, V_MAX))
}

fn arrest() -> &'static SplitArrest {
    static S: OnceLock<SplitArrest> = OnceLock::new();
    S.get_or_init(|| split_arrest(&rig(), &flight(), LO, HI, TT4_MAX, &ARREST_WALLS, PHI_FUEL,
                                  PHI_AIR_HI, "demand", TAUS, false, R, S_SETTLE, DS, V_MAX))
}

fn gains_at(coord: &'static str) -> SplitGains {
    turbojet::split_wall::split_gains(&rig(), &flight(), LO, HI, TT4_MAX, PHI_FUEL, &GAIN_AIRS,
                                      coord, TAUS, false, R, S_SETTLE, DS, V_MAX, GAINS_EVERY)
        .expect("no Abort on this rig")
}

fn gains() -> &'static SplitGains {
    static S: OnceLock<SplitGains> = OnceLock::new();
    S.get_or_init(|| gains_at("clip"))
}

fn gains_demand() -> &'static SplitGains {
    static S: OnceLock<SplitGains> = OnceLock::new();
    S.get_or_init(|| gains_at("demand"))
}

fn rows_of<'a>(l: &'a SplitLiveness, coord: &str) -> Vec<&'a SplitRow> {
    l.rows.iter().filter(|x| x.coord == coord).collect()
}

// ============================================================================== § 9.1: the reduce

/// `_sm_air = None` dispatches to rung 79 with nothing recomputed — compared through a shipped
/// rung-79 reader, not through this rung's own.
#[test]
fn reduce_none_path_is_rung79_identical() {
    let run = |m: &ScheduledStatorCore| -> CoordScan {
        coord_scan(m, &flight(), LO, HI, TT4_MAX, 0.80, 0.10, TAUS, false, R, S_SETTLE, DS, V_MAX,
                   COORD_AT_DQ, 8)
    };
    let a = run(&rig79());
    let b = run(&rig());
    assert_eq!(format!("{a:?}"), format!("{b:?}"), "rung 80 moved a rung-79 float on the unarmed path");
}

/// `sm_air == sm` rebuilds the SAME floors from the SAME factory, so it equals the `None` path to
/// the last bit. Note `sm` here is the FUEL wall's (0.75), not the rig's.
#[test]
fn reduce_sm_air_equal_sm_is_bit_for_bit() {
    let m = rig();
    let sm = PHI_FUEL / FLOOR - 1.0;
    let march = || turbojet::demand_coordinate::coord_march(
        &m, &flight(), LO, HI, TT4_MAX, sm, TAUS, 0.5, 1.2, 0.005, V_MAX, false, "demand",
        "sched", None);
    let (m0, _, _, r0) = march();
    let (m1, _, _, r1) = {
        let _air = AirScope::set(&m, Some(sm));
        march()
    };
    assert_eq!((r0.len(), r1.len()), (341, 341));
    for (p, q) in r0.iter().zip(r1.iter()) {
        let (PointExtra::Demand { b: bp, v: vp, .. }, PointExtra::Demand { b: bq, v: vq, .. }) =
            (p.extra, q.extra) else {
            panic!("DECLARED: the march did not run `demand` at s = {} -- a clip march also \
                    returns 341 points (slice_af_dispatch.rs)", p.s);
        };
        assert_eq!(p.phi_lp.to_bits(), q.phi_lp.to_bits(), "phi_lp moved at s = {}", p.s);
        assert_eq!(p.tt4.to_bits(), q.tt4.to_bits(), "Tt4 moved at s = {}", p.s);
        assert_eq!(bp.to_bits(), bq.to_bits(), "b moved at s = {}", p.s);
        assert_eq!(vp.to_bits(), vq.to_bits(), "v moved at s = {}", p.s);
        assert_eq!(p.mf.to_bits(), q.mf.to_bits(), "mf moved at s = {}", p.s);
    }
    assert_eq!((m0.fuel.inner.lag_coord.get(), m1.fuel.inner.lag_coord.get()),
               ("demand", "demand"), "DECLARED: the built rigs must carry the coordinate marched");
    assert_eq!(m.fuel.inner.sm_air.get(), None, "the scope left the knob on the caller's rig");
}

/// A split that fails to reach the plant would make every reader report *the levers did
/// nothing*, so the walls are read BACK off the built limiters, and a lower airflow wall dies by
/// name (`engine.py:21467`).
#[test]
fn the_knob_is_loud() {
    let m = rig();
    let sm = PHI_FUEL / FLOOR - 1.0;
    let (built, surge, _) = {
        let _air = AirScope::set(&m, Some(0.80 / FLOOR - 1.0));
        (m.fuel.inner.triple_hooks.shared_rig)(&m, &shared_arm(sm))
    };
    assert!(fn_addr_eq(built.triple_hooks().shared_rig, R80_TRIPLE.shared_rig),
            "DECLARED: the built rig is not a RUNG-80 machine -- Python dies here looking up \
             `_walls_of` on it (tests/test_rung80.py:126); was rung 80's `at_lever` dropped?");
    let w = walls_of(&built, surge.as_ref());
    let (lim, air) = (w.phi_lim.expect("a fuel wall"), w.phi_air.expect("an airflow wall"));
    assert!(approx(lim, PHI_FUEL, 1e-12), "{lim}");
    assert!(approx(air, 0.80, 1e-12), "{air}");
    let (valve, stator) = (w.phi_valve.expect("a valve"), w.phi_stator.expect("a stator"));
    assert!(approx(valve, stator, 1e-12), "both airflow legs must sit on ONE airflow wall");

    let msg = panic_message(|| {
        let _air = AirScope::set(&m, Some(0.70 / FLOOR - 1.0));
        (m.fuel.inner.triple_hooks.shared_rig)(&m, &shared_arm(sm))
    }).expect("an airflow wall below the fuel wall must refuse");
    assert!(msg.contains("AIRFLOW wall sits AT or ABOVE"), "{msg}");
    assert_eq!(m.fuel.inner.sm_air.get(), None,
               "DECLARED: `_with_air`'s `finally` -- the refusal left the knob on the caller's rig");
}

// ============================================================================== § 9.2: the control

/// THE CONTROL ARM FIXES THE RIG: rung 74's own bracket reappears, and the free operating point
/// lies INSIDE it, so the edge is derived rather than fitted.
#[test]
fn shared_wall_control_reproduces_rung74_bracket() {
    let a = arrest();
    let sh = a.arm("shared");
    assert_eq!(a.control_bracket, (Some(0.7731), Some(0.7732)));
    assert!(sh.monotone, "marched {:?} interleaves arrested {:?}", sh.marched, sh.arrested);
    let (lm, fa) = (sh.last_march.expect("a march"), sh.first_arrest.expect("an arrest"));
    assert!(lm < FREE_PHI0 && FREE_PHI0 < fa,
            "the free operating point must lie INSIDE the bracket -- that is what makes the edge \
             derived rather than fitted");
}

// ============================================================================== § 9.3: the coincidence

/// CORRECTS rung 74 § 2.2: split the walls in EITHER direction and the arrest is gone.
#[test]
fn neither_split_arm_arrests() {
    let a = arrest();
    assert!(a.owner.is_empty(), "a split arm arrested: {:?}", a.owner);
    for arm in ["air", "fuel"] {
        let d = a.arm(arm);
        assert!(d.arrested.is_empty(), "{arm} arrested at {:?}", d.arrested);
        assert!(d.rows.iter().all(|(_, x)| x.max_tt4 > LO * 1.15),
                "{arm} did not accelerate on some wall");
    }
}

/// The lift must be HAPPENING in the split arms, or "no arrest" would just mean "no floor acted".
#[test]
fn the_lift_is_present_so_the_null_is_not_a_dormant_knob() {
    let a = arrest();
    for arm in ["air", "fuel"] {
        let lifted: Vec<&(f64, SplitRow)> = a.arm(arm).rows.iter()
            .filter(|(_, x)| x.phi_air.is_some_and(|pa| pa > FREE_PHI0))
            .collect();
        assert!(!lifted.is_empty(), "{arm} never put a floor above the free operating point");
        for (wall, x) in lifted {
            let pa = x.phi_air.expect("filtered on it");
            assert!(approx(x.phi0, pa, 1e-9),
                    "{arm}: phi(0) was not lifted ONTO the airflow wall at {wall}");
            assert!(x.b0_frac > 0.0, "{arm}: the valve never opened at {wall}");
        }
    }
}

// ============================================================================== § 9.4: the cell opens

/// `clip` HAS four-loop cells at the shared wall; zero motion there means the READER is broken.
#[test]
fn clip_positive_control_fires() {
    let l = liveness();
    assert!(l.control_ok);
    let clip = rows_of(l, "clip");
    assert!(!clip.is_empty() && clip.iter().all(|x| x.valve_moved > 0 && x.stator_moved > 0));
}

/// THE SEAM'S OWN OBJECT, at last non-empty — and the shared-wall row is the baseline.
#[test]
fn split_opens_the_demand_four_loop_cell() {
    let dem = rows_of(liveness(), "demand");
    let shared: Vec<&&SplitRow> = dem.iter().filter(|x| x.phi_air.is_none()).collect();
    let split: Vec<&&SplitRow> = dem.iter().filter(|x| x.phi_air.is_some()).collect();
    assert!(shared.len() == 1 && !split.is_empty());
    assert!(shared[0].n_riding4 == 0 && shared[0].valve_moved == 0,
            "rung 74's result must reproduce: a shared wall leaves the levers inert");
    for x in split {
        assert!(x.riding4_valid, "n_riding4 is meaningless on an arrested plant (§ 8)");
        assert!(x.n_riding4 > 0, "no four-loop point at phi_air = {:?}", x.phi_air);
        assert!(x.valve_moved > 0 && x.stator_moved > 0);
    }
}

/// § 1.1's DERIVATION: the fuel leg's cut is evaluated at the SCHEDULED fuel, so a lever that
/// raises the ACHIEVED phi erodes it without extinguishing it. Anchor P1 predicted extinction.
#[test]
fn fuel_leg_stays_live_and_erodes_monotonically() {
    let mut dem = rows_of(liveness(), "demand");
    // Python's `sorted(key=lambda x: x["phi_air"] or 0.0)` -- a STABLE sort.
    dem.sort_by(|a, b| a.phi_air.unwrap_or(0.0).partial_cmp(&b.phi_air.unwrap_or(0.0))
        .expect("no NaN wall"));
    let cuts: Vec<usize> = dem.iter().map(|x| x.n_cut_fuel).collect();
    assert!(cuts.iter().all(|&c| c > 0), "the fuel leg went dormant: {cuts:?}");
    let mut desc = cuts.clone();
    desc.sort_by(|a, b| b.cmp(a));
    assert!(cuts == desc && cuts[0] > cuts[cuts.len() - 1],
            "the erosion is not monotone in phi_air: {cuts:?}");
}

// ============================================================================== § 9.5: the mask

/// RUNG 72, UNMOVED BY A LEVEL SPLIT: exactly one fuel-side leg holds the actuator, and the masked
/// one's column is EXACTLY zero.
#[test]
fn one_authority_per_point_and_the_mask_is_exactly_zero() {
    for out in [gains(), gains_demand()] {
        assert!(!out.ever_two_authorities);
        assert_eq!(out.max_mask_leak, Some(0.0),
                   "the masked leg leaked {:?} into the plant", out.max_mask_leak);
        for a in &out.arms {
            for c in &a.cells {
                assert_ne!(c.masked, c.authority);
            }
        }
    }
}

/// AN EXACT ZERO CAN MEAN "NOTHING WAS COMPUTED". Two discriminators: the `clip` cell where the
/// GOVERNOR is masked returns `|cyclic| ≈ 1` on the same path, and every split arm differenced all
/// of its points.
#[test]
fn the_zero_has_a_positive_control_and_an_empty_skip_count() {
    let ctrl = gains().control_nonzero.expect(
        "the instrument must produce a NON-zero on the same path, or the zeros below are \
         unfalsifiable");
    assert!(approx(ctrl, 1.0, 1e-6), "{ctrl}");
    for out in [gains(), gains_demand()] {
        for a in &out.arms {
            if a.phi_air.is_none() {
                continue; // the baseline is the control, not the subject
            }
            assert!(a.n_interior > 0);
            assert_eq!(a.skipped, (0, 0), "points were dropped at phi_air = {:?}", a.phi_air);
            assert_eq!(a.max_cyc, Some(0.0));
        }
    }
}

/// The `demand` baseline HAS no interior point, and the reader must REPORT it.
#[test]
fn demand_shared_wall_is_vacuous_and_says_so() {
    let g = gains_demand();
    let base = g.arms.iter().find(|a| a.phi_air.is_none()).expect("a shared-wall arm");
    assert!(base.n_riding == 0 && base.n_interior == 0);
    assert!(g.vacuous, "a zero-point arm must set the vacuity flag");
}

// ============================================================================== +1 … +4

/// **+1 — P5, `engine.py:21486`'s ulp band.** At the fingerprint's `sm = 0.4545454545454546`:
/// `+0` builds (exempt), `+1` and `+2` refuse, `+3` builds one ulp above the fuel wall — PyPy's
/// bits, each read back off the built machine.
#[test]
fn the_split_control_refuses_a_legal_input_two_ulp_wide() {
    let m = rig();
    let sm = rig_sm();
    assert_eq!(sm.to_bits(), 0x3fdd1745d1745d18, "not the fingerprint's margin");
    const WALL: u64 = 0x3fe999999999999a; // 0.8
    for (k, want_air) in [(0u64, Some(WALL)), (1, None), (2, None), (3, Some(WALL + 1))] {
        let sm_air = f64::from_bits(sm.to_bits() + k);
        let got = caught(|| {
            let _air = AirScope::set(&m, Some(sm_air));
            let (built, surge, _) = (m.fuel.inner.triple_hooks.shared_rig)(&m, &shared_arm(sm));
            let w = walls_of(&built, surge.as_ref());
            (w.phi_lim.expect("a fuel wall").to_bits(), w.phi_air.expect("an airflow wall").to_bits())
        });
        match (want_air, got) {
            (Some(_), Err(msg)) => panic!("+{k} ulp must build, and refused: {msg}"),
            (None, Ok(_)) => panic!("+{k} ulp must refuse, and built"),
            (None, Err(msg)) => assert!(msg.contains("the split did not reach the plant"),
                                        "+{k} ulp refused with another message: {msg}"),
            (Some(want), Ok((lim, air))) => {
                assert_eq!(lim, WALL, "+{k}: the fuel wall");
                assert_eq!(air, want, "+{k}: the airflow wall");
            }
        }
        assert_eq!(m.fuel.inner.sm_air.get(), None, "+{k}: the scope left the knob behind");
    }
}

/// **+2 — `engine.py:21551`.** `_split_row` on a valve-less machine refuses — AFTER the three reads
/// Python makes first: a point with no `b` dies on its `KeyError`-shaped message instead.
#[test]
fn a_valveless_row_refuses_after_the_three_reads() {
    let (m, surge, _, traj) = split_march(&rig(), &flight(), LO, HI, TT4_MAX, PHI_FUEL, None,
                                          "demand", TAUS, R, S_SETTLE, DS, V_MAX, false);
    // the control: the valved machine the march built returns a row
    let row = split_row(&m, surge.as_ref(), &traj, LO, PHI_FUEL, None, "demand", ROW_TOL);
    assert_eq!(row.npts, traj.len());

    let valveless = full_of(build_split_wall_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
        &LeverArm { bleed_lim: None, ..suite_arm() }));
    assert!(valveless.fuel.inner.lever.lim.is_none());
    let msg = panic_message(|| split_row(&valveless, surge.as_ref(), &traj, LO, PHI_FUEL, None,
                                         "demand", ROW_TOL))
        .expect("a valve-less row must refuse");
    assert!(msg.contains("counts points with the valve STRICTLY INTERIOR"), "{msg}");

    let mut bare = traj[0];
    bare.extra = PointExtra::None;
    let msg = panic_message(|| split_row(&valveless, surge.as_ref(), &[bare], LO, PHI_FUEL, None,
                                         "demand", ROW_TOL))
        .expect("a point with no `b` must refuse");
    assert!(msg.contains("carries no `b`/`v`/`required_*`"),
            "the valve assert fired BEFORE the reads Python makes first: {msg}");
}

/// **+3 — `engine.py:21647`, first clause**: `phi_lim_lo` not below every swept wall. Refused
/// before any march.
#[test]
fn the_arrest_walls_must_bracket_below() {
    let msg = panic_message(|| split_arrest(&rig(), &flight(), LO, HI, TT4_MAX, &ARREST_WALLS, 0.78,
                                            PHI_AIR_HI, "demand", TAUS, false, R, S_SETTLE, DS,
                                            V_MAX))
        .expect("phi_lim_lo = 0.78 is not below 0.77");
    assert!(msg.contains("the two FIXED walls must bracket the swept ones"), "{msg}");
}

/// **+4 — `engine.py:21647`, second clause**: `phi_air_hi` not above every swept wall.
#[test]
fn the_arrest_walls_must_bracket_above() {
    let msg = panic_message(|| split_arrest(&rig(), &flight(), LO, HI, TT4_MAX, &ARREST_WALLS,
                                            PHI_FUEL, 0.78, "demand", TAUS, false, R, S_SETTLE, DS,
                                            V_MAX))
        .expect("phi_air_hi = 0.78 is not above 0.78");
    assert!(msg.contains("the two FIXED walls must bracket the swept ones"), "{msg}");
}
