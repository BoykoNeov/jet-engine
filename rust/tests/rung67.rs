//! RUNG 67 — CASCADE A: rung 47's lagged Tt4 GOVERNOR beside rung 65's lagged phi VALVE.
//!
//! Slice Z step 3. Python's `tests/test_rung67.py`, all **23** collected gates, in its order and
//! under its names.
//!
//! # THE HEADLINE
//!
//! One scalar decides both faces, and ADMISSIBILITY IS NOT OBSERVABILITY. The cross-gain product
//! `P = R_q*C_g` is the whole content of a two-loop actuator block. Two loops on ONE variable have
//! `P == +1` identically (rung 66) ⇒ degenerate, no oscillation at any clock ratio. Two loops on
//! TWO variables have `P < 0` ⇒ NON-degenerate, so the pair buys AUTHORITY, and the mode rung 66
//! forbids is admissible inside `rho + 1/rho < 2 + 4|P|`. BUT THE SAME SCALAR DAMPS IT:
//! `zeta = 1/sqrt(1+|P|)` and `T = 2 pi tau / sqrt|P|`, neither containing a time constant.
//!
//! # THE TWO GATES THAT ARE PART SELF-COMPARISON, SAID RATHER THAN HIDDEN
//!
//! [[rust-port-ported-test-vacuity]] — *a better factorisation turns a real pin into
//! self-comparison* — applies to two members of this file, and both are labelled at their own
//! doc comment rather than left to read as measurements:
//!
//! * [`the_window_formula_recovers_rung66_as_its_p_to_one_limit`]'s `zeta` assertion re-spells
//!   [`window`]'s own line. What it does pin independently is the RECIPROCAL identity (off the
//!   quadratic's two roots), the branch at `P >= 0`, and the two monotonicities.
//! * [`the_sum_bound_is_measured_conservative_not_assumed`]'s `2x at matched clocks` is a derived
//!   number, so the content is that the MEASURED spectral radius agrees with it — not the algebra.
//!
//! # THE `slow` BILL
//!
//! Python marks **9 of 23** and runs the file in **59.61 s** (§ 5.24 (ix)). The markers are
//! DROPPED, per slice M's rule, and the measured Rust cost is in the step-3 write-up.
//!
//! [`window`]: turbojet::cross_loop::window

use std::panic::catch_unwind;

use turbojet::bleed_transient::LeverArm;
use turbojet::cross_loop::{build_cross_loop_cascade, exceed, joint_fixed_point,
                           detector_sensitivity, window, IcCorner, RINGS};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AsymmetricLag, Floor, FuelLimiters, FuelPoint, PointExtra,
                               SurgeLimiter, TwoSpoolFuelTransient};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::{build_lagged_bleed, valve_of};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                                 StatorLeg};
use turbojet::two_lag::build_two_lag_cascade;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------------- the grid
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
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const TMAX: f64 = 1200.0;
/// The valve clock and the governor's.
const TAU: f64 = 0.05;
const TAU_GOV: f64 = 0.05;

/// The three clock ratios `cross_identity` is swept over — Python's default `tau_govs`, which the
/// suite also passes explicitly.
const TAU_GOVS: [f64; 3] = [0.005, 0.05, 0.5];
/// Python's `detector_sensitivity` defaults, which its one caller takes wholesale.
const PS: [f64; 4] = [-0.02, -0.5, -3.0, -10.0];
const DET_TAU: f64 = 0.05;
const DET_DS: f64 = 0.0025;
const DET_S_END: f64 = 1.7;
/// Python's `oscillation_window` default `d_b0`.
const OSC_D_B0: f64 = 0.005;
/// Python's `marginal_mode_*` default `d_b0`, which this suite passes explicitly.
const D_B0: f64 = 0.01;
/// Python's `_joint_fixed_point` defaults.
const IC_TOL: f64 = 1e-12;
const IC_CAP: usize = 60;
/// Python's `cascade_bill` / `marginal_mode_cascade` default `rel_mult`, passed explicitly by the
/// two cross-rung gates.
const REL_MULT: f64 = 3.0;

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

fn full(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("the LP spool is never disabled"),
    }
}

/// Python's `_cross(...)`.
fn cross(design: &TwoSpoolEngine, arm: &LeverArm) -> ScheduledStatorCore {
    full(build_cross_loop_cascade(design.clone(), flight(), 1.0, Some(lp_map()), Some(hp_map()),
                                  1.0, arm))
}

/// Python's `_cas66(...)`.
fn cas66(design: &TwoSpoolEngine, arm: &LeverArm) -> ScheduledStatorCore {
    full(build_two_lag_cascade(design.clone(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                               arm))
}

/// Python's `_lag65(...)`.
fn lag65(design: &TwoSpoolEngine, arm: &LeverArm) -> ScheduledStatorCore {
    full(build_lagged_bleed(design.clone(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                            arm))
}

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }

fn fuel() -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm()) }

