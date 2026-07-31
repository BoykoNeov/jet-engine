---
name: always-commit-and-push
description: "User wants work committed and pushed to main automatically, without asking"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c26fd4b-a96a-48b1-b4d0-d6638ba4a998
  modified: 2026-07-31T10:33:53.804Z
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

**The green-gate is bare `pytest` — it now runs EVERYTHING (2026-07-31, the
three-gate collapse).** See [[test-suite-speed-policy]]. In short:
- **`pytest` = the gate.** 1002 tests, **2:18**. Nothing is deselected, so a
  green run is a green run — there is no longer a weaker gate to commit on.
- `pytest -m "not slow"` (**1:31**) is an ITERATION opt-out you TYPE. Never
  green-commit on it — it sheds the 224 expensive FINDING sweeps.
- `--affected`, `--runslow`-as-a-tier and the every-3rd-rung cadence are GONE.
  `--runslow` is still accepted as a no-op, so old commands keep working.
- **The former ACCEPTED RISK is retired**: no gate is unreached any more, so a
  regression can no longer hide for up to 3 rungs. (`main.py` is still untested.)

**DO NOT run the gate when ONLY docs changed (2026-07-27, user instruction).** A
docs-only commit (a `docs/*.md` negative record, a `rungN-spec.md` correction,
`CLAUDE.md`) cannot move a test — commit and push it directly. The one exception is
`CLAUDE.md` itself, which has a size guard: run just
`python tests/test_claude_md_reference.py` (instant), not the suite. Reserve the
real gate for commits that touch `turbojet/`, `tests/`, `main.py` or `conftest.py`.
**Why:** a gate run on a docs edit is pure dead time and returns no information
the change could possibly have affected.

**More generally, do not run the gate without a reason (2026-07-31, user):** at
**session end** (unless it ran shortly before) and after a **code** change — but
**NOT** at session start, **NOT** on a docs-only change, and **NOT** "just to be
sure". Cheapness is not a reason to run it reflexively.
