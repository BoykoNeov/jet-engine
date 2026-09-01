//! SLICE AD step 2 — **THE SIX-STATE MARCH, AND THIRTEEN READERS THAT WOULD HAVE MISTREATED ITS
//! TRAJECTORY IN SILENCE.**
//!
//! # THE FINDING — the crate's own "the next variant breaks the build" convention holds at 7 of 20 sites
//!
//! `cross_extra`'s doc comment states the rule this slice depended on: *"The arms are spelled out
//! rather than left to a wildcard so that the NEXT `PointExtra` variant breaks the build here and
//! gets the same question asked of it — see rung 65's `valve_of`, whose wildcard is what slice Z's
//! audit had to unpick by hand."*
//!
//! Adding `PointExtra::Shared` measured how far that convention actually reaches. Over the 20
//! `match … .extra` sites in `src`: **7 exhaustive, 13 carrying a `_ =>` wildcard.** The compiler
//! stopped at 6 of the 7 (the seventh, `key_count`, was already updated). The other **13 compiled
//! silently**, and each would have given a rung-72 trajectory a wrong answer of one of three
//! kinds:
//!
//! | fallback | sites | what a rung-72 point would have got |
//! |---|---|---|
//! | `_ => 0.0` | 3 | the DESIGN stator setting for a march that recorded a live one |
//! | `_ => false` (in a filter) | 3 | **silently dropped**, so the reader computes over an EMPTY set |
//! | `_ => panic!` / `unreachable!` | 7 | a refusal Python does not raise, on a dict that carries the key |
//!
//! **The `false`-in-a-filter three are the quietest of the three kinds** and the reason this file
//! exists: a reader that returns an empty riding set reports perfect tracking, and every statistic
//! downstream of it is then computed over nothing at all. Python's rung-72 march says in so many
//! words that *"every inherited reader works on this trajectory"*, so all thirteen are widened;
//! the ONE widening question answered NO is `cross_extra`, which refuses rung 72 for the reason it
//! already refuses rungs 66 and 68 — this march iterates the joint sweep UNDAMPED and carries no
//! `ic_damp`.
//!
//! # AND THE PROBE THAT COUNTED THE SITES WAS WRONG TWICE, IN THE SAME DIRECTION
//!
//! It reported **12** wildcards, then 12 again, before reporting 13. Both misses were the same
//! site — `three_loop.rs:2032`, a match written entirely on ONE LINE — and both were the
//! instrument's, not the crate's: the first regex anchored `_ =>` to the start of a line, and the
//! repaired one still scanned a body that began at the line AFTER the `match`, which for a
//! single-line match is empty. **An instrument that undercounts silent fallbacks is the exact
//! defect it was built to find**, and it undercounted twice before it agreed with the compiler.
//!
//! # `v_regime` IS AN `Option` HERE AND A BARE `Regime` EVERYWHERE ELSE
//!
//! Rung 70's march requires a stator (`expect("rung-70's march with no stator floor")`); rung 72's
//! does not, because 8 of `shared_bill`'s 16 cells disarm it, and Python's `stator()` returns the
//! constant `(0, None)` there. `Regime::Dormant` is a real regime meaning `b = 0` **after a solve
//! ran**, so reusing it for "no solve ran" would have put a label in the trajectory that the
//! integrator never produced.
//!
//! # WHAT THIS FILE DOES NOT GATE
//!
//! **The readers.** `shared_gains`, `shared_cells`, `authority_law`, `mask_discriminator`,
//! `shared_bill` and the quartic chain are step 3. This file gates the march that feeds them.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Authority, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::shared_actuator::{build_shared_actuator_cascade, ShareScope};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung72.py`'s own constants, verbatim.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const B: f64 = 0.10;
const SM: f64 = PHI / FLOOR - 1.0;
const V_MAX: f64 = 0.20;
const TT4_MAX: f64 = 1200.0;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
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

fn valve() -> BleedLimiter { BleedLimiter::with_tau(PHI, B, Some(TAU)) }
fn phi_stator() -> StatorLimiter { StatorLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S)) }
fn inc_stator() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S))
}
fn fuel_floor() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM)) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }
fn ramp() -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS } }

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

fn machine(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_shared_actuator_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// The `phi`-stator arm — rung 70's machine, which is what rung 72 arms its fourth loop on top of.
fn phi_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_lim: Some(phi_stator()), ..Default::default() }
}

/// The INCIDENCE arm — rung 71's machine.
fn inc_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_inc: Some(inc_stator()), ..Default::default() }
}

