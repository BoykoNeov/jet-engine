//! RUNG 69 — **THE REFERENCE SPLIT**: rung 68's SAME stator, referenced to INCIDENCE instead of
//! to `phi`, beside the SAME lagged valve (65) and the SAME lagged fuel leg (52). Five states,
//! three clocks, one lever, one physical wall at the design setting. **Only the COORDINATE moves.**
//!
//! **THE HEADLINE:** a loop's COORDINATE, not its actuator, decides whether it adds a ZERO or a
//! RANK. Every row of the actuator block is a multiple of ITS OWN constraint's gradient, so
//! `rank M = dim span{grad c}` and **ZEROS = n − m**, with `m` the number of INDEPENDENT
//! CONSTRAINTS. The loop count never enters: rung 66 (n=2, m=1) one zero; rung 67 (n=2, m=2) none;
//! rung 68 (n=3, m=1) two; this rung (n=3, m=2) **one**.
//!
//! **AND IT CORRECTS HOW RUNG 68's DECOMPOSITION MUST BE READ.** The two loops that still share
//! `phi` keep exactly PARALLEL rows, so `det J = 0` IDENTICALLY under both references — **`det` is
//! BLIND to the split**. What moves is the SECOND invariant `c1`, by twelve orders of magnitude.
//!
//! **THE MODE THE SPLIT CREATES.** The freed root does not land on the real axis: the surviving
//! pair is COMPLEX iff `k < 0`, i.e. iff the lever FIGHTS ITSELF across the two walls, and
//! `zeta >= 1/sqrt(1-k)` for EVERY choice of the three clocks.
//!
//! **AND THE LEDGER'S WHOLE SIGN TABLE FLIPS**: the same lever, same plant, same wall at the
//! design setting, protective or harmful in each currency according to which one its LOOP watches.
//!
//! Ported from `tests/test_rung69.py` — **25 tests, of which 12 carry `slow` there.** The marker
//! is dropped here per slice M's rule and `#[ignore]` is re-introduced only against a MEASURED
//! Rust cost, never inherited.
//!
//! # WHAT THE PYTHON GATES ASSERT THAT RUST CANNOT SPELL THE SAME WAY
//!
//! Two of the 25 name a Python-only observable and are ported to a **behavioural** statement that
//! is at least as strong, rather than to a weaker one:
//!
//! * `test_at_lever_keeps_the_reference` opens with `type(s) is ReferenceSplitTransient`. There is
//!   no runtime class here — every rung in this family is a [`ScheduledStatorCore`] and the rung
//!   is the TABLE it carries. Comparing the table's address would test the optimiser rather than
//!   the port (the defect this phase has now recorded twice), so the sibling is instead made to
//!   **exercise a cell only rung 69's table has**: it must refuse `v0 = -0.05`, which rung 68's
//!   band ACCEPTS. A sibling handed back with rung 68's table passes every float in that gate and
//!   fails this one.
//! * `test_a_float_comparison_against_the_stop_is_not_the_regime` ends with a set-containment
//!   check on the regime labels. Rust's [`Regime`] is a three-variant enum, so that assertion is
//!   discharged by the type; restating it would be a gate that cannot fail.

use std::panic::catch_unwind;

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::{
    build_reference_split_cascade, damping_floor, reference_bill, reference_gains, reference_modes,
    ring_visibility, rk4_margin, DampingLive, RefModesArm, RefModesClock, ReferenceModes,
    StatorIncidenceLimiter,
};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{
    build_three_loop_cascade, ic_family, v_at_point, violation_inc, StatorLimiter, TripleRigArm,
};
use turbojet::two_lag::violation;
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
/// being ONE PHYSICAL WALL is this rung's own guard D, and a rounded constant would break it
/// silently.
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
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

/// `T_c - 1/PHI` — **THE SAME WALL, read at the design setting.** Python's module-level `M_LIM`.
fn m_lim() -> f64 { lp().tan_beta1_crit() - 1.0 / PHI }

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn full(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this grid never disables LP"),
    }
}

/// Python's `_split` — a rung-69 machine.
fn split_of(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_reference_split_cascade(
        design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

/// Python's `_three` — a rung-68 machine, the object every reduce arm is compared against.
fn three(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_three_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }

/// Python's `_inc` — rung 69's INCIDENCE floor, on the same physical wall as the valve's.
fn inc(tau: Option<f64>, v_max: f64) -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), v_max, SM, tau)
}

/// Python's `_phi_stator` — rung 68's floor, built from the SAME margin so guard D's sibling (rung
/// 68's `ONE SET POINT`) is satisfied by construction rather than by a typed float.
fn phi_stator(tau: Option<f64>, v_max: f64) -> StatorLimiter {
    StatorLimiter::from_margin(&lp(), v_max, SM, tau)
}

fn fuel_floor() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, SM)) }

fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

/// Python's `_march`.
fn march(
    m: &ScheduledStatorCore, ds: f64, surge: Option<Floor>, lg: Option<AsymmetricLag>,
    scope_extra: MarchScope,
) -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge, tt4_max: None };
    m.stator_march_scoped(&flight(), &ramp(ds), None, &leg,
                          &MarchScope { lag: lg, ..scope_extra }).0
}

/// The plain armed march — rung 52's fuel leg and its lag, which is the suite's default.
fn armed_march(m: &ScheduledStatorCore) -> Vec<FuelPoint> {
    march(m, DS, Some(fuel_floor()), Some(lag()), MarchScope::DEFAULT)
}

/// Python's `_keys` — the seven-tuple per point the reduce gates compare, BIT for bit.
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

/// `pytest.approx(b, rel=..)`.
fn approx(a: f64, b: f64, rel: f64) -> bool { (a - b).abs() <= rel * b.abs() }

