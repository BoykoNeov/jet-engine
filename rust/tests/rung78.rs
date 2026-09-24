//! RUNG 78 — **THE RESIDUAL GAUGE**: rung 77 § 9's second seam, CLOSED BY REFUTING IT.
//!
//! ```text
//! cap_k(w) = w0 + k*(cap(w) - w0)      w0 := the k = 1 root, so cap_k(w0) = w0 IDENTICALLY
//! ```
//!
//! **HEADLINE: a residual's SLOPE is a GAUGE; its root's UNIQUENESS is not.** `G_k' = 1 - k*c` is a
//! free dial through zero and out the far side, and `dw*/dq` does not move. But the gauge destroys
//! UNIQUENESS: a second root collides with the true one at `k*c = 1`, and inside that band a
//! solver converges cleanly onto the WRONG root. And rung 76 SURVIVES: `solve` -> `sensed` MOVES
//! the root, so it is a device, not a gauge.
//!
//! Ported from `tests/test_rung78.py`: **12 collected tests, of which 3 carry `slow` there**
//! (`test_rung_76_measured_a_device_not_a_gauge`, `test_the_phi_legs_route_has_no_q_to_diverge_in`,
//! `test_the_march_is_gauge_invariant_but_the_leg_is_masked`). **This file has 14**, and the two
//! extra are DECLARED below rather than absorbed.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, **2 ADDED**, 0 COLLAPSED, 0 SPLIT BY PARAMETER
//!
//! | # | `tests/test_rung78.py` | here |
//! |---|---|---|
//! | 1 | `reduce_k_one_is_rung_77_by_dispatch` | [`reduce_k_one_is_rung_77_by_dispatch`] |
//! | 2 | `reduce_is_gated_in_both_directions` | [`reduce_is_gated_in_both_directions`] |
//! | 3 | `slope_is_the_predicted_free_dial` | [`slope_is_the_predicted_free_dial`] |
//! | 4 | `the_k_one_column_is_rung_76s_c` | [`the_k_one_column_is_rung_76s_c`] |
//! | 5 | `set_point_is_gauge_invariant` | [`set_point_is_gauge_invariant`] |
//! | 6 | `sensitivity_is_gauge_invariant` | [`sensitivity_is_gauge_invariant`] |
//! | 7 | `the_exclusion_is_measured_and_not_free` | [`the_exclusion_is_measured_and_not_free`] |
//! | 8 | `the_construction_is_exact_on_the_plant` | [`the_construction_is_exact_on_the_plant`] |
//! | 9 | `the_gauge_destroys_uniqueness` | [`the_gauge_destroys_uniqueness`] |
//! | 10 | `rung_76_measured_a_device_not_a_gauge` | [`rung_76_measured_a_device_not_a_gauge`] |
//! | 11 | `the_phi_legs_route_has_no_q_to_diverge_in` | [`the_phi_legs_route_has_no_q_to_diverge_in`] |
//! | 12 | `the_march_is_gauge_invariant_but_the_leg_is_masked` | [`the_march_is_gauge_invariant_but_the_leg_is_masked`] |
//! | **+1** | **— none —** | [`the_sensed_gauge_refusal_fires_and_the_real_entry_raises_it`] |
//! | **+2** | **— none —** | [`the_anchored_root_refusal_fires_and_the_real_entry_swallows_it`] |
//!
//! # THE TWO ADDED GATES — P5, OWED SINCE THE PRE-FLIGHT, AND WRITTEN FROM THE SOURCE
//!
//! Plan § 5.32 (v) measured rung 78's two refusals (`engine.py:20174`, `:20189`) and **zero**
//! refusal assertions in the suite, so nothing here could be translated. P5 predicted both
//! REACHABLE. Both are, and each gate drives its refusal twice: once by calling the cap hook
//! directly at a riding point (the refusal's own `Err`, with a control beside it that returns),
//! and once through the REAL entry, a gauged `_cap_march`. **The second route is where the finding
//! is: the two refusals of one rung surface through one entry in OPPOSITE ways.**
//!
//! * **+1, `sensed × gauge`**, fires on the very first call, which is the march's initial solve —
//!   and that sits BEFORE the loop's `try` (the port's `raise` at `demand_coordinate.rs`, Python
//!   raising out of the method). So the march RAISES. Measured in both languages: the message,
//!   with the hit counter at exactly **1** (it is bumped before the refusal).
//! * **+2, the anchored-root refusal**, depends on the STATE, so it first fires mid-march — inside
//!   the loop's `try`, whose `except AssertionError: break` (`engine.py:17967`/`:17991`) turns it
//!   into an EARLY STOP. The march returns a SHORT trajectory and no error at all. Its own message
//!   says *"A march must not run on either"*, and the march indeed does not run on — but nothing
//!   tells the caller it stopped. Measured in BOTH languages, identically: at `k = 1.1/c0` the
//!   march ends after **27 of 341** steps; at `1.05/c0` 18, at `1.2/c0` 45, and at `0.9/c0` it runs
//!   the whole 341. **The only trace is the length**, which is why `gauge_march`'s `same_len` and
//!   `clear` bars (gate 12) are load-bearing and not decoration.
//!
//! Needles transcribed from `engine.py` and NOT from `residual_gauge.rs`: the port's second message
//! formats `k` as `{k:.6}` where Python writes `{k:.6g}`, so only the prefix before the first
//! formatted value is a shared string. Each needle matches exactly ONE message in `engine.py`.
//!
//! # THE GAUGE COUNTERS ARE PROCESS-GLOBAL, SO THREE GATES HERE HOLD ONE LOCK
//!
//! [`GAUGE_HITS`](turbojet::residual_gauge::GAUGE_HITS) and `GAUGE_BINDS` are class attributes in
//! Python and `static`s in the port, and cargo runs one binary's tests on parallel threads.
//! `gauge_march` resets then reads them, so any sibling that drives a GAUGED cap during that march
//! corrupts `hits` — and **both added gates do**, since each hit is counted before its refusal
//! fires. `slice_ah_march.rs` solved this by being alone in its binary; this file cannot, so gate
//! 12, `+1` and `+2` each hold [`GAUGED`]. The other eleven never reach a gauged `_cap_fuel`:
//! `gauge_scan` and `root_census` solve the gauged residual directly and `gauge_vs_device` runs
//! at the identity.
//!
//! # WHAT OVERLAPS `slice_ah_gauge.rs` / `slice_ah_march.rs`, AND WHY IT IS PORTED ANYWAY
//!
//! Steps 4 and 5 already gate close relatives of **1–12**. Ported regardless, on `rung72.rs`–
//! `rung77.rs`'s precedent: this file's contract is a 1:1 map of the shipped suite. Their extra
//! exact pins (`(10, 30, 70)`, `n_roots == [1, 2, 4]`, `hits == 1 366`) stay in those files. Here
//! Python's `max(census["n_roots"])` is a real maximum and not a `last()`, and bare-truthiness
//! asserts are `== Some(true)`, so an absent reading fails.
//!
//! # DEFAULTS
//!
//! Every reader is called as the suite calls it — four positionals — so the rest come from
//! `engine.py`'s signatures (`:20347`, `:20477`, `:20542`, `:20655`). Where `residual_gauge.rs`
//! exports a constant for one it is used; `every = 8` is not exported for `gauge_scan` or
//! `root_census` and is typed as [`EVERY`].

