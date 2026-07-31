---
name: rung71-full-split
description: "Rung 71 (n=m=3) — rank independence is NOT constraint independence; det J factors into rungs 67's+69's own conditions; rung 69's damping floor was the c0=0 corner"
metadata: 
  node_type: memory
  type: project
  originSessionId: 571df774-4dee-4de4-beb7-04a262ba0e8b
  modified: 2026-07-31T17:26:48.136Z
---

Rung 71, **THE FULL SPLIT**, shipped 2026-07-31 (commit `f0f9551`). Rung 69's move — swap ONE
loop's COORDINATE, change nothing else — applied to rung 70's plant: rung 68's `φ` stator becomes
rung 69's INCIDENCE stator, beside rung 47's `Tt4` governor and rung 65's `φ` valve. `n = m = 3`,
zero zeros — the last unoccupied cell of rung 69's table, and rung 70's own named strongest seam.
Detail lives in `docs/rung71-spec.md`; this records only what the repo does not.

**THE HEADLINE — rank independence is not constraint independence.** Full-rank Jacobian, and the
third loop rides over 7.9 % of the march, because at the valve's set point
`M_i = m_lim + v ≥ m_lim` for every admissible `v ≥ 0`: `{φ ≥ φ_lim} ∩ {v ≥ 0} ⊆ {M_i ≥ m_lim}`.
So `zeros = n − m` counts **gradient directions**, not **live loops**. That distinction is the
transferable idea — a loop can have an independent gradient and a redundant feasible set, and no
Jacobian reading will ever show it. See [[rung69-reference-split]], [[rung70-generic-split]].

**Method lessons worth carrying (the reason this file exists):**

- **The kill-check came before the anchor, and it found the rung.** The seam could have been
  infeasible (rung 69's incidence loop saturates over 84 % of the ramp alone). Probing the window
  FIRST turned "is this buildable?" into the headline. When a seam's feasibility is genuinely in
  doubt, measure it before writing the pre-registration, and say so in the anchor's own § 0.
- **Two window numbers, and only one was mine.** I first wrote "the third loop is live over
  2.05 %" — that is the JOINT window, narrowed by a governor engaging late for rung 67's imposed
  `Tt4_max`. The stator's own window is 7.9 %. Containment owns where the stator's window *ends*;
  someone else's set point owns where the joint one *starts*. Rung 63's *check a quoted number was
  taken at THIS rung's settings*, turned on my own headline — the advisor caught it post-commit.
- **A predicted null refuted into a mechanism, again.** P2 (zero erosion, from rung 70 § 5's law)
  failed: the stator keeps 5.5 % of its solo credit while sharing no constraint. The correction
  is the *same* containment seen integrally — erosion has a set-containment channel beside rung
  70's gradient-sharing one. Rung 70's P8 did the same thing; this family's best results keep
  arriving as refuted nulls.
- **`det J` alive and still blind.** First plant in the family with `det J ≠ 0`, and it factors
  into `−(1−pair_RC)(1−pair_CV)` = rung 67's non-degeneracy condition × rung 69's. `pair_RV` — the
  only genuinely new gain — cancels against the *reverse* cyclic product, so it is invisible to
  the determinant here and to the cyclic product at rung 70. **Only `c1` has ever seen it.**
- **Know which closed form is a tautology.** `c1 = Σ(1−pair_ij)/(τ_iτ_j)` is an identity of any
  matrix with `−1` on the diagonal — gating it would be rung 67 gate 9 again. `c0`'s closed form
  uses four of six gains and asserts the other two drop out, so *that* is the claim. Ask which
  inputs a formula ignores before deciding whether checking it measures anything.
- **The damping reader has now been rebuilt three times in four rungs**, each time because the
  rung changed which root is which (69: dominant-root; 70: magnitude-sorted non-zero pair; 71: the
  pair identified by its IMAGINARY part, `None` when the spectrum is real). Rung 70's reader is
  wrong on 4 of 12 arms here. Inheriting a reader across a rank change is the recurring bug.
- **Measure a residual's floor, don't assert it.** Halving `(dg, dq, dv)` four times left every
  identity residual bit-unchanged (8.324e−4 at 1/1 … 1/16) ⇒ root-finder floor, not truncation.
  Rung 68's standard; one cheap run converts a claim into a measurement.
- **Reuse the march when nothing is added.** Rungs 68/69/70 each shipped a sibling integrator
  because a STATE was being added. Nothing is added here, so rung 70's integrator runs this plant
  unchanged — the reuse is *gated* (the method is the parent's own function object, plus
  trajectory-key equality) because `test_numeric_fingerprint.py` does not watch this path.

Contingency worth remembering: the containment holds **because** `m_lim = T_c − 1/φ_lim` exactly
(rung 69's matched wall). An offset wall breaks it — that is rung 71's own sharpest next seam, and
it trades against the confound rung 69 refused. Only the RANK half of the headline is general.

See also [[xdist-module-fixture-cost]], [[test-suite-speed-policy]].
