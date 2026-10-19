---
name: rung102-afterburner
description: "Rung 102 (afterburner) process lessons — fix a fired cost trigger by COUNTING work (calls AND instants); write ranges from printed measurements; check a direction at its own cells; print a scored quantity dry before freezing (a dead coordinate)"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c72a2043-730f-42b0-8a5e-3b17edf7d8be
  modified: 2026-10-19T09:45:15.631Z
---

Rung 102, Parts A–C (2026-10-19). Decisions in [[rung102-decisions]].

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

**5. (Part C) A fired cost trigger can have a cause its planned fix cannot reach — count INSTANTS, not only solve cost.**
The held-nozzle HP-speed governor ran 3.76× (trigger 3×). The census showed 3.3× the engine instants (the governor riding
the lean wall), each only 1.28× dearer: no faster LP solve could get under 3×. One "speed-up" (bracketing from the unlit
root) was SLOWER — measured, removed. Disclosed as fired, with the census, instead of forcing a non-deterministic fix
(a history-dependent bracket would have broken the re-read's bit-for-bit match).

**6. (Part C) Print the scored quantity's DRY value on every scored cell before freezing.** C6/C7 predicted a speed-line
PRESSURE margin on the `flat-lp` map, where flat speed lines make it identically 0 — 16 misses by construction. The dry
pre-flight (`grid_dry`) checked the runs completed but never printed the margins, so the dead coordinate went unseen.

**7. (Part C) Shell:** in bash double quotes `\\$f` swallowed the backslash AND the expansion (`census$f.txt`), so a
`cmd start /b` replay read a missing file and wrote nothing — twice. Pass Windows programs forward-slash paths.

Related: [[rung96-inferred-thrust]] (measure a named risk even on a hit), [[instrument-fed-by-what-it-certifies]],
[[rung53-variable-stator]] (a margin is a distance — and a coordinate can be dead on a map).
