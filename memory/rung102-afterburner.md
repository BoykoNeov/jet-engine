---
name: rung102-afterburner
description: "Rung 102 (afterburner) process lessons — a cost trigger fired and was fixed by COUNTING work, not guessing; two ranges written from memory into the spec were both wrong; a stated direction must be checked at the cells it was stated for"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c72a2043-730f-42b0-8a5e-3b17edf7d8be
  modified: 2026-10-19T08:09:09.145Z
---

Rung 102, Parts A and B (2026-10-19). Decisions in [[rung102-decisions]].

**1. A cost trigger is fixed by counting the work, not by guessing the hot spot.** The first Part B build cost 6–12× the
dry point (trigger 3×). A per-thread census of trials and instants (`afterburner::take_census`) found THREE separate
causes, none of which I had predicted: deep bisections inside the duct solve, ~20 refused trials an instant marching up
through a band where low turbine pressure chokes the duct's ENTRY, and the shipped speed search's own march-in (19–37
instants a point). Each fix was re-measured by the same census. Measure the trigger ON THE PRODUCT'S PATH too: the solve
was under 3× while the page's reply (which also solves the dry readout) was not.

**Why:** a micro-benchmark of one function ranks per-call cost, not how many calls a search makes; the count was the
missing half.
**How to apply:** when a cost trigger fires, add a cheap call counter before optimizing, and time the user-facing call,
not only the solver.

**2. Every range in a write-up comes from a printed measurement.** Writing Part B's spec I put two ranges from memory —
"shaft slows to 65–85 %", "flame holders cost 0.4–0.6 %" — and both were wrong when measured (72–86 %, 0.4–1.1 %).
Checking each headline number against the probe output caught them before commit.

**3. A stated direction is checked at the cells it was stated for.** The anchor said Part B's held point would be less
severe than G4's probe; I first compared Part B's DIFFERENT flights to G4's and wrote "wrong in sign". At G4's own flight
and throttle the direction held. Compare like with like before declaring a miss.

**4. A scratch copy of a memoising gas starts empty.** Freezing per-trial mixtures on clones (to dodge the per-`far`
guard) crashed the equilibrium gas until each clone re-froze station 4 — Part A's own code already did this; the copy
in Part B forgot.

Related: [[rung96-inferred-thrust]] (measure a named risk even on a hit), [[instrument-fed-by-what-it-certifies]].
