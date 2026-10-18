---
name: rung98-decisions
description: "User decision 2026-10-18 for rung 98 (δ/θ-corrected control tables): a THIRD setting 'tables made at' = design flight (DEFAULT, both corrections on) or this flight (rung 96/97 bit for bit); the two correction switches act under 'design flight'"
metadata:
  node_type: memory
  type: project
  originSessionId: 08fc14b8-3417-46cb-88af-c1084de6a116
  modified: 2026-10-18T07:09:43.877Z
---

Rung 98 (control tables made once at the design flight, readings corrected by δ/θ). Earlier scope answers are in
[[rung97-decisions]] (two switches; all three tables; rows over the run's range translated to the design flight; Mach knob).

**2026-10-18, before the anchor froze:** the advisor proposed a third, master setting "tables made at: this flight /
design flight" so off-design saved runs could still reproduce rung 96/97 (the reduce rule). A side agent flagged that
defaulting it to "this flight" hides rung 98. Asked the user; answer: **default = DESIGN FLIGHT, both corrections on**;
"this flight" stays as the reduce setting. Consequence accepted: saved off-design runs that USE a table (inferred-thrust
governor, accel schedule, HP proportional law) shift; nothing moves at the design flight or without a table.

**How to apply:** gates are "this flight ⇒ rung 97 bit for bit" and "design flight flown at the design flight ⇒ rung 96
bit for bit"; the off-design sandbox check_requests that use a table get new expected behaviour, stated in the plan.

**Status 2026-10-18:** rung 98 BUILT ([[rung98-corrected-tables]]; spec `docs/rung98-spec.md`, plan § 30).
