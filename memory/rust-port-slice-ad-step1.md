---
name: rust-port-slice-ad-step1
description: "A pre-registered count of compile errors was measured on a build that stopped at the lib, and it came back plausible and wrong"
metadata: 
  node_type: memory
  type: project
  originSessionId: f0d438fb-b4ba-467d-b0e5-f1af8995bcd2
  modified: 2026-08-31T19:35:23.097Z
---

Slice AD step 1 (rung 72 Rust port) shipped `rust/src/shared_actuator.rs`, three new `TripleHooks`
cells with their refusals, two carriers, and `rust/tests/slice_ad_cells.rs` — **12 gates**, all
green first run, **9 of 9 mutations killed**. Plan § 5.28.1.

**THE LESSON: `cargo check --all-targets` stops when the LIB fails, so a count of compile errors
taken during a broken build is a number from an instrument that never ran.** P1 predicted **5**
`E0063` sites from widening `TripleHooks` — one per shipped const — and **0** from the four alias
tables. The mechanism held perfectly. **The count was 7**: two more exhaustive literals live in
`tests/slice_ab_cells.rs` and `tests/slice_ac_cells.rs`, the width tripwires themselves, and the
probe never compiled the test targets.

**The dangerous part is that the wrong number AGREED WITH THE PREDICTION.** A zero from an
instrument that never ran at least looks like nothing happened ([[rust-port-slice-ab-step3]]); a
plausible non-zero that matches what you expected does not prompt a second look. It was caught only
because applying the change for real produced two errors the probe had promised would not exist.
**A width prediction can only be measured as: apply, fix the lib, then count what is still red.**

**And the tripwires firing IS the mechanism working** — [[rust-port-slice-ac-step7]] shipped the
measurement that `TripleHooks` is the one table type in the crate whose copies go loud, and the
first slice to add a field was stopped by them in two files at the first compile.

**A DEFAULT THAT IS THE CORRECT ANSWER ONE RUNG UP.** `reference`'s tempting slot value for rungs
40–71 is `req` — which is rung 72's own body. It would agree with rung 72 on every input any suite
reaches, so no value key could see the slot was wrong. That is `NO_TRIPLE`'s stated reason in its
sharpest form: the trap is not a guess, it is the right answer at the wrong rung.

**Two mutations worth keeping in mind, both silent failures no float reveals:** arming BOTH stators
where the flag selects one (rung 71's constraint count on rung 70's arm), and a scope guard that
restores the DEFAULT instead of the PREVIOUS value — the two agree on every shipped path, so only a
manufactured NEST separates them.

**How to apply:** never read a count off a build that failed earlier in the pipeline than the thing
you are counting. And when adding a slot for a name that does not exist below this rung, ask what
the "obvious" default would agree with — if it agrees with the child, the slot must panic.
