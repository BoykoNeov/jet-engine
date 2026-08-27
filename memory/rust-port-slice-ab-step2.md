---
name: rust-port-slice-ab-step2
description: "Slice AB (rung 69) step 2 — the smoke's first gate was refuted by the port it was written to check, and the step-1 gate it sat beside went half-vacuous the moment the bodies landed"
metadata: 
  node_type: memory
  type: project
  originSessionId: 55c1cab5-f4f9-40dd-88b9-a2499798e990
  modified: 2026-08-27T18:33:29.655Z
---

Slice AB step 2 of the Rust port shipped `src/reference_split.rs` at 1 641 lines against 713
Python (2.30x, 9 % above the pre-flight's own estimate): the nine swapped cell bodies,
`_cubic_roots_c`, `_invariants` and the six readers, plus `Census69` and a new
`tests/slice_ab_smoke.rs` (5 gates). Full Rust gate 126 binaries / 1 216 passed / 0 failed.
Plan record at § 5.26.2.

**THE SMOKE FAILED ON ITS FIRST RUN AND THE DEFECT WAS IN MY GATE, NOT THE PORT.** I asserted that
no cell could take its rung-68 reduce arm, "because the incidence stator is armed on every rig
here" — and got 985 against 0. The reader builds TWO rigs; that is its whole method (the two
references differenced on ONE trajectory), and the `phi` rig is a rung-68 machine *by arming*, so
its cells MUST reduce. Third instance in this slice of one shape:
**a predicate typed from a sentence about the rung instead of measured against the reader**
(see [[rust-port-slice-ab-step1]] (f), (g)). **How to apply:** when an assertion's justification is
a sentence you could have written before reading the code, run it first and read the number.

The replacement is derivable from the reader's STRUCTURE rather than from its output, which is what
makes it a gate and not a golden: march-only cells at exactly zero (only one rig is marched),
`manifold_parent == n_sampled`, `solve_parent == 5 * n_sampled` (one manifold solve plus the four
arms of a central difference). A cell wired to the parent unconditionally passes every reduce gate
in the slice and fails there.

**A STEP-1 GATE CAN GO VACUOUS WHEN THE NEXT STEP LANDS, AND NOTHING WARNS YOU.** The
`triple_laws`-is-inherited gate asserted *"the call does not panic"* — real only while nine slots
panicked. Once ported, nothing in the crate emits that message. Re-measured into a positive form:
rung 68's body is LAZY, so a rung-69 machine's call must dispatch **none** of rung 69's cells, and
`Census69` reads all-zero. **How to apply:** at every step, re-ask of each inherited gate what
would now have to be true for it to fail — a step that removes a defect can remove its detector's
only input. Related: [[rust-port-ported-test-vacuity]].

**AND THE ADVISOR FOUND A THIRD, WHICH FIXING EXPOSED A FOURTH — FOUR TYPED PREDICATES IN TWO
STEPS.** The write-up credited a one-point damping grid with *"equality at A = z, floor
bandwidth-independent"*; one point can show neither (bandwidth-independence needs two bandwidths,
and `A/z` was 2 there, never 1). Widening to Python's own six-point default fixed that AND gave
`tightest` more than one live row, without which its first-minimum rule was untested. **Then the
assertion written for the widened grid was typed from PYTHON's own docstring** — *"the grid straddles
A/z = 1"* — and went red: the emitted set is {1, 1, 2, 2.75, 2.75, 4}, so it TOUCHES the minimiser
and never goes below. A shipped claim measured rather than inherited. **How to apply:** widening a
grid to fix an overclaim does not make the next sentence about that grid measured either.

**A LIBRARY CALL CAN LOOK LIKE A PLATFORM EXPOSURE AND NOT BE ONE.** `cmath.sqrt(complex(d, 0.0))`
appears to drag `hypot` into the arithmetic; CPython's own algorithm makes every step exact for a
zero imaginary part, so the result is plain `sqrt(|d|)` and no `hypot` survives. Derived from the
algorithm, not assumed. Two spellings do matter: the zero case is CPython's early return (a naive
`(-0.0).sqrt()` returns `-0.0` and flips a root's sign), and Python promotes `-p` to
`complex(-p, 0.0)` before subtracting, so the third root's imaginary part is `0.0 - rt.im` — `+0.0`
on the real branch where the idiomatic `-rt.im` gives `-0.0`. The one real exposure left is
`hypot` inside `abs()` on a genuinely complex root; registered for step 4's oracle rather than
assumed either way.

Also: eight reduce arms CALL rung 68's body (six of its cells made `pub(crate)`) rather than
re-spelling it — a re-transcribed arm is a second copy nothing compares.

Related: [[rust-port-slice-ab-preflight]], [[rust-port-slice-aa-steps2345]],
[[rust-port-measure-before-registering]].