use std::ptr::fn_addr_eq;
use std::sync::{Mutex, MutexGuard, OnceLock};

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AccelSchedule, Floor, FuelPoint, PointExtra};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::demand_coordinate::cap_free;
use turbojet::residual_gauge::{
    accel_cap_fn, build_residual_gauge_cascade, c_on_frozen, gauge_counters, gauge_march,
    gauge_points, gauge_residual, gauge_scan, gauge_vs_device, reset_gauge_counters, root_census,
    GaugeRestored, GaugeScan, RootCensus, DEVICE_DQ, DEVICE_EVERY, DEVICE_SPREAD,
    GAUGE_K_IDENTITY, GAUGE_MARCH_MULTS, GAUGE_SCAN_DQ, GAUGE_SCAN_MULTS, R78_TRIPLE,
    ROOT_CENSUS_MULTS, ROOT_CENSUS_N, ROOT_COUNT_HI, ROOT_COUNT_LO,
};
use turbojet::sensed_cap::{accel_for, c_at, cap_march, C_AT_REL, CAP_LAW_SENSED};
use turbojet::shared_actuator::riding4;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::stiffness_ledger::R77_TRIPLE;
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{MarchedBleed, MarchedStator};

// ============================================================================== the grid
//
// `tests/test_rung78.py`'s module constants, verbatim.

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
const TT4_MAX: f64 = 1200.0;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const PHI_JAC: f64 = 0.80;
const MARGIN: f64 = 0.10;

