//! SLICE AD step 6 — **THE THREE CELLS, EACH REPLACED BY THE POINTER ITS PARENT TABLE ACTUALLY
//! HOLDS — AND § 5.28 (vi) SAID NO SUCH POINTER EXISTS.**
//!
//! `slice_ad_oracle.rs` is green at 54 116 keys per arm against two interpreters, `rung72.rs`
//! carries 28 ported gates, and `slice_ad_cells.rs` gates the three cell BODIES. Every one of
//! those is a value instrument, and **no value key can witness a hook table**: slice AD's residual
//! risk is a cell that is written, correct, and never READ — the reader inlining what the table
//! was supposed to dispatch. That is what this file measures, and nothing else does.
//!
//! # THE FINDING — **A FIRST DEFINER STILL HAS A PARENT POINTER, BECAUSE THE PARENT CARRIES A REFUSAL**
//!
//! § 5.28 (vi) reasoned: rung 72 is the **first** definer of all three cells, therefore *"there is
//! no parent function to install… and the gate is AB's declared exception"* — slice AB's
//! *"THE ONE INJECTION THAT IS NOT A PARENT POINTER, AND IT IS DECLARED AS SUCH"*, a body written
//! by hand in the test file.
//!
//! **Measured: `R71_TRIPLE` holds a pointer in all three slots.** It is `no_triple_reference` /
//! `no_triple_rk4_floor_shared` / `no_triple_shared_rig` — the shared refusal `NO_TRIPLE` installs
//! and every table from rung 68 to rung 71 inherits unchanged
//! ([`the_three_injected_pointers_are_the_crates_own_shared_refusal`] asserts it is the *same
//! address* at both ends of that range, so "inherited" is a measurement).
//! `slice_ad_cells.rs`'s own `the_three_cells_are_new_pointers_and_the_other_ten_are_not` already
//! established this at step 1 — it asserts `!fn_addr_eq(R72_TRIPLE.reference,
//! R71_TRIPLE.reference)`, which cannot be written unless the parent slot holds something.
//!
//! So installing it is **slice AB's RULE, not slice AB's EXCEPTION**: AB's own
//! `parent_swap!(P_WITH_REF, with_ref)` does exactly this, on a cell whose rung-68 slot is also a
//! refusal, and AB's gate `cell_9a_with_ref_is_reached_by_every_reader` reads the refusal's
//! message. **The pre-flight cited AB's exception where AB's rule applied.** AB's exception is the
//! SECOND injection into that same cell (`C_WITH_REF`, a silent counterfeit), and it exists
//! because a refusal is a loud break and cannot expose a *quiet* one.
//!
//! The distinction is not bookkeeping. A counterfeit's observability is a property of the body
//! **I** wrote, so a gate on it can be satisfied by having written a satisfiable body; a shipped
//! constant's is not. All three injections below are `R71_TRIPLE.<cell>` — spelled through a
//! macro, so the pointer is provably the parent's and not a re-spelling of it.
//!
//! **What is genuinely unavailable at this rung is the counterfeit's half — a VALUE break** — and
//! for exactly § 5.28 (vi)'s reason: `reference` is the bitwise identity here (195 278 of 195 278
//! calls), so no body substituted for it can move a number. That clause of P6 stands; slice AE,
//! where rung 73 overrides `reference` and `req` moves, is where the value break first exists.
//!
//! # THE SEAT MATRIX — **18 CELLS, AND THE PRE-FLIGHT's TABLE NAMES 3 OF THEM**
//!
//! § 5.28 (vii) fixed one seat per cell and a *"laundered by"* column. Run over all six seats
//! (a direct march plus each of the five rig readers) the picture is:
//!
//! | cell | a DIRECT march | each of the 5 rig readers |
//! |---|---|---|
//! | `reference` | **PANIC** `(_reference)` | silent — laundered, bit-identical |
//! | `rk4_floor_shared` | **PANIC** `(_rk4_floor_shared)` | silent — laundered, bit-identical |
//! | `shared_rig` | **silent** | **PANIC** `(_shared_rig)`, all five |
//!
//! 7 panics and 11 silences, emitted by [`the_tally`]. **The eleven silences are not one
//! phenomenon**, and the pre-flight's column only covers ten of them: the five-per-cell reader
//! silences are laundering (the cell is called, on a machine `at_lever` rebuilt around the
//! shipped tables), while `shared_rig`'s march silence is a path that never reaches the cell at
//! all. To an instrument that reports "did it panic" those are the same reading.
//!
//! **What separates them is the OTHER seat, and that is why the matrix is run whole.** Each of the
//! three cells is proved live on its injected core by panicking somewhere, so every silence in its
//! row is demonstrably a property of the PATH rather than of an injection that did not take. A
//! one-seat-per-cell file has no such control: three panics would be entirely consistent with the
//! other fifteen seats being silent because the table never got installed.
//!
//! # WHY A "SILENT" READING NEEDS TWO MORE THINGS BEFORE IT MEANS LAUNDERING
//!
//! With a refusal injection, laundering shows up as the absence of a panic — and *no panic* is
//! equally satisfied by never reaching the cell. So [`the_five_readers_launder_both_march_cells`]
//! asserts all three halves: the reader **completes**, its reading is **bit-identical** to the
//! shipped one, and the shipped one is **non-trivial**. Slice AC step 7's version of this gate
//! (`the_rig_launders_a_triple_injection`) already carries two of the three — the identity and a
//! shipped row count — because § 5.27 (ii)'s registered break shape is an EMPTY SAMPLE. **The
//! COMPLETION half is what a refusal injection adds**, and it is the half that matters here: AC
//! injected a rival body, which cannot raise, so "it returned" was never in question.
//!
//! # THE FINGERPRINT, AND ITS ONE BLIND SPOT, DECLARED
//!
//! The five readers return five different structs. Rather than 150 lines of per-field `to_bits`,
//! a reading is fingerprinted by its `Debug` string. Rust prints an `f64` as the shortest decimal
//! that round-trips, so the map is **injective on every finite `f64` and separates `-0.0` from
//! `0.0`** — but it collapses all NaN payloads to `NaN`, which is the one difference it cannot
//! see. Declared rather than left implicit, and paired with
//! [`the_fingerprint_moves_for_every_reader_when_the_plant_does`], which perturbs the plant and
//! requires all five strings to move — the "make the instrument prove it can see" rule
//! ([[rust-port-slice-w-step3]]), applied to this file's own instrument.
//!
//! The DIRECT march is fingerprinted `to_bits` per point, with no such caveat.
//!
//! # WHAT THIS FILE DOES NOT GATE
//!
//! **The cell bodies.** `slice_ad_cells.rs` owns them: `reference`'s bit-identity, the floor's
//! message and its `<=` boundary, and `shared_rig`'s four arming flags. This file asks only
//! whether the slot is READ at a seat, which is the one question a value instrument cannot ask.

