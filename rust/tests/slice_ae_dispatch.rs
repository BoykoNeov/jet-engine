//! SLICE AE step 5 — **THE SEVEN RE-AIMED POINTERS, EACH REPLACED BY THE ONE ITS PARENT TABLE
//! HOLDS — AND THE ONE PARENT POINTER THAT MAKES A BROKEN PAIRING WORK PERFECTLY.**
//!
//! `slice_ae_oracle.rs` is green on 71 044 compared keys per arm against two interpreters,
//! `rung73.rs` carries the 27 ported gates and `slice_ae_cells.rs` gates the cell BODIES. All
//! three are VALUE instruments, and the oracle's own header names what none of them can reach:
//! *`at_lever` … is the LAUNDERER … no value key can witness which function pointer sat in a
//! slot. That is step 5's subject*, and *`_quad_gains_at`'s POINTER … P4 assigns that seat to
//! step 5, on a DECLARED EXTRA GRID no shipped test sits in.*
//!
//! **THIS FILE IS NOT ONLY ABOUT REFUSALS.** Two of its three registered obligations are
//! refusals — § 5.29 (ix) P1's manufactured pairing, and what happens to it when the guard is
//! taken out — and the third (P4) is a plain VALUE break. Saying so in the header matters
//! because a file that claimed to test only control flow would have no reason to own § 4's
//! sign-bit finding, which is the sharpest number in it.
//!
//! # OBLIGATION 1 — **P1's MANUFACTURED PAIRING**, and the shape it is a pairing of
//!
//! § 5.29 (i): `_with_ref` is rung 69's name and rung 73's, same arity, **different mutated
//! field** (`_ref` against `_ref_law`). Both fields exist on a rung-73 machine, so no signature
//! comparison and no type error can reach it — which is why § 5.27 (x)'s phase-wide sweep filed
//! the pair as harmlessly RENAMED and the port reached the right structure anyway.
//!
//! The port re-aims the slot ([`R73_TRIPLE`]`.with_ref`), so **the refusal is the correctness**:
//! a rung-69 reader on a rung-73 machine writes `"inc"` into `ref_law`, and rung 73's
//! `integrate_fuel` stops it. `slice_ae_cells.rs` drives that assert DIRECTLY — it sets the field
//! by hand — and says in its own header that it *deliberately does not gate* the dispatch through
//! a real inherited reader. § 5.29.3 (d) then measured why that distinction is not bookkeeping:
//! deleting BOTH asserts is caught by 2 of 27 gates in Rust and by the same 2 of 27 in Python,
//! so P1's CONCLUSION is false — but its REASON, *no shipped rung-73 test calls a rung-69
//! reader*, stands untouched and **is measured by neither arm**. Here it is.
//!
//! # OBLIGATION 2 — **P4's `quad_gains_at`**, on a declared extra grid
//!
//! § 5.28.3 (b) booked this cell to slice AE having measured it *unreachable on the shipped
//! ladder*; § 5.29 (iv) refuted the booking BY VALUE with probe J, which holds the machine fixed
//! at rung 73 and swaps **only the pointer**. That device is reproduced in § 4 — never probe H's
//! seat comparison, which runs two different trajectories and so is not a comparison of two
//! bodies at all.
//!
//! # OBLIGATION 3 — **THE SEAT MATRIX, RUN WHOLE**
//!
//! AD step 6's rule: with a parent-pointer injection a *silence* is either laundering (the cell
//! ran, on a machine `at_lever` rebuilt around the shipped tables) or a path that never reaches
//! the cell, and a did-it-break instrument cannot tell those apart. What separates them is the
//! OTHER seat, so § 5 runs all **seven** pointers against all **seven** seats and prints the
//! tally. A one-seat-per-cell file has no such control.
//!
//! **AND AD's OWN CONTROL IS UNAVAILABLE FOR TWO OF THE SEVEN.** AD could assert that every
//! injection breaks somewhere; here `rk4_floor_shared` and `shared_rig` are silent at all seven
//! seats — the first differs only in a MESSAGE, the second is REDUNDANT because `at_lever`
//! carries the reference first. For those two the only evidence the injection took is the
//! STRUCTURAL one, [`assert_installed`]'s pointer identity on the rebuilt sibling, so § 5 states
//! the live set as a measured PARTITION rather than as a universal that happens to hold.
//!
//! # HOW AN INJECTION IS INSTALLED, AND WHY THERE IS NO HIDDEN STATE
//!
//! Every reader on this ladder rebuilds its machine through `at_lever`, and
//! `build_applied_reference_cascade` hardcodes `&R73`/`&R73_TRIPLE`/`&R73_FUEL` — so a lone table
//! injection is LAUNDERED before any value is read (AC step 7's finding; `rung73.rs` gate 9 is
//! the precedent and pairs its counterfeit triple table with a counterfeit LEVER one).
//!
//! [`injection!`] therefore emits, per cell, **its own `at_lever` naming its own tables**. There
//! is no selector, no thread-local and no run-order coupling: which tables a rebuild installs is
//! a property of the function pointer in the table, exactly as it is in the shipped crate.
//! [`triple_diff`] then proves the install by an **exhaustive destructuring** of `TripleHooks`,
//! so the claim *only this slot differs* is checked against all fourteen fields and goes `E0027`
//! when a fifteenth lands.
//!
//! # THE FINGERPRINT, AND ITS ONE BLIND SPOT — DECLARED
//!
//! The seven seats return seven different types, so a reading is fingerprinted by its `Debug`
//! string. Rust prints an `f64` as the shortest decimal that round-trips, so the map is injective
//! on every finite `f64` **and separates `-0.0` from `0.0`** — which § 4 needs and an `==` would
//! not give. It collapses NaN payloads, which is the one difference it cannot see.
//! [`the_matrix_instrument_moves_when_the_plant_does`] perturbs the plant and requires every seat
//! to move, so the instrument proves it can see before it is read
//! ([[rust-port-slice-w-step3]]).
//!
//! # WHAT THIS FILE DOES NOT GATE, AND WHY EACH IS A DECISION
//!
//! * **The cell BODIES.** `slice_ae_cells.rs` owns the two-sided field assertion, the class
//!   default and both refusals driven directly; `rung73.rs` owns the 27 ported claims. This file
//!   asks only whether a slot is READ at a seat, and what the answer is worth.
//! * **`shared_rig`'s carry as a VALUE break.** Pre-registered TWICE as having none — step 1
//!   § (d) (probe L2, and mutation M11 surviving all fifteen gates) and step 2 § (e) (M11 at
//!   **0 of 5 066** keys). § 6 asserts the POINTER and records the reason; it does not hunt a
//!   discriminator that was measured not to exist. The honest two-sided detector is M11b, which
//!   deletes BOTH carries, and step 2 already ran it.
//! * **A CPython arm.** Nothing here reads a golden. Every assertion is a panic, a same-run
//!   difference, or a compile-time property.
//!
//! # TWO BLIND SPOTS, MEASURED RATHER THAN SUSPECTED — a coverage claim owns its holes
//!
//! Step 5's sweep scores six source mutations against this file, `slice_ae_cells.rs`,
//! `rung73.rs` and `slice_ae_oracle.rs`. **One is caught by this file and by nothing else** —
//! collapsing rung 72's three `None` branch indicators to `Some(0.0)`, at 3/10 against 0/15,
//! 0/27, 0/7, which is § 3's discrete half earning its keep on the one distinction the crate had
//! no other instrument for. **One is missed by this file and caught elsewhere**, and its cause is
//! structural:
//!
//! 1. **[`injection!`]'s own `at_lever` copies the reference law**, on the line after it
//!    rebuilds — it must, or a sibling built under a scoped law would read the class default and
//!    every row would be measuring step 1 § (b)'s silent failure instead of the injection. So the
//!    fixture re-implements the statement a mutation of the SHIPPED carry deletes, and no seat
//!    here can see that mutation (0 of 10; `slice_ae_cells.rs` catches it at 1 of 15).
//!    [`the_rig_carry_is_rung_73s_own_pointer_and_its_zero_is_measured_not_inferred`] routes
//!    around it by driving the shipped machine.
//! 2. **[`the_seat_matrix`] is baseline-relative** — every verdict is a difference against a
//!    reading taken from the same tree, so a source change that moves the baseline and all seven
//!    rows together is invisible to it by construction.
//!
//! Both are properties of the instrument, not of the ladder, and both are stated because a file
//! whose whole claim is *this reaches what the value instruments cannot* has to say where it
//! does not reach.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::fn_addr_eq;

