# Rung 100 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung100-probe`, deleted after the work; its
manifest is `Cargo.toml.txt`). Shared helpers in `common.rs` (rung 98/99's, unchanged). The grid probes also ran
against a worktree of `f041d2d` whose `mach_axis::GRID` alone was edited (`W:\temp\claude\rung100-grid`, deleted).

Before the anchor (`docs/plans/rung100-anchor-unchoked-formula.md` § 0):
- `givens.rs` → `givens.txt` — G1 (γ, R constant), G2 (push identity, relative), G3 (rung 99's formula vs the push),
  G4 first look (approximation A at unscored flights).
- `fan2.rs` → `fan2.txt` — G4: approximations A and B against the truth, six variants, unscored flights.
- `why.rs` → `why.txt` — G5: the true unchoked engine vs the choked row at the same corrected burner temperature;
  the LP re-solve from the TRUE HP side reproduces thrust (closure).
- `gridlow.rs` → `gridlow_shipped.txt` (shipped grid + rung 99 formula), `gridlow_low005.txt` (grid + Mach 0.05),
  `gridlow_low015.txt` (grid + Mach 0.05 and 0.15); `join.sh` → `gridlow_join.txt` / `gridlow_join2.txt` — G6.
- `reach.rs` (rung 99's, `GRID=0.05,0.15,0.3,…,3.0`) → `reach_low.txt` — G7.
- `eprguess.rs` → `eprguess.txt` — G8: the exact pressure-ratio push's leftover (the airflow guess) at unscored flights.
- `bars.rs` → `bars.txt` — G2's absolute worst; G9 (B vs rung 99 where choked).

After the build (the SHIPPED code; `crash99.rs` / `census.rs` are rung 99's, unchanged, built also against a clean
worktree of `f041d2d` at `W:\temp\claude\rung100-base`, deleted):
- `score.rs` → `score.txt` — § 3 scored: 242 HIT, 35 MISS, 40 VACUOUS of 317, each miss printed.
- `smoke.rs` → `smoke.txt` — first runs of both formulas (new vs `formula_choked`), timing, the push gate's rest.
- `h4why.rs` → `h4why.txt` — H4's miss split: the rest ~1e−13, the airflow guess the whole −1.7 %; Mach 1.2 / 1.6.
- `genlp.rs` → `genlp.txt` — H1's 28 misses: the LP generator's fan-speed miss with tables made at this flight and at
  the design flight (rung 96's clean table), not the speed axis.
- `findref.rs` → `findref.txt` — the fan-speed formula's refusals (4 of 72 low-throttle sea-level requests).
- `cost.rs` → `cost.txt` — rung 100 against its reduce settings, best of 9.
- `census.rs` → `census_new.txt` / `census_base.txt` — the 91 check requests on both builds: 3 moved (85, 86, 87).
- `crash99.rs` (seed 99, 1 500) → `crash_diff.txt` (13 differing) and `crash_classified.txt` (each in class A, grid
  below Mach 0.3, or B, formula with an inferred thrust; none OUTSIDE).
