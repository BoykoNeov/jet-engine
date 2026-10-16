//! RUNG 93 — METAL TEMPERATURE. Anchor: `docs/plans/rung93-anchor-metal-temperature.md`; spec:
//! `docs/rung93-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 3, "no credit") first: Gauntner's published numbers, the
//! correlation's round trip, the reduce of the cooled design run, A ≡ the closed form, both B roots
//! at their limits, the refusals. Then the scored claims M1–M6, each pinned at a subset of the
//! spec's grid (the full grid is `docs/plans/scratch-archive/rung93/score.rs`).

use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::metal_temperature::{self as mt, FixedEngine, MetalInputs, MetalLimit, Row};
use turbojet::turbine_cooling::{mixed_tt, TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- rig A (rung 42's)
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

fn design_at(tt4: f64) -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, tt4, 50_000.0, REAL) }
fn design() -> TwoSpoolEngine { design_at(1500.0) }
fn flat() -> ComponentMap { ComponentMap::flat() }
fn fixed() -> FixedEngine { FixedEngine::new(design(), flight(), 1.0, flat(), flat()) }

fn limited_a(lim: &MetalLimit, tt4: f64) -> mt::LimitedDesign {
    mt::try_limited_design(&cpg(), 3.0, 6.0, tt4, 50_000.0, REAL, lim, &flight(), 1.0).expect("A")
}

// ---------------------------------------------------------------------------- the anchor's model
const S: f64 = 150.0 * 5.0 / 9.0;

fn need(factor: f64, tg: f64, tc: f64, p: f64, lim: f64) -> f64 {
    let y = (1.0 + p) * (tg + S - tc) / (lim - tc) - 1.0;
    if y <= 0.0 { 0.0 } else { 4.0 / 3.0 * factor * 0.022 * y.powf(1.25) }
}

/// Anchor § 1's model, as the probe wrote it: `Tt3 = tc0·(1 + a₁ε₁ + a₂ε₂)` (`a = 0`: Model 0, or
/// a fresh design), `f` frozen, both rows by fixed point.
fn model(factor: f64, tt4: f64, tc0: f64, f: f64, a1: f64, a2: f64) -> (f64, f64) {
    let g = cpg();
    let (lv, lb) = (mt::allowable(Row::Vane, 1985.0, 1e4), mt::allowable(Row::Blade, 1985.0, 1e4));
    let (mut e1, mut e2) = (0.0_f64, 0.0_f64);
    for _ in 0..200 {
        let tc = tc0 * (1.0 + a1 * e1 + a2 * e2);
        let gg = (1.0 - e1 - e2) * (1.0 + f);
        let n1 = need(factor, tt4, tc, 0.30, lv) * gg;
        let f41 = f * (1.0 - e1 - e2) / (1.0 - e2);
        let t41 = mixed_tt(&g, gg, tt4, f, n1, tc, f41);
        let n2 = need(factor, 0.92 * t41, tc, 0.13, lb) * (gg + n1);
        if (n1 - e1).abs() < 1e-13 && (n2 - e2).abs() < 1e-13 {
            return (n1, n2);
        }
        e1 = n1;
        e2 = n2;
    }
    panic!("model did not settle");
}

/// Model 1 and Model 0 at `tt4` on flat maps: the uncooled line's `Tt3`, `f`, and § 0 G2's slopes.
fn models_b(factor: f64, tt4: f64) -> ((f64, f64), (f64, f64)) {
    let mk = |e1: f64, e2: f64| {
        let e = if e1 + e2 > 0.0 { design().with_turbine_cooling(TurbineCooling::new(e1, e2)) } else { design() };
        TurbineCoolingMatcher::new(e, flight(), 1.0, flat(), flat(), 0.0).match_point(&flight(), tt4)
    };
    let (r0, rv, rb) = (mk(0.0, 0.0), mk(0.05, 0.0), mk(0.0, 0.02));
    let tc0 = r0.base.base.station("3").tt;
    let f0 = r0.base.base.station("4").far;
    let a1 = (rv.base.base.station("3").tt / tc0 - 1.0) / 0.05;
    let a2 = (rb.base.base.station("3").tt / tc0 - 1.0) / 0.02;
    (model(factor, tt4, tc0, f0, a1, a2), model(factor, tt4, tc0, f0, 0.0, 0.0))
}

// =============================================================================================
// CONSTRUCTION — the published correlation
// =============================================================================================

