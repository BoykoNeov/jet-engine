//! RUNG 70 — **THE GENERIC SPLIT**: rung 47's `Tt4` topping GOVERNOR as the odd loop, beside rung
//! 65's `phi` valve and rung 68's `phi` stator.
//!
//! Rung 67's substitution applied to rung 68's triple. Five states, three clocks, one actuator per
//! loop — rung 68's shapes exactly, and only the ODD loop's COORDINATE differs. `n = 3`, `m = 2`,
//! **the same cell as rung 69 reached by a different route**, so this is a controlled comparison
//! at equal counts.
//!
//! Headline: *the split buys the RANK, but the RING needs the odd constraint to be a SECOND WALL
//! ON THE SAME LEVER* — the floor is rung 67's `zeta`. See `docs/rung70-spec.md`.
//!
//! # What this module is — and what STEP 1 ships
//!
//! **THERE IS NO `CrossSplitCore`, AND A READER SHOULD NOT GO LOOKING FOR ONE.** One type carries
//! rungs 57–84 ([`ScheduledStatorCore`], over [`TwoSpoolTransientCore`]) —
//! `crate::bleed_transient`'s own note — and **neither rung 70 nor rung 71 defines `__init__` in
//! Python at all**, measured over both class bodies rather than assumed. So the "two cores" this
//! slice opens are the two MODULES: this one and [`crate::full_split`], each holding its rung's
//! five tables, and the carrier they share.
//!
//! Step 1 shipped: `_gov_max`'s carrier ([`TwoSpoolTransientCore::gov_max`]) and its guard
//! [`GovScope`]; [`build_cross_split_cascade`]; the five `R70*` tables; and rung 70's **three
//! swapped cells opened as NAMED PANICS**.
//!
//! **STEP 2 FILLED THEM, and added the nine remaining methods of the Python class**: the march
//! ([`r70_integrate_fuel_cross_triple`]) with its floor [`r70_rk4_floor_split`], the rig
//! [`split_rig`], the boundary instrument [`assert_state_boundary`], the damping reader
//! [`zeta_pair`], and the **seven** readers [`split_gains`], [`rung67_control`], [`split_modes`],
//! [`c1_clock_swap`], [`split_floor`], [`window_overlap`], [`split_bill`]. That is **16 methods**,
//! and the count is taken off the class body rather than off § 5.27 (iii)'s prose: 9 non-readers
//! + 7 readers = 16, which is what the emitted census says. Steps 4–5 are the gates.
//!
//! # THE TABLE ARITHMETIC — **TWO COUNTS, AND THEY ARE NOT THE SAME COUNT**
//!
//! Written out here rather than left in two files to be reconciled, which is this phase's
//! most-repeated defect. `tests/slice_ac_cells.rs` holds the assertions.
//!
//! * **ZERO CELLS ADDED**, at either rung. § 5.27 (i)'s finding: the phase's cell predicate is
//!   *new here AND overridden above*, purely by NAME, and it has never asked whether the two
//!   bodies are INTERCHANGEABLE — which is the entire requirement for a `fn` pointer in a `const`
//!   table. The one cell the column claimed for this slice, `split_gains`, is a name REUSED:
//!   rung 80's same-named reader drops `sm, tau, tau_gov, tau_s` and adds `phi_lim, phi_airs,
//!   coord, taus, inc`, and rung 70's own inherited caller `rung67_control` raises `TypeError` on
//!   a rung-80 machine. So `TripleHooks` stays **ten** fields wide and `split_gains` ports as an
//!   ordinary method on the rung-70 type.
//! * **FIVE SWAPS over the two rungs ⇒ FIVE distinct function pointers**: `at_lever` and
//!   `integrate_fuel` at BOTH rungs, `triple_laws` at rung 70 only. Three of the five are here.
//! * **TEN TABLE CONSTS, not nine** — five per rung, and the pre-flight's step-1 line said
//!   "nine". Counted rather than inherited: `R70`, `R70_TWO`, `R70_FUEL`, `R70_STATOR`,
//!   `R70_TRIPLE` here and the same five at rung 71. The gate counts the `pub const R7`
//!   declarations in the two files, so the number cannot drift back into prose.
//!
//! # WHY THE FILE IS SHAPED THE WAY IT IS — the swaps, again
//!
//! Phase 7's rule is *step 1 of every slice is the cell addition*, so a slice that forgets a cell
//! fails at its own first gate. **This slice adds no cell at all, so that rule buys nothing** —
//! and the risk it was protecting against is entirely on the other side: a forgotten SWAP is not
//! a missing function, it is the PARENT's, which compiles, runs, and is caught by nothing the
//! ladder does automatically.
//!
//! So, on slice AB's precedent: the three unported bodies are **named panics** rather than
//! `todo!()` (a `todo!()` and the parent's body read alike, and a per-cell message makes the slot
//! addressable), and `R70_TRIPLE` carries **no `..R69_TRIPLE` spread** — the nine inherited cells
//! are decisions on the page.
//!
//! **AND THE STEP-1 GATE IS NOT AB's.** AB asserted distinctness by READING the nine placeholder
//! panic messages, and step 2 deleted every one of them, so the gate had to be dismantled — its
//! whole content was *"not yet ported"*. `tests/slice_ac_cells.rs` asks the same question as
//! **pointer inequality between two shipped `const`s** instead, which survives the bodies landing.
//!
//! # WHAT STEP 1 DELIBERATELY DOES NOT GATE
//!
//! **`at_lever`'s DISPATCH gate cannot be written yet, and its absence is a decision.** § 5.27 (v)
//! measured both `at_lever` swaps: the two bodies differ only in which class they CONSTRUCT, so
//! injecting the parent's makes `_split_rig` hand back the parent's class carrying `_gov_max` as a
//! stray attribute — which Python allows. It is observable **only because the parent's
//! `integrate_fuel` then REFUSES the arming**. In Rust `at_lever` returns a table pointer and
//! nothing refuses anything until those asserts are ported, so a gate written now would report
//! UNOBSERVABLE for a reason about ORDERING rather than about the cell. Booked to step 7.
//!
//! **STEP 2 DISCHARGED THAT PREREQUISITE**: rung 70's five arming refusals are in
//! [`r70_integrate_fuel`] below (guards A–E of § 5.27 (vi), all five measured reachable BY
//! ARMING). So step 7 can write `at_lever`'s dispatch gate directly and does not need to
//! re-derive § (v)'s ordering argument — the observability it was waiting on exists now, and the
//! injection's visible failure is the PARENT's refusal firing, exactly as probe 6 measured.
//!
//! # `_gov_max` — CONFIG-kind, and **THE MIRROR OF `_ref`'s RESTORE POLICY**
//!
//! See [`TwoSpoolTransientCore::gov_max`] for the census. The half worth repeating here is the
//! half that would be wrong if slice AB's reasoning were copied across: `_with_ref` is entered to
//! SET a reference over a `None`, so all 29 of its restores put `None` back and a restore-to-`None`
//! guard agrees with Python on every shipped path. **`_with_gov` is entered to turn the governor
//! OFF**, and that is a CALL-SITE ENUMERATION rather than a sample: `engine.py` holds exactly
//! **three** `_with_gov` call sites in the whole ladder and all three pass a literal `None`. So
//! the two spellings agree at the SET and differ at the RESTORE wherever the receiver's governor
//! is armed — which on a rung-70/71 rig it always is. [`GovScope`] restores the previous value,
//! and unlike [`RefScope`](crate::reference_split::RefScope) that choice is reachable by an
//! ordinary value witness rather than only by a manufactured nest.
//!
//! [`ScheduledStatorCore`]: crate::stator_transient::ScheduledStatorCore
//! [`TwoSpoolTransientCore`]: crate::two_spool_transient::TwoSpoolTransientCore
//! [`TwoSpoolTransientCore::gov_max`]: crate::two_spool_transient::TwoSpoolTransientCore::gov_max

use std::cell::Cell;

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::cross_loop::exceed;
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    point, AccelSchedule, Floor, FuelInstant, FuelLimiters, FuelPoint, FuelTransientCore,
    FuelTransientHooks, PointExtra, SurgeLimiter,
};
use crate::gas::{powp, Abort};
use crate::lagged_bleed::{lagged, py_max3};
use crate::limited_bleed::{BleedLimiter, Regime};
use crate::map::ComponentMap;
use crate::reference_split::{
    build_split_family_cascade, c_add, c_div, c_mul, c_neg, csqrt, cubic_roots_c,
    invariants, opt_fold, py_two, sorted_by_abs, C64,
};
use crate::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
    StatorTransientHooks,
};
use crate::three_loop::{
    closer_b, closer_v, riding, triple_gains_at, v_at_point, LegRegime, StatorLimiter, TripleGains,
    TripleHooks, TripleLaws,
};
use crate::two_lag::violation;
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{
    MarchedBleed, MarchedStator, TwoSpoolTransientCore, TwoSpoolTransientHooks,
};

// ---------------------------------------------------------------------------------------------
// `_gov_max` — THE CARRIER'S GUARD
// ---------------------------------------------------------------------------------------------

/// The RAII form of Python's `_with_gov`'s `try/finally` — **the restore is `Drop`, so it survives
/// an unwind that a straight-line restore would skip.**
///
/// Python is `prev, self._gov_max = self._gov_max, val` … `try: return fn(*a, **kw)` …
/// `finally: self._gov_max = prev`, and its docstring gives the reason in rung 62's words: *a
/// leaked setting would make a reader report a plant that was never marched.*
///
/// **RESTORE-PREVIOUS, AND HERE THAT IS A LIVE CHOICE RATHER THAN A FORMALITY.** Slice AB's
/// [`RefScope`](crate::reference_split::RefScope) restores the previous value too, but measured
/// over its own suite the displaced value was `None` at every one of its 29 value-sets, so
/// restore-previous and restore-`None` agreed on every shipped path and only a manufactured nest
/// could tell them apart. This guard is the MIRROR: `_with_gov` is entered to turn the governor
/// **off** — all THREE of its call sites in the ladder pass a literal `None` — over a `prev` that
/// a rig has just armed, so a restore-to-`None` here would silently disarm the governor for every
/// reader that runs after the scope closes.
///
/// **IT WRITES THE FIELD DIRECTLY AND NOT THROUGH A CELL**, which is the opposite of `RefScope`'s
/// decision and for a measured reason: rung 73 overrides `_with_ref` to write `_ref_law`, so that
/// setter had to be dispatched; `_with_gov` is defined once in the whole ladder and overridden
/// nowhere, so there is no second field for a cell to choose between. If a later rung overrides
/// it, the repair is `RefScope`'s — add the cell — and this comment is where that reader lands.
pub struct GovScope<'a> {
    core: &'a TwoSpoolTransientCore,
    prev: Option<f64>,
}

impl<'a> GovScope<'a> {
    /// Arm (or disarm) the governor's set point for as long as the returned guard lives.
    ///
    /// `None` is a real assignment and not a no-op: it is rung 68's FUEL leg, a different law,
    /// which is exactly what `_with_gov(None, …)` is called 35 times to select.
    pub fn set(core: &'a TwoSpoolTransientCore, val: Option<f64>) -> Self {
        let prev = core.gov_max.get();
        core.gov_max.set(val);
        bump(&GOV_SCOPE_SETS);
        GovScope { core, prev }
    }

    /// What this scope displaced — Python's `prev`, exposed so a gate can read the restore POLICY
    /// rather than only its effect. [`RefScope::displaced`]'s precedent.
    ///
    /// [`RefScope::displaced`]: crate::reference_split::RefScope::displaced
    pub fn displaced(&self) -> Option<f64> {
        self.prev
    }
}

impl Drop for GovScope<'_> {
    fn drop(&mut self) {
        // COUNTED, because this is the half slice AB could not witness: a restore that puts a
        // VALUE back rather than `None` is what distinguishes this policy from `RefScope`'s, and
        // on a rung-70/71 rig it is every restore.
        if self.prev.is_some() {
            bump(&GOV_SCOPE_RESTORED_VALUE);
        }
        self.core.gov_max.set(self.prev);
    }
}

