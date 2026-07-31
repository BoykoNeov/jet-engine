# Rung 70 — THE GENERIC SPLIT

Rung 47's `Tt4` topping **governor** as the odd loop, beside rung 65's `φ` valve and rung 68's
`φ` stator. Rung 67's substitution applied to rung 68's triple: one loop's **sensor** moves, and
nothing else does.

    dg/ds = ( R(ν,q,v) − g ) / τ_g    R = rung 47's clip,  Tt4 ≤ Tt4_max   [GOVERNOR, Tt4]
    dq/ds = ( C(ν,g,v) − q ) / τ_q    C = rung 65's b_cmd, φ_lp ≥ φ_lim    [VALVE,    φ]
    dv/ds = ( V(ν,g,q) − v ) / τ_s    V = rung 68's v_cmd, φ_lp ≥ φ_lim    [STATOR,   φ]

Five states, three clocks, one actuator per loop. `n = 3`, `m = 2` — **the same cell as rung
69**, reached by a different route, so this is a controlled comparison at equal counts.

**It closes TWO seams, and rung 69 § 11 says they are one seam from two sides:** rung 68 § 10's
*three loops on TWO variables*, and rung 69 § 11's *a plant with `pair_RV ≠ pair_CV`*.

> **HEADLINE — THE SPLIT BUYS THE RANK; THE RING NEEDS THE ODD CONSTRAINT TO BE A SECOND WALL
> ON THE SAME LEVER.**
> Rung 69's ringing pair came from `k ≈ −1.7`, and that `k` was **one lever reading two walls**.
> Here the odd constraint sits on a different lever, both split pairs are cross-**lever** gains,
> and the damping floor lands at **0.990–0.992** — exactly where rung 67 put it, by the same
> scalar. Rung 69's *complex iff `k < 0`* is upgraded from a **condition** into a **mechanism**.

Pre-registration: `docs/plans/rung70-anchor-cross-split.md` (written before any code). Gates:
`tests/test_rung70.py`.

---

## 0. WHAT MOVED, AND WHAT DID NOT

