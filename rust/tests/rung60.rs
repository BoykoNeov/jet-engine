//! RUNG 60 — **THE MATCHED `phi` FLOOR: a floor PINS its own coordinate.**
//!
//! `tests/test_rung60.py` ported one-to-one: **16 Python `def test_` → 16 collected → 16
//! `#[test]` here** (no `parametrize` in this file).
//!
//! # THE GRID — and this is the ONE suite whose `ds` really is 0.005
//!
//! `DS = 0.005` here, against 0.01 in all three of `rung57.rs` / `rung58.rs` / `rung59.rs`.
//! [`Ramp::fine`]'s doc comment calls 0.005 *"rungs 58/59/60's default"*, which is true of the
//! reader-method SIGNATURES and true of this suite — and **false of the rung-58 and rung-59
//! suites**, which declare `DS = 0.01` and pass it explicitly. `KEYS` here is twelve names and
//! leads with `s`. Nothing is shared with the other three files.
//!
//! # TWO OF PYTHON'S SIXTEEN GATES TEST A REFUSAL THIS PORT MAKES UNREPRESENTABLE
//!
//! `test_floor_composite_refuses_a_feedforward_leg` hands `floor_composite` an
//! [`AccelSchedule`], and `test_composability_ladder_walks_exactly_one_axis` hands
//! `composability_ladder` both of its two mutually-exclusive keyword lists (and neither). In
//! Rust the first takes a [`Floor`] and the second a [`LadderAxis`], so **neither bad call can
//! be written down**. That is strictly stronger than Python's runtime assert, but it is NOT the
//! same gate, and pretending otherwise is the *ported test can go VACUOUS* failure.
//!
//! Both are kept at 1:1 and re-gated on what survives: an EXHAUSTIVE `match` over each enum,
//! which stops compiling the moment a third variant appears — which is exactly the event
//! Python's assert exists to catch (a leg kind reaching a reader that cannot mean it). The
//! runtime half of each test keeps whatever Python asserted that is still expressible.
//!
//! Contrast rung 57's `Shape`: there the bad value is a STRING a caller could supply, so the
//! port kept a `try_from_str` entry point and the refusal stayed a runtime observable. Here the
//! bad value is a different TYPE. The two cases are decided differently and both are stated.
//!
//! # WHAT THESE 16 GATES DO NOT ESTABLISH
//!
//! They are **relational** — step 4's oracle is the instrument for agreement with Python.
//!
//! [`Ramp::fine`]: turbojet::stator_transient::Ramp::fine
//! [`AccelSchedule`]: turbojet::fuel_transient::AccelSchedule
//! [`Floor`]: turbojet::fuel_transient::Floor
//! [`LadderAxis`]: turbojet::stator_transient::LadderAxis

use std::panic::catch_unwind;

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::fuel_transient::{Floor, FuelPoint, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    counters as scount, IncidenceLimiter, LadderAxis, LegKind, Ramp, Regime, ScheduledStatorCore,
    ScheduledStatorTransient, StatorArm, StatorLeg, StatorSchedule,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR_PHI: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
/// **0.005** — the ONE suite of the four that marches on the reader default. See the header.
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const N_LO: f64 = 0.7557;
const N_SCHED: usize = 13;
/// The three ADMISSIBLE `(v, m_lim)` pairs — mid-overlap set points on the constant ladder,
/// where `credit < excursion` holds and neither cell is dormant or binds from `s = 0`.
const ADMISSIBLE: [(f64, f64); 3] = [(0.05, 0.500), (0.10, 0.509), (0.15, 0.518)];
/// TWELVE keys, leading with `s`.
const KEYS: [&str; 13] = ["s", "nu_lp", "nu_hp", "Tt4", "phi_lp", "phi_hp", "mf", "mf_sched",
                          "f", "pi_lpc", "pi_hpc", "mdot_air", "sp_thrust"];

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
        .with_phi_surge(FLOOR_PHI)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR_PHI)
}