// ---------------------------------------------------------------------------------------------
// THE BUILDER — rung 69's constructor, INHERITED
// ---------------------------------------------------------------------------------------------

/// Rung 70's constructor: **rung 69's, verbatim, with rung 70's five tables.**
///
/// `CrossSplitTransient` defines no `__init__` in Python, so every guard that fires here is one of
/// rung 69's eleven and this function adds none of its own. That is why it is a call to
/// [`build_split_family_cascade`] rather than a re-spelling: re-writing the asserts once per rung
/// would be a copy the SOURCE does not make.
///
/// **The one arming refusal rung 70 does own is not here — it is in `integrate_fuel`**, which
/// refuses an INCIDENCE stator, a `tau_gov` with no `Tt4_max`, rung 52's fuel leg, rungs 50/51's
/// forced release edges, and an instantaneous valve beside a lagged stator. Five asserts, all
/// measured reachable by ARMING (§ 5.27 (vi)); they land at step 2.
pub fn build_cross_split_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    build_split_family_cascade(design_engine, flight_design, mdot_design, map_lp, map_hp, rho,
                               arm, &R70_TWO, &R70_STATOR, &R70_FUEL, &R70, &R70_TRIPLE)
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and THREE of them carry a swap of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 70's lever table — ONE swap, `at_lever`, the sibling constructor that must hand back a
/// rung-70 machine.
///
/// **THE NINTH INSTANCE OF THE TRAP RUNGS 61–69 EACH HIT**, and Python's own docstring says the
/// failure mode is back to rung 67's plain one: the signature does NOT grow here, because rung
/// 70's third loop is armed by a MARCH argument (`tau_gov`) rather than a machine keyword. So a
/// forgotten swap hands back the parent's class and every reader then measures rung 69's plant
/// while reporting rung 70's.
pub const R70: LeverHooks = LeverHooks {
    at_lever: r70_at_lever,
    ..crate::reference_split::R69
};

/// RUNG 70's `TwoSpoolTransientHooks` — **ZERO cells swapped**, named for `R66_TWO`'s reason: a
/// spread of the parent would make the NEXT addition to that table silent here.
pub const R70_TWO: TwoSpoolTransientHooks = crate::reference_split::R69_TWO;

/// RUNG 70's fuel table — ONE swap, `integrate_fuel`: the march that owns rung 70's five arming
/// refusals and then delegates to `_integrate_fuel_cross_triple`.
pub const R70_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r70_integrate_fuel,
    ..crate::reference_split::R69_FUEL
};

/// RUNG 70's stator table — **ZERO cells swapped**, named for the same reason.
pub const R70_STATOR: StatorTransientHooks = crate::reference_split::R69_STATOR;

/// **RUNG 70's THIRD-LOOP TABLE — ONE of the ten cells swapped, and it is the one rung 69 left
/// alone.**
///
/// `triple_laws` is the ninth of rung 68's nine, and rung 70 is the only class in the ladder that
/// overrides it — which is why `R69_TRIPLE` spells it out as `R68_TRIPLE.triple_laws` with a
/// comment naming this rung. That is the chain the swap breaks, and
/// `tests/slice_ac_cells.rs` gates all three links of it.
///
/// The other nine are written out rather than reached through a `..R69_TRIPLE` spread, for
/// `R69_TRIPLE`'s own reason: this slice's entire risk is a swap that is silently the parent's, so
/// every inherited cell is a decision on the page.
pub const R70_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: crate::reference_split::R69_TRIPLE.stator_leg,
    lagged_stator: crate::reference_split::R69_TRIPLE.lagged_stator,
    clamp_v: crate::reference_split::R69_TRIPLE.clamp_v,
    check_v0: crate::reference_split::R69_TRIPLE.check_v0,
    rk4_floor: crate::reference_split::R69_TRIPLE.rk4_floor,
    solve_v: crate::reference_split::R69_TRIPLE.solve_v,
    manifold_v: crate::reference_split::R69_TRIPLE.manifold_v,
    // RUNG 70's OWN — the one cell of rung 68's nine that reached this far unswapped.
    triple_laws: r70_triple_laws,
    triple_rig: crate::reference_split::R69_TRIPLE.triple_rig,
    with_ref: crate::reference_split::R69_TRIPLE.with_ref,
};

// ---------------------------------------------------------------------------------------------
// THE THREE SWAPPED CELLS — BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 70's `at_lever` — **rung 69's sibling constructor returning a RUNG-70 machine**, and the
/// NINTH instance of the trap rungs 61–69 each hit.
///
/// The signature does NOT grow here: rung 70's third loop is armed by a MARCH argument
/// (`tau_gov`) and not by a machine keyword, so the failure mode is back to rung 67's plain one —
/// a forgotten swap hands back the parent's class and every reader then measures rung 69's plant
/// while reporting rung 70's.
///
/// **WHAT MAKES THAT OBSERVABLE IS NOT THIS FUNCTION.** § 5.27 (v) injected rung 69's body into
/// this slot and measured the shape: both bodies differ only in which class they construct, so in
/// Python the sibling comes back carrying `_gov_max` as a stray instance attribute (which Python
/// allows) and NOTHING complains until the parent's `integrate_fuel` REFUSES the arming. In the
/// port the same is true one level down — this returns a table pointer, and the refusal that makes
/// the swap visible is [`r70_integrate_fuel`]'s five asserts, which land in this same step.
///
/// It routes through [`build_cross_split_cascade`], so every sibling re-asserts rung 69's eleven
/// guards — the same reason rung 69's own body routes through its cascade builder.
fn r70_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_cross_split_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 70's `integrate_fuel` — **the reduce arm, then the five arming refusals, then the march.**
///
/// # THE REDUCE IS BY DISPATCH AND ONE TEST COVERS BOTH ARMS
///
/// `tau_gov is None or not lagged_stator()` hands straight back to the parent, and that single
/// condition covers BOTH of § 5.27's reduce arms: no governor ⇒ rung 69/68 and everything under
/// them; no stator ⇒ rung 67, which the parent already dispatches to. **This class never
/// intercepts a march it does not own**, which is what keeps six reduce arms bit-for-bit (P9).
///
/// # `tau_gov` IS READ FROM THE CARRIER AS WELL AS THE ARGUMENT, AND THAT IS LOAD-BEARING
///
/// Rung 67's clock rides on an instance attribute and `_stator_march` does not forward it as a
/// keyword, so reading only the argument would let a rung-70 march SILENTLY BECOME A RUNG-68 ONE
/// — and the refusals below would then never fire. Rung 68's body carries the same note for the
/// same reason; this is not a copy of it but the same source line one rung up.
///
/// # THE FIVE REFUSALS ARE § 5.27 (vi)'s GUARDS A–E, AND ALL FIVE ARE REACHABLE BY ARMING
///
/// | | refuses | points, of 144 swept |
/// |---|---|---|
/// | A | an INCIDENCE stator beside the governor (`n = m = 3` — rung 71's cell) | 24 |
/// | B | `tau_gov` without `Tt4_max` — a governor with no set point | 12 |
/// | C | rung 52's fuel leg beside the governor (`n = 4`, two legs on one actuator) | 3 |
/// | D | rungs 50/51's forced release edges | 18 |
/// | E | an INSTANTANEOUS valve beside a lagged stator | 14 |
///
/// The two guards this rung does NOT reach by arming are `_ic_order` and the joint-IC residual,
/// which live in the march below; § 5.27 (vi) registered both as unreachable-by-construction so a
/// later gate does not report them as dead.
///
/// # A PYTHON CONSEQUENCE, REPRODUCED AND NOT REPAIRED
///
/// Guard C refuses a fuel leg only when a LAG is armed with it (`lag is not None and (accel is not
/// None or surge is not None)`), so an `accel` or a `surge` passed WITHOUT a lag survives every
/// guard — and then `_integrate_fuel_cross_triple` never reads either, because rung 70's `required`
/// is the GOVERNOR's clip and takes no min-select leg. The leg is silently dropped. This port
/// reproduces that exactly, and it is recorded on § 5.27 (x)'s precedent (`SplitWallTransient`
/// inheriting a `rung67_control` that `TypeError`s on it): the port is a TRANSLATION and not a
/// repair (§ 8), so a later slice must not read the behaviour as a port defect.
pub fn r70_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    let lag = lim.lag.or_else(|| ft.inner.lag.get());
    let tau_gov = lim.tau_gov.or_else(|| ft.inner.tau_gov.get());
    if tau_gov.is_none() || !ft.inner.lagged_stator() {
        bump(&INTEGRATE70_REDUCED);
        // EVERY inherited arm leaves through here. The IMMEDIATE parent's table and not rung
        // 68's: `super()` from this class is rung 69, and a grandparent spelling would call a
        // slot that is only ACCIDENTALLY the same pointer today.
        return (crate::reference_split::R69_FUEL.integrate_fuel)(
            ft, flight, fuel_schedule, nu0, s_end, ds,
            &FuelLimiters { tau_gov, lag, ..lim.clone() });
    }
    // GUARD A.
    assert!(ft.inner.stator.inc.is_none(),
            "rung-70 is THREE loops on TWO variables: the governor on `Tt4`, the valve and the \
             STATOR both on `phi`. An INCIDENCE stator here would put all three on DIFFERENT \
             constraints -- `n = m = 3`, ZERO zeros, the one cell of rung 69 s 1's table this \
             ladder has never occupied. That is rung 70's own next seam, asserted against \
             rather than run.");
    // GUARD B.
    let tt4_max = lim.tt4_max;
    assert!(tt4_max.is_some(),
            "rung-70's odd loop IS the redline: `tau_gov` without `Tt4_max` is a governor with \
             no set point, which would march as rung 68 while every reader reported rung 70.");
    // GUARD C.
    assert!(!(lag.is_some() && (lim.accel.is_some() || lim.floor().is_some())),
            "rung-70: rung 52's phi FUEL leg beside this governor is `n = 4, m = 2` -- FOUR \
             loops, and two of them on the same actuator. It is an unregistered plant and the \
             next seam after this one; rung 68's own `tau_gov` assert exists because 'silently \
             accepts it' is the failure mode. Arm one fuel-side leg, not both.");
    // GUARD D.
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-70: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger. All three legs here pin their own (rung 68's \
             argument, verbatim).");
    // GUARD E.
    assert!(ft.inner.lever.lim.is_none() || lagged(&ft.inner),
            "rung-70: an INSTANTANEOUS valve beside a lagged stator is not a control but a \
             different plant (rung 65 called the instantaneous limit singular, and rung 66 \
             refused the comparison for that reason). Give the valve a `tau` or leave it out.");
    r70_integrate_fuel_cross_triple(
        ft, flight, fuel_schedule, nu0, s_end, ds, lim.freeze,
        tt4_max.expect("guard B just fired if this was None"),
        tau_gov.expect("the reduce test above returned if this was None"))
}

/// RUNG 70's `_rk4_floor_split` — **the floor, re-justified a THIRD time on the same constant.**
///
/// Rung 68's `ds*sum(1/tau_i) <= 2` is exact-in-argument there (`J` rank one, non-zero eigenvalue
/// EXACTLY `-sum 1/tau_i`); rung 69 kept the constant on a different argument. Here
/// `min(pair) ~ 0` puts the two non-zero roots back on the REAL axis with the dominant one at
/// `~ -sum 1/tau_i` again, so the constant is conservative for rung 68's reason once more, on a
/// plant rung 68's derivation does not cover. Python RE-STATES it rather than inheriting it,
/// because rung 65 published a retraction for a trusted stability argument.
///
/// # THIS IS NOT THE `rk4_floor` CELL, AND THE DIFFERENCE IS THE WHOLE OF P5
///
/// [`TripleHooks::rk4_floor`](crate::three_loop::TripleHooks::rk4_floor) takes
/// `(ds, rate, n_states, tau_s)`; this takes **three** arguments and is a plain `@staticmethod` in
/// Python, defined once and overridden nowhere. So no function pointer exists for it, nothing in
/// the dispatch harness can substitute one, and its only gate is a `should_panic` on its own RUNG
/// TAG. Probe 2b measured why the tag and not the sentence: `rank TWO` is carried by rung 69's
/// message AND by this one, and only `rung-70: ds` matches exactly one.
///
/// # AND ITS CALL SITE STAYS INSIDE THE MARCH
///
/// § 5.27 (vi) measured the trap. Rung 71's `integrate_fuel` calls its own floor and then
/// delegates to [`r70_integrate_fuel_cross_triple`], which calls this one **on the same condition
/// with the same rate**, so on a rung-71 march rung 71's raises first. Omitting the shadowed call
/// leaves the rung-71 trajectory **identical** (341 points, 3 410 keys) with its own guard still
/// firing — **and deletes rung 70's only floor.** A port that hoists one floor into
/// `integrate_fuel` therefore loses this guard while every rung-71 gate stays green.
pub fn r70_rk4_floor_split(ds: f64, rate: f64, tau_s: f64) {
    assert!(ds * rate <= 2.0,
            "rung-70: ds*sum(1/tau_i) = {:.3} is outside the explicit RK4 stability region for \
             the three actuator states (ds = {ds}, tau_s = {tau_s}). Under the GENERIC split the \
             block is rank TWO but its non-zero pair is REAL and dominated by the rate sum \
             (min(pair) ~ 0), so this is rung 68's bound on rung 68's argument. Refine the grid \
             or slow a clock.", ds * rate);
}