// ------------------------------------------------ the readers' unspelled defaults (`engine.py`)

const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const INC: bool = false;
const R: f64 = 0.5;
const SETTLE: f64 = 1.2;
const DS: f64 = 0.005;
/// `gauge_scan`'s and `root_census`' `every` — not exported, unlike `gauge_vs_device`'s.
const EVERY: usize = 8;

/// The lock the three gauged gates share — see the module header.
static GAUGED: Mutex<()> = Mutex::new(());

/// Taken through a poisoned lock too: a gate that panics while holding it must not turn every
/// later gauged gate into a poisoning report instead of its own verdict.
fn gauged() -> MutexGuard<'static, ()> {
    GAUGED.lock().unwrap_or_else(|e| e.into_inner())
}

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

fn sm() -> f64 { PHI_JAC / FLOOR - 1.0 }

/// Python's `_rig(design)` — `ResidualGaugeTransient` with the four knobs at `demand` / `sched` /
/// `none` / `solve`, which are also the builder's own defaults; they are SET anyway, because the
/// suite sets them.
fn rig() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm(), Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm(), Some(TAU_S))),
        ..Default::default()
    };
    let m = match build_residual_gauge_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    };
    m.fuel.inner.lag_coord.set("demand");
    m.fuel.inner.ref_law.set("sched");
    m.fuel.inner.windup_law.set("none");
    m.fuel.inner.cap_law.set("solve");
    m
}

/// Python's module-scoped `scan` fixture.
fn scan() -> &'static GaugeScan {
    static S: OnceLock<GaugeScan> = OnceLock::new();
    S.get_or_init(|| gauge_scan(&rig(), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, INC, R,
                                SETTLE, DS, V_MAX, GAUGE_SCAN_DQ, EVERY, &GAUGE_SCAN_MULTS))
}

/// Python's module-scoped `census` fixture.
fn census() -> &'static RootCensus {
    static C: OnceLock<RootCensus> = OnceLock::new();
    C.get_or_init(|| root_census(&rig(), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, INC,
                                 R, SETTLE, DS, V_MAX, EVERY, &ROOT_CENSUS_MULTS, ROOT_COUNT_LO,
                                 ROOT_COUNT_HI, ROOT_CENSUS_N))
}

/// Python's `_rig(design).gauge_vs_device(FLIGHT, LO, HI, TT4_MAX)` — called afresh by each of
/// gates 10 and 11, as the suite does.
fn device() -> turbojet::residual_gauge::GaugeVsDevice {
    gauge_vs_device(&rig(), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, INC, R, SETTLE, DS,
                    V_MAX, DEVICE_EVERY, DEVICE_DQ, DEVICE_SPREAD)
}

// ======================================================================================
// THE REDUCE
// ======================================================================================

