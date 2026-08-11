---
name: rung83-corrector-law
description: "Rung 83 — a bracketing solve locates a SIGN CHANGE and a corrector needs a ROOT; one ramp has none, and two of my own numbers measured nothing"
metadata: 
  node_type: memory
  type: project
  originSessionId: 74698386-42d6-4ad6-9b65-4b3f62aa0ec2
  modified: 2026-08-11T18:35:09.897Z
---

Rung 83 (shipped 2026-08-11) closed rung 82 § 8's first seam — *does one Newton step off the
residual reach the 13-march bisection?* — by answering **NO**, for a reason the seam had not
imagined. Fourth reader-only rung (77, 81, 82, 83); `CorrectorLawTransient`, reduce is an identity.

**HEADLINE: a bracketing solve locates a SIGN CHANGE; a corrector needs a ROOT, and on a residual
built as a MINIMUM those are different objects.** At one of five ramps there is no root at all:
the residual steps `+1.65e−3 → −2.43e−3` across a τ step of `1.25e−5`, at an argmin handover. The
thirteen marches buy **an answer that exists** — not resolution. Extends [[rung78-residual-gauge]]:
78 found a residual's slope is a gauge and its root's *uniqueness* is not; its *existence* is not
either, and it flips between the two shipped `ds` values.

**Two of my own numbers measured nothing, and the advisor caught both.**

1. **An identity round-trip sold as a verification.** I quoted an error law as "verified to 2.7e−14
   across 35 rows". The mean slope in it is *defined* so the law follows in two lines for any
   function — it cannot fail, and the agreement was floating-point closure on my own definition.
   Third occurrence of this shape ([[rung70-generic-split]]'s gate computing my formula twice,
   [[rung77-stiffness-ledger]]'s perfect 1.000e+00). **Kept as a derivation, never gated** — the
   test would pass on a plant that does not exist.
2. **A slope read off an iteration's incidental path.** My first mechanism called the residual a
   "sawtooth", from a slope between two points the secant happened to visit. Measured properly it
   is smooth on each branch with ONE handover — and the *right* finding was much stronger than the
   wrong one.

**Why:** both times the number came from something I had already built rather than from a
measurement designed to answer the question. That is the tell, not the magnitude of the agreement.

**How to apply:** before quoting a number as evidence, ask what would have to be true of the
*plant* for it to come out differently. If the answer is "nothing", it is algebra or an artifact of
the path, and it belongs in the derivation section with the gate switched off.

**A pre-registered bar died of naming a DIRECTION instead of a POINT.** P5 said "the cheap reading
from a reference far ABOVE" and never fixed the reference; read with hindsight it passes 3 of 5,
but the winning reference differs at every ramp. At any single fixed reference the expensive solve
wins. Second bar in this lineage to die this way after [[rung82-threshold-law]]'s P1 — **a bar that
names a direction is not a bar until it names a point.**

**The consolation, bounded.** Where a root exists on a smooth branch the secant reaches it to
1.7e−15 in 5 marches — and its apparent 0.10 % "error" is the *bisection's*, whose 13 marches only
resolve to ±0.39 %. But it is a **polisher, not a predictor**: the bracket that tells it a root is
there is the expense it was meant to remove. A causal intervention (moving the start onto each
root) converted 3 of 4 failures, separating "the start was wrong" from "the shape is incurable".

**Also corrects rung 82 § 6:** the *side* of the root is free from one march (`sign(h)`), verified
on both branches at all five ramps. Only the root is a solve.
