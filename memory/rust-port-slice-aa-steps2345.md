---
name: rust-port-slice-aa-steps2345
description: "Slice AA (rung 68) steps 2-5 — a pre-registered exemption wrong in BOTH directions, and three dispatch gates whose own aim was the defect"
metadata: 
  node_type: memory
  type: project
  originSessionId: 93497d43-ebb8-49c9-848c-d51d8632150f
  modified: 2026-08-27T13:57:37.268Z
---

Slice AA's port shipped: `three_loop.rs` 2 302 lines, 44 test functions across four files, the
oracle bit-exact at **12 084 keys** on PyPy with a four-key CPython exemption.

**The oracle earned its keep on run one, and the 22 ported gates could not have.** One key was
wrong — Python's `max(gen, default=0.0)` ported as `fold(0.0, max)`, and `default=` fires only on
an EMPTY generator where a seeded fold CLAMPS. Every ported gate was green with it, because the
only assertion that reads that key belongs to **rung 69**, one slice ahead. *A key written for the
next rung is defended by nothing in this one — which is what a value oracle is for.*

**An exemption list can be wrong in BOTH directions, and the over-listing is the dangerous half.**
P3 said the CPython drift was "confined to `ic_family`"; the draft list had eleven names, the
measurement has four, and only three are that reader. Eight listed names do NOT drift — so the
oracle would have **passed while asserting nothing about them**. [[rust-port-slice-z-step4]] is the
same defect mirrored (two typed, eight measured). The fourth key was a march value with none of the
arithmetic I blamed: two Python builds' root-finder takes 8 vs 7 iterations from *bit-identical*
inputs, i.e. the port's long-known interpreter difference deep in the plant, surfacing only where a
slow actuator's state is small enough to record it.

**Ask of a dispatch gate WHERE ITS INJECTION LANDS.** Three of nine failed on first aim, all three
my gate's fault: one predicted a fallback where the real answer is a crash (two cells I assumed
interchangeable are not — one reads its field directly), and two applied the injection to an object
the reader rebuilds through a sibling constructor that installs the SHIPPED tables. The second is
faithful to Python and means a whole reader family is blind to those two cells.
[[rust-port-slice-w-step3]] again.

**A DEFENCE WITH NO READER IN THE SLICE IS THE RECURRING SHAPE, and it appeared four times.**
`v_max_used` (the oracle caught it); `v_at_point`/`ic_at_point`'s refusals, called only on
rung-68 points everywhere so a `0.0` return would pass all 45 tests; `round12`, whose one consumer
returns **1** and is satisfied by any rounding; and `Census68` itself, written at step 2 and read
by nothing until the last hour. Each was found by ASKING what reads a thing, never by a failure.

And two closing self-inflicted ones. `v_of`'s live arm is **dead on the shipped grid** — 0 reads,
where I asserted `> 0` (slice X's `b_of` precedent: ship it, record the zero, gate it by hand). And
the tally beside that gate said **3** where the answer is **2**, with *"the two LP ones"* written in
the same sentence — this phase's most-repeated defect, now spelled `1 + 1 + 0`.

I also nearly shipped **21 of 22** ported gates: the missing one PROVES a cell had to exist, nothing
failed, and it surfaced only by counting Python's `def test_` against cargo's own `--list`. See
[[rust-port-slice-aa-step1]] for the same slice's step-1 lessons.
