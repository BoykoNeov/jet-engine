//! RUNG 75 — **THE DECLARED ANTI-WINDUP DEVICE.** `AntiWindupTransient`, slice AG.
//!
//! Rung 74 § 4 found rung 52's `max(0, ·)` to be this family's anti-windup device *by accident*,
//! and found that removing it leaves the masked applied-referenced leg with `dw/ds =
//! (cap - mf_app)/tau > 0` and nothing in its path — no interior equilibrium, so `demand ×
//! applied` HAS NO PLANT. This rung declares the device the accident was standing in for:
//! back-calculation, pulling the masked leg's state toward the fuel actually applied on its own
//! clock `tau_t`.
//!
//! # WHAT STEP 1 OF THIS SLICE ADDS — **FOUR RE-AIMED POINTERS AND NOT ONE NEW TABLE FIELD**
//!
//! | | Python | slot | table |
//! |---|---|---|---|
//! | swap | `_windup_tau` | [`windup_tau`](crate::three_loop::TripleHooks::windup_tau) | [`R75_TRIPLE`] |
//! | swap | `_shared_rig` | [`shared_rig`](crate::three_loop::TripleHooks::shared_rig) | [`R75_TRIPLE`] |
//! | swap | `at_lever` | `LeverHooks::at_lever` | [`R75`] |
//! | swap | `integrate_fuel` | `FuelTransientHooks::integrate_fuel` | [`R75_FUEL`] |
//!
//! # THE SAFETY NET EVERY STEP 1 SINCE SLICE Z HAS HAD IS **ABSENT HERE**, AND THAT SHAPES THE GATE
//!
//! § 5.31 (ii)'s census is `0 ADD`: all 22 method names across rungs 75 and 76 are either
//! single-definer over all 58 classes or already have a [`TripleHooks`] field, because slice AF
//! took that table 14 → 18 and two of its four arrivals are exactly the names this slice needs.
//!
//! **So a missed swap does not fire a width tripwire.** § 5.30's step 1 was safe *because* it
//! widened the struct: forget a cell and the exhaustive literals go `E0063`. Here a forgotten
//! re-aim COMPILES, silently runs the rung-74 parent, and both parents return exactly this
//! slice's reduce-arm answer — [`r74_windup_tau`] returns `None`, `r74_sensed_cap` returns
//! `Ok(None)` — so **every reduce gate in the crate would go on passing**. The instrument that
//! works with no bodies yet is FUNCTION-POINTER IDENTITY, in both directions: each swapped slot
//! must DIFFER from rung 74's, and each inherited slot must be EQUAL to it, so neither a dropped
//! re-aim nor a stray one can hide. `tests/slice_ag_cells.rs` does that for all eight swaps across
//! both rungs and for every inherited slot of all five tables.
//!
//! [`r74_windup_tau`]: crate::demand_coordinate
//!
//! # THE STEP BOUNDARY IS RE-CUT, AND IT IS RECORDED RATHER THAN QUIETLY MOVED
//!
//! § 5.31 (vi)'s list puts `_windup_tau` at step 2 and *both refusal sets* at step 1. **Those two
//! sentences are jointly impossible**: two of rung 75's four asserts ARE `_windup_tau`'s body, and
//! the third exists only to call it early (`engine.py:18650`–`18653`). Shipping the refusals
//! without the cell would mean writing the same two messages twice and deleting one of them a step
//! later. So the cell lands HERE in full, and step 2 keeps the rest of that sentence:
//! `_windup_march`, `_with_windup`, `_rhs_laws`.
//!
//! **This is slice AF's own recurring defect — two claims individually plausible and jointly
//! impossible inside one section — arriving in the next slice's plan**, and it is written down for
//! the same reason AF wrote its own down rather than silently resolving it.
//!
//! # THE DEVICE IS ALREADY THREADED THROUGH RUNG 74's MARCH, AND THIS CELL IS THE SWITCH
//!
//! Slice AF pre-registered three sites in `r74_integrate_fuel_demand` as DEAD at rung 74 with
//! their proofs written first: the `2.0 / tau_t` term in the RK4 rate sum, the two
//! back-calculation lines in `der`, and `relax`'s far branch. All three are guarded by
//! `windup_tau()` returning `Some`, so **this swap is the whole of what makes them live** — which
//! is why the cell could not be inlined at rung 74 and why the port's own comment there says so.
//! It also means step 1 ships a rung-75 plant that MARCHES, and the reduce (`"none"`) is exact by
//! the branch not being taken rather than by tolerance.
//!
//! # `tau_t` IS THE FIRST NEW CONSTANT SINCE RUNG 65, AND IT IS DECLARED
//!
//! It cannot be derived from anything shipped, so rung 75 treats every finding as a property of
//! the SWEEP — the treatment `phi_lim` has had since rungs 36/49. Its fast end is GRID-LIMITED and
//! the bound is arithmetic: the device adds `1/tau_t` to each of the two fuel-side diagonals, so
//! `_rk4_floor_shared` admits `tau_t >= 2*ds / (2 - ds*sum(1/tau_i))`, i.e. `0.00625` at the
//! inherited `ds = 0.005` and four clocks of `0.05`. Perfect tracking is not reachable on this
//! grid and is not claimed — [`WINDUP_TAU_GRID_FLOOR`] carries the derivation, not a typed
//! decimal.

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::demand_coordinate::LAG_COORD_DEMAND;
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AsymmetricLag, Floor, FuelLimiters, FuelPoint, FuelTransientCore, FuelTransientHooks,
};
use crate::map::ComponentMap;
use crate::shared_actuator::SharedRigArm;
use crate::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use crate::three_loop::TripleHooks;
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{TwoSpoolTransientCore, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// THE DECLARED CONSTANTS — rung 75's two class attributes, and one derived bound
// ---------------------------------------------------------------------------------------------

/// Python's `_windup_law = "none"` — **THE CLASS DEFAULT, AND IT IS THE REDUCE ARM.**
///
/// No declared device, which is rung 74 § 4's finding rather than an absence: the clip coordinate
/// has one by accident and the demand coordinate has none at all. It is also the value
/// [`TwoSpoolTransientCore`]'s constructor already writes, so — like `_lag_coord` and unlike
/// `_ref_law` — there is nothing for [`build_anti_windup_cascade`] to overwrite and no
/// silent-wrong-plant failure to gate for.
pub const WINDUP_LAW_NONE: &str = "none";

/// Python's `"track"` — back-calculation onto the APPLIED fuel on the clock `tau_t`.
///
/// `dw/ds = (target - w)/tau + (mf_app - w)/tau_t`, and **the device disarms itself on the leg
/// that holds the actuator**: `mf_app = min(mf_sched, wf, wr)`, so on the authoritative leg
/// `mf_app == w_auth` and the added term is identically zero — not small, zero.
pub const WINDUP_LAW_TRACK: &str = "track";

/// The two laws [`r75_integrate_fuel`]'s first refusal admits, in Python's tuple order.
///
/// Named so the refusal and the gate that drives it read the same list —
/// [`LAG_COORDS_DECLARED`](crate::demand_coordinate::LAG_COORDS_DECLARED)'s reason, one knob over.
pub const WINDUP_LAWS_DECLARED: [&str; 2] = [WINDUP_LAW_NONE, WINDUP_LAW_TRACK];

/// The fast end of the `tau_t` sweep — `2*ds / (2 - ds*sum(1/tau_i))` at the inherited grid.
///
/// **SPELLED AS THE DERIVATION AND NOT AS `0.00625`**, for
/// [`CAP_GROW`](crate::demand_coordinate::CAP_GROW)'s stated reason: it is a consequence of the
/// step and the four clocks, so a typed decimal would hide which of them it answers to and would
/// also be a different float from some other expansion of the same idea.
///
/// It is not a refusal — `_rk4_floor_shared` is — and nothing in the port reads it as one. It
/// exists so a gate and a doc line can name the SAME number, and so the claim *perfect tracking is
/// not reachable on this grid* is a computation rather than a sentence.
pub const WINDUP_TAU_GRID_FLOOR: f64 = 2.0 * 0.005 / (2.0 - 0.005 * (4.0 / 0.05));

// ---------------------------------------------------------------------------------------------
// THE CASCADE BUILDER
// ---------------------------------------------------------------------------------------------

/// Build a rung-75 object, so every sibling re-asserts the whole chain's guards.
///
/// **IT STILL SETS `_ref_law` FOR RUNG 73's REASON**, inherited through
/// [`build_demand_coordinate_cascade`]'s own body rather than copied: `AntiWindupTransient`
/// subclasses down to `AppliedReferenceTransient`, so a fresh object reads `'applied'` while the
/// core's constructor writes `"sched"` for every rung in the family.
///
/// **IT SETS NEITHER `_windup_law` NOR `_tau_t`, AND THAT IS A DECISION.** Rung 75 declares
/// `"none"` and `None`, which are already the core's defaults, so the sets would be no-ops no gate
/// could see — and writing them anyway would put two lines here that look like they are doing the
/// `ref_law` job. What DOES have to carry them is [`r75_at_lever`] and [`r75_shared_rig`], because
/// a sibling built from a receiver that has been armed must inherit the receiver's device and not
/// the class default.
pub fn build_anti_windup_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = crate::reference_split::build_split_family_cascade(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
        &R75_TWO, &R75_STATOR, &R75_FUEL, &R75, &R75_TRIPLE);
    if let ScheduledStatorTransient::Full(c) = &built {
        c.fuel.inner.ref_law.set(crate::applied_reference::REF_LAW_APPLIED);
    }
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and TWO of them carry something of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 75's lever table — ONE swap, `at_lever`, and the parent it must differ from is rung 74's.
///
/// The FOURTEENTH instance of the sibling-constructor trap, now with FOUR knobs to drop instead of
/// three: hand back the parent's class and every reader measures rung 74's plant while reporting
/// the device; hand back the right class while dropping `_windup_law` and the sibling silently
/// marches `"none"`, which IS rung 74 and passes every reduce gate in the crate.
pub const R75: LeverHooks = LeverHooks {
    at_lever: r75_at_lever,
    ..crate::demand_coordinate::R74
};

/// RUNG 75's `TwoSpoolTransientHooks` — **ZERO cells swapped**, an alias.
pub const R75_TWO: TwoSpoolTransientHooks = crate::demand_coordinate::R74_TWO;

/// RUNG 75's fuel table — ONE swap, `integrate_fuel`: **one refusal, and a call made for its
/// SIDE EFFECT.**
pub const R75_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r75_integrate_fuel,
    ..crate::demand_coordinate::R74_FUEL
};

