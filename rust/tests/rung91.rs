//! RUNG 91 — TURBINE COOLING AIR. Anchor: `docs/plans/rung91-anchor-turbine-cooling.md`; spec:
//! `docs/rung91-spec.md`.
//!
//! This file holds the CONSTRUCTION gates (anchor § 3, "no credit"): the reduce, the plant gate
//! (transient equilibrium ≡ steady match), the cascade's closure (mixers, shaft balances, the
//! untouched LP turbine), the mass identities, the refusals, the sibling carry, and the dispatch.
//! The scored readings are the spec's tables; the gates that pin them are below the construction
//! block.

use turbojet::bleed_transient::{build_scheduled_bleed, LeverArm};
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorArm};
use turbojet::turbine_cooling::{self, TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses,
                          TwoSpoolMapMatcher, TwoSpoolMapResult};
use turbojet::two_spool_transient::Instant2;

// ---------------------------------------------------------------------------- the rig (rung 42's)
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL) }

fn with(cb: Option<CustomerBleed>, tc: Option<TurbineCooling>) -> TwoSpoolEngine {
    let mut e = design();
    if let Some(c) = cb { e = e.with_customer_bleed(c); }
    if let Some(t) = tc { e = e.with_turbine_cooling(t); }
    e
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(0.55)
}
fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(0.55)
}
fn flat() -> ComponentMap { ComponentMap::flat() }

