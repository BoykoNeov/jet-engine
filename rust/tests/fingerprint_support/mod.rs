//! SLICE AS — the numeric fingerprint's MACHINERY, shared by `tests/fingerprint.rs`.
//!
//! `tests/test_numeric_fingerprint.py` is the project's only ABSOLUTE-value gate: 45 kernels
//! compared against a committed CPython golden (`tests/golden/numeric_fingerprint.json`). Plan
//! § 8.1 (vi) re-anchors it on Rust. This module is what that needs and nothing else:
//!
//! * [`V`] — a kernel's scalar leaf, and [`Tree`] — a reader's nested return value, flattened by
//!   [`flat`] exactly as Python's `_flat` does (`.key` for a dict, `#n` + `[i]` for a list,
//!   `.re` / `.im` for a complex).
//! * [`Kernel`] — `{key: V}` with an emission guard ([`Kernel::put`] refuses a duplicate key,
//!   because two emitters writing the same name is a port defect a map would silently resolve).
//! * The TWO goldens: [`load_pypy`] reads `rust/oracle/fingerprint_pypy.tsv` (the bit-exact
//!   target, written by `rust/oracle/dump_fingerprint.py` from the Python module's own `KERNELS`
//!   table) and [`load_cpython`] reads the CPython JSON (`rust/oracle/numeric_fingerprint_cpython.json`,
//!   a byte copy of the one under `tests/golden/`) with a minimal reader for that one file's
//!   shape — no dependency.
//! * [`close`] — Python's `_close`, verbatim: `==` first (so `0.0 == -0.0` and `True == 1`), then
//!   the absolute leg, then the relative one; `tol == abs_tol == 0` is bit-equality.
//!
//! Not a test target: a subdirectory `mod.rs`, pulled in by `mod fingerprint_support;`.

#![allow(dead_code)]

use std::collections::BTreeMap;

