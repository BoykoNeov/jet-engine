//! SLICE AH step 5 — **RUNG 78 §§ 4–5: A GAUGE AGAINST A DEVICE, AND THE MARCH THAT CANNOT SEE
//! THE GAUGE.**
//!
//! # WHAT THIS FILE GATES
//!
//! Rung 78's last four bodies: [`gauge_vs_device`] (§ 4), [`phi_at`] and its guard
//! [`PhiAtFreeze`], [`c_on_frozen`] and [`gauge_march`] (§ 5). With them the rung's bodies are
//! complete.
//!
//! Every numeric bar is quoted from `tests/test_rung78.py`'s own `assert` lines — `device > 1e-3`,
//! `phi_open_w > 1.0`, `kill_w < 1e-6`, `phi_open_q > 1e-3`, `phi_spread < 1e-12`, `hits > 0`,
//! `same_len`, `worst < 1e-9`, `sched_moved < 1e-12`, `clear`, `binds == 0`. The discrete counts
//! (10 rows; 341 steps; 1 366 gauged executions per run) were measured by running the PYTHON
//! readers before any Rust was written, and `hits == 1 366` is pinned exactly where the suite only
//! asks for `> 0`.
//!
//! # THIS FILE IS ITS OWN TEST BINARY ON PURPOSE
//!
//! [`GAUGE_HITS`](turbojet::residual_gauge::GAUGE_HITS) and `GAUGE_BINDS` are process-global,
//! because Python's are class attributes (see `residual_gauge.rs`'s header). Cargo runs a
//! binary's tests on parallel threads, so a sibling test that drove a non-identity gauge through
//! `_cap_fuel` — a step-1 refusal gate does, since the counter is bumped before the refusal —
//! would corrupt [`gauge_march`]'s reset-then-read. **The march gate is the only test here that
//! drives a gauged cap**; § 4 and the guard pin reach the plant at the identity or not through
//! `_cap_fuel` at all. Adding a gauged test to this file breaks that, silently.
//!
//! # THREE OF § 5's BARS CANNOT FAIL ON THIS GRID, AND THEY ARE LISTED AS SUCH
//!
//! * `worst < 1e-9` holds as an exact `0.0` because the gauged cap **never binds** — it is
//!   computed 1 366 times a run and loses the min-select every time. The suite pins `binds == 0` as
//!   a DISCLOSURE, and so does this file.
//! * `sched_moved < 1e-12` holds by construction: the schedule is read off equilibria and the gauge
//!   enters only the march's cap. Measured `0.0`.
//! * `clear` is decided by the multiple, not the sweep: `c` drifts **7e-9** over the 15 points read,
//!   so Python's docstring's "`k·c` sweeps a range" does not happen here.
//!
//! What carries information is `hits` (the gauged branch RAN, the guard against the section's own
//! first, vacuous version) and `binds` (its value never reached the plant).

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::residual_gauge::{
    build_residual_gauge_cascade, gauge_march, gauge_points, gauge_vs_device, phi_at, GaugeMarch,
    GaugeVsDevice, DEVICE_DQ, DEVICE_EVERY, DEVICE_SPREAD, GAUGE_MARCH_MULTS,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{MarchedBleed, MarchedStator};

// ---------------------------------------------------------------------------- the grid
//
// `tests/test_rung78.py`'s module constants, as in `slice_ah_gauge.rs`.

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

fn arm() -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, SM, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU_S))),
        ..Default::default()
    }
}

fn gauge() -> ScheduledStatorCore {
    match build_residual_gauge_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm())
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// Built once — the two § 4 gates read one table, as the two Python tests read one reader.
fn device() -> &'static GaugeVsDevice {
    static D: std::sync::OnceLock<GaugeVsDevice> = std::sync::OnceLock::new();
    D.get_or_init(|| gauge_vs_device(
        &gauge(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R, SETTLE, DS, V_MAX,
        DEVICE_EVERY, DEVICE_DQ, DEVICE_SPREAD))
}

fn march() -> GaugeMarch {
    gauge_march(&gauge(), &flight(), LO, HI, TT4_MAX, PHI, MARGIN, TAUS, false, R, SETTLE, DS,
                V_MAX, &GAUGE_MARCH_MULTS)
}

// ---------------------------------------------------------------------------- § 4

/// P6, and half the headline: a re-writing that MOVES the root is a DEVICE. If `solve` and
/// `sensed` shared a root, rung 76 § 3 would have measured a coordinate.
#[test]
fn rung_76_measured_a_device_not_a_gauge() {
    let g = device();
    assert_eq!(g.n, 10, "the shipped grid reads 10 riding points (measured in Python)");
    let d = g.device.expect("10 rows");
    assert!(d > 1e-3, "device = {d:.3e}");
}

