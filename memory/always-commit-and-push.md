---
name: always-commit-and-push
description: "User wants work committed and pushed to main automatically, without asking — a standing preference, reaffirmed 2026-09-07"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c26fd4b-a96a-48b1-b4d0-d6638ba4a998
  modified: 2026-10-05T13:06:44.595Z
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

**The green-gate is `cargo test --release` (in `rust/`) — it runs EVERYTHING
(since 2026-10-05, phase 8 slice AU deleted the Python; the pytest gate it replaces
is at tag `python-final`).** Its size and duration live in CLAUDE.md § Commands,
measured, not tracked here. Launch it below-normal ([[run-tests-below-normal]]).
Nothing is deselected, so a green run is a green run. `main.py`'s old "untested"
risk is gone too: `cli_golden` holds the CLI byte-equal to the last Python output.

**DO NOT run the gate when ONLY docs changed (2026-07-27, user instruction).** A
docs-only commit (a `docs/*.md` negative record, a `rungN-spec.md` correction,
`CLAUDE.md`) cannot move a test — commit and push it directly. The one exception is
`CLAUDE.md` itself, which has a size guard: run just
`cargo test --release --test claude_md_reference` (seconds once built), not the gate.
When a change reaches exactly one test binary, running THAT binary is the
proportionate check, not the whole gate.

(2026-10-02, user: "stop it and proceed" — when Python still existed, a Rust-only change
owed only `cargo test` plus the two Python guards, not the whole pytest suite. Superseded
by the delete; kept for the principle: ask what can actually break.)

**Narrower still: run only the tests that CAN break (2026-10-03, user: "arent you running too many
tests now, except only ones, where something can break?").** Even `cargo test --release` whole is
too much when a change reaches one test binary. A panels-only change (`rust/src/panels/*` +
`PANELS` rows) is reached only by `cli_golden` — run `cargo test --release --test cli_golden`
(which compiles the crate), and commit on that. Ask what reads the
changed code; the full cargo run is for changes to shared `src/` modules many binaries import.

**More generally, do not run the gate without a reason (2026-07-31, user):** at
**session end** (unless it ran shortly before) and after a **code** change — but
**NOT** at session start, **NOT** on a docs-only change, and **NOT** "just to be
sure". Cheapness is not a reason to run it reflexively.
