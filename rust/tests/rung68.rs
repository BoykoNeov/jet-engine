//! RUNG 68 — **THREE LOOPS ON ONE VARIABLE**: a lagged STATOR limiter beside rung 65's lagged
//! VALVE and rung 52's lagged FUEL leg, all three holding `phi_lp` to the same `phi_lim`.
//!
//! **THE HEADLINE:** `n` loops on one variable are ONE loop with all `n` RATES ADDED. `n` laws
//! that hold the same variable to the same set point have `dU_i/du_j = -phi_j/phi_i` UNIFORMLY —
//! the diagonal is not a special case — so `J = -D c r^T` is RANK ONE at every `n`, every plant,
//! every bandwidth: `n-1` zero eigenvalues and one root at `-sum 1/tau_i`. Rung 66's identity is
//! the `n = 2` case of that, not a property of pairs.
//!
//! **THE `n >= 3` CONTENT IS THE CYCLIC PRODUCT.** Rung 66's three pairwise identities leave the
//! 3×3 with one free parameter, `x = R_q C_v V_g`, and `det = (x+1)^2/x` — so a block can be
//! pairwise-degenerate and still rank 2. Only `x` (predicted −1) tests JOINT collapse.
//!
//! **AND IT EXTENDS RUNG 64.** `v_max` — the AUTHORITY rung 64 made the ceiling on protection —
//! is EXACTLY inert on the triple and decisively binding on the same lever alone.
//!
//! **THE TWO ARTIFACTS THAT WOULD HAVE COUNTERFEITED THE RUNG**, and gates 6 and 7 exist for
//! them: a SATURATED loop costs the block a zero, so an unfiltered instrument reports a fully
//! INDEPENDENT triple (the inverse of rung 67's lesson); and rung 66's RK4 constant admits steps
//! at which this plant reports the floor EXACTLY HELD with a violation integral of zero.
//!
//! Ported from `tests/test_rung68.py` — 22 tests, of which 9 carry `slow` there. **The marker is
//! dropped here per slice M's rule** and `#[ignore]` is re-introduced only against a MEASURED
//! Rust cost, never inherited.

use std::panic::catch_unwind;

use turbojet::bleed_transient::{BleedSchedule, LeverArm, LeverArming};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    asym_extra, AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::valve_of;
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{
    build_three_loop_cascade, cyclic_sensitivity, ic_family, saturation_counterfeit, triple_bill,
    triple_gains, triple_modes, v_at_point, StatorLimiter, TripleHooks, TripleRigArm, R68,
    R68_FUEL, R68_STATOR, R68_TRIPLE, R68_TWO,
};
use turbojet::two_lag::violation;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

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
/// `PHI / FLOOR - 1.0` — spelled as the expression Python spells it as, because the three floors
/// being ONE set point is § 2's own scope and a typed decimal would break it silently.
const SM: f64 = PHI / FLOOR - 1.0;
/// The valve's and the stator's clocks.
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
/// Rung 52's fast-attack / slow-release fuel leg.
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;

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

fn full(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this grid never disables LP"),
    }
}

