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
use crate::fuel_transient::{
    Authority, AsymmetricLag, Floor, FuelInstant, FuelLimiters, FuelTransientHooks,
    PointExtra, SurgeLimiter,
};
use crate::gas::Abort;
use crate::limited_bleed::Regime;
use crate::three_loop::{closer_b, closer_v};
use crate::two_spool_transient::{MarchedBleed, MarchedStator};
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
fn r72_reference(
    _core: &crate::two_spool_transient::TwoSpoolTransientCore, req: f64, _g_own: f64,
    _gf: f64, _gr: f64,
) -> f64 {
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

/// The thirteen values rung 72's derivative returns — Python's 13-tuple, named.
///
/// Two more than [`CrossTripleDer`](crate::cross_split) because the fuel leg's clip is a SIXTH
/// state and its requirement is read beside the governor's: `dgf`/`dgr` where rung 70 has one
/// `dg`, and `rf`/`rr` where it has one `req`.
struct SharedDer {
    da: f64,
    dh: f64,
    dgf: f64,
    dgr: f64,
    dq: f64,
    dv: f64,
    mf: f64,
    inst: FuelInstant,
    rf: f64,
    rr: f64,
    cmd: f64,
    vcmd: f64,
    /// `None` where no stator is armed — Python's `(0, None)`, and the reason
    /// [`PointExtra::Shared`]'s field is an `Option`.
    vreg: Option<Regime>,
}

/// RUNG 72's `integrate_fuel` — **the entry test, four guards, and the marcher.**
///
/// # THE STATOR IS NOT PART OF THE ENTRY TEST, AND THAT IS DELIBERATE
///
/// The SHARED ACTUATOR is this rung's subject, so the two fuel-side legs together are what it
/// owns — with a stator (§ 2's four cells) or without (`shared_bill`'s `FG` and `FGV` cells,
/// which have **no inherited home at all**: rung 52's own integrator refuses `lag` beside
/// `tau_gov` in so many words, and rung 71's guard B refuses exactly this arming). Gating entry on
/// a stator would leave those two cells unmarchable and the 16-cell ledger with a hole precisely
/// where the fourth loop is alone.
///
/// # AND EVERY INHERITED ARM LEAVES THROUGH THE IMMEDIATE PARENT's TABLE
///
/// `super()` from this class is rung 71, so the reduce goes through [`R71_FUEL`] and not through a
/// grandparent spelling that is only ACCIDENTALLY the same pointer today — rung 71's own note,
/// applied one rung on.
///
/// [`PointExtra::Shared`]: crate::fuel_transient::PointExtra::Shared
/// [`R71_FUEL`]: crate::full_split::R71_FUEL
fn r72_integrate_fuel(
    ft: &crate::fuel_transient::FuelTransientCore, flight: &FlightCondition,
    fuel_schedule: &dyn Fn(f64) -> f64, nu0: (f64, f64), s_end: f64, ds: f64,
    lim: &FuelLimiters<'_>,
) -> Vec<crate::fuel_transient::FuelPoint> {
    let lag = lim.lag.or_else(|| ft.inner.lag.get());
    // RUNG 67's clock rides on an instance attribute and `_stator_march` does not forward it as a
    // keyword (rung 68's note, inherited through rungs 70/71), so reading only the argument would
    // let a rung-72 march silently become a rung-68/69 one.
    let tau_gov = lim.tau_gov.or_else(|| ft.inner.tau_gov.get());
    let has_fuel = lag.is_some() && (lim.accel.is_some() || lim.floor().is_some());
    if tau_gov.is_none() || !has_fuel {
        return (crate::full_split::R71_FUEL.integrate_fuel)(
            ft, flight, fuel_schedule, nu0, s_end, ds,
            &FuelLimiters { tau_gov, lag, ..lim.clone() });
    }
    let tt4_max = lim.tt4_max;
    assert!(tt4_max.is_some(),
            "rung-72: `tau_gov` without `Tt4_max` is a governor with no set point. It would \
             march as rung 68/69 -- ONE fuel-side leg -- while every reader reported the shared \
             actuator (rungs 70/71's assert, inherited word for word).");
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-72: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger. All four legs here pin their own (rung 68's \
             argument, verbatim through rungs 70/71).");
    assert!(ft.inner.lever.lim.is_none() || crate::lagged_bleed::lagged(&ft.inner),
            "rung-72: an INSTANTANEOUS valve beside lagged fuel-side legs is not a control but a \
             different plant (rung 65 called the instantaneous limit singular, rung 66 refused \
             the comparison for that reason). Give the valve a `tau` or leave it out.");
    let share_law = ft.inner.share_law.get();
    assert!(share_law == "max" || share_law == "sum",
            "rung-72: the composition law on the SHARED actuator is this rung's one modelling \
             decision and it is DECLARED; got {share_law:?}. 'max' is MIN-SELECT (the plant); \
             'sum' double-clips and is s 3's isolation instrument, never the plant.");
    r72_integrate_fuel_shared(
        ft, flight, fuel_schedule, nu0, s_end, ds, lim.freeze,
        tt4_max.expect("asserted above"), tau_gov.expect("the entry test returned if None"),
        lim.accel, lim.floor(), &lag.expect("has_fuel"))
}

/// RUNG 72's MARCH — **rung 70/71's five-state integrator with rung 52's fuel clip as a SIXTH
/// state, and the two clips composed on ONE actuator.**
///
/// **IT IS A SIBLING, NOT AN EDIT, AND HERE THAT IS FORCED.** Rung 71 could re-enter its parent's
/// march because nothing was added; a STATE is genuinely added here, so rung 71's *reuse, do not
/// copy* argument does not carry and rungs 68/69/70's precedent does.
///
/// # THREE THINGS DIFFER FROM RUNG 70's MARCH, AND ONLY THE FIRST IS NEW
///
/// * **`mf = mf_sched - applied_clip(gf, gr)`** — ONE call, so the plant and every reader compose
///   the two clips the same way or neither does.
/// * **BOTH `required` closures solve from the SCHEDULED fuel** — rung 47's discipline and rung
///   52's, each verbatim — so neither leg's bracket is perturbed by the other's clip and their
///   mutual cross-gains are structurally ZERO. That is a property of the two INHERITED laws, not a
///   modelling choice made here.
/// * **`Tt4_max` reaches the plant ONLY through the governor's state.** Rung 52's unlagged
///   min-select on top would clip twice and hold the redline with an instrument that is not the
///   loop under study — rung 70's note, with more force now that a second lagged leg is present.
///
/// # THE FOUR-WAY JOINT INITIAL CONDITION, AND A NEW WAY FOR IT TO FAIL
///
/// Rungs 68/70/71 sweep `g -> q -> v`; the new loop is APPENDED (`r -> q -> v -> f`), so the
/// rung-70/71 arm is reached unchanged and the fuel leg takes up only what the triple leaves.
/// **Under MIN-SELECT the sweep can CYCLE**: the fuel leg takes authority, which changes the plant
/// the governor solves against, which hands authority back. That is a FINDING about the
/// composition law and it is reported — the residual, the order and both clips — never repaired by
/// raising the cap.
///
/// **THE STATE FLOOR BELONGS IN THE SWEEP, NOT ONLY IN THE MARCH.** The march floors both clips at
/// zero after every step, so the settled state the sweep solves for must respect the same physical
/// stop. It is a NO-OP at this rung — both `required` closures already return `max(0, ·)` — and it
/// is load-bearing at rung 73, whose hook returns an INCREMENT that can be negative.
#[allow(clippy::too_many_arguments)]
fn r72_integrate_fuel_shared(
    ft: &crate::fuel_transient::FuelTransientCore, flight: &FlightCondition,
    fuel_schedule: &dyn Fn(f64) -> f64, nu0: (f64, f64), s_end: f64, ds: f64,
    freeze: Option<Spool>, tt4_max: f64, tau_gov: f64,
    accel: Option<&crate::fuel_transient::AccelSchedule>, surge: Option<Floor>,
    lag: &AsymmetricLag,
) -> Vec<crate::fuel_transient::FuelPoint> {
    let has_v = ft.inner.lagged_stator();
    let lim_s = if has_v { ft.inner.stator_leg() } else { None };
    let tau_s = lim_s.and_then(|l| l.tau);
    let has_q = crate::lagged_bleed::lagged(&ft.inner);
    let tau_q = if has_q {
        ft.inner.lever.lim.expect("has_q").tau
    } else {
        None
    };
    // PYTHON's OWN SUMMATION ORDER: governor, then the fuel lag, then valve, then stator.
    (ft.inner.triple_hooks.rk4_floor_shared)(
        ds,
        1.0 / tau_gov + 1.0 / lag.tau_att.min(lag.tau_rel)
            + (if has_q { 1.0 / tau_q.expect("has_q") } else { 0.0 })
            + (if has_v { 1.0 / tau_s.expect("has_v") } else { 0.0 }));
    let (tt2, pt2, _) = ft.inner.inlet(flight);

    // THE VALVE law — rungs 68/70/71's, verbatim.
    let command = |a: f64, h: f64, mf: f64, v: f64| -> Result<f64, Abort> {
        if !has_q {
            return Ok(0.0);
        }
        let _sv = MarchedStator::set(&ft.inner, v);
        let bl = ft.inner.lever.lim.expect("has_q");
        Ok(crate::limited_bleed::r64_solve_b(&bl, closer_b(ft, a, h, mf, tt2, pt2))?.1)
    };

    // THE STATOR law — rungs 68/70/71's, verbatim, plus Python's stator-less constant `(0, None)`.
    let stator = |a: f64, h: f64, mf: f64, q: f64| -> Result<(f64, Option<Regime>), Abort> {
        if !has_v {
            return Ok((0.0, None));
        }
        let _sb = MarchedBleed::set(&ft.inner, q);
        let (_, v, reg) = ft.inner.solve_v(&closer_v(ft, a, h, mf, tt2, pt2))?;
        Ok((v, Some(reg)))
    };

    // RUNG 52's leg — rung 68's `required`, verbatim including its `max(0, ·)` kink.
    let required_fuel = |a: f64, h: f64, q: f64, v: f64, mf_sched: f64| -> Result<f64, Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let _sv = MarchedStator::set(&ft.inner, v);
        let mut caps: Vec<f64> = Vec::new();
        if let Some(acc) = accel {
            caps.push(ft.try_sched_fuel(flight, a, h, mf_sched, acc)?);
        }
        if let Some(fl) = surge.as_ref() {
            caps.push(ft.try_surge_fuel(flight, a, h, mf_sched, fl)?);
        }
        if caps.is_empty() {
            return Ok(0.0);
        }
        // Python's `min(caps)` over a list built in THIS order — `min` on ties returns the FIRST,
        // and `f64::min` is not that function, so the fold is written the way Python folds.
        let mut lo = caps[0];
        for &c in &caps[1..] {
            if c < lo {
                lo = c;
            }
        }
        Ok(0.0f64.max(mf_sched - lo))
    };

    // RUNG 47's clip — rung 70's `required`, verbatim.
    let required_gov = |a: f64, h: f64, q: f64, v: f64, mf_sched: f64| -> Result<f64, Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let _sv = MarchedStator::set(&ft.inner, v);
        let i = ft.try_instant_fuel(flight, a, h, mf_sched)?;
        if i.base.tt4 <= tt4_max {
            return Ok(0.0);
        }
        Ok(0.0f64.max(mf_sched - ft.try_topping_fuel(flight, a, h, tt4_max, mf_sched)?))
    };

    let core_ref = |req: f64, g_own: f64, gf: f64, gr: f64| -> f64 {
        // THROUGH THE CELL, never inlined: rung 72's body is `return req` and rung 73's is not.
        (ft.inner.triple_hooks.reference)(&ft.inner, req, g_own, gf, gr)
    };

    let der = |a: f64, h: f64, gf: f64, gr: f64, q: f64, v: f64, s: f64|
     -> Result<SharedDer, Abort> {
        let mf_sched = fuel_schedule(s);
        let rf = core_ref(required_fuel(a, h, q, v, mf_sched)?, gf, gf, gr);
        let rr = core_ref(required_gov(a, h, q, v, mf_sched)?, gr, gf, gr);
        let mf = 1e-9f64.max(mf_sched - applied_clip_core(&ft.inner, gf, gr));
        let inst = {
            let _sb = MarchedBleed::set(&ft.inner, q);
            let _sv = MarchedStator::set(&ft.inner, v);
            ft.try_instant_fuel(flight, a, h, mf)?
        };
        let cmd = command(a, h, mf, v)?;
        let (vcmd, vreg) = stator(a, h, mf, q)?;
        let da = if freeze == Some(Spool::Lp) { 0.0 } else { inst.base.phi_lp_dot / ft.rho() };
        let dh = if freeze == Some(Spool::Hp) { 0.0 } else { inst.base.phi_hp_dot };
        Ok(SharedDer {
            da, dh,
            dgf: (rf - gf) / lag.tau(rf, gf),
            dgr: (rr - gr) / tau_gov,
            dq: if has_q { (cmd - q) / tau_q.expect("has_q") } else { 0.0 },
            dv: if has_v { (vcmd - v) / tau_s.expect("has_v") } else { 0.0 },
            mf, inst, rf, rr, cmd, vcmd, vreg,
        })
    };

    // --- THE JOINT INITIAL CONDITION: four-way, and the ORDER IS DECLARED ----------------------
    let (mut a, mut h) = nu0;
    let mf0 = fuel_schedule(0.0);
    let v0 = ft.inner.v0.get();
    if let (Some(x), Some(l)) = (v0, lim_s) {
        ft.inner.check_v0(x, &l);
    }
    // Python raises out of the whole method here — the initial solves sit BEFORE the loop's `try`.
    let raise = |e: Abort| -> ! { panic!("{}", e.0) };
    let (mut gf, mut gr) = (0.0f64, 0.0f64);
    let mut q = command(a, h, mf0, 0.0).unwrap_or_else(|e| raise(e));
    let mut v = if v0.is_some() && has_v { v0.expect("is_some") } else { 0.0 };
    let b0 = ft.inner.b0.get();
    if let Some(x) = b0 {
        q = x;
    }
    let order = IC_ORDER4_DECLARED;
    assert!({
                let mut cs: Vec<char> = order.chars().collect();
                cs.sort_unstable();
                cs == ['f', 'q', 'r', 'v']
            },
            "rung-72 ic_order4 is a permutation of 'frqv'; got {order:?}");
    let mut res = f64::INFINITY;
    let mut its = 0usize;
    for i in 1..=60usize {
        its = i;
        let (mut gfn, mut grn, mut qn, mut vn) = (gf, gr, q, v);
        for k in order.chars() {
            match k {
                'f' => {
                    gfn = 0.0f64.max(core_ref(
                        required_fuel(a, h, qn, vn, mf0).unwrap_or_else(|e| raise(e)),
                        gfn, gfn, grn));
                }
                'r' => {
                    grn = 0.0f64.max(core_ref(
                        required_gov(a, h, qn, vn, mf0).unwrap_or_else(|e| raise(e)),
                        grn, gfn, grn));
                }
                'q' => {
                    if b0.is_none() {
                        qn = command(a, h,
                                     1e-9f64.max(mf0 - applied_clip_core(&ft.inner, gfn, grn)),
                                     vn)
                            .unwrap_or_else(|e| raise(e));
                    }
                }
                'v' => {
                    if v0.is_none() && has_v {
                        vn = stator(a, h,
                                    1e-9f64.max(mf0 - applied_clip_core(&ft.inner, gfn, grn)),
                                    qn)
                            .unwrap_or_else(|e| raise(e)).0;
                    }
                }
                _ => unreachable!("the permutation assert above admits only f/q/r/v"),
            }
        }
        // Python's `max(abs(n[i] - x) for i, x in enumerate((gf, gr, q, v)))` — the tuple order is
        // `(gf, gr, q, v)` and NOT the sweep order, which is `_ic_order4`'s.
        res = py_max4((gfn - gf).abs(), (grn - gr).abs(), (qn - q).abs(), (vn - v).abs());
        gf = gfn;
        gr = grn;
        q = qn;
        v = vn;
        if res <= 1e-12 {
            break;
        }
    }
    assert!(res <= 1e-9,
            "rung-72: the joint initial condition did not converge (residual {res:.3e} after \
             {its} iterations) in order {order:?}, at gf = {gf:.6e}, gr = {gr:.6e}. Two loops \
             share the fuel actuator, so a sweep under MIN-SELECT can cycle between 'the fuel \
             leg holds it' and 'the governor holds it'. That is a FINDING about the composition \
             law: report the state, the order and both clips; do not raise the cap.");

    // --- THE RK4 LOOP -------------------------------------------------------------------------
    let share_law = ft.inner.share_law.get();
    let mut pts: Vec<crate::fuel_transient::FuelPoint> = Vec::new();
    let mut s = 0.0f64;
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok(k1) = der(a, h, gf, gr, q, v, s) else { break };
        let clip = applied_clip_core(&ft.inner, gf, gr);
        pts.push(crate::fuel_transient::point(
            s, a, h, &k1.inst, k1.mf, fuel_schedule(s),
            PointExtra::Shared {
                g: clip, required: k1.rf.max(k1.rr), b: q, b_cmd: k1.cmd,
                v, v_cmd: k1.vcmd, v_regime: k1.vreg,
                ic_iters: its, ic_res: res, ic_order: order,
                g_fuel: gf, g_gov: gr, required_fuel: k1.rf, required_gov: k1.rr,
                authority: authority(gf, gr), share_law,
            }));
        let stages = (|| -> Result<[f64; 18], Abort> {
            let k2 = der(a + ds / 2.0 * k1.da, h + ds / 2.0 * k1.dh, gf + ds / 2.0 * k1.dgf,
                         gr + ds / 2.0 * k1.dgr, q + ds / 2.0 * k1.dq, v + ds / 2.0 * k1.dv,
                         s + ds / 2.0)?;
            let k3 = der(a + ds / 2.0 * k2.da, h + ds / 2.0 * k2.dh, gf + ds / 2.0 * k2.dgf,
                         gr + ds / 2.0 * k2.dgr, q + ds / 2.0 * k2.dq, v + ds / 2.0 * k2.dv,
                         s + ds / 2.0)?;
            let k4 = der(a + ds * k3.da, h + ds * k3.dh, gf + ds * k3.dgf, gr + ds * k3.dgr,
                         q + ds * k3.dq, v + ds * k3.dv, s + ds)?;
            Ok([k2.da, k2.dh, k2.dgf, k2.dgr, k2.dq, k2.dv,
                k3.da, k3.dh, k3.dgf, k3.dgr, k3.dq, k3.dv,
                k4.da, k4.dh, k4.dgf, k4.dgr, k4.dq, k4.dv])
        })();
        let Ok([k2a, k2h, k2gf, k2gr, k2q, k2v,
                k3a, k3h, k3gf, k3gr, k3q, k3v,
                k4a, k4h, k4gf, k4gr, k4q, k4v]) = stages else { break };
        a += ds / 6.0 * (k1.da + 2.0 * k2a + 2.0 * k3a + k4a);
        h += ds / 6.0 * (k1.dh + 2.0 * k2h + 2.0 * k3h + k4h);
        gf += ds / 6.0 * (k1.dgf + 2.0 * k2gf + 2.0 * k3gf + k4gf);
        gr += ds / 6.0 * (k1.dgr + 2.0 * k2gr + 2.0 * k3gr + k4gr);
        q += ds / 6.0 * (k1.dq + 2.0 * k2q + 2.0 * k3q + k4q);
        v += ds / 6.0 * (k1.dv + 2.0 * k2v + 2.0 * k3v + k4v);
        // Every position is PHYSICAL (rung 65, verbatim): the actuators' own hardware stops,
        // applied to the STATE and never to a command. BOTH clips are floored at zero — a
        // negative clip is fuel ADDED by a limiter, which no leg here can do.
        if has_q {
            let bmax = ft.inner.lever.lim.expect("has_q").b_max;
            q = bmax.min(0.0f64.max(q));
        }
        if has_v {
            v = ft.inner.clamp_v(v, &lim_s.expect("has_v"));
        }
        gf = 0.0f64.max(gf);
        gr = 0.0f64.max(gr);
        s += ds;
    }
    pts
}

