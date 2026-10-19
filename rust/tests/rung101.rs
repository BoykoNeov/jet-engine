//! RUNG 101 — THE AIRFLOW OFF THE LINE. Anchor: `docs/plans/rung101-anchor-airflow-source.md`; spec:
//! `docs/rung101-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 2, "no credit") first: the default and the inert settings, the speed grid's
//! airflow where it is the carried table's, the push gate under it, the sensor gate (the nozzle's closed form against
//! the plant's own face flow), its closed-loop consequence (rung 95's thrust governor), the lag's start and a run with
//! no move, the lag's range. Then the scored claims, pinned at a subset of the anchor's cells (the full set:
//! `docs/plans/scratch-archive/rung101/score.rs`), the MISSES pinned as found.

use turbojet::airflow_source::{nozzle_flow, sensed_air, Air, AirflowFrom, Sensed, TAU_MAX, TAU_MIN};
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::inferred_thrust::face;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, controls, controls_precheck, ControlsSettings, ControlsSolver, CustomerMode,
                                 LeverChoice, ThrustBy};
use turbojet::unchoked_formula::{Formula, Push};

fn gov(by: ThrustBy) -> ControlsSettings {
    ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, ..ControlsSettings::defaults() }
}
fn air(s: ControlsSettings, m: f64) -> ControlsSettings { ControlsSettings { t0: 250.0, p0: 50_000.0, m0: m, ..s } }
fn formula(s: ControlsSettings, a: AirflowFrom) -> ControlsSettings {
    ControlsSettings { speed_corr: SpeedCorr::Formula, airflow: a, ..s }
}
fn bits(p: &[FuelPoint]) -> Vec<[u64; 4]> { p.iter().map(|q| [q.nu_lp, q.nu_hp, q.mf, q.tt4].map(f64::to_bits)).collect() }
fn run_bits(s: &ControlsSettings) -> Vec<[u64; 4]> { bits(&controls(s).unwrap_or_else(|e| panic!("{}", e.0)).1.points) }
fn solver(s: &ControlsSettings) -> ControlsSolver { build(s).unwrap_or_else(|e| panic!("{}", e.0)) }
/// Every recorded point's (true thrust, believed thrust, gap rest).
fn beliefs(sv: &ControlsSolver) -> Vec<(f64, f64, Option<f64>)> {
    sv.march_marked().0.iter().map(|p| {
        let r = sv.read(p).unwrap_or_else(|e| panic!("{}", e.0));
        (r.thrust, r.inferred.expect("an inferred run"), r.gap_nozzle)
    }).collect()
}
fn variants() -> [ControlsSettings; 4] {
    let d = ControlsSettings::defaults();
    [d, ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..d }, ControlsSettings { customer: CustomerMode::Share, ..d },
     ControlsSettings { lever: LeverChoice::Bleed, ..d }]
}
fn with(s: ControlsSettings, v: &ControlsSettings) -> ControlsSettings {
    ControlsSettings { cool_vane: v.cool_vane, cool_blade: v.cool_blade, customer: v.customer, lever: v.lever, ..s }
}

// =============================================================================================
// CONSTRUCTION
// =============================================================================================

#[test]
fn the_default_is_rung_100s_table_and_the_lag_is_off() {
    let d = ControlsSettings::defaults();
    assert!(d.airflow == AirflowFrom::Table && !d.exhaust_lag && d.exhaust_tau == 1.0);
    assert!(TAU_MIN == 0.05 && TAU_MAX == 5.0);
}

#[test]
fn the_new_settings_are_inert_where_they_do_not_act() {
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        // Under the speed grid, and with no speed correction: no formula is built.
        for corr in [SpeedCorr::Grid, SpeedCorr::None] {
            let s = ControlsSettings { speed_corr: corr, ..air(gov(by), 1.4) };
            let x = ControlsSettings { airflow: AirflowFrom::Exhaust, exhaust_lag: true, exhaust_tau: 0.3, ..s };
            assert_eq!(run_bits(&s), run_bits(&x), "{corr:?} {by:?}");
        }
        // The lag acts only on the sensor.
        for a in [AirflowFrom::Table, AirflowFrom::Grid] {
            let s = formula(air(gov(by), 1.4), a);
            assert_eq!(run_bits(&s), run_bits(&ControlsSettings { exhaust_lag: true, exhaust_tau: 0.3, ..s }), "{a:?} {by:?}");
        }
    }
    // At the design flight no formula is built at all.
    let s = formula(gov(ThrustBy::Epr), AirflowFrom::Exhaust);
    assert!(solver(&s).tables.is_none());
    assert_eq!(run_bits(&s), run_bits(&gov(ThrustBy::Epr)));
}

