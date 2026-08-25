//! RUNG 58 — **THE COMPOSITE MIN-SELECT: two levers DO NOT SUPERPOSE.**
//!
//! `tests/test_rung58.py` ported one-to-one: **15 Python `def test_` → 15 collected → 15
//! `#[test]` here** (no `parametrize` in this file).
//!
//! # THIS SUITE'S GRID IS NOT RUNG 57's, AND IT IS NOT THE READERS' EITHER
//!
//! `N_LO` is **0.7557** where `rung57.rs` writes 0.75574 — four digits against five, in a
//! constant that sets where the schedule's knee sits. [`KEYS`] here is **eleven** names against
//! rung 57's nine (`mf_sched` and `s` join).
//!
//! **And `ds` is the trap the shipped source points the wrong way on.** Rung 58's reader methods
//! default to `ds = 0.005` — that is what [`Ramp::fine`]'s doc comment records, correctly, about
//! the SIGNATURES. `test_rung58.py` does not use it: it declares `DS = 0.01` and passes it
//! EXPLICITLY at every call site, so the suite marches on rung 57's step, not on its own
//! methods'. Porting through [`Ramp::fine`] because its doc names rung 58 would halve the step
//! and move every number below. Measured off the file, not read off the docstring.
//!
//! # WHAT THESE 15 GATES DO NOT ESTABLISH
//!
//! They are **relational** — every one asserts a relation among values this crate computed, so a
//! Rust/Python arithmetic divergence moves both sides and leaves all 15 green. § 5.20 (ii)'s own
//! headline one level up. Step 4's oracle is the instrument for agreement with Python.
//!
//! [`Ramp::fine`]: turbojet::stator_transient::Ramp::fine

