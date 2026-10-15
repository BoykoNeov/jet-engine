# Rung 89 scratch scripts

Kept so the givens and the scoring in `docs/rung89-spec.md` can be re-run. A scratch crate (package `r89probe`, path dependency on `rust/`): `rig.rs` is its `src/lib.rs` (rung 88's rig, the 19 scored cells, the lag grid); every other file is a binary in `src/bin/`. The anchor's section 0 givens, all on the SYMMETRIC valve only: `g0.rs` the reduce (`closing(tau)` == symmetric, bit for bit) and a march's wall time; `g1.rs` (arg: ds) each re-tuned valve's minimum against its first closing read, the slack after it, the edge travel and the lag identity; `g2.rs` (arg: ds) the bills' ratio `R` beside the closing-travel ratio; `g3.rs` (arg: ds) the exact one-rise-one-fall count, the phi valve's floor and the forecast `R_f`; `g4.rs` (arg: ds) g3 plus the two bounds on `R` for the close-first rows, each floor's residual and the break-even set point at `tau_close` 0.2.

**The scoring (spec § 3).** `score.rs` (args: ds, then an optional comma list of indices into `CELLS`, so the grid
can run as parallel pieces) re-tunes both valves at every `(tau_open, tau_close)` of every cell, writes one TSV row
per run and prints every H's misses and the tally. It ran as 15 below-normal pieces (5 at ds 0.005, 10 at 0.0025);
`grid_0.005.tsv` / `grid_0.0025.tsv` are the merged rows, `score_0.005.log` / `score_0.0025.log` the concatenated
piece logs (merge the TALLY lines by summing). `awk/` holds the readings made from the TSVs after scoring (run as
`awk -F'\t' -f awk/<name>.awk grid_<ds>.tsv`): `sides.awk` and `flips.awk` (who is cheaper per row and column, and
that every row and column changes sign at most once), `table.awk` (spec § 4's crossover table), `onedge.awk`
(which edge sets each valve's set point), `stats.awk` (the § 3 measured column).

**After scoring.** `d1_prime_steps.rs` — the D1′ residual at five halvings of the step (the test's O(ds²)
envelope); `page_step_close.rs` — the page's step (0.02) against 0.005 for a valve closing at its own speed;
`crash_asym_valve.rs` (args: n, seed) — the Controls crash map with every request closing at its own speed.
