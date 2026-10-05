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

const MAX_BYTES: usize = 35_970;
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
