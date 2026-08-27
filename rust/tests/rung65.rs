//! RUNG 65 — the LAGGED BLEED VALVE: what a finite bandwidth costs, and what it gives back.
//!
//! Slice Y step 3. Python's `tests/test_rung65.py`, all 21 collected gates, plus **one the Python
//! suite does not have**: § 5.23 (vii)'s constructor chain, gated assert-by-assert.
//!
//! # The ADDED gate, and why a translation would not have needed it
//!
//! Python's `LaggedBleedTransient` has no `__init__` at all — it inherits rung 64's and flips a
//! class constant, `_LAG_OK`, that rung 64's last assert reads. Rust has no class constants on a
//! `fn`-pointer ladder, so [`build_lagged_bleed`] **re-spells the chain** with that one assert
//! satisfied instead of refused. Probe 5 measured the chain at **ten** asserts; re-spelling can
//! silently drop any of the other nine and **every value key stays green**, because the machines
//! the suite builds are all legal. So [`the_constructor_chain_is_ten_asserts_and_only_one_is_relaxed`]
//! exercises each one individually and pins the COUNT — a gate the Python suite has no reason to
//! own, and the port cannot do without.
//!
//! # The `slow` bill, measured here rather than deferred again
//!
//! § 5.19 (viii) booked the phase's 263 `slow` gates as a measurement owed at the first
//! `slow`-heavy suite. At **9 of 21 (42.9 %)** this is it. Slice M's rule applies unchanged: port
//! the gate, DROP the marker, re-introduce `#[ignore]` only against a MEASURED Rust cost. The
//! whole-file cost is reported in the step-3 write-up beside PyPy's.

use std::panic::catch_unwind;

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::fuel_transient::{Floor, FuelPoint, PointExtra, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const N_LO: f64 = 0.65;
const B: f64 = 0.10;
/// Strictly inside `[0.7354 shut, 0.8095 fully open]`.
const PHI: f64 = 0.80;
/// The representative bandwidth — `ds/tau = 0.1` at `DS`.
const TAU: f64 = 0.05;
const TAUS: [f64; 6] = [0.4, 0.2, 0.1, 0.05, 0.02, 0.01];

fn sm() -> f64 { PHI / FLOOR - 1.0 }

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

/// Python's `_gt(...)` — a rung-65 machine.
fn gt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// A rung-64 machine on the SAME hardware — the reduce's reference.
fn lt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }

/// Python's `_march_keys`.
fn keys(t: &[FuelPoint]) -> Vec<(u64, u64, u64, u64, u64, u64, u64)> {
    t.iter()
        .map(|p| (p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits()))
        .collect()
}

fn b_of_point(p: &FuelPoint) -> (f64, f64) {
    match p.extra {
        PointExtra::Valve { b, b_cmd } => (b, b_cmd),
        _ => panic!("not a lagged-valve point"),
    }
}

fn panics<F: FnOnce() + std::panic::UnwindSafe>(f: F) -> bool {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f).is_err();
    std::panic::set_hook(prev);
    out
}

// =============================================================================================
// GATE 1 — THE REDUCE. Three arms, and the third is deliberately NOT an equality.
// =============================================================================================

/// The whole rung is a subclass, so rung 64's class is LITERALLY untouched. An unlagged rung-65
/// machine must march identically to the rung-64 one on the same hardware, under EVERY arming
/// mode — otherwise the bandwidth sweep would be comparing two code paths.
#[test]
fn reduce_no_lag_is_rung64_bit_for_bit() {
    for arm in [LeverArm::default(), LeverArm::constant(B),
                LeverArm::scheduled(BleedSchedule::new(B, N_LO)),
                LeverArm::floored(valve(None))] {
        let (a, _) = gt(&arm).stator_march(&flight(), &ramp(0.01), None, &StatorLeg::default());
        let (b, _) = lt(&arm).stator_march(&flight(), &ramp(0.01), None, &StatorLeg::default());
        assert_eq!(keys(&a), keys(&b), "{:?}", arm.keys());
    }
}