#[test]
fn gauntner_published_numbers_reproduce() {
    // His own example: FACTOR 1, phi 0.5 -> 0.022; his worked limits (degrees Rankine).
    assert!((mt::coolant_flow(1.0, 0.5) - 0.022).abs() < 1e-15);
    let r = |k: f64| k * 9.0 / 5.0;
    assert!((r(mt::allowable(Row::Vane, 1985.0, 1e4)) - 2210.0).abs() < 1e-9);
    assert!((r(mt::allowable(Row::Blade, 1985.0, 1e4)) - 2110.0).abs() < 1e-9);
    assert!((r(mt::allowable(Row::Vane, 2000.0, 1e4)) - 2360.0).abs() < 1e-9);
    assert!((r(mt::allowable(Row::Blade, 1977.8, 1e4)) - 2038.0).abs() < 1e-9);
    let lim = MetalLimit::gauntner_1985(1.0);
    assert!((lim.vane - 1227.777_777_777_777_8).abs() < 1e-9 && (lim.blade - 1172.222_222_222_222_3).abs() < 1e-9);
    // Kandiyoor (2024) Fig. 6.5 (TM 1100 K, TC 800 K, gas 2100 K) and the Y&W case's FACTOR 2.01.
    let phi = (2100.0 - 1100.0) / (2100.0 - 800.0);
    assert!((mt::coolant_flow(2.0, phi) - 0.1982).abs() < 1e-4, "{}", mt::coolant_flow(2.0, phi));
    assert!((mt::coolant_flow(1.0, phi) - 0.0991).abs() < 1e-4);
    let phi_yw = (1700.0 - 1100.0) / (1700.0 - 867.0);
    assert!((0.1447 / mt::coolant_flow(1.0, phi_yw) - 2.016).abs() < 1e-3);
    assert!((mt::SAFETY - 83.333_333_333_333_33).abs() < 1e-9);
}

#[test]
fn correlation_round_trips() {
    for &factor in &[2.0, 1.5, 1.0, 0.8] {
        for k in 1..20 {
            let phi = 0.05 * k as f64;
            let back = mt::effectiveness(factor, mt::coolant_flow(factor, phi));
            assert!((back - phi).abs() < 1e-12, "F={factor} phi={phi}: {back}");
        }
        // need -> metal lands ON the limit, for both rows, across gas and coolant temperatures.
        for &(t_row, tc, lim) in &[(1500.0, 700.0, 1228.0), (2000.0, 850.0, 1172.0), (1300.0, 600.0, 1100.0)] {
            for row in [Row::Vane, Row::Blade] {
                let w = mt::try_need(row, t_row, tc, lim, factor).unwrap();
                let m = mt::metal(row, t_row, tc, w, factor);
                if w > 0.0 {
                    assert!((m - lim).abs() < 1e-9, "{row:?} F={factor}: {m} vs {lim}");
                } else {
                    assert!(m <= lim);
                }
            }
        }
    }
    // No coolant: the metal is the hot spot itself. A limit above the hot spot needs no air.
    assert_eq!(mt::metal(Row::Vane, 1400.0, 650.0, 0.0, 1.0), mt::hot_spot(Row::Vane, 1400.0, 650.0));
    let hot = mt::hot_spot(Row::Blade, 1000.0, 600.0);
    assert_eq!(mt::try_need(Row::Blade, 1000.0, 600.0, hot + 1.0, 1.0).unwrap(), 0.0);
    // The hot spot is Gauntner's phi_hot = (P + phi)/(P + 1) written as a temperature.
    let (t, tc, tm) = (1500.0 + mt::SAFETY, 700.0, 1228.0);
    let phi = (t - tm) / (t - tc);
    let hot = mt::hot_spot(Row::Vane, 1500.0, tc);
    assert!(((hot - tm) / (hot - tc) - (0.3 + phi) / 1.3).abs() < 1e-14);
}

#[test]
fn a_limit_at_or_below_the_coolant_is_refused_in_words() {
    let e = mt::try_need(Row::Vane, 1500.0, 700.0, 700.0, 1.0).unwrap_err();
    assert!(e.0.contains("no amount of that air"), "{}", e.0);
    // A technology that needs more than a stream may take is refused at the ROOT, in words.
    let lim = MetalLimit::new(2.0, 760.0, 740.0);
    let e = mt::try_limited_design(&cpg(), 3.0, 6.0, 1600.0, 50_000.0, REAL, &lim, &flight(), 1.0).unwrap_err();
    assert!(e.0.contains("cannot hold the metal"), "{}", e.0);
}

// =============================================================================================
// CONSTRUCTION — the cooled design run
// =============================================================================================

