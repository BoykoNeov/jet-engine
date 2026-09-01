//! RUNG 72 — **TWO LOOPS ON ONE ACTUATOR**: rung 52's `phi` fuel leg armed BESIDE rung 47's `Tt4`
//! governor, so two limiters drive the SAME actuator. Six states, four clocks, four loops, THREE
//! actuators — `n = 4`, the last unoccupied shape after rungs 68–71 filled every `(3, m)` cell.
//!
//! **THE HEADLINE: A SHARED ACTUATOR ADDS A SWITCH BETWEEN PLANTS, NOT A LOOP.** Min-select makes
//! authority EXCLUSIVE, so the masked leg reaches the plant through a `max()` that is FLAT in it:
//! its column is `(−1, 0, 0, 0)`, the block is triangular, and this ONE plant IS rung 68, 69, 70
//! or 71 at every instant — polynomial for polynomial — plus a free pole at the masked leg's own
//! clock. `zeros = n_live − m_live`, and **the RANK CHANGES at the hand-over with no state, no
//! gain and no clock moving.**
//!
//! Ported from `tests/test_rung72.py` — **28 collected tests, of which 13 carry `slow` there.**
//! Both numbers are MEASURED (`pytest --collect-only -q -n0`, twice, once with `-m slow`), never
//! read off a sentence: slice AC step 4's header shipped a typed `slow` count that was wrong, and
//! its own reconciliation found `grep` reporting 28 `#[test]` where `cargo` ran 27 because the
//! 28th sat inside a doc comment. **The count below is reconciled against `cargo test -- --list`,
//! not against a `grep`.** The `slow` marker is dropped here per slice M's rule; `#[ignore]` is
//! re-introduced only against a MEASURED Rust cost, never inherited.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, 0 ADDED, 0 COLLAPSED, 3 SPLIT BY PARAMETER
//!
//! Python's three `@pytest.mark.parametrize("inc", [False, True])` sites each collect as TWO
//! tests, so they land here as two `#[test]` functions apiece rather than as one loop. That keeps
//! the collected counts equal on both sides, which is the only form in which "1:1" is checkable.
//!
//! | # | `tests/test_rung72.py` | here |
//! |---|---|---|
//! | 1 | `reduces_to_rung71_no_fuel_leg` | [`reduces_to_rung71_no_fuel_leg`] |
//! | 2 | `reduces_to_rung70_no_fuel_leg` | [`reduces_to_rung70_no_fuel_leg`] |
//! | 3 | `reduces_to_rung69_no_governor` | [`reduces_to_rung69_no_governor`] |
//! | 4 | `reduces_to_rung68_no_governor` | [`reduces_to_rung68_no_governor`] |
//! | 5 | `reduces_to_rung67_no_stator_no_fuel_leg` | [`reduces_to_rung67_no_stator_no_fuel_leg`] |
//! | 6 | `at_lever_returns_this_class` | [`at_lever_returns_this_class`] — **body substituted** |
//! | 7 | `charpoly_selftest` | [`charpoly_selftest_is_clean_on_both_matrices`] |
//! | 8 | `charpoly_selftest_catches_the_broken_recursion` | [`charpoly_selftest_catches_the_broken_recursion`] |
//! | 9 | `authority_changes_hands_once_inside_the_joint_window` | [`authority_changes_hands_once_inside_the_joint_window`] |
//! | 10a/b | `the_two_legs_cannot_see_each_other[False/True]` | `..._on_the_phi_arm` / `..._on_the_incidence_arm` |
//! | 11a/b | `the_masked_leg_reaches_the_plant_through_nothing[False/True]` | `..._on_the_phi_arm` / `..._on_the_incidence_arm` |
//! | 12 | `the_four_cells_are_rungs_68_69_70_and_71` | [`the_four_cells_are_rungs_68_69_70_and_71`] |
//! | 13 | `the_rank_changes_at_the_hand_over_with_nothing_moving` | [`the_rank_changes_at_the_hand_over_with_nothing_moving`] |
//! | 14 | `only_the_rung71_cell_has_a_live_determinant` | [`only_the_rung71_cell_has_a_live_determinant`] |
//! | 15 | `the_free_pole_separates_the_laws_ONLY_at_unmatched_clocks` | [`the_free_pole_separates_the_laws_only_at_unmatched_clocks`] |
//! | 16 | `the_sum_law_gives_the_masked_leg_its_rank_back` | [`the_sum_law_gives_the_masked_leg_its_rank_back`] |
//! | 17a/b | `the_masked_leg_still_buys_something[False/True]` | `..._on_the_phi_arm` / `..._on_the_incidence_arm` |
//! | 18 | `refuses_tau_gov_without_a_set_point` | [`refuses_tau_gov_without_a_set_point`] |
//! | 19 | `refuses_a_forced_release_edge` | [`refuses_a_forced_release_edge`] — **introspection** |
//! | 20 | `refuses_an_instantaneous_valve` | [`refuses_an_instantaneous_valve`] |
//! | 21 | `refuses_an_undeclared_composition_law` | [`refuses_an_undeclared_composition_law`] |
//! | 22 | `the_rk4_floor_is_on_all_four_clocks` | [`the_rk4_floor_is_on_all_four_clocks`] — **needle strengthened** |
//! | 23 | `the_composition_law_lives_in_one_place` | [`the_composition_law_lives_in_one_place`] |
//! | 24 | `authority_labels_the_switch_itself` | [`authority_labels_the_switch_itself`] |
//! | 25 | `this_rungs_march_MOVES_and_all_four_loops_are_live` | [`this_rungs_march_moves_and_all_four_loops_are_live`] |
//!
//! # EVERY READER ARGUMENT BELOW THE FIFTH IS THE READER's OWN DEFAULT, AND THEY ARE NOT ALL EQUAL
//!
//! **This is the highest-risk transcription surface in the step and it is the one § 5.27.6 (i)
//! already burned a slice on** — there, a shipped row was measured at `every = 40` while the
//! fixture passed `every = 10`, and five numbers were wrong. Every rung-72 reader call in the
//! Python file passes exactly `FLIGHT, LO, HI, TT4_MAX, SM` plus at most `inc=` or `clocks=`, so
//! `r`, `s_settle`, `ds`, `v_max`, `every` and (where not passed) `taus`/`clocks` come from
//! **`turbojet/engine.py`'s own `def` line** — never from this module's `DS`, which is a march
//! constant and agrees with only two of the five.
//!
//! | reader | `ds` | `every` | grid |
//! |---|---|---|---|
//! | `authority_law` | **0.005** | — | [`CLOCKS`], two arms |
//! | `shared_gains` | **0.002** | **2** | `taus` = matched |
//! | `shared_cells` | **0.002** | **2** | [`CLOCKS`], two arms |
//! | `mask_discriminator` | **0.002** | **4** | [`MD_CLOCKS`], **THREE** arms, its own |
//! | `shared_bill` | **0.005** | — | `taus` = matched |
//!
//! And [`CLOCKS`] — the constant the Python file defines and passes explicitly to two of them — is
//! **bit-identical to those two readers' own defaults**, so passing it substitutes nothing. That
//! is recorded rather than assumed, because the same sentence read the other way is exactly the
//! § 5.27.6 (i) defect.
//!
//! # WHAT OVERLAPS `slice_ad_cells.rs` / `slice_ad_march.rs`, AND WHY IT IS PORTED ANYWAY
//!
//! Four gates here have a relative in an earlier step's file: the three arming refusals
//! (`slice_ad_march.rs::the_four_arming_guards_each_refuse_with_their_own_message`) and the RK4
//! floor (`slice_ad_cells.rs::the_floor_fires_on_a_message_that_names_this_rung_and_its_own_
//! argument` + `..._admits_its_own_boundary_exactly`). They are ported anyway because this file's
//! contract is a **1:1 map of the shipped suite** — a hole in it would be invisible to every count
//! — and because the two files gate different things: the earlier ones drive the CELL directly,
//! these drive it through `_stator_march`, which is the entry the suite actually uses.
//!
//! # THE FLOOR's SHIPPED NEEDLE DISCRIMINATES NOTHING, SO THIS PORT DOES NOT INHERIT IT
//!
//! `tests/test_rung72.py:445` fires the floor under `match=r"FOUR actuator states"`, and § 5.28
//! (v) measured that phrase in rungs 72, 73 **and** 74's messages, whose conditions are identical
//! character for character. **The shipped Python gate would pass with rung 73's or rung 74's floor
//! installed.** The ported gate reads `rung-72` and `-1/tau_f` instead, which are unique to this
//! rung's message, and is therefore strictly stronger than its source.

