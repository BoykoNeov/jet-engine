---
name: rung28-beta-margin-hardened
description: "Rung 28's beta<1 seam re-checked — beta is exactly pressure-invariant and pi_c pushes it DOWN; CONFIRMATION, not a rung"
metadata: 
  node_type: memory
  type: project
  originSessionId: d751137b-fcde-42e5-8bea-b90ed994f46c
  modified: 2026-07-20T10:43:24.143Z
---

Rung 28's one factor-not-orders margin (`β<1`, ~0.51 hot) was re-checked on the axis it
named — higher `π_c` / hotter cycles. Result is a **CONFIRMATION that inverts the worry**;
shipped as `docs/rung28-beta-margin.md`, **not** a new rung (no new physics/constant/code
path). See [[rung28-coupled-no-march]].

- **β is EXACTLY pressure-invariant** — `c_tot²` cancels because every `R` is a product of
  two concentrations: `β = k1f·x_O·x_N2 / (x_NOe·(k2r·x_O + k3r·x_H))`. Flat to 8 digits
  over 160× in `p`. So `π_c` has **no direct channel** into the bound.
- Both indirect channels (lower `far`, lower `Tt9`) push β **DOWN**: `0.512→0.278` over
  `π_c` 10→80. **Higher `π_c` is protective**; entry `Da_NO` falls with it too, so rung 27
  hardens on the same axis.
- Two corrections *against* rung 28: its `0.512→0.513` flatness is **not** a plateau (β
  climbs monotonically in T, crosses 1 near ~3200 K), and the true whole-plane max is
  **0.5444** (`Tt4=2300`, `π_c=8`) — slightly **above** the 0.513 it quoted.
- That max is **INTERIOR**, not a scan edge: β **turns over** below `π_c≈8` because the two
  channels compete with opposite signs there (falling `π_c` raises `far` ⇒ β down, and
  raises `Tt9` ⇒ β up; composition wins at low `π_c`). The ridge is flat at ≈0.544 and is
  reached at several `(Tt4, π_c)` pairs.
- Best statement of the margin: a **temperature headroom** — the β=1 crossing sits
  **1.6–1.9×** above the hottest reachable nozzle entry, and the cycle stops solving
  (`Tt4≥2450–2500`) long before.

**Why:** this is the precedent for *closing a disclosed seam by measurement* — the honest
outcome included two findings that made rung 28 look slightly worse, and both were kept.

**How to apply:** don't re-run the π_c/pressure attack on β — it's settled. A new attempt
needs a genuinely hotter *nozzle entry* (not a hotter `Tt4` at these component
efficiencies; that route is closed by the rung-6 burner balance). Gates:
`test_beta_is_exactly_pressure_invariant`, `test_beta_falls_with_pressure_ratio`,
`test_beta_margin_is_disclosed_not_comfortable` in `tests/test_rung28.py`.
