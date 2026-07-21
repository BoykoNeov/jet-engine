---
name: always-commit-and-push
description: "User wants work committed and pushed to main automatically, without asking"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c26fd4b-a96a-48b1-b4d0-d6638ba4a998
  modified: 2026-07-21T20:27:59.415Z
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
run `pytest --runslow` (all 371, ~10–15 min). See CLAUDE.md Commands + `conftest.py`.
