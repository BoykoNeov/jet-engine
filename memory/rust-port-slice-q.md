---
name: rust-port-slice-q
description: "Slice Q (rung 37) — a dead arm is a property of the GRID, not the code; and three gates that measured the wrong set"
metadata: 
  node_type: memory
  type: project
  originSessionId: d3ef1fc3-403a-46c6-9456-b4f1aa4bef2d
  modified: 2026-08-17T21:05:54.907Z
---

Phase 6 slice Q (rung 37, `CombustorTransient` → `rust/src/combustor.rs`) shipped 2026-08-18 in
four steps: port + smoke (517 values), the 7 Python gates + 3 added, the oracle (2 066 keys,
100 % bit-exact against PyPy first run, CPython arm 21.9 %), docs. All ten pre-registered
predictions held.

**A DEAD ARM IS A PROPERTY OF THE GRID, NOT OF THE CODE.** Slice P measured `try_illinois`'s
`maxit`-exhaustion arm at **zero** firings, showed no value gate could see whether it returned
`a` or `b`, and closed the blind spot with a counter instead of deleting the claim
([[rust-port-slice-p]]). One rung later that arm is the path **94.5 %** of one call site's calls
take — `_plenum_pt4_at` passes a `1e-12` **absolute** bracket width on a pressure of order `1e5`
— and the `a`-vs-`b` choice is worth 456 of 2 066 oracle keys while failing **zero of ten** gates.
The counter that could only be justified on principle is what made the measurement possible.
**Do not delete a claim you cannot currently test; instrument it.**

**FIVE INSTRUMENT DEFECTS IN ONE SLICE, AND THREE OF THEM WERE GATES.** A probe counter was global
where its sentence said scoped; an injection harness's parser swallowed cargo's own output into
its section list; then three oracle gates failed, each measuring a different wrong set —
a precondition that named a section making **zero** root-finds, a rate whose denominator counted
116 calls that never reached the loop, and a per-section census asked a per-**call-path**
question (the same conflation as the probe, one stage later). *A probe that measures nothing
wastes a run; a gate that measures nothing ships.* Related: [[rust-port-slice-m]],
[[rust-port-slice-n-step2]].

**A QUANTITY CAN BE COMPUTED CORRECTLY, READ ONCE, AND READ ONLY WHERE ITS CORRECTNESS CANNOT
MATTER.** Rung 37's docstring leads with computing the plenum shaft power "honestly" on two
distinct mass flows. That `Phi` is read at exactly one site — the equilibrium residual — and the
difference from the naive per-unit-air version is proportional to `mdot_ngv − mdot_c*(1+f)`, which
is **exactly** the condition that residual drives to zero. So the rung's headline arithmetic is
gated by nothing it ships; only an off-equilibrium bit dump sees it. **Before trusting a source
comment's emphasis, grep who reads the value and check whether the readers can tell.**

**ENUMERATE THE REGISTERED PREDICTIONS BEFORE WRITING THE STEP UP, NOT AFTER.** Prediction 9 was
carried by nothing — the dump emitted no `phi_max` arm tallies — and the enumeration caught it
pre-write-up. This is [[rust-port-slice-p]]'s step-3 lesson applied rather than repeated.

Detail: `docs/plans/todo-rust-port.md` § 5.14. Remaining in phase 6: slices R, S, T, U.