#[test]
fn at_the_design_slices_mach_the_grid_airflow_is_the_carried_tables() {
    // Mach 0.85 at another altitude: the grid's one slice IS the design flight, so its airflow column is the carried
    // design table's. On the pressure ratio both read it by the same straight line in the reading — rung 100's run,
    // bit for bit. On the fan speed the table's column is read along B's row parameter and the grid's along the fan
    // speed: the same rows, a different straight line between them, so equal to rounding only.
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        let s = ControlsSettings { t0: 230.0, p0: 30_000.0, m0: 0.85, from: 900.0, ..gov(by) };
        let (t, g) = (formula(s, AirflowFrom::Table), formula(s, AirflowFrom::Grid));
        let sv = solver(&g);
        let infer = sv.governor.as_ref().and_then(|x| x.infer.as_ref()).expect("an inferred run");
        assert!(matches!(infer.formula, Some(Formula::Epr { air: Air::Grid(_), .. }) | Some(Formula::Fan { air: Air::Grid(_), .. })));
        match by {
            ThrustBy::Epr => assert_eq!(run_bits(&t), run_bits(&g)),
            _ => {
                let (a, b) = (controls(&t).unwrap().1.points, controls(&g).unwrap().1.points);
                assert_eq!(a.len(), b.len());
                let w = a.iter().zip(&b).map(|(p, q)| ((p.mf - q.mf) / q.mf).abs()).fold(0.0f64, f64::max);
                assert!(w <= 1e-9, "fan speed: worst fuel {w:e}");
            }
        }
    }
}

#[test]
fn the_push_gate_holds_under_the_grid_airflow() {
    // Rung 100's: the rest after the airflow guess is zero at every point, choked or not — now with the grid's guess.
    for (m, from) in [(0.05, 700.0), (1.4, 1000.0)] {
        let sv = solver(&ControlsSettings { from, ..formula(air(gov(ThrustBy::Epr), m), AirflowFrom::Grid) });
        let rest = beliefs(&sv).iter().map(|q| q.2.expect("the formula splits the gap").abs()).fold(0.0f64, f64::max);
        assert!(rest <= 1e-9, "Mach {m}: rest {rest:e} N per kg/s");
    }
}

#[test]
fn the_sensor_gate_the_belief_is_the_true_thrust_at_every_point() {
    // The nozzle's closed form against the plant's own face flow: four engines, unchoked (Mach 0.05) and choked (1.4).
    for v in &variants() {
        for (m, from) in [(0.05, 700.0), (1.4, 1000.0)] {
            let s = ControlsSettings { from, ..with(formula(air(gov(ThrustBy::Epr), m), AirflowFrom::Exhaust), v) };
            let gap = beliefs(&solver(&s)).iter().map(|q| (q.0 - q.1).abs()).fold(0.0f64, f64::max);
            assert!(gap <= 1e-9, "Mach {m} {:?} {:?}: |F - F_hat| {gap:e} N per kg/s", v.customer, v.lever);
        }
    }
}

#[test]
fn the_sensor_closes_the_plants_mass_bookkeeping() {
    // At the steady start, directly: (nozzle flow − fuel)/(share reaching the nozzle) is the face air.
    for v in &variants() {
        let sv = solver(&with(formula(air(gov(ThrustBy::Epr), 1.4), AirflowFrom::Exhaust), v));
        let i = &sv.start;
        let push = Push::new(&sv.core.fuel, sv.flight.p0);
        let r = sv.core.fuel.inner.gas().r_t_at(0.0);
        let pt5 = i.pi_lpt * i.pi_hpt * i.close.pt4;
        let got = sensed_air(&push, r, pt5, &Sensed::of(i, None));
        assert!((got - face(i)).abs() <= 1e-12, "{got} vs {}", face(i));
        assert!(nozzle_flow(&push, r, pt5, i.tt5) > face(i) * 0.5);
    }
}

#[test]
fn closed_loop_the_unlagged_sensor_is_rung_95s_thrust_governor() {
    for (m, from) in [(0.05, 700.0), (1.4, 700.0)] {
        let s = ControlsSettings { from, ..formula(air(gov(ThrustBy::Epr), m), AirflowFrom::Exhaust) };
        let thrusts = |sv: &ControlsSolver| -> Vec<f64> {
            sv.march_marked().0.iter().map(|p| sv.read(p).unwrap_or_else(|e| panic!("{}", e.0)).thrust).collect()
        };
        let a = thrusts(&solver(&s));
        let b = thrusts(&solver(&ControlsSettings { thrust_by: ThrustBy::True, ..s }));
        assert_eq!(a.len(), b.len());
        let w = a.iter().zip(&b).map(|(x, y)| ((x - y) / y).abs()).fold(0.0f64, f64::max);
        assert!(w <= 1e-6, "Mach {m}: worst {w:e}");
    }
}

