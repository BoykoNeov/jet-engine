//! RUNG 71 — **THE FULL SPLIT**: rung 69's INCIDENCE stator beside rung 70's governor and valve.
//!
//! `n = m = 3`, **ZERO zeros** — the last unoccupied cell of rung 69's table, and rung 70's own
//! strongest seam. Five states, three clocks, one actuator per loop, and now THREE constraints:
//! the governor on `Tt4`, the valve on `phi`, the stator on `M_i`.
//!
//! Headline: *a constraint can be INDEPENDENT in RANK and REDUNDANT on the BAND* — `det J` is
//! non-zero for the first time in the family and FACTORS into rung 67's and rung 69's own
//! non-degeneracy conditions, one per rung, while the third loop rides over only 7.9 % of the
//! march because the valve's wall CONTAINS the stator's on the whole admissible band. So
//! `zeros = n - m` counts GRADIENT DIRECTIONS, not LIVE loops. See `docs/rung71-spec.md`.
//!
//! # What this module is — and what STEP 1 ships
//!
//! [`crate::cross_split`]'s note applies unchanged and is not repeated: **there is no
//! `FullSplitCore`**, one type carries rungs 57–84, and `FullSplitTransient` defines no `__init__`
//! in Python either — so this module is the second of the slice's two "cores" in the only sense
//! the architecture has one.
//!
//! Step 1 ships [`build_full_split_cascade`], the five `R71*` tables, and rung 71's **two swapped
//! cells opened as NAMED PANICS**. Steps 3 fills the bodies; the carrier and guard `_gov_max`
//! needs are rung 70's and live in [`crate::cross_split`], because `_full_rig` writes the same
//! field `_split_rig` does.
//!
//! # THE TABLE ARITHMETIC — this rung's half
//!
//! * **ZERO cells added** (§ 5.27 (i)), and rung 71 overrides **none** of rung 68's nine nor rung
//!   69's one — so `R71_TRIPLE` is the whole of `R70_TRIPLE`, spelled out.
//! * **TWO swaps**, `at_lever` and `integrate_fuel`, both against RUNG 70's bodies and not rung
//!   69's. That matters for the gate: the pointer this rung must differ from is `R70`'s, and a
//!   swap that reached back past rung 70 would be a different defect that a rung-69 comparison
//!   would miss.
//! * **FIVE table consts**, the second half of the slice's ten. See [`crate::cross_split`] for the
//!   reconciled count and why it is not the pre-flight's "nine".
//!
//! # THE TWO SWAPS' MEASURED BREAK SHAPES (§ 5.27 (v))
//!
//! * `at_lever` ← rung 70's — **PANIC + VALUE**, seen by 3 of 6 readers, and observable only
//!   because the parent's `integrate_fuel` then refuses the arming with rung 70's *"THREE loops on
//!   TWO variables"*. Its dispatch gate therefore cannot be written before the arming asserts
//!   land; booked to step 7, exactly as rung 70's is.
//! * `integrate_fuel` ← rung 70's — **PANIC**, seen by 2 of 6. Rung 71 has FOUR asserts where rung
//!   70 has five: the guard rung 70 spends on refusing an incidence stator is the guard this rung
//!   REMOVES, which is the whole rung. Probe 2 classifies the swap as RESTRUCTURED, 11 → 11
//!   comparisons with the guard set MOVED.
//!
//! # THE TWO FLOORS ARE THE SAME CALL SERVING TWO RUNGS — **do not hoist**
//!
//! Registered here at step 1 because it is a design constraint on step 3, not a discovery to be
//! made during it. Rung 71's `integrate_fuel` calls `_rk4_floor_full` and then delegates to rung
//! 70's `_integrate_fuel_cross_triple`, which calls `_rk4_floor_split` **on the same condition
//! with the same rate** — all three floors in the family assert the character-identical
//! `ds * rate <= 2.0` and differ only in the MESSAGE. Probe 12 removed the shadowed call: the
//! rung-71 trajectory is IDENTICAL (341 points, 3 410 keys) and the rung-71 guard still fires —
//! **and the rung-70 guard is GONE.** The shadowed call is not a redundant copy; it is the only
//! floor on the rung-70 arm. A port that hoists one floor into `integrate_fuel` silently deletes
//! rung 70's guard while every rung-71 gate stays green.
//!
//! None of the three floors is a cell (each is defined once, under its own name), so no function
//! pointer exists for a floor and no dispatch gate can substitute one — P5. Each is gated by a
//! `should_panic` on its RUNG TAG, and the tag is the only match string that is unique to one
//! floor: `RK4 stability region` matches all three and `rank TWO` matches two (rungs 69 and 70).

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{FuelLimiters, FuelPoint, FuelTransientCore, FuelTransientHooks};
use crate::map::ComponentMap;
use crate::reference_split::build_split_family_cascade;
use crate::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use crate::three_loop::TripleHooks;
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::TwoSpoolTransientHooks;

// ---------------------------------------------------------------------------------------------
// THE BUILDER — rung 69's constructor, INHERITED THROUGH RUNG 70
// ---------------------------------------------------------------------------------------------

