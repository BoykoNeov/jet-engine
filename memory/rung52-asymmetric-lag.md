---
name: rung52-asymmetric-lag
description: "SHIPPED rung 52 = the asymmetric fast-attack/slow-release LAG; refuted rung 51's own deferral reason with a ONE-LINE structural argument (a self-releasing leg pins its own trigger); headline = a self-releasing limiter cannot debit the spool it watches, so rung 50's watched-side debit is an ARTIFACT OF FORCING"
metadata: 
  node_type: memory
  type: project
  originSessionId: d496b940-18e7-41b3-9f76-145b15849d35
  modified: 2026-07-28T01:34:49.706Z
---

Shipped **rung 52** (2026-07-28) — the asymmetric fast-attack / slow-release lag on rungs
48/49's min-select legs (`AsymmetricLag`, `integrate_fuel(…,lag=…)`, `_integrate_fuel_asym`,
`lag_relief`, `lag_sweep`, `factorization_grid`). It was [[rung51-release-rate]]'s own named
next seam, and the rung is largely about **checking the reasons rung 51 deferred it with**.

**The method lesson — the probe was decidable at the desk, and I should have seen it sooner.**
Rung 51 deferred the lag because "its release edge is EMERGENT: sweep the rate and the release
moves with it." That is **false by a one-line argument**: `τ_rel` is never *read* while
`required > g`, so the whole pre-crossing march is bit-identical across a rate sweep — the leg
**pins its own trigger**, which is the property rung 50 had to *force* with `s_off`. I ran a
136 s sweep to find this; the advisor was right that the question was decidable before building.
The spec now says explicitly that **the numerics corroborate the argument, they do not carry
it** — worth repeating rather than dressing a sweep as a discovery.

**The headline, and the cross-rung payoff.** A self-releasing leg releases only *after* the
watched variable has begun to recover, and its own attack transient has already pinned that
spool's minimum at the engagement edge (rung 48's arrest law). So **a self-releasing limiter
cannot debit the spool it watches** — which **BOUNDS [[rung50-release-edge-isolated]]'s
watched-side debit to FORCED releases** and **RESTORES [[rung49-phi-feedback-limiter]]'s
watched-side identity** for every physically-realisable leg.

**The advisor asked for a row that breaks the credit's zero; there is none, and that was the
better result.** I searched 7 floors × 2 ramp rates × 5 attack constants, found no exception,
and produced the mechanism instead. Surfacing that conflict (rather than switching to satisfy
the request) was the right move — the advisor retracted and said to lead with it. **Careful
detail it flagged**: the argument needs the *actual* `φ_lp` minimum, not `required`'s turnover —
under a lag `φ_lp` dips *below* `φ_lim`, so they are different objects; the step that closes it
is "the lag's undershoot is largest EARLY."

**Second finding**: the two clocks separate **one way**. `τ_att` owns the credit *exactly*
(machine-zero spread over `τ_rel`); the debit is irreducibly **joint** (additive residual
59–70 % of the main effects at both ramp rates, and not multiplicatively separable either — the
ratios drift and change sign). The fast-attack/slow-release design premise is **half true, and
the half that fails is the protective one.**

**Where the bit-identity actually stops** — a test failure taught this and it is now in the
spec: strictly, up to the RK4 step that *straddles* the crossing (its later sub-stages already
read `τ_rel`, and the crossing is *recorded* one grid point downstream). So `s_cross`, `s_eng`
and `relief_watched` are exact, but `g` at the crossing carries a ~4e-4 relative partial-step
residual. Loosening a tolerance is fine — silently loosening it is not.

**Rung 51's other two reasons**: (2) was **form-dependent** — an asymmetric-*rate* switch has
both branches sharing the vanishing `(required − g)` numerator, so the RHS is a **kink, not a
jump** (Lipschitz, RK4-legal); rung 51 had sketched the bad `max(g, required)` *level* form.
(3) **stands** — an exponential never completes, answered by *declaring* the release edge
fractional-of-schedule and reporting every debit at **two** ε.

**Next seam**: the lag's SHAPE, and the two-lag cascade (`tau_gov` + `lag`) this rung refuses —
a redline lag and a surge lag on one plant, which is what a real FADEC runs.

Note: adding this rung tripped the CLAUDE.md size guard — see [[claude-md-is-a-reference]]. The
fix was compressing rungs 44–52's rows and several negative-entry lines, **not** raising the
budget.
