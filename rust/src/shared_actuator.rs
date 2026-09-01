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
//! # A FOURTH CELL THE PRE-FLIGHT's CENSUS COULD NOT SEE — **AND THE TRAP ITS DEFERRAL LEAVES**
//!
//! [`quad_gains_at`] has **two definers** (rungs 72 and 73), an identical signature and a
//! behavioural override, so it clears every filter § 5.28 (ii) applied except *"a caller exists"* —
//! and it fails that one because **nothing calls it**. It is handed to `_with_share` / `_with_ref`
//! as a bound method at twelve sites across six rungs, so neither `"." + name + "("` nor an AST
//! `Call` node ever matches it. The cell column measures **4**, not 3.
//!
//! No [`TripleHooks`] field is installed, because on the shipped ladder the dispatch is
//! unreachable: all five rung-72 readers are redefined by NOBODY among the twelve descendants, and
//! no rung-73-or-later code calls one. A pointer nothing can select differently is a defence with
//! no reader. Booked to slice **AE** on § 5.27 (x)'s `_legs` precedent.
//!
//! **THE TRAP THAT DEFERRAL LEAVES, WRITTEN DOWN HERE BECAUSE NO GATE CAN SEE IT.** Only rungs 72
//! and 73 define the method, so Python's MRO resolves it to **rung 73's body** for
//! `demand_gains` (74), `split_gains` (80) and `authority_mask` (81) — measured over the live
//! classes, not read off the source. A later slice that wires any of those three readers to
//! [`quad_gains_at`] below is a silent value error, and the port has no value key that would
//! notice. Wire them to rung 73's body, or install the cell at that point.
//!
//! [`shared_cells`]: https://example.invalid/ "landed at slice AD step 3"

use std::cell::Cell;

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, Authority, AsymmetricLag, Floor, FuelInstant, FuelLimiters,
    FuelPoint, FuelTransientHooks, PointExtra, SurgeLimiter,
};
use crate::gas::Abort;
use crate::limited_bleed::Regime;
use crate::three_loop::{closer_b, closer_v};
use crate::two_spool_transient::{MarchedBleed, MarchedStator};
use crate::limited_bleed::BleedLimiter;
use crate::map::ComponentMap;
use crate::reference_split::{
    c_add, c_div, c_is_zero, c_mul, c_powu, c_real, c_sub, opt_fold,
    StatorIncidenceLimiter, C64,
};
use crate::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
    StatorTransientHooks,
};
use crate::three_loop::{LegRegime, StatorLimiter, TripleHooks};
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
    // THE THREE SLICE AD ADDS.
    reference: r72_reference,
    rk4_floor_shared: r72_rk4_floor_shared,
    shared_rig: r72_shared_rig,
    // AND THE FOURTEENTH, ADDED BY SLICE AE STEP 2 — rung 72 is its FIRST DEFINER, so the body is
    // this file's own. Slice AD measured it a cell and booked it forward as unreachable; § 5.29
    // (iv) refuted that by value, which is why the slot exists at all.
    quad_gains_at: quad_gains_at,
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
///
/// `pub(crate)` because RUNG 73's `_reference` needs the SAME spelling: its middle branch is
/// `clip == g_own`, a float-IDENTITY test, so a reader that re-derived the clip through
/// [`applied_clip`]'s `&ScheduledStatorCore` form would be comparing against a second
/// expression of the same algebra. One body, one clip — rung 72's own discipline, and the
/// reason `_applied_clip` is defined exactly once in Python too.
pub(crate) fn applied_clip_core(t: &crate::two_spool_transient::TwoSpoolTransientCore, gf: f64, gr: f64)
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

// ---------------------------------------------------------------------------------------------
// § 1 — THE FOUR LAWS, AND THE TWELVE GAINS
// ---------------------------------------------------------------------------------------------

/// The FOUR control laws of § 1, as closures of the other three states — what [`quad_laws`]
/// returns.
///
/// Each goes through a SHIPPED closure and **none knows the others exist**. That mutual ignorance
/// is what makes their products a MEASUREMENT of § 1's algebra rather than a restatement of it.
///
/// `f` and `r` are rung 68's and rung 70's `required`, verbatim — including the fact that each
/// computes its clip from the SCHEDULED fuel and therefore CANNOT see the other's state. That is
/// not a choice made here; it is the property of the two inherited laws that § 1.1 turns into
/// `F_r = R_f = 0`, and it is DIFFERENCED rather than assumed.
///
/// `c` and `v` are rungs 65/68/69's, and they see the two clips ONLY through [`applied_clip`] — so
/// under MIN-SELECT the masked one reaches them through a function that is FLAT in it.
#[allow(clippy::type_complexity)]
pub struct QuadLaws<'a> {
    /// **F** — rung 52's leg, `(gr, q, v) -> (clip, regime)`. `max(0, .)` has a KINK at its own
    /// dormant edge; the regime label is what the caller filters on.
    pub f: Box<dyn Fn(f64, f64, f64) -> Result<(f64, LegRegime), Abort> + 'a>,
    /// **R** — rung 47's clip, `(gf, q, v) -> (clip, regime)`, with rung 70's own kink.
    pub r: Box<dyn Fn(f64, f64, f64) -> Result<(f64, LegRegime), Abort> + 'a>,
    /// **C** — the VALVE law, `(gf, gr, v) -> (b, regime)`.
    pub c: Box<dyn Fn(f64, f64, f64) -> Result<(f64, Regime), Abort> + 'a>,
    /// **V** — the STATOR law, `(gf, gr, q) -> (v, regime)`.
    pub v: Box<dyn Fn(f64, f64, f64) -> Result<(f64, Regime), Abort> + 'a>,
}

/// RUNG 72's `_quad_laws` — **the four laws, each blind to the other three.**
///
/// # THE `b_state` / `v_state` BOUNDARY IS THE RUNG's TRAP IN ITS FIFTH SHAPE, AND IT GUARDS THE
/// HEADLINE
///
/// Rung 68's table (a law that TRIALS an actuator must not see that actuator's state, and MUST see
/// the other two) is inherited unchanged. What is new is the CONSEQUENCE of losing it here: a fuel
/// leg whose `required` lost the boundary would return `F_q = F_v = 0` and **its row would look
/// exactly like a MASKED one** — so this rung would confirm its own headline through a bug. That
/// is what [`assert_fuel_boundary`] is for, and why it is an assert and not a comment.
///
/// Both guards write `None` on the way out rather than restoring a previous value, which is what
/// Python's `finally: self._b_state, self._v_state = None, None` does — 68 of the ladder's 72
/// reload guards are this shape, and [`MarchedBleed`]'s `Drop` already is.
#[allow(clippy::too_many_arguments)]
pub fn quad_laws<'a>(
    core: &'a ScheduledStatorCore, flight: &'a FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&'a AccelSchedule>, surge: Option<&'a Floor>, tt4_max: f64,
) -> QuadLaws<'a> {
    let ft = &core.fuel;
    let (tt2, pt2, _) = ft.inner.inlet(flight);

    // **F** — RUNG 52's leg. It takes `gr` and ignores it: the two legs each solve from the
    // SCHEDULED fuel, which is precisely the property § 1.1 measures as `F_r == 0` EXACTLY. The
    // argument is named and unused on purpose — wiring it in would delete the measurement.
    let f = move |_gr: f64, q: f64, v: f64| -> Result<(f64, LegRegime), Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let _sv = MarchedStator::set(&ft.inner, v);
        let mut caps: Vec<f64> = Vec::new();
        if let Some(ac) = accel {
            caps.push(ft.try_sched_fuel(flight, a, h, mf_sched, ac)?);
        }
        if let Some(su) = surge {
            caps.push(ft.try_surge_fuel(flight, a, h, mf_sched, su)?);
        }
        let raw = match caps.iter().copied().reduce(f64::min) {
            Some(m) => mf_sched - m,
            None => 0.0,
        };
        Ok((0.0f64.max(raw), if raw > 0.0 { LegRegime::Riding } else { LegRegime::Dormant }))
    };

    // **R** — RUNG 47's clip. `gf` is ignored for the same reason, and that is `R_f == 0`.
    let r = move |_gf: f64, q: f64, v: f64| -> Result<(f64, LegRegime), Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let _sv = MarchedStator::set(&ft.inner, v);
        let i = ft.try_instant_fuel(flight, a, h, mf_sched)?;
        if i.base.tt4 <= tt4_max {
            return Ok((0.0, LegRegime::Dormant));
        }
        let raw = mf_sched - ft.try_topping_fuel(flight, a, h, tt4_max, mf_sched)?;
        Ok((0.0f64.max(raw), if raw > 0.0 { LegRegime::Riding } else { LegRegime::Dormant }))
    };

    // **C** — the VALVE law: it trials `b`, so NO `b_state`, but `v_state = v`. The two clips
    // reach it ONLY through `_applied_clip`, which under `max` is FLAT in the smaller one.
    let c = move |gf: f64, gr: f64, v: f64| -> Result<(f64, Regime), Abort> {
        let _sv = MarchedStator::set(&ft.inner, v);
        let bl = ft.inner.lever.lim.expect("rung-72's valve law on an unfloored machine");
        let clip = applied_clip_core(&ft.inner, gf, gr);
        let (_, b, reg) = crate::limited_bleed::r64_solve_b(
            &bl, closer_b(ft, a, h, 1e-9f64.max(mf_sched - clip), tt2, pt2))?;
        Ok((b, reg))
    };

    // **V** — the STATOR law: the exact mirror, trialling `v` with `b_state = q`.
    let v = move |gf: f64, gr: f64, q: f64| -> Result<(f64, Regime), Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let clip = applied_clip_core(&ft.inner, gf, gr);
        let (_, vv, reg) = ft.inner.solve_v(
            &closer_v(ft, a, h, 1e-9f64.max(mf_sched - clip), tt2, pt2))?;
        Ok((vv, reg))
    };

    QuadLaws { f: Box::new(f), r: Box::new(r), c: Box::new(c), v: Box::new(v) }
}

