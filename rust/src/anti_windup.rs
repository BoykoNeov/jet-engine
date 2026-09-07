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
    applied_demand, cap_gov, demand_authority, demand_reference, demand_target, LAG_COORD_CLIP,
    LAG_COORD_DEMAND, LAG_COORD_LATCHED,
};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Authority, Floor, FuelLimiters, FuelPoint, FuelTransientCore,
    FuelTransientHooks, PointExtra,
};
use crate::gas::Abort;
use crate::limited_bleed::Regime;
use crate::map::ComponentMap;
use crate::shared_actuator::{charpoly4, quartic_roots_c, riding4, SharedRigArm};
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

/// `"applied"` — named here because §§ 1/2's targets BRANCH on it, and a typo would
/// silently take the `sched` arm's formula on the `applied` cell.
const REF_APPLIED: &str = crate::applied_reference::REF_LAW_APPLIED;

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
#[allow(clippy::type_complexity)]
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

// ---------------------------------------------------------------------------------------------
// § 1 — `_rhs_gains_at`: THE SIXTEEN DIFFERENCES, TAKEN ON THE *RATES*
// ---------------------------------------------------------------------------------------------

/// RUNG 75's `_rhs_gains_at` return — **the WHOLE 4x4, diagonals included.**
///
/// # THIS IS NOT [`QuadGains`](crate::shared_actuator::QuadGains), AND THE DIFFERENCE IS THE RUNG
///
/// Every inherited gains reader returns the fourteen OFF-diagonal entries and hands them to
/// `_jac4`, which *constructs* the diagonal as `(dcmd_i/dx_i - 1)/tau_i`. That construction is
/// exactly what this rung's headline is about: the tracking term writes `-1/tau_t` onto the masked
/// leg's own diagonal, and `tau_t` is not in `taus`, so a constructed diagonal cannot contain it.
/// **Sixteen measured entries, and no assembly step** — [`rhs_laws`]'s doc carries the same
/// argument from the other end.
///
/// # THE THREE ABSENT KEYS ARE ABSENT, NOT ZERO
///
/// Python's two refusal dicts carry `interior`, `off_regime`, `s` and `near_switch` and nothing
/// else; the interior dict adds nine more. So [`j`](RhsGains::j) and [`v_base`](RhsGains::v_base)
/// are `None` on a refusal because the KEY IS MISSING — slice AE's *absent versus zero* rule, which
/// a reader that asked for `J` on a dropped point would hit as a `KeyError` in Python and hits as
/// an `unwrap` on `None` here.
#[derive(Clone, Debug)]
pub struct RhsGains {
    pub interior: bool,
    /// `"F+0"` … `"V-3"` — the LAW's letter, the sign, and the COLUMN index. `["switch"]` on the
    /// near-switch arm, which is Python's one non-positional label.
    pub off_regime: Vec<String>,
    pub near_switch: bool,
    pub s: f64,
    /// The full Jacobian, `J[i][j] = d(rate_i)/d(state_j)`. `None` on both refusal arms.
    pub j: Option<[[f64; 4]; 4]>,
    pub authority: Option<Authority>,
    /// Which leg is MASKED — the one that does not hold the actuator. `None` on `Dormant`/`Tie`.
    pub masked: Option<Authority>,
    pub v_base: Option<f64>,
    /// The MASKED COLUMN: `max |J[i][im]|` over `i != im`. **Rung 72/73/74's triangularity**, and
    /// the reading that keeps `n_live <= 3` at this rung too.
    pub mask_leak: Option<f64>,
    pub masked_diag: Option<f64>,
    pub auth_diag: Option<f64>,
    /// `J[im][ia]` — the masked ROW's coupling to the leg that holds. The device's own entry.
    pub masked_row_auth: Option<f64>,
    /// `max |mf_app - w_auth|` over the base point and both perturbations of the authoritative
    /// leg's own state — **the proof that the tracking term is the zero FUNCTION there and not a
    /// small number.**
    pub track_leak: Option<f64>,
}

impl RhsGains {
    /// Python's `dict(interior=False, off_regime=…, s=…, near_switch=…)` — four keys, and the
    /// other nine ABSENT.
    fn dropped(s: f64, off: Vec<String>, near_switch: bool) -> Self {
        RhsGains {
            interior: false, off_regime: off, near_switch, s,
            j: None, authority: None, masked: None, v_base: None, mask_leak: None,
            masked_diag: None, auth_diag: None, masked_row_auth: None, track_leak: None,
        }
    }
}

