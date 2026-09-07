//! RUNG 76 — **THE FUEL-DEPENDENT CAP.** `SensedCapTransient`, slice AG.
//!
//! Every cap in this family is a SET-POINT SOLVE — `_cap_gov` returns the `w` at which
//! `Tt4(w) = Tt4_max`, `_cap_fuel`'s phi branch the `w` at which `phi(w) = phi_lim` — so it is a
//! function of the STATE alone, `d(cap)/d(mf) = 0`, and rung 73's `{0, 1}` follows. **Rung 48's
//! `Wf/pt3` leg is the ONE whose law is not a solve.** Its own docstring states it as an
//! inequality ON THE FUEL, `Wf <= (1 + margin)·kappa_ss(n_H)·pt3`, and a real limiter evaluates
//! that right-hand side from the delivery pressure it SENSES — the `pt3` produced by the fuel
//! actually burning. `_sched_fuel` instead solves the implicit fixed point `w* = cap(w*)`. **That
//! is a modelling choice, not the schedule**, and this rung declares the other branch.
//!
//! # WHAT STEP 1 OF THIS SLICE ADDS — **FOUR RE-AIMED POINTERS AND NOT ONE NEW TABLE FIELD**
//!
//! | | Python | slot | table |
//! |---|---|---|---|
//! | swap | `_sensed_cap` | [`sensed_cap`](crate::three_loop::TripleHooks::sensed_cap) | [`R76_TRIPLE`] |
//! | swap | `_shared_rig` | [`shared_rig`](crate::three_loop::TripleHooks::shared_rig) | [`R76_TRIPLE`] |
//! | swap | `at_lever` | `LeverHooks::at_lever` | [`R76`] |
//! | swap | `integrate_fuel` | `FuelTransientHooks::integrate_fuel` | [`R76_FUEL`] |
//!
//! The `0 ADD` consequence — that a missed re-aim COMPILES, runs the rung-74 parent and returns
//! this slice's own reduce-arm answer, so every reduce gate in the crate goes on passing — is
//! written once, at [`anti_windup`](crate::anti_windup), and the function-pointer identity gates
//! that answer it cover both rungs in `tests/slice_ag_cells.rs`.
//!
//! # THE KNOB THAT ADDS NO CONSTANT AT ALL — the fifth declared law, and the first of those
//!
//! `margin` is rung 48's own already-imposed scalar and `kappa`'s shape is still read off the
//! plant's own equilibria. **Its domain is DECLARED, not conceded**: `_cap_law` reaches the ACCEL
//! branch of `_cap_fuel` and nothing else. The phi leg and the governor have no sensed form in any
//! rung — a floor on a STATE is not a formula for a FUEL.
//!
//! # WHAT THE ARM DECIDES AND WHAT IT DOES NOT — measured before the port, § 5.31 (i)
//!
//! Slice AF measured `sensed_cap` **unreachable from every rung-74 reader**, because not one of
//! its seven seats arms an `AccelSchedule`, and booked the closure here with the sentence *rung 76
//! replaces the body, and the accel arm is where it lands*. **That booking is right and it is not
//! sufficient.** Arming the arm dispatches this cell at **1 366 of 1 366** calls on both marched
//! arms; whether a value MOVES is decided by the `min`-select one level down. At `phi_lim = 0.80`
//! the sensed cap sits **2.32 % below** the solve at every one of those calls and the accel leg
//! wins **0** of them, so the trajectory is bit-identical at 341 of 341 points; at `0.76` the leg
//! wins all 1 366 and every trajectory point moves, `max|ΔTt4| = 1.0255e+01` — which independently
//! reproduces `docs/rung76-spec.md` § 1.3's shipped `1179.24 → 1168.98 K`.
//!
//! **So reader discrimination, plant reachability and scope entry are three different questions
//! here and at `phi_lim = 0.80` they have three different answers** — the reason this slice's
//! arming gate is three-sided and lands at step 7 rather than at step 1.

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::demand_coordinate::LAG_COORD_CLIP;
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelLimiters, FuelPoint, FuelTransientCore,
    FuelTransientHooks,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::shared_actuator::SharedRigArm;
use crate::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
    StatorTransientHooks,
};
use crate::three_loop::TripleHooks;
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{
    MarchedBleed, MarchedStator, TwoSpoolTransientCore, TwoSpoolTransientHooks,
};

