//! SLICE AG step 1 — **EIGHT RE-AIMED POINTERS ACROSS TWO RUNGS, ZERO NEW TABLE FIELDS, AND THE
//! FIRST STEP 1 SINCE SLICE Z WITH NO WIDTH TRIPWIRE UNDER IT.**
//!
//! # WHY THIS FILE IS SHAPED THE WAY IT IS
//!
//! § 5.31 (ii)'s census is `0 ADD`. Every step 1 from slice AA to AF widened
//! [`TripleHooks`](turbojet::three_loop::TripleHooks), so a forgotten cell was an `E0063` before
//! any gate ran. **Here a forgotten re-aim compiles.** It silently runs the rung-74 parent, and
//! both parent bodies return exactly this slice's own reduce-arm answer — `windup_tau` gives
//! `None` (no device) and `sensed_cap` gives `Ok(None)` (take the shipped solve) — so **every
//! reduce gate in the crate would go on passing** while the `track` and `sensed` arms silently
//! marched their own parents.
//!
//! The instrument that works with no bodies yet is FUNCTION-POINTER IDENTITY IN BOTH DIRECTIONS,
//! and [`the_slice_re_aims_eight_pointers_and_adds_none`] is it: every swapped slot must DIFFER
//! from the parent's and every inherited slot must EQUAL it, so a dropped re-aim and a stray one
//! are both visible. [`a_missed_re_aim_is_silent_under_sched_and_loud_under_applied`] then proves
//! that gate is load-bearing rather than decorative, by building the defect and measuring what a
//! full march can and cannot see — and the answer SPLITS BY REFERENCE, which is sharper than the
//! blanket silence the gate was written expecting.
//!
//! # THE TWO CELLS LAND HERE IN FULL, AND THE STEP BOUNDARY IS RE-CUT ON PURPOSE
//!
//! § 5.31 (vi) puts `_windup_tau` and `_sensed_cap` at steps 2 and 4 and *both refusal sets* at
//! step 1 — jointly impossible, because three of this slice's nine shipped refusals ARE those two
//! bodies. The re-cut is recorded in both module headers rather than made quietly.
//!
//! # WHAT THIS FILE DELIBERATELY DOES NOT GATE
//!
//! **The `track` march's own trajectory**, beyond the equality above: rung 75's device turns on
//! three sites slice AF ported dead, and comparing what they produce is step 2's job with the
//! rest of the RHS.
//!
//! **The `sensed` cap's VALUE against the solve.** § 5.31 (i) measured the gap at 2.32 % at
//! `phi_lim = 0.80` and the min-select discarding it at 1 366 of 1 366 calls, so the value break
//! lives on the OTHER arm and needs a real accel schedule built off the plant's own equilibria —
//! `accel_for`, which lands at step 4. What this file gates is that the two branches ANSWER
//! differently through the table.
//!
//! **NOTHING HERE READS A GOLDEN.** Every assertion is a panic, a same-run difference, or a
//! compile-time property.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;

