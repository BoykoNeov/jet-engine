//! RUNG 66 — THE TWO-LAG CASCADE: a lagged bleed VALVE beside a lagged FUEL leg.
//!
//! Slice Z step 3. Python's `tests/test_rung66.py`, all **15** collected gates, in its order and
//! under its names.
//!
//! # THE HEADLINE
//!
//! Two loops on one variable are ONE loop with the RATES ADDED. Two control laws holding the same
//! variable to the same set point have `R_q * C_g == 1` IDENTICALLY — both are implicit functions
//! of the same constraint `phi(w, b) = phi_lim` — hence `det J == 0` at every point, bandwidth and
//! plant, and the spectrum is exactly `{0, -(1/t_g + 1/t_v)}`. A second limiter on the same
//! variable buys BANDWIDTH, NOT AUTHORITY.
//!
//! # WHAT ALL FIFTEEN GREEN DOES *NOT* COVER, STATED HERE SO GREEN IS NOT READ AS COVERAGE
//!
//! § 5.24 (vi), measured by probe 7 over `test_rung66.py` alone: [`eig`]'s COMPLEX arm runs **0
//! of 80** times on this suite. That is rung 66's own headline doing it — `det J ≡ 0` makes the
//! discriminant `tr² − 4·0 = tr² ≥ 0` identically — so the arm is DEAD on the rung that defines
//! it and live only one rung up. **A port that deleted it passes every gate in this file.** It is
//! P5's, gated by a direct call at step 5, not by anything here.
//!
//! Two more arms are dead on this grid and are step 5's for the same reason (§ 5.24 (v)):
//! [`gains`](turbojet::stator_transient::ScheduledStatorCore::gains) with `accel` armed (0 of 80)
//! and `gains` with NEITHER leg (0 of 80).
//!
//! # AND [`violation`]'s DROPPED CELL IS NOT SMALL, IT IS EXACTLY ZERO — MEASURED AT STEP 3
//!
//! `violation` breaks on `traj[i].s > s_hi` and its own doc comment calls the resulting dropped
//! cell *"immaterial — the phi violation is an EARLY-ramp object and its integrand is ~0 by
//! `s = r`"*. Step 3's injection I8 gave `violation` rung 67's lower guard instead (the cell taken
//! whole) and **not one of the 62 gates in this slice moved.** The probe says why, and it is
//! sharper than the docstring: at `ds = 0.01 / 0.005 / 0.0025` the accumulated `s` lands at
//! `5.00000000000000222e-1`, so the straddling cell **is** real and **is** dropped — but
//! `phi_lim − phi_lp` is `−5.2565e-3` at both of its ends, so `max(0, ·)` clamps the added area to
//! **exactly `0.0`, bit-for-bit, at all three grids.** Not approximately immaterial: identically
//! zero, and by the CLAMP rather than by decay. So the *"two functions, never one with a flag"*
//! rule is protected by nothing on the shipped grid, and no oracle at the suites' own grid can
//! reach it either — it is step 5's, on a constructed trajectory.
//!
//! # THE `slow` BILL
//!
//! Python marks **5 of 15** and runs the file in **31.46 s** (§ 5.24 (ix)). Slice M's rule
//! applies unchanged: port the gate, DROP the marker, re-introduce `#[ignore]` only against a
//! MEASURED Rust number. The measured cost is in the step-3 write-up.
//!
//! [`eig`]: turbojet::two_lag::eig
//! [`violation`]: turbojet::two_lag::violation

use std::panic::catch_unwind;

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AsymmetricLag, Floor, FuelLimiters, FuelPoint, PointExtra,
                               SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                                 StatorLeg};
use turbojet::two_lag::build_two_lag_cascade;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------------- the grid
// Python's module constants, name for name.
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
/// The valve clock.
const TAU: f64 = 0.05;
/// The fuel leg's — rung 52's fast-attack / slow-release pair.
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;

/// Python's `merge_identity` default `tau_rels`.
const TAU_RELS: [f64; 3] = [0.15, 0.30, 0.60];
/// Python's `cascade_identity` default `tau_atts`.
const TAU_ATTS: [f64; 3] = [0.005, 0.05, 0.5];
/// Python's default `rel_mult` on all four readers.
const REL_MULT: f64 = 3.0;
/// Python's `cascade_identity` default `n_sample`. **A REQUEST, not the delivered count** — the
/// sub-sample is a stride, which is § 5.24 (i)'s leading finding.
const N_SAMPLE: usize = 12;
/// Python's `marginal_mode_cascade` default `d_b0`.
const D_B0: f64 = 0.01;

