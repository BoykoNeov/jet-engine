# Rung-13 external anchors — the resolved mixing PDF: a mixture-fraction β-PDF that replaces rung-12's parameterised segregation with a continuous distribution (composition variance, isolated from dwell)

Verified scalings + a machine-checked worked example that anchor the **resolved-mixing-PDF**
model (rung 13): the closure that replaces rung 12's *parameterised segregation* (`w(C)`) with a
**continuous, mean-preserving β-PDF of mixture fraction** whose single width — the **segregation**
`g` — is driven by the same Holdeman group `C = (S/H)·√J`. It pins the **Holdeman emissions optimum
at `C_opt ≈ 2.5`** from a *distribution* rather than two discrete lumps, and it makes the underlying
lesson exact: **NO is sharply peaked at stoich, so any unmixedness raises the mean NO whenever the
mean is off-stoich** — most dramatically the leaner the mean.

**A MECHANISM SEPARATION (the sharp result — read this before the "turn-up" instinct).** Rung 12's
NO-vs-`J` curve *turned back up* on the over-penetration flank. That climb came from the **dwell**
effect — an **absolute**, off-optimum-growing `τ_core` that survives `J → ∞` (a **TIME** mechanism).
Rung 13 isolates the **COMPOSITION** mechanism (the mixture-fraction variance) and **drops the
quench chain**, so it **structurally cannot** reproduce that climb. What composition variance *alone*
delivers, robustly: the emissions minimum **pinned AT `C_opt`** (perfect mixing → uniform lean →
≈0), shifting as `(H/S)²`; **both immediate flanks rise** (the convexity jump, reversing at a stoich
mean); and at *extreme* segregation ⟨EI⟩ **descends again** — the β-PDF goes **bimodal** (mass to
pure-air `ξ→0` and the rich cap, both off the stoich peak), **real physics, not an artifact**. So
composition variance pins the optimum *location*; the dwell effect (rung 12) makes the
over-penetration *climb*; **carrying the PDF through the quench (rung 15) combines them** into the
full bowl. This separation is a sharper teaching point than "rung 13 reproduces rung 12 with a PDF."

Every number was re-derived **before any production code** (project discipline); the worked example
runs on the *existing* rung-6/7/9 primitives (`_equilibrium_composition`, `_primary_aft`,
`_thermal_no`) evaluated **once** on a fixed fine mixture-fraction grid and then integrated against
the PDF — rung 13 adds **no new chemistry and no new integrator**, only a **PDF quadrature** over
the ideal bell `EI(ξ)`. The prototype lives in `M:\claud_projects\temp\rung13\proto_quad2.py`.

> **Read `docs/rung12-spec.md` first.** Rung 12 showed the mixing **optimum** is a **variance**
> effect and captured it with the *minimal* variance model — **two streams** (a mean-field bulk +
> an under-mixed core), with the segregated fraction `w(C)` and the core dwell `τ_core(C)`
> **parameterised** by hand. Rung 12 named its own successor explicitly: a **resolved mixing PDF**
> — more than two streams, a *distribution* of mixture fraction, so the segregation is a property
> of one continuous width rather than two hand-tuned lumps. Rung 13 is exactly that distribution.
> NO stays a **trace, decoupled diagnostic**, so the cycle is still **bit-for-bit rung 6**.

**The lesson — and why it is NOT "convexity/Jensen" (the framing matters).** A common shorthand
is "⟨EI(φ)⟩ ≫ EI(⟨φ⟩) by Jensen, because EI is convex." **That is wrong as stated**: the NO-vs-φ
bell is convex on its flanks but **concave at the peak**, so there is no global convexity to invoke.
The correct statement is sharper and honest:

> **NO production is sharply peaked at stoich. Unmixedness (spreading the local φ around a fixed
> mean) raises the mean NO whenever the mean is OFF-stoich — because the stoich-ward tail of the
> distribution samples the peak while the mean itself sits in a low-EI wing. The effect is largest
> the leaner (or richer) the mean.** At a **stoich mean the sign reverses**: spreading moves mass
> *off* the peak and *lowers* the mean NO.

Our combustor mean is **lean** (`φ_overall ≈ 0.40` — the dilution zone conserves mass, so the mean
mixture fraction is the *overall* value, and the Holdeman group is a dilution-jet quantity). So
segregation raises NO, and the recovered optimum is where segregation is smallest (`C_opt`). This
framing both explains the mechanism correctly and justifies the lean-mean choice.

