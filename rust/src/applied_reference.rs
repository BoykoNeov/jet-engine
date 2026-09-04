//! RUNG 73 — **THE APPLIED REFERENCE.** `AppliedReferenceTransient`, slice AE.
//!
//! Rung 72's two fuel-side legs both compute their clip from the SCHEDULED fuel, which is what
//! makes `F_r = R_f = 0` EXACTLY and the four-loop plant block-triangular. Its § 11 named the
//! seam: *a leg that reads the APPLIED fuel gives `F_r != 0`, couples the two fuel rows, and
//! destroys the block form.* **The coupling is real and it lands in the WRONG COLUMN** — `F_r`
//! sits in the AUTHORITATIVE leg's column, and under min-select the masked column is zero under
//! every reference, so triangularity was min-select's all along. What the reference buys is the
//! POLE: rung 72's free pole at `-1/tau_masked` moves to EXACTLY the origin.
//!
//! # WHAT STEP 1 OF THIS SLICE ADDS — **SIX RE-AIMED POINTERS, ZERO NEW TABLE FIELDS**
//!
//! | | Python | slot | table |
//! |---|---|---|---|
//! | swap | `_reference` | [`reference`](TripleHooks::reference) | [`R73_TRIPLE`] |
//! | swap | `_with_ref` | [`with_ref`](TripleHooks::with_ref) | [`R73_TRIPLE`] |
//! | swap | `_rk4_floor_shared` | [`rk4_floor_shared`](TripleHooks::rk4_floor_shared) | [`R73_TRIPLE`] |
//! | swap | `_shared_rig` | [`shared_rig`](TripleHooks::shared_rig) | [`R73_TRIPLE`] |
//! | swap | `at_lever` | `LeverHooks::at_lever` | [`R73`] |
//! | swap | `integrate_fuel` | `FuelTransientHooks::integrate_fuel` | [`R73_FUEL`] |
//!
//! **§ 5.29 (ix)'s P7, restated after its own repair, says `TripleHooks` stays at 13. It does —
//! for these six.** All six re-aim a slot that already exists, so no table grows and neither
//! width tripwire fires. The prediction is nevertheless **already known false for the SEVENTH
//! pointer**, and that is written down here rather than discovered at step 2: § 5.29 (iv) commits
//! this slice to installing `_quad_gains_at` as a cell, and that name has **no field in any of the
//! five table types** — measured by grep over `TripleHooks`, `LeverHooks`, `FuelTransientHooks`,
//! `StatorTransientHooks` and `TwoSpoolTransientHooks`; it is a free `pub fn` in
//! [`shared_actuator`](crate::shared_actuator). So **step 2 takes `TripleHooks` 13 → 14**, and
//! P7's repair fixed one of the two inconsistencies it was written against and left the other.
//! That is § 5.29 (x)'s sixth defect — *two claims individually plausible and jointly impossible*
//! — arriving a second time inside the same section.
//!
//! # THE NAME REUSE, AND WHY THE PORT IS THE RE-AIM AND NOT A SECOND FIELD
//!
//! `_with_ref` is defined at rung 69 and again here with an **identical signature** and a
//! **different mutated field**: rung 69 writes `_ref`, rung 73 writes `_ref_law`. Both fields
//! exist on a rung-73 machine, so nothing type-errors and no signature comparison can see it —
//! which is why § 5.27 (x)'s phase-wide sweep classified the pair as harmlessly RENAMED.
//!
//! **The port had already reached the right structure from the right observation.**
//! [`RefScope`](crate::reference_split::RefScope) goes through the
//! [`with_ref`](TripleHooks::with_ref) cell *because* rung 73 moves the field, and says so;
//! `cross_split.rs`'s [`GovScope`](crate::cross_split::GovScope) states the MIRROR of that
//! reasoning — it writes its field DIRECTLY, because `_with_gov` is defined once in the whole
//! ladder and there is no second field for a cell to choose between.
//!
//! **THAT SENTENCE USED TO NAME `cross_split.rs`'s `CoordScope`, WHICH HAS NEVER EXISTED.**
//! `git log -S "CoordScope"` returns exactly one commit — the one that wrote the sentence — so the
//! name was invented in prose, and the substance was wrong with it: the type it meant writes its
//! field directly, which the sentence credited as REPEATING the cell decision when the guard's own
//! doc calls it *the opposite*. Corrected at slice AF, whose § (i) is this defect class one slice
//! on, and urgently: that slice introduces the crate's first real
//! [`CoordScope`](crate::demand_coordinate::CoordScope), so the stale reference would have
//! resolved to a live type and told a reader a wrong story. What was missing is
//! not the cell — it is **the REFUSAL**. A re-aimed slot on its own means a rung-69 reader run on
//! a rung-73 machine writes `ref_law`, leaves `ref_` at `None`, falls through `triple_rig`'s
//! `self._ref or (…)` fallback, and marches a plant nobody asked for **silently**. Python's loud
//! failure is [`r73_integrate_fuel`]'s first assert, and it is ported here, at the step that
//! re-aims the slot, rather than at the step that gates the dispatch.
//!
//! # THE CLASS DEFAULT IS THIS STEP'S OTHER SILENT FAILURE, AND IT IS A ONE-LINE ONE
//!
//! Python declares `_ref_law = "applied"` as a rung-73 class attribute; the port's constructor
//! initialises [`ref_law`](crate::two_spool_transient::TwoSpoolTransientCore::ref_law) to
//! [`REF_LAW_DEFAULT`](crate::shared_actuator::REF_LAW_DEFAULT) = `"sched"` **unconditionally**.
//! A builder that did not overwrite it would hand back a machine that
//!
//! * **passes** [`r73_integrate_fuel`]'s refusal, because `"sched"` is one of the two declared
//!   laws;
//! * marches rung 72's plant, because `_reference` is then the identity;
//! * and reports rung 73 in every reader.
//!
//! Measured on the source (probe L1): a fresh `AppliedReferenceTransient` reads `'applied'` and a
//! fresh `SharedActuatorTransient` reads `'sched'`. [`build_applied_reference_cascade`] sets it;
//! [`r73_at_lever`] and [`r73_shared_rig`] then **overwrite from the source core**, because a
//! sibling built inside a `RefScope`-set `"sched"` must be `"sched"` and not the class default.
//!
//! # A MEASURED NO-OP, DISCLOSED RATHER THAN FACTORED AWAY
//!
//! Python's `_shared_rig` override sets `m._ref_law = self._ref_law` on the machine
//! `super()._shared_rig` handed back — but rung 72's body reaches that machine through
//! `self.at_lever(…)`, which at rung 73 is [`r73_at_lever`] and **already carried the law**.
//! Driven both ways on the same receiver (probe L2), with the override and with rung 72's body
//! called directly on a rung-73 `self`, under both `'applied'` and `'sched'`: **the two agree, so
//! the override is a NO-OP.** It is ported unchanged anyway ([[rust-port-copy-vs-rederivation]] —
//! a deliberate duplication the source makes is not the port's to factor out), and the fact is
//! pre-registered for step 5: **this swap has no value break**, so no discriminator exists to
//! hunt for.
//!
//! # THE FLOAT-IDENTICAL BRANCH IS LOAD-BEARING AND MUST NOT BE FOLDED AWAY
//!
//! `_reference`'s middle branch returns `req` ITSELF when the leg holds the actuator. `g_own + req
//! - g_own` is not `req` in binary floating point, and through a central difference of step `1e-7`
//! the cancellation appears as a `4e-11` entry on the authoritative leg's own diagonal — which
//! turns *`M3` is the parent's block ENTRY FOR ENTRY* from an exact claim into a `1e-11` one.
//! **That sits below every relative bar in the crate**, so the branch is gated on `to_bits` and
//! never on a tolerance. This is rung 48's `_sched_fuel` device, second instance.
//!
//! § 5.29 (iii) measured all three paths live over the whole shipped rung-73 suite: 41 346 calls
//! take the `ref_law != "applied"` path, 109 537 the float-identity path (every one returning
//! `req` bitwise), and **109 307 — 42.01 % — the arithmetic path, not one of which returns
//! `req`.** So the dispatch gates at step 5 are plain value gates.

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelLimiters, FuelPoint, FuelTransientCore,
    Authority, FuelTransientHooks, PointExtra,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::reference_split::{c_add, c_real, opt_fold, RefScope, C64};
use crate::shared_actuator::{
    applied_clip, applied_clip_core, assert_fuel_boundary, authority, charpoly4, jac4, leg4,
    py_running_max, quad_laws, quartic_roots_c, reg4, riding4, BoundaryCheck, QuadGains,
    SharedBill, SharedRigArm,
};
use crate::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use crate::three_loop::{LegRegime, TripleHooks};
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{TwoSpoolTransientCore, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// THE DECLARED CONSTANT — rung 73's one class attribute
// ---------------------------------------------------------------------------------------------

/// Python's `_ref_law = "applied"` — **READING B, and the only thing this rung declares.**
///
/// Rung 72 declares the same name with `"sched"`
/// ([`REF_LAW_DEFAULT`](crate::shared_actuator::REF_LAW_DEFAULT)) and reads it nowhere; this rung
/// re-declares it and three of its own bodies read it. The two spellings are kept as two
/// constants rather than one parameterised default because they are two class attributes in the
/// source, and because the difference between them is exactly what
/// [`build_applied_reference_cascade`]'s overwrite exists to install.
///
/// **A `&'static str` and not an enum**, [`ref_law`]'s own reason: Python compares against the
/// literals `"sched"` / `"applied"` and ASSERTS on anything else, and that shipped refusal is what
/// a two-variant enum would delete — along with the whole of § 5.29 (i)'s finding, whose entire
/// mechanism is a rung-69 reader writing `"inc"` into this field.
///
/// [`ref_law`]: crate::two_spool_transient::TwoSpoolTransientCore::ref_law
pub const REF_LAW_APPLIED: &str = "applied";

/// The two laws [`r73_integrate_fuel`]'s first refusal admits, in Python's tuple order.
///
/// Named so the refusal and the gate that drives it read the same list — the pair is quoted in
/// the message, and a message that drifted from the test would be a needle for a refusal that no
/// longer exists.
pub const REF_LAWS_DECLARED: [&str; 2] = ["sched", "applied"];

// ---------------------------------------------------------------------------------------------
// THE CASCADE BUILDER
// ---------------------------------------------------------------------------------------------

/// Build a rung-73 object, so every sibling re-asserts the whole chain's guards — and **set the
/// class default, which is the one thing this builder does that rung 72's does not.**
///
/// See the module header: the core's constructor writes `"sched"` for every rung in the family,
/// and a rung-73 machine that kept it would pass its own refusal while marching rung 72.
pub fn build_applied_reference_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = crate::reference_split::build_split_family_cascade(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
        &R73_TWO, &R73_STATOR, &R73_FUEL, &R73, &R73_TRIPLE);
    // Python's CLASS ATTRIBUTE, applied where the object is made. Only the `Full` arm has a core
    // to carry it — `lp_disabled` hands back rung 34/35's single-spool object, which has no fuel
    // reference at all ([`with_ref_tables`]'s handling of `stator_inc`, verbatim).
    if let ScheduledStatorTransient::Full(c) = &built {
        c.fuel.inner.ref_law.set(REF_LAW_APPLIED);
    }
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and THREE of them carry something of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 73's lever table — ONE swap, `at_lever`, and the parent it must differ from is rung 72's.
///
/// The TWELFTH instance of the sibling-constructor trap, and the first where it **grows a second
/// head**: handing back the parent's class reports rung 73 while measuring rung 72, and handing
/// back the right class while dropping the reference does the same thing one level down, in every
/// ledger cell, silently.
pub const R73: LeverHooks = LeverHooks {
    at_lever: r73_at_lever,
    ..crate::shared_actuator::R72
};

/// RUNG 73's `TwoSpoolTransientHooks` — **ZERO cells swapped**, an alias; width pinned by the
/// tripwire named at `R70_TWO`.
pub const R73_TWO: TwoSpoolTransientHooks = crate::shared_actuator::R72_TWO;

/// RUNG 73's fuel table — ONE swap, `integrate_fuel`: **two refusals, then the parent's march.**
pub const R73_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r73_integrate_fuel,
    ..crate::shared_actuator::R72_FUEL
};

