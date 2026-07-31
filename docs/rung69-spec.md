# Rung 69 — THE REFERENCE SPLIT

**Rung 68's SAME stator, referenced to INCIDENCE (`M_i = T_c − (1/φ − v)`, rung 60's currency)
instead of to `φ`, beside the SAME lagged valve (65) and the SAME lagged fuel leg (52).** Five
states, three clocks, one lever, one physical wall at the design setting. Rung 68's named
strongest seam. **The only thing that moves is the coordinate the third loop watches.**

> **HEADLINE — A LOOP'S COORDINATE, NOT ITS ACTUATOR, DECIDES WHETHER IT ADDS A ZERO OR A
> RANK.** Every row of the actuator block is a multiple of *its own* constraint's gradient, so
> `rank M = dim span{∇c⁽ⁱ⁾}` and **ZEROS = `n − m`**, with `m` the number of INDEPENDENT
> CONSTRAINTS. The loop count never enters. Rung 68's *`n` loops on one variable are ONE loop
> with all `n` rates added* is the `m = 1` corner of that, and rung 67's non-degenerate pair is
> the `n = m` corner.

> **AND `det J` CANNOT SEE IT — which CORRECTS how rung 68's decomposition must be read.** The
> two loops that still share `φ` keep exactly parallel rows, so `det J = 0` **identically**,
> whatever the third row is. Rung 68's `c0 = (x+1)²/(x·τ_gτ_vτ_s)` was derived under
> `ac = be = df = 1` and does not survive the split. What moves is `c1`, by **twelve orders**.

> **AND THE MODE THE SPLIT CREATES RINGS — iff the lever fights itself.** The freed root does
> not land on the real axis: the surviving pair is complex for some bandwidth **iff `k < 0`**,
> and `ζ ≥ 1/√(1−k)` for *every* choice of the three clocks. One scalar sets the pairwise
> split, the cyclic product and the damping floor — rung 67's `P`, different mechanism.

---

## 0. WHAT MOVED, AND WHAT DID NOT

Held fixed from rung 68: the plant, the lever (LP variable stators), the two other loops and
their laws, the clocks, `v_max = 0.20`, the ramp, `ds`, and the **wall** — `m_lim = T_c −
1/φ_lim = 0.568182` is rung 64's `φ_lim = 0.80` read at the design stator setting. Changed:
**one coordinate.**

That matters because "one set point" cannot mean *one float* across a change of coordinate.
Rung 68 asserted `stator_lim.phi_lim == bleed_lim.phi_lim`; here the constructor asserts
`m_lim == T_c − 1/φ_lim` instead. Anything else would confound the reference split with a
set-point offset — rung 66 measured a −2.5 % offset moving its own product to 0.951.

### 0.1 The direction is now the PHYSICAL one

Measured at `s = 0.29` on the shipped rung-66 march (`docs/plans/rung69-anchor-reference-split.md`
§ 0.1), against a trial LP stator setting:

| `v` | −0.20 | −0.05 | 0.00 | +0.05 | +0.20 | +0.40 |
|---|---|---|---|---|---|---|
| `φ_lp` | 0.8949 | 0.8196 | 0.7978 | 0.7773 | 0.7230 | 0.6637 |
| `M_i` | 0.5007 | 0.5481 | 0.5647 | 0.5817 | 0.6350 | 0.7115 |

    dφ_lp/dv = −0.42300        dM_i/dv = +0.33537

`M_i` is **increasing** in `v`, so this loop **closes** the stators — what a real VSV schedule
does at low corrected speed, and the exact opposite of rung 68's. **Rung 68 had to disclose an
anti-physical lever; this rung does not.** The band is one-sided the other way, `v ∈ [0, +v_max]`,
so `_solve_v`'s bracket and both clamp tests flip **back** to `_solve_b`'s — rung 62's `_powers`
trap in its fifth reload, and it fails silently. Gated from both sides.

### 0.2 The scalar the whole rung turns on

    ψ := M_i = T_c − 1/φ + v        ⇒     ψ_v = φ_v/φ² + 1
    k := (φ_v/φ²)/ψ_v               measured **−1.664 … −2.000** over the riding arc

**`k < 0` iff the lever's two channels FIGHT** — iff it raises one wall while lowering the
other. That one number is the split, compressed.

### 0.3 Feasibility, and the one thing it does NOT clear

The incidence-rooted command over three inherited marches, `s ≤ r`:

| march | `min φ_lp` | max `V` needed | n(`V > 0.20`) | n(dormant) |
|---|---|---|---|---|
| bare | 0.735442 | **0.3314** | **84 / 100** | 0 |
| fuel only | 0.773116 | 0.1175 | 0 | 0 |
| fuel + valve | 0.793085 | 0.0295 | 0 | 27 |