/// The rung-47 GOVERNOR's leg.
fn gov_leg() -> StatorLeg<'static> {
    StatorLeg { accel: None, surge: None, tt4_max: Some(TMAX) }
}

/// Python's `_keys(traj, extra)` — the seven base fields.
fn keys(t: &[FuelPoint]) -> Vec<(f64, f64, f64, f64, f64, f64, f64)> {
    t.iter().map(|p| (p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.tt4, p.mf)).collect()
}

/// Python's `_keys(traj, ("b", "b_cmd"))`.
fn keys_valve(t: &[FuelPoint]) -> Vec<(f64, f64, f64, f64, f64, f64, f64, f64, f64)> {
    t.iter().map(|p| {
        let (b, b_cmd) = valve_of(p);
        (p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.tt4, p.mf, b, b_cmd)
    }).collect()
}

/// Python's `_keys(traj, ("b", "g", "required", "b_cmd"))`.
#[allow(clippy::type_complexity)]
fn keys_cascade(t: &[FuelPoint]) -> Vec<(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)> {
    t.iter().map(|p| {
        let (b, b_cmd) = valve_of(p);
        let (g, required) = turbojet::fuel_transient::asym_extra(p);
        (p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.tt4, p.mf, b, g, required, b_cmd)
    }).collect()
}

fn message<F: FnOnce()>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(std::panic::AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    let e = out.expect_err("the call was supposed to refuse and did not");
    match e.downcast_ref::<String>() {
        Some(s) => s.clone(),
        None => e.downcast_ref::<&str>().map(|s| (*s).to_string())
                 .unwrap_or_else(|| "<non-string panic>".into()),
    }
}

/// Python's `_ramp(m)` — the rung-45 accel ramp by hand, because rung 47 predates `_stator_march`.
/// Returns the schedule's two ends and the equilibrium start.
fn hand_ramp(m: &turbojet::fuel_transient::FuelTransientCore) -> (f64, f64, (f64, f64)) {
    let fl = flight();
    let (mf_lo, mf_hi) = (m.fuel_for_tt4(&fl, LO), m.fuel_for_tt4(&fl, HI));
    let eq = m.inner.equilibrium(&fl, LO);
    (mf_lo, mf_hi, (eq.nu_lp, eq.nu_hp))
}

fn sched_of(mf_lo: f64, mf_hi: f64) -> impl Fn(f64) -> f64 {
    move |s: f64| {
        if s <= 0.0 { mf_lo } else if s >= R { mf_hi } else { mf_lo + (mf_hi - mf_lo) * (s / R) }
    }
}

// =============================================================================================
// GATE 1-3 — THE REDUCE, all three bit-for-bit arms. The cross integrator is entered ONLY when
//            BOTH clocks are armed.
// =============================================================================================

/// `tau_gov=None`, `lag=None`: rung 65's arm, with and without the redline armed.
#[test]
fn reduce_valve_alone_is_rung65_bit_for_bit() {
    let des = design();
    let arm = LeverArm::floored(valve(Some(TAU)));
    for tmax in [None, Some(TMAX)] {
        let leg = StatorLeg { accel: None, surge: None, tt4_max: tmax };
        let (a, _) = cross(&des, &arm).stator_march(&flight(), &ramp(DS), None, &leg);
        let (b, _) = lag65(&des, &arm).stator_march(&flight(), &ramp(DS), None, &leg);
        assert_eq!(keys_valve(&a), keys_valve(&b), "Tt4_max = {tmax:?}");
        assert!(matches!(a[0].extra, PointExtra::Valve { .. }),
                "rung 65's arm must not carry a fourth state, got {:?}", a[0].extra);
    }
}

/// `tau_gov=None`, `lag` set: cascade B untouched — and all three of ITS arms with it. This is the
/// arm that breaks first if `stator_march`'s `tau_gov` plumbing leaks a default through.
#[test]
fn reduce_cascade_b_is_rung66_bit_for_bit() {
    let des = design();
    let arm = LeverArm::floored(valve(Some(TAU)));
    let lag = AsymmetricLag::new(TAU, 3.0 * TAU);
    let leg = StatorLeg { accel: None, surge: Some(Floor::Phi(fuel())), tt4_max: None };
    let scope = MarchScope { lag: Some(lag), ..MarchScope::DEFAULT };
    let (a, _) = cross(&des, &arm).stator_march_scoped(&flight(), &ramp(DS), None, &leg, &scope);
    let (b, _) = cas66(&des, &arm).stator_march_scoped(&flight(), &ramp(DS), None, &leg, &scope);
    assert_eq!(keys_cascade(&a), keys_cascade(&b));
}