/// RUNG 73's stator table — **ZERO cells swapped**, an alias; width pinned as [`R73_TWO`]'s is.
pub const R73_STATOR: StatorTransientHooks = crate::shared_actuator::R72_STATOR;

/// RUNG 73's third-loop table — **FOUR of rung 72's thirteen cells re-aimed, NONE added.**
///
/// Spelled out field by field rather than reached through a `..R72_TRIPLE` spread, for
/// [`R72_TRIPLE`](crate::shared_actuator::R72_TRIPLE)'s stated reason and with more force here:
/// only 4 of 13 change, so nine INHERITED decisions sit on the page as decisions instead of as
/// the residue of a spread. It is also this slice's share of the width tripwire — an exhaustive
/// literal is what goes loud when step 2 takes the struct to 14.
pub const R73_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::shared_actuator::R72_TRIPLE.stator_leg,
    lagged_stator: crate::shared_actuator::R72_TRIPLE.lagged_stator,
    clamp_v: crate::shared_actuator::R72_TRIPLE.clamp_v,
    check_v0: crate::shared_actuator::R72_TRIPLE.check_v0,
    rk4_floor: crate::shared_actuator::R72_TRIPLE.rk4_floor,
    solve_v: crate::shared_actuator::R72_TRIPLE.solve_v,
    manifold_v: crate::shared_actuator::R72_TRIPLE.manifold_v,
    triple_laws: crate::shared_actuator::R72_TRIPLE.triple_laws,
    triple_rig: crate::shared_actuator::R72_TRIPLE.triple_rig,
    // THE FOUR THIS STEP RE-AIMS. `with_ref` is rung 69's slot pointed at a DIFFERENT FIELD;
    // the other three are rung 72's slots pointed at rung 73's bodies.
    with_ref: r73_with_ref,
    reference: r73_reference,
    rk4_floor_shared: r73_rk4_floor_shared,
    shared_rig: r73_shared_rig,
    // AND THE SEVENTH POINTER, WHICH STEP 1 PRE-REGISTERED AS THE ONE THAT WOULD BREAK P7.
    // `_quad_gains_at` had no field in any of the five table types, so this is the slice's ONE
    // added cell and `TripleHooks` goes 13 -> 14 here. Step 1 wrote that down rather than meeting
    // it as a surprise.
    quad_gains_at: r73_quad_gains_at,
    // NONE OF SLICE AF's FOUR — all four names arrive at rung 74, so this rung carries
    // `NO_TRIPLE`'s refusal for each. Reached through rung 68's table, which is where the
    // panicking slots live.
    cap_fuel: crate::three_loop::R68_TRIPLE.cap_fuel,
    sensed_cap: crate::three_loop::R68_TRIPLE.sensed_cap,
    windup_tau: crate::three_loop::R68_TRIPLE.windup_tau,
    with_coord: crate::three_loop::R68_TRIPLE.with_coord,
};

// ---------------------------------------------------------------------------------------------
// THE SIX RE-AIMED BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 73's `at_lever` — **rung 72's sibling constructor returning a RUNG-73 machine THAT CARRIES
/// THE REFERENCE.**
///
/// Twelfth instance of the trap, second head. The class is half of the fix and the law is the
/// other half: a rung-73 sibling built while the receiver sits under a `RefScope`-set `"sched"`
/// must be `"sched"`, so the value is copied from the SOURCE core and never left at
/// [`REF_LAW_APPLIED`] — which is why the copy is here and not only in the builder.
fn r73_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let m = match build_applied_reference_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    };
    m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
    m
}

/// RUNG 73's `_with_ref` — **THE NAME REUSE, and the whole of it is WHICH FIELD IS WRITTEN.**
///
/// Rung 69's body writes [`ref_`]; this one writes [`ref_law`] and changes nothing else — same
/// arity, one parameter renamed (`ref` → `law`), same `try/finally`. Both fields exist on a
/// rung-73 machine, which is why no signature comparison and no type error can reach this and why
/// the discriminator has to be DRIVEN: *does the inherited caller still work on the downstream
/// machine?* It does not — [`r73_integrate_fuel`]'s first refusal is the answer, and it is the
/// only thing standing between the re-aim and a silent wrong plant.
///
/// # `None` IS REFUSED HERE, WHERE PYTHON REFUSES IT AT THE MARCH
///
/// The shared guard's signature is `Option<&'static str>` because rung 69's field is optional;
/// this rung's is not — Python declares `_ref_law` as a plain `str` with no unset state, so
/// `Cell<&'static str>` has nowhere to put a `None`. Python would happily assign it and then raise
/// out of `integrate_fuel` with *"got None"*; here the refusal is taken at the setter, and the
/// divergence is disclosed rather than hidden because the failure MODE differs even though the
/// outcome (a loud stop before any march) does not.
///
/// **AND IT CANNOT FIRE FROM A `Drop`.** This body always returns `Some(prev)` — `ref_law` has no
/// empty state — so [`RefScope`](crate::reference_split::RefScope)'s restore never feeds `None`
/// back in, and a panic here can never land inside an unwind. That is a reachability MEASUREMENT
/// and not a hope, and it is what makes the panic admissible at all.
///
/// [`ref_`]: crate::two_spool_transient::TwoSpoolTransientCore::ref_
/// [`ref_law`]: crate::two_spool_transient::TwoSpoolTransientCore::ref_law
fn r73_with_ref(t: &TwoSpoolTransientCore, law: Option<&'static str>) -> Option<&'static str> {
    let law = law.unwrap_or_else(|| {
        panic!("rung-73: `_with_ref` writes `_ref_law`, which Python declares as a plain `str` \
                with no unset state, so there is no `None` to write. Rung 69's `_with_ref` \
                writes `_ref` and takes one; this is the NAME REUSE, not the same cell. If you \
                meant rung 69's reference, you are holding the wrong machine.")
    });
    let prev = t.ref_law.get();
    t.ref_law.set(law);
    Some(prev)
}

/// RUNG 73's `_reference` — **READING B, and the ONE place the reference lives.**
///
/// Three paths, and § 5.29 (iii) drove all three over the whole shipped suite:
///
/// | path | condition | calls | returns `req` bitwise |
/// |---|---|---|---|
/// | 1 | `ref_law != "applied"` | 41 346 | 41 346 of 41 346 |
/// | 2 | `clip == g_own` | 109 537 | 109 537 of 109 537 |
/// | 3 | otherwise | 109 307 | **0 of 109 307** |
///
/// # PATH 1 IS A DISPATCH AND NOT A FORMALITY
///
/// The first version of this method in Python applied reading B unconditionally, so
/// `_with_ref('sched', ·)` was a NO-OP and every A-vs-B reader differenced the plant against
/// ITSELF. It did not fail — it returned a PERFECT confirmation of the rung's headline from an
/// instrument that had measured nothing. The port inherits both the dispatch and the reason.
///
/// # PATH 2 IS A FLOAT-IDENTITY DEVICE, AND PATH 3's ASSOCIATION IS PINNED
///
/// Path 2 exists because `g_own + req - g_own` is not `req` in binary floating point; see the
/// module header for the `4e-11` it would otherwise put on the authoritative diagonal.
///
/// Path 3 is spelled `(g_own + req) - clip`, which is Python's association, and the parenthesis is
/// explicit because `req + (g_own - clip)` is a DIFFERENT float and is exactly the rearrangement a
/// later tidy-up writes. Measured (probe L4): at `g_own = 1e16, req = 1.0, clip = 1e16` the shipped
/// order gives `0.0` and the rearrangement `1.0`.
fn r73_reference(
    t: &TwoSpoolTransientCore, req: f64, g_own: f64, gf: f64, gr: f64,
) -> f64 {
    if t.ref_law.get() != REF_LAW_APPLIED {
        return req;
    }
    // THE SAME BODY THE MARCH USES, never a second spelling of the same algebra: the test below
    // is a float IDENTITY, so a re-derived clip would be comparing against a different rounding.
    let clip = applied_clip_core(t, gf, gr);
    if clip == g_own {
        return req;
    }
    // PYTHON'S ASSOCIATION, PINNED. Do not rewrite as `req + (g_own - clip)`.
    (g_own + req) - clip
}

/// RUNG 73's `_rk4_floor_shared` — **the floor, re-justified a SIXTH time, and the previous five
/// do NOT carry.**
///
/// Rung 72 argued *the masked leg's eigenvalue is exactly `-1/tau_f` and the other three share the
/// remainder*. Here the masked leg's eigenvalue is exactly **ZERO** — neutrally stable — so *the
/// dominant root is below the rate sum* is no longer the sentence. The new one: `lam = 0` is
/// interior to every explicit stability region at every step size, and the remaining three share a
/// trace `1/tau_masked` MORE negative than rung 72's, so the inherited constant is **more**
/// conservative here.
///
/// **THE CONDITION IS `ds * rate <= 2.0` IN RUNGS 72, 73 AND 74 CHARACTER FOR CHARACTER, SO THE
/// MESSAGE IS THE ENTIRE CELL** — and the shipped Python needle is worse here than at rung 72.
/// § 5.29 (vii) re-measured it over all 58 ladder classes with the names EMITTED: `"FOUR actuator
/// states"` reaches **nine** classes, back to rung 43. The tokens a gate may read are `rung-73`
/// and `origin`, which § 5.29 (vii) measured unique to this class.
fn r73_rk4_floor_shared(ds: f64, rate: f64) {
    assert!(
        ds * rate <= 2.0,
        "rung-73: ds*sum(1/tau_i) = {:.3} is outside the explicit RK4 stability region for the \
         FOUR actuator states (ds = {}). The masked leg contributes a pole at EXACTLY the origin \
         -- neutrally stable, interior to every region -- and the other three share a trace more \
         negative than rung 72's, so the inherited constant is more conservative here. Refine \
         the grid or slow a clock.",
        ds * rate, ds);
}

/// RUNG 73's `integrate_fuel` — **TWO REFUSALS, AND THEY ARE THE POINT OF THIS STEP.**
///
/// # THE FIRST IS WHAT MAKES THE NAME REUSE LOUD
///
/// [`r73_with_ref`] re-aims a slot rung 69's readers also use. Without this assert, a rung-69
/// reader on a rung-73 machine writes `"inc"` into `ref_law`, leaves `ref_` at `None`, falls
/// through `triple_rig`'s `self._ref or (…)` fallback and marches a plant nobody asked for — with
/// no panic, no wrong type and no value key that could see it. **The refusal IS the port's
/// correctness here**, which is why it lands at the step that re-aims the slot.
///
/// # AND BOTH FIRE BEFORE ANY ENTRY TEST — MEASURED, NOT ASSUMED
///
/// Rung 72's body early-returns into rung 71's table when no governor clock or no fuel leg is
/// armed. Python's two asserts sit ABOVE the `super()` call, so they precede that test; driven on
/// the source (probe L5) with neither leg armed, `_ref_law = 'inc'` still raises. Putting them
/// after a copied entry test would skip the refusal on exactly the arming a rung-69 reader is
/// likeliest to build.
///
/// # THE DELEGATION IS THE IMMEDIATE PARENT'S TABLE
///
/// `super()` from this class is rung 72, so the reduce goes through
/// [`R72_FUEL`](crate::shared_actuator::R72_FUEL) and not through a grandparent spelling that is
/// only ACCIDENTALLY the same pointer today — rung 71's rule, two rungs on.
fn r73_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    let ref_law = ft.inner.ref_law.get();
    assert!(REF_LAWS_DECLARED.contains(&ref_law),
            "rung-73: the fuel reference is this rung's subject and it is DECLARED; got \
             {ref_law:?}. 'applied' is reading B (the plant); 'sched' is rung 72.");
    assert!(!(ref_law == REF_LAW_APPLIED && ft.inner.share_law.get() == "sum"),
            "rung-73: an APPLIED reference on top of the SUM composition swaps TWO declared laws \
             at once. `max(gf,gr) == g_own` never holds under `sum`, so the hook never takes its \
             identity branch, BOTH fuel rows gain a cross term and the block form goes -- a \
             fourth plant, whose result could be attributed to neither law. That is rung 63's \
             lesson in its plainest form: change one law at a time.");
    (crate::shared_actuator::R72_FUEL.integrate_fuel)(
        ft, flight, fuel_schedule, nu0, s_end, ds, lim)
}

