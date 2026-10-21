//! RUNG 106 — THE GOVERNOR BESIDE THE SLOW TEMPERATURE LIMITER. Anchor: `docs/plans/rung106-anchor-governor-lag.md`;
//! spec: `docs/rung106-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 2, "no credit") first: the open ramp as the demand IS rung 47's march; before the
//! clip engages the run IS rung 95's; `τ → 0` tends to rung 95's instant redline; the clip's ledger; the lean hold at
//! model level and the page's refusals; every step re-runs. Then the scored claims H1–H5, pinned at a subset of the
//! spec's grid.

use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::governor::{self as gv, Leg};
use turbojet::governor_lag::{self as gl, ClipLag, Demand};
use turbojet::airflow_source::AirflowFrom;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ControlsSolver, Route, SpeedLaw, ThrustBy, DS};

fn d() -> ControlsSettings { ControlsSettings::defaults() }
/// The governed slam beside the lagged redline (rung 47's default lag 0.2 unless set).
fn gl_on(s: ControlsSettings) -> ControlsSettings {
    ControlsSettings { govern: true, redline_on: true, gov_lag_on: true, ..s }
}
const FLOW_PRESS: usize = 0;
const FLAT_LP: usize = 3;
fn clip(p: &FuelPoint) -> (f64, f64) {
    match p.extra { PointExtra::Asym { g, required } => (g, required), _ => panic!("the clip's route records its clip") }
}
fn bits(p: &FuelPoint) -> [u64; 4] { [p.nu_lp, p.nu_hp, p.mf, p.tt4].map(f64::to_bits) }
/// The governed lagged march called directly (any lag, any step) on the machine the page builds for `s`.
fn direct(s: &ControlsSettings, tau: f64, ds: f64) -> Vec<FuelPoint> {
    let sv = build(&ControlsSettings { gov_lag_on: false, ..*s }).unwrap();
    let g = sv.governor.as_ref().unwrap();
    gl::march(&sv.core.fuel, &sv.flight, Demand::Governor(g), ClipLag { tt4_max: s.redline, tau }, &sv.lag_limiters(), None,
              (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), ds).0
}

// =============================================================================================
// CONSTRUCTION
// =============================================================================================

#[test]
fn the_open_ramp_as_the_demand_is_rung47s_march_bit_for_bit() {
    // Two code paths: rung 47's lagged route on the Controls view, and the new march with the ramp handed in.
    for shape in 0..4 {
        for tau in [0.02, 0.05, 0.2, 0.8] {
            let s = ControlsSettings { shape, redline_on: true, gov_lag_on: true, gov_lag: tau, ..d() };
            let sv = build(&s).unwrap();
            assert_eq!(sv.route(), Route::GovLag);
            let r47 = sv.march();
            let ramp = s.schedule(sv.mf_lo, sv.mf_hi);
            let (pts, marks) = gl::march(&sv.core.fuel, &sv.flight, Demand::Ramp(&ramp), ClipLag { tt4_max: s.redline, tau },
                                         &sv.lag_limiters(), None, (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), DS);
            assert_eq!(pts.len(), r47.len(), "shape {shape} tau {tau}");
            assert_eq!(pts.len(), s.expected_points());
            for (a, b) in pts.iter().zip(&r47) {
                assert_eq!(bits(a), bits(b), "shape {shape} tau {tau} at s={}", a.s);
            }
            assert!(marks.iter().any(|m| m.leg == Leg::Lag), "the clip engages");
        }
    }
}