fn march(
    m: &ScheduledStatorCore, surge: Option<Floor>, lg: Option<AsymmetricLag>,
    tt4_max: Option<f64>, tau_gov: Option<f64>,
) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge, tt4_max };
    m.stator_march_scoped(&flight(), &ramp(), None, &leg,
                          &MarchScope { lag: lg, tau_gov, ..MarchScope::DEFAULT }).0
}

/// **THE RUNG-72 MARCH** — both fuel-side legs armed, which is the arming rung 71's guard B
/// refuses in so many words.
fn shared_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    march(m, Some(fuel_floor()), Some(lag()), Some(TT4_MAX), Some(TAU_GOV))
}

/// The seven-tuple per point the reduce gates compare, BIT for bit — `tests/rung71.rs`'s `keys`.
fn keys(traj: &[FuelPoint]) -> Vec<[u64; 7]> {
    traj.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits()])
        .collect()
}

fn message_of<F: FnOnce()>(f: F) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    match out {
        Ok(()) => String::new(),
        Err(e) => e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    }
}

// =============================================================================================
// 1 — THE MARCH RUNS, AND EVERY POINT IS THIS RUNG'S
// =============================================================================================

#[test]
fn the_shared_march_dispatches_to_rung_72_and_carries_thirty_keys() {
    let traj = shared_march(&machine(&phi_arm()));
    assert!(traj.len() > 100, "a real trajectory, not a stub: {} points", traj.len());
    for p in &traj {
        assert!(matches!(p.extra, PointExtra::Shared { .. }),
                "every point comes from rung 72's integrator, not an inherited one");
        assert_eq!(p.key_count(), 30, "rung 72's point is 30 keys — the widest in the port");
    }
}

/// **`mf = mf_sched - g` IS THE INVARIANT THAT MAKES ONE `g` HONEST FOR TWO LEGS**, and it is
/// asserted BIT-for-bit rather than to a tolerance.
///
/// `g` is the APPLIED clip — one `applied_clip` call shared by the plant and every reader — so an
/// inherited reader asking *what did the limiter take off* gets the answer for the loop that HOLDS
/// the actuator. If the port had recorded `g_fuel` or their sum here, this identity would fail on
/// every point where the two legs disagree.
#[test]
fn the_applied_clip_is_the_one_that_closes_the_fuel_identity() {
    let traj = shared_march(&machine(&phi_arm()));
    let mut n_both_cutting = 0usize;
    for p in &traj {
        let PointExtra::Shared { g, g_fuel, g_gov, required, required_fuel, required_gov, .. } =
            p.extra else { panic!("rung 72's point") };
        // `required` is the BINDING requirement, so an inherited reader asking "how much did the
        // limiter want" gets the leg that is about to hold the actuator — not a sum, and not
        // whichever leg happens to be listed first.
        assert_eq!(required.to_bits(), required_fuel.max(required_gov).to_bits(),
                   "required = max(required_fuel, required_gov) at s = {}", p.s);
        // `mf` is floored at 1e-9, so the identity holds exactly wherever the floor is not active.
        let expect = (p.mf_sched - g).max(1e-9);
        assert_eq!(p.mf.to_bits(), expect.to_bits(),
                   "mf = max(1e-9, mf_sched - g) at s = {}", p.s);
        assert_eq!(g.to_bits(), g_fuel.max(g_gov).to_bits(),
                   "under MIN-SELECT the applied clip is the LARGER of the two");
        if g_fuel > 0.0 && g_gov > 0.0 {
            n_both_cutting += 1;
        }
    }
    assert!(n_both_cutting > 0,
            "the gate is VACUOUS unless some point has BOTH legs cutting — otherwise `max` and \
             `sum` and `g_fuel` would all satisfy it");
}

