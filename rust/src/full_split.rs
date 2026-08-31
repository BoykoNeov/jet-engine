//! RUNG 71 — **THE FULL SPLIT**: rung 69's INCIDENCE stator beside rung 70's governor and valve.
//!
//! `n = m = 3`, **ZERO zeros** — the last unoccupied cell of rung 69's table, and rung 70's own
//! strongest seam. Five states, three clocks, one actuator per loop, and now THREE constraints:
//! the governor on `Tt4`, the valve on `phi`, the stator on `M_i`.
//!
//! Headline: *a constraint can be INDEPENDENT in RANK and REDUNDANT on the BAND* — `det J` is
//! non-zero for the first time in the family and FACTORS into rung 67's and rung 69's own
//! non-degeneracy conditions, one per rung, while the third loop rides over only 7.9 % of the
//! march because the valve's wall CONTAINS the stator's on the whole admissible band. So
//! `zeros = n - m` counts GRADIENT DIRECTIONS, not LIVE loops. See `docs/rung71-spec.md`.
//!
//! # What this module is — and what STEPS 1 and 3 ship
//!
//! [`crate::cross_split`]'s note applies unchanged and is not repeated: **there is no
//! `FullSplitCore`**, one type carries rungs 57–84, and `FullSplitTransient` defines no `__init__`
//! in Python either — so this module is the second of the slice's two "cores" in the only sense
//! the architecture has one.
//!
//! Step 1 shipped [`build_full_split_cascade`], the five `R71*` tables, and rung 71's **two
//! swapped cells opened as NAMED PANICS**. **Step 3 fills them** and adds the rung's other three
//! plain methods ([`full_rig`], [`r71_rk4_floor_full`], [`zeta_ring`]) and its **six readers**.
//! The carrier and guard `_gov_max` needs are rung 70's and live in [`crate::cross_split`],
//! because `_full_rig` writes the same field `_split_rig` does.
//!
//! # THE TABLE ARITHMETIC — this rung's half
//!
//! * **ZERO cells added** (§ 5.27 (i)), and rung 71 overrides **none** of rung 68's nine nor rung
//!   69's one — so `R71_TRIPLE` is the whole of `R70_TRIPLE`, spelled out.
//! * **TWO swaps**, `at_lever` and `integrate_fuel`, both against RUNG 70's bodies and not rung
//!   69's. That matters for the gate: the pointer this rung must differ from is `R70`'s, and a
//!   swap that reached back past rung 70 would be a different defect that a rung-69 comparison
//!   would miss.
//! * **FIVE table consts**, the second half of the slice's ten. See [`crate::cross_split`] for the
//!   reconciled count and why it is not the pre-flight's "nine".
//!
//! # THE TWO SWAPS' MEASURED BREAK SHAPES (§ 5.27 (v))
//!
//! * `at_lever` ← rung 70's — **PANIC + VALUE**, seen by 3 of 6 readers, and observable only
//!   because the parent's `integrate_fuel` then refuses the arming with rung 70's *"THREE loops on
//!   TWO variables"*. Its dispatch gate therefore cannot be written before the arming asserts
//!   land; booked to step 7, exactly as rung 70's is.
//! * `integrate_fuel` ← rung 70's — **PANIC**, seen by 2 of 6. Rung 71 has FOUR asserts where rung
//!   70 has five: the guard rung 70 spends on refusing an incidence stator is the guard this rung
//!   REMOVES, which is the whole rung. Probe 2 classifies the swap as RESTRUCTURED, 11 → 11
//!   comparisons with the guard set MOVED.
//!
//! # THE TWO FLOORS ARE THE SAME CALL SERVING TWO RUNGS — **do not hoist**
//!
//! Registered at step 1 because it is a design constraint on step 3, not a discovery to be made
//! during it. Rung 71's `integrate_fuel` calls `_rk4_floor_full` and then delegates to rung 70's
//! `_integrate_fuel_cross_triple`, which calls `_rk4_floor_split` **on the same condition with the
//! same rate** — all three floors in the family assert the character-identical `ds * rate <= 2.0`
//! and differ only in the MESSAGE. Probe 12 removed the shadowed call: the rung-71 trajectory is
//! IDENTICAL (341 points, 3 410 keys) and the rung-71 guard still fires — **and the rung-70 guard
//! is GONE.** The shadowed call is not a redundant copy; it is the only floor on the rung-70 arm.
//! A port that hoists one floor into `integrate_fuel` silently deletes rung 70's guard while every
//! rung-71 gate stays green. **Step 3 kept both calls**; [`r71_integrate_fuel`] is where.
//!
//! None of the three floors is a cell (each is defined once, under its own name), so no function
//! pointer exists for a floor and no dispatch gate can substitute one — P5. Each is gated by a
//! `should_panic` on its RUNG TAG, and the tag is the only match string that is unique to one
//! floor: `RK4 stability region` matches all three and `rank TWO` matches two (rungs 69 and 70).

use std::cell::Cell;

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::cross_loop::exceed;
use crate::cross_split::{
    assert_state_boundary, r70_integrate_fuel_cross_triple, split_rig, Span, StateBoundary,
};
use crate::engine::FlightCondition;
use crate::fuel_transient::{FuelLimiters, FuelPoint, FuelTransientCore, FuelTransientHooks};
use crate::gas::powp;
use crate::lagged_bleed::{lagged, py_max3};
use crate::limited_bleed::{BleedLimiter, Regime};
use crate::map::ComponentMap;
use crate::reference_split::{
    build_split_family_cascade, cubic_roots_c, invariants, opt_fold, StatorIncidenceLimiter, C64,
};
use crate::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
    StatorTransientHooks,
};
use crate::three_loop::{
    riding, triple_gains_at, v_at_point, violation_inc, TripleGains, TripleHooks,
};
use crate::two_lag::violation;
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::TwoSpoolTransientHooks;

// ---------------------------------------------------------------------------------------------
// THE BUILDER — rung 69's constructor, INHERITED THROUGH RUNG 70
// ---------------------------------------------------------------------------------------------

/// Rung 71's constructor: **rung 69's, verbatim, with rung 71's five tables.**
///
/// Neither rung 70 nor rung 71 defines `__init__`, so the guard sequence is rung 69's eleven and
/// this rung adds none — [`build_cross_split_cascade`]'s note, one rung up.
///
/// **AND ONE OF THOSE ELEVEN IS WHY THIS RUNG EXISTS AT ALL.** Rung 69's guard D refuses an
/// incidence floor that is not the valve's `phi` floor at the design setting — *one PHYSICAL
/// wall*. That match is what makes rung 71's containment result true (`M_i = m_lim + v` at
/// `phi = phi_lim` holds BECAUSE `m_lim = T_c - 1/phi_lim` exactly), so the rung's headline is
/// contingent on a constructor assert two rungs below it. Tighten the incidence wall by `delta`
/// and the containment fails for `v < delta`; the guard is what stops that arming from being
/// built.
///
/// [`build_cross_split_cascade`]: crate::cross_split::build_cross_split_cascade
pub fn build_full_split_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    build_split_family_cascade(design_engine, flight_design, mdot_design, map_lp, map_hp, rho,
                               arm, &R71_TWO, &R71_STATOR, &R71_FUEL, &R71, &R71_TRIPLE)
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and TWO of them carry a swap of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 71's lever table — ONE swap, `at_lever`, and the parent it must differ from is **rung
/// 70's**.
///
/// The TENTH instance of the sibling-constructor trap. As at rung 70 the signature does not grow:
/// rung 71 changes which CONSTRAINT the stator watches, and that is armed by `stator_inc` — a
/// keyword rung 69 already added.
pub const R71: LeverHooks = LeverHooks {
    at_lever: r71_at_lever,
    ..crate::cross_split::R70
};

/// RUNG 71's `TwoSpoolTransientHooks` — **ZERO cells swapped**, named for `R66_TWO`'s reason.
pub const R71_TWO: TwoSpoolTransientHooks = crate::cross_split::R70_TWO;

/// RUNG 71's fuel table — ONE swap, `integrate_fuel`: **four arming asserts where rung 70 has
/// five**, then `_rk4_floor_full` and a delegation into rung 70's marcher.
pub const R71_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r71_integrate_fuel,
    ..crate::cross_split::R70_FUEL
};

/// RUNG 71's stator table — **ZERO cells swapped**, named for the same reason.
pub const R71_STATOR: StatorTransientHooks = crate::cross_split::R70_STATOR;

/// RUNG 71's third-loop table — **ZERO of the ten cells swapped.**
///
/// Measured, not assumed: § 5.27 (iii) ran the census per rung and rung 71 overrides none of rung
/// 68's nine and none of rung 69's one. The whole of rung 70's table is inherited, and it is
/// spelled out here rather than reached through a `..R70_TRIPLE` spread for `R69_TRIPLE`'s reason
/// — a spread would make the NEXT addition to that table silent at this rung.
pub const R71_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::cross_split::R70_TRIPLE.stator_leg,
    lagged_stator: crate::cross_split::R70_TRIPLE.lagged_stator,
    clamp_v: crate::cross_split::R70_TRIPLE.clamp_v,
    check_v0: crate::cross_split::R70_TRIPLE.check_v0,
    rk4_floor: crate::cross_split::R70_TRIPLE.rk4_floor,
    solve_v: crate::cross_split::R70_TRIPLE.solve_v,
    manifold_v: crate::cross_split::R70_TRIPLE.manifold_v,
    triple_laws: crate::cross_split::R70_TRIPLE.triple_laws,
    triple_rig: crate::cross_split::R70_TRIPLE.triple_rig,
    with_ref: crate::cross_split::R70_TRIPLE.with_ref,
};

