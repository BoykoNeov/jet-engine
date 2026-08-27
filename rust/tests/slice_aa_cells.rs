//! SLICE AA step 1 — **THE NINE CELLS AND THE FOUR SCOPED FIELDS, AND NOTHING ELSE.**
//!
//! § 5.19 (x)'s rule for phase 7 is that *step 1 of every slice is the cell addition*, so a slice
//! that forgets a cell fails at its own first gate rather than at a value key nine rungs
//! downstream. Rung 68 is the phase's widest step on BOTH counts — nine cells (against slice X's
//! one and Y's and Z's zero) and four dynamically-scoped fields — so this file is what makes the
//! addition a measurement rather than a compile.
//!
//! **NOTHING HERE READS A GOLDEN**, which is slice V step 5's rule applied at the other end of the
//! slice: every assertion is a panic, a counter-free property of a shipped function, or a
//! same-run difference between two guard spellings. Step 2's bodies cannot make any of them pass
//! or fail by accident.
//!
//! # What it asserts, and why each one is not free
//!
//! * **All nine [`NO_TRIPLE`] cells panic**, on a real rung-65 machine, one test per cell. The
//!   tempting default is `stator_leg -> None` and `lagged_stator -> false`, which would agree with
//!   the truth on exactly the machines the suites build — **a claim no value gate could see.**
//!   [`NO_STATOR`](turbojet::stator_transient::NO_STATOR)'s precedent, third use in the port.
//! * **[`ForcedStator`] PANICS on a same-field nest** where [`InitialStator`] and
//!   [`DeclaredOrder`] restore the value they displaced. The two policies are invisible to every
//!   value key on every shipped path — § 5.25 (iii) measured **0 overwrites in 811 632 sets** —
//!   so the nest is MANUFACTURED here, exactly as `slice_y_dispatch.rs` manufactures `_b0`'s.
//! * **[`DeclaredOrder`]'s absent argument re-asserts the current value rather than clearing it.**
//!   Python is `ic_order or self._ic_order`, and a port that spelled it `set(None)` would leave
//!   the field as `None` — a difference no rung-68 march can reach, because every march passes
//!   through the same guard.
//! * **[`MarchScope`] grew by two fields and no existing literal moved** — slice Z's P1 verdict
//!   (*growth is free from the SECOND time on*) re-tested at the growth it predicted.