/// RUNG 75's stator table — **ZERO cells swapped**, an alias.
pub const R75_STATOR: StatorTransientHooks = crate::demand_coordinate::R74_STATOR;

/// RUNG 75's third-loop table — **TWO of rung 74's eighteen cells re-aimed, and NOTHING added.**
///
/// Spelled out field by field rather than reached through a `..R74_TRIPLE` spread, for
/// [`R73_TRIPLE`](crate::applied_reference::R73_TRIPLE)'s stated reason: sixteen INHERITED
/// decisions sit on the page as decisions instead of as the residue of a spread, and an exhaustive
/// literal is what goes loud when a later slice widens the struct again.
///
/// **AND HERE THE SPREAD WOULD COST MORE THAN TIDINESS.** With `0 ADD` there is no width tripwire
/// in this slice at all, so the exhaustive literal is the only place a reader can see that
/// `windup_tau` moved and `cap_fuel` did not.
pub const R75_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::demand_coordinate::R74_TRIPLE.stator_leg,
    lagged_stator: crate::demand_coordinate::R74_TRIPLE.lagged_stator,
    clamp_v: crate::demand_coordinate::R74_TRIPLE.clamp_v,
    check_v0: crate::demand_coordinate::R74_TRIPLE.check_v0,
    rk4_floor: crate::demand_coordinate::R74_TRIPLE.rk4_floor,
    solve_v: crate::demand_coordinate::R74_TRIPLE.solve_v,
    manifold_v: crate::demand_coordinate::R74_TRIPLE.manifold_v,
    triple_laws: crate::demand_coordinate::R74_TRIPLE.triple_laws,
    triple_rig: crate::demand_coordinate::R74_TRIPLE.triple_rig,
    with_ref: crate::demand_coordinate::R74_TRIPLE.with_ref,
    reference: crate::demand_coordinate::R74_TRIPLE.reference,
    quad_gains_at: crate::demand_coordinate::R74_TRIPLE.quad_gains_at,
    rk4_floor_shared: crate::demand_coordinate::R74_TRIPLE.rk4_floor_shared,
    cap_fuel: crate::demand_coordinate::R74_TRIPLE.cap_fuel,
    sensed_cap: crate::demand_coordinate::R74_TRIPLE.sensed_cap,
    with_coord: crate::demand_coordinate::R74_TRIPLE.with_coord,
    // THE TWO THIS RUNG RE-AIMS.
    windup_tau: r75_windup_tau,
    shared_rig: r75_shared_rig,
};

