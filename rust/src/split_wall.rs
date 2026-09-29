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
//! refusal reads the walls back off the built machine.
//!
//! # WHAT STEP 4 ADDS — THE REST OF RUNG 80, AND NO TABLE CELL
//!
//! [`AirScope`] (`_with_air`), [`split_march`], [`split_row`] and the four readers
//! [`split_liveness`], [`split_arrest`], [`split_saturation`] and [`split_gains`] — all
//! single-definer, so none is a cell. Gated bit for bit in `tests/slice_ai_split.rs`.
//!
//! **On every one of them rung 79's six counters stay at ZERO** (measured in Python before the port):
//! rung 80 marches in `phi` reference, so rung 79's incidence branch is never entered. The
//! instrument that caught steps 2–3's coordinate defects is blind here; what sees a coordinate
//! written into the wrong field is the READBACK of `(lag_coord, phi_ref)` on the built rig, and the
//! values themselves.
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

use std::cell::Cell;

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AsymmetricLag, Authority, Floor, FuelPoint, FuelTransientHooks, PointExtra,
};
use crate::gas::Abort;
use crate::limited_bleed::BleedLimiter;
use crate::map::ComponentMap;
use crate::reference_split::StatorIncidenceLimiter;
use crate::shared_actuator::{
    charpoly4, jac4, quartic_roots_c, riding4, ShareScope, SharedRigArm,
};
use crate::state_coordinate::{R79, R79_FUEL, R79_STATOR, R79_TRIPLE, R79_TWO};
use crate::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks};
use crate::three_loop::{StatorLimiter, TripleHooks};
use crate::two_lag::{py_max_of, py_min_of};
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
///
/// **BUT ON EVERY SHIPPED PATH THE ROUND TRIP IS NEVER READ** — measured at slice AI step 6 (plan
/// § 5.33.6). Every rung-80 rig arms the VALVE, `phi_air` takes the valve's wall first, and
/// [`split_row`] refuses a valve-less machine; `phi_stator` enters no reported field. Reading the
/// incidence wall as `999.0` moved 0 of the oracle's 37 945 keys and no suite gate. The branch
/// is Python's shape kept, and it decides the band only on a valve-less rig no reader builds.
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

// =============================================================================================
// SLICE AI STEP 4 — THE REST OF RUNG 80: `_with_air`, `_split_march`, `_split_row` AND THE FOUR
// READERS (`engine.py:21512`–`21792`)
// =============================================================================================

/// `split_liveness`'s Python default `phi_airs`. The four readers' defaults are named once here so
/// the ported suite and the oracle cannot drift from them.
pub const LIVENESS_PHI_AIRS: [Option<f64>; 3] = [None, Some(0.76), Some(0.77)];
/// `split_liveness`'s default `coords`.
pub const LIVENESS_COORDS: [&str; 2] = ["demand", "clip"];
/// `split_arrest`'s default `walls`.
pub const ARREST_WALLS: [f64; 6] = [0.7700, 0.7725, 0.7731, 0.7732, 0.7740, 0.7800];
/// `split_saturation`'s default `phi_airs` — the eight walls `docs/rung80-spec.md` § 4 reports.
pub const SATURATION_PHI_AIRS: [f64; 8] = [0.78, 0.80, 0.82, 0.84, 0.85, 0.855, 0.86, 0.88];
/// `split_gains`'s default `phi_airs`.
pub const GAINS_PHI_AIRS: [Option<f64>; 2] = [None, Some(0.77)];
/// `split_gains`'s default stride — **5, not rung 72's 4.**
pub const GAINS_EVERY: usize = 5;
/// `_split_row`'s default `tol`.
pub const ROW_TOL: f64 = 1e-12;

/// Python's two-argument `max(x, y)` — `x` unless `y` is STRICTLY larger. Never `f64::max`,
/// which returns the other argument on a `NaN`.
fn py_max2(x: f64, y: f64) -> f64 {
    if y > x { y } else { x }
}

