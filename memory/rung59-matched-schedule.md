---
name: rung59-matched-schedule
description: "SHIPPED rung 59 = the matched schedule; a schedule's ORDINATE cannot see a stator, only its INDEX can, so matching is pure re-indexing; discharged rung 58's concession as VACUOUS (not small) — its refused experiment was the one it already ran"
metadata: 
  node_type: memory
  type: project
  originSessionId: d1f0ca1e-029e-4a94-96cf-47bbc8ddf425
  modified: 2026-07-28T18:48:13.270Z
---

Rung 59 (shipped 2026-07-28) took rung 58's own named seam — re-derive the `Wf/pt3` accel
schedule ON the stator-armed machine, "what a FADEC burns in" — and found the seam was
half-empty.

**HEADLINE: a derived schedule's ORDINATE cannot see a stator; only its INDEX can.**
`κ_ss = π_b·f·MFP_A4/[(1+f)√Tt4]` is a function of `Tt4` alone — `A4` is choked so the
corrected group is *hardware* (measured `MFP = 2.962907072632e−05`, the same number at every
`Tt4` **and** every stator), and `Tt3` is pinned by the map-free shaft balances (rung 31's ★).
So matching is **pure RE-INDEXING**, measured by splicing the two tables: **abscissa carries
100.00 %, ordinate 0.00 %**.

**The per-spool SPLIT.** An LP stator cannot move `n_H(Tt4)` — rung 39's one arrow, `π_LPC`
cancels out of the HP face — so its table is invariant and `Δ_match ≡ 0` (1e−15). An HP stator
moves the index 3.3–6.7 %, and there an **unmatched leg MANUFACTURES an interaction**: 48–96×
too large on the LP spool, and of the **wrong sign** on the statored spool.

**Cross-rung: rung 58's concession discharged as VACUOUS, not small.** Its shipped docstring
and spec asserted "an armed machine derives a DIFFERENT κ_ss table" and refused the matched
variant as confounded *on that basis*. Rung 58 ran an **LP** stator, so its leg already WAS
the matched leg — identical in `s_eng`, fuel removed and `s*`, not just the margin. Its
numbers were never confounded. Both sites corrected in place (rung 28's precedent). See
[[rung58-composite-minselect]], [[rung53-variable-stator]], [[rung39-two-spool-maps]].

**Scoring: P1 HIT · P2 HIT · P3 HIT · P4 MISS · P5 SPLIT.** P3 made the 100/0 isolation
ramp-rate-INDEPENDENT (no clock — it is a property of a *table*, not a march). P4's miss beat
its hit: at slow ramp the unmatched leg is DORMANT while the matched leg binds, so an
unmatched schedule can report a limiter that **is not there**. P5 split — the abscissa share
holds to the authority edge, but `Δ_match` has an INTERIOR MAXIMUM (rung 48/50's
truncated-descent ceiling) even though the index shift is monotone.

**Lessons worth keeping:**
- **The surprise arrived as a suspected artifact.** The first probe showed the two tables
  identical *including the abscissa*, which reads exactly like "the code never saw the
  stator". Probe B (is `map_lp.vsv` nonzero after `equilibrium`? does `nu_lp` move?) is what
  separated a finding from a bug. **Always run that separation before believing an invariance.**
- **A "no-op" result can still be a rung** if it comes with a proof and a split. The advisor's
  own pre-registered rule said Δ_match ≈ 0 ⇒ ship a NEGATIVE doc; the exact identity plus the
  HP branch made it a rung instead, and the advisor retracted the rule when shown the data.
- **The advisor's proposed bit-level gate was wrong and measurement caught it**: nonzero LP
  settings agree to ~1e−13, NOT to the last bit, because `equilibrium`'s Newton converges to a
  tolerance. Tuple equality is claimed only at `v = 0` (the reduce); the invariance gate is a
  tolerance gate. Don't assert more precision than the solver delivers.
- **Rung 58's predictor did NOT extend** (its profile-credit recovery returns 3.6 % here,
  vs 86 % there) because its channel is the schedule's *state-feed* and the HP branch runs a
  *constant* setting. Reported, not buried.
- The **clamp blocker** (`AccelSchedule.cap` clamps outside its abscissa bracket, and this rung
  re-indexes that very abscissa) was the one thing that could have counterfeited the result —
  audited on every cell and now a standing assertion inside `matched_credit`.
