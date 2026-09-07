---
name: rust-port-slice-ag-step7
description: "Slice AG step 7 (the dispatch gates, slice AG closed) — a counting pointer answers per cell what every dispatch matrix since AD inferred from the rest of the row, and a prediction about needles was refuted by an instrument that reads numbers"
metadata: 
  node_type: memory
  type: project
  originSessionId: 0ce3e683-3c6d-45e1-aa9c-28d47c8ad3ce
  modified: 2026-09-07T13:02:10.253Z
---

Slice AG step 7 shipped `rust/tests/slice_ag_dispatch.rs` (8 gates) and **CLOSES SLICE AG** at
seven steps. Detail is in `M:\claud_projects\jet engine\docs\plans\todo-rust-port.md` § 5.31.7.

**THE LESSON — a "did it move?" instrument reports one bit, and the missing bit is cheap.**
Every dispatch step since slice AD has carried the rule *a silent row is either laundering or a
path that never reaches the cell, and what separates them is the OTHER SEATS IN THE ROW*. That
rule is a workaround. A pointer that **counts and then delegates to the shipped body** reports
*was this cell entered* directly, at every cell, for one `Cell<usize>`. Two rows of this matrix
are verdict-for-verdict identical and mean opposite things:

* `shared_rig` → parent: `same` at all seven seats, tally **2,1,4,2,3,3,2** ⇒ it RAN, REDUNDANT.
* `sensed_cap` → parent: `same` at four, tally **0,0,0,0**,144,1100,24 ⇒ UNREACHED at those four.

**How to apply:** in any dispatch/mutation matrix, install a delegating counter beside the
parent-pointer injection — and assert the counted reading is BIT-IDENTICAL to the baseline first,
or the tally belongs to a different plant. Cf. [[instrument-fed-by-what-it-certifies]].

**THE SECOND LESSON — a prediction can be about the wrong INSTRUMENT.** P6 predicted that rung
76's one untagged assert message (*"the two cap laws marched different grids"*) was where a port
defect could hide, *because a `rung-76:` prefix check cannot see it*. That premise is true and is
now gated. The prediction is still **REFUTED**: both realistic transcription slips at that cell —
dropping `pt4 / pi_b`, and swapping `cap`'s two same-typed `f64` arguments — are killed by
`rung76.rs` (5 and 9 gates) and by the value oracle (2 and 3). **P6 reasoned about NEEDLES and the
cell is covered by VALUES.** Ask which instrument actually reads the site before predicting where
a defect can hide. Cf. [[rust-port-slice-ag-step6]] (a prediction naming the right cause and the
wrong property).

**And the residual is sharper than the prediction was.** The message IS reachable, but only in a
narrow band (scale the cap ×0.6 or ×0.4 and the sensed march dies part-way, 166 / 99 of 341);
outside it the arm dies at `s = 0` and what surfaces is **rung 43's** refusal. A reader is told the
wrong rung either way — once by omission, once by attribution.

**A recurrence worth its own line: the coarse grid was typed outside the region the rung
declares** (`ds = 0.02` ⇒ `ds·Σ(1/τ) = 2.400 > 2`), which is slice AF step 6's own instrument
defect number one, one slice later, in the same kind of file. The step is now DERIVED from rung
75's floor. **A grid a step coarsens is a claim about an asserted stability region, not a free
knob.** Two more self-caught defects: a control that read its own EXPECTED row instead of the
measured one (rung 70's *a gate computing my own formula twice*), and a whole-point comparison
that omitted `PointExtra` and so reproduced the `Tt4` count (333) while calling it the tuple (341).

**P7 CONFIRMED, cleanly** — seven steps, with steps 2 and 4 still carrying three and four method
bodies, so the VOID clause does not apply. **AF's method-count law fails its first two-class test;
AC's class-count reading survives.** P1 falsified again and in the same direction (**2.73×** vs a
2.0–2.6 band): a ratio re-fit to the last three points is an estimator with no theory in it.

See [[rust-port-status]], [[rust-port-slice-ag-step6]].
