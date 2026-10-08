# Slice AI step 3 — injection predictions (written BEFORE any injection ran)

Gates: `rust/tests/slice_ai_march.rs` (4 tests) + the rest of the slice AI files.

| # | injection | predicted | why |
|---|---|---|---|
| J1 | counter reset moved to the TOP of `coord_march` | SURVIVE | probe: both `accel_for` calls move 0 counters; the `phi` march dispatches to rung 78 (advisor's registered survivor) |
| J2 | `gap_med` as the LOWER median (`len/2 - 1`) | lean SURVIVE (uncertain) | 129 distinct gaps over 1 366 rows, ~10 per run; neighbours at index 682/683 likely equal floats |
| J3 | `vacuous`: `d_max == 0` sent to `None` (Python's recorded first version) | KILL | `vacuous == Some(true)` pinned |
| J4 | `forced_cap` Illinois tol = `LEG_TOL` | SURVIVE | same value, 1e-13 |
| J5 | `forced_cap` binding walk resets `glo = None` after a non-negative reading | SURVIVE | the walk breaks on the first negative; the stale arm is never the last one read |
| J6 | `forced_cap` SLACK walk uses `shrink` for `grow` | SURVIVE | every point binds (`n_binding = 10`) — slack arm UNREACHED on this rig; owed at step 5 |
| J7 | `with_probe` restores the log to `None`, not the previous | KILL | nest gate: outer loses its first row / `expect` on the next append |
| J8 | `with_probe` does not reset the log on entry | KILL | nest gate: inner gets 2 rows |
| J9 | incidence scope entered OUTSIDE the probe instead of inside | SURVIVE | restore order unobservable |
| J10 | `n_reach` via `f64::min` | SURVIVE | no NaN reached |
| J11 | `py_set_len` dedups on bits (no `-0.0` merge) | SURVIVE | no `-0.0` in the log (abs of positives, positive caps) |
| J12 | the reference (`traj0`) march run at INCIDENCE | SURVIVE | the march is value-invisible to the coordinate (`worst = 0`, `d_max = 0`); counters reset after it |
| J13 | `coord_forced`'s `w_inc` solved in `phi` | KILL | `w_inc`, `d_forced`, `n_same_float` bits |
| J14 | `coord_forced`'s `w_shipped` taken from `forced_cap("phi")` instead of `_surge_fuel` | SURVIVE | a copy of the same arithmetic: `d_shipped == 0` cannot tell the reference from the instrument |
| J15 | the probe's `p_phi` solved in the machine's coordinate (incidence) | KILL by COUNTERS only | `p_cap == p_phi` bitwise at every row already (`d_max = 0`), so the digest holds; `calls_inc` doubles, `calls_phi` 0 |
| J16 | `CoordScope::drop` writes `lag_coord` directly (item L's survivor) | KILL | post-march readback of `(lag_coord, phi_ref)` |