use turbojet::applied_reference::{
    applied_bill, applied_cells, applied_gains, build_applied_reference_cascade, handover_law,
    ref_discriminator, REF_LAWS_DECLARED, REF_LAW_APPLIED, R73, R73_FUEL, R73_STATOR, R73_TRIPLE,
    R73_TWO,
};
use turbojet::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Authority, FuelPoint, FuelTransientHooks};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{
    build_reference_split_cascade, reference_bill, RefScope, StatorIncidenceLimiter, R69_TRIPLE,
};
use turbojet::shared_actuator::{
    riding4, shared_march, QuadGains, SharedRigArm, R72, R72_FUEL, R72_TRIPLE,
};
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::{TripleHooks, TripleRigArm};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/rung73.rs`'s own constants, which are `tests/test_rung73.py`'s. This file adds no
// physical constant; the two grids it DOES add are march coordinates and are declared at their
// point of use (§ 4's is `applied_gains`' own default, § 5's is coarse and says so).

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
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
const TT4_MAX: f64 = 1200.0;

/// `applied_gains(…, taus=(0.05,)*4)` — read off `engine.py`'s `def` line, as `rung73.rs`'s own
/// reader-default table is.
const CLOCK: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
/// `applied_gains(…, ds=0.002, every=2)` — § 4 runs the cell at the reader's OWN grid, so the
/// break it measures is the one the shipped reader would have met.
const AG_DS: f64 = 0.002;
const AG_EVERY: usize = 2;

/// **§ 5's DECLARED COARSE GRID.** The matrix runs 7 injections x 7 seats and only ever asks
/// *did this seat's reading move*, never *by how much*; the value questions are §§ 1-4's, at the
/// shipped grids. Coarsening it is what makes the matrix affordable whole, and AD step 6's rule
/// is that running it whole is the point. Declared here rather than buried at the call site.
const MX_DS: f64 = 0.01;
const MX_EVERY: usize = 8;

// ============================================================================== the fixtures

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
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

fn applied_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_applied_reference_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn ref69_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_reference_split_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn valve() -> BleedLimiter { BleedLimiter::from_margin_tau(&lp(), B, SM, Some(TAU)) }

fn inc_stator() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S))
}

fn valve_arm() -> LeverArm { LeverArm { bleed_lim: Some(valve()), ..Default::default() } }

/// The arming §§ 1-3 and § 5 run on. Rung 69's `reference_bill` is a STATOR ledger, so the
/// incidence limiter has to be armed for the reader to have a subject at all — and it is
/// `rung69.rs`'s own `split()` arming (valve + incidence floor), spelled from this file's
/// constants rather than remembered.
fn inc_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_inc: Some(inc_stator()), ..Default::default() }
}

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

/// Rung 69's own reader arm — Python's six clock defaults, `sm` supplied. `rung69.rs`'s
/// `rig_arm()`, verbatim.
fn rig_arm() -> TripleRigArm { TripleRigArm { sm: SM, ..TripleRigArm::default() } }

/// **THIS FILE SILENCES A PANIC PER THREAD, NOT PER PROCESS — AND THE REASON IS A DEFECT THIS
/// FILE's OWN FIRST RUN PRODUCED.**
///
/// The crate's usual idiom (`rung69.rs`, `rung73.rs`) is take-hook / set an empty hook /
/// catch / restore. `std::panic::set_hook` is **process-wide**, and `#[test]` functions run on
/// concurrent threads: this file expects panics in six of its ten gates, so on the first run the
/// empty hook was installed almost continuously and **two genuinely failing gates reported
/// `FAILED` with no message at all** — the assertion text, the file and the line were all
/// swallowed. A test file that cannot show why it failed is not a gate.
///
/// So the hook is installed **once** for the binary and consults a THREAD-LOCAL: a thread inside
/// [`caught`] or [`run_seat`] is quiet, every other thread keeps the default hook and prints its
/// failure. `Once` makes the install race-free, and nothing else in this file touches the hook —
/// which is what makes the per-thread scoping true rather than intended.
///
/// The `AssertUnwindSafe` is `rung73.rs`'s, for its reason: every machine in this family carries
/// the `Cell` fields that ARE the dynamically scoped state, and they are rebuilt per test, so a
/// poisoned one cannot outlive the assertion.
fn quiet_hook() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !QUIET.with(|q| q.get()) {
                prev(info);
            }
        }));
    });
}

thread_local! {
    static QUIET: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run `f`, returning its panic message if it had one. `pytest.raises(AssertionError, match=…)`,
/// with the message RETURNED so a caller can put a second needle on it.
fn caught<F: FnOnce()>(f: F) -> Option<String> {
    quiet_hook();
    QUIET.with(|q| q.set(true));
    let out = catch_unwind(AssertUnwindSafe(f));
    QUIET.with(|q| q.set(false));
    match out {
        Ok(()) => None,
        Err(e) => Some(e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()),
    }
}

// =============================================================================================
// THE INJECTIONS — one `at_lever` per cell, so a rebuild installs the injected tables
// =============================================================================================

/// Build a rung-73 machine on an ARBITRARY table triple.
///
/// **NOT through [`build_applied_reference_cascade`]**, which hardcodes `&R73`, `&R73_FUEL` and
/// `&R73_TRIPLE` — the point is to install tables it would never install. The class attribute the
/// cascade applies is re-applied here, because a machine that kept the core constructor's
/// `"sched"` would PASS rung 73's own refusal while marching rung 72 (step 1 § (b)'s first silent
/// failure), and every reading below would then be measuring that instead of the injection.
fn with_tables(
    core: &ScheduledStatorCore, arm: &LeverArm, lever: &'static LeverHooks,
    fuel: &'static FuelTransientHooks, triple: &'static TripleHooks,
) -> ScheduledStatorCore {
    let c = full_of(ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        arm.stator, &R73_TWO, &R73_STATOR, fuel, lever,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        triple, arm.stator_lim, arm.stator_inc));
    c.fuel.inner.ref_law.set(REF_LAW_APPLIED);
    c
}

/// One injection: a `LeverHooks` whose `at_lever` rebuilds with **that injection's own tables**.
///
/// The sibling constructor is what launders a table injection (AC step 7), so an injection that
/// did not re-aim it would be undone the first time a reader rebuilt — and the gate would be
/// green and vacuous, which is this phase's most-repeated defect. Emitting the lever table beside
/// the cell table makes the pairing structural: there is no selector to forget to set, and each
/// `at_lever` names its tables in its own body.
///
/// The law is copied from the SOURCE core for `r73_at_lever`'s reason — a sibling built while the
/// receiver sits under a `RefScope`-set law must carry that law and not the class default.
macro_rules! injection {
    ($lever:ident, $rebuild:ident, $fuel:expr, $triple:expr) => {
        static $lever: LeverHooks = LeverHooks { at_lever: $rebuild, ..R73 };
        fn $rebuild(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
            let m = with_tables(core, arm, &$lever, $fuel, $triple);
            m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
            m
        }
    };
}

/// RUNG 69's `_with_ref` — the NAME REUSE, pointed back at the field it writes there (`_ref`).
static T_WITH_REF: TripleHooks = TripleHooks { with_ref: R69_TRIPLE.with_ref, ..R73_TRIPLE };
/// RUNG 72's `_reference` — the bitwise identity. `rung73.rs` gate 9 injects a COUNTERFEIT here
/// (Python's `Broken` subclass); this is the parent POINTER, which is a different question.
static T_REFERENCE: TripleHooks = TripleHooks { reference: R72_TRIPLE.reference, ..R73_TRIPLE };
static T_RK4: TripleHooks =
    TripleHooks { rk4_floor_shared: R72_TRIPLE.rk4_floor_shared, ..R73_TRIPLE };
static T_SHARED_RIG: TripleHooks = TripleHooks { shared_rig: R72_TRIPLE.shared_rig, ..R73_TRIPLE };
/// **P4's INJECTION.** The one cell this slice ADDS, replaced by the pointer rung 72's table
/// holds — which is `shared_actuator::quad_gains_at`, a real body and not a refusal, so the
/// break can be a VALUE break. (AD step 6's finding was that a first definer still has a parent
/// pointer *because the parent slot carries a refusal*; here the parent slot carries a rival.)
static T_QUAD: TripleHooks = TripleHooks { quad_gains_at: R72_TRIPLE.quad_gains_at, ..R73_TRIPLE };
/// **P1's INJECTION** — rung 73's `integrate_fuel` replaced by the parent's, which is the port
/// that re-aims `with_ref` and omits the refusal.
static F_INTEGRATE: FuelTransientHooks =
    FuelTransientHooks { integrate_fuel: R72_FUEL.integrate_fuel, ..R73_FUEL };