/// THE rung-69 machine and its march — Python's module-scoped `split` fixture.
///
/// **Rebuilt per test rather than shared**, for `tests/rung68.rs`'s reason exactly: each `#[test]`
/// is its own thread and [`ScheduledStatorCore`]'s `Cell` fields — which ARE the dynamically
/// scoped state — are deliberately not `Sync`.
fn split() -> (ScheduledStatorCore, Vec<FuelPoint>) {
    let m = split_of(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                 stator_inc: Some(inc(Some(TAU_S), V_MAX)),
                                 ..Default::default() });
    let t = armed_march(&m);
    (m, t)
}

/// The rig arm every reader below is called with — Python's own six clock defaults, `sm` supplied.
fn rig_arm() -> TripleRigArm { TripleRigArm { sm: SM, ..TripleRigArm::default() } }

// =============================================================================================
// GATE 1 — THE REDUCE, and THE BAND FLIP. Rung 69 changes a COORDINATE, so every rung-68 arm must
//          be reached bit-for-bit and the one-sided band must run the OTHER way — the failure
//          mode that raises nothing.
// =============================================================================================

/// `stator_inc = None` with a rung-68 `phi` stator armed: rung 68's own five-state cascade.
#[test]
fn reduce_no_incidence_stator_is_rung68_bit_for_bit() {
    let arm = LeverArm { bleed_lim: Some(valve(Some(TAU))),
                         stator_lim: Some(phi_stator(Some(TAU_S), V_MAX)),
                         ..Default::default() };
    let a = armed_march(&split_of(&arm));
    let b = armed_march(&three(&arm));
    assert_eq!(keys(&a), keys(&b));
    assert!(carries_v(&a[0]));
    assert_eq!(v_at_point(&a[0]), 0.0);
    assert!(a.iter().map(v_at_point).fold(f64::INFINITY, f64::min) < 0.0,
            "rung 68's band is the NEGATIVE one");
}

/// Rung 66's arm (no stator at all), rung 65's, rung 52's, rung 64's and rung 62's all leave
/// through the same parent, so a rung-69 machine with no stator **is** every ancestor.
///
/// **FIVE cases, which is Python's count and not four** — the fifth is rung 62's `BleedSchedule`
/// arm. A silently-shortened loop is this phase's *"a count typed instead of added up"*, so the
/// length is asserted rather than trusted to the reader.
#[test]
fn reduce_inherited_arms_bit_for_bit() {
    let cases: [(LeverArm, Option<Floor>, Option<AsymmetricLag>); 5] = [
        // rung 66 — both other clocks armed, no stator
        (LeverArm { bleed_lim: Some(valve(Some(TAU))), ..Default::default() },
         Some(fuel_floor()), Some(lag())),
        // rung 65 — no fuel lag
        (LeverArm { bleed_lim: Some(valve(Some(TAU))), ..Default::default() },
         Some(fuel_floor()), None),
        // rung 52 — no valve
        (LeverArm::default(), Some(fuel_floor()), Some(lag())),
        // rung 64 — an UNLAGGED valve and no fuel leg
        (LeverArm { bleed_lim: Some(valve(None)), ..Default::default() }, None, None),
        // rung 62 — the valve SCHEDULE
        (LeverArm { bleed_sched: Some(BleedSchedule::new(B, 0.65)), ..Default::default() },
         None, None),
    ];
    assert_eq!(cases.len(), 5, "Python's loop has five arms");
    for (i, (arm, surge, lg)) in cases.into_iter().enumerate() {
        let a = march(&split_of(&arm), DS, surge, lg, MarchScope::DEFAULT);
        let b = march(&three(&arm), DS, surge, lg, MarchScope::DEFAULT);
        assert_eq!(keys(&a), keys(&b), "case {i}");
        assert!(!carries_v(&a[0]), "case {i}");
    }
}

/// `M_i` is INCREASING in `v` where `phi_lp` DECREASES, so the admissible band is `[0, +v_max]` —
/// the MIRROR of rung 68's. Getting the orientation wrong returns a wrong regime label with
/// **nothing raising** (rung 62's `_powers` trap, fifth reload), so the band is gated from BOTH
/// sides.
#[test]
fn the_band_flips_and_an_out_of_band_start_is_refused() {
    let (m, traj) = split();
    assert!(traj.iter().map(v_at_point).all(|v| v >= 0.0),
            "the incidence loop CLOSES the stators");
    assert!(traj.iter().map(v_at_point).fold(f64::NEG_INFINITY, f64::max) > 0.0,
            "...and it actually moved");
    // rung 68's side of the band is outside this one.
    assert!(panics_with(|| {
        let (m, _) = split();
        march(&m, DS, Some(fuel_floor()), Some(lag()),
              MarchScope { v0: Some(-0.05), ..MarchScope::DEFAULT });
    }, "stator POSITION"));
    // ...and this one is not.
    let t = march(&m, DS, Some(fuel_floor()), Some(lag()),
                  MarchScope { v0: Some(0.05), ..MarchScope::DEFAULT });
    assert!(!t.is_empty());
}

#[test]
fn one_stator_one_reference() {
    assert!(panics_with(|| {
        split_of(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                             stator_lim: Some(phi_stator(Some(TAU_S), V_MAX)),
                             stator_inc: Some(inc(Some(TAU_S), V_MAX)),
                             ..Default::default() });
    }, "ONE reference"));
}