/// One leaf. `J` is a JSON-spelled list (r66's `keys` / `edges.*` only — ints or strings).
#[derive(Clone, Debug, PartialEq)]
pub enum V {
    F(f64),
    I(i64),
    B(bool),
    S(String),
    N,
    J(Vec<JItem>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum JItem {
    I(i64),
    S(String),
}

impl V {
    /// The TSV token `dump_fingerprint.py`'s `tok` writes. Floats as IEEE bits, so the PyPy
    /// comparison is bit-equality (a NaN is `f:nan`, as Python's `x != x` branch spells it).
    pub fn token(&self) -> String {
        match self {
            V::F(x) if x.is_nan() => "f:nan".into(),
            V::F(x) => format!("f:{:016x}", x.to_bits()),
            V::I(i) => format!("i:{i}"),
            V::B(b) => format!("b:{}", *b as u8),
            V::S(s) => format!("s:{s}"),
            V::N => "n".into(),
            V::J(items) => {
                // `json.dumps` of a list: `[1, 2]`, `["a", "b"]`, `[]`.
                let parts: Vec<String> = items
                    .iter()
                    .map(|it| match it {
                        JItem::I(i) => i.to_string(),
                        JItem::S(s) => json_string(s),
                    })
                    .collect();
                format!("j:[{}]", parts.join(", "))
            }
        }
    }
}

/// `json.dumps(str)` for the ASCII identifiers r66 pins (`ensure_ascii`, `\"` / `\\` escaped).
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A reader's return value, as Python's dict / list / complex / scalar.
#[derive(Clone, Debug)]
pub enum Tree {
    D(Vec<(String, Tree)>),
    L(Vec<Tree>),
    C(f64, f64),
    X(V),
}

impl Tree {
    pub fn f(x: f64) -> Tree { Tree::X(V::F(x)) }
    pub fn i(x: i64) -> Tree { Tree::X(V::I(x)) }
    pub fn u(x: usize) -> Tree { Tree::X(V::I(x as i64)) }
    pub fn b(x: bool) -> Tree { Tree::X(V::B(x)) }
    pub fn s(x: &str) -> Tree { Tree::X(V::S(x.to_string())) }
    pub fn n() -> Tree { Tree::X(V::N) }
    pub fn of(x: Option<f64>) -> Tree { x.map_or(Tree::n(), Tree::f) }
    pub fn ob(x: Option<bool>) -> Tree { x.map_or(Tree::n(), Tree::b) }
    pub fn fl(xs: &[f64]) -> Tree { Tree::L(xs.iter().map(|&x| Tree::f(x)).collect()) }
    pub fn sl(xs: &[&str]) -> Tree { Tree::L(xs.iter().map(|x| Tree::s(x)).collect()) }
}

/// A dict literal: `d(vec![("k", v), …])`.
pub fn d(items: Vec<(&str, Tree)>) -> Tree {
    Tree::D(items.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

/// One kernel's `{key: value}`.
#[derive(Default, Debug)]
pub struct Kernel(pub BTreeMap<String, V>);

impl Kernel {
    pub fn put(&mut self, k: impl Into<String>, v: V) {
        let k = k.into();
        assert!(!self.0.contains_key(&k), "duplicate fingerprint key {k:?}");
        self.0.insert(k, v);
    }
    pub fn f(&mut self, k: impl Into<String>, x: f64) { self.put(k, V::F(x)) }
    /// `_floats_of`'s rule for an attribute: emitted only when it IS a float — a `None` (an
    /// unarmed diagnostic) is not, and is skipped, which is how Python's `isinstance` reads it.
    pub fn of(&mut self, k: impl Into<String>, x: Option<f64>) {
        if let Some(x) = x {
            self.f(k, x)
        }
    }
}

/// Python's `_flat`: recurse to scalar leaves; a list's LENGTH is pinned as `#n`.
pub fn flat(t: &Tree, prefix: &str, out: &mut Kernel) {
    match t {
        Tree::D(items) => {
            for (k, v) in items {
                flat(v, &format!("{prefix}.{k}"), out);
            }
        }
        Tree::L(xs) => {
            out.put(format!("{prefix}#n"), V::I(xs.len() as i64));
            for (i, v) in xs.iter().enumerate() {
                flat(v, &format!("{prefix}[{i}]"), out);
            }
        }
        Tree::C(re, im) => {
            out.f(format!("{prefix}.re"), *re);
            out.f(format!("{prefix}.im"), *im);
        }
        Tree::X(v) => out.put(prefix.to_string(), v.clone()),
    }
}

/// Rebuild a [`Tree`] from slice AJ's `Flat` stream (`tests/slice_aj_flat/mod.rs`): one
/// `(path, token)` per node in Python's order, a dict written `keys:N` then its N children, a list
/// `len:N` then its N items. That stream is the AJ oracle's own, verified bit for bit against PyPy
/// at slice AJ step 6, so re-reading it here reuses those converters instead of describing every
/// rung-81/82 struct a second time. A child's FIRST line is its own path, so a dict key is the
/// whole remainder after `parent.` — which keeps keys that contain a dot (`clip@0.02`) intact.
pub fn tree_from_flat(lines: &[(String, String)]) -> Tree {
    fn node(lines: &[(String, String)], i: &mut usize, path: &str) -> Tree {
        let (p, tok) = &lines[*i];
        assert_eq!(p, path, "flat stream out of step at line {}", *i);
        *i += 1;
        if let Some(n) = tok.strip_prefix("keys:") {
            let n: usize = n.parse().unwrap();
            let mut items = Vec::with_capacity(n);
            for _ in 0..n {
                let child = lines[*i].0.clone();
                let key = child
                    .strip_prefix(path)
                    .and_then(|r| r.strip_prefix('.'))
                    .unwrap_or_else(|| panic!("{child:?} is not a child of {path:?}"))
                    .to_string();
                let t = node(lines, i, &child);
                items.push((key, t));
            }
            Tree::D(items)
        } else if let Some(n) = tok.strip_prefix("len:") {
            let n: usize = n.parse().unwrap();
            let items = (0..n).map(|k| node(lines, i, &format!("{path}.{k}"))).collect();
            Tree::L(items)
        } else {
            Tree::X(leaf(tok))
        }
    }
    fn leaf(tok: &str) -> V {
        if tok == "n" {
            V::N
        } else if tok == "f:nan" {
            V::F(f64::NAN)
        } else if let Some(h) = tok.strip_prefix("f:") {
            V::F(f64::from_bits(u64::from_str_radix(h, 16).unwrap()))
        } else if let Some(x) = tok.strip_prefix("i:") {
            V::I(x.parse().unwrap())
        } else if let Some(x) = tok.strip_prefix("b:") {
            V::B(x == "1")
        } else if let Some(x) = tok.strip_prefix("s:") {
            V::S(x.to_string())
        } else {
            panic!("unknown flat token {tok:?}")
        }
    }
    let mut i = 0;
    let root = lines[0].0.clone();
    let t = node(lines, &mut i, &root);
    assert_eq!(i, lines.len(), "trailing lines after the root node");
    t
}

// ------------------------------------------------------------------------------- the goldens

pub const PYPY_TSV: &str = include_str!("../../oracle/fingerprint_pypy.tsv");
/// A BYTE-IDENTICAL copy of `tests/golden/numeric_fingerprint.json` (the CPython anchor, left
/// untouched there as the audit record), kept inside `rust/` so this gate compiles with no Python
/// tree present — plan § 8.1 (viii)'s delete proof removes `tests/`.
pub const CPYTHON_JSON: &str = include_str!("../../oracle/numeric_fingerprint_cpython.json");

/// THE RUST ANCHOR — written ONLY by `FINGERPRINT_REGEN=1 cargo test --release --test fingerprint
/// regenerate_anchor`, from Rust's own kernels. Read at RUN time (not `include_str!`) so the
/// regeneration mode can create it.
pub fn anchor_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("oracle").join("fingerprint_rust.tsv")
}

/// The published deviation table, written by the same regeneration run.
pub fn deviation_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("oracle").join("fingerprint_deviation.tsv")
}

pub fn load_anchor() -> BTreeMap<String, BTreeMap<String, String>> {
    let text = std::fs::read_to_string(anchor_path()).unwrap_or_else(|e| panic!(
        "no Rust fingerprint anchor at {:?} ({e}). It is created ONLY deliberately: \
         FINGERPRINT_REGEN=1 cargo test --release --test fingerprint regenerate_anchor",
        anchor_path()));
    parse_tsv(&text)
}

/// `kernel -> key -> token`, from the PyPy capture.
pub fn load_pypy() -> BTreeMap<String, BTreeMap<String, String>> { parse_tsv(PYPY_TSV) }

/// A token back to its value — the inverse of [`V::token`].
pub fn v_from_token(tok: &str) -> V {
    if tok == "n" {
        V::N
    } else if tok == "f:nan" {
        V::F(f64::NAN)
    } else if let Some(h) = tok.strip_prefix("f:") {
        V::F(f64::from_bits(u64::from_str_radix(h, 16).unwrap()))
    } else if let Some(x) = tok.strip_prefix("i:") {
        V::I(x.parse().unwrap())
    } else if let Some(x) = tok.strip_prefix("b:") {
        V::B(x == "1")
    } else if let Some(x) = tok.strip_prefix("s:") {
        V::S(x.to_string())
    } else if let Some(x) = tok.strip_prefix("j:") {
        decode(&parse_json(x))
    } else {
        panic!("unknown token {tok:?}")
    }
}

/// One row of the published deviation table: one kernel, Rust (== the anchor) against CPython.
///
/// `max_used` is the fraction of the kernel's band the worst value consumes, read on whichever
/// leg passes it best — `_close` passes a value if EITHER leg covers it, so that is the leg that
/// decides. `0` when nothing differs; `inf` for a differing DISCRETE value (none exist).
pub struct Deviation {
    pub n: usize,
    pub n_differ: usize,
    pub max_rel: f64,
    pub max_abs: f64,
    pub max_used: f64,
    /// Which leg decided `max_used` — `rel` or `abs` — for the worst value; `-` when none differ.
    pub leg: &'static str,
}

pub fn deviation(name: &str, got: &BTreeMap<String, V>, golden: &BTreeMap<String, V>) -> Deviation {
    let (t, a) = (tol(name), abs_tol(name));
    let mut d = Deviation {
        n: golden.len(), n_differ: 0, max_rel: 0.0, max_abs: 0.0, max_used: 0.0, leg: "-",
    };
    for (k, w) in golden {
        let g = &got[k];
        if py_eq(g, w) {
            continue;
        }
        d.n_differ += 1;
        let (V::F(x), V::F(y)) = (g, w) else {
            d.max_used = f64::INFINITY;
            continue;
        };
        let ab = (x - y).abs();
        let rel = if *y != 0.0 { ab / y.abs() } else { x.abs() };
        d.max_abs = d.max_abs.max(ab);
        d.max_rel = d.max_rel.max(rel);
        let (mut used, mut leg) = (f64::INFINITY, "-");
        if t > 0.0 && rel / t < used {
            (used, leg) = (rel / t, "rel");
        }
        if a > 0.0 && ab / a < used {
            (used, leg) = (ab / a, "abs");
        }
        if used >= d.max_used {
            (d.max_used, d.leg) = (used, leg);
        }
    }
    d
}

pub fn parse_tsv(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let mut it = line.splitn(3, '\t');
        let (k, key, tok) = (it.next().unwrap(), it.next().unwrap(), it.next().unwrap());
        let prev = out.entry(k.to_string()).or_default().insert(key.to_string(), tok.to_string());
        assert!(prev.is_none(), "duplicate PyPy key {k}/{key}");
    }
    out
}

/// Python's `float.fromhex` for what `float.hex` writes: `[-]0x<h>.<hhh>p<±d>`, `inf`, `nan`.
pub fn from_hex(s: &str) -> f64 {
    let (neg, body) = match s.strip_prefix('-') {
        Some(b) => (true, b),
        None => (false, s),
    };
    let v = match body {
        "inf" => f64::INFINITY,
        "nan" => f64::NAN,
        _ => {
            let b = body.strip_prefix("0x").unwrap_or_else(|| panic!("not a hex float: {s:?}"));
            let (mant, exp) = b.split_once('p').unwrap_or_else(|| panic!("no exponent: {s:?}"));
            let exp: i32 = exp.parse().unwrap();
            let (int_part, frac) = mant.split_once('.').unwrap_or((mant, ""));
            let lead = u64::from_str_radix(int_part, 16).unwrap();
            assert!(lead <= 1 && frac.len() <= 13, "unexpected hex float layout {s:?}");
            let frac_bits = if frac.is_empty() {
                0
            } else {
                u64::from_str_radix(frac, 16).unwrap() << (4 * (13 - frac.len()))
            };
            if lead == 0 {
                // zero or subnormal: `float.hex` writes a subnormal as `0x0.<f>p-1022`
                assert!(frac_bits == 0 || exp == -1022, "unexpected subnormal {s:?}");
                f64::from_bits(frac_bits)
            } else {
                let e = (exp + 1023) as u64;
                assert!((1..=2046).contains(&e), "exponent out of range {s:?}");
                f64::from_bits((e << 52) | frac_bits)
            }
        }
    };
    if neg { -v } else { v }
}

/// A JSON value, for the one file this reads.
#[derive(Clone, Debug)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

struct P<'a> {
    b: &'a [u8],
    i: usize,
}

