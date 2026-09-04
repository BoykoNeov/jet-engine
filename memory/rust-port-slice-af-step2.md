---
name: rust-port-slice-af-step2
description: "Slice AF step 2 — a rig constant copied from a sibling test file sat inside a shipped refusal, and the flatness gate it broke had been passing between two SATURATED ZEROS; only the assert_ne control beside it could see"
metadata: 
  node_type: memory
  type: project
  originSessionId: 45b4785c-72e1-49cc-869e-fc23f7bd0d5d
  modified: 2026-09-04T12:40:41.721Z
---

Slice AF step 2 (rung 74, `DemandCoordinateTransient`). Plan § 5.30.2. The six single-definer
demand laws — `_applied_demand`, `_demand_target`, `_demand_reference`, `_demand_tau`,
`_demand_authority`, `_demand_laws`. See [[rust-port-status]], [[rust-port-slice-af-step1]],
[[rust-port-slice-t-step1]], [[instrument-fed-by-what-it-certifies]].

**THE LESSON: the standing question — *what supplies the value under test?* — has a companion that
no pre-flight had named: *what supplies the POINT the test is DRIVEN AT?*** The gates took their
rig point `(a, h, mf_sched) = (1.0, 1.0, 0.02)` from the sibling step-1 file, where it works. It
does not work here, because step 1's one call armed a DIFFERENT cap through a DIFFERENT path — an
accel schedule, where these laws arm the surge floor. Through the floor at the design speed pair
the cap solve cannot bracket at all, and the rung's own shipped guard fires. **A rig constant
inherited from a sibling file is a claim about the plant, and it is only as true as the call that
file happened to make.**

**AND THE SHARPER HALF IS THE GATE THAT DID NOT ABORT.** The min-select flatness gate asserts the
valve law returns the same position, bit for bit, at two points sharing an applied demand. **That
equality HELD — between `0.0` and `0.0`.** At the copied point the valve was saturated on its stop
at every demand probed, so an equality gate certified the rung's headline from an actuator that was
not solving. Only the `assert_ne` control beside it failed, reading `left: 0.0, right: 0.0`.

That is [[rust-port-slice-t-step1]]'s class — *an exact zero blinds its own gate* — **in a new
shape: the zero is an ACTUATOR'S STOP, not an arithmetic identity.** Nothing in the arithmetic
hints at it, and a `min` over two saturated solves is flat for the same reason a constant function
is. A permanent `assert_ne!(value, 0.0)` now stands in FRONT of the equality.

**AND THE MECHANISM IS NARROWER THAN *the point was wrong*, which is what makes the lesson
recursive.** The point was TRUE — for a different ARMING than the code under test exercises. So the
question is not *is this constant good?* but *which arming was it measured under, and is that
mine?* **The point this step then MEASURED is, at the next step, an inherited constant with exactly
the same property**, because the next step arms a third thing again. A measured constant is only
measured for the arming that measured it; carry it forward as a starting guess, never as a warrant.

**How to apply it:** for any gate whose content is *these two differ* or *these two agree*, ask
what the operating point supplies before asking what the code does — and put the off-the-stop /
non-degenerate check in front of the comparison, not beside it. Fix it by MEASURING a point (a
sweep over the grid, printing every law's return and regime) rather than by nudging the constant
until the red goes away.

## A predicted survivor that had been nominated a must-kill, settled by algebra before the sweep

The advisor called *the regime label read off the LATCHED target instead of the cap* a must-KILL.
It is arithmetically **inert**: under two of the three coordinate tags the target IS the cap, and
under the third `min(mf_sched, cap) < mf_sched` and `cap < mf_sched` are the same predicate. The
port keeps the source's spelling, and the gate asserts the INVARIANCE that makes the mutation inert
— with the label asserted non-dormant so the three-way agreement is a measurement rather than three
equal nothings. **Recording a predicted survivor with its proof is step 1's lesson paid forward**:
there, a correct survivor was defended by a code comment that was false, and the comment shipped.

## Three instrument defects, and one of them was step 1's own rule paying off

* **The dispatch gate's own arguments put the reference on its identity branch**, so its value
  assertion would have checked a formula against a body that never ran it. Caught by reading the
  arguments against the three branches before running.
* **A patch needle silently lost a line continuation.** Written as a shell heredoc, `\\` collapsed
  to `\` and Python then read it as a line continuation, so the needle lost a backslash AND a
  newline. It reported `PATCH-MISS: 0 != 1` and aborted before writing — **the count guard step 1
  installed doing exactly its job, one step after the lesson that produced it.** Repaired by
  SPLICING the text out of the file rather than retyping it. Belongs with
  [[windows-tooling-file-hazards]].
* `3.0 * 0.05` is `0.15000000000000002`, so an exact-equality assertion failed on a true statement
  about the rig.

## The module header that reports step 1's own finding kept the count step 1 corrected

Step 1 measured four width tripwires where the record said two, and renamed all four in the two
files that hold them — but not in the header of the module whose own step reports the finding. The
slice's recurring lesson (*check the ROW, not just whether the correction exists somewhere*)
landing on the file that states it, one step later.

## Two more, both about a warrant rather than a bug

* **A census that stops at the method names misses the HELPERS the body reaches.** One law calls a
  solver DIRECTLY while its mirror goes through the dispatch table, and that asymmetry arrived by
  copying the parent rung's spelling. Copying is not a warrant: **a frozen dispatch is invisible at
  the rung that owns it**, because that rung is the only machine the slice instantiates. Censusing
  the four helpers separately showed the direct calls are right — the one dispatched helper is the
  only one with two definers — so the finding is that the WARRANT was missing, not that the code
  was wrong. Census the callees, not just the cells.
* **A gate's exit code does not survive a pipe.** The prior step rested its whole claim on
  *"`cargo test`, exit 0 — because cargo returns non-zero if any target fails"*. Piping that
  through `grep` records **grep's** status instead, so the line that made the claim load-bearing
  reports the filter's health. Run the gate unpiped, or read the totals from the summed per-target
  lines — never both a pipe and an exit-code claim.
