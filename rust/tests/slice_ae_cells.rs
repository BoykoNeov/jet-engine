//! SLICE AE step 1 — **SIX RE-AIMED POINTERS, A CLASS DEFAULT THAT WOULD HAVE PASSED ITS OWN
//! REFUSAL, AND THE REFUSAL THAT MAKES A NAME REUSE LOUD.**
//!
//! § 5.19 (x)'s rule for phase 7 is *step 1 of every slice is the cell addition*. This slice adds
//! **no `TripleHooks` field at all** (§ 5.29 (ix) P7, after its own repair), so what step 1 buys
//! here is the other half of that rule: **six slots re-aimed, and the two places where doing that
//! correctly is invisible to every value key in the crate.**
//!
//! # THE TWO SILENT FAILURES THIS FILE EXISTS TO CATCH
//!
//! **1 — THE CLASS DEFAULT.** The core's constructor writes `ref_law = "sched"` for every rung in
//! the family. Python declares `_ref_law = "applied"` at rung 73. A port that forgot the overwrite
//! hands back a machine that **passes** [`the refusal`](refuses_an_undeclared_reference) — because
//! `"sched"` is one of the two declared laws — marches rung 72's plant, and reports rung 73 in
//! every reader. There is no panic, no wrong type, and no shipped rung-73 gate that would notice;
//! the reduce arm below would go on passing, because the reduce is *rung 73 under `"sched"` IS
//! rung 72*. Gated by [`the_class_default_is_applied_and_rung_72s_is_not`], with rung 72's own
//! default as the control.
//!
//! **2 — THE NAME REUSE.** `_with_ref` is rung 69's name and rung 73's, same arity, different
//! MUTATED FIELD (`_ref` against `_ref_law`). Both fields exist on a rung-73 machine, so nothing
//! type-errors — which is why § 5.27 (x)'s phase-wide sweep, comparing SIGNATURES, filed the pair
//! as harmlessly renamed. The gate has to be **two-sided or it measures nothing**: a body that
//! wrote BOTH fields passes *"`ref_law` moved"*. See
//! [`with_ref_moves_ref_law_and_leaves_ref_alone`].
//!
//! # WHAT THIS FILE DELIBERATELY DOES NOT GATE
//!
//! **The DISPATCH of the six swaps through a rung-69 reader.** Step 1 drives the refusal
//! DIRECTLY — write the field by hand, march, read the message. Step 5 drives it through a real
//! inherited reader on a rung-73 machine, which is the manufactured pairing § 5.29 (ix) P1 names
//! as the shippable defect. They are not the same gate and neither replaces the other.
//!
//! **`_shared_rig`'s carrying, as a VALUE break.** Probe L2 measured the override a NO-OP — rung
//! 72's body reaches its sibling through `at_lever`, which already carried the law — so the gate
//! below asserts the OUTCOME (the rig carries the law) and the module header records that no
//! discriminator exists. Pre-registered so step 5 does not hunt one.
//!
//! **NO GATE HERE HAS AN EXPIRY DATE.** Slice AB's step-1 mistake was an *"every slot panics"*
//! gate that had to be dismantled when the bodies landed. Everything below survives step 2: the
//! `R73_TRIPLE` destructuring will go loud when the struct reaches 14 fields, which is the
//! tripwire working and not a gate breaking.
//!
//! **NOTHING HERE READS A GOLDEN.** Every assertion is a panic, a same-run difference, or a
//! compile-time property.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;

