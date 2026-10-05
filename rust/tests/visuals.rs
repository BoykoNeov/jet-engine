//! PHASE 8, slice AR — **the two published pages are the MODEL's, and they are what the build
//! actually produces** (`docs/plans/todo-rust-port.md` § 8.9).
//!
//! The Rust twin of `tests/test_visuals_data.py`'s fourteen gates, one `#[test]` per Python test
//! and under the same name, plus what the port adds:
//!
//! * `data_json_is_the_models_byte_for_byte` — the WHOLE of `docs/visuals/data.json` regenerated
//!   and compared byte for byte. The Python could only afford the cycle blocks (its sweeps took
//!   ~10 minutes, measured 94 s under PyPy); the Rust run is ~20 s, so the sweeps are gated too —
//!   the failure mode the Python file's docstring names first ("a cycle change ... leaves
//!   `data.json` on yesterday's numbers") is now caught for every block, not five.
//! * (retired at slice AU) `keep_is_build_cutaways` held the Rust `KEEP` equal to
//!   `build_cutaway.py`'s while both existed; the script is deleted, so the Rust list is the only
//!   one, and `the_trim_drops_nothing_unnamed` / `every_field_the_cutaway_reads_survives_the_trim`
//!   still bind it to the data and the page.
//! * `the_ts_diagram_is_what_plot_ts_diagram_drew` — every array `main.py`'s chart handed to
//!   matplotlib, bit for bit (`rust/oracle/ts_diagram_pypy.tsv`, recorded at the call in slice AK).
//!
//! **The hand-written censuses.** The crate has no regex dependency, so each Python `re` pattern
//! below is a small matcher. A matcher can drift from its regex in silence (`\b` on a non-ASCII
//! letter, a non-greedy block end), so each census is PINNED to the exact set Python's own pattern
//! found on the committed templates when this was written (`W:\temp\claude\phase8-ar\census.py`) —
//! strictly stronger than the Python's five-name self-check, and the price is that a legitimate
//! template edit re-pins here, on purpose.
//!
//! **No test here writes into `docs/visuals/`.** The pages are built by `turbojet visuals` /
//! `turbojet splice`; these tests compute in memory and compare.

use std::path::{Path, PathBuf};

use turbojet::panels::{flight, real_losses, Design, PI_C, TT4};
use turbojet::visuals::{self, cutaway_payload, Json, KEEP, SWEEP_BLOCKS};

fn vis_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("docs").join("visuals")
}

/// A committed text file as Python's `read_text` sees it — universal newlines. The working copy
/// of `template.html` and `turbojet-visuals.html` is CRLF (git stores LF), so a byte comparison
/// would report a difference that is not one.
fn text(name: &str) -> String {
    let p = vis_dir().join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display())).replace("\r\n", "\n")
}

fn data_text() -> String { text("data.json") }
fn data() -> Json { Json::parse(&data_text()) }

// ------------------------------------------------------------------- the hand-written censuses

fn is_word(c: char) -> bool { c.is_alphanumeric() || c == '_' }
fn is_ident_start(c: char) -> bool { c.is_ascii_alphabetic() || c == '_' }
fn is_ident(c: char) -> bool { c.is_ascii_alphanumeric() || c == '_' }

/// `\b` at char index `i` before a word character: the previous char is not a word char.
fn boundary_before(s: &[char], i: usize) -> bool { i == 0 || !is_word(s[i - 1]) }

/// `[A-Za-z_][A-Za-z_0-9]*` starting at `i`; returns its end, or `None`.
fn ident_at(s: &[char], i: usize) -> Option<usize> {
    if i < s.len() && is_ident_start(s[i]) {
        let mut j = i + 1;
        while j < s.len() && is_ident(s[j]) {
            j += 1;
        }
        Some(j)
    } else {
        None
    }
}

fn starts(s: &[char], i: usize, lit: &str) -> bool {
    let l: Vec<char> = lit.chars().collect();
    i + l.len() <= s.len() && s[i..i + l.len()] == l[..]
}

