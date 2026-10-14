---
name: rung86-three-levers
description: "Rung 86 (fuel + bleed + stator on one plant, 2026-10-14) — process lessons; I called a result \"genuine\" before checking its own registered condition and its step stability"
metadata:
  node_type: memory
  type: project
  originSessionId: 2c980df6-f956-4226-83ce-d4ede3b32ad5
  modified: 2026-10-14T19:18:45.268Z
---

Rung 86 shipped 2026-10-14 (spec `docs/rung86-spec.md`, anchor
`docs/plans/rung86-anchor-three-levers.md`, module `rust/src/three_levers.rs`, sandbox plan § 15:
the Controls view's lever choice "both"). User choices: EVERY fuel limiter, placement `n_lo` 0.65,
page change in the same batch.

**Lessons (process, not physics — the verdict is in CLAUDE.md's row):**

- **Probe before planning a rung.** A ten-minute scratch probe (advisor's first move) showed the
  three-lever plant already ran through rung 63's `neighbour` readers, so the rung became an
  INSTRUMENT plus knobs, not plant code. Check whether an "open" seam is open physics or only an
  un-asked question before designing.
- **I told the user a result was "a genuine three-lever effect" before checking it.** Its own
  registered condition was not met (the bleed-alone cell was reached), its size moved 25–34 % with
  the step, and a pair term flipped sign. The advisor caught it; I withdrew it in my next message.
  **Why:** a striking number arrives before its checks. **How to apply:** before reporting a
  result as a finding, check (1) the bar's own conditional clause, (2) the step-size bar, (3) any
  sign flip in its parts — then report.
- **"Bit-for-bit its leg-free self" overreached: the READING is not the RUN.** A limiter acting
  after the margin minimum leaves the minimum's reading identical but still trims later fuel. I
  narrowed the gate after measuring; the spec scores the frozen clause MISS as written and gives
  the narrowed form no credit, with a status note in the anchor (never edit a frozen bar).
- **An equal-length filter can drop the strongest cases.** 64 of 72 φ-floor fuel comparisons were
  excluded because the statored run starved — every one in the claim's direction. Report how many
  comparisons SURVIVE a filter, and what the excluded ones say.
- A run that records ZERO points (a floor above the start) crashed the reader (`traj[0]`); a cube
  cell must turn that into a result (NaN), never a panic — the advisor's pre-measure warning, met.

Related: [[schedule-check-flip]] (equal run lengths), [[rung63-fuel-bleed]],
[[instrument-fed-by-what-it-certifies]], [[sandbox-direction]].
