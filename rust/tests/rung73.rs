//! RUNG 73 — **THE APPLIED REFERENCE**: rung 72 § 11's own sharpest seam. Its masked fuel leg
//! stops reading the SCHEDULED fuel and reads the fuel actually burnt, which is the one change
//! rung 72 predicted would give `F_r != 0`, couple the two fuel rows and destroy the block form.
//!
//! **THE HEADLINE: THE COUPLING IS REAL AND IT LANDS IN THE WRONG COLUMN.** `F_r = -1` exactly, so
//! the premise HOLDS — but triangularity was never a property of the masked leg's ROW; it is a
//! property of its COLUMN, and `F_r` sits in the AUTHORITATIVE one. The masked column is zero
//! under EVERY reference, because `max()` is flat in the masked state. **Triangularity is a
//! property of MIN-SELECT alone.** What the reference buys is the POLE: rung 72's free pole at
//! `-1/tau_masked` moves to EXACTLY the origin, every per-cell zero count gains one, and `det J`
//! dies in rung 71's cell — the only full-rank plant in the family.
//!
//! Ported from `tests/test_rung73.py` — **27 collected tests, of which 13 carry `slow` there.**
//! Both numbers are MEASURED (`pytest --collect-only -q -n0`, once plain and once with `-m slow`),
//! never read off a sentence, and the count here is reconciled against `cargo test -- --list` and
//! not against a `grep`: slice AC step 4 and slice AD step 4 each had `grep` over-count by one
//! because the extra sat inside a doc comment, and **this header quotes the number 27, so this
//! file is the third instance waiting to happen.** The `slow` marker is dropped per slice M's
//! rule; `#[ignore]` is re-introduced only against a MEASURED Rust cost, never inherited.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, 0 ADDED, 0 COLLAPSED, 4 SPLIT BY PARAMETER
//!
//! Python's four `@pytest.mark.parametrize("inc", [False, True])` sites each collect as TWO tests,
//! so they land here as two `#[test]` functions apiece rather than as one loop. That keeps the
//! collected counts equal on both sides, which is the only form in which "1:1" is checkable.
//!
//! | # | `tests/test_rung73.py` | here |
//! |---|---|---|
//! | 1 | `reduces_to_rung72_under_the_scheduled_reference` | [`reduces_to_rung72_under_the_scheduled_reference`] |
//! | 2 | `the_scheduled_reduce_is_not_vacuous` | [`the_scheduled_reduce_is_not_vacuous`] |
//! | 3 | `reduces_to_rung71_no_fuel_leg` | [`reduces_to_rung71_no_fuel_leg`] |
//! | 4 | `reduces_to_rung70_no_fuel_leg` | [`reduces_to_rung70_no_fuel_leg`] |
//! | 5 | `reduces_to_rung69_no_governor` | [`reduces_to_rung69_no_governor`] |
//! | 6 | `reduces_to_rung68_no_governor` | [`reduces_to_rung68_no_governor`] |
//! | 7 | `reduces_to_rung67_no_stator_no_fuel_leg` | [`reduces_to_rung67_no_stator_no_fuel_leg`] |
//! | 8 | `at_lever_and_the_rig_both_carry_the_reference` | [`at_lever_and_the_rig_both_carry_the_reference`] |
//! | 9 | `the_reference_dispatch_is_live` | [`the_reference_dispatch_is_live`] — **counterfeit TABLE for a Python subclass** |
//! | 10 | `the_handover_is_late_and_the_masked_leg_winds_down` | [`the_handover_is_late_and_the_masked_leg_winds_down`] |
//! | 11a/b | `the_masked_leg_couples_and_still_reaches_nothing[False/True]` | `..._on_the_phi_arm` / `..._on_the_incidence_arm` |
//! | 12a/b | `only_two_entries_of_J_move_and_both_by_one_over_tau[False/True]` | `..._on_the_phi_arm` / `..._on_the_incidence_arm` |
//! | 13 | `every_cell_is_its_rung72_parent_plus_one_zero` | [`every_cell_is_its_rung72_parent_plus_one_zero`] |
//! | 14 | `the_parent_polynomial_survives_with_the_pole_at_the_origin` | [`the_parent_polynomial_survives_with_the_pole_at_the_origin`] |
//! | 15 | `the_determinant_dies_in_the_one_cell_where_it_lived` | [`the_determinant_dies_in_the_one_cell_where_it_lived`] |
//! | 16a/b | `the_two_readings_move_disjoint_halves_of_the_matrix[False/True]` | `..._on_the_phi_arm` / `..._on_the_incidence_arm` |
//! | 17a/b | `rung72_under_reported_its_own_peak_debit[False/True]` | `..._on_the_phi_arm` / `..._on_the_incidence_arm` |
//! | 18 | `refuses_the_applied_reference_on_top_of_the_sum_law` | [`refuses_the_applied_reference_on_top_of_the_sum_law`] |
//! | 19 | `refuses_an_undeclared_reference` | [`refuses_an_undeclared_reference`] |
//! | 20 | `the_rk4_floor_is_re_justified_and_still_armed` | [`the_rk4_floor_is_re_justified_and_still_armed`] — **needle SPLIT, see below** |
//! | 21 | `the_inherited_refusals_are_still_armed` | [`the_inherited_refusals_are_still_armed`] — **introspection substituted** |
//! | 22 | `the_reference_lives_in_one_place` | [`the_reference_lives_in_one_place`] — **both bars RE-MEASURED** |
//! | 23 | `this_rungs_march_MOVES_and_all_four_loops_are_live` | [`this_rungs_march_moves_and_all_four_loops_are_live`] |
//!
//! # EVERY READER ARGUMENT BELOW THE FIFTH IS THE READER's OWN DEFAULT, AND THEY ARE NOT EQUAL
//!
//! § 5.27.6 (i) burned a slice on exactly this: a shipped row measured at `every = 40` while the
//! fixture passed `every = 10`, and five numbers were wrong. Every rung-73 reader call in the
//! Python file passes `FLIGHT, LO, HI, TT4_MAX, SM` plus at most `taus=`/`clocks=`/`inc=`, so
//! `r`, `s_settle`, `ds`, `v_max` and `every` come from **`turbojet/engine.py`'s own `def` line** —
//! never from this module's `DS`, which agrees with only two of the five.
//!
//! | reader | `ds` | `every` | grid |
//! |---|---|---|---|
//! | `handover_law` | **0.005** | — | [`CLOCKS`], three arms |
//! | `applied_gains` | **0.002** | **2** | `taus` = matched |
//! | `applied_cells` | **0.002** | **2** | [`CLOCKS`], three arms |
//! | `ref_discriminator` | **0.002** | **4** | `taus` = matched |
//! | `applied_bill` | **0.005** | — | `taus` = matched |
//!
//! **AND THERE IS EXACTLY ONE CALL IN THE PYTHON FILE THAT OVERRIDES A DEFAULT**, which is why the
//! table above is not the whole story: the broken-instrument probe (#9) passes `ds = 0.01` and
//! `every = 8`, neither of which is `applied_gains`'s own. Transcribed from the call site, not
//! from this table.
//!
//! [`CLOCKS`] is bit-identical to `handover_law`'s and `applied_cells`'s own defaults, so passing
//! it substitutes nothing — recorded rather than assumed, because the same sentence read the other
//! way is § 5.27.6 (i)'s defect exactly.
//!
//! # WHAT OVERLAPS `slice_ae_cells.rs`, AND WHY IT IS PORTED ANYWAY
//!
//! Step 1's cell file already carries close relatives of gates **1, 2, 8, 18, 19** and of the
//! floor's message check inside **20**. They are ported here anyway, on `rung72.rs`'s precedent:
//! this file's contract is a **1:1 map of the shipped suite**, and a hole in it is invisible to
//! every count on both sides. The two files also gate different things — step 1's drive the CELL
//! directly (`(R73_TRIPLE.rk4_floor_shared)(…)`, a hand-set field, `bare_march`), these drive it
//! through `_stator_march` with the suite's own arming, which is the entry the suite actually
//! uses.
//!
//! # THE ONE NEEDLE THAT CANNOT PORT AS WRITTEN, AND THE TWO THAT PORT STRICTLY STRONGER
//!
//! **`match=r"rung-73.*origin"` is a REGEX and this file's `panics_with` is a SUBSTRING match.**
//! There is no `.*` here, so the needle is split into `"rung-73"` and `"origin"` **plus an
//! explicit ORDER assertion** (`find("rung-73") < find("origin")`), which is what the `.*` was
//! carrying. Disclosed rather than quietly weakened.
//!
//! **The two INHERITED refusals name the rung that OWNS them, which Python's cannot.** § 5.29
//! (vii) measured `"no set point"` in rungs 70/71/72/74's messages and `"FORCED release"` in
//! **nine** classes back to rung 43 — so both shipped gates filed under rung 73 assert a refusal
//! rung 73 does not own, and neither needle discriminates. The Rust messages carry a `rung-NN:`
//! prefix, and rung 73's fuel table delegates to rung 72's, so both gates here additionally assert
//! `"rung-72"` — P6 satisfied by MEASUREMENT of the shipped strings and not by hope.

