---
name: rung73-applied-reference
description: "Rung 73 — a predicted coupling landed in the wrong COLUMN, so rung 72's block form survived its own sharpest seam; and weakening an instrument cost five orders of magnitude"
metadata: 
  node_type: memory
  type: project
  originSessionId: ee1bb5b3-3bf3-438a-840d-81bfcc41dd85
  modified: 2026-08-09T15:05:17.421Z
---

Rung 73 (shipped 2026-08-09) closed rung 72 § 11's sharpest seam: the two fuel-side legs
re-referenced to the fuel actually burnt. **Rung 72 predicted `F_r ≠ 0` would destroy its
triangular block. `F_r = −1` exactly — and the block survived**, because `F_r` is a ROW
entry in the *authoritative* column while triangularity lives in the *masked* one.
Triangularity is a property of **min-select alone**, not of the reference. What the
reference moves is the pole: `−1/τ_masked` → **exactly the origin** (a masked leg becomes
a pure integrator), so `zeros` gains `n_masked` and `det J` dies in rung 71's cell — the
only live determinant in the family — with no loop, gain, clock or state added.

**Four things worth carrying forward:**

- **Check WHICH INDEX a predicted coupling lands in before believing its consequence.**
  Row vs column decided the whole rung. The seam's premise and its conclusion were
  independent, and only the premise was right.
- **Weakening an instrument has a measurable price, and it can be worth paying.** To keep
  the pole claim from being the shipped instrument agreeing with itself (the *fifth*
  instance in this family), `_jac4` had to MEASURE the two fuel-side diagonals it used to
  write. A measured diagonal carries float cancellation, so the parent-polynomial identity
  landed at 5e−12 where rung 72's constructed one reached 7e−17. Right trade — the
  alternative was a headline the instrument writes itself. See [[rung72-shared-actuator]].
- **An exact ZERO survives a difference quotient; an exact ONE does not.** `self_live` is
  `== 0.0` because the hook takes an explicit identity branch; `self_masked` is 1.0 ± 3e−12
  because it differences a SUM. Anchor P7 asked for both and got half.
- **The bug that produced a PERFECT confirmation.** `_reference`'s first version ignored
  `_ref_law`, so the A-vs-B reader differenced the plant against itself and returned
  exactly the rung's headline (`delta_rest = 0.0`, `mask_leak = 0.0`) having measured
  nothing. The only defence that works is a gate that FAILS when the two laws are the same
  one — and the probe must re-bless `at_lever`'s hard-coded class or it tests the shipped
  one and passes.

**And it CORRECTED rung 72's ledger by 110×/39×**: a masked governor referenced to the
schedule is credited with a cut the other leg already made, so it takes the actuator too
soon and rung 72 under-reported its own peak `Tt4` debit (+0.29 K → +32 K). The error
exists only at rung 72 — with one fuel-side leg the two references coincide exactly.

Next seam named: the **state-as-demand coordinate** (a leg lags its DEMAND, not its clip;
they differ by `ṁf_sched·τ` on a ramp) — the last place `n_live = 4` could hide.
