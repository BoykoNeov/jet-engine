# The hold searches' tolerance — scratch instruments (2026-11-09)

`docs/plans/sandbox-plan.md` § 43's follow-up: how coarse may `burner_hold::T_TOL` (1e-9 K) be? Measured on a git
worktree of `5e0ebe0` at `W:\temp\claude\hold-tol\wt` (deleted afterwards), with the metal cap's relative tolerance
split out and held at 1e-9 (`CAP_TOL_REL`) so only `T_TOL` moved. A scratch crate depended on the worktree's
`turbojet` by path (manifest `Cargo.toml.txt`, thin LTO); the census binary is `../gov-slow/bytes.rs` unchanged.

- `slow.rs` — times each request line of a file natively (`slow <reqs>`), 64 MB stack thread per request.
- `slow_reqs.txt` — the five slow requests (copy of `../rung106/r95time_reqs.txt`). Page times: `../gov-slow/wasm_time.mjs`.
- `split.py A B` — census split: ran ↔ refused flips, refusals with moved numbers / changed words, runs with moved numbers.
- `miss.py A B` — per run: change of each end's held quantity relative to its target, of each end's `Tt4` (K), and
  the worst relative change of any other number (|x| ≥ 1e-3, `settled_*` residuals excluded).
- `refnum.py A B` — the largest moved number inside each changed refusal message.

## Results (census: 400 requests, seed 31, 264 ran in every variant)

| Variant | Slowest request, page / native | Census against 1e-9 | Gate (`test-all.ps1`) |
|---|---|---|---|
| today, 1e-9 K | 13.3 s / 7.9 s | — | green |
| floor search only at 0.01 K | 6.8 s / 4.0 s | runs IDENTICAL; 20 refusals' quoted numbers move (worst 138.59 → 139.35 N/(kg/s), the least thrust at the floor — steep there) | 1 fails: `cli_golden` (`print_burner_hold_table`: floor 973.99 → 974.00 K, least thrust 142.6912 → 142.6946) |
| all three searches at 0.01 K (**the user's choice**) | 6.9 s / 4.0 s | 0 flips; 164 runs move: held quantity ≤ 2.0e-5 of its target, end `Tt4` ≤ 0.003 K; same 20 refusals | 8 fail (below) |
| all three at 1 K | 4.8 s / 2.7 s | 0 flips; held quantity ≤ 0.23 % of target, `Tt4` ≤ 0.39 K | 14 fail, some REAL (below) |

The whole gain is the floor search (`floor_of`): below the floor every solve fails, slowly; the other two searches
(`try_hold_root*`, rung 101's `sensed_end_root`) cost nothing measurable.

**At 0.01 K the 8 failures are bars set to the old precision** (misses ~1e-5 relative against 1e-9 / 1e-6 bars):
`rung94` — `the_root_finder_refuses_above_its_range_and_the_cap_stops_at_the_limit`,
`the_searches_from_the_set_throttle_land_as_the_full_ones_without_asking_below_the_floor` (land at 1234.0027 against
1e-8), `every_hold_meets_its_target` (732.1787 vs 732.1753); `rung95` `the_governed_run_settles_on_the_end_and_holds_its_demand`
(line 70, thrust within 1e-6); `rung96` `the_inferred_run_settles_on_its_end_and_holds_its_reading`; `sandbox_hold`
`a_thrust_hold_meets_the_unloaded_engines_thrust_at_both_ends` (1e-9); the browser check
`holding_thrust_runs_the_ends_hotter_and_meets_the_unloaded_thrust` (miss 9.7e-6); `cli_golden` (71.68 → 71.67).

**At 1 K some failures are real:** rungs 99–101's start misses (the inferred-thrust error, tenths of a percent, a
fraction of a kelvin of `Tt4`) read EXACTLY 0 (`rung99` h4, `rung100` h2_h3, `rung101` h1/h6_h7/h8) — the search
resolves less than the effect those rungs show; rungs 95/96's governed runs no longer settle on their drawn end;
`rung94`'s zero-load slope (finite differences over a 0.01 load) is off by ~6 %. A tolerance must stay finer than
the smallest effect the model is built to show.