/// RUNG 75's `_rhs_gains_at` — **all SIXTEEN central differences of the RATE vector.**
///
/// Rung 72's two filters are kept verbatim: REGIME on every one of the 32 perturbed evaluations,
/// and SWITCH PROXIMITY on the `min()` kink. What is NOT kept is rung 74's `manifold` knob, and its
/// absence is the honest option rather than a dropped feature — rung 74's reader read the DEMAND
/// matrix at the CLIP plant's states, where `v` belongs to a different plant; every cell here is
/// read on the trajectory of the plant it is a Jacobian of, so `p["v"]` IS the state.
///
/// # THE EVALUATION ORDER IS PYTHON'S, AND ALL 32 RUN BEFORE ANY REGIME IS READ
///
/// `for j in 0..4 { for sgn in [+1, -1] { for i in 0..4 } }` — column, then sign, then law. Rung
/// 73's rule and for its reason: a short circuit on the first off-regime arm would change how many
/// closure calls the plant sees, and the off-regime LIST would lose its tail.
///
/// # THE TRANSPOSITION NO GATE IN THIS CRATE CAN CATCH
///
/// The `track_leak` block calls [`applied_demand`] three times, and Python's argument order is
/// `(wf, wr, mf_sched)` against this crate's `(mf_sched, wf, wr)`. **That is unobservable**: the
/// body is a plain minimum of all three arguments, so it is symmetric and every permutation returns
/// the same float. Named rather than gated — a gate here would score `SURVIVED` for a structural
/// reason and read as coverage.
#[allow(clippy::too_many_arguments)]
pub fn rhs_gains_at(
    core: &ScheduledStatorCore, flight: &FlightCondition, p: &FuelPoint,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>, tt4_max: f64, tau_f: f64, tau_gov: f64,
    dg: f64, dq: f64, dv: f64, switch_guard: f64,
) -> Result<RhsGains, Abort> {
    let (a, h, mf_sched) = (p.nu_lp, p.nu_hp, p.mf_sched);
    // Python's `p["w_fuel"] if "w_fuel" in p else mf_sched - p["g_fuel"]`, and then `p["b"]`,
    // `p["v"]` unconditionally — so a point with neither pair raises a `KeyError` there and panics
    // here. [`windup_rows`] drives the DEMAND arm, which is the one the shipped grid takes.
    let (wf, wr, q, v) = match p.extra {
        PointExtra::Demand { w_fuel, w_gov, b, v, .. } => (w_fuel, w_gov, b, v),
        PointExtra::Shared { g_fuel, g_gov, b, v, .. } =>
            (mf_sched - g_fuel, mf_sched - g_gov, b, v),
        _ => panic!("rung-75's RHS gains need a point carrying `w_fuel`/`w_gov` or a \
                     `g_fuel`/`g_gov` pair to project from, plus `b` and `v`."),
    };
    assert!(crate::lagged_bleed::lagged(&core.fuel.inner) && core.fuel.inner.lagged_stator(),
            "rung-75: the RHS reader measures all FOUR rows, so both the valve and the stator \
             must be lagged states -- rungs 65/66's refusal of an instantaneous leg beside \
             lagged ones, inherited.");
    let laws = rhs_laws(core, flight, a, h, mf_sched, accel, surge, tt4_max, tau_f, tau_gov);
    if (wf - wr).abs() <= switch_guard * dg {
        return Ok(RhsGains::dropped(p.s, vec!["switch".to_string()], true));
    }
    let base = [wf, wr, q, v];
    let steps = [dg, dg, dq, dv];
    let mut off: Vec<String> = Vec::new();
    let mut vals = [[[0.0f64; 4]; 2]; 4]; // [column j][sign index][law i]
    for j in 0..4 {
        for (si, sgn) in [1.0f64, -1.0].into_iter().enumerate() {
            let mut x = base;
            x[j] += sgn * steps[j];
            for i in 0..4 {
                let (val, riding) = match i {
                    0 => { let (w, rg) = (laws.f)(x[0], x[1], x[2], x[3])?;
                           (w, rg == LegRegime::Riding) }
                    1 => { let (w, rg) = (laws.r)(x[0], x[1], x[2], x[3])?;
                           (w, rg == LegRegime::Riding) }
                    2 => { let (w, rg) = (laws.c)(x[0], x[1], x[2], x[3])?;
                           (w, rg == Regime::Riding) }
                    _ => { let (w, rg) = (laws.v)(x[0], x[1], x[2], x[3])?;
                           (w, rg == Regime::Riding) }
                };
                if !riding {
                    off.push(format!("{}{}{}", ["F", "R", "C", "V"][i],
                                     if si == 0 { "+" } else { "-" }, j));
                }
                vals[j][si][i] = val;
            }
        }
    }
    if !off.is_empty() {
        return Ok(RhsGains::dropped(p.s, off, false));
    }
    let mut jm = [[0.0f64; 4]; 4];
    for (i, row) in jm.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (vals[j][0][i] - vals[j][1][i]) / (2.0 * steps[j]);
        }
    }
    let auth = demand_authority(wf, wr, mf_sched);
    let masked = match auth {
        Authority::Gov => Some(Authority::Fuel),
        Authority::Fuel => Some(Authority::Gov),
        _ => None,
    };
    // Python's `im`/`ia`: the MASKED row's index and the AUTHORITATIVE row's index.
    let (im, ia) = match masked {
        Some(Authority::Fuel) => (Some(0usize), Some(1usize)),
        Some(Authority::Gov) => (Some(1usize), Some(0usize)),
        _ => (None, None),
    };
    // THE TRACKING TERM ON THE LEG THAT HOLDS THE ACTUATOR IS THE ZERO **FUNCTION**, not a small
    // number: `mf_app = min(mf_sched, wf, wr) == w_auth` in a NEIGHBOURHOOD, so it is zero at the
    // base point AND at both perturbations of that leg's own state. That is what leaves rung 72's
    // `ONE plant IS rungs 68/69/70/71 by AUTHORITY` untouched.
    let track_leak = ia.map(|ia| {
        let mut leak = 0.0f64;
        for sgn in [1.0f64, -1.0] {
            let mut x = base;
            x[ia] += sgn * steps[ia];
            leak = leak.max((applied_demand(mf_sched, x[0], x[1]) - x[ia]).abs());
        }
        leak.max((applied_demand(mf_sched, wf, wr) - base[ia]).abs())
    });
    Ok(RhsGains {
        interior: true,
        off_regime: Vec::new(),
        near_switch: false,
        s: p.s,
        j: Some(jm),
        authority: Some(auth),
        masked,
        v_base: Some(v),
        mask_leak: im.map(|im| (0..4).filter(|i| *i != im)
                                     .fold(f64::NEG_INFINITY, |m, i| m.max(jm[i][im].abs()))),
        masked_diag: im.map(|im| jm[im][im]),
        auth_diag: ia.map(|ia| jm[ia][ia]),
        masked_row_auth: match (im, ia) { (Some(im), Some(ia)) => Some(jm[im][ia]), _ => None },
        track_leak,
    })
}

// ---------------------------------------------------------------------------------------------
// § 1 (continued) — `_windup_rows` / `windup_gains`: THE POLE LEAVES THE ORIGIN
// ---------------------------------------------------------------------------------------------

/// One row of [`windup_rows`] — the SAME state read through BOTH devices.
///
/// Every field ending `0` is the `"none"` arm's reading at the identical state, which is what makes
/// each claim in § 1 a DIFFERENCE between two laws rather than a property of a trajectory.
#[derive(Clone, Debug)]
pub struct WindupRow {
    pub s: f64,
    pub auth: Authority,
    pub masked: Authority,
    /// `tau_f` when the FUEL leg is masked, `taus[1]` when the governor is — the clock the masked
    /// row would divide by with no device.
    pub tau_masked: f64,
    pub masked_diag: f64,
    pub masked_diag0: f64,
    pub auth_diag: f64,
    pub auth_diag0: f64,
    pub row_auth: f64,
    pub row_auth0: f64,
    pub mask_leak: f64,
    pub mask_leak0: f64,
    pub track_leak: f64,
    /// `charpoly4(J)[4]` — the CONSTANT term, i.e. `det J` up to the sign convention the quartic
    /// carries. Dead since rung 73 on the `applied` arm; alive here.
    pub det: f64,
    pub det0: f64,
    pub zeros: usize,
    pub zeros0: usize,
}

