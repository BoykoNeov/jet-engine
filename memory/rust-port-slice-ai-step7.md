---
name: rust-port-slice-ai-step7
description: "Slice AI step 7 (CLOSED) — a call count on a cell says the cell RAN, not that what it built was USED: rung 79's at_lever builds a rung-78 rig at three seats that never dispatch it; and rung 80's at_lever is visible to pointer identity alone, in both languages"
metadata:
  node_type: memory
  type: project
  originSessionId: f93faeff-92b7-4da9-91f1-02870ee45210
  modified: 2026-09-29T04:43:56.170Z
---

Slice AI step 7 shipped 2026-09-29 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.33.7): `rust/tests/slice_ai_dispatch.rs`, 9 gates, 32 s. Every verdict row landed as
pre-registered. Rung 79's three table swaps (`cap_fuel`, `with_coord`, `at_lever`) are ONE deletion
to every seat; rung 80's `at_lever` is silent at all nine seats. SLICE AI CLOSES; P1 refuted
(2 195 lines, the per-LINE mechanism).

**THE LESSON: THERE ARE THREE KINDS OF SILENCE, NOT TWO.** AH taught *ran, no difference* vs
*never entered*, split by a call counter. The third: **entered, built a DIFFERENT machine, never
dispatched it.** Under rung 79's `at_lever → 78` the three gauge-point readers enter `at_lever`,
march a rig carrying rung 78's `cap_fuel`/`with_coord`, and read `same` — because they call the
plant with an explicit coordinate and never touch the rig's table. A nonzero count there would
have been read as redundancy. Only reading the BUILT rig's pointers, plus a zero count on the
cells that differ, says what the silence is.

**Why:** a call counter measures the CONSTRUCTOR, not the consumer. What a constructor builds
matters only if a later reader dispatches through it.

**How to apply:** when a swapped constructor reads `same`, read back the object it built AND count
the cells of that object that actually differ; `same` + entered + those cells never dispatched is
the third kind, and it must not be reported as redundancy.

Also measured:
- Counting SCOPES under-predicts dispatches 2×: a `CoordScope` dispatches on set AND drop.
- The closing mutation (delete `at_lever: r80_at_lever,`) was caught by pointer gates only; I
  predicted the ported `rung80.rs` would fail and it was 16/16 — `test_rung80.py` has no carry gate.
- Rows ran on parallel threads with no lock, on the thread-local counters' guarantee.
- A bare `` `:N` `` citation in a comment that never names `engine.py` is unscanned by the guard.
- `Start-Process .venv\Scripts\pytest.exe` exited 1 with empty logs; `python.exe -m pytest` works.

Related: [[rust-port-slice-ah-step7]], [[rust-port-slice-ai-step6]], [[rust-port-status]].
