---
name: rust-port-slice-n-step5
description: "Slice N step 5 (the rung 55/56 suites) — two of the SOURCE's own gates could not see the defect they name, and a suite reaching inside a constant forced a fifth gated-code edit"
metadata: 
  node_type: memory
  type: project
  originSessionId: 6ecc4aa5-64d2-4910-9c3f-678688d41b4b
  modified: 2026-08-17T14:23:51.933Z
---

Slice N step 5 shipped the two suites: `rung55.rs` (20 gates) + `rung56.rs` (23) from Python's
18 + 21. Crate 556 → **599 run, 0 ignored**, verified as a `--list` **name diff** (43 additions,
0 removals), not a count — see [[rust-port-slice-n-step4]] for why a count is not the instrument.

**THE LESSON — A GATE THAT PASSES FIRST TRY IS THE ONE TO INJECT A DEFECT INTO, AND TWICE THE
DEFECT IT COULD NOT SEE WAS IN THE SOURCE'S OWN GATE.** Both suites passed on the first run.
Four detectors were injected into `stage.rs`; two gates did not fire:

- `test_reduce_stack_object_dispatches_at_K1` claims (in its docstring) that a value equality
  proves *"the same code and not merely the same algebra"*. Deleting the `K = 1` dispatch left it
  passing **bit-for-bit** — both paths bisect the same bracket to the same tolerance on
  sign-equivalent residuals, and a bisection reads only SIGNS. Fixed with a structural clause off
  the module's census (0 passes / 0 marches when it dispatches, 144 when it does not).
- `test_p6_verdicts_survive_the_work_split` asserts only UPPER bounds, all satisfied at `x == y`,
  so collapsing the two work splits onto each other left it green. A *"nothing rides on this
  knob"* gate is vacuous unless something else says the knob is LIVE. Two `assert_ne!` clauses.
- A third: Python's `cost == dict(sorted(cost.items()))` is `True` for ANY curve (dict equality
  ignores order), so the `or` short-circuits and the monotonicity it appears to gate is never
  evaluated. Measured that the live half holds, then asserted it alone.

**AND THE SECOND LESSON — PORTING THE CODE DOES NOT BOUND THE EDITS THE TESTS FORCE.** The step
table called step 1 *"ALL changes to already-gated code"*; step 3 refuted it once
([[rust-port-slice-n-step3]]), and step 5 again from a direction neither looked: a shipped test
sets `m._V_SCAN = 0.01`, overriding a class attribute per instance, which a Rust associated const
cannot express. `StageStackCore` gained a `v_scan` field and a builder. `grep '\._[A-Z_]* *='`
over the four suites finds every such reach in seconds — run it at PRE-FLIGHT, not at step 5.

**AND THE OBVIOUS GATE FOR THAT EDIT WAS VACUOUS TOO.** The row-count experiment that needs the
override passes unchanged at the default step: the two scan steps move `v*` at ~1e-11, which is
orders below every bar. So the edit is justified by FAITHFULNESS (run the source's experiment,
not a neighbouring one that agrees), it gets its OWN gate asserting the field is live, and the
source's stated reason for the finer scan — *a coarse scan "could have been a bracket artifact"* —
is REFUTED rather than merely unconfirmed. Third instance of this slice's
*a dead thing's spelling still has to be right* ([[rust-port-slice-n-step3]]).

**STEP 6 (docs-only) WAS SCOPED AT THE WRONG DEFECT, WHICH CLOSES THE SLICE.** It was
pre-registered as *"§ (iii)'s dead constants, if the specs assert otherwise"*; grepped, neither
rung 55's nor rung 56's spec mentions a single one of those constants, so that correction does
not exist. What it actually shipped is step 5's four findings written into the two specs. **A step
named after a PREDICTED defect finds the predicted defect or nothing** — the useful corrections
came from the step before it. The Python suites are left untouched: repairing the oracle's own
tests is outside the port (§ 8), so the findings are recorded with their measurements instead.

Related: [[rust-port-ported-test-vacuity]], [[rust-port-documented-gate-that-doesnt-exist]],
[[rust-port-copy-vs-rederivation]] (the one detector that DID fire cleanly: re-deriving the `K=1`
throat row instead of copying it diverges by 1 ULP).
