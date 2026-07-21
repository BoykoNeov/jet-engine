# Rung-35 anchor — fuel is the control, `Tt4` is an output

Two-part anchor, mirroring rungs 31–34. Part A is the **method** (the standard fuel-metered
acceleration picture — the acceleration line between the surge and the turbine-inlet-temperature
limits). Part B is the **reduce gate** (control-invariance: the fuel-control equilibrium reproduces
the Tt4-control running-line point via the independent forward-burner closure) plus the finding data.

Design REFERENCE = the rung-34 setup unchanged: the choked-**convergent** design point
(`π_c=10, Tt4=1500, M0=0.85`), real losses (`pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
eta_t=0.90, eta_m=0.99, pi_n=0.98`), `nozzle_convergent=True`, fixed throats `A4, A8` from rung 31,
and rung 32's `ComponentMap` shapes (with rung 34's linear loading slope `l`). Fast
`thermally_perfect` gas — the transient physics is gas-independent (rungs 32–34 precedent).

## Part A — the method (fuel-metered acceleration)

Standard gas-turbine acceleration model (Cohen–Rogers–Saravanamuttoo *Gas Turbine Theory* Ch. 9;
Walsh & Fletcher *Gas Turbine Performance*, transient / fuel-schedule chapters). A fuel-metering unit
sets the fuel **mass flow**; the turbine-inlet temperature `Tt4` is an **output** of the burner
balance against the airflow the spool can currently pump. During an acceleration the running point is
bounded **above** by the surge line and by the **TIT (over-temperature) limit**; the fuel schedule is
shaped to thread between them. At a lagging spool the fuel-air ratio and `Tt4` **overshoot**.

**The forward-burner + fuel closure at `(N, ṁ_fuel)`** — the structural novelty vs rung 34 (which
pinned `Tt4`):

```
n      = ν·√(Tt2_d/Tt2)                                        # corrected speed from ν=N/N_d
τ_c    = 1 + (τ_c,d−1)·ψ(φ)·n² ,  φ=m/n                        # Euler speed line, FORWARD (rung 34)
π_c    = pr_c(Tt3s)/pr_c(Tt2) ,  pt4 = π_b·π_c·pt2
ṁ_air  = m·ṁ_corr,d·pt2/√Tt2                                  # m fixes the compressor-face airflow
f      = ṁ_fuel/ṁ_air                                         # FUEL imposed ⇒ f is an OUTPUT
Tt4    = T_from_h_t( (h_c(Tt3)+f·η_b·hPR)/(1+f) , f )         # forward burner ⇒ Tt4 an OUTPUT
g(m)   = m − (A4·pt4·MFP*(Tt4,f)/√Tt4 /(1+f))·√Tt2/(pt2·ṁ_corr,d)   # trial m vs NGV-implied m
```

Root-find `g(m)=0`; the instant then flows into rung 34's shared `_instant_tail` (turbine dispatch,
shaft ODE `Φ`, thrust). The **airflow lag** is the whole rung: at a frozen spool a fuel step raises
`f`→`Tt4`, the hot NGV passes less corrected mass, and the operating point climbs above the running
line further than the commanded-`Tt4` acceleration did.

## Part B — the reduce gates and the finding

### Reduce — control-invariance (non-tautological)

A steady point is the same point however it is named. With `ṁ_fuel = f_eq·ṁ_air,eq` of a Tt4-control
running-line point, `equilibrium_fuel` returns the same instant via the forward-burner closure:

| `Tt4` | `ν` (Tt4-ctrl / fuel-ctrl) | `π_c` (Tt4 / fuel) | `Tt4_out` |
|-------|-----------|-----------|-----------|
| 1500 | 1.00000000 / 1.00000000 (Δ 2e-13) | 10.000000 / 10.000000 | 1500.000 (Δ 4e-10) |
| 1300 | 0.90625317 / 0.90625317 (Δ 1e-12) | 7.834843 / 7.834843 | 1300.000 (Δ 2e-9) |
| 1100 | 0.81502514 / 0.81502514 (Δ 2e-13) | 6.025913 / 6.025913 | 1100.000 (Δ 2e-9) |

Two genuinely different closures (forward burner + fuel vs pinned `Tt4`) onto one operating point.

### The finding — fuel control enlarges the surge excursion; the TIT overshoot

Fast gas, `surge_flow` map, acceleration `Tt4` 1100→1400, endpoints pinned to steady fuels:

| `r = τ_fuel/τ_spool` | `E_surge` Tt4-ctrl (rung 34) | `E_surge` fuel-ctrl | gap | `E_temp` | `Tt4_peak` |
|------|------|------|------|------|------|
| 0 (algebraic) | 5.39% | 10.16% | **4.77%** | 59.5% | **1754 K** |
| 0.3 | 4.71% | 7.69% | 2.98% | 37.9% | 1629 K |
| 1.0 | 3.47% | 4.26% | 0.79% | 18.0% | 1507 K |
| 3.0 | 1.67% | 1.78% | 0.11% | 7.6% | 1434 K |

`E_surge`/`E_temp` are referenced to the running line at the current speed (`E_temp` is the `E_surge`
analogue, not an overshoot above the target). `Tt4_peak` is the absolute turbine-inlet temperature: the
`r→0` step peaks at **1754 K, +25% over the 1400 K target** (and +59.5% over the frozen-speed
running-line value, which is what `E_temp` reports) — the number a real engine reads against a TIT
redline. Magnitude claim rests on the `r→0` step (both control modes are steps, unconfounded); the
finite-`r` gap mixes the coupling with a forcing-shape difference (linear-`Tt4` vs linear-`ṁ_fuel`).

- `E_surge(fuel) > E_Tt4` at every `r`; gap **max at `r→0`**, **vanishing as `r→∞`** (0.11% at r=3).
  The correction of rung 34: commanding `Tt4` suppressed the over-temperature that amplifies the
  airflow deficit, so rung 34 **under-counted** the surge excursion a fuel-metered engine sees.
- **Shape-robust sign** (r→0, `Tt4` 1300→1500, three surge maps):

  | map | `E_Tt4` | `E_surge` fuel | gap | `E_temp` |
  |-----|------|------|------|------|
  | surge_flow | 3.84% | 7.36% | +3.52% | 31.7% |
  | surge_pressure | 4.68% | 8.77% | +4.09% | 30.3% |
  | surge_tilted | 4.24% | 8.06% | +3.81% | 31.1% |

- `E_temp` (the TIT overshoot) is a separate, on these maps **larger** acceleration limit; monotone
  in `r`, `r→0` = the algebraic constant-`N` map displacement. Magnitude disclaimed (rung-32
  methodology); only the sign of the correction and the existence of the overshoot are claimed.

### Forward-burner inverse

`Tt4(f)` solved from the enthalpy balance inverts the shipped burner `f`-solve to <1e-10 (gate 4);
the fuel closure at a Tt4-instant's fuel recovers `(Tt4, π_c, ṁ_air)` off the running line too.