fn t_c() -> f64 {
    lp_map().tan_beta1_crit()
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

fn ramp(r: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds: DS }
}

/// Python's `_march`.
fn march(m: &ScheduledStatorCore, rp: &Ramp, leg: &StatorLeg<'_>) -> Vec<FuelPoint> {
    m.stator_march(&flight(), rp, None, leg).0
}

fn pt(p: &FuelPoint, k: &str) -> f64 {
    match k {
        "s" => p.s,
        "nu_lp" => p.nu_lp,
        "nu_hp" => p.nu_hp,
        "Tt4" => p.tt4,
        "phi_lp" => p.phi_lp,
        "phi_hp" => p.phi_hp,
        "mf" => p.mf,
        "mf_sched" => p.mf_sched,
        "f" => p.f,
        "pi_lpc" => p.pi_lpc,
        "pi_hpc" => p.pi_hpc,
        "mdot_air" => p.mdot_air,
        "sp_thrust" => p.sp_thrust,
        _ => unreachable!("{k}"),
    }
}

/// Python's `_bitwise` — the length assertion is Python's own, kept ahead of the zip.
fn bitwise(a: &[FuelPoint], b: &[FuelPoint]) {
    assert_eq!(a.len(), b.len(), "the two marches must land on the same grid");
    for (i, (p, q)) in a.iter().zip(b.iter()).enumerate() {
        for k in KEYS {
            assert_eq!(pt(p, k).to_bits(), pt(q, k).to_bits(),
                       "{k} at row {i} (s = {}): {} vs {}", p.s, pt(p, k), pt(q, k));
        }
    }
}

fn panic_text(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_default()
}

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
// THE REDUCE
// =============================================================================================

/// Python `test_reduce_incidence_floor_at_v_zero_is_bit_for_bit_rung49`. THE STRONG IDENTITY
/// REDUCE: `at()` computes `1/(T_c + 0.0 - m_lim)` and `x + 0.0 == x` exactly, so the SAME float
/// reaches rung 49's surge path and the whole march is bit-for-bit.
#[test]
fn test_reduce_incidence_floor_at_v_zero_is_bit_for_bit_rung49() {
    let m_lim = 0.500;
    let tc = t_c();
    let inc = IncidenceLimiter::new(Spool::Lp, m_lim);
    let phi = SurgeLimiter::new(Spool::Lp, 1.0 / (tc - m_lim));
    assert_eq!(inc.at(tc, 0.0).phi_lim.to_bits(), phi.phi_lim.to_bits(),
               "float-identical, not close");
    assert_eq!(inc.at(tc, 0.0), phi, "and the whole leg");
    let bare = st(StatorArm::default());
    let rp = ramp(0.5);
    bitwise(&march(&bare, &rp, &StatorLeg { surge: Some(Floor::Incidence(inc)),
                                            ..Default::default() }),
            &march(&bare, &rp, &StatorLeg { surge: Some(Floor::Phi(phi)),
                                            ..Default::default() }));
}

/// Python `test_reduce_a_rung49_floor_passes_the_resolver_by_identity`.
///
/// **A RE-GATE, and the shipped source already says so** at `resolve_floor`'s own definition.
/// Python asserts `mach._resolve_floor(phi, .9, .9) is phi` — object identity, so rungs 49–59
/// provably reach the very object they passed in. [`SurgeLimiter`] is a `Copy` value type here,
/// so identity is unspellable and what remains is equality. What restores the missing half is
/// the crate's own dispatch counters: the `phi` arm must be the one that RAN
/// (`resolve_phi` moves, `resolve_incidence` does not), which is what "handed back unchanged"
/// reduces to when the alternative is a CONVERSION rather than a copy.
#[test]
fn test_reduce_a_rung49_floor_passes_the_resolver_by_identity() {
    let phi = SurgeLimiter::new(Spool::Lp, 0.75);
    scount::reset();
    let arms = [StatorArm::default(), StatorArm::constant(0.15, 0.0),
                StatorArm::scheduled_lp(StatorSchedule::new(0.20, N_LO))];
    let n = arms.len() as u64;
    for arm in arms {
        let got = st(arm).fuel.resolve_floor(&Floor::Phi(phi), 0.9, 0.9)
            .expect("a phi floor resolves");
        assert_eq!(got, phi);
        assert_eq!(got.phi_lim.to_bits(), phi.phi_lim.to_bits());
    }
    let c = scount::take();
    assert_eq!(c.resolve_phi, n, "every call must take the PASS-THROUGH arm");
    assert_eq!(c.resolve_incidence, 0, "...and never the CONVERSION one");
}

