---
name: rung33-double-root-repair
description: "2026-10-21 solver repair (not a rung): rung 33's subsonic residual has two roots off the design flight, so its two-ended march refused SUB-IDLE on points that run (edges move with the flight: 550–725 K up to 775 K–past 1000 K at sea level); the march now walks up from its low end where both ends share a sign; oracle cells licensed by a rule (user) — lessons: trace the PAGE's call path before calling a bug page-visible; a probe's grid is not the bug's extent; a licence predicate must follow the code's own dispatch; a licence predicate can cost more than the gate"
metadata:
  node_type: memory
  type: project
  originSessionId: a0a0dec2-0c90-4244-8b8c-6d2c29d3915f
  modified: 2026-10-21T09:47:37.833Z
---

Repair (not a rung), 2026-10-21, closing rung 105's seam (anchor G4). Spec: `docs/rung33-spec.md` § The double root.
Code: `matcher.rs` `match_subsonic` (pin path; the law path already searched from the design expansion). Gate:
`rust/tests/rung33_double_root.rs` (two roots + same-sign ends; LOWER root to 1e-10; = rung 34 flat-map
equilibrium to 3.9e-11, bar 1e-9; below the merge still refuses, old words first). Oracle licence:
`rust/tests/common/double_root.rs` — 6 `offdesign_oracle` cells (3 gases, Mach 0.3, 600/650 K) + 4 cells of
`map_oracle`'s rung-31 cross-check, SUB-IDLE → subsonic point. **User chose "allow by a rule"** over a default-off
setting. Then the user chose option 3: this first, then the Fly-it page's own low-throttle refusal (rung 34:
cannot evaluate below ~0.6 design speed, "nozzle back-pressure exceeds pt9"; the running point sits just below —
hot day 650–725 K) — repaired the same day, [[rung34-low-speed-wall-repair]]; its check is this repair's roots.

**Lesson 1 — trace the page's call path before saying "you can hit it on the page".** I recommended the fix as a
page bug; Fly it solves through rung 34, not rung 33. A side agent and the advisor both flagged it; re-asked.
**Lesson 2 — a probe's grid is not the bug's extent.** G4 said "650–900 K, one flight"; it reached six sea-level
flights, ISA static included — and then I overstated it the other way in the commit message ("646 K to ~1000 K at every sea-level flight"; the advisor caught it: edges 550–725 K / 775 K–past 1000 K by flight). Quote an extent per flight, from the tally, never from the headline cell.
**Lesson 3 — a licence predicate must follow the code's own dispatch.** "Same-sign ends + a root between" also
named CHOKED cells (900–1500 K): the subsonic residual has the same shape there, but the choke solve settles them
first. An exit-Mach test did not separate them; the matcher's branch (upstream of the repair) did.
**Lesson 4 — a licence predicate can cost more than the gate.** A 400-point grid on every cell took the
off-design oracle from 2 to 13 min. Pre-filtering from the gate's own values did NOT fix it (still 15 min):
timing per cell showed 7 refused equilibrium-gas cells scanning all 400 points (~0.3 s each). Fixed by proving the
same fact cheaper — one residual evaluation at the returned point, between the same-sign ends. Time it, then fix.
Also: typed `python3` → the Store stub hung again ([[windows-tooling-file-hazards]]); stopped by task ID.
Related: [[rung105-hp-vane]], [[solver-walls-repair]], [[idle-floor]].
