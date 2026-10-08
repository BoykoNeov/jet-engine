# Slice AI step 5 — injection predictions, written BEFORE any run

Run: `cargo test --release --no-fail-fast --test rung79 --test rung80`, one injection at a time.

| # | injection | file | predicted |
|---|---|---|---|
| N1 | `_a_cap`'s outer freeze removed (the helper solves `cap_free` on an UNFROZEN plant) | rust/tests/rung79.rs | KILLED — `gap_at_zero_margin_is_exactly_zero` and `the_gap_residual_is_rung_77s_stiffness`: `w` is solved with `b_state`/`v_state` = None, so the plant solves its own valve/stator and the fixed point moves. Lean, not certain: the standstill state may equal the instantaneous one. |
| N2 | `R79.at_lever` left at rung 78's (`at_lever: r79_at_lever` deleted) | rust/src/state_coordinate.rs | KILLED in rung79.rs, several gates: `the_carried_knob_survives_at_lever` (pointer AND phi_ref), and every march reader (`coord_march` rebuilds through `at_lever` → a rung-78 rig → the incidence branch never runs → `hits = 0`). rung80.rs: SURVIVES (R80 has its own `at_lever`; #1's scan passes explicit coordinates). |
| N3 | J6: `forced_cap`'s slack walk `hh *= shrink` | rust/src/state_coordinate.rs | KILLED — +3 (a refusal where a root was due) and +2 (the walk length `searched to 4.181503e` moves). Every other gate SURVIVES (shipped reading binds at all 10 points). |
| N4 | the valve wall re-spelled `ps + sm*ps` in `BleedLimiter::from_margin_tau` | rust/src/limited_bleed.rs | KILLED — +1 (P5's band moves, or the k=0/k=3 wall bits change). Other rung-80 gates: uncertain — if the re-spelling moves 0.8 by an ulp the "shared" wall is split by one ulp and the arrest control may move. |
| N5 | `split_arrest`'s first clause dropped (`&& phi_lim_lo < min(walls)`) | rust/src/split_wall.rs | KILLED — +3 only. +4 and every fixture SURVIVE. |
| N6 | `_split_row`'s valve assert HOISTED above the three reads | rust/src/split_wall.rs | KILLED — +2 only, on its second half (the `KeyError`-shaped message). |
| N7 | `AirScope::drop` restores nothing (step 4's K2) | rust/src/split_wall.rs | KILLED — rung80 #2 (`sm_air` readback), #3 (readback after the refusal), +1 (readback at k=0). No VALUE gate in rung80.rs sees it. |
| N8 | `r79_cap_fuel`'s `phi` early return removed (at `"phi"` the incidence-branch CODE runs, solving in `phi`) | rust/src/state_coordinate.rs | KILLED — `reduce_phi_is_rung_78_by_dispatch` by its COUNTER assertion (`hits` moves at `phi`). The bit equality may well hold (same residual, same fallback), which is why the counters are paired with it. Others: `the_carried_knob…` survives; march-based rung-79 gates may move (`hits` counts double?) — uncertain. |
