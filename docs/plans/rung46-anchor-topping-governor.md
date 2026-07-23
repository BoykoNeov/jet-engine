# Rung 46 anchor — the TIT topping governor

**Method anchor.** Cohen–Rogers–Saravanamuttoo, *Gas Turbine Theory*, Ch. 9 — the acceleration
fuel schedule limited by a maximum turbine-inlet temperature (the TIT-topping governor). The fuel
control follows the accel schedule until turbine temperature reaches its ceiling, then meters fuel
to hold that ceiling. Here: a min-select `mdot_fuel = min(schedule, topping_setpoint(ν_L,ν_H,Tt4_max))`
on rung 43's floating-`Tt4` two-shaft plant.

**Config for every table below** (`tests/test_rung46.py` reproduces them):
`FLIGHT = FlightCondition(T0=250, p0=50_000, M0=0.85)`, design `π_LPC=3, π_HPC=6, Tt4=1500`,
`REAL` losses (as rung 45), `nozzle_convergent=True`, TPG (`Gas.thermally_perfect()`).
Shapes (rung 45's set): `flow/press`, `press/flow`, `tilted`, `hp-only` (LP flat).

---

## Table A — THE HEADLINE: the surge-relief SPLIT (accel 1000→1400, r=0.5, ρ=1, Tt4_max=1480)

`relief_* = min_phi_*(topped) − min_phi_*(bare)`; `>0` = safer than bare.

| shape | relief_lp | relief_hp | LP-min Tt4 | HP-min Tt4 | held |
|---|---|---|---|---|---|
| flow/press | +0.00000 | +0.00279 | 1373.7 | 1558.5 | True |
| press/flow | +0.00000 | +0.00357 | 1403.9 | 1583.3 | True |
| tilted     | +0.00000 | +0.00308 | 1404.9 | 1564.6 | True |
| hp-only    | +0.00000 | +0.00267 | 1372.9 | 1556.7 | True |

LP relief machine-zero at every shape (incl. mode-free hp-only); HP relief strictly positive.
The exact zero is because the topped trajectory is bit-identical to the bare one up to and through
the LP surge minimum — the clip has not fired there.

## Table B — THE MECHANISM: bare accel trajectory (flow/press, r=0.5, ρ=1), key samples

| s | Tt4 | phi_lp | phi_hp |
|---|---|---|---|
| 0.20 | 1318.3 | 0.7410 | 0.8804 |
| 0.24 | 1373.7 | 0.7399 ← LP min | 0.8738 |
| 0.26 | 1400.0 | 0.7400 | 0.8712 |
| 0.40 | 1558.5 | 0.7520 | 0.8627 ← HP min |
| 0.50 | 1644.9 ← Tt4 peak | 0.7705 | 0.8654 |

LP surge min at s=0.24/Tt4≈1374 (during the ramp, below the 1400 endpoint), then self-recovers.
HP surge min at s=0.40/Tt4≈1558; Tt4 peak at s=0.50. Any valid redline (≥1400) engages the clip
AFTER the LP min and around/after the HP min — so it misses the LP, catches the HP.

## Table C — redline sweep (flow/press, r=0.5, ρ=1): LP relief zero at every valid redline

| Tt4_max | relief_lp | relief_hp | Tt4_peak_top | held |
|---|---|---|---|---|
| 1410 | +0.00000 | +0.00846 | 1410.0 | True |
| 1440 | +0.00000 | +0.00606 | 1440.0 | True |
| 1480 | +0.00000 | +0.00279 | 1480.0 | True |
| 1522 | +0.00000 | +0.00077 | 1522.0 | True |
| 1580 | +0.00000 | +0.00000 | 1580.0 | True |
| 1620 | +0.00000 | +0.00000 | 1620.0 | True |

Even the tightest valid redline (1410, just above the endpoint) gives zero LP relief — the LP min
(Tt4≈1374) is below it. HP relief grows as the redline tightens (the clip bites harder/earlier).

## Table D — THE LEVER: fast-ramp switch-on (flow/press, Tt4_max=1440, ρ=1)

| r | relief_lp | relief_hp | LP-min Tt4 |
|---|---|---|---|
| 1.00 | +0.00000 | +0.00000 | 1219.1 |
| 0.50 | +0.00000 | +0.00606 | 1373.7 |
| 0.30 | +0.00264 | +0.02319 | 1564.2 |
| 0.15 | +0.01922 | +0.03869 | 1745.3 |

As r falls, the LP surge minimum migrates to higher Tt4; once it exceeds the redline (between
r=0.5 and r=0.3 here) relief_lp goes positive. The governor becomes a modest LP-surge lever in the
fast-accel limit.

## Table E — ρ sweep (flow/press, r=0.5, Tt4_max=1522): bare surge object ρ-flat, LP relief still zero

| ρ | Tt4_peak_bare | min_phi_lp_bare | relief_lp | relief_hp | held |
|---|---|---|---|---|---|
| 0.2 | 1554.4 | 0.74372 | +0.00000 | +0.00000 | True |
| 0.5 | 1600.8 | 0.74104 | +0.00000 | +0.00012 | True |
| 1.0 | 1644.9 | 0.73986 | +0.00000 | +0.00077 | True |
| 2.0 | 1680.5 | 0.73934 | +0.00000 | +0.00208 | True |
| 5.0 | 1708.5 | 0.73904 | +0.00000 | +0.00301 | True |

Bare `min_phi_lp` spread over the 25× ρ range = 0.63% (rung 45's ρ-flat surge object). `relief_lp`
is machine-zero at every ρ (the LP min stays below the redline for all ρ at moderate r). The HP
rebate carries a weak ρ trend (the clip amount tracks rung 43's ρ-loud overshoot) — disclaimed,
not calibrated.

## Table F — DECEL: clip never fires ⇒ bit-for-bit rung 45 (Tt4 1400→1000, r=0.5, Tt4_max=1480)

| shape | topped == bare (float-for-float) | Tt4_peak |
|---|---|---|
| flow/press | True | 1400.0 |
| press/flow | True | 1400.0 |
| tilted     | True | 1400.0 |
| hp-only    | True | 1400.0 |

Decel undershoots Tt4; no redline above the endpoint is ever reached; the topping governor is inert.