/// Python's `max(xs, default=None)` over a list that may hold `None` — **and it refuses where
/// Python raises.** Two or more elements with a `None` among them compare `None` with something,
/// which is Python's `TypeError`; a lone element is returned as itself. Every shipped cell carries
/// a `Some`, so the refusing arm is Python's shape kept, not a reached path.
fn py_max_opt(xs: &[Option<f64>]) -> Option<f64> {
    match xs {
        [] => None,
        [x] => *x,
        _ => {
            let v: Vec<f64> = xs.iter()
                .map(|x| x.unwrap_or_else(|| panic!(
                    "rung-80: `max` over a list holding None -- Python's TypeError")))
                .collect();
            Some(py_max_of(&v))
        }
    }
}

// ---------------------------------------------------------------------------------------------
// `_with_air` — THE KNOB'S SCOPE
// ---------------------------------------------------------------------------------------------

/// RUNG 80's `_with_air` (`engine.py:21512`) — run a reader at a named AIRFLOW margin, restored on
/// drop, which runs on an unwind too (Python's `finally`).
///
/// **A DIRECT WRITE, NOT A DISPATCH.** `_with_air` is single-definer over all 58 classes, so the
/// setter rule ([`CoordScope`](crate::demand_coordinate::CoordScope)'s doc) gives a guard on the
/// field itself — `ShareScope`'s and `_with_windup`'s precedent (§ 5.33 (ii)).
pub struct AirScope<'a> {
    cell: &'a Cell<Option<f64>>,
    prev: Option<f64>,
}

impl<'a> AirScope<'a> {
    /// Set `_sm_air` for as long as the returned guard lives.
    pub fn set(core: &'a ScheduledStatorCore, sm_air: Option<f64>) -> Self {
        let cell = &core.fuel.inner.sm_air;
        let prev = cell.get();
        cell.set(sm_air);
        AirScope { cell, prev }
    }

    /// What this scope displaced — so a gate can read the restore POLICY, not only its effect.
    pub fn displaced(&self) -> Option<f64> {
        self.prev
    }
}

impl Drop for AirScope<'_> {
    fn drop(&mut self) {
        self.cell.set(self.prev);
    }
}

// ---------------------------------------------------------------------------------------------
// `_split_march` — ONE RIG AT TWO WALLS
// ---------------------------------------------------------------------------------------------

/// RUNG 80's `_split_march` (`engine.py:21524`) — **one rig at TWO walls, marched under a named
/// coordinate.** `phi_air = None` is the shared wall, i.e. exactly rung 74's `_coord_march`.
///
/// # THE TWO MARGINS ARE PYTHON'S ARITHMETIC TO THE OPERATION
///
/// `sm = phi_lim / ps - 1.0` and `sm_air = phi_air / ps - 1.0` — a divide, then a subtract. The
/// rig rebuilds each wall as `(1 + sm_air)·ps` inside the limiters' `from_margin`, so this round
/// trip is where `engine.py:21486`'s ulp band comes from (§ 5.33 (vi)); another spelling of the
/// margin moves which inputs refuse.
///
/// # IT CALLS RUNG 74's `_coord_march`, FULLY QUALIFIED
///
/// `self._coord_march` (`engine.py:21531`) is single-definer — rung 79's public `coord_march` is a
/// DIFFERENT method — so a rung-80 machine runs rung 74's body, which writes `_lag_coord` by plain
/// ASSIGNMENT on the rig it builds. Named [`crate::demand_coordinate::coord_march`] at the call by
/// § 5.33 (ix)'s naming rule, with no `use` of either bare name. `ref = "sched"` is Python's
/// default and no reader passes another; `nu0` is `None` because `_coord_march`'s own default is.
#[allow(clippy::too_many_arguments)]
pub fn split_march(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, phi_air: Option<f64>, coord: &'static str, taus: (f64, f64, f64, f64), r: f64,
    s_settle: f64, ds: f64, v_max: f64, inc: bool,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>, Vec<FuelPoint>) {
    let ps = core.arming().map_lp_design.phi_surge;
    let sm = phi_lim / ps - 1.0;
    let sm_air = phi_air.map(|pa| pa / ps - 1.0);
    let _air = AirScope::set(core, sm_air);
    crate::demand_coordinate::coord_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc, coord,
        "sched", None)
}

