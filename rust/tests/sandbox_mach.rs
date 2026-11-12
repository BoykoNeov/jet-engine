//! The Controls view's Mach knob (`docs/plans/sandbox-plan.md` § 29): the two-shaft engine's hardware stays captured at
//! the design flight (Mach 0.85) while it is FLOWN at the knob's Mach. A missing knob is the earlier page; the engine
//! reads only its inlet; the march runs at the flight flown; the governor's step-checked limits scale with the density of
//! the air the engine takes in; every new refusal is words.

use turbojet::burner_hold::Hold;
use turbojet::matcher::Branch;
use turbojet::sandbox::call;
use turbojet::sandbox_controls::{build, controls_precheck, ControlsSettings, ControlsSolver, SpeedLaw, DS, GAIN_MAX, TAU_MIN};
use turbojet::visuals::Json;

fn run(s: &ControlsSettings) -> Json {
    Json::parse(&call(&format!(r#"{{"op":"controls","controls":{}}}"#, s.to_json().dump_compact())))
}
fn refused(s: &ControlsSettings, words: &str) {
    let r = run(s);
    assert_eq!(r.get("ok"), Some(&Json::Int(0)), "{s:?}");
    match r.get("reason") { Some(Json::Str(m)) => assert!(m.contains(words), "{m}"), o => panic!("{o:?}") }
}
fn ran(s: &ControlsSettings) {
    let r = run(s);
    assert_eq!(r.get("ok"), Some(&Json::Int(1)), "{s:?}: {:?}", r.get("reason"));
}

#[test]
fn the_default_is_the_design_mach_and_a_missing_key_the_earlier_page() {
    let d = ControlsSettings::defaults();
    assert_eq!(d.m0, d.design_flight().m0);
    assert_eq!(d.flight().m0, 0.85);
    let bare = call(r#"{"op":"controls","controls":{"redline_on":1,"accel_on":1}}"#);
    let named = call(r#"{"op":"controls","controls":{"redline_on":1,"accel_on":1,"M0":0.85}}"#);
    assert_eq!(bare, named);
    let s = ControlsSettings { m0: 0.47, ..d };
    assert_eq!(ControlsSettings::from_json(&s.to_json()).unwrap(), s);
}

/// The engine reads only its inlet: flown at Mach 0.5 in air chosen so the inlet's total temperature and pressure are
/// the design flight's, both nozzles choked, every shaft speed, airflow and fuel is the design flight's — only the
/// thrust differs (a lower flight speed, a higher back-pressure). If a solve read the design Mach, they would not match.
#[test]
fn the_engine_reads_only_its_inlet() {
    let d = ControlsSettings::defaults();
    let b0 = build(&d).unwrap();
    let (tt2, pt2, _) = b0.core.fuel.inner.inlet(&b0.flight);
    let g = b0.start.close.gas(&b0.core.fuel.inner.inner).gamma_c();
    let m = 0.5;
    let k = 1.0 + 0.5 * (g - 1.0) * m * m;
    let s = ControlsSettings { m0: m, t0: tt2 / k, p0: pt2 / (b0.core.fuel.inner.inner.base.pi_d_max * k.powf(g / (g - 1.0))), ..d };
    let b = build(&s).unwrap();
    let (tt2b, pt2b, _) = b.core.fuel.inner.inlet(&b.flight);
    assert!((tt2b / tt2 - 1.0).abs() < 1e-13 && (pt2b / pt2 - 1.0).abs() < 1e-13, "the same inlet");
    for (x, y) in [(&b0.start, &b.start), (&b0.end, &b.end)] {
        assert!(x.branch == Branch::Choked && y.branch == Branch::Choked);
        assert!((x.nu_lp - y.nu_lp).abs() < 1e-10 && (x.nu_hp - y.nu_hp).abs() < 1e-10);
        assert!((y.close.mdot_air / x.close.mdot_air - 1.0).abs() < 1e-10);
        assert!(((y.close.f * y.close.mdot_air) / (x.close.f * x.close.mdot_air) - 1.0).abs() < 1e-10);
        assert!((ControlsSolver::thrust(y) / ControlsSolver::thrust(x) - 1.0).abs() > 0.05, "the thrust does differ");
    }
}

/// The hardware is not re-designed at the Mach flown: the design throttle (1500 K) is the design point — both shafts at
/// speed 1 — at Mach 0.85 only.
#[test]
fn the_hardware_stays_captured_at_the_design_mach() {
    let d = ControlsSettings { to: 1500.0, ..ControlsSettings::defaults() };
    let e = build(&d).unwrap().end;
    assert!((e.nu_hp - 1.0).abs() < 1e-9 && (e.nu_lp - 1.0).abs() < 1e-9);
    for m in [0.3, 1.4] {
        let e = build(&ControlsSettings { m0: m, ..d }).unwrap().end;
        assert!((e.nu_hp - 1.0).abs() > 1e-3, "Mach {m}: the 1500 K point moved off design speed ({})", e.nu_hp);
    }
}

/// The view's march is the core's at the flight flown (its Mach included), bit for bit; and flying slowly from a low
/// throttle the nozzle unchokes (rung 97's branch, drawn dashed on the page).
#[test]
fn the_march_runs_at_the_mach_flown() {
    let s = ControlsSettings { m0: 0.3, ..ControlsSettings::defaults() };
    let b = build(&s).unwrap();
    assert_eq!(b.flight.m0, 0.3);
    let nu0 = (b.start.nu_lp, b.start.nu_hp);
    let own = b.core.fuel.integrate_fuel(&b.flight, b.settings.schedule(b.mf_lo, b.mf_hi), nu0, s.s_end(), DS, &b.limiters());
    let mine = b.march();
    assert_eq!(mine.len(), own.len());
    assert!(mine.iter().zip(&own).all(|(p, q)| p.nu_hp.to_bits() == q.nu_hp.to_bits() && p.nu_lp.to_bits() == q.nu_lp.to_bits()));
    let slow = build(&ControlsSettings { m0: 0.05, from: 700.0, ..ControlsSettings::defaults() }).unwrap();
    assert_eq!(slow.start.branch, Branch::Subsonic, "flying slowly the 700 K start has the nozzle unchoked");
}

#[test]
fn the_governors_limits_scale_with_the_density_of_the_air_taken_in() {
    let d = ControlsSettings { govern: true, ..ControlsSettings::defaults() };
    let (slow, fast) = (ControlsSettings { m0: 0.3, ..d }, ControlsSettings { m0: 1.6, ..d });
    assert!(slow.density_ratio() < 0.8 && fast.density_ratio() > 1.5, "{} {}", slow.density_ratio(), fast.density_ratio());
    let hp_prop = |s: ControlsSettings, g: f64| ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: g, ..s };
    let hp_rate = |s: ControlsSettings, t: f64| ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: t, ..s };
    refused(&hp_prop(fast, GAIN_MAX), "the air the engine takes in is");
    ran(&hp_prop(fast, GAIN_MAX / fast.density_ratio() * 0.999));
    refused(&hp_rate(slow, TAU_MIN), "as dense as at the design flight");
    ran(&hp_rate(slow, TAU_MIN / slow.density_ratio() * 1.001));
}

/// Roadmap A3: the two-shaft engine STANDING STILL (Mach 0) is the limit of slow flight. Flown at Mach 0 and at Mach
/// 1e-6 — the latter through the unchanged path, every check and all — both steady ends agree (shaft speeds to 1e-8,
/// thrust to the ram drag's 1e-6) and so does every point of the march (to 1e-6, see below). Over the opening move, both airflow levers with the
/// aircraft's air, cooling and generator, the governor holding thrust, and the afterburner lit. (A census of every
/// page check request at the two speeds, 2026-11-12, found no request that runs at one and not the other: plan § 49.)
#[test]
fn the_two_shaft_engine_standing_still_is_the_limit_of_slow_flight() {
    use turbojet::afterburner::AbMode;
    let rel = |a: f64, b: f64| (a - b).abs() / a.abs().max(b.abs());
    let d = ControlsSettings::defaults();
    let loaded = ControlsSettings::from_json(&Json::parse(
        r#"{"lever":"both","customer":"share","customer_share":0.02,"cool_vane":0.03,"cool_blade":0.02,"generator":"hp","generator_power":0.05}"#)).unwrap();
    for s in [d, loaded, ControlsSettings { govern: true, hold: Hold::Thrust, ..d }, ControlsSettings { ab: Some(AbMode::Lit), ..d }] {
        let at = |m0: f64| build(&ControlsSettings { m0, ..s }).unwrap_or_else(|e| panic!("Mach {m0}: {} ({s:?})", e.0));
        let (still, slow) = (at(0.0), at(1e-6));
        for (a, b) in [(&still.start, &slow.start), (&still.end, &slow.end)] {
            assert!(rel(a.nu_hp, b.nu_hp) < 1e-8 && rel(a.nu_lp, b.nu_lp) < 1e-8, "{s:?}");
            assert!(rel(ControlsSolver::thrust(a), ControlsSolver::thrust(b)) < 1e-6, "{s:?}");
        }
        let (p, q) = (still.march(), slow.march());
        assert_eq!(p.len(), q.len(), "{s:?}");
        let worst = p.iter().zip(&q).enumerate().map(|(i, (x, y))| (rel(x.nu_hp, y.nu_hp).max(rel(x.nu_lp, y.nu_lp)), i))
            .fold((0.0, 0), |a, b| if b.0 > a.0 { b } else { a });
        // Not the steady ends' 1e-8: the march's inner fuel closures stop at their own tolerances, so two flights whose
        // inlets differ by 1e-12 land on iterates a few 1e-8 apart mid-ramp (measured 2.4e-8, the opening move). The
        // bar is the thrust's — a standing engine wrongly formed would miss by percent.
        assert!(worst.0 < 1e-6, "march off by {:.2e} at point {} of {}: {s:?}", worst.0, worst.1, p.len());
    }
    ran(&ControlsSettings { m0: 0.0, t0: 288.15, p0: 101_325.0, ..d });
}

#[test]
fn every_new_refusal_is_words() {
    let d = ControlsSettings::defaults();
    refused(&ControlsSettings { m0: -0.2, ..d }, "Mach number cannot be below zero");
    let nan = controls_precheck(&ControlsSettings { m0: f64::NAN, ..d }).unwrap_err().0;
    assert!(nan.contains("M0 is not a number"), "{nan}");
}

/// The crash map's one first-step redline stop (seed 29, #379: governed on thrust with both levers, the redline on, Mach
/// 1.56 at 14.7 km): at these shaft speeds even the least fuel that solves burns hotter than the redline. It stops at
/// once with its own words, and runs to the end with the redline raised to 1650 K or switched off.
#[test]
fn a_redline_below_the_least_fuels_temperature_stops_in_its_own_words() {
    let req = r#"{"op":"controls","controls":{"T0":245.64999999999998,"p0":12695.766315741415,"shape":"flat-lp","lever":"both","bleed_mode":"schedule","valve_speed":"instant","redline_on":1,"gov_lag_on":0,"accel_on":0,"floor_on":0,"floor_ref":"incidence","floor_phi":0.62,"release_on":0,"govern":1,"hold":"thrust","thrust_by":"true","speed_law":"rate","gov_tau":0.43,"gov_gain":17,"generator":"hp","generator_power":0.095,"customer":"share","customer_share":0.02,"customer_amount":0.02,"cool_vane":0,"cool_blade":0,"metal_on":1,"from":1260,"to":1250,"ramp":0.7000000000000001,"settle":5.9,"rho":2.9000000000000004,"M0":1.56}}"#;
    let r = Json::parse(&call(req));
    assert_eq!(r.get("ok"), Some(&Json::Int(1)));
    assert_eq!(r.get("stopped"), Some(&Json::Int(1)));
    let stop = r.get("stop").unwrap();
    assert_eq!(stop.get("kind"), Some(&Json::Str("redline_floor".into())));
    match stop.get("words") { Some(Json::Str(w)) => assert!(w.contains("Raise the redline"), "{w}"), o => panic!("{o:?}") }
    for other in [req.replace(r#""redline_on":1"#, r#""redline_on":1,"redline":1650.0"#), req.replace(r#""redline_on":1"#, r#""redline_on":0"#)] {
        let r = Json::parse(&call(&other));
        assert_eq!(r.get("stopped"), Some(&Json::Int(0)), "{other}");
    }
}
