//! The sandbox page's JOINTS (`docs/sandbox/`), as `tests/visuals.rs` checks the other two pages.
//!
//! These hold the page's TEXT to the model's code. They cannot see whether the embedded browser
//! build is CURRENT, nor run the page — `rust/sandbox-wasm/check.ps1` does both, in the gate
//! (`rust/test-all.ps1`): it rebuilds the build and compares it with the committed page, checks the
//! build against the native model on a grid, and drives the page in a headless Chrome.

use turbojet::sandbox::{base64, call, splice_page, unbase64, GasModel, NozzleMode, Settings, WASM_PLACEHOLDER};
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
    // `$("v-" + id)` / `$("d-" + id)` for the four tiles.
    for tile in ["thrust", "st", "tsfc", "eo"] {
        looked_up.push(format!("v-{tile}"));
        looked_up.push(format!("d-{tile}"));
    }
    assert!(looked_up.len() > 40, "census too small: {}", looked_up.len());
    for id in looked_up {
        assert!(declared.contains(&id.as_str()), "the script looks up #{id}, which the template never declares");
    }
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
    assert_eq!(options("nozzle"), nozzles);
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
