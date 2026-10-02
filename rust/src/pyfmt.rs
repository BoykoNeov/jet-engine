//! **Python's string formatting, reproduced byte for byte** — the phase-8 shim the CLI prints
//! through (`docs/plans/todo-rust-port.md` § 8.1 (ii)).
//!
//! `main.py` is held to BYTE equality with its PyPy stdout, and Rust ≡ PyPy on every value it
//! prints, so whatever differs between the two outputs is SPELLING. The pre-flight probe measured
//! that the digits never differ: Rust's `{:.N}` and `{:.Ne}` are correctly rounded from the exact
//! binary value with ties to even, exactly as CPython's `format_float_short` is, and Rust's bare
//! `{:e}` is the same shortest round-trip digit string as Python's `repr`. So this module takes
//! its DIGITS from Rust's formatter — one source — and only lays them out the way Python does:
//!
//! * `e` — Python signs the exponent and pads it to two digits (`e-03`, `e+16`); Rust writes
//!   `e-3`, `e16`.
//! * `repr` / `str` — the same shortest digits, but Python switches to exponent notation for a
//!   decimal exponent below −4 or at 16 and above, and always shows a point (`1.0`).
//! * `g` — no Rust equivalent; precision `0` means `1`, the notation is chosen by the exponent of
//!   the ROUNDED value, and trailing zeros are stripped.
//! * `%` — Python multiplies by `100.0` (a real floating-point multiply, kept as one here) and then
//!   formats `f`.
//! * NaN and the infinities — `nan`, `inf`, `-inf`; a NaN's sign bit is never printed.
//! * `bool` — `True`/`False` bare, but with any non-empty spec Python formats it as the INT `1`/`0`.
//! * Width counts CODE POINTS, not bytes — the panels pad `Δ`, `·` and `≈`.
//!
//! Only the spec grammar `main.py` actually uses is accepted: `[[fill]align][sign][width]
//! [.precision][type]` with align `<>^`, sign `+`/space, and types `f e g % d s`. The census of
//! every f-string and `%`-template in `main.py` found no `0`-padding, no `#`, no `,`/`_` grouping
//! and no `=` alignment, so those PANIC rather than guess — a panel that needs one fails loudly on
//! its first run instead of printing something plausible. The whole grammar is gated against
//! Python's own output in `tests/pyfmt.rs` (`oracle/pyfmt_*.tsv`, written by PyPy).
//!
//! Panels spell a Python f-string as [`pyf!`] and a `%`-template as [`pct!`], with each `{…}`
//! expression hoisted into an argument and the spec left exactly as Python wrote it:
//!
//! ```
//! use turbojet::pyf;
//! assert_eq!(pyf!("{:>8} {:>10.1f}", "4", 1500.0), "       4     1500.0");
//! ```
//!
//! `demand_coordinate::{py_e, py_g, py_repr}` are three narrower copies of rules in here, written
//! for rung 74's returned messages before this module existed. They stay where they are — each is
//! gated by its own oracle, and folding them in would move a gate, not remove a duplication.

/// A parsed format spec: `[[fill]align][sign][width][.precision][type]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spec {
    pub fill: char,
    pub align: Option<char>,
    pub sign: Option<char>,
    pub width: usize,
    pub prec: Option<usize>,
    pub ty: Option<char>,
}

