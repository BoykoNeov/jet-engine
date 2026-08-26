//! RUNG 64 — the `phi`-REFERENCED BLEED LIMITER: the first CLOSED LOOP on an airflow lever.
//!
//! Slice X. Python's `BleedLimiter` + `LimitedBleedTransient` (`turbojet/engine.py`, 505 lines),
//! ported onto [`crate::bleed_transient`]'s shape.
//!
//! **HEADLINE (the rung's, not the port's): a limiter's LAW cannot buy PROTECTION, only its
//! PRICE.** The ceiling on the protected coordinate is `min phi` over the FULLY-OPEN march, which
//! is a property of `b_max` — the lever's AUTHORITY, i.e. hardware — and `b = b_max` is itself an
//! OPEN-LOOP law. What feedback buys is the BILL.
//!
//! # What slice X adds to the table
//!
//! **ONE cell** — [`b_at_point`](crate::bleed_transient::LeverHooks::b_at_point), whose two ladder
//! call sites are rung 64's own `_bill_cell` and rung 65's override — and **eight swaps**. § 5.22
//! (iii); it is the first row of § 5.19 (x)'s cell column an emitter confirms.
//!
//! # The two dead branches, with their counts — measured, not read
//!
//! § 5.22 (vi), on one floored march at `ds = 0.02`:
//!
//! * `b_of`'s **fall-through to rung 62** is taken **0 of 1 705 times WITHIN A MARCH** on an
//!   ARMED machine — every `b_of` a closure reaches is reached from inside a live solve, so the
//!   forced carrier answers first. It is live only where rung 64's cell and rung 62's are
//!   behaviourally identical, which is [[rust-port-slice-u-step3]]'s *a function exercised only
//!   on cells chosen for INERTNESS*.
//!
//!   **THE SCOPE IS LOAD-BEARING AND WAS NEARLY LEFT OFF.** A reader may call `b_of` DIRECTLY,
//!   outside any solve, and then the fall-through is the ONLY branch — `slice_x_smoke.rs`
//!   section B does exactly that four times. So a step-5 gate asserting `b_of_super == 0` must
//!   [`Census64::reset`] immediately before the march it measures, or it fails on a binary that
//!   merely read the valve. `b_of_state` carries no such caveat: it is zero at rung 64
//!   EVERYWHERE, which makes it the stronger of the two, and they are asserted separately for
//!   that reason.
//! * `b_of`'s **`b_state` override** is taken **0 of 1 705 times** at rung 64 ENTIRELY — it is
//!   rung 65's lagged valve, declared at 64. A port that drops it passes every slice-X gate and
//!   breaks at slice Y.
//!
//! Both are gated by MANUFACTURED bugs at step 5, never by a value key, because no value key can
//! reach them.

use crate::map::ComponentMap;

// ---------------------------------------------------------------------------------------------
// THE DEVICE
// ---------------------------------------------------------------------------------------------

/// RUNG 64's control law: **the smallest valve position in `[0, b_max]` that holds
/// `phi_lp >= phi_lim`.**
///
/// Every arming of this valve from rung 42 to 63 was OPEN LOOP — a constant position (42) or a
/// schedule `b(n_L)` read off the state (62). This one watches the **protected variable**, so it
/// is to rung 62 exactly what rung 49's `SurgeLimiter` is to rung 48's feedforward schedule.
///
/// **THE THREE CLAMPS ARE THE THREE REGIMES, and they are the rung** — see [`Regime`].
///
/// **WATCHES THE LP AND ONLY THE LP**, disclosed rather than parameterised: rung 42 established
/// the valve is a degree of freedom on the LP spool and not the HP, and the outer solve needs
/// `phi` MONOTONE in `b`, which the choked-`A4` argument gives for the LP face flow (it carries
/// `1/(1-b)`) and does not give for the HP.
///
/// `Copy` because [`LeverArming`](crate::bleed_transient::LeverArming) is, and that is what keeps
/// § 5.21 (iii)'s "the signature is never re-opened" true when this field is added to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BleedLimiter {
    /// The floor, in the map's own flow-coefficient units.
    pub phi_lim: f64,
    /// The valve's AUTHORITY — hardware, not a control setting.
    pub b_max: f64,
    /// RUNG 65: the valve's BANDWIDTH — hardware too. `None` IS rung 64's instantaneous valve,
    /// **not** `Some(0.0)`; Python's own assert says so and rung 65's finding is that the
    /// difference does not vanish as `tau -> 0`.
    pub tau: Option<f64>,
}

impl BleedLimiter {
    /// Python's `__init__` + `__post_init__` — all three asserts, in Python's order.
    pub fn new(phi_lim: f64, b_max: f64) -> Self {
        Self::with_tau(phi_lim, b_max, None)
    }

    /// The full constructor. **`b_max = 0` is REFUSED, not silently reduced**: a limiter that
    /// cannot act is a DIFFERENT object from an absent one (that is `bleed_lim = None`), and the
    /// distinction is the whole rung — the ceiling belongs to `b_max`.
    pub fn with_tau(phi_lim: f64, b_max: f64, tau: Option<f64>) -> Self {
        assert!(phi_lim > 0.0, "rung-64 phi floor is a flow coefficient");
        assert!(b_max > 0.0 && b_max < 0.5,
                "rung-64 needs a valve with AUTHORITY: b_max = 0 is a limiter that cannot act, \
                 which is a DIFFERENT object from an absent one (that is `bleed_lim=None`), and \
                 b >= 0.5 is rung 42's own starved-core bound; got b_max = {b_max}");
        assert!(tau.is_none_or(|t| t > 0.0),
                "rung-65 tau is a time constant on the march coordinate; the INSTANTANEOUS valve \
                 is rung 64 (tau=None), not tau=0. The two are different objects and rung 65's \
                 finding is that the difference does not vanish as tau -> 0; got tau = {tau:?}");
        BleedLimiter { phi_lim, b_max, tau }
    }

