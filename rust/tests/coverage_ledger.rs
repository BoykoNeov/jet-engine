//! PHASE 8, slice AT — **THE COVERAGE LEDGER: every Python test, and where it went.**
//!
//! `coverage_ledger.tsv` (beside this file) rows every `def test_` the Python suite had at the
//! last commit that still carried it — **1 203 functions, 1 387 collected cases, 92 files** — and
//! says, for each, which Rust `#[test]` carries it now, or why none does. It is what the delete at
//! slice AU stands on (plan § 8.1 (v)): a count of tests cannot say whether a missing test was
//! renamed, merged, looped or simply never written, and only a row per test can.
//!
//! **The statuses, a closed list:**
//!
//! | status | meaning |
//! |---|---|
//! | `PORTED` | one Rust test carries it (the name may differ; a header table or a hand-read paired it) |
//! | `LOOPED` | a parametrized Python test whose cases ONE Rust test loops over |
//! | `SPLIT` | one Python test (or its cases / arms) spread over several Rust tests |
//! | `MERGED` | carried inside a Rust test that also carries another Python test |
//! | `NARROWED` | carried on fewer cases than the Python ran, the reason in the note |
//! | `CORRECTED` | the Python's claim was measured wrong (or a self-comparison in Rust); the Rust test gates the corrected claim |
//! | `DECLARED-NONPORT` | not carried, and the Rust file says why (a compile error, a `&self` guarantee, a vacuity) |
//! | `RE-ANCHORED` | a fingerprint kernel: compared to the CPython golden, Rust's own anchor beside it (slice AS) |
//! | `RETIRED` | a guard of Python-only material that the delete removes (plan § 8.1 (v)) |
//! | `PORTED-IN-AT` | had no Rust test until this slice wrote one |
//!
//! There is no `GAP` status: the slice ported every gap it found, so a gap cannot be committed.
//!
//! **How the rows were made** (the source run is `W:\temp\claude\slice_at`, plan § 8.12). Python
//! tests by AST, Rust tests by `cargo test -- --list`; a name tier (exact, gate-prefixed, reworded),
//! then the porters' OWN header tables (`| test_x | [`fn`] |` lines), then a hand-read of every
//! row neither settled, and a literal census of every parametrized test's case values in the Rust
//! body. **What the ledger cannot see** is an assertion dropped INSIDE a test that kept its name: a
//! row says where a test went, not that every line of it went too.
//!
//! **What this file gates**, so the ledger cannot rot after the Python is gone: the frozen counts,
//! the closed status list, a reason on every row that is not a plain port, and — the check that
//! keeps it honest under renames — every cited `file::fn` is a `#[test]` in `rust/tests/file.rs`
//! today. The last test shows the checker failing on a corrupted copy, so a green run is a
//! measurement and not a detector that cannot see.

use std::collections::HashSet;
use std::path::PathBuf;

const LEDGER: &str = include_str!("coverage_ledger.tsv");
const HEADER: &str = "py_file\tpy_test\tpy_cases\tstatus\trust\tnote";
/// Frozen at the last Python commit (`pytest --collect-only`: 1 387 IDs from 1 203 functions).
const N_FUNCTIONS: usize = 1203;
const N_CASES: usize = 1387;
const N_FILES: usize = 92;
const STATUSES: [&str; 10] = [
    "PORTED", "LOOPED", "SPLIT", "MERGED", "NARROWED", "CORRECTED", "DECLARED-NONPORT",
    "RE-ANCHORED", "RETIRED", "PORTED-IN-AT",
];
/// The statuses whose row may cite no Rust test at all.
const NO_RUST_OK: [&str; 2] = ["DECLARED-NONPORT", "RETIRED"];

/// Every `#[test]` function name in a Rust test file's source: `#[test]` followed (past further
/// attributes and doc lines) by `fn NAME`, plus the entries of a `gate! { name => … }` block —
/// `fingerprint.rs` generates its 45 kernel tests that way.
fn test_fns(src: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let lines: Vec<&str> = src.lines().map(str::trim).collect();
    for (i, l) in lines.iter().enumerate() {
        if let Some(rest) = l.strip_prefix("#[test]") {
            // `#[test] fn name() { … }` on ONE line is how rungs 62/63 spell a pytest case each.
            let rest = rest.trim();
            let same_line = if rest.is_empty() { None } else { Some(rest) };
            for m in same_line.into_iter().chain(lines[i + 1..].iter().copied()) {
                if m.starts_with("#[") || m.starts_with("//") {
                    continue;
                }
                let m = m.strip_prefix("pub ").unwrap_or(m);
                if let Some(rest) = m.strip_prefix("fn ") {
                    let name: String =
                        rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                    out.insert(name);
                }
                break;
            }
        }
    }
    if let Some(start) = src.find("\ngate! {") {
        let body = &src[start..];
        let body = &body[..body.find("\n}").unwrap_or(body.len())];
        for part in body.split(',') {
            if let Some((name, _)) = part.split_once("=>") {
                let name = name.trim().trim_start_matches("gate! {").trim();
                out.insert(name.to_string());
            }
        }
    }
    out
}

