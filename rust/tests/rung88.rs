//! RUNG 88 — the SLOW valve on a moving target: does a lag's cost depend on the wall it watches?
//! `docs/rung88-spec.md`; anchor `docs/plans/rung88-anchor-lagged-incidence.md` (§ refs are to it).
//!
//! Gate groups:
//!
//!   1. THE DEVICE — a lag keeps the valve's reference (it did not: `lagged` reset it to `φ`); with
//!      the stators off the lagged incidence valve IS the lagged `φ` valve bit for bit; the rung
//!      66–80 spelling still refuses an incidence valve.
//!   2. THE ARITHMETIC (no credit) — D1's ledger: the lag's bleed bill is `τ·(b(0) − b(end))`.
//!   3. THE EXPERIMENT — § 3's bars on representative cells, each naming its HIT / MISS, and the
//!      spec's equal-requirement table (both slow valves re-tuned to rung 87's margin).

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::lagged_incidence::{matched_lagged, matched_phi_lagged, run_lagged, trapz, valve_at};
use turbojet::limited_bleed::{r64_solve_b, BleedLimiter, ValveReference};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use turbojet::three_levers::march;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::valve_reference::{incidence_valve, read, sibling};

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

/// The bare machine, through RUNG 65's builder: rung 64's refuses a lagged valve, and with `τ = None`
/// the two march bit for bit (anchor G1).
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

fn valve_bits(p: &FuelPoint) -> (u64, u64) {
    match p.extra {
        PointExtra::Valve { b, b_cmd } => (b.to_bits(), b_cmd.to_bits()),
        _ => panic!("a lagged march records the valve at every point"),
    }
}

// =============================================================================================
// GATE 1 — THE DEVICE
// =============================================================================================

/// § 1: `lagged` rebuilt through `with_tau`, which resets the reference to `φ` — a lagged incidence
/// valve came back a `φ` valve without a word. It keeps it now, and both spellings build one object.
#[test]
fn a_lag_keeps_the_reference() {
    let inc = incidence_valve(phi_of(0.42), B);
    let slow = inc.lagged(0.05);
    assert_eq!(slow.reference, ValveReference::Incidence);
    assert_eq!(slow.tau, Some(0.05));
    assert_eq!(slow, BleedLimiter::with_tau(phi_of(0.42), B, Some(0.05)).on_incidence());
    assert_eq!(BleedLimiter::new(phi_of(0.42), B).lagged(0.05).reference, ValveReference::Phi);
}

/// G1: with the stators off `v ≡ 0`, so the lagged incidence valve IS the lagged `φ` valve — every
/// point, every recorded position and every recorded command, bit for bit.
#[test]
fn the_lagged_incidence_valve_with_the_stators_off_is_the_lagged_phi_valve_bit_for_bit() {
    let m = gt();
    let rp = ramp(0.5, DS);
    let phi = BleedLimiter::with_tau(phi_of(0.42), B, Some(0.05));
    let (ta, _) = march(&sibling(&m, None, Some(phi)), &flight(), &rp, None);
    let (tb, _) = march(&sibling(&m, None, Some(phi.on_incidence())), &flight(), &rp, None);
    assert_eq!(ta.len(), tb.len());
    assert!(ta.len() > 300, "the march ran: {} points", ta.len());
    for (p, q) in ta.iter().zip(&tb) {
        assert!(p.s.to_bits() == q.s.to_bits() && p.nu_lp.to_bits() == q.nu_lp.to_bits()
                    && p.nu_hp.to_bits() == q.nu_hp.to_bits() && p.phi_lp.to_bits() == q.phi_lp.to_bits()
                    && p.mf.to_bits() == q.mf.to_bits() && p.sp_thrust.to_bits() == q.sp_thrust.to_bits(),
                "differs at s = {}", p.s);
        assert_eq!(valve_bits(p), valve_bits(q), "the recorded valve differs at s = {}", p.s);
    }
}

/// Rungs 66–80 solve every valve through `r64_solve_b`, which reads `phi_lim` as a flow coefficient —
/// so a LAGGED incidence valve reaching it is refused exactly as rung 87's instantaneous one was.
#[test]
fn the_rung66_to_80_spelling_still_refuses_an_incidence_valve() {
    struct S(f64);
    impl turbojet::limited_bleed::HasPhiLp for S {
        fn phi_lp(&self) -> f64 { self.0 }
    }
    let slow = incidence_valve(phi_of(0.42), B).lagged(0.05);
    assert!(std::panic::catch_unwind(|| r64_solve_b(&slow, |b| Ok(S(0.7 + b)))).is_err());
}

// =============================================================================================
// GATE 2 — THE ARITHMETIC (no credit): D1's ledger
// =============================================================================================