/// `bleed_lim=None` with `tau_gov` set: rung 47's `_integrate_fuel_lagged`, untouched.
///
/// IT IS ALSO THE `Tt4_max` PLACEMENT DETECTOR. Rung 66 recorded that rung 52 and rung 65 place
/// the redline differently and that *"nothing would catch a wrong pick"* — because cascade B never
/// armed it. Cascade A's fuel leg IS the redline, so a wrong placement shows up right here, as a
/// diff against rung 47 itself.
#[test]
fn reduce_no_valve_is_rung47_bit_for_bit() {
    let (des, fl) = (design(), flight());
    let (a, _) = cross(&des, &LeverArm::default()).stator_march_scoped(
        &fl, &ramp(DS), None, &gov_leg(),
        &MarchScope { tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT });

    let m47 = TwoSpoolFuelTransient::new(des.clone(), fl, 1.0, lp_map(), hp_map(), 1.0);
    let core = m47.core();
    let (mf_lo, mf_hi, nu0) = hand_ramp(core);
    let b = core.integrate_fuel(
        &fl, sched_of(mf_lo, mf_hi), nu0, R + SETTLE, DS,
        &FuelLimiters { tt4_max: Some(TMAX), tau_gov: Some(TAU_GOV), ..Default::default() });

    assert_eq!(keys(&a), keys(&b));
    assert!(matches!(a[0].extra, PointExtra::None),
            "rung 47's arm must not carry a valve state, got {:?}", a[0].extra);
}

/// Only BOTH clocks armed reaches four states — the dispatch, stated as a gate.
#[test]
fn the_cross_cascade_is_the_only_four_state_path() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let (traj, _) = m.stator_march_scoped(
        &flight(), &ramp(DS), None, &gov_leg(),
        &MarchScope { tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT });
    // Python loops over the six keys; the named variant is all six at once, plus the refusal of a
    // route that carries six under other names.
    assert!(matches!(traj[0].extra, PointExtra::CrossCascade { .. }),
            "the cross integrator did not run: {:?}", traj[0].extra);
    assert_eq!(traj[0].key_count(), 21, "rung 67's dict is rung 66's twenty plus `ic_damp`");
    assert!(traj.iter().any(|p| turbojet::fuel_transient::asym_extra(p).1 > 0.0),
            "the governor never engaged");
    assert!(traj.iter().any(|p| { let c = valve_of(p).1; 0.0 < c && c < B }),
            "the valve never rode interior");
}

// =============================================================================================
// GATE 4 — the refusals. One rung, one headline.
// =============================================================================================

fn refusal(m: &ScheduledStatorCore, lim: &FuelLimiters<'_>) -> String {
    let (fl, l) = (flight(), lim.clone());
    let (mf_lo, mf_hi, nu0) = hand_ramp(&m.fuel);
    message(|| { let _ = m.fuel.integrate_fuel(&fl, sched_of(mf_lo, mf_hi), nu0, 0.05, DS, &l); })
}

fn gov_lim() -> FuelLimiters<'static> {
    FuelLimiters { tt4_max: Some(TMAX), tau_gov: Some(TAU_GOV), ..Default::default() }
}

#[test]
fn cascade_b_beside_cascade_a_is_refused() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let e = refusal(&m, &FuelLimiters { surge: Some(fuel()),
                                        lag: Some(AsymmetricLag::new(TAU, 3.0 * TAU)),
                                        ..gov_lim() });
    assert!(e.contains("CASCADE A"), "{e}");
}

/// A `surge` leg beside the governor puts a SECOND loop back on `phi_lp`, superposing rung 66's
/// identity onto this rung's window.
#[test]
fn a_second_fuel_leg_is_refused() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let e = refusal(&m, &FuelLimiters { surge: Some(fuel()), ..gov_lim() });
    assert!(e.contains("three loops"), "{e}");
}

#[test]
fn a_governor_clock_without_a_redline_is_refused() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let e = refusal(&m, &FuelLimiters { tau_gov: Some(TAU_GOV), ..Default::default() });
    assert!(e.contains("redline"), "{e}");
}

#[test]
fn forced_release_edges_are_refused() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let e = refusal(&m, &FuelLimiters { s_off: Some(0.02), ..gov_lim() });
    assert!(e.contains("FORCED release"), "{e}");
}