use std::ptr::fn_addr_eq;

use turbojet::bleed_transient::{LeverArm, LeverArming};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::{
    authority_law, build_shared_actuator_cascade, mask_discriminator, shared_bill, shared_cells,
    shared_gains, R72, R72_FUEL, R72_STATOR, R72_TRIPLE, R72_TWO,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{StatorLimiter, TripleHooks, NO_TRIPLE, R68_TRIPLE};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------------- the grid
//
// **COPIED FROM `tests/slice_ad_march.rs`, WHICH IS `tests/test_rung72.py`'s OWN.** Spelled
// rather than remembered for § 5.27.6 (a)'s reason: slice AC's pre-flight table was taken at a
// stride the fixture never passes, and every delta read off it would have been wrong while still
// passing. This slice adds no constant of its own and this file must not be where one appears.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const B: f64 = 0.10;
/// The EXPRESSION, never a typed decimal — the floors being ONE physical wall is what the rung's
/// pair readings rest on, and a rounded constant would break it silently.
const SM: f64 = PHI / FLOOR - 1.0;
const V_MAX: f64 = 0.20;
const TT4_MAX: f64 = 1200.0;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;

/// ONE clock quadruple, and one stride. The readers' own grids live in the oracle; this file asks
/// a question about DISPATCH, which no grid can change, so it runs the cheapest one that still
/// produces a non-trivial reading — asserted, not assumed, by
/// [`the_five_readers_launder_both_march_cells`].
const CLOCKS: [(f64, f64, f64, f64); 1] = [(TAU_ATT, TAU_GOV, TAU, TAU_S)];
const EVERY: usize = 8;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn valve() -> BleedLimiter { BleedLimiter::with_tau(PHI, B, Some(TAU)) }
fn phi_stator() -> StatorLimiter { StatorLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S)) }