/// `_gauge_k = 1.0` must take the PARENT's `_cap_fuel`, not an algebraically-equal copy.
#[test]
fn reduce_k_one_is_rung_77_by_dispatch() {
    let m = rig();
    assert_eq!(m.fuel.inner.gauge_k.get(), 1.0, "the identity gauge is the default");
    assert!(!fn_addr_eq(R78_TRIPLE.cap_fuel, R77_TRIPLE.cap_fuel),
            "rung 78 declares its own `_cap_fuel`; if it did not, there would be nothing to reduce");
    // the gauged residual at k = 1 is the shipped EXPRESSION -- `w0` does not appear
    let cap = |w: f64| -> Result<f64, Abort> { Ok(0.3 * w + 1.0) };
    // Python's `m._gauge_residual(cap, 999.0)` reads `self._gauge_k`; so does this.
    let g1 = gauge_residual(m.fuel.inner.gauge_k.get(), &cap, 999.0);
    for w in [0.5, 1.0, 2.0] {
        assert!(g1(w).expect("this cap never refuses") == w - (0.3 * w + 1.0), "w = {w}");
    }
}

/// Rung 73's discipline: at `k != 1` the SLOPE must differ (else the knob is dead).
#[test]
fn reduce_is_gated_in_both_directions() {
    let s = scan();
    let moved = s.rows.iter()
        .flat_map(|x| x.ks.iter().map(move |d| (x.gw1, d)))
        .filter(|(gw1, d)| d.mult != 1.0 && (d.gw - gw1).abs() > 1e-3)
        .count();
    assert!(moved > 0, "no gauge changed the residual slope — the knob is not wired");
    assert_eq!(s.sign_change, Some(true),
               "the slope never changed sign — the dial did not reach zero");
}

// ======================================================================================
// § 1 — THE SLOPE IS A FREE DIAL
// ======================================================================================

/// P2: `G_w == 1 - k*c`, spanning BOTH signs.
#[test]
fn slope_is_the_predicted_free_dial() {
    let s = scan();
    let e = s.gw_err.expect("the sweep keeps readings");
    assert!(e < 1e-6, "Gw_err = {e:e}");
    let (lo, hi) = s.gw_span.expect("the sweep keeps readings");
    assert!(lo < -1.0 && hi > 1.0, "({lo}, {hi})");
}

/// NON-VACUITY: agreement with rung 76's `_c_at` is what pins the instrument.
#[test]
fn the_k_one_column_is_rung_76s_c() {
    let e = scan().c_err.expect("the sweep has rows");
    assert!(e < 1e-8, "c_err = {e:e}");
}

// ======================================================================================
// § 2 — AND THE SENSITIVITY DOES NOT MOVE
// ======================================================================================

/// P1, on the TRUE root.
#[test]
fn set_point_is_gauge_invariant() {
    let w = scan().w_move.expect("the sweep keeps readings");
    assert!(w < 1e-9, "w_move = {w:e}");
}

/// P3 — THE RUNG. `dw*/dq` does not move, including where `G_w < 0`.
#[test]
fn sensitivity_is_gauge_invariant() {
    let g = scan().gain_move.expect("the sweep keeps readings");
    assert!(g < 1e-6, "gain_move = {g:e}");
}

/// A rung that drops points must say what it dropped, and the dropped ones must MATTER.
#[test]
fn the_exclusion_is_measured_and_not_free() {
    let s = scan();
    assert!(s.n_excluded > 0, "nothing was excluded; the sweep never entered the band");
    assert!(s.n_kept > s.n_excluded,
            "{} of {} readings were dropped — an exclusion that takes most of the sweep is not \
             an exclusion", s.n_excluded, s.n_kept + s.n_excluded);
    let worst = s.excluded_worst.expect("something was excluded");
    assert!(worst > 1e-3,
            "the excluded points moved only {worst:.2e} — if they are harmless then excluding \
             them bought this rung a hold it did not need, and § 1.2 is wrong");
}

// ======================================================================================
// § 3 — THE ROOT SURVIVES, ITS UNIQUENESS DOES NOT
// ======================================================================================

/// `w0` is a root of `G_k` at EVERY gauge.
#[test]
fn the_construction_is_exact_on_the_plant() {
    let c = census();
    let g = c.g_at_w0.expect("the census has cells");
    assert!(g < 1e-14, "G_at_w0 = {g:e}");
    assert!(c.true_found, "the walk lost the true root");
}

