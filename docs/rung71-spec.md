# Rung 71 — THE FULL SPLIT

Rung 69's move — swap **one loop's COORDINATE and nothing else** — applied to rung 70's plant.
Rung 68's `φ` stator becomes rung 69's INCIDENCE stator, beside rung 47's `Tt4` governor and
rung 65's `φ` valve.

    dg/ds = ( R(ν,q,v) − g ) / τ_g    R = rung 47's clip,  Tt4 ≤ Tt4_max   [GOVERNOR, Tt4]
    dq/ds = ( C(ν,g,v) − q ) / τ_q    C = rung 65's b_cmd, φ_lp ≥ φ_lim    [VALVE,    φ]
    dv/ds = ( V(ν,g,q) − v ) / τ_s    V = rung 69's v_cmd, M_i ≥ m_lim     [STATOR,   M_i]

Five states, three clocks, one actuator per loop, **three constraints**. `n = m = 3`, so
`zeros = 0` — **the last unoccupied cell of rung 69 § 1's table**, named by rung 70 § 6.1 and
§ 9 as its strongest seam.

> **HEADLINE — A CONSTRAINT CAN BE INDEPENDENT IN RANK AND REDUNDANT ON THE BAND.**
> The Jacobian is full rank and the third loop is live over **2.05 %** of the march, because at
> the valve's own set point `M_i = m_lim + v ≥ m_lim` for every admissible `v ≥ 0`: the third
> constraint is **implied** by the second's on the whole band. `zeros = n − m` counts GRADIENT
> DIRECTIONS; it does not count LIVE loops. Rung 69's rank law is **bounded**, not corrected.

> **AND `det J`, NON-ZERO FOR THE FIRST TIME IN THIS FAMILY, FACTORS INTO THE TWO PRIOR RUNGS'
> OWN NON-DEGENERACY CONDITIONS — ONE FACTOR PER RUNG** — `−(1 − pair_RC)(1 − pair_CV)`, rung
> 67's times rung 69's. It is therefore still **blind to `pair_RV`**, the only gain this rung
> contains that no earlier one measured: it cancels exactly against the *reverse* cyclic
> product. Only `c1` has ever seen that gain, at rung 70 or here.

> **AND RUNG 69's DAMPING FLOOR TURNS OUT TO BE THE `c0 = 0` CORNER.** All three roots share one
> trace budget; at rung 69 the third root *was* the zero and took none of it. Here the third
> loop's own pole drains it, the bound is violated on 3 of 10 clock arms, and a **Routh
> certificate** (`u + w + z ≥ u·z`) replaces it.

**AND IT CORRECTS RUNG 70 § 5.** *A loop is eroded by the loops it shares a constraint with, and
by no others* — no two loops share here and the stator keeps **5.5 %** of its solo credit.

Pre-registration: `docs/plans/rung71-anchor-full-split.md`, whose § 0 discloses its own order.
Gates: `tests/test_rung71.py`.

---

## 0. WHAT MOVED, AND WHAT DID NOT

| | rung 68 | rung 69 | rung 70 | **rung 71** |
|---|---|---|---|---|
| the three loops | fuel, valve, stator | fuel, valve, stator | **gov**, valve, stator | **gov, valve, stator** |
| coordinates | `φ,φ,φ` | `φ,φ,M_i` | `Tt4,φ,φ` | **`Tt4, φ, M_i`** |
| `(n, m)` | (3, 1) | (3, 2) | (3, 2) | **(3, 3)** |
| zeros | 2 | 1 | 1 | **0** |
| pairs equal to 1 | all three | `(R,C)` | `(C,V)` | **none** |
| `det J` | 0 | 0 | 0 | **≠ 0, and it FACTORS** |
| the ring | none (real) | complex, `ζ ≥ 0.61` | real except on a ray | **complex on 9/10 arms, no floor** |
| `s = 0` fixed points | a curve | a curve | a curve | **a POINT** |

The plant, the ramp, the maps, `φ_lim`, `b_max`, `v_max`, `m_lim` and `Tt4_max` are all
inherited. **The only thing that changes between rungs 70 and 71 is which wall the stator
watches** — which is exactly the move rung 69 made on rung 68, so the two pairs of rungs are a
matched 2×2 and the four cells of the `(n,m)` table at `n = 3` are now all occupied.

---

## 1. THE DERIVATION

Rung 69 § 1, unchanged: each lagged law solves its own constraint for its own actuator, so every
row of the actuator block is a multiple of **its own** constraint's gradient,

    row_i(M) = −(1/c⁽ⁱ⁾_i)·∇c⁽ⁱ⁾ᵀ      ⇒      rank M = dim span{∇c⁽ⁱ⁾} =: m ,   zeros = n − m

With `T := Tt4`, `φ := φ_lp`, `ψ := M_i = T_c − 1/φ + v` and `σ := 1/φ²`,

    ∇ψ  =  σ·∇φ  +  e_v          ⇐   ψ_g = σφ_g,  ψ_q = σφ_q,  ψ_v = σφ_v + 1

