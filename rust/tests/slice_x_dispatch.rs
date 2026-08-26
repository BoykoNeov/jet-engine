//! SLICE X step 5 — **THE GATES NO VALUE KEY CAN CARRY**, and the manufactured bugs.
//!
//! Everything rungs 62–64 do that a float cannot witness:
//!
//! * **THE REDUCE IS BY DISPATCH.** `bleed_lim = None` returns the PARENT'S FUNCTION OBJECT, not
//!   "rung 64's body with `b = 0`". The two produce identical numbers at every state — that is
//!   what the reduce MEANS — so § 5.21 (v)'s point stands: no value key can tell them apart, and
//!   only a counter can.
//! * **THE REGIME IS RETURNED AND NOBODY READS IT.** Python's `_solve_b` returns
//!   `(closure, b, regime)` and its own docstring says the regime is *"reported, never inferred by
//!   a reader comparing floats"* — and then no ladder caller reads element `[2]`. P3 asked whether
//!   a gate that runs one machine tests two branches of three; it does, and the third needs
//!   `authority_ceiling`'s deliberately over-set floor.
//! * **TWO BRANCHES ARE DEAD, MEASURED.** `b_of`'s `b_state` override is rung 65's, declared at
//!   64 and taken **0 times**; `_solve_b`'s abort arms are taken **0 of 156 373** closure calls
//!   (probe 8). A port that drops either passes every value gate in the slice and breaks at slice
//!   Y. They are gated by MANUFACTURED bugs, because no reachable input reaches them.
//! * **THE RUNG-62 PIN.** Python's 16 `super(LimitedBleedTransient, self)` sites at rungs 65–75
//!   are a STATIC pin to ONE ancestor — the parent's BODY on the LEAF's table. In Rust that is
//!   `r62_try_close_fuel(leaf_core, …)`, and calling it with a rung-62 core instead compiles, runs
//!   and silently freezes the ladder. P7's gate is manufactured: the same rung-62 body, same
//!   hardware, once reached through rung 64's table and once through rung 62's, asserting the
//!   values DIFFER.
//!
//! **[`Census64`] IS THREAD-LOCAL AND HAS NO PER-TEST RESET**, so every test here resets first.
//! Cargo gives each `#[test]` its own thread today; a reset makes that irrelevant rather than
//! relied upon.

use std::panic::catch_unwind;

use turbojet::bleed_transient::{
    build_scheduled_bleed, r62_try_close_fuel, BleedSchedule, LeverArm,
};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter, Census64};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::two_spool_transient::ForcedBleed;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const DS: f64 = 0.02;
const N_LO: f64 = 0.65;
const B: f64 = 0.10;
const PHI: f64 = 0.80;

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

/// A rung-64 machine — the LEAF table.
fn lt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// A rung-62 machine — `R62`, the table the PIN must NOT be called with.
fn bt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_scheduled_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp() -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS }
}

fn valve() -> BleedLimiter { BleedLimiter::new(PHI, B) }

/// One floored march, with the census taken around it and nothing else running.
fn march_census(arm: &LeverArm) -> Census64 {
    Census64::reset();
    let m = lt(arm);
    let _ = m.bill_cell(&flight(), &ramp(), false);
    Census64::take()
}

// =============================================================================================
// THE REDUCE — by DISPATCH, and no float can see it
// =============================================================================================

/// `bleed_lim = None` must reach rung 62's body **by returning its function object**, not by
/// running rung 64's with a zero position. The numbers are identical either way — that is what the
/// reduce means — so this is the only instrument that can tell the two ports apart.
#[test]
fn the_reduce_is_a_dispatch_and_not_a_zero_position() {
    let bare = march_census(&LeverArm::default());
    assert!(bare.solve_b_calls == 0,
            "an unfloored machine must never enter the outer solve; it made {} calls",
            bare.solve_b_calls);
    assert!(bare.close_unfloored > 0 && bare.close_fuel_unfloored > 0,
            "both closure cells must have handed back to rung 62: {} / {}",
            bare.close_unfloored, bare.close_fuel_unfloored);
    assert!(bare.b_at_point_resolved == 0 && bare.b_at_point_unfloored > 0,
            "an unfloored `b_at_point` reads `b_of`, it does not re-solve: {} / {}",
            bare.b_at_point_resolved, bare.b_at_point_unfloored);

    let floored = march_census(&LeverArm::floored(valve()));
    assert!(floored.solve_b_calls > 0, "a floored machine must enter the outer solve");
    assert!(floored.close_fuel_unfloored == 0,
            "a floored machine must never take the reduce leg: {}",
            floored.close_fuel_unfloored);
    assert!(floored.b_at_point_resolved > 0 && floored.b_at_point_unfloored == 0,
            "a floored `b_at_point` RE-SOLVES: {} / {}",
            floored.b_at_point_resolved, floored.b_at_point_unfloored);
}