/// P5, REFUTED as worded and replaced structurally: `dw*/dq` is an OPEN-loop object. Where it is
/// defined `G_w` is finite; where `G_w` dies, there is no `q` at all.
#[test]
fn the_phi_legs_route_has_no_q_to_diverge_in() {
    let g = device();
    let open_w = g.phi_open_w.expect("10 rows");
    let kill_w = g.kill_w.expect("10 rows");
    let open_q = g.phi_open_q.expect("10 rows");
    let spread = g.phi_spread.expect("10 rows");
    assert!(open_w > 1.0, "phi_open_w = {open_w:.3e} -- finite where dw*/dq exists");
    assert!(kill_w < 1e-6, "kill_w = {kill_w:.3e} -- dead where the valve rides");
    assert!(open_q > 1e-3, "dphi/dq measured {open_q:.3e} -- the anchor's `0/0` prediction \
            claimed this dies too, and § 4.2 scores it REFUTED on the strength of it being FINITE");
    assert!(spread < 1e-12, "phi_spread = {spread:.3e}");
}

/// **The one hand-built nest in this slice that CAN tell the two restore policies apart.** Hold
/// an outer freeze at `(q, v)`, call [`phi_at`] at `q + dq` inside it, and read the cells after.
/// Python's `finally` would leave `(None, None)`; [`PhiAtFreeze`] leaves the enclosing `(q, v)`.
///
/// And the reading itself must see the INNER `q`: if the guard ignored its argument, `phi_at`
/// would read the plant at `q` and the central difference in § 4 would be an exact zero.
#[test]
fn the_phi_at_guard_restores_the_enclosing_freeze() {
    let core = gauge();
    let (m, surge, _accel, pts) = gauge_points(
        &core, &flight(), LO, HI, TT4_MAX, MARGIN, TAUS, R, SETTLE, DS, V_MAX, false, PHI,
        DEVICE_EVERY);
    let surge = surge.expect("`_shared_rig` arms the phi leg");
    let p = &pts[0];
    let (q, v) = match p.extra {
        // `_gauge_points` rides the SHARED rig, so its points carry the shared extra; the demand
        // extra carries the same pair.
        turbojet::fuel_transient::PointExtra::Demand { b, v, .. }
        | turbojet::fuel_transient::PointExtra::Shared { b, v, .. } => (b, v),
        ref e => panic!("the riding points carry a valve and a stator; got {e:?}"),
    };
    let dq = DEVICE_DQ;
    let st = &m.fuel.inner;
    assert_eq!((st.b_state.get(), st.v_state.get()), (None, None), "enters thawed");
    let (inner_hi, inner_at_q) = {
        let _sb = MarchedBleed::set(st, q);
        let _sv = MarchedStator::set(st, v);
        let hi = phi_at(&m.fuel, &flight(), p.nu_lp, p.nu_hp, p.mf, &surge, Some(q + dq), Some(v))
            .expect("the shipped point is inside the model");
        assert_eq!(st.b_state.get(), Some(q),
                   "PhiAtFreeze must RESTORE the enclosing valve freeze, not clobber it to None \
                    -- Two guards, two policies: do not unify them");
        assert_eq!(st.v_state.get(), Some(v), "... and the enclosing stator freeze");
        let at_q = phi_at(&m.fuel, &flight(), p.nu_lp, p.nu_hp, p.mf, &surge, Some(q), Some(v))
            .expect("the shipped point is inside the model");
        (hi, at_q)
    };
    assert_eq!((st.b_state.get(), st.v_state.get()), (None, None),
               "the OUTER guards are Python's clobber and leave the plant thawed");
    assert_ne!(inner_hi, inner_at_q, "phi_at read the plant at the enclosing q, not its own");
}

// ---------------------------------------------------------------------------- § 5

/// § 5 — a DISCLOSURE gate, not a passing claim, exactly as `tests/test_rung78.py` words it. The
/// trajectory is bit-identical under every gauge because the gauged cap LOSES the min-select at
/// every step; if a future edit makes it bind, `binds` goes positive and this gate fails, which is
/// the correct outcome.
#[test]
fn the_march_is_gauge_invariant_but_the_leg_is_masked() {
    let g = march();
    assert_eq!(g.n, 341, "the base march is 341 steps (measured in Python)");
    assert_eq!(g.cells.len(), GAUGE_MARCH_MULTS.len());
    for c in &g.cells {
        assert_eq!(c.hits, 1366,
                   "mult {}: the gauged branch ran {} times, Python measures 1 366 -- zero is the \
                    FIRST of § 5.1's three vacuities (wrong coordinate)", c.mult, c.hits);
    }
    assert!(g.hits > 0);
    assert!(g.same_len, "a gauge changed the number of steps");
    let worst = g.worst.expect("four cells");
    assert!(worst < 1e-9, "worst = {worst:.3e} at {:?}",
            g.cells.iter().map(|c| c.where_).collect::<Vec<_>>());
    let moved = g.sched_moved.expect("four cells");
    assert!(moved < 1e-12, "the accel SCHEDULE moved {moved:.3e} with the gauge -- \
            `_shared_rig` carries `_gauge_k`, so this section would be comparing two schedules");
    assert!(g.clear, "a run's swept k*c crossed the multi-root band: {:?}", g.kc);
    assert_eq!(g.binds, 0,
               "the gauged cap won the min-select {} times -- § 5 is scored NOT ESTABLISHED \
                precisely because it never did. If this now binds, the trajectory result has \
                become real evidence and docs/rung78-spec.md § 5 must be rewritten to say so.",
               g.binds);
}