/// Rung 71's constructor: **rung 69's, verbatim, with rung 71's five tables.**
///
/// Neither rung 70 nor rung 71 defines `__init__`, so the guard sequence is rung 69's eleven and
/// this rung adds none — [`build_cross_split_cascade`]'s note, one rung up.
///
/// **AND ONE OF THOSE ELEVEN IS WHY THIS RUNG EXISTS AT ALL.** Rung 69's guard D refuses an
/// incidence floor that is not the valve's `phi` floor at the design setting — *one PHYSICAL
/// wall*. That match is what makes rung 71's containment result true (`M_i = m_lim + v` at
/// `phi = phi_lim` holds BECAUSE `m_lim = T_c - 1/phi_lim` exactly), so the rung's headline is
/// contingent on a constructor assert two rungs below it. Tighten the incidence wall by `delta`
/// and the containment fails for `v < delta`; the guard is what stops that arming from being
/// built.
///
/// [`build_cross_split_cascade`]: crate::cross_split::build_cross_split_cascade
pub fn build_full_split_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    build_split_family_cascade(design_engine, flight_design, mdot_design, map_lp, map_hp, rho,
                               arm, &R71_TWO, &R71_STATOR, &R71_FUEL, &R71, &R71_TRIPLE)
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and TWO of them carry a swap of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 71's lever table — ONE swap, `at_lever`, and the parent it must differ from is **rung
/// 70's**.
///
/// The TENTH instance of the sibling-constructor trap. As at rung 70 the signature does not grow:
/// rung 71 changes which CONSTRAINT the stator watches, and that is armed by `stator_inc` — a
/// keyword rung 69 already added.
pub const R71: LeverHooks = LeverHooks {
    at_lever: r71_at_lever,
    ..crate::cross_split::R70
};

/// RUNG 71's `TwoSpoolTransientHooks` — **ZERO cells swapped**, named for `R66_TWO`'s reason.
pub const R71_TWO: TwoSpoolTransientHooks = crate::cross_split::R70_TWO;

/// RUNG 71's fuel table — ONE swap, `integrate_fuel`: **four arming asserts where rung 70 has
/// five**, then `_rk4_floor_full` and a delegation into rung 70's marcher.
pub const R71_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r71_integrate_fuel,
    ..crate::cross_split::R70_FUEL
};

/// RUNG 71's stator table — **ZERO cells swapped**, named for the same reason.
pub const R71_STATOR: StatorTransientHooks = crate::cross_split::R70_STATOR;

/// RUNG 71's third-loop table — **ZERO of the ten cells swapped.**
///
/// Measured, not assumed: § 5.27 (iii) ran the census per rung and rung 71 overrides none of rung
/// 68's nine and none of rung 69's one. The whole of rung 70's table is inherited, and it is
/// spelled out here rather than reached through a `..R70_TRIPLE` spread for `R69_TRIPLE`'s reason
/// — a spread would make the NEXT addition to that table silent at this rung.
pub const R71_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::cross_split::R70_TRIPLE.stator_leg,
    lagged_stator: crate::cross_split::R70_TRIPLE.lagged_stator,
    clamp_v: crate::cross_split::R70_TRIPLE.clamp_v,
    check_v0: crate::cross_split::R70_TRIPLE.check_v0,
    rk4_floor: crate::cross_split::R70_TRIPLE.rk4_floor,
    solve_v: crate::cross_split::R70_TRIPLE.solve_v,
    manifold_v: crate::cross_split::R70_TRIPLE.manifold_v,
    triple_laws: crate::cross_split::R70_TRIPLE.triple_laws,
    triple_rig: crate::cross_split::R70_TRIPLE.triple_rig,
    with_ref: crate::cross_split::R70_TRIPLE.with_ref,
};

// ---------------------------------------------------------------------------------------------
// THE TWO SWAPPED CELLS — OPENED, NOT YET FILLED
// ---------------------------------------------------------------------------------------------

const UNPORTED: &str = "slice AC step 1 opened this rung-71 cell and has not filled it. This \
                        panic is scaffolding, NOT a refusal: if you are reading it from a march, \
                        the body is owed by step 3 of the slice, not by the caller.";

/// RUNG 71's `at_lever` — **UNPORTED at step 1.** Rung 70's body with `FullSplitTransient` as the
/// class it constructs, which here means [`build_full_split_cascade`].
fn r71_at_lever(_: &ScheduledStatorCore, _: &LeverArm) -> ScheduledStatorCore {
    panic!("{UNPORTED} (at_lever)");
}

/// RUNG 71's `integrate_fuel` — **UNPORTED at step 1.** Four arming asserts, `_rk4_floor_full`,
/// then the delegation into rung 70's `_integrate_fuel_cross_triple` — which calls the SECOND
/// floor. Both calls stay; see this module's note on why hoisting one deletes rung 70's guard.
fn r71_integrate_fuel(
    _: &FuelTransientCore, _: &FlightCondition, _: &dyn Fn(f64) -> f64, _: (f64, f64), _: f64,
    _: f64, _: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    panic!("{UNPORTED} (integrate_fuel)");
}