use std::panic::catch_unwind;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::fuel_transient::{AsymmetricLag, Floor, PointExtra, SurgeLimiter};
use turbojet::three_loop::{
    ic_at_point, round12, v_at_point, StatorLegArm, StatorLimiter, TripleRigArm,
    IC_ORDER_DECLARED,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{
    DeclaredOrder, ForcedStator, InitialStator, MarchedStator, TwoSpoolTransientCore,
};

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const B: f64 = 0.10;
const TAU: f64 = 0.05;
/// Rungs 57/58's swept setting `V = 0.20`, INHERITED rather than chosen — rung 68 adds no new
/// constant, and this file must not be the place one appears.
const V_MAX: f64 = 0.20;
/// `PHI / FLOOR - 1.0` — the suite's own spelling.
const SM: f64 = PHI / FLOOR - 1.0;

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

/// A REAL rung-65 machine — the highest rung the port has shipped, and therefore the one whose
/// triple table is still [`NO_TRIPLE`]. Building the panics on a live object rather than on a
/// hand-made core is what makes "unreachable by construction" a statement about the ladder.
fn lagged_machine() -> ScheduledStatorCore {
    let arm = LeverArm::floored(BleedLimiter::with_tau(PHI, B, Some(TAU)));
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn panics<F: FnOnce() + std::panic::UnwindSafe>(f: F) -> bool {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f).is_err();
    std::panic::set_hook(prev);
    out
}

fn leg() -> StatorLegArm { StatorLegArm { v_max: V_MAX, tau: Some(TAU) } }

// =============================================================================================
// GATE 1 — THE NINE CELLS EXIST AND THEIR DEFAULT PANICS
// =============================================================================================

/// **The whole point of step 1.** Nine names, nine panics, on a machine the ladder really builds.
///
/// A cell that does not exist cannot panic, so a missing cell fails HERE rather than at a value
/// key in slice AB. And a cell whose default ANSWERED — `None`, `false`, `0.0` — would agree with
/// the truth on every rung-40..67 machine the suites construct, which is precisely the class of
/// defect this port has been caught on: *a claim no value gate could see*.
#[test]
fn all_nine_no_triple_cells_panic_and_none_of_them_answers() {
    let m = lagged_machine();
    let core: &TwoSpoolTransientCore = &m.fuel.inner;

    assert!(panics(|| { let _ = lagged_machine().fuel.inner.stator_leg(); }), "stator_leg");
    assert!(panics(|| { let _ = lagged_machine().fuel.inner.lagged_stator(); }), "lagged_stator");
    assert!(panics(|| { let _ = lagged_machine().fuel.inner.clamp_v(0.0, &leg()); }), "clamp_v");
    assert!(panics(|| lagged_machine().fuel.inner.check_v0(0.0, &leg())), "check_v0");
    assert!(panics(|| lagged_machine().fuel.inner.rk4_floor(0.01, 20.0, 3, TAU)), "rk4_floor");
    assert!(panics(|| { let _ = lagged_machine().fuel.inner.solve_v(&|_| unreachable!()); }),
            "solve_v");
    assert!(panics(|| {
        let _ = lagged_machine().manifold_v(&flight(), 1.0, 1.0, 1.0, 0.0, 0.0, &|_, _| {
            unreachable!()
        });
    }), "manifold_v");
    assert!(panics(|| {
        let _ = lagged_machine().triple_laws(&flight(), 1.0, 1.0, 1.0, None, None);
    }), "triple_laws");
    assert!(panics(|| {
        let _ = lagged_machine().triple_rig(&TripleRigArm::default());
    }), "triple_rig");

    // **AND THE PANICS ARE THE DEFAULT TABLE'S, NOT SOME OTHER FAILURE.** Nine `assert!(panics)`
    // calls are satisfied by nine bugs as readily as by nine unreachable-by-construction cells,
    // which is this port's own recurring shape — a gate whose pass condition its own defect
    // satisfies. So the MESSAGE is read.
    //
    // **NOT `ptr::eq(core.triple_hooks, &NO_TRIPLE)`, WHICH IS WHAT THIS ASSERTION FIRST WAS AND
    // WHICH FAILED.** A `const` has no single address: `&NO_TRIPLE` in this file is a fresh
    // temporary, so the comparison tests the optimiser rather than the table — slice Y step 3's
    // lesson, reproduced here by making the same mistake.
    let msg = message_of(|| { let _ = lagged_machine().fuel.inner.stator_leg(); });
    assert!(msg.contains("no triple table on this object") && msg.contains("(_stator_leg)"),
            "the panic must be NO_TRIPLE's own, not an unrelated failure; got: {msg}");
    let _ = core;
}

/// The panic message a closure produces, or `""` if it did not panic.
fn message_of<F: FnOnce() + std::panic::UnwindSafe>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f);
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
// GATE 2 — THE FOUR SCOPED FIELDS, AND THE TWO RESTORE POLICIES
// =============================================================================================

/// The four fields start where Python's class attributes start: three `None` and `_ic_order` at
/// the DECLARED order.
///
/// `"gqv"` is not a default parameter — it is a class attribute Python reads off `self`, which is
/// what makes it overridable per march. A port that spelled it as a parameter default would leave
/// `ic_family`'s whole instrument unreachable.
#[test]
fn the_four_scoped_fields_start_at_pythons_class_attributes() {
    let m = lagged_machine();
    let c = &m.fuel.inner;
    assert_eq!(c.v_forced.get(), None);
    assert_eq!(c.v_state.get(), None);
    assert_eq!(c.v0.get(), None);
    assert_eq!(c.ic_order.get(), IC_ORDER_DECLARED);
    assert_eq!(IC_ORDER_DECLARED, "gqv");
}

