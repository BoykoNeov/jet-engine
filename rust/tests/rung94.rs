//! RUNG 94 — A BURNER THAT CARRIES THE LOAD. Anchor: `docs/plans/rung94-anchor-burner-hold.md`; spec:
//! `docs/rung94-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 3, "no credit") first: the reduce, the target met, the zero-load identities, the
//! search's brackets (G5), the refusals, the metal cap. Then the scored claims H1–H4, pinned at a subset of the
//! spec's grid (the full grid is `docs/plans/scratch-archive/rung94/score.rs`).

use turbojet::burner_hold::{self as bh, Hold, HoldRefusal};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

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
fn flat() -> ComponentMap { ComponentMap::flat() }
fn fp() -> (ComponentMap, ComponentMap) {
    (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() },
     ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() })
}
fn matcher(so: Option<ShaftOfftake>, ml: ComponentMap, mh: ComponentMap) -> ShaftOfftakeMatcher {
    let e = match so { Some(o) => design().with_shaft_offtake(o), None => design() };
    ShaftOfftakeMatcher::new(e, flight(), 1.0, ml, mh, 0.0)
}
fn hp(w: f64) -> ShaftOfftake { ShaftOfftake::hp(Demand::power(w)) }
fn lp(w: f64) -> ShaftOfftake { ShaftOfftake::lp(Demand::power(w)) }
const LO: f64 = 600.0;
const HI: f64 = 2200.0;

fn hold(so: ShaftOfftake, ml: ComponentMap, mh: ComponentMap, t0: f64, h: Hold) -> bh::HoldPoint {
    bh::try_hold_point(&matcher(Some(so), ml, mh), &matcher(None, ml, mh), &flight(), t0, h, LO, HI).unwrap()
}

// =============================================================================================
// CONSTRUCTION
// =============================================================================================

#[test]
fn holding_the_burner_temperature_or_with_no_load_is_rung92_exactly() {
    for (so, h) in [(Some(hp(0.03)), Hold::BurnerTemp), (None, Hold::Thrust), (None, Hold::HpSpeed),
                    (Some(ShaftOfftake::hp(Demand::power(0.0))), Hold::Thrust)] {
        let m = matcher(so, flat(), flat());
        let got = bh::try_hold_point(&m, &matcher(None, flat(), flat()), &flight(), 1200.0, h, LO, HI).unwrap();
        let want = m.match_point(&flight(), 1200.0);
        assert_eq!(got.tt4, 1200.0, "{so:?} {h:?}");
        assert!(got.point.base.base.thrust.to_bits() == want.base.base.thrust.to_bits());
        assert!(got.point.base.phi_lp.to_bits() == want.base.phi_lp.to_bits());
    }
}

#[test]
fn every_hold_meets_its_target() {
    for (so, h, t0) in [(hp(0.05), Hold::Thrust, 1500.0), (lp(0.05), Hold::Thrust, 1000.0), (hp(0.02), Hold::HpSpeed, 700.0),
                        (lp(0.05), Hold::HpSpeed, 1500.0)] {
        let p = hold(so, flat(), flat(), t0, h);
        let got = bh::held(&p.point, h);
        assert!((got / p.target - 1.0).abs() < 1e-9, "{so:?} {h:?} {t0}: {got} vs {}", p.target);
        let bare = bh::held(&matcher(None, flat(), flat()).match_point(&flight(), t0), h);
        assert!(p.target.to_bits() == bare.to_bits(), "the target is the unloaded engine's at the set throttle");
    }
}

#[test]
fn the_zero_load_identities_hold() {
    // dT/dw at vanishing load (Richardson 0.01/0.02) is -Q_w/Q_T; the finite-difference error grows toward the fold.
    let (ml, mh) = fp();
    for (t0, tol) in [(1500.0, 0.005), (1000.0, 0.02), (700.0, 0.06)] {
        for (h, mk) in [(Hold::Thrust, hp as fn(f64) -> ShaftOfftake), (Hold::HpSpeed, hp), (Hold::Thrust, lp), (Hold::HpSpeed, lp)] {
            let q = |m: &ShaftOfftakeMatcher, t: f64| {
                let x = bh::held(&m.match_point(&flight(), t), h);
                if h == Hold::Thrust { x.ln() } else { x }
            };
            let (m0, m1, m2) = (matcher(None, ml, mh), matcher(Some(mk(0.01)), ml, mh), matcher(Some(mk(0.02)), ml, mh));
            let qw = 2.0 * (q(&m1, t0) - q(&m0, t0)) / 0.01 - (q(&m2, t0) - q(&m0, t0)) / 0.02;
            let qt = (q(&m0, t0 + 10.0) - q(&m0, t0 - 10.0)) / 20.0;
            let ident = -qw / qt;
            let dev = 2.0 * (hold(mk(0.01), ml, mh, t0, h).tt4 - t0) / 0.01 - (hold(mk(0.02), ml, mh, t0, h).tt4 - t0) / 0.02;
            assert!((dev - ident).abs() <= tol * ident.abs().max(30.0), "{h:?} {t0}: device {dev} vs identity {ident}");
        }
    }
}

