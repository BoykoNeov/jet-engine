//! RUNG 108 — THE TWO-SPEED TEMPERATURE LIMITER. Anchor: `docs/plans/rung108-anchor-two-speed-limiter.md`; spec:
//! `docs/rung108-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 2, "no credit") first: off is every earlier run; equal constants are rung 106's march
//! (governed) and rung 47's (open); the pinned trigger, the overshoot's with it; both constants → 0 tend to rung 95's
//! instant redline; the ledger per phase at its frozen bars; the page's fine step; every step re-runs; the refusals.
//! Then the scored claims (anchor § 3), pinned at a subset of the grid the spec scores.

use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{AsymmetricLag, FuelPoint, PointExtra};
use turbojet::governor::{self as gv, Leg};
use turbojet::governor_lag::{self as gl, ClipLag, Demand};
use turbojet::sandbox_controls::{build, controls, replay_step, stop_reason, ControlsSettings, ControlsSolver, Route,
                                 SpeedLaw, ThrustBy, DS};
use turbojet::two_speed_clip::{self as tsc, TwoSpeedClip};

fn d() -> ControlsSettings { ControlsSettings::defaults() }
/// The two-speed limiter on (cut-in `att`, give-back `rel`), governed or open.
fn two(s: ControlsSettings, att: f64, rel: f64) -> ControlsSettings {
    ControlsSettings { redline_on: true, gov_lag_on: true, gov_lag_two: true, gov_lag_att: att, gov_lag: rel, ..s }
}
fn clip(p: &FuelPoint) -> (f64, f64) {
    match p.extra { PointExtra::Asym { g, required } => (g, required), _ => panic!("the clip's route records its clip") }
}
fn bits(p: &FuelPoint) -> [u64; 5] { [p.nu_lp, p.nu_hp, p.mf, p.tt4, p.mf_sched].map(f64::to_bits) }
fn same(a: &[FuelPoint], b: &[FuelPoint], what: &str) {
    assert_eq!(a.len(), b.len(), "{what}: lengths");
    for (x, y) in a.iter().zip(b) {
        assert_eq!(bits(x), bits(y), "{what} at s={}", x.s);
        assert_eq!(clip(x).0.to_bits(), clip(y).0.to_bits(), "{what}: the clip at s={}", x.s);
    }
}
/// The two-speed march called directly on the machine the page builds for `s` (the tick box off, so any constants and
/// any step are allowed): governed when `s.govern`, else the open ramp.
fn direct(s: &ControlsSettings, att: f64, rel: f64, ds: f64) -> Vec<FuelPoint> {
    let sv = build(&ControlsSettings { gov_lag_on: false, gov_lag_two: false, ..*s }).unwrap();
    let ramp = s.schedule(sv.mf_lo, sv.mf_hi);
    let dem = match &sv.governor { Some(g) => Demand::Governor(g), None => Demand::Ramp(&ramp) };
    tsc::march(&sv.core.fuel, &sv.flight, dem, TwoSpeedClip { tt4_max: s.redline, lag: AsymmetricLag::new(att, rel) },
               &sv.lag_limiters(), None, (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), ds).0
}
/// Rung 106's (governed) or rung 47's (open, via the ramp as the demand — rung 106's gate) march at one lag.
fn single(s: &ControlsSettings, tau: f64, ds: f64) -> Vec<FuelPoint> {
    let sv = build(&ControlsSettings { gov_lag_on: false, gov_lag_two: false, ..*s }).unwrap();
    let ramp = s.schedule(sv.mf_lo, sv.mf_hi);
    let dem = match &sv.governor { Some(g) => Demand::Governor(g), None => Demand::Ramp(&ramp) };
    gl::march(&sv.core.fuel, &sv.flight, dem, ClipLag { tt4_max: s.redline, tau }, &sv.lag_limiters(), None,
              (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), ds).0
}
/// The first recorded crossing: the first point with `r ≤ g` after a point with `g > 0`.
fn crossing(p: &[FuelPoint]) -> Option<usize> {
    (1..p.len()).find(|&i| clip(&p[i - 1]).0 > 0.0 && { let (g, r) = clip(&p[i]); r <= g })
}
fn peak(p: &[FuelPoint]) -> (usize, f64) {
    p.iter().enumerate().map(|(k, x)| (k, x.tt4)).fold((0, f64::NEG_INFINITY), |a, b| if b.1 > a.1 { b } else { a })
}
const GOV: fn(ControlsSettings) -> ControlsSettings = |s| ControlsSettings { govern: true, hold: Hold::Thrust, ..s };
const OPEN: fn(ControlsSettings) -> ControlsSettings = |s| s;

