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
use crate::demand_coordinate::{
    applied_demand, cap_gov, demand_reference, demand_target, LAG_COORD_DEMAND,
};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelLimiters, FuelPoint, FuelTransientCore,
    FuelTransientHooks,
};
use crate::gas::Abort;
use crate::limited_bleed::Regime;
use crate::map::ComponentMap;
use crate::shared_actuator::SharedRigArm;
use crate::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
    StatorTransientHooks,
};
use crate::three_loop::{closer_b, closer_v, LegRegime, TripleHooks};
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{
    MarchedBleed, MarchedStator, TwoSpoolTransientCore, TwoSpoolTransientHooks,
};

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

// ---------------------------------------------------------------------------------------------
// STEP 2 — THE DEVICE ITSELF: the scope guard, the four-knob march, and the FOUR RIGHT-HAND SIDES
// ---------------------------------------------------------------------------------------------

/// RUNG 75's `_with_windup` — **a named DEVICE for the length of one reader, restored in a
/// `finally`.** Rung 62's reason, NINTH reload.
///
/// # IT WRITES THE FIELDS DIRECTLY, AND THAT IS MEASURED RATHER THAN COPIED
///
/// [`CoordScope`](crate::demand_coordinate::CoordScope) and
/// [`RefScope`](crate::reference_split::RefScope) go through their cells, because `_with_coord` has
/// two definers (rungs 74 and 79) and a direct write would restore the wrong field on a rung-79
/// machine. **`_with_windup` has exactly ONE definer over all 58 classes** (§ 5.31 (iii)), so there
/// is no later body for a cell to reach and a slot would be dead —
/// [`ShareScope`](crate::shared_actuator::ShareScope)'s decision, on the same evidence.
///
/// # IT RESTORES **BOTH** FIELDS, AND A HALF-RESTORE IS SILENT
///
/// Python saves the PAIR — `prev = (self._windup_law, self._tau_t)` — and writes both back. A guard
/// that restored only the law would leave `_tau_t` set on the receiver, and
/// [`r75_windup_tau`]'s refusal fires on `None` but **not** on a stale positive: the next reader to
/// arm `"track"` would silently run on the previous one's clock. The pair is restored together and
/// `tests/slice_ag_laws.rs` reads both fields after the drop.
pub struct WindupScope<'a> {
    core: &'a TwoSpoolTransientCore,
    prev: (&'static str, Option<f64>),
}

impl<'a> WindupScope<'a> {
    /// Arm the device for as long as the returned guard lives.
    pub fn set(core: &'a TwoSpoolTransientCore, law: &'static str, tau_t: Option<f64>) -> Self {
        let prev = (core.windup_law.get(), core.tau_t.get());
        core.windup_law.set(law);
        core.tau_t.set(tau_t);
        WindupScope { core, prev }
    }

    /// What this scope displaced — Python's `prev`, exposed so a gate can read the restore POLICY
    /// rather than only its effect. [`RefScope::displaced`](crate::reference_split::RefScope) and
    /// [`CoordScope::displaced`](crate::demand_coordinate::CoordScope)'s precedent, now a PAIR.
    pub fn displaced(&self) -> (&'static str, Option<f64>) {
        self.prev
    }
}

impl Drop for WindupScope<'_> {
    fn drop(&mut self) {
        self.core.windup_law.set(self.prev.0);
        self.core.tau_t.set(self.prev.1);
    }
}

