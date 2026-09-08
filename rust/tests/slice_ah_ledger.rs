//! SLICE AH step 3 — **RUNG 77 § 3 AND § 4: THE CLOSED VALVE LOOP, AND THE ORDER OVER THE ARMS.**
//!
//! # WHAT THIS FILE GATES
//!
//! Step 2 shipped rung 77's instrument, its three residuals and readers § 1/§ 2. This step ships
//! the last two — [`singular_limit`], which reads the phi leg with the valve's loop CLOSED, and
//! [`stiffness_ledger`], which re-takes § 2's ordering over 24 corners. Every bar below is quoted
//! from `tests/test_rung77.py`'s own text (`phi_open > 1.0`, `phi_closed < 1e-6`, the `1e6` ratio,
//! `phi_off`/`phi_spread < 1e-12`, `gov_rel < 0.1`, `gov_open > 1e4`, `sep > 1e-2`,
//! `ift_err < 3e-8`, `gov_norm > 0.5`, `c_max < 0.35`) or is a discrete count that file states
//! (24 cells, two raw orderings, the two guarded ones). **No value is transcribed from a run.**
//!
//! # THE FILE'S OWN FINDING — **THE LATE BINDING IS § 2's DEFECT AND § 3's INSTRUMENT**
//!
//! `_residuals`' Python docstring closes: *"Splitting the closures out makes the fix structural: § 2
//! rebuilds them at each `qq` INSIDE that `qq`'s own block, **so a residual can only ever be
//! evaluated on the plant it was built for**."* The clause after the colon is quantified over every
//! residual in the class, and `singular_limit` — 195 lines further down the same class — breaks it
//! on purpose: it builds the three residuals inside the frozen block, drops the block, re-freezes
//! the **stator only**, and re-evaluates the SAME closures. That second reading IS rung 64's
//! measurement. The split is therefore not structural; it is per-call-site discipline, and the two
//! call sites want opposite things from one property.
//!
//! That is step 2's `"only"` finding in a second costume — **a sentence quantified over a SET,
//! written before the set had its last member** — and it is repaired where it stands, in
//! [`Residuals`](turbojet::stiffness_ledger::Residuals)' doc comment.
//!
//! [`the_closed_reading_is_a_function_of_the_CELL_and_re_arming_the_valve_guard_restores_the_open_one`]
//! measures it rather than asserting it: one closure, one set point, three readings that differ
//! only in what the two `Cell`s hold when it is called. Re-arming [`MarchedBleed`] in block 2 — the
//! single most natural "tidy the two blocks into one" edit — returns the OPEN number **bit for
//! bit**, which is `phi_closed ≈ 10` against a `1e-6` bar: loud, but only because the bar exists.
//!
//! # AND THE TWO `Cell`s ARE READ DIRECTLY, NOT INFERRED FROM THE NUMBERS
//!
//! `b_state` must be `None` and `v_state` must be `Some(v)` at the instant `cl_s` is taken. A
//! reader that got that wrong would still produce plausible § 3 numbers — they would just be the
//! open loop's — so the state is asserted at the moment of the read and not deduced from the
//! result.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::PointExtra;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::riding4;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::stiffness_ledger::{
    build_stiffness_ledger_cascade, ledger_march, legs, singular_limit, slope_at,
    stiffness_ledger, Leg, SLOPE_AT_REL,
};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{MarchedBleed, MarchedStator};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung77.py`'s module constants, unchanged from step 2's file.

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

// `singular_limit`'s own two defaults (`engine.py:19944`).
const SPREAD: f64 = 0.10;
const EVERY: usize = 8;

// `stiffness_ledger`'s five sweep axes and its own `every` (`engine.py:20009`).
const PHI_LIMS: [f64; 2] = [0.76, 0.80];
const MARGINS: [f64; 3] = [0.05, 0.10, 0.40];
const TT4_MAXES: [f64; 2] = [1180.0, 1200.0];
const ARMS: [bool; 2] = [false, true];
const LEDGER_EVERY: usize = 16;

// The bars `tests/test_rung77.py` states in its own text.
const IFT_BAR: f64 = 3e-8;
const SEP_BAR: f64 = 1e-2;

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

/// The valve AND the stator loop — Python's `_rig(..., inc=False)`. § 4 sweeps `inc` as a MARCH
/// argument on this same rig, exactly as the suite does; it is not a re-arming.
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

fn singular() -> turbojet::stiffness_ledger::SingularLimit {
    singular_limit(&ledger(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R, SETTLE,
                   DS, V_MAX, SPREAD, EVERY)
}

// =============================================================================================
// § 3 — THE SINGULAR LIMIT, AND RUNG 64's DERIVATION MEASURED
// =============================================================================================

/// **P6** — rung 64 marked this *"DERIVED, not measured"*. Closing the valve's loop kills the phi
/// leg's residual slope.
///
/// The bar sits ABOVE the measurement floor and not at it: differencing `phi ≈ 0.8` at `dw ≈ 1e-8`
/// has a roundoff floor near `eps·phi/dw ≈ 1.8e-8`, so the anchor's `1e-7` has ~5× headroom and
/// would be a flake rather than a detector. `1e-6` is still eight orders below the open reading.
#[test]
fn rung_64s_degeneracy_is_measured() {
    let s = singular();
    assert!(s.n >= 5, "n = {} — § 3 needs points to have measured anything", s.n);
    let open = s.phi_open.expect("rows exist");
    let closed = s.phi_closed.expect("rows exist");
    assert!(open > 1.0, "phi_open = {open:e}");
    assert!(closed < 1e-6, "phi_closed = {closed:e}");
    // Python's `max(phi_closed, 1e-30)` — argument 0 is an expression, so the explicit fold.
    let den = if 1e-30 > closed { 1e-30 } else { closed };
    assert!(open / den > 1e6, "ratio = {:e}", open / den);
}

/// § 3 — **stronger than the derivative, and immune to any differencing argument**: under the
/// riding valve `phi_lp` IS `phi_lim` at `0.9·w`, at `w` and at `1.1·w`.
#[test]
fn rung_64s_degeneracy_in_its_blunt_form() {
    let s = singular();
    let off = s.phi_off.expect("rows exist");
    let spread = s.phi_spread.expect("rows exist");
    assert!(off < 1e-12, "phi_off = {off:e}");
    assert!(spread < 1e-12, "phi_spread = {spread:e}");
}

/// § 3.1 — **WITHOUT THIS THE RUNG IS INADMISSIBLE.** Closing a loop perturbs any residual a
/// little; the phi leg's collapse is a claim only because the governor's, read the same way at the
/// same states, barely moves.
#[test]
fn the_governor_is_the_control() {
    let s = singular();
    let rel = s.gov_rel.expect("rows exist");
    let open = s.gov_open.expect("rows exist");
    assert!(rel < 0.1, "gov_rel = {rel:e}");
    assert!(open > 1e4, "gov_open = {open:e}");
}

/// **THE FILE'S LEADING FINDING, MEASURED.** One closure, one set point, three readings that differ
/// only in what the two `Cell`s hold when it is CALLED.
///
/// * both frozen → the OPEN slope, `|G_s'| ≈ 10`;
/// * stator only → the CLOSED slope, `≈ 1e-8`, which is § 3's whole measurement;
/// * both frozen AGAIN inside block 2 — the "tidy the two blocks into one" edit — → **bit for bit
///   the open number**, so `phi_closed` would come back near 10 against a `1e-6` bar.
///
/// The third reading is the one that matters. It shows the ported § 3 DEPENDS on the residual
/// outliving its block, which is exactly what `_residuals`' docstring says can never happen, and it
/// shows the failure is loud only because the bar in
/// [`rung_64s_degeneracy_is_measured`] exists to hear it.
///
/// **And the two `Cell`s are read at the moment of the call**, so "§ 3 measures the closed valve" is
/// asserted rather than inferred from a plausible number.
#[test]
#[allow(non_snake_case)]
fn the_closed_reading_is_a_function_of_the_CELL_and_re_arming_the_valve_guard_restores_the_open_one()
{
    let core = ledger();
    let flight = flight();
    let (m, surge, _lag, traj, accel) = ledger_march(
        &core, &flight, LO, HI, TT4_MAX, SM, TAUS, R, SETTLE, DS, V_MAX, false, MARGIN);
    let surge = surge.expect("the rig arms rung 49's floor");
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let p = pts.iter().step_by(EVERY).next().expect("the march produces riding points");
    let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let (q, v) = match p.extra {
        PointExtra::Demand { b, v, .. } => (b, v),
        PointExtra::Shared { b, v, .. } => (b, v),
        _ => panic!("the demand march carries `b`/`v` on every point"),
    };

    // BLOCK 1 — both frozen, and the closures are built here.
    let sb = MarchedBleed::set(&m.fuel.inner, q);
    let sv = MarchedStator::set(&m.fuel.inner, v);
    let ls = legs(&m.fuel, &flight, a, h, ms, Some(&accel), Some(&surge), Some(TT4_MAX))
        .unwrap_or_else(|e| panic!("{}", e.0));
    let ws = ls.at(Leg::Phi).w;
    let open = slope_at(&*ls.at(Leg::Phi).g, ws, SLOPE_AT_REL)
        .unwrap_or_else(|e| panic!("{}", e.0));
    drop(sv);
    drop(sb);

    // BLOCK 2, AS SHIPPED — the stator only.
    let sv2 = MarchedStator::set(&m.fuel.inner, v);
    // THE STATE, READ AT THE MOMENT OF THE CALL.
    assert_eq!(m.fuel.inner.b_state.get(), None,
               "block 2 must run with the valve's loop CLOSED — a `Some` here means the open \
                plant is being read and every § 3 number is the wrong one");
    assert_eq!(m.fuel.inner.v_state.get(), Some(v),
               "the stator is still frozen in block 2; only the valve is not");
    let closed = slope_at(&*ls.at(Leg::Phi).g, ws, SLOPE_AT_REL)
        .unwrap_or_else(|e| panic!("{}", e.0));
    drop(sv2);

    // BLOCK 2, THE TIDY-UP DEFECT — the valve guard re-armed beside the stator's.
    let sv3 = MarchedStator::set(&m.fuel.inner, v);
    let sb3 = MarchedBleed::set(&m.fuel.inner, q);
    let tidied = slope_at(&*ls.at(Leg::Phi).g, ws, SLOPE_AT_REL)
        .unwrap_or_else(|e| panic!("{}", e.0));
    drop(sb3);
    drop(sv3);

    assert!(open.abs() > 1.0, "the open slope must be the live one: {open:e}");
    assert!(closed.abs() < 1e-6, "the closed slope must have collapsed: {closed:e}");
    assert_eq!(tidied.to_bits(), open.to_bits(),
               "re-arming the valve guard reproduces the OPEN reading BIT FOR BIT ({tidied:e} vs \
                {open:e}) — the same closure, the same set point, a different `Cell`");
    assert!(tidied.abs() > 1e-6,
            "and that is what `phi_closed < 1e-6` would then be asked to accept");
}