/// THE OTHER HALF OF THE HEADLINE: a second root collides with the true one at `k*c = 1`.
#[test]
fn the_gauge_destroys_uniqueness() {
    let c = census();
    let most = c.n_roots.iter().copied().max().expect("the census has cells");
    assert!(most > 1, "n_roots = {:?}", c.n_roots);
    assert_eq!(c.brackets, Some(true),
               "the multi-root band {:?} does not bracket the singular gauge — the collision is \
                then not at `k*c = 1` and § 3's mechanism is wrong", c.band);
    let a = c.approach.expect("some cell is multi-rooted");
    assert!(a < 0.2, "approach = {a:e}");
}

// ======================================================================================
// § 4 — A GAUGE AGAINST A DEVICE, AND THE OTHER ROUTE (`slow` in Python)
// ======================================================================================

/// P6: a re-writing that MOVES the root is a DEVICE.
#[test]
fn rung_76_measured_a_device_not_a_gauge() {
    let d = device().device.expect("the reader has rows");
    assert!(d > 1e-3, "device = {d:e}");
}

/// P5 of the RUNG (not of this slice), REFUTED as worded and replaced structurally.
#[test]
fn the_phi_legs_route_has_no_q_to_diverge_in() {
    let g = device();
    let open_w = g.phi_open_w.expect("rows");
    let kill_w = g.kill_w.expect("rows");
    let open_q = g.phi_open_q.expect("rows");
    let spread = g.phi_spread.expect("rows");
    assert!(open_w > 1.0, "phi_open_w = {open_w:e}");
    assert!(kill_w < 1e-6, "kill_w = {kill_w:e}");
    assert!(open_q > 1e-3,
            "dphi/dq measured {open_q:.3e} — the anchor's `0/0` prediction claimed this dies too, \
             and § 4.2 scores it REFUTED on the strength of it being FINITE");
    assert!(spread < 1e-12, "phi_spread = {spread:e}");
}

// ======================================================================================
// § 5 — THE MARCH (`slow` in Python). A DISCLOSURE gate, not a passing claim.
// ======================================================================================

/// If a future edit makes the accel leg bind, `binds` goes positive, THIS TEST FAILS, and § 5 has
/// to be rewritten as a result instead of a blocked section. Holds [`GAUGED`].
#[test]
fn the_march_is_gauge_invariant_but_the_leg_is_masked() {
    let _lock = gauged();
    let g = gauge_march(&rig(), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, INC, R, SETTLE,
                        DS, V_MAX, &GAUGE_MARCH_MULTS);
    assert!(g.hits > 0,
            "the gauged branch never executed — the comparison is vacuous for the FIRST of the \
             three reasons § 5.1 lists (wrong coordinate)");
    assert!(g.same_len, "a gauge changed the number of steps");
    let worst = g.worst.expect("the sweep has cells");
    assert!(worst < 1e-9, "worst = {worst:e}");
    let moved = g.sched_moved.expect("the sweep has cells");
    assert!(moved < 1e-12,
            "the accel SCHEDULE moved {moved:.3e} with the gauge — `_shared_rig` carries \
             `_gauge_k`, so this section would be comparing two schedules");
    assert!(g.clear, "a run's swept k*c crossed the multi-root band: {:?}", g.kc);
    assert_eq!(g.binds, 0,
               "the gauged cap won the min-select {} times — § 5 is scored NOT ESTABLISHED \
                precisely because it never did", g.binds);
}

// ======================================================================================
// +1 / +2 — RUNG 78's TWO REFUSALS (P5). Written from the source; see the module header.
// ======================================================================================

/// `engine.py:20174`'s literal, up to its first formatted value.
const NEEDLE_SENSED: &str = "rung-78: `sensed` x a non-identity GAUGE is REFUSED";
/// `engine.py:20189`'s literal, up to its first formatted value (`{w!r}`).
const NEEDLE_ROOT: &str = "rung-78: the GAUGED accel solve returned";

/// Python's `pytest.raises(AssertionError, match=…)` — the message, or `""` on a clean return.
fn message_of<F: FnOnce()>(f: F) -> String {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(()) => String::new(),
        Err(e) => match e.downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => e.downcast_ref::<&str>().map(|s| (*s).to_string())
                     .unwrap_or_else(|| "<non-string panic>".into()),
        },
    }
}

