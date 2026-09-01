---
name: rust-port-slice-ae-step1
description: "An exact-bits gate written for a float-identity injection was PASSED by that injection, because the triple it was driven at cancelled exactly"
metadata: 
  node_type: memory
  type: project
  originSessionId: 499b6182-82fa-4b8b-a790-952c9fb5e575
  modified: 2026-09-01T16:00:15.375Z
---

Slice AE step 1 (rung 73 Rust port) shipped `rust/src/applied_reference.rs` — five `R73*` tables,
**six re-aimed pointers, ZERO new table fields** — and `rust/tests/slice_ae_cells.rs`, **15 gates**,
all green first run. **15 mutations: 14 killed, 1 predicted survivor.**
Plan § 5.29.1.

**THE LESSON: an exact-bits gate is only as sharp as the arithmetic it is handed.** The pre-flight
(P5) said a port that folds `_reference`'s float-identity branch away sits below every RELATIVE bar
in the crate, so the gate was written on `to_bits`. **The injection passed it anyway.** All three
paths were driven at the probe's tuple `(req, g_own, gf, gr) = (3.5, 2.0, 2.0, 1.0)`, and at those
magnitudes `(2.0 + 3.5) - 2.0` **is** `3.5` bit for bit — branch and fold-away agree, so deleting
the branch moved nothing the gate could see. Re-driven at `(0.3, 0.1, 0.1, 0.05)`: shipped `0.3`,
fold-away `0.30000000000000004`, relative gap `1.85e-16`.

**Nothing else caught it** — 15 green gates, a bit-for-bit reduce march and its own vacuity control
all passed with the branch deleted. Only mutating the step's own source did.
[[rust-port-slice-w-step3]]'s *make the instrument prove it can SEE*, applied to a two-line
assertion instead of a census.

**How to apply:** an exact-bits assertion must assert, in the same test, that the defect it is
written for **would have moved the answer** — `assert_ne!(out, <the fold-away>)` on the line below.
A round number chosen for readability is chosen for cancelling; pick the inputs from the source, at
realistic magnitudes, and keep a benign one only if it is LABELLED as discriminating nothing.

**A CONSTRUCTOR DEFAULT THAT PASSES ITS OWN RUNG'S REFUSAL.** The core's ctor writes
`ref_law = "sched"` for the whole family; Python declares `"applied"` at rung 73. A port that
forgot the overwrite hands back a machine that **passes** rung 73's `integrate_fuel` refusal
(`"sched"` is a declared law), marches rung 72, and reports rung 73 — **and the reduce arm goes on
passing, because the reduce IS "rung 73 under `sched` is rung 72"**. The `at_lever` trap, one level
up at the CONSTRUCTOR, and it was not in the pre-flight. The advisor named it; probe L1 measured
it. Ask what a class attribute's value is on a FRESH object, never what the parent's ctor writes.

**A one-sided field gate admits a body that writes BOTH.** `_with_ref` is rung 69's name and rung
73's, same arity, different mutated field — the shape a phase-wide SIGNATURE sweep cleared. Assert
the field that moved **and** the field that did not, on both machines
([[rust-port-slice-ac-preflight]]).

**And a prediction repaired for one object can stay false for another.** P7 ("`TripleHooks` stays
at 13") holds exactly for step 1's six pointers and is already known false for the seventh —
`_quad_gains_at` has no field in any of the five table types, so step 2 takes it to 14.
Pre-registered rather than met as a surprise.

**A CONTROL THAT READS A NAMED CONSTANT CAN COMPARE A MACHINE AGAINST THE CONSTANT THAT BUILT
IT.** The rung-72 control asserts `shared(...).ref_law == REF_LAW_DEFAULT`, so flipping that
constant would pass. It does not — and the line that stops it is
`assert_ne!(REF_LAW_APPLIED, REF_LAW_DEFAULT)`, measured (not argued) to be the assertion the
mutation dies on. A gate comparing against a named constant needs a second assertion pinning that
the constant is not the other one.

**AND A CONTROL THAT ASSERTS AN ABSENT SUBSTRING IS SATISFIED BY AN UNRELATED ABORT.** Both
refusal gates' controls now assert the march RETURNS CLEANLY — not merely that the refusal's words
are missing. Measured first: the same arming really does abort elsewhere when driven with a
hand-built initial condition instead of through the ramp.

See [[rust-port-status]], [[rust-port-copy-vs-rederivation]] (the `_shared_rig` carry is a MEASURED
no-op — `at_lever` already carries the law — ported anyway and pre-registered as having no value
break, which mutation M11's survival confirms).
