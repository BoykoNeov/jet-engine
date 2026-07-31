# Rung 71 anchor — THE FULL SPLIT (`n = m = 3`, the last unoccupied cell)

Scored in `docs/rung71-spec.md` § 9. The rule this project runs under: **a prediction that is
edited after the measurement is not a prediction.**

**AND THIS ANCHOR DISCLOSES ITS OWN ORDER, because it is not rungs 68–70's.** Rung 70's seam
could have been *infeasible*: rung 69 § 0.3 measured its incidence loop saturating over 84 % of
the ramp when it runs alone, and rung 70's own joint window was already down to 17.9 %. A gain
table over an empty intersection reports the pairwise algebra of loops that were never
simultaneously live, so **§ 0's kill-check was run BEFORE this document existed** — rung 69's
own order (its anchor § 0 likewise carries measured characterization, § 2 the predictions).

§ 2 is therefore split. **§ 2A is DERIVED-THEN-CONFIRMED** — worked out on paper before any code
and then met in the kill-check, so it is listed as derivation and **is not scored as
prediction**. **§ 2B is genuinely open** at the time of writing and is what § 9 scores.

---

## 0. THE KILL-CHECK — the window, and the thing it found

Rung 70's plant with the stator's REFERENCE moved (rung 69's move, applied to rung 70's
plant). Rung 47's `Tt4` governor, rung 65's `φ` valve, rung 69's `M_i` incidence stator:

    dg/ds = ( R(ν,q,v) − g ) / τ_g    R = rung 47's clip,  Tt4 ≤ Tt4_max   [GOVERNOR, Tt4]
    dq/ds = ( C(ν,g,v) − q ) / τ_q    C = rung 65's b_cmd, φ_lp ≥ φ_lim    [VALVE,    φ]
    dv/ds = ( V(ν,g,q) − v ) / τ_s    V = rung 69's v_cmd, M_i ≥ m_lim     [STATOR,   M_i]

Three loops, **three constraints**, `n = m = 3` ⇒ **zeros = 0** — the one cell of rung 69 § 1's
table this ladder has never occupied, and the seam rung 70 § 6.1/§ 9 named as its strongest.

### 0.1 The windows, measured first (`ds = 0.005`, `τ = (0.05, 0.05, 0.05)`)

| leg | window in `s` | points |
|---|---|---|
| governor | 0.105 … 1.700 | 320 |
| valve | 0.000 … 0.590 | 119 |
| **stator** | **0.005 … 0.135** | **27** |
| **joint** | **0.105 … 0.135** | **7** (2.05 % of the march) |

Against rung 70's 61 joint points / 17.9 %. **The stator's own window is what collapsed**, from
83 points to 27, and it did not collapse for a numerical reason.

### 0.2 WHY — and it is derivable with zero new constants

The valve holds `φ_lp ≥ φ_lim`. The incidence loop's band is `v ∈ [0, v_max]` (rung 69 § 0.1:
`M_i` is INCREASING in `v`). At the valve's own set point,

    φ = φ_lim   ⇒   M_i = T_c − 1/φ_lim + v = m_lim + v   ≥   m_lim    for every v ≥ 0

so **{φ ≥ φ_lim} ∩ {v ≥ 0} ⊆ {M_i ≥ m_lim}**: the third constraint is IMPLIED by the second's
on the whole admissible band. The incidence loop can only ride where the valve is *failing* —
i.e. inside the valve's own LAG (or its saturation). Measured on the trajectory: `v_cmd` climbs
while `b < b_cmd`, peaks at `s ≈ 0.05`, and hits its dormant stop at `s = 0.140`, which is
exactly where `φ_lp` recovers through `φ_lim = 0.80`.

**And the valve's clock moves the window, monotonically** (`ds = 0.005`):

| `τ_q` | 0.005 | 0.05 | 0.20 | 0.50 | 2.00 |
|---|---|---|---|---|---|
| stator riding ends at `s` | 0.115 | 0.135 | 0.210 | 0.280 | 0.365 |
| joint interior points | 2 | 6 | 5 | 9 | **17** |

