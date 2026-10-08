# Slice AJ step 5 — injection sweep predictions (written BEFORE any injection ran)

Scope: the four NEW binaries only (`rung81`, `rung82`, `rung83`, `rung84`), `--no-fail-fast`.
Each injection is one edit to `rust/src/`, rebuilt in `W:\temp\claude\slice-aj-step5\target`,
reverted before the next. "ONLY" means no other test in the four binaries fails.

| # | injection (src) | prediction |
|---|---|---|
| I1 | `authority_clock`: march at `(tg, tf, tau_q, tau_s)` (clocks swapped) | KILLED — `rung81::reduce_the_reader_reads_split_marchs_own_march` (the substitution's floor), plus others that read the grid (fuel_leg, valve_clock, ... — not predicted individually) |
| I2 | `scan_cells`: march at `(tau_f, tau_gov, kw.tau_s, kw.tau_q)` (q/s swapped) | KILLED — `rung82::reduce_the_scan_reads_split_marchs_own_march` ONLY (every other call has q = s = 0.05) |
| I3 | `authority_clock`: march at `(tf, tg, tau_s, tau_q)` | SURVIVES — the suite's fixture has q = s = 0.05; a coverage gap of the substitution, recorded |
| I4 | `threshold_scan` echoes `tau_gov: kw.tau_q` | KILLED — `rung82::reduce_...` ONLY (the echo check; everywhere else tau_gov = tau_q) |
| I5 | `corrector_secant`: a failed START `continue`s instead of `break`s (step 4's C13) | KILLED — `rung83::void_v4_a_start_point_with_no_residual_stops_the_secant` ONLY (its second arm) |
| I6 | `corrector_step`: void string always `V5: …` | KILLED — `rung83::void_v4_the_step_on_an_empty_window` ONLY (the degenerate-slope test only checks `contains("V5")` at c = 1 with F present) |
| I7 | S2 message formats `flat` with `{}` instead of `py_g` | KILLED — `rung83::void_s2_a_flat_pair_aborts` ONLY |
| I8 | `bisect`'s V3 message formats with `{}` | SURVIVES — `0.004`/`0.3` print identically (declared in `rung82.rs`) |
| I9 | `staircase_number`'s V5 message formats with `{}` | SURVIVES — `0.0198` prints identically (declared in `rung84.rs`) |
| I10 | `root_class`'s V6 message formats with `{}` | SURVIVES — `1.0` prints `1` under `{}` (declared) |
| I11 | `lattice_count`'s void string → `"V3: no edge"` | KILLED — `rung84::void_v3_lattice_count_on_an_empty_window` ONLY |
| I12 | `staircase_number`: V5 checked before V2 | KILLED — `rung84::void_v2_staircase_number_on_an_empty_window` ONLY (both reads empty, `edge_moved` false, so V5 fires first) |
| I13 | `bisect`: V3 straddle test before the window test | KILLED — `rung82::void_v1_a_bracket_end_with_no_window` (at r = 1.0 the empty low end has n_fuel = 0 and the open 0.30 end is all fuel, so the straddle HOLDS and falls through to ... n = 0 returns Ok) ; also `rung84::void_v6_and_the_carried_v1_in_root_class` (root_class's V1 carry). `rung82::v1_no_four_loop_window...` is a scan, not a bisect — untouched |

Floors: I1, I2, I4, I5, I6, I7, I11, I12, I13 must be killed by the gate named. Survivors I3,
I8, I9, I10 are coverage gaps / identities of the formatter, not defects of the gates.

## Addendum, written after I13 SURVIVED and BEFORE this follow-up ran

I13 survived all four binaries: my written premise ("the straddle holds and falls through") was
right, but I forgot that the V1 window test still runs NEXT, so the same void comes back. The
order of the two checks is visible only where BOTH ends are empty (step 3's finding). Follow-up:
I13 against step 3/4's gate binaries. Prediction: KILLED in `slice_aj_threshold` by its
both-ends-empty reading; SURVIVES `slice_aj_staircase` (its `root_v1` sits at r = 1.0, where the
0.30 end is open, the same blind input as `rung84.rs` +4).

## Follow-up after the advisor's closing review — written BEFORE the arms or injections ran

Two more check ORDERS no shipped gate trips both halves of. PyPy measured first
(`probe_order.py`, log opens `3.11.15 … [PyPy 7.3.23 …]`): at r = 1.0, tau = 0.05, c = 1.0 the
step returns `V4: kappa impure` (F None checked first); at r = 1.0, n_bisect = 0, eps = 1.0
`root_class` returns the CARRIED `V1: …` (the carry precedes V6; the void's lo/hi span 0.296 < 1).

| # | injection | prediction |
|---|---|---|
| I14 | `corrector_step`: V5's condition tested before V4's (`V4` only when `|1-c|` is not tiny) | KILLED — `rung83::void_v4_the_step_on_an_empty_window` ONLY, by its new `c = 1.0` arm |
| I15 | `root_class`: a V6 test on the BRACKET's width before the carry | KILLED — `rung84::void_v6_and_the_carried_v1_in_root_class` ONLY, by its new `eps = 1.0`, r = 1.0 arm |
