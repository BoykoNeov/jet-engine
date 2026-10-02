//! PHASE 8 SLICE AK — **the formatting shim against PYTHON's own output** (plan § 8.1 (ii)).
//!
//! `rust/oracle/dump_pyfmt.py` (run under PyPy) wrote two files:
//!
//! * `pyfmt_battery.tsv` — the inputs: floats as u64 bit patterns (exact binary ties `k/2ⁿ`,
//!   every special and rounding boundary, and log-uniform random draws committed AS DATA, since a
//!   `random.seed` dies with the interpreter), ints, strs (with `Δ`, `·`, `≡`, `≈`, which pad by
//!   code point) and bools.
//! * `pyfmt_expect.tsv` — Python's output for each (style, spec, kind) over that kind's battery.
//!   **DEEP**: every float through `str`, `repr` and `f`/`e`/`g`/`%` at precisions 0–8, 10, 12, 15.
//!   **BROAD**: every spec the AST census found in `main.py` — 250 new-style specs and every
//!   `%`-template conversion — plus neighbours, on a small battery of each kind.
//!
//! A cell Python REFUSED (`format(1.5, 'd')`, `'%d' % inf`) is written `\x01ERR` and skipped
//! here: the shim panics on those instead, and the panels never reach one. The counts are pinned
//! so the oracle cannot quietly lose rows.

use turbojet::pyfmt::{printf, PyFormat, Spec};

const BATTERY: &str = include_str!("../oracle/pyfmt_battery.tsv");
const EXPECT: &str = include_str!("../oracle/pyfmt_expect.tsv");

enum Val {
    F(f64),
    I(i64),
    S(String),
    B(bool),
}

impl Val {
    fn dynv(&self) -> &dyn PyFormat {
        match self {
            Val::F(x) => x,
            Val::I(x) => x,
            Val::S(x) => x,
            Val::B(x) => x,
        }
    }
}

fn battery(kind: &str) -> Vec<Val> {
    BATTERY
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let (k, v) = l.split_once('\t').expect("a battery row is `kind<TAB>value`");
            (k == kind).then(|| match k {
                "float" | "float_small" => Val::F(f64::from_bits(u64::from_str_radix(v, 16).unwrap())),
                "int" => Val::I(v.parse().unwrap()),
                "str" => Val::S(v.to_string()),
                "bool" => Val::B(v == "True"),
                _ => unreachable!(),
            })
        })
        .collect()
}

#[test]
fn the_shim_reproduces_python_on_every_oracle_cell() {
    let kinds = ["float", "float_small", "int", "str", "bool"];
    let bats: Vec<(&str, Vec<Val>)> = kinds.iter().map(|k| (*k, battery(k))).collect();
    assert_eq!(bats.iter().map(|(_, b)| b.len()).collect::<Vec<_>>(), vec![616, 80, 24, 14, 2],
               "the battery lost or gained values");

    let (mut rows, mut cells, mut refused, mut exempt) = (0usize, 0usize, 0usize, 0usize);
    let mut bad: Vec<String> = Vec::new();
    for line in EXPECT.lines().filter(|l| !l.starts_with('#')) {
        let mut f = line.split('\t');
        let (style, spec, kind) = (f.next().unwrap(), f.next().unwrap(), f.next().unwrap());
        let outs: Vec<&str> = f.collect();
        let vals = &bats.iter().find(|(k, _)| *k == kind).expect("a known kind").1;
        assert_eq!(outs.len(), vals.len(), "row {style} {spec:?} {kind}: cell count");
        rows += 1;
        for (v, want) in vals.iter().zip(&outs) {
            if *want == "\x01ERR" {
                refused += 1;
                continue;
            }
            // PyPy's Python-2 `'%f'`-becomes-`'%g'` rule at |x| >= 1e50 — refused by the shim
            // (it panics there); see `PyFormat for f64`. Counted, not silently dropped.
            if style == "pct" && spec.ends_with('f') {
                if let Val::F(x) = v {
                    if x.is_finite() && x.abs() >= 1e50 {
                        exempt += 1;
                        continue;
                    }
                }
            }
            let v = v.dynv();
            let got = match style {
                "str" => v.py_str(),
                "repr" => v.py_repr(),
                "new" => v.py_format(&Spec::parse(spec)),
                "pct" => printf(spec, &[v]),
                s => panic!("unknown style {s}"),
            };
            cells += 1;
            if got != *want {
                bad.push(format!("{style} {spec:?} {kind}: Python {want:?}, shim {got:?}"));
            }
        }
    }
    assert!(bad.is_empty(), "{} of {cells} cells differ; first: {:#?}", bad.len(), &bad);
    println!("rows {rows}, cells {cells}, refused {refused}, exempt {exempt}");
    assert_eq!((rows, cells, refused, exempt), (1212, 72_666, 32, 70), "the oracle's size moved");
}

