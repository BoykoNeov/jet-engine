//! **The two published pages' data, and the T–s chart's** — `docs/visuals/extract_data.py`,
//! `build.py`, `build_cutaway.py` and `main.py`'s `plot_ts_diagram`, ported (phase 8, slice AR;
//! `docs/plans/todo-rust-port.md` § 8.1 (iii)–(iv), § 8.9).
//!
//! * [`build_data`] runs the model at `main.py`'s design point and returns `data.json` as a
//!   [`Json`] value. [`Json::dump`] writes it exactly as Python's `json.dump(OUT, fh)` did — the
//!   committed `docs/visuals/data.json` is the reference, and `tests/visuals.rs` holds the two
//!   BYTE-equal.
//! * [`splice_visuals`] / [`splice_cutaway`] are the two template splices; [`cutaway_payload`] is
//!   `build_cutaway.py`'s trim, emitted from the value rather than parsed back.
//! * [`ts_diagram`] computes the points `plot_ts_diagram` drew — the work legs, the two 80-point
//!   isobar-shaped curves per cycle and the six station points — and the title, so the surviving
//!   Python (`plot_ts_diagram.py`) only DRAWS: it reads no engine code and does no physics.
//!
//! The crate has no JSON dependency by decision (`Cargo.toml`), so the tiny value type, its
//! writer and its reader live here. They accept exactly what these files need — no booleans;
//! strings escaped as `json.dumps` escapes them by default — and panic outside it.
//!
//! **Porting rules** are the panels' (`panels` module doc): keep each expression's operation
//! order, spell a square as a multiply (`4.0 * kp ** 2` is `4.0 * (kp * kp)`), and sum in Python's
//! iteration order with a plain left fold (the reference was written by PyPy, whose `sum` does not
//! compensate). The spelling is kept even where it cannot matter: `(4.0 * kp) * kp` gives the same
//! bits, because a multiply by a power of two is exact — an injection of exactly that survived
//! `data_json_is_the_models_byte_for_byte` (slice AR's sweep, § 8.9), as predicted.

use crate::engine::{build_turbojet, EngineResult, Losses};
use crate::gas::{equilibrium_composition, f_stoich, hf_fuel_default, Gas, RU};
use crate::nox::{
    beta_pdf_nodes_weights, bell_interpolator, ideal_bell_ei, quench_trajectory, spatial_segregation,
    super_eq_o_multiplier, two_stream_ceiling, ExhaustClampOpts, JetMixing, MixingPdf, PocketQuenchPdf,
    PromptNo, SpatialDwellPdf, SpatialPdf, Unmixedness, ZonedNoxOpts,
};
use crate::panels::{flight, real_losses, Design, PI_C, TT4};
use crate::pyf;
use crate::pyfmt::repr_f64;

// ------------------------------------------------------------------------------------ the value

/// A JSON value, with an object's keys kept in INSERTION order (Python's `dict`).
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    /// A Python `int` — written without a point (`m_of_T`'s `T` comes from `range`).
    Int(i64),
    /// A Python `float` — written as `repr(x)`, which always shows a point or an exponent.
    Float(f64),
    Str(String),
    List(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

/// Build a [`Json::Obj`] from `key => value` pairs, keeping their order.
#[macro_export]
macro_rules! jobj {
    ($($k:expr => $v:expr),* $(,)?) => {
        $crate::visuals::Json::Obj(vec![$(($k.to_string(), $crate::visuals::Json::from($v))),*])
    };
}

impl From<f64> for Json {
    fn from(x: f64) -> Self { Json::Float(x) }
}
impl From<Option<f64>> for Json {
    fn from(x: Option<f64>) -> Self { x.map_or(Json::Null, Json::Float) }
}
impl From<i64> for Json {
    fn from(n: i64) -> Self { Json::Int(n) }
}
impl From<&str> for Json {
    fn from(s: &str) -> Self { Json::Str(s.to_string()) }
}
impl From<String> for Json {
    fn from(s: String) -> Self { Json::Str(s) }
}
impl From<Vec<Json>> for Json {
    fn from(v: Vec<Json>) -> Self { Json::List(v) }
}
impl From<Vec<f64>> for Json {
    fn from(v: Vec<f64>) -> Self { Json::List(v.into_iter().map(Json::Float).collect()) }
}

impl Json {
    /// `obj[key]`, or `None` if this is not an object or has no such key.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// `obj[key]`, panicking with the key's name if it is missing.
    pub fn at(&self, key: &str) -> &Json {
        self.get(key).unwrap_or_else(|| panic!("JSON: no key {key:?}"))
    }

    /// The keys of an object, in order (empty for anything else).
    pub fn keys(&self) -> Vec<&str> {
        match self {
            Json::Obj(kv) => kv.iter().map(|(k, _)| k.as_str()).collect(),
            _ => Vec::new(),
        }
    }

    /// `json.dumps(x)` — Python's default separators `", "` and `": "`.
    pub fn dump(&self) -> String {
        let mut s = String::new();
        self.write(&mut s, ", ", ": ");
        s
    }

    /// `json.dumps(x, separators=(",", ":"))` — the cutaway's compact form.
    pub fn dump_compact(&self) -> String {
        let mut s = String::new();
        self.write(&mut s, ",", ":");
        s
    }

    fn write(&self, out: &mut String, item: &str, kv: &str) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Int(n) => out.push_str(&n.to_string()),
            Json::Float(x) => {
                // Python writes `NaN`/`Infinity` here (allow_nan=True); the pages would then hold
                // a value no browser parses as JSON, and `test_sweep_blocks_present_and_finite`
                // exists to refuse exactly that — so the writer refuses it first.
                assert!(x.is_finite(), "JSON: a non-finite number ({x}) reached the dump");
                out.push_str(&repr_f64(*x));
            }
            Json::Str(s) => write_str(out, s),
            Json::List(v) => {
                out.push('[');
                for (i, x) in v.iter().enumerate() {
                    if i > 0 {
                        out.push_str(item);
                    }
                    x.write(out, item, kv);
                }
                out.push(']');
            }
            Json::Obj(m) => {
                out.push('{');
                for (i, (k, x)) in m.iter().enumerate() {
                    if i > 0 {
                        out.push_str(item);
                    }
                    write_str(out, k);
                    out.push_str(kv);
                    x.write(out, item, kv);
                }
                out.push('}');
            }
        }
    }

    /// `json.loads(text)` for the subset [`Json::dump`] writes. A number with a `.`, `e` or `E`
    /// is a float (as in Python), anything else an int.
    pub fn parse(text: &str) -> Json {
        let b = text.as_bytes();
        let mut i = 0;
        let v = parse_value(b, &mut i);
        skip_ws(b, &mut i);
        assert!(i == b.len(), "JSON: trailing text at byte {i}");
        v
    }
}