use std::panic::catch_unwind;

use turbojet::cross_loop::build_cross_loop_cascade;
use turbojet::cross_split::build_cross_split_cascade;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Authority, Floor, FuelLimiters, FuelPoint, PointExtra,
    SurgeLimiter,
};
use turbojet::full_split::build_full_split_cascade;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::{build_reference_split_cascade, StatorIncidenceLimiter, C64};
use turbojet::shared_actuator::{
    applied_clip, authority, authority_law, build_shared_actuator_cascade, charpoly_selftest,
    mask_discriminator, quartic_roots_c, shared_bill, shared_cells, shared_gains, shared_march,
    ShareScope,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{build_three_loop_cascade, StatorLimiter};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung72.py`'s module constants, verbatim.

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
const V_MAX: f64 = 0.20;
/// `PHI / FLOOR - 1.0` — the expression Python spells, never a typed decimal. Rung 69's
/// constructor asserts `m_lim == T_c − 1/phi_lim`, so a rounded constant breaks a wall identity
/// silently.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
/// **RUNG 67's imposed redline, VERBATIM through rungs 70/71/72.**
const TT4_MAX: f64 = 1200.0;

/// Python's module-level `CLOCKS` — the MATCHED arm and the WIDE-CELL arm, `(tau_f, tau_gov,
/// tau_q, tau_s)`.
///
/// The second exists because the incidence arm's GOVERNOR-authority cell — rung 71's own — holds
/// **1 point of 35** at matched clocks: a fast governor and a slow fuel leg hand over EARLY, and a
/// slow valve keeps the stator riding LATE.
///
/// **AND IT IS BIT-IDENTICAL TO `authority_law`'s AND `shared_cells`'s OWN DEFAULT**, so the two
/// call sites that pass it substitute nothing. Recorded because the opposite reading of the same
/// sentence is § 5.27.6 (i)'s defect exactly.
const CLOCKS: [(f64, f64, f64, f64); 2] = [(0.05, 0.05, 0.05, 0.05), (0.20, 0.01, 0.50, 0.05)];

// ----------------------------------------------------------- THE READERS' OWN DEFAULTS
//
// See the module header's table. These are read off `turbojet/engine.py`'s `def` lines, not off
// the constants above — `DS` agrees with two of the five and NOT with the other three.

/// Shared by all five readers: `r = 0.5`, `s_settle = 1.2`, `v_max = 0.20`. Equal to this module's
/// `R`, `SETTLE` and `V_MAX`, which is why they are spelled from those and not re-typed.
const RD_R: f64 = R;
const RD_SETTLE: f64 = SETTLE;
const RD_V_MAX: f64 = V_MAX;

/// `authority_law(..., ds=0.005)`.
const AL_DS: f64 = 0.005;
/// `shared_gains(..., taus=(0.05,)*4, ds=0.002, every=2)`.
const SG_TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const SG_DS: f64 = 0.002;
const SG_EVERY: usize = 2;
/// `shared_cells(..., ds=0.002, every=2)`.
const SC_DS: f64 = 0.002;
const SC_EVERY: usize = 2;
/// `mask_discriminator`'s OWN three-arm grid — **not** [`CLOCKS`], and the Python test passes no
/// `clocks=`. Arm 0 is MATCHED (`tau_f == tau_gov`) and is the confound; arms 1 and 2 are the
/// discriminator.
const MD_CLOCKS: [(f64, f64, f64, f64); 3] = [
    (0.05, 0.05, 0.05, 0.05), (0.05, 0.08, 0.05, 0.05), (0.02, 0.09, 0.05, 0.05),
];
const MD_DS: f64 = 0.002;
const MD_EVERY: usize = 4;
/// `shared_bill(..., taus=(0.05,)*4, ds=0.005)`.
const SB_TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const SB_DS: f64 = 0.005;

/// § 1.3's law, per cell: `zeros = n_live − m_live`, and `n_live = 3` always. Python's
/// `PREDICTED`.
const PREDICTED: [((bool, Authority), usize); 4] = [
    ((false, Authority::Fuel), 2), ((false, Authority::Gov), 1),
    ((true, Authority::Fuel), 1), ((true, Authority::Gov), 0),
];

/// Python's `PARENT` — which rung each cell IS, quoted in the failure messages exactly as there.
const PARENT: [((bool, Authority), &str); 4] = [
    ((false, Authority::Fuel), "rung 68"), ((false, Authority::Gov), "rung 70"),
    ((true, Authority::Fuel), "rung 69"), ((true, Authority::Gov), "rung 71"),
];

// ============================================================================== the fixtures

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

/// Python's module-scoped `design` fixture. **Rebuilt per test rather than shared**, for
/// `tests/rung68.rs`'s reason: each test is its own thread and [`ScheduledStatorCore`]'s `Cell`
/// fields — which ARE the dynamically scoped state — are deliberately not `Sync`.
fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// Python's `_shared(design, **kw)`.
fn shared_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_shared_actuator_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0,
                                          arm))
}