impl P<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && (self.b[self.i] as char).is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn eat(&mut self, c: u8) {
        self.ws();
        assert_eq!(self.b[self.i], c, "JSON: expected {:?} at byte {}", c as char, self.i);
        self.i += 1;
    }
    fn string(&mut self) -> String {
        self.eat(b'"');
        let mut out: Vec<u16> = Vec::new();
        let mut s = String::new();
        loop {
            let c = self.b[self.i];
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = self.b[self.i];
                    self.i += 1;
                    match e {
                        b'u' => {
                            let h = std::str::from_utf8(&self.b[self.i..self.i + 4]).unwrap();
                            out.push(u16::from_str_radix(h, 16).unwrap());
                            self.i += 4;
                            continue;
                        }
                        b'n' => s.push('\n'),
                        b't' => s.push('\t'),
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        b'/' => s.push('/'),
                        _ => panic!("JSON: escape \\{}", e as char),
                    }
                }
                _ => {
                    // flush any pending \u run before a literal byte
                    if !out.is_empty() {
                        s.push_str(&String::from_utf16(&out).unwrap());
                        out.clear();
                    }
                    // raw UTF-8: copy the whole sequence
                    let start = self.i - 1;
                    let len = match c {
                        0x00..=0x7f => 1,
                        0xc0..=0xdf => 2,
                        0xe0..=0xef => 3,
                        _ => 4,
                    };
                    self.i = start + len;
                    s.push_str(std::str::from_utf8(&self.b[start..self.i]).unwrap());
                    continue;
                }
            }
            if !out.is_empty() {
                // an escape other than \u ended a \u run: it was already pushed above, so
                // re-order — never happens in this file, refuse rather than mis-order
                panic!("JSON: mixed escape run");
            }
        }
        if !out.is_empty() {
            s.push_str(&String::from_utf16(&out).unwrap());
        }
        s
    }
    fn value(&mut self) -> Json {
        self.ws();
        match self.b[self.i] {
            b'{' => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.b[self.i] == b'}' {
                    self.i += 1;
                    return Json::Obj(items);
                }
                loop {
                    let k = self.string();
                    self.eat(b':');
                    let v = self.value();
                    items.push((k, v));
                    self.ws();
                    let c = self.b[self.i];
                    self.i += 1;
                    if c == b'}' {
                        return Json::Obj(items);
                    }
                    assert_eq!(c, b',', "JSON: object separator");
                }
            }
            b'[' => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.b[self.i] == b']' {
                    self.i += 1;
                    return Json::Arr(items);
                }
                loop {
                    items.push(self.value());
                    self.ws();
                    let c = self.b[self.i];
                    self.i += 1;
                    if c == b']' {
                        return Json::Arr(items);
                    }
                    assert_eq!(c, b',', "JSON: array separator");
                }
            }
            b'"' => Json::Str(self.string()),
            b't' => {
                self.i += 4;
                Json::Bool(true)
            }
            b'f' => {
                self.i += 5;
                Json::Bool(false)
            }
            b'n' => {
                self.i += 4;
                Json::Null
            }
            _ => {
                let start = self.i;
                while self.i < self.b.len() && (self.b[self.i] == b'-' || self.b[self.i].is_ascii_digit()) {
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.b[start..self.i]).unwrap();
                assert!(
                    !matches!(self.b.get(self.i), Some(b'.') | Some(b'e') | Some(b'E')),
                    "JSON: a bare float literal — the golden encodes floats as {{\"f\": hex}}"
                );
                Json::Int(t.parse().unwrap())
            }
        }
    }
}