// ---------------------------------------------------------------------------------------------
// THE TWO SWAPPED CELLS — BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 71's `at_lever` — **rung 70's sibling constructor returning a RUNG-71 machine**, and the
/// TENTH instance of the trap rungs 61–70 each hit.
///
/// The signature does not grow: rung 71 arms its third loop with the SAME `stator_inc` keyword
/// rung 69 added, so the failure mode is rung 70's plain one — hand back the parent's class and
/// every reader measures rung 70's plant (a `phi` stator, `m = 2`) while reporting rung 71's.
///
/// **WHAT MAKES THAT OBSERVABLE IS NOT THIS FUNCTION**, exactly as at rung 70: this returns a
/// table pointer, and the refusal that surfaces the swap is the PARENT's `r70_integrate_fuel`
/// guard A — *"an INCIDENCE stator here would put all three on DIFFERENT constraints"* — which is
/// rung 70's own way of saying *this is rung 71's cell*. § 5.27 (v) measured the shape as
/// **PANIC + VALUE**, seen by 3 of the 6 readers.
///
/// It routes through [`build_full_split_cascade`], so every sibling re-asserts rung 69's eleven
/// guards.
fn r71_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_full_split_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 71's `_rk4_floor_full` — **the floor, re-justified a FOURTH time on a THIRD argument**,
/// which is the pattern and not an oversight.
///
/// Rung 68's `ds*sum(1/tau_i) <= 2` is exact-in-argument there (`J` rank one, non-zero eigenvalue
/// EXACTLY `-sum 1/tau_i`); rung 69 kept the constant on a complex pair; rung 70 kept it because
/// `min(pair) ~ 0` put the pair back on the real axis. **Here NEITHER argument applies**: there is
/// no zero root at all, so the trace is shared THREE ways and the dominant root is strictly
/// smaller in magnitude than the rate sum. [`full_modes`] MEASURES `|lam|` against it rather than
/// trusting it — rung 65 published a retraction for exactly the failure mode of a trusted
/// stability argument.
///
/// # THIS IS NOT THE `rk4_floor` CELL EITHER — P5, in its second instance
///
/// Three arguments, a plain `@staticmethod` in Python, defined once and overridden nowhere, so no
/// function pointer exists for it and nothing in the dispatch harness can substitute one. Its only
/// gate is a `should_panic` on its RUNG TAG: probe 2b measured `RK4 stability region` matching all
/// THREE floors in the family and `rank TWO` matching two of them, so `rung-71: ds` is the one
/// string unique to this one.
pub fn r71_rk4_floor_full(ds: f64, rate: f64, tau_s: f64) {
    assert!(ds * rate <= 2.0,
            "rung-71: ds*sum(1/tau_i) = {:.3} is outside the explicit RK4 stability region for \
             the three actuator states (ds = {ds}, tau_s = {tau_s}). At FULL RANK there is no \
             zero eigenvalue to absorb the trace, so all three roots share it and the dominant \
             one is BELOW the rate sum -- the inherited constant stays conservative, for a third \
             reason. Refine the grid or slow a clock.", ds * rate);
}

/// RUNG 71's `integrate_fuel` — **the reduce arm, then FOUR arming refusals, then rung 70's
/// march.**
///
/// # THE REDUCE IS THE MIRROR IMAGE OF RUNG 70's GUARD A, ONE CALL APART
///
/// Rung 70 *asserts* `stator_inc is None`; this rung *reduces* on it. The test
/// `tau_gov is None or stator_inc is None or not lagged_stator()` covers every inherited arm —
/// rung 70 (a `phi` stator beside the governor), rung 69 (an incidence stator, no governor), rung
/// 68, rung 67 and everything under them. **This class never intercepts a march it does not own**,
/// which is what keeps P9's four reduce arms bit-for-bit.
///
/// # FOUR ASSERTS WHERE RUNG 70 HAS FIVE, AND THE MISSING ONE IS THE WHOLE RUNG
///
/// The guard rung 70 spends on refusing an incidence stator is the guard this rung REMOVES. Probe
/// 2 classifies the swap as RESTRUCTURED — 11 → 11 comparisons with the guard set MOVED — which is
/// why the swap's break shape is a PANIC (rung 70's body, installed here, refuses the very arming
/// this rung is built for) and not a value.
///
/// # BOTH FLOORS FIRE, AND THE SHADOWED ONE IS RUNG 70's ONLY GUARD
///
/// This body calls [`r71_rk4_floor_full`] and then delegates to
/// [`r70_integrate_fuel_cross_triple`], which calls `_rk4_floor_split` **on the same condition
/// with the same rate**. Probe 12 removed the shadowed call: the rung-71 trajectory is IDENTICAL
/// (341 points, 3 410 keys) and the rung-71 guard still fires — **and the rung-70 guard is GONE.**
/// See this module's header note; hoisting is the defect every rung-71 gate stays green through.
///
/// `tau_gov` is read from the CARRIER as well as the argument for rung 70's reason: rung 67's
/// clock rides on an instance attribute and `_stator_march` does not forward it as a keyword, so
/// reading only the argument would let a rung-71 march silently become a rung-69 one.
fn r71_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    let lag = lim.lag.or_else(|| ft.inner.lag.get());
    let tau_gov = lim.tau_gov.or_else(|| ft.inner.tau_gov.get());
    if tau_gov.is_none() || ft.inner.stator.inc.is_none() || !ft.inner.lagged_stator() {
        bump(&INTEGRATE71_REDUCED);
        // EVERY inherited arm leaves through here, and through the IMMEDIATE parent's table:
        // `super()` from this class is rung 70, and a grandparent spelling would call a slot that
        // is only ACCIDENTALLY the same pointer today.
        return (crate::cross_split::R70_FUEL.integrate_fuel)(
            ft, flight, fuel_schedule, nu0, s_end, ds,
            &FuelLimiters { tau_gov, lag, ..lim.clone() });
    }
    // GUARD A — rung 70's B.
    let tt4_max = lim.tt4_max;
    assert!(tt4_max.is_some(),
            "rung-71's odd loop IS the redline: `tau_gov` without `Tt4_max` is a governor with \
             no set point, which would march as rung 69 while every reader reported rung 71.");
    // GUARD B — rung 70's C, with `n = 4, m = 3` where rung 70 says `m = 2`.
    assert!(!(lag.is_some() && (lim.accel.is_some() || lim.floor().is_some())),
            "rung-71: rung 52's phi FUEL leg beside this governor is `n = 4, m = 3` -- FOUR \
             loops, two of them on one actuator. It is an unregistered plant and rung 70's own \
             named seam; rung 68's `tau_gov` assert exists because 'silently accepts it' is the \
             failure mode. Arm one fuel-side leg, not both.");
    // GUARD C — rung 70's D.
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-71: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger. All three legs here pin their own (rung 68's \
             argument, verbatim through rung 70).");
    // GUARD D — rung 70's E.
    assert!(ft.inner.lever.lim.is_none() || lagged(&ft.inner),
            "rung-71: an INSTANTANEOUS valve beside a lagged stator is not a control but a \
             different plant (rung 65 called the instantaneous limit singular, and rung 66 \
             refused the comparison for that reason). Give the valve a `tau` or leave it out.");
    let tau_gov = tau_gov.expect("the reduce test above returned if this was None");
    // **`_stator_leg()` IS CALLED TWICE HERE, AND THAT IS PYTHON's LINE, NOT AN OVERSIGHT.** The
    // source spells `… + 1.0 / self._stator_leg().tau, self._stator_leg().tau` — two dispatches
    // through a CELL whose rung-69 body bumps `LEG_INC`/`LEG_PARENT`. Binding it once is
    // arithmetically identical and moves a shipped counter by one per march, which is exactly the
    // class of difference this port's dispatch gates read. So it is a closure, called twice.
    let stator_tau = || {
        ft.inner.stator_leg().expect("rung-71's march with no stator floor")
          .tau.expect("rung-71's march on an unlagged stator")
    };
    // PYTHON's OWN SUMMATION ORDER: governor, then valve, then stator. Rung 68's Rust accumulates
    // from `1/tau_s` outward, and copying that template here would change the rounding of the
    // argument the floor tests.
    let rate = 1.0 / tau_gov
        + (if lagged(&ft.inner) {
               1.0 / ft.inner.lever.lim.expect("lagged()").tau.expect("lagged()")
           } else {
               0.0
           })
        + 1.0 / stator_tau();
    r71_rk4_floor_full(ds, rate, stator_tau());
    // RUNG 70's FIVE-STATE INTEGRATOR, UNCHANGED AND UNCOPIED. Rung 69 made the five seams this
    // needs overridable (`_stator_leg`, `_clamp_v`, `_check_v0`, `_manifold_v`, `_solve_v`) and
    // each is the IDENTITY of what it replaced, so the ONLY thing that moves between rungs 70 and
    // 71 is which limiter `_stator_leg` hands back. Rungs 68/69/70 each shipped a sibling
    // integrator because a state was being ADDED; nothing is added here.
    r70_integrate_fuel_cross_triple(
        ft, flight, fuel_schedule, nu0, s_end, ds, lim.freeze,
        tt4_max.expect("guard A just fired if this was None"), tau_gov)
}

// ---------------------------------------------------------------------------------------------
// THE RIG, THE DAMPING READER AND THE ROUNDING — the three PLAIN pieces the readers stand on
// ---------------------------------------------------------------------------------------------

