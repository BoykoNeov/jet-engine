---
name: rung74-demand-coordinate
description: "Rung 74 — a STATE's coordinate is pure bill (no rank); the redline break was the coordinate, not the lag; the clip floor was an accidental anti-windup device"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2c0441d7-bf7b-430c-9663-64b2eeafc61c
  modified: 2026-08-09T16:56:43.550Z
---

Rung 74 (shipped 2026-08-09) closed rung 73's sharpest seam — **the state-as-demand
coordinate** — and closed it the same way rungs 72 and 73 closed theirs: **by refutation.**

Every fuel-side leg since rung 47 lags its **clip** `g`; a real fuel control lags its
**demand** `w = the fuel it would allow`, with `mf = min(mf_sched, w_f, w_r)`. Substituting
gives `dg/ds = (req − g)/τ + ṁf_sched` — a **state-independent forcing**, so it is in no
Jacobian.

**What the coordinate decides is what the lag is lagging behind.** In clip coordinates the
target rides the SCHEDULE (steady error `ṁf_sched·τ`); in demand coordinates it rides the
PLANT (error `ċap·τ`). Same leg, same clock. So the clip plant overshoots the redline by
+79…+154 K on all six arms and the demand plant sits **under** it — **rung 47's shipped
"the cost of realism is that a lagged governor breaks the redline hold" is a property of the
COORDINATE, not of the lag.**

**Lessons that generalise beyond this rung:**

- **Ask what a proposed change is a change *of* before predicting its effect.** A coordinate
  on a state is a similarity transform plus a forcing: it can move the entire bill (81–354 K)
  while moving the spectrum by exactly nothing. Contrast [[rung69-reference-split]]: a
  *constraint's* coordinate moves the rank. A constraint's coordinate is geometry; a state's
  is bookkeeping, and only one of them is in the Jacobian.
- **A closed-loop difference cannot isolate a forcing.** I pre-registered "the two plants
  agree wherever both legs ride" — refuted at 65 K, because two plants that differ at all
  differ *everywhere downstream*. That was a **law-vs-trajectory** confusion; the fix was an
  **open-loop** reader along ONE trajectory, where the prediction landed at ratio 0.9969.
  Same family as [[rung58-composite-minselect]] (check the SUM, not the term).
- **Check whether a quantity the new law needs has ever been computed.** The demand target is
  the leg's *cap*, and every shipped cap is floored at the schedule (`_surge_fuel` returns
  `mf_sched` itself when dormant). Using the floored one would have **manufactured** the
  dormant-leg cut and let me report it as a finding. The advisor flagged this as a blocker
  before any code was written; measuring it first (reachable 341/341) is what made the rung
  honest rather than an artifact.
- **A ported sign test can invert silently and still read as a finding.** Attack is
  `required > g` in clip coordinates and `cap < w` in demand ones. Keeping rung 52's argument
  order would have selected `τ_rel` on attack — a 3× clock error in the direction that *slows
  protection*, which would have looked like "the demand coordinate is less protective" and
  passed every gate I'd have thought to write. Gated directly.
- **A stop can be doing work no one attributed to it.** [[rung73-applied-reference]] § 0.2
  claimed self-anti-winding was "a property of the composition". Corrected: it is a property
  of the **coordinate's stop**. Remove the stop and the same motion has nothing in its path —
  the masked leg has **no interior equilibrium at all** and the plant does not exist. Rung
  52's `max(0,·)`, inherited unexamined for 22 rungs, is this family's implicit anti-windup
  device.
- **"Negligible" is a statement about a currency.** The floor's address is worth ≤0.33 K on
  peak Tt4 and **0.135 in hand-over time** (a 22% shift). Same law, two answers.
- **Two findings were not predicted at all** and both came from the FIRST march that ran (the
  arrest at the inherited φ floor; the missing equilibrium). Argument for marching before
  writing the readers.

Three of thirteen anchor predictions were refuted and all three became content. I also
mis-scored one of my own (P4) in the first draft of § 9 and corrected it against the table.

Spec `docs/rung74-spec.md`; anchor `docs/plans/rung74-anchor-demand-coordinate.md`;
gates `tests/test_rung74.py`. See also [[rung72-shared-actuator]], [[rung47-lagged-topping-governor]],
[[rung52-asymmetric-lag]].