/// Python `test_reduce_rung57_58_59_marches_untouched`.
#[test]
fn test_reduce_rung57_58_59_marches_untouched() {
    let f = flight();
    let rp = ramp(0.5);
    let none = StatorLeg::default();
    bitwise(&march(&st(StatorArm::default()), &rp, &none),
            &march(&st(StatorArm::default()), &rp, &none));
    bitwise(&march(&st(StatorArm::constant(0.10, 0.0)), &rp, &none),
            &march(&st(StatorArm::constant(0.10, 0.0)), &rp, &none));
    let leg_sched = st(StatorArm::default()).fuel.accel_schedule(&f, LO, HI, 0.25, N_SCHED);
    let leg = StatorLeg { accel: Some(&leg_sched), ..Default::default() };
    bitwise(&march(&st(StatorArm::constant(0.10, 0.0)), &rp, &leg),
            &march(&st(StatorArm::constant(0.10, 0.0)), &rp, &leg));
}

/// Python `test_reduce_rung58_composite_still_runs_and_reports_its_leg`.
#[test]
fn test_reduce_rung58_composite_still_runs_and_reports_its_leg() {
    let f = flight();
    let m = st(StatorArm::scheduled_lp(StatorSchedule::new(0.20, N_LO)));
    let leg = m.at_stator(StatorArm::default()).fuel.accel_schedule(&f, LO, HI, 0.25, N_SCHED);
    let d = m.composite_credit(&f, &ramp(0.5).with_ds(0.01), Spool::Lp,
                               &StatorLeg { accel: Some(&leg), ..Default::default() });
    assert_eq!(d.leg, LegKind::Accel);
    assert!(d.cells.both.fuel_removed > 0.0, "{}", d.cells.both.fuel_removed);
}

/// Python `test_cycle_untouched_by_rung60_bit_for_bit_rung6`.
#[test]
fn test_cycle_untouched_by_rung60_bit_for_bit_rung6() {
    let f = flight();
    let single = || {
        build_turbojet(Gas::reacting_equilibrium(), 3.0 * 6.0, 1500.0, 50_000.0, Losses {
            pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
            pi_n: 0.98, ..Losses::default()
        }).run(&f, 50.0)
    };
    let a = single();
    st(StatorArm::constant(0.10, 0.0)).floor_composite(
        &f, &ramp(0.5).with_ds(0.02),
        &Floor::Incidence(IncidenceLimiter::new(Spool::Lp, 0.509)), Spool::Lp);
    let b = single();
    assert_eq!(a.performance.specific_thrust.to_bits(),
               b.performance.specific_thrust.to_bits());
    assert_eq!(a.performance.tsfc.to_bits(), b.performance.tsfc.to_bits());
}

// =============================================================================================
// THE GUARDS
// =============================================================================================