/// One derivative evaluation's full return — Python's eleven-tuple out of rung 70's `der`.
struct CrossTripleDer {
    da: f64,
    dh: f64,
    dg: f64,
    dq: f64,
    dv: f64,
    mf: f64,
    inst: FuelInstant,
    req: f64,
    cmd: f64,
    vcmd: f64,
    vreg: Regime,
}

/// RUNG 70's MARCH — **rung 68's five-state integrator with ONE substitution, the odd loop's
/// SENSOR**, exactly as rung 67 substituted into rung 66's.
///
/// **IT IS A SIBLING, NOT AN EDIT.** Rungs 68/69's arms have to stay bit-for-bit and
/// `tests/test_numeric_fingerprint.py` is the project's only ABSOLUTE gate, so the two integrators
/// are kept apart even where they agree line for line. The port keeps them apart for the same
/// reason and adds one of its own: a shared body would need a flag, and a flag is a cell nobody
/// registered.
///
/// # TWO THINGS DIFFER FROM RUNG 68's MARCH, AND BOTH ARE RUNG 67's PLACEMENT
///
/// * **`Tt4_max` is the GOVERNOR's set point, carried BY THE STATE** (`mf = mf_sched - g`) the way
///   rung 47 carries it — **NOT** rung 52's unlagged min-select on top of the already-clipped
///   fuel. Rung 68's body applies the redline inside `der` before the instant solve; this one does
///   not, and applying both would clip twice with the redline held by an instrument that is not
///   the loop under study.
/// * **`required` is the governor's clip, solved from the SCHEDULED fuel on the plant as the OTHER
///   TWO ACTUATORS ACTUALLY ARE** (`b_state = q`, `v_state = v`). Forget either and that
///   cross-gain is identically zero, the loop silently decouples, and NOTHING FAILS — rung 62's
///   `_powers` trap, sixth reload. [`assert_state_boundary`] is the instrument that refuses to let
///   that pass, and `split_gains` runs it on every interior row.
///
/// Every key rungs 52/65/66/67/68 record is recorded here byte-unchanged (the 24-key
/// [`PointExtra::Triple`] dict, rung 68's own), so every reader in the family works on this
/// trajectory too.
///
/// # THE JOINT INITIAL CONDITION IS RUNG 68's FAMILY, FOR RUNG 68's REASON
///
/// The governor opens DORMANT (the ramp starts below the redline), so `g0 = 0` exactly and rung
/// 67's damped 2×2 solve is not what is needed. What remains is rung 68's situation unchanged: the
/// VALVE and the STATOR are both live at `s = 0` and they SHARE the constraint, so their pairwise
/// contraction is `|C_v V_q| = 1` EXACTLY — marginal. The `s = 0` fixed points are a
/// ONE-PARAMETER FAMILY and a Gauss-Seidel sweep lands on whichever member its ORDER selects. The
/// order is DECLARED, never inferred: `g -> q -> v`, rung 68's, so the rung-68 arm is reached
/// unchanged.
#[allow(clippy::too_many_arguments)]
pub fn r70_integrate_fuel_cross_triple(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, freeze: Option<Spool>, tt4_max: f64, tau_gov: f64,
) -> Vec<FuelPoint> {
    let lim_s = ft.inner.stator_leg().expect("rung-70's march with no stator floor");
    let tau_s = lim_s.tau.expect("rung-70's march on an unlagged stator");
    // The VALVE is OPTIONAL, rung 68's `has_q` verbatim — the ledger's `G`, `S` and `GS` cells are
    // marches of this same integrator with it disarmed, which is what keeps every cell
    // differenceable against every other (rung 63's lesson).
    let has_q = lagged(&ft.inner);
    let tau_q = if has_q {
        Some(ft.inner.lever.lim.expect("has_q").tau.expect("has_q"))
    } else {
        None
    };
    // PYTHON's OWN SUMMATION ORDER: governor, then valve, then stator. Rung 68's Rust accumulates
    // from `1/tau_s` outward, and copying that template here would change the rounding of the
    // argument the floor tests.
    r70_rk4_floor_split(ds,
                        1.0 / tau_gov + (if has_q { 1.0 / tau_q.expect("has_q") } else { 0.0 })
                            + 1.0 / tau_s,
                        tau_s);
    let (tt2, pt2, _) = ft.inner.inlet(flight);

    // THE VALVE law — rung 68's, verbatim. Roots over TRIAL positions, so NO `b_state`; `v_state`
    // IS set, because it solves against the plant as the STATORS actually are.
    let command = |a: f64, h: f64, mf: f64, v: f64| -> Result<f64, Abort> {
        if !has_q {
            return Ok(0.0);
        }
        let _sv = MarchedStator::set(&ft.inner, v);
        let bl = ft.inner.lever.lim.expect("has_q");
        Ok(crate::limited_bleed::r64_solve_b(&bl, closer_b(ft, a, h, mf, tt2, pt2))?.1)
    };

    // THE STATOR law — rung 68's, verbatim. Trials `v`, so NO `v_state`, but `b_state = q`.
    // Returns `(v, regime)`; the regime is CARRIED, never re-derived.
    let stator = |a: f64, h: f64, mf: f64, q: f64| -> Result<(f64, Regime), Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let (_, v, reg) = ft.inner.solve_v(&closer_v(ft, a, h, mf, tt2, pt2))?;
        Ok((v, reg))
    };

    // THE GOVERNOR law — rung 67's `required` with the stator's state added. It trials NEITHER
    // other actuator, so it sees BOTH.
    let required = |a: f64, h: f64, q: f64, v: f64, mf_sched: f64| -> Result<f64, Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let _sv = MarchedStator::set(&ft.inner, v);
        let i = ft.try_instant_fuel(flight, a, h, mf_sched)?;
        if i.base.tt4 <= tt4_max {
            return Ok(0.0);
        }
        Ok(0.0f64.max(mf_sched - ft.try_topping_fuel(flight, a, h, tt4_max, mf_sched)?))
    };

    let der = |a: f64, h: f64, g: f64, q: f64, v: f64, s: f64|
     -> Result<CrossTripleDer, Abort> {
        let mf_sched = fuel_schedule(s);
        let req = required(a, h, q, v, mf_sched)?;
        // THE REDLINE RIDES ON THE STATE (rungs 47/67) — no min-select clip here.
        let mf = 1e-9f64.max(mf_sched - g);
        let inst = {
            let _sb = MarchedBleed::set(&ft.inner, q);
            let _sv = MarchedStator::set(&ft.inner, v);
            ft.try_instant_fuel(flight, a, h, mf)?
        };
        let cmd = command(a, h, mf, v)?;
        let (vcmd, vreg) = stator(a, h, mf, q)?;
        let da = if freeze == Some(Spool::Lp) { 0.0 } else { inst.base.phi_lp_dot / ft.rho() };
        let dh = if freeze == Some(Spool::Hp) { 0.0 } else { inst.base.phi_hp_dot };
        Ok(CrossTripleDer {
            da, dh,
            dg: (req - g) / tau_gov,
            dq: if has_q { (cmd - q) / tau_q.expect("has_q") } else { 0.0 },
            dv: (vcmd - v) / tau_s,
            mf, inst, req, cmd, vcmd, vreg,
        })
    };

    // --- THE JOINT INITIAL CONDITION ----------------------------------------------------------
    let (mut a, mut h) = nu0;
    let mf0 = fuel_schedule(0.0);
    let v0 = ft.inner.v0.get();
    if let Some(x) = v0 {
        ft.inner.check_v0(x, &lim_s);
    }
    // Python raises out of the whole method here — the initial solves sit BEFORE the loop's `try`.
    let raise = |e: Abort| -> ! { panic!("{}", e.0) };
    let mut g = 0.0f64;
    let mut q = command(a, h, mf0, 0.0).unwrap_or_else(|e| raise(e));
    let mut v = v0.unwrap_or(0.0);
    let b0 = ft.inner.b0.get();
    if let Some(x) = b0 {
        q = x;
    }
    let order = ft.inner.ic_order.get();
    assert!({
                let mut cs: Vec<char> = order.chars().collect();
                cs.sort_unstable();
                cs == ['g', 'q', 'v']
            },
            "rung-70 ic_order is a permutation of 'gqv'; got {order:?}");
    let mut res = f64::INFINITY;
    let mut its = 0usize;
    for i in 1..=60usize {
        its = i;
        let (mut gn, mut qn, mut vn) = (g, q, v);
        for k in order.chars() {
            match k {
                'g' => gn = required(a, h, qn, vn, mf0).unwrap_or_else(|e| raise(e)),
                'q' => {
                    if b0.is_none() {
                        qn = command(a, h, 1e-9f64.max(mf0 - gn), vn)
                            .unwrap_or_else(|e| raise(e));
                    }
                }
                'v' => {
                    if v0.is_none() {
                        vn = stator(a, h, 1e-9f64.max(mf0 - gn), qn)
                            .unwrap_or_else(|e| raise(e)).0;
                    }
                }
                _ => unreachable!("the permutation assert above admits only g/q/v"),
            }
        }
        res = py_max3((gn - g).abs(), (qn - q).abs(), (vn - v).abs());
        g = gn;
        q = qn;
        v = vn;
        if res <= 1e-12 {
            break;
        }
    }
    assert!(res <= 1e-9,
            "rung-70: the joint initial condition did not converge (residual {res:.3e} after \
             {its} iterations) in order {order:?}. The two `phi` loops still SHARE a constraint, \
             so their `s = 0` fixed points are a CURVE and a sweep can only land on a member. \
             Report the state and the order; do not raise the cap.");

    // --- THE RK4 LOOP -------------------------------------------------------------------------
    let mut pts: Vec<FuelPoint> = Vec::new();
    let mut s = 0.0f64;
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok(k1) = der(a, h, g, q, v, s) else { break };
        pts.push(point(s, a, h, &k1.inst, k1.mf, fuel_schedule(s),
                       PointExtra::Triple { g, required: k1.req, b: q, b_cmd: k1.cmd,
                                            v, v_cmd: k1.vcmd, v_regime: k1.vreg,
                                            ic_iters: its, ic_res: res, ic_order: order }));
        let stages = (|| -> Result<[f64; 15], Abort> {
            let k2 = der(a + ds / 2.0 * k1.da, h + ds / 2.0 * k1.dh, g + ds / 2.0 * k1.dg,
                         q + ds / 2.0 * k1.dq, v + ds / 2.0 * k1.dv, s + ds / 2.0)?;
            let k3 = der(a + ds / 2.0 * k2.da, h + ds / 2.0 * k2.dh, g + ds / 2.0 * k2.dg,
                         q + ds / 2.0 * k2.dq, v + ds / 2.0 * k2.dv, s + ds / 2.0)?;
            let k4 = der(a + ds * k3.da, h + ds * k3.dh, g + ds * k3.dg, q + ds * k3.dq,
                         v + ds * k3.dv, s + ds)?;
            Ok([k2.da, k2.dh, k2.dg, k2.dq, k2.dv,
                k3.da, k3.dh, k3.dg, k3.dq, k3.dv,
                k4.da, k4.dh, k4.dg, k4.dq, k4.dv])
        })();
        let Ok([k2a, k2h, k2g, k2q, k2v, k3a, k3h, k3g, k3q, k3v, k4a, k4h, k4g, k4q, k4v]) =
            stages else { break };
        a += ds / 6.0 * (k1.da + 2.0 * k2a + 2.0 * k3a + k4a);
        h += ds / 6.0 * (k1.dh + 2.0 * k2h + 2.0 * k3h + k4h);
        g += ds / 6.0 * (k1.dg + 2.0 * k2g + 2.0 * k3g + k4g);
        q += ds / 6.0 * (k1.dq + 2.0 * k2q + 2.0 * k3q + k4q);
        v += ds / 6.0 * (k1.dv + 2.0 * k2v + 2.0 * k3v + k4v);
        // Every position is PHYSICAL (rung 65, verbatim): the actuators' own hardware stops,
        // applied to the STATE and never to a command.
        if has_q {
            let bmax = ft.inner.lever.lim.expect("has_q").b_max;
            q = bmax.min(0.0f64.max(q));
        }
        v = ft.inner.clamp_v(v, &lim_s);
        g = 0.0f64.max(g);
        s += ds;
    }
    pts
}

