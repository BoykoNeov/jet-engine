//! **THE FUEL CLOSURES' LEAN WALL** (2026-10-20). Not a rung — a solver repair on the Controls view (sandbox plan § 40),
//! the open item § 39 found: a slam from a low throttle stopped at its first step.
//!
//! Rung 43's fuel closure (and rungs 62/90/91's bled twins) scans the LP-face flow up from the flow where the mixture
//! hits 0.065 to the flow where it hits 0.004 — `F_FLOOR`, a SEARCH bound (rung 43 chose it to keep the scan off the
//! region past the root where the maps go non-physical; the page's lean-stop words already say "not a combustion
//! model"). A steady start can burn leaner: 1.5–4.3 g/kg at the six probed flights, down to ~0.1 g/kg at Mach 1.6–2 by
//! the lowest throttle that builds. The march's first closure then cannot reach its own start: 1 553 of § 39's 1 691
//! first-step stops were starts leaner than the wall, and the other 138 roots sat within one scan step (0.04) under it.
//!
//! `TwoSpoolCore::lean_wall` — `None` everywhere by default (the old code, every Python-held oracle bit for bit); the
//! Controls view sets it per run to HALF the leaner steady end's mixture when that is under 0.004 (the user's choice:
//! tied to the run, so a fast power cut from ends richer than 0.008 still stops lean as before). Lowering the wall only
//! extends the scan past the old one, so a closure that bracketed before brackets on the same samples — the reduce
//! below is that claim, run. Probes archived, `docs/plans/scratch-archive/low-speed/`.

use turbojet::atmosphere::standard;
use turbojet::fuel_transient::{FuelPoint, FuelTransientCore};
use turbojet::sandbox::call;
use turbojet::sandbox_controls::{classify, controls, lean_wall, run_mixture, ControlsOutcome, ControlsSettings,
                                 ControlsSolver, CustomerMode, LeverChoice, StopKind, CONTROL_SHAPES};
use turbojet::visuals::Json;

fn shape(name: &str) -> usize { CONTROL_SHAPES.iter().position(|&k| k == name).unwrap() }

fn bits(p: &FuelPoint) -> [u64; 6] {
    [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.mf.to_bits(), p.f.to_bits(), p.tt4.to_bits()]
}

fn run(s: &ControlsSettings) -> (ControlsSolver, ControlsOutcome) { controls(s).unwrap() }

fn wall_of(sv: &ControlsSolver) -> Option<f64> { sv.core.fuel.inner.inner.base.lean_wall.get() }

/// The five variants, each reaching its own closure: plain and the stator lever rung 43's, the bleed lever rung 62's,
/// the customer bleed rung 90's, turbine cooling rung 91's.
fn variants(base: ControlsSettings) -> Vec<(&'static str, ControlsSettings)> {
    vec![
        ("plain", base),
        ("stator lever", ControlsSettings { lever: LeverChoice::Stator, ..base }),
        ("bleed lever", ControlsSettings { lever: LeverChoice::Bleed, ..base }),
        ("customer", ControlsSettings { customer: CustomerMode::Share, ..base }),
        ("cooling", ControlsSettings { cool_vane: 0.03, cool_blade: 0.02, ..base }),
    ]
}

#[test]
fn a_run_whose_ends_burn_at_0_008_or_richer_keeps_the_old_wall_bit_for_bit() {
    // The default slam (1000 → 1400 K): no wall set, and the run is the old code's.
    let d = ControlsSettings::defaults();
    let (sv, o) = run(&d);
    assert!(sv.start.close.f.min(sv.end.close.f) >= 2.0 * FuelTransientCore::F_FLOOR);
    assert_eq!(wall_of(&sv), None);
    let (sv0, o0) = run(&ControlsSettings { lean_old: true, ..d });
    assert_eq!(wall_of(&sv0), None, "the old code sets no wall");
    assert_eq!(o.points.len(), o0.points.len());
    for (a, b) in o.points.iter().zip(&o0.points) {
        assert_eq!(bits(a), bits(b), "s = {}", a.s);
    }
}

#[test]
fn a_lowered_wall_moves_no_run_the_old_wall_already_solved() {
    // A start at 560 K at the design flight burns ~6 g/kg: the wall drops to ~3 g/kg, yet the old wall's whole slam
    // ran — so every closure bracketed before the old wall and the march is the old one, bit for bit.
    let base = ControlsSettings { shape: shape("flat-lp"), from: 560.0, ..ControlsSettings::defaults() };
    for (name, s) in variants(base) {
        let (sv, o) = run(&s);
        let (sv0, o0) = run(&ControlsSettings { lean_old: true, ..s });
        let w = wall_of(&sv).unwrap_or_else(|| panic!("{name}: a start under 8 g/kg lowers the wall"));
        assert!(w < FuelTransientCore::F_FLOOR && (w - 0.5 * sv.start.close.f.min(sv.end.close.f)).abs() == 0.0, "{name}");
        assert_eq!(wall_of(&sv0), None, "{name}");
        assert_eq!(o0.points.len(), s.expected_points(), "{name}: the old wall's slam runs (the premise)");
        assert_eq!(o.points.len(), o0.points.len(), "{name}");
        for (a, b) in o.points.iter().zip(&o0.points) {
            assert_eq!(bits(a), bits(b), "{name}: s = {}", a.s);
        }
    }
}

