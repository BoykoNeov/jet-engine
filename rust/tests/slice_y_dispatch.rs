//! SLICE Y step 5 — **THE GATES NO VALUE KEY CAN CARRY**, and the manufactured bugs.
//!
//! `slice_y_oracle.rs` is green at **35 994 keys** against two interpreters, and step 4's
//! injection census then asked the only question a green oracle leaves open: *which defects would
//! it catch?* Six cells were corrupted in the shipped `src`, one at a time. Three go red
//! (`b_at_point` re-solving — 394 keys; `stator_march` ignoring `scope.b0` — 1 280 keys;
//! `integrate_fuel` skipping the valve-lag march — aborts at the port's own assert). **Three stay
//! green at exactly zero keys moved**, and this file is those three:
//!
//! * **THE TWO MIRROR ZEROS** (§ 5.23 (i)). Rung 65 overrides `_close` and `_close_fuel` with the
//!   SAME two-way test, and **each override only ever takes the arm that equals its parent's**:
//!   `_close` is reached only from STEADY solves (`b_state` never set) and `_close_fuel` only from
//!   inside a derivative evaluation (`b_state` always set). The two zeros are on OPPOSITE arms, so
//!   a port that drops either test agrees on every shipped path. Reached here by setting the
//!   carrier by hand — **and the values on the two arms are asserted to DIFFER**, so the gate is a
//!   statement about the plant and not about the counter.
//! * **THE PREVIOUS-VALUE GUARD** (P4). `InitialBleed` restores the value it displaced where
//!   `ForcedBleed` restores `None`; probe 3 measured the `_b0` guard's max nesting depth at **1**
//!   over the whole four-suite grid, so on every shipped path the two spellings agree. The nest is
//!   manufactured, and the inner scope is a REAL march rather than a bare guard pair.
//! * **`py_max3`** — the defect step 4 found by asking a reader for its degenerate case. Python's
//!   `max(a, b, c)` is not `a.max(b).max(c)`, and the difference is `NaN` in the first position.
//!   No reachable march distinguishes them (`n_ride` is 340 / 251 / 214), so the FUNCTION is gated
//!   rather than the call site, and that limit is stated rather than papered over.
//!
//! It also reports **P6**, both halves, at the rung the slice-X panic named as its own expiry.
//!
//! # WHAT THIS FILE READS, ASKED OF EVERY ASSERTION
//!
//! Slice V step 5's lesson: *ask of every assertion in a manufactured-bug gate WHAT FILE IT READS
//! — the four that read nothing survive a regenerated golden.* **Nothing here reads a golden.**
//! Every gate is either a counter, a same-run difference between two dispatch arms, or a property
//! of a shipped function. Regenerating `slice_y_pypy.tsv` cannot make any of them pass or fail,
//! which is the point: they cover exactly what the 35 994 keys cannot.
//!
//! [`Census65`] is thread-local with no per-test reset, so every test resets first. Cargo gives
//! each `#[test]` its own thread today; the reset makes that irrelevant rather than relied upon.

use std::panic::catch_unwind;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::{
    build_lagged_bleed, py_max3, r65_try_close, r65_try_close_fuel, Census65,
};
use turbojet::limited_bleed::{BleedLimiter, Census64};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::two_spool_transient::{ForcedBleed, InitialBleed, MarchedBleed};
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
/// Deliberately coarse: nothing here reads a trajectory's SHAPE, only which body produced it.
const DS: f64 = 0.02;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const TAU: f64 = 0.05;

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

fn gt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

fn lagged_machine() -> ScheduledStatorCore {
    gt(&LeverArm::floored(BleedLimiter::with_tau(PHI, B, Some(TAU))))
}

fn panics<F: FnOnce() + std::panic::UnwindSafe>(f: F) -> bool {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f).is_err();
    std::panic::set_hook(prev);
    out
}

