---
name: rung82-threshold-law
description: Rung 82 — a criterion read FORWARD inherits the SIGN of its own reference; the intended headline died in the pre-check; two registered bars were VOID for comparing a physical quantity to a numerical tolerance
metadata: 
  node_type: memory
  type: project
  originSessionId: 6a09a5cb-7f3a-409b-85df-d213120701ff
  modified: 2026-08-11T15:19:36.191Z
---

Rung 82 (shipped 2026-08-11), `ThresholdLawTransient` — rung 81 § 8's first seam: turn rung 81's
criterion from a *label* predictor into a *threshold* predictor. Reader-only (no state, knob or
constant), rungs 77/81's precedent. **The seam's own question is answered NO.** See
[[rung81-authority-clock]], [[rung77-stiffness-ledger]], [[rung74-demand-coordinate]].

**HEADLINE: a forward reading inherits the sign of its own reference.** At one ramp on one plant,
sweeping the reference march across the threshold, `sign(forward − τ*)` follows **the side the
reference sat on — 5 of 5**. A reference above under-predicts, below over-predicts. So the forward
reading is not a prediction of the threshold; it is a report on where the reader started. The
**fixed point** (root of the criterion's own residual) has no reference to inherit from and lands
to 2.7–9.4 %; the forward reading is off by 11–183 %.

**The mechanism, and it is [[rung77-stiffness-ledger]]'s `1/(1−c)`:** read as an iteration
`τ_ref ↦ forward(τ_ref)`, the local slope is **≈ +0.044 above** the threshold and **≈ −1.83 below**
it. So it converges in one step from above — the error *shrinks* as the reference moves further
away (16.3 → 7.0 → 2.4 %) — and diverges from below. **The threshold is the boundary between the
two regimes**, and it explains the ramp sweep entirely: the two ramps where the forward reading
blew up are exactly the two whose reference sat low.

**What it bounds:** rung 81's 99.15 % is a *label* score and holds because every input is read at
the very point being labelled. Across trajectories the criterion has no separation to exploit —
measured on two more knobs: the governor clock keeps **53 %** of its frozen-trajectory coefficient,
and the surge floor, which the criterion places in the *set-point* term, moves the fuel-cap
**slope by +144 %**. The terms are not independent coordinates.

**Why the lessons here are worth keeping:**

1. **THE PRE-CHECK KILLED THE INTENDED HEADLINE BEFORE THE ANCHOR WAS WRITTEN.** I was going to
   ship "the ramp divides one term and leaves the other invariant" — which needs `ċ_f/ċ_r` to be
   ramp-invariant. Two cheap scripts on the shipped plant showed it moving (1.4–3.3 at one ramp,
   1.4–8.2 at another). The advisor's rule that produced this: **when a headline rides on a
   quantity staying put, measure that quantity first — no rig, no bisection.** Same shape as
   [[rung81-authority-clock]] § 0.
2. **TWO REGISTERED BARS WERE VOID FOR COMPARING A PHYSICAL QUANTITY TO A NUMERICAL TOLERANCE.**
   P1 asked whether a prediction lands "inside the measured bracket" — a width I set by choosing
   10 bisections. V5 asked whether a step-refinement moves the threshold "by less than the bracket
   width". **A test whose outcome you set by picking a loop count measures nothing.** Both are
   scored **VOID and not re-scored** in the currency that works; the substantive comparison lives
   in a different prediction that was registered properly. Cf. [[rung78-residual-gauge]]'s vacuity
   traps and [[rung79-state-coordinate]]'s "registering the vacuity condition beat registering the
   result".
3. **A 5-of-5 CORRELATION ACROSS A CONFOUNDED AXIS IS NOT A LAW.** The ramp sweep's sign
   correlation was perfect — but every row used the *same* reference, so the reference's side was
   collinear with the ramp. The advisor blocked the write-up on one discriminator: **hold the
   confound fixed and sweep the thing you think is causal.** 46 s, and it turned a correlation
   into the rung's headline. [[rung74-arrest-interval]]'s "a closed-loop difference cannot isolate
   a forcing", one rung along.
4. **THE SEARCH RESOLUTION WAS COARSER THAN THE EFFECT, AND TWO GATES FLIPPED.** Trimming the test
   from 10 bisections to 7 widened the bracket to 2.3e-3 while the closest signed error is 9.2e-4 —
   the headline read 4-of-5 for a purely numerical reason. Fixed by asserting **bracket < min
   margin / 2** as a *relation*, so a future trim fails there loudly instead of silently.
   **Trim the sweep's extent, never its resolution.**
5. **MY OWN READER FELL INTO THE TRAP MY OWN ANCHOR NAMED.** § 4.5 warned that quoting the swept
   clock instead of the effective one is a 3× error; the first `p4` dropped exactly that factor and
   reported a 5.7× miss where the honest number is 1.9×. Left named in the code, not tidied away.
6. **P5 FAILED IN BOTH HALVES AND THE SECOND FAILURE WAS THE FINDING.** I had the *sign* of
   `∂gap/∂φ_lim` backwards (the floor is the fuel leg's *own*, so raising it makes that leg's cap
   more severe and *lowers* the threshold). And the separation V7 was written to test collapsed —
   the wall moves every term. The advisor also caught me scoring that separation on `ċ_f/ċ_r`,
   **which is not one of the criterion's terms**: the lag term is `τ_gov·ċ_r`. Withdrawing a
   prediction on a quantity the derivation does not contain would have been a false verdict.

**Also:** the ramp rate turns out to be a fuel-side authority lever the whole 46–52 limiter family
never used — at rung 80's clocks it opens the fuel region with **no limiter change at all**.
[[rung44-transient-surge-line]] called the excursion "ramp-rate-driven"; this is the same lever
reaching *which leg holds the actuator*.

**CLAUDE.md funding — a new site.** The row itself was ~58 B of the 334 B shortfall, because it
carried a mechanism sentence and a measured number, both of which the file's own banner forbids in
a rung row. **Check the new row against the rule before hunting prose elsewhere.** Also: a scripted
edit converted the file LF → CRLF and added 253 B of pure whitespace — the guard reads bytes, so
open it with `newline=''`. See [[claude-md-is-a-reference]].
