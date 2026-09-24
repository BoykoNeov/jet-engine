//! RUNG 77 — **THE STIFFNESS LEDGER**: rung 76 § 8's third seam.
//!
//! Rung 76 § 3 found that writing rung 48's leg as a set-point solve multiplies its sensitivity to
//! every other state by `1/(1-c)`, and § 8 predicted that every other set-point solve in this
//! family has one and it has never been read.
//!
//! ```text
//! accel (48)  G_a(w) = w - cap(w)             G_a' = 1 - c        DIMENSIONLESS
//! gov   (46)  G_g(w) = Tt4(w) - Tt4_max       G_g' = dTt4/dw      K per kg/s
//! phi   (49)  G_s(w) = phi_lim - phi_lp(w)    G_s' = -dphi/dw     phi per kg/s
//! ```
//!
//! **THE HEADLINE: a set-point solve's sensitivity is a FORCING OVER A SLOPE, and `1/(1-c)` is the
//! SLOPE HALF of one leg.** `Tt4_max` and `phi_lim` are CONSTANTS, so the other two legs have a
//! STIFFNESS but can never have a GAIN. Rung 76 § 8's wording is REFUTED.
//!
//! Ported from `tests/test_rung77.py`: **17 collected tests, of which 3 carry `slow` there**
//! (`test_the_order_needs_the_dormancy_guard`, `test_c_never_approaches_one`,
//! `test_the_ledger_march_MOVES_and_keeps_THREE_loops_live`). **This file has 17**, one per test,
//! in the suite's order. Nothing is added: plan § 5.32 (v) measured **zero** shipped refusals at
//! rung 77 and zero refusal assertions in its suite, so there is no refusal section to write — P4
//! says the reduce is gated by VALUES and DISPATCH only, and this file is the values half.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, 0 ADDED, 0 COLLAPSED, 0 SPLIT BY PARAMETER
//!
//! | # | `tests/test_rung77.py` | here |
//! |---|---|---|
//! | 1 | `reduces_to_rung76_bit_for_bit` | [`reduces_to_rung76_bit_for_bit`] |
//! | 2 | `reduces_on_an_at_lever_rig` | [`reduces_on_an_at_lever_rig`] |
//! | 3 | `the_reduce_is_not_vacuous` | [`the_reduce_is_not_vacuous`] |
//! | 4 | `at_lever_carries_the_class` | [`at_lever_carries_the_class`] |
//! | 5 | `the_instrument_reproduces_rung76_c` | [`the_instrument_reproduces_rung76_c`] |
//! | 6 | `the_accel_column_is_rung76_section3_gain` | [`the_accel_column_is_rung76_section3_gain`] |
//! | 7 | `the_three_slopes_do_not_share_a_unit` | [`the_three_slopes_do_not_share_a_unit`] |
//! | 8 | `the_governor_slope_never_collapses` | [`the_governor_slope_never_collapses`] |
//! | 9 | `the_implicit_function_theorem_holds_per_leg` | [`the_implicit_function_theorem_holds_per_leg`] |
//! | 10 | `the_phi_leg_is_the_stiffest` | [`the_phi_leg_is_the_stiffest`] |
//! | 11 | `the_valve_sign_splits_across_the_legs` | [`the_valve_sign_splits_across_the_legs`] |
//! | 12 | `rung64_degeneracy_measured` | [`rung64_degeneracy_measured`] |
//! | 13 | `rung64_degeneracy_in_its_blunt_form` | [`rung64_degeneracy_in_its_blunt_form`] |
//! | 14 | `the_governor_is_the_control` | [`the_governor_is_the_control`] |
//! | 15 | `the_order_needs_the_dormancy_guard` | [`the_order_needs_the_dormancy_guard`] |
//! | 16 | `c_never_approaches_one` | [`c_never_approaches_one`] |
//! | 17 | `the_ledger_march_MOVES_and_keeps_THREE_loops_live` | [`the_ledger_march_moves_and_keeps_three_loops_live`] |
//!
//! # WHAT OVERLAPS `slice_ah_laws.rs` / `slice_ah_ledger.rs`, AND WHY IT IS PORTED ANYWAY
//!
//! Steps 2 and 3 already gate close relatives of **5–16**. They are ported here regardless, on
//! `rung72.rs`–`rung76.rs`'s precedent: this file's contract is a **1:1 map of the shipped
//! suite**, and a hole in it is invisible to every count on both sides.
//!
//! **AND THE STEP FILES' EXTRA PINS ARE NOT COPIED IN.** They assert measured exact counts the
//! suite never states (3 of 24 cells inverting, every one at `margin = 0.40`). Those are the step
//! files' own claims and stay there. What is here is what `tests/test_rung77.py` asserts, spelled
//! the way it spells it — including Python TRUTHINESS: where the suite writes `assert
//! gains["order_stable"]`, this file writes `== Some(true)`, so an absent reading fails rather than
//! passing as "not false".
//!
//! # THE READERS' UNSPELLED DEFAULTS ARE TYPED HERE, FROM `engine.py`'s SIGNATURES
//!
//! The suite calls every reader with four positionals plus at most `phi_lim`/`margin`, so the rest
//! come from the signatures (`engine.py:19799`, `:19856`, `:19944`, `:20009`). They are typed here
//! from THERE and not from the step files — and not promoted to `pub const`s in
//! `stiffness_ledger.rs` either, unlike § 5.31.5 (l): that section's reason was *this file is
//! their only call site*, and here it is not (`slice_ah_laws.rs` and `slice_ah_ledger.rs` already
//! pass them). A module constant would be a third home for the same numbers, not a single one.
//!
//! **`stiffness_ledger(FLIGHT, LO, HI, TT4_MAX)`'s FOURTH ARGUMENT BINDS A PARAMETER THE BODY NEVER
//! READS.** Python's `stiffness_ledger` takes `Tt4_max` positionally and then sweeps its own
//! `Tt4_maxes=(1180.0, 1200.0)`; the only `Tt4_max` in the body is the `Tt4_max=tmax` it passes
//! down. The port dropped the parameter. So the suite's `TT4_MAX` has no Rust counterpart at gates
//! 15/16, and the `1180.0` in [`TT4_MAXES`] is the signature's default and not a number this file
//! chose. Recorded rather than silently absorbed, because a reader comparing the two calls would
//! otherwise see an argument vanish.
//!
//! # THE THREE MODULE FIXTURES ARE `OnceLock`s
//!
//! `slopes`, `gains` and `singular` are module-scoped in Python. Each is one `_ledger_march` plus a
//! reader pass, so each is shared here through [`OnceLock`] exactly as `rung76.rs` shares its
//! `gains`. `get_or_init` over a `LazyLock` for `rung76.rs`'s reason: a panicking initialiser
//! leaves the cell UNSET and the next gate re-runs it, so a real failure surfaces in every gate
//! with its own message.
//!
//! The two `slow` gates each call `stiffness_ledger` themselves in Python, and so do these — a
//! 24-cell sweep twice. Python recomputes; so does the port.

