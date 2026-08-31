---
name: rust-port-slice-ac-step5
description: "A gated condition measured on the READERS was falsified by the shipped TEST SUITE one file over, and the ported gate that drove it could not have told the wrong answer from the right one"
metadata: 
  node_type: memory
  type: project
  originSessionId: 60b9167f-1803-47f0-b424-e812767b6120
  modified: 2026-08-31T14:00:14.012Z
---

Slice AC step 5 (rungs 70/71 Rust port) shipped `rust/tests/rung71.rs` — 30 gates, 1:1 with
`tests/test_rung71.py`'s 30. Two failed on first run and **neither was a transcription slip**.

**THE LESSON: a condition measured over the READERS is not a condition about the RUNG, and the
shipped test suite lives in the second set.** Step 3 measured `p` real on 18 of 18 calls of rung
70's `_zeta_pair` from its own readers, registered `p.im == 0` as a *gated condition*, and shipped
it as an `assert!`. Rung 71's damping gate drives that same function on a CONSTRUCTED spectrum
where `p = 4462 + 4947i` — and the port had already published the resulting number, `1.279`, in a
sibling doc comment at the same step. It printed a value its own `sqrt` could not produce.

**AND THE SECOND HALF IS THE ONE TO CARRY: the `assert!` caught what the ported gate could not.**
Driven past its own assertion the real-only spelling returns `1.624` where Python returns `1.278`,
and the shipped gate asks `|zeta − ring| > 0.5` — the two candidates are `0.608` and `0.954`, the
same side of a one-sided bar. A port that had shipped the fast path *without* the assertion would
have been green and wrong. Step 4 found gates too weak to catch injections
([[rust-port-slice-ac-step4]]); this is the mirror.

**A count I had backwards until I measured it.** Intercepting all 96 shipped `cmath.sqrt(p)`
calls: **90 carry `im == -0.0`**, not `+0.0` as I first wrote. That matters because CPython's
`copysign(s, z.imag)` flips the sign of a NON-zero component in the `re < 0` arm — harmless here
only because `p.re < 0` on **0** of the 90. `sqrt` differs bit-wise on 91 calls; the returned value
differs on 1.

**The other failure: a census counter the thing being measured never touches.** The gate asserted
`Census70::triple_laws_gov > 0` on a march, on the theory that rung 70's `_triple_laws` runs at
every step. It does not — the five-state integrator calls `solve_v`, and `triple_laws` is a
READER-side cell. All six counters read 0 on a march that had plainly run.
[[rust-port-slice-aa-steps2345]]'s *ask what reads a thing* in its cheapest form. Rewritten around
a counter with its own control: run the REDUCE arm first so the counter is shown to move, then the
armed arm and assert it does not.

**THE SWEEP ADDED THREE MORE, ALL ABOUT WHAT A `MISS` MEANS.** Ten injections, both binaries:
7 caught, 3 missed. (i) Every miss must be shown able to move something — j01 moves 3 of 6 clock
arms and shifts `arms_below_r69` 2 -> 1, which the gate absorbs because its bar is `>= 1`; j05
flips `kept.stator` from +0.055 to -0.006, which a one-sided `< 0.25` upper bound accepts
maximally. (ii) **j10 moves NOTHING**, and that is a different finding: `round10` is a DEFENCE WITH
NO READER on this plant, not a gate hole. The two call for opposite responses, so do not file them
together. (iii) The sweep labelled a build-lock contention failure *"did not build"*; re-run by
hand with the output kept, that injection compiles and is caught. **A driver that greps for the
line it hopes to find cannot diagnose its own failure** — record the exit code and keep the output.

**And the two analogues step 4 handed forward SPLIT**, which is the useful half: the clock-order
blindness misses in both languages at both rungs (an inherited family property), while the same
`joint`-predicate widening that missed at rung 70 is CAUGHT at rung 71 in both languages — because
rung 71's bar is two-sided where rung 70's were one-sided lower bounds. Same defect, same reader
shape, opposite verdict, **from the bar**.

**AND A THIRD DEFECT WAS MINE, CAUGHT BY THE ADVISOR BEFORE IT SHIPPED.** The ported
introspection gate read the WHOLE module with `include_str!` and asserted `count() == 1`, where
Python scopes to one method's source and uses a bare `in`. Measured: with the guard deleted and a
doc comment quoting it, the whole-module form **PASSES ON A DELETED GUARD**. That is step 4's
doc-comment-`#[test]` finding running in its dangerous direction — a stray copy satisfying an
assertion on behalf of code that no longer exists. Scoped to the function body, it fails as it
must. **A source-text gate must be scoped at least as tightly as the Python it ports**, and the
scoping is what earns a count bar, not the other way round.

**AND A FOURTH, IN THE REPAIR ITSELF.** The invariant written to keep the `csqrt` replacement
honest drove `im = +0.0` — the sign **5** of 96 shipped calls carry — while the `-0.0` that **90**
carry got one input asserted only to diverge. **An instrument written for a measured population
must be pointed at the common case, not the tidy one.** Extended in place; its negative-real
control then failed, and the CONTROL was wrong: with a purely real `s` the divergence cannot reach
`.real` at all, so it needs a negative `p.re` AND a complex `s`.

**How to apply:** when a port ships an `assert!` justified by a measurement, write down what
population was measured. *Readers* and *the shipped suite* are different populations, and a
constructed-input gate is exactly the case a reader census cannot see. And before asserting a
counter, check what calls it — not whether it exists.
