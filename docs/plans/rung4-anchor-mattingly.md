# Rung-4 external anchors — Mattingly variable-`cp`, `f`-parameterized gas tables

Verified worked examples + the McKinney gas model that anchor the **reacting-products**
machinery of rung 4: an *explicit* lean-complete-combustion products composition that
varies with the fuel/air ratio `f`, its `cp_t(T,f)`, `R_t(f)`, the `h`/`pr` property
functions on it, and the **implicit `f = g(f)` burner solve**. Source, throughout, is
the project's chosen book:

- **Mattingly**, *Elements of Propulsion: Gas Turbines and Rockets* (AIAA), local PDF
  `M:\claud_projects\booksandresources\Elements of Propulsion ...pdf`.
  §2.6.6–2.6.8 (gas tables, Table 2.2, Eqs. 2.53–2.64), **Examples 2.7 / 2.8**
  (compression / nozzle, air `f=0`), **§6.10 Component Performance with Variable Cp**
  (freestream→turbine in `h`/`Pr` form, parameterized by `f`), **Examples 6.2 / 6.3**
  (compressor `f=0` / turbine **products `f=0.0338`**).

All numbers below were re-derived and **certified against two independent property
models before any production code** (rung-3 discipline): (1) the rung-3 **NASA-7**
species machinery extended with `(CH2)n` combustion stoichiometry — the Option-A
*production* model; (2) Mattingly's own **McKinney** `f`-blend coefficients (Table 2.2)
— the *test-only* cross-check. They agree with Mattingly and with each other.

---

## Why the anchor is per-process, not a full turbojet (carried from rung 3)
A Ch. 7/8 scan (pypdf) found **no worked full variable-`cp` turbojet cycle** — Mattingly's
cycle-analysis chapters are calorically perfect (Example 7.1 = constant dual-`cp`, the
rung-2 anchor). Variable-`cp` lives in Ch. 2 fundamentals + **§6.10 component**
performance only. So rung 4's TPG-reacting path is anchored at the **component /
process** level (Ex 2.7/2.8/6.2/6.3); the **turbojet topology** (shaft balance, nozzle
`V9`, thrust) stays anchored by the rung-1/2 **reduce-to-ideal** gate (CPG closed form),
untouched by rung 4 (the reacting gas is a *new* `Gas.reacting(f)` path, added alongside).

⚠ Same topology caveat as rung 3: Ex 6.2/6.3 are single-component processes, not a
turbojet. They pin the *property + process* machinery (stations 3/5 substate math, the
`Pr`-ratio, η-on-enthalpy, the implicit `f`), which is exactly what rung 4 newly adds.

---

## The reacting model in Mattingly (§2.6.7, Table 2.2, Eqs. 2.63–2.64)
Fuel is a hydrocarbon of composition **`(CH2)n`**; `f = ṁ_fuel/ṁ_air`; `f_max = 0.0676`.
Properties of the burned gas are a mass-blend of an "air" stream and a "combustion
products" stream, weighted by `f`:
```
R(f)    = 1.9857117 / (28.97 − f·0.946186)              Btu/(lbm·°R)     [f=0 → R_air]
cp(T,f) = (cp_air(T) + f·cp_prod(T)) / (1+f)                              (Eq. 2.64b)
h(T,f)  = (h_air(T)  + f·h_prod(T))  / (1+f)                              (Eq. 2.64c)
φ(T,f)  = (φ_air(T)  + f·φ_prod(T))  / (1+f)                              (Eq. 2.64d)
Pr(T,f) = 2·exp( (φ(T,f) − φ(600°R, 0)) / R(f) )         [ref: Pr=2 at 600°R, f=0]
```
`cp_air`, `cp_prod` are 8-term polynomials `cp = Σ_{i=0}^{7} A_i T^i`, **`T` in °R,
`cp` in Btu/(lbm·°R)**, valid 300–4000 °R (McKinney / AFAPL, "widely used in industry",
Ref. 16 in Mattingly; the AFPROP program).

