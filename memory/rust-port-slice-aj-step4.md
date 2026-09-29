---
name: rust-port-slice-aj-step4
description: "Slice AJ step 4 (rungs 83 + 84's ten readers) — the first 'PyPy' oracle was written by system CPython through a quoted `cmd start` launch; an hour of root cause ran against the wrong interpreter. A file name is a label, not an entry control; a hazard filed in one note does not reach a recipe filed in another"
metadata:
  type: project
---

Slice AJ step 4 COMPLETE 2026-09-29 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.34.4). `rust/src/corrector_law.rs` (349 lines: `corrector_read`, `corrector_step`,
`residual_shape`, `corrector_secant`) and `rust/src/staircase_law.rs` (578: `edge_read`,
`classify`, `staircase_scan`, `lattice_count`, `staircase_number`, `root_class`). Gates
`rust/tests/slice_aj_corrector.rs` (14 readings) + `slice_aj_staircase.rs` (22), bit for bit + key
order vs `rust/oracle/probe_slice_aj_step4.py` → `slice_aj_step4_pypy.tsv` (3 387 lines). Sweep
29 injections, 29 of 29 verdicts right (11 survivors: 1 identity, 1 conditional, 9 coverage gaps).
Citation guard 57/395/239. **Gates NOT complete** (user said stop and wrap up):
`cargo test --release` 91/177 blocks, 1 194/0, killed by PID in `rung77`, the `slice_aj_*`
binaries not reached; `pytest` not run. **Step 5 opens by running both gates whole.**
Scratch in `W:\temp\claude\slice-aj-step4\`. Next: step 5 = the four ported suites + the ten driven
voids.

**THE LESSON: a file name that claims an interpreter is a LABEL; the entry control is the script
printing `sys.version` (and refusing a `*pypy*` output off PyPy).** The first oracle was launched
from Git Bash as `cmd //c start //belownormal //b //wait "…\.venv\Scripts\python.exe" "probe.py"
out_pypy.tsv`. `start` ate the quoted exe as the WINDOW TITLE and ran the script through the `.py`
association, which is system CPython 3.14. The Rust gates failed on 27 lines at 2 readings, and
**an hour of root cause went into CPython's arithmetic**, which was accurate and irrelevant: the
Illinois `tol = 1e-12` in `_close_fuel` (`engine.py:9088`) amplifying a one-ulp `eta_c_at`
`(n - 1.0) ** 2` (`engine.py:1054`, CPython `pow` vs PyPy `x*x`). A `--jit off` flag exposed it
("The system cannot find the file --jit"). Against a real PyPy oracle, 27/27 gates were green with
no port change.

**Why it happened although it was known:** [[windows-tooling-file-hazards]] hazard 7 recorded this
exact trap on 2026-09-26. The recipe I FOLLOWED, [[run-tests-below-normal]]'s Git Bash line, did not
mention it (it works only for unquoted `cargo`). Fixed there now.

**How to apply:** launch any quoted program with PowerShell `Start-Process -FilePath`, set
`PriorityClass`, and make the first line of every probe log `sys.version`. When a Python-vs-Rust
divergence appears, check the log's interpreter line BEFORE the first root-cause step. When a
hazard is recorded, grep the recipes that could trigger it and patch them in the same commit.

Also:
- **Over-correction:** after the CPython discovery I withdrew EVERY claim measured on it, including
  "F differs at r = 0.5, tau = 0.08". The sweep then killed C2 at exactly that reading. Re-measured
  off the PyPy oracle, it holds: `read_ctl` was not among the 27 values CPython moved, and `/ + -`
  round the same on both interpreters. Withdraw only what the wrong interpreter could have moved.
- An empty float-keyed dict (`summands`) was flattened as `keys:0`: detect a dict's encoding by its
  FIELD name, not by testing its keys (an empty dict has none).
- For step 6: P2's "bit-exact on CPython except authority_mask" is refuted for rungs 83/84 at 27
  keys (`scan_p4` 24, `secant_iter_v4` 3). Point-specific, mechanism named. Not scored at step 4.
- Steps 2 and 3's oracles re-run on confirmed PyPy: byte-identical (16 119, 957 lines).
- A citation re-bless prints REMOVALS as `text -> None`: my rewritten header had dropped step 1's
  `22881` and misquoted `22879`'s spacing; the first run silently un-blessed the anchor. Read the
  removals as carefully as the additions.
- `V4: kappa impure at an iterate` (never driven by the pre-flight) IS reachable: `(0.30, 0.29)` at
  `r = 1.0`.

Related: [[rust-port-slice-aj-step3]], [[rust-port-slice-aj-preflight]], [[rust-port-status]],
[[rust-port-power-spelling]].