/// Rung 64's gate, re-run through the new tables: a floor below every `phi` on the march must
/// reach the rung-63 grandparent at every state, not merely agree to a tolerance.
#[test]
fn reduce_a_dormant_floor_still_dispatches_away_at_every_state() {
    let m = gt(&LeverArm::default());
    let (a, _) = m.at_lever(&LeverArm::floored(BleedLimiter::new(0.30, B)))
                  .stator_march(&flight(), &ramp(0.01), None, &StatorLeg::default());
    let (b, _) = m.at_lever(&LeverArm::default())
                  .stator_march(&flight(), &ramp(0.01), None, &StatorLeg::default());
    assert_eq!(keys(&a), keys(&b));
}

/// `b0` is an ISOLATION instrument (§ 3's continuum needs it). Passing it explicitly at the value
/// the march would have chosen must reproduce that march bit-for-bit, or the instrument is
/// perturbing the thing it measures.
#[test]
fn reduce_b0_none_is_the_physical_initial_condition_bit_for_bit() {
    let m = gt(&LeverArm::floored(valve(Some(TAU))));
    let (a, _) = m.stator_march(&flight(), &ramp(0.01), None, &StatorLeg::default());
    let (b0, cmd0) = b_of_point(&a[0]);
    let (b, _) = m.stator_march_scoped(&flight(), &ramp(0.01), None, &StatorLeg::default(),
                                       &MarchScope { b0: Some(b0), ..MarchScope::DEFAULT });
    assert_eq!(keys(&a), keys(&b));
    assert_eq!(b0, cmd0, "b(0) must be the EQUILIBRIUM command (§ 0, probe A)");
    assert!(b0 > 0.0,
            "§ 0 probe A: the limiter RIDES at s = 0 on this grid, which is precisely why \
             b(0) = 0 would inject a startup transient into the binding early-ramp LP minimum.");
}

/// Rung 65 adds only a transient subclass and its readers. The default single-spool design run
/// must be bit-for-bit rung 6 (the project's spine).
#[test]
fn cycle_untouched_design_run_is_rung6_bit_for_bit() {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_LPC * PI_HPC, TT4, 50_000.0,
                             Losses { pi_d: 0.97, eta_b: 0.99, pi_b: 0.96, eta_m: 0.99,
                                      pi_n: 0.98, ..Losses::default() });
    let res = eng.run(&flight(), 1.0);
    let reference = eng.run(&flight(), 1.0);
    assert!(res.performance.specific_thrust > 0.0 && res.performance.tsfc > 0.0);
    for s in ["2", "3", "4", "5", "9"] {
        assert!(res.station(s).tt.to_bits() == reference.station(s).tt.to_bits());
        assert!(res.station(s).pt.to_bits() == reference.station(s).pt.to_bits());
    }
    assert!(res.performance.specific_thrust.to_bits()
            == reference.performance.specific_thrust.to_bits());
}

// =============================================================================================
// GATE 2 — THE OBJECT. A lagged valve is a DIFFERENT object from an instantaneous one, and
//          from a cascade.
// =============================================================================================

#[test]
fn tau_zero_is_refused_the_instantaneous_valve_is_tau_none() {
    assert!(panics(|| { BleedLimiter::with_tau(PHI, B, Some(0.0)); }));
    assert!(panics(|| { BleedLimiter::with_tau(PHI, B, Some(-0.1)); }));
}

/// The whole rung is that the lag changes the plant's STRUCTURE. A rung-64 machine handed a
/// lagged limiter would silently march it instantaneously and report a bandwidth it never had —
/// so it refuses instead.
#[test]
fn rung64s_builder_refuses_a_lagged_limiter_rather_than_dropping_the_lag() {
    assert!(panics(|| { lt(&LeverArm::floored(valve(Some(TAU)))); }));
}

