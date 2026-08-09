---
name: never-run-the-gate-for-timing
description: Never run the test gate just to refresh a quoted timing — take timings from runs that were already happening for another reason
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ee1bb5b3-3bf3-438a-840d-81bfcc41dd85
  modified: 2026-08-09T15:03:46.832Z
---

Do **not** run `pytest` (or any subset) for the purpose of measuring or refreshing a
quoted run time. If CLAUDE.md's gate timing looks stale, either take the number from a
gate run that was already happening for a real reason (a code change, session end), or
leave it stale and say so.

**Why:** the gate costs minutes of wall-clock. A timing number is documentation, not a
correctness signal — spending a full gate run on it buys nothing the user needs, and
repeating the run "to see if it's repeatable" doubles the cost for a number that was
never load-bearing. Rung 72 already established that an unreliable timing should just be
flagged stale; that disclosure is cheap and re-measuring is not.

**How to apply:** run the gate only for the reasons [[always-commit-and-push]] and
CLAUDE.md name — after a code change, or at session end if it hasn't run recently. When
one of those runs happens to produce a clean timing, update the quoted number from it as
a free side effect. Never schedule a run whose only output is a timing, and never run a
second one to confirm a timing. See also [[test-suite-speed-policy]].