// =============================================================================================
// CONSTRUCTION
// =============================================================================================

#[test]
fn off_is_every_earlier_run_and_the_tick_box_alone_moves_nothing() {
    for mk in [GOV, OPEN] {
        // The tick box with the lag off (or the limiter off) is not read: the same route, the same run.
        for base in [ControlsSettings { redline_on: true, ..mk(d()) }, mk(d())] {
            let (sv0, o0) = controls(&base).unwrap();
            let (sv1, o1) = controls(&ControlsSettings { gov_lag_two: true, gov_lag_att: 0.05, ..base }).unwrap();
            assert_eq!(sv0.route(), sv1.route());
            assert_eq!(o0.points.len(), o1.points.len());
            for (a, b) in o0.points.iter().zip(&o1.points) {
                assert_eq!(bits(a), bits(b));
            }
        }
        // Off: rung 47's / 106's route, as before.
        let lag = ControlsSettings { redline_on: true, gov_lag_on: true, ..mk(d()) };
        let sv = build(&lag).unwrap();
        assert!(matches!(sv.route(), Route::GovLag | Route::GovernorLag));
        assert_eq!(lag.march_ds(), DS);
    }
    // The default settings' JSON carries neither new key (every earlier reply byte-for-byte).
    let j = d().to_json().dump_compact();
    assert!(!j.contains("gov_lag_two") && !j.contains("gov_lag_att"), "{j}");
    let s = two(GOV(d()), 0.05, 0.4);
    let back = ControlsSettings::from_json(&s.to_json()).unwrap();
    assert!(back.gov_lag_two && back.gov_lag_att == 0.05 && back.gov_lag == 0.4);
}

#[test]
fn equal_constants_are_rung106s_and_rung47s_march_bit_for_bit() {
    for shape in 0..4 {
        for tau in [0.1, 0.2, 0.8] {
            // Open: the page's two-speed route at (τ, τ) IS rung 47's page route (its own march, `integrate_fuel`).
            let s = two(ControlsSettings { shape, ..d() }, tau, tau);
            let (sv, o) = controls(&s).unwrap();
            assert_eq!(sv.route(), Route::TwoSpeed);
            let (sv47, o47) = controls(&ControlsSettings { gov_lag_two: false, ..s }).unwrap();
            assert_eq!(sv47.route(), Route::GovLag);
            assert_eq!(o.points.len(), s.expected_points());
            for (a, b) in o.points.iter().zip(&o47.points) {
                assert_eq!(bits(a), bits(b), "open shape {shape} tau {tau} at s={}", a.s);
            }
            // Governed: the page's route (fine step) IS rung 106's march at that step; and at the page step, rung 106's
            // page route.
            let s = two(ControlsSettings { shape, ..GOV(d()) }, tau, tau);
            let (sv, o) = controls(&s).unwrap();
            assert_eq!(sv.route(), Route::GovernorTwoSpeed);
            assert_eq!(s.march_ds(), DS / 4.0);
            assert_eq!(o.points.len(), s.expected_points());
            same(&o.points, &single(&s, tau, DS / 4.0), &format!("governed shape {shape} tau {tau}"));
            let (_, o106) = controls(&ControlsSettings { gov_lag_two: false, ..s }).unwrap();
            same(&direct(&s, tau, tau, DS), &o106.points, &format!("governed page step shape {shape} tau {tau}"));
        }
    }
}