injection!(L_NONE, none_at_lever, &R73_FUEL, &R73_TRIPLE);
injection!(L_WITH_REF, with_ref_at_lever, &R73_FUEL, &T_WITH_REF);
injection!(L_REFERENCE, reference_at_lever, &R73_FUEL, &T_REFERENCE);
injection!(L_RK4, rk4_at_lever, &R73_FUEL, &T_RK4);
injection!(L_SHARED_RIG, shared_rig_at_lever, &R73_FUEL, &T_SHARED_RIG);
injection!(L_QUAD, quad_at_lever, &R73_FUEL, &T_QUAD);
injection!(L_INTEGRATE, integrate_at_lever, &F_INTEGRATE, &R73_TRIPLE);

/// The `at_lever` cell itself — rung 72's sibling constructor, which builds a RUNG-72 machine.
/// It is the only row with no `injection!`: re-aiming `at_lever` at the parent is precisely
/// *stop carrying this rung's tables*, so a rebuild that carried them would be the opposite of
/// the injection.
static L_AT_LEVER: LeverHooks = LeverHooks { at_lever: R72.at_lever, ..R73 };

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Inj { None, AtLever, WithRef, Reference, Rk4Floor, SharedRig, IntegrateFuel, QuadGains }

/// The seven injections plus the baseline, in the order § 5 prints them.
const INJS: [Inj; 8] = [Inj::None, Inj::AtLever, Inj::WithRef, Inj::Reference, Inj::Rk4Floor,
                        Inj::SharedRig, Inj::IntegrateFuel, Inj::QuadGains];

fn tables_of(inj: Inj) -> (&'static LeverHooks, &'static FuelTransientHooks, &'static TripleHooks) {
    match inj {
        Inj::None => (&L_NONE, &R73_FUEL, &R73_TRIPLE),
        Inj::AtLever => (&L_AT_LEVER, &R73_FUEL, &R73_TRIPLE),
        Inj::WithRef => (&L_WITH_REF, &R73_FUEL, &T_WITH_REF),
        Inj::Reference => (&L_REFERENCE, &R73_FUEL, &T_REFERENCE),
        Inj::Rk4Floor => (&L_RK4, &R73_FUEL, &T_RK4),
        Inj::SharedRig => (&L_SHARED_RIG, &R73_FUEL, &T_SHARED_RIG),
        Inj::IntegrateFuel => (&L_INTEGRATE, &F_INTEGRATE, &R73_TRIPLE),
        Inj::QuadGains => (&L_QUAD, &R73_FUEL, &T_QUAD),
    }
}

/// A rung-73 machine carrying one injection. **[`Inj::None`] is a real machine and not a
/// shortcut** — it goes through the same `with_ref_tables` path with the shipped tables, so every
/// baseline reading below differs from an injected one in the POINTER and in nothing else.
fn build(inj: Inj, arm: &LeverArm) -> ScheduledStatorCore {
    let (lever, fuel, triple) = tables_of(inj);
    with_tables(&applied_of(arm), arm, lever, fuel, triple)
}

/// Which of [`TripleHooks`]' fields differ between two tables, **by exhaustive destructuring**.
///
/// A `fn_addr_eq` on the one slot an injection names proves the slot moved; it does not prove
/// nothing else did. This lists all fourteen, so *only this slot differs* is a measurement — and
/// the destructuring goes `E0027` when a fifteenth field lands, which is `slice_ae_cells.rs`'s
/// own tripwire (it fired at step 2, exactly as its doc comment predicted).
fn triple_diff(a: &'static TripleHooks, b: &'static TripleHooks) -> Vec<&'static str> {
    let TripleHooks {
        stator_leg, lagged_stator, clamp_v, check_v0, rk4_floor, solve_v, manifold_v, triple_laws,
        triple_rig, with_ref, reference, rk4_floor_shared, shared_rig, quad_gains_at,
        // SLICE AF's FOUR. Named and CHECKED below, not bound and dropped: the whole content of
        // this function is *only the named slot differs*, and a field left unchecked would make
        // that sentence false for four slots at once.
        cap_fuel, sensed_cap, windup_tau, with_coord,
    } = *a;
    let mut out = Vec::new();
    let mut chk = |name, same: bool| if !same { out.push(name) };
    chk("stator_leg", fn_addr_eq(stator_leg, b.stator_leg));
    chk("lagged_stator", fn_addr_eq(lagged_stator, b.lagged_stator));
    chk("clamp_v", fn_addr_eq(clamp_v, b.clamp_v));
    chk("check_v0", fn_addr_eq(check_v0, b.check_v0));
    chk("rk4_floor", fn_addr_eq(rk4_floor, b.rk4_floor));
    chk("solve_v", fn_addr_eq(solve_v, b.solve_v));
    chk("manifold_v", fn_addr_eq(manifold_v, b.manifold_v));
    chk("triple_laws", fn_addr_eq(triple_laws, b.triple_laws));
    chk("triple_rig", fn_addr_eq(triple_rig, b.triple_rig));
    chk("with_ref", fn_addr_eq(with_ref, b.with_ref));
    chk("reference", fn_addr_eq(reference, b.reference));
    chk("rk4_floor_shared", fn_addr_eq(rk4_floor_shared, b.rk4_floor_shared));
    chk("shared_rig", fn_addr_eq(shared_rig, b.shared_rig));
    chk("quad_gains_at", fn_addr_eq(quad_gains_at, b.quad_gains_at));
    chk("cap_fuel", fn_addr_eq(cap_fuel, b.cap_fuel));
    chk("sensed_cap", fn_addr_eq(sensed_cap, b.sensed_cap));
    chk("windup_tau", fn_addr_eq(windup_tau, b.windup_tau));
    chk("with_coord", fn_addr_eq(with_coord, b.with_coord));
    out
}

/// The install proof every injected reading below is preceded by: rebuild a sibling through the
/// injected `at_lever`, and assert the tables it hands back are the injected ones and differ from
/// the shipped ones in EXACTLY the named slot.
///
/// # **`ptr::eq` IS NOT A TABLE-IDENTITY TEST HERE, AND THE VERSION THAT PASSED WAS THE WORSE ONE**
///
/// The first writing of this function asserted `std::ptr::eq(sib.triple_hooks(), triple)`. Every
/// table in this family is a `pub const`, not a `static`, so `&R72_TRIPLE` is a fresh rvalue
/// promotion **at each use site** and two of them need not share an address. It failed on the
/// `at_lever` row, where the crate's own builder supplies the pointer.
///
/// **And it PASSED everywhere else, for a reason worse than luck.** On every other row the
/// machine holds the pointer *this fixture handed the builder* three lines earlier, so the
/// assertion was comparing the instrument against itself — the pattern rung 67 gate 9, rung 71
/// § 1.4, rung 72 § 4 and `rung73.rs`'s own gate 9 are all written against, arriving in an
/// install proof. [`triple_diff`] replaces it: fourteen `fn_addr_eq`s, which compare FUNCTION
/// addresses and are stable across promotions, so the claim survives being true for the right
/// reason.
fn assert_installed(m: &ScheduledStatorCore, inj: Inj, triple_slot: &[&str]) {
    let (_, fuel, triple) = tables_of(inj);
    let sib = m.at_lever(&valve_arm());
    assert_eq!(triple_diff(sib.triple_hooks(), &R73_TRIPLE), triple_slot,
               "{inj:?}: the REBUILT sibling must carry the injection and nothing else — a lone \
                table injection is laundered by `at_lever` before any value is read");
    assert_eq!(triple_diff(sib.triple_hooks(), triple), Vec::<&str>::new(),
               "{inj:?}: and it must agree with the intended table in all fourteen slots");
    assert_eq!(fn_addr_eq(sib.fuel.hooks.integrate_fuel, R73_FUEL.integrate_fuel),
               inj != Inj::IntegrateFuel,
               "{inj:?}: the fuel table travels with the rebuild too");
    assert!(fn_addr_eq(sib.fuel.hooks.integrate_fuel, fuel.integrate_fuel));
}

// =============================================================================================
// 1 — P1's MANUFACTURED PAIRING. **A rung-69 reader on a rung-73 machine, refused.**
// =============================================================================================