/// Python's `_three` — a rung-68 machine.
fn three(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_three_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_two` — the rung-66 machine every reduce arm is compared against.
///
/// **Rung 66's constructor is reached through rung 67's builder**, which is what
/// `build_two_lag_cascade` is in this crate; rung 67 rebinds no construction assert, so the two
/// differ only in the tables and rung 66's is the one `TwoLagCascadeTransient` installs.
fn two(arm: &LeverArm) -> ScheduledStatorCore {
    full(turbojet::two_lag::build_two_lag_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }

fn stator(tau: Option<f64>, v_max: f64) -> StatorLimiter {
    StatorLimiter::new(PHI, v_max, tau)
}

fn fuel_floor() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM)) }

fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

/// Python's `_march`.
fn march(
    m: &ScheduledStatorCore, ds: f64, surge: Option<Floor>, lg: Option<AsymmetricLag>,
    tt4_max: Option<f64>, scope_extra: MarchScope,
) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge, tt4_max };
    m.stator_march_scoped(&flight(), &ramp(ds), None, &leg,
                          &MarchScope { lag: lg, ..scope_extra }).0
}

/// The plain armed march every gate below opens with.
fn armed_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    march(m, DS, Some(fuel_floor()), Some(lag()), None, MarchScope::DEFAULT)
}

/// Python's `_keys` — the seven-tuple per point that the reduce gates compare.
fn keys(traj: &[FuelPoint]) -> Vec<[u64; 7]> {
    traj.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits()])
        .collect()
}

/// Python's `"v" in p` — a key test on a dict, which in Rust is a variant test on the point.
fn carries_v(p: &FuelPoint) -> bool { matches!(p.extra, PointExtra::Triple { .. }) }

fn v_regime_of(p: &FuelPoint) -> Regime {
    match p.extra {
        PointExtra::Triple { v_regime, .. } => v_regime,
        _ => panic!("this point carries no stator regime"),
    }
}

fn panics_with<F: FnOnce() + std::panic::UnwindSafe>(f: F, needle: &str) -> bool {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f);
    std::panic::set_hook(prev);
    match out {
        Ok(()) => false,
        Err(e) => {
            let msg = e.downcast_ref::<String>().cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            msg.contains(needle)
        }
    }
}

fn approx(a: f64, b: f64, rel: f64) -> bool { (a - b).abs() <= rel * b.abs() }

/// The rung's own machine and its march — Python's module-scoped `triple` fixture.
///
/// **Rebuilt per test rather than shared.** [[xdist-module-fixture-cost]] is the Python side of
/// this; here each `#[test]` is its own thread and a shared lazy would have to be `Sync`, which
/// [`ScheduledStatorCore`]'s `Cell` fields are not — deliberately, because those cells ARE the
/// dynamically-scoped state.
fn triple() -> (ScheduledStatorCore, Vec<FuelPoint>) {
    let m = three(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                              stator_lim: Some(stator(Some(TAU_S), V_MAX)),
                              ..Default::default() });
    let t = armed_march(&m);
    (m, t)
}

// =============================================================================================
// GATE 1 — THE REDUCE
// =============================================================================================

/// `stator_lim = None` with both other clocks armed: rung 66's cascade, unchanged.
#[test]
fn reduce_no_stator_is_rung66_bit_for_bit() {
    let arm = LeverArm { bleed_lim: Some(valve(Some(TAU))), ..Default::default() };
    let a = armed_march(&three(&arm));
    let b = armed_march(&two(&arm));
    assert_eq!(keys(&a), keys(&b));
    assert!(!carries_v(&a[0]), "rung 66's arm must not carry a fifth state");
    // ...but must still carry rung 66's four.
    assert!(matches!(a[0].extra, PointExtra::Cascade { .. }));
}

/// Rung 65's arm (`lag = None`), rung 52's (no valve) and rung 64's (no clocks at all) all leave
/// through the SAME parent `integrate_fuel`, so a rung-68 machine with no stator **is** every one
/// of its ancestors.
#[test]
fn reduce_inherited_arms_bit_for_bit() {
    let cases: [(LeverArm, Option<Floor>, Option<AsymmetricLag>); 4] = [
        // rung 65
        (LeverArm { bleed_lim: Some(valve(Some(TAU))), ..Default::default() },
         Some(fuel_floor()), None),
        // rung 52
        (LeverArm::default(), Some(fuel_floor()), Some(lag())),
        // rung 64
        (LeverArm { bleed_lim: Some(valve(None)), ..Default::default() }, None, None),
        // rung 62
        (LeverArm { bleed_sched: Some(BleedSchedule::new(B, 0.65)), ..Default::default() },
         None, None),
    ];
    for (i, (arm, surge, lg)) in cases.into_iter().enumerate() {
        let a = march(&three(&arm), DS, surge, lg, None, MarchScope::DEFAULT);
        let b = march(&two(&arm), DS, surge, lg, None, MarchScope::DEFAULT);
        assert_eq!(keys(&a), keys(&b), "case {i}");
        assert!(!carries_v(&a[0]), "case {i}");
    }
}

/// A `StatorLimiter` without `tau` cannot be marched, and dropping it would make every reader
/// report a third loop that never acted.
#[test]
fn an_unlagged_stator_is_refused_not_silently_dropped() {
    // `tau = 0` is NOT the instantaneous loop, it is a bug.
    assert!(panics_with(|| { stator(Some(0.0), V_MAX); }, "INSTANTANEOUS"));
    let m = three(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                              stator_lim: Some(stator(None, V_MAX)),
                              ..Default::default() });
    let a = armed_march(&m);
    assert!(!carries_v(&a[0]), "an unlagged stator must not enter the five-state integrator");
}