### 0.3 What that buys, and what it costs

* The joint window is **thin but not empty**, and it refines: 6 interior points at `ds = 0.005`,
  16 at `0.002`, 32 at `0.001`. Every gain table here runs at `ds = 0.002`, and the **slow-valve
  arm (`τ_q = 2.0`) is carried beside it** as the wide-window reading — it is the same plant with
  one swept clock moved, and it is where the third loop is genuinely live.
* `Tt4_max = 1200 K` stays **rung 67's imposed value, verbatim**. Lowering it to 1150 K widens
  the joint window to 8 points; that is disclosed in the spec and **not adopted**, because it
  would break differenceability against rungs 67 and 70 (rung 63's lesson).

### 0.4 The instrument that had to be rebuilt — for the THIRD time in four rungs

Rung 70's `_zeta_pair` sorts the roots by magnitude and drops the smallest, which is exact when
exactly one root is ZERO. **Here no root is zero**, and the complex pair is not always the two
largest. Measured against a reader that identifies the pair by its IMAGINARY PART:

| `(τ_g, τ_q, τ_s)` | rung 70's reader | the ring | roots |
|---|---|---|---|
| (0.02, 0.05, 0.10) | 0.960 | **0.686** | −45.70, −17.15 ± 18.21i |
| (0.005, 0.05, 0.05) | 1.279 | **0.670** | −194.0, −22.99 ± 25.48i |
| (0.02, 0.20, 0.05) | 1.045 | **0.924** | −39.43, −17.78 ± 7.37i |
| (0.05, 0.005, 0.05) | 1.035 | **none — all real** | −20.15, −81.73, −138.1 |

Wrong on 4 of 12 arms. Rung 69's reader (`−Re(dom)/|dom|`) returns 1.0 for any real root; rung
70's assumes one zero root; **rung 71's must assume neither.** `_cubic_roots_c` itself is sound
and is inherited unchanged — checked, max `|poly(root)|` = 1.8e−14 over the same 12 arms.

---

## 1. THE DERIVATION (on paper, before any code)

Rung 69 § 1 unchanged: each lagged law solves its own constraint for its own actuator, so every
row of the actuator block is a multiple of **its own** constraint's gradient,

    row_i(M) = −(1/c⁽ⁱ⁾_i)·∇c⁽ⁱ⁾ᵀ      ⇒      rank M = dim span{∇c⁽ⁱ⁾} =: m ,   zeros = n − m

With `T := Tt4`, `φ := φ_lp`, `ψ := M_i = T_c − 1/φ + v`, and `σ := 1/φ²`:

    ∇ψ  =  σ·∇φ  +  e_v          ⇐  ψ_g = σφ_g,  ψ_q = σφ_q,  ψ_v = σφ_v + 1

### 1.1 `m = 3` ⟺ `pair_RC ≠ 1`

`span{∇φ, ∇ψ} = span{∇φ, e_v}` **unconditionally** — the lever's own `+1` in `ψ_v` is what puts
`e_v` in the span, and it is there whatever the plant does. So the valve+stator pair is *always*
rank 2, and the only question is whether the governor's gradient escapes that plane:

    ∇T ∈ span{∇φ, e_v}   ⟺   T_g φ_q = T_q φ_g   ⟺   pair_RC = 1

**So `m = 3` is exactly rung 67's own non-degeneracy condition**, and `m = 2` (the rank-2
valve+stator plane) is exactly rung 69's.

### 1.2 The six gains, and the three pairwise products

    R_q = −T_q/T_g    R_v = −T_v/T_g
    C_g = −φ_g/φ_q    C_v = −φ_v/φ_q
    V_g = −ψ_g/ψ_v    V_q = −ψ_q/ψ_v

    pair_RC = R_q·C_g = (T_q φ_g)/(T_g φ_q)      ← rung 67's `P`, and rung 70's, UNCHANGED
    pair_CV = C_v·V_q = (φ_v ψ_q)/(φ_q ψ_v) = σφ_v/ψ_v = k    ← rung 69's `k`, UNCHANGED
    pair_RV = R_v·V_g = (T_v ψ_g)/(T_g ψ_v)      ← the ONLY new number this rung contains

