//! RUNG 74 — **THE DEMAND COORDINATE.** `DemandCoordinateTransient`, slice AF.
//!
//! Every fuel-side leg since rung 47 carries the CLIP as its state — the CUT, floored at zero —
//! and rung 73 § 11 concedes that a real fuel control does not. Substituting `w = mf_sched - g`
//! and `cap = mf_sched - req` turns the lag into `dg/ds = (req - g)/tau + d(mf_sched)/ds`: the
//! added term is STATE-INDEPENDENT, so it appears in no Jacobian and **a coordinate on the lag is
//! PURE BILL — it cannot touch the rank, and it moves the cut by the schedule's own slope.**
//!
//! # WHAT STEP 1 OF THIS SLICE ADDS — **FOUR NEW TABLE FIELDS AND FOUR RE-AIMED POINTERS**
//!
//! | | Python | slot | table |
//! |---|---|---|---|
//! | **ADD** | `_cap_fuel` | [`cap_fuel`](TripleHooks::cap_fuel) | [`R74_TRIPLE`] |
//! | **ADD** | `_sensed_cap` | [`sensed_cap`](TripleHooks::sensed_cap) | [`R74_TRIPLE`] |
//! | **ADD** | `_windup_tau` | [`windup_tau`](TripleHooks::windup_tau) | [`R74_TRIPLE`] |
//! | **ADD** | `_with_coord` | [`with_coord`](TripleHooks::with_coord) | [`R74_TRIPLE`] |
//! | swap | `_rk4_floor_shared` | [`rk4_floor_shared`](TripleHooks::rk4_floor_shared) | [`R74_TRIPLE`] |
//! | swap | `_shared_rig` | [`shared_rig`](TripleHooks::shared_rig) | [`R74_TRIPLE`] |
//! | swap | `at_lever` | `LeverHooks::at_lever` | [`R74`] |
//! | swap | `integrate_fuel` | `FuelTransientHooks::integrate_fuel` | [`R74_FUEL`] |
//!
//! **`TripleHooks` GOES 14 → 18 HERE — the widest single arrival the table has had**, and it fired
//! **FOUR** test-target tripwires, not the two this very paragraph claimed until slice AF step 2
//! read it back: two `E0063` initializer literals (`tests/slice_ab_cells.rs`,
//! `tests/slice_ac_cells.rs`) and two `E0027` exhaustive destructurings (`tests/slice_ae_cells.rs`,
//! `tests/slice_ae_dispatch.rs`). Step 1 measured all four and renamed them in the two `_cells`
//! files; **this header — the one that reports the finding — kept the stale two.** That is
//! § 5.30 (ii)'s own lesson landing on the file that states it: check the ROW, not just whether
//! the correction exists somewhere. No count was pre-registered: slice AD's P1 predicted five
//! `E0063` sites and needed seven, because `cargo check --all-targets` stops when the lib fails
//! and never reaches the test targets. A width prediction can only be measured as *apply, fix the
//! lib, count what is still red*.
//!
//! # § 5.30 (v)'s STEP LIST ASSIGNED TWO OF THESE CELLS TWICE, AND THE BOUNDARY IS RE-CUT HERE
//!
//! Step 1 is *"the plumbing, the four ADD cells + four SWAPs, the refusals, and a smoke file"* and
//! step 2 is *"`_cap_free` / `_cap_gov` / `_cap_fuel` / `_sensed_cap` and the demand laws"* —
//! `_cap_fuel` and `_sensed_cap` are in both. That is § 5.30's own recurring defect (two claims
//! individually plausible and jointly impossible) a third time inside the same section, and it is
//! recorded rather than silently resolved.
//!
//! **All four ADD cells land here, and `cap_free` / `cap_gov` come with them.** The reason is not
//! tidiness: adding a field to [`TripleHooks`] means editing seven exhaustive literals in `src`
//! plus two in `tests`, so splitting the arrival 14 → 17 → 18 would pay that toll twice and
//! measure nothing between the halves. The alternative — a `cap_fuel` field pointing at a
//! `todo!()` — is a live panic sitting in a `const` table for a whole step. Step 2 keeps the
//! sentence's second half: `_applied_demand`, `_demand_target`, `_demand_reference`,
//! `_demand_tau`, `_demand_authority` and `_demand_laws`.
//!
//! The file is `tests/slice_af_cells.rs` and not `slice_af_smoke.rs` for the same kind of reason:
//! a smoke file runs READERS end to end (`slice_ab_smoke.rs` is the pattern) and this step has
//! none — they land at step 4.
//!
//! # THE NAME REUSE, CAUGHT BEFORE ITS SECOND DEFINER RATHER THAN AFTER
//!
//! `_with_coord` is defined at rung 74 and again at rung **79** with an identical signature and a
//! different mutated field: rung 74 writes `_lag_coord`, rung 79 writes `_phi_ref`. This is
//! `_with_ref`'s defect (rung 69 writes `_ref`, rung 73 writes `_ref_law`) — which slice AE had to
//! repair after the fact, because both fields exist on the downstream machine so nothing
//! type-errors and no signature comparison can see it.
//!
//! **§ 5.30 (ii) re-derived the obligation from the census rather than inheriting slice AE's
//! booking**, and that matters: the booking named `_cap_march` as this rung's candidate reader,
//! and `_cap_march` is rung 76's method and rung 79's call site. The census answers the same
//! question from the source — `_with_coord` has exactly two definers, rungs 74 and 79 — which is
//! a stronger warrant than a sentence in a prior slice. So [`lag_coord`] is rung 74's OWN field,
//! `phi_ref` stays unborn until rung 79's slice, and no slot here is one a later slice can re-aim.
//!
//! # THE COORDINATE IS READ, AND ON THE ONE PATH THROUGH `_with_coord` IT IS THE IDENTITY
//!
//! Slice AE withdrew `_with_coord`'s behavioural claim as *undriven* and booked the drive test
//! here. § 5.30 (i) discharges it BY MEASUREMENT and inverts the stated reason. The coordinate is
//! read — by exactly one method, on exactly one line, sixteen times per gains reading:
//! `_demand_target`'s `min(mf_sched, cap) if self._lag_coord == "demand-latched" else cap`. It is
//! a **three-valued tag read by a two-valued test**, so `clip` and `demand` are indistinguishable
//! to it by construction and the third value is distinguishable only where `cap > mf_sched`.
//!
//! It never is, anywhere this scope's one call site looks: **0 of 1 040 calls on the `phi_lim`
//! arm the shipped test uses and 0 of 624 on the second arm**, with the third arm refusing through
//! a shipped guard. A mutated reader moves 4 of 20 float keys on every arm, so the instrument can
//! see and the zero is ARITHMETIC. **So no value gate exists for this cell at this rung and the
//! field is gated structurally** — which is P5, and if a value gate IS found the finding inverts.
//!
//! # THE ARITHMETIC SURFACE — a solver where rung 73 added none, and it is measured clean
//!
//! [`cap_free`] walks a geometric bracket and calls `_illinois` — the function slice AA measured
//! taking 8 iterations on one interpreter and 7 on the other from bit-identical inputs. On the
//! anchor demand march: **2 732 calls, 2 593 short-circuits, 139 that actually bracket and
//! solve, and 2 732 of 2 732 returned caps bitwise identical across PyPy 3.11.15 and CPython
//! 3.14.3.** So the CPython exemption is not inherited from rung 73's *adds no solver* and is
//! also not needed — a falsifiable claim, settled at the oracle step.
//!
//! [`lag_coord`]: crate::two_spool_transient::TwoSpoolTransientCore::lag_coord

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelInstant, FuelLimiters, FuelPoint, FuelTransientCore,
    FuelTransientHooks,
};
use crate::applied_reference::REF_LAW_APPLIED;
use crate::fuel_transient::Authority;
use crate::gas::Abort;
use crate::limited_bleed::Regime;
use crate::map::ComponentMap;
use crate::shared_actuator::{py_max4, SharedRigArm, IC_ORDER4_DECLARED};
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use crate::three_loop::{closer_b, closer_v, LegRegime, TripleHooks};
use crate::two_spool_transient::{MarchedBleed, MarchedStator};
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{TwoSpoolTransientCore, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// THE DECLARED CONSTANTS — rung 74's two class attributes, and `_cap_free`'s two defaults
// ---------------------------------------------------------------------------------------------

/// Python's `_lag_coord = "clip"` — **THE CLASS DEFAULT, AND IT IS THE REDUCE ARM.**
///
/// Rungs 52–73 carry the state `g`, the CUT floored at zero, and this is that plant. It is also
/// the value [`TwoSpoolTransientCore`]'s constructor already writes, so — unlike rung 73's
/// `_ref_law` — there is nothing for [`build_demand_coordinate_cascade`] to overwrite and no
/// silent-wrong-plant failure of slice AE step 1's second kind to gate for.
pub const LAG_COORD_CLIP: &str = "clip";

/// Python's `"demand"` — the state `w`, the fuel the leg would ALLOW, floored on the COMPOSITION.
/// **THIS IS THE PLANT** the rung is about.
pub const LAG_COORD_DEMAND: &str = "demand";

/// Python's `"demand-latched"` — the demand state with the target capped at the schedule, i.e.
/// the floor moved back onto the STATE. § 3's isolation instrument, and exactly the clip plant
/// plus the forcing.
pub const LAG_COORD_LATCHED: &str = "demand-latched";

/// The three coordinates [`r74_integrate_fuel`]'s first refusal admits, in Python's tuple order.
///
/// Named so the refusal and the gate that drives it read the same list —
/// [`REF_LAWS_DECLARED`](crate::applied_reference::REF_LAWS_DECLARED)'s reason, one knob over.
pub const LAG_COORDS_DECLARED: [&str; 3] = [LAG_COORD_CLIP, LAG_COORD_DEMAND, LAG_COORD_LATCHED];

/// Python's `_ic_cap = 60` — the pass cap on this rung's joint initial-condition fixed point.
///
/// Rung 75 § 0.3 shows that residual falling GEOMETRICALLY at a ratio the anti-windup device sets,
/// so `60` is a cut across a geometric sequence and not a property of any plant. It is raised only
/// in a rung-75 READER, to measure a derived iteration count; every plant in the family keeps it.
///
/// Its one reader is this rung's own march (`engine.py:17941`), which lands at step 3 — measured,
/// not assumed, and the reason it is a `Cell` on the core rather than a bare constant is written
/// at [`ic_cap`](crate::two_spool_transient::TwoSpoolTransientCore::ic_cap).
pub const IC_CAP_DECLARED: usize = 60;

/// [`cap_free`]'s bracket growth — Python's `grow: float = 1.0 / 0.9`.
///
/// **SPELLED AS THE DIVISION AND NOT AS A DECIMAL**, because it is the mirror image of
/// `_surge_fuel`'s own `0.9` shrink: it is a DIRECTION, not a new constant, and `1.1111111111111112`
/// would hide that while also being a different float from some other expansion of the same idea.
///
/// **Never overridden.** All fourteen `_cap_free` call sites in the ladder (rungs 74, 77, 78, 79)
/// take both defaults, so the parameters are constants here rather than arguments nobody varies.
pub const CAP_GROW: f64 = 1.0 / 0.9;

/// [`cap_free`]'s bracket-walk cap — Python's `n: int = 60`, and **not `40`**.
///
/// Written down with its wrong value beside it because § 5.30 (vii)'s fourth instrument defect was
/// exactly this: a pre-flight probe typed `n = 40` against the shipped `60` and would have measured
/// a different bracket walk while *passing*.
pub const CAP_BRACKET_N: usize = 60;

// ---------------------------------------------------------------------------------------------
// `_lag_coord` — THE CARRIER'S GUARD
// ---------------------------------------------------------------------------------------------

/// The RAII form of Python's `_with_coord`'s `try/finally` — **the restore is `Drop`, so it
/// survives an unwind that a straight-line restore would skip.**
///
/// Python is `prev, self._lag_coord = self._lag_coord, coord` … `try: return fn(*a, **kw)` …
/// `finally: self._lag_coord = prev`, which its own docstring calls *rung 62's reason, EIGHTH
/// reload*.
///
/// **THE GUARD IS SHARED AND THE SETTER IS THE CELL**, which is
/// [`RefScope`](crate::reference_split::RefScope)'s decision and not
/// [`ShareScope`](crate::shared_actuator::ShareScope)'s or
/// [`GovScope`](crate::cross_split::GovScope)'s. The rule those three comments state between them
/// is: dispatch the setter iff a later rung overrides `_with_*` to write a DIFFERENT field. Rung
/// 79 does exactly that (`_phi_ref`), so this one is dispatched.
///
/// **AND THE SENTENCE THAT USED TO POINT HERE NAMED A TYPE THAT NEVER EXISTED.**
/// `applied_reference.rs`'s header said *"`cross_split.rs`'s `CoordScope` repeats the reasoning
/// from the mirror side"*. `git log -S "CoordScope"` returns exactly one commit — the one that
/// wrote that sentence — so the name was invented in prose; the type in `cross_split.rs` is
/// [`GovScope`](crate::cross_split::GovScope), and its own doc says it writes its field
/// **directly and not through a cell**, which is the OPPOSITE of the decision the sentence
/// credited it with. Corrected at its source in this commit. It is § 5.30 (i)'s finding one slice
/// on — *verify the name owns what the sentence says it owns* — and it was urgent because this
/// type is the first real `CoordScope` in the crate, so the stale reference would have resolved
/// to it and told a future reader a wrong story.
///
/// **RESTORE-PREVIOUS**, which is what Python's `finally` does. Whether it can be told apart from
/// restore-to-default on any shipped path is [`RefScope`]'s question and is not answered here:
/// this rung's readers set the coordinate on a machine whose class default is `"clip"`, so the
/// two agree wherever the displaced value is the default. The discriminator is a manufactured
/// nest, and it is written at step 5 with the rest of the dispatch gates.
///
/// [`RefScope`]: crate::reference_split::RefScope
pub struct CoordScope<'a> {
    core: &'a TwoSpoolTransientCore,
    prev: &'static str,
}