// ---------------------------------------------------------------------------------------------
// THE FOUR RE-AIMED BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 75's `_windup_tau` — **THE HOOK RUNG 74's MARCH ALREADY READS, ANSWERING FOR THE FIRST
/// TIME.**
///
/// `None` is rung 74 EXACTLY: every branch guarded by it is skipped, so the reduce is a DISPATCH
/// and not a tolerance. `Some(tau_t)` turns on the three sites slice AF ported dead with their
/// proofs written first.
///
/// # THE TWO REFUSALS, AND WHY THEY ARE NOT ALSO SPELLED IN THE MARCH
///
/// `track` is REFUSED outside the plain `"demand"` coordinate: in `clip` rung 52's `max(0, ·)` is
/// still there and in `demand-latched` the latch is, so either cell would run TWO anti-windup
/// devices at once and attribute the result to this one — rung 63's *change one law at a time*.
/// And `tau_t` is DECLARED, never defaulted, because it has no derivation from anything shipped.
///
/// Python's second assert is `isinstance(self._tau_t, (int, float)) and self._tau_t > 0.0` — **one
/// test covering two different failures**, unset and non-positive. That is why the carrier is
/// [`Cell<Option<f64>>`](TwoSpoolTransientCore::tau_t) and not a `0.0` sentinel: collapsing the
/// two would make the refusal fire for the wrong reason while printing the right message.
///
/// # `panic!` AND NOT `Result`, FOR THE REASON THAT WAS MEASURED RATHER THAN THE ONE INHERITED
///
/// See [`TripleHooks::windup_tau`](crate::three_loop::TripleHooks::windup_tau): the crate's
/// original argument — *no caller catches* — is FALSE at the third of three call sites, and the
/// conclusion survives because the catching caller (`contraction_law`) raises `ValueError` or
/// `ZeroDivisionError` from `math.log(tau_t/(taus[0]+tau_t))` **before** the march, so no input it
/// can supply reaches this body's refusals.
fn r75_windup_tau(t: &TwoSpoolTransientCore) -> Option<f64> {
    if t.windup_law.get() != WINDUP_LAW_TRACK {
        return None;
    }
    let coord = t.lag_coord.get();
    assert!(
        coord == LAG_COORD_DEMAND,
        "rung-75: `track` is REFUSED outside the plain DEMAND coordinate. In `clip` rung 52's \
         `max(0, .)` is still there and in `demand-latched` the latch is, so either cell would run \
         TWO anti-windup devices at once and attribute the result to this one -- rung 63's \
         change-one-law-at-a-time. Got _lag_coord = {coord:?}.");
    let tau_t = t.tau_t.get();
    assert!(
        tau_t.is_some_and(|x| x > 0.0),
        "rung-75: the tracking clock `tau_t` is this rung's ONE new constant and it is DECLARED, \
         never defaulted; got {tau_t:?}. It has no derivation from anything shipped, so every \
         finding is a property of the SWEEP.");
    tau_t
}