use std::panic::catch_unwind;
use std::sync::OnceLock;

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::fuel_transient::{AccelSchedule, Floor, FuelPoint, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    CompositeCredit, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg,
    StatorSchedule,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const V: f64 = 0.20;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
/// **0.01, spelled by the SUITE** — the reader methods' own default is 0.005. See the header.
const DS: f64 = 0.01;
const SETTLE: f64 = 1.2;
/// **0.7557** — rung 58's four-digit spelling. `rung57.rs` writes 0.75574.
const N_LO: f64 = 0.7557;
/// Rung 48's schedule margin: engages at `s ~ 0.123`, accel completes.
const MARGIN: f64 = 0.25;
/// Rung 48's derived-table row count, Python's `n = 13` default.
const N_SCHED: usize = 13;
/// ELEVEN keys — rung 57's nine plus `mf_sched` and `s`.
const KEYS: [&str; 11] = ["nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "mf", "mf_sched",
                          "pi_lpc", "pi_hpc", "s"];

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

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

fn st(arm: StatorArm) -> ScheduledStatorCore {
    match ScheduledStatorTransient::new(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn sched_at(n_lo: f64) -> StatorSchedule {
    StatorSchedule::new(V, n_lo)
}

fn sched() -> StatorSchedule {
    sched_at(N_LO)
}

fn ramp(r: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds: DS }
}

fn accel() -> AccelSchedule {
    st(StatorArm::default()).fuel.accel_schedule(&flight(), LO, HI, MARGIN, N_SCHED)
}

fn pt(p: &FuelPoint, k: &str) -> f64 {
    match k {
        "nu_lp" => p.nu_lp,
        "nu_hp" => p.nu_hp,
        "phi_lp" => p.phi_lp,
        "phi_hp" => p.phi_hp,
        "Tt4" => p.tt4,
        "f" => p.f,
        "mf" => p.mf,
        "mf_sched" => p.mf_sched,
        "pi_lpc" => p.pi_lpc,
        "pi_hpc" => p.pi_hpc,
        "s" => p.s,
        _ => unreachable!("{k}"),
    }
}

/// Python's `_same` — **with the length assertion ahead of the zip**, which Python has and Rust
/// would otherwise lose silently (both languages truncate).
fn same(a: &[FuelPoint], b: &[FuelPoint]) {
    assert_eq!(a.len(), b.len(), "the two marches must land on the same grid");
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        for k in KEYS {
            assert_eq!(pt(x, k).to_bits(), pt(y, k).to_bits(),
                       "{k} at row {i} (s = {}): {} vs {}", x.s, pt(x, k), pt(y, k));
        }
    }
}

fn panic_text(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default()
}

/// Python's `pytest.raises(AssertionError, match=...)` — the `match=` half is why the substring
/// is required rather than a bare "it panicked".
fn refuses(what: &str, f: impl FnOnce() + std::panic::UnwindSafe) {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(f);
    std::panic::set_hook(prev);
    match out {
        Ok(()) => panic!("expected a refusal naming {what:?}, but the call SUCCEEDED"),
        Err(e) => {
            let msg = panic_text(e);
            assert!(msg.contains(what), "panicked, but not on {what:?}: {msg}");
        }
    }
}

// =============================================================================================
// THE REDUCE — rung 58 off is rung 57, bit-for-bit  (NEVER slow-tagged)
// =============================================================================================

/// Python `test_reduce_no_fuel_leg_is_bit_for_bit_rung57`.
#[test]
fn test_reduce_no_fuel_leg_is_bit_for_bit_rung57() {
    let f = flight();
    let none = StatorLeg::default();
    for arm in [StatorArm::default(), StatorArm::scheduled_lp(sched()),
                StatorArm::constant(V, 0.0)] {
        let m = st(arm);
        let (mine, nu0) = m.stator_march(&f, &ramp(0.5), None, &none);
        let (a, b) = (m.fuel.fuel_for_tt4(&f, LO), m.fuel.fuel_for_tt4(&f, HI));
        let s = move |x: f64| a + (b - a) * (x / 0.5).max(0.0).min(1.0);
        let raw = m.fuel.integrate_fuel(&f, s, nu0, 0.5 + SETTLE, DS, &Default::default());
        same(&mine, &raw);
    }
}

/// Python `test_reduce_dormant_leg_is_bit_for_bit_unarmed`. The STRONG reduce: a leg ARMED but
/// never binding must leave the march bit-identical to no leg at all.
#[test]
fn test_reduce_dormant_leg_is_bit_for_bit_unarmed() {
    let f = flight();
    let rp = ramp(0.5);
    let m = st(StatorArm::scheduled_lp(sched()));
    let (base, _) = m.stator_march(&f, &rp, None, &StatorLeg::default());
    let dorm_a = m.fuel.accel_schedule(&f, LO, HI, 0.60, N_SCHED);
    let (with_a, _) =
        m.stator_march(&f, &rp, None, &StatorLeg { accel: Some(&dorm_a), ..Default::default() });
    same(&with_a, &base);
    let (with_s, _) = m.stator_march(&f, &rp, None, &StatorLeg {
        surge: Some(Floor::Phi(SurgeLimiter::new(Spool::Lp, 0.50))), ..Default::default() });
    same(&with_s, &base);
}

/// Python `test_reduce_rung57_readers_untouched` — gated against rung 57's own published
/// constant-`v` erosion band.
#[test]
fn test_reduce_rung57_readers_untouched() {
    let c = st(StatorArm::constant(V, 0.0)).stator_credit(&flight(), &ramp(0.5), Spool::Lp);
    assert!(c.pointwise_exact);
    assert!((c.credit_pointwise - V).abs() < 1e-12, "{}", c.credit_pointwise);
    assert!(0.60 < c.erosion && c.erosion < 0.70, "rung 57's published band: {}", c.erosion);
}

/// Python `test_cycle_untouched_by_rung58_bit_for_bit_rung6`. The design run is still rung 6,
/// byte for byte, ACROSS a composite read — the composite runs between the two builds.
#[test]
fn test_cycle_untouched_by_rung58_bit_for_bit_rung6() {
    let f = flight();
    let single = || {
        build_turbojet(Gas::reacting_equilibrium(), 3.0 * 6.0, 1500.0, 50_000.0, Losses {
            pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
            pi_n: 0.98, ..Losses::default()
        }).run(&f, 50.0)
    };
    let a = single();
    let acc = accel();
    st(StatorArm::scheduled_lp(sched())).composite_credit(
        &f, &ramp(0.5).with_ds(0.02), Spool::Lp,
        &StatorLeg { accel: Some(&acc), ..Default::default() });
    let b = single();
    assert_eq!(a.performance.specific_thrust.to_bits(),
               b.performance.specific_thrust.to_bits());
    assert_eq!(a.performance.tsfc.to_bits(), b.performance.tsfc.to_bits());
}

/// Python `test_two_fuel_legs_is_refused`. Fuel-leg × fuel-leg is min-select ALGEBRA, refused at
/// the door — both the TWO-leg call and the ZERO-leg one.
#[test]
fn test_two_fuel_legs_is_refused() {
    let f = flight();
    let acc = accel();
    refuses("EXACTLY ONE fuel-side leg", || {
        st(StatorArm::scheduled_lp(sched())).composite_credit(
            &f, &ramp(0.5), Spool::Lp,
            &StatorLeg { accel: Some(&acc), surge: Some(Floor::Phi(SurgeLimiter::new(
                Spool::Lp, 0.75))), ..Default::default() });
    });
    refuses("EXACTLY ONE fuel-side leg", || {
        st(StatorArm::scheduled_lp(sched())).composite_credit(
            &f, &ramp(0.5), Spool::Lp, &StatorLeg::default());
    });
}

/// Python `test_composite_needs_an_armed_stator`.
#[test]
fn test_composite_needs_an_armed_stator() {
    let f = flight();
    let acc = accel();
    refuses("ARMED stator", || {
        st(StatorArm::default()).composite_credit(
            &f, &ramp(0.5), Spool::Lp,
            &StatorLeg { accel: Some(&acc), ..Default::default() });
    });
    refuses("BARE machine", || {
        st(StatorArm::constant(V, 0.0)).interaction_sweep(
            &f, &ramp(0.5), &[("x".to_string(), StatorArm::constant(V, 0.0))], Spool::Lp,
            &StatorLeg { accel: Some(&acc), ..Default::default() });
    });
}

// =============================================================================================
// THE FINDING
// =============================================================================================

/// Python's `_CACHE` — the four cells, memoised, so every finding gate below reads the SAME run.
/// Python memoises per pytest worker; a `OnceLock` per key is the same thing per test binary.
/// Both keys the suite ever asks for are `(kind, r = 0.5, ds = DS)`, so two locks cover it.
fn composite(kind: &str) -> &'static CompositeCredit {
    static SCHED: OnceLock<CompositeCredit> = OnceLock::new();
    static CONST: OnceLock<CompositeCredit> = OnceLock::new();
    let build = || {
        let f = flight();
        let acc = accel();
        let arm = if kind == "sched" {
            StatorArm::scheduled_lp(sched())
        } else {
            StatorArm::constant(V, 0.0)
        };
        st(arm).composite_credit(&f, &ramp(0.5), Spool::Lp,
                                 &StatorLeg { accel: Some(&acc), ..Default::default() })
    };
    match kind {
        "sched" => SCHED.get_or_init(build),
        "const" => CONST.get_or_init(build),
        _ => unreachable!("{kind}"),
    }
}