#[test]
fn the_triple_is_the_only_five_state_path() {
    let (_, traj) = triple();
    assert!(carries_v(&traj[0]));
    assert_eq!(traj[0].key_count(), 24);
    assert_eq!(traj.len(), 341);
}

/// The SIXTH instance of the trap rungs 61–66 each hit — and the first where the signature GROWS,
/// so the failure mode is also *silently drops the third loop*.
#[test]
fn at_lever_returns_this_class_and_keeps_the_third_loop() {
    let (m, _) = triple();
    let s = m.at_lever(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                   stator_lim: Some(stator(Some(TAU_S), V_MAX)),
                                   ..Default::default() });
    assert_eq!(s.fuel.inner.stator.lim.map(|l| l.tau), Some(Some(TAU_S)));
    assert!(carries_v(&armed_march(&s)[0]),
            "the sibling must still march the five-state integrator");
}

/// § 2's identity needs ONE SET POINT, not merely one variable: rung 66 measured a −2.5 % offset
/// moving the product to 0.951. Two floors that disagree are a different rung.
#[test]
fn one_set_point_is_enforced() {
    assert!(panics_with(|| {
        three(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                          stator_lim: Some(StatorLimiter::new(0.78, V_MAX, Some(TAU_S))),
                          ..Default::default() });
    }, "ONE SET POINT"));
}

/// Rung 47's `tau_gov` watches `Tt4` — adding it here is THREE loops on TWO variables, which
/// superposes rung 67's `P < 0` block onto this rank-one one. Rung 68's own next seam.
#[test]
fn three_loops_on_two_variables_is_refused() {
    assert!(panics_with(|| {
        let (m, _) = triple();
        march(&m, DS, Some(fuel_floor()), Some(lag()), Some(1200.0),
              MarchScope { tau_gov: Some(0.05), ..MarchScope::DEFAULT });
    }, "THREE loops on TWO variables"));
}

// =============================================================================================
// GATE 2 — THE CYCLIC PRODUCT
// =============================================================================================

#[test]
fn cyclic_product_is_minus_one_and_the_pairs_are_one() {
    let (m, _) = triple();
    let g = triple_gains(&m, &flight(), &ramp(DS), SM, &TripleRigArm::default(), 10);
    assert!(g.n_riding >= 50, "{}", g.n_riding);
    assert!(g.rows.len() >= 8);
    for row in &g.rows {
        assert!((row.on.cyclic + 1.0).abs() < 1e-6, "{} {}", row.s, row.on.cyclic);
        for (k, x) in [("pair_RC", row.on.pair_rc), ("pair_RV", row.on.pair_rv),
                       ("pair_CV", row.on.pair_cv)] {
            assert!((x - 1.0).abs() < 1e-6, "{} {k} {x}", row.s);
        }
    }
}

/// **THE GATE THAT KEEPS THE ONE ABOVE FROM BEING A TAUTOLOGY.** With the three pairwise
/// identities imposed exactly, the block still has a free parameter: build one whose pairs are all
/// 1 and whose cyclic product is NOT −1, and check `det != 0`. If this ever passes with
/// `det == 0`, the cyclic measurement above is measuring nothing.
#[test]
fn the_cyclic_product_is_not_implied_by_the_pairwise_ones() {
    let (a, c): (f64, f64) = (-7.0e-2, -1.0 / 7.0e-2);   // ac = 1
    let (b, e): (f64, f64) = (4.0e-2, 1.0 / 4.0e-2);     // be = 1
    let (d, f): (f64, f64) = (2.0, 0.5);                 // df = 1
    let x = a * d * e;                       // the cyclic product, free
    let det = -1.0 * ((-1.0) * (-1.0) - d * f) - a * (c * (-1.0) - d * e)
        + b * (c * f - (-1.0) * e);
    assert!((x + 1.0).abs() > 0.1, "this hand-built block must NOT be cyclically degenerate");
    assert!(det.abs() > 0.1, "...and must therefore be rank 3 despite all pairs being 1");
    assert!(approx(det, (x + 1.0) * (x + 1.0) / x, 1e-9),
            "det is a monotone re-expression of the cyclic product -- which is why the spec \
             quotes x and not det, tr or the second invariant");
}