/// Rung 52's standing seam is a CASCADE, and rung 65 does not take it. A lagged valve beside a
/// lagged FUEL leg is four states and two clocks; rungs 50/51's forced edges are an instrument
/// for a leg that cannot pin its own trigger, which this one can.
#[test]
fn the_two_lag_cascade_and_the_forced_edges_are_refused() {
    use turbojet::fuel_transient::{AsymmetricLag, FuelLimiters};
    let fuel = SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm());
    let lag = AsymmetricLag::new(0.05, 0.2);
    let combos: Vec<FuelLimiters<'_>> = vec![
        FuelLimiters { lag: Some(lag), surge: Some(fuel), ..Default::default() },
        FuelLimiters { tau_gov: Some(0.05), tt4_max: Some(1450.0), ..Default::default() },
        FuelLimiters { s_off: Some(0.3), surge: Some(fuel), ..Default::default() },
        FuelLimiters { s_off: Some(0.3), tau_rel: Some(0.1), surge: Some(fuel),
                       ..Default::default() },
    ];
    for lim in &combos {
        let m = gt(&LeverArm::floored(valve(Some(TAU))));
        assert!(panics(std::panic::AssertUnwindSafe(|| {
            m.fuel.integrate_fuel(&flight(), |_s| 0.01, (0.75, 0.79), 0.05, 0.01, lim);
        })), "a rung-65 march must refuse this limiter set");
    }
}

/// Rung 62's two-way assert became rung 64's three-way; the lag rides on the limiter, so it must
/// not open a fourth back door.
#[test]
fn the_three_arming_modes_stay_mutually_exclusive() {
    assert!(panics(|| {
        gt(&LeverArm { bleed: B, bleed_lim: Some(valve(Some(TAU))), ..Default::default() });
    }));
    assert!(panics(|| {
        gt(&LeverArm { bleed_sched: Some(BleedSchedule::new(B, N_LO)),
                       bleed_lim: Some(valve(Some(TAU))), ..Default::default() });
    }));
}

// =============================================================================================
// GATE 3 — THE TRAP, fifth instance; and rung 64's re-solve comment CORRECTED
// =============================================================================================

/// Rungs 61/62/63/64 each hit the same trap: a sibling constructor that drops the newest lever
/// turns every inherited reader into an armed-vs-armed comparison. The lag rides on `bleed_lim`
/// precisely so there is no separate keyword to drop — this gate pins that.
///
/// **THE `isinstance` HALF IS PORTED AS FN-POINTER EQUALITY**, which is strictly stronger:
/// Python asks whether the sibling is a `LaggedBleedTransient`, and what actually matters is
/// whether it carries rung 65's CELLS. § 5.19 (vii)'s replacement for `test_rung71.py:190`,
/// applied here for the same reason.
#[test]
fn sibling_constructors_return_this_class_carrying_the_lag() {
    use turbojet::lagged_bleed::{R65, R65_FUEL, R65_STATOR, R65_TWO};
    let m = gt(&LeverArm::floored(valve(Some(TAU))));
    for sib in [m.at_lever(&LeverArm::floored(valve(Some(TAU)))),
                m.at_stator(StatorArm::default())] {
        // **CELL-BY-CELL, NOT `ptr::eq` ON THE TABLE.** A `const` is inlined at every use, so
        // `&R65` is a fresh promotion and pointer identity on the STRUCT proves nothing — it
        // failed here first time round. `fn_addr_eq` on the four cells rung 65 actually swaps is
        // both reliable and stronger: it says WHICH bodies the sibling carries.
        assert!(std::ptr::fn_addr_eq(sib.fuel.inner.lever_hooks.at_lever, R65.at_lever));
        assert!(std::ptr::fn_addr_eq(sib.fuel.inner.lever_hooks.b_at_point, R65.b_at_point));
        assert!(std::ptr::fn_addr_eq(sib.fuel.hooks.try_close_fuel, R65_FUEL.try_close_fuel));
        assert!(std::ptr::fn_addr_eq(sib.fuel.hooks.integrate_fuel, R65_FUEL.integrate_fuel));
        assert!(std::ptr::fn_addr_eq(sib.fuel.inner.stator_hooks.stator_march,
                                     R65_STATOR.stator_march));
        assert!(std::ptr::fn_addr_eq(sib.fuel.inner.hooks.try_close, R65_TWO.try_close));
        assert_eq!(sib.fuel.inner.lever.lim.and_then(|l| l.tau), Some(TAU));
    }
    // isolation still isolates
    assert!(m.at_lever(&LeverArm::default()).fuel.inner.lever.lim.is_none());
}

