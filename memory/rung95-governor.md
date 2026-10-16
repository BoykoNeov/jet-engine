---
name: rung95-governor
description: "Rung 95 (governor in the slam) process lessons — a search seeded from fixed guesses fails where the state wanders from all of them (crash map found it; fallback reached only on failure keeps scored runs bit-identical); a clamp on a solver search bound is rough at any knob setting — disclose, don't tune; a spec sentence about a neighbour rung needs its own measurement"
metadata:
  node_type: memory
  type: project
  originSessionId: c9380267-43e0-4c29-a2e4-34afc90f063b
  modified: 2026-10-16T09:54:38.726Z
---

Rung 95 (2026-10-16): `governor.rs` (the governor is the TOP of the min-select; thrust/Tt4 by instant inversion, HP
speed by commanded rate or proportional law; clamped at the fuel-air walls; metal leg). Anchor `bc14046`; H1–H6 all HIT.
Decisions: [[rung95-decisions]]. Follows [[rung94-burner-hold]].

**Lesson 1 — a search seeded from a fixed list of guesses fails where the state wanders from all of them.** The
governor's root search started from the ramp's fuel or either end's; on deep slowdowns (≈1650 → 620 K) all three were
unrunnable (ramp too lean for fast shafts, start too rich). Only the CRASH MAP found it (3 of 1 000), never the scored
grid. Fix: a fallback scan reached only where the first tries fail, so every scored run stays bit-identical — then gate
the crash map's case. **How to apply:** run the crash map before writing the spec, and read every "other" stop's message.

**Lesson 2 — a clamp on a solver's search bound is rough at every knob setting.** The commanded rate's step error on the
lean edge did not shrink with `τ_N` (erratic 1e−3–5e−4): the wall is the closure's bracket-search bound, not smooth in
the state. Set knob limits from the clean (accel) side and DISCLOSE the edge's roughness on the page; don't hunt for a
knob value that hides it.

**Lesson 3 — a spec sentence about a NEIGHBOUR rung needs its own measurement.** I first wrote "rung 46's finding no
longer holds under a governor" — never measured; caught on re-read before commit and replaced with what was measured.
Same family as [[instrument-fed-by-what-it-certifies]]: a neighbour verdict is a claim, not a flourish.

Also: the sandbox's settings panel scrolls inside itself, so a full-page screenshot shows only its top — un-clip the
panel (max-height/overflow/position) for the shot, as `browser.mjs`'s governor shot does.
