# Rung 69 anchor — THE REFERENCE SPLIT

**Status: § 0 is MEASURED (a direction check, a feasibility pre-check and a discriminator — all
run before the predictions were written). §§ 1–4 are DERIVATION ONLY.** Every number in §§ 1–4
is a prediction, not a measurement. Predictions are scored HIT / MISS after the sweep, and a
MISS is published in place, as in rungs 51/58/63/64/65/66/67/68.

Rung 68's standing seam, stated there in full:

> **THE REFERENCE SPLIT — does the coordinate a loop is referenced in decide whether it adds a
> ZERO or a RANK?** § 1 and § 4 are the setup and this rung deliberately does not run it: the
> SAME stator, referenced to incidence (rung 60's `IncidenceLimiter`) instead of to `φ`, solves
> a constraint the other two do not share, so § 2's derivation predicts the cyclic product
> leaves −1 and the block goes to rank 2 — one zero, not two. It is also the physically correct
> direction for the lever (`dM_i/dv = +0.344`), so it is the pairing where redundancy and
> hardware sense finally agree. **The strongest open seam in this family.**

Nothing about the hardware changes. Rung 68's lever, rung 68's clocks, rung 68's plant, rung
68's two other loops. **The ONLY thing that moves is the coordinate the third loop watches.**

---

## 0. THE PRE-CHECK — measured FIRST, because it decides whether this is a rung at all

Rung 68 § 0's precedent. Settings verbatim from `tests/test_rung68.py` (`φ_lim = 0.80`,
`b_max = 0.10`, `τ_v = τ_s = 0.05`, `τ_att/τ_rel = 0.05/0.15`, `ds = 0.005`, `r = 0.5`,
`s_settle = 1.2`, `Tt4` 1000 → 1400 K, `v_max = 0.20`). `T_c = 1/0.55 = 1.818182`, and the
matched incidence floor is `m_lim = T_c − 1/φ_lim = 0.568182` — the SAME wall as rung 68's `φ`
floor, at the design stator setting.

### 0.1 THE DIRECTION — and this time it IS the physical one

Read at `s = 0.29` on the shipped rung-66 cascade march (`φ_lp = 0.797776`, `b = 0.0765`,
`g = 2.52e−3`), against a trial LP stator setting:

| `v` | −0.20 | −0.10 | −0.05 | −0.02 | 0.00 | +0.02 | +0.05 | +0.10 | +0.20 | +0.30 | +0.40 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `φ_lp` | 0.8949 | 0.8430 | 0.8196 | 0.8063 | 0.7978 | 0.7894 | 0.7773 | 0.7581 | 0.7230 | 0.6917 | 0.6637 |
| `M_i` | 0.5007 | 0.5320 | 0.5481 | 0.5580 | 0.5647 | 0.5714 | 0.5817 | 0.5990 | 0.6350 | 0.6725 | 0.7115 |

    dφ_lp/dv = −0.42300        dM_i/dv = +0.33537        [central, dv = 1e−4]

**`M_i` is INCREASING in `v`, so the incidence-referenced loop CLOSES the stators** — which is
what a real VSV schedule does at low corrected speed, and the exact opposite of rung 68's
`φ`-referenced loop. The band is one-sided the other way: `v ∈ [0, +v_max]`, dormant at `0`,
saturated at `+v_max`. **Both clamp tests and the bracket orientation therefore flip BACK to
`_solve_b`'s**, which is rung 62's `_powers` trap in its fifth reload and fails silently.

The scalar this whole rung turns on, defined here and measured throughout:

    ψ := M_i = T_c − 1/φ + v        ⇒   ψ_v = φ_v/φ² + 1
    k := (φ_v/φ²) / ψ_v             = −1.98182   at this point

`k < 0` **iff the lever's two channels FIGHT** — iff it raises one wall while lowering the
other. That is the entire content of the split, compressed to one number.

### 0.2 FEASIBILITY — is `v_max = 0.20` enough, and on which cells?

The incidence-rooted command `V(g, q)` (a DIAGNOSTIC root only — no new state, no new
integrator) evaluated at every point of three inherited marches, `s ≤ r = 0.5`:

| march | n | `min φ_lp` | max `V` needed | n(`V > 0.20`) | n(dormant) |
|---|---|---|---|---|---|
| bare | 100 | 0.735442 | **0.3314** | **84** | 0 |
| fuel only | 100 | 0.773116 | 0.1175 | 0 | 0 |
| fuel + valve | 100 | 0.793085 | 0.0295 | 0 | 27 |