// ---------------------------------------------------------------------------------------------
// THE DECLARED CONSTANTS — rung 76's ONE class attribute, and it adds no number
// ---------------------------------------------------------------------------------------------

/// Python's `_cap_law = "solve"` — **THE CLASS DEFAULT, AND IT IS THE REDUCE ARM.**
///
/// Rung 48's set-point solve, which is rungs 48–75 verbatim. It is also the value
/// [`TwoSpoolTransientCore`](crate::two_spool_transient::TwoSpoolTransientCore)'s constructor
/// already writes, so — like `_lag_coord` and
/// `_windup_law`, and unlike `_ref_law` — there is nothing for [`build_sensed_cap_cascade`] to
/// overwrite and no silent-wrong-plant failure to gate for.
pub const CAP_LAW_SOLVE: &str = "solve";

/// Python's `"sensed"` — rung 48's inequality AS WRITTEN, evaluated at the fuel actually burning.
///
/// `cap(w) = (1 + margin)·kappa(n_H(w))·pt3(w)` at `w = mf_app`, so the cap is a function of the
/// FUEL and not of the state alone — the first in the ladder that can read `mf_app`.
pub const CAP_LAW_SENSED: &str = "sensed";

/// The two laws [`r76_integrate_fuel`]'s first refusal admits, in Python's tuple order.
///
/// Named so the refusal and the gate that drives it read the same list —
/// [`WINDUP_LAWS_DECLARED`](crate::anti_windup::WINDUP_LAWS_DECLARED)'s reason, one knob over.
pub const CAP_LAWS_DECLARED: [&str; 2] = [CAP_LAW_SOLVE, CAP_LAW_SENSED];

// ---------------------------------------------------------------------------------------------
// THE CASCADE BUILDER
// ---------------------------------------------------------------------------------------------

/// Build a rung-76 object, so every sibling re-asserts the whole chain's guards.
///
/// `_ref_law` is set for rung 73's reason, inherited through the chain rather than copied; nothing
/// else is written, because all four of this class's other declared laws — `_share_law`,
/// `_lag_coord`, `_windup_law`, `_cap_law` — declare exactly the core's own defaults, and a set
/// that no gate could see is a line that looks like it is doing the `ref_law` job.
pub fn build_sensed_cap_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = crate::reference_split::build_split_family_cascade(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
        &R76_TWO, &R76_STATOR, &R76_FUEL, &R76, &R76_TRIPLE);
    if let ScheduledStatorTransient::Full(c) = &built {
        c.fuel.inner.ref_law.set(crate::applied_reference::REF_LAW_APPLIED);
    }
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and TWO of them carry something of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 76's lever table — ONE swap, `at_lever`, and the parent it must differ from is rung 75's.
///
/// The FIFTEENTH instance of the sibling-constructor trap, now with FIVE knobs to drop.
pub const R76: LeverHooks = LeverHooks {
    at_lever: r76_at_lever,
    ..crate::anti_windup::R75
};

/// RUNG 76's `TwoSpoolTransientHooks` — **ZERO cells swapped**, an alias.
pub const R76_TWO: TwoSpoolTransientHooks = crate::anti_windup::R75_TWO;

/// RUNG 76's fuel table — ONE swap, `integrate_fuel`: **THREE refusals, two of them nested inside
/// the first's answer.**
pub const R76_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r76_integrate_fuel,
    ..crate::anti_windup::R75_FUEL
};

/// RUNG 76's stator table — **ZERO cells swapped**, an alias.
pub const R76_STATOR: StatorTransientHooks = crate::anti_windup::R75_STATOR;