/// RUNG 71's `_full_rig` — **rung 70's [`split_rig`] with the stator's REFERENCE moved and NOTHING
/// else.**
///
/// One constructor for every cell of every table here, so a cell can differ from another only by
/// which loops are armed (rung 63's lesson).
///
/// Both floors still come from the SAME `from_margin(cmap, ., sm)`. Under rung 70 that was
/// load-bearing because `pair_CV = 1` is an identity of a SHARED set point; here nothing is shared
/// and no identity depends on it — but it is kept, because rung 69's constructor asserts
/// `m_lim == T_c - 1/phi_lim` and because comparing two references at UNEQUAL walls would confound
/// this rung with a set-point offset (rung 66 measured −2.5 % moving its own product to 0.951).
///
/// # THE SET POINT IS A BARE, PERMANENT ASSIGNMENT — **NOT [`GovScope`]**
///
/// Step 2's highest-consequence line, in this file's copy of it. Python is `m = self.at_lever(…)`
/// then `m._gov_max = Tt4_max` — the half a `try/finally` census cannot see, because `_gov_max` is
/// written **two different ways**. Reaching for the RAII guard here would restore on drop, rung
/// 70's `_triple_laws` would then find `gov_max == None`, take its reduce arm, and **every reader
/// would measure rung 68** — which per § 5.27 (ii) returns successfully with ZERO rows, so a
/// value-diff gate passes on two empty tables.
///
/// # AND THE SET IS UNCONDITIONAL, INCLUDING FOR [`full_bill`]'s GOVERNOR-LESS CELLS
///
/// The ledger's `bare`/`V`/`S`/`VS` cells turn the governor off **at the MARCH**, never at the
/// rig. A `gov` flag here would be a second way to disarm the same loop.
///
/// `b_max` falls back to Python's own `0.10` when the receiver carries no valve.
///
/// [`GovScope`]: crate::cross_split::GovScope
#[allow(clippy::too_many_arguments)]
pub fn full_rig(
    core: &ScheduledStatorCore, sm: f64, tau: f64, tau_s: f64, v_max: f64, tt4_max: f64,
    valve: bool, stator: bool,
) -> ScheduledStatorCore {
    let cmap = core.arming().map_lp_design;
    let b_max = core.fuel.inner.lever.lim.map(|l| l.b_max).unwrap_or(0.10);
    let bl = if valve {
        Some(BleedLimiter::from_margin_tau(&cmap, b_max, sm, Some(tau)))
    } else {
        None
    };
    let sl = if stator {
        Some(StatorIncidenceLimiter::from_margin(&cmap, v_max, sm, Some(tau_s)))
    } else {
        None
    };
    let m = core.at_lever(&LeverArm { bleed_lim: bl, stator_inc: sl, ..Default::default() });
    // THE BARE, PERMANENT SET. See the note above — this is NOT `GovScope`.
    m.fuel.inner.gov_max.set(Some(tt4_max));
    bump(&FULL_RIG_CALLS);
    m
}

/// RUNG 71's `_zeta_ring` — **the pair identified by its IMAGINARY PART, and it CANNOT be rung
/// 70's [`zeta_pair`] either.**
///
/// THE THIRD REBUILD OF THIS INSTRUMENT IN FOUR RUNGS, each with the same cause: the rung changed
/// WHICH ROOT IS WHICH.
///
/// | rung | reads | exact when |
/// |---|---|---|
/// | 69 | `-Re(dom)/\|dom\|` | the DOMINANT pair is complex; returns exactly 1.0 for a real root |
/// | 70 | both NON-ZERO roots, magnitude-sorted | exactly ONE root is zero |
/// | 71 | the pair identified by its IMAGINARY PART | always — and `None` when there is none |
///
/// **Here NO root is zero and the pair is not always the two largest.** Measured against rung 70's
/// reader over a 12-arm clock grid it disagrees on FOUR: 0.960 vs 0.686, 1.279 vs 0.670, 1.045 vs
/// 0.924, and 1.035 on an arm whose spectrum is entirely REAL. A reader that returns a number
/// where there is no ring is worse than one that returns nothing, so this returns `None` and every
/// caller reports the count of real-spectrum arms.
///
/// # THREE SPELLING DIFFERENCES FROM [`zeta_pair`], AND EACH ONE MOVES A KEY
///
/// * **No `sorted_by_abs`.** Python filters the roots in the cubic solver's OWN order and takes
///   `cx[0]` — the FIRST complex root, not the largest.
/// * **The test is PER ROOT**, `abs(r.imag) > 1e-6 * abs(r)`, so `complex_pair` at rung 71 is
///   *any* root complex where rung 70's `SplitModesRow` asks it of the DOMINANT one. Two aggregate
///   keys ([`FullModes::arms_with_ring`] and [`FullModes::arms_real`]) read that difference
///   directly, and copying rung 70's row spelling would move both.
/// * **No complex arithmetic at all** — `-r.real / abs(r)` is two real operations, where rung 70's
///   goes through a complex sum, product, `sqrt` and division. § 5.27 (iv)'s division exemption
///   does not reach this reader.
///
/// [`zeta_pair`]: crate::cross_split::zeta_pair
pub fn zeta_ring(roots: [C64; 3]) -> Option<f64> {
    let r = *roots.iter().find(|r| r.im.abs() > 1e-6 * r.abs())?;
    if r.abs() == 0.0 {
        None
    } else {
        Some(-r.re / r.abs())
    }
}

/// Python's `round(x, 10)` — correctly rounded to ten DECIMAL digits, ties to even.
///
/// Format-and-parse, not `(x*1e10).round()/1e10`: the latter rounds the SCALED value and the
/// scaling is itself inexact. [`round12`](crate::three_loop::round12),
/// [`round6`](crate::two_spool::round6) and [`round3`](crate::two_spool_transient::round3) are the
/// same decision at three other widths, each with its divergence class closed by construction.
///
/// It exists for exactly one reader: [`ic_contraction`]'s `members` counts the DISTINCT fixed
/// points the six sweep orders land on, and its set key is a rounded triple. A wrong rounding
/// there does not move a float — it moves an INTEGER, and that integer IS this rung's § 3 headline
/// (*at `n = m` the `s = 0` fixed point is a POINT*).
///
/// **`to_bits` IS NOT PYTHON'S SET KEY, AND SIGNED ZERO IS WHERE THEY PART** — rung 68's
/// `ic_family` note, verbatim, and normalised here for its reason: a Python set compares floats
/// with `==`, under which `-0.0 == 0.0`, so a member reached at `-0.0` and one reached at `+0.0`
/// are ONE member there and would be TWO here. `g` starts at exactly `+0.0` on the shipped grid,
/// so the difference is unreachable and therefore invisible to every value key; normalising it
/// anyway is what stops a latent off-by-one in a COUNT no value gate can witness.
pub fn round10(x: f64) -> f64 {
    format!("{x:.10}").parse::<f64>().expect("a formatted finite double parses back")
}

