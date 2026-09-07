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

use crate::anti_windup::{rhs_gains_at, WindupScope, WINDUP_LAW_NONE};
use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::demand_coordinate::{applied_demand, LAG_COORD_CLIP, LAG_COORD_DEMAND};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Authority, Floor, FuelLimiters, FuelPoint, FuelTransientCore,
    FuelTransientHooks, PointExtra,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::shared_actuator::{charpoly4, quartic_roots_c, riding4, SharedRigArm};
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

/// Rung 73's `"sched"` reference, under this module's own name.
///
/// [`anti_windup`](crate::anti_windup)'s `REF_APPLIED` one rung on, and for the mirror reason:
/// `cap_gains`'s `row_err` target BRANCHES on which reference the cell was read under, so the
/// string is compared rather than merely threaded, and a bare literal at that comparison is the
/// one place in this file a typo could not be caught by a type.
const REF_SCHED: &str = crate::applied_reference::REF_LAWS_DECLARED[0];

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

// ---------------------------------------------------------------------------------------------
// STEP 5 — THE READERS: § 1's AUTHORITATIVE DIAGONAL, § 2's BILL, § 3's GAIN
// ---------------------------------------------------------------------------------------------
//
// Four readers, and every one of them CONSUMES step 4's four methods rather than re-deriving
// anything: `cap_rows` and `solve_gain` each open with a [`cap_march`], all three public readers
// open with an [`accel_for`], and `cap_rows`/`solve_gain` both close over [`c_at`]. So this step
// re-DRIVES step 4 rather than re-proving it, which is what § 5.31.4 (j) booked here.
//
// THE ONE CALL-SITE DIFFERENCE FROM RUNG 75's READERS IS THE ARMED SCHEDULE, AND IT IS THE RUNG.
// [`windup_rows`](crate::anti_windup::windup_rows) passes `accel = None` into `rhs_gains_at`;
// `cap_rows` passes the real one (`engine.py:19352`). Copying the parent's call site there is the
// single highest-value defect this step can ship, because the resulting reader still returns
// well-formed Jacobians at every point — the accel branch of `_cap_fuel` simply never runs, so
// `sensed` and `solve` agree and § 1's whole headline reads as REFUTED.
//
// **AND IT IS SILENT, WHICH BOUNDS § 5.31.4 (a) ON A THIRD SIDE.** Step 4 deleted the armed leg
// from [`cap_march`] and the drive PANICKED, on [`r76_integrate_fuel`]'s third refusal (*`sensed`
// re-reads rung 48's schedule, so there must BE one*) — from which the domain was written as *the
// blindness is the `solve` arm's, and the arm this rung is about is defended*. Deleting the leg
// HERE is a different question with a different answer: step 5's sweep measured **KILLED by VALUE,
// 144 of 1 814 keys, NO panic**, because a reader reaches `_cap_fuel` directly and never enters
// `integrate_fuel`, so that refusal is not on the path at all. The refusal defends the MARCH's
// callers; it does not defend the READERS, and nothing else does either. What catches it is a
// POSITIVE count of what the two laws differ by — which is what §§ E/F of the drive are.

/// One row of [`cap_rows`] — the SAME state read under BOTH cap laws.
///
/// Every field ending `0` is the `"solve"` arm's reading at the identical state, which is what
/// makes each claim in § 1 a DIFFERENCE between two laws rather than a property of a trajectory —
/// [`WindupRow`](crate::anti_windup::WindupRow)'s discipline, one knob over.
///
/// # IT CARRIES THREE CAPS WHERE THE PARENT CARRIED NONE, AND THAT IS THE MIN-SELECT GUARD
///
/// `_cap_fuel` is `min(accel, phi)` — min-select ONE LEVEL DOWN — and the phi leg has no sensed
/// form. So wherever the phi cap is the lower one the knob is INERT, and an aggregate that pooled
/// those points would report a real law as broken. Worse, the two laws being differenced sit on
/// OPPOSITE SIDES of that `min` near a crossover, so a point can be accel-bound under `solve` and
/// phi-bound under `sensed`; differencing it measures a LEG CHANGE and reports it as a law that
/// broke. [`accel_binds`](CapRow::accel_binds) is the filter that answers both, and it is rung 72's
/// `switch_guard` one min-select level down.
#[derive(Clone, Copy, Debug)]
pub struct CapRow {
    pub s: f64,
    pub auth: Authority,
    pub masked: Authority,
    /// `d(cap_sensed)/dw` at this point, from [`c_at`] at the APPLIED demand `mf_app`.
    pub c: f64,
    /// `max(cap_a, cap_a2) < cap_s` — the accel leg is the lower cap under **BOTH** laws.
    ///
    /// Python's `max` here has TWO EXPRESSIONS and no literal, which is a third category beside
    /// step 1's literal-first census and [`c_at`]'s single expression-first fold. Both arguments
    /// can be `f64::INFINITY` (`_cap_fuel` returns the empty `min`'s identity when neither leg is
    /// armed), so the fold is spelled out rather than taken as `f64::max`: Python replaces only on
    /// a strict `>` and keeps argument 0 on a tie, and `f64::max` additionally discards a NaN that
    /// Python would propagate.
    ///
    /// **AND THE WHOLE GUARD IS INERT AT THE SUITE'S OWN MARGIN.** Step 5's drive measured
    /// `n_inert = 0` in every cell at `margin = 0.10` — 10 of 10 rows bind under both laws, so
    /// nothing is filtered and the three caps above are computed to decide a constant. Replacing
    /// this `max` with a `min` moves **0 keys at 0.10 and 33 at 0.20**, where 7 of 9 rows bind.
    /// That is step 4 § (b)'s *a drive that picks one cell scores the bug as correct* moved from a
    /// reload guard onto a min-select, and it is why the drive carries two margins.
    pub accel_binds: bool,
    /// The accel cap under `solve` — Python's `cap_a`. `cap_a2`, the sensed one, is NOT recorded:
    /// it exists only inside [`accel_binds`](CapRow::accel_binds), exactly as in the source.
    pub cap_accel: f64,
    /// The PHI cap — `_cap_fuel(…, accel = None, surge)`, read under whatever law the receiver
    /// carries, because the source's third call is a BARE one with no scope around it.
    pub cap_phi: f64,
    /// The clock the AUTHORITATIVE row divides by, and the one the MASKED row does.
    ///
    /// **THE TWO ARE THE SAME NUMBER ON EVERY SHIPPED GRID, so this pair is a DEFENCE WITH NO
    /// READER** ([[rust-port-slice-aa-steps2345]]). `taus[1]` is `0.05` and rung 52's lag returns
    /// its ATTACK clock, also `0.05`, at every riding point of an accel ramp — so step 5's sweep
    /// SWAPPED the two conditions and **0 of 1 814 keys moved**, at both margins. The distinction
    /// is real in the source and undrivable on this plant; it is written the source's way and the
    /// unreachability is disclosed here rather than left for a later slice to wonder about.
    pub tau_auth: f64,
    pub tau_masked: f64,
    pub auth_diag: f64,
    pub auth_diag0: f64,
    pub masked_diag: f64,
    pub masked_diag0: f64,
    pub row_auth: f64,
    pub row_auth0: f64,
    pub mask_leak: f64,
    pub mask_leak0: f64,
    pub det: f64,
    pub det0: f64,
    /// **P9** — the GOVERNOR's row cannot move in any cell, because `_cap_gov` has no branch.
    /// `max |J[1][j] - J0[1][j]|` over the four columns.
    pub gov_row: f64,
    pub zeros: usize,
    pub zeros0: usize,
}

