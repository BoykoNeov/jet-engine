//! RUNG 90 — THE CUSTOMER BLEED. Anchor: `docs/plans/rung90-anchor-customer-bleed.md`; spec:
//! `docs/rung90-spec.md`.
//!
//! This file holds the CONSTRUCTION gates (anchor § 3, "no credit"): the reduce, the plant gate
//! (transient equilibrium ≡ steady match), the mass identities, the refusals, the sibling carry,
//! and the dispatch. The scored readings are the spec's tables; the gates that pin them are below
//! the construction block.

use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::bleed_transient::{build_scheduled_bleed, LeverArm};
use turbojet::customer_bleed::{self, CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorArm};
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

fn with(cb: Option<CustomerBleed>) -> TwoSpoolEngine {
    match cb { Some(c) => design().with_customer_bleed(c), None => design() }
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(0.55)
}
fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(0.55)
}

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

fn machine(cb: Option<CustomerBleed>, arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(with(cb), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

const INACTIVE: [Option<CustomerBleed>; 3] =
    [None, Some(CustomerBleed::Share(0.0)), Some(CustomerBleed::Amount(0.0))];

// =============================================================================================
// THE REDUCE — inactive ⇒ the earlier code, bit for bit, by dispatch
// =============================================================================================

#[test]
fn reduce_steady_inactive_is_rung42_bit_for_bit() {
    for b in [0.0, 0.05] {
        let r42 = TwoSpoolBleedMatcher::new(design(), flight(), 1.0, lp(), hp(), b);
        for cb in INACTIVE {
            let m = CustomerBleedMatcher::new(with(cb), flight(), 1.0, lp(), hp(), b);
            for tt4 in [1500.0, 1200.0, 1000.0] {
                let want = r42.match_point(&flight(), tt4).base;
                let got = m.match_point(&flight(), tt4);
                assert!(got.booking.is_none(), "an inactive customer bleed books nothing");
                for ((k, a), (_, w)) in steady_keys(&got.base).into_iter().zip(steady_keys(&want)) {
                    assert!(a.to_bits() == w.to_bits(), "b={b} {cb:?} Tt4={tt4} {k}: {a:?} != {w:?}");
                }
            }
        }
    }
}

#[test]
fn reduce_transient_inactive_is_rung62_bit_for_bit() {
    for arm in [LeverArm::default(), LeverArm::constant(0.05),
                LeverArm::floored(BleedLimiter::new(1.38 * 0.55, 0.10))] {
        let want = machine(None, &arm);
        for cb in INACTIVE {
            let got = machine(cb, &arm);
            for tt4 in [1400.0, 1000.0] {
                let (a, w) = (got.fuel.inner.equilibrium(&flight(), tt4),
                              want.fuel.inner.equilibrium(&flight(), tt4));
                for ((k, x), (_, y)) in inst_keys(&a).into_iter().zip(inst_keys(&w)) {
                    assert!(x.to_bits() == y.to_bits(), "{cb:?} Tt4={tt4} {k}: {x:?} != {y:?}");
                }
                assert!(a.close.customer.is_none(), "an inactive customer bleed reaches rung 62's dict");
            }
            let mf = want.fuel.fuel_for_tt4(&flight(), 1200.0);
            assert!(mf.to_bits() == got.fuel.fuel_for_tt4(&flight(), 1200.0).to_bits());
            let (a, w) = (got.fuel.instant_fuel(&flight(), 0.85, 0.88, mf),
                          want.fuel.instant_fuel(&flight(), 0.85, 0.88, mf));
            for ((k, x), (_, y)) in inst_keys(&a.base).into_iter().zip(inst_keys(&w.base)) {
                assert!(x.to_bits() == y.to_bits(), "fuel path {cb:?} {k}: {x:?} != {y:?}");
            }
        }
    }
}

#[test]
fn the_design_run_ignores_the_customer_bleed() {
    let (a, b) = (design().run(&flight(), 1.0),
                  with(Some(CustomerBleed::share(0.1))).run(&flight(), 1.0));
    for s in ["3", "4", "9"] {
        assert!(a.station(s).tt.to_bits() == b.station(s).tt.to_bits());
        assert!(a.station(s).mdot.to_bits() == b.station(s).mdot.to_bits());
    }
}

// =============================================================================================
// THE PLANT GATE — the transient's equilibrium IS the steady match (rung 62's lesson)
// =============================================================================================

fn plant_gate(cb: CustomerBleed, b: f64, tt4: f64) {
    let arm = if b == 0.0 { LeverArm::default() } else { LeverArm::constant(b) };
    let eq = machine(Some(cb), &arm).fuel.inner.equilibrium(&flight(), tt4);
    let od = CustomerBleedMatcher::new(with(Some(cb)), flight(), 1.0, lp(), hp(), b)
        .match_point(&flight(), tt4);
    let k = od.booking.expect("active");
    for (name, got, want) in [("n_lp", eq.close.n_lp, od.base.n_lp),
                              ("n_hp", eq.close.n_hp, od.base.n_hp),
                              ("phi_lp", eq.close.phi_lp, od.base.phi_lp),
                              ("phi_hp", eq.close.phi_hp, od.base.phi_hp),
                              ("pi_lpc", eq.close.pi_lpc, od.base.base.pi_lpc),
                              ("pi_hpc", eq.close.pi_hpc, od.base.base.pi_hpc),
                              ("share", eq.close.customer.unwrap(), k.customer),
                              ("burner", eq.close.mdot_air, k.mdot_burner),
                              ("face", eq.close.mdot_face.unwrap(), od.base.base.mdot_air)] {
        assert!((got / want - 1.0).abs() < 1e-9,
                "{cb:?} b={b} Tt4={tt4} {name}: transient {got:?} vs steady {want:?}");
    }
    let thrust = eq.sp_thrust_inlet.unwrap() * eq.close.mdot_face.unwrap();
    assert!((thrust / od.base.base.thrust - 1.0).abs() < 1e-9,
            "{cb:?} b={b} Tt4={tt4} thrust: {thrust} vs {}", od.base.base.thrust);
}

#[test] fn plant_gate_share_1400() { plant_gate(CustomerBleed::share(0.05), 0.0, 1400.0); }
#[test] fn plant_gate_share_1000() { plant_gate(CustomerBleed::share(0.05), 0.0, 1000.0); }
#[test] fn plant_gate_share_with_valve() { plant_gate(CustomerBleed::share(0.05), 0.05, 1200.0); }
#[test] fn plant_gate_amount_1400() { plant_gate(CustomerBleed::amount(0.05), 0.0, 1400.0); }
#[test] fn plant_gate_amount_1000() { plant_gate(CustomerBleed::amount(0.05), 0.0, 1000.0); }
#[test] fn plant_gate_amount_with_valve() { plant_gate(CustomerBleed::amount(0.05), 0.05, 1200.0); }

// =============================================================================================
// MASS, AND WHAT THE AIRCRAFT GETS
// =============================================================================================

#[test]
fn mass_identities_and_the_amount_mode_delivers_its_amount() {
    for (cb, b) in [(CustomerBleed::share(0.05), 0.0), (CustomerBleed::share(0.08), 0.05),
                    (CustomerBleed::amount(0.05), 0.0), (CustomerBleed::amount(0.05), 0.05)] {
        let m = CustomerBleedMatcher::new(with(Some(cb)), flight(), 1.0, lp(), hp(), b);
        for tt4 in [1500.0, 1100.0] {
            let od = m.match_point(&flight(), tt4);
            let k = od.booking.unwrap();
            let s = &od.base.base;
            let face = s.mdot_air;
            assert!((k.mdot_burner / ((1.0 - b) * (1.0 - k.customer) * face) - 1.0).abs() < 1e-12);
            let s4 = s.station("4");
            assert!((s4.mdot / ((1.0 + s4.far) * k.mdot_burner) - 1.0).abs() < 1e-12,
                    "the burner sees the air after BOTH extractions");
            assert!((s.station("3").mdot / k.mdot_hpc - 1.0).abs() < 1e-12,
                    "station 3 is the HPC delivery, before the customer takes its share");
            assert!((k.tt3 - s.station("3").tt).abs() == 0.0);
            if let CustomerBleed::Amount(w) = cb {
                assert!((k.mdot_customer / w - 1.0).abs() < 1e-9,
                        "the amount mode must deliver its amount: {} vs {w}", k.mdot_customer);
            }
        }
    }
}

// =============================================================================================
// REFUSALS — an explicit demand is never silently dropped
// =============================================================================================

#[test]
#[should_panic(expected = "rung-90")]
fn rung39_refuses_an_active_customer_bleed() {
    TwoSpoolMapMatcher::new(with(Some(CustomerBleed::share(0.05))), flight(), 1.0, lp(), hp())
        .match_point(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "rung-90")]
fn rung42_refuses_an_active_customer_bleed() {
    TwoSpoolBleedMatcher::new(with(Some(CustomerBleed::share(0.05))), flight(), 1.0, lp(), hp(),
                              0.05).match_point(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "rung-90")]
fn rung57_refuses_an_active_customer_bleed() {
    let c = match ScheduledStatorTransient::new(with(Some(CustomerBleed::share(0.05))), flight(),
                                                1.0, Some(lp()), Some(hp()), 1.0,
                                                StatorArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    c.fuel.inner.equilibrium(&flight(), 1200.0);
}

#[test]
#[should_panic(expected = "rung-90 customer share")]
fn a_share_past_half_is_refused() {
    CustomerBleed::share(0.5);
}

// =============================================================================================
// SIBLINGS KEEP IT, AND THE DISPATCH IS LIVE
// =============================================================================================

#[test]
fn every_sibling_keeps_the_customer_bleed() {
    let cb = CustomerBleed::share(0.05);
    let m = machine(Some(cb), &LeverArm::floored(BleedLimiter::new(1.38 * 0.55, 0.10)));
    for sib in [m.at_lever(&LeverArm::default()), m.at_lever(&LeverArm::constant(0.03))] {
        assert!(sib.fuel.inner.inner.base.customer == Some(cb));
    }
    let s62 = match build_scheduled_bleed(with(Some(cb)), flight(), 1.0, Some(lp()), Some(hp()),
                                          1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    };
    assert!(s62.at_lever(&LeverArm::constant(0.02)).fuel.inner.inner.base.customer == Some(cb));
}

#[test]
fn the_dispatch_is_live_on_both_sides() {
    customer_bleed::reset_counts();
    let m = machine(None, &LeverArm::default());
    m.fuel.inner.equilibrium(&flight(), 1200.0);
    assert_eq!(customer_bleed::counts()[2..], [0, 0, 0, 0], "inactive must never reach rung 90");
    let m = machine(Some(CustomerBleed::share(0.05)), &LeverArm::default());
    m.fuel.inner.equilibrium(&flight(), 1200.0);
    let mf = m.fuel.fuel_for_tt4(&flight(), 1200.0);
    m.fuel.instant_fuel(&flight(), 0.85, 0.88, mf);
    let c = customer_bleed::counts();
    assert!(c[2] > 0 && c[3] > 0 && c[4] > 0 && c[5] > 0, "every rung-90 cell must run: {c:?}");
}

// =============================================================================================
// THE SCORED READINGS (anchor § 3, scored in docs/rung90-spec.md § 3) — one gate per claim, on
// a subset of the scored grid. Bars are the ANCHOR's (504d428), not the measured values, except
// where a gate says it was narrowed after measuring.
// =============================================================================================

fn flat() -> ComponentMap { ComponentMap::flat() }

/// The anchor's linear model, on the customer-free point.
fn model(r0: &TwoSpoolMapResult) -> (f64, f64) {
    let s = &r0.base;
    let a = |tau: f64, eta: f64| 3.5 * eta * (tau - 1.0) / (1.0 + eta * (tau - 1.0));
    let (tl, th) = (s.station("25").tt / s.station("2").tt, s.station("3").tt / s.station("25").tt);
    let (al, ah, t_l) = (a(tl, r0.eta_lpc), a(th, r0.eta_hpc), (tl - 1.0) / tl);
    (1.5 - al - ah * (1.0 - t_l), 0.5 + (1.0 - t_l) * (1.0 - ah))
}

/// `(d ln phi_L/dc, d ln phi_H/dc)` one-sided at c = 0.02, and the c = 0 point.
fn sens(ml: ComponentMap, mh: ComponentMap, tt4: f64) -> (f64, f64, TwoSpoolMapResult) {
    let r0 = CustomerBleedMatcher::new(design(), flight(), 1.0, ml, mh, 0.0).match_point(&flight(), tt4).base;
    let r2 = CustomerBleedMatcher::new(with(Some(CustomerBleed::share(0.02))), flight(), 1.0, ml, mh, 0.0)
        .match_point(&flight(), tt4).base;
    ((r2.phi_lp / r0.phi_lp).ln() / 0.02, (r2.phi_hp / r0.phi_hp).ln() / 0.02, r0)
}

/// A1 + A3: the model holds on the flat maps (bars 0.03 HP, 0.05 LP; measured ≤ 0.0093).
#[test]
fn a1_a3_the_linear_model_predicts_both_spools() {
    for tt4 in [1500.0, 1200.0, 1000.0, 800.0, 700.0] {
        let (dl, dh, r0) = sens(flat(), flat(), tt4);
        let (ml, mh) = model(&r0);
        assert!((dh - mh).abs() < 0.03, "A1 Tt4={tt4}: HP {dh} vs model {mh}");
        assert!((dl - ml).abs() < 0.05, "A3 Tt4={tt4}: LP {dl} vs model {ml}");
    }
}

/// A2 + A4 + A6: the HP is helped everywhere (no pi*); the LP changes sign ONCE, near 900 K.
#[test]
fn a2_a4_a6_the_hp_never_reverses_and_the_lp_crosses_near_900k() {
    let mut prev: Option<(f64, f64)> = None;
    let mut crossings = Vec::new();
    for tt4 in [1500.0, 1300.0, 1100.0, 1000.0, 900.0, 800.0, 650.0] {
        let (dl, dh, _) = sens(flat(), flat(), tt4);
        assert!(dh > 0.0, "A2 Tt4={tt4}: HP {dh}");
        if let Some((t0, l0)) = prev {
            if (l0 < 0.0) != (dl < 0.0) { crossings.push(t0 + (tt4 - t0) * l0 / (l0 - dl)); }
        }
        prev = Some((tt4, dl));
    }
    assert!(crossings.len() == 1 && (crossings[0] - 900.0).abs() < 50.0, "A4 crossings {crossings:?}");
    for (tt4, lp_neg) in [(1500.0, true), (650.0, false)] {
        let (dl, dh, _) = sens(lp(), hp(), tt4);
        assert!(dh > 0.0 && (dl < 0.0) == lp_neg, "A6 Tt4={tt4}: LP {dl} HP {dh}");
    }
}

/// A5 — scored a MISS as written (> 1 % at every Tt4; 0.55 % at 1500 K, 0.77 % at 1400 K). This
/// gate pins what was MEASURED, narrowed after measuring (no credit): the c = 0.05 HP point is
/// off the c = 0 line at the same x_H by more than 0.5 % — 35× rung 42's 0.016 % — and the
/// displacement grows as the throttle falls.
#[test]
fn a5_the_hp_gets_a_new_freedom_growing_as_the_throttle_falls() {
    let m0 = CustomerBleedMatcher::new(design(), flight(), 1.0, flat(), flat(), 0.0);
    let m5 = CustomerBleedMatcher::new(with(Some(CustomerBleed::share(0.05))), flight(), 1.0, flat(), flat(), 0.0);
    let xh = |t: f64| { let r = m0.match_point(&flight(), t); (t / r.base.base.station("25").tt, r.base.phi_hp) };
    let mut offs = Vec::new();
    for tt4 in [1500.0, 1000.0, 700.0] {
        let r5 = m5.match_point(&flight(), tt4);
        let xt = tt4 / r5.base.base.station("25").tt;
        let (mut lo, mut hi) = (tt4, tt4 + 400.0);
        for _ in 0..60 { let mid = 0.5 * (lo + hi); if xh(mid).0 < xt { lo = mid } else { hi = mid } }
        offs.push(r5.base.phi_hp / xh(0.5 * (lo + hi)).1 - 1.0);
    }
    assert!(offs[0] > 0.005 && offs[0] < offs[1] && offs[1] < offs[2], "A5 {offs:?}");
}

/// A7: customer air costs more thrust and more fuel per unit thrust than interstage air.
#[test]
fn a7_customer_air_costs_more_than_interstage_air() {
    let m0 = CustomerBleedMatcher::new(design(), flight(), 1.0, flat(), flat(), 0.0);
    let m5 = CustomerBleedMatcher::new(with(Some(CustomerBleed::share(0.05))), flight(), 1.0, flat(), flat(), 0.0);
    let b5 = TwoSpoolBleedMatcher::new(design(), flight(), 1.0, flat(), flat(), 0.05);
    for tt4 in [1500.0, 1100.0, 800.0] {
        let p0 = m0.match_point(&flight(), tt4).base.base.performance;
        let (k, kb) = (m5.match_point(&flight(), tt4).booking.unwrap(),
                       b5.match_point(&flight(), tt4).booking.unwrap());
        assert!(k.st_inlet < kb.st_inlet && kb.st_inlet < p0.specific_thrust, "A7 thrust Tt4={tt4}");
        assert!(k.tsfc_inlet > kb.tsfc_inlet && kb.tsfc_inlet > p0.tsfc, "A7 tsfc Tt4={tt4}");
    }
}

/// A8: the amount mode's share rises as the throttle falls, and its LP shift at 700 K is positive
/// and larger than the share mode's. Its 650 K point is NOT scorable: the amount takes 12.8 % of
/// the air there and the nozzle unchokes — refused, out of scope (rung 42's envelope shrinks too).
#[test]
fn a8_the_amount_mode_bites_harder_at_low_power_and_shrinks_the_envelope() {
    let m0 = CustomerBleedMatcher::new(design(), flight(), 1.0, flat(), flat(), 0.0);
    let mw = CustomerBleedMatcher::new(with(Some(CustomerBleed::amount(0.05))), flight(), 1.0, flat(), flat(), 0.0);
    let m5 = CustomerBleedMatcher::new(with(Some(CustomerBleed::share(0.05))), flight(), 1.0, flat(), flat(), 0.0);
    let mut prev = 0.0;
    for tt4 in [1500.0, 1200.0, 900.0, 700.0] {
        let c = mw.match_point(&flight(), tt4).booking.unwrap().customer;
        assert!(c > prev, "A8 share must rise as the throttle falls: {c} at {tt4}");
        prev = c;
    }
    let p0 = m0.match_point(&flight(), 700.0).base.phi_lp;
    let (dw, dc) = ((mw.match_point(&flight(), 700.0).base.phi_lp / p0).ln(),
                    (m5.match_point(&flight(), 700.0).base.phi_lp / p0).ln());
    assert!(dw > dc && dc > 0.0, "A8 at 700 K: amount {dw} share {dc}");
    let e = mw.try_match_point(&flight(), 650.0).err().expect("650 K must be refused in the amount mode");
    assert!(e.0.contains("UNCHOKED"), "{}", e.0);
}

/// B1–B5 on one cell of rig B (shaped maps, r = 0.5, ds 0.005, rung 64's valve).
#[test]
fn b1_to_b5_the_protection_valve_works_around_the_customer_bleed() {
    use turbojet::stator_transient::Ramp;
    let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.005 };
    let read = |cb: Option<CustomerBleed>| {
        let m = machine(cb, &LeverArm::default());
        let shut = m.bill_cell(&flight(), &rp, false);
        let full = m.at_lever(&LeverArm::constant(0.10)).bill_cell(&flight(), &rp, false);
        let v = m.at_lever(&LeverArm::floored(BleedLimiter::new(1.38 * 0.55, 0.10)))
            .bill_cell(&flight(), &rp, false);
        assert!(v.b_int > 0.0 && v.b_peak < 0.10, "the valve must RIDE at sm 0.38");
        (shut.min_phi_lp, shut.min_phi_hp, full.min_phi_lp, v.b_int, v.npts)
    };
    let (n, s, a) = (read(None), read(Some(CustomerBleed::share(0.05))),
                     read(Some(CustomerBleed::amount(0.05))));
    assert!(s.0 < n.0, "B1 {s:?} vs {n:?}");
    assert!(s.1 > n.1 && (s.1 - n.1) > (n.0 - s.0), "B2 {s:?} vs {n:?}");
    assert!(n.4 == s.4 && s.4 == a.4, "bills compared only at equal run length");
    assert!(s.3 > n.3, "B3 {s:?} vs {n:?}");
    assert!(s.2 < n.2, "B4 {s:?} vs {n:?}");
    assert!(a.3 > s.3, "B5 {a:?} vs {s:?}");
}