/// RUNG 75's `_windup_march` — **one rig, one march, under FOUR named knobs.**
///
/// # IT IS A COPY OF RUNG 74's `_coord_march` PLUS ONE LINE, AND IT IS PORTED AS A COPY
///
/// The two Python bodies are identical but for `engine.py:18694`
/// (`m._windup_law, m._tau_t, m._ic_cap = law, tau_t, self._ic_cap`) and `ref` losing its default.
/// Python re-spells the other eleven lines rather than calling
/// [`coord_march`](crate::demand_coordinate::coord_march), and a deliberate duplication is not the
/// port's to factor away.
///
/// **AND HERE THE FACTORED SPELLING WOULD BE WRONG, NOT MERELY UNFAITHFUL.** Rung 74's body RUNS
/// THE MARCH before it returns, so a `windup_march` that delegated to it and then set the three
/// knobs would set them on a machine that had already marched — every trajectory would be rung
/// 74's, every reduce gate would pass, and the device would be reported by a reader that never saw
/// it. That is the same failure the sibling-constructor trap has, one level up.
///
/// # `_ic_cap` IS CARRIED FROM THE CALLER, AND IT IS NEW AT THIS RUNG
///
/// `self._ic_cap` — the RECEIVER's, not the class default — because
/// [`contraction_law`] is the first reader in the ladder that RAISES the cap, inside a
/// `try/finally`, and every march it drives has to see the raised value. Rung 74's `_coord_march`
/// has no such line because nothing at that rung writes the field. Dropping it here is invisible
/// until step 3, where the cap is the measured quantity itself.
///
/// [`contraction_law`]: crate::anti_windup
#[allow(clippy::too_many_arguments)]
pub fn windup_march(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), r: f64, s_settle: f64, ds: f64, v_max: f64, inc: bool,
    coord: &'static str, ref_law: &'static str, law: &'static str, tau_t: Option<f64>,
    nu0: Option<(f64, f64)>,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>, Vec<FuelPoint>) {
    let (tau_f, tau_gov, tau_q, tau_s) = taus;
    let (m, surge, lag) = (core.triple_hooks().shared_rig)(core, &SharedRigArm {
        sm,
        tau: tau_q,
        tau_s,
        v_max,
        tt4_max,
        tau_att: tau_f,
        tau_rel: 3.0 * tau_f,
        inc,
        ..Default::default()
    });
    // PLAIN ASSIGNMENTS, all five, exactly as Python spells them — see
    // [`coord_march`](crate::demand_coordinate::coord_march)'s note for why a dispatch here is the
    // defect slice AF step 6 had to repair at four sites.
    m.fuel.inner.lag_coord.set(coord);
    m.fuel.inner.ref_law.set(ref_law);
    m.fuel.inner.windup_law.set(law);
    m.fuel.inner.tau_t.set(tau_t);
    m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());
    let leg = StatorLeg { accel: None, surge, tt4_max: Some(tt4_max) };
    let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
    let traj = m.stator_march_scoped(
        flight, &ramp, nu0, &leg,
        &MarchScope { tau_gov: Some(tau_gov), lag, ..MarchScope::DEFAULT }).0;
    (m, surge, lag, traj)
}