/// Python's `max(seq)` — replace only on a strict `>`, keep argument 0 on a tie, propagate a NaN
/// that arrived FIRST.
///
/// `f64::max` does none of those three the same way, and this rung is the first in the slice where
/// a NaN is reachable in an aggregate at all ([`SolveGainRow::gain`] is `nan` when `dS == 0`). The
/// crate's older aggregates spell `fold(f64::NEG_INFINITY, f64::max)`, which is bit-identical on
/// every NaN-free population and silently different on this one; step 5's folds are all written
/// this way instead, and the difference is named rather than inherited.
fn py_max<T, F: Fn(&T) -> f64>(xs: &[T], f: F) -> f64 {
    xs.iter().map(f).reduce(|a, b| if b > a { b } else { a })
      .expect("Python's `max()` raises on an empty sequence; every caller filters first")
}

/// Python's `min(seq)` — [`py_max`]'s mirror.
fn py_min<T, F: Fn(&T) -> f64>(xs: &[T], f: F) -> f64 {
    xs.iter().map(f).reduce(|a, b| if b < a { b } else { a })
      .expect("Python's `min()` raises on an empty sequence; every caller filters first")
}

/// RUNG 76's `_cap_rows` — **`sensed` against `solve` AT THE SAME STATES, both through
/// [`rhs_laws`](crate::anti_windup::rhs_laws).**
///
/// # THE STATES ARE THE CLIP PLANT's AT THE INHERITED FLOOR, AND THEY ARE READ UNDER `solve`
///
/// Rung 74 § 1.3 / rung 75 § 1.2's disclosure inherited word for word — and here it acquires a
/// second, harder reason: `clip × sensed` is REFUSED ([`r76_integrate_fuel`]'s second assert), so
/// the base march CANNOT carry this rung's law. The difference measured is therefore PURELY the
/// law at ONE state, which is what a Jacobian comparison has to be. The march runs
/// `clip × none × solve`; both devices are then applied IN THE READER.
///
/// # `_lag_coord` IS FLIPPED BY A PLAIN ASSIGNMENT, AND IT IS THE READER's ADMISSION TICKET
///
/// `engine.py:19341` writes `m._lag_coord = "demand"` after the march and before the filter, by
/// plain attribute assignment — a direct `set`, no scope and no hook, for
/// [`windup_rows`](crate::anti_windup::windup_rows)'s reason. § 5.31.3 (a) measured what dropping
/// the identical line one rung down costs: not a value break but a PANIC, because rung 75's
/// `windup_tau` refuses `track` outside the plain demand coordinate. **That refusal is live HERE
/// too and it is MEASURED, not inherited**: step 5's sweep dropped this line and hit
/// `anti_windup.rs:261` — rung 75's assert, reached from rung 76's reader, at the two `track`
/// cells of the `laws` grid. The `none` cells are silent, so a reader driven at `none` alone would
/// have scored the deletion SURVIVED.
///
/// # THE FOUR DIFFERENCE STEPS ARE `_rhs_gains_at`'s **DEFAULTS**, NOT THIS READER's CHOICE
///
/// `engine.py:18789-18790` declares `dg = 1e-7, dq = 1e-5, dv = 1e-4, switch_guard = 4.0` and both
/// callers take them; Rust has no defaults, so they are spelled at the call site. They coincide
/// with the numbers [`windup_rows`](crate::anti_windup::windup_rows) writes — which is a fact about
/// the DEFAULTS and not a value copied from the sibling, and the distinction is written down
/// because the next reader to add a call site can only get it right from the signature.
///
/// # AND THE `accel` ARGUMENT IS THE ONE THAT MAKES THIS READER RUNG 76 AT ALL
///
/// See this section's header: `Some(accel)`, where the parent passes `None`.
#[allow(clippy::too_many_arguments)]
pub fn cap_rows(
    core: &ScheduledStatorCore, flight: &FlightCondition, sm: f64, ref_law: &'static str,
    law: &'static str, tau_t: Option<f64>, taus: (f64, f64, f64, f64), inc: bool, tt4_lo: f64,
    tt4_hi: f64, tt4_max: f64, r: f64, s_settle: f64, ds: f64, v_max: f64,
    accel: &AccelSchedule, every: usize,
) -> (Vec<CapRow>, usize) {
    // THE BASE MARCH CARRIES NEITHER DEVICE — `clip` × `none` × `solve`.
    let (m, surge, lag, traj) = cap_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
        LAG_COORD_CLIP, ref_law, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE, accel, None);
    // `m._lag_coord = "demand"` — PLAIN, see the header.
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let lag = lag.expect("`_shared_rig` arms the fuel leg, so it carries the lag");
    let mut rows: Vec<CapRow> = Vec::new();
    for p in pts.iter().step_by(every) {
        let (required_fuel, g_fuel, g_gov, b, v, wf_opt, wr_opt) = match p.extra {
            PointExtra::Demand { required_fuel, g_fuel, g_gov, b, v, w_fuel, w_gov, .. } =>
                (required_fuel, g_fuel, g_gov, b, v, Some(w_fuel), Some(w_gov)),
            PointExtra::Shared { required_fuel, g_fuel, g_gov, b, v, .. } =>
                (required_fuel, g_fuel, g_gov, b, v, None, None),
            _ => panic!("rung-76's rows read `required_fuel`/`g_fuel`/`b`/`v` off every filtered \
                         point, and project `w_fuel`/`w_gov` when the point does not carry them."),
        };
        let tau_f = lag.tau(required_fuel, g_fuel);
        let mf_sched = p.mf_sched;
        let wf = wf_opt.unwrap_or(mf_sched - g_fuel);
        let wr = wr_opt.unwrap_or(mf_sched - g_gov);
        // Python's `(wf, wr, mf_sched)` against this crate's `(mf_sched, wf, wr)` — a plain
        // minimum of all three, so the permutation is unobservable. Named for
        // [`rhs_gains_at`](crate::anti_windup::rhs_gains_at)'s reason: a gate here would score
        // SURVIVED structurally and read as coverage.
        let ma = applied_demand(mf_sched, wf, wr);
        // `_with_windup(law, tau_t, m._with_cap, cl, m._rhs_gains_at, …)` — the windup scope
        // OUTSIDE, the cap scope INSIDE. Rust drops in reverse declaration order, which is
        // Python's inner-`finally`-first unwind exactly. **The ORDER is unobservable and is
        // measured to be**: the two guards write disjoint fields, so step 5's sweep nested them
        // the other way round and 0 of 1 814 keys moved. Written the source's way because the
        // source is what a later reader will diff against, not because a gate could tell.
        let read = |cl: &'static str| {
            let _ws = WindupScope::set(&m.fuel.inner, law, tau_t);
            let _cs = CapScope::set(&m.fuel.inner, cl);
            rhs_gains_at(&m, flight, p, Some(accel), surge.as_ref(), tt4_max, tau_f, taus.1,
                         1e-7, 1e-5, 1e-4, 4.0).unwrap_or_else(|e| panic!("{}", e.0))
        };
        let g = read(CAP_LAW_SENSED);
        if !g.interior || g.masked.is_none() {
            continue;
        }
        let g0 = read(CAP_LAW_SOLVE);
        if !g0.interior {
            continue;
        }
        let c = c_at(&m, flight, p.nu_lp, p.nu_hp, accel, ma, b, v, C_AT_REL)
            .unwrap_or_else(|e| panic!("{}", e.0));
        // THE THREE CAPS, all inside BOTH state guards — and they differ in ALL THREE of `accel`,
        // `surge` and `mf_app`, so unlike `applied_demand` above a transposition here IS
        // observable. Python's `finally` restores both states to `None`, which is what
        // [`MarchedBleed`]/[`MarchedStator`] do on drop.
        let (cap_a, cap_a2, cap_s) = {
            let _sb = MarchedBleed::set(&m.fuel.inner, b);
            let _sv = MarchedStator::set(&m.fuel.inner, v);
            let cf = |accel: Option<&AccelSchedule>, surge: Option<&Floor>, mf_app: Option<f64>| {
                (m.fuel.inner.triple_hooks.cap_fuel)(
                    &m.fuel, flight, p.nu_lp, p.nu_hp, mf_sched, accel, surge, mf_app)
                    .unwrap_or_else(|e| panic!("{}", e.0))
            };
            let a1 = { let _cs = CapScope::set(&m.fuel.inner, CAP_LAW_SOLVE);
                       cf(Some(accel), None, None) };
            let a2 = { let _cs = CapScope::set(&m.fuel.inner, CAP_LAW_SENSED);
                       cf(Some(accel), None, Some(ma)) };
            // THE THIRD IS A **BARE** CALL — no scope. It reads whatever law the receiver carries,
            // which the march above left at `"solve"`; `accel = None` means the cell is never
            // reached anyway, so the law is unobservable here and the source's shape is kept.
            let s3 = cf(None, surge.as_ref(), None);
            (a1, a2, s3)
        };
        let rate = 1.0 / tau_f + 1.0 / taus.1 + 1.0 / taus.2 + 1.0 / taus.3;
        let jm = g.j.expect("an interior reading carries J");
        let jm0 = g0.j.expect("an interior reading carries J");
        let ch = charpoly4(&jm);
        let ch0 = charpoly4(&jm0);
        let rt = quartic_roots_c(&ch);
        let rt0 = quartic_roots_c(&ch0);
        let auth = g.authority.expect("an interior reading carries an authority");
        let masked = g.masked.expect("filtered above");
        rows.push(CapRow {
            s: p.s,
            auth,
            masked,
            c,
            // Python's two-expression `max`, spelled as the fold — see [`CapRow::accel_binds`].
            accel_binds: (if cap_a2 > cap_a { cap_a2 } else { cap_a }) < cap_s,
            cap_accel: cap_a,
            cap_phi: cap_s,
            tau_auth: if auth == Authority::Fuel { tau_f } else { taus.1 },
            tau_masked: if masked == Authority::Fuel { tau_f } else { taus.1 },
            auth_diag: g.auth_diag.expect("a masked leg implies an authoritative one"),
            auth_diag0: g0.auth_diag.expect("the same state, the same mask"),
            masked_diag: g.masked_diag.expect("a masked leg has a diagonal"),
            masked_diag0: g0.masked_diag.expect("the same state, the same mask"),
            row_auth: g.masked_row_auth.expect("both indices exist here"),
            row_auth0: g0.masked_row_auth.expect("both indices exist here"),
            mask_leak: g.mask_leak.expect("both indices exist here"),
            mask_leak0: g0.mask_leak.expect("both indices exist here"),
            det: ch[4],
            det0: ch0[4],
            gov_row: (0..4).map(|j| (jm[1][j] - jm0[1][j]).abs())
                           .reduce(|x, y| if y > x { y } else { x })
                           .expect("four columns"),
            zeros: rt.iter().filter(|z| z.abs() < 1e-4 * rate).count(),
            zeros0: rt0.iter().filter(|z| z.abs() < 1e-4 * rate).count(),
        });
    }
    (rows, pts.len())
}