/// The `authority` label partitions the trajectory, and **both non-trivial labels actually occur**
/// — otherwise this rung's whole subject, the hand-over, is untested.
///
/// # THE FIRST DRAFT OF THIS GATE COMPARED THE LABEL AGAINST `authority()` AND WAS BLIND
///
/// Mutation m7 swapped `gf > gr` to `gr > gf` inside `authority()` — the label on every point where
/// the two legs disagree, inverted — and **this gate passed**. It asserted
/// `au == authority(g_fuel, g_gov)`, which is the recorded label against THE SAME FUNCTION THAT
/// RECORDED IT, and the counting half survived because inverting the map just exchanges two
/// non-zero counts.
///
/// That is the defect rung 72's own spec names — *"the fifth instance of the
/// shipped-instrument-agrees-with-itself pattern in this family … the only defence that has ever
/// worked is a gate that FAILS when the two laws are the same one"* — reproduced here by me, in a
/// gate written to check that very rung. The expectation is now spelled out from the two clips
/// directly, so nothing in the assertion routes through the function under test.
#[test]
fn the_authority_label_partitions_the_march_and_the_handover_happens() {
    let traj = shared_march(&machine(&phi_arm()));
    let mut n = [0usize; 4];
    for p in &traj {
        let PointExtra::Shared { authority: au, g_fuel, g_gov, .. } = p.extra else {
            panic!("rung 72's point")
        };
        // SPELLED OUT, not delegated: `Fuel` means the FUEL leg's clip is the larger one.
        let want = if g_fuel <= 1e-12 && g_gov <= 1e-12 {
            Authority::Dormant
        } else if (g_fuel - g_gov).abs() <= 1e-12 {
            Authority::Tie
        } else if g_fuel > g_gov {
            Authority::Fuel
        } else {
            Authority::Gov
        };
        assert_eq!(au, want,
                   "the label names the leg whose clip is APPLIED: g_fuel = {g_fuel:e},                     g_gov = {g_gov:e} at s = {}", p.s);
        // and the label agrees with the APPLIED clip, which is an independent route to the same
        // claim: whichever leg is named must be the one `max` selected.
        if au == Authority::Fuel {
            assert!(g_fuel > g_gov, "`Fuel` must be the larger clip");
        }
        if au == Authority::Gov {
            assert!(g_gov > g_fuel, "`Gov` must be the larger clip");
        }
        n[match au {
            Authority::Dormant => 0, Authority::Tie => 1, Authority::Fuel => 2, Authority::Gov => 3,
        }] += 1;
    }
    assert!(n[2] > 0 && n[3] > 0,
            "BOTH legs must hold the actuator somewhere in this ramp, or the rung's subject — \
             the hand-over — is not exercised: dormant {} tie {} fuel {} gov {}",
            n[0], n[1], n[2], n[3]);
}

/// The joint initial condition converged, and the record of it is CONSTANT down the trajectory —
/// the sweep runs once, before the loop.
#[test]
fn the_four_way_joint_ic_converged_and_is_recorded_once() {
    let traj = shared_march(&machine(&phi_arm()));
    let PointExtra::Shared { ic_iters, ic_res, ic_order, share_law, .. } = traj[0].extra else {
        panic!("rung 72's point")
    };
    assert_eq!(ic_order, "rqvf", "the DECLARED order — rung 70's `g -> q -> v` with `f` appended");
    assert_eq!(share_law, "max", "and MIN-SELECT is the plant");
    assert!(ic_iters >= 1 && ic_iters <= 60, "iterations {ic_iters} inside the cap");
    assert!(ic_res <= 1e-9, "the sweep converged: residual {ic_res:e}");
    for p in &traj {
        let PointExtra::Shared { ic_iters: i2, ic_res: r2, ic_order: o2, share_law: s2, .. } =
            p.extra else { panic!() };
        assert_eq!((i2, r2.to_bits(), o2, s2), (ic_iters, ic_res.to_bits(), ic_order, share_law),
                   "all four are constants over the trajectory");
    }
}

// =============================================================================================
// 2 — THE REDUCE, BY DISPATCH
// =============================================================================================

/// **NO FUEL LEG ⇒ RUNG 71 or RUNG 70, BIT FOR BIT.** The entry test is `tau_gov` AND a fuel leg,
/// so dropping either leaves through the immediate parent's table.
#[test]
fn without_a_fuel_leg_the_march_is_rung_71s_bit_for_bit() {
    for (name, arm) in [("phi (rung 70)", phi_arm()), ("incidence (rung 71)", inc_arm())] {
        let m = machine(&arm);
        let got = march(&m, None, None, Some(TT4_MAX), Some(TAU_GOV));
        assert!(!got.is_empty(), "{name}: the parent marched");
        for p in &got {
            assert!(matches!(p.extra, PointExtra::Triple { .. }),
                    "{name}: with no fuel leg this is the PARENT's five-state point, not rung 72's");
        }
    }
}

