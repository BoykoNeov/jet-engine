//! RUNG 98 — THE CORRECTED TABLES. Anchor: `docs/plans/rung98-anchor-corrected-tables.md`; spec: `docs/rung98-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 2, "no credit") first: tables made at this flight, flown at the design flight, or
//! with no table read are the old code bit for bit; the switches with nothing to correct are inert; the pressure
//! correction reproduces a table built at the flight by its own solves; the pressure-off sign; the cross-flight thrust
//! split; the out-of-reach refusal. Then the scored claims H1–H4, pinned at a subset of the anchor's grid (the full
//! grid is `docs/plans/scratch-archive/rung98/score.rs`), H4's MISS pinned as the finding it is.

use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::TableAt;
use turbojet::fuel_transient::FuelPoint;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, CustomerMode, GeneratorBy, GeneratorMode, Holder,
                                 LeverChoice, SpeedLaw, ThrustBy};

const FLOW_PRESS: usize = 0;
const FLAT_LP: usize = 3;

fn gov(by: ThrustBy) -> ControlsSettings {
    ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, ..ControlsSettings::defaults() }
}
fn accel() -> ControlsSettings { ControlsSettings { accel_on: true, ..ControlsSettings::defaults() } }
fn prop() -> ControlsSettings {
    ControlsSettings { govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, ..ControlsSettings::defaults() }
}
fn at(s: ControlsSettings, t0: f64, p0: f64, m0: f64) -> ControlsSettings { ControlsSettings { t0, p0, m0, ..s } }
fn hot(s: ControlsSettings) -> ControlsSettings { at(s, 290.0, 50_000.0, 0.85) }
fn cold(s: ControlsSettings) -> ControlsSettings { at(s, 216.65, 50_000.0, 0.85) }
// Rung 99 made the speed grid the default; off Mach 0.85 these tests keep testing rung 98's one design-flight table.
fn m14(s: ControlsSettings) -> ControlsSettings { at(ControlsSettings { speed_corr: SpeedCorr::None, ..s }, 250.0, 50_000.0, 1.4) }
fn bits(p: &[FuelPoint]) -> Vec<[u64; 4]> { p.iter().map(|q| [q.nu_lp, q.nu_hp, q.mf, q.tt4].map(f64::to_bits)).collect() }
fn run_bits(s: &ControlsSettings) -> Vec<[u64; 4]> { bits(&controls(s).unwrap_or_else(|e| panic!("{}", e.0)).1.points) }
fn rel(a: f64, b: f64) -> f64 { ((a - b) / b).abs() }
/// Each end's (achieved − target)/target.
fn miss(s: &ControlsSettings) -> [f64; 2] {
    let sv = build(s).unwrap_or_else(|e| panic!("{}", e.0));
    let (a, b) = (sv.hold_start.unwrap(), sv.hold_end.unwrap());
    [(a.achieved - a.target) / a.target, (b.achieved - b.target) / b.target]
}

// =============================================================================================
// CONSTRUCTION
// =============================================================================================

#[test]
fn flown_at_the_design_flight_a_design_table_is_the_runs_own_bit_for_bit() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        for base in [gov(ThrustBy::Epr), gov(ThrustBy::N1), accel(), prop()] {
            let base = ControlsSettings { shape, ..base };
            let reference = run_bits(&ControlsSettings { table_at: TableAt::Flight, ..base });
            for (p, t) in [(true, true), (false, true), (true, false), (false, false)] {
                let s = ControlsSettings { table_at: TableAt::Design, corr_p: p, corr_t: t, ..base };
                assert!(build(&s).unwrap().carry.is_none(), "at the design flight nothing is carried");
                assert_eq!(run_bits(&s), reference);
            }
        }
    }
}

