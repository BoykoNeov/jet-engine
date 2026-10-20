# The Controls view warns below idle — probes (sandbox plan § 39; the scratch crate, worktrees and crash-map builds were temporary and are deleted)

All `[[bin]]`s of one scratch crate built against the repo (the § 39 code). Their temp-folder paths
(`W:/temp/claude/idle-warn/…`) point at a folder now deleted.

- `signs.rs` → `signs.txt` — at every below-idle start the page now keeps (six variants × four flights × four maps,
  400–1100 K): does the DISPLAYED thrust (`governor::thrust`) share the sign of the thrust per inlet air the rule judges?
  1 210 starts, 0 disagree.
- `slam_floors.rs` → `slam_floors.txt` — per variant, map and flight: the lowest start that BUILDS, the lowest above
  idle, the lowest whose whole default slam (to 1400 K) finishes, and how the slams between stop (1 854 stops, all
  "fuel closure does not bracket", 1 691 at the first step).
- `mono.rs` → `mono.txt` — 642 inferred-thrust tables a governed run reads (EPR / fan speed, speed correction none /
  grid, three airs, Mach 1.6–3.3), 330 holding rows below idle: every one one-way.
- `leans.rs` → `leans.txt` — every crash-map run (rung 101's map, seed 99) now stopping "lean": its trial mixture (all
  at 0.0036–0.0045, the lean wall 0.004, but one slam up at 0.0097 — reclassified `low_speed`), points, ends.
- `gap.rs` → `gap.txt` — governed runs (EPR / fan speed, speed correction none / grid, Mach 2–3.3, three airs, four
  shapes, starts 850/1000/1150 K) the old code refused as beyond the tables' reach, which the page now runs: the worst
  belief-vs-thrust miss along each, against runs that reached before at the same flights. `split.rs` → `split.txt` —
  rung 98's Mach-3 sea-level cell: the gap's ram part (−4 441) and nozzle part (+4 724), the gap itself +283 → −38.
- `stops.rs` — the last of a throwaway series: which sea-level low-start starts march to the end (540 K flat-LP, 550 K
  flow/press, the bleed lever at 520/560 K — all do).
- The crash maps: `../rung101/crash101.rs` (seed 99, 1 500) and `../idle-floor/crash.rs` (seed 36, 1 200), unchanged,
  built against worktrees of `6e49d33` and `1f57583` and the repo; the counts are in plan § 39.