#[test]
fn off_is_the_earlier_run_and_before_the_clip_engages_it_is_rung95s() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        for hold in [Hold::Thrust, Hold::HpSpeed] {
            let s = gl_on(ControlsSettings { shape, hold, ..d() });
            let (sv, o) = controls(&s).unwrap();
            assert_eq!(sv.route(), Route::GovernorLag);
            let (_, r95) = controls(&ControlsSettings { gov_lag_on: false, ..s }).unwrap();
            let first = o.points.iter().position(|p| clip(p).0 != 0.0).expect("the clip engages");
            assert!(first > 0);
            for (a, b) in o.points[..first].iter().zip(&r95.points) {
                assert_eq!(bits(a), bits(b), "{hold:?} shape {shape} at s={}", a.s);
            }
        }
    }
    // A redline above the governed peak: the clip never engages and the whole run is rung 95's (whose redline never cuts).
    let s = gl_on(ControlsSettings { hold: Hold::Thrust, redline: 2500.0, ..d() });
    let (_, o) = controls(&s).unwrap();
    let (_, r95) = controls(&ControlsSettings { gov_lag_on: false, ..s }).unwrap();
    assert_eq!(o.points.len(), r95.points.len());
    assert!(o.points.iter().all(|p| clip(p) == (0.0, 0.0)));
    for (a, b) in o.points.iter().zip(&r95.points) {
        assert_eq!(bits(a), bits(b));
    }
    // The burner-temperature hold below the redline never engages either.
    let (_, o) = controls(&gl_on(ControlsSettings { hold: Hold::BurnerTemp, ..d() })).unwrap();
    assert!(o.points.iter().all(|p| clip(p) == (0.0, 0.0)));
}

#[test]
fn a_vanishing_lag_tends_to_rung95s_instant_redline() {
    // τ 0.02, 0.01, 0.005 at a step τ/4: the gap to rung 95 with the instant redline, at the same step, shrinks.
    let s = gl_on(ControlsSettings { hold: Hold::Thrust, ..d() });
    let sv = build(&ControlsSettings { gov_lag_on: false, ..s }).unwrap();
    let g = sv.governor.as_ref().unwrap();
    let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
    let mut gaps = Vec::new();
    for tau in [0.02, 0.01, 0.005] {
        let ds = tau / 4.0;
        let (inst, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), ds);
        let (lag, _) = gl::march(&sv.core.fuel, &sv.flight, Demand::Governor(g), ClipLag { tt4_max: s.redline, tau },
                                 &sv.lag_limiters(), None, nu0, s.s_end(), ds);
        assert_eq!(inst.len(), lag.len());
        let gap = inst.iter().zip(&lag).map(|(a, b)| (a.nu_lp - b.nu_lp).abs().max((a.nu_hp - b.nu_hp).abs())).fold(0.0, f64::max);
        eprintln!("tau {tau}: max speed gap to the instant redline {gap:e}");
        gaps.push(gap);
    }
    assert!(gaps[1] < gaps[0] && gaps[2] < gaps[1], "{gaps:?}");
    eprintln!("orders: {:.2} {:.2}", (gaps[0] / gaps[1]).log2(), (gaps[1] / gaps[2]).log2());
}

/// `|trapz(r − g) − τ·(g_end − g_0)|` on recorded points, and `τ·max g`.
fn ledger(pts: &[FuelPoint], tau: f64) -> (f64, f64) {
    let e: Vec<(f64, f64)> = pts.iter().map(|p| { let (g, r) = clip(p); (p.s, r - g) }).collect();
    let integral: f64 = e.windows(2).map(|w| 0.5 * (w[1].0 - w[0].0) * (w[0].1 + w[1].1)).sum();
    let (g0, gn) = (clip(&pts[0]).0, clip(pts.last().unwrap()).0);
    let gmax = pts.iter().map(|p| clip(p).0).fold(0.0, f64::max);
    ((integral - tau * (gn - g0)).abs(), tau * gmax)
}

#[test]
fn the_clips_ledger_balances_to_the_frozen_bar() {
    // Anchor § 2 (D1): ∫(r − g) ds = τ·Δg — the fuel let through early is paid back later, measured against the instant
    // limiter at the same states. Bar frozen: 2e−2 of τ·max g on the recorded points.
    for shape in 0..4 {
        for tau in [0.05, 0.1, 0.2, 0.4, 0.8] {
            // the march itself: the page's floor (0.1) would refuse the anchor's 0.05
            let s = gl_on(ControlsSettings { shape, hold: Hold::Thrust, ..d() });
            let pts = direct(&s, tau, DS);
            assert_eq!(pts.len(), s.expected_points());
            let (miss, scale) = ledger(&pts, tau);
            eprintln!("shape {shape} tau {tau}: ledger miss {miss:e} of tau*max g {scale:e} ({:.1e})", miss / scale);
            assert!(miss <= 2e-2 * scale, "shape {shape} tau {tau}: {miss:e} > 2e-2 * {scale:e}");
        }
    }
    // The miss is the trapezoid's: it falls when the step is halved.
    let s = gl_on(ControlsSettings { hold: Hold::Thrust, ..d() });
    let (m2, _) = ledger(&direct(&s, 0.2, DS), 0.2);
    let (m1, _) = ledger(&direct(&s, 0.2, DS / 2.0), 0.2);
    eprintln!("ledger miss at step 0.02 {m2:e}, at 0.01 {m1:e}");
    assert!(m1 < m2);
}