/// A state the closures can actually be evaluated at — the START of a lagged march, which is
/// where § 5.23's own probes take theirs and where the floor is known to RIDE. Returns
/// `(nu_lp, nu_hp, Tt4, mf, Tt2, pt2)`.
///
/// It also asserts the march really dispatched to the valve-lag integrator: a point with no
/// recorded valve state would mean the probe is reading a machine other than the one under test.
fn probe_state(m: &ScheduledStatorCore) -> (f64, f64, f64, f64, f64, f64) {
    let (traj, _) = m.stator_march(&flight(), &ramp(DS), None, &StatorLeg::default());
    let p = &traj[0];
    match p.extra {
        turbojet::fuel_transient::PointExtra::Valve { b, .. } => {
            assert!(b > 0.0 && b < B, "the probe state must be one where the valve RIDES; at a \
                                       stop the two dispatch arms would agree by accident");
        }
        _ => panic!("the probe march did not dispatch to the valve-lag integrator"),
    }
    let (tt2, pt2, _) = m.fuel.inner.inlet(&flight());
    (p.nu_lp, p.nu_hp, p.tt4, p.mf, tt2, pt2)
}

// =============================================================================================
// P3a — `_close`'s LIVE ARM: 0 of 2 928 on the shipped grid, reached here by hand
// =============================================================================================

/// § 5.23 (i)'s first mirror zero. `_close` is reached only from the STEADY solves — `equilibrium`
/// and the running line a march starts on — and none of those runs inside a derivative
/// evaluation, so `b_state` is never set when it is called. Its `lagged && b_state` arm therefore
/// takes **0** of 2 928 calls, and deleting Python's whole override moves every witness key by
/// zero.
///
/// **THE ARM IS REACHED BY SETTING THE CARRIER, AND THE TWO ARMS ARE ASSERTED TO DISAGREE.** A
/// counter alone would pass on a port that took the right branch and computed the wrong thing in
/// it; the value difference is what makes this a statement about the PLANT. Rung 62's closure runs
/// the valve at the imposed state, rung 64's re-solves its instantaneous root, and at a state
/// where the floor rides those are different machines.
#[test]
fn close_takes_the_marched_arm_only_when_the_state_is_live_and_the_arms_differ() {
    let m = lagged_machine();
    let (nu_lp, nu_hp, tt4, _mf, tt2, pt2) = probe_state(&m);
    let core = &m.fuel.inner;

    Census65::reset();
    let steady = r65_try_close(core, nu_lp, nu_hp, tt4, tt2, pt2).expect("the steady arm closes");
    let c = Census65::take();
    assert_eq!((c.close_steady, c.close_marched), (1, 0),
               "with no live state a lagged `_close` must run rung 64's INSTANTANEOUS root");

    // The manufactured bug: set the carrier the shipped grid never sets here.
    Census65::reset();
    let marched = {
        let _g = MarchedBleed::set(core, 0.5 * B);
        r65_try_close(core, nu_lp, nu_hp, tt4, tt2, pt2).expect("the marched arm closes")
    };
    let c = Census65::take();
    assert_eq!((c.close_steady, c.close_marched), (0, 1),
               "with a live state a lagged `_close` must dispatch PAST rung 64 to rung 63's \
                closure — a port that drops `&& b_state.is_some()` fails HERE and nowhere else");
    assert!(core.b_state.get().is_none(), "the guard must restore on the way out");

    assert_ne!(steady.phi_lp.to_bits(), marched.phi_lp.to_bits(),
               "the two arms must be two MACHINES. If they agree, this gate has become a \
                tautology on the counter and the probe state is no longer one where the floor \
                rides.");
}

// =============================================================================================
// P3b — `_close_fuel`'s STEADY ARM: the MIRROR zero, on the OPPOSITE branch
// =============================================================================================

