# Rung 24 — The LOCALLY-RESOLVED mixing time: the ceiling rungs 11–23 deferred by name

Every rung from 11 to 23 ran the dilution on **ONE GLOBAL mixing time** `τ_mix = H/(C_e√J·U_c)`. Each one
named the same successor in its concessions, and rung 23's §9 named it with a hypothesis attached: *"a real
scalar-dissipation field would give each cell its own mixing rate (**and could restore an off-optimum dwell
GROWTH that pins the emissions optimum non-circularly**)."* **Rung 24 asks that question.**

**THE ANSWER IS A SPLIT, and the headline is the NEGATIVE half.** The off-optimum dwell growth **IS**
derived — rung-16's imposed kink, vindicated in *shape*, from the plume's own gradients. And it is **~40%
against a ~20× trend**, so it changes **nothing**: `⟨EI⟩(J)` stays monotone and the emissions `C_opt` pin is
**still not recovered**. **Rung 24 localizes the RATE, not the SCALE.**

**THE CONSTRUCTION.** Keep rung-22's **TERMINAL field exactly** (`β_final` untouched ⇒ `g` **identical by
construction**) and change only the **PATH** to it: each cell relaxes at its **own** gradient-derived rate,
renormalized to complete at `τ_mix`:

```
  ω(y,z) = D_t·|∇ξ|²/⟨(ξ−ξ̄)²⟩,   D_t = σ_final²/(2·τ_mix)          ← rung-18's OWN ω=χ/2var, made LOCAL
  β(t)   = β_final·(1 − e^(−ω t))/(1 − e^(−ω τ_mix))                ← same endpoint, own path
  τ_cell = ∫₀^{τ_mix}(1 − β/β_final)dt = τ_mix·[1 − 1/E + 1/u],  u = ω·τ_mix,  E = 1 − e^(−u)
```

**No `C_opt`, no `τ_res`, no `b_u`, and — the new part — NO NEW CONSTANT**: `D_t` is built from `σ_final`
and `τ_mix`, both already defined. The dwell is **ANALYTIC** (rung 23 needed `nt` time-stepping; rung 24
integrates in closed form), bounded in `[0, τ_mix/2]`.

**THE SPINE — an EXACT factorization.** `τ_mix` **CANCELS** out of `u`
(`u = ω·τ_mix = σ_final²|∇ξ|²/(2·var)`), so the shape is a **pure field functional** and

```
  ⟨τ⟩(J) = τ_mix(J) · F(C)        ← EXACT. Scale × shape, cleanly separated.
```

Rung 23 has `F ≈ const` (one shared schedule ⇒ `⟨τ⟩ ∝ 1/√J`, monotone). **`F(C)` is rung 24's content** —
and it is **U-shaped with its minimum AT `C_opt`**. This factorization is *why* the rung can give a split
answer at all: it separates the thing rung 24 derives (shape) from the thing it does **not** touch (scale).

**WHY a local rate differs from rung 23 at all** (the structural point): rung-23's arrival deficit
`∫(1 − β(t)/β_final)dt` is a **RATIO**, normalized by each cell's **own** terminal value — so a stagnant
region that barely receives air has a low `β_final` too, and its ratio still reaches 1 on schedule. **The
"this region never mixes" penalty normalizes away.** A local **rate** is absolute.

**WHAT RUNG 24 CERTIFIES.**

- **THE REDUCE.** `spatial_local=None` ⇒ exact prior path. `g` is **identical to rung-22's to machine
  precision** (`<1e-12`) — *by construction*, not to a tolerance (rung 23 re-derived the terminal field
  through a time development and matched to <1%; rung 24 reuses it outright).
- **THE FACTORIZATION.** `F` is **bit-identically independent of `τ_mix`** (×0.2/×5), and every `τ(ξ)`
  scales **exactly** linearly in it. "Scale vs shape" is a *decomposition*, not a narrative.
- **THE POSITIVE — `F(C)` U-shaped, minimum AT `C_opt`.** Grid-converged (3 decimals, 32→96); both flanks
  rise; `J_opt` shifts as **`(H/S)²`** (64 → 16 → 4 as S doubles twice, `C=2.50` throughout) — rung-22's
  signature, **inherited by the DWELL**.
