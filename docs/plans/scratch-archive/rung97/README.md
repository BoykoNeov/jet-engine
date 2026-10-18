# Rung 97 and the sandbox § 29 Mach knob — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung97-probe`, deleted after the work).

- `sim.txt` — the pre-anchor similarity probe for rung 98 (δ/θ): pressure scaling exact, temperature scaling not
  (the fuel's fixed heating value); fixed generator power / customer amount break pressure scaling.
- `mach_reach.rs` → `mach_reach.txt` — § 29: the default move over Mach 0.05–3.5 on four maps, after rung 97.
- `mach_choke.rs` → `mach_choke.txt` — § 29: unchoked points per run over Mach and air (default move and from 700 K), after
  rung 97.
- `mach_high.rs` → `mach_high.txt` — § 29: higher throttles at Mach 3 and 3.5 (three maps run from 1300 K; flat-LP never).
- `mach_crash.rs` → `mach_crash29.txt` — § 29: the crash map over Mach 0.05–3.5 (`crash.rs` plus a random Mach; seed 29,
  1 500 requests; the full request is kept only behind the odd stops).
- `mach_default_compare.txt` — § 29: the previous commit's 69 check requests through a worktree of it and through the
  Mach tree; only two refusals differ, in words.
- `choke.rs` → `choke.txt` — unchoked points per run over Mach and flight (§ 28's field; G1 of the rung-97 anchor).
- `mach_rulecheck.rs` → `mach_rulecheck.txt` — § 29: the governor's density rule with the Mach factor, checked at its limits
  (792 runs; measured before rung 97, rerun after it byte-identical. The .rs was archived as a copy of the choke probe by
  mistake and restored from the session transcript on 2026-10-18).
- `g1_root.rs` → `g1_root.txt` — the rung-97 prototype: the subsonic LP-turbine root at 1 667 unchoked instants
  (anchor G3/G4). Prints no speed, thrust, temperature or margin.
- `census.tsv` — anchor G2: a logger (never committed) in the four transient tails, `RUNG97_CENSUS=<file>`, run over
  the whole gate: `binary  test  kind  count` at each power-of-two count. Its code: `r97_census` appended to
  `two_spool_transient.rs` (a `OnceLock` path, a `Mutex<HashMap>` of counts, one `write_all` per record) and, in each
  tail, `.inspect_err(|_| r97_census("abort"))` on the nozzle and `if exit.p9 <= p0 + 1e-6 { r97_census("subsonic") }`.
- `h1_cells.rs` → `h1_cells.txt` — before freezing H1: each 700 K start end's branch and pin-to-root `π` gap (sea level
  +40 / +20 / 0 K, 2 km +40 K; four maps). Same kind of print as G4.

After the build (the SHIPPED branch):
- `score.rs` → `score.txt` — § 3: H1 and H2 against the old model (`lp_pinned`), the named risks, and the reported
  thrust, `Tt4` and flow coefficients. (Written before the crossing print was added; the archived script has it.)
- `gate_margins.txt` — the boundary gate's root-to-pin gap per variant, the Newton's steps either side, the fixed-point
  drift and the start ends' pin-to-root gap; `mass_check.txt` — mass conservation at the nozzle per variant (both from a
  scratch script whose checks are now `tests/rung97.rs`'s).
- `stepcheck.txt` — 0.02 against 0.005 on the unchoked 700 → 1400 K runs, every governor law at its limit, with the
  branch and with the pin side by side.
- `cost.rs` → `cost.txt` — the march's cost with the branch against the pin, best of 15, interleaved.
- `crash.rs` → `crash14.txt`, `crash15.txt`, `crash16.txt` — sandbox § 27's crash map (seeds 14, 15) re-run, and seed 16
  at −1 … 3 km; prints unclassified refusals in full (`UNCLASSIFIED`).
- `rerun.txt` — rungs 95/96's archived scoring scripts re-run against this rung, diffed against their archives.
- `followup.rs` → `followup.txt` — the final review: fuel monotonicity at frozen speeds on unchoked and boundary-crossing
  sweeps (rungs 95/96 keep one root), rung 96's G2 at unchoked instants, and airflow / `pt5/p0` / thrust per air at the H1 ends.
- `crash_compare.txt` — the crash map on this rung and on `36b4908` (a worktree), back to back, diffed request by request
  (`crash.rs` now prints one `OUT` line per request); `miss107.txt` — seed 16 #107 at 940–1100 K: the LP-balanced curve
  scanned for an HP balance, pin against branch (the pin finds a steady point at 940 K; the branch has none).