/// RUNG 73's `_shared_rig` — **rung 72's rig with the reference carried onto the new machine, and
/// a MEASURED NO-OP.**
///
/// Python's docstring calls this *the other half of the fix* beside `at_lever`. Driven both ways
/// on the same receiver under both laws (probe L2), it is not: rung 72's body reaches its sibling
/// through `self.at_lever(…)`, which at rung 73 is [`r73_at_lever`], which has already copied the
/// law. **With the override and with it removed, `_ref_law` agrees.**
///
/// Ported unchanged regardless — a duplication the source makes is not the port's to remove, and
/// the belt-and-braces set is what keeps the carrying true if a later rung's `at_lever` stops
/// doing it. **Pre-registered for step 5: this swap has NO value break**, so a discriminator for
/// it does not exist and none should be hunted.
fn r73_shared_rig(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let (m, surge, lag) = (crate::shared_actuator::R72_TRIPLE.shared_rig)(core, arm);
    m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
    (m, surge, lag)
}

// ---------------------------------------------------------------------------------------------
// THE SEVENTH POINTER — the ONE cell this slice ADDS, and step 2's first body
// ---------------------------------------------------------------------------------------------

/// RUNG 73's `_quad_gains_at` — **THE FOURTEEN CENTRAL DIFFERENCES: rung 72's twelve, plus `F_f`
/// and `R_r`.**
///
/// Rung 72 never took those two, because its [`jac4`] could write `-1/tau_i` on the diagonal BY
/// CONSTRUCTION — correct there, since neither law reads its own state. **Reading B does**, so the
/// diagonal has to be MEASURED, and that is also what keeps this rung's headline off rung 72
/// § 1.2's list: a pole at the origin reported from a diagonal the instrument itself wrote would
/// be the FOURTH instance of the shipped instrument agreeing with itself (rung 67 gate 9, rung 71
/// § 1.4's `c1`, rung 72 § 4's matched clocks). Here `F_f` is a difference through the SHIPPED
/// closure, and it comes back EXACTLY 1 when the leg is masked and EXACTLY 0 when it holds.
///
/// # THE REFERENCE IS REACHED THROUGH THE TABLE, NEVER BY CALLING [`r73_reference`]
///
/// Python spells it `self._reference(...)`, and the whole mechanism the five readers depend on is
/// [`RefScope`](crate::reference_split::RefScope) flipping `ref_law` under this body: driven at
/// `"sched"` the reference is the identity, `F` stops depending on `gf`, and the twelve shared
/// gains come back rung 72's — which is what [`applied_gains`] differences against. A direct call
/// to [`r73_reference`] would still honour the law (it reads the same field), but rungs 74/80/81
/// inherit THIS body with THEIR tables, and a hard-wired callee is the shape
/// [`at_lever`](crate::bleed_transient::LeverHooks::at_lever)'s trap has taken twelve times.
///
/// # THE EVALUATION ORDER IS PYTHON'S AND IT IS NOT RUNG 72's
///
/// Rung 72 evaluates 24 arms in the order `F±r F±q F±v · R±f R±q R±v · C · V`; this rung evaluates
/// **28**, `F±f` first and `R±r` in the middle, because both legs gained a self-difference. The
/// order is load-bearing for the same reason it is at rung 72 — every arm runs before any regime
/// is inspected, so a short circuit would change how many closure calls the plant sees. The four
/// EXTRA arms are the only route by which this body's `interior` could disagree with rung 72's on
/// a point where every shared gain agrees, and **probe M measured that route DEAD on the shipped
/// grid: 0 disagreements over 101 points, at both `inc` arms.**
///
/// **COUNT ARMS, NOT LAW CALLS.** Probe M's raw per-call totals are `29/25` on the `inc = False`
/// arm and `28/24` on the `inc = True` one, which reads like the difference moving. It does not:
/// probe N splits the counter per law and the ARM LIST is `F 8 / R 8 / C 6 / V 6` against rung
/// 72's `6/6/6/6` at BOTH arms. The seventh `V` is not an arm — probe O attributes it to
/// [`manifold_v`](crate::three_loop::TripleHooks::manifold_v), resolved to RUNG 69's override,
/// whose first line falls through to the parent when `stator_inc` is `None` (one `V` call) and
/// otherwise runs an Illinois root on `phi_lp - phi_lim` that never touches `V` at all: 70 of 70
/// evaluations disarmed against 0 of 31 armed. It is rung 69's branch, present under BOTH
/// bodies, so it cancels from the difference.
///
/// [`jac4`]: crate::shared_actuator::jac4
#[allow(clippy::too_many_arguments)]
fn r73_quad_gains_at(
    core: &ScheduledStatorCore, flight: &FlightCondition, p: &FuelPoint,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>, tt4_max: f64,
    dg: f64, dq: f64, dv: f64, manifold: bool, switch_guard: f64,
) -> Result<QuadGains, Abort> {
    let (a, h, mf_sched) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let (gf, gr, q, v_live) = match p.extra {
        PointExtra::Shared { g_fuel, g_gov, b, v, .. }
        // SLICE AF (1 of 31): rung 74 records the same four. The two clips are UNFLOORED
        // projections `mf_sched - w` there, so this reader can now be handed a NEGATIVE
        // `g_fuel`/`g_gov` -- which is the number Python's bare dict index returns, so
        // admitting is the faithful arm and refusing would be stricter than the source.
        | PointExtra::Demand { g_fuel, g_gov, b, v, .. } => (g_fuel, g_gov, b, v),
        _ => panic!("rung-73's gains need a SIX-state trajectory: the point carries no \
                     `g_fuel`/`g_gov` pair, so there is no authority to difference across."),
    };
    let laws = quad_laws(core, flight, a, h, mf_sched, accel, surge, tt4_max);
    let v = if manifold {
        let vlaw = |g_: f64, q_: f64| (laws.v)(g_, 0.0, q_);
        core.manifold_v(flight, a, h, mf_sched, applied_clip(core, gf, gr), q, &vlaw)?
    } else {
        v_live
    };
    if core.fuel.inner.share_law.get() == "max" && (gf - gr).abs() <= switch_guard * dg {
        return Ok(QuadGains::dropped(p.s, v, vec!["switch"], true));
    }

    // `self._reference(...)`, THROUGH THE TABLE — see the header. `&core.fuel.inner` is the
    // receiver the cell's signature names, and it is the same core `RefScope` writes `ref_law` on.
    let refr = |raw: f64, g_own: f64, gf_: f64, gr_: f64| {
        (core.fuel.inner.triple_hooks.reference)(&core.fuel.inner, raw, g_own, gf_, gr_)
    };
    // Python's `F(gf_, gr_, q_, v_)`: rung 52's leg through `F0`, then the reference with the
    // FUEL leg's own clip as `g_own`. `F0` ignores `gf_`, so the ONLY route by which `F` depends
    // on `gf_` is the reference — which is exactly what `F_f` measures.
    let f_law = |gf_: f64, gr_: f64, q_: f64, v_: f64| -> Result<(f64, LegRegime), Abort> {
        let (raw, reg) = (laws.f)(gr_, q_, v_)?;
        Ok((refr(raw, gf_, gf_, gr_), reg))
    };
    // and `R(...)`: rung 47's clip through `R0`, then the reference with the GOVERNOR's clip as
    // `g_own`. Note `R0` takes `gf_` and ignores it — rung 72's `R_f == 0` — so `R_r` is the
    // mirror measurement.
    let r_law = |gf_: f64, gr_: f64, q_: f64, v_: f64| -> Result<(f64, LegRegime), Abort> {
        let (raw, reg) = (laws.r)(gf_, q_, v_)?;
        Ok((refr(raw, gr_, gf_, gr_), reg))
    };

    // PYTHON'S OWN ORDER, all 28 arms evaluated before any regime is read.
    let ev: Vec<(&'static str, f64, bool)> = vec![
        leg4("F+f", f_law(gf + dg, gr, q, v)?),
        leg4("F-f", f_law(gf - dg, gr, q, v)?),
        leg4("F+r", f_law(gf, gr + dg, q, v)?),
        leg4("F-r", f_law(gf, gr - dg, q, v)?),
        leg4("F+q", f_law(gf, gr, q + dq, v)?),
        leg4("F-q", f_law(gf, gr, q - dq, v)?),
        leg4("F+v", f_law(gf, gr, q, v + dv)?),
        leg4("F-v", f_law(gf, gr, q, v - dv)?),
        leg4("R+f", r_law(gf + dg, gr, q, v)?),
        leg4("R-f", r_law(gf - dg, gr, q, v)?),
        leg4("R+r", r_law(gf, gr + dg, q, v)?),
        leg4("R-r", r_law(gf, gr - dg, q, v)?),
        leg4("R+q", r_law(gf, gr, q + dq, v)?),
        leg4("R-q", r_law(gf, gr, q - dq, v)?),
        leg4("R+v", r_law(gf, gr, q, v + dv)?),
        leg4("R-v", r_law(gf, gr, q, v - dv)?),
        reg4("C+f", (laws.c)(gf + dg, gr, v)?),
        reg4("C-f", (laws.c)(gf - dg, gr, v)?),
        reg4("C+r", (laws.c)(gf, gr + dg, v)?),
        reg4("C-r", (laws.c)(gf, gr - dg, v)?),
        reg4("C+v", (laws.c)(gf, gr, v + dv)?),
        reg4("C-v", (laws.c)(gf, gr, v - dv)?),
        reg4("V+f", (laws.v)(gf + dg, gr, q)?),
        reg4("V-f", (laws.v)(gf - dg, gr, q)?),
        reg4("V+r", (laws.v)(gf, gr + dg, q)?),
        reg4("V-r", (laws.v)(gf, gr - dg, q)?),
        reg4("V+q", (laws.v)(gf, gr, q + dq)?),
        reg4("V-q", (laws.v)(gf, gr, q - dq)?),
    ];
    let off: Vec<&'static str> = ev.iter().filter(|(_, _, r)| !r).map(|(k, _, _)| *k).collect();
    if !off.is_empty() {
        return Ok(QuadGains::dropped(p.s, v, off, false));
    }
    let at = |k: &str| ev.iter().find(|(n, _, _)| *n == k).expect("the 28 keys above").1;
    let d = |kp: &str, km: &str, h2: f64| (at(kp) - at(km)) / (2.0 * h2);
    let (f_f, f_r) = (d("F+f", "F-f", dg), d("F+r", "F-r", dg));
    let (f_q, f_v) = (d("F+q", "F-q", dq), d("F+v", "F-v", dv));
    let (r_f, r_r) = (d("R+f", "R-f", dg), d("R+r", "R-r", dg));
    let (r_q, r_v) = (d("R+q", "R-q", dq), d("R+v", "R-v", dv));
    let (c_f, c_r, c_v) = (d("C+f", "C-f", dg), d("C+r", "C-r", dg), d("C+v", "C-v", dv));
    let (v_f, v_r, v_q) = (d("V+f", "V-f", dg), d("V+r", "V-r", dg), d("V+q", "V-q", dq));
    let auth = authority(gf, gr);
    let masked = match auth {
        Authority::Gov => Some(Authority::Fuel),
        Authority::Fuel => Some(Authority::Gov),
        _ => None,
    };
    let mask_leak = match masked {
        Some(Authority::Fuel) => Some(c_f.abs().max(v_f.abs())),
        Some(Authority::Gov) => Some(c_r.abs().max(v_r.abs())),
        _ => None,
    };
    // § 1's BRANCH INDICATOR, MEASURED: the masked leg's own self-gain, its cross-gain onto the
    // AUTHORITATIVE axis, and the holding leg's self-gain. Under reading B these are exactly
    // 1, -1 and 0; under rung 72 all three are 0 — and rung 72's body returns them ABSENT.
    let (self_masked, cross_masked, self_live) = match masked {
        Some(Authority::Fuel) => (Some(f_f), Some(f_r), Some(r_r)),
        Some(Authority::Gov) => (Some(r_r), Some(r_f), Some(f_f)),
        _ => (None, None, None),
    };
    Ok(QuadGains {
        interior: true,
        off_regime: Vec::new(),
        near_switch: false,
        s: p.s,
        v_base: v,
        authority: Some(auth),
        f_f,
        r_r,
        f_r,
        f_q,
        f_v,
        r_f,
        r_q,
        r_v,
        c_f,
        c_r,
        c_v,
        v_f,
        v_r,
        v_q,
        pair_fr: f_r * r_f,
        pair_rc: r_q * c_r,
        pair_cv: c_v * v_q,
        pair_rv: r_v * v_r,
        masked,
        mask_leak,
        self_masked,
        cross_masked,
        self_live,
    })
}

