---
name: rung85-blade-speed-in-progress
description: "Rung 85 (blade-speed walls) is IN DESIGN, no code — the anchor doc docs/plans/rung85-anchor-blade-speed.md carries the probes, sources, user decisions and the open items; the pre-check found the stack's K and C were two disguised, inconsistent blade speeds"
metadata:
  node_type: memory
  type: project
  originSessionId: edd8c029-bea9-4b2c-99e0-c4ce222591f5
  modified: 2026-10-06T02:33:39.336Z
---

**State (2026-10-06):** design only; nothing in `rust/` touched. The anchor doc
`docs/plans/rung85-anchor-blade-speed.md` is the record — read it first.

**User decisions:** (1) BOTH walls — airflow (front-row relative tip Mach) and strength
(Ti-6Al-4V blade-root pull, 14 CFR 33.27's 120 %) — whichever binds; (2) the design flow
coefficient `Φ_d` becomes a DESIGN INPUT (Rotor 37 mean-line 0.537, NASA TP 1659 Table II),
`Φ_d` = 1 the bit-for-bit reduce to rungs 55/56.

**Next:** advisor check-in on § 6 (is the incidence loops' `1/Φ_d` gain change in scope?),
source the airflow-wall LEVEL from a textbook and Ti-6Al-4V first-hand, register predictions
(Q4 level-vs-level re-measured on the walls' own K; Q5 two-sided — stator pre-swirl LOWERS tip
Mach), then a separate commit for a Rust-owned CLI golden (first rung with no Python segment),
then code.

**Why — process lessons:** (a) a dimensionless model can still carry a physical quantity
IMPLIED by several inputs — convert each to that quantity before adding a wall on it (P-A: `K`
and `C` implied 158 vs 220 m/s on the LP spool); (b) the advisor caught a COST compared to a
LEVEL (rung 53's +66.7 % is a part-power change, not N/N_d) and a wrong "both walls scale with
N" (pre-swirl); (c) probe the side-effects of a new wall on the neighbouring disclosed constant
before registering anything (P-C: `φ_d` = 1 forced C ≈ 0.97–0.998).

**How to apply:** follow the anchor doc's § 3/§ 4/§ 6 open items in order; take no credit for
§ 0's pre-check numbers. See [[rung56-per-row-capacity]], [[per-row-blading-negative]].
