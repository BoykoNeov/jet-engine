---
name: rung29-m0-margin
description: "Rung 29's LAST 'one design point' seam CLOSED on the M0/flight axis — CONFIRMATION (monotone-protective, opposite of pi_c) + a CORRECTION of the pi_c doc's unification; NOT a rung"
metadata: 
  node_type: memory
  type: project
  originSessionId: 7dd711cb-9857-4ddc-971e-b392fcb7b50d
  modified: 2026-07-20T16:39:48.029Z
---

Rung 29's `M0` / flight-axis seam (the last piece of its "one design point" concession, after
[[rung29-pi-c-margin]]) was checked on 2026-07-20 and shipped as `docs/rung29-M0-margin.md` — a
**CONFIRMATION + CORRECTION, not a rung**, in the mould of [[rung29-pi-c-margin]] and
[[rung28-beta-margin-hardened]]. Gates in `tests/test_rung29.py` (5 new, all green: 15 total).

**Verdict confirmed, with MORE margin than `π_c` — and it is the clean OPPOSITE of `π_c`.** Over `M0`
0.3→3.0 (fixed ambient T0=250/p0=50 kPa, `π_c`=10) the design-point bound never exceeds 0.0113% (8.8×
under `_SHIFT_EARNED_TOL`), worst case is **low-`M0` takeoff, not cruise**, and the shift is
**MONOTONE-PROTECTIVE** — no turnover (the bracket's `β`-like axis, vs `π_c`'s interior hump).

**The substantive result is a CORRECTION of [[rung29-pi-c-margin]]'s unification.** Both axes share the
`ENERGY = INVENTORY × COMPLETION` currency (recombined inventory tracks the shift on `M0` too, ~4720–5070
K/mole, +7.3%). The tempting reading — "`M0` is monotone because completion is saturated" — is WRONG. A
**`π_c`=2 control** restores completion headroom (33→61%) and the `M0` sweep is **still monotone**. The
real discriminator is the **`delta_h` SWING** (the completion *driver*): `π_c` swings `delta_h` ×11 (it
raises the compressor temperature ratio `τ_c` — a work climb) so completion outpaces inventory early and
the shift turns over; `M0` swings it only ×2–3 (it moves the ram temperature *datum* Tt2 with `τ_c` fixed,
`delta_h ∝ Tt2·(τ_c−1)`), so inventory suppression (×4.7 via `pt4`) dominates throughout → monotone.

**The flight axis is DOUBLE-EDGED in a way `π_c` is not.** Protective per operating point (shift falls,
`Tt4*` rises 1838→1961 K), YET ram heating lifts the burner-squeeze **floor** (633→1377 K) faster than the
boundary, so the earned **operating band shrinks ×2.1** while the not-earned band widens ×1.7 (earned
fraction 69%→39%). "Protective" = deeper margin per point, NOT more earned territory.

**Why: an axis that separates a candidate explanation's factors is worth more than one that confirms.**
`π_c` separated inventory vs completion (they oppose) and exposed the incomplete currency; `M0` separates
completion *headroom* from its *driver* and exposes that the driver (`delta_h` swing), not headroom, sets
turnover-vs-monotone. Same lesson as the `π_c` check, one level deeper.

**How to apply:** two diagnostic failure modes were traced honestly before trusting null cells — `M0`=0
trips a numerical `ram must not cool` edge (h_c inverse returns T0−ε), and the high-`Tt4` ceiling is the
equilibrium **burner-balance** assert (the cycle's own limit), NOT the turbine. Print exception *types*
during a sweep so a solver crash is never masked as a physical ceiling.

Disclaimed: **fixed ambient** — this is the `M0` axis, NOT a flight envelope (real high-Mach flies thinner,
lower `p0`); "supersonic cruise is safe" NOT claimed. `CO→CO₂` caveat worse (`pt4`≈17 MPa at `M0`=3).
`η_t`=1 and no rate, unchanged. Both `π_c`-family concessions now closed; only the finite-rate turbine
march remains open on rung 29.
