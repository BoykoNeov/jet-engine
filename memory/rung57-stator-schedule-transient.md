---
name: rung57-stator-schedule-transient
description: "SHIPPED rung 57 = the stator schedule on the transient plant; a wall-moving lever has NO CLOCK (bounds rungs 46-52's timing family as rung 53 bounded their currency); my own probe error inverted a sign and the advisor's blocking checks killed two candidate findings"
metadata: 
  node_type: memory
  type: project
  originSessionId: 226734a1-f6e1-4800-9fe4-47eec0a23959
  modified: 2026-07-28T10:31:36.869Z
---

Rung 57 (shipped 2026-07-28) put rung 53's floor-moving variable stator on rungs 43/45's
fuel-metered two-shaft transient — `StatorSchedule` + `ScheduledStatorTransient`.

**HEADLINE: a wall-moving lever has no CLOCK.** Over a 20× ramp-rate range the surge margin
swings 52 % while the surviving share of the stator's rotation moves 1.05 points, and rung 53's
*design-point* Jacobian `1/(2+l)` predicts that share to 3.9 % — off design, out of equilibrium,
mid-transient. So rungs 46–52's engagement-timing family (rung 48's engagement time, 49's two
edges, 50's relocation, 51's rate, 52's self-pinned trigger) is a property of **point**-movers.
**Rung 53 bounded their CURRENCY; rung 57 bounds their CLOCK** — same shape, one axis over.
Two thirds of the rotation is eaten by the lever's own work channel. A state-fed schedule
**self-cancels** 10–25 % (closing raises the speed the schedule reads, so it opens back up).
**CORRECTS rung 53's P5**: both `==`-reported exact zeros break on the transient and neither via
η — they were the shaft balance's doing, which rung 40 deliberately removed. See
[[rung53-variable-stator]], [[rung56-per-row-capacity]].

**Why:** the method lessons here are the durable part, and three of them cost real work.

**How to apply:**
- **My own probe was wrong, and the error had a specific shape: an INCONSISTENT START.** I
  overrode `_close_fuel` only, so `equilibrium`/`fuel_for_Tt4` still saw the bare maps and the
  scheduled march began off its own running line. That start transient inverted a sign (the HP
  stator appeared to *debit its own spool* by −0.02; consistently armed it *credits* by +0.06)
  and contaminated three probes' worth of numbers. **On any lever that moves the running line,
  arm every closure the start point is solved through, and check `nu0` moved.** Generalises
  beyond this repo: whenever a knob changes the equilibrium, the initial condition is part of
  the knob.
- **The advisor's blocking checks killed two candidate findings, both of which I would have
  shipped.** (1) A monotone `r`-drift and a residual sign-flip that looked like the rung's
  content were artifacts of the contaminated start — a `ds`-convergence run plus a
  constant-`v` (consistent-by-construction) re-run dissolved both. (2) A "the residual changes
  sign at r≈0.22" claim did not survive changing the *matching convention* between the schedule
  and its comparison constant. **Before building on a differenced quantity, vary the thing you
  matched on.**
- **The advisor's own hypothesis was refuted too** — it predicted FULL ≈ START-ONLY (the
  schedule as an initial-condition device). Measured: START-ONLY delivers 0.27 → −0.07 of the
  credit as `r` grows. Bringing the measurement back beat complying, exactly as in
  [[rung52-asymmetric-lag]] and [[rung45-transient-fuel-surge]].
- **A guard fix can break a path that depended on nonsense.** My first version of the off-map
  fix asserted `Tt3 > 0` inside the closure; that broke every shaped-map `equilibrium`, because
  the bracket's high wall legitimately sits at `Tt3` = −721 K and only *works* because a shaped
  η island cancels the sign back. The shipped guard tests the *residual* for non-realness
  instead — it fires only where the arithmetic actually left the reals, and changes no number.