use std::panic::catch_unwind;
use std::ptr::fn_addr_eq;

use turbojet::applied_reference::{
    applied_bill, applied_cells, applied_gains, build_applied_reference_cascade, handover_law,
    ref_discriminator, REF_LAW_APPLIED, R73, R73_FUEL, R73_STATOR, R73_TRIPLE, R73_TWO,
};
use turbojet::bleed_transient::{LeverArm, LeverArming, LeverHooks};
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
use turbojet::reference_split::{build_reference_split_cascade, RefScope, StatorIncidenceLimiter};
use turbojet::shared_actuator::{
    applied_clip, build_shared_actuator_cascade, shared_march, SharedRigArm,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{build_three_loop_cascade, StatorLimiter, TripleHooks};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

// ============================================================================== the grid
//
// `tests/test_rung73.py`'s module constants, verbatim.

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
/// constructor asserts `m_lim == T_c - 1/phi_lim`, so a rounded constant breaks a wall identity.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
/// **RUNG 67's imposed redline, VERBATIM through rungs 70/71/72.**
const TT4_MAX: f64 = 1200.0;

/// Python's module-level `CLOCKS` — **THREE arms where rung 72 had two.**
///
/// The applied reference DELAYS the hand-over, so rung 72's coverage does not transfer: at matched
/// clocks the incidence/governor cell is EMPTY (0 points, against rung 72's 1), and rung 72's
/// wide-cell arm reaches it with 4. The third arm is rung 72 § 2.3's own device pushed one notch —
/// governor twice as fast, valve 1.6x slower. All four entries are swept march coordinates; no
/// physical constant enters.
///
/// **AND IT IS BIT-IDENTICAL TO `handover_law`'s AND `applied_cells`'s OWN DEFAULT.**
const CLOCKS: [(f64, f64, f64, f64); 3] = [
    (0.05, 0.05, 0.05, 0.05), (0.20, 0.01, 0.50, 0.05), (0.20, 0.005, 0.80, 0.05),
];

// ----------------------------------------------------------- THE READERS' OWN DEFAULTS
//
// See the module header's table. Read off `turbojet/engine.py`'s `def` lines, not off the
// constants above — `DS` agrees with two of the five and NOT with the other three.

/// Shared by all five readers: `r = 0.5`, `s_settle = 1.2`, `v_max = 0.20`. Equal to this module's
/// `R`, `SETTLE` and `V_MAX`, which is why they are spelled from those and not re-typed.
const RDR_R: f64 = R;
const RDR_SETTLE: f64 = SETTLE;
const RDR_V_MAX: f64 = V_MAX;

/// `handover_law(..., ds=0.005)`.
const HL_DS: f64 = 0.005;
/// `applied_gains(..., taus=(0.05,)*4, ds=0.002, every=2)`.
const AG_DS: f64 = 0.002;
const AG_EVERY: usize = 2;
/// `applied_cells(..., ds=0.002, every=2)`.
const AC_DS: f64 = 0.002;
const AC_EVERY: usize = 2;
/// `ref_discriminator(..., taus=(0.05,)*4, ds=0.002, every=4)`.
const RD_DS: f64 = 0.002;
const RD_EVERY: usize = 4;
/// `applied_bill(..., taus=(0.05,)*4, ds=0.005)`.
const AB_DS: f64 = 0.005;

/// **THE ONLY OVERRIDDEN DEFAULTS IN THE WHOLE PYTHON FILE** — the broken-instrument probe's, at
/// `tests/test_rung73.py:243`. Neither is `applied_gains`'s own (`0.002` / `2`), and they are
/// transcribed from that call site rather than from the table above.
const BROKEN_DS: f64 = 0.01;
const BROKEN_EVERY: usize = 8;

/// § 1.2's law: `zeros = n_live - m_live + n_masked` — rung 72's own counts EACH PLUS ONE.
/// Python's `PREDICTED`, **declared here rather than read off the port's own
/// `AppliedCells::predicted`**: a gate that compared the machine against the constant that built
/// it would be this phase's own recurring defect.
const PREDICTED: [((bool, Authority), usize); 4] = [
    ((false, Authority::Fuel), 3), ((false, Authority::Gov), 2),
    ((true, Authority::Fuel), 2), ((true, Authority::Gov), 1),
];

/// Python's `RUNG72` — the parent's counts, so `PREDICTED == RUNG72 + 1` is an assertion and not a
/// restatement.
const RUNG72: [((bool, Authority), usize); 4] = [
    ((false, Authority::Fuel), 2), ((false, Authority::Gov), 1),
    ((true, Authority::Fuel), 1), ((true, Authority::Gov), 0),
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

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// Python's `_applied(design, **kw)`.
fn applied_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_applied_reference_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn shared72_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_shared_actuator_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn full71_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_full_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn cross70_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_cross_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn ref69_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_reference_split_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn triple68_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_three_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn cross67_of(arm: &LeverArm) -> ScheduledStatorCore {
    full_of(build_cross_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn valve() -> BleedLimiter { BleedLimiter::from_margin_tau(&lp(), B, SM, Some(TAU)) }
fn valve_tau(tau: f64) -> BleedLimiter { BleedLimiter::from_margin_tau(&lp(), B, SM, Some(tau)) }
fn phi_stator() -> StatorLimiter { StatorLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S)) }
fn phi_stator_tau(tau: f64) -> StatorLimiter {
    StatorLimiter::from_margin(&lp(), V_MAX, SM, Some(tau))
}
fn inc_stator() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S))
}
fn surge() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM)) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }
fn lag_of(att: f64, rel: f64) -> AsymmetricLag { AsymmetricLag::new(att, rel) }