/// Across a change of coordinate, *"one set point"* can only mean **ONE PHYSICAL WALL**: the
/// incidence floor must BE the valve's `phi` floor at the design setting. An offset here would
/// confound the reference split with a set-point offset.
#[test]
fn one_physical_wall_is_enforced_not_one_float() {
    let l = inc(Some(TAU_S), V_MAX);
    assert!((l.m_lim - m_lim()).abs() <= 1e-15);
    assert!(approx(l.phi_lim_at(&lp()), PHI, 1e-15));
    assert!(panics_with(|| {
        split_of(&LeverArm {
            bleed_lim: Some(valve(Some(TAU))),
            stator_inc: Some(StatorIncidenceLimiter::new(m_lim() + 0.01, V_MAX, Some(TAU_S))),
            ..Default::default() });
    }, "ONE PHYSICAL WALL"));
}

#[test]
fn an_unlagged_incidence_stator_is_refused() {
    assert!(panics_with(|| { StatorIncidenceLimiter::new(m_lim(), V_MAX, Some(0.0)); },
                        "INSTANTANEOUS"));
    let m = split_of(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                 stator_inc: Some(inc(None, V_MAX)),
                                 ..Default::default() });
    assert!(!carries_v(&armed_march(&m)[0]),
            "an unlagged incidence stator must not enter the five-state integrator");
}

/// THE SEVENTH instance of the trap rungs 61–68 each hit — and the second in a row where the
/// signature GROWS, so *"silently swaps the REFERENCE"* joins *"silently drops the loop"*.
///
/// **Python's `type(s) is ReferenceSplitTransient` has no runtime counterpart here** (see the
/// module header). What replaces it is stronger than a type check and stronger than an address
/// comparison: the sibling is made to run rung 69's OWN `check_v0`, which refuses a `v0` rung 68
/// accepts. A sibling handed back carrying rung 68's table passes every float below and fails that.
#[test]
fn at_lever_keeps_the_reference() {
    let m = split_of(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                 stator_inc: Some(inc(Some(TAU_S), V_MAX)),
                                 ..Default::default() });
    let s = m.at_lever(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                   stator_inc: Some(inc(Some(TAU_S), V_MAX)),
                                   ..Default::default() });
    assert!(s.fuel.inner.stator.inc.is_some() && s.fuel.inner.stator.lim.is_none());
    assert!(armed_march(&s).iter().map(v_at_point).all(|v| v >= 0.0));
    assert!(panics_with(|| {
        let m = split_of(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                     stator_inc: Some(inc(Some(TAU_S), V_MAX)),
                                     ..Default::default() });
        let s = m.at_lever(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                       stator_inc: Some(inc(Some(TAU_S), V_MAX)),
                                       ..Default::default() });
        march(&s, DS, Some(fuel_floor()), Some(lag()),
              MarchScope { v0: Some(-0.05), ..MarchScope::DEFAULT });
    }, "stator POSITION"),
            "the sibling must carry rung 69's OWN band, which is what `type(s) is \
             ReferenceSplitTransient` asserts in Python");
}

/// Rung 68's trap, mirrored: `v > 0` is TRUE for a saturated incidence stator and for a riding one
/// alike, so a reader that inferred the regime from the float would admit both.
///
/// Python closes with `{p["v_regime"] for p in t} <= {"dormant", "riding", "saturated"}`, which
/// here is discharged by [`Regime`] being a three-variant enum — restating it would be a gate that
/// cannot fail.
#[test]
fn a_float_comparison_against_the_stop_is_not_the_regime() {
    let m = split_of(&LeverArm { stator_inc: Some(inc(Some(TAU_S), 0.02)),
                                 ..Default::default() });
    let t = march(&m, DS, None, None, MarchScope::DEFAULT);
    let sat: Vec<&FuelPoint> = t.iter().filter(|p| v_regime_of(p) == Regime::Saturated).collect();
    let rid: Vec<&FuelPoint> = t.iter().filter(|p| v_regime_of(p) == Regime::Riding).collect();
    assert!(!sat.is_empty() && !rid.is_empty());
    assert!(sat.iter().all(|p| v_at_point(p) > 0.0));
    assert!(rid.iter().all(|p| v_at_point(p) > 0.0));
}

// =============================================================================================
// GATE 2 — THE PAIRWISE SPLIT. Which pairs keep rung 66's identity reads off WHICH LOOPS SHARE A
//          CONSTRAINT.
// =============================================================================================

/// `pair_RC` — fuel and valve, still both on `phi` — must stay at 1 to the root-finders' floor
/// under BOTH references, while `pair_RV` and `pair_CV` move to `k` under the split only. That
/// contrast at ONE base point on ONE trajectory is the whole measurement.
#[test]
fn the_shared_pair_survives_and_the_split_pairs_do_not() {
    let (m, _) = split();
    let g = reference_gains(&m, &flight(), &ramp(DS), SM, &rig_arm(), 10);
    assert!(g.n_riding >= 40, "{}", g.n_riding);
    assert!(g.rows.len() >= 6 && g.skipped.is_empty());
    assert!(g.worst_rc_inc.expect("live rows") < 1e-8, "{:?}", g.worst_rc_inc);
    assert!(g.worst_rc_phi.expect("live rows") < 1e-8, "{:?}", g.worst_rc_phi);
    for x in &g.rows {
        let (i, p) = (&x.inc, &x.phi);
        // the rung-68 reference at the SAME point: every pair at 1, cyclic at −1
        for (k, v) in [("pair_RC", p.pair_rc), ("pair_RV", p.pair_rv), ("pair_CV", p.pair_cv)] {
            assert!((v - 1.0).abs() < 1e-6, "{} {k} {v}", x.s);
        }
        assert!((p.cyclic + 1.0).abs() < 1e-6);
        // the split: RC survives, RV and CV do not, and the cyclic product FLIPS SIGN
        assert!((i.pair_rv - x.k).abs() < 0.01 * x.k.abs());
        assert!((i.pair_cv - x.k).abs() < 0.01 * x.k.abs());
        assert!(i.pair_rv < -1.5 && i.pair_cv < -1.5, "{} {}", x.s, i.pair_rv);
        assert!(i.cyclic > 1.5, "{} {}", x.s, i.cyclic);
    }
}