fn full71_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_full_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn cross70_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_cross_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn ref69_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_reference_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0,
                                          arm))
}

fn triple68_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_three_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn cross67_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_cross_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

use turbojet::bleed_transient::LeverArm;

fn valve() -> BleedLimiter { BleedLimiter::from_margin_tau(&lp(), B, SM, Some(TAU)) }
fn phi_stator() -> StatorLimiter { StatorLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S)) }
fn inc_stator() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S))
}
fn surge() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM)) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

/// The valve-only arm — Python's `_shared(design, bleed_lim=_valve())`, the receiver every § 0–§ 4
/// reader is called on. The readers build their OWN rig, so the receiver's stator arming does not
/// reach the march.
fn valve_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), ..Default::default() }
}

fn phi_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_lim: Some(phi_stator()), ..Default::default() }
}

fn inc_arm() -> LeverArm {
    LeverArm { bleed_lim: Some(valve()), stator_inc: Some(inc_stator()), ..Default::default() }
}

/// Python's `_march(m, **kw)` — `_stator_march(FLIGHT, LO, HI, R, SETTLE, DS, **kw)[0]`.
fn march(
    m: &ScheduledStatorCore, floor: Option<Floor>, lg: Option<AsymmetricLag>,
    tt4_max: Option<f64>, tau_gov: Option<f64>,
) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: floor, tt4_max };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg,
                          &MarchScope { lag: lg, tau_gov, ..MarchScope::DEFAULT }).0
}

/// The governor-only march — `Tt4_max=TT4_MAX, tau_gov=TAU_GOV`, no fuel leg.
fn gov_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    march(m, None, None, Some(TT4_MAX), Some(TAU_GOV))
}

/// The fuel-leg-only march — `surge=_surge(), lag=_lag()`, no governor.
fn fuel_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    march(m, Some(surge()), Some(lag()), None, None)
}

/// Python's `p["b"]` — a `KeyError` off a trajectory that records no valve, which is a panic here.
fn b_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { b, .. } | PointExtra::Triple { b, .. }
        | PointExtra::CrossCascade { b, .. } | PointExtra::Cascade { b, .. }
        | PointExtra::Valve { b, .. } => b,
        _ => panic!("rung-72's `_keys` reads `b` with a bare index"),
    }
}

/// Python's `p["v"]` — likewise, and it is the key the rung-67 arm DROPS.
fn v_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { v, .. } | PointExtra::Triple { v, .. } => v,
        _ => panic!("rung-72's `_keys` reads `v` with a bare index"),
    }
}

/// Python's `_keys(traj)` — the NINE-tuple, compared BIT for bit.
fn keys(traj: &[FuelPoint]) -> Vec<[u64; 9]> {
    traj.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits(),
                  b_of(p).to_bits(), v_of(p).to_bits()])
        .collect()
}

/// Python's `_keys(traj, ks)` with the SHORTENED tuple — **eight keys, `v` dropped**, because the
/// rung-67 arm arms no stator and a bare `p["v"]` would raise there. The narrowing is Python's and
/// is ported rather than silently widened.
fn keys8(traj: &[FuelPoint]) -> Vec<[u64; 8]> {
    traj.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits(), b_of(p).to_bits()])
        .collect()
}

/// `pytest.raises(AssertionError, match=...)` — the message is RETURNED so a caller can put a
/// second, stronger needle on it (see [`the_rk4_floor_is_on_all_four_clocks`]).
///
/// `AssertUnwindSafe` because every rung-72 machine carries `Cell` fields — the dynamically
/// scoped state — and those are exactly what `UnwindSafe` is designed to flag. The state is
/// rebuilt per test here, so a poisoned one cannot outlive the assertion.
fn panics_with<F: FnOnce()>(f: F, needle: &str) -> String {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(std::panic::AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    let msg = match out {
        Ok(()) => String::new(),
        Err(e) => e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    };
    assert!(msg.contains(needle), "expected a refusal naming {needle:?}; got {msg:?}");
    msg
}

/// The `(inc, Authority)` key's entry in a `Vec`-of-pairs the port uses where Python has a dict.
fn cell<'a, K: PartialEq, V>(v: &'a [(K, V)], k: &K) -> &'a V {
    &v.iter().find(|(kk, _)| kk == k).expect("the key is present, as Python's `[]` requires").1
}

// =============================================================================================
// THE REDUCE SPINE — five arms, all by DISPATCH, all bit-for-bit (anchor P8)
//
// NOT MARKED `slow` in Python, deliberately: each runs two 341-point marches, and the reduce
// spine is the project's spine. Nothing is marked here either, per slice M's rule.
// =============================================================================================

/// No fuel leg + an incidence stator + the governor IS rung 71, entry for entry. The dispatch
/// never enters this rung's march for a plant it does not own.
#[test]
fn reduces_to_rung71_no_fuel_leg() {
    let a = gov_march(&shared_of(&inc_arm()));
    let b = gov_march(&full71_of(&inc_arm()));
    assert_eq!(keys(&a), keys(&b));
}