/// **MEASURE THE DETECTOR, DO NOT ASSERT THE NULL.** Displacing the stator off the shared manifold
/// by `delta` must move the departure LINEARLY and far above the noise floor — otherwise
/// `cyclic == -1` is a statement about the instrument, not the plant.
#[test]
fn the_detector_resolves_far_below_what_it_claims() {
    let (m, _) = triple();
    let s = cyclic_sensitivity(&m, &flight(), &ramp(DS), SM, &TripleRigArm::default(),
                               &[0.0, 1e-4, 1e-3, 1e-2, 3e-2]);
    assert!(s.floor < 1e-7, "{}", s.floor);
    let gain = s.gain.expect("a live detector has a gain");
    assert!((1.0..2.0).contains(&gain), "{gain}");
    let dep = |d: f64| -> f64 {
        s.rows.iter().find(|r| r.delta == d).and_then(|r| r.dep)
            .expect("this delta stayed interior")
    };
    assert!(dep(1e-3).abs() > 100.0 * s.floor, "{} {}", dep(1e-3), s.floor);
    // LINEARITY: a decade in delta is a decade in the departure.
    assert!(approx((dep(1e-2) / dep(1e-3)).abs(), 10.0, 0.05));
    assert!(approx((dep(1e-3) / dep(1e-4)).abs(), 10.0, 0.05));
}

// =============================================================================================
// GATE 3 — THE SPECTRUM: n−1 = 2 zeros, and the RATES ADD at n = 3
// =============================================================================================

/// `tr J = -sum 1/tau_i` is the ODE's own diagonal and is **NOT a measurement**. What IS measured
/// is that the other two roots vanish — equivalently that the second invariant (the three PAIRWISE
/// identities, weighted) and the determinant (the CYCLIC one) are both zero. The dominant root
/// then equals the rate sum as a CONSEQUENCE.
#[test]
fn two_zero_eigenvalues_and_the_rates_add() {
    let (m, _) = triple();
    let clocks = [(0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05),
                  (0.02, 0.05, 0.10)];
    let arms = triple_modes(&m, &flight(), &ramp(0.002), SM, &clocks, V_MAX, 3.0, 20);
    assert_eq!(arms.len(), 4);
    for arm in &arms {
        assert!(!arm.rows.is_empty(), "{:?}", arm.taus);
        assert!(arm.skipped <= 2, "{:?} {}", arm.taus, arm.skipped);
        let scale = arm.rate_sum.abs();
        let worst = arm.worst_zero.expect("a live arm has rows");
        assert!(worst < 1e-4 * scale, "{:?} {worst}", arm.taus);
        for x in &arm.rows {
            assert!(approx(x.dom, arm.rate_sum, 1e-4), "{:?}", arm.taus);
            assert!((x.cyclic + 1.0).abs() < 1e-6);
        }
    }
}

// =============================================================================================
// GATE 4 — WHAT THE TRIPLE DELIVERS. All three marginals, and both walls.
// =============================================================================================

fn bill() -> turbojet::three_loop::TripleBill {
    let (m, _) = triple();
    triple_bill(&m, &flight(), &ramp(DS), SM, &TripleRigArm::default())
}

#[test]
fn the_pair_beats_every_single_and_the_triple_beats_every_pair() {
    let b = bill();
    for one in ["F", "V", "S"] {
        assert!(b.cell("FVS").i < b.cell(one).i, "{one}");
    }
    for two in ["FV", "FS", "VS"] {
        assert!(b.cell("FVS").i < b.cell(two).i, "{two}");
    }
}

/// Rung 66 § 9 predicted the third limiter would buy LESS than the second's 1.59 %. **It does
/// not**, and the reason is that credit is not a function of the rate sum: rung 66's own two
/// marginals differed by 21× while BOTH doubled it. All three are quoted.
#[test]
fn strongly_subadditive_and_the_ordering_is_the_object() {
    let b = bill();
    assert!(b.sum_singles > 2.4 * b.delivered);
    let (fu, va, st) = b.marginal;
    assert!(fu < st && st < va, "{fu} {st} {va}");
    let e = [b.erosion.0, b.erosion.1, b.erosion.2];
    let hi = e.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let lo = e.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(hi / lo > 4.0, "{e:?}");                    // 122x vs 10x, measured
    assert!(st > 1.59, "the seam's magnitude prediction is a MISS: {st}");
}

