# Rung 68 — THREE LOOPS ON ONE VARIABLE

**A lagged STATOR limiter beside rung 65's lagged VALVE and rung 52's lagged FUEL leg, all
three holding `φ_lp` to the same `φ_lim`.** Five states, three clocks. Rung 66's standing seam.

> **HEADLINE — `n` LOOPS ON ONE VARIABLE ARE ONE LOOP WITH ALL `n` RATES ADDED.** `n` laws
> holding the same variable to the same set point have `∂U_i/∂u_j = −φ_j/φ_i` **uniformly** —
> at `j = i` that formula returns `−1` by itself, so the diagonal is not a special case — hence
> `J = −D·c·rᵀ` is **RANK ONE** at every `n`, every plant, every gain, every bandwidth: `n − 1`
> zero eigenvalues and one root at `−Σ 1/τ_i`. Rung 66's `det J ≡ 0` was never a property of
> PAIRS.

> **AND IT EXTENDS RUNG 64.** `v_max` — the lever's AUTHORITY, which rung 64 made *the* ceiling
> on protection — is **inert to the solver's own resolution on the triple and decisively
> binding on the same lever alone**. So authority is not a property of a lever; it is a
> property of the lever *plus whatever else holds the same variable*.

---

## 0. WHAT THE `n ≥ 3` CONTENT ACTUALLY IS — and what it is not

Rung 66's identity is `R_q·C_g ≡ 1`, one scalar. Stating it three times over three pairs is
**not** a test of joint collapse. Imposing all three pairwise identities on the block leaves
one free parameter:

    M = [[−1, a, b], [c, −1, d], [e, f, −1]],   ac = be = df = 1,   x := a·d·e = R_q·C_v·V_g
    ⇒   det M  =  2 + x + 1/x  =  (x + 1)² / x

**A block can be pairwise-degenerate and still rank 2.** Only the CYCLIC product `x` tests the
rank, and its predicted value is `−1` (three factors of `−φ_j/φ_i`). Everything else a reader
might quote is a re-expression of something already counted:

| quantity | what it really is |
|---|---|
| `tr M = −3` | the **hardcoded diagonal**. Not a measurement at all. |
| `Σ`(2×2 principal minors)` = 3 − Σ`(pairwise products) | rung 66's identity, three times |
| `det M` | a monotone function of `x` alone — see above |

So this spec quotes **`x`**, and `tests/test_rung68.py` carries a gate (`test_the_cyclic_
product_is_not_implied_by_the_pairwise_ones`) that hand-builds a block with all three pairs at
1 and `x = −3.5`, confirming `det ≠ 0`. Without that gate, § 2's measurement would be a
tautology of the kind this project has caught itself in three times.

**This is a correction to how rung 66's result should be read**, not to the result: rung 66
measured the only independent product its pair HAD.

---

## 1. The plant — five states, three clocks

Spools `ν_lp, ν_hp` (rung 40), the fuel-clip amount `g` (rung 52), the valve position `q`
(rung 65), and the LP stator setting `v`:

    dg/ds = ( R(ν, q, v) − g ) / lag.tau(R, g)    R = rung 52's required clip     [FUEL]
    dq/ds = ( C(ν, g, v) − q ) / τ_v              C = rung 65's b_cmd             [VALVE]
    dv/ds = ( V(ν, g, q) − v ) / τ_s              V = the setting putting φ on the floor

### The third lever, and the direction it has to move

`V` is **one-sided**, like every limiter in this family: `v ∈ [−v_max, 0]`, with `v = 0` the
**dormant** stop (the design setting) and `−v_max` the saturated one. Measured (`φ_lp` against
a trial setting at `s = 0.29`):

| `v` | −0.20 | −0.10 | −0.05 | −0.02 | 0.00 | +0.02 | +0.05 | +0.10 | +0.20 |
|---|---|---|---|---|---|---|---|---|---|
| `φ_lp` | 0.8949 | 0.8430 | 0.8196 | 0.8063 | 0.7978 | 0.7894 | 0.7773 | 0.7581 | 0.7230 |

