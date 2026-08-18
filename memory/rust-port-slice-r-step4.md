---
name: rust-port-slice-r-step4
description: "Slice R step 4 (the rung 40/44 oracle) — a value probe that feeds the same wrong input to both sides cannot see a wrong input, and the IOU the plan's own step entry had dropped"
metadata: 
  node_type: memory
  type: project
  originSessionId: abfc17b0-e6fd-451d-8736-8bf295625b89
  modified: 2026-08-18T15:36:24.470Z
---

Slice R step 4 (2026-08-18) shipped `oracle/dump_two_spool_transient.py` +
`tests/two_spool_transient_oracle.rs` — 6 853 + 1 120 keys bit-exact against PyPy on the first
run, plus a tiered CPython arm. Two process lessons, both about instruments rather than physics.

**A value probe that supplies the input to BOTH sides is blind to a wrong input.** `rung44.rs`
had shipped at step 3 with its CPG gas constant copied from the file beside it —
`test_rung40.py` hard-codes `R_c = 286.9`, `test_rung44.py` derives `286.857…` — so every gate
in the ported suite ran the wrong gas. All nine gates are signs, orderings and spreads, so none
could see it; and step 3's own value probe, written precisely to check "the port agrees on
values, not only on the signs its gates assert", hard-coded `286.9` on the Python side too. The
probe compared the port against itself on that axis. It surfaced only when step 4 enumerated
**each suite's own grid** for the oracle instead of reading a constant off its neighbour.
**Why:** a probe's inputs are part of what it tests; sharing them with the subject removes the
axis from the measurement. **How to apply:** when writing a value probe for a ported test file,
take every constant from the SOURCE suite by reading it there, never from the ported file or its
neighbour — and dump the constants themselves as keys, so a mismatch fails at the constant
rather than propagating.

**A registered IOU rides one more step unless something forces it.** Step 1's postscript had
said `best`'s strict `<` was invisible to all 1 174 smoke values and that *"step 4's larger
reacting grid is where it could be"*. My step-4 plan did not mention it; the advisor's
pre-work review did. Measured: 0 ties over 21 CPG cells, **22 over the 12 reacting ones**, all
in cells taking the noise exit — the only exit that reads `best` — and injecting `<=` moves 9 of
1 120 keys. So step 1's *"0 keys moved → INVISIBLE"* row was **corrected**, not confirmed.
**How to apply:** when a step note defers a measurement to a named later step, the later step's
plan entry must name it back, or grep the previous step's notes for "registered", "deferred" and
"step N" before writing the step.

Two smaller things worth keeping: a counter over MISSES cannot measure an equivalence relation
against a coarser one (needed a second counter over every lookup, or the collision count is 0 by
construction); and an interpreter arm's tolerance must split by MECHANISM — direct value, residual
near zero, finite-difference derivative — because one relative bar loose enough for a residual
admits everything. See [[rust-port-slice-r-step1]], [[rust-port-slice-r-step3]],
[[golden-gate-slice5]].