/// RUNG 75's `_windup_rows` — **`track` against `none` AT THE SAME STATES, both through
/// [`rhs_laws`].**
///
/// # THE STATES ARE THE CLIP PLANT's AT THE INHERITED FLOOR, AND THAT IS A DISCLOSURE
///
/// Rung 74 § 1.3's disclosure inherited word for word and for its own reason: `_shared_rig` gives
/// every leg ONE margin, so at the lowered floor § 2's arms need, the VALVE is off-regime at every
/// point and `riding4` returns NOTHING — measured, 0 of 0. A Jacobian is a function of the STATE,
/// and every claim in this section is a DIFFERENCE between two laws at ONE state, so the choice of
/// trajectory cannot manufacture one.
///
/// # `_lag_coord` IS SET BY A **PLAIN ASSIGNMENT**, AND THIS IS THE THIRD SITE OF THAT KIND
///
/// Python writes `m._lag_coord = "demand"` on the marched sibling, after the march and before the
/// filter. Slice AF step 6's leading finding was FOUR production call sites that dispatched a write
/// Python makes by plain assignment — a dispatched pin here would hit rung 79's second definer of
/// the same carrier and turn an attribute write into a refusal. So: a direct `set`, no scope, no
/// hook, matching [`windup_march`]'s five and [`demand_gains`](crate::demand_coordinate)'s two.
///
/// **AND THE WRITE ITSELF IS LOAD-BEARING, FOR A READER THAT DOES NOT EXIST ONE RUNG DOWN.** The
/// march above runs `clip` x `none` — legal, because the device is disarmed — and the flip is what
/// makes the [`WindupScope`] two lines below legal: [`r75_windup_tau`](self) REFUSES `track`
/// anywhere but the plain demand coordinate. Rung 74's enumeration of this tag's readers is
/// `demand_target` alone, where `clip` and `demand` are indistinguishable BY CONSTRUCTION, and on
/// that enumeration this line is a no-op. **It is not**: § 5.31.3 (a) records the sweep dropping it
/// and hitting a panic. An enumeration of who reads a carrier has an expiry date, exactly as a line
/// citation does, and it expires at the rung that adds a reader.
///
/// # THE RATE THRESHOLD DELIBERATELY EXCLUDES `1/tau_t`
///
/// `rate = 1/tau_f + sum(1/t for t in taus[1:])` — the four PLANT clocks, not the device's. That is
/// the same refusal [`rhs_laws`] makes by keeping `tau_t` out of `taus`, one level up: a zero
/// threshold scaled by the device's own clock would move with the knob being swept, and `zeros`
/// would report the threshold rather than the spectrum.
#[allow(clippy::too_many_arguments)]
pub fn windup_rows(
    core: &ScheduledStatorCore, flight: &FlightCondition, sm: f64, ref_law: &'static str,
    tau_t: f64, taus: (f64, f64, f64, f64), inc: bool, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    r: f64, s_settle: f64, ds: f64, v_max: f64, every: usize,
) -> (Vec<WindupRow>, usize) {
    let (m, surge, lag, traj) = windup_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
        LAG_COORD_CLIP, ref_law, WINDUP_LAW_NONE, None, None);
    // `m._lag_coord = "demand"` — PLAIN, see the header.
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let lag = lag.expect("`_shared_rig` arms the fuel leg, so it carries the lag");
    let mut rows: Vec<WindupRow> = Vec::new();
    for p in pts.iter().step_by(every) {
        let (required_fuel, g_fuel) = match p.extra {
            PointExtra::Demand { required_fuel, g_fuel, .. }
            | PointExtra::Shared { required_fuel, g_fuel, .. } => (required_fuel, g_fuel),
            _ => panic!("rung-75's rows read `required_fuel`/`g_fuel` off every filtered point."),
        };
        let tau_f = lag.tau(required_fuel, g_fuel);
        let g = {
            let _ws = WindupScope::set(&m.fuel.inner, WINDUP_LAW_TRACK, Some(tau_t));
            rhs_gains_at(&m, flight, p, None, surge.as_ref(), tt4_max, tau_f, taus.1,
                         1e-7, 1e-5, 1e-4, 4.0).unwrap_or_else(|e| panic!("{}", e.0))
        };
        if !g.interior || g.masked.is_none() {
            continue;
        }
        let g0 = {
            let _ws = WindupScope::set(&m.fuel.inner, WINDUP_LAW_NONE, None);
            rhs_gains_at(&m, flight, p, None, surge.as_ref(), tt4_max, tau_f, taus.1,
                         1e-7, 1e-5, 1e-4, 4.0).unwrap_or_else(|e| panic!("{}", e.0))
        };
        if !g0.interior {
            continue;
        }
        let rate = 1.0 / tau_f + 1.0 / taus.1 + 1.0 / taus.2 + 1.0 / taus.3;
        let c = charpoly4(&g.j.expect("an interior reading carries J"));
        let c0 = charpoly4(&g0.j.expect("an interior reading carries J"));
        let rt = quartic_roots_c(&c);
        let rt0 = quartic_roots_c(&c0);
        let masked = g.masked.expect("filtered above");
        let tau_masked = if masked == Authority::Fuel { tau_f } else { taus.1 };
        rows.push(WindupRow {
            s: p.s,
            auth: g.authority.expect("an interior reading carries an authority"),
            masked,
            tau_masked,
            masked_diag: g.masked_diag.expect("a masked leg has a diagonal"),
            masked_diag0: g0.masked_diag.expect("the same state, the same mask"),
            auth_diag: g.auth_diag.expect("a masked leg implies an authoritative one"),
            auth_diag0: g0.auth_diag.expect("the same state, the same mask"),
            row_auth: g.masked_row_auth.expect("both indices exist here"),
            row_auth0: g0.masked_row_auth.expect("both indices exist here"),
            mask_leak: g.mask_leak.expect("both indices exist here"),
            mask_leak0: g0.mask_leak.expect("both indices exist here"),
            track_leak: g.track_leak.expect("both indices exist here"),
            det: c[4],
            det0: c0[4],
            zeros: rt.iter().filter(|z| z.abs() < 1e-4 * rate).count(),
            zeros0: rt0.iter().filter(|z| z.abs() < 1e-4 * rate).count(),
        });
    }
    (rows, pts.len())
}