pub fn parse_json(s: &str) -> Json {
    let mut p = P { b: s.as_bytes(), i: 0 };
    let v = p.value();
    p.ws();
    assert_eq!(p.i, p.b.len(), "JSON: trailing bytes");
    v
}

fn obj<'a>(j: &'a Json, key: &str) -> &'a Json {
    match j {
        Json::Obj(items) => &items.iter().find(|(k, _)| k == key).unwrap_or_else(|| panic!("no {key:?}")).1,
        _ => panic!("not an object"),
    }
}

/// The golden's `_decode`: `{"f": hex}` is a float; everything else is itself.
pub fn decode(j: &Json) -> V {
    match j {
        Json::Obj(items) if items.len() == 1 && items[0].0 == "f" => match &items[0].1 {
            Json::Str(h) => V::F(from_hex(h)),
            other => panic!("{{\"f\": …}} holding {other:?}"),
        },
        Json::Null => V::N,
        Json::Bool(b) => V::B(*b),
        Json::Int(i) => V::I(*i),
        Json::Str(s) => V::S(s.clone()),
        Json::Arr(xs) => V::J(
            xs.iter()
                .map(|x| match x {
                    Json::Int(i) => JItem::I(*i),
                    Json::Str(s) => JItem::S(s.clone()),
                    other => panic!("list item {other:?}"),
                })
                .collect(),
        ),
        other => panic!("unexpected golden value {other:?}"),
    }
}

