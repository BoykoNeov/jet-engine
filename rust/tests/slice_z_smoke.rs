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
//!   are disarmed — so the proof is that arming the carrier ALONE selects a different march.
//! * **E — THE TWO MARCHES**, added at step 2: P2's remaining THREE reduce arms (rung 52, rung
//!   66, rung 47), the two REDLINE PLACEMENTS, the joint IC's own record, the RK4 floor that is a
//!   SUM, the damping ladder no shipped grid reaches, the caught-panic arm no shipped grid fires,
//!   and the ringing detector's own two ends. **P2's six arms onto five targets are now gated
//!   six-for-six**, and the count is stated because the noun is checkable.
//!
//! **THREE OF STEP 1's GATES WERE REWRITTEN HERE, NOT ADJUSTED.** They asserted on the panic
//! string `"SLICE Z STEP 2"`, which this step deletes — scheduled work, booked at step 1 so this
//! step met it as a task rather than as breakage. The carrier claim survives as *the armed march
//! produces a trajectory the disarmed one does not*, which is strictly stronger, and the
//! guard-restore gate found a new unwind INSIDE the guard's lifetime: the rung's own RK4 floor.
//!
//! **THREE OF THIS FILE'S OWN ROUTE BARS WERE TYPED AND WRONG**, and the third was wrong in a way
//! the bar could not show: `14` where `16` was right (twice), and then a `14` that was right on
//! both sides of a comparison whose floats still disagreed, because rung 46's unlagged governor
//! and rung 47's lagged one emit the SAME fourteen keys. Every one traces to the same cause —
//! a [`MarchScope`] field a junior rung silently ignores where Python raises `TypeError` — which
//! is recorded there.
//!
//! The hardware is `slice_y_smoke.rs`'s, unchanged, so a difference between the two files is a
//! difference between the rungs and not between two harnesses.

use std::panic::catch_unwind;

use turbojet::bleed_transient::LeverArm;
use turbojet::cross_loop::{build_cross_loop_cascade, cross_extra, detector_sensitivity,
                           joint_fixed_point, IcCorner, RINGS};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{asym_extra, AsymmetricLag, Floor, FuelLimiters, FuelPoint,
                               PointExtra, SurgeLimiter};
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
/// Rung 67's IMPOSED redline. Chosen for OVERLAP: the scheduled fuel drives instantaneous
/// `Tt4` to ~1900 K during the accel, so any redline below that engages EARLY, over the valve's
/// own window. Rung 46/47's own value would put the governor's window past the ramp entirely.
const TT4_MAX: f64 = 1200.0;

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

/// The rung-47 GOVERNOR's leg — the redline rung 67's clock lags.
fn gov_leg() -> StatorLeg<'static> {
    StatorLeg { accel: None, surge: None, tt4_max: Some(TT4_MAX) }
}

/// Every float a trajectory point publishes that the reduce arms must agree on, bit-for-bit.
fn keys(t: &[FuelPoint]) -> Vec<(f64, f64, f64, f64, f64, f64, f64, f64)> {
    t.iter().map(|p| (p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.tt4, p.mf, p.mf_sched))
        .collect()
}

