# Rung 97 (and the parked sandbox § 28 Mach knob) — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung97-probe`, deleted after the work).

- `sim.txt` — the pre-anchor similarity probe for rung 98 (δ/θ): pressure scaling exact, temperature scaling not
  (the fuel's fixed heating value); fixed generator power / customer amount break pressure scaling.
- `mach_reach.txt` — the default move over Mach 0.05–2.2 on four maps (§ 28, needs the stashed Mach field).
- `choke.rs` → `choke.txt` — unchoked points per run over Mach and flight (§ 28's field; G1 of the rung-97 anchor).
- `mach_rulecheck.rs` → `mach_rulecheck.txt` — the governor's density rule with the Mach factor, checked at its limits
  (792 runs).
- `g1_root.rs` → `g1_root.txt` — the rung-97 prototype: the subsonic LP-turbine root at 1 667 unchoked instants
  (anchor G3/G4). Prints no speed, thrust, temperature or margin.
- `census.tsv` — anchor G2: a logger (never committed) in the four transient tails, `RUNG97_CENSUS=<file>`, run over
  the whole gate: `binary  test  kind  count` at each power-of-two count. Its code: `r97_census` appended to
  `two_spool_transient.rs` (a `OnceLock` path, a `Mutex<HashMap>` of counts, one `write_all` per record) and, in each
  tail, `.inspect_err(|_| r97_census("abort"))` on the nozzle and `if exit.p9 <= p0 + 1e-6 { r97_census("subsonic") }`.
- `h1_cells.rs` → `h1_cells.txt` — before freezing H1: each 700 K start end's branch and pin-to-root `π` gap (sea level
  +40 / +20 / 0 K, 2 km +40 K; four maps). Same kind of print as G4.
