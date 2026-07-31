---
name: golden-gate-slice2
description: "Slice 2 of the PyPy plan — the golden gate's drift is set by CONDITIONING, not by rung, and the reduced-resolution worry measured backwards"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2626b397-8cb2-4435-8cda-5cf6918fc06d
  modified: 2026-07-31T07:29:12.963Z
---

Slice 2 of `docs/plans/todo-pypy-switch.md` (2026-07-31) extended the golden fingerprint from
8 arms to **26** — 18 new ones keyed BY RUNG (`prop`, `r7`, `r8`, `r10`–`r18`, `r22`–`r25`,
`r27`, `r28`), 8 044 pinned values. Slices 0–3 are now done; only slice 4 (the switch) remains.

**The finding that generalises: cross-interpreter drift is distributed by CONDITIONING, not by
rung.** Every arm at ~1e-15 reads a well-conditioned quantity. Every arm above 1e-10 reads a
difference-of-near-equals (`dS_finite`, `x_no_e_exit`), a log-ratio (`channel_ratio`), or runs an
iterative inverse (`prop.T_from_pr_t` — 78 412 ulp out of a 1-ulp property difference). Kernel F
looked like slice 1's lone outlier and got a one-off explanation; with r25 and r28 beside it, it
is a class. Consequence: **the gate's floor is set by conditioning, so tightening a constant can
never move it** — only a better-conditioned probe can.

**The reduced-resolution worry measured BACKWARDS.** The arms run at cut node counts, and the
mechanism behind the drift (naive `sum()` reassociation) grows with term count — so a reduced
probe should under-report. At 4× the terms the drift did not grow, it FELL (r24 3.26e-14 →
2.49e-14). The drift is inherited from the fixed upstream layer and the quadrature AVERAGES it
down; the inherited floor is `prop`'s own 8.91e-12, measured directly rather than inferred.

**Why:** two habits paid here and are worth repeating — (1) build the probe in the temp dir,
measure drift, and only THEN set a tolerance; (2) when a concession could be systematically
optimistic rather than merely imprecise, measure the direction of the bias instead of arguing
about it.

**How to apply:** never set a golden tolerance from a guess; the rule is one round decade above a
MEASURED drift, and every one of the 26 arms consumes 1.3–10.5% of its band. Give the conftest
spine prefix `test_golden_fingerprint_*` only to arms ≤2 s idle — heavier ones get
`test_golden_kernel_*` and are seeded slow. See [[golden-fingerprint-gate]] for the anchor rule
(the goldens are CPython's, never regenerate under another interpreter) and
[[perf-sonic-throat-and-pypy]] for the scoped bit-identity claim this whole plan came out of.

Side-finding, disclosed not repaired: `CoupledNOFreezeOutState.no_collapse_ratio` in `gas.py`
reads `self.x_no_e_entry`, a field that dataclass does not have — dead code that raises
unconditionally, called by nothing.
