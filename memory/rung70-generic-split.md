---
name: rung70-generic-split
description: "Rung 70 — the split buys the RANK but the RING needs one lever on two walls; a predicted null REFUTED into an invariance, and a gate I caught computing my own formula twice"
metadata: 
  node_type: memory
  type: project
  originSessionId: 22b36d25-fe2b-48cb-aaeb-d8a65809b062
  modified: 2026-07-31T15:59:58.843Z
---

Rung 70 (shipped 2026-07-31, commit 27559e6) put rung 47's `Tt4` governor as the odd loop
beside rung 65's φ valve and rung 68's φ stator — rung 67's substitution applied to rung 68's
triple. `n=3, m=2`, the same cell as [[rung69-reference-split]], reached by a different route.
It closed **two** listed seams at once, which rung 69 § 11 had already identified as one seam
from two sides.

**The methodological lessons, which are what generalize:**

1. **A predicted NULL that fails is worth more than one that holds.** I pre-registered "no
   complex pair at ANY bandwidth" (P8). It is false — the ray that nearly silences the stator
   rings. Scoring it REFUTED rather than "refined" forced the better claim out: the floor is
   *rung 67's ζ exactly*, because `min()` selects the pair that IS rung 67's `P`. A third loop
   sharing a constraint adds a zero and moves the achievable damping **nowhere**. That
   invariance was not in the anchor. The anchor stays unedited — a prediction revised after
   the measurement is not a prediction.

2. **State an identity's CONTINGENCY or it will read as structural.** The floor-equals-rung-67
   identity holds only because `pair_RV` came back positive; had it been the more negative one,
   `min()` would select a gain rung 67 never measured. The gate asserts the *condition*
   alongside the consequence, so a plant that broke the sign fails there instead of silently
   invalidating the claim.

3. **I wrote a tautological gate and caught it before shipping** — the [[rung68-three-loops]]
   /rung-67-gate-9 failure mode, again. My first `c1` test evaluated my own closed form under
   two clock assignments and called the disagreement a measurement. Fixed by routing every `c1`
   through the shipped `_invariants` (the actual Jacobian) and letting the closed form appear
   exactly once, as the thing under test. **Ask of every gate: what is the independent
   quantity, and did I compute it twice?**

4. **"It moves across a grid" is not a discriminator when the grid moves too.** `c1 != 0` was
   rung 69's result, not mine. What discriminates a two-term `c1` from rung 69's one-scalar
   form is a **clock SWAP at fixed τ_g**: this plant gives ratio 0.9077, a one-scalar null built
   from its OWN gains gives exactly 1.000000. Build the null from the same measurement, so it
   differs in exactly one respect.

5. **Sibling integrator, never an in-place edit.** Rungs 68/69 stayed bit-for-bit (worst diff
   exactly 0.0 over 341 points) and [[golden-fingerprint-gate]] agreed. Copying ~100 lines that
   differ in two places beat parameterizing one integrator.

The advisor caught three real things pre-build: that rung 69's floor isn't attained at matched
clocks either (so my "no longer attained there" clause was wrong), that both split pairs would
be weak cross-lever gains (predicting the null that then failed usefully), and that the clock
swap was the missing gate. See also [[rung67-cascade-a]], [[claude-md-is-a-reference]].
