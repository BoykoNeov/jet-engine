---
name: rung69-reference-split
description: "Rung 69 — a loop's COORDINATE decides zero-vs-rank (zeros = n − m, m = CONSTRAINT count); det J was the wrong instrument and c1 the right one"
metadata: 
  node_type: memory
  type: project
  originSessionId: c73b9a77-7345-4119-a8b0-aa639c413e40
  modified: 2026-07-31T15:10:03.901Z
---

Rung 69 (shipped 2026-07-31) closed [[rung68-three-loops]]'s named strongest seam: the SAME
stator, referenced to INCIDENCE (`M_i = T_c − (1/φ − v)`) instead of to `φ`, beside the same
lagged valve and lagged fuel leg. Same plant, same lever, same wall at the design setting —
only the coordinate moves.

**The transferable lesson: the rank counts CONSTRAINTS, not loops.** Every row of the actuator
block is a multiple of *its own* constraint's gradient, so `rank M = dim span{∇c⁽ⁱ⁾}` and
`ZEROS = n − m`. The loop count never enters. That one line subsumes rung 66 (n=2,m=1→1),
rung 67 (n=2,m=2→0), rung 68 (n=3,m=1→2) and this one (n=3,m=2→1).

**The instrument I would have inherited was BLIND, and the advisor caught it before I built.**
Rung 68's discriminator is `det J` via `c0 = (x+1)²/x`. Under the split the two loops still on
`φ` keep *exactly parallel rows*, so `det J = 0` **identically** — it never looks at the third
row at all. Inheriting that test would have read "rank one" and shown nothing. The
discriminator is the SECOND invariant `c1` (≥0.20 vs ≤1e−11, ten-plus orders). **Generalising
a rung means re-deriving which invariant carries the claim, not just re-running the reader.**

**Method notes worth keeping:**
- **Fill a table from measured output, never from memory.** I drafted per-cell `v` figures into
  the spec's ledger before re-running with the new column; two were wrong by 4×. Same family as
  [[rung63-fuel-bleed]]'s "check a quoted number was taken at THIS rung's settings".
- **Name a negative control as one.** `pair_RC` involves only the fuel and valve closures, so at
  a fixed base point it is literally the same computation in both columns — it *structurally
  cannot* move with the reference. Quoting it as "the shared pair keeps the identity in BOTH"
  reads as independent evidence; it is a floor calibration.
- **The evaluation manifold can be FORCED.** `R_q·C_g = 1` needs both φ loops at their own rest
  points. Under a split there is no point where all three constraints hold (`φ=φ_lim` and
  `M_i=m_lim` force `v=0`, the dormant stop), so the base must be the SHARED constraint's — and
  it then lies *outside* the new loop's own band. Exactness is a property of a shared constraint.
- **Grid-converge anything whose SIGN is the finding.** The dominant root here is a complex
  pair, a different aliasing character from rung 68's real root, so its `ds` table is not
  inherited. Halving `ds` moved nothing past the third figure — but [[rung65-lagged-valve]] is
  why that gets measured rather than assumed.

**A prediction that missed, and the miss is content:** I expected a smaller null space to mean
a smaller `s=0` IC family. It is the opposite — 187 %/291 % spreads against rung 68's
45.2 %/105.5 %. **A null space is a SHOCK ABSORBER**: redundant loops redistribute a moved
start among themselves (a displaced stator start is absorbed to 2.3e−14 under the shared
constraint, and 22 % of it survives under the split). So zero count and IC sensitivity move in
OPPOSITE directions, and rung 68's refusal to attribute its own growth to the second zero was
right.

Cross-rung: [[rung53-variable-stator]]'s "a margin is a DISTANCE" lands one level up again —
rung 68 showed a credit needs its WALL named, this one shows it needs its loop's REFERENCE
named too (every one of the four stator credit cells flips sign). And one scalar `k` sets the
pairwise split, the cyclic product AND a bandwidth-independent damping floor `ζ ≥ 1/√(1−k)` —
[[rung67-cascade-a]]'s `P` recurring in a different mechanism.