/// RUNG 75's `at_lever` — **rung 74's sibling constructor returning a RUNG-75 machine THAT CARRIES
/// FOUR KNOBS.**
///
/// Fourteenth instance of the trap. The class is one part of the fix and the knobs are the other:
/// a sibling built from a receiver that has been armed with `"track"` must be `"track"`, so every
/// value is copied from the SOURCE core and never left at the class default.
///
/// **`_ic_cap` IS CARRIED HERE WHERE RUNG 74's `at_lever` DOES NOT CARRY IT, AND THAT IS PYTHON's
/// LINE.** `engine.py:18673` copies `_windup_law`, `_tau_t` **and** `_ic_cap`; `17713` copies
/// neither of the first two nor the third. The reason the difference is real rather than
/// housekeeping is that rung 75 is the first rung with a reader that RAISES the cap
/// (`contraction_law`), so from here on a sibling built inside that reader's `try/finally` must
/// see the raised value — at rung 74 the copy would have been invisible because nothing wrote the
/// field.
fn r75_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let m = match build_anti_windup_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    };
    m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
    m.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
    m.fuel.inner.windup_law.set(core.fuel.inner.windup_law.get());
    m.fuel.inner.tau_t.set(core.fuel.inner.tau_t.get());
    m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());
    m
}