impl<'a> CoordScope<'a> {
    /// Set the coordinate for as long as the returned guard lives, **through the cell**.
    ///
    /// Nothing else in the crate may write the carrier — this is the only public way in, which is
    /// what makes the pairing structural rather than a discipline.
    pub fn set(core: &'a TwoSpoolTransientCore, coord: &'static str) -> Self {
        let prev = core.with_coord(coord);
        CoordScope { core, prev }
    }

    /// What this scope displaced — Python's `prev`, exposed so a gate can read the restore POLICY
    /// rather than only its effect. [`RefScope::displaced`]'s precedent.
    ///
    /// [`RefScope::displaced`]: crate::reference_split::RefScope::displaced
    pub fn displaced(&self) -> &'static str {
        self.prev
    }
}

impl Drop for CoordScope<'_> {
    fn drop(&mut self) {
        // Through the SAME cell, so a rung that moves the field moves both halves of the guard at
        // once. Writing `self.core.lag_coord.set(self.prev)` here would work at rung 74 and
        // silently restore the WRONG field at rung 79 — `RefScope`'s recorded reason, and the one
        // this family has already been bitten by once.
        (self.core.triple_hooks.with_coord)(self.core, self.prev);
    }
}

// ---------------------------------------------------------------------------------------------
// THE CASCADE BUILDER
// ---------------------------------------------------------------------------------------------

/// Build a rung-74 object, so every sibling re-asserts the whole chain's guards.
///
/// **IT STILL SETS `_ref_law`, BECAUSE PYTHON'S CLASS ATTRIBUTE IS INHERITED.**
/// `DemandCoordinateTransient` subclasses `AppliedReferenceTransient`, so a fresh rung-74 object
/// reads `'applied'` — while [`TwoSpoolTransientCore`]'s constructor writes
/// [`REF_LAW_DEFAULT`](crate::shared_actuator::REF_LAW_DEFAULT) = `"sched"` for every rung in the
/// family. A rung-74 builder that dropped the overwrite would hand back a machine that passes its
/// own refusals, marches rung 72's reference, and reports rung 74 in every reader — slice AE step
/// 1's second finding, which is inherited by construction rather than by copying.
///
/// **IT DOES NOT SET `_lag_coord`, AND THAT IS A DECISION.** Rung 74 declares `"clip"`, which is
/// already the core's default, so the set would be a no-op no gate could see — and writing it
/// anyway would put a line here that looks like it is doing the `ref_law` job. What DOES have to
/// carry the coordinate is [`r74_at_lever`] and [`r74_shared_rig`], because a sibling built while
/// the receiver sits under a [`CoordScope`] must inherit the receiver's coordinate and not the
/// class default.
pub fn build_demand_coordinate_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = crate::reference_split::build_split_family_cascade(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
        &R74_TWO, &R74_STATOR, &R74_FUEL, &R74, &R74_TRIPLE);
    if let ScheduledStatorTransient::Full(c) = &built {
        c.fuel.inner.ref_law.set(crate::applied_reference::REF_LAW_APPLIED);
    }
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and TWO of them carry something of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 74's lever table — ONE swap, `at_lever`, and the parent it must differ from is rung 73's.
///
/// The THIRTEENTH instance of the sibling-constructor trap, now with THREE knobs to drop instead
/// of two: hand back the parent's class and every reader measures rung 73's plant while reporting
/// the demand coordinate; hand back the right class while dropping `_lag_coord` and the sibling
/// silently marches `"clip"`.
pub const R74: LeverHooks = LeverHooks {
    at_lever: r74_at_lever,
    ..crate::applied_reference::R73
};

/// RUNG 74's `TwoSpoolTransientHooks` — **ZERO cells swapped**, an alias.
pub const R74_TWO: TwoSpoolTransientHooks = crate::applied_reference::R73_TWO;

/// RUNG 74's fuel table — ONE swap, `integrate_fuel`: **five refusals SPLIT across a dispatch.**
pub const R74_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r74_integrate_fuel,
    ..crate::applied_reference::R73_FUEL
};

/// RUNG 74's stator table — **ZERO cells swapped**, an alias.
pub const R74_STATOR: StatorTransientHooks = crate::applied_reference::R73_STATOR;

/// RUNG 74's third-loop table — **TWO of rung 73's cells re-aimed and FOUR added, of eighteen.**
///
/// Spelled out field by field rather than reached through a `..R73_TRIPLE` spread, for
/// [`R73_TRIPLE`](crate::applied_reference::R73_TRIPLE)'s stated reason: twelve INHERITED decisions
/// sit on the page as decisions instead of as the residue of a spread, and an exhaustive literal
/// is what goes loud when a later slice widens the struct again.
pub const R74_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::applied_reference::R73_TRIPLE.stator_leg,
    lagged_stator: crate::applied_reference::R73_TRIPLE.lagged_stator,
    clamp_v: crate::applied_reference::R73_TRIPLE.clamp_v,
    check_v0: crate::applied_reference::R73_TRIPLE.check_v0,
    rk4_floor: crate::applied_reference::R73_TRIPLE.rk4_floor,
    solve_v: crate::applied_reference::R73_TRIPLE.solve_v,
    manifold_v: crate::applied_reference::R73_TRIPLE.manifold_v,
    triple_laws: crate::applied_reference::R73_TRIPLE.triple_laws,
    triple_rig: crate::applied_reference::R73_TRIPLE.triple_rig,
    with_ref: crate::applied_reference::R73_TRIPLE.with_ref,
    reference: crate::applied_reference::R73_TRIPLE.reference,
    quad_gains_at: crate::applied_reference::R73_TRIPLE.quad_gains_at,
    // THE TWO THIS RUNG RE-AIMS.
    rk4_floor_shared: r74_rk4_floor_shared,
    shared_rig: r74_shared_rig,
    // AND THE FOUR IT ADDS — the arrival that takes the table 14 -> 18.
    cap_fuel: r74_cap_fuel,
    sensed_cap: r74_sensed_cap,
    windup_tau: r74_windup_tau,
    with_coord: r74_with_coord,
};

// ---------------------------------------------------------------------------------------------
// THE FOUR ADDED CELLS
// ---------------------------------------------------------------------------------------------

