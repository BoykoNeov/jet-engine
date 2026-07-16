# Rung 23 — The DERIVED dwell spectrum: the ξ–τ correlation (completing rung 22's partial closure)

Rung 22 resolved the dilution cross-plane and **derived** the β-PDF **width** `g(C)` — the Holdeman `C_opt`
emerging as an OUTPUT. But it fed that width through the per-pocket quench with the **imported rung-16
KINKED scalar dwell** `τ_core(C)=τ_res·(1+b_u·|ln(C/C_opt)|)`, which **bakes `C_opt` in**. Rung 22's own
honest concession (`docs/rung22-spec.md` §deferred): the *"width AND dwell"* seam is only **partially**
closed — width derived, dwell still imposed. **Rung 23 closes the other half.**

**THE CONSTRUCTION.** Develop the SAME resolved cross-plane in **time**, as it mixes out over the rung-11
mixing time `τ_mix=H/(C_e·√J·U_c)`. Standard turbulent-mixing laws (`σ(t)∝√t`, `δ(t)∝t^(1/3)`), both
terminating at `τ_mix` so the `t=τ_mix` field IS rung-22's exactly (the consistency anchor). Each cell's
dwell is the **arrival-time deficit** `τ_cell=∫₀^{τ_mix}(1−β(t)/β_final) dt` — how long it lingers
un-diluted. Feed each pocket through its OWN derived dwell `τ(ξ)` (the rung-16 per-pocket quench, now with
a `tau_of_xi` callable). **No `C_opt`, no `τ_res`, no `b_u`** — the absolute scale is rung-11's `τ_mix`, the
geometry is rung-22's `k_p/k_y/k_z`. So `SpatialDwellPDF` adds **NO new dwell knob**.

**THE LOAD-BEARING RESULT (a positive one) — the ξ–τ CORRELATION.** Rich pockets are the **late-arriving**
ones (the jet reaches them last), so dwell **correlates** with composition. Rung-16's scalar `τ_core` is
ONE dwell for ALL pockets (`corr≡0` by construction) — this correlation is the physics rung 16 structurally
cannot express. Its NO effect is a **computation**, not an intuition (a longer dwell = more time crossing
stoich = more NO; but the correlated pocket is richer, further from stoich, cools faster = LESS NO — they
fight). The **matched-mean** experiment isolates it: term 2 with the correlated `τ(ξ)` vs term 2 with a
scalar dwell `=⟨τ⟩_PDF` (same `g`, same mean dwell, correlation the only difference).

**WHAT RUNG 23 CERTIFIES (the certified content).**

- **THE REDUCE / CONSISTENCY ANCHOR.** `spatial_dwell=None` ⇒ exact prior path; the terminal (`t=τ_mix`)
  field reproduces rung-22's `g_spatial` (`_spatial_dwell_field == _spatial_segregation`).
- **THE CORRELATION SIGN + under-penetration concentration.** `corr_ratio = term2_correlated /
  term2_meanfield > 1` — the correlation **ADDS NO** (rich pockets dwell long ⇒ re-make more),
  **ONE-SIGNED across `τ_mix ×0.2–×5`** (the pockets stay **formation-limited**, `max_a<1` — the Jensen
  concavity of EI-in-τ never wins), and **CONCENTRATED under-penetration** (`~+5.8%` at J=4 → `~+2.5%` at
  `C_opt` → a `~+0.8%` floor on the over-penetration flank; the shipped 32/24 grid `main.py` prints).
- **`g_spatial < g_ceiling`** — the rung-18 two-stream ceiling still bounds the resolved field.

**WHAT STAYS HONEST (the concessions — stated loudly, rung-18-flavored).**

- **The correlation's absolute MAGNITUDE / off-optimum TREND rides on rung-11's un-anchored `τ_mix`
  (`∝1/√J`).** Only the **SHAPE** (the sign + the under-penetration concentration) is derived — exactly as
  rung-22's `C_opt≈2.5` rides on `k_p` (a group/direction, not a magnitude).
