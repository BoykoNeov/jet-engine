# Slice AI step 6 — injection predictions, written BEFORE any injected run

## L1 — item L's survivor: `CoordScope::drop` writes `lag_coord` DIRECTLY
`(self.core.triple_hooks.with_coord)(self.core, self.prev)` -> `self.core.lag_coord.set(self.prev)`.

On an R79/R80 machine the guard's `set` dispatches to rung 79 (writes `phi_ref`, returns the displaced
`phi_ref`), and the defective drop writes that into `lag_coord`. So `phi_ref` is never restored.

Predicted, on `slice_ai_oracle.rs`:
- **Section I, r79 and r80, both walls, both arms**: `demand_gains`' FIRST sample point leaves the
  built rig at `phi_ref = "demand"`, `lag_coord = "phi"`. Every LATER `_quad_gains_at` entry on that rig
  then runs rung 79's incidence branch, so `ctr/hits`, `ctr/binds`, `ctr/fb_inc`, `ctr/calls_inc` go
  UP (above 128 / 80). `fb_phi`, `calls_phi` stay 0. **No `/g/` value key moves.**
- **Section I, r78**: unchanged (rung 74's `with_coord` IS `lag_coord`, so the direct write is right there).
- **Sections C and P/coord**: the four scopes are on the CORE. `phi_ref` gets stuck at whatever the last
  scope set, and `lag_coord` on the core picks up a `phi_ref` value. `cap_march` passes its coordinate
  explicitly, so I predict NO value change and NO counter change there (the counters are reset after
  the `phi` march and the incidence scope then sets `phi_ref` itself). Uncertain: if a march carries
  the core's `lag_coord` without re-setting it, rung 74's "the lag's COORDINATE ... DECLARED" refusal
  fires instead. Either outcome is recorded.
- **Rung 80 sections E-J, P/split**: unchanged (rung 74's `_coord_march` assigns `lag_coord`; no `CoordScope`).
- Gates: `the_two_key_sets_are_equal` PASSES; `every_value_is_bit_identical_to_the_pypy_golden` FAILS on
  counter keys only; `p2b_the_re_aim_moves_counters_and_no_value` PASSES (values still equal r78, hits>0,
  fb_inc == calls_inc); `p3_two_threads...` FAILS on the same counter keys.
- Outside this file: `slice_ai_cells.rs`'s field readback should also catch it (pre-flight's claim).

## L1 OUTCOME (run 2026-09-29): prediction WRONG on the counters
Oracle: 0 of 37 945 keys moved, counters included. `slice_ai_cells.rs:372`
(`coord_scope_moves_lag_coord_at_78_and_phi_ref_at_79`) is the ONLY catcher. Mechanism: the stuck
`phi_ref` is left on `demand_gains`' own built rig, and nothing on that rig reads it before the next
sample point's scope overwrites it. P2's "caught by the counters" half REFUTED; "by the readback" CONFIRMED.

## L2 — `walls_of` reads the incidence stator's wall as 999.0 instead of `phi_lim_at`
Every rig here arms the VALVE, and `phi_air` takes the valve's wall first; `phi_stator` enters no
`SplitRow` field and the `apart` check reads `phi_air`. Prediction: **0 keys move**, i.e. the
incidence round trip is DEAD on every rung-80 reader, and the claim that the i1 arm drives it is false.

## L2 OUTCOME: prediction RIGHT. 0 of 37 945 oracle keys; slice_ai_split 16/16 and rung80 7/7 green.
The incidence round trip in walls_of is DEAD on every shipped rung-80 path (valve always armed).

## P3G — rung 79's six counters made PROCESS-GLOBAL atomics (flag and log stay thread-local)
Is the P3 gate a catcher, or only a demonstration? Predicted BEFORE running:
- **Parallel harness (default)**: `p3_two_threads...` FAILS (two concurrent counter drives sum into one
  another's readings; `used_fb` too, since it is a before/after read of `fb_inc`). The full-drive PyPy
  gate PROBABLY fails as well (its OnceLock drive overlaps P3's threads). `slice_ai_cells.rs`'s
  per-thread test (`the_counters_are_per_thread_and_move_on_the_real_plant`) FAILS.
- **`--test-threads=1`**: the P3 gate is the ONLY oracle gate that fails — its own two threads still
  overlap; every other drive then runs alone and matches. `slice_ai_cells.rs`'s per-thread test still
  fails (it reads from a second thread by construction).
If P3 passes in the serial mode, the gate is BLIND and "P3 CONFIRMED" is withdrawn.

## P3G OUTCOME: prediction RIGHT in both modes.
serial: p3 FAILED (thread A 53/13 654), every other oracle gate ok; cells per-thread FAILED.
parallel: p3 FAILED (59/13 654), full PyPy gate FAILED (16/37 945), cells per-thread FAILED.
