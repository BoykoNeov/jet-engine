//! SLICE AH step 2 — **RUNG 77's INSTRUMENT, ITS THREE RESIDUALS, AND THE LATE-BINDING TRAP THE
//! SOURCE RECORDS SHIPPING ONCE.**
//!
//! # WHAT THIS FILE GATES, AND WHY IT IS NOT A PORT OF `tests/test_rung77.py`
//!
//! The ported suite lands later in the slice. What is gated here is the set of claims rung 77's own
//! docstrings PRE-REGISTER about the bodies this step writes:
//!
//! * the instrument — `1 − G_a'` IS rung 76's `c`, differenced at the SAME step so the roundoff
//!   cancels and the check tests the ALGEBRA;
//! * non-vacuity — the three normalised slopes are three quantities and not one in three costumes,
//!   which is what makes a knob-less rung's reduce a test rather than a tautology;
//! * the implicit function theorem, per leg, by two independent computations;
//! * the ordering, and the half of it that has to hold alone.
//!
//! # AND ONE GATE IS PROVED LOAD-BEARING BY BUILDING THE DEFECT, WHICH STEP 1 COULD NOT DO
//!
//! `slice_ah_cells.rs`'s header discloses that its pointer-identity gate asserts the four swaps and
//! does **not** demonstrate that a dropped one would pass everything else, because the cascade
//! builder it would need is `pub(crate)`. That limitation is a property of THAT defect, not of this
//! one. Rung 77's late-binding trap is module-local and every piece of it is `pub`, so
//! [`a_hoisted_residual_reads_the_closed_valve_plant_and_G_q_is_EXACTLY_zero`] constructs the
//! defect Python's own docstring records shipping — residuals built inside the frozen block and
//! evaluated after it — and measures its signature: `G_q` identically `0.0` and a relative error of
//! exactly `1.0`. **Then it measures the shipped reader at the same point and finds neither.**
//! A gate that asserts `ift_err` is small is only worth something if the failure it excludes is
//! reachable; this file shows it is, rather than asserting that it is.
//!
//! # NOTHING HERE READS A GOLDEN
//!
//! Every bar is either a same-run difference, a discrete ordering, or one of the tolerances
//! `tests/test_rung77.py` states in its own text (`c_err < 3e-9`, `sep > 1e-2`, `ift_err < 3e-8`,
//! `n >= 5`, and the three magnitude bars in § 1's stiffness column, § 1's governor column and
//! § 2's phi-over-gov factor). Those are quoted from the suite as BARS; no value is transcribed
//! from a run.
//!
//! # WHAT IS **NOT** SHOWN HERE — and it would be a weaker file if this were left implicit
//!
//! **No oracle comparison runs at this step.** These bodies are pinned by bars; they have NOT been
//! shown to compute PyPy's numbers, and bit-equality is the oracle step's job. Until then a
//! UNIFORM SCALE ERROR is the failure this file has to work hardest to see: `c_err` differences two
//! readings that would share one, `ift_err` compares two computations that would share one, `sep`
//! scales with one, and the raw-slope chain is a ladder of ratios. That is why
//! [`the_accel_column_is_rung_76_section_3s_measured_gain`] is here — it is the only ABSOLUTE bar
//! available before the oracle, and it is read by an independent route.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::PointExtra;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::sensed_cap::C_AT_REL;
use turbojet::shared_actuator::riding4;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::stiffness_ledger::{
    build_stiffness_ledger_cascade, ledger_march, leg_slopes, residuals, set_point_gains, Leg,
    SLOPE_AT_REL,
};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{MarchedBleed, MarchedStator};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung77.py`'s module constants, which are rung 76's unchanged. This rung declares no
// knob, so it contributes no number of its own.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const SM: f64 = PHI / FLOOR - 1.0;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const TT4_MAX: f64 = 1200.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const MARGIN: f64 = 0.10;
const EVERY: usize = 8;
const DQ: f64 = 1e-5;

// The four bars `tests/test_rung77.py` states in its own text.
const C_ERR_BAR: f64 = 3e-9;
const SEP_BAR: f64 = 1e-2;
const IFT_BAR: f64 = 3e-8;
const N_BAR: usize = 5;

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

/// The valve AND the stator loop — Python's `_rig(..., inc=False)`.
fn arm() -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    }
}

fn ledger() -> ScheduledStatorCore {
    match build_stiffness_ledger_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm())
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

// ---------------------------------------------------------------------------------------------
// THE INSTRUMENT
// ---------------------------------------------------------------------------------------------

/// `_slope_at`'s step is `_c_at`'s, and § 1's whole instrument rests on that.
///
/// The two are separate literals in the source and separate constants in the port — the deliberate
/// duplication the COPY-vs-REDERIVATION rule protects. What must not be silent is a later slice
/// moving one of them: § 1 differences `1 − G_a'` against `c` and the roundoff cancels ONLY because
/// the two readings share a step. So the binding is asserted here rather than enforced by a shared
/// constant, which would have deleted the source's own shape.
#[test]
fn the_slope_step_is_rung_76s_own_and_that_is_what_makes_the_instrument_a_check_on_the_algebra() {
    assert_eq!(SLOPE_AT_REL, C_AT_REL,
               "rung 77 § 1 differences two readings taken at the SAME step so their roundoff \
                cancels. If these ever diverge, `c_err` stops testing the algebra and starts \
                testing the arithmetic — and it would still pass, just weakly.");
}