// ---------------------------------------------------------------------------------------------
// THE FIVE READERS — §§ 0, 1, 2, 3, 4
// ---------------------------------------------------------------------------------------------

/// Python's `round(v, 12)` — [`round10`](crate::full_split::round10) at this rung's own width.
///
/// Formatted and re-parsed rather than scaled, for `round10`'s stated reason: `x * 1e12` rounds
/// twice. Rust's `{:.12}` and CPython's `round` are both correctly-rounded decimal conversions
/// with ties to even, which is why the two agree bit for bit rather than nearly.
pub fn round12(x: f64) -> f64 {
    format!("{x:.12}").parse::<f64>().expect("a formatted finite double parses back")
}

/// **PYTHON'S `sorted({…})` OVER FLOATS, AND THE SET IS NOT A `BTreeSet`.**
///
/// A `set` deduplicates by `==`, so `0.0` and `-0.0` collapse to ONE member and the survivor is
/// **whichever was inserted first**; `sorted` then compares with `<`, which cannot separate them
/// either. So a sign that reaches this function's output is a fact about INSERTION ORDER, and the
/// only reader here that can meet the pair is [`AppliedGains::self_live`], whose whole claim is
/// that the holding leg's self-gain is exactly zero.
///
/// Reproduced literally: dedup by `==` keeping the FIRST, then a stable sort. Anything that sorted
/// the values first, or that keyed on `to_bits`, would report `-0.0` and `0.0` as two members —
/// a set Python cannot produce.
fn py_float_set(vals: impl Iterator<Item = f64>) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    for v in vals {
        if !out.iter().any(|x| *x == v) {
            out.push(v);
        }
    }
    out.sort_by(|a, b| a.partial_cmp(b).expect("no NaN reaches a reported float set"));
    out
}

/// The same, over the integer counts §§ 2 and 3 report as `sorted({…})`.
fn py_int_set(vals: impl Iterator<Item = i64>) -> Vec<i64> {
    let mut out: Vec<i64> = Vec::new();
    for v in vals {
        if !out.contains(&v) {
            out.push(v);
        }
    }
    out.sort_unstable();
    out
}

/// The six-state fields §§ 0–3 read off a point, refused on any other march.
fn shared_of(p: &FuelPoint) -> (f64, f64, f64, f64, usize, f64) {
    match p.extra {
        PointExtra::Shared { g_fuel, g_gov, required_fuel, required_gov, ic_iters, ic_res, .. }
        // SLICE AF (2 of 31): rung 74's march records all six, and runs the SAME four-way
        // joint sweep -- `ic_iters` is 2 on the `demand` arm against 1 on `clip`, measured.
        | PointExtra::Demand { g_fuel, g_gov, required_fuel, required_gov, ic_iters, ic_res, .. } =>
            (g_fuel, g_gov, required_fuel, required_gov, ic_iters, ic_res),
        _ => panic!("rung-73's readers march the SHARED-actuator rig, so every point carries the \
                     two clips and the two requirements. This one does not, which means the \
                     trajectory came from a different integrator."),
    }
}

/// Python's `traj[i]["authority"]` — a **bare index**, so a point without the key raises.
///
/// § 0 always marches the full rung-73 rig, so every point has one; the refusal STATES that
/// invariant rather than skipping, which would report a smaller census and no error at all.
/// [`authority_law`](crate::shared_actuator::authority_law)'s own note, one rung on.
fn auth_at(p: &FuelPoint) -> Authority {
    crate::shared_actuator::authority_of(p).expect(
        "rung-73: a point on this march carries no `authority` label. Python indexes the key \
         directly and raises here; answering `Dormant` would report a hand-over that never \
         happened.")
}

// --- § 0: THE HAND-OVER MOVES, AND THE MASKED LEG WINDS DOWN ----------------------------------

/// One law's reading at one `(inc, taus)` arm of [`handover_law`].
#[derive(Clone, Debug, PartialEq)]
pub struct HandoverRead {
    pub n: usize,
    /// Every `s` at which authority changed hands between two NON-dormant labels.
    pub handovers: Vec<f64>,
    /// `gov -> fuel`, which § 0 predicts never happens.
    pub hands_back: Vec<f64>,
    pub first_gov: Option<f64>,
    /// The largest clip the MASKED leg ever wound up to — the windup check, and § 0's feasibility
    /// gate. `None` when no point on the arm had a live authority at all.
    pub max_masked: Option<f64>,
    pub final_g_fuel: f64,
    pub final_g_gov: f64,
    pub max_tt4: f64,
    pub min_phi: f64,
    pub ic_iters: usize,
    pub ic_res: f64,
}

/// One `(inc, taus)` arm of [`handover_law`] — both laws, and the four differences.
#[derive(Clone, Debug, PartialEq)]
pub struct HandoverArm {
    pub inc: bool,
    pub taus: (f64, f64, f64, f64),
    pub sched: HandoverRead,
    pub applied: HandoverRead,
    /// Did the APPLIED reference take the actuator LATER? § 0's sign.
    pub later: bool,
    pub delay: Option<f64>,
    pub d_tt4: f64,
    pub d_phi: f64,
}

/// [`handover_law`]'s whole reading.
#[derive(Clone, Debug, PartialEq)]
pub struct HandoverLaw {
    pub arms: Vec<HandoverArm>,
    pub clocks: Vec<(f64, f64, f64, f64)>,
    pub ds: f64,
    pub always_later: bool,
    pub never_back: bool,
    pub one_handover: bool,
    /// **VACUOUSLY TRUE ON THE `sched` HALF, AND PORTED THAT WAY.** Python compares each law's `n`
    /// against `a["laws"]["sched"]["n"]` — including the `sched` arm itself, which is comparing a
    /// number with itself. The claim it can actually carry is *the applied march runs to the same
    /// length as the scheduled one*; the spelling is reproduced rather than repaired, because a
    /// port that tightened it would no longer be measuring the shipped reader.
    pub full_march: bool,
    pub worst_d_tt4: f64,
    pub worst_d_phi: f64,
    /// **A `min`, DESPITE THE NAME** — Python's `min(a["delay"] for a in out)`. The worst case of a
    /// quantity predicted POSITIVE is its smallest value, so the name is right and the reduction
    /// is the one that looks wrong.
    pub worst_delay: f64,
}