/// Rung 66's refusal is not weakened by rung 67 existing — a rung-66 machine still asserts, and
/// reaching cascade A means reaching the rung-67 class.
#[test]
fn rung66_still_refuses_cascade_a_on_its_own_class() {
    let m = cas66(&design(), &LeverArm::floored(valve(Some(TAU))));
    let e = refusal(&m, &FuelLimiters { surge: Some(fuel()),
                                        lag: Some(AsymmetricLag::new(TAU, 3.0 * TAU)),
                                        ..gov_lim() });
    assert!(e.contains("cascade A"), "{e}");
}

// =============================================================================================
// GATE 5 — THE SCALAR. Opposite signs, P < 0, and `R_q != 0` as the `_b_state` gate: a zero
//          cross-gain is a MISSING coupling, not a weak one.
// =============================================================================================

#[test]
fn the_cross_gains_have_opposite_signs() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let d = m.cross_identity(&flight(), &ramp(DS), TMAX, TAU, &TAU_GOVS, 8);
    assert!(d.all_negative, "{:?}", d.rows);
    assert!(-0.05 < d.prod_lo && d.prod_lo <= d.prod_hi && d.prod_hi < 0.0,
            "{} {}", d.prod_lo, d.prod_hi);
    for row in &d.rows {
        assert!(row.n_ride > 50, "{row:?}");
        assert!(row.r_q_lo > 0.0 && row.r_q_hi > 0.0, "{row:?}");   // more bleed => hotter
        assert!(row.c_g_lo < 0.0 && row.c_g_hi < 0.0, "{row:?}");   // more clip  => less bleed
        assert_eq!(row.n_saturated, 0,
                   "a saturated valve reads C_g = 0, not a decoupled one");
        // the CONTROL: a near-constant product must not be a constant plant
        assert!(row.gain_span_r > 1.1 || row.gain_span_c > 1.1, "{row:?}");
    }
    // THE `_b_state` GATE. Drop it and R_q vanishes identically, with nothing else failing.
    assert!(d.r_q_min_abs > 1e-4,
            "R_q is machine-zero: the governor is not sensing the live valve position, so this \
             is two INDEPENDENT loops and not a cascade");
}

// =============================================================================================
// GATE 6-7 — THE WINDOW, and the null it implies. Admissible != observable.
// =============================================================================================

/// Complex INSIDE `rho + 1/rho < 2 + 4|P|`, real outside, edges exact reciprocals.
#[test]
fn the_window_is_log_symmetric_and_the_spectrum_lands_in_it() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let d = m.cross_identity(&flight(), &ramp(DS), TMAX, TAU, &TAU_GOVS, 8);
    // Python's `x["rho_lo"] < x["rho_clock"] < x["rho_hi"]` on a dict whose window keys may be
    // absent; a missing key would raise there and is `None` here, so an unopened window is
    // OUTSIDE rather than a silent pass.
    let inside: Vec<_> = d.rows.iter()
        .filter(|x| matches!((x.rho_lo, x.rho_hi),
                             (Some(lo), Some(hi)) if lo < x.rho_clock && x.rho_clock < hi))
        .collect();
    let outside: Vec<_> = d.rows.iter()
        .filter(|x| !matches!((x.rho_lo, x.rho_hi),
                              (Some(lo), Some(hi)) if lo < x.rho_clock && x.rho_clock < hi))
        .collect();
    assert!(inside.len() == 1 && outside.len() == 2, "{:?}",
            d.rows.iter().map(|x| x.rho_clock).collect::<Vec<_>>());
    for x in &inside {
        assert_eq!(x.n_complex, x.n_sample, "{x:?}");
    }
    for x in &outside {
        assert_eq!(x.n_complex, 0, "{x:?}");
    }
    for x in &d.rows {
        assert_eq!(x.opens, Some(true), "{x:?}");
        assert!(x.reciprocal.expect("an open window records its reciprocal") < 1e-12, "{x:?}");
        let (lo, hi) = (x.rho_lo.unwrap(), x.rho_hi.unwrap());
        assert!(0.70 < lo && lo < 0.80 && 1.25 < hi && hi < 1.40, "{x:?}");
        let z = x.zeta.unwrap();
        assert!(0.98 < z && z < 0.995, "{x:?}");
        let tt = x.t_over_tau.unwrap();
        assert!(40.0 < tt && tt < 50.0, "{x:?}");
    }
}

