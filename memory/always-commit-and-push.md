---
name: always-commit-and-push
description: "User wants work committed and pushed to main automatically, without asking"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c26fd4b-a96a-48b1-b4d0-d6638ba4a998
  modified: 2026-07-27T11:51:25.457Z
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

**The green-gate is `pytest --runslow`, NOT bare `pytest` (2026-07-21).** The
suite is now fast-by-default: bare `pytest` runs only the FAST subset and
**deselects the `slow`-tagged expensive FINDING / robustness gates** (the mixing-PDF
per-pocket sweeps of rungs 16/20–24, the transient marches). The bit-for-bit reduce
SPINE (`test_reduce_*` / `test_cycle_untouched_*` / `*_bit_for_bit`) IS kept in bare
`pytest` (the `_is_spine` override), so a routine run still guards "each rung reduces
to its predecessor" — but it does NOT run the finding gates. Before committing green,
run `pytest --runslow` (~481 tests, ~15 min). See CLAUDE.md Commands + `conftest.py`.

**DO NOT run the gate when ONLY docs changed (2026-07-27, user instruction).** A
docs-only commit (a `docs/*.md` negative record, a `rungN-spec.md` correction,
`CLAUDE.md`) cannot move a test — commit and push it directly. The one exception is
`CLAUDE.md` itself, which has a size guard: run just
`python tests/test_claude_md_reference.py` (instant), not the suite. Reserve
`--runslow` for commits that touch `turbojet/`, `tests/`, `main.py` or `conftest.py`.
**Why:** a full 15-minute run on a docs edit is pure dead time and returns no
information the change could possibly have affected.