In company the loop rides comfortably inside the inherited `v_max`; **alone it saturates over
84 % of the ramp.** The `S` and `FS` ledger cells are therefore authority-limited by a ceiling
this project chose in rungs 57/58. Disclosed, not raised — raising it would make it a new
constant. The five-state march itself: 341 points, **64 riding**, `s ∈ [0.005, 0.320]`,
`v ∈ [0, 0.012604]`, `min φ_lp = 0.791380`, IC `(g, b, v) = (0, 0.036626, 0)` in one iteration
with residual exactly 0 — rung 66's own declared member, unchanged.

---

## 1. THE DERIVATION — rank is the CONSTRAINT count

`n` lagged laws, each solving its own constraint for its own actuator:

    du_i/ds = (U_i(u_{−i}) − u_i)/τ_i ,      c⁽ⁱ⁾(u) = 0  defines  U_i

    ∂U_i/∂u_j = −c⁽ⁱ⁾_j / c⁽ⁱ⁾_i      ⇒     row_i(M) = −(1/c⁽ⁱ⁾_i)·∇c⁽ⁱ⁾ᵀ

    M := [∂U_i/∂u_j − δ_ij] ,   J = D·M ,   D = diag(1/τ_i)

Every row is a scalar multiple of **its own** gradient, so

    rank M  =  dim span{ ∇c⁽¹⁾ … ∇c⁽ⁿ⁾ }  =:  m        ⇒        n − m  ZERO EIGENVALUES

| rung | `n` loops | `m` constraints | zeros | the shape of it |
|---|---|---|---|---|
| 66 | 2 (fuel, valve) | 1 (`φ`) | 1 | `{0, −(1/τ_g+1/τ_v)}` |
| 67 | 2 (valve, governor) | 2 (`φ`, `Tt4`) | 0 | `P = R_q C_g < 0` |
| 68 | 3 (+ stator·`φ`) | 1 (`φ`) | 2 | `x = −1`, rank one |
| **69** | **3 (+ stator·`M_i`)** | **2 (`φ`, `M_i`)** | **1** | **this rung** |

### 1.1 What the split does to the two invariants — and why only ONE sees it

Two loops (`R`, `C`) on `φ`, one (`V`) on `ψ`:

    M = −[ ∇φᵀ/φ_g ; ∇φᵀ/φ_q ; ∇ψᵀ/ψ_v ]

Rows 1 and 2 are **parallel**, so `det M = 0` identically whatever `ψ` is. The rank deficiency
of exactly 1 is carried by the pair that still shares its constraint, and `det` never looks at
the third row at all:

    pair_RC = 1                                                     [SURVIVES]
    pair_RV = (φ_v/φ_g)(ψ_g/ψ_v) ,  pair_CV = (φ_v/φ_q)(ψ_q/ψ_v)    [SPLIT]
    cyclic  = R_q·C_v·V_g = −pair_RV
    c1 = Σ_{i<j}(1 − a_ij a_ji)/(τ_i τ_j)          ≠ 0
    c0 = det J                                     = 0      ALWAYS

**`pair_RV = pair_CV` is NOT general to a split.** It holds iff `ψ_g/φ_g = ψ_q/φ_q`, i.e. iff
the odd constraint depends on `(g, q)` **only through the shared one**. Here
`M_i = T_c − 1/φ + v` does, trivially — the two walls differ by exactly the lever's own direct
channel — so both split pairs collapse to the single

    k = (φ_v/φ²)/(φ_v/φ² + 1)

That equality is therefore a **measurement of the two walls' relationship**, not a restatement
of the rank. `tests/test_rung69.py::test_a_determinant_provably_cannot_see_a_split` hand-builds
both blocks — a generic `ψ` and this plant's — and confirms `det = 0` with rank 2 in *both*,
with the split pairs equal only in the second.

### 1.2 THE EVALUATION MANIFOLD IS FORCED, NOT CHOSEN

