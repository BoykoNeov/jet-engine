//! RUNG 87 — the CONTROLLED valve beside the stator schedule: what should it watch?
//! `docs/rung87-spec.md`; anchor `docs/plans/rung87-anchor-valve-reference.md` (§ refs are to it;
//! § 3's bars were committed in `9f45f4e` before any of them was measured).
//!
//! Gate groups:
//!
//!   1. THE DEVICE — the incidence valve with the stators off IS the `φ` valve bit for bit; every
//!      sibling constructor carries the reference; the routes that would read it in the wrong
//!      coordinate refuse it; rung 62's builder drops a floor SILENTLY (G1, recorded).
//!   2. THE ARITHMETIC (no credit) — D1's pins, D2's dormancy ledger, D4's dormant fuel floor.
//!   3. THE EXPERIMENT — § 3's bars on representative cells, each naming its HIT / MISS.

use turbojet::bleed_transient::{build_scheduled_bleed, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::FuelPoint;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, r64_solve_b, BleedLimiter, ValveReference};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use turbojet::three_levers::{march, ArmedLimiter, Limiter};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::valve_reference::{incidence_valve, m_lim, matched_phi, run, sibling};

// ---------------------------------------------------------------------------- rung 86's rig
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

fn tilt_map() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

/// The bare machine, through RUNG 64's builder (G1).
fn lt_on(lp: ComponentMap, hp: ComponentMap) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn lt() -> ScheduledStatorCore {
    lt_on(lp_map(), hp_map())
}

fn stat() -> LeverArm {
    LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)))
}

fn ramp(r: f64, ds: f64) -> Ramp {
    Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds }
}

fn phi_of(sm: f64) -> f64 {
    FLOOR * (1.0 + sm)
}

fn t_c() -> f64 {
    1.0 / FLOOR
}

fn same_traj(a: &[FuelPoint], b: &[FuelPoint], what: &str) {
    assert_eq!(a.len(), b.len(), "{what}: npts");
    for (p, q) in a.iter().zip(b) {
        assert!(p.s.to_bits() == q.s.to_bits() && p.mf.to_bits() == q.mf.to_bits()
                    && p.nu_lp.to_bits() == q.nu_lp.to_bits() && p.nu_hp.to_bits() == q.nu_hp.to_bits()
                    && p.phi_lp.to_bits() == q.phi_lp.to_bits() && p.phi_hp.to_bits() == q.phi_hp.to_bits()
                    && p.tt4.to_bits() == q.tt4.to_bits() && p.sp_thrust.to_bits() == q.sp_thrust.to_bits(),
                "{what}: differs at s={}", p.s);
    }
}

// =============================================================================================
// GATE 1 — THE DEVICE
// =============================================================================================

/// § 1: with the stators off `v ≡ 0`, and `φ_lim/(1 + 0·φ_lim)` is `φ_lim` exactly — so the
/// incidence valve IS rung 64's valve, every point and every re-solved opening, bit for bit. On a
/// riding cell AND a saturating one (shaped `r` = 0.25, `sm` 0.46).
#[test]
fn the_incidence_valve_with_the_stators_off_is_the_phi_valve_bit_for_bit() {
    let m = lt();
    for (r, sm) in [(0.5, 0.42), (0.25, 0.46)] {
        let (phi, inc) = (BleedLimiter::new(phi_of(sm), B), incidence_valve(phi_of(sm), B));
        assert_eq!(inc.target(0.0).to_bits(), phi.phi_lim.to_bits());
        let (a, b) = (sibling(&m, None, Some(phi)), sibling(&m, None, Some(inc)));
        let rp = ramp(r, DS);
        let (ta, _) = march(&a, &flight(), &rp, None);
        let (tb, _) = march(&b, &flight(), &rp, None);
        same_traj(&ta, &tb, &format!("r={r} sm={sm}"));
        for (p, q) in ta.iter().zip(&tb) {
            assert_eq!(a.b_at_point(&flight(), p).to_bits(), b.b_at_point(&flight(), q).to_bits());
        }
    }
}

