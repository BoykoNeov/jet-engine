---
name: rust-port-slice-ai-preflight
description: Slice AI (rungs 79/80) pre-flight — a zero value-diff is two findings and only a counter splits them; and a static census scoped to engine.py missed four nests in the TEST file the port also ports
metadata:
  node_type: memory
  type: project
  originSessionId: ee285e2a-3c55-4a91-a8f2-79c5d6df3685
  modified: 2026-09-26T17:35:59.149Z
---

Slice AI (rungs 79 + 80, `StateCoordinateTransient` + `SplitWallTransient`) was PRE-REGISTERED on
2026-09-26 off **eleven measured probes**, at
`W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md` § 5.33. The census is 0 ADD, 6 swaps
over 4 names (all of them already table fields), 1 incompatible name reuse (`split_gains`), and two
new core carriers (`phi_ref`, `sm_air`). The Python is 1 037 lines / 23 methods / 530 body lines,
which is AH's class count at 83 % of AH's body. The probes are in
`W:\temp\claude\slice-ai-preflight\`.

**THE LESSON: A ZERO VALUE-DIFF IS TWO FINDINGS, AND ONLY A COUNTER SPLITS THEM.** The
`_with_coord` "value break" was booked to AI back at slice AE. It is this: rung 74's `demand_gains`,
run on a rung-79 machine, dispatches to rung 79's setter. The first probe compared 64 keys against
rung 74, had no counter, and read **0 differ**. That reading says *inert*. The advisor then
required four things: diff against rung 78, snapshot rung 79's class counters, read both fields
inside the scope, and report "reached" separately from "values moved". With those, the rerun gave
a different answer. The scope really does leave both fields wrong: `("clip", "demand")` where
`("demand", "phi")` was meant. Rung 79's branch runs **128 times**, and **0 of 196** values move,
because of two masks already on record. So the booking resolves as *real, reached, value-invisible*.
Its gate has to be a counter plus a field readback, never a value diff.

**THE SECOND LESSON: a static census is scoped by the FILES it opens.** The derived nest census over
`engine.py` found 0 `_b_state`/`_v_state` nests at rungs 79/80. The runtime counter found **4**. All
four are in `tests/test_rung79.py`'s helper `_a_cap`, which holds a guard and calls `_c_at` inside
it. The port ports tests as well as the engine, so the census missed exactly the one file it never
opened. This is [[rust-port-slice-ah-preflight]]'s defect 5 (a scope error deletes its own
evidence) from a new direction. A second case in the same probe set: an AST census of `_with_coord`
CALLS missed a bound-method REFERENCE passed to `_with_probe`, and the runtime wrapper caught it.

**How to apply:** for any "is X reachable / does X move" question, run a counter beside the value
diff and state both results. Scope a static census by the files the port ports, tests included.
Treat the runtime counter as the measurement and the static census as the cross-check.

**Also decided here:**
* Rung 79's six class counters, flag and log become **`thread_local!`**, not statics. The library
  spawns no threads, a static races (AH step 7's lesson), and the crate's own counters are already
  thread-local. Rung 78's statics stay as they are, by a deliberate choice.
* `phi_ref` becomes a `Cell<&'static str>` with an `== "phi"` test, not an enum, because the third
  value `"demand"` is reached.
* One refusal at rung 80 (`engine.py:21486`) is written to catch a wiring bug, but it also fires on
  a legal input: an airflow margin only 1–2 ulp above the fuel margin. The band must reproduce bit
  for bit.

A launch hazard hit this pre-flight: `cmd start` treated the first quoted argument as a window
title. It is recorded in [[windows-tooling-file-hazards]].

See [[rust-port-status]] and [[rust-port-slice-ah-step7]].