impl Spec {
    /// Parse a Python new-style spec (the text after `:` in `{x:spec}`). Panics on any feature
    /// outside the census, naming it.
    pub fn parse(s: &str) -> Spec {
        let c: Vec<char> = s.chars().collect();
        let mut i = 0;
        let mut fill = ' ';
        let mut align = None;
        let is_align = |ch: char| matches!(ch, '<' | '>' | '^' | '=');
        if c.len() >= 2 && is_align(c[1]) {
            fill = c[0];
            align = Some(c[1]);
            i = 2;
        } else if !c.is_empty() && is_align(c[0]) {
            align = Some(c[0]);
            i = 1;
        }
        assert!(align != Some('='), "pyfmt: `=` alignment is outside the census: {s:?}");
        let mut sign = None;
        if i < c.len() && matches!(c[i], '+' | '-' | ' ') {
            sign = Some(c[i]);
            i += 1;
        }
        assert!(!(i < c.len() && matches!(c[i], '#' | 'z')), "pyfmt: `#`/`z` outside the census: {s:?}");
        assert!(!(i < c.len() && c[i] == '0'), "pyfmt: `0`-padding is outside the census: {s:?}");
        let mut width = 0usize;
        while i < c.len() && c[i].is_ascii_digit() {
            width = width * 10 + c[i].to_digit(10).unwrap() as usize;
            i += 1;
        }
        assert!(!(i < c.len() && matches!(c[i], ',' | '_')), "pyfmt: grouping is outside the census: {s:?}");
        let mut prec = None;
        if i < c.len() && c[i] == '.' {
            i += 1;
            let mut p = 0usize;
            let start = i;
            while i < c.len() && c[i].is_ascii_digit() {
                p = p * 10 + c[i].to_digit(10).unwrap() as usize;
                i += 1;
            }
            assert!(i > start, "pyfmt: `.` without a precision: {s:?}");
            prec = Some(p);
        }
        let mut ty = None;
        if i < c.len() {
            assert!(matches!(c[i], 'f' | 'e' | 'g' | '%' | 'd' | 's'),
                    "pyfmt: type {:?} is outside the census: {s:?}", c[i]);
            ty = Some(c[i]);
            i += 1;
        }
        assert!(i == c.len(), "pyfmt: trailing characters in spec {s:?}");
        Spec { fill, align, sign, width, prec, ty }
    }

    /// The spec a printf-style conversion (`%-10.3f`, without the leading `%` and the type)
    /// means. Unlike new-style, `%`-formatting right-aligns EVERYTHING by default, strings
    /// included; `-` is the only way to left-align.
    fn from_printf(flags: &str, width: usize, prec: Option<usize>, ty: char) -> Spec {
        assert!(!flags.contains('#') && !flags.contains('0'),
                "pyfmt: printf flags {flags:?} are outside the census");
        let sign = if flags.contains('+') { Some('+') } else if flags.contains(' ') { Some(' ') } else { None };
        let align = Some(if flags.contains('-') { '<' } else { '>' });
        Spec { fill: ' ', align, sign, width, prec, ty: Some(ty) }
    }
}

/// Pad `body` (already carrying its sign) to `spec.width` code points. `default` is the
/// alignment the VALUE's type uses when the spec names none (numbers right, strings left).
fn pad(body: String, spec: &Spec, default: char) -> String {
    let n = body.chars().count();
    if n >= spec.width {
        return body;
    }
    let k = spec.width - n;
    let f = |m: usize| spec.fill.to_string().repeat(m);
    match spec.align.unwrap_or(default) {
        '<' => body + &f(k),
        '>' => f(k) + &body,
        '^' => f(k / 2) + &body + &f(k - k / 2),
        a => unreachable!("alignment {a:?}"),
    }
}

fn sign_prefix(negative: bool, spec: &Spec) -> &'static str {
    if negative {
        "-"
    } else {
        match spec.sign {
            Some('+') => "+",
            Some(' ') => " ",
            _ => "",
        }
    }
}

/// Split Rust's `{:e}`-style output (`d.ddde-5`) into its digit string and decimal exponent.
fn split_e(s: &str) -> (String, i32) {
    let (mant, exp) = s.split_once('e').expect("Rust's `{:e}` always emits the `e`");
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    (digits, exp.parse().expect("Rust's exponent field is a bare signed integer"))
}

fn py_exp(e: i32) -> String {
    format!("e{}{:02}", if e < 0 { '-' } else { '+' }, e.abs())
}

/// Python's `repr(float)` of a FINITE, NON-NEGATIVE value: the shortest round-trip digits, in
/// fixed notation for a decimal exponent in `[-4, 16)` and exponent notation outside it.
fn repr_abs(a: f64) -> String {
    if a == 0.0 {
        return "0.0".to_string();
    }
    let (digits, e) = split_e(&format!("{:e}", a));
    if !(-4..16).contains(&e) {
        let mant = if digits.len() > 1 { format!("{}.{}", &digits[..1], &digits[1..]) } else { digits };
        return format!("{mant}{}", py_exp(e));
    }
    if e >= 0 {
        let k = (e + 1) as usize;
        if digits.len() <= k {
            format!("{digits}{}.0", "0".repeat(k - digits.len()))
        } else {
            format!("{}.{}", &digits[..k], &digits[k..])
        }
    } else {
        format!("0.{}{digits}", "0".repeat((-e - 1) as usize))
    }
}

/// `repr(x)` for a float.
pub fn repr_f64(x: f64) -> String {
    if x.is_nan() {
        return "nan".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf" } else { "-inf" }.to_string();
    }
    let body = repr_abs(x.abs());
    if x.is_sign_negative() { format!("-{body}") } else { body }
}

