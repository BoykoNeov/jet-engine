---
name: rust-port-guessed-census-bars
description: "Phase 4 guessed five census-count bars and all five were wrong; four times the shortfall was the PHYSICS, not a defect"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 519adbe9-fbbb-49f5-9712-29d4dd7fed36
  modified: 2026-08-12T18:11:51.077Z
---

Across phase 4's three slices I wrote five distinct-value count bars into oracle dumps
(`assert len(vals) >= N`). **All five were wrong**, and in four of them the shortfall was a real
structural property the guess was hiding:

* a lumped bar over the recombination clock's three kill-test arms — the arm with the density
  pinned MUST collapse, because pinning it removes the clock's only pressure dependence;
* a per-arm bar that assumed 30 distinct values — two grid cells shared the same `p/T` ratio, so
  the count rode on a floating-point coincidence;
* `>= 24` sonic-throat roots — the throat temperature is PRESSURE-INDEPENDENT, so a 6×4 grid holds
  six roots, which is the same property rung 31's `choked_mfp` is built on, one rung early;
* a `p*` count of 24 — structurally 4, measured 19 by rounding accident, so it was DROPPED rather
  than pinned;
* a fixed ambient-pressure ladder — impossible at some design points, because the total pressure
  moves with the design point.

**Why:** the measure-before-registering rule was already known and being followed for *bars on
physical quantities* — the probes ran first, the tolerances were measured. It kept failing in one
specific place: **counts invented while authoring the gate**. A count feels like bookkeeping rather
than a claim, so it gets typed rather than measured; and when it is wrong it surfaces as a failing
test rather than a wrong number, which makes it feel harmless. It is not — each of these was a
statement about the physics that nobody had checked.

**How to apply:** a census bar is a measurement, so measure it. Run the dump once with the counts
PRINTED and no assertion, read them, then write them down with the reason each holds. Prefer
per-arm counts over one lumped bar — a lumped bar hides exactly the structure worth knowing. And
when a count turns out to rest on a floating-point coincidence rather than a structural fact,
**drop it instead of pinning it**: the values are already gated at bit-equality, and a coincidence
bar only adds fragility across interpreters. Related: [[rust-port-measure-before-registering]],
[[rust-port-documented-gate-that-doesnt-exist]], [[golden-gate-slice7]].