use std::ptr::fn_addr_eq;
use std::sync::OnceLock;

use turbojet::anti_windup::{WINDUP_LAW_NONE, WINDUP_LAW_TRACK};
use turbojet::applied_reference::REF_LAW_APPLIED;
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::sensed_cap::{
    accel_for, build_sensed_cap_cascade, CAP_LAW_SENSED, CAP_LAW_SOLVE, R76,
};
use turbojet::shared_actuator::{SharedRigArm, REF_LAW_DEFAULT};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::stiffness_ledger::{
    build_stiffness_ledger_cascade, ledger_march, leg_slopes, set_point_gains, singular_limit,
    stiffness_ledger, Leg, LegSlopes, SetPointGains, SingularLimit, R77,
};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung77.py`'s module constants, verbatim.

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
const V_MAX: f64 = 0.20;
const TT4_MAX: f64 = 1200.0;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
const TAU_T: f64 = 0.05;
const MARGIN: f64 = 0.10;
const PHI_JAC: f64 = 0.80;
const PHI_BOTH: f64 = 0.76;

/// **THE DIFFERENCING FLOOR, measured rather than estimated** (spec § 2.2): `dq` swept over three
/// decades gives a textbook central-difference V, so the residual at the optimum is arithmetic,
/// not a gap. The anchor asked for `< 3e-9` and § 8 scores that tolerance REFUTED and the law
/// HELD.
const IFT_FLOOR: f64 = 3e-8;

// ---------------------------------------------------------------- the readers' unspelled defaults
//
// From `engine.py`'s signatures — see the module header for why typed here.

/// `leg_slopes`, `set_point_gains` and `singular_limit` all default `every = 8`.
const EVERY: usize = 8;
/// `set_point_gains`' `dq`.
const DQ: f64 = 1e-5;
/// `singular_limit`'s `spread`.
const SPREAD: f64 = 0.10;
/// `stiffness_ledger`'s four sweep axes and its own `every = 16`.
const PHI_LIMS: [f64; 2] = [0.76, 0.80];
const MARGINS: [f64; 3] = [0.05, 0.10, 0.40];
const TT4_MAXES: [f64; 2] = [1180.0, 1200.0];
const ARMS: [bool; 2] = [false, true];
const LEDGER_EVERY: usize = 16;

/// `PHI / FLOOR - 1.0` — the EXPRESSION Python spells, never a typed decimal.
fn sm_of(phi_lim: f64) -> f64 { phi_lim / FLOOR - 1.0 }

// ============================================================================== the fixtures

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
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

fn suite_arm(sm: f64, inc: bool) -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: if inc { None }
                    else { Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))) },
        stator_inc: if inc {
            Some(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S)))
        } else { None },
        ..Default::default()
    }
}

/// Which class Python's `_rig(design, cls, …)` constructs.
#[derive(Clone, Copy, Debug)]
enum Cls {
    /// `StiffnessLedgerTransient` — rung 77.
    Ledger,
    /// `SensedCapTransient` — rung 76, the parent.
    Sensed,
}

/// Python's `_rig` — five knobs, all by PLAIN ASSIGNMENT, on either class.
#[allow(clippy::too_many_arguments)]
fn rig(cls: Cls, sm: f64, inc: bool, coord: &'static str, ref_law: &'static str,
       law: &'static str, tau_t: Option<f64>, cap_law: &'static str) -> ScheduledStatorCore {
    let arm = suite_arm(sm, inc);
    let m = full_of(match cls {
        Cls::Ledger => build_stiffness_ledger_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm),
        Cls::Sensed => build_sensed_cap_cascade(
            design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm),
    });
    m.fuel.inner.lag_coord.set(coord);
    m.fuel.inner.ref_law.set(ref_law);
    m.fuel.inner.windup_law.set(law);
    m.fuel.inner.tau_t.set(tau_t);
    m.fuel.inner.cap_law.set(cap_law);
    m
}

/// Python's `_rig`'s default arm.
fn rig_default(cls: Cls, sm: f64) -> ScheduledStatorCore {
    rig(cls, sm, false, "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE)
}

/// Python's `_accel` — **THE SCHEDULE IS BUILT ON THE RIG THAT WILL MARCH IT** (rung 76 § 7's
/// trap), on a RUNG-77 rig whichever class then marches it.
fn accel(sm: f64) -> AccelSchedule {
    accel_for(&rig_default(Cls::Ledger, sm), &flight(), LO, HI, sm, TT4_MAX, TAUS, V_MAX, false,
              MARGIN)
}

fn ramp() -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS } }

/// Python's `_march(m, sm, acc)`.
fn march(m: &ScheduledStatorCore, sm: f64, acc: &AccelSchedule) -> Vec<FuelPoint> {
    let leg = StatorLeg {
        accel: Some(acc),
        surge: Some(Floor::Phi(SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm))),
        tt4_max: Some(TT4_MAX),
    };
    m.stator_march_scoped(&flight(), &ramp(), None, &leg,
                          &MarchScope { lag: Some(AsymmetricLag::new(TAU_ATT, TAU_REL)),
                                        tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT }).0
}

/// Python's `_keys(traj)` — `if k in p` is per point: a `clip` arm records no `w_fuel`/`w_gov`.
fn keys(traj: &[FuelPoint]) -> Vec<Vec<u64>> {
    traj.iter().map(|p| {
        let mut v = vec![p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(),
                         p.phi_lp.to_bits(), p.phi_hp.to_bits(), p.tt4.to_bits(),
                         p.mf.to_bits()];
        match p.extra {
            PointExtra::Shared { b, v: vv, .. } => { v.push(b.to_bits()); v.push(vv.to_bits()); }
            PointExtra::Demand { b, v: vv, w_fuel, w_gov, .. } => {
                v.push(b.to_bits());
                v.push(vv.to_bits());
                v.push(w_fuel.to_bits());
                v.push(w_gov.to_bits());
            }
            _ => panic!("rung-77's reduce compares marches that record `b` and `v`."),
        }
        v
    }).collect()
}

/// Python's module-scoped `slopes` fixture.
fn slopes() -> &'static LegSlopes {
    static S: OnceLock<LegSlopes> = OnceLock::new();
    S.get_or_init(|| leg_slopes(
        &rig_default(Cls::Ledger, sm_of(PHI_JAC)), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN,
        TAUS, false, R, SETTLE, DS, V_MAX, EVERY))
}

/// Python's module-scoped `gains` fixture.
fn gains() -> &'static SetPointGains {
    static G: OnceLock<SetPointGains> = OnceLock::new();
    G.get_or_init(|| set_point_gains(
        &rig_default(Cls::Ledger, sm_of(PHI_JAC)), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN,
        TAUS, false, R, SETTLE, DS, V_MAX, DQ, EVERY))
}

/// Python's module-scoped `singular` fixture.
fn singular() -> &'static SingularLimit {
    static S: OnceLock<SingularLimit> = OnceLock::new();
    S.get_or_init(|| singular_limit(
        &rig_default(Cls::Ledger, sm_of(PHI_JAC)), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN,
        TAUS, false, R, SETTLE, DS, V_MAX, SPREAD, EVERY))
}

/// Python's `stiffness_ledger(FLIGHT, LO, HI, TT4_MAX)` — see the module header for the fourth
/// argument, which has no counterpart here.
fn ledger_sweep() -> turbojet::stiffness_ledger::StiffnessLedger {
    stiffness_ledger(&rig_default(Cls::Ledger, sm_of(PHI_JAC)), &flight(), LO, HI, &PHI_LIMS,
                     &MARGINS, &TT4_MAXES, &ARMS, TAUS, R, SETTLE, DS, V_MAX, LEDGER_EVERY)
}

// ======================================================================================
// THE REDUCE SPINE — BY CONSTRUCTION. This rung overrides NO plant method, so every march
// it runs is rung 76's own code. NOT a tolerance, and NOT marked `slow`.
// ======================================================================================

/// Five cells plus the accel-armed `sensed` arm. The parent's methods are the ones that run.
#[test]
fn reduces_to_rung76_bit_for_bit() {
    let sm = sm_of(PHI_BOTH);
    let acc = accel(sm);
    for (coord, ref_law, law, tt, cap) in [
        ("clip", REF_LAW_APPLIED, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE),
        ("demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE),
        ("demand", REF_LAW_DEFAULT, WINDUP_LAW_TRACK, Some(TAU_T), CAP_LAW_SOLVE),
        ("demand", REF_LAW_APPLIED, WINDUP_LAW_TRACK, Some(TAU_T), CAP_LAW_SOLVE),
        ("demand-latched", REF_LAW_APPLIED, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE),
        ("demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None, CAP_LAW_SENSED),
    ] {
        let a = keys(&march(&rig(Cls::Ledger, sm, false, coord, ref_law, law, tt, cap), sm, &acc));
        let b = keys(&march(&rig(Cls::Sensed, sm, false, coord, ref_law, law, tt, cap), sm, &acc));
        assert_eq!(a, b, "{coord}|{ref_law}|{law}|{cap}");
    }
}

/// **THE ARM THAT COVERS THE OVERRIDE.** The gate above constructs both machines DIRECTLY, so it
/// never touches `at_lever` — the only method this rung overrides. But `_shared_rig` builds
/// through `at_lever`, and that is the path every reader in §§ 1–4 runs on. So the comparison is
/// re-taken on a rig OBTAINED FROM `_shared_rig`, marched with the `surge` and `lag` IT returns —
/// rebuilding those two from constants would delete half of what this gate compares.
///
/// Python's `assert isinstance(m, cls)` is a TABLE identity here, because in this port the class
/// is the hooks table: read through a field with `fn_addr_eq`, never `ptr::eq` on a `pub const`
/// (slice AE step 5's finding — every use site promotes a fresh copy).
#[test]
fn reduces_on_an_at_lever_rig() {
    let sm = sm_of(PHI_BOTH);
    let acc = accel(sm);
    let arm = SharedRigArm {
        sm, tau: TAU, tau_s: TAU_S, v_max: V_MAX, tt4_max: TT4_MAX, tau_att: TAU_ATT,
        tau_rel: TAU_REL, ..SharedRigArm::default()
    };
    let mut out = Vec::new();
    for cls in [Cls::Ledger, Cls::Sensed] {
        let base = rig_default(cls, sm);
        let (m, surge, lag) = (base.triple_hooks().shared_rig)(&base, &arm);
        let want = match cls { Cls::Ledger => R77.at_lever, Cls::Sensed => R76.at_lever };
        assert!(fn_addr_eq(m.fuel.inner.lever_hooks.at_lever, want),
                "`_shared_rig` on a {cls:?} machine must return a {cls:?} machine");
        // The rung-77 arm also needs its NEGATIVE half (see `at_lever_carries_the_class`). The
        // rung-76 arm is held STRICTER than Python, whose `isinstance(m, SensedCapTransient)`
        // would accept a rung-77 subclass: `base` was built as rung 76, so anything else is a
        // leak.
        if let Cls::Ledger = cls {
            assert!(!fn_addr_eq(m.fuel.inner.lever_hooks.at_lever, R76.at_lever),
                    "a rung-77 `_shared_rig` must not hand back the parent");
        }
        let leg = StatorLeg { accel: Some(&acc), surge, tt4_max: Some(TT4_MAX) };
        out.push(keys(&m.stator_march_scoped(
            &flight(), &ramp(), None, &leg,
            &MarchScope { lag, tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT }).0));
    }
    assert_eq!(out[0], out[1]);
}

/// **P5 — ARM 1 MUST BE A TEST, NOT A TAUTOLOGY.** A knob-less rung cannot gate its reduce on a
/// knob differing, so it gates on the LEDGER having three distinct columns.
#[test]
fn the_reduce_is_not_vacuous() {
    let s = slopes();
    assert!(s.n >= 5, "n = {}", s.n);
    let sep = s.sep.expect("the fixture has rows");
    assert!(sep > 1e-2, "sep = {sep:e}");
}

/// § 0.1 — the carried trap's FIFTEENTH face, and here the thing that would not travel is the
/// CLASS. Exactly the FIVE knobs Python compares — `_tau_t` is not among them, and adding it here
/// would be a gate the suite does not run.
#[test]
fn at_lever_carries_the_class() {
    let sm = sm_of(PHI_JAC);
    let m = rig_default(Cls::Ledger, sm);
    let arm = SharedRigArm {
        sm, tau: TAU, tau_s: TAU_S, v_max: V_MAX, tt4_max: TT4_MAX, ..SharedRigArm::default()
    };
    let (r, _, _) = (m.triple_hooks().shared_rig)(&m, &arm);
    // `isinstance(rig, StiffnessLedgerTransient)` is BOTH halves: it is the rung-77 table, and it
    // is NOT the parent's. The first half alone is vacuous under exactly the defect this gate
    // exists for — `R77.at_lever` re-aimed at rung 76's body makes `R77.at_lever` rung 76's, and
    // an equality against it then passes. Measured by injection at slice AH step 6.
    assert!(fn_addr_eq(r.fuel.inner.lever_hooks.at_lever, R77.at_lever),
            "`_shared_rig` must hand back a RUNG-77 machine");
    assert!(!fn_addr_eq(r.fuel.inner.lever_hooks.at_lever, R76.at_lever),
            "... and not a rung-76 one, which is what `isinstance` rejects");
    assert_eq!(r.fuel.inner.cap_law.get(), m.fuel.inner.cap_law.get(), "_cap_law");
    assert_eq!(r.fuel.inner.ref_law.get(), m.fuel.inner.ref_law.get(), "_ref_law");
    assert_eq!(r.fuel.inner.lag_coord.get(), m.fuel.inner.lag_coord.get(), "_lag_coord");
    assert_eq!(r.fuel.inner.windup_law.get(), m.fuel.inner.windup_law.get(), "_windup_law");
    assert_eq!(r.fuel.inner.ic_cap.get(), m.fuel.inner.ic_cap.get(), "_ic_cap");
}

// ======================================================================================
// § 1 — THE THREE SLOPES, AND THE INSTRUMENT
// ======================================================================================

/// P1 — `1 - G_a'` IS rung 76's `c`. Both readings share a step size so their roundoff cancels:
/// this gates the ALGEBRA, not agreement to eleven figures.
#[test]
fn the_instrument_reproduces_rung76_c() {
    let e = slopes().c_err.expect("the fixture has rows");
    assert!(e < 3e-9, "c_err = {e:e}");
}