/// RUNG 73's `handover_law` — **§ 0 MEASURED: the hand-over is LATE under the applied reference,
/// on every arm — and the masked leg winds DOWN, not up.**
///
/// The sign is derivable and it is this rung's first correction of rung 72. A masked governor
/// referenced to the SCHEDULE races toward `req_sched`, the clip the SCHEDULE would need — so it
/// is given credit for a cut the fuel leg has already made. Referenced to the APPLIED fuel it
/// integrates `req_sched - gf`, the cut still OWED. **The physically-correct governor is therefore
/// the SLOWER one**: it takes the actuator later and the redline is approached with less margin.
///
/// THE WINDUP CHECK IS REPORTED HERE, AND IT WAS THE FEASIBILITY GATE. A masked integrator with
/// only a floor under it is textbook min-select windup; had `g_masked` run away, the hand-over
/// would slam a wound-up clip onto the actuator and starve the engine — which is how rung 72 § 4's
/// SUM law died, at 84 points of 341. It does not: masked means `gr > gf ~ req_f`, so the
/// integrand is negative and the leg winds DOWN.
#[allow(clippy::too_many_arguments)]
pub fn handover_law(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, clocks: &[(f64, f64, f64, f64)], r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> HandoverLaw {
    let mut out: Vec<HandoverArm> = Vec::new();
    for inc in [false, true] {
        for taus in clocks.iter().copied() {
            let mut reads: Vec<HandoverRead> = Vec::new();
            for law in REF_LAWS_DECLARED {
                // `self._with_ref(law, self._shared_march, …)` — the rig, and every sibling it
                // builds, run under THIS law. `shared_rig` and `at_lever` carry it onto the
                // machine, which is what step 1's two copies exist for.
                let traj = {
                    let _rs = RefScope::set(&core.fuel.inner, Some(law));
                    crate::shared_actuator::shared_march(
                        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max,
                        inc).3
                };
                let mut hand: Vec<f64> = Vec::new();
                let mut back: Vec<f64> = Vec::new();
                for i in 1..traj.len() {
                    let (a, b) = (auth_at(&traj[i]), auth_at(&traj[i - 1]));
                    if a != b && a != Authority::Dormant && b != Authority::Dormant {
                        hand.push(traj[i].s);
                    }
                    if a == Authority::Fuel && b == Authority::Gov {
                        back.push(traj[i].s);
                    }
                }
                // THE MASKED leg's clip at every point where SOMETHING holds the actuator: the
                // governor's own `g_gov` where the FUEL leg holds, and `g_fuel` where the
                // governor does. Getting the two the wrong way round would report the leg that
                // is riding as if it were wound up.
                let masked: Vec<f64> = traj.iter()
                    .filter(|p| matches!(auth_at(p), Authority::Fuel | Authority::Gov))
                    .map(|p| {
                        let (g_fuel, g_gov, ..) = shared_of(p);
                        if auth_at(p) == Authority::Gov { g_fuel } else { g_gov }
                    })
                    .collect();
                let last = traj.last().expect("§ 0's march emits at least one point");
                let (lf, lg, ..) = shared_of(last);
                let (.., ic_iters, ic_res) = shared_of(&traj[0]);
                reads.push(HandoverRead {
                    n: traj.len(),
                    handovers: hand,
                    hands_back: back,
                    first_gov: traj.iter().find(|p| auth_at(p) == Authority::Gov).map(|p| p.s),
                    max_masked: opt_fold(masked.iter().copied(), f64::max),
                    final_g_fuel: lf,
                    final_g_gov: lg,
                    max_tt4: opt_fold(traj.iter().map(|p| p.tt4), f64::max)
                        .expect("§ 0's march emits at least one point"),
                    min_phi: opt_fold(traj.iter().map(|p| p.phi_lp), f64::min)
                        .expect("§ 0's march emits at least one point"),
                    ic_iters,
                    ic_res,
                });
            }
            let applied = reads.pop().expect("both declared laws ran");
            let sched = reads.pop().expect("both declared laws ran");
            let (s0, s1) = (sched.first_gov, applied.first_gov);
            out.push(HandoverArm {
                inc,
                taus,
                later: matches!((s0, s1), (Some(a), Some(b)) if b > a),
                delay: match (s0, s1) {
                    (Some(a), Some(b)) => Some(b - a),
                    _ => None,
                },
                d_tt4: applied.max_tt4 - sched.max_tt4,
                d_phi: applied.min_phi - sched.min_phi,
                sched,
                applied,
            });
        }
    }
    let arms = out;
    HandoverLaw {
        always_later: arms.iter().all(|a| a.later),
        never_back: arms.iter().all(|a| a.sched.hands_back.is_empty()
                                     && a.applied.hands_back.is_empty()),
        one_handover: arms.iter().all(|a| a.sched.handovers.len() <= 1
                                       && a.applied.handovers.len() <= 1),
        // See `HandoverLaw::full_march`: the `sched` term compares `n` with itself.
        full_march: arms.iter().all(|a| a.sched.n == a.sched.n && a.applied.n == a.sched.n),
        worst_d_tt4: opt_fold(arms.iter().map(|a| a.d_tt4), f64::max)
            .expect("§ 0 sweeps at least one arm"),
        worst_d_phi: opt_fold(arms.iter().map(|a| a.d_phi.abs()), f64::max)
            .expect("§ 0 sweeps at least one arm"),
        // Python's `min(a["delay"] …)` over a list that may hold `None` raises `TypeError`, so
        // the absence of a hand-over under EITHER law is a loud stop and not a skipped arm.
        worst_delay: opt_fold(
            arms.iter().map(|a| a.delay.expect(
                "rung-73 § 0: an arm where one law never hands over has `delay = None`, and \
                 Python's `min` over `None` raises rather than dropping it.")),
            f64::min).expect("§ 0 sweeps at least one arm"),
        arms,
        clocks: clocks.to_vec(),
        ds,
    }
}

// --- § 1: THE FOURTEEN GAINS, AND THE ENTRYWISE J-DELTA ---------------------------------------

/// One sampled point of [`applied_gains`].
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedGainRow {
    pub s: f64,
    pub authority: Option<Authority>,
    pub masked: Option<Authority>,
    pub gains: QuadGains,
    pub taus: (f64, f64, f64, f64),
    pub self_masked: Option<f64>,
    pub cross_masked: Option<f64>,
    pub self_live: Option<f64>,
    pub mask_leak: Option<f64>,
    /// The TWO entries of `J73 - J72` that move, each already multiplied by `tau_masked` — so the
    /// prediction *"both move by exactly `1/tau_masked`"* is read as a pure number.
    pub delta_moved: (f64, f64),
    /// The largest of the OTHER fourteen, unscaled.
    pub delta_rest: f64,
    pub det: f64,
}

/// [`applied_gains`]'s whole reading.
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedGains {
    pub inc: bool,
    pub taus: (f64, f64, f64, f64),
    pub ds: f64,
    pub rows: Vec<AppliedGainRow>,
    pub skipped_switch: usize,
    pub skipped_regime: usize,
    pub boundary: Vec<BoundaryCheck>,
    pub n_riding: usize,
    pub n_sampled: usize,
    pub by_authority_fuel: usize,
    pub by_authority_gov: usize,
    /// THE EXACT ONES — gated as `== 1.0` / `== -1.0` / `== 0.0`, never as `< tol`.
    pub self_masked: Vec<f64>,
    pub cross_masked: Vec<f64>,
    pub self_live: Vec<f64>,
    pub worst_mask_leak: Option<f64>,
    pub worst_delta_rest: Option<f64>,
    pub moved_scaled: Vec<f64>,
    /// and the LIVE gains, so "exactly zero everywhere" is not bought with a dead reader.
    pub min_live_gain: Option<f64>,
    pub det_range: Option<(f64, f64)>,
}

/// RUNG 73's `applied_gains` — **§ 1 MEASURED: the masked leg's self-gain is EXACTLY 1 and the
/// holding leg's EXACTLY 0**, and the masked COLUMN is still exactly zero.
///
/// ```text
/// self_masked  == +1.0   the masked leg reads its OWN state (rung 72: 0)
/// cross_masked == -1.0   and the AUTHORITATIVE one (rung 72: 0) -- § 11's `F_r != 0`
/// self_live    ==  0.0   the holding leg's applied reference IS the scheduled one
/// mask_leak    ==  0.0   and the masked leg STILL reaches the plant through nothing
/// ```
///
/// The last line is the headline: **the seam's premise holds and its conclusion does not.**
///
/// AND THE J-DELTA IS REPORTED ENTRYWISE, at the SAME base points under both references (rung 71's
/// device, rung 72 § 4's) — 14 of the 16 entries are EXACTLY `0.0`, and the two that move are BOTH
/// exactly `1/tau_masked`.
///
/// # `g72` IS THIS BODY UNDER `sched`, NOT RUNG 72's BODY
///
/// Python differences against `m._with_ref("sched", m._quad_gains_at, …)` — rung 73's own cell with
/// the law flipped, which is why [`r73_quad_gains_at`] reaches the reference through the table.
/// Under `sched` the reference is the identity, `F` stops depending on `gf`, and the twelve shared
/// gains come back rung 72's — **measured, not argued: probe M ran both bodies on the same
/// receiver at all 101 sampled points of both arms and the worst difference over those twelve is
/// EXACTLY `0.0`.** What does not come back is the arm COUNT (28 against 24, a difference of
/// exactly four at every point) — and that count **observes nothing here**: `interior` disagrees
/// on 0 of 101. The separation that does exist is DISCRETE and it is the five keys rung 72 never
/// writes (`F_f`, `R_r` and the three branch indicators), 505 of them over the same points.
///
/// So a port that called rung 72's own body for `g72` would pass every gate this reader can
/// state, and step 5's pointer-level gate is where that is caught — not here.
#[allow(clippy::too_many_arguments)]
pub fn applied_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64,
    every: usize,
) -> Result<AppliedGains, Abort> {
    let (m, surge, lag, traj) = crate::shared_actuator::shared_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let lag = lag.expect("§ 1's rig arms the fuel leg, so it carries the lag");
    let mut rows: Vec<AppliedGainRow> = Vec::new();
    let mut boundary: Vec<BoundaryCheck> = Vec::new();
    let (mut sk_switch, mut sk_regime) = (0usize, 0usize);
    let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
    for p in sampled.iter() {
        let gg = {
            let _rs = RefScope::set(&m.fuel.inner, Some(REF_LAW_APPLIED));
            (m.triple_hooks().quad_gains_at)(
                &m, flight, p, None, surge.as_ref(), tt4_max, 1e-7, 1e-5, 1e-4, true, 4.0)?
        };
        if !gg.interior {
            if gg.near_switch {
                sk_switch += 1;
            } else {
                sk_regime += 1;
            }
            continue;
        }
        let g72 = {
            let _rs = RefScope::set(&m.fuel.inner, Some(REF_LAWS_DECLARED[0]));
            (m.triple_hooks().quad_gains_at)(
                &m, flight, p, None, surge.as_ref(), tt4_max, 1e-7, 1e-5, 1e-4, true, 4.0)?
        };
        if !g72.interior {
            // Python has no `near_switch` branch here: the switch guard is law-independent, so a
            // second drop on the same point can only be a REGIME.
            sk_regime += 1;
            continue;
        }
        boundary.push(assert_fuel_boundary(&m, flight, p, tt4_max, surge.as_ref(), 1e-5, 1e-4)?);
        let (g_fuel, _, required_fuel, ..) = shared_of(p);
        let tt = (lag.tau(required_fuel, g_fuel), taus.1, taus.2, taus.3);
        let (j73, j72) = (jac4(&gg, tt), jac4(&g72, tt));
        let masked = gg.masked;
        let tau_m = if masked == Some(Authority::Fuel) { tt.0 } else { taus.1 };
        let moved: [(usize, usize); 2] = if masked == Some(Authority::Fuel) {
            [(0, 0), (0, 1)]
        } else {
            [(1, 1), (1, 0)]
        };
        let delta = |i: usize, j: usize| j73[i][j] - j72[i][j];
        let mut rest = f64::NAN;
        for i in 0..4 {
            for j in 0..4 {
                if !moved.contains(&(i, j)) {
                    rest = py_running_max(rest, delta(i, j).abs());
                }
            }
        }
        rows.push(AppliedGainRow {
            s: p.s,
            authority: gg.authority,
            masked,
            taus: tt,
            self_masked: gg.self_masked,
            cross_masked: gg.cross_masked,
            self_live: gg.self_live,
            mask_leak: gg.mask_leak,
            delta_moved: (delta(moved[0].0, moved[0].1) * tau_m,
                          delta(moved[1].0, moved[1].1) * tau_m),
            delta_rest: rest,
            det: charpoly4(&j73)[4],
            gains: gg,
        });
    }
    let live_min = |x: &AppliedGainRow| -> f64 {
        let g = &x.gains;
        opt_fold([g.f_q.abs(), g.f_v.abs(), g.r_q.abs(), g.r_v.abs()].into_iter(), f64::min)
            .expect("four gains")
    };
    // `sorted({x["self_masked"] …})` over a set that held a `None` raises in Python, so the three
    // branch indicators are read with an `expect` rather than filtered — a row whose authority is
    // neither leg would be a march this reader is not entitled to summarise.
    let ind = |sel: fn(&AppliedGainRow) -> Option<f64>| -> Vec<f64> {
        py_float_set(rows.iter().map(|x| sel(x).expect(
            "rung-73 § 1: a sampled row has no masked leg, so its branch indicator is `None` and \
             Python's `sorted` over the resulting set raises.")))
    };
    Ok(AppliedGains {
        inc,
        taus,
        ds,
        skipped_switch: sk_switch,
        skipped_regime: sk_regime,
        n_riding: pts.len(),
        n_sampled: sampled.len(),
        by_authority_fuel: rows.iter().filter(|x| x.authority == Some(Authority::Fuel)).count(),
        by_authority_gov: rows.iter().filter(|x| x.authority == Some(Authority::Gov)).count(),
        self_masked: ind(|x| x.self_masked),
        cross_masked: ind(|x| x.cross_masked),
        self_live: ind(|x| x.self_live),
        worst_mask_leak: opt_fold(rows.iter().filter_map(|x| x.mask_leak.map(f64::abs)), f64::max),
        worst_delta_rest: opt_fold(rows.iter().map(|x| x.delta_rest), f64::max),
        moved_scaled: py_float_set(rows.iter()
            .flat_map(|x| [round12(x.delta_moved.0), round12(x.delta_moved.1)])),
        min_live_gain: opt_fold(rows.iter().map(live_min), f64::min),
        det_range: match (opt_fold(rows.iter().map(|x| x.det), f64::min),
                          opt_fold(rows.iter().map(|x| x.det), f64::max)) {
            (Some(lo), Some(hi)) => Some((lo, hi)),
            _ => None,
        },
        boundary,
        rows,
    })
}

