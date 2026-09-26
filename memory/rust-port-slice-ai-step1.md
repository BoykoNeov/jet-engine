---
name: rust-port-slice-ai-step1
description: "Slice AI step 1 (rungs 79/80 plumbing + rung 79's plant) — the first test state could not reach the code under test, because the valve held phi at exactly the wall the phi leg watches"
metadata:
  node_type: memory
  type: project
  originSessionId: 1a5af0c5-bbcb-40d5-beff-6afac99beb7e
  modified: 2026-09-26T18:11:09.710Z
---

Slice AI step 1 shipped 2026-09-26 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.33.1): `rust/src/state_coordinate.rs` + `rust/src/split_wall.rs`, carriers `phi_ref` / `sm_air`,
six re-aimed pointers, 0 new `TripleHooks` fields, `rust/tests/slice_ai_cells.rs` (10 gates).
Step boundary re-cut: rung 79's PLANT (`_phi_residual`, `_phi_cap`, `_cap_fuel`) moved into step 1
because a re-aimed pointer needs a real body; step 2 is now `_coord_at`/`coord_scan`/`coord_census`.

**THE LESSON: A TEST STATE CAN BE UNABLE TO REACH THE CODE UNDER TEST FOR A PHYSICAL REASON, NOT A
NUMERIC ONE.** The plant gates were first written at a borrowed state `(1, 1, 0.02)` on the valve rig
and three aborted inside RUNG 74's cap. A probe of `phi(w)` showed the valve — armed at the SAME 0.80
wall — holding `phi` at exactly `0.8000` wherever it would dip, so the phi leg's residual is pinned at
zero and never goes positive. That is rung 74's own finding (levers act only inside the fuel leg's
tracking error), surfacing as a gate that could never enter rung 79's branch. Fix: valve off, one
SLACK and one BINDING state, found by probing.

**AND "SAME ROOT TO 1e-9" CANNOT TELL A LIVE COORDINATE FROM A DEAD ONE.** The two coordinates differ
by 1 ulp at the slack state and by 0 at a neighbouring fuel value. Without an explicit
`bits differ at SLACK` assertion, injection I1 (incidence arm silently written as `Gs`) passed every
other gate.

**How to apply:** before reusing another file's test state, check it reaches the branch under test
(a counter or a probe, not a green result). When a gate's claim is "the knob changes the floats",
assert the bits DIFFER at a state measured to differ — a tolerance-equality gate is satisfied by the
knob being dead. Sweep 8 injections, 6 killed; the misprediction (I4, rung 80's rig `sm_air` line)
survived because `at_lever` already writes the same value — [[rust-port-slice-ag-step2]]'s N-site
lesson again, and not foldable, so pointer identity cannot see it either.

Also: item E had a second misattribution in the very next clause, and item H was three sites (plus
`_with_ref` in `applied_reference.rs`), not one — a census of doc claims lists sentences, and the
defect lives in clauses. See [[rust-port-slice-ai-preflight]] and [[rust-port-status]].
