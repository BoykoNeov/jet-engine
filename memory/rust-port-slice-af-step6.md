---
name: rust-port-slice-af-step6
description: Slice AF step 6 — a port that routes a plain attribute assignment through a virtual dispatch cell is bit-identical at the rung that introduces it and silently wrong at the rung that re-aims the field
metadata: 
  node_type: memory
  type: project
  originSessionId: e1ded472-8cd7-4892-82f2-b7a2f8271183
  modified: 2026-09-06T07:11:36.245Z
---

Slice AF step 6 (rung 74, the dispatch gates) found **four production call sites that dispatched a
write Python makes by plain assignment**. Two wrote the coordinate, two the reference.

**The lesson, and it is a sibling of *what supplies the value?*: ask whether the source line is a
DISPATCH at all.** A virtual call and a direct field write are the same function at the rung that
owns the cell, so every value instrument in the slice — 20 643 oracle keys on two interpreters, 17
ported gates, 42 more — was blind to it by construction. They stop being the same function the
moment a later rung overrides the setter to write a **different field**, and rung 79 does exactly
that. Rung 74's readers are single-definer, so rungs 75–84 inherit them, and
`tests/test_rung80.py:110` already calls one of them on a rung-80 object. The port would have
written the wrong carrier, left the coordinate at its class default, and marched the PARENT while
reporting this rung.

**The tell was already in the repo, and it was cheaper than any argument from first principles.**
The identical Python construct was spelled three different ways inside one slice: two shipped
functions wrote the field directly, a test fixture wrote it directly *and said in its own doc
comment that routing it through the table would test a different line* — and the two production
pins routed it through the table. **Look for one construct with two spellings before reasoning
about which spelling is right.** The crate's rule (*dispatch the setter iff a later rung re-aims
the field*) was correct and merely OVER-APPLIED: it governs the `_with_*` method, not every write
of the field it moves.

**Second half, which the registered prediction missed.** I predicted the fix would move only the
rows naming the two setters. It also moved the `at_lever` row, because a dispatched pin on a
machine whose table does not carry the cell hits the parent slot's **panic**: the port was turning
a benign Python attribute assignment into a hard REFUSAL — `obj._attr = x` succeeds on any Python
object, including one that never reads the attribute. Four of seven seats went from refusing to
reading. **A wrong dispatch does not only compute a different value; it changes which
configurations are answerable at all.**

**Method:** run the injection matrix BEFORE applying the fix, register what should go silent, then
prove neutrality against the **unregenerated** goldens — regenerating first makes the claim
vacuous ([[instrument-fed-by-what-it-certifies]]). Reach measured: 6 of 7 seats → 0 of 7.

Related: [[rust-port-slice-af-step5b]], [[rust-port-slice-ae-step5]], [[rust-port-copy-vs-rederivation]].
