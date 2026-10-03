---
name: always-commit-and-push
description: "User wants work committed and pushed to main automatically, without asking — a standing preference, reaffirmed 2026-09-07"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c26fd4b-a96a-48b1-b4d0-d6638ba4a998
  modified: 2026-10-02T08:13:07.702Z
---

When work reaches a green, complete state, commit it and push to `main`
without waiting for an explicit "commit" request.

**Why:** The user stated "always commit and push" — they don't want to be
asked each time; the default should be to persist finished work. **Reaffirmed
2026-09-07, as a standing preference**, after a session opened by *listing*
uncommitted work as a decision for them instead of just committing it. Finished
green work sitting in the working tree is not a question — it is an unfinished
chore. **A dirty tree inherited from a previous session is covered too:** if it
is green and coherent, commit and push it, and say so in one line; do not open
with "shall I commit this?".

**How to apply:** After a coherent unit of work passes its checks, stage
everything, write a descriptive commit, and push to origin main. Still respect
the [[session-end-routine]] (also refresh memory + docs at session end) and
[[git-remote-setup]] (origin = github.com/BoykoNeov/jet-engine over SSH). What
still IS a question: anything outward-facing that a commit is not — publishing
or re-minting an artifact URL, for instance ([[visuals-artifact]]).

**The green-gate is bare `pytest` — it runs EVERYTHING (2026-07-31, the
three-gate collapse).** See [[test-suite-speed-policy]]. In short:
- **`pytest` = the gate.** Nothing is deselected, so a green run is a green run
  — there is no weaker gate to commit on. Its size and duration live in
  CLAUDE.md § Commands, measured, not tracked here.
- `pytest -m "not slow"` is an ITERATION opt-out you TYPE. Never green-commit on
  it — it sheds the expensive FINDING sweeps.
- `--affected`, `--runslow`-as-a-tier and the every-3rd-rung cadence are GONE.
  `--runslow` is still accepted as a no-op, so old commands keep working.
- **The former ACCEPTED RISK is retired**: no gate is unreached any more, so a
  regression can no longer hide for up to 3 rungs. (`main.py` is still untested.)

**DO NOT run the gate when ONLY docs changed (2026-07-27, user instruction).** A
docs-only commit (a `docs/*.md` negative record, a `rungN-spec.md` correction,
`CLAUDE.md`) cannot move a test — commit and push it directly. The one exception is
`CLAUDE.md` itself, which has a size guard: run just
`python tests/test_claude_md_reference.py` (instant), not the suite. Reserve the
real gate for commits that touch `turbojet/`, `tests/`, `main.py` or `conftest.py`
— and when the change reaches exactly one test file, running THAT file is the
proportionate check, not the whole gate.

**A RUST-ONLY change does not owe the Python suite either (2026-10-02, user: "stop it and
proceed").** When a commit touches only `rust/` (+ docs/memory) and no `.py` under `turbojet/`,
`tests/`, `main.py` or `conftest.py`, the gate is `cargo test --release` plus the two Python files
that READ Rust or CLAUDE.md: `tests/test_rust_line_citations.py` and
`tests/test_claude_md_reference.py`. The other ~1 380 tests check an unchanged Python model and
cannot move. Phase 8 slice AK launched the full suite anyway (an advisor read "green = bare pytest"
literally); the user stopped it. New `rust/oracle/*.py` dump scripts are not tests and do not count.

**Narrower still: run only the tests that CAN break (2026-10-03, user: "arent you running too many
tests now, except only ones, where something can break?").** Even `cargo test --release` whole is
too much when a change reaches one test binary. A panels-only change (`rust/src/panels/*` +
`PANELS` rows) is reached only by `cli_golden` — run `cargo test --release --test cli_golden`
(which compiles the crate) plus the two Python guards, and commit on that. Ask what reads the
changed code; the full cargo run is for changes to shared `src/` modules many binaries import.

**More generally, do not run the gate without a reason (2026-07-31, user):** at
**session end** (unless it ran shortly before) and after a **code** change — but
**NOT** at session start, **NOT** on a docs-only change, and **NOT** "just to be
sure". Cheapness is not a reason to run it reflexively.
