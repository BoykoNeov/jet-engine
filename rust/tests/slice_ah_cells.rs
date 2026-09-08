//! SLICE AH step 1 — **FOUR RE-AIMED POINTERS ACROSS TWO RUNGS, ZERO NEW TABLE FIELDS, AND A CELL
//! WHOSE ONLY CONTENT IS THE TABLE IT POINTS AT.**
//!
//! # WHY THIS FILE IS SHAPED THE WAY IT IS
//!
//! Plan § 5.32 (ii)'s census is `0 ADD` — the fifth slice running, and probe 7 makes it durable
//! past rungs 79–84 rather than merely correct today. So, as at slice AG, **a forgotten re-aim
//! compiles**: there is no `E0063` width tripwire anywhere in this slice. The machine silently
//! runs the parent, returns this slice's own reduce-arm answer, and every reduce gate in the crate
//! goes on passing. The instrument that works under that condition is FUNCTION-POINTER IDENTITY
//! IN BOTH DIRECTIONS — every swapped slot must DIFFER from the parent's and every inherited slot
//! must EQUAL it — and [`the_slice_re_aims_four_pointers_and_adds_none`] is it.
//!
//! # AND AT THIS SLICE THE INSTRUMENT IS NOT MERELY THE BEST ONE, IT IS THE ONLY ONE
//!
//! Probe 8 measured what each `at_lever` in the chain copies forward, and rung 77's is the FIRST
//! in the entire chain that carries **nothing new**: its field list is identical to rung 76's, and
//! its only difference is the class it constructs. In this port the class **is** the hooks table.
//! So a defect at that cell cannot move a copied field — the six values are the same either way —
//! and can only be a wrong table: **loud to pointer identity, silent to every value gate.**
//! [`at_lever_differs_from_its_parent_in_the_TABLE_and_in_nothing_else`] states that as a
//! measurement rather than an assertion, by showing the six carried knobs agree and the tables do
//! not.
//!
//! # THE PROFILE THIS FILE RUNS IN IS PART OF ITS POWER — **MEASURED, AND THE FIRST WRITING OF
//! THIS PARAGRAPH HAD IT BACKWARDS**
//!
//! Identical-code folding is what makes a pass-through body indistinguishable from its parent, and
//! the linker does more of it with optimisations on. So the profile a pointer-identity gate runs
//! in decides how much it can see, and this file was drafted saying `cargo test` runs it in the
//! **dev** profile — that a folding defect could hide under `--release` and be invisible here.
//!
//! **That is false, and `Cargo.toml` says so in two lines:** this crate sets `[profile.test]
//! opt-level = 2`, and the build line reads *Finished `test` profile [optimized + debuginfo]*.
//! The gate therefore runs in the folding-FRIENDLY profile, not the folding-shy one — it is at its
//! STRONGEST here, and a body that survives it as distinct is distinct under optimisation.
//!
//! Recorded rather than quietly corrected, because the error had the sign that flatters the gate's
//! author: it would have excused a future miss as a profile artefact. **A claim about the
//! instrument's own power is a measurement, not a default** — one `grep` of `Cargo.toml` settles
//! it, and slice AG measured the live form of the underlying defect at `r76_shared_rig`.
//!
//! # WHAT THIS FILE DELIBERATELY DOES NOT GATE
//!
//! **The gauge's own numbers.** `gauge_scan`, `root_census`, `gauge_vs_device` and `gauge_march`
//! are readers and land later in the slice; what is gated here is that the knob's DEFAULT is the
//! identity and that the identity arm dispatches to the parent.
//!
//! **Which restore semantics the frozen-state guard uses.** Plan § 5.32 (i): the two are
//! value-identical at every reachable site, so no gate in this port can assert the choice. The
//! claim is a doc claim, in [`residual_gauge`](turbojet::residual_gauge)'s header, with the six
//! measured nest sites and their dead-window criterion beside it. **A booking no gate can close
//! is not a gate nobody wrote — it is a claim in the wrong currency**, and writing a gate here
//! that measured something else would be an instrument fed by what it certifies.
//!
//! That header also records what step 1 measured against the pre-flight's own proposal: the
//! crate-wide repair § 5.32 (i) called *free* is value-invisible below rung 77 (0 nests in
//! 10 038 348 sets over rungs 70–76) and **blocked anyway**, because
//! `slice_y_dispatch.rs` already manufactures the nest and pins the opposite policy. Nothing in
//! this file re-states that; the two are kept in step deliberately, so the claim has ONE home.
//!
//! **THAT A MISSED RE-AIM IS INVISIBLE TO VALUES — asserted in this header, NOT demonstrated in
//! this file.** Slice AG step 1 proved its identity gate load-bearing by building the defect: a
//! counterfeit table with one cell left at the parent's body, marched, and measured silent. That
//! needs a cascade built on a table chosen by the test, and
//! `reference_split::build_split_family_cascade` is `pub(crate)` — unreachable from an integration
//! test. So this file asserts the four swaps and does **not** yet show that a dropped one would
//! pass everything else. Closing it needs a test-visible builder (or an in-crate `#[cfg(test)]`
//! module) and lands with the slice's dispatch gates; it is written here rather than left as an
//! absence for a later reader to discover, which is § 5.31.6's *what is the census DECLINING to
//! check?* asked of this file about itself.
//!
//! **NOTHING HERE READS A GOLDEN.** Every assertion is a pointer comparison, a same-run
//! difference, or a compile-time property.