/// **NO GOVERNOR ⇒ the fuel-leg-only arm, and it is NOT rung 72's point either.**
#[test]
fn without_a_governor_the_march_leaves_through_the_parent() {
    let m = machine(&phi_arm());
    let got = march(&m, Some(fuel_floor()), Some(lag()), None, None);
    assert!(!got.is_empty());
    assert!(got.iter().all(|p| !matches!(p.extra, PointExtra::Shared { .. })),
            "`tau_gov = None` is one fuel-side leg — an inherited arm, never this rung's");
}

/// The two reduce arms differ from the rung-72 march in the STATE KEYS, not merely in the extras —
/// so "it reduced" is a claim about the trajectory and not about a tag.
#[test]
fn the_reduce_arms_and_the_shared_march_are_different_trajectories() {
    let m = machine(&phi_arm());
    let shared = keys(&shared_march(&m));
    let parent = keys(&march(&m, None, None, Some(TT4_MAX), Some(TAU_GOV)));
    assert!(!shared.is_empty() && !parent.is_empty());
    assert!(shared != parent,
            "arming rung 52's leg beside the governor must MOVE the plant — if these agreed, the \
             entry test would be dispatching to the parent and every rung-72 gate would be \
             measuring rung 71");
}

// =============================================================================================
// 3 — THE FOUR ARMING REFUSALS
// =============================================================================================

#[test]
fn the_four_arming_guards_each_refuse_with_their_own_message() {
    let m = machine(&phi_arm());

    // GUARD A — `tau_gov` without `Tt4_max`.
    let a = message_of(|| { march(&m, Some(fuel_floor()), Some(lag()), None, Some(TAU_GOV)); });
    assert!(a.contains("governor with no set point"), "guard A: {a:?}");

    // GUARD D — the composition law. Set through the guard, so the refusal is reachable at all.
    let d = message_of(|| {
        let _g = ShareScope::set(&m, "product");
        march(&m, Some(fuel_floor()), Some(lag()), Some(TT4_MAX), Some(TAU_GOV));
    });
    assert!(d.contains("composition law") && d.contains("product"),
            "guard D names the law it refused: {d:?}");

    // GUARD C — an INSTANTANEOUS valve beside lagged fuel-side legs.
    let unlagged = LeverArm {
        bleed_lim: Some(BleedLimiter::with_tau(PHI, B, None)),
        stator_lim: Some(phi_stator()), ..Default::default()
    };
    let c = message_of(|| {
        march(&machine(&unlagged), Some(fuel_floor()), Some(lag()), Some(TT4_MAX), Some(TAU_GOV));
    });
    assert!(c.contains("INSTANTANEOUS valve"), "guard C: {c:?}");
}

/// **AND THE SHARE-LAW GUARD IS REACHED THROUGH THE SCOPE, WHICH IS THE ONLY WAY IN.** The control:
/// both admissible values march without raising, so guard D refuses a VALUE rather than a path.
///
/// # THE FIRST DRAFT OF THIS GATE ASSERTED `len() > 100` AND THE `sum` ARM FAILED IT
///
/// `sum` DOUBLE-CLIPS, so the fuel actually burnt falls further and the closure stops bracketing
/// earlier: the march takes its `break` arm and returns a SHORT trajectory. That is not a port
/// defect — driven at these exact settings, **Python returns 341 points on `max` and 84 on `sum`**,
/// and the Rust marcher returns the same two lengths.
///
/// The bar is written as a **same-run comparison** rather than as either number, because this file
/// reads no golden: `sum` ends strictly earlier than `max` on one machine in one test. A bar of
/// `> 100` was a guess about the physics dressed up as a check on the port, and it failed for the
/// physics being right.
#[test]
fn both_admissible_composition_laws_march_and_sum_starves_earlier() {
    let m = machine(&phi_arm());
    let n_max = { let _g = ShareScope::set(&m, "max"); shared_march(&m).len() };
    let n_sum = { let _g = ShareScope::set(&m, "sum"); shared_march(&m).len() };
    assert!(n_sum > 0 && n_max > 0, "both admissible laws march: max {n_max}, sum {n_sum}");
    assert!(n_sum < n_max,
            "`sum` double-clips, so it starves the burner and takes the march's break arm              EARLIER than min-select does: max {n_max}, sum {n_sum}");
}

/// **`sum` IS AN INSTRUMENT AND `max` IS THE PLANT, AND THEY MARCH DIFFERENT TRAJECTORIES.**
/// Without this the composition law would be a knob with no effect and every § 3 reading vacuous.
#[test]
fn the_two_composition_laws_march_different_plants() {
    let m = machine(&phi_arm());
    let a = { let _g = ShareScope::set(&m, "max"); keys(&shared_march(&m)) };
    let b = { let _g = ShareScope::set(&m, "sum"); keys(&shared_march(&m)) };
    assert!(!a.is_empty() && a != b,
            "`sum` DOUBLE-CLIPS: it must move the fuel actually burnt, or § 3's isolation \
             instrument is measuring the plant it is meant to differ from");
}