/// Python's `max(...)` over the FOUR-element residual — [`py_max3`]'s sibling, and it exists for
/// the same reason: `f64::max` is not Python's `max`, which returns the FIRST of equal arguments
/// and propagates a NaN the moment one appears rather than swallowing it.
///
/// [`py_max3`]: crate::lagged_bleed::py_max3
fn py_max4(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let mut m = a;
    if b > m { m = b; }
    if c > m { m = c; }
    if d > m { m = d; }
    m
}

/// [`applied_clip`] on the shared core, for the march's own use.
fn applied_clip_core(t: &crate::two_spool_transient::TwoSpoolTransientCore, gf: f64, gr: f64)
 -> f64 {
    if t.share_law.get() == "max" { gf.max(gr) } else { gf + gr }
}

/// Python's `_authority(gf, gr)` — see [`Authority`].
///
/// The `tol` is Python's `1e-12` and it is **inert on every shipped input** (§ 5.28 (iv)): 36 calls
/// land at `|gf - gr| == 0.0`, 36 at `<= tol`, and ZERO in the open interval between. It is ported
/// because Python has it, and it is not gated, because a gate on it could not fail.
pub fn authority(gf: f64, gr: f64) -> Authority {
    const TOL: f64 = 1e-12;
    if gf <= TOL && gr <= TOL {
        Authority::Dormant
    } else if (gf - gr).abs() <= TOL {
        Authority::Tie
    } else if gf > gr {
        Authority::Fuel
    } else {
        Authority::Gov
    }
}
