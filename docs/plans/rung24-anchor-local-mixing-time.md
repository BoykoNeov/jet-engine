# Rung-24 external anchors — the LOCALLY-RESOLVED mixing time (numbers-before-code)

Rung 23 developed the rung-22 cross-plane in **time** and derived each pocket's dwell `τ(ξ)` — but under
**ONE global `τ_mix`** (rung-11's `H/(C_e√J·U_c)`). Its own §9 named the next step and the hypothesis: *"a
real scalar-dissipation field would give each cell its own mixing rate (and could restore an off-optimum
dwell GROWTH that pins the emissions optimum non-circularly)."*

**Rung 24 asks that question and gets a SPLIT answer** — the honest kind. The off-optimum dwell **growth is
DERIVED** (rung-16's imposed kink, vindicated in *shape*), but it is **~40%** against a **~20×** global
trend, so the **emissions `C_opt` pin is STILL not recovered**: rung 23's negative **survives**, now for a
sharper reason. Framed as a question the rung answers, **not** a result it was built to produce (had the
local field still given a flat `F`, that too would have been the finding).

Every number below was re-derived first; the prototypes live in `M:\claud_projects\temp\rung24\`
(`proto.py` geometry+dwell, `proto2.py` robustness, `proto3.py` the S>H diagnosis, `proto4.py` **the
circularity kill test**, `proto5/6.py` the ξ–τ scatter, `proto7.py` the real `⟨EI⟩(J)`).

Design point = rung 16/22/23's: flight 250 K/50 kPa/M0.85, PR 10, `Tt4→1500 K`, `τ=3 ms`, `φ_p=1.5`;
`far=0.02718`, `ξ̄=0.02646`, `ξ_p=0.09278`. Holdeman `J=16, H=0.10, S=0.0625 ⇒ C=2.5`. `C_e=0.20, U_c=75 m/s`.

> **Read `docs/rung23-spec.md` first** (the dwell it localizes) and **`docs/rung22-spec.md`** (the
> cross-plane both ride on). NO stays a **trace, decoupled diagnostic** ⇒ the cycle is **bit-for-bit rung 6**.

---

## 1. THE CONSTRUCTION — same endpoint, different paths (and NO new constant)

The **terminal field is rung-22's EXACTLY** (`β_final` untouched) ⇒ **`g` is unchanged by construction** —
the reduce/consistency anchor holds *identically*, not approximately. Only the **path** to it differs: each
cell relaxes at its **OWN** rate, renormalized to complete at `τ_mix`:

```
  β(t) = β_final · (1 − e^(−ω t)) / (1 − e^(−ω τ_mix))
  τ_cell = ∫₀^{τ_mix}(1 − β/β_final)dt = τ_mix·[1 − 1/E + 1/u],   u = ω·τ_mix,  E = 1 − e^(−u)
  limits:  u→∞ (fast local mixing) ⇒ τ_cell→0 ;  u→0 (stagnant) ⇒ τ_cell→τ_mix/2      [bounded, analytic]
```

The rate is the **standard turbulent mixing frequency `ω = χ/(2·var)`** (rung-18's own form) made **local in
the numerator**:

```
  ω(y,z) = D_t·|∇ξ|² / ⟨(ξ−ξ̄)²⟩,      D_t = σ_final²/(2·τ_mix)      ← ALREADY DEFINED. NO NEW KNOB.
```

**THE EXACT FACTORIZATION (elegant, and load-bearing).** `τ_mix` **CANCELS** out of `u`:
`u = ω·τ_mix = σ_final²·|∇ξ|²/(2·var)` — no `τ_mix` anywhere. So **`F` is a PURE FIELD FUNCTIONAL** and

```
  ⟨τ⟩(J) = τ_mix(J) · F(C)          ← EXACT, not a fit: scale × shape, cleanly separated
```

Rung 23 has `F ≈ const` (one shared schedule ⇒ `⟨τ⟩ ∝ 1/√J`, monotone). **`F(C)` is rung 24's derived
content.**

**Why a local rate can differ from rung 23 at all** (the structural point): rung-23's arrival deficit
`∫(1 − β(t)/β_final)dt` is a **RATIO**, normalized by each cell's **own** terminal value — so a stagnant
region that barely receives air has a low `β_final` too, and its ratio still reaches 1 on schedule. The
*"this region never mixes"* penalty **normalizes away**. A local **rate** is absolute.

**No `C_opt`, no `τ_res`, no `b_u`, no new constant** — the non-circularity signature, extended.

---

## 2. THE CIRCULARITY KILL TEST (this gates the positive claim's WORDING)

`ω = D_t|∇ξ|²/var` and `var = g·ξ̄(1−ξ̄)` ⇒ `u` carries an explicit **1/g** — and rung 22 **already**
minimizes `g` at `C_opt`. So "argmin `F` == argmin `g`" is a **TELL, not a confirmation**: `F`'s minimum
could be *built into the rate definition*. Decomposed (`proto4.py`, 64×64):

> **Read the evidence columns exactly as labelled — only ONE of them is `g`-free.** `⟨|∇ξ|²⟩` carries no `g`
> algebraically: **that is the whole kill test.** `F(var_FIXED)` is a **cross-check, not a second witness** —
> with `σ_final=k_y·H` J-independent and `var_ref` frozen, `u_fixed ∝ |∇ξ|²` per cell, so it is the *same
> fact* viewed through `τ_cell`'s nonlinearity. And **`ℓ² = var/⟨|∇ξ|²⟩` is NOT `g`-free** — its numerator
> *is* `var = g·ξ̄(1−ξ̄)`, so `argmin ℓ²` is `argmin[g/⟨|∇ξ|²⟩]`, minimal partly *because `g` is*. It is
> listed **only** as a length-scale illustration and **carries no weight in the verdict**.

| J | C | `g` | **`⟨\|∇ξ\|²⟩`** *(the g-free witness)* | `ℓ²` *(illustration — carries g)* | `ℓ` (mm) | `F(var_local)` | `F(var_FIXED)` *(cross-check)* |
|---|---|-----|------------|---------------------|----------|----------------|----------------|
| 1   | 0.62  | 0.04269 | 2.967 | 3.71e-4 | 19.25 | 0.4329 | 0.3945 |
| 4   | 1.25  | 0.03155 | 3.641 | 2.23e-4 | 14.94 | 0.3911 | 0.3543 |
| 9   | 1.88  | 0.02238 | 4.254 | 1.36e-4 | 11.64 | 0.3401 | 0.3233 |
| **16** | **2.50** | **0.01835** | **4.328** | **1.09e-4** | **10.45** | **0.3154** | **0.3154** |
| 25  | 3.12  | 0.02147 | 4.274 | 1.29e-4 | 11.38 | 0.3348 | 0.3212 |
| 64  | 5.00  | 0.03963 | 2.927 | 3.49e-4 | 18.68 | 0.4264 | 0.3858 |
| 400 | 12.50 | 0.04905 | 3.562 | 3.55e-4 | 18.83 | 0.4392 | 0.4057 |

- **THE WITNESS — argmax `⟨|∇ξ|²⟩` at C=2.50.** Gradients are **steepest** at `C_opt`. **No `g` anywhere in
  this quantity**: two fields at equal `g` can carry different `⟨|∇ξ|²⟩`. This single fact **is** the kill test.
- **The cross-check — argmin `F(var_FIXED)` at C=2.50.** With the `1/g` channel **removed entirely**
  (variance frozen at the `J=16` value) the minimum **does not move**. Not an independent witness
  (`u_fixed ∝ |∇ξ|²`), but it confirms the gradient fact **survives `τ_cell`'s nonlinearity** — i.e. that it
  actually reaches the dwell.
- **The independent corroboration — same `g`, different `F`.** `g=0.01835` gives `F=0.233` at S=0.03125/J=64
  but `F=0.315` at S=0.0625/J=16: **`F` is not a function of `g`.**
- *(`ℓ²` mins at C=2.50 too — 10.45 mm vs ~19 mm on both flanks — but it carries `g` in its numerator and is
  shown only to make the length scale concrete.)*

**VERDICT: the gradients locate `C_opt`, the `1/g` factor does not.** The `ω=χ/2var` coupling to rung-22's
`g` **amplifies** the U (depth **39%** with `var_local` vs **28.6%** with `var_FIXED` ⇒ ~¾ gradient-driven)
but **does NOT create it**. The evidence is **one `g`-free witness plus a cross-check** — *not* three
independent tests.

**WHY the best-mixed field is also the steepest-gradient one** (the skeptic's question, answered at the
honest altitude): at `C_opt` the jet fills to mid-height, so the residual structure sits at the **plume's own
scale `σ`** — fine, hence steep; off-optimum the air piles into **wall-scale slabs** (near-wall at low `C`,
far-wall at high `C`) — coarse, hence shallow. **This fine-vs-coarse behaviour is a property of the FIXED-σ
Gaussian-plume CARTOON, not a general turbulent-mixing law** — exactly the altitude of rung-22's
"`C_opt≈2.5` rides on `k_p`". A real field's `σ` would grow with penetration and could blunt it.

---

## 3. THE LOAD-BEARING POSITIVE — `F(C)` is U-shaped with its minimum AT `C_opt`

The **derived off-optimum dwell GROWTH** — the qualitative shape rung 16 **imposed** as
`τ_core=τ_res(1+b_u|ln(C/C_opt)|)`, now **derived from the plume's own gradients**.

**Grid-converged** (`proto2.py`; `F` at J = 1, 4, 9, 16, 36, 64, 144, 400):

| grid | `F` values | argmin |
|------|-----------|--------|
| 32×32 | 0.434 0.393 0.341 **0.316** 0.371 0.428 0.439 0.439 | J=16 (C=2.50) |
| 48×48 | 0.433 0.392 0.340 **0.316** 0.370 0.427 0.439 0.439 | J=16 (C=2.50) |
| 64×64 | 0.433 0.391 0.340 **0.315** 0.370 0.426 0.439 0.439 | J=16 (C=2.50) |
| 96×96 | 0.433 0.391 0.340 **0.315** 0.369 0.426 0.439 0.439 | J=16 (C=2.50) |

Three decimals stable 32→96. Fine J grid (64×64): `F` = 0.3401 (J=9), 0.3228 (12), **0.3154 (16)**, 0.3203
(20), 0.3348 (25) — **argmin `F` at J=16, exactly co-located with argmin `g`**.

**THE `(H/S)²` COLLAPSE — rung-22's signature, inherited by the DWELL** (`proto3.py`, 64×64): the minimum is
a function of the Holdeman group `C=(S/H)√J` **alone**, so `J_opt` must shift as `(H/S)²`:

| S (m) | S/H | argmin `g` | argmin `F` | C at min | predicted `J_opt=(H/S)²·C_opt²` |
|-------|-----|-----------|-----------|----------|-------------------------------|
| 0.03125 | 0.312 | J=64 | **J=64** | **2.50** | 64.0 ✔ |
| 0.0625  | 0.625 | J=16 | **J=16** | **2.50** | 16.0 ✔ |
| 0.125   | 1.250 | J=4  | **J=4** *(local)* | **2.50** | 4.0 ✔ |

`J_opt = 64 → 16 → 4` as S doubles twice — **exactly `(H/S)²`**, at fixed `C=2.50`.

---

## 4. THE NEGATIVE HEADLINE — the emissions `C_opt` pin is STILL not recovered (certified on real chemistry)

Not inferred from `⟨τ⟩`: the **actual per-pocket quench** run with rung-24's `τ(ξ)` (`proto7.py`, 32×32,
`n_quad=160`, the rung-23 twin alongside):

```
    J      C        g  tmix(ms)   <t>24  F=<t>/tm    EI_24    EI_23     d%  max_a
    4   1.25  0.03155     3.333   1.061    0.3183   3.3197   3.4647  -4.18  0.553
    9   1.88  0.02238     2.222   0.574    0.2583   2.6304   2.7151  -3.12  0.540
   16   2.50  0.01833     1.667   0.383    0.2296   2.2544   2.3022  -2.08  0.532   <- C_opt
   25   3.12  0.02146     1.333   0.349    0.2621   2.0541   2.0839  -1.43  0.526
   36   3.75  0.02722     1.111   0.318    0.2864   1.8974   1.9233  -1.35  0.522
   64   5.00  0.03963     0.833   0.286    0.3435   1.6315   1.6217  +0.60  0.518
  144   7.50  0.04782     0.556   0.171    0.3084   1.3651   1.3765  -0.83  0.517
```

- **`⟨EI⟩₂₄` is MONOTONE-DECREASING in J** — argmin at the sweep max (J=144), **NOT `C_opt`**. Same as
  rung 23's. **The emissions optimum is NOT recovered.**
- **WHY, quantified — the factorization does the explaining.** `⟨τ⟩ = τ_mix(J)·F(C)`: `F`'s U is worth
  **~1.5×** over this sweep (~39% over the full J=1–400), while `τ_mix ∝ 1/√J` swings **6×** here (**~20×**
  over J=1–400). **The scale swamps the shape.** The derived growth is real and in the right place — and
  still an order of magnitude too weak to turn `⟨EI⟩` around.
- **The whole locally-resolved upgrade moves `⟨EI⟩` by ≤ ~4%** (−4.2% … +0.6% vs rung 23). Formation-limited
  throughout (`max_a<1`), as in rungs 16/23.

**THE ADJUDICATION (what the fork actually resolves to).** Rung 23 left it open: does the off-optimum dwell
**GROW** (rung-16, imposed) or **FALL** (rung-23, one global `τ_mix`)? **Both, in different factors** — the
**shape** `F(C)` grows off-optimum (rung 16 vindicated, and *derived*); the **product** `⟨τ⟩` still falls
(rung 23 vindicated). Rung-16's kink is **not an artifact** — it is a real effect, **mis-scaled**: rung 16
put the off-optimum growth in the *only* factor it had, at a magnitude (`b_u`) that made it dominate. **The
emissions `C_opt` pin needs the growth to beat `1/√J`; the derived growth does not come close.**

---

## 5. THE ξ–τ SCATTER — the vehicle's honest scope (checked before code, not after)

Rung-23's `τ` was **monotone in ξ** by construction (late-arriving = rich = long dwell). Rung-24's rate keys
on **|∇ξ|**, and **both** plateaus are flat — the rich near-wall **and the LEAN far-wall** — so a naive read
says lean cells dwell long too. Resolved (`proto6.py`, 32×32):

- **The lean long-dwell is INERT.** Lean-of-mean pockets take the `_ideal_bell_ei` branch, which **ignores
  `τ` entirely** (rung-16 scope: a lean pocket never re-crosses stoich, so it has no finite quench). The
  `ξ→0` far-wall plateau's `τ=0.477·τ_mix` **never reaches the chemistry**.
- **Over the NO-relevant RICH branch, `⟨τ|ξ⟩` rises and tightens**: at J=4, 0.85 → **1.62 ms** across the
  rich pockets (fully monotone at J=64; a mild non-monotone wiggle just above `ξ̄`, at `φ≈0.43–0.69`, where
  the pockets are barely rich and make little NO). Within-bin `sd` **collapses toward rich** — 0.025 ms at
  the richest bin vs ~0.40 ms near the mean — so `τ` is a **tight function of ξ exactly where NO is made**,
  and a **wide average only where it doesn't matter**.
- **Rung-23's correlation is CORROBORATED from an INDEPENDENT mechanism.** Rich pockets dwell longest here
  too — but via **gradient structure**, not arrival time. Two unrelated constructions, same sign.

---

## 6. THE INHERITED CARTOON LIMIT (the S>H wrap-around — stop-and-explain, not swept)

At `S=0.125` (`S/H=1.25`), `F`'s minimum at `C=2.50` is **LOCAL, not global**: at extreme `J` (256, 400)
`F` descends again to 0.377, dipping **~2%** below the `C_opt` value (0.384) and capturing the global argmin.

**Diagnosed, and it is NOT a rung-24 defect: `g` does the same thing** (`proto3.py`: `g` falls 0.051 at
J=64 → **0.023** at J=400). Cause: `δ = k_p√(S·H)·J^(1/4)` reaches **0.158 m** against `H=0.10` — the plume
over-penetrates so far that, with mirror images at **both** walls, it **wraps and re-uniformizes** the
field. An **inherited rung-22 geometry-cartoon boundary** at `S>H`, visible in the **width** too. Rung 22's
argmin `g` survives it (0.0184 at J=4 still beats 0.0232); **`F`'s margin is thinner, so it does not.**

**Therefore the certified claim is about the LOCAL minimum** (`C=2.50` at every spacing, `(H/S)²`-collapsing)
— **not** a global-argmin claim at `S>H`. Stated, not swept.

---

## 7. The reduce (exact by construction) + the knobs

- **`spatial_local=None`** ⇒ branch never entered; rung-24 fields stay `None`; the rung 1–23 suite untouched.
- **`g` is IDENTICAL to rung 22/23's** — same terminal field, by construction (not a tolerance).
- **Cycle bit-for-bit rung 6** (NO/N never enter `_equil_solve`); the primary `ei_no`/`x_no_mix` untouched.
- **The uniform-rate limit** (`ω` forced constant across cells) ⇒ `τ` constant ⇒ **zero ξ–τ correlation** —
  the structural check that the correlation is the **local rate field's** doing.

| knob | value | role |
|------|-------|------|
| `S`, `k_p`, `k_y`, `k_z` | 0.0625 m, 0.316, 0.28, 0.28 | rung-22 geometry (unchanged; `C_opt=1/(4k_p²)`) |
| `ny, nz` | 32 (panel/tests) / 48 (default) | cross-plane grid (`F` stable to 3 decimals 32→96) |
| `n_bell, n_quad` | 40/56 (panel/tests), 120/160 (default) | rung-16 per-pocket grids |
| **`D_t`** | **`σ_final²/(2·τ_mix)`** | **REUSED — no new constant; and `τ_mix` CANCELS out of `u`** |

**No `C_opt`, no `τ_res`, no `b_u`, and no new constant.** `n_quad=56` trips the β-PDF mean-preservation
guard at mid-J over-penetration (the **known** rung-23 §7 limitation, not a regression); `n_quad=160` clears
it and is what §4 runs.

---

## 8. Scope — what rung 24 certifies, and what stays open

- **Certified:** the reduce (`g` identical by construction); **`F(C)` U-shaped with its minimum AT `C_opt`,
  gradient-located INDEPENDENTLY of `g`** (the kill test), grid-converged 32→96, `(H/S)²`-collapsing;
  the ξ–τ correlation corroborated on the rich branch from an independent mechanism; **`⟨EI⟩(J)` monotone ⇒
  the emissions `C_opt` pin NOT recovered**; cycle bit-for-bit rung 6.
- **Concessions (loud):** `F`'s magnitude (~39% U) still rides on the rung-22 geometry constants
  (`k_p/k_y/k_z`) and the absolute `⟨τ⟩` still on rung-11's un-anchored `τ_mix` — **rung 24 localizes the
  RATE, not the SCALE**; the `S>H` global-argmin caveat (§6); `τ(ξ)` is a tight function only over the
  NO-relevant rich branch (§5).
- **STILL OPEN (and now sharper):** this is a **local rate on ONE frozen cross-plane geometry**, not a
  transported-PDF/CFD solve — the field's *pattern* is still the Gaussian-plume cartoon; only its *relaxation*
  is now locally resolved. And the honest revision of rung-23 §9's hope: **a locally-resolved mixing time does
  NOT let rung 17 claim a firing MAGNITUDE** — the magnitude rides on the global scale, which rung 24 does not
  touch. It buys a sharper *direction* and the derived `F(C)`, nothing more. **That §9 wording must be
  corrected, not inherited.**
