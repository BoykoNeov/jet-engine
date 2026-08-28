//! RUNG 70 — **THE GENERIC SPLIT**: rung 47's `Tt4` topping GOVERNOR as the odd loop, beside rung
//! 65's `phi` valve and rung 68's `phi` stator.
//!
//! Rung 67's substitution applied to rung 68's triple. Five states, three clocks, one actuator per
//! loop — rung 68's shapes exactly, and only the ODD loop's COORDINATE differs. `n = 3`, `m = 2`,
//! **the same cell as rung 69 reached by a different route**, so this is a controlled comparison
//! at equal counts.
//!
//! Headline: *the split buys the RANK, but the RING needs the odd constraint to be a SECOND WALL
//! ON THE SAME LEVER* — the floor is rung 67's `zeta`. See `docs/rung70-spec.md`.
//!
//! # What this module is — and what STEP 1 ships
//!
//! **THERE IS NO `CrossSplitCore`, AND A READER SHOULD NOT GO LOOKING FOR ONE.** One type carries
//! rungs 57–84 ([`ScheduledStatorCore`], over [`TwoSpoolTransientCore`]) —
//! `crate::bleed_transient`'s own note — and **neither rung 70 nor rung 71 defines `__init__` in
//! Python at all**, measured over both class bodies rather than assumed. So the "two cores" this
//! slice opens are the two MODULES: this one and [`crate::full_split`], each holding its rung's
//! five tables, and the carrier they share.
//!
//! Step 1 ships: `_gov_max`'s carrier ([`TwoSpoolTransientCore::gov_max`]) and its guard
//! [`GovScope`]; [`build_cross_split_cascade`]; the five `R70*` tables; and rung 70's **three
//! swapped cells opened as NAMED PANICS**. Steps 2–3 fill the bodies.
//!
//! # THE TABLE ARITHMETIC — **TWO COUNTS, AND THEY ARE NOT THE SAME COUNT**
//!
//! Written out here rather than left in two files to be reconciled, which is this phase's
//! most-repeated defect. `tests/slice_ac_cells.rs` holds the assertions.
//!
//! * **ZERO CELLS ADDED**, at either rung. § 5.27 (i)'s finding: the phase's cell predicate is
//!   *new here AND overridden above*, purely by NAME, and it has never asked whether the two
//!   bodies are INTERCHANGEABLE — which is the entire requirement for a `fn` pointer in a `const`
//!   table. The one cell the column claimed for this slice, `split_gains`, is a name REUSED:
//!   rung 80's same-named reader drops `sm, tau, tau_gov, tau_s` and adds `phi_lim, phi_airs,
//!   coord, taus, inc`, and rung 70's own inherited caller `rung67_control` raises `TypeError` on
//!   a rung-80 machine. So `TripleHooks` stays **ten** fields wide and `split_gains` ports as an
//!   ordinary method on the rung-70 type.
//! * **FIVE SWAPS over the two rungs ⇒ FIVE distinct function pointers**: `at_lever` and
//!   `integrate_fuel` at BOTH rungs, `triple_laws` at rung 70 only. Three of the five are here.
//! * **TEN TABLE CONSTS, not nine** — five per rung, and the pre-flight's step-1 line said
//!   "nine". Counted rather than inherited: `R70`, `R70_TWO`, `R70_FUEL`, `R70_STATOR`,
//!   `R70_TRIPLE` here and the same five at rung 71. The gate counts the `pub const R7`
//!   declarations in the two files, so the number cannot drift back into prose.
//!
//! # WHY THE FILE IS SHAPED THE WAY IT IS — the swaps, again
//!
//! Phase 7's rule is *step 1 of every slice is the cell addition*, so a slice that forgets a cell
//! fails at its own first gate. **This slice adds no cell at all, so that rule buys nothing** —
//! and the risk it was protecting against is entirely on the other side: a forgotten SWAP is not
//! a missing function, it is the PARENT's, which compiles, runs, and is caught by nothing the
//! ladder does automatically.
//!
//! So, on slice AB's precedent: the three unported bodies are **named panics** rather than
//! `todo!()` (a `todo!()` and the parent's body read alike, and a per-cell message makes the slot
//! addressable), and `R70_TRIPLE` carries **no `..R69_TRIPLE` spread** — the nine inherited cells
//! are decisions on the page.
//!
//! **AND THE STEP-1 GATE IS NOT AB's.** AB asserted distinctness by READING the nine placeholder
//! panic messages, and step 2 deleted every one of them, so the gate had to be dismantled — its
//! whole content was *"not yet ported"*. `tests/slice_ac_cells.rs` asks the same question as
//! **pointer inequality between two shipped `const`s** instead, which survives the bodies landing.
//!
//! # WHAT STEP 1 DELIBERATELY DOES NOT GATE
//!
//! **`at_lever`'s DISPATCH gate cannot be written yet, and its absence is a decision.** § 5.27 (v)
//! measured both `at_lever` swaps: the two bodies differ only in which class they CONSTRUCT, so
//! injecting the parent's makes `_split_rig` hand back the parent's class carrying `_gov_max` as a
//! stray attribute — which Python allows. It is observable **only because the parent's
//! `integrate_fuel` then REFUSES the arming**. In Rust `at_lever` returns a table pointer and
//! nothing refuses anything until those asserts are ported, so a gate written now would report
//! UNOBSERVABLE for a reason about ORDERING rather than about the cell. Booked to step 7.
//!
//! # `_gov_max` — CONFIG-kind, and **THE MIRROR OF `_ref`'s RESTORE POLICY**
//!
//! See [`TwoSpoolTransientCore::gov_max`] for the census. The half worth repeating here is the
//! half that would be wrong if slice AB's reasoning were copied across: `_with_ref` is entered to
//! SET a reference over a `None`, so all 29 of its restores put `None` back and a restore-to-`None`
//! guard agrees with Python on every shipped path. **`_with_gov` is entered to turn the governor
//! OFF**, and that is a CALL-SITE ENUMERATION rather than a sample: `engine.py` holds exactly
//! **three** `_with_gov` call sites in the whole ladder and all three pass a literal `None`. So
//! the two spellings agree at the SET and differ at the RESTORE wherever the receiver's governor
//! is armed — which on a rung-70/71 rig it always is. [`GovScope`] restores the previous value,
//! and unlike [`RefScope`](crate::reference_split::RefScope) that choice is reachable by an
//! ordinary value witness rather than only by a manufactured nest.
//!
//! [`ScheduledStatorCore`]: crate::stator_transient::ScheduledStatorCore
//! [`TwoSpoolTransientCore`]: crate::two_spool_transient::TwoSpoolTransientCore
//! [`TwoSpoolTransientCore::gov_max`]: crate::two_spool_transient::TwoSpoolTransientCore::gov_max

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, Floor, FuelLimiters, FuelPoint, FuelTransientCore, FuelTransientHooks,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::reference_split::build_split_family_cascade;
use crate::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use crate::three_loop::{TripleHooks, TripleLaws};
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{TwoSpoolTransientCore, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// `_gov_max` — THE CARRIER'S GUARD
// ---------------------------------------------------------------------------------------------

/// The RAII form of Python's `_with_gov`'s `try/finally` — **the restore is `Drop`, so it survives
/// an unwind that a straight-line restore would skip.**
///
/// Python is `prev, self._gov_max = self._gov_max, val` … `try: return fn(*a, **kw)` …
/// `finally: self._gov_max = prev`, and its docstring gives the reason in rung 62's words: *a
/// leaked setting would make a reader report a plant that was never marched.*
///
/// **RESTORE-PREVIOUS, AND HERE THAT IS A LIVE CHOICE RATHER THAN A FORMALITY.** Slice AB's
/// [`RefScope`](crate::reference_split::RefScope) restores the previous value too, but measured
/// over its own suite the displaced value was `None` at every one of its 29 value-sets, so
/// restore-previous and restore-`None` agreed on every shipped path and only a manufactured nest
/// could tell them apart. This guard is the MIRROR: `_with_gov` is entered to turn the governor
/// **off** — all THREE of its call sites in the ladder pass a literal `None` — over a `prev` that
/// a rig has just armed, so a restore-to-`None` here would silently disarm the governor for every
/// reader that runs after the scope closes.
///
/// **IT WRITES THE FIELD DIRECTLY AND NOT THROUGH A CELL**, which is the opposite of `RefScope`'s
/// decision and for a measured reason: rung 73 overrides `_with_ref` to write `_ref_law`, so that
/// setter had to be dispatched; `_with_gov` is defined once in the whole ladder and overridden
/// nowhere, so there is no second field for a cell to choose between. If a later rung overrides
/// it, the repair is `RefScope`'s — add the cell — and this comment is where that reader lands.
pub struct GovScope<'a> {
    core: &'a TwoSpoolTransientCore,
    prev: Option<f64>,
}