#[test]
fn the_pinned_trigger_and_the_overshoot_before_the_crossing() {
    // Anchor § 2 (as amended 2026-11-11): τ_rel is read only where an evaluation has r ≤ g, and the march's own RK4
    // STAGES can get there before the recorded points do. So the claim is pinned up to the SPLIT — the first point where
    // give-back 0.1 and 0.8 differ, i.e. the first point an early stage read reaches — and the split itself is pinned:
    //   * governed, on its 0.005 step: no stage reads τ_rel before the recorded crossing — the split IS the crossing, and
    //     every point before it is rung 106's at τ = τ_att bit for bit, the overshoot with it;
    //   * open, on its 0.02 step: at cut-in 0.02 the crossing step's k4 stage crosses one step early (split = k − 1, rung
    //     107's lesson 4); on shape 0 at cut-in ≤ 0.05 the ENGAGEMENT step's stages overshoot a target just leaving zero
    //     (split 15, s = 0.30, twelve steps early); elsewhere the split is the crossing.
    // Before the split: rung 47's at τ = τ_att bit for bit. From it to the crossing: within 5e−4 relative (measured
    // 2.2e−4), and the overshoot at the same point within 1e−3 K of rung 47's (measured 1.6e−4 K of a 24 K overshoot).
    for mk in [GOV, OPEN] {
        for shape in 0..4 {
            for att in [0.02, 0.05, 0.1] {
                let s = mk(ControlsSettings { shape, redline_on: true, ..d() });
                let ds = if s.govern { DS / 4.0 } else { DS };
                let r106 = single(&s, att, ds);
                let runs: Vec<Vec<FuelPoint>> = [0.1, 0.4, 0.8].iter().map(|&rel| direct(&s, att, rel, ds)).collect();
                let k = crossing(&runs[0]).expect("the clip crosses");
                let at = format!("govern {} shape {shape} att {att} k {k}", s.govern);
                let split = (0..runs[0].len())
                    .find(|&i| bits(&runs[0][i]) != bits(&runs[2][i]) || clip(&runs[0][i]).0 != clip(&runs[2][i]).0)
                    .expect("the give-back time matters somewhere");
                let want = match (s.govern, shape, att) {
                    (true, ..) => k,
                    (false, 0, a) if a <= 0.05 => 15,
                    (false, _, a) if a == 0.02 => k - 1,
                    _ => k,
                };
                assert_eq!(split, want, "the first point a give-back read reaches, {at}");
                let (kp, _) = peak(&runs[0]);
                assert!(kp < k, "{at}: peak at {kp}");
                let (kp106, tp106) = peak(&r106);
                assert_eq!(kp, kp106, "{at}: the peak's point");
                for r in &runs {
                    assert_eq!(crossing(r), Some(k));
                    same(&r[..split], &runs[0][..split], &format!("across the give-back time, {at}"));
                    same(&r[..split], &r106[..split], &format!("against rung 106/47 at the cut-in time, {at}"));
                    for (x, y) in r[split..k].iter().zip(&r106[split..k]) {
                        for (u, v) in [(x.tt4, y.tt4), (x.mf, y.mf)] {
                            assert!(((u - v) / v).abs() <= 5e-4, "{at}: the gap after the split at s={}", x.s);
                        }
                    }
                    let (kr, t) = peak(r);
                    assert_eq!(kr, kp, "{at}: the peak's point across the give-back time");
                    if kp < split {
                        assert_eq!(t.to_bits(), tp106.to_bits(), "{at}: the overshoot is the cut's");
                    } else {
                        assert!((t - tp106).abs() <= 1e-3, "{at}: the overshoot within 1e-3 K: {}", t - tp106);
                    }
                }
            }
        }
    }
}

