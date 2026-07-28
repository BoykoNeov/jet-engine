---
name: rung58-composite-minselect
description: "SHIPPED rung 58 = the composite min-select (stator schedule + fuel leg on one plant); two levers do not superpose; my headline \"inherits a clock\" was refuted by my own table and caught pre-ship"
metadata: 
  node_type: memory
  type: project
  originSessionId: daf467e5-1c17-47b4-9f26-dda553bd5146
  modified: 2026-07-28T15:32:23.008Z
---

SHIPPED rung 58 (2026-07-28, commit `e9e7123`) = **the COMPOSITE min-select** — rung 53/57's
stator schedule armed BESIDE one fuel-side limiter on one plant, the seam rung 57 named.

**Headline: two levers DO NOT SUPERPOSE.** The mixed second difference `ΔI` is +9.51 % of the
stator's credit and runs ONE WAY (the stator moves the fuel leg's engagement time 0.16 % — a
factor of 59). Mechanism = **relocation × state-feed**, and 86 % of it is *predicted* from the
two marches that never saw the fuel leg.

**The lesson worth keeping: my headline was wrong and my own data refuted it.** `ΔI` is
strongly ramp-rate-dependent, so I wrote "a clock-free lever INHERITS its partner's clock" into
the spec, CLAUDE.md, `main.py` and a test name. The advisor did arithmetic on the table I had
already produced — no new run — and showed `ΔI` ANTI-correlates with the bare credit, so the
DELIVERED credit (`bare + ΔI`) is *flatter* in `r` than the bare one (8.53 → 6.80 % schedule,
3.11 → 0.89 % constant). The finding inverted from "BOUNDS rung 57" to **"CONFIRMS rung 57"**:
only the DECOMPOSITION is clocked, and a decomposition is not a deliverable.

**Why:** this is the project's fourth brush with a currency artifact ([[rung43-two-shaft-fuel-metering]]
circularity, [[rung45-transient-fuel-surge]] referenced excursion, [[rung49-phi-feedback-limiter]]
confound) and the FIRST caught before shipping. The trap shape: a *component* of a quantity
moves a lot while the *quantity itself* moves less — always check the sum, not the term.

**How to apply:** whenever a rung's headline rests on "X varies strongly", compute the thing a
designer is actually handed and check its spread too, before writing the claim anywhere.

Other content, all gated: the CURRENCY is a finding (`M_i`'s wall is the metal, `M_φ`'s moves —
they disagree on the SIGN, so only `M_i` can carry a four-cell difference); rung 49's φ-floor is
**not composable at all** (the admissible-floor windows of bare and statored machines are
DISJOINT — [[rung53-variable-stator]]'s law reaching a limiter's SET POINT); and where a floor
pins both cells, credit == `v` to machine precision, so [[rung57-stator-schedule-transient]]'s
two-thirds erosion goes to exactly zero. Next seam = the MATCHED schedule (re-derive `κ_ss` on
the armed machine — the confound this rung refuses, now readable because the mechanism says
what to subtract).