/// § 5.23 (i)'s second, and the one that costs a gate. `_b_state` is set only inside
/// `_integrate_fuel_valve_lag`'s `der`, which imposes FUEL — so every close under a live state is
/// a `_close_fuel`, and this method's `lagged && !b_state` arm ran **0** times in 221 437 calls.
///
/// **THE RUNG'S OWN DOCSTRING IS WHAT MAKES THIS WORTH GATING.** It justifies that half of the
/// test at length — *"Outside it — every STEADY solve — the lag is meaningless and rung 64's
/// instantaneous root runs, which is what makes the initial running line identical to the machine
/// this rung is compared against"* — and the mechanism it describes is REAL. It is just delivered
/// entirely through `_close`, the sibling, and never once through the method whose docstring
/// claims it. A port spelling this *"if lagged, dispatch to rung 63"* agrees on all 35 994 oracle
/// keys (step 4's injection I1: **0 moved**).
#[test]
fn close_fuel_takes_the_steady_arm_only_when_the_state_is_dead_and_the_arms_differ() {
    let m = lagged_machine();
    let (nu_lp, nu_hp, _tt4, mf, tt2, pt2) = probe_state(&m);
    let core = &m.fuel;

    Census65::reset();
    let steady = r65_try_close_fuel(core, nu_lp, nu_hp, mf, tt2, pt2).expect("steady closes");
    let c = Census65::take();
    assert_eq!((c.close_fuel_steady, c.close_fuel_marched), (1, 0),
               "OUTSIDE a derivative evaluation a lagged `_close_fuel` must run rung 64's \
                instantaneous root — the arm the shipped grid never takes, and the reason the \
                initial running line matches the machine this rung is compared against");

    Census65::reset();
    let marched = {
        let _g = MarchedBleed::set(&m.fuel.inner, 0.5 * B);
        r65_try_close_fuel(core, nu_lp, nu_hp, mf, tt2, pt2).expect("marched closes")
    };
    let c = Census65::take();
    assert_eq!((c.close_fuel_steady, c.close_fuel_marched), (0, 1));

    assert_ne!(steady.base.phi_lp.to_bits(), marched.base.phi_lp.to_bits(),
               "the valve-AT-THE-STATE plant and the instantaneous-root plant must be two \
                different machines at this state");
}

/// **AND THE TWO ZEROS ARE ON OPPOSITE ARMS** — the fact that makes one idea written twice into
/// two gates. Read off ONE lagged march rather than asserted: `_close` is all-steady and
/// `_close_fuel` is all-marched, and neither method's test is ever exercised both ways.
#[test]
fn one_march_exercises_each_closure_on_exactly_one_arm_and_they_are_different_arms() {
    let m = lagged_machine();
    Census65::reset();
    m.stator_march(&flight(), &ramp(DS), None, &StatorLeg::default());
    let c = Census65::take();
    assert!(c.close_steady > 0 && c.close_marched == 0,
            "`_close` must be all-STEADY on a march: {c:?}");
    assert!(c.close_fuel_marched > 0 && c.close_fuel_steady == 0,
            "`_close_fuel` must be all-MARCHED on a march: {c:?}");
}

// =============================================================================================
// P4 — `InitialBleed` RESTORES THE PREVIOUS VALUE. `ForcedBleed`'s spelling would restore `None`.
// =============================================================================================

/// **THE MANUFACTURED NEST.** Probe 3 measured the `_b0` guard's max nesting depth at **1** over
/// rungs 62–65, so on every shipped path a restore-to-`None` guard and a restore-to-previous guard
/// agree, and step 4's injection I5 confirmed it from the other side: swapping the `Drop` body
/// moves **0 of 35 994** oracle keys.
///
/// The inner scope is a REAL march, not a bare guard pair — `stator_march_scoped` builds its own
/// [`InitialBleed`] from the scope it is handed, so this nests the shipped guard through the
/// shipped cell rather than exercising the type in isolation.
#[test]
fn the_b0_guard_restores_the_previous_value_through_a_manufactured_nest() {
    let m = lagged_machine();
    let core = &m.fuel.inner;
    assert!(core.b0.get().is_none(), "a fresh machine carries no overridden initial position");

    let outer = 0.4 * B;
    let inner = 0.7 * B;
    {
        let _g = InitialBleed::set(core, Some(outer));
        assert_eq!(core.b0.get(), Some(outer));

        // The nest: a march that sets `b0` to something else for its own duration.
        let (t, _) = m.stator_march_scoped(&flight(), &ramp(DS), None, &StatorLeg::default(),
                                           &MarchScope { b0: Some(inner) });
        // **AND THE SET HALF, WHICH THIS GATE DID NOT ORIGINALLY CHECK.** The step-5 mutation
        // census predicted that corrupting `r65_stator_march` to pass `None` instead of
        // `scope.b0` would fail here, and it did NOT: a guard that saves and restores the OUTER
        // value passes every assertion below no matter what it SETS. The two halves of one guard
        // were owned by two different files — the oracle caught the set at 1 280 keys, this
        // caught the restore — and neither caught both. This line closes that split.
        assert_eq!(match t[0].extra {
                       turbojet::fuel_transient::PointExtra::Valve { b, .. } => b,
                       _ => panic!("the nested march did not dispatch to the valve-lag integrator"),
                   }.to_bits(), inner.to_bits(),
                   "the scoped `b0` must reach the marcher, not merely be saved and put back");
        assert_eq!(core.b0.get(), Some(outer),
                   "P4: the inner guard must restore the value it DISPLACED. A `ForcedBleed`-style \
                    `Drop` (restore `None`) passes every value key in this slice and fails here — \
                    which is the only place it can fail.");

        // …and a march WITHOUT a `b0` is a real assignment too, not an omission: it must clear
        // the outer one for its own duration and put it back after.
        m.stator_march_scoped(&flight(), &ramp(DS), None, &StatorLeg::default(),
                              &MarchScope::DEFAULT);
        assert_eq!(core.b0.get(), Some(outer));
    }
    assert!(core.b0.get().is_none(), "the OUTER guard restores to what IT displaced");
}

