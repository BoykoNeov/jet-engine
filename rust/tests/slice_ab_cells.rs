//! SLICE AB step 1 — **ONE ADDED CELL, NINE OPENED SWAPS, AND THE FOUR GUARDS.**
//!
//! § 5.19 (x)'s rule for phase 7 is that *step 1 of every slice is the cell addition*, so a slice
//! that forgets a cell fails at its own first gate rather than at a value key nine rungs
//! downstream. **That rule buys almost nothing here, and the pre-flight said so before a line was
//! written** (§ 5.26 (ii)): slice AA ADDED nine cells, this one adds **one** and SWAPS **ten** —
//! and a forgotten swap is not a missing function, it is the parent's, which compiles, runs, and
//! is caught by nothing.
//!
//! **THE TWO TENS ARE DIFFERENT TENS, AND THE ARITHMETIC IS WRITTEN OUT ONCE** (in
//! [`the_triple_table_is_exactly_ten_cells_wide`]), because an unreconciled pair of counts in two
//! files is this phase's most-repeated defect:
//!
//! * **10 SWAPS** = the 9 cells rung 69 overrides + `__init__`, which is **not a cell** — no
//!   shipped table carries a constructor hook, and it ports as the builder's four `assert!`s.
//! * **10 TABLE CELLS** = those same 9 + the one this rung ADDS, `with_ref`.
//!
//! So the step-1 gate was inverted to match the risk:
//!
//! * ~~**Every swapped cell has a DISTINCT rung-69 function pointer**, asserted by reading nine
//!   panic messages that name themselves.~~ **DISMANTLED AT STEP 2, as § 5.26.1 (k) said it would
//!   be.** That gate's whole content was *"not yet ported"*: it read the nine placeholder panics,
//!   and step 2 replaced every one of them with a body. Nothing in the crate emits that message
//!   any more, so re-adding the gate would be a tautology of exactly the kind this file's own
//!   comments keep catching. **The step-5 dispatch gates are its successor**, and they ask the
//!   stronger question the placeholders were standing in for: swap each cell for rung 68's
//!   function pointer and see which gate breaks.
//! * **The one ADDED cell is real and rung 68's slot PANICS**, because `_with_ref` does not exist
//!   below rung 69 in Python at all and a slot answering `None` would agree with the truth on
//!   exactly the machines the rung-68 suite builds.
//! * **The four `__init__` guards fire, by MESSAGE**, on the 96-point arming grid's own witnesses.
//! * **`StatorIncidenceLimiter` has TWO asserts where rung 68's limiter has three**, and the
//!   missing one is a decision: at `sm = 0` — the floor ON the surge line, which `from_margin`'s
//!   own assert admits — `m_lim` is EXACTLY zero, so a copied-over `phi_lim > 0` would refuse it.
//!   (The first draft of that gate justified the absence differently and was **wrong**; see the
//!   gate's own comment.)
//! * **P1 and P5 — `MarchScope` and `StatorLegArm` do NOT grow**, both spelled as exhaustive
//!   literals so the compiler is the assertion.
//!
//! **NOTHING HERE READS A GOLDEN.** Slice V step 5's rule at the other end of the slice: every
//! assertion is a panic, a same-run difference, or a compile-time property. Step 2's bodies cannot
//! make any of them pass or fail by accident.

use std::panic::catch_unwind;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{
    build_reference_split_cascade, Census69, RefScope, StatorIncidenceLimiter, R69_TRIPLE,
};
use turbojet::stator_transient::{
    MarchScope, ScheduledStatorCore, ScheduledStatorTransient, StatorArm,
};
use turbojet::three_loop::{
    build_three_loop_cascade, StatorLegArm, StatorLimiter, TripleHooks, NO_TRIPLE,
};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const B: f64 = 0.10;
const TAU: f64 = 0.05;
/// Rungs 57/58's swept setting `V = 0.20`, INHERITED rather than chosen — rung 69 adds no new
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