/// RUNG 76's third-loop table — **TWO of rung 75's eighteen cells re-aimed, and NOTHING added.**
///
/// Spelled out field by field for [`R75_TRIPLE`](crate::anti_windup::R75_TRIPLE)'s reason, which
/// is sharper at this rung than anywhere: `windup_tau` must stay pointed at RUNG 75's body here
/// while `sensed_cap` moves to this one, and with no width tripwire in the slice the exhaustive
/// literal is the only place a reader sees both facts at once.
pub const R76_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::anti_windup::R75_TRIPLE.stator_leg,
    lagged_stator: crate::anti_windup::R75_TRIPLE.lagged_stator,
    clamp_v: crate::anti_windup::R75_TRIPLE.clamp_v,
    check_v0: crate::anti_windup::R75_TRIPLE.check_v0,
    rk4_floor: crate::anti_windup::R75_TRIPLE.rk4_floor,
    solve_v: crate::anti_windup::R75_TRIPLE.solve_v,
    manifold_v: crate::anti_windup::R75_TRIPLE.manifold_v,
    triple_laws: crate::anti_windup::R75_TRIPLE.triple_laws,
    triple_rig: crate::anti_windup::R75_TRIPLE.triple_rig,
    with_ref: crate::anti_windup::R75_TRIPLE.with_ref,
    reference: crate::anti_windup::R75_TRIPLE.reference,
    quad_gains_at: crate::anti_windup::R75_TRIPLE.quad_gains_at,
    rk4_floor_shared: crate::anti_windup::R75_TRIPLE.rk4_floor_shared,
    cap_fuel: crate::anti_windup::R75_TRIPLE.cap_fuel,
    windup_tau: crate::anti_windup::R75_TRIPLE.windup_tau,
    with_coord: crate::anti_windup::R75_TRIPLE.with_coord,
    // THE TWO THIS RUNG RE-AIMS.
    sensed_cap: r76_sensed_cap,
    shared_rig: r76_shared_rig,
};

// ---------------------------------------------------------------------------------------------
// THE FOUR RE-AIMED BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 76's `_sensed_cap` — **RUNG 48's SCHEDULE AS WRITTEN, EVALUATED AT THE FUEL ACTUALLY
/// BURNING.**
///
/// `None` is rung 75 EXACTLY: the caller then takes the shipped set-point solve and no float in
/// this family moves. `Some` is the first cap in the ladder whose value depends on `mf_app`.
///
/// # THE BODY LANDS AT STEP 1 AND NOT AT STEP 4, AND THE RE-CUT IS RECORDED
///
/// § 5.31 (vi) lists this method at step 4 and *both refusal sets* at step 1 — jointly impossible,
/// because this cell's refusal IS one of rung 76's five and there is nowhere else to put it. The
/// alternative shape, a slot that answers the reduce arm and `unimplemented!()`s the other, is a
/// live panic sitting in a `const` table for three steps, which slice AF's own header rejects.
/// Step 4 keeps the rest of that sentence: `_cap_march`, `_with_cap`, `accel_for`, `_c_at`.
///
/// # THE REFUSAL IS ABOUT A THREADING BUG, WHICH IS WHY IT IS NOT A FALLBACK
///
/// `mf_app` is threaded from `_applied_demand` at every site that can have one, so a `None` here
/// is a wiring defect and **falling back to the solve would report rung 75 as this rung**. That is
/// the anchor's own second refutation, and it is unobservable in any reader — the two bodies agree
/// wherever the solve is what gets used.
///
/// # THE RECEIVER IS PRESENT AT RUNG 74 AND UNUSED THERE, WHICH IS WHY THE CELL HAS ONE
///
/// `_sensed_cap` is a `@staticmethod` at rung 74 and an INSTANCE method here — invisible at the
/// one call site, because `self._x(…)` binds a staticmethod without a receiver. The cell carries
/// the receiver the rung-74 body ignores and this body needs: it reads `_cap_law`,
/// `_instant_fuel` and `pi_b`. `Result` for the same reason — `_instant_fuel` can abort.
///
/// **`1e-9f64.max(mf_app)` IS PYTHON's `max(1e-9, mf_app)` ON EVERY INPUT INCLUDING NaN**, argument
/// 0 being a literal; the general rule and the census are at
/// [`demand_laws`](crate::demand_coordinate)'s C-law note, corrected in this same commit.
fn r76_sensed_cap(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, accel: &AccelSchedule,
    mf_app: Option<f64>,
) -> Result<Option<f64>, Abort> {
    if ft.inner.cap_law.get() != CAP_LAW_SENSED {
        return Ok(None);
    }
    let mf_app = mf_app.expect(
        "rung-76: a SENSED cap is a function of the fuel it is asked about, and this call site did \
         not supply one. `mf_app` is threaded from `_applied_demand` at every site that can have \
         it; a `None` here is a THREADING bug, and falling back to the solve would report rung 75 \
         as this rung (anchor s 3's second refutation).");
    let i = ft.try_instant_fuel(flight, a, h, 1e-9f64.max(mf_app))?;
    Ok(Some(accel.cap(i.base.close.n_hp, i.base.close.pt4 / ft.inner.inner.base.pi_b)))
}

