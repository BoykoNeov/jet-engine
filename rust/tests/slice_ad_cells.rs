//! SLICE AD step 1 — **THREE ADDED CELLS, TWO OPENED SWAPS, AND A PRE-REGISTERED COUNT THAT WAS
//! SHORT BECAUSE ITS PROBE STOPPED AT THE LIB.**
//!
//! § 5.19 (x)'s rule for phase 7 is *step 1 of every slice is the cell addition*. Slice AC added
//! none, so that rule bought nothing there; here it buys everything, because all three of rung
//! 72's cells are ones a wrong default would hide.
//!
//! # THE THREE COUNTS, RECONCILED IN ONE PLACE
//!
//! * **3 CELLS ADDED** — `reference`, `rk4_floor_shared`, `shared_rig`. The ADD column of § 5.19
//!   (x) said 3, and **this is the first back-half slice where the hand-written number measures
//!   right**. It was still checked three independent ways first (§ 5.28 (ii)), because AC's said 1
//!   and measured 0 and slice X's `_closer` was listed while being defined exactly once.
//! * **2 SWAPS ⇒ 2 DISTINCT FUNCTION POINTERS** — `at_lever` and `integrate_fuel`, both rung 72's.
//! * **5 TABLE CONSTS** — `R72`, `R72_TWO`, `R72_FUEL`, `R72_STATOR`, `R72_TRIPLE`, counted in code
//!   below rather than asserted in prose, because an unreconciled pair of counts in two files is
//!   this phase's most-repeated defect.
//!
//! # P1 WAS FALSIFIED BY COUNT AND CONFIRMED IN MECHANISM, AND THE REASON IS WORTH MORE THAN THE NUMBER
//!
//! § 5.28 (ix) predicted that widening `TripleHooks` would produce **exactly 5 `E0063` sites**, one
//! per shipped `TripleHooks` const, and **0** from the four whole-const alias tables — a prediction
//! written to test slice AC step 7's finding rather than to restate it.
//!
//! The mechanism held exactly: every exhaustive `TripleHooks` literal went loud, and not one of
//! `R70_TWO` / `R70_STATOR` / `R71_TWO` / `R71_STATOR` said a word. **The count was 7, not 5.**
//! Two more exhaustive literals live in `tests/slice_ab_cells.rs` and `tests/slice_ac_cells.rs` —
//! the width tripwires themselves — and the probe never saw them because **`cargo check` stops
//! when the lib fails and never compiles the test targets.** The probe measured a build that had
//! already given up.
//!
//! That is [[rust-port-slice-ab-step3]]'s shape one level down (*an injection sweep that ran 2 of
//! the slice's 3 binaries and printed MISS*): **a zero from an instrument that never ran is
//! indistinguishable from a zero it measured.** The repair is not a bigger number — it is that a
//! count of compile errors must come from a build that reached every target, and there is no such
//! build when the lib is broken. The honest instrument is *"apply, fix the lib, then count what is
//! still red"*, which is what produced the 7.
//!
//! # WHAT THIS FILE DELIBERATELY DOES NOT GATE
//!
//! **`at_lever`'s and `integrate_fuel`'s DISPATCH behaviour.** Slice AC's step 1 recorded the same
//! absence for the same reason: `at_lever` returns a table pointer, and what makes the swap
//! observable is the PARENT's refusal further down. Booked to step 6.
//!
//! **NO GATE HERE HAS AN EXPIRY DATE.** Slice AB wrote an *"every slot panics with its own name"*
//! gate and had to dismantle it at step 2 when the bodies landed. The refusal gates below survive
//! step 2 intact, because the tables they read (`NO_TRIPLE` … `R71_TRIPLE`) will still be refusing
//! after rung 72's bodies land — a rung-40..71 object has no shared actuator at any step.
//!
//! **NOTHING HERE READS A GOLDEN.** Every assertion is a panic, a same-run difference, or a
//! compile-time property.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;

