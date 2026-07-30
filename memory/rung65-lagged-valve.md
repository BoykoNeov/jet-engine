---
name: rung65-lagged-valve
description: Rung 65 — a lag repairs the SOLVE without removing the DEGENERACY; the § 0 pre-check that was itself a numerical artifact
metadata: 
  node_type: memory
  type: project
  originSessionId: bf984703-24a4-4dc6-8640-93f92c6648fb
  modified: 2026-07-30T16:05:17.327Z
---

Rung 65 (shipped 2026-07-30) gave rung 64's φ-referenced bleed valve a finite bandwidth — the
position becomes a third RK4 state chasing rung 64's command. **HEADLINE: a lag repairs the
SOLVE without removing the DEGENERACY.** Two loops on one variable stay redundant; rung 64 hid
the redundancy in a solver (roundoff), a lag moves it into the STATE as a marginal mode —
exactly frozen, τ-invariant to 1e−15, a one-parameter family whose upper edge is derivably the
valve's own minimality law `b_cmd(0)`.

**Why:** three method lessons worth carrying, all of them about *pre-checks*.

1. **A § 0 pre-check can itself be an artifact.** Probe C appeared to show "a fast valve bleeds
   MORE" (a trade, not pure loss). It was explicit-RK4 instability: `ds/τ = 5` is outside the
   stability region. The scored sweep at `ds/τ ≤ 0.5` showed the exact opposite. The artifact
   looked *physical* — that is what made it dangerous. It is now retracted in the anchor AND
   asserted against in the plant, so no future sweep can reproduce it silently.
2. **The advisor's initial-condition question was load-bearing and I would not have asked it.**
   `b_cmd(0) = 0.0366 > 0` — the valve already rides at s=0, so `b(0)=0` would have injected a
   startup transient into the binding early-ramp LP minimum. Rungs 47/52 start their third
   state at 0 because a *clip* is zero before engagement; a *position* is not. Not every third
   state starts at zero — check, don't inherit the pattern.
3. **My predicted mechanism was right and my predicted consequence was wrong** (the rung-58
   lesson again: check the SUM, not the term). `dφ/dWf < 0` at frozen b — true, ratio 7.8e12.
   "So the second limiter gets part of its plant back" — false; it gets ALL of it back and the
   composite is *still* under-determined. Two predictions (P7 HP-sign-flip, P8 saturated-floor
   degradation) were also refuted, and P8's refutation CONFIRMED rung 64 on a second axis.

**How to apply:** when a rung adds a state whose derivative is a relaxation, gate `ds/τ`
explicitly and pick the initial condition by measuring the equilibrium command, not by
analogy. When a pre-check produces a surprising sign, refine the grid *before* letting it kill
a prediction. See [[rung64-phi-bleed-limiter]], [[rung58-composite-minselect]],
[[rung52-asymmetric-lag]].
