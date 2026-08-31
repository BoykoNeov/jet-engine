//! RUNG 70 — **THE GENERIC SPLIT**: rung 47's `Tt4` topping GOVERNOR as the odd loop beside rung
//! 65's `phi` valve and rung 68's `phi` stator. Rung 67's substitution applied to rung 68's triple.
//! Five states, three clocks, `n = 3`, `m = 2` — **the same cell as rung 69, reached by a different
//! route**, so this is a controlled comparison at equal counts.
//!
//! **THE HEADLINE:** the split buys the RANK; the RING needs the odd constraint to be a SECOND
//! WALL ON THE SAME LEVER. Rung 69's ringing pair came from `k ≈ −1.7`, which was ONE LEVER
//! READING TWO WALLS. Here the odd constraint sits on a different lever, both split pairs are
//! cross-LEVER gains, and the damping floor lands at ~0.99 — where rung 67 put it, by the same
//! scalar.
//!
//! **AND THE IDENTITY MOVES RATHER THAN VANISHING.** `pair_CV = 1` now (the valve and the stator
//! share `phi`); `pair_RC` and `pair_RV` split — and they come back with OPPOSITE SIGNS, which no
//! single scalar can summarise. The cyclic product equals `−pair_RC` and is structurally BLIND to
//! `pair_RV`, so rung 68's *quote `x`* and rung 69's *`x = −k`* both stop being complete.
//!
//! Ported from `tests/test_rung70.py` — **27 tests, of which 11 carry `slow` there** (MEASURED at
//! step 4 by `pytest -m slow --collect-only`, which reports 22 of 57 over the slice and 11 in each
//! file; the first writing of this line said 15, and § 5.27's own sizing had 22 all along). The marker is
//! dropped here per slice M's rule; `#[ignore]` is re-introduced only against a MEASURED Rust cost,
//! never inherited.
//!
//! # THE ONE PYTHON OBSERVABLE RUST CANNOT SPELL THE SAME WAY
//!
//! `test_at_lever_returns_this_class` opens with `type(m) is CrossSplitTransient`. There is no
//! runtime class here — every rung in this family is a [`ScheduledStatorCore`] and the rung is the
//! TABLE it carries; comparing the table's address would test the optimiser rather than the port,
//! the defect this phase has now recorded twice. So the sibling is instead made to **exercise a
//! cell only rung 70's table has**: it must MARCH under `tau_gov` with a `phi` stator armed, which
//! rung 69's inherited table refuses outright (rung 68's `assert tau_gov is None`). A sibling
//! handed back carrying the parent's table passes every float in that gate and panics on this.
//!
//! # TWO GRIDS ARE WRITTEN OUT HERE AND THEY ARE NOT WRITTEN IN THE STATE VECTOR's ORDER
//!
//! `split_modes`' `clocks` and `split_floor`'s `grid` are Python DEFAULTS that the Rust takes as
//! explicit slices, so this file spells them. Both are `(tau_q, tau_gov, tau_s)` on the way IN and
//! are reported as `taus = (tau_g, tau_q, tau_s)` — the `(g, q, v)` order of the STATE VECTOR — on
//! the way out. Entries asymmetric in the first two slots therefore key differently from how they
//! are written, which is rung 69's own recorded trap; see [`CLOCKS`] and [`FLOOR_GRID`].