`R_q·C_g = 1` is an implicit-function identity that holds only when **both** `φ` loops sit at
their own rest points — i.e. with the base point ON `φ = φ_lim`. Rung 68 could put all three
laws there at once. **Here there is no such point:** `φ = φ_lim` and `M_i = m_lim` together
force `v = 0`, the stator's own dormant stop. So the base is the **shared** constraint's
manifold — rung 68's `manifold=True` instrument unchanged, which is what keeps the two rungs'
numbers differenceable (rung 63's lesson). Read at the stator's own root instead, `pair_RC`
degrades from `3.9e−10` to `3.1e−2`; that reading is reported and never gated on.

**Exactness is a property of a SHARED constraint.** That is the derivation's own corollary and
it is why the split pairs agree with `k` to 0.7 % rather than to 1e−10.

**AND THE BASE POINT LIES OUTSIDE THE INCIDENCE LOOP'S OWN BAND.** Every `v_φ` in § 1.3 is
NEGATIVE (−0.0039 … −0.0162), because the shared manifold sits at `v < 0` wherever the two `φ`
loops still lag their commands — and the incidence limiter's band is `[0, +v_max]`. So the
gated rows are linearisations of the ODE at a state the clamped plant **cannot occupy**. The
rank claim survives untouched (it is a statement about row spans, valid at any point), but this
is stated plainly rather than left implied by *a diagnostic base point and not a state*.

### 1.3 MEASURED — both references, one trajectory, one base point

The incidence five-state march; the rung-68 rig evaluated at *its* points, so the two
references are differenced on one trajectory rather than on two. `dg = 1e−7`, `dq = 1e−5`,
`dv = 1e−4`, `τ = (0.05, 0.05, 0.05)`:

| `s` | `pair_RC` | `pair_RV` | `pair_CV` | cyclic `x` | `k` | rung 68 at the same point |
|---|---|---|---|---|---|---|
| 0.005 | 1.00000000 | −1.66662 | −1.66193 | **+1.66662** | −1.6643 | `1 / 1 / 1`, `x = −1` |
| 0.105 | 1.00000000 | −1.74777 | −1.74126 | **+1.74777** | −1.7445 | `1 / 1 / 1`, `x = −1` |
| 0.205 | 1.00000000 | −1.85338 | −1.84948 | **+1.85338** | −1.8514 | `1 / 1 / 1`, `x = −1` |
| 0.305 | 1.00000000 | −2.00071 | −2.00008 | **+2.00071** | −2.0004 | `1 / 1 / 1`, `x = −1` |

**`pair_RC` IS THE NEGATIVE CONTROL, NOT A SIGNAL, and the distinction matters.** `R_q·C_g`
involves only the fuel and valve closures, neither of which can see the stator's reference, and
both columns are evaluated at the same base point — so at a fixed base it is *literally the
same computation twice*. It **structurally cannot** move with the reference. That is exactly
what makes it useful: it calibrates the split pairs' departure against this instrument's own
floor. Worst `|pair_RC − 1|`: **3.88e−10** (split) and **1.54e−10** (`φ`), so the split's
factor-of-two-and-a-sign departure is 9 orders above the floor that produced it — against a
detector rung 68 measured as resolving `δ ≳ 3e−10`.

Worst `|pair_RV − pair_CV|/|k|`: **0.73 %** over the sampled points at `every = 10` (a coarser
sample reports a smaller worst — `main.py`'s panel runs `every = 20` and prints 0.37 %; the
quantity is a per-point residual, not a converged bound).

---

## 2. THE SPECTRUM — one zero, and `det` blind to it

`triple`-grade readers on the shipped closures, `ds = 0.002`, four clock arms, both references:

| `(τ_g, τ_v, τ_s)` | ref | zeros | max `|c0|/Σ³` | min `|c1|/Σ²` | pair | `ζ` |
|---|---|---|---|---|---|---|
| (0.05, 0.05, 0.05) | **inc** | **[1]** | 2.87e−11 | **6.00e−1** | complex | 0.616 … 0.645 |
| (0.05, 0.05, 0.05) | phi | [2] | 7.79e−18 | 7.43e−14 | real | 1.0 |
| (0.05, 0.005, 0.05) | **inc** | **[1]** | 4.99e−12 | **2.03e−1** | **real** | 1.0 |
| (0.05, 0.005, 0.05) | phi | [2] | 1.11e−18 | 1.92e−12 | real | 1.0 |
| (0.05, 0.5, 0.05) | **inc** | **[1]** | 3.64e−12 | **6.76e−1** | complex | 0.584 … 0.608 |
| (0.05, 0.5, 0.05) | phi | [2] | 9.97e−19 | 1.61e−12 | real | 1.0 |
| (0.02, 0.05, 0.10) | **inc** | **[1]** | 1.80e−11 | **2.95e−1** | complex | 0.889 … 0.920 |
| (0.02, 0.05, 0.10) | phi | [2] | 3.53e−18 | 1.62e−12 | real | 1.0 |

Sample roots, matched clocks: `[0, −30 ± 35.52i]` at `s = 0.042` (`k = −1.702`) and
`[0, −30 ± 38.33i]` at `s = 0.282` (`k = −1.962`), against rung 68's `[0, 0, −60]` throughout.

**Three readings, three different contents:**

* **zeros** — `[1]` on every arm here, `[2]` on every arm under `φ`. This is the rung.
* **`c0 = det J`** — zero under **both**, and that is not a near miss: `2.9e−11` against a
  natural scale of 1. **A reader that inherited rung 68's determinant test would report rank
  one and see nothing.** (The split's `c0` sits ~7 orders above the shared case's `1e−18`
  because the parallelism of rows 1 and 2 is reached through two independent root-finds rather
  than being structural in the instrument — disclosed, and still zero by any scale that matters.)
* **`c1`** — `≥ 0.20` under the split against `≤ 7.4e−14` under `φ`. The discriminator, and it
  is not the one rung 68 used. **The `φ` column is a differencing noise floor, not a bound:**
  its true value is 0, so what is quoted moves with the sampling (`main.py`'s coarser panel run
  reports 1.7e−11 for the same arm). Read the separation as *more than ten orders*, which is
  what the gate asserts, rather than as a reproducible twelve.

`c2 = tr J` is the ODE's own diagonal in both and is not a measurement.

---

## 3. THE MODE — a damping floor set by `k` alone

`J` has rank 2, so its non-zero spectrum is the 2×2 reduction on `span{∇φ, ∇ψ}`. With
`A := 1/τ_g + 1/τ_q` and `z := 1/τ_s`:

    λ₁ + λ₂ = −(A + z) = −Σ 1/τ_i            λ₁λ₂ = A·z·(1 − k)

    ζ = (A + z) / (2√(A z (1−k)))    ≥    1/√(1−k)        [AM–GM, equality at A = z]

Measured (`damping_floor`, `ds = 0.005`), the grid straddling `A/z = 1`:

| `(τ_g, τ_v, τ_s)` | `A/z` | `k` | `ζ` measured | `ζ` closed form | floor `1/√(1−k)` | `\|λ\|` |
|---|---|---|---|---|---|---|
| (0.05, 0.05, 0.025) | **1.00** | −1.81042 | **0.59651** | 0.59651 | **0.59651** | 67.06 |
| (0.10, 0.10, 0.05) | **1.00** | −1.82270 | **0.59521** | 0.59521 | **0.59521** | 33.60 |
| (0.05, 0.05, 0.05) | 2.00 | −1.80423 | 0.63339 | 0.63339 | 0.59716 | 47.36 |
| (0.20, 0.02, 0.05) | 2.75 | −1.79581 | 0.67635 | 0.67621 | 0.59806 | 55.45 |
| (0.02, 0.20, 0.05) | 2.75 | −1.74435 | 0.68223 | 0.68252 | 0.60364 | 54.97 |
| (0.05, 0.05, 0.10) | 4.00 | −1.78757 | 0.74868 | 0.74868 | 0.59895 | 33.39 |

The closed form matches the shipped cubic's own dominant root to **4.3e−4** worst. Three things
follow, and all three are the rung:

* **The pair is complex for some bandwidth iff `k < 0`** — iff the lever fights itself across
  the two walls. Ringing here is not a control-design accident; it is the geometric signature
  of a lever that helps one constraint and hurts the other.
* **The floor is BANDWIDTH-INDEPENDENT.** The two `A/z = 1` arms differ by 2× in every clock
  and reach the same `ζ` to 0.2 %; no choice of the three bandwidths can make this plant ring
  harder than `k` allows.
* **The window has EDGES, so "complex" is a measurement.** At `τ_g = 0.005` (`A/z = 11`) the
  pair goes back on the real axis (`[0, −68.0, −172.0]`) — **while the rank does not care**:
  that arm still reads exactly one zero.

**ONE SCALAR sets all three faces** — the pairwise split, the cyclic product, and the damping
floor. That is rung 67's `P` in a different mechanism, and the second time this family has
found a single number doing all the work.

### 3.1 Is the mode OBSERVABLE? — rung 67's question, and a better answer than the count

`ζ ≥ 0.595` allows at most one overshoot of ~11 % of a displacement. The probe displaces the
stator's initial position off its own command (rung 68's `v0`) and reads the tracking error
`e = v − v_cmd` over the riding arc, with **rung 68's reference on the same rig as a negative
control** — its spectrum is provably real, so anything it shows is not a ring.

