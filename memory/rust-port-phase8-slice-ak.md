---
name: rust-port-phase8-slice-ak
description: "Phase 8 slice AK (2026-10-02): goldens captured, the pyfmt shim, first 8 main.py steps byte-exact — a byte gate certifies PRINTED text, not the arithmetic under it"
metadata:
  type: project
---

Slice AK of phase 8 (the full `main.py` port the user chose 2026-10-02), plan § 8.2. Shipped:
the PyPy stdout golden + an 86-row segment map (`rust/oracle/main_stdout.txt`,
`main_segments.tsv`), the T–s chart's arrays recorded AT the matplotlib call
(`ts_diagram_pypy.tsv`), `data.json` re-run under PyPy (byte-identical — no republish), the
`rust/src/pyfmt.rs` shim (72 666 cells vs Python), `rust/src/panels/` + `src/main.rs` with steps
1–8 byte-exact first compile, gated SEGMENT-exact by `tests/cli_golden.rs`.

**Process lessons (one per bullet):**
- **A byte gate certifies the PRINTED text, not the arithmetic under it.** Measured: re-associating
  `f * M_AIR / M_CH2` in the rung-6 AFT bisection PASSED (printed to `.1f`); a text change FAILED.
  Panel-local arithmetic is held by porting rules on reading, not by this gate — say so per slice.
- **The oracle interpreter has its own quirks.** PyPy keeps Python 2's `'%f'`→`'%g'` switch above
  ~1e50 (CPython 3.14 does not). REFUSE (panic) what the golden never exercises and COUNT the
  exempt oracle cells, rather than reproducing it or silently dropping rows.
- **Run the format census BEFORE writing a formatter** (advisor's point): the AST walk fixed the
  grammar (no `0`/`#`/grouping, one-arg `print`), so out-of-census specs panic instead of guessing.
- **Instrument one run for several captures** — the 30-min `main.py` run produced the segment map
  AND the chart arrays AND re-confirmed determinism (stdout byte-equal to the day-old golden).

**How to apply (slices AL–AQ):** extend `PANELS` in `main()` order, move `PORTED`, keep each
expression's operation order and `powp`, and expect the gate to miss sub-print-precision slips.
Related: [[rust-port-status]], [[instrument-fed-by-what-it-certifies]], [[rust-port-power-spelling]].
