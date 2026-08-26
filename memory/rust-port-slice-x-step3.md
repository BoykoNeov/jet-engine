---
name: rust-port-slice-x-step3
description: A probe that cannot tell a crash from a compile error reports its best result as its own failure
metadata:
  type: project
---

Slice X step 3's injection census classified any non-zero exit from its witness binary as "BUILD FAIL"
and then **skipped running the gates for that row entirely**. Four of eight injections exited 101 — a
runtime PANIC, on the ported code's OWN internal assert, which is the strongest detector in the set.
The instrument was reporting its best evidence as its own inability to compile.

**Why:** exit-code triage that collapses "did not build" into "did not run" throws away the rows where
the subject defended itself loudest, and it does so silently — the table just shows a column of n/a.

**How to apply:** in any inject-and-observe harness, separate BUILD FAIL from RUNTIME PANIC from RED
from GREEN, and run the gates unconditionally regardless of what the witness did. Also from this step:
the two gates ADDED beyond the ported suite were the ONLY things that caught the two measured holes —
against the ported gates alone both holes remained holes, so report hole counts against BOTH grids or
the slice reads as refuting its own pre-registration.
