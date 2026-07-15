# Rung-23 external anchors — the DERIVED dwell spectrum: the ξ–τ correlation (numbers-before-code)

Rung 22 resolved the y-z dilution cross-plane and **derived** the β-PDF **width** `g(C)` — the Holdeman
`C_opt` emerging as an OUTPUT (the inversion of rung 18). But rung 22 fed that width through the
per-pocket quench with the **imported rung-16 KINKED scalar dwell** `τ_core(C)=τ_res·(1+b_u·|ln(C/C_opt)|)`
— which **bakes `C_opt` in**. Rung 22's own honest concession (`docs/rung22-spec.md` § deferred, and
`docs/plans/rung22-anchor-spatial-pdf.md` §7): the *"width AND dwell"* seam is only **partially** closed —
the **width** derived, the **dwell** still the imposed kink. Rung 23 closes the other half.

**This anchor establishes — before production code — that the SAME resolved cross-plane, watched as it
MIXES OUT over the rung-11 mixing time `τ_mix`, yields each pocket's dwell `τ(ξ)` from first principles
(no `C_opt`, no `τ_res`, no `b_u`), and that the ONE genuinely new physical quantity is the ξ–τ
CORRELATION** (rich pockets arrive late ⇒ dwell long — the physics rung-16's scalar `τ_core` structurally
cannot express). Every number was re-derived first; the prototypes live in `M:\claud_projects\temp\rung23\`
(`proto.py` geometry/dwell, `proto2.py` matched-mean + J-sweep, `proto3.py` decomposition + divergence,
`proto4.py` the sign-vs-τ_mix blocker check).

Design point = rung 16/22's: flight 250 K/50 kPa/M0.85, PR 10, `Tt4→1500 K`, `τ=3 ms`, `φ_p=1.5`;
`far=0.02718`, `ξ̄=0.02646`, `ξ_p=0.09216`, `g_ceiling=0.0675`. Holdeman `J=16, H=0.10, S=0.0625 ⇒ C=2.5`
sits AT the optimum. Dwell scale `τ_mix=H/(C_e·√J·U_c)` (rung 11; `C_e=0.20, U_c=75 m/s`).

> **Read `docs/rung22-spec.md` first** (the width it inverts and the cross-plane it develops) and
> `docs/rung16-spec.md` (the per-pocket quench — the vehicle). NO stays a **trace, decoupled diagnostic**,
> so the cycle is still **bit-for-bit rung 6**.

---

## 1. THE NON-CIRCULARITY CONTRACT (what makes this a real inversion, not a repaint)

The derived-dwell path contains **no `C_opt`, no `τ_res`, no `b_u`** — those are rung-16's imposed dwell.
Rung 23 uses ONLY: the rung-22 geometry constants `k_p/k_y/k_z` and rung-11's mixing time `τ_mix=mixing.tau_q`.
If any of the three leaked in, `C_opt` would be re-baked and the seam un-closed. (This is the exact analog of
rung 22's signature — **no `C_opt` knob**; `SpatialDwellPDF.C_opt()=1/(4k_p²)` is a derived property.)

**Time development** (standard turbulent-mixing laws — EXPONENTS, not fitted constants; both terminate at
`τ_mix` so the `t=τ_mix` field IS rung-22's exactly — the consistency anchor):
- `σ(t) = σ_final·√(t/τ_mix)` — turbulent diffusion (`σ²∝D_t·t`)
- `δ(t) = δ_final·(t/τ_mix)^(1/3)` — jet-in-crossflow trajectory (`y∝x^(1/3)`)

**Per-cell dwell** = the arrival-time deficit `τ_cell = ∫₀^{τ_mix}(1 − β(t)/β_final) dt` — how long the
pocket lingers un-diluted before the air reaches it. Late-arriving (rich) cells ⇒ large `τ_cell`.

---

## 2. THE REDUCE / CONSISTENCY ANCHOR — the terminal field == rung 22

`_spatial_dwell_field` at `t=τ_mix` must reproduce `_spatial_segregation` (rung 22). Machine-checked
(`proto.py`; production pin: `g_spatial_dwell == SpatialPDF.segregation` to <1%):

| J | `g_spatial` (rung 23 terminal) | rung-22 anchor |
|---|-------------------------------|----------------|
| 1   | 0.0427 | 0.0423 |
| 16  | 0.0183 | 0.0182 |
| 400 | 0.0490 | 0.0486 |

Mean-preserving `⟨ξ⟩=ξ̄` asserted every call (the rung-22 contract).

---

## 3. THE LOAD-BEARING POSITIVE — the ξ–τ correlation (matched-mean isolation)

The certified result is the **matched-mean-per-J** experiment: term 2 with the correlated `τ(ξ)` spectrum
vs term 2 with a scalar dwell `= ⟨τ⟩_PDF`. At a fixed operating point this holds `g` **and** `⟨τ⟩` fixed
and varies ONLY the correlation (exactly as rung 16 isolated cooling from rung 15's linear dwell).
`corr/mean = term2_correlated / term2_meanfield`, the shipped `zoned_nox(…, spatial_dwell=…)` path at the
grid `main.py` prints (32×32 cross-plane, `nt=24`, `n_quad=56`):

```
   J     C      g   τ_mix(ms)  ⟨τ⟩(ms)  corr/mean   max_a
   4   1.25  0.0312    3.333    2.012     1.058     0.553   ← under-penetration: correlation ADDS ~+5.8%
   9   1.88  0.0221    2.222    1.302     1.039     0.540
  16   2.50  0.0181    1.667    0.919     1.025     0.532   ← C_opt (+2.5%)
  64   5.00  0.0392    0.833    0.375     1.010     0.519   ← over-penetration
 400  12.50  0.0486    0.333    0.213     1.008     0.518   ← deep over-penetration (+0.8% floor)
