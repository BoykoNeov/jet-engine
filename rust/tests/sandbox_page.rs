//! The sandbox page's JOINTS (`docs/sandbox/`), as `tests/visuals.rs` checks the other two pages.
//!
//! These hold the page's TEXT to the model's code. They cannot see whether the embedded browser
//! build is CURRENT, nor run the page — `rust/sandbox-wasm/check.ps1` does both, in the gate
//! (`rust/test-all.ps1`): it rebuilds the build and compares it with the committed page, checks the
//! build against the native model on a grid, and drives the page in a headless Chrome.

use turbojet::blade_speed::SHAPES;
use turbojet::sandbox::{base64, call, splice_page, unbase64, FlySettings, GasModel, MapShape, NozzleMode, Settings,
                        WASM_PLACEHOLDER};
use turbojet::sandbox_blades::{lever_key, BladeGas, BladeSettings};
use turbojet::visuals::Json;

fn repo(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// Every `open … close` span in `text`, in order.
fn spans<'a>(text: &'a str, open: &str, close: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find(open) {
        rest = &rest[i + open.len()..];
        let j = rest.find(close).expect("unclosed span");
        out.push(&rest[..j]);
        rest = &rest[j + close.len()..];
    }
    out
}

const TEMPLATE: &str = "docs/sandbox/template.html";
const PAGE: &str = "docs/sandbox/turbojet-sandbox.html";

#[test]
fn base64_matches_the_rfc_vectors_and_round_trips() {
    for (plain, coded) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foob", "Zm9vYg=="),
                           ("fooba", "Zm9vYmE="), ("foobar", "Zm9vYmFy")] {
        assert_eq!(base64(plain.as_bytes()), coded);
        assert_eq!(unbase64(coded), plain.as_bytes());
    }
    let all: Vec<u8> = (0..=255u8).chain((0..=255u8).rev()).collect();
    assert_eq!(unbase64(&base64(&all)), all);
}

#[test]
fn the_built_page_is_the_splice_of_its_template() {
    let page = read(PAGE);
    let coded = spans(&page, "const WASM_B64 = \"", "\";");
    assert_eq!(coded.len(), 1, "the page must embed exactly one browser build");
    assert!(coded[0].len() > 100_000, "the embedded build is implausibly small: {} chars", coded[0].len());
    assert!(splice_page(&read(TEMPLATE), &unbase64(coded[0])) == page,
               "docs/sandbox/turbojet-sandbox.html is not its template + its build: a hand edit, or a template \
                change without `rust/sandbox-wasm/build.ps1`");
}

#[test]
fn the_template_holds_the_placeholder_once() {
    assert_eq!(read(TEMPLATE).matches(WASM_PLACEHOLDER).count(), 1);
}

