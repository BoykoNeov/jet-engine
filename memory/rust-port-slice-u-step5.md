---
name: rust-port-slice-u-step5
description: "Slice U step 5 — the step written to close the slice produced TWO near-vacuous gates of its own: one comparing my formula with my formula, one that could not fail on a number"
metadata: 
  node_type: memory
  type: project
  originSessionId: 3688e397-f7fc-4957-8677-b3e66aa2c742
  modified: 2026-08-20T11:37:49.890Z
---

Slice U step 5 (the oracle for rungs 49–52's nine readers) shipped 2026-08-20, completing slice U
and **phase 6 of the Rust port**. 4 179 keys over eight sections, bit-exact against PyPy on the
first run and against CPython too. Its contents were DERIVED from the four injection batteries —
each section answers a named row — rather than chosen. Two process lessons, both about the step's
OWN instruments.

**THE GATE WRITTEN TO CLOSE A SLICE IS NOT EXEMPT FROM THE SLICE'S OWN LESSONS.** Two near-vacuous
gates in this one step:

1. The manufactured trajectories for two rules unreachable from any marched cell were first
   written against a **re-spelled copy of the loop inside the test file** — my formula against my
   formula, which is [[rung70-generic-split]]'s lesson arriving on the closing gate. The fix is
   the one the neighbouring function's own doc comment already records for the identical case:
   **lift the loop into a callable** so the gates hold the shipped code. Two checks make that
   real — the oracle's 4 179 keys re-ran bit-exact across the extraction (which is what makes
   "behaviour-neutral" a measurement), and an extra gate asserts on a marched cell that the reader
   and the manufactured cells go through the same function, so a later edit cannot orphan it.
2. The CPython arm routed **every** disagreement into a printout and panicked only on key
   presence, so it **could not fail on any number** — it gated coverage, not values. The
   precedent in the same port is that CPG keys stay bit-exact on the CPython arm; every cell here
   is CPG, so there is no tolerance tier and a drift is a defect. *A documented gate that doesn't
   exist*, self-inflicted.

**A DETECTOR REPORTING ZERO HAS DEMONSTRATED NO SENSITIVITY, AND SAYING SO IS PART OF THE
RESULT.** The CPython arm came back 0 drifts / 0 flips. That confirms the CPG prediction; it does
not establish that the arm would catch anything. The earlier slice could quote a measured
sensitivity because its arm had sections that genuinely moved. Report the confirmation, not a
discrimination.

Also: an IOU written at step 1 ("only step 5's oracle bits can hold this") was checked at step 5
by grepping the golden for the exact cell and bit pattern, not assumed discharged. **A promise
made four steps earlier needs one sentence of verification, not one of memory.**

See `docs/plans/todo-rust-port.md` § 5.18 step 5. Related: [[rust-port-slice-u-step4]],
[[rust-port-oracle-cannot-see-a-missing-gate]], [[rust-port-documented-gate-that-doesnt-exist]].