fn valve() -> BleedLimiter { BleedLimiter::with_tau(PHI, B, Some(TAU)) }

/// The rung-69 incidence floor matched to [`valve`]'s `phi` floor at the DESIGN setting — the ONE
/// PHYSICAL WALL guard D exists to enforce, built the way the guard's own message says to.
fn inc() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU))
}

/// A REAL rung-69 machine. The nine placeholder panics are asserted on one of these rather than on
/// a hand-made core, because "the table the ladder actually installs" is the claim.
fn split_machine(arm: &LeverArm) -> ScheduledStatorCore {
    match build_reference_split_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// The default arming every gate below starts from: rung 65's valve under a floor, plus rung 69's
/// incidence stator on the SAME physical wall.
fn split_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_inc: Some(inc()), ..Default::default() }
}

/// A rung-**68** machine, for the two gates that need rung 69's slot to be somebody else's.
fn three_loop_machine() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(valve()),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU))),
        ..Default::default()
    };
    match build_three_loop_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// A rung-**65** machine — the last rung whose triple table is [`NO_TRIPLE`].
fn lagged_machine() -> ScheduledStatorCore {
    let arm = LeverArm::floored(valve());
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// The panic message a closure produces, or `""` if it did not panic.
///
/// **THE MESSAGE AND NOT THE FACT OF A PANIC.** `assert!(panics(…))` nine times over is satisfied
/// by nine unrelated bugs as readily as by nine opened cells — this port's own recurring shape, a
/// gate whose pass condition its own defect satisfies.
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
// GATE 1 — THE INHERITED CELL, AND THE ONE ADDED CELL
// =============================================================================================

/// **`triple_laws` IS THE ONE OF THE NINE THIS RUNG DOES *NOT* SWAP, AND THAT IS A DECISION.**
///
/// Rung **70** overrides it, alone among rung 68's nine. Spelling [`R69_TRIPLE`] with a
/// `..R68_TRIPLE` spread would have inherited it for free — and would have inherited the other
/// eight for free too, which is the defect the file exists to refuse. So the inheritance is
/// written out, and this asserts it landed on the ONE cell it was meant to.
///
/// **THE ASSERTION IS POSITIVE, AND THE FIRST DRAFT OF IT COULD NOT FAIL.** It read
/// `assert!(!msg.contains(": _triple_laws"))` — and no function in the crate emits that string.
/// [`NO_TRIPLE`](turbojet::three_loop::NO_TRIPLE)'s body spells the name `(_triple_laws)` in
/// parentheses, and there is no `r69_triple_laws` at all, so the predicate was a tautology on
/// every possible input: slice V step 2's *"both gates written to CLOSE a step could not fail"*,
/// for the third time in this phase.
///
/// So the message was MEASURED instead of predicted, and it is **empty** — rung 68's body builds
/// its three closures lazily and dispatches none of the eight cells rung 69 swaps.
///
/// **AND STEP 2 HAD TO RE-MEASURE IT, BECAUSE THE STEP-1 FORM WENT HALF-VACUOUS THE MOMENT THE
/// BODIES LANDED.** *"It does not panic"* was a real statement only while nine slots panicked;
/// with them ported, nothing in the crate emits that message and the assertion survives only as a
/// check that the slot is not [`NO_TRIPLE`](turbojet::three_loop::NO_TRIPLE)'s. The POSITIVE half
/// is what replaces it: rung 68's body is LAZY, so a rung-69 machine's `triple_laws` call must
/// dispatch **none** of rung 69's cells — [`Census69`] reads all-zero afterwards. That fails if
/// the slot is ever pointed at an eager body, and it is the measurement the placeholders were
/// only standing in for.
#[test]
fn triple_laws_is_inherited_from_rung68_and_dispatches_none_of_rung69s_cells() {
    let m = split_machine(&split_arm());
    // AFTER the build: the steady solve inside it reaches `_stator_leg` several times, which is
    // this rung's own cell and not the question here.
    Census69::reset();
    // `AssertUnwindSafe` because the machine is built OUTSIDE the closure -- it has to be, or the
    // build's own `_stator_leg` calls would land inside the census window. It carries `Cell`s, so
    // the compiler asks whether a half-torn state could leak out of the unwind; nothing reads `m`
    // after this call, which is what makes the assertion true rather than convenient.
    let msg = message_of(std::panic::AssertUnwindSafe(|| {
        let _ = m.triple_laws(&flight(), 1.0, 1.0, 1.0, None, None);
    }));
    assert!(msg.is_empty(),
            "`triple_laws` must be RUNG 68's -- rung 70 overrides it, not rung 69 -- and rung \
             68's body returns Ok here. Got: {msg:?}");
    assert_eq!(Census69::read(), Census69::default(),
               "rung 68's `_triple_laws` builds its three laws LAZILY: it must reach no rung-69 \
                cell at all until one of the returned closures is called. A census that is not \
                all-zero here means the body dispatched something eagerly, which would put the \
                `b_state`/`v_state` boundary -- this rung's own trap in its fourth shape -- on \
                the wrong side of the call.");
}

/// **THE TABLE HAS EXACTLY TEN CELLS, AND THE COMPILER SAYS SO.**
///
/// [`UNPORTED_AT_STEP1`] and the nine entries [`all_nine_swapped_cells_are_open_and_each_names_itself`]
/// exercises are BOTH hand-written, so comparing their lengths compares two typed numbers — a
/// cell forgotten in both passes, which is
/// [[rust-port-documented-gate-that-doesnt-exist]] exactly: *a count guard is blind to a class
/// absent from BOTH sides.*
///
/// This is the non-typed instrument. The literal has no `..` spread, so a fourteenth field is
/// `E0063` **here**, at the file whose job is the cell census, rather than being silently
/// absorbed by a spread nine rungs downstream.
///
/// **THIS COMMENT USED TO NAME SLICE AC AS THE ELEVENTH FIELD's AUTHOR, AND THAT WAS WRONG.**
/// § 5.19 (x)'s cell column said slice AC adds `split_gains`; § 5.27 (i) measured that rung 80's
/// same-named method drops four parameters and adds five, so rung 70's own inherited caller
/// `rung67_control` raises `TypeError` on a rung-80 machine. Two functions sharing a name cannot
/// share a `fn` pointer: **slice AC adds no cell and this table stays thirteen fields wide.** The
/// addressee is deliberately not replaced with another slice's letter — the old one was written
/// on the strength of a predicate (*new here AND overridden above*) that is by NAME and never
/// checked substitutability, and naming the next slice from the same column would repeat the
/// error one addressee over. The tripwire needs no addressee to work.
/// **UPDATED AT SLICE AD, AND THE UPDATE IS THE TRIPWIRE WORKING.** Rung 72 adds `reference`,
/// `rk4_floor_shared` and `shared_rig`, and this literal stopped compiling — which is the entire
/// point of spelling the fields out. Slice AD's own P1 predicted five `E0063` sites (the five
/// `TripleHooks` consts in `src`) and the landed edit needed **seven**: `cargo check` stops at the
/// lib, so the probe never reached this file or its sibling. The mechanism P1 was testing —
/// exhaustive literals go loud, `..` spreads and whole-const aliases stay silent — held; only its
/// count was short, and short for a reason worth more than the number.
#[test]
fn the_triple_table_is_exactly_thirteen_cells_wide() {
    let _pin = TripleHooks {
        stator_leg: NO_TRIPLE.stator_leg,
        lagged_stator: NO_TRIPLE.lagged_stator,
        clamp_v: NO_TRIPLE.clamp_v,
        check_v0: NO_TRIPLE.check_v0,
        rk4_floor: NO_TRIPLE.rk4_floor,
        solve_v: NO_TRIPLE.solve_v,
        manifold_v: NO_TRIPLE.manifold_v,
        triple_laws: NO_TRIPLE.triple_laws,
        triple_rig: NO_TRIPLE.triple_rig,
        with_ref: NO_TRIPLE.with_ref,
        // THE THREE SLICE AD ADDS — this file failed to compile until they
        // were written, which is what a width tripwire is for.
        reference: NO_TRIPLE.reference,
        rk4_floor_shared: NO_TRIPLE.rk4_floor_shared,
        shared_rig: NO_TRIPLE.shared_rig,
    };
    // 10 TABLE CELLS = 9 that rung 69 SWAPS + 1 it ADDS (`with_ref`) -- the literal above.
    // 10 SWAPS       = those same 9 cells + `__init__`, which is NOT a cell (no shipped table
    //                  carries a constructor hook; it ports as the builder's four `assert!`s).
    //
    // **AND THAT ARITHMETIC IS A COMMENT, NOT AN ASSERTION, BECAUSE THE ASSERTION WAS A THIRD
    // TAUTOLOGY.** It read `assert_eq!(UNPORTED_AT_STEP1.len() + 1, 10)` -- and the array is
    // `[&str; 9]`, so `.len()` is 9 at compile time and the whole line is `assert_eq!(10, 10)`.
    // Written INSIDE the test whose own doc comment explains that class. The exhaustive literal
    // above is the real gate and needs no help from a tally.
}

// =============================================================================================
// GATE 2 — THE ONE ADDED CELL, AND RUNG 68's SLOT FOR IT
// =============================================================================================

/// **`_with_ref` DOES NOT EXIST BELOW RUNG 69, SO RUNG 68's SLOT PANICS.**
///
/// The tempting default is `None` — "no reference selected" — and it is the move this family has
/// been caught on four times: it agrees with the truth on exactly the machines the rung-40..68
/// suites build, so **no value key could ever see it**.
/// [`b_at_point`](turbojet::bleed_transient::LeverHooks::b_at_point)'s precedent, second use.
#[test]
fn with_ref_panics_on_every_machine_below_rung69() {
    // The machine is built INSIDE the closure — a `&Core` cannot cross a `catch_unwind` boundary
    // because the carriers this slice adds are `Cell`s, which is the borrow checker restating
    // exactly why they need a guard.
    let builders: [(&str, fn() -> ScheduledStatorCore); 2] =
        [("rung 65", lagged_machine), ("rung 68", three_loop_machine)];
    for (name, build) in builders {
        let msg = message_of(move || { build().fuel.inner.with_ref(Some("inc")); });
        assert!(msg.contains("_with_ref is RUNG 69's"),
                "{name}'s slot must PANIC, not answer `None`. Got: {msg:?}");
    }
}

/// The added cell, on the rung it arrives at: **set, and hand back what was displaced.**
///
/// The carrier starts at Python's class attribute (`_ref = None`) and the cell is the only thing
/// that writes it.
#[test]
fn with_ref_sets_the_carrier_and_returns_the_displaced_value() {
    let m = split_machine(&split_arm());
    let c = &m.fuel.inner;
    assert_eq!(c.ref_.get(), None, "Python's class attribute `_ref = None`");
    assert_eq!(c.with_ref(Some("inc")), None, "the first set displaces `None`");
    assert_eq!(c.ref_.get(), Some("inc"));
    assert_eq!(c.with_ref(Some("phi")), Some("inc"), "and hands back what it displaced");
    assert_eq!(c.ref_.get(), Some("phi"));
    assert_eq!(c.with_ref(None), Some("phi"), "`None` is a real assignment, not a no-op");
    assert_eq!(c.ref_.get(), None);
}

/// **THE GUARD RESTORES THE PREVIOUS VALUE, AND THE NEST THAT SEPARATES THE TWO POLICIES IS
/// MANUFACTURED HERE.**
///
/// Python's `_with_ref` is `prev, self._ref = self._ref, ref` … `finally: self._ref = prev`.
/// Measured over the rung-69 suite, the displaced value is `None` at every one of the 29
/// value-sets — so a port that restored to `None` instead would be bit-for-bit identical on every
/// shipped path and **no value key in the whole slice could tell them apart.** That combination is
/// what a manufactured-bug gate is for: [`InitialBleed`]'s nest in `slice_y_dispatch.rs`, one
/// carrier over.
///
/// [`InitialBleed`]: turbojet::two_spool_transient::InitialBleed
#[test]
fn the_ref_guard_restores_the_previous_value_and_a_nest_proves_it() {
    let m = split_machine(&split_arm());
    let c = &m.fuel.inner;
    {
        let outer = RefScope::set(c, Some("phi"));
        assert_eq!(outer.displaced(), None);
        assert_eq!(c.ref_.get(), Some("phi"));
        {
            // THE MANUFACTURED NEST. No shipped path reaches it — that is the point.
            let inner = RefScope::set(c, Some("inc"));
            assert_eq!(inner.displaced(), Some("phi"));
            assert_eq!(c.ref_.get(), Some("inc"));
        }
        assert_eq!(c.ref_.get(), Some("phi"),
                   "a restore-to-`None` guard would leave `None` here and nothing else in the \
                    slice would notice");
    }
    assert_eq!(c.ref_.get(), None, "and the outer scope restores Python's class attribute");
}

/// **THE RESTORE IS `Drop`, SO IT SURVIVES AN UNWIND A STRAIGHT-LINE RESTORE WOULD SKIP.**
///
/// Python gets this from `finally`. A naive port writes the restore after the call and loses it on
/// any raise — and every reader of `_ref` is a diagnostic that can abort, so the leak would show
/// up as a reader reporting a plant that was never marched (rung 62's own stated reason for the
/// `finally`).
#[test]
fn the_ref_guard_restores_on_an_unwind() {
    let m = split_machine(&split_arm());
    let c = &m.fuel.inner;
    let out = catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _g = RefScope::set(c, Some("inc"));
        panic!("a reader aborting mid-scope");
    }));
    assert!(out.is_err());
    assert_eq!(c.ref_.get(), None, "the `finally` is the destructor");
}