#[test]
fn every_element_the_script_looks_up_is_declared() {
    let t = read(TEMPLATE);
    let declared: Vec<&str> = spans(&t, " id=\"", "\"");
    // Literal lookups only: an id straight after the opening quote, closed by `quote)`. A computed
    // one (`$("v-" + id)`) fails that shape and is listed below by hand.
    let lookups = |open: &str, quote: char| -> Vec<String> {
        t.match_indices(open).filter_map(|(i, _)| {
            let rest = &t[i + open.len()..];
            let id: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect();
            rest[id.len()..].starts_with(&format!("{quote})")).then_some(id).filter(|id| !id.is_empty())
        }).collect()
    };
    let mut looked_up = lookups("$(\"", '"');
    looked_up.extend(lookups("getElementById('", '\''));
    // `$(k)` / `$(k + "-r")` over the NUM list: each knob and its slider, if it has one.
    let num = num_list(&t);
    looked_up.extend(num.iter().cloned());
    looked_up.extend(["altitude", "delta_t", "altitude-r", "delta_t-r"].map(String::from));
    // `$(id)` / `$(id + "-r")` over the FLY_NUM map: the fly view's own knobs and sliders.
    for id in fly_num(&t).into_iter().map(|(id, _)| id) {
        looked_up.push(format!("{id}-r"));
        looked_up.push(id);
    }
    // `$(id)` / `$(id + "-r")` over the blade view's two knob maps.
    for (id, _) in js_map(&t, "BLADE_NUM").into_iter().chain(js_map(&t, "BLADE_KNOB")) {
        looked_up.push(format!("{id}-r"));
        looked_up.push(id);
    }
    // `$(id)` over the id arrays the script hides, shows and greys out as a group.
    looked_up.extend(["cycle-set", "components-set", "nozzle-set", "fly-set", "fly-tiles", "map-card",
                      "out", "ts", "stations", "perf", "map", "flight-set", "ts-card", "stations-card", "perf-card",
                      "blades-out", "bl-machine-card", "bl-stair-card", "bl-lever-card", "st-lp", "st-hp"].map(String::from));
    let view = spans(&t, "const BLADE_VIEW = [", "];");
    assert_eq!(view.len(), 1);
    looked_up.extend(spans(view[0], "\"", "\"").into_iter().map(String::from));
    // The blade view's helpers take the id as their first argument: `click("id", …)`, `press("id", …)`.
    for helper in ["click(\"", "press(\""] {
        let ids: Vec<&str> = spans(&t, helper, "\"");
        assert!(ids.len() >= 6, "{helper}: {ids:?}");
        looked_up.extend(ids.into_iter().map(String::from));
    }
    // `$("v-" + id)` / `$("d-" + id)` for the tiles (the fly view's three computed ones included).
    for tile in ["thrust", "st", "tsfc", "eo", "nu", "pic", "mdot", "klp", "khp", "rlp", "rhp"] {
        looked_up.push(format!("v-{tile}"));
        looked_up.push(format!("d-{tile}"));
    }
    assert!(looked_up.len() > 40, "census too small: {}", looked_up.len());
    for id in looked_up {
        assert!(declared.contains(&id.as_str()), "the script looks up #{id}, which the template never declares");
    }
}

/// A knob map of the script, `const NAME = { "id": "setting", … };`, as (id, setting) pairs.
fn js_map(t: &str, name: &str) -> Vec<(String, String)> {
    let body = spans(t, &format!("const {name} = {{"), "};");
    assert_eq!(body.len(), 1, "{name}");
    let q: Vec<&str> = spans(body[0], "\"", "\"");
    assert!(q.len() % 2 == 0 && !q.is_empty(), "{name}: {q:?}");
    q.chunks(2).map(|c| (c[0].to_string(), c[1].to_string())).collect()
}

/// The fly view's knob map, `FLY_NUM`.
fn fly_num(t: &str) -> Vec<(String, String)> { js_map(t, "FLY_NUM") }

fn numeric_keys(j: &Json) -> Vec<String> {
    let mut v: Vec<String> = match j {
        Json::Obj(kv) => kv.iter().filter(|(_, v)| matches!(v, Json::Float(_))).map(|(k, _)| k.clone()).collect(),
        _ => unreachable!(),
    };
    v.sort();
    v
}

#[test]
fn every_blade_knob_is_a_blade_setting_and_every_numeric_one_has_a_knob() {
    let t = read(TEMPLATE);
    let d = BladeSettings::defaults().to_json();
    let mut engine: Vec<String> = js_map(&t, "BLADE_NUM").into_iter().map(|(_, k)| k).collect();
    engine.sort();
    assert_eq!(engine, numeric_keys(&d), "the engine knobs");
    let mut blade: Vec<String> = js_map(&t, "BLADE_KNOB").into_iter().map(|(_, k)| k).collect();
    blade.sort();
    assert_eq!(blade, numeric_keys(d.get("lp").unwrap()), "the blade knobs (each spool's)");
    assert_eq!(numeric_keys(d.get("lp").unwrap()), numeric_keys(d.get("hp").unwrap()));
}

/// The shared flight knobs, `const FLIGHT = [ … ];`.
fn flight_list(t: &str) -> Vec<String> {
    let body = spans(t, "const FLIGHT = [", "];");
    assert_eq!(body.len(), 1);
    spans(body[0], "\"", "\"").into_iter().map(String::from).collect()
}

