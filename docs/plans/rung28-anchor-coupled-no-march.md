# Rung 28 anchor — the rung-26-coupled NO march: is "can ONLY slow NO further" true? (numbers before code)

> **Verified anchor data** for rung 28 (rung-27's NO clock read on rung-26's *relaxing* pool instead of
> the frozen station-4 pool). The feasibility **probes** are
> `M:\claud_projects\temp\rung28\probe_coupled.py` (the two-channel decomposition),
> `probe_limit.py` (the pool-rate limit sweep) and `probe_trend.py` (the `Tt4` trend) — outside git
> (project temp policy), reusing the shipped rung-6/7/14/26/27 machinery
> (`_equilibrium_composition`, `_tau_no_destroy`, `_tau_chem_recomb`, `_mix_h_abs_B`,
> `_mix_entropy_molar`, `nozzle_flow`, `build_turbojet`) read-only. Design point: the standard
> real-loss cycle (`π_d=0.97, η_c=0.88, η_b=0.99, π_b=0.96, η_t=0.90, η_m=0.99, π_n=0.98`, `M0=0.85`,
> `π_c=10`), swept over `Tt4`. **No rung code written yet — this file is the go/no-go.**

## The claim under test — rung 27's OWN deferred note

`docs/rung27-spec.md` § Deferred says of this refinement:

> "NO riding the *relaxing* pool, so `[O],[H]` deplete under it. This **can only** *slow* NO further
> (radical-poorer ⇒ larger `τ_NO`), moving the answer **deeper into frozen**."

That word **"only" is the overclaim under test.** Coupling to rung 26 couples to *all* of rung 26 —
including the fact that recombination is **exothermic**. Two channels, opposing:

| channel | mechanism | effect on `τ_NO` | effect on the verdict |
|---|---|---|---|
| **(1) radical depletion** | `[O],[H]` recombine on the relaxing path | `τ_NO ∝ 1/([O],[H])` **rises** | **DEEPER** frozen |
| **(2) heat release** | recombination heat ⇒ `T(s)` **above** frozen-isentropic | Arrhenius `k` (`θ≈20820/24560 K`) **rises** ⇒ `τ_NO` falls | **LESS** frozen |

Rung 27's note sees only channel 1. **A composition-only coupling (thread `[O],[H]`, keep the frozen
`T`) would structurally exclude channel 2 and thereby manufacture rung 27's tidy prediction** — so the
probe couples **both** `T(s)` and `comp(s)`, and decomposes.

## The decomposition — four evaluations of the SAME clock

`Da_NO(s) = τ_res/τ_NO(T,p;comp)` is evaluated on four combinations of (T-path, comp-path):

| label | T path | comp path | isolates |
|---|---|---|---|
| `rung27` | frozen isentropic | frozen station-4 | the rung-27 baseline |
| `ch1` | frozen isentropic | rung-26 freeze-out | **depletion alone** |
| `ch2` | rung-26 freeze-out | frozen station-4 | **heat release alone** |
| `coupled` | rung-26 freeze-out | rung-26 freeze-out | **the rung-28 march** |

## 1. The headline is UNTOUCHED — entry `Da_NO` is bit-for-bit rung 27's

The nozzle-entry state is **path-independent** (`comp_entry`, `Tt9`, `pt9` are inputs, not marched), so:

| `Tt4` [K] | `Da_NO` entry (rung 27) | `Da_NO` entry (coupled) | identical |
|---|---|---|---|
| 1500 | 8.284708e-09 | 8.284708e-09 | **True** |
| 1800 | 5.994204e-06 | 5.994204e-06 | **True** |
| 2200 | 2.010558e-03 | 2.010558e-03 | **True** |

**`frozen_from_entry` is therefore rung 27's result, verbatim** — and the whole coupled correction below
is under one order against rung 27's 3–9 order margin. **The coupling cannot reach the headline.**

## 2. Both channels are REAL — and channel 2 is NOT negligible (exit station, anchored rate)

| `Tt4` [K] | `ΔT_exit` [K] | `x_rad` frozen | `x_rad` coupled | `ch1` | `ch2` | **net** | `\|ln ch2 / ln ch1\|` |
|---|---|---|---|---|---|---|---|
| 1500 | 0.0 | 5.195e-07 | 4.571e-07 | 0.8799 | 1.0004 | **0.8803** | 0.003 |
| 1650 | 0.1 | 3.106e-06 | 2.376e-06 | 0.7650 | 1.0024 | **0.7669** | 0.009 |
| 1800 | 0.4 | 1.353e-05 | 8.579e-06 | 0.6340 | 1.0092 | **0.6398** | 0.020 |
| 2000 | 2.2 | 6.592e-05 | 3.175e-05 | 0.4813 | 1.0372 | **0.4992** | 0.050 |
| 2200 | 9.1 | 2.266e-04 | 8.715e-05 | 0.3821 | 1.1302 | **0.4318** | 0.127 |
| 2300 | 18.5 | 3.715e-04 | 1.349e-04 | 0.3583 | 1.2534 | **0.4488** | 0.220 |
| 2400 | 41.7 | 5.473e-04 | 2.091e-04 | 0.3791 | 1.5942 | **0.6041** | **0.481** |

**Rung 27's conclusion is CONFIRMED; its mechanism is CORRECTED.** `net < 1` at every in-band `Tt4`
(deeper into frozen — the conclusion holds). But `ch2 > 1` everywhere and **grows monotonically**, from
0.3% of the depletion effect (in log space) to **48% at the hot edge**. "Can **only** slow NO further"
is **false as a mechanism statement**: there is a real opposing channel, and at the hot edge it cancels
nearly half of the depletion.