/// RUNG 76's `at_lever` — **rung 75's sibling constructor returning a RUNG-76 machine THAT CARRIES
/// FIVE KNOBS.**
///
/// Fifteenth instance of the trap, and the fifth knob is the one this rung is about: hand back a
/// sibling at the class default and every reader measures `"solve"`, which IS rung 75 and passes
/// every reduce gate in the crate.
fn r76_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let m = match build_sensed_cap_cascade(
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
    m.fuel.inner.cap_law.set(core.fuel.inner.cap_law.get());
    m
}

/// RUNG 76's `_shared_rig` — **rung 75's rig with the CAP LAW carried too, a fifth knob on the
/// same leak.**
///
/// Predicted a NO-OP for the same structural reason its two predecessors were: rung 72's body
/// reaches its sibling through `self.at_lever(…)`, which on a rung-76 receiver is [`r76_at_lever`],
/// which has already copied all five. Ported unchanged regardless, and driven rather than asserted
/// in `tests/slice_ag_cells.rs` — a duplication the source makes is not the port's to remove.
///
/// **THE PREDICTION HELD AND THE SWEEP STILL KILLED ITS DELETION**, which is worth separating:
/// removing the line leaves the carried value untouched (the value gate passes), and what fails is
/// the POINTER gate, because the remaining body forwards straight to rung 75's and the linker
/// FOLDS the two into one address. So this line is redundant on the value and load-bearing on the
/// identity — see `the_slice_re_aims_eight_pointers_and_adds_none`'s note.
fn r76_shared_rig(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let (m, surge, lag) = (crate::anti_windup::R75_TRIPLE.shared_rig)(core, arm);
    m.fuel.inner.cap_law.set(core.fuel.inner.cap_law.get());
    (m, surge, lag)
}

/// RUNG 76's `integrate_fuel` — **THREE REFUSALS, AND TWO OF THEM ARE NESTED INSIDE THE FIRST's
/// ANSWER.**
///
/// Python asserts the declared law unconditionally, and then — **only on the `sensed` arm** —
/// refuses the `clip` coordinate and a missing schedule. The nesting is load-bearing in both
/// directions and both failures are silent:
///
/// * hoisting the inner two rejects a legal `solve` march on a `clip` machine with no accel leg,
///   which is rungs 48–75 and is exactly what the reduce arm has to keep marching;
/// * flattening them away lets `clip × sensed` through, and `clip` dispatches out of this ladder
///   before `_cap_fuel` is ever called — so the march would silently be rung 73 and be reported as
///   this rung. That is rung 75 § 0.1's refusal one knob over, and rung 75's own body spells the
///   mirror of it (see [`r75_integrate_fuel`](crate::anti_windup)'s side-effect call).
///
/// The schedule refusal is not defensive tidiness: `sensed` RE-READS rung 48's `Wf/pt3` law, so
/// there must BE one, and this rung's domain is declared rather than conceded — the phi leg and
/// the governor have no sensed form in any rung.
fn r76_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    let law = ft.inner.cap_law.get();
    assert!(
        CAP_LAWS_DECLARED.contains(&law),
        "rung-76: the CAP LAW is this rung's subject and it is DECLARED; got {law:?}. 'solve' is \
         rung 48's set-point solve (rungs 48-75); 'sensed' is the schedule as written, evaluated \
         at the applied fuel.");
    if law == CAP_LAW_SENSED {
        let coord = ft.inner.lag_coord.get();
        assert!(
            coord != LAG_COORD_CLIP,
            "rung-76: `sensed` is REFUSED in the CLIP coordinate. `clip` dispatches out of this \
             ladder before `_cap_fuel` is ever called, so the march would silently be rung 73 and \
             be reported as this rung -- rung 75 s 0.1's refusal, one knob over. Got _lag_coord = \
             {coord:?}.");
        assert!(
            lim.accel.is_some(),
            "rung-76: `sensed` re-reads rung 48's `Wf/pt3` schedule, so there must BE one. The phi \
             leg and the governor have no sensed form in any rung -- a floor on a STATE is not a \
             formula for a FUEL (spec s 0.1). Pass `accel=`.");
    }
    (crate::anti_windup::R75_FUEL.integrate_fuel)(
        ft, flight, fuel_schedule, nu0, s_end, ds, lim)
}