/// One cell of [`cap_gains`] — Python's `cells[f"{ref}|{law}|{auth}"]`, a **SIX-key empty dict OR a
/// twenty-six-key reading**.
///
/// An enum for [`WindupCell`](crate::anti_windup::WindupCell)'s reason, and the empty arm is WIDER
/// than the parent's: rung 75's carries `n` and `n_riding` alone, where this one adds `ref`, `law`,
/// `auth` **and `n_inert`** — the count of points at which the PHI cap was the lower one, so the
/// knob was inert by construction. A cell that reports zero live rows is not the same fact as a
/// cell that had none to begin with, and the source keeps the two distinguishable even where it
/// keeps nothing else.
#[derive(Clone, Debug)]
pub enum CapCell {
    /// `dict(n=0, n_riding=n, ref=…, law=…, auth=…, n_inert=…)`.
    Empty { n_riding: usize, ref_law: &'static str, law: &'static str, auth: Authority,
            n_inert: usize },
    Read(Box<CapCellRead>),
}

/// The twenty-six keys a populated [`CapCell`] carries.
#[derive(Clone, Debug)]
pub struct CapCellRead {
    pub n: usize,
    pub n_riding: usize,
    pub ref_law: &'static str,
    pub law: &'static str,
    pub auth: Authority,
    /// The points where the OTHER cap was the lower one. **REPORTED, NEVER POOLED IN.**
    pub n_inert: usize,
    pub c: (f64, f64),
    /// **P2/P4** — the AUTHORITATIVE diagonal, which nothing in rungs 73/74/75 could move.
    pub auth_diag: (f64, f64),
    pub auth_diag0: (f64, f64),
    pub auth_moved: f64,
    /// `max |auth_diag - (c-1)/tau_auth| * tau_auth` — the LAW, scored per point against THAT
    /// point's own `c`. Pooling a single `c` across the cell would be rung 73 § 4's failure.
    pub auth_err: f64,
    pub masked_diag: (f64, f64),
    pub masked_moved: f64,
    /// **P6** — the masked ROW's coupling to the leg that holds.
    pub row_auth: (f64, f64),
    pub row_auth0: (f64, f64),
    /// `None` on the FUEL-authoritative cell: the target is only stated where the governor holds.
    pub row_err: Option<f64>,
    /// **P5** — the masked COLUMN, untouched: `n_live <= 3` a FIFTH time.
    pub mask_leak: f64,
    pub mask_leak0: f64,
    /// **P7** — `det J` scales by `1 - c`, per point.
    pub det: (f64, f64),
    pub det0: (f64, f64),
    pub det_ratio: Option<(f64, f64)>,
    pub det_err: Option<f64>,
    /// **P8** — the spectrum's count.
    pub zeros: (usize, usize),
    pub zeros0: (usize, usize),
    pub zeros_moved: usize,
    /// **P9** — the governor's row is bit-identical.
    pub gov_row: f64,
}