use std::panic::catch_unwind;

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::cross_loop::build_cross_loop_cascade;
use turbojet::cross_split::{
    build_cross_split_cascade, c1_clock_swap, rung67_control, split_bill, split_floor, split_gains,
    split_modes, window_overlap, SplitFloorLive,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{powp, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{build_reference_split_cascade, invariants, StatorIncidenceLimiter};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{build_three_loop_cascade, StatorLimiter, TripleGains};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------------- the grid
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
/// `PHI / FLOOR - 1.0` — the expression Python spells, never a typed decimal: the three floors
/// being ONE PHYSICAL WALL is what makes `pair_CV = 1` a measurement rather than a coincidence,
/// and a rounded constant would break it silently.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
/// **RUNG 67's imposed redline, VERBATIM** — see `docs/rung70-spec.md` § 3. Rung 70 inherits the
/// number rather than choosing one, which is exactly why `all_three_windows_overlap` is a GATE.
const TT4_MAX: f64 = 1200.0;

/// `split_modes`' Python default, in the `(tau_q, tau_gov, tau_s)` order the reader TAKES.
///
/// Reported back as `taus = (tau_g, tau_q, tau_s)`, so entry 2 goes in as `(0.05, 0.005, 0.05)`
/// and is keyed as `(0.005, 0.05, 0.05)`. Nothing below looks an arm up by its clocks — the gates
/// sweep every arm — so the reorder cannot bite here; it is written down because a later reader
/// that DOES look one up would be the first to hit it.
const CLOCKS: [(f64, f64, f64); 4] =
    [(0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05), (0.02, 0.05, 0.10)];

/// `split_floor`'s Python default — NINE entries, same `(tau_q, tau_gov, tau_s)` convention.
///
/// The grid deliberately straddles BOTH extremes (a slow valve AND a slow stator) rather than
/// assuming which loop the equality set silences; `silenced` is a per-row reading, and gate 22
/// asserts the column came back constant instead of taking it on faith.
const FLOOR_GRID: [(f64, f64, f64); 9] = [
    (0.05, 0.05, 0.05), (0.05, 0.05, 0.025), (0.05, 0.05, 0.10),
    (0.10, 0.10, 0.05), (0.02, 0.20, 0.05), (0.20, 0.02, 0.05),
    (2.00, 0.05, 0.05), (0.05, 0.05, 2.00), (0.05, 2.00, 2.00),
];

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

/// Python's `_cross` — THE rung-70 machine.
fn cross_of(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_cross_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_ref` — rung 69.
fn ref_of(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_reference_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_three` — rung 68.
fn three_of(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_three_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_cross67` — rung 67's cascade A.
fn cross67_of(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_cross_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_valve`. Spelled through `from_margin_tau` and NOT through `with_tau(PHI, …)`: the
/// margin form is what Python calls, and `(1 + SM) * phi_surge` is not obliged to round back to
/// `PHI` exactly.
fn valve(tau: f64) -> BleedLimiter { BleedLimiter::from_margin_tau(&lp(), B, SM, Some(tau)) }

/// Python's `_phi_stator` — rung 68's floor, from the SAME margin, so the two `phi` loops share
/// ONE physical wall by construction rather than by a typed float.
fn phi_stator(tau: f64, v_max: f64) -> StatorLimiter {
    StatorLimiter::from_margin(&lp(), v_max, SM, Some(tau))
}

/// Python's `_inc` — rung 69's INCIDENCE floor. Used here only to be REFUSED.
fn inc(tau: f64, v_max: f64) -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), v_max, SM, Some(tau))
}

fn fuel_floor() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM)) }

fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

/// Python's `_march`. The two arming knobs live in two different places here — `Tt4_max` is a LEG
/// argument and `tau_gov` is a SCOPE field — where Python passes both as march keywords.
fn march(
    m: &ScheduledStatorCore, ds: f64, surge: Option<Floor>, lg: Option<AsymmetricLag>,
    tt4_max: Option<f64>, tau_gov: Option<f64>,
) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge, tt4_max };
    m.stator_march_scoped(&flight(), &ramp(ds), None, &leg,
                          &MarchScope { lag: lg, tau_gov, ..MarchScope::DEFAULT }).0
}

/// The rung-70 march — the governor armed, no fuel leg (guard C forbids the pair).
fn gov_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    march(m, DS, None, None, Some(TT4_MAX), Some(TAU_GOV))
}

/// Python's `_keys` — the seven-tuple per point the reduce gates compare, BIT for bit.
fn keys(traj: &[FuelPoint]) -> Vec<[u64; 7]> {
    traj.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits()])
        .collect()
}