/// The TWELVE central differences at one trajectory point, plus § 1's four products and the mask
/// leak — Python's `_quad_gains_at` dict, named.
#[derive(Clone, Debug, PartialEq)]
pub struct QuadGains {
    /// Was EVERY perturbed evaluation riding-interior? A dropped point is a COVERAGE CLAIM, so the
    /// caller reports the count rather than filtering silently.
    pub interior: bool,
    /// Which arms were off-regime, in Python's own key order — or the single pseudo-key
    /// `"switch"`, which is [`QuadGains::near_switch`]'s companion and not a regime at all.
    pub off_regime: Vec<&'static str>,
    /// Was the point dropped for straddling the AUTHORITY hand-over rather than for a regime?
    pub near_switch: bool,
    pub s: f64,
    pub v_base: f64,
    /// `None` exactly when the point was dropped — Python's dict has no `authority` key there.
    pub authority: Option<Authority>,
    /// **THE TWO FUEL-SIDE SELF-GAINS, ABSENT AT THIS RUNG.** Python spells them
    /// `gg.get("F_f", 0.0)` / `gg.get("R_r", 0.0)` inside [`jac4`], and rung 72's dict carries
    /// NEITHER key. They are fields defaulting to `0.0` rather than a written `-1/tau_i` diagonal,
    /// because rung 73 § 1.3 weakens exactly this and a diagonal WRITTEN by the instrument would
    /// make the free pole a construction rather than a measurement.
    pub f_f: f64,
    pub r_r: f64,
    pub f_r: f64,
    pub f_q: f64,
    pub f_v: f64,
    pub r_f: f64,
    pub r_q: f64,
    pub r_v: f64,
    pub c_f: f64,
    pub c_r: f64,
    pub c_v: f64,
    pub v_f: f64,
    pub v_r: f64,
    pub v_q: f64,
    /// **THE PAIR PRODUCT AT ITS OPPOSITE CORNER.** Rung 66's two loops on one VARIABLE gave
    /// exactly 1; two loops on one ACTUATOR give exactly 0.
    pub pair_fr: f64,
    pub pair_rc: f64,
    pub pair_cv: f64,
    pub pair_rv: f64,
    /// Which leg `max()` is masking — the OTHER one from [`QuadGains::authority`], and `None`
    /// wherever authority is `Dormant` or `Tie`.
    pub masked: Option<Authority>,
    /// The masked leg's own coupling into the plant — **predicted EXACTLY zero, not small.**
    pub mask_leak: Option<f64>,
    /// **RUNG 73's THREE, AND THEY ARE `None` AT RUNG 72 BECAUSE PYTHON HAS NO SUCH KEY THERE.**
    ///
    /// § 1's BRANCH INDICATOR, measured rather than written: the masked leg's own self-gain
    /// ([`self_masked`](Self::self_masked)), its cross-gain onto the AUTHORITATIVE axis
    /// ([`cross_masked`](Self::cross_masked)), and the holding leg's self-gain
    /// ([`self_live`](Self::self_live)). Under the applied reference these are exactly `+1`, `-1`
    /// and `0`; under rung 72 all three are `0` — but rung 72 does not COMPUTE them, and the
    /// difference between *absent* and *zero* is the whole of § 5.29 (iv)'s 70 vanishing keys.
    /// `None` is therefore Python's missing key and never a value.
    pub self_masked: Option<f64>,
    pub cross_masked: Option<f64>,
    pub self_live: Option<f64>,
}

impl QuadGains {
    /// The dropped-point form — Python's five-key dict, with every gain absent.
    ///
    /// The gains are `NaN` rather than `0.0` so that a reader which forgets to check
    /// [`interior`](QuadGains::interior) produces a NaN rather than a plausible zero — the same
    /// choice [`TripleGains`](crate::three_loop::TripleGains) makes, and for the same reason: at
    /// this rung a plausible zero is *indistinguishable from the headline*.
    pub(crate) fn dropped(s: f64, v_base: f64, off_regime: Vec<&'static str>, near_switch: bool) -> Self {
        QuadGains {
            interior: false,
            off_regime,
            near_switch,
            s,
            v_base,
            authority: None,
            f_f: 0.0,
            r_r: 0.0,
            f_r: f64::NAN,
            f_q: f64::NAN,
            f_v: f64::NAN,
            r_f: f64::NAN,
            r_q: f64::NAN,
            r_v: f64::NAN,
            c_f: f64::NAN,
            c_r: f64::NAN,
            c_v: f64::NAN,
            v_f: f64::NAN,
            v_r: f64::NAN,
            v_q: f64::NAN,
            pair_fr: f64::NAN,
            pair_rc: f64::NAN,
            pair_cv: f64::NAN,
            pair_rv: f64::NAN,
            masked: None,
            mask_leak: None,
            // NOT `NaN`: Python's dropped dict has no such key AT EITHER RUNG, and `f_f`/`r_r`
            // stay `0.0` above for the same reason — `jac4` reads them through `.get(…, 0.0)`.
            self_masked: None,
            cross_masked: None,
            self_live: None,
        }
    }
}

/// RUNG 72's `_jac4` — the 4x4 `J`, rows `(f, r, q, v)`,
/// `J[i][j] = (dcmd_i/dx_j − delta_ij)/tau_i`.
///
/// Built EXPLICITLY rather than through a closed form, because the closed form is what § 1.2 is
/// claiming. `taus` is `(tau_f, tau_g, tau_q, tau_s)`.
///
/// The first two diagonal entries read [`QuadGains::f_f`] / [`QuadGains::r_r`], which are `0.0` at
/// this rung — see their own doc comment for why that is a READ and not a written `-1/tau_i`.
pub fn jac4(gg: &QuadGains, taus: (f64, f64, f64, f64)) -> [[f64; 4]; 4] {
    let (tf, tg, tq, ts) = taus;
    [[(gg.f_f - 1.0) / tf, gg.f_r / tf, gg.f_q / tf, gg.f_v / tf],
     [gg.r_f / tg, (gg.r_r - 1.0) / tg, gg.r_q / tg, gg.r_v / tg],
     [gg.c_f / tq, gg.c_r / tq, -1.0 / tq, gg.c_v / tq],
     [gg.v_f / ts, gg.v_r / ts, gg.v_q / ts, -1.0 / ts]]
}

/// RUNG 72's `_quad_gains_at` — the twelve central differences at one trajectory point, **with TWO
/// filters.**
///
/// REGIME, rung 68's, inherited: every perturbed evaluation is checked, not just the base point,
/// because a base point can be comfortably riding while one arm of a difference has crossed into
/// `dormant` or onto a stop.
///
/// **SWITCH PROXIMITY, AND THIS ONE IS NEW.** A difference in `gf` of step `dg` taken where
/// `|gf − gr| < dg` straddles the `max()` kink and returns the slope of NEITHER branch — the
/// authority hand-over's version of the regime trap, and the regime filter does not catch it,
/// because at the switch BOTH legs are comfortably riding and nowhere near a stop. Points inside
/// `switch_guard * dg` of the hand-over are skipped and COUNTED (rung 68's rule: a dropped point
/// is a coverage claim, never a silent truncation).
///
/// # THE SWITCH FILTER NEVER FIRES ON THE SHIPPED GRID — MEASURED, AND THE FIRST WRITING OF THIS
/// PARAGRAPH CLAIMED OTHERWISE
///
/// Python's condition is `self._share_law == "max" and abs(gf - gr) <= switch_guard * dg`, and
/// this comment first said that dropping the law half *"would silently thin the arm carrying § 3's
/// whole discriminator"*, because [`mask_discriminator`] reads the SUM law at the min-select
/// trajectory's own base points. **That is a mechanism, so it is testable, and it is false.**
/// Deleting the `share_law == "max"` half and re-running the whole step-3 reader dump moves
/// **0 of 3 216 keys** — the filter fires ZERO times under EITHER law, on every shipped arm
/// (`skipped.switch` is 0 in both [`shared_gains`] arms and all four [`shared_cells`] arms), so
/// the guard on it cannot be observed either.
///
/// It is dead, but **not by much**, and the margin is worth having written down because a
/// widened `switch_guard` is the obvious next knob. The closest any sampled riding point comes to
/// the bar, over all ten shipped `(clocks, inc)` arms:
///
/// | arm | sampled | min `\|gf − gr\|` | × the bar |
/// |---|---|---|---|
/// | `md` arm 3, incidence | 38 | 1.63e−6 | **4.08** |
/// | `md` arm 3, `phi` | 49 | 3.44e−6 | 8.61 |
/// | `sc` wide-cell, `phi` | 80 | 7.62e−6 | 19.0 |
/// | … | | | up to 145 |
///
/// So `switch_guard ≈ 16.4` rather than `4.0` would start dropping points. This is rung 69's
/// `n_zero` disclosure with a much thinner margin — 0.6 of a decade, not 3.5 — which is exactly
/// why it is a NUMBER here and not the word "unreachable".
///
/// `manifold = true` puts the stator on the shared manifold before differencing (rung 68's exact
/// statement of the algebra, inherited unchanged), with the APPLIED clip standing in for rung 68's
/// single `g`.
///
/// # AND `manifold = false` IS DEAD AT THIS RUNG — a SECOND unreachable branch, measured
///
/// `_quad_gains_at` is never CALLED; it is passed to `_with_share` / `_with_ref` as a bound method
/// at **twelve** sites over six rungs, and **not one of them supplies a `manifold=` keyword**. So
/// the parameter is `True` on every shipped input and the `else` arm below — which is the ONLY
/// reader of a rung-72 point's live `v` inside this function — is unreachable. The contrast is
/// [`triple_gains_at`](crate::three_loop::triple_gains_at), whose own callers DO pass
/// `manifold=False` at four sites, which is why rung 68 needs both arms and rung 72 does not.
///
/// Ported faithfully and **disclosed, never gated** — § 5.28 (iii)'s rule for the quartic's three
/// dead roots, in its second place in this file.
#[allow(clippy::too_many_arguments)]
pub fn quad_gains_at(
    core: &ScheduledStatorCore, flight: &FlightCondition, p: &FuelPoint,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>, tt4_max: f64,
    dg: f64, dq: f64, dv: f64, manifold: bool, switch_guard: f64,
) -> Result<QuadGains, Abort> {
    let (a, h, mf_sched) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let (gf, gr, q, v_live) = match p.extra {
        PointExtra::Shared { g_fuel, g_gov, b, v, .. } => (g_fuel, g_gov, b, v),
        _ => panic!("rung-72's gains need a SIX-state trajectory: the point carries no \
                     `g_fuel`/`g_gov` pair, so there is no authority to difference across."),
    };
    let laws = quad_laws(core, flight, a, h, mf_sched, accel, surge, tt4_max);
    let v = if manifold {
        // rung 68's `_manifold_v`, with the applied clip standing in for its single `g`
        let vlaw = |g_: f64, q_: f64| (laws.v)(g_, 0.0, q_);
        core.manifold_v(flight, a, h, mf_sched, applied_clip(core, gf, gr), q, &vlaw)?
    } else {
        v_live
    };
    if core.fuel.inner.share_law.get() == "max" && (gf - gr).abs() <= switch_guard * dg {
        return Ok(QuadGains::dropped(p.s, v, vec!["switch"], true));
    }

    // Python builds a dict of twenty-four `(key, value)` pairs in ONE literal, so every evaluation
    // runs BEFORE any regime is inspected. Reproduced as a `Vec` in the same order:
    // short-circuiting on the first off-regime arm would change how many closure calls the plant
    // sees, which is a difference the counters can read even where the floats agree.
    let ev: Vec<(&'static str, f64, bool)> = vec![
        leg4("F+r", (laws.f)(gr + dg, q, v)?),
        leg4("F-r", (laws.f)(gr - dg, q, v)?),
        leg4("F+q", (laws.f)(gr, q + dq, v)?),
        leg4("F-q", (laws.f)(gr, q - dq, v)?),
        leg4("F+v", (laws.f)(gr, q, v + dv)?),
        leg4("F-v", (laws.f)(gr, q, v - dv)?),
        leg4("R+f", (laws.r)(gf + dg, q, v)?),
        leg4("R-f", (laws.r)(gf - dg, q, v)?),
        leg4("R+q", (laws.r)(gf, q + dq, v)?),
        leg4("R-q", (laws.r)(gf, q - dq, v)?),
        leg4("R+v", (laws.r)(gf, q, v + dv)?),
        leg4("R-v", (laws.r)(gf, q, v - dv)?),
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
    let at = |k: &str| ev.iter().find(|(n, _, _)| *n == k).expect("the 24 keys above").1;
    let d = |kp: &str, km: &str, h2: f64| (at(kp) - at(km)) / (2.0 * h2);
    let (f_r, f_q, f_v) = (d("F+r", "F-r", dg), d("F+q", "F-q", dq), d("F+v", "F-v", dv));
    let (r_f, r_q, r_v) = (d("R+f", "R-f", dg), d("R+q", "R-q", dq), d("R+v", "R-v", dv));
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
    Ok(QuadGains {
        interior: true,
        off_regime: Vec::new(),
        near_switch: false,
        s: p.s,
        v_base: v,
        authority: Some(auth),
        f_f: 0.0,
        r_r: 0.0,
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
        // RUNG 72's DICT CARRIES NO SUCH KEY — see their doc comment. Writing `Some(0.0)` here
        // would be true of rung 72's plant and would delete the discrete half of § 5.29 (iv)'s
        // witness, which is 70 keys that are ABSENT rather than zero.
        self_masked: None,
        cross_masked: None,
        self_live: None,
    })
}

pub(crate) fn leg4(k: &'static str, x: (f64, LegRegime)) -> (&'static str, f64, bool) {
    (k, x.0, x.1 == LegRegime::Riding)
}

pub(crate) fn reg4(k: &'static str, x: (f64, Regime)) -> (&'static str, f64, bool) {
    (k, x.0, x.1 == Regime::Riding)
}

/// RUNG 72's `_riding4` — trajectory points where **ALL FOUR loops are live and strictly
/// interior.**
///
/// Rung 68's [`riding`](crate::three_loop::riding) with the fourth leg added; the stator is still
/// filtered on the REGIME LABEL and never on a float comparison against a stop.
///
/// A point with no stator (`v_regime: None`) is not RIDING a stator it does not have — `false`
/// here is the same answer Python gives for `p.get("v_regime") == "riding"` on `None`, not a
/// fallback.
pub fn riding4(traj: &[FuelPoint], b_max: f64) -> Vec<FuelPoint> {
    traj.iter()
        .filter(|p| match p.extra {
            PointExtra::Shared { required_fuel, required_gov, b_cmd, v_regime: Some(vr), .. } =>
                required_fuel > 0.0 && required_gov > 0.0
                    && 0.0 < b_cmd && b_cmd < b_max && vr == Regime::Riding,
            // A stator-less rung-72 point is not RIDING a stator it does not have: `false` is
            // the answer Python's `p.get("v_regime") == "riding"` gives on `None`, not a fallback.
            PointExtra::Shared { v_regime: None, .. } => false,
            // **AND EVERY OTHER VARIANT REFUSES, WHICH IS THE OPPOSITE OF A FILTER's USUAL ARM.**
            // Python reads `p["required_fuel"]`, `p["required_gov"]` and `p["b_cmd"]` with a BARE
            // index — only `v_regime` goes through `.get` — so a non-six-state point raises
            // `KeyError` here exactly as it does in `authority_law`'s census. Slice AD step 2
            // measured `false`-in-a-filter to be the QUIETEST of the three silent-fallback shapes
            // (the reader computes over an empty set and reports perfect tracking), and this
            // function is a filter, so the two sites get the SAME treatment and not opposite ones.
            _ => panic!("rung-72's `_riding4` reads `required_fuel`/`required_gov`/`b_cmd` with a                          bare index, so a trajectory that is not this rung's raises rather than                          filtering to nothing. An empty riding set reports PERFECT tracking and                          every statistic downstream of it is then computed over nothing."),
        })
        .copied()
        .collect()
}

/// The output of [`assert_fuel_boundary`] — Python's `dict(s=…, live=…, dead=…)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundaryCheck {
    pub s: f64,
    pub live_f_q: f64,
    pub live_f_v: f64,
    pub live_r_q: f64,
    pub live_r_v: f64,
    pub dead_f_q: f64,
    pub dead_f_v: f64,
    pub dead_r_q: f64,
    pub dead_r_v: f64,
}