/// Python `test_p1_the_two_currencies_disagree_on_the_sign` (`slow`). P1 — THE CURRENCY IS A
/// FINDING: `M_i`'s wall is the METAL, one number shared by all four cells; `M_phi`'s moves with
/// the stator, so a four-cell second difference in it crosses two walls. The two disagree on the
/// SIGN of the stator's own credit.
///
/// **THE SECOND HALF IS A RE-GATE.** Python asserts `m.map_lp_design is LP` and
/// `m.at_stator().map_lp_design is LP` — object identity, i.e. *the wall is literally one
/// object*. [`ComponentMap`] is `Copy` and has no identity, so what is spellable here is value
/// equality across the sibling constructor. That is weaker than Python's by exactly the
/// distinction between "the same object" and "a copy with the same numbers" — and on a `Copy`
/// type those are the same claim about the WALL, which is what the gate is for. Stated rather
/// than silently dropped.
#[test]
fn test_p1_the_two_currencies_disagree_on_the_sign() {
    let d = composite("sched");
    let c = &d.cells;
    let phi_bare = c.stator.m_phi - c.neither.m_phi;
    let phi_fuel = c.both.m_phi - c.fuel.m_phi;
    assert!(d.credit_bare > 0.0, "{}", d.credit_bare);
    assert!(phi_bare < 0.0, "OPPOSITE signs: {phi_bare}");
    assert!(d.interaction > 0.0, "{}", d.interaction);
    assert!(phi_fuel - phi_bare < 0.0, "{}", phi_fuel - phi_bare);
    let m = st(StatorArm::scheduled_lp(sched()));
    assert_eq!(m.arming().map_lp_design, lp_map());
    assert_eq!(m.at_stator(StatorArm::default()).arming().map_lp_design, lp_map());
}

