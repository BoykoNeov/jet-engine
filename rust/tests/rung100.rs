//! RUNG 100 — BELOW CHOKE. Anchor: `docs/plans/rung100-anchor-unchoked-formula.md`; spec: `docs/rung100-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 2, "no credit") first: the reduce settings, the slow slices at their own flights,
//! the push gate (the plant's thrust against the formula's push, choked and not), B against rung 99 where choked, B's
//! rebalance at an unchoked end, the refusal in words. Then the scored claims, pinned at a subset of the anchor's
//! cells (the full set: `docs/plans/scratch-archive/rung100/score.rs`), the MISSES pinned as found.

use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::TableAt;
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{face, Reading};
use turbojet::mach_axis::{self as ma, SpeedCorr, Tables};
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ThrustBy};
use turbojet::unchoked_formula::{Formula, Push};

const FLOW_PRESS: usize = 0;
const FLAT_LP: usize = 3;

fn gov(by: ThrustBy) -> ControlsSettings {
    ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, ..ControlsSettings::defaults() }
}
fn air(s: ControlsSettings, m: f64) -> ControlsSettings { ControlsSettings { t0: 250.0, p0: 50_000.0, m0: m, ..s } }
fn formula(s: ControlsSettings) -> ControlsSettings { ControlsSettings { speed_corr: SpeedCorr::Formula, ..s } }
fn r99(s: ControlsSettings) -> ControlsSettings { ControlsSettings { low_grid: false, formula_choked: true, ..s } }
fn bits(p: &[FuelPoint]) -> Vec<[u64; 4]> { p.iter().map(|q| [q.nu_lp, q.nu_hp, q.mf, q.tt4].map(f64::to_bits)).collect() }
fn run_bits(s: &ControlsSettings) -> Vec<[u64; 4]> { bits(&controls(s).unwrap_or_else(|e| panic!("{}", e.0)).1.points) }
/// The calibrated starting end's (achieved − target)/target.
fn start_miss(s: &ControlsSettings) -> f64 {
    let a = build(s).unwrap_or_else(|e| panic!("{}", e.0)).hold_start.unwrap();
    (a.achieved - a.target) / a.target
}

// =============================================================================================
// CONSTRUCTION
// =============================================================================================

#[test]
fn the_grid_gains_two_slow_slices_and_keeps_rung_99s_eight() {
    assert_eq!(&ma::GRID[2..], &ma::GRID99[..]);
    assert_eq!(&ma::GRID[..2], &[0.05, 0.15]);
    assert!(ControlsSettings::defaults().low_grid && !ControlsSettings::defaults().formula_choked);
    // A flight at Mach 0.1 reads the two slow slices; at 0.5 the same two slices as rung 99.
    let (a, b, w) = ma::bracket(&ma::GRID, 0.1);
    assert!(a == 0 && b == Some(1) && (w - 0.5).abs() < 1e-15);
    let (a, b, _) = ma::bracket(&ma::GRID, 0.5);
    assert!(ma::GRID[a] == 0.3 && ma::GRID[b.unwrap()] == 0.6);
}

#[test]
fn at_mach_0_3_and_above_the_grid_is_rung_99s_bit_for_bit() {
    for m in [0.3, 0.5, 1.4] {
        for by in [ThrustBy::Epr, ThrustBy::N1] {
            let s = air(ControlsSettings { shape: FLOW_PRESS, accel_on: true, ..gov(by) }, m);
            assert_eq!(run_bits(&s), run_bits(&ControlsSettings { low_grid: false, ..s }), "Mach {m} {by:?}");
        }
    }
}

#[test]
fn the_new_settings_are_inert_where_nothing_is_carried_or_nothing_is_read() {
    for s in [gov(ThrustBy::Epr), ControlsSettings { table_at: TableAt::Flight, ..air(gov(ThrustBy::N1), 0.1) },
              air(ControlsSettings::defaults(), 0.1), ControlsSettings { speed_corr: SpeedCorr::None, ..air(gov(ThrustBy::N1), 0.1) }] {
        assert_eq!(run_bits(&s), run_bits(&r99(s)));
    }
}

#[test]
fn at_a_slow_slices_own_flight_the_grid_is_a_table_made_there_bit_for_bit() {
    for m in [0.05, 0.15] {
        for by in [ThrustBy::Epr, ThrustBy::N1] {
            let s = air(ControlsSettings { from: 1000.0, ..gov(by) }, m);
            let sv = build(&s).unwrap_or_else(|e| panic!("Mach {m}: {}", e.0));
            assert!(sv.tables.is_none() && sv.carry.is_none(), "nothing carried at a slice's own flight");
            assert_eq!(run_bits(&s), run_bits(&ControlsSettings { table_at: TableAt::Flight, ..s }));
        }
    }
}