/// **THE TWO POLICIES, AND THE MANUFACTURED NEST THAT SEPARATES THEM.**
///
/// `_v_forced` restores to `None`; `_v0` and `_ic_order` restore the value they displaced. On
/// every shipped path the two spellings agree — § 5.25 (iii) measured **0 overwrites in 811 632
/// sets** across the restore-to-`None` fields — so no march can tell them apart and the nest has
/// to be built by hand.
#[test]
fn forced_restores_to_none_and_initial_restores_the_previous_value() {
    let m = lagged_machine();
    let c = &m.fuel.inner;

    // restore-to-None
    {
        let _g = ForcedStator::set(c, -0.05);
        assert_eq!(c.v_forced.get(), Some(-0.05));
    }
    assert_eq!(c.v_forced.get(), None);

    {
        let _g = MarchedStator::set(c, -0.03);
        assert_eq!(c.v_state.get(), Some(-0.03));
    }
    assert_eq!(c.v_state.get(), None);

    // restore-PREVIOUS, nested. `ForcedBleed`'s spelling would leave `None` after the inner guard
    // drops, and the outer scope would silently lose its own override.
    {
        let _outer = InitialStator::set(c, Some(-0.10));
        assert_eq!(c.v0.get(), Some(-0.10));
        {
            let _inner = InitialStator::set(c, Some(-0.02));
            assert_eq!(c.v0.get(), Some(-0.02));
        }
        assert_eq!(c.v0.get(), Some(-0.10),
                   "InitialStator must restore the value it DISPLACED, not None -- Python is \
                    `prev_v, self._v0 = self._v0, v0` ... `finally: self._v0 = prev_v`");
    }
    assert_eq!(c.v0.get(), None);

    {
        let _outer = DeclaredOrder::set(c, Some("vqg"));
        assert_eq!(c.ic_order.get(), "vqg");
        {
            let _inner = DeclaredOrder::set(c, Some("qgv"));
            assert_eq!(c.ic_order.get(), "qgv");
        }
        assert_eq!(c.ic_order.get(), "vqg");
    }
    assert_eq!(c.ic_order.get(), IC_ORDER_DECLARED);
}

/// Python's `ic_order or self._ic_order` — **an absent argument re-asserts the CURRENT value, and
/// does not clear the field.**
///
/// The difference is unreachable from any march (every one passes through the same guard), and it
/// is the difference between `ic_family` sweeping six orders and sweeping one.
#[test]
fn an_absent_order_re_asserts_the_current_one_rather_than_clearing_it() {
    let m = lagged_machine();
    let c = &m.fuel.inner;
    let _outer = DeclaredOrder::set(c, Some("qvg"));
    {
        let _inner = DeclaredOrder::set(c, None);
        assert_eq!(c.ic_order.get(), "qvg",
                   "`ic_order or self._ic_order` keeps the outer order; a `set(None)` port would \
                    leave the field empty and silently re-declare rung 66's member");
    }
    assert_eq!(c.ic_order.get(), "qvg");
}

/// **THE PANIC IS A PORT DECISION AND IT IS LOUDER THAN THE SOURCE**, so it is asserted rather
/// than left to a reader of the message.
///
/// Python's `_closer_v` restores to `None`, so a nested guard would CLOBBER the outer trial and
/// carry on. § 5.25 (iii) measured the field at 0 overwrites in 478 511 sets — **on rung-68
/// machines only**, which is why the assert stays and the scope is written into its message.
#[test]
fn a_nested_forced_stator_panics_where_python_would_clobber() {
    let m = lagged_machine();
    let c = &m.fuel.inner;
    let _outer = ForcedStator::set(c, -0.05);
    assert!(panics(|| {
        let mm = lagged_machine();
        let cc = &mm.fuel.inner;
        let _o = ForcedStator::set(cc, -0.05);
        let _i = ForcedStator::set(cc, -0.01);
    }), "a same-field nest on `_v_forced` must panic, not clobber");
}

// =============================================================================================
// GATE 3 — THE DEVICE
// =============================================================================================

