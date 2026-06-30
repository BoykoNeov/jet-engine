# Rung-2 external anchor — Mattingly *Elements of Propulsion* Example 7.1

Verified worked "real turbojet" example. Source: Jack D. Mattingly,
*Elements of Propulsion: Gas Turbines and Rockets* (2nd ed., AIAA), Example 7.1,
pp. 387–389. Every number below was re-derived by hand from the inputs to confirm
the OCR; all are self-consistent.

## Inputs
| Quantity | Value |
|---|---|
| Flight Mach `M0` | 2 |
| Ambient temp `T0` | 216.7 K |
| Cold-section `γc`, `cpc` | 1.4, 1004 J/(kg·K) |
| Hot-section `γt`, `cpt` | 1.3, 1239 J/(kg·K) |
| Fuel heating value `hPR` | 42.8 MJ/kg |
| Max inlet recovery `π_dmax` | 0.95 |
| Burner pressure ratio `π_b` | 0.94 |
| Nozzle pressure ratio `π_n` | 0.96 |
| Compressor/turbine **polytropic** eff `e_c`, `e_t` | 0.9, 0.9 |
| Burner (combustion) eff `η_b` | 0.98 |
| Mechanical (shaft) eff `η_m` | 0.99 |
| Exit pressure ratio `P0/P9` | **0.5** (under-expanded — P9 = 2·P0) |
| Turbine-inlet temp `Tt4` | 1800 K |
| Compressor pressure ratio `πc` | 10 |

Derived section gas constants: `Rc = (γc−1)/γc·cpc = 286.9`, `Rt = (γt−1)/γt·cpt = 285.9` J/(kg·K).

## MIL-spec ram recovery (inlet)
`η_r = 1 − 0.075·(M0−1)^1.35` for 1 ≤ M0 ≤ 5 (= 1 for M0 ≤ 1).
At M0=2: `η_r = 0.925`; `π_d = π_dmax·η_r = 0.95·0.925 = 0.87875`.

## Expected outputs (anchor targets)
| Quantity | Value |
|---|---|
| `a0` | 295.0 m/s |
| `V0` | 590 m/s |
| `τr` (= Tt0/T0) | 1.8 |
| `πr` | 7.82445 |
| `τλ` (= cpt·Tt4/(cpc·T0)) | 10.2506 |
| `τc` (= Tt3/Tt2) | 2.0771 |
| **isentropic** `η_c` (from `e_c`) | 0.8641 |
| fuel-air ratio `f` | 0.03567 |
| `τt` (= Tt5/Tt4, from shaft balance) | 0.8155 |
| `π_t` | 0.3746 |
| **isentropic** `η_t` (from `e_t`/`π_t`) | 0.9099 |
| `Pt9/P9` | 11.621 |
| `T9/T0` (→ T9 ≈ 833.4 K) | 3.846 |
| `V9/a0` (→ V9 ≈ 1253.8 m/s) | 4.250 |
| exit Mach `M9` | 2.253 |
| specific thrust `F/ṁ0` | 806.9 N·s/kg |
| TSFC `S` | 44.21 (mg/s)/N = 4.421e-5 kg/(N·s) |
| thermal eff `η_T` (KE/fuel basis) | 41.92% |
| propulsive eff `η_P` | 74.39% |
| overall eff `η_O` | 31.18% |

## Polytropic → isentropic conversion (exact for a calorically perfect gas)
Our rung-2 code uses **isentropic** component efficiencies; Mattingly inputs
**polytropic**. The conversion is exact:

- Compressor: `η_c = (πc^(Rc/cpc) − 1) / (πc^(Rc/(cpc·e_c)) − 1)`.
  At πc=10, e_c=0.9 → `η_c = 0.8641`.
- Turbine: with `τt` fixed by the shaft balance and `π_t = τt^(γt/((γt−1)e_t))`,
  the isentropic `η_t = (1 − τt)/(1 − τt^(1/e_t)) = (1 − τt)/(1 − π_t^((γt−1)/γt))`.
  → `η_t = 0.9099` (at τt = 0.8156).

So the anchor test inputs `η_c = 0.8641`, `η_t = 0.9099` (isentropic) and must
reproduce the table above. The test derives `η_t` at run time from a provisional
pass (τt is independent of η_t — the shaft sets the drop), so it stays exact.

## Scope implication (the under-expanded nozzle)
`P0/P9 = 0.5` ⇒ the nozzle does **not** fully expand and the specific thrust carries
a pressure term:

```
F/ṁ0 = (a0/gc)[ (1+f)(V9/a0) − M0 + (1+f)·(Rt/Rc)·(T9/T0)/(V9/a0)·(1 − P0/P9)/γc ]
```

Matching this requires the rung-2 nozzle to expand to a **specified exit static
pressure P9** (parameter; default P9 = P0 → fully expanded, reduces to rung-1) and
the thrust equation to include the pressure term. This is a *specified pressure
ratio*, **not** choked-nozzle detection — no new solver control flow. The choked
convergent nozzle (choke detection + branch) stays deferred to a later rung.
