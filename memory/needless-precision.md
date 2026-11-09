---
name: needless-precision
description: "User principle 2026-11-09: drop needless precision on principle, not only for speed — a solver tolerance far below anything physical is a cost in itself; keep it just finer than the smallest effect the model shows"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 08e841b3-c18a-42da-a411-7f1d0030cec3
  modified: 2026-11-09T21:00:15.467Z
---

When choosing a search or solver tolerance, the user wants it set by physical meaning, not by "as tight as possible".
Offered three ways to coarsen the hold searches (1e-9 K today), they chose 0.01 K for ALL of them over "only the slow
search coarse", which was just as fast and left every run byte-identical: "it is not about speed — it is about the
spirit of needless precision."

**Why:** precision far below anything physical (a billionth of a kelvin on a burner temperature) is false rigour.
It costs solves, and it makes tests pin digits that mean nothing. The user would rather update those tests than keep
the digits.

**How to apply:**
- When a tolerance comes up, offer the physically meaningful value even if a narrower change would be enough
  for the immediate goal. Don't argue for a smaller blast radius when the only thing protected is meaningless digits.
- The floor on coarseness is the smallest effect the model must still show: 1 K was too coarse because rungs
  99–101's tenths-of-a-percent misses read as exactly 0. Measure that floor before proposing a value.
- Tests whose bars were set to the old precision (1e-9 "meets its target") move to the new tolerance as a normal part
  of the change, not as a cost against it.
- Still a rule-side change: measure first (census, failing tests), bring the numbers, and wait for the user's choice.
  Related: [[slow-held-runs-speedup]].