/// **THE INJECTION P1 ACTUALLY NAMES, AND IT IS MEASURED BY NEITHER SHIPPED ARM.**
///
/// § 5.29 (ix) P1's REASON clause — *no shipped rung-73 test calls a rung-69 reader* — survived
/// § 5.29.3 (d)'s falsification of its conclusion, and this is the gate it was left waiting for.
/// [`reference_bill`] is rung 69 § 4: it runs its ledger twice, once under each reference, and
/// sets each through `RefScope`, which writes **through the cell** — so on a rung-73 machine the
/// setter is `r73_with_ref` and `"inc"` lands in `ref_law`, where Python's own driven probe C
/// puts it.
///
/// **THE NEEDLE IS THE ONE § 5.29 (vii) MEASURED TO DISCRIMINATE.** `"FOUR actuator states"`
/// reaches NINE classes back to rung 43 and `"no set point"` does not match rung 73 at all, so
/// neither is read here. `rung-73` and the reported law are, in that ORDER — the ordering half is
/// what a pair of `contains` alone would drop, and it is `rung73.rs` gate 21's rule applied to a
/// message this file is the first to reach through a reader.
///
/// **AND THE CONTROL ASSERTS A POSITIVE READING, NOT AN ABSENT PANIC.** Step 1 § (b) measured
/// that this arming really can abort for a second reason (probe L5's two declared rows died on
/// `rung-43 fuel closure does not bracket`), so *it did not refuse* is worthless on its own. The
/// rung-69 control must RETURN, its four stator-free cells must agree exactly, and — the half
/// that makes it a reader rather than a survivor — **its two arms must DIFFER**, which is the
/// whole content of a ledger run once per reference.
#[test]
fn a_rung_69_reader_on_a_rung_73_machine_is_refused() {
    let m73 = build(Inj::None, &inc_arm());
    assert_eq!(triple_diff(m73.triple_hooks(), &R73_TRIPLE), Vec::<&str>::new(),
               "the baseline machine carries the SHIPPED table — a difference here would make \
                every reading in this file a measurement of the fixture");

    let msg = caught(|| { let _ = reference_bill(&m73, &flight(), &ramp(DS), SM, &rig_arm()); })
        .expect("rung 73's `integrate_fuel` refuses a reference it did not declare");
    let (tag, got) = ("rung-73", "got \"inc\"");
    assert!(msg.contains(tag) && msg.contains(got), "{msg:?}");
    assert!(msg.find(tag) < msg.find(got),
            "the rung TAG opens the message and the offending law follows it — `match=` in Python \
             is a regex and `str::contains` is not, so the ORDER is asserted separately: {msg:?}");

    // THE CONTROL — the same reader, the same arming, on the machine it belongs to.
    let m69 = ref69_of(&inc_arm());
    let b = reference_bill(&m69, &flight(), &ramp(DS), SM, &rig_arm());
    assert_eq!(b.common_max_rel, 0.0,
               "the four stator-free cells are identical BY CONSTRUCTION: {:?}", b.common);
    assert_ne!(b.delivered.0.to_bits(), b.delivered.1.to_bits(),
               "a ledger run once per reference must DELIVER differently, or the control is a \
                reader that measured nothing: {:?}", b.delivered);
    assert_ne!(b.delivered_inc.0.to_bits(), b.delivered_inc.1.to_bits(), "{:?}", b.delivered_inc);
}

// =============================================================================================
// 2 — **THE REFUSAL IS THE CORRECTNESS.** Take it out and the reader returns a perfect agreement.
// =============================================================================================

/// **P1's SHIPPABLE DEFECT, DRIVEN — AND THE SHAPE OF THE WRONG ANSWER IS THE FINDING.**
///
/// § 5.29 (i): without the assert, a rung-69 reader on a rung-73 machine *writes `"inc"` into
/// `ref_law`, leaves `_ref` at `None`, falls through `_triple_rig`'s `self._ref or (…)` fallback
/// and marches a plant nobody asked for — **silently**.* § 5.28.5's lesson is that the silent
/// shape is the dangerous one, and this measures how silent it is.
///
/// It does not merely return a different number. **It returns the SAME number twice**: both arms
/// of a ledger whose entire content is the difference between two references land on identical
/// floats, because neither arm ever set a reference. A reader built to compare two things reports
/// a perfect agreement, having compared one thing with itself — which is `rung73.rs` gate 9's
/// finding (the first `_reference` applied reading B unconditionally and returned
/// `worst_delta_rest = 0.0`, a perfect confirmation from an instrument that had measured
/// nothing), arriving here in the PLUMBING rather than in a law.
///
/// So the assertion is two-sided: the refusal is gone (it returns) **and** what it returns is the
/// degenerate reading. A gate that only checked *it no longer panics* would be satisfied by any
/// change at all.
#[test]
fn deleting_the_refusal_makes_the_reader_return_the_same_number_twice() {
    let m = build(Inj::IntegrateFuel, &inc_arm());
    assert_installed(&m, Inj::IntegrateFuel, &[]);
    assert!(!fn_addr_eq(m.fuel.hooks.integrate_fuel, R73_FUEL.integrate_fuel));
    assert!(fn_addr_eq(m.fuel.hooks.integrate_fuel, R72_FUEL.integrate_fuel),
            "the injection is the PARENT's pointer, not a body written in this file");

    let b = reference_bill(&m, &flight(), &ramp(DS), SM, &rig_arm());
    assert_eq!(b.delivered.0.to_bits(), b.delivered.1.to_bits(),
               "both arms marched one plant: {:?}", b.delivered);
    assert_eq!(b.delivered_inc.0.to_bits(), b.delivered_inc.1.to_bits(), "{:?}", b.delivered_inc);
    assert_eq!(b.common_max_rel, 0.0);

    // and it is NOT the rung-69 machine's reading, which is the next gate's subject and is
    // asserted here from the other side so the two cannot be confused.
    let ctl = reference_bill(&ref69_of(&inc_arm()), &flight(), &ramp(DS), SM, &rig_arm());
    assert_ne!(fingerprint(&b), fingerprint(&ctl));
    assert_eq!(b.delivered.0.to_bits(), ctl.delivered.0.to_bits(),
               "the FIRST arm agrees — the loss is entirely in the second, which is the one the \
                unset reference was supposed to move");
}

/// **THE SHARPEST HALF: AT THIS SEAT, THE PORT PYTHON REFUSES IS THE ONE THAT WORKS PERFECTLY.**
///
/// § 5.29 (ix) P1 named two structurally different ports and probe K chose between them by
/// measurement (**0** reads of `_ref` across a full rung-73 reader run, against a liveness
/// control): *(1)* leave rung 69's slot alone and give rung 73 a second field — under which a
/// rung-69 reader on a rung-73 table writes `_ref` **correctly**, which is not what Python does;
/// or *(2)* re-aim the slot, and make the REFUSAL the guard, which is Python exactly.
///
/// **THIS GATE BUILDS THE HALF OF (1) THAT THE PAIRING CAN SEE, AND IT IS NOT ALL OF (1) — SAID
/// HERE BECAUSE THE FIRST WRITING OF THIS COMMENT CLAIMED IT WAS.** Option (1) leaves rung 69's
/// slot alone **and gives rung 73 a second, differently-named field**; this injection installs
/// the first half only. What the two share is the half that matters at this seat: under either,
/// a rung-69 reader on a rung-73 machine writes `_ref` and gets rung 69's answer.
///
/// And it gets it exactly. The result is not *nearly* right and not *differently* right —
/// **`reference_bill`'s whole reading is byte-identical to the one the rung-69 machine itself
/// produces**, the entire `Debug` fingerprint and every one of the five headline numbers by
/// `to_bits`. So rung 73's refusal is a DELIBERATE refusal rather than a safety net: at this seat
/// it stops a configuration that reproduces the parent's own answer.
///
/// **WHAT MUST NOT BE READ INTO THAT: that such a port would ship green.** Measured, on the
/// step-5 sweep's four binaries, this injection is caught by **24 of 59 gates** (6/10, 6/15,
/// 10/27, 2/7) — rung 73's own five readers reach `ref_law` through the SAME cell, so pointing it
/// at rung 69's body leaves every A-vs-B reader differencing the plant against itself, which is
/// [`the_reference_dispatch_is_live`]'s bug arriving through the table instead of the law. The
/// pairing cannot tell the two ports apart; nearly everything else can.
///
/// [`the_reference_dispatch_is_live`]: ../rung73/index.html
#[test]
fn rung_69s_own_setter_reproduces_rung_69s_reading_exactly_and_python_still_refuses_it() {
    let m = build(Inj::WithRef, &inc_arm());
    assert_installed(&m, Inj::WithRef, &["with_ref"]);
    assert!(!fn_addr_eq(R69_TRIPLE.with_ref, R73_TRIPLE.with_ref),
            "the two setters are different bodies — without this the gate compares a table with \
             itself");

    let got = reference_bill(&m, &flight(), &ramp(DS), SM, &rig_arm());
    let ctl = reference_bill(&ref69_of(&inc_arm()), &flight(), &ramp(DS), SM, &rig_arm());
    assert_eq!(fingerprint(&got), fingerprint(&ctl),
               "option (1) reproduces the rung-69 machine's ledger in full");
    for (a, c) in [(got.common_max_rel, ctl.common_max_rel),
                   (got.delivered.0, ctl.delivered.0), (got.delivered.1, ctl.delivered.1),
                   (got.delivered_inc.0, ctl.delivered_inc.0),
                   (got.delivered_inc.1, ctl.delivered_inc.1)] {
        assert_eq!(a.to_bits(), c.to_bits());
    }
    // and the reading is a real one — the fingerprint equality above would also be satisfied by
    // two dead readers agreeing on nothing.
    assert_ne!(ctl.delivered.0.to_bits(), ctl.delivered.1.to_bits());
    assert!(ctl.delivered.0 > 0.0 && ctl.delivered.1 > 0.0, "{:?}", ctl.delivered);
}