/// CORRECTS a rung-64 code comment. There the valve is a pure state function, so `b_at_point`
/// RE-SOLVES it. A lagged position carries history; re-solving it would hand back the COMMAND —
/// the one number that is not the valve.
#[test]
fn a_lagged_position_must_be_recorded_not_re_solved() {
    let m = gt(&LeverArm::floored(valve(Some(TAU))));
    let (traj, _) = m.stator_march(&flight(), &ramp(0.01), None, &StatorLeg::default());
    let mut p = &traj[0];
    for q in &traj[1..] {
        let (b, c) = b_of_point(q);
        let (pb, pc) = b_of_point(p);
        if (b - c).abs() > (pb - pc).abs() {
            p = q;
        }
    }
    let (b, c) = b_of_point(p);
    assert!((b - c).abs() > 1e-4, "need a point where the valve is genuinely behind");
    assert_eq!(m.b_at_point(&flight(), p), b);
    // …and a point from a DIFFERENT integrator must be refused, not reconstructed.
    let mut stripped = *p;
    stripped.extra = PointExtra::None;
    assert!(panics(std::panic::AssertUnwindSafe(|| { m.b_at_point(&flight(), &stripped); })),
            "a point that did not record the valve must raise, not re-solve");
}

/// Rung 62's `_powers` failure mode, on the new carriers: every one is set and restored around a
/// scope, so nothing may be left behind after a march.
#[test]
fn a_leaked_state_cannot_survive_a_march() {
    let m = gt(&LeverArm::floored(valve(Some(TAU))));
    m.stator_march(&flight(), &ramp(0.02), None, &StatorLeg::default());
    assert!(m.fuel.inner.b_state.get().is_none());
    assert!(m.fuel.inner.b_forced.get().is_none());
    assert!(m.fuel.inner.b0.get().is_none());
    // …and a SCOPED march must not leak either: `InitialBleed` restores the PREVIOUS value, and
    // outside any scope the previous value is `None`.
    m.stator_march_scoped(&flight(), &ramp(0.02), None, &StatorLeg::default(),
                          &MarchScope { b0: Some(0.03), ..MarchScope::DEFAULT });
    assert!(m.fuel.inner.b0.get().is_none());
}

// =============================================================================================
// GATE 4 — BANDWIDTH IS PURE LOSS (§ 1), and buys nothing at the STOP (§ 2)
// =============================================================================================

/// § 1. Both currencies monotone in `tau` and in the SAME direction: a slower valve protects LESS
/// and bleeds MORE. Rung 64's instantaneous law brackets the sweep from the good side on both —
/// it delivers its set point EXACTLY and pays the least bleed.
#[test]
fn bandwidth_is_pure_loss_on_both_axes() {
    let bc = gt(&LeverArm::default())
        .bandwidth_ceiling(&flight(), &ramp(DS), PHI, B, &TAUS);
    let rows = &bc.rows;
    assert!(rows.iter().all(|x| !x.saturated), "§ 1 is read on RIDING cells (see gate 5)");
    // the sweep is descending in tau, so BOTH must improve monotonically along it
    let under: Vec<f64> = rows.iter().map(|x| x.undershoot).collect();
    let bint: Vec<f64> = rows.iter().map(|x| x.b_int).collect();
    assert!((0..under.len() - 1).all(|i| under[i] < under[i + 1]), "{under:?}");
    assert!((0..bint.len() - 1).all(|i| bint[i] > bint[i + 1]), "{bint:?}");
    // rung 64 brackets it: exact set point, least bleed
    assert!((bc.inst_min_phi - PHI).abs() < 1e-9);
    assert!(rows.iter().all(|x| x.min_phi_lp < PHI - 1e-4));
    assert!(rows.iter().all(|x| x.b_int > bc.inst_b_int));
}

