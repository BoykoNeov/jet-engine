---
name: rust-port-slice-ae-step5
description: "Slice AE step 5 — an install proof that passed by comparing the machine against the pointer my own fixture had just handed the builder, and two assertions typed from the sentence I wanted while the counterexample sat in my own measurement file"
metadata: 
  node_type: memory
  type: project
  originSessionId: ac75ba91-78e4-49d0-979a-55d0d922efb9
  modified: 2026-09-03T14:38:36.428Z
---

Slice AE step 5 (rung 73, the dispatch gates — `rust/tests/slice_ae_dispatch.rs`, 1 144 lines,
10 gates). Plan § 5.29.5. Closes slice AE. See [[rust-port-status]], [[rust-port-slice-ae-step4]].

**THE LESSON: a fixture is a claim, and it is the one place nobody re-reads as one.** Two of this
step's three defects lived in the fixture rather than in a gate.

* **`ptr::eq` on a `const` table is not a table-identity test.** Every hook table in this family
  is a `pub const`, so `&R72_TRIPLE` is a fresh rvalue promotion at each use site. It failed on
  the one row where the crate's own builder supplies the pointer — and **where it passed, it
  passed because the machine held the pointer my fixture had handed the builder three lines
  earlier.** That is the shipped-instrument-agrees-with-itself pattern inside an *install proof*,
  whose whole job is to be independent of what it certifies. Replaced by fourteen `fn_addr_eq`s
  under an exhaustive destructuring (function addresses are stable across promotions, and the
  destructuring is an `E0027` tripwire when a field lands). Measured 2 passes / 1 failure /
  4 unreached, written that way rather than as a tidier six.
* **My counterfeit `at_lever` re-implements the law carry**, because it must — a sibling built
  under a scoped law would otherwise read the class default. So the file is structurally blind to
  a mutation of the shipped carry: the sweep scores it **0 of 10** where `slice_ae_cells.rs`
  catches it 1 of 15. Disclosed in the header rather than discovered later, together with the
  matrix's second blindness: it is baseline-relative, so a source change that moves the baseline
  and all seven rows together is invisible by construction.

**Why to apply it:** review reads gates as claims and fixtures as plumbing. A fixture that
supplies the value under test, or re-implements the statement a mutation deletes, produces green
that means nothing — and it does it silently, in a file whose entire purpose is coverage.

**How to apply it:** ask of every fixture line *could the gate above pass because of this line
rather than because of the code?* And for a coverage file specifically, score the mutation sweep
on **every** binary in the slice, not just the new one: that is what turned both blind spots from
suspicions into numbers.

## Two assertions typed from the narrative, with the counterexample in my own measurement file

Every number was measured first, into `step5_measurements.md`. Two gates were then written from
the sentence I wanted rather than from that file, and both failed on the first run:

* *"the largest move is a CROSS term"* — the table says `r_f` on one arm and **`r_r`** on the
  other. The self and cross indicators are `+1` and `-1`, equal in magnitude to 1e-15, so *the
  largest* is a tie broken by the last bits and is **not a property a gate may pin at all**.
* *"every injection is live at some seat"* — AD step 6 could assert that; here it is false for two
  of seven, and both rows were already all-silent in the table.

**A measurement file is only an instrument if the gate is transcribed FROM it.** The tell is a
gate whose expected value is stated in words that never appear in the table. Both were *checked* —
against the sentence.

## A silent panic hook is process-wide and ate both failure messages

The crate's `pytest.raises` idiom (take-hook / empty hook / catch / restore) is fine when one gate
expects a panic. Six of ten here do, `#[test]`s run concurrently, and on the first run **both
genuinely failing gates printed `FAILED` with no message, file or line.** Repaired with one hook
installed behind a `Once` consulting a thread-local. A test file that cannot show why it failed is
not a gate.

## What the step measured, kept because each was measured

* **P1's manufactured pairing, finally driven.** § 5.29.3 (d) falsified P1's conclusion and left
  its reason — *no shipped rung-73 test calls a rung-69 reader* — measured by neither arm.
  Shipped, it refuses. **Delete the refusal and the reader returns the same number twice**: a
  ledger whose whole content is a comparison between two references lands on identical floats,
  because neither arm ever set one. `rung73.rs` gate 9's finding, arriving in the plumbing.
* **The port Python refuses is the one that works — AT THAT SEAT ONLY.** With `with_ref` pointing
  at rung 69's body, `reference_bill` returns a reading **byte-identical** to a real rung-69
  machine's (all 6 033 chars of the `Debug` fingerprint). **But my draft's next sentence — that
  such a port would ship green — was falsified by my own sweep: 24 of 59 gates fire**, because
  rung 73's five readers reach `ref_law` through the same cell. Reaching from *the pairing cannot
  tell them apart* to *nothing can* is [[rust-port-slice-w-step3]]'s rule applied to a sentence.
  Also: that injection is **not** P1 option (1), which adds a second field; it is option (1)
  minus the replacement setter, and the comment now says so.
* **A third of P4's value break is a SIGN BIT.** `pair_fr = f_r * r_f`, and reading B puts a `-1`
  against an exact `0`, so it is `-0.0` at every point where rung 72 gives `+0.0`. `==` scores
  140/62 against `to_bits`'s 210/93. **This is the mechanism under two numbers already recorded
  twice** (step 2's M22 and step 4's P11 both measured that all 101 `-0.0` keys in the oracle are
  `pair_FR`, and neither said why).
* **The port's discrete set is narrower than Python's, 3 against 5** — `f_f`/`r_r` are `f64`
  fields the port writes as `0.0`, so two of Python's absent keys become value moves here. Gated
  from both sides.
* **The matrix rediscovers the source's call-site census**: `quad_gains_at` is loud at
  `applied_gains`, moves `applied_cells` and `ref_discriminator`, invisible at `handover_law`,
  `applied_bill` and a bare march — three readers dispatch it, two do not, measured rather than
  grepped.
* **j5 was pre-registered as the row only this file catches, and it is** — 3/10 against 0/15,
  0/27, 0/7. `shared_actuator.rs`'s doc comment claims collapsing `None` to `Some(0.0)` deletes
  the discrete witness; until this step nothing in the crate could test that sentence.
