---
name: rust-port-inside-outside-exactness
description: "An operation applied INSIDE an accumulation and removed OUTSIDE is exact in algebra and not in arithmetic — and the source will call it 'exactly'; plus: a residual needs an ABSOLUTE bar, and an invented bar fails"
metadata: 
  node_type: memory
  type: project
  originSessionId: 1ca4b562-ce6b-45a6-9940-ddf2569a50df
  modified: 2026-08-12T12:32:50.145Z
---

Phase 3 slice D of the Rust port (rungs 22/23/24 — the resolved cross-plane, 2026-08-12) landed at
100% bit-equality again (462/462 vs PyPy), so for the fourth time the bit-count was not where the
value was.

**THE PATTERN WORTH CARRYING: an operation applied INSIDE an accumulation and removed OUTSIDE is
exact in algebra and not in floating point — and the source's own prose will say "EXACTLY".** Rung
24 does it twice. Its mean is a hierarchical `sum(sum(row) for row in field)` where the rung it
claims to reproduce runs one flat pass, so its "identical BY CONSTRUCTION, not to a tolerance"
reduce is the one that ISN'T exact — while the neighbouring rung, whose docstring only claims 1%,
is exact at every point measured. And it forms `Σ(scale · shape)` then divides by `scale`, so the
"scale cancels EXACTLY" factorisation moves in the 14th digit. **When a comment says "exactly",
ask whether the code applies and removes the operation at the same level of the loop.**

Two consequences for a port specifically:
- **The inexactness must be REPRODUCED, and gated from both sides.** Flattening the hierarchical
  sum would be *more accurate than the source* and is therefore a defect. Both gates assert the
  non-exactness, so a future tidy-up fails loudly. See [[rust-port-power-spelling]] for the phase
  where a tolerance let the opposite kind of defect ride.
- **Port the summation shape PER LINE.** Rung 24's mean is hierarchical and its mean-square, two
  lines later, is flat. A blanket rule either way injects a defect on one of them. The advisor
  caught this before it was written, from the two lines alone.

**A RESIDUAL NEEDS AN ABSOLUTE BAR.** The oracle reported a worst *relative* disagreement of 1.60 —
on a key that is a difference of two nearly-equal numbers, where the operands' last bits are the
whole answer. Relative error is meaningless there. Same lesson as [[golden-gate-slice5]], arriving
from the other direction.

**AN INVENTED BAR FAILED INSIDE THE HOUR, TWICE** — after [[rust-port-location-keys-refute]] had
already recorded that exact trap in slice B. Reading the measurement is apparently a habit that has
to be re-formed each slice. The instructive one: a kill-test peak cleared its nearest neighbour by
only 1.4%, and the fix was to move it onto a **4× coarser grid** (clearances 19.5% and 47.8%), not
to loosen the bar. That is [[rust-port-ported-test-vacuity]]'s coarse-grid rule applied a second
time — first to a hump, now to a location key that is itself a circularity kill test.

**My own first gate over-claimed and my own wider sweep caught it.** It swept 3×3, found the
inexactness 9/9, and stated "never bit-equal" as a law; adding a coarser grid and two extreme
points found two cases where the two summation orders round together. Fifth consecutive slice where
sweeping past the first gate written changed what could be CLAIMED, not just what was covered.

Also: the third and fourth **vacuity** cases (a `TypeError` test that in Rust is a compile error;
the twin of rung 16's helper-vs-production test). The pattern now has a name — *the source's test
guards what the target's type system or factorisation already guarantees* — and the replacements
are always the same move: assert the DERIVATION, or an identity the source cannot state.

Related: [[rust-port-decided]], [[rust-port-shape-keys]], [[rust-port-arithmetic-is-pypy]].