/// RUNG 75's `_rhs_laws` — **the four RIGHT-HAND SIDES, and NOT the four targets.**
///
/// # THIS TYPE EXISTS BECAUSE THE LAWS CHANGE *KIND*, AND A SHARED TYPE WOULD HIDE IT
///
/// [`DemandLaws`](crate::demand_coordinate::DemandLaws)'s `F` and `R` return a TARGET — the
/// reference a leg is chasing — and its `C` and `V` return the SOLVED `b` and `v`. Every law here
/// returns a RATE: `(tgt - w)/tau + (mf_app - w)/tau_t` on the fuel rows, `(b - q)/tau_q` and
/// `(v - v)/tau_s` on the others. The two are the same Rust type up to arity, so sharing one struct
/// would make a target and a derivative interchangeable at every call site in the crate, and step
/// 3's `_rhs_gains_at` would consume either without complaint. **The kind difference is pinned by a
/// gate as well as by the type**: at one point, rung 74's `F` and this `F` must return DIFFERENT
/// numbers, and the difference must be exactly `(that - wf)/tau_f + track`.
///
/// # WHY THE DERIVATIVE IS DIFFERENCED END TO END — this rung's ONE implementation cost
///
/// Every inherited gains reader differences the TARGET and hands the result to `_jac4`, which
/// assembles `J[i][j] = (dcmd_i/dx_j - delta_ij)/tau_i`. **The tracking term is in neither**: it is
/// not part of any leg's target and `tau_t` is not in `taus`. So `demand_gains` run on the `track`
/// cell would report the masked diagonal unchanged, `det J` still dead and the spectrum invariant —
/// **a PERFECT REFUTATION OF THIS RUNG'S HEADLINE, having measured nothing.**
///
/// The repair is this function: difference the DERIVATIVE, so the fuel rows are measured end to end
/// and the diagonal is a measurement rather than a construction. **`tau_t` is deliberately NOT
/// added to `taus`** — letting `_jac4` write `-1/tau_t` onto the diagonal would be the SEVENTH
/// instance of the shipped-instrument-agrees-with-itself pattern this project has booked (rung 67
/// gate 9, rung 71 § 1.4, rung 72 §§ 4 and 8, rung 73's `_reference`, rung 74 § 1.1).
///
/// # `tau_f` IS FROZEN AT THE BASE POINT
///
/// Rung 72's convention, passed in as a constant rather than recomputed per perturbation: rung 52's
/// lag is a STEP in the attack/release direction, and central-differencing across it would measure
/// the step and not the plant.
///
/// # THE DISPATCHES, AND THE ONE DIRECT CALL THAT IS A CENSUS RESULT
///
/// `windup_tau` is read ONCE at the top and **through the table**, so a rung-76 machine running
/// this inherited body takes rung 76's chain. `cap_fuel` is a cell (three definers) and `_solve_v`
/// is a cell (two); `_cap_gov`, `_solve_b`, `_closer` and `_closer_v` are single-definer and are
/// direct calls — [`demand_laws`](crate::demand_coordinate::demand_laws)'s censused asymmetry,
/// inherited here with the same evidence rather than copied from the sibling's shape.
///
/// # THE TWO DIVISORS PYTHON REACHES THROUGH AN `Option`
///
/// `self.bleed_lim.tau` and `self._stator_leg().tau` are `None` on an unlagged valve and a
/// stator-free machine; Python raises `TypeError` dividing by them, so `expect` is the faithful
/// spelling and the message says which arm the caller is on.
pub struct RhsLaws<'a> {
    /// **F** — rung 52's leg, as a RATE: `(wf, wr, q, v) -> (dwf/ds, regime)`.
    pub f: Box<dyn Fn(f64, f64, f64, f64) -> Result<(f64, LegRegime), Abort> + 'a>,
    /// **R** — rung 47's governor, as a RATE: `(wf, wr, q, v) -> (dwr/ds, regime)`.
    pub r: Box<dyn Fn(f64, f64, f64, f64) -> Result<(f64, LegRegime), Abort> + 'a>,
    /// **C** — the VALVE law, as a RATE: `(wf, wr, q, v) -> (dq/ds, regime)`.
    pub c: Box<dyn Fn(f64, f64, f64, f64) -> Result<(f64, Regime), Abort> + 'a>,
    /// **V** — the STATOR law, as a RATE: `(wf, wr, q, v) -> (dv/ds, regime)`.
    pub v: Box<dyn Fn(f64, f64, f64, f64) -> Result<(f64, Regime), Abort> + 'a>,
}

