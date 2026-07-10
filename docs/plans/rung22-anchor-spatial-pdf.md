# Rung-22 external anchors — the resolved cross-plane / spatial PDF: the optimum as an OUTPUT (numbers-before-code)

Rung 18 proved (its load-bearing **negative** result) that a **0-D** mixture-fraction variance transport
**cannot derive** the Holdeman `C_opt` mixing optimum: with any mean-field `ω(J)` the residual `g(J)` is
monotone, so the optimum's **location** had to be imposed through a spatial coverage `ω(C)` — the jet
spacing `S` injected by hand. The rung 17/18 specs named the successor as the **deferred ceiling**: a
**spatial / transported-CFD PDF** from which the optimum would *emerge*. Rung 22 does the honest first
step — resolve the dilution **cross-plane** and watch `C_opt` come out as an **OUTPUT**.

**This anchor establishes — before any production code — that a resolved cross-plane INVERTS rung 18: the
Holdeman group `C=(S/H)√J`, the `(H/S)²` shift, and `C_opt≈2.5` all emerge from geometry + a penetration
law, with `C_opt` fed in NOWHERE.** Every number was re-derived first; the prototypes live in
`M:\claud_projects\temp\rung22\` (`proto.py`, `proto2.py`, `proto3.py`, `gstar2.py`, `pyport.py`,
`CHARACTERIZATION.md`).

Design point = rung 16/18's: flight 250 K/50 kPa/M0.85, PR 10, `Tt4→1500 K`, `τ=3 ms`, `φ_p=1.5`;
`far=0.02718` (`φ_ov=0.402`), `ξ̄=0.02646`, `ξ_p=0.09216`, `g_ceiling=(ξ_p−ξ̄)/(1−ξ̄)=0.0675`. The
Holdeman design point `J=16, H=0.10, S=0.0625 ⇒ C=(S/H)√J=2.5` sits AT the optimum.

> **Read `docs/rung18-spec.md` first** (the negative result rung 22 inverts, and the derived ceiling it
> reuses) and `docs/rung13-spec.md` (the ideal-bell β-PDF — the shared vehicle). NO stays a **trace,
> decoupled diagnostic**, so the cycle is still **bit-for-bit rung 6**.

---

## 1. The load-bearing POSITIVE result: the resolved cross-plane DERIVES the optimum (C_opt is an OUTPUT)

**The model (one dilution cell cross-plane; `C_opt` NOT among the inputs).** Penetration `y∈[0,H]` × span
`z∈[0,S]`, one dilution jet from the wall `y=0` at `z=S/2`, a Gaussian air plume:

- **penetration** `δ = k_p·√(S·H)·J^(1/4)` — jet-in-crossflow `δ∝d_j·√J` with a **fixed dilution mass
  ratio** ⇒ jet diameter `d_j∝√(S·H)·J^(−1/4)` (the jet shrinks as `J` rises at fixed air mass), so the
  **spacing `S`** enters the penetration, **not** just the momentum ratio. *This is what rung 18's
  mean-field `ω(J)` could not reach.*
- **spread** = a **fixed mixing length** `σ_y=k_y·H, σ_z=k_z·S` — **J-independent** (does not grow with
  penetration). This makes an **over-penetration penalty** survive: a jet past mid-height **reflects off
  the far wall** `y=H` (mirror images at both walls keep mass in `[0,H]`), piling air at the far wall.
- **mean-preserving**: the air scale is root-found so `⟨ξ⟩=ξ̄` **exactly** at every `J`.
- `g_spatial = Var[ξ]/(ξ̄(1−ξ̄))`. Constants: `k_p=0.316`, `k_y=k_z=0.28`. **No `C_opt` fed in.**

**The closed form (why `C_opt` is an output).** Uniformity is best where the jet fills half the height,
`δ≈H/2`:

```
k_p·√(S·H)·J^(1/4) = H/2   ⇒   √J = H/(4·k_p²·S)   ⇒   C = (S/H)·√J = 1/(4·k_p²)   (S,H-independent)
```

`k_p=0.316 ⇒ C_opt = 1/(4·0.316²) = 2.504` — **Holdeman's ≈2.5, as an OUTPUT**. `J_opt ∝ (H/S)²`.

**The machine-checked collapse (pure-Python port, `pyport.py`; ny=nz grid-converged):**

| S | H | J_opt | C_opt (OUTPUT) | g_min |
|---|---|-------|----------------|-------|
| 0.0625 | 0.100 | 15.6 | 2.47 | **0.0182** |
| 0.03125 | 0.100 | 61.5 | 2.45 | **0.0182** | (halve S ⇒ J_opt ×4) |
| 0.125 | 0.100 | 3.95 | 2.48 | **0.0182** | (double S ⇒ J_opt ÷4) |
| 0.0625 | 0.200 | 61.5 | 2.45 | **0.0182** | (double H ⇒ J_opt ×4) |
| 0.125 | 0.200 | 15.6 | 2.47 | **0.0182** | (S/H fixed ⇒ J_opt unchanged) |

**The `g_min` VALUE is identical (`0.0182`) across every geometry** — the true collapse signature; only
`J_opt` moves, **exactly as `(H/S)²`**. `C_opt` scatter is grid-snapping in the log-`J` sweep (mean 2.53);
the identity of `g_min` carries the result. **Grid-converged**: `ny=nz ∈ {32,48,64} ⇒ C_opt=2.59` (same).

**Robust to `k_p` (`proto2.py`-B).** `k_p` sets `C_opt=1/(4k_p²)` as an output and the collapse holds at
each: `k_p=0.20 ⇒ C_opt≈6.2` (all geometries agree), `0.316 ⇒ 2.5`, `0.45 ⇒ 1.26`. So what is **derived**
is the **group** (collapse + `(H/S)²` shift), not the number — the number rides on `k_p`.

**This inverts rung 18.** rung 18: 0-D `⇒` monotone `g(J)`, optimum must be imposed. rung 22: the resolved
cross-plane (the spacing `S` present in `δ`) `⇒` an interior optimum whose location is an **output**.

---

## 2. The rung-18 tie: `g_spatial < g_ceiling` always

The resolved partial-mix field is **less segregated than the two-δ extreme**, so it stays below rung-18's
**derived** ceiling `g_ceiling=(ξ_p−ξ̄)/(1−ξ̄)=0.0675` at every `J` (production path, `proto3.py`):

```
J=1   g_spatial=0.0423 < 0.0675      J=16  g_spatial=0.0182 < 0.0675      J=400 g_spatial=0.0486 < 0.0675
```

The one quantity rung 18 derived is exactly the bound the resolved field respects — asserted in the
branch (`g_spatial < g_ceiling + 1e-9`).

---

## 3. The emissions, honest: `C_opt` is a LOCAL min; the GLOBAL min is at max segregation

Through the **pure ideal bell** (rung 13, production path with coarse `n_bell=48/n_quad=64`):

**LOCAL min at `C_opt`** (both immediate flanks up — rung-18's reported behaviour):
```
ei_no_spatial:  J=9 → 1.1819   >   J=16 → 1.1752   <   J=25 → 1.1793
```

**GLOBAL min at an ENDPOINT** (a wide sweep beats the `C_opt` floor on the far flank):
```
ei_no_spatial:  J=1 → 1.0482   J=4 → 1.1824   J=16 → 1.1752   J=64 → 1.0719   J=256 → 0.9829  ← argmin
```

**Why (the mechanism, `gstar2.py`).** The ideal-bell `⟨EI⟩(g)` is **humped** — it rises to a peak at
`g* = 0.0211` (`⟨EI⟩=1.184`) then **descends** (rung-13's far flank; segregation at a lean mean moves mass
off the stoich peak). The **derived floor** `g(C_opt)=0.0182` sits **just below** `g*` (`⟨EI⟩=1.174`), so
`C_opt` is a **local** min with a **narrow** basin; by `g=0.051` (max penetration mismatch) `⟨EI⟩=0.977`,
**below** the floor value — hence the global min at the endpoint. rung 18's arbitrary floor
`g_ceiling·exp(−Da_opt)=0.0091` sits **lower** on the rising flank (`⟨EI⟩=0.876`, a wider basin) — **same
curve, different floor placement, neither wrong.** This is **why UNIFORMITY (`g`), not emissions, is the
clean headline.**

To pin the emissions global-min back at `C_opt` you need the rung-16 **dwell** `τ_core(C)` — but that
bakes `C_opt` in (`|ln(C/C_opt)|`), circular here. A **derived** dwell (the field's residence-time
distribution) is the deferred seam.

---

## 4. The reduce (exact by construction)

- **`spatial=None`** ⇒ the branch is never entered; the rung-22 fields stay `None`; the whole rung 1–21
  suite is untouched.
- **A spatial call** touches only `spatial`/`g_spatial`/`ei_no_spatial` (+ reused `C_holdeman`/`g_ceiling`/
  `g_seg`); the **primary** diagnostic `ei_no`/`x_no_mix` is **bit-identical** to a mixing-only call.
- **Cycle** far **bit-for-bit rung 6** (NO/N never enter `_equil_solve`).

---

## 5. The knobs (order-of-magnitude — disclosed, not fit)

| knob | value | role |
|------|-------|------|
| `S` | 0.0625 m | jet spacing (forms the Holdeman group with `H`, `J`) |
| `k_p` | 0.316 | penetration constant — **SETS `C_opt=1/(4k_p²)≈2.5` as an OUTPUT** (the only `C_opt` control; **not** a `C_opt`) |
| `k_y` | 0.28 | streamwise (penetration) mixing length / `H` (fixed, J-independent) |
| `k_z` | 0.28 | spanwise mixing length / `S` (fixed, J-independent) |
| `ny,nz` | 48 | cross-plane grid (converged; 32/48/64 agree) |
| `n_bell,n_quad` | 200 | ideal-bell / β-PDF grids (rung 13) |

**No `C_opt` field** — the signature of the inversion. `C_opt` is a derived property `SpatialPDF.C_opt()`;
`SpatialPDF(C_opt=2.5)` is a constructor error.

---

## 6. Vehicle + scope

- **Vehicle:** the resolved width `g(C)` feeds the **same rung-13 ideal bell** as rung 18 (`_pdf_mean_ei`).
  Only the **source of `g`** changed — imposed ODE (rung 18) → resolved cross-plane (rung 22).
- **Certified:** the group **collapse** (`g_min` geometry-independent), the **`(H/S)²` shift**, `C_opt` =
  the **closed form** (an output), **no `C_opt` knob**, `k_p`-robustness, `g_spatial<g_ceiling`, the
  **local** emissions min + the **global** min at max segregation.
- **Concessions (loud):** the **value** `C_opt≈2.5` rides on `k_p`; rung 22 derives the **width**, not the
  **dwell** (rung-16 kink imported); the emissions optimum at `C_opt` is **local**; the field is a
  Gaussian-plume **cartoon** feeding the β-PDF closure, not a CFD/PDF-transport solve.

---

## 7. What rung 22 leaves open (the still-deferred ceiling)

- **A DERIVED dwell spectrum `τ_core(C)`** — the residence-time distribution of the resolved field, which
  (through the rung-16 per-pocket quench) would pin the *emissions* global-min at `C_opt` without the
  circular rung-16 kink. Rung 22 derives the width; the dwell stays imported.
- **A real PDF-transport / CFD cross-plane** — predict the full resolved-PDF **shape** and spatial pattern
  from a scalar-flux equation (also what would let rung 17 claim a firing **magnitude**). Rung 22 is a
  cartoon feeding the β-PDF closure; the spatial pattern stays the ceiling.