/// The shipped reader's first row IS the hand-built point — so the three readings above are
/// [`singular_limit`]'s own arithmetic and not a parallel implementation that happens to agree.
#[test]
fn the_shipped_reader_reproduces_the_hand_built_first_point() {
    let core = ledger();
    let flight = flight();
    let s = singular_limit(&core, &flight, LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R, SETTLE,
                           DS, V_MAX, SPREAD, EVERY);
    let row = s.rows.first().expect("§ 3 produced rows");

    let (m, surge, _lag, traj, accel) = ledger_march(
        &core, &flight, LO, HI, TT4_MAX, SM, TAUS, R, SETTLE, DS, V_MAX, false, MARGIN);
    let surge = surge.expect("the rig arms rung 49's floor");
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let p = pts.iter().step_by(EVERY).next().expect("the march produces riding points");
    let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let (q, v) = match p.extra {
        PointExtra::Demand { b, v, .. } => (b, v),
        PointExtra::Shared { b, v, .. } => (b, v),
        _ => panic!("the demand march carries `b`/`v` on every point"),
    };
    let sb = MarchedBleed::set(&m.fuel.inner, q);
    let sv = MarchedStator::set(&m.fuel.inner, v);
    let ls = legs(&m.fuel, &flight, a, h, ms, Some(&accel), Some(&surge), Some(TT4_MAX))
        .unwrap_or_else(|e| panic!("{}", e.0));
    let ws = ls.at(Leg::Phi).w;
    let open = slope_at(&*ls.at(Leg::Phi).g, ws, SLOPE_AT_REL)
        .unwrap_or_else(|e| panic!("{}", e.0));
    drop(sv);
    drop(sb);

    assert_eq!(row.s.to_bits(), p.s.to_bits(), "the same marched point");
    assert_eq!(row.w_phi.to_bits(), ws.to_bits(), "the same phi set point");
    assert_eq!(row.phi_open.to_bits(), open.to_bits(), "the same OPEN slope, bit for bit");
}

