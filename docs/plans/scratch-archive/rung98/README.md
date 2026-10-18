# Rung 98 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung98-probe`, deleted after the work;
its manifest is `Cargo.toml.txt`). Shared helpers in `common.rs`.

Before the anchor (`docs/plans/rung98-anchor-corrected-tables.md` § 0):
- `sim.rs` → `sim.txt` — G1 pressure similarity (six non-power-of-two `p0`), G2 temperature residual at the
  corrected-equivalent throttle, G3 (printed as `G4` rows) Mach at fixed inlet totals, core quantities only. Every map,
  ten variants. Prints corrected STATE quantities, never a table read.
- `reach.rs` → `reach.txt` — G4: do the 13 design-flight rows at `Tt4·Tt2_d/Tt2` solve and rise, 90 flights × 3 moves ×
  4 maps. Prints counts and failing flights only.
- `cells.rs` → `cells.txt` — G5: the scoring flights' own ends solve, choked (y/n only).
- `cells2.rs` → `cells2.txt` — after the advisor's review: (a) the scoring cells' design-flight rows solve, choked, and
  rise (y/n); (b) G6, the cross-flight thrust split's SPREAD over each run's throttles, every variant (value not printed).
