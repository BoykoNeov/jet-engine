---
name: rust-port-slice-ai-step6
description: "Slice AI step 6 (the oracle for rungs 79/80) — where a corrupted value can reach an output is a question only an injection answers; both of the step's REACH claims (counters catch item L; the incidence arm drives walls_of's round trip) were false, and each was found only by injecting into the thing claimed to be reached"
metadata:
  node_type: memory
  type: project
  originSessionId: ccc6a410-42f4-4cbb-9478-853d5ab07e32
  modified: 2026-09-28T23:55:42.385Z
---

Slice AI step 6 shipped 2026-09-29 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.33.6): `rust/oracle/dump_slice_ai.py` + `rust/tests/slice_ai_oracle.rs`, 37 945 keys, every rung-79/80
reader on BOTH stator arms, rung 74's `demand_gains` on R78/R79/R80 with rung 79's counter deltas, both
plants. Rust ≡ PyPy on every key, first run. CPython: 227 differ, ALL on the 340 keys of the two float
`sum()` paths a runtime census found before the golden was opened (`_charpoly4`, `split_gains`' `rate`);
no `zeros` count moved. P3 measured by marching on two threads at once (a single drive cannot test it).

**THE LESSON: WHERE A VALUE CAN REACH AN OUTPUT IS MEASURED BY INJECTING INTO IT, NOT BY REASONING.**
Two reach claims, both false:
1. Item L's survivor (`CoordScope::drop` writing `lag_coord` directly) — predicted in writing to move
   the counters. Moved **0 of 37 945 keys**; only `slice_ai_cells.rs`'s field readback caught it. The stuck
   `phi_ref` sits on `demand_gains`' own built rig and the next scope overwrites it before anything reads it.
2. My dumper header said the incidence arm drives `_walls_of`'s round trip (and `split_wall.rs`'s doc
   says it decides `engine.py:21486`'s band). Injected `999.0`: **0 keys, no suite gate**. Every rig arms
   the valve, whose wall `phi_air` reads first. Dead on every shipped path; three texts corrected.

**Why:** "the arm is swept" and "the counter exists" are claims about INPUTS. Whether a fault reaches an
output depends on a READER downstream of it, which is exactly what reasoning skips.

**How to apply:** before writing "X drives/catches Y" in a header or prediction, inject a blatant fault
into Y (999.0, a direct write) and count keys. Zero keys means the sentence is false, whatever the sweep.

Also: the pre-flight's counts (128 / 80) were taken at `every = 4` though its probe script defaulted to
16 — check which setting produced a number before registering it as a prediction. PowerShell `Start-Process -PassThru` returns a blank
`ExitCode` unless `$null = $p.Handle` is read right after start. Related: [[rust-port-slice-ai-step5]],
[[instrument-fed-by-what-it-certifies]].
