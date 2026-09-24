---
name: rust-port-slice-ah-step6
description: "Slice AH step 6 — the two ported suites (rung77.rs 17 gates, rung78.rs 12+2); a ported isinstance compared against the very table the defect rewrites, so it was vacuous under the defect it exists for"
metadata:
  node_type: memory
  type: project
  originSessionId: d01f11e2-687a-46bb-bba2-0a810fc07ad5
  modified: 2026-09-24T06:05:25.815Z
---

Slice AH step 6, 2026-09-24: `rust/tests/rung77.rs` (17 gates, 1:1) and `rust/tests/rung78.rs`
(12 ported + 2 declared refusal gates, P5). No module line changed. Plan § 5.32.6.

**The lesson: an identity check whose REFERENCE is the thing the defect rewrites is vacuous
under exactly that defect — port the NEGATIVE half too.** Python's `isinstance(rig,
StiffnessLedgerTransient)` was first ported as `fn_addr_eq(rig.at_lever, R77.at_lever)`. The defect
the gate exists for is `R77.at_lever` re-aimed at rung 76's body — which makes `R77.at_lever`
rung 76's, so the equality passes. `isinstance` also REJECTS the parent; the fix asserts
`!fn_addr_eq(…, R76.at_lever)`. Injection I3 then failed ONLY those two negative halves; all 15
value gates stayed green (P2's shape, early).

**Why:** "compare against the expected table" silently assumes the expected table is fixed; in a
port where the class IS the hooks table, the expected value and the defect live in one place.
**How to apply:** for every identity/equality gate, ask "does the defect I'm guarding against also
move the right-hand side?" — if yes, anchor against something it cannot move (the parent, a
sibling) and reason through the injection BEFORE running it.

Also measured (the physics finding, in the plan not here): rung 78's two refusals surface through
one march entry in opposite ways — `sensed × gauge` fires at the initial solve and RAISES; the
anchored-root one fires inside the loop's `try/except: break` and becomes a SILENT SHORT
trajectory (27/341 at 1.1/c0), identical in PyPy and Rust. Counters are process-global, so the
three gauged gates in `rung78.rs` share a `Mutex`. See [[rust-port-slice-ah-step5]],
[[rust-port-ported-test-vacuity]], [[instrument-fed-by-what-it-certifies]].
