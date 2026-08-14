---
name: rust-port-shape-keys
description: "Dump a finding's ARGMAX as its own oracle key — the peak's value drifts between interpreters while its location does not"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 9964211f-4436-4198-9e01-847b54b09afa
  modified: 2026-08-14T17:01:03.596Z
---

Phase 3 slice A (rungs 7/8/9/19) added **shape keys** to the oracle: for any finding whose
claim is a LOCATION rather than a value, dump the argmax as its own key beside the curve.

Rung 9's claim is "EI_NO peaks near φ≈0.95". Measured: CPython and PyPy **disagree on the peak
value in the last bit and agree on the peak location exactly.** A value-only gate would have
reported that as a deviation to adjudicate; the argmax key says the finding did not move. That
distinction is the whole content of the port plan's "a tolerance is not a valid substitute"
register, and until slice A it was an argument rather than a measurement.

Keep the argmax grid **coarse enough that the peak sits several steps clear of its neighbours**
(Δφ=0.01 here). A fine grid makes the argmax a coin-flip between adjacent cells and converts a
real detector into a flaky test.

**Why:** the port's bar is bit-equality, and it held (1790/1790 vs PyPy). But the point of the
shape key is what happens when it eventually does not: a location claim needs an instrument
that can distinguish "a number moved" from "the finding moved", and that instrument has to
exist BEFORE the bar slips, not be invented during the adjudication.

**How to apply:** every remaining port slice with a location claim gets an argmax key — rungs
12, 22 and 24 above all ("the minimum pinned AT `C_opt`", "`C_opt` EMERGES as an output"). The
same reasoning applies outside the port: when a rung's headline is *where* an extremum sits,
the test should read the location, not the value at it.

**A second lesson from the same slice, opposite direction:** I pre-registered rung 8's
split-independence (α cancels from the mix-out balance) as a BIT-equality and the gate refuted
it. α cancels algebraically, but `α·far_p = far_ov` holds only to rounding, so a bisection's
final sign test can land on the other side. The spread is `2500/2**32` EXACTLY — one quantum of
the solver's own bracket, not drift. The right assertion was one quantum, spelled as the
arithmetic. **An analytic cancellation read through a solver is exact to the solver's grid, not
to the bit** — expect this wherever a later rung asserts one.

**BOUNDED by slice L (2026-08-14) — the headline holds only for an argmax over a GRID.** Rung
41's `flow_coefficient_turn` golden-sections to an interior maximum over a CONTINUUM, and there
the two interpreters agree on the peak VALUE to 4.07e-11 and on its LOCATION only to 7.39e-6 —
five orders the other way. The grid was doing the work all along: it quantises the answer so both
interpreters snap to one node, and the "keep it coarse" advice above is that mechanism stated
without knowing it. Take a continuum argmax's location as √(objective noise)-conditioned and give
it its own, wider, MEASURED bar. See [[rust-port-slice-l-step4]].

Related: [[rust-port-arithmetic-is-pypy]], [[rust-port-power-spelling]], [[rust-port-decided]].
