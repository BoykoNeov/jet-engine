---
name: rung101-decisions
description: "User scope decisions 2026-10-19 for rung 101 (rung 100's seam, the airflow off the line): airflow source a CHOICE — rung 100's table (reduce) / speed-grid tables / exhaust-temperature sensor + nozzle flow formula; sensor lag knob default OFF; BOTH readings"
metadata:
  node_type: memory
  type: project
  originSessionId: bf0d7d09-e096-4e3f-8c52-c91f6b04c6b1
  modified: 2026-10-19T00:41:21.209Z
---

Rung 100's seam "the airflow off the line". User picked it 2026-10-19 after `/clear` ("1" from my list).

Pre-anchor probe (`W:\temp\claude\rung101-probe\airflow.txt`, `airflow_overboard.txt`), pressure-ratio reading, exact
push, airflow candidates at the steady start / along the governed move:
- rung 100's carried design table: start up to 55 % (cooled, M1.6 from 700 K), move 2–48 %.
- table made AT THIS FLIGHT (floor): start ~0, move 2–48 % — the move's error is rung 96's accelerating engine, no
  steady table reaches it.
- speed-grid tables' airflow beside the exact push: start ≤ 0.6 % except Mach 0.5 (−0.3…−0.45 %, the 0.3–0.6 gap) and
  tiny-thrust M1.6 cells; move unchanged.
- rung 100's rebalanced row: ~70 % removed at M1.4–1.6, no root when unchoked — dropped.
- exhaust-temperature sensor: ṁ9 from pt9 (EPR·pt2·π_n), Tt5, a8, γ, R; face = (ṁ9 − fuel)/((1−b)(1−c)) — EXACT
  (1e-14) at start AND along the move, every variant; without the overboard term up to −80 %.

**User answers (AskUserQuestion):** "Both, as a choice" (old table = reduce, speed tables, exhaust sensor); "Lag knob,
default off"; "Both readings" (pressure ratio AND fan speed).

**How to apply:** the formula is a non-default speed correction, so default runs should not move. The exhaust sensor
makes the pressure-ratio control know thrust exactly (≈ rung 95's direct reading) — the lag knob brings back a
realistic in-move error. Related: [[rung100-decisions]], [[rung100-below-choke]], [[rung96-inferred-thrust]].

**Second probe** (`W:\temp\claude\rung101-probe\fan.txt`, fan-speed reading under B, + the EPR sensor lagged open-loop):
the speed-table airflow makes the FAN reading's start WORSE (B's push error and airflow error had cancelled — rung
100 lesson 2 again: −4.9 → −6.8 % at Mach 1.4); the exhaust sensor (fed B's own pt5 guess) helps there (−4.9 → −2.7 %,
in-move +55 → +21 %). A lag of only 0.25 HP-τ costs +2…+75 % in-move on EPR; at ≥ 1 τ often worse than the tables.
**User answers:** offer the speed tables on fan speed anyway ("show the result"); lag knob in HP-shaft τ.
**Default (user):** the OLD TABLE (rung 100) — nothing existing moves; the speed tables and the exhaust sensor are opt-in, shown only under the *formula* speed correction.
**Status 2026-10-19:** rung 101 BUILT ([[rung101-airflow-source]]; spec `docs/rung101-spec.md`, plan § 33).