#[test]
fn reduces_to_rung70_no_fuel_leg() {
    let a = gov_march(&shared_of(&phi_arm()));
    let b = gov_march(&cross70_of(&phi_arm()));
    assert_eq!(keys(&a), keys(&b));
}

#[test]
fn reduces_to_rung69_no_governor() {
    let a = fuel_march(&shared_of(&inc_arm()));
    let b = fuel_march(&ref69_of(&inc_arm()));
    assert_eq!(keys(&a), keys(&b));
}

#[test]
fn reduces_to_rung68_no_governor() {
    let a = fuel_march(&shared_of(&phi_arm()));
    let b = fuel_march(&triple68_of(&phi_arm()));
    assert_eq!(keys(&a), keys(&b));
}

/// A governor and a valve with NO stator and NO fuel leg is rung 67 — and it stays rung 67 even
/// though this rung's dispatch no longer asks for a stator, because it asks for BOTH fuel legs
/// instead.
///
/// **THE KEY TUPLE IS EIGHT WIDE HERE AND NINE EVERYWHERE ELSE**, which is Python's own narrowing:
/// no stator is armed, so `p["v"]` does not exist on either side.
#[test]
fn reduces_to_rung67_no_stator_no_fuel_leg() {
    let a = gov_march(&shared_of(&valve_arm()));
    let b = gov_march(&cross67_of(&valve_arm()));
    assert_eq!(keys8(&a), keys8(&b));
}

/// **THE TENTH INSTANCE of the trap rungs 61–71 each hit**: hand back the parent's class and every
/// reader measures rung 71's plant while reporting rung 72's.
///
/// # PYTHON's `type(m) is SharedActuatorTransient` HAS NO RUNTIME COUNTERPART, AND `ptr::eq` IS NOT ONE
///
/// Every rung in this family is a [`ScheduledStatorCore`] and the rung is the TABLE it carries.
/// Comparing the table's address would test the optimiser (slice AA step 1's recorded
/// `ptr::eq`-on-a-`const` trap), so the sibling is instead made to **exercise a cell only rung
/// 72's table has**: it must march with BOTH fuel-side legs armed — a `phi` floor and a lag beside
/// `tau_gov` — which rung 71's inherited table refuses outright with *"n = 4, m = 3"*. A sibling
/// handed back carrying the parent's table panics there, and one carrying rung 72's marches and
/// records the six-state point.
///
/// Python's second half — `stator_inc is not None and bleed_lim is not None` — ports as the plain
/// field check beside it.
#[test]
fn at_lever_returns_this_class() {
    let m = shared_of(&valve_arm()).at_lever(&inc_arm());
    // Python's second half, verbatim.
    assert!(m.arming().inc.is_some() && m.fuel.inner.lever.lim.is_some());
    let t = march(&m, Some(surge()), Some(lag()), Some(TT4_MAX), Some(TAU_GOV));
    assert!(!t.is_empty(), "the sibling marches");
    assert!(t.iter().all(|p| matches!(p.extra, PointExtra::Shared { .. })),
            "the sibling must carry rung 72's OWN table — the parent's refuses two fuel-side legs \
             beside `tau_gov` outright, which is what `type(m) is SharedActuatorTransient` \
             asserts");
}

// =============================================================================================
// THE INSTRUMENT — gated against itself, because a broken one looked plausible
// =============================================================================================

/// `_charpoly4`'s first version had `A` where Faddeev–LeVerrier needs `M_{k−1}` and returned a
/// WRONG polynomial with entirely plausible downstream numbers: a stable-looking spectrum, a
/// determinant of 5.9e+05 and a root residual of 1e−09, because the root finder was faithfully
/// solving the wrong polynomial. Nothing downstream could tell, so the polynomial is checked
/// against an INDEPENDENT trace and cofactor determinant and against a triangular matrix whose
/// spectrum is its own diagonal.
#[test]
fn charpoly_selftest_is_clean_on_both_matrices() {
    let out = charpoly_selftest();
    for (name, d) in out.iter() {
        assert!(d.trace_err < 1e-9, "{name}: {d:?}");
        assert!(d.det_err < 1e-9, "{name}: {d:?}");
        assert!(d.det_vs_a0 < 1e-9, "{name}: {d:?}");
        assert!(d.resid < 1e-9, "{name}: {d:?}");
    }
    // the triangular arm: the spectrum IS the diagonal, and it is REAL
    let tri = cell(&out, &"triangular");
    assert!(tri.diag_err.expect("the triangular arm carries it") < 1e-9);
    assert!(tri.max_imag.expect("the triangular arm carries it") < 1e-9);
}