### McKinney coefficients — Table 2.2, OCR-restored and CERTIFIED
The PDF text layer mangled the exponent minus-signs; the values below are restored and
**self-certified** by reproducing Ex 2.7 (air) and Ex 6.3 (products) — see the table
below. **These are the *test-only* cross-check constants (English units); the production
model is Option A (NASA-7 stoichiometry, SI).**
```
                A0             A1            A2             A3
air     :  2.5020051e-1  -5.1536879e-5   6.5519486e-8  -6.7178376e-12
products:  7.3816638e-2   1.2258630e-3  -1.3771901e-6   9.9686793e-10
                A4             A5            A6             A7           href(Btu/lbm)  φref
air     : -1.5128259e-14  7.6215767e-18  -1.4526770e-21  1.0115540e-25   -1.7558886   0.0454323
products:-4.2051104e-13   1.0212913e-16  -1.3335668e-20  7.2678710e-25   30.58153     0.6483398
```
Note the ~+32 Btu/lbm `href` gap (products vs air): this is the **cross-datum
calibration** — Mattingly keeps sensible-style tables + a fixed `hPR`, absorbing the
reference offset into `href`, *not* into formation enthalpies. So the whole model is
**Fork A** (fixed `hPR`), realized with Mattingly's own calibration. (Formation-enthalpy
bookkeeping = Fork B = rung 5.)

---

## The implicit burner is MATTINGLY'S OWN (§6.10.4, Eq. 6.35–6.36)
```
η_b = [ (1+f)·h_t4 − h_t3 ] / (f·hPR)                    (Eq. 6.35)
f   = ( h_t4 − h_t3 ) / ( η_b·hPR − h_t4 )               (Eq. 6.36)
```
Verbatim Note after Eq. 6.36: *"The value of h_t4 is a function of the fuel/air ratio f,
and thus the solution of Eq. (6.36) is iterative."* → the rung-4 `f = g(f)` fixed-point
solve (seeded from the rung-3 frozen `h_t`, convergence residual a standing assert) is
Mattingly's own, not a project invention. This is rung 4's load-bearing new mechanic.

---

## Worked-example anchors (all datum-independent quantities are digit-checkable)
| Ex | process | `f` | inputs | Mattingly result |
|----|---------|-----|--------|------------------|
| 2.7 | isentropic compression | 0 | 293.15 K, `Pr1=1.2768`, `×15 → Pr2=19.152` | **`T2 = 627.57 K`** |
| 2.8 | isentropic nozzle | 0 | 3000 °R, `V1=0`, `Δh=179.74 Btu/lbm` | `T2=2377.7 °R`, `P2/P1=0.3757` |
| 6.2 | compressor (polytropic `ec=0.9`) | 0 | 540 °R, `πc=15`; `ht2=129.02`, `Prt2=1.384` | `Tt3=1251.92 °R`, `ηc=0.8586` |
| 6.3 | turbine (polytropic `et=0.9`) | **0.0338** | 20 atm, 3000 °R; `ht4=828.75`, `Prt4=1299.6`; `Δh=100 Btu/lbm` | `Tt5=2677.52 °R`, `Prt5=777.39`, `πt=0.5650`, `Tt5i=2643.64 °R`, **`ηt=0.9057`** |

Ex 6.3 is **the `f>0` products anchor** that closes rung-3's "products float" gap.

---

## De-risk verification (BEFORE coding) — both models vs Mattingly

### (1) Option-A production model — NASA-7 species + `(CH2)n` stoichiometry (SI)
`CH2 + 1.5 O2 → CO2 + H2O`; dry-air oxidizer (N2/O2/Ar from rung-3 `_AIR`); lean,
complete. Product composition is deterministic and smooth in `f`.
- **`f_stoich = 0.0677`** (Mattingly `f_max = 0.0676`, 0.15%) — stoichiometry certified.
- At `f=0.0338` (φ≈0.5): mole fractions **N2 0.7548, O2 0.1014, CO2 0.0674, H2O 0.0674,
  Ar 0.0090**; `R_t = 287.4 J/(kg·K)`.
- vs **Ex 6.3** (datum-independent — survives the `h(0)=0` datum, being Δh/`pr`-ratios):

  | quantity | NASA-7 stoich | Mattingly | gap |
  |---|---|---|---|
  | `Tt5` from Δh=100 Btu/lbm | 2677.54 °R | 2677.52 °R | 0.001% |
  | `πt` (polytropic, e=0.9) | 0.5651 | 0.5650 | 0.02% |
  | `Tt5i` | 2643.61 °R | 2643.64 °R | 0.001% |
  | **`ηt`** (implied isentropic) | **0.9056** | **0.9057** | 0.01% |

  Products agree to ~0.02% — *tighter* than rung-3's ~0.1% air gap. The anchor
  **exercises the actual rung-4 stoichiometry + property code**.

