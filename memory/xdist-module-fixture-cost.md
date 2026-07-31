---
name: xdist-module-fixture-cost
description: "Under pytest-xdist a module-scoped fixture is rebuilt PER WORKER, so each extra consumer of an expensive reader can cost a whole re-run — measured at rung 71"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 571df774-4dee-4de4-beb7-04a262ba0e8b
  modified: 2026-07-31T17:27:04.899Z
---

**A module-scoped fixture is materialised once PER XDIST WORKER, not once per session.** With
`--dist load` (this repo's `pytest.ini`), N tests consuming one expensive fixture can land on N
different workers and pay for it N times.

**Measured at rung 71 (2026-07-31).** A new test file whose 30 tests ran in 55 s *serially* added
**2:37 to a 2:59 gate**. The cause was a 20-second `full_modes` fixture with **five** consumers.
Two changes took the addition down to **19 s** (gate 2:59 → 3:18):

1. consolidated the five consumers to three (merging the cheap invariant checks — `c0`, Routh,
   RK4 — into one test whose docstring records *why* it is one test);
2. cut the reader's default clock grid from 10 arms to 6 — the smallest grid still spanning the
   three regimes the claim needs, against the 4 rungs 68/69/70 default to. The spec's wider table
   is that same reader called with an explicit `clocks`, and it was re-run to confirm.

**Why:** a per-rung test file that adds minutes to the ONE gate erodes the policy that makes the
single gate viable ([[test-suite-speed-policy]]). The fix is not "run fewer tests" — it is to stop
paying for the same computation on several workers.

**How to apply:** when a new rung's tests noticeably slow the gate, run
`pytest tests/test_rungN.py -q -n0 --durations=12` first. If a `setup` line dominates, count that
fixture's consumers — that count multiplies its cost. Consolidate consumers, or make the reader
cheaper, before concluding the work itself is expensive. Also worth knowing: a serial timing of
the file alone **understates** its gate cost for exactly this reason.

Related: the CLAUDE.md gate timings drift and should be re-measured, not trusted — see
[[test-suite-speed-policy]]. At rung 71 the documented 2:58 was confirmed accurate by running the
gate with the new file excluded (1099 tests, 2:59), which is the cheap way to tell a regression
from a stale baseline.