/// Rung 66's *"no oscillation at any clock ratio"* is the `P >= 0` branch of the SAME closed form,
/// not a separate assertion.
///
/// **THE `zeta` LINE IS A SELF-COMPARISON, IT PINS EVEN LESS THAN THAT, AND BOTH HALVES ARE
/// MEASURED.** `window` computes `1.0 / (1.0 + |P|).sqrt()` and this assertion re-spells it, so at
/// best it could catch a port that changed the SPELLING. **It cannot even do that**: step 3's
/// injection I6 swapped in `(1.0 / (1.0 + |P|)).sqrt()` — algebraically equal, and measured to
/// differ in the last bit at **5 of the 8** `P` values this file evaluates, the plant's own
/// `P_mid = -2.0388646020554284e-2` among them — and the `< 1e-15` bar admits a ~1.1e-16 gap, so
/// this gate stayed green and so did the other 61. The spelling is the ORACLE's to pin.
///
/// The assertions here with independent content are the RECIPROCAL identity — off the two roots of
/// `rho² − k rho + 1` — the closed branch at `P >= 0`, and the two monotonicities.
#[test]
fn the_window_formula_recovers_rung66_as_its_p_to_one_limit() {
    assert!(!window(1.0).opens && window(1.0).rho_lo.is_none());
    assert!(!window(0.5).opens);
    for p in [-1e-3, -0.02, -0.5, -3.0] {
        let x = window(p);
        assert!(x.opens, "{x:?}");
        let (lo, hi) = (x.rho_lo.unwrap(), x.rho_hi.unwrap());
        assert!((lo * hi - 1.0).abs() < 1e-12, "{x:?}");
        assert!(lo < 1.0 && 1.0 < hi, "{x:?}");
        assert!((x.zeta - 1.0 / (1.0 + p.abs()).sqrt()).abs() < 1e-15, "{x:?}");
    }
    // the window WIDENS and the damping FALLS with the same scalar
    assert!(window(-3.0).rho_hi.unwrap() > window(-0.5).rho_hi.unwrap()
            && window(-0.5).rho_hi.unwrap() > window(-0.02).rho_hi.unwrap());
    assert!(window(-3.0).zeta < window(-0.5).zeta && window(-0.5).zeta < window(-0.02).zeta);
}

/// A null result is worth nothing until the instrument is shown to fire. The counter reads 0 at
/// this plant's `|P|` because the mode is DEAD (`T = 44 tau`, `e^-44` per period), not because it
/// is blind: at `|P| = 0.5/3/10` the same RK4 and the same counter read 3/7/13.
#[test]
fn the_ringing_detector_fires_before_the_null_is_quoted() {
    let d = detector_sensitivity(&PS, DET_TAU, DET_DS, DET_S_END);
    assert!(d.fires && d.quiet_at_weak == Some(true));
    let by_p = |p: f64| d.rows.iter().find(|x| x.p == p).expect("a swept P").sign_changes;
    assert_eq!(by_p(-0.02), 0);
    assert!(by_p(-0.5) >= 2);
    assert!(by_p(-10.0) > by_p(-3.0) && by_p(-3.0) > by_p(-0.5));
    // a real pair is allowed ONE zero crossing, so the threshold is two
    assert_eq!(RINGS, 2);
}

/// The free response — natural vs `b0`-offset, differenced, so the ramp cancels — never rings, at
/// any clock ratio, inside the window or outside it.
#[test]
fn the_mode_is_admissible_and_unobservable() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let d = m.oscillation_window(&flight(), &ramp(DS), TMAX, TAU, &[0.5, 1.0, 2.0], OSC_D_B0);
    assert!(d.n_complex >= 1 && d.n_real >= 1, "the sweep must straddle the window edge");
    assert!(!d.rings_anywhere, "{:?}", d.rows);
    assert!(d.max_sign_changes <= 1,
            "one crossing is admissible for a REAL pair; two is not");
    assert!(d.window.zeta > 0.98);
}

// =============================================================================================
// GATE 8 — THE LEDGER. Opposite-sign off-diagonals, near-unity diagonal erosion.
// =============================================================================================

#[test]
fn the_cross_credit_off_diagonals_have_opposite_signs() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let d = m.cross_bill(&flight(), &ramp(DS), TMAX, TAU, TAU_GOV);
    assert!(d.valve_debits_t, "{} {} {}", d.credit_t_gov, d.credit_t_valve, d.credit_t_both);
    assert!(d.gov_credits_phi, "{} {} {}",
            d.credit_phi_gov, d.credit_phi_valve, d.credit_phi_both);
    assert!(-0.15 < d.valve_on_t && d.valve_on_t < -0.01, "{}", d.valve_on_t);
    assert!(0.10 < d.gov_on_phi && d.gov_on_phi < 0.35, "{}", d.gov_on_phi);
    // and it shows in the TRAJECTORY, not only the integral: the valve runs the engine HOTTER
    assert!(d.valve.max_tt4 > d.bare.max_tt4 + 10.0, "{} {}", d.valve.max_tt4, d.bare.max_tt4);
    for (name, c) in [("bare", d.bare), ("gov", d.gov), ("valve", d.valve), ("both", d.both)] {
        assert!(!c.truncated, "{name}: a truncated march is not comparable");
    }
}

