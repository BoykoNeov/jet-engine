# Slice AJ step 3 — injection sweep predictions (written BEFORE any injection ran)

Gate: `cargo test --release --test slice_aj_threshold --no-fail-fast`, 11 readings:
law, ref, terms, scan_ctl, scan_v1, bisect_v3_hi, bisect_v3_lo, bisect_v3_g, law_fine, ref_void, terms_void.

Facts from the explore/nan_search probes (45 marches): kappa = [3.0] everywhere, n_slope_excluded = 0 everywhere.

| # | injection | prediction | why |
|---|---|---|---|
| K1 | `tau_hat_min := arg.hat` | SURVIVE | kappa pure -> tau_eff constant over hats -> argmin(hat - te) = argmin(hat) |
| K2 | argmin keeps the LAST minimum (`<=`) | SURVIVE | no exact tie in hat - te |
| K3 | kappa over HATS, not cells | SURVIVE | no slope-excluded cell |
| K4 | bisect ok: `at_hi` = a scan CACHED when `b` was last set (no fresh scan) | SURVIVE | the march is deterministic |
| K5 | V3 message with `{}` not `py_g` | KILLED — bisect_v3_g ONLY | default bracket prints the same both ways |
| K6 | `gap_falls` by truthiness, not `is not None` | SURVIVE | no gap_bind is exactly 0 |
| K7 | walls iterate UNSORTED `phi_lims` | KILLED — terms_void ONLY | the fixture's list is already sorted |
| K8 | returned `phi_lims` SORTED | KILLED — terms_void ONLY | same |
| K9 | row void reason fix-first | SURVIVE | on law_fine r=1.0 both void with the SAME V1 string |
| K10 | `all_kappa_pure` over LIVE rows only | KILLED — law_fine ONLY | its void row has kappa [] -> impure |
| K11 | unrun step control reads as passed (`None` -> `Some(true)`) | KILLED — law ONLY | law_fine runs the control |
| K12 | threshold_law skips the fix bisection when meas voids | SURVIVE | fix is read only on an ok meas |
| K13 | forward reading ignores kappa (`fwd = tau_hat_min`) | KILLED — law, ref, law_fine | the 3x |
| K14 | `ratio_bind` without the `slope_r != 0` filter | SURVIVE | slope_r never 0 at a binding point |
| K15 | `_grows` sorts by `err` not `dist` | KILLED — ref ONLY | grows_above is False on the fixture; sorted by err it is trivially True |
| K16 | bisect tests the straddle BEFORE the window | KILLED — law_fine, ref_void | r=1.0: key false at both ends -> V3 instead of V1 |
| K17 | mid-bisection V1 returns the ORIGINAL lo/hi | SURVIVE | unreached |
| K18 | P4 `pred` off gl[0] not gl[-1] | KILLED — terms ONLY | terms_void has p4 = None |
| K19 | P5 `d_lag` by `is_some` not truthiness | SURVIVE | slope_r never exactly 0 |
| K20 | walls at `tau_govs[0]`, not the hard-coded 0.05 | KILLED — terms ONLY | terms_void's tau_govs = (0.05,) |
| K21 | `scan_cells` keeps trajectory ENDS | SURVIVE | no riding point at an end on this rig (rung 81 n_edge = 0) |
| K22 | `n_fuel` counted over CELLS (interior), not `ride` | SURVIVE | same — ride has no ends |
