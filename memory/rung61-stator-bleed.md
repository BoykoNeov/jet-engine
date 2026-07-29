---
name: rung61-stator-bleed
description: "SHIPPED rung 61 = stator + bleed together; a compensating lever buys back the COORDINATE not the BILL (73-102% of the overspeed survives because the phi-debit was a loading REBATE); the seam's \"takeover\" refuted with the opposite sign to my own prediction"
metadata: 
  node_type: memory
  type: project
  originSessionId: 252271dc-6ae3-4f7d-ab36-15290fd469da
  modified: 2026-07-29T02:13:35.041Z
---

**Rung 61 (2026-07-29) — STATOR + BLEED together**, the two halves of rungs 36/41's standing
concession. `StatorBleedMatcher` in `turbojet/engine.py`, spec `docs/rung61-spec.md`, anchor
`docs/plans/rung61-anchor-stator-bleed.md`, 43 gates in `tests/test_rung61.py`.

**HEADLINE:** a compensating lever buys back the **COORDINATE, not the BILL**. Rung 42's
valve removes the whole of rung 53's `φ`-debit machine-exactly, and **73–102 % of the
stator's overspeed survives** — because the stator's `φ`-drop was itself a partial **rebate**
on the loading it removed (`base(φ)` rises as `φ` falls), and restoring `φ` forfeits it. At
`v = 0.30` the compensated point **overspeeds the uncompensated one**: undoing the lever is
strictly worse than leaving it alone.

**What generalises beyond this project:**

- **Three predictions scored WRONG carried the rung.** P5a (overspeed retention < 50 %) was
  refuted and its refutation *became* the headline; P1a's sign was backwards and that turned
  "does the valve take over?" into "the valve competes for the same budget." Writing all six
  predictions down before measuring is what made those scorable — see [[rung56-per-row-capacity]].
- **The advisor's blocker was a duplicate-finding check, not a physics check.** My first
  headline (`ΔM_i = v`, exact, map-invariant) was *rung 60's already-published tautology*
  (`docs/rung60-spec.md` line 82) reached by a third route. Before claiming an exact identity
  as a finding, grep the prior specs for the same value. It became a demoted lemma.
- **Then the advisor caught the SAME failure mode a second time**, in the repair: the
  `ψ_comp = base(φ) − v(1+l)φ` "closed form" is the shipped `psi` method evaluated at a known
  argument. Gated as a *plumbing check*, labelled an identity in the spec. The finding is the
  `base` slope (the rebate) and the two-term factorisation, not the algebra.
- **A "derived scaling" whose binding constant is MINE is not derived.** The compensable
  ceiling looked like `1/(1+l)` across five shapes (ratio 0.70–0.72). Re-running at three
  values of my own `_B_CAP` moved the ratio 0.55 → 0.70, so **the ceiling is truncation and
  is NOT published**. What survived cap-free is the price scaling `b* ∝ v(1+l)` (spread
  1.3–3.3 % across shapes). There is a gate asserting the ceiling is cap-dependent, so the
  killed claim cannot creep back.
- **Two silent-failure traps in composing two matcher ladders by MRO** (both would have given
  plausible numbers with no exception): a parent's sibling-constructor (`at_setting`)
  hard-constructing its own class silently drops the other lever from every sweep; and a
  co-operative `super().__init__` chain that forwards a fixed argument list silently leaves
  the second device at its neutral setting. Call the other parent's `__init__` explicitly.

**Cross-rung:** CORRECTS rung 53's "cleaner per-spool DoF than rung 42's bleed" — scoped to
the lever *in isolation*; the pair's inter-spool arrow **survives the flat-η island** (it is
bleed's energy channel). New machine-zero: the pair's **thrust** interaction is exactly `0.0`
on flat-η (η-mediated, switchable off) while its **speed** interaction is not.

Related: [[rung53-variable-stator]], [[rung60-matched-floor]], [[rung58-composite-minselect]].