**PASS, with one disclosed consequence.** In company the loop rides comfortably inside the
inherited `v_max = 0.20`; **ALONE it saturates over 84 % of the ramp.** So the ledger's `S` and
`FS` cells are authority-limited by a ceiling this project chose in rungs 57/58 — that is
reported as a measurement, not tuned away by raising `v_max`, which would make it a new
constant.

### 0.3 THE DISCRIMINATOR — the six cross-gains, both references, ONE base point

The discriminator that separates the hypotheses before the build (rung 64's move). Central
differences on the **shipped, mutually ignorant** closures — `_surge_fuel` (fuel), `_solve_b`
(valve), and the stator's own root — at `dg = 1e−7`, `dq = 1e−5`, `dv = 1e−4`, on the riding
points of the shipped rung-66 march.

**THE EVALUATION MANIFOLD IS THE SHARED CONSTRAINT'S, and that is forced rather than chosen.**
`R_q·C_g = 1` is an implicit-function identity that holds only when BOTH `φ` loops sit at their
own rest points — i.e. when the base point is ON `φ = φ_lim`. Rung 68 could put all three there
at once. **Here there is no such point:** `φ = φ_lim` and `M_i = m_lim` together force `v = 0`,
the stator's own dormant stop. So the base is `v = v_φ(g, q)`, the setting putting `φ` on the
shared floor — rung 68's `manifold=True` instrument, unchanged, which is what keeps the two
rungs' numbers differenceable (rung 63's lesson). Measured at the STATOR's own root instead,
`pair_RC` degrades to 0.94–0.98; that reading is reported beside it and never gated on.

**Stator referenced to INCIDENCE** (`τ = (0.05, 0.05, 0.05)`, so `Σ1/τ = 60`):

| `s` | `v_φ` | `pair_RC` | `pair_RV` | `pair_CV` | cyclic `x` | `c1` | `c0` | roots |
|---|---|---|---|---|---|---|---|---|
| 0.005 | −0.0039 | **1.00000000** | −1.66667 | −1.66195 | **+1.66667** | 2.131e+3 | 4.9e−6 | `0, −30 ± 35.09i` |
| 0.105 | −0.0162 | **1.00000000** | −1.77113 | −1.75090 | **+1.77113** | 2.209e+3 | 7.8e−6 | `0, −30 ± 36.18i` |
| 0.205 | −0.0106 | **1.00000000** | −1.87073 | −1.85658 | **+1.87073** | 2.291e+3 | 8.9e−7 | `0, −30 ± 37.30i` |
| 0.305 | −0.0043 | **1.00000000** | −2.01098 | −2.00467 | **+2.01098** | 2.406e+3 | 3.2e−7 | `0, −30 ± 38.81i` |

**The SAME lever at the SAME base points, referenced to `φ` (rung 68):**

| `s` | `pair_RC` | `pair_RV` | `pair_CV` | cyclic `x` | `c1` | `c0` | roots |
|---|---|---|---|---|---|---|---|
| 0.005 … 0.305 | 1.00000000 | 1.00000000 | 1.00000000 | −1.00000000 | ≤ 2.5e−7 | ≤ 3.3e−13 | `0, 0, −60` |

Four things are settled by that pair of tables, and every one of them is a `§ 2` claim:

1. **The pairwise products SPLIT.** `pair_RC` — the two loops that still share `φ` — stays at
   `1.00000000`. `pair_RV` and `pair_CV` go to `k`, **negative and ~1.7–2.0 in magnitude**.
   *Which* pairs keep rung 66's identity is a direct read of *which loops share a constraint*.
2. **The cyclic product is `−k`,** so it leaves `−1` by a factor of ~2 **and flips sign**. This
   is not a tolerance question: rung 68's detector resolves `δ ≳ 3e−10`.
3. **`det J = c0 = 0` in BOTH.** Rung 68's `c0 = (x+1)²/(x·τ_gτ_vτ_s)` was derived under
   `ac = be = df = 1` and does not apply once the pairs split. **`det` cannot see the split;
   `c1` can** — it moves from `≤ 2.5e−7` to `2.1e+3`. A reader that inherited rung 68's
   determinant test would report rank one and see nothing.
4. **The surviving pair is COMPLEX.** Two zeros collapse to one, and the freed root does not
   land on the real axis — it pairs up. `−30 ± 38.8i` is `ζ = 0.61`.

---

## 1. THE DERIVATION — rank is the number of distinct CONSTRAINTS, not of loops