// =============================================================================================
// § 4 — THE ORDER OVER THE ARMS, AND THE GUARD THAT MAKES IT LEGAL
// =============================================================================================

fn ledger_sweep() -> turbojet::stiffness_ledger::StiffnessLedger {
    stiffness_ledger(&ledger(), &flight(), LO, HI, &PHI_LIMS, &MARGINS, &TT4_MAXES, &ARMS, TAUS,
                     R, SETTLE, DS, V_MAX, LEDGER_EVERY)
}

/// **P4 — REFUTED RAW, HELD GUARDED**, and this gate asserts BOTH halves so the correction cannot
/// be quietly dropped.
///
/// Raw, cells invert — at `margin = 0.40` the accel leg has gone DORMANT and the ledger is ordering
/// a leg that is not acting. Under rung 76 § 1.3's own switch guard one level over, the orderings
/// agree on every pair both contain and the phi leg is top everywhere. The second guarded ordering
/// is the first with the dormant leg REMOVED, not re-ordered — asserted as a subsequence rather
/// than by eye.
#[test]
fn the_order_needs_the_dormancy_guard() {
    let s = ledger_sweep();
    assert_eq!((s.n_live, s.n_cells), (24, 24), "every cell must march: {:?}",
               s.cells.iter().map(|c| c.n).collect::<Vec<_>>());
    // the refutation, asserted AS a refutation
    assert!(!s.order_invariant, "orders = {:?}", s.orders);
    assert_eq!(s.orders.len(), 2, "orders = {:?}", s.orders);
    // ... and the guarded reading, which is what § 4 claims
    assert!(s.phi_top, "the phi leg must be top in every cell");
    assert_eq!(s.guarded_orders,
               vec![vec![Leg::Accel, Leg::Gov, Leg::Phi], vec![Leg::Gov, Leg::Phi]],
               "guarded = {:?}", s.guarded_orders);
    // the second IS the first with the dormant leg removed, not re-ordered
    let full = &s.guarded_orders[0];
    let short = &s.guarded_orders[1];
    assert!(short.iter().all(|k| full.contains(k)) && short.len() < full.len(),
            "{short:?} must be a sub-ordering of {full:?}");
    let kept: Vec<Leg> = full.iter().copied().filter(|k| short.contains(k)).collect();
    assert_eq!(&kept, short, "the surviving legs keep their relative order");

    let sep = s.sep.expect("live cells exist");
    let ift = s.ift_err.expect("live cells exist");
    let gov = s.gov_norm.expect("live cells exist");
    assert!(sep > SEP_BAR, "sep = {sep:e}");
    assert!(ift < IFT_BAR, "ift_err = {ift:e}");
    assert!(gov > 0.5, "gov_norm = {gov:e}");
}