/// The SECOND arm of the reduce, and it is deliberately a limit and not an equality: a different
/// code path with a third state cannot be bit-for-bit. `dev` shrinks monotonically and its
/// consecutive-halving ratio approaches first order from below (it SATURATES at large `tau`,
/// being bounded by the valve-shut march's own deficit).
#[test]
fn the_tau_to_zero_arm_of_the_reduce_converges() {
    let bc = gt(&LeverArm::default())
        .bandwidth_ceiling(&flight(), &ramp(DS), PHI, B, &TAUS);
    let dev: Vec<f64> = bc.rows.iter().map(|x| x.dev).collect();
    assert!(bc.dev_shrinks && dev.iter().all(|&d| d > 0.0));
    assert!(dev[dev.len() - 1] < 0.25 * dev[0]);
    let r_small = dev[dev.len() - 2] / dev[dev.len() - 1];   // 0.02 -> 0.01
    let r_large = dev[0] / dev[1];                            // 0.4 -> 0.2, saturated end
    assert!(r_small > 1.6 && r_small < 2.4, "{r_small}");
    assert!(r_large < r_small, "{r_large} {r_small}");
}

/// § 2's closing leg. A floor above the fully-open march's own minimum SATURATES, and there the
/// protected coordinate is tau-INVARIANT: where the valve is against its stop, bandwidth is
/// exactly as powerless as law was (rung 64's headline, second axis). The bleed integral still
/// pays the pure-loss bill, so the two axes SPLIT.
#[test]
fn at_the_stop_bandwidth_buys_nothing_confirming_rung64() {
    let m = gt(&LeverArm::default());
    let over = m.at_lever(&LeverArm::constant(B)).bill_cell(&flight(), &ramp(DS), false)
                .min_phi_lp * 1.10;
    let reference = m.at_lever(&LeverArm::floored(BleedLimiter::new(over, B)))
                     .bill_cell(&flight(), &ramp(DS), false);
    assert!(reference.min_phi_lp < over, "rung 64's witness: an over-set floor is VIOLATED");
    let mut prev: Option<f64> = None;
    for tau in [0.01, 0.05, 0.2] {
        let c = m.at_lever(&LeverArm::floored(BleedLimiter::with_tau(over, B, Some(tau))))
                 .bill_cell(&flight(), &ramp(DS), false);
        assert!((c.b_peak - B).abs() <= 1e-12 * B, "the cell must be SATURATED");
        assert!((c.min_phi_lp - reference.min_phi_lp).abs() < 1e-9, "{tau}");
        assert!(c.b_int > reference.b_int);
        if let Some(p) = prev {
            assert!(c.b_int > p, "the bleed bill is still monotone in tau");
        }
        prev = Some(c.b_int);
    }
}

// =============================================================================================
// GATE 5 — RUNG 64 § 4's DESTROYED ARGMIN, RESTORED (§ 2)
// =============================================================================================

/// Rung 64 § 4: a RIDING floor pins `phi_lp` over an INTERVAL, so the argmin is a 1-ulp tie and
/// its location is not a result. A trailing actuator cannot pin what it has not caught up to.
/// Read on RIDING cells ONLY — a SATURATED lagged floor also has `plateau_pts == 1`, for a reason
/// that has nothing to do with tracking error (gate 4).
#[test]
fn the_plateau_breaks_at_every_bandwidth() {
    let bc = gt(&LeverArm::default())
        .bandwidth_ceiling(&flight(), &ramp(DS), PHI, B, &TAUS);
    assert!(bc.rows.iter().all(|x| !x.saturated), "the exclusion this gate depends on");
    assert!(bc.rows.iter().all(|x| x.plateau_pts == 1),
            "{:?}", bc.rows.iter().map(|x| (x.tau, x.plateau_pts)).collect::<Vec<_>>());
    assert!(bc.rows.iter().all(|x| x.plateau_span == 0.0));
    assert!(bc.inst_plateau_pts >= 100, "{}", bc.inst_plateau_pts);
}

