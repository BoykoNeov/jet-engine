---
name: never-run-the-gate-for-timing
description: Never run the test gate just to refresh a quoted timing — take timings from runs that were already happening for another reason
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ee1bb5b3-3bf3-438a-840d-81bfcc41dd85
  modified: 2026-08-10T05:15:24.382Z
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

**And the corollary, learned the hard way at rung 78: a stale timing may be QUOTED but must
never be DIFFERENCED.** The quiet-box ship run read 14:14 against a carried `~9:40 at 1242,
shared box`, and I attributed the 274 s rise to the fingerprint slice in a commit message
without measuring anything. One `--durations=0` on that module killed it: the slice costs
<=91 s and rung 78's own module 50.8 s, and *added wall clock cannot exceed added serial work
under any scheduler*, so at most half the rise was the new tests. The two numbers were never
measurements of the same machine. Measuring ONE MODULE's cost is allowed and cheap — it is not
"refreshing a gate timing" — so when a rise wants explaining, measure the suspect module
instead of reasoning from the difference of two gate runs. And a wrong claim already pushed
gets corrected in place (here, the fingerprint module's § SLICE 3), never force-pushed away.