/// Python's `SM = PHI / FLOOR - 1.0`, spelled as the expression rather than as a decimal so a
/// rounded literal cannot drift the surge floor away from the valve's.
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

/// Python's `_cas(...)` — a rung-66 machine. The `design` argument is threaded exactly where
/// Python threads it, because two machines built from two `_design()` calls are two engines and a
/// bit-for-bit comparison between them would be asserting about the builder.
fn cas(design: &TwoSpoolEngine, arm: &LeverArm) -> ScheduledStatorCore {
    full(build_two_lag_cascade(design.clone(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                               arm))
}

/// Python's `_lag65(...)` — the rung-65 reference machine.
fn lag65(design: &TwoSpoolEngine, arm: &LeverArm) -> ScheduledStatorCore {
    full(build_lagged_bleed(design.clone(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                            arm))
}

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

/// Python's `_valve(tau)`.
fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }

/// Python's `_fuel()` — the rung-49 phi floor the fuel lag lags.
fn fuel() -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm()) }

/// Python's `_lag(att, rel)`.
fn lag(att: f64, rel: f64) -> AsymmetricLag { AsymmetricLag::new(att, rel) }

/// The default `_lag()`.
fn lag_d() -> AsymmetricLag { lag(TAU_ATT, TAU_REL) }

/// A leg with the fuel floor armed — Python's `surge=_fuel()`.
fn surge_leg() -> StatorLeg<'static> {
    StatorLeg { accel: None, surge: Some(Floor::Phi(fuel())), tt4_max: None }
}

/// Python's `_keys(traj)` — compared on RAW `f64`, which is bit-for-bit here because every value
/// on both sides comes off the same arithmetic and `assert_eq!` on `f64` is exact equality. (A
/// `NaN` would make that vacuously false rather than vacuously true, which is the safe direction.)
fn keys(t: &[FuelPoint]) -> Vec<(f64, f64, f64, f64, f64, f64, f64)> {
    t.iter()
        .map(|p| (p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.tt4, p.mf))
        .collect()
}

/// Run `f` and return the panic message it produced. Panics itself if `f` does not — Python's
/// `pytest.raises` fails the same way.
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

/// Python's `_ramp`-by-hand for a direct `integrate_fuel` — the schedule `_stator_march` builds
/// internally, spelled out because gates 3, 6, 7 and 8 call the integrator directly.
fn hand_ramp(m: &ScheduledStatorCore) -> (f64, f64, (f64, f64)) {
    let (fl, mf_lo, mf_hi) = (flight(), m.fuel.fuel_for_tt4(&flight(), LO),
                              m.fuel.fuel_for_tt4(&flight(), HI));
    let eq = m.fuel.inner.equilibrium(&fl, LO);
    (mf_lo, mf_hi, (eq.nu_lp, eq.nu_hp))
}

// =============================================================================================
// GATE 1 — THE REDUCE, all three bit-for-bit arms. The merged integrator is entered ONLY when
//          BOTH clocks are armed; every other combination must reach the SAME code path it
//          always did.
// =============================================================================================

/// `tau=None` and `lag=None`: rung 64's arm, inherited through rung 65.
#[test]
fn reduce_no_lags_is_rung64_bit_for_bit() {
    let des = design();
    let bare = StatorLeg { accel: None, surge: None, tt4_max: None };
    for arm in [LeverArm::default(), LeverArm::constant(B),
                LeverArm::scheduled(BleedSchedule::new(B, 0.65)),
                LeverArm::floored(valve(None))] {
        let (a, _) = cas(&des, &arm).stator_march(&flight(), &ramp(DS), None, &bare);
        let (b, _) = lag65(&des, &arm).stator_march(&flight(), &ramp(DS), None, &bare);
        assert_eq!(keys(&a), keys(&b), "arming mode {arm:?}");
    }
}