#[test]
fn the_lagged_sensor_starts_at_the_steady_start_and_records_its_reading() {
    let s = ControlsSettings { exhaust_lag: true, ..formula(air(gov(ThrustBy::Epr), 1.4), AirflowFrom::Exhaust) };
    let sv = solver(&s);
    let lag = sv.sensor_lag().expect("a lagged sensor");
    assert_eq!(lag.t0.to_bits(), sv.start.tt5.to_bits());
    let (pts, _) = sv.march_marked();
    assert!(matches!(pts[0].extra, PointExtra::Sensor { tm } if tm.to_bits() == sv.start.tt5.to_bits()));
    assert!(pts.iter().all(|p| matches!(p.extra, PointExtra::Sensor { .. }) && p.key_count() == 15));
    // The start's belief is the unlagged one's, to the governor's own root.
    let unl = solver(&ControlsSettings { exhaust_lag: false, ..s });
    let (a, b) = (beliefs(&sv)[0], beliefs(&unl)[0]);
    assert!(((a.1 - b.1) / b.1).abs() <= 1e-6, "{a:?} {b:?}");
    // A recorded point re-runs from its recorded reading: the next point, bit for bit.
    let next = turbojet::sandbox_controls::replay_step(&sv, &pts[3]).expect("the governed route re-runs").unwrap();
    assert_eq!([next.0, next.1, next.2].map(f64::to_bits),
               [pts[4].nu_lp, pts[4].nu_hp, match pts[4].extra { PointExtra::Sensor { tm } => tm, _ => f64::NAN }]
                   .map(f64::to_bits));
}

#[test]
fn a_faster_sensor_comes_closer_to_the_instant_one() {
    // The anchor's "run with no move" cannot be built under the formula (a design table between equal throttles has no
    // span — refused); the lag's reduce is checked by its trend instead: the TRUE thrust's worst gap to the instant
    // sensor's run falls with the lag, at most in proportion. Not the belief's: on the pressure ratio the governor holds
    // the BELIEF on the demand line whatever the lag (measured: 3e−12 apart) — the lag shows in the thrust delivered.
    // Measured (`scratch-archive/rung101/lagorder.txt`): 1.32–1.69 per doubling, SUBLINEAR — the governor moves the
    // fuel with the demand, so the exhaust temperature jumps at the move's start and even a fast probe misses part of
    // the jump. The band is that measurement's, widened.
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        let s = formula(air(gov(by), 0.5), AirflowFrom::Exhaust);
        let base: Vec<f64> = beliefs(&solver(&s)).iter().map(|q| q.0).collect();
        let gap = |tau: f64| {
            let b = beliefs(&solver(&ControlsSettings { exhaust_lag: true, exhaust_tau: tau, ..s }));
            assert_eq!(b.len(), base.len());
            b.iter().zip(&base).map(|(x, y)| (x.0 - y).abs()).fold(0.0f64, f64::max)
        };
        let (g1, g2, g4) = (gap(0.05), gap(0.1), gap(0.2));
        for r in [g2 / g1, g4 / g2] {
            assert!(g1 > 0.0 && (1.2..=2.0).contains(&r), "{by:?}: {g1} {g2} {g4}");
        }
    }
}

#[test]
fn the_lag_is_refused_outside_its_range_where_it_acts() {
    let s = ControlsSettings { exhaust_lag: true, ..ControlsSettings::defaults() };
    for t in [TAU_MIN, 1.0, TAU_MAX] {
        assert!(controls_precheck(&ControlsSettings { exhaust_tau: t, ..s }).is_ok(), "{t}");
    }
    for t in [0.04, 5.1, f64::NAN] {
        let e = controls_precheck(&ControlsSettings { exhaust_tau: t, ..s }).expect_err("refused");
        assert!(t.is_nan() || e.0.contains("exhaust sensor's lag"), "{}", e.0);
    }
    assert!(controls_precheck(&ControlsSettings { exhaust_tau: 0.04, ..ControlsSettings::defaults() }).is_ok(),
            "inert with the lag off");
}

