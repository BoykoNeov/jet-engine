//! SLICE AA step 5 — **THE NINE CELLS, ONE MANUFACTURED BUG EACH.**
//!
//! `slice_aa_oracle.rs` is green at **12 084 keys** against two interpreters. A green oracle
//! leaves exactly one question open, and it is the question a hook table exists to raise: *which
//! defects would it catch?* **No value key can witness a table.** Swap a cell for a body that
//! computes the same number a different way and every key still passes; swap it for one that
//! computes a different number and the oracle goes red — but only if the cell is actually
//! REACHED, and *"the port dispatches through this name"* is precisely what a value dump cannot
//! assert.
//!
//! Rung 68 adds **nine** cells, the widest step in the port, so this file is the slice's signature
//! instrument rather than an extra. Each test corrupts ONE cell in a table built for the purpose,
//! runs a reader that should see it, and asserts a named quantity MOVES BY A MEASURED AMOUNT.
//!
//! # WHY "IT MOVED" IS NOT ENOUGH, AND EVERY ASSERTION HERE NAMES A DELTA
//!
//! [[rust-port-slice-w-step5]]: *a "did it move" assertion passes a HALF-APPLIED injection.* An
//! injection that fires on one of two call sites moves the answer, and a bare `!=` calls that a
//! pass. So each gate below asserts the direction and the rough magnitude of the change, and the
//! two whose observable is a PANIC assert the panic's own message.
//!
//! # P2, SETTLED PER CELL
//!
//! § 5.25's P2 read: *all nine cells are observable; a cell that cannot be broken is reported as
//! UNOBSERVABLE rather than quietly gated on something else.* The verdict is in
//! [`the_nine_cells_are_all_observable`], which re-runs every injection and tallies them, so the
//! count is EMITTED and not typed — this phase has been caught five times on a tally written from
//! memory beside the addends that disprove it.

use std::panic::catch_unwind;

use turbojet::bleed_transient::{LeverArm, LeverArming};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{
    riding, triple_bill, triple_gains_at, Census68, StatorLegArm, StatorLimiter, TripleGains,
    TripleHooks, TripleLaws, TripleRigArm, R68, R68_FUEL, R68_STATOR, R68_TRIPLE, R68_TWO,
};
use turbojet::two_lag::violation;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{ForcedStator, MarchedStator, TwoSpoolTransientCore};

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn arm() -> LeverArm {
    LeverArm { bleed_lim: Some(BleedLimiter::with_tau(PHI, B, Some(TAU))),
               stator_lim: Some(StatorLimiter::new(PHI, V_MAX, Some(TAU_S))),
               ..Default::default() }
}

/// A rung-68 machine carrying `hooks` as its TRIPLE table and rung 68's four others.
fn machine(hooks: &'static TripleHooks) -> ScheduledStatorCore {
    let a = arm();
    match ScheduledStatorTransient::with_triple_tables(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, a.stator,
        &R68_TWO, &R68_STATOR, &R68_FUEL, &R68,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        hooks, a.stator_lim)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

fn march_of(m: &ScheduledStatorCore, ds: f64, scope: MarchScope) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>,
                          surge: Some(Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM))),
                          tt4_max: None };
    m.stator_march_scoped(&flight(), &ramp(ds), None, &leg,
                          &MarchScope { lag: Some(AsymmetricLag::new(0.05, 0.15)), ..scope }).0
}

/// The three numbers every injection below is scored on: the violation integral, the deepest
/// stator excursion, and how many points the march carried a FIVE-state record at all.
fn reading(hooks: &'static TripleHooks) -> (f64, f64, usize) {
    let t = march_of(&machine(hooks), DS, MarchScope::DEFAULT);
    let v = |p: &FuelPoint| match p.extra { PointExtra::Triple { v, .. } => v, _ => f64::NAN };
    let five = t.iter().filter(|p| matches!(p.extra, PointExtra::Triple { .. })).count();
    let vmin = t.iter().map(v).filter(|x| x.is_finite())
        .fold(f64::INFINITY, f64::min);
    (violation(&t, PHI, R), if vmin.is_finite() { vmin } else { 0.0 }, five)
}