/// All three of `__post_init__`'s refusals, in Python's order, plus the two they distinguish.
///
/// **`v_max = 0` is REFUSED and not silently reduced**: a limiter that cannot act is a DIFFERENT
/// object from an absent one (that is `stator_lim=None`), and the distinction is the whole rung —
/// the ceiling belongs to `v_max`. Rung 64's assert on the valve, one lever over.
#[test]
fn the_stator_limiters_three_refusals() {
    assert!(panics(|| { StatorLimiter::new(0.0, V_MAX, Some(TAU)); }), "phi_lim > 0");
    assert!(panics(|| { StatorLimiter::new(PHI, 0.0, Some(TAU)); }), "v_max = 0 is refused");
    assert!(panics(|| { StatorLimiter::new(PHI, 1.0, Some(TAU)); }), "|v| >= 1 is refused");
    assert!(panics(|| { StatorLimiter::new(PHI, V_MAX, Some(0.0)); }), "tau = 0 is not tau = None");
    // ...and the two that stand: an instantaneous loop is a DIFFERENT object, not a refused one.
    let inst = StatorLimiter::new(PHI, V_MAX, None);
    assert_eq!(inst.tau, None);
    let lag = StatorLimiter::new(PHI, V_MAX, Some(TAU));
    assert_eq!(lag.tau, Some(TAU));
}

/// `from_margin` is rung 49's and rung 64's **verbatim**, which is what makes all three floors ONE
/// set point rather than three numbers that happen to agree. § 2's identity needs exactly that, so
/// the equality is asserted against the valve's own constructor rather than against a literal.
#[test]
fn from_margin_is_the_same_set_point_as_the_valves() {
    let cmap = lp_map();
    for sm in [0.0, 0.05, 0.2545] {
        let s = StatorLimiter::from_margin(&cmap, V_MAX, sm, Some(TAU));
        let v = BleedLimiter::from_margin_tau(&cmap, B, sm, Some(TAU));
        assert_eq!(s.phi_lim, v.phi_lim,
                   "one variable is not one set point: rung 66 § 2 measured a -2.5 % offset \
                    moving the product to 0.951");
        assert_eq!(s.phi_lim, (1.0 + sm) * cmap.phi_surge);
    }
    assert!(panics(|| {
        StatorLimiter::from_margin(&ComponentMap::flat(), V_MAX, 0.0, None);
    }), "from_margin needs a surge line");
    assert!(panics(|| { StatorLimiter::from_margin(&lp_map(), V_MAX, -0.1, None); }),
            "the floor sits AT or ABOVE the surge line");
}

/// [`StatorLegArm`] narrows `_stator_leg`'s return to the two fields every caller of it reads.
///
/// Python's `_stator_leg` hands back the limiter OBJECT, and rung 69 hands back a **different
/// type** from the same name. Read body by body, the callers touch `.tau` and `.v_max` and never
/// the limit itself — `_solve_v` reads its own limiter off `self` on both rungs — so the
/// conversion is total and slice AB's incidence limiter feeds the same shape.
#[test]
fn the_leg_arm_carries_exactly_what_its_callers_read() {
    let l = StatorLimiter::new(PHI, V_MAX, Some(TAU));
    let arm: StatorLegArm = l.into();
    assert_eq!(arm.v_max, V_MAX);
    assert_eq!(arm.tau, Some(TAU));
}