/// A string as `json.dumps` writes it under its default `ensure_ascii=True`: the short escapes
/// for `"` `\` and the five named controls, `\u00XX` for any other control, and every non-ASCII
/// code point as lowercase `\uXXXX` (a surrogate pair above the BMP).
fn write_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            c => {
                let mut units = [0u16; 2];
                for u in c.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{:04x}", u));
                }
            }
        }
    }
    out.push('"');
}

fn skip_ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && matches!(b[*i], b' ' | b'\n' | b'\r' | b'\t') {
        *i += 1;
    }
}

fn parse_value(b: &[u8], i: &mut usize) -> Json {
    skip_ws(b, i);
    assert!(*i < b.len(), "JSON: unexpected end");
    match b[*i] {
        b'{' => {
            *i += 1;
            let mut kv = Vec::new();
            skip_ws(b, i);
            if b[*i] == b'}' {
                *i += 1;
                return Json::Obj(kv);
            }
            loop {
                skip_ws(b, i);
                let k = match parse_value(b, i) {
                    Json::Str(s) => s,
                    other => panic!("JSON: object key {other:?} is not a string"),
                };
                skip_ws(b, i);
                assert!(b[*i] == b':', "JSON: expected ':' at byte {i}");
                *i += 1;
                kv.push((k, parse_value(b, i)));
                skip_ws(b, i);
                match b[*i] {
                    b',' => *i += 1,
                    b'}' => {
                        *i += 1;
                        return Json::Obj(kv);
                    }
                    c => panic!("JSON: unexpected {:?} at byte {i}", c as char),
                }
            }
        }
        b'[' => {
            *i += 1;
            let mut v = Vec::new();
            skip_ws(b, i);
            if b[*i] == b']' {
                *i += 1;
                return Json::List(v);
            }
            loop {
                v.push(parse_value(b, i));
                skip_ws(b, i);
                match b[*i] {
                    b',' => *i += 1,
                    b']' => {
                        *i += 1;
                        return Json::List(v);
                    }
                    c => panic!("JSON: unexpected {:?} at byte {i}", c as char),
                }
            }
        }
        b'"' => {
            *i += 1;
            let mut units: Vec<u16> = Vec::new();
            while b[*i] != b'"' {
                assert!(b[*i].is_ascii(), "JSON: raw non-ASCII string byte at {i} (the writer escapes it)");
                if b[*i] == b'\\' {
                    *i += 1;
                    let c = match b[*i] {
                        b'"' => '"' as u16,
                        b'\\' => '\\' as u16,
                        b'/' => '/' as u16,
                        b'n' => '\n' as u16,
                        b'r' => '\r' as u16,
                        b't' => '\t' as u16,
                        b'b' => 8,
                        b'f' => 12,
                        b'u' => {
                            let h = std::str::from_utf8(&b[*i + 1..*i + 5]).unwrap();
                            *i += 4;
                            u16::from_str_radix(h, 16).unwrap_or_else(|_| panic!("JSON: bad \\u{h}"))
                        }
                        c => panic!("JSON: unknown escape \\{} at byte {i}", c as char),
                    };
                    units.push(c);
                } else {
                    units.push(b[*i] as u16);
                }
                *i += 1;
            }
            *i += 1;
            Json::Str(String::from_utf16(&units).expect("JSON: unpaired surrogate"))
        }
        b'n' => {
            assert!(b[*i..].starts_with(b"null"), "JSON: bad literal at byte {i}");
            *i += 4;
            Json::Null
        }
        _ => {
            let start = *i;
            while *i < b.len() && matches!(b[*i], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') {
                *i += 1;
            }
            let t = std::str::from_utf8(&b[start..*i]).unwrap();
            assert!(!t.is_empty(), "JSON: unexpected byte {:?} at {start}", b[start] as char);
            if t.contains(['.', 'e', 'E']) {
                Json::Float(t.parse().unwrap_or_else(|_| panic!("JSON: bad float {t:?}")))
            } else {
                Json::Int(t.parse().unwrap_or_else(|_| panic!("JSON: bad int {t:?}")))
            }
        }
    }
}