```

**`corr/mean > 1` everywhere** — the correlation ADDS NO (rich pockets dwell long ⇒ re-make more), and it
is **CONCENTRATED under-penetration** (where the dwell is longest), fading **monotonically** to a ~+0.8%
floor on the over-penetration flank. Rung-16's scalar `τ_core` has `corr/mean ≡ 1` by construction — this
is the new physics.

> **Reconciliation (the honest surprise — "stop and explain").** The exploratory prototypes
> (`proto2.py`/`proto4.py`) reported a ~2× larger excess (`corr/mean ≈ 1.12` at J=4, "~+12%") and a smaller
> `⟨τ⟩ ≈ 1.17 ms`. That is **not** a grid/resolution effect (grid convergence moves `corr/mean` only ~1.3%
> over 32/24→64/48; the gap is ~6%): the sketch and the production `_spatial_dwell_field` compute a
> **genuinely different `τ(ξ)`**. The single cause (`confirm_reason.py`, bisected at identical resolution):
> the prototype **re-solved the air-source strength `s*` at every time-slice** against that slice's
> (narrower) plume shape; production **fixes `s*` once** via the terminal mean-preservation contract and
> lets the plume spread — the physically-consistent *"fixed air source, evolving spread"* (the injected
> dilution mass is fixed; only its spatial spread evolves). The prototype's per-slice renormalization made
> the early-time air fraction rise faster, **shrinking the arrival-time deficit** ⇒ a smaller `⟨τ⟩` and an
> **inflated** correlation contrast. Production is the refinement, not a bug (patching the prototype to fix
> `s*` reproduces production's `⟨τ⟩`); the certified **sign + under-penetration concentration** are
> unchanged, and the shipped, reproducible magnitude is what `main.py` prints.

---

## 4. THE BLOCKER — is the correlation SIGN one-signed across τ_mix?

The sign is *a priori* ambiguous (rung-16 EI is CONCAVE in τ ⇒ Jensen pushes `corr/mean` **below** 1; the
cross-term — rich pockets have larger `∂EI/∂τ` — can BEAT that, above 1; the balance can flip when the
dwell magnitude changes). Swept `τ_mix ×0.2, ×1, ×5` at J=4 and J=16 (shipped path, 32/24 grid — the range
`tests/test_rung23.py` gates):

```
   J   scale  ⟨τ⟩(ms)  corr/mean   max_a
   4    0.2    0.402     1.014      0.521
   4    1.0    2.012     1.058      0.553
   4    5.0   10.060     1.165      0.714
  16    0.2    0.184     1.005      0.517
  16    1.0    0.919     1.025      0.532
  16    5.0    4.593     1.095      0.606
```

**One-signed (`corr/mean > 1`) across the whole ×0.2–×5 range**, and it STRENGTHENS with `τ_mix`. The Jensen
flip never fires because `max_a` stays **< 1** (formation-limited) even at ×5 — we never enter the
destruction regime where the sign could reverse. **Certified: one-signed throughout the formation-limited
regime (max_a<1), which spans the plausible τ_mix range.** The *magnitude* rides on `τ_mix` (as
rung-22's `C_opt≈2.5` rides on `k_p`); the *sign + under-penetration concentration* are the derived shape.

---

## 5. THE HONEST NEGATIVE — the emissions C_opt pin is NOT recovered

The derived `τ(ξ)` has its absolute scale in rung-11's `τ_mix ∝ 1/√J` (monotone), so it **FALLS**
off-optimum. It does NOT grow like rung-16's kink, so the over-penetration flank is not lifted. term2 (and
the full `⟨EI⟩`) is monotone-decreasing in J (`proto2.py`): argmin at max over-penetration, NOT `C_opt`.

**But this is not rung-23's discovery:** rung 16 ALREADY declined the global-min LOCATION (its GATE 3 —
it flips across quadrature ~5% / the φ>2 tail / the `C_e` regime). So rung 23 confirms from first
principles what rung 16 conceded. The honest framing (rung-18-flavored): **whether the emissions `C_opt`
pin survives depends on whether the dwell GROWS off-optimum (rung-16, imposed) or FALLS with the mixing
time (rung-23, derived) — and neither is pinned from data.** Only the **width** `g` pins `C_opt`
(rung-22's uniformity result, preserved).

**The divergence, quantified** (shipped path, 32/24 grid — `main.py`'s panel): rung-16 `τ_core(C)` vs
rung-23 `⟨τ⟩(J)` — ~3×–70× apart AND opposite-trending off-optimum, **both un-anchored**:

```
   J     C    rung16 τ_core(ms)   rung23 ⟨τ⟩(ms)   ratio 16/23
   4   1.25         7.70               2.01             3.8
  16   2.50         2.50               0.92             2.7
  64   5.00         7.70               0.38            20.5
 400  12.50        14.57               0.21            68.4