**`dφ_lp/dv ≈ −0.42`: closing the stators LOWERS `φ_lp`, so a `φ`-referenced loop must OPEN
them.** A real VSV schedule closes at low corrected speed, and it does so for a reason rung 53
published — closing lowers the *wall* `φ_surge(v) = 1/(T_c+v)` faster than it lowers `φ`. A
`φ`-referenced loop cannot see the wall. Two derived consequences (`T_c = 1.8182`, `φ ≈ 0.798`):

    dM_φ/dv = dφ/dv − dφ_surge/dv = −0.4176 + 0.3025 = −0.115
    dM_i/dv = (1/φ²)·dφ/dv + 1    = −0.6563 + 1     = +0.344

**The loop protects `φ` and ERODES incidence margin** — § 4 measures what that does to a
ledger. `_solve_v` therefore has **both clamp tests and its bracket orientation inverted**
relative to `_solve_b`, and it returns a three-way regime label for the reason § 5 gives.

### The two traps

* **THE REFERENCE.** `V` is rooted on the **running-line `φ_lp`** through `with_vsv`. Root it
  on the moved wall through `phi_surge_at` and this is rung 60's *incidence* loop by accident:
  the constraint stops being shared, `x ≠ −1`, the rank comes out 2, **and nothing fails.**
  Rung 62's `_powers` trap, fourth reload.
* **THE `_b_state`/`_v_state` BOUNDARY**, generalised from rung 66. A law that TRIALS an
  actuator must not see that actuator's state, and must see the other two:

      R (fuel)   `_b_state = q` and `_v_state = v`        — it trials neither
      C (valve)  `_v_state = v`, `_b_forced` trials `b`   — NOT `_b_state`
      V (stator) `_b_state = q`, `_v_forced` trials `v`   — NOT `_v_state`

**A bug this actually caught.** Rung 67's `tau_gov` rides on an instance attribute and
`ScheduledStatorTransient._stator_march` — the method every reader in this family calls — does
not forward it as a keyword. The first version of `integrate_fuel` read only the argument, so
a rung-68 march would have **accepted `tau_gov` and silently ignored the governor**, with the
refusal below never firing. Found by the gate that asserts the refusal, not by inspection.

---

## 2. THE DERIVATION — `J = −D·c·rᵀ`, and it is rank one at every `n`

`n` laws each solving the **same** constraint `φ(u₁ … u_n) = φ_lim` for its own actuator.
Differentiating `φ(U_i(u_{−i}), u_{−i}) = φ_lim` in `u_j`:

    φ_i·∂U_i/∂u_j + φ_j = 0     ⇒     ∂U_i/∂u_j = −φ_j/φ_i          [uniformly in i, j]

    M := [∂U_i/∂u_j − δ_ij] = −c·rᵀ ,  c_i = 1/φ_i , r_j = φ_j
    J  = D·M = −D·c·rᵀ ,               D = diag(1/τ_i)

Rank one, so `n − 1` zeros and one non-zero root equal to the trace:

    tr J = −Σ_i (1/τ_i)·c_i·r_i = −Σ_i 1/τ_i          [c_i r_i = 1 for every i]

At `n = 2` this reproduces rung 66's `{0, −(1/τ_g + 1/τ_v)}` exactly.

### Measured — the six cross-gains on three mutually ignorant shipped closures

`_surge_fuel` (fuel), `_solve_b` (valve) and `_solve_v` (stator). None knows the others exist,
which is what makes their products a measurement rather than a restatement. 105 riding-interior
points on the five-state march, `s ∈ [0.005, 0.525]`:

| `s` | `R_q` | `C_g` | `R_v` | `V_g` | `C_v` | `V_q` |
|---|---|---|---|---|---|---|
| 0.005 | −5.736e−2 | −17.433 | +3.229e−2 | +30.97 | +0.5629 | +1.7764 |
| 0.305 | −7.588e−2 | −13.179 | +4.479e−2 | +22.32 | +0.5904 | +1.6939 |
| 0.505 | −9.503e−2 | −10.523 | +6.212e−2 | +16.10 | +0.6537 | +1.5297 |