/// **P8 — BOUNDS rung 76 § 8's fourth seam before it is built**: `c → 1` is the divergent-gain
/// limit, and no setting this family already has gets near it, so that rung needs a NEW schedule
/// reference and cannot be had by turning `margin` up.
///
/// **The non-vacuity half is this file's own and is not in the suite.** `c_max` folds into a `0.0`
/// seed, so a family whose `c` never went positive would report exactly `0.0` and pass `< 0.35`
/// having measured nothing — the `max(…, default=)` shape plan § 5.32 (iv) names live at this
/// slice. So the reading is required to be strictly positive AND to be ACHIEVED by a named cell.
#[test]
fn c_never_approaches_one() {
    let s = ledger_sweep();
    assert!(s.c_max < 0.35, "c_max = {}", s.c_max);
    assert!(s.c_max > 0.0, "c_max = {} — that is the fold's SEED, not a measurement", s.c_max);
    let hit = s.cells.iter().filter_map(|c| c.c).any(|(lo, hi)| {
        (1.0 - lo).to_bits() == s.c_max.to_bits() || (1.0 - hi).to_bits() == s.c_max.to_bits()
    });
    assert!(hit, "c_max = {} is not attained by any cell's `c` span", s.c_max);
}

/// A cell that produced no points carries ONLY its four coordinates and `n = 0` — and on this grid
/// there are none, which is the count [`the_order_needs_the_dormancy_guard`] asserts from the other
/// side. The gate is here because a silent `n = 0` cell is a march defect and not an arithmetic
/// one, and the two would otherwise share one failure message.
#[test]
fn no_cell_is_empty_and_every_live_cell_carries_its_whole_row() {
    let s = ledger_sweep();
    for c in &s.cells {
        assert!(c.n > 0, "cell (inc={}, phi={}, margin={}, Tt4_max={}) produced no points",
                c.inc, c.phi_lim, c.margin, c.tt4_max);
        assert!(c.order.is_some() && c.c.is_some() && c.norm.is_some() && c.sep.is_some()
                    && c.gov_norm.is_some() && c.ift_err.is_some() && c.phi_top.is_some(),
                "a live cell must carry every field § 4 aggregates");
    }
}