/// RUNG 72's `_assert_fuel_boundary` — **rung 70's boundary check FOR BOTH FUEL LEGS, and here it
/// guards the headline.**
///
/// Rung 70 measures the governor's cross-gains against a deliberately blind version, because
/// losing `_b_state`/`_v_state` around `required` decouples the odd loop and NOTHING FAILS. Under
/// a shared actuator that trap has a twin with teeth: **a fuel leg whose `required` lost the
/// boundary would return `F_q = F_v = 0` and its row would look exactly like a MASKED one — this
/// rung would confirm its own headline through a bug.** So both legs are measured against both
/// blind versions.
///
/// # THE TWO BLIND CONTROLS IGNORE THEIR ARGUMENTS BY DESIGN
///
/// `blind_fuel` and `blind_gov` take `(qq, vv)` and read neither: that is the ENTIRE content of
/// the control, because what it demonstrates is that a law which does not set the boundary has
/// **identically zero** cross-gains. Rust warns about unused parameters, and wiring them in to
/// silence the warning would destroy the instrument — so they are named `_qq` / `_vv` and this
/// paragraph is the reason.
///
/// Both bars are Python's, literally: `== 0.0` on the dead controls and `abs(x) > 0.0` on the live
/// gains. Neither is a tolerance, and neither may become one — a zero cross-gain here is not a
/// weak coupling and not saturation (rung 67's gate), it is a LOST boundary.
pub fn assert_fuel_boundary(
    core: &ScheduledStatorCore, flight: &FlightCondition, p: &FuelPoint, tt4_max: f64,
    surge: Option<&Floor>, dq: f64, dv: f64,
) -> Result<BoundaryCheck, Abort> {
    let (a, h, mf_sched) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let (q, v) = match p.extra {
        PointExtra::Shared { b, v, .. } => (b, v),
        _ => panic!("rung-72's boundary check needs a six-state point"),
    };
    let laws = quad_laws(core, flight, a, h, mf_sched, None, surge, tt4_max);
    let ft = &core.fuel;

    let blind_fuel = |_qq: f64, _vv: f64| -> Result<f64, Abort> {
        let su = surge.expect("rung-72's blind fuel control needs the leg's own floor");
        Ok(0.0f64.max(mf_sched - ft.try_surge_fuel(flight, a, h, mf_sched, su)?))
    };
    let blind_gov = |_qq: f64, _vv: f64| -> Result<f64, Abort> {
        let i = ft.try_instant_fuel(flight, a, h, mf_sched)?;
        if i.base.tt4 <= tt4_max {
            return Ok(0.0);
        }
        Ok(0.0f64.max(mf_sched - ft.try_topping_fuel(flight, a, h, tt4_max, mf_sched)?))
    };

    let live_f_q = ((laws.f)(0.0, q + dq, v)?.0 - (laws.f)(0.0, q - dq, v)?.0) / (2.0 * dq);
    let live_f_v = ((laws.f)(0.0, q, v + dv)?.0 - (laws.f)(0.0, q, v - dv)?.0) / (2.0 * dv);
    let live_r_q = ((laws.r)(0.0, q + dq, v)?.0 - (laws.r)(0.0, q - dq, v)?.0) / (2.0 * dq);
    let live_r_v = ((laws.r)(0.0, q, v + dv)?.0 - (laws.r)(0.0, q, v - dv)?.0) / (2.0 * dv);
    let dead_f_q = (blind_fuel(q + dq, v)? - blind_fuel(q - dq, v)?) / (2.0 * dq);
    let dead_f_v = (blind_fuel(q, v + dv)? - blind_fuel(q, v - dv)?) / (2.0 * dv);
    let dead_r_q = (blind_gov(q + dq, v)? - blind_gov(q - dq, v)?) / (2.0 * dq);
    let dead_r_v = (blind_gov(q, v + dv)? - blind_gov(q, v - dv)?) / (2.0 * dv);

    assert!(dead_f_q == 0.0 && dead_f_v == 0.0 && dead_r_q == 0.0 && dead_r_v == 0.0,
            "rung-72: the BLIND controls are supposed to be identically zero; got \
             F_q={dead_f_q} F_v={dead_f_v} R_q={dead_r_q} R_v={dead_r_v}. If they are not, this \
             instrument is not measuring what it claims.");
    assert!(live_f_q.abs() > 0.0 && live_f_v.abs() > 0.0
            && live_r_q.abs() > 0.0 && live_r_v.abs() > 0.0,
            "rung-72: a fuel leg's cross-gains came back F_q={live_f_q} F_v={live_f_v} \
             R_q={live_r_q} R_v={live_r_v} at s = {}. A ZERO cross-gain here is not a weak \
             coupling and not saturation (rung 67's gate) -- it is a LOST `_b_state`/`_v_state` \
             boundary, and it would make a LIVE leg's row look exactly like a MASKED one. The \
             rung would then confirm its own headline.", p.s);
    Ok(BoundaryCheck { s: p.s, live_f_q, live_f_v, live_r_q, live_r_v,
                       dead_f_q, dead_f_v, dead_r_q, dead_r_v })
}

/// RUNG 72's `_shared_march` — **one rig, one march, under MIN-SELECT.**
///
/// The plant is ALWAYS marched under `max`, whatever law a reader then reads the Jacobian under.
/// Marching the SUM law would compare two laws on two different trajectories and confound the law
/// with the state (§ 3) — measured: the SUM law's own march starves the engine and stops at 84
/// points of 341.
#[allow(clippy::too_many_arguments)]
pub fn shared_march(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), r: f64, s_settle: f64, ds: f64, v_max: f64, inc: bool,
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
    let leg = StatorLeg { accel: None, surge, tt4_max: Some(tt4_max) };
    let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
    let traj = {
        let _sh = ShareScope::set(&m, "max");
        m.stator_march_scoped(
            flight, &ramp, None, &leg,
            &MarchScope { tau_gov: Some(tau_gov), lag, ..MarchScope::DEFAULT }).0
    };
    (m, surge, lag, traj)
}

// ---------------------------------------------------------------------------------------------
// § 1 — THE QUARTIC CHAIN: a 4×4 Jacobian, Faddeev–LeVerrier, and Durand–Kerner
// ---------------------------------------------------------------------------------------------

/// Python's `max(...)` over FIVE arguments — [`py_max4`]'s sibling, one wider.
///
/// It exists for the same reason as [`py_max3`](crate::lagged_bleed::py_max3): `f64::max` is not
/// Python's `max`, which returns the FIRST of equal arguments and propagates a NaN the moment one
/// appears rather than swallowing it. Its one call site is [`quartic_roots_c`]'s `scale`, whose
/// first argument is the literal `1.0` — the position where the two spellings part company.
fn py_max5(a: f64, b: f64, c: f64, d: f64, e: f64) -> f64 {
    let mut m = a;
    if b > m { m = b; }
    if c > m { m = c; }
    if d > m { m = d; }
    if e > m { m = e; }
    m
}

/// RUNG 72's `_charpoly4` — `[1, a3, a2, a1, a0]` of `det(lam I − A)` by Faddeev–LeVerrier.
///
/// Exact in exact arithmetic, no iteration, and it needs no pivoting to be stable at this size.
///
/// # THE ACCUMULATION ORDER IS COPIED, BECAUSE PYTHON's `sum()` HAS ONE
///
/// Both traces and the inner matrix product are Python `sum(...)` over a generator, which starts
/// at the **`int` 0** and adds left to right. `0 + x` is not an identity on a signed zero
/// (`0.0 + (-0.0)` is `+0.0`), and a right-to-left or pairwise fold is a different function in
/// floating point, so the accumulator starts at `0.0` and the loop runs in index order.
///
/// The division is `/ k` with an `int` `k`, which Python widens to a float — reproduced as
/// `/ (k as f64)` rather than by multiplying by a reciprocal.
pub fn charpoly4(a: &[[f64; 4]; 4]) -> [f64; 5] {
    let n = 4usize;
    let mut c = [1.0f64; 5];
    let mut m = [[0.0f64; 4]; 4];
    for k in 1..=n {
        if k == 1 {
            m = *a;
        } else {
            // `T = M + c[-1] I`, then `M = A T` — the recursion needs `M_{k-1}`, and the first
            // version of the Python had `A` here. `charpoly_selftest` is the instrument that
            // caught it; see its own doc comment.
            let mut t = [[0.0f64; 4]; 4];
            for (i, tr) in t.iter_mut().enumerate() {
                for (j, tv) in tr.iter_mut().enumerate() {
                    *tv = m[i][j] + if i == j { c[k - 1] } else { 0.0 };
                }
            }
            let mut next = [[0.0f64; 4]; 4];
            for (i, nr) in next.iter_mut().enumerate() {
                for (j, nv) in nr.iter_mut().enumerate() {
                    let mut s = 0.0f64;
                    for (tt, trow) in t.iter().enumerate() {
                        s += a[i][tt] * trow[j];
                    }
                    *nv = s;
                }
            }
            m = next;
        }
        let mut tr = 0.0f64;
        for (i, row) in m.iter().enumerate() {
            tr += row[i];
        }
        c[k] = -tr / (k as f64);
    }
    c
}