The individual gains move by a factor of ~1.9 across the march. The three pairwise products
hold at `1.00000000` and the **cyclic product at `−1.0000000000 … −1.0000000007`** — worst
departure `6.68e−10`, which is the shipped **root-finders'** tolerance floor and not the
differencing truncation (halving every step four times leaves it wandering in
`[−2.6e−8, +2.6e−9]` rather than shrinking).

**Off the manifold it degrades, exactly as rung 66's did.** Read at the LIVE marched `v` —
rung 66's own choice, and where a transient actually sits — the cyclic product spans
**`[−1.0187, −0.9821]`**, a ±1.9 % departure against rung 66's ±3.5 % at `n = 2`.

### The detector's sensitivity — MEASURED, not asserted

Displacing the stator off the shared manifold by `δ`:

| `δ` | 0 | 1e−4 | 1e−3 | 1e−2 | 3e−2 |
|---|---|---|---|---|---|
| `x + 1` | −4.03e−10 | 1.467e−4 | 1.466e−3 | 1.463e−2 | 4.365e−2 |

Linear, gain **1.463**, noise floor **4.03e−10** ⇒ the instrument resolves `δ ≳ 3e−10`. A null
result is worth what its instrument can resolve and no more.

### The spectrum — two zeros, and the rates add

`tr J = −Σ1/τ_i` is the ODE's own diagonal and is **not** a measurement. What is measured is
that the other two roots vanish — equivalently that both invariants do, and they carry
**different** content:

    c1 = Σ_{i<j} (1 − a_ij a_ji)/(τ_i τ_j)      = 0  iff every PAIRWISE product is 1
    c0 = det J = (x+1)²/(x·τ_g τ_v τ_s)         = 0  iff the CYCLIC product is −1

So the `n − 1` rank deficiency **decomposes** into the three pairwise identities plus the one
cyclic identity. Measured at `ds = 0.002`, sampling every 20th interior point:

| `(τ_g, τ_v, τ_s)` | used / sampled | `−Σ1/τ` | dominant root | max \|zero\| | `c1` | `c0` |
|---|---|---|---|---|---|---|
| (0.05, 0.05, 0.05) | 13 / 14 | −60 | −60.0000 | 4.5e−7 | 3.7e−7 | 1.7e−12 |
| (0.005, 0.05, 0.05) | 19 / 19 | −240 | −240.0000 | 1.7e−6 | 3.4e−6 | 1.5e−11 |
| (0.5, 0.05, 0.05) | 7 / 8 | −42 | −42.0000 | 2.2e−7 | 4.8e−8 | 7.4e−14 |
| (0.05, 0.02, 0.10) | 12 / 13 | −80 | −80.0000 | 6.3e−7 | 6.3e−7 | 1.8e−12 |

(The dropped points are DISCLOSED, never silently truncated — see § 5.)

---

## 3. THE STABILITY FLOOR — and rung 66's constant counterfeits PERFECT PROTECTION

    assert  ds · Σ_i (1/τ_i)  ≤  2.0

At three matched clocks that reads `ds/τ ≤ 2/3`, against rung 66's `1.0` and rung 65's `2.0`.
**A sweep inheriting rung 66's constant would run at 1.5× the admissible step.**

An assert nobody has run past is a tautology, so the guard is a separate method
(`_rk4_floor`) that a gate overrides to a no-op in order to measure the band it refuses:

| `ds` | `ds·Σ1/τ` | rung 66 admits | rung 68 admits | `I` | vs finest | `min φ_lp` |
|---|---|---|---|---|---|---|
| 0.003125 | 0.188 | ✓ | ✓ | 8.9565e−4 | — | 0.795155 |
| 0.0125 | 0.750 | ✓ | ✓ | 8.9119e−4 | −0.50 % | 0.795159 |
| 0.03125 | 1.875 | ✓ | ✓ | 8.2762e−4 | −7.60 % | 0.795617 |
| **0.04** | 2.400 | **✓** | ✗ | 5.8064e−4 | **−35.2 %** | 0.797179 |
| **0.05** | 3.000 | **✓** | ✗ | **0.0** | **−100 %** | **0.800000** |

