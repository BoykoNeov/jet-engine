---
name: gov-fast-attack-decisions
description: "User decisions 2026-11-11 for rung 107's first seam (a finer step so the page admits a FAST attack beside the governor): attack floor 0.02 τ (4 substeps), record EVERY substep so the page shows the true sag; sandbox plan § 45"
metadata:
  node_type: memory
  type: project
  originSessionId: 39da515b-68ff-4167-9dfa-7ad4aff5d040
  modified: 2026-11-11T09:24:56.226Z
---

Rung 107's first named seam, chosen by the user 2026-11-11 ("go with your recommendation"). Not a rung: a page/solver
change, sandbox plan § 45.

Measured before asking (`W:\temp\claude\gov-fast-attack`, probe `sweep.rs`/`sag.rs`/`timing.rs`; archived under
`docs/plans/scratch-archive/gov-fast-attack/`): substeps needed for every law × 3 cases × 3 flights under 1e-4 against a
4× finer step — attack 0.1/0.05 → 2, 0.02/0.01 → 4, 0.005 → 8. Slowest march natively 1.5 s at k=1 → ~3 / 6 / 13.5 s.
Rung 107's fine-step sag numbers CONFIRMED to k=32. A fast attack's sag has a sub-0.02 dip: read on the 0.02 grid it is
~20 % shallow (1.18 vs 1.51 %) — rung 107's concession "the fast attack is off at the coarse step" was mostly the
READING grid, not the integration.

**Decisions:** attack floor beside the governor **0.02 τ** (4 substeps; option 0.05 and 0.005 declined). **Record every
substep** (show the in-between points), over "keep 0.02 spacing". Default runs (attack ≥ 0.2) stay bit-for-bit.
Promised: measure the page's extra per-run time from recording more points and report before shipping.

Process: a side-agent note asked "how slow will the slowest runs be" — answered with measured numbers before any code.
Related: [[rung107-decisions]].

**BUILT 2026-11-11** (plan § 45 "Built"): `march_ds()` = DS/4 when governed + release + attack < 0.2; every bar met
(B2 1 323 runs worst 4.4e-5; crash map 0 panics, 421 admitted; page slowest 11.2 s, default 0.02-cut run 1.9 s).

**Process lessons:**
- A gap blamed on the STEP was the READING grid: read the fine run on the coarse grid before attributing a coarse/fine
  difference to integration. **How to apply:** whenever a quantity is a max/min over marched points, compare at equal
  reading grids first.
- My test helper `fast(s)` set `tau_att` itself AFTER `..s`, so every `fast(ControlsSettings { tau_att: X, ..base })`
  silently ran at 0.02. **How to apply:** a struct-update helper must not set the field a caller varies — vary it
  OUTSIDE (`ControlsSettings { tau_att: X, ..fast(base) }`).
- I typed `python` in a Bash heredoc (the Store stub — hangs; already in [[windows-tooling-file-hazards]]). Use awk.