#[test]
fn a_run_that_reads_no_table_is_unchanged_anywhere() {
    // Off the design flight, with no inferred thrust, no schedule and no proportional law, the setting is inert.
    for s in [ControlsSettings::defaults(), ControlsSettings { govern: true, hold: Hold::HpSpeed, ..ControlsSettings::defaults() },
              gov(ThrustBy::True)] {
        for f in [hot as fn(ControlsSettings) -> ControlsSettings, m14] {
            let (a, b) = (f(ControlsSettings { table_at: TableAt::Design, ..s }), f(ControlsSettings { table_at: TableAt::Flight, ..s }));
            assert!(build(&a).unwrap().carry.is_none());
            assert_eq!(run_bits(&a), run_bits(&b));
        }
    }
}

#[test]
fn a_switch_with_nothing_to_correct_is_inert() {
    // The pressure ratio and thrust over delta carry no temperature; Wf/pt3 over the corrected HP speed no pressure.
    let e = m14(gov(ThrustBy::Epr));
    assert!(build(&e).unwrap().carry.is_some());
    assert_eq!(run_bits(&e), run_bits(&ControlsSettings { corr_t: false, ..e }));
    let a = m14(accel());
    assert_eq!(run_bits(&a), run_bits(&ControlsSettings { corr_p: false, ..a }));
}

#[test]
fn the_pressure_correction_reproduces_a_table_built_at_the_flight() {
    // A pressure-only flight (theta = 1 exactly): the design-flight tables carried by delta against the tables rungs
    // 96/48/95 build AT the flight by their own solves. Bar 1e-9 (anchor § 2; G1 measured <= 3.6e-11).
    let thin = |s| at(s, 250.0, 35_000.0, 0.85);
    let mut worst = 0.0f64;
    for shape in [FLOW_PRESS, FLAT_LP] {
        for v in [ControlsSettings::defaults(), ControlsSettings { customer: CustomerMode::Share, ..ControlsSettings::defaults() },
                  ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..ControlsSettings::defaults() },
                  ControlsSettings { lever: LeverChoice::Bleed, ..ControlsSettings::defaults() }] {
            for base in [gov(ThrustBy::Epr), ControlsSettings { accel_on: true, ..prop() }] {
                let s = thin(ControlsSettings { shape, govern: base.govern, hold: base.hold, thrust_by: base.thrust_by,
                                                speed_law: base.speed_law, accel_on: base.accel_on, ..v });
                let (d, f) = (build(&s).unwrap(), build(&ControlsSettings { table_at: TableAt::Flight, ..s }).unwrap());
                let c = d.carry.expect("carried");
                assert!(c.theta == 1.0 && c.k == 1.0, "a pressure-only flight has theta = 1 exactly");
                let (gd, gf) = (d.governor.as_ref().unwrap(), f.governor.as_ref().unwrap());
                if let (Some(td), Some(tf)) = (gd.infer.as_ref(), gf.infer.as_ref()) {
                    for (a, b) in td.rows.iter().zip(&tf.rows) {
                        for (x, y) in [(a.thrust, b.thrust), (a.epr, b.epr), (a.nu_lp, b.nu_lp), (a.fuel, b.fuel), (a.face, b.face)] {
                            worst = worst.max(rel(x, y));
                        }
                    }
                }
                for (a, b) in gd.table.iter().zip(&gf.table) {
                    worst = worst.max(rel(a.0, b.0)).max(rel(a.1, b.1));
                }
                if let (Some(ad), Some(af)) = (d.accel.as_ref(), f.accel.as_ref()) {
                    for k in 0..ad.n_h.len() {
                        worst = worst.max(rel(ad.n_h[k], af.n_h[k])).max(rel(ad.kappa[k], af.kappa[k]));
                    }
                }
            }
        }
    }
    assert!(worst <= 1e-9, "carried design-flight table off the flight's own by {worst:e}");
}

#[test]
fn a_pressure_only_flight_runs_the_same_move_as_a_table_made_there() {
    // Bar 1e-7 (anchor § 2, derived from the calibrated ends' bisection and the governor's root tolerance).
    for shape in [FLOW_PRESS, FLAT_LP] {
        let s = at(ControlsSettings { shape, ..gov(ThrustBy::Epr) }, 250.0, 35_000.0, 0.85);
        let (_, a) = controls(&s).unwrap();
        let (_, b) = controls(&ControlsSettings { table_at: TableAt::Flight, ..s }).unwrap();
        assert_eq!(a.points.len(), b.points.len());
        let w = a.points.iter().zip(&b.points)
            .map(|(x, y)| rel(x.tt4, y.tt4).max(rel(x.sp_thrust * x.mdot_air, y.sp_thrust * y.mdot_air)))
            .fold(0.0f64, f64::max);
        assert!(w <= 1e-7, "pressure-only run off the flight's own by {w:e}");
    }
}