/// Python `test_floor_composite_refuses_a_feedforward_leg` — **RE-GATED AS A COMPILE-TIME
/// EXHAUSTIVENESS CHECK; see the file header.**
///
/// Python hands `floor_composite` an [`AccelSchedule`] and asserts it refuses with *"FLOOR leg"*.
/// Rust's `floor_composite` takes `&Floor`, whose only inhabitants are a `phi` floor and an
/// incidence one, so the bad call **cannot be written**. The `match` below is exhaustive over
/// exactly those two, and it stops compiling if a third variant is ever added — which is the
/// event Python's assert exists to catch. Beside it, the runtime half Python's test is really
/// about: the two readers report DIFFERENT things on the same machine, so a schedule can never
/// be mistaken for a floor by reading the result.
#[test]
fn test_floor_composite_refuses_a_feedforward_leg() {
    let f = flight();
    let inc = IncidenceLimiter::new(Spool::Lp, 0.509);
    // The compile-time half. Adding a `Floor` variant breaks this line before any test runs.
    let kind_of = |fl: &Floor| match fl {
        Floor::Phi(_) => "phi",
        Floor::Incidence(_) => "incidence",
    };
    assert_eq!(kind_of(&Floor::Incidence(inc)), "incidence");
    assert_eq!(kind_of(&Floor::Phi(SurgeLimiter::new(Spool::Lp, 0.75))), "phi");

    // The runtime half: a FLOOR composite and a SCHEDULE composite are different readings of
    // the same machine, and only the floor one reports a regime.
    let m = st(StatorArm::constant(0.10, 0.0));
    let rp = ramp(0.5).with_ds(0.02);
    let fc = m.floor_composite(&f, &rp, &Floor::Incidence(inc), Spool::Lp);
    let leg = m.at_stator(StatorArm::default()).fuel.accel_schedule(&f, LO, HI, 0.25, N_SCHED);
    let cc = m.composite_credit(&f, &rp, Spool::Lp,
                                &StatorLeg { accel: Some(&leg), ..Default::default() });
    assert_eq!(cc.leg, LegKind::Accel, "a schedule leg reports itself as a schedule");
    assert!(matches!(fc.regime, Regime::BothPinned | Regime::ArmedClears | Regime::Mixed));
    assert_ne!(fc.credit_fuel.to_bits(), cc.credit_fuel.to_bits(),
               "the two readers must not be measuring the same quantity");
}

/// Python `test_floor_composite_and_bands_need_an_armed_stator`. Both halves survive as runtime
/// refusals — the receiver is the thing being refused, and a bare machine is perfectly
/// representable.
#[test]
fn test_floor_composite_and_bands_need_an_armed_stator() {
    let f = flight();
    let floor = Floor::Incidence(IncidenceLimiter::new(Spool::Lp, 0.509));
    refuses("ARMED stator", || {
        st(StatorArm::default()).floor_composite(
            &f, &ramp(0.5).with_ds(0.02), &floor, Spool::Lp);
    });
    refuses("ARMED machine", || {
        st(StatorArm::default()).set_point_bands(&f, &ramp(0.5).with_ds(0.02), Spool::Lp);
    });
}

