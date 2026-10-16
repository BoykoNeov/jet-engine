//! RUNG 95 — A GOVERNOR IN THE SLAM. Anchor: `docs/plans/rung95-anchor-governor.md`; spec: `docs/rung95-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 2, "no credit") first: governing the burner temperature IS rung 40's
//! temperature-commanded march, the fixed point is the end, the hold identity, the step check, the metal leg. Then the
//! scored claims H1–H6, pinned at a subset of the spec's grid (the full grid is
//! `docs/plans/scratch-archive/rung95/score.rs`).

use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor::{self as gv, Clamp, Leg};
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ControlsSolver, GeneratorMode, SpeedLaw, DS, GAIN_MAX, TAU_MIN};

fn d() -> ControlsSettings { ControlsSettings::defaults() }
fn gov(s: ControlsSettings) -> ControlsSettings { ControlsSettings { govern: true, ..s } }
const FLOW_PRESS: usize = 0;
const FLAT_LP: usize = 3;
fn thrust_of(sv: &ControlsSolver, p: &FuelPoint) -> f64 { gv::thrust(&sv.reread(p).unwrap().base) }
fn peak(o: &[FuelPoint]) -> (usize, f64) {
    o.iter().enumerate().fold((0, f64::NEG_INFINITY), |m, (k, p)| if p.tt4 > m.1 { (k, p.tt4) } else { m })
}
fn low(o: &[FuelPoint], f: fn(&FuelPoint) -> f64) -> (usize, f64) {
    o.iter().enumerate().fold((0, f64::INFINITY), |m, (k, p)| if f(p) < m.1 { (k, f(p)) } else { m })
}

// =============================================================================================
// CONSTRUCTION
// =============================================================================================

#[test]
fn off_is_the_earlier_run_bit_for_bit() {
    // The governor's knobs move nothing while it is off.
    let (_, a) = controls(&d()).unwrap();
    let (_, b) = controls(&ControlsSettings { speed_law: SpeedLaw::Prop, gov_tau: 0.3, gov_gain: 7.0, hold: Hold::Thrust, ..d() }).unwrap();
    assert!(b.marks.is_empty());
    assert_eq!(a.points.len(), b.points.len());
    for (p, q) in a.points.iter().zip(&b.points) {
        assert_eq!([p.nu_lp, p.nu_hp, p.mf, p.tt4].map(f64::to_bits), [q.nu_lp, q.nu_hp, q.mf, q.tt4].map(f64::to_bits));
    }
}

#[test]
fn governing_the_burner_temperature_is_rung40s_temperature_commanded_march() {
    // Two code paths through the same machine: a fuel root through the fuel closure vs. the temperature closure.
    for shape in 0..4 {
        let s = gov(ControlsSettings { shape, ..d() });
        let sv = build(&s).unwrap();
        let pts = sv.march();
        let g = sv.governor.as_ref().unwrap();
        let tp = sv.core.fuel.inner.integrate(&sv.flight, |x: f64| g.demand(x), (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), DS);
        assert_eq!(pts.len(), tp.len());
        for (a, b) in pts.iter().zip(&tp) {
            assert!((a.nu_lp - b.nu_lp).abs() <= 1e-8 && (a.nu_hp - b.nu_hp).abs() <= 1e-8, "shape {shape} at s={}", a.s);
        }
    }
}

