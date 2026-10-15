# Rung 91 — TURBINE COOLING AIR: HP-compressor air that rejoins at the turbines

**Code:** `rust/src/turbine_cooling.rs` (the shares `TurbineCooling`, the two mixers `mix` /
`mixed_tt`, the cooled HP turbine pin `try_solve_cooled_hpt`, the steady cascade with its `Tt3`
fixed point `try_cascade_cooled`, the matcher `TurbineCoolingMatcher` / `R91`, and rung 62's four
transient cells as rung 90 carried them, copied with the cooling: `r91_try_close`,
`r91_try_close_fuel`, `r91_powers`, `r91_try_instant_tail`); the shares live on
`TwoSpoolEngine::cooling` and are copied onto rung 38's core; rung 62's cells dispatch to rung 91's
first when cooling is active · **Gates:** `rust/tests/rung91.rs` · **Panel:**
`print_turbine_cooling_table` (born in Rust, held to `rust/oracle/rust_owned/`, a change detector
only) · **Anchor (the derivation, the linear model and every bar, committed `e46ab3d` before any
cooling code existed):** `docs/plans/rung91-anchor-turbine-cooling.md` · **Sandbox:** the Controls
view, "Turbine cooling, vane air / blade air" (`docs/plans/sandbox-plan.md` § 20) · **Scratch
scripts and outputs:** `docs/plans/scratch-archive/rung91/`.

**Shipped as KNOBS** (the sandbox direction): predictions were registered first and are scored
here; a miss is a finding.

Rung 90 closed by naming this seam: *"Turbine cooling air — HP-compressor air that skips the HP
turbine's nozzle and rejoins downstream: a bleed that changes the turbines' flow and work, not only
the compressors'."* The user chose two streams (vane air before the HP rotor, blade air after it),
the whole seam in one rung as rung 90 (steady, slam, valve, sandbox knob), and shaft-power offtake
left to its own rung (2026-10-15).