// ---------------------------------------------------------------------------------------------
// STEP 4 — THE CAP: the scope guard, the ACCEL-ARMED march, the schedule's OWN plant, and the
// one derivative this rung rests on
// ---------------------------------------------------------------------------------------------

/// Python's `accel_schedule(…, n = 13)` — **the DEFAULT this rung's [`accel_for`] relies on, and
/// the only site in the crate that does.**
///
/// The Rust `accel_schedule` takes `n` positionally because Rust has no defaults, so every other
/// caller in `rust/src` passes a value its own Python line spells out. [`accel_for`] is the one
/// caller whose Python line does NOT — `m.accel_schedule(flight, Tt4_lo, Tt4_hi, margin)` at
/// `engine.py:19301` — so the default is a fact about `engine.py:4694` that has to be carried here
/// by hand. Named rather than typed inline: a bare `13` at the call site is a number nobody can
/// check against a signature, and a wrong `n` is a silently different schedule on a rung whose
/// entire subject is WHICH schedule is being read.
pub const ACCEL_SCHEDULE_N: usize = 13;

/// Python's `_c_at(…, rel = 1e-6)` — the central-difference step, RELATIVE to `w`.
///
/// Carried for [`ACCEL_SCHEDULE_N`]'s reason: both of this rung's callers of [`c_at`] (`_cap_rows`
/// and `solve_gain`, step 5) take the default, so the value is invisible at either call site in
/// Python and would be invisible at neither in Rust.
pub const C_AT_REL: f64 = 1e-6;

/// RUNG 76's `_with_cap` — **a named CAP LAW for the length of one reader, restored in a
/// `finally`.** Rung 62's reason, TENTH reload.
///
/// # IT WRITES THE FIELD DIRECTLY, ON THE SAME EVIDENCE AS ITS PREDECESSOR
///
/// [`WindupScope`] took a direct write because `_with_windup` has exactly one definer over all 58
/// classes; `_with_cap` has exactly one too (`engine.py:19283`, and no other `def _with_cap`
/// anywhere), so there is no later body for a cell to reach and a slot would be dead —
/// [`ShareScope`](crate::shared_actuator::ShareScope)'s decision, third instance.
///
/// # IT RESTORES THE **PREVIOUS** VALUE, WHICH PUTS IT ON THE OTHER SIDE OF THE CRATE'S SPLIT
///
/// § 5.19 (iv) measured 72 reload guards over the nine STATE-kind fields: 68 restore to `None` and
/// 4 restore what they displaced. This is a LAW-kind field, not a state-kind one, and every law
/// guard in the family restores the previous value — Python's `prev = self._cap_law` … `finally:
/// self._cap_law = prev`. **The distinction bites at this rung's own call sites**: step 5's
/// `_cap_rows` nests `_with_windup(…, m._with_cap, …)`, so a `None`-restoring guard here would
/// leave the receiver carrying no cap law at all, and [`r76_integrate_fuel`]'s first refusal —
/// which admits exactly two strings — would fire on the NEXT reader rather than on this one.
///
/// **AND BOTH WRONG RESTORE POLICIES ARE NEARLY INVISIBLE, WHICH IS A HAZARD FOR STEP 6's GATES
/// AND NOT FOR THIS ONE.** Step 4's sweep measured them: restoring the CONSTANT `"solve"` instead
/// of `prev` moves **exactly ONE key of 1 422** — the one nested read whose outer scope was armed
/// `"sensed"` — because everywhere else the drive's machine already sits at `"solve"` and the
/// injected constant coincides with the value it displaced. Restoring NOTHING moves 19, and **not
/// one of them is a cap VALUE**: every caller sets the law immediately before reading it, so a
/// leaked law is always overwritten before anything consults it, and only the tag reads catch it.
/// So a step-6 gate that drives this guard end to end through `_cap_fuel` — the natural shape —
/// is blind to both defects; the field has to be READ WITHOUT BEING SET FIRST, on a machine whose
/// resting law is NOT the one the guard arms.
///
/// # THE CURRIED FORM IS NOT PORTED, AND THAT IS A SHAPE DIFFERENCE WORTH NAMING
///
/// Python's `_with_cap(law, fn, *a, **kw)` CALLS `fn` inside the `try`. The Rust guard is an RAII
/// value the caller holds, so the callee is spelled at the call site instead of passed to it. The
/// two are equivalent for every use in `engine.py` because no caller passes a `fn` that captures
/// the guard's own lifetime — checked at all five call sites (`engine.py:19351`, `19373`, `19375`,
/// `19591`, `19598`) — and the RAII form additionally survives an unwind, which Python's `finally`
/// also does. Same reason [`WindupScope`] is a guard and not a combinator.
///
/// [`WindupScope`]: crate::anti_windup::WindupScope
pub struct CapScope<'a> {
    core: &'a TwoSpoolTransientCore,
    prev: &'static str,
}