use turbojet::applied_reference::{
    build_applied_reference_cascade, REF_LAWS_DECLARED, REF_LAW_APPLIED, R73, R73_FUEL,
    R73_STATOR, R73_TRIPLE, R73_TWO,
};
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{
    build_reference_split_cascade, RefScope, StatorIncidenceLimiter,
};
use turbojet::shared_actuator::{
    build_shared_actuator_cascade, SharedRigArm, REF_LAW_DEFAULT, R72_TRIPLE,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::TripleHooks;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung73.py`'s module constants, verbatim. This slice adds no constant of its own and
// this file must not be the place one appears.
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
const V_MAX: f64 = 0.20;
/// `PHI / FLOOR - 1.0` — the expression Python spells, never a typed decimal: rung 69's
/// constructor asserts `m_lim == T_c - 1/phi_lim`, so a rounded constant breaks a wall identity.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
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
fn inc_stator() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))
}
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

/// Python's `_applied(design, **kw)`.
fn applied(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_applied_reference_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Python's `SharedActuatorTransient(design, …)` — rung 72, the control at every step below.
fn shared(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_shared_actuator_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Rung 69 — the machine whose `_with_ref` writes the OTHER field.
fn split69(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_reference_split_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

fn valve_arm() -> LeverArm { LeverArm { bleed_lim: Some(valve()), ..Default::default() } }

fn inc_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_inc: Some(inc_stator()), ..Default::default() }
}

/// The panic message a closure produces, or `""` if it did not panic. Slice AB/AC/AD's helper
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

// =============================================================================================
// 1 — THE SIX RE-AIMED POINTERS, AND THE NINE THAT ARE STILL RUNG 72's
// =============================================================================================

/// The six swaps are DIFFERENT function pointers from rung 72's, and the equality control rides
/// beside each inequality — a gate that would pass for a broken instrument fails visibly instead
/// ([[rust-port-slice-aa-step1]]: never `ptr::eq` on the `const` itself).
#[test]
fn the_slice_re_aims_exactly_six_pointers_and_inherits_the_rest() {
    // INEQUALITY — the four in the third-loop table.
    assert!(!fn_addr_eq(R73_TRIPLE.with_ref, R72_TRIPLE.with_ref),
            "swap 1 of 6 — `_with_ref`, the NAME REUSE: rung 69's slot pointed at `ref_law`");
    assert!(!fn_addr_eq(R73_TRIPLE.reference, R72_TRIPLE.reference),
            "swap 2 of 6 — `_reference`, reading B");
    assert!(!fn_addr_eq(R73_TRIPLE.rk4_floor_shared, R72_TRIPLE.rk4_floor_shared),
            "swap 3 of 6 — `_rk4_floor_shared`, the message IS the cell");
    assert!(!fn_addr_eq(R73_TRIPLE.shared_rig, R72_TRIPLE.shared_rig),
            "swap 4 of 6 — `_shared_rig`, carrying the reference");

    // INEQUALITY — the two in the other tables.
    assert!(!fn_addr_eq(R73.at_lever, turbojet::shared_actuator::R72.at_lever),
            "swap 5 of 6 — `at_lever`, the sibling constructor");
    assert!(!fn_addr_eq(R73_FUEL.integrate_fuel,
                        turbojet::shared_actuator::R72_FUEL.integrate_fuel),
            "swap 6 of 6 — `integrate_fuel`, the two refusals");

    // EQUALITY CONTROL — the nine inherited third-loop cells ARE rung 72's, so the instrument can
    // tell the two answers apart on the same table.
    assert!(fn_addr_eq(R73_TRIPLE.stator_leg, R72_TRIPLE.stator_leg));
    assert!(fn_addr_eq(R73_TRIPLE.lagged_stator, R72_TRIPLE.lagged_stator));
    assert!(fn_addr_eq(R73_TRIPLE.clamp_v, R72_TRIPLE.clamp_v));
    assert!(fn_addr_eq(R73_TRIPLE.check_v0, R72_TRIPLE.check_v0));
    assert!(fn_addr_eq(R73_TRIPLE.rk4_floor, R72_TRIPLE.rk4_floor));
    assert!(fn_addr_eq(R73_TRIPLE.solve_v, R72_TRIPLE.solve_v));
    assert!(fn_addr_eq(R73_TRIPLE.manifold_v, R72_TRIPLE.manifold_v));
    assert!(fn_addr_eq(R73_TRIPLE.triple_laws, R72_TRIPLE.triple_laws));
    assert!(fn_addr_eq(R73_TRIPLE.triple_rig, R72_TRIPLE.triple_rig));

    // AND THE TWO ALIAS TABLES CARRY NO SWAP AT ALL — spelled as exhaustive destructurings so a
    // future field is a compile error here rather than a silent pass.
    let turbojet::two_spool_transient::TwoSpoolTransientHooks {
        try_close: _, try_instant_tail: _, powers: _,
    } = R73_TWO;
    let turbojet::stator_transient::StatorTransientHooks {
        stator_march: _, v_of: _, arm: _, at_stator: _,
    } = R73_STATOR;
    assert!(fn_addr_eq(R73_TWO.try_close, turbojet::shared_actuator::R72_TWO.try_close));
    assert!(fn_addr_eq(R73_STATOR.stator_march,
                       turbojet::shared_actuator::R72_STATOR.stator_march));

    // And every OTHER lever/fuel cell is rung 72's, so "exactly six" is a measurement.
    assert!(fn_addr_eq(R73.b_at_point, turbojet::shared_actuator::R72.b_at_point));
    assert!(fn_addr_eq(R73_FUEL.try_close_fuel,
                       turbojet::shared_actuator::R72_FUEL.try_close_fuel));
    assert!(fn_addr_eq(R73_FUEL.try_surge_fuel,
                       turbojet::shared_actuator::R72_FUEL.try_surge_fuel));
}

/// **`TripleHooks` IS 14 AFTER STEP 2 — and the count is spelled as an exhaustive destructuring,
/// which is the only form that fails when the struct grows.**
///
/// § 5.29 (ix)'s P7 was FALSIFIED before step 1 (it had said 13 → 14) and restated as *no new
/// field at all*. It holds for step 1's six pointers, every one of which re-aims an existing slot.
///
/// **AND IT IS ALREADY KNOWN FALSE FOR THE SEVENTH.** § 5.29 (iv) commits this slice to installing
/// `_quad_gains_at` as a cell, and that name has **no field in any of the five table types** — it
/// was a free `pub fn` in `shared_actuator`. So step 2 takes this struct to 14 and this gate is
/// the tripwire that said so — **it went `E0027` on the step-2 build, and the field below is the
/// discharge.** Recorded at step 1 rather than met as a surprise, because P7's repair
/// fixed one of the two inconsistencies it was written against and left the other — § 5.29 (x)'s
/// sixth defect, a second time inside the same section.
#[test]
fn the_third_loop_table_is_fourteen_fields_wide() {
    let TripleHooks {
        stator_leg: _, lagged_stator: _, clamp_v: _, check_v0: _, rk4_floor: _, solve_v: _,
        manifold_v: _, triple_laws: _, triple_rig: _, with_ref: _, reference: _,
        rk4_floor_shared: _, shared_rig: _,
        // THE FOURTEENTH, ADDED AT STEP 2 EXACTLY AS THIS GATE'S DOC COMMENT PREDICTED. The
        // destructuring went `E0027` the moment the field landed; the prediction was written at
        // step 1 and is discharged here rather than met as a surprise.
        quad_gains_at: _,
    } = R73_TRIPLE;
}

// =============================================================================================
// 2 — THE CLASS DEFAULT, WHICH IS THE SILENT FAILURE THIS STEP EXISTS TO PREVENT
// =============================================================================================

/// **A FRESH RUNG-73 MACHINE READS `"applied"`, AND A FRESH RUNG-72 ONE READS `"sched"`.**
///
/// The core's constructor writes `"sched"` for the whole family, so the overwrite in
/// `build_applied_reference_cascade` is load-bearing and nothing else in the crate would notice
/// its absence: `"sched"` PASSES rung 73's own refusal, and the reduce arm below is *rung 73 under
/// `"sched"` IS rung 72* — it would go on passing while every reader measured the parent.
///
/// The rung-72 control is what makes this a measurement rather than a restatement of the constant:
/// without it, a builder that set `"applied"` on EVERY machine in the family would pass.
///
/// Measured on the source first (probe L1), so the expected values are read off Python and not
/// off this port.
#[test]
fn the_class_default_is_applied_and_rung_72s_is_not() {
    assert_eq!(applied(&valve_arm()).fuel.inner.ref_law.get(), REF_LAW_APPLIED,
               "rung 73 declares `_ref_law = 'applied'` as a class attribute");
    assert_eq!(shared(&valve_arm()).fuel.inner.ref_law.get(), REF_LAW_DEFAULT,
               "THE CONTROL — rung 72 declares `'sched'`, and a builder that set `'applied'` \
                everywhere would pass the assertion above");
    assert_ne!(REF_LAW_APPLIED, REF_LAW_DEFAULT,
               "the two class attributes are two constants because they are two declarations");
    assert!(REF_LAWS_DECLARED.contains(&REF_LAW_APPLIED)
            && REF_LAWS_DECLARED.contains(&REF_LAW_DEFAULT),
            "the refusal's admitted pair is exactly the two class attributes");
}

// =============================================================================================
// 3 — THE NAME REUSE: WHICH FIELD IS WRITTEN, GATED FROM BOTH SIDES
// =============================================================================================

/// **THE DISCRIMINATOR § 5.27 (x)'s SIGNATURE SWEEP COULD NOT REACH, AND IT IS TWO-SIDED.**
///
/// Rung 69's `_with_ref` writes `_ref`; rung 73's writes `_ref_law`. Same arity, same shape, both
/// fields present on both machines. A one-sided assertion (*"`ref_law` moved"*) is passed by a
/// body that writes BOTH, which is exactly the failure mode a re-aim invites — so each arm asserts
/// the field that MOVED **and** the field that did not.
///
/// § 5.29 (v)'s sentinel lesson applied to this file's own instrument: the untouched field is
/// checked against a value it could not already have been sitting at by accident — `ref_` starts
/// `None` on both machines and `ref_law` starts at a KNOWN class default, both asserted in
/// [`the_class_default_is_applied_and_rung_72s_is_not`] before this gate reads them.
#[test]
fn with_ref_moves_ref_law_and_leaves_ref_alone() {
    // RUNG 73 — the re-aimed slot.
    let m73 = applied(&valve_arm());
    let c73: &TwoSpoolTransientCore = &m73.fuel.inner;
    assert_eq!(c73.ref_.get(), None, "the untouched field's starting value, stated");
    let prev = (R73_TRIPLE.with_ref)(c73, Some("inc"));
    assert_eq!(prev, Some(REF_LAW_APPLIED), "it hands back what it DISPLACED, Python's `prev`");
    assert_eq!(c73.ref_law.get(), "inc", "rung 73 writes `_ref_law` …");
    assert_eq!(c73.ref_.get(), None, "… and NOTHING ELSE — the half a one-sided gate misses");

    // RUNG 69 — the CONTROL, the same call on the machine whose body writes the other field.
    let m69 = split69(&inc_arm());
    let c69: &TwoSpoolTransientCore = &m69.fuel.inner;
    assert_eq!(c69.ref_law.get(), REF_LAW_DEFAULT, "the untouched field's starting value");
    let prev69 = (turbojet::reference_split::R69_TRIPLE.with_ref)(c69, Some("inc"));
    assert_eq!(prev69, None, "rung 69's `_ref` starts unset, so `prev` is `None`");
    assert_eq!(c69.ref_.get(), Some("inc"), "rung 69 writes `_ref` …");
    assert_eq!(c69.ref_law.get(), REF_LAW_DEFAULT, "… and leaves `_ref_law` where it was");
}

/// **THE SHARED GUARD RESTORES THROUGH THE RE-AIMED CELL**, which is the reason `RefScope` was
/// written against `TripleHooks::with_ref` and never against `core.ref_.set(prev)`.
///
/// A manufactured NEST, for `RefScope`'s own stated reason: restore-previous and restore-to-default
/// agree on every shipped path (`_ref_law` is set to a value and back, never nested, in the whole
/// rung-73 suite), so only a nest can tell the two policies apart.
#[test]
fn ref_scope_nests_and_restores_the_previous_law_not_the_default() {
    let m = applied(&valve_arm());
    let c = &m.fuel.inner;
    {
        let outer = RefScope::set(c, Some("sched"));
        assert_eq!(outer.displaced(), Some(REF_LAW_APPLIED));
        assert_eq!(c.ref_law.get(), "sched");
        {
            let inner = RefScope::set(c, Some(REF_LAW_APPLIED));
            assert_eq!(inner.displaced(), Some("sched"), "the nest's `prev` is the OUTER value");
            assert_eq!(c.ref_law.get(), REF_LAW_APPLIED);
        }
        // RESTORE-PREVIOUS: a restore-to-default would read `"applied"` here and pass by accident
        // on every shipped path, which is why the nest exists at all.
        assert_eq!(c.ref_law.get(), "sched", "the inner scope restored the OUTER law");
    }
    assert_eq!(c.ref_law.get(), REF_LAW_APPLIED, "and the outer scope restored the class default");
    assert_eq!(c.ref_.get(), None, "through all of it, rung 69's field never moved");
}

/// **`None` IS REFUSED, AND THE REFUSAL IS DRIVEN RATHER THAN LEFT AS A DEFENCE WITH NO READER.**
///
/// The shared guard's parameter is `Option<&'static str>` because rung 69's field is optional;
/// rung 73's is a plain `str` with no unset state. Python would assign `None` and raise out of
/// `integrate_fuel` (*"got None"*); the port refuses at the setter, because `Cell<&'static str>`
/// has nowhere to put it.
///
/// **THE REACHABILITY IS THE POINT.** `r73_with_ref` always returns `Some(prev)`, so `RefScope`'s
/// `Drop` can never feed `None` back in and this panic can never land inside an unwind. That is
/// what makes it admissible; [[rust-port-slice-aa-steps2345]]'s *defence with no reader* is why it
/// is driven here anyway.
#[test]
fn with_ref_refuses_none_by_name_on_a_rung_73_machine() {
    let m = applied(&valve_arm());
    let msg = message_of(|| { (R73_TRIPLE.with_ref)(&m.fuel.inner, None); });
    assert!(msg.contains("rung-73"), "the refusal names the rung: {msg:?}");
    assert!(msg.contains("NAME REUSE"), "and says WHY the two cells differ: {msg:?}");
    // THE CONTROL — rung 69's body takes `None` happily, which is what makes the refusal a
    // property of this rung and not of the argument.
    let m69 = split69(&inc_arm());
    assert_eq!(message_of(|| { (turbojet::reference_split::R69_TRIPLE.with_ref)(
                   &m69.fuel.inner, None); }),
               "", "rung 69's `_ref` HAS an unset state, so `None` is a real assignment there");
}

// =============================================================================================
// 4 — READING B: THE THREE PATHS, EVERY ONE ON `to_bits`
// =============================================================================================

/// **ALL THREE PATHS, COMPARED BIT FOR BIT — AND PATH 2 IS DRIVEN AT A TRIPLE WHERE FOLDING THE
/// BRANCH AWAY ACTUALLY MOVES THE ANSWER.**
///
/// § 5.29 (ix) P5 says an injection that folds the float-identity branch away is invisible to
/// every RELATIVE bar in the crate. It is also invisible to an EXACT-bits gate driven at the wrong
/// numbers, and that is not a hypothetical: the first version of this gate used probe L3's
/// `(req, g_own, gf, gr) = (3.5, 2.0, 2.0, 1.0)` for all three paths, and **the mutation that
/// deletes path 2 SURVIVED it** — at those magnitudes `(2.0 + 3.5) - 2.0` is `3.5` exactly, so the
/// deleted branch and the shipped one agree bitwise and the gate could not tell them apart.
///
/// [[rust-port-slice-w-step3]]'s rule, on this file's own instrument: **make the instrument prove
/// it can SEE.** Path 2 is therefore driven at `(0.3, 0.1, 0.1, 0.05)`, measured against the
/// source: the shipped branch returns `0.3` and the fold-away returns `0.30000000000000004`, a
/// relative gap of `1.85e-16`. The benign triple is kept beside it and labelled, because a reader
/// who reaches for it needs to know it discriminates nothing.
#[test]
fn reference_takes_three_paths_and_two_of_them_return_req_bit_for_bit() {
    let m = applied(&valve_arm());
    let c = &m.fuel.inner;
    let req = 3.5f64;

    // PATH 1 — the dispatch. `_ref_law != "applied"` returns `req` before the clip is even formed.
    {
        let _r = RefScope::set(c, Some("sched"));
        let out = (R73_TRIPLE.reference)(c, req, 2.0, 2.0, 1.0);
        assert_eq!(out.to_bits(), req.to_bits(), "path 1 is the identity, bitwise");
    }

    // PATH 2, AT THE DISCRIMINATING TRIPLE — the leg HOLDS (`clip == max(0.1, 0.05) == g_own`), so
    // `req` ITSELF comes back. This is the assertion that kills the fold-away.
    let (rq2, g2, gr2) = (0.3f64, 0.1f64, 0.05f64);
    let out2 = (R73_TRIPLE.reference)(c, rq2, g2, g2, gr2);
    assert_eq!(out2.to_bits(), rq2.to_bits(),
               "path 2 is the FLOAT-IDENTITY device: it returns `req` and not `g_own+req-clip`");
    assert_ne!(out2.to_bits(), ((g2 + rq2) - g2).to_bits(),
               "AND THE INSTRUMENT CAN SEE — at this triple the fold-away is a DIFFERENT float,                 which is what the first version of this gate lacked");

    // PATH 2 AT THE BENIGN TRIPLE, KEPT AND LABELLED. It passes whatever the cell does, because
    // `(2.0 + 3.5) - 2.0` is `3.5` exactly. Not a gate — a disclosed control.
    let out2b = (R73_TRIPLE.reference)(c, req, 2.0, 2.0, 1.0);
    assert_eq!(out2b.to_bits(), req.to_bits());
    assert_eq!(((2.0f64 + req) - 2.0).to_bits(), req.to_bits(),
               "and here the fold-away AGREES, which is why this triple discriminates nothing");

    // PATH 3 — the leg is MASKED: `g_own = gf = 1.0`, `gr = 2.0`, so `clip = 2.0 != g_own`.
    let out3 = (R73_TRIPLE.reference)(c, req, 1.0, 1.0, 2.0);
    assert_ne!(out3.to_bits(), req.to_bits(), "path 3 must NOT return `req` — 0 of 109 307 do");
    assert_eq!(out3.to_bits(), ((1.0f64 + req) - 2.0).to_bits(), "and it is Python's 2.5");

    // AND RUNG 72's CELL IS THE IDENTITY ON THE SAME MASKED TRIPLE — the parent control, so the
    // three paths above are a property of this rung's body and not of the arguments.
    let m72 = shared(&valve_arm());
    assert_eq!((R72_TRIPLE.reference)(&m72.fuel.inner, req, 1.0, 1.0, 2.0).to_bits(),
               req.to_bits());
}

/// **THE ASSOCIATION IS PINNED: `(g_own + req) - clip`, NEVER `req + (g_own - clip)`.**
///
/// The two are the same algebra and different floats, and the rearrangement is exactly what a
/// later tidy-up writes. Measured on the source (probe L4): at `g_own = 1e16, req = 1.0,
/// clip = 1e16` Python's order gives `0.0` and the rearrangement `1.0` — a full unit apart, on a
/// gate that no relative bar would need.
///
/// The `clip` is reached by arming `gr` above `gf`, so this drives the SHIPPED path 3 rather than
/// calling an arithmetic helper: a gate on a re-spelled expression would pass whatever the cell
/// does.
#[test]
fn path_threes_association_is_pythons_and_the_rearrangement_would_show() {
    let m = applied(&valve_arm());
    let c = &m.fuel.inner;
    let (g_own, req, clip) = (1e16f64, 1.0f64, 1e16f64);
    // `gf = g_own`, `gr = clip` with `gr > gf` is false here (they are equal), so drive the
    // masked leg the other way round: `g_own` is the FUEL leg's state and `gr` holds.
    let out = (R73_TRIPLE.reference)(c, req, g_own, g_own, clip + 2.0);
    assert_eq!(out.to_bits(), ((g_own + req) - (clip + 2.0)).to_bits(),
               "Python's association");
    assert_ne!(out.to_bits(), (req + (g_own - (clip + 2.0))).to_bits(),
               "and the rearrangement a tidy-up writes is a DIFFERENT float: -2.0 against -1.0");
}

// =============================================================================================
// 5 — THE TWO REFUSALS, DRIVEN DIRECTLY (step 5 drives them through a real inherited reader)
// =============================================================================================

/// **THE REFUSAL THAT MAKES THE NAME REUSE LOUD** — § 5.29 (ix) P2, and the whole reason the
/// re-aim is a correct port rather than a shippable defect.
///
/// Without it, a caller that wrote `"inc"` into `ref_law` would march with `ref_` still unset,
/// fall through `triple_rig`'s `self._ref or (…)` fallback and produce a plant nobody asked for,
/// silently. The field is written BY HAND here — the manufactured pairing through a real rung-69
/// reader is step 5's, and neither gate replaces the other.
///
/// **AND THE ARMING IS DELIBERATELY BARE.** Rung 72's `integrate_fuel` early-returns into rung
/// 71's table when no governor clock and no fuel leg are armed. Python's asserts sit ABOVE the
/// `super()` call and so precede that test — driven on the source (probe L5), `_ref_law = 'inc'`
/// with neither leg armed still raises. Marching with the legs armed would pass even if the
/// refusal had been placed after a copied entry test.
#[test]
fn refuses_an_undeclared_reference() {
    for law in ["inc", "phi", ""] {
        let m = applied(&valve_arm());
        m.fuel.inner.ref_law.set(law);
        let msg = message_of(|| { bare_march(&m); });
        assert!(msg.contains("rung-73"), "the refusal names the rung: {msg:?}");
        assert!(msg.contains(&format!("{law:?}")),
                "and quotes the offending law {law:?}: {msg:?}");
        assert!(msg.contains("DECLARED"), "and says the reference is declared: {msg:?}");
    }
    // THE CONTROL, AND IT ASSERTS A CLEAN RETURN RATHER THAN AN ABSENT SUBSTRING.
    //
    // *"the refusal did not fire"* is satisfied by an UNRELATED abort just as well as by a passing
    // march — and this arming really can abort elsewhere: driven on the source with a hand-built
    // `nu0`, probe L5's two declared rows died on `rung-43 fuel closure does not bracket`. Measured
    // here rather than assumed: through the RAMP both declared laws return cleanly, so the control
    // asserts the empty message and would go red if this arming ever started aborting for a second
    // reason.
    for law in REF_LAWS_DECLARED {
        let m = applied(&valve_arm());
        m.fuel.inner.ref_law.set(law);
        assert_eq!(message_of(|| { bare_march(&m); }), "",
                   "{law:?} is declared: the bare march must RETURN, not merely miss the refusal");
    }
}

/// **THE SECOND REFUSAL: reading B on top of rung 72's SUM composition swaps two declared laws at
/// once**, so `max(gf,gr) == g_own` never holds, the identity branch is never taken, and the
/// result could be attributed to neither law.
///
/// The control is the same arming under `"sched"`, which is rung 72 and is admissible with `sum` —
/// so the gate measures the CONJUNCTION and not merely the presence of `"sum"`.
#[test]
fn refuses_the_applied_reference_on_top_of_the_sum_law() {
    let m = applied(&valve_arm());
    m.fuel.inner.share_law.set("sum");
    let msg = message_of(|| { bare_march(&m); });
    assert!(msg.contains("rung-73") && msg.contains("SUM composition"),
            "the conjunction is refused by name: {msg:?}");
    assert!(msg.contains("change one law at a time"), "rung 63's lesson, quoted: {msg:?}");

    // THE CONTROL — `sum` under the SCHEDULED reference is rung 72 and is allowed. A CLEAN RETURN
    // and not an absent substring, for the reason given on the gate above.
    let m2 = applied(&valve_arm());
    m2.fuel.inner.share_law.set("sum");
    m2.fuel.inner.ref_law.set("sched");
    assert_eq!(message_of(|| { bare_march(&m2); }), "",
               "`sum` is rung 72's own isolation instrument and only the CONJUNCTION is refused");
}

/// A march with **neither the governor clock nor the fuel leg armed** — the arming on which rung
/// 72's body early-returns into rung 71's table. Used by both refusal gates above so that a
/// refusal placed after a copied entry test would be caught.
fn bare_march(m: &ScheduledStatorCore) {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: None, tt4_max: None };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg, &MarchScope::DEFAULT);
}