/// **MEASURE THE DETECTOR'S SENSITIVITY, do not assert it.** The self-test is only worth having if
/// it FAILS on the bug it was written for, so the bug is rebuilt and fed to it.
///
/// # THE FOLD HERE IS NAIVE BY CONSTRUCTION, AND THAT IS A DECISION AND NOT AN ACCIDENT
///
/// Python's `broken` uses `sum(...)` twice — once over the inner matrix product and once over the
/// trace — and § 5.28.3 (d) measured that CPython 3.12+'s `sum()` is Neumaier-COMPENSATED where
/// PyPy's is a naive fold. This reproduction is the naive fold, i.e. PyPy's, which is the arm the
/// port is bit-compared against. It cannot matter to the assertion — the bar is `> 1.0` on a
/// deliberately broken recursion whose error is tens of kelvin-equivalents — but a later reader
/// asking *which `sum()` is this* deserves the answer in the file rather than in a plan.
#[test]
fn charpoly_selftest_catches_the_broken_recursion() {
    /// the bug: `A` where the Faddeev–LeVerrier recursion needs `M_{k−1}`.
    fn broken(a: &[[f64; 4]; 4]) -> [f64; 5] {
        let n = 4usize;
        let mut c = [1.0f64; 5];
        // `M = [row[:] for row in A]` at `k == 1` — hoisted to the initialiser, because a
        // sentinel here would be a value the loop never reads.
        let mut m = *a;
        for k in 1..=n {
            if k > 1 {
                let mut t = [[0.0f64; 4]; 4];
                for (i, row) in t.iter_mut().enumerate() {
                    for (j, x) in row.iter_mut().enumerate() {
                        *x = a[i][j] + if i == j { c[k - 1] } else { 0.0 };
                    }
                }
                let mut prod = [[0.0f64; 4]; 4];
                for (i, row) in prod.iter_mut().enumerate() {
                    for (j, x) in row.iter_mut().enumerate() {
                        let mut acc = 0.0f64;
                        for (tt, trow) in t.iter().enumerate() {
                            acc += a[i][tt] * trow[j];
                        }
                        *x = acc;
                    }
                }
                m = prod;
            }
            let mut tr = 0.0f64;
            for (i, row) in m.iter().enumerate() {
                tr += row[i];
            }
            c[k] = -tr / k as f64;
        }
        c
    }

    let tri: [[f64; 4]; 4] = [[-20.0, 7.0, -3.0, 9.0], [0.0, -25.0, 4.0, -6.0],
                              [0.0, 0.0, -30.0, 8.0], [0.0, 0.0, 0.0, -50.0]];
    let mut got: Vec<f64> = quartic_roots_c(&broken(&tri)).iter().map(|z: &C64| z.re).collect();
    got.sort_by(|x, y| x.partial_cmp(y).expect("finite real parts"));
    let mut diag: Vec<f64> = (0..4).map(|i| tri[i][i]).collect();
    diag.sort_by(|x, y| x.partial_cmp(y).expect("a finite diagonal"));
    let worst = diag.iter().zip(got.iter()).map(|(a, b)| (a - b).abs())
                    .fold(f64::NEG_INFINITY, f64::max);
    assert!(worst > 1.0,
            "the broken recursion must be CAUGHT by the triangular arm, or the self-test is \
             ceremony: worst {worst}, diag {diag:?}, got {got:?}");
}

// =============================================================================================
// § 0 — WHO HOLDS THE ACTUATOR (anchor § 0)
// =============================================================================================

/// The plant splits into a fuel-leg interval and a governor interval, with ONE hand-over, and it
/// sits INSIDE the joint window on every arm — which is what lets § 2 measure a rank change on
/// BOTH sides of it on ONE trajectory, with no second plant.
#[test]
fn authority_changes_hands_once_inside_the_joint_window() {
    let al = authority_law(&shared_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, &CLOCKS,
                           RD_R, RD_SETTLE, AL_DS, RD_V_MAX);
    assert!(al.one_handover,
            "{:?}", al.arms.iter().map(|a| a.handovers.clone()).collect::<Vec<_>>());
    for a in al.arms.iter() {
        assert!(a.joint.n > 0, "{:?}", a.taus);
        assert!(a.handover_inside, "{:?} {:?} {:?} {:?}",
                a.inc, a.taus, a.handovers, a.joint);
        // the masked leg is RIDING and reaching nothing, not dormant
        assert!(a.both_want as f64 > 0.5 * a.n as f64, "{}", a.both_want);
    }
    // the WIDE-CELL arm reaches BOTH authority cells inside the joint window, on both arms
    assert!(al.both_cells_everywhere);
}

// =============================================================================================
// § 1 — THE FOUR EXACT ZEROS (anchor P4, P7)
// =============================================================================================

/// `F_r = R_f = 0` EXACTLY, so `pair_FR = 0` exactly — both legs solve from the SCHEDULED fuel
/// (rungs 47/52's own discipline, inherited verbatim). Rung 66's two loops on one VARIABLE gave a
/// pair product of exactly 1; two loops on one ACTUATOR give exactly 0. The two corners of one
/// question.
fn the_two_legs_cannot_see_each_other(inc: bool) {
    let g = shared_gains(&shared_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, SG_TAUS, inc,
                         RD_R, RD_SETTLE, SG_DS, RD_V_MAX, SG_EVERY)
        .expect("§ 1's march converges");
    assert_eq!(g.worst_f_r, Some(0.0));
    assert_eq!(g.worst_r_f, Some(0.0));
    assert_eq!(g.worst_pair_fr, Some(0.0));
}

#[test]
fn the_two_legs_cannot_see_each_other_on_the_phi_arm() {
    the_two_legs_cannot_see_each_other(false);
}

#[test]
fn the_two_legs_cannot_see_each_other_on_the_incidence_arm() {
    the_two_legs_cannot_see_each_other(true);
}

/// `C_masked = V_masked = 0` EXACTLY — not small. `max()` is FLAT in the masked clip, so the
/// coupling is absent rather than weak. This is the GATED quantity; the free pole at
/// `−1/tau_masked` follows from it ALGEBRAICALLY and is reported, never gated — rung 67 gate 9's
/// retraction in a third shape.
///
/// And the LIVE gains are checked non-zero on the same points, so "exactly zero everywhere" is not
/// being bought with a decoupled instrument.
fn the_masked_leg_reaches_the_plant_through_nothing(inc: bool) {
    let g = shared_gains(&shared_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, SG_TAUS, inc,
                         RD_R, RD_SETTLE, SG_DS, RD_V_MAX, SG_EVERY)
        .expect("§ 1's march converges");
    assert_eq!(g.worst_mask_leak, Some(0.0));
    let live = g.min_live_gain.expect("§ 1 samples at least one interior point");
    assert!(live > 1e-6, "{live}");
    assert!(g.by_authority_fuel > 0 && g.by_authority_gov > 0);
}

#[test]
fn the_masked_leg_reaches_the_plant_through_nothing_on_the_phi_arm() {
    the_masked_leg_reaches_the_plant_through_nothing(false);
}

#[test]
fn the_masked_leg_reaches_the_plant_through_nothing_on_the_incidence_arm() {
    the_masked_leg_reaches_the_plant_through_nothing(true);
}

// =============================================================================================
// § 2 — THE FOUR CELLS: ONE PLANT, FOUR PARENT RUNGS (anchor P1, P2, P3)
// =============================================================================================