// --- § 2: THE FOUR CELLS — every zero count PLUS ONE, and a determinant that dies -------------

/// One authority cell of one [`applied_cells`] arm.
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedCellStat {
    pub n: usize,
    pub n_parent: usize,
    /// The DISTINCT zero counts seen in this cell — a `set`, sorted. § 2's law says it has exactly
    /// one member, and that member is rung 72's count PLUS ONE.
    pub zeros: Vec<usize>,
    pub gap: f64,
    /// The same comparison **with the trace coefficient left out** — see [`applied_cells`].
    pub gap_hi: f64,
    pub vgap: f64,
    /// `min |z| / rate` — the distance of the NEAREST root to the ORIGIN, which is rung 72's
    /// `pole` measured against a different point. Not the same quantity as
    /// [`CellStat::pole`](crate::shared_actuator::CellStat::pole), which measures the distance to
    /// `-1/tau_masked`.
    pub pole: f64,
    pub null: f64,
    pub lam_max: f64,
    /// `(min, max)` of `coef[4]` over the cell — Python collapses its list here.
    pub det: (f64, f64),
    pub s: (f64, f64),
    pub parent: &'static str,
}

/// One `(inc, taus)` arm of [`applied_cells`].
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedCellsArm {
    pub inc: bool,
    pub taus: (f64, f64, f64, f64),
    pub cells: Vec<(Authority, AppliedCellStat)>,
    pub skipped_switch: usize,
    pub skipped_regime: usize,
    pub skipped_parent: usize,
    pub n_riding: usize,
    pub n_sampled: usize,
}

/// The union of one `(inc, authority)` key across every arm — Python's `seen`.
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedSeenCell {
    pub parent: &'static str,
    pub zeros: Vec<usize>,
    pub gap: f64,
    pub gap_hi: f64,
    pub vgap: f64,
    pub pole: f64,
    pub null: f64,
    pub lam_max: f64,
    /// **THE FOLD RUNS OVER AN ALREADY-COLLAPSED 2-TUPLE.** Python writes
    /// `max(abs(x) for x in c["det"])` where `c["det"]` is by then `(min, max)`, not the list of
    /// per-point determinants. The two agree whenever the extreme `|det|` is at an end of the
    /// range — which it is, `abs` being monotone off zero — so this is a spelling and not a bug;
    /// it is written down because a reader who assumed the list would compute the same number by a
    /// different route and never learn that the list was gone.
    pub det: f64,
    pub n: usize,
    pub n_parent: usize,
}

/// [`applied_cells`]'s whole reading.
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedCells {
    pub arms: Vec<AppliedCellsArm>,
    pub clocks: Vec<(f64, f64, f64, f64)>,
    pub ds: f64,
    pub cells: Vec<((bool, Authority), AppliedSeenCell)>,
    pub law_holds: bool,
    /// rung 72's per-cell counts, EACH PLUS ONE.
    pub predicted: [((bool, Authority), usize); 4],
    pub rung72: [((bool, Authority), usize); 4],
    pub all_four_cells: bool,
    pub worst_parent_gap: f64,
    /// the INDEPENDENT half of the comparison — see [`applied_cells`]'s `gap_hi` note.
    pub worst_parent_gap_hi: f64,
    pub worst_v_gap: f64,
    pub worst_null: f64,
    pub worst_det: f64,
    pub worst_lam: f64,
    pub pole_at_origin: f64,
}