fn stat() -> LeverArm {
    LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)))
}

fn inc_lag(sm: f64, tau: Option<f64>) -> BleedLimiter {
    BleedLimiter::with_tau(phi_of(sm), B, tau).on_incidence()
}

fn m_lim_of(m: &ScheduledStatorCore, sm: f64) -> f64 {
    m.design_map(Spool::Lp).tan_beta1_crit() - 1.0 / phi_of(sm)
}

/// The machine's own bleed at each point (re-solved if instant, recorded if slow), integrated over the
/// first `n` points.
fn bleed_over(m: &ScheduledStatorCore, t: &[FuelPoint], n: usize) -> f64 {
    let b: Vec<f64> = t.iter().map(|p| m.b_at_point(&flight(), p)).collect();
    let mut a = 0.0;
    for k in 1..n.min(t.len()) {
        a += 0.5 * (t[k].s - t[k - 1].s) * (b[k] + b[k - 1]);
    }
    a
}

/// D1: `∫(b − b_cmd) ds = τ·(b(0) − b(end))`, to the trapezoid's error (measured ≤ 3.5e-6 over every
/// run at `ds` 0.005), with the clamp silent — on an incidence valve that starts SHUT and a `φ` valve that
/// starts OPEN (rung 87's matched partner, shaped `r` 0.5, `sm*` 0.3447: `b0` 1.76e-2).
#[test]
fn d1_the_lags_bleed_ledger_closes() {
    let m = gt();
    let rp = ramp(0.5, DS);
    for (what, v) in [("incidence, starts shut", inc_lag(0.42, Some(0.05))),
                      ("phi, starts open", BleedLimiter::with_tau(phi_of(0.3447), B, Some(0.05)))] {
        let (t, l) = run_lagged(&sibling(&m, Some(&stat()), Some(v)), &flight(), &rp);
        assert_eq!(t.len(), 341, "{what}: ran to the end");
        assert_eq!(l.clamp_hits, 0, "{what}: D1's precondition");
        assert!(l.identity_residual.abs() < 1e-5, "{what}: residual {:.2e}", l.identity_residual);
        assert!(l.b_end.abs() < 1e-9, "{what}: shut at the end, b_end {:.2e}", l.b_end);
        if what.starts_with("phi") {
            assert!(l.b0 > 0.017 && (l.int_b - l.int_b_cmd - 0.05 * l.b0).abs() < 1e-5, "{what}: the bill is tau*b0");
        } else {
            assert_eq!(l.b0, 0.0, "{what}: the stators protect the start");
        }
    }
}

// =============================================================================================
// GATE 3 — THE EXPERIMENT (§ 3; bars frozen at 81831f4)
// =============================================================================================

/// H1 (HIT, 19 of 19 at both steps): the moving target adds no first-order drift — the command
/// integral re-read along the lagged run stays within `max(0.05·τ·b_peak, 1e-5)` of the instant run's
/// bleed. H3 (HIT): its own-coordinate protection falls below `m_lim`, monotone in `τ`. H4 (HIT): the
/// minimum sits where the command RISES. On two scored cells (shaped `r` 0.5 `sm` 0.42 and 0.46).
#[test]
fn h1_h3_h4_the_slow_incidence_valve_on_scored_cells() {
    let m = gt();
    for sm in [0.42, 0.46] {
        let rp = ramp(0.5, DS);
        let mi = sibling(&m, Some(&stat()), Some(inc_lag(sm, None)));
        let ti = march(&mi, &flight(), &rp, None).0;
        let b_peak = read(&mi, &flight(), &ti).b_peak;
        let m_lim = m_lim_of(&m, sm);
        let mut prev = f64::INFINITY;
        for tau in [0.01, 0.05, 0.2] {
            let (tl, l) = run_lagged(&sibling(&m, Some(&stat()), Some(inc_lag(sm, Some(tau)))), &flight(), &rp);
            let n = tl.len().min(ti.len());
            let drift = trapz(&tl, n, |p| valve_at(p).1) - bleed_over(&mi, &ti, n);
            assert!(drift.abs() <= (0.05 * tau * b_peak).max(1e-5), "H1 sm {sm} tau {tau}: drift {drift:.2e}");
            assert!(l.run.m_i < m_lim && l.run.m_i < prev, "H3 sm {sm} tau {tau}: M_i {} vs m_lim {m_lim}", l.run.m_i);
            prev = l.run.m_i;
            assert_eq!(l.cmd_rising_at_m_i, Some(true), "H4 sm {sm} tau {tau}");
        }
    }
}