/// **A DORMANT floor is still a FLOORED machine.** Rung 64's own reduce gate asserts a floor below
/// every `phi` marches bit-for-bit like the valve-shut machine — and that is exactly the reading
/// that cannot distinguish "dispatched away" from "solved and got zero". The counters can: the
/// outer solve RUNS at every state and returns `Dormant` every time.
#[test]
fn a_dormant_floor_still_solves_at_every_state() {
    let low = march_census(&LeverArm::floored(BleedLimiter::new(0.30, B)));
    assert!(low.solve_b_calls > 0, "a dormant floor still enters the solve");
    assert_eq!(low.dormant, low.solve_b_calls,
               "every solve on a floor below the whole march must clamp DORMANT");
    assert!(low.riding == 0 && low.saturated == 0,
            "riding={} saturated={}", low.riding, low.saturated);
}

// =============================================================================================
// P3 — THE THREE REGIMES, and the headline machine reaches only two
// =============================================================================================

/// The regime is `_solve_b`'s third return element and **no ladder caller reads it**, so this is
/// the only gate it has. The rung's headline machine reaches DORMANT and RIDING; SATURATED needs
/// `authority_ceiling`'s deliberately over-set floor, which is the rung's own witness that a law
/// can fail to deliver its set point on HARDWARE rather than on control.
#[test]
fn all_three_regimes_are_reached_and_the_headline_machine_reaches_only_two() {
    let head = march_census(&LeverArm::floored(valve()));
    assert!(head.dormant > 0 && head.riding > 0, "{head:?}");
    assert_eq!(head.saturated, 0,
               "the headline set point sits strictly inside [shut, fully-open], so it must never \
                saturate — {head:?}");
    assert_eq!(head.dormant + head.riding + head.saturated, head.solve_b_calls,
               "every solve lands on exactly one clamp");

    // the over-set floor: ABOVE the fully-open march's own minimum, so it saturates AND is
    // violated. `authority_ceiling` builds it from the `full` march's own reading.
    Census64::reset();
    let ac = lt(&LeverArm::default()).authority_ceiling(&flight(), &ramp(), B, N_LO, 0.10);
    let over = Census64::take();
    assert!(over.saturated > 0,
            "the over-set floor must SATURATE — that is the rung's witness that the ceiling \
             belongs to b_max: {over:?}");
    assert!(ac.violated, "and be violated");
}

// =============================================================================================
// P4 — THE TWO DEAD BRANCHES, gated by MANUFACTURE because no input reaches them
// =============================================================================================

/// **`b_of`'s `b_state` override is RUNG 65's, and dead at 64.** Declared rather than omitted
/// because a port that drops it passes every value gate in the slice and breaks at slice Y.
///
/// **THE OBVIOUS GATE IS VACUOUS AND WAS WRITTEN THAT WAY FIRST.** Asserting `b_of_state == 0` on
/// every march is satisfied by a port that DELETED the branch — the count stays zero either way,
/// so the assertion cannot see the defect it names. [[rust-port-slice-u-step4]]'s *a gate
/// comparing a key with ITSELF cannot see its value*, one shape over.
///
/// So the branch is MANUFACTURED reachable: the carrier is set directly and `b_of` must return
/// it, which fails to compile if the field goes and fails to pass if the branch does. The
/// zero-count assertion is kept BESIDE it, where it now means "and no shipped path reaches it"
/// rather than standing alone.
#[test]
fn the_lagged_position_override_is_declared_live_and_unreached() {
    let m = lt(&LeverArm::floored(valve()));
    let core = &m.fuel.inner;

    // (1) THE BRANCH EXISTS AND ANSWERS — rung 65's lagged position, set by hand.
    core.b_state.set(Some(0.037));
    assert_eq!(core.b_of(0.85, None), 0.037,
               "`b_state` must override rung 62's constant — this is rung 65's marched valve \
                position, declared at 64 so slice Y inherits it rather than re-deriving it");

    // (2) AND `b_forced` WINS OVER IT — Python's own precedence, and its reason: the command
    // solve trials positions on a plant whose live state is the one being commanded AWAY from.
    {
        let _g = ForcedBleed::set(core, 0.081);
        assert_eq!(core.b_of(0.85, None), 0.081,
                   "the forced trial must win over the marched state");
    }
    // and the guard's drop restores the STATE, not `None`-for-everything
    assert_eq!(core.b_of(0.85, None), 0.037,
               "dropping the trial guard must expose `b_state` again, not erase it");
    core.b_state.set(None);
    assert_eq!(core.b_of(0.85, None), 0.0, "and clearing it falls through to rung 62's constant");

    // (3) NO SHIPPED PATH REACHES IT — the count, which alone would be vacuous.
    for arm in [LeverArm::default(), LeverArm::floored(valve()), LeverArm::constant(B),
                LeverArm::scheduled(BleedSchedule::new(B, N_LO))] {
        let c = march_census(&arm);
        assert_eq!(c.b_of_state, 0,
                   "rung 64 has no marched valve state; `b_state` was read {} times on {:?}",
                   c.b_of_state, arm.keys());
    }
}