// =============================================================================================
// 6 — THE FLOOR: THE MESSAGE IS THE CELL, AND THE SHIPPED NEEDLE REACHES NINE CLASSES
// =============================================================================================

/// **THE CONDITION IS RUNG 72's CHARACTER FOR CHARACTER, SO THE GATE IS WRITTEN ON THE ARGUMENT.**
///
/// § 5.29 (vii) re-measured the shipped Python needle over all 58 ladder classes with the names
/// emitted: `"FOUR actuator states"` matches **nine** of them, reaching back to rung 43, so a gate
/// on it discriminates nothing. `rung-73` and `origin` are unique to this class — the second is
/// this rung's whole argument (the masked pole moves to EXACTLY the origin, where rung 72's sat at
/// `-1/tau_f`), so the two messages differ in the one place the physics does.
#[test]
fn the_floor_fires_on_a_message_that_names_this_rung_and_its_own_argument() {
    let msg = message_of(|| { (R73_TRIPLE.rk4_floor_shared)(0.005, 500.0); });
    assert!(msg.contains("rung-73"), "the floor names its own rung: {msg:?}");
    assert!(msg.contains("origin"), "and gives THIS rung's argument, not rung 72's: {msg:?}");
    assert!(!msg.contains("-1/tau_f"),
            "rung 72's argument does NOT carry here — the masked eigenvalue is zero: {msg:?}");

    // THE PARENT CONTROL — the same violation on rung 72's cell gives rung 72's argument.
    let p = message_of(|| { (R72_TRIPLE.rk4_floor_shared)(0.005, 500.0); });
    assert!(p.contains("rung-72") && p.contains("-1/tau_f"),
            "the instrument can tell the two messages apart: {p:?}");
}