/// **RUNG 53's *a margin is a DISTANCE*, landing on a ledger.** The stator MOVES the `phi` wall and
/// leaves the metal one alone, so a credit quoted without its wall is meaningless: the same loop
/// is strongly protective in `phi` and actively harmful in incidence.
#[test]
fn the_credit_flips_sign_between_the_two_walls() {
    let b = bill();
    assert!(b.marginal.2 > 0.0);
    assert!(b.marginal_incidence.2 < 0.0);
    assert!(b.cell("S").credit > 80.0);
    assert!(b.cell("S").credit_inc < 0.0);
    // the valve, which does NOT move either wall, keeps its sign in both
    assert!(b.marginal.1 > 0.0 && b.marginal_incidence.1 > 0.0);
}

// =============================================================================================
// GATE 5 — v_max: INERT in company, BINDING alone. This EXTENDS rung 64.
// =============================================================================================

/// Rung 64: *a limiter's LAW cannot buy PROTECTION, only its PRICE — the ceiling is the lever's
/// AUTHORITY.* That is a statement about a lever ALONE. Here the SAME ceiling is **EXACTLY inert**
/// once two other loops hold the same variable, because they take up the demand before the stop is
/// reached.
#[test]
fn authority_is_inert_on_the_triple_and_binds_on_the_lever_alone() {
    let run = |v_max: f64, with_valve: bool| -> (f64, f64, bool) {
        let m = three(&LeverArm {
            bleed_lim: if with_valve { Some(valve(Some(TAU))) } else { None },
            stator_lim: Some(stator(Some(TAU_S), v_max)),
            ..Default::default() });
        let t = march(&m, DS,
                      if with_valve { Some(fuel_floor()) } else { None },
                      if with_valve { Some(lag()) } else { None },
                      None, MarchScope::DEFAULT);
        (violation(&t, PHI, R),
         t.iter().map(v_at_point).fold(f64::INFINITY, f64::min),
         t.iter().any(|p| v_regime_of(p) == Regime::Saturated))
    };
    let trip: Vec<(f64, f64, bool)> = [0.05, 0.10, 0.20].iter().map(|&v| run(v, true)).collect();
    let alone: Vec<(f64, f64, bool)> = [0.05, 0.10, 0.20].iter().map(|&v| run(v, false)).collect();
    // IN COMPANY: identical to the ROOT TOLERANCE across a 4x ceiling, and never on the stop.
    // NOT bit-for-bit, and the reason is disclosed rather than tuned away: `v_max` is one end of
    // `solve_v`'s bracket, so it moves Illinois's first secant and the converged root lands ~1e-15
    // apart. That is the solver's own resolution, four orders below the 1.4x effect the same
    // ceiling has on the lever ALONE.
    assert!(approx(trip[0].0, trip[2].0, 1e-12));
    assert!(approx(trip[1].0, trip[2].0, 1e-12));
    assert!(!trip.iter().any(|t| t.2));
    // ALONE: the ceiling is decisive, and it saturates.
    assert!(alone[0].0 > 1.4 * alone[2].0);
    assert!(alone[0].2 && alone[1].2);
    // and a TIGHT enough ceiling reaches even the triple, so the inertness is a MEASUREMENT.
    let tight = run(0.02, true);
    assert!(tight.2 && tight.0 > 1.2 * trip[2].0);
}

// =============================================================================================
// GATE 6 — THE SATURATION CONFOUND
// =============================================================================================

/// The INVERSE of rung 67's lesson (*a zero cross-gain is saturation, never decoupling*): there a
/// stop faked the absence of COUPLING in one entry; here it fakes the absence of REDUNDANCY in the
/// whole block. **This is why every reader filters on the REGIME LABEL.**
#[test]
fn a_saturated_loop_costs_the_block_a_zero() {
    let (m, _) = triple();
    let r = saturation_counterfeit(&m, &flight(), &ramp(DS), SM, &TripleRigArm::default(), 0.02);
    assert!(r.n_saturated > 10 && r.n_riding > 10, "{} {}", r.n_saturated, r.n_riding);
    let sat = r.rows.iter().find(|x| x.regime == Regime::Saturated).expect("a saturated row");
    let rid = r.rows.iter().find(|x| x.regime == Regime::Riding).expect("a riding row");
    assert!(sat.v_g == 0.0 && sat.v_q == 0.0, "a stop returns EXACT zeros, measured");
    assert!(!sat.off_regime.is_empty(), "and the filter must have flagged it");
    assert_eq!(sat.n_zero, 0, "the unfiltered block reads as fully INDEPENDENT");
    assert_eq!(rid.n_zero, 2, "the riding one keeps both zeros");
    assert!(rid.off_regime.is_empty());
}