use turbojet::bleed_transient::LeverArm;
use turbojet::cross_split::R70_TRIPLE;
use turbojet::engine::FlightCondition;
use turbojet::full_split::R71_TRIPLE;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::R69_TRIPLE;
use turbojet::shared_actuator::{
    applied_clip, build_shared_actuator_cascade, SharedRigArm, ShareScope, IC_ORDER4_DECLARED,
    REF_LAW_DEFAULT, SHARE_LAW_DEFAULT, R72, R72_FUEL, R72_STATOR, R72_TRIPLE, R72_TWO,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::{StatorLimiter, TripleHooks, NO_TRIPLE, R68_TRIPLE};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// Slice AC's grid verbatim, which is slice AB's. This slice adds no constant of its own and this
// file must not be the place one appears.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const B: f64 = 0.10;
const TAU: f64 = 0.05;
const V_MAX: f64 = 0.20;
const SM: f64 = PHI / FLOOR - 1.0;
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

fn valve() -> BleedLimiter { BleedLimiter::with_tau(PHI, B, Some(TAU)) }

fn phi_stator() -> StatorLimiter { StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU)) }

/// Rung 72's arming: rung 70's machine — the governor is armed by a MARCH argument and `_gov_max`,
/// never by an `at_lever` keyword.
fn shared_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_lim: Some(phi_stator()), ..Default::default() }
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