/// RUNG 72's `_quartic_roots_c` — the four roots of a monic quartic by **Durand–Kerner**, in
/// complex arithmetic.
///
/// Ferrari's closed form needs branch choices that go badly conditioned exactly where this rung
/// lives — near a DOUBLE root, which is what a block-triangular `M` with a repeated clock
/// produces. Durand–Kerner has no branches, and its residual is CHECKED by the caller rather than
/// trusted.
///
/// # THREE OF THE FIVE `scale` TERMS ARE DEAD ON EVERY SHIPPED INPUT — DISCLOSED, NEVER GATED
///
/// § 5.28 (iii) intercepted every call the whole rung-72 suite makes:
///
/// | | measured |
/// |---|---|
/// | calls | **1 068** |
/// | **distinct coefficient vectors** | **375** — the real size of the claim |
/// | which term wins `scale` | **`|a3|`, on 1 068 of 1 068** |
/// | iterations to converge | 9–29; the **500 cap is never hit** |
/// | the `den == 0` guard | **never fires** |
/// | min root separation `< 1e-6` | 167 of 1 068 |
///
/// So `|a2|**0.5`, `|a1|**(1/3.)` and `|a0|**0.25` — a cube root among them, the operation with
/// the least chance of agreeing across libms — are **UNREACHABLE**, and a port defect in any of
/// them would be invisible to every gate. They are ported faithfully with `powf`, spelled as
/// Python spells them, and this note is the instrument: a gate on an unreachable branch passes
/// forever and says nothing.
///
/// # THE ROOT COUNTS ARE IMPOSSIBLE FOR A REAL QUARTIC, AND THAT SETS THE BAR
///
/// Complex roots per call came back `{4: 717, 3: 237, 2: 92, 1: 22}`, and 3 and 1 are impossible
/// for a real quartic, whose complex roots come in conjugate pairs. They are Durand–Kerner leaving
/// an asymmetric last-bit imaginary residue — one member of a pair at exactly `0.0` and the other
/// not. **That makes bit-exactness the only achievable bar**: a port agreeing to 1e-14 would move
/// 259 of these counts.
///
/// It is also why the final sort must be **stable and use Python's `<`**. `sorted(z, key=lambda w:
/// (w.real, w.imag))` compares `-0.0` and `0.0` as EQUAL and falls through to the imaginary part,
/// preserving input order on a genuine tie; a total order over the bit patterns
/// ([`f64::total_cmp`]) puts `-0.0` strictly before `0.0` and would reorder exactly the pairs this
/// finding is about.
pub fn quartic_roots_c(coef: &[f64; 5]) -> [C64; 4] {
    let (a3, a2, a1, a0) = (coef[1], coef[2], coef[3], coef[4]);
    let poly = |z: C64| -> C64 {
        // `(((z + a3) * z + a2) * z + a1) * z + a0`, with every `+ float` a promoted complex add.
        let t = c_mul(c_add(z, c_real(a3)), z);
        let t = c_mul(c_add(t, c_real(a2)), z);
        c_add(c_mul(c_add(t, c_real(a1)), z), c_real(a0))
    };
    // a fixed, deterministic start — `Math.random` has no place in a reproducible gate
    let start = C64 { re: 0.4, im: 0.9 };
    let scale = py_max5(1.0, a3.abs(), a2.abs().powf(0.5), a1.abs().powf(1.0 / 3.0),
                        a0.abs().powf(0.25));
    let mut z = [C64 { re: 0.0, im: 0.0 }; 4];
    for (k, zk) in z.iter_mut().enumerate() {
        *zk = c_mul(c_powu(start, k as u32), c_real(scale));
    }
    for _ in 0..500 {
        let mut step = 0.0f64;
        for i in 0..4 {
            let mut den = C64 { re: 1.0, im: 0.0 };
            for j in 0..4 {
                if j != i {
                    den = c_mul(den, c_sub(z[i], z[j]));
                }
            }
            if c_is_zero(den) {
                den = C64 { re: 1e-30, im: 0.0 };
            }
            let d = c_div(poly(z[i]), den);
            z[i] = c_sub(z[i], d);
            let ad = d.abs();
            if ad > step {
                step = ad;
            }
        }
        if step <= 1e-14 * scale {
            break;
        }
    }
    // Python's `sorted(z, key=lambda w: (w.real, w.imag))` — TIMSORT, which is STABLE, comparing
    // the key tuples with `<`. `sort_by` is Rust's stable sort; `partial_cmp` on the tuple is
    // Python's comparison, INCLUDING `-0.0 == 0.0` falling through to the imaginary part.
    z.sort_by(|p, q| (p.re, p.im).partial_cmp(&(q.re, q.im)).expect(
        "rung-72: a NaN root reached the sort. Python's `sorted` would not raise here, but a NaN \
         root means Durand-Kerner diverged and every reader downstream is reading noise."));
    z
}

/// RUNG 72's `_parent_quartic` — § 1.2's claim AS A POLYNOMIAL IDENTITY.
///
/// `p4(lam) = (lam + 1/tau_m) * p3(lam)`, with `p3` the parent rung's own characteristic
/// polynomial built from the SHIPPED [`invariants`](crate::reference_split::invariants)
/// (`lam^3 − c2 lam^2 + c1 lam − c0`).
///
/// **COEFFICIENTS, NOT ROOTS, AND THAT IS THE POINT.** In the rank-ONE cell the parent has a
/// DOUBLE zero root and this rung a TRIPLE one, and a repeated root is resolved only to the square
/// root of the working precision — measured, the individual roots come back at 3e−07 while every
/// invariant sits at 1e−13. Matching roots there would report a 4.6e−07 "disagreement" that is the
/// root finder's resolution and not a difference between two plants.
///
/// **`tau_m = inf` IS A SHIPPED ARGUMENT, NOT A DEGENERATE ONE.** Rung 73 calls
/// `_parent_quartic(c3, float("inf"))` to state `a = 1/tau_m -> 0` exactly; `1.0 / f64::INFINITY`
/// is `+0.0` in IEEE and every product below then vanishes, which is the identity that rung wants.
/// No special case, and no guard that would break it.
pub fn parent_quartic(c3: (f64, f64, f64), tau_m: f64) -> [f64; 5] {
    let (c2, c1, c0) = c3;
    let p3 = (1.0, -c2, c1, -c0);
    let a = 1.0 / tau_m;
    [1.0, p3.1 + a, p3.2 + a * p3.1, p3.3 + a * p3.2, a * p3.3]
}

/// One arm of [`charpoly_selftest`] — Python's per-matrix dict, key for key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelftestArm {
    pub trace_err: f64,
    pub det_err: f64,
    pub det_vs_a0: f64,
    pub resid: f64,
    /// Only the triangular arm carries these two; Python adds them after the loop.
    pub diag_err: Option<f64>,
    pub max_imag: Option<f64>,
}

