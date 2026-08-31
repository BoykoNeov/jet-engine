//! RUNG 72 — **TWO LOOPS ON ONE ACTUATOR.** `SharedActuatorTransient`, slice AD.
//!
//! Rung 52's `phi` fuel leg armed BESIDE rung 47's `Tt4` governor, so two limiters drive the SAME
//! actuator. Six states, four clocks, four loops, three actuators. **A shared actuator adds a
//! SWITCH BETWEEN PLANTS, not a loop**: min-select makes authority exclusive, the masked leg's
//! column is `(-1, 0, 0, 0)`, and this one plant IS rung 68, 69, 70 or 71 at every instant plus a
//! free pole at the masked leg's own clock.
//!
//! # WHAT SLICE AD ADDS — **THREE CELLS AND TWO SWAPS**
//!
//! | | name | why |
//! |---|---|---|
//! | CELL | [`reference`](crate::three_loop::TripleHooks::reference) | 2 definers (72, 73) |
//! | CELL | [`rk4_floor_shared`](crate::three_loop::TripleHooks::rk4_floor_shared) | 3 definers (72, 73, 74) |
//! | CELL | [`shared_rig`](crate::three_loop::TripleHooks::shared_rig) | **8** definers (72–80) |
//! | swap | `at_lever` | [`R72`] |
//! | swap | `integrate_fuel` | [`R72_FUEL`] |
//!
//! **The ADD column of § 5.19 (x) said 3, and this is the first back-half slice where it measures
//! right.** AC's said 1 and measured 0; § 5.19 (xi).1 recorded `_closer` listed as a cell while
//! being defined exactly once. So it was checked three independent ways before a field was
//! written — definition count, a caller's existence, and a behavioural-vs-pure-forward AST diff —
//! and the phase-wide substitutability sweep of § 5.27 (x) contains none of these three names.
//!
//! **AND THE PREDICTION THAT `rk4_floor_shared` WOULD FALL OUT WAS REFUTED BY COUNT.** It is a
//! `@staticmethod` whose name differs from rung 68's `_rk4_floor`, which makes it look like rung
//! 70's `_rk4_floor_split` and rung 71's `_rk4_floor_full` — both plain functions, both defined
//! exactly once, neither a cell. This one is defined **three** times.
//!
//! # TWO OF THE THREE CELLS ARE INVISIBLE TO EVERY VALUE KEY, FOR DIFFERENT REASONS
//!
//! [`reference`](crate::three_loop::TripleHooks::reference) is `return req` at this rung —
//! measured the **bitwise identity on 195 278 of 195 278 calls** over the whole shipped suite. And
//! rung 72 is its FIRST definer, so there is no parent pointer to install either. Both halves of
//! the usual dispatch recipe are unavailable, and its gate is slice AB's declared exception: a
//! sentinel proving the cell is REACHED, with the value break arriving at slice AE.
//!
//! [`rk4_floor_shared`](crate::three_loop::TripleHooks::rk4_floor_shared) is
//! [`rk4_floor`](crate::three_loop::TripleHooks::rk4_floor)'s shape — condition identical across
//! all three rungs, the whole cell being the assertion's prose. **What is new is that the SHIPPED
//! PYTHON GATE cannot see it either**: `tests/test_rung72.py:445` fires the floor under
//! `match=r"FOUR actuator states"`, and that phrase is in rungs 73's and 74's messages too. Rung
//! 69's analogue needle (`"rank TWO"`) is unique to it and does discriminate. The ported gate is
//! written on `rung-72` and `-1/tau_f`, so it is strictly stronger than the source's.
//!
//! # THE LAUNDERING MAP, DECIDED BEFORE THE DISPATCH STEP
//!
//! Slice AC step 7 measured that `at_lever`'s body rebuilds through the cascade builder and
//! installs the **shipped** tables, so an injection into a core is washed out before a rig reader
//! reads anything. [`r72_shared_rig`] calls `core.at_lever(…)` at its third line, so the same
//! mechanism is live here and each cell's honest seat is fixed in advance:
//!
//! | cell | scored on | laundered by |
//! |---|---|---|
//! | `shared_rig` | any of the 5 rig readers — it is dispatched ON the core | — |
//! | `reference` | a **DIRECT** `integrate_fuel` march on the injected core | every `*_rig` reader |
//! | `rk4_floor_shared` | a **DIRECT** march | every `*_rig` reader |
//!
//! # A DOCUMENTATION DEFECT IN THE SOURCE, RECORDED BECAUSE THE PORT CANNOT INHERIT IT
//!
//! Rung 72's Python class docstring documents `t.shared_modes(FLIGHT, 1000., 1400., 1200.,
//! sm=0.4545)`. **That method has zero definitions anywhere in `engine.py`** — 0 `def` sites, 0
//! instance assignments, 0 locals. Swept over all 58 ladder classes, three class docstrings name a
//! method that does not exist on their own class: `restored_plant` (rung 65), `cascade_modes`
//! (rung 66) and `shared_modes` (rung 72). Each is a **renamed reader whose docstring kept the old
//! name** — here, [`shared_cells`]. Rung 73's exactly-parallel sentence names `applied_cells`,
//! which does exist and which three shipped tests call.
//!
//! The port carries none of the three, so it is clean by construction; it is recorded here so a
//! later reader of this module does not go looking for a rung-72 reader that was never written.
//!
//! [`shared_cells`]: https://example.invalid/ "landed at slice AD step 3"

