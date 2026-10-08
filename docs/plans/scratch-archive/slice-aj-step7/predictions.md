# Slice AJ step 7 — predictions, written BEFORE the file or the mutation runs (2026-10-01)

Seat: rung 81's `authority_mask` at `tests/rung81.rs`'s fixture — PHI stator arm, walls
0.75 / 0.77, `MASK_CLOCKS`, `demand`, r 0.5, settle 1.2, ds 0.005, `inc = false`, every 1.
Fingerprint = `format!("{:?}")` of the whole `AuthorityMask`.

## Part A — P4, `rust/tests/slice_aj_dispatch.rs`

| # | row | install proof | prediction |
|---|---|---|---|
| A0 | non-vacuity of the shipped mask | — | `n_fuel_interior > 0`, `n_gov_interior > 0`, some interior cell with `f_q != 0` |
| A1 | `MacroNone` (this file's rebuild helper, shipped tables) | caller + rig carry `R80_TRIPLE` exactly | **same** fingerprint |
| A2 | COUNT: counter A on the caller's `quad_gains_at`, counter B on the rig's, both delegating | caller carries A only, rig carries B only | **A = 0; B = Σ n_sampled over the two arms** (= Σ n_riding, every = 1); fingerprint **same** |
| A3 | DISTORT on the rig only (`f_q *= 2` after the shipped body) | caller shipped, rig distorted | fingerprint **DIFFERS** (at least `c1`/`c0`/`cyc` on fuel-authority cells) |
| A4 | DISTORT on the caller only | caller distorted, rig shipped | fingerprint **same**, bit for bit |
| A5 | INFORMATIVE: rung 72's body on the rig (the parent the cell was re-aimed FROM at rung 73) | caller shipped, rig carries R72's pointer | **same** (ref_law = "sched" ⇒ rung 73's gains ARE rung 72's; jac4's `F_f`/`R_r` default 0.0 there and are exactly 0.0 under the identity reference). Medium confidence; a DIFF would be a finding about the reduce contract, not a P4 failure |
| A6 | entry control for Part B: the rig `split_march` returns from a shipped rung-80 machine is a RUNG-80 machine, read by `shared_rig` (the deletion cannot move `R80_TRIPLE.shared_rig`) | — | passes |

P4 scored on A2 (dispatch counted) + A3/A4 (value-visible both ways). A5 does not score P4.

## Part B — P5, the closing source mutation: delete `at_lever: r80_at_lever,` in `rust/src/split_wall.rs`

Compiles (`..R79` fills it); rustc warns `r80_at_lever` never used.

| binary | prediction |
|---|---|
| `rung80.rs` | **15 / 16 — `the_knob_is_loud` FAILS** at its step-1 pointer assert (P5's ported-catcher half) |
| `rung81.rs`, `rung82.rs`, `rung83.rs`, `rung84.rs` | all green |
| `slice_aj_clock`, `_threshold`, `_corrector`, `_staircase`, `_plumbing` | all green |
| `slice_aj_oracle.rs` | 5 / 5, **0 tokens moved** |
| `slice_aj_dispatch.rs` (new) | A6 FAILS (rig is rung 79's); the rebuild-helper proof FAILS (shipped `R80.at_lever` no longer carries `sm_air`); A4's install proof FAILS (its rig is built by the shipped table, now `R79_TRIPLE`); A1's install proof FAILS for the same reason only if it compares the SHIPPED rig — it compares the helper rig, so A1 passes; A0, A2, A3, A5 pass (their rigs are the helper's) |
| `slice_ai_cells.rs` | 8 / 10 (as at AI step 7 (e)) |
| `slice_ai_dispatch.rs` | 0 / 9 (as at AI step 7 (e)) |

Restore by `git checkout -- rust/src/split_wall.rs`, blob hash verified against `git hash-object` before.

## Part C — closing scores (read after the last source edit)

* P1: `wc -l` of `authority_clock.rs` + `threshold_law.rs` + `corrector_law.rs` + `staircase_law.rs`; predicted band 2 700 – 3 700. Step 6 read 2 749.
* P6: `TripleHooks` field count from the struct definition: predicted 18. 0 ADD, 0 R81..R84 tables.
* P7: not scored.

## Part A — RESULT (run 1, before Part B): all 7 green, every row as predicted
A0 47 fuel / 33 gov interior, Σ n_sampled 80. A2: caller 0, rig 80. A3: cyc_fuel 1.0437… → 2.0875… (gov cyc 0.0 both). A4 same. A5 same.

## Part B — refined per gate, added BEFORE the mutation (blob of split_wall.rs = 87947cef993fbfa98b3ab51da2c93f08321e44e7)

`slice_aj_dispatch.rs` under the deletion: **4 / 7**.
* FAIL `the_shipped_rig_is_a_rung_80_machine` (shared_rig is R79's)
* FAIL `the_rebuild_helper_is_the_shipped_constructor` (shipped constructor now r79_at_lever: no sm_air)
* FAIL `a_distortion_moves_the_mask_on_the_rig_and_not_on_the_caller` (CallerDistort's install proof: its rig is built by the shipped lever → R79_TRIPLE, diff ["shared_rig","quad_gains_at"]... actually ["shared_rig"] + the quad slot check passes; fails on the diff)
* pass: not_vacuous, helper_row (values unchanged), count, R72.
Also run: `slice_ai_oracle` 6/6 0 keys moved, `slice_ai_split` green (AI step 7 (e)).
Binaries: rung80 rung81 rung82 rung83 rung84 slice_ai_cells slice_ai_dispatch slice_ai_oracle slice_ai_split slice_aj_clock slice_aj_threshold slice_aj_corrector slice_aj_staircase slice_aj_plumbing slice_aj_oracle slice_aj_dispatch.

## Part B — RESULT: every binary as predicted
rung80 15/16 (the_knob_is_loud at rung80.rs:321, the pointer assert); rung81-84 green; slice_ai_cells 8/10; slice_ai_dispatch 0/9; slice_ai_oracle 6/6; slice_ai_split 7/7; slice_aj_{clock,corrector,plumbing,staircase,threshold,oracle} green (oracle 5/5); slice_aj_dispatch 4/7 — the three predicted, CallerDistort's diff ["shared_rig"]. rustc: `r80_at_lever` never used. Restored, blob 87947cef verified.

## Part C — RESULT
P1 2 750 (816+1007+349+578) — in band, per-line mechanism. P6 TripleHooks 18 fields; 0 pub const R81..R84.
Citation guard re-blessed 61/421/245 -> 62/424/245.

## Full gates — predicted before running
cargo test --release --no-fail-fast: 183 result blocks, 1 891 passed, 0 failed.
pytest: 1 387 passed.
cargo 183 / 1891 / 0 — as predicted. pytest 1387 passed (18:56) — as predicted.
