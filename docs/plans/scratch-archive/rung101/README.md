# Rung 101 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung101-probe`, deleted after the work; its
manifest is `Cargo.toml.txt`). Shared helpers in `common.rs` (rung 100's, unchanged). Every probe reads candidate
beliefs on rung 100's OWN governed run (open loop) — see the anchor's Status.

Before the anchor (`docs/plans/rung101-anchor-airflow-source.md` § 0):
- `airflow_v1.rs` → `airflow.txt` — G1/G3: the four candidate airflows on the pressure ratio (sensor WITHOUT the
  overboard term), start and every 8th march point; plain / cooled / customer share / bleed lever.
- `airflow.rs` (the overboard term added, the overboard variants only) → `airflow_overboard.txt` — G2.
- `fan.rs` → `fan.txt` — G4 (the fan-speed reading under B: rung 100, B's push with the true air, the grid's air, the
  sensor) and G5 (the EPR sensor lagged after the fact, τ 0.25–2).
- `shape.rs` → `shape.txt` — G6: the governor's residual under the sensor over a fuel scan (EPR unlagged and frozen;
  the fan rate law with B fed the sensor).

After the build (the SHIPPED code):
- `smoke.rs` → `smoke.txt` — first closed-loop runs of each source, the sensor gate, against `thrust_by: true`, timing.
- `lagorder.rs` → `lagorder.txt` — the lag's trend gate measured: the true thrust's gap per halving of τ (1.32–1.69).
- `score.rs` → `score_first.txt` (746 / 84 / 50: 19 cells refused while calibrating — the end search from 600 K) and
  `score.txt` (778 / 46 / 56, the search started at the set throttle); the two differ only in those 19 cells' H6/H7 lines.
- `why6.rs` — the 19 refusals' messages; `h8why.rs` → `h8why.txt` — H8's misses are the choked starts.
- `crash101.rs` (rung 99's generator, the three keys on a separate stream, `CRASH_STRIP`, `CRASH_FORMULA`) →
  `crash_summary.txt`, built also against a worktree of `54ee954` (`W:\temp\claude\rung101-base`, deleted);
  `onereq.rs` + `five.jsonl` — the five newly refused requests before the second calibration fix.