/// The reference rides ON the floor, so every sibling constructor that keeps the floor keeps the
/// reference: `at_lever` (merged with the stator), `at_stator`, and rung 64's `isolating`.
#[test]
fn every_sibling_constructor_carries_the_reference() {
    let m = lt();
    let inc = incidence_valve(phi_of(0.42), B);
    let both = sibling(&m, Some(&stat()), Some(inc));
    assert_eq!(both.fuel.inner.lever.lim.map(|l| l.reference), Some(ValveReference::Incidence));
    let v = sibling(&m, None, Some(inc));
    let st = v.at_stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65)));
    assert_eq!(st.fuel.inner.lever.lim, Some(inc));
    let (_, armed) = m.isolating(&LeverArm::floored(inc), Some(&stat()));
    assert_eq!(armed.fuel.inner.lever.lim, Some(inc));
}

/// G1, recorded: rung 62's builder DROPS a floor without a word — the reason every rung-87
/// machine is built through rung 64's, and `sibling` asserts what it built.
#[test]
fn g1_rung62s_builder_drops_the_floor_silently() {
    let m62 = match build_scheduled_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                                          &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    let a = m62.at_lever(&LeverArm::floored(incidence_valve(phi_of(0.42), B)));
    assert!(a.fuel.inner.lever.lim.is_none() && !a.armed_bleed());
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sibling(&m62, None, Some(incidence_valve(phi_of(0.42), B)))
    }));
    assert!(caught.is_err(), "sibling must refuse a builder that drops the floor");
}

/// Rung 65's lagged valve and the readers of rungs 65–80 read `phi_lim` as a flow coefficient, so
/// a lagged incidence valve is refused, and so is an incidence valve reaching `r64_solve_b`.
#[test]
fn the_wrong_coordinate_routes_refuse_the_incidence_valve() {
    assert!(std::panic::catch_unwind(|| BleedLimiter::with_tau(phi_of(0.42), B, Some(0.1)).on_incidence())
                .is_err());
    let inc = incidence_valve(phi_of(0.42), B);
    struct S(f64);
    impl turbojet::limited_bleed::HasPhiLp for S {
        fn phi_lp(&self) -> f64 { self.0 }
    }
    assert!(std::panic::catch_unwind(|| r64_solve_b(&inc, |b| Ok(S(0.7 + b)))).is_err());
    let phi = BleedLimiter::new(phi_of(0.42), B);
    assert!(r64_solve_b(&phi, |b| Ok(S(0.7 + b))).is_ok());
}

// =============================================================================================
// GATE 2 — THE ARITHMETIC (no credit)
// =============================================================================================

/// D1 (rung 60's pin, one lever over): while riding, each valve holds ITS coordinate to rounding
/// — the incidence valve `M_i = m_lim`, the `φ` valve `φ = φ_lim`, so `M_i = m_lim + v`.
#[test]
fn d1_each_valve_pins_its_own_coordinate() {
    let m = lt();
    let rp = ramp(0.5, DS);
    let ml = m_lim(t_c(), phi_of(0.42));
    let inc = run(&sibling(&m, Some(&stat()), Some(incidence_valve(phi_of(0.42), B))), &flight(), &rp, None);
    assert!(inc.riding > 50 && inc.pin_dev < 1e-12, "{} {}", inc.riding, inc.pin_dev);
    assert!((inc.m_i - ml).abs() < 1e-12, "{} vs {ml}", inc.m_i);
    let phi = run(&sibling(&m, Some(&stat()), Some(BleedLimiter::new(phi_of(0.42), B))), &flight(), &rp, None);
    assert!(phi.riding > 20 && phi.pin_dev < 1e-12);
    assert!(phi.m_i > ml + 0.05, "the phi valve over-protects in incidence beside the stators: {}", phi.m_i);
}

