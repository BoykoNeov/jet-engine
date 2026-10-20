//! RUNG 99 — THE SPEED AXIS. Anchor: `docs/plans/rung99-anchor-mach-axis.md`; spec: `docs/rung99-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 2, "no credit") first: the grid at Mach 0.85 is rung 98 bit for bit, at a slice's
//! own flight it is tables made there; the pressure correction reproduces each slice built at a pressure-only flight;
//! the blend is the blend of the two slices' answers; the formula leaves only the airflow guess while choked; the
//! out-of-reach refusal. Then the scored claims H1–H6, pinned at a subset of the anchor's grid (the full grid is
//! `docs/plans/scratch-archive/rung99/score.rs`).

use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::{self as ct, TableAt};
use turbojet::fuel_transient::FuelPoint;
use turbojet::inferred_thrust::Reading;
use turbojet::mach_axis::{self as ma, SpeedCorr, Tables};
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, CustomerMode, Holder, LeverChoice, SpeedLaw, ThrustBy};
use turbojet::two_spool::IdleRule;

const FLOW_PRESS: usize = 0;
const FLAT_LP: usize = 3;

/// Rung 99 as it shipped: rung 100's two reduce settings (its grid without the slow slices, its formula column), and
/// the below-idle rule at `Old` (sandbox plan §§ 38–39: its anchored cells include engines below idle — the Mach-3 slice
/// from 1000 K among them — read as they were before the page's rule).
fn r99() -> ControlsSettings {
    ControlsSettings { low_grid: false, formula_choked: true, idle: IdleRule::Old, ..ControlsSettings::defaults() }
}
fn gov(by: ThrustBy) -> ControlsSettings {
    ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, ..r99() }
}
fn accel() -> ControlsSettings { ControlsSettings { accel_on: true, ..r99() } }
fn prop() -> ControlsSettings {
    ControlsSettings { govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, ..r99() }
}
fn at(s: ControlsSettings, t0: f64, p0: f64, m0: f64) -> ControlsSettings { ControlsSettings { t0, p0, m0, ..s } }
/// The design altitude's air at Mach `m`.
fn air(s: ControlsSettings, m: f64) -> ControlsSettings { at(s, 250.0, 50_000.0, m) }
fn none(s: ControlsSettings) -> ControlsSettings { ControlsSettings { speed_corr: SpeedCorr::None, ..s } }
fn formula(s: ControlsSettings) -> ControlsSettings { ControlsSettings { speed_corr: SpeedCorr::Formula, ..s } }
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
fn the_grid_is_on_by_default_and_its_design_slice_is_the_design_flight() {
    let d = ControlsSettings::defaults();
    assert_eq!(d.speed_corr, SpeedCorr::Grid);
    let fd = d.design_flight();
    assert!(ma::GRID99.contains(&ma::DESIGN_MACH) && ma::GRID.contains(&ma::DESIGN_MACH) && ma::DESIGN_MACH == fd.m0);
    let s = ma::slice_flight(&fd, ma::DESIGN_MACH);
    assert!(s.t0 == fd.t0 && s.p0 == fd.p0 && s.m0 == fd.m0);
}

#[test]
fn a_flight_reads_the_two_slices_around_it_and_the_end_pair_outside() {
    assert_eq!(ma::bracket(&ma::GRID99, 1.2), (3, None, 0.0));
    let (a, b, w) = ma::bracket(&ma::GRID99, 0.5);
    assert!(a == 0 && b == Some(1) && (w - 2.0 / 3.0).abs() < 1e-15);
    let (a, b, w) = ma::bracket(&ma::GRID99, 0.15);
    assert!(a == 0 && b == Some(1) && (w + 0.5).abs() < 1e-15, "below the grid: the 0.3/0.6 line carried on");
    let (a, b, w) = ma::bracket(&ma::GRID99, 3.2);
    assert!(a == 6 && b == Some(7) && (w - 1.4).abs() < 1e-12, "above the grid: the 2.5/3.0 line carried on");
}

