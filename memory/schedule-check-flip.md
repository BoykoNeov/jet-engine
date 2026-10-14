---
name: schedule-check-flip
description: "2026-10-14: below_ceiling turned ON by default (user) after a forced census; oracles held to Python with it OFF, ON licensed by a rule — process lessons"
metadata:
  node_type: memory
  type: project
  originSessionId: d4d9844a-c401-4575-b1c7-9f5e6b99946f
  modified: 2026-10-14T15:47:57.257Z
---

2026-10-14, same session as [[schedule-check-census]]. The user chose to flip rung 43's
schedule-check switch ON everywhere after a census showed every move is a march the SOLVER had
stopped, now run on (shared points bit-identical). Record: sandbox plan § 14.5–14.6. Lessons:

- **A total over a run is only comparable at equal run length.** Rung 65 § 3's published table
  (and its gated 1.166 ratio) compared a 341-point run with 251- and 195-point runs the schedule
  check had cut short — wrong even with the switch OFF, and over a common window the ORDER
  INVERTED. A side agent named the pattern ("bigger totals mostly mean longer runs"); check
  `npts` beside any `removed`-style sum before reading a difference.
- **A "what moved" census cannot see code that never ran.** Every 66–73 move came from rung 64's
  fuel-only cell, so the new per-rung branches had zero executions. Tag each fallback search with
  its call chain (a thread-local stack) and COUNT per branch; build a rig for every zero (the
  advisor's blocker → `tests/ceiling_branches.rs`).
- **An "off" probe in a forced copy may not be off.** My first rung-64 off/on probe set the
  carrier, but the copy hard-wired rung 43's read ON, so both columns were ON (a side agent
  caught it). Re-run off/on probes on the UNCHANGED tree, where the knob is real.
- **The forced census hid failures behind the first panic in a program** (slice W died on its
  licence helper before reaching rung 59's reader). And the shared panic hook printed a CAUGHT
  refusal's message for a different real failure — re-run single-threaded before diagnosing
  ([[schedule-check-census]] lesson, recurred).
- **Licence shape that worked:** run each oracle's emitter twice — OFF vs Python (unchanged port
  check), ON vs OFF under `tests/common/ceiling_rule.rs` (no key lost except a superseded sampled
  end point; new keys are trajectory points; changed keys are whole-run readings) with counts
  pinned from the measured flip. The pins equalled the census exactly.
- A reused rung-70 machine started its 6th march off-equilibrium — not traced; gates use a fresh
  machine per march.

Related: [[solver-walls-repair]], [[instrument-fed-by-what-it-certifies]].