/// RUNG 70's `_triple_laws` — **rung 68's three closures with `R` swapped for rung 47's GOVERNOR
/// when a set point is armed.**
///
/// `C` and `V` are the PARENT's own closures, untouched — which is what makes the pairwise
/// products a MEASUREMENT rather than a restatement: the two `phi` laws still know nothing of each
/// other or of the governor.
///
/// # THE REDUCE ARM IS `gov_max == None`, AND ITS BREAK SHAPE IS THIS SLICE's SECOND HEADLINE
///
/// With the set point unset this is the parent's answer verbatim, so every rung-68/69 reader
/// reached through a rung-70 machine measures rung 68/69's plant. **That is also how a wrong body
/// in this slot fails, and it does not fail on a value.** § 5.27 (ii) injected rung 68's body here
/// and measured: the governor simply is not there, every sampled point comes back
/// `interior = false`, `rows` goes **2 → 0**, and [`split_gains`] returns **successfully** with
/// every aggregate `None`.
///
/// | | `n_riding` | `len(rows)` | `worst_CV` | `min_pair_gap` | `pair_RC` |
/// |---|---|---|---|---|---|
/// | shipped | 61 | **2** | 1.061e−10 | 1.132 | (−0.0167, −0.0190) |
/// | rung 68's body | 61 | **0** | `None` | `None` | `()` |
///
/// **BOTH ROWS ARE AT § (ii)'s OWN STRIDE**, which delivers `len(rows) = 2`. The port's own
/// drive-once check (§ 5.27.2 (f)) runs the reader's shipped `every = 10` and gets 7 rows, so
/// its `pair_RC` spans −0.0167…−0.0199 over a longer window — a WIDER sample of the same
/// trajectory, not a different plant. Said here because the numbers above are copied from
/// Python and a reader diffing them against a 7-row run would book a defect that is a stride.
///
/// A dispatch gate of the shape every previous slice has written — march both, diff the value keys
/// — compares two empty tables and passes. **The gate for this cell is a NON-EMPTINESS assertion**
/// (P2), and the general form is worth carrying: *a cell whose output is a SAMPLE can break by
/// changing the sample's SIZE rather than its values, and a value-diff gate is blind to that by
/// construction.* Rung 70's readers are all sample-shaped, so every one of them can fail this way.
///
/// # THE `R` CLOSURE's KINK IS RUNG 52's, ON A DIFFERENT SENSOR
///
/// Rung 47's law has a `max(0, .)` at its own dormant edge, so a central difference straddling it
/// returns the slope of neither branch. The regime label is what the caller filters on, which is
/// why it is CARRIED as a [`LegRegime`] and never re-derived from the float — the same discipline,
/// and the same type, as the fuel leg it replaces.
fn r70_triple_laws<'a>(
    core: &'a ScheduledStatorCore, flight: &'a FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&'a AccelSchedule>, surge: Option<&'a Floor>,
) -> Result<TripleLaws<'a>, Abort> {
    // `super()` from rung 70 is rung 69 — spelled through the IMMEDIATE parent's table, even
    // though rung 69 inherits this cell from rung 68 and the two pointers are equal today.
    let laws = (crate::reference_split::R69_TRIPLE.triple_laws)(
        core, flight, a, h, mf_sched, accel, surge)?;
    let Some(tt4_max) = core.fuel.inner.gov_max.get() else {
        bump(&TRIPLE_LAWS70_PARENT);
        return Ok(laws);
    };
    bump(&TRIPLE_LAWS70_GOV);
    let ft = &core.fuel;
    let r = move |q: f64, v: f64| -> Result<(f64, LegRegime), Abort> {
        let raw = {
            let _sb = MarchedBleed::set(&ft.inner, q);
            let _sv = MarchedStator::set(&ft.inner, v);
            let i = ft.try_instant_fuel(flight, a, h, mf_sched)?;
            if i.base.tt4 <= tt4_max {
                // Python returns from INSIDE the `try`, so the `finally` restores on the way out;
                // here the two guards drop at the `return` for the same effect.
                return Ok((0.0, LegRegime::Dormant));
            }
            mf_sched - ft.try_topping_fuel(flight, a, h, tt4_max, mf_sched)?
        };
        Ok((0.0f64.max(raw),
            if raw > 0.0 { LegRegime::Riding } else { LegRegime::Dormant }))
    };
    Ok(TripleLaws { r: Box::new(r), c: laws.c, v: laws.v })
}

// ---------------------------------------------------------------------------------------------
// THE RIG, THE BOUNDARY INSTRUMENT AND THE DAMPING READER — the three PLAIN methods the readers
// stand on
// ---------------------------------------------------------------------------------------------

/// RUNG 70's `_split_rig` — **ONE constructor for every cell of every table here**, with any
/// SUBSET of the two AIRFLOW loops armed and the governor's set point attached.
///
/// A cell can differ from another only by which loops are armed (rung 63's lesson, and the reason
/// the credits are differenceable at all). Both floors come from the SAME `from_margin(cmap, ., sm)`,
/// which under THIS rung is not a nicety: `pair_CV = 1` is an identity of a SHARED constraint, and
/// a set-point offset would break it and look like a failed prediction.
///
/// # THE SET POINT IS A BARE, PERMANENT ASSIGNMENT — **NOT [`GovScope`]**
///
/// Python is `m = self.at_lever(...)` then `m._gov_max = Tt4_max`, and § 5.27 (vii) calls this out
/// as the half a `try/finally` census cannot see: `_gov_max` is written **two different ways**,
/// bare on a FRESH machine here and save/set/restore in `_with_gov`. Reaching for the RAII guard
/// here would restore on drop, [`r70_triple_laws`] would then find `gov_max == None`, take its
/// reduce arm, and **every reader would measure rung 68** — which per § 5.27 (ii) returns
/// successfully with ZERO rows, so a value-diff gate passes on two empty tables. It is the
/// highest-consequence single line in the port of this rung.
///
/// # AND THE SET IS UNCONDITIONAL, INCLUDING FOR [`split_bill`]'s GOVERNOR-LESS CELLS
///
/// The ledger's `bare`/`V`/`S`/`VS` cells turn the governor off **at the MARCH** (`Tt4_max=None`,
/// `tau_gov=None`), never at the rig. Adding a `gov` flag here would be a second way to disarm the
/// same loop, and the two would then have to be kept in step — exactly the kind of duplication
/// rung 63's one-constructor rule exists to prevent.
///
/// `b_max` falls back to Python's own `0.10` when the receiver carries no valve.
#[allow(clippy::too_many_arguments)]
pub fn split_rig(
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
        Some(StatorLimiter::from_margin(&cmap, v_max, sm, Some(tau_s)))
    } else {
        None
    };
    let m = core.at_lever(&LeverArm { bleed_lim: bl, stator_lim: sl, ..Default::default() });
    // THE BARE, PERMANENT SET. See the note above — this is NOT `GovScope`.
    m.fuel.inner.gov_max.set(Some(tt4_max));
    bump(&SPLIT_RIG_CALLS);
    m
}

/// One point's reading of [`assert_state_boundary`] — Python's `dict(s=…, live=…, dead=…)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateBoundary {
    pub s: f64,
    /// The governor's cross-gains WITH the `b_state`/`v_state` boundary — both non-zero.
    pub live_r_q: f64,
    pub live_r_v: f64,
    /// The same two with the boundary DROPPED — both identically zero, by construction.
    pub dead_r_q: f64,
    pub dead_r_v: f64,
}

/// RUNG 70's `_assert_state_boundary` — **the one thing rung 68 says can go wrong silently,
/// ASSERTED rather than inherited.**
///
/// `R_q != 0` and `R_v != 0` ONLY because the governor senses `Tt4` on the machine as the other
/// two actuators actually are. Drop the state boundary around `required` and both cross-gains are
/// identically zero: the odd loop DECOUPLES, `m` reads 1 instead of 2 **by accident**, `c1`
/// collapses — and every prediction in this rung would 'confirm' rung 68 instead.
///
/// So the boundary is measured against its own broken version. **`blind` deliberately ignores both
/// of its arguments**: that is the failure mode built on purpose, not an oversight, and the whole
/// content of the `dead` column is that four calls at four different arguments return the same
/// number. Keeping the parameters is what makes the two central differences the SAME expression
/// with one thing changed.
///
/// [`split_gains`] runs this on every interior row, which makes the `MarchedBleed`/`MarchedStator`
/// placement inside [`r70_triple_laws`]'s `R` self-checking: a port that lost either guard fails
/// here rather than quietly reporting rung 68's plant.
pub fn assert_state_boundary(
    core: &ScheduledStatorCore, flight: &FlightCondition, p: &FuelPoint, tt4_max: f64,
    dq: f64, dv: f64,
) -> Result<StateBoundary, Abort> {
    let (a, h, mf_sched) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let q = crate::lagged_bleed::valve_of(p).0;
    let v = v_at_point(p);
    let laws = core.triple_laws(flight, a, h, mf_sched, None, None)?;
    let ft = &core.fuel;

    // `required` WITHOUT the state boundary — the failure mode, built on purpose. Both parameters
    // are unused ON PURPOSE; that is the property under test.
    let blind = |_qq: f64, _vv: f64| -> Result<f64, Abort> {
        let i = ft.try_instant_fuel(flight, a, h, mf_sched)?;
        if i.base.tt4 <= tt4_max {
            return Ok(0.0);
        }
        Ok(0.0f64.max(mf_sched - ft.try_topping_fuel(flight, a, h, tt4_max, mf_sched)?))
    };

    let live_r_q = ((laws.r)(q + dq, v)?.0 - (laws.r)(q - dq, v)?.0) / (2.0 * dq);
    let live_r_v = ((laws.r)(q, v + dv)?.0 - (laws.r)(q, v - dv)?.0) / (2.0 * dv);
    let dead_r_q = (blind(q + dq, v)? - blind(q - dq, v)?) / (2.0 * dq);
    let dead_r_v = (blind(q, v + dv)? - blind(q, v - dv)?) / (2.0 * dv);
    assert!(dead_r_q == 0.0 && dead_r_v == 0.0,
            "rung-70: the BLIND control is supposed to be identically zero -- if it is not, this \
             instrument is not measuring what it claims.");
    assert!(live_r_q.abs() > 0.0 && live_r_v.abs() > 0.0,
            "rung-70: the governor's cross-gains came back R_q = {live_r_q}, R_v = {live_r_v} at \
             s = {}. A ZERO cross-gain is not a weak coupling, it is a MISSING one (rung 67's \
             gate): the b_state/v_state boundary around `required` has been lost, and with it the \
             second constraint. Every prediction in this rung would then confirm rung 68.", p.s);
    Ok(StateBoundary { s: p.s, live_r_q, live_r_v, dead_r_q, dead_r_v })
}