fn steady_keys(r: &TwoSpoolMapResult) -> [(&'static str, f64); 9] {
    [("phi_lp", r.phi_lp), ("phi_hp", r.phi_hp), ("n_lp", r.n_lp), ("n_hp", r.n_hp),
     ("pi_lpc", r.base.pi_lpc), ("pi_hpc", r.base.pi_hpc), ("thrust", r.base.thrust),
     ("mdot_air", r.base.mdot_air), ("eta_hpc", r.eta_hpc)]
}

fn inst_keys(i: &Instant2) -> [(&'static str, f64); 12] {
    [("nu_lp", i.nu_lp), ("nu_hp", i.nu_hp), ("phi_lp", i.close.phi_lp),
     ("phi_hp", i.close.phi_hp), ("Tt4", i.tt4), ("f", i.close.f), ("pi_lpc", i.close.pi_lpc),
     ("pi_hpc", i.close.pi_hpc), ("Phi_lp", i.phi_lp_dot), ("Phi_hp", i.phi_hp_dot),
     ("sp_thrust", i.sp_thrust), ("Tt3", i.close.tt3)]
}

fn machine(cb: Option<CustomerBleed>, tc: Option<TurbineCooling>, arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(with(cb, tc), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

const INACTIVE: [Option<TurbineCooling>; 2] = [None, Some(TurbineCooling { vane: 0.0, blade: 0.0 })];
const VANE: TurbineCooling = TurbineCooling { vane: 0.05, blade: 0.0 };
const BLADE: TurbineCooling = TurbineCooling { vane: 0.0, blade: 0.05 };
const BOTH: TurbineCooling = TurbineCooling { vane: 0.06, blade: 0.03 };

// =============================================================================================
// THE REDUCE — inactive ⇒ the earlier code, bit for bit, by dispatch
// =============================================================================================

#[test]
fn reduce_steady_inactive_is_rung90_bit_for_bit() {
    for b in [0.0, 0.05] {
        for cb in [None, Some(CustomerBleed::share(0.05))] {
            let r90 = CustomerBleedMatcher::new(with(cb, None), flight(), 1.0, lp(), hp(), b);
            for tc in INACTIVE {
                let m = TurbineCoolingMatcher::new(with(cb, tc), flight(), 1.0, lp(), hp(), b);
                for tt4 in [1500.0, 1200.0, 1000.0] {
                    let want = r90.match_point(&flight(), tt4).base;
                    let got = m.match_point(&flight(), tt4);
                    assert!(got.booking.is_none(), "inactive cooling books nothing");
                    for ((k, a), (_, w)) in steady_keys(&got.base).into_iter().zip(steady_keys(&want)) {
                        assert!(a.to_bits() == w.to_bits(),
                                "b={b} {cb:?} {tc:?} Tt4={tt4} {k}: {a:?} != {w:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn reduce_transient_inactive_is_rung90_or_rung62_bit_for_bit() {
    for arm in [LeverArm::default(), LeverArm::floored(BleedLimiter::new(1.38 * 0.55, 0.10))] {
        for cb in [None, Some(CustomerBleed::share(0.05))] {
            let want = machine(cb, None, &arm);
            for tc in INACTIVE {
                let got = machine(cb, tc, &arm);
                for tt4 in [1400.0, 1000.0] {
                    let (a, w) = (got.fuel.inner.equilibrium(&flight(), tt4),
                                  want.fuel.inner.equilibrium(&flight(), tt4));
                    for ((k, x), (_, y)) in inst_keys(&a).into_iter().zip(inst_keys(&w)) {
                        assert!(x.to_bits() == y.to_bits(), "{cb:?} {tc:?} Tt4={tt4} {k}: {x:?} != {y:?}");
                    }
                    assert!(a.close.cooling.is_none(), "inactive cooling reaches a closure");
                }
                let mf = want.fuel.fuel_for_tt4(&flight(), 1200.0);
                assert!(mf.to_bits() == got.fuel.fuel_for_tt4(&flight(), 1200.0).to_bits());
                let (a, w) = (got.fuel.instant_fuel(&flight(), 0.85, 0.88, mf),
                              want.fuel.instant_fuel(&flight(), 0.85, 0.88, mf));
                for ((k, x), (_, y)) in inst_keys(&a.base).into_iter().zip(inst_keys(&w.base)) {
                    assert!(x.to_bits() == y.to_bits(), "fuel path {cb:?} {tc:?} {k}: {x:?} != {y:?}");
                }
            }
        }
    }
}

#[test]
fn the_design_run_ignores_the_cooling() {
    let (a, b) = (design().run(&flight(), 1.0), with(None, Some(BOTH)).run(&flight(), 1.0));
    for s in ["3", "4", "45", "9"] {
        assert!(a.station(s).tt.to_bits() == b.station(s).tt.to_bits());
        assert!(a.station(s).mdot.to_bits() == b.station(s).mdot.to_bits());
    }
}

// =============================================================================================
// THE PLANT GATE — the transient's equilibrium IS the steady match (rung 62's lesson)
// =============================================================================================

fn plant_gate(cb: Option<CustomerBleed>, tc: TurbineCooling, b: f64, tt4: f64) {
    let arm = if b == 0.0 { LeverArm::default() } else { LeverArm::constant(b) };
    let eq = machine(cb, Some(tc), &arm).fuel.inner.equilibrium(&flight(), tt4);
    let od = TurbineCoolingMatcher::new(with(cb, Some(tc)), flight(), 1.0, lp(), hp(), b)
        .match_point(&flight(), tt4);
    let k = od.booking.expect("active");
    for (name, got, want) in [("n_lp", eq.close.n_lp, od.base.n_lp),
                              ("n_hp", eq.close.n_hp, od.base.n_hp),
                              ("phi_lp", eq.close.phi_lp, od.base.phi_lp),
                              ("phi_hp", eq.close.phi_hp, od.base.phi_hp),
                              ("pi_lpc", eq.close.pi_lpc, od.base.base.pi_lpc),
                              ("pi_hpc", eq.close.pi_hpc, od.base.base.pi_hpc),
                              ("pi_hpt", eq.pi_hpt, od.base.base.pi_hpt),
                              ("tau_hpt", eq.tau_hpt, od.base.base.tau_hpt),
                              ("Tt45", eq.tt45, k.tt45),
                              ("burner", eq.close.mdot_air, k.mdot_burner),
                              ("face", eq.close.mdot_face.unwrap(), od.base.base.mdot_air)] {
        assert!((got / want - 1.0).abs() < 1e-9,
                "{cb:?} {tc:?} b={b} Tt4={tt4} {name}: transient {got:?} vs steady {want:?}");
    }
    let share = eq.close.customer.unwrap();
    assert!((share - k.customer).abs() < 1e-9, "customer share: {share} vs {}", k.customer);
    let thrust = eq.sp_thrust_inlet.unwrap() * eq.close.mdot_face.unwrap();
    assert!((thrust / od.base.base.thrust - 1.0).abs() < 1e-9,
            "{cb:?} {tc:?} b={b} Tt4={tt4} thrust: {thrust} vs {}", od.base.base.thrust);
}

#[test] fn plant_gate_vane_1400() { plant_gate(None, VANE, 0.0, 1400.0); }
#[test] fn plant_gate_vane_1000() { plant_gate(None, VANE, 0.0, 1000.0); }
#[test] fn plant_gate_blade_1400() { plant_gate(None, BLADE, 0.0, 1400.0); }
#[test] fn plant_gate_blade_1000() { plant_gate(None, BLADE, 0.0, 1000.0); }
#[test] fn plant_gate_both_with_valve() { plant_gate(None, BOTH, 0.05, 1200.0); }
#[test] fn plant_gate_both_with_customer_share() {
    plant_gate(Some(CustomerBleed::share(0.05)), BOTH, 0.0, 1200.0);
}
#[test] fn plant_gate_both_with_customer_amount_and_valve() {
    plant_gate(Some(CustomerBleed::amount(0.05)), BOTH, 0.05, 1200.0);
}

// =============================================================================================
// THE CASCADE CLOSES — mixers, both shaft balances, and the LP turbine untouched (CPG)
// =============================================================================================

#[test]
fn the_cooled_point_balances_both_shafts_and_both_mixers() {
    let g = cpg();
    for (cb, tc, b) in [(None, VANE, 0.0), (None, BLADE, 0.0),
                        (Some(CustomerBleed::share(0.04)), BOTH, 0.05)] {
        let m = TurbineCoolingMatcher::new(with(cb, Some(tc)), flight(), 1.0, lp(), hp(), b);
        for tt4 in [1500.0, 1100.0] {
            let od = m.match_point(&flight(), tt4);
            let k = od.booking.unwrap();
            let s = &od.base.base;
            let st = |l: &str| *s.station(l);
            let (s2, s25, s3, s4, s41, s44, s45, s5) = (st("2"), st("25"), st("3"), st("4"),
                                                         st("41"), st("44"), st("45"), st("5"));
            // Mass: the burner sees the HPC air less the customer and both streams; the nozzle
            // passes the burner gas and both streams.
            let burner = (1.0 - k.customer - tc.total()) * k.mdot_hpc;
            assert!((k.mdot_burner / burner - 1.0).abs() < 1e-12, "{tc:?} burner air");
            assert!((s3.mdot / k.mdot_hpc - 1.0).abs() < 1e-12);
            assert!((s45.mdot / (s4.mdot + k.mdot_vane + k.mdot_blade) - 1.0).abs() < 1e-12);
            assert!((k.mdot_nozzle_air / (k.mdot_burner + k.mdot_vane + k.mdot_blade) - 1.0).abs() < 1e-12);
            // Mixer energy.
            let e1 = s41.mdot * g.h_t(s41.tt, s41.far) - s4.mdot * g.h_t(s4.tt, s4.far)
                - k.mdot_vane * g.h_c(s3.tt);
            let e2 = s45.mdot * g.h_t(s45.tt, s45.far) - s44.mdot * g.h_t(s44.tt, s44.far)
                - k.mdot_blade * g.h_c(s3.tt);
            assert!(e1.abs() < 1e-9 * s41.mdot * g.h_t(s41.tt, s41.far) && e2.abs() < 1e-9 * s45.mdot * g.h_t(s45.tt, s45.far));
            // Shaft balances, as the rebuild books them.
            let hp_c = s3.mdot * (g.h_c(s3.tt) - g.h_c(s25.tt));
            let hp_t = 0.99 * s41.mdot * (g.h_t(s41.tt, s41.far) - g.h_t(s44.tt, s44.far));
            assert!((hp_t / hp_c - 1.0).abs() < 1e-9, "{tc:?} Tt4={tt4} HP balance {hp_t} vs {hp_c}");
            let lp_c = s2.mdot * (g.h_c(s25.tt) - g.h_c(s2.tt));
            let lp_t = 0.99 * s45.mdot * (g.h_t(s45.tt, s45.far) - g.h_t(s5.tt, s45.far));
            assert!((lp_t / lp_c - 1.0).abs() < 1e-9, "{tc:?} Tt4={tt4} LP balance {lp_t} vs {lp_c}");
            // The rebuild agrees with the cascade on the HP turbine and the mixers.
            assert!((s45.tt / k.tt45 - 1.0).abs() < 1e-12 && (s44.tt / k.tt44 - 1.0).abs() < 1e-12);
            assert!((s44.tt / s41.tt / s.tau_hpt - 1.0).abs() < 1e-9, "rotor tau: rebuild vs pin");
            assert!((s45.pt / s4.pt / s.pi_hpt - 1.0).abs() < 1e-9, "pi_hpt: rebuild vs pin");
            assert!(s41.tt < s4.tt || tc.vane == 0.0, "vane air cools the rotor inlet");
        }
    }
}

#[test]
fn the_lp_turbine_does_not_move_on_the_perfect_gas() {
    let m0 = TurbineCoolingMatcher::new(design(), flight(), 1.0, flat(), flat(), 0.0);
    for tc in [VANE, BLADE, BOTH] {
        let m = TurbineCoolingMatcher::new(with(None, Some(tc)), flight(), 1.0, flat(), flat(), 0.0);
        for tt4 in [1500.0, 1000.0] {
            let (a, z) = (m.match_point(&flight(), tt4).base.base, m0.match_point(&flight(), tt4).base.base);
            assert!((a.tau_lpt / z.tau_lpt - 1.0).abs() < 1e-11 && (a.pi_lpt / z.pi_lpt - 1.0).abs() < 1e-11,
                    "{tc:?} Tt4={tt4}: LPT moved ({}, {}) vs ({}, {})", a.tau_lpt, a.pi_lpt, z.tau_lpt, z.pi_lpt);
            assert!(a.pi_hpt > z.pi_hpt, "{tc:?}: the HP turbine expands LESS (A45 back-pressured)");
        }
    }
}

// =============================================================================================
// REFUSALS — an explicit demand is never silently dropped
// =============================================================================================

#[test]
#[should_panic(expected = "rung-91")]
fn rung90_refuses_active_cooling() {
    CustomerBleedMatcher::new(with(Some(CustomerBleed::share(0.05)), Some(VANE)), flight(), 1.0,
                              lp(), hp(), 0.0).match_point(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "rung-91")]
fn rung39_refuses_active_cooling() {
    TwoSpoolMapMatcher::new(with(None, Some(BLADE)), flight(), 1.0, lp(), hp()).match_point(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "rung-91")]
fn rung42_refuses_active_cooling() {
    TwoSpoolBleedMatcher::new(with(None, Some(BLADE)), flight(), 1.0, lp(), hp(), 0.05)
        .match_point(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "rung-91")]
fn rung57_refuses_active_cooling() {
    let c = match ScheduledStatorTransient::new(with(None, Some(VANE)), flight(), 1.0, Some(lp()),
                                                Some(hp()), 1.0, StatorArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    c.fuel.inner.equilibrium(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "perfect (constant-property) gas only")]
fn a_thermally_perfect_gas_is_refused() {
    build_two_spool_turbojet(Gas::thermally_perfect(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
        .with_turbine_cooling(VANE);
}

#[test]
#[should_panic(expected = "rung-91 blade cooling share")]
fn a_share_past_a_quarter_is_refused() {
    TurbineCooling::new(0.0, 0.25);
}

#[test]
fn together_past_half_is_refused_at_the_root_in_words() {
    let m = TurbineCoolingMatcher::new(with(Some(CustomerBleed::share(0.3)), Some(TurbineCooling::new(0.12, 0.1))),
                                       flight(), 1.0, flat(), flat(), 0.0);
    let e = m.try_match_point(&flight(), 1500.0).err().expect("0.3 + 0.22 must be refused");
    assert!(e.0.contains("more than half"), "{}", e.0);
}

// =============================================================================================
// SIBLINGS KEEP IT, AND THE DISPATCH IS LIVE
// =============================================================================================

#[test]
fn every_sibling_keeps_the_cooling() {
    let m = machine(None, Some(BOTH), &LeverArm::floored(BleedLimiter::new(1.38 * 0.55, 0.10)));
    for sib in [m.at_lever(&LeverArm::default()), m.at_lever(&LeverArm::constant(0.03))] {
        assert!(sib.fuel.inner.inner.base.cooling == Some(BOTH));
    }
    let s62 = match build_scheduled_bleed(with(None, Some(BOTH)), flight(), 1.0, Some(lp()), Some(hp()),
                                          1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    assert!(s62.at_lever(&LeverArm::constant(0.02)).fuel.inner.inner.base.cooling == Some(BOTH));
}

#[test]
fn the_dispatch_is_live_on_both_sides() {
    turbine_cooling::reset_counts();
    let m = machine(Some(CustomerBleed::share(0.05)), None, &LeverArm::default());
    m.fuel.inner.equilibrium(&flight(), 1200.0);
    assert_eq!(turbine_cooling::counts()[2..], [0, 0, 0, 0], "no cooling must never reach rung 91");
    let m = machine(None, Some(BOTH), &LeverArm::default());
    m.fuel.inner.equilibrium(&flight(), 1200.0);
    let mf = m.fuel.fuel_for_tt4(&flight(), 1200.0);
    m.fuel.instant_fuel(&flight(), 0.85, 0.88, mf);
    let c = turbine_cooling::counts();
    assert!(c[2] > 0 && c[3] > 0 && c[4] > 0 && c[5] > 0, "every rung-91 cell must run: {c:?}");
    turbine_cooling::reset_counts();
    let s = TurbineCoolingMatcher::new(design(), flight(), 1.0, flat(), flat(), 0.0);
    s.match_point(&flight(), 1200.0);
    let a = TurbineCoolingMatcher::new(with(None, Some(VANE)), flight(), 1.0, flat(), flat(), 0.0);
    a.match_point(&flight(), 1200.0);
    let c = turbine_cooling::counts();
    assert!(c[0] == 1 && c[1] == 1, "steady: one active, one inactive: {c:?}");
}

// =============================================================================================
// THE SCORED READINGS (anchor § 3, scored in docs/rung91-spec.md § 3) — one gate per claim, on
// a subset of the scored grid. Bars are the ANCHOR's (e46ab3d).
// =============================================================================================

/// The anchor's linear model on the uncooled point, per unit of the stream `(e1, e2)`:
/// `(d ln pi_HPT, d ln phi_L, d ln phi_H)`.
fn model(r0: &TwoSpoolMapResult, e1: f64, e2: f64) -> (f64, f64, f64) {
    let s = &r0.base;
    let (tt2, tt25, tt3, tt4, tt45) = (s.station("2").tt, s.station("25").tt, s.station("3").tt,
                                       s.tt4, s.station("45").tt);
    let f = s.station("4").far;
    let (kappa, g_t) = (1004.0 / 1239.0, 0.3 / 1.3);
    let a = |tau: f64, eta: f64| 3.5 * eta * (tau - 1.0) / (1.0 + eta * (tau - 1.0));
    let (tl, th) = (tt25 / tt2, tt3 / tt25);
    let (al, ah, t_l) = (a(tl, r0.eta_lpc), a(th, r0.eta_hpc), (tl - 1.0) / tl);
    let sigma = r0.eta_hpt * g_t * s.pi_hpt.powf(g_t) / s.tau_hpt;
    let rho = s.tau_hpt * sigma / (1.0 - s.tau_hpt);
    let (kt, ktp) = (kappa * tt3 / tt4, kappa * tt3 / tt45);
    let (m1, m2, e) = (e1 / (1.0 + f), e2 / (1.0 + f), e1 + e2);
    let p = (e / (1.0 + f) + 0.5 * m1 * (kt - 1.0) + 0.5 * m2 * (ktp - 1.0)) / (1.0 - 0.5 * sigma);
    let h = -e + m1 * kt - rho * p;
    let l = -e + m1 * kt + sigma * p + m2 * ktp;
    (p, al * l + ah * (h - t_l * l) + e - 0.5 * l, (ah - 0.5) * (h - t_l * l) + 0.5 * t_l * l + e)
}

/// `(d ln pi_HPT, d ln phi_L, d ln phi_H)` per unit stream, Richardson-extrapolated from 0.01 and
/// 0.02 (anchor § 2), and the uncooled point.
fn sens(unit: (f64, f64), ml: ComponentMap, mh: ComponentMap, tt4: f64) -> (f64, f64, f64, TwoSpoolMapResult) {
    let at = |e: f64| {
        let tc = (e > 0.0).then(|| TurbineCooling::new(e * unit.0, e * unit.1));
        TurbineCoolingMatcher::new(with(None, tc), flight(), 1.0, ml, mh, 0.0).match_point(&flight(), tt4).base
    };
    let (r0, r1, r2) = (at(0.0), at(0.01), at(0.02));
    let rich = |x0: f64, x1: f64, x2: f64| 2.0 * (x1 / x0).ln() / 0.01 - (x2 / x0).ln() / 0.02;
    (rich(r0.base.pi_hpt, r1.base.pi_hpt, r2.base.pi_hpt), rich(r0.phi_lp, r1.phi_lp, r2.phi_lp),
     rich(r0.phi_hp, r1.phi_hp, r2.phi_hp), r0)
}

const UNIT_VANE: (f64, f64) = (1.0, 0.0);
const UNIT_BLADE: (f64, f64) = (0.0, 1.0);

/// A1 + A2 + A3: the model holds on the flat maps (bars 0.03 turbine, 0.03 HP, 0.05 LP; measured
/// ≤ 0.0006, 0.0043, 0.0062).
#[test]
fn a1_a2_a3_the_linear_model_predicts_the_turbine_and_both_spools() {
    for unit in [UNIT_VANE, UNIT_BLADE] {
        for tt4 in [1500.0, 1200.0, 1000.0, 800.0, 700.0] {
            let (dp, dl, dh, r0) = sens(unit, flat(), flat(), tt4);
            let (mp, ml, mh) = model(&r0, unit.0, unit.1);
            assert!((dp - mp).abs() < 0.03, "A1 {unit:?} Tt4={tt4}: pi_HPT {dp} vs model {mp}");
            assert!((dh - mh).abs() < 0.03, "A2 {unit:?} Tt4={tt4}: HP {dh} vs model {mh}");
            assert!((dl - ml).abs() < 0.05, "A3 {unit:?} Tt4={tt4}: LP {dl} vs model {ml}");
        }
    }
}

/// A4 + A5 + A6: the HP arrow reverses at high power, crossing ONCE (vane 1120 ± 40 K, blade
/// 798 ± 25 K; measured 1116 / 795); the LP never crosses; blade below vane on both spools.
#[test]
fn a4_a5_a6_the_hp_reverses_the_lp_never_recovers_and_blade_is_worse() {
    let grid = [1500.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 800.0, 750.0, 700.0, 650.0];
    let mut crossings = [Vec::new(), Vec::new()];
    let mut prev: [Option<(f64, f64)>; 2] = [None, None];
    for tt4 in grid {
        let (_, vl, vh, _) = sens(UNIT_VANE, flat(), flat(), tt4);
        let (_, bl, bh, _) = sens(UNIT_BLADE, flat(), flat(), tt4);
        assert!(vl < 0.0 && bl < 0.0, "A5 Tt4={tt4}: LP vane {vl} blade {bl}");
        assert!(bl < vl && bh < vh, "A6 Tt4={tt4}: blade ({bl}, {bh}) vs vane ({vl}, {vh})");
        for (i, h) in [vh, bh].into_iter().enumerate() {
            if let Some((t0, h0)) = prev[i] {
                if (h0 < 0.0) != (h < 0.0) { crossings[i].push(t0 + (tt4 - t0) * h0 / (h0 - h)); }
            }
            prev[i] = Some((tt4, h));
        }
    }
    assert!(prev[0].unwrap().1 > 0.0 && sens(UNIT_VANE, flat(), flat(), 1500.0).2 < 0.0, "A4 vane: negative above, positive below");
    assert!(crossings[0].len() == 1 && (crossings[0][0] - 1120.0).abs() < 40.0, "A4 vane {:?}", crossings[0]);
    assert!(crossings[1].len() == 1 && (crossings[1][0] - 798.0).abs() < 25.0, "A4 blade {:?}", crossings[1]);
}

/// A7 (shaped, sign only): both spools negative at 1500 K; at 650 K the LP negative, the HP positive.
#[test]
fn a7_the_signs_survive_the_shaped_maps() {
    let (l, h) = (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() },
                  ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() });
    for unit in [UNIT_VANE, UNIT_BLADE] {
        let (_, dl, dh, _) = sens(unit, l, h, 1500.0);
        assert!(dl < 0.0 && dh < 0.0, "A7 {unit:?} 1500 K: LP {dl} HP {dh}");
        let (_, dl, dh, _) = sens(unit, l, h, 650.0);
        assert!(dl < 0.0 && dh > 0.0, "A7 {unit:?} 650 K: LP {dl} HP {dh}");
    }
}

/// B1–B5 on one cell of rig B (shaped maps, r = 0.5, ds 0.005, rung 64's valve at sm 0.38, where
/// it rides in every run).
#[test]
fn b1_to_b5_the_valve_pays_more_for_cooling_than_for_the_customer() {
    use turbojet::stator_transient::Ramp;
    let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.005 };
    let read = |cb: Option<CustomerBleed>, tc: Option<TurbineCooling>| {
        let m = machine(cb, tc, &LeverArm::default());
        let shut = m.bill_cell(&flight(), &rp, false);
        let full = m.at_lever(&LeverArm::constant(0.10)).bill_cell(&flight(), &rp, false);
        let v = m.at_lever(&LeverArm::floored(BleedLimiter::new(1.38 * 0.55, 0.10)))
            .bill_cell(&flight(), &rp, false);
        assert!(v.b_int > 0.0 && v.b_peak < 0.10, "the valve must RIDE at sm 0.38: {cb:?} {tc:?}");
        (shut.min_phi_lp, shut.min_phi_hp, full.min_phi_lp, v.b_int, v.npts)
    };
    let n = read(None, None);
    let v = read(None, Some(TurbineCooling::new(0.05, 0.0)));
    let b = read(None, Some(TurbineCooling::new(0.0, 0.05)));
    let c = read(Some(CustomerBleed::share(0.05)), None);
    assert!(n.4 == v.4 && v.4 == b.4 && b.4 == c.4, "bills compared only at equal run length");
    assert!(v.0 < n.0 && b.0 < n.0, "B1 {v:?} {b:?} vs {n:?}");
    assert!(b.1 < n.1, "B2 {b:?} vs {n:?}");
    assert!(v.3 > n.3 && b.3 > n.3, "B3 {v:?} {b:?} vs {n:?}");
    assert!(v.2 < n.2 && b.2 < n.2, "B4 {v:?} {b:?} vs {n:?}");
    assert!(b.3 > v.3 && v.3 > c.3, "B5 blade {b:?} > vane {v:?} > customer {c:?}");
}
