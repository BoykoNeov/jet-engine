---
name: rust-port-slice-aj-step6
description: "Slice AJ step 6 (the oracle for rungs 81–84) — a census that COUNTS where a mechanism fires cannot predict its effect when it fires everywhere; remove the mechanism and demand exact equality instead (the reverse run). And a failing gate must NAME what moved, or an injection's reach is inferred"
metadata:
  node_type: memory
  type: project
  originSessionId: fb2b383d-f72c-4bcf-8257-553aa2604718
  modified: 2026-09-30T12:12:05.559Z
---

Slice AJ step 6 COMPLETE 2026-09-30 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.34.6). `rust/oracle/dump_slice_aj.py` + `rust/tests/slice_aj_oracle.rs` (5 gates), steps 2–4's
line format, converters LIFTED into `rust/tests/slice_aj_flat/mod.rs` (the four step binaries re-verify
them: 44 green). Rust ≡ PyPy on 24 323 lines, first run. Probes in `W:\temp\claude\slice-aj-step6\`.

**THE LESSON: when a mechanism fires on every run, don't census WHERE it fires — REMOVE it and
demand exact equality.** My plan was a per-reading census of CPython's `x**2` vs `x*x`
disagreements. The advisor predicted it would flag everything; one march measured 26 flips in
1.29 M calls — every reading flagged, nothing predicted. The reverse run (AST import hook
rewriting all 27 `** 2` sites to `x*x`, plus a naive `sum`, on CPython) came out BYTE-IDENTICAL to
PyPy on all 24 324 lines. That proves the explanation is complete, which no census can: it does
not enumerate sites, so it cannot miss one. Which lines the plain CPython run moves (421, all
floats) was then FROZEN, not predicted.

**Why:** a census answers "is the mechanism present"; the question was "is it the WHOLE
difference". Only removing it answers that.

**How to apply:** for any "A differs from B because of X" claim, build A-without-X and require
A-without-X == B exactly; a single surviving line names a second mechanism.

Also:
- **A gate that fails on a LENGTH mismatch cannot score an injection's reach.** J1 first
  reported "compare tokens only after the paths are gated"; `moved_readings` was added so the
  gate names every reading that differs. Then 3 of 3 injections (dropping `inc` at three
  sites) moved exactly the predicted readings, and J1 was invisible to every other binary that can reach
  `scan_cells` (grepped: the six AJ binaries that name its three modules, all run) — the
  incidence arm is the first gate that sees it. Say WHY a set of binaries is the whole crate.
- The incidence arm is live only at `r = 0.25` (probed before writing); two readings came out
  thin (`i_secant` S2 flat pair, `i_mask` vacuous) and were RECORDED, not rerun — a rerun would
  have discarded two ~50-min CPython runs already in flight.
- **A reading "aimed at" a gap is not a reading that CLOSES it — inject before claiming.** The
  `n = 4` `residual_shape` reading was written up as closing step 4's C6 (ladder spelling). The
  advisor asked for the injection; computed first, the two spellings are bit-identical at both
  `n = 4` ladders, and the mutant SURVIVED the oracle. C6 stays OPEN (271 of 589 nearby ladders
  put a point elsewhere, e.g. `[0.004, 0.024]` at `n = 11` — `rust/oracle/probe_slice_aj_step6_ladder.py`;
  that a shifted point moves a REPORTED value is still a prediction). A typed count with no
  committed script behind it was caught only on review — commit the probe with the number.
- `cargo test -- --exact-not` is not a flag; a bad libtest flag fails every binary instantly
  with "Unrecognized option" — read the error, not the exit code.

Related: [[rust-port-slice-aj-step5]], [[rust-port-slice-aj-step4]], [[rust-port-power-spelling]],
[[rust-port-status]].