/// P1 — **`1 − G_a'` IS rung 76's `c`.**
#[test]
fn the_instrument_reproduces_rung_76s_c() {
    let s = leg_slopes(&ledger(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R,
                       SETTLE, DS, V_MAX, EVERY);
    assert!(s.n >= N_BAR, "n = {}", s.n);
    let c_err = s.c_err.expect("rows");
    assert!(c_err < C_ERR_BAR, "c_err = {c_err:e}");
}

/// P5 — **ARM 1 MUST BE A TEST, NOT A TAUTOLOGY.**
///
/// A knob-less rung cannot gate its reduce on a knob differing, so it gates on the ledger having
/// three DISTINCT columns: a reader whose three normalised slopes agreed would pass a bit-for-bit
/// reduce and mean nothing. This is the same shape as the port's recorded *a ported test can go
/// VACUOUS*, asked of a rung that has no other arm to compare against.
#[test]
fn the_three_normalised_slopes_are_three_quantities_and_not_one_in_three_costumes() {
    let s = leg_slopes(&ledger(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R,
                       SETTLE, DS, V_MAX, EVERY);
    let sep = s.sep.expect("rows");
    assert!(sep > SEP_BAR, "sep = {sep:e}");

    // § 1.1 — the RAW slopes are orders apart BECAUSE they are different physical quantities, and
    // that is the evidence for the headline's second half: `Tt4_max` and `phi_lim` are constants,
    // so those two legs have no `1` to subtract from.
    let gw = s.gw.expect("rows");
    let (a, g, p) = (gw[Leg::Accel as usize], gw[Leg::Gov as usize], gw[Leg::Phi as usize]);
    assert!(a.1 < 1.0 && 1.0 < p.0 && p.0 < p.1 && p.1 < 1e3 && 1e3 < g.0,
            "accel {a:?}  gov {g:?}  phi {p:?}");
}

/// **THE ONE ABSOLUTE BAR IN THIS FILE, AND IT IS HERE BECAUSE EVERY OTHER GATE SURVIVES A
/// UNIFORM SCALE ERROR.**
///
/// `c_err` differences two readings that would share such an error; `ift_err` compares two
/// computations that would share it; `sep` scales with it; the raw-slope chain above is a ladder of
/// RATIOS. So a reader whose `norm`/`stiff` column were uniformly wrong by a factor would pass
/// every one of them. `tests/test_rung77.py` states the fix and this ports it: read by an
/// INDEPENDENT route — `1/G_a'` off one residual, never through rung 76's `solve_gain` — the accel
/// leg's stiffness IS rung 76 § 3's measured gain, `1.22799 … 1.24573`.
///
/// This is the port's recorded *a residual needs an ABSOLUTE bar*, and it is the only absolute
/// check available before the oracle step.
#[test]
fn the_accel_column_is_rung_76_section_3s_measured_gain() {
    let s = leg_slopes(&ledger(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R,
                       SETTLE, DS, V_MAX, EVERY);
    let (lo, hi) = s.stiff.expect("rows")[Leg::Accel as usize];
    assert!(1.22 < lo && lo < 1.24 && 1.24 < hi && hi < 1.26, "stiff[accel] = ({lo}, {hi})");

    // P7 / D5 — nothing in this family pins `Tt4` at a fixed fuel, so the governor has NO route to
    // `G_w = 0`. A second bar with a magnitude in it, from the same suite.
    let gov = s.norm.expect("rows")[Leg::Gov as usize];
    assert!(gov.0 > 0.5, "norm[gov] = {gov:?}");
}

// ---------------------------------------------------------------------------------------------
// § 2 — THE CURRENCY, AND THE TRAP
// ---------------------------------------------------------------------------------------------

/// P2 / D1 — **two computations of one number.**
///
/// `direct` re-solves each leg's whole set point at `q ± dq`; `ift` differences the residual's two
/// partials separately at the unperturbed set point. A reader that computed `ift` and called it
/// `direct` would be rung 70's gate computing its own formula twice.
#[test]
fn the_implicit_function_theorem_holds_per_leg() {
    let g = set_point_gains(&ledger(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R,
                            SETTLE, DS, V_MAX, DQ, EVERY);
    assert!(g.n >= N_BAR, "n = {}", g.n);
    let e = g.ift_err.expect("rows");
    assert!(e < IFT_BAR, "ift_err = {e:e}");
}

/// P3 — in the legal currency the ordering is `accel < gov < phi`, at every point, and phi is on
/// top wherever it is LIVE.
#[test]
fn the_phi_leg_is_the_stiffest_and_the_order_is_stable() {
    let g = set_point_gains(&ledger(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R,
                            SETTLE, DS, V_MAX, DQ, EVERY);
    assert_eq!(g.order.expect("rows"), [Leg::Accel, Leg::Gov, Leg::Phi], "{:?}", g.order);
    assert!(g.order_stable.expect("rows"), "the ordering must not turn over along the ramp");
    assert!(g.phi_top, "wherever phi is live it must be the LARGEST gain");

    // ... and by a MARGIN, not by a hair — the suite's own factor, which is a magnitude claim and
    // not another ordering.
    let gain = g.gain.expect("rows");
    let (phi, gov) = (gain[Leg::Phi as usize], gain[Leg::Gov as usize]);
    assert!(phi.0.abs() > 20.0 * gov.1.abs(), "phi {phi:?} against gov {gov:?}");
}

// ---------------------------------------------------------------------------------------------
// THE DEFECT, BUILT — **the demonstration step 1 could not do for its own gate**
// ---------------------------------------------------------------------------------------------

/// **THE LATE-BINDING TRAP, CONSTRUCTED AND MEASURED.**
///
/// Rung 77's `_residuals` docstring records that § 2's first version built the residual closures
/// inside a `_b_state = q` block and evaluated them after the `finally` had run — so both `q ± dq`
/// readings landed on the same closed-valve plant, `G_q` came back identically zero, and the
/// relative error against `direct` was a clean `1.000e+00`.
///
/// A Rust closure over `&FuelTransientCore` has that property for the identical reason: the two
/// frozen-state carriers are `Cell`s on the core it borrows, so the residual reads them when it is
/// CALLED. This test builds that exact arrangement out of the module's public pieces — nothing
/// defective is shipped in `src` — and asserts BOTH halves of the signature, then re-measures the
/// same point through the shipped reader and finds neither.
///
/// **That is what makes [`the_implicit_function_theorem_holds_per_leg`] load-bearing.** Without it,
/// `ift_err < 3e-8` is a bar with no demonstrated way to fail; with it, the failure it excludes is
/// exhibited, and `err == 1.0` exactly is a signature no tolerance could be tuned around.
#[test]
#[allow(non_snake_case)]
fn a_hoisted_residual_reads_the_closed_valve_plant_and_G_q_is_EXACTLY_zero() {
    let core = ledger();
    let flight = flight();
    let (m, surge, _lag, traj, accel) = ledger_march(
        &core, &flight, LO, HI, TT4_MAX, SM, TAUS, R, SETTLE, DS, V_MAX, false, MARGIN);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let p = pts.iter().step_by(EVERY).next().expect("the march produces riding points");
    let (a, h) = (p.nu_lp, p.nu_hp);
    let (q, v) = match p.extra {
        PointExtra::Demand { b, v, .. } => (b, v),
        PointExtra::Shared { b, v, .. } => (b, v),
        _ => panic!("the demand march carries `b`/`v` on every point"),
    };
    let sg = surge.as_ref();

    // THE SHIPPED READER, at this same point — the control.
    let g = set_point_gains(&core, &flight, LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R,
                            SETTLE, DS, V_MAX, DQ, EVERY);
    let row = &g.rows[0];
    let leg = Leg::Accel;
    let w0 = row.leg(leg).w;
    assert!(row.leg(leg).gq != 0.0,
            "the shipped reader must see a NON-ZERO G_q; a zero here IS the defect below");

    // THE DEFECT: the residuals are built inside the frozen block and the guards are dropped
    // before the closure is called. One `}` earlier than the shipped body puts it.
    let hoisted = |qq: f64| -> f64 {
        let r = {
            let _sb = MarchedBleed::set(&m.fuel.inner, qq);
            let _sv = MarchedStator::set(&m.fuel.inner, v);
            residuals(&m.fuel, &flight, a, h, Some(&accel), sg, Some(TT4_MAX))
            // <-- the two guards drop HERE, and the closure has not been called yet
        };
        let f = r.get(leg).expect("the accel leg is armed");
        f(w0).unwrap_or_else(|e| panic!("{}", e.0))
    };
    let gq_bad = (hoisted(q + DQ) - hoisted(q - DQ)) / (2.0 * DQ);
    assert_eq!(gq_bad, 0.0,
               "both readings must land on the SAME closed-valve plant — this is the defect's \
                first half, and it is an EXACT zero rather than a small one");

    // ... and its second half: the relative error against `direct` is exactly 1.
    let direct = row.leg(leg).direct;
    assert!(direct != 0.0, "the control needs a non-zero `direct` to divide by");
    let ift_bad = -gq_bad / row.leg(leg).gw;
    let err_bad = (direct - ift_bad).abs() / direct.abs();
    assert_eq!(err_bad, 1.0,
               "`ift` collapses to zero, so the error is `|direct|/|direct|` — the clean \
                1.000e+00 with no noise in it that gave the defect away in Python");

    // AND THE SHIPPED READER IS NOWHERE NEAR IT.
    assert!(row.leg(leg).err < IFT_BAR,
            "err = {:e} — the same quantity, on the body that rebuilds per block",
            row.leg(leg).err);
}
