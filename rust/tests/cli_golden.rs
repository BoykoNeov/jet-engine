//! PHASE 8 — **the gate `main.py` never had**: every ported panel reproduces ITS OWN SEGMENT of
//! `python main.py`'s stdout, byte for byte (`docs/plans/todo-rust-port.md` § 8.1 (ii)).
//!
//! * `rust/oracle/main_stdout.txt` — the PyPy stdout of `main.py` at tree `0003753` (unchanged
//!   through `5338ac4`), in its LF form: 229 987 bytes, 3 413 lines, sha256 `fc3a287997b268d8…`.
//!   Captured ONCE: the run takes ~30 minutes and the interpreter that wrote it is deleted at the
//!   end of phase 8.
//! * `rust/oracle/main_segments.tsv` — the byte range each of `main()`'s steps wrote, recorded by
//!   an instrumented second PyPy run whose stdout was byte-identical to the first. A `print_*`
//!   call is named by its function; text `main()` prints itself is `main:<what>`.
//!
//! The gate is SEGMENT-exact, not prefix-exact: a prefix test passes an empty or truncated output,
//! and cannot say which panel moved. Here each panel must equal its own range, the ranges must
//! tile the golden with no gap, and the number of ported panels is pinned — so a slice that ports
//! a panel moves the pin on purpose.
//!
//! # The panels born in Rust (rung 85 on — `docs/plans/rung85-anchor-blade-speed.md` D6)
//!
//! A panel with no Python ancestor has no segment, so it is held to its OWN capture,
//! `rust/oracle/rust_owned/<name>.txt`, written only by `cargo run --release -- panel <name>
//! --write`. **That capture came from the code it checks: a pass says the text did not MOVE,
//! never that it is RIGHT** — the rung's own `tests/rungN.rs` carries the correctness, against
//! numbers this code did not produce. The panel count therefore splits in two: [`PORTED`] still
//! counts the panels held to the PYTHON capture (86, and it stays 86 for ever — no Python panel is
//! left to port), and [`RUST_OWNED_PANELS`] counts the ones held to a Rust capture. Which list a
//! panel is on is DECLARED ([`RUST_OWNED`]), never inferred from a missing segment.

use turbojet::panels::{Design, PANELS, RUST_OWNED};
use turbojet::pyfmt::Printer;

const GOLDEN: &str = include_str!("../oracle/main_stdout.txt");
const SEGMENTS: &str = include_str!("../oracle/main_segments.tsv");

/// How many of `main()`'s steps are ported. Slice AK: the two station tables, the losses line,
/// and rungs 2b, 3, 4, 5, 6 (26 after slice AL: rungs 7–24; 39 after AM: rungs 25–37; 47 after
/// AN: rungs 38–45; 54 after AO: rungs 46–52; 64 after AP: rungs 53–63, which has no rung-59 panel;
/// 72 after AQ's first part: rungs 64–71; 78 after its second: rungs 72–77; 85 after its third:
/// rungs 78–84 — every `print_*` panel; 86 after AR: `plot_ts_diagram`'s line, RE-CUT below).
const PORTED: usize = 86;

/// How many panels are born in Rust and held to their own capture. 0 until rung 85's panel ships.
const RUST_OWNED_PANELS: usize = 0;

/// The Python-backed panels, in order — [`PANELS`] with the declared Rust-owned names taken out.
fn python_panels() -> Vec<&'static (&'static str, turbojet::panels::Panel)> {
    PANELS.iter().filter(|(n, _)| !RUST_OWNED.contains(n)).collect()
}

fn rust_owned_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("oracle").join("rust_owned").join(format!("{name}.txt"))
}

/// The one comparison every panel goes through: `Ok` iff `got` is `want` byte for byte, else the
/// first differing line. Factored out so its power to FAIL is itself tested
/// ([`the_comparison_can_see_a_difference`]) — the Rust-owned gate is vacuous while its list is
/// empty, and must not be vacuous for want of eyes once it is not.
fn compare(label: &str, want: &str, got: &str) -> Result<(), String> {
    if got == want {
        return Ok(());
    }
    let line = want.lines().zip(got.lines()).position(|(a, b)| a != b);
    let detail = match line {
        Some(k) => format!("first differing line {k}:\n  golden: {:?}\n  rust:   {:?}",
                           want.lines().nth(k).unwrap(), got.lines().nth(k).unwrap()),
        None => format!("same lines but lengths differ: golden {} bytes, rust {} bytes", want.len(), got.len()),
    };
    Err(format!("{label} differs from its golden; {detail}"))
}