/// H2 (HIT, 14 of 14 — mostly construction, ranked last): where the `φ` partner starts more open, its
/// lag costs more air than the incidence valve's.
#[test]
fn h2_the_wall_decides_the_lags_bleed_bill_through_where_the_valve_starts() {
    let m = gt();
    let rp = ramp(0.5, DS);
    let (ip, pp) = (inc_lag(0.42, None), BleedLimiter::new(phi_of(0.3447), B));
    let bill = |inst: BleedLimiter, slow: BleedLimiter| {
        let mi = sibling(&m, Some(&stat()), Some(inst));
        let ti = march(&mi, &flight(), &rp, None).0;
        let (tl, _) = run_lagged(&sibling(&m, Some(&stat()), Some(slow)), &flight(), &rp);
        let n = tl.len().min(ti.len());
        trapz(&tl, n, |p| valve_at(p).0) - bleed_over(&mi, &ti, n)
    };
    for tau in [0.01, 0.2] {
        let (bi, bp) = (bill(ip, ip.lagged(tau)), bill(pp, pp.lagged(tau)));
        assert!(bi < bp, "tau {tau}: incidence {bi:.3e} vs phi {bp:.3e}");
    }
}

/// H5 — **MISS** (39 of 95 cell-and-`τ` pairs, at both steps). Matched to the slow incidence valve's own
/// minimum, the slow `φ` valve bleeds MORE at `τ` 0.01 (rung 87's order) and LESS by `τ` 0.1, in every
/// scored cell — rung 87's saving is a property of a FAST valve. Shaped `r` 0.5, `sm` 0.42.
#[test]
fn h5_miss_rung87s_order_reverses_for_a_slow_enough_valve() {
    let m = gt();
    let rp = ramp(0.5, DS);
    for (tau, phi_dearer) in [(0.01, true), (0.1, false)] {
        let (tl, l) = run_lagged(&sibling(&m, Some(&stat()), Some(inc_lag(0.42, Some(tau)))), &flight(), &rp);
        let (_, tx, _) = matched_phi_lagged(&m, &flight(), &rp, Some(&stat()), B, Some(tau), l.run.m_i, 1e-12)
            .expect("brackets");
        let n = tx.len().min(tl.len());
        let (bx, bi) = (trapz(&tx, n, |p| valve_at(p).0), trapz(&tl, n, |p| valve_at(p).0));
        assert_eq!(bx > bi, phi_dearer, "tau {tau}: phi {bx:.4e} vs incidence {bi:.4e}");
    }
}

/// The spec's headline table: BOTH slow valves re-tuned to hold rung 87's margin `m_lim` exactly. Instant,
/// the `φ` valve costs 2.28× the air; at `τ` 0.05 and 0.2, less than the incidence valve. And the `φ`
/// valve's set point first FALLS with `τ` (0.3447 instant → 0.3205 at 0.05): slowness buys it protection
/// in the incidence currency — rung 65's "bandwidth is pure loss" holds in a valve's OWN coordinate only.
#[test]
fn equal_requirement_both_slow_valves_retuned_to_the_same_margin() {
    let m = gt();
    let rp = ramp(0.5, DS);
    let m_lim = m_lim_of(&m, 0.42);
    let fit = |tau, rf| matched_lagged(&m, &flight(), &rp, Some(&stat()), B, tau, rf, m_lim, 1e-12).expect("brackets");
    let bleed = |x: f64, tau: Option<f64>, inc: bool, t: &[FuelPoint], n: usize| {
        let v = BleedLimiter::with_tau(x, B, tau);
        bleed_over(&sibling(&m, Some(&stat()), Some(if inc { v.on_incidence() } else { v })), t, n)
    };
    let mut phi_sm = Vec::new();
    for (tau, phi_dearer) in [(None, true), (Some(0.05), false), (Some(0.2), false)] {
        let ((xi, ti, ri), (xp, tp, rpp)) = (fit(tau, ValveReference::Incidence), fit(tau, ValveReference::Phi));
        assert!((ri.m_i - m_lim).abs() < 1e-9 && (rpp.m_i - m_lim).abs() < 1e-9, "tau {tau:?}: both hold m_lim");
        assert!(ri.saturated == 0 && rpp.saturated == 0, "tau {tau:?}: met by the law, not at the stop");
        let n = ti.len().min(tp.len());
        let (bi, bp) = (bleed(xi, tau, true, &ti, n), bleed(xp, tau, false, &tp, n));
        assert_eq!(bp > bi, phi_dearer, "tau {tau:?}: phi {bp:.4e} vs incidence {bi:.4e}");
        phi_sm.push(xp / FLOOR - 1.0);
    }
    assert!(phi_sm[1] < phi_sm[0], "the slow phi valve needs a LOWER set point: {phi_sm:?}");
}
