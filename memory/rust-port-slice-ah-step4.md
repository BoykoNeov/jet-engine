---
name: rust-port-slice-ah-step4
description: "Slice AH step 4 — eleven gates green on the first run could not see three of six injected defects, and the one that mattered was hidden by the reader's own exclusion"
metadata: 
  node_type: memory
  type: project
  originSessionId: f3c55711-1d71-43f4-9e46-664d7977acbe
  modified: 2026-09-08T11:51:13.950Z
---

Rung 78 §§ 1–3 ported (`gauge_scan`, `root_census`, `root_count`, `accel_cap_fn`,
`gauge_points`); `residual_gauge.rs` 532 → 1 165, `tests/slice_ah_gauge.rs` 526 lines / 12 gates.
Plan § 5.32.4.

**THE LESSON: a gate file green on the first run has told you nothing until you mutate the code
under it — and the defect worth finding is the one the SHIPPED READER's own filtering hides.**
Six plausible slips were injected one at a time. Three were invisible to all eleven gates. The
one that mattered: reading the residual's slope at the SOLVED root instead of at the anchor.
Python's comment says exactly why that is wrong ("inside the band that is the other one"), and it
is right — but the cells where the two readings differ are **precisely the cells the reader's
exclusion drops**, so no aggregate the suite computes can ever contain one. A documented reason
can be true and still be defended by nothing.

**AND THE FIRST GATE WRITTEN FOR IT DID NOT CATCH IT EITHER.** It demonstrated the physics by
calling the slope helper directly and never went through the shipped reader, so the injection
stayed green. Re-running the sweep after adding a gate is not optional; a gate is only proven by
the mutation it fails on. What finally closed it was a claim about the reader's OWN output —
`gw == 1 − k·c` at every cell, dropped ones included — which is a sharper statement of the
section than either the suite or the spec makes. Same shape as [[rust-port-slice-w-step5]] from
the other end: assert the exact delta, and mutate your own gates to find out.

**The two that stayed blind were recorded, not gated.** A three-argument `max` that never
disagrees with its first argument (100 of 100 cells) is a defence with no reader
([[rust-port-slice-aa-steps2345]]); `0.0 / c` is exactly `0.0`, so Python's falsy short-circuit
is a no-op. Inventing a gate for either would mean inventing a grid the rung does not run on.

**THE STEP'S OTHER FINDING: the slice's leading hazard had a second instance on a variable its
census was not scoped to see.** The pre-flight priced "one hazard, three treatments, one class"
on the freeze pair `_b_state`/`_v_state`. The sixth declared knob `_gauge_k` has the same shape
one section apart — one reader saves and restores `prev`, one clobbers to the literal identity,
one uses the declared helper the other two ignore — and the clobber is reachable-wrong INSIDE one
call, because `_shared_rig` propagates the caller's gauge onto the marched machine. Measured at
`k = 2.5`: row 1 runs at 2.5, rows 2–3 at 1.0. This is [[rust-port-slice-ah-step3]]'s lesson
promoted from one docstring to a class property. **A census scoped by a variable NAME cannot see
the same defect on a different variable** — so when a hazard is booked, ask what else has its
shape.

**And the first probe of it measured the wrong object**: the reader writes the knob on the MARCHED
machine, not on `self`, so reading `self` after the call showed nothing and the finding looked
like a non-event. The thing a method mutates is not always the thing you hold a handle on.

**The citation guard, run inside the step per [[rust-port-slice-ah-step2]], found a blindness of
its own**: sixteen references were written as a bare `` `:20404` `` where its pattern matches
`engine.py:20404` — citations no gate could check, ten of them shipped by step 1, in the file
whose own comment says *a guard that watches the right directory still needs someone to run it*.
The sibling rule: **a guard that matches one spelling is blind to the other.** All sixteen
expanded, 33/181/112 → 35/196/121.