/// The body (no sign) of a finite non-negative value under type `ty` at precision `p`.
fn float_body(a: f64, ty: char, p: usize) -> String {
    match ty {
        'f' => format!("{:.*}", p, a),
        'e' => {
            let s = format!("{:.*e}", p, a);
            let (mant, exp) = s.split_once('e').unwrap();
            format!("{mant}{}", py_exp(exp.parse().unwrap()))
        }
        'g' => {
            // CPython: precision 0 is treated as 1; round to p significant digits, read the
            // exponent X off the ROUNDED value, fixed iff -4 <= X < p, then strip zeros. The
            // fixed form is laid out from the SAME rounded digits, never from a second rounding.
            let p = p.max(1);
            let (digits, e) = split_e(&format!("{:.*e}", p - 1, a));
            let strip = |t: String| -> String {
                if t.contains('.') { t.trim_end_matches('0').trim_end_matches('.').to_string() } else { t }
            };
            if e >= -4 && (e as i64) < p as i64 {
                let body = if e >= 0 {
                    let k = (e + 1) as usize;
                    if digits.len() <= k {
                        format!("{digits}{}", "0".repeat(k - digits.len()))
                    } else {
                        format!("{}.{}", &digits[..k], &digits[k..])
                    }
                } else {
                    format!("0.{}{digits}", "0".repeat((-e - 1) as usize))
                };
                strip(body)
            } else {
                let mant = strip(if digits.len() > 1 {
                    format!("{}.{}", &digits[..1], &digits[1..])
                } else {
                    digits
                });
                format!("{mant}{}", py_exp(e))
            }
        }
        _ => unreachable!("float type {ty:?}"),
    }
}

/// `format(x, spec)` for a float.
pub fn format_f64(x: f64, spec: &Spec) -> String {
    let ty = spec.ty;
    assert!(!matches!(ty, Some('d') | Some('s')), "pyfmt: Python refuses format(float, {:?})", ty);
    // Python's `%` type is `f` of `x * 100.0` — a real multiply, so it is kept as one.
    let (x, core_ty, suffix) = match ty {
        Some('%') => (x * 100.0, Some('f'), "%"),
        _ => (x, ty, ""),
    };
    let negative = x.is_sign_negative() && !x.is_nan();
    let body = if x.is_nan() {
        "nan".to_string()
    } else if x.is_infinite() {
        "inf".to_string()
    } else {
        match core_ty {
            None => {
                assert!(spec.prec.is_none(), "pyfmt: a precision without a type is outside the census");
                repr_abs(x.abs())
            }
            Some(t) => float_body(x.abs(), t, spec.prec.unwrap_or(6)),
        }
    };
    pad(format!("{}{body}{suffix}", sign_prefix(negative, spec)), spec, '>')
}

/// `format(n, spec)` for an int. A float type converts it to float first, as Python does.
pub fn format_int(n: i128, spec: &Spec) -> String {
    match spec.ty {
        None | Some('d') => {
            assert!(spec.prec.is_none(), "pyfmt: Python refuses a precision on an int");
            pad(format!("{}{}", sign_prefix(n < 0, spec), n.unsigned_abs()), spec, '>')
        }
        Some('s') => panic!("pyfmt: Python refuses format(int, 's')"),
        Some(_) => format_f64(n as f64, spec),
    }
}

/// `format(s, spec)` for a str.
pub fn format_str(s: &str, spec: &Spec) -> String {
    assert!(matches!(spec.ty, None | Some('s')), "pyfmt: Python refuses format(str, {:?})", spec.ty);
    assert!(spec.sign.is_none(), "pyfmt: Python refuses a sign on a str");
    assert!(spec.prec.is_none(), "pyfmt: a str precision (truncation) is outside the census");
    pad(s.to_string(), spec, '<')
}

/// Python's `repr(str)` for the strings the panels print: single quotes unless the text holds
/// a single quote and no double one. Escapes are outside the census, so a backslash or a
/// control character panics rather than being spelled wrong.
pub fn repr_str(s: &str) -> String {
    assert!(!s.chars().any(|c| c == '\\' || c.is_control()), "pyfmt: repr of {s:?} needs escapes");
    if s.contains('\'') && !s.contains('"') {
        format!("\"{s}\"")
    } else {
        assert!(!s.contains('\''), "pyfmt: repr of {s:?} needs escapes");
        format!("'{s}'")
    }
}

