---
name: back-pressure-repair
description: "Nozzle back-pressure follows the flight (2026-10-16, sandbox plan § 26) — process lessons: a census logger in a multithreaded test process garbles its lines unless each record is one write; the red check = stash rust/src only, keep the new test; a static (M0=0) grid point trips the efficiency cascade, not the repair"
metadata:
  node_type: memory
  type: project
  originSessionId: 76b15153-bd90-4f99-a6aa-01d622d066ba
  modified: 2026-10-16T15:25:52.715Z
---

Repair (not a rung), 2026-10-16: every off-design matcher kept the DESIGN ambient as nozzle back-pressure; field
deleted, every nozzle call reads `flight.p0` (sandbox plan § 26; CLAUDE.md "Solver repairs"). Gate:
`rust/tests/flight_back_pressure.rs` (similarity law at ¼/½/2× p0). Closes the seam [[sandbox-slice3]] found.

**Lesson 1 — a census logger must write each record in ONE call.** `writeln!(file, "{a}\t{b}…")` on a shared
append file issues several writes; the test harness runs tests on many threads in one process, so records came out
interleaved mid-line. Format to a `String` first and `write_all` once (or a file per thread). Still recoverable with
a strict regex over the clean lines — but count from the clean lines only.

**Lesson 2 — rename the field, don't just add a logger.** Renaming the stored field (`p_ambient` →
`census_p_design`) made the compiler list all 31 sites ([[census-by-compiler-not-by-name]]) AND kept the old value
available to log against; delete it after the census.

**Lesson 3 — the red check without hand-reverting:** `git stash push -- rust/src` keeps the new test file (it used
only APIs the fix didn't change), build that one test, `git stash pop`. All 5 went red; 2 by a wrong VALUE, not a
crash — check that at least one red is a value miss, or the test only proves the old code crashed.

**Lesson 4 — choose a similarity grid that avoids M0 = 0** (static: propulsive efficiency undefined, `score`'s
cascade assert fires) and require EACH branch per gas: a π_c 10 engine stays choked down to ~600 K, so the subsonic
points are rung 33's 500–560 K at M 0.85.