/// RUNG 76's `cap_gains` return.
#[derive(Clone, Debug)]
pub struct CapGains {
    pub phi_lim: f64,
    pub margin: f64,
    pub taus: (f64, f64, f64, f64),
    pub tau_t: f64,
    pub inc: bool,
    pub ds: f64,
    /// Python's `cells`, keyed `f"{ref}|{law}|{auth}"`. **The key is built HERE, not by the
    /// oracle**, because all three components are strings —
    /// [`WindupGains::cells`](crate::anti_windup::WindupGains::cells) carries a tuple only because
    /// one of ITS two components is an `f64` with no shortest-repr spelling in Rust. Inheriting
    /// that workaround would be a mechanism with no reason.
    pub cells: Vec<(String, CapCell)>,
}

/// RUNG 76 § 1 — **a device in a leg's LAW reaches only the MASKED leg; a device in the PLANT THE
/// LEGS READ reaches only the AUTHORITATIVE one.**
///
/// Rung 75's back-calculation is a term in the masked leg's own law, and min-select masks a LAW —
/// so it wrote `-1/tau_t` on the MASKED diagonal and left the authoritative one *moved 0.0
/// relative*, as rungs 73 and 74 each also report. A sensed cap is in NO leg's law: it is in the
/// plant BOTH legs read through `mf_app`, and min-select cannot mask a PLANT. So it writes
/// `c/tau_f` on the AUTHORITATIVE fuel diagonal — the one entry this whole family has measured at
/// exactly zero, four rungs running.
///
/// # THE TWO AUTHORITY CELLS CARRY DIFFERENT HALVES OF IT AND ARE NEVER POOLED
///
/// * **FUEL authoritative** — the fuel row IS the authoritative row: `d(cap)/dw_f = c`, diagonal
///   `(c-1)/tau_f`, and `det J` scales by `1 - c`. The masked GOVERNOR's cap has no sensed form, so
///   its row does not move at all ([`gov_row`](CapCellRead::gov_row), P9).
/// * **GOV authoritative** — the fuel leg is MASKED and reads the governor through `mf_app`: its
///   diagonal is UNMOVED (`min` is flat in what the masked leg holds) and its CROSS moves. `det J`
///   is then unmoved EXACTLY.
///
/// That split is rung 73 § 4's pooling failure and rung 75 § 1.4's, inherited rather than
/// rediscovered — and it is why [`row_err`](CapCellRead::row_err) exists only on the `gov` cell.
///
/// # `c` IS PER POINT, AND EVERY RATIO IS SCORED AGAINST THAT POINT's OWN `c`
///
/// [`c_at`] is called once per row and the errors difference against it there. A cell-wide `c`
/// would be a fitted constant on a rung whose whole claim is that the number is MEASURED.
#[allow(clippy::too_many_arguments)]
pub fn cap_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), tau_t: f64, refs: &[&'static str],
    laws: &[&'static str], inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64, every: usize,
) -> CapGains {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let accel = accel_for(core, flight, tt4_lo, tt4_hi, sm, tt4_max, taus, v_max, inc, margin);
    let mut cells: Vec<(String, CapCell)> = Vec::new();
    for ref_law in refs {
        for law in laws {
            let (rows, n) = cap_rows(
                core, flight, sm, ref_law, law,
                if *law == crate::anti_windup::WINDUP_LAW_TRACK { Some(tau_t) } else { None },
                taus, inc, tt4_lo, tt4_hi, tt4_max, r, s_settle, ds, v_max, &accel, every);
            for auth in [Authority::Fuel, Authority::Gov] {
                let n_inert = rows.iter().filter(|x| x.auth == auth && !x.accel_binds).count();
                let rr: Vec<CapRow> =
                    rows.iter().filter(|x| x.auth == auth && x.accel_binds).copied().collect();
                let key = format!("{}|{}|{}", ref_law, law, auth.as_str());
                if rr.is_empty() {
                    cells.push((key, CapCell::Empty {
                        n_riding: n, ref_law, law, auth, n_inert }));
                    continue;
                }
                // Python's `det_rat` — the PAIR, so every residual is scored against the `c` of
                // the point its ratio came from.
                let det_rat: Vec<(f64, f64)> = rr.iter().filter(|x| x.det0.abs() > 1e-30)
                                                 .map(|x| (x.det / x.det0, x.c)).collect();
                let sched = *ref_law == REF_SCHED;
                cells.push((key, CapCell::Read(Box::new(CapCellRead {
                    n: rr.len(),
                    n_riding: n,
                    ref_law,
                    law,
                    auth,
                    n_inert,
                    c: (py_min(&rr, |x| x.c), py_max(&rr, |x| x.c)),
                    auth_diag: (py_min(&rr, |x| x.auth_diag), py_max(&rr, |x| x.auth_diag)),
                    auth_diag0: (py_min(&rr, |x| x.auth_diag0), py_max(&rr, |x| x.auth_diag0)),
                    auth_moved: py_max(&rr, |x| (x.auth_diag - x.auth_diag0).abs()
                                                / 1e-30f64.max(x.auth_diag0.abs())),
                    auth_err: py_max(&rr, |x| (x.auth_diag - (x.c - 1.0) / x.tau_auth).abs()
                                              * x.tau_auth),
                    masked_diag: (py_min(&rr, |x| x.masked_diag), py_max(&rr, |x| x.masked_diag)),
                    // Python's conditional expression INSIDE the generator: the RELATIVE move
                    // where the base is nonzero, the ABSOLUTE one where it is not. A single
                    // `/max(1e-30, .)` would agree numerically on every live point and lose the
                    // source's shape at the one it was written for — MEASURED: collapsing it to
                    // the relative form alone moves 0 of 1 814 keys, because a live 4x4's masked
                    // diagonal is never within `1e-30` of zero. A third defence with no reader in
                    // this step, disclosed rather than discovered later.
                    masked_moved: py_max(&rr, |x| if x.masked_diag0.abs() > 1e-30 {
                        (x.masked_diag - x.masked_diag0).abs()
                            / 1e-30f64.max(x.masked_diag0.abs())
                    } else {
                        x.masked_diag.abs()
                    }),
                    row_auth: (py_min(&rr, |x| x.row_auth), py_max(&rr, |x| x.row_auth)),
                    row_auth0: (py_min(&rr, |x| x.row_auth0), py_max(&rr, |x| x.row_auth0)),
                    row_err: if auth == Authority::Gov {
                        Some(py_max(&rr, |x| {
                            let tgt = if sched { x.c } else { x.c - 1.0 };
                            (x.row_auth - tgt / x.tau_masked).abs() * x.tau_masked
                        }))
                    } else {
                        None
                    },
                    mask_leak: py_max(&rr, |x| x.mask_leak.abs()),
                    mask_leak0: py_max(&rr, |x| x.mask_leak0.abs()),
                    det: (py_min(&rr, |x| x.det), py_max(&rr, |x| x.det)),
                    det0: (py_min(&rr, |x| x.det0), py_max(&rr, |x| x.det0)),
                    det_ratio: if det_rat.is_empty() { None } else {
                        Some((py_min(&det_rat, |x| x.0), py_max(&det_rat, |x| x.0)))
                    },
                    det_err: if det_rat.is_empty() { None } else {
                        Some(py_max(&det_rat, |x| (x.0 - (1.0 - x.1)).abs()))
                    },
                    zeros: (rr.iter().map(|x| x.zeros).min().expect("non-empty"),
                            rr.iter().map(|x| x.zeros).max().expect("non-empty")),
                    zeros0: (rr.iter().map(|x| x.zeros0).min().expect("non-empty"),
                             rr.iter().map(|x| x.zeros0).max().expect("non-empty")),
                    zeros_moved: rr.iter()
                                   .map(|x| (x.zeros as i64 - x.zeros0 as i64).unsigned_abs()
                                            as usize)
                                   .max().expect("non-empty"),
                    gov_row: py_max(&rr, |x| x.gov_row),
                }))));
            }
        }
    }
    CapGains { phi_lim, margin, taus, tau_t, inc, ds, cells }
}

// ---------------------------------------------------------------------------------------------
// § 2 — `cap_bill`: THE PATH MOVES AND THE DESTINATION DOES NOT
// ---------------------------------------------------------------------------------------------

/// RUNG 76's `cap_bill` return.
#[derive(Clone, Debug)]
pub struct CapBill {
    pub phi_lim: f64,
    pub margin: f64,
    pub ref_law: &'static str,
    pub law: &'static str,
    pub inc: bool,
    pub ds: f64,
    pub n: usize,
    pub s_tail: f64,
    /// **P11** — the sign of the BILL, `(solve, sensed)` in Python's tuple order.
    pub max_tt4: (f64, f64),
    pub min_phi: (f64, f64),
    /// `sum(mf) * ds` on each arm — a LEFT-TO-RIGHT fold seeded at `0.0`, which is Python's `sum`
    /// exactly. A pairwise or SIMD reduction would be a different float, and § 5.31 (vii)'s **P2**
    /// names precisely these two keys as the CPython arm's only exemption in 83 273: they are the
    /// two `sum()` calls in either class that add a 341-long trajectory rather than a literal `1`.
    /// Changing the summation order here would make that prediction untestable at step 6.
    pub fuel_int: (f64, f64),
    /// **P10** — the destination is the SAME, the path is not. `None` when the window is empty.
    pub wf_tail: Option<f64>,
    pub wf_ramp: Option<f64>,
    pub cuts_harder: bool,
    pub traj_solve: Vec<FuelPoint>,
    pub traj_sensed: Vec<FuelPoint>,
}

/// RUNG 76 § 2 — **the knob is a PURE TRANSIENT DEVICE on the leg that holds.**
///
/// Setting `dw_f/ds = 0` with the fuel leg authoritative (`mf_app = w_f`) gives
/// `w_f* = cap_sensed(w_f*) = cap_solve` EXACTLY, because `cap_solve` is BY CONSTRUCTION the fixed
/// point of `cap_sensed` — and under `applied` identically, since `_demand_reference` returns `cap`
/// itself when `mf_app == w_own`. So the two laws have the SAME equilibrium and different paths:
/// the marching lag performs the set-point solve's fixed-point iteration IN TIME rather than at a
/// POINT, at contraction ratio `c` per lag time constant.
///
/// The BILL's sign is the claim and the magnitude is reported with its `ds` band: during the ramp
/// `mf_app < cap_solve`, so `cap_sensed < cap_solve` (the droop identity, D1) and the sensed leg
/// cuts HARDER.
///
/// # THE GRID-EQUALITY ASSERT IS THE ONLY MESSAGE IN EITHER CLASS THAT DOES NOT NAME ITS RUNG
///
/// `engine.py:19523` — *"the two cap laws marched different grids"*. Every other assert in
/// `AntiWindupTransient` and `SensedCapTransient` opens `rung-75:` or `rung-76:`; this one opens
/// with nothing. § 5.31 (v) counted them (4 of 4 tagged at rung 75, **4 of 5** here) and § 5.31
/// (vii)'s **P6** names this message as the candidate for a port defect that every ported gate
/// passes, because a suite needle checking for a `rung-76:` prefix cannot see it. **So it is ported
/// VERBATIM and UNTAGGED.** Adding a prefix would be an improvement that silently settles P6 in the
/// port's favour before step 7 can test it.
///
/// # THE TAIL WINDOW IS `r + tail * max(taus)`, AND `max` OVER A TUPLE IS PYTHON's FOLD
///
/// [`py_max`]'s rule, on four constants rather than on a row field — and a **FOURTH defence with
/// no reader**: every shipped grid in this family sets all four clocks to `0.05`, so `min` and
/// `max` return the same number and step 5's sweep measured the substitution at 0 of 1 814 keys.
///
/// # AND `fuel_int`'s FOLD DIRECTION IS THE ONE SUMMATION IN THIS STEP THAT IS LOAD-BEARING
///
/// Folding the 341-point trajectory right-to-left instead of left-to-right moves **exactly 4
/// keys** — the two arms of the two driven bill cells and nothing else in the file. That is
/// § 5.31 (vii)'s **P2** asked of the port rather than of the interpreters: the two
/// [`fuel_int`](CapBill::fuel_int) values are the only place in either class where summation
/// ORDER can be observed, which is why they are also the only two keys the CPython arm needs an
/// exemption for.
#[allow(clippy::too_many_arguments)]
pub fn cap_bill(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), tau_t: f64, ref_law: &'static str,
    law: &'static str, inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64, tail: f64,
) -> CapBill {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let accel = accel_for(core, flight, tt4_lo, tt4_hi, sm, tt4_max, taus, v_max, inc, margin);
    let mut out: Vec<Vec<FuelPoint>> = Vec::with_capacity(2);
    for cl in [CAP_LAW_SOLVE, CAP_LAW_SENSED] {
        out.push(cap_march(
            core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
            LAG_COORD_DEMAND, ref_law, law,
            if law == crate::anti_windup::WINDUP_LAW_TRACK { Some(tau_t) } else { None },
            cl, &accel, None).3);
    }
    let b = out.pop().expect("two arms");
    let a = out.pop().expect("two arms");
    // VERBATIM AND UNTAGGED — see this function's doc.
    assert!(a.len() == b.len(), "the two cap laws marched different grids");
    let s_tail = r + tail * [taus.0, taus.1, taus.2, taus.3].into_iter()
                                .reduce(|x, y| if y > x { y } else { x }).expect("four clocks");
    let ramp: Vec<usize> = a.iter().enumerate().filter(|(_, p)| p.s <= r).map(|(i, _)| i).collect();
    let tl: Vec<usize> = a.iter().enumerate().filter(|(_, p)| p.s >= s_tail).map(|(i, _)| i)
                          .collect();
    let w_fuel_of = |p: &FuelPoint| match p.extra {
        PointExtra::Demand { w_fuel, .. } => w_fuel,
        _ => panic!("rung-76's bill reads `w_fuel` off every marched point of a DEMAND march."),
    };
    let wf: Vec<f64> = (0..a.len())
        .map(|i| (w_fuel_of(&b[i]) - w_fuel_of(&a[i])).abs() / 1e-30f64.max(w_fuel_of(&a[i]).abs()))
        .collect();
    // Python's `sum(key(t, "mf")) * ds` — left to right from `0.0`. See [`CapBill::fuel_int`].
    let fold_mf = |t: &[FuelPoint]| t.iter().fold(0.0f64, |acc, p| acc + p.mf) * ds;
    CapBill {
        phi_lim, margin, ref_law, law, inc, ds,
        n: a.len(),
        s_tail,
        max_tt4: (py_max(&a, |p| p.tt4), py_max(&b, |p| p.tt4)),
        min_phi: (py_min(&a, |p| p.phi_lp), py_min(&b, |p| p.phi_lp)),
        fuel_int: (fold_mf(&a), fold_mf(&b)),
        wf_tail: if tl.is_empty() { None } else { Some(py_max(&tl, |i| wf[*i])) },
        wf_ramp: if ramp.is_empty() { None } else { Some(py_max(&ramp, |i| wf[*i])) },
        cuts_harder: ramp.iter().all(|&i| b[i].mf <= a[i].mf + 1e-15),
        traj_solve: a,
        traj_sensed: b,
    }
}