/// D2: at `b = 0` the closed loop IS the shut run, so a set point at or below the shut run's own
/// minimum never opens the valve — the run is the shut run bit for bit. Shaped `r` = 0.5: the
/// stators alone clear `sm` 0.34 (`m_lim` 0.4613 < 0.4963) though without them the valve opens.
#[test]
fn d2_the_dormancy_ledger() {
    let m = lt();
    let rp = ramp(0.5, DS);
    let inc = incidence_valve(phi_of(0.34), B);
    let shut = sibling(&m, Some(&stat()), None);
    let (ts, _) = march(&shut, &flight(), &rp, None);
    let v = sibling(&m, Some(&stat()), Some(inc));
    let (tv, _) = march(&v, &flight(), &rp, None);
    same_traj(&ts, &tv, "dormant");
    let rv = run(&v, &flight(), &rp, None);
    assert!(rv.b_int == 0.0 && rv.riding == 0 && rv.saturated == 0);
    let off = run(&sibling(&m, None, Some(inc)), &flight(), &rp, None);
    assert!(off.riding > 0 && off.b_int > 0.0, "without the stators it opens");
}

/// D4: a `φ` valve keeps `M_i ≥ m_lim + v ≥ m_lim`, so an INCIDENCE fuel floor at the same `sm`
/// withholds exactly nothing while the valve is not saturated (the stators on).
#[test]
fn d4_an_incidence_fuel_floor_beside_the_phi_valve_is_dormant() {
    let m = lt();
    for (r, sm) in [(0.5, 0.38), (1.0, 0.46)] {
        let rp = ramp(r, DS);
        let b0 = sibling(&m, None, None);
        let incf = ArmedLimiter::on(&b0, &flight(), &rp, Spool::Lp, Limiter::IncidenceFloor { sm });
        let v = sibling(&m, Some(&stat()), Some(BleedLimiter::new(phi_of(sm), B)));
        let fr = run(&v, &flight(), &rp, Some(&incf));
        assert_eq!(fr.fuel_withheld, 0.0, "r={r} sm={sm}");
        assert_eq!(fr.fuel_cut_max, 0.0);
    }
}

// =============================================================================================
// GATE 3 — THE EXPERIMENT
// =============================================================================================

/// H1 + H2 — **HIT** (19 of 19 riding cells, both steps; the 20th saturated — H4's miss —
/// and has no protection to match; `docs/rung87-spec.md` § 3): at EQUAL incidence protection the
/// `φ` valve bleeds more and costs more thrust. Two cells here, the widest and the narrowest
/// bleed ratio of § 3's table (3.46 and 1.27).
#[test]
fn h1_h2_at_equal_protection_the_phi_valve_pays_more() {
    for (lp, hp, r, sm, ds, lo, hi) in [(lp_map(), hp_map(), 1.0, 0.42, DS, 3.3, 3.6),
                                        (lp_map(), hp_map(), 1.0, 0.42, DS / 2.0, 3.3, 3.6),
                                        (lp_map(), hp_map(), 1.0, 0.54, DS, 1.15, 1.40),
                                        (tilt_map(), tilt_map(), 1.0, 0.46, DS, 1.95, 2.25)] {
        let m = lt_on(lp, hp);
        let rp = ramp(r, ds);
        let shut = run(&sibling(&m, Some(&stat()), None), &flight(), &rp, None);
        let inc = run(&sibling(&m, Some(&stat()), Some(incidence_valve(phi_of(sm), B))), &flight(), &rp, None);
        assert!(inc.riding > 0 && inc.saturated == 0, "a scored cell rides");
        let (phi, pr) = matched_phi(&m, &flight(), &rp, Some(&stat()), B, inc.m_i, 1e-10)
            .expect("brackets");
        assert!((pr.m_i - inc.m_i).abs() < 1e-7, "matched protection");
        assert!(phi < phi_of(sm), "the matched phi set point sits BELOW the incidence one's name");
        assert!(pr.b_int > inc.b_int, "H1");
        assert!(pr.thrust_int - shut.thrust_int < inc.thrust_int - shut.thrust_int, "H2");
        let ratio = pr.b_int / inc.b_int;
        assert!(lo < ratio && ratio < hi, "r={r} sm={sm}: ratio {ratio}");
    }
}

