//! RUNG 92 — SHAFT-POWER OFFTAKE. Anchor: `docs/plans/rung92-anchor-shaft-offtake.md`; spec:
//! `docs/rung92-spec.md`.
//!
//! This file holds the CONSTRUCTION gates (anchor § 3, "no credit"): the reduce, the two exact
//! identities (a bleed is a power offtake plus air thrown away), the untouched turbine pins, the
//! plant gate (transient equilibrium ≡ steady match) over every lower path, the shaft balances
//! with the load, the refusals, the sibling carry, and the dispatch. The scored readings are the
//! spec's tables; the gates that pin them are below the construction block.

use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::bleed_transient::{build_scheduled_bleed, LeverArm};
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::shaft_offtake::{self, Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorArm};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
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

fn with(cb: Option<CustomerBleed>, tc: Option<TurbineCooling>, so: Option<ShaftOfftake>) -> TwoSpoolEngine {
    let mut e = design();
    if let Some(c) = cb { e = e.with_customer_bleed(c); }
    if let Some(t) = tc { e = e.with_turbine_cooling(t); }
    if let Some(o) = so { e = e.with_shaft_offtake(o); }
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

fn machine(cb: Option<CustomerBleed>, tc: Option<TurbineCooling>, so: Option<ShaftOfftake>,
           arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(with(cb, tc, so), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn matcher(cb: Option<CustomerBleed>, tc: Option<TurbineCooling>, so: Option<ShaftOfftake>,
           ml: ComponentMap, mh: ComponentMap, b: f64) -> ShaftOfftakeMatcher {
    ShaftOfftakeMatcher::new(with(cb, tc, so), flight(), 1.0, ml, mh, b)
}

const INACTIVE: [Option<ShaftOfftake>; 3] = [
    None,
    Some(ShaftOfftake { hp: Some(Demand::Share(0.0)), lp: None }),
    Some(ShaftOfftake { hp: Some(Demand::Power(0.0)), lp: Some(Demand::Share(0.0)) }),
];
const HP_SHARE: ShaftOfftake = ShaftOfftake { hp: Some(Demand::Share(0.06)), lp: None };
const HP_POWER: ShaftOfftake = ShaftOfftake { hp: Some(Demand::Power(0.03)), lp: None };
const LP_SHARE: ShaftOfftake = ShaftOfftake { hp: None, lp: Some(Demand::Share(0.10)) };
const LP_POWER: ShaftOfftake = ShaftOfftake { hp: None, lp: Some(Demand::Power(0.02)) };
const VANE: TurbineCooling = TurbineCooling { vane: 0.05, blade: 0.0 };

// =============================================================================================
// THE REDUCE — inactive ⇒ the earlier code, bit for bit, by dispatch
// =============================================================================================

#[test]
fn reduce_steady_inactive_is_rung91_bit_for_bit() {
    for b in [0.0, 0.05] {
        for (cb, tc) in [(None, None), (Some(CustomerBleed::share(0.05)), None), (None, Some(VANE))] {
            let r91 = TurbineCoolingMatcher::new(with(cb, tc, None), flight(), 1.0, lp(), hp(), b);
            for so in INACTIVE {
                let m = matcher(cb, tc, so, lp(), hp(), b);
                for tt4 in [1500.0, 1200.0, 1000.0] {
                    let want = r91.match_point(&flight(), tt4).base;
                    let got = m.match_point(&flight(), tt4);
                    assert!(got.booking.is_none(), "an inactive load books nothing");
                    for ((k, a), (_, w)) in steady_keys(&got.base).into_iter().zip(steady_keys(&want)) {
                        assert!(a.to_bits() == w.to_bits(),
                                "b={b} {cb:?} {tc:?} {so:?} Tt4={tt4} {k}: {a:?} != {w:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn reduce_transient_inactive_is_the_earlier_cells_bit_for_bit() {
    for arm in [LeverArm::default(), LeverArm::floored(BleedLimiter::new(1.38 * 0.55, 0.10))] {
        for (cb, tc) in [(None, None), (Some(CustomerBleed::share(0.05)), None), (None, Some(VANE))] {
            let want = machine(cb, tc, None, &arm);
            for so in INACTIVE {
                let got = machine(cb, tc, so, &arm);
                for tt4 in [1400.0, 1000.0] {
                    let (a, w) = (got.fuel.inner.equilibrium(&flight(), tt4),
                                  want.fuel.inner.equilibrium(&flight(), tt4));
                    for ((k, x), (_, y)) in inst_keys(&a).into_iter().zip(inst_keys(&w)) {
                        assert!(x.to_bits() == y.to_bits(), "{cb:?} {tc:?} {so:?} Tt4={tt4} {k}: {x:?} != {y:?}");
                    }
                    assert!(a.close.offtake.is_none(), "an inactive load marks a closure");
                }
                let mf = want.fuel.fuel_for_tt4(&flight(), 1200.0);
                assert!(mf.to_bits() == got.fuel.fuel_for_tt4(&flight(), 1200.0).to_bits());
                let (a, w) = (got.fuel.instant_fuel(&flight(), 0.85, 0.88, mf),
                              want.fuel.instant_fuel(&flight(), 0.85, 0.88, mf));
                for ((k, x), (_, y)) in inst_keys(&a.base).into_iter().zip(inst_keys(&w.base)) {
                    assert!(x.to_bits() == y.to_bits(), "fuel path {cb:?} {tc:?} {so:?} {k}: {x:?} != {y:?}");
                }
            }
        }
    }
}

#[test]
fn the_design_run_ignores_the_load() {
    let (a, b) = (design().run(&flight(), 1.0),
                  with(None, None, Some(ShaftOfftake::both(Demand::share(0.1), Demand::power(0.05))))
                      .run(&flight(), 1.0));
    for s in ["3", "4", "45", "9"] {
        assert!(a.station(s).tt.to_bits() == b.station(s).tt.to_bits());
        assert!(a.station(s).mdot.to_bits() == b.station(s).mdot.to_bits());
    }
}

// =============================================================================================
// THE TWO IDENTITIES (flat maps, perfect gas) — a bleed is a power offtake plus air thrown away
// =============================================================================================

fn same(what: &str, got: f64, want: f64) {
    assert!((got / want - 1.0).abs() < 1e-9, "{what}: offtake {got:?} vs bleed {want:?}");
}

fn same_core(what: &str, a: &TwoSpoolMapResult, z: &TwoSpoolMapResult) {
    for (k, x, y) in [("pi_lpc", a.base.pi_lpc, z.base.pi_lpc), ("pi_hpc", a.base.pi_hpc, z.base.pi_hpc),
                      ("tau_lpc", a.base.tau_lpc, z.base.tau_lpc), ("tau_hpc", a.base.tau_hpc, z.base.tau_hpc),
                      ("n_lp", a.n_lp, z.n_lp), ("n_hp", a.n_hp, z.n_hp),
                      ("pi_hpt", a.base.pi_hpt, z.base.pi_hpt), ("tau_hpt", a.base.tau_hpt, z.base.tau_hpt),
                      ("pi_lpt", a.base.pi_lpt, z.base.pi_lpt), ("Tt3", a.base.station("3").tt, z.base.station("3").tt)] {
        same(&format!("{what} {k}"), x, y);
    }
}

/// An LP-shaft share `s` IS rung 42's interstage bleed `b = s` less the dumped air: every
/// pressure ratio, temperature ratio and speed identical; `φ_H` identical; `φ_L` by `(1−s)`.
#[test]
fn identity_an_lp_share_is_rung42s_bleed_less_the_air() {
    for s in [0.04, 0.12] {
        let m = matcher(None, None, Some(ShaftOfftake::lp(Demand::share(s))), flat(), flat(), 0.0);
        let r42 = TwoSpoolBleedMatcher::new(design(), flight(), 1.0, flat(), flat(), s);
        for tt4 in [1500.0, 1000.0, 700.0] {
            let a = m.match_point(&flight(), tt4).base;
            let z = r42.match_point(&flight(), tt4).base;
            same_core(&format!("s={s} Tt4={tt4}"), &a, &z);
            same(&format!("s={s} Tt4={tt4} phi_hp"), a.phi_hp, z.phi_hp);
            same(&format!("s={s} Tt4={tt4} phi_lp"), a.phi_lp, (1.0 - s) * z.phi_lp);
        }
    }
}

/// An HP plus an LP share `s` IS rung 90's customer bleed `c = s` less the dumped air: both
/// faces by `(1−s)`.
#[test]
fn identity_both_shares_are_rung90s_customer_bleed_less_the_air() {
    for s in [0.04, 0.12] {
        let m = matcher(None, None, Some(ShaftOfftake::both(Demand::share(s), Demand::share(s))),
                        flat(), flat(), 0.0);
        let r90 = CustomerBleedMatcher::new(with(Some(CustomerBleed::share(s)), None, None), flight(),
                                            1.0, flat(), flat(), 0.0);
        for tt4 in [1500.0, 1000.0, 700.0] {
            let a = m.match_point(&flight(), tt4).base;
            let z = r90.match_point(&flight(), tt4).base;
            same_core(&format!("s={s} Tt4={tt4}"), &a, &z);
            same(&format!("s={s} Tt4={tt4} phi_hp"), a.phi_hp, (1.0 - s) * z.phi_hp);
            same(&format!("s={s} Tt4={tt4} phi_lp"), a.phi_lp, (1.0 - s) * z.phi_lp);
        }
    }
}

/// The load moves no gas: on the perfect gas, with no cooling, both turbines stay at their pins.
#[test]
fn the_turbines_do_not_move() {
    let m0 = matcher(None, None, None, flat(), flat(), 0.0);
    for so in [HP_SHARE, HP_POWER, LP_SHARE, LP_POWER] {
        let m = matcher(None, None, Some(so), flat(), flat(), 0.0);
        for tt4 in [1500.0, 1000.0] {
            let (a, z) = (m.match_point(&flight(), tt4).base.base, m0.match_point(&flight(), tt4).base.base);
            for (k, x, y) in [("tau_hpt", a.tau_hpt, z.tau_hpt), ("pi_hpt", a.pi_hpt, z.pi_hpt),
                              ("tau_lpt", a.tau_lpt, z.tau_lpt), ("pi_lpt", a.pi_lpt, z.pi_lpt)] {
                assert!((x / y - 1.0).abs() < 1e-11, "{so:?} Tt4={tt4} {k}: {x} vs {y}");
            }
        }
    }
}

// =============================================================================================
// THE PLANT GATE — the transient's equilibrium IS the steady match, over every lower path
// =============================================================================================

fn plant_gate(cb: Option<CustomerBleed>, tc: Option<TurbineCooling>, so: ShaftOfftake, b: f64, tt4: f64) {
    let arm = if b == 0.0 { LeverArm::default() } else { LeverArm::constant(b) };
    let eq = machine(cb, tc, Some(so), &arm).fuel.inner.equilibrium(&flight(), tt4);
    assert!(eq.close.offtake == Some(so), "the closure carries the load's mark");
    let od = matcher(cb, tc, Some(so), lp(), hp(), b).match_point(&flight(), tt4);
    let k = od.booking.expect("active");
    let face = eq.close.mdot_face.unwrap_or(eq.close.mdot_air);
    for (name, got, want) in [("n_lp", eq.close.n_lp, od.base.n_lp),
                              ("n_hp", eq.close.n_hp, od.base.n_hp),
                              ("phi_lp", eq.close.phi_lp, od.base.phi_lp),
                              ("phi_hp", eq.close.phi_hp, od.base.phi_hp),
                              ("pi_lpc", eq.close.pi_lpc, od.base.base.pi_lpc),
                              ("pi_hpc", eq.close.pi_hpc, od.base.base.pi_hpc),
                              ("pi_hpt", eq.pi_hpt, od.base.base.pi_hpt),
                              ("tau_hpt", eq.tau_hpt, od.base.base.tau_hpt),
                              ("Tt3", eq.close.tt3, od.base.base.station("3").tt),
                              ("burner", eq.close.mdot_air, k.mdot_burner),
                              ("face", face, od.base.base.mdot_air)] {
        assert!((got / want - 1.0).abs() < 1e-9,
                "{cb:?} {tc:?} {so:?} b={b} Tt4={tt4} {name}: transient {got:?} vs steady {want:?}");
    }
    let m = machine(cb, tc, Some(so), &arm);
    let (sh, sl) = shaft_offtake::shares(&m.fuel.inner, &eq);
    assert!((sh - k.share_hp).abs() < 1e-9 && (sl - k.share_lp).abs() < 1e-9,
            "{cb:?} {tc:?} {so:?} b={b} Tt4={tt4} shares: transient ({sh}, {sl}) vs steady ({}, {})",
            k.share_hp, k.share_lp);
    let thrust = eq.sp_thrust_inlet.map_or(eq.sp_thrust * face, |s| s * face);
    assert!((thrust / od.base.base.thrust - 1.0).abs() < 1e-9,
            "{cb:?} {tc:?} {so:?} b={b} Tt4={tt4} thrust: {thrust} vs {}", od.base.base.thrust);
}

/// HP and LP shaft × share and power × every lower path (rung 57's, rung 62's bled, rung 90's,
/// rung 91's) — each at a high and a low throttle.
#[test]
fn plant_gate_every_shaft_mode_and_lower_path() {
    for so in [HP_SHARE, HP_POWER, LP_SHARE, LP_POWER] {
        for (cb, tc, b) in [(None, None, 0.0), (None, None, 0.05),
                            (Some(CustomerBleed::share(0.05)), None, 0.0),
                            (Some(CustomerBleed::amount(0.05)), None, 0.05),
                            (None, Some(VANE), 0.0)] {
            for tt4 in [1400.0, 1000.0] {
                plant_gate(cb, tc, so, b, tt4);
            }
        }
    }
}

// =============================================================================================
// THE SHAFT BALANCES CLOSE WITH THE LOAD BOOKED; THE POWER MODE DELIVERS ITS POWER
// =============================================================================================

#[test]
fn the_loaded_point_balances_both_shafts() {
    let g = cpg();
    let de = design();
    let p_ref = shaft_offtake::p_ref(&TwoSpoolMapMatcher::new(de, flight(), 1.0, flat(), flat()).core().base);
    for (cb, tc, so, b) in [(None, None, HP_POWER, 0.0), (None, None, LP_SHARE, 0.0),
                            (Some(CustomerBleed::share(0.04)), Some(VANE),
                             ShaftOfftake::both(Demand::power(0.02), Demand::share(0.05)), 0.05)] {
        let m = matcher(cb, tc, Some(so), lp(), hp(), b);
        for tt4 in [1500.0, 1100.0] {
            let od = m.match_point(&flight(), tt4);
            let k = od.booking.unwrap();
            let s = &od.base.base;
            let st = |l: &str| *s.station(l);
            let (s2, s25, s3, s41, s44, s45, s5) = (st("2"), st("25"), st("3"), st("41"), st("44"),
                                                     st("45"), st("5"));
            let hp_c = s3.mdot * (g.h_c(s3.tt) - g.h_c(s25.tt));
            let hp_t = 0.99 * s41.mdot * (g.h_t(s41.tt, s41.far) - g.h_t(s44.tt, s44.far));
            assert!(((hp_t - k.power_hp) / hp_c - 1.0).abs() < 1e-9, "{so:?} Tt4={tt4} HP balance");
            assert!((hp_t / k.turbine_hp - 1.0).abs() < 1e-9, "{so:?} Tt4={tt4} HP turbine: rebuild vs cascade");
            let lp_c = s2.mdot * (g.h_c(s25.tt) - g.h_c(s2.tt));
            let lp_t = 0.99 * s45.mdot * (g.h_t(s45.tt, s45.far) - g.h_t(s5.tt, s45.far));
            assert!(((lp_t - k.power_lp) / lp_c - 1.0).abs() < 1e-9, "{so:?} Tt4={tt4} LP balance");
            assert!((lp_t / k.turbine_lp - 1.0).abs() < 1e-9, "{so:?} Tt4={tt4} LP turbine: rebuild vs cascade");
            assert!((s.tau_hpt - s44.tt / s41.tt).abs() < 1e-9 * s.tau_hpt, "rotor tau: rebuild vs pin");
            for (d, p, sh) in [(so.hp, k.power_hp, k.share_hp), (so.lp, k.power_lp, k.share_lp)] {
                match d {
                    Some(Demand::Power(w)) => assert!((p / (w * p_ref) - 1.0).abs() < 1e-9,
                                                      "{so:?} Tt4={tt4}: the power mode delivers w*P_ref"),
                    Some(Demand::Share(x)) => assert!((sh - x).abs() < 1e-12, "{so:?}: the share is the share"),
                    None => assert!(p == 0.0 && sh == 0.0),
                }
            }
        }
    }
}

// =============================================================================================
// REFUSALS — an explicit demand is never silently dropped
// =============================================================================================

#[test]
#[should_panic(expected = "rung-92")]
fn rung91_refuses_an_active_load() {
    TurbineCoolingMatcher::new(with(None, Some(VANE), Some(HP_SHARE)), flight(), 1.0, lp(), hp(), 0.0)
        .match_point(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "rung-92")]
fn rung90_refuses_an_active_load() {
    CustomerBleedMatcher::new(with(Some(CustomerBleed::share(0.05)), None, Some(LP_POWER)), flight(),
                              1.0, lp(), hp(), 0.0).match_point(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "rung-92")]
fn rung42_refuses_an_active_load() {
    TwoSpoolBleedMatcher::new(with(None, None, Some(HP_POWER)), flight(), 1.0, lp(), hp(), 0.05)
        .match_point(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "rung-92")]
fn rung39_refuses_an_active_load() {
    TwoSpoolMapMatcher::new(with(None, None, Some(LP_SHARE)), flight(), 1.0, lp(), hp())
        .match_point(&flight(), 1200.0);
}

/// A machine below rung 62 (rung 57's) never marks its closures, so rung 40's power cell refuses.
#[test]
#[should_panic(expected = "rung-92")]
fn rung57_refuses_an_active_load() {
    let c = match ScheduledStatorTransient::new(with(None, None, Some(HP_SHARE)), flight(), 1.0,
                                                Some(lp()), Some(hp()), 1.0, StatorArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    c.fuel.inner.equilibrium(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "perfect (constant-property) gas only")]
fn a_thermally_perfect_gas_is_refused() {
    build_two_spool_turbojet(Gas::thermally_perfect(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
        .with_shaft_offtake(HP_SHARE);
}

#[test]
#[should_panic(expected = "rung-92 offtake share")]
fn a_share_of_half_is_refused() {
    Demand::share(0.5);
}

#[test]
fn a_power_past_half_its_turbine_is_refused_at_the_root_in_words() {
    // The design HP compressor power is 2.61x the LP's. Measured at 1500 K: 0.16 of it lands at a
    // root taking 57 % of the LP turbine's work (past the bound); 0.25 runs away — less work, less
    // air, a larger share — past the fold where no steady point exists.
    let m = matcher(None, None, Some(ShaftOfftake::lp(Demand::power(0.16))), flat(), flat(), 0.0);
    let e = m.try_match_point(&flight(), 1500.0).err().expect("must be refused");
    assert!(e.0.contains("half or more"), "{}", e.0);
    let m = matcher(None, None, Some(ShaftOfftake::lp(Demand::power(0.25))), flat(), flat(), 0.0);
    let e = m.try_match_point(&flight(), 1500.0).err().expect("must be refused");
    assert!(e.0.contains("no steady point"), "{}", e.0);
}

// =============================================================================================
// SIBLINGS KEEP IT, AND THE DISPATCH IS LIVE
// =============================================================================================

#[test]
fn every_sibling_keeps_the_load() {
    let so = ShaftOfftake::both(Demand::power(0.02), Demand::share(0.05));
    let m = machine(None, None, Some(so), &LeverArm::floored(BleedLimiter::new(1.38 * 0.55, 0.10)));
    for sib in [m.at_lever(&LeverArm::default()), m.at_lever(&LeverArm::constant(0.03))] {
        assert!(sib.fuel.inner.inner.base.offtake == Some(so));
    }
    let s62 = match build_scheduled_bleed(with(None, None, Some(so)), flight(), 1.0, Some(lp()), Some(hp()),
                                          1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    assert!(s62.at_lever(&LeverArm::constant(0.02)).fuel.inner.inner.base.offtake == Some(so));
}

#[test]
fn the_dispatch_is_live_on_both_sides() {
    shaft_offtake::reset_counts();
    let m = machine(Some(CustomerBleed::share(0.05)), None, None, &LeverArm::default());
    m.fuel.inner.equilibrium(&flight(), 1200.0);
    assert_eq!(shaft_offtake::counts()[2..], [0, 0, 0, 0], "no load must never reach rung 92");
    let m = machine(None, None, Some(HP_POWER), &LeverArm::default());
    m.fuel.inner.equilibrium(&flight(), 1200.0);
    let mf = m.fuel.fuel_for_tt4(&flight(), 1200.0);
    m.fuel.instant_fuel(&flight(), 0.85, 0.88, mf);
    let c = shaft_offtake::counts();
    assert!(c[2] > 0 && c[3] > 0 && c[4] > 0 && c[5] > 0, "every rung-92 cell must run: {c:?}");
    shaft_offtake::reset_counts();
    matcher(None, None, None, flat(), flat(), 0.0).match_point(&flight(), 1200.0);
    matcher(None, None, Some(LP_SHARE), flat(), flat(), 0.0).match_point(&flight(), 1200.0);
    let c = shaft_offtake::counts();
    assert!(c[0] == 1 && c[1] == 1, "steady: one active, one inactive: {c:?}");
}

// =============================================================================================
// THE SCORED READINGS (anchor § 3, scored in docs/rung92-spec.md § 3) — one gate per claim, on a
// subset of the scored grid. Bars are the ANCHOR's (1773b7d).
// =============================================================================================

/// The anchor's linear model on the unloaded point, per unit SHARE:
/// `((HP shaft: d ln phi_L, d ln phi_H), (LP shaft: …))`.
fn model(r0: &TwoSpoolMapResult) -> ((f64, f64), (f64, f64)) {
    let s = &r0.base;
    let (tt2, tt25, tt3) = (s.station("2").tt, s.station("25").tt, s.station("3").tt);
    let a = |tau: f64, eta: f64| 3.5 * eta * (tau - 1.0) / (1.0 + eta * (tau - 1.0));
    let (tl, th) = (tt25 / tt2, tt3 / tt25);
    let (al, ah, t_l) = (a(tl, r0.eta_lpc), a(th, r0.eta_hpc), (tl - 1.0) / tl);
    ((-ah, 0.5 - ah), (0.5 - al + ah * t_l, t_l * (ah - 1.0)))
}

/// `(d ln phi_L, d ln phi_H)` per unit load on one shaft, Richardson-extrapolated from 0.01 and 0.02
/// (anchor § 2), and the unloaded point.
fn sens(hp: bool, power: bool, ml: ComponentMap, mh: ComponentMap, tt4: f64) -> (f64, f64, TwoSpoolMapResult) {
    let at = |x: f64| {
        let d = if power { Demand::power(x) } else { Demand::share(x) };
        let so = (x > 0.0).then(|| if hp { ShaftOfftake::hp(d) } else { ShaftOfftake::lp(d) });
        matcher(None, None, so, ml, mh, 0.0).match_point(&flight(), tt4).base
    };
    let (r0, r1, r2) = (at(0.0), at(0.01), at(0.02));
    let rich = |x0: f64, x1: f64, x2: f64| 2.0 * (x1 / x0).ln() / 0.01 - (x2 / x0).ln() / 0.02;
    (rich(r0.phi_lp, r1.phi_lp, r2.phi_lp), rich(r0.phi_hp, r1.phi_hp, r2.phi_hp), r0)
}

fn shaped() -> (ComponentMap, ComponentMap) {
    (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() },
     ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() })
}

/// A1 + A2 + A3: the model holds on the flat maps (bar 0.01; measured ≤ 0.0030 HP shaft, ≤ 0.0011 LP
/// shaft), and the HP-shaft load pushes both spools toward surge.
#[test]
fn a1_a2_a3_the_linear_model_predicts_both_shafts() {
    for tt4 in [1500.0, 1200.0, 1000.0, 800.0, 700.0] {
        let (hl, hh, r0) = sens(true, false, flat(), flat(), tt4);
        let (ll, lh, _) = sens(false, false, flat(), flat(), tt4);
        let ((mhl, mhh), (mll, mlh)) = model(&r0);
        assert!((hl - mhl).abs() < 0.01 && (hh - mhh).abs() < 0.01, "A1 Tt4={tt4}: HP shaft ({hl}, {hh}) vs ({mhl}, {mhh})");
        assert!(hl < 0.0 && hh < 0.0, "A2 Tt4={tt4}");
        assert!((ll - mll).abs() < 0.01 && (lh - mlh).abs() < 0.01, "A3 Tt4={tt4}: LP shaft ({ll}, {lh}) vs ({mll}, {mlh})");
    }
}

/// A4 + A5: the LP-shaft load's two crossings — HP at 783 ± 75 K (measured 789), LP at 1290 ± 55 K
/// (measured 1295), each once, in the anchor's directions.
#[test]
fn a4_a5_the_lp_shaft_crossings() {
    let grid = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 850.0, 800.0, 750.0, 700.0];
    let pts: Vec<(f64, f64, f64)> = grid.iter()
        .map(|&t| { let (l, h, _) = sens(false, false, flat(), flat(), t); (t, l, h) }).collect();
    let cross = |pick: &dyn Fn(&(f64, f64, f64)) -> f64| -> Vec<f64> {
        pts.windows(2).filter_map(|w| {
            let (y0, y1) = (pick(&w[0]), pick(&w[1]));
            ((y0 < 0.0) != (y1 < 0.0)).then(|| w[0].0 + (w[1].0 - w[0].0) * y0 / (y0 - y1))
        }).collect()
    };
    let (h, l) = (cross(&|p| p.2), cross(&|p| p.1));
    assert!(h.len() == 1 && (h[0] - 783.0).abs() < 75.0 && pts[0].2 > 0.0, "A4 {h:?}");
    assert!(l.len() == 1 && (l[0] - 1290.0).abs() < 55.0 && pts[0].1 < 0.0, "A5 {l:?}");
}

/// A6: per unit POWER, the HP-shaft load lowers both flow coefficients more than the LP-shaft load
/// (flat and flow/press; at 650 K the 0.02 reading unchokes the nozzle — not scorable as written).
#[test]
fn a6_the_same_power_costs_more_on_the_hp_shaft() {
    let (l, h) = shaped();
    for (ml, mh) in [(flat(), flat()), (l, h)] {
        for tt4 in [1500.0, 1100.0, 700.0] {
            let (hl, hh, _) = sens(true, true, ml, mh, tt4);
            let (ll, lh, _) = sens(false, true, ml, mh, tt4);
            assert!(hl < ll && hh < lh, "A6 Tt4={tt4}: HP ({hl}, {hh}) vs LP ({ll}, {lh})");
        }
    }
}

/// A7 (flow/press, sign only).
#[test]
fn a7_the_signs_survive_the_shaped_maps() {
    let (l, h) = shaped();
    let (hl, hh, _) = sens(true, false, l, h, 1500.0);
    let (ll, lh, _) = sens(false, false, l, h, 1500.0);
    assert!(hl < 0.0 && hh < 0.0 && lh > 0.0 && ll < 0.0, "A7 1500 K: HP ({hl}, {hh}) LP ({ll}, {lh})");
    let (hl, hh, _) = sens(true, false, l, h, 650.0);
    let (ll, lh, _) = sens(false, false, l, h, 650.0);
    assert!(hl < 0.0 && hh < 0.0 && lh < 0.0 && ll > 0.0, "A7 650 K: HP ({hl}, {hh}) LP ({ll}, {lh})");
}

/// One cell of rig B: `(min phi_LP, min phi_HP, ceiling, valve bill at sm 0.34, HP t90, points)`.
fn slam(so: Option<ShaftOfftake>, r: f64) -> (f64, f64, f64, f64, f64, usize) {
    use turbojet::stator_transient::Ramp;
    let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds: 0.005 };
    let m = machine(None, None, so, &LeverArm::default());
    let shut = m.bill_cell(&flight(), &rp, true);
    let tr = shut.traj.as_ref().unwrap();
    let (n0, n1) = (tr[0].nu_hp, tr[tr.len() - 1].nu_hp);
    let target = n0 + 0.9 * (n1 - n0);
    let t90 = tr.windows(2).find(|w| w[0].nu_hp < target && w[1].nu_hp >= target)
        .map(|w| w[0].s + (w[1].s - w[0].s) * (target - w[0].nu_hp) / (w[1].nu_hp - w[0].nu_hp)).unwrap();
    let full = m.at_lever(&LeverArm::constant(0.10)).bill_cell(&flight(), &rp, false);
    // sm 0.34: the valve rides with the HP load (measured peak 0.0745), barely without (0.0024).
    let v = m.at_lever(&LeverArm::floored(BleedLimiter::new(1.34 * 0.55, 0.10))).bill_cell(&flight(), &rp, false);
    (shut.min_phi_lp, shut.min_phi_hp, full.min_phi_lp, v.b_int, t90, v.npts)
}

/// B1, B3, B4, B5 on one cell of rig B (shaped maps, r = 0.5): the HP-shaft load lowers both minima,
/// makes the valve bleed more and its ceiling fall, costs the valve more than the LP-shaft load of the
/// same power, and delays the HP spool's 90 % rise.
#[test]
fn b1_b3_b4_b5_the_hp_generator_costs_the_valve_and_slows_the_spool() {
    let n = slam(None, 0.5);
    let h = slam(Some(ShaftOfftake::hp(Demand::power(0.02))), 0.5);
    let l = slam(Some(ShaftOfftake::lp(Demand::power(0.02))), 0.5);
    assert!(n.5 == h.5 && h.5 == l.5, "bills compared only at equal run length");
    assert!(h.0 < n.0 && h.1 < n.1, "B1 {h:?} vs {n:?}");
    assert!(h.3 > n.3 && h.2 < n.2, "B3 {h:?} vs {n:?}");
    assert!(h.3 > l.3, "B4 HP {h:?} vs LP {l:?}");
    assert!(h.4 > n.4, "B5 t90 {} vs {}", h.4, n.4);
}

/// B2 — the MISS, pinned as measured: the LP-shaft load's HP relief is a steady property that a fast
/// slam reverses (shaped maps: +0.0015 at r = 1, −0.0004 at r = 0.25).
#[test]
fn b2_miss_a_fast_slam_reverses_the_lp_generators_hp_relief() {
    let lp = Some(ShaftOfftake::lp(Demand::power(0.02)));
    let (slow_n, slow_l) = (slam(None, 1.0), slam(lp, 1.0));
    let (fast_n, fast_l) = (slam(None, 0.25), slam(lp, 0.25));
    assert!(slow_l.1 > slow_n.1, "slow slam: the LP load raises min phi_HP ({} vs {})", slow_l.1, slow_n.1);
    assert!(fast_l.1 < fast_n.1, "fast slam: the LP load lowers min phi_HP ({} vs {})", fast_l.1, fast_n.1);
}
