//! RUNG 66 — the TWO-LAG CASCADE: what a SECOND limiter buys, and what it provably cannot.
//!
//! Slice Z, first of two. Python's `TwoLagCascadeTransient` (`turbojet/engine.py`, **653 lines**,
//! 11 methods), ported onto [`crate::lagged_bleed`]'s shape: a lagged bleed VALVE beside a lagged
//! FUEL leg, **both watching `phi_lp`**. Four states, two clocks, and no new control law — rung
//! 52's `AsymmetricLag` over rung 49's floor, merged with rung 65's valve.
//!
//! **HEADLINE (the rung's): two loops on one variable are ONE loop with the RATES ADDED.** Both
//! laws are implicit functions of the same constraint `phi(w, b) = phi_lim`, so their cross-gains
//! are reciprocals *by construction* — `R_q · C_g ≡ 1` is an IDENTITY, not a locus, hence
//! `det J ≡ 0`. The pair has ONE effective actuator direction, so the credits cannot add: 60.46 %
//! and 92.51 % alone deliver **94.09 %** together. A second limiter — its own sensor, law,
//! actuator and clock — buys BANDWIDTH, not AUTHORITY.
//!
//! # What slice Z adds to the table
//!
//! **ZERO new cells** — probe 1's emitted census, and the THIRD row of § 5.19 (x)'s cell column
//! an emitter confirms (after X's 1 and Y's 0). Rungs 66 and 67 swap the **same three**
//! already-open cells: `stator_march` (opened at slice V), `at_lever` (slice W) and
//! `integrate_fuel` (slice Y). Nothing here re-opens a signature.
//!
//! # Step 1 ships the PLUMBING and the REFUSALS; the march arrives at step 2
//!
//! [`r66_integrate_fuel`]'s dispatch and its three asserts are complete and live — which means
//! the reduce arms are gate-able **before a single line of the march exists**, and that is the
//! point of splitting the step: rung 66 reduces to rung 64 (`_lagged()` false), to rung 65
//! (`lag is None`) and to rung 52, all three BY DISPATCH, and all three route through code that
//! already ships. [`r66_integrate_fuel_cascade`] is the only stub.
//!
//! # The precedence no value key can see
//!
//! Python opens with `lag = lag if lag is not None else self._lag` — **the ARGUMENT wins over the
//! carrier**, and the RESOLVED value is what gets forwarded to `super()`. On every shipped grid
//! at most one of the two is ever set, so a port that reads the carrier first, or that forwards
//! the raw argument instead of the resolved one, agrees everywhere. Both are spelled explicitly
//! below for that reason.

