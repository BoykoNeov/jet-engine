//! RUNG 80 — **THE SPLIT WALL.** `SplitWallTransient`, slice AI.
//!
//! Every floor in this family since rung 49 comes from ONE margin through ONE factory —
//! `phi_lim = (1 + sm)·phi_surge` feeds the fuel leg (49), the valve (64) and the stator (68) as the
//! same float. This rung gives the AIRFLOW legs their own margin:
//!
//! ```text
//! phi_lim = (1 + sm    )·phi_surge      the fuel leg's floor
//! phi_air = (1 + sm_air)·phi_surge      the valve's and the stator's
//! ```
//!
//! `sm_air = None` is the inherited SHARED wall by EXACT DISPATCH. Headline: **a set of floors on
//! one variable is a total order, and only the top one is ever live** — splitting the wall
//! re-labels which loop leads; it does not put them in parallel.
//!
//! # WHAT STEP 1 OF THIS SLICE ADDS AT THIS RUNG — **TWO RE-AIMED POINTERS AND NO NEW TABLE FIELD**
//!
//! | | Python | slot | table |
//! |---|---|---|---|
//! | swap | `at_lever` (`engine.py:21429`) | `LeverHooks::at_lever` | [`R80`] |
//! | swap | `_shared_rig` (`engine.py:21450`) | [`shared_rig`](TripleHooks::shared_rig) | [`R80_TRIPLE`] |
//!
//! Plus [`walls_of`] (`engine.py:21495`), which the re-aimed rig cannot ship without — its second
//! refusal reads the walls back off the built machine. The rest of rung 80 (`_with_air`,
//! `_split_march`, `_split_row` and the four readers) is step 4's.
//!
//! **Rung 80 inherits rung 79's `cap_fuel` and `with_coord` UNCHANGED**, and that is why a rung-80
//! machine carries § 5.33 (i)'s latent setter defect too: the pre-flight measured the same
//! `("clip", "demand")` and the same 128 branch entries there. `tests/slice_ai_cells.rs` names both
//! inheritances rather than leaving them to the equality sweep.
//!
//! **THE CARRY CHAIN ENDS HERE** (§ 5.33 (ii), probe 8): the four classes after rung 80 define no
//! `at_lever` at all, so every rig a rung-81–84 machine builds is a `SplitWallTransient` by class.
//! Booked forward to slice AJ: its `R81`…`R84` lever tables must carry [`R80`]'s `at_lever`.
//!
//! # THE TWO REFUSALS PANIC, AND ONE OF THEM FIRES ON A LEGAL INPUT
//!
//! Both are `assert`s in `_shared_rig`, whose cell returns a tuple, not a `Result`; and no Python
//! caller of `_shared_rig` wraps it in `except AssertionError` (checked at all seven call sites), so
//! slice L's per-call-site rule gives `panic!`.
//!
//! **The second (`engine.py:21486`) is written to catch a WIRING bug** — *"the split did not reach
//! the plant"* — **and it also fires on an admissible input** (§ 5.33 (vi)): at the fingerprint's
//! `sm = 0.4545454545454546`, `phi_surge = 0.55`, an `sm_air` one or two ulp above `sm` rebuilds a
//! wall that rounds to the SAME float as the fuel leg's, because `1 + sm_air` has four times `sm`'s
//! ulp. Python's reading stands — a split that does not separate the walls must not march as one —
//! but the band is a float fact, so the wall arithmetic here is Python's to the operation:
//! `(1.0 + sm_air) * phi_surge` inside each limiter's own `from_margin`, and plain `==` / `>=` / `>`
//! in both tests. Gated at step 5 (P5: `+1`, `+2` ulp refuse, `+3` marches).

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{AsymmetricLag, Floor, FuelTransientHooks};
use crate::limited_bleed::BleedLimiter;
use crate::map::ComponentMap;
use crate::reference_split::StatorIncidenceLimiter;
use crate::shared_actuator::SharedRigArm;
use crate::state_coordinate::{R79, R79_FUEL, R79_STATOR, R79_TRIPLE, R79_TWO};
use crate::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks};
use crate::three_loop::{StatorLimiter, TripleHooks};
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::TwoSpoolTransientHooks;