    /// `phi_lim = (1+sm) * phi_surge` off the map's OWN imposed surge line — rung 49's
    /// `from_margin`, so the two floors are set in identical units and rung 63 § 3's band edges
    /// are directly comparable set points.
    pub fn from_margin(cmap: &ComponentMap, b_max: f64, sm: f64) -> Self {
        Self::from_margin_tau(cmap, b_max, sm, None)
    }

    /// [`from_margin`](Self::from_margin) with rung 65's bandwidth.
    pub fn from_margin_tau(cmap: &ComponentMap, b_max: f64, sm: f64, tau: Option<f64>) -> Self {
        assert!(cmap.phi_surge > 0.0,
                "rung-64 from_margin needs a surge line: build the map with .with_phi_surge(.)");
        assert!(sm >= 0.0, "the rung-64 floor sits AT or ABOVE the surge line");
        Self::with_tau((1.0 + sm) * cmap.phi_surge, b_max, tau)
    }

    /// RUNG 65. The SAME control law on a valve with finite bandwidth — the only difference
    /// between rung 64's object and rung 65's, so every comparison between them holds `phi_lim`
    /// and `b_max` fixed by construction.
    pub fn lagged(&self, tau: f64) -> Self {
        Self::with_tau(self.phi_lim, self.b_max, Some(tau))
    }
}

/// Which clamp `_solve_b` landed on — **reported, never inferred by a reader comparing floats**,
/// which is Python's own sentence.
///
/// The distribution is not uniform and a gate that runs one machine tests two branches of three
/// (§ 5.22 (vi)/P3): on the rung's headline machine (`phi_lim = 0.80`, reachable) the split is
/// **257 dormant / 135 riding / 0 saturated**; only `authority_ceiling`'s deliberately over-set
/// floor saturates, at **167 / 74 / 151**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regime {
    /// `b = 0` — `phi` already clears the floor, and the closure dispatches to rung 63's parent
    /// **bit-for-bit**, not to a `0.0` position.
    Dormant,
    /// `0 < b < b_max` — rung 60's tautology pins `min phi_lp == phi_lim` EXACTLY.
    Riding,
    /// `b = b_max` — the floor is VIOLATED. The first law in this family that cannot deliver its
    /// own set point, and the regime that proves the CEILING belongs to `b_max` and not to the
    /// law.
    Saturated,
}

use crate::bleed_transient::{
    r62_armed_bleed, r62_b_of, r62_try_close, r62_try_close_fuel, BleedSchedule, LeverArm,
    LeverArming, LeverHooks, R62, R62_FUEL, R62_STATOR, R62_TWO,
};
use crate::fuel_transient::{
    Floor, FuelCloseState, FuelPoint, FuelTransientCore, FuelTransientHooks, SurgeLimiter,
};
use crate::gas::Abort;
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::stator_transient::{
    CellRead, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg,
    StatorTransientHooks,
};
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{
    CloseState, ForcedBleed, TwoSpoolTransientCore, TwoSpoolTransientHooks,
};
use crate::engine::FlightCondition;
use crate::two_spool::Spool;
use std::cell::Cell;

// ---------------------------------------------------------------------------------------------
// COUNTERS — the reduce and the regimes are BOTH invisible to every value key
// ---------------------------------------------------------------------------------------------
//
// § 5.21 (v): `bleed_lim = None` returns the PARENT'S FUNCTION OBJECT, not "the parent body with
// b = 0", so no float a reader can print distinguishes the two. And `_solve_b` returns the regime
// as its third element, which Python's own docstring says is "reported, never inferred by a
// reader comparing floats" — and then NOTHING in the ladder reads it. Both are counted here so
// step 5's gates have something to assert on.

thread_local! {
    /// `bleed_lim` absent, so the cell handed straight back to rung 62's body. **THE REDUCE.**
    static CLOSE_UNFLOORED: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_UNFLOORED: Cell<u64> = const { Cell::new(0) };
    static SOLVE_B_CALLS: Cell<u64> = const { Cell::new(0) };
    static REGIME_DORMANT: Cell<u64> = const { Cell::new(0) };
    static REGIME_RIDING: Cell<u64> = const { Cell::new(0) };
    static REGIME_SATURATED: Cell<u64> = const { Cell::new(0) };
    /// `b_of` read the FORCED trial position — the outer solve is mid-flight.
    static B_OF_FORCED: Cell<u64> = const { Cell::new(0) };
    /// **RUNG 65's, DEAD AT 64.** Gated at exactly zero; see the module note.
    static B_OF_STATE: Cell<u64> = const { Cell::new(0) };
    /// `b_of` fell through to rung 62 — **0 of 1 705 on any ARMED machine**, module note again.
    static B_OF_SUPER: Cell<u64> = const { Cell::new(0) };
    /// `b_at_point` with no floor, i.e. the `b_of` leg rather than the re-solve.
    static B_AT_POINT_UNFLOORED: Cell<u64> = const { Cell::new(0) };
    static B_AT_POINT_RESOLVED: Cell<u64> = const { Cell::new(0) };
    /// An [`Abort`] escaped `_solve_b`'s bracket. **PROBE 8 MEASURED 0 of 156 373 closure calls**
    /// over `tests/test_rung64.py`, so this is a gated-at-zero arm exactly as
    /// `Census::read_foreign_v_of` is — carried and counted rather than `expect`-ed away.
    static SOLVE_B_ABORTS: Cell<u64> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

/// Rung 64's dispatch and regime census.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census64 {
    pub close_unfloored: u64,
    pub close_fuel_unfloored: u64,
    pub solve_b_calls: u64,
    pub dormant: u64,
    pub riding: u64,
    pub saturated: u64,
    pub b_of_forced: u64,
    pub b_of_state: u64,
    pub b_of_super: u64,
    pub b_at_point_unfloored: u64,
    pub b_at_point_resolved: u64,
    pub solve_b_aborts: u64,
}