**NO pair is 1.** Rung 66's identity survived three times at rung 68, once at 69, once at 70,
and **zero times here** — it is a property of a SHARED constraint and nothing is shared.

Two of the three are inherited scalars: `pair_RC` is byte-identical to rung 70's (rows R and C
are the same closures), and `pair_CV` is rung 69's `k` on the same two loops. And, exactly,

    pair_RV(71)  =  pair_CV · pair_RV(70)         at an identical base point

because `ψ_g/ψ_v = (σφ_g)/ψ_v = (φ_g/φ_v)·(σφ_v/ψ_v)`.

### 1.3 BOTH cyclic products are REDUNDANT, and the determinant FACTORS

    x := R_q·C_v·V_g = −pair_RC · pair_CV
    y := R_v·C_g·V_q = −pair_RV                    (exactly, at any base point)

    det M = −1 + pair_RC + pair_RV + pair_CV + x + y
          = −1 + pair_RC + pair_CV − pair_RC·pair_CV          [pair_RV cancels against y]
          = **−(1 − pair_RC)·(1 − pair_CV)**

Independently: `det M = −det[∇T; ∇φ; ∇ψ]/(T_g φ_q ψ_v)`, and row-reducing `∇ψ − σ∇φ = e_v`
gives `det[·] = T_gφ_q − T_qφ_g`, so `det M = −(1−pair_RC)/ψ_v = −(1−pair_RC)(1−pair_CV)` since
`1/ψ_v = 1 − k`. Both routes agree.

> **THE FULL-RANK DETERMINANT IS THE TWO PRIOR RUNGS' NON-DEGENERACY CONDITIONS, MULTIPLIED —
> ONE FACTOR PER RUNG.** And it is **BLIND to `pair_RV`**, the one gain no earlier rung has
> measured.

**CONTINGENT, not structural.** It rests on `∇ψ = σ∇φ + e_v`, which is rung 69 § 1.1's factoring
structure. A generic third constraint would leave `x` and `y` independent and the determinant
would not factor. Gated as a condition beside its consequence, the way rung 70 § 4.1 gated its
own identity on `pair_RV > 0`.

### 1.4 The invariants, and which of them is a tautology

    c0 = det J = −(1 − pair_RC)(1 − pair_CV)/(τ_g τ_q τ_s)     ≠ 0 — THE FIRST TIME IN THE FAMILY
    c1 = Σ_{i<j} (1 − pair_ij)/(τ_i τ_j)                        all THREE terms alive
    c2 = tr J = −Σ 1/τ_i                                        the ODE's own diagonal

**`c1`'s closed form is a TAUTOLOGY here and must not be gated as a measurement.** For any
matrix with `−1` on the diagonal, `Σ(1−pair_ij)/(τ_iτ_j)` *is* the second invariant — the
shipped `_invariants` computes the same thing twice. (Rung 67 gate 9 was retracted for exactly
this; rung 70 § 3.1 rewrote its own gate before shipping for the same reason.) **`c0`'s closed
form is NOT**: it uses only four of the six gains, so it asserts that `R_v` and `V_g` drop out.
That is the claim, and it is what gets gated.

### 1.5 The ring — and rung 69's floor is the `c0 = 0` corner

With `c0 ≠ 0` there is no zero root, so all three roots share the fixed trace budget:

    λ₁ + λ₂ + λ₃ = −Σ 1/τ_i        ⇒     Re(pair) = −(Σ 1/τ_i − |λ₃|)/2

At rung 69 the third root **was** the zero, so the pair took the whole budget (`Re = −Σ/2`) and
`ζ ≥ 1/√(1−k)` followed by AM–GM. Here the third loop's own pole `λ₃` **drains** the budget, so
rung 69's bound is not derived for this plant and there is no reason to expect it to hold.

