---
name: rust-port-slice-x-step4
description: Both interpreters came out bit-exact, so the measured "bar" was that there is nothing to type
metadata:
  type: project
---

Slice X step 4's oracle compares 1906 keys against PyPy AND CPython. Measured by diffing the two
goldens directly: **0 of 1744 float keys drifted, 0 of 146 discrete keys flipped**. Slice W needed
exactly one cross-interpreter exemption, for Python's built-in `sum()`, whose accumulation order
differs; rung 64's readers contain **no `sum()` at all** (grepped, not assumed) — every accumulation is
an explicit `+=` and every extremum a `max`/`min`. So the arm asserts EXACT agreement instead of
carrying a tolerance that suppresses nothing.

**Why:** a bar that never fires is a rule nobody has re-read since the day it was written. Measuring
first turned "pick a tolerance" into "there is no tolerance to pick", which is a stronger gate.

**How to apply:** measure the drift between the two goldens BEFORE writing any bar (see
[[rust-port-guessed-census-bars]]). If it is zero, assert exactness and record the mechanism that makes
it zero, so a future drift is diagnosable rather than absorbed. Two tooling notes from this step:
redirecting a dump with `2>&1` interleaved its own stderr INTO the middle of a data line and the loader
said only `InvalidDigit` ([[windows-tooling-file-hazards]] again — send stderr to a separate file, and
make the loader name the offending line); and an exhaustive struct destructure in the emitter caught 16
keys the Python dump was not emitting.