/// **AND THE BOUNDARY IS ADMITTED EXACTLY** — `ds * rate <= 2.0`, so `2.0` itself passes and the
/// next float above it does not. A floor gated only on a comfortable violation cannot see an
/// inverted comparison.
#[test]
fn the_floor_admits_its_own_boundary_exactly() {
    assert_eq!(message_of(|| { (R73_TRIPLE.rk4_floor_shared)(1.0, 2.0); }), "",
               "`ds*rate == 2.0` is admitted — the condition is `<=`");
    assert!(!message_of(|| { (R73_TRIPLE.rk4_floor_shared)(1.0, f64::from_bits(
                2.0f64.to_bits() + 1)); }).is_empty(),
            "and one ULP above it is not");
}

// =============================================================================================
// 7 — THE SIBLING CONSTRUCTOR AND THE RIG BOTH CARRY THE REFERENCE
// =============================================================================================

/// **THE TWELFTH INSTANCE OF THE TRAP, WITH A SECOND HEAD.** Handing back the parent's class
/// reports rung 73 while measuring rung 72; handing back the right class while dropping the law
/// does the same thing one level down, in every ledger cell.
///
/// The `"sched"` arm is the one that matters and is the reason the copy lives in the BODY and not
/// only in the builder: a sibling built while the receiver sits under a scope must inherit the
/// SCOPED law, not the class default.
///
/// **`_shared_rig`'s carrying is a MEASURED NO-OP** (probe L2) — rung 72's body reaches its sibling
/// through `at_lever`, which has already copied the law — so this gate asserts the OUTCOME and the
/// module header pre-registers that no value discriminator exists for that swap.
#[test]
fn at_lever_and_the_rig_both_carry_the_reference() {
    let m = applied(&valve_arm());

    // `at_lever` — the class is carried (it dispatches through rung 73's own table) and so is the
    // law.
    let lv = (R73.at_lever)(&m, &inc_arm());
    assert!(fn_addr_eq(lv.triple_hooks().reference, R73_TRIPLE.reference),
            "the sibling is a RUNG-73 machine, not rung 72's");
    assert_eq!(lv.fuel.inner.ref_law.get(), REF_LAW_APPLIED);

    // the rig — same two properties.
    let arm = SharedRigArm { sm: SM, tt4_max: TT4_MAX, inc: true, tau: TAU, tau_s: TAU_S,
                             v_max: V_MAX, ..Default::default() };
    let (rig, _, _) = (R73_TRIPLE.shared_rig)(&m, &arm);
    assert!(fn_addr_eq(rig.triple_hooks().reference, R73_TRIPLE.reference));
    assert_eq!(rig.fuel.inner.ref_law.get(), REF_LAW_APPLIED);

    // AND THE SCOPED LAW REACHES BOTH — the arm the class default would silently pass.
    {
        let _r = RefScope::set(&m.fuel.inner, Some("sched"));
        let lv2 = (R73.at_lever)(&m, &inc_arm());
        let (rig2, _, _) = (R73_TRIPLE.shared_rig)(&m, &arm);
        assert_eq!(lv2.fuel.inner.ref_law.get(), "sched",
                   "a sibling built under a scope inherits the SCOPED law, not the class default");
        assert_eq!(rig2.fuel.inner.ref_law.get(), "sched");
    }
    assert_eq!(m.fuel.inner.ref_law.get(), REF_LAW_APPLIED, "and the receiver is restored");
}

