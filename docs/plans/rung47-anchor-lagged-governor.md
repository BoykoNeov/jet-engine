# Rung 47 anchor — the lagged / actuator topping governor (τ_gov)

**Method anchor.** Cohen–Rogers–Saravanamuttoo, *Gas Turbine Theory*, Ch. 9 — the acceleration
fuel schedule limited by a maximum turbine temperature, with the limiter's **finite response
lag**. A real TIT limiter senses turbine temperature and winds fuel down through a finite-bandwidth
loop (thermocouple + control lag, τ_gov ~ seconds — the dominant lag, far larger than valve slew);
when the accel drives temperature up faster than the loop can respond, TIT OVERSHOOTS the limit
(the classic topping overshoot). Here: the clip AMOUNT `g` (the fuel reduction below the schedule)
is a THIRD state relaxing toward `required = max(0, schedule − topping)` with τ_gov, on rung 43's
floating-`Tt4` two-shaft plant.

**Config for every table below** (`tests/test_rung47.py` reproduces them):
`FLIGHT = FlightCondition(T0=250, p0=50_000, M0=0.85)`, design `π_LPC=3, π_HPC=6, Tt4=1500`,
`REAL` losses (as rung 45/46), `nozzle_convergent=True`, **CPG** gas
(`γc=1.4, cp_c=1004, γt=1.3, cp_t=1239, hPR=42.8e6`; the finding is gas-independent — rung
35/43/45/46's carried concession). Shapes (rung 45/46's set): `flow/press`, `press/flow`,
`tilted`, `hp-only` (LP flat). Accel 1000→1400, `ρ=1`, `s_settle=2.0`, `ds=0.02`. The bare `Tt4`
peak is ~1670 K; the redline 1480 sits in the gap (above the 1400 endpoint, below the peak).

`overshoot = Tt4_peak_top − Tt4_max`; `relief_* = min_phi_*(topped) − min_phi_*(bare)`, `>0` safer.

---

## Table A — THE COST OF REALISM: τ_gov sweep (flow/press, r=0.5, redline 1480)

| τ_gov | overshoot (K) | relief_lp | relief_hp | held |
|---|---|---|---|---|
| inst (None) | +0.0 | +0.0000000 | +0.005267 | **True** |
| 0.05 | +55.6 | +0.0000000 | +0.003560 | False |
| 0.10 | +95.3 | +0.0000000 | +0.002873 | False |
| 0.20 | +137.5 | +0.0000000 | +0.002135 | False |
| 0.40 | +170.2 | +0.0000000 | +0.001410 | False |
| 0.80 | +190.9 | +0.0000000 | +0.000860 | False |

The instantaneous governor holds (overshoot 0, the rung-46 gate-3 result). ANY finite lag breaks
the hold; the overshoot grows monotonically with τ_gov (the topping overshoot). The HP rebate
erodes monotonically toward zero. `relief_lp` is machine-zero at every τ_gov.

## Table B — THE REFUTATION: relief_lp = 0 at every shape (τ_gov=0.2, r=0.5, redline 1480)

| shape | overshoot (K) | relief_lp | relief_hp | held |
|---|---|---|---|---|
| flow/press | +137.5 | +0.0000000 | +0.002135 | False |
| press/flow | +141.1 | +0.0000000 | +0.002985 | False |
| tilted     | +140.6 | +0.0000000 | +0.002601 | False |
| hp-only    | +134.6 | +0.0000000 | +0.001992 | False |

LP relief machine-zero at every shape, incl. mode-free hp-only (LP flat) — the refutation is the
timing/window mechanism, not a mode artifact. The LP surge min (s≈0.24, Tt4≈1374) is upstream of
engagement, where the topped-lagged march is bit-identical to bare; the lag (trailing-edge) cannot
reach it. Overshoot > 0 and HP rebate eroded (< the instantaneous ~0.0027–0.0037) at every shape.

## Table C — THE LEVER, LAGGED: fast ramp erosion (flow/press, r=0.15, redline 1440)

| τ_gov | overshoot (K) | relief_lp | relief_hp |
|---|---|---|---|
| inst (None) | +0.0 | **+0.02687** | +0.05088 |
| 0.05 | +218.9 | +0.01512 | +0.02568 |
| 0.10 | +292.3 | +0.01033 | +0.01534 |
| 0.20 | +343.8 | +0.00603 | +0.00843 |
| 0.40 | +388.5 | +0.00317 | +0.00443 |

At the fast ramp rung 46's INSTANTANEOUS governor reaches the LP (relief_lp=+0.027, the lever). A
lag that "reached earlier" would EXCEED that; instead it ERODES relief_lp monotonically toward zero
(as τ_gov→∞, g→0, topped→bare). Neutral at moderate r, strictly worse at fast r — in no regime does
the lag reach the LP better than the ideal. Overshoot is enormous (~220→390 K).

## Table D — THE SECONDARY: the topping command is monotone (why a valve lag is inert)

`topping_command_trace` over the rung-46 instantaneous engaged window (flow/press, r=0.5,
redline 1480): **monotone_nondecreasing = True**, n_engaged = 45, applied `mf` rises
0.01766 → 0.02334. The binding topping command rises monotonically (ν↑ ⇒ airflow↑ ⇒ more fuel to
hold the redline), so an instant-up / lag-down metering VALVE tracks it with no lag — a valve-
position lag is inert on the accel (probe: valve-slew gives zero overshoot at both r=0.5 and 0.15).
The overshoot lives in the sensing/limiter-LOOP lag, which lags the clip AMOUNT.

## Table E — REDUCE: τ_gov=None bit-for-bit rung 46; dormant/decel bit-for-bit rung 45

`τ_gov=None` (instantaneous) reproduces rung 46 float-for-float (`relief_lp`, `relief_hp`,
`Tt4_peak_top`, `held`). A redline above the bare peak ⇒ `required≡0` ⇒ `g≡0` ⇒ the lagged march
equals the bare rung-45 march at any τ_gov. A decel never fires the clip ⇒ bit-for-bit rung 45.

---

## Why the loop lag, not the coupled single-valve lag (the rejected Model X)

A single physical actuator lagging the WHOLE fuel path (schedule included) was measured and
rejected: it de-fangs the accel (bare `Tt4` peak collapses ~1698 → 1462 K from sluggishness
alone), so "bare" is no longer rung 45 — the **rung-45 dormant reduce breaks**. Its apparent LP
relief (relief_lp climbing off zero with τ) is the accel being slowed (the up-ramp lag lowering ν,
lowering topping, moving engagement earlier), not the clip reaching earlier — a confound of the
rung-42/43/45 currency kind. Isolating the governor (bare = rung 45 exactly, τ_gov→0 = rung 46
exactly) is what makes the differential admissible; the loop-lag model preserves both reduces and
gives the honest topping overshoot.