**Routh, in place of it.** With `a = 1/τ_g`, `b = 1/τ_q`, `c = 1/τ_s`, `u = 1−pair_RC`,
`w = 1−pair_RV`, `z = 1−pair_CV`, the characteristic polynomial is
`λ³ + (a+b+c)λ² + (u·ab + w·ac + z·bc)λ + u·z·abc`, and

    A₂A₁ − A₀ = u a²b + u ab² + w a²c + w ac² + z b²c + z bc² + (u + w + z − u·z)·abc

Six unconditionally positive terms when `u, w, z > 0`, so

> **`u + w + z ≥ u·z` is SUFFICIENT for stability at EVERY bandwidth triple** — the structural
> replacement for rung 69's bandwidth-independent floor, and the first non-trivial stability
> condition this family has had (at `m < n` a zero root plus a negative trace made it automatic).

---

## 2A. DERIVED THEN CONFIRMED — listed, NOT scored

Each of these follows from § 1 and was met in § 0's kill-check. They are stated so § 9 can
report them honestly, and they are **not** scored as predictions.

| | statement | kill-check |
|---|---|---|
| D1 | `zeros = 0` — no root within `1e−4·Σ1/τ` on any arm | held; min `\|λ\|/Σ` = 0.013 |
| D2 | `c0 = −(1−pair_RC)(1−pair_CV)/Πτ`, blind to `pair_RV` | held to 6.0e−4 rel |
| D3 | `y + pair_RV = 0` and `x + pair_RC·pair_CV = 0` | ≤ 1.5e−3 / ≤ 2.3e−4 |
| D4 | no pair is 1; `pair_RC` reproduces rung 67/70's `P`, `pair_CV` rung 69's `k` | −0.0195, −1.80 |
| D5 | the Routh certificate holds with margin | `u+w+z−u·z` ≈ 2.19 |
| D6 | rung 69's floor is not a bound here | ζ = 0.5895 vs 0.5974 at matched clocks |

---

## 2B. PRE-REGISTERED PREDICTIONS — scored in § 9

**P1 — THE INITIAL CONDITION BECOMES UNIQUE, and this is the sharpest one.** Rungs 68/69/70 all
carry a `n−m ≥ 1` null space, so their `s = 0` fixed points are a one-parameter FAMILY and a
Gauss-Seidel sweep lands on whichever member its ORDER selects. At `n = m` there is no null
space, so the fixed point should be a **POINT**: every sweep order and every displaced start
must converge to the SAME `(g, q, v)`. **Rung 69 § 6 found the IC spread GROWING as the nullity
fell (rung 66 ±20 % at nullity 1, rung 68 45.2 % at 2, rung 69 187.0 % at 1) and called a null
space a SHOCK ABSORBER. At nullity 0 the prediction is not "grows again" but "COLLAPSES" — the
solve REJECTS a moved start instead of absorbing or amplifying it.** A spread that does not
collapse would correct rung 69 § 6, which is also content.

**P2 — ZERO EROSION IN THE LEDGER.** Rung 70 § 5 measured that each `φ` loop keeps only a
fraction of its solo credit while the governor keeps ~100 % of its own, and stated the law: *a
loop is eroded by the loops it shares a constraint with, and by no others.* **Here no two loops
share, so every loop's marginal contribution should be ~100 % of its solo credit, in its own
currency.** This is that law's zero-sharing corner and its crispest falsifiable test. A failure
is a correction to rung 70 § 5.

**P3 — THE LEDGER NEEDS THREE CURRENCIES, and the stator's is nearly a null column.** Rung 70's
had two (`I` on `φ`, `E` on `Tt4`); this needs rung 68's `_violation_inc` as a third. Predicted:
the stator's *marginal* delivery on `M_i` is **small in absolute terms** despite being ~100 % of
its own solo credit, because § 0.2 confines it to the valve's lag — i.e. **P2 and a near-empty
column are not in conflict, and quoting either alone would mislead.**