- **NON-CIRCULARITY (the kill test).** `ω` carries an explicit **1/g** (`var = g·ξ̄(1−ξ̄)`) and rung 22
  **already** mins `g` at `C_opt` — so *"argmin F == argmin g"* is a **TELL, not a confirmation**. The
  **g-free witness**: `⟨|∇ξ|²⟩` (no `g` algebraically) is **MAXIMAL at `C_opt`**; and `F` against a
  **frozen** reference variance still mins there. **The gradients place the optimum**; the `1/g` coupling
  amplifies the U (39% → 28.6% depth without it) but does **not** create it.
- **THE NEGATIVE (THE HEADLINE) — `⟨EI⟩(J)` stays MONOTONE-decreasing**, argmin at the strongest jet, **not
  `C_opt`**. Certified on the **real per-pocket chemistry**, not inferred from `⟨τ⟩` (EI is nonlinear in τ
  and richness-weighted — the inference had to be checked). The whole upgrade moves `⟨EI⟩` by **≤ ~4%**.
- **THE ADJUDICATION.** Rung 23 left an explicit fork: does the off-optimum dwell **GROW** (rung-16,
  imposed) or **FALL** (rung-23, derived)? — *"neither is pinned from data."* **It resolves BOTH ways, in
  different factors**: the **shape** grows (rung 16 vindicated, *and derived*), the **product** still falls
  (rung 23 vindicated). **Rung-16's kink is NOT an artifact — it is real and MIS-SCALED**: rung 16 put the
  growth in the only factor it had, at a `b_u` that made it dominate.
- **THE CORROBORATION.** Rung-23's ξ–τ correlation (rich pockets dwell longest ⇒ **add NO**) is
  **re-derived from independent physics** — gradient structure, not arrival time. `corr_ratio_local` runs
  **1.038 (J=4) → 1.026 (`C_opt`) → 1.009 (J=64)**: same sign, same under-penetration concentration as
  rung-23's independent 1.058 → 1.008.

**WHAT STAYS HONEST (the concessions — rung-18-flavored, stated loudly).**

- **RUNG 24 LOCALIZES THE RATE, NOT THE SCALE.** `⟨τ⟩`'s magnitude still rides on rung-11's un-anchored
  `τ_mix`; `F`'s magnitude on rung-22's `k_p/k_y/k_z`. **Only the shape of the shape is derived.**
- **Rung-23 §9's hope is NOT delivered, and is CORRECTED rather than inherited.** A locally-resolved mixing
  time does **not** let rung 17 claim a firing **MAGNITUDE** — magnitude rides on the global scale, which
  rung 24 does not touch. It buys a sharper **direction** and the derived `F(C)`. **Nothing more.**
- **The emissions `C_opt` pin is still NOT recovered** — rungs 16 and 23 both declined the global-min
  location; rung 24 declines it too, now having **measured why** (the shape grows; the scale wins).
- **The fine-vs-coarse gradient story** — *why* the best-mixed field is also the steepest-gradient one (at
  `C_opt` residual structure sits at the plume's own scale `σ`, fine ⇒ steep; off-optimum it piles into
  **wall-scale slabs**, coarse ⇒ shallow) — **is a property of the FIXED-σ Gaussian-plume CARTOON, not a
  general turbulent-mixing law.** Rung-22's altitude exactly (`C_opt≈2.5` rides on `k_p`). A real field's
  `σ` would grow with penetration and could blunt it.
- **`τ(ξ)` is a tight function only over the NO-relevant RICH branch.** The rate keys on `|∇ξ|`, and the
  **LEAN far-wall plateau is also flat** ⇒ also slow ⇒ also long-dwell. That long dwell is **INERT**: lean
  pockets take `_ideal_bell_ei`, which ignores `τ` entirely. Within-bin scatter **collapses toward rich**
  (0.025 ms at the richest bin vs ~0.40 ms near the mean) — tight where NO is made, loose where it isn't.
- **At `S>H` the `F`-minimum at `C_opt` is LOCAL, not global** — the cartoon's deep-over-penetration plume
  **wraps** (`δ>H`, both-wall images) and re-uniformizes. **Visible in `g` too** ⇒ an **inherited rung-22
  boundary**, not a rung-24 result. Stated, not swept.
- **Still ONE frozen cross-plane PATTERN.** Only the *relaxation* is now locally resolved; the field's
  *shape* is still the Gaussian-plume cartoon feeding the β-PDF closure. A transported-PDF/CFD solve
  remains the ceiling.

