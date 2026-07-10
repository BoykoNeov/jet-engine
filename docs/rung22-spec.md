# Rung 22 — The resolved cross-plane / spatial PDF: the INVERSION of rung 18 (the optimum becomes an OUTPUT)

Rung 18's headline was a **negative** one: a 0-D mixture-fraction variance transport **cannot derive** the
Holdeman `C_opt` mixing optimum — with any mean-field `ω(J)` the residual `g(J)` is monotone, so rung 18
had to **impose** the coverage `ω(C=(S/H)√J)` peaked at `C_opt` (the jet spacing `S` injected by hand).
The rung 17/18 specs both named the successor as their **deferred ceiling**: a **spatial / transported-CFD
PDF** that predicts the cross-plane pattern, from which the optimum would *emerge*. Rung 22 does the honest
first step of that — it resolves the dilution **cross-plane** and shows the optimum come out.

**THE LOAD-BEARING RESULT (a positive one — the inversion of rung 18).** Resolve one dilution cell as a
2-D cross-plane (penetration `y∈[0,H]` × span `z∈[0,S]`), deliver dilution air as a Gaussian jet whose
**penetration** `δ=k_p·√(S·H)·J^(1/4)` couples the **spacing `S`** in (a fixed dilution mass ratio ⇒ jet
diameter `d_j∝√(S·H)·J^(−1/4)`, so `δ∝d_j·√J`), with a **fixed mixing-length** spread and **far-wall
reflection** so over-penetration piles air at the far wall. The normalized spatial variance `g_spatial`
then has an **interior minimum**, and — varying `S` and `H` independently — that minimum **collapses onto
a constant Holdeman group** `C=(S/H)√J`, with `J_opt` shifting **exactly as `(H/S)²`**. `C_opt` is now an
**OUTPUT** of the penetration law (the closed form: `δ` fills half the height at the optimum ⇒
`(S/H)√J=1/(4·k_p²)`, `S,H`-independent). **There is no `C_opt` knob.** This is exactly what rung 18 proved
a 0-D transport could not do; the missing ingredient was the **resolved space** (the spacing `S`), now
present.

**WHAT RUNG 22 DERIVES (the certified content — lead with the collapse).**

- **The GROUP COLLAPSE.** `g_spatial`'s minimum **VALUE** is geometry-independent (`0.0182` across
  `2×`-independent `S`/`H` variations); only `J_opt` moves, and **exactly as `(H/S)²`** (halve `S` ⇒
  `J_opt ×4`; double `H` ⇒ `×4`; `S/H` fixed ⇒ unchanged). The Holdeman group is an **output** of the
  cross-plane, not an input — the inversion of rung 18's negative result.
