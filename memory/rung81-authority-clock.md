---
name: rung81-authority-clock
description: "Rung 81 — authority is decided by the LAG not the SET POINT; a masked leg's clock is an EXACT null knob; the pre-check answered the seam before the anchor was written"
metadata: 
  node_type: memory
  type: project
  originSessionId: 7720a05e-ee95-44a9-8883-e5c05e93663e
  modified: 2026-08-11T12:11:13.159Z
---

Rung 81 (shipped 2026-08-11), `AuthorityClockTransient` — rung 80 § 10's first seam: a `demand`
four-loop cell with the φ fuel leg holding the actuator. Reader-only (no state, knob or constant),
rung 77's precedent. See [[rung80-split-wall]], [[rung74-demand-coordinate]], [[rung72-shared-actuator]].

**HEADLINE: a leg that never holds the actuator has no clock.** In the `clip` coordinate at the
split wall the fuel leg is masked for the whole ramp, and a **10× sweep of its own time constant
moves 0 of 1364 floats** (341 points × 4 plant states), at all three governor clocks — while the
same sweep at the shared wall, where that leg does take the actuator, is live. Rung 72's "`min` is
flat in the masked leg" promoted from a Jacobian zero to an **exact invariance of the plant**.

**The mechanism: authority is the LAG's, not the SET POINT's.** In `demand` each leg's state is
the fuel it *allows*, tracking a rising cap through a lag, so it sits below the cap by `τ·dc/ds`.
`min` gives the actuator to the smaller state — the slower leg, not the leg whose own limit is
more severe. Measured: the governor's `required` is larger at **every** fuel-authority point and it
loses anyway. Criterion, derived from rung 74's law with zero constants: fuel holds iff
`required_gov − required_fuel < τ_f·ċ_f − τ_gov·ċ_r`. **CORRECTS rung 74's "pure bill"** — the bill
has no rank of its own, but it *chooses which leg is masked*, hence which Jacobian rows are zero.

**Why the lessons here are worth keeping:**

1. **The § 0 pre-check answered the seam before the anchor existed.** The advisor's "probe the
   shipped cells first" step found the mirror cell outright (7 points, all fuel-held). The right
   move was to declare that in the anchor's own § 0 — *this rung claims no credit for it* — and
   pre-register the questions that were left. The pre-check also moved **three clocks at once**, so
   it could not say which one did it; naming that confound as the rung's first job was what made
   P2 a real prediction instead of a restatement.
2. **THE FIRST GRID SCORED 100 % AND WAS THE WEAKER MEASUREMENT.** `τ_f ∈ {0.02, 0.05, 0.20}`
   gave 506/506 with the closest point **25 % from the criterion's tie** — a test with no hard
   cases in it. Refining the axis to 0.08/0.10/0.12 put points at `9e-04` of the tie, produced 9
   misses, and dropped the score to 99.15 %. **A perfect score is a reason to check whether the
   test can fail**, not a result.
3. **A registered clause can fail while its headline passes.** P1's ≥95 % bar held (worst cell
   95.24 % — cleared by 0.24 points); its "misses within 10 % of the tie" clause was **refuted at
   11.77 %**. Both halves go in the spec, and the refuted clause is **not gated** — gating it at
   the measured 12 % would be fitting the test to the result.
4. **An exact-invariance claim must not rest on a scalar.** The first version compared `max_Tt4`
   and a count. The advisor's push to compare the whole march bit-for-bit is what makes it a
   claim: a reduced number lets a compensating pair read as inertness — the shape of rung 77's
   closure returning a perfect `1.000e+00` having outlived its state block.
5. **A null needs the same knob demonstrated live.** "clip is inert in `τ_f`" is a statement about
   the coordinate until the shared-wall control shows the identical sweep moving the march (fuel-
   held 13 → 3, monotone). With it, it becomes a statement about **masking**.
6. **P3's wording died and its mechanism lived.** Predicted "the fuel region sits on the opposite
   side of the diagonal in `clip`"; measured **no region at all** there (0 of 18 cells). The sign
   flip is real but only visible on the control arm — so it is scored as *wording refuted,
   mechanism confirmed*, and the control's success is not allowed to launder the prediction.
7. **Don't say "independent" when the table says "modulated".** The `τ_f` threshold is crossed at
   every `τ_gov`, so the region spans both sides of the diagonal — but `τ_gov` still moves the
   fuel-held count ~2× at fixed `τ_f`. A gate was written specifically to kill a future
   "independent of `τ_gov`" sentence. Same failure mode as [[rung63-fuel-bleed]].

**Also:** P5 (rung 72 § 3's "`zeros` moves by one at fuel-authority points") is **refuted** in
`demand` — `{1}` in both regimes; reported, never gated. The mask is symmetric: `mask_leak == 0`
exactly on both sides, `n_live ≤ 3` a **seventh** time. The cyclic-product discriminator is
better than rung 80's because both branches (0.0 masked-fuel, 1.0438 masked-gov) sit in **one
table on one code path** instead of being imported from another arm.

**CLAUDE.md paid for itself again** (third rung running): the § Layout ladder was enumerating one
class name per rung 66→80 and growing ~32 B forever; collapsing it to "exactly ONE class per rung,
66→81, each named in its own spec's header" with the two endpoints returned ~390 B and made the
entry rung-count-invariant. See [[claude-md-is-a-reference]].