fn message_of<F: FnOnce() + std::panic::UnwindSafe>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f);
    std::panic::set_hook(prev);
    match out {
        Ok(()) => String::new(),
        Err(e) => e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    }
}

// ============================================================================== the nine bugs
//
// Each is the SHIPPED body with one thing changed, and the change is chosen to be the mistake a
// porter would plausibly make rather than an arbitrary corruption.

/// 1. `stator_leg` — *the machine has no third loop*. The mistake: reading the wrong field, or
///    `NO_TRIPLE`'s tempting `None` default.
fn bug_stator_leg(_: &TwoSpoolTransientCore) -> Option<StatorLegArm> { None }
static BUG_STATOR_LEG: TripleHooks =
    TripleHooks { stator_leg: bug_stator_leg, ..R68_TRIPLE };

/// 2. `lagged_stator` — *the clock is absent*. The mistake: `is_some()` on the limiter instead of
///    on its `tau`, inverted.
fn bug_lagged_stator(_: &TwoSpoolTransientCore) -> bool { false }
static BUG_LAGGED_STATOR: TripleHooks =
    TripleHooks { lagged_stator: bug_lagged_stator, ..R68_TRIPLE };

/// 3. `clamp_v` — **RUNG 69's BAND, one slice early.** The mistake is the one Python's docstring
///    warns about in capitals: `phi` is DECREASING in `v` and `M_i` INCREASING, so the open side
///    flips, and a port that wrote `max(0, min(v_max, v))` here clamps every riding setting to
///    zero **with nothing raising**.
fn bug_clamp_v(_: &TwoSpoolTransientCore, v: f64, lim_s: &StatorLegArm) -> f64 {
    0.0f64.max(lim_s.v_max.min(v))
}
static BUG_CLAMP_V: TripleHooks = TripleHooks { clamp_v: bug_clamp_v, ..R68_TRIPLE };

/// 4. `check_v0` — *the band is not checked*. Observable only through the refusal.
fn bug_check_v0(_: &TwoSpoolTransientCore, _: f64, _: &StatorLegArm) {}
static BUG_CHECK_V0: TripleHooks = TripleHooks { check_v0: bug_check_v0, ..R68_TRIPLE };

/// 5. `rk4_floor` — **rung 66's constant**, which is the mistake this cell exists to make
///    measurable: `2.0` over the two-clock rate instead of over all three.
fn bug_rk4_floor(_: f64, _: f64, _: usize, _: f64) {}
static BUG_RK4_FLOOR: TripleHooks = TripleHooks { rk4_floor: bug_rk4_floor, ..R68_TRIPLE };

/// 6. `solve_v` — *the loop never rides*. The mistake: `_solve_b`'s orientation copied without
///    inverting the clamps, which returns DORMANT wherever the real law rides.
fn bug_solve_v(
    _: &TwoSpoolTransientCore,
    closer: &dyn Fn(f64) -> Result<turbojet::fuel_transient::FuelCloseState, Abort>,
) -> Result<(turbojet::fuel_transient::FuelCloseState, f64, Regime), Abort> {
    Ok((closer(0.0)?, 0.0, Regime::Dormant))
}
static BUG_SOLVE_V: TripleHooks = TripleHooks { solve_v: bug_solve_v, ..R68_TRIPLE };

/// 7. `manifold_v` — *difference at the marched `v` instead of at the shared root*. The mistake:
///    rung 66's own choice carried forward, which is OFF the manifold during a transient.
#[allow(clippy::too_many_arguments)]
fn bug_manifold_v(
    _: &ScheduledStatorCore, _: &FlightCondition, _: f64, _: f64, _: f64, _: f64, _: f64,
    _: &dyn Fn(f64, f64) -> Result<(f64, Regime), Abort>,
) -> Result<f64, Abort> {
    Ok(0.0)
}
static BUG_MANIFOLD_V: TripleHooks = TripleHooks { manifold_v: bug_manifold_v, ..R68_TRIPLE };