#[test]
fn with_the_pressure_correction_off_thin_air_gets_too_little_thrust_and_dense_air_too_much() {
    // Anchor § 2's sign gate (the first draft's H1, construction).
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        for (p0, over) in [(26_500.0, false), (80_000.0, true)] {
            let m = miss(&at(ControlsSettings { corr_p: false, ..gov(by) }, 250.0, p0, 0.85));
            assert!(m.iter().all(|&x| if over { x > 0.0 } else { x < 0.0 }), "{by:?} p0 {p0}: {m:?}");
        }
    }
}

#[test]
fn the_pressure_ratio_gap_splits_into_ram_drag_and_a_constant_nozzle_term() {
    // Anchor § 2 / G6: on a design-flight table read elsewhere (pressure corrected), the gap's NOZZLE part is the same
    // at every choked point of the run. Bar: spread <= 1e-12 x the thrust.
    let gen = ControlsSettings { generator: GeneratorMode::Hp, generator_by: GeneratorBy::Share, ..ControlsSettings::defaults() };
    for v in [ControlsSettings::defaults(), ControlsSettings { customer: CustomerMode::Share, ..ControlsSettings::defaults() },
              ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..ControlsSettings::defaults() },
              ControlsSettings { lever: LeverChoice::Bleed, ..ControlsSettings::defaults() }, gen] {
        for f in [hot as fn(ControlsSettings) -> ControlsSettings, m14] {
            let s = f(ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..v });
            let (sv, o) = controls(&s).unwrap();
            let (mut lo, mut hi, mut fmax, mut n) = (f64::INFINITY, f64::NEG_INFINITY, 0.0f64, 0);
            for p in &o.points {
                let r = sv.read(p).unwrap();
                if let Some(g) = r.gap_nozzle {
                    lo = lo.min(g);
                    hi = hi.max(g);
                    fmax = fmax.max(r.thrust.abs());
                    n += 1;
                    assert!((r.gap_ram.unwrap() + g - (r.thrust - r.inferred.unwrap())).abs() <= 1e-12 * fmax.max(1.0));
                }
            }
            assert!(n > 100, "the split is shown at the choked points");
            assert!((hi - lo) <= 1e-12 * fmax, "nozzle part spread {:e} of the thrust", (hi - lo) / fmax);
        }
    }
}

