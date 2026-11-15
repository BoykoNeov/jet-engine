---
name: equivalent-jet-split-a6
description: "Roadmap A6 shipped 2026-11-15: the page's efficiency split on the equivalent jet speed; the roadmap's own bound claim was false — derive a bound's CONDITION before testing it; never test a product of a quotient"
metadata:
  node_type: memory
  type: project
  originSessionId: ec1e12e7-75c6-4477-b867-737deb49fd94
  modified: 2026-11-15T00:26:29.766Z
---

Roadmap A6 (2026-11-15): Design + Fly it show thermal/propulsive efficiency on the EQUIVALENT jet speed
(`sandbox::equivalent_jet`), beside the model's speed-only split (kept, recorded bit for bit). Convergent
opening design: 40.02 % / 43.21 % against speed-only 15.07 % / 114.73 %.

**Process lessons:**
- **A plan's "stays between 0 and 100 %" was false.** Algebra first: `η_p ≤ 1` iff `(1+f)(V_eq−V0)² ≥ f·V0²`,
  so near zero thrust the fuel's mass still lifts it (3 of 507 grid cells). Test the bound only where its
  derived condition holds, assert the converse on the rest, and COUNT both so the test cannot go vacuous.
- **Defining a factor as a quotient makes the product test a tautology** — compute each factor from its own
  formula, then check the product (the advisor caught this before code).
- **A new reply field must not add a refusal path:** `outcome_json`'s `finite(..)?` turns a NaN into "does
  not run"; new display-only fields go out as null (`or_null`) and the page maps null → NaN → "–".
- Notes' number checker (`docs/notes/check-numbers.ps1`) accepts a number from `EXPERIMENTS.md` when the
  section says "experiment" — so quote new page numbers in the guide first.

Related: [[roadmap-and-standing-engine]], [[instrument-fed-by-what-it-certifies]].
