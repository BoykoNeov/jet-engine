//! Rung 109 on the sandbox's Design view (`src/sandbox.rs`): the turbofan's reply to the page.
//!
//! What these hold it to, and where each value comes from:
//! - bypass ratio 0 — the turbojet's reply, byte for byte (the model's own reduce is `tests/rung109.rs`);
//! - the model's own turbofan run (`turbofan::build_turbofan(…).try_finish`) — the reply is a VIEW of it;
//! - the thrust bookkeeping `F = (F_core + α·F_fan)·ṁ_core`, an identity;
//! - roadmap A6's one-jet split (`sandbox::equivalent_jet`), which the two-jet split must reduce to at α = 0, bit for bit;
//! - the best fan's own definition (no runnable fan pressure ratio burns less per thrust than it).

use turbojet::sandbox::{call, equivalent_jet, equivalent_jets, run, GasModel, NozzleMode, Settings};
use turbojet::visuals::Json;

fn num(j: &Json, k: &str) -> f64 {
    match j.get(k) {
        Some(Json::Float(x)) => *x,
        Some(Json::Int(n)) => *n as f64,
        other => panic!("{k}: {other:?}"),
    }
}

fn ask(settings: &str) -> Json { Json::parse(&call(&format!(r#"{{"op":"run","settings":{settings}}}"#))) }

fn list(j: &Json, k: &str) -> Vec<Json> {
    match j.get(k) { Some(Json::List(v)) => v.clone(), other => panic!("{k}: {other:?}") }
}

#[test]
fn bypass_ratio_zero_is_the_turbojets_reply_byte_for_byte() {
    for gas in ["perfect", "equilibrium"] {
        for nz in ["expanded", "convergent"] {
            let base = format!(r#"{{"gas":"{gas}","nozzle":"{nz}"}}"#);
            let zero = format!(r#"{{"gas":"{gas}","nozzle":"{nz}","alpha":0,"pi_f":2.5,"eta_f":0.8,"pi_fn":0.95}}"#);
            let (a, b) = (call(&format!(r#"{{"op":"run","settings":{base}}}"#)),
                          call(&format!(r#"{{"op":"run","settings":{zero}}}"#)));
            // The fan's other knobs are echoed (they are off their defaults), so compare all but the settings.
            let (ja, jb) = (Json::parse(&a), Json::parse(&b));
            assert!(ja.get("fan").is_none() && ja.get("ts_fan").is_none(), "{gas} {nz}: a turbojet reply grew a fan");
            for k in ja.keys() {
                if k != "settings" { assert_eq!(ja.at(k).dump(), jb.at(k).dump(), "{gas} {nz}: {k}"); }
            }
            assert_eq!(ja.keys(), jb.keys());
        }
    }
    // The default reply carries no fan key at all.
    let d = ask("{}");
    for k in ["alpha", "pi_f", "eta_f", "e_f", "pi_fn"] { assert!(d.at("settings").get(k).is_none(), "{k}"); }
    // …but the page's opening values do, so it can show and share them.
    let defaults = Json::parse(&call(r#"{"op":"defaults"}"#));
    assert_eq!(num(&defaults, "alpha"), 0.0);
    assert_eq!(num(&defaults, "pi_f"), Settings::defaults().pi_f);
}

#[test]
fn the_reply_is_the_models_turbofan_run() {
    for gas in GasModel::ALL {
        for nozzle in [NozzleMode::Expanded, NozzleMode::Convergent] {
            let s = Settings { alpha: 6.0, gas, nozzle, ..Settings::defaults() };
            let o = run(&s).unwrap();
            let fo = o.fan.as_ref().expect("a fan");
            let j = Json::parse(&call(&format!(r#"{{"op":"run","settings":{}}}"#, s.to_json().dump_compact())));
            let fj = j.at("fan");
            let tr = &fo.result;
            // The thrust: (core + α·fan) per kg of core air, times the core's airflow.
            let want = (tr.core_thrust + 6.0 * tr.fan_thrust) * s.mdot;
            let got = num(&j, "thrust");
            assert!(((got - want) / want).abs() < 1e-12, "{gas:?} {nozzle:?}: thrust {got} vs {want}");
            assert_eq!(num(&j, "specific_thrust").to_bits(), tr.core.performance.specific_thrust.to_bits());
            assert_eq!(num(&j, "tsfc").to_bits(), tr.core.performance.tsfc.to_bits());
            assert_eq!(num(fj, "mdot_bypass"), 6.0 * s.mdot);
            assert_eq!(num(fj, "V19").to_bits(), tr.fan.as_ref().unwrap().exit.v9.to_bits());
            // Stations 13 and 19 follow the core's, with no fuel.
            let st = list(&j, "stations");
            let labels: Vec<String> = st.iter().map(|x| match x.at("label") { Json::Str(s) => s.clone(), _ => panic!() }).collect();
            assert_eq!(labels, ["0", "2", "3", "4", "5", "9", "13", "19"], "{gas:?}");
            assert_eq!(num(&st[6], "far"), 0.0);
            // The fan raises the bypass air's entropy (a lossy fan) and its jet leaves cooler than the fan exit.
            let ts = list(&j, "ts"); let tf = list(&j, "ts_fan");
            assert!(num(&tf[0], "s") > num(&ts[1], "s"), "{gas:?}: 13 not right of 2");
            assert!(num(&tf[1], "T") < num(&tf[0], "T"));
        }
    }
}

#[test]
fn the_two_jet_split_reduces_to_the_one_jet_split() {
    for (fs, v0, f) in [(900.0, 250.0, 0.02), (1100.0, 0.0, 0.025), (450.0, 300.0, 0.018)] {
        let one = equivalent_jet(fs, v0, f, 43e6);
        let (two, _) = equivalent_jets(fs, 123.0, 0.0, v0, f, 43e6);
        for (a, b) in [(one.v_eq, two.v_eq), (one.eta_thermal, two.eta_thermal), (one.eta_propulsive, two.eta_propulsive)] {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }
    // Fully expanded, each equivalent speed IS its jet's speed; overall = thermal × propulsive.
    let j = ask(r#"{"alpha":5,"gas":"perfect"}"#);
    let fj = j.at("fan");
    assert!((num(fj, "V_eq19") / num(fj, "V19") - 1.0).abs() < 1e-12);
    assert!((num(&j, "V_eq") / num(&j, "V9") - 1.0).abs() < 1e-12);
    let prod = num(&j, "eta_thermal_eq") * num(&j, "eta_propulsive_eq");
    assert!((prod / num(&j, "eta_overall") - 1.0).abs() < 1e-12, "{prod}");
    // The two-jet thermal efficiency counts both jets: the model's speed-only one agrees when fully expanded.
    assert!((num(&j, "eta_thermal_eq") / num(&j, "eta_thermal") - 1.0).abs() < 1e-10);
}

#[test]
fn the_best_fan_and_the_chart_agree_with_the_run() {
    for gas in [GasModel::Perfect, GasModel::Equilibrium] {
        for m0 in [0.85, 0.0] {
            let s = Settings { alpha: 6.0, pi_f: 1.4, gas, m0, ..Settings::defaults() };
            let o = run(&s).unwrap();
            let fo = o.fan.as_ref().unwrap();
            let best = fo.best.expect("a best fan");
            assert!(best.tsfc <= fo.result.core.performance.tsfc, "{gas:?} M{m0}: the best burns more than yours");
            assert!((1.0..=4.0).contains(&best.pi_f));
            // The chart: α 0 is the turbojet, α 6 at your fan IS this run, bit for bit; every best ≤ every current.
            let jet = run(&Settings { alpha: 0.0, ..s }).unwrap();
            assert_eq!(fo.chart.len(), 13);
            assert_eq!(fo.chart[0].current.unwrap().0.to_bits(), jet.result.performance.tsfc.to_bits());
            let c6 = fo.chart[6].current.unwrap();
            assert_eq!(c6.0.to_bits(), fo.result.core.performance.tsfc.to_bits());
            assert_eq!(c6.1.to_bits(), fo.result.thrust_per_core().to_bits());
            assert_eq!(fo.chart[6].best.unwrap().0.to_bits(), best.pi_f.to_bits());
            for p in &fo.chart {
                if let (Some(c), Some(b)) = (p.current, p.best) { assert!(b.1 <= c.0, "α {}", p.alpha); }
            }
            // More bypass, each at its best: less fuel per thrust, through α 4 at least (the rung's P1).
            let b: Vec<f64> = fo.chart[..=4].iter().map(|p| p.best.unwrap().1).collect();
            assert!(b.windows(2).all(|w| w[1] < w[0]), "{gas:?} M{m0}: {b:?}");
        }
    }
}

#[test]
fn the_refusals_are_in_words() {
    for (settings, words) in [
        (r#"{"alpha":10,"pi_f":3}"#, "cannot pay for this fan"),
        (r#"{"alpha":6,"pi_f":1.02,"M0":0}"#, "The fan's pressure rise"),
        (r#"{"alpha":6,"afterburner":1}"#, "afterburner"),
        (r#"{"alpha":20}"#, "bypass ratio"),
        (r#"{"alpha":3,"pi_f":5}"#, "fan pressure ratio"),
    ] {
        let j = ask(settings);
        assert_eq!(j.get("ok"), Some(&Json::Int(0)), "{settings} ran");
        let Some(Json::Str(r)) = j.get("reason") else { panic!("{settings}: no reason") };
        assert!(r.contains(words), "{settings}: {r}");
        assert!(!r.contains("Model message"), "{settings}: a refusal fell through to the panic words: {r}");
    }
    let fly = Json::parse(&call(r#"{"op":"fly","fly":{"design":{"alpha":4}}}"#));
    assert_eq!(fly.get("ok"), Some(&Json::Int(0)));
    let Some(Json::Str(r)) = fly.get("reason") else { panic!() };
    assert!(r.contains("turbofan"), "{r}");
}