/// RUNG 74's `_with_coord` — **THE SETTER, and the whole of it is WHICH FIELD IS WRITTEN.**
///
/// Rung 79's body has the same arity, the same `try/finally` and one parameter renamed
/// (`coord` → `ref`); it writes `_phi_ref`. Both fields exist on a rung-79 machine, which is why
/// no signature comparison and no type error can reach the difference and why the cell exists
/// before the second definer is ported rather than after — [`CoordScope`]'s note.
///
/// **NO REFUSAL HERE, WHERE `_with_ref`'s ANALOGUE HAS ONE.** That cell's shared signature carries
/// an `Option` because rung 69's field is optional and rung 73's is not, so its rung-73 body has a
/// `None` to reject. Both definers of THIS name declare a plain `str` with a class default, so
/// there is no unset state to spell — and inventing a refusal for a value that cannot arrive would
/// be a gate with nothing to catch.
///
/// **THE VALUE IS NOT VALIDATED HERE EITHER, AND THAT IS PYTHON's PLACEMENT.** The three declared
/// coordinates are checked in [`r74_integrate_fuel`], at the march, so a reader may legally set a
/// coordinate the plant would refuse — which is exactly what makes the refusal reachable.
fn r74_with_coord(t: &TwoSpoolTransientCore, coord: &'static str) -> &'static str {
    let prev = t.lag_coord.get();
    t.lag_coord.set(coord);
    prev
}

/// RUNG 74's `_sensed_cap` — **`None`, which is rung 73 § 6's concession stated as code.**
///
/// Every cap in this family is a SET-POINT SOLVE, so it is a function of the STATE alone and
/// `d(cap)/d(mf) = 0` — which is what collapses rung 73's applied reference from a continuum to
/// three readings. Rung 76 replaces this with rung 48's schedule AS WRITTEN, evaluated at the fuel
/// actually burning, and that is the first cap in the ladder that can read `mf_app`.
///
/// **`static` in Python HERE and an instance method THERE**, so the receiver is present and
/// unused; and the return is `Result` because rung 76's body calls `_instant_fuel`. Both are the
/// cell's width being the family's rather than the introducing rung's.
///
/// **A `None` RETURN IS THE DANGEROUS DEFAULT, WHICH IS WHY THE PARENT SLOT PANICS RATHER THAN
/// ANSWERING IT** — a rung-40..73 object answering `None` would agree with this body on every
/// input any suite reaches. See `NO_DEMAND_MSG` in [`three_loop`](crate::three_loop).
fn r74_sensed_cap(
    _: &FuelTransientCore, _: &FlightCondition, _: f64, _: f64, _: &AccelSchedule, _: Option<f64>,
) -> Result<Option<f64>, Abort> {
    Ok(None)
}

/// RUNG 74's `_windup_tau` — **`None`, which is § 4's finding stated as code.**
///
/// The clip coordinate has an anti-windup device BY ACCIDENT — rung 52's `max(0, .)`, named for
/// the first time in 22 rungs — and the demand coordinate has NONE, which is why `demand ×
/// applied` has no interior equilibrium here. Rung 75 declares one and this hook is where it
/// enters the march.
///
/// **NOT `Result`, unlike its two cap siblings, and the asymmetry is measured rather than chosen.**
/// Python calls it at `engine.py:17816`, at the top of `_integrate_fuel_demand` and outside every
/// `except AssertionError` in that body, so rung 75's two declared-knob asserts propagate out of
/// the march instead of ending it. A `panic!` is the faithful spelling there; an `Abort` would be
/// caught by the march's own `break` and would silently truncate a trajectory.
fn r74_windup_tau(_: &TwoSpoolTransientCore) -> Option<f64> {
    None
}

/// The set point ABOVE the schedule — **the one quantity this ladder has never computed.**
///
/// `G(w) > 0` means the leg must CUT at `w` (rung 49's sign, and rung 46's after the `Tt4_max`
/// subtraction). When `G(mf_sched) > 0` the leg is BINDING and the **shipped solve is returned
/// untouched**, so on every point where the family has ever consulted a cap this returns the
/// family's own number and nothing is re-bracketed. Only in the SLACK regime does the search run,
/// and there it grows the bracket by [`CAP_GROW`] — the mirror image of `_surge_fuel`'s `0.9`
/// shrink, a direction rather than a new constant.
///
/// **IT ABORTS BY NAME RATHER THAN FALLING BACK**, because the fallback is `mf_sched` and
/// `mf_sched` is precisely the floored cap whose use would MANUFACTURE this rung's own finding.
/// A silent fallback here would be undetectable in every reader.
///
/// # WHY THIS IS AN `Abort` AND NOT A `panic!` — measured at the call site, not chosen
///
/// Python's refusal is an `AssertionError`, and the march wraps its whole derivative in
/// `except AssertionError: break` (`engine.py:17965` and `17989`). A `panic!` here would end the
/// process where Python ends the march and truncates the trajectory. That is slice L's recorded
/// rule — fallibility is a property of the CALL SITE — and it is the difference between reporting
/// a short march and reporting nothing at all.
///
/// # THE LOOP SHAPE IS `try_sched_fuel`'s, INCLUDING THE ARM THAT READS AS REDUNDANT
///
/// `ghi` is reset to `None` after a non-positive reading, so the check after the loop is *"we
/// broke on a sign change"* and not *"we ever evaluated"*. And on the abort arm the loop breaks
/// **after** `hi` has already been multiplied, so the `hi` the message interpolates is the value
/// that raised — both reproduced deliberately.
pub fn cap_free(
    big_g: &dyn Fn(f64) -> Result<f64, Abort>,
    mf_sched: f64,
    shipped: &dyn Fn() -> Result<f64, Abort>,
) -> Result<f64, Abort> {
    let g0 = big_g(mf_sched)?;
    if g0 > 0.0 {
        return shipped();
    }
    let (lo, glo) = (mf_sched, g0);
    let mut hi = mf_sched;
    let mut ghi: Option<f64> = None;
    for _ in 0..CAP_BRACKET_N {
        hi *= CAP_GROW;
        match big_g(hi) {
            Err(_) => {
                ghi = None;
                break;
            }
            Ok(v) => ghi = Some(v),
        }
        if ghi.expect("just assigned") > 0.0 {
            break;
        }
        // Python RESETS to None here, exactly as `try_sched_fuel` does -- the arm that makes the
        // test below "we broke", not "we ever evaluated".
        ghi = None;
    }
    let Some(ghi) = ghi else {
        return Err(Abort(format!(
            "rung-74: the UNFLOORED cap is unreachable above mf_sched = {mf_sched:.6e} \
             (searched to {hi:.6e}). The demand coordinate's target IS this cap; falling back to \
             the floored one would manufacture a dormant-leg cut and report it as a finding \
             (anchor § 0.2). Measured reachable at 341 of 341 anchor points -- if it fails here, \
             the operating point is outside what this rung measured.")));
    };
    try_illinois(|w| big_g(w), lo, hi, glo, ghi, FuelTransientCore::LEG_TOL, ILLINOIS_MAXIT)
}

/// RUNG 47's set point as a DEMAND: the fuel at which `Tt4 == Tt4_max`, computed in the slack
/// regime too.
///
/// `required_gov`'s short-circuit is exactly what has to be removed — it is a guard on the
/// BRACKET, and in this coordinate it is a guard on the ANSWER.
///
/// **NOT A CELL.** One definer in the whole ladder (`engine.py:17604`), measured over all 58
/// classes, so a table field would be a mechanism with no reader — `_with_share`'s case, and the
/// rule those two comments state together.
pub fn cap_gov(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64, tt4_max: f64,
) -> Result<f64, Abort> {
    let big_g = |w: f64| -> Result<f64, Abort> {
        Ok(ft.try_instant_fuel(flight, a, h, w)?.base.tt4 - tt4_max)
    };
    cap_free(&big_g, mf_sched, &|| ft.try_topping_fuel(flight, a, h, tt4_max, mf_sched))
}

/// RUNG 74's `_cap_fuel` — **RUNG 52's LEG AS A DEMAND: the minimum of its (up to two) unfloored
/// set points**, which is min-select one level down and is rung 48/49's own `min(caps)`.
///
/// # `mf_app` IS RUNG 76's ARGUMENT AND IS IGNORED HERE
///
/// Only a cap that DEPENDS on the fuel it is asked about can read it, and this rung has none. The
/// phi leg never reads it in any rung — a floor on a state is not a formula for a fuel (rung 76
/// § 0.1). It is threaded from the first definer because the cell's width is the family's.
///
/// # THE PHI RESIDUAL READS `phi_lim` **RAW**, AND THE SOURCE SAYS WHY
///
/// `_resolve_floor` returns a plain `SurgeLimiter` BY IDENTITY (rung 60), so on this family's rig
/// the raw read and the resolved one agree — and using the raw value keeps this residual the exact
/// transform of the one `_surge_fuel` actually solves rather than of a nearby one. Rung 79's own
/// `_phi_residual` docstring states this about *the shipped `_cap_fuel`*, i.e. about this body.
/// [`Floor::phi`](crate::fuel_transient::Floor::phi) therefore panics on an incidence floor, which
/// is Python's `AttributeError` at the same place — nothing in the ladder catches one, so nothing
/// may catch this.
///
/// # THE `min` IS PYTHON's, NOT `f64::min`
///
/// `min([x, y])` keeps the FIRST of equals and has no NaN rule; `f64::min` has one. Spelled as the
/// fold `if b < a { b } else { a }` so the tie behaviour is the source's — this family's recorded
/// `min`-of-equals hazard (rung 43's argmin tie at a gap of exactly `0.000e+00`).
///
/// # THE `sensed_cap` CALL IS **DISPATCHED**, NEVER CALLED DIRECTLY
///
/// Python reaches it as `self._sensed_cap(…)`, so an inherited rung-74 reader run on a rung-76
/// machine takes rung 76's body. Calling [`r74_sensed_cap`] here would freeze that dispatch and
/// reproduce slice AE's recorded defect — a census restricted to direct calls scoring a method at
/// zero readers when it has eleven call sites.
#[allow(clippy::too_many_arguments)]
fn r74_cap_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>, mf_app: Option<f64>,
) -> Result<f64, Abort> {
    let mut caps: Vec<f64> = Vec::with_capacity(2);
    if let Some(accel) = accel {
        // THROUGH THE TABLE. Rung 76's body is what a rung-76 machine must take here.
        match (ft.inner.triple_hooks.sensed_cap)(ft, flight, a, h, accel, mf_app)? {
            Some(sensed) => caps.push(sensed),
            None => {
                let pi_b = ft.inner.inner.base.pi_b;
                let big_ga = |w: f64| -> Result<f64, Abort> {
                    let i = ft.try_instant_fuel(flight, a, h, w)?;
                    Ok(w - accel.cap(i.base.close.n_hp, i.base.close.pt4 / pi_b))
                };
                caps.push(cap_free(
                    &big_ga, mf_sched,
                    &|| ft.try_sched_fuel(flight, a, h, mf_sched, accel))?);
            }
        }
    }
    if let Some(surge) = surge {
        let phi = surge.phi();
        let big_gs = |w: f64| -> Result<f64, Abort> {
            Ok(phi.phi_lim - phi.read(&ft.try_instant_fuel(flight, a, h, w)?))
        };
        caps.push(cap_free(
            &big_gs, mf_sched, &|| ft.try_surge_fuel(flight, a, h, mf_sched, surge))?);
    }
    Ok(caps.into_iter()
        .reduce(|x, y| if y < x { y } else { x })
        .unwrap_or(f64::INFINITY))
}

