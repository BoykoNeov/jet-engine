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
