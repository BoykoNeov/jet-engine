# Rung 70 anchor — THE GENERIC SPLIT (three loops, two variables, unshared)

Pre-registered **before** any code was written or any number measured. Scored in
`docs/rung70-spec.md` § 9. The rule this project runs under: a prediction that is
edited after the measurement is not a prediction.

---

## 0. THE CONSTRUCTION — rung 67's substitution, applied to rung 68's triple

Rung 67 built cascade A by taking rung 66's pair and moving **one leg's SENSOR** from `φ` to
`Tt4`. Rung 70 does exactly that to rung 68's triple: rung 52's `φ`-referenced fuel leg is
replaced by **rung 47's `Tt4` topping governor**, beside rung 65's `φ` valve and rung 68's `φ`
stator, all three lagged.

    dg/ds = ( R(ν,q,v) − g ) / τ_g    R = rung 47's clip,  Tt4 ≤ Tt4_max   [GOVERNOR, Tt4]
    dq/ds = ( C(ν,g,v) − q ) / τ_q    C = rung 65's b_cmd, φ_lp ≥ φ_lim    [VALVE,    φ]
    dv/ds = ( V(ν,g,q) − v ) / τ_s    V = rung 68's v_cmd, φ_lp ≥ φ_lim    [STATOR,   φ]

Five states (`ν_lp, ν_hp, g, q, v`), three clocks — **the same shapes as rungs 68 and 69**.
One actuator per loop, as at rung 68; only the odd loop's *coordinate* differs.

**Why this rung exists.** It is the SAME seam from two sides, and closing it discharges both:

* **rung 68 § 10** — *three loops on TWO variables* (asserted against in `integrate_fuel`,
  never run).
* **rung 69 § 11** — *a plant with `pair_RV ≠ pair_CV`*, § 1.1's generic split, where the odd
  constraint does **not** depend on the shared actuators only through the shared constraint.

Rung 69 § 11 names them as one seam explicitly. Rung 70 is the run.

---

## 1. THE DERIVATION

Rung 69 § 1, unchanged: each lagged law solves its own constraint for its own actuator, so
every row of the actuator block is a multiple of **its own** constraint's gradient,

    row_i(M) = −(1/c⁽ⁱ⁾_i)·∇c⁽ⁱ⁾ᵀ      ⇒      rank M = dim span{∇c⁽ⁱ⁾} =: m ,   zeros = n − m

Write `T := Tt4` and `φ := φ_lp`. The three constraints are `T = Tt4_max` (governor) and
`φ = φ_lim` **twice** (valve, stator). So

    row_R = −(1/T_g)·∇Tᵀ ,      row_C = −(1/φ_q)·∇φᵀ ,      row_V = −(1/φ_v)·∇φᵀ

**Rows C and V are parallel; row R is not.** Hence `m = 2` and

    n = 3, m = 2   ⇒   ZEROS = 1        — the same (n,m) cell as rung 69

and that is the point: rung 70 is a **controlled comparison at equal counts**. What differs
from rung 69 is not the arithmetic of `n − m` but *which* pair shares, and whether the odd
constraint factors.

