---
name: rung87-valve-reference
description: "Rung 87 (the controlled valve beside the stator schedule, 2026-10-15) — process lessons; my headline comparison was nearly built into the device, and an equal-set-point case I called dormant was rung 66's degeneracy"
metadata:
  node_type: memory
  type: project
  originSessionId: 94d67dd1-4d0c-4aec-845a-0a18a4b2d49e
  modified: 2026-10-15T07:43:07.587Z
---

Rung 87 shipped 2026-10-15 (spec `docs/rung87-spec.md`, anchor
`docs/plans/rung87-anchor-valve-reference.md`, device in `rust/src/limited_bleed.rs`, reading in
`rust/src/valve_reference.rs`, sandbox plan § 16: the bleed valve's "opens on its own" mode). The
user picked rung 86's named seam.

**Lessons (process, not physics — the verdict is in CLAUDE.md's row):**

- **A comparison against a device defined as the pointwise optimum is construction.** The incidence
  valve opens exactly as far as each moment needs, so "the φ valve costs more at equal protection"
  was nearly implied; a side reviewer flagged it mid-run. I had written "a prediction, not a theorem"
  but still made it the headline bar. **How to apply:** when a new device is built to minimise the
  metric a prediction compares, register the SIZE and the one channel that could flip the sign, and
  rank it last among the findings — the spec now orders findings by how much the construction
  already implied them.
- **"Dormant by D4" was wrong for EQUAL set points.** D4 covers a floor strictly below the valve;
  the anchor's own G4 had already seen that equal set points share one pin by solver choice (rung
  66). The sandbox's browser-vs-native check found it: the matched case at 0.78/0.78 landed 1e-4
  apart in shaft speed between two builds equal to the last place elsewhere. **How to apply:** never
  use two loops on one variable at the same set point as a check or headline case; a cross-build
  comparison is a degeneracy detector — read a failure there as physics before tolerance.
- **Probe the builder before adding a knob.** Rung 62's builder silently drops a controlled valve
  (no panic, `lim = None`); the sandbox built everything through it. The advisor's first probe found
  it ([[instrument-fed-by-what-it-certifies]]'s family: a gate would have compared valve-less runs).
- **Keep test cells inside the scored set.** My first H1 gate used a cell inside the unscored band
  (ratio 6.55), and its doc line claimed it was the table's widest; fixed to scored cells.
- Heredocs with Python in the Bash tool broke twice on quoting; writing the script with Write and
  running it is reliable ([[windows-tooling-file-hazards]]).

Related: [[rung86-three-levers]], [[rung64-phi-bleed-limiter]], [[rung66-two-lag-cascade]],
[[sandbox-direction]].