impl Census64 {
    pub fn take() -> Census64 {
        Census64 {
            close_unfloored: CLOSE_UNFLOORED.with(Cell::get),
            close_fuel_unfloored: CLOSE_FUEL_UNFLOORED.with(Cell::get),
            solve_b_calls: SOLVE_B_CALLS.with(Cell::get),
            dormant: REGIME_DORMANT.with(Cell::get),
            riding: REGIME_RIDING.with(Cell::get),
            saturated: REGIME_SATURATED.with(Cell::get),
            b_of_forced: B_OF_FORCED.with(Cell::get),
            b_of_state: B_OF_STATE.with(Cell::get),
            b_of_super: B_OF_SUPER.with(Cell::get),
            b_at_point_unfloored: B_AT_POINT_UNFLOORED.with(Cell::get),
            b_at_point_resolved: B_AT_POINT_RESOLVED.with(Cell::get),
            solve_b_aborts: SOLVE_B_ABORTS.with(Cell::get),
        }
    }

    pub fn reset() {
        for c in [&CLOSE_UNFLOORED, &CLOSE_FUEL_UNFLOORED, &SOLVE_B_CALLS, &REGIME_DORMANT,
                  &REGIME_RIDING, &REGIME_SATURATED, &B_OF_FORCED, &B_OF_STATE, &B_OF_SUPER,
                  &B_AT_POINT_UNFLOORED, &B_AT_POINT_RESOLVED, &SOLVE_B_ABORTS] {
            c.with(|x| x.set(0));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// THE OUTER SOLVE
// ---------------------------------------------------------------------------------------------

/// What `_solve_b` needs of a closure's return — the ONE field it reads.
pub trait HasPhiLp {
    fn phi_lp(&self) -> f64;
}

impl HasPhiLp for CloseState {
    fn phi_lp(&self) -> f64 { self.phi_lp }
}

impl HasPhiLp for FuelCloseState {
    fn phi_lp(&self) -> f64 { self.base.phi_lp }
}

/// **THE OUTER SOLVE**: the smallest `b` in `[0, b_max]` holding `phi_lp >= phi_lim`.
///
/// ONE scalar bracketed root, no nested Newton and no 2×2. `phi_lp` is monotone increasing in `b`
/// because the choked `A4` imposes the CORE flow and the FACE flow the closure must find to feed
/// it carries `1/(1-b)` (`_close_fuel`'s `m_imp`), so both clamps are decided by two evaluations
/// and the root by Illinois between them.
///
/// **THE REGIME IS RETURNED, NEVER INFERRED BY A READER COMPARING FLOATS** — Python's own
/// sentence. And then no ladder caller reads it, which is why [`Census64`] counts it.
///
/// **THE FALLIBLE ARMS ARE FAITHFUL, NOT DEFENSIVE.** Python's residual raises straight out of
/// `_illinois` and out of `_solve_b`; [`try_illinois`] propagates an [`Abort`] the same way, so no
/// stash and no `expect` is needed and there is no iteration-count divergence to disclose. Probe 8
/// measured **0 aborts in 156 373 closure calls** over `tests/test_rung64.py` — the arms are dead
/// on that grid and counted at zero rather than assumed away.
fn r64_solve_b<T, F>(lim: &BleedLimiter, closer: F) -> Result<(T, f64, Regime), Abort>
where
    T: HasPhiLp,
    F: Fn(f64) -> Result<T, Abort>,
{
    bump(&SOLVE_B_CALLS);
    let guard = |r: Result<T, Abort>| -> Result<T, Abort> {
        if r.is_err() { bump(&SOLVE_B_ABORTS); }
        r
    };
    let c0 = guard(closer(0.0))?;
    if c0.phi_lp() >= lim.phi_lim {
        bump(&REGIME_DORMANT);
        return Ok((c0, 0.0, Regime::Dormant));
    }
    let c1 = guard(closer(lim.b_max))?;
    if c1.phi_lp() <= lim.phi_lim {
        bump(&REGIME_SATURATED);
        return Ok((c1, lim.b_max, Regime::Saturated));
    }
    let f0 = c0.phi_lp() - lim.phi_lim;
    let f1 = c1.phi_lp() - lim.phi_lim;
    let b = match try_illinois(|b| closer(b).map(|c| c.phi_lp() - lim.phi_lim),
                               0.0, lim.b_max, f0, f1, 1e-13, ILLINOIS_MAXIT) {
        Ok(b) => b,
        Err(e) => { bump(&SOLVE_B_ABORTS); return Err(e); }
    };
    bump(&REGIME_RIDING);
    // Python re-evaluates at the root rather than returning `c1` — the returned closure IS the
    // one the march consumes, so a reconstruction here would be § 5.22 (ii) a second time.
    Ok((guard(closer(b))?, b, Regime::Riding))
}

// ---------------------------------------------------------------------------------------------
// THE CELLS
// ---------------------------------------------------------------------------------------------

/// RUNG 64's `_armed_bleed` — rung 62's, plus the floor.
fn r64_armed_bleed(t: &TwoSpoolTransientCore) -> bool {
    r62_armed_bleed(t) || t.lever.lim.is_some()
}

/// RUNG 64's `b_of` — rung 62's state function with **one** addition that is live here and one
/// that is not.
///
/// While the outer solve is trialling a position, `b_forced` **IS** the valve. Nothing else may
/// set it and [`ForcedBleed`]'s destructor always clears it — a leaked trial position would make
/// the closure silently report a state the plant never visited.
///
/// `b_state` is RUNG 65's lagged position carried as a march state, and `b_forced` wins over it:
/// the command solve trials positions on a plant whose live state is the one being commanded away
/// from. **DEAD AT RUNG 64** (0 of 1 705), declared because dropping it passes every slice-X gate
/// and breaks at slice Y.
fn r64_b_of(t: &TwoSpoolTransientCore, nu_lp: f64, tt2: Option<f64>) -> f64 {
    if let Some(b) = t.b_forced.get() {
        bump(&B_OF_FORCED);
        return b;
    }
    if let Some(b) = t.b_state.get() {
        bump(&B_OF_STATE);
        return b;
    }
    bump(&B_OF_SUPER);
    r62_b_of(t, nu_lp, tt2)
}

/// RUNG 64's `at_lever` — rung 63's sibling constructor with the THIRD arming mode threaded
/// through.
///
/// **THE FOURTH INSTANCE OF ONE TRAP** (rung 61's `at_setting`, rung 62's `at_stator`, rung 63's
/// `_isolating`): a sibling constructor that silently drops the newest lever turns every inherited
/// reader into an armed-vs-armed comparison that measures nothing while returning a plausible
/// number.
fn r64_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 64's `at_stator` — rung 62's, plus `bleed_lim=self.bleed_lim`.
///
/// Rung 62's body sets the floor to `None` **deliberately** (it has no such name), so a rung-64
/// machine reaching it would silently lose its floor. This override is what closes that.
fn r64_at_stator(core: &ScheduledStatorCore, arm: StatorArm) -> ScheduledStatorCore {
    let lever = LeverArm {
        stator: arm,
        bleed: core.fuel.inner.lever.bleed,
        bleed_sched: core.fuel.inner.lever.sched,
        bleed_lim: core.fuel.inner.lever.lim,
    };
    core.at_lever(&lever)
}

/// RUNG 64's `_isolating` — rung 63's gate with the floor counted as an arming mode.
///
/// **THE ENTIRE CONTENT OF THIS OVERRIDE IS `want`.** The assert's other side,
/// `reference.armed_bleed()`, is DISPATCHED and already gains the floor at
/// [`r64_armed_bleed`]. So a rung-64 machine running rung 63's body would fire the assert on a
/// floored NEIGHBOUR — Python's *"a reader carrying the floor as a NEIGHBOUR would fail it for
/// the wrong reason"*. Extending [`LeverArm::arms_valve`] in place instead of adding
/// [`LeverArm::arms_valve_floored`] would make this body textually identical to rung 62's and the
/// override a no-op; see that method's note.
fn r64_isolating(core: &ScheduledStatorCore, lever: &LeverArm, neighbour: Option<&LeverArm>)
    -> (ScheduledStatorCore, ScheduledStatorCore) {
    let empty = LeverArm::default();
    let nb = neighbour.unwrap_or(&empty);
    let lk = lever.keys();
    assert!(!lk.is_empty(), "rung-64 isolates a lever: pass one `at_lever` keyword");
    let nk = nb.keys();
    for k in &lk {
        assert!(!nk.contains(k),
                "rung-64: '{k}' is the LEVER being isolated, so the reference sibling must not \
                 also carry it.");
    }
    let reference = core.at_lever(nb);
    let armed = core.at_lever(&LeverArm::merged(nb, lever));
    let want = nb.arms_valve_floored();
    assert!(reference.armed_bleed() == want,
            "rung-64's reference sibling must carry the NEIGHBOUR's valve and nothing else; it \
             reports armed={} against neighbour={want}.", reference.armed_bleed());
    (reference, armed)
}

/// RUNG 64's `_close` — **the closure, with the loop closed.**
fn r64_try_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    let Some(lim) = t.lever.lim else {
        bump(&CLOSE_UNFLOORED);
        return r62_try_close(t, nu_lp, nu_hp, tt4, tt2, pt2);
    };
    Ok(r64_solve_b(&lim, |b| {
        let _g = ForcedBleed::set(t, b);
        r62_try_close(t, nu_lp, nu_hp, tt4, tt2, pt2)
    })?.0)
}

/// RUNG 64's `_close_fuel` — the same, on the fuel-metered closure.
fn r64_try_close_fuel(
    ft: &FuelTransientCore, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
) -> Result<FuelCloseState, Abort> {
    let Some(lim) = ft.inner.lever.lim else {
        bump(&CLOSE_FUEL_UNFLOORED);
        return r62_try_close_fuel(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2);
    };
    Ok(r64_solve_b(&lim, |b| {
        let _g = ForcedBleed::set(&ft.inner, b);
        r62_try_close_fuel(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2)
    })?.0)
}

/// RUNG 64's `b_at_point` — **THE CELL SLICE X CREATES.**
///
/// **IT RE-SOLVES; IT DOES NOT RECONSTRUCT.** Python: *"the valve is a pure function of the state,
/// so this RE-SOLVES it exactly rather than reconstructing it — which is what makes the bleed
/// integral below a measurement and not an estimate."* Reconstructing via `b_of` instead drives a
/// floored march's `b_int` and `b_peak` to **exactly 0** and both published ratios to 0 with all
/// 111 rung-62/63/64 gates green (§ 5.22 (ii)) — because `b_of` off a march point reads no forced
/// position and no state, so it falls through to rung 62's *constant*, which on a floored machine
/// is `0.0`.
fn r64_b_at_point(core: &ScheduledStatorCore, flight: &FlightCondition, p: &FuelPoint) -> f64 {
    let (tt2, pt2, _) = core.fuel.inner.inlet(flight);
    let Some(lim) = core.fuel.inner.lever.lim else {
        bump(&B_AT_POINT_UNFLOORED);
        return core.fuel.inner.b_of(p.nu_lp, Some(tt2));
    };
    bump(&B_AT_POINT_RESOLVED);
    let ft = &core.fuel;
    r64_solve_b(&lim, |b| {
        let _g = ForcedBleed::set(&ft.inner, b);
        r62_try_close_fuel(ft, p.nu_lp, p.nu_hp, p.mf, tt2, pt2)
    })
    // Probe 8 measured 0 aborts in 156 373 closure calls over `tests/test_rung64.py`. Python
    // PROPAGATES here rather than raising; this panics, on a re-solve at a point the march has
    // ALREADY closed successfully, so reaching it would mean the state moved in between.
    .expect("rung-64 b_at_point re-solves at a point the march already closed: probe 8 measured 0 \
             aborts in 156 373 closure calls, and Python propagates rather than raising here")
    .1
}

// ---------------------------------------------------------------------------------------------
// THE TABLES
// ---------------------------------------------------------------------------------------------

/// RUNG 64's own table — **five of six cells swapped**, and `b_at_point` is the one that did not
/// exist below. Only `legs` is inherited.
pub const R64: LeverHooks = LeverHooks {
    at_lever: r64_at_lever,
    armed_bleed: r64_armed_bleed,
    b_of: r64_b_of,
    isolating: r64_isolating,
    b_at_point: r64_b_at_point,
    // NOT overridden — rung 77's is the next body. `..R62` would say the same thing; it is spelled
    // out because five of six being swapped makes a spread read as an oversight.
    legs: R62.legs,
};

/// RUNG 64's swap into rung 62's table — ONE cell, and **`..R62_TWO`, never `..R57_TWO`**: rung 64
/// does not override `_powers` or `_instant_tail`, so it must inherit rung 62's BLED bodies.
pub const R64_TWO: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: r64_try_close,
    ..R62_TWO
};

/// RUNG 64's swap into rung 62's fuel table — ONE cell.
pub const R64_FUEL: FuelTransientHooks = FuelTransientHooks {
    try_close_fuel: r64_try_close_fuel,
    ..R62_FUEL
};

/// RUNG 64's swap into rung 62's stator table — ONE cell, the `at_stator` that carries the floor.
pub const R64_STATOR: StatorTransientHooks = StatorTransientHooks {
    at_stator: r64_at_stator,
    ..R62_STATOR
};

// ---------------------------------------------------------------------------------------------
// THE CONSTRUCTOR
// ---------------------------------------------------------------------------------------------

/// RUNG 64's constructor — Python's `LimitedBleedTransient.__init__`.
///
/// **THE ASSERT ORDER IS PYTHON'S.** `super().__init__(…)` runs FIRST, so rung 57's four
/// capture-discipline asserts and rung 62's two fire before rung 64's two.
///
/// Rung 62's TWO-way arming assert is **EXTENDED to three, never replaced** — the three are the
/// legs the rung differences.
pub fn build_limited_bleed(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = ScheduledStatorTransient::with_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        &R64_TWO, &R64_STATOR, &R64_FUEL, &R64,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim });
    // Rung 62's own two, AFTER super()'s — reproduced rather than delegated to
    // `build_scheduled_bleed`, which would install rung 62's TABLES.
    assert!(!(arm.bleed != 0.0 && arm.bleed_sched.is_some()),
            "rung-62: the valve gets a CONSTANT position or a SCHEDULE, not both -- they are the \
             two legs the rung differences (rung 57's discipline).");
    assert!((0.0..0.5).contains(&arm.bleed),
            "rung-42 bleed fraction must be in [0, 0.5): b>=0.5 starves the core and the choked \
             branch is long gone by then");
    // Rung 64's own two.
    assert!(!(arm.bleed_lim.is_some() && (arm.bleed != 0.0 || arm.bleed_sched.is_some())),
            "rung-64: the valve gets a CONSTANT position (42), a SCHEDULE (62) or a FLOOR (64) -- \
             exactly one. They are the three legs this rung differences, and rung 62's two-way \
             assert is extended rather than replaced.");
    // Python spells this `... or self._LAG_OK`, and `_LAG_OK` is a CLASS ATTRIBUTE that
    // **RUNG 65 FLIPS IN ITS SUBCLASS**. Here it is hard-coded false, which is right for
    // every rung-64 machine and is a decision slice Y must UNDO rather than work around.
    assert!(arm.bleed_lim.is_none_or(|l| l.tau.is_none()),
            "rung-64's valve is INSTANTANEOUS: it is a pure function of the state, re-solved at \
             every sub-evaluation. A limiter carrying `tau` is rung 65's LAGGED valve, whose \
             position is a THIRD STATE and needs `LaggedBleedTransient` to march it. Silently \
             dropping the lag here would make every rung-64 reader report a bandwidth it never \
             had.\n\nTO THE SLICE-Y PORTER: this is Python's `_LAG_OK`, and RUNG 65 FLIPS IT. \
             `build_limited_bleed` hard-codes it false, so rung 65's constructor CANNOT delegate \
             here to inherit rungs 57/62/64's assert chain -- it would refuse its own lag. \
             Re-spell the chain in the rung-65 builder with this one relaxed; do NOT add a bool \
             parameter, which would put rung 65's state in rung 64's signature.");
    built
}