use std::ptr::fn_addr_eq;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::residual_gauge::{
    build_residual_gauge_cascade, gauge_counters, reset_gauge_counters, GAUGE_K_IDENTITY, R78,
    R78_FUEL, R78_STATOR, R78_TRIPLE, R78_TWO,
};
use turbojet::sensed_cap::{build_sensed_cap_cascade, R76, R76_FUEL, R76_STATOR, R76_TRIPLE, R76_TWO};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::stiffness_ledger::{
    build_stiffness_ledger_cascade, R77, R77_FUEL, R77_STATOR, R77_TRIPLE, R77_TWO,
};
use turbojet::three_loop::TripleHooks;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung77.py` and `tests/test_rung78.py`'s module constants, which are rung 76's
// unchanged. **This slice adds no number of its own** — rung 77 declares no knob at all and rung
// 78's `k` is swept, never set.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const B: f64 = 0.10;

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

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// Python's `SensedCapTransient(design, …)` — rung 76, the parent and the control.
fn sensed(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_sensed_cap_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Python's `StiffnessLedgerTransient(design, …)` — rung 77.
fn ledger(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_stiffness_ledger_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// Python's `ResidualGaugeTransient(design, …)` — rung 78.
fn gauge(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_residual_gauge_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

fn valve_arm() -> LeverArm { LeverArm { bleed_lim: Some(valve()), ..Default::default() } }

// ---------------------------------------------------------------------------------------------

/// **THE STEP's INSTRUMENT** — four swaps that must differ, and every other cell that must not.
#[test]
fn the_slice_re_aims_four_pointers_and_adds_none() {
    // RUNG 77's ONE, against rung 76.
    assert!(!fn_addr_eq(R77.at_lever, R76.at_lever),
            "swap 1 of 4 — `at_lever`, FIFTEENTH instance of the sibling-constructor trap and the \
             FIRST in the chain that carries nothing new. The whole cell is the table.");

    // RUNG 78's THREE, against rung 77.
    assert!(!fn_addr_eq(R78.at_lever, R77.at_lever),
            "swap 2 of 4 — `at_lever`, SIXTEENTH, and the seventh knob is `_gauge_k`");
    assert!(!fn_addr_eq(R78_TRIPLE.shared_rig, R77_TRIPLE.shared_rig),
            "swap 3 of 4 — `_shared_rig`, a SIXTH knob on the same leak");
    assert!(!fn_addr_eq(R78_TRIPLE.cap_fuel, R77_TRIPLE.cap_fuel),
            "swap 4 of 4 — `_cap_fuel`, the gauged accel branch and BOTH of this rung's refusals");

    // THE CROSS CHECK — rung 78 re-aims `cap_fuel` and MUST leave `sensed_cap` at rung 76's body,
    // because its own gauged branch deliberately does not consult it and the parent's does.
    assert!(fn_addr_eq(R78_TRIPLE.sensed_cap, R76_TRIPLE.sensed_cap),
            "rung 78 keeps RUNG 76's sensed cap — a re-aim here would disarm rung 76 through the \
             identity arm, silently");

    // RUNG 77 RE-AIMS NOTHING IN THE THIRD-LOOP TABLE AT ALL. It is a pure reader: no knob, no
    // state, no plant code. Eighteen equalities, and with no width tripwire this is the only
    // place a stray re-aim would show.
    for (a, b, name) in triple_pairs(&R77_TRIPLE, &R76_TRIPLE) {
        assert_eq!(a, b, "rung 77 INHERITS `{name}` — it overrides NO plant method");
    }

    // RUNG 78 RE-AIMS EXACTLY TWO. Sixteen equalities, and the two swaps are asserted above.
    for (a, b, name) in triple_pairs(&R78_TRIPLE, &R77_TRIPLE) {
        if name == "shared_rig" || name == "cap_fuel" {
            assert_ne!(a, b, "rung 78 RE-AIMS `{name}`");
        } else {
            assert_eq!(a, b, "rung 78 INHERITS `{name}`");
        }
    }

    // THE ALIAS TABLES CARRY NO SWAP ON EITHER RUNG.
    assert!(fn_addr_eq(R77_TWO.try_close, R76_TWO.try_close));
    assert!(fn_addr_eq(R78_TWO.try_close, R77_TWO.try_close));
    assert!(fn_addr_eq(R77_STATOR.stator_march, R76_STATOR.stator_march));
    assert!(fn_addr_eq(R78_STATOR.stator_march, R77_STATOR.stator_march));
    assert!(fn_addr_eq(R77_FUEL.integrate_fuel, R76_FUEL.integrate_fuel),
            "rung 77 declares no law, so it inherits rung 76's THREE cap-law refusals");
    assert!(fn_addr_eq(R78_FUEL.integrate_fuel, R77_FUEL.integrate_fuel),
            "and so does rung 78 — its own two refusals live in the CAP, not in the march");

    // And every OTHER lever cell is the parent's on both rungs.
    assert!(fn_addr_eq(R77.b_at_point, R76.b_at_point));
    assert!(fn_addr_eq(R78.b_at_point, R77.b_at_point));
    assert!(fn_addr_eq(R77.legs, R76.legs),
            "`_legs` is a name REUSED at rung 77, not an override — rungs 63 and 77 have \
             INCOMPATIBLE parameter lists, so it cannot go in this slot. § 5.28 (x)'s booking, \
             measured and discharged at § 5.32 (iii) item D.");
    assert!(fn_addr_eq(R78.legs, R77.legs), "and no class after rung 77 defines `_legs` at all");
}

/// **`TripleHooks` IS STILL EIGHTEEN AFTER THIS SLICE** — an exhaustive destructuring, so a
/// nineteenth field is a compile error here rather than a silent pass. P6.
#[test]
fn triple_hooks_is_still_eighteen_fields() {
    let TripleHooks {
        stator_leg: _, lagged_stator: _, clamp_v: _, check_v0: _, rk4_floor: _, solve_v: _,
        manifold_v: _, triple_laws: _, triple_rig: _, with_ref: _, reference: _, quad_gains_at: _,
        rk4_floor_shared: _, cap_fuel: _, windup_tau: _, with_coord: _, sensed_cap: _,
        shared_rig: _,
    } = R78_TRIPLE;
}

/// **RUNG 77's CELL DIFFERS FROM ITS PARENT'S IN THE TABLE AND IN NOTHING ELSE** — probe 8's
/// measurement, made as a same-run difference rather than quoted.
///
/// The six knobs a rung-77 `at_lever` copies are exactly the six a rung-76 one copies, so a
/// machine reached through either carries the same six values. What separates them is the tables
/// the machine is BUILT on — and in this port that is the whole content of the cell.
#[allow(non_snake_case)]
#[test]
fn at_lever_differs_from_its_parent_in_the_TABLE_and_in_nothing_else() {
    let arm = valve_arm();
    let s = sensed(&arm);
    let l = ledger(&arm);

    // THE SIX CARRIED KNOBS AGREE, VALUE FOR VALUE. This is the half a value gate can see, and it
    // is the half that says nothing.
    assert_eq!(l.fuel.inner.ref_law.get(), s.fuel.inner.ref_law.get());
    assert_eq!(l.fuel.inner.lag_coord.get(), s.fuel.inner.lag_coord.get());
    assert_eq!(l.fuel.inner.windup_law.get(), s.fuel.inner.windup_law.get());
    assert_eq!(l.fuel.inner.tau_t.get(), s.fuel.inner.tau_t.get());
    assert_eq!(l.fuel.inner.ic_cap.get(), s.fuel.inner.ic_cap.get());
    assert_eq!(l.fuel.inner.cap_law.get(), s.fuel.inner.cap_law.get());

    // THE TABLES DO NOT. A rung-77 machine carries rung 77's lever table; a rung-76 machine
    // carries rung 76's. Compared through a FIELD and not with `ptr::eq` on the table itself:
    // every hook table here is a `pub const`, so `&R77` is a fresh promotion per use site and a
    // table-identity test would be measuring the promotion. Slice AE step 5's finding.
    assert!(fn_addr_eq(l.fuel.inner.lever_hooks.at_lever, R77.at_lever),
            "a rung-77 machine must be built on RUNG 77's lever table");
    assert!(fn_addr_eq(s.fuel.inner.lever_hooks.at_lever, R76.at_lever),
            "and a rung-76 machine on rung 76's — the control that makes the line above a \
             measurement rather than a tautology");
    assert!(!fn_addr_eq(l.fuel.inner.lever_hooks.at_lever, s.fuel.inner.lever_hooks.at_lever));

    // AND THE SIBLING IT HANDS BACK CARRIES THE TABLE TOO — which is the failure the cell exists
    // to prevent: a rung-77 reader running on a rung-76 rig looks right and answers rung 76.
    let sib = l.at_lever(&arm);
    assert!(fn_addr_eq(sib.fuel.inner.lever_hooks.at_lever, R77.at_lever),
            "`at_lever` on a rung-77 machine must return a RUNG-77 machine");
    assert!(fn_addr_eq(sib.fuel.inner.triple_hooks.cap_fuel, R77_TRIPLE.cap_fuel));
}

/// **RUNG 78's CELL CARRIES A SEVENTH KNOB, AND THE SIBLING WOULD OTHERWISE RUN AT `k = 1`.**
#[test]
fn at_lever_carries_the_gauge_and_the_default_is_the_reduce_arm() {
    let arm = valve_arm();
    let g = gauge(&arm);

    // THE DEFAULT IS THE IDENTITY, on the core's own constructor — Python's class attribute.
    assert_eq!(g.fuel.inner.gauge_k.get(), GAUGE_K_IDENTITY);
    assert_eq!(GAUGE_K_IDENTITY, 1.0);

    // ... and it is carried, not re-defaulted. Set the knob off its identity and the sibling must
    // arrive holding it: hand back a machine at `k = 1` and every reader below measures rung 77,
    // which is a legal plant and passes every reduce gate in the crate.
    g.fuel.inner.gauge_k.set(1.5);
    let sib = g.at_lever(&arm);
    assert_eq!(sib.fuel.inner.gauge_k.get(), 1.5,
               "`at_lever` must carry `_gauge_k` — the SIXTEENTH instance of the carried-knob trap");
    assert!(fn_addr_eq(sib.fuel.inner.triple_hooks.cap_fuel, R78_TRIPLE.cap_fuel),
            "and the sibling must be built on rung 78's table, not rung 77's");

    // A rung-77 machine has the field and never leaves the identity: the carrier is on the shared
    // core, so its PRESENCE says nothing about which rung reads it.
    assert_eq!(ledger(&arm).fuel.inner.gauge_k.get(), GAUGE_K_IDENTITY);
}

/// **THE TWO INSTRUMENTS ARE PROCESS-GLOBAL, AND THAT IS PYTHON's DECISION PORTED, NOT A
/// CONVENIENCE.**
///
/// Python's comment on `_gauge_hits`: *"It MUST be written on the CLASS: `self._x += 1` would
/// create an instance attribute and leave the class one at zero forever, so the instrument built
/// to catch a vacuous section would itself have been vacuous."* A per-core `Cell` would be exactly
/// that instance attribute, because the march builds its rig through `at_lever` and the object
/// that runs is not the object a reader afterwards interrogates.
#[test]
fn the_gauge_counters_are_global_and_resettable_as_a_pair() {
    let (h0, b0) = reset_gauge_counters();
    let _ = (h0, b0);
    assert_eq!(gauge_counters(), (0, 0));
    turbojet::residual_gauge::GAUGE_HITS.fetch_add(3, std::sync::atomic::Ordering::Relaxed);
    turbojet::residual_gauge::GAUGE_BINDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(gauge_counters(), (3, 1));
    assert_eq!(reset_gauge_counters(), (3, 1), "the reset RETURNS what it cleared");
    assert_eq!(gauge_counters(), (0, 0));
}

/// The eighteen `TripleHooks` cells of two tables, paired by name, as `usize` addresses.
///
/// Written as one exhaustive destructuring per table rather than eighteen field accesses so that a
/// nineteenth field is a compile error in the pairing too — the shape
/// [`triple_hooks_is_still_eighteen_fields`] would otherwise be the only place holding the width.
fn triple_pairs(a: &TripleHooks, b: &TripleHooks) -> Vec<(usize, usize, &'static str)> {
    let TripleHooks {
        stator_leg: a1, lagged_stator: a2, clamp_v: a3, check_v0: a4, rk4_floor: a5, solve_v: a6,
        manifold_v: a7, triple_laws: a8, triple_rig: a9, with_ref: a10, reference: a11,
        quad_gains_at: a12, rk4_floor_shared: a13, cap_fuel: a14, windup_tau: a15, with_coord: a16,
        sensed_cap: a17, shared_rig: a18,
    } = *a;
    let TripleHooks {
        stator_leg: b1, lagged_stator: b2, clamp_v: b3, check_v0: b4, rk4_floor: b5, solve_v: b6,
        manifold_v: b7, triple_laws: b8, triple_rig: b9, with_ref: b10, reference: b11,
        quad_gains_at: b12, rk4_floor_shared: b13, cap_fuel: b14, windup_tau: b15, with_coord: b16,
        sensed_cap: b17, shared_rig: b18,
    } = *b;
    vec![
        (a1 as usize, b1 as usize, "stator_leg"),
        (a2 as usize, b2 as usize, "lagged_stator"),
        (a3 as usize, b3 as usize, "clamp_v"),
        (a4 as usize, b4 as usize, "check_v0"),
        (a5 as usize, b5 as usize, "rk4_floor"),
        (a6 as usize, b6 as usize, "solve_v"),
        (a7 as usize, b7 as usize, "manifold_v"),
        (a8 as usize, b8 as usize, "triple_laws"),
        (a9 as usize, b9 as usize, "triple_rig"),
        (a10 as usize, b10 as usize, "with_ref"),
        (a11 as usize, b11 as usize, "reference"),
        (a12 as usize, b12 as usize, "quad_gains_at"),
        (a13 as usize, b13 as usize, "rk4_floor_shared"),
        (a14 as usize, b14 as usize, "cap_fuel"),
        (a15 as usize, b15 as usize, "windup_tau"),
        (a16 as usize, b16 as usize, "with_coord"),
        (a17 as usize, b17 as usize, "sensed_cap"),
        (a18 as usize, b18 as usize, "shared_rig"),
    ]
}