/// Python `test_composability_ladder_walks_exactly_one_axis` — **RE-GATED AS A COMPILE-TIME
/// EXHAUSTIVENESS CHECK; see the file header.**
///
/// Python passes BOTH keyword lists (and then NEITHER) and asserts *"ONE axis"* each time.
/// Rust's `composability_ladder` takes a [`LadderAxis`], which is one axis or the other and
/// cannot be both or neither. The exhaustive `match` below is the surviving statement of that,
/// and it breaks if a third axis is added. The runtime half asserts what the refusal protects:
/// the two axes really do carry different halves of the criterion, so confounding them would
/// have destroyed the finding.
#[test]
fn test_composability_ladder_walks_exactly_one_axis() {
    let f = flight();
    let rp = ramp(0.5).with_ds(0.02);
    let legs = [("a".to_string(), StatorArm::constant(0.1, 0.0))];
    let rates = [(0.5, StatorArm::constant(0.1, 0.0))];
    // The compile-time half.
    let axis_of = |a: &LadderAxis<'_>| match a {
        LadderAxis::Legs(_) => "legs",
        LadderAxis::Rates(_) => "rates",
    };
    assert_eq!(axis_of(&LadderAxis::Legs(&legs)), "legs");
    assert_eq!(axis_of(&LadderAxis::Rates(&rates)), "rates");

    // The runtime half: one axis moves the CREDIT, the other moves the EXCURSION.
    let m = st(StatorArm::default());
    let by_legs = m.composability_ladder(&f, &rp, LadderAxis::Legs(&[
        ("v=0.05".to_string(), StatorArm::constant(0.05, 0.0)),
        ("v=0.20".to_string(), StatorArm::constant(0.20, 0.0))]), Spool::Lp);
    let by_rates = m.composability_ladder(&f, &rp, LadderAxis::Rates(&[
        (0.15, StatorArm::constant(0.20, 0.0)),
        (1.00, StatorArm::constant(0.20, 0.0))]), Spool::Lp);
    assert!((by_legs[1].credit - by_legs[0].credit).abs()
                > (by_rates[1].credit - by_rates[0].credit).abs(),
            "the LEG axis is the credit axis: {:?} vs {:?}",
            (by_legs[0].credit, by_legs[1].credit), (by_rates[0].credit, by_rates[1].credit));
    assert!((by_rates[1].excursion - by_rates[0].excursion).abs()
                > (by_legs[1].excursion - by_legs[0].excursion).abs(),
            "the RATE axis is the excursion axis: {:?} vs {:?}",
            (by_rates[0].excursion, by_rates[1].excursion),
            (by_legs[0].excursion, by_legs[1].excursion));
}

/// Python `test_incidence_floor_above_the_critical_incidence_is_refused`. `m_lim >= T_c + v`
/// means no `phi` realises the floor — caught at the conversion, not deep inside a bracket
/// search.
#[test]
fn test_incidence_floor_above_the_critical_incidence_is_refused() {
    let tc = t_c();
    refuses("critical incidence", move || {
        IncidenceLimiter::new(Spool::Lp, 2.0).at(tc, 0.0);
    });
}

// =============================================================================================
// THE FINDINGS
// =============================================================================================

/// Python `test_p2_matching_a_set_point_is_under_determined`. The two natural matching rules
/// give different floors, apart by exactly `v*sm/(1+sm)` in the incidence coordinate — DERIVED,
/// zero new constants, and zero exactly when either the lever or the margin is.
#[test]
fn test_p2_matching_a_set_point_is_under_determined() {
    let m = st(StatorArm::constant(0.20, 0.0));
    for sm in [0.0, 0.02, 0.05, 0.10, 0.25] {
        for v in [0.0, 0.05, 0.20] {
            let d = m.matching_rules(sm, v, Spool::Lp);
            assert!(d.residual.abs() < 1e-14, "sm = {sm}, v = {v}: {}", d.residual);
            if sm != 0.0 && v != 0.0 {
                assert!(d.gap > 0.0, "incidence matches the TIGHTER floor: {}", d.gap);
            } else {
                assert!(d.gap == 0.0 || d.gap.abs() < 1e-15, "sm = {sm}, v = {v}: {}", d.gap);
            }
        }
    }
}

/// Python `test_p3_re_referencing_shrinks_the_set_point_gap_by_an_order_of_magnitude` (`slow`).
/// The incidence gap obeys `credit - excursion` as an ALGEBRAIC IDENTITY, asserted EXACTLY.
#[test]
fn test_p3_re_referencing_shrinks_the_set_point_gap_by_an_order_of_magnitude() {
    let d = st(StatorArm::constant(0.20, 0.0)).set_point_bands(&flight(), &ramp(0.5), Spool::Lp);
    assert_eq!(d.identity_residual, 0.0, "an identity, not a measurement");
    assert!(d.gap_phi_bands > 1.0, "measured 1.053: {}", d.gap_phi_bands);
    assert!(0.0 < d.gap_m_bands && d.gap_m_bands < 0.10, "measured 0.044: {}", d.gap_m_bands);
    assert!(d.gap_phi_bands / d.gap_m_bands > 10.0,
            "measured 24x: {}", d.gap_phi_bands / d.gap_m_bands);
    assert!(!d.phi_admissible && !d.m_admissible);
}