### 1.1 `m = 3` **IS** rung 67's own non-degeneracy condition

`span{∇φ, ∇ψ} = span{∇φ, e_v}` **unconditionally** — the lever's own `+1` in `ψ_v` puts `e_v` in
the span whatever the plant does, so the valve+stator pair is *always* rank 2 (that is rung 69's
result, now seen as automatic rather than measured). The only question left is whether the
governor's gradient escapes that plane:

    ∇T ∈ span{∇φ, e_v}   ⟺   T_g φ_q = T_q φ_g   ⟺   pair_RC = 1

So the rank is not a new condition at all: **`m = 3` is `pair_RC ≠ 1`, which is rung 67's own,
and `m = 2` is rung 69's.** Gated by CONSTRUCTION rather than by measurement
(`test_the_rank_is_rung67s_own_non_degeneracy_condition`): forcing `T_q/T_g = φ_q/φ_g` on a
hand-built block returns `pair_RC = 1` and exactly one zero, at the same `n = 3`.

### 1.2 The six gains — and only ONE number here is new

    R_q = −T_q/T_g    R_v = −T_v/T_g
    C_g = −φ_g/φ_q    C_v = −φ_v/φ_q
    V_g = −ψ_g/ψ_v    V_q = −ψ_q/ψ_v

    pair_RC = R_q·C_g = (T_q φ_g)/(T_g φ_q)          ← rung 67's `P`, and rung 70's, UNCHANGED
    pair_CV = C_v·V_q = σφ_v/ψ_v                     ← rung 69's `k`, UNCHANGED
    pair_RV = R_v·V_g = (T_v ψ_g)/(T_g ψ_v)          ← the ONLY new number in this rung

**No pair is 1.** Rung 66's identity survived three times at rung 68, once at 69, once at 70 and
**zero times here** — it is a property of a SHARED constraint, and nothing is shared. And

    pair_RV(71)  =  pair_CV · pair_RV(70)        at an identical base point

because `ψ_g/ψ_v = (φ_g/φ_v)·(σφ_v/ψ_v)`. So even the new gain is the old one, scaled by rung
69's scalar.

### 1.3 BOTH cyclic products are redundant, and the determinant FACTORS

    x := R_q·C_v·V_g = −pair_RC · pair_CV
    y := R_v·C_g·V_q = −pair_RV                      exactly, at ANY base point

    det M = −1 + pair_RC + pair_RV + pair_CV + x + y
          = −1 + pair_RC + pair_CV − pair_RC·pair_CV          [pair_RV cancels against y]
          = **−(1 − pair_RC)·(1 − pair_CV)**

Independently: `det M = −det[∇T; ∇φ; ∇ψ]/(T_g φ_q ψ_v)`, and row-reducing `∇ψ − σ∇φ = e_v` gives
`det[·] = T_gφ_q − T_qφ_g`, so `det M = −(1−pair_RC)/ψ_v`, which is the same thing because
`1/ψ_v = 1 − pair_CV`. Both routes agree.

**This retires the cyclic product for good.** Rung 68 said *quote `x`*; rung 69 said *`x` flips
to `−k`*; rung 70 found it **blind to `pair_RV`**. Here `x` is a product of two other pairs and
`y` *is* `−pair_RV`, so the three **pairs** are the complete independent set and both cyclics are
re-expressions. **`pair_RV` is invisible to the cyclic product at rung 70 and to the determinant
here; `c1` is the only invariant that has ever seen it.** That is rung 68's *check what is
INDEPENDENT before quoting it* in its third shape.

### 1.4 The invariants — and which of them is a tautology

    c0 = det J = −(1 − pair_RC)(1 − pair_CV)/(τ_g τ_q τ_s)    ≠ 0 — FIRST TIME IN THE FAMILY
    c1 = Σ_{i<j} (1 − pair_ij)/(τ_i τ_j)                       all THREE terms alive
    c2 = tr J = −Σ 1/τ_i                                       the ODE's own diagonal

**`c1`'s closed form is a TAUTOLOGY and is reported, never gated.** For any matrix with `−1` on
the diagonal, `Σ(1−pair_ij)/(τ_iτ_j)` *is* the second invariant — the shipped `_invariants`
would be agreeing with itself. (Rung 67 gate 9 was retracted for exactly this, and rung 70 § 3.1
rewrote its own gate before shipping for the same reason.) **`c0`'s closed form is NOT** — it
uses four of the six gains and asserts the other two drop out. That is what § 4 gates.

### 1.5 The ring: rung 69's floor is the `c0 = 0` corner

With `c0 ≠ 0` there is no zero root, so all three roots share one fixed budget:

    λ₁ + λ₂ + λ₃ = −Σ 1/τ_i        ⇒     Re(pair) = −( Σ 1/τ_i − |λ₃| ) / 2