fn rust_tests_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// Every problem with a ledger text, given a lookup `(file, fn) -> is a #[test]`. Empty = clean.
fn check(ledger: &str, is_test: &mut dyn FnMut(&str, &str) -> bool) -> Vec<String> {
    let mut errs = Vec::new();
    let mut lines = ledger.lines();
    if lines.next() != Some(HEADER) {
        errs.push("header row changed".to_string());
    }
    let (mut n, mut cases) = (0usize, 0usize);
    let mut seen = HashSet::new();
    let mut files = HashSet::new();
    for (k, line) in lines.enumerate() {
        let row = k + 2;
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() != 6 {
            errs.push(format!("row {row}: {} columns, not 6", c.len()));
            continue;
        }
        let (file, test, n_cases, status, rust, note) = (c[0], c[1], c[2], c[3], c[4], c[5]);
        n += 1;
        files.insert(file);
        match n_cases.parse::<usize>() {
            Ok(x) if x >= 1 => cases += x,
            _ => errs.push(format!("row {row}: py_cases {n_cases:?} is not a count")),
        }
        if !seen.insert((file, test)) {
            errs.push(format!("row {row}: {file}::{test} appears twice"));
        }
        if !STATUSES.contains(&status) {
            errs.push(format!("row {row}: status {status:?} is not in the closed list"));
        }
        if status != "PORTED" && note.trim().is_empty() {
            errs.push(format!("row {row}: {status} {file}::{test} carries no reason"));
        }
        if rust.is_empty() {
            if !NO_RUST_OK.contains(&status) {
                errs.push(format!("row {row}: {status} {file}::{test} cites no Rust test"));
            }
            continue;
        }
        for r in rust.split(';') {
            match r.split_once("::") {
                Some((f, t)) if is_test(f, t) => {}
                _ => errs.push(format!("row {row}: {file}::{test} cites {r:?}, which is not a #[test]")),
            }
        }
    }
    if n != N_FUNCTIONS {
        errs.push(format!("{n} rows, not the {N_FUNCTIONS} Python test functions"));
    }
    if cases != N_CASES {
        errs.push(format!("{cases} cases, not the {N_CASES} pytest collected"));
    }
    if files.len() != N_FILES {
        errs.push(format!("{} Python files, not {N_FILES}", files.len()));
    }
    errs
}

/// A lookup over the real `rust/tests/*.rs`, each file read once.
fn disk_lookup() -> impl FnMut(&str, &str) -> bool {
    let mut cache: std::collections::HashMap<String, HashSet<String>> = Default::default();
    move |f: &str, t: &str| {
        let fns = cache.entry(f.to_string()).or_insert_with(|| {
            std::fs::read_to_string(rust_tests_dir().join(format!("{f}.rs")))
                .map(|s| test_fns(&s))
                .unwrap_or_default()
        });
        fns.contains(t)
    }
}

#[test]
fn the_ledger_rows_every_python_test_and_every_row_is_explained() {
    let errs = check(LEDGER, &mut disk_lookup());
    assert!(errs.is_empty(), "{} ledger problems:\n{}", errs.len(), errs.join("\n"));
}

/// The scanner finds what cargo lists. Pinned on two files with every shape it must read — the
/// `gate!` macro (45 kernels + 11 plain tests) and `#[should_panic]` between `#[test]` and `fn`.
#[test]
fn the_test_scanner_sees_the_shapes_the_ledger_cites() {
    let fp = test_fns(&std::fs::read_to_string(rust_tests_dir().join("fingerprint.rs")).unwrap());
    assert_eq!(fp.len(), 56, "fingerprint.rs: cargo lists 56 tests");
    assert!(fp.contains("k_r82t") && fp.contains("k_cpg") && fp.contains("coverage"));
    let r8 = test_fns(&std::fs::read_to_string(rust_tests_dir().join("rung8.rs")).unwrap());
    assert!(r8.contains("phi_primary_guard_rejects_25"), "a #[should_panic] test was missed");
    // one-line `#[test] fn name() { … }` — rung 62 spells most of its pytest cases this way
    let r62 = test_fns(&std::fs::read_to_string(rust_tests_dir().join("rung62.rs")).unwrap());
    assert_eq!(r62.len(), 58, "rung62.rs: cargo lists 58 tests");
}

/// THE CHECKER, SHOWN TO SEE. Each corruption a later edit could make is applied to a copy and
/// must be reported; the real ledger, through the same function, reports nothing.
#[test]
fn the_checker_fails_on_a_corrupted_ledger() {
    let mut lookup = disk_lookup();
    assert!(check(LEDGER, &mut lookup).is_empty());

    // a cited Rust test renamed away
    let renamed = LEDGER.replacen("rung61::gate5_rung60_tautology_third_route", "rung61::gate5_renamed", 1);
    let e = check(&renamed, &mut lookup);
    assert!(e.len() == 1 && e[0].contains("rung61::gate5_renamed"), "{e:?}");

    // a non-PORTED row's reason blanked
    let row = LEDGER.lines().find(|l| l.contains("\tNARROWED\t")).unwrap();
    let blank = row.rsplit_once('\t').unwrap().0.to_string() + "\t";
    let e = check(&LEDGER.replacen(row, &blank, 1), &mut lookup);
    assert!(e.len() == 1 && e[0].contains("carries no reason"), "{e:?}");

    // a status outside the closed list — a GAP cannot be committed
    let gap = row.replacen("\tNARROWED\t", "\tGAP\t", 1);
    let e = check(&LEDGER.replacen(row, &gap, 1), &mut lookup);
    assert!(e.iter().any(|x| x.contains("\"GAP\" is not in the closed list")), "{e:?}");

    // a row dropped
    let first = LEDGER.lines().nth(1).unwrap();
    let e = check(&LEDGER.replacen(&format!("{first}\n"), "", 1), &mut lookup);
    assert!(e.iter().any(|x| x.contains("1202 rows")), "{e:?}");
}
