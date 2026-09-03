//! SLICE AC step 1 — **ZERO ADDED CELLS, FIVE OPENED SWAPS, AND A TABLE COUNT THAT WAS WRONG IN
//! THE PRE-FLIGHT.**
//!
//! § 5.19 (x)'s rule for phase 7 is *step 1 of every slice is the cell addition*, so a slice that
//! forgets a cell fails at its own first gate rather than at a value key nine rungs downstream.
//! **This slice adds no cell at all** (§ 5.27 (i)), so that rule buys nothing here and the whole
//! risk sits on the other side: a forgotten SWAP is not a missing function, it is the PARENT's,
//! which compiles, runs, and is caught by nothing the ladder does automatically.
//!
//! # THE THREE COUNTS, RECONCILED IN ONE PLACE
//!
//! An unreconciled pair of counts in two files is this phase's most-repeated defect, so all three
//! are written out here and each is gated below rather than asserted in prose.
//!
//! * **0 CELLS ADDED.** The column claimed one, `split_gains`, for the seventh consecutive slice —
//!   and the predicate behind it is *new here AND overridden above*, purely by NAME, never once
//!   asking whether the two bodies are INTERCHANGEABLE. Rung 80's same-named reader drops four
//!   parameters and adds five, and rung 70's own inherited caller raises `TypeError` on a rung-80
//!   machine. So `TripleHooks` stays **ten** fields wide and `split_gains` ports as a plain method.
//! * **5 SWAPS ⇒ 5 DISTINCT FUNCTION POINTERS.** `at_lever` and `integrate_fuel` at BOTH rungs,
//!   `triple_laws` at rung 70 alone.
//! * **10 TABLE CONSTS, AND THE PRE-FLIGHT'S STEP-1 LINE SAID "nine".** Five per rung. § 5.27
//!   (iii) enumerates ten names one paragraph after the step list says nine, and nobody diffed the
//!   two — [[rust-port-phase7-preflight]]'s own lesson, one section over. **Counted, not
//!   inherited**: [`the_slice_writes_ten_table_consts_and_the_preflight_said_nine`] reads the two
//!   source files, and [`the_two_builders_install_their_own_rungs_tables`] names all ten in code,
//!   so a deleted one is a compile error and a miscount cannot drift back into prose.
//!
//! # WHY THERE IS NO "EVERY SLOT PANICS WITH ITS OWN NAME" GATE
//!
//! Slice AB wrote one and **had to dismantle it at step 2**, because its whole content was *"not
//! yet ported"*: it read nine placeholder panic messages and step 2 deleted every one of them. A
//! gate with a known expiry is a tautology in waiting, and this file has a rule against those.
//!
//! The durable form of the same question is **pointer inequality between two shipped `const`s** —
//! *is rung 70's slot a different body from rung 69's?* — which is still a question after the
//! bodies land, and is the question a `fn` pointer in a `const` table actually poses.
//! `std::ptr::fn_addr_eq` on the CELLS and never `ptr::eq` on the table: a `const` is inlined at
//! every use, so `&R70` is a fresh promotion and comparing those tests the optimiser
//! ([[rust-port-slice-y-step3]], and [[rust-port-slice-aa-step1]] where it was written a second
//! time).
//!
//! **AND EVERY INEQUALITY GATE BELOW CARRIES ITS OWN EQUALITY CONTROL**, on the same table, so a
//! gate that would pass for a broken instrument fails visibly instead.
//!
//! # WHAT THIS FILE DELIBERATELY DOES NOT GATE
//!
//! **`at_lever`'s DISPATCH behaviour.** § 5.27 (v) measured both `at_lever` swaps observable only
//! because the parent's `integrate_fuel` then REFUSES the arming. In Rust `at_lever` returns a
//! table pointer and nothing refuses anything until those asserts are ported, so a gate written
//! now would report UNOBSERVABLE for a reason about ORDERING rather than about the cell. Booked to
//! step 7 by the pre-flight; recorded here so its absence is not read as an oversight.
//!
//! **NOTHING HERE READS A GOLDEN.** Slice V step 5's rule: every assertion is a panic, a same-run
//! difference, or a compile-time property. Steps 2–3's bodies cannot make any of them pass or fail
//! by accident.