/// `pair_RV == pair_CV` is **NOT general to a split** — it holds iff the odd constraint depends on
/// `(g, q)` ONLY THROUGH the shared one, which `M_i = T_c - 1/phi + v` does. So this equality
/// measures that the two walls differ by exactly the LEVER'S OWN direct channel, and it is what
/// gives `k` its closed form.
#[test]
fn the_two_split_pairs_take_the_same_value_and_that_is_a_measurement() {
    let (m, _) = split();
    let g = reference_gains(&m, &flight(), &ramp(DS), SM, &rig_arm(), 10);
    assert!(g.worst_pair_gap.expect("live rows") < 0.01, "{:?}", g.worst_pair_gap);
    let (lo, hi) = (g.k_range.0.expect("live rows"), g.k_range.1.expect("live rows"));
    assert!(-2.1 < lo && lo < hi && hi < -1.5, "{:?}", g.k_range);
}

/// There is no point where all three constraints hold (`phi = phi_lim` and `M_i = m_lim` force
/// `v = 0`, the dormant stop), so the base MUST be the SHARED constraint's manifold — rung 68's
/// instrument unchanged. Read at the STATOR's own root instead, `pair_RC` degrades by orders.
/// **Reported, never gated on, and this gate is what keeps that honest.**
#[test]
fn the_evaluation_manifold_is_forced_and_the_alternative_is_reported() {
    let (m, _) = split();
    let g = reference_gains(&m, &flight(), &ramp(DS), SM, &rig_arm(), 10);
    let own = g.worst_rc_own.expect("the own-root arm has live rows");
    assert!(own > 1e-3, "{own}");
    assert!(own > 1e5 * g.worst_rc_inc.expect("live rows"));
}

// =============================================================================================
// GATE 3 — ZEROS = n − m. ONE zero here, TWO under rung 68's reference, on the same rig and the
//          same clock grid.
// =============================================================================================

/// Python's default clock grid, written in the `(tau_v, tau_att, tau_s)` order the reader takes.
const CLOCKS: [(f64, f64, f64); 4] =
    [(0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05), (0.02, 0.05, 0.10)];

/// **THE GRID IS WRITTEN IN ONE ORDER AND KEYED IN ANOTHER.** `reference_modes` takes its grid as
/// `(tau_v, tau_att, tau_s)` and reports `taus = (tau_att, tau_v, tau_s)` — the STATE VECTOR's
/// `(g, q, v)` order — so Python's grid entry `(0.05, 0.005, 0.05)` is keyed as
/// `(0.005, 0.05, 0.05)`. Three of the four entries are asymmetric in the first two slots, so a
/// lookup written against the grid would silently return the wrong arm. This asserts the match was
/// FOUND instead of unwrapping into one.
fn arm_at(r: &ReferenceModes, taus: (f64, f64, f64)) -> &RefModesClock {
    r.arms.iter().find(|a| a.taus == taus)
        .unwrap_or_else(|| panic!("no arm at taus = {taus:?}; the grid is keyed in the STATE \
                                   vector's (g, q, v) order, not the grid's own"))
}

#[test]
fn the_rank_is_the_constraint_count_not_the_loop_count() {
    let (m, _) = split();
    let r = reference_modes(&m, &flight(), &ramp(0.002), SM, &CLOCKS, V_MAX, 3.0, 20);
    assert_eq!(r.arms.len(), 4);
    for a in &r.arms {
        let (i, p) = (&a.inc, &a.phi);
        assert!(!i.rows.is_empty() && !p.rows.is_empty(), "{:?}", a.taus);
        assert!(i.skipped <= 2 && p.skipped <= 2, "{:?}", a.taus);
        assert_eq!(i.zeros, vec![1], "{:?} {:?}", a.taus, i.zeros);       // n − m = 3 − 2
        assert_eq!(p.zeros, vec![2], "{:?} {:?}", a.taus, p.zeros);       // n − m = 3 − 1
        for x in &p.rows {
            assert!(approx(x.zeta.expect("a non-zero dominant root"), 1.0, 1e-6),
                    "rung 68's spectrum is REAL");
        }
    }
}

/// A reader that inherited rung 68's determinant test would report rank one and see **NOTHING**.
/// Both invariants are read against the rate sum's own power, because "zero" without its scale is
/// not a measurement.
#[test]
fn det_is_blind_to_the_split_and_c1_is_the_discriminator() {
    let (m, _) = split();
    let r = reference_modes(&m, &flight(), &ramp(0.002), SM, &CLOCKS, V_MAX, 3.0, 20);
    let c0 = |x: &RefModesArm| x.max_c0_rel.expect("live rows");
    let c1 = |x: &RefModesArm| x.min_c1_rel.expect("live rows");
    for a in &r.arms {
        let (i, p) = (&a.inc, &a.phi);
        assert!(c0(i) < 1e-8, "{:?} {}", a.taus, c0(i));
        assert!(c0(p) < 1e-8, "{:?} {}", a.taus, c0(p));
        assert!(c1(i) > 0.1, "{:?} {}", a.taus, c1(i));
        assert!(c1(p) < 1e-10, "{:?} {}", a.taus, c1(p));
        assert!(c1(i) / c1(p).max(1e-300) > 1e9);
    }
}