/// One riding point of rung 77's own march, frozen, with its `k = 1` anchor and its `c` —
/// `slice_ah_gauge.rs`'s hand-built cell, the one § 3 is measured on.
struct Cell {
    m: ScheduledStatorCore,
    surge: Option<Floor>,
    accel: AccelSchedule,
    p: FuelPoint,
    w0: f64,
    c: f64,
}

fn cell(i: usize) -> Cell {
    let fl = flight();
    let (m, surge, accel, pts) = gauge_points(
        &rig(), &fl, LO, HI, TT4_MAX, MARGIN, TAUS, R, SETTLE, DS, V_MAX, INC, PHI_JAC, EVERY);
    let p = pts[i].clone();
    let (q, v) = bv(&p);
    let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let _sb = MarchedBleed::set(&m.fuel.inner, q);
    let _sv = MarchedStator::set(&m.fuel.inner, v);
    let w0 = {
        let cap = accel_cap_fn(&m.fuel, &fl, a, h, &accel);
        let g0 = |w: f64| -> Result<f64, Abort> { Ok(w - cap(w)?) };
        cap_free(&g0, ms, &|| m.fuel.try_sched_fuel(&fl, a, h, ms, &accel))
            .expect("the k = 1 anchor solves here")
    };
    let c = c_at(&m, &fl, a, h, &accel, w0, q, v, C_AT_REL).expect("rung 76's c");
    drop((_sb, _sv));
    Cell { m, surge, accel, p, w0, c }
}

fn bv(p: &FuelPoint) -> (f64, f64) {
    match p.extra {
        PointExtra::Demand { b, v, .. } | PointExtra::Shared { b, v, .. } => (b, v),
        _ => panic!("the demand march carries `b`/`v` on every point"),
    }
}

/// The cap hook at the cell, under gauge `k` — the call the march's `F` makes, frozen the same way.
fn cap_at(x: &Cell, k: f64) -> Result<f64, Abort> {
    let fl = flight();
    let (q, v) = bv(&x.p);
    let _sb = MarchedBleed::set(&x.m.fuel.inner, q);
    let _sv = MarchedStator::set(&x.m.fuel.inner, v);
    let _gk = GaugeRestored::set(&x.m.fuel.inner, k);
    (x.m.fuel.inner.triple_hooks.cap_fuel)(
        &x.m.fuel, &fl, x.p.nu_lp, x.p.nu_hp, x.p.mf_sched, Some(&x.accel), x.surge.as_ref(),
        Some(x.p.mf_sched))
}

/// Python's `m._with_gauge(k, m._cap_march, …)[3]` — the REAL entry, on the suite's rig.
fn gauged_march(core: &ScheduledStatorCore, k: f64, cap_law: &'static str,
                acc: &AccelSchedule) -> Vec<FuelPoint> {
    let _gk = GaugeRestored::set(&core.fuel.inner, k);
    cap_march(core, &flight(), LO, HI, TT4_MAX, sm(), TAUS, R, SETTLE, DS, V_MAX, INC, "demand",
              "sched", "none", None, cap_law, acc, None).3
}