#[test]
fn both_constants_vanishing_tend_to_rung95s_instant_redline() {
    let s = GOV(ControlsSettings { redline_on: true, ..d() });
    let sv = build(&s).unwrap();
    let g = sv.governor.as_ref().unwrap();
    let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
    let mut gaps = Vec::new();
    for tau in [0.02, 0.01, 0.005] {
        let ds = tau / 4.0;
        let (inst, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), ds);
        let two = direct(&s, tau, 2.0 * tau, ds);
        assert_eq!(inst.len(), two.len());
        let gap = inst.iter().zip(&two).map(|(a, b)| (a.nu_lp - b.nu_lp).abs().max((a.nu_hp - b.nu_hp).abs())).fold(0.0, f64::max);
        eprintln!("tau_att {tau}, tau_rel {}: max speed gap to the instant redline {gap:e}", 2.0 * tau);
        gaps.push(gap);
    }
    assert!(gaps[1] < gaps[0] && gaps[2] < gaps[1], "{gaps:?}");
    eprintln!("orders: {:.2} {:.2}", (gaps[0] / gaps[1]).log2(), (gaps[1] / gaps[2]).log2());
}

/// The ledger (D1) per phase on recorded points — the probe's own reading (`scratch-archive/rung108/ledger.rs`):
/// attack trapezoids `0 … k−1` of `r − g` against `τ_att·g_c`, release `k … end` of `g − r` against `τ_rel·(g_c − g_end)`,
/// each relative to `τ·g_c`.
fn ledger(p: &[FuelPoint], ta: f64, tr: f64) -> (f64, f64) {
    let k = crossing(p).expect("the clip crosses");
    let trap = |i: usize, f: &dyn Fn(&FuelPoint) -> f64| 0.5 * (f(&p[i]) + f(&p[i + 1])) * (p[i + 1].s - p[i].s);
    let att: f64 = (0..k - 1).map(|i| trap(i, &|x| { let (g, r) = clip(x); r - g })).sum();
    let rel: f64 = (k..p.len() - 1).map(|i| trap(i, &|x| { let (g, r) = clip(x); g - r })).sum();
    let (gc, ge) = (clip(&p[k]).0, clip(p.last().unwrap()).0);
    ((att - ta * gc).abs() / (ta * gc), (rel - tr * (gc - ge)).abs() / (tr * gc))
}

#[test]
fn the_ledger_per_phase_meets_its_frozen_bars() {
    // Anchor § 2: governed (0.005) ≤ 5e−3 of τ·g_c per phase; open (0.02) ≤ 3e−2.
    for (mk, ds, bar) in [(GOV, DS / 4.0, 5e-3), (OPEN, DS, 3e-2)] {
        for shape in 0..4 {
            for (ta, tr) in [(0.02, 0.2), (0.02, 0.8), (0.05, 0.4), (0.1, 1.6), (0.2, 0.2)] {
                let s = mk(ControlsSettings { shape, redline_on: true, ..d() });
                let p = direct(&s, ta, tr, ds);
                assert_eq!(p.len(), (s.s_end() / ds).round_ties_even() as usize + 1, "it runs to the end");
                let (ma, mr) = ledger(&p, ta, tr);
                eprintln!("govern {} shape {shape} ({ta}, {tr}): attack {ma:.1e}, release {mr:.1e}", s.govern);
                assert!(ma <= bar && mr <= bar, "govern {} shape {shape} ({ta}, {tr}): {ma:e} {mr:e} > {bar:e}", s.govern);
            }
        }
    }
}

#[test]
fn the_governed_two_speed_route_runs_and_reports_the_finer_step() {
    use turbojet::sandbox_controls::controls_json;
    use turbojet::visuals::Json;
    let s = two(GOV(d()), 0.02, 0.2);
    assert_eq!(s.march_ds(), DS / 4.0);
    let (sv, o) = controls(&s).unwrap();
    assert_eq!(o.points.len(), s.expected_points());
    assert!((o.points[1].s - DS / 4.0).abs() < 1e-15);
    let j = controls_json(&sv, &o);
    assert!(matches!(j.get("route"), Some(Json::Str(r)) if r == "governor_two_speed"), "route");
    assert!(matches!(j.get("ds"), Some(Json::Float(x)) if *x == DS / 4.0), "the reply carries the step");
    // The open route keeps the page step.
    let s = two(d(), 0.02, 0.2);
    assert_eq!(s.march_ds(), DS);
    let (sv, o) = controls(&s).unwrap();
    assert_eq!(o.points.len(), s.expected_points());
    assert!(matches!(controls_json(&sv, &o).get("route"), Some(Json::Str(r)) if r == "two_speed"));
    // The clip engages, and at the 0.02 cut the march records its lag's cut as the holder somewhere.
    assert!(o.points.iter().any(|p| clip(p).0 > 0.0));
}

