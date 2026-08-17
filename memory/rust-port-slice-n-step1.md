---
name: rust-port-slice-n-step1
description: A correction applied only FORWARD leaves its own precedent standing — and the count that refused to reconcile was the only thing that surfaced it
metadata: 
  node_type: memory
  type: project
  originSessionId: e1601c0d-fd3e-4bbc-a442-265d8f4fcc2c
  modified: 2026-08-17T09:05:47.727Z
---

Slice N step 1 (the gated-code carrier refactor) shipped 2026-08-17. P2 held exactly as
pre-registered — three edits, one file, no fourth site, 535 test names identical before and
after. The value was in three things P2 did not predict.

**1. A CORRECTION APPLIED ONLY FORWARD LEAVES ITS OWN PRECEDENT STANDING.** Slice M step 5
retired the `@pytest.mark.slow` → `#[ignore]` mapping (*port the gate, DROP the marker, restore
`#[ignore]` only against a MEASURED cost*) and retired it going forward only. One pre-existing
instance from slice I survived — `rung31.rs::gate4_running_line_and_direction`, deselecting
itself for two slices while the rule forbidding it was already in the repo. Measured cost:
**2.27 s**, in a gate whose slowest single target is 246 s. **How to apply:** when you correct a
rule, grep for the instances the old rule already produced; the correction is not shipped until
they are re-decided. Same shape as [[rust-port-documented-gate-that-doesnt-exist]].

**2. THE COUNT THAT WOULD NOT RECONCILE WAS THE FINDING.** 535 listed vs the 534 in the shipped
ledger — `--list` counts an ignored test, a run does not. The ±1 was the only symptom. **Never
wave off a small unexplained delta in a census**; that is [[rust-port-guessed-census-bars]]
running backwards. It also matters that the baseline was recovered as **NAMES, not counts**,
after `| tail -80` ate the per-target totals: exit-0 proves nothing FAILED, only a name diff
proves nothing VANISHED. (Piping a long `cargo test` through `tail` also buffers ALL output to
the end — capture whole, trim after.)

**3. TWO HAZARDS RAISED IN REVIEW WERE BOTH REAL, AND MEASURING THEM MADE EACH SHARPER THAN THE
WARNING.** Neither would have been caught by the bit-identity gate.
* *Lazy cache:* rung 56's per-row capacity must build at FIRST READ, not in the constructor —
  80 of the slice's own 160 schedule cells run on capacity-free maps. Measuring it found a
  SECOND assert with the same sentence one level up, which always fires first — so a gate driven
  through the matcher would check the outer guard while reading as though it checked the inner.
  Call the inner one directly.
* *Stale-copy:* the obvious "the sibling really was rebuilt" reads (`theta_d`, `e_d`) are
  **bit-identical at a moved setting** — the design ladder is map-independent — so that gate
  would be VACUOUS ([[rust-port-ported-test-vacuity]]). Only `cmap.vsv` moves.

**The general move:** when review flags a hazard, do not just accept and patch it — MEASURE it.
Twice here the measurement changed the gate rather than confirming it.
