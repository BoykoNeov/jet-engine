# Slice AI step 7 — predictions, written BEFORE the first run of `slice_ai_dispatch.rs`

Rig: the suites'/oracle's `rig()` — valve + PHI stator at `sm = 0.80/0.55 - 1`, knobs
(demand, sched, none, None, solve). ds = 0.005, no coarsening. Phi stator arm only.

Seats: R79 rows -> [coord_scan, coord_census, coord_march, coord_forced, demand_gains];
R80 rows -> those five + [split_liveness, split_arrest, split_saturation, split_gains(clip)].
demand_gains at section I's settings: wall 0.80, every 4.

Reasoning rule: a swap can only move a seat whose code DISPATCHES the swapped cell (or builds a rig
through it AND then dispatches a cell of that rig). coord_scan / census / forced call phi_cap /
phi_residual / forced_cap with an EXPLICIT coordinate and march CLIP (ledger_march) — no cap_fuel,
no with_coord dispatch.

## R79 verdict rows (5 seats)

| row | scan | census | march | forced | demand_gains (values) |
|---|---|---|---|---|---|
| AtLever -> R78 | same | same | DIFF | same | same |
| SharedRig -> R78 | same | same | same | same | same |
| CapFuel -> R78 | same | same | DIFF | same | same |
| WithCoord -> R74/78 | same | same | DIFF | same | same |

* The three coord_march DIFFs are ONE IDENTICAL STRING (a phi march compared with a phi march,
  empty log, every counter 0). Consistent with step 5's N2 (hits = 0 on march readers).
* SharedRig is REDUNDANCY: the injected at_lever already carried phi_ref.
* AtLever at scan/census/forced: a THIRD silence — entered, builds a genuinely different rig
  (R78 tables), whose differing cells are never dispatched at those seats.

## R79 coord counters per seat (vs shipped)

* shipped: demand_gains hits 128, fb_inc = calls_inc (pre-flight) — seat-on-rig check, STOP if not.
* shipped coord_march: hits 1366, fb_inc 1363 (br_inc 3) — step 3's numbers.
* AtLever / CapFuel / WithCoord: coord_march all-zero counters; demand_gains hits 0 (all six 0).
* SharedRig: counters identical to shipped at every seat.
* scan/census/forced: counters identical to shipped on ALL four rows (their phi_cap calls are
  explicit, table-free).
* A demand_gains VALUE move on any R79 row = a new finding against P2b's masks, not to explain away.

## R79 Count row (zero / nonzero only; exact numbers pinned from the run)

| cell | scan | census | march | forced | demand_gains |
|---|---|---|---|---|---|
| at_lever | >0 | >0 | >0 | >0 | >0 |
| shared_rig | >0 | >0 | >0 | >0 | >0 |
| cap_fuel | 0 | 0 | >0 | 0 | >0 |
| with_coord | 0 | 0 | >0 (4) | 0 | >0 (~16 = interior cells) |

## R80 verdict rows (9 seats)

| row | 4 x r79 readers | demand_gains | liveness | arrest | saturation | gains |
|---|---|---|---|---|---|---|
| AtLever -> R79 | same | same | same | same | same | same |
| SharedRig -> R79 | same | same | DIFF | DIFF | DIFF | DIFF |

* AtLever all-same is REDUNDANCY: r80_shared_rig re-reads sm_air off the CORE and splits the
  R79-table rig; no reader dispatches that rig's shared_rig again. Mechanism gate reads it back.
* SharedRig: the split never reaches the plant; sm_air=None elsewhere so rung-79 readers +
  demand_gains are exact dispatch. Split seats DIFF, none BROKE (no refusal reads walls on R79).
* R80 counters: every row identical to R80 shipped, except none predicted to differ.
* R80 shipped demand_gains: hits 128 as R79 (pre-flight: "the same").

## R80 Count row: at_lever and shared_rig >0 at all 9 seats; cap_fuel/with_coord pattern as R79's
on the first five seats; on the four split seats cap_fuel >0 (demand marches) except where a seat
marches only clip; with_coord 0 at split seats (rung 74's coord_march assigns lag_coord by plain
assignment).

## Source mutation (closing): delete `at_lever: r80_at_lever,` in R80

Compiles (..R79 fills it) with a dead-code WARNING for r80_at_lever. Caught by: slice_ai_cells
pointer gate + carry gate (sm_air), rung80.rs isinstance/carry gates if any, slice_ai_dispatch
install proof. slice_ai_oracle: 0 keys moved (predicted from the AtLever row's all-same).

## Source mutation M1 — delete `at_lever: r80_at_lever,` from `R80` (split_wall.rs), written before the run

Compiles, with a dead-code warning for `r80_at_lever`. Binaries, --no-fail-fast:
* slice_ai_cells: FAIL — swap 5/6 pointer assert; carry gate (sibling sm_air None, not 0.5).
* rung80: FAIL — whatever gate ports the `isinstance`/carry of at_lever (expect >=1).
* slice_ai_dispatch: FAIL — each_injection (two pointer asserts), rebuild helper (sm_air knob),
  matrix install proof (R80 AtLever `!= own_at`) => every matrix gate; mechanism gate (rig
  shared_rig is R79's).
* slice_ai_oracle: ALL GREEN, 0 of 37 945 keys moved (the AtLever row's all-same, incl. counters).
* slice_ai_split, slice_ai_scan, slice_ai_march, rung79: green.

## M1 MEASURED
slice_ai_cells 8/10 (the two predicted), slice_ai_dispatch 0/9 (all predicted), oracle 6/6 (0 keys),
split/scan/march/rung79 green, dead-code warning present. **rung80 16/16 GREEN — PREDICTION WRONG**:
test_rung80.py has no at_lever/isinstance gate at all (grep), so the ported suite is faithfully blind.
Restored by `git checkout`, hash f18dcef1… verified.

## Full gates, predicted before the run
cargo test --release --no-fail-fast: 172 result blocks / 1 776 passed / 0 failed (step 6's 171/1 767 + 1 binary, 9 tests).
pytest: 1 387 passed.