// ---------------------------------------------------------------------------------------------
// THE DEMAND LAWS — § 1 IN THE NEW COORDINATE, AND SIX BODIES THAT ARE **NOT** CELLS
// ---------------------------------------------------------------------------------------------
//
// An AST census over all 58 `engine.py` classes settles the shape of this step before a line of it
// is written: `_applied_demand`, `_demand_target`, `_demand_reference`, `_demand_tau`,
// `_demand_authority` and `_demand_laws` have **exactly one definer each** — this class. So step 2
// adds **no** `TripleHooks` field and pays **none** of step 1's width toll, and a table slot for
// any of the six would be a mechanism with no reader (`_with_share`'s case). The only DISPATCHED
// call inside this section is `_cap_fuel`, which step 1 already wired and which has three definers
// (rungs 74, 78 and 79).
//
// Rung 72's siblings score the same way and are plain functions for the same reason:
// `_applied_clip`, `_authority` and `_quad_laws` are single-definer too.

/// RUNG 74's `_applied_demand` — **MIN-SELECT in the coordinate a fuel control actually uses.**
///
/// The schedule is just another input and the LOWEST demand wins. Identical in VALUE to
/// `mf_sched - max(0, gf, gr)` and **not** identical as a plant, which is § 3.
///
/// # THE `min` IS PYTHON's, NOT `f64::min` — [`r74_cap_fuel`]'s rule at this rung's other fold
///
/// `min(mf_sched, wf, wr)` folds left from `mf_sched` and keeps the FIRST of equals; it has no NaN
/// rule, where `f64::min` returns the non-NaN operand. Spelled as the fold so the tie behaviour is
/// the source's, and the ARGUMENT ORDER is load-bearing for the same reason: `mf_sched` seeds the
/// fold, so a three-way tie returns `mf_sched` and not `wf`.
pub fn applied_demand(mf_sched: f64, wf: f64, wr: f64) -> f64 {
    let mut m = mf_sched;
    if wf < m {
        m = wf;
    }
    if wr < m {
        m = wr;
    }
    m
}

/// RUNG 74's `_demand_target` — **THE LATCH, and the only thing `demand-latched` changes.**
///
/// Capping the target at the schedule is exactly rung 52's `max(0, ·)` seen from the other
/// coordinate.
///
/// # A THREE-VALUED TAG READ BY A TWO-VALUED TEST — the module header's measurement, as code
///
/// [`LAG_COORD_CLIP`] and [`LAG_COORD_DEMAND`] are indistinguishable here **by construction**, and
/// [`LAG_COORD_LATCHED`] is distinguishable only where `cap > mf_sched`. § 5.30 (i) measured that
/// region EMPTY on the one path through [`TripleHooks::with_coord`] (0 of 1 040 and 0 of 624
/// calls) and NON-empty inside `_coord_march` (120 of 2 730 and 139 of 2 732, max ratio
/// **1.3039** — which independently reproduces the class docstring's own *`1.303 * mf_sched` at
/// the start of the ramp*).
///
/// **So a gate on this line that runs only on the interior-filter arm is VOID and a mutation sweep
/// will report it green.** `slice_af_laws.rs` drives the truth table on hand-picked floats, where
/// `cap > mf_sched` is reachable by construction rather than by luck.
pub fn demand_target(t: &TwoSpoolTransientCore, cap: f64, mf_sched: f64) -> f64 {
    if t.lag_coord.get() != LAG_COORD_LATCHED {
        return cap;
    }
    // Python's `min(mf_sched, cap)`: the fold seeds at `mf_sched`, so a tie returns `mf_sched`.
    if cap < mf_sched {
        cap
    } else {
        mf_sched
    }
}

/// RUNG 74's `_demand_reference` — **RUNG 73's hook, TERM FOR TERM, in demand coordinates.**
///
/// ```text
/// req_applied = g_own + req_sched - max(gf, gr)   <=>   w = w_own + cap - mf_app
/// ```
///
/// # THE FLOAT-IDENTITY BRANCH IS LOAD-BEARING FOR RUNG 73's OWN REASON
///
/// When the leg HOLDS, `mf_app == w_own` and this returns `cap` ITSELF, so the authoritative leg's
/// diagonal carries no cancellation and rung 73's `M3`-entry-for-entry claim survives the change of
/// coordinate. `w_own + cap - w_own` is not `cap` in binary floating point — rung 73 measured the
/// `4e-11` it would otherwise put on that diagonal. **Do not epsilonize the `==`.**
///
/// # THIS IS NOT A DISPATCH, AND THE CENSUS IS WHY
///
/// Rung 73's `_reference` IS a cell (two definers) and goes through the table. `_demand_reference`
/// has ONE definer, so `self._demand_reference(…)` can only ever reach this body — and it reads
/// `_ref_law` DIRECTLY rather than delegating to `R73_TRIPLE`'s `reference`, whose signature is the
/// clip coordinate's. A port that routed this through the table would be running rung 73's
/// `_applied_clip` on demands.
///
/// **PYTHON's ASSOCIATION IS PINNED**: `(w_own + cap) - mf_app`, never `cap + (w_own - mf_app)`.
/// Rung 73's probe L4 measured the two disagreeing at `1e16`-scale arguments.
pub fn demand_reference(t: &TwoSpoolTransientCore, cap: f64, w_own: f64, mf_app: f64) -> f64 {
    if t.ref_law.get() != REF_LAW_APPLIED {
        return cap;
    }
    if mf_app == w_own {
        return cap;
    }
    (w_own + cap) - mf_app
}

/// RUNG 74's `_demand_tau` — **RUNG 52's asymmetric lag, ARGUMENTS SWAPPED, and the swap is the
/// whole point.**
///
/// Attack in clip coordinates is `required > g`. Substituting `w = mf_sched - g` and
/// `cap = mf_sched - required` gives `required > g  <=>  cap < w`, so the shipped call is
/// `lag.tau(w, cap)` with the DEMAND in the `required` slot and the CAP in the `g` slot.
///
/// # THE TRAP, NAMED BY THE SOURCE ITSELF
///
/// A port that kept the shipped argument ORDER — `lag.tau(cap, w)` — selects
/// [`tau_rel`](AsymmetricLag::tau_rel) on ATTACK. On this family's rig `tau_rel = 3 * tau_att`, so
/// it is a **3x clock error in the direction that SLOWS protection**, and it would have read as a
/// finding (*the demand coordinate is less protective*) rather than as a bug. It is gated
/// two-sided on a known-attack and a known-release point, against the LITERAL constants the test
/// sets — never against [`AsymmetricLag::tau`], which is the code under test.
pub fn demand_tau(lag: &AsymmetricLag, cap: f64, w: f64) -> f64 {
    lag.tau(w, cap)
}

/// The `tol` of [`demand_authority`] — Python's `tol: float = 1e-12` default, named so the gate can
/// straddle it rather than restate it.
pub const DEMAND_AUTH_TOL: f64 = 1e-12;

/// RUNG 74's `_demand_authority` — **rung 72's label in the new coordinate, and BOTH senses
/// invert.**
///
/// Who holds the actuator is who DEMANDS LEAST, and `dormant` is now a statement about the
/// SCHEDULE (neither leg is below it) rather than about a state sitting on a stop — which is § 3's
/// finding in one method.
///
/// # IT MAY NOT DELEGATE TO RUNG 72's `authority`, AND THE DIFF IS TWO LINES, NOT ONE
///
/// | | rung 72, on clips | rung 74, on demands |
/// |---|---|---|
/// | dormant | `gf <= tol && gr <= tol` | `wf >= mf_sched - tol && wr >= mf_sched - tol` |
/// | holder | `fuel` iff `gf > gr` | `fuel` iff `wf < wr` |
///
/// The tie test is the only line the two share. **The BRANCH ORDER is the content**: `dormant` is
/// tested FIRST, so a point that is both tied and at/above the schedule returns `dormant` and not
/// `tie` — reversing the two is a silent relabel that no aggregate over a march would show.
pub fn demand_authority(wf: f64, wr: f64, mf_sched: f64) -> Authority {
    if wf >= mf_sched - DEMAND_AUTH_TOL && wr >= mf_sched - DEMAND_AUTH_TOL {
        return Authority::Dormant;
    }
    if (wf - wr).abs() <= DEMAND_AUTH_TOL {
        return Authority::Tie;
    }
    if wf < wr {
        Authority::Fuel
    } else {
        Authority::Gov
    }
}

/// The FOUR control laws of § 1 in DEMAND coordinates, as closures of the other three states —
/// what [`demand_laws`] returns.
///
/// [`QuadLaws`](crate::shared_actuator::QuadLaws) one coordinate over, and the SIGNATURES differ
/// where the coordinate makes them: rung 72's `F` takes `(gr, q, v)` and `R` takes `(gf, q, v)`,
/// each blind to its own leg because each solves from the SCHEDULED fuel. Here **both take
/// `(wf, wr, q, v)`** — not because the caps changed (they did not; `_cap_fuel` ignores `mf_app` at
/// this rung) but because the REFERENCE reads both demands. That is the only route by which `F`
/// depends on `wf`, and it is exactly what `F_f` differences.
#[allow(clippy::type_complexity)]
pub struct DemandLaws<'a> {
    /// **F** — rung 52's leg as a demand, `(wf, wr, q, v) -> (w, regime)`.
    pub f: Box<dyn Fn(f64, f64, f64, f64) -> Result<(f64, LegRegime), Abort> + 'a>,
    /// **R** — rung 47's governor as a demand, `(wf, wr, q, v) -> (w, regime)`.
    pub r: Box<dyn Fn(f64, f64, f64, f64) -> Result<(f64, LegRegime), Abort> + 'a>,
    /// **C** — the VALVE law, `(wf, wr, v) -> (b, regime)`.
    pub c: Box<dyn Fn(f64, f64, f64) -> Result<(f64, Regime), Abort> + 'a>,
    /// **V** — the STATOR law, `(wf, wr, q) -> (v, regime)`.
    pub v: Box<dyn Fn(f64, f64, f64) -> Result<(f64, Regime), Abort> + 'a>,
}