// ---------------------------------------------------------------------------------------------
// § 3 — `solve_gain`: WHAT THE SOLVE WAS BUYING — a GAIN, not a relocation
// ---------------------------------------------------------------------------------------------

/// One row of [`solve_gain`] — one marched state, both laws differenced in the VALVE.
#[derive(Clone, Copy, Debug)]
pub struct SolveGainRow {
    pub s: f64,
    pub cap_solve: f64,
    pub c: f64,
    /// `|cap_sensed(cap_solve) - cap_solve|` — identity (1), and it is EXACT, not small.
    ///
    /// **AND *EXACT* IS THE PROBLEM, WHICH IS A REQUIREMENT ON STEP 6 AND NOT A PORT DEFECT.**
    /// Step 5's drive measured this quantity at `+0.0` **BIT FOR BIT at 8 of 10 rows at the
    /// suite's own margin** (4 of 10 at `0.05`, 3 of 9 at `0.40`). At every one of those rows
    /// `sensed(q, w0)` returns `w0` itself, so a reader that had wrongly written `solve(q) - w0`
    /// — comparing the solve with itself — would report the identical `0.0` and be scored as
    /// verifying the identity. The sweep catches it only through the minority of rows where the
    /// last bits differ: **17 keys of 1 814, none of them at `margin = 0.10`'s eight zeros.** So a
    /// step-6 gate on identity (1) must assert on a row where this field is NONZERO, or it is
    /// [[instrument-fed-by-what-it-certifies]] in its purest form — an instrument certified by
    /// the exactness it exists to report.
    pub fixed_point: f64,
    /// `d(cap_sensed)/dq` and `d(cap_solve)/dq` at the SAME `w = cap_solve`.
    pub d_s: f64,
    pub d_d: f64,
    /// `dD/dS`, or **`nan` when `dS` is exactly zero** — Python's own conditional. This is the one
    /// NaN reachable by construction anywhere in slice AG, and it is why [`py_max`] exists.
    pub gain: f64,
    pub predicted: f64,
}