// ---------------------------------------------------------------------------------------------
// THE READING INSTRUMENTS
// ---------------------------------------------------------------------------------------------

/// One marched cell in **THE BILL's** currency — Python's `_bill_cell` return.
///
/// Deliberately NOT rung 57's `cell`: that one reports the two surge margins and the fuel a
/// min-select leg removed, and this rung's question is what the AIRFLOW cost, in the currency rung
/// 61 established is the real one (the overspeed and the thrust — **not** the bleed integral,
/// which rung 61 showed can move while 73–102 % of the overspeed survives).
#[derive(Clone, Debug)]
pub struct BillCell {
    // --- THE PLATEAU: diagnostics, NEVER results ---------------------------------------------
    /// **CHECK `plateau_pts == 1` BEFORE QUOTING THESE THREE.** A floor that RIDES pins `phi_lp`
    /// to `phi_lim` over an INTERVAL, so the minimum's VALUE is a result (rung 60) and its
    /// LOCATION is not one — the argmin is decided by which point happens to sit one ulp lower.
    /// Safe on any march with an isolated minimum: every OPEN-LOOP law and a SATURATED floor.
    /// **Every rung-44-to-52 reader that reports WHERE a minimum sits is bounded by this on a
    /// floored plant.**
    pub nu_at_min_lp: f64,
    pub s_at_min_lp: f64,
    pub b_at_min_lp: f64,
    pub plateau_span: f64,
    pub plateau_pts: usize,
    // --- the results --------------------------------------------------------------------------
    pub min_phi_lp: f64,
    pub min_phi_hp: f64,
    pub m_i_lp: f64,
    pub m_i_hp: f64,
    pub b_int: f64,
    pub b_peak: f64,
    pub b_end: f64,
    pub thrust_int: f64,
    pub thrust_end: f64,
    pub nu_lp_end: f64,
    pub nu_hp_end: f64,
    pub tt4_peak: f64,
    pub nu0_lp: f64,
    pub nu0_hp: f64,
    pub npts: usize,
    /// RUNG 65 needs the trajectory itself (the `tau -> 0` deviation is a per-point compare).
    /// Python ADDS the key rather than defaulting it to `None`, so an un-asking rung-62/63/64
    /// caller gets a dict with exactly the keys it always had.
    pub traj: Option<Vec<FuelPoint>>,
}