/// RUNG 74's `_demand_laws` — **the four laws as closures of the other three states, none knowing
/// the others exist.**
///
/// # `C` AND `V` TAKE THE DEMAND AS THE FUEL, NOT `mf_sched - clip`
///
/// This is the one place the coordinate changes an ARGUMENT rather than a label. Rung 72 spells the
/// closed-loop fuel `max(1e-9, mf_sched - _applied_clip(gf, gr))`; here it is
/// `max(1e-9, _applied_demand(wf, wr, mf_sched))`, because in this coordinate the applied demand IS
/// the fuel. The two agree in value on the interior and are different plants at the edges, which is
/// § 3. A port carrying rung 72's spelling forward would converge the valve and stator solves on a
/// residual this rung never uses — rung 62's `_powers` failure mode, and invisible wherever
/// `_applied_demand` happens to equal `mf_sched - max(gf, gr)`.
///
/// Under MIN-SELECT the masked demand reaches `C` and `V` through a function that is FLAT in it —
/// `min()` where rung 72 had `max()` — which is § 1's whole reason the triangularity survives a
/// change of coordinate.
///
/// # THE REGIME LABEL READS THE UNLATCHED `cap`, AND THAT IS NOT A SLIP
///
/// Both `F` and `R` label `riding` from `cap < mf_sched` and **not** from the latched target. After
/// [`demand_target`] the latched target satisfies `tgt <= mf_sched` unconditionally, so a port
/// reading `tgt` would be correct on every point of the interior-filter arm (§ 5.30 (i): 1 040 of
/// 1 040 and 624 of 624) and wrong only over-schedule. It is gated where the two differ.
///
/// # THE `b_state` / `v_state` BOUNDARY IS INHERITED VERBATIM
///
/// Rung 68's table, unchanged: a law that TRIALS an actuator must not see that actuator's state and
/// MUST see the other two. `F` and `R` trial neither and set BOTH; `C` trials `b` and sets
/// `v_state` only; `V` trials `v` and sets `b_state` only. Both guards write `None` on the way out,
/// which is Python's `finally`, and the SCOPE is Python's too — the caps are computed inside the
/// guard and the reference and the label outside it.
///
/// # `F` GOES THROUGH THE TABLE AND `R` DOES NOT, AND THE CENSUS IS THE REASON
///
/// `_cap_fuel` has three definers (rungs 74, 78, 79), so an inherited rung-74 reader run on a
/// rung-79 machine must take rung 79's body — hence [`TripleHooks::cap_fuel`].
///
/// # AND `C` CALLS `_solve_b` DIRECTLY WHILE `V` GOES THROUGH THE TABLE — **CENSUSED, NOT COPIED**
///
/// That asymmetry is inherited from rung 72's `_quad_laws` spelling, which is not a warrant: a
/// frozen dispatch is invisible at the rung that owns it, because that rung is the only machine a
/// slice instantiates. So the four helpers this body reaches were censused over all 58 classes
/// alongside the six laws themselves:
///
/// | helper | definers | so |
/// |---|---|---|
/// | `_solve_b`, `_closer` (`LimitedBleedTransient`) | **1** | direct calls, and a table slot would be dead |
/// | `_closer_v` (`ThreeLoopCascadeTransient`) | **1** | the same |
/// | `_cap_gov`, `_cap_free` (this class) | **1** | the same |
/// | **`_solve_v`** (`ThreeLoopCascadeTransient`, `ReferenceSplitTransient`) | **2** | **which is exactly why it IS a cell** |
///
/// So the asymmetry is the census's and not the sibling's, and `_cap_gov`'s single definer is
/// re-derived from the source rather than inherited from step 1's sentence.
#[allow(clippy::too_many_arguments)]
pub fn demand_laws<'a>(
    core: &'a ScheduledStatorCore, flight: &'a FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&'a AccelSchedule>, surge: Option<&'a Floor>, tt4_max: f64,
) -> DemandLaws<'a> {
    let ft = &core.fuel;
    let (tt2, pt2, _) = ft.inner.inlet(flight);

    // **F** — RUNG 52's leg as a DEMAND. `ma` is formed BEFORE the state guards, which is Python's
    // order; the cap is solved INSIDE them and the reference and the label outside.
    let f = move |wf: f64, wr: f64, q: f64, v: f64| -> Result<(f64, LegRegime), Abort> {
        let ma = applied_demand(mf_sched, wf, wr);
        let cap = {
            let _sb = MarchedBleed::set(&ft.inner, q);
            let _sv = MarchedStator::set(&ft.inner, v);
            // THROUGH THE TABLE — rung 78's and rung 79's bodies are what their machines must take.
            (ft.inner.triple_hooks.cap_fuel)(ft, flight, a, h, mf_sched, accel, surge, Some(ma))?
        };
        let tgt = demand_target(&ft.inner, cap, mf_sched);
        Ok((
            demand_reference(&ft.inner, tgt, wf, ma),
            if cap < mf_sched { LegRegime::Riding } else { LegRegime::Dormant },
        ))
    };

    // **R** — RUNG 47's governor as a DEMAND. `_cap_gov` is single-definer, so it is a direct call;
    // and note Python re-forms the applied demand INLINE in the reference argument rather than
    // reusing a name, so `R` and `F` reach it by two different spellings of one expression.
    let r = move |wf: f64, wr: f64, q: f64, v: f64| -> Result<(f64, LegRegime), Abort> {
        let cap = {
            let _sb = MarchedBleed::set(&ft.inner, q);
            let _sv = MarchedStator::set(&ft.inner, v);
            cap_gov(ft, flight, a, h, mf_sched, tt4_max)?
        };
        let tgt = demand_target(&ft.inner, cap, mf_sched);
        Ok((
            demand_reference(&ft.inner, tgt, wr, applied_demand(mf_sched, wf, wr)),
            if cap < mf_sched { LegRegime::Riding } else { LegRegime::Dormant },
        ))
    };

    // **C** — the VALVE law: it trials `b`, so NO `b_state`, but `v_state = v`.
    let c = move |wf: f64, wr: f64, v: f64| -> Result<(f64, Regime), Abort> {
        let _sv = MarchedStator::set(&ft.inner, v);
        let bl = ft.inner.lever.lim.expect("rung-74's valve law on an unfloored machine");
        // `1e-9f64.max(..)` is rung 72's own spelling at the identical Python `max(1e-9, ·)`, kept
        // rather than re-derived — a deliberate duplication is not a factoring opportunity.
        //
        // **AND IT IS THE ONE `min`/`max` CELL THIS STEP DID NOT DECIDE.** Every other fold here is
        // spelled Python's way because the two disagree on a NaN operand (`applied_demand`'s gate
        // drives exactly that row). Python's `max(1e-9, x)` returns `x` for a NaN `x`; Rust's
        // `1e-9f64.max(x)` returns `1e-9`. Nothing at this rung shows `x` cannot be NaN, and
        // nothing here measures that it can — so this is an UNMEASURED cell, not a decided one,
        // and it is named rather than left to look like the rest.
        let (_, b, reg) = crate::limited_bleed::r64_solve_b(
            &bl,
            closer_b(ft, a, h, 1e-9f64.max(applied_demand(mf_sched, wf, wr)), tt2, pt2))?;
        Ok((b, reg))
    };

    // **V** — the STATOR law: the exact mirror, trialling `v` with `b_state = q`.
    let v = move |wf: f64, wr: f64, q: f64| -> Result<(f64, Regime), Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let (_, vv, reg) = ft.inner.solve_v(&closer_v(
            ft, a, h, 1e-9f64.max(applied_demand(mf_sched, wf, wr)), tt2, pt2))?;
        Ok((vv, reg))
    };

    DemandLaws { f: Box::new(f), r: Box::new(r), c: Box::new(c), v: Box::new(v) }
}

// ---------------------------------------------------------------------------------------------
// THE FOUR RE-AIMED BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 74's `at_lever` — **rung 73's sibling constructor returning a RUNG-74 machine THAT CARRIES
/// BOTH DECLARED LAWS.**
///
/// Thirteenth instance of the trap. The class is one part of the fix and the two laws are the
/// other: a sibling built while the receiver sits under a [`CoordScope`]-set `"demand"` must be
/// `"demand"`, so the value is copied from the SOURCE core and never left at the class default —
/// [`r73_at_lever`](crate::applied_reference)'s reason with a second knob on it.
///
/// **`_ic_cap` IS NOT CARRIED HERE, AND THAT IS PYTHON's LINE AND NOT AN OMISSION.** Rung 74's
/// `at_lever` copies `_ref_law` and `_lag_coord` only; rung 75's copies `_windup_law`, `_tau_t` and
/// `_ic_cap` as well (`engine.py:17711` against `18671`). At this rung nothing writes `_ic_cap`, so
/// source and sibling both read the declared `60` and the copy would be invisible — which is the
/// same argument that keeps it out of [`build_demand_coordinate_cascade`].
fn r74_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let m = match build_demand_coordinate_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    };
    m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
    m.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
    m
}

/// RUNG 74's `_shared_rig` — **rung 73's rig with the COORDINATE carried too, a third knob on.**
///
/// **PRE-REGISTERED AS A NO-OP, AND THE PREDICTION IS DRIVEN RATHER THAN ASSERTED.** Rung 72's
/// body — which rung 73's delegates to — reaches its sibling through `self.at_lever(…)`, which on
/// a rung-74 receiver is [`r74_at_lever`], which has already copied both laws. So the same
/// structural argument slice AE's probe L2 made for `_ref_law` applies here to `_lag_coord`, and
/// `tests/slice_af_cells.rs` drives it the same way: call the PARENT's `shared_rig` directly on a
/// rung-74 receiver and compare the coordinate on the machine it hands back.
///
/// Ported unchanged regardless — a duplication the source makes is not the port's to remove
/// ([[rust-port-copy-vs-rederivation]]), and the belt-and-braces set is what keeps the carrying
/// true if a later rung's `at_lever` stops doing it.
fn r74_shared_rig(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let (m, surge, lag) = (crate::applied_reference::R73_TRIPLE.shared_rig)(core, arm);
    m.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
    (m, surge, lag)
}