/// Rung 72's arming — rung 70's machine. The fourth loop is armed by a MARCH argument and
/// `_gov_max`, never by an `at_lever` keyword, which is why the signature does not grow here.
fn arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_lim: Some(phi_stator()), ..Default::default() }
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// The SHIPPED construction path — every guard in the chain re-asserted.
fn cascade() -> ScheduledStatorCore {
    full_of(build_shared_actuator_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, &arm()))
}

/// A rung-72 machine with a CHOSEN triple table.
///
/// **NOT through [`build_shared_actuator_cascade`]**, which hardcodes `R72_TRIPLE` — installing a
/// table it would never install is the whole point. It also bypasses the builder's guards, which
/// is why [`the_chosen_table_constructor_reproduces_the_cascade_builder_exactly`] exists: without
/// it, every break in this file would be confoundable with the construction path.
fn chosen(tri: &'static TripleHooks) -> ScheduledStatorCore {
    let a = arm();
    full_of(ScheduledStatorTransient::with_ref_tables(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, a.stator,
        &R72_TWO, &R72_STATOR, &R72_FUEL, &R72,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        tri, a.stator_lim, a.stator_inc))
}

// ------------------------------------------------------------------------- the three injections
//
// Through a macro, so the injected pointer is provably `R71_TRIPLE`'s own and not a hand-copied
// body — slice AB's `parent_swap!`, verbatim in shape and three rungs on. A re-spelling would be
// a THIRD implementation and could agree with neither side.

macro_rules! parent_swap {
    ($name:ident, $cell:ident) => {
        static $name: TripleHooks = TripleHooks {
            $cell: turbojet::full_split::R71_TRIPLE.$cell,
            ..R72_TRIPLE
        };
    };
}
parent_swap!(P_REFERENCE, reference);
parent_swap!(P_RK4_FLOOR_SHARED, rk4_floor_shared);
parent_swap!(P_SHARED_RIG, shared_rig);

/// The tag each refusal appends. **The gates read THESE and never `NO_SHARED_MSG`'s prose**, which
/// all three carry character for character — a gate on the shared sentence would discriminate
/// nothing, which is § 5.28 (v)'s finding about the shipped Python needle reproduced inside the
/// file whose job is to avoid it.
const TAGS: [(&str, &str); 3] = [
    ("reference", "(_reference)"),
    ("rk4_floor_shared", "(_rk4_floor_shared)"),
    ("shared_rig", "(_shared_rig)"),
];

// -------------------------------------------------------------------------------- the machinery

/// The panic message a closure produced, or `""` if it returned. `slice_ad_cells.rs`'s helper, and
/// for its reason: `assert!(panics(…))` is satisfied by an unrelated bug as readily as by the
/// refusal it names.
fn message_of<F: FnOnce()>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    // `AssertUnwindSafe` because every core here is built out of `Cell`s, so `&Core` is never
    // `UnwindSafe`. Sound for the usual reason: each closure either panics (and its core is
    // dropped unread) or returns, and no gate reads a core after a caught panic.
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    match out {
        Ok(()) => String::new(),
        Err(e) => e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    }
}

/// **THE DIRECT MARCH** — the seat for the two cells `integrate_fuel` dispatches.
///
/// No `*_rig` reader, so nothing is rebuilt and nothing is laundered. All of
/// `r72_integrate_fuel`'s entry conditions are met (`tau_gov`, the lag, and a floor), so this
/// enters RUNG 72's integrator rather than falling through to rung 71's — where neither injected
/// cell exists to be reached, and every gate below would report a silence about the wrong thing.
/// [`the_direct_march_enters_rung_72s_own_integrator`] asserts that entry.
fn direct_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    let leg = StatorLeg {
        accel: None::<&AccelSchedule>,
        surge: Some(Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM))),
        tt4_max: Some(TT4_MAX),
    };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(
        &flight(), &ramp, None, &leg,
        &MarchScope { lag: Some(AsymmetricLag::new(TAU_ATT, TAU_REL)), tau_gov: Some(TAU_GOV),
                      ..MarchScope::DEFAULT }).0
}