/// RUNG 75's `_shared_rig` — **rung 74's rig with the DEVICE carried too, a fourth knob on the
/// same leak.**
///
/// **PRE-REGISTERED AS A NO-OP FOR THE FIRST TWO KNOBS AND *NOT* FOR THE THIRD**, which is where
/// it differs from [`r74_shared_rig`](crate::demand_coordinate). Rung 72's body — which this chain
/// delegates to — reaches its sibling through `self.at_lever(…)`, which on a rung-75 receiver is
/// [`r75_at_lever`], which has already copied all three. So `_windup_law` and `_tau_t` are the
/// same measured no-op `_lag_coord` was at rung 74, and `_ic_cap` is too *only because rung 75's
/// own `at_lever` newly carries it*.
///
/// Ported unchanged regardless — a duplication the source makes is not the port's to remove — and
/// the belt-and-braces set is what keeps the carrying true if a later rung's `at_lever` stops
/// doing it.
fn r75_shared_rig(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let (m, surge, lag) = (crate::demand_coordinate::R74_TRIPLE.shared_rig)(core, arm);
    m.fuel.inner.windup_law.set(core.fuel.inner.windup_law.get());
    m.fuel.inner.tau_t.set(core.fuel.inner.tau_t.get());
    m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());
    (m, surge, lag)
}

/// RUNG 75's `integrate_fuel` — **ONE REFUSAL, PLUS A CALL MADE ENTIRELY FOR ITS SIDE EFFECT.**
///
/// # THE SECOND LINE IS NOT A COMPUTATION AND MUST NOT BE OPTIMISED INTO ONE
///
/// Python's body is `assert self._windup_law in (…)`, then `if … == "track": self._windup_tau()`,
/// then `super().integrate_fuel(…)`. **The middle call discards its result.** Its own comment says
/// why: *the refusals are checked here and not only in `_windup_tau`, because `clip` DISPATCHES
/// OUT of this ladder before any hook is read* — so a `clip × track` march would have run rung 73
/// silently and reported it as this rung. Dropping the call because nothing reads it moves two
/// refusals off the exact arm they exist to catch, and **nothing in the crate would fail**: the
/// `clip` arm's whole point is that it never reaches the march that would otherwise raise them.
///
/// It is DISPATCHED (`self._windup_tau()`), not called as [`r75_windup_tau`] — so a rung-76
/// machine reaching this body takes rung 76's chain, and the pin is the same one slice AF step 6
/// had to repair four times.
///
/// # EVERY OTHER ARGUMENT GOES DOWN UNTOUCHED
///
/// Rung 74's five refusals and its `clip`/`tau_gov`/`has_fuel` entry test are all one level below
/// this one, and rung 75 adds nothing to them — so this body resolves nothing and forwards `lim`
/// verbatim. That is Python's `super().integrate_fuel(…)` with every keyword passed through.
fn r75_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    let law = ft.inner.windup_law.get();
    assert!(
        WINDUP_LAWS_DECLARED.contains(&law),
        "rung-75: the ANTI-WINDUP LAW is this rung's subject and it is DECLARED; got {law:?}. \
         'none' is rung 74 (no declared device -- the demand coordinate has NONE, which is rung 74 \
         s 4); 'track' is back-calculation onto the applied fuel.");
    if law == WINDUP_LAW_TRACK {
        // FOR THE SIDE EFFECT — see this function's doc. The result is discarded in Python too.
        (ft.inner.triple_hooks.windup_tau)(&ft.inner);
    }
    (crate::demand_coordinate::R74_FUEL.integrate_fuel)(
        ft, flight, fuel_schedule, nu0, s_end, ds, lim)
}