| | rung 68 | rung 69 | **rung 70** |
|---|---|---|---|
| odd loop | fuel (rung 52) | stator (rung 68's lever) | **governor (rung 47)** |
| odd coordinate | `φ` | `M_i` | **`Tt4`** |
| odd **lever** | fuel | *the same stator* | **a different lever again** |
| `(n, m)` | (3, 1) | (3, 2) | **(3, 2)** |
| zeros | 2 | 1 | **1** |
| the shared pair | all three | `(R,C)` | **`(C,V)`** |
| split pairs | — | `pair_RV = pair_CV = k` | **`pair_RC ≠ pair_RV`, opposite SIGNS** |
| `det J` | 0 | 0 | **0** |
| the ring | none (real) | complex, `ζ ≥ 0.61` | **real; complex only on the ray, `ζ ≥ 0.990`** |

The plant, the ramp, the maps, `φ_lim`, `b_max`, `v_max` and `Tt4_max` are all inherited. What
changes between rungs 68 and 70 is **which variable the odd loop watches**; what changes between
69 and 70 is **which lever carries it**.

---

## 1. THE DERIVATION — the identity moves, and the split stops collapsing

Rung 69 § 1, unchanged: each lagged law solves its own constraint for its own actuator, so every
row of the actuator block is a multiple of **its own** constraint's gradient,

    row_i(M) = −(1/c⁽ⁱ⁾_i)·∇c⁽ⁱ⁾ᵀ      ⇒      rank M = dim span{∇c⁽ⁱ⁾} =: m ,   zeros = n − m

With `T := Tt4` and `φ := φ_lp`, the constraints are `T = Tt4_max` once and `φ = φ_lim` **twice**:

    row_R = −(1/T_g)·∇Tᵀ ,      row_C = −(1/φ_q)·∇φᵀ ,      row_V = −(1/φ_v)·∇φᵀ

Rows C and V are parallel; row R is not. So `m = 2`, one zero — and the table of rung 69 § 1
gains a second entry in its `(3,2)` cell, the one where the odd constraint does *not* factor.

### 1.1 The six gains

    R_q = −T_q/T_g     R_v = −T_v/T_g
    C_g = −φ_g/φ_q     C_v = −φ_v/φ_q
    V_g = −φ_g/φ_v     V_q = −φ_q/φ_v

    pair_CV = C_v·V_q = 1                      ← THE SHARED PAIR: rung 66's identity, MOVED
    pair_RC = R_q·C_g = (T_q φ_g)/(T_g φ_q)      SPLIT
    pair_RV = R_v·V_g = (T_v φ_g)/(T_g φ_v)      SPLIT

**Which pair keeps rung 66's identity is a direct read of which two loops share a constraint.**
A reader that inherited rung 69's `pair_RC = 1` *negative control* would here be reading a
**signal** as a control — the same class of error as inheriting rung 68's determinant test.

### 1.2 The discriminator, and why rung 69 could not make this measurement

    pair_RC / pair_RV = (T_q/φ_q)·(φ_v/T_v)      ⇒     equal  iff  T_q/φ_q = T_v/φ_v

i.e. **iff `Tt4` depends on `(q, v)` only through `φ`**. At rung 69 that held *trivially* —
`M_i = T_c − 1/φ + v` differs from the shared wall by exactly the lever's own direct channel —
and both split pairs collapsed onto one scalar `k`. Rung 69 was explicit that this was *a
measurement of the two walls' relationship, not a restatement of the rank*, and equally explicit
that it was untested until a plant existed where it fails. **This is that plant.**

### 1.3 The cyclic product goes half-blind

    x := R_q·C_v·V_g = (−T_q/T_g)(−φ_v/φ_q)(−φ_g/φ_v) = −(T_q φ_g)/(T_g φ_q) = **−pair_RC**

Rung 68 said *quote `x`*; rung 69 said *`x` flips sign to `−k`*. Both were complete summaries
only because every split pair was one scalar. Here `x` reproduces **`pair_RC` alone and is
structurally blind to `pair_RV`** — rung 68's own *check what is INDEPENDENT before quoting it*,
in its second shape.

### 1.4 The invariants — and no single scalar summarises them

    c0 = det J = 0        ALWAYS — rows C and V stay parallel whatever the governor does
    c1 = (1 − pair_RC)/(τ_g τ_q)  +  (1 − pair_RV)/(τ_g τ_s)      [the (C,V) term vanishes]
    c2 = tr J = −Σ 1/τ_i                                          [the ODE's own diagonal]

`det J` is blind to this split exactly as it was to rung 69's, and `c1` is the discriminator
again — but rung 69's `c1 = (1−k)·A·z` had the two shared rates entering only through their
**sum**, while here the two split pairs sit on **different clock products**.

**The surviving clock factor is always the ODD loop's clock**, because the pair that shares
contributes nothing: rung 69's two terms both carry `1/τ_s` (its odd loop, the stator); both of
rung 70's carry `1/τ_g` (its odd loop, the governor). The clock products are a free read of
which two loops share a constraint — gated in `test_the_surviving_clock_product_names_which_loops_share`
against a hand-built rung-69 block.

### 1.5 The ring's floor changes CHARACTER

With `c0 = 0` the cubic is `λ(λ² − c2 λ + c1)`, so the non-zero pair has `λ₁+λ₂ = −Σ1/τ_i` and
`λ₁λ₂ = c1`. Write `a = 1/τ_g`, `b = 1/τ_q`, `c = 1/τ_s`, `u = 1 − pair_RC`, `w = 1 − pair_RV`:

    ζ  =  (a + b + c) / ( 2·sqrt( a·(u·b + w·c) ) )     ≥   1/sqrt(1 − min(pair_RC, pair_RV))

**The equality set collapses from a HYPERPLANE to a RAY.** Rung 69's `u = w` makes `b` and `c`
enter only through `b+c`, so its floor is attained on `a = b + c` — reachable with all three
clocks finite. (It is *not* attained at matched clocks: there `A = 2/τ ≠ z = 1/τ`, which is why
rung 69's own table reads ζ = 0.645 against a floor of 0.609.) Here equality needs one shared
loop **silenced** *and* `a` matched to the other, so the bound is an **infimum no admissible
triple reaches**. Which loop gets silenced is a property of the plant — the one with the smaller
`1 − pair` — and is measured, not assumed (`split_floor.silenced`).

---

## 2. MEASURED — § 1, on the shipped closures

`Tt4_max = 1200 K`, `φ_lim = 0.80`, `b_max = 0.10`, `v_max = 0.20`, `τ = (0.05, 0.05, 0.05)`,
`ds = 0.005`, `dg = 1e−7`, `dq = 1e−5`, `dv = 1e−4`, base points on the shared manifold.

### 2.1 The windows overlap — a GATE, not a remark

| leg | window in `s` | points |
|---|---|---|
| governor | 0.115 … 1.700 | 318 |
| valve | 0.000 … 0.590 | 119 |
| stator | 0.005 … 0.415 | 83 |
| **joint** | **0.115 … 0.415** | **61** (17.9 % of the march) |

`Tt4_max` is rung 67's, chosen there for overlap with **one** `φ` loop. With a third window the
intersection has to be re-measured: a gain table over an empty one would report the pairwise
algebra of loops that were never simultaneously live.

### 2.2 The pairwise split (7 sampled points, `every = 10`)

| `s` | `pair_RC` | `pair_RV` | `pair_CV` | cyclic `x` | rung 68's fuel leg, same point |
|---|---|---|---|---|---|
| 0.115 | −0.016717 | **+0.126503** | 1.0000000001 | +0.016717 | `pair_RC = 1` |
| 0.165 | −0.017209 | +0.123189 | 1.0000000001 | +0.017209 | `pair_RC = 1` |
| 0.215 | −0.017871 | +0.120558 | 1.0000000001 | +0.017871 | `pair_RC = 1` |
| 0.265 | −0.018459 | +0.118344 | 1.0000000000 | +0.018459 | `pair_RC = 1` |

* worst `|pair_CV − 1| = 1.06e−10` — the identity, on its new pair.
* worst `|pair_RC(fuel) − 1| = 3.09e−10` at the **identical base points**: this is what makes
  *the identity MOVED* a measurement on one trajectory rather than a comparison of two rungs'
  tables.
* worst `|x + pair_RC| = 1.17e−11`, while `|x + pair_RV| > 0.1` throughout — § 1.3.
* **The two split pairs come back with OPPOSITE SIGNS**, which is stronger than the registered
  prediction of "different by orders". Relative gap 1.13–1.18, against an instrument floor of
  1e−10: nine orders.

**The sign is the mechanism.** The odd constraint couples with *opposite sign* through the two
shared actuators: bleed raises `Tt4` at fixed fuel (rung 67's `R_q > 0`), while closing the
stators does not reach `Tt4` the same way. And `pair_RV = +0.12` sits an eighth of the way
toward rung 66's degenerate `+1`, so the stator's leg is partially *shared-like* while the
valve's is rung-67-like — which is why no single scalar can summarise the pair, stated
physically rather than algebraically.

### 2.3 The state boundary, asserted against its own broken version

| | `R_q` | `R_v` |
|---|---|---|
| with `_b_state`/`_v_state` | 1.122e−3 | 4.588e−3 |
| **without** (the failure mode, built on purpose) | **0.0** | **0.0** |

`R_q ≠ 0` only because the governor senses `Tt4` on the machine as the other two actuators
actually are. Drop the boundary and the odd loop decouples, `m` reads 1 by accident, and **every
prediction in this rung would "confirm" rung 68**. Rung 68 names this as the one thing here that
can go wrong without failing; it is checked at every sampled point rather than inherited.

---

## 3. THE SPECTRUM — one zero, `det` blind, `c1` clock-weighted

`ds = 0.002`, `every = 20`, clocks as `(τ_g, τ_q, τ_s)`:

| `(τ_g, τ_q, τ_s)` | zeros | max `|c0|/Σ³` | min `|c1|/Σ²` | `c1` vs § 1.4 | pair | `ζ` |
|---|---|---|---|---|---|---|
| (0.05, 0.05, 0.05) | **[1]** | 6.02e−12 | 2.10e−1 | 8.6e−11 | real | 1.086 … 1.090 |
| (0.05, 0.005, 0.05) | **[1]** | 7.96e−13 | 1.32e−1 | 6.9e−12 | real | 1.376 … 1.379 |
| (0.05, 0.5, 0.05) | **[1]** | 1.60e−12 | 4.29e−2 | 8.4e−10 | real | 2.395 … 2.413 |
| (0.02, 0.05, 0.10) | **[1]** | 4.43e−12 | 1.87e−1 | 5.6e−11 | real | 1.156 … 1.157 |

**`ζ` is read from BOTH non-zero roots**, `ζ = −(λ₁+λ₂)/2√(λ₁λ₂)`, not from rung 69's
`−Re(dom)/|dom|`. Rung 69's reader is exact for the complex pair it measured and returns exactly
1.0 for *any* real root — here the pair is real, so that reader would report `ζ = 1` on every arm
and § 1.5's floor would be untestable. An instrument that cannot tell "critically damped" from
"overdamped 3×" cannot measure a bound whose whole content is the margin above 1.

### 3.1 The clock swap — the only reading a one-scalar plant fails

That `c1 ≠ 0` is *rung 69's* result. That `c1` moves across a clock grid proves nothing (the rate
sum moves too). That the measured `c1` matches § 1.4 to 1e−10 validates the formula against
itself. **The discriminating test holds `τ_g` and exchanges `(τ_q, τ_s)`:**

| | `c1` at `(τ_q,τ_s) = (0.02, 0.10)` | at `(0.10, 0.02)` | ratio |
|---|---|---|---|
| **this plant** | 1.196220e+3 | 1.085807e+3 | **0.907698** |
| one-scalar null (rung 69's shape, this plant's gains) | 1.141014e+3 | 1.141014e+3 | **1.000000** |
| the two marched arms | 1.196220e+3 | 1.084222e+3 | 0.906373 |

measured Δ = **−1.104137e+2** = predicted Δ (§ 1.4) to 1e−9; the null's Δ is −2.3e−13.

Every `c1` in that table comes from the shipped `_invariants` — the actual 3×3 Jacobian — and
never from § 1.4's closed form, which appears once, as the thing under test. **Rung 67 gate 9
was retracted for being a tautology; this gate was rewritten before shipping for the same
reason.** The null is built from *this plant's own gains* with `pair_RC` and `pair_RV` forced to
their mean through the two gains that carry them, so it differs from the plant in exactly one
respect. The marched arms agree with the held-gains reading to 0.15 %, which is the plant's own
drift between two marches.

---

## 4. THE FLOOR, AND THE PREDICTED NULL THAT FAILED

`split_floor`, `ds = 0.005`, `(τ_g, τ_q, τ_s)`:

**Read the two ray coordinates together.** `u = 1 − pair_RC ≈ 1.020` and `w = 1 − pair_RV ≈
0.874`, so `u > w` and the equality set silences the **stator** — the loop attached to the
*smaller* coefficient. `silenced` is therefore a **plant property, constant down the column**,
naming which loop the ray would quiet; it is *not* a label for whichever clock a given row
slowed. Equality then needs `a` matched to the **surviving** shared rate, which is the
**valve** (`τ_g = τ_q`) and *not* `τ_s` — so an arm is on the ray only when **both**
`quiet_share → 0` **and** `a/loud → 1`.

| `(τ_g, τ_q, τ_s)` | quiet share | `a/loud` | `ζ` | floor | complex? |
|---|---|---|---|---|---|
| (0.05, 0.05, 0.05) | 0.333 | 1.00 | 1.0882 | 0.99090 | no |
| (0.05, 0.05, 0.025) | 0.500 | 1.00 | 1.1995 | 0.99128 | no |
| (0.05, 0.05, 0.10) | 0.200 | 1.00 | 1.0345 | 0.99054 | no |
| (0.10, 0.10, 0.05) | 0.500 | 1.00 | 1.1987 | 0.99141 | no |
| (0.20, 0.02, 0.05) | 0.267 | 0.10 | 2.0234 | 0.99049 | no |
| (0.02, 0.20, 0.05) | 0.267 | 10.0 | 1.1138 | 0.99135 | no |
| (0.05, 2.00, 0.05) | 0.494 | 40.0 | 1.0639 | 0.99197 | no |
| **(0.05, 0.05, 2.00)** | **0.012** | **1.00** | **0.99216** | 0.99041 | **YES** |
| (2.00, 0.05, 2.00) | 0.024 | 0.03 | 3.2529 | 0.99024 | no |

`ζ` agrees with the closed form to 7.1e−11 on every arm; `holds` and `strict` both true.

**Exactly one arm satisfies both ray coordinates, and it is the only one that rings.** The
bolded row has `quiet_share = 0.012` *and* `a/loud = 1.00`, and lands 0.18 % above the floor.
The last row looks close on `quiet_share` alone but has `a/loud = 0.03` — it slowed the
governor as well, so it is not on the ray and sits at ζ = 3.25. `(0.05, 2.00, 0.05)` silences
the **valve**, which is the wrong loop here, and is the farthest arm of all. That the ray
coordinates and the complex branch pick out the same single arm is the strongest form § 1.5's
claim takes on this grid.

**PRE-REGISTERED P8 SAID *NO COMPLEX PAIR AT ANY BANDWIDTH*. THAT IS FALSE.** The floor is
`≈ 0.990 < 1`, so a complex pair is *admitted*, and it is found — on the arm with `τ_s` at 40×
the others, i.e. the ray that nearly silences the stator. The honest sentence is therefore not
"no ring" but:

> The ring is reachable **only where the third loop is dynamically inert** (its share of the
> rate sum is 1.2 %), and even there `ζ = 0.992` puts it back in rung 67's *admissible,
> unobservable* class — decay by `e^{−24}` over one period.

*Reachable* and *reachable with three live loops* are different sentences, and the gate asserts
both: no complex pair on any arm with comparable clocks, one on the ray.

### 4.1 The floor IS rung 67's damping ratio — and that is CONTINGENT

    rung 67:   ζ = 1/sqrt(1 + |P|)                         |P| ≈ 0.019   ⇒  0.9906
    rung 70:   ζ ≥ 1/sqrt(1 − min(pair_RC, pair_RV))                     ⇒  0.9902 … 0.9920

The same formula, because **`min()` selects `pair_RC`, and `pair_RC` *is* rung 67's `P`** — same
governor, same valve, same closures. So on this plant:

> **A third loop that SHARES a constraint adds a zero and moves the achievable damping
> NOWHERE.** The floor is set by the cross-lever pair alone — the scalar rung 67 already
> measured on two loops.

**This is conditional on a sign, not structural.** `min()` selects `pair_RC` only because
`pair_RV` came back **positive**. Had the stator's split pair been the more negative one, the
floor would be set by a gain rung 67 never measured and the coincidence would vanish. The gate
asserts the condition (`pair_RV > 0`, and `worse_pair ∈ pair_RC`) alongside the consequence, so a
plant that broke the sign fails there rather than silently invalidating the identity.

### 4.2 The RK4 constant, conservative for a THIRD reason

Rung 68's `ds·Σ(1/τ_i) ≤ 2` is exact-in-argument there (rank one, non-zero eigenvalue exactly
`−Σ1/τ_i`). Rung 69 kept it on a different argument (a complex pair of modulus `√(Az(1−k))`,
bounded by `√(1−k)/2 × Σ`, conservative for `k ≥ −3`). Here `min(pair) ≈ 0` puts the pair back on
the real axis with the dominant root near `−Σ1/τ_i` again — rung 68's reason, on a plant rung
68's derivation does not cover. **Measured**, not trusted: max `|λ|/Σ(1/τ) = 0.976`, max
`ds·|λ| = 0.98`. The guard fires at `ds = 0.05` with a message naming its own reason.

---

## 5. THE LEDGER — two currencies, opposite-sign cross-credits

`I` = rung 66's `∫max(0, φ_lim − φ_lp) ds`; `E` = rung 67's `∫max(0, Tt4 − Tt4_max) ds`. Both
inherited unchanged, so this table differences against rungs 66/67/68 rather than resembling
them. Every cell from one rig (rung 63's lesson).

| cell | `I` (`φ`) | `E` (`Tt4`) | min `φ_lp` | max `Tt4` |
|---|---|---|---|---|
| bare | 2.5815e−2 | 109.95 | 0.7354 | 1695.4 |
| G | 2.0437e−2 | **27.47** | 0.7430 | 1279.2 |
| V | 1.9394e−3 | 117.01 | 0.7891 | 1717.5 |
| S | 2.1421e−3 | 94.85 | 0.7884 | 1679.7 |
| GV | 1.4083e−3 | 28.56 | 0.7891 | 1281.5 |
| GS | 1.6930e−3 | 26.08 | 0.7884 | 1282.7 |
| VS | 1.0229e−3 | 111.16 | 0.7934 | 1716.7 |
| **GVS** | **7.9201e−4** | **28.32** | **0.7934** | **1283.1** |

Marginal contribution to the full triple:

| | `φ` | `Tt4` |
|---|---|---|
| governor | +2.31e−4 | **+82.84** |
| valve | +9.01e−4 | **−2.25** |
| stator | +6.16e−4 | +0.24 |

**Rung 67's opposite-sign cross-credit survives the third loop.** The valve **debits** the
temperature (`R_q > 0`: bleed makes it hotter at fixed fuel) while the governor **credits** the
surge margin (`C_g < 0`: clipping fuel raises `φ_lp`). One loop helps the other; the other hurts
it — an object rung 68's one-currency ledger structurally could not hold.

**And rung 68's erosion is confirmed as a property of the SHARED constraint.** Each `φ` loop's
marginal contribution is a fraction of what it delivers alone (valve 9.0e−4 marginal against
2.39e−2 alone; stator 6.2e−4 against 2.37e−2) — while the governor keeps ~100 % of its own
currency's credit (82.8 marginal against 82.5 alone). **A loop is eroded by the loops it shares a
constraint with, and by no others.**

---

## 6. Reduce contract

| arm | reaches | how |
|---|---|---|
| `tau_gov=None`, `stator_lim` armed | **rung 68**, bit-for-bit | dispatch |
| `tau_gov=None`, `stator_inc` armed | **rung 69**, bit-for-bit | dispatch |
| no stator, `tau_gov` set | **rung 67**, bit-for-bit | dispatch (the parent's own) |
| no stator, no governor | rungs 66/65/64/62, bit-for-bit | dispatch |

All four verified over 341 points on 9 recorded keys, worst difference **exactly 0.0**. This
class never intercepts a march it does not own. `at_lever` returns `CrossSplitTransient` — the
eighth instance of the trap rungs 61–69 each hit.

`τ_gov → ∞` and `τ_s → ∞` are converging limits, not reduce arms (a different code path with a
fifth state); rung 68's argument, verbatim.

### 6.1 The four refusals, each a plant this rung is NOT

* **an INCIDENCE stator beside the governor** — `n = m = 3`, zero zeros: the one cell of rung
  69 § 1's table this ladder has never occupied. **Rung 70's own next seam.**
* **rung 52's fuel leg beside the governor** — `n = 4, m = 2`, four loops with two on one
  actuator. Rung 68's own `tau_gov` assert exists because *silently accepts it* is the failure
  mode; this is its mirror.
* **`tau_gov` without `Tt4_max`** — would march as rung 68 while every reader reported rung 70.
* **`ds = 0.05`** — the RK4 guard, which rung 68 measured as counterfeiting *perfect
  protection* when violated.

---

## 7. Predictions, scored

| | prediction | outcome |
|---|---|---|
| P1 | `zeros = 1` on every arm | **HELD** — `[1]`, four arms |
| P2 | `c0 = 0`; a rung-68 determinant reader sees nothing | **HELD** — ≤ 6.0e−12 |
| P3 | `pair_CV = 1` to the floor | **HELD** — 1.06e−10 |
| P4 | `pair_RC ≠ pair_RV` by orders | **HELD, and EXCEEDED** — opposite SIGNS |
| P5 | `x = −pair_RC`, blind to `pair_RV` | **HELD** — 1.17e−11 / > 0.1 |
| P6 | `c1 ≠ 0` and clock-re-weightable | **HELD** — and the swap kills the one-scalar null |
| P7 | `pair_RC` reproduces rung 67's `P` | **HELD** — [−0.0199, −0.0167] vs [−0.0208, −0.0191], ratio 0.92 |
| P8 | **no complex pair at ANY bandwidth** | **REFUTED** — § 4. Replaced by the ray statement, which is the better result |
| P9 | the floor is strict at every admissible triple | **HELD** — `strict = True`, 9 arms |
| P10 | the `n = 4` plant refused, not run | **HELD** |

P8's refutation is the rung's best moment: it converted a predicted null into an **invariance**
(§ 4.1) plus a **contingency** (the sign of `pair_RV`), both of which are checkable and neither
of which was in the anchor. The anchor document is **not** edited — a prediction revised after
the measurement is not a prediction.

---

## 8. Concessions

* Every one rungs 62–69 list, all inherited.
* `Tt4_max = 1200 K` is **rung 67's imposed value**, taken verbatim for differenceability. It is
  not rung 46/47's redline, and every number here is conditional on it.
* `φ_lim`, `b_max` (rung 64) and `v_max = 0.20` (rungs 57/58) remain **imposed**.
* The `φ`-referenced stator still moves the lever in the **anti-physical** direction and erodes
  incidence margin while protecting `φ` (rung 68's concession, verbatim).
* All three clocks are swept coordinates on the march's own `s`. **No actuator bandwidth is
  anchored anywhere in this family.** ORDERINGS, SIGNS and INVARIANCES are the claims; every
  MAGNITUDE is disclaimed.
* § 4.1's identity is **contingent on `pair_RV > 0`** (stated in § 4.1, gated).
* `min(pair) ≈ 0` is measured on THIS plant. Whether a cross-**lever** pair is always weak is not
  established; the claim is the mechanism, with this plant's numbers as one instance of it.
* The spectrum is sampled at finitely many trajectory points — a diagnostic that can miss a
  brief excursion (rung 65's retracted trap), not a proof of convergence.
* The joint window is 17.9 % of the march. Every gain table lives inside it, and the ledger's
  cells do not — they integrate the whole ramp, as rungs 66/67/68's do.
* The STAGE STACK (rungs 55/56) is still off the transient ladder.
* This still does **not** close rung 63's *fuel + bleed + STATOR* seam: that seam wants the
  stator as an OPEN-loop **schedule**, and this is a closed loop.

---

## 9. Next seams

* **`n = m = 3`** — an INCIDENCE stator beside the governor and the valve: three loops, three
  constraints, **zero zeros**. The refusal in § 6.1 names it, the lever exists (rung 69 built
  it), and it is the last unoccupied cell of rung 69 § 1's table at `n = 3`. **This is the
  strongest one.**
* **`n = 4, m = 2`** — the fuel leg beside the governor: two loops on ONE actuator, which no
  rung in this family has yet built. It asks whether the rank law's `m` counts constraints or
  *actuators*, and § 1's derivation assumes one law per actuator throughout.
* **A plant where `min(pair)` is NOT rung 67's `P`** — i.e. one where the stator's split pair
  goes negative. That would break § 4.1's coincidence and is the direct test of whether the
  identity is contingent (as claimed) or structural.
* **`m` moved without changing `n`** — rung 69 § 11's, still open: it needs a fourth LP lever
  this plant does not have.
* An **ASYMMETRIC valve** (rung 65) and an **asymmetric governor** (rung 67) — both still open;
  all three lags here are symmetric.
* **Fuel + bleed + STATOR-as-a-SCHEDULE** — rung 63's seam, still open after 64–70.
* Everything rung 68 § 10 left: a plant with `|P| > 1`, `n = 4` on one variable, and the real
  spatial/transported-CFD PDF.