/// 8. `triple_laws` — **the `b_state` / `v_state` boundary, crossed.** The mistake Python calls
///    rung 62's `_powers` trap in its fourth shape: the VALVE law shown the plant as the valve
///    already is. It converges a solver on a residual the plant never uses, and nothing raises.
fn bug_triple_laws<'a>(
    core: &'a ScheduledStatorCore, flight: &'a FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&'a AccelSchedule>, surge: Option<&'a Floor>,
) -> Result<TripleLaws<'a>, Abort> {
    // The SHIPPED laws, taken at a perturbed scheduled fuel — a stand-in for any body that solves
    // the right equation against the wrong plant state.
    (R68_TRIPLE.triple_laws)(core, flight, a, h, mf_sched * (1.0 + 1e-6), accel, surge)
}
static BUG_TRIPLE_LAWS: TripleHooks = TripleHooks { triple_laws: bug_triple_laws, ..R68_TRIPLE };

/// 9. `triple_rig` — *a ledger cell built with the wrong authority*. The mistake: `v_max`
///    inherited from a neighbouring reader instead of taken from the argument, which makes two
///    cells of the ledger incomparable while every one of them still looks like a number.
fn bug_triple_rig(
    core: &ScheduledStatorCore, a: &TripleRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    (R68_TRIPLE.triple_rig)(core, &TripleRigArm { v_max: 0.02, ..*a })
}
static BUG_TRIPLE_RIG: TripleHooks = TripleHooks { triple_rig: bug_triple_rig, ..R68_TRIPLE };

// ============================================================================== the gates

/// **THE BASELINE, and it is asserted rather than assumed.** Every gate below is a difference
/// against these three numbers, so a baseline that had itself drifted would make nine gates
/// report nine wrong deltas and all nine still pass.
fn baseline() -> (f64, f64, usize) {
    let r = reading(&R68_TRIPLE);
    assert!(r.0 > 0.0, "the baseline march must VIOLATE the floor somewhere: {}", r.0);
    assert!(r.1 < -0.01, "...and the stator must RIDE, not sit at its dormant stop: {}", r.1);
    assert_eq!(r.2, 341, "...on the five-state integrator");
    r
}

/// **THE FIRST DRAFT OF THIS GATE PREDICTED THE WRONG OBSERVABLE, AND THE RIGHT ONE IS MORE
/// INTERESTING.** It expected the injected `stator_leg` to send the march down the REDUCE, on the
/// reasoning that a machine with no leg has no third loop.
///
/// It does not, because **`lagged_stator` does not go through `stator_leg`** — Python is
/// `self.stator_lim is not None and self.stator_lim.tau is not None`, reading the field directly.
/// So the dispatch still enters the five-state integrator, whose first line is
/// `lim_s = self._stator_leg()`, and Python then evaluates `lim_s.tau` on `None`: an
/// `AttributeError`, which nothing in the ladder catches. **The port's `expect` IS that
/// AttributeError**, and the panic message is the observable.
///
/// Recorded rather than quietly re-aimed, because the wrong prediction is the finding: the two
/// cells are NOT two spellings of one question, and a port that routed `lagged_stator` through
/// `stator_leg` — the tidier arrangement — would be a different program.
#[test]
fn cell_1_stator_leg_is_reached() {
    baseline();
    let msg = message_of(|| { reading(&BUG_STATOR_LEG); });
    assert!(msg.contains("march with no stator floor"),
            "the injected cell must be REACHED by the integrator's first line; got {msg:?}");
}

#[test]
fn cell_2_lagged_stator_is_reached() {
    let base = baseline();
    let got = reading(&BUG_LAGGED_STATOR);
    assert_eq!(got.2, 0, "`lagged_stator = false` is the reduce, by dispatch");
    assert!(got.0 > 1.5 * base.0, "{} vs {}", got.0, base.0);
}