/// **THE GATE THAT KEEPS THE ONE ABOVE FROM BEING A COINCIDENCE.** Hand-build the two blocks the
/// algebra predicts and check `det == 0` in both while the RANK is 2 — so `det = 0` carries no
/// information about the third row at all. Rung 68's own tautology-killer, one level up: there the
/// danger was a measurement implied by the pairwise identities, here it is one implied by nothing.
///
/// Block A: a GENERIC second constraint (`pair_RV != pair_CV`).
/// Block B: this plant's, where the odd constraint depends on `(g, q)` only through `phi` — then
/// and only then do the two split pairs coincide at `k`.
#[test]
fn a_determinant_provably_cannot_see_a_split() {
    fn block(phi_g: f64, phi_q: f64, phi_v: f64, psi_g: f64, psi_q: f64, psi_v: f64)
             -> [[f64; 3]; 3] {
        [[-1.0, -phi_q / phi_g, -phi_v / phi_g],
         [-phi_g / phi_q, -1.0, -phi_v / phi_q],
         [-psi_g / psi_v, -psi_q / psi_v, -1.0]]
    }
    fn det(m: &[[f64; 3]; 3]) -> f64 {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }
    /// The three 2×2 minors of the 2×3 `[r; t]` — ALL of them zero iff the rows are parallel.
    ///
    /// **Checking ONE would fail here for a reason that is itself the finding**: in this plant
    /// rows 0 and 2 AGREE in the `(g, q)` columns up to scale (`psi` depends on them only through
    /// `phi`) and differ ONLY in the lever's own `v` column.
    fn minors(r: &[f64; 3], t: &[f64; 3]) -> [f64; 3] {
        [(r[0] * t[1] - r[1] * t[0]).abs(),
         (r[0] * t[2] - r[2] * t[0]).abs(),
         (r[1] * t[2] - r[2] * t[1]).abs()]
    }
    let (pg, pq, pv, phi) = (2.0f64, 3.0f64, 5.0f64, 0.8f64);
    let a = block(pg, pq, pv, 1.0, 7.0, 1.0);                             // generic psi
    let b = block(pg, pq, pv, pg / (phi * phi), pq / (phi * phi), pv / (phi * phi) + 1.0);
    let mx = |v: [f64; 3]| v.into_iter().fold(f64::NEG_INFINITY, f64::max);
    for (m, name) in [(&a, "generic"), (&b, "this plant")] {
        assert!(det(m).abs() < 1e-12, "{name}");
        // rank 2: rows 0 and 1 exactly parallel, row 2 NOT in their span
        assert!(mx(minors(&m[0], &m[1])) < 1e-12, "{name}");
        assert!(mx(minors(&m[0], &m[2])) > 1e-6, "{name}");
    }
    assert!(minors(&b[0], &b[2])[0] < 1e-12, "the split is carried by the `v` column alone");
    assert!(minors(&a[0], &a[2])[0] > 1e-6, "...which is a property of THIS psi, not of a split");
    let k = (pv / (phi * phi)) / (pv / (phi * phi) + 1.0);
    assert!(approx(b[0][2] * b[2][0], k, 1e-6));                          // pair_RV
    assert!(approx(b[1][2] * b[2][1], k, 1e-6));                          // pair_CV — the SAME
    assert!(approx(b[0][1] * b[1][2] * b[2][0], -k, 1e-6));               // cyclic
    assert!(!approx(a[0][2] * a[2][0], a[1][2] * a[2][1], 1e-6), "generic: NOT the same");
}

// =============================================================================================
// GATE 4 — THE DAMPING FLOOR. One scalar `k` sets the ring, and no bandwidth can beat it.
// =============================================================================================

/// Python's default six-point grid, in the reader's `(tau_v, tau_att, tau_s)` order.
const DAMP_GRID: [(f64, f64, f64); 6] =
    [(0.05, 0.05, 0.05), (0.05, 0.05, 0.025), (0.05, 0.05, 0.10),
     (0.10, 0.10, 0.05), (0.02, 0.20, 0.05), (0.20, 0.02, 0.05)];

/// `zeta = (A+z)/(2 sqrt(A z (1-k))) >= 1/sqrt(1-k)` by AM-GM, with equality at
/// `A = 1/tau_g + 1/tau_q == 1/tau_s = z`.
#[test]
fn the_damping_floor_is_bandwidth_independent_and_binds_at_a_equals_z() {
    let (m, _) = split();
    let d = damping_floor(&m, &flight(), &ramp(DS), SM, &DAMP_GRID, V_MAX, 3.0);
    let live: Vec<DampingLive> = d.rows.iter().filter_map(|x| x.live).collect();
    assert_eq!(live.len(), d.rows.len(), "every grid point must be live");
    assert!(d.rows.len() >= 6);
    assert!(d.holds, "{:?}", live);
    assert!(d.worst_pred_err.expect("live rows") < 1e-3, "{:?}", d.worst_pred_err);
    assert!(live.iter().all(|x| x.complex_pair), "k < 0 => the pair RINGS on this grid");
    let at1: Vec<&DampingLive> =
        live.iter().filter(|x| (x.a_over_z - 1.0).abs() < 1e-12).collect();
    assert_eq!(at1.len(), 2);
    for x in &at1 {
        assert!(approx(x.zeta, x.floor, 1e-9), "the floor is REACHED: {x:?}");
    }
    for x in &live {
        if x.a_over_z > 1.5 {
            assert!(x.zeta > 1.02 * x.floor, "{x:?}");
        }
    }
    // BANDWIDTH-INDEPENDENT: the two `A/z == 1` arms differ 2x in every clock, same zeta.
    assert!(approx(at1[0].zeta, at1[1].zeta, 0.02));
    assert!(!approx(at1[0].rate_sum, at1[1].rate_sum, 0.1));
}

