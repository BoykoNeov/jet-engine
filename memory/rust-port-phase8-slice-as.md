---
name: rust-port-phase8-slice-as
description: "Phase 8 slice AS (2026-10-03): the numeric fingerprint re-anchored on Rust (45 kernels, 35 075 values, Rust == PyPy bit for bit, anchor byte-identical to the PyPy capture) and the 17 fragile-rung entries adjudicated (all survive, no gate added) — and a doc claim written before the table measured it"
metadata:
  node_type: memory
  type: project
  originSessionId: 68e1b24a-4d5d-4168-bbf0-e7030422c754
  modified: 2026-10-03T17:31:50.834Z
---

Slice AS of phase 8, plan § 8.10 (part vi) and § 8.11 (part vii). Commits `97af0d5` (vi) and the
follow-up (review fixes + vii).

**What shipped.** `rust/tests/fingerprint.rs` + `rust/tests/fingerprint_support/mod.rs` port all 45
kernels of `tests/test_numeric_fingerprint.py`, key for key. Three files: the PyPy capture
(`rust/oracle/fingerprint_pypy.tsv`, written by `rust/oracle/dump_fingerprint.py` calling the
module's OWN `KERNELS` — never a hand reconstruction of the keys), the Rust anchor
(`fingerprint_rust.tsv`, written ONLY by `FINGERPRINT_REGEN=1 … regenerate_anchor`, byte-identical
to the capture), and the deviation table (`fingerprint_deviation.tsv`, recomputed every run). The
CPython JSON is untouched. 55 tests, ~55 s; regeneration ~180 s.

**Process lessons:**
- **Capture the target in the oracle's own key space first.** Running the Python module's own
  kernel table under PyPy gave the exact 35 075 keys (`dir()`-walked properties, `:g` spellings,
  `repr` float keys) — 44 of 45 kernels then matched on their first Rust run.
- **Reuse a verified serialization by re-parsing it, not by re-describing structs.** Slice AJ's
  `Flat` stream carries `keys:N` / `len:N`, so `tree_from_flat` rebuilt rungs 81–82's dicts with
  zero new converters; all five matched first try. The older oracles (Z–AI) hand-list keys and
  could not be reused this way.
- **A doc claim must not get ahead of its instrument.** § 8.10 said r70's 22.2 % was "on its
  absolute leg" before the table recorded which leg decided — the advisor caught it; a `leg`
  column was added and the measurement then agreed. Same family as [[instrument-fed-by-what-it-certifies]].
- **The Bash-heredoc-into-Python hazard recurred** ([[windows-tooling-file-hazards]]): `\t` / `\n`
  escapes inside a Python string literal written through a heredoc became REAL tabs and newlines
  in the Rust source. Fix that worked: write the snippet with the Write tool, splice it with a
  Python script that reads it from the file.
- **Read an old CPython record by MEANING.** Z–AI store counts and float bits alike as `u64`, so a
  raw diff cannot tell a count flip from drift: read location keys by name, thresholds by side of
  1, orderings as orderings, sweeps as rise/fall patterns.
- The pre-flight's "≤ 10.5 % of budget everywhere" was half right: slices 1–2 ≤ 10.45 %, the
  control ladder up to 22.2 % (inside its own ≥ 4× rule). The count is 35 075, not 35 089.