#[test]
fn the_lean_hold_holds_at_the_wall_and_the_page_refuses_its_hot_start() {
    // One of anchor G4's hot starts, marched by calling the march directly (the page refuses it).
    let s = gl_on(ControlsSettings { hold: Hold::Thrust, from: 1700.0, to: 900.0, gov_lag: 0.2, ..d() });
    let err = build(&s).err().expect("the page refuses a start above the redline");
    assert!(err.0.contains("above the temperature limiter's"), "{}", err.0);
    let sv = build(&ControlsSettings { gov_lag_on: false, ..s }).unwrap();
    assert!(sv.start.tt4 > s.redline);
    let g = sv.governor.as_ref().unwrap();
    let ft = &sv.core.fuel;
    let (pts, marks) = gl::march(ft, &sv.flight, Demand::Governor(g), ClipLag { tt4_max: s.redline, tau: 0.2 },
                                 &sv.lag_limiters(), None, (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), DS);
    assert_eq!(pts.len(), s.expected_points(), "it runs to the end");
    let held: Vec<usize> = marks.iter().enumerate().filter(|(_, m)| m.leg == Leg::MinFuel).map(|(k, _)| k).collect();
    assert!(!held.is_empty(), "the minimum-fuel stop holds somewhere");
    for &k in &held {
        let p = &pts[k];
        // On the wall: it runs, and a fuel a hair leaner does not.
        assert!(ft.try_instant_fuel(&sv.flight, p.nu_lp, p.nu_hp, p.mf).is_ok());
        assert!(ft.try_instant_fuel(&sv.flight, p.nu_lp, p.nu_hp, p.mf * (1.0 - 1e-9)).is_err(), "at s={}", p.s);
        let (gc, _) = clip(p);
        assert!(p.mf_sched - gc < p.mf, "the clip asked for less than the wall at s={}", p.s);
    }
    // The page's words for the minimum-fuel stop.
    assert_eq!(turbojet::sandbox_controls::Holder::MinFuel.key(), "min_fuel");
}

#[test]
fn the_page_refuses_a_governor_whose_request_starts_above_the_redline() {
    // The crash map's #896 (seed 12): the metal cap holds the start at ~978 K, but the thrust governor's request at s = 0
    // is above the redline — the clip would charge from nothing under the metal leg, and the fast slowdown then held the
    // burner at the lean edge for ~0.6 τ. Refused (the user's choice), as the hot start.
    use turbojet::sandbox_controls::{BleedMode, LeverChoice};
    let s = gl_on(ControlsSettings { hold: Hold::Thrust, gov_tau: 0.44, gov_gain: 44.0, hold_cap: true, metal_on: true,
                                     cool_blade: 0.13, lever: LeverChoice::Stator, bleed_mode: BleedMode::Valve, gov_lag: 0.42,
                                     from: 1250.0, to: 690.0, ramp: 0.1, rho: 1.2000000000000002, ..d() });
    let err = build(&s).err().expect("refused");
    assert!(err.0.contains("asks for more fuel than the temperature limiter allows"), "{}", err.0);
    assert!(err.0.contains("the metal cap"), "{}", err.0);
    let sv = build(&ControlsSettings { gov_lag_on: false, ..s }).unwrap();
    assert!(sv.start.tt4 < s.redline, "not a hot start: {}", sv.start.tt4);
    // Called directly, the march reaches the minimum-fuel stop — the case the refusal keeps off the page.
    let pts = direct(&s, 0.42, DS);
    assert_eq!(pts.len(), s.expected_points());
    assert!(pts.iter().any(|p| p.mf == p.mf_sched && clip(p).0 > 0.0 && p.mf < 0.002), "held at the lean edge");
}