/// Build [`RhsLaws`] — see its doc for why this is not `demand_laws` with a division.
#[allow(clippy::too_many_arguments)]
pub fn rhs_laws<'a>(
    core: &'a ScheduledStatorCore, flight: &'a FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&'a AccelSchedule>, surge: Option<&'a Floor>, tt4_max: f64, tau_f: f64,
    tau_gov: f64,
) -> RhsLaws<'a> {
    let ft = &core.fuel;
    let (tt2, pt2, _) = ft.inner.inlet(flight);
    // ONCE, at the top, and THROUGH THE TABLE — Python's `tau_t = self._windup_tau()`.
    let tau_t = (ft.inner.triple_hooks.windup_tau)(&ft.inner);

    // Python's `_track`: `0.0` when the device is disarmed, which is what makes the `"none"` arm
    // EXACT rather than small — the term is not computed and added, it is not there.
    let track = move |w: f64, mf_app: f64| -> f64 {
        match tau_t {
            None => 0.0,
            Some(t) => (mf_app - w) / t,
        }
    };

    // **F** — `ma` is formed BEFORE the state guards (rung 76 needs it above the cap call), the cap
    // is solved inside them, and the reference, the rate and the label are outside.
    let f = move |wf: f64, wr: f64, q: f64, v: f64| -> Result<(f64, LegRegime), Abort> {
        let ma = applied_demand(mf_sched, wf, wr);
        let cap = {
            let _sb = MarchedBleed::set(&ft.inner, q);
            let _sv = MarchedStator::set(&ft.inner, v);
            (ft.inner.triple_hooks.cap_fuel)(ft, flight, a, h, mf_sched, accel, surge, Some(ma))?
        };
        let tgt = demand_reference(&ft.inner, demand_target(&ft.inner, cap, mf_sched), wf, ma);
        Ok((
            (tgt - wf) / tau_f + track(wf, ma),
            if cap < mf_sched { LegRegime::Riding } else { LegRegime::Dormant },
        ))
    };

    // **R** — and here Python forms `ma` AFTER the guard block and binds it to a name, where rung
    // 74's `R` re-formed it inline inside the reference argument. Kept in Python's order.
    let r = move |wf: f64, wr: f64, q: f64, v: f64| -> Result<(f64, LegRegime), Abort> {
        let cap = {
            let _sb = MarchedBleed::set(&ft.inner, q);
            let _sv = MarchedStator::set(&ft.inner, v);
            cap_gov(ft, flight, a, h, mf_sched, tt4_max)?
        };
        let ma = applied_demand(mf_sched, wf, wr);
        let tgt = demand_reference(&ft.inner, demand_target(&ft.inner, cap, mf_sched), wr, ma);
        Ok((
            (tgt - wr) / tau_gov + track(wr, ma),
            if cap < mf_sched { LegRegime::Riding } else { LegRegime::Dormant },
        ))
    };

    // **C** — the VALVE law: it trials `b`, so NO `b_state`, but `v_state = v`; the rate is against
    // the state `q` it was handed.
    let c = move |wf: f64, wr: f64, q: f64, v: f64| -> Result<(f64, Regime), Abort> {
        let _sv = MarchedStator::set(&ft.inner, v);
        let bl = ft.inner.lever.lim.expect("rung-75's valve law on an unfloored machine");
        let (_, b, reg) = crate::limited_bleed::r64_solve_b(
            &bl,
            closer_b(ft, a, h, 1e-9f64.max(applied_demand(mf_sched, wf, wr)), tt2, pt2))?;
        let tau_q = bl.tau.expect(
            "rung-75's RHS divides by `self.bleed_lim.tau`, which is `None` on rung 64's \
             INSTANTANEOUS valve -- Python raises `TypeError` there, so this arm has no law.");
        Ok(((b - q) / tau_q, reg))
    };

    // **V** — the mirror, trialling `v` with `b_state = q`, through the `_solve_v` cell.
    let v = move |wf: f64, wr: f64, q: f64, v: f64| -> Result<(f64, Regime), Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let (_, vv, reg) = ft.inner.solve_v(&closer_v(
            ft, a, h, 1e-9f64.max(applied_demand(mf_sched, wf, wr)), tt2, pt2))?;
        let tau_s = ft.inner.stator_leg()
            .expect("rung-75's RHS law on a machine with no stator leg")
            .tau
            .expect("rung-75's RHS divides by `self._stator_leg().tau`, `None` on an unlagged \
                     stator -- Python raises `TypeError` there.");
        Ok(((vv - v) / tau_s, reg))
    };

    RhsLaws { f: Box::new(f), r: Box::new(r), c: Box::new(c), v: Box::new(v) }
}