> **Honest scope (say it at the headline, not just the footnote).** Rung 13 **isolates the one new
> effect** — composition variance acting on the *ideal* primary bell `EI(φ)` — and deliberately
> **drops the finite-quench dwell chain** (rungs 10–12's `_quench_no` trajectory). Consequently the
> optimum at `C_opt` collapses to the **well-mixed lean value ≈ 10⁻⁵ g/kg** (near zero), *deeper*
> than rung 12's finite bulk minimum. That is a **modeling boundary, not a physics claim**: primary-
> history / bulk NO is **out of scope** here. Carrying the resolved PDF **through** the rung-12
> quench (so the bulk contributes its finite NO and the PDF rides the trajectory) is the natural
> **rung-15 seam**. The **certified** content is the optimum **pinned AT the Holdeman group
> `C_opt`** with **both immediate flanks rising**, and the **`(H/S)²` shift** — recovered from a
> *continuous* PDF. (The over-penetration **climb** is *not* here — that is rung-12's dwell effect;
> see the mechanism separation above.)

---

## 1. The mechanism (numbers-before-code)

**A mean-preserving β-PDF of mixture fraction, integrated against the ideal bell:**

```
⟨EI⟩(g) = ∫₀¹ EI(φ(ξ)) · P_β(ξ; ξ̄, g) dξ ,     ξ = far/(1+far),   φ = far/f_stoich
```

- **The bell `EI(ξ)`** is the rung-9 *ideal primary* EI_NO at local equivalence ratio, computed
  from the existing primitives (`_primary_aft` → `_equilibrium_composition` → `_thermal_no`) — the
  same bell rung 9 drew, now sampled as a function of the local mixture fraction. It is **peaked at
  stoich** (`φ = 1`, `ξ ≈ 0.063`) and small in both wings.
- **The β-PDF `P_β(ξ; ξ̄, g)`** is the presumed shape universally used in turbulent-combustion PDF
  closures. It is fixed by its mean `ξ̄` and its **normalised variance (segregation)** `g ∈ (0,1)`:
  `σ² = g·ξ̄·(1−ξ̄)`, giving shape parameters `a = ξ̄·(1/g − 1)`, `b = (1−ξ̄)·(1/g − 1)`. Limits:
  `g → 0` ⇒ a **delta** at `ξ̄` (perfectly mixed); `g → 1` ⇒ mass piled at the endpoints (fully
  segregated). The mean is **preserved by construction** at every `g`.
- **The segregation `g(C) = min(g_max, k_g·|ln(C/C_opt)|)`** — the **same KINKED L1 distance from the
  Holdeman optimum** rung 12 used for its `w(C)`. `g = 0` at `C_opt` (⇒ delta ⇒ well-mixed point
  value); rising (kinked) on both flanks; capped at `g_max`. This is the one width that carries the
  whole mixing dependence.

**Why a β-PDF and not the two lumps:** rung-12's two streams *parameterised* the segregation with two
hand-tuned lumps (`w`, `τ_core`); a continuous PDF replaces that with **one** width parameter (`g`)
and resolves the *shape* of the mixture-fraction spread, not just "bulk vs core." The optimum
*location* (min at `C_opt`) then falls out of `g(C)` alone. Note this replaces the segregation
*parameterisation*, **not** rung-12's dwell mechanism — it is complementary, not a superset (the
mechanism separation, §0).

### The quadrature (the load-bearing numerical care — mean-preservation is the deliverable)

A lean mean gives shape parameter `a = ξ̄·(1/g − 1) < 1` for `g > ξ̄/(1+ξ̄) ≈ 0.026`, so the
density `ξ^{a−1}` has an **integrable singularity at ξ → 0**. Naïve uniform-in-ξ midpoint
quadrature **mis-weights** it (the first node climbs the singularity as `N` grows), corrupting the
normalisation so that ⟨ξ⟩ **drifts from ξ̄** (0.052 → 0.035 over N = 32 → 600) and ⟨EI⟩ never
converges — a closure that silently violates its own mean constraint. The fix is a **regime-aware
change of variable**:

- **`a < 1`** (the singular, lean-mean regime): substitute **`u = ξ^a`** (uniform-in-`u`). The
  Jacobian cancels the singular factor **exactly** —
  `∫ f·ξ^{a−1}(1−ξ)^{b−1} dξ = (1/a)∫ f(ξ(u))·(1−ξ)^{b−1} du`, `ξ = u^{1/a}` — leaving a bounded,
  smooth residual weight (`b ≥ 1` holds until `g ≈ 0.49`, well past `g_max`). Mean-preserving to
  **machine precision** and N-stable.
