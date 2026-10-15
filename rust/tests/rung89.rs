//! RUNG 89 — the ASYMMETRIC valve: which edge decides who is cheaper?
//! `docs/rung89-spec.md`; anchor `docs/plans/rung89-anchor-asymmetric-valve.md` (§ refs are to it).
//!
//! Gate groups:
//!
//!   1. THE DEVICE — `closing(τ)` IS the symmetric valve bit for bit (through the switching code);
//!      the closing constant survives `lagged` and rung 80's rebuild; the readers of ONE constant
//!      (rungs 66–80, through `r64_solve_b`) refuse it; the rate law picks the edge's constant; the
//!      RK4 bound reads the smaller one.
//!   2. THE ARITHMETIC (no credit) — D2: the march is bit for bit independent of `τ_close` before
//!      the first closing read; D1′: `∫(b − b_cmd)/τ(s) ds = b0 − b_end`, and the bill is
//!      `τ_close·D − τ_open·U`.
//!   3. THE EXPERIMENT — § 3's bars on representative cells, each naming its HIT / MISS.

use turbojet::asymmetric_valve::{asym_valve, march_marked, matched_asym, phi_floor};
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::lagged_incidence::{lag_read, matched_lagged, trapz, valve_at};
use turbojet::limited_bleed::{r64_solve_b, BleedLimiter, ValveReference};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use turbojet::three_levers::march;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::valve_reference::sibling;

// ---------------------------------------------------------------------------- rung 87's rig
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const DS: f64 = 0.005;
const B: f64 = 0.10;

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
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

/// The bare machine, through RUNG 65's builder (rung 64's refuses a lagged valve).
fn gt() -> ScheduledStatorCore {
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(r: f64, ds: f64) -> Ramp {
    Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds }
}

fn phi_of(sm: f64) -> f64 {
    FLOOR * (1.0 + sm)
}

fn stat() -> LeverArm {
    LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)))
}

fn m_lim_of(m: &ScheduledStatorCore, sm: f64) -> f64 {
    m.design_map(Spool::Lp).tan_beta1_crit() - 1.0 / phi_of(sm)
}

/// Every key of a point, as bits (the recorded valve included).
fn bits(p: &FuelPoint) -> Vec<u64> {
    let (b, c) = match p.extra {
        PointExtra::Valve { b, b_cmd } => (b, b_cmd),
        _ => panic!("a lagged march records the valve at every point"),
    };
    [p.s, p.nu_lp, p.nu_hp, p.phi_lp, p.phi_hp, p.mf, p.sp_thrust, p.mdot_air, b, c].iter().map(|x| x.to_bits()).collect()
}

// =============================================================================================
// GATE 1 — THE DEVICE
// =============================================================================================

/// G0: a closing constant EQUAL to the opening one, through the new switching code, marches bit for
/// bit as rung 65's symmetric valve — both references, stators on and off.
#[test]
fn closing_at_the_opening_speed_is_the_symmetric_valve_bit_for_bit() {
    let m = gt();
    let rp = ramp(0.5, DS);
    for stator in [Some(stat()), None] {
        for inc in [false, true] {
            let v = BleedLimiter::with_tau(phi_of(0.46), B, Some(0.05));
            let v = if inc { v.on_incidence() } else { v };
            let (a, _) = march(&sibling(&m, stator.as_ref(), Some(v)), &flight(), &rp, None);
            let (b, _) = march(&sibling(&m, stator.as_ref(), Some(v.closing(0.05))), &flight(), &rp, None);
            assert_eq!(a.len(), 341, "the march ran to the end");
            assert_eq!(a.len(), b.len());
            for (p, q) in a.iter().zip(&b) {
                assert_eq!(bits(p), bits(q), "stator {} inc {inc}: differs at s = {}", stator.is_some(), p.s);
            }
        }
    }
}