| | `φ` reference (no complex pair) | incidence reference |
|---|---|---|
| fraction of a `0.05` displacement surviving as an error | **2.30e−14** | **0.2244** |
| error zero-crossings, displaced | 1 | 2 |
| counter-swing | — (nothing to swing) | 19.8 % |
| error zero-crossings, UNDISPLACED | 1 | 1 |

**The headline here is the first row, not the crossings.** Under the shared constraint the
`s = 0` fixed points are a family, so a displaced stator start just selects another member and
**the other two loops absorb it exactly** — there is nothing left to ring. Under the split they
cannot, and a fifth of it survives. That is the rank difference showing up in the **trajectory**
rather than in a Jacobian.

The ring itself is **not separably observable**: the undisplaced run reverses the error's sign
once too (the ramp's own forcing does it), so a crossing count cannot tell the mode from the
forcing. Rung 67's verdict — *admissible, unobservable* — is reached a second time by a
different mechanism. Two honest limits: the one-sided band clamps an undershoot below the
dormant stop, and the ramp is the only forcing available on this plant.

---

## 4. THE LEDGER — the whole SIGN TABLE flips with the reference

Rung 68's 8-cell ledger, run twice on one rig. The four stator-free cells (`bare`, `F`, `V`,
`FV`) are **bit-identical** between the references, which is a free check that the two ledgers
are differenceable rather than merely similar.