#[test]
fn cell_3_clamp_v_is_reached_and_the_band_is_one_sided() {
    let base = baseline();
    let got = reading(&BUG_CLAMP_V);
    // Rung 69's band clamps every negative setting to zero, so the loop is present, marches, and
    // delivers NOTHING — the silent failure the cell's docstring names.
    assert_eq!(got.2, 341, "the march still runs -- that is the point");
    assert_eq!(got.1, 0.0, "...with the stator pinned at its dormant stop by the wrong band");
    assert!(got.0 > 1.5 * base.0,
            "...and the protection gone: {} vs {}", got.0, base.0);
}

#[test]
fn cell_4_check_v0_is_reached() {
    // The band check is the ONLY observable, so the gate is the refusal itself.
    let good = message_of(|| {
        march_of(&machine(&R68_TRIPLE), DS,
                 MarchScope { v0: Some(0.05), ..MarchScope::DEFAULT });
    });
    assert!(good.contains("stator POSITION"),
            "the shipped cell must refuse an out-of-band v0; got {good:?}");
    let bug = message_of(|| {
        march_of(&machine(&BUG_CHECK_V0), DS,
                 MarchScope { v0: Some(0.05), ..MarchScope::DEFAULT });
    });
    assert!(!bug.contains("stator POSITION"),
            "with the check dispatched away the march must proceed; got {bug:?}");
}

#[test]
fn cell_5_rk4_floor_is_reached() {
    let good = message_of(|| { march_of(&machine(&R68_TRIPLE), 0.04, MarchScope::DEFAULT); });
    assert!(good.contains("RATES ADD"), "the shipped floor must fire at ds = 0.04: {good:?}");
    let t = march_of(&machine(&BUG_RK4_FLOOR), 0.05, MarchScope::DEFAULT);
    // AND THE COUNTERFEIT, which is the reason the cell exists: it does not blow up.
    assert_eq!(violation(&t, PHI, R), 0.0, "the counterfeit: no violation at all");
    let min_phi = t.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min);
    assert!((min_phi - PHI).abs() < 1e-9, "...and the floor exactly held: {min_phi}");
}

#[test]
fn cell_6_solve_v_is_reached() {
    let base = baseline();
    let got = reading(&BUG_SOLVE_V);
    assert_eq!(got.2, 341, "the march still runs on five states");
    assert_eq!(got.1, 0.0, "...but the command is always the dormant stop");
    assert!(got.0 > 1.5 * base.0, "{} vs {}", got.0, base.0);
}

/// The gains at one riding point of the INJECTED machine's own march.
///
/// **NOT THROUGH `triple_gains`, AND THAT IS THIS FILE'S SHARPEST FINDING.** The first draft of
/// gates 7 and 8 called `triple_gains`, which opens by building a SIBLING through `triple_rig`
/// -> `at_lever` — and `at_lever` installs the SHIPPED tables. So both injections were applied
/// to a machine the reader then threw away, and both gates reported *"nothing moved"* while
/// asserting nothing at all. **That is faithful to Python** (its `at_lever` constructs a fresh
/// `ThreeLoopCascadeTransient`, whose methods are the class's), which is exactly why it is worth
/// a paragraph: *any* reader in this rung that goes through `_triple_rig` runs on the DEFAULT
/// bodies, so an injection into `manifold_v` or `triple_laws` is invisible to `triple_gains`,
/// `triple_modes`, `cyclic_sensitivity` and `saturation_counterfeit` alike.
///
/// [[rust-port-slice-w-step3]] in its own shape: five of six injections passed every gate because
/// the probe could not see them. This instrument is built to prove it CAN.
fn gains_on(hooks: &'static TripleHooks) -> TripleGains {
    let m = machine(hooks);
    let t = march_of(&m, DS, MarchScope::DEFAULT);
    let ride = riding(&t, B);
    assert!(!ride.is_empty(), "the injected machine must still produce riding points");
    let p = ride[ride.len() / 2];
    let surge = Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM));
    triple_gains_at(&m, &flight(), &p, None, Some(&surge), 1e-7, 1e-5, 1e-4, true, 0.0, true)
        .expect("the gains march does not abort")
}