/// **WHICH MARCHER RAN, read off the one thing a trajectory carries that a float cannot fake** —
/// its per-point key COUNT. 14 is the bare/rung-47 route, 16 rung 52's or rung 65's, **20 rung
/// 66's and 21 rung 67's.**
///
/// A reduce gate that compares only floats cannot tell "the right march ran" from "two marches
/// happened to agree", and every gate in section A is exactly that comparison. This is the
/// cheapest witness that is not a float.
fn route(t: &[FuelPoint]) -> usize {
    t[0].key_count()
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
/// **STEP 2 REWROTE THIS GATE RATHER THAN ADJUSTING IT, AND THE REWRITE IS STRICTLY STRONGER.**
/// At step 1 the witness was the stubbed march's panic: reachable only by a value that had
/// travelled scope → `Cell` → guard → the resolving `or_else`. That panic no longer exists, and a
/// gate that reads a message about to be deleted is slice V step 5's *gates that read nothing* in
/// its inverse form — so the claim becomes **the armed march produces a trajectory the disarmed
/// one does not**, which needs the carrier to land AND the march it selects to be a different
/// march.
#[test]
fn d_the_lag_carrier_travels_from_the_scope_to_integrate_fuel() {
    let m = t66(&armed());
    let leg = surge_leg();
    let disarmed = m.stator_march(&flight(), &ramp(), None, &leg).0;
    let armed_run = m.stator_march_scoped(
        &flight(), &ramp(), None, &leg,
        &MarchScope { lag: Some(lag()), ..MarchScope::DEFAULT }).0;
    assert_ne!(keys(&disarmed), keys(&armed_run),
               "arming `lag` through the scope changed NOTHING: the carrier did not reach rung \
                66's integrate_fuel, or its dispatch ignored it");
    // …and the difference is the RUNG's, not a numerical wobble: the armed run carries the
    // cascade's own 20 keys where the disarmed one carries rung 65's 16.
    assert_eq!(route(&armed_run), 20, "the armed march is not rung 66's");
    assert_eq!(route(&disarmed), 16, "the disarmed march is not rung 65's");
}

/// [`d_the_lag_carrier_travels_from_the_scope_to_integrate_fuel`]'s twin on the other field, asked
/// separately rather than inherited: a carrier claim on ONE hook says nothing about the next.
#[test]
fn d_the_tau_gov_carrier_travels_from_the_scope_to_integrate_fuel() {
    let m = t67(&armed());
    let leg = gov_leg();
    let disarmed = m.stator_march(&flight(), &ramp(), None, &leg).0;
    let armed_run = m.stator_march_scoped(
        &flight(), &ramp(), None, &leg,
        &MarchScope { tau_gov: Some(TAU), ..MarchScope::DEFAULT }).0;
    assert_ne!(keys(&disarmed), keys(&armed_run),
               "arming `tau_gov` through the scope changed NOTHING: the carrier did not reach \
                rung 67's integrate_fuel, or its dispatch ignored it");
    assert_eq!(route(&armed_run), 21, "the armed march is not rung 67's");
    // **16, MEASURED — and the first bar this file typed was 14.** With the clock disarmed the
    // machine still has a LAGGED VALVE, so the redline falls through to rung 65's marcher, which
    // min-selects it inside the caps at `mf_sched` (rung 65's placement) and records `b`/`b_cmd`.
    // Rung 46's fourteen-key route needs the valve unlagged too, which is a different machine.
    assert_eq!(route(&disarmed), 16, "the disarmed march is not rung 65's valve-lag march");
}

/// **THE GUARD RESTORES, AND IT RESTORES ON AN UNWIND.** Both carriers are `Drop`-restored, so a
/// march that PANICS inside must still leave the field as it found it — the property a
/// straight-line `finally`-style restore would lose and which Python gets from `try/finally`.
///
/// **STEP 2 HAD TO FIND A NEW UNWIND, AND THE ONE IT USES IS THE RUNG's OWN.** Step 1 raised
/// through the stub; with the march built, the reachable panic INSIDE the guard's lifetime is the
/// **RK4 stability floor** — `ds*(1/tau_v + 1/tau_g) <= 2` — which each marcher asserts before it
/// touches a state. Both clocks below are fast enough to trip it (`2.7` against the bar of `2`),
/// so the unwind starts strictly inside `_stator_march`'s guard and the following bit-for-bit
/// re-march is the check.
#[test]
fn d_both_guards_restore_through_the_unwind_their_own_refusal_causes() {
    let leg = surge_leg();
    let reference = keys(&t65(&armed()).stator_march(&flight(), &ramp(), None, &leg).0);

    let m = t66(&armed());
    assert!(panics(|| {
        // ds * (1/0.05 + 1/0.004) = 2.7 — outside the floor, and the floor is rung 66's SUM.
        let _ = m.stator_march_scoped(
            &flight(), &ramp(), None, &surge_leg(),
            &MarchScope { lag: Some(AsymmetricLag::new(0.004, 0.016)),
                          ..MarchScope::DEFAULT });
    }), "the armed march must trip rung 66's RK4 floor");
    assert_eq!(keys(&m.stator_march(&flight(), &ramp(), None, &leg).0), reference,
               "rung 66's `_lag` leaked past the unwind");

    let m = t67(&armed());
    assert!(panics(|| {
        let _ = m.stator_march_scoped(&flight(), &ramp(), None, &gov_leg(),
                                      &MarchScope { tau_gov: Some(0.004),
                                                    ..MarchScope::DEFAULT });
    }), "the armed march must trip rung 67's RK4 floor");
    assert_eq!(keys(&m.stator_march(&flight(), &ramp(), None, &leg).0), reference,
               "rung 67's `_tau_gov` leaked past the unwind");
}

// =============================================================================================
// E — THE TWO MARCHES. **Step 2's own sections.**
// =============================================================================================

/// **THE THIRD REDUCE ARM: RUNG 52, BY DISPATCH.** Section A's two gates hold the clocks disarmed;
/// this one ARMS the fuel lag and disarms the VALVE, which is the other side of the same `if`
/// (`_lagged() && lag.is_some()`).
///
/// It is a separate gate because the dispatch is a CONJUNCTION: a port that tested only the clock
/// would pass section A and fail here, and one that tested only `_lagged()` would do the reverse.
/// The `route` check is what makes it more than a float comparison — all three machines must land
/// on rung 52's SIXTEEN-key marcher, not merely agree.
///
/// **IT GOES THROUGH `integrate_fuel` DIRECTLY, AND THE FIRST DRAFT'S ROUTE WAS A REAL FINDING.**
/// Armed through `stator_march_scoped` the rung-65 machine returned FOURTEEN keys — because
/// **rung 65's `_stator_march` has no `lag` parameter at all**, so Python would raise `TypeError`
/// on that call while the port, whose [`MarchScope`] is ONE struct shared by every rung, silently
/// IGNORES the field. That is step 1's finding running downward: a scope consumes its own field
/// and drops the rungs above it, and a rung BELOW the field's owner cannot see it either.
/// Recorded at `MarchScope` rather than repaired — the port is a translation — and the gate takes
/// the route Python's own suites take.
#[test]
fn e_all_three_rungs_reduce_to_rung_52_when_the_valve_is_not_lagged() {
    let unlagged = LeverArm::floored(BleedLimiter::new(PHI, B));
    let lim = FuelLimiters { surge: Some(floor()), lag: Some(lag()), ..Default::default() };
    let run = |m: ScheduledStatorCore| {
        m.fuel.integrate_fuel(&flight(), |s| 0.02 + 0.01 * 1.0f64.min(s / R), (0.75, 0.79),
                              0.30, DS, &lim)
    };
    let a = run(t65(&unlagged));
    let b = run(t66(&unlagged));
    let c = run(t67(&unlagged));
    assert_eq!(route(&a), 16, "rung 52's marcher records `g`/`required` and nothing else");
    assert_eq!(keys(&a), keys(&b), "rung 66 on an UNLAGGED valve is not rung 52");
    assert_eq!(keys(&a), keys(&c), "rung 67 on an UNLAGGED valve is not rung 52");
    assert_eq!((route(&b), route(&c)), (16, 16),
               "one of the cascades entered its OWN marcher on an unlagged valve");
}

/// **THE REDLINE LIVES IN A DIFFERENT PLACE ON THE TWO CASCADES, AND EACH PLACEMENT IS EXHIBITED
/// HERE RATHER THAN ARGUED.**
///
/// Rung 66 min-selects an UNLAGGED `Tt4_max` on top of the already-clipped fuel (rung 52's
/// placement) and its own docstring records the ambiguity — then dodges it, because cascade B
/// arms `surge` alone and **every shipped rung-66 diagnostic passes `Tt4_max = None`.** Rung 67
/// carries the redline BY THE STATE (`mf = mf_sched − g`, rung 47's placement) and has no
/// min-select in its derivative at all.
///
/// So the two claims are opposite and both are checkable on one trajectory each:
///
/// * on rung 66 with the redline ARMED, some point must have `mf` **strictly below**
///   `mf_sched − g` — that is the min-select biting;
/// * on rung 67, **every** point must have `mf` exactly `max(1e-9, mf_sched − g)` — bit-for-bit,
///   because nothing else may touch it.
///
/// A porter who copies rung 66's derivative into rung 67 and keeps its `Tt4_max` branch compiles,
/// marches, and fails the second half. **This gate reaches rung 66's branch through a DIRECT
/// `integrate_fuel` call, because no march route arms it** — which is the same reason Python
/// could record the ambiguity without ever running it.
#[test]
fn e_the_redline_min_selects_on_rung_66_and_is_carried_by_the_state_on_rung_67() {
    let m66 = t66(&armed());
    let lim = FuelLimiters { tt4_max: Some(1250.0), surge: Some(floor()), lag: Some(lag()),
                             ..Default::default() };
    let t = m66.fuel.integrate_fuel(&flight(), |s| 0.02 + 0.01 * 1.0f64.min(s / R),
                                    (0.75, 0.79), 0.30, DS, &lim);
    assert_eq!(route(&t), 20, "the direct call did not reach rung 66's cascade marcher");
    let bit = t.iter().filter(|p| {
        let (g, _) = asym_extra(p);
        p.mf < 1e-9f64.max(p.mf_sched - g)
    }).count();
    assert!(bit > 0,
            "rung 66's UNLAGGED redline never min-selected on {} points: its `Tt4_max` branch is \
             dead, and no shipped diagnostic would have noticed", t.len());

    let m67 = t67(&armed());
    let (x, _) = m67.stator_march_scoped(&flight(), &ramp(), None, &gov_leg(),
                                         &MarchScope { tau_gov: Some(TAU),
                                                       ..MarchScope::DEFAULT });
    assert_eq!(route(&x), 21, "the march did not reach rung 67's cross marcher");
    for p in &x {
        let (g, _) = asym_extra(p);
        assert_eq!(p.mf, 1e-9f64.max(p.mf_sched - g),
                   "rung 67's applied fuel is not `mf_sched - g` at s = {}: something \
                    min-selected on top, which is rung 66's placement and breaks the rung-47 \
                    reduce that DETECTS it", p.s);
    }
    // …and the governor really did engage, so the loop above is not vacuously comparing
    // `mf == mf_sched` at every point.
    assert!(x.iter().any(|p| asym_extra(p).0 > 0.0),
            "the governor never clipped: `g == 0` throughout makes the assertion above vacuous");
}

/// **THE JOINT INITIAL CONDITION IS SOLVED ONCE, AND EVERY POINT CARRIES THE SAME RECORD.**
///
/// Python records `ic_iters` / `ic_res` (and rung 67's `ic_damp`) PER POINT although the solve
/// runs once, before the loop. That is a redundancy in the source and the port keeps it — but it
/// is also the cheapest check that the solve was not accidentally moved INSIDE the marching loop,
/// which would be invisible in every other key on a converged grid.
#[test]
fn e_the_joint_ic_record_is_constant_across_both_trajectories() {
    let (a, _) = t66(&armed()).stator_march_scoped(
        &flight(), &ramp(), None, &surge_leg(),
        &MarchScope { lag: Some(lag()), ..MarchScope::DEFAULT });
    let first = a[0].extra;
    assert!(matches!(first, PointExtra::Cascade { .. }), "rung 66's route");
    for p in &a {
        match (p.extra, first) {
            (PointExtra::Cascade { ic_iters: i, ic_res: r, .. },
             PointExtra::Cascade { ic_iters: i0, ic_res: r0, .. }) =>
                assert_eq!((i, r), (i0, r0), "rung 66's IC record moved at s = {}", p.s),
            _ => unreachable!(),
        }
    }

    let (b, _) = t67(&armed()).stator_march_scoped(
        &flight(), &ramp(), None, &gov_leg(),
        &MarchScope { tau_gov: Some(TAU), ..MarchScope::DEFAULT });
    let f0 = cross_extra(&b[0]);
    for p in &b {
        assert_eq!(cross_extra(p), f0, "rung 67's IC record moved at s = {}", p.s);
    }
}

/// **THE RK4 FLOOR IS THE *SUM* OF THE TWO RATES, WHICH IS HALF RUNG 65's BOUND AT MATCHED
/// CLOCKS — and that is the artefact this rung exists to refuse.**
///
/// Rung 65 published a RETRACTION: an instability at `ds/tau = 5` returned an `∫b ds` 4.4x the
/// converged value and looked exactly like a physical finding. A cascade has TWO clocks, and the
/// naive transfer — *bound the FASTEST one* — is wrong **in the unsafe direction, by up to 2x**,
/// because `det J == 0` makes the non-zero eigenvalue exactly `−(1/t_g + 1/t_v)`: THE RATES ADD.
///
/// The gate is the pair a naive port would get wrong: a step rung 65 ACCEPTS on the same valve
/// (`ds/tau = 1.6`) and both cascades REFUSE, because the second clock doubles the rate.
#[test]
fn e_the_cascade_rk4_floor_is_the_sum_and_rung_65_accepts_what_it_refuses() {
    // ds/tau = 0.08/0.05 = 1.6 <= 2, so rung 65's single-state bound is satisfied…
    let fast = Ramp { ds: 0.08, ..ramp() };
    let leg = surge_leg();
    assert!(!panics(|| { let _ = t65(&armed()).stator_march(&flight(), &fast, None, &leg); }),
            "rung 65 must ACCEPT ds/tau = 1.6 — otherwise the comparison below is empty");

    // …and ds*(1/0.05 + 1/0.05) = 3.2 > 2, so a matched-clock cascade refuses the same step.
    let m66 = t66(&armed());
    let e = message(move || {
        let _ = m66.stator_march_scoped(&flight(), &fast, None, &surge_leg(),
                                        &MarchScope { lag: Some(AsymmetricLag::new(0.05, 0.15)),
                                                      ..MarchScope::DEFAULT });
    });
    assert!(e.contains("rung-66") && e.contains("THE RATES ADD"),
            "rung 66 accepted a step its own identity forbids: {e}");

    let m67 = t67(&armed());
    let e = message(move || {
        let _ = m67.stator_march_scoped(&flight(), &fast, None, &gov_leg(),
                                        &MarchScope { tau_gov: Some(TAU),
                                                      ..MarchScope::DEFAULT });
    });
    assert!(e.contains("rung-67") && e.contains("CONSERVATIVE here"),
            "rung 67 accepted a step the inherited floor forbids: {e}");
}

/// **THE DAMPING LADDER IS CODE NO SHIPPED GRID REACHES, SO IT IS EXERCISED DIRECTLY.**
///
/// `joint_fixed_point`'s own docstring says why it was extracted from the march: on the anchored
/// plant `|P| ~ 0.02` and the undamped sweep converges in one or two iterations, so `w = 1/2` and
/// `w = 1/4` never run there. Fed SYNTHETIC laws with a chosen `P` — `R(q) = P·q`, `C(g) = g`, so
/// the composite multiplier is `(1−w) + wP` — each rung of the ladder is reachable, and the
/// boundaries the docstring states become checkable rather than asserted.
///
/// The fourth case is the one the march then refuses: all three dampings fail, `res` stays above
/// `1e-9`, and `w` is the LAST attempt's — Python's leaked loop variable, kept.
#[test]
fn e_the_joint_fixed_points_damping_ladder_is_reachable_at_every_rung() {
    for (p, want_w) in [(-0.5f64, 1.0f64), (-2.0, 0.5), (-5.0, 0.25)] {
        let j = joint_fixed_point(&|q| p * q, &|g| g, 1.0, false, 1e-12, 60);
        assert_eq!(j.w, want_w, "P = {p} should have settled at damping {want_w}, got {j:?}");
    }
    let j = joint_fixed_point(&|q| -20.0 * q, &|g| g, 1.0, false, 1e-12, 60);
    assert_eq!(j.w, 0.25, "the ladder must have run to its end");
    assert!(!(j.res <= 1e-9),
            "|P| = 20 is outside every damping the ladder offers and must NOT report \
             convergence: {j:?}");
    // `fix_q` holds the valve: only the clip moves, and it lands on `R(q0)` in ONE step plus the
    // step that measures zero.
    let j = joint_fixed_point(&|q| 0.5 * q, &|g| g, 0.4, true, 1e-12, 60);
    assert_eq!((j.q, j.g, j.its), (0.4, 0.2, 2), "the fixed-q arm: {j:?}");
}

/// **A NULL RESULT IS WORTH NOTHING UNTIL THE INSTRUMENT IS SHOWN TO FIRE**, and rung 67's
/// headline rests on one: ZERO sign changes in the free response at every clock pair.
///
/// [`detector_sensitivity`] is that instrument run on the LINEAR block itself, and this gate is
/// the two ends of its own table: **quiet at the plant's measured coupling** (`|P| ~ 0.02`,
/// where `T = 45 tau` and the amplitude is `e^-45` by then — not because it is blind) and
/// **firing at a strong one**. Without both halves `rings_anywhere == false` is unfalsifiable.
#[test]
fn e_the_ringing_detector_is_quiet_at_the_measured_coupling_and_fires_at_a_strong_one() {
    let d = detector_sensitivity(&[-0.02, -0.5, -3.0, -10.0], 0.05, 0.0025, 1.7);
    assert_eq!(d.quiet_at_weak, Some(true), "the detector fired at |P| = 0.02");
    assert!(d.fires, "the detector never fired at ANY coupling — it cannot see a ring at all");
    assert_eq!(d.rows[0].sign_changes, 0, "|P| = 0.02");
    assert!(d.rows[3].sign_changes >= RINGS,
            "|P| = 10 must ring: {} crossings", d.rows[3].sign_changes);
    // ONE crossing is admissible on the REAL branch — a sum of two decaying real exponentials
    // has at most one zero — so the threshold is TWO, and it is a theorem rather than a
    // tolerance.
    assert_eq!(RINGS, 2);
}

/// **P2's FIFTH ARM: RUNG 67 REDUCES TO RUNG 66, THROUGH THE MARCH — and it looks like step 1's
/// discard should block it, which is exactly why it is gated.**
///
/// Step 1 measured that arming BOTH clocks through `_stator_march` on a rung-67 machine silently
/// DISCARDS the fuel lag: rung 67's own armed branch returns before `super()`, so the carrier rung
/// 66 would read is never read. With `tau_gov` DISARMED that branch is not taken, the call falls
/// through to rung 66's `integrate_fuel`, and the carrier IS read — so the same route that
/// discards a lag when both clocks are armed delivers it when only one is.
///
/// The bar is 20 keys on BOTH machines: agreement alone cannot tell "rung 66's marcher ran on
/// both" from "neither ran and both fell to rung 65".
#[test]
fn e_rung_67_reduces_to_rung_66_with_the_governor_clock_disarmed() {
    let leg = surge_leg();
    let sc = MarchScope { lag: Some(lag()), ..MarchScope::DEFAULT };
    let (a, nu_a) = t66(&armed()).stator_march_scoped(&flight(), &ramp(), None, &leg, &sc);
    let (b, nu_b) = t67(&armed()).stator_march_scoped(&flight(), &ramp(), None, &leg, &sc);
    assert_eq!(route(&a), 20, "rung 66's own marcher did not run");
    assert_eq!(route(&b), 20, "rung 67 with `tau_gov` disarmed did not reach rung 66's marcher");
    assert_eq!(keys(&a), keys(&b), "rung 67 with `tau_gov` disarmed is not rung 66");
    assert_eq!(nu_a, nu_b, "the terminal speeds");
}

/// **P2's SIXTH ARM: RUNG 67 REDUCES TO RUNG 47 — and Python calls this arm THE `Tt4_max`
/// PLACEMENT DETECTOR.**
///
/// *"With the valve disarmed this class must reproduce `_integrate_fuel_lagged` BIT-FOR-BIT,
/// which it does by dispatch — so a wrong placement here shows up as a diff against rung 47
/// itself."* Section A's unfloored gate marches a `surge` leg and lands nowhere near rung 47's
/// lagged marcher; this one arms the REDLINE and its CLOCK on a machine with no valve at all.
///
/// **The `route == 14` bar is a discriminator but NOT the one that matters here**, and finding
/// that out cost this gate a rewrite. Rung 47's marcher records neither `g`/`required` nor
/// `b`/`b_cmd`, so a slip onto the 16-key valve-lag route or the 21-key cross route shows up —
/// but **rung 46's UNLAGGED governor emits fourteen keys too.**
///
/// The first draft armed the clock through `stator_march_scoped` and the three machines
/// disagreed on the floats while all three passed `route == 14`. The reason is the same silence
/// that rewrote the rung-52 gate, one field further up: **rung 65's AND rung 66's `_stator_march`
/// both lack a `tau_gov` parameter**, where Python raises `TypeError`, so those two marched rung
/// 46's unlagged redline while rung 67 marched rung 47's lagged one. Two different rungs, one key
/// count. So the gate takes the DIRECT route, which is the route Python's own suites take — and
/// the episode is why the finding is recorded at [`MarchScope`] rather than left in a smoke
/// comment.
#[test]
fn e_rung_67_reduces_to_rung_47_on_a_machine_with_no_valve() {
    let bare = LeverArm::default();
    let lim = FuelLimiters { tt4_max: Some(TT4_MAX), tau_gov: Some(TAU), ..Default::default() };
    let run = |m: ScheduledStatorCore| {
        m.fuel.integrate_fuel(&flight(), |s| 0.02 + 0.01 * 1.0f64.min(s / R), (0.75, 0.79),
                              0.30, DS, &lim)
    };
    let a = run(t65(&bare));
    let b = run(t66(&bare));
    let c = run(t67(&bare));
    assert_eq!(route(&a), 14, "rung 47's marcher records neither clip nor position");
    assert_eq!((route(&b), route(&c)), (14, 14),
               "a cascade marcher ran on a machine that has no valve to lag");
    assert_eq!(keys(&a), keys(&c), "rung 67 with no valve is not rung 47");
    assert_eq!(keys(&a), keys(&b), "rung 66 with no valve is not rung 47");
    // …and the governor is LIVE on that trajectory, so the three-way agreement is not three
    // machines agreeing about a limiter none of them ran.
    assert!(a.iter().any(|p| p.mf < p.mf_sched),
            "the redline never clipped: the arm agrees about nothing");
}

/// **THE CAUGHT-PANIC ARM OF `joint_ic_corners` FIRES ON NO SHIPPED GRID, SO IT IS EXHIBITED
/// HERE.**
///
/// Measured, not feared: at the probe's `ds = 0.01` all 8 corners converge, and the suite's own
/// call uses 2x2 corners that give 4 converged rows. So the `catch_unwind`, the 120-CHARACTER
/// truncation and the claim that *characters and bytes agree because the message is ASCII* would
/// otherwise be asserted in a doc comment and measured nowhere — [[rust-port-slice-m]]'s exact
/// shape, and step 4's oracle runs the suite's grid so it will not reach it either.
///
/// A `ds` outside the inherited RK4 floor makes EVERY corner raise, inside the `catch_unwind`, so
/// the arm runs eight times. **`msg_len` and `msg_hash` were measured on BOTH languages** and
/// agree — the message is Python's assert text, truncated identically — which is what makes the
/// two constants below an anchor rather than a Rust-only golden.
#[test]
fn e_the_caught_panic_arm_of_joint_ic_corners_is_exhibited() {
    let m = t67(&armed());
    // ds * (1/0.05 + 1/0.05) = 3.2 > 2 — every corner trips rung 67's floor before it marches.
    let coarse = Ramp { ds: 0.08, ..ramp() };
    let d = m.joint_ic_corners(&flight(), &coarse, &[1050.0, 1150.0, 1200.0, 1300.0],
                               &[1000.0, 1200.0], TAU, TAU);
    assert_eq!(d.rows.len(), 8, "two starts x four redlines");
    let mut seen = 0usize;
    for r in &d.rows {
        match r {
            IcCorner::Ok(x) => panic!("corner {:?} converged where the floor should have \
                                       refused it", (x.tt4_lo, x.tt4_max)),
            IcCorner::Failed { failed, .. } => {
                seen += 1;
                assert_eq!(failed.chars().count(), 120,
                           "the message was not truncated to 120 CHARACTERS: {failed}");
                assert_eq!(failed.len(), 120,
                           "characters and bytes disagree, so the message is not ASCII and \
                            Python's `str[:120]` is not this slice");
                let h: u64 = failed.chars().enumerate()
                                   .map(|(j, c)| (c as u64) * (j as u64 + 1)).sum();
                assert_eq!(h, 647_771,
                           "the truncated text differs from PyPy's: {failed}");
            }
        }
    }
    assert_eq!(seen, 8);
    // …and the SUMMARIES over an all-failed table are Python's vacuous-`all` / empty-`max`
    // answers, not a crash: `ok` is empty, so `all_converged` is true and `max_iters` is the
    // `default=0`. Both are the `default=` branches § 5.24 (v) counts as never firing.
    assert!(d.all_converged, "Python's `all([])` is True and the port must agree");
    assert_eq!((d.n_live, d.max_iters, d.ever_damped), (0, 0, false));
}