/// § 1: the closing constant needs a lag, rides through `lagged` (which re-sets only the OPENING one)
/// and through rung 80's rebuild; the readers of ONE constant refuse it.
#[test]
fn the_closing_constant_is_carried_or_refused_never_dropped() {
    let v = BleedLimiter::with_tau(phi_of(0.42), B, Some(0.05)).closing(0.2);
    assert_eq!((v.tau, v.tau_close), (Some(0.05), Some(0.2)));
    let w = v.on_incidence().lagged(0.02);
    assert_eq!((w.tau, w.tau_close, w.reference), (Some(0.02), Some(0.2), ValveReference::Incidence));
    assert!(std::panic::catch_unwind(|| BleedLimiter::new(phi_of(0.42), B).closing(0.2)).is_err(),
            "an instantaneous valve has no edges to tell apart");
    assert!(std::panic::catch_unwind(|| BleedLimiter::with_tau(phi_of(0.42), B, Some(0.05)).closing(0.0)).is_err());
    struct S(f64);
    impl turbojet::limited_bleed::HasPhiLp for S {
        fn phi_lp(&self) -> f64 { self.0 }
    }
    assert!(std::panic::catch_unwind(|| r64_solve_b(&v, |b| Ok(S(0.7 + b)))).is_err(),
            "rungs 66-80 march one constant: an asymmetric valve is refused there");
    // A SET closing constant is refused even when it equals the opening one (an explicit object).
    assert!(std::panic::catch_unwind(|| r64_solve_b(&BleedLimiter::with_tau(phi_of(0.42), B, Some(0.05)).closing(0.05),
                                                    |b| Ok(S(0.7 + b)))).is_err());
}

/// The rate law: the OPENING constant while the command is at or above the position, the CLOSING one
/// below it; `tau_close = None` closes at `tau`. And RK4's bound reads the smaller constant.
#[test]
fn the_rate_law_picks_the_edges_constant() {
    let v = BleedLimiter::with_tau(0.8, B, Some(0.05));
    let a = v.closing(0.2);
    assert_eq!(a.lag_rate(0.04, 0.02), (0.04 - 0.02) / 0.05, "opening at tau_open");
    assert_eq!(a.lag_rate(0.02, 0.04), (0.02 - 0.04) / 0.2, "closing at tau_close");
    assert_eq!(a.lag_rate(0.03, 0.03), 0.0);
    assert_eq!(v.lag_rate(0.02, 0.04), (0.02 - 0.04) / 0.05, "symmetric closes at tau");
    assert_eq!((v.tau_min(), a.tau_min(), v.closing(0.01).tau_min()), (Some(0.05), Some(0.05), Some(0.01)));
    let m = gt();
    let fast_close = sibling(&m, Some(&stat()), Some(BleedLimiter::with_tau(phi_of(0.42), B, Some(0.05)).closing(0.002)));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| march(&fast_close, &flight(), &ramp(0.5, DS), None))).is_err(),
            "ds/tau_close = 2.5: outside RK4's stability region, refused");
}

// =============================================================================================
// GATE 2 — THE ARITHMETIC (no credit)
// =============================================================================================

/// D2: up to the first RK4 stage that finds the command below the position, the march does not read
/// the closing constant — every point recorded before that mark is bit for bit the same at any
/// `τ_close`, and the runs part after it.
#[test]
fn d2_the_closing_constant_is_not_read_before_the_first_closing_stage() {
    let m = gt();
    let rp = ramp(0.5, DS);
    for inc in [false, true] {
        let mk = |tc: f64| {
            let v = asym_valve(phi_of(0.42), B, 0.05, tc, if inc { ValveReference::Incidence } else { ValveReference::Phi });
            march_marked(&sibling(&m, Some(&stat()), Some(v)), &flight(), &rp)
        };
        let (ta, fa, _) = mk(0.01);
        let (tb, fb, _) = mk(0.2);
        assert_eq!(fa, fb, "inc {inc}: the mark itself is reached identically");
        assert!(fa.is_finite() && fa > 0.0, "inc {inc}: the valve closes ({fa})");
        let before = ta.iter().take_while(|p| p.s < fa).count();
        assert!(before > 10, "inc {inc}: a stretch before the mark ({before} points)");
        for (p, q) in ta.iter().zip(&tb).take(before) {
            assert_eq!(bits(p), bits(q), "inc {inc}: differs at s = {} before the mark {fa}", p.s);
        }
        assert!(ta.iter().zip(&tb).skip(before).any(|(p, q)| bits(p) != bits(q)), "inc {inc}: and parts after it");
    }
}

