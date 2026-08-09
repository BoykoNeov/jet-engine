---
name: rung76-fuel-dependent-cap
description: "Rung 76 — a device in a leg's LAW reaches the MASKED leg, one in the shared PLANT reaches the AUTHORITATIVE one; the set-point solve turned out to be a GAIN (1/(1−c)), which was the unpredicted finding"
metadata: 
  node_type: memory
  type: project
  originSessionId: 25030ba2-d77b-4ff3-84dc-04fc67c6ff15
  modified: 2026-08-09T19:52:02.639Z
---

Shipped 2026-08-09. `SensedCapTransient` — rung 73 § 11's second seam (a cap that depends on
the fuel it is asked about), deferred by rungs 73, 74 **and** 75. Fifth declared knob
`_cap_law ∈ {solve, sensed}`, and the first rung since 71 that adds **no constant at all**.

**The headline.** Rung 48's `Wf/pt3` leg states its law as an inequality *on the fuel*, and a
real limiter evaluates it from the pt3 it senses; `_sched_fuel` instead solves the implicit
fixed point. That was a modelling choice, not the schedule. Reading it as written puts the cap
in the **plant both legs read** rather than in either leg's **law** — and min-select masks a
law but cannot mask a plant. So it moves the **authoritative** fuel diagonal (`−20.0000 →
−16.25…−16.42`, fitting `(c−1)/τ_f` to `1.1e−9`, both references, both stator arms), which is
the one entry rungs 73/74/75 each report as *moved 0.0 relative* — and leaves the masked leg
exactly alone. `n_live ≤ 3` a **fifth** time, so the obstruction is neither a law nor a plant
but the **composition**: `min`'s flatness in the masked state. See [[rung75-antiwindup-device]].

**What was worth the most, and it was a refutation.** The pre-registered `det J(sensed)/
det J(solve) = 1−c` came back at 0.7 % off. Chasing *why* produced the rung's strongest result,
which nothing predicted: differentiating the fixed point `cap = cap_sensed(cap, q)` gives
`d(cap_solve)/dq = (d(cap_sensed)/dq)/(1−c)`. **Writing a limiter as a set-point solve
multiplies its sensitivity to every other state by `1/(1−c)`** — measured 1.228…1.246 against
prediction at `< 1e−8`. A limiter written as a solve is a *stiffer* limiter than the schedule
it implements. Rungs 48–75 could not see it because none had a second reading of the same cap
to difference against.

**Why:** the transferable lessons, in order of how much they cost to learn —

1. **A prediction that misses by a little is worth more than one that lands.** The 0.7 % gap
   was the only thing pointing at § 3. Tightening the tolerance or widening the claim would
   have thrown away the rung's best finding.
2. **An aggregate over a min-select must be split on which branch binds.** `_cap_fuel` is
   `min(accel, φ)` and only the accel branch has a sensed form, so where φ binds the knob is
   inert *by construction*. Two cells came back at `1.9e−01` against `1e−9` everywhere else —
   points where the two laws being differenced sat on **opposite sides of the cap's own
   switch**, so the difference measured a *leg change* and reported it as a broken law. Guard
   the second min the way rung 72 guards the first. Related: [[rung63-fuel-bleed]].
3. **A structural absence can be a consequence, not a gap.** The masked-leg cell has zero
   points in 24/24 combinations — because rung 48's leg is feedforward (early) and the governor
   is feedback (late), so the leg that binds the cap is also the leg that holds the actuator.
   Report the mechanism, don't hunt for the cell.
4. **Check which coordinate a claim lives in before gating it on a trajectory.** "The two laws
   share an equilibrium" is exact (`6.94e−18`) as a property of the *laws*; as a claim about
   the march tail it is false, because the schedule stops but the spools are still spinning up
   — the march never reaches an equilibrium at all.
5. **Arming something the family never armed relocates every comparison.** Rungs 72–75 never
   armed the accel leg, so every quoted number of theirs is at the wrong settings. Re-measuring
   the whole 2×2 internally turned that from a concession into evidence — rung 75's headline is
   *reproduced* rather than cited. This is [[rung63-fuel-bleed]]'s lesson at plant scale.
6. **The carried-knob trap did not bite** for the first time in fifteen rungs — it was written
   into `_shared_rig`/`at_lever`/`_cap_march` before the first reader ran. Its **cousin** bit
   instead: the schedule κ is read off the plant's own equilibria, so an `AccelSchedule` built
   on one machine and marched on `_shared_rig`'s is a schedule for a *different engine*. Hence
   `accel_for`.

**How to apply:** when a rung's pre-registered identity misses by a small, *consistent* amount,
treat the residual as the subject and derive it — don't re-tolerance it. When differencing two
laws through a `min`, first ask whether both readings pick the same branch. And before quoting
any rung-73/74/75 number beside a rung-76 one, check the plant: the accel leg is armed here and
was not there.