#[test]
fn cell_7_manifold_v_is_reached() {
    let ok = gains_on(&R68_TRIPLE);
    let bug = gains_on(&BUG_MANIFOLD_V);
    // The DIRECT reading: the base point itself.
    assert!(ok.v_base < -0.01, "the shipped manifold sits where the stator's own root is: {}",
            ok.v_base);
    assert_eq!(bug.v_base, 0.0, "the injected one reports the dormant stop");
    // ...and the identity s 2 is about DEGRADES there, which is what makes the base point
    // load-bearing rather than cosmetic.
    assert!((ok.cyclic + 1.0).abs() < 1e-6, "on the manifold: {}", ok.cyclic);
    assert!((bug.cyclic + 1.0).abs() > 1e-3,
            "off it the cyclic product must MOVE: {} (baseline {})", bug.cyclic, ok.cyclic);
}

#[test]
fn cell_8_triple_laws_is_reached() {
    let ok = gains_on(&R68_TRIPLE);
    let bug = gains_on(&BUG_TRIPLE_LAWS);
    // The laws are evaluated against a plant one part in 1e6 away, so ALL SIX gains move.
    // Asserting six by name rather than one is [[rust-port-slice-w-step5]]'s rule: a
    // HALF-APPLIED injection moves one, and a bare `!=` calls that a pass.
    for (name, a, b) in [("C_g", ok.c_g, bug.c_g), ("V_g", ok.v_g, bug.v_g),
                         ("R_q", ok.r_q, bug.r_q), ("C_v", ok.c_v, bug.c_v),
                         ("V_q", ok.v_q, bug.v_q), ("R_v", ok.r_v, bug.r_v)] {
        assert!(a != b, "{name} must move: {a} vs {b}");
    }
}

#[test]
fn cell_9_triple_rig_is_reached() {
    let m_ok = machine(&R68_TRIPLE);
    let m_bug = machine(&BUG_TRIPLE_RIG);
    let arm = TripleRigArm { sm: SM, ..TripleRigArm::default() };
    let b_ok = triple_bill(&m_ok, &flight(), &ramp(DS), SM, &arm);
    let b_bug = triple_bill(&m_bug, &flight(), &ramp(DS), SM, &arm);
    // The `S` cell is the stator ALONE, which is where a strangled authority bites hardest.
    assert!(b_ok.cell("S").credit > 80.0, "{}", b_ok.cell("S").credit);
    assert!(b_bug.cell("S").credit < b_ok.cell("S").credit - 5.0,
            "a rig built at v_max = 0.02 must deliver measurably less: {} vs {}",
            b_bug.cell("S").credit, b_ok.cell("S").credit);
    assert!(b_bug.cell("S").v_saturated && !b_ok.cell("S").v_saturated,
            "...and it must SATURATE, which the shipped one never does");
}