use std::panic::catch_unwind;
use std::ptr::fn_addr_eq;

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::cross_split::{
    build_cross_split_cascade, GovScope, R70, R70_FUEL, R70_STATOR, R70_TRIPLE, R70_TWO,
};
use turbojet::engine::FlightCondition;
use turbojet::full_split::{
    build_full_split_cascade, R71, R71_FUEL, R71_STATOR, R71_TRIPLE, R71_TWO,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{
    build_reference_split_cascade, StatorIncidenceLimiter, R69, R69_FUEL, R69_TRIPLE,
};
use turbojet::stator_transient::{
    MarchScope, ScheduledStatorCore, ScheduledStatorTransient, StatorArm,
};
use turbojet::three_loop::{StatorLimiter, TripleHooks, R68_TRIPLE};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// Slice AB's grid verbatim — this slice adds no constant of its own and this file must not be the
// place one appears.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const B: f64 = 0.10;
const TAU: f64 = 0.05;
/// Rungs 57/58's swept setting `V = 0.20`, INHERITED.
const V_MAX: f64 = 0.20;
/// `PHI / FLOOR - 1.0` — the suite's own spelling.
const SM: f64 = PHI / FLOOR - 1.0;
/// Rung 67's IMPOSED redline, taken verbatim (rung 70's own first concession).
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

/// Rung 68's `phi` stator — rung 70's third loop.
fn phi_stator() -> StatorLimiter { StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU)) }

/// Rung 69's INCIDENCE stator on the SAME physical wall — rung 71's third loop.
fn inc_stator() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU))
}

/// RUNG 70's arming: the governor is armed by a MARCH argument, so the machine is rung 68's —
/// valve under a floor plus a `phi` stator.
fn cross_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_lim: Some(phi_stator()), ..Default::default() }
}

/// RUNG 71's arming: the same machine with the stator's REFERENCE moved to incidence.
fn full_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_inc: Some(inc_stator()), ..Default::default() }
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

