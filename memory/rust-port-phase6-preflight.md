---
name: rust-port-phase6-preflight
description: "Phase 6's pre-flight — the same census run in the OPPOSITE direction found a scoping bug phase 5's set could not have exposed, and the phase's named arithmetic risk was refuted by measurement"
metadata: 
  node_type: memory
  type: project
  originSessionId: cc5d95b2-526a-4146-b929-a889adee8a25
  modified: 2026-08-17T17:49:04.972Z
---

Phase 6 (the 15 transient rungs: 34–37, 40, 43–52) was **authorised 2026-08-17** — the fresh
authorisation [[rust-port-decided]] said was owed. Its pre-flight re-ran § 5.3's inheritance
census with the roles swapped: phase 6 as the ANCESTOR side, all 58 classes open on the descendant
side, because every phase-7 class descends from `TwoSpoolFuelTransient`. Result: **six names cross
into phase 7**, all on the two-spool chain, so slices P and Q need no hook table at all.

**THE CENSUS'S FIRST RUN WAS WRONG IN A WAY PHASE 5's COULD NOT BE.** It asked *"is this name
redefined by any class descending from ANY phase-6 class"* and returned 32 names. But
`SpoolTransient` (single-spool) and `TwoSpoolFuelTransient` (two-spool) are **siblings**, so a
same-named method on one is not an override of the other — 26 of the 32 were that. Phase 5's set
had **one root**, so the bug had nowhere to appear. **A census over a set with two roots needs the
override scoped to the ancestor that makes the CALL, not to the phase.** Generalises
[[rust-port-phase5-preflight]]: widening the sweep is necessary and not sufficient — the widened
sweep needs its own predicate re-checked.

**THE PHASE'S NAMED ARITHMETIC RISK WAS REFUTED BY MEASUREMENT, AND THE REFUTATION RELOCATED IT.**
The one shape that would break the phase table's *"grinding but low-risk"* label is a `min`-select
inside an RK4 march: a flipped argmin does not drift, it changes which limiter is authoritative
and propagates for every remaining step. `integrate_fuel`'s `der` collects up to three caps and
applies `min(caps)`, so the plan was to dump an argmin index. Measured over **78 cases** (ramp
rate × redline × accel margin × φ floor): **not one evaluation ever had two live caps** — ~600 000
selections, zero contested. The legs never contend, so there is no argmin to flip. The discrete
content is one level down, in the two **arming predicates**. *Measure the mechanism before
building the instrument for it.*

**AND A 100 % CPython-VS-PyPy AGREEMENT WAS RECORDED AS NOT-COVERAGE.** The follow-up dumped both
arming sequences and full trajectory bits and got **9 376 keys, 100 % identical** — next to slice
G's 8.0 % and slice K's 46.3 % that reads as a clean result and is not one: the probe runs a CPG
gas, whose properties are closed-form, so the two interpreters have almost nothing to disagree
about. **The CPython arm is only a detector on the reacting gas.** Written into the plan at the
number, so it cannot later be quoted as coverage it does not have —
[[rust-port-oracle-cannot-see-a-missing-gate]] applied to the instrument instead of the gate.

**Also settled at pre-flight rather than discovered at slice T:** rungs 46–52 are keyword arguments
on ONE method, not seven classes, so `integrate_fuel` is ported ENTIRE and only the GATES are
sliced; the slice order P→Q,R→S→T,U is FORCED by `_degenerate` (the two-spool transient
constructs a single-spool one and delegates to it); and the `4257–4506` object block spans two
phases — `IncidenceLimiter` is rung **60**, sandwiched between rung-49 and rung-51/52 objects.
That last one is [[rust-port-slice-k]]'s lesson caught before it cost anything.