// =============================================================================================
// GATE 3 — THE FOUR `__init__` GUARDS
// =============================================================================================

/// **ALL FOUR FIRE, AND ALL FOUR ARE REACHABLE.**
///
/// Slice U's pre-flight found three shipped asserts no input can reach by sweeping the ARMING
/// COMBINATIONS, so a 96-point grid over `stator_inc` × `stator_lim` × `vsv_lp` × `vsv_sched_lp` ×
/// `bleed_lim` × `lp_disabled` was swept before the builder was written: A fires 4 times, B 20, C
/// 10, D once, and **13 of 96 points build**. This gate is that grid's four witnesses.
///
/// The MESSAGE is read, not the fact of a panic: guard A and guard B are both satisfied by *"the
/// build refused something"*, and a builder that refused everything would pass a fact-of-panic
/// gate four times over.
#[test]
fn the_four_rung69_guards_fire_and_each_names_its_own_refusal() {
    // A — ONE stator, ONE reference.
    let a = message_of(|| {
        let _ = split_machine(&LeverArm {
            bleed_lim: Some(valve()),
            stator_inc: Some(inc()),
            stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU))),
            ..Default::default()
        });
    });
    assert!(a.contains("rung-69 is ONE stator with ONE reference"), "guard A: {a:?}");

    // B — a CONSTANT setting (53), a SCHEDULE (57) or a FLOOR (68/69), exactly one.
    let b = message_of(|| {
        let _ = split_machine(&LeverArm {
            bleed_lim: Some(valve()),
            stator_inc: Some(inc()),
            stator: StatorArm { vsv_lp: 0.05, ..Default::default() },
            ..Default::default()
        });
    });
    assert!(b.contains("rung-69: the LP stators get a CONSTANT setting"), "guard B: {b:?}");

    // C — an incidence floor on a DISABLED LP spool.
    let c = message_of(|| {
        let _ = build_reference_split_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
            &LeverArm {
                stator_inc: Some(inc()),
                stator: StatorArm { lp_disabled: true, ..Default::default() },
                ..Default::default()
            });
    });
    assert!(c.contains("rung-69's incidence floor watches the LP"), "guard C: {c:?}");

    // D — ONE PHYSICAL WALL. An incidence floor built off a DIFFERENT margin than the valve's is
    // the one arming that is well-formed in every other respect and still wrong.
    let d = message_of(|| {
        let _ = split_machine(&LeverArm {
            bleed_lim: Some(valve()),
            stator_inc: Some(StatorIncidenceLimiter::from_margin(
                &lp_map(), V_MAX, SM * 1.10, Some(TAU))),
            ..Default::default()
        });
    });
    assert!(d.contains("rung-69 needs ONE PHYSICAL WALL"), "guard D: {d:?}");

    // AND THE WITNESS THAT MAKES THE FOUR ABOVE MEASUREMENTS: the matched arming BUILDS.
    let m = split_machine(&split_arm());
    assert!(m.fuel.inner.stator.inc.is_some(), "the incidence floor reached the arming");
    assert!(m.fuel.inner.stator.lim.is_none(), "and rung 68's did not");
}

