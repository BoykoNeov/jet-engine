# Rung-36 anchor — the surge line

Two-part anchor, mirroring rungs 31–35. Part A is the **method** (the CRS Ch. 9
surge-margin-along-the-running-line construction) and the numbers it produces on the project's own
machinery. Part B is the **reduce + finding data** — surge line off ⇒ rung 34/35, `_pi_c_map` == the
shipped `π_c`, and the two load-bearing signs (the `SM` schedule + the reinforcing compounding).

Design REFERENCE = the rung-34 choked-convergent point: `π_c=10, Tt4=1500, M0=0.85`, real losses
(`pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98`),
`nozzle_convergent=True`, on the fast `thermally_perfect` gas (the finding is gas-independent — the
surge margin lives entirely in the cold-section compressor map, which is composition-free). Maps:
rung 34's surge-realistic `ComponentMap` shapes (`surge_flow`, `surge_pressure`, `surge_tilted`),
each carrying an imposed `phi_surge` via `.with_phi_surge(·)`.

## Part A — the method (surge margin on the compressor characteristic)

Standard single-spool surge-margin construction (Cohen–Rogers–Saravanamuttoo *Gas Turbine Theory*
Ch. 9 "Prediction of performance — off-design"): the equilibrium **running line** is the locus of
matched operating points on the compressor characteristic; the **surge line** is the low-flow
stability boundary; the **surge margin** is the pressure-ratio gap between them at a common reference
(constant speed or constant corrected flow). CRS's load-bearing qualitative result for a simple
turbojet: the running line lies **closest to the surge line at low corrected speed** — low-power
operation and low-speed acceleration are the surge-critical regimes.

The map's own language (rung 32/34): each speed line is `ψ(φ) = 1 − σ(φ−1)² − l(φ−1)`, the operating
point sits at flow coefficient `φ_op` and corrected speed `n`, and

```
π_c(n, φ) = pr_c(Tt3s)/pr_c(Tt2) ,  Tt3 = Tt2·[1 + (τ_c,d−1)·ψ(φ)·n²] ,
            Tt3s = T_from_h_c(h_c(Tt2) + η_c(φ,n)·[h_c(Tt3) − h_c(Tt2)])
```

is the **same forward arithmetic** `_close_compressor` uses. The surge line is imposed as a **stall
flow coefficient `φ_surge`** (the zero-new-constant hope — stall at the loading-law peak
`φ = 1 − l/(2σ)` — is DEAD: it lands at `φ < 0` for all three surge shapes, see the spec table, so
`ψ` is monotone-rising across `φ > 0` and there is no in-range stall point to inherit).

Margins:

```
SM_N    = π_c(n0, φ_surge)/π_c,op − 1                              # constant speed (PRIMARY currency)
SM_flow = π_c(n_s, φ_surge)/π_c,op − 1 ,  n_s = φ_op·n0/φ_surge    # constant flow (CRS default)
```

### Design numbers (fast gas, `surge_flow`, `φ_surge = 0.65`)

Running-line flow coefficient and constant-speed surge margin along the throttle sweep (`M0=0.85`,
`p0=50 kPa`):

| `Tt4` | `ν=n`  | `φ_op`  | `π_c`   | `SM_N` | `SM_flow` |
|-------|--------|---------|---------|--------|-----------|
| 1500  | 1.0000 | 1.0000  | 10.000  | 0.395  | 9.68      |
| 1300  | 0.9095 | 0.9333  |  7.842  | 0.257  | 4.76      |
| 1100  | 0.8206 | 0.8741  |  6.047  | 0.163  | 2.44      |
|  900  | 0.7315 | 0.8285  |  4.578  | 0.104  | 1.35      |
|  800  | 0.6862 | 0.8134  |  3.956  | 0.085  | 1.05      |
|  700  | 0.6400 | 0.8059  |  3.402  | 0.073  | 0.87      |

`φ_op` walks from 1.000 (design) down toward the fixed `φ_surge = 0.65` as the engine throttles, so
**both** margins fall monotonically — tightest at part power. (`SM_flow` is larger in magnitude and
less physically meaningful for the transient; it is reported only to show the sign is
definition-robust.)

## Part B — reduce + finding data

**Reduce 1 — surge off ⇒ rung 34/35 bit-for-bit.** `phi_surge` defaults to `0.0`, read only by the
surge methods; the rung 31–35 suites pass unchanged (the bit-for-bit witness).

**Reduce 2 — `π_c` reproduction (non-tautological).** `_pi_c_map(n, φ_op)` == the shipped
`equilibrium` `π_c` to machine zero (observed diff `0.0` at `Tt4=1200`, `surge_flow`) — the margin is
measured on the running-line map itself, not a parallel derivation.

**Finding data — sign robustness (the load-bearing signs).**

*Schedule sign* — `SM_N(1500) → SM_N(700)`, across 3 shapes × imposed `φ_surge`, plus an
`n`-dependent floor `φ_surge(n)=φ0+k(n−1)`: **all cells `thin@low-power`** (24/24 in the probe,
including `k<0`). Direction matches CRS Ch. 9.

*Compounding (confirmation + sharpening)* — full-throttle burst to `Tt4_hi=1500`, `surge_flow`,
`φ_surge=0.65`:

| `Tt4_lo` | `SM_N` | `E0` (r→0) | `E0/SM_N` | reaches surge |
|----------|--------|-----------|-----------|---------------|
| 1400     | 0.320  | 0.020     | 0.062     | no            |
| 1200     | 0.205  | 0.055     | 0.267     | no            |
| 1000     | 0.130  | 0.080     | 0.617     | no            |
|  900     | 0.104  | 0.089     | 0.852     | no            |
|  800     | 0.085  | 0.095     | 1.110     | **yes**       |
|  700     | 0.073  | 0.098     | 1.356     | **yes**       |

`E0/SM_N` rises monotonically as start power falls (both `E0↑` and `SM_N↓`, reinforcing) — the
low-power burst is most surge-critical on **both** axes. This **confirms + sharpens** rung 34's
implicit worst case, it does **not** relocate it: rung 34's `E0` is **already** largest at low power
(`argmax` unchanged; relocation would need `E0` and `SM_N` to point *opposite* ways). The surge line's
new content is `SM_N` (varies independently of `r`; not a rescale of `E`). **The crossing
(`reaches surge`) is DISCLAIMED**: `E0` is floor-independent, so the crossing slides with `φ_surge` —
at `Tt4_lo=700`, `E0=0.098` fixed but `SM_N=0.109` (`φ_surge=0.55`, no surge) vs `0.073`
(`φ_surge=0.65`, surge). Only the monotone rise is gated.

**Currency equivalence (airtight).** `reaches_surge == (φ_step ≤ φ_surge)` at every tested point
(both columns agree row-for-row) — certifying `E0 ≥ SM_N ⇔ φ_step ≤ φ_surge`, i.e. `SM_N` is the
exact currency the constant-speed excursion consumes.

## What this anchor deliberately does NOT establish

- No surge-margin **magnitude** and no **crossing** into surge (both ride on the imposed `φ_surge`).
- No `n`-dependence of the surge line beyond the robustness check; no hardware `(ṁ, π)` surge-line
  shape.
- No bleed-valve / variable-stator model (the devices that raise `φ_surge` at low speed).
- Choked branch only (below nozzle unchoke the surge margin is out of scope).
