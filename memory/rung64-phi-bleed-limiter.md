---
name: rung64-phi-bleed-limiter
description: "SHIPPED rung 64 = the phi-referenced bleed LIMITER; a limiter's LAW cannot buy PROTECTION, only its PRICE; TWO predictions refuted and both refutations became content; the discriminator-before-the-anchor move"
metadata: 
  node_type: memory
  type: project
  originSessionId: b7dd229e-1cba-43d8-95c3-86a6da3cf4cf
  modified: 2026-07-30T14:36:52.697Z
---

**SHIPPED rung 64** (2026-07-30) — the φ-REFERENCED BLEED LIMITER, rung 63's named seam.
`docs/rung64-spec.md`, anchor `docs/plans/rung64-anchor-phi-bleed-limiter.md`.

**The headline:** a limiter's LAW cannot buy PROTECTION, only its PRICE. The ceiling on the
protected coordinate is `min φ` over the fully-open march — a property of `b_max`, the lever's
AUTHORITY, i.e. hardware — and `b ≡ b_max` is *itself an open-loop law*. So feedback buys
nothing on the coordinate; what it buys is the BILL (49–52 % of rung 62's schedule's bleed at
an exactly-matched coordinate, with an end-of-ramp thrust bill that is machine-zero).
INVERTS rung 61; BOUNDS 46–52's CEILING as [[rung53-variable-stator]] bounded their currency
and [[rung57-stator-schedule-transient]] their clock.

**The method move worth repeating — the DISCRIMINATOR before the anchor.** The rung's trap was
that a φ-floor on a φ-credit lever is [[rung60-matched-floor]]'s tautology verbatim, already
re-found by rung 63. So before writing ANY prediction I ran one probe to decide whether the
rung had content: `min φ` under `b ≡ b_max` versus under rung 62's schedule. It needed no new
plant, and it reframed the whole rung (my staged reading — "feedback buys authority" — died to
it). The anchor then declared that probe's results as the rung's GIVEN rather than pretending
to predict them.

**TWO predictions refuted, and both refutations became the content:**
1. *P2 "bit-for-bit under a raised, untouched clamp"* — 13 of 17 keys differ. Splits: the
   VALUES move by ≤6.6e−16 (solver path — `_solve_b` brackets on `[0, b_max]`, so the clamp is
   the Illinois upper endpoint even when it never binds), but three ARGMIN keys move by O(1),
   because **a riding floor destroys the LOCATION of the minimum it pins**. That bounds
   rungs 44–52, whose readings are all about *where* a minimum sits.
2. *P8 "the fuel leg removes exactly zero"* — it removes 2.5e−4. The truth is stronger: where
   the valve rides, `dφ/dWf = 0`, so `_surge_fuel`'s `G = φ_lim − φ(w)` is identically zero
   across its bracket. **A closed-loop lever does not disarm a second limiter on the same
   variable — it DELETES that limiter's PLANT.**

**Why:** the advisor caught what I had missed on P8 — `_surge_fuel` decides dormant-vs-
degenerate-hunt on **the sign of one ulp**, so the residual's *existence* is a coin flip, not
just its size. The claim had to rest on the `G ≡ 0` derivation with the strictly-below rows as
control, and my gate asserting `removed == 0.0` was already known false.

**How to apply:** when a measurement lands at machine zero, ask whether the code path that
produced it is *decided* by roundoff before publishing an exact number. I then made the same
mistake one level down — gated `credit == 0.0` off one run, and the next run gave −4.4e−16.
Two thresholds in this rung were set from a single measurement and both failed; see
[[rung63-fuel-bleed]] for the same failure mode in its over-claim form.