> **Read `docs/rung23-spec.md`** (the dwell this localizes, and the §9 that named this rung),
> **`docs/rung22-spec.md`** (the cross-plane both ride on), **`docs/rung16-spec.md`** (the per-pocket
> quench — the vehicle), and **`docs/plans/rung24-anchor-local-mixing-time.md`** (numbers-before-code: the
> kill test, the grid/(H/S)² robustness, the real `⟨EI⟩(J)`, the ξ–τ scatter). This file states only what
> *changes*. The Zeldovich rates, the equilibrium bell/quench primitives, the β-PDF quadrature, the
> `JetMixing` config + Holdeman group, and the "derive before you code" / conservation-assert contract
> carry over **unchanged**.

---

## What rung 24 adds (and what it deliberately does not)

**Adds:**

- **A locally-resolved-rate config** (`SpatialLocalPDF`) — rides on a `JetMixing`, **≤1-of-EIGHT** mutually
  exclusive with `unmixedness`/`pdf`/`pdf_quench`/`pocket_quench`/`transported`/`spatial`/`spatial_dwell`
  (eight closures of the same variance physics). Knobs: `S`, `k_p` (**sets `C_opt=1/(4k_p²)` as an OUTPUT —
  no `C_opt` field**), `k_y`/`k_z`, grid sizes. **No `nt`** (the dwell is analytic), **no new constant**.
- **A module helper** `_spatial_local_field(far, φ_p, S, H, J, τ_mix, k_p, k_y, k_z, ny, nz)` — pure-stdlib:
  builds the rung-22 terminal field (asserting the same mean-preservation contract), forms the local rate
  from centred gradients (zero-gradient at the walls, periodic in span), integrates the analytic dwell, and
  returns `(g_spatial, tau_of_xi, F_shape)`.
- **The combination** in `zoned_nox`: `ei_no_spatial_local = ei_no_quenched + ⟨EI_pocket_quench(ξ; τ(ξ))⟩_g`
  plus the matched-mean twin `ei_no_spatial_local_meanfield` and `corr_ratio_local`. `ZonedNOxState` records
  `spatial_local`/`g_spatial_local`/`f_shape`/`tau_mean_local`/`ei_no_spatial_local`/
  `ei_no_spatial_local_meanfield`/`ei_no_spatial_local_excess`/`corr_ratio_local` (`C_holdeman`/`g_ceiling`/
  `g_seg` reused). The branch asserts `g_spatial < g_ceiling`.
- **`main.py` panel + `tests/test_rung24.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the layer is opt-in via `spatial_local`. Every cycle
  station is **bit-for-bit rung 6** (the whole rung 1–23 suite stays green). `spatial_local=None` runs the
  exact prior path.
- **Claim the emissions global-min LOCATION** (rungs 16 and 23 declined it; rung 24 declines it *having
  measured why*), **`F`'s magnitude**, or **a global-argmin at `S>H`**.
- **Introduce a new knob or constant**, or **localize the SCALE.** One global `τ_mix` still sets the
  magnitude — the rate field is local, the time scale is not.
- **Add super-equilibrium O by default.** As with rungs 21/22/23, `super_eq_o=True` threads the same `m(T)`
  lift through the shared per-pocket quench; default `False` is the equilibrium-O lower bound. Held `φ ≤ 2.0`.

---

## The one thing that makes it work (stated loudly — it IS the rung)

**Because `τ_mix` cancels out of the local rate, the dwell factorizes EXACTLY into scale × shape — and that
is what lets a rung give an honest SPLIT answer instead of a verdict.** The shape `F(C)` is derived, U-shaped,
and minimal at `C_opt`: rung-16's off-optimum dwell growth is **real physics**, recovered from gradients with
no `C_opt` anywhere. The scale `τ_mix(J) ∝ 1/√J` is untouched, un-anchored, and **~20× stronger**. So the
question rung 23 asked — *does a locally-resolved rate pin the emissions optimum?* — answers **"the mechanism
is real and it is not enough,"** which neither "yes" nor "no" would have said. What rung 24 **cannot** claim
(the magnitude, the emissions global-min, a locally-resolved *scale*, a firing magnitude for rung 17) it says
as loudly as what it can — and it **corrects** rung-23 §9's hope rather than inheriting it.