use crate::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{FuelLimiters, FuelPoint, FuelTransientCore, FuelTransientHooks};
use crate::lagged_bleed::lagged;
use crate::map::ComponentMap;
use crate::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                              StatorLeg, StatorTransientHooks};
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{LaggedFuel, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// THE CELLS — three swaps, zero additions
// ---------------------------------------------------------------------------------------------

/// RUNG 66's `_stator_march` — rung 65's march with ONE addition, `lag`.
///
/// **THE FUEL LAG IS A PER-MARCH ARGUMENT AND NOT A MACHINE KEYWORD**, which is rung 65's `b0`
/// discipline verbatim and the reason [`r66_at_lever`] has nothing to drop: *a sibling
/// constructor cannot drop what it never carries.*
///
/// **THE GUARD RESTORES THE PREVIOUS VALUE** ([`LaggedFuel`], not a restore-to-`None`) — Python is
/// `prev, self._lag = self._lag, lag` … `finally: self._lag = prev`. Probe 3 measured max nesting
/// depth **1** with **0** nested events over rungs 62–67, so the difference is invisible to every
/// value key and step 5 manufactures the nest (§ 5.24 P7).
///
/// `lag = None` is a real assignment: a rung-66 march called WITHOUT it CLEARS an outer one for
/// the duration.
pub fn r66_stator_march(
    ft: &FuelTransientCore, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
    leg: &StatorLeg<'_>, scope: &MarchScope,
) -> (Vec<FuelPoint>, (f64, f64)) {
    let _g = LaggedFuel::set(&ft.inner, scope.lag);
    // Python forwards to `super()._stator_march(...)` WITHOUT `lag` — the parameter is CONSUMED
    // here and `b0` is passed on. Each rung consumes exactly one field and forwards the rest,
    // which is why the scope is rebuilt rather than passed through.
    crate::lagged_bleed::r65_stator_march(
        ft, flight, ramp, nu0, leg, &MarchScope { b0: scope.b0, ..MarchScope::DEFAULT })
}

/// RUNG 66's `at_lever` — rung 65's sibling constructor returning THIS class.
///
/// **THE FIFTH INSTANCE OF ONE TRAP, AND THE OVERRIDE IS ONE WORD.** Rungs 61/62/63/64/65 each
/// hit it: the inherited constructor hardcodes its own class name, so a rung-66 machine calling
/// rung 65's `at_lever` would silently hand back a rung-65 one. In Rust that word is which
/// BUILDER is called, so deleting this override is caught by the tables the sibling carries
/// rather than by any value key on the machine it returns.
///
/// The fuel lag is a per-MARCH argument, so — exactly as at rung 65 — there is no lag keyword
/// here for a sibling constructor to drop.
pub fn r66_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_two_lag_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 66's `integrate_fuel` — **the dispatch, and the three refusals that keep the rung from
/// silently becoming a different one.**
///
/// # The three reduce arms all leave through the SAME `if`
///
/// The merged integrator is entered only when BOTH clocks are actually armed, which is what makes
/// every arm bit-for-bit **by dispatch** rather than by numerical agreement:
///
/// | arm | condition | lands on |
/// |---|---|---|
/// | rung 64 | `_lagged()` false, no `lag` | rung 43's march, through rung 65's cell |
/// | rung 65 | `lag is None` | `r65_integrate_fuel_valve_lag` |
/// | rung 52 | `_lagged()` false, `lag` set | `integrate_fuel_asym` |
///
/// Probe 3 measured **all four** `(armed, arg)` arms of this dispatch LIVE (18 / 4 / 12 / 32), so
/// unlike slice Y's mirror-zero pair no arm here needs a manufactured gate — a non-recurrence
/// registered by MEASUREMENT rather than assumed away, because the previous slice's headline is
/// exactly what a port is tempted to carry forward on a family resemblance.
pub fn r66_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    // Python's `lag = lag if lag is not None else self._lag` — THE ARGUMENT WINS, and the
    // RESOLVED value is what `super()` receives. Both halves matter and neither is visible in a
    // float on any shipped grid (module doc).
    let lag = lim.lag.or_else(|| ft.inner.lag.get());
    if !(lagged(&ft.inner) && lag.is_some()) {
        return crate::lagged_bleed::r65_integrate_fuel(
            ft, flight, fuel_schedule, nu0, s_end, ds,
            &FuelLimiters { lag, ..lim.clone() });
    }
    assert!(lim.tau_gov.is_none(),
            "rung-66 takes CASCADE B: rung 52's phi-referenced fuel lag beside rung 65's phi \
             valve -- two loops on ONE variable, which is what rung 65 s 3's marginal mode is \
             about. Rung 47's tau_gov watches Tt4, a DIFFERENT variable, so that pairing \
             (cascade A) tests rung 52 s 3's non-additivity instead. Its cross-gains have \
             OPPOSITE signs and it therefore admits an oscillatory mode this one provably \
             cannot -- a separate rung, asserted against rather than run.");
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-66: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger. BOTH legs here pin their own (rung 52's \
             argument on the fuel side, rung 65's on the valve), so forcing one would measure \
             the forcing. `lag.tau_rel` -- the RATE the fuel leg hands its clip back at -- is \
             a different object and is exactly what this rung sweeps.");
    assert!(lim.accel.is_some() || lim.floor().is_some(),
            "rung-66's fuel lag lags a min-select LEG's clip -- arm one (accel/surge). With \
             neither armed `required == 0` identically and the fuel clock has nothing to run \
             on, which would silently reduce the cascade to rung 65 while claiming four \
             states.");
    r66_integrate_fuel_cascade(ft, flight, fuel_schedule, nu0, s_end, ds, lim,
                               lag.expect("the dispatch above proves this is Some"))
}