/// RUNG 70's `_zeta_pair` — **the damping ratio of the NON-ZERO PAIR, and it CANNOT be rung 69's
/// reader.**
///
/// Rung 69 reads `zeta = -Re(dom)/|dom|` off the DOMINANT root, which is exact for the complex
/// pair it measures and returns exactly `1.0` for ANY real root. Here the pair is predicted REAL,
/// so that reader would report `zeta = 1` on every arm and the floor claim would be untestable —
/// an instrument that cannot distinguish 'critically damped' from 'overdamped by 3×' cannot
/// measure a bound whose whole content is how much margin the plant has above 1.
///
/// So the pair is read the way the closed form defines it, from BOTH non-zero roots:
///
/// ```text
/// zeta = -(lam1 + lam2) / (2 sqrt(lam1 lam2))
/// ```
///
/// which is `-Re/|lam|` when they are conjugate and `>= 1` when they are real. That makes `zeta`
/// and `zeta_pred` the SAME quantity, so their agreement is a check on the algebra rather than a
/// comparison of two different definitions.
///
/// # FOUR COMPLEX OPERATIONS, AND EXACTLY ONE OF THEM COSTS SOMETHING
///
/// § 5.27 (iv) replayed all 18 captured calls against the spelling a port reaches for without
/// reading CPython: the complex product agrees **18/18**, `cmath.sqrt` agrees **18/18**, and the
/// **complex division agrees on only 13/18** (worst gap 4.44e−16 absolute, with the returned
/// `.real` differing on the same five). So [`c_div`] is Smith's algorithm, and `2.0 * rt` goes
/// through [`py_two`] because a `float * complex` is a four-multiply product and not a scaling —
/// slice AB found ONE port defect in 15 957 keys and it was that exact class at `0.5`.
///
/// **`p` IS NOT ALWAYS REAL, AND THE PORT LEARNED THAT FROM A SHIPPED TEST.** § 5.27 (iv) measured
/// it positive-real on 18 of 18 calls of the rung-70 READERS and step 3 shipped that as an
/// `assert!`; `tests/test_rung71.py`'s damping-reader gate then handed this function the spectrum
/// `[-194, -23 +/- 25.5i]`, whose two largest moduli are one real root and ONE member of the pair,
/// and the assertion refused a call Python answers with `1.278`. [`csqrt`] is now CPython's full
/// `c_sqrt` and the real branch is unmoved -- see its own note, and RULE 4 in
/// `tests/porting_rules.rs`.
pub fn zeta_pair(roots: [C64; 3]) -> Option<f64> {
    let sorted = sorted_by_abs(roots);
    // Python's `sorted(roots, key=abs)[1:]` — the two LARGEST by modulus, stably ordered.
    let (n0, n1) = (sorted[1], sorted[2]);
    let s = c_add(n0, n1);
    let p = c_mul(n0, n1);
    let rt = csqrt(p);
    if rt.abs() == 0.0 {
        None
    } else {
        Some(c_div(c_neg(s), py_two(rt)).re)
    }
}

// ---------------------------------------------------------------------------------------------
// § 1 — THE PAIRWISE SPLIT, AND RUNG 67 AS THE BUILT-IN CONTROL
// ---------------------------------------------------------------------------------------------

/// One sampled point of [`split_gains`].
#[derive(Clone, Debug)]
pub struct SplitGainsRow {
    pub s: f64,
    /// RUNG 70's plant — the governor armed.
    pub gov: TripleGains,
    /// RUNG 68's `R` re-read at the IDENTICAL point. It is not marched, it is the CONTRAST, and
    /// it is what shows that the identity MOVES from `(R,C)` to `(C,V)` rather than merely
    /// appearing somewhere new.
    pub fuel: TripleGains,
    /// `|pair_RC - pair_RV| / max(|·|, 1e-300)` — ZERO at rung 69, non-zero here. **THE RUNG.**
    pub pair_gap: f64,
    /// `|cyclic + pair_RC|` — the cyclic product is `-pair_RC` IDENTICALLY, so it is structurally
    /// blind to `pair_RV`.
    pub cyclic_is_rc: f64,
}

/// RUNG 70 § 1's return.
#[derive(Clone, Debug)]
pub struct SplitGains {
    pub n_riding: usize,
    pub n_sampled: usize,
    pub rows: Vec<SplitGainsRow>,
    /// DISCLOSED, never a silent truncation — `(s, off-regime arms)`.
    pub skipped: Vec<(f64, Vec<&'static str>)>,
    pub boundary: Vec<StateBoundary>,
    pub s_window: Option<(f64, f64)>,
    pub worst_cv: Option<f64>,
    pub worst_rc_is_1: Option<f64>,
    pub worst_rv_is_1: Option<f64>,
    pub min_pair_gap: Option<f64>,
    pub max_pair_gap: Option<f64>,
    pub worst_cyclic_is_rc: Option<f64>,
    pub worst_rc_fuel: Option<f64>,
    pub pair_rc: Vec<f64>,
    pub pair_rv: Vec<f64>,
    pub worse_pair: Option<f64>,
}

/// RUNG 70 § 1 — **the six cross-gains and the three pairwise products, on ONE trajectory, under
/// BOTH odd loops at the SAME base points.**
///
/// | reading | what it carries |
/// |---|---|
/// | `pair_CV` | the SHARED pair. 1 to the differencing floor under `gov`. |
/// | `pair_RC` | SPLIT — and it IS rung 67's `P`, so it doubles as the negative control. |
/// | `pair_RV` | SPLIT — and the cyclic product CANNOT SEE IT (`x = -pair_RC`). |
/// | `pair_gap` | ZERO at rung 69, non-zero here. **THE RUNG.** |
///
/// # THE TWO SPELLINGS OF THE SET POINT APPEAR IN ONE BODY, AND SWAPPING THEM DIFFS CLEAN
///
/// The `gov` arm reads the rig's own BARE, permanent set point ([`split_rig`]); the `fuel` arm
/// turns it off for the duration of one call through [`GovScope`], which is Python's `_with_gov`.
/// Getting them the other way round compiles, runs, and produces a table — of rung 68's plant.
///
/// **BOTH ARMS ARE EVALUATED BEFORE EITHER REGIME IS INSPECTED**, exactly as Python's two
/// statements sit above the `if not gov["interior"]`, so the closure-call count a counter can read
/// does not depend on which arm was off.
#[allow(clippy::too_many_arguments)]
pub fn split_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, r: f64, s_settle: f64, ds: f64, tau: f64, tau_gov: f64, tau_s: f64, v_max: f64,
    every: usize,
) -> SplitGains {
    let m = split_rig(core, sm, tau, tau_s, v_max, tt4_max, true, true);
    let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
    let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
    let (traj, _) = m.stator_march_scoped(
        flight, &ramp, None, &leg, &MarchScope { tau_gov: Some(tau_gov), ..MarchScope::DEFAULT });
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding(&traj, b_max);
    // Rung 68's control floor, built off the RECEIVER's design map exactly as Python does
    // (`self.map_lp_design`, not `m`'s) — they are equal, and the spelling is the claim.
    //
    // **HOISTED OUT OF THE LOOP, WHICH IS A DEVIATION AND IS THEREFORE STATED.** Python builds it
    // per sampled point. `SurgeLimiter::from_margin` is two asserts on loop-invariant arguments
    // and a struct literal — no state, no counter, no closure call the plant can see — so the two
    // spellings are indistinguishable by every value key AND by every counter that exists. It is
    // hoisted only because that is true; a constructor with a side effect would stay inside.
    let ctrl = Floor::Phi(SurgeLimiter::from_margin(&core.arming().map_lp_design, Spool::Lp, sm));
    // Python's `pts[::every]`; `every = 0` raises there and panics here, which is the same
    // refusal rather than a silently accepted stride.
    let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
    let (mut rows, mut skipped, mut boundary) = (Vec::new(), Vec::new(), Vec::new());
    for p in &sampled {
        let gov = triple_gains_at(&m, flight, p, None, None, 1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-70's gains march does not abort");
        let fuel = {
            let _g = GovScope::set(&m.fuel.inner, None);
            triple_gains_at(&m, flight, p, None, Some(&ctrl), 1e-7, 1e-5, 1e-4, true, 0.0, true)
                .expect("rung-70's gains march does not abort")
        };
        if !gov.interior {
            skipped.push((p.s, gov.off_regime.clone()));
            continue;
        }
        boundary.push(assert_state_boundary(&m, flight, p, tt4_max, 1e-5, 1e-4)
                          .expect("rung-70's boundary instrument does not abort"));
        // Python's THREE-argument `max`, so `py_max3` and not a chain of `f64::max`.
        let den = py_max3(gov.pair_rc.abs(), gov.pair_rv.abs(), 1e-300);
        rows.push(SplitGainsRow {
            s: p.s,
            pair_gap: (gov.pair_rc - gov.pair_rv).abs() / den,
            cyclic_is_rc: (gov.cyclic + gov.pair_rc).abs(),
            gov,
            fuel,
        });
    }
    SplitGains {
        n_riding: pts.len(),
        n_sampled: sampled.len(),
        s_window: if pts.is_empty() { None } else { Some((pts[0].s, pts[pts.len() - 1].s)) },
        worst_cv: opt_fold(rows.iter().map(|x| (x.gov.pair_cv - 1.0).abs()), f64::max),
        worst_rc_is_1: opt_fold(rows.iter().map(|x| (x.gov.pair_rc - 1.0).abs()), f64::max),
        worst_rv_is_1: opt_fold(rows.iter().map(|x| (x.gov.pair_rv - 1.0).abs()), f64::max),
        min_pair_gap: opt_fold(rows.iter().map(|x| x.pair_gap), f64::min),
        max_pair_gap: opt_fold(rows.iter().map(|x| x.pair_gap), f64::max),
        worst_cyclic_is_rc: opt_fold(rows.iter().map(|x| x.cyclic_is_rc), f64::max),
        worst_rc_fuel: opt_fold(rows.iter().filter(|x| x.fuel.interior)
                                    .map(|x| (x.fuel.pair_rc - 1.0).abs()), f64::max),
        pair_rc: rows.iter().map(|x| x.gov.pair_rc).collect(),
        pair_rv: rows.iter().map(|x| x.gov.pair_rv).collect(),
        worse_pair: opt_fold(rows.iter().map(|x| x.gov.pair_rc.min(x.gov.pair_rv)), f64::min),
        rows,
        skipped,
        boundary,
    }
}

/// RUNG 70's `rung67_control` return.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rung67Control {
    pub n: usize,
    pub p70_lo: Option<f64>,
    pub p70_hi: Option<f64>,
    pub p67_lo: f64,
    pub p67_hi: f64,
    pub both_negative: Option<bool>,
    pub ratio: Option<f64>,
}

