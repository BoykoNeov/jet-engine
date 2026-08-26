---
name: rust-port-slice-x-step1
description: Extending a SHARED helper in place silently voided an override, and 1017 green tests could not see it
metadata:
  type: project
---

Slice X step 1 (rung 64) added a third arming mode by extending the **shared** `LeverArm::arms_valve()`
in place. That silently made rung 64's `isolating` override **textually identical to its parent's** —
because in Python the two bodies differ at exactly that one term, while the assert's other side is
dispatched and gains the term by itself. The full 1017-test gate was green and blind: no shipped test
hands the new arming mode to a parent machine, which is the only input that separates them.

**Why:** an override whose whole content is one predicate disappears if you widen the predicate its
parent also reads. The cost is not a wrong number — it is a swapped-cell row that stops being a swap,
and an injection census that then reports "inert" for a reason unrelated to the source being ported.

**How to apply:** before editing a helper that a PARENT body reads, check whether the child's override
differs from the parent ONLY in that helper. If so, add a second helper rather than widening the first,
and make each name the other in its doc. Then write the gate for the input that separates them — see
[[rust-port-slice-x-step2]]'s smoke section H. Found by the advisor, not by any test.