impl<'a> CapScope<'a> {
    /// Arm the named cap law for as long as the returned guard lives.
    pub fn set(core: &'a TwoSpoolTransientCore, law: &'static str) -> Self {
        let prev = core.cap_law.get();
        core.cap_law.set(law);
        CapScope { core, prev }
    }

    /// What this scope displaced — Python's `prev`, exposed so a gate can read the restore POLICY
    /// rather than only its effect.
    /// [`WindupScope::displaced`](crate::anti_windup::WindupScope::displaced)'s precedent, back to
    /// a single value.
    pub fn displaced(&self) -> &'static str {
        self.prev
    }
}

impl Drop for CapScope<'_> {
    fn drop(&mut self) {
        self.core.cap_law.set(self.prev);
    }
}

/// RUNG 76's `_cap_march` — **one rig, one march, under FIVE named knobs AND AN ARMED ACCEL LEG.**
///
/// # THE ARMED LEG IS THE WHOLE STRUCTURAL DIFFERENCE, AND DROPPING IT IS INVISIBLE TO EVERY
/// REDUCE GATE IN THE CRATE
///
/// [`windup_march`](crate::anti_windup::windup_march) builds its [`StatorLeg`] with `accel: None`
/// — no march in this family had ever carried a schedule, which the Python docstring states as
/// *the one thing this family's marches have never done*. Here it is `Some(accel)`, and that is
/// what makes `_cap_fuel`'s accel branch run at all. Drop it and, on the `solve` arm, the failure
/// is silent in the worst available way: [`r76_sensed_cap`] is never dispatched, and the
/// trajectory is [`windup_march`](crate::anti_windup::windup_march)'s exactly — which IS this
/// rung's reduce-arm answer, so every reduce comparison the crate owns goes on passing. That is
/// § 5.31 (ii)'s `0 ADD` blindness one level down, and no reduce can catch it; it needs a POSITIVE
/// count. The pre-flight supplies the number to assert against: on both marched arms the cell is
/// dispatched at **1 366 of 1 366** `_cap_fuel` calls, and under `sensed` the branch is taken at
/// 1 366 of 1 366.
///
/// **AND ON THE `sensed` ARM IT IS NOT SILENT AT ALL — MEASURED, AGAINST THE PARAGRAPH ABOVE,
/// WHICH FIRST CLAIMED IT WAS.** Step 4's sweep deleted the arm and the drive did not diff, it
/// PANICKED, on [`r76_integrate_fuel`]'s THIRD refusal (*there must BE one*) — which was ported at
/// step 1, three steps before any caller existed that could trip it. So the blindness has a
/// DOMAIN: it is the `solve` arm's, and the arm this rung is about is defended by a refusal
/// already in the file. That is step 3 § (a)'s lesson running the other way — there, a write
/// predicted inert was killed by a refusal born at THIS rung; here, a defect predicted silent is
/// killed by a refusal shipped THREE STEPS EARLY. An enumeration of what a refusal protects
/// expires at the step that adds its caller, exactly as an enumeration of a field's readers
/// expires at the rung that adds one.
///
/// # IT IS A COPY OF `_windup_march` PLUS TWO LINES, AND IT IS PORTED AS A COPY
///
/// The delegating spelling is not merely unfaithful here, it is WRONG, for the reason step 2 wrote
/// down one rung earlier: the parent RUNS THE MARCH before it returns, so a `cap_march` that called
/// [`windup_march`](crate::anti_windup::windup_march) and then set `cap_law` would set it on a
/// machine that had already marched. Every trajectory would be rung 75's, every reduce gate would
/// pass, and the cap would be reported by a reader that never saw it — the sibling-constructor trap
/// one level up, for the third time in this slice.
///
/// # `_ic_cap` IS STILL CARRIED FROM THE CALLER, AND `_cap_law` COMES FROM THE ARGUMENT
///
/// `self._ic_cap` — the RECEIVER's — because
/// [`contraction_law`](crate::anti_windup::contraction_law) raises the cap inside a `try/finally`
/// and every march it drives has to see the raised value. `cap_law` is the opposite: it comes from
/// the PARAMETER, declared at the call site rather than inherited from whatever `self` happens to
/// carry, which is the trap this family has now been through for five knobs in a row.
#[allow(clippy::too_many_arguments)]
pub fn cap_march(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), r: f64, s_settle: f64, ds: f64, v_max: f64, inc: bool,
    coord: &'static str, ref_law: &'static str, law: &'static str, tau_t: Option<f64>,
    cap_law: &'static str, accel: &AccelSchedule, nu0: Option<(f64, f64)>,
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
    // PLAIN ASSIGNMENTS, all six, exactly as Python spells them — see
    // [`coord_march`](crate::demand_coordinate::coord_march)'s note for why a dispatch here is the
    // defect slice AF step 6 had to repair at four sites.
    m.fuel.inner.lag_coord.set(coord);
    m.fuel.inner.ref_law.set(ref_law);
    m.fuel.inner.windup_law.set(law);
    m.fuel.inner.tau_t.set(tau_t);
    m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());
    m.fuel.inner.cap_law.set(cap_law);
    let leg = StatorLeg { accel: Some(accel), surge, tt4_max: Some(tt4_max) };
    let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
    let traj = m.stator_march_scoped(
        flight, &ramp, nu0, &leg,
        &MarchScope { tau_gov: Some(tau_gov), lag, ..MarchScope::DEFAULT }).0;
    (m, surge, lag, traj)
}