/// The inverse of rung 66. Each loop keeps ~all of its standalone credit ON ITS OWN currency
/// (erosion ~1x) where rung 66's second loop kept 1/38th of it on the shared one.
#[test]
fn two_loops_on_two_variables_buy_authority() {
    let des = design();
    let a = cross(&des, &LeverArm::floored(valve(Some(TAU))))
        .cross_bill(&flight(), &ramp(DS), TMAX, TAU, TAU_GOV);
    assert!(a.erosion_gov < 1.5 && a.erosion_valve < 1.5, "{} {}", a.erosion_gov, a.erosion_valve);
    assert!(a.credit_t_gov > 0.70 && a.credit_phi_valve > 0.90,
            "{} {}", a.credit_t_gov, a.credit_phi_valve);
    // CASCADE B, RE-RUN AT CASCADE A's SETTINGS (rung 63's lesson: never quote a number taken at
    // another rung's settings). The two must share their `bare` and `valve` cells exactly.
    let b = cas66(&des, &LeverArm::default())
        .cascade_bill(&flight(), &ramp(DS), sm(), B, TAU, TAU, REL_MULT);
    assert!((b.bare.i - a.bare.i_phi).abs() < 1e-12, "{} {}", b.bare.i, a.bare.i_phi);
    assert!((b.valve.i - a.valve.i_phi).abs() < 1e-12, "{} {}", b.valve.i, a.valve.i_phi);
    assert!(b.erosion_fuel > 20.0, "rung 66's shared-variable erosion");
    assert!(a.erosion_valve * 10.0 < b.erosion_fuel);
    // THE SHARPEST FORM: the loop that does NOT watch phi buys MORE phi at the margin than the
    // loop that does, while delivering far less alone.
    let marg_a = a.credit_phi_both - a.credit_phi_valve;
    assert!(marg_a > b.marginal_fuel && b.marginal_fuel > 0.0, "{} {}", marg_a, b.marginal_fuel);
    assert!(a.credit_phi_gov < b.credit_fuel / 2.0, "{} {}", a.credit_phi_gov, b.credit_fuel);
}

// =============================================================================================
// GATE 9 — rung 66 § 8's concession. BOTH branches were pre-registered; the answer took one
//          each, so the gate watches the SPLIT.
// =============================================================================================

/// Rung 66's violation-integral spread, recomputed from its own returned cells so the two rungs'
/// numbers come off the same definition — Python's `_b_integral_spread`.
fn b_integral_spread(b: &turbojet::two_lag::MarginalModeCascade) -> f64 {
    (b.moved_hi.i - b.moved_lo.i).abs() / b.natural.i
}

#[test]
fn the_b0_spread_splits_on_a_non_degenerate_pair() {
    let des = design();
    let a = cross(&des, &LeverArm::floored(valve(Some(TAU))))
        .marginal_mode_cross(&flight(), &ramp(DS), TMAX, TAU, TAU_GOV, D_B0);
    let b = cas66(&des, &LeverArm::default())
        .marginal_mode_cascade(&flight(), &ramp(DS), sm(), B, TAU, TAU, REL_MULT, D_B0);
    // (i) THE WITHHELD FUEL: rung 66's 84 % collapses by orders of magnitude => that spread WAS
    //     the zero eigenvalue, and rung 66 § 8's concession is discharged for it.
    assert!(b.dremoved_rel > 0.5, "rung 66's degenerate spread");
    assert!(a.dremoved_rel < 0.01, "{}", a.dremoved_rel);
    assert!(b.dremoved_rel / a.dremoved_rel > 100.0);
    // (ii) THE VIOLATION INTEGRAL: it SURVIVES on a pair with no marginal direction, so it was
    //      ordinary transient sensitivity. That half INVERTS.
    //
    //      THE GATE IS THE RATIO, AND THE FIRST VERSION OF IT WAS A TAUTOLOGY. Asserting
    //      `a > 0.9 * b` next to `a > 0.3` sets a threshold of 0.366 against a measured 0.455 —
    //      it would pass on a spread that had SHRUNK by 10 %, which is the opposite finding.
    //      The claim is a 12 % gap (45.50 % vs 40.75 %), so the gap is what must be watched.
    assert!(a.d_i_phi_rel > 0.3, "{}", a.d_i_phi_rel);
    let bs = b_integral_spread(&b);
    assert!(0.35 < bs && bs < 0.45, "rung 66's reference spread moved: {bs}");
    assert!(a.d_i_phi_rel / bs > 1.05, "{} {}", a.d_i_phi_rel, bs);
    // the natural march must actually ride, or the instrument is measuring nothing
    assert!(a.natural.n_on > 50);
}

