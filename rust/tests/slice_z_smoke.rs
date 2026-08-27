//! SLICE Z — the SMOKE file. **Step 1's sections are A–D; step 2 adds the march sections.**
//!
//! Step 1 ships rungs 66 and 67's PLUMBING and REFUSALS with the two marches stubbed, and the
//! point of splitting it that way is that the plumbing is then GATE-ABLE rather than merely
//! compile-checked. Everything below runs against code that exists today:
//!
//! * **A — the reduce arms, bit-for-bit.** Rungs 66 and 67 with their clocks disarmed must march
//!   the rung-65 machine EXACTLY, key for key. That is § 5.24 P2's arms 2/3/5, and it is the half
//!   of P2 a stubbed march can still answer.
//! * **B — the seven refusals.** Rung 66 has three and rung 67 four, and each one is what keeps
//!   the rung from silently becoming a different one.
//! * **C — [`RINGS`], the one class attribute a gate reads directly.**
//! * **D — the two CARRIERS actually reach the read.** A `MarchScope` field that never lands on
//!   its `Cell`, or a `Cell` no cell ever reads, is invisible to every value key while the clocks
//!   are disarmed — so the proof is that arming the carrier ALONE reaches the step-2 stub.
//!
//! The hardware is `slice_y_smoke.rs`'s, unchanged, so a difference between the two files is a
//! difference between the rungs and not between two harnesses.

use std::panic::catch_unwind;

use turbojet::bleed_transient::LeverArm;
use turbojet::cross_loop::{build_cross_loop_cascade, RINGS};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AsymmetricLag, Floor, FuelLimiters, FuelPoint,
                              SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                                 StatorLeg};
use turbojet::two_lag::build_two_lag_cascade;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const R: f64 = 0.5;
const SETTLE: f64 = 1.2;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const TAU: f64 = 0.05;
const SM: f64 = 0.4545;
const DS: f64 = 0.01;

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
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn full(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("the LP spool is never disabled"),
    }
}

/// A rung-65 machine — the reduce's reference.
fn t65(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_lagged_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// A rung-66 machine.
fn t66(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_two_lag_cascade(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm))
}

/// A rung-67 machine.
fn t67(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_cross_loop_cascade(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                                  arm))
}

fn ramp() -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS } }

fn valve() -> BleedLimiter { BleedLimiter::with_tau(PHI, B, Some(TAU)) }

fn armed() -> LeverArm { LeverArm::floored(valve()) }

/// The rung-49 phi floor rung 66's fuel lag lags — and, disarmed, the leg every reduce arm marches
/// through.
fn surge_leg() -> StatorLeg<'static> {
    StatorLeg { accel: None, surge: Some(Floor::Phi(SurgeLimiter::from_margin(
        &lp_map(), Spool::Lp, SM))), tt4_max: None }
}

/// Every float a trajectory point publishes that the reduce arms must agree on, bit-for-bit.
fn keys(t: &[FuelPoint]) -> Vec<(f64, f64, f64, f64, f64, f64, f64, f64)> {
    t.iter().map(|p| (p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.tt4, p.mf, p.mf_sched))
        .collect()
}

/// **`AssertUnwindSafe` HERE IS THE GATE, NOT A WORKAROUND.** The compiler refuses a borrowed
/// `ScheduledStatorCore` across a `catch_unwind` boundary precisely because it holds interior
/// mutability — the `Cell`s the two new carriers live in. Asserting it is the claim
/// `d_both_guards_restore_through_the_unwind_their_own_refusal_causes` then goes on to CHECK:
/// that the machine is bit-for-bit the same machine on the far side of a panic.
fn panics<F: FnOnce()>(f: F) -> bool {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(std::panic::AssertUnwindSafe(f)).is_err();
    std::panic::set_hook(prev);
    out
}

// =============================================================================================
// A — THE REDUCE ARMS, BIT-FOR-BIT AND BY DISPATCH
// =============================================================================================

