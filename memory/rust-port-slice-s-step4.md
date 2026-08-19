---
name: rust-port-slice-s-step4
description: "An instrument's own docstring is not evidence about what it measured — a probe claiming to run the suites' grids ran its own, and four registered numbers came off it"
metadata: 
  node_type: memory
  type: project
  originSessionId: 935bee28-045a-4b17-a752-beed5aca0e84
  modified: 2026-08-19T08:21:41.091Z
---

Slice S step 4 of the Rust port (2026-08-19) — `oracle/dump_fuel_transient.py` +
`tests/fuel_transient_oracle.rs`, 5 tests / 0 failed, 4 671 + 1 133 keys bit-exact against PyPy on
the first run that compared them, name diff 736 → 741 with 0 removals, zero `src/` edits.

**The lesson: AN INSTRUMENT'S OWN DOCSTRING IS NOT EVIDENCE ABOUT WHAT IT MEASURED.** The slice's
probe 2 header says it runs *"rungs 43 and 45's OWN grids"*. It runs a cross-product of its own
choosing, and every census number the plan registered for this step came off it. Writing the real
grid first and reading the census out **before any gate was written** killed four registered
numbers: 162 marches → 143, 21 on the rounding tie → 52, the three-arm bracket wall
24 033/200 193/3 663 → 1 398/228 801/1 210, and "0 CPG float keys move under CPython" → 15. This is
the same family as [[rust-port-guessed-census-bars]] and [[rust-port-slice-n-step4]], but sharper:
the earlier ones were bars I guessed, this one was a bar I had *measured* — on the wrong grid, and
the instrument said otherwise in prose.

**How to apply:** before porting any registered count, re-derive the grid it was measured on from
the probe's CODE, not its header, and re-measure on the grid the step will actually run. A census
is a property of the grid — that phrase now has five instances in this slice alone, the fifth being
that a swallowed-failure split (38/8) turned out to be a property of the *cell* (39/7 at one fuel
flow, 40/7 at another) rather than of the gas everyone had attributed it to.

**A rare arm can be rare because it lives in ONE cell.** All 2 608 non-map bracket-wall hits came
from a single shape in a single gate — the one whose LP map is `flat()`, so the map ceiling never
binds. That is why the census is emitted per CELL where the finding is, not per section: a section
total lets one shape's 228 801 hits bury another's 1 301. Related: [[rust-port-slice-l-step3]].

**A comparator that runs and is not read costs a gate.** My localization test ran the full sweep and
dropped its `Cmp` without calling `finish()` — every value diff and the never-compared half silently
discarded. Third instance of that shape inside this one slice, in a file written by the person who
wrote the first two down ([[rust-port-slice-s-step3]]). And section F's length check recomputed the
expected point count with the *same* rounding expression the port could get wrong, so both sides
moved together and it could not fail — a self-referential check reads exactly like a passing one.

**A refuted prediction is worth more when it names a class.** The 15 CPython movers are all
`collapse_exponent`'s scored curve: `rho**q` and `log`, i.e. libm, which a port shares with neither
interpreter. Everything through the plant stayed bit-identical, so the prediction's intent survived
and only its letter died — and the exempt class was declared wider than the measurement, because
what makes a key exempt is *being* a fractional-power/log composite. See
[[rust-port-slice-r-step4]] for the iteration-count class this sits beside.