// ------------------------------------------------------------------------------- the rounding

/// `extract_data.py`'s `r(x, sig)`: `round(x, sig − 1 − floor(log10|x|))`, 0 ⇒ `0.0`.
///
/// Python's `round(x, n)` for `n ≥ 0` is the correctly-rounded `n`-place decimal of the exact
/// binary value (ties to even), read back — the same digits as `'%.nf' % x`, which Rust's
/// `{:.n}` reproduces (`pyfmt`'s battery). A NEGATIVE `n` (`|x| ≥ 10^sig`) is a different code
/// path in Python with no probe behind it here, and no dumped value reaches it, so it PANICS.
pub fn r(x: f64, sig: i32) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    let nd = sig - 1 - x.abs().log10().floor() as i32;
    assert!(nd >= 0, "r({x}, {sig}): a negative ndigits ({nd}) is outside the ported rounding");
    format!("{:.*}", nd as usize, x).parse().unwrap()
}

/// `r(x)` at the default six significant figures.
pub fn r6(x: f64) -> f64 { r(x, 6) }

/// `r(x)` of a value Python could hand over as `None`, which `r` passes through as `null`.
pub fn r6o(x: Option<f64>) -> Json { x.map_or(Json::Null, |v| Json::Float(r6(v))) }

// ----------------------------------------------------------------------------- the design point

/// `TAU = 3e-3` — the primary residence time every sweep below uses.
pub const TAU: f64 = 3e-3;

/// `REAL_LOSSES`, as the ordered dict `main.py` declares it.
fn losses_json(l: &Losses) -> Json {
    jobj! {
        "pi_d" => l.pi_d, "eta_c" => l.eta_c, "eta_b" => l.eta_b, "pi_b" => l.pi_b,
        "eta_t" => l.eta_t, "eta_m" => l.eta_m, "pi_n" => l.pi_n,
    }
}

/// `OUT["design"]` — imported from `main.py` in the Python, from [`crate::panels`] here: ONE
/// declaration of the design point, so the pages cannot describe a different engine.
pub fn design_block() -> Json {
    let f = flight();
    jobj! {
        "T0" => f.t0, "p0" => f.p0, "M0" => f.m0, "pi_c" => PI_C, "Tt4" => TT4,
        "losses" => losses_json(&real_losses()), "tau_ms" => TAU * 1e3,
    }
}

/// `cycle_points(result)` — `{label: (s, T)}`, entropy from the station-0 static datum on the
/// single cold-air-standard gas. Shared with [`ts_diagram`]: `main.py`'s `_cycle_points` is the
/// same function, line for line.
pub fn cycle_points(result: &EngineResult) -> Vec<(&'static str, f64, f64)> {
    let gas = Gas::default();
    let fl = flight();
    let (tref, pref) = (fl.t0, fl.p0);
    let s = |t: f64, p: f64| gas.spec.cp_c * (t / tref).ln() - gas.spec.r_c * (p / pref).ln();
    let st = |l: &str| result.station(l);
    let pts = [
        ("0", fl.t0, fl.p0),
        ("2", st("2").tt, st("2").pt),
        ("3", st("3").tt, st("3").pt),
        ("4", st("4").tt, st("4").pt),
        ("5", st("5").tt, st("5").pt),
        ("9", result.t9, fl.p0),
    ];
    pts.iter().map(|&(l, t, p)| (l, s(t, p), t)).collect()
}