/// `tau` set, `lag=None`: the merged integrator is NOT entered and the state count is 3. This is
/// the arm that would break first if `stator_march`'s `lag` plumbing leaked a default through.
#[test]
fn reduce_valve_lag_alone_is_rung65_bit_for_bit() {
    let des = design();
    let arm = LeverArm::floored(valve(Some(TAU)));
    for leg in [StatorLeg { accel: None, surge: None, tt4_max: None }, surge_leg()] {
        let (a, _) = cas(&des, &arm).stator_march(&flight(), &ramp(DS), None, &leg);
        let (b, _) = lag65(&des, &arm).stator_march(&flight(), &ramp(DS), None, &leg);
        assert_eq!(keys(&a), keys(&b));
        // Python's `"g" not in a[0]`. The Rust spelling is stronger than an absence test: the
        // ROUTE is named, so a trajectory carrying `g` under a different name cannot pass.
        assert!(matches!(a[0].extra, PointExtra::Valve { .. }),
                "rung 65's arm must not carry a fourth state, got {:?}", a[0].extra);
    }
}

/// `tau=None`, `lag` set: rung 52's integrator, state count 3, the OTHER three. Dispatch leaves
/// through the same `super().integrate_fuel(..., lag=lag)` a rung-65 machine uses, so the
/// reference is a rung-65 machine with no valve.
#[test]
fn reduce_fuel_lag_alone_is_rung52_bit_for_bit() {
    let des = design();
    let (fl, arm) = (flight(), LeverArm::default());
    let (a, _) = cas(&des, &arm).stator_march_scoped(
        &fl, &ramp(DS), None, &surge_leg(),
        &MarchScope { lag: Some(lag_d()), ..MarchScope::DEFAULT });

    let m = lag65(&des, &arm);
    let (mf_lo, mf_hi, nu0) = hand_ramp(&m);
    let sched = |s: f64| -> f64 {
        if s <= 0.0 { mf_lo } else if s >= R { mf_hi } else { mf_lo + (mf_hi - mf_lo) * (s / R) }
    };
    let b = m.fuel.integrate_fuel(
        &fl, sched, nu0, R + SETTLE, DS,
        &FuelLimiters { surge: Some(fuel()), lag: Some(lag_d()), ..Default::default() });

    assert_eq!(keys(&a), keys(&b));
    // Python's `"b" not in a[0]` and `"g" in a[0]`, in one named-route assertion.
    assert!(matches!(a[0].extra, PointExtra::Asym { .. }),
            "rung 52's arm must not carry a valve state, got {:?}", a[0].extra);
}

/// Only BOTH armed enters the merged integrator, and it carries ALL FOUR of rung 52's and rung
/// 65's per-point keys, so every reader of either rung works unchanged on it.
#[test]
fn the_cascade_is_the_only_four_state_path() {
    let (t, _) = cas(&design(), &LeverArm::floored(valve(Some(TAU)))).stator_march_scoped(
        &flight(), &ramp(DS), None, &surge_leg(),
        &MarchScope { lag: Some(lag_d()), ..MarchScope::DEFAULT });
    // Python loops `for k in ("g", "required", "b", "b_cmd")`. In Rust the four live in ONE
    // variant, so the named-variant match IS the four-key assertion — and it additionally
    // refuses a route that carries four keys under some other name.
    assert!(matches!(t[0].extra, PointExtra::Cascade { .. }),
            "the merged integrator did not run: {:?}", t[0].extra);
    assert_eq!(t[0].key_count(), 20, "rung 66's dict is the fourteen plus six");
}

// =============================================================================================
// GATE 2 — P6, THE MERGE VALIDATOR. Rung 52's structural fact must SURVIVE the merge: `tau_rel`
//          is never read while `required > g`, so the whole pre-crossing march is BIT-IDENTICAL
//          across a release-rate sweep. A MISS here is a BUG (a leaked `_b_state` boundary or a
//          leg reading the wrong constant), not a finding.
// =============================================================================================

#[test]
fn the_release_constant_is_unread_before_the_crossing() {
    let out = cas(&design(), &LeverArm::floored(valve(Some(TAU)))).merge_identity(
        &flight(), &ramp(DS), sm(), B, TAU, TAU_ATT, &TAU_RELS);
    let crossing = out.crossing.expect("the sweep needs a crossing to be about anything");
    assert!(out.ok, "{:?}", out.rows);
    assert!(out.rows[0].identical, "the reference against itself must be identical");
    for row in &out.rows[1..] {
        let fd = row.first_diff.unwrap_or_else(|| panic!("{row:?} never departed"));
        assert!((fd as i64 - crossing as i64).abs() <= 1, "{row:?} vs crossing {crossing}");
    }
}