/// The trap made concrete: `v < 0` is TRUE for a saturated stator and for a riding one alike, so a
/// reader that inferred the regime from the float would admit both.
#[test]
fn a_float_comparison_against_the_stop_is_not_the_regime() {
    let m = three(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                              stator_lim: Some(stator(Some(TAU_S), 0.02)),
                              ..Default::default() });
    let t = armed_march(&m);
    let sat: Vec<&FuelPoint> = t.iter().filter(|p| v_regime_of(p) == Regime::Saturated).collect();
    let rid: Vec<&FuelPoint> = t.iter().filter(|p| v_regime_of(p) == Regime::Riding).collect();
    assert!(!sat.is_empty() && !rid.is_empty());
    assert!(sat.iter().all(|p| v_at_point(p) < 0.0) && rid.iter().all(|p| v_at_point(p) < 0.0),
            "if this ever fails the trap has gone away and this gate is dead weight");
    // The three-valued enum is what makes the last Python assertion (`<= {dormant, riding,
    // saturated}`) a TYPE fact here rather than a set comparison.
}

// =============================================================================================
// GATE 7 — THE RK4 FLOOR
// =============================================================================================

#[test]
fn the_floor_is_tighter_than_rung_66s_and_it_fires() {
    let ds = 0.04;   // ds*(1/t_g + 1/t_v) = 1.6 <= 2 -- rung 66 ADMITS this
    assert!(ds * (1.0 / TAU + 1.0 / TAU_ATT.min(TAU_REL)) <= 2.0);
    assert!(ds * (1.0 / TAU + 1.0 / TAU_ATT.min(TAU_REL) + 1.0 / TAU_S) > 2.0);
    assert!(panics_with(|| {
        let (m, _) = triple();
        march(&m, ds, Some(fuel_floor()), Some(lag()), None, MarchScope::DEFAULT);
    }, "RATES ADD"));
}

/// **AN ASSERT NOBODY HAS RUN PAST IS A TAUTOLOGY** (rung 67 gate 9). The guard is overridden to a
/// no-op and the refused band measured: at `ds = 0.05` — which rung 66's own constant ADMITS — the
/// march reports `min phi_lp` EXACTLY at the floor and a violation integral of **ZERO**. It does
/// not blow up the way rung 65's retraction did; **it counterfeits perfect protection**, which is
/// worse.
///
/// # THIS IS WHY `rk4_floor` IS A CELL, AND IT IS THE ONLY GATE IN THE SLICE THAT SAYS SO
///
/// Python subclasses the rung and overrides a `@staticmethod`. The port's answer is a swapped
/// table — [`TripleHooks`] with one cell replaced — which is the same experiment expressed as the
/// thing the port actually has. § 5.25's **P7** predicted exactly this: the cell is forced by a
/// PORTED TEST and not by a dispatch instrument, so a port that inlined the assert would fail
/// here rather than at step 5.
#[test]
fn what_the_refusal_refuses_is_measured_not_trusted() {
    fn no_op_floor(_: f64, _: f64, _: usize, _: f64) {}
    static UNGUARDED: TripleHooks = TripleHooks { rk4_floor: no_op_floor, ..R68_TRIPLE };

    let run = |hooks: &'static TripleHooks, ds: f64| -> (f64, f64) {
        let arm = LeverArm { bleed_lim: Some(valve(Some(TAU))),
                             stator_lim: Some(stator(Some(TAU_S), V_MAX)),
                             ..Default::default() };
        let m = full(ScheduledStatorTransient::with_triple_tables(
            design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm.stator,
            &R68_TWO, &R68_STATOR, &R68_FUEL, &R68,
            LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
            hooks, arm.stator_lim));
        let t = march(&m, ds, Some(fuel_floor()), Some(lag()), None, MarchScope::DEFAULT);
        (violation(&t, PHI, R), t.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min))
    };

    let fine = run(&R68_TRIPLE, 0.003125);
    let edge = run(&R68_TRIPLE, 0.03125);          // inside BOTH constants
    let bad = run(&UNGUARDED, 0.05);               // inside rung 66's, outside this
    assert!(fine.0 > 0.0 && edge.0 > 0.0);
    assert!(edge.0 < fine.0 && edge.0 > 0.9 * fine.0);   // degraded but still a number
    assert_eq!(bad.0, 0.0, "the counterfeit: no violation at all");
    assert!((bad.1 - PHI).abs() < 1e-9, "...and the floor exactly held: {}", bad.1);
}