`n` lagged laws, each solving its **own** constraint for its **own** actuator:

    du_i/ds = (U_i(u_{−i}) − u_i)/τ_i ,     c^{(i)}(u) = 0  defines  U_i

    ∂U_i/∂u_j = −c^{(i)}_j / c^{(i)}_i      ⇒   row_i(M) = −(1/c^{(i)}_i)·∇c^{(i)T}

    M := [∂U_i/∂u_j − δ_ij] ,   J = D·M ,   D = diag(1/τ_i)

**Every row of `M` is a scalar multiple of its own constraint's GRADIENT.** Therefore

    rank M  =  dim span{ ∇c^{(1)}, …, ∇c^{(n)} }  =:  m       ⇒   n − m ZERO EIGENVALUES

The loop count never enters. `m` is the number of **independent constraints**, and the whole
family is one table:

| rung | `n` loops | `m` constraints | zeros | measured |
|---|---|---|---|---|
| 66 | 2 (fuel, valve) | 1 (`φ`) | 1 | `{0, −(1/τ_g+1/τ_v)}` |
| 67 | 2 (valve, governor) | 2 (`φ`, `Tt4`) | 0 | `P = R_q C_g < 0`, no zero |
| 68 | 3 (fuel, valve, stator·`φ`) | 1 (`φ`) | 2 | `x = −1`, two zeros |
| **69** | **3 (fuel, valve, stator·`M_i`)** | **2 (`φ`, `M_i`)** | **1** | *this rung* |

So rung 68's *`n` loops on one variable are ONE loop with all `n` rates added* is the `m = 1`
corner of a statement about **constraints**, and rung 67's non-degenerate pair is the `n = m`
corner. **A loop's COORDINATE, not its actuator, decides whether it adds a zero or a rank.**

### 1.1 What the split does to the two invariants — and why only ONE of them sees it

Two loops (`R`, `C`) on `φ`, one (`V`) on `ψ`. With `φ_g, φ_q, φ_v` and `ψ_g, ψ_q, ψ_v`:

    M = −[ ∇φᵀ/φ_g ; ∇φᵀ/φ_q ; ∇ψᵀ/ψ_v ]

Rows 1 and 2 are **parallel** — both multiples of `∇φ` — so `det M = 0` **identically**,
whatever `ψ` is. The rank deficiency of exactly 1 is carried by the pair that still shares its
constraint, and `det` is blind to the third row entirely:

    pair_RC = φ_q φ_g /(φ_g φ_q)          =  1                    [SURVIVES]
    pair_RV = (φ_v/φ_g)(ψ_g/ψ_v)          =  k                    [SPLITS]
    pair_CV = (φ_v/φ_q)(ψ_q/ψ_v)          =  k                    [SPLITS, same k]
    cyclic  = R_q C_v V_g                 = −k

    c1 = Σ_{i<j} (1 − a_ij a_ji)/(τ_i τ_j) = (1−k)(1/(τ_g τ_s) + 1/(τ_q τ_s))   ≠ 0
    c0 = det J                             = 0                                   ALWAYS

Here `ψ = T_c − 1/φ + v`, so `ψ_g = φ_g/φ²`, `ψ_q = φ_q/φ²`, `ψ_v = φ_v/φ² + 1`, and

    k = (φ_v/φ²)/(φ_v/φ² + 1)          — measured −1.67 … −2.01 over the riding arc

**`pair_RV = pair_CV` exactly** (both are `k`) is a second, independent signature of the split:
the two `φ` loops see the odd one out *identically*, because they differ only by which `φ`
gradient component normalises them, and that component cancels.

### 1.2 The surviving pair, and a DAMPING FLOOR set by `k` alone

`J` has rank 2, so its nonzero spectrum is that of the 2×2 reduction on `span{∇φ, ∇ψ}`. With
`A := 1/τ_g + 1/τ_q` and `z := 1/τ_s`:

    λ₁ + λ₂ = −(A + z) = −Σ 1/τ_i             [the trace; the zero contributes nothing]
    λ₁ λ₂   = A·z·(1 − k)

    ⇒  ζ = (A + z) / (2√(A z (1−k)))   ≥   1/√(1−k)     [AM–GM, equality at A = z]

Three consequences, and all three are the rung:

* **The pair is complex for some bandwidth iff `k < 0`** — i.e. **iff the lever fights itself
  across the two walls.** Ringing is not a control-design accident here; it is the geometric
  signature of a lever that helps one constraint and hurts the other.
* **The damping FLOOR is bandwidth-independent.** `ζ ≥ 1/√(1−k)` holds for every choice of the
  three clocks; no actuator bandwidth can make the plant ring harder than `k` allows.