#[test]
fn every_two_speed_step_reruns_bit_for_bit() {
    for s in [two(GOV(ControlsSettings { accel_on: true, ..d() }), 0.02, 0.2),
              two(GOV(ControlsSettings { hold: Hold::HpSpeed, floor_on: true, ..d() }), 0.05, 0.4),
              two(GOV(ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, gov_tau: 0.15, ..d() }), 0.02, 0.8),
              two(GOV(ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0, to: 1600.0, ..d() }),
                  0.02, 0.1),
              two(ControlsSettings { floor_on: true, ..d() }, 0.02, 0.2),
              two(ControlsSettings { shape: 3, accel_on: true, ..d() }, 0.05, 0.8)] {
        let (sv, o) = controls(&s).unwrap();
        assert!(matches!(sv.route(), Route::TwoSpeed | Route::GovernorTwoSpeed));
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
fn the_refusals_are_words() {
    let err = |s: ControlsSettings| build(&s).err().unwrap_or_else(|| panic!("refused: {s:?}")).0;
    // The cut-in floor, both routes.
    assert!(err(two(d(), 0.01, 0.2)).contains("cut-in time must be at least one march step"));
    assert!(err(two(GOV(d()), 0.01, 0.2)).contains("cut-in time must be at least one march step"));
    // The give-back keeps the lag's floors: one step open, 0.1 beside the governor.
    assert!(build(&two(d(), 0.02, 0.02)).is_ok());
    assert!(err(two(GOV(d()), 0.02, 0.05)).contains("give-back time must be at least"));
    // Two lags at once (carried over).
    assert!(err(two(ControlsSettings { release_on: true, accel_on: true, ..d() }, 0.02, 0.2)).contains("cannot run together"));
    // The hot start beside the governor (carried over).
    assert!(err(two(GOV(ControlsSettings { from: 1700.0, to: 900.0, ..d() }), 0.02, 0.2))
        .contains("above the temperature limiter's"));
}

#[test]
fn a_two_speed_cut_never_reaches_the_minimum_fuel_stop_on_the_default_grid() {
    // Anchor § 0 G2, pinned on two maps: one attack, one release, no lean-wall hold.
    for shape in [0, 3] {
        let (_, o) = controls(&two(GOV(ControlsSettings { shape, ..d() }), 0.02, 0.8)).unwrap();
        assert!(o.marks.iter().all(|m| m.leg != Leg::MinFuel));
        let p = &o.points;
        let attacks = (1..p.len()).filter(|&i| { let (g0, r0) = clip(&p[i - 1]); let (g1, r1) = clip(&p[i]); r0 <= g0 && r1 > g1 }).count();
        assert_eq!(attacks, 1, "shape {shape}");
    }
}

fn thrust_of(sv: &ControlsSolver, p: &FuelPoint) -> f64 { gv::thrust(&sv.reread(p).unwrap().base) }

// ---- The scored claims (anchor § 3), pinned on the flow/press map at the frozen grid's 0.005 step (the full grid's
// ---- scoring: `docs/rung108-spec.md`, the probe `docs/plans/scratch-archive/rung108/score.rs`).

const FINE: f64 = DS / 4.0;
fn shares(sv: &ControlsSolver, p: &[FuelPoint]) -> Vec<f64> {
    let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
    p.iter().map(|x| (thrust_of(sv, x) - f0) / (f1 - f0)).collect()
}
/// Rung 107's drawdown after the move's end (by index), the frozen sag.
fn drawdown(s: &ControlsSettings, fr: &[f64]) -> f64 {
    let k0 = (s.ramp / FINE).round_ties_even() as usize;
    let (mut hi, mut dd) = (f64::NEG_INFINITY, 0.0f64);
    for &v in fr.iter().skip(k0) { hi = hi.max(v); dd = dd.max(hi - v); }
    dd
}
/// The frozen settle time: the last upward crossing of 0.95 after which every point stays at or above it.
fn settle(p: &[FuelPoint], fr: &[f64]) -> Option<f64> {
    if *fr.last().unwrap() < 0.95 { return None; }
    Some(match (0..fr.len()).rev().find(|&i| fr[i] < 0.95) {
        None => p[0].s,
        Some(i) => p[i].s + (p[i + 1].s - p[i].s) * (0.95 - fr[i]) / (fr[i + 1] - fr[i]),
    })
}
fn instant(sv: &ControlsSolver) -> Vec<FuelPoint> {
    gv::march(&sv.core.fuel, &sv.flight, sv.governor.as_ref().unwrap(), &sv.limiters(), None,
              (sv.start.nu_lp, sv.start.nu_hp), sv.settings.s_end(), FINE).0
}
fn over(s: &ControlsSettings, p: &[FuelPoint]) -> f64 { peak(p).1 - s.redline }

#[test]
fn h1_h2_h3_h8_a_fast_cut_shrinks_the_overshoot_stores_more_and_spares_the_lp() {
    let gs = GOV(ControlsSettings { redline_on: true, ..d() });
    let os = OPEN(ControlsSettings { redline_on: true, ..d() });
    let atts = [0.02, 0.05, 0.1, 0.2];
    let g: Vec<Vec<FuelPoint>> = atts.iter().map(|&a| direct(&gs, a, 0.4, FINE)).collect();
    let o: Vec<Vec<FuelPoint>> = atts.iter().map(|&a| direct(&os, a, 0.4, FINE)).collect();
    let (og, oo): (Vec<f64>, Vec<f64>) = (g.iter().map(|p| over(&gs, p)).collect(), o.iter().map(|p| over(&os, p)).collect());
    // H1 (16 of 16 HIT): the overshoot falls strictly with the cut-in time; 0.02 against 0.1 within [0.2, 0.5]
    // (measured 0.30 governed, 0.26 open: 58.6 / 194.2 K and 24.3 / 95.2 K here).
    for ov in [&og, &oo] {
        assert!(ov.windows(2).all(|w| w[0] < w[1]), "{ov:?}");
        assert!((0.2..=0.5).contains(&(ov[0] / ov[2])), "{ov:?}");
    }
    assert!((og[0] - 58.564).abs() < 0.01 && (oo[0] - 24.338).abs() < 0.01, "{og:?} {oo:?}");
    // H2 (12 of 12 HIT): the governor overshoots more than the open ramp at every cut-in time (about 2.4x).
    for i in 0..3 { assert!(og[i] > oo[i], "{i}: {og:?} {oo:?}"); }
    // H3 (4 of 4 HIT): the faster cut stores more — the cut at the crossing falls as the cut-in time rises.
    let gc: Vec<f64> = g.iter().map(|p| clip(&p[crossing(p).unwrap()]).0).collect();
    assert!(gc.windows(2).all(|w| w[0] > w[1]), "{gc:?}");
    // H8 (16 of 16 HIT): the LP's lowest flow coefficient lies before the first crossing and rises as the cut-in time
    // falls — but stays BELOW the instant redline's (a lagged cut lets the fuel through there; reported).
    let sv = build(&gs).unwrap();
    let li = instant(&sv).iter().map(|x| x.phi_lp).fold(f64::INFINITY, f64::min);
    let lp: Vec<(usize, f64)> = g[..3].iter().map(|p| p.iter().enumerate().map(|(k, x)| (k, x.phi_lp))
        .fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a })).collect();
    for (i, p) in g[..3].iter().enumerate() { assert!(lp[i].0 < crossing(p).unwrap(), "{i}"); }
    assert!(lp[0].1 > lp[1].1 && lp[1].1 > lp[2].1 && lp[0].1 < li, "{lp:?} instant {li}");
}