/// One cell of [`windup_gains`] — Python's `cells[f"{ref}|{tau_t}"]`, which is a **two-key EMPTY
/// dict OR a nineteen-key reading** and never both.
///
/// An enum for [`CoordRead`](crate::demand_coordinate::CoordRead)'s reason: the two dicts share
/// only `n` and `n_riding`, and the `ratios` block below reads `a.get("rows")` — absent on the
/// empty arm — so a port that flattened them into one struct of `Option`s would have to
/// re-discover which arm it is on at every read.
#[derive(Clone, Debug)]
pub enum WindupCell {
    /// `dict(n=0, n_riding=n)` — no row survived the interior filters at this cell.
    Empty { n_riding: usize },
    Read(Box<WindupCellRead>),
}

/// The nineteen keys a populated [`WindupCell`] carries.
#[derive(Clone, Debug)]
pub struct WindupCellRead {
    pub n: usize,
    pub n_riding: usize,
    pub tau_t: f64,
    pub ref_law: &'static str,
    /// **P1** — the masked diagonal IS the device's own clock (`applied`) or the two rates ADDED
    /// (`sched`): rung 66's identity in a fifth shape.
    pub masked_diag: (f64, f64),
    pub masked_diag0: (f64, f64),
    /// `max |masked_diag - target| * min(tau_t, 1)`, the target being `-1/tau_t` on `applied` and
    /// `-(1/tau_t + 1/tau_masked)` on `sched`.
    pub diag_err: f64,
    /// **P2** — the device is the ZERO FUNCTION on the leg that HOLDS, as a RELATIVE move.
    pub auth_diag_moved: f64,
    pub track_leak: f64,
    /// **P3** — the masked COLUMN, untouched: `n_live <= 3` a FOURTH time.
    pub mask_leak: f64,
    pub mask_leak0: f64,
    /// **P4/P5** — the determinant and the zero count, both arms.
    pub det: (f64, f64),
    pub det0: (f64, f64),
    pub det_alive: f64,
    pub det0_alive: f64,
    pub zeros: Vec<usize>,
    pub zeros0: Vec<usize>,
    /// **P6** — the masked ROW's coupling to the authoritative leg.
    pub row_err: f64,
    pub row_auth0: (f64, f64),
    pub rows: Vec<WindupRow>,
}

/// Python's `ratios[ref]` — the masked diagonal and `det J` scaled TOGETHER.
#[derive(Clone, Debug)]
pub struct WindupRatio {
    pub ref_law: &'static str,
    pub n: usize,
    pub diag: (f64, f64),
    pub det: (f64, f64),
}