// =============================================================================================
// GATE 3 — THE REFUSALS. Cascade A (rung 47's Tt4 governor) is a DIFFERENT rung with opposite
//          cross-gain signs; rungs 50/51's forced edges measure the forcing on legs that pin
//          their own triggers.
// =============================================================================================

/// Cascade A is rung 47's LAGGED governor (`tau_gov`), whose cross-gains have OPPOSITE signs and
/// which therefore admits the oscillatory actuator mode B provably cannot. The INSTANTANEOUS
/// redline (`Tt4_max` alone) is a different object and composes fine — rung 52's own precedent —
/// so it must NOT be refused.
#[test]
fn cascade_a_is_refused() {
    let (des, fl) = (design(), flight());
    let m = cas(&des, &LeverArm::floored(valve(Some(TAU))));
    let (mf_lo, _, nu0) = hand_ramp(&m);
    let e = message(|| {
        let _ = m.fuel.integrate_fuel(
            &fl, |_s| mf_lo, nu0, R + SETTLE, DS,
            &FuelLimiters { surge: Some(fuel()), lag: Some(lag_d()), tt4_max: Some(1500.0),
                            tau_gov: Some(0.05), ..Default::default() });
    });
    // Python's `match="cascade A|CASCADE B"`.
    assert!(e.contains("cascade A") || e.contains("CASCADE B"), "{e}");

    // …and the instantaneous redline runs, on rung 52's placement (clipped fuel first).
    let leg = StatorLeg { accel: None, surge: Some(Floor::Phi(fuel())), tt4_max: Some(1500.0) };
    let (t, _) = m.stator_march_scoped(&fl, &ramp(DS), None, &leg,
                                       &MarchScope { lag: Some(lag_d()), ..MarchScope::DEFAULT });
    assert!(t.len() > 10);
    assert!(matches!(t[0].extra, PointExtra::Cascade { .. }), "{:?}", t[0].extra);
}

#[test]
fn forced_release_edges_are_refused() {
    let (des, fl) = (design(), flight());
    let m = cas(&des, &LeverArm::floored(valve(Some(TAU))));
    let (mf_lo, _, nu0) = hand_ramp(&m);
    let e = message(|| {
        let _ = m.fuel.integrate_fuel(
            &fl, |_s| mf_lo, nu0, R + SETTLE, DS,
            &FuelLimiters { surge: Some(fuel()), lag: Some(lag_d()), s_off: Some(0.3),
                            ..Default::default() });
    });
    assert!(e.contains("FORCED release"), "{e}");
}

#[test]
fn a_lag_with_no_leg_is_refused() {
    let (des, fl) = (design(), flight());
    let m = cas(&des, &LeverArm::floored(valve(Some(TAU))));
    let (mf_lo, _, nu0) = hand_ramp(&m);
    let e = message(|| {
        let _ = m.fuel.integrate_fuel(
            &fl, |_s| mf_lo, nu0, R + SETTLE, DS,
            &FuelLimiters { lag: Some(lag_d()), ..Default::default() });
    });
    assert!(e.contains("min-select LEG"), "{e}");
}

// =============================================================================================
// GATE 4 — THE IDENTITY. `R_q * C_g == 1` because both laws are implicit functions of the SAME
//          constraint. Measured on the shipped closures, which do not know about each other.
// =============================================================================================

/// THE RUNG. `R_q = phi_b/phi_w` and `C_g = phi_w/phi_b` by implicit differentiation of one
/// constraint, so their product is 1 independently of plant, gains and bandwidths.
///
/// THE CONTROL IS `gain_span`: a constant product is evidence of a reciprocal pair only if the
/// INDIVIDUAL gains move. They move by ~1.4–1.8x over the same march while the product holds to a
/// few percent.
///
/// **AND THE PRODUCT IS NOT A SELF-COMPARISON HERE, WHICH IS THE ONE THING THIS GATE COULD HAVE
/// BEEN.** `R_q` comes off `try_sched_fuel`/`try_surge_fuel` by central difference in the valve
/// POSITION; `C_g` comes off `r64_solve_b` by central difference in the fuel CLIP. Two different
/// closures, two different step sizes, neither reading the other — so `prod == 1` is a
/// measurement of the identity and not an algebraic restatement of it.
#[test]
fn the_cross_gains_are_reciprocals() {
    let out = cas(&design(), &LeverArm::floored(valve(Some(TAU)))).cascade_identity(
        &flight(), &ramp(DS), sm(), B, TAU, &TAU_ATTS, REL_MULT, N_SAMPLE);
    assert!(0.94 < out.prod_lo && out.prod_hi < 1.06, "{} {}", out.prod_lo, out.prod_hi);
    for row in &out.rows {
        assert!(row.n_ride > 50, "{row:?}");
        assert!(row.gain_span_r > 1.2, "{row:?}");          // the gains MOVE…
        assert!(row.gain_span_c > 1.2, "{row:?}");
        assert!(0.94 < row.prod_lo && row.prod_hi < 1.06, "{row:?}");   // …the product does not
        // and BOTH are strictly negative, which is what makes them SUBSTITUTING loops
        assert!(row.r_q_hi < 0.0 && row.c_g_hi < 0.0, "{row:?}");
    }
}

