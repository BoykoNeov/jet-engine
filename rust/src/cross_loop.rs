//! RUNG 67 — CASCADE A: two loops on TWO variables, and the ONE SCALAR that sets both faces.
//!
//! Slice Z, second of two. Python's `CrossLoopCascadeTransient` (`turbojet/engine.py`, **843
//! lines**, 15 methods) — rung 47's lagged `Tt4` topping GOVERNOR beside rung 65's lagged
//! `phi_lp` bleed VALVE. Four states, two clocks, and — unlike [`crate::two_lag`] — **two
//! different protected variables**.
//!
//! **IT IS RUNG 66's CONSTRUCTION WITH ONE SUBSTITUTION** — the fuel leg's SENSOR moves from
//! `phi_lp` to `Tt4` — and that single change inverts the algebra. With `P = R_q · C_g`, rung 66
//! has `P ≡ +1` identically, so `det J ≡ 0` and the discriminant is `tr²`: degenerate, provably
//! no oscillation at any clock ratio. Here nothing pins `P`, `det J = (1 − P)/(t_g t_v) ≠ 0`, and
//! **one scalar decides both faces** — it ends the degeneracy (so the pair buys AUTHORITY) and it
//! opens a ringing window, then damps it: **admissible, unobservable**.
//!
//! # `_eig`'s COMPLEX ARM IS DEAD ON THE RUNG THAT DEFINES IT AND LIVE ONLY HERE
//!
//! § 5.24 (vi). Rung 66 defines `_eig`; rung 67's `cross_identity` calls it too, and a census that
//! does not split by CALLING FUNCTION conflates them (the first one did, at 134 real / 57
//! complex). Split, over `test_rung66.py` alone: **80 of 80 real, and the complex arm never runs
//! on rung 66 at all** — which is not an accident but rung 66's own headline, since `det J ≡ 0`
//! makes the discriminant `tr² − 4·0 = tr² ≥ 0` identically. The arm is kept alive one rung up.
//! **A port that drops it passes every rung-66 gate and breaks here.**
//!
//! # THE TWO JOINT INITIAL CONDITIONS ARE A DELIBERATE DUPLICATION AND MUST NOT BE FACTORED
//!
//! § 5.24 (iv) / P6. Both marches need the two laws' simultaneous equilibrium at `s = 0`, and the
//! two rungs solve it **differently on purpose**:
//!
//! * **rung 66** iterates INLINE and UNDAMPED, capped at 60, and asserts on failure — its own
//!   identity pins the contraction factor `|R_q C_g|` at 1, so a stall there genuinely IS the
//!   degeneracy the rung is about, and the assert says so.
//! * **rung 67** calls `_joint_fixed_point`, which sweeps `w ∈ (1.0, 0.5, 0.25)` — here `|P|` is
//!   pinned by nothing and `det J ≠ 0` for every `P ≠ 1`, so a stall would be a SOLVER failure and
//!   reporting it as a marginal mode would be a FALSE FINDING.
//!
//! Giving rung 66 a call to this rung's damped solver is bit-exact on the shipped grid (probe 3:
//! `w = 1.0` on 36 of 39 calls) and destroys the distinction the two asserts are FOR. That is the
//! standing *copy vs re-derivation* rule and this is the reflex it exists to stop.
//!
//! # Step 1 ships the PLUMBING, the REFUSALS and [`RINGS`]; the march arrives at step 2

use crate::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{FuelLimiters, FuelPoint, FuelTransientCore, FuelTransientHooks};
use crate::lagged_bleed::lagged;
use crate::map::ComponentMap;
use crate::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                              StatorLeg, StatorTransientHooks};
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{LaggedGovernor, TwoSpoolTransientHooks};

/// RUNG 67's `_RINGS` — **how many sign changes in a sampled eigenvalue-imaginary-part trace
/// count as a RING**, and the one class attribute in either rung of this slice that a gate reads
/// directly (`tests/test_rung67.py:285`).
///
/// **A PLAIN `const`, AND THAT IS A MEASUREMENT RATHER THAN A DEFAULT.** Python reads it through
/// `cls._RINGS` and `self._RINGS` — the dynamic-lookup spelling, which is what an OVERRIDABLE
/// constant looks like, so the port owes the question *does any rung above 67 rebind it?* before
/// choosing between an associated const and a table cell. Grepped over the whole 23 066-line
/// `engine.py` and over all 27 phase-7 suites: **one definition, two reads, both inside rung 67's
/// own readers, and no rebinding at any rung through 84.** So a module const is faithful, and if
/// a later rung ever rebinds it this comment is where the port is wrong.
///
/// The two reads are `detector_sensitivity` (`n >= cls._RINGS`) and `oscillation_window`
/// (`max(nq, ng) >= self._RINGS`) — both arrive at step 2.
pub const RINGS: usize = 2;

// ---------------------------------------------------------------------------------------------
// THE CELLS — the SAME three swaps as rung 66, and again zero additions
// ---------------------------------------------------------------------------------------------