// =============================================================================================
// GATE 10-11 — the inherited floor, and the joint IC where rung 66's could not run.
// =============================================================================================

/// Rung 66's `ds*(1/t_g + 1/t_v) <= 2` is derived from `det J == 0`. Here `det J != 0`, so the
/// radius is `sqrt(det)` and the sum OVERBOUNDS it — by the derived 2x at matched clocks. A floor
/// derived from an identity is conservative wherever the identity does not hold.
#[test]
fn the_inherited_sum_floor_is_safe_but_no_longer_the_radius() {
    let (des, fl) = (design(), flight());
    let m = cross(&des, &LeverArm::floored(valve(Some(TAU))));
    let (mf_lo, mf_hi, nu0) = hand_ramp(&m.fuel);
    let run = |ds: f64| {
        let _ = m.fuel.integrate_fuel(&fl, sched_of(mf_lo, mf_hi), nu0, 0.1, ds, &gov_lim());
    };
    let e = message(|| run(0.15));
    assert!(e.contains("RK4 stability region"), "{e}");
    // and the sum is the SUM, not the fastest clock: this step passes rung 65's floor
    // (ds/tau = 1.6 < 2) and must still be refused.
    let e = message(|| run(0.08));
    assert!(e.contains("RK4 stability region"), "{e}");
}

#[test]
fn the_sum_bound_is_measured_conservative_not_assumed() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let d = m.cross_identity(&flight(), &ramp(DS), TMAX, TAU, &TAU_GOVS, 8);
    assert!(d.sum_always_safe);
    for row in &d.rows {
        assert!(row.sum_conservative > 1.05, "{row:?}");
    }
    let matched = d.rows.iter().find(|x| (x.rho_clock - 1.0).abs() < 1e-9)
                   .expect("a matched-clock row");
    assert!(1.9 < matched.sum_conservative && matched.sum_conservative < 2.1,
            "the derived 2x at matched clocks: {}", matched.sum_conservative);
}

/// On the anchored plant `|P| ~ 0.02` converges undamped in 1-2 iterations, so the damped retries
/// are code that never runs there — untested guard code, which is a liability and not a safeguard.
/// Fed synthetic laws with a chosen `P` the ladder is exercised directly: the composite multiplier
/// is `(1-w) + wP`, so `w = 1` handles `|P| < 1`, `w = 1/2` up to 3, `w = 1/4` up to 7.
///
/// IT ALSO PINS THE THING RUNG 66's MESSAGE GOT WRONG FOR THIS CASCADE: a stall is a SOLVER
/// failure, because `det J != 0` means the equilibrium is unique at every `P != 1`.
#[test]
fn the_damped_ic_fallback_is_exercised_not_merely_shipped() {
    let (g_star, q_star) = (3.0e-3, 0.04);
    let a = 1.0e-3;
    // `required(q)` and `command(g)` linear about (g*, q*) with `dR/dq * dC/dg == P`.
    let required_of = |q: f64| g_star + a * (q - q_star);
    // THE CAP PARTICIPATES IN THE CHOICE, which is not obvious and is why this is measured:
    // `P = -0.9` contracts undamped (|P| < 1) but at 0.9 per iteration, needing ~218 to reach
    // 1e-12 from a 1e-2 offset — past the 60 cap — so the ladder drops to w = 1/2 (multiplier
    // 0.05) and converges immediately. A slow contraction is damped exactly like a divergent one,
    // and the outcome is the same equilibrium either way.
    for (p, w_expected) in [(-0.02, 1.0), (-0.5, 1.0), (-0.9, 0.5), (-2.0, 0.5), (-5.0, 0.25)] {
        let command_of = |g: f64| q_star + (p / a) * (g - g_star);
        let r = joint_fixed_point(&required_of, &command_of, q_star + 0.01, false, IC_TOL, IC_CAP);
        assert!(r.res <= 1e-9, "{p} {} {}", r.res, r.w);
        assert!((r.g - g_star).abs() < 1e-9 && (r.q - q_star).abs() < 1e-7, "{p} {} {}", r.g, r.q);
        assert_eq!(r.w, w_expected, "P = {p}");
        assert!(r.its <= IC_CAP);
    }
    // the UNDAMPED path must be untouched by the ladder's existence: w = 1 is the identity
    let command_of = |g: f64| q_star + (-0.02 / a) * (g - g_star);
    assert_eq!(joint_fixed_point(&required_of, &command_of, q_star + 0.01, false, IC_TOL, IC_CAP).w,
               1.0);
    // and `fix_q` (rung 65/66's `b0` override) holds the valve while the clip still solves
    let r = joint_fixed_point(&required_of, &command_of, 0.055, true, IC_TOL, IC_CAP);
    assert_eq!(r.q, 0.055);
    assert!(r.res <= 1e-9 && (r.g - required_of(0.055)).abs() < 1e-9, "{r:?}");
}