/// The side-by-side is the finding. Under refinement the lagged argmin holds to a couple of grid
/// cells and its value converges; rung 64's plateau GROWS in proportion to `1/ds` — it is a
/// genuine interval, not a tie of a few points.
#[test]
fn the_restored_argmin_is_a_result_and_rung64s_is_a_grid_artefact() {
    let m = gt(&LeverArm::default());
    let mut lag = Vec::new();
    let mut inst = Vec::new();
    for ds in [0.01, 0.005, 0.0025] {
        lag.push(m.at_lever(&LeverArm::floored(valve(Some(TAU))))
                  .bill_cell(&flight(), &ramp(ds), false));
        inst.push(m.at_lever(&LeverArm::floored(valve(None)))
                   .bill_cell(&flight(), &ramp(ds), false));
    }
    let s: Vec<f64> = lag.iter().map(|c| c.s_at_min_lp).collect();
    let (lo, hi) = (s.iter().cloned().fold(f64::INFINITY, f64::min),
                    s.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!(hi - lo <= 2.0 * 0.0025 + 1e-12, "{s:?}");
    assert!((lag[2].min_phi_lp - lag[1].min_phi_lp).abs() < 1e-5);
    let p: Vec<usize> = inst.iter().map(|c| c.plateau_pts).collect();
    assert!(lag.iter().all(|c| c.plateau_pts == 1), "{p:?}");
    assert!(p[1] as f64 > 1.8 * p[0] as f64 && p[2] as f64 > 1.8 * p[1] as f64, "{p:?}");
}

// =============================================================================================
// GATE 6 — THE RUNG: the SOLVE repaired, the DEGENERACY conserved (§ 3)
// =============================================================================================

/// Rung 64 § 3 DERIVED that an instantaneous valve makes `G == 0` across the fuel leg's whole
/// bracket; it could not EXHIBIT the repair, because on its own plant there is nothing to
/// exhibit. Here the same bracket is swept on both plants at one state off an armed march: rung
/// 49's premise ("phi falls monotonically with fuel") is restored verbatim.
///
/// No wall-clock number is asserted — rung 64 § 3 was explicit that no number about the tangent
/// residual is a result, and cost is machine- and load-dependent.
#[test]
fn the_fuel_legs_own_plant_is_restored_the_discriminator() {
    let fa = gt(&LeverArm::default())
        .fuel_authority(&flight(), &ramp(DS), sm(), B, TAU, &[1.0, 0.99, 0.98, 0.95, 0.90]);
    assert!(fa.deleted && fa.inst.span < 1e-9);
    assert!(fa.restored && fa.lagged.span > 1e-3);
    assert!(fa.lagged.monotone && fa.lagged.sign_change);
    assert!(fa.ratio > 1e6);
}

/// THE RUNG. Two loops on one variable stay redundant: wherever both ride, every `(b, Wf)` on
/// `phi_lp = phi_lim` satisfies BOTH laws, so `db/ds == 0` and the valve position is a CONSTANT
/// OF THE MOTION — selected by the initial condition and unreachable by `tau`.
///
/// A frozen state alone would only be one initial condition's coincidence. The gate is the
/// CONTINUUM: the frozen value tracks `b0`, both laws stay exactly satisfied with the valve
/// strictly interior, and different members withhold DIFFERENT fuel.
#[test]
fn the_degeneracy_is_conserved_a_marginal_mode_with_an_edge() {
    let mm = gt(&LeverArm::default())
        .marginal_mode(&flight(), &ramp(DS), sm(), B, TAU, &[0.2, 0.01], 0.01);
    for c in [mm.natural, mm.moved_lo] {
        assert!(c.drift < 1e-12, "{c:?}");
        assert!(c.dbds < 1e-9, "{c:?}");
        assert!(c.laws_held < 1e-12 && c.interior);
    }
    // A RATIO, not an absolute threshold: what makes the family GENUINE rather than a technicality
    // is that its members withhold MATERIALLY different fuel, and only a scale-free floor pins
    // that. ONE-SIDED on purpose — the spec disclaims the magnitude, so an upper bound would gate
    // the grid, not the finding. Measured 1.166 between the natural member and one 0.01 below it.
    let ratio = mm.natural.removed / mm.moved_lo.removed;
    assert!(ratio > 1.10, "{ratio}");
    assert!(mm.tau_span_rel < 1e-9, "{}", mm.tau_span_rel);   // tau multiplies a machine zero
}

/// The family is `b0 in (0, b_cmd(0)]`, and the edge is DERIVABLE: the valve's law is the
/// SMALLEST position holding the floor, so above `b_cmd(0)` it is doing more than its own law
/// asks, its command sits below the live position, and it closes. The physical initial condition
/// sits precisely ON that upper edge — which is why the natural march looks like a unique
/// solution.
#[test]
fn the_continuums_upper_edge_is_the_valves_own_minimality_law() {
    let m = gt(&LeverArm::default())
        .at_lever(&LeverArm::floored(BleedLimiter::from_margin_tau(&lp_map(), B, sm(),
                                                                   Some(TAU))));
    let fuel = SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm());
    let leg = StatorLeg { accel: None, surge: Some(Floor::Phi(fuel)), tt4_max: None };
    let (nat, _) = m.stator_march(&flight(), &ramp(0.01), None, &leg);
    let edge = b_of_point(&nat[0]).0;
    let drift = |b0: f64| -> f64 {
        let (t, _) = m.stator_march_scoped(&flight(), &ramp(0.01), None, &leg,
                                           &MarchScope { b0: Some(b0), ..MarchScope::DEFAULT });
        let first = b_of_point(&t[0]).0;
        t.iter().map(|p| (b_of_point(p).0 - first).abs()).fold(f64::NEG_INFINITY, f64::max)
    };
    assert!(drift(0.99 * edge) < 1e-12);          // inside  -> frozen
    assert!(drift(edge) < 1e-12);                 // ON the edge -> frozen
    assert!(drift(1.01 * edge) > 1e-6);           // outside -> the valve closes
}