#[test]
fn h4_h5_h6_h7_beside_the_governor_a_fast_cut_still_sags_and_settles_late() {
    let gs = GOV(ControlsSettings { redline_on: true, ..d() });
    let os = OPEN(ControlsSettings { redline_on: true, ..d() });
    let sv = build(&gs).unwrap();
    let svo = build(&os).unwrap();
    let rels = [0.2, 0.4, 0.8];
    let g: Vec<Vec<FuelPoint>> = rels.iter().map(|&r| direct(&gs, 0.02, r, FINE)).collect();
    let fr: Vec<Vec<f64>> = g.iter().map(|p| shares(&sv, p)).collect();
    let dd: Vec<f64> = fr.iter().map(|f| drawdown(&gs, f)).collect();
    // H4 (36 + 12 of 48 HIT): it sags, more with a slower give-back (0.027 / 0.033 / 0.040 here).
    assert!(dd[0] > 0.002 && dd[0] < dd[1] && dd[1] < dd[2], "{dd:?}");
    // H5 (8 HIT, 4 MISS): below rung 106's own sag at 0.2 and 0.4, ABOVE it at 0.8 — the named risk: the slow single lag
    // barely cuts (it lets the engine run ~387 K hot) so it has little to give back; the fast cut stores ~5.5x more.
    let d106: Vec<f64> = rels.iter().map(|&t| drawdown(&gs, &shares(&sv, &direct(&gs, t, t, FINE)))).collect();
    assert!(dd[0] < d106[0] && dd[1] < d106[1] && dd[2] > d106[2], "{dd:?} against 106 {d106:?}");
    // H6 (4 HIT, 8 + 4 VACUOUS): later than instant at 0.2 (2.16 against 1.32); at 0.4 and 0.8 the thrust never reaches
    // 95 % inside the 2.5 τ run — those cells VACUOUS by the frozen rule.
    let inst = instant(&sv);
    let xi = settle(&inst, &shares(&sv, &inst)).unwrap();
    let x2 = settle(&g[0], &fr[0]).unwrap();
    assert!(x2 > xi && (x2 - 2.1575).abs() < 1e-3 && (xi - 1.3225).abs() < 1e-3, "{x2} {xi}");
    for f in &fr[1..] { assert!(f.iter().all(|&v| v < 0.95)); }
    // H7 (36 of 36 HIT): the open ramp never sags.
    for &r in &rels {
        let p = direct(&os, 0.02, r, FINE);
        assert!(drawdown(&os, &shares(&svo, &p)) <= 0.002, "open {r}");
    }
}

#[test]
fn the_page_starts_the_box_ticked_and_the_model_does_not() {
    // The user's choice (2026-11-11): on the page the realistic two-speed limiter is what the lag gives by default; the
    // model's own default stays off, so every earlier request — and every gate and golden built on one — is unchanged.
    use turbojet::visuals::Json;
    let d0 = d();
    assert!(!d0.gov_lag_two && d0.gov_lag_att == DS);
    let reply = Json::parse(&turbojet::sandbox::call(r#"{"op":"controls_defaults"}"#));
    let c = reply.get("controls").expect("the defaults");
    assert!(matches!(c.get("gov_lag_two"), Some(Json::Int(1))), "ticked on the page");
    assert!(matches!(c.get("gov_lag_att"), Some(Json::Float(x)) if *x == DS));
    // The page's defaults, sent back, are the model's defaults with only that box ticked: with the lag off it is inert.
    let s = ControlsSettings::from_json(c).unwrap();
    assert_eq!(ControlsSettings { gov_lag_two: false, ..s }, d0);
    assert!(!s.two_speed());
}