- **`a ≥ 1`** (near-delta, no singularity): the density is bounded; a windowed uniform-in-ξ grid
  over `[0, ξ̄ + 8σ]` concentrates nodes on the mass.

**Every run asserts `⟨ξ⟩ ≈ ξ̄`** (and the variance against the target) — *that check is the
deliverable more than the number is*. For speed, the equilibrium-heavy bell `EI(ξ)` is computed
**once** on a fixed fine grid and **linearly interpolated** onto the moving quadrature nodes
(`EI(ξ)` is smooth), so a `J`-sweep costs one bell, not one per point.

---

## 2. The worked example (`main.py` design point)

**Tt3 = 583.5 K, Tt4 = 1500 K, far = 0.0272 (φ_overall = 0.402), p = 7.47 bar; lean overall mean**
(`ξ̄ = 0.02646`; the well-mixed point value `EI(ξ̄) = 1.28×10⁻⁵ g/kg` — essentially zero, deep in
the lean wing). Default segregation kink: `C_opt = 2.5`, `k_g = 0.3`, `g_max = 0.3`, reusing the
rung-12 Holdeman geometry (`S`, `H = 0.10 m`). Quadrature `N = 200`.

**(a) Mean-preservation + N-stability (u = ξ^a, `g = 0.10`):** ⟨ξ⟩ = **0.026460 exactly** at every
`N` (target 0.026460), variance = **2.576×10⁻³ exactly** (target `g·ξ̄·(1−ξ̄)`), and ⟨EI⟩ converges
to **0.6726 g/kg** by `N = 64`. Across `g ∈ [0.02, 0.30]` the mean holds to **< 0.2 %** (the regime
split fixes the `a ≥ 1` case at `g = 0.02`: ⟨ξ⟩ = 0.026422 vs 0.026460).

**(b) The convexity jump — ⟨EI⟩ ≫ EI(⟨φ⟩) (lean mean, `EI(⟨φ⟩) = 1.28×10⁻⁵ g/kg`):**

| `g` | ⟨EI⟩ (g/kg) | ratio to well-mixed |
|---|---|---|
| 0.00 | 1.28×10⁻⁵ | 1× (delta ⇒ point value) |
| 0.02 | 1.18 | 9.2×10⁴× |
| 0.05 | 0.98 | 7.7×10⁴× |
| 0.10 | 0.67 | 5.3×10⁴× |
| 0.20 | 0.39 | 3.0×10⁴× |
| 0.30 | 0.25 | 2.0×10⁴× |

Segregation lifts the mean NO by **4–5 orders of magnitude** off the lean well-mixed value — the
stoich-ward tail samples the bell peak. Note ⟨EI⟩(g) is **non-monotone in `g`** (a **tested**
feature, `test_mean_ei_is_humped_in_g`): it *peaks* near `g ≈ 0.02` and **descends** as `g` grows,
because the β-PDF goes **bimodal** — mass piles at pure-air `ξ → 0` and the rich cap, **both off the
stoich peak**. This is **real physics of a mixture-fraction PDF**, not an artifact, and it is exactly
why the far-over-penetration flank of the `J`-sweep *descends* rather than climbing (§c).

**(c) The optimum — a sharp emissions minimum PINNED AT `C_opt`, `S = 0.0625 m` (`J_opt = 16`):**

| `J` | `C = (S/H)√J` | `g(C)` | **⟨EI⟩** (g/kg) | flank |
|---|---|---|---|---|
| 4   | 1.25 | 0.208 | 0.372 | under (past hump, descended) |
| 9   | 1.88 | 0.086 | 0.739 | under (climbing toward hump) |
| 16  | 2.50 | 0.000 | **≈ 0** | ← **EI-min, AT `C_opt`** (`g = 0` ⇒ well-mixed lean value) |
| 25  | 3.13 | 0.067 | 0.855 | over (near hump) |
| 36  | 3.75 | 0.122 | 0.586 | over (descending — bimodal-ward) |
| 49  | 4.38 | 0.168 | 0.453 | over (descending) |
| 64  | 5.00 | 0.208 | 0.372 | over (descending) |
| 100 | 6.25 | 0.275 | 0.279 | over (descending) |
| 144 | 7.50 | 0.300 | 0.252 | over (descending) |

⟨EI⟩ **collapses to the well-mixed lean value (≈ 0) AT `J = 16` (`C = C_opt`)** — a *sharp notch* —
and **both immediate flanks lift by orders** (the convexity jump, localized). This is the recovered
Holdeman optimum *location*, from a continuous PDF.

