//! RUNG 107 — THE GOVERNOR BESIDE THE REALISTIC RELEASE. Anchor: `docs/plans/rung107-anchor-governor-release.md`;
//! spec: `docs/rung107-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 2, "no credit") first: the open ramp as the demand IS rung 52's march; before the
//! lagged legs first cut the run IS rung 95's; the trigger pins itself; the lag's ledger per phase; `τ → 0` tends to rung
//! 95's instant schedule; every step re-runs; the page's refusals. Then the scored claims, pinned at a subset of the grid.

use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{AsymmetricLag, FuelLimiters, FuelPoint, PointExtra};
use turbojet::governor::{self as gv, Leg};
use turbojet::governor_lag::Demand;
use turbojet::governor_release as gr;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ControlsSolver, Route, SpeedLaw, ThrustBy, DS};

fn d() -> ControlsSettings { ControlsSettings::defaults() }
/// The governed slam beside the realistic release, at the governed floors (attack 0.2) unless set.
fn rel_on(s: ControlsSettings) -> ControlsSettings {
    ControlsSettings { govern: true, release_on: true, tau_att: 0.2, tau_rel: 0.4, ..s }
}
const FLAT_LP: usize = 3;
fn lag(p: &FuelPoint) -> (f64, f64) {
    match p.extra { PointExtra::Asym { g, required } => (g, required), _ => panic!("the release's route records its lag") }
}
fn bits(p: &FuelPoint) -> [u64; 4] { [p.nu_lp, p.nu_hp, p.mf, p.tt4].map(f64::to_bits) }
/// The march called directly (any lags, any step) on the machine the page builds for `s` — governed, or the open ramp.
fn direct(s: &ControlsSettings, ta: f64, tr: f64, open: bool, ds: f64) -> Vec<FuelPoint> {
    let sv = build(&ControlsSettings { release_on: false, ..*s }).unwrap();
    let lim = FuelLimiters { lag: Some(AsymmetricLag::new(ta, tr)), ..sv.limiters() };
    let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
    let ramp = s.schedule(sv.mf_lo, sv.mf_hi);
    let dem = if open { Demand::Ramp(&ramp) } else { Demand::Governor(sv.governor.as_ref().unwrap()) };
    gr::march(&sv.core.fuel, &sv.flight, dem, &lim, None, nu0, s.s_end(), ds).0
}
/// The first recorded release point: `req < g` after `g > 0`.
fn crossing(p: &[FuelPoint]) -> Option<usize> { p.iter().position(|x| { let (g, r) = lag(x); g > 0.0 && r < g }) }

// =============================================================================================
// CONSTRUCTION
// =============================================================================================

#[test]
fn the_open_ramp_as_the_demand_is_rung52s_march_bit_for_bit() {
    // Two code paths: rung 52's release route on the Controls view, and the new march with the ramp handed in (G1's cases).
    let cases = [ControlsSettings { accel_on: true, ..d() },
                 ControlsSettings { floor_on: true, ..d() },
                 ControlsSettings { accel_on: true, floor_on: true, redline_on: true, ..d() },
                 ControlsSettings { floor_on: true, from: 1400.0, to: 1000.0, ..d() }];
    let mut released = 0;
    for c in cases {
        for shape in 0..4 {
            for (ta, tr) in [(0.02, 0.1), (0.02, 0.8), (0.1, 0.1), (0.2, 0.05)] {
                let s = ControlsSettings { shape, release_on: true, tau_att: ta, tau_rel: tr, ..c };
                let sv = build(&s).unwrap();
                assert_eq!(sv.route(), Route::Release);
                let r52 = sv.march();
                let ramp = s.schedule(sv.mf_lo, sv.mf_hi);
                let (pts, marks) = gr::march(&sv.core.fuel, &sv.flight, Demand::Ramp(&ramp), &sv.limiters(), None,
                                             (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), DS);
                assert_eq!(pts.len(), r52.len(), "shape {shape} ({ta}, {tr})");
                for (a, b) in pts.iter().zip(&r52) {
                    assert_eq!(bits(a), bits(b), "shape {shape} ({ta}, {tr}) at s={}", a.s);
                    assert_eq!(lag(a).0.to_bits(), lag(b).0.to_bits());
                    assert_eq!(a.mf_sched.to_bits(), b.mf_sched.to_bits());
                }
                assert!(marks.iter().all(|m| m.leg != Leg::MinFuel), "the open path has no lean hold");
                released += crossing(&pts).is_some() as usize;
            }
        }
    }
    assert!(released > 0, "some open run releases");
}

