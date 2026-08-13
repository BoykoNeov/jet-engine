---
name: rust-port-phase5-preflight
description: "The named risk was the EASY edge — widen the census twice before writing, and the second widening is the one that crosses a phase boundary"
metadata: 
  node_type: memory
  type: project
  originSessionId: ea32e583-4a8e-4d07-a4e0-1849ff5c88ec
  modified: 2026-08-13T08:19:07.173Z
---

Phase 5's pre-flight (2026-08-13, `docs/plans/todo-rust-port.md` § 5.3) was authorised to do
one thing: settle rung 61's two-parent class, which the plan had named as the phase's structural
risk. It took **one probe** to show that risk was almost empty — one colliding name, and rung 61
already opts out of it by hand. Everything of value came from the two widenings that followed.

**Widening 1 — from the named edge to all of them.** `engine.rs` has no matcher classes, so
phase 5 is the port's FIRST meeting with Python inheritance; the diamond is one edge of nine.
Clearing it alone would have been a false clearance. The census over all nine found the real
content: the handful of places a child overrides a method an *ancestor calls on `self`*.

**Widening 2 — from "inside phase 5" to "all 58 classes".** The first census restricted
*ancestors and descendants* to the phase-5 set, so it answered "does phase 5 need hooks for
phase 5?" and reported the set CLOSED. Opening only the descendant side found
`_solve_turbine`: rung 31's method, called on `self` in rung 31's own body, overridden by
**phase 6's** rung 34. Phase 5 must ship it hookable or phase 6 refactors already-gated code —
the exact failure the phase gating exists to prevent. **A "closed set" claim is only as wide as
the set you swept.**

Three method lessons, each of which changed the deliverable:

- **A callable-only `vars()` filter is not a census.** Class constants resolve by the same
  method order. The re-run over data found a live shadow — `_INC_MAX` 80→200 — read by inherited
  solver loops the subclass does not override. An iteration cap that silently changes under
  inheritance is exactly what must not become a literal in a ported body.
- **A `self.X` scan is blind to a sibling receiver.** Constructors that hand back a
  concrete-typed sibling (`at_setting`) mean `sib.match(...)` dispatches virtually through a
  different object. Found only by looking for what is called *on the result*.
- **Scaffolding is subject to the same evidence bar as the port.** A source-level re-parenting
  harness was designed and dropped before being written: it recompiles *the same source text*,
  so agreement is near-guaranteed and near-zero evidence — [[rust-port-copy-vs-rederivation]]
  applied to my own test rig. What is not void: does a `super()` target MOVE between cells, is
  the moving one ever traversed, and the kill test.

The kill test is the model for a **hazard** claim (as opposed to the exactness claims in
[[rust-port-inside-outside-exactness]] and [[rust-port-copy-vs-rederivation]]): build the road
not taken and measure it. The source's comment was confirmed AND sharpened — the co-operative
constructor raises nothing, and the error lands 13–15 % in the two quantities the rung is about
while thrust moves 0.1 %. "Plausible numbers with no exception" was literally true. Count it in
its own category, not in the exactness ledger — see [[two-indexes-one-spine]] for why a stale
cross-reference costs more than an uncounted one.

**Why:** the plan's named risks are written before anyone looks, so a named risk is a hypothesis,
not a finding. Twice here the named thing was the cheap thing and the census around it was the
expensive one.

**How to apply:** when a plan says "resolve risk R before phase N", treat R as the *entry point*,
not the scope. Ask what class of thing R is an instance of, sweep that whole class, then open the
sweep one boundary wider than the phase you are clearing. Related: [[rust-port-decided]],
[[rust-port-ladder-architecture]], [[rust-port-oracle-cannot-see-a-missing-gate]].