/// Python's `authority_ceiling` return — **RUNG 64, HALF ONE.**
#[derive(Clone, Debug)]
pub struct AuthorityCeiling {
    pub r: f64,
    pub ds: f64,
    pub b_max: f64,
    pub phi_surge: f64,
    pub shut: BillCell,
    pub schedule: BillCell,
    pub full: BillCell,
    pub over: BillCell,
    pub ceiling: f64,
    pub phi_lim_over: f64,
    pub gap_schedule: f64,
    /// The schedule is NOT saturated where it matters — it commands less than `b_max` at its OWN
    /// `phi` minimum, which is why a gap to the ceiling exists at all, and why that gap is about
    /// PLACEMENT and not about feedback.
    pub b_at_sched_min: f64,
    pub sched_saturated: bool,
    /// THE WITNESS: an over-set floor is VIOLATED, and by construction cannot beat the fully-open
    /// march.
    pub violated: bool,
    pub over_deficit: f64,
    pub bounded_by_full: bool,
    pub over_vs_full: f64,
}

/// One law's price, referenced to the valve-SHUT march — Python's `bill[k]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BillRow {
    pub d_nu_lp_end: f64,
    pub d_nu_hp_end: f64,
    pub d_thrust_end: f64,
    pub thrust_end_pct: f64,
    pub thrust_int_pct: f64,
    pub d_min_phi_hp: f64,
    pub b_int: f64,
    pub b_peak: f64,
}