/// **`b_of`'s FALL-THROUGH to rung 62 is never taken inside a floored solve** — every `b_of` a
/// closure reaches is reached from within a live trial, so the forced carrier answers first.
///
/// **THE SCOPE IS THE GATE.** A reader may call `b_of` DIRECTLY and then the fall-through is the
/// only branch, so `b_of_super == 0` is FALSE in general and the module note says so. What holds
/// is an IDENTITY: every fall-through is one of the three unfloored paths, and they are counted
/// separately. Measured 48 + 344 + 86 = 478 on the census grid.
#[test]
fn every_fall_through_to_rung_62_is_on_an_unfloored_path() {
    for arm in [LeverArm::default(), LeverArm::floored(valve()), LeverArm::constant(B),
                LeverArm::scheduled(BleedSchedule::new(B, N_LO))] {
        let c = march_census(&arm);
        assert_eq!(c.b_of_super,
                   c.close_unfloored + c.close_fuel_unfloored + c.b_at_point_unfloored,
                   "the fall-through count must decompose exactly across the three unfloored \
                    paths on {:?}: {c:?}", arm.keys());
    }
    // and on a FLOORED machine the identity's right-hand side is zero except for `b_at_point`'s
    // own leg, which a floored machine never takes either — so the count is exactly 0.
    let f = march_census(&LeverArm::floored(valve()));
    assert_eq!(f.b_of_super, 0, "a floored march reaches rung 62's `b_of` never: {f:?}");
    assert!(f.b_of_forced > 0, "and reads the forced carrier instead: {f:?}");
}

/// **`_solve_b`'s ABORT arms are dead on every shipped grid** — probe 8 measured 0 in 156 373
/// closure calls over `tests/test_rung64.py`. Python propagates its raise out of `_illinois`; the
/// port propagates an `Abort` through `try_illinois` identically, so the arms are FAITHFUL rather
/// than defensive.
///
/// **THIS IS A WATCHDOG, NOT A GATE, AND THE DIFFERENCE IS STATED RATHER THAN BLURRED.** A
/// zero-count assertion is satisfied by a port that deleted the arms, exactly as the `b_state`
/// one above was until it was rewritten — and unlike `b_state`, an `Abort` cannot be manufactured
/// from a test without a hook in production code, which would be worse than the gap. What this
/// DOES catch is the arms starting to fire: a grid change, a map change, or a set point that
/// walks the bracket outside the physical root. What it CANNOT catch is their removal — that is
/// carried by `r64_solve_b`'s signature (`Result<_, Abort>`), which the compiler enforces at
/// every call site, and by the oracle's 1 906 keys, which a swallowed abort would move.
#[test]
fn the_abort_arms_are_carried_and_never_taken() {
    for arm in [LeverArm::default(), LeverArm::floored(valve()),
                LeverArm::floored(BleedLimiter::new(0.95, B))] {
        let c = march_census(&arm);
        assert_eq!(c.solve_b_aborts, 0, "on {:?}: {c:?}", arm.keys());
    }
}

// =============================================================================================
// THE `R62` SLOT THAT PANICS — step 1's decision, made live
// =============================================================================================

/// **`b_at_point` DOES NOT EXIST BELOW RUNG 64**, so `R62`'s slot holds a PANIC rather than
/// `b_of`. Defaulting it to `b_of` would be right on a rung-62 machine and wrong on a floored one
/// — a claim **no value gate could see**, because the two agree on exactly the machines a rung-62
/// suite builds. This is the other side of § 5.22 (ii)'s measurement, and it has to be
/// manufactured: nothing in the ladder calls it.
#[test]
fn a_rung62_machine_refuses_to_report_a_committed_valve_position() {
    let fl = flight();
    let m62 = bt(&LeverArm::constant(B));
    let traj = m62.stator_march(&fl, &ramp(), None, &StatorLeg::default()).0;
    let p = traj[traj.len() / 2];
    let hit = catch_unwind(move || {
        let m = bt(&LeverArm::constant(B));
        m.b_at_point(&flight(), &p)
    });
    assert!(hit.is_err(),
            "rung 62 has no `b_at_point`; the slot must PANIC, not silently answer `b_of`");
}

