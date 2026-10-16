//! The sandbox's rung-94 joints (`docs/plans/sandbox-plan.md` § 23): on the Controls view each end of the move holds
//! thrust or HP speed with the generator on, the burner temperature floating; holding the burner temperature, or any
//! hold with no generator, is the earlier run bit for bit; the metal cap stops the burner at its limit; refusals are
//! words.

use turbojet::burner_hold::Hold;
use turbojet::sandbox::call;
use turbojet::sandbox_controls::{ControlsSettings, GeneratorMode};
use turbojet::visuals::Json;

fn run(s: &ControlsSettings) -> Json {
    Json::parse(&call(&format!(r#"{{"op":"controls","controls":{}}}"#, s.to_json().dump_compact())))
}
fn f(j: &Json, k: &str) -> f64 {
    match j.get(k) { Some(Json::Float(x)) => *x, Some(Json::Int(n)) => *n as f64, o => panic!("{k}: {o:?}") }
}
fn same_run(a: &Json, b: &Json) {
    for k in ["s", "Tt4", "nu_lp", "nu_hp", "fuel", "phi_lp", "phi_hp", "thrust"] {
        assert_eq!(a.get(k), b.get(k), "{k}");
    }
}

#[test]
fn holding_the_burner_temperature_or_with_no_generator_is_the_earlier_run() {
    let d = ControlsSettings::defaults();
    // No generator: every hold is the plain run.
    let plain = run(&d);
    for h in [Hold::Thrust, Hold::HpSpeed] {
        same_run(&run(&ControlsSettings { hold: h, ..d }), &plain);
    }
    // Generator on, holding the burner temperature: rung 92's run, and its ends say they held nothing.
    let g = ControlsSettings { generator: GeneratorMode::Hp, ..d };
    let r = run(&g);
    same_run(&run(&ControlsSettings { hold: Hold::BurnerTemp, ..g }), &r);
    assert_eq!(r.get("start").unwrap().get("hold_target"), Some(&Json::Null));
}

#[test]
fn a_thrust_hold_meets_the_unloaded_engines_thrust_at_both_ends() {
    let d = ControlsSettings::defaults();
    let bare = run(&d);
    for gen in [GeneratorMode::Hp, GeneratorMode::Lp] {
        let r = run(&ControlsSettings { generator: gen, hold: Hold::Thrust, ..d });
        assert_eq!(r.get("ok"), Some(&Json::Int(1)), "{}", r.dump_compact());
        for (end, set) in [("start", d.from), ("end", d.to)] {
            let (e, b) = (r.get(end).unwrap(), bare.get(end).unwrap());
            assert_eq!(f(e, "hold_set"), set);
            assert!((f(e, "thrust") / f(b, "thrust") - 1.0).abs() < 1e-9, "{gen:?} {end}: {} vs {}", f(e, "thrust"), f(b, "thrust"));
            assert!((f(e, "hold_target") - f(b, "thrust")).abs() < 1e-9 * f(b, "thrust"));
            assert!(f(e, "Tt4") > set, "{gen:?} {end}: the burner runs hotter to carry the load");
            println!("{gen:?} {end}: Tt4 {set} -> {:.2} K; phi_LP {:.4} (unloaded {:.4}), phi_HP {:.4} ({:.4})", f(e, "Tt4"),
                     f(e, "phi_lp"), f(b, "phi_lp"), f(e, "phi_hp"), f(b, "phi_hp"));
        }
    }
}

#[test]
fn the_metal_cap_stops_the_burner_at_the_limit_and_reports_the_shortfall() {
    let d = ControlsSettings::defaults();
    // A big HP load with light cooling: the thrust hold's final end needs more burner than the metal allows.
    let s = ControlsSettings { generator: GeneratorMode::Hp, generator_power: 0.08, hold: Hold::Thrust, metal_on: true,
                               hold_cap: true, cool_vane: 0.03, cool_blade: 0.02, ..d };
    let r = run(&s);
    assert_eq!(r.get("ok"), Some(&Json::Int(1)), "{}", r.dump_compact());
    let e = r.get("end").unwrap();
    assert_eq!(e.get("hold_capped"), Some(&Json::Int(1)), "{}", e.dump_compact());
    assert!(f(e, "hold_achieved") < f(e, "hold_target"), "a capped end falls short");
    let lim = r.get("metal_limit").unwrap();
    let over = (f(e, "metal_vane") - f(lim, "vane")).max(f(e, "metal_blade") - f(lim, "blade"));
    assert!(over.abs() < 1e-6, "the capped end sits at its metal limit: {over}");
    // Uncapped, the same end runs hotter than the cap allowed.
    let free = run(&ControlsSettings { hold_cap: false, ..s });
    assert!(f(free.get("end").unwrap(), "Tt4") > f(e, "Tt4"));
}

#[test]
fn the_cap_and_the_holds_refuse_in_words() {
    let d = ControlsSettings { generator: GeneratorMode::Hp, ..ControlsSettings::defaults() };
    let reason = |s: ControlsSettings| match run(&s).get("reason") { Some(Json::Str(m)) => m.clone(), o => panic!("{o:?}") };
    assert!(reason(ControlsSettings { hold_cap: true, metal_on: true, cool_vane: 0.04, ..d }).contains("floats"));
    assert!(reason(ControlsSettings { hold: Hold::Thrust, hold_cap: true, ..d }).contains("needs the metal read"));
    assert!(reason(ControlsSettings { hold: Hold::Thrust, hold_cap: true, metal_on: true, ..d }).contains("no turbine cooling"));
    // An LP-shaft load whose least thrust (about 142 N, from ~919 K) is above the 800 K throttle's (about 132 N): the hold
    // names the least it allows.
    let m = reason(ControlsSettings { generator: GeneratorMode::Lp, generator_power: 0.05, hold: Hold::Thrust, from: 800.0, ..d });
    assert!(m.contains("starting throttle") && m.contains("below the least this load allows"), "{m}");
    let r = Json::parse(&call(r#"{"op":"controls","controls":{"hold":"altitude"}}"#));
    assert!(matches!(r.get("reason"), Some(Json::Str(m)) if m.contains("unknown hold")));
}