/// RUNG 72's `charpoly_selftest` — **THE INSTRUMENT, GATED AGAINST ITSELF, and this one is not
/// ceremony.**
///
/// The first version of [`charpoly4`] had `A` where the Faddeev–LeVerrier recursion needs
/// `M_{k−1}`, and it returned a WRONG polynomial whose roots were entirely plausible: a
/// stable-looking spectrum, a determinant of 5.9e+05, a residual `|p(root)|` of 1e−09 (the root
/// finder was faithfully solving the wrong polynomial), and a parent comparison that simply came
/// out large. **Nothing downstream could tell.** So the polynomial is checked against two things
/// it cannot be wrong about at the same time as being wrong:
///
/// * an INDEPENDENT trace and determinant — `sum(roots) = tr A` and `prod(roots) = det A` by
///   cofactor expansion, neither of which goes through the recursion;
/// * a TRIANGULAR matrix, whose spectrum is its own diagonal, with off-diagonal couplings large
///   enough that a broken recursion cannot coincidentally survive.
///
/// # IT IS THE SLICE's ONE FREE ORACLE TARGET, AND IT WAS PORTED FIRST FOR THAT REASON
///
/// A classmethod with no arguments, no march, no rig and no plant: it exercises [`charpoly4`],
/// [`quartic_roots_c`] and every complex operation this slice adds, on two fixed matrices. § 5.28
/// (ix)'s **P2** predicts its dict agrees with Python **bit-for-bit** on both, and if any key
/// disagrees, Durand–Kerner is the reason and § (iii)'s asymmetric-residue reading is the
/// diagnosis. Settling it before a single reader was written is what keeps a later disagreement
/// from being re-diffed through nine methods of plant.
pub fn charpoly_selftest() -> [(&'static str, SelftestArm); 2] {
    /// Laplace expansion — independent of the recursion under test.
    fn det4(a: &[[f64; 4]; 4]) -> f64 {
        let mut tot = 0.0f64;
        for j in 0..4 {
            // `[[A[i][k] for k in range(4) if k != j] for i in range(1, 4)]` — the minor with
            // row 0 and column `j` struck out, in Python's own index order.
            let mut sub = [[0.0f64; 3]; 3];
            for (i, row) in a.iter().enumerate().skip(1) {
                let mut kk = 0usize;
                for (k, x) in row.iter().enumerate() {
                    if k != j {
                        sub[i - 1][kk] = *x;
                        kk += 1;
                    }
                }
            }
            let d3 = sub[0][0] * (sub[1][1] * sub[2][2] - sub[1][2] * sub[2][1])
                - sub[0][1] * (sub[1][0] * sub[2][2] - sub[1][2] * sub[2][0])
                + sub[0][2] * (sub[1][0] * sub[2][1] - sub[1][1] * sub[2][0]);
            // Python's `(-1.0) ** j` — an integer exponent on a float, exact either way.
            tot += (-1.0f64).powi(j as i32) * a[0][j] * d3;
        }
        tot
    }

    let gen: [[f64; 4]; 4] = [[-20.0, 3.0, -1.5, 0.7], [2.0, -25.0, 4.0, -0.3],
                              [-1.0, 5.0, -30.0, 2.5], [0.5, -2.0, 6.0, -40.0]];
    let tri: [[f64; 4]; 4] = [[-20.0, 7.0, -3.0, 9.0], [0.0, -25.0, 4.0, -6.0],
                              [0.0, 0.0, -30.0, 8.0], [0.0, 0.0, 0.0, -50.0]];
    let mut out: [(&'static str, SelftestArm); 2] = [
        ("general", SelftestArm { trace_err: 0.0, det_err: 0.0, det_vs_a0: 0.0, resid: 0.0,
                                  diag_err: None, max_imag: None }),
        ("triangular", SelftestArm { trace_err: 0.0, det_err: 0.0, det_vs_a0: 0.0, resid: 0.0,
                                     diag_err: None, max_imag: None }),
    ];
    for (slot, a) in [(0usize, &gen), (1usize, &tri)] {
        let coef = charpoly4(a);
        let roots = quartic_roots_c(&coef);
        // `sum(A[i][i] for i in range(4))` and `sum(roots)` — both Python `sum`, both from 0.
        let mut tr = 0.0f64;
        for (i, row) in a.iter().enumerate() {
            tr += row[i];
        }
        let mut sr = C64 { re: 0.0, im: 0.0 };
        for z in roots.iter() {
            sr = c_add(sr, *z);
        }
        let mut pr = C64 { re: 1.0, im: 0.0 };
        for z in roots.iter() {
            pr = c_mul(pr, *z);
        }
        let dd = det4(a);
        let norm = 1.0f64.max(dd.abs());
        // Python's `max(gen)` takes the FIRST item as the running maximum and then compares with
        // `>`; seeding from a sentinel is a different function the moment a NaN appears.
        let mut resid = f64::NAN;
        let mut first = true;
        for z in roots.iter() {
            // `(((z + coef[1]) * z + coef[2]) * z + coef[3]) * z + coef[4]`
            let t = c_mul(c_add(*z, c_real(coef[1])), *z);
            let t = c_mul(c_add(t, c_real(coef[2])), *z);
            let v = c_add(c_mul(c_add(t, c_real(coef[3])), *z), c_real(coef[4])).abs();
            if first || v > resid {
                resid = v;
                first = false;
            }
        }
        out[slot].1 = SelftestArm {
            trace_err: (sr.re + coef[1]).abs() + (sr.re - tr).abs(),
            det_err: (pr.re - dd).abs() / norm,
            det_vs_a0: (coef[4] - dd).abs() / norm,
            resid: resid / norm,
            diag_err: None,
            max_imag: None,
        };
    }
    // the triangular arm: the spectrum IS the diagonal, matched element for element. Python calls
    // `_quartic_roots_c(_charpoly4(tri))` twice MORE here rather than reusing `roots`; the calls
    // are deterministic so the values agree, but the count is what § 5.28 (iii)'s 1 068 is made
    // of and the oracle's call census reads it.
    let mut diag: Vec<f64> = (0..4).map(|i| tri[i][i]).collect();
    diag.sort_by(|x, y| x.partial_cmp(y).expect("a finite diagonal"));
    let mut got: Vec<f64> = quartic_roots_c(&charpoly4(&tri)).iter().map(|z| z.re).collect();
    got.sort_by(|x, y| x.partial_cmp(y).expect("finite real parts"));
    let mut diag_err = f64::NAN;
    for (n, (x, y)) in diag.iter().zip(got.iter()).enumerate() {
        let v = (x - y).abs();
        if n == 0 || v > diag_err {
            diag_err = v;
        }
    }
    let mut max_imag = f64::NAN;
    for (n, z) in quartic_roots_c(&charpoly4(&tri)).iter().enumerate() {
        let v = z.im.abs();
        if n == 0 || v > max_imag {
            max_imag = v;
        }
    }
    out[1].1.diag_err = Some(diag_err);
    out[1].1.max_imag = Some(max_imag);
    out
}

// ---------------------------------------------------------------------------------------------
// THE FIVE READERS — § 0's windows, § 1's gains, § 2's cells, § 3's discriminator, § 4's bill
// ---------------------------------------------------------------------------------------------

/// A point's [`Authority`] label, or `None` where the trajectory carries none.
///
/// **A rung-72 reader can be handed a trajectory that is NOT rung 72's**, and [`shared_bill`] is
/// where: 8 of its 16 cells disarm one or both fuel-side legs, the march then leaves through an
/// inherited integrator, and the points come back as a five- or four-state variant with no
/// authority in them. Python's `p.get("authority")` is `None` there and every reader compares it
/// against a string, so `None` is the ANSWER and not a fallback — spelled once, here, rather than
/// as a `_ =>` arm at each of the sites that ask.
pub fn authority_of(p: &FuelPoint) -> Option<Authority> {
    match p.extra {
        PointExtra::Shared { authority, .. } => Some(authority),
        _ => None,
    }
}

/// The stator regime a point carries, `None` where it has no stator or no such key.
fn v_regime_of(p: &FuelPoint) -> Option<Regime> {
    match p.extra {
        PointExtra::Shared { v_regime, .. } => v_regime,
        PointExtra::Triple { v_regime, .. } => Some(v_regime),
        _ => None,
    }
}

/// The valve COMMAND a point carries, `None` where it has no valve.
fn b_cmd_of(p: &FuelPoint) -> Option<f64> {
    match p.extra {
        PointExtra::Shared { b_cmd, .. } => Some(b_cmd),
        PointExtra::Triple { b_cmd, .. } => Some(b_cmd),
        _ => None,
    }
}

/// Python's `span(sel)` — `(min s, max s, count)` over the points a predicate selects, and
/// `(None, None, 0)` on an empty window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub lo: Option<f64>,
    pub hi: Option<f64>,
    pub n: usize,
}

fn span(traj: &[FuelPoint], sel: impl Fn(&FuelPoint) -> bool) -> Span {
    let w: Vec<f64> = traj.iter().filter(|p| sel(p)).map(|p| p.s).collect();
    match (opt_fold(w.iter().copied(), f64::min), opt_fold(w.iter().copied(), f64::max)) {
        (Some(lo), Some(hi)) => Span { lo: Some(lo), hi: Some(hi), n: w.len() },
        _ => Span { lo: None, hi: None, n: 0 },
    }
}

/// The refusal [`authority_law`]'s census raises on a point that carries no authority label.
const RUNG72_CENSUS_MSG: &str =
    "rung-72 § 0 marches all four loops, so every point carries an authority label; a point \
     without one means the march left through an inherited integrator and this census is \
     counting a different plant.";

/// One `(inc, taus)` arm of [`authority_law`].
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorityArm {
    pub inc: bool,
    pub taus: (f64, f64, f64, f64),
    pub n: usize,
    /// Python's `census` dict, in [`Authority`]'s own order. A count of ZERO is a real reading and
    /// Python simply has no key for it, so all four pairs are emitted rather than the dict.
    pub census: [(Authority, usize); 4],
    /// Every `s` at which authority changed hands between two NON-dormant labels.
    pub handovers: Vec<f64>,
    pub fuel: Span,
    pub gov: Span,
    pub valve: Span,
    pub stator: Span,
    pub joint: Span,
    pub joint_fraction: f64,
    /// BOTH legs want a cut here: the masked one is RIDING and reaching nothing.
    pub both_want: usize,
    pub in_joint_fuel: usize,
    pub in_joint_gov: usize,
    pub handover_inside: bool,
    pub min_phi: f64,
    pub max_tt4: f64,
}

/// [`authority_law`]'s whole reading.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorityLaw {
    pub arms: Vec<AuthorityArm>,
    pub clocks: Vec<(f64, f64, f64, f64)>,
    pub ds: f64,
    pub both_cells_everywhere: bool,
    pub one_handover: bool,
}

