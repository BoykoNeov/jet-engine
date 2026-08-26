---
name: rust-port-slice-x-step4
description: A count taken before a fix is not a count of what shipped — add the tally up
metadata:
  type: project
---

Slice X step 4's oracle compares 1906 keys against PyPy AND CPython. Measured by diffing the two
goldens directly: **0 of 1744 float keys drifted, 0 of 162 discrete keys flipped**. Slice W needed
exactly one cross-interpreter exemption, for Python's built-in `sum()`, whose accumulation order
differs; rung 64's readers contain **no `sum()` at all** (grepped, not assumed) — every accumulation is
an explicit `+=` and every extremum a `max`/`min`. So the arm asserts EXACT agreement instead of
carrying a tolerance that suppresses nothing.

**Why:** a bar that never fires is a rule nobody has re-read since the day it was written. Measuring
first turned "pick a tolerance" into "there is no tolerance to pick", which is a stronger gate.

**AND THE COUNT I LOGGED DID NOT ADD UP.** As first written this said `1744 + 146` beside a total of
1906 — short by exactly the 16 keys the destructure had just made the dump emit. The drift comparison
had been run on the pre-fix goldens and the numbers carried forward. No gate can see this: both arms
assert EXACT equality, so the interpreters really do agree on all 1906 — it was the RECORD that was
stale, in the file the next slice reads as what was measured. Re-measured by tagging each key with the
emitter that produced it, since the goldens store floats as IEEE-754 bit patterns and a reader of the
file cannot tell a float from an integer: 1744 float + 162 discrete = 1906, 0 drifted, 0 flipped.

**How to apply:** measure the drift between the two goldens BEFORE writing any bar (see
[[rust-port-guessed-census-bars]]). If it is zero, assert exactness and record the mechanism that makes
it zero, so a future drift is diagnosable rather than absorbed. Two tooling notes from this step:
redirecting a dump with `2>&1` interleaved its own stderr INTO the middle of a data line and the loader
said only `InvalidDigit` ([[windows-tooling-file-hazards]] again — send stderr to a separate file, and
make the loader name the offending line); and an exhaustive struct destructure in the emitter caught 16
keys the Python dump was not emitting. And re-take
any tally whose subject changed after it was measured — or simply ADD IT UP, which is what caught this
one: a total that does not equal the sum of its parts is a defect no test can reach.