use std::cell::Cell;

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{AsymmetricLag, Floor, FuelTransientHooks, SurgeLimiter};
use crate::limited_bleed::BleedLimiter;
use crate::map::ComponentMap;
use crate::reference_split::StatorIncidenceLimiter;
use crate::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use crate::three_loop::{StatorLimiter, TripleHooks};
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::TwoSpoolTransientHooks;

// ---------------------------------------------------------------------------------------------
// THE DECLARED CONSTANTS — all three of Python's rung-72 class attributes
// ---------------------------------------------------------------------------------------------

/// Python's `_share_law = "max"`. `"max"` is MIN-SELECT and **is** the plant; `"sum"` double-clips
/// and is § 3's isolation instrument.
pub const SHARE_LAW_DEFAULT: &str = "max";

/// Python's `_ref_law = "sched"`. **Rung 73's knob, declared at rung 72 and read by nobody here** —
/// see [`ref_law`](crate::two_spool_transient::TwoSpoolTransientCore::ref_law).
pub const REF_LAW_DEFAULT: &str = "sched";

/// Python's `_ic_order4 = "rqvf"` — rung 70's `g -> q -> v` with the new loop APPENDED.
///
/// **A `const` and not a `Cell`, and that is measured rather than chosen**: Python never assigns
/// `self._ic_order4` anywhere in the ladder (0 sites), where `_ic_order` has a guard and a
/// restore. The `s = 0` fixed points are a curve, so a Gauss–Seidel sweep lands on whichever
/// member its order selects; declaring the order is what makes that a stated sensitivity rather
/// than an accident.
pub const IC_ORDER4_DECLARED: &str = "rqvf";

// ---------------------------------------------------------------------------------------------
// `_share_law` — THE CARRIER'S GUARD
// ---------------------------------------------------------------------------------------------

/// Python's `_with_share(self, law, fn, *a, **kw)` as an RAII guard — **restore-PREVIOUS**, which
/// is what its `finally` does.
///
/// # IT WRITES THE FIELD DIRECTLY, AND THAT IS THE OPPOSITE OF [`RefScope`]
///
/// [`RefScope`] goes through the [`with_ref`](TripleHooks::with_ref) cell because rung 73
/// overrides `_with_ref` to write a **different field** and changes nothing else about the call —
/// so writing `core.ref_.set(prev)` there would work at rung 69 and silently restore the wrong
/// field at rung 73. `_with_share` has no such successor: it is defined **exactly once in the
/// entire ladder** (measured over all 58 classes), so there is no rung for a cell to serve, and
/// adding one would be a mechanism with no reader.
///
/// [`RefScope`]: crate::reference_split::RefScope
pub struct ShareScope<'a> {
    cell: &'a Cell<&'static str>,
    prev: &'static str,
}

impl<'a> ShareScope<'a> {
    /// Set the composition law for as long as the returned guard lives.
    pub fn set(core: &'a ScheduledStatorCore, law: &'static str) -> Self {
        let cell = &core.fuel.inner.share_law;
        let prev = cell.get();
        cell.set(law);
        ShareScope { cell, prev }
    }

    /// What this scope displaced — exposed so a gate can read the restore POLICY and not only its
    /// effect.
    pub fn displaced(&self) -> &'static str {
        self.prev
    }
}

impl Drop for ShareScope<'_> {
    fn drop(&mut self) {
        self.cell.set(self.prev);
    }
}