/// RUNG 76's `accel_for` — **rung 48's schedule built on THE RIG THAT WILL MARCH IT.**
///
/// # THE SCHEDULE IS A TABLE READ OFF A PLANT, SO THE PLANT IS THE ARGUMENT
///
/// `kappa_ss` is read off the machine's OWN equilibria, so a schedule built on `self` and marched
/// on [`cap_march`]'s rig would be a schedule for a different engine. Python's docstring calls this
/// the fourteenth instance of rungs 61–75's carried-knob trap *wearing its other face* — the thing
/// not carried is here a TABLE rather than a scalar, and the failure is quieter for it: a
/// mismatched schedule still produces a perfectly well-formed cap at every point of the march.
///
/// # THE RIG COMES THROUGH THE HOOK TABLE, AND AT THIS RUNG THAT IS A DISPATCH, NOT A HABIT
///
/// `self._shared_rig(…)` has **eight** definers (rungs 72…80) and rung 76 re-aims the cell
/// precisely so the rig carries `_cap_law` ([`r76_shared_rig`]). Reaching
/// `crate::anti_windup::R75_TRIPLE.shared_rig` directly would build the schedule on a machine at
/// the class default — and since a schedule is read off EQUILIBRIA, which no cap law touches, the
/// table would come back identical and the defect would surface only through whatever else that
/// machine were later used for. So it is called through `core.triple_hooks()` for the RULE rather
/// than for a measured difference, and this comment says which of the two it is — **measured**:
/// step 4's sweep replaced the dispatch with the direct rung-75 pointer and **0 of 1 422 keys
/// moved**. Two more of that sweep's survivors belong to this function and are recorded for the
/// same reason: `tau_rel = tau_f` instead of `3·tau_f`, and `tau_att` taken from the GOVERNOR's
/// clock, are both invisible here, because rung 52's lag is a MARCH parameter and a schedule is
/// read off EQUILIBRIA. The identical `tau_rel` mutation inside [`cap_march`] kills 688 keys.
///
/// # `n` IS PYTHON's DEFAULT AND THE ONLY ONE IN THE CRATE — see [`ACCEL_SCHEDULE_N`]
#[allow(clippy::too_many_arguments)]
pub fn accel_for(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, sm: f64,
    tt4_max: f64, taus: (f64, f64, f64, f64), v_max: f64, inc: bool, margin: f64,
) -> AccelSchedule {
    let (tau_f, _tau_gov, tau_q, tau_s) = taus;
    let m = (core.triple_hooks().shared_rig)(core, &SharedRigArm {
        sm,
        tau: tau_q,
        tau_s,
        v_max,
        tt4_max,
        tau_att: tau_f,
        tau_rel: 3.0 * tau_f,
        inc,
        ..Default::default()
    }).0;
    m.fuel.accel_schedule(flight, tt4_lo, tt4_hi, margin, ACCEL_SCHEDULE_N)
}