#[test]
fn the_governed_run_settles_on_the_end_and_holds_its_demand() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        for s in [ControlsSettings { hold: Hold::Thrust, ..d() },
                  ControlsSettings { hold: Hold::Thrust, generator: GeneratorMode::Hp, ..d() },
                  ControlsSettings { hold: Hold::HpSpeed, ..d() },
                  ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, ..d() }] {
            let s = gov(ControlsSettings { shape, settle: 8.0, ..s });
            let (sv, o) = controls(&s).unwrap();
            assert_eq!(o.points.len(), s.expected_points());
            let l = o.points.last().unwrap();
            assert!((l.nu_lp - sv.end.nu_lp).abs() < 1e-6 && (l.nu_hp - sv.end.nu_hp).abs() < 1e-6, "{s:?}");
            let fe = gv::thrust(&sv.end);
            assert!(((thrust_of(&sv, l) - fe) / fe).abs() < 1e-6, "{s:?}");
            // the hold identity, on the root with no leg cutting
            let g = sv.governor.as_ref().unwrap();
            for (p, m) in o.points.iter().zip(&o.marks) {
                assert_eq!((m.clamp, m.leg), (Clamp::Root, Leg::None));
                let i = sv.reread(p).unwrap().base;
                match g.law {
                    gv::Law::Thrust => assert!((gv::thrust(&i) / g.demand(p.s) - 1.0).abs() < 1e-12),
                    gv::Law::SpeedRate { tau } => assert!((i.phi_hp_dot - (g.demand(p.s) - p.nu_hp) / tau).abs() < 1e-12),
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn the_step_check_holds_at_the_gain_ceiling_and_the_fastest_rate() {
    // § 12.10's class: the march at the page's step against a step four times finer, flow coefficients to 1e-4.
    for s in [ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: GAIN_MAX, ..d() },
              ControlsSettings { hold: Hold::HpSpeed, gov_tau: TAU_MIN, ..d() },
              ControlsSettings { hold: Hold::Thrust, ..d() }] {
        let sv = build(&gov(s)).unwrap();
        let g = sv.governor.as_ref().unwrap();
        let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
        let (a, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), DS);
        let (b, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), DS / 4.0);
        let dm = a.iter().enumerate().map(|(k, p)| (p.phi_lp - b[4 * k].phi_lp).abs().max((p.phi_hp - b[4 * k].phi_hp).abs()))
            .fold(0.0, f64::max);
        assert!(dm < 1e-4, "{:?}: {dm:e}", g.law);
    }
}

#[test]
fn the_metal_leg_holds_the_limit_and_the_run_starts_settled() {
    // Governed thrust, no generator, cooled, 1985 metals: the burner floats far above the metal's limit on the move.
    let s = gov(ControlsSettings { hold: Hold::Thrust, metal_on: true, hold_cap: true, cool_vane: 0.04, cool_blade: 0.02, ..d() });
    let (sv, o) = controls(&s).unwrap();
    assert_eq!(o.points.len(), s.expected_points());
    let cut = o.marks.iter().filter(|m| m.leg == Leg::Metal).count();
    assert!(cut > 0, "the metal leg cuts somewhere");
    for (p, m) in o.points.iter().zip(&o.marks) {
        let over = sv.metal_over(&sv.reread(p).unwrap().base);
        if m.leg == Leg::Metal {
            assert!(over.abs() < 1e-6, "on the limit at s={}: {over}", p.s);
        } else {
            assert!(over <= 1e-6, "never over at s={}: {over}", p.s);
        }
    }
    // A capped start is an equilibrium the leg agrees with: the first point's fuel is the start's.
    let mf0 = sv.start.close.f * sv.start.close.mdot_air;
    assert!((o.points[0].mf / mf0 - 1.0).abs() < 1e-9);
}

#[test]
fn every_governed_step_reruns_bit_for_bit() {
    use turbojet::sandbox_controls::{replay_step, stop_reason, Route};
    for s in [ControlsSettings { hold: Hold::Thrust, redline_on: true, accel_on: true, ..d() },
              ControlsSettings { hold: Hold::HpSpeed, floor_on: true, ..d() },
              ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, from: 1400.0, to: 1000.0, ..d() }] {
        let (sv, o) = controls(&gov(s)).unwrap();
        assert_eq!(sv.route(), Route::Governor);
        for w in o.points.windows(2) {
            let (a, b, _) = replay_step(&sv, &w[0]).unwrap().unwrap();
            assert_eq!((a.to_bits(), b.to_bits()), (w[1].nu_lp.to_bits(), w[1].nu_hp.to_bits()));
        }
        assert!(stop_reason(&sv, &o.points).is_none());
    }
}

// =============================================================================================
// SCORED — H1–H6 (anchor § 3), all HIT; pinned on two maps
// =============================================================================================

#[test]
fn h1_the_thrust_governor_burns_more_on_an_acceleration_and_less_on_a_slowdown() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        for (from, to, more) in [(1000.0, 1400.0, true), (1400.0, 1000.0, false)] {
            let s = ControlsSettings { shape, from, to, hold: Hold::Thrust, ..d() };
            let (_, op) = controls(&s).unwrap();
            let (_, gp) = controls(&gov(s)).unwrap();
            for k in 1..=25 {
                assert_eq!(gp.points[k].mf > op.points[k].mf, more, "shape {shape} {from}->{to} at #{k}");
            }
        }
    }
}