impl<'a> GovScope<'a> {
    /// Arm (or disarm) the governor's set point for as long as the returned guard lives.
    ///
    /// `None` is a real assignment and not a no-op: it is rung 68's FUEL leg, a different law,
    /// which is exactly what `_with_gov(None, …)` is called 35 times to select.
    pub fn set(core: &'a TwoSpoolTransientCore, val: Option<f64>) -> Self {
        let prev = core.gov_max.get();
        core.gov_max.set(val);
        GovScope { core, prev }
    }

    /// What this scope displaced — Python's `prev`, exposed so a gate can read the restore POLICY
    /// rather than only its effect. [`RefScope::displaced`]'s precedent.
    ///
    /// [`RefScope::displaced`]: crate::reference_split::RefScope::displaced
    pub fn displaced(&self) -> Option<f64> {
        self.prev
    }
}

impl Drop for GovScope<'_> {
    fn drop(&mut self) {
        self.core.gov_max.set(self.prev);
    }
}

// ---------------------------------------------------------------------------------------------
// THE BUILDER — rung 69's constructor, INHERITED
// ---------------------------------------------------------------------------------------------

/// Rung 70's constructor: **rung 69's, verbatim, with rung 70's five tables.**
///
/// `CrossSplitTransient` defines no `__init__` in Python, so every guard that fires here is one of
/// rung 69's eleven and this function adds none of its own. That is why it is a call to
/// [`build_split_family_cascade`] rather than a re-spelling: re-writing the asserts once per rung
/// would be a copy the SOURCE does not make.
///
/// **The one arming refusal rung 70 does own is not here — it is in `integrate_fuel`**, which
/// refuses an INCIDENCE stator, a `tau_gov` with no `Tt4_max`, rung 52's fuel leg, rungs 50/51's
/// forced release edges, and an instantaneous valve beside a lagged stator. Five asserts, all
/// measured reachable by ARMING (§ 5.27 (vi)); they land at step 2.
pub fn build_cross_split_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    build_split_family_cascade(design_engine, flight_design, mdot_design, map_lp, map_hp, rho,
                               arm, &R70_TWO, &R70_STATOR, &R70_FUEL, &R70, &R70_TRIPLE)
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and THREE of them carry a swap of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 70's lever table — ONE swap, `at_lever`, the sibling constructor that must hand back a
/// rung-70 machine.
///
/// **THE NINTH INSTANCE OF THE TRAP RUNGS 61–69 EACH HIT**, and Python's own docstring says the
/// failure mode is back to rung 67's plain one: the signature does NOT grow here, because rung
/// 70's third loop is armed by a MARCH argument (`tau_gov`) rather than a machine keyword. So a
/// forgotten swap hands back the parent's class and every reader then measures rung 69's plant
/// while reporting rung 70's.
pub const R70: LeverHooks = LeverHooks {
    at_lever: r70_at_lever,
    ..crate::reference_split::R69
};