// =============================================================================================
// 3 — P4: `quad_gains_at`'s POINTER, on probe J's device
// =============================================================================================

/// One point of § 4's census: what the two bodies return at the SAME point, on the SAME machine.
struct Swap {
    used: usize,
    fuel_masked: usize,
    gov_masked: usize,
    /// moved / vanished PER POINT, deduplicated — a set, because a total can hide a distribution.
    moved_per_point: Vec<usize>,
    vanished_per_point: Vec<usize>,
    moved_names: Vec<&'static str>,
    vanished_names: Vec<&'static str>,
    /// `to_bits` against `==`, on the same population — § 4's finding.
    moved_bits: usize,
    moved_eq: usize,
    r72_diagonal_all_zero: bool,
    pair_fr_neg_zero_73: usize,
    pair_fr_neg_zero_72: usize,
    worst: (f64, &'static str),
    self_masked: Vec<f64>,
    cross_masked: Vec<f64>,
    self_live: Vec<f64>,
}

/// Every field of a [`QuadGains`] as an optional float, in the port's own declaration order.
///
/// `None` is Python's MISSING KEY and never a value — `shared_actuator`'s own doc comment says
/// why (*"the difference between absent and zero is the whole of § 5.29 (iv)'s 70 vanishing
/// keys"*), and § 4's discrete half is exactly this distinction.
///
/// **`f_f` AND `r_r` ARE NOT IN THAT SET, AND THE PORT SAYS SO.** Python's rung-72 dict carries
/// neither key, so its five-key difference is `F_f`, `R_r` and the three branch indicators; the
/// port declares `f_f`/`r_r` as plain `f64` fields that rung 72 writes as `0.0`. So of Python's
/// five, **three stay discrete here and two become value moves** — a REPRESENTATION difference,
/// disclosed rather than smoothed over, and gated from both sides below.
fn gains_fields(g: &QuadGains) -> [(&'static str, Option<f64>); 24] {
    [
        ("s", Some(g.s)), ("v_base", Some(g.v_base)),
        ("f_f", Some(g.f_f)), ("r_r", Some(g.r_r)), ("f_r", Some(g.f_r)), ("f_q", Some(g.f_q)),
        ("f_v", Some(g.f_v)), ("r_f", Some(g.r_f)), ("r_q", Some(g.r_q)), ("r_v", Some(g.r_v)),
        ("c_f", Some(g.c_f)), ("c_r", Some(g.c_r)), ("c_v", Some(g.c_v)),
        ("v_f", Some(g.v_f)), ("v_r", Some(g.v_r)), ("v_q", Some(g.v_q)),
        ("pair_fr", Some(g.pair_fr)), ("pair_rc", Some(g.pair_rc)),
        ("pair_cv", Some(g.pair_cv)), ("pair_rv", Some(g.pair_rv)),
        ("mask_leak", g.mask_leak),
        ("self_masked", g.self_masked), ("cross_masked", g.cross_masked),
        ("self_live", g.self_live),
    ]
}

/// **PROBE J's DEVICE: the machine held fixed at rung 73, ONLY the pointer swapped.**
///
/// Never probe H's — a rung-72 reader on a rung-73 machine dispatches to rung 73's body 14 times
/// against a rung-72 machine's 17, and those are two different TRAJECTORIES, so nothing in the
/// comparison is a comparison of two bodies. Here both calls take the same `&FuelPoint` off the
/// same march, under the same `RefScope`, and differ in the function pointer alone.
fn swap_census(inc: bool) -> Swap {
    let core = applied_of(&valve_arm());
    let (m, surge, _lag, traj) = shared_march(
        &core, &flight(), LO, HI, TT4_MAX, SM, CLOCK, R, SETTLE, AG_DS, V_MAX, inc);
    let pts = riding4(&traj, valve().b_max);
    let sampled: Vec<&FuelPoint> = pts.iter().step_by(AG_EVERY).collect();
    let mut s = Swap {
        used: 0, fuel_masked: 0, gov_masked: 0, moved_per_point: Vec::new(),
        vanished_per_point: Vec::new(), moved_names: Vec::new(), vanished_names: Vec::new(),
        moved_bits: 0, moved_eq: 0, r72_diagonal_all_zero: true, pair_fr_neg_zero_73: 0,
        pair_fr_neg_zero_72: 0, worst: (-1.0, ""), self_masked: Vec::new(),
        cross_masked: Vec::new(), self_live: Vec::new(),
    };
    for p in sampled.iter() {
        let _rs = RefScope::set(&m.fuel.inner, Some(REF_LAW_APPLIED));
        // THE SAME point, the SAME machine, the SAME scope, the SAME eleven arguments — only the
        // function pointer differs. Spelled twice rather than through a helper taking the pointer,
        // so the two call sites are visibly identical instead of provably so.
        let a73 = (R73_TRIPLE.quad_gains_at)(
            &m, &flight(), p, None, surge.as_ref(), TT4_MAX, 1e-7, 1e-5, 1e-4, true, 4.0);
        let b72 = (R72_TRIPLE.quad_gains_at)(
            &m, &flight(), p, None, surge.as_ref(), TT4_MAX, 1e-7, 1e-5, 1e-4, true, 4.0);
        let (a, b) = match (a73, b72) { (Ok(a), Ok(b)) => (a, b), _ => continue };
        if !a.interior || !b.interior { continue; }
        s.used += 1;
        if a.masked == Some(Authority::Fuel) { s.fuel_masked += 1; } else { s.gov_masked += 1; }
        if b.f_f != 0.0 || b.r_r != 0.0 { s.r72_diagonal_all_zero = false; }
        if a.pair_fr.to_bits() == (-0.0f64).to_bits() { s.pair_fr_neg_zero_73 += 1; }
        if b.pair_fr.to_bits() == (-0.0f64).to_bits() { s.pair_fr_neg_zero_72 += 1; }
        s.self_masked.push(a.self_masked.expect("an interior row has a masked leg"));
        s.cross_masked.push(a.cross_masked.expect("an interior row has a masked leg"));
        s.self_live.push(a.self_live.expect("an interior row has a masked leg"));
        let (mut mv, mut vn) = (0usize, 0usize);
        for ((k, va), (_, vb)) in gains_fields(&a).iter().zip(gains_fields(&b).iter()) {
            match (va, vb) {
                (Some(x), Some(y)) => {
                    if x.to_bits() != y.to_bits() {
                        mv += 1;
                        s.moved_bits += 1;
                        if !s.moved_names.contains(k) { s.moved_names.push(k); }
                        let d = (x - y).abs();
                        if d > s.worst.0 { s.worst = (d, k); }
                    }
                    if x != y { s.moved_eq += 1; }
                }
                (Some(_), None) => {
                    vn += 1;
                    if !s.vanished_names.contains(k) { s.vanished_names.push(k); }
                }
                (None, Some(_)) => panic!("rung 72 writes a key rung 73 does not: {k}"),
                (None, None) => {}
            }
        }
        s.moved_per_point.push(mv);
        s.vanished_per_point.push(vn);
    }
    s.moved_names.sort_unstable();
    s.vanished_names.sort_unstable();
    s
}

fn distinct(v: &[usize]) -> Vec<usize> {
    let mut d = v.to_vec();
    d.sort_unstable();
    d.dedup();
    d
}

/// **P4's DISCRETE HALF, WHICH IS THE STRONGER ONE — a key that is ABSENT cannot be passed by a
/// one-sided bar.**
///
/// § 5.29 (iv) measured 70 shipped-only keys and § 5.29.2 (b) re-measured the same fact as five
/// keys over 101 points; **neither number transfers**, for the reason § 5.29.4 (b) (ii) records
/// (a row measured at one stride read off at another). This gate takes the census on the reader's
/// own grid and states what it finds: **exactly three keys vanish at every usable point**, and
/// they are the three the port declares `Option` — the two the port turns into `0.0` fields are
/// asserted from the other side, so the representation difference is pinned rather than lost.
#[test]
fn the_gains_pointer_makes_three_keys_vanish_at_every_point() {
    for inc in [false, true] {
        let s = swap_census(inc);
        assert!(s.used > 25, "inc={inc}: an empty census would pass every bar below: {}", s.used);
        assert_eq!(s.fuel_masked + s.gov_masked, s.used);
        assert_eq!(distinct(&s.vanished_per_point), vec![3],
                   "inc={inc}: three, at EVERY point — a total could hide a distribution");
        assert_eq!(s.vanished_names, vec!["cross_masked", "self_live", "self_masked"]);
        assert_eq!(s.vanished_per_point.iter().sum::<usize>(), 3 * s.used);
        // THE OTHER TWO OF PYTHON'S FIVE, from the side the port puts them on.
        assert!(s.r72_diagonal_all_zero,
                "inc={inc}: the port writes rung 72's absent `F_f`/`R_r` as EXACTLY 0.0, which is \
                 what makes them value moves here and discrete keys in Python");
        assert!(s.moved_names.contains(&"f_f") || s.moved_names.contains(&"r_r"),
                "inc={inc}: and at least one of them MOVES, or the representation difference \
                 would be unobservable: {:?}", s.moved_names);
    }
}

/// **A THIRD OF THIS BREAK IS A SIGN BIT, AND AN `==` DETECTOR SCORES TWO THIRDS OF IT.**
///
/// `pair_fr` is `f_r * r_f`. Under reading B exactly one of the two cross terms is `-1` at any
/// point and the other is an exact `0`, so the product is **`-0.0`**; under rung 72 both are
/// `+0.0` and the product is `+0.0`. The two are equal under `==` and differ in one bit.
///
/// Measured on this grid: `pair_fr` is `-0.0` at **every** point under rung 73 and at **none**
/// under rung 72, and the census scores **three** moves per point by `to_bits` against **two** by
/// `==` — a detector written with `==` returns two thirds of the break and still reads as a
/// confirmation.
///
/// **THIS IS THE MECHANISM UNDER TWO NUMBERS THAT WERE ALREADY WRITTEN DOWN.** Step 2 § (e)'s M22
/// measured 101 keys exactly `-0.0` in the 5 066-key dump and *every one of them a
/// `*.g.pair_FR`*; step 4 § (d) re-measured the same 101 on a grid fifteen times wider and
/// reported the count unchanged. Both recorded WHERE the negative zeros are. This says why they
/// are there, and that they are the observable.
#[test]
fn a_third_of_the_gains_break_is_a_sign_bit() {
    for inc in [false, true] {
        let s = swap_census(inc);
        assert_eq!(s.pair_fr_neg_zero_73, s.used, "inc={inc}: `-0.0` at every point under rung 73");
        assert_eq!(s.pair_fr_neg_zero_72, 0, "inc={inc}: and at none under rung 72");
        assert_eq!(distinct(&s.moved_per_point), vec![3], "inc={inc}");
        assert_eq!(s.moved_bits, 3 * s.used);
        assert_eq!(s.moved_eq, 2 * s.used,
                   "inc={inc}: `==` cannot see the sign bit — {} against {}", s.moved_eq,
                   s.moved_bits);
        assert!(s.moved_names.contains(&"pair_fr"));
        // and the census's own instrument is proved able to separate the two zeros
        assert_ne!((-0.0f64).to_bits(), 0.0f64.to_bits());
        assert!((-0.0f64) == 0.0f64, "which is exactly why the bar above is on the bits");
    }
}

/// **THE VALUE WITNESS: the two branch indicators are `+1` and `-1`, the third is EXACTLY 0, and
/// the parent's body has none of them.**
///
/// § 5.29 (iv)'s witness is `rows[*].gains.F_r` moving `-1.000000000002735 -> 0.0`, and that
/// magnitude reproduces here to the last digit. **Which NAME carries it does not transfer, and
/// the first writing of this gate asserted that it did.**
///
/// Reading B puts `+1` on the masked leg's own diagonal and `-1` on its cross-gain, so at a
/// Fuel-masked point the pair is (`f_f`, `f_r`) and at a Gov-masked point it is (`r_r`, `r_f`) —
/// which leg is masked is a property of the march, and on the `inc = true` arm every sampled
/// point is Gov-masked. **The two are the same magnitude to within 1e-15**, so *the largest move*
/// is a tie broken by the last bits and is not a discriminating property at all: the census
/// measured `r_f` on one arm and `r_r` on the other, and the gate as first written demanded a
/// cross term on both. It is now stated as what was measured — a SELF term and a CROSS term, each
/// at unit magnitude, and the worst is one of those four.
///
/// `self_live` is the one that must be EXACT: § 1's *the holding leg's applied reference IS the
/// scheduled one* is `== 0.0` and never `< tol`, and `tests/test_rung73.py`'s own docstring gives
/// the reason two paragraphs above its assertions — *an exact zero survives a difference
/// quotient; an exact one does not*.
#[test]
fn the_gains_pointer_moves_the_branch_indicators_off_the_parents_zeros() {
    for inc in [false, true] {
        let s = swap_census(inc);
        for x in &s.self_masked { assert!((x - 1.0).abs() < 1e-11, "inc={inc}: {x}"); }
        for x in &s.cross_masked { assert!((x + 1.0).abs() < 1e-11, "inc={inc}: {x}"); }
        for x in &s.self_live { assert_eq!(*x, 0.0, "inc={inc}: EXACT, not small"); }
        assert!(s.worst.0 > 0.99 && s.worst.0 < 1.01, "inc={inc}: {:?}", s.worst);
        // ONE self term and ONE cross term move at every point, and the worst is one of them.
        let self_t = ["f_f", "r_r"];
        let cross_t = ["f_r", "r_f"];
        assert!(s.moved_names.iter().any(|k| self_t.contains(k)), "inc={inc}: {:?}",
                s.moved_names);
        assert!(s.moved_names.iter().any(|k| cross_t.contains(k)), "inc={inc}: {:?}",
                s.moved_names);
        assert!(self_t.contains(&s.worst.1) || cross_t.contains(&s.worst.1),
                "inc={inc}: the largest move is a diagonal or a cross term — {:?}", s.worst);
        // and the TIE is the point: the two differ by less than a part in 1e12, which is why the
        // name that wins a `>` is not something a gate may pin.
        let (hi_s, hi_c) = (max_abs(&s.self_masked), max_abs(&s.cross_masked));
        assert!((hi_s - hi_c).abs() < 1e-11, "inc={inc}: {hi_s} {hi_c}");
    }
}

fn max_abs(v: &[f64]) -> f64 { v.iter().fold(0.0f64, |a, x| a.max(x.abs())) }

// =============================================================================================
// 4 — THE SEAT MATRIX, RUN WHOLE
// =============================================================================================

fn fingerprint<T: std::fmt::Debug>(x: &T) -> String { format!("{x:?}") }

/// What a seat did. **`Refused` is separated from `Panicked` by the MESSAGE**: the baseline
/// already refuses at the `pairing69` seat, so an injected run that panics identically has told
/// us nothing, and one that panics differently has.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Seen { Read(String), Broke(String) }

const SEATS: [&str; 7] = ["march", "pairing69", "handover_law", "applied_gains", "applied_cells",
                          "ref_discriminator", "applied_bill"];

fn run_seat(name: &str, m: &ScheduledStatorCore, clock: (f64, f64, f64, f64)) -> Seen {
    quiet_hook();
    QUIET.with(|q| q.set(true));
    let out = catch_unwind(AssertUnwindSafe(|| match name {
        "march" => fingerprint(&shared_march(m, &flight(), LO, HI, TT4_MAX, SM, clock, R, SETTLE,
                                             MX_DS, V_MAX, false).3
            .iter().map(|p| (p.s.to_bits(), p.tt4.to_bits(), p.mf.to_bits())).collect::<Vec<_>>()),
        "pairing69" => fingerprint(&reference_bill(m, &flight(), &ramp(MX_DS), SM, &rig_arm())),
        "handover_law" => fingerprint(&handover_law(m, &flight(), LO, HI, TT4_MAX, SM, &[clock],
                                                    R, SETTLE, MX_DS, V_MAX)),
        "applied_gains" => fingerprint(&applied_gains(m, &flight(), LO, HI, TT4_MAX, SM, clock,
                                                      false, R, SETTLE, MX_DS, V_MAX, MX_EVERY)),
        "applied_cells" => fingerprint(&applied_cells(m, &flight(), LO, HI, TT4_MAX, SM, &[clock],
                                                      R, SETTLE, MX_DS, V_MAX, MX_EVERY)),
        "ref_discriminator" => fingerprint(&ref_discriminator(m, &flight(), LO, HI, TT4_MAX, SM,
                                                              clock, false, R, SETTLE, MX_DS,
                                                              V_MAX, MX_EVERY)),
        "applied_bill" => fingerprint(&applied_bill(m, &flight(), LO, HI, TT4_MAX, SM, clock,
                                                    false, R, SETTLE, MX_DS, V_MAX)),
        _ => unreachable!("SEATS is the only source of these names"),
    }));
    QUIET.with(|q| q.set(false));
    match out {
        Ok(s) => Seen::Read(s),
        Err(e) => Seen::Broke(e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()),
    }
}

/// One cell of the matrix, as printed.
fn verdict(base: &Seen, got: &Seen) -> &'static str {
    match (base, got) {
        (Seen::Read(a), Seen::Read(b)) if a == b => "same",
        (Seen::Read(_), Seen::Read(_)) => "DIFF",
        (Seen::Read(_), Seen::Broke(_)) => "BROKE",
        (Seen::Broke(_), Seen::Read(_)) => "RETURNS",
        (Seen::Broke(a), Seen::Broke(b)) if a == b => "refused",
        (Seen::Broke(_), Seen::Broke(_)) => "BROKE-OTHER",
    }
}

/// **SEVEN POINTERS x SEVEN SEATS, AND THE ELEVEN SILENCES ARE NOT ONE PHENOMENON.**
///
/// AD step 6's rule. A parent-pointer injection that produces no observable at a seat is either
/// laundered (the cell ran, on a machine rebuilt around the shipped tables) or on a path that
/// never reaches the cell — and *nothing moved* is the same reading for both. What separates them
/// is the OTHER seats in the same ROW: every injection below is proved live somewhere, so each of
/// its silences is demonstrably a property of the PATH and not of an injection that did not take.
///
/// The row worth reading twice is **`quad_gains_at`**: it goes loud at `applied_gains`, moves the
/// reading at `applied_cells` and `ref_discriminator`, and is invisible at `handover_law`,
/// `applied_bill` and a bare march. That is the source's own call-site census — three readers
/// dispatch the cell and two do not — **recovered by running the matrix, not by grepping for the
/// slot.**
///
/// And the three rows that are silent at every value seat are silent for three different reasons,
/// which is why they are not collapsed: `rk4_floor_shared` differs only in a MESSAGE (§ 6),
/// `shared_rig`'s carry is REDUNDANT because `at_lever` carries first (step 1 § (d), step 2
/// § (e)), and `integrate_fuel` is a REFUSAL that no value seat is arranged to trip — its one
/// observable is the `pairing69` column, which is § 2.
#[test]
fn the_seat_matrix() {
    let base: Vec<Seen> = SEATS.iter()
        .map(|s| run_seat(s, &build(Inj::None, &inc_arm()), CLOCK)).collect();
    assert!(matches!(base[0], Seen::Read(_)) && matches!(base[6], Seen::Read(_)),
            "the baseline's value seats must READ, or every verdict below is against a broken \
             fixture");
    assert!(matches!(base[1], Seen::Broke(_)),
            "and the baseline's `pairing69` seat must REFUSE — that is § 1");

    let mut tally: Vec<(Inj, Vec<&'static str>)> = Vec::new();
    for inj in INJS.iter().skip(1) {
        let m = build(*inj, &inc_arm());
        // THE INSTALL PROOF, BEFORE THE ROW IS READ. Two of the seven rows below are all-silent,
        // and for those this is the ONLY evidence the injection took at all.
        match *inj {
            // `at_lever` re-aimed at the parent is *stop carrying this rung's tables*, so its
            // sibling is a RUNG-72 machine by construction and there is no rung-73 slot to check.
            Inj::AtLever => {
                assert!(fn_addr_eq(m.fuel.inner.lever_hooks.at_lever, R72.at_lever));
                assert!(!fn_addr_eq(m.fuel.inner.lever_hooks.at_lever, L_NONE.at_lever));
                let sib = m.at_lever(&valve_arm());
                assert_eq!(triple_diff(sib.triple_hooks(), &R72_TRIPLE), Vec::<&str>::new(),
                           "rung 72's sibling constructor installs rung 72's table, which is the \
                            whole of this injection");
                assert_eq!(triple_diff(sib.triple_hooks(), &R73_TRIPLE),
                           vec!["with_ref", "reference", "rk4_floor_shared", "shared_rig",
                                "quad_gains_at"],
                           "and it differs from rung 73's in exactly this slice's five triple \
                            cells — the census, read off the rebuild");
            }
            Inj::WithRef => assert_installed(&m, *inj, &["with_ref"]),
            Inj::Reference => assert_installed(&m, *inj, &["reference"]),
            Inj::Rk4Floor => assert_installed(&m, *inj, &["rk4_floor_shared"]),
            Inj::SharedRig => assert_installed(&m, *inj, &["shared_rig"]),
            Inj::QuadGains => assert_installed(&m, *inj, &["quad_gains_at"]),
            Inj::IntegrateFuel => assert_installed(&m, *inj, &[]),
            Inj::None => unreachable!("skipped"),
        }
        let row: Vec<&'static str> = SEATS.iter().zip(base.iter())
            .map(|(s, b)| verdict(b, &run_seat(s, &m, CLOCK))).collect();
        println!("{inj:?}: {row:?}");
        tally.push((*inj, row));
    }
    let row_of = |i: Inj| tally.iter().find(|(k, _)| *k == i).expect("every injection ran").1
        .clone();

    // WHICH INJECTIONS ARE LIVE AT A SEAT AT ALL — a MEASURED PARTITION, not a universal.
    //
    // AD step 6's file could assert that every injection breaks somewhere, and that behavioural
    // control is what made its silences readable. **It is not available here, and the first
    // writing of this gate asserted it anyway** — with the two counterexamples already sitting in
    // this step's own measurement table. `rk4_floor_shared` and `shared_rig` are live at NONE of
    // the seven seats, and each has a gate of its own saying why: the floor differs only in its
    // MESSAGE (`the_floor_swap_is_a_message_and_not_a_value`) and the rig's carry is REDUNDANT
    // because `at_lever` carries the law first (`the_rig_carry_…`, and step 2 § (d)'s M11 vs
    // M11b).
    //
    // So for those two the *did the injection take* evidence is STRUCTURAL and nothing else:
    // `assert_installed` above, which is a pointer identity on the rebuilt sibling. That is worth
    // stating rather than hiding behind a passing universal — **the two pointers with no
    // behavioural control are exactly the two that would most benefit from one.**
    let live: Vec<Inj> = tally.iter()
        .filter(|(_, row)| row.iter().any(|v| *v != "same" && *v != "refused"))
        .map(|(i, _)| *i).collect();
    assert_eq!(live, vec![Inj::AtLever, Inj::WithRef, Inj::Reference, Inj::IntegrateFuel,
                          Inj::QuadGains],
               "five of the seven re-aimed pointers are observable at some seat; the other two \
                are `rk4_floor_shared` and `shared_rig`, and § 6 owns both");

    // `quad_gains_at` — the call-site census, recovered from the seats.
    let q = row_of(Inj::QuadGains);
    assert_eq!(q, vec!["same", "refused", "same", "BROKE", "DIFF", "DIFF", "same"],
               "three readers dispatch the gains cell and two do not");

    // the three that no VALUE seat can see, each for its own reason
    assert_eq!(row_of(Inj::Rk4Floor),
               vec!["same", "refused", "same", "same", "same", "same", "same"]);
    assert_eq!(row_of(Inj::SharedRig), row_of(Inj::Rk4Floor),
               "two different reasons, one reading — § 6 says which is which");
    let f = row_of(Inj::IntegrateFuel);
    assert_eq!(f[1], "RETURNS", "the refusal's ONLY observable is the manufactured pairing");
    assert!(f.iter().enumerate().all(|(i, v)| i == 1 || *v == "same"), "{f:?}");

    // and the two whose parent pointer changes the PLANT
    assert_eq!(row_of(Inj::Reference)[0], "DIFF", "the reference is in the march itself");
    assert_eq!(row_of(Inj::AtLever)[1], "RETURNS",
               "rung 72's sibling constructor hands back a rung-72 machine, so the rung-73 \
                refusal is not on it either");
    assert_eq!(row_of(Inj::WithRef)[1], "RETURNS", "§ 3");
}

/// **THE MATRIX's INSTRUMENT, PROVED ABLE TO SEE BEFORE IT IS READ.**
///
/// Every `same` in § 5 is a string equality between two `Debug` fingerprints. A fingerprint that
/// was constant — a reader that returned an empty struct, a seat wired to the wrong machine —
/// would make the whole matrix read `same` and look like a clean result. So the plant is
/// perturbed (one clock arm moved, nothing else) and **all seven seats are required to move**.
///
/// `pairing69` refuses on both plants, so its fingerprint is a MESSAGE; it is checked for being
/// non-empty and carrying the rung tag instead, which is the strongest thing a refusing seat can
/// offer and is stated rather than skipped.
#[test]
fn the_matrix_instrument_moves_when_the_plant_does() {
    let m = build(Inj::None, &inc_arm());
    let other = (0.20, 0.01, 0.50, 0.05);
    assert_ne!(CLOCK, other);
    for (i, s) in SEATS.iter().enumerate() {
        let a = run_seat(s, &m, CLOCK);
        let b = run_seat(s, &m, other);
        match (&a, &b) {
            (Seen::Read(x), Seen::Read(y)) => {
                assert!(!x.is_empty() && x != y, "seat {s} did not move with the plant");
            }
            (Seen::Broke(x), Seen::Broke(y)) => {
                assert_eq!(i, 1, "only `pairing69` refuses on both plants");
                assert!(x.contains("rung-73") && y.contains("rung-73"), "{x:?} {y:?}");
            }
            _ => panic!("seat {s} changed KIND under a clock move: {a:?} / {b:?}"),
        }
    }
}

// =============================================================================================
// 5 — THE TWO POINTERS WITH NO VALUE BREAK, RECORDED RATHER THAN HUNTED
// =============================================================================================

/// **`shared_rig`'s CARRY IS RUNG 73's OWN POINTER AND HAS NO DISCRIMINATOR — PRE-REGISTERED
/// TWICE, AND NOT RE-OPENED HERE.**
///
/// Step 1 § (d) drove the override and rung 72's body on the same receiver under both laws
/// (probe L2) and found them equal; mutation M11 then deleted the carry and survived all fifteen
/// of step 1's gates. Step 2 § (d) re-ran it on a grid those fifteen assertions could not reach —
/// **0 of 5 066 keys move** — and, crucially, ran the SECOND arm: with BOTH carries deleted
/// (M11b) 122 keys move and a gate fires. So the docstring's claim is true of the PAIR and false
/// of the member: `at_lever` carries the law first, and this copy is **redundant rather than
/// inert**.
///
/// That is why § 5's `SharedRig` row is all-`same` and why this gate asserts the POINTER instead
/// of hunting a value break. Hunting one would be the shape step 2 named: a zero on the value
/// seat is a property of `at_lever`, not evidence that the defence is unnecessary.
///
/// The port keeps the duplication because the source makes it
/// ([[rust-port-copy-vs-rederivation]]), and the outcome — the rig's machine carries the law — is
/// what `slice_ae_cells.rs`'s `at_lever_and_the_rig_both_carry_the_reference` asserts. That gate
/// is a **one-sided detector wearing a two-sided name**: it survives M11 and dies on M10 and
/// M11b. Named here so a later reader does not mistake its green for coverage of this pointer.
#[test]
fn the_rig_carry_is_rung_73s_own_pointer_and_its_zero_is_measured_not_inferred() {
    assert!(!fn_addr_eq(R73_TRIPLE.shared_rig, R72_TRIPLE.shared_rig),
            "rung 73 re-aims the rig, and the injection in § 5 is therefore a real swap");
    assert_eq!(triple_diff(&T_SHARED_RIG, &R73_TRIPLE), vec!["shared_rig"]);

    // THE REDUNDANCY ITSELF, as a SAME-RUN difference rather than as a zero left over from a
    // mutation. Step 2 § (d) established it by deleting the carry and getting 0 of 5 066 keys;
    // that is an absence. Here both pointers are driven on one receiver under both laws, and the
    // PARENT's rig is shown to carry the law as well — which is WHY the swap has no observable.
    //
    // **ON THE SHIPPED MACHINE, DELIBERATELY, AND NOT ON THIS FILE's OWN.** `injection!`'s
    // `at_lever` copies the law itself (it must, or the matrix would measure the class default),
    // so a machine built by `build` would have this file's fixture standing in for exactly the
    // shipped statement under measurement. That substitution is real and is why the step-5 sweep
    // scores this file 0 of 10 on a mutation that deletes `r73_at_lever`'s carry; it is named
    // here, at its cause, and routed around by using `applied_of`.
    let m = applied_of(&inc_arm());
    assert!(fn_addr_eq(m.fuel.inner.lever_hooks.at_lever, R73.at_lever),
            "the SHIPPED sibling constructor, not this file's");
    let arm = SharedRigArm { sm: SM, tt4_max: TT4_MAX, inc: true, tau: TAU, tau_s: TAU_S,
                             v_max: V_MAX, ..Default::default() };
    assert_ne!(REF_LAWS_DECLARED[0], REF_LAW_APPLIED,
               "the loop below is two readings only if the two laws differ");
    for law in REF_LAWS_DECLARED {
        let _r = RefScope::set(&m.fuel.inner, Some(law));
        let (a, _, _) = (R73_TRIPLE.shared_rig)(&m, &arm);
        let (b, _, _) = (R72_TRIPLE.shared_rig)(&m, &arm);
        assert_eq!(a.fuel.inner.ref_law.get(), law, "rung 73's rig carries the scoped law");
        assert_eq!(b.fuel.inner.ref_law.get(), law,
                   "and so does the PARENT's, because it reaches its sibling through \
                    `self.at_lever(..)` — which at rung 73 has already copied it. That is why \
                    this pointer has no value break, and it is a measurement rather than an \
                    inference from a zero.");
    }
    assert_eq!(m.fuel.inner.ref_law.get(), REF_LAW_APPLIED, "and every scope restored");
}

/// **THE FLOOR SWAP IS A MESSAGE AND NOT A VALUE, AND THE CONDITION IS CHARACTER FOR CHARACTER
/// THE PARENT's.**
///
/// `r73_rk4_floor_shared`'s own doc comment says it: *the condition is `ds * rate <= 2.0` in rungs
/// 72, 73 and 74 character for character, so the MESSAGE is the entire cell.* § 5's `Rk4Floor`
/// row is all-`same` for exactly that reason and not because the injection failed to take — which
/// is the distinction the matrix exists to make, and which this gate closes by driving the two
/// bodies past the boundary and reading what each says.
///
/// The needle is `rung-73` / `origin`, which § 5.29 (vii) measured unique to this class over all
/// 58 ladder classes with the names EMITTED. The shipped Python needle `"FOUR actuator states"`
/// is in NINE classes back to rung 43 and is deliberately not read.
#[test]
fn the_floor_swap_is_a_message_and_not_a_value() {
    // The two bodies agree on where the boundary IS, and the `<=` is driven AT it. The arguments
    // are halves and small integers so that `ds * rate` is exact in binary and the reader can see
    // that `0.5 * 4.0` really is `2.0` — a decimal pair whose product only happens to round onto
    // the boundary would make the inclusive case a coincidence rather than a test of it.
    assert_eq!(0.5f64 * 4.0, 2.0);
    for (ds, rate) in [(0.5, 3.5), (0.5, 4.0)] {
        assert!(caught(|| (R73_TRIPLE.rk4_floor_shared)(ds, rate)).is_none());
        assert!(caught(|| (R72_TRIPLE.rk4_floor_shared)(ds, rate)).is_none());
    }
    // ... and differ in what they say past it.
    let a = caught(|| (R73_TRIPLE.rk4_floor_shared)(0.5, 4.5)).expect("past the boundary");
    let b = caught(|| (R72_TRIPLE.rk4_floor_shared)(0.5, 4.5)).expect("past the boundary");
    assert!(a.contains("rung-73") && a.contains("origin"), "{a:?}");
    assert!(b.contains("rung-72") && !b.contains("rung-73"), "{b:?}");
    assert_ne!(a, b);
    // the needle this file refuses to use, and the reason, asserted rather than asserted-about
    assert!(a.contains("FOUR actuator states") && b.contains("FOUR actuator states"),
            "the shipped Python needle is in BOTH messages, which is why it discriminates \
             nothing and why the gate above reads the rung tag");
}