**And this is WORSE than rung 65's retraction, because it fails toward zero.** Rung 65's
instability inflated `∫b ds` 4.4× and looked like a physical finding (*a fast valve bleeds
more*). Here, at a step rung 66's own constant admits, the march reports the floor **exactly
held** — `min φ_lp = 0.800000`, violation integral **0**. It counterfeits a limiter that works
perfectly. Rung 65's floor was a scalar because it had one state; rung 66's was a sum because a
degenerate pair's spectral radius is the sum; this is the same sum over one more term, and § 2
is why it is the sum and not the max.

---

## 4. WHAT THE TRIPLE DELIVERS — the 7-cell ledger, in BOTH currencies

Currency: rung 66's `I = ∫₀^r max(0, φ_lim − φ_lp) ds`, its `_violation` **inherited
unchanged**. Every loop is lagged, in every cell.

**WHAT THAT DOES AND DOES NOT BUY, stated precisely because the `bare` cell is the denominator
of all eight credits.** The eight cells here are built by ONE rig differing only in which loops
are armed, so they are differenceable *against each other* — which is what every number in this
section uses. Against rung 66's published table they are **close but not identical**: this
rung's `FV` cell returns `I = 1.528558e−3` against rung 66's `1.52910e−3` (inside its own
published grid band `1.52626e−3 … 1.52922e−3`) with `min φ_lp = 0.793085`, rung 66's value
exactly; but `bare` returns `2.581532e−2` against rung 66's `2.58877e−2`, **0.28 % apart**. The
residual is almost certainly a grid difference — rung 66's own § 4 reports a 0.19 % spread
across an 8× `ds` range, and the `FV` agreement lands inside that band rather than on its
value. **It is NOT reconciled here.** § 8's P3 crosses the two ledgers (2.45 % against rung
66's 1.59 %), and that comparison survives only because a 54 % gap cannot be closed by 0.28 %
— which is an argument about margin, not a claim of identity.

| cell | `I` (φ) | credit | `I` (incidence) | credit | `min φ` | `v_min` | `b` used |
|---|---|---|---|---|---|---|---|
| bare | 2.581532e−2 | — | 4.329085e−2 | — | 0.735442 | 0 | 0 |
| **F** fuel | 1.018390e−2 | 60.55 % | 1.633403e−2 | 62.27 % | 0.773116 | 0 | 0 |
| **V** valve | 1.939430e−3 | 92.49 % | 3.062688e−3 | 92.93 % | 0.789123 | 0 | 0.0900 |
| **S** stator | 2.142124e−3 | 91.70 % | 6.814711e−2 | **−57.42 %** | 0.788430 | −0.1666 | 0 |
| FV | 1.528558e−3 | 94.08 % | 2.404086e−3 | 94.45 % | 0.793085 | 0 | 0.0789 |
| FS | 3.261458e−3 | 87.37 % | 5.101011e−2 | **−17.83 %** | 0.790284 | −0.1577 | 0 |
| VS | 1.022884e−3 | 96.04 % | 1.727632e−2 | 60.09 % | 0.793448 | −0.0497 | 0.0647 |
| **FVS** | **8.952178e−4** | **96.53 %** | 1.621330e−2 | 62.55 % | 0.795155 | −0.0436 | 0.0613 |

    sum of the three standalone credits    244.74 %
    delivered by the TRIPLE                 96.53 %

The triple beats every single and every pair — so "three protections that each work, together
fail" is false — and it is **strongly sub-additive**, as § 2 requires: rank one means the trio
has ONE effective actuator direction.

### All three marginals, quoted because the ORDERING is the object