/// RUNG 70's `rung67_control` — **THE NEGATIVE CONTROL, AND IT IS BUILT IN.**
///
/// `pair_RC` here IS rung 67's `P = R_q C_g`: same governor, same valve, same shipped closures.
/// The ONLY difference is that a third loop is present and has moved the base point, so the two
/// must agree in SIGN and ORDER OF MAGNITUDE, and a departure beyond that is a broken state
/// boundary rather than a plant that changed. It is reported as a RATIO and never gated to a
/// tolerance the base-point shift does not justify.
///
/// # THE REFERENCE MARCH RUNS ON `cross_identity`'s OWN DEFAULTS, NOT THIS READER's
///
/// Python passes only `flight, Tt4_lo, Tt4_hi, Tt4_max, tau, tau_govs` — so `n_sample = 12`,
/// `r = 0.5`, `s_settle = 1.2` and **`ds = 0.0025`** come from rung 67's signature and not from
/// the `r`/`s_settle`/`ds` this reader was called with. Forwarding this reader's grid instead
/// would re-grid the control and quietly change what the ratio compares.
///
/// **This is also the method § 5.27 (i) used to prove `split_gains` is a NAME REUSED**: it calls
/// `self.split_gains(..., sm, tau=…, tau_gov=…, tau_s=…)`, which on a rung-80 machine raises
/// `TypeError: got 3 unexpected keyword arguments`. In Python that inherited caller is BROKEN on
/// rung 80 and no test exercises it; the port is a translation and not a repair, so the Rust is
/// free of it by construction — recorded here so a later slice does not read the absence as a
/// port defect.
#[allow(clippy::too_many_arguments)]
pub fn rung67_control(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, tau: f64, tau_gov: f64, tau_s: f64, v_max: f64, r: f64, s_settle: f64, ds: f64,
    every: usize,
) -> Rung67Control {
    let got = split_gains(core, flight, tt4_lo, tt4_hi, tt4_max, sm, r, s_settle, ds, tau,
                          tau_gov, tau_s, v_max, every);
    // RUNG 67's OWN defaults — see the note above.
    let ref_ramp = Ramp { tt4_lo, tt4_hi, r: 0.5, s_settle: 1.2, ds: 0.0025 };
    let rig = split_rig(core, sm, tau, tau_s, v_max, tt4_max, true, false);
    let reference = rig.cross_identity(flight, &ref_ramp, tt4_max, tau, &[tau_gov], 12);
    let p70 = &got.pair_rc;
    let n = p70.len();
    // Python's `sum(P70)` over a list of floats — a LEFT FOLD from 0.0. § 5.27 (iv) measured this
    // site (len 7) agreeing with the naive fold on BOTH interpreters, so it is not in P8's
    // exemption.
    let mean = if n == 0 { None } else { Some(p70.iter().fold(0.0f64, |a, b| a + b) / n as f64) };
    Rung67Control {
        n,
        p70_lo: opt_fold(p70.iter().copied(), f64::min),
        p70_hi: opt_fold(p70.iter().copied(), f64::max),
        p67_lo: reference.prod_lo,
        p67_hi: reference.prod_hi,
        both_negative: if n == 0 {
            None
        } else {
            Some(p70.iter().all(|&x| x < 0.0) && reference.all_negative)
        },
        ratio: mean.map(|m| m / (0.5 * (reference.prod_lo + reference.prod_hi))),
    }
}

// ---------------------------------------------------------------------------------------------
// § 2 — THE SPECTRUM: one zero, `det` blind, `c1` alive and CLOCK-WEIGHTED
// ---------------------------------------------------------------------------------------------

/// One sampled point of one [`split_modes`] arm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitModesRow {
    pub s: f64,
    pub c2: f64,
    pub c1: f64,
    pub c0: f64,
    pub roots: [C64; 3],
    /// § 1.4's closed form, quoted BESIDE the shipped cubic's own root: `c1` is predicted to be
    /// the CLOCK-WEIGHTED sum of two DIFFERENT split pairs, and quoting only the cubic would hide
    /// which term won.
    pub c1_pred: f64,
    pub c1_err: Option<f64>,
    pub pair_rc: f64,
    pub pair_rv: f64,
    pub pair_cv: f64,
    pub cyclic: f64,
    pub zeta: Option<f64>,
    pub complex_pair: bool,
    pub n_zero: usize,
    pub worst_zero: f64,
    pub c1_rel: f64,
    pub c0_rel: f64,
}

/// One clock triple's arm of [`split_modes`].
#[derive(Clone, Debug)]
pub struct SplitModesArm {
    /// `(tau_g, tau_q, tau_s)` — the `(g, q, v)` order of the STATE VECTOR, which is not the
    /// order the `clocks` grid is written in.
    pub taus: (f64, f64, f64),
    pub rate_sum: f64,
    pub n: usize,
    pub n_sampled: usize,
    pub skipped: usize,
    pub rows: Vec<SplitModesRow>,
    pub zeros: Vec<usize>,
    pub max_c0_rel: Option<f64>,
    pub min_c1_rel: Option<f64>,
    pub max_c1_err: Option<f64>,
    pub any_complex: Option<bool>,
    pub zeta_range: (Option<f64>, Option<f64>),
}

/// RUNG 70 § 2's return.
#[derive(Clone, Debug)]
pub struct SplitModes {
    pub clocks: Vec<(f64, f64, f64)>,
    pub ds: f64,
    pub arms: Vec<SplitModesArm>,
}

/// RUNG 70 § 2 — **§ 1's spectrum across a clock grid.** `clocks` is `(tau_q, tau_gov, tau_s)`,
/// rung 68/69's ordering of the same grid, so the arms line up row for row.
///
/// | | what it says |
/// |---|---|
/// | `zeros` | `n - m` = 1. The same cell as rung 69, reached WITHOUT an incidence wall. |
/// | `c0` | `det J` = 0. BLIND to this split too — the valve and the stator keep exactly parallel rows whatever the governor watches. |
/// | `c1` | NON-ZERO, and the discriminator AGAIN. But it is now a CLOCK-WEIGHTED SUM of two different split pairs, so unlike rung 69's `(1-k) A z` it MOVES under a re-weighting of the clocks at FIXED plant. |
///
/// `c1_pred` is the closed form, reported beside the shipped cubic's own coefficient; that the two
/// agree is a check on the algebra, and [`c1_clock_swap`] is what tests the claim the agreement
/// cannot (a formula agreeing with itself is rung 67 gate 9's tautology).
#[allow(clippy::too_many_arguments)]
pub fn split_modes(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, clocks: &[(f64, f64, f64)], r: f64, s_settle: f64, ds: f64, v_max: f64,
    every: usize,
) -> SplitModes {
    let mut arms = Vec::new();
    for &(tau_q, tau_g, tau_s) in clocks {
        let m = split_rig(core, sm, tau_q, tau_s, v_max, tt4_max, true, true);
        let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
        let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
        let (traj, _) = m.stator_march_scoped(
            flight, &ramp, None, &leg,
            &MarchScope { tau_gov: Some(tau_g), ..MarchScope::DEFAULT });
        let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
        let pts = riding(&traj, b_max);
        // The `(g, q, v)` order of the state vector — NOT the grid's `(q, g, s)` order.
        let taus = (tau_g, tau_q, tau_s);
        // Python's `sum(1.0 / t for t in taus)` — a three-term LEFT FOLD, measured identical on
        // both interpreters (§ 5.27 (iv)).
        let rate = 1.0 / taus.0 + 1.0 / taus.1 + 1.0 / taus.2;
        let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
        let (mut rows, mut skipped) = (Vec::new(), 0usize);
        for p in &sampled {
            let gg = triple_gains_at(&m, flight, p, None, None, 1e-7, 1e-5, 1e-4, true, 0.0, true)
                .expect("rung-70's spectrum march does not abort");
            if !gg.interior {
                // DISCLOSED below, never a silent truncation.
                skipped += 1;
                continue;
            }
            let (c2, c1, c0) = invariants(&gg, taus);
            let roots = cubic_roots_c(c2, c1, c0);
            let nz = sorted_by_abs(roots);
            let dom = nz[2];
            let c1_pred = (1.0 - gg.pair_rc) / (tau_g * tau_q)
                + (1.0 - gg.pair_rv) / (tau_g * tau_s);
            rows.push(SplitModesRow {
                s: p.s, c2, c1, c0, roots, c1_pred,
                c1_err: if c1_pred != 0.0 { Some((c1 / c1_pred - 1.0).abs()) } else { None },
                pair_rc: gg.pair_rc, pair_rv: gg.pair_rv, pair_cv: gg.pair_cv,
                cyclic: gg.cyclic,
                zeta: zeta_pair(roots),
                complex_pair: dom.im.abs() > 1e-6 * dom.abs(),
                n_zero: roots.iter().filter(|x| x.abs() < 1e-4 * rate).count(),
                worst_zero: nz[0].abs(),
                // `rate ** 2` MULTIPLIES and `rate ** 3` calls `pow` — PyPy's JIT rewrites the
                // square and not the cube (`tests/porting_rules.rs` RULE 2).
                c1_rel: c1.abs() / (rate * rate),
                c0_rel: c0.abs() / powp(rate, 3.0),
            });
        }
        let mut zeros: Vec<usize> = rows.iter().map(|x| x.n_zero).collect();
        zeros.sort_unstable();
        zeros.dedup();
        arms.push(SplitModesArm {
            taus,
            rate_sum: -rate,
            n: pts.len(),
            n_sampled: sampled.len(),
            skipped,
            zeros,
            max_c0_rel: opt_fold(rows.iter().map(|x| x.c0_rel), f64::max),
            min_c1_rel: opt_fold(rows.iter().map(|x| x.c1_rel), f64::min),
            max_c1_err: opt_fold(rows.iter().filter_map(|x| x.c1_err), f64::max),
            any_complex: if rows.is_empty() { None }
                         else { Some(rows.iter().any(|x| x.complex_pair)) },
            zeta_range: (opt_fold(rows.iter().filter_map(|x| x.zeta), f64::min),
                         opt_fold(rows.iter().filter_map(|x| x.zeta), f64::max)),
            rows,
        });
    }
    SplitModes { clocks: clocks.to_vec(), ds, arms }
}

/// One arm of [`c1_clock_swap`] — a MARCHED plant at one clock assignment.
#[derive(Clone, Debug)]
pub struct C1SwapArm {
    pub taus: (f64, f64, f64),
    pub s: f64,
    pub c1_marched: f64,
    pub pair_rc: f64,
    pub pair_rv: f64,
    pub gains: TripleGains,
}

/// A `c1` pair under the two clock assignments, plus their ratio — Python's `held` and `one`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct C1Pair {
    pub c1_fast_valve: f64,
    pub c1_fast_stator: f64,
    pub ratio: f64,
}

/// RUNG 70 § 1.4's return.
#[derive(Clone, Debug)]
pub struct C1ClockSwap {
    pub fast_valve: C1SwapArm,
    pub fast_stator: C1SwapArm,
    /// ONE plant's gains under BOTH clock assignments — the pure discrimination.
    pub held_gains: C1Pair,
    /// What a ONE-SCALAR plant would have given, built from THIS plant's own gains.
    pub one_scalar_null: C1Pair,
    pub k_null: f64,
    pub marched_ratio: f64,
    /// **THE ONLY closed-form quantity here, and the one under test.**
    pub predicted_delta: f64,
    pub measured_delta: f64,
    pub null_delta: f64,
}