/// **The segments the port changes ON PURPOSE.** The golden is NOT edited (it stays the PyPy
/// capture, fingerprint and all): each entry names a segment, a piece of its OLD text that must
/// occur in it EXACTLY ONCE, and the NEW text the panel prints in its place — every other byte of
/// the segment is still held to the golden.
///
/// * Slice AR (`docs/plans/todo-rust-port.md` § 8.9) — `main.py` drew `ts_diagram.png` and said
///   so; the Rust CLI computes the chart's DATA and writes `ts_diagram.json`, and
///   `plot_ts_diagram.py` draws the PNG (byte-identical to the one `main.py` drew — checked when AR
///   shipped). Printing the old line would name a file this binary never writes.
/// * Rung 33's table (2026-10-06, `docs/rung33-spec.md` § The SUB-IDLE label) — Python printed
///   SUB-IDLE on ANY abort, and its 440 / 420 K rows aborted in the dispatch's choked trial (the
///   gas tables' 150 K floor), before any thrust check. The panel now runs the subsonic solve
///   directly there and labels the row by the guard that fired; the rows' text is unchanged (the
///   thrust guard DOES fire), and a three-line note says how they were reached.
const RECUTS: &[(&str, &str, &str)] = &[
    (
        "plot_ts_diagram",
        "\nT–s diagram (ideal vs real) written to ts_diagram.png\n",
        "\nT–s diagram data (ideal vs real) written to ts_diagram.json; draw it with: python plot_ts_diagram.py\n",
    ),
    (
        "print_subsonic_matching_table",
        "      420  SUB-IDLE  (net thrust <= 0: below thrust-neutral idle)\n",
        concat!(
            "      420  SUB-IDLE  (net thrust <= 0: below thrust-neutral idle)\n",
            "  (The 440/420 rows ran the subsonic solve DIRECTLY: from ~455 K down, the auto-dispatch's\n",
            "  choked trial asks the gas tables for T < 150 K and aborts BEFORE any thrust check — so\n",
            "  SUB-IDLE above is the subsonic branch's own thrust guard, not the dispatch's abort.)\n",
        ),
    ),
];

struct Segment {
    name: String,
    start: usize,
    end: usize,
}

fn segments() -> Vec<Segment> {
    SEGMENTS
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            Segment { name: f[0].to_string(), start: f[1].parse().unwrap(), end: f[2].parse().unwrap() }
        })
        .collect()
}

/// FNV-1a, 64-bit — a dependency-free fingerprint of the golden, so a re-capture or an edit to
/// the file cannot pass unnoticed. (Its sha256 is recorded in the plan.)
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3))
}

#[test]
fn the_golden_is_the_capture_it_claims_to_be() {
    assert_eq!(GOLDEN.len(), 229_987, "the golden's size moved");
    assert_eq!(GOLDEN.lines().count(), 3_413, "the golden's line count moved");
    assert!(!GOLDEN.contains('\r'), "the golden must be the LF form");
    assert_eq!(fnv1a(GOLDEN.as_bytes()), 15_355_211_642_347_498_699, "the golden's fingerprint moved");
}

#[test]
fn the_segments_tile_the_golden_in_main_order() {
    let segs = segments();
    assert_eq!(segs.first().unwrap().start, 0);
    for w in segs.windows(2) {
        assert_eq!(w[0].end, w[1].start, "a gap or overlap between {} and {}", w[0].name, w[1].name);
    }
    assert_eq!(segs.last().unwrap().end, GOLDEN.len(), "the segments do not reach the golden's end");
    for s in &segs {
        assert!(GOLDEN.is_char_boundary(s.start) && GOLDEN.is_char_boundary(s.end), "{} cuts a character", s.name);
        assert!(s.end > s.start, "{} is empty", s.name);
    }
    assert_eq!(segs.len(), 86, "85 calls from main() plus the one line it prints itself");
    assert_eq!(segs.last().unwrap().name, "plot_ts_diagram");
}