/// RUNG 76's `_c_at` — **`c = d(cap_sensed)/dw` AT A POINT, and it is MEASURED.**
///
/// The one number this whole rung rests on. **It is NOT implied by the shipped bracket working**:
/// a bracketing root-finder converges on a sign change whether or not `G = w − cap(w)` is monotone,
/// so `_sched_fuel` bracketing buys *a root exists*, never `G' > 0` (anchor § 0.3). That
/// distinction is rung 83's whole subject, arriving seven rungs early as a measurement.
///
/// # IT RE-SPELLS [`r76_sensed_cap`]'s FORMULA WITHOUT THE LAW CHECK, AND THAT IS DELIBERATE
///
/// The inner `cap` closure is `(1 + margin)·kappa(n_H(x))·pt3(x)` — the SENSED law — with no
/// `_cap_law` consultation, because this reader measures the sensed cap's slope whatever law is
/// currently armed, and every one of its callers is COMPARING the two laws. Routing it through the
/// cell would be a factoring the source refuses, and it would silently return `None` on the arm the
/// reader exists to characterise.
///
/// # BOTH STATE FIELDS ARE SET, WHICH IS THE `b_state`/`v_state` BOUNDARY'S THIRD CASE
///
/// [`MarchedStator`]'s own doc states the rule: a law that TRIALS an actuator must not see that
/// actuator's state and must see the other two, so the valve law sets `v_state` alone and the
/// stator law `b_state` alone — **and a law that trials NEITHER sets BOTH**. This trials fuel, so
/// it is that third case, and Python spells it as one tuple assignment. Both guards restore `None`
/// on drop, which is Python's `finally: self._b_state, self._v_state = None, None` exactly; note
/// that this is the OPPOSITE policy from [`CapScope`] three items up, and the two sit in one file
/// because the fields are of different KINDS.
///
/// # THE `max` IS EXPRESSION-FIRST, AND IT IS THE ONLY ONE IN THE PACKAGE
///
/// `dw = rel * max(w, 1e-9)`. Slice AG step 1 censused all 268 n-ary `max`/`min` calls in
/// `engine.py`: 103 put a literal first — every one faithful under `lit.max(x)`, because Python
/// seeds its fold at argument 0 and replaces only on a strict comparison, so a NaN in argument 1 is
/// discarded exactly as Rust's `f64::max` discards it. **This call is the single expression-first
/// `1e-9` fold in the whole package**, and there the two spellings DISAGREE: `max(w, 1e-9)` is
/// `nan` for a NaN `w` where `w.max(1e-9)` is `1e-9`. So it is written as the explicit fold. The
/// obligation was written down at [`demand_coordinate`](crate::demand_coordinate)'s C-law note at
/// step 1, naming this method and this step, and it is discharged here rather than re-derived.
pub fn c_at(
    core: &ScheduledStatorCore, flight: &FlightCondition, a: f64, h: f64, accel: &AccelSchedule,
    w: f64, q: f64, v: f64, rel: f64,
) -> Result<f64, Abort> {
    // Python's `max(w, 1e-9)` — argument 0 is the EXPRESSION, so `w.max(1e-9)` is the wrong
    // spelling on a NaN `w`. See this function's doc for the census that decides it.
    let dw = rel * if 1e-9 > w { 1e-9 } else { w };
    let _sb = MarchedBleed::set(&core.fuel.inner, q);
    let _sv = MarchedStator::set(&core.fuel.inner, v);
    let cap = |x: f64| -> Result<f64, Abort> {
        let i = core.fuel.try_instant_fuel(flight, a, h, x)?;
        Ok(accel.cap(i.base.close.n_hp, i.base.close.pt4 / core.fuel.inner.inner.base.pi_b))
    };
    Ok((cap(w + dw)? - cap(w - dw)?) / (2.0 * dw))
}
