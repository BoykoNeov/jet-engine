---
name: rust-port-ported-test-vacuity
description: "A test ported verbatim can become VACUOUS when the port's factorisation is better than the source's; and a location key sitting on a scheme boundary needs a COARSER grid, not a finer one"
metadata: 
  node_type: memory
  type: project
  originSessionId: 5e70803c-a526-46b5-908f-68437f30d895
  modified: 2026-08-12T14:00:48.591Z
---

Phase 3 slice C of the Rust port (rungs 13/15/16/18/21 — the mixture-fraction PDF family,
2026-08-12) landed at 100% bit-equality like every slice before it (2448/2448 vs PyPy), so again
the bit-count was not where the value was.

**A ported test can be VACUOUS in the target even though it is load-bearing in the source.** The
Python's rung-16 suite pins a hand-cached re-implementation against production — a real check
there. The Rust production code already splits that closure into an expensive bank plus a cheap
integration, so "helper matches production" would have compared a function to itself. Transcribing
it would have added a green test that measures nothing — the same family as
[[rung78-residual-gauge]]'s vacuity traps, arriving through the side door of faithful porting.
**When porting a test, ask what it could still FAIL for in the new code, not whether it passes.**
The replacements said something the Python cannot: that production equals term 1 plus an
independently built term 2, and that the bank really is independent of the other knob, bit-exactly.

**A location key can sit ON a scheme boundary, and the fix is a COARSER grid.** The quadrature
switches integration scheme at a computed threshold, and the extremum being detected sat one grid
cell away from it; across the switch the curve is not even locally monotone. Widening the grid so
the peak cell clears its neighbours by ~20% is what makes the detector real — a finer grid puts
the argmax inside the artifact. Also check WHICH side a tight margin falls on: at a second design
point the tight (3%) comparison was within one branch and the branch-straddling one had 33%, which
is the safe configuration, but only because it was looked at.

**Do not dump a location the source explicitly declines.** Rung 16's own docstring refuses to claim
which of two near-degenerate optima is lowest — it flips with resolution. An argmin key there would
fail for a reason that is not a defect. The advisor flagged this before any code was written; the
gate asserts the certified sublinearity RATIO instead (two values from the same sweep).

Two confirmations worth carrying forward:

- **Sweeping wider than the source's own gate paid a third consecutive time.** The dump's first run
  crashed inside the Python: the closure's mean-preservation guard has a resolution FLOOR the
  Python's own gate sits comfortably inside. Characterised rather than dodged, and the port now
  gates the guard FIRING as well as passing. Same lesson as [[rust-port-location-keys-refute]],
  different mechanism.
- **A pre-registered CONFIRMATION held**, at four spacings x two design points, with a companion
  assertion that every value moved so the equal locations are not a tautology. Slice B's refutation
  was the exception; budget for either.

**EXTENSION (slice E, 2026-08-12) — the port's OWN factorisation can create the vacuity, and that
is the case the rule misses.** Cases so far split two ways: the source's test meets the target's
type system (an unknown field is a compile error; a required argument cannot be `None`), which is
easy to spot; and the target's factorisation dissolves the thing being compared, which is not.
Slice E hit the second kind in a new place — replacing a monkey-patch with an injected function
removed the very branch the test existed to check, so the "faithful" version compared a call to
itself. **The rule has to be re-asked AFTER the target design is chosen, not only when reading the
source's test** — the pre-registration asserted the opposite for a day. The fix that ships keeps
the tautological comparison only as SETUP for an arm that can fail (feed the injected function a
DIFFERENT value and require the answer to move), which catches the real defect: a body that
ignores its injected argument. See [[rust-port-measure-before-registering]].

Related: [[rust-port-decided]], [[rust-port-shape-keys]], [[rust-port-arithmetic-is-pypy]],
[[rust-port-power-spelling]].
