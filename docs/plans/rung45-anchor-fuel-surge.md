# Rung 45 anchor — the transient two-spool surge line on the FUEL path

Verified data behind `docs/rung45-spec.md`. Design point throughout:
`build_two_spool_turbojet(gas, pi_lpc=3, pi_hpc=6, Tt4=1500, p0=50 kPa, nozzle_convergent=True)`,
`FLIGHT = (T0=250 K, p0=50 kPa, M0=0.85)`, losses
`pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96, eta_hpt=0.92, eta_lpt=0.90,
eta_m=0.99, pi_n=0.98`. CPG dual gas (rung 31/38–44's recipe): `R = (g-1)/g·cp` exactly,
`gamma_c=1.4, cp_c=1004, gamma_t=1.3, cp_t=1239, hPR=42.8e6`.

Map shapes (rung 39–44's set; `steep` dropped, the four below suffice incl. the discriminator):

| key | LP map | HP map |
|---|---|---|
| `flow/press` | `a=.20 b=.05 sigma=.1 l=.7` | `a=.08 b=.15 sigma=.1 l=1.0` |
| `press/flow` | `a=.05 b=.20 sigma=.1 l=1.0` | `a=.20 b=.05 sigma=.1 l=.7` |
| `tilted` | `a=.14 b=.10 c=.06 sigma=.2 l=.85` | same |
| `hp-only` | `flat` | `a=.08 b=.15 sigma=.1 l=1.0` |

The objects: `phi_excursion_fuel(FLIGHT, Tt4_lo, Tt4_hi, r, s_settle, ds)` marches a FUEL ramp
whose steady endpoints are `fuel_for_Tt4(Tt4_lo)→fuel_for_Tt4(Tt4_hi)` via `integrate_fuel`, and
returns the signed extremum of `phi(s) − phi_steady(Tt4_cmd(s))` per spool, referenced to the
COMMANDED (linear) `Tt4` ramp — NOT the overshooting output. `transient_surge_margin_fuel(…)`
puts the surge claim on the reference-free RAW transient min `phi` vs the commanded steady min.

---

## 1. THE HEADLINE — the currency trap (flow/press, accel 1000→1400, `r=0.5`)

| `rho` | raw `min_phi_lp` | `Tt4_peak` |
|---|---|---|
| 0.2 | 0.7393 | 1584.7 |
| 0.5 | 0.7366 | 1640.1 |
| 1.0 | 0.7355 | 1695.4 |
| 2.0 | 0.7349 | 1741.4 |
| 5.0 | 0.7346 | 1778.2 |

`min_phi_lp` spread over `rho` = **0.63%** (< rung 44's 2% bar). `Tt4_peak` spread = **~11.5%**.
The plant (`Tt4` overshoot, rung 43) is strongly `rho`-monotone; the reference-free surge object
is `rho`-invariant. `min_phi_lp` is weakly monotone in the SAME direction (0.7393→0.7346) — an
order weaker than the TIT channel, not decoupled. The output-`Tt4`-referenced excursion (rejected)
swings ~40% over the same `rho` — a moving-reference artifact.

## 2. FUEL ENLARGES the surge approach — raw `min_phi_lp`, matched `r`, `rho=1.0`

| `r` | fuel `min_phi_lp` | `Tt4`-control `min_phi_lp` (rung 44) |
|---|---|---|
| 1.0 | 0.7512 | 0.7611 |
| 0.5 | 0.7355 | 0.7515 |
| 0.3 | 0.7189 | 0.7408 |

Fuel deeper toward surge at every matched `r` (the `Tt4` overshoot amplifies the approach —
rung 35 on two shafts). Also the ramp-rate leg: fuel `min_phi_lp` monotone-deeper as `r` falls.

## 3. THE SPLIT SURVIVES, THE DOMINANCE COMPRESSES — `phi_excursion_fuel`, accel `r=0.5`, `rho=1`

| shape | `ext_lp` | `ext_hp` | ratio | decel `ext_lp` | decel `ext_hp` |
|---|---|---|---|---|---|
| flow/press | −0.18049 | −0.12246 | 1.47 | +0.18139 | +0.11712 |
| press/flow | −0.17490 | −0.14123 | 1.24 | +0.17337 | +0.13268 |
| tilted | −0.18062 | −0.13313 | 1.36 | +0.17964 | +0.12506 |
| hp-only | −0.21115 | −0.12123 | 1.74 | +0.21268 | +0.11609 |

Both spools toward surge on accel, mirror on decel, LP leads (`|ext_lp|>|ext_hp|`) every shape.
Ratio **1.24–1.74** vs rung 44's **1.6–2.2** — dominance compressed (the `Tt4` overshoot loads the
HP transient lag). The strong LP asymmetry moves to the raw margin (§4).

## 4. REPORT THE CROSSING, GATE THE FLIP — `transient_surge_margin_fuel`

flow/press, accel 1000→1400, `r=0.3`, floors `phi_surge_lp=0.746`, `phi_surge_hp=0.55`:

```
margin_min_lp = -0.0271   (raw transient min phi_lp - phi_surge)   crossed_lp = True
steady_min_lp = +0.0271   (commanded steady min - phi_surge)        -> steady CLEARS
margin_min_hp = +0.2836                                             crossed_hp = False
```

The LP transient dips below a floor every steady point clears (0.719 < 0.746 < 0.773 in raw phi);
the HP never approaches. Only the ACCEL is gated — a decel moves away from surge, so the raw min
phi relaxes onto the low-power steady point and the raw margin is degenerate there (`tr ≈ st ≈ 0`,
measured +0.00006 over a full settle). The decel mirror lives on the referenced excursion (§3).

---

## Method (literature)

Cohen–Rogers–Saravanamuttoo *Gas Turbine Theory* Ch. 9: during an acceleration the working line
swings above the steady running line toward surge, the fuel step outrunning the shaft's inertial
response. On the fuel path `Tt4` FLOATS, so the TIT overshoot (rung 35/43) and the surge approach
(rung 44) are the SAME transient in two currencies. Rung 45's contributions: the currency trap on
the surge axis (the `rho`-monotone overshoot does not reach the `rho`-invariant reference-free
surge object), the quantified enlargement vs `Tt4` control, the dominance compression of the
excursion ratio, and the report-crossing/gate-flip fuel-path surge object.