/// The direct march as EXACT BITS — `slice_ad_march.rs`'s seven-tuple per point, plus the length,
/// so a truncated trajectory cannot pass as an equal one.
fn march_fp(m: &ScheduledStatorCore) -> Vec<[u64; 7]> {
    let traj = direct_march(m);
    let mut out = vec![[traj.len() as u64; 7]];
    out.extend(traj.iter().map(|p| {
        [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
         p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits()]
    }));
    out
}

/// The five readers, each driven on one core and fingerprinted by its `Debug` string. See the
/// module header for what that map can and cannot see.
///
/// Every one of them reaches `shared_rig`: four through `shared_march`, and `shared_bill` by
/// calling the cell itself once per each of its sixteen arming cells.
type Reader = (&'static str, fn(&ScheduledStatorCore) -> String);

fn readers() -> [Reader; 5] {
    [
        ("authority_law", |c| format!("{:?}", authority_law(
            c, &flight(), LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, DS, V_MAX))),
        ("shared_gains", |c| format!("{:?}", shared_gains(
            c, &flight(), LO, HI, TT4_MAX, SM, CLOCKS[0], false, R, SETTLE, DS, V_MAX, EVERY))),
        ("shared_cells", |c| format!("{:?}", shared_cells(
            c, &flight(), LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, DS, V_MAX, EVERY))),
        ("mask_discriminator", |c| format!("{:?}", mask_discriminator(
            c, &flight(), LO, HI, TT4_MAX, SM, &CLOCKS, false, R, SETTLE, DS, V_MAX, EVERY))),
        ("shared_bill", |c| format!("{:?}", shared_bill(
            c, &flight(), LO, HI, TT4_MAX, SM, CLOCKS[0], false, R, SETTLE, DS, V_MAX))),
    ]
}

// =============================================================================================
// 0 — THE TWO CONTROLS EVERY GATE BELOW RESTS ON
// =============================================================================================

/// **THE CONSTRUCTION CONTROL.** [`chosen`] goes through `with_ref_tables`, which skips the
/// cascade builder's guards; with the SHIPPED table it must reproduce the builder's own machine
/// observationally, bit for bit. Without this, a panic in this file could be the construction path
/// rather than the cell, and a silence could be a machine that was never armed.
#[test]
fn the_chosen_table_constructor_reproduces_the_cascade_builder_exactly() {
    let a = march_fp(&cascade());
    let b = march_fp(&chosen(&R72_TRIPLE));
    assert!(a.len() > 100, "the control is worthless on a stub trajectory: {} points", a.len());
    assert_eq!(a, b,
               "`with_ref_tables` with the shipped tables must be OBSERVATIONALLY identical to \
                `build_shared_actuator_cascade` — it is a way of choosing a table, not a \
                different plant");
}

/// **THE SEAT CONTROL.** The direct march must enter RUNG 72's integrator. If any of
/// `r72_integrate_fuel`'s entry conditions were unmet it would fall through to rung 71's, where
/// `reference` and `rk4_floor_shared` are never called — and gates 1 and 2 would then be
/// measuring an arming mistake while reporting a dispatch one.
#[test]
fn the_direct_march_enters_rung_72s_own_integrator() {
    let traj = direct_march(&chosen(&R72_TRIPLE));
    assert!(traj.len() > 100, "a real trajectory: {} points", traj.len());
    assert!(traj.iter().all(|p| matches!(p.extra, PointExtra::Shared { .. })),
            "every point must come from rung 72's six-state marcher");
}

// =============================================================================================
// 1–3 — THE THREE DISPATCH GATES, ONE PER CELL
// =============================================================================================

/// **CELL 1 — `reference`, at the seat § 5.28 (vii) fixed for it.**
///
/// The claim is exactly *the slot is READ here*: rung 71's pointer is the shared refusal, so if
/// `r72_integrate_fuel` dispatches through the table the march raises, and if it inlined `req`
/// instead the march would complete and no value key in the crate could tell — `reference` is the
/// bitwise identity at this rung, so the inlined and the dispatched plants are numerically the
/// same machine. **This gate is the only instrument in the slice that can see the difference.**
///
/// Read on the discriminating tag, never on the shared refusal prose.
#[test]
fn cell_1_reference_is_reached_by_the_direct_march() {
    let msg = message_of(|| { direct_march(&chosen(&P_REFERENCE)); });
    assert!(!msg.is_empty(),
            "`reference` was not dispatched — the march completed with rung 71's refusal in the \
             slot, so `integrate_fuel` is reading something other than the table");
    assert!(msg.contains("(_reference)"),
            "and the refusal must name WHICH cell was reached; got: {msg:?}");
}

/// **CELL 2 — `rk4_floor_shared`, same seat.**
///
/// Its whole content is the assertion's prose (§ 5.28 (v)), so `slice_ad_cells.rs` gates the
/// message and the `<=` boundary and this gate gates neither. What is left, and is only askable
/// here, is whether the marcher reaches the cell **through the table** at all: a port that called
/// the free function directly would pass every gate at step 1 and this one alone would fail.
#[test]
fn cell_2_rk4_floor_shared_is_reached_by_the_direct_march() {
    let msg = message_of(|| { direct_march(&chosen(&P_RK4_FLOOR_SHARED)); });
    assert!(!msg.is_empty(), "`rk4_floor_shared` was not dispatched by the marcher");
    assert!(msg.contains("(_rk4_floor_shared)"),
            "the refusal must name WHICH cell was reached; got: {msg:?}");
}

/// **CELL 3 — `shared_rig`, AND IT IS A CENSUS OVER ALL FIVE READERS, NOT ONE.**
///
/// Eight rungs define this cell (72–80), and five readers dispatch it. Scoring it on one reader
/// would leave the other four free to have inlined the rig — a one-reader gate passes on a crate
/// where four fifths of the dispatch is missing. Slice AB's `cell_9a` is named
/// *"…is reached by EVERY reader"* for the same reason; here the census is run rather than
/// asserted in the name.
#[test]
fn cell_3_shared_rig_is_reached_by_all_five_readers() {
    let mut reached: Vec<&str> = Vec::new();
    for (name, f) in readers() {
        let msg = message_of(|| { let _ = f(&chosen(&P_SHARED_RIG)); });
        assert!(msg.contains("(_shared_rig)"),
                "`{name}` did not dispatch through `shared_rig` — it built its own rig; got: \
                 {msg:?}");
        reached.push(name);
    }
    assert_eq!(reached.len(), 5, "all five rung-72 readers reach the cell: {reached:?}");
}

// =============================================================================================
// 4–5 — THE ELEVEN SILENCES, AND WHY THEY ARE NOT ONE PHENOMENON
// =============================================================================================

/// **THE LAUNDERING, MEASURED — AND WITH THE TWO HALVES THAT STOP IT PASSING VACUOUSLY.**
///
/// `shared_rig`'s third line is `core.at_lever(…)`, whose body rebuilds through the cascade
/// builder and installs the **shipped** tables. So an injection into a core's triple table is
/// washed out before any reader downstream of the rig reads anything, and that is why § 5.28 (vii)
/// seats `reference` and `rk4_floor_shared` on a direct march instead.
///
/// With a refusal injection the laundering reads as the ABSENCE of a panic — which is equally
/// what "the cell is never reached" looks like. So all three halves are asserted:
///
/// 1. the reader **completes** (no panic),
/// 2. its reading is **bit-identical** to the shipped one,
/// 3. and the shipped reading is **non-trivial** — a long string, so two empty readings cannot
///    satisfy 2. § 5.27 (ii)'s registered break shape is an EMPTY SAMPLE, and slice AC's version
///    of this gate asserted 2 alone.
///
/// The fourth half is in the file's structure rather than here: gates 1 and 2 prove the same
/// injected core DOES raise at its own seat, so these ten silences are a property of the reader's
/// path and not of an injection that failed to take.
#[test]
fn the_five_readers_launder_both_march_cells() {
    let mut silent = 0usize;
    for (name, f) in readers() {
        let shipped = f(&chosen(&R72_TRIPLE));
        assert!(shipped.len() > 400,
                "`{name}`'s shipped reading must be non-trivial or the identity below is \
                 vacuous: {} chars", shipped.len());
        for (cell, tri) in [("reference", &P_REFERENCE),
                            ("rk4_floor_shared", &P_RK4_FLOOR_SHARED)] {
            let mut got = String::new();
            let msg = message_of(|| { got = f(&chosen(tri)); });
            assert!(msg.is_empty(),
                    "`{name}` raised on a `{cell}` injection — if this ever fires the laundering \
                     is gone and § 5.28 (vii)'s seat for that cell is wrong; got: {msg:?}");
            assert_eq!(got, shipped,
                       "`{name}` must be BIT-IDENTICAL under a laundered `{cell}` injection");
            silent += 1;
        }
    }
    assert_eq!(silent, 10, "five readers x two march-seated cells");
}

/// **THE ELEVENTH SILENCE, AND IT IS A DIFFERENT ONE.**
///
/// A `shared_rig` injection is invisible to the direct march — but not because anything laundered
/// it. The march is handed a machine that is already built, so it never calls the rig at all. The
/// reading is the same "no panic" the ten above produce, from a path that does not reach the cell
/// rather than one that reaches a rebuilt copy of it.
///
/// **The discriminator is gate 3**, which proves the very same injected core raises the moment a
/// reader asks it to build a rig. Together the two readings say the silence belongs to the march's
/// path; on its own, this one would be indistinguishable from an injection that never took.
#[test]
fn a_shared_rig_injection_is_invisible_to_the_direct_march() {
    let shipped = march_fp(&chosen(&R72_TRIPLE));
    let mut got = Vec::new();
    let msg = message_of(|| { got = march_fp(&chosen(&P_SHARED_RIG)); });
    assert!(msg.is_empty(), "the marcher does not build a rig, so it cannot raise: {msg:?}");
    assert!(shipped.len() > 100, "non-trivial, or the identity means nothing");
    assert_eq!(got, shipped, "bit-identical — the cell is simply not on this path");
}

// =============================================================================================
// 6–8 — THE INSTRUMENTS, AND THE TALLY
// =============================================================================================

/// **THE INJECTED POINTERS ARE THE CRATE's OWN, AND THE SAME ONE AT BOTH ENDS OF THE LADDER.**
///
/// This is the gate behind the module header's correction to § 5.28 (vi). If the parent slots were
/// empty — as the pre-flight reasoned they must be, rung 72 being the first definer — none of
/// these three assertions could be written at all.
///
/// The refusal is `NO_TRIPLE`'s and is carried UNCHANGED by every table from rung 68 to rung 71,
/// which is asserted as address equality across that whole range rather than inferred from one
/// end. That is what makes `R71_TRIPLE.<cell>` the parent pointer rather than a body rung 71
/// happened to write.
#[test]
fn the_three_injected_pointers_are_the_crates_own_shared_refusal() {
    // Each injection carries the PARENT's pointer, and it differs from rung 72's own body.
    assert!(fn_addr_eq(P_REFERENCE.reference, turbojet::full_split::R71_TRIPLE.reference));
    assert!(fn_addr_eq(P_RK4_FLOOR_SHARED.rk4_floor_shared,
                       turbojet::full_split::R71_TRIPLE.rk4_floor_shared));
    assert!(fn_addr_eq(P_SHARED_RIG.shared_rig, turbojet::full_split::R71_TRIPLE.shared_rig));
    assert!(!fn_addr_eq(P_REFERENCE.reference, R72_TRIPLE.reference),
            "an injection equal to the shipped body would make gate 1 unfalsifiable");
    assert!(!fn_addr_eq(P_RK4_FLOOR_SHARED.rk4_floor_shared, R72_TRIPLE.rk4_floor_shared));
    assert!(!fn_addr_eq(P_SHARED_RIG.shared_rig, R72_TRIPLE.shared_rig));

    // And every OTHER cell of each injection is still rung 72's — so a break below is the ONE
    // swapped pointer and not the spread.
    assert!(fn_addr_eq(P_REFERENCE.shared_rig, R72_TRIPLE.shared_rig));
    assert!(fn_addr_eq(P_REFERENCE.rk4_floor_shared, R72_TRIPLE.rk4_floor_shared));
    assert!(fn_addr_eq(P_RK4_FLOOR_SHARED.reference, R72_TRIPLE.reference));
    assert!(fn_addr_eq(P_SHARED_RIG.reference, R72_TRIPLE.reference));

    // The refusal is the SAME pointer from `NO_TRIPLE` up through rung 71 — inherited, measured.
    assert!(fn_addr_eq(NO_TRIPLE.reference, R68_TRIPLE.reference));
    assert!(fn_addr_eq(NO_TRIPLE.reference, turbojet::full_split::R71_TRIPLE.reference));
    assert!(fn_addr_eq(NO_TRIPLE.rk4_floor_shared, R68_TRIPLE.rk4_floor_shared));
    assert!(fn_addr_eq(NO_TRIPLE.rk4_floor_shared,
                       turbojet::full_split::R71_TRIPLE.rk4_floor_shared));
    assert!(fn_addr_eq(NO_TRIPLE.shared_rig, R68_TRIPLE.shared_rig));
    assert!(fn_addr_eq(NO_TRIPLE.shared_rig, turbojet::full_split::R71_TRIPLE.shared_rig));
}

/// **THE FINGERPRINT PROVES IT CAN SEE, ON ALL FIVE READERS.**
///
/// A `Debug`-string identity is worth nothing unless the string moves when the reading does, and
/// ten of this file's laundering assertions are equalities on exactly that string. So the plant is
/// perturbed — `sm` nudged by one part in ten thousand, which every reader's rig feeds into its
/// floors — and all five strings must change. [[rust-port-slice-w-step3]]: five of six injections
/// passed 88 gates because nobody made the instrument prove it could see.
#[test]
fn the_fingerprint_moves_for_every_reader_when_the_plant_does() {
    let core = chosen(&R72_TRIPLE);
    for (name, f) in readers() {
        let base = f(&core);
        // The SAME reader on the same core is deterministic — the equality half of the control.
        assert_eq!(f(&core), base, "`{name}` is not deterministic; every gate here is void");
        assert!(!base.is_empty(), "`{name}` produced an empty fingerprint");
    }
    // The inequality half, on a plant that moved. Spelled out per reader rather than driven
    // through `readers()` because the perturbation is in an ARGUMENT, not in the core.
    let fl = flight();
    let s2 = SM * 1.0001;
    let moved: [(&str, bool); 5] = [
        ("authority_law",
         format!("{:?}", authority_law(&core, &fl, LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, DS,
                                       V_MAX))
         != format!("{:?}", authority_law(&core, &fl, LO, HI, TT4_MAX, s2, &CLOCKS, R, SETTLE,
                                          DS, V_MAX))),
        ("shared_gains",
         format!("{:?}", shared_gains(&core, &fl, LO, HI, TT4_MAX, SM, CLOCKS[0], false, R,
                                      SETTLE, DS, V_MAX, EVERY))
         != format!("{:?}", shared_gains(&core, &fl, LO, HI, TT4_MAX, s2, CLOCKS[0], false, R,
                                         SETTLE, DS, V_MAX, EVERY))),
        ("shared_cells",
         format!("{:?}", shared_cells(&core, &fl, LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, DS,
                                      V_MAX, EVERY))
         != format!("{:?}", shared_cells(&core, &fl, LO, HI, TT4_MAX, s2, &CLOCKS, R, SETTLE,
                                         DS, V_MAX, EVERY))),
        ("mask_discriminator",
         format!("{:?}", mask_discriminator(&core, &fl, LO, HI, TT4_MAX, SM, &CLOCKS, false, R,
                                            SETTLE, DS, V_MAX, EVERY))
         != format!("{:?}", mask_discriminator(&core, &fl, LO, HI, TT4_MAX, s2, &CLOCKS, false,
                                               R, SETTLE, DS, V_MAX, EVERY))),
        ("shared_bill",
         format!("{:?}", shared_bill(&core, &fl, LO, HI, TT4_MAX, SM, CLOCKS[0], false, R,
                                     SETTLE, DS, V_MAX))
         != format!("{:?}", shared_bill(&core, &fl, LO, HI, TT4_MAX, s2, CLOCKS[0], false, R,
                                        SETTLE, DS, V_MAX))),
    ];
    let blind: Vec<&str> = moved.iter().filter(|(_, m)| !m).map(|(n, _)| *n).collect();
    assert!(blind.is_empty(),
            "these readers' fingerprints did not move when the plant did, so every equality \
             asserted on them is vacuous: {blind:?}");
}

/// **THE TALLY — THREE SWAPS, EIGHTEEN SEATS, AND EVERY NUMBER RE-RUN RATHER THAN TYPED.**
///
/// *Three dispatch gates* is a claim about SWAPS, not about `#[test]` functions: this file carries
/// three injections across **ten** tests, as slice AB carried ten swaps across fourteen tests and
/// slice AC five across nine.
///
/// # AND THIS SENTENCE WAS WRONG ON ITS FIRST WRITING, IN THIS FILE, ABOUT THIS FILE
///
/// The draft said *"across NINE tests"* and was committed to a prediction of nine before the run;
/// the binary runs **ten**. Nothing measured it — I counted from memory while looking at the ten
/// functions that disprove it, which is § 5.27.6 (g)'s shape exactly (a constant typed at
/// `39_099` beside the addends that measured 5 351) and step 5's own close-out lesson
/// ([[rust-port-slice-ad-step5]]: *re-measure every testable sentence in your own new header*)
/// reproduced one step later by the person who wrote it down.
///
/// So the count is no longer prose. It is **read off this file's own source** and pinned, which
/// is the only version of the claim that cannot drift away from the artifact it describes: adding
/// or deleting a test here fails HERE rather than leaving a stale number in a doc comment.
///
/// The eighteen seats are the module header's matrix, re-run: **7 panics and 11 silences**, and a
/// silence is only ever reported for a cell that is proved live somewhere in its own row.
#[test]
fn the_tally() {
    // THE SWAP COUNT, in code.
    let swaps: Vec<&str> = TAGS.iter().map(|(n, _)| *n).collect();
    assert_eq!(swaps.len(), 3, "three cells, three parent pointers: {swaps:?}");
    assert_eq!(swaps, vec!["reference", "rk4_floor_shared", "shared_rig"]);

    // THE TEST COUNT, READ OFF THE SOURCE — see this gate's doc comment for why it is not typed
    // into one. `include_str!` resolves relative to this file, so it is this file.
    let n_tests = include_str!("slice_ad_dispatch.rs")
        .lines().filter(|l| l.trim() == "#[test]").count();
    assert_eq!(n_tests, 10,
               "ten `#[test]` functions carrying three swaps. If this fires, a test was added or \
                removed and the header's count must move with it — which is the whole point of \
                reading it off the file instead of remembering it.");
    assert!(n_tests > swaps.len(),
            "the two counts are DIFFERENT things — `three dispatch gates` is the swap count");

    // THE SEAT MATRIX, re-run whole: 3 cells x (1 direct march + 5 readers).
    let cells: [(&str, &TripleHooks, &str); 3] = [
        ("reference", &P_REFERENCE, "(_reference)"),
        ("rk4_floor_shared", &P_RK4_FLOOR_SHARED, "(_rk4_floor_shared)"),
        ("shared_rig", &P_SHARED_RIG, "(_shared_rig)"),
    ];
    let (mut panics, mut silences, mut seats) = (0usize, 0usize, 0usize);
    let mut rows: Vec<(&str, usize, usize)> = Vec::new();
    for (cell, tri, tag) in cells {
        let (mut p, mut s) = (0usize, 0usize);
        // seat 0 — the DIRECT march.
        let msg = message_of(|| { direct_march(&chosen(tri)); });
        if msg.is_empty() { s += 1 } else {
            assert!(msg.contains(tag), "a panic at `{cell}`'s march seat must name it: {msg:?}");
            p += 1;
        }
        // seats 1..5 — the five rig readers.
        for (name, f) in readers() {
            let msg = message_of(|| { let _ = f(&chosen(tri)); });
            if msg.is_empty() { s += 1 } else {
                assert!(msg.contains(tag),
                        "a panic at `{cell}`/`{name}` must name the cell: {msg:?}");
                p += 1;
            }
        }
        assert!(p > 0,
                "`{cell}` is silent at ALL SIX seats — the injection did not take, and every \
                 silence reported for it below would be meaningless");
        seats += 6;
        panics += p;
        silences += s;
        rows.push((cell, p, s));
    }
    assert_eq!(seats, 18, "3 cells x 6 seats");
    assert_eq!(
        (panics, silences), (7, 11),
        "MEASURED at step 6: `reference` and `rk4_floor_shared` are reached ONLY by the direct \
         march (1 panic, 5 laundered silences each), `shared_rig` ONLY by the readers (5 panics, \
         1 silence). § 5.28 (vii)'s table names 3 of these 18 seats and one kind of silence. \
         rows: {rows:?}");
    assert_eq!(rows, vec![("reference", 1, 5), ("rk4_floor_shared", 1, 5), ("shared_rig", 5, 1)]);
}
