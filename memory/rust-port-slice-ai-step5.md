---
name: rust-port-slice-ai-step5
description: "Slice AI step 5 (the ported rung-79/80 suites + all seven refusals) — a gate read AT the plant's own operating point cannot see a missing state freeze, because the unfrozen re-solve lands on the frozen state there; read a freeze-dependent quantity AWAY from the plant's point"
metadata:
  node_type: memory
  type: project
  originSessionId: 8214f53a-bae3-492b-b0d4-14afd511a423
  modified: 2026-09-28T20:42:17.612Z
---

Slice AI step 5 shipped 2026-09-28 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.33.5): `rust/tests/rung79.rs` (28 gates = 25 ported + 3 declared) and `rust/tests/rung80.rs`
(16 = 12 + 4). All seven rung-79/80 refusals gated; P5's ulp band at `engine.py:21486` holds
(`+1`/`+2` ulp refuse, `+3` builds, PyPy's bits); step 3's J6 discharged by a slack-SUCCESS gate on
`forced_cap` pinned to Python's bits (probe `W:\temp\claude\slice-ai-step5\probe_py.py`).

**THE LESSON: A GATE READ AT THE PLANT'S OWN OPERATING POINT IS BLIND TO A MISSING FREEZE.**
Injection N1 removed `_a_cap`'s outer `(b_state, v_state)` freeze. I predicted two gates would die;
only the margin-SWEEP one did. At `margin = 0` the solved fuel IS the plant's own fuel, and there
the unfrozen valve/stator re-solve lands on exactly the marched state — so
`gap_at_zero_margin_is_exactly_zero` passes either way. At `margin ± 0.01` the states differ.

**Why:** freezing only matters where the trial point differs from the state being frozen; a
reading taken at that state compares the two branches where they coincide.

**How to apply:** when a gate exists to protect a freeze/scope, check it reads somewhere the
unfrozen path would land ELSEWHERE; if the only reading is at the fixed point, add an off-point one.

Also: two suite lines were vacuous in Python — `G.__code__.co_consts is not None` (always true) —
replaced by a dispatch gate PAIRED with rung 79's counters (N8 proved the pairing load-bearing:
bits held, counters caught it). Suite counts must be COLLECTED (`pytest --collect-only`), not read
off the file: I told the user 23/13 before counting 25/12. The advisor also caught a stale
"the spec still says" sentence in `state_coordinate.rs` after the spec had been fixed in `ea43d9c`
— a correction's own write-up goes stale the moment the correction lands; grep for it.
Sweep: 8 injections, 8/8 verdicts, 1 detail wrong. Related: [[rust-port-slice-ai-step4]],
[[rust-port-ported-test-vacuity]].