/// Python's `p.get("v", 0.0)` — the stator setting a point carries, **or the design setting when
/// the point came from a march with no stator state at all.**
///
/// [`v_at_point`] PANICS on those routes, and that is right for the readers that difference gains
/// off a five-state trajectory. **[`full_bill`]'s `v_hi` is the one place the non-`Triple` arm is
/// REACHABLE**: its `bare`, `G`, `V` and `GV` cells march with the stator disarmed, so the
/// trajectory reduces to a shape that never recorded `v`. Rung 70's ledger never read `v`, so
/// there is no precedent to copy — `violation_inc` is the only other reader in the family that
/// takes this fallback, and it takes it for the same reason.
fn v_or_zero(p: &FuelPoint) -> f64 {
    match p.extra {
        crate::fuel_transient::PointExtra::Triple { v, .. } => v,
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------------------------
// § 0 — THE BAND-REDUNDANCY LAW: the third loop lives inside the SECOND's LAG
// ---------------------------------------------------------------------------------------------

/// The three fields a rung-71 trajectory point carries that the window spans read.
///
/// **NOT [`crate::cross_split`]'s reader, and the duplication is DELIBERATE** — Python's
/// `window_law` writes its own `span` selectors rather than calling rung 70's, so a shared helper
/// here would factor away a duplication the SOURCE makes
/// ([[rust-port-copy-vs-rederivation]]'s rule). What it buys beyond fidelity is the RUNG TAG in
/// the panic, which is the only thing that tells a reader which march refused.
///
/// The arms are spelled out and there is no wildcard, so the NEXT [`PointExtra`] variant breaks
/// the build here and gets the same question asked of it a second time.
///
/// Python reads `p["required"]` and `p["b_cmd"]` (a `KeyError` off a five-state trajectory) but
/// `p.get("v_regime")` (None-safe). The asymmetry cannot be reached: `gov` spans the WHOLE
/// trajectory before `stator` does, so any non-`Triple` point raises there first.
///
/// [`PointExtra`]: crate::fuel_transient::PointExtra
fn full_window_extra(p: &FuelPoint) -> (f64, f64, Regime) {
    use crate::fuel_transient::PointExtra;
    match p.extra {
        PointExtra::Triple { required, b_cmd, v_regime, .. } => (required, b_cmd, v_regime),
        PointExtra::None
        | PointExtra::Asym { .. }
        | PointExtra::Valve { .. }
        | PointExtra::Cascade { .. }
        | PointExtra::CrossCascade { .. } =>
            panic!("rung-71's windows need a five-state trajectory: Python raises KeyError on \
                    `required` for every other route."),
    }
}

/// One clock arm of [`window_law`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowLawArm {
    /// `(tau_gov, tau_q, tau_s)` — the `(g, q, v)` order of the STATE VECTOR, which is not the
    /// order the sweeps are written in.
    pub taus: (f64, f64, f64),
    pub n: usize,
    pub phi_lim: f64,
    /// The MARCHED `phi` where the stator goes dormant, and the setting it holds there.
    pub phi_at_stator_off: Option<f64>,
    pub v_at_stator_off: Option<f64>,
    pub gov: Span,
    pub valve: Span,
    pub stator: Span,
    pub joint: Span,
    /// Points where all three loops ride AND every arm of the central difference stays on-regime
    /// — **the quotable sample, never inferred from the window's width.**
    pub n_interior: usize,
    pub v_hi: f64,
    pub min_phi: f64,
    pub stator_off: Option<f64>,
    pub phi_recovers_marched: Option<f64>,
}

/// RUNG 71 § 0's return.
#[derive(Clone, Debug)]
pub struct WindowLaw {
    pub base: WindowLawArm,
    pub by_tau_q: Vec<WindowLawArm>,
    pub by_tau_s: Vec<WindowLawArm>,
    pub tau_qs: Vec<f64>,
    pub tau_ss: Vec<f64>,
    /// Each arm's stator window RIGHT EDGE. `Option` because Python's `span` returns `None` on an
    /// empty selection — and the aggregates below are exactly where Python would `TypeError`.
    pub edge_q: Vec<Option<f64>>,
    pub edge_s: Vec<Option<f64>>,
    /// THE LAW: monotone in the VALVE's clock…
    pub q_monotone: bool,
    pub q_span: Option<f64>,
    /// …and comparatively flat in the STATOR's own.
    pub s_span: Option<f64>,
    pub joint_fraction: f64,
    /// The stator quits while the MARCHED `phi` is still short of the floor — **by its own
    /// contribution, and by nothing else.**
    pub phi_short_at_off: Option<f64>,
    pub v_at_off: Option<f64>,
}

/// RUNG 71 § 0 — **the third constraint is REDUNDANT ON THE BAND, so the third loop lives inside
/// the SECOND's LAG.**
///
/// The derivation carries no new constant. At the valve's own set point,
///
/// ```text
/// phi = phi_lim  =>  M_i = T_c - 1/phi_lim + v = m_lim + v >= m_lim   for all v >= 0
/// ```
///
/// and the incidence band IS `[0, v_max]` (rung 69 § 0.1), so `{phi >= phi_lim} INTERSECT
/// {v >= 0}` sits inside `{M_i >= m_lim}`. The stator can therefore only ride where the valve has
/// NOT yet delivered — which on a lagged plant is exactly the valve's own lag.
///
/// **TWO SWEEPS, because a one-sided one would not separate the mechanism from the plant.**
/// `tau_qs` moves the VALVE's clock (predicted: the right edge marches monotonically OUT) and
/// `tau_ss` moves the STATOR's own (predicted: comparatively flat). If both moved together the law
/// would be *a slower loop rides longer*, which is a different and much weaker statement.
///
/// # THE MARCHED CROSSING IS NOT THE EVENT, AND SAYING SO IS THE POINT
///
/// `_solve_v` tests dormancy on the COUNTERFACTUAL plant at `v = 0`, so the stator quits when
/// `phi` WOULD clear the floor with the stators back at the design setting — while the MARCHED
/// `phi` is still below it by exactly the stator's own contribution (measured `dphi/dv ~ -0.42`
/// times `v`). The two edges differ by a real amount and quoting their agreement would be a fudge;
/// [`band_containment`] is where the exact statement lives, and it needs no counterfactual.
#[allow(clippy::too_many_arguments)]
pub fn window_law(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, tau_qs: &[f64], tau_ss: &[f64], r: f64, s_settle: f64, ds: f64, tau: f64,
    tau_gov: f64, tau_s: f64, v_max: f64,
) -> WindowLaw {
    let arm = |tq: f64, tg: f64, ts: f64| -> WindowLawArm {
        let m = full_rig(core, sm, tq, ts, v_max, tt4_max, true, true);
        let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
        let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
        let (traj, _) = m.stator_march_scoped(
            flight, &ramp, None, &leg,
            &MarchScope { tau_gov: Some(tg), ..MarchScope::DEFAULT });
        let bl = m.fuel.inner.lever.lim.expect("the rig arms a valve");
        let b_max = bl.b_max;

        let span = |sel: &dyn Fn(&FuelPoint) -> bool| -> Span {
            let w: Vec<f64> = traj.iter().filter(|p| sel(p)).map(|p| p.s).collect();
            if w.is_empty() {
                (None, None, 0)
            } else {
                (opt_fold(w.iter().copied(), f64::min), opt_fold(w.iter().copied(), f64::max),
                 w.len())
            }
        };

        let pts = riding(&traj, b_max);
        let n_interior = pts.iter()
            .filter(|p| {
                triple_gains_at(&m, flight, p, None, None, 1e-7, 1e-5, 1e-4, true, 0.0, true)
                    .expect("rung-71's window march does not abort")
                    .interior
            })
            .count();
        // WHERE the stator goes dormant, and the MARCHED `phi` there. See the note above.
        let rid: Vec<&FuelPoint> =
            traj.iter().filter(|p| full_window_extra(p).2 == Regime::Riding).collect();
        let first_rid = rid.first().map(|p| p.s).unwrap_or(0.0);
        let caught: Vec<f64> = traj.iter()
            .filter(|p| p.phi_lp >= bl.phi_lim - 1e-9 && p.s > first_rid)
            .map(|p| p.s)
            .collect();
        let off = rid.last().copied();
        WindowLawArm {
            taus: (tg, tq, ts),
            n: traj.len(),
            phi_lim: bl.phi_lim,
            phi_at_stator_off: off.map(|p| p.phi_lp),
            v_at_stator_off: off.map(v_at_point),
            gov: span(&|p| full_window_extra(p).0 > 0.0),
            valve: span(&|p| {
                let (_, cmd, _) = full_window_extra(p);
                0.0 < cmd && cmd < b_max
            }),
            stator: span(&|p| full_window_extra(p).2 == Regime::Riding),
            joint: span(&|p| {
                let (req, cmd, reg) = full_window_extra(p);
                req > 0.0 && 0.0 < cmd && cmd < b_max && reg == Regime::Riding
            }),
            n_interior,
            v_hi: opt_fold(traj.iter().map(v_at_point), f64::max)
                      .expect("rung-71's window march is never empty"),
            min_phi: opt_fold(traj.iter().map(|p| p.phi_lp), f64::min)
                         .expect("rung-71's window march is never empty"),
            stator_off: off.map(|p| p.s),
            phi_recovers_marched: opt_fold(caught.iter().copied(), f64::min),
        }
    };

    let by_q: Vec<WindowLawArm> = tau_qs.iter().map(|&tq| arm(tq, tau_gov, tau_s)).collect();
    let by_s: Vec<WindowLawArm> = tau_ss.iter().map(|&ts| arm(tau, tau_gov, ts)).collect();
    // Python's `by_q[list(tau_qs).index(tau)] if tau in tau_qs else arm(...)` — the base arm is
    // REUSED out of the sweep when the sweep already contains it, so the marches are not
    // duplicated and every counter reads the same as Python's.
    let base = match tau_qs.iter().position(|&x| x == tau) {
        Some(i) => by_q[i],
        None => arm(tau, tau_gov, tau_s),
    };

    let edge = |rows: &[WindowLawArm]| -> Vec<Option<f64>> {
        rows.iter().map(|x| x.stator.1).collect()
    };
    let eq = edge(&by_q);
    let es = edge(&by_s);
    // Python compares and takes `max`/`min` over these lists directly, so a `None` edge is a
    // `TypeError` there rather than a skipped arm. The panic is that same refusal, said out loud:
    // an arm with no stator window at all is a plant this sweep has never reached.
    let solid = |v: &[Option<f64>], which: &str| -> Vec<f64> {
        v.iter()
         .map(|x| x.unwrap_or_else(|| panic!(
             "rung-71 window_law: an arm of the {which} sweep has an EMPTY stator window. Python \
              raises TypeError comparing None to a float here.")))
         .collect()
    };
    let ev = solid(&eq, "tau_q");
    let es_v = solid(&es, "tau_s");
    let ratio = |v: &[f64]| -> Option<f64> {
        let lo = v.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        // Python's `if min(es)` — a ZERO edge is falsy and yields None, not a division.
        if lo != 0.0 { Some(hi / lo) } else { None }
    };
    WindowLaw {
        q_monotone: (0..ev.len().saturating_sub(1)).all(|i| ev[i] <= ev[i + 1] + 1e-12),
        q_span: ratio(&ev),
        s_span: ratio(&es_v),
        joint_fraction: if base.n == 0 { 0.0 } else { base.joint.2 as f64 / base.n as f64 },
        phi_short_at_off: base.phi_at_stator_off.map(|x| base.phi_lim - x),
        v_at_off: base.v_at_stator_off,
        base,
        by_tau_q: by_q,
        by_tau_s: by_s,
        tau_qs: tau_qs.to_vec(),
        tau_ss: tau_ss.to_vec(),
        edge_q: eq,
        edge_s: es,
    }
}

/// One marched point of [`band_containment`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BandContainmentRow {
    pub s: f64,
    pub phi: f64,
    pub v: f64,
    /// `M_i - m_lim` at the LIVE state.
    pub slack: f64,
    pub delivering: bool,
    pub riding: bool,
}

/// RUNG 71 § 0's containment return.
#[derive(Clone, Debug)]
pub struct BandContainment {
    pub rows: Vec<BandContainmentRow>,
    pub n: usize,
    pub n_delivering: usize,
    /// The containment: `slack >= v >= 0` wherever the valve delivers.
    pub min_slack_delivering: Option<f64>,
    pub worst_slack_minus_v: Option<f64>,
    /// …and the stator is dormant on every one of those points.
    pub riding_while_delivering: usize,
    pub min_slack_all: f64,
    pub n_riding: usize,
}

/// RUNG 71 § 0's containment as an ARITHMETIC statement on the marched trajectory, **beside the
/// RANK it does not contradict.**
///
/// For every marched point this evaluates `M_i - m_lim` at the LIVE state and reports the minimum
/// over the points where the valve is DELIVERING (`phi_lp >= phi_lim`). The prediction is that it
/// is `>= v >= 0` there — never negative — so the incidence loop has nothing to do wherever the
/// valve has succeeded. **That is a statement about FEASIBLE SETS and it coexists with `m = 3`,
/// which is a statement about GRADIENTS.**
#[allow(clippy::too_many_arguments)]
pub fn band_containment(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, r: f64, s_settle: f64, ds: f64, tau: f64, tau_gov: f64, tau_s: f64, v_max: f64,
) -> BandContainment {
    let m = full_rig(core, sm, tau, tau_s, v_max, tt4_max, true, true);
    let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
    let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
    let (traj, _) = m.stator_march_scoped(
        flight, &ramp, None, &leg, &MarchScope { tau_gov: Some(tau_gov), ..MarchScope::DEFAULT });
    // `self.map_lp_design`, the RECEIVER's — not `m`'s. They are equal and the spelling is the
    // claim, exactly as at rung 70's `split_gains`.
    let t_c = core.arming().map_lp_design.tan_beta1_crit();
    let phi_lim = m.fuel.inner.lever.lim.expect("the rig arms a valve").phi_lim;
    let m_lim = m.fuel.inner.stator.inc.expect("the rig arms an incidence stator").m_lim;
    let rows: Vec<BandContainmentRow> = traj.iter()
        .map(|p| {
            let v = v_at_point(p);
            let mi = StatorIncidenceLimiter::margin(t_c, p.phi_lp, v);
            BandContainmentRow {
                s: p.s,
                phi: p.phi_lp,
                v,
                slack: mi - m_lim,
                delivering: p.phi_lp >= phi_lim - 1e-12,
                riding: full_window_extra(p).2 == Regime::Riding,
            }
        })
        .collect();
    let deliv: Vec<&BandContainmentRow> = rows.iter().filter(|x| x.delivering).collect();
    BandContainment {
        n: rows.len(),
        n_delivering: deliv.len(),
        min_slack_delivering: opt_fold(deliv.iter().map(|x| x.slack), f64::min),
        worst_slack_minus_v: opt_fold(deliv.iter().map(|x| x.slack - x.v), f64::min),
        riding_while_delivering: deliv.iter().filter(|x| x.riding).count(),
        min_slack_all: opt_fold(rows.iter().map(|x| x.slack), f64::min)
                           .expect("rung-71's containment march is never empty"),
        n_riding: rows.iter().filter(|x| x.riding).count(),
        rows,
    }
}