/// The window is REAL and has EDGES: `zeta < 1` needs `(A+z)^2 < 4 A z (1-k)`, so a clock ratio far
/// from 1 puts the pair back on the axis. Rung 68's own clock grid contains such an arm
/// (`tau_g = 0.005` ⇒ `A/z = 11`), which is what makes *"complex"* a measurement.
#[test]
fn a_slow_enough_stator_takes_the_pair_back_onto_the_real_axis() {
    let (m, _) = split();
    let r = reference_modes(&m, &flight(), &ramp(0.002), SM, &CLOCKS, V_MAX, 3.0, 20);
    assert_eq!(arm_at(&r, (0.05, 0.05, 0.05)).inc.all_complex, Some(true));
    assert_eq!(arm_at(&r, (0.005, 0.05, 0.05)).inc.all_complex, Some(false),
               "A/z = 11 is outside the window");
    assert_eq!(arm_at(&r, (0.005, 0.05, 0.05)).inc.zeros, vec![1],
               "...but the RANK does not care");
}

// =============================================================================================
// GATE 5 — THE LEDGER. The whole sign table flips with the reference.
// =============================================================================================

/// `bare`, `F`, `V` and `FV` carry no stator, so they CANNOT differ — a free check that the two
/// ledgers come from one rig and are differenceable (rung 63's lesson).
#[test]
fn the_stator_free_cells_are_identical_between_the_references() {
    let (m, _) = split();
    let b = reference_bill(&m, &flight(), &ramp(DS), SM, &rig_arm());
    assert_eq!(b.common_max_rel, 0.0, "{:?}", b.common);
}

/// RUNG 53's *a margin is a DISTANCE*, one level up. Rung 68 showed a credit is meaningless without
/// its WALL. Here the same lever on the same plant, against the same two walls, is protective or
/// harmful according to which wall its LOOP watches — so a credit needs its loop's REFERENCE named
/// too.
#[test]
fn the_credit_sign_table_flips_with_the_reference() {
    let (m, _) = split();
    let b = reference_bill(&m, &flight(), &ramp(DS), SM, &rig_arm());
    let (p, i) = (b.stator_credit_phi, b.stator_credit_inc);
    // phi-referenced (rung 68): protective in phi, HARMFUL in incidence
    assert!(p.alone > 80.0 && p.alone_inc < -40.0, "{p:?}");
    assert!(p.marginal > 0.0 && p.marginal_inc < 0.0, "{p:?}");
    // incidence-referenced (rung 69): the MIRROR, in every one of the four cells
    assert!(i.alone < -80.0 && i.alone_inc > 50.0, "{i:?}");
    assert!(i.marginal < 0.0 && i.marginal_inc > 0.0, "{i:?}");
    // and the triple delivers on the wall its third loop watches, in both
    assert!(b.delivered.1 > b.delivered.0, "{:?}", b.delivered);
    assert!(b.delivered_inc.0 > b.delivered_inc.1, "{:?}", b.delivered_inc);
    assert!(b.delivered_inc.0 > 99.0, "{:?}", b.delivered_inc);
}

/// The sharpest single number in the ledger: closing the stators LOWERS `phi`, so a loop that
/// protects incidence drives the flow coefficient BELOW the bare march's own minimum.
#[test]
fn the_incidence_stator_alone_is_worse_than_no_limiter_at_all_in_phi() {
    let (m, _) = split();
    let b = reference_bill(&m, &flight(), &ramp(DS), SM, &rig_arm());
    let (s, bare) = (b.inc.cell("S"), b.inc.cell("bare"));
    assert!(s.min_phi < bare.min_phi);
    assert!(s.credit < -100.0, "{}", s.credit);
    // the band is MIRRORED — `v_min` alone would read 0.0 for a loop that rode the whole ramp
    assert!(s.v_max_used > 0.0 && s.v_min < 1e-9);
    assert!(s.v_saturated, "and it is authority-limited — anchor s 0.2");
}

/// RUNG 65 PUBLISHED A RETRACTION for an RK4 artifact that read as a physical finding, and rung 68
/// published a `ds` table because of it. **That table is NOT inherited here**: this plant's
/// dominant root is a lightly-damped COMPLEX pair, a different aliasing character from rung 68's
/// real one. So the cells that carry the sign table are re-run at half the step — including the
/// smallest number in it, the incidence loop's own `phi` marginal, whose SIGN is the delicate one.
///
/// **AND IT IS THE ONE GATE THAT CALLS `triple_rig` WITHOUT A REFERENCE SCOPE**, so it runs on
/// whatever the machine is armed with. That makes it the only place a broken `_ref` FALLBACK — the
/// `"phi" if stator_lim is not None else "inc"` arm — would show up as a value difference, which
/// § 5.26.1 (j) measured no ledger key can see.
#[test]
fn the_sign_table_is_grid_converged() {
    let (m, _) = split();
    let t_c = lp().tan_beta1_crit();
    let cell = |ds: f64, fuel: bool, valve_on: bool, stator: bool| -> (f64, f64) {
        let (rig, surge, lg) = m.triple_rig(&TripleRigArm {
            sm: SM, tau: TAU, tau_s: TAU_S, v_max: V_MAX, tau_att: TAU_ATT, tau_rel: TAU_REL,
            fuel, valve: valve_on, stator });
        let t = march(&rig, ds, surge, lg, MarchScope::DEFAULT);
        (violation(&t, PHI, R), violation_inc(&t, m_lim(), t_c, R))
    };
    /// `bare, FV, S, FVS` — Python's own four, in Python's order. The indices below read off it.
    const NAMES: [(&str, bool, bool, bool); 4] = [("bare", false, false, false),
                                                  ("FV", true, true, false),
                                                  ("S", false, false, true),
                                                  ("FVS", true, true, true)];
    let run = |ds: f64| NAMES.map(|(_, f, v, s)| cell(ds, f, v, s));
    let (coarse, fine) = (run(DS), run(DS / 2.0));
    for (n, (c, f)) in NAMES.iter().zip(coarse.iter().zip(fine.iter())) {
        assert!(approx(f.0, c.0, 5e-3), "{} phi {c:?} {f:?}", n.0);
        assert!(approx(f.1, c.1, 5e-3), "{} inc {c:?} {f:?}", n.0);
    }
    let credits = |o: &[(f64, f64); 4]| -> [f64; 4] {
        // `i = 0` is the `phi` currency, `i = 1` the INCIDENCE one; `bare` is index 0.
        let cr = |c: usize, i: usize| {
            let (num, den) = if i == 0 { (o[c].0, o[0].0) } else { (o[c].1, o[0].1) };
            100.0 * (1.0 - num / den)
        };
        [cr(2, 0), cr(2, 1), cr(3, 0) - cr(1, 0), cr(3, 1) - cr(1, 1)]
    };
    let (a, b) = (credits(&coarse), credits(&fine));
    for (x, y) in a.iter().zip(b.iter()) {
        assert!(x * y > 0.0, "a SIGN must not depend on the grid: {x} {y}");
        assert!(approx(*y, *x, 5e-3), "{x} {y}");
    }
    assert!(a[0] < -100.0 && a[1] > 50.0 && a[2] < 0.0 && a[3] > 0.0, "{a:?}");
}