// =============================================================================================
// 8 — THE REDUCE, AND ITS VACUITY CONTROL
//
// `_ref_law = "sched"` makes `_reference` the identity, so the plant is rung 72 BIT FOR BIT. The
// hook is the only thing this rung adds to the march, so this is the arm that says so — and step 1
// is where the hook lands, which is why it is here rather than deferred to step 3's ported gates.
//
// NOT MARKED `slow`, on rung 72's own reasoning: each runs two 341-point marches and is not free,
// but the reduce spine is the project's spine.
// =============================================================================================

/// Python's `_march(m, **kw)` — `_stator_march(FLIGHT, LO, HI, R, SETTLE, DS, **kw)[0]`.
fn march(
    m: &ScheduledStatorCore, floor: Option<Floor>, lg: Option<AsymmetricLag>,
    tt4_max: Option<f64>, tau_gov: Option<f64>,
) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: floor, tt4_max };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg,
                          &MarchScope { lag: lg, tau_gov, ..MarchScope::DEFAULT }).0
}

fn b_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { b, .. } | PointExtra::Triple { b, .. }
        | PointExtra::CrossCascade { b, .. } | PointExtra::Cascade { b, .. }
        | PointExtra::Valve { b, .. } => b,
        _ => panic!("rung-73's `_keys` reads `b` with a bare index"),
    }
}

