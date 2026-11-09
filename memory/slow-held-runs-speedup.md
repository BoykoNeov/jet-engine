---
name: slow-held-runs-speedup
description: "Sandbox plan § 43 — slow generator hold runs: memo (2278f6d, 2026-10-22) + hold searches started at their own throttle (shipped 2026-11-09); one request stays ~13 s; POTENTIAL FOLLOW-UP: coarser hold tolerance (user: ~1 K would do), measure first"
metadata:
  node_type: memory
  type: project
  originSessionId: 2d66d455-34c9-471a-a004-8e54d538a7ae
  modified: 2026-11-09T00:00:00.000Z
---

**State (2026-11-09): SHIPPED.** The exact memo (`2278f6d`) plus the user's option "B, extended": each Controls hold
search starts at the end's own throttle (`bh::try_hold_root_from`, step-up on the full search's own `lo + k·GRID`
grid) and the metal cap brackets from above (`bh::try_metal_cap_below`). Four of the five slow page requests 0.3–1.7 s
(were ~13 s); census of 400: no run ↔ refusal flip, worst 3.9e-8 relative. The fifth stays ~13 s: its target lies just
above the loaded engine's floor, so only the floor's 1e-9 K bisection finds it. Record: plan § 43 and
`docs/plans/scratch-archive/gov-slow/README.md`.

**POTENTIAL FOLLOW-UP (user agreed 2026-11-09): a coarser hold tolerance `T_TOL`** (now 1e-9 K; the user: "about 1
kelvin" is physically enough). Fewer halvings at the floor (36 → ~12 at 0.01 K, ~6 at 1 K). It is a RULE-side change:
held answers move by up to the tolerance, near-floor targets can flip running ↔ refused, tests move. Agreed next step:
MEASURE at 0.01 K and 1 K (the fifth request's time, the 400 census, which tests move) and bring the numbers before
changing anything.

**Process lessons:**
- I described the slowness as "governed runs" from where it was found; timing the same requests ungoverned showed
  it is any held run. **How to apply:** before naming who a cost hits, time the neighbouring modes of the same request.
- A memo's byte-identical census proves skipping repeats is safe, not that solves are order-independent; a changed
  refusal's bracketed message looked like history leaking until printed in full (row 2.6e-10 K away). **How to apply:**
  print the hidden digits before calling a changed failure message a leak, or harmless. And promise "identical
  refusals" only for the search's OWN refusals — downstream ones can change their inner message.
- 2026-11-09: aggregate solver counters (HP-wall rescues, closures) pointed at the march; per-end `Instant` timing
  showed the hold end was 9.6 of 10.6 s. **How to apply:** attribute time by timing the phases, not by counters that
  cannot tell the phases apart; I told the user the wrong place before measuring.
- 2026-11-09: a fallback re-runs the full search, so a pre-search that probes OFF that search's grid pays its misses
  twice under an exact-key memo (8.8 s vs unpatched 7.7 s). **How to apply:** make a fast path probe the fallback's own
  points, and time the fallback branch, not only the branch it was written for.
- Dropping a fallback inside a search is a RULE change even when it moves 4 of 400 replies; moving where a search
  starts is a tolerance change; a coarser tolerance is a rule-side change too. Put the kinds apart when asking.
- `cmd start` cannot take this repo's spaced path: from Bash (cwd = repo root) pass RELATIVE paths (`--manifest-path
  rust/Cargo.toml`, `node docs/...`). Related: [[windows-tooling-file-hazards]], [[low-start-repair]],
  [[rung106-governor-lag]].