/// **§ 5.24 P2, the half a stubbed march can answer.** With both clocks disarmed a rung-66 and a
/// rung-67 machine must march their rung-65 parent EXACTLY — not to a tolerance, and not on a
/// summary: on every key of every point.
///
/// This is a real gate and not a compile check, because the three tables each swap a live cell on
/// the path: `stator_march` sets a carrier and rebuilds the scope, and `integrate_fuel` re-resolves
/// its argument against that carrier and forwards a REBUILT limiter set. Any of those three
/// re-spellings can move a number while the code still compiles.
#[test]
fn a_both_rungs_reduce_to_rung_65_bit_for_bit_with_the_clocks_disarmed() {
    let leg = surge_leg();
    let (a, nu_a) = t65(&armed()).stator_march(&flight(), &ramp(), None, &leg);
    let (b, nu_b) = t66(&armed()).stator_march(&flight(), &ramp(), None, &leg);
    let (c, nu_c) = t67(&armed()).stator_march(&flight(), &ramp(), None, &leg);
    // MEASURED, not guessed: 171 points, the same count `slice_y_smoke.rs` pins for this
    // hardware at this `ds`. A length bar exists so the key comparison below cannot pass
    // vacuously on two empty trajectories.
    assert_eq!(a.len(), 171, "the floored reduce grid");
    assert_eq!(keys(&a), keys(&b), "rung 66 with `lag` disarmed is not rung 65");
    assert_eq!(keys(&a), keys(&c), "rung 67 with `tau_gov` disarmed is not rung 65");
    assert_eq!(nu_a, nu_b, "rung 66's terminal speeds");
    assert_eq!(nu_a, nu_c, "rung 67's terminal speeds");
}

/// The same reduce on an UNFLOORED machine — the `_lagged() == false` arm, which is a DIFFERENT
/// branch of the same `if` and lands two rungs lower (rung 64 / rung 43).
///
/// Both arms are gated because the dispatch is `_lagged() && clock.is_some()`: a port that tested
/// only the clock would pass the gate above and fail here, and a port that tested only `_lagged()`
/// would do the reverse.
#[test]
fn a_both_rungs_reduce_to_rung_65_on_an_unfloored_machine_too() {
    let bare = LeverArm::default();
    let leg = surge_leg();
    let (a, _) = t65(&bare).stator_march(&flight(), &ramp(), None, &leg);
    let (b, _) = t66(&bare).stator_march(&flight(), &ramp(), None, &leg);
    let (c, _) = t67(&bare).stator_march(&flight(), &ramp(), None, &leg);
    // MEASURED: 68, and it differs from the floored 171 because an unfloored machine reaches
    // the ramp end sooner. Pinned rather than bounded so that a silent change of GRID cannot
    // be mistaken for a passing reduce.
    assert_eq!(a.len(), 68, "the unfloored reduce grid");
    assert_eq!(keys(&a), keys(&b), "rung 66 on an unfloored machine is not rung 65");
    assert_eq!(keys(&a), keys(&c), "rung 67 on an unfloored machine is not rung 65");
}

/// **THE SIBLING CONSTRUCTOR RETURNS ITS OWN CLASS**, which in Rust is: the sibling carries the
/// tables the parent's builder would not have installed.
///
/// Rungs 61–67 each hit this trap and the override is one word, so no value key on the returned
/// machine can see it — every number a bare sibling produces is its parent's. What CAN see it is
/// the sibling's behaviour on the cell the rung swaps, so all three siblings are handed the SAME
/// arming — a fuel lag and a governor clock at once — and asked to refuse it. Rungs 65, 66 and 67
/// each refuse, **in their own words**, and that is the discriminator.
#[test]
fn a_at_lever_hands_back_this_rungs_object_and_not_its_parents() {
    let both = FuelLimiters { lag: Some(lag()), tau_gov: Some(0.05), tt4_max: Some(HI),
                              surge: Some(floor()), ..Default::default() };
    let e65 = message_lim(&t65(&LeverArm::default()).at_lever(&armed()), &both);
    let e66 = message_lim(&t66(&LeverArm::default()).at_lever(&armed()), &both);
    let e67 = message_lim(&t67(&LeverArm::default()).at_lever(&armed()), &both);
    assert!(e65.contains("rung-65: a lagged VALVE beside a lagged FUEL leg"),
            "a rung-65 sibling must reach RUNG 65's refusal; got: {e65}");
    assert!(e66.contains("rung-66 takes CASCADE B"),
            "a rung-66 sibling must reach RUNG 66's refusal; got: {e66}");
    assert!(e67.contains("rung-67 is CASCADE A"),
            "a rung-67 sibling must reach RUNG 67's refusal; got: {e67}");
}

// =============================================================================================
// B — THE SEVEN REFUSALS
// =============================================================================================