- **The emissions `C_opt` pin is NOT recovered.** The derived `τ` **FALLS** off-optimum (it does not grow
  like rung-16's kink), so the over-penetration flank is not lifted — the global `⟨EI⟩` min sits off
  `C_opt`. **But rung 16 already declined the global-min LOCATION** (its GATE 3), so this is a
  first-principles **confirmation**, not a discovery. Whether the pin survives depends on whether the dwell
  **grows** off-optimum (rung-16, imposed) or **falls** with the mixing time (rung-23, derived) — **neither
  is pinned from data.** Only the **width** `g` pins `C_opt` (rung-22's uniformity result, preserved).
- **The divergence from rung 16 is HONEST, not "rung 16 is an artifact."** `τ_core(C)` and `⟨τ⟩(J)` diverge
  `~3×–70×` and trend **opposite** off-optimum, both un-anchored. Rung-16's growth crudely captures
  over-penetration slow-mixing that a single-global-`τ_mix` cartoon structurally cannot represent.
- **ONE global `τ_mix`** (a Gaussian-plume cartoon). A **locally-resolved mixing time** (each cell its own
  rate) — plus the full resolved-PDF **shape** — stays the deferred ceiling; that is also what would let
  rung 17 claim a firing **MAGNITUDE**, not just a direction.

> **RUNG 24 BUILT THAT LOCALLY-RESOLVED MIXING TIME AND SETTLED THIS** (`docs/rung24-spec.md`). Half of the
> hope above was right and half was wrong, so it is **corrected here rather than left standing**:
> **RIGHT** — the derived dwell **does** grow off-optimum after all: `τ_mix` cancels out of a local rate, so
> `⟨τ⟩ = τ_mix(J)·F(C)` **exactly**, and the field functional `F(C)` is **U-shaped with its minimum AT
> `C_opt`** (gradient-located, kill-tested). **WRONG** — that growth is ~40% against `τ_mix`'s ~20× swing, so
> **`⟨EI⟩` stays monotone and the emissions `C_opt` pin is still NOT recovered** (rung 23's negative
> **survives**, now measured rather than assumed), and it does **NOT** give rung 17 a firing **MAGNITUDE** —
> magnitude rides on the **SCALE**, which localizing the **RATE** does not touch.
> So this spec's fork — *"whether the dwell GROWS (rung-16, imposed) or FALLS (rung-23, derived) — neither is
> pinned"* — resolves **BOTH WAYS in different factors**: the **shape** grows (rung 16 vindicated *and
> derived*), the **product** falls (rung 23 vindicated). **Rung-16's kink is real and MIS-SCALED**, not an
> artifact — the reading this spec already leaned toward, now with a number behind it.

> **Read `docs/rung22-spec.md`** (the width it completes, the cross-plane it develops), **`docs/rung16-spec.md`**
> (the per-pocket quench — the vehicle), and **`docs/plans/rung23-anchor-dwell-spectrum.md`**
> (numbers-before-code: the matched-mean table, the τ_mix-sign check, the divergence). This file states only
> what *changes*. The Zeldovich rates, the equilibrium bell/quench primitives, the β-PDF quadrature, the
> `JetMixing` config + Holdeman group, and the "derive before you code" / conservation-assert contract carry
> over **unchanged**.

---

## What rung 23 adds (and what it deliberately does not)

**Adds:**

- **A derived-dwell config** (`SpatialDwellPDF`) — rides on a `JetMixing`, **≤1-of-seven** mutually
  exclusive with `unmixedness`/`pdf`/`pdf_quench`/`pocket_quench`/`transported`/`spatial` (seven closures of
  the same variance physics). Knobs: `S`, `k_p` (**sets `C_opt=1/(4k_p²)` as an output — no `C_opt`
  field**), `k_y`/`k_z`, `nt` (dwell-integral time steps), grid sizes. **No `C_opt`, no `τ_res`, no `b_u`**
  — the absolute dwell scale is rung-11's `mixing.tau_q`.
- **A module helper** `_spatial_dwell_field(far, φ_p, S, H, J, τ_mix, k_p, k_y, k_z, ny, nz, nt)` —
  pure-stdlib: develops the rung-22 plume in time, root-finds the terminal air scale so `⟨ξ⟩=ξ̄` exactly
  (mean-preserving), integrates the per-cell arrival-time deficit, and returns `(g_spatial, tau_of_xi)`
  where `tau_of_xi` is a monotone-binned interpolator `τ(ξ)` (idiomatic like `_bell_interpolator`). Asserts
  the mean-preservation contract and `g_spatial == _spatial_segregation` at the terminal.
- **`_pocket_quench_mean_ei` gains an optional `tau_of_xi` callable** — when passed, each pocket quenches at
  `τ(ξ)` instead of the scalar `τ_core`; `tau_of_xi=None` is byte-identical rung 16.
- **The combination** in `zoned_nox`: `ei_no_spatial_dwell = ei_no_quenched + ⟨EI_pocket_quench(ξ;τ(ξ))⟩_g`
  (correlated) plus the matched-mean twin `ei_no_spatial_dwell_meanfield` (scalar `⟨τ⟩_PDF`) and
  `corr_ratio` (their term-2 ratio). `ZonedNOxState` records `spatial_dwell`/`g_spatial_dwell`/
  `tau_mean_dwell`/`ei_no_spatial_dwell`/`ei_no_spatial_dwell_meanfield`/`ei_no_spatial_dwell_excess`/
  `corr_ratio` (`C_holdeman`/`g_ceiling`/`g_seg` reused). The branch asserts `g_spatial < g_ceiling`.
- **`main.py` panel + `tests/test_rung23.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the layer is opt-in via `spatial_dwell`. Every cycle
  station is **bit-for-bit rung 6** (the whole rung 1–22 suite stays green). `spatial_dwell=None` runs the
  exact prior path.
- **Claim the correlation's absolute magnitude, or the emissions global-min LOCATION.** It derives the
  correlation's **shape** (sign + under-penetration concentration); the magnitude/trend rides on `τ_mix`,
  and the emissions optimum is left to the un-anchored dwell trend (rung 16 already declined it).
- **Introduce a new dwell knob, or a locally-resolved mixing time.** One global `τ_mix` (rung-11's) sets the
  scale; a scalar-dissipation field with per-cell rates stays the deferred ceiling.
- **Add super-equilibrium O by default.** As with rungs 21/22, `super_eq_o=True` threads the same `m(T)`
  lift through the shared per-pocket quench; default `False` is the equilibrium-O lower bound. Held `φ ≤ 2.0`.

---

## The one thing that makes it work (stated loudly — it IS the rung)

**The dwell is a residence time, so RESOLVING the cross-plane in TIME derives its correlation with
composition.** Rich pockets are reached last by the dilution jet, so they dwell longest; feeding each pocket
through its own derived dwell — instead of one shared scalar — surfaces a ξ–τ correlation that ADDS NO,
one-signed wherever the pockets stay formation-limited. What rung 23 **cannot** claim (the absolute
magnitude, the emissions global-min, a locally-resolved mixing time) it says as loudly as what it can: the
correlation's **shape** is derived, its **magnitude** rides on rung-11's `τ_mix`, and the `C_opt` emissions
pin — which rung 16 already declined — is left to the un-anchored dwell trend.