fn cells() -> turbojet::shared_actuator::SharedCells {
    shared_cells(&shared_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, &CLOCKS, RD_R,
                 RD_SETTLE, SC_DS, RD_V_MAX, SC_EVERY).expect("§ 2's marches converge")
}

/// **THE RUNG.** Every cell's zero count is `n_live − m_live`, and every cell's characteristic
/// polynomial is the PARENT rung's own times `(lam + 1/tau_masked)` — rebuilt from the SHIPPED
/// three-loop readers, so two independent instruments reach one polynomial.
///
/// The comparison is on COEFFICIENTS, not roots: in the rung-68 cell the parent has a DOUBLE zero
/// root and this rung a TRIPLE one, and a repeated root resolves only to the square root of the
/// working precision (measured: individual roots at 3e−07 while every invariant sits at 1e−13).
///
/// **`zeros` IS A WHOLE-VECTOR CLAIM AND NOT A MEMBERSHIP ONE.** Python asserts
/// `d["zeros"] == [PREDICTED[key]]` against a sorted `set` — a cell that reached TWO distinct
/// counts fails there, and `contains` would pass it. The `assert_eq!` on the `Vec` is the faithful
/// spelling, and `law_holds` is the same claim taken over all four at once.
#[test]
fn the_four_cells_are_rungs_68_69_70_and_71() {
    let c = cells();
    assert!(c.all_four_cells, "{:?}", c.cells.iter().map(|(k, _)| *k).collect::<Vec<_>>());
    assert!(c.law_holds,
            "{:?}", c.cells.iter().map(|(k, d)| (*k, d.zeros.clone())).collect::<Vec<_>>());
    for (key, d) in c.cells.iter() {
        let want = *cell(&PREDICTED, key);
        assert_eq!(d.zeros, vec![want], "{key:?} {} {:?}", cell(&PARENT, key), d.zeros);
        assert!(d.n >= 5, "{key:?} {}", d.n);
        assert!(d.gap < 1e-10, "{key:?} {} {}", cell(&PARENT, key), d.gap);
    }
    // the two readers land on the SAME manifold base point, so the match is not a coincidence of
    // two different points (the alternative hypothesis for the rung-68 cell's precision)
    assert_eq!(c.worst_v_gap, 0.0);
}

/// **THE DISCONTINUITY.** On ONE trajectory the zero count is 2 before the hand-over and 1 after
/// (`phi` arm), 1 and 0 (incidence arm) — with no state, no gain and no clock changing. No earlier
/// rung in this family could exhibit it, because none had a quantity that could change without
/// something moving.
#[test]
fn the_rank_changes_at_the_hand_over_with_nothing_moving() {
    let c = cells();
    for inc in [false, true] {
        let lo = &cell(&c.cells, &(inc, Authority::Gov)).zeros;
        let hi = &cell(&c.cells, &(inc, Authority::Fuel)).zeros;
        assert_eq!(*hi, vec![lo[0] + 1], "{inc} {hi:?} {lo:?}");
    }
}

/// `det J` is non-zero in exactly ONE of the four cells — the incidence arm under governor
/// authority, which is rung 71's own plant and the only full-rank one in the family. That is rung
/// 71 § 1.3's factorisation surviving a rung that adds no factor: the masked leg multiplies it by
/// `−1/tau_masked` and nothing else.
#[test]
fn only_the_rung71_cell_has_a_live_determinant() {
    let c = cells();
    let live: Vec<(bool, Authority)> =
        c.cells.iter().filter(|(_, d)| d.zeros == vec![0]).map(|(k, _)| *k).collect();
    assert_eq!(live, vec![(true, Authority::Gov)], "{live:?}");
}

// =============================================================================================
// § 3 — THE ISOLATION INSTRUMENT, AND ITS OWN CONFOUND (anchor P5)
// =============================================================================================

fn discriminator() -> turbojet::shared_actuator::MaskDiscriminator {
    mask_discriminator(&shared_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, &MD_CLOCKS,
                       false, RD_R, RD_SETTLE, MD_DS, RD_V_MAX, MD_EVERY)
        .expect("§ 3's marches converge")
}

/// **THE CONFOUND IS GATED BESIDE THE RESULT.** At `tau_f = tau_g` the SUM law has `(1, −1, 0, 0)`
/// as an exact eigenvector with eigenvalue `−1/tau`, so the free-pole test passes under BOTH laws
/// and separates nothing. Unmatch the two fuel clocks and min-select keeps the pole to 1e−14 while
/// SUM loses it by ten orders of magnitude.
///
/// Gating the confound is the point: a discriminator quoted from the matched arm alone is a
/// discriminator that never tested anything.
#[test]
fn the_free_pole_separates_the_laws_only_at_unmatched_clocks() {
    let md = discriminator();
    let max_un = md.max_pole_unmatched.expect("two unmatched arms");
    let sum_un = md.sum_pole_unmatched.expect("two unmatched arms");
    let sum_mt = md.sum_pole_matched.expect("one matched arm");
    assert!(max_un < 1e-12, "{max_un}");
    assert!(sum_un > 1e-3, "{sum_un}");
    // THE CONFOUND: at matched clocks the SUM law passes the same test
    assert!(sum_mt < 1e-12, "{sum_mt}");
}

/// Restoring both legs' authority WITHOUT changing the loop count moves the zero count by exactly
/// one — at FUEL-authority points, where min-select was masking a leg, and not at governor-
/// authority ones on this arm. So the count is not blind to masking after all (the anchor's D5
/// said it would be; it is scored as refuted in the spec).
///
/// Python iterates `md["zeros_max"]` / `md["zeros_sum"]`, which are per-arm views built from
/// `arms`; the port carries the arms and the view is taken here, which is the same iteration in
/// the same order.
#[test]
fn the_sum_law_gives_the_masked_leg_its_rank_back() {
    let md = discriminator();
    for a in md.arms.iter() {
        let z = cell(&a.law_max.zeros, &Authority::Fuel);
        assert_eq!(*z, vec![2], "{:?} {:?}", a.taus, a.law_max.zeros);
    }
    for a in md.arms.iter() {
        let z = cell(&a.law_sum.zeros, &Authority::Fuel);
        assert_eq!(*z, vec![1], "{:?} {:?}", a.taus, a.law_sum.zeros);
    }
}