/// RUNG 75's `windup_gains` return.
#[derive(Clone, Debug)]
pub struct WindupGains {
    pub phi_lim: f64,
    pub taus: (f64, f64, f64, f64),
    pub tau_ts: Vec<f64>,
    pub inc: bool,
    pub ds: f64,
    /// Python's `cells`, keyed `f"{ref}|{tau_t}"`. The pair is carried as FIELDS rather than as a
    /// formatted key: a `f64` has no shortest-repr spelling in Rust, and a key built with a
    /// `{:?}` would differ from Python's on the first value that needs seventeen digits. The
    /// oracle formats the key from the pair, in one place.
    pub cells: Vec<((&'static str, f64), WindupCell)>,
    pub ratios: Vec<WindupRatio>,
}

/// RUNG 75 § 1 — **the pole LEAVES THE ORIGIN, `det J` REVIVES, and the RANK does not move.**
///
/// The exact inverse of rung 74. That coordinate was a STATE-INDEPENDENT forcing, so it was in no
/// Jacobian at all and the spectrum was invariant. This device's term is STATE-DEPENDENT: it writes
/// `-1/tau_t` onto the masked leg's own diagonal — the one rung 73's applied reference had
/// cancelled to exactly zero — so the masked pole moves off the origin, `zeros` loses `n_masked`
/// and `det J`, dead since rung 73, comes back. And `n_live` is UNMOVED, because the term sits in
/// the masked leg's ROW (it reads the authoritative leg through `mf_app`) while the masked COLUMN
/// stays zero: `min()` is still flat in what the masked leg holds.
///
/// THE REVIVAL IS `applied`-ONLY, and that is one mechanism with two faces rather than two
/// findings. Under `sched` the masked diagonal was ALREADY `-1/tau` — the target is `cap`, which
/// does not contain `w` — so the device moves it to `-(1/tau + 1/tau_t)` and nothing was ever dead
/// there.
///
/// Read through [`rhs_laws`], never `_jac4`: a target-differencing reader is BLIND to this rung's
/// whole subject and would have returned a perfect refutation of the headline.
///
/// # THE FIRST CELL's `row_err` TARGET IS IDENTICALLY ZERO ON THE SHIPPED GRID
///
/// `row_err` differences `row_auth` against `1/tau_t - 1/tau_masked` on the `applied` arm, and the
/// shipped defaults are `tau_ts[0] = 0.05` beside `taus = (0.05,) * 4`. Where the fuel leg is
/// masked, `tau_masked` is the LAG's `tau_f` and not `taus[0]`, so the two coincide only when the
/// lag returns the attack clock — but where the GOVERNOR is masked, `tau_masked` IS `taus[1]` and
/// the target is exactly `0.0`. **A cell whose target is an exact zero cannot discriminate the
/// expression from any other that vanishes there** ([[rust-port-slice-t-step1]]'s lesson), which is
/// why the second `tau_t` exists in the default grid and why the step-3 gates that test WHICH clock
/// each row reads run on a rig with five distinct clocks.
#[allow(clippy::too_many_arguments)]
pub fn windup_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, taus: (f64, f64, f64, f64), tau_ts: &[f64], refs: &[&'static str], inc: bool,
    r: f64, s_settle: f64, ds: f64, v_max: f64, every: usize,
) -> WindupGains {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let mut cells: Vec<((&'static str, f64), WindupCell)> = Vec::new();
    for ref_law in refs {
        for &tau_t in tau_ts {
            let (rows, n) = windup_rows(core, flight, sm, ref_law, tau_t, taus, inc,
                                        tt4_lo, tt4_hi, tt4_max, r, s_settle, ds, v_max, every);
            if rows.is_empty() {
                cells.push(((ref_law, tau_t), WindupCell::Empty { n_riding: n }));
                continue;
            }
            let applied = *ref_law == REF_APPLIED;
            let fold = |f: &dyn Fn(&WindupRow) -> f64| -> (f64, f64) {
                (rows.iter().map(f).fold(f64::INFINITY, f64::min),
                 rows.iter().map(f).fold(f64::NEG_INFINITY, f64::max))
            };
            let mx = |f: &dyn Fn(&WindupRow) -> f64| -> f64 {
                rows.iter().map(f).fold(f64::NEG_INFINITY, f64::max)
            };
            let mut zeros: Vec<usize> = rows.iter().map(|x| x.zeros).collect();
            zeros.sort_unstable();
            zeros.dedup();
            let mut zeros0: Vec<usize> = rows.iter().map(|x| x.zeros0).collect();
            zeros0.sort_unstable();
            zeros0.dedup();
            cells.push(((ref_law, tau_t), WindupCell::Read(Box::new(WindupCellRead {
                n: rows.len(),
                n_riding: n,
                tau_t,
                ref_law,
                masked_diag: fold(&|x| x.masked_diag),
                masked_diag0: fold(&|x| x.masked_diag0),
                diag_err: mx(&|x| {
                    let tgt = if applied { -1.0 / tau_t }
                              else { -(1.0 / tau_t + 1.0 / x.tau_masked) };
                    (x.masked_diag - tgt).abs() * tau_t.min(1.0)
                }),
                auth_diag_moved: mx(&|x| (x.auth_diag - x.auth_diag0).abs()
                                         / 1e-30f64.max(x.auth_diag0.abs())),
                track_leak: mx(&|x| x.track_leak),
                mask_leak: mx(&|x| x.mask_leak),
                mask_leak0: mx(&|x| x.mask_leak0),
                det: fold(&|x| x.det),
                det0: fold(&|x| x.det0),
                det_alive: rows.iter().map(|x| x.det.abs()).fold(f64::INFINITY, f64::min),
                det0_alive: mx(&|x| x.det0.abs()),
                zeros,
                zeros0,
                row_err: mx(&|x| {
                    let tgt = if applied { 1.0 / tau_t - 1.0 / x.tau_masked }
                              else { 1.0 / tau_t };
                    (x.row_auth - tgt).abs() * tau_t.min(1.0)
                }),
                row_auth0: fold(&|x| x.row_auth0),
                rows: rows.clone(),
            }))));
        }
    }
    // THE RATIOS: the masked diagonal and `det J` scale TOGETHER, which is what block-triangularity
    // means — `det J = masked_diag * det(live 3x3)` and the live block is rung 71's, unmoved.
    let mut ratios: Vec<WindupRatio> = Vec::new();
    for ref_law in refs {
        let get = |t: f64| cells.iter().find(|((rl, tt), _)| rl == ref_law && *tt == t)
                                       .map(|(_, c)| c);
        let a = match get(tau_ts[0]) { Some(WindupCell::Read(a)) => a, _ => continue };
        // Python's `if len(tau_ts) > 1 else None` — with ONE clock the block is skipped entirely
        // rather than comparing a cell against itself.
        if tau_ts.len() < 2 {
            continue;
        }
        let b = match get(tau_ts[tau_ts.len() - 1]) { Some(WindupCell::Read(b)) => b, _ => continue };
        let pr: Vec<(&WindupRow, &WindupRow)> = a.rows.iter().zip(b.rows.iter())
            .filter(|(x, y)| x.s == y.s).collect();
        if pr.is_empty() {
            continue;
        }
        ratios.push(WindupRatio {
            ref_law,
            n: pr.len(),
            diag: (pr.iter().map(|(x, y)| y.masked_diag / x.masked_diag)
                     .fold(f64::INFINITY, f64::min),
                   pr.iter().map(|(x, y)| y.masked_diag / x.masked_diag)
                     .fold(f64::NEG_INFINITY, f64::max)),
            det: (pr.iter().map(|(x, y)| y.det / x.det).fold(f64::INFINITY, f64::min),
                  pr.iter().map(|(x, y)| y.det / x.det).fold(f64::NEG_INFINITY, f64::max)),
        });
    }
    WindupGains { phi_lim, taus, tau_ts: tau_ts.to_vec(), inc, ds, cells, ratios }
}

// ---------------------------------------------------------------------------------------------
// § 2 — `contraction_law`: RUNG 74's OWN RESIDUAL, EXPLAINED
// ---------------------------------------------------------------------------------------------

/// `self._ic_cap = ic_cap` … `finally: self._ic_cap = prev` — **the ONE reader in this family that
/// raises the cap, as a scope.**
///
/// A `Drop` guard rather than two assignments for the reason Python uses `finally`: the body it
/// wraps raises on four of its six arms in the general case, and a cap left raised would silently
/// change every later march in the same process. `Drop` runs on the unwind too, so the machine on
/// the far side of a caught panic is the machine that went in.
struct IcCapScope<'a> {
    core: &'a TwoSpoolTransientCore,
    prev: usize,
}

impl<'a> IcCapScope<'a> {
    fn set(core: &'a TwoSpoolTransientCore, cap: usize) -> Self {
        let prev = core.ic_cap.get();
        core.ic_cap.set(cap);
        IcCapScope { core, prev }
    }
}

impl Drop for IcCapScope<'_> {
    fn drop(&mut self) {
        self.core.ic_cap.set(self.prev);
    }
}

/// [`windup_march`] with Python's `except AssertionError` around it.
///
/// [`try_coord_march`](crate::demand_coordinate)'s shape and its reasoning verbatim:
/// `AssertUnwindSafe` is legitimate because every dynamically-scoped field on this core is restored
/// by `Drop`, which runs on the unwind. **The panic HOOK is not touched** — Python prints nothing
/// and Rust's default hook writes a line to stderr per caught panic, so the caught arms produce
/// stderr noise and no differing value.
///
/// **THE INDEX READ IS DELIBERATELY OUTSIDE THE CATCH.** Python's `traj[0]["ic_iters"]` sits inside
/// the `try`, but an `IndexError` is not an `AssertionError` and would propagate; catching it here
/// would turn an empty trajectory into a silent `None` that reads exactly like a converged-too-slow
/// cell.
#[allow(clippy::too_many_arguments)]
fn try_windup_march(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), r: f64, s_settle: f64, ds: f64, v_max: f64, inc: bool,
    coord: &'static str, ref_law: &'static str, law: &'static str, tau_t: Option<f64>,
) -> Option<Vec<FuelPoint>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        windup_march(core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
                     coord, ref_law, law, tau_t, None).3
    })).ok()
}