/// Python `test_p2_the_coupling_is_one_way` (`slow`). THE HEADLINE: the fuel leg moves the
/// stator's credit by ~10 %; the stator moves the fuel leg's engagement time by ~0.16 %.
#[test]
fn test_p2_the_coupling_is_one_way() {
    let f = flight();
    let d = composite("sched");
    let acc = accel();
    let e = st(StatorArm::scheduled_lp(sched())).engagement_shift(
        &f, &ramp(0.5), &StatorLeg { accel: Some(&acc), ..Default::default() });
    assert!(e.rel_limited.abs() < 5e-3, "{}", e.rel_limited);
    assert!(e.rel_dormant.abs() < 5e-3, "{}", e.rel_dormant);
    assert!(d.share > 0.05, "{}", d.share);
    assert!(d.share > 20.0 * e.rel_dormant.abs(),
            "{} vs 20x {}", d.share, e.rel_dormant.abs());
    assert!(0.0 < e.bare_dormant && e.bare_dormant < d.cells.neither.s,
            "a real crossing, upstream of the surge minimum: {} vs {}",
            e.bare_dormant, d.cells.neither.s);
}

/// Python `test_p3_the_state_feed_is_the_channel` (`slow`). A CONSTANT setting has no state-feed
/// and its interaction sits an order of magnitude down — NOT zero (~0.8 %), so it is reported as
/// a FLOOR rather than rounded away.
#[test]
fn test_p3_the_state_feed_is_the_channel() {
    let (sch, con) = (composite("sched"), composite("const"));
    assert_eq!(con.v_ratio, 1.0, "a constant setting cannot self-feed");
    assert!(sch.v_ratio > 1.10, "{}", sch.v_ratio);
    assert!(0.0 < con.share && con.share < 0.02, "the FLOOR — real, disclosed: {}", con.share);
    assert!(sch.share > 5.0 * con.share, "{} vs 5x {}", sch.share, con.share);
    assert!(sch.relocation < -0.05, "{}", sch.relocation);
    assert!(con.relocation < -0.05, "{}", con.relocation);
}

/// Python `test_p3_the_knee_sweep_is_monotone_in_the_commanded_setting` (`slow`).
///
/// The chained `vr[0] > vr[1] > vr[2] == vr[3] == 1.0` is ported as written, including the two
/// EXACT equalities at the saturated corner and the constant leg.
#[test]
fn test_p3_the_knee_sweep_is_monotone_in_the_commanded_setting() {
    let f = flight();
    let bare = st(StatorArm::default());
    let acc = bare.fuel.accel_schedule(&f, LO, HI, MARGIN, N_SCHED);
    let mut legs: Vec<(String, StatorArm)> = [0.60, N_LO, 0.86].iter()
        .map(|&x| (format!("n_lo={x}"), StatorArm::scheduled_lp(sched_at(x))))
        .collect();
    legs.push(("const".to_string(), StatorArm::constant(V, 0.0)));
    let rows = bare.interaction_sweep(&f, &ramp(0.5), &legs, Spool::Lp,
                                      &StatorLeg { accel: Some(&acc), ..Default::default() });
    let sh: Vec<f64> = rows.iter().map(|x| x.share).collect();
    let vr: Vec<f64> = rows.iter().map(|x| x.v_ratio).collect();
    assert!(vr[0] > vr[1] && vr[1] > vr[2], "the setting ratio must fall: {vr:?}");
    assert_eq!(vr[2], 1.0, "{vr:?}");
    assert_eq!(vr[3], 1.0, "{vr:?}");
    assert!(sh[0] > sh[1] && sh[1] > sh[2] && sh[2] > sh[3] && sh[3] > 0.0,
            "...and so must the share: {sh:?}");
    assert!(sh[2] < 0.25 * sh[1], "the saturated corner COLLAPSES: {} vs {}", sh[2], sh[1]);
    assert!(sh[2] > sh[3], "...but not all the way to the floor: {} vs {}", sh[2], sh[3]);
    assert!(sched_at(0.86).at(0.94) < 0.5 * V,
            "why: it opens again downstream: {}", sched_at(0.86).at(0.94));
}

