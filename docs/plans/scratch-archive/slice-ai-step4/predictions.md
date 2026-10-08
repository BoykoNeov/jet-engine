# Slice AI step 4 — injection predictions (written BEFORE the sweep ran)

Sweep target: `--no-fail-fast --test slice_ai_split --test slice_ai_cells`.
Gates in slice_ai_split: L (liveness), A (arrest), S (saturation), GC (gains clip),
GD (gains demand), RB (split_march readback), AS (air scope).

| # | injection | prediction | why |
|---|---|---|---|
| K1 | rung 74 `coord_march` writes the coordinate THROUGH THE CELL (`CoordScope` leaked) instead of `lag_coord.set` | KILLED by L, GC (clip rows march demand: values), by RB (readback), and by COUNTERS in A/S/GD (phi_ref = "demand" enters rung 79's incidence branch) | on R80 the cell writes `phi_ref`; `lag_coord` keeps the "demand" at_lever copied, so demand marches keep their values but bump the counters |
| K2 | `AirScope::drop` does nothing | KILLED by the restore checks only (L, A, S, GC, GD, RB, AS); NO value moves | every march sets `sm_air` afresh, so a stale knob is overwritten before it is read |
| K3 | `sm_air = pa * (1.0 / ps) - 1.0` | KILLED (moderate confidence) — `phi_air_built` bits in at least one row | a reciprocal-multiply rounds differently from a divide for some walls |
| K4 | `arrested`: `<` for `<=` | SURVIVED | arrested plants sit at `max_tt4 == Tt4_lo` exactly, strictly below `Tt4_lo·(1+1e-9)` |
| K5 | `valve_moved`: `>=` for `>` tol | SURVIVED | no displacement lands exactly on 1e-12 |
| K6 | `b_max_hit`: `>` for `>=` | SURVIVED | a saturated valve reads `b == b_max`, strictly above `b_max·(1-1e-12)` |
| K7 | `cyc_fwd = f_q * (c_v * v_f)` | KILLED (moderate) by GC — the shared-wall arm's `|cyclic| ≈ 1` cells | reassociating a nonzero triple product moves the last bit; the split arms are exactly 0 either way |
| K8 | `rate` summed in reverse order | SURVIVED | `zeros` compares roots against `1e-4·rate`, nowhere near the threshold |
| K9 | stride 4 for 5 (`GAINS_EVERY`) | KILLED by GC, GD (cell count) | |
| K10 | `masked` kept in INSERTION order, not sorted | KILLED by GC's aggregate | the clip shared-wall arm's first cell is FUEL authority, so insertion gives `[gov, fuel]`, sorted `[fuel, gov]` |
| K11 | `authority` dict sorted by label instead of insertion | SURVIVED | insertion order (fuel then gov) already equals label order on every arm |
| K13 | `split_march` passes `ref = "applied"` | KILLED (moderate) by values | the applied reference changes which fuel the demand leg lags |
| K14 | arrest `fuel` arm built as `(w, None)` (= shared) | KILLED by A (rows, owner) | |
| K16 | `walls_of`: stator before valve for `phi_air` | SURVIVED (moderate) | both walls are rebuilt from the same `sm_air` through the same `(1+sm)·ps` spelling, so they are one float |
| K17 | the `ShareScope("max")` around `quad_gains_at` removed | SURVIVED | the rig's `share_law` default is already `"max"` — the scope is inert, as rung 74's `coord_march` note records |
| K19 | `control_nonzero` filters `masked == Fuel` | KILLED (moderate) by GC | the positive control's ≈1 lives on the governor-masked cells |
