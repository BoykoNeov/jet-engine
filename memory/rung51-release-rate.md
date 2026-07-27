---
name: rung51-release-rate
description: "SHIPPED rung 51 = the release RATE (tau_rel); the debit is NOT a functional of the applied-fuel trajectory — proven by a TWO-SIDED BRACKET, after the pre-registered gate turned out confounded"
metadata: 
  node_type: memory
  type: project
  originSessionId: c6ca4d8e-b519-41d8-9494-845c021445f1
  modified: 2026-07-27T19:24:47.422Z
---

**Rung 51 (shipped 2026-07-27)** — the release RATE `τ_rel`, rung 50's own named seam. The
min-select clip is **faded linearly** to zero over `[s_off, s_off+τ_rel]` instead of dropped:
stateless (a pure function of `s`), so rung 50's RK4 argument carries verbatim.

**Headline: the debit is NOT a functional of the applied-fuel trajectory.** The two HARD
releases at a fade's own two ends bracket it *pointwise in applied fuel* (measured, 0
violations) **and** in total `fuel_removed` — yet the faded run lands strictly **OUTSIDE** both,
shallower, on both spools. No monotone functional of the fuel level, and no function of the
total deficit, can do that ⇒ the debit answers to the **RATE**, and [[rung50-release-edge-isolated]]
§5's deficit law is **BOUNDED** to the instantaneous hand-back.

**Why:** three method lessons worth keeping.

1. **The pre-registered gate was CONFOUNDED and the probe is what found it.** The plan (mine and
   the advisor's) was to drop faded points onto rung 50 §5's fixed-release deficit→depth curve.
   At matched release-*completion* a fade always removes LESS fuel, and §5 already says less
   deficit ⇒ shallower — so the curve cannot separate them. The **two-sided bracket** replaced
   it: bracket the faded run by hard releases at *both* ends and show it escapes. The advisor
   explicitly retracted its own framing when I brought the measurement.
2. **My own prediction P2 was FALSIFIED and shipping it that way was the point.** "|relief|
   monotone falling in `τ_rel`" holds in the deep regime but *rises* in the shallow one, where
   the faded point merely INTERPOLATES. That scope is gated as an explicit **negative test** and
   put in the headline paragraph — the advisor's line was that a scope living only in
   Concessions "reads as universal in the rung table line, and the rung table is what future-you
   reads."
3. **The strongest number was found, not constructed.** A *naturally-occurring* matched-deficit
   pair fell out of the sweep — total fuel removed matched to 0.02 % with **opposite-signed**
   relief. Because it was found rather than solved for, it dodges the matched-currency trap that
   blocked [[rung48-accel-schedule]] twice.

**How to apply:** it also **CORRECTS a shipped rung** — rung 50's precondition (a) ("the release
must land at or after that spool's own bare minimum") is mis-stated; the relocation crossover
sits *upstream* of it, and rung 50's **own §1 LP column already violated it** un-noticed. Rung
50's relocation headline is untouched — only its boundary. Lesson: when correcting a shipped
rung, lead with the contradiction internal to *its own published table*, and re-measure any
constant you'd otherwise read off a test file (the advisor blocked the write-up until I
measured `s_hp*` myself rather than trusting `S_HP_STAR_2`).

**Next seam, named:** the asymmetric fast-attack / slow-release **LAG** — refused here because
its release edge is *emergent* (moves with the rate, reinstating the confound `s_off` exists to
kill) and it needs a state-dependent kink in the derivative. See [[rung49-phi-feedback-limiter]]
for the leg it would act on.
