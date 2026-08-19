---
name: rust-port-slice-t-step2
description: "Slice T step 2 (rung 47's 9 gates) — a suite can pass 9/9 while being blind to a 24% value error; check what an injection MOVES before calling a survival a hole"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9e3f701c-f4fe-4ed1-ad5d-c873cef2ac57
  modified: 2026-08-19T18:51:40.987Z
---

Rung 47's nine gates ported one-to-one, **0 source lines added or changed** (step 1 had already
shipped every method they call), 9/9 first run in 0.47 s. The process lessons:

**A GREEN SUITE CAN HAVE NO VALUE CONTENT AT ALL, AND THAT IS COUNTABLE BEFORE IT IS DISCOVERED.**
Five defects injected into the newly-covered lagged marcher: three were caught (by 2–3 gates each),
**two were invisible to all nine** — and those two move 13 of 18 dumped readings by 14–24%. The
reason is arithmetic, not luck: four gates are bit-identities between two runs of the SAME code and
the other five are inequalities with measured margins of 137× / 27× / 8.9e7× / 4.5× / **2.19×**. The
tightest bar sets the detection floor, so a defect had to move a number by more than 2.19× to be
seen. **Tabulate the margins of a ported suite's bars and the blind spot is predictable — don't wait
for an injection to reveal it.**

**MEASURE WHAT AN INJECTION MOVES BEFORE CALLING ITS SURVIVAL A HOLE.** [[rust-port-slice-s-step3]]
recorded the opposite failure (an injection that compiled, applied and could not have moved
anything). Here the precondition was applied up front: both survivors were value-diffed against the
baseline FIRST, and only then reported. Same for the follow-up — the claim "slice S's smoke oracle
is the only thing that catches these" was run against four targets rather than inferred from the
dump's source. It held, and the big oracle turned out NOT to cover the route.

**A VACUOUS PORTED GATE CAN LEAVE A REAL HOLE THAT NEEDS A LINE PYTHON DOES NOT HAVE.**
[[rust-port-ported-test-vacuity]] again: Python compared two calls differing by a keyword whose
default equals the value passed, which in Rust is one struct value, so the loop compares a call with
itself. Naming the vacuity is not enough — the mis-spelled dispatch it was supposed to catch would
still pass everything. The repair is one ADDED assertion on something a fall-through cannot produce.

**A GAP BOOKED AT ONE STEP CAN BE CLOSED BY THE NEXT RUNG'S SUITE RATHER THAN BY A REPAIR.**
[[rust-port-slice-t-step1]] found rung 46 blind to its own LP sign because the quantity is exactly
zero there. Rung 47's fast-ramp gate asserts the same quantity strictly positive at four knob
values, so it carries the sign four times over — and unlike rung 46's one carrier, it is not
`slow`-marked. Worth checking the NEXT suite before writing a repair for a gap in this one.