// =============================================================================================
// GATE 6 — AUTHORITY. Rung 64's ceiling, and the SIGN the split gives it.
// =============================================================================================

/// Monotone to the SOLVER's resolution, not to the last bit.
///
/// **The `phi` arm PLATEAUS** — it stops saturating at `v_max = 0.20`, so 0.20 and 0.40 agree to
/// 1e-14 and their float order is noise. That plateau is itself the finding (rung 64's ceiling,
/// located) and asserting a strict order on it would be asserting the noise.
fn monotone(seq: &[f64], down: bool) -> bool {
    seq.windows(2).all(|w| {
        (w[1] - w[0]) * (if down { -1.0 } else { 1.0 }) >= -1e-9 * w[0].abs().max(w[1].abs())
    })
}

/// RUNG 64: *a limiter's LAW cannot buy PROTECTION, only its PRICE — the ceiling is the lever's
/// AUTHORITY*. Rung 68 EXTENDED that (inert in company, binding alone). **Rung 69 gives it a
/// SIGN**: alone, more authority monotonically improves the wall the loop WATCHES and monotonically
/// degrades the other — under BOTH references, in mirror image.
#[test]
fn authority_is_inert_in_company_and_buys_only_the_watched_wall() {
    let t_c = lp().tan_beta1_crit();
    let run = |v_max: f64, reference: &str, company: bool| -> (f64, f64, bool) {
        let arm = LeverArm {
            bleed_lim: if company { Some(valve(Some(TAU))) } else { None },
            stator_inc: if reference == "inc" { Some(inc(Some(TAU_S), v_max)) } else { None },
            stator_lim: if reference == "inc" { None }
                        else { Some(phi_stator(Some(TAU_S), v_max)) },
            ..Default::default()
        };
        let m = split_of(&arm);
        let t = march(&m, DS, if company { Some(fuel_floor()) } else { None },
                      if company { Some(lag()) } else { None }, MarchScope::DEFAULT);
        (violation(&t, PHI, R), violation_inc(&t, m_lim(), t_c, R),
         t.iter().any(|p| v_regime_of(p) == Regime::Saturated))
    };
    const VS: [f64; 4] = [0.05, 0.10, 0.20, 0.40];
    let trip = VS.map(|v| run(v, "inc", true));
    for x in &trip {
        assert!(approx(x.0, trip[2].0, 1e-11), "{} vs {}", x.0, trip[2].0);
    }
    assert!(!trip.iter().any(|x| x.2), "never reaches the stop in company");

    let a_i = VS.map(|v| run(v, "inc", false));
    let a_p = VS.map(|v| run(v, "phi", false));
    let phis = |o: &[(f64, f64, bool); 4]| o.map(|x| x.0);
    let incs = |o: &[(f64, f64, bool); 4]| o.map(|x| x.1);
    //          watched wall: MONOTONE BETTER                other wall: MONOTONE WORSE
    assert!(monotone(&incs(&a_i), true), "{:?}", incs(&a_i));
    assert!(monotone(&phis(&a_i), false), "{:?}", phis(&a_i));
    assert!(monotone(&phis(&a_p), true), "{:?}", phis(&a_p));
    assert!(monotone(&incs(&a_p), false), "{:?}", incs(&a_p));
    // ...and the effect is DECISIVE, both ways
    assert!(a_i[0].1 / a_i[3].1 > 5.0, "{} {}", a_i[0].1, a_i[3].1);
    assert!(a_p[0].0 / a_p[3].0 > 5.0, "{} {}", a_p[0].0, a_p[3].0);
    // WHERE EACH LEVER RUNS OUT: rung 68's is done at 0.20, this one is still starved there
    assert!(approx(a_p[2].0, a_p[3].0, 1e-12));
    assert!(a_i[2].1 > 2.0 * a_i[3].1);
}

// =============================================================================================
// GATE 7 — THE DISPLACEMENT. A degenerate plant cannot even be displaced.
// =============================================================================================

