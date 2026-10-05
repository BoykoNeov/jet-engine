---
name: rust-port-phase8-slice-at
description: "Phase 8 slice AT (2026-10-05): the coverage ledger (1 203 Python tests rowed, gated in Rust), 8 never-ported tests found and ported, the phi-rate + CLAUDE.md gap ports, the T–s script read census — and the lesson that a MISSING porter's roster predicts a missing test"
metadata:
  node_type: memory
  type: project
  originSessionId: 3b1b6ffe-ef0f-414e-8b6e-7a185cf5fc35
  modified: 2026-10-05T11:16:58.640Z
---

Slice AT of phase 8, plan § 8.12. Commits `65b808c` and `981940e` (the advisor-forced re-check).

**What shipped.** `rust/tests/coverage_ledger.tsv` rows every Python `def test_` (1 203 functions,
1 387 cases, 92 files) under a closed status list with NO `GAP` status; `coverage_ledger.rs`
gates counts, statuses, reasons, cited-fn existence, and corrupts a copy four ways.
`rust/oracle/check_coverage_ledger.py` confirmed rows = AST, cases = pytest collection (dies at
AU). `claude_md_reference.rs`, `phi_rate_limiter_negative.rs` (walk coverage 14/16/14/16
reproduced EXACTLY — the fallible `try_instant_fuel` skips the cuts Python's `except` skips), and
`visuals.rs::the_ts_script_reads_only_keys_the_json_writes`. Source run: `W:\temp\claude\slice_at`
(match2.py → manual*.py → consolidate.py exports the TSV).

**Process lessons:**
- **A missing porter's roster predicts a missing test.** All 8 never-ported, never-declared
  tests (rungs 22 ×3, 23, 24 ×3, 28) were in the early "by gate" ports that carry no
  `| test_x | [`fn`] |` mapping table or roster; every phase-6+ file with a table was clean. When
  auditing a port, read the table-less files first.
- **A name match is not a body match.** The 108 word-overlap pairs were first accepted on names;
  the advisor blocked it. Re-checking by the NUMBERS in each body (grid values, bars) found 2
  mis-pairs and 5 narrowings, and the same check on the temperature signal found one more in a
  plain name-matched row. The number check is noisy (rung numbers, named constants) — filter by a
  telling signal before reading.
- **Measure the oracle's numbers before porting a gate that counts.** The phi-rate walk's
  `seen >= 12` cannot tell a faithful fallible twin from one that refuses early; pinning the
  MEASURED 14/16/14/16 first made the port's first green run a real check.
- **Take the test list from `cargo test -- --list`, not a regex**: the regex missed 52 (a `gate!`
  macro, multi-line attributes); the ledger's own scanner then missed one-line `#[test] fn x() {}`
  (rungs 62/63) — caught because the gate failed loudly, now pinned (rung62 = 58).
- Bash-tool escapes again ([[windows-tooling-file-hazards]]): a `\b` inside a heredoc'd Python
  regex became a literal backspace byte; fix by byte-replace or the Write tool.
