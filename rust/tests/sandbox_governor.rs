//! The sandbox's rung-95 joints (`docs/plans/sandbox-plan.md` § 24): on the Controls view the governor holds the
//! throttle's quantity all along the move; off, the page is the earlier run bit for bit; its clamp and demand reach the
//! page; every refusal is words.

use turbojet::burner_hold::Hold;
use turbojet::sandbox::call;
use turbojet::sandbox_controls::{ControlsSettings, GeneratorMode, LeverChoice, BleedMode, SpeedLaw, GAIN_MAX};
use turbojet::visuals::Json;

fn run(s: &ControlsSettings) -> Json {
    Json::parse(&call(&format!(r#"{{"op":"controls","controls":{}}}"#, s.to_json().dump_compact())))
}
fn refused(s: &ControlsSettings, words: &str) {
    let r = run(s);
    assert_eq!(r.get("ok"), Some(&Json::Int(0)), "{s:?}");
    match r.get("reason") { Some(Json::Str(m)) => assert!(m.contains(words), "{m}"), o => panic!("{o:?}") }
}
fn list(j: &Json, k: &str) -> Vec<Json> {
    match j.get(k) { Some(Json::List(v)) => v.clone(), o => panic!("{k}: {o:?}") }
}

#[test]
fn off_the_page_is_the_earlier_run_and_says_nothing_of_a_governor() {
    let d = ControlsSettings::defaults();
    let a = run(&d);
    let b = run(&ControlsSettings { gov_tau: 0.4, gov_gain: 9.0, speed_law: SpeedLaw::Prop, ..d });
    for k in ["s", "Tt4", "nu_lp", "nu_hp", "fuel", "phi_lp", "phi_hp", "thrust", "holder"] {
        assert_eq!(a.get(k), b.get(k), "{k}");
    }
    assert_eq!(a.get("governor"), Some(&Json::Null));
    assert_eq!(a.get("demand"), Some(&Json::Null));
    assert_eq!(a.get("route"), Some(&Json::from("plain")));
}

#[test]
fn governed_the_page_draws_the_demand_the_clamp_and_the_cutting_leg() {
    let d = ControlsSettings::defaults();
    let r = run(&ControlsSettings { govern: true, hold: Hold::Thrust, redline_on: true, ..d });
    assert_eq!(r.get("route"), Some(&Json::from("governor")));
    let (demand, thrust) = (list(&r, "demand"), list(&r, "thrust"));
    let (gov, holder) = (list(&r, "governor"), list(&r, "holder"));
    assert_eq!(demand.len(), thrust.len());
    // where the governor holds the fuel, thrust IS the demand; the redline cuts somewhere, and the governor is clamped
    // only while it does
    let mut held = 0;
    for k in 0..gov.len() {
        if gov[k] == Json::from("rich") {
            assert_ne!(holder[k], Json::from("none"), "a rich clamp only under a cutting leg");
        }
        if holder[k] == Json::from("none") {
            let (Json::Float(dm), Json::Float(t)) = (&demand[k], &thrust[k]) else { panic!() };
            assert!((t / dm - 1.0).abs() < 1e-12);
            held += 1;
        }
    }
    assert!(held > 0 && holder.iter().any(|h| *h == Json::from("redline")));
    // the metal leg reaches the page as a holder
    let m = run(&ControlsSettings { govern: true, hold: Hold::Thrust, metal_on: true, hold_cap: true, cool_vane: 0.04,
                                    cool_blade: 0.02, ..d });
    assert!(list(&m, "holder").iter().any(|h| *h == Json::from("metal")));
}

#[test]
fn every_refusal_is_words() {
    let d = ControlsSettings { govern: true, ..ControlsSettings::defaults() };
    // Rung 106 lifted the temperature limiter's lag beside the governor; what it still refuses there:
    refused(&ControlsSettings { redline_on: true, gov_lag_on: true, gov_lag: 0.05, ..d }, "at least 0.1 spool time constants");
    refused(&ControlsSettings { redline_on: true, gov_lag_on: true, from: 1500.0, to: 1000.0, ..d }, "above the temperature limiter's");
    refused(&ControlsSettings { accel_on: true, release_on: true, ..d }, "realistic release");
    refused(&ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, valve_lag_on: true, ..d },
            "instant valve");
    refused(&ControlsSettings { hold: Hold::HpSpeed, gov_tau: 0.03, ..d }, "time constant");
    refused(&ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: GAIN_MAX * 1.01, ..d }, "gain");
    refused(&ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 0.0, ..d }, "gain");
    // the cap still needs a floating hold, the metal read, and cooling — but not the generator
    refused(&ControlsSettings { hold_cap: true, metal_on: true, cool_vane: 0.04, ..d }, "burner temperature itself");
    refused(&ControlsSettings { hold: Hold::Thrust, hold_cap: true, cool_vane: 0.04, ..d }, "metal read");
    let ok = run(&ControlsSettings { hold: Hold::Thrust, hold_cap: true, metal_on: true, cool_vane: 0.04, ..d });
    assert_eq!(ok.get("ok"), Some(&Json::Int(1)));
    // ungoverned, rung 94's rule stands
    refused(&ControlsSettings { govern: false, hold: Hold::Thrust, hold_cap: true, metal_on: true, cool_vane: 0.04,
                                ..d }, "generator off nothing");
    let _ = GeneratorMode::Off;
}