// ---------------------------------------------------------------------------------------------
// § 1 — THE THREE PAIRS, TWO INHERITED CONTROLS, AND THE FACTORING
// ---------------------------------------------------------------------------------------------

/// Python's `min(a, b, c)` over a three-term generator — a LEFT FOLD with `<`, the mirror of
/// [`py_max3`].
fn py_min3(a: f64, b: f64, c: f64) -> f64 {
    let mut best = a;
    if b < best {
        best = b;
    }
    if c < best {
        best = c;
    }
    best
}

/// One sampled point of [`full_gains`].
#[derive(Clone, Debug)]
pub struct FullGainsRow {
    pub s: f64,
    /// RUNG 71's plant — three loops, three constraints.
    pub gains: TripleGains,
    /// RUNG 70's rig read at the IDENTICAL base point: the same machine with the stator moved
    /// back onto `phi`, which is the plant whose valve and stator SHARE a wall.
    pub phi_rig: TripleGains,
    /// The FORWARD cyclic product `R_q C_v V_g`, which is a PRODUCT of two other pairs here.
    pub x: f64,
    /// The REVERSE one `R_v C_g V_q`, which IS `-pair_RV`.
    pub y: f64,
    pub det: f64,
    pub det_pred: f64,
    pub y_is_rv: f64,
    pub x_is_product: f64,
    pub det_err: f64,
    /// `pair_RV(71) = pair_CV * pair_RV(70)` — the cross-rung identity, `None` where the rung-70
    /// rig's own row is off-regime or its pair is exactly zero.
    pub cross_rung: Option<f64>,
}