/// Python's `matched_bill` return — **RUNG 64, HALF TWO, THE RUNG.**
#[derive(Clone, Debug)]
pub struct MatchedBill {
    pub r: f64,
    pub ds: f64,
    pub phi_target: f64,
    pub b_cap: f64,
    pub n_lo: f64,
    pub b_star: f64,
    pub bmax_star: f64,
    pub shut: BillCell,
    pub constant: BillCell,
    pub schedule: BillCell,
    pub floor: BillCell,
    pub bill_constant: BillRow,
    pub bill_schedule: BillRow,
    pub bill_floor: BillRow,
    pub matched: f64,
    pub saturated: bool,
    pub b_ratio_const: f64,
    pub b_ratio_sched: f64,
}

/// Python's `floor_refusal` return — **RUNG 64's closing leg.**
#[derive(Clone, Debug, PartialEq)]
pub struct FloorRefusal {
    pub sm: f64,
    pub d_sm: f64,
    pub phi_lim: f64,
    pub phi_lim_below: f64,
    pub r: f64,
    pub ds: f64,
    pub b_cap: f64,
    pub neither: CellRead,
    pub fuel: CellRead,
    pub valve: CellRead,
    pub both: CellRead,
    pub below_bare: CellRead,
    pub below_armed: CellRead,
    pub removed_alone: f64,
    /// **NOT A RESULT** — reported for the record only. At exact tangency `_surge_fuel` decides
    /// between its dormant return and a 60-iteration degenerate hunt on the SIGN OF ONE ULP, so
    /// its very existence is a roundoff coin flip.
    pub removed_together: f64,
    /// (i) THE CLAIM. To MACHINE PRECISION and deliberately not to the bit: the degenerate solve
    /// returns an arbitrary point of a continuum, so demanding bit-equality would be asserting on
    /// the same roundoff this reader exists to expose.
    pub inert: bool,
    pub credit: f64,
    /// (ii) THE CONTROL.
    pub control_dormant: bool,
    pub removed_below_bare: f64,
    pub removed_below_armed: f64,
}

impl ScheduledStatorCore {
    /// Python's `_bill_cell`.
    pub fn bill_cell(&self, flight: &FlightCondition, ramp: &Ramp, keep_traj: bool) -> BillCell {
        let free = StatorLeg::default();
        let (traj, nu0) = self.stator_march(flight, ramp, None, &free);
        let b: Vec<f64> = traj.iter().map(|p| self.b_at_point(flight, p)).collect();
        let (mut ib, mut ith) = (0.0, 0.0);
        for i in 1..traj.len() {
            let h = traj[i].s - traj[i - 1].s;
            ib += 0.5 * h * (b[i] + b[i - 1]);
            ith += 0.5 * h * (traj[i - 1].sp_thrust * traj[i - 1].mdot_air
                              + traj[i].sp_thrust * traj[i].mdot_air);
        }
        let d = self.read(&traj, None);
        let lo = d.lp.min_phi;
        // FIRST-STRICT argmin, as Python's `min(key=...)` is: on a tie the EARLIER point wins.
        let mut ai = 0usize;
        for i in 1..traj.len() {
            if traj[i].phi_lp < traj[ai].phi_lp { ai = i; }
        }
        let flat: Vec<f64> = traj.iter().filter(|p| p.phi_lp <= lo * (1.0 + 1e-12))
                                 .map(|p| p.s).collect();
        let (mut fmin, mut fmax) = (f64::INFINITY, f64::NEG_INFINITY);
        for &s in &flat { if s < fmin { fmin = s; } if s > fmax { fmax = s; } }
        let mut b_peak = b[0];
        for &x in &b[1..] { if x > b_peak { b_peak = x; } }
        let mut tt4_peak = traj[0].tt4;
        for p in &traj[1..] { if p.tt4 > tt4_peak { tt4_peak = p.tt4; } }
        let last = &traj[traj.len() - 1];
        BillCell {
            nu_at_min_lp: traj[ai].nu_lp,
            s_at_min_lp: traj[ai].s,
            b_at_min_lp: b[ai],
            plateau_span: fmax - fmin,
            plateau_pts: flat.len(),
            min_phi_lp: d.lp.min_phi,
            min_phi_hp: d.hp.min_phi,
            m_i_lp: d.lp.m_i,
            m_i_hp: d.hp.m_i,
            b_int: ib,
            b_peak,
            b_end: b[b.len() - 1],
            thrust_int: ith,
            thrust_end: last.sp_thrust * last.mdot_air,
            nu_lp_end: last.nu_lp,
            nu_hp_end: last.nu_hp,
            tt4_peak,
            nu0_lp: nu0.0,
            nu0_hp: nu0.1,
            npts: traj.len(),
            traj: if keep_traj { Some(traj) } else { None },
        }
    }
}