/// A value a panel prints — Python's `format`, `str` and `repr` for one Rust type.
pub trait PyFormat {
    /// `format(x, spec)`.
    fn py_format(&self, spec: &Spec) -> String;
    /// `str(x)`.
    fn py_str(&self) -> String;
    /// `repr(x)`.
    fn py_repr(&self) -> String { self.py_str() }
    /// `'%<conv>' % x` for one conversion. The default covers the numeric types.
    fn py_printf(&self, spec: &Spec) -> String {
        match spec.ty {
            Some('s') => pad(self.py_str(), spec, '>'),
            Some('r') => pad(self.py_repr(), &Spec { ty: Some('s'), ..*spec }, '>'),
            _ => self.py_format(spec),
        }
    }
}

impl PyFormat for f64 {
    fn py_format(&self, spec: &Spec) -> String { format_f64(*self, spec) }
    fn py_str(&self) -> String { repr_f64(*self) }
    fn py_printf(&self, spec: &Spec) -> String {
        match spec.ty {
            Some('s') => pad(repr_f64(*self), spec, '>'),
            Some('r') => pad(repr_f64(*self), &Spec { ty: Some('s'), ..*spec }, '>'),
            // `'%d' % 3.7` is `int(3.7)`: TRUNCATION toward zero, to an unbounded int — so the
            // digits come from `{:.0}` of the (already integral) truncated value, never from a
            // cast that saturates at 2^127. NaN/inf raise in Python.
            Some('d') => {
                assert!(self.is_finite(), "pyfmt: Python raises on '%d' % {self}");
                assert!(spec.prec.is_none(), "pyfmt: a '%.Nd' precision is outside the census");
                let t = self.trunc();
                pad(format!("{}{:.0}", sign_prefix(t < 0.0, spec), t.abs()), spec, '>')
            }
            // PyPy keeps Python 2's rule that '%f' of a value at or above ~1e50 is written as '%g'
            // (`'%.2f' % 1e300` is `'1e+300'`); CPython 3 dropped it. No panel prints anything
            // near that, so the quirk is REFUSED rather than reproduced — `tests/pyfmt.rs` counts
            // the oracle cells it exempts.
            Some('f') if self.is_finite() && self.abs() >= 1e50 => {
                panic!("pyfmt: PyPy's '%f'-at-1e50 rule is outside the panels' range ({self:e})")
            }
            _ => format_f64(*self, spec),
        }
    }
}

macro_rules! int_impl {
    ($($t:ty),*) => {$(
        impl PyFormat for $t {
            fn py_format(&self, spec: &Spec) -> String { format_int(*self as i128, spec) }
            fn py_str(&self) -> String { self.to_string() }
        }
    )*};
}
int_impl!(i32, i64, u32, u64, usize, isize);

impl PyFormat for bool {
    /// Bare it is `True`/`False`; with ANY non-empty spec Python formats it as the int it is.
    fn py_format(&self, spec: &Spec) -> String {
        if *spec == Spec::parse("") { self.py_str() } else { format_int(*self as i128, spec) }
    }
    fn py_str(&self) -> String { if *self { "True" } else { "False" }.to_string() }
    fn py_printf(&self, spec: &Spec) -> String {
        match spec.ty {
            Some('s') | Some('r') => pad(self.py_str(), &Spec { ty: Some('s'), ..*spec }, '>'),
            _ => format_int(*self as i128, spec),
        }
    }
}

impl PyFormat for str {
    fn py_format(&self, spec: &Spec) -> String { format_str(self, spec) }
    fn py_str(&self) -> String { self.to_string() }
    fn py_repr(&self) -> String { repr_str(self) }
    fn py_printf(&self, spec: &Spec) -> String {
        match spec.ty {
            Some('s') => {
                assert!(spec.prec.is_none(), "pyfmt: '%.Ns' truncation is outside the census");
                pad(self.to_string(), spec, '>')
            }
            Some('r') => pad(repr_str(self), &Spec { ty: Some('s'), ..*spec }, '>'),
            t => panic!("pyfmt: Python refuses '%{}' % str", t.unwrap_or('?')),
        }
    }
}

/// A reference prints as what it refers to (Python has no references to tell apart).
impl<T: PyFormat + ?Sized> PyFormat for &T {
    fn py_format(&self, spec: &Spec) -> String { (**self).py_format(spec) }
    fn py_str(&self) -> String { (**self).py_str() }
    fn py_repr(&self) -> String { (**self).py_repr() }
    fn py_printf(&self, spec: &Spec) -> String { (**self).py_printf(spec) }
}

