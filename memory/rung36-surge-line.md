---
name: rung36-surge-line
description: "SHIPPED rung 36 = the surge line; draws the boundary rungs 32/34/35 declined, turns the excursion into a surge margin; headline = SM schedule thin at LOW power (sign-robust despite imposed phi_surge); compounding CONFIRMS+SHARPENS rung 34 (NOT a relocation — advisor corrected this)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 282aee5d-492f-4c54-8580-b3a6125d0ee0
  modified: 2026-07-21T15:33:06.570Z
---

SHIPPED **rung 36 = the surge line** (the deliberately-declined rung-32/34/35 payoff, and the
"(c) surge line" seam rung 34 left open). A **pure diagnostic again** (like rungs 7–30), built on the
rung-34 `SpoolTransient` machinery. Requested by the user as "work on the surge line."

**What it is.** Rungs 32/34/35 all reported the transient excursion as a distance **above the running
line** (`E`) and all filed "no surge line" (a representative efficiency island is not a stability
boundary; the margin rides on where you draw it). Rung 36 draws it → turns `E` into a real **surge
margin** `SM`.

**The honest problem, faced head-on (advisor caught this before any build).** The zero-new-constant
hope — stall at the map's loading-law peak `φ=1−l/(2σ)` — is **DEAD**: lands at `φ<0` for all three
rung-34 surge shapes (`surge_flow` −2.5, `surge_pressure` −4.0, `surge_tilted` −1.125), so `ψ` is
monotone-rising across `φ>0`, no in-range stall point to inherit. A stall flow coefficient `φ_surge`
must be **IMPOSED** — the exact free parameter rung 32 objected to. **So the rung lives or dies on:
does a shape-robust SIGN survive it?** It does — measured, not assumed.

**The finding — headline + a reinforcing compounding (magnitudes disclaimed — rung-32 methodology).**
1. **THE HEADLINE — `SM` schedule is thin at LOW power** — `SM(Tt4)=π_c(φ_surge)/π_c(φ_op(Tt4))−1` on
   one speed line; the imposed floor sets the *level*, the *trend* rides on the running-line `φ_op(Tt4)`
   (1.00→0.81 as throttled) which the **choked hardware determines** (rung 31/32), not the floor.
   Sign-robust across 3 shapes × imposed `φ_surge` × an `n`-slope on the floor (incl. `k<0`). Constant-
   flow SM is only a **weak sign-check** (its magnitude extrapolates absurdly — 968% at design — so it
   is NOT billed as independent confirmation; constant-speed carries the claim). Direction = CRS Ch. 9
   (running line approaches surge at low corrected speed — why low-speed accel is surge-critical).
2. **THE COMPOUNDING — confirmation + sharpening, NOT relocation** (the advisor's blocking correction).
   `E0` (rung-34 constant-speed excursion) and `SM_N` share a **currency** (both `π_c` ratios at frozen
   `n0` to the same denominator), so a step reaches surge **iff `E0≥SM_N` ⟺ `φ_step≤φ_surge`** (airtight,
   gated). `E0` **rises** AND `SM_N` **falls** as start power drops (both point low, **reinforcing**) →
   `E0/SM_N` rises monotonically → the low-power burst is worst on **both** axes. This does **NOT**
   relocate: rung 34's `E0` is **already** largest at low power (`argmax` unchanged) — I first mis-billed
   this as "relocation"/"E(r) alone cannot say," which is FALSE (rung 34's `constant_speed_excursion`
   already puts the biggest excursion at the low-power burst). Relocation would need `E0` and `SM_N` to
   point *opposite* ways (a steeply speed-dependent real surge line could). The genuinely-new content is
   `SM_N` (varies independently of `r`; not a rescale of `E`).

**Anti-overclaim discipline (advisor's load-bearing correction).** The **crossing** into surge
(`E0≥SM_N`) rides on the disclaimed `φ_surge` (`E0` is floor-independent; only `SM_N` moves) — e.g.
`Tt4_lo=700`: `E0=0.098` fixed, `SM_N=0.109`@0.55 no-surge vs 0.073@0.65 surge. So the crossing is
**deliberately NOT gated** — gate 6 instead **asserts the flip exists** (certifying only the trend is
claimed). Also dropped the wrong "E is power-agnostic" framing (E0 rises at low power too; both
ingredients point low — the surge line's *unique* contribution is `SM_N`, the division saying *where*
the excursion becomes fatal).

**Where it lives.** `ComponentMap.phi_surge` field (default 0 ⇒ off; set via `with_phi_surge`;
`is_flat` ignores it). Methods on `SpoolTransient`: `surge_margin`/`surge_margin_schedule`,
`acceleration_binding` (the finding), `_pi_c_map` (π_c at an arbitrary map point — reproduces the
shipped `π_c` bit-for-bit, the non-tautological reduce). Never perturbs the running line or transient.
Spec `docs/rung36-spec.md`, anchor `docs/plans/rung36-anchor-surge-line.md`, tests `tests/test_rung36.py`
(7 gates), main panel `print_surge_line_table`.

**Reduce.** `phi_surge=0` ⇒ rung 34/35 bit-for-bit (rung 31–35 suites pass unchanged); `_pi_c_map` ==
shipped `π_c`; default `run()` rung-6 exact. **Leaves open:** bleed valve / variable stator (raise
`φ_surge` at low speed — rung 36 exhibits the margin they protect); a real hardware/CFD surge line
(would earn a magnitude); subsonic-branch surge margin; and rung 34's [[rung34-spool-transient]]
volume-filling/heat-soak + two-spool seams. See [[rung32-component-maps]], [[rung35-fuel-metering]].
