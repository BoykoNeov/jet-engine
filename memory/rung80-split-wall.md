---
name: rung80-split-wall
description: "Rung 80 — a LEVEL split opens the four-loop cell but never a fourth AUTHORITATIVE loop; the seam had named the wrong noun, and all three pre-registered predictions were refuted"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9a63895c-02d0-4394-8646-88432a118433
  modified: 2026-08-10T18:54:48.669Z
---

Rung 80 (shipped 2026-08-10) gave the airflow legs their own surge-margin floor `sm_air` beside
the fuel leg's `sm` — `docs/rung74-arrest-interval.md` § 8's seam, the last untried route to
`n_live = 4`. Plant `SplitWallTransient`; spec `docs/rung80-spec.md`; anchor
`docs/plans/rung80-anchor-split-wall.md`; gates `tests/test_rung80.py`.

**HEADLINE: a LEVEL split separates loops on the CONSTRAINT; it cannot separate the two that
share the ACTUATOR.** The split opens the four-loop cell in `demand` (empty at every shared wall
— 0/341 lever motion) but `min` still masks one fuel-side leg with an exactly-zero column, so
`n_live` ≤ 3 a sixth time.

**The lessons, in the order they cost the most:**

1. **When a seam has survived five rungs, suspect its NOUN before its mechanism.** Rungs 72–76
   each closed a route to "four live loops" and 79 named the solver's short-circuit. The seam was
   written in the *riding* noun (all four loops off their stops) and every attempt was scored in
   the *authority* noun (which loop reaches the actuator). Both are real; they differ by exactly
   the leg min-select masks. Rung 80 satisfies the seam as written and leaves the quantity every
   prior rung was chasing untouched. Extends [[rung72-shared-actuator]].

2. **A leg's activity is a function of what its law READS, not of the achieved state.** The
   predicted failure — levers lift `φ` past the fuel wall, fuel leg goes dormant, a total order
   with one live floor — is wrong because the fuel leg's cut is evaluated at the SCHEDULED fuel
   and the airflow legs hold their floor at the APPLIED one. The lever's lift *erodes* the fuel
   leg's cut (242→233→215) without extinguishing it. Before predicting a leg goes quiet, write
   down which quantity its law is a function of.

3. **The arrest belongs to a COINCIDENCE, not to a floor** — CORRECTS `docs/rung74-spec.md` § 2.2
   and `docs/rung74-arrest-interval.md` § 4. Both split arms march at every wall while `φ(0)` is
   still visibly lifted onto 0.78 / 0.80. On a shared wall "the floor that lifts" and "the leg
   with no margin left" are the same object, so no shared-wall experiment could have told them
   apart. **A rung that separates two things a prior document held equal can correct it without
   contradicting a single measurement it made.** See [[rung74-arrest-interval]].

4. **A dropped keyword would have returned the predicted confirmation.** `_shared_rig` is
   overridden six times, each calling `super()` with an explicit argument list. A new kwarg at the
   base gets swallowed silently — and since the prediction was *the levers do nothing*, the reduce
   test would have passed BECAUSE the knob was ignored and the reader would have honestly reported
   the predicted result. Fix: carry the knob as an attribute (the ladder's own idiom) and read the
   walls BACK off the limiter objects the rig will march with. Same family as
   [[rung62-bleed-schedule]]'s `_powers` trap.

5. **An exact zero needs a positive control on the same code path.** `mask_leak = 0.0` and cyclic
   product `= 0.0` at every split cell — but in `demand` every cell masks the *same* leg, so the
   two are one zero seen twice. The `clip` baseline contains one cell where the *other* leg is
   masked, and it returns 0.99999999986 on the identical path. Without it "exactly zero" is
   indistinguishable from the instrument declining to measure. Fourth reload of
   [[rung78-residual-gauge]]'s vacuity lesson.

6. **A liveness counter on a frozen plant reports full activity — reproduced again.** The shared
   arm at `φ_lim = 0.78` reports 320 four-loop points with `max Tt4 = Tt4_lo` exactly: the plant
   never moved. Every row now carries `riding4_valid`.

**And a CLAUDE.md-budget lesson:** the largest refund in that file's history (~300 B) was deleting
a whole section whose stated deletion condition had been met twelve rungs earlier. Successive
rungs kept compressing sentences instead of re-reading their own deletion notes. Check for
SECTIONS whose deletion condition is met before compressing prose. See
[[claude-md-is-a-reference]].