/// § 1 — read by an INDEPENDENT route (`1/G_a'` from one residual, never `solve_gain`), the accel
/// leg's stiffness is rung 76 § 3's measured gain `1.22799 … 1.24573`.
#[test]
fn the_accel_column_is_rung76_section3_gain() {
    let (lo, hi) = slopes().stiff.expect("the fixture has rows")[Leg::Accel as usize];
    assert!(1.22 < lo && lo < 1.24 && 1.24 < hi && hi < 1.26, "({lo}, {hi})");
}

/// § 1.1 — the raw slopes are orders apart BECAUSE they are different physical quantities.
#[test]
fn the_three_slopes_do_not_share_a_unit() {
    let gw = slopes().gw.expect("the fixture has rows");
    let (a, g, p) = (gw[Leg::Accel as usize], gw[Leg::Gov as usize], gw[Leg::Phi as usize]);
    assert!(a.1 < 1.0 && 1.0 < p.0 && p.0 < p.1 && p.1 < 1e3 && 1e3 < g.0,
            "({a:?}, {g:?}, {p:?})");
}

/// P7 / D5 — nothing in this family pins `Tt4` at a fixed fuel, so the governor has NO route to
/// `G_w = 0`.
#[test]
fn the_governor_slope_never_collapses() {
    let g = slopes().norm.expect("the fixture has rows")[Leg::Gov as usize];
    assert!(g.0 > 0.5, "{g:?}");
}