/// Python `test_p3_the_interaction_is_predicted_by_the_no_leg_marches` (`slow`). The stator's
/// credit is a PROFILE in `s`; the fuel leg changes WHICH POINT is read, not the profile.
#[test]
fn test_p3_the_interaction_is_predicted_by_the_no_leg_marches() {
    for (kind, lo, hi) in [("sched", 0.7, 1.0), ("const", 0.7, 1.5)] {
        let d = composite(kind);
        let ratio = d.predicted / d.interaction;
        assert!(lo < ratio && ratio < hi,
                "{kind}: predicted {} / interaction {} = {ratio}", d.predicted, d.interaction);
        assert!(d.predicted > 0.0, "{kind}: {}", d.predicted);
    }
}

/// Python `test_p6_not_a_ramp_rate_artifact` (`slow`). The deflation rungs 44/48 taught the
/// project to exclude — "any clip removes fuel and slows the accel" — refused by measuring the
/// leg's own COST with and without the stator.
#[test]
fn test_p6_not_a_ramp_rate_artifact() {
    for (kind, band, fband) in [("sched", 0.04, 0.01), ("const", 0.10, 0.03)] {
        let d = composite(kind);
        let (a, b) = (d.leg_cost_bare, d.leg_cost_armed);
        assert!(a < 0.0 && b < 0.0, "{kind}: the leg does cost speed: {a} {b}");
        assert!((b - a).abs() < band * a.abs(), "{kind}: {a} vs {b}");
        assert!((d.fuel_removed_armed - d.fuel_removed_bare).abs()
                    < fband * d.fuel_removed_bare,
                "{kind}: {} vs {}", d.fuel_removed_bare, d.fuel_removed_armed);
    }
    // The exclusion is only meaningful where there IS an interaction to explain away — on the
    // CONSTANT leg the interaction is already at the floor, so that comparison is vacuous and is
    // NOT asserted (its cost drift, the larger of the two, is reported by the sweep, not hidden).
    let d = composite("sched");
    let drift = ((d.leg_cost_armed - d.leg_cost_bare) / d.leg_cost_bare).abs();
    assert!(d.share.abs() > 3.0 * drift, "share {} vs 3x drift {drift}", d.share);
}

