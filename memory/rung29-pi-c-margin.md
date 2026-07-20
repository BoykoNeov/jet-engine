---
name: rung29-pi-c-margin
description: "Rung 29's \"one design point\" seam CLOSED on the π_c axis — confirmation (9.4× margin) plus the ENERGY = INVENTORY × COMPLETION sharpening; NOT a rung"
metadata: 
  node_type: memory
  type: project
  originSessionId: de0c81de-77a3-40e3-b655-b2933de756c3
  modified: 2026-07-20T16:39:20.179Z
---

Rung 29's `π_c` seam (concession "one design point") was checked on 2026-07-20 and shipped as
`docs/rung29-pi-c-margin.md` — a **CONFIRMATION + SHARPENING, not a rung**, in the mould of
[[rung28-beta-margin-hardened]].

**Verdict confirmed, worry did NOT invert.** Over `π_c` 2→80 the design-point bound never exceeds
0.0107% (9.4× under `_SHIFT_EARNED_TOL`), and the earned/not-earned boundary `Tt4*` stays above
1846 K. But `π_c` is **weak, non-monotone, double-edged**: `Tt4*` is bowl-shaped with an **interior**
worst case near `π_c`≈15, and the not-earned band *widens* 2.7×. So unlike β, "higher `π_c` is
protective" is **false here** — that reading is explicitly forbidden by a gate.

**The substantive result is the sharpening of [[rung29-shifting-turbine]]'s own finding.** RATIO ≠
ENERGY replaced the super-eq ratio with the radical **inventory** — but the inventory is *itself*
incomplete. Along `π_c` inventory **falls** 3.4× while the shift **rises**: the same failure, committed
by the replacement. The complete currency is **ENERGY = INVENTORY × COMPLETION** (the *recombined*
inventory). The `Tt4`-axis claim stands untouched — there both channels agree and inventory swings two
orders.

**Why: an axis that separates two opposed channels is worth more than one that confirms.** The `Tt4`
axis could never have found this because both factors move together on it.

**How to apply:** when re-checking a shipped verdict on a new axis, look for an axis where the
candidate explanation's factors *oppose* — that is where an incomplete currency is exposed. And check
the solver noise floor before quoting fine digits: here the gate's `1e-6` assert threshold was
mistaken for the solver's accuracy (it bisects to 1e-13), and an 8-digit smoothness scan settled it.

The **`M0` / flight axis** is now closed too ([[rung29-M0-margin]]) — the clean opposite of `π_c`
(monotone-protective), and it CORRECTS this doc's would-be unification: the turnover discriminator is
the `delta_h` swing, not completion headroom. Still open on the family: only rung 29's deferred
finite-rate turbine march.