impl ScheduledStatorCore {
    /// **RUNG 64, HALF ONE.** The ceiling on the protected coordinate belongs to `b_max`.
    ///
    /// Four laws on identical hardware: valve SHUT, rung 62's SCHEDULE, constant `b = b_max`
    /// (FULLY OPEN throughout), and a FLOOR set `sm_over` ABOVE the fully-open march's own
    /// minimum — i.e. deliberately unreachable.
    ///
    /// `b = b_max` is ITSELF AN OPEN-LOOP LAW and it bounds every admissible `b`-history from
    /// above, so `over`'s `min_phi` cannot exceed `full`'s no matter what the loop does. The
    /// over-set floor is the witness: it SATURATES and is VIOLATED — the first law in this family
    /// that cannot deliver its own set point, and it fails on hardware, not on control.
    ///
    /// **EVERY LEG IS AN `at_lever` SIBLING, SO EVERY LEG CARRIES `R64`.** A bare sibling on rung
    /// 62's table would hit [`LeverHooks::b_at_point`]'s panic on line one — checked before the
    /// port was written, § 5.22 (x).
    pub fn authority_ceiling(
        &self, flight: &FlightCondition, ramp: &Ramp, b_max: f64, n_lo: f64, sm_over: f64,
    ) -> AuthorityCeiling {
        assert!(0.0 < b_max && b_max < 0.5, "rung-64 ceiling needs rung 42's valve bound");
        let shut = self.bare_lever().bill_cell(flight, ramp, false);
        let schedule = self.at_lever(&LeverArm::scheduled(BleedSchedule::new(b_max, n_lo)))
                           .bill_cell(flight, ramp, false);
        let full = self.at_lever(&LeverArm::constant(b_max)).bill_cell(flight, ramp, false);
        let ceiling = full.min_phi_lp;
        let over_lim = ceiling * (1.0 + sm_over);
        let over = self.at_lever(&LeverArm::floored(BleedLimiter::new(over_lim, b_max)))
                       .bill_cell(flight, ramp, false);
        let cmap = self.arming().map_lp_design;
        AuthorityCeiling {
            r: ramp.r,
            ds: ramp.ds,
            b_max,
            phi_surge: cmap.phi_surge,
            gap_schedule: ceiling - schedule.min_phi_lp,
            b_at_sched_min: schedule.b_at_min_lp,
            sched_saturated: schedule.b_at_min_lp >= b_max,
            violated: over.min_phi_lp < over_lim,
            over_deficit: over.min_phi_lp - over_lim,
            bounded_by_full: over.min_phi_lp <= ceiling,
            over_vs_full: over.min_phi_lp - ceiling,
            ceiling,
            phi_lim_over: over_lim,
            shut,
            schedule,
            full,
            over,
        }
    }

    /// The open-loop setting whose march has `min phi_lp == target`.
    ///
    /// **AN OUTER ROOT OVER MARCHES** — expensive by construction, and the only honest way to
    /// match: rung 60's pinning gives the floor its coordinate for free, so an open-loop law must
    /// be DRIVEN to the same one before any bill may be compared.
    pub fn match_open_loop(
        &self, flight: &FlightCondition, ramp: &Ramp, make: &dyn Fn(f64) -> LeverArm,
        lo: f64, hi: f64, target: f64, tol: f64,
    ) -> f64 {
        let f = |x: f64| -> f64 {
            self.at_lever(&make(x)).bill_cell(flight, ramp, false).min_phi_lp - target
        };
        let (flo, fhi) = (f(lo), f(hi));
        assert!(flo < 0.0 && 0.0 < fhi,
                "rung-64 match does not bracket phi_lp = {target} on [{lo}, {hi}]: f(lo) = \
                 {flo:+.6}, f(hi) = {fhi:+.6}. A target above the FULLY-OPEN march's own minimum \
                 is unreachable by ANY law -- that is `authority_ceiling`.");
        try_illinois(|x| Ok(f(x)), lo, hi, flo, fhi, tol, ILLINOIS_MAXIT)
            .expect("infallible residual")
    }