#[test]
fn off_is_the_earlier_run_and_before_the_lag_engages_it_is_rung95s() {
    for shape in [0, FLAT_LP] {
        for (hold, extra) in [(Hold::Thrust, ControlsSettings { accel_on: true, ..d() }),
                              (Hold::HpSpeed, ControlsSettings { accel_on: true, ..d() }),
                              (Hold::Thrust, ControlsSettings { floor_on: true, shape: 1, ..d() })] {
            let s = rel_on(ControlsSettings { shape: if extra.floor_on { 1 } else { shape }, hold, ..extra });
            let (sv, o) = controls(&s).unwrap();
            assert_eq!(sv.route(), Route::GovernorRelease);
            assert_eq!(o.points.len(), s.expected_points());
            let (sv95, r95) = controls(&ControlsSettings { release_on: false, ..s }).unwrap();
            assert_eq!(sv95.route(), Route::Governor);
            let first = o.points.iter().position(|p| lag(p).0 != 0.0).expect("the lag engages");
            assert!(first > 0);
            for (a, b) in o.points[..first].iter().zip(&r95.points) {
                assert_eq!(bits(a), bits(b), "{hold:?} shape {shape} at s={}", a.s);
            }
        }
    }
    // A schedule that never cuts: the whole run is rung 95's.
    let s = rel_on(ControlsSettings { hold: Hold::Thrust, accel_on: true, accel_margin: 3.0, ..d() });
    let (_, o) = controls(&s).unwrap();
    let (_, r95) = controls(&ControlsSettings { release_on: false, ..s }).unwrap();
    assert_eq!(o.points.len(), r95.points.len());
    assert!(o.points.iter().all(|p| lag(p) == (0.0, 0.0)));
    for (a, b) in o.points.iter().zip(&r95.points) {
        assert_eq!(bits(a), bits(b));
    }
}

#[test]
fn the_trigger_pins_itself_governed() {
    // Rung 52 § 1, structural: τ_rel is never read while req > g, so every point before the first crossing is the same.
    for shape in 0..4 {
        let s = rel_on(ControlsSettings { shape, hold: Hold::Thrust, accel_on: true, ..d() });
        let runs: Vec<Vec<FuelPoint>> = [0.1, 0.4, 0.8].iter().map(|&tr| direct(&s, 0.2, tr, false, DS)).collect();
        let k = crossing(&runs[0]).expect("the lag releases");
        assert!(k > 1, "shape {shape}");
        for r in &runs[1..] {
            assert_eq!(crossing(r), Some(k), "shape {shape}: the crossing is τ_rel-free");
            for (a, b) in runs[0][..k].iter().zip(&r[..k]) {
                assert_eq!(bits(a), bits(b), "shape {shape} at s={}", a.s);
                assert_eq!(lag(a).0.to_bits(), lag(b).0.to_bits());
            }
        }
        // ... and past it they differ (τ_rel is read).
        assert!(runs[0][k + 1..].iter().zip(&runs[2][k + 1..]).any(|(a, b)| bits(a) != bits(b)), "shape {shape}");
    }
}

fn trapz(p: &[FuelPoint], f: impl Fn(&FuelPoint) -> f64) -> f64 {
    p.windows(2).map(|w| 0.5 * (w[1].s - w[0].s) * (f(&w[0]) + f(&w[1]))).sum()
}
/// D1's per-phase misses, each a fraction of `τ·g_c`: attack points `0..k`, release `k..`, `k` the first crossing.
fn ledger(p: &[FuelPoint], ta: f64, tr: f64) -> (f64, f64) {
    let k = crossing(p).expect("the lag releases");
    let gc = lag(&p[k]).0;
    let (att, rel) = (&p[..k], &p[k..]);
    let ma = (trapz(att, |x| lag(x).1 - lag(x).0) - ta * (lag(att.last().unwrap()).0 - lag(&att[0]).0)).abs() / (ta * gc);
    let mr = (trapz(rel, |x| lag(x).0 - lag(x).1) - tr * (lag(&rel[0]).0 - lag(rel.last().unwrap()).0)).abs() / (tr * gc);
    (ma, mr)
}

