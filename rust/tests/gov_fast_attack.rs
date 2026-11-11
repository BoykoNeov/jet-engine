//! PLAN § 45 — A FAST CUT BESIDE THE GOVERNOR (rung 107's first seam; a page and solver change, not a rung). Beside the
//! governor a realistic release that cuts in faster than 0.2 τ runs on a step four times finer (0.005 τ), recording every
//! step, down to the page's own 0.02 τ; at 0.2 and above the march is rung 107's, bit for bit. Bars frozen in
//! `docs/plans/sandbox-plan.md` § 45 before the code; measurements `docs/plans/scratch-archive/gov-fast-attack/`.

use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::governor as gv;
use turbojet::governor_lag::Demand;
use turbojet::governor_release as gr;
use turbojet::sandbox_controls::{build, controls, replay_step, stop_reason, ControlsSettings, ControlsSolver, Route,
                                 SpeedLaw, DS, GOV_REL_FINE_BELOW, GOV_REL_SUBSTEPS};
use turbojet::two_spool_transient::Instant2;

fn d() -> ControlsSettings { ControlsSettings::defaults() }
/// The governed slam beside the realistic release at the page's default cut (0.02) unless set.
fn fast(s: ControlsSettings) -> ControlsSettings {
    ControlsSettings { govern: true, release_on: true, tau_att: 0.02, tau_rel: 0.4, ..s }
}
fn lag(p: &FuelPoint) -> f64 {
    match p.extra { PointExtra::Asym { g, .. } => g, _ => panic!("the release's route records its lag") }
}
/// The shipped march's own call at any step, on the machine the page builds.
fn at_step(sv: &ControlsSolver, ds: f64) -> Vec<FuelPoint> {
    let over = |i: &Instant2| sv.metal_over(i);
    let metal: Option<&dyn Fn(&Instant2) -> f64> = if sv.settings.metal_leg() { Some(&over) } else { None };
    gr::march(&sv.core.fuel, &sv.flight, Demand::Governor(sv.governor.as_ref().unwrap()), &sv.limiters(), metal,
              (sv.start.nu_lp, sv.start.nu_hp), sv.settings.s_end(), ds).0
}

#[test]
fn the_step_is_the_pages_at_a_slow_cut_and_four_times_finer_at_a_fast_one() {
    assert_eq!(GOV_REL_SUBSTEPS, 4.0);
    assert_eq!(DS / GOV_REL_SUBSTEPS, 0.005, "a power of two: the fine step is exact");
    let base = ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() };
    // The page's step: a slow cut (rung 107's admitted region), the governor off (rung 52's route), the release off.
    for s in [ControlsSettings { tau_att: GOV_REL_FINE_BELOW, ..fast(base) }, ControlsSettings { tau_att: 0.5, ..fast(base) },
              ControlsSettings { govern: false, ..fast(base) }, ControlsSettings { release_on: false, ..fast(base) }] {
        assert_eq!(s.march_ds(), DS, "{:?}", (s.govern, s.release_on, s.tau_att));
        assert_eq!(s.expected_points(), (s.s_end() / DS).round_ties_even() as usize + 1);
    }
    for ta in [0.02, 0.1, 0.19] {
        let s = ControlsSettings { tau_att: ta, ..fast(base) };
        assert_eq!(s.march_ds(), DS / 4.0, "attack {ta}");
        assert_eq!(s.expected_points(), 4 * ((s.s_end() / DS).round_ties_even() as usize) + 1);
    }
    // A slow cut's run is rung 107's march at the page's step, point for point.
    let s = ControlsSettings { tau_att: 0.2, ..fast(base) };
    let (sv, o) = controls(&s).unwrap();
    let old = at_step(&sv, DS);
    assert_eq!(o.points.len(), old.len());
    assert!(o.points.iter().zip(&old).all(|(a, b)| format!("{a:?}") == format!("{b:?}")));
}

#[test]
fn a_fast_cut_runs_to_the_end_on_the_fine_step_and_every_step_reruns_bit_for_bit() {
    // B3: two laws, the schedule and the floor, at the page's default cut.
    for s in [fast(ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() }),
              fast(ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0, floor_on: true,
                                      shape: 1, tau_rel: 0.8, ..d() })] {
        let (sv, o) = controls(&s).unwrap();
        assert_eq!(sv.route(), Route::GovernorRelease);
        assert_eq!(o.points.len(), s.expected_points(), "{:?}", o.stop);
        assert!(o.points.iter().any(|p| lag(p) > 0.0), "the lag engages");
        for w in o.points.windows(2) {
            assert!((w[1].s - w[0].s - 0.005).abs() < 1e-12, "every recorded point one fine step apart");
            let (a, b, g) = replay_step(&sv, &w[0]).unwrap().unwrap();
            assert_eq!((a.to_bits(), b.to_bits(), g.to_bits()),
                       (w[1].nu_lp.to_bits(), w[1].nu_hp.to_bits(), lag(&w[1]).to_bits()));
        }
        assert!(stop_reason(&sv, &o.points).is_none());
    }
}

#[test]
fn the_fine_step_passes_the_step_check_at_the_fastest_and_slowest_fast_cut() {
    // B2's slice (the full sweep is archived): the shipped run against one four times finer again, under 1e-4 in either
    // flow coefficient on the shipped run's points.
    for s in [fast(ControlsSettings { hold: Hold::Thrust, accel_on: true, ..d() }),
              fast(ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0, floor_on: true,
                                      shape: 1, ..d() })] {
        for ta in [0.02, 0.19] {
            let s = ControlsSettings { tau_att: ta, ..s };
            let sv = build(&s).unwrap();
            let (x, y) = (sv.march(), at_step(&sv, DS / 16.0));
            assert_eq!((x.len() - 1) * 4, y.len() - 1);
            let gap = x.iter().enumerate()
                .map(|(i, p)| (p.phi_lp - y[4 * i].phi_lp).abs().max((p.phi_hp - y[4 * i].phi_hp).abs()))
                .fold(0.0, f64::max);
            assert!(gap < 1e-4, "attack {ta}: {gap:e}");
        }
    }
}

#[test]
fn every_point_is_kept_because_a_fast_cuts_sag_is_shorter_than_a_page_step() {
    // The user's choice (plan § 45): record every fine step. Holding thrust on the schedule, a 0.02 cut letting go over
    // 0.4: the sag read on every point is ~1.5 % of the thrust step, on every fourth point (the page's old spacing) ~1.2 %.
    for shape in [0usize, 3] {
        let s = fast(ControlsSettings { shape, hold: Hold::Thrust, accel_on: true, ..d() });
        let (sv, o) = controls(&s).unwrap();
        let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
        let fr: Vec<f64> = o.points.iter().map(|x| (gv::thrust(&sv.reread(x).unwrap().base) - f0) / (f1 - f0)).collect();
        let k0 = (s.ramp / s.march_ds()).round_ties_even() as usize;
        let sag = |every: usize| {
            let (mut hi, mut dd) = (f64::NEG_INFINITY, 0.0f64);
            for &v in fr[k0..].iter().step_by(every) {
                hi = hi.max(v);
                dd = dd.max(hi - v);
            }
            dd
        };
        let (all, coarse) = (sag(1), sag(4));
        eprintln!("shape {shape}: sag on every point {all:.4}, every fourth {coarse:.4}");
        assert!(all > 0.012 && all < 0.017, "shape {shape}: {all}");
        assert!(all > 1.1 * coarse, "shape {shape}: the coarse reading misses part of it ({all} vs {coarse})");
    }
}