/// RUNG 76's `solve_gain` return.
#[derive(Clone, Debug)]
pub struct SolveGain {
    pub phi_lim: f64,
    pub margin: f64,
    pub ref_law: &'static str,
    pub inc: bool,
    pub n: usize,
    pub rows: Vec<SolveGainRow>,
    /// `None` on an empty row set — Python's `if rows else None`, three times.
    pub fixed_point: Option<f64>,
    pub gain: Option<(f64, f64)>,
    pub gain_err: Option<f64>,
}

/// RUNG 76 § 3 — **THE SET-POINT SOLVE IS NOT A RELOCATION OF THE CAP, IT IS A GAIN ON IT.**
///
/// **This section is DERIVATION AFTER MEASUREMENT and is labelled so** — it was written to explain
/// why § 1's `det J` ratio misses `1 - c` by ~0.7 % when the whole fuel row was derived to scale by
/// exactly `1 - c`. It does, at ONE `w`; the two laws are read at DIFFERENT `w` (`mf_app` against
/// `cap_solve`), and that is the entire residual.
///
/// TWO IDENTITIES, both with zero fitted constants:
///
/// 1. `cap_sensed(cap_solve) = cap_solve` EXACTLY — `cap_solve` is by construction the fixed point
///    of `cap_sensed`, so the two laws AGREE at the solve's own answer. This is D2 as a property of
///    the LAWS, and it is why the equilibrium of a leg that HOLDS does not move. The march's tail
///    is NOT that equilibrium — the spools are still spinning up there — which is why the
///    trajectory form of the claim is scored REFUTED and gated as such.
/// 2. `d(cap_solve)/dq = ( d(cap_sensed)/dq ) / ( 1 - c )` at the SAME `w = cap_solve`.
///    Differentiating the fixed point `cap = cap_sensed(cap, q)` gives it in one line. **So the
///    solve AMPLIFIES the cap's sensitivity to every other state by `1/(1-c)`** — a limiter written
///    as a solve is a STIFFER limiter than the schedule it claims to implement, and nothing in
///    rungs 48–75 could see that, because none of them had a second reading of the same cap to
///    difference against.
///
/// `q` is the VALVE state, chosen because it is the one other state the cap depends on through the
/// PLANT and not through any leg's law.
///
/// # THE `nan` GUARD IS A DEFENCE WHOSE READER IS MEASURED, NOT ASSUMED
///
/// [`SolveGainRow::gain`] is `nan` when `dS == 0.0`, and that NaN then flows into
/// [`gain`](SolveGain::gain) and [`gain_err`](SolveGain::gain_err), where Python's `min`/`max`
/// propagate it only if it arrives FIRST and the crate's older `fold(±INF, f64::min)` would discard
/// it always. Step 5's drive COUNTS the exact-zero `dS` occurrences rather than reasoning about
/// them, and the count is **0 of 29 rows across all three margins** — so the NaN is unreachable on
/// this plant, the two fold spellings are indistinguishable by any value it can produce, and
/// [`py_max`]/[`py_min`] are themselves a **defence with no reader**, the fifth this step ships.
/// They are used throughout it anyway, because the alternative is a fold that is wrong for a
/// reason no one would find later; the unreachability is disclosed here, at the step that shipped
/// it, rather than left for a slice that wonders why the crate has two aggregate spellings.
#[allow(clippy::too_many_arguments)]
pub fn solve_gain(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), ref_law: &'static str, inc: bool,
    r: f64, s_settle: f64, ds: f64, v_max: f64, dq: f64, every: usize,
) -> SolveGain {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let accel = accel_for(core, flight, tt4_lo, tt4_hi, sm, tt4_max, taus, v_max, inc, margin);
    let (m, _surge, _lag, traj) = cap_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
        LAG_COORD_CLIP, ref_law, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE, &accel, None);
    // `m._lag_coord = "demand"` — PLAIN, [`cap_rows`]'s reason.
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let mut rows: Vec<SolveGainRow> = Vec::new();
    for p in pts.iter().step_by(every) {
        let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
        let (q, v) = match p.extra {
            PointExtra::Demand { b, v, .. } | PointExtra::Shared { b, v, .. } => (b, v),
            _ => panic!("rung-76's § 3 reads `b`/`v` off every filtered point."),
        };
        // Python's two closures, each setting BOTH state fields and restoring both to `None` in a
        // `finally` — [`c_at`]'s third case of the `b_state`/`v_state` boundary, twice more. The
        // cap law is armed INSIDE those guards, matching `_with_cap`'s position in the source.
        let cap_at = |law: &'static str, qq: f64, mf_app: Option<f64>| -> f64 {
            let _sb = MarchedBleed::set(&m.fuel.inner, qq);
            let _sv = MarchedStator::set(&m.fuel.inner, v);
            let _cs = CapScope::set(&m.fuel.inner, law);
            (m.fuel.inner.triple_hooks.cap_fuel)(
                &m.fuel, flight, a, h, ms, Some(&accel), None, mf_app)
                .unwrap_or_else(|e| panic!("{}", e.0))
        };
        let solve = |qq: f64| cap_at(CAP_LAW_SOLVE, qq, None);
        let sensed = |qq: f64, w: f64| cap_at(CAP_LAW_SENSED, qq, Some(w));
        let w0 = solve(q);
        let c = c_at(&m, flight, a, h, &accel, w0, q, v, C_AT_REL)
            .unwrap_or_else(|e| panic!("{}", e.0));
        let big_s = (sensed(q + dq, w0) - sensed(q - dq, w0)) / (2.0 * dq);
        let big_d = (solve(q + dq) - solve(q - dq)) / (2.0 * dq);
        rows.push(SolveGainRow {
            s: p.s,
            cap_solve: w0,
            c,
            fixed_point: (sensed(q, w0) - w0).abs(),
            d_s: big_s,
            d_d: big_d,
            // Python's `D / S if S != 0.0 else float("nan")` — a VALUE comparison against `+0.0`,
            // which `-0.0` also satisfies (`-0.0 != 0.0` is `False` in both languages), so a
            // `to_bits`-style test here would be stricter than the source.
            gain: if big_s != 0.0 { big_d / big_s } else { f64::NAN },
            predicted: 1.0 / (1.0 - c),
        });
    }
    SolveGain {
        phi_lim, margin, ref_law, inc,
        n: rows.len(),
        fixed_point: if rows.is_empty() { None } else { Some(py_max(&rows, |x| x.fixed_point)) },
        gain: if rows.is_empty() { None }
              else { Some((py_min(&rows, |x| x.gain), py_max(&rows, |x| x.gain))) },
        gain_err: if rows.is_empty() { None }
                  else { Some(py_max(&rows, |x| (x.gain - x.predicted).abs())) },
        rows,
    }
}
