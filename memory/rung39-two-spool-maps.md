---
name: rung39-two-spool-maps
description: "SHIPPED rung 39 = two-spool + component maps; REFUTED rung 38's own prediction (map opens ONE arrow HP->LP, not a 2x2) while confirming its verdict; advisor caught my wrong framing BEFORE code via the pi_LPC-cancellation algebra"
metadata: 
  node_type: memory
  type: project
  originSessionId: d3d7071a-b4a7-40a7-851b-e1a05e0090ed
  modified: 2026-07-22T10:48:49.582Z
---

**Rung 39 (shipped 2026-07-22)** — `TwoSpoolMapMatcher` in `turbojet/engine.py`, spec
`docs/rung39-spec.md`, anchor `docs/plans/rung39-anchor-two-spool-maps.md`, gates
`tests/test_rung39.py` (10). Builds the seam [[rung38-two-spool-matching]] named as its own
likely correction: a `ComponentMap` on each spool.

**The result is the rung-28 shape** (verdict confirmed, stated reason refuted) — rung 38
predicted "a real map would very likely reintroduce genuine 2×2 coupling." **Wrong.** `π_LPC`
**cancels** out of the HP compressor's corrected flow (`pt4/pt25 = π_b·π_HPC`, because the LPC
raises pressure and mass flow proportionally), so the map opens **exactly ONE arrow, HP→LP**:
`η_HPC` moves `π_LPC`, `η_LPC` leaves `π_HPC` **bit-for-bit zero**. The cascade **acquires a
direction** rather than dissolving. Structural novelty = **two shaft speeds** ⇒ the slip
`N_L/N_H`, an exact identity (=1) on CPG+flat maps, broken predominantly BY THE MAP — which
**inverts** [[rung32-component-maps]] (there the map only re-labelled map-free work).

**Why:** two process lessons worth keeping, both about catching my own bad framing early.

1. **The advisor blocked my framing before I wrote any code, on algebra I hadn't done.** I
   proposed "the terminal leaf dies / 2×2 coupling returns" and flagged it myself as possibly
   tautological ("add a feedback loop, feedback exists"). The advisor said the framing was not
   just weak but *factually wrong*, and handed me the one-line cancellation to check. It was
   right. Had I coded my version first I'd have built a joint 4-η iteration, measured ~1e-15
   noise in the closed leaf, and shipped a much weaker rung.
2. **Structure the solve so the finding is a code-level guarantee, not a numerical reading.**
   My probe's jointly-iterated cascade left 1e-15 residue in the leaf at some points. Building
   it strictly triangular (energy → HP-η-loop → LP-η-loop, turbine-η outer loop inert at
   `a_t=0`), with the `π_LPC`-free corrected flow written in closed form in the source, made it
   `EXACT-0` everywhere and let the gate assert `==`. Same discipline as rung 38's `_cascade`.

**How to apply:** for any successor rung that a predecessor *predicted the outcome of*, do the
algebra before committing to the predicted framing — the prediction is a hypothesis, not a
premise. And when a finding is "X cannot reach Y", build the code so X literally isn't in
scope where Y is computed; then gate it bit-for-bit. See [[rung37-combustor-dynamics]] for the
probe-first precedent (initial signs BOTH wrong) — here too a measurement refuted one of my own
hypotheses (I predicted the speed slip would be largely *map-free*; it is identically **zero**
without the map).

**What rung 39 leaves open:** the two-shaft transient (now well-posed — rung 38 supplied no `N`,
rung 39 supplies two); a two-spool surge line (rung 36's is single-spool; whether the slip
*protects* the LP spool at low power is the claim this rung declines to make); the LP subsonic
branch; a real hardware/CFD map.