// ---------------------------------------------------------------------------------------------
// `_split_row` — THE ROW, WITH ITS COUNTERS NAMED
// ---------------------------------------------------------------------------------------------

/// Python's `p["b"], p["v"], p["required_fuel"], p["required_gov"]` — the two six-state routes
/// rung 74's `_coord_march` produces (`clip` → `Shared`, `demand` → `Demand`). **Refuses anything
/// else**, which is Python's `KeyError`.
fn bv_req(p: &FuelPoint) -> (f64, f64, f64, f64) {
    match p.extra {
        PointExtra::Shared { b, v, required_fuel, required_gov, .. }
        | PointExtra::Demand { b, v, required_fuel, required_gov, .. } =>
            (b, v, required_fuel, required_gov),
        _ => panic!("rung-80: this point carries no `b`/`v`/`required_*` -- it was not marched by \
                     rung 74's `_coord_march`. Python raises `KeyError` here."),
    }
}

/// One row of every rung-80 table — Python's `_split_row` dict, fields in its key order.
///
/// **THREE COUNTERS THAT ARE THREE DIFFERENT NOUNS** — motion ([`valve_moved`](Self::valve_moved),
/// [`stator_moved`](Self::stator_moved)), liveness ([`n_cut_fuel`](Self::n_cut_fuel)) and
/// [`n_riding4`](Self::n_riding4) — never to be differenced. And `n_riding4` is meaningless where
/// [`arrested`](Self::arrested) is true; [`riding4_valid`](Self::riding4_valid) carries the pairing.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitRow {
    pub coord: &'static str,
    pub phi_lim: f64,
    pub phi_air: Option<f64>,
    pub npts: usize,
    /// The fuel wall AS BUILT, read back off the rig — no table can quote a wall it did not carry.
    pub phi_lim_built: Option<f64>,
    /// The airflow wall AS BUILT.
    pub phi_air_built: Option<f64>,
    pub phi0: f64,
    pub b0: f64,
    pub v0: f64,
    pub b0_frac: f64,
    pub min_phi: f64,
    pub max_tt4: f64,
    /// Rung 74's own detector: the plant never left its initial power setting.
    pub arrested: bool,
    pub riding4_valid: bool,
    /// LIVENESS — the noun P1 is scored on. `required_* > 0.0`, which on a `demand` point reads an
    /// UNFLOORED projection and so can see a negative (§ 5.29's sign question).
    pub n_cut_fuel: usize,
    pub n_cut_gov: usize,
    /// **No `default`**, and negative on a `demand` trajectory for the same reason.
    pub max_req_fuel: f64,
    pub max_req_gov: f64,
    /// MOTION — strictly `> tol` from the initial condition.
    pub valve_moved: usize,
    pub stator_moved: usize,
    /// ALL FOUR RIDING AND STRICTLY INTERIOR — rung 72's `_riding4`, NOT its `n_live`.
    pub n_riding4: usize,
    pub b_max_hit: bool,
}