#[test]
fn the_lags_ledger_balances_to_the_frozen_bars() {
    // Anchor § 2 (D1), bars frozen from ledger.txt: the schedule ≤ 1e−2·τ·g_c per phase (falling when the step halves at
    // (0.2, 0.4)); the floor ≤ 2e−2·τ·g_c.
    let lags = [(0.2, 0.2), (0.2, 0.4), (0.2, 0.8), (0.2, 1.6), (0.4, 0.4), (0.4, 1.6)];
    for shape in 0..4 {
        let s = rel_on(ControlsSettings { shape, hold: Hold::Thrust, accel_on: true, ..d() });
        for (ta, tr) in lags {
            let (ma, mr) = ledger(&direct(&s, ta, tr, false, DS), ta, tr);
            eprintln!("schedule shape {shape} ({ta}, {tr}): attack {ma:.2e} release {mr:.2e}");
            assert!(ma <= 1e-2 && mr <= 1e-2, "schedule shape {shape} ({ta}, {tr}): {ma:e} {mr:e}");
        }
        let (a2, r2) = ledger(&direct(&s, 0.2, 0.4, false, DS), 0.2, 0.4);
        let (a1, r1) = ledger(&direct(&s, 0.2, 0.4, false, DS / 2.0), 0.2, 0.4);
        assert!(a1 < a2 && r1 < r2, "shape {shape}: the miss is the trapezoid's ({a2:e}->{a1:e}, {r2:e}->{r1:e})");
    }
    for shape in 0..3 {
        let s = rel_on(ControlsSettings { shape, hold: Hold::Thrust, floor_on: true, ..d() });
        for (ta, tr) in lags {
            let (ma, mr) = ledger(&direct(&s, ta, tr, false, DS), ta, tr);
            eprintln!("floor shape {shape} ({ta}, {tr}): attack {ma:.2e} release {mr:.2e}");
            assert!(ma <= 2e-2 && mr <= 2e-2, "floor shape {shape} ({ta}, {tr}): {ma:e} {mr:e}");
        }
    }
}

#[test]
fn vanishing_lags_tend_to_rung95s_instant_schedule() {
    // τ_att = τ_rel = 0.02, 0.01, 0.005 at a step τ/4: the gap to rung 95 with the instant schedule shrinks.
    let s = rel_on(ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() });
    let sv = build(&ControlsSettings { release_on: false, ..s }).unwrap();
    let g = sv.governor.as_ref().unwrap();
    let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
    let mut gaps = Vec::new();
    for tau in [0.02, 0.01, 0.005] {
        let ds = tau / 4.0;
        let (inst, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), ds);
        let lagged = direct(&s, tau, tau, false, ds);
        assert_eq!(inst.len(), lagged.len());
        let gap = inst.iter().zip(&lagged).map(|(a, b)| (a.nu_lp - b.nu_lp).abs().max((a.nu_hp - b.nu_hp).abs()))
            .fold(0.0, f64::max);
        eprintln!("tau {tau}: max speed gap to the instant schedule {gap:e}");
        gaps.push(gap);
    }
    assert!(gaps[1] < gaps[0] && gaps[2] < gaps[1], "{gaps:?}");
    eprintln!("orders: {:.2} {:.2}", (gaps[0] / gaps[1]).log2(), (gaps[1] / gaps[2]).log2());
}

