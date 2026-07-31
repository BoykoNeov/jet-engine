# Rung 68 anchor — THREE LOOPS ON ONE VARIABLE

**Status: § 0 is MEASURED (a feasibility pre-check + a discriminator, both run before the
predictions were written). §§ 1–5 are DERIVATION ONLY.** Every number in §§ 1–5 is a
prediction, not a measurement. Predictions are scored HIT / MISS after the sweep, and a MISS is
published in place, as in rungs 51/58/63/64/65/66/67.

Rung 66's standing seam, stated there in full:

> **THREE loops on one variable.** The identity is stated for a pair. `det J ≡ 0` for two laws
> on one constraint suggests a rank deficiency that GROWS with the loop count — an `n`-loop
> block on one manifold should have `n − 1` zero eigenvalues, so the third limiter would buy
> even less than the second's 1.59 %. Derivable, unmeasured.

The third actuator is the **variable stator** (rung 53), closed onto rung 49's `φ_lp` floor —
the same set point rung 52's fuel leg and rung 65's bleed valve already hold. It is the only
remaining lever on this plant with authority over `φ_lp`, and using it also puts a first foot
in rung 63's untouched *fuel + bleed + stator* seam.

---

## 0. THE PRE-CHECK — measured FIRST, because it decides whether this is a rung at all

Rung 65's precedent, and rung 67 § 0's: run the measurements that can kill the rung before
writing anything that depends on them, and publish them as a pre-check rather than as findings.

### 0.1 Is there an arc where all THREE loops ride, INTERIOR?

**This is the blocker, and the reason is rung 67's own lesson: a zero cross-gain is
SATURATION, never decoupling.** A third loop sitting on its stop contributes a zero row to the
actuator Jacobian, which lowers the rank for a reason that has nothing to do with a shared
constraint. So the whole rung is evidence only at points where all three laws are live and
strictly between their stops.

Measured on the **shipped rung-66 cascade march**, settings verbatim from `tests/test_rung66.py`
(`φ_lim = 0.80`, `b_max = 0.10`, `τ_v = 0.05`, `τ_att/τ_rel = 0.05/0.15`, `ds = 0.005`,
`r = 0.5`, `s_settle = 1.2`, `Tt4` 1000 → 1400 K). `v_cmd` is a **diagnostic root only** — the
stator setting that would put `φ_lp` on the floor at the live state, applied fuel and valve
position. No new state, no new integrator, no new control law:

| | |
|---|---|
| march points | 341, `s ∈ [0, 1.70]` |
| rung-66 riding points (`required > 0` **and** `0 < b_cmd < b_max`) | **114**, `s ∈ [0.005, 0.570]` |
| of those, `v_cmd` rooted | **114 / 114** |
| `v_cmd` range | **[−0.01718, +0.02705]** |
| `v_cmd < 0` (the protective side — see 0.2) | **73**, `s ∈ [0.005, ≈0.375]` |

**PASS.** 73 points with all three loops live and interior, and the arc covers the EARLY-ramp
LP minimum — the binding one (rungs 41/44), and where every rung-66 number is taken.

### 0.2 WHICH WAY does the stator have to move? — the sign, and it is not the physical one

Measured at `s = 0.29` (`φ_lp = 0.797776`, `b = 0.0765`, `g = 2.52e−3`), `φ_lp` against a
trial LP stator setting:

| `v` | −0.20 | −0.10 | −0.05 | −0.02 | 0.00 | +0.02 | +0.05 | +0.10 | +0.20 |
|---|---|---|---|---|---|---|---|---|---|
| `φ_lp` | 0.8949 | 0.8430 | 0.8196 | 0.8063 | 0.7978 | 0.7894 | 0.7773 | 0.7581 | 0.7230 |

**`dφ_lp/dv ≈ −0.42`: CLOSING the stators LOWERS `φ_lp`.** So a loop referenced to a fixed
`φ_lim` must **OPEN** them (`v < 0`) to protect. That is the opposite of what real hardware
does — a VSV schedule closes at low corrected speed — and it is opposite for a reason rung 53
already published: closing lowers the *wall* `φ_surge(v) = 1/(T_c+v)` faster than it lowers
`φ`, which is where rung 53's credit comes from. A `φ`-referenced loop cannot see the wall.