/// The valve-only arm — Python's `_applied(design, bleed_lim=_valve())`, the receiver every
/// § 0-§ 4 reader is called on. The readers build their OWN rig, so the receiver's stator arming
/// does not reach the march.
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

/// All four loops — the arming gates 1, 2, 18 and 19 use.
fn four_loop_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    march(m, Some(surge()), Some(lag()), Some(TT4_MAX), Some(TAU_GOV))
}

/// Python's `p["b"]` — a `KeyError` off a trajectory that records no valve, which is a panic here.
fn b_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { b, .. } | PointExtra::Triple { b, .. }
        | PointExtra::CrossCascade { b, .. } | PointExtra::Cascade { b, .. }
        | PointExtra::Valve { b, .. } => b,
        _ => panic!("rung-73's `_keys` reads `b` with a bare index"),
    }
}

/// Python's `p["v"]` — likewise, and it is the key the rung-67 arm DROPS.
fn v_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { v, .. } | PointExtra::Triple { v, .. } => v,
        _ => panic!("rung-73's `_keys` reads `v` with a bare index"),
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
/// rung-67 arm arms no stator and a bare `p["v"]` would raise there. Python's narrowing, ported
/// rather than silently widened.
fn keys8(traj: &[FuelPoint]) -> Vec<[u64; 8]> {
    traj.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits(), b_of(p).to_bits()])
        .collect()
}

/// `pytest.raises(AssertionError, match=...)` — the message is RETURNED so a caller can put a
/// second, stronger needle on it.
///
/// `AssertUnwindSafe` because every machine in this family carries `Cell` fields — the dynamically
/// scoped state — and those are exactly what `UnwindSafe` flags. The state is rebuilt per test
/// here, so a poisoned one cannot outlive the assertion.
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

/// The declared constant for one cell — `PREDICTED[k]` / `RUNG72[k]`, Python's dict index.
fn count_of(t: &[((bool, Authority), usize)], k: (bool, Authority)) -> usize {
    t.iter().find(|(kk, _)| *kk == k).expect("all four cells are declared").1
}

// =============================================================================================
// THE REDUCE SPINE — SIX arms. Five inherited by DISPATCH, one by the declared law.
//
// NOT MARKED `slow` in Python, on rung 72's own reasoning (and rungs 69/70/71's): each runs two
// 341-point marches and is not free, but the reduce spine is the project's spine and
// `conftest.py` is explicit that `-m "not slow"` has no backstop. Nothing is marked here either,
// per slice M's rule.
// =============================================================================================

/// **THE SIXTH ARM, AND THIS RUNG's OWN**: `_ref_law = "sched"` makes `_reference` the identity, so
/// the plant is rung 72 BIT FOR BIT. The hook is the only thing this rung adds to the march, so
/// this is the arm that says so.
///
/// Python spells the scope as `m._with_ref("sched", _march, m, …)`; the port's guard is
/// [`RefScope`], which dispatches through the re-aimed `with_ref` cell and restores on `Drop` —
/// the same `try/finally`, reached the same way.
#[test]
fn reduces_to_rung72_under_the_scheduled_reference() {
    let m = applied_of(&inc_arm());
    let a = {
        let _r = RefScope::set(&m.fuel.inner, Some("sched"));
        four_loop_march(&m)
    };
    let b = four_loop_march(&shared72_of(&inc_arm()));
    assert_eq!(keys(&a), keys(&b));
}

/// **AND THE ARM ABOVE MUST BE A TEST, NOT A TAUTOLOGY.** If `_reference` ignored `_ref_law` the
/// reduce would still pass — it would compare rung 73 with rung 73 — so the same two marches under
/// the APPLIED reference must DIFFER. Rung 72's `charpoly_selftest` discipline: a gate that has
/// never failed on the bug it was written for is ceremony.
#[test]
fn the_scheduled_reduce_is_not_vacuous() {
    let a = four_loop_march(&applied_of(&inc_arm()));
    let b = four_loop_march(&shared72_of(&inc_arm()));
    assert_ne!(keys(&a), keys(&b));
    // and it differs in the PLANT, not only in the masked state
    let worst = a.iter().zip(b.iter()).map(|(x, y)| (x.tt4 - y.tt4).abs()).fold(0.0f64, f64::max);
    assert!(worst > 1.0, "{worst}");
}

#[test]
fn reduces_to_rung71_no_fuel_leg() {
    let a = gov_march(&applied_of(&inc_arm()));
    let b = gov_march(&full71_of(&inc_arm()));
    assert_eq!(keys(&a), keys(&b));
}

#[test]
fn reduces_to_rung70_no_fuel_leg() {
    let a = gov_march(&applied_of(&phi_arm()));
    let b = gov_march(&cross70_of(&phi_arm()));
    assert_eq!(keys(&a), keys(&b));
}

/// **AND THIS ARM IS AN IDENTITY, NOT ONLY A DISPATCH.** With ONE fuel-side leg armed the sole leg
/// always holds authority, so `max(gf, gr) == g_own` everywhere and the applied reference IS the
/// scheduled one — the reduce would hold even with the dispatch removed. Rung 71's *inherited
/// identity* form, one rung on.
#[test]
fn reduces_to_rung69_no_governor() {
    let a = fuel_march(&applied_of(&inc_arm()));
    let b = fuel_march(&ref69_of(&inc_arm()));
    assert_eq!(keys(&a), keys(&b));
}

#[test]
fn reduces_to_rung68_no_governor() {
    let a = fuel_march(&applied_of(&phi_arm()));
    let b = fuel_march(&triple68_of(&phi_arm()));
    assert_eq!(keys(&a), keys(&b));
}

#[test]
fn reduces_to_rung67_no_stator_no_fuel_leg() {
    let a = gov_march(&applied_of(&valve_arm()));
    let b = gov_march(&cross67_of(&valve_arm()));
    assert_eq!(keys8(&a), keys8(&b));
}

