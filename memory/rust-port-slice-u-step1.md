---
name: rust-port-slice-u-step1
description: "Slice U step 1 — 575 keys bit-exact and 17/17 green, then eleven injections showed five of the record's 25 keys have no gate at all"
metadata: 
  node_type: memory
  type: project
  originSessionId: 7b5c5e71-5661-4720-a3e3-b0f54519b537
  modified: 2026-08-20T09:57:05.283Z
---

Slice U step 1 (rung 49's `φ` floor: `surge_relief` + `floor_sweep` + 17 gates) shipped
2026-08-20. Three process lessons.

**BIT-EXACTNESS AND GATE POWER ARE DIFFERENT QUESTIONS, AND ONLY INJECTION ANSWERS THE SECOND.**
The port came back 575 keys bit-exact against PyPy over all 23 cells the gates touch, on the first
run, and 17/17 green. Both are real and neither says which gate would notice a defect. Twelve
deliberate defects written into the shipped reader: one caught by 8 gates, three by exactly ONE
gate each, and **five live ones caught by NONE**.
[[rust-port-oracle-cannot-see-a-missing-gate]]'s companion: an oracle cannot see a missing gate,
and a green suite cannot see a missing assertion.

**AN INJECTION TABLE NEEDS A "DID IT MOVE A VALUE?" COLUMN, OR IT OVER-COUNTS.** The first writing
had seven "no gate caught it" rows; measuring which of them actually changed a number cut it to
five. Two were inert on the whole grid — breaking them moves nothing, which is a statement about
the grid, not about the gates — and one moved only cells the reading gate does not sweep, so the
honest claim was narrower than "a live defect passed 17 gates". One row was also asserted for a
field the injection never touched, and the follow-up injection was what turned it from inference
into measurement.

**A KNIFE-EDGE VALUE CAN BE PROTECTED REDUNDANTLY, SO A ONE-FAULT TEST CLEARS IT.** The published
boolean sits at one ulp on one cell. Cleaning the march coordinate to `k*ds` does NOT flip it (it
lands exactly on the bar and the strict `<` absorbs it); loosening `<` to `<=` does not flip it
either (the accumulated coordinate overshoots the bar). **Both together flip it.** So the
pre-registered "a flip means the coordinate was cleaned up" detector was wrong about the mechanism,
and the right reading is that two spellings each independently keep the value — which no single
injection would have shown.

Also: [[rust-port-slice-t-step2]]'s "tabulate BAR MARGINS" paid immediately — four of ten bars sit
inside 1.5×, against slice T's 5–7 orders on all four of its own. And the injection script's
site-count guard caught itself applying nothing, which is [[rust-port-slice-t-step4]]'s lesson
landing on the script written after it.

See `docs/plans/todo-rust-port.md` § 5.18 step 1. Related: [[rust-port-slice-u-preflight]],
[[rust-port-copy-vs-rederivation]].