/// H3 — **HIT** (11 of 11): beside the stators the incidence valve bleeds LESS than without them —
/// the opposite sign to the `φ` valve (G3).
#[test]
fn h3_the_stators_pay_part_of_the_incidence_valves_bill() {
    let m = lt();
    for (r, sm) in [(0.25, 0.34), (0.5, 0.46), (1.0, 0.42)] {
        let rp = ramp(r, DS);
        let iv = incidence_valve(phi_of(sm), B);
        let on = run(&sibling(&m, Some(&stat()), Some(iv)), &flight(), &rp, None);
        let off = run(&sibling(&m, None, Some(iv)), &flight(), &rp, None);
        assert!(on.riding > 0 && off.riding > 0, "scored only where both ride");
        assert!(on.b_int < off.b_int, "r={r} sm={sm}");
        let pv = BleedLimiter::new(phi_of(sm), B);
        let pon = run(&sibling(&m, Some(&stat()), Some(pv)), &flight(), &rp, None);
        assert!(pon.b_int > off.b_int, "and the phi valve the other way (G3)");
    }
}

/// H4 — **MISS** (the iff): the closed loop can saturate BELOW the full-open march's own minimum.
/// Shaped `r` = 0.25, `sm` 0.46, stators on: D2 put `m_lim` 0.5729 under the full-open 0.5818
/// (outside the band) and predicted R; the valve reaches `b_max` and holds only ~0.5724. A valve
/// that opens late has given up the bleed's earlier effect on the spool, which the full-open run
/// banked.
#[test]
fn h4_miss_the_closed_loop_saturates_below_the_full_open_ceiling() {
    let m = lt();
    let rp = ramp(0.25, DS);
    let ml = m_lim(t_c(), phi_of(0.46));
    let full = run(&m.at_lever(&LeverArm::merged(&stat(), &LeverArm::constant(B))), &flight(), &rp, None);
    assert!(full.m_i > ml + 0.005, "D2: outside the band, predicted to ride: {} vs {ml}", full.m_i);
    let inc = run(&sibling(&m, Some(&stat()), Some(incidence_valve(phi_of(0.46), B))), &flight(), &rp, None);
    assert!(inc.saturated > 0 && inc.m_i < ml, "MISS: it saturates, {} < {ml}", inc.m_i);
}

/// H5 — **HIT**: an incidence valve beside a `φ` FUEL floor at the same `sm`, stators on — the
/// valve lets `φ` fall to `φ_lim/(1+v·φ_lim)`, the fuel floor wakes and starves the run; with an
/// incidence fuel floor the same valve runs to the end.
#[test]
fn h5_mismatched_references_hand_the_work_to_the_fuel() {
    let m = lt();
    for (r, sm) in [(0.5, 0.38), (1.0, 0.46)] {
        let rp = ramp(r, DS);
        let b0 = sibling(&m, None, None);
        let phif = ArmedLimiter::on(&b0, &flight(), &rp, Spool::Lp, Limiter::PhiFloor { sm });
        let incf = ArmedLimiter::on(&b0, &flight(), &rp, Spool::Lp, Limiter::IncidenceFloor { sm });
        let v = sibling(&m, Some(&stat()), Some(incidence_valve(phi_of(sm), B)));
        let mixed = run(&v, &flight(), &rp, Some(&phif));
        let matched = run(&v, &flight(), &rp, Some(&incf));
        let full = run(&v, &flight(), &rp, None).npts;
        assert_eq!(matched.npts, full, "matched references run to the end");
        assert!(mixed.npts < full / 2 && mixed.fuel_withheld > 0.0, "r={r} sm={sm}: {} pts", mixed.npts);
        assert!(mixed.thrust_int < matched.thrust_int);
    }
}