| added LAST | marginal (φ) | that loop alone | erosion | marginal (incidence) |
|---|---|---|---|---|
| the FUEL leg | **0.495 %** | 60.55 % | **122.4×** | +2.46 % |
| the VALVE | 9.166 % | 92.49 % | 10.1× | +80.38 % |
| the STATOR | 2.453 % | 91.70 % | 37.4× | **−31.90 %** |

**Rung 66 § 9's magnitude prediction is a MISS, and predictably so.** It expected the third
limiter to buy *less than the second's 1.59 %*. The stator buys **2.45 %** — more — while the
fuel leg, added last, buys **0.495 %** — less. Credit is not ordered by loop count and is not a
function of `Σ1/τ`: rung 66's own two marginals (1.59 % and 33.64 %) **both doubled the rate
sum** and differed by 21×. The erosion factors here span 12× across the three orders.

### The credit FLIPS SIGN between the two walls

The stator moves the `φ` wall and leaves the metal one alone. Measured against the incidence
wall the stator does **not** move, the stator-alone cell scores **−57.4 %** — it is worse than
running no limiter at all — and its marginal contribution is **−31.9 %**. The valve, which
moves neither wall, keeps its sign in both.

**So rung 53's *a margin is a DISTANCE* has landed on a ledger.** Rung 53 bounded rungs 36–52's
*currency*; rung 54 their *constraint severity*; rung 56 a *lever's cost*. Here it reaches the
**sign of a protection credit**: the same loop, the same march, the same set point, protective
in one wall and harmful in the other. A credit quoted without its wall is meaningless.

### `v_max` — inert in company, binding alone

| `v_max` | `I` (FVS) | saturated? | `I` (S alone) | saturated? |
|---|---|---|---|---|
| 0.02 | 1.212741e−3 | **yes** | 2.231618e−2 | yes |
| 0.05 | 8.952178e−4 | no | 1.695658e−2 | yes |
| 0.10 | 8.952178e−4 | no | 8.686789e−3 | yes |
| 0.20 | 8.952178e−4 | no | 2.142124e−3 | no |
| 0.40 | 8.952178e−4 | no | 2.142124e−3 | no |

On the triple the ledger is identical to ~1e−15 across a 4× ceiling (not bit-for-bit: `v_max`
is one end of `_solve_v`'s bracket, so it moves `_illinois`'s first secant — disclosed rather
than tuned away). On the same lever ALONE the same ceiling is decisive: **7.9×** on `I` over
that same 0.05 → 0.20 range, and **10.4×** over 0.02 → 0.20.

**This EXTENDS rung 64 rather than contradicting it.** Rung 64's *the ceiling is the lever's
AUTHORITY* is a statement about a lever alone. Put two other loops on the same variable and
they take up the demand before the stop is reached, so the ceiling stops binding — and it
starts again once the ceiling is tight enough (`v_max = 0.02`), which is what makes the
inertness a measurement rather than an absence.

---

## 5. THE SATURATION CONFOUND — a stop costs the block a ZERO

A loop on its stop has `∂U/∂u_j = 0` for every `j`. It contributes a row of zeros, keeps only
its own bare `−1/τ`, and **at most one zero can survive**, from the remaining pair. What the
observable is depends on where the point sits, and only one of the two is reachable on a march:

* **Exactly on the shared manifold** the surviving pair is exact, so `det = −1 + ac = 0` and
  the triple reads as a degenerate **PAIR** — one zero instead of two.
* **Off the manifold**, where a transient always is, the surviving pair's own identity has
  degraded too, so `det ≠ 0` as well and the block reads as a **fully INDEPENDENT** triple.

Measured at `v_max = 0.02`, both from the same march:

| point | `V_g` | `V_q` | `R_q·C_g` | roots | zeros |
|---|---|---|---|---|---|
| **saturated** | **0.0** | **0.0** | 0.9869 | `[−39.87, −20.00, −0.132]` | **0** |
| riding | 19.68 | 1.640 | 1.0048 | `[−60.07, 0.013, 0.053]` | **2** |