fn v_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { v, .. } | PointExtra::Triple { v, .. } => v,
        _ => panic!("rung-73's `_keys` reads `v` with a bare index"),
    }
}

/// Python's `_keys(traj)` — the NINE-tuple, compared BIT for bit.
fn keys(traj: &[FuelPoint]) -> Vec<[u64; 9]> {
    traj.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits(),
                  b_of(p).to_bits(), v_of(p).to_bits()])
        .collect()
}

fn four_loop_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    march(m, Some(Floor::Phi(surge())), Some(lag()), Some(TT4_MAX), Some(TAU_GOV))
}

/// **THE SIXTH REDUCE ARM, AND THIS RUNG'S OWN**: under the SCHEDULED reference the hook is the
/// identity, so a rung-73 machine marches rung 72's plant entry for entry.
#[test]
fn reduces_to_rung_72_under_the_scheduled_reference() {
    let m = applied(&inc_arm());
    let a = {
        let _r = RefScope::set(&m.fuel.inner, Some("sched"));
        four_loop_march(&m)
    };
    let b = four_loop_march(&shared(&inc_arm()));
    assert_eq!(keys(&a), keys(&b), "the scheduled reference IS rung 72, bit for bit");
}

/// **AND THE ARM ABOVE MUST BE A TEST, NOT A TAUTOLOGY.**
///
/// If `reference` ignored `ref_law` the reduce would still pass — it would be comparing rung 73
/// with rung 73 — so the same two marches under the APPLIED reference must DIFFER, and differ in
/// the PLANT rather than only in the masked state. That bug is not hypothetical: it is the one the
/// source shipped first, and it returned a PERFECT confirmation of this rung's headline from an
/// instrument that had measured nothing.
#[test]
fn the_scheduled_reduce_is_not_vacuous() {
    let a = four_loop_march(&applied(&inc_arm()));
    let b = four_loop_march(&shared(&inc_arm()));
    assert_ne!(keys(&a), keys(&b), "the APPLIED reference is a different plant");
    let worst = a.iter().zip(b.iter()).map(|(x, y)| (x.tt4 - y.tt4).abs())
                 .fold(0.0f64, f64::max);
    assert!(worst > 1.0, "and it differs in the PLANT, not only in the masked state: {worst}");
}

