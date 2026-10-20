---
name: idle-floor
description: "2026-10-19 page rule (sandbox plan § 38): ONE below-idle rule in the pages via TwoSpoolCore::idle_floor (off in old code) + the customer-bleed 'off the maps' sentence — lessons: a test's premise label ('just above idle') was never checked against the quantity the rule judges; JSON-escaped replies foil a text diff; typed `python` again (Store stub hang)"
metadata:
  node_type: memory
  type: project
  originSessionId: 516d97f0-e1bb-4745-a47b-1809cca9649e
  modified: 2026-10-20T00:46:25.730Z
---

**UPDATE 2026-10-20 (§ 39, user decision on a side agent's heads-up): the Controls page WARNS instead.**
`TwoSpoolCore::idle_rule: IdleRule { Old (default, old code), Refuse (Blades/Cooling), Accept (Controls — keeps a
below-idle root from either start; reply's start/end carry `below_idle` = thrust per inlet air; page note) }`.
Controls hidden setting `idle`: warn/refuse/old; rungs 98/99 tests use `old`. Found: the march's fuel closure stops
slams that start low ("other" stop, 1 691 of 1 854 at the first step) — whole-slam floors are 20–80 K above the
build floors; OPEN, offered as the next repair. Below text is § 38's (refuse everywhere), kept as history.

Page rule (not a rung), 2026-10-19, closing [[low-start-repair]]'s open item. User said "ok do it" to my
recommendation: probe the 3 leftover "off the maps" refusals first, then the consistent-refusal switch (not
"show negative thrust with a warning" — the shared `score` is not built for it; still a possible later rung).

- The 3 refusals were REAL (no start solves; the load walks the LP shaft to a stop). Only the customer-bleed one
  lacked a "with X off this throttle runs" sentence — added, only on "off the maps", only if the generator's didn't fire.
- `idle_floor` refuses in rung 40's equilibrium (first Newton's root, thrust per inlet air ≤ 0) and in
  `steady_unchoked::dispatch` (outer call only, choked points). Pages set it via `Machine::for_pages`,
  `FixedEngine::for_pages`, and one `make` closure in `sandbox_controls::build`. Gate `rust/tests/idle_floor.rs`.
- Safe because unique: 1 022 full-speed below-idle roots, 48 starts each, no second root.
- Crash map: 24 runs now refused, 23 with the BLEED LEVER (dumped air pays full ram drag); Mach 2.6 floor 960 K
  (was 800–820, below idle); bleed lever at Mach 2.6 still below idle at 1 100 K.

**Lesson 1 — a test's premise label is a claim nobody checked.** `low_start_repair.rs` said "the Controls page
from just above idle" and started the bleed lever and customer bleed at 560 K — both below idle. The floor made
it fail; that failure was the finding, not a regression. When a new rule breaks a test, ask whether the test's
premise was ever measured before "fixing" the rule.

**Lesson 1b — a crash map measures only the knobs it draws.** I reported "24 runs refused" from the HP-wall map
(Mach 0.2–1.6, no governor/tables); the advisor flagged it, and rung 101's map (every knob, Mach to 3.5) showed
178 of 1 500 — 149 at Mach ≥ 2 — plus the gate failing in rungs 98/99 (built ON the page, anchored below idle;
fixed with a hidden `idle_floor` reduce setting, not by moving their cells). Pick the crash map that draws the knobs
the change reaches, and state the slice a count covers.

**Lesson 1c — "it builds" is not "it runs".** I quoted the page's lowest starts from `build()` (the steady ends
only); a side agent noticed the slam itself stopped from 480–540 K at the design flight. Measure the product's
whole action (`controls()`, march length == expected) before quoting a floor.

**Lesson 1d — a quoted number may be one part of a sum.** A side agent read rung 98's Mach-3 cell's −4 441 N per
kg/s as the belief's miss; it was the gap's RAM part, which the nozzle part (+4 724) cancels — the miss was +283 → −38.
I nearly wrote the old refusal's reason from memory too ("a read outside the rows"); the code said "a row could not be
built". Before answering a number, read what it sums with; before writing a mechanism, read the code that fires it.

**Lesson 2 — compare page replies structurally.** The replies are JSON with `—` escapes; a sed diff on the
plain-text sentence matched nothing and reported every change as "OTHER". Match on an ASCII-safe pattern.

**Lesson 3 — never type `python` here** (already in [[windows-tooling-file-hazards]]): it hung 120 s; stop it
with TaskStop by its task ID, then use Edit.

Related: [[rung103-steady-unchoked]], [[hp-wall-repair]], [[rung103-decisions]].
