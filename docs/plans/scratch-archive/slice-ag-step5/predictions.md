# Slice AG step 5 — injection sweep, PREDICTIONS TYPED BEFORE ANY RUN

Instrument: patch `rust/src/sensed_cap.rs`, rebuild, re-run `slice_ag_step5_drive`, diff its
**1 814** keys against the PyPy golden. Source SHA-256 verified back to pristine after every run.

Baseline: `Rust == PyPy` on all 1 814 keys, both key sets equal, first compiling run, no port fix.

Two grids are driven, and the second exists because the first cannot SEE two branches:

* **margin 0.10** — 10 of 10 rows bind (`n_inert = 0` in every cell), every live cell is
  FUEL-authoritative, 4 of 8 cells EMPTY.
* **margin 0.20** — 7 of 9 rows bind, all 8 cells LIVE, the `gov` cells populated, so
  `row_err`'s `Some` arm and the `sched`/`applied` split inside it are both evaluated.

| # | injection | PREDICTED |
|---|---|---|
| 1 | `cap_rows` passes `None` to `rhs_gains_at` (rung 75's own call site) | **KILLED by VALUE, large — and NOT by panic.** The readers' path never enters `integrate_fuel`, so its third refusal cannot fire; with `accel = None` the `if let Some(accel)` guard in `r74_cap_fuel` skips the sensed branch and both arms agree. If it PANICS, step 4 § (a)'s domain sentence is wrong again in the other direction. |
| 2 | `cap_rows` drops `lag_coord.set(DEMAND)` | **KILLED by PANIC**, at the `track` cells — rung 75's `windup_tau` refuses `track` outside the plain demand coordinate (§ 5.31.3 (a), one rung on). The `none` cells would be silent. |
| 3 | `accel_binds` folds `min` instead of `max` | **KILLED, margin 0.20 ONLY.** At 0.10 every row binds under both laws, so the fold is inert there — the same "a gate that picks one cell scores the bug as correct" shape step 4 § (b) hit on a law. |
| 4 | the SENSED cap call drops `mf_app` | **KILLED by PANIC** — `r76_sensed_cap`'s own refusal (*a `None` here is a THREADING bug*). |
| 5 | the PHI cap call passes `Some(accel)` | **KILLED, large** — `cap_phi` becomes the min of both legs, and `accel_binds` follows. |
| 6 | `tau_auth` / `tau_masked` SWAPPED | **SURVIVES.** `taus[1] = 0.05` and the lag returns its ATTACK clock, also `0.05`, on a ramp — so the two are the same number and the distinction is undrivable on this grid. If it is KILLED, the lag releases somewhere and the row detail is sharper than expected. |
| 7 | the two scopes NEST the other way round | **SURVIVES** — `WindupScope` and `CapScope` write disjoint fields, so neither restore can observe the other. |
| 8 | `auth_err` targets `(c+1)/tau_auth` | **KILLED** |
| 9 | `masked_moved`'s conditional collapsed to the relative form only | **SURVIVES** — the `else` arm needs `|masked_diag0| <= 1e-30`, which a live 4×4 diagonal never is. A defence with no reader. |
| 10 | `row_err`'s two targets INVERTED (`sched` ↔ `applied`) | **KILLED, margin 0.20 ONLY** — at 0.10 `row_err` is `None` in every live cell. |
| 11 | `det_err` scores against `(1 + c)` | **KILLED** |
| 12 | `n_inert` counts the BINDING rows | **KILLED at both margins** — 10 instead of 0 at 0.10, and the complement at 0.20. |
| 13 | `s_tail` folds `min(taus)` | **SURVIVES** — all four clocks are `0.05` on the shipped grid, so `min == max`. A second defence with no reader. |
| 14 | `fuel_int` folds RIGHT to LEFT | **KILLED, exactly 4 keys** — two arms × two bill cells, and nothing else reads a sum. This is **P2**'s own question asked of the port. |
| 15 | `fixed_point` reads `solve(q)` instead of `sensed(q, w0)` | **KILLED, small.** But if identity (1) holds to the LAST BIT, `solve(q) - w0` is exactly `0.0` and so is the faithful expression — in which case it SURVIVES, and a step-6 gate on identity (1) would be comparing the solve with itself. |
| 16 | `predicted = 1/(1 + c)` | **KILLED** |