/// RUNG 80's `_split_row` (`engine.py:21535`) — `static` in Python, so it reads the BUILT machine
/// `m`, never the caller's core.
///
/// # THE REFUSAL (`engine.py:21551`) SITS AFTER THREE READS, AS IN PYTHON
///
/// `b0`, `v0` and `maxT` are read before the `bleed_lim is not None` assert, so a trajectory
/// without those keys dies on its `KeyError` first. No reader catches `AssertionError`, so the
/// refusal is a `panic!` (slice L's rule); its gate is step 5's.
#[allow(clippy::too_many_arguments)]
pub fn split_row(
    m: &ScheduledStatorCore, surge: Option<&Floor>, traj: &[FuelPoint], tt4_lo: f64,
    phi_lim: f64, phi_air: Option<f64>, coord: &'static str, tol: f64,
) -> SplitRow {
    let (b0, v0, _, _) = bv_req(&traj[0]);
    let tt4s: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
    let max_t = py_max_of(&tt4s);
    let b_max = m.fuel.inner.lever.lim.unwrap_or_else(|| panic!(
        "rung-80: `n_riding4` counts points with the valve STRICTLY INTERIOR, which needs the \
         valve's own `b_max`. A default stop here would silently redefine the counter on the one \
         rig where it is unarmed, and the number would still print.")).b_max;
    let w = walls_of(m, surge);
    let st: Vec<(f64, f64, f64, f64)> = traj.iter().map(bv_req).collect();
    let phis: Vec<f64> = traj.iter().map(|p| p.phi_lp).collect();
    let req_f: Vec<f64> = st.iter().map(|x| x.2).collect();
    let req_g: Vec<f64> = st.iter().map(|x| x.3).collect();
    SplitRow {
        coord,
        phi_lim,
        phi_air,
        npts: traj.len(),
        phi_lim_built: w.phi_lim,
        phi_air_built: w.phi_air,
        phi0: traj[0].phi_lp,
        b0,
        v0,
        b0_frac: b0 / b_max,
        min_phi: py_min_of(&phis),
        max_tt4: max_t,
        arrested: max_t <= tt4_lo * (1.0 + 1e-9),
        riding4_valid: max_t > tt4_lo * (1.0 + 1e-9),
        n_cut_fuel: req_f.iter().filter(|&&x| x > 0.0).count(),
        n_cut_gov: req_g.iter().filter(|&&x| x > 0.0).count(),
        max_req_fuel: py_max_of(&req_f),
        max_req_gov: py_max_of(&req_g),
        valve_moved: st.iter().filter(|x| (x.0 - b0).abs() > tol).count(),
        stator_moved: st.iter().filter(|x| (x.1 - v0).abs() > tol).count(),
        n_riding4: riding4(traj, b_max).len(),
        b_max_hit: st.iter().any(|x| x.0 >= b_max * (1.0 - 1e-12)),
    }
}

// ---------------------------------------------------------------------------------------------
// § 2 — `split_liveness`: THE TOTAL ORDER
// ---------------------------------------------------------------------------------------------

/// § 2's reading — Python's `split_liveness` return dict.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitLiveness {
    pub phi_lim: f64,
    pub phi_airs: Vec<Option<f64>>,
    pub coords: Vec<&'static str>,
    pub taus: (f64, f64, f64, f64),
    pub ds: f64,
    pub rows: Vec<SplitRow>,
    /// THE POSITIVE CONTROL: `clip` rows exist and every one moved its valve.
    pub control_ok: bool,
    /// P1: every `demand` split row moved its valve …
    pub levers_woke: bool,
    /// … and cut the fuel leg nowhere. The scored pair.
    pub fuel_off: bool,
    /// `max(n_riding4 over the split rows, default=0)`.
    pub four_live: usize,
    pub n_split: usize,
}

