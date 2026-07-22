# Rung 41 anchor — the two-spool surge line

Verified data behind `docs/rung41-spec.md`. Design point throughout:
`build_two_spool_turbojet(gas, pi_lpc=3, pi_hpc=6, Tt4=1500, p0=50 kPa, nozzle_convergent=True)`,
`FLIGHT = (T0=250 K, p0=50 kPa, M0=0.85)`, losses
`pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96, eta_hpt=0.92, eta_lpt=0.90,
eta_m=0.99, pi_n=0.98`. Single-spool comparisons use
`build_turbojet(gas, pi_c=10, Tt4=1500, …, nozzle_convergent=True)`, `eta_c=0.90, eta_t=0.92`
— rung 36's own engine.

Disclosed map shapes (compressor islands only, `a_t = 0`), rung 39/40's set:

| key | LP map | HP map |
|---|---|---|
| `flow/press` | `a=.20 b=.05 sigma=.1 l=.7` | `a=.08 b=.15 sigma=.1 l=1.0` |
| `press/flow` | `a=.05 b=.20 sigma=.1 l=1.0` | `a=.20 b=.05 sigma=.1 l=.7` |
| `tilted` | `a=.14 b=.10 c=.06 sigma=.2 l=.85` | same |
| `steep` | `a=.25 b=.12 sigma=.3 l=1.2` | same |

`tilted` / `steep` / `flow` are also used **matched** (the same shape on both spools) for the
margin ordering. This matches the *shape*, **not** the design split (`π_LPC`=3 vs `π_HPC`=6) —
see § 4 for what that does and does not license.

---

## 1. THE SPLIT — the LP takes the excursion (thermally_perfect gas)

`flow/press`:

| Tt4 | phi_L | phi_H | pi_LPC | pi_HPC | n_L | n_H |
|---|---|---|---|---|---|---|
| 1500 | 1.0000 | 1.0000 | 3.0000 | 6.0000 | 1.0000 | 1.0000 |
| 1300 | 0.9061 | 0.9739 | 2.6184 | 5.1782 | 0.8995 | 0.9393 |
| 1100 | 0.8173 | 0.9511 | 2.2700 | 4.3876 | 0.8025 | 0.8742 |
| 900  | 0.7391 | 0.9350 | 1.9568 | 3.6326 | 0.7065 | 0.8032 |
| 800  | 0.7060 | 0.9315 | 1.8152 | 3.2704 | 0.6582 | 0.7649 |
| 750  | 0.6915 | 0.9315 | 1.7482 | 3.0942 | 0.6340 | 0.7449 |

`tilted`:

| Tt4 | phi_L | phi_H | pi_LPC | pi_HPC | n_L | n_H |
|---|---|---|---|---|---|---|
| 1500 | 1.0000 | 1.0000 | 3.0000 | 6.0000 | 1.0000 | 1.0000 |
| 1300 | 0.9099 | 0.9727 | 2.6171 | 5.1786 | 0.8953 | 0.9406 |
| 1100 | 0.8233 | 0.9488 | 2.2661 | 4.3894 | 0.7956 | 0.8766 |
| 900  | 0.7460 | 0.9325 | 1.9503 | 3.6370 | 0.6984 | 0.8063 |
| 800  | 0.7132 | 0.9293 | 1.8076 | 3.2765 | 0.6501 | 0.7681 |
| 750  | 0.6988 | 0.9296 | 1.7403 | 3.1012 | 0.6259 | 0.7481 |

Excursion over `Tt4` 1500 → 800 (gate 3 asserts the ratio > 3):

| shapes | `1 − phi_L` | `1 − phi_H` | ratio |
|---|---|---|---|
| `flow/press` | 0.2940 | 0.0685 | **4.29×** |
| `press/flow` | 0.2729 | 0.0713 | **3.83×** |
| `tilted`     | 0.2868 | 0.0707 | **4.06×** |
| `steep`      | 0.2686 | 0.0642 | **4.18×** |