/// RUNG 70 § 1.4 — **THE DISCRIMINATING TEST, and the one reading that cannot be fooled.**
///
/// That `c1 != 0` is rung 69's result, not this rung's; that `c1` MOVES across a clock grid proves
/// nothing (the rate sum moves too); and that the measured `c1` matches the two-term closed form
/// to 1e−10 only validates the formula against itself.
///
/// **WHAT DISCRIMINATES IS A SWAP.** Hold `tau_g` and exchange `(tau_q, tau_s)`:
///
/// ```text
/// one scalar (rung 69's shape, u == w):  c1 = u (1/(tau_g tau_q) + 1/(tau_g tau_s))
///                                         -- SYMMETRIC in the exchange => INVARIANT
/// two terms  (this rung):                 c1 changes by
///                                         (u - w)(1/(tau_g tau_q) - 1/(tau_g tau_s))
/// ```
///
/// The gains are evaluated ONCE and re-weighted under both clock assignments, so the comparison
/// isolates the CLOCKS from the plant; each arm's own marched `c1` is reported beside it as the
/// realism check.
///
/// # EVERY `c1` BELOW COMES FROM THE SHIPPED [`invariants`], NEVER FROM THE CLOSED FORM
///
/// That distinction is the whole gate. Evaluating the closed form under two clock assignments and
/// reporting that it changed would be rung 67 gate 9's TAUTOLOGY — a formula agreeing with itself.
/// The closed form appears exactly once, as `predicted_delta`, and it is the thing under test.
///
/// The NULL forces `pair_RC == pair_RV == k` at their mean **through the gains that carry them**
/// (`R_q`, `R_v`), so `pair_CV` is untouched and the null differs from the plant in exactly one
/// respect.
#[allow(clippy::too_many_arguments)]
pub fn c1_clock_swap(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, tau_g: f64, fast: f64, slow: f64, r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> C1ClockSwap {
    let mut built: Vec<C1SwapArm> = Vec::new();
    for (name, (tau_q, tau_s)) in [("fast_valve", (fast, slow)), ("fast_stator", (slow, fast))] {
        let m = split_rig(core, sm, tau_q, tau_s, v_max, tt4_max, true, true);
        let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
        let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
        let (traj, _) = m.stator_march_scoped(
            flight, &ramp, None, &leg,
            &MarchScope { tau_gov: Some(tau_g), ..MarchScope::DEFAULT });
        let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
        let pts = riding(&traj, b_max);
        assert!(!pts.is_empty(),
                "rung-70 c1_clock_swap: no riding-interior window on arm {name}");
        let p = &pts[pts.len() / 2];
        let gg = triple_gains_at(&m, flight, p, None, None, 1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-70's clock-swap march does not abort");
        assert!(gg.interior,
                "rung-70 c1_clock_swap: the {name} base point is off-regime ({:?}) -- a kink, \
                 not a gain.", gg.off_regime);
        let (_, c1, _) = invariants(&gg, (tau_g, tau_q, tau_s));
        built.push(C1SwapArm { taus: (tau_g, tau_q, tau_s), s: p.s, c1_marched: c1,
                               pair_rc: gg.pair_rc, pair_rv: gg.pair_rv, gains: gg });
    }
    let fast_stator = built.pop().expect("two arms were pushed");
    let fast_valve = built.pop().expect("two arms were pushed");

    // ONE plant, BOTH clock assignments — the pure discrimination.
    let base = &fast_valve.gains;
    let c1_shipped = |gg: &TripleGains, tau_q: f64, tau_s: f64| -> f64 {
        invariants(gg, (tau_g, tau_q, tau_s)).1
    };
    let held = {
        let (a, b) = (c1_shipped(base, fast, slow), c1_shipped(base, slow, fast));
        C1Pair { c1_fast_valve: a, c1_fast_stator: b, ratio: b / a }
    };
    // WHAT A ONE-SCALAR PLANT WOULD HAVE GIVEN, built from THIS plant's own gains.
    let k = 0.5 * (base.pair_rc + base.pair_rv);
    let mut null_gg = base.clone();
    null_gg.r_q = k / base.c_g;
    null_gg.r_v = k / base.v_g;
    let one = {
        let (a, b) = (c1_shipped(&null_gg, fast, slow), c1_shipped(&null_gg, slow, fast));
        C1Pair { c1_fast_valve: a, c1_fast_stator: b, ratio: b / a }
    };
    C1ClockSwap {
        marched_ratio: fast_stator.c1_marched / fast_valve.c1_marched,
        // delta = (w - u)(1/fast - 1/slow)/tau_g, and `w - u = pair_RC - pair_RV`
        predicted_delta: (base.pair_rc - base.pair_rv)
            * (1.0 / (tau_g * fast) - 1.0 / (tau_g * slow)),
        measured_delta: held.c1_fast_stator - held.c1_fast_valve,
        null_delta: one.c1_fast_stator - one.c1_fast_valve,
        held_gains: held,
        one_scalar_null: one,
        k_null: k,
        fast_valve,
        fast_stator,
    }
}

// ---------------------------------------------------------------------------------------------
// § 3 — THE FLOOR: an INFIMUM on a ray, not a minimum on a hyperplane
// ---------------------------------------------------------------------------------------------

/// The live half of a [`SplitFloorRow`] — present only when the mid-trajectory point was interior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitFloorLive {
    pub s: f64,
    pub pair_rc: f64,
    pub pair_rv: f64,
    /// `1 - pair_RC` and `1 - pair_RV` — the two SPLIT coefficients the clocks weight.
    pub u: f64,
    pub w: f64,
    /// WHICH loop the equality set silences — **MEASURED, not assumed**, because which one it is
    /// is a property of the plant.
    pub silenced: &'static str,
    pub quiet_share: f64,
    pub a_over_loud: f64,
    pub det2: f64,
    pub zeta_pred: f64,
    pub zeta: Option<f64>,
    pub floor: f64,
    pub modulus: f64,
    pub mod_pred: f64,
    pub rate_sum: f64,
    pub complex_pair: bool,
}

/// One grid point of [`split_floor`] — Python appends THREE different dicts here (no riding
/// window / off-regime / live), so the two dead shapes are `live: None` with the count and the
/// off-regime list beside them.
#[derive(Clone, Debug)]
pub struct SplitFloorRow {
    pub taus: (f64, f64, f64),
    pub n: usize,
    pub off_regime: Vec<&'static str>,
    pub live: Option<SplitFloorLive>,
}

/// RUNG 70 § 3's return.
#[derive(Clone, Debug)]
pub struct SplitFloor {
    pub rows: Vec<SplitFloorRow>,
    pub holds: bool,
    pub strict: bool,
    pub any_complex: bool,
    pub floor_range: (Option<f64>, Option<f64>),
    /// Python's `min(live, key=…)`, which keeps the FIRST minimum — so the comparison is STRICT.
    pub tightest: Option<SplitFloorLive>,
    pub worst_pred_err: Option<f64>,
    /// The RK4 guard, MEASURED against the plant rather than trusted.
    pub max_ds_lambda: f64,
    pub max_mod_ratio: Option<f64>,
}

/// RUNG 70 § 3 — **`zeta >= 1/sqrt(1 - min(pair_RC, pair_RV))` over EVERY bandwidth, approached
/// ONLY on a RAY — and WHICH ray is MEASURED, not assumed.**
///
/// The equality set silences whichever of the two SHARED loops carries the SMALLER coefficient
/// `1 - pair`, i.e. the one whose split pair is closer to `+1`. On this plant `pair_RC ~ -0.02`
/// and `pair_RV ~ +0.12`, so `w < u` and the ray silences the STATOR — the very loop that made
/// this `n = 3`. The grid therefore straddles BOTH extremes (a slow valve and a slow stator)
/// rather than assuming the direction, and `silenced` is a per-row reading rather than a constant.
///
/// The gains do not depend on the clocks — `R`, `C` and `V` are control LAWS and the clocks enter
/// only through `D = diag(1/tau_i)` — so each grid point measures its gains on ITS OWN march and
/// reports both the closed form and the shipped cubic's own roots, rather than pretending each arm
/// is an independent measurement of the plant.
///
/// **It is a SWEEP, not a limit**: a silenced clock is `1/tau = 0`, which is not a plant.
#[allow(clippy::too_many_arguments)]
pub fn split_floor(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, grid: &[(f64, f64, f64)], r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> SplitFloor {
    let mut rows: Vec<SplitFloorRow> = Vec::new();
    for &(tau_q, tau_g, tau_s) in grid {
        let m = split_rig(core, sm, tau_q, tau_s, v_max, tt4_max, true, true);
        let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
        let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
        let (traj, _) = m.stator_march_scoped(
            flight, &ramp, None, &leg,
            &MarchScope { tau_gov: Some(tau_g), ..MarchScope::DEFAULT });
        let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
        let pts = riding(&traj, b_max);
        let taus = (tau_g, tau_q, tau_s);
        if pts.is_empty() {
            rows.push(SplitFloorRow { taus, n: 0, off_regime: Vec::new(), live: None });
            continue;
        }
        let p = &pts[pts.len() / 2];
        let gg = triple_gains_at(&m, flight, p, None, None, 1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-70's floor march does not abort");
        if !gg.interior {
            rows.push(SplitFloorRow { taus, n: pts.len(), off_regime: gg.off_regime, live: None });
            continue;
        }
        let (aa, bb, cc) = (1.0 / tau_g, 1.0 / tau_q, 1.0 / tau_s);
        let (u, w) = (1.0 - gg.pair_rc, 1.0 - gg.pair_rv);
        let det2 = aa * (u * bb + w * cc);
        let (c2, c1, c0) = invariants(&gg, taus);
        let roots = cubic_roots_c(c2, c1, c0);
        let dom = sorted_by_abs(roots)[2];
        // THE RAY's own coordinate: the share of the shared pair's rate carried by the loop the
        // equality set SILENCES — the one with the smaller `1 - pair`.
        let quiet = if u < w { bb } else { cc };
        rows.push(SplitFloorRow {
            taus, n: pts.len(), off_regime: Vec::new(),
            live: Some(SplitFloorLive {
                s: p.s, pair_rc: gg.pair_rc, pair_rv: gg.pair_rv, u, w,
                silenced: if u < w { "valve" } else { "stator" },
                quiet_share: quiet / (aa + bb + cc),
                a_over_loud: aa / (if u < w { cc } else { bb }),
                det2,
                zeta_pred: (aa + bb + cc) / (2.0 * powp(det2, 0.5)),
                zeta: zeta_pair(roots),
                floor: powp(1.0 - gg.pair_rc.min(gg.pair_rv), -0.5),
                modulus: dom.abs(),
                mod_pred: powp(det2, 0.5),
                rate_sum: aa + bb + cc,
                complex_pair: dom.im.abs() > 1e-6 * dom.abs(),
            }),
        });
    }
    let live: Vec<SplitFloorLive> = rows.iter().filter_map(|x| x.live).collect();
    // Python compares `x["zeta"] >= x["floor"]` directly, so a `None` zeta is a `TypeError` there
    // rather than a skipped row; `_zeta_pair` returned `None` on 0 of 96 shipped calls (§ 5.27
    // (iv)). The panic is that same refusal, said out loud.
    let z = |x: &SplitFloorLive| -> f64 {
        x.zeta.expect("rung-70 split_floor: a live row with zeta = None. Python raises TypeError \
                       comparing None to a float here; _zeta_pair is measured to return None on 0 \
                       of 96 shipped calls, so this is a plant the grid has never reached.")
    };
    let mut tightest: Option<SplitFloorLive> = None;
    for x in &live {
        // Python's `min(..., key=…)` keeps the FIRST minimum, so the comparison is STRICT.
        if tightest.is_none_or(|b| z(x) / x.floor < z(&b) / b.floor) {
            tightest = Some(*x);
        }
    }
    SplitFloor {
        holds: live.iter().all(|x| z(x) >= x.floor - 1e-9),
        strict: live.iter().all(|x| z(x) > x.floor + 1e-12),
        any_complex: live.iter().any(|x| x.complex_pair),
        floor_range: (opt_fold(live.iter().map(|x| x.floor), f64::min),
                      opt_fold(live.iter().map(|x| x.floor), f64::max)),
        worst_pred_err: opt_fold(live.iter().map(|x| (z(x) / x.zeta_pred - 1.0).abs()), f64::max),
        max_ds_lambda: ds * opt_fold(live.iter().map(|x| x.modulus), f64::max).unwrap_or(0.0),
        max_mod_ratio: opt_fold(live.iter().map(|x| x.modulus / x.rate_sum), f64::max),
        tightest,
        rows,
    }
}

// ---------------------------------------------------------------------------------------------
// § 4 — THE WINDOWS, AND THE LEDGER
// ---------------------------------------------------------------------------------------------

/// Python's `span(sel)` return — `(s_lo, s_hi, count)`, with both bounds `None` on an empty
/// selection.
pub type Span = (Option<f64>, Option<f64>, usize);

/// RUNG 70 § 4's window return.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowOverlap {
    pub gov: Span,
    pub valve: Span,
    pub stator: Span,
    pub joint: Span,
    pub n: usize,
    pub overlaps: bool,
    pub joint_fraction: f64,
}

/// The four fields a rung-70 trajectory point carries that the window spans read.
///
/// **THE ARMS ARE SPELLED OUT AND THERE IS NO WILDCARD**, so the NEXT [`PointExtra`] variant breaks
/// the build here and gets the same question asked of it — rung 65's `valve_of` is the precedent,
/// and its wildcard is what slice Z's audit had to unpick by hand.
///
/// Python reads `p["required"]` and `p["b_cmd"]` (a `KeyError` off a five-state trajectory) but
/// `p.get("v_regime")` (None-safe). The asymmetry cannot be reached: `gov` spans the WHOLE
/// trajectory before `sta` does, so any non-`Triple` point raises there first.
fn triple_window_extra(p: &FuelPoint) -> (f64, f64, Regime) {
    match p.extra {
        PointExtra::Triple { required, b_cmd, v_regime, .. } => (required, b_cmd, v_regime),
        PointExtra::None
        | PointExtra::Asym { .. }
        | PointExtra::Valve { .. }
        | PointExtra::Cascade { .. }
        | PointExtra::CrossCascade { .. } =>
            panic!("rung-70's windows need a five-state trajectory: Python raises KeyError on \
                    `required` for every other route."),
    }
}

/// RUNG 70 § 4 — **DO ALL THREE WINDOWS OVERLAP? A GATE, not a remark.**
///
/// Rung 67 had to pick `Tt4_max` so that the governor's window overlapped the valve's AT ALL
/// ('post-ramp by construction' holds only at rung 46/47's own redline). Rung 70 adds a THIRD
/// window and inherits rung 67's number verbatim, so the overlap is no longer something rung 67
/// established for this plant — it has to be re-measured before any ledger cell or gain table is
/// quotable, because a table over an empty intersection would report the pairwise algebra of loops
/// that were never simultaneously live.
#[allow(clippy::too_many_arguments)]
pub fn window_overlap(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, r: f64, s_settle: f64, ds: f64, tau: f64, tau_gov: f64, tau_s: f64, v_max: f64,
) -> WindowOverlap {
    let m = split_rig(core, sm, tau, tau_s, v_max, tt4_max, true, true);
    let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(tt4_max) };
    let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
    let (traj, _) = m.stator_march_scoped(
        flight, &ramp, None, &leg, &MarchScope { tau_gov: Some(tau_gov), ..MarchScope::DEFAULT });
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let span = |sel: &dyn Fn(&FuelPoint) -> bool| -> Span {
        let w: Vec<f64> = traj.iter().filter(|p| sel(p)).map(|p| p.s).collect();
        if w.is_empty() {
            (None, None, 0)
        } else {
            (opt_fold(w.iter().copied(), f64::min), opt_fold(w.iter().copied(), f64::max), w.len())
        }
    };
    let gov = span(&|p| triple_window_extra(p).0 > 0.0);
    let valve = span(&|p| {
        let (_, cmd, _) = triple_window_extra(p);
        0.0 < cmd && cmd < b_max
    });
    let stator = span(&|p| triple_window_extra(p).2 == Regime::Riding);
    let joint = span(&|p| {
        let (req, cmd, reg) = triple_window_extra(p);
        req > 0.0 && 0.0 < cmd && cmd < b_max && reg == Regime::Riding
    });
    WindowOverlap {
        gov, valve, stator, joint,
        n: traj.len(),
        overlaps: joint.2 > 0,
        joint_fraction: if traj.is_empty() { 0.0 } else { joint.2 as f64 / traj.len() as f64 },
    }
}

/// One cell of [`split_bill`]'s 8-cell ledger, in BOTH currencies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitBillCell {
    /// Rung 66's `phi` violation integral, inherited unchanged.
    pub i: f64,
    /// Rung 67's `Tt4` exceedance integral, inherited unchanged.
    pub e: f64,
    pub min_phi: f64,
    pub max_tt4: f64,
    pub n: usize,
    pub credit_phi: Option<f64>,
    pub credit_tt4: Option<f64>,
}

