---
name: sandbox-slice3
description: "Web sandbox slice 3 'Fly it' SHIPPED 2026-10-07 — process lessons: a solver's stored constant was silently the DESIGN back-pressure (every shipped caller kept p0 fixed); a long-lived solver slowed 100x from an unbounded linear-scan memo; one panic message meant opposite things; a speed cap hid normal readings; a 'measured' timing was CPU contention"
metadata:
  node_type: memory
  type: project
  originSessionId: c96236c2-c5ed-4e6d-b683-063ff12b223e
  modified: 2026-10-06T21:55:19.135Z
---

Slice 3 of the web sandbox ([[sandbox-direction]], after [[sandbox-slice1]]) shipped 2026-10-07: the
"Fly it" mode of `docs/sandbox/turbojet-sandbox.html` — freeze the design's hardware, move throttle
and flight; one solver (rung 34 spool equilibrium + rung 36 stall margin). Plan § 10 of
`docs/plans/sandbox-plan.md` (§ 10.8 = what the build found). Also shipped first: hash-map memo
caches in `gas.rs` (b3a5e3b, gate bit-identical, reacting/Fork B ~10x faster).

**Process lessons.**
- **A first new CALLER can expose an assumption no test ever exercised.** The off-design matchers
  keep the design run's ambient as the nozzle back-pressure; every shipped caller flew at the design
  p0, so it was never wrong. Changing altitude with frozen hardware was new, and thin air failed
  outright. Ask what each stored constant MEANS when the caller varies something the old callers
  held fixed; pin "identical where they agree" AND "differs where they don't".
- **Time the thing as the product will use it.** A long-lived solver got slower point by point
  (an unbounded linear-scan memo keyed by fuel-air ratio); the same solve on a fresh solver was
  ~100x faster. And a low-priority wall-clock run measured CPU contention, not the solve: count
  thread cycles (`QueryThreadCycleTime`) and calibrate, or compare solvers as ratios.
- **One panic message can mean opposite things.** "Equilibrium does not bracket" covered below idle,
  overspeed, and a search cut short at low speed; the plain words read the search's ends. Classify
  failures by what the numbers in the message say, from a sweep, each branch driven by a test.
- **A blunt cap hides normal readings.** A 160 % speed cap stopped a gas-table crash but blanked the
  constant-flow stall margin just above design power. A fallible COPY of the read, pinned bit-equal
  to the original wherever that answers, was right ([[rust-port-copy-vs-rederivation]]).
- **A test's tolerance must match the physics it reads**: the browser test expected the
  standard-day pressure at 11 km on a non-standard day. Read back what the result itself reports.
- **Screenshots again found what checks missed**: propulsive efficiency 113.9 % (rung 2's
  kinetic-energy split with an underexpanded nozzle; labelled, not changed), a cut-off label, a
  grammar slip. Behaviour checks were green throughout ([[sandbox-slice1]]).
- **Backslash collapse re-hit 3x** — see [[windows-tooling-file-hazards]].
