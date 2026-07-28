---
name: rung55-stage-stack
description: "SHIPPED rung 55 = the STAGE STACK; a POSITIONAL lever buys relief from the part it doesn't move, so cost factorises as (1/K)x(v*ratio) and the row count has an INTERIOR optimum; discharges rung 54's seam; free rung-2b polytropic validation found while gating"
metadata: 
  node_type: memory
  type: project
  originSessionId: 1062c34d-bb04-4b53-a753-2c7522d51c54
  modified: 2026-07-28T07:02:07.628Z
---

Rung 55 (SHIPPED 2026-07-28) resolves each compressor into `K` stage blocks —
`StageStack` + `StageStackMatcher` in `turbojet/engine.py`, spec `docs/rung55-spec.md`.
It discharges the seam [[rung54-stator-throat]] named after refuting capacity as the escape
from [[rung53-variable-stator]]'s overspeed.

**Headline is a general law about POSITIONAL levers:** a lever acting on part of a machine
buys its relief from the part it does not act on, through whatever the parts share (here
shaft speed). So a front-ROW stator holds the front stage's design incidence for **+2.3 %
`N_L` against rung 53's +66.7 %** — and the collapse **factorises**, `dN_ratio =
(1/K)×(v*_front/v*_lumped)`, holding to 3 % over `K` = 2…16. The honest half is the same law
backwards: relief in the ROW COUNT peaks at 3–4 rows of 8 then **REVERSES** below bare, as
the worst stage migrates into the rows the stator doesn't move. **The first optimum in this
project that is a COUNT.**

**Why it was a rung and not a re-read** — the advisor's blocking gate, run *before* any
pre-registration: the front-to-rear `φ` spread alone is a functional of the `(τ_c, π_c)` rung
39 already solves. The content is the FEEDBACK — per-stage `ψ(φ_k)` means the work is no
longer `ψ(φ_face)·n²`, so the stack MOVES the running line (up to 27 % of `τ_c−1`, exactly
`0.00e+00` at `K`=1). **Run that discriminating measurement before building anything.**

**Method notes worth reusing:**
- `φ_1 = m/n` EXACTLY, so the face `φ` every rung since 32 reads IS the front stage's ⇒
  rungs 36–53 are BOUNDED (they read the right object, placed 2–4 % optimistically), not
  refuted — rung 53's own style.
- Reduce is an IDENTITY at `K`=1 (no stack object built, inherited eta loops), like rung 53.
- Scope had to be GATED not declared: the stack enters the SOLVER, so a leak into the
  rung-34/40/43 transient closures would move rungs 34–52. There is a test that runs a
  rung-43 transient with a stack live and demands bit-for-bit.
- **A gate I wrote asserting the WRONG SIGN found a free result:** the derived per-stage
  efficiency sits ABOVE the lumped one (reheat), and converges first-order on rung 2b's
  POLYTROPIC `e_c`. The stack interpolates rung 2 → rung 2b without being told about
  polytropic efficiency at all.
- Advisor caught a real defect: the row-count reversal was first seen at coarse scan and
  could have been a bracket artifact (rung 54's turning-point hazard). Resolved by measuring
  the residual curve + filling in rows 5 and 7 — physics, not bracket. It also caught that my
  +66.7 % silently contradicted rung 53's published +26 %: same schedule, different
  denominator (design vs bare-at-throttle) — [[rung43-two-shaft-fuel-metering]]'s
  currency-circularity lesson again. **Name the denominator.**

Next seam: **per-row capacity** — rung 54's throat channel per stage, which should bind at
the BACK while incidence binds at the front.