#[test]
fn the_push_gate_the_formula_reads_the_jet_exactly_choked_or_not() {
    // Anchor § 2 / G2: on pressure-ratio formula runs, F_true − F̂ − V0·(ṁ̂ − ṁ_face) ≤ 1e-9 N per kg/s at every point;
    // the Mach-0.05 run from 700 K starts unchoked.
    for (m, from) in [(0.05, 700.0), (1.4, 1000.0)] {
        for shape in [FLOW_PRESS, FLAT_LP] {
            let s = formula(air(ControlsSettings { shape, from, ..gov(ThrustBy::Epr) }, m));
            let (sv, o) = controls(&s).unwrap();
            let Some(Tables::Formula { choked: false, .. }) = sv.tables else { panic!("rung 100's formula") };
            let (mut n, mut unchoked, mut worst) = (0, 0, 0.0f64);
            for p in &o.points {
                let r = sv.read(p).unwrap();
                let (guess, rest) = (r.gap_ram.unwrap(), r.gap_nozzle.unwrap());
                worst = worst.max(rest.abs());
                assert!((guess + rest - (r.thrust - r.inferred.unwrap())).abs() <= 1e-9);
                unchoked += (p.branch != Branch::Choked) as usize;
                n += 1;
            }
            assert!(worst <= 1e-9, "Mach {m}: rest {worst:e} N per kg/s");
            assert!(n > 100 && (m > 1.0 || unchoked > 0), "Mach {m}: {unchoked} unchoked of {n}");
        }
    }
}

#[test]
fn the_push_is_one_closed_form_on_both_sides_of_choke() {
    // Continuous at the choke: just above and just below the critical nozzle pressure the two branches agree.
    let s = gov(ThrustBy::Epr);
    let sv = build(&s).unwrap();
    let push = Push::new(&sv.core.fuel, 50_000.0);
    let g = push.gamma;
    let crit = 50_000.0 / (push.pi_n * (2.0 / (g + 1.0)).powf(g / (g - 1.0)));
    let (lo, hi) = (push.at(crit * (1.0 - 1e-9)), push.at(crit * (1.0 + 1e-9)));
    assert!(!push.choked(crit * (1.0 - 1e-9)) && push.choked(crit * (1.0 + 1e-9)));
    assert!((hi - lo).abs() <= 1e-6 * hi.abs(), "{lo} vs {hi}");
}

#[test]
fn the_fan_formula_is_rung_99s_where_choked() {
    // Anchor § 2 / G9: on a choked fan-speed formula run (Mach 1.4, default move), F̂ at every point against rung 99's
    // column read at the same fan speed ≤ 1e-12 x the thrust.
    let s = formula(air(ControlsSettings { shape: FLOW_PRESS, ..gov(ThrustBy::N1) }, 1.4));
    let (sv, o) = controls(&s).unwrap();
    let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
    assert!(matches!(t.formula, Some(Formula::Fan { .. })));
    let mut col = t.clone();
    col.formula = None;
    for p in &o.points {
        assert_eq!(p.branch, Branch::Choked);
        let r = sv.read(p).unwrap();
        let v = r.reading.unwrap();
        let (a, b) = (t.try_inferred(&sv.core.fuel, Reading::LpSpeed, v).unwrap(), col.inferred(Reading::LpSpeed, v));
        assert!((a - b).abs() <= 1e-12 * r.thrust.abs(), "{a} vs {b}");
    }
}

#[test]
fn the_fan_formula_rebalances_the_lp_shaft_at_an_unchoked_end() {
    // Anchor § 2: at the calibrated start (Mach 0.05, 700 K, unchoked) B's root holds the end's fan speed, and its LP
    // turbine, re-solved against the flight's air, expands LESS than the row's choked one; the row it settles on is
    // hotter than the table's own row for that fan speed.
    let s = formula(air(ControlsSettings { shape: FLOW_PRESS, from: 700.0, ..gov(ThrustBy::N1) }, 0.05));
    let sv = build(&s).unwrap();
    assert_ne!(sv.start.branch, Branch::Choked);
    let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
    let Some(f @ Formula::Fan { rows, .. }) = t.formula.as_ref() else { panic!("the fan formula") };
    let nu = sv.start.nu_lp;
    let (x, at) = f.try_fan_root(&sv.core.fuel, t, nu).unwrap();
    assert!((at.nu - nu).abs() <= 1e-9 * nu);
    // Linear in the rows, the end segments carried on (the calibrated start lies just below the lowest row).
    let seg = |v: f64, key: fn(&turbojet::unchoked_formula::GasGen) -> f64| {
        let k = rows.windows(2).position(|w| v <= key(&w[1])).unwrap_or(rows.len() - 2);
        (&rows[k], &rows[k + 1])
    };
    let (a, b) = seg(nu, |r| r.nu);
    let x_table = a.x + (b.x - a.x) * (nu - a.nu) / (b.nu - a.nu);
    assert!(x > x_table + 1.0, "B's row {x} vs the table's {x_table}");
    let (base, gas) = (&sv.core.fuel.inner.inner.base, sv.core.fuel.inner.gas());
    let g = |y: fn(&turbojet::unchoked_formula::GasGen) -> f64| {
        let (a, b) = seg(x, |r| r.x);
        y(a) + (y(b) - y(a)) * (x - a.x) / (b.x - a.x)
    };
    let pin = base.try_solve_choked_turbine(gas, g(|r| r.tt45), g(|r| r.f45), base.a45, base.a8, base.pi_n, g(|r| r.eta)).unwrap();
    assert!(at.pi_lpt > pin.0 + 1e-3, "re-solved {} vs choked {}", at.pi_lpt, pin.0);
}

