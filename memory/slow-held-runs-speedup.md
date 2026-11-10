---
name: slow-held-runs-speedup
description: "Sandbox plan § 43 — slow generator hold runs: memo (2278f6d, 2026-10-22) + hold searches started at their own throttle (shipped 2026-11-09); one request stays ~13 s; FOLLOW-UP DECIDED: T_TOL 0.01 K everywhere, next session (1 K hides rungs 99-101 misses)"
metadata:
  node_type: memory
  type: project
  originSessionId: 2d66d455-34c9-471a-a004-8e54d538a7ae
  modified: 2026-11-10T07:43:30.077Z
---

**State (2026-11-09): SHIPPED.** The exact memo (`2278f6d`) plus the user's option "B, extended": each Controls hold
search starts at the end's own throttle (`bh::try_hold_root_from`, step-up on the full search's own `lo + k·GRID`
grid) and the metal cap brackets from above (`bh::try_metal_cap_below`). Four of the five slow page requests 0.3–1.7 s
(were ~13 s); census of 400: no run ↔ refusal flip, worst 3.9e-8 relative. The fifth stays ~13 s: its target lies just
above the loaded engine's floor, so only the floor's 1e-9 K bisection finds it. Record: plan § 43 and
`docs/plans/scratch-archive/gov-slow/README.md`.

**FOLLOW-UP SHIPPED 2026-11-10** (plan § 43 "DONE"): `T_TOL` = 0.01 K; metal cap kept at relative 1e-9 as
`CAP_TOL_REL`, PENDING the user's answer on whether it goes the same way. Lesson: a golden test that stops at its first
differing panel under-counts what moves — the measurement said 1 table, re-capturing all said 5. **How to apply:**
census a golden change by re-capturing EVERY panel and reading `git diff --stat`, not from the failing test's message.

**FOLLOW-UP DECIDED (user, 2026-11-09): `T_TOL` 1e-9 K -> 0.01 K for all three hold searches, IN A LATER SESSION.**
Measured (plan § 43, `docs/plans/scratch-archive/hold-tol/README.md`): slowest page request 13.3 -> 6.9 s; no run <->
refusal flip; held quantities move <= 2e-5. 1 K was ruled out: rungs 99-101's inferred-thrust misses read EXACTLY 0.
The user chose 0.01 K everywhere over "floor search only" (same speed, runs byte-identical) on principle, not speed:
[[needless-precision]]. The to-do (8 tests' bars, one golden, page, gate; then ASK about the metal cap's relative
1e-9) is in the plan.

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
- 2026-11-09: a 1 K tolerance looked harmless in the census (0.23 %) but zeroed rungs 99-101's measured misses; the
  failing TESTS showed it, the census did not. **How to apply:** before loosening a tolerance, find the smallest
  effect the model is built to show and keep the tolerance below it; read each failing test's printed numbers
  (an exact 0 is a sign the effect was wiped out), and don't count failures as "strict bars" in bulk.