#[test]
fn every_released_governed_step_reruns_bit_for_bit() {
    use turbojet::sandbox_controls::{replay_step, stop_reason};
    for s in [rel_on(ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() }),
              rel_on(ControlsSettings { hold: Hold::Thrust, floor_on: true, shape: 1, tau_rel: 0.8, ..d() }),
              rel_on(ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0, accel_on: true,
                                        floor_on: true, redline_on: true, ..d() }),
              rel_on(ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, gov_tau: 0.15, accel_on: true, ..d() })] {
        let (sv, o) = controls(&s).unwrap();
        assert_eq!(sv.route(), Route::GovernorRelease);
        assert_eq!(o.points.len(), s.expected_points());
        assert!(o.points.iter().any(|p| lag(p).0 > 0.0), "the lag engages");
        for w in o.points.windows(2) {
            let (a, b, g) = replay_step(&sv, &w[0]).unwrap().unwrap();
            assert_eq!((a.to_bits(), b.to_bits(), g.to_bits()),
                       (w[1].nu_lp.to_bits(), w[1].nu_hp.to_bits(), lag(&w[1]).0.to_bits()));
        }
        assert!(stop_reason(&sv, &o.points).is_none());
    }
}

#[test]
fn the_page_refuses_in_words() {
    let refused = |s: ControlsSettings, words: &str| {
        let e = build(&s).err().unwrap_or_else(|| panic!("refused: {words}"));
        assert!(e.0.contains(words), "{}", e.0);
    };
    let s = rel_on(ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() });
    // The knobs' floors beside the governor (the step check) — the governor off keeps rung 52's one step.
    refused(ControlsSettings { tau_att: 0.1, ..s }, "attack time must be at least 0.2");
    refused(ControlsSettings { tau_rel: 0.06, ..s }, "release time at least 0.1");
    assert!(build(&ControlsSettings { govern: false, tau_att: 0.02, tau_rel: 0.02, ..s }).is_ok());
    // Each speed law past rung 106's edges.
    refused(ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 20.0, ..s },
            "Beside the realistic release the speed governor's gain");
    refused(ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, gov_tau: 0.1, ..s },
            "Beside the realistic release the fan-speed governor's time constant");
    assert!(build(&ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0, ..s }).is_ok());
    // Two lags at once.
    refused(ControlsSettings { redline_on: true, gov_lag_on: true, ..s }, "cannot run together");
    // ... and beside rung 101's lagged exhaust sensor (as rung 106's tests build it: the formula, Mach 1.4).
    use turbojet::airflow_source::AirflowFrom;
    use turbojet::mach_axis::SpeedCorr;
    let sensed = rel_on(ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, speed_corr: SpeedCorr::Formula,
                                           airflow: AirflowFrom::Exhaust, exhaust_lag: true, accel_on: true, t0: 250.0,
                                           p0: 50_000.0, m0: 1.4, ..d() });
    assert!(build(&ControlsSettings { release_on: false, ..sensed }).unwrap().sensor_lag().is_some(), "the sensor lags");
    refused(sensed, "beside the realistic release, or beside the slow exhaust sensor, but not both");
    // The floor above the start (flat-LP's start sits below the LP floor) — the governor off keeps showing it.
    let flat = rel_on(ControlsSettings { hold: Hold::Thrust, floor_on: true, shape: FLAT_LP, ..d() });
    refused(flat, "the governor asks for more fuel than the stall floor allows");
    assert!(build(&ControlsSettings { govern: false, ..flat }).is_ok());
}

// (The governed lean hold — rung 106's minimum-fuel stop on this path — never held in the admitted region: 0 of 1 344
// runs in anchor § 0 G3′ and 0 of the crash map's 270 admitted runs. A test of it here had nothing to hold, so it is not
// shipped; spec § 2 says so.)

// =============================================================================================
// SCORED (anchor § 3 — the scoring is `docs/plans/scratch-archive/rung107/score.rs`; pinned at a subset)
// =============================================================================================