### 1.1 The six gains, and where the identity now lives

    R_q = −T_q/T_g     R_v = −T_v/T_g
    C_g = −φ_g/φ_q     C_v = −φ_v/φ_q
    V_g = −φ_g/φ_v     V_q = −φ_q/φ_v

    pair_CV = C_v·V_q = 1                     ← THE SHARED PAIR (rung 66's identity, relocated)
    pair_RC = R_q·C_g = (T_q φ_g)/(T_g φ_q)   ← SPLIT
    pair_RV = R_v·V_g = (T_v φ_g)/(T_g φ_v)   ← SPLIT

**Which pair keeps rung 66's identity is a direct read of which two loops share a constraint** —
rung 69's statement, and here it moves from `(R,C)` to `(C,V)`. A reader that inherited rung
69's `pair_RC = 1` negative control would be reading a **signal** as a control.

### 1.2 THE DISCRIMINATOR — the two split pairs are now genuinely DIFFERENT

    pair_RC / pair_RV = (T_q/φ_q)·(φ_v/T_v)      ⇒     equal  iff  T_q/φ_q = T_v/φ_v

i.e. **iff `Tt4` depends on `(q, v)` only through `φ`** — rung 69 § 1.1's condition, relabeled.
There it held *trivially*, because `M_i = T_c − 1/φ + v` differs from the shared wall by exactly
the lever's own direct channel, so both split pairs collapsed onto one scalar `k`. Here the odd
constraint sits on a **different lever entirely**: the bleed valve and the stator reach `Tt4`
and `φ_lp` through different channels, and there is no reason for the ratio to be 1.

**This is the measurement rung 69 could not make.** Its `pair_RV = pair_CV` was a *measurement
of the two walls' relationship*, not a restatement of the rank — and until a plant exists where
they differ, that sentence is untested.

### 1.3 THE CYCLIC PRODUCT GOES HALF-BLIND

    x := R_q·C_v·V_g = (−T_q/T_g)(−φ_v/φ_q)(−φ_g/φ_v) = −(T_q φ_g)/(T_g φ_q) = **−pair_RC**

At rung 69, `x = −k` summarised the whole split because both split pairs *were* `k`. Here `x`
reproduces **`pair_RC` alone and is structurally blind to `pair_RV`.** So rung 68's "quote `x`"
and rung 69's "`x` flips sign to `−k`" both stop being complete summaries — the third invariant
sees one of the two split pairs and cannot see the other. (Rung 68's own lesson — *check what is
independent before quoting it* — in its second shape.)

### 1.4 THE INVARIANTS

    c0 = det J = 0        ALWAYS — rows C and V stay parallel whatever the governor does
    c1 = (1 − pair_RC)/(τ_g τ_q)  +  (1 − pair_RV)/(τ_g τ_s)        [the (C,V) term vanishes]
    c2 = tr J = −Σ 1/τ_i                                            [the ODE's own diagonal]

`det J` is blind to this split exactly as it was to rung 69's. **`c1` is again the
discriminator — but it is no longer a single scalar times a clock factor.** Rung 69 had
`c1 = (1−k)·A·z` with `A = 1/τ_g + 1/τ_q`, `z = 1/τ_s`: the two shared-loop rates entered only
through their SUM. Here the two split pairs sit on **different clock products**, so the
bandwidths weight them independently.

### 1.5 THE RING, AND WHY ITS FLOOR CHANGES CHARACTER

With `c0 = 0` the cubic is `λ(λ² − c2 λ + c1)`, so the non-zero pair has
`λ₁+λ₂ = −Σ1/τ_i` and `λ₁λ₂ = c1`. Write `a = 1/τ_g`, `b = 1/τ_q`, `c = 1/τ_s`,
`u = 1 − pair_RC`, `w = 1 − pair_RV`:

    ζ  =  (a + b + c) / ( 2·sqrt( a·(u·b + w·c) ) )

Take `w ≥ u` WLOG. For fixed `a` and fixed `b+c = S` the denominator is largest with all of `S`
on the larger coefficient, i.e. `b → 0`; then `(a+S)² ≥ 4aS` gives

    ζ  ≥  1/sqrt(1 − min(pair_RC, pair_RV))          — the floor is set by the WORSE pair

**AND THE EQUALITY SET COLLAPSES FROM A HYPERPLANE TO A RAY.** Rung 69's `u = w = 1−k` makes
`b` and `c` enter only through `b+c`, so its floor is attained on the finite hyperplane
`a = b + c` — reachable with all three clocks finite. (It is *not* attained at matched clocks:
there `a = b = c` gives `A = 2/τ ≠ z = 1/τ`, which is why rung 69's own table reads ζ = 0.645
against a floor of 0.609.) Here equality needs `b → 0` **and** `a = c`: the floor becomes an
**infimum approached only by silencing one of the two loops that share the wall**, hence STRICT
at every admissible bandwidth triple.

---

## 2. PRE-REGISTERED PREDICTIONS

Structural — these follow from § 1 and a failure means the instrument or the state boundary is
wrong, not the plant:

* **P1** `zeros = 1` on every clock arm, at `(n,m) = (3,2)` — the second realization of that
  cell, reached by a different route than rung 69's.
* **P2** `c0 = det J = 0` to the instrument's floor on every arm. **A reader that inherited
  rung 68's determinant test reports rank one and sees nothing** — rung 69's correction,
  re-confirmed on a plant it was not derived on.
* **P3** `pair_CV = 1` to the differencing floor (the identity, relocated to the shared pair).
* **P4** `pair_RC ≠ pair_RV`, separated by **orders** above that floor. *This is the rung.*
* **P5** `x = −pair_RC` identically, and `x` carries **no information about `pair_RV`**.
* **P6** `c1 ≠ 0`, and it moves when the clocks are re-weighted at fixed plant — which
  rung 69's `c1 = (1−k)Az` cannot do at fixed `A`, `z`.

Magnitude — these can fail against the plant and would be findings:

* **P7 (the negative control)** `pair_RC` **is** rung 67's `P = R_q·C_g` — same governor, same
  valve, same closures. It must reproduce rung 67's measured `≈ −0.019` up to the base-point
  shift the third loop induces. A disagreement larger than that shift means the
  `_b_state`/`_v_state` boundary is wrong, **not** that the plant changed.
* **P8 — THE HEADLINE, AND IT IS A PREDICTED NULL.** Both split pairs are **cross-LEVER**
  fuel-vs-airflow gains, so both should sit at `|p| ≪ 1` ⇒ `1 − p ≈ 1` ⇒ **ζ_inf ≈ 0.99 ⇒ NO
  complex pair at ANY bandwidth.** Rung 69's `k ≈ −1.67…−2.01` came from **ONE lever reading
  TWO walls** (the `φ_v/φ²` geometry) — a lever fighting itself. If P8 holds, the finding is:

  > **THE SPLIT BUYS THE RANK; THE RING NEEDS THE ODD CONSTRAINT TO BE A SECOND WALL ON THE
  > SAME LEVER.**

  which upgrades rung 69's *complex iff `k < 0`* from a **condition** into a **mechanism**.
* **P9** The damping floor is STRICT at every admissible bandwidth triple (§ 1.5), against
  rung 69's attainable hyperplane. Measured as a two-axis sweep, not asserted.

Refusals — asserted rather than run:

* **P10** Arming rung 52's fuel leg AND the governor together is `n = 4, m = 2` — an
  unregistered plant, and "silently accepts it" is the exact failure rung 68's own `tau_gov`
  assert was written to prevent. It is named as rung 70's next seam.

---

## 3. SETTINGS, AND WHAT IS IMPOSED

* `Tt4_max = 1200 K` — **rung 67's imposed value, verbatim**, so rung 70's numbers difference
  against rung 67's rather than merely resembling them (rung 63's lesson). Rung 67 chose it for
  overlap with ONE `φ` loop; **all three windows must be verified to overlap before any ledger
  cell is quoted.**
* `φ_lim`, `b_max`, `v_max = 0.20` — rungs 64/57/58's, all IMPOSED and all inherited.
* `τ_g, τ_q, τ_s` are swept coordinates on the march's own `s`. No actuator bandwidth is
  anchored anywhere in this family. ORDERINGS, SIGNS and INVARIANCES are the claims; every
  MAGNITUDE is disclaimed.
* The evaluation manifold is rung 68's `manifold=True` instrument **unchanged** — it roots the
  stator on `φ = φ_lim`, which is the SHARED constraint here, so the base point is the one
  § 1.1's identities are stated at, and the two rungs' numbers stay differenceable.
* RK4: with `c1` small, `|λ| ≈ Σ1/τ_i`, which is the same dominant root rung 68's rank-one
  block gave — so the inherited constant stays conservative and is re-justified, not re-derived.