/// **THE PYTHON DOCSTRING'S OWN COUNT, REPRODUCED — and the two bars I typed from the narrative
/// beside it were BOTH false.**
///
/// `test_the_order_needs_the_dormancy_guard`'s docstring says *"Raw, 3 of 24 cells invert — every
/// one at `margin = 0.40`, where the accel leg has gone DORMANT."* That is a discrete count in the
/// suite's own text, so it is a portable bar, and it is the only ABSOLUTE cross-language agreement
/// available at a step with no oracle: **the port finds exactly 3, and every one of them sits at
/// `margin = 0.40`.**
///
/// **And the aggregates `order_stable` / `guarded_stable` are BOTH `false`.** The first writing of
/// this file asserted both `true` — neither is a bar the suite states, and both were typed from the
/// § 4 narrative rather than from `tests/test_rung77.py`'s asserts. That is the port's recorded
/// *typed from the narrative with the counterexample already in my own measurement* pattern, and
/// the repair is not to delete the reading but to assert what it actually is, with the structure
/// that explains it: **every unstable cell — raw or guarded — is a `margin = 0.40` cell.** A cell
/// whose guarded ordering moves from point to point is admissible under § 4 precisely because § 4
/// reports the SET; what would be a finding is instability somewhere the accel leg is still acting.
#[test]
fn the_raw_inversion_is_three_cells_and_every_unstable_cell_is_the_dormancy_corner() {
    let s = ledger_sweep();
    const DORMANCY_MARGIN: f64 = 0.40;
    let inverted: Vec<&turbojet::stiffness_ledger::LedgerCell> = s
        .cells
        .iter()
        .filter(|c| c.order != Some([Leg::Accel, Leg::Gov, Leg::Phi]))
        .collect();
    assert_eq!(inverted.len(), 3,
               "the suite's own docstring says 3 of 24; got {:?}",
               inverted.iter().map(|c| (c.inc, c.phi_lim, c.margin, c.tt4_max, c.order))
                   .collect::<Vec<_>>());
    for c in &inverted {
        assert_eq!(c.margin, DORMANCY_MARGIN,
                   "an inverted cell outside the dormancy corner: (inc={}, phi={}, margin={},                     Tt4_max={})", c.inc, c.phi_lim, c.margin, c.tt4_max);
        assert_eq!(c.order, Some([Leg::Gov, Leg::Accel, Leg::Phi]),
                   "the inversion swaps accel and gov and leaves phi on top");
    }
    // NEITHER aggregate is stable, and the suite asserts neither. See this test's doc.
    assert!(!s.order_stable, "the raw ordering is NOT stable across every cell on this grid");
    assert!(!s.guarded_stable, "and neither is the guarded one");
    for c in s.cells.iter().filter(|c| c.stable == Some(false)
                                       || c.guarded_stable == Some(false)) {
        assert_eq!(c.margin, DORMANCY_MARGIN,
                   "instability outside the dormancy corner would be a finding: (inc={}, phi={},                     margin={}, Tt4_max={})", c.inc, c.phi_lim, c.margin, c.tt4_max);
    }
}