/// **GUARD D IS AN EQUALITY OF WALLS, NOT OF FLOATS — and the two numbers are in different
/// units.**
///
/// `m_lim` is an incidence margin and `phi_lim` a flow coefficient; they are never equal and
/// comparing them would be the set-point offset the guard exists to refuse. What must agree is
/// `m_lim == T_c - 1/phi_lim`, i.e. the two floors are the SAME WALL read at the design setting —
/// which is what [`StatorIncidenceLimiter::phi_lim_at`] inverts.
#[test]
fn the_matched_floors_are_one_wall_read_in_two_coordinates() {
    let cmap = lp_map();
    let v = valve();
    let i = inc();
    assert_ne!(i.m_lim, v.phi_lim, "different units -- equality here would be the wrong test");
    assert_eq!(i.m_lim, cmap.tan_beta1_crit() - 1.0 / v.phi_lim);
    assert_eq!(i.phi_lim_at(&cmap), 1.0 / (cmap.tan_beta1_crit() - i.m_lim));
    // The round trip, and it is not free: `from_phi` and `phi_lim_at` are each other's inverse in
    // ALGEBRA, and this asserts they are within an ulp in ARITHMETIC too.
    assert!((i.phi_lim_at(&cmap) / v.phi_lim - 1.0).abs() <= 4.0 * f64::EPSILON);
    // And `margin` at the design setting IS the floor's own coordinate.
    assert_eq!(
        StatorIncidenceLimiter::margin(cmap.tan_beta1_crit(), v.phi_lim, 0.0), i.m_lim,
        "M_i at v=0 on the phi floor IS m_lim -- that is what 'matched at the design setting' \
         means, and it is why the two walls diverge only as the lever moves");
    // Rung 68's floor built from the SAME `from_margin(cmap, ., sm)` is the valve's, exactly —
    // which is what makes "one set point" a statement rather than a coincidence.
    assert_eq!(StatorLimiter::from_margin(&cmap, V_MAX, SM, Some(TAU)).phi_lim, v.phi_lim);
}