// ---------------------------------------------------------------------------------------------
// THE CASCADE BUILDER
// ---------------------------------------------------------------------------------------------

/// Build a rung-80 object, so every sibling re-asserts the whole chain's guards.
///
/// `_ref_law` is set for rung 73's reason. `_sm_air` is NOT written: its class default is `None`
/// and so is the core's.
pub fn build_split_wall_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = crate::reference_split::build_split_family_cascade(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
        &R80_TWO, &R80_STATOR, &R80_FUEL, &R80, &R80_TRIPLE);
    if let ScheduledStatorTransient::Full(c) = &built {
        c.fuel.inner.ref_law.set(crate::applied_reference::REF_LAW_APPLIED);
    }
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES
// ---------------------------------------------------------------------------------------------

/// RUNG 80's lever table — ONE swap, `at_lever`, against rung 79's. The EIGHTEENTH instance of the
/// trap, and the LAST: nothing after rung 80 defines `at_lever`.
pub const R80: LeverHooks = LeverHooks {
    at_lever: r80_at_lever,
    ..R79
};

/// RUNG 80's `TwoSpoolTransientHooks` — **ZERO cells swapped**, an alias.
pub const R80_TWO: TwoSpoolTransientHooks = R79_TWO;

/// RUNG 80's fuel table — **ZERO cells swapped**, an alias.
pub const R80_FUEL: FuelTransientHooks = R79_FUEL;

/// RUNG 80's stator table — **ZERO cells swapped**, an alias.
pub const R80_STATOR: StatorTransientHooks = R79_STATOR;

/// RUNG 80's third-loop table — **ONE of rung 79's eighteen cells re-aimed, and NOTHING added.**
///
/// Spelled out field by field, [`R79_TRIPLE`]'s reason. `cap_fuel` and `with_coord` stay RUNG 79's
/// — the two inheritances that carry § 5.33 (i)'s defect onto this rung.
pub const R80_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: R79_TRIPLE.stator_leg,
    lagged_stator: R79_TRIPLE.lagged_stator,
    clamp_v: R79_TRIPLE.clamp_v,
    check_v0: R79_TRIPLE.check_v0,
    rk4_floor: R79_TRIPLE.rk4_floor,
    solve_v: R79_TRIPLE.solve_v,
    manifold_v: R79_TRIPLE.manifold_v,
    triple_laws: R79_TRIPLE.triple_laws,
    triple_rig: R79_TRIPLE.triple_rig,
    with_ref: R79_TRIPLE.with_ref,
    reference: R79_TRIPLE.reference,
    quad_gains_at: R79_TRIPLE.quad_gains_at,
    rk4_floor_shared: R79_TRIPLE.rk4_floor_shared,
    windup_tau: R79_TRIPLE.windup_tau,
    sensed_cap: R79_TRIPLE.sensed_cap,
    cap_fuel: R79_TRIPLE.cap_fuel,
    with_coord: R79_TRIPLE.with_coord,
    // THE ONE THIS RUNG RE-AIMS.
    shared_rig: r80_shared_rig,
};

// ---------------------------------------------------------------------------------------------
// THE RE-AIMED BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 80's `at_lever` — **a RUNG-80 machine carrying NINE knobs** (`engine.py:21444`–`21447`).
///
/// Dropping `_sm_air` here would rebuild every rig at the SHARED wall while the caller reported a
/// split one — and because the split's whole signature is *the levers do nothing*, the loss would
/// return this rung's own predicted result having measured rung 79.
fn r80_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let m = match build_split_wall_cascade(
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
    m.fuel.inner.gauge_k.set(core.fuel.inner.gauge_k.get());
    m.fuel.inner.phi_ref.set(core.fuel.inner.phi_ref.get());
    m.fuel.inner.sm_air.set(core.fuel.inner.sm_air.get());
    m
}

/// The floors as they stand ON THE BUILT MACHINE — Python's `_walls_of` (`engine.py:21495`).
///
/// `None` where Python's dict holds `None`. **The order of the three-way `phi_air` choice is
/// Python's — valve, then `phi` stator, then incidence stator** — and the incidence wall is read
/// back through [`phi_lim_at`](StatorIncidenceLimiter::phi_lim_at), a ROUND TRIP through `m_lim`,
/// not re-derived as `(1 + sm)·phi_surge`. Which wall is read decides the ulp band at
/// `engine.py:21486`, so neither the order nor the round trip is a free choice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Walls {
    /// The fuel leg's floor, off `surge`.
    pub phi_lim: Option<f64>,
    /// The airflow legs' floor: the valve's if armed, else the stator's.
    pub phi_air: Option<f64>,
    /// The valve's own.
    pub phi_valve: Option<f64>,
    /// The stator's own, in either reference.
    pub phi_stator: Option<f64>,
}

