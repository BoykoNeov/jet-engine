---
name: rust-port-slice-aj-step3
description: "Slice AJ step 3 (rung 82's nine readers) — the one mispredicted injection rested on a docstring's 'no point at all at r >= 1.0', true only at the tau_f its suite checks; a quoted invariant is a premise to check AT the reading's own arguments"
metadata:
  node_type: memory
  type: project
  originSessionId: 25b370c6-98c8-42a1-a768-7e2e95323f1a
  modified: 2026-09-29T17:09:05.714Z
---

Slice AJ step 3 COMPLETE 2026-09-29 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.34.3). `rust/src/threshold_law.rs` (1 004 lines): `scan_cells`, `hats`, `threshold_scan`,
`scan`, `bisect`, `threshold_law`, `threshold_row`, `threshold_reference`, `threshold_terms`. Gate
`rust/tests/slice_aj_threshold.rs`, 12 readings bit for bit + key order vs
`rust/oracle/probe_slice_aj_step3.py` → `slice_aj_step3_pypy.tsv` (957 lines, PyPy; NaN token
`f:nan`). Sweep 22 injections, 21 of 22 predicted. Probes in `W:\temp\claude\slice-aj-step3\`.
Next: step 4 = rungs 83 + 84 (`corrector_law.rs`, `staircase_law.rs`).

**THE LESSON: when a prediction rests on a quoted invariant, check the invariant at the
prediction's OWN arguments.** K16 (bisect tests the straddle before the window) was predicted
killed because rung 82's docstring (`engine.py:22329`) says *"at `r >= 1.0` there is no four-loop
point at all"*. It survived: at `r = 1.0` the bracket's `0.30` end has 14 riding points (all
fuel); the claim holds only at the `tau_f = 0.05` `test_rung82.py` checks. The fix was a
both-ends-empty reading (`bisect_v1`), which kills K16 alone.

**Why:** a docstring sentence reads as universal but was measured at one setting; inheriting it as
a premise imports its scope error silently.

**How to apply:** before predicting from a shipped claim, run the two or three calls that test it at
the arguments your reading actually passes (here: three scans, seconds).

Also:
- The advisor's `%g` catch held: on the default bracket `%g` and Rust `{}` print the same, so only
  `lo = 0.0123456789` killed K5. Pick arguments where the two spellings DIFFER.
- NaN `tau_star_eff` is unreachable on this rig (kappa `[3.0]` on 45 searched marches) — recorded
  unreached, token `f:nan` still wired.
- The whole-probe rerun reproduced the first oracle 892/892 lines — a free determinism check.
- Never edit the file under an injection sweep; the sweep script restores bytes in `finally`.

Related: [[rust-port-slice-aj-step2]], [[rust-port-slice-aj-preflight]], [[rust-port-status]].
