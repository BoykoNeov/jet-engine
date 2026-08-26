---
name: rust-port-slice-x-step2
description: A coarse test grid flipped a published boolean, so emit BOTH grids rather than picking the agreeable one
metadata:
  type: project
---

Slice X step 2's smoke ran at a deliberately coarse step size. One of rung 64's own published claims —
that a composite march IS the valve-alone march — is TRUE at the fine step and FALSE at the coarse one:
the parabola-refined minimum moves 2.9e-04 (four orders above the reader's 1e-14 bar) while the other
half of the same claim still agrees to 1.1e-16. Shipping the coarse reading would have published a
bit-exact golden that reads as a refutation of the rung.

**Why:** picking the grid where the claim holds is the same act as hiding the one where it does not.
The flip is a real property of the reader (a refined extremum on a coarse march), so it is content.

**How to apply:** when a coarse grid disagrees with the suite's on a BOOLEAN the rung publishes, run
that section at the finer grid AND emit the coarse reading beside it with the mechanism named, so the
flip is gated. Also: Python's `_is_armed` was ported under a DIFFERENT name than the port's `is_armed`
(schedules-only vs the composite guard) — the two agree on every machine lacking a constant setting, so
exactly one key of 318 could separate them. See [[rust-port-slice-w-step4]] on renamed predicates.