#[test]
fn with_the_clean_table_and_a_generator_the_targets_are_rung96s_and_rung94s_root_is_not_run() {
    // Anchor § 1: the target is the generator-off engine's thrust at the set throttle (rung 96's `b0`/`b1`), whether the
    // table is made here or at the design flight. The final review found the first build running rung 94's thrust-hold
    // root there as well, refusing ends the design-flight table could reach (crash map #26, #141). Its targets were the
    // same values, so the equality below confirms only; the call to #26 at the end is what failed on that build.
    for gen in [GeneratorMode::Hp, GeneratorMode::Lp] {
        let s = hot(ControlsSettings { generator: gen, ..gov(ThrustBy::Epr) });
        let (d, f) = (build(&s).unwrap(), build(&ControlsSettings { table_at: TableAt::Flight, ..s }).unwrap());
        assert!(d.carry.is_some());
        assert_eq!(d.hold_start.unwrap().target.to_bits(), f.hold_start.unwrap().target.to_bits());
        assert_eq!(d.hold_end.unwrap().target.to_bits(), f.hold_end.unwrap().target.to_bits());
    }
    // Crash map #26 (seed 98, `scratch-archive/rung98/crash98.rs`): it ran before rung 98, and runs again.
    let r = r#"{"op":"controls","controls":{"T0":223.64999999999998,"p0":8989.634480886729,"shape":"flat-lp","lever":"none","bleed_mode":"valve","valve_speed":"instant","redline_on":0,"gov_lag_on":0,"accel_on":0,"floor_on":1,"floor_ref":"phi","floor_phi":0.68,"release_on":0,"govern":1,"hold":"thrust","thrust_by":"n1","speed_law":"prop","gov_tau":0.43,"gov_gain":24,"generator":"lp","generator_power":0.075,"customer":"off","customer_share":0.18,"customer_amount":0.18,"cool_vane":0,"cool_blade":0,"metal_on":0,"from":950,"to":1060,"ramp":1.6600000000000001,"settle":0.8,"rho":3.3000000000000003,"M0":1.62,"table_at":"design","corr_p":1,"corr_t":1}}"#;
    assert!(turbojet::sandbox::call(r).starts_with(r#"{"ok":1"#), "crash map #26 runs");
}

#[test]
fn a_flight_beyond_the_design_tables_reach_is_refused_in_words() {
    // Anchor G4: at Mach 3 at sea level the default move translates to ~360–500 K at the design flight.
    let e = build(&at(ControlsSettings { speed_corr: SpeedCorr::None, ..gov(ThrustBy::Epr) }, 288.15, 101_325.0, 3.0))
        .err().expect("refused");
    assert!(e.0.contains("made at the design flight") && e.0.contains("Make the tables at this flight"), "{}", e.0);
}

// =============================================================================================
// SCORED — anchor § 3 (subset; the full grid: scratch-archive/rung98/score.rs)
// =============================================================================================

#[test]
fn h1_uncorrected_for_temperature_the_fan_speed_control_overshoots_in_cold_air_and_falls_short_in_hot() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        let s = ControlsSettings { shape, corr_t: false, ..gov(ThrustBy::N1) };
        assert!(miss(&cold(s)).iter().all(|&x| x > 0.0));
        assert!(miss(&hot(s)).iter().all(|&x| x < 0.0));
    }
}

#[test]
fn h2_a_design_mach_table_gives_too_much_thrust_slower_and_too_little_faster() {
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        let s = ControlsSettings { shape: FLAT_LP, speed_corr: SpeedCorr::None, ..gov(by) };
        assert!(miss(&at(s, 250.0, 50_000.0, 0.5)).iter().all(|&x| x > 0.0));
        assert!(miss(&at(s, 250.0, 50_000.0, 1.4)).iter().all(|&x| x < 0.0));
    }
}

#[test]
fn h3_the_uncorrected_schedule_runs_cooler_in_hot_air_and_hotter_in_cold() {
    let peak = |s: &ControlsSettings| {
        let (sv, o) = controls(s).unwrap();
        assert!(o.points.iter().any(|p| sv.read(p).unwrap().holder == Holder::Accel), "the schedule holds the fuel");
        o.points.iter().map(|p| p.tt4).fold(f64::MIN, f64::max)
    };
    let s = ControlsSettings { shape: FLOW_PRESS, ..accel() };
    assert!(peak(&hot(ControlsSettings { corr_t: false, ..s })) < peak(&hot(s)));
    assert!(peak(&cold(ControlsSettings { corr_t: false, ..s })) > peak(&cold(s)));
}

#[test]
fn h4_miss_uncorrected_the_proportional_law_reads_off_its_table_and_settles_low_in_hot_air_too() {
    // H4 predicted HIGHER in hot air; it settles LOWER (4 MISS of 8): the rows lie where the CORRECTED reading falls,
    // so the uncorrected one runs past the top row, where rung 95's table holds its end value — too little fuel.
    let end = |s: &ControlsSettings| controls(s).unwrap().1.points.last().unwrap().nu_hp;
    let s = ControlsSettings { shape: FLOW_PRESS, ..prop() };
    assert!(end(&hot(ControlsSettings { corr_t: false, ..s })) < end(&hot(s)));
    assert!(end(&cold(ControlsSettings { corr_t: false, ..s })) < end(&cold(s)));
}