const CONTAINERS: &str = include_str!("../oracle/pyfmt_containers.tsv");

/// `None` and the containers (slice AL pre-flight): each row of `pyfmt_containers.tsv` is a value
/// `dump_pyfmt.py` NAMED; the same value is built here under that name, and its `str`, `repr`,
/// `'%s'` and `format(v, '')` must all equal PyPy's.
#[test]
fn none_and_the_containers_match_python() {
    use turbojet::pyfmt::{printf, py_dict, py_list, py_tuple, PyRaw};
    let none: Option<f64> = None;
    let build = |name: &str| -> PyRaw {
        match name {
            "none" => PyRaw(none.py_str()),
            "tuple_floats" => py_tuple(&[&0.05, &0.2]),
            "tuple_one" => py_tuple(&[&0.05]),
            "tuple_empty" => py_tuple(&[]),
            "tuple_ints" => py_tuple(&[&3i64, &7i64, &12i64]),
            "tuple_strs" => py_tuple(&[&"demand", &"applied"]),
            "tuple_mixed" => py_tuple(&[&1i64, &0.5, &"x", &true, &none]),
            "tuple_small_big" => py_tuple(&[&1e-05, &1e16, &-0.0, &2.5e-07]),
            "list_floats" => py_list(&[&0.1, &1e-05, &2.0, &1500.0]),
            "list_strs" => py_list(&[&"clip", &"demand", &"demand-latched"]),
            "list_empty" => py_list(&[]),
            "list_nested" => py_list(&[&py_tuple(&[&0.2, &1.0]), &py_tuple(&[&5.0])]),
            "list_bools" => py_list(&[&true, &false]),
            "dict_str_str" => py_dict(&[(&"CO", &"0.0123%"), (&"OH", &"1.5000%")]),
            "dict_str_float" => py_dict(&[(&"a", &0.25), (&"b", &1e-07)]),
            n => panic!("the oracle names a container this test does not build: {n}"),
        }
    };
    let mut n = 0;
    for line in CONTAINERS.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        let v = build(f[0]);
        let got = [v.py_str(), v.py_repr(), printf("%s", &[&v]), v.py_format(&Spec::parse(""))];
        assert_eq!(got.iter().map(String::as_str).collect::<Vec<_>>(), f[1..].to_vec(), "{}", f[0]);
        n += 1;
    }
    assert_eq!(n, 15, "the container oracle lost or gained rows");
    // None directly (not through PyRaw): bare, '%s', and inside an f-string.
    assert_eq!(turbojet::pyf!("{} {!r}", none, none), "None None");
    assert_eq!(printf("%-6s|", &[&none]), "None  |");
}

/// The rules written out, so the expected strings are readable here and not only in a TSV.
#[test]
fn the_spelling_rules_by_hand() {
    use turbojet::{pct, pyf};
    // `e`: Python signs and pads the exponent.
    assert_eq!(pyf!("{:.2e}", 0.002898), "2.90e-03");
    assert_eq!(pyf!("{:.0e}", 1.2e-12), "1e-12");
    // repr: the notation switch at 1e-4 / 1e16, and the point always shown.
    assert_eq!(pyf!("{}", 1e-5), "1e-05");
    assert_eq!(pyf!("{}", 0.0001), "0.0001");
    assert_eq!(pyf!("{}", 1e16), "1e+16");
    assert_eq!(pyf!("{}", 1e15), "1000000000000000.0");
    assert_eq!(pyf!("{}", 0.85), "0.85");
    // `g`: notation by the ROUNDED exponent, zeros stripped, `.0g` is `.1g`.
    assert_eq!(pyf!("{:.3g}", 999.5), "1e+03");
    assert_eq!(pyf!("{:.0g}", 0.25), "0.2");
    assert_eq!(pct!("%-8g|", 0.5), "0.5     |");
    // `%`: a real multiply by 100, then `f`.
    assert_eq!(pyf!("{:+.1%}", 0.0125), "+1.2%");
    // NaN never shows its sign; a bool with a spec is an int.
    assert_eq!(pyf!("{:.2f}", -f64::NAN), "nan");
    assert_eq!(pyf!("{:>5} {}", true, false), "    1 False");
    // `%`-templates right-align strings by default; new-style left-aligns them.
    assert_eq!(pct!("%6s|%-6s|", "ab", "cd"), "    ab|cd    |");
    assert_eq!(pyf!("{:6}|", "ab"), "ab    |");
    // Width counts code points: `Δ` is two bytes but one column.
    assert_eq!(pyf!("{:>3}|", "Δ"), "  Δ|");
}