/// The thrust share `(F − F_start)/(F_end − F_start)` at each point.
fn shares(sv: &ControlsSolver, p: &[FuelPoint]) -> Vec<f64> {
    let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
    p.iter().map(|x| (gv::thrust(&sv.reread(x).unwrap().base) - f0) / (f1 - f0)).collect()
}
/// The largest fall of the thrust share below its running maximum, from the move's end (by INDEX) on.
fn drawdown(s: &ControlsSettings, fr: &[f64]) -> f64 {
    let k0 = (s.ramp / DS).round_ties_even() as usize;
    let (mut hi, mut dd) = (f64::NEG_INFINITY, 0.0f64);
    for &v in &fr[k0..] {
        hi = hi.max(v);
        dd = dd.max(hi - v);
    }
    dd
}
fn instant(sv: &ControlsSolver) -> Vec<FuelPoint> {
    gv::march(&sv.core.fuel, &sv.flight, sv.governor.as_ref().unwrap(), &sv.limiters(), None,
              (sv.start.nu_lp, sv.start.nu_hp), sv.settings.s_end(), DS).0
}

#[test]
fn h6_the_governed_thrust_sags_beside_the_slow_release_and_neither_comparison_does() {
    // H6 (16 of 16 HIT), the thrust hold only: drawdown after the move > 0.002 governed; open and instant never fall that
    // far. The redline is OFF — no temperature-limiter lag (rung 106's) is needed. The anchor's REASON (the release pays
    // back the stored cut) is corrected by the next test: at the page's attack it is mostly the attack's let-through.
    for shape in [0, FLAT_LP] {
        let s = rel_on(ControlsSettings { shape, hold: Hold::Thrust, accel_on: true, ..d() });
        let sv = build(&ControlsSettings { release_on: false, ..s }).unwrap();
        let inst = drawdown(&s, &shares(&sv, &instant(&sv)));
        assert!(inst <= 0.002, "shape {shape}: instant {inst}");
        let mut last = 0.0;
        for tr in [0.2, 1.6] {
            let gov = drawdown(&s, &shares(&sv, &direct(&s, 0.2, tr, false, DS)));
            let open = drawdown(&s, &shares(&sv, &direct(&s, 0.2, tr, true, DS)));
            assert!(gov > 0.002 && open <= 0.002, "shape {shape} tau_rel {tr}: governed {gov}, open {open}");
            assert!(gov > last, "the sag deepens with the release time");
            last = gov;
        }
    }
}

#[test]
fn the_sag_is_mostly_the_attacks_let_through_and_the_release_alone_also_makes_one() {
    // Reported off protocol (`scratch-archive/rung107/sagcause.txt`), pinned because the headline rests on it. Step 0.005
    // (the fast lags are rough at 0.02). Each edge ALONE sags; the two do not add.
    let s = rel_on(ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() });
    let sv = build(&ControlsSettings { release_on: false, ..s }).unwrap();
    let ds = 0.005;
    let sag = |ta: f64, tr: f64| {
        let p = direct(&s, ta, tr, false, ds);
        let fr = shares(&sv, &p);
        let k0 = (s.ramp / ds).round_ties_even() as usize;
        let (mut hi, mut dd) = (f64::NEG_INFINITY, 0.0f64);
        for &v in &fr[k0..] {
            hi = hi.max(v);
            dd = dd.max(hi - v);
        }
        dd
    };
    let (attack_only, release_only, both) = (sag(0.2, 0.005), sag(0.005, 1.6), sag(0.2, 0.4));
    eprintln!("attack alone {attack_only:.4}, release alone {release_only:.4}, both {both:.4}");
    assert!(sag(0.005, 0.005) <= 0.002, "both edges instant: rung 95's");
    assert!(release_only > 0.002, "the release alone sags");
    assert!(attack_only > 0.02 && attack_only > 0.85 * both && attack_only < both, "the attack carries most of it");
}

#[test]
fn h1_the_governor_cuts_deeper_and_earlier_and_the_deepest_cut_reads_the_release() {
    // H1 (8 of 8 HIT); and the anchor's "max g is τ_rel-free" is NOT exact: the deepest cut sits at the crossing, whose
    // step's inner stages already read τ_rel (a hair, ~1e-3 relative).
    let s = rel_on(ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() });
    let max_g = |p: &[FuelPoint]| p.iter().map(|x| lag(x).0).fold(0.0, f64::max);
    let first = |p: &[FuelPoint]| p.iter().position(|x| lag(x).1 > 0.0).unwrap();
    let (g, o) = (direct(&s, 0.2, 0.4, false, DS), direct(&s, 0.2, 0.4, true, DS));
    assert!(max_g(&g) > 2.0 * max_g(&o), "{} vs {}", max_g(&g), max_g(&o));
    assert!(first(&g) < first(&o));
    let g16 = direct(&s, 0.2, 1.6, false, DS);
    assert_ne!(max_g(&g).to_bits(), max_g(&g16).to_bits());
    assert!((max_g(&g16) / max_g(&g) - 1.0).abs() < 1e-2);
}