#[test]
fn at_mach_0_85_in_any_air_the_grid_is_rung98_bit_for_bit() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        for base in [gov(ThrustBy::Epr), gov(ThrustBy::N1), accel(), prop()] {
            for (t0, p0) in [(290.0, 50_000.0), (216.65, 50_000.0), (250.0, 26_500.0)] {
                let s = at(ControlsSettings { shape, ..base }, t0, p0, 0.85);
                assert!(matches!(build(&s).unwrap().tables, Some(Tables::Design(_))));
                assert_eq!(run_bits(&s), run_bits(&none(s)));
            }
        }
    }
}

#[test]
fn at_a_slices_own_flight_the_grid_is_a_table_made_there_bit_for_bit() {
    for j in [0, 1, 3, 4, 5, 6, 7] {
        let m = ma::GRID99[j];
        // The proportional law at gain 5: rammed-dense air lowers the page's step-checked gain limit (plan § 27).
        for base in [gov(ThrustBy::Epr), accel(), ControlsSettings { gov_gain: 5.0, ..prop() }] {
            let s = air(base, m);
            let sv = build(&s).unwrap_or_else(|e| panic!("Mach {m}: {}", e.0));
            assert!(sv.tables.is_none() && sv.carry.is_none(), "nothing carried at a slice's own flight");
            assert_eq!(run_bits(&s), run_bits(&ControlsSettings { table_at: TableAt::Flight, ..s }));
        }
    }
}

#[test]
fn speed_corr_is_inert_where_nothing_is_carried() {
    // At the design flight, with tables made at this flight, or with no table read.
    let d = ControlsSettings::defaults();
    for c in SpeedCorr::ALL {
        for s in [gov(ThrustBy::Epr), ControlsSettings { table_at: TableAt::Flight, ..air(gov(ThrustBy::Epr), 1.4) },
                  air(d, 1.4)] {
            let a = ControlsSettings { speed_corr: c, ..s };
            assert!(build(&a).unwrap().tables.is_none());
            assert_eq!(run_bits(&a), run_bits(&none(s)));
        }
    }
}

