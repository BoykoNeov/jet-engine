//! SLICE AF step 1 — **FOUR NEW CELLS, FOUR RE-AIMED POINTERS, AND FIVE REFUSALS THAT ARE SPLIT
//! ACROSS A DISPATCH.**
//!
//! § 5.30 (v)'s step list calls this file *a smoke file*. It is named `slice_af_cells.rs` instead,
//! because a smoke file runs READERS end to end (`slice_ab_smoke.rs` is the pattern) and this step
//! has none — rung 74's six readers land at step 4. The job here is slice AB/AC/AD/AE's cells-file
//! job: the table, the cells, the carriers and the refusals.
//!
//! # THE THREE SILENT FAILURES THIS FILE EXISTS TO CATCH
//!
//! **1 — THE REFUSAL SPLIT.** Python fires ONE assert above the entry test and FOUR below it
//! (`engine.py:17759`–`17787`). Slice AE's rung-73 body has BOTH its asserts above, so *hoist the
//! refusals* is the inherited habit and it is wrong here: hoisting all five raises on arms Python
//! passes, and sinking all five skips the coordinate refusal on exactly the `clip` arm. **Both
//! failures are silent**, so [`the_coordinate_refusal_fires_above_the_entry_test`] and
//! [`the_other_four_refusals_fire_only_below_it`] gate the two halves separately.
//!
//! **2 — A DEMAND ARM THAT QUIETLY DELEGATES.** `_integrate_fuel_demand` lands at step 3. The
//! worst thing this step could ship is a demand arm that falls through to rung 73, because **the
//! reduce IS *rung 74 under `clip` is rung 73*** — every reduce gate in the crate would go on
//! passing. [`a_legal_demand_march_reaches_the_unimplemented_march`] asserts the arm is reached.
//!
//! **3 — A FROZEN DISPATCH.** `_cap_fuel` reaches `_sensed_cap` as `self._sensed_cap(…)`, so an
//! inherited rung-74 reader run on a rung-76 machine must take rung 76's body. Calling the rung-74
//! function directly would compile, pass every value gate at this rung (both bodies agree here —
//! rung 74's returns `None` and that is the branch the solve path takes anyway) and break rung 76
//! silently. [`cap_fuel_reaches_sensed_cap_through_the_table`] injects a table to measure it.
//!
//! # WHAT THIS FILE DELIBERATELY DOES NOT GATE
//!
//! **`_with_coord` BY VALUE.** § 5.30 (i) measured the coordinate's one reader arithmetically the
//! identity everywhere this scope's one call site looks — 0 of 1 040 and 0 of 624 calls, with a
//! mutated reader moving 4 of 20 keys on every arm, so the instrument can see and the zero is
//! ARITHMETIC. **No value gate exists for this cell at this rung**, which is P5; everything below
//! gates it structurally. Pre-registered so step 5 does not hunt a discriminator that is not there.
//!
//! **`_shared_rig`'s carry as a value break.** Predicted a NO-OP for slice AE's probe-L2 reason
//! and DRIVEN below rather than asserted — the parent's body is called directly on a rung-74
//! receiver and the two answers compared.
//!
//! **The four cells through a real inherited reader.** Step 6's dispatch gates.
//!
//! **NO GATE HERE HAS AN EXPIRY DATE.** Slice AB's step-1 mistake was an *every slot panics* gate
//! that had to be dismantled when the bodies landed. The panicking-slot gate below cannot expire:
//! these four names do not exist below rung 74 in Python at all, so no future slice fills those
//! slots. The width destructuring going loud is the tripwire working, not a gate breaking.
//!
//! **NOTHING HERE READS A GOLDEN.** Every assertion is a panic, a same-run difference, or a
//! compile-time property.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;