/// Python's `"v" in p` — a key test on a dict, which in Rust is a variant test on the point.
///
/// **`Triple` ALONE**, which is what `tests/rung68.rs` and `tests/rung69.rs` already spell under
/// this same doc line. [`PointExtra::CrossCascade`] is rung 67's cascade-A point and has **no `v`
/// field at all** — admitting it would answer `true` for a dict Python has no `v` key in, and the
/// one gate that reads this helper is the one asserting the FIFTH STATE exists.
fn carries_v(p: &FuelPoint) -> bool {
    matches!(p.extra, PointExtra::Triple { .. })
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

/// THE rung-70 machine — the governor beside the valve and the `phi` stator. **Rebuilt per test
/// rather than shared**, for `tests/rung68.rs`'s reason exactly: each `#[test]` is its own thread
/// and [`ScheduledStatorCore`]'s `Cell` fields — which ARE the dynamically scoped state — are
/// deliberately not `Sync`.
fn cross() -> ScheduledStatorCore {
    cross_of(&LeverArm { bleed_lim: Some(valve(TAU)), stator_lim: Some(phi_stator(TAU_S, V_MAX)),
                         ..Default::default() })
}

/// Python's module-scoped `gains` fixture, at its own call's arguments.
fn gains() -> turbojet::cross_split::SplitGains {
    split_gains(&cross(), &flight(), LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV, TAU_S,
                V_MAX, 10)
}

/// Python's module-scoped `bill` fixture.
fn bill() -> turbojet::cross_split::SplitBill {
    split_bill(&cross(), &flight(), LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV, TAU_S, V_MAX)
}

/// Python's `f["rows"]` filtered by `"zeta" in x` — the three shapes `split_floor` appends are one
/// live dict and two dead ones, and only the live one carries the key.
fn floor_live(f: &turbojet::cross_split::SplitFloor) -> Vec<SplitFloorLive> {
    f.rows.iter().filter_map(|x| x.live).collect()
}

/// `x["zeta"]` where Python would raise `TypeError` on `None` — the reader is measured to return
/// `None` on 0 of 96 shipped calls, so an `expect` is that same refusal said out loud.
fn zeta_of(x: &SplitFloorLive) -> f64 {
    x.zeta.expect("a live floor row with zeta = None — Python raises TypeError here")
}

// =============================================================================================
// GATE 1 — THE REDUCE. Rung 70 substitutes ONE loop's SENSOR, so every ancestor must still be
//          reached BIT-FOR-BIT, and by DISPATCH rather than by a numerical coincidence.
// =============================================================================================

/// `tau_gov = None` with a rung-68 `phi` stator: rung 68's own five-state cascade, untouched.
#[test]
fn reduce_no_governor_is_rung68_bit_for_bit() {
    let arm = LeverArm { bleed_lim: Some(valve(TAU)),
                         stator_lim: Some(phi_stator(TAU_S, V_MAX)), ..Default::default() };
    let a = march(&cross_of(&arm), DS, Some(fuel_floor()), Some(lag()), None, None);
    let b = march(&three_of(&arm), DS, Some(fuel_floor()), Some(lag()), None, None);
    assert_eq!(keys(&a), keys(&b));
}

/// `tau_gov = None` with rung 69's INCIDENCE stator: rung 69's plant, untouched.
#[test]
fn reduce_no_governor_incidence_is_rung69_bit_for_bit() {
    let arm = LeverArm { bleed_lim: Some(valve(TAU)), stator_inc: Some(inc(TAU_S, V_MAX)),
                         ..Default::default() };
    let a = march(&cross_of(&arm), DS, Some(fuel_floor()), Some(lag()), None, None);
    let b = march(&ref_of(&arm), DS, Some(fuel_floor()), Some(lag()), None, None);
    assert_eq!(keys(&a), keys(&b));
}

/// A governor and a valve with NO stator is rung 67 — this class never intercepts a march it does
/// not own, so cascade A is reached through the parent's own dispatch.
#[test]
fn reduce_no_stator_is_rung67_bit_for_bit() {
    let arm = LeverArm { bleed_lim: Some(valve(TAU)), ..Default::default() };
    let a = gov_march(&cross_of(&arm));
    let b = gov_march(&cross67_of(&arm));
    assert_eq!(keys(&a), keys(&b));
    assert!(!matches!(a[0].extra, PointExtra::Triple { .. }),
            "no stator armed => no fifth state");
}

/// Rungs 66/65/64/62's arms all leave through the same `super()`.
///
/// **FOUR cases, which is Python's count** — a silently-shortened loop is this phase's *"a count
/// typed instead of added up"*, so the length is asserted rather than trusted to the reader.
#[test]
fn reduce_inherited_arms_bit_for_bit() {
    let cases: [(LeverArm, Option<Floor>, Option<AsymmetricLag>); 4] = [
        (LeverArm { bleed_lim: Some(valve(TAU)), ..Default::default() },
         Some(fuel_floor()), Some(lag())),
        (LeverArm { bleed_lim: Some(valve(TAU)), ..Default::default() },
         Some(fuel_floor()), None),
        (LeverArm::default(), Some(fuel_floor()), Some(lag())),
        (LeverArm { bleed_sched: Some(BleedSchedule::new(B, 0.65)), ..Default::default() },
         None, None),
    ];
    assert_eq!(cases.len(), 4, "Python's loop has four arms");
    for (i, (arm, surge, lg)) in cases.into_iter().enumerate() {
        let a = march(&cross_of(&arm), DS, surge, lg, None, None);
        let b = march(&ref_of(&arm), DS, surge, lg, None, None);
        assert_eq!(keys(&a), keys(&b), "case {i}");
    }
}

/// **THE EIGHTH INSTANCE of the trap rungs 61–69 each hit**: the inherited sibling constructor
/// hardcodes its own name, so a rung-70 machine would hand back a rung-69 one and every reader
/// would measure rung 69's plant while reporting rung 70's.
///
/// Python's `type(m) is CrossSplitTransient` has no runtime counterpart here (see the module
/// header). What replaces it is stronger than a type check and stronger than an address
/// comparison: the sibling is made to MARCH UNDER THE GOVERNOR, which only rung 70's
/// `integrate_fuel` admits — rung 69 inherits rung 68's `assert tau_gov is None` and panics.
#[test]
fn at_lever_returns_this_class() {
    let m = cross_of(&LeverArm { bleed_lim: Some(valve(TAU)), ..Default::default() });
    let s = m.at_lever(&LeverArm { bleed_lim: Some(valve(TAU)),
                                   stator_lim: Some(phi_stator(TAU_S, V_MAX)),
                                   ..Default::default() });
    assert!(s.fuel.inner.stator.lim.is_some() && s.fuel.inner.stator.inc.is_none());
    let t = gov_march(&s);
    assert!(!t.is_empty() && carries_v(&t[0]),
            "the sibling must carry rung 70's OWN table — a parent's refuses `tau_gov` outright, \
             which is what `type(m) is CrossSplitTransient` asserts in Python");
}

// =============================================================================================
// GATE 2 — THE REFUSALS. Each names a plant this rung is NOT, and each is a seam.
// =============================================================================================

/// Three loops on THREE constraints — `n = m = 3`, ZERO zeros, the one cell of rung 69's table
/// this ladder has never occupied. Rung 70's own next seam, refused not run.
#[test]
fn an_incidence_stator_beside_the_governor_is_refused() {
    assert!(panics_with(|| {
        let m = cross_of(&LeverArm { bleed_lim: Some(valve(TAU)),
                                     stator_inc: Some(inc(TAU_S, V_MAX)), ..Default::default() });
        gov_march(&m);
    }, "n = m = 3"));
}

/// `n = 4, m = 2` — FOUR loops, two of them on the same actuator. Rung 68's own `tau_gov` assert
/// exists because *silently accepts it* is the failure mode; this is its mirror.
#[test]
fn the_fuel_leg_beside_the_governor_is_refused() {
    assert!(panics_with(|| {
        let m = cross();
        march(&m, DS, Some(fuel_floor()), Some(lag()), Some(TT4_MAX), Some(TAU_GOV));
    }, "n = 4, m = 2"));
}

/// `tau_gov` without `Tt4_max` would march as rung 68 while every reader reported rung 70 — a
/// wrong-plant failure that no float would reveal.
#[test]
fn a_governor_with_no_set_point_is_refused() {
    assert!(panics_with(|| {
        let m = cross();
        march(&m, DS, None, None, None, Some(TAU_GOV));
    }, "odd loop IS the redline"));
}

/// Rung 65 published a RETRACTION for an RK4 instability that looked like a physical finding, and
/// rung 68 measured that at `ds` its own constant refuses the march counterfeits PERFECT
/// PROTECTION. The guard is re-justified here on a third argument, so it must fire.
///
/// **The needle is the RUNG TAG and not the sentence**: probe 2b measured `rank TWO` in rung 69's
/// message AND in this one, so only `rung-70: ds` matches exactly one.
#[test]
fn the_rk4_floor_fires_and_names_its_own_reason() {
    assert!(panics_with(|| {
        let m = cross();
        march(&m, 0.05, None, None, Some(TT4_MAX), Some(TAU_GOV));
    }, "rung-70: ds"));
}

/// A GATE, NOT A REMARK. `Tt4_max` is inherited from rung 67, which chose it for overlap with ONE
/// `phi` loop. A gain table over an empty intersection would report the pairwise algebra of loops
/// that were never simultaneously live.
#[test]
fn all_three_windows_overlap() {
    let w = window_overlap(&cross(), &flight(), LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV,
                           TAU_S, V_MAX);
    assert!(w.overlaps, "{w:?}");
    assert!(w.joint.2 >= 20, "the joint window is too thin to sample: {:?}", w.joint);
    for (name, leg) in [("gov", w.gov), ("valve", w.valve), ("stator", w.stator)] {
        assert!(leg.2 > 0, "{name} never rides at all: {leg:?}");
    }
}

// =============================================================================================
// GATE 3 — § 1: THE IDENTITY MOVED, and the two split pairs DIFFER. THE RUNG.
// =============================================================================================

/// `pair_CV = C_v V_q = 1` EXACTLY: the valve and the stator solve the SAME constraint, so their
/// rows are parallel and the implicit-function derivatives are reciprocal.
///
/// WHICH PAIR KEEPS RUNG 66's IDENTITY IS A DIRECT READ OF WHICH LOOPS SHARE A CONSTRAINT — rung
/// 69's statement, and here it moves from `(R,C)` to `(C,V)`.
#[test]
fn the_shared_pair_is_now_cv_and_it_holds_to_the_floor() {
    let g = gains();
    assert!(!g.rows.is_empty(), "no interior riding point");
    let worst_cv = g.worst_cv.expect("rows exist, so the aggregate does");
    assert!(worst_cv < 1e-8, "{worst_cv}");
    // and NEITHER split pair is 1 — the identity did not merely spread
    assert!(g.worst_rc_is_1.expect("rows exist") > 0.9);
    assert!(g.worst_rv_is_1.expect("rows exist") > 0.8);
}

/// RUNG 69 § 1.1: `pair_RV = pair_CV` is NOT general to a split — it holds iff the odd constraint
/// depends on the shared actuators ONLY through the shared constraint. At rung 69 that held
/// trivially and both split pairs collapsed onto ONE scalar `k`. Here they do not.
///
/// AND THEY COME BACK WITH OPPOSITE SIGNS, which is stronger than the registered prediction: the
/// odd constraint couples with opposite sign through the two shared actuators (bleed makes it
/// hotter; a closed stator does not reach `Tt4` the same way). No single scalar can summarise that.
#[test]
fn the_two_split_pairs_are_different_and_that_is_the_rung() {
    let g = gains();
    assert!(g.pair_rc.iter().all(|&x| x < 0.0), "{:?}", g.pair_rc);
    assert!(g.pair_rv.iter().all(|&x| x > 0.0), "{:?}", g.pair_rv);
    // separated by ORDERS above the instrument's own floor (`worst_cv`, ~1e-10)
    let gap = g.min_pair_gap.expect("rows exist");
    assert!(gap > 0.5, "{gap}");
    let closest = g.pair_rc.iter().zip(&g.pair_rv)
        .map(|(a, b)| (a - b).abs())
        .fold(f64::INFINITY, f64::min);
    assert!(closest > 1e6 * g.worst_cv.expect("rows exist").max(1e-16));
}

/// `x = R_q C_v V_g = −pair_RC` identically. Rung 68 said *quote `x`*; rung 69 said *`x` flips to
/// `−k`*. Both were complete only because every split pair was one scalar. Here `x` reproduces ONE
/// of the two and structurally cannot see the other — rung 68's own *check what is INDEPENDENT
/// before quoting it*, in its second shape.
#[test]
fn the_cyclic_product_is_minus_pair_rc_and_blind_to_pair_rv() {
    let g = gains();
    let w = g.worst_cyclic_is_rc.expect("rows exist");
    assert!(w < 1e-8, "{w}");
    // the thing `x` cannot see: it would have to differ from `-pair_RV` by a lot, and it does
    for row in &g.rows {
        assert!((row.gov.cyclic + row.gov.pair_rv).abs() > 0.1);
    }
}

/// The contrast that makes *moved* a measurement rather than a comparison of two rungs' tables:
/// rung 68's FUEL leg is re-read at the IDENTICAL base points, and there `pair_RC` is 1 to the
/// differencing floor while under the governor it is ~ −0.018.
#[test]
fn the_identity_moved_measured_on_one_trajectory() {
    let g = gains();
    let f = g.worst_rc_fuel.expect("the fuel arm must have been read at every point");
    assert!(f < 1e-8, "{f}");
    assert!(g.worst_rc_is_1.expect("rows exist") > 0.9);
}

/// THE `_b_state`/`_v_state` BOUNDARY, asserted rather than inherited — rung 68 flags it as the
/// one thing here that can go wrong without failing. `R_q != 0` and `R_v != 0` ONLY because the
/// governor senses `Tt4` on the machine as the other two actuators actually are; drop the boundary
/// and both are identically zero, the odd loop decouples, and every prediction in this rung would
/// *confirm* rung 68 instead.
#[test]
fn a_zero_cross_gain_would_be_a_missing_coupling_not_a_weak_one() {
    let g = gains();
    assert!(!g.boundary.is_empty(), "the boundary check never ran");
    for chk in &g.boundary {
        assert_eq!(chk.dead_r_q, 0.0);
        assert_eq!(chk.dead_r_v, 0.0);
        assert!(chk.live_r_q.abs() > 0.0 && chk.live_r_v.abs() > 0.0);
    }
}

/// `pair_RC` HERE IS rung 67's `P = R_q C_g` — same governor, same valve, same shipped closures.
/// The only difference is that a third loop is present and has moved the base point, so the two
/// must agree in SIGN and ORDER OF MAGNITUDE.
///
/// IT IS A CONTROL, NOT A FINDING: a departure beyond the base-point shift means the state
/// boundary is wrong, not that the plant changed. It is therefore checked loosely and on purpose —
/// a tight tolerance here would be asserting that a third loop changes nothing.
#[test]
fn pair_rc_reproduces_rung67_p_the_negative_control() {
    let c = rung67_control(&cross(), &flight(), LO, HI, TT4_MAX, SM, TAU, TAU_GOV, TAU_S, V_MAX,
                           R, SETTLE, DS, 10);
    assert_eq!(c.both_negative, Some(true), "{c:?}");
    let ratio = c.ratio.expect("both arms produced a pair");
    assert!(0.5 < ratio && ratio < 2.0, "{c:?}");
}

// =============================================================================================
// GATE 4 — § 2: ONE ZERO, `det` BLIND, and `c1` a CLOCK-WEIGHTED SUM.
// =============================================================================================

/// `zeros = n − m` = 1 at `(n,m) = (3,2)` — the SAME cell as rung 69, reached without an incidence
/// wall. Rung 69 established the law on one realization of the cell; this is the second, and it is
/// the one where the odd constraint does NOT factor.
#[test]
fn the_rank_is_the_constraint_count_at_a_second_realization() {
    let mo = split_modes(&cross(), &flight(), LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, 0.002,
                         V_MAX, 20);
    assert_eq!(mo.arms.len(), CLOCKS.len());
    for arm in &mo.arms {
        assert!(!arm.rows.is_empty(), "{:?}", arm.taus);
        assert_eq!(arm.zeros, vec![1], "{:?} {:?}", arm.taus, arm.zeros);
    }
}

/// `c0 = det J = 0` under this split as well — the valve and the stator keep exactly parallel rows
/// whatever the governor watches. **A reader that inherited rung 68's determinant test would report
/// rank one and see nothing**, which is rung 69's correction re-confirmed on a plant its derivation
/// does not cover.
///
/// `c1` is the discriminator again, and the measured value matches the two-term closed form of
/// § 1.4 — which is what says the two split pairs enter on DIFFERENT clock products.
#[test]
fn det_is_blind_to_this_split_too_and_c1_is_the_discriminator() {
    let mo = split_modes(&cross(), &flight(), LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, 0.002,
                         V_MAX, 20);
    for arm in &mo.arms {
        let c0 = arm.max_c0_rel.expect("rows exist");
        let c1 = arm.min_c1_rel.expect("rows exist");
        let err = arm.max_c1_err.expect("rows exist");
        assert!(c0 < 1e-9, "{:?} {c0}", arm.taus);
        assert!(c1 > 1e-2, "{:?} {c1}", arm.taus);
        assert!(err < 1e-7, "{:?} {err}", arm.taus);
    }
}

/// THE DISCRIMINATING TEST, and the only one here that a one-scalar plant fails.
///
/// That `c1 != 0` is rung 69's result; that it moves across a clock grid proves nothing (the rate
/// sum moves too); that it matches this rung's own formula validates the formula against itself.
/// **Hold `tau_g` and exchange `(tau_q, tau_s)`:** rung 69's shape (`u == w`) makes `c1` SYMMETRIC
/// in that exchange and therefore INVARIANT, while two terms change by
/// `(u−w)(1/(tau_g tau_q) − 1/(tau_g tau_s))`.
///
/// The null is built from THIS plant's own gains forced to one scalar, so the comparison is between
/// two models of one measurement rather than between two plants. **Every `c1` here comes from the
/// shipped [`invariants`]** — the actual 3×3 Jacobian — so the agreement with § 1.4's closed form
/// is a test of the algebra and not a formula agreeing with itself.
#[test]
fn the_clock_swap_kills_the_one_scalar_model() {
    let sw = c1_clock_swap(&cross(), &flight(), LO, HI, TT4_MAX, SM, 0.05, 0.02, 0.10, R, SETTLE,
                           DS, V_MAX);
    // the one-scalar null is invariant under the swap — rung 69's shape, on rung 70's numbers
    assert!((sw.one_scalar_null.ratio - 1.0).abs() < 1e-12, "{:?}", sw.one_scalar_null);
    assert!(sw.null_delta.abs() < 1e-9 * sw.measured_delta.abs(), "{sw:?}");
    // and this plant is decisively NOT that
    assert!((sw.held_gains.ratio - 1.0).abs() > 0.05, "{:?}", sw.held_gains);
    assert!((sw.measured_delta / sw.predicted_delta - 1.0).abs() < 1e-9, "{sw:?}");
    // the marched arms agree with the held-gains reading up to the plant's own drift
    assert!((sw.marched_ratio / sw.held_gains.ratio - 1.0).abs() < 0.05, "{sw:?}");
}

/// A FREE STRUCTURAL CHECK. Rung 69's two `c1` terms both carry `1/tau_s`, its ODD loop's clock;
/// both of rung 70's carry `1/tau_g`, this rung's odd loop. The pair that SHARES contributes
/// nothing to `c1`, so the surviving factor is always the odd loop's clock — the clock products are
/// a read of which two loops share a constraint.
///
/// Measured by holding `tau_q = tau_s` and moving `tau_g` alone: `c1` must scale as `1/tau_g`
/// EXACTLY, which a `1/tau_s`-carrying model cannot do.
///
/// The CONTROL is a hand-built rung-69 block: there the SHARED pair is `(R,C)`, so `1/tau_s`
/// survives instead and `c1` provably cannot halve when `tau_g` doubles. **Forcing `u == w` would
/// NOT be that control** — which pair shares is what selects the clock, not whether the split pairs
/// happen to be equal.
#[test]
fn the_surviving_clock_product_names_which_loops_share() {
    let sw = c1_clock_swap(&cross(), &flight(), LO, HI, TT4_MAX, SM, 0.05, 0.05, 0.05, R, SETTLE,
                           DS, V_MAX);
    let gg = &sw.fast_valve.gains;
    assert!((gg.pair_cv - 1.0).abs() < 1e-8, "this plant's shared pair is (C,V)");
    let c1: Vec<f64> = [0.05, 0.10].iter()
        .map(|&tau_g| invariants(gg, (tau_g, 0.05, 0.05)).1).collect();
    assert!((c1[0] / c1[1] - 2.0).abs() < 1e-9, "{c1:?}");

    // THE CONTROL: rung 69's arrangement — (R,C) share, so `c1 = (1-k)(1/tau_g + 1/tau_q)/tau_s`.
    // `k = -1.7` is rung 69's own measured value, near enough. Only the six cross-gains are read
    // by `invariants`; the rest of the struct is filled with values that cannot enter it.
    let k = -1.7;
    let r69 = TripleGains {
        interior: true, off_regime: Vec::new(),
        r_q: 2.0, c_g: 0.5,     // pair_RC = 1   (the SHARED pair)
        r_v: 1.0, v_g: k,       // pair_RV = k   (split)
        c_v: 1.0, v_q: k,       // pair_CV = k   (split)
        v_base: 0.0, cyclic: 2.0 * 1.0 * k, pair_rc: 1.0, pair_rv: k, pair_cv: k, s: 0.0,
    };
    let n1: Vec<f64> = [0.05, 0.10].iter()
        .map(|&tau_g| invariants(&r69, (tau_g, 0.05, 0.05)).1).collect();
    assert!((n1[0] / n1[1] - 2.0).abs() > 0.5, "{n1:?}");
    // exactly what its own form predicts
    assert!((n1[0] / n1[1] - 4.0 / 3.0).abs() < 1e-9, "{n1:?}");
}

// =============================================================================================
// GATE 5 — § 3: THE FLOOR. An INFIMUM on a RAY, and P8's REFUTATION.
// =============================================================================================

/// `zeta >= 1/sqrt(1 − min(pair_RC, pair_RV))` over every bandwidth, and STRICTLY.
///
/// RUNG 69's EQUALITY SET WAS A HYPERPLANE (`u == w` makes `b, c` enter only through `b+c`, so
/// `a = b+c` attains it with all three clocks finite). Here it collapses to a RAY — one shared loop
/// silenced AND `a` matched to the other — so the bound is an INFIMUM that no admissible triple
/// reaches. The closed form is checked against the shipped cubic's own roots.
#[test]
fn the_floor_holds_and_is_strict_at_every_admissible_bandwidth() {
    let f = split_floor(&cross(), &flight(), LO, HI, TT4_MAX, SM, &FLOOR_GRID, R, SETTLE, DS,
                        V_MAX);
    let live = floor_live(&f);
    assert!(live.len() >= 6, "{} live rows of {}", live.len(), f.rows.len());
    assert!(f.holds && f.strict, "{:?}", f.tightest);
    let err = f.worst_pred_err.expect("live rows exist");
    assert!(err < 1e-8, "{err}");
}

/// PRE-REGISTERED P8 SAID *NO COMPLEX PAIR AT ANY BANDWIDTH*. **That is FALSE**, and the refutation
/// is the better result.
///
/// The floor is `~0.990 < 1`, so a complex pair is ADMITTED — and it is found, on the arm
/// `tau_s = 40×` the others, i.e. the RAY that nearly silences the stator. So the honest sentence
/// is not *no ring* but: the ring is reachable only where the third loop is dynamically inert, and
/// even there `zeta ~ 0.992` puts it back in rung 67's *admissible, unobservable* class.
///
/// THE RAY HAS TWO COORDINATES AND BOTH ARE CHECKED. § 1.5's equality needs the silenced loop's
/// share to vanish AND `a` matched to the SURVIVING shared rate. Here `u > w`, so the ray silences
/// the STATOR and the survivor is the VALVE — equality wants `tau_g = tau_q`, NOT `tau_g = tau_s`.
/// An arm that is near on `quiet_share` alone is not on the ray, and the grid carries one of each
/// so the distinction is measured rather than asserted.
#[test]
fn p8_refuted_the_ring_is_reachable_but_only_by_silencing_the_third_loop() {
    let f = split_floor(&cross(), &flight(), LO, HI, TT4_MAX, SM, &FLOOR_GRID, R, SETTLE, DS,
                        V_MAX);
    let live = floor_live(&f);
    // the silenced loop is a PLANT property, constant down the column
    assert!(live.iter().all(|x| x.silenced == "stator"),
            "{:?}", live.iter().map(|x| x.silenced).collect::<Vec<_>>());
    let comparable: Vec<&SplitFloorLive> = live.iter().filter(|x| x.quiet_share > 0.05).collect();
    let ray: Vec<&SplitFloorLive> = live.iter()
        .filter(|x| x.quiet_share <= 0.05 && (x.a_over_loud - 1.0).abs() < 0.1).collect();
    let near: Vec<&SplitFloorLive> = live.iter()
        .filter(|x| x.quiet_share <= 0.05 && (x.a_over_loud - 1.0).abs() >= 0.1).collect();
    assert!(!ray.is_empty() && !near.is_empty(),
            "the grid must carry an on-ray arm AND a near-but-off-ray one");
    assert!(!comparable.iter().any(|x| x.complex_pair),
            "{:?}", comparable.iter().filter(|x| x.complex_pair).map(|x| x.s).collect::<Vec<_>>());
    // ON the ray in BOTH coordinates: complex, and hard against the floor
    for x in &ray {
        assert!(x.complex_pair, "{x:?}");
        assert!(zeta_of(x) > 0.98, "{x:?}");                        // rung 67's mode, not rung 69's
        assert!(zeta_of(x) / x.floor - 1.0 < 0.01, "{x:?}");        // within 1 % of the infimum
    }
    // near on ONE coordinate only: not on the ray, and nowhere near the floor
    for x in &near {
        assert!(!x.complex_pair, "{x:?}");
        assert!(zeta_of(x) / x.floor - 1.0 > 1.0, "{x:?}");
    }
}

/// **ON THIS PLANT** the floor reduces to rung 67's `zeta = 1/sqrt(1+|P|)`, because `min()` selects
/// `pair_RC` — and it selects it only because `pair_RV` came back POSITIVE. So the invariance *a
/// third loop sharing the wall moves the achievable damping nowhere* is CONDITIONAL on that sign,
/// not structural: had `pair_RV` been the more negative one, the floor would be set by a gain rung
/// 67 never measured.
///
/// The gate asserts the condition ALONGSIDE the consequence, so a plant that broke the sign would
/// fail here rather than silently invalidating the identity.
#[test]
fn the_floor_is_rung67s_damping_ratio_and_that_is_contingent() {
    let g = gains();
    assert!(g.pair_rv.iter().all(|&x| x > 0.0), "{:?}", g.pair_rv);
    let worse = g.worse_pair.expect("rows exist");
    let min_rc = g.pair_rc.iter().copied().fold(f64::INFINITY, f64::min);
    let min_rv = g.pair_rv.iter().copied().fold(f64::INFINITY, f64::min);
    assert_eq!(worse, min_rc.min(min_rv));
    assert!(g.pair_rc.contains(&worse), "the floor must be set by rung 67's pair");
    let f = split_floor(&cross(), &flight(), LO, HI, TT4_MAX, SM, &FLOOR_GRID, R, SETTLE, DS,
                        V_MAX);
    let (lo, hi) = f.floor_range;
    let (lo, hi) = (lo.expect("live rows exist"), hi.expect("live rows exist"));
    // Python's `sum(list) / len(list)` — a LEFT fold from 0.0, which is what `Iterator::sum` is.
    let p = (g.pair_rc.iter().sum::<f64>() / g.pair_rc.len() as f64).abs();
    assert!((0.5 * (lo + hi) - powp(1.0 + p, -0.5)).abs() < 5e-3, "{lo} {hi} {p}");
}

/// The guard keeps rung 68's constant on a THIRD argument (the non-zero pair is real and dominated
/// by the rate sum). Rung 65 published a retraction for a trusted stability argument, so
/// `|lam|/sum` is measured along the arc rather than asserted.
#[test]
fn the_inherited_rk4_constant_is_conservative_and_that_is_measured() {
    let f = split_floor(&cross(), &flight(), LO, HI, TT4_MAX, SM, &FLOOR_GRID, R, SETTLE, DS,
                        V_MAX);
    let ratio = f.max_mod_ratio.expect("live rows exist");
    assert!(ratio < 1.0, "{ratio}");
    assert!(f.max_ds_lambda < 2.0, "{}", f.max_ds_lambda);
}

// =============================================================================================
// GATE 6 — § 4: THE LEDGER, in TWO currencies, with OPPOSITE-SIGN cross-credits.
// =============================================================================================

/// The governor owns `Tt4` and the airflow loops own `phi`. Rung 68's three loops shared ONE
/// currency and could only erode each other; here each buys in its own coin.
#[test]
fn each_loop_delivers_on_its_own_currency() {
    let b = bill();
    let bare = b.cell("bare");
    assert!(b.cell("G").e < 0.5 * bare.e, "{} {}", b.cell("G").e, bare.e);
    assert!(b.cell("V").i < 0.2 * bare.i, "{} {}", b.cell("V").i, bare.i);
    assert!(b.cell("S").i < 0.2 * bare.i, "{} {}", b.cell("S").i, bare.i);
    assert!(b.cell("GVS").i < b.cell("VS").i && b.cell("GVS").e < b.cell("VS").e);
}

/// RUNG 67's cross-credit, and it survives the third loop: the VALVE debits the temperature
/// (`R_q > 0` — bleed makes it hotter at fixed fuel) while the GOVERNOR credits the surge margin
/// (`C_g < 0` — clipping fuel raises `phi_lp`). One loop helps the other; the other hurts it — an
/// object a one-currency ledger structurally cannot hold.
#[test]
fn the_cross_credits_have_opposite_signs_rung67s_object_with_a_third_loop() {
    let b = bill();
    assert!(b.marginal_tt4.valve < 0.0, "{:?}", b.marginal_tt4);
    assert!(b.marginal_phi.gov > 0.0, "{:?}", b.marginal_phi);
}

/// RUNG 68's erosion is a property of the SHARED constraint, so it must appear between the valve
/// and the stator and NOT between either of them and the governor. Each `phi` loop's marginal
/// contribution to the triple is a fraction of what it delivers alone.
#[test]
fn the_two_phi_loops_erode_each_other_and_the_governor_does_not() {
    let b = bill();
    let bare = b.cell("bare");
    let alone_v = bare.i - b.cell("V").i;
    let alone_s = bare.i - b.cell("S").i;
    assert!(b.marginal_phi.valve < 0.2 * alone_v, "{:?} {alone_v}", b.marginal_phi);
    assert!(b.marginal_phi.stator < 0.2 * alone_s, "{:?} {alone_s}", b.marginal_phi);
    // the governor's own currency is NOT eroded by the pair it does not share with
    let alone_g = bare.e - b.cell("G").e;
    assert!(b.marginal_tt4.gov > 0.9 * alone_g, "{:?} {alone_g}", b.marginal_tt4);
}