/// RUNG 70's `TwoSpoolTransientHooks` — **ZERO cells swapped**, named for `R66_TWO`'s reason: a
/// spread of the parent would make the NEXT addition to that table silent here.
pub const R70_TWO: TwoSpoolTransientHooks = crate::reference_split::R69_TWO;

/// RUNG 70's fuel table — ONE swap, `integrate_fuel`: the march that owns rung 70's five arming
/// refusals and then delegates to `_integrate_fuel_cross_triple`.
pub const R70_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r70_integrate_fuel,
    ..crate::reference_split::R69_FUEL
};

/// RUNG 70's stator table — **ZERO cells swapped**, named for the same reason.
pub const R70_STATOR: StatorTransientHooks = crate::reference_split::R69_STATOR;

/// **RUNG 70's THIRD-LOOP TABLE — ONE of the ten cells swapped, and it is the one rung 69 left
/// alone.**
///
/// `triple_laws` is the ninth of rung 68's nine, and rung 70 is the only class in the ladder that
/// overrides it — which is why `R69_TRIPLE` spells it out as `R68_TRIPLE.triple_laws` with a
/// comment naming this rung. That is the chain the swap breaks, and
/// `tests/slice_ac_cells.rs` gates all three links of it.
///
/// The other nine are written out rather than reached through a `..R69_TRIPLE` spread, for
/// `R69_TRIPLE`'s own reason: this slice's entire risk is a swap that is silently the parent's, so
/// every inherited cell is a decision on the page.
pub const R70_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::reference_split::R69_TRIPLE.stator_leg,
    lagged_stator: crate::reference_split::R69_TRIPLE.lagged_stator,
    clamp_v: crate::reference_split::R69_TRIPLE.clamp_v,
    check_v0: crate::reference_split::R69_TRIPLE.check_v0,
    rk4_floor: crate::reference_split::R69_TRIPLE.rk4_floor,
    solve_v: crate::reference_split::R69_TRIPLE.solve_v,
    manifold_v: crate::reference_split::R69_TRIPLE.manifold_v,
    // RUNG 70's OWN — the one cell of rung 68's nine that reached this far unswapped.
    triple_laws: r70_triple_laws,
    triple_rig: crate::reference_split::R69_TRIPLE.triple_rig,
    with_ref: crate::reference_split::R69_TRIPLE.with_ref,
};

