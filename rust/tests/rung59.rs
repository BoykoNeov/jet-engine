//! RUNG 59 — **THE MATCHED SCHEDULE: a schedule's ORDINATE cannot see a stator, only its INDEX
//! can.**
//!
//! `tests/test_rung59.py` ported one-to-one: **12 Python `def test_` → 12 collected → 12
//! `#[test]` here** (no `parametrize` in this file).
//!
//! # THE GRID
//!
//! `N_LO` is **0.7557** (rung 57's file writes 0.75574), `MARGIN` 0.25, and `V_HP = 0.10` — the
//! HP branch, whose authority saturates near 0.15. As in rung 58, **`ds` is the suite's own
//! 0.01, passed explicitly at every call site**, NOT the 0.005 these reader methods default to;
//! [`Ramp::fine`]'s doc comment records the signature default and is not this suite's step.
//!
//! # WHAT THESE 12 GATES DO NOT ESTABLISH
//!
//! They are **relational**. A Rust/Python arithmetic divergence moves both sides of every one and
//! leaves all 12 green; step 4's oracle is the instrument for agreement with Python.
//!
//! [`Ramp::fine`]: turbojet::stator_transient::Ramp::fine

use std::panic::catch_unwind;

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::fuel_transient::AccelSchedule;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    ClampAudit, LegKind, MatchedCredit, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
    StatorArm, StatorLeg, StatorSchedule,
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
/// The SUITE's step — see the header.
const DS: f64 = 0.01;
const SETTLE: f64 = 1.2;
const N_LO: f64 = 0.7557;
const MARGIN: f64 = 0.25;
/// The HP branch; authority saturates at ~0.15.
const V_HP: f64 = 0.10;
const N_SCHED: usize = 13;

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

fn sched_v(v_max: f64) -> StatorSchedule {
    StatorSchedule::new(v_max, N_LO)
}

fn sched() -> StatorSchedule {
    sched_v(V)
}

fn ramp(r: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds: DS }
}

