---
name: rust-port-slice-x-step5
description: A zero-count assertion is satisfied by DELETING the branch it names — manufacture the branch instead
metadata:
  type: project
---

Slice X step 5 first gated a deliberately-dead branch by asserting its counter stayed at zero. That is
**vacuous**: a port that deleted the branch satisfies it too, so the assertion cannot see the defect it
exists to name. Rewritten to MANUFACTURE the branch reachable — set its carrier by hand, assert the
branch answers, assert the higher-precedence branch wins over it, assert the guard's destructor exposes
it again rather than erasing it. Mutation then confirmed it catches both the deletion and a precedence
inversion; the vacuous version caught neither.

**Why:** "this never happens" and "this does not exist" produce identical counts. Only exercising the
branch distinguishes them.

**How to apply:** for every dead-branch gate ask what a port that DELETED the branch would score. If the
same, manufacture reachability. Where that is impossible without a production-code hook (an error path
you cannot inject), say plainly in the doc that it is a WATCHDOG not a gate — what it catches (the arm
starting to fire) and what carries the rest (a `Result` signature the compiler enforces, the oracle's
keys). The step's best number: making the RAII guard's destructor a no-op reddens **8 of 9** gates,
which is the measurement justifying a destructor over a `finally` anyone can forget.
