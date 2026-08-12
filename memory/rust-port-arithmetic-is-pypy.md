---
name: rust-port-arithmetic-is-pypy
description: "Rust's float results ARE PyPy's, not a third dialect — so port risk is in solver stopping rules, not arithmetic"
metadata: 
  node_type: memory
  type: project
  originSessionId: 454e5108-5b41-4abd-b607-eac9932757b5
  modified: 2026-08-12T06:22:18.981Z
---

Phase 1 measured Rust against **both** interpreters the project ships on, not just one. Over
3232 gas values (rungs 1–6): **58 % bit-identical to CPython, 98.89 % bit-identical to PyPy.**
Every forward quantity — including `exp`, `log`, the dense Gauss-Jordan solve and the 8-species
equilibrium Newton — is **100 % bit-exact against PyPy**. CPython's libm is the outlier, not Rust.

The only spread is the two safeguarded-Newton inverses, and that is `_solve`'s own `tol = 1e-11`
stopping rule landing on a different iterate — **three orders of magnitude above every other
term in the gas layer.**

**Why it matters:** it inverts the port's risk model. The budgeted danger was "Rust's floating
point drifts from Python's"; the real danger is **solver stopping rules**. Later phases should
budget for iterate reproducibility, not last-bit polynomial drift. It also makes the approved
re-anchor cheap: Rust's numbers are PyPy's numbers, and the gate already runs on PyPy.

**How to apply:** when porting anything with an iterative solve, port it **iterate-for-iterate**
(same seed, same damping, same stopping test) — that is what bought bit-exactness on the
equilibrium solve. And always dump the oracle under **both** interpreters: the CPython↔PyPy gap
is a deviation the project already tolerates, so it is a principled tolerance bar, whereas an
invented `1e-15` is not. The method lesson: a per-quantity bar opened deliberately loose
(`equilcomp` at 1e-9) and then **closed onto the measurement** (1e-14) is what turned a
foregone conclusion into a finding. Related: [[rust-port-decided]], [[golden-fingerprint-gate]].
