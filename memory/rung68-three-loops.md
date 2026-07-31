---
name: rung68-three-loops
description: "Rung 68 — n loops on one variable are rank ONE, so only the CYCLIC product tests it; the pairwise identity restated n times is a tautology"
metadata: 
  node_type: memory
  type: project
  originSessionId: fdaa276f-2ba6-4776-bd17-818af78f127c
  modified: 2026-07-31T14:02:09.930Z
---

Rung 68 (shipped 2026-07-31, commit 821e65f) closed rung 66's "THREE loops on one variable"
seam with a φ-referenced `StatorLimiter` beside the lagged valve and lagged fuel leg — five
states, three clocks.

**The transferable lesson: when you generalise an identity from n=2 to n=3, check what is
actually INDEPENDENT before quoting it.** Rung 66's identity is one scalar (`R_q·C_g ≡ 1`).
Stating it three times over three pairs is the same measurement three times — and `tr M = −3`
is the hardcoded diagonal, and the second invariant is `3 − Σ(pairwise)`. Imposing all three
pairwise identities *still leaves the block a free parameter*: the CYCLIC product
`x = R_q·C_v·V_g`, with `det = (x+1)²/x` exactly. A block can be pairwise-degenerate and still
rank 2. I nearly published four numbers that were one number. The gate that keeps it honest
hand-builds a block with every pair at 1 and `x = −3.5`, confirming `det ≠ 0`.

**Three things measurement corrected that reasoning had gotten wrong**, all mid-build:
- I claimed `det`'s double root made it "quadratically insensitive". The sensitivity table
  showed its noise floor is squared along with its signal, so its SNR is the *square*. The real
  reason to quote `x` is redundancy, not sensitivity.
- I wrote `τ_s → 0` converges to rung 66. It runs the other way (−88%). A third loop is an
  ADDITION, so only `τ_s → ∞` removes it — inverting every earlier lag in the family, where
  the fast limit was the richer object.
- My "correction" that a saturated loop leaves `det = 0` was itself wrong off-manifold, which
  is where a march always is. The original prediction was right.

**Two artifacts that would have counterfeited the rung**, both now gated:
- Checking the regime of only the BASE point, not the twelve PERTURBED evaluations — a central
  difference straddling the `max(0,·)` kink returned `c1 = 1.3e+2` where the derivation says ~0.
- Rung 66's RK4 constant admits `ds` at which this plant reports the floor EXACTLY held with a
  violation integral of ZERO. Worse than [[rung65-lagged-valve]]'s retraction, which at least
  blew up. **A guard nobody has run past is a tautology** — so `_rk4_floor` is a separate method
  purely so a gate can override it and measure the band it refuses.

Cross-rung: EXTENDS [[rung64-phi-bleed-limiter]] — authority is inert in company and binding
alone, so it is a property of the lever *plus what else holds the same variable*. And
[[rung53-variable-stator]]'s "a margin is a DISTANCE" finally reaches the SIGN of a protection
credit (+91.7% in φ, −57.4% in incidence, same loop, same march).