/// § 2 (`engine.py:21583`) — **what the split actually buys, in both coordinates.** `clip` is the
/// positive control and is not optional; `phi_air = None` is each coordinate's shared-wall
/// baseline at identical settings. Rows in Python's loop order: coordinate outer, wall inner.
#[allow(clippy::too_many_arguments)]
pub fn split_liveness(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, phi_airs: &[Option<f64>], coords: &[&'static str], taus: (f64, f64, f64, f64),
    inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> SplitLiveness {
    let mut rows = Vec::new();
    for &coord in coords {
        for &pa in phi_airs {
            let (m, surge, _, traj) = split_march(
                core, flight, tt4_lo, tt4_hi, tt4_max, phi_lim, pa, coord, taus, r, s_settle,
                ds, v_max, inc);
            rows.push(split_row(&m, surge.as_ref(), &traj, tt4_lo, phi_lim, pa, coord, ROW_TOL));
        }
    }
    let split: Vec<&SplitRow> = rows.iter().filter(|x| x.phi_air.is_some()).collect();
    let clip: Vec<&SplitRow> = rows.iter().filter(|x| x.coord == "clip").collect();
    let dem: Vec<&SplitRow> = split.iter().copied().filter(|x| x.coord == "demand").collect();
    SplitLiveness {
        phi_lim,
        phi_airs: phi_airs.to_vec(),
        coords: coords.to_vec(),
        taus,
        ds,
        control_ok: !clip.is_empty() && clip.iter().all(|x| x.valve_moved > 0),
        levers_woke: !dem.is_empty() && dem.iter().all(|x| x.valve_moved > 0),
        fuel_off: !dem.is_empty() && dem.iter().all(|x| x.n_cut_fuel == 0),
        four_live: split.iter().map(|x| x.n_riding4).max().unwrap_or(0),
        n_split: split.len(),
        rows,
    }
}

// ---------------------------------------------------------------------------------------------
// § 3 — `split_arrest`: THE ARREST EDGE CHANGES OWNER
// ---------------------------------------------------------------------------------------------

/// One arm of § 3 — `shared`, `air` or `fuel`.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrestArm {
    /// `(wall, row)` — Python's `dict(wall=w, **row)`.
    pub rows: Vec<(f64, SplitRow)>,
    pub marched: Vec<f64>,
    pub arrested: Vec<f64>,
    pub last_march: Option<f64>,
    pub first_arrest: Option<f64>,
    /// A bracket is an EDGE only if the two sets do not interleave.
    pub monotone: bool,
}

/// § 3's reading — Python's `split_arrest` return dict.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitArrest {
    pub walls: Vec<f64>,
    pub phi_lim_lo: f64,
    pub phi_air_hi: f64,
    pub coord: &'static str,
    pub taus: (f64, f64, f64, f64),
    pub ds: f64,
    /// In Python's dict order: `shared`, `air`, `fuel`.
    pub arms: Vec<(&'static str, ArrestArm)>,
    /// THE CONTROL — rung 74's bracket must reappear on the `shared` arm.
    pub control_bracket: (Option<f64>, Option<f64>),
    /// The split arms that arrested anywhere, in `("air", "fuel")` order.
    pub owner: Vec<&'static str>,
}

impl SplitArrest {
    /// The arm under Python's key.
    pub fn arm(&self, name: &str) -> &ArrestArm {
        &self.arms.iter().find(|(k, _)| *k == name).expect("one of shared/air/fuel").1
    }
}

/// § 3 (`engine.py:21623`) — **which floor owns the arrest: three arms across ONE wall axis.**
///
/// # THE REFUSAL (`engine.py:21647`) RUNS FIRST
///
/// The two fixed walls must bracket the swept ones, else an arm changes identity mid-sweep.
/// Python's `max(walls)` / `min(walls)`; no caller catches the `AssertionError`, so a `panic!`.
#[allow(clippy::too_many_arguments)]
pub fn split_arrest(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    walls: &[f64], phi_lim_lo: f64, phi_air_hi: f64, coord: &'static str,
    taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> SplitArrest {
    assert!(phi_air_hi > py_max_of(walls) && phi_lim_lo < py_min_of(walls),
            "rung-80: the two FIXED walls must bracket the swept ones, else an arm changes \
             identity mid-sweep. Got phi_lim_lo = {phi_lim_lo}, walls = {walls:?}, \
             phi_air_hi = {phi_air_hi}.");
    let mut arms: Vec<(&'static str, ArrestArm)> = Vec::new();
    for arm in ["shared", "air", "fuel"] {
        let mut rows = Vec::new();
        for &w in walls {
            let (pl, pa) = match arm {
                "shared" => (w, None),
                "air" => (phi_lim_lo, Some(w)),
                _ => (w, Some(phi_air_hi)),
            };
            let (m, surge, _, traj) = split_march(
                core, flight, tt4_lo, tt4_hi, tt4_max, pl, pa, coord, taus, r, s_settle, ds,
                v_max, inc);
            rows.push((w, split_row(&m, surge.as_ref(), &traj, tt4_lo, pl, pa, coord, ROW_TOL)));
        }
        let marched: Vec<f64> = rows.iter().filter(|x| !x.1.arrested).map(|x| x.0).collect();
        let arrested: Vec<f64> = rows.iter().filter(|x| x.1.arrested).map(|x| x.0).collect();
        let last_march = if marched.is_empty() { None } else { Some(py_max_of(&marched)) };
        let first_arrest = if arrested.is_empty() { None } else { Some(py_min_of(&arrested)) };
        let monotone = marched.is_empty() || arrested.is_empty()
            || py_max_of(&marched) < py_min_of(&arrested);
        arms.push((arm, ArrestArm { rows, marched, arrested, last_march, first_arrest, monotone }));
    }
    let control_bracket = (arms[0].1.last_march, arms[0].1.first_arrest);
    let owner: Vec<&'static str> = ["air", "fuel"].into_iter()
        .filter(|a| !arms.iter().find(|(k, _)| k == a).expect("built above").1.arrested.is_empty())
        .collect();
    SplitArrest {
        walls: walls.to_vec(),
        phi_lim_lo,
        phi_air_hi,
        coord,
        taus,
        ds,
        arms,
        control_bracket,
        owner,
    }
}

// ---------------------------------------------------------------------------------------------
// § 4 — `split_saturation`: THE TWO EDGES ON ONE AXIS
// ---------------------------------------------------------------------------------------------

/// § 4's reading — Python's `split_saturation` return dict.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitSaturation {
    pub phi_lim: f64,
    pub coord: &'static str,
    pub rows: Vec<SplitRow>,
    pub ds: f64,
    pub first_sat: Option<f64>,
    pub last_march: Option<f64>,
    /// Walls saturated AND marching — the four-loop cell's precondition.
    pub cell: Vec<f64>,
    pub impossible: bool,
}

/// § 4 (`engine.py:21679`) — **the saturation edge and the arrest edge in one table**, so the
/// impossibility is read and not argued.
///
/// `phi_airs` is `&[f64]`, not `&[Option<f64>]`: Python's `min(sat)` / `max(march)` compare the
/// walls, and a `None` among them would raise. Each row still stores `Some(pa)`.
#[allow(clippy::too_many_arguments)]
pub fn split_saturation(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, phi_airs: &[f64], coord: &'static str, taus: (f64, f64, f64, f64), inc: bool,
    r: f64, s_settle: f64, ds: f64, v_max: f64,
) -> SplitSaturation {
    let mut rows = Vec::new();
    for &pa in phi_airs {
        let (m, surge, _, traj) = split_march(
            core, flight, tt4_lo, tt4_hi, tt4_max, phi_lim, Some(pa), coord, taus, r, s_settle,
            ds, v_max, inc);
        rows.push(split_row(&m, surge.as_ref(), &traj, tt4_lo, phi_lim, Some(pa), coord,
                            ROW_TOL));
    }
    let wall = |x: &SplitRow| x.phi_air.expect("§ 4's rows are all split");
    let sat: Vec<f64> = rows.iter().filter(|x| x.b_max_hit).map(wall).collect();
    let march: Vec<f64> = rows.iter().filter(|x| !x.arrested).map(wall).collect();
    let first_sat = if sat.is_empty() { None } else { Some(py_min_of(&sat)) };
    let last_march = if march.is_empty() { None } else { Some(py_max_of(&march)) };
    let cell: Vec<f64> = rows.iter().filter(|x| x.b_max_hit && !x.arrested).map(wall).collect();
    // `bool(sat) and (last_march is None or first_sat > last_march)`
    let impossible = !sat.is_empty() && match (first_sat, last_march) {
        (_, None) => true,
        (Some(fs), Some(lm)) => fs > lm,
        (None, Some(_)) => unreachable!("`sat` is non-empty"),
    };
    SplitSaturation { phi_lim, coord, rows, ds, first_sat, last_march, cell, impossible }
}

// ---------------------------------------------------------------------------------------------
// § 5 — `split_gains`: THE DISCRIMINATOR, OR NOTHING
// ---------------------------------------------------------------------------------------------

/// One interior cell of § 5.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitCell {
    pub s: f64,
    pub phi: f64,
    pub authority: Option<Authority>,
    /// Rung 72's noun: the fuel-side leg `min` masks, whose column is predicted EXACTLY zero.
    pub masked: Option<Authority>,
    pub mask_leak: Option<f64>,
    /// Quartic roots with `|z| < 1e-4·rate`.
    pub zeros: usize,
    pub c1: f64,
    pub c0: f64,
    /// `(F_q·C_v)·V_f` — the three `phi` loops around their cycle, multiplied left to right.
    pub cyc_fwd: f64,
    /// `(F_v·V_q)·C_f`.
    pub cyc_rev: f64,
    pub pair_rc: f64,
    pub pair_cv: f64,
}

/// One `phi_air` arm of § 5.
#[derive(Clone, Debug, PartialEq)]
pub struct GainsArm {
    pub phi_air: Option<f64>,
    pub n_riding: usize,
    pub n_sampled: usize,
    pub n_interior: usize,
    /// Python's `skipped` dict: `(switch, regime)`.
    pub skipped: (usize, usize),
    pub cells: Vec<SplitCell>,
    /// Python's `auth` dict — **insertion-ordered**, first appearance first.
    pub authority: Vec<(Option<Authority>, usize)>,
    /// `sorted({c["masked"]})` — in Python's STRING order (`"fuel" < "gov"`).
    pub masked: Vec<Option<Authority>>,
    pub max_mask_leak: Option<f64>,
    /// `sorted({c["zeros"]})`.
    pub zeros: Vec<usize>,
    pub max_cyc: Option<f64>,
}

/// § 5's reading — Python's `split_gains` return dict.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitGains {
    pub phi_lim: f64,
    pub coord: &'static str,
    pub taus: (f64, f64, f64, f64),
    pub ds: f64,
    pub arms: Vec<GainsArm>,
    /// THE VACUITY FLAG, first and unconditional: some arm had no interior point.
    pub vacuous: bool,
    pub n_interior: Vec<usize>,
    pub all_differenced: bool,
    pub ever_two_authorities: bool,
    /// THE POSITIVE CONTROL FOR THE ZERO: the largest cyclic product where the GOVERNOR is masked.
    pub control_nonzero: Option<f64>,
    pub max_mask_leak: Option<f64>,
}