- **`C_opt` as an OUTPUT** `=1/(4·k_p²)` (closed form; `k_p=0.316 ⇒ C_opt≈2.5`, Holdeman's). The signature
  of the inversion: **no `C_opt` field** on the config (contrast every rung-12..18 config, which takes
  `C_opt=2.5` as an input); passing `C_opt=` is a constructor error.
- **`g_spatial < g_ceiling` always** — the resolved field's variance stays **below rung-18's derived
  two-stream ceiling** (a partial-mix realization is less segregated than the two-δ extreme). The one
  quantity carried over from rung 18 is exactly the bound the resolved field respects.

**WHAT STAYS HONEST (the concessions — stated loudly).**

- **The VALUE `C_opt≈2.5` rides on the semi-empirical `k_p`** (the penetration constant folds the
  jet-law constant × `√(mass ratio)`). Only the **collapse** and the **`(H/S)²` shift** are derived; the
  magnitude is calibrated (the `k_p`-robustness gate certifies the collapse holds at *each* `k_p`, i.e.
  what's derived is a *group*, not a number).
- **Rung 22 derives the WIDTH `g(C)`, NOT the DWELL `τ_core(C)`.** The dwell that rung 15/16 needed to pin
  the *emissions* optimum stays the **imported rung-16 kink**. So this is a **partial** closure of the
  "width *and* dwell" seam — a derived dwell (the residence-time distribution of the resolved field) stays
  the thinner deferred ceiling above rung 22.
- **The EMISSIONS optimum at `C_opt` is only LOCAL.** Through the **pure ideal bell** (rung 13), `⟨EI⟩`'s
  **global** minimum is at **max segregation** (rung-13's descending far flank, spatialized): segregation
  at a lean mean moves mass **off** the stoich peak, lowering mean NO. The reason is precise — the
  **derived floor** `g(C_opt)≈0.018` sits **just below** the ideal-bell `⟨EI⟩(g)` **hump peak `≈0.021`**,
  so the `C_opt` emissions basin is **narrow** and a wide-enough `J`-sweep beats it on the far flank. This
  is **why UNIFORMITY (`g`), not emissions, is the clean headline.** rung 18 was **not wrong** — it
  reported the real **local** behaviour (both immediate flanks up); a wide sweep exposes the descending
  flank in either. To pin the emissions global min back at `C_opt` you need the rung-16 **dwell** (which
  bakes `C_opt` in — circular here, hence deferred).

> **Read `docs/rung18-spec.md`** (the negative result rung 22 inverts, and the derived ceiling it reuses),
> **`docs/rung13-spec.md`** (the ideal-bell β-PDF — the shared vehicle), and
> **`docs/plans/rung22-anchor-spatial-pdf.md`** (numbers-before-code: the collapse table, the closed form,
> the emissions curve, the reduce). This file states only what *changes*. The Zeldovich rates, the
> equilibrium bell primitives, the `_pdf_mean_ei` quadrature, the `JetMixing`/`MixingPDF` configs +
> Holdeman group, and the "derive before you code" / conservation-assert contract carry over **unchanged**.

---

## What rung 22 adds (and what it deliberately does not)

**Adds:**

- **A resolved cross-plane config** (`SpatialPDF`) — rides on a `JetMixing`, **≤1-of-six** mutually
  exclusive with `unmixedness`/`pdf`/`pdf_quench`/`pocket_quench`/`transported` (six closures of the same
  variance physics). It computes `g(C)` as the variance of a resolved `y`-`z` field and feeds it through
  the **same rung-13 ideal bell** as rung 18 (`_pdf_mean_ei`). **The delta over rung 18 is minimal, and
  that is the point:** rung 18 got `g` from a lumped ODE with an *imposed* `ω(C)`; rung 22 gets the same
  `g` from a *resolved* cross-plane where `C_opt` is *derived*. Only the **source of `g`** changes.
  Knobs: `S`, `k_p` (**sets `C_opt=1/(4k_p²)` as an output — the only `C_opt` control, and it is not a
  `C_opt`**), `k_y`/`k_z` (fixed mixing lengths), grid sizes. **No `C_opt` field.**
- **A module helper** `_spatial_segregation(far, φ_p, S, H, J, k_p, k_y, k_z, ny, nz)` — pure-stdlib:
  builds the separable Gaussian air plume (both-wall images in `y`, spanwise-periodic in `z`),
  **root-finds the air scale so `⟨ξ⟩=ξ̄` exactly** (mean-preserving), returns `g=Var[ξ]/(ξ̄(1−ξ̄))`.
- **The combination** in `zoned_nox`: `ei_no_spatial = _pdf_mean_ei(far, …, g_spatial)` — the rung-13
  ideal bell at the resolved width. `ZonedNOxState` records
  `spatial`/`g_spatial`/`ei_no_spatial` (`C_holdeman`/`g_ceiling`/`g_seg` reused). The branch asserts
  `g_spatial < g_ceiling`.
- **`main.py` panel + `tests/test_rung22.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the layer is opt-in via `spatial`. Every cycle
  station is **bit-for-bit rung 6** (the whole rung 1–21 suite stays green). `spatial=None` runs the exact
  prior path.
- **Claim to DERIVE the emissions optimum, or the number `2.5`.** It derives the *uniformity* optimum's
  **group** (collapse + `(H/S)²` shift); the emissions optimum at `C_opt` is only *local*, and the
  magnitude rides on `k_p`.
- **Transport the DWELL, or solve a real PDF-transport / CFD field.** `τ_core(C)` stays the rung-16 kink;
  the field is a Gaussian-plume **cartoon** of one cross-plane, not a scalar-flux solve. Rung 22 yields
  the **width** `g(C)`, not the dwell spectrum nor the full resolved-PDF **shape** (which it feeds through
  the β-PDF closure, consistent with rung 18) — those stay the thinner deferred ceiling.
- **Add super-equilibrium O by default.** As with rung 21, `super_eq_o=True` threads the same `m(T)` lift
  through the shared `_pdf_mean_ei`; default `False` is the equilibrium-O lower bound. Held `φ ≤ 2.0`.

---

## The one thing that makes it work (stated loudly — it IS the rung)

**The optimum is irreducibly spatial (rung 18), so RESOLVING the space DERIVES it.** The spacing `S`
enters the penetration `δ=k_p·√(S·H)·J^(1/4)` (through the fixed-mass-ratio jet diameter), a fixed
mixing-length spread + far-wall reflection make over-penetration cost variance, and the uniformity best is
where `δ` fills half the height. Minimizing over `J` then gives `(S/H)√J=1/(4k_p²)=const` — the Holdeman
group and its `(H/S)²` shift as **outputs**. Rung 18 said 0-D can't reach this; rung 22 supplies the one
missing ingredient (the resolved cross-plane) and the optimum falls out. What rung 22 **cannot** claim
(the number, the emissions global-min, the dwell) it says as loudly as what it can.

---

## The equations — a resolved-field width over the ideal bell, no station changes

Every cycle station is **bit-for-bit rung 6**. `zoned_nox` is the rung-8..18 flow; rung 22 only adds the
resolved-field PDF integral when a `SpatialPDF` is passed:

```
SPATIAL (spatial=SpatialPDF(…), REQUIRES mixing=JetMixing(J,…)):
   ξ̄        = far/(1+far),   ξ_p = φ_p·f_st/(1+φ_p·f_st)          (assert ξ_p > ξ̄ — RQL geometry)
   δ        = k_p·√(S·H)·J^(1/4)                                   penetration (spacing S enters)
   σ_y,σ_z  = k_y·H, k_z·S                                          FIXED mixing length (J-independent)
   â(y,z)   = [Σ images at y=0,H] · [Σ spanwise-periodic images]   separable unit-mean air plume
   scale s: ⟨ξ⟩ = ξ̄   with ξ = ξ_p·(1 − clip(s·β̄·â, 0, 1))          mean-preserving (bisection)
   g_spatial = Var[ξ]/(ξ̄(1−ξ̄))                                     RESOLVED width  (assert 0 < g < g_ceiling)
   C        = (S/H)·√J                                             Holdeman group (an OUTPUT at the min)
   ei_no_spatial = _pdf_mean_ei(far, …, g_spatial)                 rung-13 ideal-bell β-PDF at the resolved width

   min_J g_spatial  ⇒  (S/H)√J = 1/(4k_p²) = C_opt   (an OUTPUT; J_opt ∝ (H/S)²)   [the collapse — the rung]
   g_spatial < g_ceiling = (ξ_p−ξ̄)/(1−ξ̄)  ∀J                                        [the rung-18 tie]
```

- **`spatial` REQUIRES `mixing`** (needs `J`, `H` for `δ` and `C`; assert). `spatial=None` keeps the exact
  prior path. **≤1-of-six** with `unmixedness`/`pdf`/`pdf_quench`/`pocket_quench`/`transported` (assert).
- **`C_opt=1/(4k_p²)` is a DERIVED property**, not a knob (`SpatialPDF.C_opt()`); the config has no `C_opt`
  field.
- **Standing asserts (rung-22 deltas):** `ξ_p>ξ̄` (RQL geometry, inside `_spatial_segregation`); the
  resolved `0<g_spatial<g_ceiling` (the two-stream bound, in the branch); the rung-7 **K-check** + **trace
  guard** at every bell node `T` and the rung-13 **mean-preservation** on the β-PDF quadrature (reused via
  `_pdf_mean_ei`); `S,k_p,k_y,k_z>0`, grid sizes `>1`; `spatial ⇒ mixing`; at most one of the six closures.

---

## Verification gates (priority order)

1. **Reduce (load-bearing).** `spatial=None` ⇒ the exact prior path (rung-22 fields `None`); a spatial call
   leaves the **primary** diagnostic (`ei_no`/`x_no_mix`) **bit-identical**. Cycle far bit-for-bit rung 6.
2. **THE COLLAPSE (headline).** `g_spatial`'s minimum **value** is geometry-independent across `2×`
   independent `S`/`H`; `J_opt` shifts **exactly as `(H/S)²`**; the argmin `C` lands at the closed-form
   `1/(4k_p²)`; **no `C_opt` knob** (constructor rejects `C_opt=`); the collapse is **robust to `k_p`**
   (the magnitude moves, the group does not).
3. **The rung-18 tie.** `g_spatial < g_ceiling` at under-, at-, and over-penetration.
4. **Emissions (honest).** `C_opt` is a **local** `⟨EI⟩` min (both immediate flanks up); the **global** min
   over a wide `J`-sweep is at an **endpoint** (max segregation, beating the `C_opt` floor); the derived
   floor sits **just below** the ideal-bell hump peak (why the basin is narrow).
5. **Grid convergence.** `ny=nz ∈ {32,48,64}` agree on `C_opt`.
6. **Cycle untouched.** `far` bit-identical.
7. **Guards.** requires-`mixing`; ≤1-of-six; RQL `ξ_p>ξ̄`; `SpatialPDF` positivity.

---

## Conservation asserts (rung-22 deltas)

- **Mean preservation:** `⟨ξ⟩=ξ̄` at the root-found air scale (the resolved field just redistributes a
  fixed total air) — the same discipline as the rung-13 β-PDF quadrature.
- **Two-stream bound:** `0 < g_spatial < g_ceiling` (a resolved partial-mix field cannot exceed the two-δ
  extreme) — asserted in the branch, tying back to rung 18's derived ceiling.
- **RQL geometry:** `ξ_p > ξ̄` (a rich primary diluting down) — asserted in `_spatial_segregation`.
- All rung-6..21 asserts (equilibrium closure, K-check, trace guard, mean-preservation) unchanged.

---

## Done when

- All rung-22 gates pass and the rung 1–21 suite stays green (`spatial=None` is the exact prior path).
- The collapse is machine-checked: `g_min` geometry-independent, `J_opt ∝ (H/S)²`, `C_opt` = the closed
  form, no `C_opt` knob, robust to `k_p` — `C_opt` is an **OUTPUT**.
- The honest scope is stated in the spec, the anchor, the panel, and the docstrings: the **value** rides on
  `k_p`; the **width** is derived, the **dwell** imported; the emissions optimum at `C_opt` is **local**.
- `main.py` shows the inversion (the collapse table + the `(H/S)²` shift) and the honest emissions curve.

---

## The deferred seam (what rung 22 leaves open on purpose)

- **A DERIVED dwell spectrum `τ_core(C)`** — the residence-time distribution of the resolved field, which
  (fed through the rung-16 per-pocket quench) would pin the *emissions* global-min back at `C_opt` without
  the circular rung-16 kink. Rung 22 derives the **width**; the dwell stays imported.
- **A real PDF-transport / CFD cross-plane** — predict the full resolved-PDF **shape** (not just its
  variance `g`) and the spatial pattern from a scalar-flux equation, which is also what would let rung 17
  claim a firing **magnitude**. Rung 22 is a Gaussian-plume **cartoon** feeding the β-PDF closure; the
  spatial pattern stays the ceiling.
