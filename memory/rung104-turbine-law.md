---
name: rung104-turbine-law
description: "Rung 104 (2026-10-20), the LP turbine's flow law replacing rung 38's pin on A45 — process lessons: an equality-in-math must be equality-in-bits where readings sit on kinks; measure the old model's own floor before freezing a reproduce-design bar; re-ask a decision once its cost is measured"
metadata:
  node_type: memory
  type: project
  originSessionId: 608a4dc7-68b6-4be6-be9a-6ef698d34bc2
  modified: 2026-10-20T13:25:59.159Z
---

Rung 104 shipped the vane's flow law (ellipse default; two rows, one row, pin) — spec `docs/rung104-spec.md`, anchor
`docs/plans/rung104-anchor-turbine-law.md` (`0ddab63`). Headline: below choke the back-pressure slows BOTH shafts;
corrections to rungs 97, 102, 103 under every law. Decisions: [[rung104-decisions]].

**Lessons (process):**
1. **Math-equal is not bit-equal, and kinks amplify the difference.** On the perfect gas, choked, the law IS the pin in
   the algebra, but its own search landed ~1e−14 off — and 11 older rungs failed, because their readings sit on kinks
   (rung 58: a minimum over marched steps that several steps tie on; rung 66: a central difference straddling
   `max(0, …)` halved a product to exactly 0.5). The advisor's test decided it: march both, all instants choked, per-step
   difference ≤ 5e−15 ⇒ dust ⇒ a shortcut onto the pin's own solve under EXACT conditions (cpg, η = design, LP result on
   the pin's root bitwise), with a switch so R2 still compares the law's own search.
2. **A "reproduces the design" bar must be set after measuring the OLD model's own floor.** R3 froze 1e−12; the pin
   itself lands 4.8e−12 from its design run on the equilibrium gas — a MISS of the bar, recorded, not loosened.
3. **A user decision taken before its cost was visible gets re-asked with the measured cost.** "Retire the Python
   oracles" turned out to drop checks on many unmoved keys (measured after: 176 189 of 263 647 unmoved, 67 %; some arms
   99.97 %) and one oracle was the only guard on three bugs; asked again, the user chose to retire only the moved keys,
   by name. Lists are written by each arm itself; COUNT asserts the law moves are not in any list — they surface only
   when the arm runs past its value check, so run every arm to completion, not just the writer. A writer that lists
   "law vs pin" differences is blind to any engine the test builds WITHOUT the law switch (slice L's turn cells: law
   compared with law, 16 moved keys unlisted) — grep every build site for the switch before trusting such a list.
4. **The shortcut created its own edge bug**, found by rung 103's boundary gate: just below unchoke the subsonic root sits
   ON the choked one and the residual there read a hair negative. Older boundary gates are worth rerunning after any
   solver change.
5. **Witness tests (crash-map cases) move with the model**: a law change can dissolve the case a page sentence was driven
   by. Keep the old witness on the old setting (the code path still exists) and search for a new one.