#[test]
fn every_fly_knob_is_a_fly_setting_and_every_numeric_fly_setting_has_a_knob() {
    let t = read(TEMPLATE);
    // The fly view's numbers: its own knobs, plus the flight knobs it shares with the design.
    let mut knobs: Vec<String> = fly_num(&t).into_iter().map(|(_, k)| k).chain(flight_list(&t)).collect();
    knobs.sort();
    let mut numeric: Vec<String> = match FlySettings::defaults().to_json() {
        Json::Obj(kv) => kv.into_iter().filter(|(_, v)| matches!(v, Json::Float(_))).map(|(k, _)| k).collect(),
        _ => unreachable!(),
    };
    numeric.sort();
    assert_eq!(knobs, numeric);
    // ...and every shared flight knob is a design knob too (one input, two owners).
    let num = num_list(&t);
    for k in flight_list(&t) { assert!(num.contains(&k), "flight knob {k} is not a design knob"); }
}

fn num_list(t: &str) -> Vec<String> {
    let body = spans(t, "const NUM = [", "];");
    assert_eq!(body.len(), 1);
    spans(body[0], "\"", "\"").into_iter().map(String::from).collect()
}

#[test]
fn every_knob_is_a_setting_and_every_numeric_setting_has_a_knob() {
    let t = read(TEMPLATE);
    let mut knobs = num_list(&t);
    knobs.sort();
    let mut numeric: Vec<String> = match Settings::defaults().to_json() {
        Json::Obj(kv) => kv.into_iter().filter(|(_, v)| matches!(v, Json::Float(_))).map(|(k, _)| k).collect(),
        _ => unreachable!(),
    };
    numeric.sort();
    assert_eq!(knobs, numeric);
}

#[test]
fn the_selects_offer_exactly_the_models_choices() {
    let t = read(TEMPLATE);
    let options = |id: &str| -> Vec<String> {
        let sel = spans(&t, &format!("<select id=\"{id}\""), "</select>");
        assert_eq!(sel.len(), 1, "select #{id}");
        let mut v: Vec<String> = spans(sel[0], "<option value=\"", "\"").into_iter().map(String::from).collect();
        v.sort();
        v
    };
    let mut gases: Vec<String> = GasModel::ALL.iter().map(|g| g.key().to_string()).collect();
    gases.sort();
    let mut nozzles: Vec<String> = NozzleMode::ALL.iter().map(|n| n.key().to_string()).collect();
    nozzles.sort();
    assert_eq!(options("gas"), gases);
    assert_eq!(options("fly-gas"), gases);
    assert_eq!(options("nozzle"), nozzles);
    let mut maps: Vec<String> = MapShape::ALL.iter().map(|m| m.key().to_string()).collect();
    maps.sort();
    assert_eq!(options("fly-map"), maps);
    let mut bg: Vec<String> = BladeGas::ALL.iter().map(|g| g.key().to_string()).collect();
    bg.sort();
    assert_eq!(options("bl-gas"), bg);
    let mut sh: Vec<String> = SHAPES.iter().map(|s| s.to_string()).collect();
    sh.sort();
    assert_eq!(options("bl-shape"), sh);
    let mut lv: Vec<String> = [turbojet::blade_speed::Lever::Lumped, turbojet::blade_speed::Lever::AllRows,
                               turbojet::blade_speed::Lever::FrontRow].iter().map(|l| lever_key(*l).to_string()).collect();
    lv.sort();
    assert_eq!(options("bl-lever"), lv);
}

#[test]
fn every_export_the_javascript_calls_exists_in_the_shell() {
    let shell = read("rust/sandbox-wasm/src/lib.rs");
    let exported: Vec<&str> = spans(&shell, "extern \"C\" fn ", "(");
    assert!(exported.len() >= 8, "exports found: {exported:?}");
    for file in [TEMPLATE, "rust/sandbox-wasm/check.mjs"] {
        let js = read(file);
        let called: Vec<&str> = spans(&js, "ex.", "(").into_iter()
            .filter(|n| n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')).collect();
        assert!(called.len() >= 6, "{file}: calls found {called:?}");
        for name in called {
            assert!(exported.contains(&name), "{file} calls ex.{name}(), which the shell does not export");
        }
    }
}

#[test]
fn every_op_the_page_sends_is_answered() {
    let t = read(TEMPLATE);
    let ops: Vec<&str> = spans(&t, "op: \"", "\"");
    assert!(ops.len() >= 4, "ops found: {ops:?}");
    for op in ops {
        let reply = call(&format!(r#"{{"op":"{op}"}}"#));
        assert!(!reply.contains("unknown op"), "the page sends op {op:?}, which `sandbox::call` does not know");
    }
}