fn cross_machine(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_cross_split_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

fn full_machine(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_full_split_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// The panic message a closure produces, or `""` if it did not panic. Slice AB's helper verbatim,
/// and for its reason: `assert!(panics(…))` is satisfied by an unrelated bug as readily as by the
/// refusal it names.
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
// GATE 1 — THE TABLE COUNT, MEASURED
// =============================================================================================

/// **TEN, AND THE PRE-FLIGHT'S STEP LIST SAID NINE.**
///
/// § 5.27 (xi) step 1 reads *"the nine tables of § (iii)"*; § (iii) one page earlier enumerates
/// `R70`, `R70_FUEL`, `R70_TRIPLE`, `R71`, `R71_FUEL` **and** `R70_TWO`, `R70_STATOR`, `R71_TWO`,
/// `R71_STATOR`, `R71_TRIPLE` — five plus five. The two numbers were written a page apart and
/// never diffed, which is [[rust-port-phase7-preflight]]'s own recorded lesson (*the plan stated
/// the same set twice and nobody diffed the two*), one section over.
///
/// The count is taken from the SOURCE rather than from a list typed here, because a list typed
/// here is the thing that was wrong. `include_str!` is the phase's established instrument for
/// this (§ 6's replacement for `test_rung73.py:488`).
///
/// **AND THE CONTROL IS THE POINT.** A `.matches().count()` over a pattern that happens to appear
/// nowhere returns 0 and passes an `assert!(n >= …)`; over a pattern that appears in prose it
/// over-counts. So the prefix is anchored at a line start (`\npub const R7`), the two files are
/// counted separately, and a deliberately-absent pattern is counted too.
#[test]
fn the_slice_writes_ten_table_consts_and_the_preflight_said_nine() {
    let cross = include_str!("../src/cross_split.rs");
    let full = include_str!("../src/full_split.rs");

    let n70 = cross.matches("\npub const R70").count();
    let n71 = full.matches("\npub const R71").count();
    assert_eq!(n70, 5, "rung 70's tables: R70, R70_TWO, R70_FUEL, R70_STATOR, R70_TRIPLE");
    assert_eq!(n71, 5, "rung 71's tables: R71, R71_TWO, R71_FUEL, R71_STATOR, R71_TRIPLE");
    assert_eq!(n70 + n71, 10, "TEN table consts -- the pre-flight's step-1 line said nine");

    // Neither file writes the OTHER rung's tables, which is what makes the two counts separable.
    assert_eq!(cross.matches("\npub const R71").count(), 0);
    assert_eq!(full.matches("\npub const R70").count(), 0);

    // THE INSTRUMENT'S OWN CONTROL: a prefix that is deliberately absent must read zero, or the
    // three counts above are not measurements. `R72` is slice AD's, unwritten.
    assert_eq!(cross.matches("\npub const R72").count(), 0, "the counter can miss");
    assert_eq!(full.matches("\npub const R72").count(), 0, "the counter can miss");

    // And ZERO cells are added at this slice, so no TABLE TYPE is declared in either file.
    //
    // **THE SPELLING IS REPAIRED AT STEP 2, AND THE SENTENCE IT REPLACES IS WHY.** Step 1 wrote
    // this as `pub struct` == 1 ("the only new struct in the slice is `GovScope`"), which was
    // true while the two files held one struct between them and became false the instant step 2
    // landed the seven readers' RETURN types -- eighteen plain data structs, not one of them a
    // table. The doc line already named the property (`pub struct .*Hooks`); the code was a
    // PROXY for it that happened to coincide. That is this slice's own step-1 lesson in its
    // other direction: ask whether the step that follows deletes -- or here ADDS -- the thing
    // your gate reads. P1 is about `TripleHooks` not growing, and a count of data structs cannot
    // see that.
    let table_types = |s: &str| {
        s.lines().filter(|l| l.starts_with("pub struct ") && l.contains("Hooks")).count()
    };
    assert_eq!(table_types(cross), 0, "rung 70 declares no table type");
    assert_eq!(table_types(full), 0, "rung 71 declares no table type");
    // THE POSITIVE CONTROL, because a filter that matches nothing passes an `== 0` for free: the
    // same detector must SEE the table type this slice asserts it does not re-declare.
    //
    // **AND ITS REACH IS STATED RATHER THAN LEFT TO BE ASSUMED**: the filter anchors at COLUMN 0,
    // so a `Hooks` type declared inside a `mod` or an `impl` block reads zero. Every table type in
    // this crate is a top-level declaration, which is what makes the anchor safe here -- and is
    // exactly the assumption the next slice should re-check rather than inherit.
    assert_eq!(table_types(include_str!("../src/three_loop.rs")), 1,
               "the detector must find `TripleHooks` where it IS declared, or the two zeros \
                above are not measurements");
    // `GovScope` is still the slice's one non-data type, named so a rename fails HERE.
    assert!(cross.contains("\npub struct GovScope"), "GovScope is still declared here");
}

// =============================================================================================
// GATE 2 — THE BUILDERS INSTALL THEIR OWN RUNG'S TABLES
// =============================================================================================

/// **ALL TEN CONSTS NAMED IN CODE, so a deleted one is a compile error rather than a prose
/// mismatch — and the swapped cells are read where the LADDER installed them.**
///
/// This is the gate that catches a builder wired to the parent's tables, which pointer inequality
/// between two `const`s cannot see: `R70.at_lever != R69.at_lever` stays true even if nothing ever
/// installs `R70`.
#[test]
fn the_two_builders_install_their_own_rungs_tables() {
    let c = cross_machine(&cross_arm());
    assert!(fn_addr_eq(c.fuel.inner.lever_hooks.at_lever, R70.at_lever), "R70");
    assert!(fn_addr_eq(c.fuel.hooks.integrate_fuel, R70_FUEL.integrate_fuel), "R70_FUEL");
    assert!(fn_addr_eq(c.fuel.inner.triple_hooks.triple_laws, R70_TRIPLE.triple_laws),
            "R70_TRIPLE");
    assert!(fn_addr_eq(c.fuel.inner.hooks.try_close, R70_TWO.try_close), "R70_TWO");
    assert!(fn_addr_eq(c.fuel.inner.stator_hooks.stator_march, R70_STATOR.stator_march),
            "R70_STATOR");

    let f = full_machine(&full_arm());
    assert!(fn_addr_eq(f.fuel.inner.lever_hooks.at_lever, R71.at_lever), "R71");
    assert!(fn_addr_eq(f.fuel.hooks.integrate_fuel, R71_FUEL.integrate_fuel), "R71_FUEL");
    assert!(fn_addr_eq(f.fuel.inner.triple_hooks.triple_laws, R71_TRIPLE.triple_laws),
            "R71_TRIPLE");
    assert!(fn_addr_eq(f.fuel.inner.hooks.try_close, R71_TWO.try_close), "R71_TWO");
    assert!(fn_addr_eq(f.fuel.inner.stator_hooks.stator_march, R71_STATOR.stator_march),
            "R71_STATOR");

    // THE CONTROL: the two machines carry DIFFERENT lever tables, so the five assertions above are
    // not all satisfied by one shared table.
    assert!(!fn_addr_eq(c.fuel.inner.lever_hooks.at_lever, f.fuel.inner.lever_hooks.at_lever));
    // And the arming reached the plant it was meant to.
    assert!(c.fuel.inner.stator.lim.is_some() && c.fuel.inner.stator.inc.is_none(),
            "rung 70's stator watches `phi`");
    assert!(f.fuel.inner.stator.inc.is_some() && f.fuel.inner.stator.lim.is_none(),
            "rung 71's stator watches INCIDENCE");
}

// =============================================================================================
// GATE 3 — THE FIVE SWAPS ARE FIVE DISTINCT POINTERS
// =============================================================================================

/// **THE STEP-1 GATE, IN THE FORM THAT SURVIVES STEP 2.**
///
/// Five swaps, five inequalities, each against the parent the SOURCE overrides — rung 69 for the
/// three at rung 70, **rung 70** for the two at rung 71. That last part is not cosmetic: a rung-71
/// slot that reached back past rung 70 to rung 69's body is a real defect, and an inequality taken
/// against rung 69 would call it clean.
///
/// Every inequality is paired with an equality on the SAME table, so a comparison that could not
/// distinguish two pointers fails here rather than passing everything.
#[test]
fn the_five_swapped_cells_are_distinct_function_pointers_from_their_parents() {
    // --- rung 70's three, against rung 69 ---
    assert!(!fn_addr_eq(R70.at_lever, R69.at_lever), "swap 1: r70 at_lever");
    assert!(!fn_addr_eq(R70_FUEL.integrate_fuel, R69_FUEL.integrate_fuel),
            "swap 2: r70 integrate_fuel");
    assert!(!fn_addr_eq(R70_TRIPLE.triple_laws, R69_TRIPLE.triple_laws),
            "swap 3: r70 triple_laws");

    // --- rung 71's two, against RUNG 70 ---
    assert!(!fn_addr_eq(R71.at_lever, R70.at_lever), "swap 4: r71 at_lever");
    assert!(!fn_addr_eq(R71_FUEL.integrate_fuel, R70_FUEL.integrate_fuel),
            "swap 5: r71 integrate_fuel");

    // --- THE CONTROLS: the cells the slice does NOT swap, on the same three tables ---
    assert!(fn_addr_eq(R70.isolating, R69.isolating), "control: r70 inherits `isolating`");
    assert!(fn_addr_eq(R70_FUEL.try_close_fuel, R69_FUEL.try_close_fuel),
            "control: r70 inherits `try_close_fuel`");
    assert!(fn_addr_eq(R71.isolating, R70.isolating), "control: r71 inherits `isolating`");
    assert!(fn_addr_eq(R71_FUEL.try_close_fuel, R70_FUEL.try_close_fuel),
            "control: r71 inherits `try_close_fuel`");
    // Rung 71 overrides NONE of the ten triple cells -- measured per rung, not assumed.
    assert!(fn_addr_eq(R71_TRIPLE.triple_laws, R70_TRIPLE.triple_laws),
            "rung 71 does not override `triple_laws`");
    assert!(fn_addr_eq(R71_TRIPLE.solve_v, R70_TRIPLE.solve_v));
    assert!(fn_addr_eq(R71_TRIPLE.with_ref, R70_TRIPLE.with_ref));
}

/// **THE `triple_laws` CHAIN IS THREE LINKS AND ONLY THE MIDDLE ONE BREAKS.**
///
/// `R69_TRIPLE` spells `triple_laws` out as `R68_TRIPLE.triple_laws` with a comment naming rung 70
/// as the class that overrides it — the one cell of rung 68's nine that rung 69 leaves alone. This
/// asserts the whole chain rather than the one link, because *"rung 70 differs from rung 69"* is
/// also true if rung 69's slot had silently drifted off rung 68's.
#[test]
fn the_triple_laws_chain_is_three_links_and_rung70_breaks_the_last() {
    assert!(fn_addr_eq(R69_TRIPLE.triple_laws, R68_TRIPLE.triple_laws),
            "link 1: rung 69 INHERITS rung 68's body -- the one of the nine it does not swap");
    assert!(!fn_addr_eq(R70_TRIPLE.triple_laws, R68_TRIPLE.triple_laws),
            "link 2: rung 70 is the only class in the ladder that overrides it");
    assert!(fn_addr_eq(R71_TRIPLE.triple_laws, R70_TRIPLE.triple_laws),
            "link 3: and rung 71 inherits rung 70's");
}

// =============================================================================================
// GATE 4 — P1: THE TABLE DOES NOT GROW
// =============================================================================================

/// **`TripleHooks` IS STILL EXACTLY TEN CELLS WIDE — P1, and the compiler is the assertion.**
///
/// Written as an exhaustive literal with no `..` spread, so a field added by any future slice is
/// `E0063` here. Slice AB's own tripwire lives in `slice_ab_cells.rs`; this is the same instrument
/// at this slice's table, and it is written because AB's tripwire named **slice AC** as the author
/// of the fourteenth field — a guess taken from the very column § 5.27 (i) repairs, corrected to
/// name no slice at all.
///
/// **AND THE CELL COUNT IS THE PREDICTION, NOT THE SWAP COUNT.** Five swaps, zero additions: a
/// swap moves a pointer and cannot change the width, which is exactly why the width gate says
/// nothing about whether the swaps landed and Gate 3 exists.
///
/// **THE TRIPWIRE WAS FIRED RATHER THAN ASSUMED, AND THE FIRST ATTEMPT MEASURED THE WRONG THING.**
/// Adding a fourteenth field to `TripleHooks` and building does NOT reach this file: `src/` holds
/// five exhaustive `TripleHooks` literals of its own (`NO_TRIPLE`, `R68_TRIPLE`, `R69_TRIPLE`,
/// `R70_TRIPLE`, `R71_TRIPLE`), so the LIB is `E0063` first and `cargo build --tests` never
/// compiles a test target. **The tripwire is SHADOWED, and that is a property of the ordering, not
/// a defect** — the scenario it exists for is a slice that adds a cell and repairs every `src/`
/// literal, because the lib must compile before anything else can. Simulated exactly that way, the
/// lib builds and this literal is `E0063` at line 339. Recorded because *"a fourteenth field is
/// `E0063` at the file whose job is the cell census"* is true only on the second half of that
/// sequence, and slice AB's version of this sentence was already wrong once about its addressee.
/// **UPDATED AT SLICE AD, AND THE UPDATE IS THE TRIPWIRE WORKING.** Rung 72 adds `reference`,
/// `rk4_floor_shared` and `shared_rig`, and this literal stopped compiling — which is the entire
/// point of spelling the fields out. Slice AD's own P1 predicted five `E0063` sites (the five
/// `TripleHooks` consts in `src`) and the landed edit needed **seven**: `cargo check` stops at the
/// lib, so the probe never reached this file or its sibling. The mechanism P1 was testing —
/// exhaustive literals go loud, `..` spreads and whole-const aliases stay silent — held; only its
/// count was short, and short for a reason worth more than the number.
/// **UPDATED AT SLICE AF, THE FIFTH FIRING, AND IT FOUND TWO TRIPWIRES THE PHASE HAD NOT NAMED.**
/// Rung 74 adds `cap_fuel`, `sensed_cap`, `windup_tau` and `with_coord` — 14 → 18, the widest
/// single arrival — and the count was measured AD's way (apply, fix the lib, count what is still
/// red) rather than predicted. It came back **FOUR test-target sites, not two**: this literal, its
/// sibling in the other cells file, and **two exhaustive DESTRUCTURINGS in `slice_ae_cells.rs` and
/// `slice_ae_dispatch.rs`** that go `E0027` on the same event. A destructuring is a second,
/// differently-typed instrument for this job and nothing in the crate called it a tripwire.
#[test]
fn the_triple_table_is_still_exactly_eighteen_cells_wide() {
    let ten = TripleHooks {
        stator_leg: R70_TRIPLE.stator_leg,
        lagged_stator: R70_TRIPLE.lagged_stator,
        clamp_v: R70_TRIPLE.clamp_v,
        check_v0: R70_TRIPLE.check_v0,
        rk4_floor: R70_TRIPLE.rk4_floor,
        solve_v: R70_TRIPLE.solve_v,
        manifold_v: R70_TRIPLE.manifold_v,
        triple_laws: R70_TRIPLE.triple_laws,
        triple_rig: R70_TRIPLE.triple_rig,
        with_ref: R70_TRIPLE.with_ref,
        // THE THREE SLICE AD ADDS — this file failed to compile until they
        // were written, which is what a width tripwire is for.
        reference: R70_TRIPLE.reference,
        rk4_floor_shared: R70_TRIPLE.rk4_floor_shared,
        shared_rig: R70_TRIPLE.shared_rig,
        // AND THE FOURTEENTH, ADDED BY SLICE AE STEP 2 — `quad_gains_at`. This literal stopped
        // compiling when it landed, which is the fourth time this tripwire has done its job.
        quad_gains_at: R70_TRIPLE.quad_gains_at,
        // AND SLICE AF's FOUR — the FIFTH firing, and the widest single arrival the table has
        // had. `_cap_fuel`, `_sensed_cap`, `_windup_tau` and `_with_coord` all land at rung 74.
        cap_fuel: R70_TRIPLE.cap_fuel,
        sensed_cap: R70_TRIPLE.sensed_cap,
        windup_tau: R70_TRIPLE.windup_tau,
        with_coord: R70_TRIPLE.with_coord,
    };
    // THESE TWO ASSERTS ARE NOT THE PIN AND MUST NOT BE READ AS ONE. Every field of `ten` was
    // copied FROM the table it is compared against, so they are self-comparisons —
    // [[rust-port-ported-test-vacuity]]'s shape, harmless here because the literal above is the
    // assertion and the compiler makes it. They exist to keep `ten` alive against dead-code
    // elimination and to name the second table, so BOTH are covered by one literal's width.
    assert!(fn_addr_eq(ten.triple_laws, R70_TRIPLE.triple_laws));
    assert!(fn_addr_eq(ten.rk4_floor, R71_TRIPLE.rk4_floor));
}

/// **P4 — `MarchScope` DOES NOT GROW**, and `_gov_max` is why the question is asked.
///
/// It is the slice's one arriving dynamically-scoped field, and § 5.27 (vii) measured it
/// CONFIG-kind on the WHOLE 57-test suite rather than on a reader sample: **256 sets, 98 through
/// `_with_gov` and 158 through the two rigs' bare assignments, 0 inside any march, 0 overwrites**,
/// per-instance max nesting depth 1. A field set outside every march does not ride on the march's
/// scope struct.
///
/// Exhaustive literal (no `..` spread), so a field added to `MarchScope` is a compile error here
/// — **and it is SHADOWED exactly as the `TripleHooks` tripwire above is, which was measured and
/// not assumed.** `src/` holds 30 `MarchScope` literals, so a bare field addition is `E0063` in
/// the LIB and `cargo build --tests` never reaches a test target (measured: the addition produced
/// two `E0063`s and named no test file). It fires on the second half of the same sequence — a
/// slice that adds the field and repairs `src/`, because the lib must compile — and it fires by
/// construction, since this literal names all five fields and spreads none.
#[test]
fn p4_the_march_scope_does_not_grow() {
    let s = MarchScope { b0: None, lag: None, tau_gov: None, v0: None, ic_order: None };
    assert_eq!(s.b0, MarchScope::DEFAULT.b0);
    assert_eq!(s.v0, MarchScope::DEFAULT.v0);
    assert_eq!(s.ic_order, MarchScope::DEFAULT.ic_order);
}

// =============================================================================================
// GATE 5 — `_gov_max`'s CARRIER AND ITS GUARD
// =============================================================================================

/// **THE CARRIER STARTS UNARMED, AND A SIBLING BUILT BY THE LADDER STARTS UNARMED TOO.**
///
/// Python's `_gov_max = None` is a CLASS attribute, so every fresh machine reads it until
/// `_split_rig` / `_full_rig` write an instance one. That matters beyond a default: the rigs write
/// the field on a machine `at_lever` has just constructed, so if the constructor carried a
/// governor over, every rig cell in every table would inherit the previous cell's set point.
#[test]
fn gov_max_starts_unarmed_on_every_machine_the_ladder_builds() {
    assert_eq!(cross_machine(&cross_arm()).fuel.inner.gov_max.get(), None);
    assert_eq!(full_machine(&full_arm()).fuel.inner.gov_max.get(), None);
    // And rungs below 70 carry the field unarmed rather than not carrying it -- the class
    // attribute is `None` all the way down.
    let r69 = full_of(build_reference_split_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &full_arm()));
    assert_eq!(r69.fuel.inner.gov_max.get(), None);
}

/// **THE GUARD SETS, REPORTS WHAT IT DISPLACED, AND RESTORES THE PREVIOUS *VALUE* — AND HERE THAT
/// LAST WORD IS REACHABLE BY AN ORDINARY WITNESS.**
///
/// This is the half of slice AB's reasoning that must NOT be copied across. `RefScope` also
/// restores the previous value, but over its own suite `_ref`'s displaced value was `None` at
/// every one of its 29 value-sets, so restore-previous and restore-`None` agreed on every shipped
/// path and only a manufactured nest could separate them.
///
/// `_with_gov` is the MIRROR, and the evidence is an ENUMERATION rather than a count: `engine.py`
/// holds exactly **three** `_with_gov` call sites in the whole ladder — `split_gains` at rung 70
/// and two inherited readers at rungs 80/81 — and **all three pass a literal `None`**. So the two
/// spellings agree at the SET and differ at the RESTORE whenever a rig has armed the receiver,
/// which is the shape below. It is the SHIPPED nesting shape, not a manufactured one, and a
/// restore-to-`None` spelling fails it (mutation 5 of the step's nine).
///
/// **§ 5.27 (vii)'s TWO COUNTS FOR THIS DO NOT RECONCILE, SO NEITHER IS THE WITNESS.** Probe 8
/// attributes **98 sets** to `_with_gov` over the whole suite — two writes per call, so 49 calls —
/// against probe 4's **35 calls**. The enumeration above needs neither, which is why it is the
/// thing asserted; recorded rather than silently resolved in favour of the smaller number.
#[test]
fn the_gov_guard_restores_the_previous_value_which_is_the_mirror_of_slice_ab() {
    let m = cross_machine(&cross_arm());
    let core = &m.fuel.inner;

    {
        // The rigs' shape: a machine with the governor armed.
        let outer = GovScope::set(core, Some(TT4_MAX));
        assert_eq!(outer.displaced(), None, "the fresh machine had none");
        assert_eq!(core.gov_max.get(), Some(TT4_MAX));

        {
            // `_with_gov(None, …)` — the 35-of-35 shape: a reader run against rung 68's fuel leg
            // on a machine whose governor IS armed.
            let inner = GovScope::set(core, None);
            assert_eq!(inner.displaced(), Some(TT4_MAX),
                       "the displaced value is a VALUE here, which is what AB's never was");
            assert_eq!(core.gov_max.get(), None, "the governor is off inside the scope");
        }

        // THE ASSERTION THE POLICY LIVES OR DIES ON: a restore-to-`None` guard would leave `None`
        // here, and every reader after the scope would silently measure rung 68's plant.
        assert_eq!(core.gov_max.get(), Some(TT4_MAX),
                   "restore-PREVIOUS, not restore-None -- and unlike `_ref` the two disagree");
    }
    assert_eq!(core.gov_max.get(), None, "and the outer scope restores the class default");
}

/// **THE RESTORE SURVIVES AN UNWIND**, which is the half of `finally` a straight-line restore
/// misses. `RefScope`'s gate, one carrier over.
#[test]
fn the_gov_guard_restores_on_an_unwind() {
    let m = cross_machine(&cross_arm());
    let core = &m.fuel.inner;
    let _outer = GovScope::set(core, Some(TT4_MAX));

    let msg = message_of(std::panic::AssertUnwindSafe(|| {
        let _inner = GovScope::set(core, None);
        assert_eq!(core.gov_max.get(), None);
        panic!("a reader aborted inside `_with_gov`");
    }));
    assert!(msg.contains("a reader aborted"), "the witness panicked for its own reason: {msg:?}");
    assert_eq!(core.gov_max.get(), Some(TT4_MAX), "Drop ran on the unwind path");
}

// =============================================================================================
// GATE 6 — THE INHERITED CONSTRUCTOR
// =============================================================================================

/// **NEITHER RUNG DEFINES `__init__`, SO BOTH BUILDERS MUST FIRE RUNG 69's GUARDS — MEASURED BY
/// MESSAGE, NOT BY THE FACT OF A PANIC.**
///
/// This is the regression gate on step 1's one structural change to shipped code: rung 69's
/// builder was split into a table-parameterised body so that rungs 70 and 71 could call it, which
/// is the port of INHERITANCE (the source duplicates nothing here). The risk in that refactor is
/// silent: a guard dropped from the shared body still leaves every rung-69 gate green if the gate
/// only checks that *some* refusal fired.
///
/// **GUARD C's PLACEMENT IS THE PART THAT COULD ONLY BREAK QUIETLY.** It is asserted BEFORE the
/// build, deliberately — rung 57's `lp_disabled` early return is a separate constructor here, so a
/// post-build guard C would be unreachable and rung 69's refusal would be replaced by rung 57's.
/// One shared body is what keeps that placement true for all three builders without restating it,
/// and this gate reads the message that proves which refusal won.
#[test]
fn both_new_builders_inherit_rung69s_constructor_guards_by_message() {
    type Build = fn(TwoSpoolEngine, FlightCondition, f64, Option<ComponentMap>,
                    Option<ComponentMap>, f64, &LeverArm) -> ScheduledStatorTransient;

    for (name, build) in [("rung 70", build_cross_split_cascade as Build),
                          ("rung 71", build_full_split_cascade as Build)] {
        let go = |arm: LeverArm| {
            message_of(move || {
                let _ = build(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm);
            })
        };

        // A — ONE stator, ONE reference.
        let a = go(LeverArm {
            bleed_lim: Some(valve()), stator_inc: Some(inc_stator()),
            stator_lim: Some(phi_stator()), ..Default::default()
        });
        assert!(a.contains("rung-69 is ONE stator with ONE reference"), "{name} guard A: {a:?}");

        // B — a CONSTANT setting, a SCHEDULE or a FLOOR, exactly one.
        let b = go(LeverArm {
            bleed_lim: Some(valve()), stator_inc: Some(inc_stator()),
            stator: StatorArm { vsv_lp: 0.05, ..Default::default() }, ..Default::default()
        });
        assert!(b.contains("rung-69: the LP stators get a CONSTANT setting"),
                "{name} guard B: {b:?}");

        // C — the HOISTED one. The message decides whether rung 69's refusal or rung 57's won.
        let c = go(LeverArm {
            stator_inc: Some(inc_stator()),
            stator: StatorArm { lp_disabled: true, ..Default::default() }, ..Default::default()
        });
        assert!(c.contains("rung-69's incidence floor watches the LP"),
                "{name} guard C fired POST-build, so rung 57's refusal won instead: {c:?}");

        // D — ONE PHYSICAL WALL. Rung 71's containment result is contingent on this one.
        let d = go(LeverArm {
            bleed_lim: Some(valve()),
            stator_inc: Some(StatorIncidenceLimiter::from_margin(
                &lp_map(), V_MAX, SM * 1.10, Some(TAU))),
            ..Default::default()
        });
        assert!(d.contains("rung-69 needs ONE PHYSICAL WALL"), "{name} guard D: {d:?}");

        // Rung 68's set-point identity, inherited two rungs down.
        let e = go(LeverArm {
            bleed_lim: Some(valve()),
            stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM * 1.10, Some(TAU))),
            ..Default::default()
        });
        assert!(e.contains("rung-68 s 2's identity needs ONE SET POINT"), "{name} rung 68: {e:?}");

        // Rung 62's two-way valve exclusion, inherited eight rungs down.
        let f = go(LeverArm { bleed: 0.05, bleed_sched: Some(BleedSchedule::new(B, 0.65)),
                              ..Default::default() });
        assert!(f.contains("rung-62: the valve gets a CONSTANT position"), "{name} rung 62: {f:?}");

        // THE CONTROL: the matched arming BUILDS, so the six messages above are measurements.
        assert!(go(cross_arm()).is_empty(), "{name}: the rung-70 arming must build");
        assert!(go(full_arm()).is_empty(), "{name}: the rung-71 arming must build");
    }
}
