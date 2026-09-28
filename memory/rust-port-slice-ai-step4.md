---
name: rust-port-slice-ai-step4
description: "Slice AI step 4 (rung 80 whole: AirScope, split_march, split_row, four readers) — the instrument that caught the previous steps' defects (rung 79's counters) reads ZERO on every rung-80 reader; measure that an inherited catcher MOVES before relying on it"
metadata:
  node_type: memory
  type: project
  originSessionId: 8214f53a-bae3-492b-b0d4-14afd511a423
  modified: 2026-09-28T17:25:06.434Z
---

Slice AI step 4 shipped 2026-09-28 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.33.4): `AirScope`, `split_march`, `split_row`, `split_liveness`, `split_arrest`,
`split_saturation`, `split_wall::split_gains` in `rust/src/split_wall.rs`, gated by
`rust/tests/slice_ai_split.rs` (7 gates; 38 rows + 21 cells + aggregates as word vectors spliced from
the Python probe by `W:\temp\claude\slice-ai-step4\gen.py`). Green on the first run.

**THE LESSON: AN INHERITED CATCHER CAN BE STRUCTURALLY BLIND AT THE NEXT STEP — MEASURE THAT IT
MOVES.** Steps 2 and 3 each had defects caught by rung 79's six counters ALONE. The step-4 probe
snapshotted them around all five rung-80 readings: `[0,0,0,0,0,0]` every time (rung 80 marches in
`phi` reference, so the incidence branch never runs). Pinning them anyway would have been a gate that
agrees with itself. The replacement instrument was the readback of `(lag_coord, phi_ref)` on each
built rig plus the caller's `sm_air` after each reader — and K2 (`AirScope` not restoring) was killed
by that readback ALONE, every value passing.

**Why:** a catcher is only as live as the branch it counts; a new reader can route around the branch.

**How to apply:** in the pre-port probe, snapshot the previous step's catching instrument around the
NEW readers first; if it reads zero, say so in the test's header and find what does move.

Also: 16 injections, 16/16 verdicts, 2 MECHANISMS wrong. K1 (coordinate written through the cell)
moved `demand` VALUES by 1 ulp where I predicted counters only — I forgot step 1's own measured
slack-state 1-ulp difference ([[rust-port-slice-ai-step1]]). K13 (`ref="applied"`) died on a REFUSAL
(rung 74's joint IC), not on values. Unreached branches recorded: `monotone=False`,
`impossible=True`, non-empty `owner`, the `switch` skip. `0.76` round-trips to
`0.7599999999999999` as a built wall. Related: [[rust-port-slice-ai-step3]].