fn lag() -> AsymmetricLag { AsymmetricLag::new(0.05, 0.20) }

fn floor() -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, SM) }

/// Run `f` and return the panic message it produced. Panics itself if `f` does not.
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

/// **THE REFUSALS ARE REACHED THROUGH `integrate_fuel` DIRECTLY, AND THAT IS A FINDING RATHER
/// THAN A CONVENIENCE.**
///
/// The first draft of this file armed them through `stator_march_scoped` and could not reach a
/// single one that names a knob from ANOTHER rung. The reason is structural: each rung's
/// `_stator_march` **consumes its own scope field and forwards the rest**, so a rung-66 march has
/// no `tau_gov` to forward at all, and a rung-67 march hands `lag` to rung 66's CARRIER — which
/// rung 67's own armed branch then never reads, because it returns before `super()`.
///
/// So on a rung-67 machine, arming the fuel lag through the march is **silently ignored**, and
/// Python does exactly the same thing: rung 67's `assert lag is None` reads the ARGUMENT, and the
/// carrier belongs to the rung below. Every refusal below therefore takes the only route that
/// reaches it — a direct call, which is also the route Python's own suites take.
fn message_lim(m: &ScheduledStatorCore, lim: &FuelLimiters<'_>) -> String {
    let (f, l) = (flight(), lim.clone());
    message(|| { let _ = m.fuel.integrate_fuel(&f, |_s| 0.02, (0.75, 0.79), 0.05, DS, &l); })
}

/// **RUNG 66's THREE.** Each is a claim about which rung this is, and each would otherwise pass
/// silently as a different rung's answer: `tau_gov` beside `lag` is cascade A (rung 67), a forced
/// release edge measures the forcing, and an unarmed min-select leg reduces the cascade to rung 65
/// while claiming four states.
#[test]
fn b_rung_66_refuses_cascade_a_the_forced_edges_and_an_unarmed_leg() {
    let m = t66(&armed());
    let armed_lag = FuelLimiters { lag: Some(lag()), surge: Some(floor()), ..Default::default() };

    let e = message_lim(&m, &FuelLimiters { tau_gov: Some(0.05), tt4_max: Some(HI),
                                            ..armed_lag.clone() });
    assert!(e.contains("rung-66 takes CASCADE B") && e.contains("OPPOSITE signs"),
            "the cascade-A refusal: {e}");

    let e = message_lim(&m, &FuelLimiters { s_off: Some(0.3), ..armed_lag.clone() });
    assert!(e.contains("FORCED release edges"), "the s_off refusal: {e}");
    let e = message_lim(&m, &FuelLimiters { tau_rel: Some(0.1), ..armed_lag.clone() });
    assert!(e.contains("FORCED release edges"), "the tau_rel refusal: {e}");

    let e = message_lim(&m, &FuelLimiters { lag: Some(lag()), ..Default::default() });
    assert!(e.contains("arm one (accel/surge)"), "the unarmed-leg refusal: {e}");
}

/// **RUNG 67's FOUR.** The `Tt4_max` one is the load-bearing member: rung 66 recorded an ambiguity
/// about WHERE the redline is applied and dodged it by never arming one; here the redline IS the
/// lagged leg, so the refusal is what makes the placement testable against rung 47 itself.
#[test]
fn b_rung_67_refuses_a_clockless_redline_cascade_b_a_second_leg_and_the_forced_edges() {
    let m = t67(&armed());
    let gov = FuelLimiters { tau_gov: Some(0.05), tt4_max: Some(HI), ..Default::default() };

    let e = message_lim(&m, &FuelLimiters { tt4_max: None, ..gov.clone() });
    assert!(e.contains("a governor needs a redline to lag"), "the clockless-redline refusal: {e}");

    let e = message_lim(&m, &FuelLimiters { lag: Some(lag()), ..gov.clone() });
    assert!(e.contains("rung-67 is CASCADE A"), "the cascade-B refusal: {e}");

    let e = message_lim(&m, &FuelLimiters { surge: Some(floor()), ..gov.clone() });
    assert!(e.contains("arms the GOVERNOR as its fuel leg"), "the second-fuel-leg refusal: {e}");

    let e = message_lim(&m, &FuelLimiters { s_off: Some(0.3), ..gov.clone() });
    assert!(e.contains("FORCED release edges"), "the s_off refusal: {e}");
}

// =============================================================================================
// C — `_RINGS`
// =============================================================================================

