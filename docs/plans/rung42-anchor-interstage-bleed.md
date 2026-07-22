# Rung 42 anchor — interstage bleed

Verified data behind `docs/rung42-spec.md`. Design: `build_two_spool_turbojet` with
`π_LPC = 3`, `π_HPC = 6`, `Tt4 = 1500 K`, flight `T0 = 250 K, p0 = 50 kPa, M0 = 0.85`,
losses `π_d=0.97, η_LPC=0.90, η_HPC=0.88, η_b=0.99, π_b=0.96, η_HPT=0.92, η_LPT=0.90,
η_m=0.99, π_n=0.98`, convergent nozzle. Maps as in `tests/test_rung41.py`.

CPG gas = self-consistent dual gas (`γ_c=1.4, cp_c=1004`, `γ_t=1.3, cp_t=1239`,
`R = (γ−1)/γ·cp`); "fast" gas = `Gas.thermally_perfect()`; "reacting" =
`Gas.reacting_equilibrium()`.

`π* = γ_c^(γ_c/(γ_c−1)) = 1.4^3.5 = 3.246750`.

---

## A. The trade at fixed `Tt4` (probe 1) — `flow/press` shapes, fast gas, `φ_surge = 0.55`

| `Tt4` | `b` | `dφ_L` | `dφ_H` | `dSM_L` | `dSM_H` | `dF` | `dTSFC` | `dπ_LPC` | `dπ_HPC` | `dTt25` |
|---|---|---|---|---|---|---|---|---|---|---|
| 1500 | 0.05 | +3.675 % | +0.340 % | +2.59 pp | +1.83 pp | −5.00 % | +3.01 % | −4.65 % | +2.05 % | −1.44 % |
| 1500 | 0.10 | +7.664 % | +0.696 % | +5.53 pp | +3.76 pp | −10.10 % | +6.44 % | −9.20 % | +4.19 % | −2.87 % |
| 1100 | 0.10 | +9.569 % | +0.318 % | +2.85 pp | +1.49 pp | −11.41 % | +9.70 % | −6.83 % | +2.89 % | −2.27 % |
| 900 | 0.10 | +10.484 % | +0.111 % | +1.98 pp | +0.79 pp | −14.00 % | +13.99 % | −5.60 % | +2.20 % | −1.92 % |

Signs are **shape-robust**: `dφ_L > 0`, `dφ_H > 0`, `dSM_L > 0`, `dSM_H > 0`, `dF < 0`,
`dTSFC > 0` at every point of `{flow/press, press/flow, tilted, steep, flat} × {1500, 1100,
900}`. The `dφ_L / dφ_H` ratio grows monotonically with throttle-down on every shape.

---

## B. `s_H` under the valve vs rung 41's closed form (probe 2) — CPG + FLAT, `db = 0.02`

`s_H(π) = k(1 − π^(−1/k)) − 1`, `k = γ_c/(γ_c−1) = 3.5`.

| `Tt4` | `π_HPC` | `x_H` | `dlnφ_H` | `dlnx_H` | `s_H` meas | `s_H` closed | `dlnφ_L` | ratio |
|---|---|---|---|---|---|---|---|---|
| 1500 | 6.0000 | 3.7188 | +0.00235 | +0.00581 | +0.4039 | +0.4023 | +0.01961 | 8.4 |
| 1300 | 5.1884 | 3.3570 | +0.00164 | +0.00522 | +0.3138 | +0.3134 | +0.02027 | 12.4 |
| 1100 | 4.4060 | 2.9626 | +0.00095 | +0.00459 | +0.2081 | +0.2088 | +0.02106 | 22.0 |
| 1000 | 4.0280 | 2.7520 | +0.00063 | +0.00425 | +0.1480 | +0.1494 | +0.02150 | 34.2 |
| 900 | 3.6603 | 2.5318 | +0.00032 | +0.00390 | +0.0821 | +0.0842 | +0.02199 | 68.6 |
| 800 | 3.3038 | 2.3013 | +0.00003 | +0.00354 | +0.0097 | +0.0124 | +0.02253 | 659.0 |
| 750 | 3.1301 | 2.1820 | −0.00010 | +0.00335 | −0.0294 | −0.0263 | +0.02282 | −231.6 |
| 700 | 2.9597 | 2.0599 | −0.00022 | +0.00316 | −0.0705 | −0.0670 | +0.02313 | −103.8 |
| 650 | 2.7926 | 1.9349 | −0.00034 | +0.00297 | −0.1139 | −0.1100 | +0.02345 | −69.4 |
| 620 | 2.6941 | 1.8585 | −0.00040 | +0.00285 | −0.1410 | −0.1369 | +0.02365 | −58.9 |

Max |Δ| = 0.0041. `x_L = Tt4/Tt2` asserted **exactly** equal (`==`) at every row.

Shaped maps (`flow/press`) and the variable-`cp` gas loosen the agreement (`s_H` meas 0.269
vs 0.402 closed at design on shaped; 0.351 on TPG+flat) — rung 41's own disclaimer that `(★)`
is a CPG + flat-map statement. The **sign** and the ordering survive on all three.

---

## C. The `π*` sign crossing (probe 3) — CPG + FLAT, `db = 0.02`

| `Tt4` | `π_HPC` | `π_HPC − π*` | `dlnφ_H` | `dlnφ_L` |
|---|---|---|---|---|
| 820 | 3.37411 | +0.12736 | +8.94e-5 | +0.022416 |
| 800 | 3.30376 | +0.05702 | +3.42e-5 | +0.022528 |
| 790 | 3.26878 | +0.02203 | **+7.0e-6** | +0.022585 |
| 780 | 3.23391 | −0.01283 | **−1.98e-5** | +0.022643 |
| 770 | 3.19918 | −0.04757 | −4.64e-5 | +0.022701 |
| 750 | 3.13009 | −0.11665 | −9.85e-5 | +0.022819 |