    /// **RUNG 64, HALF TWO — THE RUNG.** Three laws of ONE lever, matched to the SAME
    /// `min phi_lp`, billed in rung 61's currency.
    ///
    /// ```text
    /// 1 constant b        state-BLIND open loop                  (rung 42)
    /// 2 schedule b(n_L)   state-FED   open loop                  (rung 62)
    /// 3 phi floor         CLOSED loop on the protected variable  (rung 64)
    /// ```
    ///
    /// which is the ladder's own information ordering, one lever over from the fuel side.
    ///
    /// The match is EXACT for law 3 by rung 60's tautology and DRIVEN for laws 1 and 2 by
    /// [`match_open_loop`](Self::match_open_loop). **THE COMPARATOR IS LAW 2, NOT LAW 1**: a
    /// constant bleed through a transient is a straw man — it bleeds hardest where `phi` is
    /// already highest.
    ///
    /// Billed in rung 61's currency (`nu_*_end`, thrust) and NOT merely in `int b ds`, because
    /// rung 61's own finding is that the two need not track.
    pub fn matched_bill(
        &self, flight: &FlightCondition, ramp: &Ramp, phi_target: f64, b_cap: f64, n_lo: f64,
        b_hi: f64,
    ) -> MatchedBill {
        let b_star = self.match_open_loop(flight, ramp, &LeverArm::constant,
                                          0.0, b_cap, phi_target, 1e-7);
        let bmax_star = self.match_open_loop(
            flight, ramp, &|x| LeverArm::scheduled(BleedSchedule::new(x, n_lo)),
            1e-9, b_hi, phi_target, 1e-7);
        let shut = self.bare_lever().bill_cell(flight, ramp, false);
        let constant = self.at_lever(&LeverArm::constant(b_star)).bill_cell(flight, ramp, false);
        let schedule = self.at_lever(&LeverArm::scheduled(BleedSchedule::new(bmax_star, n_lo)))
                           .bill_cell(flight, ramp, false);
        let floor = self.at_lever(&LeverArm::floored(BleedLimiter::new(phi_target, b_cap)))
                        .bill_cell(flight, ramp, false);
        let row = |c: &BillCell| BillRow {
            d_nu_lp_end: c.nu_lp_end - shut.nu_lp_end,
            d_nu_hp_end: c.nu_hp_end - shut.nu_hp_end,
            d_thrust_end: c.thrust_end - shut.thrust_end,
            thrust_end_pct: (c.thrust_end / shut.thrust_end - 1.0) * 100.0,
            thrust_int_pct: (c.thrust_int / shut.thrust_int - 1.0) * 100.0,
            d_min_phi_hp: c.min_phi_hp - shut.min_phi_hp,
            b_int: c.b_int,
            b_peak: c.b_peak,
        };
        let mut matched: f64 = 0.0;
        for c in [&constant, &schedule, &floor] {
            let d = (c.min_phi_lp - phi_target).abs();
            if d > matched { matched = d; }
        }
        MatchedBill {
            r: ramp.r,
            ds: ramp.ds,
            phi_target,
            b_cap,
            n_lo,
            b_star,
            bmax_star,
            bill_constant: row(&constant),
            bill_schedule: row(&schedule),
            bill_floor: row(&floor),
            matched,
            saturated: floor.b_peak >= b_cap,
            b_ratio_const: floor.b_int / constant.b_int,
            b_ratio_sched: floor.b_int / schedule.b_int,
            shut,
            constant,
            schedule,
            floor,
        }
    }

    /// **RUNG 64's CLOSING LEG** — rung 63 § 3's refusal, with BOTH objects now watching `phi`.
    ///
    /// Rung 63 found a `phi` FUEL floor and the IMPOSED valve have no composable middle. With both
    /// watching `phi_lp` the band collapses, and the reason is stronger than disarming:
    ///
    /// > **A CLOSED-LOOP LEVER DOES NOT DISARM A SECOND LIMITER ON THE SAME VARIABLE — IT DELETES
    /// > THAT LIMITER'S PLANT.**
    ///
    /// DERIVED, not measured. `_surge_fuel` solves `G(w) = phi_lim - phi(w) = 0` in the fuel `w`,
    /// on its own stated premise that `phi` falls MONOTONICALLY with fuel at fixed spool speeds.
    /// Where this valve RIDES it re-pins `phi_lp` to `phi_lim` at ANY fuel, so `dphi/dWf = 0` and
    /// `G == 0` across the entire bracket: the leg's set-point solve is DEGENERATE and returns an
    /// arbitrary point of a continuum. Its authority over `phi` is not inverted
    /// (`docs/phi-rate-limiter-negative.md`) but ZERO.
    ///
    /// `s_eng` is deliberately NOT reported, for rung 63 § 3's reason: a floor violated from
    /// `s = 0` has no upward crossing and `_s_eng` returns NaN.
    pub fn floor_refusal(
        &self, flight: &FlightCondition, ramp: &Ramp, sm: f64, b_cap: f64, d_sm: f64,
    ) -> FloorRefusal {
        assert!(0.0 < d_sm && d_sm <= sm,
                "rung-64's control floor sits strictly BELOW the valve's");
        let cmap = self.arming().map_lp_design;
        let fuel_lim = SurgeLimiter::from_margin(&cmap, Spool::Lp, sm);
        let below_lim = SurgeLimiter::from_margin(&cmap, Spool::Lp, sm - d_sm);
        let valve = BleedLimiter::from_margin(&cmap, b_cap, sm);
        let bare = self.bare_lever();
        let armed = self.at_lever(&LeverArm::floored(valve));
        let free = StatorLeg::default();
        let leg = |l: SurgeLimiter| StatorLeg { accel: None, surge: Some(Floor::Phi(l)),
                                                tt4_max: None };
        let sp = Spool::Lp;
        let neither = bare.cell(flight, ramp, sp, &free);
        let fuel = bare.cell(flight, ramp, sp, &leg(fuel_lim));
        let valve_c = armed.cell(flight, ramp, sp, &free);
        let both = armed.cell(flight, ramp, sp, &leg(fuel_lim));
        let below_bare = bare.cell(flight, ramp, sp, &leg(below_lim));
        let below_armed = armed.cell(flight, ramp, sp, &leg(below_lim));
        FloorRefusal {
            sm,
            d_sm,
            phi_lim: fuel_lim.phi_lim,
            phi_lim_below: below_lim.phi_lim,
            r: ramp.r,
            ds: ramp.ds,
            b_cap,
            removed_alone: fuel.fuel_removed,
            removed_together: both.fuel_removed,
            inert: (both.m_i - valve_c.m_i).abs() < 1e-14
                && (both.min_phi - valve_c.min_phi).abs() < 1e-14,
            credit: both.m_i - fuel.m_i,
            control_dormant: below_armed.fuel_removed == 0.0 && below_bare.fuel_removed > 0.0,
            removed_below_bare: below_bare.fuel_removed,
            removed_below_armed: below_armed.fuel_removed,
            neither,
            fuel,
            valve: valve_c,
            both,
            below_bare,
            below_armed,
        }
    }
}
