# Rung 43 anchor — two-shaft fuel metering

Verified data behind `docs/rung43-spec.md`. Design: `build_two_spool_turbojet` with
`π_LPC = 3`, `π_HPC = 6`, `Tt4 = 1500 K`, flight `T0 = 250 K, p0 = 50 kPa, M0 = 0.85`,
losses `π_d=0.97, η_LPC=0.90, η_HPC=0.88, η_b=0.99, π_b=0.96, η_HPT=0.92, η_LPT=0.90,
η_m=0.99, π_n=0.98`, convergent nozzle. Maps as in `tests/test_rung41.py`.

CPG gas = self-consistent dual gas (`γ_c=1.4, cp_c=1004`, `γ_t=1.3, cp_t=1239`,
`R = (γ−1)/γ·cp`). **Every number below is on the CPG gas** — reacting-gas fuel control is
refused by construction (rung 35's concession, carried verbatim).

The fuel step is rung 35's own, `Tt4` **1250 → 1450 K**, so the comparison to rung 35 is
apples-to-apples. `ρ = τ_L/τ_H` is rung 40's clock ratio; `r = τ_fuel/τ_H` the ramp ratio.

**All numbers produced by the SHIPPED `TwoSpoolFuelTransient`** (RK4), not by the
development probes (forward Euler) — the probe values quoted during development differ in
the last digits; every sign, ordering and crossover is identical.

---

## A. Control-invariance — the reduce, and the non-tautological gate (gate 1)

Feed `ṁ_fuel = f_eq · ṁ_air,eq` of a rung-40 `Tt4`-control point to `equilibrium_fuel`;
it must return that point, through the forward **burner** (`Tt4` an OUTPUT).

| `Tt4` | `Δν_L` | `Δν_H` | `ΔTt4` | `Δπ_LPC` | `Δπ_HPC` |
|---|---|---|---|---|---|
| 1500 | +0.00e+00 | +0.00e+00 | +0.00e+00 | +0.00e+00 | +0.00e+00 |
| 1300 | +3.77e-15 | −1.44e-15 | −8.88e-16 | +6.22e-15 | −5.11e-15 |
| 1100 | −1.78e-15 | +0.00e+00 | +1.33e-15 | −2.55e-15 | +8.88e-16 |

Machine zero. Two closures, one point — and the **empirical death** of the withdrawn
framing "fuel metering breaks rung 39's `(†)`": if both controls land on the same manifold,
the control knob cannot change the coupling.

---

## B. The `r → 0` frozen step — INHERITED from rung 35 (gate 8)

Both spools held; no clock can enter, so this is **exactly `ρ`-free** (verified `==` over
`ρ` = 0.2 vs 5.0 on all three shape pairs, on `Tt4_peak`, `E_temp`, `E_lp`, `E_hp`, `f`).

| shapes | `Tt4_peak` [K] | `E_temp` | `E_lp` | `E_hp` | ratio |
|---|---|---|---|---|---|
| flow/press | 1687.0 | +0.349591 | +0.024404 | +0.079192 | **4.41×** |
| press/flow | 1693.3 | +0.354656 | +0.042429 | +0.056066 | **6.33×** |
| tilted | 1692.9 | +0.354309 | +0.031990 | +0.068035 | **5.21×** |

TIT-limited **before** surge-limited on these maps — rung 35's ordering, re-measured on two
shafts. The multiple (4.4–6.3×) is **disclosed, not tuned**; the gate asserts >4×. Which
limit binds first is map-dependent and **no TIT redline is modelled**.

---

## C. THE MECHANISM — channel isolation (gate 5)

March the fuel ramp with **one spool's speed held** at its initial value. `Tt4_peak` [K],
`r = 0.25`:

| shapes | `ρ` | both free | LP frozen | HP frozen | `ΔLP` | `ΔHP` |
|---|---|---|---|---|---|---|
| flow/press | 0.5 | 1591.2 | 1643.3 | 1627.7 | **+52.1** | +36.5 |
| flow/press | 1.0 | 1611.9 | 1643.3 | 1651.4 | +31.3 | **+39.4** |
| flow/press | 2.0 | 1625.9 | 1643.3 | 1667.3 | +17.3 | **+41.4** |
| press/flow | 0.5 | 1595.2 | 1648.5 | 1633.4 | **+53.3** | +38.2 |
| press/flow | 1.0 | 1616.3 | 1648.5 | 1657.2 | +32.2 | **+40.9** |
| press/flow | 2.0 | 1630.7 | 1648.5 | 1673.3 | +17.8 | **+42.6** |
| tilted | 0.5 | 1594.0 | 1647.1 | 1632.4 | **+53.1** | +38.4 |
| tilted | 1.0 | 1615.1 | 1647.1 | 1656.5 | +32.0 | **+41.3** |
| tilted | 2.0 | 1629.4 | 1647.1 | 1672.7 | +17.7 | **+43.3** |

1. **Freezing EITHER spool makes the overshoot WORSE — 9/9 here, 6/6 in the gate.** Both
   spools' motion *relieves* it; neither is a bystander.
2. **The share of the relief TRADES with `ρ`**, and the crossover is visible on every shape:
   at `ρ = 0.5` the LP channel is the larger, by `ρ = 1` the HP channel has overtaken it.

`ΔLP` and `ΔHP` do **not** sum to anything and are **not** calibrated weights — direction
only. *That* is why no single spool's clock can govern the overshoot: the responsibility for
quenching it is **shared and `ρ`-dependent**.

---

## D. The positive, and its CEILING (gates 6, 7)

`X = Tt4_peak − Tt4_target` [K], `flow/press`:

| `r` | `ρ`=0.25 | 0.5 | 1 | 2 | 4 | 8 | 32 | 128 | LP-frozen |
|---|---|---|---|---|---|---|---|---|---|
| 0.25 | 117.25 | 141.18 | 161.93 | 175.95 | 184.15 | 188.59 | 192.08 | 192.98 | **193.28** |
| 1.00 | 52.30 | 60.11 | 74.84 | 93.34 | 109.21 | 119.85 | 129.39 | 132.02 | **132.92** |

Same on the other two shape pairs (`ρ` = 0.25 … 8, then the ceiling):

| shapes | r | X(ρ = 0.25, 0.5, 1, 2, 4, 8) | LP-frozen |
|---|---|---|---|
| press/flow | 0.25 | 121.0 145.2 166.3 180.7 189.1 193.7 | **198.51** |
| press/flow | 1.00 | 52.7 61.0 76.5 95.9 112.6 123.8 | **137.68** |
| tilted | 0.25 | 119.8 144.0 165.1 179.4 187.8 192.3 | **197.12** |
| tilted | 1.00 | 52.7 60.7 75.8 94.7 111.0 121.9 | **135.38** |

Monotone rising in `ρ` on all three (gate 7), and **converging upward onto the LP-frozen
march** on all three (gate 6: at `ρ`=128, `X` reaches 99.8%, 99.8% and 99.8% of the ceiling
at `r`=0.25). The bound is
**structural**, not fitted: `ρ` multiplies **only** the LP ODE (`dν_L/ds = Φ_L/ρ`), so
`ρ → ∞` **IS** the LP-frozen system. The LP-frozen march is therefore `ρ`-independent
**bit-for-bit**:

```
r = 0.25:  X = 193.27628904725066   identical over rho = 0.25, 1, 7, 50
r = 1.00:  X = 132.91849305455617   identical over rho = 0.25, 1, 7, 50
```

So the worst TIT excursion a heavy LP spool can produce is computable **without marching the
LP spool at all**.

---

## E. THE NEGATIVE — the currencies are CIRCULAR (gate 9)

Best-fit exponent `q` in `r_eff = r/ρ^q`, and the residual scatter it leaves:

| shapes | `q*(E_temp_H)` | `q*(X)` | `q*(E_temp_L)` | residual on `X` |
|---|---|---|---|---|
| flow/press | 0.05 | 0.35 | 0.65 | 14.2 % |
| press/flow | 0.05 | 0.45 | 0.45 | 14.9 % |
| tilted | 0.05 | 0.35 | 0.65 | 14.7 % |

`E_temp_H` is referenced to the `ν_H` running line, `E_temp_L` to `ν_L`, and `X` to neither.
**The fitted exponent tracks the denominator**: the HP-referenced currency sits at 0.05 on
every shape while the neutral and LP-referenced ones sit far above it. So `E_temp`'s `q ≈ 0`
was **never** evidence that "the HP clock governs" — it was the reference reading itself back.
Only `X` is spool-neutral, which is why every magnitude in this document is quoted in `X`:
**the data selected the instrument, not the answer it gave.**

**Stated at its true strength.** The ordering is `q*(E_temp_H) < q*(X) ≤ q*(E_temp_L)` —
strict on two shapes and a **TIE on `press/flow`** (0.45 = 0.45). The claim gated is
therefore the **HP-referenced currency reading low against a spool-neutral one**, not a
strict three-way monotone (an earlier draft of the spec claimed the latter across all three
shapes; the shipped measurement refuted it and it was corrected).

And even on `X` there is **no collapse**: ~14–15 % residual, against the ~1–2 % a genuine
collapse reaches elsewhere in this project. Scoring the two single-spool clocks on the same
(shipped `collapse_exponent`) metric:

| shapes | res @ `q=0` (HP clock) | `q*` | res @ `q*` | res @ `q=1` (LP clock) |
|---|---|---|---|---|
| flow/press | 0.689 | 0.35 | **0.142** | 0.512 |
| press/flow | 0.705 | 0.45 | **0.149** | 0.513 |
| tilted | 0.696 | 0.35 | **0.147** | 0.516 |

The interior optimum fits **3.4×–4.9× better** than either endpoint, and **`q=0` is the
clearly WORSE of the two endpoints on every shape** — so the withdrawn claim "`q=1` (the
slow spool rate-limits) is refuted in every currency" is **false**, and is recorded as such.
If either single-spool clock were preferred it would be the **LP** one — the *opposite* of
what the HP-referenced currency's `q ≈ 0` appeared to say, which is the circularity in one
line. None of the three collapses.