> **This is NOT rung 12's "falls then rises" bowl — and that difference is the result.** Rung 12's
> over-penetration **climb** came from the **dwell** effect (an absolute, growing `τ_core` surviving
> `J → ∞`) — a **TIME** mechanism. Rung 13 isolates the **COMPOSITION** mechanism and drops the
> quench chain, so it **structurally cannot** climb: past the hump the over-penetration flank
> **descends** (`J = 25 → 144`: 0.855 → 0.252) as the PDF goes bimodal. **Composition variance pins
> the optimum location; the dwell effect makes the climb; combining them (the PDF through the quench)
> is the rung-15 seam.** (Scope, §0: the minimum is ≈ 0 because this rung drops the finite-quench
> bulk NO; that bulk floor is the rung-15 addition.)

**(d) The optimum sits AT the Holdeman group — shrink `S`, it MOVES as `(H/S)²`:**

| `S` (m) | `J_opt = (C_opt·H/S)²` | EI-min lands at `J` |
|---|---|---|
| 0.0625 | 16 | **16** |
| 0.0500 | 25 | **25** |

The min lands **on `J_opt`** for both spacings (the kinked `g(C)` pins it at `C_opt`), so the
emissions optimum shifts **exactly as `(H/S)²`** (16 → 25 = (0.0625/0.05)²) — the Holdeman group
recovered from the PDF closure, not a bare dip in `J`.

**(e) The stoich-mean sign reversal (the framing test).** Re-centre the PDF at a **stoich mean**
(`φ = 1`, `EI(⟨φ⟩) = 20.7 g/kg`): spreading now **lowers** the mean — ⟨EI⟩ = 20.7 → 2.05 → 1.32 →
0.77 g/kg for `g = 0 → 0.05 → 0.10 → 0.20`. This is the honest content of the lesson: unmixedness
raises NO for an **off-stoich** mean and *lowers* it at the peak. (A naïve "convex ⇒ Jensen always
raises it" claim would fail here.)

---

## 3. The reduce gates (exact by construction — but NOT a bit-for-bit reduce to rung 12)

**Rung 13 is an ALTERNATIVE closure of the same unmixedness physics, not "rung 12 + one effect."**
Every prior rung reduced to its predecessor by zeroing a knob (rung 12: `k_u = 0`; rung 11:
`shape_n = 1`). Rung 13 does **not** — a continuous PDF does not contain the two discrete streams as
a limit of one parameter. So only **two** legitimate reduces are claimed, and exactly these:

1. **`pdf=None` ⇒ code-path-identical rung 12.** `zoned_nox` gains a `pdf` parameter (a `MixingPDF`
   config) defaulting to `None`; with it unset the path is the **exact rung-12 two-stream** result
   (or rung 11 / rung 10 / rung 9, per the existing short-circuits). The whole existing suite and
   `main.py` are untouched and stay **bit-for-bit** (hence cycle bit-for-bit rung 6).
2. **`g → 0` ⇒ the well-mixed point value `EI(⟨φ⟩)`.** The β-PDF collapses to a delta at `ξ̄`, so
   ⟨EI⟩ → `EI(φ_overall)` exactly (the delta short-circuit). This is the "perfectly mixed" limit and
   is the value the emissions minimum pins to at `C_opt`.

---

## 4. The invariants (discriminating checks, not book digits)

- **Reduce is exact** (§3): `pdf=None` is the rung-12 path (bit-for-bit); `g → 0` is the well-mixed
  point value `EI(⟨φ⟩)`.
- **Mean-preservation holds every run** (§1, §2a): `⟨ξ⟩ ≈ ξ̄` and variance ≈ target — asserted in
  the quadrature. The regime-aware `u = ξ^a` transform is what makes this exact for the lean-mean
  singular regime.
- **The emissions minimum is PINNED AT `C_opt`** (§2c): a sharp notch (⟨EI⟩ → the well-mixed lean
  value ≈ 0 at `g = 0`) with **both immediate flanks lifting by orders** — from a **continuous PDF**.
  **Not** rung-12's climbing bowl: the far-over-penetration flank **descends** (the humped ⟨EI⟩(g),
  §2b), because the over-penetration *climb* is rung-12's **dwell** effect, absent from this
  composition-only closure (the mechanism separation, §0).
- **⟨EI⟩(g) is HUMPED** (§2b): peaks near `g ≈ 0.02`, descends toward high `g` as the β-PDF goes
  bimodal (mass to pure-air / rich cap, both off the stoich peak) — a **tested** feature
  (`test_mean_ei_is_humped_in_g`), the reason the far flank descends.