#[test]
fn the_search_reaches_g5s_brackets_and_refuses_in_words() {
    // Holding the 700 K thrust (flat): an HP 0.10 settles between its floor (974 K) and 1000 K ...
    let p = hold(hp(0.10), flat(), flat(), 700.0, Hold::Thrust);
    assert!(p.tt4 > 974.0 && p.tt4 <= 1000.0 && (p.t_floor - 974.0).abs() < 1.0, "{} (floor {})", p.tt4, p.t_floor);
    // ... and an LP 0.05 cannot run that low: refused, naming its least thrust (143 N at 913 K).
    let e = bh::try_hold_point(&matcher(Some(lp(0.05)), flat(), flat()), &matcher(None, flat(), flat()), &flight(), 700.0,
                               Hold::Thrust, LO, HI).unwrap_err();
    assert!(e.contains("below the least this load allows") && e.contains("913 K") && e.contains("142.") , "{e}");
}

#[test]
fn the_root_finder_refuses_above_its_range_and_the_cap_stops_at_the_limit() {
    let q = |t: f64| -> Result<f64, Abort> { if t < 800.0 { Err(Abort("no".into())) } else { Ok(t / 10.0) } };
    assert_eq!(bh::try_hold_root(&q, 500.0, 600.0, 2000.0), Err(HoldRefusal::AboveRange { t_top: 2000.0, q_top: 200.0 }));
    assert!(matches!(bh::try_hold_root(&q, 50.0, 600.0, 2000.0), Err(HoldRefusal::BelowFloor { .. })));
    let r = bh::try_hold_root(&q, 123.4, 600.0, 2000.0).unwrap();
    assert!((r.t - 1234.0).abs() < 1e-8 && (r.t_floor - 800.0).abs() < 1e-8);
    // The cap: excess = T − 1100 crosses at 1100 K inside [900, 1300]; none needed below; refused when over at the floor.
    let ex = |t: f64| -> Result<f64, Abort> { Ok(t - 1100.0) };
    assert!((bh::try_metal_cap(&ex, 900.0, 1300.0).unwrap().unwrap() - 1100.0).abs() < 1e-8);
    assert_eq!(bh::try_metal_cap(&ex, 900.0, 1050.0).unwrap(), None);
    assert!(bh::try_metal_cap(&ex, 1150.0, 1300.0).unwrap_err().contains("over its limit"));
}

// =============================================================================================
// SCORED — H1–H4 (anchor § 3), pinned at a subset of the spec's grid
// =============================================================================================

#[test]
fn h1_the_thrust_holds_burner_rise_within_its_curvature_bar() {
    let (ml, mh) = fp();
    for (so, ml, mh, t0, model, bar) in [(hp(0.05), flat(), flat(), 1500.0, 47.7, 5.2), (hp(0.02), flat(), flat(), 700.0, 41.5, 17.4),
                                         (lp(0.05), flat(), flat(), 1000.0, 78.4, 34.3), (hp(0.02), ml, mh, 700.0, 45.6, 24.7),
                                         (lp(0.05), ml, mh, 1500.0, 47.6, 6.2)] {
        let d = hold(so, ml, mh, t0, Hold::Thrust).tt4 - t0;
        assert!((d - model).abs() <= bar, "{so:?} {t0}: {d} vs {model} ± {bar}");
    }
}

#[test]
fn h2_h3_carrying_the_load_relieves_the_compressors_and_makes_an_lp_generator_a_gain() {
    let (fml, fmh) = fp();
    for (ml, mh) in [(flat(), flat()), (fml, fmh)] {
        let bare = matcher(None, ml, mh);
        for (so, t0) in [(hp(0.05), 1500.0), (hp(0.05), 1000.0), (lp(0.05), 1500.0), (lp(0.05), 1000.0)] {
            let b = bare.match_point(&flight(), t0);
            let fixed = matcher(Some(so), ml, mh).match_point(&flight(), t0);
            let h = hold(so, ml, mh, t0, Hold::Thrust).point;
            assert!(h.base.phi_lp > fixed.base.phi_lp && h.base.phi_hp > fixed.base.phi_hp, "H2 {so:?} {t0}");
            if so.lp.is_some() {
                assert!(h.base.phi_lp > b.base.phi_lp && h.base.phi_hp > b.base.phi_hp, "H3 {t0}");
            }
        }
    }
}

#[test]
fn h4_the_speed_hold_is_not_the_thrust_hold() {
    let (ml, mh) = fp();
    let bare = matcher(None, ml, mh);
    // An HP-shaft load: the speed hold burns hotter than the thrust hold and RAISES thrust.
    let (s, t) = (hold(hp(0.05), ml, mh, 1500.0, Hold::HpSpeed), hold(hp(0.05), ml, mh, 1500.0, Hold::Thrust));
    assert!(s.tt4 > t.tt4 && s.point.base.base.thrust > bare.match_point(&flight(), 1500.0).base.base.thrust);
    assert!((s.tt4 - 1500.0 - 98.0).abs() <= 7.3);
    // An LP-shaft load on flow/press: the speed hold COOLS the burner, and thrust falls more than at the fixed T0.
    let s = hold(lp(0.05), ml, mh, 1500.0, Hold::HpSpeed);
    assert!((s.tt4 - 1500.0 + 12.9).abs() <= 5.8, "{}", s.tt4);
    assert!(s.point.base.base.thrust < matcher(Some(lp(0.05)), ml, mh).match_point(&flight(), 1500.0).base.base.thrust);
}
