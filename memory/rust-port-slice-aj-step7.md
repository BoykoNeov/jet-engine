---
name: rust-port-slice-aj-step7
description: "Slice AJ step 7 (rig dispatch + closing deletion; SLICE AJ CLOSED) — pointing a cell back at its PARENT is a reduce test, not a dispatch test, wherever the fixture sits on the rung's reduce arm; carry a dispatch verdict on a two-table COUNTER and a DISTORTION instead"
metadata:
  node_type: memory
  type: project
  originSessionId: b13fd9c4-76e5-4a4f-89ce-6e10871f7be2
  modified: 2026-10-01T11:05:35.028Z
---

Slice AJ step 7 COMPLETE 2026-10-01 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.34.7) — **SLICE AJ CLOSED**, the ladder ported through rung 84. `rust/tests/slice_aj_dispatch.rs`,
7 gates, green first run. Predictions in `W:\temp\claude\slice-aj-step7\predictions.md`.

**THE LESSON: a parent-pointer injection is VACUOUS wherever the fixture sits on the rung's reduce
arm.** Every earlier dispatch file pointed a swapped cell back at the parent it was re-aimed from.
Here the cell (`quad_gains_at`, rung 73's) reduces EXACTLY to rung 72's at the suites'
`ref_law = "sched"` — measured: the `RigR72` row reads `same`. Had it been the verdict row, P4
("the rig's table is read") would have looked falsified for a reason unrelated to dispatch. The
advisor flagged it before any code; I ran it as an informative row and carried P4 on:
- a **counter on each table in ONE run** (caller 0, rig 80 = Σ `n_sampled` exactly — which also
  proves the march never dispatches the cell), and
- a **distortion** (`f_q × 2`) on one table at a time: rig → DIFF, caller → same.

**Why:** the reduce contract (every rung reduces to its predecessor) is the project's spine, so
the parent's function is BY DESIGN indistinguishable at the default knobs.

**How to apply:** before using "point it back at the parent" as an injection, check whether the
fixture's knob settings are the rung's reduce arm; if so, use a counter + a distortion, and keep
the parent swap as a recorded, non-scoring row.

Also:
- **P5 confirmed:** deleting `at_lever: r80_at_lever,` in Rust now fails `rung80.rs::the_knob_is_loud`
  at step 1's pointer assert (AI step 7 measured 16/16 there), every AJ value/oracle token
  unchanged, and the new file's entry control (`the_shipped_rig_is_a_rung_80_machine`, by
  `shared_rig`) fails beside the oracle's zero — so that zero is *ran, no difference*, shown in-binary.
- Every row asserts its install (caller AND the rig `split_march` returns) before its reading counts.
- P1 2 750 (band floor 2 700), P6 18 fields, 0 `R81`…`R84`. One item booked forward: C6 (the
  `residual_shape` ladder mutant that survives every gate).
- I wrote C6 backwards in the first draft of § 5.34.7 (said Rust spells `lo + i*step`); the Rust
  is Python's spelling and `lo + i*step` is the surviving MUTANT. Read the line before restating a
  prior step's finding.