// ======================================================================================
// § 2 — dw*/dq, THE CURRENCY ALL THREE LEGS SHARE
// ======================================================================================

/// P2 / D1 — `direct` re-solves the whole set point at `q ± dq`; `ift` differences the two
/// partials separately. Two computations of one number.
#[test]
fn the_implicit_function_theorem_holds_per_leg() {
    let e = gains().ift_err.expect("the fixture has rows");
    assert!(e < IFT_FLOOR, "ift_err = {e:e}");
}

/// P3 — in the legal currency, `‖dw*/dq‖` is ordered accel < gov < φ at every point, and the φ
/// leg is ~50× the governor.
#[test]
fn the_phi_leg_is_the_stiffest() {
    let g = gains();
    assert_eq!(g.order, Some([Leg::Accel, Leg::Gov, Leg::Phi]), "{:?}", g.order);
    assert_eq!(g.order_stable, Some(true), "Python's bare `assert gains[\"order_stable\"]`");
    let gain = g.gain.expect("the fixture has rows");
    assert!(gain[Leg::Phi as usize].0.abs() > 20.0 * gain[Leg::Gov as usize].1.abs(),
            "phi {:?} gov {:?}", gain[Leg::Phi as usize], gain[Leg::Gov as usize]);
}

/// § 2.1 — the ONE lever that loosens the leg watching φ TIGHTENS both fuel-side caps.
#[test]
fn the_valve_sign_splits_across_the_legs() {
    for r in &gains().rows {
        assert!(r.leg(Leg::Accel).direct < 0.0 && r.leg(Leg::Gov).direct < 0.0, "{}", r.s);
        assert!(r.leg(Leg::Phi).direct > 0.0, "{}", r.s);
    }
}