**The net is NON-MONOTONE in `Tt4`** (deepest ~2200–2300 K, then reversing toward 2400) because ch2
accelerates faster than ch1 saturates. **Location DISCLAIMED** — how much the pool relaxes rides on
`L/τ_res`, exactly as rung 26/27 disclaim `s_freeze`. What is claimed is the **monotone rise of
`|ln ch2 / ln ch1|`**, which is a ratio of the two channels on the *same* path and is far more robust.

## 3. WHY depletion wins — the structural argument (the load-bearing result)

Both channels are switched on by the *same* thing (how much the rung-26 pool relaxes), so the honest
robustness test is the **limit**: drive the pool `rate_scale` up toward instant re-equilibration —
**maximum heat release AND maximum depletion** — and see whether the net can flip.

`Tt4 = 2200 K`:

| pool `rate_scale` | `ΔT_exit` [K] | `x_rad` exit | `ch1` | `ch2` | **net** | winner |
|---|---|---|---|---|---|---|
| 1.0e0 | 9.1 | 8.715e-05 | 0.3821 | 1.1302 | 0.4318 | depletion |
| 1.0e1 | 13.6 | 1.076e-05 | 0.0453 | 1.1997 | 0.0542 | depletion |
| 1.0e2 | 14.3 | 1.305e-06 | 0.0053 | 1.2106 | 0.0064 | depletion |
| 1.0e3 | 14.5 | 1.926e-07 | 0.0008 | 1.2130 | 0.0009 | depletion |
| 1.0e4 | 14.5 | 5.222e-08 | 0.0002 | 1.2136 | 0.0003 | depletion |
| 1.0e6 | 14.5 | 3.087e-08 | 0.0001 | 1.2138 | **0.0001** | depletion |

Same structure at 1800 K (ch2 ceiling ≈ 1.024) and 2400 K (ch2 ceiling ≈ 2.140).

**The asymmetry is structural, not numerical:**

- **Channel 1 is UNBOUNDED.** `τ_NO ∝ 1/([O],[H])`, and as the pool equilibrates `[O],[H]` → their
  *local equilibrium* values, which **crater** as the gas cools. `ch1 → 0` with no floor.
- **Channel 2 SATURATES.** The heat release is bounded by the **finite chemical enthalpy** in the
  frozen-in dissociation — its ceiling is the rung-14 equilibrium-vs-frozen exit-`T` gap. `ch2` reaches
  a hard ceiling (×1.02 / ×1.21 / ×2.14 at 1800/2200/2400 K) and stops.

So **for any chemistry faster than anchored, depletion wins decisively** — 6 orders of pool rate move
the net from 0.43 to 1e-4 while ch2 moves 1.13→1.21. Rung 27's *conclusion* is robust for a reason that
is asymptotic, not incidental. **This is the rung's load-bearing claim.**

## 4. No crossing is reachable — and the band is closed by the cycle, not by choice

`net > 1` (heat release actually winning) does not occur at any in-band `Tt4`. The trend toward 2400 K
is upward, but **`Tt4 ≥ 2500 K` does not run at all** at this `π_c` — the rung-6 burner balance assert
fires (`h_air + n_f·hf != Σ n_i h_i + loss`), i.e. the cycle itself has no solution there. So no
crossing is claimed and **none can be probed**, exactly as rung 27 declined to extrapolate its
`Da_NO`-vs-`Da_recomb` crossing.

## 5. Resolution check — the finding is converged

The hot-end finding sits where the trapezoid `ΔT` is largest (69 K at 2400 K), so `nstep` was swept:

| `Tt4` | `nstep` | `ΔT_exit` | `ch1` | `ch2` | net |
|---|---|---|---|---|---|
| 2200 | 400 | 9.128 | 0.38215 | 1.13018 | 0.43182 |
| 2200 | 800 | 9.123 | 0.38264 | 1.13011 | 0.43234 |
| 2200 | 1600 | 9.121 | 0.38288 | 1.13007 | 0.43260 |
| 2400 | 400 | 41.662 | 0.37914 | 1.59415 | 0.60413 |
| 2400 | 800 | 41.639 | 0.37961 | 1.59376 | 0.60474 |
| 2400 | 1600 | 41.628 | 0.37985 | 1.59356 | 0.60504 |

**4× the resolution moves the net by 0.2% (2200 K) and 0.15% (2400 K).** Converged; `nstep=400`
(rung 26's default) is adequate.

## What the Da-ratios do and do NOT mean

**`net = ×0.43` is the CLOCK's depth, not NO's motion.** NO does **not** move by 43% — it does not move
at all: `Da_NO` stays 3–9 orders below 1, so `relaxed_fraction ≈ 0` on every path above. The ratios
measure *how much further below the freeze threshold the coupling pushes the clock*. The **fate** of NO
is "frozen"; the **mechanism** is what these ratios describe. Any presentation must keep those apart.

## Go / no-go — GO

- **The headline survives, provably** — entry `Da_NO` is bit-for-bit rung 27's (path-independent).
- **There is real content** — rung 27's "can only" is wrong as a mechanism; the opposing channel reaches
  48% of the depletion at the hot edge, and it makes the net trend non-monotone.
- **The verdict is structural** — depletion unbounded vs heat release saturating, certified over 6
  orders of pool rate.
- **Zero new constants** — every rate is rung 26's (GRI-Mech 3.0) or rung 27's (Zeldovich reverse).
- **Character: a CONFIRMATION with a MECHANISTIC CORRECTION.** Rung 27's *conclusion* stands; its stated
  *reason* was one-sided. Not a refutation — and the spec must say so in those words.