// =============================================================================================
// GATE 8 — THE LIMITS. tau_s -> INFINITY removes the loop; tau_s -> 0 does NOT.
// =============================================================================================

/// **INVERTS every earlier lag in this family.** Rungs 65/66 send a clock to ZERO to recover the
/// instantaneous loop, so there the fast limit is the richer object. A third loop is an ADDITION,
/// so only the SLOW limit removes it.
#[test]
fn the_converging_limit_is_the_slow_one() {
    let refm = two(&LeverArm { bleed_lim: Some(valve(Some(TAU))), ..Default::default() });
    let i66 = violation(&armed_march(&refm), PHI, R);
    let i_at = |tau_s: f64| -> f64 {
        let m = three(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                  stator_lim: Some(stator(Some(tau_s), V_MAX)),
                                  ..Default::default() });
        violation(&armed_march(&m), PHI, R)
    };
    let slow: Vec<f64> = [0.5, 2.0, 10.0, 500.0].iter().map(|&t| i_at(t)).collect();
    let mut sorted = slow.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    assert_eq!(slow, sorted, "monotone in tau_s");
    assert!((slow[3] / i66 - 1.0).abs() < 1e-3, "{}", slow[3]);
    assert!((slow[0] / i66 - 1.0).abs() > 0.05, "...and not already there at tau_s = 0.5");
    assert!(i_at(0.02) < 0.7 * i66, "the FAST limit runs the other way -- a different object");
}

// =============================================================================================
// GATE 9 — THE INITIAL CONDITION is a FAMILY, and the member is DECLARED
// =============================================================================================

/// Rung 66's joint solve converged in one iteration because ITS march opened dormant. That escape
/// is gone at `n = 3` — the valve and the stator are both live at `s = 0` and they SHARE the
/// constraint — so the `s = 0` fixed points are a CURVE. From the DECLARED start every sweep order
/// lands on the same member; the family shows up when the START moves, which is rung 66 § 0's own
/// diagnosis: **non-uniqueness of the IC, not a stalled solve.**
#[test]
fn the_declared_start_is_rung_66s_member_and_the_family_is_real() {
    let (m, traj) = triple();
    let f = ic_family(&m, &flight(), &ramp(DS), SM, &TripleRigArm::default(),
                      &["gqv", "gvq", "qgv", "qvg", "vgq", "vqg"],
                      &[None, Some(0.0), Some(0.02), Some(0.06)]);
    assert_eq!(asym_extra(&traj[0]).0, 0.0);
    assert_eq!(v_at_point(&traj[0]), 0.0);
    assert!((valve_of(&traj[0]).0 - 0.036626).abs() < 1e-5);   // rung 66's own b0
    let (iters, res, order) = turbojet::three_loop::ic_at_point(&traj[0]);
    assert_eq!(iters, 1);
    assert_eq!(res, 0.0);
    assert_eq!(order, "gqv");
    assert_eq!(f.order_members, 1, "the order is NOT the lever from the declared start");
    assert!(f.by_order.iter().all(|(_, x)| x.iters == 1));
    let si = f.start_spread_i.expect("a live spread");
    let sw = f.start_spread_withheld.expect("a live spread");
    assert!(si > 0.5, "{si}");
    assert!(sw > 1.0, "{sw}");
}

#[test]
fn an_out_of_band_start_is_refused() {
    // `v0 > 0` is out of the band.
    assert!(panics_with(|| {
        let (m, _) = triple();
        march(&m, DS, Some(fuel_floor()), Some(lag()), None,
              MarchScope { v0: Some(0.05), ..MarchScope::DEFAULT });
    }, "stator POSITION"));
    assert!(panics_with(|| {
        let (m, _) = triple();
        march(&m, DS, Some(fuel_floor()), Some(lag()), None,
              MarchScope { ic_order: Some("ggv"), ..MarchScope::DEFAULT });
    }, "permutation"));
}
