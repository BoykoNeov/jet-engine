---
name: rung63-fuel-bleed
description: "SHIPPED rung 63 = fuel + bleed on one plant; a fuel schedule's TABLE has two guards only a mass-extracting lever reaches; I over-claimed the consequence THREE times and each was killed by my own next measurement"
metadata: 
  node_type: memory
  type: project
  originSessionId: a53d62cd-67f9-4194-adcd-208b2f840c4b
  modified: 2026-07-29T08:24:00.715Z
---

**SHIPPED rung 63** (2026-07-29) — FUEL + BLEED on one plant, rung 62's named seam.
`docs/rung63-spec.md`, anchor `docs/plans/rung63-anchor-fuel-bleed.md`.

**The result that held from derivation to ship:** a `Wf/pt3` schedule's TABLE has exactly two
guards — a choked `A4` for the ordinate `κ_ss`, rung 39's `π_LPC` cancellation for the
abscissa `n_H(Tt4)`. Rung 59 read a stator satisfying both as a fact about *schedules*. It is
a fact about levers that preserve `ṁ_face = ṁ_core`. A bleed is the only one that breaks it,
and it sits in the LP shaft balance — the one balance carrying `(1−b)` — upstream of both
guards. So a bleed moves both halves ~1e−2 where a stator moves neither (~1e−13), with
`MFP_A4` pinned at 1e−16. Derived on paper before any probe; measured exactly.

**The lesson — I over-claimed the CONSEQUENCE three times, and my own next measurement killed
each one.** The `s_eng` re-timing (+2.9…+4.2 %) is real, but:
1. *"twenty times rung 58's stator"* — died when I ran the second map shape (stator reaches
   +1.28 %, ratio 2.3, not 134).
2. *"the gap is limited-vs-dormant"* — died when I measured both readings (they agree to
   under 1 % in all twelve cells).
3. The actual cause: **rung 58 placed its schedule at `n_lo` = 0.7557 and I use 0.65** (rung
   62's placement fix). A different schedule, so its −0.162 % was never a control for my grid.

**Why:** all three shared one root — I treated `s_eng` as if it were a TABLE quantity. It is a
TRAJECTORY quantity, and a stator moves the trajectory with its table bit-identical. Structural
vs trajectory-mediated is the distinction; "presence vs absence" was never available.

**How to apply:** before quoting a prior rung's number as a control, check it was measured at
THIS rung's settings — placement, band, map, ramp rate. And when a headline rests on a ratio,
run the second map shape *before* writing it, not after. See [[rung62-bleed-schedule]] for the
placement fix that caused the mismatch, and [[rung58-composite-minselect]] for the arrow this
rung bounds.

**Also shipped:** § 3's dichotomy — a `φ` floor and the valve have no composable middle. Over
`sm` ∈ [0.3372, 0.4344] (edges = the two plants' own min `φ`) the bleed DISARMS the floor
exactly (`removed` == 0.0, armed cell bit-for-bit its leg-free march); above it both bind and
the valve's credit is exactly zero (−1.3e−15) — rung 60's tautology, with a new regime below.

**The blocker worth remembering:** rung 62 overrode `at_stator` to carry its valve, which
silently turns SIX inherited rung-57/58/59 readers into armed-vs-armed. `schedule_invariance`
on a bleed-armed machine returns `ordinate_identical = True` — *numerically rung 59's headline*
— while measuring nothing. Every rung-63 reader is built on `_isolating` (`at_lever`-based,
asserts the reference is valve-shut), and gate 2 pins the trap directly.