/// RUNG 71 § 1's return.
#[derive(Clone, Debug)]
pub struct FullGains {
    pub n_riding: usize,
    pub n_sampled: usize,
    pub rows: Vec<FullGainsRow>,
    /// DISCLOSED, never a silent truncation — `(s, off-regime arms)`.
    pub skipped: Vec<(f64, Vec<&'static str>)>,
    pub boundary: Vec<StateBoundary>,
    pub ds: f64,
    pub s_window: Option<(f64, f64)>,
    /// **NO pair is 1** — rung 66's identity is a property of a SHARED constraint, and nothing is
    /// shared, so it appears ZERO times for the first time in the family.
    pub closest_to_1: Option<f64>,
    pub worst_y_is_rv: Option<f64>,
    pub worst_x_is_product: Option<f64>,
    /// THE FACTORING — and it uses only FOUR of the six gains.
    pub worst_det_err: Option<f64>,
    pub det_scale: Option<f64>,
    pub worst_cross_rung: Option<f64>,
    pub pair_rc: Vec<f64>,
    pub pair_rv: Vec<f64>,
    pub pair_cv: Vec<f64>,
}

/// RUNG 71 § 1 — **the six cross-gains, the THREE pairwise products, BOTH cyclic products and the
/// determinant, with the rung-70 rig read at the IDENTICAL base points.**
///
/// | reading | what it carries |
/// |---|---|
/// | no pair is 1 | rung 66's identity needs a SHARED constraint, and nothing is shared |
/// | `y + pair_RV = 0` | the reverse cyclic product IS the new pair, negated |
/// | `x + pair_RC*pair_CV` | the forward one is a PRODUCT of the other two |
/// | `det + (1-RC)(1-CV)` | **THE FACTORING** — one factor per rung, from FOUR of the six gains |
/// | `RV / (CV * RV70)` | `pair_RV(71) = pair_CV * pair_RV(70)`, the cross-rung identity |
///
/// # THE TWO CONTROLS ARE DIFFERENT KINDS, AND CONFLATING THEM WOULD BE THE ERROR
///
/// * `pair_RC` is a **NUMERICAL** control — rows R and C are the SAME closures rungs 70 and 67
///   used, at the same base point, so it must reproduce rung 67's `P` to the differencing floor.
/// * `pair_CV` is a **FUNCTIONAL-FORM** control — rung 69's `k` on rung 69's own two loops, but
///   re-measured on a DIFFERENT trajectory. Its FORM and BAND are gated; a tolerance the
///   trajectory shift does not justify is not.
///
/// # THE RUNG-70 ARM IS EVALUATED **AFTER** THE INTERIOR TEST, WHICH IS NOT RUNG 70's ORDER
///
/// Rung 70's `split_gains` evaluates BOTH arms above its `if not gov["interior"]`, so its closure
/// counts do not depend on which arm was off. **Python's rung-71 body puts `g70` below the
/// `continue`**, so a skipped point costs one gains evaluation here and two there. Copying rung
/// 70's template would change how many closure calls the plant sees on every off-regime sample —
/// invisible to every float, visible to a counter.
///
/// `m70` is [`split_rig`] called on a RUNG-71 receiver, so `at_lever` hands back a rung-71 machine
/// armed with a `phi` stator. That is the SHARED plant, not a rung-70 class; building the latter
/// would be a different object that still produced a table.
#[allow(clippy::too_many_arguments)]
pub fn full_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, r: f64, s_settle: f64, ds: f64, tau: f64, tau_gov: f64, tau_s: f64, v_max: f64,
    every: usize,
) -> FullGains {
    let m = full_rig(core, sm, tau, tau_s, v_max, tt4_max, true, true);
    let m70 = split_rig(core, sm, tau, tau_s, v_max, tt4_max, true, true);
    let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
    let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
    let (traj, _) = m.stator_march_scoped(
        flight, &ramp, None, &leg, &MarchScope { tau_gov: Some(tau_gov), ..MarchScope::DEFAULT });
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding(&traj, b_max);
    // Python's `pts[::every]`; `every = 0` raises there and panics here.
    let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
    let (mut rows, mut skipped, mut boundary) = (Vec::new(), Vec::new(), Vec::new());
    for p in &sampled {
        let gg = triple_gains_at(&m, flight, p, None, None, 1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-71's gains march does not abort");
        if !gg.interior {
            skipped.push((p.s, gg.off_regime.clone()));
            continue;
        }
        boundary.push(assert_state_boundary(&m, flight, p, tt4_max, 1e-5, 1e-4)
                          .expect("rung-71's boundary instrument does not abort"));
        let g70 = triple_gains_at(&m70, flight, p, None, None, 1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-71's shared-plant control does not abort");
        let x = gg.r_q * gg.c_v * gg.v_g;
        let y = gg.r_v * gg.c_g * gg.v_q;
        // PYTHON's OWN LEFT-TO-RIGHT SUM. Regrouping the six terms would move the last bits of
        // `det`, and `det_err` is a DIFFERENCE against a closed form — the one key where that
        // rounding is the whole measurement.
        let det = -1.0 + gg.pair_rc + gg.pair_rv + gg.pair_cv + x + y;
        let det_pred = -(1.0 - gg.pair_rc) * (1.0 - gg.pair_cv);
        rows.push(FullGainsRow {
            s: p.s,
            x, y, det, det_pred,
            y_is_rv: (y + gg.pair_rv).abs(),
            x_is_product: (x + gg.pair_rc * gg.pair_cv).abs(),
            det_err: (det - det_pred).abs(),
            cross_rung: if g70.interior && g70.pair_rv != 0.0 {
                Some((gg.pair_rv / (gg.pair_cv * g70.pair_rv) - 1.0).abs())
            } else {
                None
            },
            gains: gg,
            phi_rig: g70,
        });
    }
    FullGains {
        n_riding: pts.len(),
        n_sampled: sampled.len(),
        ds,
        s_window: if pts.is_empty() { None } else { Some((pts[0].s, pts[pts.len() - 1].s)) },
        closest_to_1: opt_fold(rows.iter().map(|x| py_min3((x.gains.pair_rc - 1.0).abs(),
                                                           (x.gains.pair_rv - 1.0).abs(),
                                                           (x.gains.pair_cv - 1.0).abs())),
                               f64::min),
        worst_y_is_rv: opt_fold(rows.iter().map(|x| x.y_is_rv), f64::max),
        worst_x_is_product: opt_fold(rows.iter().map(|x| x.x_is_product), f64::max),
        worst_det_err: opt_fold(rows.iter().map(|x| x.det_err), f64::max),
        det_scale: opt_fold(rows.iter().map(|x| x.det_pred.abs()), f64::min),
        worst_cross_rung: opt_fold(rows.iter().filter_map(|x| x.cross_rung), f64::max),
        pair_rc: rows.iter().map(|x| x.gains.pair_rc).collect(),
        pair_rv: rows.iter().map(|x| x.gains.pair_rv).collect(),
        pair_cv: rows.iter().map(|x| x.gains.pair_cv).collect(),
        rows,
        skipped,
        boundary,
    }
}

// ---------------------------------------------------------------------------------------------
// § 2 — THE SPECTRUM: ZERO zeros, `det` ALIVE, Routh non-trivial
// ---------------------------------------------------------------------------------------------

/// One sampled point of one [`full_modes`] arm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FullModesRow {
    pub s: f64,
    pub c2: f64,
    pub c1: f64,
    pub c0: f64,
    pub roots: [C64; 3],
    /// `-u z a b c` — the closed form that uses FOUR of the six gains and asserts the other two
    /// drop out. **`c1`'s closed form is deliberately NOT here**: for any matrix with `-1` on the
    /// diagonal it IS the second invariant, so gating it would be the shipped `_invariants`
    /// agreeing with itself (rung 67 gate 9's retraction).
    pub c0_pred: f64,
    pub c0_err: Option<f64>,
    pub u: f64,
    pub w: f64,
    pub z: f64,
    /// `u + w + z - u z` — SUFFICIENT for stability at every bandwidth triple, and the first
    /// non-trivial stability condition this family has had.
    pub routh: f64,
    pub pair_rc: f64,
    pub pair_rv: f64,
    pub pair_cv: f64,
    pub zeta: Option<f64>,
    /// Rung 69's `1/sqrt(1-pair_CV)`, which is **NOT a bound here**: rung 69's third root was the
    /// ZERO and took nothing from the trace budget, while this one drains it.
    pub r69_floor: Option<f64>,
    pub below_r69: bool,
    /// **`zeta is not None`** — ANY root complex, which is not rung 70's dominant-root test.
    pub complex_pair: bool,
    pub n_zero: usize,
    pub min_root: f64,
    pub max_root: f64,
    pub stable: bool,
    pub ds_lambda: f64,
    pub mod_ratio: f64,
}

/// One clock triple's arm of [`full_modes`].
#[derive(Clone, Debug)]
pub struct FullModesArm {
    /// `(tau_g, tau_q, tau_s)` — the `(g, q, v)` order of the STATE VECTOR, not the grid's.
    pub taus: (f64, f64, f64),
    pub rate_sum: f64,
    pub n: usize,
    pub n_sampled: usize,
    pub skipped: usize,
    pub rows: Vec<FullModesRow>,
    pub zeros: Vec<usize>,
    pub min_root_rel: Option<f64>,
    pub max_c0_err: Option<f64>,
    pub min_routh: Option<f64>,
    pub all_stable: Option<bool>,
    pub any_complex: Option<bool>,
    pub any_below_r69: Option<bool>,
    pub max_mod_ratio: Option<f64>,
    pub zeta_range: (Option<f64>, Option<f64>),
}

/// RUNG 71 § 2's return.
#[derive(Clone, Debug)]
pub struct FullModes {
    pub clocks: Vec<(f64, f64, f64)>,
    pub ds: f64,
    pub arms: Vec<FullModesArm>,
    /// **`[0]`** — the last unoccupied cell, and the first plant in this family whose actuator
    /// block is invertible.
    pub zeros_everywhere: Vec<usize>,
    pub arms_with_ring: usize,
    pub arms_real: usize,
    pub arms_below_r69: usize,
    pub max_c0_err: Option<f64>,
    pub min_routh: Option<f64>,
    pub max_mod_ratio: Option<f64>,
    pub all_stable: bool,
}

/// RUNG 71 § 2 — **§ 1's spectrum across a clock grid.** `clocks` is `(tau_q, tau_gov, tau_s)`,
/// rungs 68/69/70's ordering of the same grid, so the arms line up row for row.
///
/// **THE DEFAULT GRID IS SIX ARMS**, chosen to span the three RING regimes § 5 needs (two below
/// rung 69's line, three above it, one with no complex pair at all) at the smallest arm count that
/// still does. Rungs 68/69/70 default to FOUR; a march at `ds = 0.002` is the cost, so arms are
/// not free.
///
/// # `complex_pair` IS `zeta is not None`, AND THAT IS NOT RUNG 70's TEST
///
/// Rung 70's `SplitModesRow` asks it of the DOMINANT root; this asks it of ANY root, through
/// [`zeta_ring`]. The two disagree whenever a non-dominant pair is complex, and
/// [`FullModes::arms_with_ring`] / [`FullModes::arms_real`] read the difference directly.
#[allow(clippy::too_many_arguments)]
pub fn full_modes(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, clocks: &[(f64, f64, f64)], r: f64, s_settle: f64, ds: f64, v_max: f64, every: usize,
) -> FullModes {
    let mut arms: Vec<FullModesArm> = Vec::new();
    for &(tau_q, tau_g, tau_s) in clocks {
        let m = full_rig(core, sm, tau_q, tau_s, v_max, tt4_max, true, true);
        let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
        let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
        let (traj, _) = m.stator_march_scoped(
            flight, &ramp, None, &leg,
            &MarchScope { tau_gov: Some(tau_g), ..MarchScope::DEFAULT });
        let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
        let pts = riding(&traj, b_max);
        // The `(g, q, v)` order of the state vector — NOT the grid's `(q, g, s)` order.
        let taus = (tau_g, tau_q, tau_s);
        // Python's `sum(1.0 / t for t in taus)` — a three-term LEFT FOLD.
        let rate = 1.0 / taus.0 + 1.0 / taus.1 + 1.0 / taus.2;
        let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
        let (mut rows, mut skipped) = (Vec::new(), 0usize);
        for p in &sampled {
            let gg = triple_gains_at(&m, flight, p, None, None, 1e-7, 1e-5, 1e-4, true, 0.0, true)
                .expect("rung-71's spectrum march does not abort");
            if !gg.interior {
                // DISCLOSED below, never a silent truncation.
                skipped += 1;
                continue;
            }
            let (c2, c1, c0) = invariants(&gg, taus);
            let roots = cubic_roots_c(c2, c1, c0);
            let nz = crate::reference_split::sorted_by_abs(roots);
            let u = 1.0 - gg.pair_rc;
            let w = 1.0 - gg.pair_rv;
            let z = 1.0 - gg.pair_cv;
            let (aa, bb, cc) = (1.0 / tau_g, 1.0 / tau_q, 1.0 / tau_s);
            let c0_pred = -u * z * aa * bb * cc;
            let zeta = zeta_ring(roots);
            // `z ** -0.5` is a GENERAL power and goes through `powp`, not a `sqrt` reciprocal —
            // `tests/porting_rules.rs` RULE 2.
            let r69_floor = if z > 0.0 { Some(powp(z, -0.5)) } else { None };
            rows.push(FullModesRow {
                s: p.s, c2, c1, c0, roots, c0_pred,
                c0_err: if c0_pred != 0.0 { Some((c0 / c0_pred - 1.0).abs()) } else { None },
                u, w, z,
                routh: u + w + z - u * z,
                pair_rc: gg.pair_rc, pair_rv: gg.pair_rv, pair_cv: gg.pair_cv,
                zeta, r69_floor,
                below_r69: match (zeta, r69_floor) {
                    (Some(zt), Some(fl)) => zt < fl,
                    _ => false,
                },
                complex_pair: zeta.is_some(),
                n_zero: roots.iter().filter(|x| x.abs() < 1e-4 * rate).count(),
                min_root: nz[0].abs(),
                max_root: nz[2].abs(),
                stable: roots.iter().all(|x| x.re < 0.0),
                ds_lambda: ds * nz[2].abs(),
                mod_ratio: nz[2].abs() / rate,
            });
        }
        let mut zeros: Vec<usize> = rows.iter().map(|x| x.n_zero).collect();
        zeros.sort_unstable();
        zeros.dedup();
        arms.push(FullModesArm {
            taus,
            rate_sum: -rate,
            n: pts.len(),
            n_sampled: sampled.len(),
            skipped,
            zeros,
            min_root_rel: opt_fold(rows.iter().map(|x| x.min_root / rate), f64::min),
            max_c0_err: opt_fold(rows.iter().filter_map(|x| x.c0_err), f64::max),
            min_routh: opt_fold(rows.iter().map(|x| x.routh), f64::min),
            all_stable: if rows.is_empty() { None } else { Some(rows.iter().all(|x| x.stable)) },
            any_complex: if rows.is_empty() { None }
                         else { Some(rows.iter().any(|x| x.complex_pair)) },
            any_below_r69: if rows.is_empty() { None }
                           else { Some(rows.iter().any(|x| x.below_r69)) },
            max_mod_ratio: opt_fold(rows.iter().map(|x| x.mod_ratio), f64::max),
            zeta_range: (opt_fold(rows.iter().filter_map(|x| x.zeta), f64::min),
                         opt_fold(rows.iter().filter_map(|x| x.zeta), f64::max)),
            rows,
        });
    }
    // Python's `live = [a for a in out if a["rows"]]` — an arm whose every sample was off-regime
    // contributes to NONE of the aggregates below, and its own keys are `None` rather than absent.
    let live: Vec<&FullModesArm> = arms.iter().filter(|a| !a.rows.is_empty()).collect();
    let mut zeros_everywhere: Vec<usize> =
        live.iter().flat_map(|a| a.zeros.iter().copied()).collect();
    zeros_everywhere.sort_unstable();
    zeros_everywhere.dedup();
    let some = |b: Option<bool>| b.expect("a live arm has rows, so this is never None");
    FullModes {
        arms_with_ring: live.iter().filter(|a| some(a.any_complex)).count(),
        arms_real: live.iter().filter(|a| !some(a.any_complex)).count(),
        arms_below_r69: live.iter().filter(|a| some(a.any_below_r69)).count(),
        max_c0_err: opt_fold(live.iter().filter_map(|a| a.max_c0_err), f64::max),
        min_routh: opt_fold(live.iter().filter_map(|a| a.min_routh), f64::min),
        max_mod_ratio: opt_fold(live.iter().filter_map(|a| a.max_mod_ratio), f64::max),
        all_stable: live.iter().all(|a| some(a.all_stable)),
        zeros_everywhere,
        clocks: clocks.to_vec(),
        ds,
        arms,
    }
}

// ---------------------------------------------------------------------------------------------
// § 3 — THE INITIAL CONDITION: a POINT, not a family
// ---------------------------------------------------------------------------------------------

/// One Gauss-Seidel sweep of [`ic_contraction`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IcSweepRow {
    pub order: &'static str,
    pub start: (f64, f64, f64),
    /// The displacement as a FRACTION of this rig's own band — see [`ic_contraction`].
    pub band: f64,
    pub g: f64,
    pub q: f64,
    pub v: f64,
    pub res: f64,
    pub iters: usize,
}

