---
name: mixing-jicf-anchor-negative
description: "The anchored-δ(J)/JICF-trajectory attack on the mixing ceiling — NEGATIVE, NOT a rung; confirms rung 22, sharpens the SCALE-negative, isolates a SECOND unanchored exponent"
metadata: 
  node_type: memory
  type: project
  originSessionId: 37d984d9-aae3-4fa0-9d69-bf60e0d7c67e
  modified: 2026-07-20T19:49:24.727Z
---

The mixing-ceiling seam's "anchored `δ(J)` law" attack (named by [[mixing-scale-negative]] as the one
worthwhile next step) was **built, tested, NEGATIVE, NOT shipped and NOT a rung**
(`docs/mixing-jicf-anchor-negative.md`). It anchors rung-22's penetration exponent with the published
jet-in-crossflow trajectory `y/rd=A(x/rd)^m` (m≈0.28 Pratte-Baines / 0.33 Hasselbrink-Mungal).

**Two findings.**
1. **PENETRATION axis — CONFIRMS rung 22.** Any bent-trajectory exponent (m>0) breaks the measured
   Holdeman `(S/H)√J` collapse (g-optimum drifts 27–30% per 2× geometry); only `δ∝rd` (momentum-DEPTH
   scaling, m=0 = rung-22's own law) is consistent. **Deflated honestly** (advisor caught the over-read):
   g depends on δ/H BY CONSTRUCTION, so "collapse ⟺ p=1/4" is algebra, not data — the claim is a
   RULING-OUT of the bent forms, NOT "data anchors m=0", and m=0 is a depth scaling, NOT a near-field
   claim (near field is m≈1/2).
2. **EMISSIONS axis — still NOT pinned.** Holding penetration at p=1/4, the emissions turn's LOCATION is
   penetration-anchored and grid-robust at C≈3.12 (stable across cap ×0.4–2, c_D ×0.5–2), BUT its
   EXISTENCE rides on a SECOND unanchored exponent — the spread/entrainment growth p_σ (0 flat, 0.25
   turns, 0.5 erases) — which JICF TRAJECTORY scaling doesn't supply; and the GLOBAL min sits at a
   max-segregation endpoint in 6/7 configs (rung-22's concession). **Anchoring penetration MOVES the free
   parameter from penetration to spread; it does not eliminate it.**

**Why:** the standing mixing ceiling (a real spatial/transported-CFD PDF) is the project's biggest open
seam; this records that the JICF-trajectory route confirms rung 22 but does not pin the emissions optimum
— the seam needs BOTH mixing exponents anchored, or the full CFD pattern.

**How to apply:** do NOT re-run the JICF-penetration + growing-σ-at-hand-picked-p_σ construction. A new
attempt needs an ANCHORED spread/entrainment law (murkier than the trajectory for a confined jet) or a
real transported/CFD cross-plane PATTERN. Method precedent: bound/consistency-first (proto1 is pure
algebra+geometry, cheap and decisive), then the expensive chemistry discriminator only to settle what
the cheap check couldn't. Probes in `M:\claud_projects\temp\rung-mixing-jicf\` (outside git); the tracked
doc is the durable record; nothing in the shipped tree changed (monkeypatch harness like
[[mixing-scale-negative]]). Related: [[tau-res-negative]], [[turbine-march-negative]].
