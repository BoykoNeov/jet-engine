---
name: schedule-check-census
description: "2026-10-14: rung 63's floor table re-derived with below_ceiling on; the census showed the switch reaches rungs 49-73, so the default stays OFF (user) — process lessons"
metadata:
  node_type: memory
  type: project
  originSessionId: e4d621d9-cfe0-43ec-934c-ff0a78178a12
  modified: 2026-10-14T11:41:48.157Z
---

2026-10-14. CLAUDE.md said only "rung 58's floor dichotomy (rung 63's table)" stopped on rung 43's
schedule check. A runtime census showed otherwise. A throwaway worktree logged every hit of the
check's error arm (exe + test-thread name → TSV) and ran the full gate with the switch off, then
forced on. It reached rungs 49, 63, 64 and 66–73, the CLI, 12 bit-gates, and the reduce to
rung 65 (rungs 65+ have their own marchers without the ceiling). The user chose to re-derive
rung 63's table only (sandbox plan § 14.4, rung63-spec § 3). Lessons:

- **A seam's stated scope is a claim; census it by RUNNING, not by name.** Logging the error arm
  with `std::thread::current().name()` names the TEST that hit it, for free. Stop and ask the
  user once the reach is wider than the task, before licensing anything (the advisor's call).
- **Pin a licence's key set from the measurement, never from the prediction.** I predicted 2
  moved oracle keys; the measured set was 5 (the oracle's stator-lever variants stop on the
  check too). An empty `LICENSED` const that fails with the measured list is the cheap way.
- **Parallel tests share ONE panic hook.** `slice_w_oracle`'s `refuses()` swaps it, so the real
  failure's message was silenced and an expected refusal printed instead. Re-run with
  `--test-threads=1` before diagnosing a printed panic.
- **The Start-Process wrapper reported exit 0 on a failing cargo test.** Read the test output
  (global rule), never the wrapper's code.
- Licence shape that worked: re-run the old march's failing step through public calls (validated
  bit-equal at every recorded point first). Hold the moved key to Python THROUGH the unchanged
  route, and assert the new value moved. `tests/common/schedule_check.rs`.

Related: [[solver-walls-repair]], [[census-by-compiler-not-by-name]], [[sandbox-slice4b]].