`−20.00 = −1/τ_s` is the saturated actuator standing alone. **The practical counterfeit is
INDEPENDENCE**, and it is the exact inverse of rung 67's lesson: there a stop faked the absence
of COUPLING in one entry; here a stop fakes the absence of REDUNDANCY in the whole block.

That is why **every gain and spectrum reader filters on the regime label `_solve_v`/`_solve_b`
return, never on a float comparison against a stop** — and why the filter checks the twelve
PERTURBED evaluations, not merely the base point. Measured cost of checking only the base
point: `c1`, which § 2 predicts ~0, came back at **1.3e+2** on a handful of edge points where
one arm of a central difference had crossed the `max(0, ·)` kink, while the interior points sat
at 1e−7. Points dropped by the filter are counted and reported; a silent drop is a coverage
claim.

---

## 6. THE INITIAL CONDITION — the case rung 66 escaped by accident

Rung 66 wrote that its joint solve *"converges exactly when `det J > 0`, and a failure to
converge is the degeneracy announcing itself at `s = 0`"*, then measured that its march opens
DORMANT at all six corners tested — so `R_q = 0`, the contraction was trivially 0, and its own
backstop never fired. **That escape is gone at `n = 3`:** the valve is live at `s = 0` (rung 66
measured `b0 = 0.037`) and so is the stator, and those two SHARE the constraint, so their
contraction factor is `|C_v·V_q| = 1` — marginal. The `s = 0` fixed points are a **CURVE**.

The declared starting member is rung 66's (`g = 0`, `q = b_cmd(0)`, `v = 0` — the stator's own
dormant stop), and from it **every one of the six Gauss-Seidel orders lands on the same member
in ONE iteration with residual exactly 0** (`g0 = 0`, `b0 = 0.036626`, `v0 = 0`). So the ORDER
is not the lever — a **SPLIT** against the anchor's P5, which expected it to be.

The family shows up when the START moves, which is rung 66 § 0's own diagnosis (*the
degeneracy's signature at `s = 0` is non-uniqueness of the initial condition, not a stalled
solve*). At rung 66's own ±0.01 offsets:

| | rung 66 (`n = 2`) | rung 68 (`n = 3`) |
|---|---|---|
| violation integral spread | ±20 % | **45.2 %** |
| withheld-fuel spread | 84 % | **105.5 %** |