/// Python's `sorted(set(...))` over the `masked` labels. A `None` beside a label is Python's
/// `TypeError` (`<` between `NoneType` and `str`); a lone `None` sorts as itself.
fn sorted_masked(cells: &[SplitCell]) -> Vec<Option<Authority>> {
    let mut v: Vec<Option<Authority>> = Vec::new();
    for c in cells {
        if !v.contains(&c.masked) {
            v.push(c.masked);
        }
    }
    if v.len() > 1 && v.contains(&None) {
        panic!("rung-80: sorting masked labels with a None among them -- Python's TypeError");
    }
    v.sort_by_key(|x| x.map(|a| a.as_str()));
    v
}

/// `max(abs(c["cyc_fwd"]), abs(c["cyc_rev"]))` — Python's two-argument `max`, expression-first.
fn cyc_abs(c: &SplitCell) -> f64 {
    py_max2(c.cyc_fwd.abs(), c.cyc_rev.abs())
}

/// § 5 (`engine.py:21716`) — **does a LEVEL split move `c1`?** Or, with no interior point,
/// VACUOUS and nothing scored.
///
/// Named `split_wall::split_gains` at every call: rung 70's `split_gains` is
/// [`crate::cross_split::split_gains`], an INCOMPATIBLE reuse of the name (§ 5.33 (iii) B).
///
/// # THE GAINS GO THROUGH THE TABLE, UNDER `ShareScope("max")`
///
/// `m._with_share("max", m._quad_gains_at, flight, p, None, surge, Tt4_max)` (`engine.py:21742`)
/// with `_quad_gains_at`'s defaults `(1e-7, 1e-5, 1e-4, True, 4.0)` — rung 72's `shared_gains`
/// call, verbatim. An `Abort` from the gains chain propagates: Python does not catch it here.
///
/// # `rate` IS A NAIVE LEFT FOLD
///
/// `sum(1.0 / t for t in tt)` (`engine.py:21750`) — four terms from an integer `0`, the PyPy 3.11
/// builtin, which does not compensate. Ported as the four precedents at rungs 72/73/75/76 port it.
#[allow(clippy::too_many_arguments)]
pub fn split_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, phi_airs: &[Option<f64>], coord: &'static str, taus: (f64, f64, f64, f64),
    inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64, every: usize,
) -> Result<SplitGains, Abort> {
    let mut out: Vec<GainsArm> = Vec::new();
    for &pa in phi_airs {
        let (m, surge, lag, traj) = split_march(
            core, flight, tt4_lo, tt4_hi, tt4_max, phi_lim, pa, coord, taus, r, s_settle, ds,
            v_max, inc);
        let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
        let pts = riding4(&traj, b_max);
        let sampled: Vec<&FuelPoint> = pts.iter().step_by(every).collect();
        let mut cells: Vec<SplitCell> = Vec::new();
        let (mut sk_switch, mut sk_regime) = (0usize, 0usize);
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
            let (rf, gfv) = match p.extra {
                PointExtra::Shared { required_fuel, g_fuel, .. }
                | PointExtra::Demand { required_fuel, g_fuel, .. } => (required_fuel, g_fuel),
                _ => unreachable!("`riding4` admits only six-state points"),
            };
            let tau_f = lag.expect("the rig arms the fuel leg, so it carries the lag").tau(rf, gfv);
            let tt = (tau_f, taus.1, taus.2, taus.3);
            let coef = charpoly4(&jac4(&gg, tt));
            let roots = quartic_roots_c(&coef);
            let rate = 1.0 / tt.0 + 1.0 / tt.1 + 1.0 / tt.2 + 1.0 / tt.3;
            cells.push(SplitCell {
                s: p.s,
                phi: p.phi_lp,
                authority: gg.authority,
                masked: gg.masked,
                mask_leak: gg.mask_leak,
                zeros: roots.iter().filter(|z| z.abs() < 1e-4 * rate).count(),
                c1: coef[3],
                c0: coef[4],
                cyc_fwd: gg.f_q * gg.c_v * gg.v_f,
                cyc_rev: gg.f_v * gg.v_q * gg.c_f,
                pair_rc: gg.pair_rc,
                pair_cv: gg.pair_cv,
            });
        }
        let mut auth: Vec<(Option<Authority>, usize)> = Vec::new();
        for c in &cells {
            match auth.iter_mut().find(|(a, _)| *a == c.authority) {
                Some(e) => e.1 += 1,
                None => auth.push((c.authority, 1)),
            }
        }
        let mut zeros: Vec<usize> = cells.iter().map(|c| c.zeros).collect();
        zeros.sort_unstable();
        zeros.dedup();
        let cycs: Vec<Option<f64>> = cells.iter().map(|c| Some(cyc_abs(c))).collect();
        let leaks: Vec<Option<f64>> = cells.iter().map(|c| c.mask_leak).collect();
        out.push(GainsArm {
            phi_air: pa,
            n_riding: pts.len(),
            n_sampled: sampled.len(),
            n_interior: cells.len(),
            skipped: (sk_switch, sk_regime),
            authority: auth,
            masked: sorted_masked(&cells),
            max_mask_leak: py_max_opt(&leaks),
            zeros,
            max_cyc: py_max_opt(&cycs),
            cells,
        });
    }
    let cells_all: Vec<&SplitCell> = out.iter().flat_map(|a| a.cells.iter()).collect();
    let ctrl: Vec<Option<f64>> = cells_all.iter()
        .filter(|c| c.masked == Some(Authority::Gov))
        .map(|c| Some(cyc_abs(c)))
        .collect();
    let leaks_all: Vec<Option<f64>> = cells_all.iter().map(|c| c.mask_leak).collect();
    Ok(SplitGains {
        phi_lim,
        coord,
        taus,
        ds,
        vacuous: out.iter().any(|a| a.n_interior == 0),
        n_interior: out.iter().map(|a| a.n_interior).collect(),
        all_differenced: out.iter().all(|a| a.skipped == (0, 0)),
        ever_two_authorities: cells_all.iter()
            .any(|c| !matches!(c.authority, Some(Authority::Fuel) | Some(Authority::Gov))),
        control_nonzero: py_max_opt(&ctrl),
        max_mask_leak: py_max_opt(&leaks_all),
        arms: out,
    })
}
