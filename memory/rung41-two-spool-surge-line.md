---
name: rung41-two-spool-surge-line
description: "SHIPPED rung 41 = the two-spool surge line; exposure SPLITS onto the LP spool; a LIVE zero-new-constant closed form pi*=gamma_c^(gamma_c/(gamma_c-1)); corrects rung 36's stated mechanism"
metadata: 
  node_type: memory
  type: project
  originSessionId: 91b50966-21e9-44e9-bb34-66409239bb67
  modified: 2026-07-22T15:28:55.545Z
---

**Rung 41 (shipped 2026-07-22) = the two-spool surge line** — the seam rungs 39 AND 40 both
named. `surge_margin`/`surge_margin_schedule`/`running_line_map`/`flow_coefficient_turn`/
`critical_flow_turn_pi`/`_pi_c_spool` on `TwoSpoolMapMatcher`, plus
`SpoolTransient.surge_margin_channels`. Spec `docs/rung41-spec.md`, anchor
`docs/plans/rung41-anchor-two-spool-surge.md`, gates `tests/test_rung41.py`.

**The finding:** the two-spool running line does NOT halve the low-power surge problem — it
**CONCENTRATES** it on the **LP** compressor (`φ_L` −29% over a 2:1 throttle, `φ_H` −7% and
*bounded*). Cause = rung 39's `(†)` cancellation, in **closed form, not a sign**: the HP face's
sensitivity `s_H = k(1−π_HPC^(−1/k))−1` contains **no LP quantity**; the LP's needs the
**product**, and dropping `π_HPC` from `s_L` fails by 0.8–1.0 **with the wrong sign**.

**The live anchor** (rung 36's was DEAD — its loading peak landed at `φ<0`):
`1+η_c(τ_c−1)=γ_c` ⟺ **`π_c* = γ_c^(γ_c/(γ_c−1))`** ≈ 3.2467. `γ_c` ALONE — every efficiency,
both hot-section knobs, the design split and the flight condition drop out while `Tt4*` moves
1.76×. *The closest approach is at a pressure ratio, not a throttle setting.* Kill test: the
whole +0.44% residual is the fuel fraction (`hPR`×1000 ⇒ `f`→1e-5 ⇒ residual→0, linear in `f`).

**Method lessons worth keeping:**
- **The advisor's blocking probe changed the headline.** I was about to headline `π*` as
  "closest approach to surge". It is NOT a margin extremum — `SM_N` keeps falling past it
  because the speed line flattens (`τ_c−1 ∝ n²`). Advisor made me compute `SM_H` *before*
  writing the spec. `(★)` is an **incidence/running-line-geometry** fact, and gate 7a asserts
  the divergence deliberately so the wrong reading cannot creep back.
- **That divergence became the payoff, not a caveat**: it is the **cross-rung CORRECTION of
  rung 36** (rung-28 shape) — the same turn sits INSIDE rung 36's own choked envelope, its
  gated verdict SURVIVES (no rung-36 test changed), but its stated single-channel mechanism
  ("the trend is set by `φ_op`") is corrected: φ-walk ~56% / speed-line flattening ~48%, and
  below `π*` the φ channel REVERSES.
- **A gate I wrote, ran, and had to DELETE**: "the HP running line collapses across flight
  conditions, the LP's does not." It is **vacuous** — `τ_LPC−1=K_L·x_L` and `x_H=x_L/τ_LPC` put
  `x_L`/`x_H` in **bijection**, so the whole matched state is a one-parameter family and BOTH
  collapse on EITHER ratio. My first version was measuring linear-interpolation error, not
  physics. Replaced by the quantitative sensitivity gate. Recorded in the spec as a withdrawn
  framing (rung-40 discipline).
- **Ratio, not gap.** "The margin gap widens with throttle" is FALSE — both margins tend to
  zero, so the absolute gap peaks mid-throttle. The floor-fair measure is `SM_L/SM_H`.
- **Matching the map SHAPE is not a controlled comparison.** I wrote "matched shapes + a common
  floor, so only the running line differs" — wrong, and the advisor caught it *in a rung whose
  whole point is fixing rung 36's over-attribution*. The two spools still carry different
  **design pressure ratios** (3 vs 6), so `SM_L < SM_H` **already holds at the design point**
  where `φ_L=φ_H=1` and there is no exposure difference at all — that level offset is `π_LPC`
  alone. Only the **collapsing ratio** is a running-line statement. General lesson: before
  attributing an ordering to the effect you are studying, check it at the point where that
  effect is *identically zero*.
- **Declined deliberately:** "the slip protects the LP spool" (the textbook twin-spool
  rationale) — that is a **rigid-shaft counterfactual this model does not run**. What is shown
  is the complementary truth: the LP is the **exposed** spool. Also: `slip` is a speed ratio,
  NOT a surge-proximity measure (`φ_L` is).

See [[rung36-surge-line]] (corrected by this rung), [[rung39-two-spool-maps]] (the `(†)`
cancellation this rung's mechanism rides on), [[rung40-two-shaft-transient]] (named the seam;
its complex mode measured against a boundary is still open).