/// RUNG 74's `_rk4_floor_shared` — **the floor, re-justified a SEVENTH time, and the previous six
/// do NOT carry.**
///
/// Rungs 72 and 73 both argued from the masked leg's EIGENVALUE — a bare pole at `-1/tau_f`, then
/// a pole exactly at the origin. Neither sentence is the one needed here, and the reason is this
/// rung's own § 1: **a coordinate does not change a rate**, so whichever reference is set that
/// argument carries verbatim and says nothing new. What IS new is WHICH STATES ARE LIVE — removing
/// the state floor makes a dormant leg an ACTIVE first-order lag over the whole march instead of a
/// state parked at a stop. A parked state contributes no root at all, so the rate sum this constant
/// bounds is if anything better populated here, and the forcing term is state-independent and
/// cannot move a root.
///
/// **THE CONDITION IS `ds * rate <= 2.0` IN RUNGS 72, 73 AND 74 CHARACTER FOR CHARACTER, SO THE
/// MESSAGE IS THE ENTIRE CELL** — and the shipped Python needle is useless here for the reason
/// § 5.29 (vii) measured: `"FOUR actuator states"` reaches nine classes back to rung 43, and all
/// three of these messages carry it. The tokens a gate may read are `rung-74` and `ACTIVE lag`.
fn r74_rk4_floor_shared(ds: f64, rate: f64) {
    assert!(
        ds * rate <= 2.0,
        "rung-74: ds*sum(1/tau_i) = {:.3} is outside the explicit RK4 stability region for the \
         FOUR actuator states (ds = {}). A coordinate does not change a rate, so the inherited \
         constant carries -- but removing the state floor makes a dormant leg an ACTIVE lag, so \
         more of these rates are live at once. Refine the grid or slow a clock.",
        ds * rate, ds);
}

/// RUNG 74's `integrate_fuel` — **FIVE REFUSALS SPLIT ACROSS A DISPATCH, AND THE SPLIT IS THE
/// POINT OF THIS STEP.**
///
/// # ONE FIRES ABOVE THE ENTRY TEST AND FOUR BELOW IT — measured from the source, not assumed
///
/// Python's order (`engine.py:17759`–`17787`): the declared-coordinate assert fires FIRST; then
/// `lag` and `tau_gov` resolve; then the `clip or tau_gov is None or not has_fuel` early return to
/// `super()`; then the other four. Slice AE's rung-73 body has BOTH its asserts above the entry
/// test (probe L5) and the port hoisted both, so *hoist the refusals* is the inherited habit and it
/// is **wrong here**:
///
/// * hoisting all five raises on arms Python passes — a `clip` machine with no governor clock is
///   rung 73 and legal, and the `share_law == "max"` assert would reject a legal `sum` run;
/// * sinking all five skips the coordinate refusal on exactly the `clip` arm, where an undeclared
///   coordinate would then dispatch quietly into rung 73.
///
/// Both failures are silent, which is why `tests/slice_af_cells.rs` drives both halves rather than
/// only the refusals themselves.
///
/// # THE `clip` ARM IS EXACT BY DISPATCH — P3
///
/// This class never intercepts a march it does not own: `clip` is rung 73 by **not entering at
/// all**, so none of this rung's march lines execute and the reduce is not a tolerance. § 5.30 (i)
/// measured 0 `_demand_target` calls on that arm, which is the same fact from the other side.
///
/// # THE DEMAND ARM — [`r74_integrate_fuel_demand`], LANDED AT STEP 3
///
/// Step 1 shipped this arm as an `unimplemented!` that panicked by name, because the most
/// dangerous thing that step could have shipped is a demand arm quietly delegating to the parent:
/// it would pass every reduce gate in the crate, since the reduce IS *rung 74 under `clip` is rung
/// 73*. **That obligation did not expire when the body arrived, it changed shape** — the gate that
/// asserted the panic was REACHED now asserts the march RAN and produced a trajectory the parent
/// could not have produced. See `tests/slice_af_march.rs`.
fn r74_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    let coord = ft.inner.lag_coord.get();
    assert!(LAG_COORDS_DECLARED.contains(&coord),
            "rung-74: the lag's COORDINATE is this rung's subject and it is DECLARED; got \
             {coord:?}. 'clip' is rung 73/72; 'demand' is the plant; 'demand-latched' is § 3's \
             instrument.");
    let lag = lim.lag.or_else(|| ft.inner.lag.get());
    let tau_gov = lim.tau_gov.or_else(|| ft.inner.tau_gov.get());
    // `lim.floor()` and not `lim.surge`: Python's ONE `surge=` argument is split into a phi floor
    // and an incidence floor here, and reading only the first would make an incidence-floored
    // machine take the `clip` arm silently. Rung 72's own spelling, inherited.
    let has_fuel = lag.is_some() && (lim.accel.is_some() || lim.floor().is_some());
    if coord == LAG_COORD_CLIP || tau_gov.is_none() || !has_fuel {
        // EXACT DISPATCH: rung 73 and everything under it. The RESOLVED `lag` and `tau_gov` go
        // down with it, which is Python's `super().integrate_fuel(..., tau_gov=tau_gov, lag=lag)`
        // and rung 72's own spelling.
        //
        // **AND IT IS A MEASURED NO-OP, WHICH THIS COMMENT USED TO DENY.** It said forwarding
        // `lim` unchanged would send `None` where the instance attribute had supplied a clock and
        // the parent would march a different plant. That is FALSE: rung 73 passes `lim` straight
        // through and rung 72 re-resolves both with the identical `or_else`
        // (`shared_actuator.rs:502`/`506`), so the two spellings agree on every input. The
        // mutation that drops the resolution SURVIVES all 17 gates, which is how the claim was
        // caught — a survivor is a question, not a verdict. Kept because the source spells it
        // ([[rust-port-copy-vs-rederivation]]) and because a later rung that stopped re-resolving
        // would make it load-bearing; pre-registered as having NO value break.
        return (crate::applied_reference::R73_FUEL.integrate_fuel)(
            ft, flight, fuel_schedule, nu0, s_end, ds,
            &FuelLimiters { tau_gov, lag, ..lim.clone() });
    }
    assert!(ft.inner.share_law.get() == crate::shared_actuator::SHARE_LAW_DEFAULT,
            "rung-74: the DEMAND coordinate composes as `min(mf_sched, wf, wr)`, which has no \
             `sum` reading that keeps the schedule as an input. Marching it would swap two \
             declared laws at once -- rung 73's refusal of `applied x sum`, verbatim.");
    assert!(lim.tt4_max.is_some(),
            "rung-74: `tau_gov` without `Tt4_max` is a governor with no set point (rungs \
             70/71/72/73's assert, inherited word for word).");
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-74: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger; all four legs here pin their own.");
    assert!(ft.inner.lever.lim.is_none() || crate::lagged_bleed::lagged(&ft.inner),
            "rung-74: an INSTANTANEOUS valve beside lagged fuel-side legs is not a control but a \
             different plant (rung 65/66's refusal, inherited).");
    r74_integrate_fuel_demand(
        ft, flight, fuel_schedule, nu0, s_end, ds, lim.freeze,
        lim.tt4_max.expect("asserted above"), tau_gov.expect("the entry test returned if None"),
        lim.accel, lim.floor(), lag.as_ref().expect("has_fuel"))
}

/// One RK4 stage of rung 74's march — [`SharedDer`](crate::shared_actuator) one coordinate over.
///
/// **`mf` IS THE CLAMPED APPLIED DEMAND AND NOT THE RAW ONE, AND PYTHON KEEPS BOTH NAMES ALIVE.**
/// `der` computes `mf_app = _applied_demand(...)`, uses THAT for the two caps and both references,
/// and only then binds a SECOND name `mf = max(1e-9, mf_app)` for the instant solve, the valve, the
/// stator and the recorded point. A port that clamped in place would feed the clamped value back
/// into `_demand_reference` and change the plant wherever the demand fell below `1e-9`.
struct DemandDer {
    da: f64,
    dh: f64,
    dwf: f64,
    dwr: f64,
    dq: f64,
    dv: f64,
    /// Python's `mf` — `max(1e-9, mf_app)`, which is what the recorded point carries.
    mf: f64,
    inst: FuelInstant,
    /// The two LATCHED targets, `cf` / `cr`. The point records these under `cap_fuel` / `cap_gov`.
    cf: f64,
    cr: f64,
    cmd: f64,
    vcmd: f64,
    /// `None` where no stator is armed — Python's `(0, None)`.
    vreg: Option<Regime>,
    /// The schedule at this stage's `s`. Python reads it once per `der` call and the recorded
    /// point takes `k1`'s, never a second evaluation.
    ms: f64,
}

