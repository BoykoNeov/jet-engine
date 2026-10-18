//! Guard: `CLAUDE.md` is a REFERENCE / index, not a handout.
//!
//! The Rust port of `tests/test_claude_md_reference.py` (slice AT — plan § 8.1 (v): *"project
//! policy, not Python"*), so the budget outlives the Python suite. `CLAUDE.md` is loaded into
//! context at the start of every session, so its size is a recurring, real cost; it has twice grown
//! into a multi-hundred-KB document by accreting an essay per rung that already lived in that
//! rung's spec. This is the mechanical backstop for the file's own banner rule.
//!
//! **If this fails:** move the detail into `docs/rungN-spec.md` and leave a ONE-LINE hook — do NOT
//! raise the budget. Raise it only for real one-line-per-rung growth, deliberately, in the commit
//! that adds the content. The budget's whole history — every bump, the one drop, and which sites
//! to search for slack first — is the comment block above `MAX_BYTES` in the Python file, at the
//! `python-final` tag; it is not copied here, and the next entry belongs in THIS file.
//!
//! The constants are the Python's verbatim, and so are the two readings: BYTES (the failure mode
//! is prose volume) and lines counted as `count('\n') + 1`. A line-ending change reads as content
//! (rung 82's edit pass once added 253 B of CRLF), so any scripted edit of `CLAUDE.md` must keep
//! it LF.

/// History since the port (earlier entries: the Python file at `python-final`):
/// * 2026-10-14, rung 86: 35 970 → 36 256. The rung row (346 B) less the OPEN entry it closes,
///   plus the scope/family/layout touches — real one-line-per-rung growth; the slack stays 20 B.
/// * 2026-10-15, rung 87: 36 256 → 36 605. The rung row (322 B) plus its family label — real
///   one-line-per-rung growth; the slack stays 20 B.
/// * 2026-10-15, rung 88: 36 605 → 36 985. The rung row (345 B) plus its family label and the layout's
///   module name — real one-line-per-rung growth; the slack stays 20 B.
/// * 2026-10-15, rung 89: 36 985 → 37 358. The rung row (336 B) plus its family label and the layout's
///   module name — real one-line-per-rung growth; the slack stays 20 B.
/// * 2026-10-15, rung 90: 37 358 → 37 739. The rung row (328 B) plus its family label, the layout's
///   module name and the OPEN entry it closes rewritten — real one-line-per-rung growth; the slack
///   stays 20 B.
/// * 2026-10-15, rung 91: 37 739 → 38 092. The rung row (335 B) plus its family label, the layout's
///   module name, and the OPEN entry it closes struck — real one-line-per-rung growth; the slack
///   stays 20 B.
/// * 2026-10-15, rung 92: 38 092 → 38 418. The rung row (315 B) plus its family label, the layout's
///   module name, and the OPEN entry it closes struck — real one-line-per-rung growth; the slack
///   stays 20 B.
/// * 2026-10-16, rung 93: 38 418 → 38 795. The rung row (339 B) plus its family label, the scope lines and
///   the layout's module name — real one-line-per-rung growth; the slack stays 20 B.
/// * 2026-10-16, rung 94: 38 795 → 39 150. The rung row (319 B) plus its family label, the scope lines and the
///   layout's module name — real one-line-per-rung growth; the slack stays 20 B.
/// * 2026-10-18, rung 97: 39 150 → 39 480. The rung row (338 B) plus its family label, the scope line, the layout's
///   module name and the OPEN entry it half-closes rewritten (rungs 95/96 had fitted in the old slack) — real
///   one-line-per-rung growth; the slack stays 20 B.
const MAX_BYTES: usize = 39_480;
const MAX_LINES: usize = 300;

/// The file, as bytes. `include_bytes!` makes cargo rebuild this test whenever `CLAUDE.md` changes.
const CLAUDE_MD: &[u8] = include_bytes!("../../CLAUDE.md");

fn lines() -> usize {
    std::str::from_utf8(CLAUDE_MD).expect("CLAUDE.md must be UTF-8");
    CLAUDE_MD.iter().filter(|&&b| b == b'\n').count() + 1
}

#[test]
fn claude_md_within_byte_budget() {
    assert!(
        CLAUDE_MD.len() <= MAX_BYTES,
        "CLAUDE.md is {} bytes, over the {MAX_BYTES}-byte budget. CLAUDE.md is a reference/index — \
         rung detail belongs in docs/rungN-spec.md, not here. Move the detail out (do NOT raise the \
         budget) unless this is real one-line-per-rung growth. See this file's header.",
        CLAUDE_MD.len()
    );
}

#[test]
fn claude_md_within_line_budget() {
    let n = lines();
    assert!(
        n <= MAX_LINES,
        "CLAUDE.md is {n} lines, over the {MAX_LINES}-line budget. Keep the rung table to one line \
         per rung and 'Deferred seams' to one line per entry; the detail lives in the specs."
    );
}