fn coord(c: &[(&'static str, f64, f64)], label: &str) -> (f64, f64) {
    c.iter().find(|&&(l, _, _)| l == label).map(|&(_, s, t)| (s, t)).unwrap()
}

/// `ts_legs(coords)` — the 60-point page version of the chart's legs, rounded to 5 figures.
fn ts_legs(c: &[(&'static str, f64, f64)]) -> Json {
    let pair = |s: f64, t: f64| Json::List(vec![Json::Float(r(s, 5)), Json::Float(r(t, 5))]);
    let mut out = Vec::new();
    for leg in [["0", "2", "3"], ["4", "5", "9"]] {
        out.push(Json::List(leg.iter().map(|l| { let (s, t) = coord(c, l); pair(s, t) }).collect()));
    }
    let cp = Gas::default().spec.cp_c;
    for (a, b) in [("3", "4"), ("9", "0")] {
        let ((sa, ta), (sb, tb)) = (coord(c, a), coord(c, b));
        let residual = sb - (sa + cp * (tb / ta).ln());
        let pts = (0..60)
            .map(|i| {
                let t = ta + (tb - ta) * i as f64 / 59.0;
                pair(sa + cp * (t / ta).ln() + residual * (t - ta) / (tb - ta), t)
            })
            .collect();
        out.push(Json::List(pts));
    }
    Json::List(out)
}

/// `pack_run(result)` — one cycle's block.
fn pack_run(res: &EngineResult) -> Json {
    let stations = Json::Obj(
        res.stations
            .iter()
            .map(|(l, s)| (l.to_string(), jobj! { "Tt" => r6(s.tt), "pt" => r6(s.pt), "far" => r6(s.far) }))
            .collect(),
    );
    let perf = &res.performance;
    let cp = cycle_points(res);
    let points = Json::Obj(
        cp.iter()
            .map(|&(l, s, t)| (l.to_string(), Json::List(vec![Json::Float(r(s, 5)), Json::Float(r(t, 5))])))
            .collect(),
    );
    jobj! {
        "stations" => stations,
        "V0" => r6(res.v0), "V9" => r6(res.v9), "M9" => r6(res.m9), "T9" => r6(res.t9),
        "specific_thrust" => r6(perf.specific_thrust), "tsfc" => r6(perf.tsfc),
        "eta_brayton" => r6(perf.eta_brayton), "eta_thermal" => r6(perf.eta_thermal),
        "eta_propulsive" => r6(perf.eta_propulsive), "eta_overall" => r6(perf.eta_overall),
        "points" => points,
        "legs" => ts_legs(&cp),
    }
}

/// `OUT["design"]`, `OUT["ideal"]`, `OUT["real"]` — the cheap blocks (two CPG design runs). Kept
/// separately callable so their gate does not wait on the sweeps.
pub fn cycle_blocks(d: &Design) -> Vec<(String, Json)> {
    vec![
        ("design".to_string(), design_block()),
        ("ideal".to_string(), pack_run(&d.ideal)),
        ("real".to_string(), pack_run(&d.real)),
    ]
}

// ------------------------------------------------------------------------------------ the sweeps

/// The rung-6 equilibrium design point every sweep runs at.
struct EqPoint {
    eq: Gas,
    tt3: f64,
    tt4: f64,
    far: f64,
    p: f64,
    tt5: f64,
    pt5: f64,
}

fn eq_point() -> EqPoint {
    let fl = flight();
    let eq = Gas::reacting_equilibrium();
    let er = build_turbojet(eq.clone(), PI_C, TT4, fl.p0, real_losses()).run(&fl, 1.0);
    let (st3, st4, st5) = (er.station("3"), er.station("4"), er.station("5"));
    EqPoint { eq, tt3: st3.tt, tt4: st4.tt, far: st4.far, p: st4.pt, tt5: st5.tt, pt5: st5.pt }
}

fn zopts() -> ZonedNoxOpts { ZonedNoxOpts { tau: TAU, ..ZonedNoxOpts::default() } }

/// `comp[key]` for a composition kept in Python's dict order.
fn species(comp: &[(&'static str, f64)], key: &str) -> f64 {
    comp.iter().find(|&&(s, _)| s == key).unwrap_or_else(|| panic!("no species {key:?}")).1
}

/// `sum(comp.values())` — a left fold in the composition's own order.
fn total(comp: &[(&'static str, f64)]) -> f64 { comp.iter().fold(0.0, |a, &(_, v)| a + v) }

fn eq_design(e: &EqPoint) -> Json {
    jobj! {
        "Tt3" => r6(e.tt3), "Tt4" => r6(e.tt4), "far" => r6(e.far), "p" => r6(e.p),
        "phi_overall" => r6(e.far / f_stoich()),
    }
}

fn bell(e: &EqPoint, mark: &mut dyn FnMut(&str)) -> Json {
    let phis = [0.6, 0.7, 0.8, 0.9, 0.95, 1.0, 1.05, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0];
    let mut rows = Vec::new();
    for phi in phis {
        let z = e.eq.zoned_nox(e.far, e.tt3, e.tt4, e.p, phi, zopts());
        let z2 = e.eq.zoned_nox(e.far, e.tt3, e.tt4, e.p, phi,
                                ZonedNoxOpts { super_eq_o: true, prompt: Some(PromptNo::default()), ..zopts() });
        let comp = equilibrium_composition(z.far_primary, z.t_primary, e.p);
        let nt = total(&comp);
        rows.push(jobj! {
            "phi" => phi, "T_p" => r6(z.t_primary), "ei" => r6(z.ei_no()), "ppm_eq" => r6(z.primary.ppm_eq()),
            "ppm_kin" => r6(z.ppm_primary()), "T_mix" => r6(z.t_mix),
            "xco" => r6(species(&comp, "CO") / nt), "xh2" => r6(species(&comp, "H2") / nt),
            "ei_lift" => r6(z2.ei_no()), "ei_prompt" => r6(z2.ei_no_prompt), "m" => r6(z2.o_multiplier),
        });
        mark(&pyf!("  phi={:.2f}  T_p={:.0f}  EI={:.3g}", phi, z.t_primary, z.ei_no()));
    }
    Json::List(rows)
}

fn quench(e: &EqPoint, mark: &mut dyn FnMut(&str)) -> Json {
    let mut out = Vec::new();
    for (tag, phi) in [("rich", 1.4), ("lean", 0.9)] {
        let z = e.eq.zoned_nox(e.far, e.tt3, e.tt4, e.p, phi, zopts());
        let comp_prim = equilibrium_composition(z.far_primary, z.t_primary, e.p);
        let tab = quench_trajectory(&comp_prim, z.t_primary, z.alpha, e.far, e.tt3, e.p, 81);
        let n = tab.len();
        let rows = tab
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let b = i as f64 / (n - 1) as f64;
                let conc = e.p / (RU * row.t);
                jobj! { "beta" => r(b, 4), "T" => r(row.t, 5), "xnoe_ppm" => r(1e6 * row.c_noe / conc, 4) }
            })
            .collect::<Vec<_>>();
        out.push((tag.to_string(), jobj! { "phi" => phi, "T_primary" => r6(z.t_primary), "rows" => rows }));
    }
    let mut tq_rows = Vec::new();
    for tq_ms in [0.3, 0.5, 1.0, 2.0, 3.0, 5.0] {
        let z = e.eq.zoned_nox(e.far, e.tt3, e.tt4, e.p, 1.4, ZonedNoxOpts {
            tau_q: Some(tq_ms * 1e-3), quench_ngrid: 80, quench_nsteps: 600, ..zopts()
        });
        tq_rows.push(jobj! { "tau_q_ms" => tq_ms, "ei" => r6o(z.ei_no_quenched), "T_peak" => r6o(z.t_peak) });
        mark(&pyf!("  tau_q={} ms -> EI={:.3g}", tq_ms, z.ei_no_quenched.unwrap()));
    }
    let z_ideal = e.eq.zoned_nox(e.far, e.tt3, e.tt4, e.p, 1.4, zopts());
    out.push(("tau_sweep".to_string(), Json::List(tq_rows)));
    out.push(("ideal_ei".to_string(), Json::Float(r6(z_ideal.ei_no()))));
    Json::Obj(out)
}

fn jsweep(e: &EqPoint, mark: &mut dyn FnMut(&str)) -> Json {
    let (s, h) = (0.0625, 0.10);
    let um = Unmixedness { s, ..Unmixedness::default() };
    let pdfc = MixingPdf { s, ..MixingPdf::default() };
    let hf = hf_fuel_default();
    mark("  building the ideal bell (n_bell=80) ...");
    let bell_f = bell_interpolator(e.p, e.tt3, hf, TAU, 80, false);
    let point_val = ideal_bell_ei(e.far, e.p, e.tt3, hf, TAU, false);
    let pdf_ei = |g: f64| -> f64 {
        if g <= 1e-9 {
            return point_val;
        }
        let xibar = e.far / (1.0 + e.far);
        let (nodes, w) = beta_pdf_nodes_weights(xibar, g, 160);
        w.iter().zip(&nodes).fold(0.0, |a, (wi, x)| a + wi * bell_f.at(*x))
    };
    let mut rows = Vec::new();
    for j in [4.0, 6.0, 9.0, 12.0, 16.0, 20.0, 25.0, 36.0, 49.0, 64.0, 100.0, 144.0, 225.0, 400.0] {
        let m = JetMixing { j, c_e: 0.20, shape_n: 2.0, ..JetMixing::default() };
        let z = e.eq.zoned_nox(e.far, e.tt3, e.tt4, e.p, 1.5, ZonedNoxOpts {
            mixing: Some(m), unmixedness: Some(um), quench_ngrid: 60, quench_nsteps: 400, ..zopts()
        });
        let g = pdfc.segregation(pdfc.c(&JetMixing { j, ..JetMixing::default() }));
        rows.push(jobj! {
            "J" => j, "C" => r6o(z.c_holdeman), "tau_q_ms" => r6(z.tau_q.unwrap() * 1e3),
            "ei_bulk" => r6o(z.ei_no_quenched), "ei_unmixed" => r6o(z.ei_no_unmixed),
            "w_core" => r6o(z.w_core), "g" => r6(g), "ei_pdf" => r6(pdf_ei(g)),
        });
        mark(&pyf!("  J={:.0f}  C={:.2f}  bulk={:.3g}  two-stream={:.3g}  pdf={:.3g}", j,
                   z.c_holdeman.unwrap(), z.ei_no_quenched.unwrap(), z.ei_no_unmixed.unwrap(), pdf_ei(g)));
    }
    let q = um.c_opt * h / s;
    jobj! { "S" => s, "H" => h, "C_opt" => um.c_opt, "J_opt" => q * q, "point_val" => r6(point_val), "rows" => rows }
}

fn m_of_t() -> Json {
    Json::List((1500..2801).step_by(50).map(|t: i64| jobj! { "T" => t, "m" => r6(super_eq_o_multiplier(t as f64)) }).collect())
}

fn prompt_shape() -> Json {
    let pr = PromptNo::default();
    Json::List(
        (0..71)
            .map(|i| {
                let phi = 0.6 + 0.02 * i as f64;
                let fc = pr.f_correction(phi);
                // Python's `max(fc, 0.0)` returns its FIRST argument unless the second is larger.
                let f = if 0.0 > fc { 0.0 } else { fc };
                jobj! { "phi" => r(phi, 4), "f" => r6(f) }
            })
            .collect(),
    )
}

fn spatial(e: &EqPoint, mark: &mut dyn FnMut(&str)) -> Json {
    let d = SpatialPdf::default();
    let kp = d.k_p;
    let js = [2.0, 3.0, 4.0, 5.5, 7.5, 10.0, 13.0, 16.0, 20.0, 25.0, 32.0, 42.0, 56.0, 75.0, 100.0, 140.0,
              200.0, 280.0, 400.0];
    let mut curves = Vec::new();
    for (sg, hg) in [(0.0625, 0.10), (0.03125, 0.10), (0.0625, 0.05)] {
        let rows = js
            .iter()
            .map(|&j| {
                // `_spatial_segregation(far, 1.5, Sg, Hg, J, ny=32, nz=32)` — the plume constants
                // at the Python signature's defaults, which are `SpatialPdf`'s.
                let g = spatial_segregation(e.far, 1.5, sg, hg, j, d.k_p, d.k_y, d.k_z, 32, 32);
                jobj! { "J" => j, "C" => r6((sg / hg) * j.sqrt()), "g" => r6(g) }
            })
            .collect::<Vec<_>>();
        curves.push(jobj! { "S" => sg, "H" => hg, "rows" => rows });
        mark(&pyf!("  geometry S={} H={} done", sg, hg));
    }
    jobj! {
        "k_p" => kp, "C_opt" => 1.0 / (4.0 * (kp * kp)),
        "g_ceiling" => r6(two_stream_ceiling(e.far, 1.5)), "curves" => curves,
    }
}

/// The rung-17 ladder. The Python wraps it in `try/except Exception` and writes `null` on a
/// failure; a Rust failure PANICS instead (louder, and the byte gate would see it either way).
fn ladder(e: &EqPoint, mark: &mut dyn FnMut(&str)) -> Json {
    let fl = flight();
    let (tt9, pt9, p9) = (e.tt5, real_losses().pi_n * e.pt5, fl.p0);
    let mix = JetMixing { j: 225.0, c_e: 0.20, shape_n: 2.0, ..JetMixing::default() };
    let pq = PocketQuenchPdf { s: 0.0625, n_bell: 40, n_quad: 120, ..PocketQuenchPdf::default() };
    let mut out = Vec::new();
    for (tag, seo) in [("eq_o", false), ("super_eq_o", true)] {
        let c = e.eq.exhaust_no_clamp(e.far, e.tt3, e.tt4, e.p, tt9, pt9, p9, 1.5, mix, pq, ExhaustClampOpts {
            tau: TAU, super_eq_o: seo, quench_ngrid: 60, quench_nsteps: 400,
        });
        out.push((tag.to_string(), jobj! {
            "a_mixed" => r6(c.a_mixed_out), "a_bulk" => r6(c.a_bulk_quench),
            "a_pocket" => r6(c.a_pocket), "collapse" => r6(c.no_collapse_ratio),
        }));
        mark(&pyf!("  {}: a_mixed={:.3g} a_bulk={:.3g} a_pocket={:.3g}", tag, c.a_mixed_out,
                   c.a_bulk_quench, c.a_pocket));
    }
    Json::Obj(out)
}

fn dwell(e: &EqPoint, mark: &mut dyn FnMut(&str)) -> Json {
    let cfg = SpatialDwellPdf { s: 0.0625, ny: 32, nz: 32, nt: 24, n_bell: 40, n_quad: 120, ..SpatialDwellPdf::default() };
    let mut rows = Vec::new();
    for j in [4.0, 9.0, 16.0, 36.0, 64.0, 144.0, 400.0] {
        let m = JetMixing { j, c_e: 0.20, u_c: 75.0, h: 0.10, ..JetMixing::default() };
        let z = e.eq.zoned_nox(e.far, e.tt3, e.tt4, e.p, 1.5, ZonedNoxOpts {
            mixing: Some(m), spatial_dwell: Some(cfg), quench_ngrid: 24, ..zopts()
        });
        let t2_corr = z.ei_no_spatial_dwell_excess.unwrap();
        let floor = z.ei_no_spatial_dwell.unwrap() - t2_corr;
        let t2_mean = z.ei_no_spatial_dwell_meanfield.unwrap() - floor;
        rows.push(jobj! {
            "J" => j, "C" => r6(cfg.c(&m)), "g" => r6o(z.g_spatial_dwell),
            "tau_mix_ms" => r6(m.tau_q() * 1e3), "tau_mean_ms" => r6(z.tau_mean_dwell.unwrap() * 1e3),
            "ei_corr" => r6o(z.ei_no_spatial_dwell), "ei_mean" => r6o(z.ei_no_spatial_dwell_meanfield),
            "floor" => r6(floor), "t2_corr" => r6(t2_corr), "t2_mean" => r6(t2_mean),
            "corr" => r6o(z.corr_ratio), "max_a" => r6o(z.max_a_quench),
        });
        mark(&pyf!("  J={:.0f}  C={:.2f}  corr/mean={:.4f}  max_a={:.3f}", j, cfg.c(&m),
                   z.corr_ratio.unwrap(), z.max_a_quench.unwrap()));
    }
    let kp = cfg.k_p;
    jobj! { "S" => cfg.s, "k_p" => kp, "C_opt" => 1.0 / (4.0 * (kp * kp)), "rows" => rows }
}

/// The nine expensive blocks, in `extract_data.py`'s order — the names `test_visuals_data.py`
/// calls `_SWEEP_BLOCKS`.
pub const SWEEP_BLOCKS: [&str; 9] =
    ["eq_design", "bell", "quench", "jsweep", "m_of_T", "prompt_shape", "spatial", "ladder", "dwell"];

/// The whole of `data.json`, in `extract_data.py`'s order. `mark` receives the progress lines the
/// Python printed (the CLI shows them; the tests drop them).
pub fn build_data(d: &Design, mark: &mut dyn FnMut(&str)) -> Json {
    mark("CPG cycle (ideal vs real) + T-s legs");
    let mut out = cycle_blocks(d);
    mark("equilibrium-gas design point (rung-6 cycle)");
    let e = eq_point();
    out.push(("eq_design".into(), eq_design(&e)));
    mark("rung-9/19 phi_p bell sweep");
    out.push(("bell".into(), bell(&e, mark)));
    mark("rung-10 quench trajectories + tau_q sweep");
    out.push(("quench".into(), quench(&e, mark)));
    mark("rung-11/12/13 J-sweeps (bulk / two-stream / beta-PDF)");
    out.push(("jsweep".into(), jsweep(&e, mark)));
    mark("rung-19 m(T) Westenberg multiplier");
    out.push(("m_of_T".into(), m_of_t()));
    out.push(("prompt_shape".into(), prompt_shape()));
    mark("rung-22 spatial collapse (3 geometries)");
    out.push(("spatial".into(), spatial(&e, mark)));
    mark("rung-17 exhaust-NO clamp ladder (J=225, reduced grids)");
    out.push(("ladder".into(), ladder(&e, mark)));
    mark("rung-23 derived dwell spectrum (correlated vs matched-mean twin)");
    out.push(("dwell".into(), dwell(&e, mark)));
    Json::Obj(out)
}

// ------------------------------------------------------------------------------------ the splices

/// `build_cutaway.py`'s `KEEP` — the cycle fields the cutaway embeds. `eta_brayton` is dumped and
/// deliberately NOT kept (the Brayton bound is the charts page's story). While the Python
/// exists, `tests/visuals.rs` holds this list equal to `build_cutaway.py`'s.
pub const KEEP: [&str; 12] = ["stations", "V0", "V9", "M9", "T9", "specific_thrust", "tsfc",
                              "eta_thermal", "eta_propulsive", "eta_overall", "points", "legs"];

/// The placeholder `template.html` carries, replaced by `data.json` verbatim.
pub const VISUALS_PLACEHOLDER: &str = "/*__DATA_JSON__*/";
/// The placeholder `cutaway-template.html` carries, replaced by the compact trimmed payload.
pub const CUTAWAY_PLACEHOLDER: &str = "/*__DATA__*/";

/// `{"design": …, "ideal": {k: … for k in KEEP}, "real": {…}}` — in KEEP's order, as the Python's
/// dict comprehension builds it.
pub fn cutaway_payload(data: &Json) -> Json {
    let trim = |case: &str| Json::Obj(KEEP.iter().map(|k| (k.to_string(), data.at(case).at(k).clone())).collect());
    jobj! { "design" => data.at("design").clone(), "ideal" => trim("ideal"), "real" => trim("real") }
}

/// `build.py`: the template with its ONE placeholder replaced by `data.json`'s text. The template
/// is read as Python's `read_text` reads it (universal newlines), so a CRLF working copy splices
/// to the same LF page.
pub fn splice_visuals(template: &str, data_json: &str) -> String {
    let tpl = template.replace("\r\n", "\n");
    assert!(tpl.matches(VISUALS_PLACEHOLDER).count() == 1, "template placeholder missing/duplicated");
    tpl.replacen(VISUALS_PLACEHOLDER, data_json, 1)
}

/// `build_cutaway.py`: the cutaway template with its placeholder replaced by the compact payload.
pub fn splice_cutaway(template: &str, data: &Json) -> String {
    let tpl = template.replace("\r\n", "\n");
    assert!(tpl.matches(CUTAWAY_PLACEHOLDER).count() == 1, "cutaway template placeholder missing/duplicated");
    tpl.replace(CUTAWAY_PLACEHOLDER, &cutaway_payload(data).dump_compact())
}

// -------------------------------------------------------------------------------- the T–s chart

/// One cycle as `plot_ts_diagram`'s `draw` plots it.
pub struct TsCycle {
    /// `0→2→3` and `4→5→9`, as `(s, T)` arrays (vertical when isentropic).
    pub work_legs: [(Vec<f64>, Vec<f64>); 2],
    /// The combustion `3→4` and heat-rejection `9→0` legs, 80 points each.
    pub isobars: [(Vec<f64>, Vec<f64>); 2],
    /// The six station points, `(label, s, T)`.
    pub points: Vec<(&'static str, f64, f64)>,
}

/// Everything the chart draws that comes from the model.
pub struct TsDiagram {
    pub ideal: TsCycle,
    pub real: TsCycle,
    /// The figure title, rendered from the design point (`main.py` typed its loss half).
    pub title: String,
}

fn ts_cycle(c: Vec<(&'static str, f64, f64)>) -> TsCycle {
    let leg = |ls: [&str; 3]| -> (Vec<f64>, Vec<f64>) {
        (ls.iter().map(|l| coord(&c, l).0).collect(), ls.iter().map(|l| coord(&c, l).1).collect())
    };
    let cp = Gas::default().spec.cp_c;
    let iso = |a: &str, b: &str| -> (Vec<f64>, Vec<f64>) {
        let ((sa, ta), (sb, tb)) = (coord(&c, a), coord(&c, b));
        let residual = sb - (sa + cp * (tb / ta).ln()); // = -R*ln(pb/pa)
        let ts: Vec<f64> = (0..80).map(|i| ta + (tb - ta) * i as f64 / 79.0).collect();
        let ss = ts.iter().map(|&t| sa + cp * (t / ta).ln() + residual * (t - ta) / (tb - ta)).collect();
        (ss, ts)
    };
    TsCycle { work_legs: [leg(["0", "2", "3"]), leg(["4", "5", "9"])], isobars: [iso("3", "4"), iso("9", "0")], points: c }
}

/// `plot_ts_diagram(ideal, real, flight)`'s data: both cycles' arrays and the title.
pub fn ts_diagram(d: &Design) -> TsDiagram {
    let l = real_losses();
    let title = pyf!("Turbojet T–s diagram — ideal vs real components\n(M0={}, π_c={:.0f}, Tt4={:.0f} K; real: η_c={:.2f}, η_t={:.2f}, π losses)",
                     d.flight.m0, PI_C, TT4, l.eta_c, l.eta_t);
    TsDiagram { ideal: ts_cycle(cycle_points(&d.ideal)), real: ts_cycle(cycle_points(&d.real)), title }
}

/// The JSON `plot_ts_diagram.py` draws from. Every float is written as its shortest round-trip
/// `repr`, so Python reads back the exact bits Rust computed.
pub fn ts_diagram_json(t: &TsDiagram) -> Json {
    let xy = |(s, t): &(Vec<f64>, Vec<f64>)| jobj! { "s" => s.clone(), "T" => t.clone() };
    let cyc = |c: &TsCycle| {
        jobj! {
            "work_legs" => c.work_legs.iter().map(xy).collect::<Vec<_>>(),
            "isobars" => c.isobars.iter().map(xy).collect::<Vec<_>>(),
            "points" => c.points.iter().map(|&(l, s, t)| jobj! { "label" => l, "s" => s, "T" => t }).collect::<Vec<_>>(),
        }
    };
    jobj! { "title" => t.title.clone(), "ideal" => cyc(&t.ideal), "real" => cyc(&t.real) }
}
