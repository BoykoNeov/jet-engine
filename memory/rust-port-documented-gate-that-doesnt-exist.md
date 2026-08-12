---
name: rust-port-documented-gate-that-doesnt-exist
description: "A key class can be documented in two places, computed, and emitted by nobody — the count guard cannot catch it because neither side emits it"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 519adbe9-fbbb-49f5-9712-29d4dd7fed36
  modified: 2026-08-12T17:30:36.416Z
---

Slice F's oracle header described an `iters/…/min|max` discrete-key class in two passages, and
`March` computed both fields on every step with a paragraph justifying them. **Nothing emitted
them and nothing read them.** The Python could not have emitted them without instrumenting the
source, so the class was documentation for a gate that did not exist.

**The oracle's own key-COUNT guard cannot catch this.** It asserts `rust.len() == oracle.len()`,
which is exactly the check that fails when one side forgets a key — but here *neither* side had
the class, so 912 == 912 and it passed. A guard against drift between two sides is blind to
something absent from both.

**Why:** the port's prose is unusually load-bearing (it is the deliverable, per the project's
"understanding, not the tool" contract), which makes a confident paragraph about a non-existent
gate more dangerous here than a missing comment would be. `porting_rules.rs` exists precisely
because a rule stated only in a comment drifts.

**How to apply:** when a doc block names a key class or a gate, grep that the name exists in the
emitting code AND in the reading code before shipping the block. When the class turns out dead,
prefer making it earn its place over deleting it — ask what it could gate that no dumped value
can. Here the answer was **convergence**: `used == 200` means the bisection never met its stopping
rule, and that is invisible in the result, because `0.5*(lo+hi)` off an unconverged bracket is a
plausible number and BOTH sides of a Python↔Rust dump would agree on it. So the counts moved from
the dump to a Rust-side band gate. Related: [[rust-port-ported-test-vacuity]],
[[rust-port-measure-before-registering]], [[golden-fingerprint-gate]].