/// **THE ELEVENTH INSTANCE of the trap rungs 61-72 each hit, with a SECOND HEAD.** Handing back
/// the parent's class reports rung 73 while measuring rung 72; handing back the right class but
/// dropping `_ref_law` does the same thing one level down, in every ledger cell.
///
/// Python asserts `type(lv) is AppliedReferenceTransient`; the port's equivalent of a class
/// identity is which TABLE the machine carries, so the class half is a function-pointer
/// comparison against this rung's own `reference` cell.
#[test]
fn at_lever_and_the_rig_both_carry_the_reference() {
    let m = applied_of(&valve_arm());
    let lv = (R73.at_lever)(&m, &inc_arm());
    assert!(fn_addr_eq(lv.triple_hooks().reference, R73_TRIPLE.reference),
            "the sibling is a RUNG-73 machine");
    assert_eq!(lv.fuel.inner.ref_law.get(), REF_LAW_APPLIED);

    let arm = SharedRigArm { sm: SM, tt4_max: TT4_MAX, inc: true, tau: TAU, tau_s: TAU_S,
                             v_max: V_MAX, ..Default::default() };
    let (rig, _, _) = (R73_TRIPLE.shared_rig)(&m, &arm);
    assert!(fn_addr_eq(rig.triple_hooks().reference, R73_TRIPLE.reference));
    assert_eq!(rig.fuel.inner.ref_law.get(), REF_LAW_APPLIED);

    // and the restore-in-`finally` must reach the rig too (rung 62's reason, seventh reload)
    let rig2_law = {
        let _r = RefScope::set(&m.fuel.inner, Some("sched"));
        let (rig2, _, _) = (R73_TRIPLE.shared_rig)(&m, &arm);
        rig2.fuel.inner.ref_law.get()
    };
    assert_eq!(rig2_law, "sched");
    assert_eq!(m.fuel.inner.ref_law.get(), REF_LAW_APPLIED);
}

// =============================================================================================
// THE INSTRUMENT, GATED AGAINST ITSELF — the bug here produced a PERFECT confirmation
// =============================================================================================

/// **PYTHON's `Broken` SUBCLASS, AS A COUNTERFEIT TABLE.**
///
/// `_reference`'s first version applied reading B unconditionally, so `_with_ref('sched', .)` was
/// a NO-OP and every A-vs-B reader differenced the plant against ITSELF. It did not fail: it
/// returned `worst_delta_rest = 0.0` and `mask_leak = 0.0` — a perfect confirmation of this rung's
/// headline from an instrument that had measured nothing. **That is the fifth instance of this
/// family's shipped-instrument-agrees-with-itself pattern** (rung 67 gate 9, rung 71 § 1.4, rung
/// 72 § 4 and § 8's `_charpoly4`). So the bug is REBUILT and fed to the gate.
///
/// The clip is RE-SPELLED here rather than reached, because `applied_clip_core` is `pub(crate)` —
/// and the re-spelling is PINNED against the shipped public `applied_clip` inside the gate, so a
/// counterfeit that drifted from the plant's own law would be caught rather than believed.
fn broken_reference(
    t: &TwoSpoolTransientCore, req: f64, g_own: f64, gf: f64, gr: f64,
) -> f64 {
    // THE BUG, VERBATIM: no `ref_law` dispatch. Python's
    //     clip = self._applied_clip(gf, gr); return req if clip == g_own else g_own + req - clip
    let clip = if t.share_law.get() == "max" { gf.max(gr) } else { gf + gr };
    if clip == g_own { req } else { (g_own + req) - clip }
}

/// Python's `Broken.at_lever`, whose whole content is *re-bless the sibling*: `at_lever` names its
/// class explicitly (the rung-61..72 trap's own fix), so a probe that did not re-bless would test
/// the SHIPPED class and pass.
fn broken_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let m = broken_from(core, arm);
    m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
    m
}

static BROKEN_TRIPLE: TripleHooks = TripleHooks { reference: broken_reference, ..R73_TRIPLE };
static BROKEN_LEVER: LeverHooks = LeverHooks { at_lever: broken_at_lever, ..R73 };

/// A rung-73 machine carrying the two counterfeit tables.
///
/// **NOT through [`build_applied_reference_cascade`]**, which hardcodes `&R73` and `&R73_TRIPLE` —
/// the point is to install a table it would never install. `build_split_family_cascade`'s guard C
/// is not re-spelled: this arming (no `stator_inc`, an LP spool) is one it admits, and
/// `slice_ae_cells.rs` is where the guards themselves are gated.
fn broken_from(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let built = ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        arm.stator, &R73_TWO, &R73_STATOR, &R73_FUEL, &BROKEN_LEVER,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        &BROKEN_TRIPLE, arm.stator_lim, arm.stator_inc);
    let c = full_of(built);
    // The CLASS ATTRIBUTE `build_applied_reference_cascade` applies — `Broken` inherits it.
    c.fuel.inner.ref_law.set(REF_LAW_APPLIED);
    c
}

/// The live gate is `moved_scaled == +-1`
/// ([`only_two_entries_of_j_move_and_both_by_one_over_tau_on_the_phi_arm`]), and the broken version
/// must FAIL it while still passing `worst_delta_rest == 0.0`.
#[test]
fn the_reference_dispatch_is_live() {
    let m = broken_from(&applied_of(&valve_arm()), &valve_arm());

    // THE INSTRUMENT PROVES IT CAN SEE, THREE WAYS, BEFORE IT IS READ.
    //
    // (a) the counterfeit SURVIVES the rig rebuild. Every reader on this ladder rebuilds its
    //     machine through `at_lever` (AC step 7's laundering finding), so without the re-blessed
    //     lever table the reader below would run the SHIPPED cell and the gate would be vacuous.
    let lv = (BROKEN_LEVER.at_lever)(&m, &inc_arm());
    assert!(fn_addr_eq(lv.triple_hooks().reference, BROKEN_TRIPLE.reference),
            "the sibling carries the COUNTERFEIT, which is Python's `m.__class__ = Broken`");
    assert!(!fn_addr_eq(lv.triple_hooks().reference, R73_TRIPLE.reference));
    // (b) the counterfeit's re-spelled clip IS the plant's own law, on this machine.
    assert_eq!(applied_clip(&m, 0.3, 0.7), 0.7f64.max(0.3));
    // (c) and the counterfeit really does ignore the law: under `"sched"` it still applies B,
    //     where the shipped cell returns `req` bitwise.
    {
        let _r = RefScope::set(&m.fuel.inner, Some("sched"));
        let (req, g_own, gf, gr) = (3.5f64, 1.0f64, 1.0f64, 2.0f64);
        assert_ne!(broken_reference(&m.fuel.inner, req, g_own, gf, gr).to_bits(), req.to_bits());
        assert_eq!((R73_TRIPLE.reference)(&m.fuel.inner, req, g_own, gf, gr).to_bits(),
                   req.to_bits());
    }

    let g = applied_gains(&m, &flight(), LO, HI, TT4_MAX, SM, CLOCKS[0], false, RDR_R, RDR_SETTLE,
                          BROKEN_DS, RDR_V_MAX, BROKEN_EVERY)
        .expect("the broken probe's coarse grid marches");
    assert!(!g.rows.is_empty(), "the broken-instrument probe needs at least one interior point");
    // the broken reader still passes the weak gate ...
    assert_eq!(g.worst_delta_rest.expect("rows exist"), 0.0);
    // ... and FAILS the live one, which is why the live one is the gate
    assert!(g.moved_scaled.iter().all(|v| v.abs() < 1e-9), "{:?}", g.moved_scaled);
}