/// The other half of the same claim, and the reason the three carriers are three types rather than
/// one generic one: [`MarchedBleed`] restores `None` because Python's `der` does, and nesting it
/// must NOT preserve an outer value.
#[test]
fn the_b_state_guard_restores_none_which_is_the_opposite_policy() {
    let m = lagged_machine();
    let core = &m.fuel.inner;
    {
        let _outer = MarchedBleed::set(core, 0.3 * B);
        assert_eq!(core.b_state.get(), Some(0.3 * B));
        {
            let _inner = MarchedBleed::set(core, 0.6 * B);
            assert_eq!(core.b_state.get(), Some(0.6 * B));
        }
        assert!(core.b_state.get().is_none(),
                "Python's `der` restores `_b_state` to None in its `finally`, so the inner scope \
                 CLEARS the outer one. Two guards, two policies — do not unify them.");
    }
}

// =============================================================================================
// `py_max3` — the defect step 4 found by asking a reader for its DEGENERATE case
// =============================================================================================

/// **PYTHON'S `max` IS NOT `f64::max`, AND THE DIFFERENCE IS `NaN` IN THE FIRST POSITION.**
///
/// `marginal_mode`'s `laws_held` is `float("nan")` on a cell with no riding points, and the
/// aggregate is `max(natural, lo, hi)` with `natural` FIRST. `f64::max` discards a NaN operand;
/// Python holds the first element and replaces it only on a strict `>`, and every comparison
/// against NaN is false. The port shipped `a.max(b).max(c)` until this was measured.
///
/// **NO VALUE GATE CAN REACH THE CALL SITE.** `n_ride` is 340 / 251 / 214 on natural/lo/hi and
/// 340 on both taucells, so no reachable march produces a NaN in any position — a gate on
/// `marginal_mode`'s OUTPUT would be satisfied by either spelling and would be vacuous. The
/// mutation census confirmed it: re-spelling the call site back to `f64::max` (injection I8) was
/// caught by nothing.
///
/// **SO THE CALL SITE IS GATED TEXTUALLY INSTEAD** — `include_str!` + `.matches().count()`, which
/// is the project's sanctioned replacement for a "this is not reachable" assertion (§ 6's
/// runtime-introspection table, already used at `test_rung73.py:488`'s port). It is not a value
/// gate and does not pretend to be one; it is the only instrument that can distinguish two
/// spellings that no input distinguishes.
#[test]
fn py_max3_is_pythons_max_and_not_f64_max() {
    // The call site, pinned by TEXT because no march can pin it by value. Both counts were
    // MEASURED against the shipped file before being typed.
    const SRC: &str = include_str!("../src/lagged_bleed.rs");
    assert_eq!(SRC.matches("laws_held: py_max3(nat.laws_held, lo.laws_held, hi.laws_held)")
                  .count(), 1,
               "`marginal_mode`'s `laws_held` must aggregate with `py_max3`. NO VALUE KEY IN THIS \
                SLICE CAN SEE THIS -- 35 994 oracle keys and 8 gates all pass on `f64::max` here \
                -- because the NaN that separates them needs a cell with `n_ride == 0` and the \
                shipped grid has none.");
    assert_eq!(SRC.matches("nat.laws_held.max(").count(), 0,
               "the `f64::max` chain must not come back at this call site");

    // The three positions, against the interpreter's own answers (measured, not recalled):
    //   max(nan, 1.0, 2.0) -> nan   ·   max(1.0, nan, 2.0) -> 2.0   ·   max(1.0, 2.0, nan) -> 2.0
    assert!(py_max3(f64::NAN, 1.0, 2.0).is_nan(),
            "a NaN in the FIRST position survives Python's `max`; `f64::max` would return 2.0");
    assert_eq!(py_max3(1.0, f64::NAN, 2.0), 2.0);
    assert_eq!(py_max3(1.0, 2.0, f64::NAN), 2.0);
    assert!(py_max3(f64::NAN, f64::NAN, f64::NAN).is_nan());
    // …and it is an ORDINARY max everywhere else, which is what the shipped grid actually uses.
    assert_eq!(py_max3(1.0, 3.0, 2.0), 3.0);
    assert_eq!(py_max3(-3.0, -5.0, -4.0), -3.0);
    assert_eq!(py_max3(2.0, 2.0, 2.0), 2.0);
    // The spelling this replaces, exhibited so the difference is in the file rather than in a
    // comment about the file.
    assert_eq!(f64::NAN.max(1.0).max(2.0), 2.0);
}