/// RUNG 72's `authority_law` — **§ 0 MEASURED: the four windows, and the ONE hand-over inside
/// them.**
///
/// Under min-select exactly one fuel-side leg reaches the actuator at a time, so the trajectory
/// splits into an interval where rung 52's leg holds it and one where rung 47's governor does. THE
/// HAND-OVER SITS INSIDE THE JOINT WINDOW on both arms and at both clock settings, which is what
/// lets every claim in § 2 be measured on both sides of a rank change **on one trajectory**, with
/// no second plant.
///
/// AND THE TWO EVENTS THAT END THE FUEL LEG's AUTHORITY AND THE INCIDENCE STATOR's WINDOW ARE THE
/// SAME EVENT: `phi_lp` recovering through the floor shrinks rung 52's clip (so the governor
/// overtakes it) and simultaneously makes the stator dormant (rung 71 § 0.2). That is why the
/// incidence arm's governor-authority cell is nearly empty at matched clocks — 1 point of 35 — and
/// why the WIDE-CELL clock arm exists: a fast governor and a slow fuel leg hand over EARLY, and a
/// slow valve keeps the stator riding LATE. All four are swept march coordinates, disclosed as
/// such.
#[allow(clippy::too_many_arguments)]
pub fn authority_law(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, clocks: &[(f64, f64, f64, f64)], r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> AuthorityLaw {
    let mut out: Vec<AuthorityArm> = Vec::new();
    for inc in [false, true] {
        for taus in clocks.iter().copied() {
            let (m, _surge, _lag, traj) = shared_march(
                core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc);
            let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
            let joint = riding4(&traj, b_max);
            let mut census = [(Authority::Dormant, 0usize), (Authority::Tie, 0),
                              (Authority::Fuel, 0), (Authority::Gov, 0)];
            for p in traj.iter() {
                // Python indexes `census[p["authority"]]` with no `.get`, so a point without the
                // key raises. § 0 always marches the FULL rung-72 rig, so every point has one, and
                // the refusal STATES that invariant rather than papering over its failure with a
                // skip — which would report a smaller census and no error at all.
                let a = authority_of(p).expect(RUNG72_CENSUS_MSG);
                for slot in census.iter_mut() {
                    if slot.0 == a {
                        slot.1 += 1;
                    }
                }
            }
            let mut hand: Vec<f64> = Vec::new();
            for i in 1..traj.len() {
                let (a, b) = (authority_of(&traj[i]), authority_of(&traj[i - 1]));
                if a != b && a != Some(Authority::Dormant) && b != Some(Authority::Dormant) {
                    hand.push(traj[i].s);
                }
            }
            let in_joint_fuel = joint.iter()
                .filter(|p| authority_of(p) == Some(Authority::Fuel))
                .count();
            let in_joint_gov = joint.iter()
                .filter(|p| authority_of(p) == Some(Authority::Gov))
                .count();
            // Python takes `joint[0]["s"]` and `joint[-1]["s"]` — the FIRST and LAST of the
            // filtered list, NOT its min and max. The march is monotone in `s`, so the two agree
            // today; the spelling is the claim, and a non-monotone march would part them.
            let joint_span = if joint.is_empty() {
                Span { lo: None, hi: None, n: 0 }
            } else {
                Span { lo: Some(joint[0].s), hi: Some(joint[joint.len() - 1].s), n: joint.len() }
            };
            let handover_inside = !joint.is_empty()
                && !hand.is_empty()
                && hand.iter().any(|s| joint[0].s <= *s && *s <= joint[joint.len() - 1].s);
            let req_fuel = |p: &FuelPoint| match p.extra {
                PointExtra::Shared { required_fuel, .. } => required_fuel > 0.0,
                _ => false,
            };
            let req_gov = |p: &FuelPoint| match p.extra {
                PointExtra::Shared { required_gov, .. } => required_gov > 0.0,
                _ => false,
            };
            out.push(AuthorityArm {
                inc,
                taus,
                n: traj.len(),
                census,
                handovers: hand,
                fuel: span(&traj, req_fuel),
                gov: span(&traj, req_gov),
                valve: span(&traj, |p| matches!(b_cmd_of(p), Some(b) if 0.0 < b && b < b_max)),
                stator: span(&traj, |p| v_regime_of(p) == Some(Regime::Riding)),
                joint: joint_span,
                joint_fraction: if traj.is_empty() {
                    0.0
                } else {
                    joint.len() as f64 / traj.len() as f64
                },
                both_want: span(&traj, |p| req_fuel(p) && req_gov(p)).n,
                in_joint_fuel,
                in_joint_gov,
                handover_inside,
                min_phi: opt_fold(traj.iter().map(|p| p.phi_lp), f64::min)
                    .expect("rung-72's § 0 marches at least one point"),
                max_tt4: opt_fold(traj.iter().map(|p| p.tt4), f64::max)
                    .expect("rung-72's § 0 marches at least one point"),
            });
        }
    }
    let last = *clocks.last().expect("§ 0 sweeps at least one clock arm");
    AuthorityLaw {
        both_cells_everywhere: out.iter()
            .filter(|a| a.taus == last)
            .all(|a| a.in_joint_fuel > 0 && a.in_joint_gov > 0),
        one_handover: out.iter().all(|a| a.handovers.len() <= 1),
        arms: out,
        clocks: clocks.to_vec(),
        ds,
    }
}

/// One sampled point of [`shared_gains`].
#[derive(Clone, Debug, PartialEq)]
pub struct GainRow {
    pub s: f64,
    pub gains: QuadGains,
    pub authority: Option<Authority>,
    pub masked: Option<Authority>,
    pub mask_leak: Option<f64>,
    /// `coef[4]` — the quartic's constant term.
    pub det: f64,
    pub taus: (f64, f64, f64, f64),
}

/// [`shared_gains`]'s whole reading.
#[derive(Clone, Debug, PartialEq)]
pub struct SharedGains {
    pub inc: bool,
    pub taus: (f64, f64, f64, f64),
    pub ds: f64,
    pub rows: Vec<GainRow>,
    pub skipped_switch: usize,
    pub skipped_regime: usize,
    pub n_riding: usize,
    pub n_sampled: usize,
    pub boundary: Vec<BoundaryCheck>,
    pub s_window: Option<(f64, f64)>,
    pub by_authority_fuel: usize,
    pub by_authority_gov: usize,
    /// THE FOUR EXACT ZEROS — gated as `== 0.0`, never as `< tol`.
    pub worst_f_r: Option<f64>,
    pub worst_r_f: Option<f64>,
    pub worst_pair_fr: Option<f64>,
    pub worst_mask_leak: Option<f64>,
    /// and the gains that are NOT zero, so the plant is not trivially decoupled
    pub min_live_gain: Option<f64>,
    pub det_range: Option<(f64, f64)>,
}

/// RUNG 72's `shared_gains` — **§ 1 MEASURED: the twelve cross-gains, and the four that are
/// EXACTLY zero.**
///
/// ```text
/// F_r = R_f = 0     the two legs cannot see each other -- both solve from the
///                   SCHEDULED fuel (rungs 47/52's own discipline, inherited)
/// C_m = V_m = 0     the MASKED leg reaches the plant through nothing, because
///                   `max()` is FLAT in it
/// ```
///
/// so `pair_FR = 0` **exactly**. Rung 66's two loops on one VARIABLE gave a pair product of exactly
/// 1 — maximally redundant; two loops on one ACTUATOR give exactly 0 — maximally exclusive. Those
/// are the two corners of the same question and they are one rung apart in subject and six in
/// number.
///
/// **THE MASK LEAK IS THE GATED QUANTITY AND THE FREE POLE IS ITS CONSEQUENCE, NOT A SECOND
/// MEASUREMENT.** [`jac4`] puts `-1/tau_i` on the diagonal BY CONSTRUCTION, so once the masked
/// column's off-diagonal entries are measured zero, `A e_m = -(1/tau_m) e_m` is ALGEBRA and
/// reporting the eigenvalue as a separate confirmation would be the shipped instrument agreeing
/// with itself — rung 67 gate 9's retraction and rung 71 § 1.4's `c1`, in a third shape. **The pole
/// is reported; the leak is gated.**
#[allow(clippy::too_many_arguments)]
pub fn shared_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64,
    every: usize,
) -> Result<SharedGains, Abort> {
    let (m, surge, lag, traj) = shared_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let lag = lag.expect("§ 1's rig arms the fuel leg, so it carries the lag");
    let mut rows: Vec<GainRow> = Vec::new();
    let mut boundary: Vec<BoundaryCheck> = Vec::new();
    let (mut sk_switch, mut sk_regime) = (0usize, 0usize);
    let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
    for p in sampled.iter() {
        let gg = {
            let _sh = ShareScope::set(&m, "max");
            (m.triple_hooks().quad_gains_at)(&m, flight, p, None, surge.as_ref(), tt4_max,
                          1e-7, 1e-5, 1e-4, true, 4.0)?
        };
        if !gg.interior {
            if gg.near_switch {
                sk_switch += 1;
            } else {
                sk_regime += 1;
            }
            continue;
        }
        boundary.push(assert_fuel_boundary(&m, flight, p, tt4_max, surge.as_ref(), 1e-5, 1e-4)?);
        let (rf, gfv) = match p.extra {
            PointExtra::Shared { required_fuel, g_fuel, .. } => (required_fuel, g_fuel),
            _ => unreachable!("`riding4` admits only six-state points"),
        };
        let tt = (lag.tau(rf, gfv), taus.1, taus.2, taus.3);
        let coef = charpoly4(&jac4(&gg, tt));
        rows.push(GainRow {
            s: p.s,
            authority: gg.authority,
            masked: gg.masked,
            mask_leak: gg.mask_leak,
            det: coef[4],
            taus: tt,
            gains: gg,
        });
    }
    let leaks: Vec<f64> = rows.iter().filter_map(|x| x.mask_leak).collect();
    // Python's `min(abs(g[k]) for k in ("F_q", "F_v", "R_q", "R_v"))` — the four gains that must
    // NOT be zero, so "exactly zero everywhere" is not being bought with a decoupled instrument.
    let live_min = |x: &GainRow| -> f64 {
        let g = &x.gains;
        opt_fold([g.f_q.abs(), g.f_v.abs(), g.r_q.abs(), g.r_v.abs()].into_iter(), f64::min)
            .expect("four gains")
    };
    Ok(SharedGains {
        inc,
        taus,
        ds,
        skipped_switch: sk_switch,
        skipped_regime: sk_regime,
        n_riding: pts.len(),
        n_sampled: sampled.len(),
        s_window: if pts.is_empty() {
            None
        } else {
            Some((pts[0].s, pts[pts.len() - 1].s))
        },
        by_authority_fuel: rows.iter().filter(|x| x.authority == Some(Authority::Fuel)).count(),
        by_authority_gov: rows.iter().filter(|x| x.authority == Some(Authority::Gov)).count(),
        worst_f_r: opt_fold(rows.iter().map(|x| x.gains.f_r.abs()), f64::max),
        worst_r_f: opt_fold(rows.iter().map(|x| x.gains.r_f.abs()), f64::max),
        worst_pair_fr: opt_fold(rows.iter().map(|x| x.gains.pair_fr.abs()), f64::max),
        worst_mask_leak: opt_fold(leaks.iter().copied(), f64::max),
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

/// Python's `max` as a fold seed — the first item wins outright, then `>` decides. Seeded with
/// `f64::NAN`, which is what makes the first comparison a no-op rather than a bound.
pub(crate) fn py_running_max(acc: f64, x: f64) -> f64 {
    if acc.is_nan() || x > acc {
        x
    } else {
        acc
    }
}

/// One authority cell of one [`shared_cells`] arm.
#[derive(Clone, Debug, PartialEq)]
pub struct CellStat {
    pub n: usize,
    pub n_parent: usize,
    /// The DISTINCT zero counts seen in this cell — a `set`, sorted. § 2's law says it has exactly
    /// one member.
    pub zeros: Vec<usize>,
    pub gap: f64,
    pub vgap: f64,
    pub pole: f64,
    pub det: Option<(f64, f64)>,
    pub s: Option<(f64, f64)>,
    pub parent: &'static str,
}

/// One `(inc, taus)` arm of [`shared_cells`].
#[derive(Clone, Debug, PartialEq)]
pub struct CellsArm {
    pub inc: bool,
    pub taus: (f64, f64, f64, f64),
    pub cells: Vec<(Authority, CellStat)>,
    pub skipped_switch: usize,
    pub skipped_regime: usize,
    pub skipped_parent: usize,
    pub n_riding: usize,
    pub n_sampled: usize,
}

/// The union of one `(inc, authority)` key across every arm — Python's `seen`.
#[derive(Clone, Debug, PartialEq)]
pub struct SeenCell {
    pub parent: &'static str,
    pub zeros: Vec<usize>,
    pub gap: f64,
    pub vgap: f64,
    pub pole: f64,
    pub n: usize,
}

/// [`shared_cells`]'s whole reading.
#[derive(Clone, Debug, PartialEq)]
pub struct SharedCells {
    pub arms: Vec<CellsArm>,
    pub clocks: Vec<(f64, f64, f64, f64)>,
    pub ds: f64,
    pub cells: Vec<((bool, Authority), SeenCell)>,
    /// THE LAW: `zeros = n_live − m_live`, one value per cell, four cells.
    pub law_holds: bool,
    pub predicted: [((bool, Authority), usize); 4],
    pub all_four_cells: bool,
    pub worst_parent_gap: f64,
    pub worst_v_gap: f64,
    pub worst_pole: f64,
}

/// Which parent rung a cell IS — § 2's table, and the rung.
fn parent_of(inc: bool, auth: Authority) -> &'static str {
    match (inc, auth) {
        (false, Authority::Fuel) => "rung 68",
        (false, Authority::Gov) => "rung 70",
        (true, Authority::Fuel) => "rung 69",
        (true, Authority::Gov) => "rung 71",
        _ => panic!("rung-72's § 2 has four cells, indexed by a LIVE authority; \
                     `Dormant`/`Tie` name no parent because no leg holds the actuator there."),
    }
}

/// RUNG 72's `shared_cells` — **§ 2 MEASURED, AND IT IS THE RUNG: this ONE six-state plant IS rung
/// 68, 69, 70 or 71 at every instant — polynomial for polynomial — plus a free pole at the masked
/// leg's own clock.**
///
/// Which one is selected by AUTHORITY and by the stator's coordinate:
///
/// | stator watches | fuel leg holds | governor holds |
/// |---|---|---|
/// | `phi` | RUNG 68 (m_live 1, zeros 2) | RUNG 70 (m_live 2, zeros 1) |
/// | `M_i` | RUNG 69 (m_live 2, zeros 1) | RUNG 71 (m_live 3, zeros 0) |
///
/// The whole `(3, m)` table rungs 68–71 spent four rungs filling is a property of ONE plant,
/// indexed by which leg holds the actuator. **The rank CHANGES at the hand-over with no state, no
/// gain and no clock moving** — 2 to 1 on the `phi` arm and 1 to 0 on the incidence one — which is
/// a discontinuity no previous rung in this family could exhibit, because none had a quantity that
/// could change without something moving.
///
/// **THE TEST IS A POLYNOMIAL IDENTITY, NOT A ROOT MATCH** ([`parent_quartic`]): the parent's
/// characteristic polynomial is rebuilt from the SHIPPED rung-68/69/70/71 readers
/// ([`triple_gains_at`](crate::three_loop::triple_gains_at) →
/// [`invariants`](crate::reference_split::invariants)), multiplied by `(lam + 1/tau_masked)`, and
/// compared to this rung's own quartic coefficient by coefficient. Two independent instruments
/// reaching the same polynomial is the measurement; the free pole is a consequence of § 1's
/// measured zeros and is reported, never gated ([`shared_gains`]).
#[allow(clippy::too_many_arguments)]
pub fn shared_cells(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, clocks: &[(f64, f64, f64, f64)], r: f64, s_settle: f64, ds: f64, v_max: f64,
    every: usize,
) -> Result<SharedCells, Abort> {
    let mut arms: Vec<CellsArm> = Vec::new();
    for inc in [false, true] {
        for taus in clocks.iter().copied() {
            let (m, surge, lag, traj) = shared_march(
                core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc);
            let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
            let pts = riding4(&traj, b_max);
            let lag = lag.expect("§ 2's rig arms the fuel leg");
            let mut cells: Vec<(Authority, CellStat)> = Vec::new();
            let (mut sk_switch, mut sk_regime, mut sk_parent) = (0usize, 0usize, 0usize);
            let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
            for p in sampled.iter() {
                let gg = {
                    let _sh = ShareScope::set(&m, "max");
                    (m.triple_hooks().quad_gains_at)(&m, flight, p, None, surge.as_ref(), tt4_max,
                                  1e-7, 1e-5, 1e-4, true, 4.0)?
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
                let (rf, gfv) = match p.extra {
                    PointExtra::Shared { required_fuel, g_fuel, .. } => (required_fuel, g_fuel),
                    _ => unreachable!("`riding4` admits only six-state points"),
                };
                let tau_f = lag.tau(rf, gfv);
                let tt = (tau_f, taus.1, taus.2, taus.3);
                let coef = charpoly4(&jac4(&gg, tt));
                let roots = quartic_roots_c(&coef);
                let rate = 1.0 / tt.0 + 1.0 / tt.1 + 1.0 / tt.2 + 1.0 / tt.3;
                let nz = roots.iter().filter(|z| z.abs() < 1e-4 * rate).count();
                let tau_m = if auth == Authority::Gov { tau_f } else { taus.1 };
                // THE PARENT, chosen by AUTHORITY, through the SHIPPED three-loop readers:
                //   the governor holding  => {gov, valve, stator} = rung 70/71
                //   the fuel leg holding  => {rung 52's leg, valve, stator} = rung 68/69
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
                let (mut gap, mut vgap) = (None, None);
                if g3.interior {
                    let pred = parent_quartic(crate::reference_split::invariants(&g3, t3), tau_m);
                    // `max(abs(coef[j] - pred[j]) / rate**j for j in range(1, 5))` — each
                    // coefficient normalised by its OWN dimension, so a `j`-th coefficient is
                    // compared against a `j`-th power of the natural rate and the four terms are
                    // commensurable.
                    let g = opt_fold(
                        (1..5usize).map(|j| (coef[j] - pred[j]).abs() / rate.powi(j as i32)),
                        f64::max).expect("four coefficients");
                    gap = Some(g);
                    vgap = Some((g3.v_base - gg.v_base).abs());
                } else {
                    sk_parent += 1;
                }
                // `min(abs(z + 1.0/tau_m) for z in roots) * tau_m` — `z + float` is a PROMOTED
                // complex add, so the imaginary part is `z.im + 0.0` and not `z.im`. The two
                // differ on a signed zero, which § 5.28 (iii) measured Durand-Kerner producing on
                // one member of a conjugate pair; `hypot` happens to absorb it here, and the
                // spelling is kept anyway so the next reader need not redo that argument.
                let pole = opt_fold(
                    roots.iter().map(|z| c_add(*z, c_real(1.0 / tau_m)).abs()), f64::min)
                    .expect("a quartic has four roots") * tau_m;
                let slot = match cells.iter().position(|(k, _)| *k == auth) {
                    Some(i) => i,
                    None => {
                        cells.push((auth, CellStat {
                            n: 0,
                            n_parent: 0,
                            zeros: Vec::new(),
                            gap: 0.0,
                            vgap: 0.0,
                            pole: 0.0,
                            det: None,
                            s: None,
                            parent: parent_of(inc, auth),
                        }));
                        cells.len() - 1
                    }
                };
                let c = &mut cells[slot].1;
                c.n += 1;
                if !c.zeros.contains(&nz) {
                    c.zeros.push(nz);
                }
                c.det = Some(match c.det {
                    None => (coef[4], coef[4]),
                    Some((lo, hi)) => (lo.min(coef[4]), hi.max(coef[4])),
                });
                c.s = Some(match c.s {
                    None => (p.s, p.s),
                    Some((lo, hi)) => (lo.min(p.s), hi.max(p.s)),
                });
                if pole > c.pole {
                    c.pole = pole;
                }
                if let (Some(g), Some(vg)) = (gap, vgap) {
                    c.n_parent += 1;
                    if g > c.gap {
                        c.gap = g;
                    }
                    if vg > c.vgap {
                        c.vgap = vg;
                    }
                }
            }
            for (_, c) in cells.iter_mut() {
                c.zeros.sort_unstable();
            }
            arms.push(CellsArm {
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
    let mut seen: Vec<((bool, Authority), SeenCell)> = Vec::new();
    for a in arms.iter() {
        for (auth, c) in a.cells.iter() {
            let k = (a.inc, *auth);
            let slot = match seen.iter().position(|(kk, _)| *kk == k) {
                Some(i) => i,
                None => {
                    seen.push((k, SeenCell {
                        parent: c.parent,
                        zeros: Vec::new(),
                        gap: 0.0,
                        pole: 0.0,
                        n: 0,
                        vgap: 0.0,
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
            if c.gap > d.gap {
                d.gap = c.gap;
            }
            if c.vgap > d.vgap {
                d.vgap = c.vgap;
            }
            if c.pole > d.pole {
                d.pole = c.pole;
            }
            d.n += c.n;
        }
    }
    for (_, d) in seen.iter_mut() {
        d.zeros.sort_unstable();
    }
    Ok(SharedCells {
        law_holds: seen.iter().all(|(_, d)| d.zeros.len() == 1),
        predicted: [((false, Authority::Fuel), 2), ((false, Authority::Gov), 1),
                    ((true, Authority::Fuel), 1), ((true, Authority::Gov), 0)],
        all_four_cells: seen.len() == 4,
        worst_parent_gap: seen.iter().map(|(_, d)| d.gap).fold(f64::NAN, py_running_max),
        worst_v_gap: seen.iter().map(|(_, d)| d.vgap).fold(f64::NAN, py_running_max),
        worst_pole: seen.iter().map(|(_, d)| d.pole).fold(f64::NAN, py_running_max),
        cells: seen,
        arms,
        clocks: clocks.to_vec(),
        ds,
    })
}

/// One composition law's reading at one clock arm of [`mask_discriminator`].
#[derive(Clone, Debug, PartialEq)]
pub struct LawRead {
    pub zeros: Vec<(Authority, Vec<usize>)>,
    pub worst_pole: Option<f64>,
    pub worst_re: f64,
    pub authority: Vec<Authority>,
}

/// One clock arm of [`mask_discriminator`].
#[derive(Clone, Debug, PartialEq)]
pub struct MaskArm {
    pub taus: (f64, f64, f64, f64),
    /// `tau_f == tau_g` — **THE CONFOUND**, carried in the table as the confound it is.
    pub matched: bool,
    pub law_max: LawRead,
    pub law_sum: LawRead,
    pub n: usize,
}

/// [`mask_discriminator`]'s whole reading.
#[derive(Clone, Debug, PartialEq)]
pub struct MaskDiscriminator {
    pub inc: bool,
    pub arms: Vec<MaskArm>,
    pub ds: f64,
    /// THE DISCRIMINATOR, unmatched clocks only.
    pub max_pole_unmatched: Option<f64>,
    pub sum_pole_unmatched: Option<f64>,
    /// THE CONFOUND, quoted so it cannot be quoted as a result.
    pub sum_pole_matched: Option<f64>,
    /// Reported as TWO FACTS, never as a stability theorem (a frozen Jacobian on a trajectory the
    /// SUM law never marched).
    pub sum_worst_re: f64,
    pub max_worst_re: f64,
}

/// RUNG 72's `mask_discriminator` — **§ 3: the SUM law read at the MIN-SELECT trajectory's own base
/// points.**
///
/// One law swapped, nothing else, which is rung 71's `m70`-at-identical-points device applied to a
/// composition law instead of a rung.
///
/// **IT IS NOT MARCHED.** The SUM law double-clips: its own march starves the engine and stops at
/// 84 points of 341 with `Tt4` never reaching the redline. Comparing two laws on two different
/// trajectories would confound the law with the state.
///
/// # AND THE FIRST VERSION OF THIS READER AGREED WITH ITSELF
///
/// At `tau_f = tau_g` the SUM law has `(1, −1, 0, 0)` as an exact eigenvector with eigenvalue
/// `-1/tau` — the two fuel rows are `(−1, 0, .)` and `(0, −1, .)` and the shared columns cancel in
/// the difference direction — so the free-pole test passes under BOTH laws at matched clocks
/// (residual 3.6e−16) and separates them only when the two fuel clocks DIFFER (measured 1.4e−02 to
/// 3.6e−01). The matched arm is carried in the table as the confound it is, because a
/// discriminator that would have been quoted from the matched arm alone is a discriminator that
/// never tested anything (rung 66's own lesson, and rung 70's).
#[allow(clippy::too_many_arguments)]
pub fn mask_discriminator(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, clocks: &[(f64, f64, f64, f64)], inc: bool, r: f64, s_settle: f64, ds: f64,
    v_max: f64, every: usize,
) -> Result<MaskDiscriminator, Abort> {
    // Python marches `clocks[0]` into `m0, surge0, lag0, _` and then never reads any of them. The
    // call is kept because it is a march the plant sees — the counters it bumps are part of what a
    // bit-exact port owes — and the bindings are discarded here exactly as Python discards `_`.
    let _first = shared_march(core, flight, tt4_lo, tt4_hi, tt4_max, sm, clocks[0], r, s_settle,
                              ds, v_max, inc);
    let mut out: Vec<MaskArm> = Vec::new();
    for taus in clocks.iter().copied() {
        let (m, surge, lag, traj) = shared_march(
            core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc);
        let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
        let pts = riding4(&traj, b_max);
        let lag = lag.expect("§ 3's rig arms the fuel leg");
        let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
        let mut reads: Vec<LawRead> = Vec::new();
        for law in ["max", "sum"] {
            let mut zc: Vec<(Authority, Vec<usize>)> = Vec::new();
            let mut poles: Vec<f64> = Vec::new();
            let mut worst_re = -1e30f64;
            let mut auth: Vec<Authority> = Vec::new();
            for p in sampled.iter() {
                let gg = {
                    let _sh = ShareScope::set(&m, law);
                    (m.triple_hooks().quad_gains_at)(&m, flight, p, None, surge.as_ref(), tt4_max,
                                  1e-7, 1e-5, 1e-4, true, 4.0)?
                };
                if !gg.interior {
                    continue;
                }
                let (rf, gfv) = match p.extra {
                    PointExtra::Shared { required_fuel, g_fuel, .. } => (required_fuel, g_fuel),
                    _ => unreachable!("`riding4` admits only six-state points"),
                };
                let tau_f = lag.tau(rf, gfv);
                let tt = (tau_f, taus.1, taus.2, taus.3);
                let roots = quartic_roots_c(&charpoly4(&jac4(&gg, tt)));
                let rate = 1.0 / tt.0 + 1.0 / tt.1 + 1.0 / tt.2 + 1.0 / tt.3;
                let a = gg.authority.expect("an interior point carries a label");
                if !auth.contains(&a) {
                    auth.push(a);
                }
                let nz = roots.iter().filter(|z| z.abs() < 1e-4 * rate).count();
                match zc.iter_mut().find(|(k, _)| *k == a) {
                    Some((_, v)) => {
                        if !v.contains(&nz) {
                            v.push(nz);
                        }
                    }
                    None => zc.push((a, vec![nz])),
                }
                let tau_m = if a == Authority::Gov { tau_f } else { taus.1 };
                poles.push(opt_fold(
                    roots.iter().map(|z| c_add(*z, c_real(1.0 / tau_m)).abs()), f64::min)
                    .expect("a quartic has four roots") * tau_m);
                let mt = opt_fold([tt.0, tt.1, tt.2, tt.3].into_iter(), f64::min)
                    .expect("four clocks");
                let hi = opt_fold(roots.iter().map(|z| z.re), f64::max)
                    .expect("a quartic has four roots") * mt;
                if hi > worst_re {
                    worst_re = hi;
                }
            }
            for (_, v) in zc.iter_mut() {
                v.sort_unstable();
            }
            zc.sort_by_key(|(k, _)| *k as u8);
            auth.sort_by_key(|k| *k as u8);
            reads.push(LawRead {
                zeros: zc,
                worst_pole: opt_fold(poles.iter().copied(), f64::max),
                worst_re,
                authority: auth,
            });
        }
        let law_sum = reads.pop().expect("two laws");
        let law_max = reads.pop().expect("two laws");
        out.push(MaskArm { taus, matched: taus.0 == taus.1, law_max, law_sum, n: sampled.len() });
    }
    // Python's `max((… for a in un), default=None)` over a generator that can yield `None`: with a
    // single arm it returns that `None` unexamined, and with two it would RAISE on the comparison.
    // No shipped grid produces one, and the `filter_map` here is the same answer on every input
    // that does not raise.
    let un: Vec<&MaskArm> = out.iter().filter(|a| !a.matched).collect();
    let mt: Vec<&MaskArm> = out.iter().filter(|a| a.matched).collect();
    Ok(MaskDiscriminator {
        inc,
        ds,
        max_pole_unmatched: opt_fold(un.iter().filter_map(|a| a.law_max.worst_pole), f64::max),
        sum_pole_unmatched: opt_fold(un.iter().filter_map(|a| a.law_sum.worst_pole), f64::min),
        sum_pole_matched: opt_fold(mt.iter().filter_map(|a| a.law_sum.worst_pole), f64::max),
        sum_worst_re: out.iter().map(|a| a.law_sum.worst_re).fold(f64::NAN, py_running_max),
        max_worst_re: out.iter().map(|a| a.law_max.worst_re).fold(f64::NAN, py_running_max),
        arms: out,
    })
}

/// One cell of [`shared_bill`]'s sixteen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BillCell {
    /// `(F, G, V, S)` — which of the four loops this cell arms.
    pub on: (bool, bool, bool, bool),
    /// The `phi` violation integral.
    pub i: f64,
    /// The `Tt4` exceedance integral.
    pub e: f64,
    /// The INCIDENCE violation integral.
    pub m: f64,
    pub min_phi: f64,
    pub max_tt4: f64,
    pub n: usize,
    /// WHERE the fuel leg held the actuator, in this cell.
    pub auth_fuel: usize,
    /// The first `s` at which the label goes `fuel -> gov`, or `None` if it never does.
    pub handover: Option<f64>,
    pub credit_phi: Option<f64>,
    pub credit_tt4: Option<f64>,
    pub credit_inc: Option<f64>,
}

/// [`shared_bill`]'s whole reading — the sixteen cells and the four marginals.
#[derive(Clone, Debug, PartialEq)]
pub struct SharedBill {
    pub cells: Vec<(&'static str, BillCell)>,
    pub inc: bool,
    pub taus: (f64, f64, f64, f64),
    /// Each loop's OWN currency and the cell that omits it — `(leg, currency, without)`.
    pub own_currency: [(&'static str, char, &'static str); 4],
    /// Per leg, in Python's `F, G, V, S` order.
    pub marginal: [(&'static str, f64); 4],
    pub alone: [(&'static str, f64); 4],
    pub kept: [(&'static str, Option<f64>); 4],
    /// P6: the fuel leg buys `phi` and does NOT spend the governor's currency.
    pub fuel_marginal_phi: f64,
    pub fuel_marginal_tt4: f64,
    pub tt4_full: f64,
    pub tt4_no_fuel: f64,
    pub phi_full: f64,
    pub phi_no_fuel: f64,
    pub handover: Option<f64>,
    pub delivered_phi: Option<f64>,
    pub delivered_tt4: Option<f64>,
    pub delivered_inc: Option<f64>,
}

/// The sixteen cell names, indexed by Python's own bit pattern.
///
/// `names = ("F", "G", "V", "S")` and `on[i] = bits & (1 << i)`, so bit 0 is the FUEL leg and the
/// key is the set names in that order — `"bare"` for the empty set. Tabulated rather than built
/// with a `String`, so every key is a `&'static str` and the lookups below compare literals,
/// exactly as Python's dict keys do.
const BILL_KEYS: [&str; 16] = ["bare", "F", "G", "FG", "V", "FV", "GV", "FGV",
                               "S", "FS", "GS", "FGS", "VS", "FVS", "GVS", "FGVS"];

/// RUNG 72's `shared_bill` — **THE 16-CELL LEDGER: every subset of the FOUR loops, in THREE
/// currencies.**
///
/// Rungs 66/68 had 8 cells in one currency, rungs 70/71 8 in two and three. Here the subsets
/// double because the fourth loop is real even where its authority is not, and the question the
/// ledger answers is the one a rank argument cannot: **what does a MASKED leg buy?** It is coupled
/// to nothing while masked, so a spectral reading says "nothing"; the ledger says otherwise,
/// because authority is a function of `s` and a leg that is masked late held the actuator early.
///
/// THE PREDICTION UNDER TEST (anchor P6): the fuel leg's marginal `phi` credit is POSITIVE but
/// delivered ENTIRELY inside its own authority window, and it leaves `max Tt4` unmoved — a leg
/// that is masked wherever the governor binds cannot spend the governor's currency. **The shipped
/// suite refutes the second half in BOTH directions at once**: the exceedance INTEGRAL improves
/// while the PEAK gets worse, so the sign is the claim and the magnitudes are disclaimed.
///
/// Reported with the ABSOLUTE integrals beside the ratios, because rung 71 § 4's own lesson is
/// that a loop can keep 100 % of a credit that is small — and here the `F` solo cell is degenerate
/// (rung 52's leg ALONE starves the accel outright, so `E = 0` and `max Tt4` stays at the initial
/// value), which puts the `kept` denominator on a trajectory no other cell shares.
///
/// # EIGHT OF THE SIXTEEN CELLS CARRY NO STATOR, AND THAT IS WHY `v_regime` IS AN `Option`
///
/// ONE constructor builds every cell ([`shared_rig`](crate::three_loop::TripleHooks::shared_rig)),
/// so a cell differs from another only by which loops are armed — rung 63's lesson, and the reason
/// the credits are differenceable at all.
#[allow(clippy::too_many_arguments)]
pub fn shared_bill(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> SharedBill {
    let mut cells: Vec<(&'static str, BillCell)> = Vec::new();
    for bits in 0u32..16 {
        let on = (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let (m2, surge2, lag2) = (core.triple_hooks().shared_rig)(core, &SharedRigArm {
            sm,
            tau: taus.2,
            tau_s: taus.3,
            v_max,
            tt4_max,
            tau_att: taus.0,
            tau_rel: 3.0 * taus.0,
            inc,
            fuel: on.0,
            gov: on.1,
            valve: on.2,
            stator: on.3,
        });
        let leg = StatorLeg {
            accel: None,
            surge: surge2,
            tt4_max: if on.1 { Some(tt4_max) } else { None },
        };
        let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
        let traj = {
            let _sh = ShareScope::set(&m2, "max");
            m2.stator_march_scoped(
                flight, &ramp, None, &leg,
                &MarchScope {
                    tau_gov: if on.1 { Some(taus.1) } else { None },
                    lag: lag2,
                    ..MarchScope::DEFAULT
                }).0
        };
        // Built off the RECEIVER's design map exactly as Python does (`self.map_lp_design`, not
        // `m2`'s) — they are equal, and the spelling is the claim.
        let phi_lim = (1.0 + sm) * core.arming().map_lp_design.phi_surge;
        let t_c = core.arming().map_lp_design.tan_beta1_crit();
        // Python's `next((p["s"] for i, p in enumerate(traj[1:], 1) if …), None)` — the FIRST
        // `fuel -> gov` step, and `None` where the label never makes that transition, which is
        // every cell whose march left through an inherited integrator.
        let mut handover = None;
        for i in 1..traj.len() {
            if authority_of(&traj[i]) == Some(Authority::Gov)
                && authority_of(&traj[i - 1]) == Some(Authority::Fuel) {
                handover = Some(traj[i].s);
                break;
            }
        }
        cells.push((BILL_KEYS[bits as usize], BillCell {
            on,
            i: crate::two_lag::violation(&traj, phi_lim, r),
            e: crate::cross_loop::exceed(&traj, tt4_max, r),
            m: crate::three_loop::violation_inc(&traj, t_c - 1.0 / phi_lim, t_c, r),
            min_phi: opt_fold(traj.iter().map(|p| p.phi_lp), f64::min)
                .expect("rung-72's ledger marches at least one point"),
            max_tt4: opt_fold(traj.iter().map(|p| p.tt4), f64::max)
                .expect("rung-72's ledger marches at least one point"),
            n: traj.len(),
            auth_fuel: traj.iter().filter(|p| authority_of(p) == Some(Authority::Fuel)).count(),
            handover,
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
    let at = |name: &str| -> BillCell {
        cells.iter()
            .find(|(k, _)| *k == name)
            .unwrap_or_else(|| panic!("rung-72's ledger has no cell {name:?}"))
            .1
    };
    let cur = |c: &BillCell, ch: char| match ch {
        'I' => c.i,
        'E' => c.e,
        'M' => c.m,
        _ => unreachable!("three currencies"),
    };
    // `own = dict(F=("I","GVS"), G=("E","FVS"), V=("I","FGS"), S=("M" if inc else "I","FGV"))`
    // — and the STATOR's currency is the ONLY one that depends on the arm, because a `phi` stator
    // and an incidence stator do not defend the same wall.
    let own: [(&'static str, char, &'static str); 4] = [
        ("F", 'I', "GVS"),
        ("G", 'E', "FVS"),
        ("V", 'I', "FGS"),
        ("S", if inc { 'M' } else { 'I' }, "FGV"),
    ];
    let full = at("FGVS");
    let mut marginal = [("F", 0.0f64); 4];
    let mut alone = [("F", 0.0f64); 4];
    let mut kept: [(&'static str, Option<f64>); 4] = [("F", None); 4];
    for (n, (leg, ch, without)) in own.iter().enumerate() {
        let marg = cur(&at(without), *ch) - cur(&full, *ch);
        let solo = cur(&base, *ch) - cur(&at(leg), *ch);
        marginal[n] = (leg, marg);
        alone[n] = (leg, solo);
        kept[n] = (leg, if solo != 0.0 { Some(marg / solo) } else { None });
    }
    let gvs = at("GVS");
    SharedBill {
        inc,
        taus,
        own_currency: own,
        marginal,
        alone,
        kept,
        fuel_marginal_phi: marginal[0].1,
        fuel_marginal_tt4: gvs.e - full.e,
        tt4_full: full.max_tt4,
        tt4_no_fuel: gvs.max_tt4,
        phi_full: full.min_phi,
        phi_no_fuel: gvs.min_phi,
        handover: full.handover,
        delivered_phi: full.credit_phi,
        delivered_tt4: full.credit_tt4,
        delivered_inc: full.credit_inc,
        cells,
    }
}
