---
name: hp-wall-repair
description: "2026-10-19 solver repair (sandbox plan § 36): rung 40's two-shaft closure capped at the HP map's own wall where its bracket failed — process lessons: seed at the true root to test a 'bad start' hypothesis; a finite non-physical branch slips past a NaN guard; user chose ON everywhere with 4 Python march lengths licensed by a RULE"
metadata:
  node_type: memory
  type: project
  originSessionId: cd29b06a-e475-4741-8e63-7c477860dce7
  modified: 2026-10-19T18:02:46.505Z
---

Repair (not a rung), 2026-10-19: rung 40's equilibrium refused running engines (flat-LP map, low throttle; 11 km top of
range on curved maps). Fix: `two_spool_transient::hp_wall_rescue` in the four Tt4 closures (40/62/90/91), only on the
failing branch. Gate `rust/tests/hp_wall_repair.rs`. User decision: **on everywhere** (not a switch), 4 Python-held
`slice_v` march lengths licensed by a rule. The Newton's full-speed START just above idle: closed by [[low-start-repair]].

**Lesson 1 — test a "bad starting guess" hypothesis by seeding AT the true root.** Rung 103's notes proposed "start
from the steady matcher"; one probe seeding the Newton at the steady root still failed, which moved the search from
the solver's path to the closure itself in one step. Cheap, decisive — do it first.

**Lesson 2 — a NaN guard does not catch a FINITE non-physical branch.** Past the HP map's wall `Tt3 < 0` but the gas
tables return finite numbers, so the residual flips sign silently far from the root. Scan the residual over the whole
bracket (`gscan`) before trusting a "does not bracket" message.

**Lesson 3 — a wall on one side only.** The search was bounded by the LP map's wall; a flat LP map has none, and the
HP side was never bounded. When a bound comes from one component, ask what bounds the next one downstream.

**Lesson 4 — census by running the whole gate with the fix on, then ask.** The advisor predicted Err-as-signal callers
would move; the gate found exactly 4 Python keys (march lengths that now finish) + tests pinning the old stop. That
was the user's call, asked with plain consequences. Related: [[solver-walls-repair]], [[back-pressure-repair]],
[[rung103-steady-unchoked]].

**Hazard hit again:** a stray `python -` in a bash line hung (the Store stub) — see [[windows-tooling-file-hazards]];
and `cmd //c` cannot take a manifest path with a space (`jet engine`) — launch cargo via PowerShell `Start-Process` with
`PriorityClass = 'BelowNormal'` instead — WRONG: setting it after launch misses child processes; the right form is in
[[low-start-repair]].