impl PyFormat for String {
    fn py_format(&self, spec: &Spec) -> String { self.as_str().py_format(spec) }
    fn py_str(&self) -> String { self.clone() }
    fn py_repr(&self) -> String { self.as_str().py_repr() }
    fn py_printf(&self, spec: &Spec) -> String { self.as_str().py_printf(spec) }
}

/// Python's `None`: `Option<T>` prints `None` when empty and as `T` otherwise. A spec on `None`
/// is a `TypeError` in Python, so it panics here.
impl<T: PyFormat> PyFormat for Option<T> {
    fn py_format(&self, spec: &Spec) -> String {
        match self {
            Some(v) => v.py_format(spec),
            None => {
                assert!(*spec == Spec::parse(""), "pyfmt: Python refuses format(None, {spec:?})");
                "None".to_string()
            }
        }
    }
    fn py_str(&self) -> String { self.as_ref().map_or_else(|| "None".to_string(), |v| v.py_str()) }
    fn py_repr(&self) -> String { self.as_ref().map_or_else(|| "None".to_string(), |v| v.py_repr()) }
    fn py_printf(&self, spec: &Spec) -> String {
        match self {
            Some(v) => v.py_printf(spec),
            None => {
                assert!(matches!(spec.ty, Some('s') | Some('r')), "pyfmt: Python refuses '%{:?}' % None", spec.ty);
                pad("None".to_string(), &Spec { ty: Some('s'), ..*spec }, '>')
            }
        }
    }
}

/// An already-rendered container (a tuple, list or dict): its `str` and `repr` are the same
/// text, so it nests inside another container unquoted. Python refuses a non-empty spec on a
/// container — port `f"{str(t):>9}"` as `pyf!("{:>9}", t.py_str())`, which formats the STR.
#[derive(Clone, Debug, PartialEq)]
pub struct PyRaw(pub String);

impl PyFormat for PyRaw {
    fn py_format(&self, spec: &Spec) -> String {
        assert!(*spec == Spec::parse(""), "pyfmt: Python refuses a spec on a container: {spec:?}");
        self.0.clone()
    }
    fn py_str(&self) -> String { self.0.clone() }
    fn py_printf(&self, spec: &Spec) -> String {
        assert!(matches!(spec.ty, Some('s') | Some('r')), "pyfmt: a container only takes '%s'/'%r'");
        pad(self.0.clone(), &Spec { ty: Some('s'), ..*spec }, '>')
    }
}

/// `str(tuple)` — `(a, b)`, `(a,)`, `()` — each element by its `repr`.
pub fn py_tuple(items: &[&dyn PyFormat]) -> PyRaw {
    let inner: Vec<String> = items.iter().map(|v| v.py_repr()).collect();
    PyRaw(if inner.len() == 1 { format!("({},)", inner[0]) } else { format!("({})", inner.join(", ")) })
}

/// `str(list)` — `[a, b]` — each element by its `repr`.
pub fn py_list(items: &[&dyn PyFormat]) -> PyRaw {
    let inner: Vec<String> = items.iter().map(|v| v.py_repr()).collect();
    PyRaw(format!("[{}]", inner.join(", ")))
}

/// `str(dict)` — `{k: v, …}` in the order given (Python's insertion order), each side by `repr`.
pub fn py_dict(items: &[(&dyn PyFormat, &dyn PyFormat)]) -> PyRaw {
    let inner: Vec<String> = items.iter().map(|(k, v)| format!("{}: {}", k.py_repr(), v.py_repr())).collect();
    PyRaw(format!("{{{}}}", inner.join(", ")))
}