// =============================================================================================
// GATE 7 — THE MODELLING FLOOR: the artifact that would have counterfeited the rung
// =============================================================================================

/// § 0's RETRACTION, made unreachable. A first pre-check ran `ds/tau = 5` and returned an
/// `int b ds` 4.4× the grid-converged value — an instability that looks exactly like a physical
/// finding. No future sweep may reproduce it silently.
#[test]
fn the_rk4_stability_floor_on_ds_over_tau_is_asserted() {
    let m = gt(&LeverArm::floored(valve(Some(0.002))));
    assert!(panics(std::panic::AssertUnwindSafe(|| {
        m.stator_march(&flight(), &ramp(0.01), None, &StatorLeg::default());
    })));
    let m2 = gt(&LeverArm::floored(valve(Some(0.01))));   // ds/tau = 0.5, the sweep's floor
    let (traj, _) = m2.stator_march(&flight(), &ramp(0.005), None, &StatorLeg::default());
    assert!(traj.len() > 300);
}

/// The modelling floor rungs 62/63/64 each check, at the WIDEST position a rung-65 law can
/// command — a saturated floor under the SLOWEST valve in the sweep.
#[test]
fn every_march_stays_on_the_choked_branch() {
    let m = gt(&LeverArm::default());
    let over = m.at_lever(&LeverArm::constant(B)).bill_cell(&flight(), &ramp(0.01), false)
                .min_phi_lp * 1.10;
    for lim in [valve(Some(0.4)), valve(Some(TAU)), BleedLimiter::with_tau(over, B, Some(0.4))] {
        let (traj, _) = m.at_lever(&LeverArm::floored(lim))
                         .stator_march(&flight(), &ramp(0.01), None, &StatorLeg::default());
        assert!(!traj.is_empty() && traj.iter().all(|p| p.branch == Branch::Choked), "{lim:?}");
    }
}

// =============================================================================================
// THE ADDED GATE — § 5.23 (vii): the constructor chain, assert by assert
// =============================================================================================