**P4 — `pair_RV(71) = pair_CV · pair_RV(70)` at identical base points**, to the differencing
floor (§ 1.2). Measured by evaluating rung 70's `φ`-referenced rig at THIS march's points, the
way rung 69 differenced its two references on one trajectory.

**P5 — `m = 3` is `pair_RC ≠ 1` (§ 1.1), and it is testable by CONSTRUCTION, not only by
measurement.** A hand-built block with `∇T` forced into `span{∇φ, e_v}` must come back rank 2
with one zero, at the same `n = 3`. (Rung 69's `test_a_determinant_provably_cannot_see_a_split`
is the precedent for hand-building the counter-example rather than arguing it.)

**P6 — THE WINDOW LAW IS MONOTONE IN `τ_q` AND FLAT IN `τ_s`.** § 0.2 says the stator rides
inside the valve's lag, so the window's right edge should be a monotone increasing function of
`τ_q` and comparatively insensitive to the stator's own clock. § 0.1's table measured `τ_q`;
`τ_s` is **not yet measured** and is the prediction.

**P7 — THE RING IS ARM-DEPENDENT, WITH NO FLOOR.** § 1.5 removes rung 69's bound rather than
replacing it, so over a bandwidth grid the plant should show **all three regimes** — a ring below
rung 69's floor, a ring above it, and arms with **no complex pair at all**. A single monotone
trend, or a floor that survives, would refute § 1.5's trace-budget mechanism.

**P8 — REDUCE, FOUR ARMS, BIT-FOR-BIT BY DISPATCH.** `tau_gov=None` ⇒ rung 69; a `stator_lim`
armed instead of `stator_inc` ⇒ rung 70; no stator ⇒ rung 67; neither ⇒ 66/65/64/62. **And the
march itself is NOT duplicated** — rung 69 already made `_stator_leg`, `_clamp_v`, `_check_v0`,
`_manifold_v` and `_solve_v` overridable, each the identity of what it replaced, so rung 70's
five-state integrator runs this plant unchanged. The prediction is that reuse is exact: 341
points on 9 recorded keys, worst difference **0.0**, on every inherited arm.

**P9 — THE REFUSALS.** `n = 4` (rung 52's fuel leg beside the governor) refused, not run;
`tau_gov` without `Tt4_max` refused; rungs 50/51's forced release edges refused; `ds = 0.05`
caught by the RK4 guard.

---

## 3. SETTINGS, AND WHAT IS IMPOSED

* `Tt4_max = 1200 K` — rung 67's, **verbatim** (§ 0.3).
* `φ_lim`, `b_max` (rung 64), `v_max = 0.20` (rungs 57/58) — all IMPOSED, all inherited.
  `m_lim = T_c − 1/φ_lim` adds no constant (rung 69 § 10, verbatim): it is the SAME physical
  wall at the design setting, and rung 69's constructor assert enforces it.
* `τ_g, τ_q, τ_s` are swept coordinates on the march's own `s`. **No actuator bandwidth is
  anchored anywhere in this family.** ORDERINGS, SIGNS and INVARIANCES are the claims; every
  MAGNITUDE is disclaimed.
* The evaluation base point is rung 69's `_manifold_v` **unchanged** — `φ = φ_lim`. Two reasons,
  and they must both be stated: (i) at `n = m` there is **no shared constraint**, so no identity
  needs a manifold and the base point stops being load-bearing for exactness — a finding in
  itself; (ii) `pair_RC` and `pair_CV` are differenceable against rungs 67/70 and 69 only if
  read at the base point *those rungs* read them at. It carries rung 69 § 1.2's disclosure
  verbatim: the base sits at `v < 0`, **outside** the incidence loop's own band.
* RK4: the guard's constant survives a FOURTH time on a THIRD argument (rung 68: rank one,
  `λ = −Σ1/τ`; rung 69: a complex pair bounded by `√(1−k)/2·Σ`; rung 70: `min(pair) ≈ 0` back on
  the real axis). Here the dominant root is a complex pair with `|λ|/Σ` measured, not trusted.