At rung 69 the third root **was** the zero, so the pair took the whole budget (`Re = −Σ/2`) and
`ζ ≥ 1/√(1−k)` followed by AM–GM. Here the third loop's own pole **drains** it, so rung 69's
bound is not derived for this plant — and § 5 measures that it does not hold.

**Routh replaces it.** With `a,b,c = 1/τ_{g,q,s}` and `u,w,z = 1 − pair_{RC,RV,CV}`, the
characteristic polynomial is `λ³ + (a+b+c)λ² + (u·ab + w·ac + z·bc)λ + u·z·abc`, and

    A₂A₁ − A₀ = u a²b + u ab² + w a²c + w ac² + z b²c + z bc² + (u + w + z − u·z)·abc

Six unconditionally positive terms when `u, w, z > 0`, so

> **`u + w + z ≥ u·z` is SUFFICIENT for stability at EVERY bandwidth triple** — the first
> non-trivial stability condition this family has had. At `m < n` a zero root plus a negative
> trace made it automatic.

---

## 2. MEASURED — THE CONTAINMENT, WHICH IS THE RUNG

`Tt4_max = 1200 K`, `φ_lim = 0.80`, `b_max = 0.10`, `v_max = 0.20`, `m_lim = T_c − 1/φ_lim`.

### 2.1 The windows, and what collapsed

`τ = (0.05, 0.05, 0.05)`, `ds = 0.005`:

| leg | window in `s` | points | rung 70's |
|---|---|---|---|
| governor | 0.105 … 1.700 | 320 | 318 |
| valve | 0.000 … 0.590 | 119 | 119 |
| **stator** | **0.005 … 0.135** | **27** | **83** |
| **joint** | **0.105 … 0.135** | **7** (2.05 %) | **61** (17.9 %) |

The governor's and the valve's windows are essentially rung 70's. **The stator's is what
collapsed**, and not for a numerical reason.

### 2.2 WHY — derivable, zero new constants

The valve holds `φ_lp ≥ φ_lim`; the incidence band is `v ∈ [0, v_max]` (rung 69 § 0.1: `M_i` is
INCREASING in `v`). At the valve's own set point,

    φ = φ_lim   ⇒   M_i = T_c − 1/φ_lim + v = m_lim + v   ≥   m_lim     for every v ≥ 0

    ⇒   {φ ≥ φ_lim} ∩ {v ≥ 0}   ⊆   {M_i ≥ m_lim}

so the incidence loop can only ride where the valve is **failing** — inside its lag or its
saturation. Written as a slack, `M_i − m_lim − v = 1/φ_lim − 1/φ`, which is `≥ 0` exactly when
the valve delivers and `= 0` exactly when it pins `φ` on the floor. **The bound is tight and
needs no tolerance**, which is why the gate asserts equality rather than an epsilon:

| | measured |
|---|---|
| points where the valve delivers (`φ ≥ φ_lim`) | **307** of 341 |
| of those, points where the stator RIDES | **0** |
| worst `slack − v` over them | **exactly 0.0** |
| min slack over the WHOLE march | −7.449e−3 (so the wall *is* violated, where the valve fails) |

### 2.3 The window law — swept from BOTH sides