// =============================================================================================
// § 0 — THE HAND-OVER MOVES, AND THE MASKED LEG WINDS DOWN (anchor P9, § 0.2)
// =============================================================================================

/// **The applied reference DELAYS the hand-over on every arm and at every clock**, and the sign is
/// derivable: a masked governor referenced to the SCHEDULE races toward the clip the schedule
/// would need — credit for a cut the fuel leg already made — while referenced to the APPLIED fuel
/// it integrates the cut still OWED. The physically-correct governor is the SLOWER one.
///
/// AND THE WINDUP CHECK, which was the feasibility gate: a masked integrator with only a floor
/// under it is textbook min-select windup, and had it run away the hand-over would slam a wound-up
/// clip onto the actuator (rung 72 § 4's SUM law died that way, at 84 points of 341). It winds
/// DOWN instead — masked means `gr > gf ~ req_f`, so the integrand is negative.
#[test]
fn the_handover_is_late_and_the_masked_leg_winds_down() {
    let h = handover_law(&applied_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, &CLOCKS,
                         RDR_R, RDR_SETTLE, HL_DS, RDR_V_MAX);
    assert!(h.always_later, "{:?}",
            h.arms.iter().map(|a| (a.inc, a.taus, a.delay)).collect::<Vec<_>>());
    assert!(h.never_back && h.one_handover && h.full_march);
    assert!(h.worst_d_tt4 > 0.0);
    for a in &h.arms {
        let p = &a.applied;
        assert_eq!(p.n, a.sched.n, "{:?} {}", a.taus, p.n);
        // NO WINDUP: the masked leg ends AT ITS FLOOR, and never exceeds the live clip's scale
        assert_eq!(p.final_g_fuel, 0.0, "{} {:?} {}", a.inc, a.taus, p.final_g_fuel);
        assert!(p.max_masked.expect("the applied arm hands over")
                < a.sched.max_masked.expect("so does the scheduled one"),
                "{} {:?}", a.inc, a.taus);
        // the IC is unchanged: both legs open dormant (rung 72's P9, inherited)
        assert!(p.ic_iters == 1 && p.ic_res == 0.0);
    }
}

// =============================================================================================
// § 1 — THE COUPLING IS REAL AND LANDS IN THE WRONG COLUMN (anchor P7, § 0.4)
// =============================================================================================

/// RUNG 72 § 11's PREMISE HOLDS AND ITS CONCLUSION DOES NOT, in four numbers:
///
/// ```text
/// cross_masked ~ -1     the masked leg DOES read the authoritative one — `F_r != 0`
/// self_masked  ~ +1     and its own state: it is an INTEGRATOR
/// self_live    == 0.0   while the HOLDING leg's applied reference IS the scheduled one
/// mask_leak    == 0.0   and the masked leg STILL reaches the plant through nothing
/// ```
///
/// **THE TWO EXACT ZEROS ARE GATED AS EQUALITY AND THE TWO ONES ARE NOT**, which is not a double
/// standard: `self_live` is exact because the hook takes an explicit identity BRANCH, while
/// `self_masked` is a central difference of a SUM (`gf +- dg + raw - gr`) and float addition does
/// not distribute. An exact zero survives a difference quotient; an exact one does not.
fn the_masked_leg_couples_and_still_reaches_nothing(inc: bool) {
    let g = applied_gains(&applied_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, CLOCKS[0],
                          inc, RDR_R, RDR_SETTLE, AG_DS, RDR_V_MAX, AG_EVERY)
        .expect("§ 1's rig marches");
    assert!(!g.rows.is_empty() && g.skipped_switch == 0 && g.skipped_regime == 0);
    assert_eq!(g.self_live, vec![0.0]);
    assert_eq!(g.worst_mask_leak.expect("rows exist"), 0.0);
    assert!(g.self_masked.iter().all(|v| (v - 1.0).abs() < 1e-9), "{:?}", g.self_masked);
    assert!(g.cross_masked.iter().all(|v| (v + 1.0).abs() < 1e-9), "{:?}", g.cross_masked);
    // and the plant is not trivially decoupled — the LIVE gains are non-zero
    assert!(g.min_live_gain.expect("rows exist") > 1e-4);
}

#[test]
fn the_masked_leg_couples_and_still_reaches_nothing_on_the_phi_arm() {
    the_masked_leg_couples_and_still_reaches_nothing(false);
}

#[test]
fn the_masked_leg_couples_and_still_reaches_nothing_on_the_incidence_arm() {
    the_masked_leg_couples_and_still_reaches_nothing(true);
}

/// THE ENTRYWISE `J(73) - J(72)`, at the SAME base points (rung 71's device, rung 72 § 4's: one law
/// swapped, nothing else). **14 of the 16 entries are EXACTLY 0.0**, and the two that move — the
/// masked leg's own diagonal and its cross-gain onto the AUTHORITATIVE axis — are both exactly
/// `1/tau_masked`. That is the whole reach of the reference.
fn only_two_entries_of_j_move_and_both_by_one_over_tau(inc: bool) {
    let g = applied_gains(&applied_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, CLOCKS[0],
                          inc, RDR_R, RDR_SETTLE, AG_DS, RDR_V_MAX, AG_EVERY)
        .expect("§ 1's rig marches");
    let rest = g.worst_delta_rest.expect("rows exist");
    assert_eq!(rest, 0.0, "{rest}");
    assert!(!g.moved_scaled.is_empty(), "nothing moved — the reference reader is dead (see § 8)");
    assert!(g.moved_scaled.iter().all(|v| (v.abs() - 1.0).abs() < 1e-9), "{:?}", g.moved_scaled);
    // both signs are present: +1 on the diagonal, -1 on the cross-gain
    let lo = g.moved_scaled.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = g.moved_scaled.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(lo < 0.0 && 0.0 < hi, "({lo}, {hi})");
}

#[test]
fn only_two_entries_of_j_move_and_both_by_one_over_tau_on_the_phi_arm() {
    only_two_entries_of_j_move_and_both_by_one_over_tau(false);
}

#[test]
fn only_two_entries_of_j_move_and_both_by_one_over_tau_on_the_incidence_arm() {
    only_two_entries_of_j_move_and_both_by_one_over_tau(true);
}

// =============================================================================================
// § 2 — EVERY ZERO COUNT PLUS ONE, AND A DETERMINANT THAT DIES (anchor P1, P2, P3, P4)
// =============================================================================================

fn cells() -> turbojet::applied_reference::AppliedCells {
    applied_cells(&applied_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, &CLOCKS,
                  RDR_R, RDR_SETTLE, AC_DS, RDR_V_MAX, AC_EVERY).expect("§ 2's three arms march")
}