/// RUNG 74's MARCH — **rung 72/73's six states, with the two fuel-side ones carrying the DEMAND
/// instead of the CLIP.**
///
/// # IT IS A SIBLING, AND HERE THAT IS FORCED FOR A NEW REASON
///
/// Rung 72 sired one because a STATE was added; nothing is added here, and the temptation is
/// therefore to add the forcing term `d(mf_sched)/ds` to the parent's march and be done. **That
/// would make § 1 a construction**: the identity `dg/ds = (req - g)/tau + d(mf_sched)/ds` is what
/// this rung CLAIMS, and a march built on it could not measure it. `w` is marched as a genuine
/// state, the schedule is never differentiated, and the identity is left to be differenced against
/// the parent — which also handles the ramp's two KINKS exactly, where a derivative would have had
/// to pick a branch.
///
/// In Rust the temptation is stronger than in Python, because [`r72_integrate_fuel_shared`] is
/// ~90% identical and one file away. It is NOT parameterised to serve both
/// ([[rust-port-copy-vs-rederivation]]): a shared marcher would put the rung's own claim inside
/// the code that is supposed to test it.
///
/// [`r72_integrate_fuel_shared`]: crate::shared_actuator
///
/// # THE PARENT's LAST TWO LINES ARE **NOT** COPIED, AND THAT IS THE ONE THING THAT MUST NOT CARRY
///
/// Rung 72 ends every step with `gf = max(0, gf); gr = max(0, gr)` — rung 52's floor on the STATE.
/// Rung 74 **replaces** that pair with a conditional clamp:
///
/// ```text
/// if latched:   nxt = fuel_schedule(s + ds);   wf, wr = min(nxt, wf), min(nxt, wr)
/// ```
///
/// Under plain `"demand"` **there is no state stop at all** — § 4's *no interior equilibrium*, and
/// what rung 75's device exists to supply. Carrying the parent's floors forward would hand the
/// unlatched arm an anti-windup device by accident, make the rung's central finding unmeasurable,
/// and **pass every reduce gate in the crate**, because the `clip` arm never enters this function.
/// The `q` and `v` hardware stops above it DO carry verbatim: those are metal, not a coordinate.
///
/// Two details of the clamp are load-bearing and neither is the obvious spelling: the schedule is
/// read at `s + ds` (the NEXT point, not this one), and Python's `min(nxt, w)` seeds the fold at
/// `nxt`, so a tie returns `nxt`.
///
/// # THREE PLACES WHERE THE PARENT's CONSTANT IS A CELL HERE
///
/// * **`ic_cap`.** Rung 72 hardcodes `1..=60`; Python reads `self._ic_cap`, whose only writer in
///   the ladder is rung 75's `_with_ic_cap`. This step is where that field gains its first reader.
///   It is **not** unobservable at rung 74: on the converging arms the sweep settles in 2 passes
///   with `ic_res` exactly `0.0`, so 60 and 1000 are identical — but at `ic_cap = 1` the same arm
///   RAISES, and on the `demand × applied` arm (which never converges) the refusal's own iteration
///   count tracks the cap exactly. Measured, not assumed.
/// * **`tau_t`.** `self._windup_tau()` is a DISPATCHED call and goes through
///   [`TripleHooks::windup_tau`]. Inlining `None` would freeze rung 75's dispatch — the defect
///   step 2 § (f) censused against, one call over.
/// * **`lag_coord`.** Read once, before the loop, exactly as Python binds `latched`.
///
/// # `tau_t` LEAVES THREE DEAD SITES AT THIS RUNG, PRE-REGISTERED AS SUCH
///
/// `windup_tau` returns `None` at rung 74, so the `2.0 / tau_t` term in the RK4 rate sum, the two
/// back-calculation lines in `der`, and [`relax`]'s far branch are all UNREACHABLE here. A mutation
/// deleting any of the three SURVIVES, and that is predicted with its proof rather than discovered
/// in the sweep (step 2 § (c)'s lesson). They are ported because Python has them and because rung
/// 75 is the reader that makes them live.
///
/// # EVERY RECORDED KEY IS THE PARENT's, PLUS FIVE
///
/// `g_fuel` / `g_gov` are the CLIP PROJECTIONS `mf_sched - w` and `g` is `mf_sched - mf_app`, so
/// every inherited reader works on this trajectory unchanged — see
/// [`PointExtra::Demand`](crate::fuel_transient::PointExtra::Demand) for the thirty-one arms that
/// claim makes load-bearing, and for the SIGN it changes under them.
#[allow(clippy::too_many_arguments)]
fn r74_integrate_fuel_demand(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, freeze: Option<Spool>, tt4_max: f64, tau_gov: f64,
    accel: Option<&AccelSchedule>, surge: Option<Floor>, lag: &AsymmetricLag,
) -> Vec<FuelPoint> {
    let has_v = ft.inner.lagged_stator();
    let lim_s = if has_v { ft.inner.stator_leg() } else { None };
    let tau_s = lim_s.and_then(|l| l.tau);
    let has_q = crate::lagged_bleed::lagged(&ft.inner);
    let tau_q = if has_q { ft.inner.lever.lim.expect("has_q").tau } else { None };
    // RUNG 75's ONE HOOK INTO THIS MARCH, and it is DISPATCHED. `None` here is rung 74, and every
    // branch guarded by it is NOT TAKEN — so this rung's floats are untouched by construction and
    // the inherited bit-for-bit gates are what say so.
    let tau_t = (ft.inner.triple_hooks.windup_tau)(&ft.inner);
    // PYTHON's OWN SUMMATION ORDER: governor, fuel lag, valve, stator, then rung 75's pair.
    (ft.inner.triple_hooks.rk4_floor_shared)(
        ds,
        1.0 / tau_gov + 1.0 / lag.tau_att.min(lag.tau_rel)
            + (if has_q { 1.0 / tau_q.expect("has_q") } else { 0.0 })
            + (if has_v { 1.0 / tau_s.expect("has_v") } else { 0.0 })
            + (if let Some(t) = tau_t { 2.0 / t } else { 0.0 }));
    let (tt2, pt2, _) = ft.inner.inlet(flight);
    let latched = ft.inner.lag_coord.get() == LAG_COORD_LATCHED;

    // THE VALVE law — rung 72's, verbatim.
    let command = |a: f64, h: f64, mf: f64, v: f64| -> Result<f64, Abort> {
        if !has_q {
            return Ok(0.0);
        }
        let _sv = MarchedStator::set(&ft.inner, v);
        let bl = ft.inner.lever.lim.expect("has_q");
        Ok(crate::limited_bleed::r64_solve_b(&bl, closer_b(ft, a, h, mf, tt2, pt2))?.1)
    };

    // THE STATOR law — rung 72's, verbatim, including the stator-less constant `(0, None)`.
    let stator = |a: f64, h: f64, mf: f64, q: f64| -> Result<(f64, Option<Regime>), Abort> {
        if !has_v {
            return Ok((0.0, None));
        }
        let _sb = MarchedBleed::set(&ft.inner, q);
        let (_, v, reg) = ft.inner.solve_v(&closer_v(ft, a, h, mf, tt2, pt2))?;
        Ok((v, Some(reg)))
    };

    // THE TWO CAPS, each inside BOTH state guards — Python's `cap_fuel` / `cap_gov` closures,
    // whose `finally` writes `None` to both, which is what the two `Drop`s do.
    let cap_fuel_at = |a: f64, h: f64, q: f64, v: f64, mf_sched: f64, mf_app: f64|
     -> Result<f64, Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let _sv = MarchedStator::set(&ft.inner, v);
        // THROUGH THE TABLE: rungs 78 and 79 redefine `_cap_fuel`, so a march inherited by one of
        // their machines must take their body.
        (ft.inner.triple_hooks.cap_fuel)(
            ft, flight, a, h, mf_sched, accel, surge.as_ref(), Some(mf_app))
    };
    let cap_gov_at = |a: f64, h: f64, q: f64, v: f64, mf_sched: f64| -> Result<f64, Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let _sv = MarchedStator::set(&ft.inner, v);
        // A DIRECT call: `_cap_gov` has exactly one definer in the ladder (step 2 § (f)'s census),
        // so a table slot for it would be a mechanism with no reader.
        cap_gov(ft, flight, a, h, mf_sched, tt4_max)
    };

    let der = |a: f64, h: f64, wf: f64, wr: f64, q: f64, v: f64, s: f64|
     -> Result<DemandDer, Abort> {
        let mf_sched = fuel_schedule(s);
        let mf_app = applied_demand(mf_sched, wf, wr);
        let cf = demand_target(&ft.inner, cap_fuel_at(a, h, q, v, mf_sched, mf_app)?, mf_sched);
        let cr = demand_target(&ft.inner, cap_gov_at(a, h, q, v, mf_sched)?, mf_sched);
        let tf = demand_reference(&ft.inner, cf, wf, mf_app);
        let tr = demand_reference(&ft.inner, cr, wr, mf_app);
        // Python's SECOND name. `mf_app` stays RAW for everything above and for rung 75's
        // back-calculation below; only the plant sees the clamp.
        //
        // **`1e-9f64.max(·)` IS RUNG 72's SPELLING AND IT IS THE THIRD OF THE UNMEASURED `max`
        // CELLS STEP 2 NAMED.** Python's `max(1e-9, x)` returns `x` for a NaN `x`; Rust's
        // `1e-9f64.max(x)` returns `1e-9`. Step 3's own measurement closes the reachability half
        // rather than the algebra: over `test_rung74.py`'s three `phi` arms on both demand tags,
        // **0 of 2 046 marched points carry a NaN `mf`**. So the divergence is unreachable on
        // everything this rung ships — recorded as a measurement, not repaired into a difference
        // from the sibling it was copied from.
        let mf = 1e-9f64.max(mf_app);
        let inst = {
            let _sb = MarchedBleed::set(&ft.inner, q);
            let _sv = MarchedStator::set(&ft.inner, v);
            ft.try_instant_fuel(flight, a, h, mf)?
        };
        let cmd = command(a, h, mf, v)?;
        let (vcmd, vreg) = stator(a, h, mf, q)?;
        let da = if freeze == Some(Spool::Lp) { 0.0 } else { inst.base.phi_lp_dot / ft.rho() };
        let dh = if freeze == Some(Spool::Hp) { 0.0 } else { inst.base.phi_hp_dot };
        let mut dwf = (tf - wf) / demand_tau(lag, tf, wf);
        let mut dwr = (tr - wr) / tau_gov;
        if let Some(t) = tau_t {
            // RUNG 75, THE DECLARED DEVICE: back-calculation onto the APPLIED fuel. Identically
            // zero on whichever leg HOLDS the actuator (`mf_app == w_auth`), so it disarms itself
            // on the authoritative leg and acts only on the masked one. DEAD at rung 74.
            dwf += (mf_app - wf) / t;
            dwr += (mf_app - wr) / t;
        }
        Ok(DemandDer {
            da, dh, dwf, dwr,
            dq: if has_q { (cmd - q) / tau_q.expect("has_q") } else { 0.0 },
            dv: if has_v { (vcmd - v) / tau_s.expect("has_v") } else { 0.0 },
            mf, inst, cf, cr, cmd, vcmd, vreg, ms: mf_sched,
        })
    };

    // --- THE JOINT INITIAL CONDITION, IN THE NEW COORDINATE ------------------------------------
    // Rung 72's order (`r -> q -> v -> f`) and its cap, unchanged. The STARTING point is
    // `w = mf_sched` (i.e. `g = 0`, the parent's own start) and the STOP is the LATCH's, applied
    // only when the latch is armed — an unlatched leg has no state stop at all, which is § 3's
    // whole subject and must not be smuggled in through the sweep.
    let (mut a, mut h) = nu0;
    let mf0 = fuel_schedule(0.0);
    let v0 = ft.inner.v0.get();
    if let (Some(x), Some(l)) = (v0, lim_s) {
        ft.inner.check_v0(x, &l);
    }
    // Python raises out of the whole method here — the initial solves sit BEFORE the loop's `try`.
    let raise = |e: Abort| -> ! { panic!("{}", e.0) };
    let (mut wf, mut wr) = (mf0, mf0);
    let mut q = command(a, h, mf0, 0.0).unwrap_or_else(|e| raise(e));
    let mut v = if v0.is_some() && has_v { v0.expect("is_some") } else { 0.0 };
    let b0 = ft.inner.b0.get();
    if let Some(x) = b0 {
        q = x;
    }

    // Python's `_stop`. `min(mf0, w)` seeds the fold at `mf0`, so a tie returns `mf0`.
    let stop = |w: f64| -> f64 {
        if !latched {
            return w;
        }
        if w < mf0 { w } else { mf0 }
    };
    // Python's `_relax` — RUNG 75's fixed point of BOTH terms. `tau_t is None` returns the target
    // ITSELF, so rung 74's sweep is this expression with the branch not taken.
    //
    // **`w` IS A PARAMETER THE BODY NEVER READS**, in Python too: the state enters only through
    // the `tau` the caller computed from it. Kept in the signature because the source has it, and
    // named `_w` so the fact is on the page rather than looking like a dropped argument.
    let relax = |tgt: f64, _w: f64, mf_app: f64, tau: f64| -> f64 {
        match tau_t {
            None => tgt,
            Some(t) => (t * tgt + tau * mf_app) / (tau + t),
        }
    };

    let order = IC_ORDER4_DECLARED;
    // TRANSCRIBED FROM `engine.py:17947`, not from the parent with the number swapped: this is the
    // ONE shipped rung-74 message that does NOT open `rung-74:` — there is no colon after the tag
    // — so a needle built on the pre-flight's *all 9 open with `rung-74:`* would miss it.
    assert!({
                let mut cs: Vec<char> = order.chars().collect();
                cs.sort_unstable();
                cs == ['f', 'q', 'r', 'v']
            },
            "rung-74 ic_order4 is a permutation of 'frqv'; got {order:?}");
    let mut res = f64::INFINITY;
    let mut its = 0usize;
    for i in 1..=ft.inner.ic_cap.get() {
        its = i;
        let (mut wfn, mut wrn, mut qn, mut vn) = (wf, wr, q, v);
        for k in order.chars() {
            match k {
                'f' => {
                    let ma = applied_demand(mf0, wfn, wrn);
                    let tgt = demand_reference(
                        &ft.inner,
                        demand_target(
                            &ft.inner,
                            cap_fuel_at(a, h, qn, vn, mf0, ma).unwrap_or_else(|e| raise(e)),
                            mf0),
                        wfn, ma);
                    wfn = stop(relax(tgt, wfn, ma, demand_tau(lag, tgt, wfn)));
                }
                'r' => {
                    let ma = applied_demand(mf0, wfn, wrn);
                    let tgt = demand_reference(
                        &ft.inner,
                        demand_target(
                            &ft.inner,
                            cap_gov_at(a, h, qn, vn, mf0).unwrap_or_else(|e| raise(e)),
                            mf0),
                        wrn, ma);
                    wrn = stop(relax(tgt, wrn, ma, tau_gov));
                }
                'q' => {
                    if b0.is_none() {
                        qn = command(a, h, 1e-9f64.max(applied_demand(mf0, wfn, wrn)), vn)
                            .unwrap_or_else(|e| raise(e));
                    }
                }
                'v' => {
                    if v0.is_none() && has_v {
                        vn = stator(a, h, 1e-9f64.max(applied_demand(mf0, wfn, wrn)), qn)
                            .unwrap_or_else(|e| raise(e))
                            .0;
                    }
                }
                _ => unreachable!("the permutation assert above admits only f/q/r/v"),
            }
        }
        // Python's `max(abs(n[i] - x) for i, x in enumerate((wf, wr, q, v)))` — the tuple order is
        // `(wf, wr, q, v)` and NOT the sweep order, which is `_ic_order4`'s.
        res = py_max4((wfn - wf).abs(), (wrn - wr).abs(), (qn - q).abs(), (vn - v).abs());
        wf = wfn;
        wr = wrn;
        q = qn;
        v = vn;
        if res <= 1e-12 {
            break;
        }
    }
    assert!(res <= 1e-9,
            "rung-74: the joint initial condition did not converge (residual {res:.3e} after \
             {its} iterations) in order {order:?}, at wf = {wf:.6e}, wr = {wr:.6e}, under \
             ({:?}, {:?}). Under MIN-SELECT the sweep can cycle between the two fuel-side legs \
             (rung 72's reason) -- and under ('demand', 'applied') there is a SECOND, structural \
             reason, which is this rung's s 4: a MASKED applied-referenced leg obeys dw/ds = \
             (cap - mf_app)/tau, which is state-independent and POSITIVE, so with no stop in its \
             path it has NO INTERIOR EQUILIBRIUM AT ALL. The same motion in CLIP coordinates runs \
             INTO the floor at g = 0 and halts there, which is what rung 73 s 0.2 read as \
             self-anti-winding. Neither is a cap to raise: report the state, the order and both \
             demands.",
            ft.inner.lag_coord.get(), ft.inner.ref_law.get());

    // --- THE RK4 LOOP --------------------------------------------------------------------------
    let share_law = ft.inner.share_law.get();
    let lag_coord = ft.inner.lag_coord.get();
    let mut pts: Vec<FuelPoint> = Vec::new();
    let mut s = 0.0f64;
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok(k1) = der(a, h, wf, wr, q, v, s) else { break };
        let ms = k1.ms;
        // Python's `clip = ms - mf_app`, where its `mf_app` at this point is the CLAMPED `mf`
        // returned from `der` — the same name rebound in the caller. So `g` and the recorded `mf`
        // are the same quantity, and `mf = mf_sched - g` stays true for an inherited reader.
        let clip = ms - k1.mf;
        // Python's `max(ms - cf, ms - cr)`, spelled as the fold rather than `f64::max`: the two
        // differ when an operand is NaN and Python's is what the source runs. (Rung 72's port
        // spells the same construct `rf.max(rr)` — a latent divergence at a place no NaN reaches,
        // recorded here rather than silently matched.)
        let required = {
            let (x, y) = (ms - k1.cf, ms - k1.cr);
            if y > x { y } else { x }
        };
        pts.push(crate::fuel_transient::point(
            s, a, h, &k1.inst, k1.mf, ms,
            crate::fuel_transient::PointExtra::Demand {
                g: clip, required, b: q, b_cmd: k1.cmd, v, v_cmd: k1.vcmd, v_regime: k1.vreg,
                ic_iters: its, ic_res: res, ic_order: order,
                g_fuel: ms - wf, g_gov: ms - wr,
                required_fuel: ms - k1.cf, required_gov: ms - k1.cr,
                authority: demand_authority(wf, wr, ms), share_law,
                w_fuel: wf, w_gov: wr, cap_fuel: k1.cf, cap_gov: k1.cr, lag_coord,
            }));
        let stages = (|| -> Result<[f64; 18], Abort> {
            let k2 = der(a + ds / 2.0 * k1.da, h + ds / 2.0 * k1.dh, wf + ds / 2.0 * k1.dwf,
                         wr + ds / 2.0 * k1.dwr, q + ds / 2.0 * k1.dq, v + ds / 2.0 * k1.dv,
                         s + ds / 2.0)?;
            let k3 = der(a + ds / 2.0 * k2.da, h + ds / 2.0 * k2.dh, wf + ds / 2.0 * k2.dwf,
                         wr + ds / 2.0 * k2.dwr, q + ds / 2.0 * k2.dq, v + ds / 2.0 * k2.dv,
                         s + ds / 2.0)?;
            let k4 = der(a + ds * k3.da, h + ds * k3.dh, wf + ds * k3.dwf, wr + ds * k3.dwr,
                         q + ds * k3.dq, v + ds * k3.dv, s + ds)?;
            Ok([k2.da, k2.dh, k2.dwf, k2.dwr, k2.dq, k2.dv,
                k3.da, k3.dh, k3.dwf, k3.dwr, k3.dq, k3.dv,
                k4.da, k4.dh, k4.dwf, k4.dwr, k4.dq, k4.dv])
        })();
        let Ok([k2a, k2h, k2wf, k2wr, k2q, k2v,
                k3a, k3h, k3wf, k3wr, k3q, k3v,
                k4a, k4h, k4wf, k4wr, k4q, k4v]) = stages else { break };
        a += ds / 6.0 * (k1.da + 2.0 * k2a + 2.0 * k3a + k4a);
        h += ds / 6.0 * (k1.dh + 2.0 * k2h + 2.0 * k3h + k4h);
        wf += ds / 6.0 * (k1.dwf + 2.0 * k2wf + 2.0 * k3wf + k4wf);
        wr += ds / 6.0 * (k1.dwr + 2.0 * k2wr + 2.0 * k3wr + k4wr);
        q += ds / 6.0 * (k1.dq + 2.0 * k2q + 2.0 * k3q + k4q);
        v += ds / 6.0 * (k1.dv + 2.0 * k2v + 2.0 * k3v + k4v);
        // The two HARDWARE stops carry from rung 72 verbatim: a valve position and a stator
        // setting are metal, and a change of coordinate on the FUEL lag does not move them.
        if has_q {
            let bmax = ft.inner.lever.lim.expect("has_q").b_max;
            q = bmax.min(0.0f64.max(q));
        }
        if has_v {
            v = ft.inner.clamp_v(v, &lim_s.expect("has_v"));
        }
        if latched {
            // THE LATCH — the clip plant's `g >= 0`, in this coordinate, and the ONLY state stop
            // this march has. Applied here and nowhere else: an unlatched demand has no stop, and
            // that is the finding rather than an oversight (§ 3).
            //
            // **THIS IS WHERE RUNG 72's `gf = max(0, gf)` PAIR WOULD HAVE LANDED IF IT HAD BEEN
            // COPIED**, and it would have been invisible: `w` never goes negative on any arm the
            // suite runs (measured — minimum `9.44e-03` over 2 046 marched points, and 0 of them
            // below zero), so an added `max(0, w)` is arithmetically inert HERE and would still
            // have given the unlatched arm a stop the moment a plant reached one.
            let nxt = fuel_schedule(s + ds);
            // Python's `min(nxt, w)`: the fold seeds at `nxt`, so a tie returns `nxt` and so
            // does a NaN state -- `f64::min` would return the other operand on both rows.
            wf = if wf < nxt { wf } else { nxt };
            wr = if wr < nxt { wr } else { nxt };
        }
        s += ds;
    }
    pts
}