#[test]
fn a_formula_that_cannot_cover_the_throttle_is_refused_in_words() {
    // Anchor § 1 / G4: deep below choke at sea level the table's engine, pushing against the flight's air, has no
    // working state.
    // Sea level, Mach 0.1, from 600 K: the starting thrust lies below the table, where its engine has no working state.
    let s = formula(ControlsSettings { shape: FLOW_PRESS, from: 600.0, t0: 288.15, p0: 101_325.0, m0: 0.1, ..gov(ThrustBy::N1) });
    match build(&s) {
        Err(e) => assert!(e.0.contains("fan-speed formula") && e.0.contains("no working state") && e.0.contains("make the tables at this flight"), "{}", e.0),
        Ok(sv) => panic!("ran, start miss {:?}", sv.hold_start),
    }
}

// =============================================================================================
// SCORED — anchor § 3: 242 HIT, 35 MISS, 40 VACUOUS of 317 (subset; the full set: scratch-archive/rung100/score.rs)
// =============================================================================================

#[test]
fn h1_the_slow_slices_hold_the_grid_under_one_percent() {
    let a3 = turbojet::atmosphere::Ambient::from_altitude(3000.0, 0.0);
    for (t0, p0, m0) in [(318.15, 101_325.0, 0.08), (a3.t0, a3.p0, 0.22)] {
        for by in [ThrustBy::Epr, ThrustBy::N1] {
            let s = ControlsSettings { shape: FLOW_PRESS, from: 800.0, t0, p0, m0, ..gov(by) };
            let (n, o) = (start_miss(&s), start_miss(&ControlsSettings { low_grid: false, ..s }));
            assert!(n.abs() < 0.01 && (o.abs() <= 0.003 || n.abs() < o.abs()), "{t0} M{m0} {by:?}: {n} vs rung 99 {o}");
        }
    }
}

#[test]
fn h2_h3_the_pressure_ratio_formula_leaves_the_airflow_guess_over_and_smaller() {
    for m in [0.05, 0.15] {
        for shape in [FLOW_PRESS, FLAT_LP] {
            let s = formula(air(ControlsSettings { shape, from: 700.0, ..gov(ThrustBy::Epr) }, m));
            let (n, o) = (start_miss(&s), start_miss(&r99(s)));
            assert!(n > 0.0 && n < 0.003 && n.abs() < o.abs(), "Mach {m}: {n} vs rung 99 {o}");
            // The miss IS the airflow guess at the end's own reading.
            let sv = build(&s).unwrap();
            let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
            let i = &sv.start;
            let e = t.read(Reading::Epr, i);
            let guess = t.v0 * (t.face_at_epr(e) - face(i));
            let f_hat = t.try_inferred(&sv.core.fuel, Reading::Epr, e).unwrap();
            assert!((thrust(i) - f_hat - guess).abs() <= 1e-9);
        }
    }
}

#[test]
fn h4_miss_at_mach_1_4_the_exact_push_loses_a_cancelling_error() {
    // The finding (scored MISS, 3 of 3): from 700 K at Mach 1.4 the reading lies below the design table, whose lowest
    // row is itself unchoked; rung 99's push and airflow came from that one row and cancelled — the exact push keeps
    // only the airflow guess, about −1.7 %.
    let s = formula(air(ControlsSettings { shape: FLOW_PRESS, from: 700.0, ..gov(ThrustBy::Epr) }, 1.4));
    let (n, o) = (start_miss(&s), start_miss(&r99(s)));
    assert!(n < -0.01 && o.abs() < 0.002, "{n} vs rung 99 {o}");
}

#[test]
fn h5_h6_the_fan_formula_overshoots_and_removes_most_of_the_error() {
    for m in [0.05, 0.15] {
        for shape in [FLOW_PRESS, FLAT_LP] {
            let s = formula(air(ControlsSettings { shape, from: 700.0, ..gov(ThrustBy::N1) }, m));
            let (n, o) = (start_miss(&s), start_miss(&r99(s)));
            let q = n.abs() / o.abs();
            assert!(n < 0.0 && (0.15..=0.6).contains(&q), "Mach {m}: {n} vs rung 99 {o} (ratio {q})");
        }
    }
}
