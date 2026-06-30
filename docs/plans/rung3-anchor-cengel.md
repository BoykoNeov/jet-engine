# Rung-3 external anchors — Çengel air-table Brayton + Mattingly gas-table examples

Verified worked examples that anchor the **variable-`cp(T)` machinery** of rung 3:
the gas property interface (`h`, `pr`, `T_from_h`, `T_from_pr`), the isentropic
substate via a `pr` ratio, and the efficiency-on-enthalpy application. Two sources,
both validated against the **same NASA-7 air model** (see § NASA-air verification):

- **Çengel & Boles**, *Thermodynamics: An Engineering Approach*, Ch. 9, problems
  **9-88E / 9-89 / 9-90** — full Brayton cycle, air, variable specific heats via
  Table A-17 (SI) / A-17E (English).
- **Mattingly**, *Elements of Propulsion* (the project's chosen anchor book),
  §2.6.6 "Gas Tables" + **Examples 2.7 / 2.8** — per-process variable-cp examples
  (compression, nozzle) using his Appendix D gas table.

Every number below was re-derived by hand and confirmed self-consistent. **Finding
(from reading the Mattingly PDF directly):** Mattingly has *no worked variable-`cp`
turbojet cycle* — his Ch. 5/7/8 cycle analysis is calorically perfect throughout
(Example 7.1 is constant dual-cp, the rung-2 anchor); "variable specific heat"
appears only in the Ch. 2 fundamentals. So the variable-`cp` anchor is necessarily
*per-process machinery*, from both books, not a single turbojet number.

## ⚠ Topology caveat — these are POWER cycles, not turbojets
A simple Brayton **power** cycle expands the turbine across the **full** pressure
ratio (state 3→4 drops ×π back to `P1`), so `w_T > w_C` and the surplus is net
shaft work. A **turbojet**'s turbine does only `w_T = w_C/η_m` (shaft balance) and
the *nozzle* takes the leftover pressure as thrust. So these examples **cannot be
matched end-to-end through `build_turbojet`** — they anchor the *property + process*
machinery (stations-3/5-style compression/expansion and η application), which is
exactly what rung 3 newly introduces. The turbojet topology itself is already
anchored by the rung-1/2 reduce-to-ideal gate (CPG closed-form branch); the TPG
integral path through the shaft balance + nozzle is covered by the dual-section
discriminating check (see `docs/rung3-variable-cp.md` gate 3).

What these anchor, what they don't:
- **Anchored:** the air `cp(T)` functions end-to-end up to 1240 K; the isentropic
  `pr`-ratio substate; η-on-enthalpy; cycle energetics from `h`-differences.
- **NOT anchored:** the hot-**products** polynomial (Çengel is single-gas air —
  products coefficients float, see § Products are unanchored); the turbojet shaft
  balance, nozzle `V9`, and thrust (topology differs — covered elsewhere).

---

## 9-89 (SI, with component efficiencies) — the PRIMARY anchor
Simple Brayton, air, **variable specific heats (Table A-17)**, `η_c = 0.83`,
`η_t = 0.87`, pressure ratio `r_p = 10`.

| State / step | Quantity | Value |
|---|---|---|
| 1 (compressor in) | `T1` | 295 K |
| | `h1` | 295.17 kJ/kg |
| | `Pr1` | 1.3068 |
| 1→2s isentropic | `Pr2 = r_p·Pr1` | 13.07 |
| | `h2s` | 570.26 kJ/kg |
| | **`T2s`** | **564.9 K** |
| 2 (actual, η_c) | `h2 = h1 + (h2s−h1)/η_c` | 626.60 kJ/kg |
| 3 (turbine in) | `T3` | 1240 K |
| | `h3` | 1324.93 kJ/kg |
| | `Pr3` | 272.3 |
| 3→4s isentropic | `Pr4 = Pr3/r_p` | 27.23 |
| | `h4s` | 702.07 kJ/kg |
| | **`T4s`** | **689.6 K** |
| 4 (actual, η_t) | `h4 = h3 − η_t·(h3−h4s)` | 783.04 kJ/kg |
| | **`T4`** | **764.4 K** |
| (b) | `q_in = h3 − h2` | 698.3 kJ/kg |
| | `q_out = h4 − h1` | 487.9 kJ/kg |
| | `w_net = q_in − q_out` | 210.4 kJ/kg |
| (c) | **`η_th = w_net/q_in`** | **0.3013 (30.1%)** |

**Datum-independent checkables** (immune to the table's enthalpy/`Pr` datum, so my
NASA-air model can be tested against them directly):
- `T2s ≈ 564.9 K` from `pr(T2s) = 10·pr(295 K)` (isentropic compression).
- `T4s ≈ 689.6 K` from `pr(1240 K) = 10·pr(T4s)` (isentropic expansion).
- `h2s − h1 = 275.09`, `h3 − h2 = 698.3`, `h3 − h4s = 622.86` kJ/kg (Δh, datum cancels).
- `η_th = 0.301` (ratio of Δh's).

## 9-88E (English, *ideal* cycle) — independent cross-check
Same machinery, **no** component efficiencies, English units / a different table
datum (Table A-17E). Matching this proves the `pr`-ratio and Δh logic is genuinely
datum- and unit-system-independent. `r_p = 10`.

| State / step | Quantity | Value |
|---|---|---|
| 1 | `T1`, `h1`, `Pr1` | 520 R, 124.27 Btu/lbm, 1.2147 |
| 1→2 isentropic | `Pr2 = 12.147` → `T2`, `h2` | 996.5 R, 240.11 Btu/lbm |
| 3 | `T3`, `h3`, `Pr3` | 2000 R, 504.71 Btu/lbm, 174.0 |
| 3→4 isentropic | `Pr4 = 17.4` → `h4` | 265.83 Btu/lbm |
| (a) | `w_C,in = h2−h1` | 115.84 Btu/lbm |
| | `w_T,out = h3−h4` | 238.88 Btu/lbm |
| | back-work ratio `r_bw` | 48.5% |
| (c) | `q_in = h3−h2` | 264.60 Btu/lbm |
| | `w_net` | 123.04 Btu/lbm |
| | **`η_th`** | **46.5%** |

The model is SI-only (CLAUDE.md), so 9-88E is checked by converting inputs
(520 R = 288.89 K, 2000 R = 1111.11 K) and comparing the *dimensionless* results
(`η_th`, `r_bw`) and the isentropic temperatures — the unit-agnostic part.

## 9-90 (EES parametric) — TREND only, NOT a numeric anchor
9-90 re-solves 9-89 in EES across `r_p = 2…20` (η_c=0.83, η_t=0.87, T1=295 K,
P1=100 kPa, T3=1240 K, ṁ=20 kg/s). Use it **only** for the qualitative shape: `η`
rises then falls with `r_p` (hump near `r_p ≈ 6–8`), and the back-work ratio climbs
toward 1 as `r_p` grows.

⚠ **Surprise flagged (do not bake into a test):** the printed EES table is
*numerically inconsistent* with the 9-89 hand solution — at `r_p = 10` it reads
`η = 0.170`, `Ẇ_net = 1822 kW` (→ 91 kJ/kg) vs. 9-89's `η = 0.301`,
`w_net = 210.4 kJ/kg`. The EES "Air" property model / inputs evidently differ from
the Table-A-17 hand solve. Per the "stop and explain surprises" contract: 9-90 is a
trend illustration, the anchors are **9-89 (primary) and 9-88E (cross-check)**.

---

## Mattingly Examples 2.7 / 2.8 (his own gas-table method) — SECONDARY anchors
Mattingly *Elements of Propulsion* §2.6.6 defines the **identical** machinery the
rung-3 spec adopts: `h = ∫cp dT` (Eq. 2.53), `φ = ∫cp/T dT` (2.54),
**`Pr = exp((φ−φref)/R)`** (2.55, "reduced pressure"), and isentropic
`P2/P1 = Pr2/Pr1` (2.58). His worked variable-cp examples (Appendix D gas table,
`f` = fuel-air ratio; `f = 0` ⇒ air) anchor two specific rung-3 stations:

| Example | Process | Inputs | Mattingly result |
|---|---|---|---|
| **2.7** | isentropic **compression** (station 3) | air, 293.15 K, `Pr1 = 1.2768`, `π = 15` → `Pr2 = 19.152` | **`T2 = 627.57 K`** (7.9 K below the calorically-perfect 635.5 K) |
| **2.8** | isentropic **nozzle** (station 9) | air, `T1 = 3000 °R`, `V1 = 0`, `P1 = 10 atm`, `V2 = 3000 ft/s` ⇒ `Δh = 179.74 Btu/lbm` | **`T2 = 2377.7 °R`**, `Pr2 = 352.6` → **`P2 = 3.757 atm`** (`P2/P1 = 0.3757`) |

Ex 2.8 is especially valuable: it exercises the rung-3 station-9 pair *together* —
`V2` from the **enthalpy** split (`h1 + V1²/2gc = h2 + V2²/2gc`) and the exit
pressure from the **`Pr` ratio** — the exact form of the spec's nozzle equation.
These honor the original "also match Mattingly" intent at the machinery level even
though no Mattingly turbojet uses variable cp.

---

## NASA-air verification (the anchors are de-risked BEFORE coding)
A standalone NASA-7 air model (mole-weighted N₂/O₂/Ar, `R_air = 287.10`) was run
against every anchor *before* writing production code. It agrees with both books'
gas tables to ~0.1%, which both validates the NASA coefficients and certifies the
anchor numbers:

| Anchor | Quantity | NASA-7 air | Book | gap |
|---|---|---|---|---|
| Çengel 9-89 | `T2s` (295 K, π=10) | 564.48 K | 564.9 K | −0.074% |
| Çengel 9-89 | `T4s` (1240 K, /10) | 689.39 K | 689.6 K | −0.031% |
| Çengel 9-89 | `Δh` 295→T2s | 275.10 | 275.09 kJ/kg | +0.004% |
| Çengel 9-89 | cycle `η_th` | 0.3014 | 0.3013 | +0.03% |
| Mattingly 2.7 | `T2` (293.15 K, π=15) | 626.92 K | 627.57 K | −0.103% |
| Mattingly 2.8 | `T2` (Δh nozzle) | 2377.5 °R | 2377.7 °R | −0.006% |
| Mattingly 2.8 | `P2/P1` | 0.3758 | 0.3757 | +0.030% |

(De-risk prototypes run during sourcing; the permanent versions of these checks
become `tests/test_variable_cp.py` when rung 3 is coded.)

## How this maps onto the rung-3 code path
Çengel states 1→2 and 3→4 are *exactly* the rung-3 station-3 (compressor) and
station-5 (turbine) substate equations, run on a single air section:

```
T2s = T_from_pr_c( pr_c(T1) · r_p )            # 9-89:  → 564.9 K
h2  = h_c(T1) + (h_c(T2s) − h_c(T1))/η_c        # 9-89:  → 626.60 kJ/kg  (η_c=0.83)
T4s = T_from_pr_t( pr_t(T3) / r_p )            # 9-89:  → 689.6 K
h4  = h_c(T3) − η_t·(h_c(T3) − h_c(T4s))        # 9-89:  → 783.04 → T4=764.4 K
```

(Here both sections are air, so `_c == _t`.) The test reproduces these four states
with a single-air `Gas` built on the NASA-air `cp(T)` and asserts each published
number within the observed table-vs-NASA gap (set empirically — see § Tolerance).

## Products are unanchored (state it plainly)
9-89 validates the **air** `cp(T)` up to 1240 K. The rung-3 hot-**products**
polynomial is a *different* fit and **no Çengel number pins it**. For a teaching
tool this is acceptable; it is recorded here so the scope is honest: with the
Çengel anchor, the air machinery is anchored and the products coefficients float
(chosen to pass through the rung-2 `cp_t ≈ 1239` neighborhood, not externally
verified to the digit).

## Tolerance (set empirically — now measured)
Table A-17 / Appendix D and the NASA-7 air fit are *different* correlations of real
air, so they agree only to table-rounding + fit-difference. The verification above
measured that gap: **≤ 0.11% on temperatures, ≤ 0.03% on Δh / `η_th` / pressure
ratio.** Set the test tolerance from that observed gap — **~0.15% (≈ ±1 K) on the
isentropic temperatures, ~0.1% on `η_th` and the Δh's** — not from a guessed bound.
The tighter (1e-9 / 5e-4) gates stay reserved for reduce-to-ideal, where the CPG
closed-form branch must match the *prior rungs* exactly.
