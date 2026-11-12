---
name: standing-engine-a3
description: "Roadmap A3 shipped 2026-11-12 (b815bf2): Mach 0 on every flown view via an off-by-default `standing` switch. Lessons: a census must write its knob where the request READS it (the slam nests its flight); a reply with no ok key is a result, not a refusal; agreement with Mach 1e-6 proves nothing off a table's grid"
metadata:
  node_type: memory
  type: project
  originSessionId: 17d5bff2-877b-4a6d-b4b1-7c9587ef171c
  modified: 2026-11-12T18:35:00.220Z
---

**Shipped 2026-11-12, commit `b815bf2`** (plan § 49): the standing engine on Fly it (dry and lit), the throttle slam,
Controls, and the Design view's lit afterburner. `OffDesignMatcher::standing` and `TwoSpoolCore::standing` are
`Cell<bool>`, off by default; the page's solvers set them. Rung 41 and the oracle gates catch Mach-0 failures
(including panics, via `catch_unwind`) as control flow, so a switch is the safe form. The gate proved nothing moved.

**Lessons.**
- **A census must set the knob where the request reads it.** The slam keeps its flight under `slam.fly`, so my first
  census set `slam.M0`, ran the default Mach twice and "agreed" on all 19 rows. The advisor caught it. Check one row's
  parsed settings before trusting any census.
- **A reply with no `ok` key is a result, not a refusal.** Grid and map ops reply that way. Counting them as "both
  refused" left Fly it's running-line and map ops uncompared.
- **A limit check cannot validate something that is off the grid at both ends.** Mach 0 and Mach 1e-6 both sit below
  the speed tables' lowest slice, so their agreement says nothing about the extrapolation. Compare against points
  INSIDE the grid instead (grid miss: +1.3 % at rest vs ≤0.41 % between slices).
- **A march's inner closures stop at their own tolerances**, so two flights 1e-12 apart differ ~2e-8 mid-ramp. Set a
  march bar from that measurement, not from the steady ends' 1e-8.
- The gate's one failure was an old test asserting the refusal I removed (`sandbox_fly.rs`). Grep the tests for the
  old refusal's words BEFORE the gate run: `"Mach"` with `m0: 0.0`.

Related: [[roadmap-and-standing-engine]], [[always-commit-and-push]], [[windows-tooling-file-hazards]].