/// Python's `_applied_clip` — **THE COMPOSITION LAW, in ONE place so no reader can disagree with
/// the march.**
///
/// Not a cell: defined exactly once. The one place the `"max"` / `"sum"` split is spelled, which
/// is why [`r72_integrate_fuel`]'s refusal on any other value is a live guard and not ceremony.
pub fn applied_clip(core: &ScheduledStatorCore, gf: f64, gr: f64) -> f64 {
    if core.fuel.inner.share_law.get() == "max" {
        gf.max(gr)
    } else {
        gf + gr
    }
}

// ---------------------------------------------------------------------------------------------
// THE RIG ARM
// ---------------------------------------------------------------------------------------------

/// The keyword bundle [`shared_rig`](TripleHooks::shared_rig) takes — Python's thirteen
/// parameters as one struct, on [`TripleRigArm`](crate::three_loop::TripleRigArm)'s precedent.
///
/// Two flags more than that one: `inc` selects rung 71's INCIDENCE stator over rung 70's `phi`
/// one, and `gov` decides whether the sibling carries `_gov_max`. Every floor still comes from the
/// same `from_margin(cmap, …, sm)` — comparing the two arms at unequal walls would confound this
/// rung with a set-point offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SharedRigArm {
    pub sm: f64,
    /// The VALVE's clock.
    pub tau: f64,
    /// The STATOR's clock.
    pub tau_s: f64,
    pub v_max: f64,
    /// The governor's set point, applied as a BARE PERMANENT set when `gov`.
    pub tt4_max: f64,
    pub tau_att: f64,
    pub tau_rel: f64,
    /// `false` = rung 70's `phi` stator (TWO constraints); `true` = rung 71's INCIDENCE stator
    /// (THREE).
    pub inc: bool,
    /// Arm rung 52's fuel leg — its `SurgeLimiter` floor and its `AsymmetricLag`.
    pub fuel: bool,
    /// Arm rung 65's lagged valve.
    pub valve: bool,
    /// Arm rung 68's lagged stator.
    pub stator: bool,
    /// Arm rung 47's `Tt4` governor.
    pub gov: bool,
}