Note `phi_H` **turns**: 0.9315 at both 800 and 750 in `flow/press`, and 0.9293 → 0.9296 in
`tilted`. That is the `(★)` stationary point, at `pi_HPC ≈ 3.1–3.3`.

---

## 2. THE SHIELDING, quantified (gate 4) — CPG, flat maps, split 3×6, design flight

```
s_H = k(1 − pi_HPC^(−1/k)) − 1                              k = gamma_c/(gamma_c−1) = 3.5
s_L = k(1 − pi_LPC^(−1/k)) + k(1 − pi_HPC^(−1/k))/tau_LPC − 1
```

| Tt4 | pi_HPC | `s_H` | `s_H` pred | `s_L` | `s_L` pred | `s_L` **drop-HP** | err(drop) |
|---|---|---|---|---|---|---|---|
| 1400 | 5.5910 | 0.3531 | 0.3596 | 0.8911 | 0.8787 | −0.1054 | **0.9966** |
| 1200 | 4.7931 | 0.2544 | 0.2633 | 0.7530 | 0.7461 | −0.2070 | **0.9600** |
| 1000 | 4.0280 | 0.1389 | 0.1494 | 0.5913 | 0.5892 | −0.3158 | **0.9071** |
| 850  | 3.4805 | 0.0382 | 0.0492 | 0.4503 | 0.4513 | −0.4026 | **0.8529** |
| 750  | 3.1301 | −0.0373 | −0.0263 | 0.3447 | 0.3474 | −0.4632 | **0.8079** |

`s_H` (which contains **no LP quantity**) and `s_L` (which contains the **product**) each land
within **0.013** of the measured value; **dropping `pi_HPC` from `s_L`** misses by **0.81–1.00**
— 60–100× worse, and with the **wrong sign** (negative where the truth is strongly positive).
Also holds at split 4.5×4, at `M0 = 1.6`, and at `gamma_c = 1.35` (gate 4).

`s_H` crosses zero between `Tt4` 850 and 750 — i.e. between `pi_HPC` 3.48 and 3.13, bracketing
`pi* = 3.2467`.

---

## 3. THE CLOSED FORM (★) — `1 + eta_c(tau_c−1) = gamma_c`, `pi* = gamma_c^(gamma_c/(gamma_c−1))`

CPG, flat maps. `pi*` (closed form) = **3.24674** at `gamma_c = 1.4`.

| case | `Tt4*` | measured `pi*` | `1+eta(tau−1)` | vs `gamma_c` |
|---|---|---|---|---|
| split 3×6      | 798.2  | 3.2973 | 1.40620 | +0.443 % |
| split 4.5×4    | 1170.6 | 3.3368 | 1.41098 | +0.785 % |
| split 2.25×8   | 666.2  | 3.2858 | 1.40479 | +0.342 % |
| `eta_HPC`=0.80 | 797.6  | 3.2962 | 1.40606 | +0.433 % |
| `eta_HPC`=0.95 | 798.6  | 3.2981 | 1.40630 | +0.450 % |
| `eta_HPT`=0.85 | 798.2  | 3.2973 | 1.40620 | +0.443 % |
| `eta_LPC`=0.80 | 784.3  | 3.2965 | 1.40609 | +0.435 % |
| `gamma_t`=1.25 | 798.2  | 3.2973 | 1.40620 | +0.443 % |
| `cp_t`=1300    | 800.0  | 3.3007 | 1.40661 | +0.472 % |
| `M0`=1.60      | 1056.8 | 3.3140 | 1.40823 | +0.588 % |
| `T0`=288       | 920.5  | 3.3052 | 1.40716 | +0.511 % |
| SHAPED maps    | 766.3  | 3.1494 | 1.39251 | −0.535 % |
| TPG gas        | 806.7  | 3.3269 | 1.40313 | +0.223 % |

**`Tt4*` moves 666 → 1171 K (1.76×); `pi*` moves 3.286 → 3.337 (1.5 %).** The turn is located
by a **pressure ratio**, not a throttle setting. `eta_HPT` and `gamma_t` are **bit-identical**
to the baseline — hot-section knobs cannot enter a cold-section closed form.