/// **THE PORT'S OWN GATE, AND THE PYTHON SUITE HAS NO REASON TO OWN IT.**
///
/// Python's rung 65 inherits rung 64's `__init__` and flips `_LAG_OK`; Rust re-spells the chain.
/// Probe 5 measured **ten** asserts on it, and re-spelling can drop any of the nine that are NOT
/// relaxed while every value key stays green — the machines the suite builds are all legal, so
/// nothing exercises a refusal that has gone missing.
///
/// Each row below is one assert, driven individually on a rung-65 machine. The COUNT is asserted
/// too, so a future rung adding an eleventh cannot pass silently.
#[test]
fn the_constructor_chain_is_ten_asserts_and_only_one_is_relaxed() {
    let vsv = |v: f64| StatorArm { vsv_lp: v, ..StatorArm::default() };
    let armed_map = || lp_map().with_vsv(0.05);
    let mut fired = 0;

    // 3 — rung 57's capture discipline: the DESIGN-setting maps, not ones already statored.
    assert!(panics(|| {
        build_lagged_bleed(design(), flight(), 1.0, Some(armed_map()), Some(hp_map()), 1.0,
                           &LeverArm::default());
    }), "3");
    fired += 1;
    // 4 — a spool gets a CONSTANT setting or a SCHEDULE, not both.
    assert!(panics(|| {
        gt(&LeverArm::stator(StatorArm {
            vsv_lp: 0.05,
            sched_lp: Some(turbojet::stator_transient::StatorSchedule::new(0.1, N_LO)),
            ..StatorArm::default()
        }));
    }), "4");
    fired += 1;
    // 5 — the same on the HP.
    assert!(panics(|| {
        gt(&LeverArm::stator(StatorArm {
            vsv_hp: 0.05,
            sched_hp: Some(turbojet::stator_transient::StatorSchedule::new(0.1, N_LO)),
            ..StatorArm::default()
        }));
    }), "5");
    fired += 1;
    // 6 — `lp_disabled` is not a reduce axis for rung 57's per-spool findings.
    assert!(panics(|| {
        gt(&LeverArm::stator(StatorArm { lp_disabled: true, ..vsv(0.05) }));
    }), "6");
    fired += 1;
    // 7 — rung 62: the valve gets a CONSTANT position or a SCHEDULE, not both.
    assert!(panics(|| {
        gt(&LeverArm { bleed: B, bleed_sched: Some(BleedSchedule::new(B, N_LO)),
                       ..Default::default() });
    }), "7");
    fired += 1;
    // 8 — rung 42's starved-core bound.
    assert!(panics(|| { gt(&LeverArm::constant(0.6)); }), "8");
    fired += 1;
    // 9 — rung 64's THREE-way arming exclusion, both of its arms.
    assert!(panics(|| {
        gt(&LeverArm { bleed: B, bleed_lim: Some(valve(None)), ..Default::default() });
    }), "9a");
    fired += 1;
    assert!(panics(|| {
        gt(&LeverArm { bleed_sched: Some(BleedSchedule::new(B, N_LO)),
                       bleed_lim: Some(valve(None)), ..Default::default() });
    }), "9b");

    // 10 — THE RELAXED ONE. Rung 64 refuses a lagged limiter; rung 65 is the class that flips
    //      `_LAG_OK`, so the SAME argument must be accepted here and refused there. Asserting
    //      both sides is what makes "relaxed" mean something: one arm alone is satisfied by a
    //      builder with no chain at all.
    assert!(panics(|| { lt(&LeverArm::floored(valve(Some(TAU)))); }), "10 must refuse at rung 64");
    let _ok = gt(&LeverArm::floored(valve(Some(TAU))));
    fired += 1;

    // 1–2 are `TwoSpoolMatcher`'s and are reached through the same `with_tables` call; they are
    // driven by `tests/rung38.rs` and not re-driven here. The COUNT is what this line pins.
    assert_eq!(fired + 2, 10,
               "probe 5 measured TEN asserts on the chain rung 65 re-spells. If this number \
                moved, a rung added one and `build_lagged_bleed` has not been re-read.");
}
