---
name: rung79-state-coordinate
description: Rung 79 — a coordinate is a GAUGE the PLANT cannot REACH; the authoritative branch restores the original coordinate; registering the VACUITY CONDITION beat registering the result
metadata: 
  node_type: memory
  type: project
  originSessionId: d393aed3-d767-4505-a718-a69613ca4e43
  modified: 2026-08-10T07:08:54.637Z
---

Rung 79 (shipped 2026-08-10, commit `a47d512`) re-referenced rung 49's φ fuel leg to rung 60's
incidence `M_i`. `Gi = 1/φ − 1/φ_lim = Gs·h` with `h = 1/(φ·φ_lim) > 0`; `T_c` and `v` CANCEL.

**The headline was not the registered one.** `_cap_free` short-circuits iff the cap is BELOW the
schedule — exactly when the cap survives `_applied_demand` — and routes to `_surge_fuel`, which
brackets its **own hardcoded `Gs`**. It brackets the coordinated residual only when the cap is
ABOVE the schedule — exactly when `_applied_demand` discards it. So `{knob live}` and `{leg
reaches applied fuel}` are **disjoint by construction** (3 / 1363 / 0 of 1366). A third masking
mechanism, after rung 72's `min` and rung 76's law-vs-plant — and the first one located in *where
a solver short-circuits*. See [[rung78-residual-gauge]], [[rung76-fuel-dependent-cap]].

**Why:** the transferable lessons, in order of how much they cost to learn.

1. **Registering the VACUITY CONDITION beat registering the result.** The anchor pre-registered
   P2n ("if the leg-to-leg gap ≫ the perturbation, P2 holds vacuously and the spec must say so")
   *and named that as the likely disappointing outcome*. It fired on both grounds. Every other
   scored row was a confirmation, a vacuous hold, or a split. Contrast [[rung78-residual-gauge]],
   which hit three vacuity traps after the fact and rewrote its § 5 twice.
2. **An identity is not a finding, and a six-HELD anchor is a failure shape.** §§ 1–4 were
   declared UNSCORED *in advance* because "a positive multiplier preserves roots" is a theorem
   about residuals, not a fact about this plant. Run them anyway — an unconfirmed identity is an
   unrun code path — but never score them. Same family as [[rung73-applied-reference]]'s perfect
   confirmation and [[rung77-stiffness-ledger]]'s `1.000e+00`.
3. **A near-perfect number gets COUNTED, never eyeballed.** `gap_min` and `gap_med` agreed to 13
   digits over 1366 samples; counting distinct values (129 gaps, 42 φ caps, 16 accel caps) proved
   the log was real rather than one state repeated. The instinct to check was right; only the
   count settled it.
4. **`binds` is not the last selector.** Winning `min(accel, φ)` inside `_cap_fuel` is not
   reaching the plant — `_applied_demand` is a second `min` downstream. A counter modelled on
   rung 78's landed one selector too high.

**How to apply:** when a rung's core claim is provable in a few lines, put the anchor's scored
weight on the *plumbing* question (does it reach the plant?), not the theorem — and pre-register
what would make the answer vacuous. Before quoting any invariance measured through `_cap_free`,
check `bracketed` vs `fallback` **split by coordinate**: a single total cannot tell "3 slack
states in each of 2 coordinates" from "6 in the original alone".

Instrument failures, all caught by counting: the probe flag written on the *instance* (log empty
while `hits`/`binds` read 1366/1366 and looked flawless — [[rung72-shared-actuator]]'s carried-knob
trap, landing on the instrument); the vacuity guard returning `None` in its most vacuous case
(`d_max == 0`); the unsplit fallback total.

CLAUDE.md paid for the new row with **no budget bump** — the first rung in the sequence to do so.
The per-seam BUILT list had quietly re-grown into a per-rung enumeration (~60 B/rung, duplicating
the table); deleting it returned ~400 B. See [[claude-md-is-a-reference]]: check the *no-grow*
sites for re-growth before assuming the slack is only in the uncompressed ones.
