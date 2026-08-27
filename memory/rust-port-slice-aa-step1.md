---
name: rust-port-slice-aa-step1
description: "Slice AA (rung 68) step 1 — a prediction that quietly narrowed its own scope between registration and settlement, and `ptr::eq` on a `const` written a second time"
metadata: 
  node_type: memory
  type: project
  originSessionId: 93497d43-ebb8-49c9-848c-d51d8632150f
  modified: 2026-08-27T13:03:41.782Z
---

Slice AA of the Rust port (rung 68, `StatorLimiter` + `ThreeLoopCascadeTransient`) pre-registered
at § 5.25 off six probes and shipped step 1: nine cells, four dynamically-scoped fields, four RAII
guards, 10 gates green.

**The process lesson, and it is about the PREDICTION rather than the code.** P1 read *"`MarchScope`
grows by two fields and zero existing struct literals move."* That held exactly. But `MarchScope`
was not the only struct the step grows — `StatorArming` and `LeverArm` each gained a field too, and
between them the compiler named **six** literals (five in `src/`, one in `tests/`). The prediction
was written about the struct the PREVIOUS slice had been burnt on and never asked the same question
of the two this slice adds to. It is true, and narrower than the sentence it was registered as.
**Ask a growth prediction of every struct the step grows, not of the one that hurt last time.**

**And `ptr::eq` on a `const` was written a second time.** [[rust-port-slice-y-step3]] already
records that it tests the optimiser; the first version of the step-1 gate asserted
`ptr::eq(core.triple_hooks, &NO_TRIPLE)` and failed on the first run. The replacement reads the
panic MESSAGE — which is also the stronger gate, because nine `assert!(panics(…))` calls are
satisfied by nine bugs as readily as by nine unreachable cells.

Two measurements worth keeping: a named arithmetic risk (a cube root in `_cubic_roots`, which the
crate has no porting rule for) **measured to zero** by reading the body — the whole rung has one
`**` and its exponent is `0.5`; and my own nesting probe printed *"`_ic_order` nests to depth 106"*
for a field that cannot nest, because the depth counter was driven by value-nullity and that
guard restores to the PREVIOUS value. The column that answers the question is OVERWRITE, and only
for restore-to-`None` fields. See [[rust-port-slice-z-step5]] for the same shape one slice back.