pub struct Cpython {
    pub meta: Vec<(String, Json)>,
    pub kernels: Vec<(String, BTreeMap<String, V>)>,
}

pub fn load_cpython() -> Cpython {
    let root = parse_json(CPYTHON_JSON);
    let meta = match obj(&root, "meta") {
        Json::Obj(items) => items.clone(),
        _ => panic!("meta"),
    };
    let kernels = match obj(&root, "kernels") {
        Json::Obj(items) => items
            .iter()
            .map(|(name, body)| {
                let m = match body {
                    Json::Obj(kv) => kv.iter().map(|(k, v)| (k.clone(), decode(v))).collect(),
                    _ => panic!("kernel body"),
                };
                (name.clone(), m)
            })
            .collect(),
        _ => panic!("kernels"),
    };
    Cpython { meta, kernels }
}

// ------------------------------------------------------------------------------- comparison

/// Python's numeric `==` across `bool` / `int` / `float` (`True == 1 == 1.0`, `0.0 == -0.0`,
/// `nan != nan`); every other pair compares by value and type.
pub fn py_eq(a: &V, b: &V) -> bool {
    fn num(v: &V) -> Option<f64> {
        match v {
            V::F(x) => Some(*x),
            V::I(i) => Some(*i as f64),
            V::B(b) => Some(*b as i64 as f64),
            _ => None,
        }
    }
    match (a, b) {
        (V::I(x), V::I(y)) => x == y,
        _ => match (num(a), num(b)) {
            (Some(x), Some(y)) => x == y,
            _ => a == b,
        },
    }
}

