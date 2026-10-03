---
name: rust-port-phase8-slice-ar
description: "Phase 8 slice AR (2026-10-03): the visuals (data.json + both pages) and the T–s chart byte-exact in Rust on the first run; PORTED=86 with the last line RE-CUT on purpose — and a Copy-Item restore kept an OLD mtime, so cargo ran a stale mutant"
metadata:
  node_type: memory
  type: project
  originSessionId: 84959ca5-27c2-4ae4-8e54-395e8944abc9
  modified: 2026-10-03T15:28:41.156Z
---

Slice AR of phase 8, plan § 8.9. `rust/src/visuals.rs` (a JSON value type, writer and reader;
Python's `round`; `build_data`; the two splices; `ts_diagram`) plus `rust/tests/visuals.rs` (the
14 Python gates under their own names, plus six). New CLI subcommands: `visuals [DIR]`, `splice
[DIR]`, `ts-diagram`. The new root script `plot_ts_diagram.py` only draws. `data.json` and both
pages were byte-identical on the first run (17.6 s against PyPy's 94 s). The PNG drawn from the
Rust JSON is byte-identical to the one `main.plot_ts_diagram` draws and to the committed one.

**Process lessons:**
- **`Copy-Item` keeps the SOURCE's mtime, so a restore from a backup is invisible to cargo.**
  The injection loop restored with `Copy-Item`. When the next mutation failed to apply, cargo
  did not rebuild and re-ran the PREVIOUS mutant's binary. That produced a convincing "FAILED"
  carrying another injection's message. Caught because the `.err` log had no
  `Compiling turbojet` line. Restore by WRITING the file (`[IO.File]::WriteAllText`, a Python
  write) or `touch` it, and check each mutant run's log for `Compiling`.
- **An injection whose text occurs twice silently does not apply**, if the injector asserts
  `count == 1` and the loop does not stop on that error. Make the loop stop on an injector
  failure.
- **Predict the survivors as well as the kills.** `(4·k)·k` against `4·(k·k)` cannot differ:
  a multiply by a power of two is exact. My own module doc had claimed that association
  mattered; the predicted survivor exposed the false comment.
- **A hand-written census replacing a regex: pin it to the regex's own output.** There is no
  regex crate, so the matchers are hand-written. While Python still exists, `census.py` recorded
  the exact sets each Python pattern finds, and the Rust tests assert equality with them.
- **Re-cut a golden line in the GATE, never in the golden.** `cli_golden.rs` holds a named
  `RECUT` (the old text is asserted to be exactly the golden's segment, and the panel is
  compared to the new text), so the PyPy capture and its fingerprint stay untouched.
- **The TSV oracle holds arrays, not the picture.** Styles, call order and the title were checked
  by rendering the PNG twice under PyPy and comparing bytes. A one-line-width mutation moved the
  hash, so the comparison was shown to be sensitive.

**How to apply (AS onward):** restore mutants by writing the file; stop the loop on an
injector error; pin any hand-written parser to its Python twin's output while Python exists.
Related: [[rust-port-phase8-slice-aq]], [[rust-port-status]], [[windows-tooling-file-hazards]].