/// Rung 66's joint solve converged only because every start it tried opened DORMANT; its
/// contraction is pinned at 1 by its identity. Here it is `|P| ~ 0.02`, and starts with the fuel
/// leg LIVE at `s = 0` exist and converge in a couple of iterations.
#[test]
fn the_joint_ic_converges_where_rung66s_could_not_be_exercised() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let d = m.joint_ic_corners(&flight(), &ramp(DS), &[1150.0, 1300.0], &[1000.0, 1200.0],
                               TAU, TAU_GOV);
    assert!(d.all_converged && !d.ever_damped);
    assert!(d.max_iters <= 4, "{:?}", d.rows);
    assert!(d.n_live >= 1, "no corner opened with the governor engaged — P7 is unexercised");
    // Not Python's, and not decoration: `all_converged` is vacuously true over a row set that is
    // all `Failed`, so the FOUR corners are counted as OK ones. Python's `max_iters` reads a key
    // only an OK row has and would `KeyError`; Rust's enum would silently skip.
    assert_eq!(d.rows.iter().filter(|r| matches!(r, IcCorner::Ok(_))).count(), 4);
}

// =============================================================================================
// GATE 12 — the repaired instrument. The defect was a grid artefact that reads exactly like slow
//           convergence.
// =============================================================================================

/// A synthetic point carrying ONLY the two fields [`exceed`] reads. The rest are `NaN` rather
/// than zero, so a port that started reading a third field returns `NaN` and fails loudly instead
/// of returning a plausible number — Python's dicts simply have no other key, and `NaN` is the
/// nearest thing to a `KeyError` a struct can offer.
fn pt(s: f64, tt4: f64) -> FuelPoint {
    FuelPoint {
        s, tt4, nu_lp: f64::NAN, nu_hp: f64::NAN, f: f64::NAN, pi_lpc: f64::NAN,
        pi_hpc: f64::NAN, phi_lp: f64::NAN, phi_hp: f64::NAN, mdot_air: f64::NAN,
        sp_thrust: f64::NAN, branch: Branch::Choked, mf: f64::NAN, mf_sched: f64::NAN,
        extra: PointExtra::None,
    }
}

/// `violation` breaks on `s > s_hi`, dropping the straddling cell — immaterial on an early-ramp
/// currency, worth `ds * peak` on a temperature one. `exceed` interpolates it. Checked on a
/// synthetic ramp where the answer is exact by hand.
#[test]
fn the_exceedance_integral_does_not_drop_its_final_cell() {
    let traj: Vec<FuelPoint> =
        (0..8).map(|i| pt(i as f64 * 0.1, 1000.0 + 100.0 * i as f64)).collect();   // 1000..1700
    // over [0, 0.5] with Tt4_max = 1000: integrand 0, 100, 200, 300, 400, 500 -> area 125.0
    assert!((exceed(&traj, 1000.0, 0.5) - 125.0).abs() < 1e-9);
    // a limit that STRADDLES a cell must be interpolated, not dropped
    assert!((exceed(&traj, 1000.0, 0.55) - 151.25).abs() < 1e-9);
    // and a float's width past a grid point must not lose a whole cell
    let a = exceed(&traj, 1000.0, 0.5);
    let b = exceed(&traj, 1000.0, 0.5 * (1.0 + 1e-15));
    assert!((a - b).abs() < 1e-9);
}

#[test]
fn the_headline_numbers_are_grid_converged() {
    let m = cross(&design(), &LeverArm::floored(valve(Some(TAU))));
    let fl = flight();
    let out: Vec<[f64; 4]> = [0.01, 0.005, 0.0025].iter().map(|&ds| {
        let idt = m.cross_identity(&fl, &ramp(ds), TMAX, TAU, &[TAU], 6);
        let bill = m.cross_bill(&fl, &ramp(ds), TMAX, TAU, TAU_GOV);
        [idt.rows[0].p_mid, bill.both.i_t, bill.both.i_phi, bill.credit_t_gov]
    }).collect();

    let spread = |i: usize| -> f64 {
        let v: Vec<f64> = out.iter().map(|x| x[i]).collect();
        let (lo, hi) = (v.iter().cloned().fold(f64::INFINITY, f64::min),
                        v.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
        (hi - lo).abs() / (v.iter().sum::<f64>() / v.len() as f64).abs()
    };
    assert!(spread(0) < 0.01, "P");
    assert!(spread(1) < 0.01, "I_T -- the repaired integral (2.8 % before the fix)");
    assert!(spread(2) < 0.01, "I_phi");
    assert!(spread(3) < 0.01, "credit_T");
}