/// D1′: integrating the rate law edge by edge, `∫(b − b_cmd)·τ_open/τ(s) ds = τ_open·(b0 − b_end)`, so
/// the bill is `τ_close·D − τ_open·U`. The trapezoid closes it to the march's own order, with an ERRATIC
/// coefficient: the weight `τ_open/τ_close` (20 here) jumps where the valve turns, and where that turn
/// falls inside a step moves the residual (rung 84's sawtooth — here −4.6e-6, −5.2e-6, −8.2e-7, −5.8e-7
/// down the halvings, not a clean ×4). So the bar is an O(ds²) ENVELOPE, `0.25·max(1, τ_open/τ_close)·ds²`,
/// at four steps: the worst reading over § 3's grid and five halvings of five runs is 0.19 of `ds²·weight`.
/// A constant or O(ds) error fails at the fine end.
#[test]
fn d1_prime_the_edge_ledger_closes_to_the_steps_order() {
    let m = gt();
    let v = asym_valve(phi_of(0.38), B, 0.2, 0.01, ValveReference::Phi);
    let mm = sibling(&m, Some(&stat()), Some(v));
    for ds in [DS, DS / 2.0, DS / 4.0, DS / 8.0] {
        let (t, _, clamps) = march_marked(&mm, &flight(), &ramp(0.5, ds));
        assert_eq!(clamps, 0, "ds {ds}: D1′'s precondition");
        let l = lag_read(&mm, &flight(), &t, clamps);
        assert!(l.rise > 0.0 && l.fall > l.rise, "ds {ds}: the valve opens, then closes further (it starts open)");
        let bar = 0.25 * (0.2 / 0.01) * ds * ds;
        assert!(l.identity_residual.abs() <= bar, "ds {ds}: residual {:.2e} above the O(ds²) envelope {bar:.2e}",
                l.identity_residual);
    }
}

// =============================================================================================
// GATE 3 — THE EXPERIMENT (one cell of § 3's grid, shaped maps r 0.5 sm 0.46 — the panel's)
// =============================================================================================

/// Both valves at `(τ_open, τ_close)`, re-tuned to the cell's `m_lim`: `(φ_lim, trajectory, its
/// first closing read, s at its M_i minimum)`.
fn tuned(m: &ScheduledStatorCore, to: f64, tc: f64, rf: ValveReference) -> (f64, Vec<FuelPoint>, f64, f64) {
    let rp = ramp(0.5, DS);
    let (x, _, run) = matched_asym(m, &flight(), &rp, Some(&stat()), B, to, tc, rf, m_lim_of(m, 0.46), 1e-12)
        .expect("the cell brackets");
    let (t, fc, clamps) = march_marked(&sibling(m, Some(&stat()), Some(asym_valve(x, B, to, tc, rf))), &flight(), &rp);
    assert_eq!(clamps, 0);
    (x, t, fc, run.s_at_m_i)
}

/// The `φ` valve's bleed over the incidence valve's, over the stretch both reach (rung 88 § 4's R).
fn ratio(inc: &[FuelPoint], phi: &[FuelPoint]) -> f64 {
    let n = inc.len().min(phi.len());
    trapz(phi, n, |p| valve_at(p).0) / trapz(inc, n, |p| valve_at(p).0)
}

/// H1 (HIT at `ds` 0.005; one onset-stage row GRID-SENSITIVE at 0.0025 — spec § 3) and H2: the incidence
/// valve's set point is its OPENING edge's — the same at any closing speed, its minimum before its first
/// closing read — and its bill moves by `(τ_c − τ_o)·D` (D1′). The symmetric end is rung 88's own solve.
#[test]
fn h1_h2_the_incidence_valve_is_priced_by_its_opening_edge() {
    let m = gt();
    for (to, tc) in [(0.01, 0.2), (0.05, 0.01)] {
        let (xd, td, _, _) = tuned(&m, to, to, ValveReference::Incidence);
        let (xs, _, _) = matched_lagged(&m, &flight(), &ramp(0.5, DS), Some(&stat()), B, Some(to),
                                        ValveReference::Incidence, m_lim_of(&m, 0.46), 1e-12).unwrap();
        assert_eq!(xd.to_bits(), xs.to_bits(), "equal constants ARE rung 88's valve");
        let (x, t, fc, s_min) = tuned(&m, to, tc, ValveReference::Incidence);
        assert!((x - xd).abs() <= 1e-9, "({to},{tc}): H1 set point {x} vs symmetric {xd}");
        assert!(s_min < fc, "({to},{tc}): H1 minimum at {s_min} before the first closing read {fc}");
        let vd = sibling(&m, Some(&stat()), Some(asym_valve(xd, B, to, to, ValveReference::Incidence)));
        let ld = lag_read(&vd, &flight(), &td, 0);
        let n = t.len().min(td.len());
        let drift = trapz(&t, n, |p| valve_at(p).0) - trapz(&td, n, |p| valve_at(p).0) - (tc - to) * ld.fall;
        let b_peak = td.iter().map(|p| valve_at(p).0).fold(0.0, f64::max);
        assert!(drift.abs() <= (0.05 * (tc - to).abs() * b_peak).max(1e-5), "({to},{tc}): H2 drift {drift:.2e}");
    }
}

