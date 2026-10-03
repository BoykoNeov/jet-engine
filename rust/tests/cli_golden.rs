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

use turbojet::panels::{Design, PANELS};
use turbojet::pyfmt::Printer;

const GOLDEN: &str = include_str!("../oracle/main_stdout.txt");
const SEGMENTS: &str = include_str!("../oracle/main_segments.tsv");

/// How many of `main()`'s steps are ported. Slice AK: the two station tables, the losses line,
/// and rungs 2b, 3, 4, 5, 6 (26 after slice AL: rungs 7–24; 39 after AM: rungs 25–37; 47 after
/// AN: rungs 38–45; 54 after AO: rungs 46–52).
const PORTED: usize = 54;

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
    assert_eq!(PANELS.len(), PORTED, "a panel was ported or dropped without moving the pin");
    let design = Design::new();
    for (i, ((name, panel), seg)) in PANELS.iter().zip(&segs).enumerate() {
        assert_eq!(*name, seg.name, "step {i}: the port's order is not main()'s");
        let mut p = Printer::new();
        panel(&mut p, &design);
        let want = &GOLDEN[seg.start..seg.end];
        let got = p.as_str();
        if got != want {
            let line = want.lines().zip(got.lines()).position(|(a, b)| a != b);
            let detail = match line {
                Some(k) => format!("first differing line {k}:\n  python: {:?}\n  rust:   {:?}",
                                   want.lines().nth(k).unwrap(), got.lines().nth(k).unwrap()),
                None => format!("same lines but lengths differ: python {} bytes, rust {} bytes", want.len(), got.len()),
            };
            panic!("step {i} ({name}) differs from the golden; {detail}");
        }
    }
}