/// See [`Walls`]. `static` in Python, so no receiver beyond the machine it reads.
pub fn walls_of(m: &ScheduledStatorCore, surge: Option<&Floor>) -> Walls {
    let cmap = m.arming().map_lp_design;
    let bl = m.fuel.inner.lever.lim;
    let sl = m.fuel.inner.stator.lim;
    let si = m.fuel.inner.stator.inc;
    let stator = sl.map(|l| l.phi_lim).or_else(|| si.map(|i| i.phi_lim_at(&cmap)));
    Walls {
        phi_lim: surge.map(|s| s.phi().phi_lim),
        phi_air: bl.map(|l| l.phi_lim).or(stator),
        phi_valve: bl.map(|l| l.phi_lim),
        phi_stator: stator,
    }
}

/// RUNG 80's `_shared_rig` — **the split applied ON THE MACHINE rung 79 RETURNS, never threaded as
/// an argument** (`engine.py:21450`).
///
/// `_shared_rig` is overridden six times up this ladder, each calling its parent with an EXPLICIT
/// argument list, so a new keyword at the base would be swallowed and the reduce test would pass
/// BECAUSE the knob was ignored. Python carries the knob as an attribute and asserts the walls apart
/// by reading them back; so does this.
///
/// **`b_max` AND EVERY `tau` ARE READ OFF THE MACHINE rung 79 BUILT, NOT OFF `core`** — Python's
/// `m.bleed_lim.b_max`, `m.bleed_lim.tau`, … The rebuilt limiter keeps the hardware and moves only
/// the floor.
fn r80_shared_rig(
    core: &ScheduledStatorCore, arm: &SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let (mut m, surge, lag) = (R79_TRIPLE.shared_rig)(core, arm);
    let sm_air = core.fuel.inner.sm_air.get();
    m.fuel.inner.sm_air.set(sm_air);
    let Some(sm_air) = sm_air else {
        return (m, surge, lag); // EXACT DISPATCH: rung 79, bit-for-bit
    };
    let sm = arm.sm;
    assert!(sm_air >= sm,
            "rung-80: the AIRFLOW wall sits AT or ABOVE the fuel leg's -- that is the whole \
             construction (the levers must own a violation the fuel leg is not asked to \
             prevent). Got sm_air = {sm_air} below sm = {sm}; below it the fuel leg is the top \
             floor again and this is rungs 49-79 with an extra float.");
    let cmap = core.arming().map_lp_design;
    if let Some(bl) = m.fuel.inner.lever.lim {
        m.fuel.inner.lever.lim =
            Some(BleedLimiter::from_margin_tau(&cmap, bl.b_max, sm_air, bl.tau));
    }
    if let Some(sl) = m.fuel.inner.stator.lim {
        m.fuel.inner.stator.lim = Some(StatorLimiter::from_margin(&cmap, sl.v_max, sm_air, sl.tau));
    }
    if let Some(si) = m.fuel.inner.stator.inc {
        m.fuel.inner.stator.inc =
            Some(StatorIncidenceLimiter::from_margin(&cmap, si.v_max, sm_air, si.tau));
    }
    // THE DROPPED-KWARG CONTROL, run on the OBJECTS and not on the arguments.
    let w = walls_of(&m, surge.as_ref());
    let apart = match (w.phi_air, w.phi_lim) {
        (Some(air), Some(lim)) => air > lim,
        _ => true,
    };
    assert!(sm_air == sm || apart,
            "rung-80: the split did not reach the plant -- phi_air = {:?} is not above phi_lim = \
             {:?} at sm_air = {sm_air} > sm = {sm}. The rig would then march rung 79 while every \
             reader here reported a split wall, and the levers' silence would read as this \
             rung's finding.", w.phi_air, w.phi_lim);
    (m, surge, lag)
}