/// Python `test_p3_the_criterion_is_crossed_on_the_stator_ladder` (`slow`). The load-bearing
/// claim is that the criterion is CROSSED inside the swept range and that the verdict tracks the
/// sign — the gate is the IMPLICATION plus the two measured verdict vectors, not "phi always
/// fails", which is false at the smallest setting.
#[test]
fn test_p3_the_criterion_is_crossed_on_the_stator_ladder() {
    let mut legs: Vec<(String, StatorArm)> = [0.05, 0.15, 0.20].iter()
        .map(|&v| (format!("const v={v}"), StatorArm::constant(v, 0.0)))
        .collect();
    legs.push(("sched v_max=0.20".to_string(),
               StatorArm::scheduled_lp(StatorSchedule::new(0.20, N_LO))));
    let rows = st(StatorArm::default())
        .composability_ladder(&flight(), &ramp(0.5), LadderAxis::Legs(&legs), Spool::Lp);
    for row in &rows {
        assert_eq!(row.m_admissible, row.criterion < 0.0, "{}", row.tag);
        assert!(row.m_admissible || !row.phi_admissible,
                "re-referencing can only HELP: {}", row.tag);
    }
    assert_eq!(rows.iter().map(|r| r.m_admissible).collect::<Vec<_>>(),
               vec![true, true, false, true]);
    assert_eq!(rows.iter().map(|r| r.phi_admissible).collect::<Vec<_>>(),
               vec![true, false, false, false]);
    assert!(rows[0].credit < rows[1].credit && rows[1].credit < rows[2].credit,
            "monotone in v: {:?}", rows.iter().map(|r| r.credit).collect::<Vec<_>>());
}

