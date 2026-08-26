---
name: rust-port-slice-w-step5
description: "Slice W step 5 — six mutations to prove five gates can fail, and the table showed that a 'did it move' assertion is satisfied by a HALF-APPLIED injection"
metadata: 
  node_type: memory
  type: project
  originSessionId: 75ae92c3-7944-4f18-bafd-48b3783bfa66
  modified: 2026-08-26T14:09:34.429Z
---

Slice W step 5 of the Rust port (rungs 62/63), 2026-08-26. `slice_w_dispatch.rs`: **5 gates, zero
source lines**, closing slice W. It builds the two instruments step 3 measured to be blind and
discharges P2. Gate: 107 result lines, 1 017 passed, 0 failed, 0 ignored — `1010 + 2 + 5`.

**A "DID IT MOVE" ASSERTION IS SATISFIED BY A HALF-APPLIED INJECTION.** Each of the file's own
injections was neutered in turn to prove the gate that exists to catch it goes red — six
mutations, six caught by exactly the intended gate. But the table measured something nobody asked
for. The `b_of` re-read is injected at TWO sites (`powers` and `tail`). Remove ONE and the gate
asserting the call count merely ROSE stays **green**; only removing both fires it. The other gate
asserts it rose by EXACTLY `powers_total + tail_total`, and catches either alone. **How to apply:
in an injection harness, assert the EXACT expected delta, never `>`** — a `>` reports a
half-applied injection as applied, which is [[rust-port-slice-t-step4]] (*an injection matching
TWICE applies nothing and still reports green*) one notch over: applying ONCE where it should
apply twice. And the mutation table is what surfaced it; neither gate's own green run could.

**A MANUFACTURED-BUG GATE NEEDS ZERO SOURCE LINES, AND THE SEAMS ARE USUALLY ALREADY THERE.**
Every injected table was built from already-`pub` items. The sharpest instance: the override under
test was shipped as `{ at_stator: <override>, ..R57 }`, so **the machine with the override removed
is the machine built on `R57` itself** — no new item, no visibility widening. **How to apply:
before writing an injection, check whether the shipped table's own SPREAD already names the
un-overridden version.**

**A PORT'S REFUSAL CAN BE A STRONGER WITNESS THAN THE VALUE THE PLAN PREDICTED.** The plan
recorded the un-overridden sibling as *"no such method"* (Python's `AttributeError`). The Rust
reaches the same place through a table whose cell PANICS rather than answering `false` — the crate
declines to make a claim no value gate could see. Asserting the REFUSAL, with the message matched
via a helper that tries **both** panic payload types (`String` from an interpolated `assert!`,
`&'static str` from a literal one — step 3's finding 1), is sharper than asserting a boolean.

**AND A SHIPPED GATE'S CLAIM WAS CORRECTED BY THE STEP THAT REPLACED IT.** Step 2's smoke section
said it caught a wrong table spread. It cannot: the input kind it builds cannot separate the two
bodies at all, AND its leg removes **exactly 0** fuel on that grid — two inert paths agreeing. The
section is left as it is (its keys are real readings, its golden bit-exact) and its CLAIM is
corrected in place, pointing at the gate that now does the job.
[[rust-port-documented-gate-that-doesnt-exist]]. **How to apply: when a gate's set point is a
threshold, assert the leg actually BOUND before comparing anything across it.**

See [[rust-port-slice-w-step4]], [[rust-port-slice-w-step3]], [[rust-port-slice-w]],
[[rust-port-decided]].