#[test]
fn the_pressure_correction_reproduces_each_slice_built_at_a_pressure_only_flight() {
    // At each slice's Mach in 250 K air at 35 kPa (theta_j = 1 exactly), the carried slice against the tables rungs
    // 96/48/95 build AT the flight by their own solves. Bar 1e-9 (anchor § 2, rung 98's).
    let mut worst = 0.0f64;
    for &m in &ma::GRID99 {
        for v in [r99(), ControlsSettings { customer: CustomerMode::Share, ..r99() },
                  ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..r99() },
                  ControlsSettings { lever: LeverChoice::Bleed, ..r99() }] {
            for base in [gov(ThrustBy::Epr), ControlsSettings { accel_on: true, gov_gain: 5.0, ..prop() }] {
                let s = at(ControlsSettings { govern: base.govern, hold: base.hold, thrust_by: base.thrust_by,
                                              speed_law: base.speed_law, accel_on: base.accel_on, gov_gain: base.gov_gain, ..v },
                           250.0, 35_000.0, m);
                let (d, f) = (build(&s).unwrap(), build(&ControlsSettings { table_at: TableAt::Flight, ..s }).unwrap());
                match d.tables.expect("carried") {
                    Tables::Design(c) => assert!(m == ma::DESIGN_MACH && c.theta == 1.0),
                    Tables::Grid { a, b, .. } => assert!(a.m == m && b.is_none() && a.carry.theta == 1.0 && a.carry.k == 1.0),
                    Tables::Formula { .. } => unreachable!(),
                }
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
    assert!(worst <= 1e-9, "carried slice off the flight's own table by {worst:e}");
}

#[test]
fn the_blend_is_the_two_slices_answers_blended_at_equal_reading() {
    // Anchor § 2: inside both slices' rows, the blended table's value = (1 − w)·(slice a) + w·(slice b), every column,
    // all three tables. Bar 1e-12.
    for m in [0.5, 1.4, 0.15] {
        let s = air(ControlsSettings { accel_on: true, ..gov(ThrustBy::Epr) }, m);
        let sv = build(&s).unwrap();
        let Some(Tables::Grid { a, b: Some(b), w }) = sv.tables else { panic!("two slices at Mach {m}") };
        let ft = &sv.core.fuel;
        let (lo, hi) = (s.from, s.to);
        for r in [Reading::Epr, Reading::LpSpeed] {
            let one = |c: &ct::Carry| c.thrust_table(&ct::try_design_table(ft, c, lo, hi).unwrap());
            let (ta, tb) = (one(&a.carry), one(&b.carry));
            let t = ma::blend_thrust(&ta, &tb, w, r);
            let x = |row: &turbojet::inferred_thrust::Row| if r == Reading::Epr { row.epr } else { row.nu_lp };
            let (x0, x1) = (x(&ta.rows[0]).max(x(&tb.rows[0])), x(ta.rows.last().unwrap()).min(x(tb.rows.last().unwrap())));
            for k in 0..=40 {
                let v = x0 + (x1 - x0) * k as f64 / 40.0;
                let want = (1.0 - w) * ta.inferred(r, v) + w * tb.inferred(r, v);
                assert!(rel(t.inferred(r, v), want) <= 1e-12, "Mach {m} {r:?} at {v}");
            }
        }
        // The steady table and the schedule, on real slices: end values held, as their readers hold them.
        let steady = |c: &ct::Carry| c.steady_table(&ct::try_design_steady_table(ft, c, lo, hi).unwrap());
        let sched = |c: &ct::Carry| {
            let t = c.accel(&ct::try_design_accel(ft, c, lo, hi, 13, 0.25).unwrap());
            t.n_h.iter().copied().zip(t.kappa.iter().copied()).collect::<Vec<_>>()
        };
        for (pa, pb) in [(steady(&a.carry), steady(&b.carry)), (sched(&a.carry), sched(&b.carry))] {
            let bl = ma::blend_pairs(&pa, &pb, w);
            let (x0, x1) = (pa[0].0.max(pb[0].0), pa.last().unwrap().0.min(pb.last().unwrap().0));
            for k in 0..=40 {
                let v = x0 + (x1 - x0) * k as f64 / 40.0;
                let want = (1.0 - w) * ma::held_lin(&pa, v) + w * ma::held_lin(&pb, v);
                assert!(rel(ma::held_lin(&bl, v), want) <= 1e-12, "Mach {m} at {v}");
            }
        }
    }
}

#[test]
fn the_run_reads_all_three_tables_blended_from_the_two_slices_around_it() {
    // The final review: H6 passes whether or not the schedule and the steady table are blended (it would pass on rung
    // 98's table too). At Mach 0.5, rebuild the Mach-0.3 and Mach-0.6 slices here, from flights built by hand, and the
    // run's OWN three tables must be their blend, bit for bit — and differ from rung 98's.
    let fl = |m: f64| turbojet::engine::FlightCondition::new(250.0, 50_000.0, m);
    let w = (0.5 - 0.3) / (0.6 - 0.3);
    let check = |s: ControlsSettings| {
        let sv = build(&s).unwrap();
        let ft = &sv.core.fuel;
        let (a, b) = (ct::Carry::new(ft, &fl(0.3), &fl(0.5), true, true), ct::Carry::new(ft, &fl(0.6), &fl(0.5), true, true));
        let g = sv.governor.as_ref();
        if let Some(t) = g.and_then(|g| g.infer.as_ref()) {
            let one = |c: &ct::Carry| c.thrust_table(&ct::try_design_table(ft, c, s.from, s.to).unwrap());
            assert_eq!(*t, ma::blend_thrust(&one(&a), &one(&b), w, Reading::Epr), "the thrust table");
        }
        if let Some(g) = g.filter(|g| !g.table.is_empty()) {
            let one = |c: &ct::Carry| c.steady_table(&ct::try_design_steady_table(ft, c, sv.start.tt4, sv.end.tt4).unwrap());
            assert_eq!(g.table, ma::blend_pairs(&one(&a), &one(&b), w), "the steady table");
        }
        let acc = sv.accel.as_ref().unwrap();
        let one = |c: &ct::Carry| {
            let t = c.accel(&ct::try_design_accel(ft, c, s.from, s.to, turbojet::sandbox_controls::ACCEL_ROWS, s.accel_margin).unwrap());
            t.n_h.iter().copied().zip(t.kappa.iter().copied()).collect::<Vec<_>>()
        };
        let want = ma::blend_pairs(&one(&a), &one(&b), w);
        assert_eq!(acc.n_h, want.iter().map(|p| p.0).collect::<Vec<_>>(), "the schedule's abscissa");
        assert_eq!(acc.kappa, want.iter().map(|p| p.1).collect::<Vec<_>>(), "the schedule");
        let rung98 = build(&none(s)).unwrap();
        assert_ne!(acc.kappa, rung98.accel.as_ref().unwrap().kappa, "not rung 98's schedule");
        sv
    };
    let e = check(air(ControlsSettings { shape: FLOW_PRESS, accel_on: true, ..gov(ThrustBy::Epr) }, 0.5));
    assert!(e.governor.as_ref().unwrap().infer.is_some());
    let p = check(air(ControlsSettings { shape: FLOW_PRESS, accel_on: true, ..prop() }, 0.5));
    assert!(!p.governor.as_ref().unwrap().table.is_empty());
}

#[test]
fn the_formula_leaves_only_the_airflow_guess_while_choked() {
    // Anchor § 2 / G3: on an EPR run under the formula at Mach 1.4 (every point and row choked), the rest of the gap
    // after the airflow guess is zero. Bar 1e-12 x the thrust.
    for shape in [FLOW_PRESS, FLAT_LP] {
        let s = formula(air(ControlsSettings { shape, ..gov(ThrustBy::Epr) }, 1.4));
        let (sv, o) = controls(&s).unwrap();
        assert!(matches!(sv.tables, Some(Tables::Formula { .. })));
        let mut n = 0;
        for p in &o.points {
            assert_eq!(p.branch, Branch::Choked);
            let r = sv.read(p).unwrap();
            let (guess, rest) = (r.gap_ram.unwrap(), r.gap_nozzle.unwrap());
            assert!(rest.abs() <= 1e-12 * r.thrust.abs(), "rest {rest:e} of thrust {}", r.thrust);
            assert!((guess + rest - (r.thrust - r.inferred.unwrap())).abs() <= 1e-12 * r.thrust.abs());
            n += 1;
        }
        assert!(n > 100);
    }
}

#[test]
fn a_flight_beyond_the_grids_reach_is_refused_in_words() {
    // Anchor § 2 / G1: Mach 3 at sea level reads the Mach-3 slice alone, whose row at ~868 K has no steady point.
    let e = build(&at(gov(ThrustBy::Epr), 288.15, 101_325.0, 3.0)).err().expect("refused");
    assert!(e.0.contains("made at Mach 3 in the design altitude's air") && e.0.contains("Make the tables at this flight"),
            "{}", e.0);
    // Sandbox plan §§ 38–39: anchored with the below-idle rule at `Old` (§ 37's); the live page (`Accept`) keeps the
    // below-idle 1000 K start there, and the tables reach it: the whole move runs.
    let s = at(ControlsSettings { idle: IdleRule::Accept, ..gov(ThrustBy::Epr) }, 288.15, 101_325.0, 3.0);
    let (sv, o) = controls(&s).expect("the live page runs it");
    assert!(sv.start.sp_thrust < 0.0 && o.points.len() == s.expected_points());
}

// =============================================================================================
// SCORED — anchor § 3: 116 HIT, 0 MISS (subset; the full grid: scratch-archive/rung99/score.rs)
// =============================================================================================

#[test]
fn h1_h2_the_grid_leaves_a_small_miss_with_the_sign_of_the_bend() {
    // Mach 0.5: the curve bends up, the chord over-states it, the control lands short. Mach 1.4's low end: the nozzle's
    // push bends it down, the control overshoots. Both under 1 % (rung 98: 4.6–9.7 %).
    for shape in [FLOW_PRESS, FLAT_LP] {
        for by in [ThrustBy::Epr, ThrustBy::N1] {
            let s = ControlsSettings { shape, ..gov(by) };
            let slow = miss(&air(s, 0.5));
            assert!(slow.iter().all(|&x| x < 0.0 && x > -0.01), "{by:?} Mach 0.5: {slow:?}");
            let fast = miss(&air(s, 1.4));
            assert!(fast[0] > 0.0 && fast.iter().all(|&x| x.abs() < 0.01), "{by:?} Mach 1.4: {fast:?}");
            assert!(miss(&none(air(s, 1.4)))[1] < -0.09, "rung 98's miss beside it");
        }
    }
}

#[test]
fn h3_extending_the_trend_below_the_grid_gives_too_much_thrust() {
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        let m = miss(&air(ControlsSettings { shape: FLOW_PRESS, ..gov(by) }, 0.15));
        assert!(m.iter().all(|&x| x > 0.0), "{by:?}: {m:?}");
    }
}

#[test]
fn h4_choked_the_formula_wins_where_the_bend_is_large_and_the_grid_where_it_cancels() {
    let s = ControlsSettings { shape: FLOW_PRESS, ..gov(ThrustBy::Epr) };
    for m in [0.5, 1.4] {
        let (f, g) = (miss(&formula(air(s, m))), miss(&air(s, m)));
        assert!(f[0].abs() < g[0].abs(), "Mach {m} low end: formula {f:?} grid {g:?}");
        if m == 0.5 { assert!(f[1].abs() < g[1].abs()) } else { assert!(g[1].abs() < f[1].abs(), "Mach 1.4 high end") }
    }
}

#[test]
fn h5_unchoked_the_formula_gives_too_little_thrust_on_the_pressure_ratio_and_too_much_on_the_fan_speed() {
    // The pressure ratio (scored): the choked nozzle's push over-states the unchoked one. The fan speed (reported, the
    // finding): the unchoked engine's fan speed at a given thrust moves first order, the other way — about +5 % on rung
    // 38's choked vane, about +2.7 % under rung 104's flow law (the LP shaft slows less below choke): the bar follows.
    let s = |by| formula(air(ControlsSettings { shape: FLOW_PRESS, from: 700.0, ..gov(by) }, 0.05));
    let sv = build(&s(ThrustBy::Epr)).unwrap();
    assert_ne!(sv.start.branch, Branch::Choked);
    let (a, b) = (miss(&s(ThrustBy::Epr))[0], miss(&s(ThrustBy::N1))[0]);
    assert!(a < -0.005 && b > 0.02, "EPR {a}, fan {b}");
}

#[test]
fn h6_on_the_schedule_and_the_hp_table_the_speed_axis_is_nearly_inert_while_choked() {
    for m in [0.5, 1.4] {
        let a = air(ControlsSettings { shape: FLOW_PRESS, ..accel() }, m);
        let peak = |s: &ControlsSettings| {
            let (sv, o) = controls(s).unwrap();
            assert!(o.points.iter().any(|p| sv.read(p).unwrap().holder == Holder::Accel), "the schedule holds the fuel");
            o.points.iter().map(|p| p.tt4).fold(f64::MIN, f64::max)
        };
        assert!((peak(&a) - peak(&none(a))).abs() < 10.0);
        let p = air(ControlsSettings { shape: FLOW_PRESS, ..prop() }, m);
        let end = |s: &ControlsSettings| controls(s).unwrap().1.points.last().unwrap().nu_hp;
        assert!(rel(end(&p), end(&none(p))) < 1e-3);
    }
}