/// Python `test_p4_the_crossing_is_clocked_by_the_ramp_not_by_the_lever` (`slow`). THE
/// MECHANISM: at a FIXED stator setting the threshold is crossed by ramp rate alone.
#[test]
fn test_p4_the_crossing_is_clocked_by_the_ramp_not_by_the_lever() {
    let rates: Vec<(f64, StatorArm)> = [0.15, 0.25, 0.50, 0.75, 1.00].iter()
        .map(|&r| (r, StatorArm::constant(0.20, 0.0)))
        .collect();
    let rows = st(StatorArm::default())
        .composability_ladder(&flight(), &ramp(0.5), LadderAxis::Rates(&rates), Spool::Lp);
    let cr: Vec<f64> = rows.iter().map(|r| r.credit).collect();
    let ex: Vec<f64> = rows.iter().map(|r| r.excursion).collect();
    let mm = |v: &[f64]| (v.iter().cloned().fold(f64::INFINITY, f64::min),
                          v.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let (crlo, crhi) = mm(&cr);
    let (exlo, exhi) = mm(&ex);
    let (spread_cr, spread_ex) = (crhi / crlo - 1.0, exhi / exlo - 1.0);
    assert!(spread_cr < 0.015, "measured 0.93 %: {cr:?}");
    assert!(spread_ex > 2.0, "measured 4.21x: {ex:?}");
    assert!(spread_ex / spread_cr > 100.0, "measured ~345x: {spread_cr} {spread_ex}");
    assert!(rows[0].m_admissible && !rows[rows.len() - 1].m_admissible);
    for w in ex.windows(2) {
        assert!(w[1] < w[0], "monotone in r: {ex:?}");
    }
    let flips = rows.windows(2).filter(|w| w[0].m_admissible != w[1].m_admissible).count();
    assert_eq!(flips, 1, "the verdict flips exactly once, on the excursion's back: {:?}",
               rows.iter().map(|r| r.m_admissible).collect::<Vec<_>>());
}

/// Python `test_p1_a_floor_pins_its_own_coordinate_so_the_composite_is_a_tautology` (`slow`).
/// THE RUNG. The gate is that the measurement MEETS the derived value at machine precision —
/// the opposite of the usual gate, and that is the point: a number reproduced to 1e-15 by an
/// identity is not evidence about the machine.
#[test]
fn test_p1_a_floor_pins_its_own_coordinate_so_the_composite_is_a_tautology() {
    let f = flight();
    let rp = ramp(0.5);
    // the incidence end — the matched floor the seam asked for
    for (v, m_lim) in ADMISSIBLE {
        let d = st(StatorArm::constant(v, 0.0)).floor_composite(
            &f, &rp, &Floor::Incidence(IncidenceLimiter::new(Spool::Lp, m_lim)), Spool::Lp);
        assert_eq!(d.regime, Regime::BothPinned, "v = {v}");
        assert!(d.admissible, "v = {v}: {:?} {:?}", d.audit_fuel, d.audit_both);
        assert_eq!(d.pinned_prediction, 0.0);
        assert!(d.credit_fuel.abs() < 1e-12, "v = {v}: {}", d.credit_fuel);
        assert!((d.interaction + d.credit_bare).abs() < 1e-12,
                "the interaction carries nothing: v = {v}, {} {}", d.interaction, d.credit_bare);
    }
    // the phi end — rung 58's by-product, at a setting rung 58 never ran
    for v in [0.15, 0.20] {
        let d = st(StatorArm::constant(v, 0.0)).floor_composite(
            &f, &rp, &Floor::Phi(SurgeLimiter::new(Spool::Lp, 0.750)), Spool::Lp);
        assert_eq!(d.regime, Regime::BothPinned);
        assert!((d.credit_fuel - v).abs() < 1e-12, "v = {v}: {}", d.credit_fuel);
        assert!(d.pinned_residual.abs() < 1e-12, "v = {v}: {}", d.pinned_residual);
        assert!(d.audit_both.from_zero, "v = {v}");
        assert!(!d.admissible, "v = {v}");
    }
}

/// Python `test_p1_the_third_regime_carries_no_armed_cell_dynamics_either` (`slow`). The escape
/// from pinning is a floor the ARMED machine clears — and it is no escape.
#[test]
fn test_p1_the_third_regime_carries_no_armed_cell_dynamics_either() {
    let d = st(StatorArm::constant(0.15, 0.0)).floor_composite(
        &flight(), &ramp(0.5),
        &Floor::Incidence(IncidenceLimiter::new(Spool::Lp, 0.490)), Spool::Lp);
    assert_eq!(d.regime, Regime::ArmedClears);
    assert!(d.audit_both.dormant);
    assert_eq!(d.removed_armed, 0.0);
    assert!(d.pinned_residual.abs() < 1e-12, "{}", d.pinned_residual);
    assert_eq!(d.cells.both.m_i.to_bits(), d.cells.stator.m_i.to_bits(), "bit-identical");
}

/// Python `test_p5_the_timing_half_survives_because_a_time_has_no_wall` (`slow`). WHAT IS NOT
/// PINNED: `s_eng` is a time, not a margin, so nothing floors it.
#[test]
fn test_p5_the_timing_half_survives_because_a_time_has_no_wall() {
    let f = flight();
    let rp = ramp(0.5);
    for (v, m_lim) in ADMISSIBLE {
        let d = st(StatorArm::constant(v, 0.0)).floor_composite(
            &f, &rp, &Floor::Incidence(IncidenceLimiter::new(Spool::Lp, m_lim)), Spool::Lp);
        assert!(d.s_eng_bare.is_finite() && d.s_eng_armed.is_finite(),
                "v = {v}: {} {}", d.s_eng_bare, d.s_eng_armed);
        assert!(d.d_s_eng > 0.0, "the stator DELAYS the engagement: v = {v}, {}", d.d_s_eng);
        assert!((d.d_s_eng / d.s_eng_bare).abs() > 0.50, "v = {v}: {}", d.d_s_eng);
        assert!(d.removed_armed < d.removed_bare,
                "v = {v}: {} vs {}", d.removed_armed, d.removed_bare);
    }
}