/// One rig's half of [`ic_contraction`] — Python's `out["full"]` / `out["shared"]`.
#[derive(Clone, Debug)]
pub struct IcContractionRig {
    pub rows: Vec<IcSweepRow>,
    pub n: usize,
    pub n_converged: usize,
    /// **THE HEADLINE, AND IT IS AN INTEGER** — how many DISTINCT fixed points the sweeps land on.
    /// At nullity zero the prediction is ONE. See [`round10`] for why the set key is a rounded
    /// triple and what a wrong rounding would do to this number.
    pub members: usize,
    pub spread: Option<(f64, f64, f64)>,
    pub marched: (f64, f64, f64),
    pub max_iters: Option<usize>,
}

/// RUNG 71 § 3's return — **a struct and not a map, because the two rigs' ORDER is what makes a
/// dump reproducible** (Python's dict is insertion-ordered and inserts `full` first).
#[derive(Clone, Debug)]
pub struct IcContraction {
    pub full: IcContractionRig,
    pub shared: IcContractionRig,
}

/// RUNG 71 § 3 — **at `n = m` the `s = 0` fixed point is a POINT, and the sweep REJECTS a moved
/// start instead of absorbing it.**
///
/// Rungs 68/69/70 all carry a null space, so their `s = 0` fixed points are a ONE-PARAMETER FAMILY
/// and a Gauss-Seidel sweep lands on whichever member its ORDER selects. Rung 69 § 6 measured the
/// IC spread GROWING as the nullity fell and called a null space a SHOCK ABSORBER. **At nullity
/// ZERO the prediction is neither absorption nor growth but COLLAPSE.**
///
/// # THE INSTRUMENT IS NOT `ic_family`'s, AND THAT IS DELIBERATE
///
/// `_stator_march`'s `b0`/`v0` arguments PIN their actuator — the integrator's steps skip
/// re-solving a pinned one — so a march started off the fixed point HOLDS the displacement by
/// construction and could never reject it. This runs the sweep ITSELF, from the same three shipped
/// laws, with nothing pinned:
///
/// ```text
/// g <- R(q, v) ,   q <- C(g, v) ,   v <- V(g, q)        in the given order
/// ```
///
/// **RUNG 70's PLANT IS THE NEGATIVE CONTROL, on the same rig.** Its valve and stator SHARE `phi`,
/// so `|C_v V_q| = 1` exactly and its sweep is marginal by construction. A contraction here that
/// is not matched by a failure to contract there would be measuring the SOLVER, not the rank.
///
/// # THE STARTING DISPLACEMENT IS A FRACTION OF EACH RIG's OWN BAND
///
/// The stator's band runs the OTHER WAY under the two references (rung 69 § 0.1), so `v_hi` is
/// `+stator_inc.v_max` on the full rig and **`-stator_lim.v_max`** on the shared one. Comparing
/// two plants at equal `v` would compare a dormant loop against a riding one.
///
/// The three laws are rebuilt INSIDE each sweep, once per `(order, fraction)` pair, because that
/// is where Python builds them — 24 constructions per rig, and a hoist would change every closure
/// count without moving a float.
#[allow(clippy::too_many_arguments)]
pub fn ic_contraction(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, orders: &[&'static str], fracs: &[f64], r: f64, s_settle: f64, ds: f64, tau: f64,
    tau_gov: f64, tau_s: f64, v_max: f64,
) -> IcContraction {
    let one = |rig: &ScheduledStatorCore| -> IcContractionRig {
        let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
        let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
        let (traj, _) = rig.stator_march_scoped(
            flight, &ramp, None, &leg,
            &MarchScope { tau_gov: Some(tau_gov), ..MarchScope::DEFAULT });
        let p0 = &traj[0];
        let at = (p0.nu_lp, p0.nu_hp, p0.mf_sched);
        let b_max = rig.fuel.inner.lever.lim.expect("both rigs arm a valve").b_max;
        let v_hi = match rig.fuel.inner.stator.inc {
            Some(si) => si.v_max,
            None => -rig.fuel.inner.stator.lim.expect("the shared rig arms a phi stator").v_max,
        };
        let mut rows: Vec<IcSweepRow> = Vec::new();
        for &order in orders {
            for &f in fracs {
                let start = (0.0, f * b_max, f * v_hi);
                let laws = rig.triple_laws(flight, at.0, at.1, at.2, None, None)
                              .expect("rung-71's IC sweep does not abort");
                let (mut g, mut q, mut v) = start;
                let mut res = f64::INFINITY;
                let mut its = 0usize;
                for i in 1..=120usize {
                    its = i;
                    let (mut gn, mut qn, mut vn) = (g, q, v);
                    for k in order.chars() {
                        match k {
                            'g' => gn = (laws.r)(qn, vn)
                                            .expect("rung-71's IC sweep does not abort").0,
                            'q' => qn = (laws.c)(gn, vn)
                                            .expect("rung-71's IC sweep does not abort").0,
                            'v' => vn = (laws.v)(gn, qn)
                                            .expect("rung-71's IC sweep does not abort").0,
                            _ => panic!("rung-71 ic_contraction order is over 'g'/'q'/'v'; got \
                                         {order:?}"),
                        }
                    }
                    // Python's THREE-argument `max`, so `py_max3` and not a chain of `f64::max`.
                    res = py_max3((gn - g).abs(), (qn - q).abs(), (vn - v).abs());
                    g = gn;
                    q = qn;
                    v = vn;
                    if res <= 1e-13 {
                        break;
                    }
                }
                rows.push(IcSweepRow { order, start, band: f, g, q, v, res, iters: its });
            }
        }
        let conv: Vec<&IcSweepRow> = rows.iter().filter(|x| x.res <= 1e-9).collect();
        // Python's `{(round(g,10), round(q,10), round(v,10)) …}` — a SET over ROUNDED triples, so
        // `members` is an INTEGER a wrong rounding moves outright. `+ 0.0` normalises `-0.0`,
        // because a Python set compares with `==` and `-0.0 == 0.0`.
        let key = |x: f64| (round10(x) + 0.0).to_bits();
        let mut members: Vec<[u64; 3]> =
            conv.iter().map(|x| [key(x.g), key(x.q), key(x.v)]).collect();
        members.sort_unstable();
        members.dedup();
        let span = |sel: fn(&IcSweepRow) -> f64| -> f64 {
            let hi = conv.iter().map(|x| sel(x)).fold(f64::NEG_INFINITY, f64::max);
            let lo = conv.iter().map(|x| sel(x)).fold(f64::INFINITY, f64::min);
            hi - lo
        };
        IcContractionRig {
            n: rows.len(),
            n_converged: conv.len(),
            members: members.len(),
            spread: if conv.is_empty() {
                None
            } else {
                Some((span(|x| x.g), span(|x| x.q), span(|x| x.v)))
            },
            marched: (match p0.extra {
                          crate::fuel_transient::PointExtra::Triple { g, .. } => g,
                          _ => panic!("rung-71's IC reader needs a five-state trajectory"),
                      },
                      crate::lagged_bleed::valve_of(p0).0,
                      v_at_point(p0)),
            max_iters: conv.iter().map(|x| x.iters).max(),
            rows,
        }
    };
    IcContraction {
        full: one(&full_rig(core, sm, tau, tau_s, v_max, tt4_max, true, true)),
        shared: one(&split_rig(core, sm, tau, tau_s, v_max, tt4_max, true, true)),
    }
}

// ---------------------------------------------------------------------------------------------
// § 4 — THE LEDGER: THREE currencies, one per loop
// ---------------------------------------------------------------------------------------------

/// One cell of [`full_bill`]'s 8-cell ledger, in ALL THREE currencies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FullBillCell {
    /// Rung 66's `phi` violation integral, inherited unchanged.
    pub i: f64,
    /// Rung 67's `Tt4` exceedance integral, inherited unchanged.
    pub e: f64,
    /// Rung 68's `_violation_inc`, the INCIDENCE currency — the third loop's own wall.
    pub m: f64,
    pub min_phi: f64,
    pub max_tt4: f64,
    pub v_hi: f64,
    pub n: usize,
    pub credit_phi: Option<f64>,
    pub credit_tt4: Option<f64>,
    pub credit_inc: Option<f64>,
}

/// One reading per loop — Python's `dict(gov=…, valve=…, stator=…)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FullTriple {
    pub gov: f64,
    pub valve: f64,
    pub stator: f64,
}

/// The `kept` ratios, which are `None` where the solo credit is exactly zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FullKept {
    pub gov: Option<f64>,
    pub valve: Option<f64>,
    pub stator: Option<f64>,
}