// =============================================================================================
// 4 — THE THIRTEEN WIDENED READERS, ON A REAL RUNG-72 TRAJECTORY
// =============================================================================================

/// **THE GATE THE FINDING NEEDED.** Each of these readers takes a `_ =>` fallback for any variant
/// it does not name, so before this slice widened them every one would have answered a rung-72
/// trajectory with a default, an empty set, or a panic — and nothing would have failed.
///
/// The `false`-in-a-filter pair is checked by requiring a NON-EMPTY result, which is the only
/// assertion shape that can tell "widened" from "silently dropped".
#[test]
fn the_widened_readers_answer_on_a_rung_72_trajectory() {
    let traj = shared_march(&machine(&phi_arm()));

    // `v_at_point` / `ic_at_point` — the panic-fallback pair.
    let v = turbojet::three_loop::v_at_point(&traj[0]);
    assert!(v.is_finite(), "`v_at_point` answers rather than refusing: {v}");
    let (its, res, order) = turbojet::three_loop::ic_at_point(&traj[0]);
    assert_eq!(order, "rqvf");
    assert!(its >= 1 && res <= 1e-9);

    // `asym_extra` — rung 52's reader, on a trajectory it has never seen.
    let (g, req) = turbojet::fuel_transient::asym_extra(&traj[0]);
    assert!(g >= 0.0 && req >= 0.0, "rung 52's reader answers: g = {g}, required = {req}");

    // `valve_of` — rung 65's.
    let (b, b_cmd) = turbojet::lagged_bleed::valve_of(&traj[0]);
    assert!(b.is_finite() && b_cmd.is_finite());

    // `riding` — THE `false`-IN-A-FILTER SITE. An un-widened version returns EMPTY and reports
    // perfect tracking, which is why the bar is non-emptiness and not a property of the members.
    let rid = turbojet::three_loop::riding(&traj, B);
    assert!(!rid.is_empty(),
            "the riding filter must ADMIT rung-72 points — an empty set here is exactly the \
             silent failure this slice's widening exists to prevent");
    for p in &rid {
        let PointExtra::Shared { v_regime, .. } = p.extra else { panic!() };
        assert_eq!(v_regime, Some(Regime::Riding));
    }
}

/// **AND `cross_extra` STILL REFUSES**, which is the one widening question answered NO. Rung 72
/// iterates the joint sweep UNDAMPED, so it carries `ic_iters`/`ic_res` and no `ic_damp`; admitting
/// it would hand a rung-67 reader a damping factor this integrator never computed.
#[test]
fn cross_extra_refuses_rung_72_for_rung_66s_reason() {
    let traj = shared_march(&machine(&phi_arm()));
    let msg = message_of(|| { turbojet::cross_loop::cross_extra(&traj[0]); });
    assert!(msg.contains("no joint-IC record") || msg.contains("ic_damp"),
            "the refusal names the missing key rather than the variant: {msg:?}");
}

/// **A STATOR-LESS RUNG-72 MARCH RUNS, AND ITS `v_regime` IS `None`.** This is the arming that
/// forced `Option<Regime>` — `shared_bill`'s `FG` cell, which has no inherited home at all.
#[test]
fn a_stator_less_shared_march_records_no_regime() {
    let bare = LeverArm { bleed_lim: Some(valve()), ..Default::default() };
    let traj = shared_march(&machine(&bare));
    assert!(traj.len() > 100, "the stator-less cell marches: {} points", traj.len());
    for p in &traj {
        let PointExtra::Shared { v_regime, v, v_cmd, .. } = p.extra else { panic!() };
        assert_eq!(v_regime, None,
                   "no stator solve ran, so there is no regime — `Dormant` would be a label the \
                    integrator never produced");
        assert_eq!(v, 0.0, "and the lever sits at its DESIGN setting");
        assert_eq!(v_cmd, 0.0);
    }
    // and the two filters answer `false` on it rather than panicking — Python compares `None`
    // against a string and gets `False`.
    assert!(turbojet::three_loop::riding(&traj, B).is_empty(),
            "a stator-less trajectory is not RIDING a stator it does not have");
}