```

rung-16 GROWS off-optimum (imposed `|ln(C/C_opt)|`); rung-23 FALLS `∝1/√J` (rung-11 mixing time). An
HONEST split — a derived correlation SHAPE (certified) + an un-pinned off-optimum TREND — **not** "rung 16
is an artifact."

---

## 6. The reduce (exact by construction)

- **`spatial_dwell=None`** ⇒ the branch is never entered; the rung-23 fields stay `None`; the whole rung
  1–22 suite is untouched.
- A `spatial_dwell` call touches only `spatial_dwell`/`g_spatial_dwell`/`tau_mean_dwell`/
  `ei_no_spatial_dwell`/`ei_no_spatial_dwell_meanfield`/`ei_no_spatial_dwell_excess`/`corr_ratio`
  (+ reused `C_holdeman`/`g_ceiling`/`g_seg`); the **primary** diagnostic `ei_no`/`x_no_mix` is bit-identical.
- **Cycle** bit-for-bit rung 6 (NO/N never enter `_equil_solve`).
- `tau_of_xi=None` in `_pocket_quench_mean_ei` ⇒ the SCALAR path is byte-identical rung 16.

---

## 7. The knobs (order-of-magnitude — disclosed, not fit) — and what is DERIVED

| knob | value | role |
|------|-------|------|
| `S` | 0.0625 m | jet spacing (Holdeman group with `H`, `J`) |
| `k_p` | 0.316 | penetration constant — SETS `C_opt=1/(4k_p²)≈2.5` as an OUTPUT (the only `C_opt` control) |
| `k_y`, `k_z` | 0.28 | mixing lengths / `H`, `S` (fixed, J-independent) |
| `ny, nz` | 40 (default) | cross-plane grid — `main.py`'s panel + the tests coarsen to **32** for speed |
| `nt` | 32 (default) | time steps of the dwell integral — panel/tests use **24** |
| `n_bell, n_quad` | 120/160 (default) | per-pocket ξ-grid / β-PDF quadrature (rung 16) — panel/tests use **40/56** |
| **`τ_mix`** | **rung-11 `mixing.tau_q`** | **the dwell's absolute scale — NO new knob (reused, not added)** |

**No `C_opt`, no `τ_res`, no `b_u`** — the non-circularity signature. `C_opt()` is a derived property.

> **The cited numbers (§3–§5) are the panel/test speed grid (32/24, `n_quad=56`)** — the reproducible
> `python main.py` artifact — not the finer config defaults. One consequence of the coarser `n_quad=56`:
> the β-PDF mean-preservation guard in `_beta_pdf_nodes_weights` can trip at a few mid-`J` over-penetration
> points (e.g. `J≈36`, where the shape parameter lands badly); the config-default `n_quad=160` clears it
> (the prototype ran to `J=400`). A **known coarse-quadrature limitation of the demo path**, at `J` values
> the panel/tests avoid — not a rung-23 regression.

---

## 8. Vehicle + scope

- **Vehicle:** the derived `τ(ξ)` feeds the **same rung-16 per-pocket quench** (`_pocket_quench_mean_ei`,
  now taking an optional `tau_of_xi` callable). Only the **dwell source** changed — imposed scalar kink
  (rung 16 / rung 22) → derived per-pocket spectrum (rung 23).
- **Certified:** the reduce (terminal field == rung 22), the correlation **sign + under-penetration
  concentration** (matched-mean, one-signed across `τ_mix ×0.2–×5`, formation-limited), `g_spatial <
  g_ceiling`, the divergence-from-rung-16 (opposite off-optimum trend), cycle bit-for-bit rung 6.
- **Concessions (loud):** the correlation's absolute MAGNITUDE/TREND rides on rung-11's un-anchored `τ_mix`;
  the emissions `C_opt` pin is NOT recovered (rung 16 already declined the global-min location); ONE global
  `τ_mix` (a Gaussian-plume cartoon), NOT a locally-resolved mixing time.

---

## 9. What rung 23 leaves open (the still-deferred ceiling)

- **A LOCALLY-resolved mixing time** — rung 23 develops the field under ONE global `τ_mix`; a real scalar-
  dissipation field would give each cell its own mixing rate (and could restore an off-optimum dwell
  GROWTH that pins the emissions optimum non-circularly). This — plus the full resolved-PDF **shape** — is
  what would let rung 17 claim a firing **MAGNITUDE**, not just a direction. Rung 23 is a cartoon feeding
  the rung-16 β-PDF closure; the locally-resolved mixing time stays the ceiling.