/// RUNG 67's `_stator_march` — rung 66's march with ONE addition, `tau_gov`.
///
/// The governor's clock rides on the carrier for rung 65/66's reason verbatim: a dozen
/// rung-57-to-66 readers call this cell knowing nothing about it, and every one must keep
/// reaching the IDENTICAL march. `tau_gov = None` leaves them all bit-for-bit.
///
/// **THE REDLINE NEEDS NO PLUMBING OF ITS OWN.** `Tt4_max` has been a rung-58 [`StatorLeg`] field
/// since slice V, so only the CLOCK is new here — which is why this rung adds one scope field and
/// not two.
///
/// The guard is [`LaggedGovernor`], restore-PREVIOUS, and the two questions (*does it restore the
/// previous value*, *does it ever nest*) were asked separately from rung 66's rather than
/// inherited from it — a carrier claim on ONE hook says nothing about the next. Both answers came
/// back the same: restore-previous, max depth **1**, **0** nested events.
pub fn r67_stator_march(
    ft: &FuelTransientCore, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
    leg: &StatorLeg<'_>, scope: &MarchScope,
) -> (Vec<FuelPoint>, (f64, f64)) {
    let _g = LaggedGovernor::set(&ft.inner, scope.tau_gov);
    // Python forwards `lag=lag` and drops `tau_gov` — this rung consumes exactly one field and
    // passes the other two on, as rung 66 passes `b0` on.
    crate::two_lag::r66_stator_march(
        ft, flight, ramp, nu0, leg,
        &MarchScope { b0: scope.b0, lag: scope.lag, ..MarchScope::DEFAULT })
}

/// RUNG 67's `at_lever` — **the SIXTH instance of the sibling-constructor trap** rungs
/// 61/62/63/64/65/66 each hit: the inherited constructor hardcodes its own class name, so a
/// rung-67 machine would silently hand back a rung-66 one.
///
/// The governor clock is a per-MARCH argument, not a machine keyword, so there is nothing here
/// for it to drop.
pub fn r67_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_cross_loop_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 67's `integrate_fuel` — the dispatch and **four** refusals (rung 66 has three).
///
/// | reduce arm | condition | lands on |
/// |---|---|---|
/// | rung 66 | `tau_gov is None`, `lag` set | rung 66's merged march |
/// | rung 65 | `tau_gov is None`, `lag is None` | `r65_integrate_fuel_valve_lag` |
/// | rung 47 | `_lagged()` false | `integrate_fuel_lagged`, untouched |
///
/// Probe 3 measured all four `(armed, arg)` arms of this dispatch LIVE (5 / 6 / 8 / 38).
///
/// **THE `Tt4_max` REFUSAL IS THE ONE THAT MAKES A PLACEMENT CHOICE TESTABLE.** Rung 66 recorded
/// an ambiguity and dodged it — rung 52 min-selects the redline UNLAGGED on top of already-clipped
/// fuel, rung 65 puts it inside the caps at `mf_sched`, and cascade B never armed it. Here the
/// redline IS the lagged leg, so it is carried BY the state exactly as rung 47 carries it, and a
/// wrong pick shows up as a diff against rung 47 itself.
pub fn r67_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    // Python's `tau_gov = tau_gov if tau_gov is not None else self._tau_gov` — the ARGUMENT wins,
    // and the RESOLVED value is what `super()` receives. `lag` is forwarded RAW: rung 66's cell
    // resolves it against its OWN carrier, which is why this rung must not resolve it here.
    let tau_gov = lim.tau_gov.or_else(|| ft.inner.tau_gov.get());
    if !(lagged(&ft.inner) && tau_gov.is_some()) {
        return crate::two_lag::r66_integrate_fuel(
            ft, flight, fuel_schedule, nu0, s_end, ds,
            &FuelLimiters { tau_gov, ..lim.clone() });
    }
    assert!(lim.tt4_max.is_some(),
            "rung-67: `tau_gov` is the GOVERNOR's clock and a governor needs a redline to \
             lag (rung 47's own assert, one cascade up). Without `Tt4_max` the fuel state has \
             nothing to run on and the cascade would silently reduce to rung 65 while \
             claiming four states.");
    assert!(lim.lag.is_none(),
            "rung-67 is CASCADE A: rung 47's Tt4 governor beside rung 65's phi valve -- two \
             loops on TWO variables. Rung 52's AsymmetricLag over rung 49's phi floor is \
             CASCADE B, which is rung 66 and reached by leaving `tau_gov` None. Running both \
             fuel legs at once is THREE loops on two variables -- a separate rung (rung 67's \
             own next seam), asserted against rather than run.");
    assert!(lim.accel.is_none() && lim.floor().is_none(),
            "rung-67 arms the GOVERNOR as its fuel leg. A second fuel-side leg (rung 48's \
             accel schedule, rung 49's phi floor) makes it three loops and, for `surge`, puts \
             a SECOND loop back on `phi_lp` -- which would superpose rung 66's identity onto \
             this rung's window and measure neither cleanly. One rung, one headline.");
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-67: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger. Both legs here pin their own (rung 47's \
             governor rides its own signal, rung 65's valve its own), so forcing one would \
             measure the forcing.");
    r67_integrate_fuel_cross(
        ft, flight, fuel_schedule, nu0, s_end, ds, lim,
        lim.tt4_max.expect("the assert above proves this is Some"),
        tau_gov.expect("the dispatch above proves this is Some"))
}

