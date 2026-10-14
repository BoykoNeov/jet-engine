---
name: reused-machine-map-reset
description: 2026-10-14 reused rung-70 machine drift — traced to the live stator map; fixed by a START-only reset because the end-of-march map is an oracle-gated Python behaviour
metadata:
  node_type: memory
  type: project
  originSessionId: bdca5870-ff35-47db-8794-63eea7c33563
  modified: 2026-10-14T17:13:32.268Z
---

A rung-70 machine reused for six marches started march 6 off-equilibrium. Cause: a stator march
writes the live LP/HP maps (`_arm`) and never restores them (Python's shape), and rung 68's `_arm`
returns early once the march's stator position is cleared, so the next march's `s = 0` solve ran on
the last march's map. Fix: `stator_transient::reset_built_maps` at the start of
`FuelTransientCore::stator_march_scoped` (as-built = design + constant setting). Gate:
`rust/tests/reused_machine.rs`. Detail: sandbox plan § 14.6.

**Why:** the reviewer's advice was "restore on exit too, so the engine is clean after a run".
Measured, that moved 19 slice-V oracle keys — every `final_*`/`stale_*` reading of the map a march
LEAVES, which slice V gates ON PURPOSE as Python's persistence. Start-only moved none. The user
chose start-only (2026-10-14).

**How to apply:** before "cleaning up" leftover state in this crate, grep whether a test READS
that leftover (slice V's `stale_*` keys are one) — leftover state here is often a ported Python
behaviour with a gate on it. Fix at the point the leftover does harm (the next march's start),
run the full gate, and ask the user before any oracle key is re-captured. Also: a first sighting's
quoted numbers (0.74745 / 1000.09 K) did not match the re-measured ones (0.747260 / 1000.29 K) —
re-measure on a named setup rather than citing the sighting. See [[schedule-check-flip]],
[[solver-walls-repair]].
