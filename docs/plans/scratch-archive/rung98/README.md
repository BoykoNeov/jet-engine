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

After the build (the SHIPPED code):
- `score.rs` → `score.txt` — § 3 scored (H1–H4) and § 2's pressure-off sign gate, plus the both-corrections residual.
- `h4why.rs` → `h4why.txt` — H4's MISS: the proportional law's demand against its carried table's rows (the uncorrected
  reading runs off the top row in hot air) and the steady-fuel exponent `k`.
- `margins.rs` → `margins.txt` — the gates' measured margins, four maps × seven variants.
- `report.rs` → `report.txt` — the reported quantities: fixed-amount variants, the thrust gap through the move.
- `cost.rs` → `cost.txt` — design-flight against this-flight tables, best of 9.
- `crash98.rs` → `crash_compare.txt` — the crash map (seed 98, 1 500) on this tree and a worktree of `897001b`, request
  by request; `classify.py` checks every differing request reads a table off the design flight with tables made there.
- `census.rs` → `census.txt` — the Controls check requests through both builds (one moved: index 69).