*(Method note: an intermediate draft of this document scored these endpoints with an
ad-hoc binned-variance helper rather than the shipped metric, and got a spurious near-tie —
0.263 vs 0.256. The table above is the metric `collapse_exponent` actually reports, which is
also what the gate and the `main.py` panel use. Same direction, very different separation.)*

---

## F. What is deliberately NOT here

* **No effective clock ratio.** No `r_eff` is claimed, in any currency.
* **No `√det` composite clock.** `√det·√ρ` = const is a true **rung-40** Jacobian identity,
  but it is not connected to the overshoot; `q*(X)` landing near the midpoint of the two
  circular currencies is an averaging artifact, not evidence for ½.
* **No refutation of `q = 1` in general.** On `X`, `q = 0` fits *worse* than `q = 1`.
  Nothing about the exponent is currency-independent.
* **No claim of irreducible two-dimensionality.** Only *power-law* collapses were tested.
* **No calibrated channel split.** `ΔLP` + `ΔHP` sums to nothing.
* **No surge-survival claim and no TIT redline** — there is no surge line on either spool in
  transient (rung 41's is steady-only).

---

## G. The rung-40 defect found here

Rung 40's `equilibrium` chased an **absolute** `_EQ_TOL = 1e-12` while the reacting gas's own
residual noise floor is ~1e-10 (the equilibrium sub-solve inside `_close`). It therefore
converged physically in ~5 iterations and then spun to the iteration cap:

```
reacting, Tt4 = 1400:  |Phi| = 6.4e-2 -> 2.5e-11 by iteration 4, then FLAT at 1.08e-10
                       for the remaining 75 iterations -> AssertionError
```

Non-monotone in `Tt4` (1500/1450/1200 squeaked under, 1400/1300 did not) — a solver artifact,
not physics. Fixed with a **best-so-far acceptance AFTER the loop**, reached **only** by
inputs that previously *raised*, so every previously-converging input returns at the identical
iteration with identical `(ν_L, ν_H)` — rungs 40/41/42 untouched **by construction**. The
sweep is now smooth:

| `Tt4` | 1500 | 1450 | 1400 | 1300 | 1200 | 1100 |
|---|---|---|---|---|---|---|
| `ν_L` | 1.000000 | 0.974310 | 0.948932 | 0.899069 | 0.850132 | 0.801773 |
| final `\|Φ\|` | 1.3e-14 | 2.2e-13 | 2.6e-11 | 4.0e-12 | 3.9e-15 | 6.0e-13 |

The scattered floors are exactly why the hole was non-monotone.