#[test]
fn the_page_refuses_the_lag_beside_the_slow_exhaust_sensor() {
    // Rung 101's lagged sensor as its own tests build it (the formula, flown at Mach 1.4).
    let s = gl_on(ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, speed_corr: SpeedCorr::Formula,
                                     airflow: AirflowFrom::Exhaust, exhaust_lag: true, t0: 250.0, p0: 50_000.0, m0: 1.4, ..d() });
    assert!(build(&ControlsSettings { gov_lag_on: false, ..s }).unwrap().sensor_lag().is_some(), "the sensor lags");
    let err = build(&s).err().expect("two lags at once is refused");
    assert!(err.0.contains("not both"), "{}", err.0);
}

#[test]
fn every_lagged_governed_step_reruns_bit_for_bit() {
    use turbojet::sandbox_controls::{replay_step, stop_reason};
    for s in [gl_on(ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() }),
              gl_on(ControlsSettings { hold: Hold::HpSpeed, floor_on: true, ..d() }),
              gl_on(ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, to: 1600.0, gov_lag: 0.1, ..d() }),
              gl_on(ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, ..d() })] {
        let (sv, o) = controls(&s).unwrap();
        assert_eq!(sv.route(), Route::GovernorLag);
        assert_eq!(o.points.len(), s.expected_points());
        for w in o.points.windows(2) {
            let (a, b, g) = replay_step(&sv, &w[0]).unwrap().unwrap();
            assert_eq!((a.to_bits(), b.to_bits(), g.to_bits()),
                       (w[1].nu_lp.to_bits(), w[1].nu_hp.to_bits(), clip(&w[1]).0.to_bits()));
        }
        assert!(stop_reason(&sv, &o.points).is_none());
    }
}

#[test]
fn the_step_check_at_the_fastest_lag() {
    // § 12.10's class at the governed lag's floor: the march at 0.02 against 0.005, flow coefficients to 1e-4, on each
    // hold at its own most demanding knob (rung 95's step-check floors). Measured first: at ONE step (0.02, rung 47's
    // floor alone) the governed march misses by ~2e-4 under thrust and HP speed (the open ramp: 9.4e-5); at three steps
    // the fastest speed law (TAU_MIN) still 2.0e-4, at 0.1 6.9e-5, at 0.2 9.4e-5 — so the governed floor is 0.1.
    use turbojet::sandbox_controls::{GOV_LAG_MIN, GAIN_MAX, TAU_MIN};
    // (each hold at the floor, and the fastest speed law also at rung 47's own default lag 0.2)
    for (s, lag) in [(ControlsSettings { hold: Hold::Thrust, ..d() }, GOV_LAG_MIN),
                     (ControlsSettings { hold: Hold::HpSpeed, gov_tau: TAU_MIN, ..d() }, GOV_LAG_MIN),
                     (ControlsSettings { hold: Hold::HpSpeed, gov_tau: TAU_MIN, ..d() }, 0.2),
                     (ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: GAIN_MAX, ..d() }, GOV_LAG_MIN),
                     (ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d() }, GOV_LAG_MIN),
                     (ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, ..d() }, GOV_LAG_MIN),
                     (ControlsSettings { hold: Hold::BurnerTemp, to: 1600.0, ..d() }, GOV_LAG_MIN)] {
        let s = gl_on(ControlsSettings { gov_lag: lag, ..s });
        let sv = build(&s).unwrap();
        let g = sv.governor.as_ref().unwrap();
        let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
        let run = |ds: f64| gl::march(&sv.core.fuel, &sv.flight, Demand::Governor(g), sv.clip_lag().unwrap(), &sv.lag_limiters(),
                                      None, nu0, s.s_end(), ds).0;
        let (a, b) = (run(DS), run(DS / 4.0));
        assert_eq!(a.len(), s.expected_points());
        let dm = a.iter().enumerate().map(|(k, p)| (p.phi_lp - b[4 * k].phi_lp).abs().max((p.phi_hp - b[4 * k].phi_hp).abs()))
            .fold(0.0, f64::max);
        eprintln!("{:?} {:?} at the lag {lag}: step 0.02 vs 0.005, flow coefficients {dm:e}", g.law, s.hold);
        assert!(dm < 1e-4, "{:?}: {dm:e}", g.law);
    }
}

fn thrust_of(sv: &ControlsSolver, p: &FuelPoint) -> f64 { gv::thrust(&sv.reread(p).unwrap().base) }