/// The marginal contribution of each of the three loops, in one currency.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitMarginal {
    pub gov: f64,
    pub valve: f64,
    pub stator: f64,
}

/// RUNG 70 § 4's ledger return.
#[derive(Clone, Debug)]
pub struct SplitBill {
    /// The eight cells IN PYTHON's ORDER — a `Vec` and not a map, because the order is what makes
    /// a dump reproducible.
    pub cells: Vec<(&'static str, SplitBillCell)>,
    pub tt4_max: f64,
    // Python's ninth return key, `phi_lim_source="from_margin(sm)"`, is a CONSTANT STRING and is
    // deliberately absent: it carries no measurement, cannot differ between two runs, and rung
    // 68's `TripleBill` keeps `phi_lim`/`m_lim` as FLOATS rather than a label for the same reason.
    // Named here so step 6's oracle records a decision rather than an omission.
    pub marginal_phi: SplitMarginal,
    pub marginal_tt4: SplitMarginal,
    pub delivered_phi: Option<f64>,
    pub delivered_tt4: Option<f64>,
}

impl SplitBill {
    /// One named cell — a PANIC on an unknown name, because Python raises `KeyError`.
    pub fn cell(&self, name: &str) -> &SplitBillCell {
        &self.cells.iter().find(|(k, _)| *k == name)
             .unwrap_or_else(|| panic!("rung-70's ledger has no cell {name:?}")).1
    }
}

/// RUNG 70 § 4 — **THE 8-CELL LEDGER IN TWO CURRENCIES**: every subset of the three loops, every
/// loop lagged, and the SAME rig for every cell (rung 63's lesson).
///
/// TWO currencies because the loops watch two variables and a one-currency ledger would score the
/// governor in the valve's coin. `I` is rung 66's `phi` violation integral and `E` is rung 67's
/// `Tt4` exceedance integral, both INHERITED UNCHANGED so this table is differenceable against
/// rungs 66/67/68 rather than merely similar.
///
/// **THE ASYMMETRY IS THE POINT**, and it is rung 67's cross-credit with a THIRD loop: the airflow
/// loops DEBIT the temperature (bleed and closed stators both make it hotter at fixed fuel) while
/// the governor CREDITS the surge margin (clipping fuel raises `phi`). Rung 68's three loops shared
/// ONE currency and could only erode each other.
///
/// # THE GOVERNOR IS DISARMED AT THE MARCH, NEVER AT THE RIG
///
/// [`split_rig`] sets the set point unconditionally; the `bare`/`V`/`S`/`VS` cells pass
/// `tt4_max: None` and `tau_gov: None` to the march instead. That is Python's own arrangement and
/// it is what keeps ONE constructor for eight cells.
#[allow(clippy::too_many_arguments)]
pub fn split_bill(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, r: f64, s_settle: f64, ds: f64, tau: f64, tau_gov: f64, tau_s: f64, v_max: f64,
) -> SplitBill {
    let mut cells: Vec<(&'static str, SplitBillCell)> = Vec::new();
    for (name, valve, stator, gov) in [("bare", false, false, false),
                                       ("G", false, false, true),
                                       ("V", true, false, false),
                                       ("S", false, true, false),
                                       ("GV", true, false, true),
                                       ("GS", false, true, true),
                                       ("VS", true, true, false),
                                       ("GVS", true, true, true)] {
        let m = split_rig(core, sm, tau, tau_s, v_max, tt4_max, valve, stator);
        let leg = StatorLeg { accel: None, surge: None,
                              tt4_max: if gov { Some(tt4_max) } else { None } };
        let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
        let (traj, _) = m.stator_march_scoped(
            flight, &ramp, None, &leg,
            &MarchScope { tau_gov: if gov { Some(tau_gov) } else { None },
                          ..MarchScope::DEFAULT });
        let phi_lim = match m.fuel.inner.lever.lim {
            Some(bl) => bl.phi_lim,
            None => StatorLimiter::from_margin(&core.arming().map_lp_design, v_max, sm, None)
                        .phi_lim,
        };
        cells.push((name, SplitBillCell {
            i: violation(&traj, phi_lim, r),
            e: exceed(&traj, tt4_max, r),
            min_phi: opt_fold(traj.iter().map(|p| p.phi_lp), f64::min)
                         .expect("rung-70's ledger marches at least one point"),
            max_tt4: opt_fold(traj.iter().map(|p| p.tt4), f64::max)
                         .expect("rung-70's ledger marches at least one point"),
            n: traj.len(),
            credit_phi: None,
            credit_tt4: None,
        }));
    }
    let base = cells[0].1;
    for (_, c) in cells.iter_mut() {
        c.credit_phi = if base.i > 0.0 { Some(1.0 - c.i / base.i) } else { None };
        c.credit_tt4 = if base.e > 0.0 { Some(1.0 - c.e / base.e) } else { None };
    }
    let at = |name: &str| -> SplitBillCell {
        cells.iter().find(|(k, _)| *k == name)
             .unwrap_or_else(|| panic!("rung-70's ledger has no cell {name:?}")).1
    };
    let (gvs, vs, gs, gv) = (at("GVS"), at("VS"), at("GS"), at("GV"));
    SplitBill {
        tt4_max,
        // THE MARGINAL contribution of each loop to the FULL triple — the only reading that
        // survives rung 58's *check the SUM, not the term*.
        marginal_phi: SplitMarginal { gov: vs.i - gvs.i, valve: gs.i - gvs.i,
                                      stator: gv.i - gvs.i },
        marginal_tt4: SplitMarginal { gov: vs.e - gvs.e, valve: gs.e - gvs.e,
                                      stator: gv.e - gvs.e },
        delivered_phi: gvs.credit_phi,
        delivered_tt4: gvs.credit_tt4,
        cells,
    }
}

// ---------------------------------------------------------------------------------------------
// COUNTERS — the reduce arms and the set point's TWO SPELLINGS are invisible to every value key
// ---------------------------------------------------------------------------------------------
//
// Three things this rung does cannot be reached from a float a reader can print:
//
// * **THE REDUCE.** `integrate_fuel` hands back to rung 69 on `tau_gov is None or not
//   lagged_stator()`, and a reduce arm then emits rung 68/69's numbers BY CONSTRUCTION. That is
//   the contract, so agreement proves nothing about WHICH body ran.
// * **`_triple_laws`' OWN REDUCE.** `gov_max is None` returns the parent's laws — and § 5.27 (ii)
//   measured that the wrong body here empties the SAMPLE rather than moving a value, so a
//   value-diff gate compares two empty tables and passes.
// * **THE TWO SPELLINGS OF THE SET POINT.** `split_rig`'s bare permanent assignment and
//   `GovScope`'s save/restore write the SAME field, and the restore policy is only observable
//   where the receiver's governor was already armed.
//
// These are what `slice_ac_dispatch.rs` reads at step 7.

thread_local! {
    static INTEGRATE70_REDUCED: Cell<u64> = const { Cell::new(0) };
    static TRIPLE_LAWS70_PARENT: Cell<u64> = const { Cell::new(0) };
    static TRIPLE_LAWS70_GOV: Cell<u64> = const { Cell::new(0) };
    static SPLIT_RIG_CALLS: Cell<u64> = const { Cell::new(0) };
    static GOV_SCOPE_SETS: Cell<u64> = const { Cell::new(0) };
    static GOV_SCOPE_RESTORED_VALUE: Cell<u64> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

/// What the counters above hold.
///
/// `gov_scope_restored_value` is the one that carries a CLAIM rather than a count: slice AB
/// measured 29 of 29 `_with_ref` restores putting `None` back, concluded restore-previous and
/// restore-`None` agreed on every shipped path, and could only tell them apart with a manufactured
/// nest. **`_with_gov` is the MIRROR** — it is entered to turn the governor OFF, and all three of
/// its call sites in the whole ladder pass a literal `None` — so on a rung-70/71 rig every restore
/// puts a VALUE back and this counter is non-zero by an ORDINARY value witness.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census70 {
    pub integrate_reduced: u64,
    pub triple_laws_parent: u64,
    pub triple_laws_gov: u64,
    pub split_rig_calls: u64,
    pub gov_scope_sets: u64,
    pub gov_scope_restored_value: u64,
}

impl Census70 {
    pub fn read() -> Self {
        Census70 {
            integrate_reduced: INTEGRATE70_REDUCED.with(Cell::get),
            triple_laws_parent: TRIPLE_LAWS70_PARENT.with(Cell::get),
            triple_laws_gov: TRIPLE_LAWS70_GOV.with(Cell::get),
            split_rig_calls: SPLIT_RIG_CALLS.with(Cell::get),
            gov_scope_sets: GOV_SCOPE_SETS.with(Cell::get),
            gov_scope_restored_value: GOV_SCOPE_RESTORED_VALUE.with(Cell::get),
        }
    }

    pub fn reset() {
        INTEGRATE70_REDUCED.with(|x| x.set(0));
        TRIPLE_LAWS70_PARENT.with(|x| x.set(0));
        TRIPLE_LAWS70_GOV.with(|x| x.set(0));
        SPLIT_RIG_CALLS.with(|x| x.set(0));
        GOV_SCOPE_SETS.with(|x| x.set(0));
        GOV_SCOPE_RESTORED_VALUE.with(|x| x.set(0));
    }
}
