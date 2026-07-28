---
name: always-commit-and-push
description: "User wants work committed and pushed to main automatically, without asking"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c26fd4b-a96a-48b1-b4d0-d6638ba4a998
  modified: 2026-07-28T12:15:44.810Z
---

When work reaches a green, complete state, commit it and push to `main`
without waiting for an explicit "commit" request.

**Why:** The user stated "always commit and push" — they don't want to be
asked each time; the default should be to persist finished work.

**How to apply:** After a coherent unit of work passes its checks (tests green,
build clean), stage everything, write a descriptive commit, and push to origin
main. Still respect the [[session-end-routine]] (also refresh memory + docs at
session end) and [[git-remote-setup]] (origin = github.com/BoykoNeov/jet-engine
over SSH).

**The green-gate is `pytest --affected`, NOT bare `pytest` and no longer
`--runslow` every time (2026-07-28, user's cadence choice).** See
[[test-suite-speed-policy]] for how the selector works. In short:
- `pytest` (fast subset) is for ITERATION only — it deselects the expensive
  `slow` FINDING gates. Never green-commit on it.
- **`pytest --affected` is the per-rung gate** (~6–16 min). It keeps every fast
  test and re-enables the slow gates only for the modules the working diff can
  reach. It self-escalates to the full gate when it cannot reason.
- **`pytest --runslow` (~22 min) every 3rd rung**, at session end, and whenever
  `--affected` escalates. The report header counts rung commits since the last
  full gate and nags when the cadence is due.
- The bit-for-bit reduce SPINE (`test_reduce_*` / `test_cycle_untouched_*` /
  `*_bit_for_bit`) is never slow-tagged, so it runs on EVERY invocation of all
  three. That is what makes the reduced gate safe.
- ACCEPTED RISK the user signed off on: a regression in an unreached non-spine
  FINDING gate can hide for up to 3 rungs.

**DO NOT run the gate when ONLY docs changed (2026-07-27, user instruction).** A
docs-only commit (a `docs/*.md` negative record, a `rungN-spec.md` correction,
`CLAUDE.md`) cannot move a test — commit and push it directly. The one exception is
`CLAUDE.md` itself, which has a size guard: run just
`python tests/test_claude_md_reference.py` (instant), not the suite. Reserve the
real gate for commits that touch `turbojet/`, `tests/`, `main.py` or `conftest.py`.
**Why:** a full 22-minute run on a docs edit is pure dead time and returns no
information the change could possibly have affected.
