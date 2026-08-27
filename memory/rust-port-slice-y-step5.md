---
name: rust-port-slice-y-step5
description: "Slice Y step 5 — mutating my own gates found one testing half of what it named: a save-and-restore guard's SET and RESTORE halves were owned by two different files and neither caught both"
metadata:
  node_type: memory
  type: project
---

The step-5 gate file covers the three defects a 35,994-key oracle cannot see. Nine injections, each
re-run against BOTH files, and every prediction held **except one** — which is the whole value of
having run them.

**A GATE CAN TEST HALF OF WHAT ITS NAME CLAIMS.** The manufactured-nest gate for a
save-and-restore-previous guard asserted that after an inner scope the outer value is back. A guard
that saves and restores the outer value passes that **no matter what it SETS** — so corrupting the
cell to set `None` instead of the value it was handed walked straight through. The oracle caught
that half (1,280 keys) and the gate file caught the other; **neither caught both**, and the split
was invisible until the injection was run. Repaired by also asserting the inner scope's own output
carries the value it was given.

**ASKING A READER FOR ITS DEGENERATE CASE FOUND A REAL BUG THE ORACLE COULD NOT.** Python's
`max(a, b, c)` is *not* `a.max(b).max(c)`: `f64::max` discards a NaN operand, while Python holds
the first element and replaces it only on a strict `>`, so a NaN in the **first** position survives
and one in any later position does not. The one reduction in the rung that can be NaN has that
value first. "It propagates" was itself a wrong guess — the three positions had to be run in an
interpreter.

**AND A DECLARED LIMIT IS NOT WORTH KEEPING IF THE PROJECT ALREADY OWNS THE INSTRUMENT THAT
REMOVES IT.** No reachable input produces the NaN, so I first gated the FUNCTION, left the call
site uncovered, and wrote the hole into the gate's doc comment — honest, and one step short. The
project's own convention already covers "this branch is not reachable": read the source at compile
time and count the occurrences. Two counted assertions on the call-site text closed it.

**Why:** a green gate proves the code passes the gate, never that the gate watches what its name
says. Only mutation separates the two.

**How to apply:** after writing a manufactured-bug gate, inject the very defect it was written for
and confirm it reddens. When a prediction fails, do not just fix the injection list — ask which
half of the property the gate was actually reading, because the other half is being covered by
accident or not at all. And when two files both look like they cover something, check that one of
them really does; "the other one has it" is how a hole survives two reviews.

Related: [[rust-port-slice-y-step4]], [[rust-port-slice-w-step5]], [[rust-port-slice-v-step5]],
[[rust-port-slice-u-step4]], [[rust-port-copy-vs-rederivation]].
