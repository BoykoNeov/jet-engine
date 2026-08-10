---
name: golden-gate-slice3
description: "Fingerprint slice 3 (rungs 67-77) — a stride knob is not a resolution knob, and a structural zero needs an absolute tolerance leg"
metadata: 
  node_type: memory
  type: project
  originSessionId: 43aa3272-cde6-4e56-a40e-e9c9fccb57ff
  modified: 2026-08-10T04:18:14.844Z
---

Fingerprint slice 3 shipped 2026-08-10 (commit `e7e7c22`): 11 arms, one per rung 67–77, pinning
each rung's INSTRUMENTS (`det J`, `c1`, `zeros`, eigenvalues, leg slopes) rather than its
trajectories. 10,531 new values; golden 26→37 kernels, 8,042→18,573 values. It retires rung 76
§ 6.1's hand-rolled 229k-float `git worktree` parent-edit check — `pytest
tests/test_numeric_fingerprint.py` is now that check. See [[golden-fingerprint-gate]] and
[[golden-gate-slice2]].

**Two things that were nearly shipped wrong:**

1. **A STRIDE KNOB IS NOT A RESOLUTION KNOB when it strides a short arc.** At `every=40` the
   rung-71 arm sampled ONE base point of a 7-point riding window, skipped off-regime, and
   returned `rows#n = 0` with 27 of 83 values `None`. It would have pinned 83 values, passed
   forever and guarded nothing — [[rung77-stiffness-ledger]]'s vacuity trap at gate level. The
   detector that caught it now ships as a test.
2. **A STRUCTURAL ZERO NEEDS AN ABSOLUTE LEG.** Where a rung's own finding says a determinant is
   DEAD, relative drift is 1–19 while absolute drift is 1e-10..1e-16. An opt-in `ABS_TOL` pair
   fixes it — and the constant must be justified from BOTH sides (≥4× above measured drift AND
   ≥2.8 decades below the quantity's live scale), else it silently certifies a dead determinant
   that has come alive.

**Why:** both failures look identical to success from the outside — a green arm pinning many
values. Only a non-vacuity check and a two-sided constant tell them apart.

**How to apply:** when adding fingerprint arms, (a) check `rows#n > 0` and the `None` fraction
before trusting an arm, (b) never fit a tolerance from drift alone — bound it above by the live
scale too, (c) exclude genuinely undetermined 0/0 values by name and disclose it rather than
inventing a tolerance for them, (d) calibrate at "the round decade ≥4× above the drift", not
slice 1's "one decade above" (which would have shipped arms at 89% of band), and (e) verify
headroom RUN-vs-SHIPPED-GOLDEN, not probe-vs-probe.

Also fixed here: the golden writer now pins `newline=""`. `.gitattributes` normalises to LF on
commit, so a CRLF write left the working tree differing from the blob and would have made the
next regeneration's diff show every line changed — burying the "READ THAT DIFF" procedure the
whole file rests on. And `MAX_BYTES` was LOWERED for the first time (36,840→35,560) because the
bump that paid for the defect disclosure was given back when the defect was paid off — see
[[claude-md-is-a-reference]].