> **THE HEADLINE — air that comes back is CHEAP in thrust and DEAR in surge margin.** The coolant
> rejoins ahead of the LP turbine's choked throat (A45), so that throat must pass more gas: its
> back-pressure rises, the HP turbine expands LESS (`d ln π_HPT/dε` ≈ +0.75 vane, +0.79 blade —
> rung 31's choked pin had held it at one point), and the HP spool loses MORE work than if the air
> had been thrown overboard. So **rung 90's HP arrow reverses**: the HP is pushed toward surge at
> high power, crossing at **1116 K** for vane air (predicted 1120) and **795 K** for blade air
> (predicted 798). And **the LP is pushed toward surge at every power** — the customer bleed's
> low-power relief is gone. Blade air is worse than vane air on both spools everywhere: it
> back-pressures the HP turbine as much and does no work there. A linear model written before any
> code predicts all of it to ≤ 0.0062.
>
> **THE VALVE PAYS FOR IT.** On rung 64's slam the LP protection valve bleeds MORE with either
> stream (all 19 riding pairs), its ceiling falls (all 12), and the bill runs blade > vane >
> customer at every set point where all three ride (9/9) — 2–12× (vane) and 3–29× (blade) the
> customer share's bill. Meanwhile 5 % of vane air costs about a third of the thrust 5 % of customer
> air does (−2.9 % vs −8.3 % at 1500 K), and at 1500 K its fuel per unit thrust barely moves (−0.04 %).

## 1. What it adds

Per unit HPC delivery: the customer `c` (rung 90), vane `ε₁`, blade `ε₂` — shares that ADD; the
burner gets `1 − c − ε`. Mixer 1 (after the A4 throat, before the rotor) adds `ε₁` at `Tt3`; the
rotor works on `G + ε₁` from `Tt41` to `Tt44`; mixer 2 (after the rotor, before A45) adds `ε₂`; the
LPT and nozzle pass `G + ε`. The mixers are adiabatic at constant total pressure, on the burner's
own enthalpy zero, diluting the fuel ratio; a mixer takes two streams, so it is NOT an
`apply(state, gas)` component (the Turbine and Nozzle's declared divergence) and asserts its own
mass, fuel and energy balances.

**The HP turbine pin is the structural novelty** (`π_HPT √(Tt4/Tt45) = A4·MFP4/(A45·MFP45) ·
(G+ε)/G`): the first rung since 31 where the choked HP turbine moves on the perfect gas. The LP
turbine's pin keeps its form, and on the perfect gas `τ_LPT`, `π_LPT` stay where they were
(gated, to 1e−11).

**The steady cascade loses its triangle**: both mixers need `Tt3`, which the HP balance makes from
the HPT — closed by a fixed point on `Tt3` inside the turbine-efficiency loop (contraction ≈ 0.02:
the mixers barely feel `Tt3`). The transient closure has no loop: `Tt3` comes from the maps first.

Rung 90's efficiency loops are CALLED with the extraction `c + ε` (their bodies carry it only in
the face referrals, where cooling air is as absent as customer air); everything else is a copy.
The HPT map's corrected speed is referred to the ROTOR inlet `Tt41`. Thrust: the cooling air leaves
through the nozzle (scored per nozzle air at the diluted `f45`, by a copy of rung 1's scoring with
the fuel ratio passed explicitly — the shipped one reads station 4's).

**Where it lives.** On the design engine, beside the customer bleed; the design run ignores it
(hardware sized with cooling OFF — the knob is cooling MODULATED on fixed hardware). Every sibling
keeps it (gated). Every body that cannot model it refuses it — rungs 38, 39, 42 steady; 40, 43
(hence 57) transient; **and rung 90's own bodies** (gated). **The perfect gas only**:
`with_turbine_cooling` refuses the others in words (a mixer would have to re-react the mixture).
Bounds: each share in `[0, 0.25)`; the customer and the cooling together below half, checked at
the ROOT (rung 90's lesson 2).

## 2. Reduces

Inactive (`None`, both shares 0) dispatches to the earlier bodies verbatim: steady → rung 90's
`try_match_customer` (→ rung 42 → 39), transient → rung 90's or rung 62's cells (→ rung 57). Gated
bit for bit at three throttles × two valves × customer off/share (steady) and two lever arms ×
customer off/share (transient, both closures and the fuel path); the closure carries no `cooling`
field then. The design run is unchanged (gated). **Plant gate**: the transient equilibrium equals
the steady match to 1e−9 in speeds, flow coefficients, pressure ratios, the HP turbine's `π` and
`τ`, `Tt45`, burner and face flow, customer share and thrust — vane alone, blade alone, both; with
the valve; with a customer share; with a customer amount and the valve. The cascade's mixers and
both shaft balances close to 1e−9 on the rebuilt stations. No oracle, no CLI golden and no earlier
rung's gate moved.

## 3. Scoring (anchor § 3 — bars frozen at `e46ab3d`)

Rig A: rungs 42/90's (CPG, π 3 × 6, 250 K / 50 kPa / M 0.85), `Tt4` 1500 → 650 K; readings
Richardson-extrapolated from shares 0.01 and 0.02. Rig B: rung 64's (shaped / tilted × `r` ∈
{0.25, 0.5, 1}, `ds` 0.005, 1000 → 1400 K, `φ` valve `b_max` 0.10, `sm` 0.30–0.42); every
comparison between equal-length runs (291 / 341 / 441 points per cell, all four demands).

| # | verdict | measured |
|---|---|---|
| A1 the HP turbine moves | **HIT** 24/24 | `d ln π_HPT/dε` within 0.0006 of the model (bar 0.03): +0.751 → +0.876 vane, +0.791 → +0.938 blade |
| A2 HP magnitude | **HIT** 24/24 | within 0.0043 (bar 0.03); measured sits a near-constant −0.003 / −0.004 below |
| A3 LP magnitude | **HIT** 24/24 | within 0.0062 (bar 0.05) |
| A4 HP crossing | **HIT** both | one sign change each, negative above: vane **1116 K** (bar 1120 ± 40), blade **795 K** (798 ± 25) |
| A5 LP, no relief | **HIT** 24/24 | negative everywhere: vane −1.023 → −0.178, blade −1.603 → −0.771 |
| A6 ordering | **HIT** 24/24 | blade below vane on both spools at every `Tt4` |
| A7 shaped signs | **HIT** 8/8 | 1500 K: vane −0.758 / −0.161, blade −1.187 / −0.419; 650 K: vane −0.172 / +0.310, blade −0.722 / +0.120 |
| B1 | **HIT** 12/12 | valve shut, `min φ_LP` falls: vane by 0.022–0.025, blade by 0.041–0.045 |
| B2 | **HIT** 6/6 | blade lowers `min φ_HP` by 0.014–0.018 |
| B3 | **HIT** 19/19 riding pairs | the valve's bleed rises (shaped `r` 0.5, `sm` 0.38: 9.29e−3 → 2.73e−2 vane, 4.35e−2 blade) |
| B4 | **HIT** 12/12 | ceiling falls: vane by 0.024–0.027, blade by 0.044–0.049 |
| B5 | **HIT** 9/9 | blade > vane > customer at every set point where all three ride |

The named likeliest miss (the vane HP through the amplified HP-work term) did not happen.

## 4. After scoring (reported, not scored)

* **The thrust bill runs the other way.** At 5 % and the same `Tt4` (flat): thrust per inlet air
  −2.9 / −3.6 / −4.5 % (vane, 1500 / 1100 / 800 K), −3.0 / −4.1 / −6.0 % (blade), against the
  customer share's −8.3 / −10.4 / −15.3 %; fuel per unit thrust −0.04 / +0.79 / +2.0 % (vane) against
  +5.5 / +8.2 / +14.7 %.
* **Vane air buys a cooler rotor**: 5 % lowers the rotor-inlet temperature by 46 K at 1500 K
  (1454 K). The model has no metal temperature, so at a fixed burner temperature cooling can only
  look like a cost — what it buys is THIS number (concession).
* **Vane cooling's HP is the open one in the slam**: § 1's crossing sits inside the ramp; measured,
  vane 5 % lowers `min φ_HP` slightly in all 6 cells (by 0.002–0.004 — the slam's minimum sits on
  the high-power side), far less than blade's.
* **A valve that held can no longer hold.** Where the valve rode without cooling (shaped `r` 0.25,
  `sm` 0.38, peak 0.071) it saturates with vane air (violation `min φ_LP` 0.7531 < 0.759).

## 5. Why — the model

Anchor § 1. The HPT pin: `p = [μ + ½μ₁(κθ−1) + ½μ₂(κθ'−1)]/(1 − σ/2)` — the A45 throat passes `ε`
more gas, partly relieved by the coolant lowering `Tt45`. HP work per HPC air `h = −ε + μ₁κθ − ρp`
(the burner gas lost, the vane enthalpy back, and the expansion the back-pressure takes, `ρ` ≈ 1.07
amplifying it); LP work `ℓ = −ε + μ₁κθ + σp + μ₂κθ'`. Fed through rung 90's compressor algebra
(`−c → h, ℓ`; referral `c → ε`): the HP's large `h` collapses `π_HPC`, which starves the LP face
(`a_H` term) more than the missing air feeds it, at every power. Strike the put-back terms and it
is rung 90's model exactly.

## What it does to its neighbours

* **Rung 90 — its HP verdict BOUNDED to air that leaves.** "The HP is helped at every power" holds
  for air thrown overboard; air returned ahead of the next choked throat reverses it at high power.
  The difference is the A45 back-pressure, which rung 90's extraction never touched.
* **Rung 31 — its turbine pin BOUNDED.** The choked HP turbine sits at ONE point only while both
  throats pass the same gas; cooling air between them moves it (on the perfect gas, with both
  nozzles choked).
* **Rung 64 — its ceiling moved by a load again**, now by air that returns.
* **Rung 41 — CONFIRMED.** The LP is still the exposed spool.

## Concessions

* **No metal temperature**: at a fixed burner temperature cooling is pure loss here; the rotor-inlet
  temperature it buys is reported, not used. A real engine trades it for a hotter burner.
* **Hardware sized with cooling off**: a real A45 is sized to pass its coolant at design. The knob
  is MODULATED cooling on fixed hardware; the mechanism (more gas through A45 than it was sized for
  back-pressures the HPT) is the derivative at any base point.
* Ideal mixers (no total-pressure loss); vane air enters downstream of the A4 throat; no cooled-
  turbine efficiency penalty; coolant at `Tt3` (no cooler, no pumping).
* The perfect gas only (refused otherwise).
* Inherited: invented maps, rung 36's imposed `φ_surge`, both nozzles choked, the LP valve only, no
  fuel leg in rig B (the sandbox runs them beside it, replayed bit for bit).

## The next seam

**Shaft-power offtake** — the aircraft's generator load on the HP shaft: the other half of what an
aircraft takes (rung 90 took its air). Beside it: **a metal-temperature limit** that would let the
cooling buy a hotter burner — the trade this rung could only report.
