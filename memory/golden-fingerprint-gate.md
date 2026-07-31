---
name: golden-fingerprint-gate
description: "tests/test_numeric_fingerprint.py (slice 1 of the PyPy switch, 2026-07-31) is the project's ONLY absolute-value gate — 6381 values pinned against a CPython-generated golden. Its goldens must NEVER be regenerated under another interpreter, and its per-kernel tolerances are load-bearing permanently."
metadata: 
  node_type: memory
  type: project
  originSessionId: a6e5038b-edb9-421d-8bb5-a62a5802f181
  modified: 2026-07-31T06:41:26.856Z
---

**The hole it closes:** every other tight assertion in the suite (185 at <=1e-9) is a **same-run
relative identity** — it computes two quantities in one process and asserts they agree. That is
the right shape for the reduce-to-prior spine, but it is **blind to anything that moves BOTH
sides together**: an interpreter swap, a library update, a solver rewrite. Proven, not supposed:
the rung-30 `_sonic_throat` closed-form fix moved every CPG value by 1-2 ulp and the whole suite
could not have told you.

**The load-bearing constraint — the goldens are CPython's, permanently.** They must **NOT** be
regenerated as part of the PyPy switch (or any later interpreter change). Regenerating under the
new interpreter destroys the only cross-interpreter anchor and silently reduces the file to
"the current interpreter agrees with itself" — the exact hole it was built to close.
`test_golden_file_declares_its_provenance` fails if the `meta` block stops naming CPython.
Consequence: the per-kernel tolerances are load-bearing **forever**, not just during a transition.

**The result that made it worth building:** run under PyPy against the CPython goldens, **every
arm is green and no arm consumes more than 10.5% of its tolerance (none under 2.4%)**. That is
the FIRST cross-interpreter check this project has had against ABSOLUTE numbers — every prior
PyPy result, including a 973-green full gate, could only show PyPy agreeing with itself.

**Shape:** 8 kernel arms + 2 meta-guards; goldens in `tests/golden/numeric_fingerprint.json`
(6 381 values = 6 369 floats + 12 discrete, 384 KB, floats as `.hex()` so a 1-ulp change is
visible in the committed diff). Tolerance = **one round decade above the measured
CPython-vs-PyPy drift**, recorded beside each constant. The two CPG arms assert **bit-equality**
(measured exact). `conftest._is_spine` gained a 4th pattern, `test_golden_fingerprint*`
(deliberately narrower than `test_golden*`), because kernel E is 7.7 s idle and would be
slow-tagged out of bare `pytest` under an 8-worker load.

**The method lesson — measure a detector's sensitivity, do not assert it.** The plan's done-when
was "a 1e-7 perturbation makes it fail", uniformly. That is impossible for kernel F (tol 1e-4,
because 3.7e-6 of real drift lives under it). Sweeping a relative input perturbation per arm gave
the honest claim: **catches 1e-5 anywhere, 1e-10 or tighter on 7 of 8 arms**; kernel D's turbine
bisection *amplifies* ~6 decades past its own tolerance (detects 1e-13 at tol 1e-7). An
unmeasured sensitivity claim is the same class of error as the unscoped bit-identity sentence
that started this thread — see [[perf-sonic-throat-and-pypy]].

**Regeneration is a PROCEDURE, not a flag.** `python tests/test_numeric_fingerprint.py
--regenerate` prints every changing value before writing. On a trip, decide: shape drift /
regression / accepted ulp shift (rung 30 is the worked precedent for the third), and record in
the commit message WHICH values moved, BY HOW MUCH, and WHY. Plan + evidence:
`docs/plans/todo-pypy-switch.md`. See also [[test-suite-speed-policy]], [[claude-md-is-a-reference]].