/// **The plant is STILL rung 68/69/70/71 plus a pole — and the pole is now at the ORIGIN.**
/// `zeros = n_live - m_live + n_masked`: one value per cell, all four cells, each exactly one
/// above rung 72's own count.
#[test]
fn every_cell_is_its_rung72_parent_plus_one_zero() {
    let c = cells();
    assert!(c.all_four_cells && c.law_holds, "{:?}", c.cells);
    for (k, d) in &c.cells {
        let pred = count_of(&PREDICTED, *k);
        assert_eq!(d.zeros, vec![pred], "{:?} {} {:?} {}", k, d.parent, d.zeros, pred);
        assert_eq!(pred, count_of(&RUNG72, *k) + 1);
        assert!(d.n >= 4 && d.n_parent == d.n, "{:?} {} {}", k, d.n, d.n_parent);
    }
    // the RK4 floor, MEASURED rather than trusted (rung 65's retraction)
    assert!(c.worst_lam < 1.0, "{}", c.worst_lam);

    // AND THE PORT's OWN DECLARED TABLES AGREE WITH PYTHON's — asserted SEPARATELY, because the
    // loop above deliberately reads this file's constants and not `c.predicted`: a gate that
    // scored the machine against the constant that built it would be this phase's own recurring
    // defect ([[rust-port-slice-ae-step1]]).
    for (k, n) in PREDICTED {
        assert_eq!(count_of(&c.predicted, k), n, "the port's own PREDICTED table, {k:?}");
    }
    for (k, n) in RUNG72 {
        assert_eq!(count_of(&c.rung72, k), n, "the port's own RUNG72 table, {k:?}");
    }
}

/// `p4(lam) = lam * p3(lam)`, `p3` rebuilt from the SHIPPED rung-68/69/70/71 readers. Coefficients,
/// not roots — and the argument is STRONGER here than in rung 72, because the added root is exactly
/// zero, so every cell has at least a DOUBLE zero root and a root match would resolve it only to
/// `sqrt(eps)`.
///
/// **`gap` AND `null` ARE ONE NUMBER, NOT TWO.** The masked column's only non-zero entry is its own
/// diagonal, and `a3` is minus the trace, so the `j = 1` term of `gap` reproduces `null` entry for
/// entry. Quoting both as agreement would be this family's sixth instrument-agrees-with-itself;
/// `gap_hi` (`j = 2, 3, 4`) is where the two INDEPENDENT readers actually meet, and it is gated
/// separately.
#[test]
fn the_parent_polynomial_survives_with_the_pole_at_the_origin() {
    let c = cells();
    assert!(c.worst_parent_gap < 1e-10, "{}", c.worst_parent_gap);
    assert!(c.worst_parent_gap_hi < 1e-10, "{}", c.worst_parent_gap_hi);
    // the two readers land on the SAME manifold base point — a mismatch there is ruled out
    assert_eq!(c.worst_v_gap, 0.0);
    // the zero EIGENVECTOR lies ON the masked axis: `A e_masked = 0`. THE GATED HALF of the pole
    // claim — the eigenVALUE is reported, never gated (rung 72 § 1.2's discipline).
    assert!(c.worst_null < 1e-10, "{}", c.worst_null);
}

/// Rung 72 measured `det J = +5.9e4` in rung 71's cell — the only live determinant in the whole
/// family — and `~ 0` in the other three. Under the applied reference it is dead in ALL FOUR.
/// **A reference is not a gain, not a clock and not a loop, and it changes the RANK.**
#[test]
fn the_determinant_dies_in_the_one_cell_where_it_lived() {
    let c = cells();
    let rung71_cell = cell(&c.cells, &(true, Authority::Gov));
    assert_eq!(rung71_cell.parent, "rung 71");
    // normalised by the rate^4 the determinant scales with, it is eleven orders below rung 72's
    assert!(c.worst_det < 1e-3, "{}", c.worst_det);
    assert_eq!(rung71_cell.zeros, vec![1]);
}

// =============================================================================================
// § 3 — THE ISOLATION INSTRUMENT: reading C moves the OTHER half (anchor P5)
// =============================================================================================

/// Reading C is the LITERAL reading of rung 72 § 11 (`req = mf_app - cap`, no increment): a
/// well-posed proportional law with 2x droop, refused as the plant only because a leg that cannot
/// reach its own floor measures a different object than rungs 46-72 did.
///
/// ```text
/// B: the pole MOVES to the origin, the LIVE diagonal is unmoved, `M3` IS the parent's
/// C: the pole STAYS at -1/tau_m, the LIVE diagonal moves by exactly -1, `M3` is not
/// ```
///
/// Two readings that agree on `F_r != 0` and disagree on everything it was supposed to imply. That
/// is what makes the headline a measurement rather than a choice of law.
fn the_two_readings_move_disjoint_halves_of_the_matrix(inc: bool) {
    let d = ref_discriminator(&applied_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM,
                              CLOCKS[0], inc, RDR_R, RDR_SETTLE, RD_DS, RDR_V_MAX, RD_EVERY)
        .expect("§ 3's rig marches");
    assert!(d.n > 0);
    // C and rung 72 keep the free pole at -1/tau_masked; B does not
    assert!(d.worst_pole_c.expect("rows exist") < 1e-9
            && d.worst_pole_72.expect("rows exist") < 1e-9);
    let best_b = d.best_pole_b.expect("rows exist");
    assert!(best_b > 1e-2, "{best_b}");
    // the LIVE leg's own diagonal: EXACTLY 0 under B (the identity branch), EXACTLY -1 under C
    assert_eq!(d.live_diag_b, vec![0.0]);
    assert_eq!(d.live_diag_c, vec![-1.0]);
    // AND THE COUNTS SEPARATE ALL THREE READINGS — differenced PER POINT, because this reader
    // spans both authority cells and their counts already differ by one under rung 72 alone, so a
    // pooled comparison would compare one cell against the other and say nothing.
    assert_eq!(d.dzeros_b, vec![1], "{:?}", d.dzeros_b);
    assert!(d.dzeros_c.iter().copied().max().expect("rows exist") <= 0, "{:?}", d.dzeros_c);
}

#[test]
fn the_two_readings_move_disjoint_halves_of_the_matrix_on_the_phi_arm() {
    the_two_readings_move_disjoint_halves_of_the_matrix(false);
}

#[test]
fn the_two_readings_move_disjoint_halves_of_the_matrix_on_the_incidence_arm() {
    the_two_readings_move_disjoint_halves_of_the_matrix(true);
}

// =============================================================================================
// § 4 — THE LEDGER: what the scheduled reference was quietly buying (anchor P6)
// =============================================================================================