* **ONE SCALAR sets all three faces** — the pairwise split, the cyclic product, and the damping
  floor. That is rung 67's `P` in a different mechanism, and it is the second time this family
  has found a single number doing all the work.

---

## 2. THE PLANT

`ReferenceSplitTransient` = rung 68's five states and three clocks, with the third law swapped:

    dg/ds = ( R(ν, q, v) − g ) / lag.tau(R, g)     R = rung 52's required clip     [FUEL, φ]
    dq/ds = ( C(ν, g, v) − q ) / τ_v               C = rung 65's b_cmd             [VALVE, φ]
    dv/ds = ( V(ν, g, q) − v ) / τ_s               V = the setting putting M_i on m_lim
                                                                                   [STATOR, M_i]

`StatorIncidenceLimiter(m_lim, v_max, tau)`, band `[0, +v_max]`. The `_b_state`/`_v_state`
boundary is rung 68's, unchanged and still load-bearing. The set-point discipline changes shape:
rung 68 asserted `stator_lim.phi_lim == bleed_lim.phi_lim`; here the matched condition is
`m_lim == T_c − 1/φ_lim`, i.e. **the same physical wall at the design setting** — which is the
only reading of "one set point" that survives a change of coordinate (rung 60's own argument).

---

## 3. PREDICTIONS

| | prediction |
|---|---|
| **P1** | On the FIVE-STATE incidence march, at every riding-interior point: **exactly ONE zero eigenvalue** and a complex conjugate pair. Rung 68's own arm on the same rig gives TWO zeros and a real `−Σ1/τ`. |
| **P2** | **`c0 = det J ≈ 0` in BOTH references** (`|c0|` below `1e−8` of the natural scale) while `c1` moves from `~0` to `(1−k)(1/(τ_gτ_s)+1/(τ_qτ_s))`. **`det` is NOT the discriminator — `c1` is.** A reader inheriting rung 68's determinant test sees nothing. |
| **P3** | The pairwise products SPLIT `1 / k / k` on the shared manifold, with `pair_RV` and `pair_CV` **equal to each other to ≲1 %**, and `cyclic = −k` — all three to the root-finders' floor on `pair_RC` and to the point-mismatch on the other two. |
| **P4** | `ζ ≥ 1/√(1−k)` over every clock grid tried, with the minimum **at `1/τ_g + 1/τ_q = 1/τ_s`** and `ζ ≈ 0.576` there; `ζ ≈ 0.61` at matched clocks. The floor is **bandwidth-independent**. |
| **P5** *(expected to MISS)* | The ring is **NOT observable** in the marched `v`: `ζ ≈ 0.6` allows one ~11 % overshoot, which decays inside one period (`≈0.16` in `s`) while the ramp's own forcing dominates. Rung 67's verdict — *admissible, unobservable* — reached by a different mechanism. |
| **P6** | The ledger's SIGNS **INVERT** relative to rung 68: the incidence stator's `φ`-credit is **≤ 0** (closing lowers `φ`) and its incidence credit **> 0**, against rung 68's +91.70 % / −57.42 %. The valve, which moves neither wall, keeps both signs in both rungs. |
| **P7** | `v_max` is **INERT in company and BINDING alone**, exactly as at rung 68 — the reference does not touch rung 64's extension. Measured § 0.2: 0.0295 needed in company against 0.3314 alone. |
| **P8** | Rung 68's RK4 constant `ds·Σ1/τ ≤ 2` stays **CONSERVATIVE** here because `|λ|/Σ(1/τ) = √(1−k)/2 ≤ 0.87` for the measured `k`, but its **REASON changes**: the true bound is `ds·√(Az(1−k)) ≲ 2.8` and is **PLANT-dependent through `k`**, where rung 68's was pure bandwidth. Predicted margin at matched clocks: `|λ| ≈ 49` against `Σ = 60`. |
| **P9** | The `s = 0` fixed-point family is **SMALLER** than rung 68's (nullity 1 against 2): order-independence survives, and the start-spread comes in **below** rung 68's 45.2 % (`I`) / 105.5 % (withheld fuel). |

---

## 4. WHAT WOULD KILL IT

* `pair_RC` failing to return `1.00000000` on the shared manifold — the split would then be a
  numerics story, not a rank one. (§ 0.3 has already cleared this.)
* The five-state incidence march failing to produce riding-interior points — § 0.2 clears it in
  company and explicitly does **not** clear it alone.
* `c1` coming back at rung 68's `~0` — the constraint would be shared after all, meaning `M_i`
  is a re-parameterisation of `φ` at fixed `v`. It is not, because of the explicit `+v`.