/// H3a (HIT, 56 of 56 rows): where the `φ` valve closes before its minimum, its set point FALLS with its
/// closing speed — and stops at the floor (anchor D5) once the minimum comes first. Here `τ_open` 0.05:
/// sm 0.3807 → 0.3613 (symmetric) → the floor 0.3465 at 0.1 and 0.2.
#[test]
fn h3_the_phi_valves_set_point_falls_with_its_closing_speed_to_a_floor() {
    let m = gt();
    let (floor, resid) = phi_floor(&m, &flight(), &ramp(0.5, DS), Some(&stat()), B, 0.05, m_lim_of(&m, 0.46), 1e-12)
        .expect("the floor brackets");
    assert!(resid.abs() < 1e-12, "the floor is a root, not a jump (rung 83): {resid:.1e}");
    let runs: Vec<_> = [0.01, 0.05, 0.1, 0.2].iter().map(|&tc| tuned(&m, 0.05, tc, ValveReference::Phi)).collect();
    let (x, first): (Vec<f64>, Vec<bool>) = runs.iter().map(|(x, _, fc, s)| (*x, s < fc)).unzip();
    assert_eq!(first, [false, false, true, true], "the minimum comes first from tau_c 0.1");
    assert!(x[0] > x[1] && x[1] > x[2], "strictly falling above the floor: {x:?}");
    for k in [2, 3] {
        assert!((x[k] - floor).abs() <= 1e-9, "at the floor once the minimum comes first: {} vs {floor}", x[k]);
    }
    assert!((floor / FLOOR - 1.0 - 0.3465).abs() < 5e-5, "the floor, sm {}", floor / FLOOR - 1.0);
}

/// H5 — the content. (a) HIT, 40 of 40: a SLOW close makes the `φ` valve the cheaper one at a fast
/// opening where the symmetric valve keeps rung 87's verdict. (c) MISS, 51 of 190 (the finding): a FAST
/// close flips it back the other way at a slow opening — the `φ` valve's set point rises with it while the
/// incidence valve's does not move. So each valve is priced by ONE edge: the `φ` valve's set point is the
/// closing speed's (here the same to 0.001 sm across openings, 0.04 sm across closings).
#[test]
fn h5_the_closing_speed_moves_the_verdict_both_ways() {
    let m = gt();
    let pair = |to, tc| {
        let (_, ti, _, _) = tuned(&m, to, tc, ValveReference::Incidence);
        let (xp, tp, _, _) = tuned(&m, to, tc, ValveReference::Phi);
        (ratio(&ti, &tp), xp / FLOOR - 1.0)
    };
    let (r_ff, sm_ff) = pair(0.01, 0.01);
    let (r_fs, sm_fs) = pair(0.01, 0.2);
    let (r_ss, _) = pair(0.1, 0.1);
    let (r_sf, sm_sf) = pair(0.1, 0.01);
    assert!(r_ff > 1.0 && r_fs < 1.0, "H5a: a slow close reverses the fast-opening verdict ({r_ff:.4} -> {r_fs:.4})");
    assert!(r_ss < 1.0 && r_sf > 1.0, "H5c MISS: a fast close reverses the slow-opening verdict ({r_ss:.4} -> {r_sf:.4})");
    assert!((sm_sf - sm_ff).abs() < 0.002 && sm_ff - sm_fs > 0.03,
            "the phi set point follows the CLOSING speed: {sm_ff:.4} / {sm_sf:.4} (opening 0.01 / 0.1), {sm_fs:.4} (closing 0.2)");
}