// =============================================================================================
// GATE 4 — THE DEVICE'S OWN REFUSALS, AND THE ONE RUNG 68 HAS THAT IT DOES NOT
// =============================================================================================

/// Rung 69's `__post_init__` — **two asserts where rung 68's limiter has three.**
///
/// The third, `assert self.phi_lim > 0.0`, has no counterpart — and the witness for that is
/// MEASURED rather than argued, because **the first draft of this gate argued it and the argument
/// was false.** It read *"`m_lim` is a signed margin, so the shipped floor's is negative"*. The
/// shipped floor's `m_lim` is **positive**: `T_c = 1/phi_surge` exactly (rung 53, zero new
/// constants) and the floor sits at or above the surge line, so `m_lim = 1/phi_surge - 1/phi_lim`
/// is non-negative for every floor this rung builds. Slice AB's own pre-flight lesson, hit inside
/// slice AB — a mechanism typed beside a measurement is not a measurement.
///
/// What actually makes the absence a decision is the BOUNDARY: at `sm = 0` the two reciprocals
/// cancel and `m_lim` is **exactly zero** — the case
/// [`from_margin`](turbojet::reference_split::StatorIncidenceLimiter::from_margin)'s own assert
/// explicitly admits (*"sits AT or ABOVE the surge line"*), and the one a copied-over `> 0` would
/// refuse.
#[test]
fn the_incidence_limiter_refuses_no_authority_and_no_clock_but_not_a_zero_margin() {
    let e = message_of(|| { let _ = StatorIncidenceLimiter::new(-0.2, 0.0, Some(TAU)); });
    assert!(e.contains("rung-69 needs stators with AUTHORITY"), "{e:?}");
    let e = message_of(|| { let _ = StatorIncidenceLimiter::new(-0.2, 1.0, Some(TAU)); });
    assert!(e.contains("rung-69 needs stators with AUTHORITY"), "{e:?}");
    let e = message_of(|| { let _ = StatorIncidenceLimiter::new(-0.2, V_MAX, Some(0.0)); });
    assert!(e.contains("rung-69 tau is a time constant"), "{e:?}");

    // THE ABSENT THIRD ASSERT, as a witness rather than as a comment.
    assert!(inc().m_lim > 0.0,
            "the shipped floor's margin is POSITIVE -- measured, after the first draft of this \
             gate asserted the opposite from an argument");
    assert_eq!(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, 0.0, None).m_lim, 0.0,
               "AT the surge line the two reciprocals cancel EXACTLY, and `from_margin`'s own \
                assert admits `sm = 0`. A copied-over `m_lim > 0` would refuse it -- which is \
                what makes rung 69's two-assert `__post_init__` a decision and not an omission");
    // And a directly-built negative floor is accepted, which is the same statement with the
    // constructor out of the way.
    let _ = StatorIncidenceLimiter::new(-1e9, V_MAX, None);

    // `from_margin` carries rung 68's two, verbatim.
    let e = message_of(|| {
        let _ = StatorIncidenceLimiter::from_margin(&ComponentMap::flat(), V_MAX, SM, Some(TAU));
    });
    assert!(e.contains("rung-69 from_margin needs a surge line"), "{e:?}");
    let e = message_of(|| {
        let _ = StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, -0.1, Some(TAU));
    });
    assert!(e.contains("sits AT or ABOVE the surge line"), "{e:?}");
}

