---
name: rung74-arrest-interval
description: "Rung 74's arrest is an INTERVAL with two derived edges, not a cell — and the demand coordinate has NO wall where all four loops are live; the advisor killed my first framing with a row from my own table"
metadata: 
  node_type: memory
  type: project
  originSessionId: b32d0148-c70f-48b2-96f4-cb3f82e00659
  modified: 2026-08-10T16:04:11.499Z
---

Checked the "φ-floor march with NON-ZERO initial margin" seam (2026-08-10). Doc:
`docs/rung74-arrest-interval.md`, gates in `tests/test_rung74.py`. **A CORRECTION, not rung 80** —
no plant code, no knob, no constant.

**The seam was unbuildable as written, in three ways.** Rung 79 § 9 / `rung79-gap-margin.md` § 6 /
the march audit § 6 all wanted a wall in `φ_lim ∈ (0.7731, 0.7884)`. But `0.7731162133` is the
**free operating point**, and a φ floor with authority to spare **sits EXACTLY on its wall** (6
walls, ≤1e−15) — so the named interval is the **ANTI-window**: precisely the walls that pin the
initial margin to ZERO. `0.7884` has no provenance in the rig at all. And the rig wanted was
already shipped: `(clip, 0.76)` — rungs 75/76's own wall — has all four loops live with +1.3%
initial margin. See [[rung79-gap-margin]], [[rungs72-77-march-audit]].

**The findings.**

* **The arrest is an INTERVAL, both edges DERIVED.** `(0.7731162133, ≈0.852)` in `demand`.
  Lower edge = the free operating point (bracketed 0.7731 marches / 0.7732 does not); upper edge =
  the **valve's saturation** (`b/b_max` 0.987 arrested / 1.000 not). Above it the plant does not
  recover — it **DECELERATES**, to 911.8 K at `φ_lim = 0.96`. In `clip`, no arrest at any wall.
  Rung 74 § 2.2 found, mechanised and gated the arrest; this only measures how far it reaches.
* **The `demand` coordinate has NO four-loop cell.** Above the free point → arrested; at or below
  → the leg HOLDS (breach positive at every wall) → valve and stator inert 0/341. Exhaustive on
  the wall axis, 17 walls. So every trajectory claim rungs 74–79 make in `demand` is read on an
  arrested plant or a two-loop one.

**Lessons.**

* **The advisor killed my framing with a row from my own table.** I proposed "{airflow live} ∩
  {fuel margin} = ∅ under a shared wall" — the audit's `(clip, 0.76)` row satisfies every clause
  of it *with* a shared wall. **Re-read the evidence you already pulled before building on a
  pattern.**
* **The number that doesn't parse IS the pivot signal.** I noticed the seam's interval sat on the
  wrong side of the free point, called it odd, and kept going. That anomaly was the whole finding.
* **Count the INFORMATIVE cells, not the rows.** My "0 counter-examples of 10" was 4 informative
  cells: 2 rows were the arrest (both sides at float noise) and 4 had the leg dormant. Same shape
  as [[rung78-residual-gauge]] and [[rung79-state-coordinate]]'s traps.
* **Name the definitional half as definitional.** "A φ floor commands nothing when φ ≥ φ_lim" is
  what a floor IS. So it is deliberately **NOT gated** — a test would pass forever and guard
  nothing. The measured content is which loop wins the **min-select**.
* **A refuted candidate with a positive control beats a bare concession.** The clip breach is
  monotone in `tau_f` (40×) but has a floor. Rung 73's reference — the obvious candidate — moves
  `max Tt4` by **129 K** and the breach by **zero digits**. Live knob, refuted explanation.
* **"CORRECTS N" is a debt owed in N's own spec** — edits landed in rung74 § 2.2, the march audit
  § 4/§ 6, rung79 § 9 and rung79-gap-margin § 6, not just in the new doc.
* CLAUDE.md had **0 B** free: the line was **merged into** the bullet it corrects and funded from
  15 word-deletions, budget unchanged ([[claude-md-is-a-reference]]). One substitution was
  reverted mid-pass for deleting a *fact* rather than a word.
* **Next: the SPLIT WALL** (`sm_air` beside `sm`) — arm the airflow levers above the fuel leg so
  the queue order reverses. That one IS a rung.