/// Rung 68's `s = 0` fixed points are a FAMILY, so displacing the stator's initial position just
/// selects another member: the other two loops take it up EXACTLY and no tracking error survives.
/// Under the split they cannot, and a fifth of the displacement survives as an error that then
/// swings back. That is the rank difference showing up in the TRAJECTORY, not in a Jacobian — and
/// it is why the ring is not separably observable without one.
#[test]
fn a_shared_constraint_absorbs_a_displaced_start_and_a_split_one_cannot() {
    let (m, _) = split();
    let r = ring_visibility(&m, &flight(), &ramp(0.002), SM, &rig_arm(), 0.05);
    assert!(r.phi.displaced.survives.expect("a displacement was applied") < 1e-10,
            "{:?}", r.phi.displaced);
    assert!(r.inc.displaced.survives.expect("a displacement was applied") > 0.1,
            "{:?}", r.inc.displaced);
    assert!(r.inc.displaced.crossings >= 1);
    // HONEST LIMIT: the ramp's own forcing reverses the error in the UNDISPLACED run too, so a
    // crossing count cannot separate the mode from the forcing. Reported, not claimed.
    assert!(r.inc.base.crossings >= 1);
}

// =============================================================================================
// GATE 8 — THE RK4 FLOOR: the CONSTANT survives, its REASON does not.
// =============================================================================================

/// **THE ONE SWAP NO VALUE KEY CAN SEE.** § 5.26 (ii) measured 0 value disagreements in 77 calls —
/// the condition is `ds * rate <= 2.0` in both rungs, character for character, and the entire
/// difference is the assertion MESSAGE. So this is a PANIC-STRING gate and never a value diff;
/// writing it as one is exactly how the cell would end up silently ungated.
///
/// It is spelled with `panics_with` rather than `#[should_panic(expected = "rank TWO")]` because
/// Python's gate has TWO halves and the attribute form cannot carry the first: rung 66's own
/// two-clock constant ADMITS `ds = 0.04`, so the step refused below is one the parent's REASON
/// would have allowed.
#[test]
fn the_floor_still_fires_and_its_message_names_the_new_reason() {
    let ds = 0.04;
    assert!(ds * (1.0 / TAU + 1.0 / TAU_ATT.min(TAU_REL)) <= 2.0,
            "rung 66's two-clock constant must ADMIT this step, or the gate below is refusing \
             something the parent refuses too");
    assert!(panics_with(|| {
        let (m, _) = split();
        march(&m, ds, Some(fuel_floor()), Some(lag()), MarchScope::DEFAULT);
    }, "rank TWO"));
}

/// Rung 68's constant survives on a DIFFERENT argument: the dominant root is no longer
/// `-sum 1/tau` but a complex pair of modulus `sqrt(A z (1-k))`, bounded by `sqrt(1-k)/2` times
/// that sum. **An assert nobody has checked against the plant is a tautology** (rung 67 gate 9),
/// and rung 65 published a retraction for exactly a trusted stability argument.
#[test]
fn the_inherited_constant_is_conservative_and_that_is_measured() {
    let (m, _) = split();
    let g = rk4_margin(&m, &flight(), &ramp(DS), SM, &rig_arm(), 10);
    assert!(g.n >= 5, "{}", g.n);
    let (mo, ra, bo) = (g.max_mod.expect("live rows"), g.max_ratio.expect("live rows"),
                        g.max_bound.expect("live rows"));
    assert!(mo < g.rate_sum, "{mo} {}", g.rate_sum);
    assert!(ra < bo + 1e-9, "{ra} {bo}");
    assert!(bo < 1.0, "the bound only holds while k >= -3");
    assert!(ra > 0.7, "...and it is not a slack bound either");
}

// =============================================================================================
// GATE 9 — THE INITIAL CONDITION. Removing a zero eigenvalue makes the plant MORE sensitive to a
//          moved start, not less: redundancy ABSORBS.
// =============================================================================================

/// **PRE-REGISTERED THE OTHER WAY AND MISSED (anchor P9), and the miss is the content.**
///
/// Rung 68 measured its `s = 0` start-spread at 45.2 % / 105.5 % and DECLINED to attribute the
/// growth over rung 66's 84 % to its second zero eigenvalue. This rung supplies the
/// counter-example: dropping the nullity from 2 to 1 makes both spreads GROW again. So the zero
/// count and the IC sensitivity move in OPPOSITE directions, and **a null space is a SHOCK
/// ABSORBER** — the redundant loops redistribute a moved start among themselves. GATE 7 is the same
/// mechanism read at a single point.
#[test]
fn a_smaller_null_space_is_a_larger_ic_sensitivity() {
    let (m, _) = split();
    let f = ic_family(&m, &flight(), &ramp(DS), SM, &rig_arm(),
                      &["gqv", "gvq", "qgv", "qvg", "vgq", "vqg"],
                      &[None, Some(0.0), Some(0.02), Some(0.06)]);
    assert_eq!(f.order_members, 1, "the order is still NOT the lever from the declared start");
    assert!(f.by_order.iter().all(|(_, x)| x.iters == 1));
    let si = f.start_spread_i.expect("a live spread");
    let sw = f.start_spread_withheld.expect("a live spread");
    assert!(si > 0.452, "{si}");            // rung 68's own number
    assert!(sw > 1.055, "{sw}");
    assert!(si > 1.5 && sw > 2.5, "{si} {sw}");
    // the DECLARED member is still rung 66's, and the stator still opens at its dormant stop
    let z = &f.by_order.iter().find(|(o, _)| *o == "gqv").expect("the declared order").1;
    assert_eq!(z.g0, 0.0);
    assert_eq!(z.v0, 0.0);
    assert!((z.b0 - 0.036626).abs() < 1e-5, "{}", z.b0);
}