/// **`round12` HAS EXACTLY ONE CONSUMER AND THAT CONSUMER RETURNS 1**, so the oracle's 12 084 keys
/// say nothing about the function.
///
/// `ic_family`'s `order_members` counts the DISTINCT members of the `s = 0` family, and on the
/// shipped grid every sweep order lands on the same one — a count of **1**, which is satisfied by
/// any rounding whatsoever, including none. That is this port's own recurring shape: *a gate that
/// reads a key only in a way that cannot distinguish its values.* So the function is pinned
/// directly, against PyPy's own answers.
///
/// **The reference column is EMITTED by `probe_aa9.py`, not retyped**, and the cases are chosen to
/// separate correct decimal rounding (ties-to-even, which `{:.12}` and Python's `round` both do)
/// from the scaled-arithmetic spelling `(x*1e12).round()/1e12`, which rounds an already-inexact
/// product. `round6` and `round3` closed the same class by construction at other widths.
#[test]
fn round12_agrees_with_pythons_round_on_the_ties_that_discriminate() {
    // (input, PyPy `round(x, 12)` as IEEE-754 bits)
    const CASES: [(f64, u64); 19] = [
        (0.0, 0),
        (-0.0, 0),
        (1.0, 4607182418800017408),
        (-1.0, 13830554455654793216),
        // rung 66's own b0 — the value the consumer actually rounds.
        (0.0366255144032922, 4585439113932357745),
        // the tau_s = 500 march's v_min, which is the slice's one CPython-exempt march key.
        (-7.4768789476212245e-06, 13753813203431271770),
        (5e-13, 0),
        (1.5e-12, 4431990193862339089),
        (2.5e-12, 4431990193862339089),
        (1.0000000000005, 4607182418800021912),
        (1.5e-13, 0),
        (2.5e-13, 0),
        (3.5e-13, 0),
        (123.4567890123455, 4638387916139875433),
        (123.4567890123465, 4638387916139875574),
        (1e-13, 0),
        (9.999999999995e-13, 4427486594234968593),
        (9.094947017729282e-13, 4427486594234968593),
        (4.547473508864641e-13, 0),
    ];
    let mut scaled_disagreements = 0usize;
    for (x, want) in CASES {
        // The signed-zero normalisation `ic_family` applies, so this gate reads the key the
        // consumer actually builds rather than a function the consumer does not call.
        let got = (round12(x) + 0.0).to_bits();
        assert_eq!(got, want,
                   "round12({x:?}) = {:?} ({got:016x}), PyPy says {:?} ({want:016x})",
                   f64::from_bits(got), f64::from_bits(want));
        // ...and the WRONG spelling is measured rather than merely warned against, so this gate
        // is known to be able to fail.
        if ((x * 1e12).round() / 1e12 + 0.0).to_bits() != want {
            scaled_disagreements += 1;
        }
    }
    assert!(scaled_disagreements >= 3,
            "this gate must DISCRIMINATE: the scaled spelling `(x*1e12).round()/1e12` disagrees \
             on only {scaled_disagreements} of the {} cases, so it is not testing the thing it \
             names. Add ties until it does.", CASES.len());
    println!("round12: {}/{} cases, and the scaled spelling fails {} of them",
             CASES.len(), CASES.len(), scaled_disagreements);
}