// ======================================================================================
// § 3 — THE SINGULAR LIMIT, AND RUNG 64's DERIVATION MEASURED
// ======================================================================================

/// P6 — closing the valve's loop kills the φ leg's residual slope. The gate sits ABOVE the
/// roundoff floor (`eps·φ/dw ≈ 1.8e-08`), not at the measurement.
///
/// Python's `max(singular["phi_closed"], 1e-30)` is a two-argument `max`: argument 0 kept unless
/// strictly beaten, spelled out below so a NaN reads the way it does there.
#[test]
fn rung64_degeneracy_measured() {
    let s = singular();
    let open = s.phi_open.expect("the fixture has rows");
    let closed = s.phi_closed.expect("the fixture has rows");
    assert!(open > 1.0, "phi_open = {open:e}");
    assert!(closed < 1e-6, "phi_closed = {closed:e}");
    let den = if 1e-30 > closed { 1e-30 } else { closed };
    assert!(open / den > 1e6, "{open:e} / {closed:e}");
}

/// § 3 — under the riding valve `φ_lp` IS `φ_lim` at 0.9·w, at w and at 1.1·w.
#[test]
fn rung64_degeneracy_in_its_blunt_form() {
    let s = singular();
    let off = s.phi_off.expect("the fixture has rows");
    let spread = s.phi_spread.expect("the fixture has rows");
    assert!(off < 1e-12, "phi_off = {off:e}");
    assert!(spread < 1e-12, "phi_spread = {spread:e}");
}