/// `det J == 0` makes the spectrum exactly `{0, -(1/t_g + 1/t_v)}`: REAL for a stronger reason
/// than the anchor's sign argument, and the non-zero root is the SUM OF THE RATES.
///
/// Measured against the closed form at three clock ratios spanning 100x.
///
/// **`all_real` IS STRUCTURAL ON THIS RUNG AND THE GATE IS THEREFORE HALF A TAUTOLOGY**, which is
/// said rather than hidden: `det ≡ 0` makes the discriminant `tr² ≥ 0` identically, so the first
/// assertion cannot fail unless the port computed `det` wrong. `rho_err` is the half with content
/// — it compares a measured eigenvalue against a closed form built from the two clocks alone.
#[test]
fn the_eigenvalues_are_real_and_the_rates_add() {
    let out = cas(&design(), &LeverArm::floored(valve(Some(TAU)))).cascade_identity(
        &flight(), &ramp(DS), sm(), B, TAU, &TAU_ATTS, REL_MULT, N_SAMPLE);
    assert!(out.all_real, "{:?}",
            out.rows.iter().map(|r| (r.tau_att, r.n_real, r.n_sample)).collect::<Vec<_>>());
    for row in &out.rows {
        assert!(row.rho_err < 0.05, "{row:?}");   // |lambda| vs 1/t_g + 1/t_v
    }
}

// =============================================================================================
// GATE 5 — WHAT THE PAIR DELIVERS. `det J == 0` means ONE effective actuator, so the second loop
//          buys the RATE and not the AUTHORITY: the pair beats both singles yet its credit is
//          strongly SUB-ADDITIVE.
// =============================================================================================

#[test]
fn a_second_limiter_buys_bandwidth_not_authority() {
    let out = cas(&design(), &LeverArm::floored(valve(Some(TAU)))).cascade_bill(
        &flight(), &ramp(DS), sm(), B, TAU, TAU_ATT, REL_MULT);
    assert!(out.beats_both, "{} {} {}", out.credit_fuel, out.credit_valve, out.credit_both);
    assert!(out.subadditive, "{} {} {}", out.credit_fuel, out.credit_valve, out.credit_both);
    // the two standalone credits OVER-PREDICT the pair by more than half again
    assert!(out.sum_alone > 1.4 * out.delivered, "{} {}", out.sum_alone, out.delivered);
    // THE HEADLINE NUMBER: a whole second limiter on top of the stronger one buys almost
    // nothing. ONE-SIDED — the spec disclaims the magnitude, so an upper bound would gate the
    // grid and not the finding. Measured 38.1x at these settings.
    assert!(out.erosion_fuel > 10.0, "{}", out.erosion_fuel);
    assert!(out.marginal_fuel < 0.05, "{}", out.marginal_fuel);
    // and the direction is ASYMMETRIC: the stronger loop eroded far less
    assert!(out.erosion_valve < out.erosion_fuel, "{} {}", out.erosion_valve, out.erosion_fuel);
}

/// WHY `min phi` is unusable, asserted so the choice cannot be quietly undone: on the
/// fuel-leg-alone control the argmin sits at the FIRST point off the running line, so the number
/// is the initial condition and not a protected minimum. That cell's march also truncates. An
/// area cannot be clamped by its own initial condition.
#[test]
fn the_currency_had_to_be_the_integral() {
    let out = cas(&design(), &LeverArm::floored(valve(Some(TAU)))).cascade_bill(
        &flight(), &ramp(DS), sm(), B, TAU, TAU_ATT, REL_MULT);
    assert!(out.fuel.s_at_min <= 2.0 * DS, "{:?}", out.fuel);
    assert!(out.fuel.truncated, "{:?}", out.fuel);
    // the valve and the pair are NOT clamped — their minima are interior
    for (name, c) in [("valve", out.valve), ("both", out.both)] {
        assert!(c.s_at_min > 10.0 * DS, "{name}: {c:?}");
        assert!(!c.truncated, "{name}: {c:?}");
    }
}

