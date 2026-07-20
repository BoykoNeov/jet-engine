# Rung 29 anchor — the shifting-turbine bracket

Verified data behind `docs/rung29-spec.md`. Regenerate with the shipped API
(`Gas.shifting_turbine`); every number below came from it, not from a probe.

## Configuration

```
FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
REAL_LOSSES = pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98
gas = Gas.reacting_equilibrium()      # rung-6 dissociating products, frozen downstream
delta_h = (h_c(Tt3) - h_c(Tt2)) / (eta_m * (1 + far))     # the shaft-set drop, engine-owned
```

## The bracket

| `Tt4` [K] | `far` | `T5` frozen [K] | `T5` shifting [K] | `ΔT5` [K] | `ΔT5/T5` | `Δp5/p5` | earned |
|------|---------|-----------|----------|--------|----------|----------|--------|
| 1500 | 0.02718 | 1262.69 | 1262.83 | +0.13 | +0.0107% | +0.0051% | YES |
| 1800 | 0.03788 | 1576.20 | 1577.41 | +1.21 | +0.0765% | +0.0282% | YES |
| 2100 | 0.04980 | 1888.03 | 1894.98 | +6.95 | +0.3680% | +0.1121% | no |
| 2400 | 0.06590 | 2200.06 | 2240.96 | +40.90 | +1.8592% | +0.4653% | no |

`pt4 = 7.47 bar` at every row (the burner's exit pressure does not depend on `Tt4`).

**Reduce check (gate 1), exact.** The frozen column reproduces the shipped `Turbine.apply`
at `eta_t = 1` bit-for-bit — asserted with `==`:

```
Tt5 = gas.T_from_h_t(gas.h_t(Tt4, far) - delta_h, far)
pt5 = pt4 * gas.pr_t(Tt5, far) / gas.pr_t(Tt4, far)
```

The independent work-limited bisection (gate 2) agrees with it to < 1e-6 relative — a
separate code path (mixture entropy + `_mix_h_abs_B`) onto the same physics.

## The inversion — ratio vs inventory

| `Tt4` [K] | super-eq ratio (max over CO/OH/O/H/H2) | inventory `x_O+x_H+x_OH` | `ΔT5/T5` |
|------|--------|-----------|----------|
| 1500 | 109.4× | 3.178e-05 | +0.0107% |
| 1800 | 17.7×  | 3.106e-04 | +0.0765% |
| 2100 | 6.6×   | 1.522e-03 | +0.3680% |
| 2400 | 3.3×   | 3.835e-03 | +1.8592% |

Span across the band: ratio **÷33**, inventory **×121**, shift **×174**. The inventory
tracks the shift (both ~2 orders); the ratio moves the other way.

Per-species super-equilibrium of the frozen pool at station 5, `Tt4 = 1500` vs `2400`
(`x_frozen / x_eq`) — the anti-correlation is uniform across species, not an artefact of
picking the max:

| species | 1500 K | 2400 K |
|---------|--------|--------|
| CO | 47.4× | 2.32× |
| OH | 9.58× | 2.01× |
| O  | 31.5× | 3.02× |
| H  | 109.4× | 3.32× |
| H2 | 29.6× | 2.13× |

## Downstream consequence (rung-14/25 nozzle bounds from each station 5)

`p9 = 50 kPa`, `pt9 = pi_n * p5`. Velocities m/s.

| `Tt4` | entry | `V_F` | `V_I` | `V_R` | `(I−F)` | `(R−I)` | `(R−I)/(R−F)` |
|------|---------|---------|---------|---------|--------|--------|--------|
| 1500 | frozen  | 1060.183 | 1060.244 | 1060.255 | 0.0611 | 0.0109 | 0.1510 |
| 1500 | shifted | 1060.249 | 1060.255 | 1060.255 | 0.0057 | ~0 | ~0 |
| 1800 | frozen  | 1225.694 | 1226.262 | 1226.326 | 0.5678 | 0.0647 | 0.1022 |
| 1800 | shifted | 1226.209 | 1226.327 | 1226.327 | 0.1173 | ~0 | ~0 |
| 2100 | frozen  | 1374.054 | 1377.467 | 1377.741 | 3.4128 | 0.2743 | 0.0744 |
| 2100 | shifted | 1376.695 | 1377.743 | 1377.743 | 1.0480 | ~0 | ~0 |
| 2400 | frozen  | 1515.249 | 1540.818 | 1542.053 | 25.5690 | 1.2350 | 0.0461 |
| 2400 | shifted | 1529.196 | 1542.072 | 1542.069 | 12.8759 | ~0 | ~0 |

The frozen-entry `(R−I)/(R−F)` = 7.4% at `Tt4=2100` corroborates rung 25's quoted "~7% of
the bracket hot" — an independent re-derivation of a shipped number.

**`(R−I) → 0` on the shifted entry is STRUCTURAL, not a measurement** (an entry at
equilibrium has no super-equilibrium left to relax irreversibly). The reported ~0 is
-2.6e-3 m/s at `Tt4=2400`, i.e. bisection tolerance. Not quoted as a finding — see the spec.

What *is* substantive: the `(I−F)` bracket itself **halves** hot (25.57 → 12.88 m/s), so
about half the recombination benefit rung 14 attributes to the nozzle would already have
been banked upstream on a shifting path.

## Supporting sketch — the rate (NOT a claim)

The rung-26 anchored clock `_tau_chem_recomb` read at station 4:

| `Tt4` [K] | `τ_chem(4)` [s] | `Da_turb` over `τ_res` = 5e-5 … 5e-4 s |
|------|-----------|------------------|
| 1500 | 9.108e-04 | 0.05 … 0.55 |
| 1800 | 1.987e-04 | 0.25 … 2.52 |
| 2100 | 7.843e-05 | 0.64 … 6.38 |
| 2400 | 5.676e-05 | 0.88 … 8.81 |

Transitional, and notably **not fast despite the high pressure** — the residence time is
short too. But `τ_res` here is **un-anchored** (an order-of-magnitude guess at a turbine
passage), so this is a sketch supporting "the real turbine does not even reach the bound at
the design point," not a rung claim. Do not quote `Da_turb` as a result.