/// **THE TWO REFUSALS THAT NOTHING ELSE IN THE SLICE READS.**
///
/// `v_at_point` and `ic_at_point` panic on a trajectory that did not record their keys, because
/// Python raises there and nothing in the ladder catches it. **Every other file in the slice calls
/// them only on `Triple` points**: the oracle's sections J and H, `rung68.rs`'s gates, and the
/// dispatch file all march a rung-68 machine first. So a port that returned `0.0` instead of
/// panicking would pass all 45 tests.
///
/// That is the same class as the defect the oracle caught at `v_max_used` — **a defence with no
/// reader in this slice** — and it is the class this port has been caught on repeatedly. The
/// refusal is therefore reached deliberately, from a rung-66 machine, and the MESSAGE is read: a
/// bare `catch_unwind` would be satisfied by any panic, including one from the march itself.
#[test]
fn the_two_point_readers_refuse_a_trajectory_that_did_not_record_them() {
    // A rung-66 machine: the valve is lagged, the stator absent. Its points carry `b`/`g` and
    // `ic_iters`/`ic_res`, so this is NOT a trajectory with no extras at all — it is the nearest
    // NEIGHBOUR, which is what makes the refusal a statement about `v` and not about emptiness.
    let arm = LeverArm { bleed_lim: Some(BleedLimiter::with_tau(PHI, B, Some(TAU))),
                         ..Default::default() };
    let m = match turbojet::two_lag::build_two_lag_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    let leg = StatorLeg {
        accel: None::<&turbojet::fuel_transient::AccelSchedule>,
        surge: Some(Floor::Phi(SurgeLimiter::from_margin(&lp_map(), Spool::Lp, SM))),
        tt4_max: None };
    let (traj, _) = m.stator_march_scoped(
        &flight(), &Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.02 },
        None, &leg,
        &MarchScope { lag: Some(AsymmetricLag::new(0.05, 0.15)), ..MarchScope::DEFAULT });
    assert!(!traj.is_empty());
    // The neighbour really is a rung-66 trajectory — otherwise the two refusals below would be
    // measuring an empty march.
    assert!(matches!(traj[0].extra, PointExtra::Cascade { .. }),
            "this gate needs rung 66's four-state march, not some other route");

    let p0 = traj[0];
    let m1 = message_of(move || { let _ = v_at_point(&p0); });
    assert!(m1.contains("cannot be recovered from a trajectory point that did not record it"),
            "v_at_point must REFUSE a rung-66 point, not answer 0.0; got {m1:?}");
    let p1 = traj[0];
    let m2 = message_of(move || { let _ = ic_at_point(&p1); });
    assert!(m2.contains("carries no joint initial condition"),
            "ic_at_point must REFUSE it too; got {m2:?}");

    // ...and both ANSWER on the rung-68 trajectory, so the gate is not satisfied by a reader that
    // panics unconditionally.
    let armed = LeverArm { bleed_lim: Some(BleedLimiter::with_tau(PHI, B, Some(TAU))),
                           stator_lim: Some(StatorLimiter::new(PHI, V_MAX, Some(TAU))),
                           ..Default::default() };
    let m68 = match turbojet::three_loop::build_three_loop_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &armed) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    let (t68, _) = m68.stator_march_scoped(
        &flight(), &Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.02 },
        None, &leg,
        &MarchScope { lag: Some(AsymmetricLag::new(0.05, 0.15)), ..MarchScope::DEFAULT });
    assert_eq!(v_at_point(&t68[0]), 0.0, "the declared start is the dormant stop");
    assert_eq!(ic_at_point(&t68[0]).2, IC_ORDER_DECLARED);
}

// =============================================================================================
// GATE 4 — THE GROWTH, AND WHAT IT COST
// =============================================================================================

/// Slice Z's P1 verdict re-tested at the growth it predicted: **`MarchScope` grew by two fields
/// and no un-scoped caller moved.**
///
/// The `..MarchScope::DEFAULT` spread that slice Z paid for at nine literals absorbs slice AA's
/// two additions silently, which is the measurement behind *"growth is free from the SECOND time
/// on"*. That the crate still compiles is the other half; this asserts the values.
#[test]
fn march_scope_grew_by_two_and_the_default_still_means_no_override() {
    let d = MarchScope::DEFAULT;
    assert_eq!(d.v0, None);
    assert_eq!(d.ic_order, None);
    // A partial literal over the spread — the spelling every shipped call site uses.
    let s = MarchScope { v0: Some(-0.05), ..MarchScope::DEFAULT };
    assert_eq!(s.v0, Some(-0.05));
    assert_eq!(s.ic_order, None);
    assert_eq!(s.b0, None);
    assert_eq!(s.lag, None);
    assert_eq!(s.tau_gov, None);
}

/// `at_lever` goes to its EIGHTH keyword here, exactly as `bleed_lim`'s own note predicted, and a
/// bare sibling still carries no floor.
#[test]
fn the_lever_arm_carries_the_eighth_keyword_and_defaults_it_absent() {
    assert_eq!(LeverArm::default().stator_lim, None);
    assert_eq!(LeverArm::floored(BleedLimiter::with_tau(PHI, B, Some(TAU))).stator_lim, None);
    let l = StatorLimiter::new(PHI, V_MAX, Some(TAU));
    let arm = LeverArm { stator_lim: Some(l), ..Default::default() };
    assert_eq!(arm.stator_lim, Some(l));
    // `merged` takes the LEVER's field where set and the NEIGHBOUR's otherwise — the same rule
    // every other field on this struct follows.
    let merged = LeverArm::merged(&arm, &LeverArm::default());
    assert_eq!(merged.stator_lim, Some(l));
    let merged2 = LeverArm::merged(&LeverArm::default(), &arm);
    assert_eq!(merged2.stator_lim, Some(l));
}
