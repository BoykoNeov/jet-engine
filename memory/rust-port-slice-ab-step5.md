---
name: rust-port-slice-ab-step5
description: "Slice AB step 5 — a predicted failure SHAPE is a claim about the parent's body, and two of four \"breaks by panic\" broke by value"
metadata: 
  node_type: memory
  type: project
  originSessionId: 0c22f3fc-4861-40a2-b65c-12746bba5bf2
  modified: 2026-08-28T14:54:48.578Z
---

Slice AB step 5 (rung 69's ten dispatch gates, 2026-08-28) swapped each cell for its **parent's own
function pointer** — the pre-registered injection, because this slice's risk is *a swap that is
silently still the parent*. The pre-flight had also predicted, per cell, **how** each break would
show: four "by PANIC", one "by MESSAGE only", the rest "by value".

**Two of the four panic predictions were wrong, both for the same reason.** The prediction assumed
rung 68's body dereferences the floor a rung-69 machine does not carry. `_triple_rig` never *reads*
that floor — it *builds* one from the map, so the parent hands back a well-formed sibling carrying
the WRONG REFERENCE and nothing raises. `_manifold_v` is a one-liner that reads no field at all, so
it returns the loop's own root where rung 69 returns the shared manifold — **opposite signs** at the
sampled point. Both were measured by running each injection once and printing what it did, before a
single assertion was written; the Python was then re-read to confirm the port, not the prediction,
was faithful.

**Why:** a break's SHAPE is a claim about the *parent's* body, not about the cell being gated, and
it is the half nobody re-reads. Getting it wrong is not harmless: a gate written to expect a panic
asserts `should_panic`, and a cell that actually breaks by value would then be gated by a test that
passes for the wrong reason — or, worse, be called UNOBSERVABLE. **The silent shape is the dangerous
one**, so a wrong "it panics" prediction always errs toward under-gating.

**How to apply:** never write the assertion shape from the pre-flight's table. Run every injection
once, print panic-or-value, and write the gates from that. Two more things the same step wants:

* **Parameterise the tally so a CONTROL exists.** The same ten predicates run with *nothing*
  injected must report ten zeros — otherwise a predicate that is true against the shipped table is a
  sentence, not a measurement. This slice shipped two gates that could not fail at its own step 1.
* **A reader that rebuilds a sibling throws your injection away.** All six of rung 69's readers go
  through `_triple_rig` → `at_lever`, which installs the SHIPPED table, so every value injection had
  to be scored on a direct march instead. Asserted, not described: with the parent installed the
  reader returns bit-for-bit identical rows.

Related: [[rust-port-slice-ab-step4]], [[rust-port-slice-aa-steps2345]], [[rust-port-slice-w-step3]],
[[rust-port-slice-x-step1]].
