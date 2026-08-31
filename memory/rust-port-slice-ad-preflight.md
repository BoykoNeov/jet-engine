---
name: rust-port-slice-ad-preflight
description: "A shipped Usage: block documents a method with zero definitions, and the quartic's three risky roots never win the max on any input the suite reaches"
metadata: 
  node_type: memory
  type: project
  originSessionId: f0d438fb-b4ba-467d-b0e5-f1af8995bcd2
  modified: 2026-08-31T19:35:02.232Z
---

Slice AD pre-flight (rung 72, `SharedActuatorTransient`, Rust port) — twelve probes, plan § 5.28.
The cell column measures **3**, the first back-half row where the hand-written number is right.

**THE LESSON: a count of things a probe never reached is not a zero, and neither is a plausible
non-zero.** Two instruments in this pre-flight printed a confident number from a run that had
stopped early or never started. Probe F's first run said `calls: 0` for the quartic solver, which
reads as *the solver is cold*; the cause was `-n auto` — **xdist workers are other processes and
an in-process spy sees none of them**. Repaired with `-n0` and, more importantly, an
`assert stats['calls'] > 0` so the instrument proves it can see. The second is worse and is in
[[rust-port-slice-ad-step1]].

**THE FOUR FINDINGS, all vacuity of one kind or another:**
- **`shared_modes` does not exist.** Rung 72's class docstring documents `t.shared_modes(...)`;
  0 defs, 0 instance assignments, 0 locals. Swept over all 58 ladder classes: **three** phantoms —
  `restored_plant` (65), `cascade_modes` (66), `shared_modes` (72) — each a renamed reader whose
  docstring kept the old name. The Rust port carries none.
- **The quartic's three risky roots are dead.** `scale = max(1.0, |a3|, |a2|**0.5, |a1|**(1/3.),
  |a0|**0.25)`, and `|a3|` wins on **1068 of 1068** calls. So a port defect in the cube root would
  be invisible to every gate — disclose it, never gate it. 375 DISTINCT coefficient vectors is the
  real size of the claim; the 500-iteration cap and the `den==0` guard never fire.
- **`_authority`'s 1e-12 tolerance never does any work**: 36 exact zeros, **0 calls in the open
  interval**, so `gf == gr` is bit-identical on every shipped input.
- **The floor's shipped needle discriminates nothing**: `match=r"FOUR actuator states"` is in
  rungs 72, 73 AND 74's messages, whose conditions are identical — the Python suite passes with
  either successor's floor installed. Rung 69's analogue (`"rank TWO"`) does discriminate.

**And probe I's second sweep was 19-for-19 FALSE** — 4 were methods on another class, `_b_state`
and `_b_forced` are instance attributes with 75 and 3 assignments. Probe J classified all 19
*before* one was written down, which is the only reason it cost nothing.

**How to apply:** before reporting a count, ask what the instrument could not have reached — a
worker process, an unbuilt target, a name that is an attribute rather than a method. And when a
threshold, a tolerance or a branch is about to be gated, first measure whether any shipped input
lands in it; a gate on an unreachable branch passes forever and says nothing.