/// Rung 72 § 5 reports the fuel leg's marginal peak `Tt4` debit as +0.29 K / +1.86 K and calls the
/// `phi` credit the finding. **Under the correct reference the debit is 110x and 39x larger** —
/// because the fuel leg's own authority window is EARLY, where the reference is the identity, while
/// the governor's is LATE, where it is not, and a masked governor given credit for a cut it did not
/// make takes the actuator too soon.
///
/// The ordering is the claim and every magnitude is disclaimed; the gate is a 10x floor.
fn rung72_under_reported_its_own_peak_debit(inc: bool) {
    let b = applied_bill(&applied_of(&valve_arm()), &flight(), LO, HI, TT4_MAX, SM, CLOCKS[0],
                         inc, RDR_R, RDR_SETTLE, AB_DS, RDR_V_MAX);
    assert!(b.debit_sched > 0.0 && b.debit_applied > 0.0);
    let ratio = b.debit_ratio.expect("both debits are positive, so the ratio is defined");
    assert!(ratio > 10.0, "({}, {})", b.debit_sched, b.debit_applied);
    // the hand-over is later in the ledger's own full cell, too
    assert!(b.handover_applied.expect("the full cell hands over")
            > b.handover_sched.expect("so does the scheduled one"));
    // and `min phi` is UNMOVED, so the debit is not bought by moving the other currency
    assert_eq!(b.phi_full_applied, b.phi_full_sched);
}

#[test]
fn rung72_under_reported_its_own_peak_debit_on_the_phi_arm() {
    rung72_under_reported_its_own_peak_debit(false);
}

#[test]
fn rung72_under_reported_its_own_peak_debit_on_the_incidence_arm() {
    rung72_under_reported_its_own_peak_debit(true);
}

// =============================================================================================
// THE REFUSALS (anchor P10)
// =============================================================================================

/// TWO DECLARED LAWS AT ONCE. Under `sum` the hook never takes its identity branch, BOTH fuel rows
/// gain a cross term and the block form goes — a fourth plant, whose result could be attributed to
/// neither law. Rung 63's lesson in its plainest form.
///
/// Python writes `m._share_law = "sum"` directly and never restores it; the field is a plain `Cell`
/// here, so the direct set is the faithful spelling and the machine is discarded with the test.
#[test]
fn refuses_the_applied_reference_on_top_of_the_sum_law() {
    let m = applied_of(&phi_arm());
    m.fuel.inner.share_law.set("sum");
    panics_with(|| { four_loop_march(&m); }, "TWO declared");
}

#[test]
fn refuses_an_undeclared_reference() {
    let m = applied_of(&phi_arm());
    m.fuel.inner.ref_law.set("whatever");
    panics_with(|| { four_loop_march(&m); }, "DECLARED");
}

/// THE SIXTH JUSTIFICATION, and the previous five do not carry: the masked leg's eigenvalue is
/// exactly ZERO, which is neutrally stable, so *the dominant root is below the rate sum* is no
/// longer the sentence. The constant is unchanged and the message says why.
///
/// # THE NEEDLE IS A REGEX IN PYTHON AND A SUBSTRING HERE
///
/// `match=r"rung-73.*origin"` cannot port as one needle: [`panics_with`] takes a substring. It is
/// split into the two literals **plus the ORDER assertion the `.*` was carrying**, which is what
/// keeps the port from being weaker than its source. The negative control — rung 72's own argument
/// must NOT appear — is this file's own addition, and it is what separates the two rungs' messages
/// where § 5.29 (vii) measured the shipped `"FOUR actuator states"` reaching nine classes.
#[test]
fn the_rk4_floor_is_re_justified_and_still_armed() {
    let arm = LeverArm { bleed_lim: Some(valve_tau(0.005)),
                         stator_lim: Some(phi_stator_tau(0.005)), ..Default::default() };
    let m = applied_of(&arm);
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge: Some(surge()),
                          tt4_max: Some(TT4_MAX) };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: 0.02 };
    let msg = panics_with(|| {
        m.stator_march_scoped(&flight(), &ramp, None, &leg,
                              &MarchScope { lag: Some(lag_of(0.002, 0.006)), tau_gov: Some(0.002),
                                            ..MarchScope::DEFAULT });
    }, "rung-73");
    assert!(msg.contains("origin"), "the second half of Python's regex: {msg:?}");
    assert!(msg.find("rung-73") < msg.find("origin"),
            "and `rung-73.*origin` is ORDERED, which a pair of `contains` is not: {msg:?}");
    assert!(!msg.contains("-1/tau_f"),
            "rung 72's argument does NOT carry here — the masked eigenvalue is zero: {msg:?}");
}

/// Rung 72's own five stay live through this rung's `integrate_fuel` override.
///
/// # THE SIGNATURE HALF IS A COMPILE-TIME CLAIM HERE, WHICH IS STRICTLY STRONGER THAN PYTHON's
///
/// Python reads `inspect.signature(AppliedReferenceTransient._stator_march).parameters` at run
/// time and asserts `s_off`/`tau_rel` are absent. § 6's decided replacement is the **narrowed
/// config view**: the march entry takes [`StatorLeg`] and [`MarchScope`], neither of which carries
/// either field, spelled as an exhaustive destructuring with **no `..`** — the only form that
/// breaks the build when a field is ADDED. A runtime assertion would be ceremony.
///
/// # AND BOTH NEEDLES NAME THE RUNG THAT OWNS THE REFUSAL, WHICH PYTHON's CANNOT
///
/// § 5.29 (vii) measured `"no set point"` in rungs 70/71/72/74's messages and `"FORCED release"`
/// in NINE classes back to rung 43 — **neither is rung 73's, and neither discriminates.** The Rust
/// messages carry a `rung-NN:` prefix and this rung's fuel table delegates to rung 72's, so both
/// halves additionally assert `"rung-72"`. That is P6, met by measuring the shipped strings.
#[test]
fn the_inherited_refusals_are_still_armed() {
    let m = applied_of(&phi_arm());
    let a = panics_with(|| { march(&m, Some(surge()), Some(lag()), None, Some(TAU_GOV)); },
                        "no set point");
    assert!(a.contains("rung-72"), "the refusal names the rung that OWNS it: {a:?}");

    // (1) THE OUTER, STRUCTURAL GUARD — Python's `inspect.signature(...).parameters`.
    let StatorLeg { accel: _, surge: _, tt4_max: _ } =
        StatorLeg { accel: None::<&AccelSchedule>, surge: None, tt4_max: Some(TT4_MAX) };
    let MarchScope { b0: _, lag: _, tau_gov: _, v0: _, ic_order: _ } = MarchScope::DEFAULT;

    // (2) THE INNER GUARD, REACHED — Python's `m.integrate_fuel(..., s_off=0.3)`, and it goes
    //     through THIS rung's table, so rung 73's own two refusals run first and pass.
    let floor = surge();
    let lg = lag();
    let b = panics_with(|| {
        let lim = FuelLimiters {
            freeze: None, tt4_max: Some(TT4_MAX), tau_gov: Some(TAU_GOV), accel: None,
            surge: match floor { Floor::Phi(s) => Some(s), _ => None },
            incidence: None, s_off: Some(0.3), tau_rel: None, lag: Some(lg),
        };
        (m.fuel.hooks.integrate_fuel)(&m.fuel, &flight(), &|_s: f64| 1.0, (1.0, 1.0), 0.1, DS,
                                      &lim);
    }, "FORCED release");
    assert!(b.contains("rung-72"), "likewise: {b:?}");
    assert!(fn_addr_eq(m.fuel.hooks.integrate_fuel, R73_FUEL.integrate_fuel),
            "and the call went through RUNG 73's table, so its own refusals were passed first");
}