/// § 3.1 — WITHOUT THIS THE RUNG IS INADMISSIBLE: the governor, read the same way at the same
/// states, moves 2% and not 100%.
#[test]
fn the_governor_is_the_control() {
    let s = singular();
    let rel = s.gov_rel.expect("the fixture has rows");
    let open = s.gov_open.expect("the fixture has rows");
    assert!(rel < 0.1, "gov_rel = {rel:e}");
    assert!(open > 1e4, "gov_open = {open:e}");
}

// ======================================================================================
// § 4 — THE ORDER OVER THE ARMS, AND THE GUARD THAT MAKES IT LEGAL (`slow` in Python)
// ======================================================================================

/// P4 — REFUTED RAW, HELD GUARDED, and both halves asserted so the correction cannot be quietly
/// dropped.
#[test]
fn the_order_needs_the_dormancy_guard() {
    let s4 = ledger_sweep();
    assert!(s4.n_live == s4.n_cells && s4.n_cells == 24, "{} / {}", s4.n_live, s4.n_cells);
    // the refutation, asserted as a refutation
    assert!(!s4.order_invariant, "{:?}", s4.orders);
    assert_eq!(s4.orders.len(), 2);
    // ... and the guarded reading, which is what § 4 claims
    assert!(s4.phi_top, "the phi leg must be top in every cell");
    let mut want = vec![vec![Leg::Accel, Leg::Gov, Leg::Phi], vec![Leg::Gov, Leg::Phi]];
    let mut got = s4.guarded_orders.clone();
    got.sort_by_key(|o| o.iter().map(|l| *l as usize).collect::<Vec<_>>());
    got.dedup();
    want.sort_by_key(|o| o.iter().map(|l| *l as usize).collect::<Vec<_>>());
    assert_eq!(got, want, "Python compares the SET of guarded orderings");
    // the second is the first with the DORMANT leg removed, not re-ordered
    let sep = s4.sep.expect("live cells");
    let ift = s4.ift_err.expect("live cells");
    assert!(sep > 1e-2 && ift < IFT_FLOOR, "sep = {sep:e}, ift_err = {ift:e}");
    let gn = s4.gov_norm.expect("live cells");
    assert!(gn > 0.5, "gov_norm = {gn}");
}

