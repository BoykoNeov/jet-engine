---
name: rung62-bleed-schedule
description: "SHIPPED rung 62 = the bleed schedule beside the stator schedule on the transient; a state-fed schedule's LOOP has a SIGN; the advisor blocked my headline as rung 57's own published sentence"
metadata: 
  node_type: memory
  type: project
  originSessionId: 91059527-b7f9-424e-b862-3a038bea5219
  modified: 2026-07-29T04:46:36.726Z
---

SHIPPED rung 62 (2026-07-29) = rung 61's named seam — a `b(n_L)` bleed schedule beside a
`v(n)` stator schedule on the transient plant. `BleedSchedule` + `ScheduledBleedTransient`
in `turbojet/engine.py`; spec `docs/rung62-spec.md`, anchor
`docs/plans/rung62-anchor-bleed-schedule.md`, 57 gates in `tests/test_rung62.py`.

**HEADLINE:** a state-fed schedule closes a feedback loop on itself through the shaft speed
it reads, and the loop's SIGN is the sign of the lever's own `dn/d(setting)`. Rung 57's
stator schedule self-cancels (FULL/RAMP 0.77–0.83); the bleed schedule AMPLIFIES itself
(1.09–1.10). **Both signs were derivable from rungs 57 and 61's own published tables before
either was measured.** SECOND FINDING: two loops through ONE state don't compose — a bleed
schedule triples the stator's surrender (0.169→0.724) while the stator leaves the bleed's
amplification alone to 0.7 %; a ONE-WAY arrow. CORRECTS rung 61: its steady ≤2.3 %
superposition was the SHAFT BALANCE's doing (same shape as [[rung57-stator-schedule-transient]]
correcting rung 53's exact zeros) — 9–29 % sub-additive here.

**Why:** three method lessons worth more than the physics.

1. **The advisor killed my headline before any code**, because it was rung 57's own § 2
   sentence ("both channels are algebraic in the instantaneous state … neither has anywhere
   to put a clock"). I was about to bill a CONFIRMATION as a correction — the same
   duplicate-identity failure recorded in [[rung61-stator-bleed]] and [[rung60-matched-floor]].
   **Before claiming a mechanism is new, grep the predecessor spec for it.**
2. **A pre-registered prediction refuted with the OPPOSITE sign produced the better finding**
   (P3 said the bleed's loop would RESTORE the stator's surrender; it triples it). Again the
   pattern in [[rung61-stator-bleed]] and [[rung49-phi-feedback-limiter]].
3. **The advisor's second block was on evidence quality, not framing** — it demanded a
   grid re-run, a relocation check, and a LEVEL-MATCHED control before the 3× could be
   quoted. The matched control (constant `b` at the schedule's own commanded 0.0709 vs the
   schedule) is what turned "it's the loop" from assertion into result.

**How to apply:** the `_powers` trap generalises — rung 40 factored the ODE residual out of
`_instant_tail` into `_powers` for the Newton's inner loop, so ANY new physics in the
transient closure must be threaded through BOTH or the equilibrium converges to 1e-12 on a
residual the plant does not use (it returned `n_L` 5.3 % wrong with `φ_L` right to 1e-3 and
no exception). Nothing internal catches it; the cross-object gate against rung 42's steady
`match` did. **When a corner of a reduce contract has no ancestor, validate it against a
different object, not against itself.** Also: a schedule placed so its own head start pushes
the state past `n_lo` is measured SATURATED (`db/dn`=0) — check the placement before reading
any state-fed schedule's loop.