/// **+1 — `sensed × a non-identity GAUGE` IS REFUSED, AND THROUGH A MARCH IT RAISES.**
///
/// Directly: at a riding point, `sensed` at `k = 2` refuses, and `sensed` at the identity does
/// not — so the refusal is the GAUGE's, not the law's (rung 76 marches `sensed` all day). Through
/// the real entry the very first cap call is the march's initial solve, outside the loop's `try`,
/// so the march panics with the message — and the hit counter reads exactly **1**, which is the
/// Python run's number: the gauged branch is counted before it refuses.
#[test]
fn the_sensed_gauge_refusal_fires_and_the_real_entry_raises_it() {
    let _lock = gauged();
    let x = cell(0);
    x.m.fuel.inner.cap_law.set(CAP_LAW_SENSED);
    let e = cap_at(&x, 2.0).expect_err("`sensed` under a gauge must refuse");
    assert!(e.0.contains(NEEDLE_SENSED), "{}", e.0);
    assert!(cap_at(&x, GAUGE_K_IDENTITY).is_ok(),
            "`sensed` at the IDENTITY is rung 76's plant and must not refuse — the control that \
             makes the line above about the gauge");
    assert_eq!(x.m.fuel.inner.gauge_k.get(), GAUGE_K_IDENTITY, "`GaugeRestored` restored it");

    let core = rig();
    let acc = accel_for(&core, &flight(), LO, HI, sm(), TT4_MAX, TAUS, V_MAX, INC, MARGIN);
    reset_gauge_counters();
    let msg = message_of(|| { gauged_march(&core, 2.0, CAP_LAW_SENSED, &acc); });
    assert!(msg.contains(NEEDLE_SENSED), "the march must RAISE this refusal: {msg:?}");
    assert_eq!(gauge_counters(), (1, 0),
               "one hit, counted before the refusal, and no bind — the Python run's reading");
    assert_eq!(core.fuel.inner.gauge_k.get(), GAUGE_K_IDENTITY,
               "the gauge is restored on the unwinding path too — Python's `finally`");
}

/// **+2 — THE ANCHORED-ROOT REFUSAL, AND THROUGH A MARCH IT IS SWALLOWED.**
///
/// Directly, at riding point 1 (§ 3's cell): gauge `1.1/c` sits inside the collision band, the
/// damped Newton converges onto the SECOND root, and the cap refuses; at `2.0/c`, outside the band,
/// the same call returns the anchor. Through the real entry the refusal first fires INSIDE the
/// loop's `try`, whose `except AssertionError: break` ends the march early and silently.
///
/// `c0` is read as `gauge_march` reads its sweep (`c_on_frozen` at the demand march's first riding
/// point), and the lengths are pinned where Python measured them on the same rig — the only
/// absolute cross-language agreement this gate has, and the whole of the finding.
#[test]
fn the_anchored_root_refusal_fires_and_the_real_entry_swallows_it() {
    let _lock = gauged();
    let x = cell(1);
    let e = cap_at(&x, 1.1 / x.c).expect_err("inside the band the solver finds the other root");
    assert!(e.0.contains(NEEDLE_ROOT), "{}", e.0);
    let w = cap_at(&x, 2.0 / x.c).expect("outside the band the gauged solve returns");
    assert!((w - x.w0).abs() <= 1e-6 * x.w0.abs(), "w = {w:e}, w0 = {:e}", x.w0);

    let core = rig();
    let fl = flight();
    let acc = accel_for(&core, &fl, LO, HI, sm(), TT4_MAX, TAUS, V_MAX, INC, MARGIN);
    let (m0, _s, _l, traj0) = cap_march(&core, &fl, LO, HI, TT4_MAX, sm(), TAUS, R, SETTLE, DS,
                                        V_MAX, INC, "demand", "sched", "none", None, "solve", &acc,
                                        None);
    let b_max = m0.fuel.inner.lever.lim.expect("the rig arms the valve").b_max;
    let c0 = c_on_frozen(&m0, &fl, &riding4(&traj0, b_max)[0], &acc).expect("rung 76's c");
    assert_eq!(traj0.len(), 341);

    let mut lens = Vec::new();
    for mult in [1.1, 1.05, 1.2, 0.9] {
        reset_gauge_counters();
        let msg = message_of(|| { lens.push(gauged_march(&core, mult / c0, "solve", &acc).len()); });
        assert_eq!(msg, "", "the march must NOT raise at mult {mult} — the refusal is swallowed");
        assert!(gauge_counters().0 > 0, "the gauged branch ran at mult {mult}");
    }
    assert_eq!(lens, vec![27, 18, 45, 341],
               "inside the band the march STOPS EARLY with no error; outside it runs to the end");
    assert_eq!(core.fuel.inner.gauge_k.get(), GAUGE_K_IDENTITY);
}
