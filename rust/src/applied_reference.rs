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
//! `cross_split.rs`'s `CoordScope` repeats the reasoning from the mirror side. What was missing is
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
    AsymmetricLag, Floor, FuelLimiters, FuelPoint, FuelTransientCore, FuelTransientHooks,
};
use crate::map::ComponentMap;
use crate::shared_actuator::{applied_clip_core, SharedRigArm};
use crate::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use crate::three_loop::TripleHooks;
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