- **The optimum is AT the Holdeman group `C_opt`** (§2d): `J_min = J_opt = (C_opt·H/S)²`, shifting
  **exactly** as `(H/S)²` when `S` changes (the kinked `g(C)` pins it at `C_opt` for every `S`).
- **The convexity jump is large and correct-signed** (§2b): ⟨EI⟩ ≫ EI(⟨φ⟩) for the lean mean
  (4–5 orders), and **reverses sign at a stoich mean** (§2e) — the discriminator that certifies the
  *peaked-at-stoich × off-stoich-mean* framing over the loose "convexity" claim.
- **`g(C)` is the segregation**: 0 at `C_opt`, rising (kinked) on both flanks, symmetric in `ln C`,
  capped at `g_max`.
- **Cycle bit-for-bit rung 6** (NO/N never enter `_equil_solve`; `zoned_nox` is a pure diagnostic;
  the PDF layer is opt-in via `pdf`).

---

## 5. What stays UN-anchored / deferred (state it plainly)

- **This rung ISOLATES composition variance on the IDEAL bell** — it drops the rungs 10–12 finite-
  quench dwell chain, so the optimum minimum is the **well-mixed lean value ≈ 0**, not a finite
  bulk. Primary-history / bulk NO is **out of scope**; the "optimum makes ≈ 0 NO" is a modeling
  boundary, not a physics claim. **Carrying the PDF through the rung-12 quench** (bulk + the resolved
  spread riding the trajectory) is the immediate **rung-15 seam**.
- **The β-PDF is a PRESUMED shape**, not a *transported* PDF — its width `g(C)` is still a modeled
  function of the Holdeman group, not solved from a PDF-transport / mixing-frequency equation. It
  resolves the mixture-fraction *distribution* (rung 12 resolved only two points of it) but does not
  predict its shape from first principles. A **transported / CFD PDF** is the deeper seam beyond.
- **The absolute knobs stay order-of-magnitude / un-anchored** — `S`, `k_g`, `g_max`, `C_e`, `H`,
  `U_c` (as in rungs 9–12). `C_opt ≈ 2.5` is Holdeman's uniformity value; the *pin* of the emissions
  optimum at `C_opt` is a modeling choice (the kink in `g`), disclosed — not an empirical fit to a
  book EI. Certified: the optimum pinned at the Holdeman group (both flanks rising) + the `(H/S)²`
  scaling + the sign of the unmixedness effect (and its stoich-mean reversal).
- **Super-equilibrium O / prompt (Fenimore) NO** (rung-7 seam) still deferred — and matters **most**
  in exactly the near-stoich pockets this PDF now resolves (the stoich-ward tail), so even the PDF
  ⟨EI⟩ is an equilibrium-O **lower bound** (carried from rungs 7–12).
- **The soot bound (φ ≤ 2.0)** carried from rung 9 unchanged — the bell returns 0 for `φ > 2`
  (O-starved, 5-species basis invalid), so the rich PDF tail beyond `φ = 2` contributes no NO.
- **Equilibrium-vs-frozen nozzle expansion** (the rung-6 seam) still open.

---

## Sources
- R. B. Pompei & J. B. Heywood, "The role of mixing in burner-generated carbon monoxide and nitric
  oxide," *Combustion and Flame* 19 (1972) — the classic experimental demonstration that fuel-air
  **unmixedness raises NO** above the well-mixed value at fixed mean stoichiometry (the qualitative
  external anchor for the *sign* of this rung's effect).
- J. D. Holdeman, "Mixing of multiple jets with a confined subsonic crossflow," *Prog. Energy
  Combust. Sci.* 19 (1993) — the `C = (S/H)·√J ≈ 2.5` uniformity optimum (the group the PDF width
  rides; the emissions optimum lands here by the kink, as in rung 12).
- R. W. Bilger; N. Peters — the **presumed β-PDF of mixture fraction** as the standard closure for
  turbulent non-premixed combustion (the object rung 13 adopts); the mean/variance parameterisation.
- S. R. Turns, *An Introduction to Combustion*; J. B. Heywood, *ICE Fundamentals* — the NO-vs-φ bell
  and its sharp stoich peak (carried from rungs 7–12); unmixedness as a driver of NO above the mean.
- A. H. Lefebvre & D. R. Ballal, *Gas Turbine Combustion* — dilution-zone jets, `J`, the RQL quick-
  quench (carried from rungs 9–12).