#[test]
fn the_page_fills_in_the_three_keys_at_the_models_defaults() {
    // They are written to a reply only off their defaults, so the page's opening settings lack them and it fills them
    // in itself (`docs/sandbox/template.html`, at `controls_defaults`): those values must be the model's.
    let d = ControlsSettings::defaults();
    let page = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/sandbox/template.html")).unwrap();
    let want = format!("{{ airflow: \"{}\", exhaust_lag: {}, exhaust_tau: {} }}", d.airflow.key(), d.exhaust_lag as i32, d.exhaust_tau);
    assert!(page.contains(&want), "the page's fill-in is not {want}");
}

// =============================================================================================
// SCORED (anchor § 3) — pinned at a few cells; the full set is `scratch-archive/rung101/score.rs` (778 HIT, 46 MISS,
// 56 VACUOUS of 880). "start" here is the calibrated starting end's (achieved − asked)/asked.
// =============================================================================================

fn start_miss(s: &ControlsSettings) -> f64 {
    let a = solver(s).hold_start.expect("a carried inferred run calibrates its ends");
    (a.achieved - a.target) / a.target
}
fn cell(by: ThrustBy, m0: f64, from: f64) -> ControlsSettings { ControlsSettings { from, ..air(gov(by), m0) } }

#[test]
fn h1_the_speed_tables_fix_the_pressure_ratios_steady_start() {
    // Mach 1.2 from 750 K (a cell; 112 / 112 over the anchor's).
    let (t, g) = (start_miss(&formula(cell(ThrustBy::Epr, 1.2, 750.0), AirflowFrom::Table)),
                  start_miss(&formula(cell(ThrustBy::Epr, 1.2, 750.0), AirflowFrom::Grid)));
    assert!(g.abs() < 0.007 && g.abs() < t.abs(), "grid {g} table {t}");
}

#[test]
fn h4_a_slower_sensor_costs_more_in_the_move() {
    // 112 / 112: the move's worst (true − believed)/true, positive and rising with the lag.
    let mv = |tau: f64| {
        let s = ControlsSettings { exhaust_lag: true, exhaust_tau: tau,
                                   ..formula(cell(ThrustBy::Epr, 0.7, 750.0), AirflowFrom::Exhaust) };
        beliefs(&solver(&s)).iter().filter(|q| q.0 >= 20.0).map(|q| (q.0 - q.1) / q.0).fold(0.0f64, |a, x| if x.abs() > a.abs() { x } else { a })
    };
    let (a, b, c) = (mv(0.1), mv(0.5), mv(2.0));
    assert!(0.0 < a && a < b && b < c, "{a} {b} {c}");
}

#[test]
fn h6_h7_the_sensor_helps_the_fan_speed_fast() {
    // Mach 1.2 from 750 K: the start's miss well under rung 100's (77 HIT, 3 MISS — the bars, both ways — 32 VACUOUS).
    let (t, x) = (start_miss(&formula(cell(ThrustBy::N1, 1.2, 750.0), AirflowFrom::Table)),
                  start_miss(&formula(cell(ThrustBy::N1, 1.2, 750.0), AirflowFrom::Exhaust)));
    assert!(x.abs() < 0.7 * t.abs(), "sensor {x} table {t}");
}

#[test]
fn h8_miss_the_speed_tables_hurt_the_fan_speed_only_below_choke() {
    // 38 HIT, 42 MISS: the lost cancellation needs B's PUSH to be wrong, which it is only below choke (the start, or
    // its lowest design rows). Mach 0.25 from 750 K on the cooled engine — start unchoked: worse, as predicted.
    // Mach 1.2 from 750 K, plain — start and rows choked, B is rung 99's exact column there and its only error the
    // airflow guess: the speed tables make it BETTER (`scratch-archive/rung101/h8why.txt`).
    let cool = ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..cell(ThrustBy::N1, 0.25, 750.0) };
    let (t, g) = (start_miss(&formula(cool, AirflowFrom::Table)), start_miss(&formula(cool, AirflowFrom::Grid)));
    assert!(g.abs() > t.abs() && g < 0.0 && t < 0.0, "unchoked: grid {g} table {t}");
    let plain = cell(ThrustBy::N1, 1.2, 750.0);
    assert!(solver(&formula(plain, AirflowFrom::Table)).start.branch == turbojet::matcher::Branch::Choked);
    let (t, g) = (start_miss(&formula(plain, AirflowFrom::Table)), start_miss(&formula(plain, AirflowFrom::Grid)));
    assert!(g.abs() < t.abs(), "choked: grid {g} table {t}");
}