**Reference = INCIDENCE (this rung):**

| cell | `I` (φ) | credit | `I` (incid.) | credit | `min φ` | `v` used | `b` used |
|---|---|---|---|---|---|---|---|
| bare | 2.581532e−2 | — | 4.329085e−2 | — | 0.735442 | 0 | 0 |
| F | 1.018390e−2 | 60.55 % | 1.633403e−2 | 62.27 % | 0.773116 | 0 | 0 |
| V | 1.939430e−3 | 92.49 % | 3.062688e−3 | 92.93 % | 0.789123 | 0 | 0.0899 |
| **S** | **5.546652e−2** | **−114.86 %** | 1.035550e−2 | **+76.08 %** | **0.673049** | +0.2000 † | 0 |
| FV | 1.528558e−3 | 94.08 % | 2.404086e−3 | 94.45 % | 0.793085 | 0 | 0.0789 |
| FS | 1.114897e−2 | 56.81 % | 8.288092e−4 | 98.09 % | 0.773336 | +0.0426 | 0 |
| VS | 2.016977e−3 | 92.19 % | 4.166002e−4 | 99.04 % | 0.785043 | +0.0225 | 0.0924 |
| **FVS** | 1.593313e−3 | 93.83 % | **2.346933e−4** | **99.46 %** | 0.791380 | +0.0126 | 0.0809 |

† saturated — see § 0.3.

**Reference = `φ` (rung 68, same rig):**

| cell | `I` (φ) | credit | `I` (incid.) | credit | `min φ` | `v` used |
|---|---|---|---|---|---|---|
| **S** | 2.142124e−3 | **+91.70 %** | 6.814711e−2 | **−57.42 %** | 0.788430 | −0.1666 |
| FS | 3.261458e−3 | 87.37 % | 5.101011e−2 | −17.83 % | 0.790284 | −0.1577 |
| VS | 1.022884e−3 | 96.04 % | 1.727632e−2 | 60.09 % | 0.793448 | −0.0496 |
| **FVS** | **8.952178e−4** | **96.53 %** | 1.621330e−2 | 62.55 % | 0.795155 | −0.0435 |

**Every one of the four stator cells changes sign:**

| | `φ`-referenced | incidence-referenced |
|---|---|---|
| stator alone, `φ` credit | **+91.70 %** | **−114.86 %** |
| stator alone, incidence credit | **−57.42 %** | **+76.08 %** |
| marginal stator, `φ` | +2.453 % | **−0.251 %** |
| marginal stator, incidence | **−31.899 %** | +5.011 % |

The sharpest single number: the incidence-referenced stator running **alone** drives
`min φ_lp` to **0.673**, *below the bare march's own 0.735* — a limiter that is measurably
worse than no limiter at all, in the currency it does not watch. And the triple delivers
99.46 % on the wall its third loop watches against rung 68's 62.55 %, and 93.83 % on the other
against rung 68's 96.53 %.

### 4.1 IS THE SIGN TABLE GRID-CONVERGED? — measured, because rung 65 retracted one

Rung 68 published a `ds` refinement because rung 65 shipped an RK4 artifact that read as a
physical finding. This plant's dominant root is a lightly-damped **complex** pair, a different
aliasing character from rung 68's real one, so the question is not inherited. Every cell that
carries the sign table, at `ds = 0.005` and `0.0025`:

