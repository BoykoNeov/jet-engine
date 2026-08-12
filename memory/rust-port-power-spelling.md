---
name: rust-port-power-spelling
description: "Python's ** is a libm pow call; the faithful Rust spelling is SPLIT (multiply the square, pow above it) and a tolerance bar hid the bug for a whole phase"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 454e5108-5b41-4abd-b607-eac9932757b5
  modified: 2026-08-12T07:27:45.023Z
---

Porting `A[i] * T ** n` to Rust has exactly one faithful spelling, and it is **split**:

- `T ** 2` → `t * t` (PyPy's JIT rewrites the square to a multiply — 6013/6013 exact)
- `T ** 3` and above → a real `pow` call (`x*x*x` matches `x ** 3` only 4519/6013; binary
  exponentiation `(x*x)*(x*x)` is *worse* than the naive chain, 3054/6013)
- `x ** 0.5` → a real `pow` call, and LLVM **folds `x.powf(0.5)` into `sqrt`** unless the
  exponent is wrapped in `std::hint::black_box` (`gas::powp`). Python's `** 0.5` differs from
  `sqrt` about 1 point in 670.

Both simplifications lose: "always multiply" costs 3196/3232, "always pow" costs 3230/3232, the
split gives 3232/3232.

**Why:** phase 1 shipped product chains and its own oracle passed at 100 % on enthalpy, because
the mis-spelled terms are the HIGH-ORDER ones — a 1-ULP error in `a[4]*T^5/5` is ~1e-20
relative to the sum and only occasionally tips the last bit. It surfaced only when phase 2
evaluated at **cycle-determined** temperatures (`T9 = 775.53`) instead of a round grid, and even
then it presented as a solver artefact because the Newton amplified it to 1e-11.

**How to apply:**
- A 100 %-bit-exact result **on a chosen grid certifies the grid, not the arithmetic.** Probe at
  values the system produces, not only at round numbers.
- **Hold an oracle arm to a COUNT, not a tolerance,** where the count is achievable. A tolerance
  bar cannot tell a real defect from acceptable noise, and here it did not for a whole phase.
  Both oracles now assert `exact == total` on the PyPy arm with a message saying what to
  investigate before loosening it.
- The price of fidelity was measured, not assumed: **2.1× on a Fork-B cycle, 1.36× on an
  equilibrium cycle**, confined to the three polynomial functions and therefore to real-gas
  paths — CPG sections are closed-form and the deepest transient ladders (rungs 66–84) run on
  CPG. Measure the cost before recording it as the port's price.
- The rules live in `rust/tests/porting_rules.rs`, which is deliberately **oracle-free**: it
  checks that the three spellings are still *different operations*, since the day they stop
  being different, the reasoning above silently goes vacuous.

Related: [[rust-port-arithmetic-is-pypy]], [[golden-fingerprint-gate]], [[rust-port-decided]].