// =============================================================================================
// GATE 5 — P1 AND P5: THE TWO STRUCTS THAT DO NOT GROW
// =============================================================================================

/// **P5 — [`StatorLegArm`] does not grow, and the BAND'S SIGN is not in it.**
///
/// Slice AA built this type FOR this slice: `_stator_leg`'s callers touch exactly `.tau` and
/// `.v_max`, so the cell's return is narrowed to those two rather than being an enum over the two
/// limiter types. The two limiters therefore produce the SAME arm for the same hardware — which is
/// the whole reason `_clamp_v` and `_check_v0` are cells: the band is `[-v_max, 0]` at rung 68 and
/// `[0, +v_max]` here, and **nothing in the value distinguishes them**.
///
/// The literal is exhaustive on purpose. A third field added by a later slice fails to compile
/// here, which is the prediction stated as a compiler error rather than as a sentence.
#[test]
fn p5_the_stator_leg_arm_does_not_grow_and_carries_no_sign() {
    let from_inc: StatorLegArm = inc().into();
    let from_phi: StatorLegArm =
        StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU)).into();
    assert_eq!(from_inc, from_phi,
               "the two references' arms are IDENTICAL -- the band's sign lives in the CELLS");
    let exhaustive = StatorLegArm { v_max: V_MAX, tau: Some(TAU) };
    assert_eq!(exhaustive, from_inc);
}