The two disclaimed departures are the two idealizations: **shaped maps −3.0 %** (in the `pi`
form; `psi ≢ 1`, `eta_c` varies) and the **variable-`cp` gas +2.5 %`**.

### 3b. `gamma_c` is the only parameter

| `gamma_c` | closed form `pi*` | measured `pi*` | `1+eta(tau−1)` |
|---|---|---|---|
| 1.30 | 3.11713 | 3.16441 | 1.30452 (+0.348 %) |
| 1.35 | 3.18212 | 3.23113 | 1.35536 (+0.397 %) |
| 1.40 | 3.24674 | 3.29733 | 1.40620 (+0.443 %) |
| 1.45 | 3.31103 | 3.36300 | 1.45702 (+0.484 %) |

The residual is essentially **constant** across the sweep — the closed form tracks `gamma_c`
exactly and the offset is a separate, additive effect. Which is:

### 3c. KILL TEST — the whole residual is the fuel fraction

`(★)` is exact with `f` frozen (`f` enters both `K` and the choked corrected flow). Raise
`hPR` so `f → 0`:

| `hPR` (J/kg) | `f` at the turn | measured `pi*` | `1+eta(tau−1)` | residual |
|---|---|---|---|---|
| 4.28e7  | 0.01157 | 3.29733 | 1.40620 | **+0.443 %** |
| 1.284e8 | 0.00364 | 3.26298 | 1.40200 | +0.143 % |
| 4.28e8  | 0.00107 | 3.25155 | 1.40059 | +0.042 % |
| 4.28e9  | 0.00011 | 3.24722 | 1.40006 | +0.004 % |
| 4.28e10 | 0.00001 | 3.24679 | 1.40001 | **+0.000 %** |

Monotone, and **linear in `f`** (`f` ÷3.18 per row over the first four ⇒ residual ÷3.10).
Nothing else hides in the offset.

### 3d. Reachability — the three regimes

| design `pi_HPC` | result | `Tt4*` | choked band |
|---|---|---|---|
| 3.0 (**< `pi*`**) | `RAIL` — `phi_H` **rises** monotonically 1.000 → 1.157: the HP walks *away* from surge | — | 610–1500 |
| 4.0 | `MIN` | 1171 | 610–1500 |
| 6.0 | `MIN` | 798 | 610–1500 |
| 12.0 | `RAIL` — the turn is **below the unchoke boundary** | — | 600–1500 |

At `M0 = 0.40` the turn also rails out (the choked band ends first). `flow_coefficient_turn`
reports `kind="RAIL"` rather than inventing an interior minimum.

---

## 4. THE MARGIN ORDERING (gate 6) — matched `tilted` shape, common floor, TPG gas

| `phi_surge` | Tt4 | `SM_L` | `SM_H` | `SM_L/SM_H` |
|---|---|---|---|---|
| **0.50** | 1500 | 0.3422 | 0.5636 | 0.6071 |
|      | 1300 | 0.2067 | 0.4558 | 0.4535 |
|      | 1100 | 0.1188 | 0.3671 | 0.3237 |
|      | 900  | 0.0647 | 0.2964 | 0.2181 |
|      | 800  | 0.0469 | 0.2677 | **0.1753** |
| **0.55** | 1500 | 0.3165 | 0.5186 | 0.6103 |
|      | 1100 | 0.1044 | 0.3355 | 0.3111 |
|      | 800  | 0.0377 | 0.2444 | **0.1543** |
| **0.60** | 1500 | 0.2883 | 0.4698 | 0.6137 |
|      | 1100 | 0.0884 | 0.3009 | 0.2937 |
|      | 800  | 0.0273 | 0.2185 | **0.1251** |

`SM_L < SM_H` everywhere, and `SM_L/SM_H` falls monotonically to **under a third** of its
design value.

**Read the design row before attributing the ordering.** At `Tt4` = 1500, `φ_L = φ_H = 1` —
there is *no* exposure difference — yet `SM_L` (0.3422) is already well below `SM_H` (0.5636).
That level offset is **`π_LPC` = 3 vs `π_HPC` = 6** (a smaller design pressure ratio gives a
smaller pressure-ratio margin at the same flow-coefficient gap), **not** the running line.
The running-line content is the **collapse of the ratio** (0.607 → 0.175), which is what gate 6
asserts. The flow-coefficient split of § 1 is unaffected — both `φ` are normalized to 1 at
design. The **absolute gap** `SM_H − SM_L` is deliberately *not* the measure: it peaks
near `Tt4 ≈ 1300` and then shrinks, because both margins tend to zero at deep throttle. At
`phi_surge = 0.70` the LP running line **crosses** the surge line (`SM_L < 0` by `Tt4 = 750`)
while the HP is still at `SM_H ≈ 0.20`.

**Not claimed:** the ordering at *unmatched* shapes/floors — with `press/flow` at design,
`SM_L = 0.4537 > SM_H = 0.3954`.

---

## 5. THE RUNG-36 CORRECTION (gate 7) — single spool `pi_c = 10`, `surge_flow`, `phi_surge = 0.55`

| Tt4 | `pi_c` | `phi_op` | `SM_N` | SM (φ-walk, `n` frozen) | SM (speed-line, `φ` frozen) |
|---|---|---|---|---|---|
| 1500 | 10.0000 | 1.00000 | 0.4838 | 0.4838 | 0.4838 |
| 1300 | 7.8447  | 0.93351 | 0.3281 | 0.3730 | 0.4235 |
| 1100 | 6.0516  | 0.87462 | 0.2215 | 0.2889 | 0.3645 |
| 900  | 4.5844  | 0.82925 | 0.1518 | 0.2319 | 0.3057 |
| 800  | 3.9616  | 0.81428 | 0.1280 | 0.2145 | 0.2762 |
| 700  | 3.4078  | 0.80682 | 0.1108 | 0.2061 | 0.2466 |
| 650  | 3.1552  | 0.80674 | 0.1045 | 0.2060 | 0.2318 |
| 600  | 2.9180  | **0.80975** | **0.0997** | **0.2094** | 0.2169 |

Read the last two rows: **`phi_op` turns UP** (0.80674 → 0.80975) as `pi_c` crosses
`pi* = 3.2467` — inside rung 36's own choked envelope — and the **φ-walk channel turns up with
it** (0.2060 → 0.2094), yet **`SM_N` keeps falling** (0.1045 → 0.0997) because the
**speed-line channel** does not turn (0.2318 → 0.2169).

Log-decay shares over 1500 → 650: full `ln(0.4838/0.1045) = 1.532`; φ-walk
`ln(0.4838/0.2060) = 0.854` (**56 %**); speed-line `ln(0.4838/0.2318) = 0.736` (**48 %**) —
they multiply to 1.590 against the true 1.532, the small excess being the interaction term.
Both channels are real and comparable; **neither is negligible**, which is the correction.
The same pattern holds on `surge_pressure` and `surge_tilted` (gate 7).

`SM_N` monotone-decreasing throughout ⇒ **rung 36's gated verdict is untouched** and no rung-36
test changes.

---

## What is gated vs. reported

**Gated:** the reduce (bit-for-bit, both directions), the per-spool `pi` reproduction, the
split sign (>3×) and `phi_L < phi_H`, the sensitivity formulas incl. the drop-HP failure,
`(★)` within 1 % across every listed invariance case plus the `gamma_c` sweep, the kill-test
monotone convergence, the margin ordering + monotone ratio at matched shapes, the
`phi`-turns-up/`SM`-keeps-falling divergence, and rung 36's `SM_N` monotonicity + both
channels non-negligible.

**Reported, not gated:** every margin magnitude; the turn's `Tt4` location; the −3.0 % /
+2.5 % shaped-map and TPG departures; the ordering at unmatched shapes; the reachability
table's boundaries.