/// Python `test_p4_the_decomposition_is_clocked_but_the_delivered_credit_is_not` (`slow`).
///
/// The INTERACTION is strongly clocked and NON-MONOTONE; the DELIVERED credit is not. The
/// envelope edge is gated with it: at `r = 2.00` the leg never binds (`fuel_removed` EXACTLY
/// zero), which makes `fuel` bit-identical to `neither` and the second difference trivially
/// zero — that row is asserted to BE dormant so it can never be quoted as evidence.
#[test]
fn test_p4_the_decomposition_is_clocked_but_the_delivered_credit_is_not() {
    let f = flight();
    let acc = accel();
    let leg = StatorLeg { accel: Some(&acc), ..Default::default() };
    let rates = [0.15, 0.25, 0.50, 1.00, 2.00];
    let rows: Vec<CompositeCredit> = rates.iter()
        .map(|&r| st(StatorArm::scheduled_lp(sched()))
                      .composite_credit(&f, &ramp(r), Spool::Lp, &leg))
        .collect();
    assert_eq!(rows[4].fuel_removed_bare, 0.0, "r = 2.00 must be DORMANT");
    assert_eq!(rows[4].interaction, 0.0, "...so its zero is inadmissible");
    let live = &rows[..4];
    for (r, d) in rates.iter().zip(live.iter()) {
        assert!(d.fuel_removed_bare > 0.0, "every scored row really binds — r = {r}");
    }
    let sh: Vec<f64> = live.iter().map(|d| d.share).collect();
    let (shmin, shmax) = (sh.iter().cloned().fold(f64::INFINITY, f64::min),
                          sh.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!(shmax > 10.0 * shmin.max(1e-4), "a CLOCK on the DECOMPOSITION: {sh:?}");
    let cr: Vec<f64> = live.iter().map(|d| d.credit_bare).collect();
    let (crmin, crmax) = (cr.iter().cloned().fold(f64::INFINITY, f64::min),
                          cr.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!((crmax - crmin) / crmin < 0.10, "rung-57 invariant over the SAME range: {cr:?}");
    let co: Vec<f64> = live.iter().map(|d| d.credit_bare + d.interaction).collect();
    let (comin, comax) = (co.iter().cloned().fold(f64::INFINITY, f64::min),
                          co.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!((comax - comin) / comin < (crmax - crmin) / crmin,
            "THE DELIVERED CREDIT IS FLATTER STILL: {cr:?} vs {co:?}");
    assert!(live[1].share > live[0].share, "rising limb: {sh:?}");
    assert!(live[1].share > live[2].share, "falling limb: {sh:?}");
    assert!(live[1].relocation.abs() > live[3].relocation.abs(),
            "the share tracks the relocation: {} vs {}", live[1].relocation,
            live[3].relocation);
}

/// Python `test_p5_the_phi_leg_is_not_composable_at_a_fixed_set_point` (`slow`). P5 — REFUTED as
/// pre-registered, by something harder than a magnitude: the admissible WINDOWS for the bare and
/// statored machines are DISJOINT.
#[test]
fn test_p5_the_phi_leg_is_not_composable_at_a_fixed_set_point() {
    let f = flight();
    let rp = ramp(0.5);
    let none = StatorLeg::default();
    let window = |arm: StatorArm| {
        let (traj, _) = st(arm).stator_march(&f, &rp, None, &none);
        (traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min), traj[0].phi_lp)
    };
    let bare = window(StatorArm::default());
    for (tag, arm) in [("sched", StatorArm::scheduled_lp(sched())),
                       ("const", StatorArm::constant(V, 0.0))] {
        let w = window(arm);
        assert!(w.0 < w.1, "{tag}: a window exists at all: {w:?}");
        assert!(w.1 < bare.0, "{tag}: ...and it is DISJOINT: {w:?} vs {bare:?}");
    }
    let sched_w = window(StatorArm::scheduled_lp(sched()));
    assert!(bare.0 - sched_w.1 > 0.01, "by a resolvable gap: {}", bare.0 - sched_w.1);
}

/// Python `test_p5_a_pinned_floor_annihilates_rung57_erosion_exactly` (`slow`). The sharpest
/// number in the rung: when the `phi` floor pins BOTH cells' incidence minima at `phi = phi_lim`,
/// the stator's credit is EXACTLY the setting it commands there, with rung 57's erosion at
/// exactly zero. Gated at machine precision and at TWO floors — the floor-independence is what
/// proves it is the pinning and not a coincidence.
#[test]
fn test_p5_a_pinned_floor_annihilates_rung57_erosion_exactly() {
    let f = flight();
    let rp = ramp(0.5);
    for arm in [StatorArm::constant(V, 0.0), StatorArm::scheduled_lp(sched())] {
        let mut got = Vec::new();
        for floor in [0.7450, 0.7500] {
            let c = st(arm).composite_credit(&f, &rp, Spool::Lp, &StatorLeg {
                surge: Some(Floor::Phi(SurgeLimiter::new(Spool::Lp, floor))),
                ..Default::default() });
            assert!((c.cells.fuel.min_phi - floor).abs() < 1e-9, "pinned: {}", c.cells.fuel.min_phi);
            assert!((c.cells.both.min_phi - floor).abs() < 1e-9, "pinned: {}", c.cells.both.min_phi);
            assert!((c.credit_fuel - c.v_fuel).abs() < 1e-12,
                    "THE IDENTITY: {} vs {}", c.credit_fuel, c.v_fuel);
            got.push(c.credit_fuel);
        }
        assert!((got[0] - got[1]).abs() < 1e-12, "floor-INDEPENDENT: {got:?}");
    }
    let c = st(StatorArm::constant(V, 0.0)).composite_credit(&f, &rp, Spool::Lp, &StatorLeg {
        surge: Some(Floor::Phi(SurgeLimiter::new(Spool::Lp, 0.7450))), ..Default::default() });
    assert!((c.credit_fuel - V).abs() < 1e-12,
            "a CONSTANT setting's credit IS the setting: {}", c.credit_fuel);
}