#[test]
fn h2_h3_the_governed_acceleration_is_hotter_and_nearer_stall_hottest_at_the_ramps_end() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        let s = ControlsSettings { shape, hold: Hold::Thrust, ..d() };
        let (_, op) = controls(&s).unwrap();
        let (_, gp) = controls(&gov(s)).unwrap();
        assert!(peak(&gp.points).1 > peak(&op.points).1);
        for f in [(|p: &FuelPoint| p.phi_lp) as fn(&FuelPoint) -> f64, |p: &FuelPoint| p.phi_hp] {
            let (g, o) = (low(&gp.points, f), low(&op.points, f));
            assert!(g.0 > 0 && o.0 > 0, "interior minima (else VACUOUS)");
            assert!(g.1 < o.1);
        }
        assert_eq!(peak(&gp.points).0, 25, "the ramp's end");
    }
}

#[test]
fn h4_a_fixed_gain_closes_the_speed_gap_slower_at_idle() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        let tau = |from: f64, to: f64| {
            let s = gov(ControlsSettings { shape, from, to, ramp: 0.02, settle: 3.0, hold: Hold::HpSpeed,
                                           speed_law: SpeedLaw::Prop, gov_gain: 20.0, ..d() });
            let sv = build(&s).unwrap();
            let g = sv.governor.as_ref().unwrap();
            let ds = 0.005;
            let (p, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), ds);
            let e: Vec<f64> = p.iter().map(|q| g.hi - q.nu_hp).collect();
            let k0 = 4;
            let k = (k0 + 1..e.len()).find(|&k| e[k] <= e[k0] / std::f64::consts::E).unwrap();
            let (l0, l1, x) = (e[k - 1].ln(), e[k].ln(), e[k0].ln() - 1.0);
            p[k - 1].s + (x - l0) / (l1 - l0) * ds - p[k0].s
        };
        let r = tau(990.0, 1000.0) / tau(1390.0, 1400.0);
        assert!(r > 1.0, "shape {shape}: {r}");
        assert!((r - 1.90).abs() < 0.02, "the measured ratio (the spec's): {r}");
    }
}

#[test]
fn h5_with_the_redline_the_governor_still_gets_the_thrust_sooner() {
    let at95 = |sv: &ControlsSolver, o: &[FuelPoint]| {
        let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
        let x = f0 + 0.95 * (f1 - f0);
        let y: Vec<f64> = o.iter().map(|p| thrust_of(sv, p)).collect();
        (1..y.len()).find(|&k| y[k - 1] < x && y[k] >= x)
            .map(|k| o[k - 1].s + (x - y[k - 1]) / (y[k] - y[k - 1]) * (o[k].s - o[k - 1].s)).expect("reaches 95 %")
    };
    for shape in [FLOW_PRESS, FLAT_LP] {
        let s = ControlsSettings { shape, hold: Hold::Thrust, redline_on: true, ..d() };
        let (svo, oo) = controls(&s).unwrap();
        let (svg, og) = controls(&gov(s)).unwrap();
        assert!(at95(&svg, &og.points) < at95(&svo, &oo.points), "shape {shape}");
    }
}

#[test]
fn h6_the_commanded_rate_trades_time_for_temperature() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        let peaks: Vec<f64> = [0.05, 0.1, 0.2, 0.5].iter().map(|&t| {
            peak(&controls(&gov(ControlsSettings { shape, hold: Hold::HpSpeed, gov_tau: t, ..d() })).unwrap().1.points).1
        }).collect();
        assert!(peaks.windows(2).all(|w| w[1] < w[0]), "shape {shape}: {peaks:?}");
    }
}

#[test]
fn a_deep_slowdown_finds_a_runnable_fuel_to_start_the_governors_search_from() {
    // The crash map's case (`docs/rung95-spec.md`): 1670 -> 600 K in 0.42 τ, the bleed scheduled. Mid-run the ramp's fuel
    // is too lean for the shafts still spinning fast and the start's too rich; the fallback scan finds one between, and
    // the run reaches its end with the governor on the lean edge for a while.
    let s = gov(ControlsSettings { hold: Hold::Thrust, from: 1670.0, to: 600.0, ramp: 0.42, rho: 0.7,
                                   lever: turbojet::sandbox_controls::LeverChoice::Bleed, ..d() });
    let (_, o) = controls(&s).unwrap();
    assert_eq!(o.points.len(), s.expected_points(), "{:?}", o.stop);
    assert!(o.marks.iter().any(|m| m.clamp == Clamp::Lean));
}