use turbojet::anti_windup::{
    build_anti_windup_cascade, R75, R75_FUEL, R75_STATOR, R75_TRIPLE, R75_TWO, WINDUP_LAWS_DECLARED,
    WINDUP_LAW_NONE, WINDUP_LAW_TRACK, WINDUP_TAU_GRID_FLOOR,
};
use turbojet::applied_reference::REF_LAW_APPLIED;
use turbojet::bleed_transient::{LeverArm, LeverArming};
use turbojet::demand_coordinate::{
    build_demand_coordinate_cascade, IC_CAP_DECLARED, LAG_COORD_CLIP, LAG_COORD_DEMAND,
    LAG_COORD_LATCHED, R74, R74_FUEL, R74_STATOR, R74_TRIPLE, R74_TWO,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::sensed_cap::{
    build_sensed_cap_cascade, CAP_LAWS_DECLARED, CAP_LAW_SENSED, CAP_LAW_SOLVE, R76, R76_FUEL,
    R76_STATOR, R76_TRIPLE, R76_TWO,
};
use turbojet::shared_actuator::{
    build_shared_actuator_cascade, SharedRigArm, REF_LAW_DEFAULT, SHARE_LAW_DEFAULT,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::TripleHooks;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung75.py` and `tests/test_rung76.py`'s module constants. This slice adds ONE
// number of its own — the tracking clock — and it is the shipped sweep's own slow end, never a
// value invented here.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
/// `test_rung76.py`'s `PHI_JAC` — the arm whose march § 5.31 (i) measured BIT-IDENTICAL under the
/// two cap laws.
const PHI: f64 = 0.80;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
const TT4_MAX: f64 = 1200.0;
/// `test_rung75.py`'s slow end of the `tau_ts` sweep — a declared value from the shipped suite,
/// not one chosen here, and comfortably above [`WINDUP_TAU_GRID_FLOOR`].
const TAU_T: f64 = 0.05;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

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

fn valve() -> BleedLimiter { BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU)) }
fn surge() -> SurgeLimiter {
    SurgeLimiter::from_margin(&lp_map(), turbojet::two_spool::Spool::Lp, SM)
}
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// Python's `AntiWindupTransient(design, …)` — rung 75.
fn windup(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_anti_windup_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Python's `SensedCapTransient(design, …)` — rung 76.
fn sensed(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_sensed_cap_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Rung 74 — the immediate parent, and the control at every step below.
fn demand(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Rung 72 — the grandparent, whose `_ref_law` default is the OTHER one.
fn shared(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_shared_actuator_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

fn valve_arm() -> LeverArm { LeverArm { bleed_lim: Some(valve()), ..Default::default() } }

/// The panic message a closure produces, or `""` if it did not panic. Slice AB–AF's helper
/// verbatim, and for its reason: `assert!(panics(…))` is satisfied by an unrelated bug as readily
/// as by the refusal it names.
fn message_of<F: FnOnce()>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    match out {
        Ok(()) => String::new(),
        Err(e) => e
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    }
}

/// A march with **neither the governor clock nor the fuel leg armed** — the arming rung 72's body
/// early-returns on, which is what makes it the instrument for *above the entry test*.
fn bare_march(m: &ScheduledStatorCore) {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: None, tt4_max: None };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg, &MarchScope::DEFAULT);
}

/// A FULLY ARMED MARCH — the fuel leg, its lag, the governor clock and a set point, reached
/// through the real march entry. Slice AF's helper, and its recorded reason carries: the ramp
/// supplies the schedule the plant is actually matched to, so a clean-return control measures a
/// march that RAN rather than one that aborted for an unrelated reason.
fn armed_march(m: &ScheduledStatorCore, tt4_max: Option<f64>, tau_gov: Option<f64>)
 -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: Some(Floor::Phi(surge())),
                          tt4_max };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg,
                          &MarchScope { lag: Some(lag()), tau_gov, ..MarchScope::DEFAULT }).0
}

/// A flat `Wf/pt3` schedule, built by hand rather than derived from a march: every gate in this
/// file is about DISPATCH, and a derived schedule would make it depend on the running line too.
/// The plant's own schedule is `accel_for`, and it lands at step 4.
fn flat_schedule() -> AccelSchedule {
    AccelSchedule { margin: 0.0, n_h: vec![0.5, 1.5], kappa: vec![1.0, 1.0] }
}

/// **THE DEFECT THIS SLICE HAS NO COMPILER TRIPWIRE FOR** — rung 75's table with `windup_tau` left
/// pointing at rung 74's body. Used by
/// [`a_missed_re_aim_is_silent_under_sched_and_loud_under_applied`].
const R75_TRIPLE_MISSED: TripleHooks = TripleHooks {
    windup_tau: R74_TRIPLE.windup_tau,
    ..R75_TRIPLE
};

/// Rebuild a rung-75 machine with an injected third-loop table — slice AF's `with_triple`, aimed
/// at this slice's tables.
fn with_triple(arm: &LeverArm, triple: &'static TripleHooks) -> ScheduledStatorCore {
    let base = windup(arm);
    let c = full_of(ScheduledStatorTransient::with_ref_tables(
        base.design_engine().clone(), *base.flight_design(), base.mdot_design(),
        Some(base.arming().map_lp_design), Some(base.arming().map_hp_design), base.rho(),
        arm.stator, &R75_TWO, &R75_STATOR, &R75_FUEL, &R75,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        triple, arm.stator_lim, arm.stator_inc));
    c.fuel.inner.ref_law.set(REF_LAW_APPLIED);
    c
}

/// The two MASKED-LEG STATES and the burnt fuel's temperature, as bits.
///
/// **THE STATES ARE WHAT RUNG 75 MOVES AND `Tt4` IS NOT**, which is the rung's own headline: the
/// device disarms itself on the leg that holds the actuator, so everything it does it does to the
/// masked leg — and `min`-select then hides that from the plant. A gate that compared only the
/// output would read an exact zero and conclude the device was inert.
fn state_of(p: &FuelPoint) -> (u64, u64, u64) {
    match p.extra {
        PointExtra::Demand { w_fuel, w_gov, .. } =>
            (w_fuel.to_bits(), w_gov.to_bits(), p.tt4.to_bits()),
        _ => panic!("this arming marches the demand coordinate, so every point is `Demand`"),
    }
}

/// Arm a machine for the `demand × applied × track` cell — this slice's one live plant.
fn arm_track(m: &ScheduledStatorCore, tau_t: f64) {
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    m.fuel.inner.windup_law.set(WINDUP_LAW_TRACK);
    m.fuel.inner.tau_t.set(Some(tau_t));
}

// =============================================================================================
// 1 — THE TABLES: EIGHT SWAPS, ZERO ADDS, AND THE INHERITED SLOTS AS THE CONTROL
// =============================================================================================

/// **EIGHT POINTERS MOVE AND THE STRUCT DOES NOT GROW**, with the equality control riding beside
/// every inequality so a broken instrument fails visibly instead of passing
/// ([[rust-port-slice-aa-step1]]: never `ptr::eq` on the `const` itself).
///
/// The negative half is the one that matters at a `0 ADD` slice: with no width tripwire, an
/// inherited slot that was re-aimed by accident — say rung 76's table taking rung 74's
/// `windup_tau` instead of rung 75's — produces no compile error and no value break at any arm
/// either rung's suite reaches.
///
/// # WHAT THIS GATE ACTUALLY COMPARES — **COMPILED BODIES, NOT SOURCE SYMBOLS**, and that was
/// MEASURED rather than assumed
///
/// Step 1's mutation sweep dropped `r76_shared_rig`'s one statement, leaving a body that just
/// forwards to `R75_TRIPLE.shared_rig`. **The kill came from THIS gate rather than from the
/// carry's own value gate, and the reason is identical-code folding**: with the statement gone the
/// two functions get the SAME ADDRESS (`0x…5aac0` for both, against `0x…c220` / `0x…aac0` when
/// pristine — driven, printed and compared). The value assertions passed under the same mutation,
/// so the carry IS the measured no-op it was predicted to be and the kill is a different fact.
///
/// Two consequences, both worth having on the page. **In the useful direction** the gate is
/// stronger than designed: a swapped cell whose body degenerates into a pure forwarder is caught,
/// because folding makes it literally the parent's function. **In the other direction it is a
/// documented false-positive mode**: a cell that is legitimately a pure forwarder would fail here
/// while being a correct port, and would have to be gated some other way. No cell in this slice is
/// one — both re-aimed bodies at each rung do work of their own — which is why the gate stands as
/// written rather than being weakened to a source-level check it cannot make.
#[test]
fn the_slice_re_aims_eight_pointers_and_adds_none() {
    // RUNG 75's FOUR, against rung 74.
    assert!(!fn_addr_eq(R75_TRIPLE.windup_tau, R74_TRIPLE.windup_tau),
            "swap 1 of 8 — `_windup_tau`, the device's ONE hook into rung 74's march");
    assert!(!fn_addr_eq(R75_TRIPLE.shared_rig, R74_TRIPLE.shared_rig),
            "swap 2 of 8 — `_shared_rig`, carrying the device and the IC cap");
    assert!(!fn_addr_eq(R75.at_lever, R74.at_lever),
            "swap 3 of 8 — `at_lever`, FOURTEENTH instance of the sibling-constructor trap");
    assert!(!fn_addr_eq(R75_FUEL.integrate_fuel, R74_FUEL.integrate_fuel),
            "swap 4 of 8 — `integrate_fuel`, one refusal and a call made for its SIDE EFFECT");

    // RUNG 76's FOUR, against rung 75.
    assert!(!fn_addr_eq(R76_TRIPLE.sensed_cap, R75_TRIPLE.sensed_cap),
            "swap 5 of 8 — `_sensed_cap`, the first cap in the ladder that reads `mf_app`");
    assert!(!fn_addr_eq(R76_TRIPLE.shared_rig, R75_TRIPLE.shared_rig),
            "swap 6 of 8 — `_shared_rig` again, a FIFTH knob on the same leak");
    assert!(!fn_addr_eq(R76.at_lever, R75.at_lever), "swap 7 of 8 — `at_lever`, FIFTEENTH");
    assert!(!fn_addr_eq(R76_FUEL.integrate_fuel, R75_FUEL.integrate_fuel),
            "swap 8 of 8 — `integrate_fuel`, three refusals with two NESTED inside the first");

    // THE CROSS CHECK — the two rungs re-aim DIFFERENT third-loop cells, and each must leave the
    // other's alone. This is the pair a stray `..` spread would collapse.
    assert!(fn_addr_eq(R76_TRIPLE.windup_tau, R75_TRIPLE.windup_tau),
            "rung 76 keeps RUNG 75's device — a re-aim here would silently disarm it");
    assert!(fn_addr_eq(R75_TRIPLE.sensed_cap, R74_TRIPLE.sensed_cap),
            "and rung 75 keeps RUNG 74's `None` cap, which is the reduce arm one rung early");

    // EQUALITY CONTROL — every OTHER third-loop cell is the parent's, on BOTH tables. Sixteen
    // inherited at rung 75 and sixteen at rung 76, so "exactly two swaps each" is a measurement.
    for (a, b, name) in [
        (R75_TRIPLE.stator_leg as usize, R74_TRIPLE.stator_leg as usize, "stator_leg"),
        (R75_TRIPLE.lagged_stator as usize, R74_TRIPLE.lagged_stator as usize, "lagged_stator"),
        (R75_TRIPLE.clamp_v as usize, R74_TRIPLE.clamp_v as usize, "clamp_v"),
        (R75_TRIPLE.check_v0 as usize, R74_TRIPLE.check_v0 as usize, "check_v0"),
        (R75_TRIPLE.rk4_floor as usize, R74_TRIPLE.rk4_floor as usize, "rk4_floor"),
        (R75_TRIPLE.solve_v as usize, R74_TRIPLE.solve_v as usize, "solve_v"),
        (R75_TRIPLE.manifold_v as usize, R74_TRIPLE.manifold_v as usize, "manifold_v"),
        (R75_TRIPLE.triple_laws as usize, R74_TRIPLE.triple_laws as usize, "triple_laws"),
        (R75_TRIPLE.triple_rig as usize, R74_TRIPLE.triple_rig as usize, "triple_rig"),
        (R75_TRIPLE.with_ref as usize, R74_TRIPLE.with_ref as usize, "with_ref"),
        (R75_TRIPLE.reference as usize, R74_TRIPLE.reference as usize, "reference"),
        (R75_TRIPLE.quad_gains_at as usize, R74_TRIPLE.quad_gains_at as usize, "quad_gains_at"),
        (R75_TRIPLE.rk4_floor_shared as usize, R74_TRIPLE.rk4_floor_shared as usize,
         "rk4_floor_shared"),
        (R75_TRIPLE.cap_fuel as usize, R74_TRIPLE.cap_fuel as usize, "cap_fuel"),
        (R75_TRIPLE.with_coord as usize, R74_TRIPLE.with_coord as usize, "with_coord"),
    ] {
        assert_eq!(a, b, "rung 75 INHERITS `{name}` — a stray re-aim here has no tripwire");
    }
    for (a, b, name) in [
        (R76_TRIPLE.stator_leg as usize, R75_TRIPLE.stator_leg as usize, "stator_leg"),
        (R76_TRIPLE.lagged_stator as usize, R75_TRIPLE.lagged_stator as usize, "lagged_stator"),
        (R76_TRIPLE.clamp_v as usize, R75_TRIPLE.clamp_v as usize, "clamp_v"),
        (R76_TRIPLE.check_v0 as usize, R75_TRIPLE.check_v0 as usize, "check_v0"),
        (R76_TRIPLE.rk4_floor as usize, R75_TRIPLE.rk4_floor as usize, "rk4_floor"),
        (R76_TRIPLE.solve_v as usize, R75_TRIPLE.solve_v as usize, "solve_v"),
        (R76_TRIPLE.manifold_v as usize, R75_TRIPLE.manifold_v as usize, "manifold_v"),
        (R76_TRIPLE.triple_laws as usize, R75_TRIPLE.triple_laws as usize, "triple_laws"),
        (R76_TRIPLE.triple_rig as usize, R75_TRIPLE.triple_rig as usize, "triple_rig"),
        (R76_TRIPLE.with_ref as usize, R75_TRIPLE.with_ref as usize, "with_ref"),
        (R76_TRIPLE.reference as usize, R75_TRIPLE.reference as usize, "reference"),
        (R76_TRIPLE.quad_gains_at as usize, R75_TRIPLE.quad_gains_at as usize, "quad_gains_at"),
        (R76_TRIPLE.rk4_floor_shared as usize, R75_TRIPLE.rk4_floor_shared as usize,
         "rk4_floor_shared"),
        (R76_TRIPLE.cap_fuel as usize, R75_TRIPLE.cap_fuel as usize, "cap_fuel"),
        (R76_TRIPLE.with_coord as usize, R75_TRIPLE.with_coord as usize, "with_coord"),
    ] {
        assert_eq!(a, b, "rung 76 INHERITS `{name}`");
    }

    // THE ALIAS TABLES CARRY NO SWAP AT ALL — exhaustive destructurings, so a future field is a
    // compile error here rather than a silent pass.
    let turbojet::two_spool_transient::TwoSpoolTransientHooks {
        try_close: _, try_instant_tail: _, powers: _,
    } = R75_TWO;
    let turbojet::stator_transient::StatorTransientHooks {
        stator_march: _, v_of: _, arm: _, at_stator: _,
    } = R76_STATOR;
    assert!(fn_addr_eq(R75_TWO.try_close, R74_TWO.try_close));
    assert!(fn_addr_eq(R76_TWO.try_close, R75_TWO.try_close));
    assert!(fn_addr_eq(R75_STATOR.stator_march, R74_STATOR.stator_march));
    assert!(fn_addr_eq(R76_STATOR.stator_march, R75_STATOR.stator_march));

    // And every OTHER lever/fuel cell is the parent's on both rungs.
    assert!(fn_addr_eq(R75.b_at_point, R74.b_at_point));
    assert!(fn_addr_eq(R76.b_at_point, R75.b_at_point));
    assert!(fn_addr_eq(R75_FUEL.try_close_fuel, R74_FUEL.try_close_fuel));
    assert!(fn_addr_eq(R76_FUEL.try_surge_fuel, R75_FUEL.try_surge_fuel));
}

/// **`TripleHooks` IS STILL EIGHTEEN AFTER THIS SLICE** — spelled as an exhaustive destructuring
/// of BOTH new tables, the only form that fails when the struct grows.
///
/// This is the phase table's `0 ADD` row, checked rather than assumed. AC's `1` measured `0`,
/// AE's `0` measured `1 ADD + 6 SWAPS` and AF's `3` measured `4 ADD`; *the row happened to be
/// right* and *the row was checked* are different facts and only the second is evidence.
#[test]
fn the_third_loop_table_is_still_eighteen_fields_wide() {
    let TripleHooks {
        stator_leg: _, lagged_stator: _, clamp_v: _, check_v0: _, rk4_floor: _, solve_v: _,
        manifold_v: _, triple_laws: _, triple_rig: _, with_ref: _, reference: _,
        rk4_floor_shared: _, shared_rig: _, quad_gains_at: _, cap_fuel: _, sensed_cap: _,
        windup_tau: _, with_coord: _,
    } = R75_TRIPLE;
    let TripleHooks {
        stator_leg: _, lagged_stator: _, clamp_v: _, check_v0: _, rk4_floor: _, solve_v: _,
        manifold_v: _, triple_laws: _, triple_rig: _, with_ref: _, reference: _,
        rk4_floor_shared: _, shared_rig: _, quad_gains_at: _, cap_fuel: _, sensed_cap: _,
        windup_tau: _, with_coord: _,
    } = R76_TRIPLE;
}

/// **THE GATE ABOVE IS LOAD-BEARING, AND THE MEASUREMENT SPLITS BY REFERENCE** — a rung-75 table
/// with `windup_tau` left at rung 74's body is **BIT-FOR-BIT SILENT under `sched`** and **RAISES
/// under `applied`**, which is the whole of rung 75's own reason for existing.
///
/// The gate was written expecting silence on both arms and the `applied` arm FAILED it, which is
/// the better answer: rung 74 § 4 measured `demand × applied` as having no interior equilibrium,
/// so a march with the device missing hits rung 74's joint-IC refusal after 60 passes with the
/// residual not decaying at all. Under `sched` rung 74 DOES have a plant, so the same defect just
/// marches a different trajectory — 341 points that no panic, no reduce gate and no oracle key
/// distinguishes from the unarmed one, because they ARE the unarmed one.
///
/// **So whether a forgotten swap is loud depends on which cell the reader happens to drive**, and
/// at `0 ADD` nothing else is watching. That is why the pointer gate above exists rather than
/// being taken on trust.
///
/// It reproduces `test_rung75.py`'s `test_the_reduce_is_not_vacuous` (the `sched` half) and
/// `test_the_cell_rung74_has_no_plant_for_is_reached` (the `applied` half) from the port's own
/// side, without reading either.
#[test]
fn a_missed_re_aim_is_silent_under_sched_and_loud_under_applied() {
    // ---- ARM A: `demand x sched`, where rung 74 HAS a plant -- the defect is invisible.
    let good = with_triple(&valve_arm(), &R75_TRIPLE);
    arm_track(&good, TAU_T);
    good.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
    let armed = armed_march(&good, Some(TT4_MAX), Some(TAU_GOV));

    let broken = with_triple(&valve_arm(), &R75_TRIPLE_MISSED);
    arm_track(&broken, TAU_T);
    broken.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
    let missed = armed_march(&broken, Some(TT4_MAX), Some(TAU_GOV));

    let off = with_triple(&valve_arm(), &R75_TRIPLE);
    off.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    off.fuel.inner.ref_law.set(REF_LAW_DEFAULT);
    let unarmed = armed_march(&off, Some(TT4_MAX), Some(TAU_GOV));

    assert_eq!(missed.len(), unarmed.len(),
               "the MISSED re-aim marches exactly as far as the unarmed device");
    assert!(missed.iter().zip(unarmed.iter()).all(|(a, b)| state_of(a) == state_of(b)),
            "AND BIT-FOR-BIT THE SAME TRAJECTORY under `sched` — both masked states and the burnt              fuel — so a forgotten swap here is invisible to every value gate, every reduce gate              and the oracle");

    // THE POSITIVE CONTROL, AND IT HAD TO BE RE-AIMED ONCE: written against `Tt4` it FAILED, and
    // the failure is rung 75's own headline. The device sits in the MASKED leg's law, which
    // `min`-select hides from the plant, so on this arming it moves both leg STATES and leaves the
    // burnt fuel bit-identical. A control on the output alone would have read an exact zero and
    // concluded the correctly-aimed cell was inert — which is [[rust-port-slice-t-step1]]'s
    // *an EXACT ZERO blinds its own gate* one slice on, caught by the control failing rather than
    // by the gate passing.
    assert_eq!(armed.len(), unarmed.len());
    assert!(armed.iter().zip(unarmed.iter()).any(|(a, b)| state_of(a).0 != state_of(b).0
                                                       || state_of(a).1 != state_of(b).1),
            "THE POSITIVE CONTROL — with the cell correctly aimed the device MOVES both masked              STATES, so the equality above is a measurement and not a march that never ran");
    assert!(armed.iter().zip(unarmed.iter()).all(|(a, b)| a.tt4.to_bits() == b.tt4.to_bits()),
            "AND THE BURNT FUEL IS BIT-IDENTICAL — the device disarms itself on the leg that holds              the actuator, so rung 72's `ONE plant IS rungs 68-71 by AUTHORITY` is untouched and              this rung acts only where `min` is looking away");

    // ---- ARM B: `demand x applied`, the cell rung 74 has NO PLANT for -- the defect is loud.
    let ok = with_triple(&valve_arm(), &R75_TRIPLE);
    arm_track(&ok, TAU_T);
    let lit = armed_march(&ok, Some(TT4_MAX), Some(TAU_GOV));
    assert!(lit.len() > 300,
            "WITH THE DEVICE DECLARED THE CELL MARCHES — rung 75's reason for existing, and its              own suite's `test_the_cell_rung74_has_no_plant_for_is_reached`: {} points",
            lit.len());

    let dead = with_triple(&valve_arm(), &R75_TRIPLE_MISSED);
    arm_track(&dead, TAU_T);
    let msg = message_of(|| { armed_march(&dead, Some(TT4_MAX), Some(TAU_GOV)); });
    assert!(msg.contains("rung-74") && msg.contains("NO INTERIOR EQUILIBRIUM AT ALL"),
            "and WITHOUT it the same arming reaches rung 74's joint-IC refusal, so on THIS              reference the missed swap is loud: {msg:?}");
}

// =============================================================================================
// 2 — THE THREE DECLARED LAWS ON A FRESH MACHINE
// =============================================================================================

/// **A FRESH RUNG-76 MACHINE READS `"none"`, `None`, `"solve"` — ALL THREE REDUCE ARMS — AND THE
/// INHERITED `"applied"`.**
///
/// The first three are the constructor's own defaults, so they are stated rather than gated as
/// builder properties: an assertion that the builder SET them would pass for the reason that
/// nothing writes those fields ([`ref_law`]'s recorded lesson). The `_ref_law` half IS a real
/// builder property and is the thing a new cascade builder is likeliest to drop — hence the
/// rung-72 control, which is what makes it a measurement.
///
/// [`ref_law`]: turbojet::two_spool_transient::TwoSpoolTransientCore::ref_law
#[test]
fn a_fresh_machine_reads_all_three_reduce_arms_and_the_inherited_applied() {
    for (m, rung) in [(windup(&valve_arm()), 75), (sensed(&valve_arm()), 76)] {
        let c = &m.fuel.inner;
        assert_eq!(c.windup_law.get(), WINDUP_LAW_NONE, "rung {rung}: no declared device");
        assert_eq!(c.tau_t.get(), None, "rung {rung}: and no clock — UNSET, not zero");
        assert_eq!(c.cap_law.get(), CAP_LAW_SOLVE, "rung {rung}: rung 48's set-point solve");
        assert_eq!(c.lag_coord.get(), LAG_COORD_CLIP, "rung {rung}: rung 74's own reduce arm");
        assert_eq!(c.ic_cap.get(), IC_CAP_DECLARED, "rung {rung}: the inherited 60");
        assert_eq!(c.ref_law.get(), REF_LAW_APPLIED,
                   "rung {rung}: AND RUNG 73's CLASS ATTRIBUTE IS INHERITED — a builder that \
                    dropped the overwrite would march rung 72's reference and report this rung");
    }
    assert_eq!(shared(&valve_arm()).fuel.inner.ref_law.get(), REF_LAW_DEFAULT,
               "THE CONTROL — rung 72 declares `'sched'`, so a builder that set `'applied'` \
                everywhere would pass the assertions above");
}

/// The declared names are distinct strings, the admitted lists are exactly those names, and the
/// grid floor is a COMPUTATION.
///
/// A spelling pin, labelled as one: it catches a literal that drifted between a constant and the
/// refusal's admitted list, which is the only failure a constant-to-constant comparison can catch.
///
/// **The floor is the half that is not a pin.** `2*ds/(2 - ds*sum(1/tau_i))` at the inherited grid
/// is rung 75's own bound on how fast the tracking clock may run, and the assertion is that the
/// spelled derivation lands on the `0.00625` the class docstring quotes — so a future change to
/// `ds` or to the clock count moves both together instead of leaving a stale decimal.
#[test]
fn the_declared_laws_are_distinct_and_the_grid_floor_is_a_derivation() {
    assert_ne!(WINDUP_LAW_NONE, WINDUP_LAW_TRACK);
    assert_ne!(CAP_LAW_SOLVE, CAP_LAW_SENSED);
    assert_eq!(WINDUP_LAWS_DECLARED, [WINDUP_LAW_NONE, WINDUP_LAW_TRACK]);
    assert_eq!(CAP_LAWS_DECLARED, [CAP_LAW_SOLVE, CAP_LAW_SENSED]);
    assert_eq!(SHARE_LAW_DEFAULT, "max",
               "these are the FOURTH and FIFTH declared laws beside rung 72's composition, which \
                neither rung moves");
    // **AND THE DERIVATION IS NOT THE DOCSTRING's DECIMAL** — measured on the repo's own
    // interpreter, `2*ds/(2 - ds*sum(1/tau_i))` is `0.0062499999999999995` in Python too, one ULP
    // below the `0.00625` rung 75's class docstring quotes. Both spellings of the sum agree
    // (`sum(1.0/t for t in taus)` and `4.0/tau` are both exactly `80.0`), so the gap is the final
    // division's, not the sum's. That is precisely why the constant is spelled as the derivation:
    // a typed decimal here would be a DIFFERENT float from the one the bound actually is.
    assert_eq!(WINDUP_TAU_GRID_FLOOR, 0.0062499999999999995,
               "the port's expression lands on the float PYTHON's does");
    assert_ne!(WINDUP_TAU_GRID_FLOOR, 0.00625,
               "and NOT on the rounded decimal the docstring quotes — 1 ULP apart");
    assert!((WINDUP_TAU_GRID_FLOOR - 0.00625).abs() < 1e-17,
            "which is a last-bit gap and not a different bound");
    assert!(TAU_T > WINDUP_TAU_GRID_FLOOR,
            "and the shipped sweep's slow end is admissible on this grid");
}

// =============================================================================================
// 3 — RUNG 75's REFUSALS: TWO IN THE CELL, ONE AT THE MARCH, AND A CALL FOR ITS SIDE EFFECT
// =============================================================================================

/// **THE DEVICE IS REFUSED OUTSIDE THE PLAIN DEMAND COORDINATE**, with the legal cell as control.
///
/// `clip` still carries rung 52's `max(0, ·)` and `demand-latched` carries the latch, so either
/// would run TWO anti-windup devices at once and attribute the result to this one.
#[test]
fn the_device_is_refused_in_both_of_the_other_two_coordinates() {
    for coord in [LAG_COORD_CLIP, LAG_COORD_LATCHED] {
        let m = windup(&valve_arm());
        m.fuel.inner.lag_coord.set(coord);
        m.fuel.inner.windup_law.set(WINDUP_LAW_TRACK);
        m.fuel.inner.tau_t.set(Some(TAU_T));
        let msg = message_of(|| { let _ = (R75_TRIPLE.windup_tau)(&m.fuel.inner); });
        assert!(msg.contains("rung-75"), "the refusal names the rung: {msg:?}");
        assert!(msg.contains(&format!("{coord:?}")), "and quotes the coordinate: {msg:?}");
        assert!(msg.contains("TWO anti-windup devices"),
                "and gives the REASON rather than the rule: {msg:?}");
    }

    // THE CONTROL — the legal cell returns the declared clock, so the refusal above is measuring
    // the coordinate and not an arming mistake shared by all three.
    let ok = windup(&valve_arm());
    arm_track(&ok, TAU_T);
    assert_eq!((R75_TRIPLE.windup_tau)(&ok.fuel.inner), Some(TAU_T));

    // AND THE LAW IS CHECKED BEFORE THE COORDINATE — `"none"` in ANY coordinate is rung 74, which
    // is what makes the reduce a dispatch rather than a tolerance.
    for coord in [LAG_COORD_CLIP, LAG_COORD_DEMAND, LAG_COORD_LATCHED] {
        let m = windup(&valve_arm());
        m.fuel.inner.lag_coord.set(coord);
        assert_eq!((R75_TRIPLE.windup_tau)(&m.fuel.inner), None,
                   "{coord:?} under `none` is rung 74 EXACTLY, with no refusal on the way");
    }
}

/// **THE CLOCK IS DECLARED, NEVER DEFAULTED — and one Python assert covers TWO failures.**
///
/// `isinstance(self._tau_t, (int, float)) and self._tau_t > 0.0` refuses an UNSET clock and a
/// non-positive one with the same message, which is why the carrier is `Option<f64>` rather than a
/// `0.0` sentinel: collapsing them would make the refusal fire for the wrong reason.
///
/// **THE `NaN` ROW IS A CROSS-LANGUAGE CHECK, NOT PADDING.** Python's `nan > 0.0` is `False` and
/// Rust's is too, so both refuse — and a port that had written `!(x <= 0.0)` would admit it.
#[test]
fn the_tracking_clock_is_refused_when_unset_or_non_positive() {
    for tau in [None, Some(0.0), Some(-0.05), Some(f64::NAN)] {
        let m = windup(&valve_arm());
        m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
        m.fuel.inner.windup_law.set(WINDUP_LAW_TRACK);
        m.fuel.inner.tau_t.set(tau);
        let msg = message_of(|| { let _ = (R75_TRIPLE.windup_tau)(&m.fuel.inner); });
        assert!(msg.contains("rung-75"), "{tau:?}: the refusal names the rung: {msg:?}");
        assert!(msg.contains("DECLARED"), "{tau:?}: and says the clock is declared: {msg:?}");
        assert!(msg.contains("property of the SWEEP"),
                "{tau:?}: and why — no derivation exists, so no single value is a finding: \
                 {msg:?}");
    }
}

/// **THE `clip × track` REFUSAL FIRES THROUGH A MARCH THAT DISPATCHES STRAIGHT OUT** — and this is
/// the gate for the one line in rung 75's `integrate_fuel` that discards its own result.
///
/// Python's body calls `self._windup_tau()` and throws the value away. Its comment says why: the
/// `clip` coordinate dispatches OUT of this ladder before any hook is read, so without this call a
/// `clip × track` march runs rung 73 silently and reports rung 75. **A port that dropped the call
/// as dead code would fail nothing else in the crate** — the `clip` arm's whole point is that it
/// never reaches the march that would otherwise raise.
///
/// The control is the same arming under `"none"`, which must return CLEANLY rather than merely
/// miss the refusal.
#[test]
fn the_side_effect_call_puts_the_refusal_on_the_arm_that_dispatches_out() {
    let m = windup(&valve_arm());
    m.fuel.inner.lag_coord.set(LAG_COORD_CLIP);
    m.fuel.inner.windup_law.set(WINDUP_LAW_TRACK);
    m.fuel.inner.tau_t.set(Some(TAU_T));
    let msg = message_of(|| { bare_march(&m); });
    assert!(msg.contains("rung-75") && msg.contains("TWO anti-windup devices"),
            "the DEVICE's own refusal is reached from a march that never enters rung 74's \
             integrator: {msg:?}");

    let c = windup(&valve_arm());
    c.fuel.inner.lag_coord.set(LAG_COORD_CLIP);
    assert_eq!(message_of(|| { bare_march(&c); }), "",
               "THE CONTROL — `clip x none` is rung 73 and must RETURN, not merely miss the \
                refusal");
}

/// **THE ANTI-WINDUP LAW IS DECLARED**, and its refusal sits above everything.
#[test]
fn the_windup_law_refusal_admits_exactly_the_two_declared_names() {
    for law in ["clip", "back-calc", ""] {
        let m = windup(&valve_arm());
        m.fuel.inner.windup_law.set(law);
        let msg = message_of(|| { bare_march(&m); });
        assert!(msg.contains("rung-75"), "the refusal names the rung: {msg:?}");
        assert!(msg.contains(&format!("{law:?}")), "and quotes the offending law: {msg:?}");
        assert!(msg.contains("DECLARED"), "and says the law is declared: {msg:?}");
    }
    for law in WINDUP_LAWS_DECLARED {
        let m = windup(&valve_arm());
        m.fuel.inner.windup_law.set(law);
        // `track` needs the demand coordinate to get past the device's own refusal — which is the
        // conjunction the gate above measures from the other side.
        if law == WINDUP_LAW_TRACK { arm_track(&m, TAU_T); }
        assert_eq!(message_of(|| { bare_march(&m); }), "",
                   "{law:?} is declared: the bare march RETURNS");
    }
}

// =============================================================================================
// 4 — RUNG 76's REFUSALS: ONE UNCONDITIONAL, TWO NESTED INSIDE ITS ANSWER
// =============================================================================================

/// **THE NESTING IS GATED FROM BOTH SIDES, BECAUSE BOTH FAILURES ARE SILENT.**
///
/// Hoisting the inner two rejects a legal `solve` march on a `clip` machine with no accel leg —
/// which is rungs 48–75 and is exactly what the reduce arm has to keep marching. Flattening them
/// away lets `clip × sensed` through, and `clip` dispatches out before `_cap_fuel` is ever called,
/// so the march would silently be rung 73 and be reported as this rung.
#[test]
fn the_sensed_arms_two_refusals_fire_only_under_sensed() {
    // (1) `clip x sensed` — REFUSED.
    let m = sensed(&valve_arm());
    m.fuel.inner.cap_law.set(CAP_LAW_SENSED);
    let a = message_of(|| { bare_march(&m); });
    assert!(a.contains("rung-76") && a.contains("silently be rung 73"),
            "the CLIP conjunction is refused by name and by consequence: {a:?}");

    // (2) `sensed` with no schedule — REFUSED, and the reason is this rung's declared domain.
    let m2 = sensed(&valve_arm());
    m2.fuel.inner.cap_law.set(CAP_LAW_SENSED);
    m2.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let b = message_of(|| { armed_march(&m2, Some(TT4_MAX), Some(TAU_GOV)); });
    assert!(b.contains("rung-76") && b.contains("no sensed form in any rung"),
            "a floor on a STATE is not a formula for a FUEL: {b:?}");

    // (3) THE CONTROL, AND IT IS THE OTHER DIRECTION OF THE NESTING — the SAME two armings under
    //     `solve` must return cleanly, because that is rungs 48-75 and the reduce arm.
    let c = sensed(&valve_arm());
    assert_eq!(message_of(|| { bare_march(&c); }), "",
               "`clip x solve` is rungs 48-75 and MARCHES: hoisting the inner refusals would \
                reject it");
    let d = sensed(&valve_arm());
    d.fuel.inner.lag_coord.set(LAG_COORD_LATCHED);
    assert_eq!(message_of(|| { armed_march(&d, Some(TT4_MAX), Some(TAU_GOV)); }), "",
               "and so does a demand march with no accel leg under `solve`");
}

/// **THE CAP LAW IS DECLARED**, and its refusal is the unconditional one.
#[test]
fn the_cap_law_refusal_admits_exactly_the_two_declared_names() {
    for law in ["sense", "wf_pt3", ""] {
        let m = sensed(&valve_arm());
        m.fuel.inner.cap_law.set(law);
        let msg = message_of(|| { bare_march(&m); });
        assert!(msg.contains("rung-76"), "the refusal names the rung: {msg:?}");
        assert!(msg.contains(&format!("{law:?}")), "and quotes the offending law: {msg:?}");
        assert!(msg.contains("DECLARED"), "and says the law is declared: {msg:?}");
    }
}

/// **`_sensed_cap` ANSWERS `None` ON THE REDUCE ARM AND A NUMBER ON THE OTHER**, through the table.
///
/// `None` is rung 75 EXACTLY — the caller then takes the shipped set-point solve and no float in
/// this family moves. The `Some` half is the first cap in the ladder whose value depends on the
/// fuel it is asked about, and the two calls below differ only in `mf_app`, which is the whole
/// content of the rung.
///
/// **AND THE THREADING REFUSAL IS NOT A FALLBACK**, which is the sharp half: `mf_app` is threaded
/// from `_applied_demand` at every site that can have one, so a `None` there is a wiring defect —
/// and falling back to the solve would report rung 75 as this rung, undetectably.
#[test]
fn the_sensed_cap_answers_none_on_the_solve_arm_and_reads_the_fuel_on_the_other() {
    let sch = flat_schedule();

    let solve = sensed(&valve_arm());
    assert!((R76_TRIPLE.sensed_cap)(&solve.fuel, &flight(), 1.0, 1.0, &sch, Some(0.02))
                .expect("the solve arm cannot fail").is_none(),
            "`solve` is rung 75 EXACTLY — the branch is not taken and no float moves");

    let m = sensed(&valve_arm());
    m.fuel.inner.cap_law.set(CAP_LAW_SENSED);
    let lo = (R76_TRIPLE.sensed_cap)(&m.fuel, &flight(), 1.0, 1.0, &sch, Some(0.010))
        .expect("a sensed cap at a reachable fuel").expect("the sensed branch answers");
    let hi = (R76_TRIPLE.sensed_cap)(&m.fuel, &flight(), 1.0, 1.0, &sch, Some(0.030))
        .expect("a sensed cap at a reachable fuel").expect("the sensed branch answers");
    assert!(lo.is_finite() && hi.is_finite());
    assert_ne!(lo, hi,
               "THE WHOLE RUNG IN ONE ASSERTION — the cap DEPENDS on the fuel it is asked about, \
                where every other cap in this family is a function of the state alone");

    // THE THREADING REFUSAL.
    let msg = message_of(|| {
        let _ = (R76_TRIPLE.sensed_cap)(&m.fuel, &flight(), 1.0, 1.0, &sch, None);
    });
    assert!(msg.contains("rung-76") && msg.contains("THREADING bug"),
            "a missing `mf_app` is a wiring defect and is refused rather than defaulted: {msg:?}");
    assert!(msg.contains("report rung 75 as this rung"),
            "and the message says what the fallback would cost: {msg:?}");
}

// =============================================================================================
// 5 — THE CARRIERS: FOUR KNOBS AT RUNG 75, FIVE AT RUNG 76
// =============================================================================================

/// **`at_lever` HANDS BACK A MACHINE OF THE RIGHT CLASS CARRYING EVERY DECLARED KNOB**, at both
/// rungs, with an untouched-machine control so the assertions measure a COPY and not a constant.
#[test]
fn at_lever_carries_four_knobs_at_rung_75_and_five_at_rung_76() {
    // RUNG 75 — four.
    let m = windup(&valve_arm());
    arm_track(&m, TAU_T);
    m.fuel.inner.ref_law.set("sched");
    m.fuel.inner.ic_cap.set(400);
    let sib = m.at_lever(&valve_arm());
    assert!(fn_addr_eq(sib.triple_hooks().windup_tau, R75_TRIPLE.windup_tau),
            "the sibling is a RUNG-75 machine: its table has this rung's own device");
    assert_eq!(sib.fuel.inner.windup_law.get(), WINDUP_LAW_TRACK);
    assert_eq!(sib.fuel.inner.tau_t.get(), Some(TAU_T));
    assert_eq!(sib.fuel.inner.ic_cap.get(), 400,
               "AND `_ic_cap` IS CARRIED HERE WHERE RUNG 74's `at_lever` DOES NOT CARRY IT — \
                rung 75 is the first rung with a reader that RAISES it");
    assert_eq!(sib.fuel.inner.lag_coord.get(), LAG_COORD_DEMAND);
    assert_eq!(sib.fuel.inner.ref_law.get(), "sched");

    // THE RUNG-74 CONTROL for the `_ic_cap` half — its `at_lever` copies neither, so the cap falls
    // back to the declared 60 and the assertion above is a measurement.
    let m74 = demand(&valve_arm());
    m74.fuel.inner.ic_cap.set(400);
    assert_eq!(m74.at_lever(&valve_arm()).fuel.inner.ic_cap.get(), IC_CAP_DECLARED,
               "rung 74's sibling constructor does NOT carry the cap — Python's line, not an \
                omission");

    // RUNG 76 — five.
    let s = sensed(&valve_arm());
    arm_track(&s, TAU_T);
    s.fuel.inner.cap_law.set(CAP_LAW_SENSED);
    let ssib = s.at_lever(&valve_arm());
    assert!(fn_addr_eq(ssib.triple_hooks().sensed_cap, R76_TRIPLE.sensed_cap),
            "the sibling is a RUNG-76 machine");
    assert!(fn_addr_eq(ssib.fuel.hooks.integrate_fuel, R76_FUEL.integrate_fuel),
            "and carries this rung's fuel table, so its refusals are armed on the sibling too");
    assert_eq!(ssib.fuel.inner.cap_law.get(), CAP_LAW_SENSED);
    assert_eq!(ssib.fuel.inner.windup_law.get(), WINDUP_LAW_TRACK, "and rung 75's two");
    assert_eq!(ssib.fuel.inner.tau_t.get(), Some(TAU_T));

    // THE CONTROL — a sibling of an UNTOUCHED machine reads the class attributes.
    let fresh = sensed(&valve_arm()).at_lever(&valve_arm());
    assert_eq!(fresh.fuel.inner.windup_law.get(), WINDUP_LAW_NONE);
    assert_eq!(fresh.fuel.inner.tau_t.get(), None);
    assert_eq!(fresh.fuel.inner.cap_law.get(), CAP_LAW_SOLVE);
}

/// **BOTH `_shared_rig` CARRIES ARE MEASURED NO-OPS — the prediction is DRIVEN, not asserted.**
///
/// Rung 72's body reaches its sibling through `self.at_lever(…)`, which on a rung-75 or rung-76
/// receiver is that rung's own `at_lever`, which has already copied every knob. Slice AE's probe
/// L2 made this argument for `_ref_law` and slice AF re-made it for `_lag_coord`; the Rust form is
/// to call the PARENT's cell directly on the child's receiver and compare.
///
/// Ported unchanged regardless — a duplication the source makes is not the port's to remove — and
/// pre-registered here so step 6 does not hunt a value break that does not exist.
#[test]
fn both_shared_rig_carries_are_no_ops_because_at_lever_already_did_them() {
    let arm = SharedRigArm { sm: SM, tt4_max: TT4_MAX, ..SharedRigArm::default() };

    let m = windup(&valve_arm());
    arm_track(&m, TAU_T);
    let (own, _, _) = (R75_TRIPLE.shared_rig)(&m, &arm);
    let (parent, _, _) = (R74_TRIPLE.shared_rig)(&m, &arm);
    assert_eq!(own.fuel.inner.tau_t.get(), Some(TAU_T), "the rig carries the device");
    assert_eq!(parent.fuel.inner.tau_t.get(), own.fuel.inner.tau_t.get(),
               "AND RUNG 74's BODY ALREADY DID, through this rung's `at_lever` — so this swap has \
                NO value break");

    let s = sensed(&valve_arm());
    s.fuel.inner.cap_law.set(CAP_LAW_SENSED);
    let (sown, _, _) = (R76_TRIPLE.shared_rig)(&s, &arm);
    let (sparent, _, _) = (R75_TRIPLE.shared_rig)(&s, &arm);
    assert_eq!(sown.fuel.inner.cap_law.get(), CAP_LAW_SENSED);
    assert_eq!(sparent.fuel.inner.cap_law.get(), sown.fuel.inner.cap_law.get());

    // THE INSTRUMENT CAN SEE — the same comparison on a receiver whose `at_lever` knows nothing
    // about these fields. A rung-74 machine's sibling constructor drops all three, so the value
    // falls back to the class default and the two answers separate.
    let m74 = demand(&valve_arm());
    m74.fuel.inner.windup_law.set(WINDUP_LAW_TRACK);
    m74.fuel.inner.tau_t.set(Some(TAU_T));
    let (p74, _, _) = (R74_TRIPLE.shared_rig)(&m74, &arm);
    assert_eq!(p74.fuel.inner.tau_t.get(), None,
               "THE POSITIVE CONTROL — without rung 75's `at_lever` the clock is LOST, so the \
                equalities above are measurements and not tautologies");
}

// =============================================================================================
// 6 — THE FOLD THAT WAS FILED AS UNDECIDED, AND IS NOT
// =============================================================================================

/// **`1e-9f64.max(x)` IS PYTHON's `max(1e-9, x)` ON EVERY INPUT, NaN INCLUDED** — the measurement
/// behind the comment repaired in `demand_coordinate.rs` in this same commit.
///
/// Slice AF filed three `1e-9f64.max(·)` sites as *unmeasured* cells on the claim that *Python's
/// `max(1e-9, x)` returns `x` for a NaN `x`; Rust's `1e-9f64.max(x)` returns `1e-9`*. **The Python
/// half is false.** Python seeds the fold at argument 0 and replaces only on a strict comparison,
/// which a NaN never satisfies, so `max(1e-9, nan)` is `1e-9` — measured on the repo's own
/// interpreter and transcribed here as the bar. A NaN propagates only from argument 0, and Rust
/// discards it from either side, so the two spellings agree wherever **argument 0 is a literal**.
///
/// Censused over `engine.py`: 800 `max`/`min` calls, 268 of them n-ary, and **103 of those put a
/// literal first** — every one exactly faithful under `lit.max(x)`. The 165 with an expression
/// first are where the spelling is a decision, which is `applied_demand`'s gate in
/// `slice_af_laws.rs`.
///
/// This gate is the port's half of that measurement, and it is here rather than in a comment
/// because a claim about arithmetic is worth exactly what its instrument is.
#[test]
fn the_literal_first_fold_agrees_with_python_on_a_nan() {
    // MEASURED IN PYTHON 3.11 (PyPy, the repo venv): 1e-09, 1e-09, nan.
    assert_eq!(1e-9f64.max(f64::NAN), 1e-9, "Python's `max(1e-9, nan)` is 1e-09");
    assert_eq!(1e-9f64.min(f64::NAN), 1e-9, "and `min(1e-9, nan)` is 1e-09");
    assert!(f64::NAN.max(1e-9) == 1e-9,
            "THE DIVERGENCE IS REAL AND IT IS ON THE OTHER SIDE — Python's `max(nan, 1e-9)` is \
             `nan` and Rust's is 1e-09, so only a fold whose FIRST argument can be NaN is a \
             decision");
    // And the ordinary rows, so the gate is not three NaN facts and nothing else.
    assert_eq!(1e-9f64.max(0.02), 0.02);
    assert_eq!(1e-9f64.max(-1.0), 1e-9);
}

