---
name: rung88-slow-valve
description: "Rung 88 (the slow valve on a moving target, 2026-10-15) — process lessons; a verdict that was nearly construction for an instant device is exactly the one a perturbation overturns, and the model's stability bound is not the product's accuracy bound"
metadata:
  node_type: memory
  type: project
  originSessionId: 68ca826f-7a04-47fa-92b7-d3c822b37811
  modified: 2026-10-15T10:33:04.985Z
---

Rung 88 shipped 2026-10-15 (spec `docs/rung88-spec.md`, anchor
`docs/plans/rung88-anchor-lagged-incidence.md`, reading in `rust/src/lagged_incidence.rs`, sandbox
plan § 17: the bleed valve's "Moves slowly"). The user picked rung 87's named seam. H1–H4 HIT, H5
MISS (rung 87's order REVERSES for a slow enough valve) — the miss is the headline.

**Lessons (process, not physics — the verdict is in CLAUDE.md's row):**

- **A comparison against a pointwise-optimal device is the one a perturbation overturns.** Rung 87's
  "watch the wall, it costs less" was nearly construction (rung 87's own lesson) — and that is
  exactly why a lag reversed it: a law that does no more than needed has no slack to absorb an
  actuator's delay. I predicted the gap would "close but not to zero". **How to apply:** when a
  rung perturbs a device that was defined as the optimum, predict the SIGN of the comparison, not
  just its shrinkage; the optimum's lack of surplus is a reason it can flip.
- **Match to the requirement, not to the degraded result.** H5 matched the `φ` valve to the slow
  incidence valve's own (degraded) margin; the advisor pushed to re-tune BOTH slow valves to the
  original margin — the cleaner currency, and the one the spec leads with. Report the registered
  bar as scored, lead with the after-measurement table, say which is which.
- **The sibling trap can live inside the device.** `BleedLimiter::lagged` rebuilt through a
  constructor that reset the new field; the advisor found it before any measurement
  ([[instrument-fed-by-what-it-certifies]]'s family). A compiler census (make the field private on a
  scratch copy) found every reader of the set point ([[census-by-compiler-not-by-name]]).
- **The model's stability bound is not the product's accuracy bound.** Rung 65 asserts `ds/τ ≤ 2`;
  the sandbox runs at `ds` 0.02, where `τ` = 0.01 is legal but 5–10× less accurate than `τ` = 0.02.
  Measure at the product's own step before setting a knob's range.
- **An identity can explain a published finding.** Integrating the lag gives `∫(b − b_cmd) =
  τ(b0 − b_end)`: rung 65 § 1's "closing tail dominates" was arithmetic. Check whether a measured
  "mechanism" is an integral of the model's own equation before calling it physics.
- Bash heredocs carrying text with apostrophes broke twice more; write scripts with Write and run
  them ([[windows-tooling-file-hazards]]).

Related: [[rung87-valve-reference]], [[rung65-lagged-valve]], [[sandbox-direction]].
