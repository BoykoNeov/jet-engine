//! The sandbox's rung-96 joints (`docs/plans/sandbox-plan.md` § 25): on the Controls view a governed thrust can be READ
//! from the pressure ratio or the fan speed through the control's table; read directly, the page is rung 95's run bit
//! for bit; the reading, the thrust the control believes and the table reach the page; every refusal is words.

use turbojet::burner_hold::Hold;
use turbojet::sandbox::call;
use turbojet::sandbox_controls::{ControlsSettings, GeneratorMode, SpeedLaw, ThrustBy, GAIN_MAX_LP};
use turbojet::visuals::Json;

fn run(s: &ControlsSettings) -> Json {
    Json::parse(&call(&format!(r#"{{"op":"controls","controls":{}}}"#, s.to_json().dump_compact())))
}
fn refused(s: &ControlsSettings, words: &str) {
    let r = run(s);
    assert_eq!(r.get("ok"), Some(&Json::Int(0)), "{s:?}");
    match r.get("reason") { Some(Json::Str(m)) => assert!(m.contains(words), "{m}"), o => panic!("{o:?}") }
}
fn floats(j: &Json, k: &str) -> Vec<f64> {
    match j.get(k) {
        Some(Json::List(v)) => v.iter().map(|x| match x { Json::Float(f) => *f, o => panic!("{k}: {o:?}") }).collect(),
        o => panic!("{k}: {o:?}"),
    }
}
fn d() -> ControlsSettings { ControlsSettings { govern: true, hold: Hold::Thrust, ..ControlsSettings::defaults() } }

#[test]
fn read_directly_the_page_is_rung95s_and_says_nothing_of_a_reading() {
    let a = run(&d());
    let b = run(&ControlsSettings { table_clean: false, gov_tau: 0.3, ..d() });
    for k in ["s", "Tt4", "nu_lp", "nu_hp", "fuel", "thrust", "holder", "governor", "demand"] {
        assert_eq!(a.get(k), b.get(k), "{k}");
    }
    for k in ["reading", "thrust_demand", "table"] {
        assert_eq!(a.get(k), Some(&Json::Null), "{k}");
    }
    assert_eq!(a.get("calibrated"), Some(&Json::Int(0)));
}

#[test]
fn inferred_the_page_draws_the_reading_the_belief_and_the_table() {
    for by in [ThrustBy::Epr, ThrustBy::N1] {
        let r = run(&ControlsSettings { thrust_by: by, generator: GeneratorMode::Hp, ..d() });
        assert_eq!(r.get("ok"), Some(&Json::Int(1)));
        assert_eq!(r.get("reading"), Some(&Json::from(by.key())));
        assert_eq!(r.get("calibrated"), Some(&Json::Int(1)));
        let (t, f, dem) = (floats(&r, "thrust"), floats(&r, "thrust_inferred"), floats(&r, "thrust_demand"));
        assert_eq!((t.len(), f.len()), (dem.len(), dem.len()));
        // through the acceleration the engine gives more than the control believes (H1/H3), and the page shows it
        assert!((1..=25).all(|k| t[k] > f[k]));
        match r.get("table") { Some(Json::List(rows)) => assert_eq!(rows.len(), 13), o => panic!("{o:?}") }
        // calibrated on the clean engine with a generator: each end says what it gives against what was asked
        let end = r.get("end").unwrap();
        let (Some(Json::Float(want)), Some(Json::Float(got))) = (end.get("hold_target"), end.get("hold_achieved")) else { panic!() };
        assert!(got != want);
    }
}

#[test]
fn every_inferred_refusal_is_words() {
    let e = ControlsSettings { thrust_by: ThrustBy::Epr, ..d() };
    refused(&ControlsSettings { hold_cap: true, metal_on: true, cool_vane: 0.04, ..e }, "infers thrust");
    let n = ControlsSettings { thrust_by: ThrustBy::N1, ..d() };
    refused(&ControlsSettings { gov_tau: 0.03, ..n }, "fan-speed governor's time constant");
    refused(&ControlsSettings { speed_law: SpeedLaw::Prop, gov_gain: GAIN_MAX_LP * 1.01, ..n }, "fan-speed governor's gain");
    refused(&ControlsSettings { speed_law: SpeedLaw::Prop, gov_gain: 0.0, ..n }, "gain");
    // an unknown reading is refused in words by the settings reader
    let r = Json::parse(&call(r#"{"op":"controls","controls":{"thrust_by":"torque"}}"#));
    match r.get("reason") { Some(Json::Str(m)) => assert!(m.contains("unknown thrust reading"), "{m}"), o => panic!("{o:?}") }
}