/// The engine behind [`pyf!`]: a Python f-string with every `{expr}` hoisted into `args`, in
/// order. Accepts `{}`, `{:spec}`, `{!r}`, `{!r:spec}`, and the `{{`/`}}` escapes.
pub fn fstring(template: &str, args: &[&dyn PyFormat]) -> String {
    let mut out = String::with_capacity(template.len() + 8 * args.len());
    let mut it = template.chars().peekable();
    let mut k = 0;
    while let Some(ch) = it.next() {
        match ch {
            '{' if it.peek() == Some(&'{') => { it.next(); out.push('{'); }
            '}' if it.peek() == Some(&'}') => { it.next(); out.push('}'); }
            '}' => panic!("pyf!: a lone `}}` in {template:?}"),
            '{' => {
                let mut field = String::new();
                loop {
                    match it.next() {
                        Some('}') => break,
                        Some(c) => field.push(c),
                        None => panic!("pyf!: an unclosed `{{` in {template:?}"),
                    }
                }
                let (conv, spec) = match field.split_once(':') {
                    Some((c, s)) => (c, s),
                    None => (field.as_str(), ""),
                };
                let v = *args.get(k).unwrap_or_else(|| panic!("pyf!: too few arguments for {template:?}"));
                k += 1;
                let spec = Spec::parse(spec);
                match conv {
                    "" => out.push_str(&v.py_format(&spec)),
                    "!r" => out.push_str(&format_str(&v.py_repr(), &spec)),
                    c => panic!("pyf!: field {c:?} — hoist the expression into an argument"),
                }
            }
            c => out.push(c),
        }
    }
    assert!(k == args.len(), "pyf!: {} arguments for {k} fields in {template:?}", args.len());
    out
}

/// The engine behind [`pct!`]: Python's `template % (args…)`. Accepts `%%` and conversions
/// `%[flags][width][.prec]type` with flags `-+ ` and types `s r d f e g`.
pub fn printf(template: &str, args: &[&dyn PyFormat]) -> String {
    let c: Vec<char> = template.chars().collect();
    let mut out = String::with_capacity(template.len() + 8 * args.len());
    let (mut i, mut k) = (0, 0);
    while i < c.len() {
        if c[i] != '%' {
            out.push(c[i]);
            i += 1;
            continue;
        }
        i += 1;
        if c.get(i) == Some(&'%') {
            out.push('%');
            i += 1;
            continue;
        }
        let mut flags = String::new();
        while i < c.len() && matches!(c[i], '-' | '+' | ' ' | '#' | '0') {
            flags.push(c[i]);
            i += 1;
        }
        let mut width = 0usize;
        while i < c.len() && c[i].is_ascii_digit() {
            width = width * 10 + c[i].to_digit(10).unwrap() as usize;
            i += 1;
        }
        let mut prec = None;
        if c.get(i) == Some(&'.') {
            i += 1;
            let mut p = 0usize;
            while i < c.len() && c[i].is_ascii_digit() {
                p = p * 10 + c[i].to_digit(10).unwrap() as usize;
                i += 1;
            }
            prec = Some(p);
        }
        let ty = *c.get(i).unwrap_or_else(|| panic!("pct!: a dangling `%` in {template:?}"));
        assert!(matches!(ty, 's' | 'r' | 'd' | 'f' | 'e' | 'g'), "pct!: conversion {ty:?} is outside the census");
        i += 1;
        let v = *args.get(k).unwrap_or_else(|| panic!("pct!: too few arguments for {template:?}"));
        k += 1;
        out.push_str(&v.py_printf(&Spec::from_printf(&flags, width, prec, ty)));
    }
    assert!(k == args.len(), "pct!: not all arguments converted in {template:?}");
    out
}

/// A Python f-string: `pyf!("{:>8} {:.1f}", label, x)` is `f"{label:>8} {x:.1f}"`.
#[macro_export]
macro_rules! pyf {
    ($t:expr $(, $a:expr)* $(,)?) => {
        $crate::pyfmt::fstring($t, &[$(&$a as &dyn $crate::pyfmt::PyFormat),*])
    };
}

/// A Python `%`-template: `pct!("%-10s %.2f", a, b)` is `"%-10s %.2f" % (a, b)`.
#[macro_export]
macro_rules! pct {
    ($t:expr $(, $a:expr)* $(,)?) => {
        $crate::pyfmt::printf($t, &[$(&$a as &dyn $crate::pyfmt::PyFormat),*])
    };
}

/// Python's `print` into a buffer: each call appends the text and one `\n` (LF — the golden is
/// the LF form of the PyPy capture, § 8.1 (ii)). The CLI drains it to stdout panel by panel.
#[derive(Default, Debug)]
pub struct Printer {
    buf: String,
}

impl Printer {
    pub fn new() -> Self { Printer::default() }
    /// `print(s)`.
    pub fn print(&mut self, s: impl AsRef<str>) {
        self.buf.push_str(s.as_ref());
        self.buf.push('\n');
    }
    /// Everything printed so far.
    pub fn as_str(&self) -> &str { &self.buf }
    /// Take what has been printed, leaving the buffer empty.
    pub fn take(&mut self) -> String { std::mem::take(&mut self.buf) }
}
