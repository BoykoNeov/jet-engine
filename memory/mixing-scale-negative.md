---
name: mixing-scale-negative
description: "The locally-resolved-SCALE mixing-ceiling attack was investigated in temp and deliberately NOT shipped — framing-B negative; NOT rung 25 (that slot is now finite-rate nozzle). Revisit only when δ(J) is anchored."
metadata:
  node_type: memory
  type: project
  originSessionId: cb4bb8b4-ddd1-4dee-8200-86cdda3007d2
---

The **locally-resolved SCALE** — the next attack on the standing mixing-ceiling seam after rung 24
("localizes the RATE, not the SCALE"). Construction: a penetration-growing plume σ_y=k_y·H·f,
σ_z=k_z·S·f with f=(J/J_opt)^p and jet centers δ=k_p√(SH)·J^p, plus rung-16's finite-τ_res dwell cap
(removes rung-24's monotone τ_mix backbone). Absolute rate carries a genuinely NEW O(1) constant c_D
(the "c_D = C_e·k_y" no-new-constant hope FAILED — 8× off).

**NOT A RUNG, NOT "RUNG 25".** It was originally filed as "rung 25" while that slot was empty. As of
2026-07-16 the shipped **rung 25 is finite-rate nozzle chemistry** (see [[rung25-finite-rate-nozzle]]) —
an unrelated subject. This investigation was renamed to decouple: the tracked authority is now
`docs/mixing-scale-negative.md` (self-contained — the temp folder is outside git), pointed to from
CLAUDE.md's "A real spatial / transported-CFD PDF" deferred seam. There is deliberately NO spec, no
rung-table row, no `gas.py`/test code for it.

**Investigated in `M:\claud_projects\temp\rung25\` (proto5–8, RESULT_*.md); NOT added to the ladder.**
Decision (user): "don't build yet — stop at the proto verdict."

**Verdict = framing (B), a sharp NEGATIVE.** The finite-τ_res cap DOES turn ⟨EI⟩(J) off monotone
(first in the project; rung 24 could not) and the field even carries an over-penetration penalty
(g is U-shaped in J — jet centers δ∝J^p overshoot at high J, leaving the far region under-mixed).
BUT the σ-law discriminator (proto8: sweep p ∈ {1/4,1/3,1/2}, both c_D pivots) shows the turn's
location, depth, and even its EXISTENCE all ride on the unanchored penetration exponent p: clean
interior U-min only at the hand-picked p=1/4 (J=25, C≈3.12, both pivots); at the more physically-
standard p≈1/2 (jet-in-crossflow trajectory) ⟨EI⟩ is monotone-DOWN with no interior turn. So there
is NO defensible optimum location (framing A is dead). The ⟨EI⟩-min location is set by term2/g
(the larger term), not the finite-τ_res dwell (term1).

**What the seam actually needs next** (corrected, per advisor): NOT "a pattern with a penalty" — the
penalty is already there. It needs an **anchored δ(J)/σ(J) penetration law** (a physically-derived
exponent p), because δ∝J^p relocates the penetration optimum in J and p is a cartoon free parameter.
That is the precondition before this could be shipped as anything but a hand-tuned demo.

**Contrast — [[rung26-freeze-out]] is the counter-example that DID ship.** The difference is exactly
the anchor: rung 26's chemical clock is GRI-Mech 3.0 verbatim (zero new constants), so its finding
(the moving freeze point) stands on anchored physics. This seam's penetration exponent p has no such
anchor — same "needs an anchored law" gap, opposite outcome. The lesson generalizes: an off-monotone
turn / moving optimum is only shippable when the law that locates it is anchored, not hand-picked.