use turbojet::applied_reference::{
    build_applied_reference_cascade, REF_LAW_APPLIED, R73, R73_FUEL, R73_STATOR, R73_TRIPLE,
    R73_TWO,
};
use turbojet::bleed_transient::{LeverArm, LeverArming};
use turbojet::demand_coordinate::{
    build_demand_coordinate_cascade, cap_free, CoordScope, CAP_BRACKET_N, CAP_GROW,
    IC_CAP_DECLARED, LAG_COORDS_DECLARED, LAG_COORD_CLIP, LAG_COORD_DEMAND, LAG_COORD_LATCHED,
    R74, R74_FUEL, R74_STATOR, R74_TRIPLE, R74_TWO,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelLimiters, SurgeLimiter,
};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::{
    build_shared_actuator_cascade, SharedRigArm, REF_LAW_DEFAULT, SHARE_LAW_DEFAULT, R72_TRIPLE,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::TripleHooks;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung74.py`'s module constants. This slice adds no constant of its own and this file
// must not be the place one appears.
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
const PHI: f64 = 0.80;
/// `PHI / FLOOR - 1.0` — the expression Python spells, never a typed decimal.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
const TT4_MAX: f64 = 1200.0;

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
/// The valve WITHOUT a clock — rung 65/66's refused arming, and the only way to reach the fourth
/// post-entry refusal.
fn bare_valve() -> BleedLimiter { BleedLimiter::from_margin(&lp_map(), B, SM) }
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

/// Python's `DemandCoordinateTransient(design, …)` — rung 74.
fn demand(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Rung 73 — the immediate parent, and the control at every step below.
fn applied(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_applied_reference_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Rung 72 — the grandparent, whose `_ref_law` default is the OTHER one.
fn shared(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_shared_actuator_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

fn valve_arm() -> LeverArm { LeverArm { bleed_lim: Some(valve()), ..Default::default() } }
fn bare_valve_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(bare_valve()), ..Default::default() }
}

/// The panic message a closure produces, or `""` if it did not panic. Slice AB/AC/AD/AE's helper
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
/// through the real march entry.
///
/// **THE SCHEDULE HAS TO BE THE PLANT's OWN, AND THE FIRST VERSION OF THIS HELPER WAS NOT.** It
/// called `integrate_fuel` directly with a hand-written `|_s| 1.0`, and the `clip` control — which
/// asserts a CLEAN RETURN, not an absent substring — went red with
/// `rung-43 fuel closure does not bracket at nu=(1.0000,1.0000), mdot_fuel=1.00000`. One kilogram
/// per second is off the modelled speed-line region, so the control was measuring an unrelated
/// abort. That is slice AE's recorded reason for asserting the empty message rather than a missing
/// needle, and here the strong form caught its own driver: a weaker control would have passed on a
/// march that never ran. The ramp supplies the schedule the plant is actually matched to.
fn armed_march(m: &ScheduledStatorCore, tt4_max: Option<f64>, tau_gov: Option<f64>) {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: Some(Floor::Phi(surge())),
                          tt4_max };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg,
                          &MarchScope { lag: Some(lag()), tau_gov, ..MarchScope::DEFAULT });
}

/// The ONE refusal a march cannot reach: `s_off` is rungs 50/51's forced release edge, and neither
/// [`StatorLeg`] nor [`MarchScope`] carries it — which is itself rung 73's structural guard, so
/// the field can only be supplied by calling the fuel table directly.
///
/// `rung73.rs`'s own spelling, and the crude `|_s| 1.0` schedule is harmless HERE for a reason
/// worth stating: every refusal in this body fires before a single derivative is evaluated, so the
/// schedule is never consulted. It would not be harmless in a gate that expects a clean return,
/// which is exactly what [`armed_march`] exists for.
fn forced_release_drive(m: &ScheduledStatorCore) {
    let lim = FuelLimiters {
        freeze: None, tt4_max: Some(TT4_MAX), tau_gov: Some(TAU_GOV), accel: None,
        surge: Some(surge()), incidence: None, s_off: Some(0.3), tau_rel: None, lag: Some(lag()),
    };
    (m.fuel.hooks.integrate_fuel)(&m.fuel, &flight(), &|_s: f64| 1.0, (1.0, 1.0), 0.1, DS, &lim);
}

// =============================================================================================
// 1 — THE TABLE: FOUR ADDED CELLS, FOUR RE-AIMED POINTERS, TWELVE INHERITED
// =============================================================================================

/// The four ADDs point at rung 74's own bodies where rung 73's table carries the panicking slot;
/// the four swaps are DIFFERENT pointers from rung 73's; and the equality control rides beside
/// each inequality so a broken instrument fails visibly instead of passing
/// ([[rust-port-slice-aa-step1]]: never `ptr::eq` on the `const` itself).
#[test]
fn the_slice_adds_four_cells_re_aims_four_and_inherits_twelve() {
    // THE FOUR ADDITIONS — rung 73's slots are `NO_TRIPLE`'s panicking bodies, rung 74's are not.
    assert!(!fn_addr_eq(R74_TRIPLE.cap_fuel, R73_TRIPLE.cap_fuel), "ADD 1 of 4 — `_cap_fuel`");
    assert!(!fn_addr_eq(R74_TRIPLE.sensed_cap, R73_TRIPLE.sensed_cap),
            "ADD 2 of 4 — `_sensed_cap`");
    assert!(!fn_addr_eq(R74_TRIPLE.windup_tau, R73_TRIPLE.windup_tau),
            "ADD 3 of 4 — `_windup_tau`");
    assert!(!fn_addr_eq(R74_TRIPLE.with_coord, R73_TRIPLE.with_coord),
            "ADD 4 of 4 — `_with_coord`, THE NAME REUSE rung 79 will repeat on another field");

    // AND RUNG 73's FOUR SLOTS ARE RUNG 68's — the panicking bodies, reached where they live.
    for (a, b, name) in [
        (R73_TRIPLE.cap_fuel as usize, R72_TRIPLE.cap_fuel as usize, "cap_fuel"),
        (R73_TRIPLE.sensed_cap as usize, R72_TRIPLE.sensed_cap as usize, "sensed_cap"),
        (R73_TRIPLE.windup_tau as usize, R72_TRIPLE.windup_tau as usize, "windup_tau"),
        (R73_TRIPLE.with_coord as usize, R72_TRIPLE.with_coord as usize, "with_coord"),
    ] {
        assert_eq!(a, b, "rungs 72 and 73 carry the SAME refusal for `{name}`");
    }

    // THE FOUR SWAPS.
    assert!(!fn_addr_eq(R74_TRIPLE.rk4_floor_shared, R73_TRIPLE.rk4_floor_shared),
            "swap 1 of 4 — `_rk4_floor_shared`, where the MESSAGE is the entire cell");
    assert!(!fn_addr_eq(R74_TRIPLE.shared_rig, R73_TRIPLE.shared_rig),
            "swap 2 of 4 — `_shared_rig`, carrying the coordinate");
    assert!(!fn_addr_eq(R74.at_lever, R73.at_lever),
            "swap 3 of 4 — `at_lever`, the sibling constructor, THIRTEENTH instance");
    assert!(!fn_addr_eq(R74_FUEL.integrate_fuel, R73_FUEL.integrate_fuel),
            "swap 4 of 4 — `integrate_fuel`, the five split refusals");

    // EQUALITY CONTROL — the twelve inherited third-loop cells ARE rung 73's, so the instrument
    // can tell the two answers apart on the same table.
    assert!(fn_addr_eq(R74_TRIPLE.stator_leg, R73_TRIPLE.stator_leg));
    assert!(fn_addr_eq(R74_TRIPLE.lagged_stator, R73_TRIPLE.lagged_stator));
    assert!(fn_addr_eq(R74_TRIPLE.clamp_v, R73_TRIPLE.clamp_v));
    assert!(fn_addr_eq(R74_TRIPLE.check_v0, R73_TRIPLE.check_v0));
    assert!(fn_addr_eq(R74_TRIPLE.rk4_floor, R73_TRIPLE.rk4_floor));
    assert!(fn_addr_eq(R74_TRIPLE.solve_v, R73_TRIPLE.solve_v));
    assert!(fn_addr_eq(R74_TRIPLE.manifold_v, R73_TRIPLE.manifold_v));
    assert!(fn_addr_eq(R74_TRIPLE.triple_laws, R73_TRIPLE.triple_laws));
    assert!(fn_addr_eq(R74_TRIPLE.triple_rig, R73_TRIPLE.triple_rig));
    assert!(fn_addr_eq(R74_TRIPLE.with_ref, R73_TRIPLE.with_ref));
    assert!(fn_addr_eq(R74_TRIPLE.reference, R73_TRIPLE.reference));
    assert!(fn_addr_eq(R74_TRIPLE.quad_gains_at, R73_TRIPLE.quad_gains_at));

    // THE TWO ALIAS TABLES CARRY NO SWAP AT ALL — exhaustive destructurings, so a future field is
    // a compile error here rather than a silent pass.
    let turbojet::two_spool_transient::TwoSpoolTransientHooks {
        try_close: _, try_instant_tail: _, powers: _,
    } = R74_TWO;
    let turbojet::stator_transient::StatorTransientHooks {
        stator_march: _, v_of: _, arm: _, at_stator: _,
    } = R74_STATOR;
    assert!(fn_addr_eq(R74_TWO.try_close, R73_TWO.try_close));
    assert!(fn_addr_eq(R74_STATOR.stator_march, R73_STATOR.stator_march));

    // And every OTHER lever/fuel cell is rung 73's, so "exactly four swaps" is a measurement.
    assert!(fn_addr_eq(R74.b_at_point, R73.b_at_point));
    assert!(fn_addr_eq(R74_FUEL.try_close_fuel, R73_FUEL.try_close_fuel));
    assert!(fn_addr_eq(R74_FUEL.try_surge_fuel, R73_FUEL.try_surge_fuel));
}

/// **`TripleHooks` IS EIGHTEEN AFTER THIS STEP** — spelled as an exhaustive destructuring, the
/// only form that fails when the struct grows.
///
/// The arrival 14 → 18 fired **four** test-target tripwires and not the two the phase record
/// names: slice AB's and AC's `E0063` initializer literals, and **two `E0027` destructurings in
/// `slice_ae_cells.rs` and `slice_ae_dispatch.rs`** that nothing in the crate had called
/// tripwires. Measured slice AD's way — apply, fix the lib, count what is still red — because
/// `cargo check --all-targets` stops when the lib fails and never reaches a test target.
#[test]
fn the_third_loop_table_is_eighteen_fields_wide() {
    let TripleHooks {
        stator_leg: _, lagged_stator: _, clamp_v: _, check_v0: _, rk4_floor: _, solve_v: _,
        manifold_v: _, triple_laws: _, triple_rig: _, with_ref: _, reference: _,
        rk4_floor_shared: _, shared_rig: _, quad_gains_at: _,
        // SLICE AF's FOUR.
        cap_fuel: _, sensed_cap: _, windup_tau: _, with_coord: _,
    } = R74_TRIPLE;
}

/// **RUNGS 40–73 PANIC ON ALL FOUR NAMES, AND THE MESSAGE SAYS WHY A DEFAULT WOULD BE WORSE.**
///
/// `_cap_fuel`, `_sensed_cap`, `_windup_tau` and `_with_coord` do not exist below rung 74 in
/// Python at all, so **this gate cannot expire** — no future slice fills those slots, which is
/// what slice AB's dismantled *every slot panics* gate lacked.
///
/// The sharp half is the two that return `None` at rung 74: a parent slot answering `None` would
/// agree with rung 74's own body on every input any suite reaches, so the refusal is what keeps
/// the slot visible. That is [`NO_TRIPLE`](turbojet::three_loop::NO_TRIPLE)'s stated reason and
/// the assertion below reads the message for it rather than merely for a panic.
#[test]
fn the_parent_slots_refuse_all_four_names() {
    let m = applied(&valve_arm());
    let c = &m.fuel.inner;
    let sched = flat_schedule();

    let a = message_of(|| { let _ = (R73_TRIPLE.cap_fuel)(
        &m.fuel, &flight(), 1.0, 1.0, 0.02, None, None, None); });
    let b = message_of(|| { let _ = (R73_TRIPLE.sensed_cap)(
        &m.fuel, &flight(), 1.0, 1.0, &sched, None); });
    let d = message_of(|| { let _ = (R73_TRIPLE.windup_tau)(c); });
    let e = message_of(|| { let _ = (R73_TRIPLE.with_coord)(c, LAG_COORD_DEMAND); });

    for (msg, name) in [(&a, "_cap_fuel"), (&b, "_sensed_cap"), (&d, "_windup_tau"),
                        (&e, "_with_coord")] {
        assert!(msg.contains("RUNG 74's"), "{name}: the refusal names the owning rung: {msg:?}");
        assert!(msg.contains(name), "{name}: and the offending cell: {msg:?}");
        assert!(msg.contains("no value gate could see"),
                "{name}: and states why a DEFAULT would be worse than a panic: {msg:?}");
    }
    // AND THE `None` HALF IS NAMED EXPLICITLY, because two of these four return `None` at rung 74
    // and that is the tempting default.
    assert!(a.contains("`None`"), "the refusal calls out the dangerous default: {a:?}");
}

// =============================================================================================
// 2 — THE TWO DECLARED LAWS ON A FRESH MACHINE
// =============================================================================================

/// **A FRESH RUNG-74 MACHINE READS `"clip"` AND `"applied"`.**
///
/// The coordinate half is the DEFAULT the constructor already writes, so — unlike rung 73's
/// `_ref_law` — there is nothing for the builder to overwrite. That is stated rather than gated as
/// a builder property: an assertion that the builder SET it would pass for the reason that nothing
/// writes the field ([`ref_law`]'s own recorded lesson, mirrored).
///
/// **The `_ref_law` half is a real builder property and it is INHERITED**, which is the thing a
/// new cascade builder is likeliest to drop: `DemandCoordinateTransient` subclasses
/// `AppliedReferenceTransient`, so Python's class attribute is `"applied"` here too, while the
/// core's constructor writes `"sched"` for the whole family. The rung-72 control is what makes it
/// a measurement rather than a restatement of the constant.
///
/// [`ref_law`]: turbojet::two_spool_transient::TwoSpoolTransientCore::ref_law
#[test]
fn a_fresh_machine_reads_clip_and_the_inherited_applied() {
    let m = demand(&valve_arm());
    assert_eq!(m.fuel.inner.lag_coord.get(), LAG_COORD_CLIP,
               "rung 74 declares `_lag_coord = 'clip'`, which is also the reduce arm");
    assert_eq!(m.fuel.inner.ref_law.get(), REF_LAW_APPLIED,
               "AND RUNG 73's CLASS ATTRIBUTE IS INHERITED — a builder that dropped the \
                overwrite would march rung 72's reference and report rung 74");
    assert_eq!(shared(&valve_arm()).fuel.inner.ref_law.get(), REF_LAW_DEFAULT,
               "THE CONTROL — rung 72 declares `'sched'`, so a builder that set `'applied'` \
                everywhere would pass the assertion above");
    assert_eq!(m.fuel.inner.ic_cap.get(), IC_CAP_DECLARED,
               "and `_ic_cap` carries the declared 60 — its one reader is step 3's march");

    // THE THREE DECLARED COORDINATES ARE EXACTLY THE REFUSAL's ADMITTED LIST.
    assert_eq!(LAG_COORDS_DECLARED, [LAG_COORD_CLIP, LAG_COORD_DEMAND, LAG_COORD_LATCHED]);
    assert!(LAG_COORDS_DECLARED.contains(&m.fuel.inner.lag_coord.get()),
            "the class default is one of the declared three");
}

// =============================================================================================
// 3 — THE NAME REUSE: WHICH FIELD IS WRITTEN, GATED FROM BOTH SIDES
// =============================================================================================

/// **`_with_coord` MOVES `lag_coord` AND LEAVES EVERY NEIGHBOURING CARRIER ALONE.**
///
/// Rung 79 defines the same name with the same signature and writes `_phi_ref`. Both fields will
/// exist on a rung-79 machine, so nothing type-errors and no signature comparison can reach the
/// difference — which is exactly how slice AE's `_with_ref` pair got filed as *harmlessly
/// renamed*. **The gate has to be two-sided or it measures nothing**: a body that wrote BOTH
/// fields would pass *"`lag_coord` moved"*.
///
/// `phi_ref` does not exist yet, so the negative half is taken on the two carriers that DO —
/// `ref_` (rung 69's) and `ref_law` (rung 73's), the two a mis-aimed setter would most plausibly
/// hit, since they are the fields the neighbouring guards write.
#[test]
fn with_coord_moves_lag_coord_and_leaves_the_neighbouring_carriers_alone() {
    let m = demand(&valve_arm());
    let c = &m.fuel.inner;
    c.ref_.set(Some("phi"));
    c.ref_law.set(REF_LAW_APPLIED);

    let displaced = (R74_TRIPLE.with_coord)(c, LAG_COORD_LATCHED);
    assert_eq!(displaced, LAG_COORD_CLIP, "it hands back what it displaced — Python's `prev`");
    assert_eq!(c.lag_coord.get(), LAG_COORD_LATCHED, "the POSITIVE half: `_lag_coord` moved");
    assert_eq!(c.ref_.get(), Some("phi"),
               "THE NEGATIVE HALF — rung 69's carrier is untouched");
    assert_eq!(c.ref_law.get(), REF_LAW_APPLIED,
               "THE NEGATIVE HALF — rung 73's carrier is untouched");

    // AND THE OTHER DIRECTION: rung 73's `_with_ref` must not move THIS field.
    let before = c.lag_coord.get();
    let _ = (R74_TRIPLE.with_ref)(c, Some("sched"));
    assert_eq!(c.lag_coord.get(), before,
               "the two setters are two cells: `_with_ref` leaves the coordinate alone");
}

/// **THE GUARD RESTORES ON DROP, AND ON AN UNWIND** — which is what a `finally` buys and a
/// straight-line restore does not.
///
/// The nested case is manufactured, because on every shipped path the displaced value is the class
/// default and restore-previous agrees with restore-to-default there. That is
/// [`RefScope`](turbojet::reference_split::RefScope)'s recorded situation and the reason a nest is
/// the only instrument that separates the two policies.
#[test]
fn the_coord_scope_restores_previous_on_drop_and_on_unwind() {
    let m = demand(&valve_arm());
    let c = &m.fuel.inner;

    // A MANUFACTURED NEST — the only shape in which restore-previous and restore-to-default
    // disagree. The inner scope must put `"demand"` back, not `"clip"`.
    {
        let outer = CoordScope::set(c, LAG_COORD_DEMAND);
        assert_eq!(outer.displaced(), LAG_COORD_CLIP);
        assert_eq!(c.lag_coord.get(), LAG_COORD_DEMAND);
        {
            let inner = CoordScope::set(c, LAG_COORD_LATCHED);
            assert_eq!(inner.displaced(), LAG_COORD_DEMAND);
            assert_eq!(c.lag_coord.get(), LAG_COORD_LATCHED);
        }
        assert_eq!(c.lag_coord.get(), LAG_COORD_DEMAND,
                   "RESTORE-PREVIOUS, not restore-to-default: a `clip` here is the wrong policy");
    }
    assert_eq!(c.lag_coord.get(), LAG_COORD_CLIP, "and the outer scope restores the default");

    // AND THROUGH AN UNWIND — Python's `finally`, which is the whole reason the guard is RAII.
    let msg = message_of(|| {
        let _g = CoordScope::set(c, LAG_COORD_DEMAND);
        panic!("a reader that raised");
    });
    assert_eq!(msg, "a reader that raised");
    assert_eq!(c.lag_coord.get(), LAG_COORD_CLIP,
               "the restore survived the unwind — a straight-line restore would have leaked");
}

// =============================================================================================
// 4 — THE TWO CARRIERS: `at_lever` AND `_shared_rig`
// =============================================================================================

/// **THE THIRTEENTH INSTANCE OF THE TRAP, GATED ON ALL THREE OF ITS HEADS.**
///
/// A sibling must be a RUNG-74 machine (the class), and it must carry BOTH declared laws (the
/// values) — a sibling built while the receiver sits under a [`CoordScope`]-set `"demand"` that
/// came back reading the class default `"clip"` would march rung 73 silently.
///
/// `_ic_cap` is deliberately NOT asserted as carried: rung 74's `at_lever` copies `_ref_law` and
/// `_lag_coord` only, and rung 75's is the one that adds `_ic_cap` (`engine.py:17711` against
/// `18671`). Asserting a carry Python does not make would be gating the port against a habit.
#[test]
fn at_lever_hands_back_a_rung_74_machine_carrying_both_laws() {
    let m = demand(&valve_arm());
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    m.fuel.inner.ref_law.set("sched");

    let sib = m.at_lever(&valve_arm());

    // THE CLASS — measured on the tables the sibling carries, not on a type name.
    assert!(fn_addr_eq(sib.triple_hooks().with_coord, R74_TRIPLE.with_coord),
            "the sibling is a RUNG-74 machine: its table has this rung's own cell");
    assert!(fn_addr_eq(sib.fuel.hooks.integrate_fuel, R74_FUEL.integrate_fuel),
            "and this rung's fuel table, so its refusals are armed on the sibling too");

    // THE TWO VALUES — copied from the SOURCE core and never left at the class default.
    assert_eq!(sib.fuel.inner.lag_coord.get(), LAG_COORD_DEMAND,
               "the coordinate is carried; the class default `clip` here would be rung 73");
    assert_eq!(sib.fuel.inner.ref_law.get(), "sched",
               "and so is the reference — rung 73's half, still live");

    // THE CONTROL — a sibling of an UNTOUCHED machine reads the two class attributes, so the two
    // assertions above are measuring a COPY and not a constant.
    let fresh = demand(&valve_arm()).at_lever(&valve_arm());
    assert_eq!(fresh.fuel.inner.lag_coord.get(), LAG_COORD_CLIP);
    assert_eq!(fresh.fuel.inner.ref_law.get(), REF_LAW_APPLIED);
}

/// **`_shared_rig`'s CARRY IS A MEASURED NO-OP — the prediction is DRIVEN, not asserted.**
///
/// Rung 72's body reaches its sibling through `self.at_lever(…)`, which on a rung-74 receiver is
/// this rung's own `at_lever`, which has already copied both laws. Slice AE's probe L2 made the
/// same argument for `_ref_law` and measured it; the Rust form of that measurement is to call the
/// PARENT's cell directly on a rung-74 receiver and compare what comes back.
///
/// Ported unchanged regardless — a duplication the source makes is not the port's to remove
/// ([[rust-port-copy-vs-rederivation]]) — and pre-registered here so step 5 does not hunt a value
/// break that does not exist.
#[test]
fn shared_rigs_coordinate_carry_is_a_no_op_because_at_lever_already_did_it() {
    let m = demand(&valve_arm());
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let arm = SharedRigArm { sm: SM, tt4_max: TT4_MAX, ..SharedRigArm::default() };

    let (own, _, _) = (R74_TRIPLE.shared_rig)(&m, &arm);
    let (parent, _, _) = (R73_TRIPLE.shared_rig)(&m, &arm);

    assert_eq!(own.fuel.inner.lag_coord.get(), LAG_COORD_DEMAND, "the rig carries the coordinate");
    assert_eq!(parent.fuel.inner.lag_coord.get(), own.fuel.inner.lag_coord.get(),
               "AND THE PARENT's BODY ALREADY DID — so this swap has NO value break, and step 5 \
                must not go looking for one");

    // THE INSTRUMENT CAN SEE — the same comparison on a machine whose `at_lever` does NOT carry
    // the coordinate. A rung-73 receiver's sibling constructor knows nothing about `_lag_coord`,
    // so the field falls back to the class default and the two answers separate.
    let m73 = applied(&valve_arm());
    m73.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let (p73, _, _) = (R73_TRIPLE.shared_rig)(&m73, &arm);
    assert_eq!(p73.fuel.inner.lag_coord.get(), LAG_COORD_CLIP,
               "THE POSITIVE CONTROL — without rung 74's `at_lever` the coordinate is LOST, so \
                the equality above is a measurement and not a tautology");
}

// =============================================================================================
// 5 — THE REFUSAL SPLIT: ONE ABOVE THE ENTRY TEST, FOUR BELOW IT
// =============================================================================================

/// **THE COORDINATE REFUSAL FIRES ON AN ARMING THAT DISPATCHES STRAIGHT OUT TO RUNG 73.**
///
/// The bare march has no governor clock and no fuel leg, so `has_fuel` is false and the body's
/// early return is taken — and Python's first assert sits ABOVE that return, so it still raises.
/// A port that sank the refusals below the entry test would pass every other gate in this file and
/// fail here, which is the whole reason this gate exists separately from the next one.
#[test]
fn the_coordinate_refusal_fires_above_the_entry_test() {
    for coord in ["demand-latch", "phi", ""] {
        let m = demand(&valve_arm());
        m.fuel.inner.lag_coord.set(coord);
        let msg = message_of(|| { bare_march(&m); });
        assert!(msg.contains("rung-74"), "the refusal names the rung: {msg:?}");
        assert!(msg.contains(&format!("{coord:?}")),
                "and quotes the offending coordinate {coord:?}: {msg:?}");
        assert!(msg.contains("DECLARED"), "and says the coordinate is declared: {msg:?}");
    }
    // THE CONTROL, AND IT ASSERTS A CLEAN RETURN RATHER THAN AN ABSENT SUBSTRING — *"the refusal
    // did not fire"* is satisfied by an unrelated abort just as well as by a passing march.
    for coord in LAG_COORDS_DECLARED {
        let m = demand(&valve_arm());
        m.fuel.inner.lag_coord.set(coord);
        assert_eq!(message_of(|| { bare_march(&m); }), "",
                   "{coord:?} is declared: the bare march must RETURN, not merely miss the \
                    refusal — and note that even `demand` returns here, because this arming \
                    takes the entry test's early return");
    }
}

/// **THE OTHER FOUR FIRE ONLY BELOW THE ENTRY TEST**, each with its `clip` twin as the control.
///
/// Every row drives the SAME arming twice, changing only the coordinate. Under `demand` the
/// refusal is rung 74's; under `clip` the call dispatches out and whatever fires is the parent's —
/// so a port that hoisted these four would show `rung-74` in the control column and be caught.
///
/// The control is written as *the message does NOT name rung 74* rather than as *no message*,
/// because three of these four armings are refused by rung 72 as well and asserting silence would
/// be asserting the wrong thing.
#[test]
fn the_other_four_refusals_fire_only_below_it() {
    // (1) THE `sum` COMPOSITION. Under `clip` with a SCHEDULED reference this is rung 72's own
    //     isolation instrument and is admissible, so the control asserts a clean return.
    let m = demand(&valve_arm());
    m.fuel.inner.share_law.set("sum");
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let a = message_of(|| { armed_march(&m, Some(TT4_MAX), Some(TAU_GOV)); });
    assert!(a.contains("rung-74") && a.contains("`sum` reading"),
            "the DEMAND x SUM conjunction is refused by name: {a:?}");
    let m2 = demand(&valve_arm());
    m2.fuel.inner.share_law.set("sum");
    m2.fuel.inner.ref_law.set("sched");
    assert_eq!(message_of(|| { armed_march(&m2, Some(TT4_MAX), Some(TAU_GOV)); }), "",
               "THE CONTROL — under `clip` the same arming is rung 72's `sum` and is allowed");

    // (2) A GOVERNOR CLOCK WITH NO SET POINT.
    let m = demand(&valve_arm());
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let b = message_of(|| { armed_march(&m, None, Some(TAU_GOV)); });
    assert!(b.contains("rung-74") && b.contains("no set point"),
            "the inherited set-point refusal is rung 74's here: {b:?}");
    let m2 = demand(&valve_arm());
    let b2 = message_of(|| { armed_march(&m2, None, Some(TAU_GOV)); });
    assert!(!b2.contains("rung-74") && b2.contains("rung-72"),
            "THE CONTROL — under `clip` the SAME refusal is raised by the PARENT: {b2:?}");

    // (3) RUNGS 50/51's FORCED RELEASE EDGE.
    let m = demand(&valve_arm());
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let c = message_of(|| { forced_release_drive(&m); });
    assert!(c.contains("rung-74") && c.contains("FORCED release"), "{c:?}");
    let m2 = demand(&valve_arm());
    let c2 = message_of(|| { forced_release_drive(&m2); });
    assert!(!c2.contains("rung-74") && c2.contains("rung-72"),
            "THE CONTROL — the parent's, under `clip`: {c2:?}");

    // (4) AN INSTANTANEOUS VALVE BESIDE LAGGED FUEL-SIDE LEGS.
    let m = demand(&bare_valve_arm());
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let d = message_of(|| { armed_march(&m, Some(TT4_MAX), Some(TAU_GOV)); });
    assert!(d.contains("rung-74") && d.contains("INSTANTANEOUS valve"), "{d:?}");
    let m2 = demand(&bare_valve_arm());
    let d2 = message_of(|| { armed_march(&m2, Some(TT4_MAX), Some(TAU_GOV)); });
    assert!(!d2.contains("rung-74") && d2.contains("rung-72"),
            "THE CONTROL — the parent's, under `clip`: {d2:?}");
}

/// **A LEGAL DEMAND MARCH REACHES THIS RUNG's OWN MARCH — which does not exist yet, by design.**
///
/// Every refusal has passed at this point, so the only two things that can happen are *rung 74's
/// march runs* and *the call silently became rung 73's*. The second is the most dangerous defect
/// this step could ship — it would pass every reduce gate in the crate, because the reduce IS
/// *rung 74 under `clip` is rung 73* — so the arm panics by name and this gate asserts it is
/// reached.
///
/// At step 3 this gate becomes the entry point of the real march and the assertion moves from
/// *reached* to *marched*; it is not an expiry, it is the same claim with a body behind it.
#[test]
fn a_legal_demand_march_reaches_the_unimplemented_march() {
    for coord in [LAG_COORD_DEMAND, LAG_COORD_LATCHED] {
        let m = demand(&valve_arm());
        m.fuel.inner.lag_coord.set(coord);
        let msg = message_of(|| { armed_march(&m, Some(TT4_MAX), Some(TAU_GOV)); });
        assert!(msg.contains("_integrate_fuel_demand") && msg.contains("step 3"),
                "{coord:?}: the demand arm is REACHED and says where its body lands: {msg:?}");
        assert!(msg.contains("would pass every reduce gate"),
                "and says why it is a panic rather than a delegation: {msg:?}");
    }
    // THE CONTROL — the SAME arming under `clip` marches cleanly all the way through rung 73.
    let m = demand(&valve_arm());
    assert_eq!(message_of(|| { armed_march(&m, Some(TT4_MAX), Some(TAU_GOV)); }), "",
               "P3 — `clip` is rung 73 by NOT ENTERING, so the reduce is a dispatch and this \
                arming must run to completion");
}

// =============================================================================================
// 6 — THE FLOOR: THE MESSAGE IS THE ENTIRE CELL
// =============================================================================================

/// **THE CONDITION IS RUNGS 72/73/74's CHARACTER FOR CHARACTER, SO THE GATE IS ON THE MESSAGE.**
///
/// All three cells are `ds * rate <= 2.0`, so no argument separates them and the shipped Python
/// needle is worse than useless: § 5.29 (vii) measured `"FOUR actuator states"` reaching nine
/// classes back to rung 43, and all three of these messages carry it. The tokens that discriminate
/// are `rung-74` and this rung's own argument — *removing the state floor makes a dormant leg an
/// ACTIVE lag* — and the negative half asserts that neither parent's argument appears, which is
/// what separates a re-justification from a copy.
#[test]
fn the_floor_is_re_justified_a_seventh_time_and_the_parents_arguments_do_not_appear() {
    let (ds, rate) = (0.05f64, 60.0f64); // ds*rate = 3.0 > 2.0
    let msg = message_of(|| { (R74_TRIPLE.rk4_floor_shared)(ds, rate); });
    assert!(msg.contains("rung-74"), "the message names the rung: {msg:?}");
    assert!(msg.contains("ACTIVE lag"), "and gives THIS rung's argument: {msg:?}");
    assert!(msg.contains("A coordinate does not change a rate"),
            "including why the parents' arguments carry but say nothing new: {msg:?}");
    assert!(!msg.contains("-1/tau_f"), "rung 72's argument is NOT reused: {msg:?}");
    assert!(!msg.contains("origin"), "nor rung 73's: {msg:?}");

    // THE CONDITION IS IDENTICAL — all three fire on the same argument and none fires just under
    // it, so the cell really is the message and nothing else.
    for (name, cell) in [("rung-72", R72_TRIPLE.rk4_floor_shared),
                         ("rung-73", R73_TRIPLE.rk4_floor_shared),
                         ("rung-74", R74_TRIPLE.rk4_floor_shared)] {
        assert!(message_of(|| { cell(ds, rate); }).contains(name),
                "{name} fires at ds*rate = 3.0");
        assert_eq!(message_of(|| { cell(0.05, 40.0); }), "",
                   "{name} does NOT fire at ds*rate = 2.0 — the boundary is `<=`, in all three");
    }
}

// =============================================================================================
// 7 — THE UNFLOORED CAP: TWO REGIMES, TWO CONSTANTS, AND ONE ABORT
// =============================================================================================

/// **THE BINDING REGIME RETURNS THE SHIPPED SOLVE BITWISE AND NEVER BRACKETS.**
///
/// `G(mf_sched) > 0` means the leg must cut, which is every point at which this family has ever
/// consulted a cap — so on those points nothing is re-bracketed and rung 74 reads the family's own
/// number. The residual and the fallback are supplied BY THIS TEST, not by the code under test
/// (§ 5.30 (viii) item 1), so the gate cannot be satisfied by the plant agreeing with itself.
#[test]
fn cap_free_short_circuits_when_the_leg_binds_and_solves_when_it_is_slack() {
    // BINDING — `G(mf) = +1 > 0`, so the shipped value comes straight back and the residual is
    // called EXACTLY ONCE.
    let calls = std::cell::Cell::new(0usize);
    let out = cap_free(
        &|_w| { calls.set(calls.get() + 1); Ok(1.0) },
        0.02,
        &|| Ok(0.1234567890123456),
    ).expect("the binding arm cannot fail");
    assert_eq!(out.to_bits(), 0.1234567890123456f64.to_bits(),
               "the SHIPPED solve is returned untouched, bit for bit");
    assert_eq!(calls.get(), 1, "and the bracket walk never ran");

    // SLACK — a residual with a root strictly above `mf_sched`, so the walk runs and Illinois
    // converges on it. `G(w) = w - 0.05`, root `0.05`, and `mf_sched = 0.02` gives `G < 0`.
    let out = cap_free(&|w| Ok(w - 0.05), 0.02, &|| unreachable!("the leg is SLACK"))
        .expect("the root is inside the bracket");
    assert!((out - 0.05).abs() < 1e-12, "the unfloored cap is the root, not the schedule: {out}");
    assert!(out > 0.02, "AND IT IS ABOVE THE SCHEDULE — which is the whole point of the cell");
}

/// **THE TWO BRACKET CONSTANTS, MEASURED THROUGH THE BEHAVIOUR RATHER THAN RESTATED.**
///
/// A gate that asserted `CAP_GROW == 1.0 / 0.9` would compute the value under test from the same
/// expression that produced it — § 5.30 (viii) item 1's void. So the residual RECORDS every
/// abscissa it is handed, and the gate reads the geometric ratio and the walk length off that log.
///
/// `n = 60` is asserted because a probe in this slice's own pre-flight typed **40** against the
/// shipped 60 (§ 5.30 (vii), fourth defect) — a walk that merely *passed* with the wrong `n` would
/// have measured a different bracket and reported it as the shipped one.
#[test]
fn the_bracket_walk_is_geometric_in_one_over_zero_point_nine_and_sixty_steps_long() {
    let seen: std::cell::RefCell<Vec<f64>> = std::cell::RefCell::new(Vec::new());
    // A residual that is NEGATIVE everywhere, so the walk can never bracket and runs to its cap.
    let err = cap_free(
        &|w| { seen.borrow_mut().push(w); Ok(-1.0) },
        0.02,
        &|| unreachable!("the leg is SLACK"),
    ).expect_err("a residual with no sign change must ABORT by name");

    let log = seen.borrow();
    assert_eq!(log.len(), 1 + CAP_BRACKET_N,
               "one probe at `mf_sched` plus exactly `n` grown ones — and `n` is 60, not 40");
    assert_eq!(log[0].to_bits(), 0.02f64.to_bits(), "the first probe is the schedule itself");

    // **THE REFERENCE IS PYTHON's LITERAL, NOT `CAP_GROW`** — and the first writing of this gate
    // got that wrong. It asserted `log[k] == prev * CAP_GROW`, and the mutation setting
    // `CAP_GROW = 1.0 / 0.85` (`_sched_fuel`'s own shrink, the likeliest wrong neighbour)
    // **SURVIVED IT**: the bar was the walk's self-consistency with the constant, not the
    // constant's value. § 5.30 (viii) item 1 — *what supplies the value under test?* — caught in
    // this slice's own instrument by its own mutation sweep.
    const PY_GROW: f64 = 1.0 / 0.9; // `engine.py:17567`, transcribed from the SOURCE
    for k in 1..log.len() {
        let prev = if k == 1 { 0.02 } else { log[k - 1] };
        assert_eq!(log[k].to_bits(), (prev * PY_GROW).to_bits(),
                   "step {k} grows by PYTHON's ratio, bit for bit");
    }
    assert!(CAP_GROW > 1.0, "it GROWS — `_surge_fuel`'s 0.9 shrink is the mirror image");

    // AND THE COMPOUNDED MAGNITUDE, WHICH SEPARATES THE NEIGHBOURS BY ORDERS RATHER THAN BY BITS:
    // (1/0.9)^60 = 557.4, against (1/0.85)^60 = 1.7e4 and (1/0.95)^60 = 21.7. A one-sided
    // "> 500x" bar — which is what this line used to be — passes ALL of the growing ones.
    let total = log[log.len() - 1] / 0.02;
    assert!((550.0..565.0).contains(&total),
            "60 steps of 1/0.9 compound to 557.4x the schedule, got {total}");

    // THE ABORT NAMES ITSELF, AND SAYS WHY IT IS NOT A FALLBACK.
    assert!(err.0.contains("rung-74") && err.0.contains("UNFLOORED cap is unreachable"),
            "{:?}", err.0);
    assert!(err.0.contains("manufacture a dormant-leg cut"),
            "and states why falling back to `mf_sched` would be worse than aborting: {:?}", err.0);
    assert!(err.0.contains("searched to"), "and reports the top of the walk: {:?}", err.0);
}

/// **IT IS AN `Abort` AND NOT A `panic!`, WHICH IS A PROPERTY OF THE CALL SITE.**
///
/// Python's refusal is an `AssertionError` and the march wraps its whole derivative in
/// `except AssertionError: break` (`engine.py:17965`, `17989`), so this failure ENDS THE MARCH
/// where a panic would end the process. Slice L's rule: fallibility is decided per call site.
#[test]
fn the_unreachable_cap_is_recoverable_because_the_march_catches_it() {
    let out: Result<f64, Abort> =
        cap_free(&|_w| Ok(-1.0), 0.02, &|| unreachable!("the leg is SLACK"));
    assert!(out.is_err(), "an `Err`, so a caller can `break` on it");
    assert_eq!(message_of(|| { let _ = cap_free(&|_w| Ok(-1.0), 0.02, &|| Ok(0.0)); }), "",
               "AND IT DOES NOT PANIC — a panic here would end the process, not the march");
}

// =============================================================================================
// 8 — THE DISPATCH THROUGH `sensed_cap`, WHICH IS RUNG 76's SEAT
// =============================================================================================

/// **`_cap_fuel` REACHES `_sensed_cap` THROUGH THE TABLE, NOT BY CALLING RUNG 74's BODY.**
///
/// Python reaches it as `self._sensed_cap(…)`, so an inherited rung-74 reader run on a rung-76
/// machine takes rung 76's body. A port that called this rung's function directly would compile,
/// pass every value gate AT THIS RUNG — rung 74's body returns `None`, which is exactly the branch
/// the solve path takes anyway — and break rung 76 silently. Slice AE recorded the same defect
/// from the other side: a census restricted to direct calls scored `_quad_gains_at` at zero readers
/// when it had eleven call sites.
///
/// The injected table's `sensed_cap` returns a value **no solve on this plant could produce**, so
/// the positive reading is unambiguous; the shipped table is the control, and it is asserted to
/// return `Ok` of something else rather than merely *not* the injected value — an abort would
/// satisfy the weaker form.
#[test]
fn cap_fuel_reaches_sensed_cap_through_the_table() {
    const SENTINEL: f64 = 0.987_654_321;
    fn injected_sensed_cap(
        _: &turbojet::fuel_transient::FuelTransientCore, _: &FlightCondition, _: f64, _: f64,
        _: &AccelSchedule, _: Option<f64>,
    ) -> Result<Option<f64>, Abort> {
        Ok(Some(SENTINEL))
    }
    static INJ: TripleHooks = TripleHooks { sensed_cap: injected_sensed_cap, ..R74_TRIPLE };

    let arm = valve_arm();
    let sched = flat_schedule();

    let m = with_triple(&demand(&arm), &arm, &INJ);
    let got = (R74_TRIPLE.cap_fuel)(&m.fuel, &flight(), 1.0, 1.0, 0.02, Some(&sched), None, None)
        .expect("the injected cap needs no solve at all");
    assert_eq!(got.to_bits(), SENTINEL.to_bits(),
               "the sentinel came back, so the call went THROUGH the receiver's table");

    // THE CONTROL — the shipped table, same arguments. Rung 74's `sensed_cap` returns `None`, so
    // the accel branch takes the set-point solve and lands somewhere else entirely.
    let m0 = demand(&arm);
    let base = (R74_TRIPLE.cap_fuel)(&m0.fuel, &flight(), 1.0, 1.0, 0.02, Some(&sched), None, None)
        .expect("the shipped accel branch must SOLVE, not abort — an abort would satisfy a \
                 weaker control for the wrong reason");
    assert!((base - SENTINEL).abs() > 1e-6,
            "THE INSTRUMENT CAN SEE — the shipped path does not happen to return the sentinel");

    // AND `windup_tau` IS `None` AT THIS RUNG, which is rung 75's hook lying dormant.
    assert!((R74_TRIPLE.windup_tau)(&m0.fuel.inner).is_none(),
            "rung 74 has NO anti-windup device — that is § 4's finding, stated as code");
    assert!((R74_TRIPLE.sensed_cap)(&m0.fuel, &flight(), 1.0, 1.0, &sched, None)
                .expect("rung 74's body cannot fail").is_none(),
            "and no sensed cap — every cap here is a set-point solve");
}

/// A flat `Wf/pt3` schedule, built by hand rather than derived from a march: the gate above is
/// about DISPATCH, and a derived schedule would make it depend on the running line as well.
fn flat_schedule() -> AccelSchedule {
    AccelSchedule { margin: 0.0, n_h: vec![0.5, 1.5], kappa: vec![1e-8, 1e-8] }
}

/// Rebuild a machine with an injected third-loop table. `slice_ae_dispatch.rs`'s `with_tables`,
/// narrowed to the one table this file injects into.
fn with_triple(
    core: &ScheduledStatorCore, arm: &LeverArm, triple: &'static TripleHooks,
) -> ScheduledStatorCore {
    let c = full_of(ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        arm.stator, &R74_TWO, &R74_STATOR, &R74_FUEL, &R74,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        triple, arm.stator_lim, arm.stator_inc));
    c.fuel.inner.ref_law.set(REF_LAW_APPLIED);
    c.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
    c
}

// =============================================================================================
// 9 — THE CONSTANTS THIS RUNG DECLARES
// =============================================================================================

/// The three coordinates are three DISTINCT strings and the composition law is untouched.
///
/// This is a spelling pin and is labelled as one: it catches a literal that drifted between the
/// constant and the refusal's admitted list, which is the only failure a constant-to-constant
/// comparison can catch. The behavioural content of these names is gated by § 5's refusals.
#[test]
fn the_declared_coordinates_are_three_distinct_names_and_the_share_law_is_untouched() {
    assert_ne!(LAG_COORD_CLIP, LAG_COORD_DEMAND);
    assert_ne!(LAG_COORD_DEMAND, LAG_COORD_LATCHED);
    assert_ne!(LAG_COORD_CLIP, LAG_COORD_LATCHED);
    assert!(LAG_COORD_LATCHED.starts_with(LAG_COORD_DEMAND),
            "`demand-latched` is the demand plant plus a latch, and the names say so");
    assert_eq!(SHARE_LAW_DEFAULT, "max",
               "the coordinate is a THIRD declared law beside rung 72's composition, which this \
                rung does not move");
    assert_eq!(IC_CAP_DECLARED, 60);
}
