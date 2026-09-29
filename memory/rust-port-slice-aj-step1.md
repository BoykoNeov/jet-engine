---
name: rust-port-slice-aj-step1
description: "Slice AJ step 1 (plumbing, rungs 81–84) — a guard written on a reviewer's warning could never be false in MY construction; an injection that deleted it moved nothing, and only a GENERATED oracle (not hand pins) killed the real defect"
metadata:
  node_type: memory
  type: project
  originSessionId: b7dfe779-7bde-417f-97ad-ca94e4876a60
  modified: 2026-09-29T10:58:29.670Z
---

Slice AJ step 1 COMPLETE 2026-09-29 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.34.1 part 1 = the `rung80.rs` pointer assert; § 5.34.1 (cont.) = the rest). Landed: four header-only
modules (`authority_clock.rs`, `threshold_law.rs`, `corrector_law.rs`, `staircase_law.rs` — no
tables, NO builder), `demand_coordinate::py_g` (Python `%g`) gated on a 10 511-row oracle
(`rust/oracle/dump_py_g.py`, PyPy = CPython byte for byte), `shared_actuator::riding4_idx` (the
`id(p)` port; `riding4` now maps over it) gated on Python-probed indices of three rung-81 marches,
items C and D reworded. Probes in `W:\temp\claude\slice-aj-step1\`. Next: step 2 = rung 81's readers.

**THE LESSON: a defensive check copied from a warning must be tested against YOUR construction.**
The advisor warned `%g`'s zero-strip must only touch strings with a point. I added a
`contains('.')` guard; an injection deleting it moved 0 of 10 511 rows, because my builder always
inserts a point. Dead code that reads as protection — removed, with the reason in the comment.

**Why:** a warning describes a hazard of the general problem; whether it can occur depends on the
specific code. A check that cannot fire costs nothing at runtime but misleads every later reader.

**How to apply:** after adding a guard, delete it once and run the gate. If nothing fails, either
the gate is blind or the guard is dead — find out which before keeping it.

Also:
- The real design risk (exponent read before rounding) was killed by 4 generated carry-case rows
  and by NONE of the hand-typed pins. Generate the expected set; include the cases the design exists for.
- 462 exact six-digit ties in the oracle, 231 where half-away would differ — so the tie rule IS tested.
- A citation was wrong at birth again (`22097` for `22096`); the plan's own `22099–22103` is off too.
- A stray `python -` in a chained command blocked on stdin; killed by the PID found through my own
  command's parent chain, leaving another session's Pythons alone. Never put a bare `python -` without a heredoc.

Related: [[rust-port-slice-aj-preflight]], [[rust-port-status]], [[instrument-fed-by-what-it-certifies]].