**Initialising all three at zero instead lands on a different member entirely** (`g0 = 2.0e−3`
against rung 66's exact 0, the fuel leg taking the whole clip) and moves `min φ_lp` in the
fifth figure. The member is therefore DECLARED in the integrator and the alternatives are
REPORTED by `ic_family`, never silently chosen.

This rung reports the growth 84 % → 105.5 % as a MEASUREMENT and does **not** attribute it to
the second zero eigenvalue: `n = 3` also has one more actuator free to move, and separating
those needs a non-degenerate triple to compare against, which § 0's scope forbids.

---

## 7. Reduce contract

* `stator_lim=None` ⇒ **rung 67/66 bit-for-bit, by dispatch** — the five-state integrator is
  never entered, `_arm` returns before it can touch a map, and the state count is 4.
* Every inherited arm leaves through the same `super().integrate_fuel(…)` and is bit-for-bit:
  rung 65 (`lag=None`), rung 52 (no valve), rung 64 (no clocks), rung 62 (a bleed schedule).
* A `StatorLimiter` with `tau=None` is **refused**, not silently dropped — an instantaneous
  stator loop is a different object (rung 66's discipline: a lagged loop against an
  instantaneous one is not a control but a different plant).
* **The converging limit is `τ_s → ∞`, NOT `τ_s → 0`**, and that INVERTS every earlier lag in
  this family. Rungs 65/66 send a clock to zero to recover the loop's instantaneous version, so
  there the FAST limit is the richer object. A third loop is an ADDITION, so only the SLOW
  limit removes it. Measured against rung 66's `I = 1.528558e−3`: `τ_s =` 0.5 / 2 / 10 / 50 /
  500 / 5000 give −6.62 % / −1.74 % / −0.35 % / −0.071 % / −0.007 % / −0.001 %, monotone, with
  `min φ_lp → 0.793085` exactly. `τ_s → 0` runs the **other way**, to −88 %. Neither limit is
  bit-for-bit; both are REPORTED per clock and never asserted to zero.

---

## 8. Predictions, scored

The anchor (`docs/plans/rung68-anchor-three-loops.md`) pre-registered eight.

| | prediction | verdict |
|---|---|---|
| **P1** | the CYCLIC product `= −1` at every riding-interior point, to the root-finders' floor | **HIT.** Worst departure 6.68e−10 against a measured 4.03e−10 floor; the detector resolves `δ ≳ 3e−10`. |
| **P2** | exactly 2 zero eigenvalues, and one root at `−(1/τ_g + 1/τ_v + 1/τ_s)` | **HIT** across four clock arms (−60, −240, −42, −80), max \|zero\| ≤ 1.7e−6. **Sharpened in the reporting**: `tr J` is the hardcoded diagonal, so what is measured is `c1 = 0` (the three pairwise identities) and `c0 = 0` (the cyclic one), and the rank deficiency DECOMPOSES into exactly those two. |
| **P3** *(expected to MISS)* | rung 66 § 9's *the third buys even less than 1.59 %* is unsupported; the ORDERING ASYMMETRY is the object | **HIT on the prediction, so rung 66's seam MISSES.** The stator added last buys **2.45 %** (> 1.59 %), the fuel leg added last buys **0.495 %** (< 1.59 %). Erosion spans 10.1× … 122.4×. |
| **P4** | `v_max` INERT, ≲ 9 % of authority used | **SPLIT, and the split is the finding.** Inert on the TRIPLE (identical to 1e−15 across a 4× ceiling) but the *number* was wrong — the loop uses 22 %, not ≲9 %, because in the real five-state march it carries a real share of the floor rather than the residue § 0.1 measured on a rung-66 trajectory. And ALONE the same ceiling is decisive (10× on `I`), which the anchor did not anticipate at all. **Rung 64 EXTENDED, not merely contrasted.** |
| **P5** | the `s = 0` IC is a one-parameter family, ORDER-dependent | **SPLIT.** The family is real (a moved start gives 45.2 % / 105.5 % spreads), but the ORDER is not the lever: from the declared start all six orders land on one member in one iteration. The anchor's own § 3 argument was right about the curve and wrong about which knob moves along it. |
| **P6** | a saturated loop makes the block look MORE independent, `det` away from 0, *while the surviving pair keeps its product at 1* | **HIT on the claim, MISS on the stated mechanism.** `det` does move away from 0 (measured `c0 = −105`, zero roots near zero) — but *because* the surviving pair does NOT keep its product at 1 (0.9869 off-manifold). On the manifold the pair is exact and `det` stays 0, giving one zero rather than none. The mid-course "correction" to *det stays 0* was itself wrong; both branches are now stated in § 5. |
| **P7** | the credit has OPPOSITE SIGNS in the two currencies | **HIT, and larger than written.** Stator alone: +91.70 % in `φ`, **−57.42 %** in incidence. Marginal stator: +2.45 % vs **−31.90 %**. |
| **P8** | the RK4 floor binds strictly tighter and a run at rung 66's constant is visibly unconverged | **HIT, and the failure mode is the opposite of the one expected.** It does not blow up like rung 65's; at `ds = 0.05` it reports the floor EXACTLY held with a violation integral of **zero** — a counterfeit of perfect protection, which is harder to catch. |

---

## 9. Concessions

* Every one rungs 62/63/64/65/66/67 list, all inherited.
* **The `φ`-referenced stator loop moves the lever in the ANTI-PHYSICAL direction** (§ 1) and
  erodes incidence margin while protecting `φ` (§ 4). It is a legitimate control law — *hold
  the LP flow coefficient at 0.80 with the stators* — and it is the law the rank question
  requires, since that question needs all three loops on the SAME constraint. **Disclosed, not
  defended, and not a recommendation.**
* `τ_s` joins `τ_v` and `τ_g` as a swept coordinate on the march's own `s`. No actuator
  bandwidth is anchored anywhere in this family. ORDERINGS, SIGNS and INVARIANCES are the
  claims; every MAGNITUDE is disclaimed.
* `φ_lim` and `b_max` remain **IMPOSED** (rung 64, verbatim); `v_max` is **inherited** from
  rungs 57/58's swept setting `V = 0.20` rather than derived — so § 4's inertness result is
  about a ceiling this project chose elsewhere, not one the hardware fixed.
* All three lags are **SYMMETRIC** except the fuel leg, which is rung 52's asymmetric one.
  Rung 65's asymmetric-valve seam and rung 67's asymmetric-governor seam remain untouched, and
  an asymmetric stator would be a fourth constant.
* The **STAGE STACK** (rungs 55/56) is not on the transient ladder, so rung 56's binding-row
  migration is invisible here; the stator enters only through rung 53's two derived channels
  (`psi`, `phi_surge_at`), with the parabolic loss term stator-inert as rung 53 left it.
* § 2's spectrum is sampled at finitely many trajectory points — a DIAGNOSTIC that can miss a
  brief excursion (rung 65's retracted trap), not a proof of convergence.
* The `n = 2` → `n = 3` growth in IC sensitivity (84 % → 105.5 %) is **not attributed** to the
  second zero eigenvalue (§ 6).
* This rung puts **ONE FOOT** in rung 63's *fuel + bleed + STATOR* seam and does **not** close
  it: that seam wants the stator as a SCHEDULE — an OPEN loop, state-fed — and this is a closed
  loop on the same variable as the other two. The seam stays open.

---

## 10. Next seams

* **THE REFERENCE SPLIT — does the coordinate a loop is referenced in decide whether it adds a
  ZERO or a RANK?** § 1 and § 4 are the setup and this rung deliberately does not run it: the
  SAME stator, referenced to incidence (rung 60's `IncidenceLimiter`) instead of to `φ`, solves
  a constraint the other two do not share, so § 2's derivation predicts the cyclic product
  leaves −1 and the block goes to rank 2 — one zero, not two. It is also the physically correct
  direction for the lever (`dM_i/dv = +0.344`), so it is the pairing where redundancy and
  hardware sense finally agree. **The strongest open seam in this family.**
* ~~**THREE loops on TWO variables**~~ — **CLOSED BY RUNG 70** (`docs/rung70-spec.md`). Rung
  47's `Tt4` governor replaces this rung's fuel leg beside the same valve and the same stator:
  `n = 3, m = 2`, one zero. It is the SAME seam as rung 69 § 11's *`pair_RV ≠ pair_CV`*
  approached from the other side, and both close together. The prediction above was half right —
  the block does stop being rank one — but the `P < 0` block does **not** simply superpose:
  `pair_CV` inherits this rung's identity, `pair_RC` **is** rung 67's `P`, and `pair_RV` comes
  back with the OPPOSITE SIGN, so no single scalar survives. **And rung 70 retires this rung's
  *quote `x`*:** the cyclic product equals `−pair_RC` and is structurally blind to `pair_RV`.
* **A plant with `|P| > 1`** (rung 67) — untouched.
* **An ASYMMETRIC valve** (rung 65) and an **asymmetric governor** (rung 67) — both still open.
* **Fuel + bleed + STATOR-as-a-SCHEDULE**, all three on one plant — rung 63's seam, still open
  after 64/65/66/67 and now after 68 (see § 9).
* **`n` = 4 on one variable.** § 2 is `n`-general and this rung tests `n = 3`. There is no
  fourth lever on this plant with authority over `φ_lp`, so closing it needs new hardware
  (customer bleed at station 3, a second bleed station) rather than a new law.
