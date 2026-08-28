---
name: rust-port-slice-ac-step1
description: "Slice AC (rungs 70/71) step 1 — the pre-flight's own step list and its census disagreed on the table count, and the gate I inherited had a scheduled expiry date"
metadata:
  type: project
---

Slice AC of the Rust port opened rungs 70 + 71: `src/cross_split.rs`, `src/full_split.rs`,
`_gov_max`'s carrier + `GovScope`, ten `R70*`/`R71*` tables, five swapped cells as named panics,
`tests/slice_ac_cells.rs` — 10 gates. Plan record at § 5.27.1.

**THE PRE-FLIGHT CONTRADICTED ITSELF ONE PAGE APART AND I ALMOST INHERITED THE WRONG HALF.** Its
step-1 line says *"the nine tables of § (iii)"*; § (iii) enumerates **ten**. Same shape as
[[rust-port-phase7-preflight]]'s own recorded lesson — the plan stated the same set twice and
nobody diffed the two — this time inside the pre-flight that records it. The repair was not to pick
a number but to make the count fall out of the SOURCE: `include_str!` + `.matches().count()` on
each file, with an absent-prefix control, plus a gate that names all ten consts in code so a
deleted one is a compile error.

**A GATE THE PLAN PRESCRIBED HAD A SCHEDULED EXPIRY DATE, AND THE PREVIOUS SLICE HAD ALREADY PAID
IT.** The prescribed step-1 gate was *every swapped cell has a distinct function pointer*, and
slice AB implemented that by reading nine placeholder panic messages — then had to dismantle it at
its own step 2, because step 2 deleted the messages and the gate's whole content was "not yet
ported". Same gate, durable form: `std::ptr::fn_addr_eq` INEQUALITY between two shipped `const`s,
each paired with an equality control on the same table. **Ask whether the step that follows this
one deletes the thing your gate reads.**

**AND THE PARENT AN INEQUALITY IS TAKEN AGAINST IS PART OF THE CLAIM.** Rung 71's swaps compare to
rung **70**'s bodies, not rung 69's — a slot reaching back past its immediate parent is a real
defect a grandparent comparison calls clean. Gated the `triple_laws` chain as three links (69
inherits 68's, 70 breaks it, 71 inherits 70's) rather than the one link the swap names.

**AND I NEARLY SHIPPED AN UNRECONCILED PAIR OF COUNTS INSIDE THE PARAGRAPH NAMING THEM AS THIS
PHASE'S MOST-REPEATED DEFECT.** Three doc comments quoted the pre-flight's *"35 of 35 `_with_gov`
calls with `val=None`"* beside its own *"98 sets through `_with_gov`"* — 98 sets is 49 calls, and
35 ≠ 49. Neither is the witness the claim needs: `engine.py` has exactly **three** `_with_gov` call
sites in the whole ladder and all three pass a literal `None`. **An enumeration beats a sample, and
the cheapest check on a quoted count is the other count in the same paragraph.**

**NINE MUTATIONS, NINE CAUGHT** — run before shipping, not after: each swap wired to its parent,
the builder wired to the wrong tables, the RAII guard restoring `None`, a constructor guard
neutered, a constructor guard MOVED below the build, and a spurious sixth const.

**THE WIDTH TRIPWIRE IS SHADOWED BY THE LIB AND ONLY FIRES ON THE SECOND STEP.** Adding a field to
`TripleHooks` never reaches a test target — `src/` holds five exhaustive literals, so the lib is
`E0063` first and cargo stops. It fires exactly in the scenario it is for (a slice adds a cell and
repairs `src/` because the lib must compile), which I established by simulating it rather than by
asserting the doc sentence. *A detector whose trigger you have not fired is a sentence.*

**Why:** every one of these is the same failure — an assertion, a count or a comment that is
satisfied by something other than the property it names.

**How to apply:** before writing a step's gates, read the NEXT step's deliverable and ask what it
deletes. Take counts from the artefact, never from prose that states them twice. When comparing a
child to a parent, use the IMMEDIATE parent. And fire every tripwire in the shape that would
actually hit it.

Related: [[rust-port-slice-ac-preflight]], [[rust-port-slice-ab-step1]], [[rust-port-slice-y-step3]],
[[rust-port-slice-aa-step1]], [[rust-port-copy-vs-rederivation]].