/// RUNG 66's MARCH — **SLICE Z STEP 2**, and the only stub step 1 leaves.
///
/// Rung 52's `_integrate_fuel_asym` and rung 65's `_integrate_fuel_valve_lag`, merged: four
/// states, and the two actuators coupled ONLY through the plant. `g`/`required` (rung 52's keys)
/// and `b`/`b_cmd` (rung 65's) are ALL recorded per point, so both tracking errors read straight
/// off one trajectory and every rung-52 and rung-65 reader works unchanged on it.
///
/// It is a stub rather than an omission so that [`r66_integrate_fuel`]'s dispatch and its three
/// asserts ship LIVE at step 1 — the reduce arms are then gated before the march exists, which is
/// a real gate rather than a compile check.
#[allow(clippy::too_many_arguments)]
pub fn r66_integrate_fuel_cascade(
    _ft: &FuelTransientCore, _flight: &FlightCondition, _fuel_schedule: &dyn Fn(f64) -> f64,
    _nu0: (f64, f64), _s_end: f64, _ds: f64, _lim: &FuelLimiters<'_>,
    _lag: crate::fuel_transient::AsymmetricLag,
) -> Vec<FuelPoint> {
    unimplemented!(
        "rung-66's merged four-state march is SLICE Z STEP 2. Step 1 ships the plumbing (the \
         MarchScope fields, the two carriers, the three cell swaps) and the refusals, so that \
         the three reduce arms -- rung 64, rung 65 and rung 52, all by DISPATCH -- are gated \
         before the march exists. Reaching this means a caller armed BOTH clocks.")
}

// ---------------------------------------------------------------------------------------------
// THE CONSTRUCTOR
// ---------------------------------------------------------------------------------------------

/// RUNG 66's object — **rung 65's constructor with the TABLES swapped and NOTHING else, and that
/// is a measurement.**
///
/// `TwoLagCascadeTransient` defines no `__init__` and rebinds no class attribute except `_lag`
/// (the carrier, not a constructor knob), so **none of rung 65's ten construction asserts is
/// added to, relaxed or re-ordered here** — `_LAG_OK` in particular stays `True` by inheritance.
/// The difference between this builder and `build_lagged_bleed` is the four table constants it
/// passes, and saying so explicitly is the point: a reader who finds two builders differing by
/// four words should be told that the SAMENESS was measured and is the finding.
pub fn build_two_lag_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = ScheduledStatorTransient::with_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        &R66_TWO, &R66_STATOR, &R66_FUEL, &R66,
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
// THE TABLES — three cells swapped across three tables, and ZERO across the fourth
// ---------------------------------------------------------------------------------------------

/// RUNG 66's lever table — ONE cell, `at_lever`. `b_of` / `b_at_point` / `armed_bleed` /
/// `isolating` / `legs` are rung 64's and rung 65's, inherited.
pub const R66: LeverHooks = LeverHooks {
    at_lever: r66_at_lever,
    ..crate::lagged_bleed::R65
};

/// RUNG 66's `TwoSpoolTransientHooks` — **ZERO cells swapped.**
///
/// **AN ALIAS, NOT A RE-ENUMERATION, AND THAT IS THE FAITHFUL SPELLING.** Rung 66 subclasses rung
/// 65, so a change to any rung-65 cell propagates to rung 66 BY INHERITANCE; an alias reproduces
/// exactly that, where a hand-enumerated literal would freeze rung 65's current bodies into rung
/// 66 and silently stop tracking it.
///
/// It is NAMED rather than left implicit at the builder so the cell census reads off the table
/// list instead of off an absence. Note what that does and does not buy: it makes the ZERO
/// legible, and it does **not** make a future addition to `R65_TWO` loud here — nothing should,
/// because propagating is what Python does.
pub const R66_TWO: TwoSpoolTransientHooks = crate::lagged_bleed::R65_TWO;

/// RUNG 66's fuel table — ONE cell, `integrate_fuel`. `try_close_fuel` is rung 65's.
pub const R66_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r66_integrate_fuel,
    ..crate::lagged_bleed::R65_FUEL
};

/// RUNG 66's stator table — ONE cell, the march that carries `lag`.
pub const R66_STATOR: StatorTransientHooks = StatorTransientHooks {
    stator_march: r66_stator_march,
    ..crate::lagged_bleed::R65_STATOR
};
