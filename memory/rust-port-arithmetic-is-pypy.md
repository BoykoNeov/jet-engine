---
name: rust-port-arithmetic-is-pypy
description: "Rust's float results ARE PyPy's — 100% bit-exact through rungs 1-6 and the whole design cycle, once the power spelling was fixed"
metadata:
  node_type: memory
  type: project
  originSessionId: 454e5108-5b41-4abd-b607-eac9932757b5
  modified: 2026-08-12T07:27:25.471Z
---

Measured against **both** interpreters the project ships on, not just one:

| | vs PyPy | vs CPython |
|---|---|---|
| gas oracle, 3232 values (rungs 1–6) | **3232 / 3232 (100 %)** | 1883 / 3232 |
| cycle oracle, 1481 values (design point) | **1481 / 1481 (100 %)** | 903 / 1481 |

`exp`, `log`, `pow`, the dense Gauss-Jordan solve, the 8-species equilibrium Newton, both
safeguarded-Newton inverses, and both burner solvers all reproduce exactly. CPython's libm is
the outlier, not Rust — the two Python interpreters agree with each other on only 64 % of the
cycle values, so "Rust IS PyPy" is a stronger statement than "Python is Python".

**CORRECTION to what phase 1 concluded.** Phase 1 measured 98.89 % and blamed the residual on
`_solve`'s `tol = 1e-11` stopping rule, and wrote "the real danger is solver stopping rules,
not arithmetic" into the plan. **That was wrong.** The cause was arithmetic: a transcription
defect in the polynomial power spelling (see [[rust-port-power-spelling]]). The stopping rule
was the *amplifier* — it turned a ~1e-20 relative error into a 1e-11 one — which is exactly
what made it look like a solver artefact. Fixing the spelling took both oracles to 100 %.

**Why it matters:** the port's risk model is now "neither" rather than "solvers" — through
rungs 1–6 there is no measured drift of any kind. Phase 3's mixing PDFs bring new solvers and
may break that; if so, the fallback is the published-deviation tolerance policy, not a silently
loosened bar.

**How to apply:** when porting anything with an iterative solve, port it **iterate-for-iterate**
(same seed, same damping, same stopping test). Always dump the oracle under **both**
interpreters — the CPython↔PyPy gap is a deviation the project already tolerates, so it is a
principled bar, whereas an invented `1e-15` is not. And size a solver claim by its **distinct
roots**, not its row count: "far 114/114 bit-exact" was 19 real measurements, of which the
rung-6 bisection contributed 3, so the oracle grew a sweep (`pi_c` 6→30, `Tt4` 1000→2100 K,
`eta_b`, `M0`, `p0`, `mdot`) to reach 19 fixed-point and 15 bisection roots before the claim was
written down. Related: [[rust-port-decided]], [[golden-fingerprint-gate]].