/// RUNG 71 § 4's ledger return.
#[derive(Clone, Debug)]
pub struct FullBill {
    /// The eight cells IN PYTHON's ORDER — a `Vec` and not a map, because the order is what makes
    /// a dump reproducible.
    pub cells: Vec<(&'static str, FullBillCell)>,
    pub tt4_max: f64,
    /// **WHICH CELLS MAKE A CURRENCY WORSE THAN THE BARE MARCH.** Rung 69 § 4 measured the
    /// incidence-referenced stator alone driving `min phi_lp` BELOW the bare march's own; that
    /// cell then INFLATES another loop's marginal, because the other loop is repairing damage
    /// rather than delivering protection. Recorded so a `kept` ratio above 1 is read as the
    /// confound it is (rung 58's *check the SUM, not the term*).
    ///
    /// Python's `own_currency` key is deliberately absent beside it: it is a table of CONSTANT
    /// strings that cannot differ between two runs, and rung 70's `phi_lim_source` was dropped for
    /// the same reason. Named here so step 6's oracle records a decision rather than an omission.
    pub degrades: Vec<(&'static str, Vec<&'static str>)>,
    /// **THE SHARPEST SINGLE NUMBER**: the loop that does NOT watch `M_i` protects it better than
    /// the loop that does — § 0's containment, read in the ledger.
    pub inc_credit_valve_alone: Option<f64>,
    pub inc_credit_stator_alone: Option<f64>,
    pub marginal: FullTriple,
    pub alone: FullTriple,
    pub kept: FullKept,
    pub marginal_phi: FullTriple,
    pub marginal_tt4: FullTriple,
    pub marginal_inc: FullTriple,
    pub delivered_phi: Option<f64>,
    pub delivered_tt4: Option<f64>,
    pub delivered_inc: Option<f64>,
}

impl FullBill {
    /// One named cell — a PANIC on an unknown name, because Python raises `KeyError`.
    pub fn cell(&self, name: &str) -> &FullBillCell {
        &self.cells.iter().find(|(k, _)| *k == name)
             .unwrap_or_else(|| panic!("rung-71's ledger has no cell {name:?}")).1
    }
}

/// RUNG 71 § 4 — **THE 8-CELL LEDGER IN THREE CURRENCIES**, one per loop, and the first table in
/// this family that needs one column per loop.
///
/// Rungs 66/68 had one (`I`, rung 66's `phi` violation integral); rung 70 had two (`+ E`, rung
/// 67's `Tt4` exceedance). Here the three loops watch three walls, so rung 68's `_violation_inc`
/// joins them as the incidence currency. All three are INHERITED unchanged, so this table
/// DIFFERENCES against rungs 66/67/68/70 rather than resembling them.
///
/// # THE PREDICTION UNDER TEST IS RUNG 70 § 5's LAW AT ITS ZERO-SHARING CORNER
///
/// *A loop is eroded by the loops it shares a constraint with, and by no others.* Rung 70 measured
/// each `phi` loop keeping a small fraction of its solo credit while the governor kept ~100 % of
/// its own. Here NO two loops share, so every loop's MARGINAL contribution should be ~100 % of its
/// SOLO one — in its own currency, which is the only place the question is even well posed (rung
/// 53: a margin is a DISTANCE, so a credit without its wall is meaningless).
///
/// **AND THE TWO READINGS MUST BE QUOTED TOGETHER.** § 0 confines the stator to the valve's lag, so
/// it can keep 100 % of a SMALL credit; reporting the ratio without the absolute integral, or the
/// reverse, would each mislead in a different direction.
///
/// # `v_hi` IS THE ONE PLACE A NON-FIVE-STATE TRAJECTORY IS REACHABLE
///
/// The `bare`, `G`, `V` and `GV` cells march with the stator disarmed, so their points never
/// recorded `v` and Python's `p.get("v", 0.0)` takes its fallback. [`v_or_zero`] is that fallback;
/// [`v_at_point`] would panic on four of the eight cells.
#[allow(clippy::too_many_arguments)]
pub fn full_bill(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, r: f64, s_settle: f64, ds: f64, tau: f64, tau_gov: f64, tau_s: f64, v_max: f64,
) -> FullBill {
    let mut cells: Vec<(&'static str, FullBillCell)> = Vec::new();
    for (name, valve, stator, gov) in [("bare", false, false, false),
                                       ("G", false, false, true),
                                       ("V", true, false, false),
                                       ("S", false, true, false),
                                       ("GV", true, false, true),
                                       ("GS", false, true, true),
                                       ("VS", true, true, false),
                                       ("GVS", true, true, true)] {
        let m = full_rig(core, sm, tau, tau_s, v_max, tt4_max, valve, stator);
        let leg = StatorLeg { accel: None, surge: None,
                              tt4_max: if gov { Some(tt4_max) } else { None } };
        let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
        let (traj, _) = m.stator_march_scoped(
            flight, &ramp, None, &leg,
            &MarchScope { tau_gov: if gov { Some(tau_gov) } else { None },
                          ..MarchScope::DEFAULT });
        // BUILT PER CELL, as Python builds them — off the RECEIVER's design map and never `m`'s,
        // and never read off `m.stator_inc.m_lim` (four of the eight cells carry no stator at all).
        let phi_lim = (1.0 + sm) * core.arming().map_lp_design.phi_surge;
        let t_c = core.arming().map_lp_design.tan_beta1_crit();
        let m_lim = t_c - 1.0 / phi_lim;
        cells.push((name, FullBillCell {
            i: violation(&traj, phi_lim, r),
            e: exceed(&traj, tt4_max, r),
            m: violation_inc(&traj, m_lim, t_c, r),
            min_phi: opt_fold(traj.iter().map(|p| p.phi_lp), f64::min)
                         .expect("rung-71's ledger marches at least one point"),
            max_tt4: opt_fold(traj.iter().map(|p| p.tt4), f64::max)
                         .expect("rung-71's ledger marches at least one point"),
            v_hi: opt_fold(traj.iter().map(v_or_zero), f64::max).unwrap_or(0.0),
            n: traj.len(),
            credit_phi: None,
            credit_tt4: None,
            credit_inc: None,
        }));
    }
    let base = cells[0].1;
    for (_, c) in cells.iter_mut() {
        c.credit_phi = if base.i > 0.0 { Some(1.0 - c.i / base.i) } else { None };
        c.credit_tt4 = if base.e > 0.0 { Some(1.0 - c.e / base.e) } else { None };
        c.credit_inc = if base.m > 0.0 { Some(1.0 - c.m / base.m) } else { None };
    }
    let at = |name: &str| -> FullBillCell {
        cells.iter().find(|(k, _)| *k == name)
             .unwrap_or_else(|| panic!("rung-71's ledger has no cell {name:?}")).1
    };
    // Python's `own` and `solo`: each loop in ITS OWN currency, against the cell that omits it.
    let (gvs, vs, gs, gv) = (at("GVS"), at("VS"), at("GS"), at("GV"));
    let (g_solo, v_solo, s_solo) = (at("G"), at("V"), at("S"));
    let marginal = FullTriple { gov: vs.e - gvs.e, valve: gs.i - gvs.i, stator: gv.m - gvs.m };
    let alone = FullTriple { gov: base.e - g_solo.e, valve: base.i - v_solo.i,
                             stator: base.m - s_solo.m };
    let kept = FullKept {
        gov: if alone.gov != 0.0 { Some(marginal.gov / alone.gov) } else { None },
        valve: if alone.valve != 0.0 { Some(marginal.valve / alone.valve) } else { None },
        stator: if alone.stator != 0.0 { Some(marginal.stator / alone.stator) } else { None },
    };
    let degrades: Vec<(&'static str, Vec<&'static str>)> = cells.iter()
        .filter(|(n, _)| *n != "bare")
        .map(|(n, c)| {
            let mut worse: Vec<&'static str> = Vec::new();
            if c.i > base.i * (1.0 + 1e-12) {
                worse.push("I");
            }
            if c.e > base.e * (1.0 + 1e-12) {
                worse.push("E");
            }
            if c.m > base.m * (1.0 + 1e-12) {
                worse.push("M");
            }
            (*n, worse)
        })
        .collect();
    FullBill {
        tt4_max,
        degrades,
        inc_credit_valve_alone: v_solo.credit_inc,
        inc_credit_stator_alone: s_solo.credit_inc,
        marginal,
        alone,
        kept,
        // The SAME three "without" cells, read in each currency in turn — Python's three
        // `marginal_*` dicts, whose keys are the LOOPS and whose values cross the currencies.
        marginal_phi: FullTriple { gov: vs.i - gvs.i, valve: gs.i - gvs.i,
                                   stator: gv.i - gvs.i },
        marginal_tt4: FullTriple { gov: vs.e - gvs.e, valve: gs.e - gvs.e,
                                   stator: gv.e - gvs.e },
        marginal_inc: FullTriple { gov: vs.m - gvs.m, valve: gs.m - gvs.m,
                                   stator: gv.m - gvs.m },
        delivered_phi: gvs.credit_phi,
        delivered_tt4: gvs.credit_tt4,
        delivered_inc: gvs.credit_inc,
        cells,
    }
}

// ---------------------------------------------------------------------------------------------
// COUNTERS — the reduce arm and the rig's bare set point are invisible to every value key
// ---------------------------------------------------------------------------------------------
//
// Two things this rung does cannot be reached from a float a reader can print:
//
// * **THE REDUCE.** `integrate_fuel` hands back to rung 70 on `tau_gov is None or stator_inc is
//   None or not lagged_stator()`, and a reduce arm then emits rung 67/68/69/70's numbers BY
//   CONSTRUCTION. That is the contract, so agreement proves nothing about WHICH body ran.
// * **THE RIG's BARE, PERMANENT SET.** `full_rig` writes `_gov_max` on a machine `at_lever` has
//   just built. Every reader here stands on it, and the counter is what shows a march measured a
//   rung-71 plant rather than a governor-less one.
//
// These are what `slice_ac_dispatch.rs` reads at step 7.

thread_local! {
    static INTEGRATE71_REDUCED: Cell<u64> = const { Cell::new(0) };
    static FULL_RIG_CALLS: Cell<u64> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

/// What the counters above hold. [`Census70`](crate::cross_split::Census70)'s sibling, and
/// deliberately NOT an extension of it: the two rungs' reduce arms take DIFFERENT tests one call
/// apart, and a shared counter could not tell which of them fired.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census71 {
    pub integrate_reduced: u64,
    pub full_rig_calls: u64,
}

impl Census71 {
    pub fn read() -> Self {
        Census71 {
            integrate_reduced: INTEGRATE71_REDUCED.with(Cell::get),
            full_rig_calls: FULL_RIG_CALLS.with(Cell::get),
        }
    }

    pub fn reset() {
        INTEGRATE71_REDUCED.with(|x| x.set(0));
        FULL_RIG_CALLS.with(|x| x.set(0));
    }
}