### (2) Test-only McKinney cross-check (Table 2.2 coeffs, English)
| check | McKinney (restored) | Mattingly | gap |
|---|---|---|---|
| `Pr(600°R,0)` reference | 2.0000 | 2 (defn) | exact |
| Ex 2.7 `T2` (air, ×15) | 627.58 K | 627.57 K | exact |
| Ex 2.8 `h(3000°R,0)` (air) | 790.46 Btu/lbm | 790.46 | **exact** |
| Ex 6.3 `h(3000°R,0.0338)` (products) | 828.75 Btu/lbm | 828.75 | **exact** |
| Ex 6.3 `Pr(3000°R,0.0338)` | 1300.1 | 1299.6 | 0.04% |
| Ex 2.7 `Pr(527.67°R,0)` | 1.2764 | 1.2768 | 0.03% |

Enthalpies reproduce to the digit (air *and* products), validating both the coefficients
and the `f`-blend rule. Only ever **Pr *ratios* at fixed `f`** are used in the cycle, so
the absolute-`Pr` reference convention (nailed above) is a consistency detail, not a
physics input.

### (3) Burner de-risk — the ONE datum-dependent step (cross-datum sensitivity)
Ex 6.3 and all the process anchors are **datum-independent** (Δh / `pr`-ratios), so they
are structurally blind to the burner — the only step that subtracts a *hot*-section
enthalpy from a *cold* one, `f = (h_t(Tt4) − h_c(Tt3))/(η_b·hPR − h_t(Tt4))`. Mattingly's
tables carry a **+32 Btu/lbm href offset** (products +30.58 vs air −1.756) that his
Eq. 6.36 consumes; the Option-A production model uses `h(0)=0` for *both* sections. To
prove that difference doesn't move `f`, the fixed point was solved in BOTH models at
matched inputs (`Tt3=800 K, Tt4=1600 K, η_b=0.99, hPR=42.8 MJ/kg = 18400 Btu/lbm` — the
value Mattingly uses):

| burner model | `f` |
|---|---|
| NASA-7, `h(0)=0` (production) | 0.024538 |
| McKinney **with** href offsets | 0.024579 |
| McKinney, href zeroed | 0.024533 |

- Production `h(0)=0` vs Mattingly's full-datum model: **0.17%** on `f`.
- The href offset alone: **0.19%** — the cross-datum energy is negligible at lean `f`, so
  **`hPR=42.8 MJ/kg` needs no recalibration and no carried per-section offset**. Fork A
  with `h(0)=0` is validated for the burner (not just asserted).
- Two different cp correlations, both zero-datum: **0.02%** — burner is insensitive to the
  species model too.
- **Convergence measured:** fixed point contracts linearly, factor ≈ 0.09 — ~5–6 steps to
  1e-6, ~11 to 1e-12 (from a cold `f₀=0.02` seed; the rung-3 frozen estimate cuts it).
  *Not* the "2–4 steps" first guessed.

No standalone Mattingly burner example exists (Ex 6.4 is not one; Ch. 7 discusses `f`-
trends only qualitatively) — so this two-model comparison IS the burner anchor. Directional
corroboration (Mattingly pg. 398): `f` rises with variable `cp` because "the increase in
`cp` … means more energy is needed to increase the temperature of the products."

---

## Tolerances (set from the measured gaps, not guessed)
- **Option-A production anchor (Ex 6.3, datum-independent):** ~0.05% on `ηt`/`Tt5`/`πt`
  (measured 0.001–0.02%; allow table-rounding headroom). This is the primary rung-4
  reacting-gas anchor.
- **McKinney test cross-check:** enthalpies to ~1e-4 relative (measured exact to the
  digit); `Pr` to ~0.1% (measured 0.03–0.04%).
- **Reduce-to-ideal (unchanged, tight):** `Gas()` / dual-CPG reproduce rung-1/2/2b to
  the digit (1e-9 / 5e-4) — the reacting gas is a *separate* path, so existing suites
  stay green untouched.

## What stays UN-anchored (state it plainly)
The fuel is idealized `(CH2)n` (Jet-A ≈ C12H23 ≈ (CH2)n, close). Lean + **complete**
combustion only (no CO/H2, no dissociation) — valid for `f < f_stoich`, the main-burner
regime. Rich combustion and T-coupled equilibrium/dissociation are **rung 5** (Fork B
formation enthalpies + equilibrium composition, on this same explicit-composition
machinery). `hPR` is held fixed across the lean composition shift (Mattingly does the
same) — the honest Fork-A cost.