| cell | `I`(φ) shift | `I`(incid.) shift | | credit | at 0.005 | at 0.0025 |
|---|---|---|---|---|---|---|
| bare | +0.28 % | +0.27 % | | **inc**: S alone, φ | −114.86 % | **−115.04 %** |
| FV | +0.04 % | +0.04 % | | **inc**: S alone, incid. | +76.08 % | **+76.14 %** |
| inc S | +0.36 % | +0.01 % | | **inc**: marginal, φ | −0.251 % | **−0.250 %** |
| inc FVS | +0.04 % | +0.33 % | | **inc**: marginal, incid. | +5.011 % | **+4.998 %** |
| phi S | +0.03 % | +0.35 % | | **phi**: S alone, φ / incid. | +91.70 / −57.42 % | **+91.72 / −57.54 %** |
| phi FVS | +0.06 % | +0.30 % | | **phi**: marginal, φ / incid. | +2.453 / −31.899 % | **+2.447 / −31.923 %** |

**Nothing moves past the third figure**, including the smallest number in the table (the
incidence loop's own `φ` marginal, `−0.25 %`, whose *sign* is the delicate one). `ds·|λ| = 0.245`
against the § 7 guard, so this is the interior of the stability region and not its edge.

**So rung 53's *a margin is a DISTANCE* lands one level up again.** Rung 53 bounded rungs
36–52's currency; rung 54 their constraint severity; rung 56 a lever's cost; rung 68 the *sign
of a protection credit* — showing a credit is meaningless without its **wall**. Rung 69 shows
it is meaningless without its loop's **reference** as well: same lever, same plant, same two
walls, same set point, and every sign in the table decided by which wall the loop watches.

---

## 5. AUTHORITY — rung 64's ceiling, and the SIGN the split gives it

| `v_max` | inc, `I`(φ) | inc, `I`(incid.) | phi, `I`(φ) | phi, `I`(incid.) |
|---|---|---|---|---|
| 0.05 | 3.430874e−2 | 3.408005e−2 | 1.695658e−2 | 5.257605e−2 |
| 0.10 | 4.244719e−2 | 2.492079e−2 | 8.686789e−3 | 6.092089e−2 |
| 0.20 | 5.546652e−2 | 1.035550e−2 | 2.142124e−3 | 6.814711e−2 |
| 0.40 | 6.199758e−2 | 3.248533e−3 | 2.142124e−3 | 6.814711e−2 |

On the lever **alone**, under **both** references: more authority monotonically **improves the
wall the loop watches** and monotonically **degrades the other**. Rung 64's *the ceiling is the
lever's AUTHORITY* acquires a sign, and it is the loop's reference that sets it.

The two levers also **run out at different ceilings**: the `φ`-referenced one is done at
`v_max = 0.20` (0.20 and 0.40 agree to 1e−12 — it stops saturating), while the incidence one is
still authority-starved there and improves 3.2× more out to 0.40. That is § 0.3's 0.3314
requirement, showing up in the ledger.

**In company the ceiling is inert**, exactly as at rung 68: `I` identical to 1e−11 across an
8× ceiling (0.05 → 0.40) with the loop never reaching its stop, because the other two take up
the demand first. **Rung 68's extension of rung 64 survives the reference change untouched.**

---

## 6. THE INITIAL CONDITION — a null space is a SHOCK ABSORBER

| | rung 66 (`n−m = 1`) | rung 68 (`n−m = 2`) | **rung 69 (`n−m = 1`)** |
|---|---|---|---|
| violation-integral spread over a moved start | ±20 % | 45.2 % | **187.0 %** |
| withheld-fuel spread | 84 % | 105.5 % | **291.1 %** |

**Pre-registered the other way (P9) and MISSED, and the miss is the content.** Rung 68 measured
the growth 84 % → 105.5 % and explicitly declined to attribute it to its second zero
eigenvalue. This rung supplies the counter-example: dropping the nullity from 2 back to 1 makes
both spreads grow *again*, so **the zero count and the IC sensitivity move in opposite
directions**. A null space is a shock absorber — redundant loops redistribute a moved start
among themselves. § 3.1 is the same mechanism read at a single point (`2.3e−14` absorbed
against `0.2244` surviving), and rung 68's refusal to attribute was right.

The declared starting member is unchanged (`g = 0`, `q = b_cmd(0) = 0.036626`, `v = 0`) and all
six Gauss-Seidel orders still land on it in one iteration with residual exactly 0, so the ORDER
is still not the lever.

---

## 7. THE RK4 FLOOR — the constant survives, its REASON does not

Rung 68's guard is `ds·Σ(1/τ_i) ≤ 2` and its justification is that `J` is rank one with its
non-zero eigenvalue exactly `−Σ1/τ_i`. **That justification is gone.** The dominant root is now
a complex pair of modulus `√(A z (1−k))`, and by AM–GM

    |λ| / Σ(1/τ_i)  =  √(A z (1−k)) / (A + z)  ≤  √(1−k)/2

so the constant stays conservative for every plant with `k ≥ −3`. Measured on the arc
(`rk4_margin`, matched clocks): `max|λ| = 48.99` against `Σ1/τ = 60`, ratio **0.8165** against
the bound **0.8661** — conservative, and not slackly so. The guard's **character** changed:
rung 68's floor was a property of the CLOCKS alone; this one is a property of the **PLANT**,
through `k`. It is measured against the plant rather than trusted, because rung 65 published a
retraction for exactly the failure mode of a trusted stability argument — and § 4.1 carries that
through to the ledger, halving `ds` on every cell that sets a sign.

---

## 8. Reduce contract

* `stator_inc=None` ⇒ **rung 68/67/66 bit-for-bit, by dispatch.** Every override returns the
  parent's answer verbatim; with a `stator_lim` armed instead, rung 68's own five-state path is
  reached unchanged (gated on the full trajectory key, and on the band running *negative*).
* Every inherited arm leaves through the same `super()`: rung 66 (no stator), rung 65
  (`lag=None`), rung 52 (no valve), rung 64 (no clocks), rung 62 (a bleed schedule).
* A `StatorIncidenceLimiter` with `tau=None` is **refused**, not silently dropped (rung 66's
  discipline, inherited).
* Arming **both** stator references is refused — that would be two loops on one *actuator*,
  a different object again.
* The four seams rung 69 reaches into rung 68 through (`_stator_leg`, `_clamp_v`/`_check_v0`,
  `_manifold_v`, plus `_solve_v`, which was already a method) are each the **identity** of what
  they replaced, so rung 68's own gates are untouched.

---

## 9. Predictions, scored

The anchor (`docs/plans/rung69-anchor-reference-split.md`) pre-registered nine.

| | prediction | verdict |
|---|---|---|
| **P1** | exactly ONE zero eigenvalue and a complex pair, at every riding-interior point | **SPLIT.** `zeros == [1]` on all four clock arms against rung 68's `[2]` — but the pair is complex only *inside* the window § 3 derives. At `A/z = 11` it is real, **and the rank does not care.** The anchor stated "complex" unconditionally; the derivation it was drawn from did not. |
| **P2** | `c0 ≈ 0` in BOTH; `c1` moves from ~0 to `(1−k)(1/(τ_gτ_s)+1/(τ_qτ_s))`; **`det` is not the discriminator** | **HIT.** `c0/Σ³ ≤ 2.9e−11` (split) and `≤ 7.8e−18` (shared); `c1/Σ² ≥ 0.20` against `≤ 7.4e−14`. Twelve orders. |
| **P3** | pairwise products SPLIT `1 / k / k`, the two split pairs equal to ≲1 %, `cyclic = −k` | **HIT** (0.73 %), and **SHARPENED**: the equality is not general to a split — it needs `ψ` to depend on `(g,q)` only through `φ`. § 1.1, and a gate hand-builds the counter-example. |
| **P4** | `ζ ≥ 1/√(1−k)` over every clock grid, minimum at `A = z`, floor ≈0.576, matched ≈0.61 | **HIT on the law and the minimiser** (reached to 1e−9 at both `A/z = 1` arms; closed form matches the cubic to 4.3e−4). **Magnitudes off ~3 %** — 0.595 and 0.633 measured — because the anchor quoted them at the arc's *extreme* `k = −2.01` rather than at the sampled point's `−1.80`. |
| **P5** | the ring is NOT observable in the marched `v` | **HIT on the verdict, and the probe found something the prediction did not.** Crossings cannot separate the mode from the ramp (the undisplaced run reverses too) — but a displaced start reveals that the **shared-constraint plant absorbs the displacement to 2.3e−14** while the split one keeps 22.4 % of it. (The anchor's "*expected to MISS*" tag was a slip; the prediction as written is what is scored.) |
| **P6** | the ledger's SIGNS invert relative to rung 68 | **HIT, in all four cells.** +91.70/−57.42 becomes −114.86/+76.08; +2.45/−31.90 becomes −0.25/+5.01. The incidence loop alone drives `min φ` *below the bare march*. |
| **P7** | `v_max` inert in company, binding alone — the reference does not touch rung 64's extension | **HIT, and EXTENDED.** Inert to 1e−11 across an 8× ceiling. Alone, authority acquires a **sign**: monotone better on the watched wall, monotone worse on the other, under both references — and the two levers run out at different ceilings. |
| **P8** | rung 68's RK4 constant stays conservative but its REASON changes; `\|λ\| ≈ 49` vs `Σ = 60` | **HIT.** `max\|λ\| = 48.99`, ratio 0.8165 against the derived bound 0.8661. |
| **P9** | the `s = 0` IC family is SMALLER than rung 68's (nullity 1 vs 2) | **MISS, and the miss is the finding.** 187 % / 291 % against rung 68's 45.2 % / 105.5 %. **A null space ABSORBS a moved start**, so the zero count and the IC sensitivity move in opposite directions — § 6. Rung 68's refusal to attribute its own growth to the second zero was right. |

---

## 10. Concessions

* Every one rungs 62–68 list, all inherited.
* **`v_max = 0.20` is rungs 57/58's inherited setting and the loop SATURATES on it over 84 % of
  the ramp when it runs alone.** The `S` and `FS` cells are authority-limited by a ceiling
  chosen elsewhere; § 5 measures out to 0.40 and reports the difference rather than adopting it.
* `φ_lim`/`m_lim` and `b_max` remain **IMPOSED** (rung 64, verbatim). `m_lim` adds no constant —
  it is `T_c − 1/φ_lim` with `T_c = 1/φ_surge`, rung 53's zero-new-constant channel.
* The two floors are matched **at the design setting** and diverge as the lever moves. That
  divergence *is* the experiment, but it means the two references are compared at equal **wall**
  and not at equal excursion.
* `τ_s` remains a swept coordinate on the march's own `s`; no actuator bandwidth is anchored
  anywhere in this family. ORDERINGS, SIGNS and INVARIANCES are the claims; every MAGNITUDE is
  disclaimed.
* § 3.1's observability verdict rests on a crossing count with a one-sided clamp and a single
  forcing (the ramp). It is a bound on what *this* instrument can see, not a proof the mode is
  invisible.
* § 2's spectrum is sampled at finitely many trajectory points — a DIAGNOSTIC that can miss a
  brief excursion (rung 65's retracted trap), not a proof of convergence.
* All three lags are SYMMETRIC except rung 52's fuel leg. Rung 65's asymmetric-valve seam and
  rung 67's asymmetric-governor seam remain untouched.
* The STAGE STACK (rungs 55/56) is still not on the transient ladder, so rung 56's binding-row
  migration is invisible here.
* This still does **not** close rung 63's *fuel + bleed + STATOR* seam: that seam wants the
  stator as an OPEN-loop SCHEDULE, and this is a closed loop.

---

## 11. Next seams

* **`m` DIRECTLY, without changing `n`.** § 1 predicts `zeros = n − m` for every `(n, m)`, and
  four points now sit on that line — but all four were reached by *changing a loop*. The clean
  test moves `m` alone: two loops on `φ` plus **two** on `M_i` (`n = 4, m = 2` ⇒ 2 zeros again,
  at a different loop count). It needs a fourth lever with authority over the LP, which this
  plant does not have — the same hardware wall rung 68's `n = 4` seam hit.
* ~~**A plant with `pair_RV ≠ pair_CV`**~~ — **CLOSED BY RUNG 70** (`docs/rung70-spec.md`), and
  it did close rung 68's *three loops on TWO variables* with it, exactly as predicted here. The
  governor's constraint does not factor, so the two split pairs separate — **with OPPOSITE
  SIGNS**, `−0.017…−0.020` against `+0.113…+0.127`. Three consequences for THIS rung's reading:
  § 1.1's `pair_RV = pair_CV` is confirmed as *a measurement of the two walls' relationship*
  and not of the rank; § 1.4's `c1 = (1−k)(…)` is the factoring special case of a **clock-
  weighted two-term sum**, so its bandwidth-independence was a property of the collapse; and
  § 3's damping floor, attained here on a finite hyperplane, becomes an **infimum on a ray**.
  **Rung 70's floor is rung 67's `ζ` exactly** — a lever fighting itself is what made THIS
  rung's `k ≈ −1.7` and hence its visible ring.
* **A plant with `k < −3`**, where the inherited RK4 constant stops being conservative (§ 7) and
  the guard would have to be re-derived rather than re-justified. `k` is a plant property, so
  this needs a lever whose two channels fight harder — not a new law.
* **Is the ring reachable by any forcing this plant admits?** § 3.1 could only ask the ramp.
  A step, or a displaced start away from the one-sided stop, would separate
  *unobservable-because-damped* from *unobservable-because-clamped*.
* **An ASYMMETRIC valve** (rung 65) and an **asymmetric governor** (rung 67) — both still open.
* **Fuel + bleed + STATOR-as-a-SCHEDULE**, all three on one plant — rung 63's seam, still open
  after 64/65/66/67/68 and now after 69.
* Everything rung 68 § 10 left: a plant with `|P| > 1`, `n = 4` on one variable, and the real
  spatial/transported-CFD PDF.