/// `tests/test_rung67.py:285` reads the class attribute directly, so the port owes a name that can
/// be read the same way. It is a plain `const` because a grep over the whole 23 066-line
/// `engine.py` finds ONE definition and no rebinding at any rung through 84 — see [`RINGS`]'s own
/// comment, which is where the port is wrong if a later rung ever overrides it.
#[test]
fn c_rings_is_two() {
    assert_eq!(RINGS, 2);
}

// =============================================================================================
// D — THE CARRIERS REACH THE READ
// =============================================================================================

/// **A CARRIER THAT NEVER LANDS IS INVISIBLE WHILE THE CLOCKS ARE DISARMED**, which is every gate
/// in section A. `_lag` and `_tau_gov` are set by `_stator_march` and read two frames down by
/// `integrate_fuel`, and the cell in between takes no such argument — so the only way to prove the
/// chain is to arm the carrier ALONE and show the read happened.
///
/// At step 1 the read lands on the step-2 stub, and that panic IS the witness: it can only be
/// reached by a value that travelled scope → `Cell` → guard → the resolving `or_else`. Step 2
/// replaces this assertion with the march it names; nothing else about the gate changes.
#[test]
fn d_the_lag_carrier_travels_from_the_scope_to_integrate_fuel() {
    let m = t66(&armed());
    let leg = surge_leg();
    let e = message(move || {
        let _ = m.stator_march_scoped(&flight(), &ramp(), None, &leg,
                                      &MarchScope { lag: Some(lag()), ..MarchScope::DEFAULT });
    });
    assert!(e.contains("SLICE Z STEP 2") && e.contains("rung-66"),
            "the `lag` carrier did not reach rung 66's integrate_fuel; got: {e}");
}

/// [`d_the_lag_carrier_travels_from_the_scope_to_integrate_fuel`]'s twin on the other field, asked
/// separately rather than inherited: a carrier claim on ONE hook says nothing about the next.
#[test]
fn d_the_tau_gov_carrier_travels_from_the_scope_to_integrate_fuel() {
    let m = t67(&armed());
    let leg = StatorLeg { accel: None, surge: None, tt4_max: Some(HI) };
    let e = message(move || {
        let _ = m.stator_march_scoped(&flight(), &ramp(), None, &leg,
                                      &MarchScope { tau_gov: Some(0.05),
                                                    ..MarchScope::DEFAULT });
    });
    assert!(e.contains("SLICE Z STEP 2") && e.contains("rung-67"),
            "the `tau_gov` carrier did not reach rung 67's integrate_fuel; got: {e}");
}

/// **THE GUARD RESTORES, AND IT RESTORES ON AN UNWIND.** Both carriers are `Drop`-restored, so a
/// march that PANICS inside must still leave the field as it found it — which is the property a
/// straight-line `finally`-style restore would lose and which Python gets from `try/finally`.
///
/// Gated here rather than at step 5 because it is free: section D's two refusals already unwind
/// through both guards, so the only thing left to assert is that the next disarmed march is still
/// bit-for-bit rung 65's. If either guard leaked, this trajectory would be the ARMED one — or the
/// stub's panic.
#[test]
fn d_both_guards_restore_through_the_unwind_their_own_refusal_causes() {
    let leg = surge_leg();
    let reference = keys(&t65(&armed()).stator_march(&flight(), &ramp(), None, &leg).0);

    let m = t66(&armed());
    assert!(panics(|| {
        let _ = m.stator_march_scoped(&flight(), &ramp(), None, &surge_leg(),
                                      &MarchScope { lag: Some(lag()), ..MarchScope::DEFAULT });
    }), "the armed march must reach the step-2 stub");
    assert_eq!(keys(&m.stator_march(&flight(), &ramp(), None, &leg).0), reference,
               "rung 66's `_lag` leaked past the unwind");

    let m = t67(&armed());
    let leg67 = StatorLeg { accel: None, surge: None, tt4_max: Some(HI) };
    assert!(panics(|| {
        let _ = m.stator_march_scoped(&flight(), &ramp(), None, &leg67,
                                      &MarchScope { tau_gov: Some(0.05), ..MarchScope::DEFAULT });
    }), "the armed march must reach the step-2 stub");
    assert_eq!(keys(&m.stator_march(&flight(), &ramp(), None, &leg).0), reference,
               "rung 67's `_tau_gov` leaked past the unwind");
}