Linear interpolation of the zero: `π_HPC ≈ 3.260` → **+0.40 %** above `π*`; rung 41's own
kill test isolated the identical residual (+0.44 %) as the **fuel fraction**.

---

## D. HP running line — bleed-invariant AS A CURVE (probe 2), CPG, `b = 0.10`

Take the bled point's `x_H`, root-find the `b = 0` throttle `Tt4'` with the same `x_H`,
compare `φ_H`. LP contrast is at the **same `x_L`** (which bleed leaves exactly invariant).

| shapes | `Tt4` (bled) | `Tt4'` (`b`=0) | `x_H` | `Δφ_H` | LP `Δφ_L` at same `x_L` |
|---|---|---|---|---|---|
| flat | 1400 | 1456.16 | 3.642126 | **+0.0123 %** | **+11.17 %** |
| flat | 1100 | 1133.96 | 3.032062 | **+0.0160 %** | **+11.81 %** |
| flat | 900 | 922.44 | 2.582074 | **+0.0142 %** | **+12.33 %** |
| flow/press | 1400 | 1456.16 | 3.642126 | +0.0081 % | +8.22 % |
| flow/press | 1100 | 1133.96 | 3.032062 | +0.0104 % | +9.63 % |
| flow/press | 900 | 922.44 | 2.582074 | +0.0096 % | +10.52 % |

Contrast ≈ **700–1300×**. Note `Tt4'` differs from `Tt4` by ~4 % (and `f` with it), so the
collapse is a genuine numerical statement about the real gas, not the CPG algebra.

---

## E. Self-targeting in φ-space (probe 4) — `b = 0.10`

`flow/press`, `φ_surge = 0.55`:

| `Tt4` | `φ_L` | gap | `Δφ_L` | frac closed | `φ_H` | gapH | `Δφ_H` | frac (HP) |
|---|---|---|---|---|---|---|---|---|
| 1500 | 1.0000 | 0.4500 | +0.0776 | 17.2 % | 1.0000 | 0.4500 | +0.00806 | 1.79 % |
| 1300 | 0.9059 | 0.3559 | +0.0787 | 22.1 % | 0.9755 | 0.4255 | +0.00591 | 1.39 % |
| 1100 | 0.8153 | 0.2653 | +0.0785 | 29.6 % | 0.9528 | 0.4028 | +0.00367 | 0.91 % |
| 950 | 0.7532 | 0.2032 | +0.0776 | 38.2 % | 0.9394 | 0.3894 | +0.00202 | 0.52 % |
| 900 | 0.7342 | 0.1842 | +0.0773 | 42.0 % | 0.9360 | 0.3860 | +0.00149 | 0.39 % |
| 850 | 0.7163 | 0.1663 | +0.0769 | 46.3 % | 0.9335 | 0.3835 | +0.00097 | 0.25 % |

Same monotone pattern on `tilted` and at `φ_surge ∈ {0.50, 0.55, 0.60}`. `Δφ_L` spread over
the band is **±1.2 %** (`flow/press`) and **+2.5 %** (`tilted`); `Δφ_H` falls **×8.3**.

**The confounded version, recorded so it is not used:** relative `SM_L` gain +23.2 % (1500) →
+52.6 % (850) while **absolute** `ΔSM_L` **shrinks** 0.0560 → 0.0178 pp. The relative figure
grows because the base collapses — not a controlled comparison.

---

## F. Envelope + margins (probe 3)

Lowest choked `Tt4` (CPG):

| shapes | `b`=0.00 | 0.05 | 0.10 | 0.15 |
|---|---|---|---|---|
| flat | 605 | 610 | 620 | 630 |
| flow/press | 620 | 625 | 635 | 640 |

Margin + trade at `b = 0.10`, `φ_surge = 0.55`, CPG:

| shapes | `Tt4` | `SM_L` | `SM_H` | `dF` | `dTSFC` |
|---|---|---|---|---|---|
| flow/press | 1500 | 0.2413 → 0.2973 | 0.6895 → 0.7281 | −10.00 % | +6.25 % |
| flow/press | 1100 | 0.0804 → 0.1089 | 0.4749 → 0.4910 | −11.21 % | +9.18 % |
| flow/press | 900 | 0.0408 → 0.0605 | 0.3896 → 0.3983 | −13.56 % | +12.94 % |
| flow/press | 850 | 0.0340 → 0.0518 | 0.3711 → 0.3784 | −14.65 % | +14.59 % |
| tilted | 1500 | 0.3123 → 0.3810 | 0.4855 → 0.5150 | −9.95 % | +6.24 % |
| tilted | 850 | 0.0437 → 0.0643 | 0.2510 → 0.2561 | −14.82 % | +14.71 % |

---

## G. Reduce (probe 3)

`bleed = 0` vs rung 39 `TwoSpoolMapMatcher.match`, `==` on
`(π_LPC, π_HPC, φ_L, φ_H, thrust, ṁ_air)`:

* fast gas, `Tt4 ∈ {1500, 1100, 900}` — **True**
* **reacting** gas (`Gas.reacting_equilibrium`), `Tt4 ∈ {1500, 1200}` — **True**

Reacting gas, `Tt4 = 1100`, `b = 0.08` (`π_HPC` = 4.3894): `dφ_L = +7.509 %`,
`dφ_H = +0.248 %`, `dF = −9.09 %` — same signs as the fast/CPG gas.