/// Python's `_matched`.
fn matched(arm: StatorArm, spool: Spool) -> MatchedCredit {
    st(arm).matched_credit(&flight(), &ramp(0.5), MARGIN, spool, N_SCHED)
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

/// Python `test_reduce_v_zero_gives_tuple_identity_and_exactly_zero_delta_match`.
///
/// THE STRONG IDENTITY REDUCE. At `v = 0` the armed machine IS the bare machine, so it derives
/// the SAME equilibria and hence a table equal by Python TUPLE equality — not to a tolerance.
/// Rust's `Vec<f64>` equality is the same claim, and it is asserted **bit-for-bit** here (`==`
/// on `f64` treats `-0.0 == 0.0` and NaN as unequal; the bit compare below cannot be satisfied
/// by a sign-of-zero difference the Python tuple equality would have accepted, so this is
/// marginally STRONGER, in the direction that cannot hide a defect).
#[test]
fn test_reduce_v_zero_gives_tuple_identity_and_exactly_zero_delta_match() {
    let f = flight();
    let l_bare = st(StatorArm::default()).fuel.accel_schedule(&f, LO, HI, MARGIN, N_SCHED);
    for (tag, arm) in [("sched v_max=0", StatorArm::scheduled_lp(sched_v(0.0))),
                       ("const v=0", StatorArm::constant(0.0, 0.0))] {
        let l = st(arm).fuel.accel_schedule(&f, LO, HI, MARGIN, N_SCHED);
        assert_eq!(l.kappa.len(), l_bare.kappa.len(), "{tag}");
        assert_eq!(l.n_h.len(), l_bare.n_h.len(), "{tag}");
        for i in 0..l.kappa.len() {
            assert_eq!(l.kappa[i].to_bits(), l_bare.kappa[i].to_bits(), "{tag}: kappa[{i}]");
            assert_eq!(l.n_h[i].to_bits(), l_bare.n_h[i].to_bits(), "{tag}: n_h[{i}]");
        }
    }
}

/// Python `test_reduce_matched_cell_is_bit_for_bit_rung58_composite`. DISPATCH REDUCE: rung 59
/// adds cells BESIDE rung 58's; it does not perturb them.
#[test]
fn test_reduce_matched_cell_is_bit_for_bit_rung58_composite() {
    let f = flight();
    let m = st(StatorArm::scheduled_lp(sched()));
    let l_bare = m.at_stator(StatorArm::default())
        .fuel.accel_schedule(&f, LO, HI, MARGIN, N_SCHED);
    let r58 = m.composite_credit(&f, &ramp(0.5), Spool::Lp,
                                 &StatorLeg { accel: Some(&l_bare), ..Default::default() });
    let r59 = matched(StatorArm::scheduled_lp(sched()), Spool::Lp);
    for (tag, a, b) in [("neither", &r58.cells.neither, &r59.cells.neither),
                        ("stator", &r58.cells.stator, &r59.cells.stator),
                        ("fuel", &r58.cells.fuel, &r59.cells.fuel),
                        ("both/both_bare_leg", &r58.cells.both, &r59.cells.both_bare_leg)] {
        for (key, x, y) in [("m_i", a.m_i, b.m_i), ("m_phi", a.m_phi, b.m_phi),
                            ("s", a.s, b.s), ("v", a.v, b.v),
                            ("min_phi", a.min_phi, b.min_phi),
                            ("fuel_removed", a.fuel_removed, b.fuel_removed)] {
            assert_eq!(x.to_bits(), y.to_bits(), "{tag}.{key}: {x} vs {y}");
        }
        assert_eq!(a.npts, b.npts, "{tag}.npts");
    }
    assert_eq!(r58.credit_bare.to_bits(), r59.credit_bare.to_bits());
    assert_eq!(r58.interaction.to_bits(), r59.interaction_bare_leg.to_bits());
}

/// Python `test_reduce_rung58_readers_untouched`.
#[test]
fn test_reduce_rung58_readers_untouched() {
    let f = flight();
    let m = st(StatorArm::scheduled_lp(sched()));
    let l = m.at_stator(StatorArm::default()).fuel.accel_schedule(&f, LO, HI, MARGIN, N_SCHED);
    let d = m.engagement_shift(&f, &ramp(0.5), &StatorLeg { accel: Some(&l), ..Default::default() });
    assert_eq!(d.leg, LegKind::Accel);
    assert!(d.rel_limited.abs() < 0.01,
            "rung 58's 0.16 %, at this coarser grid: {}", d.rel_limited);
}

/// Python `test_cycle_untouched_by_rung59_bit_for_bit_rung6`.
#[test]
fn test_cycle_untouched_by_rung59_bit_for_bit_rung6() {
    let f = flight();
    let single = || {
        build_turbojet(Gas::reacting_equilibrium(), 3.0 * 6.0, 1500.0, 50_000.0, Losses {
            pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
            pi_n: 0.98, ..Losses::default()
        }).run(&f, 50.0)
    };
    let a = single();
    st(StatorArm { vsv_hp: V_HP, ..Default::default() })
        .matched_credit(&f, &ramp(0.5).with_ds(0.02), MARGIN, Spool::Lp, N_SCHED);
    let b = single();
    assert_eq!(a.performance.specific_thrust.to_bits(),
               b.performance.specific_thrust.to_bits());
    assert_eq!(a.performance.tsfc.to_bits(), b.performance.tsfc.to_bits());
}

// =============================================================================================
// THE GUARDS
// =============================================================================================

/// Python `test_synthetic_leg_refuses_a_margin_mismatch`. The splice exists to isolate the
/// abscissa from the ordinate; splicing two tables of DIFFERENT margins would reintroduce the
/// very leg-change it excludes.
#[test]
fn test_synthetic_leg_refuses_a_margin_mismatch() {
    let f = flight();
    let bare = st(StatorArm::default());
    let a = bare.fuel.accel_schedule(&f, LO, HI, 0.25, N_SCHED);
    let b = bare.fuel.accel_schedule(&f, LO, HI, 0.40, N_SCHED);
    let _ok: AccelSchedule = ScheduledStatorCore::synthetic_leg(&a, &a); // same margin: fine
    refuses("ONE schedule margin", move || {
        ScheduledStatorCore::synthetic_leg(&a, &b);
    });
}

/// Python `test_matched_credit_needs_an_armed_stator`.
#[test]
fn test_matched_credit_needs_an_armed_stator() {
    refuses("ARMED stator", || {
        st(StatorArm::default())
            .matched_credit(&flight(), &ramp(0.5), MARGIN, Spool::Lp, N_SCHED);
    });
}

// =============================================================================================
// THE FINDINGS
// =============================================================================================

/// Python `test_p1_the_ordinate_is_a_function_of_Tt4_alone` (`slow`). Checked on the three
/// factors `kappa_ss` is BUILT from rather than asserted.
///
/// **NOT to the last bit, deliberately**: `equilibrium`'s Newton converges to a tolerance, so a
/// nonzero setting lands ~1e-13 away. Tuple equality is claimed only at `v = 0` (the reduce
/// above); asserting it here would claim more than the solver can deliver.
#[test]
fn test_p1_the_ordinate_is_a_function_of_tt4_alone() {
    let f = flight();
    for (tag, arm) in [("LP const", StatorArm::constant(V, 0.0)),
                       ("LP sched", StatorArm::scheduled_lp(sched())),
                       ("HP const", StatorArm { vsv_hp: V_HP, ..Default::default() })] {
        let d = st(arm).schedule_invariance(&f, LO, HI, MARGIN, N_SCHED);
        assert!(d.d_ordinate < 1e-12, "{tag}: {}", d.d_ordinate);
        for row in &d.chain {
            for (key, x) in [("d_Tt25", row.d_tt25), ("d_Tt3", row.d_tt3), ("d_f", row.d_f),
                             ("d_mfp", row.d_mfp), ("d_ratio", row.d_ratio),
                             ("d_kappa", row.d_kappa)] {
                assert!(x.abs() < 1e-12, "{tag}: Tt4 = {}, {key} = {x}", row.tt4);
            }
        }
    }
}

/// Python `test_p1_the_abscissa_is_what_splits_the_two_spools` (`slow`). THE SPLIT: an LP stator
/// cannot move `n_H(Tt4)` — rung 39's ONE ARROW — so its whole table is invariant. An HP stator
/// moves the face itself, so the SAME CURVE comes back RE-INDEXED.
#[test]
fn test_p1_the_abscissa_is_what_splits_the_two_spools() {
    let f = flight();
    for (tag, arm) in [("LP const", StatorArm::constant(V, 0.0)),
                       ("LP sched", StatorArm::scheduled_lp(sched()))] {
        let d = st(arm).schedule_invariance(&f, LO, HI, MARGIN, N_SCHED);
        assert!(d.d_abscissa < 1e-12, "{tag}: {}", d.d_abscissa);
    }
    let d = st(StatorArm { vsv_hp: V_HP, ..Default::default() })
        .schedule_invariance(&f, LO, HI, MARGIN, N_SCHED);
    assert!(d.d_abscissa > 0.03, "measured 6.69 %: {}", d.d_abscissa);
    assert!(d.d_ordinate < 1e-12, "...with the ordinate STILL flat: {}", d.d_ordinate);
}

/// Python `test_p1_lp_stator_matching_is_a_no_op` (`slow`). RUNG 58's CONCESSION, DISCHARGED AS
/// VACUOUS: it ran an LP stator, so the leg it derived once on the bare machine already WAS the
/// matched leg.
#[test]
fn test_p1_lp_stator_matching_is_a_no_op() {
    for arm in [StatorArm::scheduled_lp(sched()), StatorArm::constant(V, 0.0)] {
        let d = matched(arm, Spool::Lp);
        assert!(d.delta_match.abs() < 1e-12, "{}", d.delta_match);
        assert!((d.interaction_matched - d.interaction_bare_leg).abs() < 1e-12,
                "{} vs {}", d.interaction_matched, d.interaction_bare_leg);
        assert!((d.s_eng_matched - d.s_eng_bare_leg).abs() < 1e-12,
                "{} vs {}", d.s_eng_matched, d.s_eng_bare_leg);
        assert!((d.removed_matched - d.removed_bare_leg).abs() < 1e-12,
                "{} vs {}", d.removed_matched, d.removed_bare_leg);
    }
}

/// Python `test_p2_the_abscissa_carries_all_of_it` (`slow`). THE ISOLATION, and the answer to
/// "you just swapped in a tighter schedule": splice the two tables and the ARMED index with the
/// BARE values reproduces the matched leg. Measured 100.00 % / 0.00 %.
#[test]
fn test_p2_the_abscissa_carries_all_of_it() {
    let d = matched(StatorArm { vsv_hp: V_HP, ..Default::default() }, Spool::Lp);
    assert!(d.delta_match > 1e-3, "a real effect to decompose: {}", d.delta_match);
    assert!((d.abscissa_share - 1.0).abs() < 1e-6, "{}", d.abscissa_share);
    assert!(d.ordinate_share.abs() < 1e-6, "{}", d.ordinate_share);
}

/// Python `test_p3_an_unmatched_schedule_manufactures_an_interaction` (`slow`). THE PRACTICAL
/// RESULT. The gate gives the ORDER and the SIGN headroom rather than pinning the weakest
/// measured row — rung 58's discipline: gate the claim, not the boundary value.
#[test]
fn test_p3_an_unmatched_schedule_manufactures_an_interaction() {
    let lp = matched(StatorArm { vsv_hp: V_HP, ..Default::default() }, Spool::Lp);
    assert!(lp.interaction_bare_leg.abs() > 5.0 * lp.interaction_matched.abs(),
            "{} vs 5x {}", lp.interaction_bare_leg.abs(), lp.interaction_matched.abs());
    let hp = matched(StatorArm { vsv_hp: V_HP, ..Default::default() }, Spool::Hp);
    assert!(hp.interaction_bare_leg < 0.0 && 0.0 < hp.interaction_matched,
            "of the WRONG SIGN on the spool carrying the stator: {} {}",
            hp.interaction_bare_leg, hp.interaction_matched);
}

/// Python `test_the_clamp_blocker_stays_clear` (`slow`). THE STANDING BLOCKER:
/// `AccelSchedule::cap` CLAMPS outside its abscissa bracket, and this rung RE-INDEXES that very
/// abscissa. Python iterates a dict of audits; Rust names the same three as fields.
#[test]
fn test_the_clamp_blocker_stays_clear() {
    for arm in [StatorArm::scheduled_lp(sched()),
                StatorArm { vsv_hp: V_HP, ..Default::default() }] {
        let d = matched(arm, Spool::Lp);
        let audits: [(&str, &ClampAudit); 3] = [("fuel", &d.audit_fuel),
                                                ("both_bare_leg", &d.audit_both_bare_leg),
                                                ("both_matched", &d.audit_both_matched)];
        for (tag, a) in audits {
            assert_eq!(a.clamped, 0, "{tag}: {a:?}");
            assert!(a.n_cuts > 0, "{tag}: the leg never binds — nothing was audited");
            assert!(a.lo < a.cut_lo && a.cut_hi < a.hi, "{tag}: {a:?}");
        }
    }
}