/// Python's `_close(a, b, tol, abs_tol)`: `(ok, rel_err)`, `a` measured, `b` golden.
pub fn close(a: &V, b: &V, tol: f64, abs_tol: f64) -> (bool, f64) {
    if py_eq(a, b) {
        return (true, 0.0);
    }
    let (V::F(a), V::F(b)) = (a, b) else {
        return (false, f64::INFINITY);
    };
    if abs_tol > 0.0 && (a - b).abs() <= abs_tol {
        return (true, 0.0);
    }
    let err = if *b != 0.0 { (a - b).abs() / b.abs() } else { a.abs() };
    (if tol > 0.0 { err <= tol } else { false }, err)
}

/// The module's `TOL` table, transcribed — see `tests/test_numeric_fingerprint.py` for each
/// arm's measured drift. A `0.0` is bit-equality.
pub fn tol(name: &str) -> f64 {
    match name {
        "cpg" | "r66" | "r68" | "r77" | "r78" | "r79" | "r81" | "r82" | "r82r" | "r82t" => 0.0,
        "A" | "prop" => 1e-10,
        "B" | "r76" => 1e-11,
        "C" | "r7" | "r8" | "r10" | "r11" | "r12" | "r13" | "r15" | "r16" | "r18" | "r22"
        | "r23" => 1e-13,
        "D" => 1e-7,
        "E" | "r14" | "r17" | "r75" => 1e-9,
        "F" | "r25" => 1e-4,
        "r24" => 1e-12,
        "r27" => 1e-8,
        "r28" => 1e-5,
        "r67" | "r72" | "r73" | "r80" | "r81m" => 1e-15,
        "r69" | "r70" | "r71" | "r74" => 1e-14,
        _ => panic!("no TOL for kernel {name:?}"),
    }
}

/// The module's `ABS_TOL` table; an absent entry is `0.0`.
pub fn abs_tol(name: &str) -> f64 {
    match name {
        "r70" => 1e-15,
        "r72" | "r73" | "r74" | "r75" | "r76" | "r80" | "r81m" => 1e-9,
        _ => 0.0,
    }
}