#[test]
fn h4_h5_on_the_floor_both_spools_minima_come_before_the_release() {
    // H4 (6 of 6 HIT): the LP minimum is τ_rel-free, below the floor, while the floor cuts, before the first crossing.
    // H5 (6 of 6 MISS): the HP minimum ALSO comes before the crossing governed, so it is τ_rel-free too — and with the
    // governor OFF at the same attack it comes AFTER the crossing and rises with τ_rel (rung 52 § 4): the governor's.
    for shape in 0..3 {
        let s = rel_on(ControlsSettings { shape, hold: Hold::Thrust, floor_on: true, ..d() });
        let argmin = |p: &[FuelPoint], f: fn(&FuelPoint) -> f64| {
            (0..p.len()).min_by(|&i, &j| f(&p[i]).total_cmp(&f(&p[j]))).unwrap()
        };
        let gov: Vec<Vec<FuelPoint>> = [0.2, 1.6].iter().map(|&tr| direct(&s, 0.2, tr, false, DS)).collect();
        for r in &gov {
            let k = crossing(r).unwrap();
            let (il, ih) = (argmin(r, |x| x.phi_lp), argmin(r, |x| x.phi_hp));
            assert!(il < k && ih < k, "shape {shape}: LP min {il}, HP min {ih}, crossing {k}");
            assert!(r[il].phi_lp < 0.75 && lag(&r[il]).1 > 0.0);
        }
        assert_eq!(gov[0][argmin(&gov[0], |x| x.phi_lp)].phi_lp.to_bits(), gov[1][argmin(&gov[1], |x| x.phi_lp)].phi_lp.to_bits());
        assert_eq!(gov[0][argmin(&gov[0], |x| x.phi_hp)].phi_hp.to_bits(), gov[1][argmin(&gov[1], |x| x.phi_hp)].phi_hp.to_bits());
        let open: Vec<Vec<FuelPoint>> = [0.2, 1.6].iter().map(|&tr| direct(&s, 0.2, tr, true, DS)).collect();
        let (h0, h1) = (argmin(&open[0], |x| x.phi_hp), argmin(&open[1], |x| x.phi_hp));
        assert!(h0 > crossing(&open[0]).unwrap() && h1 > crossing(&open[1]).unwrap(), "shape {shape}");
        assert!(open[1][h1].phi_hp > open[0][h0].phi_hp, "shape {shape}: open, a slower release debits the HP less");
    }
}

#[test]
fn h2_the_fastest_release_beats_the_instant_schedule_and_a_slower_one_does_not() {
    // H2 vs instant: 8 HIT, 4 MISS (every MISS at τ_rel 0.2 — the attack's let-through arrives first), 4 VACUOUS (τ_rel 1.6
    // never reaches 95 % in the slam's 2.5).
    let t95 = |p: &[FuelPoint], fr: &[f64]| {
        let k = fr.iter().position(|&x| x >= 0.95)?;
        let (s0, s1, a, b) = (p[k - 1].s, p[k].s, fr[k - 1], fr[k]);
        Some(s0 + (s1 - s0) * (0.95 - a) / (b - a))
    };
    let s = rel_on(ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() });
    let sv = build(&ControlsSettings { release_on: false, ..s }).unwrap();
    let i = instant(&sv);
    let xi = t95(&i, &shares(&sv, &i)).unwrap();
    let x = |tr: f64| { let p = direct(&s, 0.2, tr, false, DS); t95(&p, &shares(&sv, &p)) };
    assert!(x(0.2).unwrap() < xi);
    assert!(x(0.4).unwrap() > xi && x(0.8).unwrap() > x(0.4).unwrap());
    assert!(x(1.6).is_none());
}