// =============================================================================================
// P6 — BOTH HALVES, REPORTED AT THE RUNG THE SLICE-X PANIC NAMED AS ITS OWN EXPIRY
// =============================================================================================

/// **P6, HALF ONE — and it is a statement about rung 64, not about the counter.**
///
/// `slice_x_dispatch.rs::the_lagged_position_override_is_declared_live_and_unreached` asserts
/// `b_of_state == 0`, and it stays GREEN because its four census marches all build rung-64
/// machines. § 5.23 (vi) predicted that BEFORE step 1 rather than discovering it at ship. This is
/// the other side of the same fact: on a rung-**65** machine the branch slice X declared dead is
/// live, so the two files are consistent only because they march different rungs.
#[test]
fn the_branch_slice_x_measured_dead_is_live_at_rung_65() {
    Census64::reset();
    lagged_machine().stator_march(&flight(), &ramp(DS), None, &StatorLeg::default());
    let c = Census64::take();
    assert!(c.b_of_state > 0,
            "rung 64 shipped `b_of`'s `b_state` override at 0 of 1 705 calls, gated by a \
             manufactured bug because a port that dropped it would break HERE. It did not break, \
             and this is the measurement that says why: {c:?}");
}

/// **P6, HALF TWO — `ForcedBleed`'s never-nests panic is NOT relaxed at rung 65.**
///
/// Its message names its own expiry: *"the paths that could break it are rung 65's closures and
/// the 16 `super(LimitedBleedTransient, self)` pin sites at rungs 66–75."* This slice is the first
/// of those. Re-measured at RUNTIME over rungs 62–65 (§ 5.23 (v)): max depth **1**, **0** nested
/// events in 501 506 trials — an upper bound from a name-based call graph replaced by a count.
///
/// The reason is structural and it is rung 65's own: `der` clears `_b_state` in a `finally`
/// BEFORE `command()` runs, and `command` binds rung 63's closure — so the trial solve never
/// re-enters rung 64's.
#[test]
fn forced_bleeds_never_nests_panic_still_stands_at_rung_65() {
    let m = lagged_machine();
    // A whole lagged march does not trip it.
    m.stator_march(&flight(), &ramp(DS), None, &StatorLeg::default());
    assert!(m.fuel.inner.b_forced.get().is_none(), "nothing may be left behind");

    // And the decision has not been quietly softened into a clobber.
    let core = &m.fuel.inner;
    assert!(panics(std::panic::AssertUnwindSafe(|| {
        let _outer = ForcedBleed::set(core, 0.02);
        let _inner = ForcedBleed::set(core, 0.03);
    })), "the guard must still REFUSE to nest — Python clobbers silently, the port is louder on \
          purpose, and relaxing that is a decision nobody has taken");
    assert!(core.b_forced.get().is_none(), "the unwind must still restore");
}