// =============================================================================================
// P7 — THE RUNG-62 PIN: the parent's BODY on the LEAF's table
// =============================================================================================

/// **THE PIN, MANUFACTURED.** Python's 16 `super(LimitedBleedTransient, self)` sites at rungs
/// 65–75 name ONE ancestor statically and run its body with `self` — the LEAF — still supplying
/// every dispatched name. The port spells that `r62_try_close_fuel(leaf_core, …)`, and passing a
/// rung-62 core instead **compiles, runs, and silently freezes the ladder**.
///
/// The discriminator is the forced carrier. Inside rung 62's body the very first thing is
/// `b_of`, which is DISPATCHED: on the leaf's table it reads the trial position, on rung 62's it
/// reads the stored constant. Same body, same hardware, same arguments — different answer, and
/// only because the table came from the leaf.
///
/// **Falsified if they agree**, which would mean the leaf table is not being reached.
#[test]
fn the_rung62_pin_runs_the_parents_body_on_the_leafs_table() {
    let fl = flight();
    let leaf = lt(&LeverArm::floored(valve()));
    let parent = bt(&LeverArm::default());
    let (tt2, pt2, _) = leaf.fuel.inner.inlet(&fl);
    let eq = leaf.fuel.inner.equilibrium(&fl, 1200.0);
    let mf = leaf.fuel.fuel_for_tt4(&fl, 1200.0);

    // (1) the parent's body, reached with the LEAF's table, with a trial position forced.
    let trial = 0.06;
    let with_leaf = {
        let _g = ForcedBleed::set(&leaf.fuel.inner, trial);
        r62_try_close_fuel(&leaf.fuel, eq.nu_lp, eq.nu_hp, mf, tt2, pt2)
            .expect("the closure converges at this state")
    };
    // (2) the SAME body on rung 62's table. It cannot see the carrier — the name `b_forced` is
    // rung 64's — so it reads the stored constant, which is 0.0.
    let with_parent = r62_try_close_fuel(&parent.fuel, eq.nu_lp, eq.nu_hp, mf, tt2, pt2)
        .expect("the closure converges at this state");

    assert!(with_leaf.base.bleed == Some(trial),
            "the leaf's `b_of` must return the forced trial: {:?}", with_leaf.base.bleed);
    assert!(with_parent.base.bleed.is_none() || with_parent.base.bleed == Some(0.0),
            "rung 62's `b_of` on an unarmed machine is the stored constant: {:?}",
            with_parent.base.bleed);
    assert!(with_leaf.base.phi_lp != with_parent.base.phi_lp,
            "THE PIN IS NOT LIVE: the same rung-62 body gave the same answer through both \
             tables ({}), so the leaf's dispatched `b_of` is not being reached and every \
             `super(LimitedBleedTransient, self)` site at rungs 65-75 would freeze silently.",
            with_leaf.base.phi_lp);

    // and the carrier is CLEAR again — the guard's destructor, not a `finally` anyone can forget.
    assert!(leaf.fuel.inner.b_forced.get().is_none(),
            "the RAII guard must have cleared the carrier when it dropped");
}

/// **THE CARRIER'S PANIC IS A PORT DECISION, and it is live.** Python CLOBBERS a nested
/// `_b_forced` silently; the port refuses. Nothing in rungs 62–64 nests one — measured — so this
/// too is manufactured, and it exists so that the slice-Y porter who trips it reads the reasoning
/// at the failure site rather than in a document they would have to know about.
#[test]
fn a_nested_trial_position_is_refused_rather_than_clobbering() {
    let leaf = lt(&LeverArm::floored(valve()));
    let hit = catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _outer = ForcedBleed::set(&leaf.fuel.inner, 0.03);
        let _inner = ForcedBleed::set(&leaf.fuel.inner, 0.07);
    }));
    assert!(hit.is_err(), "a nested forced position must be refused, not silently clobbered");
    // the OUTER guard still dropped on the way out, so the carrier is clear
    assert!(leaf.fuel.inner.b_forced.get().is_none(),
            "unwinding must still clear the carrier: {:?}", leaf.fuel.inner.b_forced.get());
}