A one-sided sweep could not separate the mechanism from "a slower loop rides longer", so both
clocks are moved (`ds = 0.005`; the stator's right edge in `s`):

| `τ_q` | 0.005 | 0.05 | 0.20 | 0.50 | 2.00 | | `τ_s` | 0.005 | 0.05 | 0.20 | 0.50 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| edge | 0.115 | 0.135 | 0.210 | 0.280 | **0.365** | | edge | 0.160 | 0.135 | 0.155 | 0.170 |

**Monotone over 3.17× in the VALVE's clock; a 1.26× NON-monotone band in its own.** The third
loop's window is the second loop's lag, and its own bandwidth is not even a monotone influence.

### 2.4 The two edges are NOT the same number, and that is stated rather than fudged

`_solve_v` tests dormancy on the **counterfactual** plant at `v = 0`, so the loop quits while the
*marched* `φ` is still below the floor by exactly its own contribution. Measured at the dormancy
point: `φ_lp = 0.792807`, short of `φ_lim` by **7.193e−3**, against `|dφ/dv|·v = 0.423 × 0.016381
= 6.93e−3` — the shortfall is the stator's own channel and nothing else. The **exact** statement
of the containment is § 2.2's, which needs no counterfactual at all.

---

## 3. THE GAINS — three pairs, zero identities, two inherited

`ds = 0.002`, `every = 2`, base points on `φ = φ_lim` (rung 69's `_manifold_v`, unchanged).
8 interior rows over `s ∈ [0.108, 0.138]`; one sampled point dropped at `s = 0.104` where the
governor sits on its own dormant kink (**disclosed** — a dropped point is a coverage claim).

| `s` | `pair_RC` | `pair_RV` | `pair_CV` | `det M` | `−(1−RC)(1−CV)` |
|---|---|---|---|---|---|
| 0.108 | −0.019479 | −0.225851 | −1.802232 | −2.858536 | −2.856818 |
| 0.116 | −0.019723 | −0.225359 | −1.812612 | −2.869394 | −2.868085 |
| 0.124 | −0.019972 | −0.224783 | −1.822136 | −2.879350 | −2.878499 |
| 0.136 | −0.020318 | −0.223930 | −1.835225 | −2.892997 | −2.892832 |

* **No pair is 1**: the closest any of the three comes is **1.0195** away. For the first time in
  this family rung 66's identity appears zero times.
* `pair_RC ∈ [−0.0203, −0.0195]` — rung 67's `P` and rung 70's, on the same closures.
* `pair_CV ∈ [−1.835, −1.802]` — rung 69's `k`, inside its published `−1.66 … −2.01` band.
* `|y + pair_RV| ≤ 1.49e−3` (0.66 % of `pair_RV`) and `|x + pair_RC·pair_CV| ≤ 2.33e−4` — the
  two cyclic products, redundant.
* `|det − (−(1−RC)(1−CV))| ≤ 1.72e−3` against a determinant of magnitude 2.86, i.e. **0.06 %**.
* `pair_RV(71) / (pair_CV · pair_RV(70))` — read on rung 70's own rig at the **identical** base
  points — departs by at most **0.66 %**, the same differencing floor.

### 3.1 The two controls are DIFFERENT KINDS

`pair_RC` is a **numerical** control: rows R and C are the same shipped closures rungs 67 and 70
used, so it is nearly the same computation and is read against a genuinely separate rung-67 march
(`cross_identity` on a stator-free rig), reported as a ratio. `pair_CV` is a **functional-form**
control: it *is* rung 69's `k` on rung 69's own two loops, but re-measured on a different
trajectory, so its FORM and BAND are gated and no tolerance the trajectory shift cannot justify
is. Conflating the two would be the error rung 70 § 1.1 warns about, one rung on.

### 3.2 The state boundary, asserted at every sampled point

`R_q ≠ 0` and `R_v ≠ 0` only because the governor senses `Tt4` on the machine as the other two
actuators actually are. Rung 70's `_assert_state_boundary` — which builds the broken version on
purpose and requires it to read exactly zero — runs at every sampled point here, inherited
unchanged. Drop the boundary and the odd loop decouples, `m` reads 2 by accident, and this rung
would "confirm" rung 69.

---

## 4. THE SPECTRUM — zero zeros, `det J` alive, Routh non-trivial

`ds = 0.002`, `every = 4`, **ten** clock arms, `(τ_g, τ_q, τ_s)`. `full_modes` **defaults to
six** — the smallest grid that still spans § 5's three ring regimes, against the four rungs
68/69/70 default to; a march at this `ds` is the cost here, so arms are not free. The four extra
rows below are that reader called with a wider `clocks`, and every one is reproducible.

| `(τ_g, τ_q, τ_s)` | zeros | min `|λ|`/Σ | `ζ` | points below rung 69's line |
|---|---|---|---|---|
| (0.05, 0.05, 0.05) | **0** | 0.303 | 0.5861 … 0.5890 | **4 / 4** |
| (0.005, 0.05, 0.05) | **0** | 0.143 | 0.6687 … 0.6695 | 0 / 2 |
| (0.50, 0.05, 0.05) | **0** | 0.048 | 0.5839 … 0.5940 | **9 / 9** |
| (0.05, 0.02, 0.10) | **0** | 0.237 | 0.7846 … 0.7859 | 0 / 2 |
| (0.05, 0.005, 0.05) | **0** | 0.084 | — **all real** | 0 / 2 |
| (2.00, 0.05, 0.05) | **0** | 0.013 | 0.5812 … 0.5960 | **12 / 12** |
| (0.05, 0.05, 2.00) | **0** | 0.036 | 0.9867 … 0.9877 | 0 / 10 |
| (0.10, 0.10, 0.05) | **0** | 0.228 | 0.6068 … 0.6142 | 0 / 5 |
| (0.02, 0.20, 0.05) | **0** | 0.257 | 0.9160 … 0.9238 | 0 / 5 |
| (0.20, 0.02, 0.05) | **0** | 0.068 | 0.6526 … 0.6578 | 0 / 5 |

**Where an arm breaches, it breaches at EVERY sampled point on it** (4/4, 9/9, 12/12), which is
what a structural effect looks like against a per-point one.

* **`zeros = 0` on every arm** — the last unoccupied cell, and the first plant in this family
  whose actuator block is invertible. The smallest root is never a near-miss: the tightest arm
  puts it at 1.3 % of the rate sum, which is `~10²` above the `1e−4·Σ` test.
* **`c0` matches `−(1−pair_RC)(1−pair_CV)/Πτ` to 1.5e−3 relative**, worst over all arms. That
  closed form ignores `R_v` and `V_g` entirely, which is the claim.
* **Routh margin `u + w + z − u·z ≥ 2.185`** on every sampled point, and every spectrum is
  measured stable arm by arm rather than inferred from the certificate.
* `max |λ|/Σ(1/τ) = 0.849`, so the inherited RK4 constant stays conservative for a **third**
  reason (§ 9).

### 4.1 The damping reader had to be rebuilt — for the THIRD time in four rungs

| rung | reader | exact when |
|---|---|---|
| 69 | `−Re(dom)/|dom|` | the dominant root is a complex pair; returns exactly 1.0 for any real root |
| 70 | both NON-ZERO roots, magnitude-sorted | exactly one root is zero |
| **71** | **the pair identified by its IMAGINARY PART**, `None` when there is none | always |

Here no root is zero *and* the pair is not always the two largest, so magnitude ordering can drop
a pair **member** and keep the odd real root. Measured against rung 70's reader over a 12-arm
grid it disagrees on **four**: 0.960 vs 0.686, 1.279 vs 0.670, 1.045 vs 0.924, and 1.035 on an
arm whose spectrum is entirely real. **A reader that returns a number where there is no ring is
worse than one that returns nothing**, so this one returns `None` and callers report the count.
Gated on constructed spectra, so it does not depend on the plant.

---

## 5. THE RING — rung 69's floor does not survive, and the mechanism says why

Nine of the ten arms ring; one is entirely real. **Three arms land BELOW `1/√(1−pair_CV)`**, the
bound rung 69 proved bandwidth-independent on its own plant — deepest breach **1.313 %** at
matched clocks (`ζ = 0.588974` against that point's own floor `0.596811`), with the roots
satisfying the characteristic polynomial to 1e−15.

That is § 1.5's trace budget, visible. At matched clocks rung 69's spectrum was
`[0, −30 ± 35.5i]` — the zero took nothing, so the pair carried the whole `−Σ1/τ = −60`. Here
it is `[−18.169, −20.915 ± 28.699i]`: **the third pole took 18.2 of the same 60, and the pair's
real part fell from 30 to 20.9 for it.**

> **RUNG 69's FLOOR WAS A PROPERTY OF THE RANK DEFICIENCY, NOT OF `k`.** The same lever, the same
> two walls, the same `k ≈ −1.8` — and filling the null space makes it ring harder than rung 69's
> bound allows.

The honest replacement is a **removal plus a certificate**, not a new bound: the grid shows all
three regimes (below the old line, above it, and no complex pair at all), which is what "the
bound is removed" has to look like, while § 1.5's Routh condition holds everywhere with margin.
A single monotone trend, or a floor that survived, would have refuted the mechanism.

---

## 6. THE INITIAL CONDITION — a POINT, and rung 69 § 6 completed

Rungs 68/69/70 all carry a null space, so their `s = 0` fixed points are a one-parameter FAMILY
and a Gauss-Seidel sweep lands on whichever member its ORDER selects. Rung 69 § 6 measured the IC
spread *growing* as the nullity fell (rung 66 ±20 % at nullity 1, rung 68 45.2 % at 2, rung 69
187.0 % at 1) and called a null space a **shock absorber**. At nullity **zero** there is nothing
to absorb with.

`ic_contraction` runs the sweep itself from the three shipped laws — **nothing pinned**, because
`_stator_march`'s `b0`/`v0` arguments *hold* their actuator and a march started off the fixed
point could never reject a displacement by construction. 6 orders × 4 displaced starts = 24 runs,
per plant:

| plant | converged | limit points | spread `(g, q, v)` | iterations |
|---|---|---|---|---|
| **rung 71 (`n = m = 3`)** | 24/24 | **1** | **(0.0, 0.0, 0.0) exactly** | ≤ 3 |
| rung 70 (`n = 3, m = 2`) — the CONTROL | 24/24 | **4** | (0.0, 3.66e−2, 6.43e−2) | ≤ 2 |

Rung 70's plant is run on the same rig as the negative control: its valve and stator share `φ`,
so `|C_v V_q| = 1` exactly and its sweep is marginal by construction. A contraction here not
matched by a failure to contract there would be measuring the solver.

> **THE SWEEP REJECTS A MOVED START INSTEAD OF ABSORBING OR AMPLIFYING IT.** Rung 69's shock
> absorber has its zero-nullity end, and it is a hard one: the spread is not small, it is
> **identically zero**.

---

## 7. THE LEDGER — three currencies, and rung 70 § 5 CORRECTED

`I` = rung 66's `∫max(0, φ_lim − φ_lp) ds`; `E` = rung 67's `∫max(0, Tt4 − Tt4_max) ds`;
`M` = rung 68's `∫max(0, m_lim − M_i) ds`. All three inherited unchanged. **The first table in
this family that needs one currency per loop.**

| cell | `I` (`φ`) | `E` (`Tt4`) | `M` (`M_i`) | min `φ_lp` | max `Tt4` |
|---|---|---|---|---|---|
| bare | 2.5815e−2 | 109.949 | 4.3291e−2 | 0.735442 | 1695.41 |
| G | 2.0437e−2 | **27.466** | 3.3819e−2 | 0.742994 | 1279.18 |
| V | 1.9394e−3 | 117.011 | **3.0627e−3** | 0.789123 | 1717.46 |
| S | 5.5467e−2 | 131.659 | 1.0356e−2 | 0.673049 | 1725.64 |
| GV | 1.4083e−3 | 28.564 | 2.2249e−3 | 0.789123 | 1281.51 |
| GS | 4.6600e−2 | 29.734 | 3.3743e−3 | 0.684856 | 1283.74 |
| VS | 2.0170e−3 | 117.224 | 4.1660e−4 | 0.785043 | 1716.07 |
| **GVS** | **1.6189e−3** | **28.542** | **4.1660e−4** | 0.785043 | 1280.91 |

**Six of the eight cells are bit-identical to their ancestors' published tables** — every cell
without an incidence stator is a rung-70 march (`bare`, `G`, `V`, `GV`), every cell without a
governor is a rung-69 one (`S`, `VS`). Only `GS` and `GVS` are new. That is a free
differenceability check (rung 63's lesson) and it is gated, because a drift in a cell that
*cannot* have one would mean the rigs are not comparable.

### 7.1 The correction

Each loop's MARGINAL contribution to the full triple, in **its own** currency, against its SOLO
one:

| loop | own wall | marginal | alone | **kept** |
|---|---|---|---|---|
| governor | `Tt4` | 88.68 | 82.48 | **107.5 %** |
| valve | `φ` | 4.498e−2 | 2.388e−2 | **188.4 %** |
| **stator** | `M_i` | **1.808e−3** | **3.294e−2** | **5.5 %** |

**No two loops share a constraint, and the stator is eroded to 5.5 % anyway.** Rung 70 § 5's law
is refuted as stated. The correction is § 2's mechanism, integrated:

> **A LOOP IS ERODED BY ANY LOOP THAT PUSHES ITS CONSTRAINT INTO THE SLACK REGION — not only by
> the loops it SHARES a constraint with.** Rung 70's channel is `∇`-sharing (a statement about
> the Jacobian); this one is set containment (a statement about the feasible sets). Rung 70 could
> not see it because none of its loops could satisfy another's wall on its behalf.

**And the two `kept` figures above 1 are a confound, recorded rather than explained away** (rung
58's *check the SUM, not the term*). The stator running alone **degrades** `φ` below the bare
march — `min φ_lp = 0.673` against 0.735, which is rung 69 § 4's own finding — so the valve's
"188 %" is repair work, not protection. `full_bill` returns a `degrades` map naming every cell
whose currency is worse than bare (`S`: `I` and `E`; `GS`: `I`; `V`, `VS`: `E`), so a ratio above
1 is read as what it is.

### 7.2 The sharpest single number

    incidence credit, VALVE alone   =  92.9 %
    incidence credit, STATOR alone  =  76.1 %

**The loop that cannot see `M_i` at all protects it better than the loop that watches it** —
because holding `φ` on its floor implies the incidence wall with margin `v`, while the reverse
is not true. § 2's containment, in the ledger. (Both figures are bit-identical to rung 69 § 4's
own `V` and `S` cells, which carry neither governor nor fuel leg.)

Rung 70's opposite-sign cross-credit survives the coordinate change: both airflow loops **debit**
the temperature (`V`: 117.0 and `S`: 131.7 against a bare 109.9) while the governor **credits**
the surge margin (`G`: 2.04e−2 against 2.58e−2).

---

## 8. Reduce contract

| arm | reaches | how |
|---|---|---|
| `tau_gov=None` | **rung 69**, bit-for-bit | dispatch |
| `stator_lim` armed instead of `stator_inc` | **rung 70**, bit-for-bit | dispatch |
| no stator armed | **rung 67**, bit-for-bit | dispatch (the parent's own) |
| neither | rungs 66/65/64/62, bit-for-bit | dispatch |

Verified over 341 points on 9 recorded keys, worst difference **exactly 0.0**.
`at_lever` returns `FullSplitTransient` — the **ninth** instance of the trap rungs 61–70 each hit.

### 8.1 THE MARCH IS REUSED, NOT COPIED — and that is gated

Rungs 68, 69 and 70 each shipped a sibling integrator, and each had a reason: a state was being
**added**, and `tests/test_numeric_fingerprint.py` is the project's only absolute gate, so the
paths had to be kept apart. **Nothing is added here.** Rung 69 already made the five seams this
rung needs overridable — `_stator_leg`, `_clamp_v`, `_check_v0`, `_manifold_v`, `_solve_v` — each
the *identity* of what it replaced, so rung 70's `_integrate_fuel_cross_triple` runs this plant
unchanged and the only thing rung 71 removes is rung 70's own refusal to enter it. A copy would
be 130 lines that could not differ.

The reuse is asserted (`_integrate_fuel_cross_triple` is not in this class's `__dict__`, and is
the parent's function object) rather than argued, because the fingerprint gate does not watch
this path.

### 8.2 The refusals

* **`n = 4, m = 3`** — rung 52's fuel leg beside the governor: four loops, two on one actuator.
  Rung 70's own named seam; still asserted against rather than run.
* **`tau_gov` without `Tt4_max`** — would march as rung 69 while every reader reported rung 71.
* **Rungs 50/51's forced release edges** — refused twice over, and the outer refusal is
  structural: `_stator_march`, the entry every reader in this family actually calls, does not
  plumb `s_off`/`tau_rel` through at all, so they cannot reach a march on this ladder even by
  mistake. The assert in `integrate_fuel` is the inner guard for a caller that goes around it.
* **An instantaneous valve beside a lagged stator** — rung 65 called that limit singular.
* **`ds = 0.05`** — the RK4 guard, which rung 68 measured as counterfeiting *perfect protection*
  when violated.

---

## 9. Predictions, scored

The anchor discloses its own order: § 0's kill-check ran **before** the anchor existed, because
the seam could have been infeasible. § 2A lists what was DERIVED on paper and then met in that
check — **not scored**; § 2B is what was open at the time of writing.

### 9.1 § 2A — derived, then confirmed (listed, not scored)

| | statement | measured |
|---|---|---|
| D1 | `zeros = 0` on every arm | held; min `|λ|/Σ` = 0.013 |
| D2 | `c0 = −(1−pair_RC)(1−pair_CV)/Πτ`, blind to `pair_RV` | 1.5e−3 rel |
| D3 | `y + pair_RV = 0`, `x + pair_RC·pair_CV = 0` | ≤ 1.49e−3 / ≤ 2.33e−4 |
| D4 | no pair is 1; `pair_RC` = rung 67's `P`, `pair_CV` = rung 69's `k` | −0.0195, −1.80 |
| D5 | the Routh certificate holds with margin | ≥ 2.185 |
| D6 | rung 69's floor is not a bound here | breached on 3 of 10 arms, deepest 1.31 % |

### 9.2 § 2B — scored

| | prediction | outcome |
|---|---|---|
| **P1** | the `s = 0` fixed point becomes UNIQUE; the sweep REJECTS a moved start | **HIT, and exactly** — 1 limit point from 24 (order, start) pairs, spread **identically 0.0**, against the control's 4 points and 6.4e−2 |
| **P2** | ZERO erosion — every loop keeps ~100 % of its solo credit | **REFUTED, and the refutation is the rung's best moment.** The governor keeps 107.5 %, but the stator keeps **5.5 %** while sharing nothing. **Rung 70 § 5's law gains a second channel** (§ 7.1) — set containment, not gradient sharing |
| **P3** | three currencies; the stator's column near-empty in absolute terms | **HIT on the column** (marginal `M` = 1.8e−3 against a bare 4.3e−2), **MOOT on its framing** — it was registered as the reconciliation of P2's "100 %" with a small credit, and P2 did not hold |
| **P4** | `pair_RV(71) = pair_CV · pair_RV(70)` at identical base points | **HIT** — ≤ 0.66 %, the instrument's own differencing floor |
| **P5** | `m = 3` is `pair_RC ≠ 1`, testable by CONSTRUCTION | **HIT** — forcing `∇T` into the plane returns rank 2 with exactly one zero at the same `n = 3` |
| **P6** | the window is monotone in `τ_q` and comparatively flat in `τ_s` | **HIT, and SHARPENED** — 3.17× monotone against a 1.26× band that is not even monotone |
| **P7** | all three ring regimes present; no floor survives | **HIT** — 3 arms below rung 69's line, 6 above, 1 with no complex pair |
| **P8** | four reduce arms bit-for-bit; the march REUSED not copied | **HIT** — worst difference 0.0 on 9 keys × 341 points; the integrator is the parent's own function object |
| **P9** | the refusals hold | **HIT, with one CORRECTION**: the forced-release refusal is unreachable through `_stator_march` at all, so `integrate_fuel`'s assert is a second guard rather than the only one (§ 8.2) |

P2's refutation is the rung's best moment for the same reason rung 70's P8 was: it converted a
predicted null into a **mechanism** — and the mechanism turned out to be the *same* one § 2
derived for the window, seen integrally instead of dynamically. The anchor is **not** edited.

---

## 10. Concessions

* Every one rungs 62–70 list, all inherited.
* **THE JOINT WINDOW IS 2.05 % OF THE MARCH** (7 points at `ds = 0.005`, 6 interior). That is
  the rung's own subject rather than an accident, but every gain table here is a reading over 30
  units of `s` in 1700 and is quoted as such. The tables run at `ds = 0.002` (16 interior points,
  8 sampled) and § 2.3's slow-valve arm carries 47 interior points as the wide-window reading.
* `Tt4_max = 1200 K` is **rung 67's imposed value**, taken verbatim so the numbers difference
  against rungs 67 and 70. Lowering it to 1150 K widens the joint window to 8 interior points;
  **disclosed and not adopted** (rung 63's lesson).
* `φ_lim`, `b_max` (rung 64) and `v_max = 0.20` (rungs 57/58) remain **imposed**; `m_lim` adds no
  constant (rung 69 § 10, verbatim) — it is `T_c − 1/φ_lim` at the design setting.
* **The base point is rung 69's `_manifold_v` (`φ = φ_lim`) and it carries rung 69 § 1.2's
  disclosure verbatim: it sits at `v < 0`, OUTSIDE the incidence loop's own band.** At `n = m` no
  identity *needs* a manifold — that is itself a finding — so the choice is made for
  differenceability against rungs 67/69/70 and not for exactness. Stated, not implied.
* **The determinant's FACTORING is CONTINGENT on `∇ψ = σ∇φ + e_v`** (rung 69 § 1.1's structure).
  A generic third constraint leaves `x` and `y` independent and `det M` would not factor. Gated
  as a condition beside its consequence (rung 70 § 4.1's form).
* `pair_CV` is compared against rung 69's *published band* rather than to a tolerance, because it
  is a functional-form control re-measured on a different trajectory (§ 3.1).
* All three clocks are swept coordinates on the march's own `s`. **No actuator bandwidth is
  anchored anywhere in this family.** ORDERINGS, SIGNS and INVARIANCES are the claims; every
  MAGNITUDE is disclaimed.
* The RK4 guard's constant survives a **fourth** time on a **third** argument (rung 68: rank one,
  `λ = −Σ1/τ`; rung 69: a complex pair bounded by `√(1−k)/2·Σ`; rung 70: `min(pair) ≈ 0`, back on
  the real axis). Here there is no zero root at all, so the trace is shared three ways and the
  dominant root sits strictly below the sum: measured `max|λ|/Σ = 0.849`.
* The spectrum is sampled at finitely many trajectory points — a diagnostic that can miss a brief
  excursion (rung 65's retracted trap), not a proof of convergence.
* § 5's deepest breach of rung 69's line is **1.31 %**. It is systematic — every sampled point on
  a breaching arm breaches — and far above the root solver's 1e−15 residual, but it is a small
  number and is reported as a **sign**, which is all § 1.5 derives.
* The STAGE STACK (rungs 55/56) is still off the transient ladder.
* This still does **not** close rung 63's *fuel + bleed + STATOR* seam: that seam wants the stator
  as an OPEN-loop **schedule**, and this is a closed loop.

---

## 11. Next seams

* **`n = 4`, and it is now the ONLY unoccupied shape at this size.** Rungs 68–71 fill every
  `(3, m)` cell. The two `n = 4` plants both need something this one does not have: rung 52's
  fuel leg beside the governor is `(4, 3)` with **two loops on ONE actuator**, which tests
  whether § 1's `m` counts constraints or *actuators* — § 1 assumes one law per actuator
  throughout — and rung 69 § 11's `(4, 2)` needs a fourth LP lever this plant does not have.
* **A plant where the third constraint is NOT contained in the second.** § 2's containment is
  what makes this rung's window thin, and it follows from `M_i = m_lim + v` at the valve's set
  point — i.e. from the two walls being *matched at the design setting* (rung 69 § 10). A
  DELIBERATELY OFFSET `m_lim` would break the containment and give the third loop a real window
  — at the cost of confounding the coordinate split with a set-point offset, which is exactly
  what rung 69 refused. **That trade is the seam**, and it is the sharpest one here.
* **A generic third constraint**, i.e. one where `∇ψ` is not `σ∇φ + e_v`. Then `x` and `y` are
  independent, `det M` does not factor, and § 1.3's *one factor per rung* would have to be
  re-derived rather than re-read. It is the direct test of whether that identity is contingent
  (as claimed) or structural.
* **A plant that VIOLATES the Routh certificate** — `u + w + z < u·z` needs a pair far more
  degenerate than anything measured here (`u z ≈ 2.9` against `u + w + z ≈ 5.1`). It would be the
  first UNSTABLE actuator block in this family, and § 1.5 says it can only happen at full rank.
* **An ASYMMETRIC valve** (rung 65) and an **asymmetric governor** (rung 67) — both still open;
  all three lags here are symmetric.
* **Fuel + bleed + STATOR-as-a-SCHEDULE** — rung 63's seam, still open after 64–71.
* Everything rung 68 § 10 left: a plant with `|P| > 1`, and the real spatial/transported-CFD PDF.