/// P8 — `c -> 1` is the divergent-gain limit, and no setting this family already has gets near it.
#[test]
fn c_never_approaches_one() {
    let s4 = ledger_sweep();
    assert!(s4.c_max < 0.35, "c_max = {}", s4.c_max);
}

// ======================================================================================
// THE MARCH AUDIT — rung 79's gap seam, checked from the other end (`slow` in Python).
// ======================================================================================

fn b_cmd_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { b_cmd, .. } | PointExtra::Demand { b_cmd, .. } => b_cmd,
        _ => panic!("the ledger march records `b_cmd` on every point."),
    }
}

fn required_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { required, .. } | PointExtra::Demand { required, .. } => required,
        _ => panic!("the ledger march records `required` on every point."),
    }
}

/// Python's `p.get("v_regime")` — `None` where the key is absent.
fn v_regime_of(p: &FuelPoint) -> Option<Regime> {
    match p.extra {
        PointExtra::Shared { v_regime, .. } | PointExtra::Demand { v_regime, .. } => v_regime,
        _ => None,
    }
}

/// `_ledger_march` marches in **CLIP** at `PHI_JAC`'s wall — the arrested rows' own wall — and is
/// nevertheless the liveliest march in the family. `b_max` is read off the OUTER rig, which is
/// Python's `m.bleed_lim.b_max`.
#[test]
fn the_ledger_march_moves_and_keeps_three_loops_live() {
    let sm = sm_of(PHI_JAC);
    let m = rig_default(Cls::Ledger, sm);
    let traj = ledger_march(&m, &flight(), LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, false,
                            MARGIN).3;
    assert!(traj.len() > 300, "{}", traj.len());
    let nu: Vec<f64> = traj.iter().map(|p| p.nu_lp).collect();
    let (nmin, nmax) = (nu.iter().copied().fold(f64::INFINITY, f64::min),
                        nu.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    assert!((nmax - nmin) / nmin > 1e-2, "({nmin}, {nmax})");
    let t4: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
    let (tmin, tmax) = (t4.iter().copied().fold(f64::INFINITY, f64::min),
                        t4.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    assert!(tmax - tmin > 100.0, "({tmin}, {tmax})");
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms the valve").b_max;
    assert!(traj.iter().filter(|p| required_of(p) > 0.0).count() > 300);
    assert!(traj.iter().filter(|p| { let b = b_cmd_of(p); 0.0 < b && b < b_max }).count() > 50);
    assert!(traj.iter().filter(|p| v_regime_of(p) == Some(Regime::Riding)).count() > 50);
    // the phi leg is live and the CLIP coordinate tracks with error, so the droop crosses it
    let pmin = traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min);
    assert!(pmin < PHI_JAC, "{pmin}");
    assert!(pmin > 0.780, "the phi leg would be dormant below the droop: {pmin}");
}
