---
name: rung54-stator-throat
description: "SHIPPED rung 54 = the stator-row THROAT (rung 53's refused capacity half); BIND-NEVER-RELIEVE theorem + the severity law; CORRECTS rung 53's turning-point concession AND finds a latent defect in its shipped root-finder"
metadata: 
  node_type: memory
  type: project
  originSessionId: 684250ef-b266-4594-a80e-3cc08a255119
  modified: 2026-07-28T05:01:26.310Z
---

SHIPPED rung 54 (2026-07-28) — **the stator-row FLOW CAPACITY channel**, the seam
[[rung53-variable-stator]] named and refused.

**The constant problem, resolved by SPLITTING it.** Rung 53 refused this channel because it
"needs a new constant (area per unit setting)". The cascade cosine rule `o/s = cos α₁` derives
the area law's **SHAPE** off rung 53's OWN coordinate (`v = tan α₁`) with zero new constants —
but the **LEVEL** still needs one (`C = MFP(M_th0)/MFP(1)`, the design row's fraction of choking
capacity). The escape: every verdict is delivered as a **derived threshold ON** `C`, and `C` is
disclosed as a *design throat Mach* so a reader can judge it. **Do not let the project's
"zero new constants" habit distort a header when there IS one** — the advisor blocked exactly
that, and "shape derived / level disclosed / verdict as threshold" is the honest form.

**Two headline results.**
1. **BIND, NEVER RELIEVE** — a *theorem*, not a measurement: `v` enters the solve through
   `solve_n` alone (rung 53 P1) and the throat enters no solver, so `X` is a post-hoc functional
   of the solved state. ⇒ the reduce is an **INVARIANCE OVER A PARAMETER** (every matched field
   bit-identical for *every* `C`), stronger than rung 53's identity at a point. ⇒ the seam's own
   expectation — that capacity would buy back rung 53's +26 % overspeed — is **REFUTED
   structurally**; no area law could. The real mechanism is stage rematching.
2. **The severity law** — rung 53: a MARGIN is coordinate-dependent; rung 54: **so is a
   CONSTRAINT'S SEVERITY.** The throat cuts the SETTING 30 % and the incidence MARGIN 4 %.
   Rung 53 named saturation as a *limitation*; it is also what makes a hard limit **cheap**.

**Two corrections to a shipped rung (the rung-28 shape — verdict kept, reason corrected).**
Rung 53's Concessions say the incidence benefit "does not turn back … the apparent turning point
is **not** reached". Measured: **interior peak on 3 of 5 disclosed shapes** (material on 2), so
the ceiling is the incidence PEAK, not `solve_n`'s map-validity artifact — and rung 53's P7
schedule **ceases to exist** inside the envelope on 2 shapes. Rung 53 generalised the one shape
it tabulated.

**And the correction reached its CODE, not just its prose.** `incidence_schedule` brackets by a
**doubling ladder** justified by "the residual is monotone decreasing in `v`". Where the peak is
interior that premise fails and the ladder steps *over* the root, reporting a schedule
unreachable when it exists (`steep` @Tt4=1200, root at `v*`=0.909). Left algorithmically
untouched (its published table is the shape where the premise holds) + a docstring pointer;
rung 54 brackets off a scan instead. **Lesson: a shipped rung's stated assumption can be
load-bearing in its solver, not only in its claims — check the code, not just the prose.**

**Process notes.** Three of four registered predictions FAILED, and P-B's failure (a retention
>100 %, impossible unless `M_i` turns over) is what forced out the biggest finding — the same
probe-inverts-the-author pattern as rungs 42/46/49. I also made a claim ("a tighter throat can
be worth more") and **withdrew it in writing** in the anchor once I defined usable authority
correctly (clip at the peak). The advisor set the order of work (probe before prose), fixed the
constant framing, and caught that the spec's 20/20 claim is carried by `binds != "edge"`, not by
the weaker `throat_before_edge`.

Next seam named: **STAGE REMATCHING — the stage stack** (`K` stage blocks sharing `τ_c`).