#[test]
fn every_ported_panel_reproduces_its_segment_exactly() {
    let segs = segments();
    let python = python_panels();
    assert_eq!(python.len(), PORTED, "a panel was ported or dropped without moving the pin");
    for (name, _, _) in RECUTS {
        assert!(segs.iter().any(|s| s.name == *name), "the re-cut names no segment: {name}");
    }
    let design = Design::new();
    for (i, ((name, panel), seg)) in python.into_iter().zip(&segs).enumerate() {
        assert_eq!(*name, seg.name, "step {i}: the port's order is not main()'s");
        let mut p = Printer::new();
        panel(&mut p, &design);
        let mut want = GOLDEN[seg.start..seg.end].to_string();
        for (_, old, new) in RECUTS.iter().filter(|r| r.0 == *name) {
            assert_eq!(want.matches(old).count(), 1, "step {i} ({name}): the re-cut text is not in its segment exactly once");
            want = want.replace(old, new);
        }
        if let Err(e) = compare(&format!("step {i} ({name})"), &want, p.as_str()) {
            panic!("{e}");
        }
    }
}

/// The Rust-owned list is well-formed and SITS where it must: each name once in [`PANELS`], in no
/// Python segment, and the whole block contiguous just before the chart line — so the chart line
/// stays the CLI's last output, and a later rung cannot drift in among the Python panels.
#[test]
fn the_rust_owned_panels_are_declared_and_placed() {
    assert_eq!(RUST_OWNED.len(), RUST_OWNED_PANELS, "a Rust-owned panel was added or dropped without moving the pin");
    assert_eq!(PANELS.len(), PORTED + RUST_OWNED_PANELS, "every panel is either Python-backed or declared Rust-owned");
    let segs = segments();
    let n = PANELS.len();
    assert_eq!(PANELS[n - 1].0, "plot_ts_diagram", "the chart line must stay the CLI's last output");
    for (j, name) in RUST_OWNED.iter().enumerate() {
        assert_eq!(PANELS.iter().filter(|(p, _)| p == name).count(), 1, "{name} must be in PANELS exactly once");
        assert!(!segs.iter().any(|s| s.name == *name), "{name} has a PYTHON segment — it is not Rust-owned");
        let at = PANELS.iter().position(|(p, _)| p == name).unwrap();
        assert_eq!(at, n - 1 - RUST_OWNED.len() + j, "{name} is out of place: the Rust-owned block sits, in \
                   RUST_OWNED's order, between the last Python rung panel and the chart line");
    }
}

/// Every Rust-owned panel reproduces its own capture byte for byte. Its own test, so it fails on
/// its own and runs beside the long Python sweep. A change DETECTOR only — see the module header.
#[test]
fn every_rust_owned_panel_reproduces_its_capture() {
    let design = Design::new();
    for name in RUST_OWNED {
        let path = rust_owned_path(name);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!(
            "{name}: no capture at {} ({e}); make it with `cargo run --release -- panel {name} --write`",
            path.display()));
        assert!(!bytes.starts_with(&[0xEF, 0xBB, 0xBF]), "{name}: the capture carries a byte-order mark");
        let want = String::from_utf8(bytes).unwrap_or_else(|_| panic!("{name}: the capture is not UTF-8"));
        assert!(!want.contains('\r'), "{name}: the capture must be the LF form");
        assert!(!want.is_empty(), "{name}: the capture is empty");
        let (_, panel) = PANELS.iter().find(|(p, _)| p == name).unwrap();
        let mut p = Printer::new();
        panel(&mut p, &design);
        if let Err(e) = compare(name, &want, p.as_str()) {
            panic!("{e}");
        }
    }
}

/// The comparison has EYES: it passes on equal text and fails on a one-character change, a
/// dropped line and an added one. Without this, an empty [`RUST_OWNED`] would make the gate above
/// pass having compared nothing — and a broken [`compare`] would make it pass for ever.
#[test]
fn the_comparison_can_see_a_difference() {
    let base = "row  1.000  φ – a\nrow  2.000  φ – b\n";
    assert!(compare("t", base, base).is_ok());
    assert!(compare("t", base, "row  1.000  φ – a\nrow  2.001  φ – b\n").is_err(), "a changed digit");
    assert!(compare("t", base, "row  1.000  φ – a\n").is_err(), "a dropped line");
    assert!(compare("t", base, "row  1.000  φ – a\nrow  2.000  φ – b\nextra\n").is_err(), "an added line");
    assert!(compare("t", base, "row  1.000  φ – a\r\nrow  2.000  φ – b\r\n").is_err(), "a CRLF re-encoding");
}