/// RUNG 73's `applied_cells` — **§ 2 MEASURED, AND IT IS THE RUNG: the plant is STILL rung
/// 68/69/70/71 plus a pole, and the pole is now at the ORIGIN.**
///
/// ```text
/// | stator watches | fuel leg holds          | governor holds        |
/// | phi            | RUNG 68 + a zero  (3)   | RUNG 70 + a zero (2)  |
/// | M_i            | RUNG 69 + a zero  (2)   | RUNG 71 + a zero (1)  |
/// ```
///
/// `zeros = n_live - m_live + n_masked`. Every one of rung 72's four counts gains exactly one, and
/// **rung 71's cell — the only full-rank plant in the family, `det J = +5.9e4` under rung 72 —
/// goes to zero.** A reference is not a gain, not a clock and not a loop.
///
/// THE TEST IS THE SAME POLYNOMIAL IDENTITY with the pole moved: `p4 = (lam + a) * p3` with
/// `a = 1/tau_m -> 0`, so [`parent_quartic`] is called with `tau_m = f64::INFINITY` and states it
/// exactly, reusing rung 72's instrument unchanged. **Coefficients, not roots** — and the argument
/// is STRONGER here than at rung 72, because the added root is exactly zero, so every cell now has
/// at least a DOUBLE zero root and a root match would resolve it only to `sqrt(eps)`.
///
/// THE ZERO EIGENVECTOR's DIRECTION IS THE GATED HALF (rung 72 § 1.2's discipline, and the reason
/// this rung does not gate its own pole): `A e_masked = 0` is a claim about the MEASURED masked
/// column, whereas the eigenvalue would be a claim about a diagonal — and here that diagonal is
/// measured too, so the pole is REPORTED and the null direction and the COUNT are gated.
///
/// # THE `gap_hi` COLUMN EXISTS BECAUSE `gap` AND `null` ARE NOT TWO MEASUREMENTS
///
/// The masked column's only non-zero entry is its own diagonal (`F_f - 1`, which is `~0` only up
/// to the cancellation in `gf + req - gf`), and `a3` IS minus the trace — so `j = 1` reproduces
/// `null` entry for entry. Quoting both as agreement would be this family's SIXTH
/// instrument-agrees-with-itself. `gap_hi` (`j = 2, 3, 4`) is where the two INDEPENDENT readers
/// actually meet.
///
/// A THIRD CLOCK ARM is carried and disclosed: the applied reference delays the hand-over, so rung
/// 72's coverage does not transfer — at matched clocks the incidence/governor cell is EMPTY. All
/// four entries are swept march coordinates and no physical constant enters.
///
/// [`parent_quartic`]: crate::shared_actuator::parent_quartic
#[allow(clippy::too_many_arguments)]
pub fn applied_cells(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, clocks: &[(f64, f64, f64, f64)], r: f64, s_settle: f64, ds: f64, v_max: f64,
    every: usize,
) -> Result<AppliedCells, Abort> {
    let mut arms: Vec<AppliedCellsArm> = Vec::new();
    for inc in [false, true] {
        for taus in clocks.iter().copied() {
            let (m, surge, lag, traj) = crate::shared_actuator::shared_march(
                core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc);
            let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
            let pts = riding4(&traj, b_max);
            let lag = lag.expect("§ 2's rig arms the fuel leg");
            let mut cells: Vec<(Authority, AppliedCellStat)> = Vec::new();
            let (mut sk_switch, mut sk_regime, mut sk_parent) = (0usize, 0usize, 0usize);
            let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
            for p in sampled.iter() {
                let gg = {
                    let _rs = RefScope::set(&m.fuel.inner, Some(REF_LAW_APPLIED));
                    (m.triple_hooks().quad_gains_at)(
                        &m, flight, p, None, surge.as_ref(), tt4_max, 1e-7, 1e-5, 1e-4, true, 4.0)?
                };
                if !gg.interior {
                    if gg.near_switch {
                        sk_switch += 1;
                    } else {
                        sk_regime += 1;
                    }
                    continue;
                }
                let auth = gg.authority.expect("an interior point carries a label");
                if auth != Authority::Fuel && auth != Authority::Gov {
                    continue;
                }
                let (g_fuel, _, required_fuel, ..) = shared_of(p);
                let tau_f = lag.tau(required_fuel, g_fuel);
                let tt = (tau_f, taus.1, taus.2, taus.3);
                let a_mat = jac4(&gg, tt);
                let coef = charpoly4(&a_mat);
                let roots = quartic_roots_c(&coef);
                let rate = 1.0 / tt.0 + 1.0 / tt.1 + 1.0 / tt.2 + 1.0 / tt.3;
                let nz = roots.iter().filter(|z| z.abs() < 1e-4 * rate).count();
                // `im = 0 if auth == "gov" else 1` — THE MASKED COLUMN's index, not the live
                // one: the governor holding means the FUEL leg (row/column 0) is masked.
                let im = if auth == Authority::Gov { 0 } else { 1 };
                let null_res = opt_fold((0..4).map(|i| a_mat[i][im].abs()), f64::max)
                    .expect("four rows") / rate;
                let (g3, t3) = if auth == Authority::Gov {
                    (crate::three_loop::triple_gains_at(&m, flight, p, None, None,
                                                        1e-7, 1e-5, 1e-4, true, 0.0, true)?,
                     (taus.1, taus.2, taus.3))
                } else {
                    let _g = crate::cross_split::GovScope::set(&m.fuel.inner, None);
                    (crate::three_loop::triple_gains_at(&m, flight, p, None, surge.as_ref(),
                                                        1e-7, 1e-5, 1e-4, true, 0.0, true)?,
                     (tau_f, taus.2, taus.3))
                };
                let (mut gap, mut gap_hi, mut vgap) = (None, None, None);
                if g3.interior {
                    // `float("inf")` — the added root is at the ORIGIN, so `a = 1/tau_m = 0`.
                    // `parent_quartic` never multiplies by `tau_m`, so the infinity reaches the
                    // arithmetic only as that exact zero and no NaN can enter the coefficients.
                    let pred = crate::shared_actuator::parent_quartic(
                        crate::reference_split::invariants(&g3, t3), f64::INFINITY);
                    gap = opt_fold(
                        (1..5usize).map(|j| (coef[j] - pred[j]).abs() / rate.powi(j as i32)),
                        f64::max);
                    gap_hi = opt_fold(
                        (2..5usize).map(|j| (coef[j] - pred[j]).abs() / rate.powi(j as i32)),
                        f64::max);
                    vgap = Some((g3.v_base - gg.v_base).abs());
                } else {
                    sk_parent += 1;
                }
                let lam_max = opt_fold(roots.iter().map(|z| z.abs()), f64::max)
                    .expect("a quartic has four roots") / rate;
                let pole = opt_fold(roots.iter().map(|z| z.abs()), f64::min)
                    .expect("a quartic has four roots") / rate;
                let slot = match cells.iter().position(|(k, _)| *k == auth) {
                    Some(i) => i,
                    None => {
                        cells.push((auth, AppliedCellStat {
                            n: 0,
                            n_parent: 0,
                            zeros: Vec::new(),
                            gap: 0.0,
                            gap_hi: 0.0,
                            vgap: 0.0,
                            pole: 0.0,
                            null: 0.0,
                            lam_max: 0.0,
                            det: (f64::NAN, f64::NAN),
                            s: (f64::NAN, f64::NAN),
                            parent: applied_parent_of(inc, auth),
                        }));
                        cells.len() - 1
                    }
                };
                let c = &mut cells[slot].1;
                c.n += 1;
                if !c.zeros.contains(&nz) {
                    c.zeros.push(nz);
                }
                // Python appends to a list and collapses after the loop; the running `(min, max)`
                // is the same collapse taken one point at a time, seeded with `NaN` so the first
                // point wins outright ([`py_running_max`]'s device, mirrored for `min`).
                c.det = (py_running_min(c.det.0, coef[4]), py_running_max(c.det.1, coef[4]));
                c.s = (py_running_min(c.s.0, p.s), py_running_max(c.s.1, p.s));
                c.null = c.null.max(null_res);
                c.lam_max = c.lam_max.max(lam_max);
                c.pole = c.pole.max(pole);
                if let (Some(g), Some(gh), Some(vg)) = (gap, gap_hi, vgap) {
                    c.n_parent += 1;
                    c.gap = c.gap.max(g);
                    c.gap_hi = c.gap_hi.max(gh);
                    c.vgap = c.vgap.max(vg);
                }
            }
            for (_, c) in cells.iter_mut() {
                c.zeros.sort_unstable();
            }
            arms.push(AppliedCellsArm {
                inc,
                taus,
                cells,
                skipped_switch: sk_switch,
                skipped_regime: sk_regime,
                skipped_parent: sk_parent,
                n_riding: pts.len(),
                n_sampled: sampled.len(),
            });
        }
    }
    let mut seen: Vec<((bool, Authority), AppliedSeenCell)> = Vec::new();
    for a in arms.iter() {
        for (auth, c) in a.cells.iter() {
            let k = (a.inc, *auth);
            let slot = match seen.iter().position(|(kk, _)| *kk == k) {
                Some(i) => i,
                None => {
                    seen.push((k, AppliedSeenCell {
                        parent: c.parent,
                        zeros: Vec::new(),
                        gap: 0.0,
                        gap_hi: 0.0,
                        vgap: 0.0,
                        pole: 0.0,
                        null: 0.0,
                        lam_max: 0.0,
                        det: 0.0,
                        n: 0,
                        n_parent: 0,
                    }));
                    seen.len() - 1
                }
            };
            let d = &mut seen[slot].1;
            for z in c.zeros.iter() {
                if !d.zeros.contains(z) {
                    d.zeros.push(*z);
                }
            }
            d.gap = d.gap.max(c.gap);
            d.gap_hi = d.gap_hi.max(c.gap_hi);
            d.vgap = d.vgap.max(c.vgap);
            d.pole = d.pole.max(c.pole);
            d.null = d.null.max(c.null);
            d.lam_max = d.lam_max.max(c.lam_max);
            // The 2-tuple, iterated — see `AppliedSeenCell::det`.
            d.det = d.det.max(c.det.0.abs().max(c.det.1.abs()));
            d.n += c.n;
            d.n_parent += c.n_parent;
        }
    }
    for (_, d) in seen.iter_mut() {
        d.zeros.sort_unstable();
    }
    Ok(AppliedCells {
        law_holds: seen.iter().all(|(_, d)| d.zeros.len() == 1),
        predicted: [((false, Authority::Fuel), 3), ((false, Authority::Gov), 2),
                    ((true, Authority::Fuel), 2), ((true, Authority::Gov), 1)],
        rung72: [((false, Authority::Fuel), 2), ((false, Authority::Gov), 1),
                 ((true, Authority::Fuel), 1), ((true, Authority::Gov), 0)],
        all_four_cells: seen.len() == 4,
        worst_parent_gap: seen.iter().map(|(_, d)| d.gap).fold(f64::NAN, py_running_max),
        worst_parent_gap_hi: seen.iter().map(|(_, d)| d.gap_hi).fold(f64::NAN, py_running_max),
        worst_v_gap: seen.iter().map(|(_, d)| d.vgap).fold(f64::NAN, py_running_max),
        worst_null: seen.iter().map(|(_, d)| d.null).fold(f64::NAN, py_running_max),
        worst_det: seen.iter().map(|(_, d)| d.det).fold(f64::NAN, py_running_max),
        worst_lam: seen.iter().map(|(_, d)| d.lam_max).fold(f64::NAN, py_running_max),
        pole_at_origin: seen.iter().map(|(_, d)| d.pole).fold(f64::NAN, py_running_max),
        cells: seen,
        arms,
        clocks: clocks.to_vec(),
        ds,
    })
}

/// Which parent rung a cell IS — § 2's table.
///
/// The SAME four pairs [`parent_of`](crate::shared_actuator::shared_cells) names at rung 72, and
/// deliberately a second definition rather than a widened import: the two tables are equal today
/// **because rung 73 adds a pole and not a loop**, which is the finding, and sharing one function
/// would make the port assert it by construction.
fn applied_parent_of(inc: bool, auth: Authority) -> &'static str {
    match (inc, auth) {
        (false, Authority::Fuel) => "rung 68",
        (false, Authority::Gov) => "rung 70",
        (true, Authority::Fuel) => "rung 69",
        (true, Authority::Gov) => "rung 71",
        _ => panic!("rung-73's § 2 has four cells, indexed by a LIVE authority; \
                     `Dormant`/`Tie` name no parent because no leg holds the actuator there."),
    }
}

/// [`py_running_max`]'s mirror — Python's `min` as a fold seeded with `NaN`.
fn py_running_min(acc: f64, x: f64) -> f64 {
    if acc.is_nan() || x < acc {
        x
    } else {
        acc
    }
}

// --- § 3: THE ISOLATION INSTRUMENT — reading C, which moves the OTHER half --------------------

/// One sampled point of [`ref_discriminator`].
#[derive(Clone, Debug, PartialEq)]
pub struct RefDiscRow {
    pub s: f64,
    pub authority: Option<Authority>,
    pub masked: Option<Authority>,
    pub taus: (f64, f64, f64, f64),
    pub tau_live: f64,
    /// Does a root sit AT THE ORIGIN? (B: yes. C and rung 72: no.)
    pub origin_b: f64,
    pub origin_c: f64,
    pub origin_72: f64,
    /// Does a root sit at `-1/tau_masked`? (C and rung 72: yes. B: not by law.)
    pub pole_b: f64,
    pub pole_c: f64,
    pub pole_72: f64,
    /// The LIVE leg's own diagonal: B leaves it at rung 72's, C moves it by `-1`.
    pub live_diag_b: f64,
    pub live_diag_c: f64,
    pub zeros_b: i64,
    pub zeros_c: i64,
    pub zeros_72: i64,
}