#[test]
fn cooled_design_run_reduces_to_the_design_run() {
    mt::reset_counts();
    for tt4 in [1300.0, 1500.0, 2000.0] {
        let want = design_at(tt4).run(&flight(), 1.0);
        for cool in [None, Some(TurbineCooling::new(0.0, 0.0))] {
            let got = mt::try_cooled_design_run(&cpg(), 3.0, 6.0, tt4, 50_000.0, REAL, cool, &flight(), 1.0).unwrap();
            assert_eq!(got.stations.len(), want.stations.len());
            for ((la, a), (lb, b)) in got.stations.iter().zip(&want.stations) {
                assert_eq!(la, lb);
                for (x, y) in [(a.tt, b.tt), (a.pt, b.pt), (a.mdot, b.mdot), (a.far, b.far)] {
                    assert!(x.to_bits() == y.to_bits(), "Tt4={tt4} station {la}");
                }
            }
            assert!(got.st_inlet.to_bits() == want.performance.specific_thrust.to_bits());
            assert!(got.tsfc_inlet.to_bits() == want.performance.tsfc.to_bits());
        }
        // The cooled body itself, at a vanishing share, lands on the design run.
        let tiny = mt::try_cooled_design_run(&cpg(), 3.0, 6.0, tt4, 50_000.0, REAL,
                                             Some(TurbineCooling::new(1e-9, 1e-9)), &flight(), 1.0).unwrap();
        assert!((tiny.st_inlet / want.performance.specific_thrust - 1.0).abs() < 1e-7);
        assert!((tiny.tsfc_inlet / want.performance.tsfc - 1.0).abs() < 1e-7);
    }
    assert_eq!(mt::counts(), [3, 6], "inactive cooling dispatches to TwoSpoolEngine::run");
}

#[test]
fn a_limited_is_the_closed_form() {
    // A fresh design's Tt3 and f do not see the cooling, so anchor § 1's model is exact there.
    for &factor in &[2.0, 1.0, 0.8] {
        let lim = MetalLimit::gauntner_1985(factor);
        for tt4 in [1300.0, 1700.0, 2200.0] {
            let a = limited_a(&lim, tt4);
            let r = design_at(tt4).run(&flight(), 1.0);
            let st = |l: &str| r.stations.iter().find(|(x, _)| *x == l).unwrap().1;
            let (e1, e2) = model(factor, tt4, st("3").tt, st("4").far, 0.0, 0.0);
            assert!((a.vane - e1).abs() < 1e-9 && (a.blade - e2).abs() < 1e-9,
                    "F={factor} Tt4={tt4}: ({}, {}) vs ({e1}, {e2})", a.vane, a.blade);
            assert!((a.reading.vane - lim.vane).abs() < 1e-6 && (a.reading.blade - lim.blade).abs() < 1e-6);
            assert!(a.passes <= 5, "Tt41 depends only on the vane stream's ratio: {} passes", a.passes);
        }
    }
}

// =============================================================================================
// CONSTRUCTION — framing B's two roots
// =============================================================================================

#[test]
fn b_limited_puts_both_metals_at_their_limits_and_round_trips() {
    let fe = fixed();
    for (factor, tt4) in [(1.0, 1300.0), (1.0, 1700.0), (2.0, 2200.0)] {
        let lim = MetalLimit::gauntner_1985(factor);
        let l = fe.try_limited(&lim, &flight(), tt4).unwrap();
        assert!((l.reading.vane - lim.vane).abs() < 1e-6 && (l.reading.blade - lim.blade).abs() < 1e-6,
                "F={factor} Tt4={tt4}: {:?}", l.reading);
        // The knob form at those shares returns the same Tt4.
        let mx = fe.try_max_tt4(TurbineCooling::new(l.vane, l.blade), &lim, &flight(), 900.0, 2400.0).unwrap();
        assert!((mx.tt4 - tt4).abs() < 1e-5, "F={factor}: {} vs {tt4}", mx.tt4);
        // The reading off the point is the inputs the root used.
        assert_eq!(MetalInputs::of_cooled(&l.point), l.inputs);
    }
}

#[test]
fn b_knob_refuses_in_words_when_the_range_does_not_bracket() {
    let fe = fixed();
    let lim = MetalLimit::gauntner_1985(1.0);
    let e = fe.try_max_tt4(TurbineCooling::new(0.0, 0.0), &lim, &flight(), 1100.0, 1300.0).unwrap_err();
    assert!(e.0.contains("over its limit"), "{}", e.0);
    let e = fe.try_max_tt4(TurbineCooling::new(0.10, 0.05), &lim, &flight(), 900.0, 1000.0).unwrap_err();
    assert!(e.0.contains("holds all the way"), "{}", e.0);
}

#[test]
#[should_panic(expected = "sets the turbine cooling itself")]
fn fixed_engine_refuses_a_design_already_cooled() {
    FixedEngine::new(design().with_turbine_cooling(TurbineCooling::new(0.05, 0.0)), flight(), 1.0, flat(), flat());
}

#[test]
#[should_panic(expected = "perfect gas only")]
fn fixed_engine_refuses_a_non_perfect_gas() {
    let e = build_two_spool_turbojet(Gas::thermally_perfect(), 3.0, 6.0, 1500.0, 50_000.0, REAL);
    FixedEngine::new(e, flight(), 1.0, flat(), flat());
}