// =============================================================================================
// § 4 — THE LEDGER (anchor P6)
// =============================================================================================

/// A spectral reading says a masked leg is coupled to nothing; the ledger says otherwise, because
/// authority is a function of `s` and a leg masked LATE held the actuator EARLY. The fuel leg's
/// marginal `phi` credit is POSITIVE — and tiny, ~0.1–1.2 % of its solo credit.
///
/// **ITS SOLO CELL IS QUOTED BESIDE THE RATIO AND THAT IS NOT A FORMALITY**: rung 52's leg ALONE
/// holds `max Tt4` at the initial 1000 K (it starves the accel outright, `E = 0`), so the `kept`
/// denominator is taken on a trajectory no other cell shares.
fn the_masked_leg_still_buys_something(inc: bool) {
    let b = shared_bill(&shared_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, SB_TAUS, inc,
                        RD_R, RD_SETTLE, SB_DS, RD_V_MAX);
    assert!(b.fuel_marginal_phi > 0.0, "{}", b.fuel_marginal_phi);
    let kept_f = cell(&b.kept, &"F").expect("the solo `F` credit is non-zero in `I`");
    assert!(kept_f > 0.0 && kept_f < 0.10, "{kept_f}");
    assert!(b.phi_full > b.phi_no_fuel);            // it does buy phi
    // AND IT DOES SPEND THE GOVERNOR'S CURRENCY, in OPPOSITE directions on the two readings: the
    // exceedance INTEGRAL improves, the PEAK gets worse. The anchor's P6 predicted the peak
    // unmoved; it is refuted, and the sign is the claim (magnitudes disclaimed).
    assert!(b.fuel_marginal_tt4 > 0.0);             // integral: a credit
    assert!(b.tt4_full > b.tt4_no_fuel);            // peak: a debit
    // the degenerate solo cell, recorded so the ratio above cannot be read without it
    let f = cell(&b.cells, &"F");
    assert_eq!(f.e, 0.0);
    // `pytest.approx(LO, abs=1e-6)` — an ABSOLUTE tolerance, kept as one. Upgrading it to a bit
    // comparison would be a stricter claim than the suite makes.
    assert!((f.max_tt4 - LO).abs() <= 1e-6, "{}", f.max_tt4);
}

#[test]
fn the_masked_leg_still_buys_something_on_the_phi_arm() {
    the_masked_leg_still_buys_something(false);
}

#[test]
fn the_masked_leg_still_buys_something_on_the_incidence_arm() {
    the_masked_leg_still_buys_something(true);
}

// =============================================================================================
// THE REFUSALS (anchor P10)
// =============================================================================================

#[test]
fn refuses_tau_gov_without_a_set_point() {
    let m = shared_of(&inc_arm());
    panics_with(|| { march(&m, Some(surge()), Some(lag()), None, Some(TAU_GOV)); },
                "governor with no set point");
}

/// Refused TWICE OVER, and the outer refusal is STRUCTURAL — rung 71 § 8.2's own reading,
/// inherited: `_stator_march`, the entry every reader on this ladder actually calls, does not plumb
/// `s_off`/`tau_rel` at all, so they cannot reach a march even by mistake. The assert in
/// `integrate_fuel` is the inner guard for a caller that goes around it, and it is reached
/// directly here because there is no other way to reach it.
///
/// # THE OUTER HALF IS A COMPILE-TIME CLAIM HERE, WHICH IS STRICTLY STRONGER THAN PYTHON's
///
/// Python reads `inspect.signature(_stator_march).parameters` at run time. § 6's decided
/// replacement is the **narrowed config view**: the march entry takes [`StatorLeg`] and
/// [`MarchScope`], neither of which carries either field, spelled as an exhaustive destructuring
/// with **no `..`** — the only form that breaks the build when a field is ADDED.
///
/// The inner half is Python's own second half and is the REAL CALL, not a source scan: the port
/// can construct a [`FuelLimiters`] carrying `s_off` and hand it straight to the table's
/// `integrate_fuel`, which is exactly what `m.integrate_fuel(..., s_off=0.3)` does there.
#[test]
fn refuses_a_forced_release_edge() {
    // (1) THE OUTER, STRUCTURAL GUARD — Python's `inspect.signature(...).parameters`.
    let StatorLeg { accel: _, surge: _, tt4_max: _ } =
        StatorLeg { accel: None::<&AccelSchedule>, surge: None, tt4_max: Some(TT4_MAX) };
    let MarchScope { b0: _, lag: _, tau_gov: _, v0: _, ic_order: _ } = MarchScope::DEFAULT;

    // (2) THE INNER GUARD, REACHED — Python's `m.integrate_fuel(..., s_off=0.3)`.
    let m = shared_of(&inc_arm());
    let floor = surge();
    let lg = lag();
    panics_with(|| {
        let lim = FuelLimiters {
            freeze: None, tt4_max: Some(TT4_MAX), tau_gov: Some(TAU_GOV), accel: None,
            surge: match floor { Floor::Phi(s) => Some(s), _ => None },
            incidence: None, s_off: Some(0.3), tau_rel: None, lag: Some(lg),
        };
        (m.fuel.hooks.integrate_fuel)(&m.fuel, &flight(), &|_s: f64| 1.0, (1.0, 1.0), 0.1, DS,
                                      &lim);
    }, "FORCED release edges");
}

#[test]
fn refuses_an_instantaneous_valve() {
    let arm = LeverArm { bleed_lim: Some(BleedLimiter::from_margin(&lp(), B, SM)),
                         stator_inc: Some(inc_stator()), ..Default::default() };
    let m = shared_of(&arm);
    panics_with(|| { march(&m, Some(surge()), Some(lag()), Some(TT4_MAX), Some(TAU_GOV)); },
                "INSTANTANEOUS valve");
}

/// The composition law on the shared actuator is this rung's ONE modelling decision, so it is
/// declared and never inferred.
///
/// Python writes `m._share_law = "mean"` directly; the port's field is behind [`ShareScope`], which
/// is the guard the plant's own `finally` is — so the inadmissible value is installed through it
/// and the refusal is reached the same way.
#[test]
fn refuses_an_undeclared_composition_law() {
    let m = shared_of(&inc_arm());
    panics_with(|| {
        let _g = ShareScope::set(&m, "mean");
        march(&m, Some(surge()), Some(lag()), Some(TT4_MAX), Some(TAU_GOV));
    }, "composition law");
}