// =============================================================================================
// GATE 6 — THE CORRECTION TO RUNG 65. Its own `b0` instrument, verbatim, on a plant whose second
//          loop also has a clock: the FROZEN STATE is gone while the degeneracy is not. The
//          freeze was the MANIFOLD.
// =============================================================================================

#[test]
fn the_frozen_state_is_gone_but_the_initial_condition_still_bites() {
    let out = cas(&design(), &LeverArm::floored(valve(Some(TAU)))).marginal_mode_cascade(
        &flight(), &ramp(DS), sm(), B, TAU, TAU_ATT, REL_MULT, D_B0);
    // (i) rung 65 measured drift EXACTLY 0 and db_end/db0 EXACTLY 1.0
    assert!(out.frozen > 1e-2, "{}", out.frozen);
    assert!(out.washed_out, "{}", out.db_db0);
    // (ii) the state is genuinely OFF-manifold — neither law is satisfied instantaneously
    assert!(out.track_b > 1e-3, "{}", out.track_b);
    assert!(out.track_g > 1e-6, "{}", out.track_g);
    // (iii) …and the initial condition is STILL load-bearing on the OUTCOME
    assert!(out.dremoved_rel > 0.2, "{}", out.dremoved_rel);
}

// =============================================================================================
// GATE 7 — THE MODELLING FLOOR, and it is the artifact that would have counterfeited the rung.
//          THE RATES ADD, so the naive transfer of rung 65's constant (bound the FASTEST clock)
//          is optimistic by up to 2x — wrong in the UNSAFE direction. Rung 65 published a
//          retraction for exactly this failure mode at one state; here there are two.
// =============================================================================================

#[test]
fn the_stability_floor_counts_the_sum_of_the_rates() {
    let (des, fl) = (design(), flight());
    let m = cas(&des, &LeverArm::floored(valve(Some(0.01))));
    let (mf_lo, _, nu0) = hand_ramp(&m);
    let run = |ds: f64, att: f64| {
        let _ = m.fuel.integrate_fuel(
            &fl, |_s| mf_lo, nu0, R + SETTLE, ds,
            &FuelLimiters { surge: Some(fuel()), lag: Some(lag(att, 3.0 * att)),
                            ..Default::default() });
    };
    // ds/min(tau) = 0.9 passes EITHER bound — ds*(1/t_v + 1/t_g) = 1.8, inside the sum too.
    // (Deliberately NOT ds = 0.01, which lands the sum on 2.0 exactly: a float knife-edge
    //  against a `<=` assert is a flake, not a gate.)
    run(0.009, 0.01);
    // …and one step past the SUM it is refused, where the naive bound still reads 1.2
    let e = message(|| run(0.012, 0.01));
    assert!(e.contains("RATES ADD"), "{e}");
}

/// The pair MISSES the floor, and the number is real: `-6.9e-3`, stable across a 4x `ds` range.
/// Rung 65's retraction was a plausible magnitude that was a step-size artifact, so this rung
/// refuses to quote one that has not been halved.
#[test]
fn the_grid_converged_undershoot_is_not_a_step_size_artifact() {
    let m = cas(&design(), &LeverArm::floored(valve(Some(TAU))));
    let (fl, scope) = (flight(), MarchScope { lag: Some(lag_d()), ..MarchScope::DEFAULT });
    let mins: Vec<f64> = [0.01, 0.005, 0.0025].iter().map(|&ds| {
        let (t, _) = m.stator_march_scoped(&fl, &ramp(ds), None, &surge_leg(), &scope);
        t.iter().filter(|p| p.s > 0.0).map(|p| p.phi_lp).fold(f64::INFINITY, f64::min)
    }).collect();
    assert!(mins.iter().all(|&x| x < PHI - 5e-3), "{mins:?}");   // the floor IS undershot
    let (lo, hi) = (mins.iter().cloned().fold(f64::INFINITY, f64::min),
                    mins.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!(hi - lo < 1e-5, "{mins:?}");                          // and it is grid-converged
}
