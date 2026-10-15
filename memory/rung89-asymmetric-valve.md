---
name: rung89-asymmetric-valve
description: "Rung 89 (the asymmetric valve, 2026-10-15) — process lessons; a forecast that holds the set point fixed cannot bound a re-tuned comparison where the perturbation can MOVE the set point, and a step-convergence bar must be an envelope when the integrand switches weight"
metadata:
  node_type: memory
  type: project
  originSessionId: 033978ee-0eb8-422a-92e3-5f5e5bfc5211
  modified: 2026-10-15T14:09:42.399Z
---

Rung 89 shipped 2026-10-15 (spec `docs/rung89-spec.md`, anchor
`docs/plans/rung89-anchor-asymmetric-valve.md` committed `43ae914`, reading in
`rust/src/asymmetric_valve.rs`, sandbox plan § 18: the slow valve's "Closes at its own speed"). The
user picked rung 88's named seam. H2–H5b HIT, H1 grid-sensitive in one pre-named row, H5c MISS (51 of
190): a FAST close flips the verdict too, so the closing speed moves it BOTH ways.

**Lessons (process, not physics — the verdict is in CLAUDE.md's row):**

- **Measuring the bounds before freezing flipped the headline, correctly.** The first draft said "the
  opening edge decides"; symmetric-only givens (G4 floors, G5 bounds, G6 break-even) showed nothing
  bounded the sign on the close-first rows, and the frozen H5a (slow close reverses) hit 40/40. Keep
  doing the givens-first pass ([[rust-port-measure-before-registering]]).
- **But a forecast at FIXED set points cannot bound a re-tuned comparison when the perturbation can
  move the set point the other way.** D2 protects a minimum that comes before the first close from a
  SLOWER close; a FASTER close can move the close ahead of the minimum (62 of 130 such pairs) and raise
  the set point. H4b got the side, H5c missed the size by up to 2.6×. I flagged it "likeliest miss" yet
  registered it unbounded. **How to apply:** for each direction of a perturbation, ask whether it can
  change WHICH case a run is in (here: min-first ↔ close-first); a bound derived inside one case says
  nothing once the run can leave it.
- **A step-convergence bar needs an ENVELOPE when the integrand's weight jumps.** D1′'s residual
  (weight `τ_o/τ_c` = 20 switching where the valve turns) went −4.6e-6, −5.2e-6, −8.2e-7, −5.8e-7 down
  the halvings — not ×4. My first test (fine < coarse/2.5) failed on a correct identity. Measured five
  halvings over five runs, then gated `|r| ≤ 0.25·weight·ds²` at four steps (rung 84's sawtooth).
- **Check "identical at both steps" for EVERY H, not the ones you diffed** (advisor): the member diff
  covered H3a/H5a/H5c and would have hidden H1's one fine-step row.
- **Check a monotone CLAIM mechanically before writing it.** `R` itself is not monotone (112 row breaks);
  only its SIDE is (every row and column changes side at most once, 190/190) — the headline is the side.
- **Grid scoring split into 15 below-normal pieces by cell index** (scorer arg 2) took ~12 min instead
  of hours; merge tallies by summing.
- **Re-hit the joined Rust line-continuation hazard** ([[windows-tooling-file-hazards]]) in a sandbox
  precheck message, found only by reading the file; and a `perl -0pi` replacement wrote a literal `\n`.
  Multi-line edits to source go through Edit.
