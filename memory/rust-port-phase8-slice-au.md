---
name: rust-port-phase8-slice-au
description: "Slice AU (2026-10-05) — the Python deleted on top of tag python-final; the port is DONE. Lesson: a 'compare while both exist, else skip' test becomes one that cannot fail at the deletion it anticipates — pin it with a value measured BEFORE the delete."
metadata:
  node_type: memory
  type: project
  originSessionId: c0f62838-cee4-4e8e-81f7-8b5e4563b2c7
  modified: 2026-10-05T13:07:54.788Z
---

**Slice AU closed phase 8 and the port, 2026-10-05** (plan § 8.13). Annotated tag `python-final`
on `e56d30d` (pushed), made only after the two citation/CLAUDE.md guards ran green there; then ONE
commit deleting 165 files (`turbojet/`, `tests/`, `main.py`, `conftest.py`, `pytest.ini`,
`docs/visuals/*.py`, 61 `rust/oracle/*.py`). `plot_ts_diagram.py` is the only Python left. The gate
is now `cargo test --release`.

**The proof ran in a scratch worktree** (`git worktree add` under `W:\temp\claude\slice_au\wt`,
`--manifest-path`, since `cd` is hook-blocked), predictions written first: `--no-run` 0 errors /
191 binaries; of the five RUNTIME file readers only `visuals::keep_is_build_cutaways` failed. All
held. A clean worktree is the point: it has no ignored leftovers that could hide a dependency.

**The lesson — a deletion-tolerant test turns vacuous AT the deletion.**
`fingerprint::the_cpython_copy_is_the_audit_record` was written in slice AS to "compare the copy
with the original while it exists; after the delete, print and pass". The proof showed it passing
— through the branch that can never fail again. It was repaired to pin the copy by size + FNV-1a,
measured on the original while it still existed. **How to apply:** before deleting anything a test
compares against, grep for tests with an "absent/skip" branch and convert each into a pin of a
value measured now — after the delete there is nothing left to measure it from.

Also: rung 33's stderr panic lines ACCEPTED (a panic-hook swap is process-global and would
swallow other tests' messages in a parallel binary). The `.venv` is the user's to remove.
See [[rust-port-status]], [[rust-port-phase8-slice-at]], [[always-commit-and-push]].