/// One row of [`contraction_law`] — one device clock, its ratio, and the count it predicts.
#[derive(Clone, Debug)]
pub struct ContractionRow {
    pub tau_t: f64,
    /// `sigma = tau_t / (taus[0] + tau_t)` — the fixed point's slope in the leg's own state.
    pub sigma: f64,
    /// `ceil(ln(tol/res0) / ln sigma)`, an INTEGER because Python's `math.ceil` returns one.
    pub predicted: i64,
    /// `traj[0]["ic_iters"]`, or `None` where the joint sweep refused.
    pub measured: Option<usize>,
    pub res: Option<f64>,
    pub within_inherited_cap: bool,
}

/// RUNG 75's `contraction_law` return.
#[derive(Clone, Debug)]
pub struct ContractionLaw {
    pub phi_lim: f64,
    pub taus: (f64, f64, f64, f64),
    pub res0: f64,
    pub tol: f64,
    pub ic_cap: usize,
    pub rows: Vec<ContractionRow>,
    pub n: usize,
    pub n_exact: usize,
    /// `bool(hit) and all(...)` — **the `bool(hit)` conjunct is Python's and it is load-bearing**:
    /// an `all()` over an empty set is `True`, so a port that dropped it would report a perfect
    /// score on a grid where nothing converged at all. Rung 78's vacuity trap, inherited.
    pub all_exact: bool,
}

