# Slice AJ step 4 — injection sweep, PREDICTED BEFORE ANY RUN (2026-09-29)

Written before the oracle probe finished and before the first gate run. Each injection is one
exact-string edit to `rust/src/corrector_law.rs` or `rust/src/staircase_law.rs`, run with
`cargo test --release --no-fail-fast --test slice_aj_corrector --test slice_aj_staircase`, bytes
restored in `finally`. A test function checks several readings and stops at the first that differs,
so "killed by X" names the READING, and the test is the resolution the runner reports.

Kinds: KILL (named readings), SURVIVE-IDENTITY (no input can separate), SURVIVE-CONDITIONAL (a
measured plant property hides it), SURVIVE-GAP (no reading reaches the case).

## Rung 83 — corrector_law.rs

| # | injection | prediction |
|---|---|---|
| C1 | `exact`'s pred as `t - tau_f * k` (commuted multiply) | SURVIVE-IDENTITY — IEEE multiply commutes exactly |
| C2 | `F := h/kappa + tau_f` (rung 84's spelling) | KILL — `read_ctl` (F measured to differ at r=0.5, tau=0.08) and so `step_ok`; others not predicted |
| C3 | V5 guard `<=` instead of `<` | SURVIVE-GAP — no reading has `|1-c|` exactly `1e-12` |
| C4 | step void always says V5 | KILL — `step_v4` ONLY |
| C6 | ladder point `lo + i*step` | SURVIVE-GAP — every ladder here has `n-1` in {1, 2}: a power of two, where both spellings round identically |
| C8 | `jump_in_F` guard `is_some()` not truthiness | SURVIVE-GAP — no `F` is exactly 0.0 |
| C9 | `n_argmin_switches` counts only Some/Some pairs | KILL — `shape_v1` ONLY (None -> Some) |
| C10 | clamps not counted | KILL — `secant_clamp` ONLY |
| C11 | clamped flag set but the UNCLAMPED `tn` read | KILL — `secant_clamp` ONLY |
| C12 | S2 message with `{}` | KILL — `secant_s2` ONLY |
| C13 | a failed start `continue`s instead of `break` | SURVIVE-GAP — the only start failure here is the SECOND start, where break and continue both end the loop |
| C14 | `converged` ignores clamps | SURVIVE-GAP — the clamping secant never reaches 1e-12 anyway |
| C16 | iterate void says "at a start point" | KILL — `secant_iter_v4` ONLY |

## Rung 84 — staircase_law.rs

| # | injection | prediction |
|---|---|---|
| S1 | `n_scored := cells.len()` | SURVIVE-GAP — `n_slope_excluded = 0` and no merged key |
| S2 | `edge` from the first HAT, not `ride[0]` | SURVIVE-GAP — no riding point at a trajectory end; hats = cells = interior ride |
| S3 | `g := min(hat)/kappa - tau_f` (rung 83's spelling) | KILL — `edge_jump_lo` (g measured to differ at both jump taus); others not predicted |
| S4 | summands keep the FIRST value on a repeated key | SURVIVE-GAP — no merge within a march |
| S5 | `s_bind` keeps the LAST minimum | SURVIVE-GAP — no tie |
| S6 | `edge_index` via `round()` (half away) | SURVIVE-GAP — no exact .5 |
| S8 | `set_changed` from `entered` only | KILL — `classify_rev` ONLY |
| S9 | `kind` without the `sign_full` guard (never None) | KILL — `classify_v1`, `scan_p4` (pairs without a sign change) |
| S11 | `edge_monotone` with `>` | KILL — `scan_p4` ONLY |
| S12 | `n_jumps := ib - ia` | KILL — `lattice_p3`, `lattice_fine` |
| S14 | tread guard `is_some()` not truthiness | KILL — `sn_zero` ONLY (tread 0.0 vs None) |
| S15 | V5 message with `{}` | KILL — `sn_v5` ONLY |
| S16 | V5 checked before V2 | KILL — `sn_v2` ONLY |
| S17 | V6 message with `{}` | KILL — `root_v6` ONLY |
| S18 | `root_exists := kind.is_some()` | KILL — `root_jump` ONLY |
| S19 | `root_class` bisects on `key_fuel` | KILL — `root_jump`, `root_cross`; `root_v6` SURVIVES (V6 carries no bracket, and the fuel bisection is also ok at r=0.35), `root_v1` SURVIVES (V1 before any key) |

29 injections (13 rung 83, 16 rung 84): 16 predicted KILL by exact reading set, 2 KILL with only
a floor named (C2, S3), 1 IDENTITY (C1), 10 GAPS (C3 C6 C8 C13 C14 S1 S2 S4 S5 S6). S19 also
predicts two readings that SURVIVE inside a killed injection.