#[test]
fn a_slam_from_a_start_leaner_than_the_old_wall_leaves_it_on_every_closure() {
    // Sea level, 450 K: every variant's start burns under 4 g/kg. The old wall stops the slam before its first point
    // — a lean stop, read off the start (§ 39 called it "no burner state"); the run's wall takes the whole slam, and its
    // first point is the start's own steady state.
    let (t, p) = standard(0.0);
    let base = ControlsSettings { shape: shape("flat-lp"), t0: t, p0: p, m0: 0.85, from: 450.0, ..ControlsSettings::defaults() };
    for (name, s) in variants(base) {
        let (sv0, o0) = run(&ControlsSettings { lean_old: true, ..s });
        assert!(sv0.start.close.f < FuelTransientCore::F_FLOOR, "{name}: the start is leaner than the old wall");
        assert!(o0.points.is_empty(), "{name}: the old wall stops at once ({} points)", o0.points.len());
        let st = o0.stop.as_ref().unwrap();
        assert_eq!(classify(&sv0, st, None), StopKind::Lean, "{name}: {:?}", st.failure);
        let x = run_mixture(&sv0, st.failure.as_ref().unwrap(), None).unwrap();
        assert!((x / sv0.start.close.f - 1.0).abs() < 0.05, "{name}: the trial mixture is the start's ({x})");

        let (sv, o) = run(&s);
        assert!(lean_wall(&sv) < sv.start.close.f, "{name}");
        assert_eq!(o.points.len(), s.expected_points(), "{name}: the whole slam runs ({:?})", o.stop);
        let p0 = &o.points[0];
        assert_eq!((p0.nu_lp, p0.nu_hp), (sv.start.nu_lp, sv.start.nu_hp), "{name}");
        assert!((p0.f / sv.start.close.f - 1.0).abs() < 1e-12, "{name}: the first point is the start ({} vs {})",
                p0.f, sv.start.close.f);
    }
}

#[test]
fn at_the_design_flight_a_low_slam_now_meets_the_rich_wall_and_the_schedule_takes_it() {
    // 480 K at the design flight: past its first step, the ramp outruns the slow shafts and the mixture reaches the
    // rich wall (a real stop, named rich); the acceleration schedule takes the same move to the end.
    let s = ControlsSettings { shape: shape("flat-lp"), from: 480.0, ..ControlsSettings::defaults() };
    let (sv0, o0) = run(&ControlsSettings { lean_old: true, ..s });
    assert!(o0.points.is_empty());
    assert_eq!(classify(&sv0, o0.stop.as_ref().unwrap(), None), StopKind::Lean);
    let (sv, o) = run(&s);
    assert!(o.points.len() > 5 && o.points.len() < s.expected_points(), "{} points", o.points.len());
    assert_eq!(classify(&sv, o.stop.as_ref().unwrap(), o.points.last()), StopKind::Rich);
    let (_, o) = run(&ControlsSettings { accel_on: true, ..s });
    assert_eq!(o.points.len(), s.expected_points(), "{:?}", o.stop);
}

#[test]
fn a_slam_from_a_crawling_lp_shaft_is_still_named_low_speed() {
    // The one first-step stop the run's wall leaves (probe 3, 1 of 1 691): the customer bleed at sea level +40 K from
    // 440 K starts with the LP shaft at 15 % speed (5 K higher, 38 %), and its closure fails with the start's mixture
    // ABOVE the run's wall — no wall bound it, so "the shafts are too slow" is the reason.
    // Rung 104: this crawling start is rung 38's choked vane's (the `pin` setting). Under the LP turbine's flow law the
    // same start runs its LP shaft at 37 %, and a search (four maps, three hot/warm flights, every customer mode, 400–490 K)
    // found no slam stopping at once for slow shafts — the stop's words stay driven here, on the pin.
    let (t, p) = standard(0.0);
    let s = ControlsSettings { shape: shape("tilted"), t0: t + 40.0, p0: p, m0: 0.85, customer: CustomerMode::Share,
                               from: 440.0, turbine: turbojet::turbine_law::TurbineLaw::Pin, ..ControlsSettings::defaults() };
    let (sv, o) = run(&s);
    assert!(sv.start.nu_lp < 0.2, "{}", sv.start.nu_lp);
    assert!(o.points.is_empty());
    assert!(sv.start.close.f > lean_wall(&sv));
    assert_eq!(classify(&sv, o.stop.as_ref().unwrap(), None), StopKind::LowSpeed);
}

#[test]
fn the_page_takes_the_hidden_setting_and_writes_it_only_off_its_default() {
    let req = |extra: &str| format!(r#"{{"op":"controls","controls":{{"shape":"flat-lp","from":480{extra}}}}}"#);
    let new = Json::parse(&call(&req("")));
    let old = Json::parse(&call(&req(r#","lean":"old""#)));
    let kind = |j: &Json| match j.get("stop").and_then(|s| s.get("kind")) { Some(Json::Str(k)) => k.clone(), _ => String::new() };
    assert_eq!((kind(&new), kind(&old)), ("rich".to_string(), "lean".to_string()));
    let echoed = |j: &Json| j.get("controls").map(|s| s.get("lean").is_some());
    assert_eq!((echoed(&new), echoed(&old)), (Some(false), Some(true)));
    assert!(call(&req(r#","lean":"nope""#)).contains("unknown lean wall"));
}