/// The floor is re-justified a FIFTH time on a FOURTH argument: the masked leg's eigenvalue is
/// exactly `−1/tau_f` and the other three share the remainder, so no root exceeds the rate sum.
/// Re-stated rather than inherited, because rung 65 published a retraction for a trusted stability
/// argument.
///
/// # THE NEEDLE IS NOT PYTHON's, AND THAT IS THE ONE DELIBERATE DIVERGENCE IN THIS FILE
///
/// The shipped gate matches `r"FOUR actuator states"`, and § 5.28 (v) measured that phrase in
/// rungs 72, 73 **and** 74's messages, whose conditions are identical character for character —
/// so the Python gate passes with rung 73's or rung 74's floor installed and discriminates
/// nothing. `rung-72` and `-1/tau_f` are unique to this rung's message. Rung 69's analogue does
/// not have the defect: its needle is `"rank TWO"`.
#[test]
fn the_rk4_floor_is_on_all_four_clocks() {
    let m = shared_of(&inc_arm());
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: Some(surge()),
                          tt4_max: Some(TT4_MAX) };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: 0.05 };
    let msg = panics_with(|| {
        m.stator_march_scoped(&flight(), &ramp, None, &leg,
                              &MarchScope { lag: Some(lag()), tau_gov: Some(TAU_GOV),
                                            ..MarchScope::DEFAULT });
    }, "rung-72");
    assert!(msg.contains("-1/tau_f"),
            "the needle must be one only THIS rung's message carries — `FOUR actuator states` is \
             in rungs 73's and 74's too: {msg:?}");
    // and it ADMITS the grid every reader here runs on
    (m.triple_hooks().rk4_floor_shared)(0.002, 4.0 / 0.05);
}

/// `_applied_clip` is the plant's law AND every reader's, so a reader cannot compose the two clips
/// differently from the march that produced its base point.
#[test]
fn the_composition_law_lives_in_one_place() {
    let m = shared_of(&valve_arm());
    assert_eq!(applied_clip(&m, 0.3, 0.7), 0.7);
    {
        let _g = ShareScope::set(&m, "sum");
        assert!((applied_clip(&m, 0.3, 0.7) - 1.0).abs() < 1e-12);
    }
    // restored on `Drop` — rung 62's reason, and Python's `finally`
    assert_eq!(m.fuel.inner.share_law.get(), "max");
}

/// A third regime label no prior rung needed: `dormant`, `tie` (the kink, where a central
/// difference straddles two branches) and the holder's name.
#[test]
fn authority_labels_the_switch_itself() {
    assert_eq!(authority(0.0, 0.0), Authority::Dormant);
    assert_eq!(authority(1e-3, 1e-3), Authority::Tie);
    assert_eq!(authority(2e-3, 1e-3), Authority::Fuel);
    assert_eq!(authority(1e-3, 2e-3), Authority::Gov);
}

// --- THE MARCH AUDIT: rung 79's gap seam, checked from the other end ------------------------
// `docs/rungs72-77-march-audit.md`. Added by a CONFIRMATION, not by this rung's anchor, and
// honest about that: nothing here was pre-registered.

/// `docs/rung79-gap-margin.md` proved rungs 78/79's marches never leave their initial state and
/// flagged that THIS RUNG SHARES THE RIG. It does not stand still.
///
/// **AND THE GATE IS A COUNTER-EXAMPLE, NOT A LIVENESS ASSERTION.** Same rig, same wall
/// (`phi_lim = 0.80`), same 341 steps as the arrested rows — the ONLY difference is the
/// coordinate, which is `clip` here and `demand` there (rung 74 § 2.2's arrest arm). That is what
/// localises the arrest to the CELL rather than to the rig.
///
/// The four loop counts are the SECOND vacuity mode (audit § 1): a plant that moves while the
/// section's own loop does nothing is just as vacuous as a frozen one.
#[test]
fn this_rungs_march_moves_and_all_four_loops_are_live() {
    let m = shared_of(&valve_arm());
    let traj = shared_march(&m, &flight(), LO, HI, TT4_MAX, SM, CLOCKS[0], R, SETTLE, DS, V_MAX,
                            false).3;
    assert!(traj.len() > 300, "{}", traj.len());
    let nu: Vec<f64> = traj.iter().map(|p| p.nu_lp).collect();
    let (nu_lo, nu_hi) = (nu.iter().copied().fold(f64::INFINITY, f64::min),
                          nu.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    assert!((nu_hi - nu_lo) / nu_lo > 1e-2, "({nu_lo}, {nu_hi})");
    let t4: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
    let (t4_lo, t4_hi) = (t4.iter().copied().fold(f64::INFINITY, f64::min),
                          t4.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    assert!(t4_hi - t4_lo > 200.0, "({t4_lo}, {t4_hi})");
    // ALL FOUR LOOPS ACT — governor, valve interior, stator riding, and rung 49's `phi` leg, the
    // last by its observable signature: the droop is held far above the free one (0.7430 at the
    // 0.70 arm, audit § 3) while still crossing the wall the clip coordinate tracks.
    let b_max = m.fuel.inner.lever.lim.expect("the receiver arms a valve").b_max;
    let required = |p: &FuelPoint| match p.extra {
        PointExtra::Shared { required, .. } => required,
        _ => panic!("rung-72's audit reads `required` with a bare index"),
    };
    assert!(traj.iter().filter(|p| required(p) > 0.0).count() > 300);
    assert!(traj.iter().filter(|p| matches!(p.extra,
                PointExtra::Shared { b_cmd, .. } if 0.0 < b_cmd && b_cmd < b_max)).count() > 50);
    assert!(traj.iter().filter(|p| matches!(p.extra,
                PointExtra::Shared { v_regime: Some(Regime::Riding), .. })).count() > 50);
    let min_phi = traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min);
    assert!(min_phi > 0.78, "{min_phi}");
}