/// `_reference` is the ONE seat of the law, as `_applied_clip` is for the composition — so no
/// reader can compose it differently from the march that produced its base point.
///
/// # BOTH OF PYTHON's BARS ARE RE-MEASURED, AND ONE OF THEM CHANGES NUMBER
///
/// § 6's table sanctions `include_str!` + `.matches().count()` for `test_rung73.py:488` and
/// `:492`. Neither number transfers as written:
///
/// * **The needle differs.** Python counts `g_own + req - clip`; the port spells the same
///   expression `(g_own + req) - clip`, with the parenthesis PINNED (probe L4: at `g_own = 1e16`
///   the rearrangement is a full unit apart). And the port's own doc comment QUOTES the needle at
///   `applied_reference.rs:308`, where `inspect.getsource(AppliedReferenceTransient)` had a
///   class-sized denominator and this has a file-sized one. So the code count and the doc count
///   are taken SEPARATELY and both are asserted — the doc occurrence is not filtered away and
///   forgotten, because a filter that silently dropped the code line too would leave `0 == 0`.
/// * **The count differs, 4 -> 1 SEAT and 4 CALLS.** Python reaches the law at four
///   `self._reference(` sites inside `_integrate_fuel_shared`. The port hoists them into ONE
///   closure, `core_ref`, which holds the single dispatch through the cell and is called at exactly
///   the same four places (twice in `der`, twice in the initial-condition sweep). So Python's
///   `== 4` ports as the CALL count and the port gains a SEAT count of 1 that Python cannot state.
///
/// **THE INSTRUMENT CARRIES ITS OWN CONTROL**, `slice_ac_cells.rs`'s rule: a `.matches()` over a
/// pattern that happens to appear nowhere returns 0 and passes an `assert!(n >= …)`. A deliberately
/// absent needle is counted too. And the scoping is by `.lines()`, never by a brace-newline anchor
/// — § 5.29.2 (f)'s CRLF trap took down exactly such a gate in `rung71.rs`, and 21 files in this
/// crate are still CRLF.
#[test]
fn the_reference_lives_in_one_place() {
    const SRC: &str = include_str!("../src/applied_reference.rs");
    const PARENT: &str = include_str!("../src/shared_actuator.rs");

    let is_doc = |l: &&str| l.trim_start().starts_with("//");
    let split = |src: &str, needle: &str| -> (usize, usize) {
        let code = src.lines().filter(|l| !is_doc(l)).filter(|l| l.contains(needle)).count();
        let doc = src.lines().filter(is_doc).filter(|l| l.contains(needle)).count();
        (code, doc)
    };

    // THE LAW ITSELF — once in the body, and once in the doc comment that pins its association.
    let (code, doc) = split(SRC, "(g_own + req) - clip");
    assert_eq!(code, 1, "the applied reference must appear exactly once in this module");
    assert_eq!(doc, 1, "and its association is documented exactly once — a filter that dropped \
                        BOTH lines would leave `0 == 0` and gate nothing");
    // THE CONTROL, AND IT IS TWO-SIDED FOR THE SAME REASON THE NEEDLE ABOVE IS.
    //
    // **THE FIRST WRITING OF THIS LINE WAS A BARE FILE-WIDE `== 0` AND IT FAILED, 2 AGAINST 0.**
    // The rearrangement appears TWICE in `applied_reference.rs` — at `:309` in the doc comment
    // that explains why the association is pinned, and at `:324` in the inline *"Do not rewrite
    // as …"* beside the expression itself. Both are prose warning AGAINST it; neither is code.
    // A bare count could not tell those apart from the defect, and the split below is strictly
    // stronger than the `== 0` it replaces: the CODE count says the rearrangement is not written,
    // and the PROSE count says the counter can find the string at all — which a needle absent
    // everywhere cannot demonstrate. `slice_ac_cells.rs`'s deliberately-absent-pattern rule, made
    // two-sided.
    let (bad_code, bad_doc) = split(SRC, "req + (g_own - clip)");
    assert_eq!(bad_code, 0, "the rearrangement is not written in code");
    assert_eq!(bad_doc, 2, "and it IS written twice in prose — `:309` explains the pin, `:324` \
                            forbids the rewrite — which is how this control proves it can SEE");

    // THE PARENT's MARCH — ONE seat, FOUR calls.
    assert_eq!(PARENT.matches("(ft.inner.triple_hooks.reference)(").count(), 1,
               "the march reaches the reference through ONE dispatch site");
    assert_eq!(PARENT.matches("core_ref(").count(), 4,
               "and calls it at Python's four: twice in `der` and twice in the initial-condition \
                sweep, once per leg in each");
    // THE INSTRUMENT's CONTROL — a needle the file does not carry must read zero. Here the bare
    // form is admissible where it was not above, because the counter's ability to SEE inside
    // `PARENT` is already demonstrated by the two positive counts on the same string two lines up.
    assert_eq!(PARENT.matches("self._reference(").count(), 0,
               "the counter can miss: Python's own spelling is not in the port");
}

// --- THE MARCH AUDIT: rung 79's gap seam, checked from the other end ------------------------
// `docs/rungs72-77-march-audit.md`. A CONFIRMATION's gate, not this rung's anchor.

/// The applied reference does not change the answer rung 72's arm gives: at `phi_lim = 0.80` in the
/// CLIP coordinate the plant accelerates, and all four loops act. Rungs 78/79 stand still at the
/// same wall in the DEMAND coordinate (rung 74 § 2.2) — the cell, not the rig.
#[test]
fn this_rungs_march_moves_and_all_four_loops_are_live() {
    let m = applied_of(&valve_arm());
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
    let b_max = m.fuel.inner.lever.lim.expect("the receiver arms a valve").b_max;
    let required = |p: &FuelPoint| match p.extra {
        PointExtra::Shared { required, .. } => required,
        _ => panic!("rung-73's audit reads `required` with a bare index"),
    };
    assert!(traj.iter().filter(|p| required(p) > 0.0).count() > 300);
    assert!(traj.iter().filter(|p| matches!(p.extra,
                PointExtra::Shared { b_cmd, .. } if 0.0 < b_cmd && b_cmd < b_max)).count() > 50);
    assert!(traj.iter().filter(|p| matches!(p.extra,
                PointExtra::Shared { v_regime: Some(Regime::Riding), .. })).count() > 50);
    let min_phi = traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min);
    assert!(min_phi > 0.78, "{min_phi}");
}