// ---------------------------------------------------------------------------------------------
// THE THREE SWAPPED CELLS — OPENED, NOT YET FILLED
// ---------------------------------------------------------------------------------------------
//
// Each panics with its OWN message naming the cell, on slice AB step 1's precedent and for its
// reason: a `todo!()` reads exactly like the parent's body to anyone scanning the table, and this
// slice's risk is a slot that silently holds the parent. A per-cell message makes each slot
// addressable — but the GATE that reads these messages is deliberately not written, because step
// 2 deletes them and AB had to dismantle exactly that gate. `tests/slice_ac_cells.rs` asks for
// pointer INEQUALITY instead, which is still a question after the bodies land.

const UNPORTED: &str = "slice AC step 1 opened this rung-70 cell and has not filled it. This \
                        panic is scaffolding, NOT a refusal: if you are reading it from a march, \
                        the body is owed by step 2 of the slice, not by the caller.";

/// RUNG 70's `at_lever` — **UNPORTED at step 1.** Rung 69's body with `CrossSplitTransient` as the
/// class it constructs, which on this side of the port means [`build_cross_split_cascade`].
fn r70_at_lever(_: &ScheduledStatorCore, _: &LeverArm) -> ScheduledStatorCore {
    panic!("{UNPORTED} (at_lever)");
}

/// RUNG 70's `integrate_fuel` — **UNPORTED at step 1.** The five arming asserts, the reduce arm
/// (`tau_gov is None or not _lagged_stator()` ⇒ the parent verbatim), then
/// `_integrate_fuel_cross_triple`.
fn r70_integrate_fuel(
    _: &FuelTransientCore, _: &FlightCondition, _: &dyn Fn(f64) -> f64, _: (f64, f64), _: f64,
    _: f64, _: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    panic!("{UNPORTED} (integrate_fuel)");
}

/// RUNG 70's `_triple_laws` — **UNPORTED at step 1.** Rung 68's three closures with `R` swapped
/// for rung 47's governor when [`gov_max`] is armed, and the parent's answer verbatim when it is
/// not.
///
/// **ITS BREAK SHAPE IS THE SLICE'S SECOND HEADLINE, AND IT IS NOT A VALUE.** § 5.27 (ii)
/// injected rung 68's body here and measured: the governor simply is not there, every sampled
/// point comes back `interior = false`, `rows` goes **2 → 0**, and `split_gains` returns
/// **successfully** with every aggregate `None`. A dispatch gate of the shape every previous
/// slice has written — march both, diff the value keys — compares two empty tables and passes.
/// The gate for this cell is a **non-emptiness** assertion (P2), and the general form is worth
/// carrying: *a cell whose output is a SAMPLE can break by changing the sample's SIZE rather than
/// its values, and a value-diff gate is blind to that by construction.*
///
/// [`gov_max`]: crate::two_spool_transient::TwoSpoolTransientCore::gov_max
fn r70_triple_laws<'a>(
    _: &'a ScheduledStatorCore, _: &'a FlightCondition, _: f64, _: f64, _: f64,
    _: Option<&'a AccelSchedule>, _: Option<&'a Floor>,
) -> Result<TripleLaws<'a>, Abort> {
    panic!("{UNPORTED} (_triple_laws)");
}