Two consequences, both derived from the measured slope and both **pre-registered as
concessions, not discoveries** (`T_c = 1/0.55 = 1.8182`, `φ ≈ 0.7978`):

    dM_φ/dv = dφ/dv − dφ_surge/dv = −0.4176 + 0.3025 = −0.115     [the φ-margin wall MOVES]
    dM_i/dv = (1/φ²)·dφ/dv + 1    = −0.6563 + 1     = +0.344      [the metal wall does NOT]

**The φ-referenced stator loop protects `φ` and ERODES incidence margin.** Every credit this
rung quotes must therefore name its wall — that is rung 53's *a margin is a DISTANCE* landing
directly on this rung's ledger, not an optional caveat.

### 0.3 THE DISCRIMINATOR — the six cross-gains, before the anchor

Rung 64's move: run the one measurement that separates the hypotheses before committing to the
build. Central differences on three **shipped, mutually ignorant** closures — `_surge_fuel`
(the fuel law), `_solve_b` (the valve's), and a bisection on `_instant_fuel`'s `φ_lp` (the
stator's) — at the shared-manifold point, `dg = 1e−7`, `dq = 1e−5`, `dv = 1e−4`:

| `s` | `R_q` | `C_g` | `R_v` | `V_g` | `C_v` | `V_q` |
|---|---|---|---|---|---|---|
| 0.005 | −5.736e−2 | −17.433 | +3.229e−2 | +31.0 | +0.5629 | +1.7764 |
| 0.305 | −7.421e−2 | −13.475 | +4.325e−2 | +23.1 | +0.5827 | +1.7160 |
| 0.555 | −9.535e−2 | −10.488 | +6.320e−2 | +15.8 | +0.6628 | +1.5087 |

The three **pairwise** products are `1.000000000`, `1.000000005`, `1.000000001` — which is
rung 66's identity, three times, and **not** evidence of joint collapse. What is independent at
`n ≥ 3` is the **CYCLIC** product. With the three pairwise constraints imposed, the block

    M = [[−1, a, b], [c, −1, d], [e, f, −1]],   ac = be = df = 1,   x := a·d·e = R_q·C_v·V_g

has **one free parameter left, `x`**, and `det M = 2 + x + 1/x = (x + 1)²/x` exactly. So `det`
is a monotone re-expression of `x` and carries nothing beyond it; `tr M = −3` is the hardcoded
diagonal and `Σ`(2×2 principal minors)` = 3 − Σ`(pairwise products) is the pairwise result
restated. **Only `x` is quoted.** Its predicted value is **−1** (three factors of `−φ_j/φ_i`).

Measured, `x = −1.0000000052` at every sampled riding point — and the departure is **at the
shipped root-finders' tolerance floor, not at the differencing truncation**: halving
`dv/dq/dg` four times leaves `x+1` wandering in `[−2.6e−8, +2.6e−9]` instead of shrinking.

**The detector's sensitivity is MEASURED, not asserted** (the golden-gate lesson). Displacing
`v` off the shared manifold by `δ`:

| `δ` | 0 | 1e−4 | 1e−3 | 1e−2 | 3e−2 | 1e−1 |
|---|---|---|---|---|---|---|
| `x + 1` | 2.4e−9 | 1.50e−4 | 1.50e−3 | 1.49e−2 | 4.45e−2 | 1.00 |
| `det` | 1.3e−16 | −4.6e−9 | −4.6e−7 | −4.6e−5 | −4.1e−4 | −0.116 |

Linear in `δ` with gain ≈ 1.5, against a noise floor of 5e−9: the instrument resolves an
off-manifold displacement of `δ ≳ 1e−8`.

**AND THE `δ = 1e−1` ROW IS THE RUNG'S OWN TRAP, CAUGHT IN THE PRE-CHECK.** There `ac−1 = −1`
and `df−1 = −1`, i.e. `C_g = C_v = 0`: the **valve saturated** at `b_max`, and the block's
`det` went to −0.116. **A saturated loop counterfeits INDEPENDENCE, not degeneracy** — it
removes its own row from the coupling, so a reader that only checks `det ≠ 0` would report a
non-degenerate triple that is really a degenerate *pair* plus a stop. This is the exact inverse
of the shape rung 67 named, and § 4's P6 registers it.

### 0.4 THE ANCHOR

`φ_lim = 0.80`, `b_max = 0.10`, `v_max = 0.20`, `τ_v = τ_s = 0.05`, `τ_att/τ_rel = 0.05/0.15`,
`ds = 0.005`, `r = 0.5`, `s_settle = 1.2`, `Tt4` 1000 → 1400 K. Everything except `v_max` and
`τ_s` is **rung 66's, verbatim**, so the two rungs are comparable at the same settings (rung
63's lesson). `v_max = 0.20` is rungs 57/58's swept setting `V`, inherited rather than chosen —
**zero new constants**; `τ_s` is a swept coordinate on the march's own `s`, exactly as `τ_v`
and `τ_g` are, with every magnitude disclaimed.

---

## 1. The plant — FIVE states, THREE clocks

Spools `ν_lp, ν_hp` (rung 40), the fuel-clip amount `g` (rung 52), the valve position `q`
(rung 65), and now the LP stator setting `v`:

    dg/ds = ( R(ν, q, v) − g ) / lag.tau(R, g)     R = rung 52's required clip   [the FUEL]
    dq/ds = ( C(ν, g, v) − q ) / τ_v               C = rung 65's b_cmd           [the VALVE]
    dv/ds = ( V(ν, g, q) − v ) / τ_s               V = the setting putting φ_lp on the floor

`V` is **one-sided**, like every limiter on this plant: `v ∈ [−v_max, 0]` with **`v = 0` the
DORMANT stop** (the design setting) and `−v_max` the saturated one. Per 0.2 the protective
direction is *negative*, so `_solve_v`'s bracket orientation and both of its clamp tests are
**inverted relative to `_solve_b`** — `φ_lp` decreases in `v` where it increases in `b`. Get
that backwards and the regime label is wrong with no test failing: **the rung-62 `_powers`
trap, fourth reload.**

**THE SECOND HALF OF THAT TRAP IS THE REFERENCE.** `V` must be rooted on the **running-line
`φ_lp`** reached through `ComponentMap.with_vsv`, never on the moved wall through
`phi_surge_at`. Target the wall and this is rung 60's *incidence* loop by accident: the
constraint stops being shared, `x ≠ −1`, the rank comes out 2, and **nothing fails.**

The coupling is by construction and is an assumption with a physical justification, not a
discovery (rung 66's wording, extended): every closure that represents *the plant* runs with
both `_b_state` and `_v_state` set, because a real limiter watches the machine it is on; only
the two roots over TRIAL positions run without their own.

---

## 2. THE DERIVATION — `J = −D·c·rᵀ`, and the diagonal is not a special case

`n` laws, each solving the **same** constraint `φ(u₁ … u_n) = φ_lim` for its own actuator given
the others. Differentiating `φ(U_i(u_{−i}), u_{−i}) = φ_lim` in `u_j`:

    φ_i · ∂U_i/∂u_j + φ_j = 0       ⇒       ∂U_i/∂u_j = −φ_j/φ_i

and at `j = i` that same formula returns `−1`, so **the diagonal is not a special case** and

    M := [∂U_i/∂u_j − δ_ij]  =  −c·rᵀ ,    c_i = 1/φ_i ,  r_j = φ_j
    J  =  D·M  =  −D·c·rᵀ ,                D = diag(1/τ_i)

**`J` is RANK ONE, for every `n`, every plant, every gain, every bandwidth.** Its eigenvalues
are `n − 1` zeros and one non-zero equal to its trace:

    tr J  =  −Σ_i (1/τ_i)·c_i·r_i  =  −Σ_i 1/τ_i            [c_i r_i = 1 for every i]

**THE RATES ADD, at every `n`.** At `n = 2` this reproduces rung 66's `{0, −(1/τ_g + 1/τ_v)}`
exactly, so rung 66's result is the `n = 2` case of a general statement rather than a pair
identity — and rung 66's `R_q·C_g ≡ 1` is one entry of `−c·rᵀ`. The `n ≥ 3` content beyond it
is the **cyclic** product `∏(−φ_j/φ_i)` around a 3-cycle `= −1` (§ 0.3), which the pairwise
products do **not** force.

### The guard this buys, and it is TIGHTER again

    assert  ds · Σ_i (1/τ_i)  ≤  2.0

At three matched clocks that is `ds/τ ≤ 2/3`, against rung 66's `1.0` and rung 65's `2.0`.
**A sweep inheriting rung 66's constant would run at 1.5× the admissible step** — and rung 65
published a RETRACTION for exactly this failure mode, where an RK4 instability returned an
`∫b ds` 4.4× the converged value and looked like a physical finding. Rung 65's floor was a
scalar because it had one state; rung 66's was a sum because a degenerate pair's spectral
radius is the sum; this rung's is the same sum over one more term, and the derivation above is
why it is the sum and not the max.

---

## 3. THE JOINT INITIAL CONDITION — the case rung 66 escaped by accident

Rung 66 solved `(g, q)` as the two laws' simultaneous fixed point and wrote:

> THE ITERATION IS ITSELF A DIAGNOSTIC: it contracts at `|R_q C_g|`, so it converges exactly
> when `det J > 0`, and a failure to converge is the degeneracy announcing itself at `s = 0`.

It then measured that **the march opens dormant at all six corners tested** (`required(0) = 0`,
`ic_iters = 1`, residual exactly 0), so its own backstop never fired: with the fuel leg
dormant, `R_q = 0` and the iteration contracted trivially. **That escape is not available at
`n = 3.`** § 0.1 measured `v_cmd = −0.0039` already at `s = 0.005`, and rung 66 § 0 measured
`b0 = 0.037` at `Tt4_lo = 1000 K`: the **valve and the stator are BOTH live at `s = 0`**, and
they share the constraint, so their pairwise contraction factor is `|C_v·V_q| = 1` — marginal.

**Predicted: the `s = 0` fixed point is a ONE-parameter family (two live laws, one constraint),
so the iteration is order-dependent rather than divergent.** Solving `q` first drives `φ` onto
the floor, leaving `V = 0` (the dormant stop) as an immediate fixed point; solving `v` first
lands on a *different* member with `q` dormant. Both are legitimate initial conditions and they
are not the same trajectory. This is rung 66 § 0's own diagnosis — *the degeneracy's signature
at `s = 0` is non-uniqueness of the initial condition, not a stalled solve* — now firing on a
march where it is load-bearing rather than moot.

**The canonical order is DECLARED HERE, before running: `g → q → v`**, i.e. rung 66's order
with the new actuator appended last, so the rung-66 arm is reached unchanged and the stator
takes up only what the inherited pair leaves. The alternative orders are to be **reported as a
sensitivity**, never silently chosen — rung 66 § 5 measured that a ±0.01 offset in the initial
valve position alone moves the withheld fuel by 84 % and the violation integral by ±20 %.

---

## 4. Pre-registered predictions

| | prediction | why, and what would falsify it |
|---|---|---|
| **P1** | The **cyclic** product `x = R_q·C_v·V_g = −1` at every riding-interior point, to the root-finders' floor (~5e−9). | § 2's `−c·rᵀ` around a 3-cycle. FALSIFIED by any riding-interior point with `\|x+1\| ≫ 1.5·δ_manifold`. This is the one quantity the pairwise products do not already contain. |
| **P2** | The actuator block has **exactly 2 zero eigenvalues** and one at **`−(1/τ_g + 1/τ_v + 1/τ_s)`**, measured on the shipped closures across a clock grid. | § 2. Rung 66 measured the `n = 2` case to 4 significant figures (39.97 vs 40). FALSIFIED by a third non-zero root, or by a sum that tracks `max` instead. |
| **P3** *(expected to MISS)* | Rung 66 § 9's own magnitude — *the third limiter buys even less than the second's 1.59 %* — **is unsupported by rung 66's own numbers**, so the marginal credits will NOT be ordered by loop count. | Rung 66 measured `Δ_fuel = 1.59 %` (38× erosion) and `Δ_valve = 33.64 %` (2.8×) — **both doubled the rate sum**, yet they differ by 21×. So credit is not a function of `Σ1/τ`, and "the third buys least" has no mechanism behind it. **The object is the ORDERING ASYMMETRY.** Scored against `Δ_stator` (the new loop added last), with `min(Δ_fuel, Δ_valve, Δ_stator)` reported as a secondary. |
| **P4** | **`v_max` is INERT** — the loop uses ≲ 9 % of its authority (`v_cmd ≥ −0.0172` against `v_max = 0.20`), so sweeping the stop changes nothing. | § 0.1. A predicted **CONTRAST with rung 64**, whose headline is that *a limiter's LAW cannot buy PROTECTION, only its PRICE — the ceiling is the lever's AUTHORITY*. Registered now so an inert stop reads as a finding and not as a missing sweep. FALSIFIED if the ledger moves with `v_max`. |
| **P5** | The `s = 0` initial condition is a **one-parameter family**, order-dependent, converging in one pass per order to different members; and the ledger is measurably sensitive to which. | § 3. FALSIFIED if all orders land on the same `(g, q, v)`. |
| **P6** | A **saturated** loop makes the block look **MORE** independent, not less: forcing one actuator to its stop drives `det` **away** from 0 while the surviving pair keeps its own pairwise product at 1. | § 0.3's `δ = 1e−1` row, measured. So the rank test is evidence **only** under a `regime == "riding"` filter, and a reader comparing floats against the stop must fail its gate. |
| **P7** | The triple's credit has **opposite signs in the two currencies** — positive against the `φ` wall, negative against the metal (incidence) wall. | § 0.2's measured slopes (`dM_φ/dv = −0.115`, `dM_i/dv = +0.344`). FALSIFIED if the incidence-referenced ledger comes out protective too. |
| **P8** | The RK4 floor `ds·Σ1/τ_i ≤ 2.0` **binds strictly tighter** than rung 66's inherited two-clock assert, and a run at rung 66's constant is visibly unconverged. | § 2. FALSIFIED if a run between the two constants is grid-converged. |

---

## 5. Concessions (declared before the probe)

* Every one rungs 62/63/64/65/66/67 list, all inherited.
* **The `φ`-referenced stator loop moves the lever in the ANTI-PHYSICAL direction** (§ 0.2). It
  is a legitimate control law — *hold the LP flow coefficient at 0.80 with the stators* — and it
  is the law the question requires, because the rank result needs all three loops on the SAME
  constraint. It is not a recommendation, and rung 53's own critique predicts it is the wrong
  currency for this lever. Disclosed, not defended.
* `τ_s` joins `τ_v` and `τ_g` as a swept coordinate on the march's own `s`. No actuator
  bandwidth is anchored. ORDERINGS, SIGNS and INVARIANCES are the claims; every MAGNITUDE is
  disclaimed.
* `φ_lim` and `b_max` remain **IMPOSED** (rung 64's concession, verbatim); `v_max` is inherited
  from rungs 57/58 rather than derived.
* All three lags are **SYMMETRIC** except the fuel leg, which is rung 52's asymmetric one —
  rung 66's split, unchanged. Rung 65's asymmetric-valve seam and rung 67's asymmetric-governor
  seam are both still untouched.
* The stator enters only through rung 53's two derived channels (`psi`, `phi_surge_at`); the
  **stage stack** (rungs 55/56) is not on the transient ladder and is not reached here, so the
  binding-row migration rung 56 found is invisible to this rung.
* The spectrum is sampled at finitely many trajectory points — a DIAGNOSTIC that can miss a
  brief excursion (rung 65's retracted trap), not a proof of convergence.
* This rung puts **one foot** in rung 63's *fuel + bleed + stator* seam and does not close it:
  that seam wants the stator as a **SCHEDULE** (rung 57/62's open loop), and this is a closed
  loop on the same variable as the other two. The seam stays open.