fn shared_machine(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_shared_actuator_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// The panic message a closure produces, or `""` if it did not panic. Slice AB/AC's helper
/// verbatim, and for its reason: `assert!(panics(…))` is satisfied by an unrelated bug as readily
/// as by the refusal it names.
fn message_of<F: FnOnce()>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    // `AssertUnwindSafe` because every core in this crate is built out of `Cell`s and a
    // `&Core` is therefore never `UnwindSafe`. The assertion is sound here for the reason it
    // always is in these gates: each closure below either panics (and the core is dropped
    // unread) or returns, and no gate reads a core after a caught panic.
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
// 1 — THE THREE CELLS EXIST AND ARE RUNG 72's, AND EVERY TABLE BELOW REFUSES
// =============================================================================================

/// **THE REFUSAL IS THE GATE, NOT A PLACEHOLDER — and one of the three shows exactly why.**
///
/// `reference` returning `req` IS rung 72's own body. A rung-40..71 slot that answered it would
/// agree with rung 72 on every input any suite reaches, so **no value key anywhere in the crate
/// could ever see that the slot was wrong.** That is [`NO_TRIPLE`]'s stated reason and this is its
/// sharpest instance: the tempting default is not merely a guess, it is the correct answer one
/// rung up.
///
/// This gate has no expiry (slice AB's step-1 mistake): these five tables refuse for a reason that
/// is true at every future step, because a rung-40..71 object has no shared actuator ever.
#[test]
fn every_table_below_rung_72_refuses_all_three_cells_by_name() {
    let m = shared_machine(&shared_arm());
    let arm = SharedRigArm { sm: SM, tt4_max: TT4_MAX, ..Default::default() };
    let tables: [(&str, &TripleHooks); 5] = [
        ("NO_TRIPLE", &NO_TRIPLE),
        ("R68_TRIPLE", &R68_TRIPLE),
        ("R69_TRIPLE", &R69_TRIPLE),
        ("R70_TRIPLE", &R70_TRIPLE),
        ("R71_TRIPLE", &R71_TRIPLE),
    ];
    for (name, t) in tables {
        let a = message_of(|| { (t.reference)(&m.fuel.inner, 1.0, 2.0, 3.0, 4.0); });
        let b = message_of(|| { (t.rk4_floor_shared)(0.005, 20.0); });
        let c = message_of(|| { (t.shared_rig)(&m, &arm); });
        for (cell, msg) in [("_reference", &a), ("_rk4_floor_shared", &b), ("_shared_rig", &c)] {
            assert!(msg.contains("RUNG 72's"),
                    "{name}.{cell} must refuse and say whose name it is: {msg:?}");
            assert!(msg.contains(cell),
                    "{name}'s refusal must name WHICH cell was reached: {msg:?}");
        }
    }
}

/// **AND RUNG 72's OWN THREE DO NOT REFUSE** — the control for the gate above, on the same tables'
/// successor. Without it, a `reference` that panicked everywhere would pass the refusal gate.
#[test]
fn rung_72s_own_three_cells_answer() {
    let m = shared_machine(&shared_arm());
    let arm = SharedRigArm { sm: SM, tt4_max: TT4_MAX, ..Default::default() };
    assert_eq!(message_of(|| { (R72_TRIPLE.reference)(&m.fuel.inner, 1.0, 2.0, 3.0, 4.0); }),
               "");
    assert_eq!(message_of(|| { (R72_TRIPLE.rk4_floor_shared)(0.005, 20.0); }), "");
    assert_eq!(message_of(|| { (R72_TRIPLE.shared_rig)(&m, &arm); }), "");
}

/// The three cells are DIFFERENT function pointers from every parent's, and the equality control
/// rides beside each inequality — a gate that would pass for a broken instrument fails visibly
/// instead ([[rust-port-slice-aa-step1]]: never `ptr::eq` on the `const` itself).
#[test]
fn the_three_cells_are_new_pointers_and_the_other_ten_are_not() {
    // INEQUALITY: the three this slice adds.
    assert!(!fn_addr_eq(R72_TRIPLE.reference, R71_TRIPLE.reference),
            "`reference` is rung 72's own body, not the refusal rung 71 carries");
    assert!(!fn_addr_eq(R72_TRIPLE.rk4_floor_shared, R71_TRIPLE.rk4_floor_shared),
            "`rk4_floor_shared` is rung 72's own");
    assert!(!fn_addr_eq(R72_TRIPLE.shared_rig, R71_TRIPLE.shared_rig),
            "`shared_rig` is rung 72's own");

    // EQUALITY CONTROL: all ten inherited cells ARE rung 71's, so the instrument can tell the two
    // answers apart on the same table.
    assert!(fn_addr_eq(R72_TRIPLE.stator_leg, R71_TRIPLE.stator_leg));
    assert!(fn_addr_eq(R72_TRIPLE.lagged_stator, R71_TRIPLE.lagged_stator));
    assert!(fn_addr_eq(R72_TRIPLE.clamp_v, R71_TRIPLE.clamp_v));
    assert!(fn_addr_eq(R72_TRIPLE.check_v0, R71_TRIPLE.check_v0));
    assert!(fn_addr_eq(R72_TRIPLE.rk4_floor, R71_TRIPLE.rk4_floor));
    assert!(fn_addr_eq(R72_TRIPLE.solve_v, R71_TRIPLE.solve_v));
    assert!(fn_addr_eq(R72_TRIPLE.manifold_v, R71_TRIPLE.manifold_v));
    assert!(fn_addr_eq(R72_TRIPLE.triple_laws, R71_TRIPLE.triple_laws));
    assert!(fn_addr_eq(R72_TRIPLE.triple_rig, R71_TRIPLE.triple_rig));
    assert!(fn_addr_eq(R72_TRIPLE.with_ref, R71_TRIPLE.with_ref));

    // AND `rk4_floor_shared` IS NOT `rk4_floor` — asserted by the TYPE SYSTEM rather than by a
    // pointer compare, which is the stronger form: their `fn` types differ (`fn(f64, f64)` against
    // `fn(f64, f64, usize, f64)`), so no expression can even ask whether they are equal. The first
    // draft of this gate cast both to `*const ()` to force the comparison, which would have
    // compared two addresses whose inequality the compiler already guarantees.
}

/// The TWO swaps, and the three tables that carry none. Counted in code so a deleted table is a
/// compile error rather than a prose correction.
#[test]
fn the_slice_writes_five_table_consts_and_swaps_exactly_two_pointers() {
    assert!(!fn_addr_eq(R72.at_lever, turbojet::full_split::R71.at_lever),
            "swap 1 of 2 — `at_lever`");
    assert!(!fn_addr_eq(R72_FUEL.integrate_fuel, turbojet::full_split::R71_FUEL.integrate_fuel),
            "swap 2 of 2 — `integrate_fuel`");

    // The three tables with NO swap, and their equality controls.
    let turbojet::two_spool_transient::TwoSpoolTransientHooks {
        try_close: _, try_instant_tail: _, powers: _,
    } = R72_TWO;
    let turbojet::stator_transient::StatorTransientHooks {
        stator_march: _, v_of: _, arm: _, at_stator: _,
    } = R72_STATOR;
    assert!(fn_addr_eq(R72_TWO.try_close, turbojet::full_split::R71_TWO.try_close));
    assert!(fn_addr_eq(R72_STATOR.stator_march, turbojet::full_split::R71_STATOR.stator_march));

    // And every OTHER lever/fuel cell is rung 71's — so "exactly two" is a measurement.
    assert!(fn_addr_eq(R72.b_at_point, turbojet::full_split::R71.b_at_point));
    assert!(fn_addr_eq(R72_FUEL.try_close_fuel, turbojet::full_split::R71_FUEL.try_close_fuel));
    assert!(fn_addr_eq(R72_FUEL.try_surge_fuel, turbojet::full_split::R71_FUEL.try_surge_fuel));
}

// =============================================================================================
// 2 — THE CELL BODIES, WHERE A GATE CAN EXIST AT THIS STEP
// =============================================================================================

/// **`reference` IS THE BITWISE IDENTITY, AND THAT IS ASSERTED AS AN EXACT-BITS CLAIM.**
///
/// § 5.28 (vi) measured 195 278 of 195 278 calls returning `req` unchanged. `assert_eq!` on
/// `f64` would admit `-0.0` for `0.0` and both NaN spellings for neither, so the comparison is on
/// `to_bits` — the same reason rung 73's own float-identical branch exists.
#[test]
fn reference_returns_req_bit_for_bit_at_this_rung() {
    let m = shared_machine(&shared_arm());
    for req in [0.0_f64, -0.0, 1.0, -3.25e-7, 1.7976931348623157e308, f64::MIN_POSITIVE] {
        for (g_own, gf, gr) in [(0.0, 0.0, 0.0), (1.0, 2.0, 3.0), (-5.0, 1e9, -1e9)] {
            let got = (R72_TRIPLE.reference)(&m.fuel.inner, req, g_own, gf, gr);
            assert_eq!(got.to_bits(), req.to_bits(),
                       "rung 72's `_reference` is `return req`: {req} with ({g_own}, {gf}, {gr})");
        }
    }
}

/// **THE FLOOR'S MESSAGE IS THE ENTIRE CELL, SO THE GATE READS THE MESSAGE — AND ON A TOKEN THE
/// PYTHON SUITE'S NEEDLE DOES NOT DISCRIMINATE.**
///
/// `tests/test_rung72.py:445` fires this floor under `match=r"FOUR actuator states"`, and § 5.28
/// (v) measured that phrase present in rungs 73's and 74's messages too — so the shipped Python
/// gate passes with either successor's floor installed. Rung 69's analogue needle (`"rank TWO"`)
/// is unique to it and does not have the defect.
///
/// This gate therefore asserts the discriminating tokens **and** asserts the shared phrase is
/// present, so that when rungs 73 and 74 land, the cross-rung gate that proves the shared phrase
/// insufficient has both halves already written down here.
#[test]
fn the_floor_fires_on_a_message_that_names_this_rung_and_its_own_argument() {
    // ADMITTED: the grid every reader in this rung runs on.
    assert_eq!(message_of(|| (R72_TRIPLE.rk4_floor_shared)(0.005, 80.0)), "",
               "ds*rate = 0.4 is well inside the region and must not fire");

    let msg = message_of(|| (R72_TRIPLE.rk4_floor_shared)(0.05, 80.0));
    assert!(!msg.is_empty(), "ds*rate = 4.0 is outside the region and must fire");
    assert!(msg.contains("rung-72"), "the message must name the RUNG it came from: {msg:?}");
    assert!(msg.contains("-1/tau_f"),
            "and the ARGUMENT that is new at this rung — the masked leg's bare pole: {msg:?}");
    assert!(msg.contains("FOUR actuator states"),
            "the phrase the Python suite matches on is present — and it is not enough, which is \
             the point: rungs 73 and 74 carry it too. {msg:?}");
    assert!(msg.contains("4.000"),
            "and the VALUE, formatted to Python's `:.3f`: {msg:?}");
}

/// The boundary is `<=`, not `<`. A one-sided bar here would admit a port that wrote `<` and
/// refused the exactly-admissible step — the constant's whole content is where it stops.
#[test]
fn the_floor_admits_its_own_boundary_exactly() {
    assert_eq!(message_of(|| (R72_TRIPLE.rk4_floor_shared)(0.05, 40.0)), "",
               "ds*rate = 2.0 EXACTLY is admitted — the condition is `<= 2.0`");
    assert!(!message_of(|| (R72_TRIPLE.rk4_floor_shared)(0.05, 40.000000001)).is_empty(),
            "and anything above it is not");
}

/// **`shared_rig`'s FOUR ARMING FLAGS EACH DO SOMETHING, AND `inc` CHOOSES BETWEEN TWO STATORS
/// RATHER THAN ADDING ONE.**
///
/// Python's `sl = si = None` then one branch: the arm selects WHICH coordinate the single stator
/// watches. A port that armed both would give rung 71's constraint count to rung 70's arm and
/// every rank reading in the slice would be one too high, with nothing raising.
#[test]
fn shared_rig_arms_each_of_the_four_loops_independently() {
    let m = shared_machine(&shared_arm());
    let base = SharedRigArm { sm: SM, tt4_max: TT4_MAX, ..Default::default() };

    let (all, surge, lag) = (R72_TRIPLE.shared_rig)(&m, &base);
    assert!(surge.is_some() && lag.is_some(), "`fuel` arms BOTH the floor and the lag");
    assert!(all.fuel.inner.lever.lim.is_some(), "`valve` arms the bleed limiter");
    assert_eq!(all.fuel.inner.gov_max.get(), Some(TT4_MAX), "`gov` sets the set point");

    let (no_fuel, s2, l2) = (R72_TRIPLE.shared_rig)(&m, &SharedRigArm { fuel: false, ..base });
    assert!(s2.is_none() && l2.is_none(), "`fuel: false` disarms BOTH — never one of the two");
    assert!(no_fuel.fuel.inner.lever.lim.is_some(), "and touches nothing else");

    let (no_valve, _, _) = (R72_TRIPLE.shared_rig)(&m, &SharedRigArm { valve: false, ..base });
    assert!(no_valve.fuel.inner.lever.lim.is_none(), "`valve: false` drops the limiter");

    let (no_gov, _, _) = (R72_TRIPLE.shared_rig)(&m, &SharedRigArm { gov: false, ..base });
    assert_eq!(no_gov.fuel.inner.gov_max.get(), None,
               "`gov: false` CLEARS the set point — a real assignment, not a skip");
}

/// `inc` moves the stator's reference and does not add a second stator. Both directions asserted,
/// because the failure that matters is *both armed*, which no float would reveal.
#[test]
fn inc_selects_the_stators_coordinate_and_never_arms_two() {
    let m = shared_machine(&shared_arm());
    let base = SharedRigArm { sm: SM, tt4_max: TT4_MAX, ..Default::default() };

    let (phi_arm, _, _) = (R72_TRIPLE.shared_rig)(&m, &SharedRigArm { inc: false, ..base });
    assert!(phi_arm.fuel.inner.stator.lim.is_some(), "`inc: false` is rung 70's `phi` stator");
    assert!(phi_arm.fuel.inner.stator.inc.is_none(),
            "and it does NOT also arm the incidence one");

    let (inc_arm, _, _) = (R72_TRIPLE.shared_rig)(&m, &SharedRigArm { inc: true, ..base });
    assert!(inc_arm.fuel.inner.stator.inc.is_some(),
            "`inc: true` is rung 71's INCIDENCE stator");
    assert!(inc_arm.fuel.inner.stator.lim.is_none(), "and it does NOT also arm the `phi` one");

    let (none, _, _) = (R72_TRIPLE.shared_rig)(&m, &SharedRigArm { stator: false, ..base });
    assert!(none.fuel.inner.stator.lim.is_none() && none.fuel.inner.stator.inc.is_none(),
            "`stator: false` arms neither, on either coordinate");
}

// =============================================================================================
// 3 — THE DECLARED CARRIERS
// =============================================================================================

/// The three class attributes Python declares at rung 72, and **the one that is a `const` rather
/// than a `Cell` is the one Python never assigns** — measured, 0 `self._ic_order4 =` sites.
#[test]
fn the_three_declared_attributes_carry_pythons_defaults() {
    let m = shared_machine(&shared_arm());
    assert_eq!(SHARE_LAW_DEFAULT, "max");
    assert_eq!(REF_LAW_DEFAULT, "sched");
    assert_eq!(IC_ORDER4_DECLARED, "rqvf");
    assert_eq!(m.fuel.inner.share_law.get(), "max",
               "a freshly built rung-72 machine composes by MIN-SELECT");
    assert_eq!(m.fuel.inner.ref_law.get(), "sched",
               "and reads the SCHEDULED fuel — rung 73's knob, at its rung-72 default");

    // `_ic_order4` is a PERMUTATION of the four loops, which is what Python's own assert says.
    let mut c: Vec<char> = IC_ORDER4_DECLARED.chars().collect();
    c.sort_unstable();
    assert_eq!(c, vec!['f', 'q', 'r', 'v'],
               "rung-72 ic_order4 is a permutation of 'frqv'");
}

/// **`ShareScope` RESTORES THE PREVIOUS VALUE, NOT THE DEFAULT — and a NEST is the only instrument
/// that can tell those apart.**
///
/// The two spellings agree on every shipped path, exactly as `RefScope`'s note says, so this gate
/// manufactures the nest rather than waiting for a reader to produce one.
#[test]
fn share_scope_restores_the_previous_value_and_a_nest_proves_it() {
    let m = shared_machine(&shared_arm());
    assert_eq!(m.fuel.inner.share_law.get(), "max");
    {
        let outer = ShareScope::set(&m, "sum");
        assert_eq!(outer.displaced(), "max", "the guard exposes what it displaced");
        assert_eq!(m.fuel.inner.share_law.get(), "sum");
        assert_eq!(applied_clip(&m, 3.0, 4.0), 7.0, "`sum` DOUBLE-CLIPS — the instrument");
        {
            let inner = ShareScope::set(&m, "max");
            assert_eq!(inner.displaced(), "sum",
                       "the inner guard displaces the OUTER's value, not the default");
            assert_eq!(applied_clip(&m, 3.0, 4.0), 4.0, "`max` is MIN-SELECT — the plant");
        }
        assert_eq!(m.fuel.inner.share_law.get(), "sum",
                   "**restore-PREVIOUS**: dropping the inner guard returns `sum`, and a \
                    restore-to-DEFAULT would have written `max` here and passed every other \
                    assertion in this file");
    }
    assert_eq!(m.fuel.inner.share_law.get(), "max", "and the outer guard restores the default");
}

/// `applied_clip` is the composition law in ONE place, and the two laws DISAGREE on the sampled
/// point — so a reader that recomputed either one could be caught disagreeing with the march.
#[test]
fn the_two_composition_laws_disagree_where_it_matters() {
    let m = shared_machine(&shared_arm());
    for (gf, gr) in [(3.0, 4.0), (0.0, 2.5), (1e-9, 0.0)] {
        let max_law = { let _g = ShareScope::set(&m, "max"); applied_clip(&m, gf, gr) };
        let sum_law = { let _g = ShareScope::set(&m, "sum"); applied_clip(&m, gf, gr) };
        assert_eq!(max_law, gf.max(gr));
        assert_eq!(sum_law, gf + gr);
        assert!(max_law != sum_law || gf == 0.0 || gr == 0.0,
                "with both legs cutting the two laws MUST differ: {gf}, {gr}");
    }
    // and the one place they agree is exactly where one leg is dormant, which is the rung's own
    // statement about what a masked leg costs.
    let _g = ShareScope::set(&m, "sum");
    assert_eq!(applied_clip(&m, 2.5, 0.0), 2.5, "a dormant leg makes `sum` and `max` agree");
}