// =============================================================================================
// SCORED — M1–M6 (anchor § 3), pinned at a subset of the spec's grid
// =============================================================================================

#[test]
fn m1_m2_b_against_the_model() {
    let fe = fixed();
    for (factor, tt4) in [(1.0, 1300.0), (1.0, 2000.0), (2.0, 2200.0), (0.8, 1700.0)] {
        let lim = MetalLimit::gauntner_1985(factor);
        let l = fe.try_limited(&lim, &flight(), tt4).unwrap();
        let (m1, m0) = models_b(factor, tt4);
        assert!((l.vane - m1.0).abs() < 1.5e-3 && (l.blade - m1.1).abs() < 1.5e-3,
                "M1 F={factor} Tt4={tt4}: ({}, {}) vs ({}, {})", l.vane, l.blade, m1.0, m1.1);
        assert!(l.vane + l.blade < m0.0 + m0.1, "M2 F={factor} Tt4={tt4}");
    }
}

#[test]
fn m3_the_fixed_and_fresh_engines_cross() {
    let fe = fixed();
    let gap = |factor: f64, t: f64| {
        let lim = MetalLimit::gauntner_1985(factor);
        let b = fe.try_limited(&lim, &flight(), t).unwrap();
        let a = limited_a(&lim, t);
        (b.vane + b.blade) - (a.vane + a.blade)
    };
    // FACTOR 1.0: the crossing sits inside 1676 +- 70 K, negative below, positive above.
    assert!(gap(1.0, 1606.0) < 0.0 && gap(1.0, 1746.0) > 0.0);
    assert!(gap(1.0, 1300.0) < 0.0 && gap(1.0, 2200.0) > 0.0);
    // FACTOR 2.0: the fixed engine needs less throughout.
    assert!(gap(2.0, 1300.0) < 0.0 && gap(2.0, 2200.0) < 0.0);
}

#[test]
fn m4_m5_no_best_share_and_cooling_lowers_the_fuel_bill() {
    let fe = fixed();
    for &factor in &[2.0, 1.0] {
        let lim = MetalLimit::gauntner_1985(factor);
        let mut prev_b = f64::NEG_INFINITY;
        for t in [1300.0, 1750.0, 2200.0] {
            let st = fe.try_limited(&lim, &flight(), t).unwrap().point.booking.unwrap().st_inlet;
            assert!(st > prev_b, "M4 (B) F={factor} Tt4={t}");
            prev_b = st;
        }
        let (mut prev_a, mut prev_gap) = (f64::NEG_INFINITY, f64::INFINITY);
        for k in 0..10 {
            let t = 1300.0 + 100.0 * k as f64;
            let a = limited_a(&lim, t);
            assert!(a.run.st_inlet > prev_a, "M4 (A) F={factor} Tt4={t}");
            let gap = a.run.tsfc_inlet - design_at(t).run(&flight(), 1.0).performance.tsfc;
            assert!(gap < 0.0 && gap < prev_gap, "M5 F={factor} Tt4={t}: {gap}");
            prev_a = a.run.st_inlet;
            prev_gap = gap;
        }
    }
}

#[test]
fn m6_vane_air_hands_the_limit_to_the_blade() {
    let fe = fixed();
    let lim = MetalLimit::gauntner_1985(1.0);
    let row_lim = |e1: f64, row: Row| -> f64 {
        let m = fe.matcher(TurbineCooling::new(e1, 0.02));
        let g = |t: f64| {
            let r = mt::read(&MetalInputs::of_cooled(&m.match_point(&flight(), t)), &lim);
            match row { Row::Vane => r.vane - lim.vane, Row::Blade => r.blade - lim.blade }
        };
        let (mut a, mut b) = (800.0, 2400.0);
        for _ in 0..50 {
            let mid = 0.5 * (a + b);
            if g(mid) < 0.0 { a = mid } else { b = mid }
        }
        a
    };
    // The crossover sits inside 3.38 +- 0.10 %: the vane binds below, the blade above.
    assert!(row_lim(0.0328, Row::Vane) < row_lim(0.0328, Row::Blade));
    assert!(row_lim(0.0348, Row::Vane) > row_lim(0.0348, Row::Blade));
    let at = |e1: f64| fe.try_max_tt4(TurbineCooling::new(e1, 0.02), &lim, &flight(), 800.0, 2400.0).unwrap();
    assert_eq!(at(0.02).binding, Row::Vane);
    assert_eq!(at(0.04).binding, Row::Blade);
    // The blade's own limit rises with vane air (the vane coolant dilutes the rotor's gas).
    let (b0, b8, b16) = (row_lim(0.0, Row::Blade), row_lim(0.08, Row::Blade), row_lim(0.16, Row::Blade));
    assert!(b0 < b8 && b8 < b16, "{b0} {b8} {b16}");
}