/// RUNG 67's MARCH — **SLICE Z STEP 2**, and the second of the slice's two stubs.
///
/// Rung 47's `_integrate_fuel_lagged` and rung 65's `_integrate_fuel_valve_lag`, merged: four
/// states, the two actuators coupled ONLY through the plant, and the two laws watching DIFFERENT
/// variables.
///
/// **THE `_b_state` BOUNDARY IS LOAD-BEARING HERE IN A WAY IT WAS NOT ON CASCADE B.** `R_q ≠ 0`
/// only because the governor senses `Tt4` on the machine AS THE VALVE ACTUALLY IS. Drop the state
/// around `required` and `R_q ≡ 0`, the rung silently becomes two INDEPENDENT loops with
/// `det J = 1/(t_g t_v)`, no complex branch anywhere — **and nothing fails.** `cross_identity`
/// measures `R_q ≠ 0` as a gate for exactly that reason.
#[allow(clippy::too_many_arguments)]
pub fn r67_integrate_fuel_cross(
    _ft: &FuelTransientCore, _flight: &FlightCondition, _fuel_schedule: &dyn Fn(f64) -> f64,
    _nu0: (f64, f64), _s_end: f64, _ds: f64, _lim: &FuelLimiters<'_>,
    _tt4_max: f64, _tau_gov: f64,
) -> Vec<FuelPoint> {
    unimplemented!(
        "rung-67's cross-loop four-state march is SLICE Z STEP 2. Step 1 ships the plumbing (the \
         MarchScope fields, the two carriers, the three cell swaps, _RINGS) and the refusals, so \
         that the three reduce arms -- rung 66, rung 65 and rung 47, all by DISPATCH -- are \
         gated before the march exists. Reaching this means a caller armed BOTH clocks.")
}

// ---------------------------------------------------------------------------------------------
// THE CONSTRUCTOR
// ---------------------------------------------------------------------------------------------

/// RUNG 67's object — like [`build_two_lag_cascade`], **rung 65's constructor with the TABLES
/// swapped and nothing else**, and for the same measured reason: `CrossLoopCascadeTransient`
/// defines no `__init__` and rebinds no class attribute except `_tau_gov` (the carrier) and
/// [`RINGS`] (a reader's threshold). None of rung 65's ten construction asserts moves.
///
/// [`build_two_lag_cascade`]: crate::two_lag::build_two_lag_cascade
pub fn build_cross_loop_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = ScheduledStatorTransient::with_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        &R67_TWO, &R67_STATOR, &R67_FUEL, &R67,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim });
    // Rung 62's two, in Python's order.
    assert!(!(arm.bleed != 0.0 && arm.bleed_sched.is_some()),
            "rung-62: the valve gets a CONSTANT position or a SCHEDULE, not both -- they are the \
             two legs the rung differences (rung 57's discipline).");
    assert!((0.0..0.5).contains(&arm.bleed),
            "rung-42 bleed fraction must be in [0, 0.5): b>=0.5 starves the core and the choked \
             branch is long gone by then");
    // Rung 64's three-way arming exclusion.
    assert!(!(arm.bleed_lim.is_some() && (arm.bleed != 0.0 || arm.bleed_sched.is_some())),
            "rung-64: the valve gets a CONSTANT position (42), a SCHEDULE (62) or a FLOOR (64) -- \
             exactly one. They are the three legs this rung differences, and rung 62's two-way \
             assert is extended rather than replaced.");
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — the same three cells, one rung further up
// ---------------------------------------------------------------------------------------------

/// RUNG 67's lever table — ONE cell, `at_lever`.
pub const R67: LeverHooks = LeverHooks {
    at_lever: r67_at_lever,
    ..crate::two_lag::R66
};

/// RUNG 67's `TwoSpoolTransientHooks` — **ZERO cells swapped**, named for [`R66_TWO`]'s reason.
///
/// [`R66_TWO`]: crate::two_lag::R66_TWO
pub const R67_TWO: TwoSpoolTransientHooks = crate::two_lag::R66_TWO;

/// RUNG 67's fuel table — ONE cell, `integrate_fuel`.
pub const R67_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r67_integrate_fuel,
    ..crate::two_lag::R66_FUEL
};

/// RUNG 67's stator table — ONE cell, the march that carries `tau_gov`.
pub const R67_STATOR: StatorTransientHooks = StatorTransientHooks {
    stator_march: r67_stator_march,
    ..crate::two_lag::R66_STATOR
};