/// RUNG 75 § 2 — **rung 74 § 4's `2.898e-3` was not a solver failing to find a plant; it was a
/// CONTRACTION WITH RATIO EXACTLY ONE**, and this reader is the derivation that says so.
///
/// The joint IC sweep is a fixed-point iteration. Adding the device changes its map's slope in the
/// leg's own state from `1` to
///
/// ```text
/// sigma = tau_t / (tau + tau_t)   < 1 for every FINITE tau_t,  -> 1 as tau_t -> inf
/// ```
///
/// so the residual falls GEOMETRICALLY and the sweep converges in `ceil(ln(tol/res0)/ln sigma)`
/// iterations — with `res0` **rung 74's own reported residual** and `tol` the inherited one. **Zero
/// fitted constants.**
///
/// So the `exists / does not exist` boundary the first probe showed is the 60-iteration cap cutting
/// a geometric sequence, not a property of any plant: the park law gives a FINITE equilibrium at
/// every finite `tau_t`, and rung 74's cell is the `sigma = 1` limit where the residual has nowhere
/// to go. **Rung 74's verdict stands and its number is explained.**
///
/// # THE CAP IS RAISED HERE AND ONLY HERE
///
/// `_ic_cap` defaults to the inherited `60` on every plant in this family; this reader is measuring
/// a DERIVED count and needs room for it. On the shipped grid the two slowest clocks predict **185**
/// and **98** — both above 60 — which is what the `400` is for and what `within_inherited_cap`
/// reports per row.
///
/// # THE SIX PREDICTIONS ARE ARITHMETIC, AND THEY WERE DERIVED BEFORE THE PORT RAN
///
/// `ln(1e-12/2.898e-3) = -21.7874…`, so at `taus[0] = 0.05` the grid
/// `(0.4, 0.2, 0.1, 0.05, 0.025, 0.0125)` gives `sigma` of `8/9, 4/5, 2/3, 1/2, 1/3, 1/5` and
/// predictions **185, 98, 54, 32, 20, 14**. Those are typed here from the closed form and not read
/// off any output — [[instrument-fed-by-what-it-certifies]] is the standing reason.
#[allow(clippy::too_many_arguments)]
pub fn contraction_law(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, taus: (f64, f64, f64, f64), tau_ts: &[f64], res0: f64, tol: f64, ic_cap: usize,
    inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> ContractionLaw {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let mut rows: Vec<ContractionRow> = Vec::new();
    for &tau_t in tau_ts {
        let sigma = tau_t / (taus.0 + tau_t);
        // `math.ceil` returns an `int`; `f64::ceil` returns a float, so the cast is where the two
        // spellings meet and the comparison against `measured` is integral on both sides.
        let pred = ((tol / res0).ln() / sigma.ln()).ceil() as i64;
        let (got, res) = {
            let _ic = IcCapScope::set(&core.fuel.inner, ic_cap);
            match try_windup_march(core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle,
                                   ds, v_max, inc, LAG_COORD_DEMAND, REF_APPLIED,
                                   WINDUP_LAW_TRACK, Some(tau_t)) {
                None => (None, None),
                Some(traj) => match traj[0].extra {
                    PointExtra::Demand { ic_iters, ic_res, .. }
                    | PointExtra::Shared { ic_iters, ic_res, .. } =>
                        (Some(ic_iters), Some(ic_res)),
                    _ => panic!("rung-75's contraction reads `ic_iters`/`ic_res` off the joint \
                                 initial condition, which only a rung-72-or-later point carries."),
                },
            }
        };
        rows.push(ContractionRow {
            tau_t, sigma, predicted: pred, measured: got, res,
            within_inherited_cap: pred <= 60,
        });
    }
    let hit: Vec<&ContractionRow> = rows.iter().filter(|x| x.measured.is_some()).collect();
    ContractionLaw {
        phi_lim, taus, res0, tol, ic_cap,
        n: hit.len(),
        n_exact: hit.iter()
                    .filter(|x| x.measured.expect("filtered") as i64 == x.predicted).count(),
        all_exact: !hit.is_empty()
                   && hit.iter().all(|x| x.measured.expect("filtered") as i64 == x.predicted),
        rows,
    }
}

// ---------------------------------------------------------------------------------------------
// § 3 — `device_control`: THE ACCIDENT AND THE DEVICE, WHERE NOTHING IS CUTTING
// ---------------------------------------------------------------------------------------------

/// One cell of [`device_control`] — one `(ref, tau_t)` pair, the device against the accident.
#[derive(Clone, Debug)]
pub struct DeviceCell {
    pub ref_law: &'static str,
    pub tau_t: f64,
    pub n_dormant: usize,
    pub n_cutting: usize,
    /// `max |a - b|` over `mf`, `Tt4`, `nu_lp` on the DORMANT indices — **exactly `0.0` on the
    /// shipped grid**, which is the half of anchor P8 that held.
    pub dormant_output: Option<f64>,
    /// The same over `w_fuel`, `w_gov` — **NOT zero**, which is the half that was REFUTED. A gate
    /// that read only the output would confirm the prediction while missing that it was wrong.
    pub dormant_state: Option<f64>,
    pub cutting_output: Option<f64>,
}

/// RUNG 75's `device_control` return.
#[derive(Clone, Debug)]
pub struct DeviceControl {
    pub phi_lim: f64,
    pub taus: (f64, f64, f64, f64),
    pub inc: bool,
    pub cells: Vec<DeviceCell>,
}

/// RUNG 75 § 3, **THE CONTROL ROW — and it is where this rung's own prediction died.**
///
/// ANCHOR P8 SAID the two devices coincide where no leg is cutting, because there
/// `mf_app = mf_sched` and the tracker pulls to exactly where the latch clamps. **REFUTED ON THE
/// STATE, HELD EXACTLY ON THE OUTPUT**, and the reason is the park law the same anchor derived two
/// lines earlier: the tracking term pulls toward `mf_app`, but the TARGET term still pushes toward
/// `cap`, and with `cap > mf_sched` (rung 74 § 0.2 measures `1.303x` at `s = 0`) the balance sits
/// ABOVE the schedule. The latch clamps AT it.
///
/// So the honest statement is a distinction, not an equality: **while nothing is cutting the two
/// devices burn identically — 0.0, exactly — and their STATES never agree at all**, with the gap
/// scaling as the park law's `tau_t/tau` says it must.
///
/// # THE TWO INDEX SETS ARE NOT SYMMETRIC, AND THAT IS PYTHON's LINE
///
/// `dorm` requires BOTH trajectories dormant at `i`; `cut` requires only `a[i]`. A port that
/// symmetrised either one would move points between two sets that do not partition the march, and
/// `cutting_output` — which can legitimately be `None` — would start reading as a coverage
/// statement about the accident rather than about the device.
#[allow(clippy::too_many_arguments)]
pub fn device_control(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, taus: (f64, f64, f64, f64), tau_ts: &[f64], refs: &[&'static str], inc: bool,
    r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> DeviceControl {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let mut cells: Vec<DeviceCell> = Vec::new();
    for ref_law in refs {
        for &tau_t in tau_ts {
            let a = windup_march(core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds,
                                 v_max, inc, LAG_COORD_DEMAND, ref_law, WINDUP_LAW_TRACK,
                                 Some(tau_t), None).3;
            let b = windup_march(core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds,
                                 v_max, inc, LAG_COORD_LATCHED, ref_law, WINDUP_LAW_NONE, None,
                                 None).3;
            let n = a.len().min(b.len());
            let auth_of = |p: &FuelPoint| match p.extra {
                PointExtra::Demand { authority, .. } | PointExtra::Shared { authority, .. } =>
                    authority,
                _ => panic!("rung-75's control reads `authority` off every marched point."),
            };
            let dorm: Vec<usize> = (0..n).filter(|&i| auth_of(&a[i]) == Authority::Dormant
                                                   && auth_of(&b[i]) == Authority::Dormant)
                                         .collect();
            let cut: Vec<usize> = (0..n).filter(|&i| matches!(auth_of(&a[i]),
                                                              Authority::Fuel | Authority::Gov))
                                        .collect();
            // Python's `ok` and `sk`, in its order.
            let out_keys: [fn(&FuelPoint) -> f64; 3] = [|p| p.mf, |p| p.tt4, |p| p.nu_lp];
            let state_keys: [fn(&FuelPoint) -> f64; 2] = [
                |p| match p.extra {
                    PointExtra::Demand { w_fuel, .. } => w_fuel,
                    _ => panic!("rung-75's control reads `w_fuel` off a demand point."),
                },
                |p| match p.extra {
                    PointExtra::Demand { w_gov, .. } => w_gov,
                    _ => panic!("rung-75's control reads `w_gov` off a demand point."),
                },
            ];
            let worst = |idx: &[usize], keys: &[fn(&FuelPoint) -> f64]| -> Option<f64> {
                if idx.is_empty() {
                    return None;
                }
                let mut m = f64::NEG_INFINITY;
                for &i in idx {
                    for k in keys {
                        m = m.max((k(&a[i]) - k(&b[i])).abs());
                    }
                }
                Some(m)
            };
            cells.push(DeviceCell {
                ref_law, tau_t,
                n_dormant: dorm.len(),
                n_cutting: cut.len(),
                dormant_output: worst(&dorm, &out_keys),
                dormant_state: worst(&dorm, &state_keys),
                cutting_output: worst(&cut, &out_keys),
            });
        }
    }
    DeviceControl { phi_lim, taus, inc, cells }
}

// ---------------------------------------------------------------------------------------------
// § 4 — `windup_bill`: THE THRESHOLD ON THE ONE NEW CONSTANT
// ---------------------------------------------------------------------------------------------

/// One row of [`windup_bill`] — one device clock and what it bought.
#[derive(Clone, Debug)]
pub struct BillRow {
    pub tau_t: f64,
    /// `tau_t / taus[0]` — the comparison that makes the threshold a RATIO rather than a decimal.
    pub ratio: f64,
    pub max_tt4: f64,
    pub over: f64,
    pub holds: bool,
    pub min_phi: f64,
    /// The first `s` at which the GOVERNOR takes the actuator, or `None` if it never does.
    pub handover: Option<f64>,
}

/// The accident's own row — rung 52's `max(0, .)` with no declared device.
#[derive(Clone, Debug)]
pub struct BillAccident {
    pub max_tt4: f64,
    pub over: f64,
    pub min_phi: f64,
    pub handover: Option<f64>,
}

/// RUNG 75's `windup_bill` return.
#[derive(Clone, Debug)]
pub struct WindupBill {
    pub phi_lim: f64,
    pub taus: (f64, f64, f64, f64),
    pub ref_law: &'static str,
    pub inc: bool,
    pub ds: f64,
    pub rows: Vec<BillRow>,
    pub accident: BillAccident,
    /// THE THRESHOLD: the last clock that holds the redline and the first that breaks it.
    pub tau_t_holds: Option<f64>,
    pub tau_t_breaks: Option<f64>,
    pub ratio_holds: Option<f64>,
    pub ratio_breaks: Option<f64>,
    /// **P9** — monotone in the sweep order, and the direction discriminates the mechanism.
    ///
    /// **VACUOUSLY TRUE ON A GRID WHERE AT MOST ONE ROW HANDS OVER**, because the `None`s are
    /// filtered out first. Python has no key for the surviving count, and neither does this: the
    /// count is recoverable from [`rows`](WindupBill::rows) and the gate is where it is asserted.
    pub handover_monotone: bool,
    pub handover_span: Option<(f64, f64)>,
    pub handover_vs_accident: Option<((f64, f64), Option<f64>)>,
    pub tt4_monotone: bool,
}

/// RUNG 75 § 4 — **the device's whole delivered credit is a THRESHOLD ON ITS OWN CLOCK**, and the
/// comparison that matters is against the ACCIDENT, not against `tau_t`.
///
/// Rung 47's headline concession — *the cost of realism is that a lagged governor breaks the
/// redline hold* — was corrected by rung 74 into a property of the COORDINATE. This rung finds the
/// third layer: within the demand coordinate, whether the redline holds is a **threshold on
/// `tau_t`**, the one constant this rung adds and cannot derive. Rung 54's shape (*every verdict is
/// a threshold ON the disclosed constant*), now on a clock.
///
/// # THE FASTEST CLOCK ON THE SHIPPED GRID IS THE DERIVED FLOOR ITSELF
///
/// `tau_ts[0]` is `0.00625`, which is what [`WINDUP_TAU_GRID_FLOOR`] evaluates to at the inherited
/// `ds = 0.005` and four clocks of `0.05`. The grid therefore RUNS ON its own admissibility
/// boundary, so whether `_rk4_floor_shared` admits equality is load-bearing rather than a detail —
/// and the const is spelled as the derivation rather than as a decimal precisely so the two sides
/// of that comparison cannot drift apart.
#[allow(clippy::too_many_arguments)]
pub fn windup_bill(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, taus: (f64, f64, f64, f64), tau_ts: &[f64], ref_law: &'static str, inc: bool,
    r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> WindupBill {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let hand = |traj: &[FuelPoint]| -> Option<f64> {
        traj.iter().find(|p| match p.extra {
            PointExtra::Demand { authority, .. } | PointExtra::Shared { authority, .. } =>
                authority == Authority::Gov,
            _ => panic!("rung-75's bill reads `authority` off every marched point."),
        }).map(|p| p.s)
    };
    let mut rows: Vec<BillRow> = Vec::new();
    for &tau_t in tau_ts {
        let traj = windup_march(core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds,
                                v_max, inc, LAG_COORD_DEMAND, ref_law, WINDUP_LAW_TRACK,
                                Some(tau_t), None).3;
        let mx = traj.iter().map(|p| p.tt4).fold(f64::NEG_INFINITY, f64::max);
        rows.push(BillRow {
            tau_t,
            ratio: tau_t / taus.0,
            max_tt4: mx,
            over: mx - tt4_max,
            holds: mx <= tt4_max,
            min_phi: traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min),
            handover: hand(&traj),
        });
    }
    let acc = windup_march(core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max,
                           inc, LAG_COORD_LATCHED, ref_law, WINDUP_LAW_NONE, None, None).3;
    let acc_mx = acc.iter().map(|p| p.tt4).fold(f64::NEG_INFINITY, f64::max);
    let hs: Vec<f64> = rows.iter().filter_map(|x| x.handover).collect();
    let span = if hs.is_empty() { None }
               else { Some((hs.iter().copied().fold(f64::INFINITY, f64::min),
                            hs.iter().copied().fold(f64::NEG_INFINITY, f64::max))) };
    WindupBill {
        phi_lim, taus, ref_law, inc, ds,
        accident: BillAccident {
            max_tt4: acc_mx,
            over: acc_mx - tt4_max,
            min_phi: acc.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min),
            handover: hand(&acc),
        },
        tau_t_holds: rows.iter().filter(|x| x.holds).map(|x| x.tau_t)
                         .fold(None, |m: Option<f64>, t| Some(m.map_or(t, |m| m.max(t)))),
        tau_t_breaks: rows.iter().filter(|x| !x.holds).map(|x| x.tau_t)
                          .fold(None, |m: Option<f64>, t| Some(m.map_or(t, |m| m.min(t)))),
        ratio_holds: rows.iter().filter(|x| x.holds).map(|x| x.ratio)
                         .fold(None, |m: Option<f64>, t| Some(m.map_or(t, |m| m.max(t)))),
        ratio_breaks: rows.iter().filter(|x| !x.holds).map(|x| x.ratio)
                          .fold(None, |m: Option<f64>, t| Some(m.map_or(t, |m| m.min(t)))),
        handover_monotone: hs.windows(2).all(|w| w[0] <= w[1]),
        handover_span: span,
        handover_vs_accident: span.map(|s| (s, hand(&acc))),
        tt4_monotone: rows.windows(2).all(|w| w[0].max_tt4 <= w[1].max_tt4),
        rows,
    }
}
