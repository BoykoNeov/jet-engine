---
name: both-edges-limiter-negative
description: "Rung 49's both-edges-inside-the-ramp limiter is NEGATIVE for the whole pt3-filter family; its by-product UPGRADES rung 48's law to truncated descent"
metadata: 
  node_type: memory
  type: project
  originSessionId: 557be32e-bbe5-4fe3-baeb-1b82ebe06cd0
  modified: 2026-07-27T11:52:14.686Z
---

The attack on the seam [[pt3-sensor-lag-negative]] left open — **a limiter whose
engagement AND release both land inside the ramp** (rate-limited or
washout/lead-lag-filtered `pt3`) — is **NEGATIVE**, and it closes the whole
`pt3`-filter FAMILY rather than one attack. `docs/both-edges-limiter-negative.md`.
NOT shipped, NOT a rung (the [[tau-res-negative]] / [[mixing-scale-negative]] /
[[turbine-march-negative]] / [[mixing-jicf-anchor-negative]] family).

**Why:** the unifying reason is worth remembering — **the ramp is the only clock in
the system.** Every candidate second edge (the pure lag's `ρ→1`, a slew limiter's
hard catch-up, a washout's turnover, and the `R` turnover a lead must exploit) is
manufactured by the *same* event, the fuel ramp flattening; both surge minima are
ramp-driven and hence strictly inside the ramp. The fuel-command rate limit is
rung 44's ramp-rate lever **by identity** (algebra alone: a slope-limited linear
ramp IS rung 45's ramp with `r' = Δmf/Ẇ`).

**The by-product is the real content, and it is a strict UPGRADE of
[[rung48-accel-schedule]]:** a clip **ARRESTS** the φ descent immediately and
permanently, so the limited march's minimum sits AT `s_eng`. Hence rung 48's law is
an **EDGE** condition *necessarily* — the engaged window's length, its fuel removed
and its release edge are all causally downstream of an already-determined minimum,
so the window reading is EMPTY not merely untestable. And the relief becomes
closed-form from ONE bare march:
`relief = min_{s ≤ s_eng} φ_bare − min_s φ_bare` — exact as `ds→0`, and identically
0 whenever `s_eng > s*`, which DERIVES rung 48's exact-zero crossing.

**How to apply:** do not re-attack this with another `pt3` filter. The live door is a
**φ / surge-margin FEEDBACK limiter** — the only signal with a turnover *upstream* of
a surge minimum is the surge variable itself. Two method traps recorded in the doc:
difference `d(pt3)/ds` **one-sidedly** inside the ramp (central differences straddle
the ramp-end kink and fabricate a false fall), and check the arrest to the **END** of
the trajectory, not just to `s*`, since the predictor is a global-min claim.

Also corrected rung 48's spec text: the admissible window is `m ∈ [0.15, 0.45]` (not
`[0.10, …]` — at `m=0.10` `ν_H_end` moves 1.9e-3, ~4× gate 10's own tolerance), and
gate 10 asserts 5e-4 over `MARGINS=(0.15…0.48)`, not "1e-4 for `m ≥ 0.10`".
