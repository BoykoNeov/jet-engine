# Rung 99 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung99-probe`, deleted after the work;
its manifest is `Cargo.toml.txt`). Shared helpers in `common.rs` (rung 98's, unchanged).

Before the anchor (`docs/plans/rung99-anchor-mach-axis.md` § 0):
- `reach.rs` → `reach.txt` (grid to Mach 2.0) and `reach_to3.txt` (`GRID=0.3,…,3.0`) — G1: do the two bracketing grid
  slices build and rise for 162 flights × 3 moves × 4 maps; unchoked-row counts. Rung 98's single table's reach beside
  it. Prints counts and failing flights only. The first run is why the grid runs to 3.0 (user's choice).
- `sim.rs` → `sim.txt` — (a) G2: non-thrust corrected columns between slices at equal corrected throttle, choked and
  unchoked, every map and variant (the thrust column not printed); (b) where rung 98's split identity fails (counts,
  spreads).
- `cells.rs` → `cells.txt` — G3/G4: the scoring cells (Mach 0.05/0.15/0.5/1.4 at 250 K, 50 kPa) — ends, slices,
  design rows build; the formula's nozzle constant against `base.a8·(δ·p0_d − p0)`.
- `bend.rs` → `bend.txt` — after the advisor's pre-freeze review: G5, the grid's interpolation error at a fixed
  corrected engine computed analytically from state, `a8` and the inlet; G6, the formula's unchoked nozzle term and the
  airflow-guess bound at the 700 K start. No table built or read.