impl Default for SharedRigArm {
    /// Python's own defaults: all four loops armed, the `phi` arm, and the six clocks. `sm` and
    /// `tt4_max` have no Python default — every caller passes them — and are zeroed rather than
    /// guessed, [`TripleRigArm`](crate::three_loop::TripleRigArm)'s precedent.
    fn default() -> Self {
        SharedRigArm {
            sm: 0.0,
            tau: 0.05,
            tau_s: 0.05,
            v_max: 0.20,
            tt4_max: 0.0,
            tau_att: 0.05,
            tau_rel: 0.15,
            inc: false,
            fuel: true,
            valve: true,
            stator: true,
            gov: true,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// THE CASCADE BUILDER
// ---------------------------------------------------------------------------------------------

/// Build a rung-72 object, so every sibling re-asserts the whole chain's guards.
pub fn build_shared_actuator_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    crate::reference_split::build_split_family_cascade(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
        &R72_TWO, &R72_STATOR, &R72_FUEL, &R72, &R72_TRIPLE)
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and THREE of them carry something of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 72's lever table — ONE swap, `at_lever`, and the parent it must differ from is **rung
/// 71's**.
///
/// The ELEVENTH instance of the sibling-constructor trap. The signature does not grow: rung 72
/// arms its fourth loop with the `Tt4` governor, which rides on `_gov_max` and `_tau_gov` rather
/// than on an `at_lever` keyword — so the note on
/// [`LeverArm::bleed_lim`](crate::bleed_transient::LeverArm::bleed_lim) (*"9 at rung 69, then
/// stops"*) still holds three rungs later.
pub const R72: LeverHooks = LeverHooks {
    at_lever: r72_at_lever,
    ..crate::full_split::R71
};

/// RUNG 72's `TwoSpoolTransientHooks` — **ZERO cells swapped**, an alias, and its width is pinned
/// by the tripwire named at [`R70_TWO`].
///
/// [`R70_TWO`]: crate::cross_split::R70_TWO
pub const R72_TWO: TwoSpoolTransientHooks = crate::full_split::R71_TWO;

/// RUNG 72's fuel table — ONE swap, `integrate_fuel`: **four arming asserts**, then
/// `_rk4_floor_shared` and this rung's own six-state marcher.
pub const R72_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r72_integrate_fuel,
    ..crate::full_split::R71_FUEL
};

/// RUNG 72's stator table — **ZERO cells swapped**, an alias; width pinned as [`R72_TWO`]'s is.
pub const R72_STATOR: StatorTransientHooks = crate::full_split::R71_STATOR;

/// RUNG 72's third-loop table — **THE THREE NEW CELLS, and none of rung 68/69's ten swapped.**
///
/// Spelled out rather than reached through a `..R71_TRIPLE` spread, and unlike the four aliases
/// above **that is a real property here**: slice AC step 7 measured `TripleHooks` the ONE table
/// type in the crate whose copies go loud when the struct grows — 5 of 5, against 2 or 3 of 10–11
/// for the other four types — and adding these three fields produced exactly 5 `E0063` sites, one
/// per const, and none from the aliases. The tripwire is what made this const's edit compulsory.
pub const R72_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::full_split::R71_TRIPLE.stator_leg,
    lagged_stator: crate::full_split::R71_TRIPLE.lagged_stator,
    clamp_v: crate::full_split::R71_TRIPLE.clamp_v,
    check_v0: crate::full_split::R71_TRIPLE.check_v0,
    rk4_floor: crate::full_split::R71_TRIPLE.rk4_floor,
    solve_v: crate::full_split::R71_TRIPLE.solve_v,
    manifold_v: crate::full_split::R71_TRIPLE.manifold_v,
    triple_laws: crate::full_split::R71_TRIPLE.triple_laws,
    triple_rig: crate::full_split::R71_TRIPLE.triple_rig,
    with_ref: crate::full_split::R71_TRIPLE.with_ref,
    // THE THREE THIS SLICE ADDS.
    reference: r72_reference,
    rk4_floor_shared: r72_rk4_floor_shared,
    shared_rig: r72_shared_rig,
};

// ---------------------------------------------------------------------------------------------
// THE SWAPPED CELLS AND THE THREE NEW ONES — BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 72's `at_lever` — **rung 71's sibling constructor returning a RUNG-72 machine.**
///
/// Eleventh instance of the trap. Hand back the parent's class and every reader measures rung
/// 71's plant — one fuel-side leg, `n = 3` — while reporting the shared actuator's four loops.
/// As at rungs 70 and 71, what makes the swap observable is not this function but the PARENT's
/// `integrate_fuel` refusal, which is why the dispatch gate reads a message rather than a float.
fn r72_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_shared_actuator_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 72's `_reference` — **the identity, and it is a cell anyway.**
///
/// Rung 72's two legs both solve their clip from the SCHEDULED fuel (rung 47's discipline and rung
/// 52's, each verbatim), which is what makes `F_r = R_f = 0` EXACTLY and the block triangular.
/// § 6 of the spec concedes that a leg reading the APPLIED fuel would not, and § 11 names it the
/// sharpest seam; this cell is the seam's one seat, and rung 72 is untouched by its existence.
///
/// **195 278 CALLS, 195 278 RETURNED `req` BITWISE UNCHANGED** — so no value gate at this rung can
/// see this cell, and there is no parent pointer to install because rung 72 is the first definer.
/// Slice AE is where the break lives.
fn r72_reference(_core: &ScheduledStatorCore, req: f64, _g_own: f64, _gf: f64, _gr: f64) -> f64 {
    req
}

/// RUNG 72's `_rk4_floor_shared` — **the floor, re-justified a FIFTH time on a FOURTH argument.**
///
/// Rung 68's `ds*sum(1/tau_i) <= 2` was exact-in-argument there; rung 69 kept it on a complex
/// pair; rung 70 because `min(pair) ~ 0` put the pair back near `-sum`; rung 71 because at full
/// rank the trace is shared three ways. Here the argument is new again: **the masked leg's
/// eigenvalue is EXACTLY `-1/tau_f`** — a decoupled first-order lag — and the other three share
/// the remainder, so no root can exceed the rate sum in magnitude.
///
/// The condition is `ds * rate <= 2.0` in rungs 72, 73 and 74 character for character, so **the
/// message is the entire cell**, and the tokens `rung-72` and `-1/tau_f` are what a gate must read.
/// The shipped Python needle (`"FOUR actuator states"`) is in all three messages and cannot.
fn r72_rk4_floor_shared(ds: f64, rate: f64) {
    assert!(
        ds * rate <= 2.0,
        "rung-72: ds*sum(1/tau_i) = {:.3} is outside the explicit RK4 stability region for the \
         FOUR actuator states (ds = {}). The masked leg contributes a bare pole at -1/tau_f and \
         the other three share what is left of the trace, so the dominant root is below the rate \
         sum -- the inherited constant stays conservative, for a fourth reason. Refine the grid \
         or slow a clock.",
        ds * rate, ds);
}

/// RUNG 72's `_shared_rig` — **ONE constructor for every cell of every table in this rung.**
///
/// A cell may differ from another only by which loops are armed and which coordinate the stator
/// watches (rung 63's lesson, and the reason the credits are differenceable at all). Every floor
/// comes from the same `from_margin(cmap, …, sm)`, and `m_lim = T_c - 1/phi_lim` adds no constant
/// (rung 69 § 10, verbatim).
///
/// **THE `_gov_max` SET IS BARE AND PERMANENT, NOT A SCOPE** — [`split_rig`]'s note, third use:
/// Python writes `m._gov_max = Tt4_max if gov else None` on the freshly built sibling, and a
/// `GovScope` here would restore it on drop and hand back a machine with no set point.
///
/// **AND ITS THIRD LINE CALLS `at_lever`**, which rebuilds through the cascade builder and
/// installs the SHIPPED tables — so an injection into a core's triple table is laundered before
/// any reader downstream of this function sees it (slice AC step 7). That is why this cell is
/// scored on a rig reader and the other two are scored on a direct march.
///
/// [`split_rig`]: crate::cross_split::split_rig
fn r72_shared_rig(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let cmap = core.arming().map_lp_design;
    let b_max = core.fuel.inner.lever.lim.map(|l| l.b_max).unwrap_or(0.10);
    let bl = if arm.valve {
        Some(BleedLimiter::from_margin_tau(&cmap, b_max, arm.sm, Some(arm.tau)))
    } else {
        None
    };
    // `sl` and `si` are EXCLUSIVE and both start `None`, exactly as Python's `sl = si = None`:
    // the arm chooses WHICH coordinate the one stator watches, never how many stators there are.
    let (sl, si) = if arm.stator {
        if arm.inc {
            (None, Some(StatorIncidenceLimiter::from_margin(&cmap, arm.v_max, arm.sm,
                                                            Some(arm.tau_s))))
        } else {
            (Some(StatorLimiter::from_margin(&cmap, arm.v_max, arm.sm, Some(arm.tau_s))), None)
        }
    } else {
        (None, None)
    };
    let m = core.at_lever(&LeverArm {
        bleed_lim: bl, stator_lim: sl, stator_inc: si, ..Default::default()
    });
    // THE BARE, PERMANENT SET — and `None` when the governor is not armed, which is a real
    // assignment and not a skip: the sibling inherits nothing here.
    m.fuel.inner.gov_max.set(if arm.gov { Some(arm.tt4_max) } else { None });
    let surge = if arm.fuel {
        Some(Floor::Phi(SurgeLimiter::from_margin(&cmap, Spool::Lp, arm.sm)))
    } else {
        None
    };
    let lag = if arm.fuel {
        Some(AsymmetricLag::new(arm.tau_att, arm.tau_rel))
    } else {
        None
    };
    (m, surge, lag)
}

/// RUNG 72's `integrate_fuel` — **STEP 2 LANDS THIS BODY.**
///
/// It panics rather than delegating to rung 71's, and the difference matters: delegating would
/// make every rung-72 march silently a rung-71 one — one fuel-side leg where this rung has two —
/// and no value gate could see it, because rung 71's marcher answers every call without raising.
/// That is [`NO_TRIPLE`](crate::three_loop::NO_TRIPLE)'s stated reason applied to a swap instead
/// of a cell.
#[allow(clippy::too_many_arguments)]
fn r72_integrate_fuel(
    _core: &crate::fuel_transient::FuelTransientCore,
    _flight: &FlightCondition,
    _sched: &dyn Fn(f64) -> f64,
    _nu0: (f64, f64),
    _s_end: f64,
    _ds: f64,
    _lim: &crate::fuel_transient::FuelLimiters<'_>,
) -> Vec<crate::fuel_transient::FuelPoint> {
    unimplemented!(
        "rung-72 `integrate_fuel` lands at slice AD step 2. It panics rather than delegating to \
         rung 71's, because a delegation would march ONE fuel-side leg where this rung has two \
         and no value gate could see the difference.");
}