/// `re.findall(r"\b(?:D\(\)|DATA|[IRd])\.([A-Za-z_][A-Za-z_0-9]*)", src)` as a set.
fn read_census(src: &str) -> std::collections::BTreeSet<String> {
    let s: Vec<char> = src.chars().collect();
    let mut out = std::collections::BTreeSet::new();
    let mut i = 0;
    while i < s.len() {
        if is_word(s[i]) && boundary_before(&s, i) {
            // The alternatives in the regex's order; each must be followed by `.` + identifier.
            let heads: &[&str] = &["D()", "DATA", "I", "R", "d"];
            let mut matched = None;
            for h in heads {
                if starts(&s, i, h) && starts(&s, i + h.chars().count(), ".") {
                    let k = i + h.chars().count() + 1;
                    if let Some(e) = ident_at(&s, k) {
                        matched = Some((k, e));
                        break;
                    }
                }
            }
            if let Some((k, e)) = matched {
                out.insert(s[k..e].iter().collect());
                i = e;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// `[A-Za-z0-9_-]+` starting at `i`; returns its end, or `None`.
fn id_token(s: &[char], i: usize) -> Option<usize> {
    let mut j = i;
    while j < s.len() && (s[j].is_ascii_alphanumeric() || s[j] == '_' || s[j] == '-') {
        j += 1;
    }
    (j > i).then_some(j)
}

/// `getElementById\((['"])([A-Za-z0-9_-]+)\1\)` — the looked-up ids, as a set.
fn lookups(src: &str) -> std::collections::BTreeSet<String> {
    let s: Vec<char> = src.chars().collect();
    let mut out = std::collections::BTreeSet::new();
    for i in 0..s.len() {
        if starts(&s, i, "getElementById(") {
            let q = i + "getElementById(".len();
            if q < s.len() && (s[q] == '\'' || s[q] == '"') {
                if let Some(e) = id_token(&s, q + 1) {
                    if e + 1 < s.len() && s[e] == s[q] && s[e + 1] == ')' {
                        out.insert(s[q + 1..e].iter().collect());
                    }
                }
            }
        }
    }
    out
}

/// `\bid=["']([A-Za-z0-9_-]+)["']` — the declared ids, as a set (the quotes need not match).
fn declared_ids(src: &str) -> std::collections::BTreeSet<String> {
    let s: Vec<char> = src.chars().collect();
    let mut out = std::collections::BTreeSet::new();
    for i in 0..s.len() {
        if starts(&s, i, "id=") && boundary_before(&s, i) {
            let q = i + 3;
            if q < s.len() && (s[q] == '"' || s[q] == '\'') {
                if let Some(e) = id_token(&s, q + 1) {
                    if e < s.len() && (s[e] == '"' || s[e] == '\'') {
                        out.insert(s[q + 1..e].iter().collect());
                    }
                }
            }
        }
    }
    out
}

/// `re.search(open + "(.*?)" + close, src, re.S).group(1)` for literal `open`/`close`.
fn block<'a>(src: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let a = src.find(open)? + open.len();
    let b = src[a..].find(close)?;
    Some(&src[a..a + b])
}

/// `re.findall(r"([A-Za-z_][A-Za-z_0-9]*):", body)` as a set. The regex's match at a run of word
/// characters followed by `:` starts at the run's first letter/underscore (a leading digit is
/// skipped, never a later letter), and a run not followed by `:` never matches.
fn colon_keys(body: &str) -> std::collections::BTreeSet<String> {
    let s: Vec<char> = body.chars().collect();
    let mut out = std::collections::BTreeSet::new();
    let mut i = 0;
    while i < s.len() {
        if is_ident(s[i]) {
            let start = i;
            while i < s.len() && is_ident(s[i]) {
                i += 1;
            }
            if i < s.len() && s[i] == ':' {
                if let Some(k) = (start..i).find(|&k| is_ident_start(s[k])) {
                    out.insert(s[k..i].iter().collect());
                }
            }
        } else {
            i += 1;
        }
    }
    out
}

/// `re.findall(r"\['([0-9]+)'", body)`, in order.
fn stn_labels(body: &str) -> Vec<String> {
    let s: Vec<char> = body.chars().collect();
    let mut out = Vec::new();
    for i in 0..s.len() {
        if starts(&s, i, "['") {
            let mut j = i + 2;
            while j < s.len() && s[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 2 && j < s.len() && s[j] == '\'' {
                out.push(s[i + 2..j].iter().collect());
            }
        }
    }
    out
}

fn set(items: &[&str]) -> std::collections::BTreeSet<String> { items.iter().map(|s| s.to_string()).collect() }

// ---------------------------------------------------------------------------------- 1. splices

#[test]
fn cutaway_html_is_the_current_splice() {
    let expected = visuals::splice_cutaway(&text("cutaway-template.html"), &data());
    assert!(expected == text("turbojet-cutaway.html"),
            "docs/visuals/turbojet-cutaway.html is not the splice of its template and data.json — \
             hand-edited, or a template/data change committed without `cargo run --release -- splice`. \
             Rebuild, then republish the artifact (memory/cutaway-artifact.md).");
}

#[test]
fn visuals_html_is_the_current_splice() {
    let expected = visuals::splice_visuals(&text("template.html"), &data_text());
    assert!(expected == text("turbojet-visuals.html"),
            "docs/visuals/turbojet-visuals.html is not the splice of its template and data.json — \
             hand-edited, built without `cargo run --release -- splice`, or data.json was REFORMATTED \
             (the page splices it verbatim). Rebuild, then republish the artifact (memory/visuals-artifact.md).");
}

// ---------------------------------------------------------------- 2. the design point is ONE

/// The twin of `test_extract_data_imports_the_design_point_rather_than_copying_it`: the generator
/// takes the design point from `panels` (where `main.py`'s constants live) and declares none of
/// its own.
#[test]
fn extract_data_imports_the_design_point_rather_than_copying_it() {
    let src = include_str!("../src/visuals.rs");
    assert!(src.contains("use crate::panels::{flight, real_losses, Design, PI_C, TT4};"),
            "src/visuals.rs no longer imports the design point from crate::panels");
    for decl in ["const PI_C", "const TT4", "fn flight(", "fn real_losses(", "FlightCondition::new", "Losses {"] {
        assert!(!src.contains(decl),
                "src/visuals.rs declares `{decl}` itself — a second design point the pages could \
                 describe under the same name");
    }
}

#[test]
fn data_json_design_point_is_main_pys() {
    let d = data();
    let dp = d.at("design");
    let f = flight();
    let num = |k: &str| match dp.at(k) {
        Json::Float(x) => *x,
        other => panic!("data.json design.{k} is {other:?}, not a float"),
    };
    assert_eq!((num("T0").to_bits(), num("p0").to_bits(), num("M0").to_bits()),
               (f.t0.to_bits(), f.p0.to_bits(), f.m0.to_bits()),
               "docs/visuals/data.json's flight condition is not main.py's (panels::flight())");
    assert_eq!((num("pi_c").to_bits(), num("Tt4").to_bits()), (PI_C.to_bits(), TT4.to_bits()),
               "docs/visuals/data.json's design point is not main.py's");
    let l = real_losses();
    let want = [("pi_d", l.pi_d), ("eta_c", l.eta_c), ("eta_b", l.eta_b), ("pi_b", l.pi_b),
                ("eta_t", l.eta_t), ("eta_m", l.eta_m), ("pi_n", l.pi_n)];
    let losses = dp.at("losses");
    assert_eq!(losses.keys(), want.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
               "data.json's losses are not REAL_LOSSES's keys, in its order");
    for (k, v) in want {
        assert_eq!(losses.at(k), &Json::Float(v), "data.json losses.{k} != main.py's REAL_LOSSES");
    }
}

// ------------------------------------------------------------ 3. the blocks are the model's

/// Stronger than the Python's 1e-5 relative bar: the cycle blocks are regenerated and compared as
/// TEXT, so every rounding `r()` performs is held too. Cheap (two CPG design runs).
#[test]
fn data_json_cycle_blocks_match_the_live_model() {
    let committed = data();
    for (name, block) in visuals::cycle_blocks(&Design::new()) {
        let want = committed.at(&name).dump();
        let got = block.dump();
        assert!(got == want, "data.json['{name}'] is not the model's:\n  committed: {}\n  model:     {}",
                &want[..want.len().min(400)], &got[..got.len().min(400)]);
    }
}

#[test]
fn data_json_is_the_models_byte_for_byte() {
    let model = visuals::build_data(&Design::new(), &mut |_| {});
    let got = model.dump();
    let want = data_text();
    if got != want {
        let committed = Json::parse(&want);
        for k in model.keys() {
            let (a, b) = (committed.get(k).map(Json::dump), model.at(k).dump());
            assert!(a.as_deref() == Some(b.as_str()), "data.json['{k}'] differs from the model:\n  committed: {}\n  model:     {}",
                    a.as_deref().unwrap_or("<missing>"), b);
        }
        panic!("data.json differs from the model outside any block (key order or a missing block): \
                committed {:?} vs model {:?}", committed.keys(), model.keys());
    }
}

/// The reader and the writer are each other's inverse on the committed file — which is what lets
/// the splice gates parse `data.json` and the cutaway re-serialise it.
#[test]
fn the_json_writer_round_trips_the_committed_file() {
    let t = data_text();
    assert_eq!(Json::parse(&t).dump(), t);
}

// -------------------------------------------- 4. what the page READS survives the trim

const NOT_A_CYCLE_FIELD: [&str; 3] = ["design", "ideal", "real"];
/// The cutaway's perf table shows the three efficiencies a reader can act on; the Brayton bound
/// is the charts page's story, not the cutaway's.
const DELIBERATELY_DROPPED: [&str; 1] = ["eta_brayton"];

fn fields_the_cutaway_reads() -> std::collections::BTreeSet<String> {
    let mut f = read_census(&text("cutaway-template.html"));
    for n in NOT_A_CYCLE_FIELD {
        f.remove(n);
    }
    f
}

#[test]
fn the_read_census_can_actually_see() {
    // Python's `_READ_RE` on the committed template found exactly these (census.py).
    let python = set(&["M9", "T9", "V0", "V9", "design", "eta_overall", "eta_propulsive", "eta_thermal",
                       "ideal", "legs", "points", "real", "specific_thrust", "stations", "tsfc"]);
    assert_eq!(read_census(&text("cutaway-template.html")), python,
               "the read census no longer finds what Python's regex found — either the template \
                changed (re-pin from the regex) or this matcher drifted from it");
}

#[test]
fn every_field_the_cutaway_reads_survives_the_trim() {
    let payload = cutaway_payload(&data());
    for field in fields_the_cutaway_reads() {
        for case in ["ideal", "real"] {
            assert!(payload.at(case).get(&field).is_some(),
                    "cutaway-template.html reads `{field}`, but KEEP drops it from '{case}' — the page \
                     renders undefined/NaN there");
        }
    }
}

#[test]
fn the_trim_drops_nothing_unnamed() {
    for k in DELIBERATELY_DROPPED {
        assert!(!KEEP.contains(&k), "{k} is both KEPT and listed as deliberately dropped");
    }
    let d = data();
    for case in ["ideal", "real"] {
        let fields = d.at(case).keys();
        let unaccounted: Vec<_> = fields.iter().filter(|f| !KEEP.contains(f) && !DELIBERATELY_DROPPED.contains(f)).collect();
        assert!(unaccounted.is_empty(), "data.json['{case}'] carries {unaccounted:?}, neither KEPT nor named as dropped");
        let missing: Vec<_> = KEEP.iter().filter(|k| !fields.contains(k)).collect();
        assert!(missing.is_empty(), "KEEP names {missing:?}, which data.json['{case}'] does not have");
    }
}

#[test]
fn the_cutaway_renders_its_design_point_from_the_data() {
    let tpl = text("cutaway-template.html");
    assert!(tpl.contains("function paintChrome()") && tpl.contains("paintChrome();"),
            "cutaway-template.html no longer defines/calls paintChrome()");
    let d = data();
    let design = d.at("design");
    for field in ["M0", "T0", "p0", "pi_c", "Tt4", "losses"] {
        assert!(design.get(field).is_some(), "data.json['design'] has no `{field}`, which paintChrome() reads");
    }
    let labels = block(&tpl, "const LOSS_LABEL = {", "};").expect("cutaway-template.html no longer declares LOSS_LABEL");
    let known = colon_keys(labels);
    assert_eq!(known, set(&["eta_b", "eta_c", "eta_m", "eta_t", "pi_b", "pi_d", "pi_n"]),
               "the LOSS_LABEL census no longer finds what Python's regex found (census.py)");
    for loss in design.at("losses").keys() {
        assert!(known.contains(loss), "data.json's losses carry `{loss}`, which LOSS_LABEL does not name");
    }
}

/// The design-point spellings that used to be typed into `template.html`'s prose.
const TYPED_DESIGN_POINT: [&str; 5] =
    ["M<sub>0</sub>=0.85", "π<sub>c</sub>=10", "T<sub>t4</sub>=1500", "η<sub>c</sub>=0.88", "η<sub>t</sub>=0.90"];

#[test]
fn the_charts_page_renders_its_design_point_from_the_data() {
    let tpl = text("template.html");
    assert!(tpl.contains("function paintDesignPoint()") && tpl.contains("\npaintDesignPoint();"),
            "template.html no longer defines/calls paintDesignPoint()");
    for span in ["ts-design", "footer-design", "lede-eta"] {
        assert!(tpl.contains(&format!("id=\"{span}\"")), "template.html has no `id={span}` for paintDesignPoint() to fill");
    }
    let typed: Vec<_> = TYPED_DESIGN_POINT.iter().filter(|t| tpl.contains(*t)).collect();
    assert!(typed.is_empty(), "template.html types the design point again as {typed:?}");
    let d = data();
    let design = d.at("design");
    for field in ["M0", "pi_c", "Tt4", "losses"] {
        assert!(design.get(field).is_some(), "data.json['design'] has no `{field}`, which paintDesignPoint() reads");
    }
    for loss in ["eta_c", "eta_t"] {
        assert!(design.at("losses").get(loss).is_some(), "data.json's losses have no `{loss}`, which the T-s lede renders");
    }
}

#[test]
fn every_element_the_pages_look_up_actually_exists() {
    // What Python's two regexes found on the committed templates (census.py): 15 + 21 lookups.
    let pinned = [
        ("cutaway-template.html", set(&["btn-ideal", "btn-play", "btn-real", "cbar", "chips", "cutaway", "legend",
                                        "losses", "perf", "probe", "readouts", "speed", "speed-out", "stations", "ts"]), 18),
        ("template.html", set(&["aft-chart", "bell-chart", "dwell-chart", "engine-canvas", "hero-tiles", "jsweep-chart",
                                "ladder-chart", "ladder-fig", "lede-eta", "m-chart", "quenchT-chart", "ratio-chart",
                                "rung-list", "spatial-C", "spatial-J", "spatial-chart", "station-chips",
                                "station-readout", "station-table", "tauq-chart", "ts-chart"]), 47),
    ];
    for (name, python_lookups, python_declared) in pinned {
        let src = text(name);
        let looked = lookups(&src);
        assert_eq!(looked, python_lookups, "{name}: the getElementById census no longer finds what Python's regex found");
        let declared = declared_ids(&src);
        assert_eq!(declared.len(), python_declared, "{name}: the id= census no longer finds Python's count");
        let missing: Vec<_> = looked.iter().filter(|i| !declared.contains(*i)).collect();
        assert!(missing.is_empty(), "{name} looks up {missing:?} but declares no such id");
    }
}

#[test]
fn the_cutaway_station_labels_all_exist() {
    let tpl = text("cutaway-template.html");
    let body = block(&tpl, "const STN = [", "];").expect("cutaway-template.html no longer declares `const STN = [...]`");
    let labels = stn_labels(body);
    assert_eq!(labels, ["0", "2", "3", "4", "5", "9"], "the STN census no longer finds what Python's regex found");
    let payload = cutaway_payload(&data());
    for case in ["ideal", "real"] {
        for l in &labels {
            let s = payload.at(case).at("stations").get(l)
                .unwrap_or_else(|| panic!("the cutaway draws station {l}, but data.json['{case}'] has none"));
            for f in ["Tt", "pt", "far"] {
                assert!(s.get(f).is_some(), "station {l} in '{case}' has no {f}");
            }
        }
    }
}

// ------------------------------------------------------- 5. the expensive blocks, shape only

fn finite_leaves(v: &Json, path: &str, n: &mut usize) {
    match v {
        Json::Obj(kv) => kv.iter().for_each(|(k, x)| finite_leaves(x, &format!("{path}.{k}"), n)),
        Json::List(xs) => {
            assert!(!xs.is_empty(), "{path} is an empty list");
            xs.iter().enumerate().for_each(|(i, x)| finite_leaves(x, &format!("{path}[{i}]"), n));
        }
        Json::Float(x) => {
            assert!(x.is_finite(), "{path} is {x} — a non-finite number reached data.json");
            *n += 1;
        }
        Json::Int(_) => *n += 1,
        Json::Null | Json::Str(_) => {}
    }
}

#[test]
fn sweep_blocks_present_and_finite() {
    let d = data();
    for b in SWEEP_BLOCKS {
        let blk = d.get(b).unwrap_or_else(|| panic!("data.json has no '{b}' block"));
        let mut n = 0;
        finite_leaves(blk, b, &mut n);
        assert!(n > 0, "data.json['{b}'] carries no numbers at all");
    }
}

#[test]
fn data_json_has_no_unexpected_top_level_blocks() {
    let mut expected: Vec<&str> = vec!["design", "ideal", "real"];
    expected.extend(SWEEP_BLOCKS);
    assert_eq!(data().keys(), expected, "data.json's top-level blocks changed");
}

// ------------------------------------------------------------------------------- 6. the rounding

#[test]
#[should_panic(expected = "negative ndigits")]
fn r_refuses_the_unported_negative_ndigits_branch() {
    visuals::r(1.5e6, 6);
}

// ------------------------------------------------------------------------------ 7. the T–s chart

const TS_TSV: &str = include_str!("../oracle/ts_diagram_pypy.tsv");

/// The recorded calls: `(kind, series-or-text, bit patterns)`, grouped by call index.
fn ts_calls() -> Vec<Vec<(String, String, Vec<u64>)>> {
    let mut calls: Vec<Vec<(String, String, Vec<u64>)>> = Vec::new();
    for line in TS_TSV.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        let idx: usize = f[0].parse().unwrap();
        let n: usize = f[3].parse().unwrap();
        // A `style` row carries the call's keyword arguments as text (`tab:blue|--|1.8|0.7`), not
        // bits; the slimmed script owns those, and the PNG comparison (§ 8.9) is what holds them.
        let bits: Vec<u64> = if f[2] == "style" {
            Vec::new()
        } else {
            f[4..].iter().map(|s| u64::from_str_radix(s, 16).unwrap()).collect()
        };
        if f[2] != "style" {
            assert_eq!(bits.len(), n, "row {line:?}: count field disagrees with its data");
        }
        if calls.len() <= idx {
            calls.resize(idx + 1, Vec::new());
        }
        calls[idx].push((f[1].to_string(), f[2].to_string(), bits));
    }
    calls
}

fn bits(v: &[f64]) -> Vec<u64> { v.iter().map(|x| x.to_bits()).collect() }

#[test]
fn the_ts_diagram_is_what_plot_ts_diagram_drew() {
    let ts = visuals::ts_diagram(&Design::new());
    // main.py's call order: per cycle (ideal, then real) two work legs and two isobars as `plot`,
    // then the six stations as `scatter`; then the six real-cycle labels as `annotate`.
    let mut want: Vec<(&str, Vec<(String, Vec<u64>)>)> = Vec::new();
    for c in [&ts.ideal, &ts.real] {
        for (s, t) in c.work_legs.iter().chain(c.isobars.iter()) {
            want.push(("plot", vec![("x".into(), bits(s)), ("y".into(), bits(t))]));
        }
        for &(_, s, t) in &c.points {
            want.push(("scatter", vec![("x".into(), bits(&[s])), ("y".into(), bits(&[t]))]));
        }
    }
    for &(l, s, t) in &ts.real.points {
        want.push(("annotate", vec![(format!("'  {l}'"), bits(&[s, t]))]));
    }
    let got = ts_calls();
    assert_eq!(got.len(), 26, "the oracle should hold 26 calls");
    assert_eq!(want.len(), got.len(), "the port draws a different number of calls");
    for (i, ((kind, series), rec)) in want.iter().zip(&got).enumerate() {
        let rec: Vec<_> = rec.iter().filter(|(_, s, _)| s != "style").collect();
        assert_eq!(rec.len(), series.len(), "call {i}: series count");
        for ((name, b), (k, s, rb)) in series.iter().zip(rec) {
            assert_eq!((kind, name), (&k.as_str(), s), "call {i}: kind/series");
            assert_eq!(b, rb, "call {i} ({kind} {name}): not the bits plot_ts_diagram drew");
        }
    }
    // The title is the one main.py set — now rendered from the design point, not typed.
    assert_eq!(ts.title, "Turbojet T–s diagram — ideal vs real components\n\
                          (M0=0.85, π_c=10, Tt4=1500 K; real: η_c=0.88, η_t=0.90, π losses)");
}

/// The chart JSON carries the exact bits: every float survives `repr` → parse.
#[test]
fn the_ts_json_round_trips_the_bits() {
    let ts = visuals::ts_diagram(&Design::new());
    let j = visuals::ts_diagram_json(&ts);
    let back = Json::parse(&j.dump());
    assert_eq!(back, j);
    assert_eq!(back.at("title"), &Json::Str(ts.title.clone()));
}

// ------------------------------------------------------------------------------------------
// SLICE AT — `plot_ts_diagram.py`'s READ CENSUS. After the delete at slice AU that script is the
// only Python left, and it reads `ts_diagram.json` by key with no test of its own: rename a key in
// `ts_diagram_json` and it breaks with every gate green (slice AR booked this, plan § 8.9). So the
// script's subscripts are read off its source and each must be a key the JSON actually writes.
//
// What the census can and cannot see, said here so it is not mistaken for more:
// * it reads LITERAL subscripts, `["k"]` and `['k']` (the f-string's `pt['label']` is the second
//   form). Any other way of reading — `.get(`, `.items()`, `.keys()`, `.values()`, `**`,
//   `getattr` — is REFUSED outright, so a later edit cannot route around the census silently;
// * it checks NAMES, not paths: `s` and `T` are written at three depths, so a rename at one depth
//   alone would pass. That is the honest scope of a name census and is not claimed beyond it.
// ------------------------------------------------------------------------------------------

const TS_SCRIPT: &str = include_str!("../../plot_ts_diagram.py");

/// Every literal-string subscript in `src`, in order of first appearance.
fn script_keys(src: &str) -> Vec<String> {
    let b = src.as_bytes();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'[' && (b[i + 1] == b'"' || b[i + 1] == b'\'') {
            let q = b[i + 1];
            if let Some(len) = b[i + 2..].iter().position(|&c| c == q) {
                let end = i + 2 + len;
                if b.get(end + 1) == Some(&b']') {
                    let k = &src[i + 2..end];
                    if !out.iter().any(|x| x == k) {
                        out.push(k.to_string());
                    }
                    i = end + 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// Every object key anywhere in `j`.
fn json_keys(j: &Json, acc: &mut Vec<String>) {
    match j {
        Json::Obj(kv) => {
            for (k, v) in kv {
                if !acc.contains(k) {
                    acc.push(k.clone());
                }
                json_keys(v, acc);
            }
        }
        Json::List(xs) => xs.iter().for_each(|x| json_keys(x, acc)),
        _ => {}
    }
}

/// The census: the script keys the JSON does NOT write (empty = clean).
fn unwritten_reads(src: &str, j: &Json) -> Vec<String> {
    let mut have = Vec::new();
    json_keys(j, &mut have);
    script_keys(src).into_iter().filter(|k| !have.contains(k)).collect()
}

/// A copy of `j` with every object key `from` renamed `to` — the defect the census exists for.
fn rename(j: &Json, from: &str, to: &str) -> Json {
    match j {
        Json::Obj(kv) => Json::Obj(
            kv.iter()
                .map(|(k, v)| (if k == from { to.to_string() } else { k.clone() }, rename(v, from, to)))
                .collect(),
        ),
        Json::List(xs) => Json::List(xs.iter().map(|x| rename(x, from, to)).collect()),
        other => other.clone(),
    }
}

#[test]
fn the_ts_script_reads_only_keys_the_json_writes() {
    for form in [".get(", ".items(", ".keys(", ".values(", "**", "getattr"] {
        assert!(
            !TS_SCRIPT.contains(form),
            "plot_ts_diagram.py reads the JSON through `{form}`, which this census cannot follow — \
             extend the census before shipping that form"
        );
    }
    // PINNED: the census sees exactly the reads the script makes today, so a scanner that went
    // blind (zero keys) or a script that grew a read is a deliberate re-pin, not a quiet pass.
    let keys = script_keys(TS_SCRIPT);
    assert_eq!(
        keys,
        ["work_legs", "s", "T", "isobars", "points", "ideal", "real", "label", "title"],
        "the script's literal subscripts, in first-appearance order"
    );
    let j = visuals::ts_diagram_json(&visuals::ts_diagram(&Design::new()));
    assert_eq!(unwritten_reads(TS_SCRIPT, &j), Vec::<String>::new(), "the script reads keys the JSON does not write");
    // THE INSTRUMENT, SHOWN TO SEE: a renamed key is reported, under either quote style.
    assert_eq!(unwritten_reads(TS_SCRIPT, &rename(&j, "work_legs", "legs")), vec!["work_legs"]);
    assert_eq!(unwritten_reads(TS_SCRIPT, &rename(&j, "label", "name")), vec!["label"]);
}