/// **THE CENSUS, READ — because an instrument nobody reads is dead weight**, and this file is the
/// only place in the slice that can read it.
///
/// [`Census68`] counts three things the value oracle reaches only indirectly and one it cannot
/// reach at all:
///
/// * **`arm`'s FOUR ARMS.** `r68_arm` returns early when there is no leg and again when neither
///   `v_forced` nor `v_state` is set, and otherwise writes the design map or a `with_vsv` one. The
///   two early returns leave the map exactly where they found it, so a port that collapsed them
///   would agree on every key; the counters are what say all four are LIVE.
/// * **THE REDUCE.** `integrate_reduced` fires exactly once per reduced march — the dispatch that
///   makes a rung-68 machine with no stator every one of its ancestors.
/// * **THE THREE REGIMES.** `solve_v` returns the label and the ladder then reads it only through
///   `_riding`'s filter, so the DISTRIBUTION is a claim about the plant that no single key carries.
///   The rung's own machine never saturates and the `v_max = 0.02` one does, which is what makes
///   gate 6's confound reachable at all.
#[test]
fn the_census_shows_every_arm_of_arm_and_both_ends_of_the_regime() {
    Census68::reset();
    let t = march_of(&machine(&R68_TRIPLE), DS, MarchScope::DEFAULT);
    assert_eq!(t.len(), 341);
    let c = Census68::read();
    assert!(c.solve_v_calls > 1_000, "the stator law runs four times per RK4 step: {}",
            c.solve_v_calls);
    assert_eq!(c.regime_dormant + c.regime_riding + c.regime_saturated, c.solve_v_calls,
               "every `solve_v` call must land on exactly one of the three regimes");
    assert!(c.regime_riding > 0 && c.regime_dormant > 0,
            "the rung's own machine must visit BOTH reachable regimes: {c:?}");
    assert_eq!(c.regime_saturated, 0,
               "...and never the third -- gate 5's `v_max` inertness depends on it");
    // `arm`'s arms. The two that write a map and the one that does not are all live.
    assert!(c.arm_moved > 0, "a live limiter position must reach the LP map: {c:?}");
    assert!(c.arm_no_position > 0,
            "and the STEADY solves inside the march must leave it alone: {c:?}");
    assert_eq!(c.integrate_reduced, 0, "an armed march is not a reduce");

    // **AND `v_of`'s LIVE ARM IS DEAD ON THIS GRID -- 0 of the whole march, MEASURED.** This
    // assertion was first written as `> 0` and it FAILED, which is the finding: nothing inside the
    // five-state integrator reads the stator setting through `v_of`. The march reaches the moved
    // map through `arm`, and every `v_of` caller in the family is a rung-57..60 READER that runs
    // OUTSIDE a march, where neither carrier is set and the override hands straight back to the
    // parent. Slice X's precedent exactly (`b_of`'s `b_state` override, 0 of 1 705 at rung 64,
    // shipped anyway because a port that drops it breaks one slice later) -- so the zero is
    // recorded and the branch is gated BY HAND below, because no value key can reach it.
    assert_eq!(c.v_of_live, 0,
               "`v_of`'s live arm is dead on the shipped grid; if this ever fires, a reader \
                started calling it from inside a march and the branch is no longer manufactured");

    // THE REDUCE, counted: exactly one dispatch, and the five-state integrator never entered.
    Census68::reset();
    let red = march_of(&machine(&BUG_LAGGED_STATOR), DS, MarchScope::DEFAULT);
    let c = Census68::read();
    assert_eq!(c.integrate_reduced, 1, "the reduce dispatches ONCE per march: {c:?}");
    assert_eq!(c.solve_v_calls, 0, "...and the stator law is never reached: {c:?}");
    assert!(!red.is_empty());

    // THE THIRD REGIME, on the machine built to reach it.
    Census68::reset();
    let a = arm();
    let sat = match ScheduledStatorTransient::with_triple_tables(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, a.stator,
        &R68_TWO, &R68_STATOR, &R68_FUEL, &R68,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        &R68_TRIPLE, Some(StatorLimiter::new(PHI, 0.02, Some(TAU_S))))
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    march_of(&sat, DS, MarchScope::DEFAULT);
    let c = Census68::read();
    assert!(c.regime_saturated > 0,
            "a strangled authority must SATURATE -- otherwise gate 6 measures nothing: {c:?}");
    assert!(c.regime_riding > 0, "...while still riding somewhere: {c:?}");

    // **THE DEAD ARM, MANUFACTURED.** `v_of` prefers `v_forced` over `v_state` for rung 65's
    // reason one lever over: the stator's own command solve trials settings on a plant whose live
    // setting is the one being commanded away from. Both carriers and the precedence are pinned
    // here because nothing on the shipped grid reaches any of it.
    Census68::reset();
    let m = machine(&R68_TRIPLE);
    let core = &m.fuel.inner;
    let parent = m.v_of(Spool::Lp, 0.9, 0.9, None);
    assert_eq!(parent, 0.0, "outside a march the override hands back rung 57's answer");
    {
        let _st = MarchedStator::set(core, -0.07);
        assert_eq!(m.v_of(Spool::Lp, 0.9, 0.9, None), -0.07, "the marched position wins");
        let _fo = ForcedStator::set(core, -0.03);
        assert_eq!(m.v_of(Spool::Lp, 0.9, 0.9, None), -0.03,
                   "...and a TRIAL setting wins over it");
        // The HP spool is not this loop's, at any setting.
        assert_eq!(m.v_of(Spool::Hp, 0.9, 0.9, None), 0.0,
                   "rung 68's floor watches the LP and only the LP");
    }
    assert_eq!(m.v_of(Spool::Lp, 0.9, 0.9, None), parent, "and both guards restore");
    // **1 + 1 + 0 = 2, WRITTEN AS A SUM BECAUSE I TYPED 3.** The sentence beside this assertion
    // already said "the two LP ones inside the guards" and the number next to it said three. The
    // HP read cannot bump the counter at all: the override tests `spool == Lp` BEFORE it looks at
    // either carrier. That is this phase's most-repeated defect -- a tally remembered beside the
    // addends that disprove it -- and it is left visible rather than quietly corrected.
    let live = Census68::read().v_of_live;
    assert_eq!(live, 1 + 1 + 0,
               "the live arm is taken by the two LP reads inside the guards and by NOTHING else: \
                not the LP read outside them (no carrier set), and not the HP one at any setting \
                (rung 68's floor watches the LP and only the LP). Got {live}");
}