/// **P1 — [`MarchScope`] does not grow at this slice.**
///
/// `_ref` is the phase's first CONFIG-kind dynamically-scoped field: 58 sets, every one from
/// `_with_ref`, **every one outside every march**. So it takes phase 5's carrier precedent and the
/// march's own scope struct is untouched — where slice AA grew it by two.
///
/// Exhaustive literal, no `..MarchScope::DEFAULT`: a sixth field fails to compile here.
#[test]
fn p1_the_march_scope_does_not_grow() {
    let s = MarchScope { b0: None, lag: None, tau_gov: None, v0: None, ic_order: None };
    assert_eq!(s.b0, MarchScope::DEFAULT.b0);
    assert_eq!(s.v0, MarchScope::DEFAULT.v0);
    assert_eq!(s.ic_order, MarchScope::DEFAULT.ic_order);
}

/// **WHAT DID GROW, MEASURED — because slice AA's step-1 lesson is that a growth prediction gets
/// asked of the struct that hurt last time and not of the ones the step actually grows.**
///
/// Four structs gained one field each: [`LeverArm`] (`stator_inc`, its NINTH `at_lever` keyword),
/// [`StatorArming`](turbojet::stator_transient::StatorArming) (`inc`),
/// [`TwoSpoolTransientCore`](turbojet::two_spool_transient::TwoSpoolTransientCore) (`ref_`), and
/// [`TripleHooks`] (`with_ref`). The compiler named **nine** literals — **8 in `src/`** (
/// `three_loop.rs` ×2, `stator_transient.rs` ×2, `bleed_transient.rs` ×2, `limited_bleed.rs`,
/// `two_spool_transient.rs`) **and 1 in `tests/`** (`slice_v_dispatch.rs`).
///
/// **THAT COUNT WAS TYPED AS "SIX, AND NONE IN `tests/`" BEFORE IT WAS ADDED UP, AND BOTH HALVES
/// WERE WRONG.** The `tests/` half was worse than a miscount: it was read off a backgrounded
/// `cargo build --tests` that reported exit 0 **with empty output**, and the real run does not
/// compile until `slice_v_dispatch.rs` moves. [[windows-tooling-file-hazards]] — *a status read
/// off the runner* — and [[rust-port-guessed-census-bars]], in one line.
///
/// Each new field's `None` is asserted below, because a field added with the wrong default is
/// invisible to a compile.
#[test]
fn the_four_structs_that_grew_default_to_unarmed() {
    assert!(LeverArm::default().stator_inc.is_none());
    let m = lagged_machine();
    assert!(m.fuel.inner.stator.inc.is_none(), "rungs 57-68 build an arming with no `inc`");
    assert_eq!(m.fuel.inner.ref_.get(), None, "Python's class attribute `_ref = None`");
    // And the table's new slot is a real, distinct pointer on rung 69's table — read through the
    // behaviour, never through `ptr::eq` on a `const` (slice Y step 3, twice).
    let s = split_machine(&split_arm());
    assert_eq!((R69_TRIPLE.with_ref)(&s.fuel.inner, Some("inc")), None);
    assert_eq!(s.fuel.inner.ref_.get(), Some("inc"));
}