// =============================================================================================
// SCORED — H1–H5 (anchor § 3), all HIT on the full grid (`docs/plans/scratch-archive/rung106/score.rs`); pinned here on
// two maps at the page's default lag (0.2). "Open" = the march with the open ramp handed in (≡ rung 47, gate above).
// =============================================================================================

fn runs(shape: usize, tau: f64) -> (ControlsSolver, Vec<FuelPoint>, Vec<FuelPoint>, Vec<FuelPoint>) {
    let s = gl_on(ControlsSettings { shape, hold: Hold::Thrust, gov_lag: tau, ..d() });
    let sv = build(&ControlsSettings { gov_lag_on: false, ..s }).unwrap();
    let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
    let gov = direct(&s, tau, DS);
    let ramp = s.schedule(sv.mf_lo, sv.mf_hi);
    let open = gl::march(&sv.core.fuel, &sv.flight, Demand::Ramp(&ramp), ClipLag { tt4_max: s.redline, tau }, &sv.lag_limiters(),
                         None, nu0, s.s_end(), DS).0;
    let inst = gv::march(&sv.core.fuel, &sv.flight, sv.governor.as_ref().unwrap(), &sv.limiters(), None, nu0, s.s_end(), DS).0;
    (sv, gov, open, inst)
}
fn peak(p: &[FuelPoint]) -> f64 { p.iter().map(|x| x.tt4).fold(f64::NEG_INFINITY, f64::max) }
fn stored(p: &[FuelPoint]) -> f64 { p.iter().map(|x| { let (g, r) = clip(x); g - r }).fold(f64::NEG_INFINITY, f64::max) }
/// The thrust step's fraction at every point.
fn frac(sv: &ControlsSolver, p: &[FuelPoint]) -> Vec<f64> {
    let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
    p.iter().map(|x| (thrust_of(sv, x) - f0) / (f1 - f0)).collect()
}
fn cross95(p: &[FuelPoint], fr: &[f64]) -> f64 {
    let k = (1..fr.len()).find(|&k| fr[k] >= 0.95).expect("reaches 95 %");
    p[k - 1].s + (p[k].s - p[k - 1].s) * (0.95 - fr[k - 1]) / (fr[k] - fr[k - 1])
}

#[test]
fn h1_h2_h3_h5_the_governor_overshoots_more_stores_more_and_engages_earlier() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        let (_, gov, open, _) = runs(shape, 0.2);
        assert!(peak(&gov) > peak(&open), "H1 shape {shape}: {} vs {}", peak(&gov), peak(&open));
        assert!(stored(&gov) > stored(&open) && stored(&open) > 0.0, "H3 shape {shape}");
        let engage = |p: &[FuelPoint]| p.iter().position(|x| clip(x).1 > 0.0).unwrap();
        assert!(engage(&gov) < engage(&open), "H5 shape {shape}");
        let (_, slow, _, _) = runs(shape, 0.4);
        assert!(peak(&slow) > peak(&gov), "H2 shape {shape}");
    }
}

#[test]
fn h4_the_lag_makes_the_governed_thrust_later_and_the_thrust_sags_after_the_move() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        let (sv, gov, open, inst) = runs(shape, 0.2);
        let (fg, fo, fi) = (frac(&sv, &gov), frac(&sv, &open), frac(&sv, &inst));
        assert!(cross95(&gov, &fg) > cross95(&inst, &fi), "H4 shape {shape}");
        // After scoring (reported, pinned): only the governed lagged run's thrust FALLS after the move — the stored cut
        // paid back. The open ramp's never falls; the instant limiter's reaches its thrust and holds it.
        let first_fall = |fr: &[f64]| (1..fr.len()).find(|&k| fr[k] < fr[k - 1] - 1e-9);
        let k = first_fall(&fg).expect("the governed lagged thrust sags");
        assert!((gov[k - 1].s - 0.5).abs() < 1e-9, "it sags from the end of the move");
        let sag = fg[k - 1] - fg[k..].iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(sag > 0.03, "a sag of {sag} of the thrust step");
        assert!(first_fall(&fo).is_none(), "the open ramp's thrust never falls");
        assert!(fi.windows(2).all(|w| w[1] >= w[0] - 1e-9), "the instant limiter's thrust never falls");
    }
}