/// **P2's LEDGER, EMITTED.** Nine injections, nine cells; the count is re-derived here rather than
/// typed, because this phase has been caught five times on a tally written from memory beside the
/// addends that disprove it.
#[test]
fn the_nine_cells_are_all_observable() {
    let base = baseline();
    let march_bugs: [(&str, &'static TripleHooks); 5] = [
        ("stator_leg", &BUG_STATOR_LEG),
        ("lagged_stator", &BUG_LAGGED_STATOR),
        ("clamp_v", &BUG_CLAMP_V),
        ("solve_v", &BUG_SOLVE_V),
        ("rk4_floor", &BUG_RK4_FLOOR),
    ];
    let mut moved: Vec<&str> = Vec::new();
    for (name, h) in march_bugs {
        let got = if name == "rk4_floor" {
            let t = march_of(&machine(h), 0.05, MarchScope::DEFAULT);
            (violation(&t, PHI, R), 0.0, t.len())
        } else if name == "stator_leg" {
            // Its observable is a PANIC, not a number — see the gate.
            let msg = message_of(|| { reading(h); });
            assert!(msg.contains("march with no stator floor"));
            (f64::NAN, 0.0, usize::MAX)
        } else {
            reading(h)
        };
        if got.0 != base.0 || got.2 != base.2 {
            moved.push(name);
        }
    }
    // `check_v0`'s observable is a refusal; `manifold_v`'s and `triple_laws`' are the gains taken
    // on the INJECTED machine, which no `triple_rig`-built sibling can carry; `triple_rig`'s is a
    // ledger cell. Each is asserted in its own gate above and re-derived here rather than assumed.
    let g_ok = gains_on(&R68_TRIPLE);
    if g_ok.v_base != gains_on(&BUG_MANIFOLD_V).v_base { moved.push("manifold_v"); }
    if g_ok.c_g != gains_on(&BUG_TRIPLE_LAWS).c_g { moved.push("triple_laws"); }
    if message_of(|| {
        march_of(&machine(&BUG_CHECK_V0), DS,
                 MarchScope { v0: Some(0.05), ..MarchScope::DEFAULT });
    }).is_empty() { moved.push("check_v0"); }
    let rig_arm = TripleRigArm { sm: SM, ..TripleRigArm::default() };
    let m_ok = machine(&R68_TRIPLE);
    let m_bug = machine(&BUG_TRIPLE_RIG);
    if triple_bill(&m_ok, &flight(), &ramp(DS), SM, &rig_arm).cell("S").credit
        != triple_bill(&m_bug, &flight(), &ramp(DS), SM, &rig_arm).cell("S").credit {
        moved.push("triple_rig");
    }
    assert_eq!(moved.len(), 9,
               "P2 says every one of the nine cells is observable; {} were: {:?}",
               moved.len(), moved);
    println!("slice_aa_dispatch: {}/9 cells observable -- {:?}", moved.len(), moved);
}
