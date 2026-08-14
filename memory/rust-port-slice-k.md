---
name: rust-port-slice-k
description: "Slice K (rungs 38/39) — the phase table's own scope list had never been enumerated, and it both dropped a rung and double-counted one"
metadata: 
  node_type: memory
  type: project
  originSessionId: 475a500f-97bb-4f77-ac01-3592dc3d3a66
  modified: 2026-08-14T10:25:08.782Z
---

Phase 5 slice K of the Rust port (`docs/plans/todo-rust-port.md` § 5.7) covers rungs 38 and 39,
the two-spool matchers. Four process lessons, in the order they cost something.

**A SCOPE LIST IS A CLAIM ABOUT A SET, AND NOBODY HAD EVER COUNTED.** Enumerating rungs 1–84
across the plan's eight phase rows found rung 41 in **no** phase and rung 61 in **two**. Not
cosmetic: rung 41's code lives *inside* the class slice K ports, so an unassigned rung would have
been ported by accident or dropped by accident. The general form — audit the coverage of any list
that claims to partition something, by enumerating the thing it partitions. See
[[rust-port-decided]].

**THE HOOK'S JUSTIFICATION AND THE SLICE BOUNDARY WERE THE SAME QUESTION.** The pre-flight census
justified a virtual `match` by three call sites; checking which `def` encloses each showed all
three are rung-41 methods. So deferring rung 41 means shipping a hook with zero live call sites —
which is fine for a hook (its job is to exist when the overrider lands, cf. [[rust-port-slice-i]]'s
`solve_turbine`) and is exactly what [[rust-port-slice-j]] deleted for an *instrument* (whose job
is to fire). Keep the distinction; state which one you are shipping.

**"MEASURED INERT" IS A STATEMENT ABOUT THE SWEEP, NOT ABOUT THE FIELD.** Slice J left rung 34's
`l` off the ported `ComponentMap` having measured it bit-identical over 26 900 evaluations. Rung
39's own test shapes set it, and there the two spellings differ by 27–43 % **relative**. Before
reusing an earlier slice's "inert" finding, check whether the new consumer stays inside the band
that was swept. And adding a term to a function is the same hazard class as mis-spelling one:
float subtraction is not associative, so term ORDER is load-bearing and a value oracle cannot see
it — [[rust-port-slice-j]]'s lesson generalises from "gate squares" to "gate any change to the
function the oracle is blind to". Measured discriminability: reordering differs at 1 point in 48,
the pow-vs-multiply spelling at 1 in 4 012.

**A COUNT WITHOUT ITS GRID IS NOT A MEASUREMENT — AND THIS TIME THE BAD INSTRUMENT WAS MINE.**
My first probe grid (`Tt4 >= 900`) found rung 38's scope guard firing **zero** times, which reads
as "dead". On the dump grid it fires on 23 of 147 cells. Worse, I then wrote "exactly 20" into a
pre-registered prediction from a 126-cell probe while the dump grid had 147 — the advisor caught
it, and re-measuring gave 23. Read a census off the grid the gate will use, never off the probe's.
Same shape as [[rust-port-slice-i]]'s lesson, one level up.

**AN ASSERTION COPIED FORWARD FROM AN EARLIER SLICE IS A HYPOTHESIS, NOT A FACT.**
[[rust-port-slice-i]]'s gate asserts its pass-count flips are all on the equilibrium gas and calls
the instability "a property of the EQUILIBRIUM gas". Copied into slice K's gate it FAILED: 13 of
81 flips are on the thermally-perfect gas. The common factor is the ROUTE to a property — `tpg`
and `eq` both reach `cp` through an integral and a root-find, so last-bit arithmetic can tip an
unmeetable stopping rule; the closed-form gas flips nowhere. The failing copy is what produced
the finding, so copy earlier assertions forward *deliberately* and treat a failure as a
measurement. (Slice I's assertion still holds on its own grid; only its reason was wrong — the
[[rung28-coupled-no-march]] shape, applied to the port's own instruments.)

**A KEY NAMED FOR A DEFECT IT CANNOT DETECT — TWICE OVER, AND THE FIRST FIX WAS HALF A FIX.**
The gate's doc claimed two `*_passes_max` census keys would catch a `do`-while loop shape. They
are **maxima over the whole grid**, set by the shaped cells, so flipping every flat cell from 0 to
1 leaves them unmoved. The tell was visible in the dump itself — one loop got a `_min` and a
`_max`, the other only a `_max`. Adding `hp/lp_passes_min` fixed the *quantity* — but I put them
only in the **comparison** table, which asserts *Rust agrees with Python*: if BOTH sides ran the
loop `do`-while, both dump 1 and the gate passes clean. So the second half is an **ABSOLUTE**
assertion (`== 0`, plus a shaped-map vacuity guard proving the counter can move), verified by
MUTATING the shipped loop to `do`-while and watching it fail. Two lessons in one: check the
quantity would MOVE under the defect, then check the BAR is one the defect can't satisfy on both
sides — [[rust-port-measure-before-registering]]'s blind-to-a-shared-assumption trap arriving via
[[rust-port-documented-gate-that-doesnt-exist]]. And a hand-read of the dump is not a gate.

**A QUANTITY'S CLASS COMES FROM WHAT PRODUCES IT, NEVER FROM WHAT IT IS CALLED.** A key named
`bisect/n_solves_swept` was classed as a fixed count; it is a pass-count sum, so it inherits the
joint loop's interpreter-dependence and the CPython arm failed on a key that was behaving
correctly. Same family as [[rust-port-slice-j]]'s dead `debug_assert_eq!`: instruments need the
same scrutiny as the code they measure.

**One more, cheap to state:** a prediction that can be confirmed without checking anything is
worthless. P1's first draft said the HP efficiency loop's signature "carries no LP quantity" —
false on code already read (it takes `Tt25`). The narrow true claim is "no LP *efficiency* and no
LP *pressure ratio*", and the port makes it structural by keeping that loop a free function over
explicit scalars.