/// [`ref_discriminator`]'s whole reading.
#[derive(Clone, Debug, PartialEq)]
pub struct RefDiscriminator {
    pub inc: bool,
    pub taus: (f64, f64, f64, f64),
    pub ds: f64,
    pub rows: Vec<RefDiscRow>,
    pub n: usize,
    pub worst_origin_b: Option<f64>,
    pub best_origin_c: Option<f64>,
    pub best_origin_72: Option<f64>,
    pub worst_pole_c: Option<f64>,
    pub worst_pole_72: Option<f64>,
    pub best_pole_b: Option<f64>,
    pub live_diag_b: Vec<f64>,
    pub live_diag_c: Vec<f64>,
    /// `("B", "C", "72")`, in Python's key order.
    pub zeros: [(&'static str, Vec<i64>); 3],
    /// **DIFFERENCED PER POINT, NEVER POOLED** — see [`ref_discriminator`].
    pub dzeros_b: Vec<i64>,
    pub dzeros_c: Vec<i64>,
}

/// RUNG 73's `ref_discriminator` — **§ 3: reading C, read at reading B's own base points** — one
/// law swapped, nothing else (rung 71's device, rung 72 § 4's, third instance).
///
/// C is the LITERAL reading of rung 72 § 11 (`req = mf_app - cap`, no increment) and it is a
/// well-posed proportional law with 2x droop. It is not the plant, and it is **NOT MARCHED**: a leg
/// that lands at half its own required clip holds neither floor, so its trajectory would confound
/// the reference with the state — rung 72 § 4's reason, verbatim. So `gc` is built from the SAME
/// measured differences as `g72`, with four entries overwritten, and never from a second march.
///
/// **THE POINT OF CARRYING IT IS THAT IT MOVES THE OTHER HALF OF THE MATRIX.** Under C the masked
/// row is `(-1/tau_m, -1/tau_m, ., .)`: the diagonal is rung 72's, so THE POLE STAYS at
/// `-1/tau_masked` — while the AUTHORITATIVE leg picks up `-1` on its own diagonal, so `M3` is NO
/// LONGER the parent's block.
///
/// ```text
/// B: the pole MOVES to the origin, `M3` IS the parent's        (the plant)
/// C: the pole STAYS at -1/tau_m, `M3` is NOT the parent's      (the instrument)
/// ```
///
/// Two readings of one seam that agree on `F_r != 0` and disagree on everything it was supposed to
/// imply. That is what makes the headline a measurement rather than a choice of law — and it is
/// why C is carried instead of dismissed (rung 63's lesson).
///
/// # THE COUNTS ARE DIFFERENCED PER POINT
///
/// This reader spans BOTH authority cells, whose counts already differ by one under rung 72 alone,
/// so a pooled `min(B) > max(72)` compares the `phi` arm's fuel cell against its governor cell and
/// says nothing. Per point, B adds EXACTLY one zero everywhere and C never adds one — it REMOVES
/// one wherever the live leg's droop restores full rank.
#[allow(clippy::too_many_arguments)]
pub fn ref_discriminator(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64,
    every: usize,
) -> Result<RefDiscriminator, Abort> {
    let (m, surge, lag, traj) = crate::shared_actuator::shared_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let lag = lag.expect("§ 3's rig arms the fuel leg");
    let mut rows: Vec<RefDiscRow> = Vec::new();
    let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
    for p in sampled.iter() {
        let gb = {
            let _rs = RefScope::set(&m.fuel.inner, Some(REF_LAW_APPLIED));
            (m.triple_hooks().quad_gains_at)(
                &m, flight, p, None, surge.as_ref(), tt4_max, 1e-7, 1e-5, 1e-4, true, 4.0)?
        };
        if !gb.interior
            || !matches!(gb.authority, Some(Authority::Fuel) | Some(Authority::Gov)) {
            continue;
        }
        let g72 = {
            let _rs = RefScope::set(&m.fuel.inner, Some(REF_LAWS_DECLARED[0]));
            (m.triple_hooks().quad_gains_at)(
                &m, flight, p, None, surge.as_ref(), tt4_max, 1e-7, 1e-5, 1e-4, true, 4.0)?
        };
        if !g72.interior {
            continue;
        }
        let (g_fuel, _, required_fuel, ..) = shared_of(p);
        let tau_f = lag.tau(required_fuel, g_fuel);
        let tt = (tau_f, taus.1, taus.2, taus.3);
        let masked = gb.masked;
        let m_fuel = masked == Some(Authority::Fuel);
        // `live = "gov" if masked == "fuel" else "fuel"`.
        let tau_m = if m_fuel { tau_f } else { taus.1 };
        let tau_l = if m_fuel { taus.1 } else { tau_f };
        // READING C, built from the SAME measured differences (`req = req_sched - clip`): the
        // masked leg keeps rung 72's diagonal and gains the cross term, and the LIVE leg's own
        // diagonal picks up the `-1` that B's identity branch removes.
        let mut gc = g72.clone();
        gc.f_r = if m_fuel { gb.f_r } else { 0.0 };
        gc.r_f = if !m_fuel { gb.r_f } else { 0.0 };
        gc.f_f = if m_fuel { 0.0 } else { -1.0 };
        gc.r_r = if !m_fuel { 0.0 } else { -1.0 };
        let rate = 1.0 / tt.0 + 1.0 / tt.1 + 1.0 / tt.2 + 1.0 / tt.3;
        let rb = quartic_roots_c(&charpoly4(&jac4(&gb, tt)));
        let rc = quartic_roots_c(&charpoly4(&jac4(&gc, tt)));
        let r72 = quartic_roots_c(&charpoly4(&jac4(&g72, tt)));
        // `min(abs(z + 1.0/tau_m) …) * tau_m` — `z + float` is a PROMOTED complex add, so the
        // imaginary part is `z.im + 0.0` and not `z.im`; the spelling is rung 72's and kept.
        let pole_of = |rs: &[C64; 4]| opt_fold(
            rs.iter().map(|z| c_add(*z, c_real(1.0 / tau_m)).abs()), f64::min)
            .expect("a quartic has four roots") * tau_m;
        let origin_of = |rs: &[C64; 4]| opt_fold(rs.iter().map(|z| z.abs()), f64::min)
            .expect("a quartic has four roots") / rate;
        let zeros_of = |rs: &[C64; 4]| rs.iter().filter(|z| z.abs() < 1e-4 * rate).count() as i64;
        rows.push(RefDiscRow {
            s: p.s,
            authority: gb.authority,
            masked,
            taus: tt,
            tau_live: tau_l,
            origin_b: origin_of(&rb),
            origin_c: origin_of(&rc),
            origin_72: origin_of(&r72),
            pole_b: pole_of(&rb),
            pole_c: pole_of(&rc),
            pole_72: pole_of(&r72),
            // `live == "fuel"` exactly when the GOVERNOR is masked.
            live_diag_b: if m_fuel { gb.r_r } else { gb.f_f },
            live_diag_c: if m_fuel { gc.r_r } else { gc.f_f },
            zeros_b: zeros_of(&rb),
            zeros_c: zeros_of(&rc),
            zeros_72: zeros_of(&r72),
        });
    }
    Ok(RefDiscriminator {
        inc,
        taus,
        ds,
        n: rows.len(),
        worst_origin_b: opt_fold(rows.iter().map(|x| x.origin_b), f64::max),
        best_origin_c: opt_fold(rows.iter().map(|x| x.origin_c), f64::min),
        best_origin_72: opt_fold(rows.iter().map(|x| x.origin_72), f64::min),
        worst_pole_c: opt_fold(rows.iter().map(|x| x.pole_c), f64::max),
        worst_pole_72: opt_fold(rows.iter().map(|x| x.pole_72), f64::max),
        best_pole_b: opt_fold(rows.iter().map(|x| x.pole_b), f64::min),
        live_diag_b: py_float_set(rows.iter().map(|x| x.live_diag_b)),
        live_diag_c: py_float_set(rows.iter().map(|x| x.live_diag_c)),
        zeros: [("B", py_int_set(rows.iter().map(|x| x.zeros_b))),
                ("C", py_int_set(rows.iter().map(|x| x.zeros_c))),
                ("72", py_int_set(rows.iter().map(|x| x.zeros_72)))],
        dzeros_b: py_int_set(rows.iter().map(|x| x.zeros_b - x.zeros_72)),
        dzeros_c: py_int_set(rows.iter().map(|x| x.zeros_c - x.zeros_72)),
        rows,
    })
}

// --- § 4: THE LEDGER — what the SCHEDULED reference was quietly buying ------------------------

/// [`applied_bill`]'s whole reading — rung 72's ledger under both references, and the differences.
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedBill {
    pub inc: bool,
    pub taus: (f64, f64, f64, f64),
    pub ds: f64,
    pub sched: SharedBill,
    pub applied: SharedBill,
    /// THE PEAK `Tt4` DEBIT the fuel leg imposes, under each reference.
    pub debit_sched: f64,
    pub debit_applied: f64,
    pub debit_ratio: Option<f64>,
    /// and the `phi` credit, which should not move.
    pub phi_marginal_sched: f64,
    pub phi_marginal_applied: f64,
    pub phi_full_sched: f64,
    pub phi_full_applied: f64,
    pub kept_sched: Option<f64>,
    pub kept_applied: Option<f64>,
    pub handover_sched: Option<f64>,
    pub handover_applied: Option<f64>,
    /// the governor's own currency as an INTEGRAL, both references.
    pub tt4_integral_sched: f64,
    pub tt4_integral_applied: f64,
}

/// RUNG 73's `applied_bill` — **§ 4: rung 72's own 16-cell ledger, run under BOTH references and
/// differenced.**
///
/// The spectral finding says the reference reaches only the masked leg's own two entries, and a
/// masked leg is coupled to nothing. The ledger is where that stops being the whole story:
/// **authority is a function of `s`**, the reference moves the HAND-OVER, and the hand-over is when
/// the redline stops being defended by a leg that is not watching it.
///
/// THE PREDICTION UNDER TEST (anchor P6): rung 72 § 5 reports the fuel leg's marginal peak `Tt4`
/// debit as `+0.29 K` / `+1.86 K` and calls the `phi` credit the finding. Under the correct
/// reference the debit should be more than TEN TIMES larger on both arms, with the `phi` column
/// unmoved — because the fuel leg's own authority window is EARLY, where the reference is the
/// identity, while the governor's is LATE, where it is not.
///
/// # THIS IS THE FIRST READER THAT BUILDS ITS CELLS THROUGH `_shared_rig`
///
/// Every cell is built by [`shared_rig`](crate::three_loop::TripleHooks::shared_rig) (rung 63's
/// lesson: a cell may differ from another only by which loops are armed), and rung 73's override
/// carries `_ref_law` onto each — *"without which every cell here would march rung 72 while the
/// caller reported rung 73"*, in the shipped docstring's own words. **Step 1 measured that carry a
/// NO-OP** (probe L2: `at_lever` has already copied the law by the time rung 72's body returns) and
/// pre-registered it as having no value break. The two statements are not in conflict — the carry
/// is redundant, not inert — but this reader is the one that would expose it if the redundancy ever
/// failed, and it is the grid step 5's discriminator must be re-measured on rather than inheriting
/// step 1's fifteen-gate verdict.
///
/// # `super().shared_bill` IS A NON-DISPATCHED CALL, ON PURPOSE
///
/// Python names the PARENT's body explicitly, so this reader runs rung 72's ledger even on a rung-74
/// machine. [`shared_bill`](crate::shared_actuator::shared_bill) is therefore called directly and
/// not through a table — the one place in this file where that is right.
#[allow(clippy::too_many_arguments)]
pub fn applied_bill(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> AppliedBill {
    let one = |law: &'static str| {
        let _rs = RefScope::set(&core.fuel.inner, Some(law));
        crate::shared_actuator::shared_bill(
            core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, inc, r, s_settle, ds, v_max)
    };
    let s = one(REF_LAWS_DECLARED[0]);
    let a = one(REF_LAW_APPLIED);
    let kept_of = |b: &SharedBill| b.kept.iter().find(|(k, _)| *k == "F")
        .expect("the ledger's four legs are named F, G, V, S").1;
    AppliedBill {
        inc,
        taus,
        ds,
        debit_sched: s.tt4_full - s.tt4_no_fuel,
        debit_applied: a.tt4_full - a.tt4_no_fuel,
        debit_ratio: if s.tt4_full != s.tt4_no_fuel {
            Some((a.tt4_full - a.tt4_no_fuel) / (s.tt4_full - s.tt4_no_fuel))
        } else {
            None
        },
        phi_marginal_sched: s.fuel_marginal_phi,
        phi_marginal_applied: a.fuel_marginal_phi,
        phi_full_sched: s.phi_full,
        phi_full_applied: a.phi_full,
        kept_sched: kept_of(&s),
        kept_applied: kept_of(&a),
        handover_sched: s.handover,
        handover_applied: a.handover,
        tt4_integral_sched: s.fuel_marginal_tt4,
        tt4_integral_applied: a.fuel_marginal_tt4,
        sched: s,
        applied: a,
    }
}
