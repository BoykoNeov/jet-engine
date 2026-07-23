# Rung 44 anchor — the transient two-spool surge line

Verified data behind `docs/rung44-spec.md`. Design point throughout:
`build_two_spool_turbojet(gas, pi_lpc=3, pi_hpc=6, Tt4=1500, p0=50 kPa, nozzle_convergent=True)`,
`FLIGHT = (T0=250 K, p0=50 kPa, M0=0.85)`, losses
`pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96, eta_hpt=0.92, eta_lpt=0.90,
eta_m=0.99, pi_n=0.98`. CPG dual gas (rung 31/38–41's recipe): `R = (g-1)/g·cp` exactly,
`gamma_c=1.4, cp_c=1004, gamma_t=1.3, cp_t=1239, hPR=42.8e6`.

Disclosed map shapes (compressor islands only, `a_t = 0`), rung 39/40/41's set:

| key | LP map | HP map |
|---|---|---|
| `flow/press` | `a=.20 b=.05 sigma=.1 l=.7` | `a=.08 b=.15 sigma=.1 l=1.0` |
| `press/flow` | `a=.05 b=.20 sigma=.1 l=1.0` | `a=.20 b=.05 sigma=.1 l=.7` |
| `tilted` | `a=.14 b=.10 c=.06 sigma=.2 l=.85` | same |
| `steep` | `a=.25 b=.12 sigma=.3 l=1.2` | same |
| `hp-only` | `flat` | `a=.08 b=.15 sigma=.1 l=1.0` |

`hp-only` (LP map flat) is rung 40's **discriminator**: it carries NO complex mode
(`oscillatory_band = None`), so any effect present there is not the LP-map mode.

The excursion object: `phi_excursion(FLIGHT, Tt4_lo, dTt4, r_ramp, s_end, ds)` marches a linear
`Tt4` ramp from the running-line start (`nu0 = match(Tt4_lo)` speeds), `Tt4 → Tt4_lo+dTt4` over
`s ∈ [0, r_ramp]`, and returns the signed extremum of `phi(s) − phi_steady(Tt4(s))` per spool,
referenced to the **running line** at the instantaneous `Tt4`.

---

## 1. THE SPLIT SURVIVES DYNAMICALLY — accel `Tt4` 1000→1400, `r_ramp=0.5`, `rho=1.0`

| shape pair | `ext_lp` | `ext_hp` | `|ext_lp|/|ext_hp|` | complex band @1200 | `|Im/Re|max` |
|---|---|---|---|---|---|
| flow/press | −0.19411 | −0.10198 | 1.90 | (1.233, 2.082) | 0.1314 |
| press/flow | −0.18540 | −0.11539 | 1.61 | (1.160, 1.980) | 0.1341 |
| tilted | −0.19294 | −0.10972 | 1.76 | (1.198, 2.042) | 0.1338 |
| steep | −0.17498 | −0.09661 | 1.81 | (1.250, 2.402) | 0.1640 |
| **hp-only** | **−0.22601** | −0.10142 | **2.23** | **None** | 0.0000 |

Both spools negative (**toward surge**); LP eats 1.6–2.2×. DECEL 1400→1000 is the exact mirror
(all `ext > 0`, away from surge): flow/press `ext_lp=+0.17711 ext_hp=+0.09460`.

**The airtight leg** is the damping ratio: every `|Im/Re|max ≤ 0.164 < 0.25` ⇒ the ring e-folds
before a quarter cycle and cannot cross a line the steady point clears — the mode is surge-irrelevant
by its own strength. **Corroborating (not necessary):** `hp-only` (LP flat) has no complex mode yet
the LARGEST LP/HP ratio, so the asymmetry is present with no mode ⇒ the mode is not its cause. The
2.23-vs-1.90 *value* also swaps LP-shaped→flat (a loading change), so it is a **partial artifact,
named** (rung 41's discipline) — corroboration, not proof; the damping ratio carries the claim.

---

## 2. SCHEDULE-SLAVED — `rho`-invariant (flow/press, accel 1000→1400, `r_ramp=0.5`)

| `rho` | `ext_lp` | `ext_hp` | ratio | `s` at extremum |
|---|---|---|---|---|
| 0.2 | −0.19108 | −0.10512 | 1.82 | 0.50 |
| 0.5 | −0.19369 | −0.10334 | 1.87 | 0.50 |
| 1.0 | −0.19411 | −0.10198 | 1.90 | 0.50 |
| 2.0 | −0.19372 | −0.10101 | 1.92 | 0.50 |
| 5.0 | −0.19307 | −0.10029 | 1.93 | 0.50 |

`ext_lp` moves <2 % over a 25× `rho` range; the extremum sits at `s = r_ramp` (the ramp's end)
for every `rho`. The inter-spool clock ratio is **powerless** over the surge excursion — rung 40's
"schedule-slaved, not lead-governed" scope-limit, on the surge axis.

## 3. RAMP-RATE-driven (flow/press, `rho=1.0`, accel 1000→1400)

| `r_ramp` | `ext_lp` | `ext_hp` | `s` at extremum |
|---|---|---|---|
| 0.1 | −0.23129 | −0.12324 | 0.10 |
| 0.3 | −0.21194 | −0.11208 | 0.30 |
| 0.5 | −0.19411 | −0.10198 | 0.50 |
| 1.0 | −0.15611 | −0.08098 | 1.00 |
| 2.0 | −0.10390 | −0.05302 | 2.00 |
| 5.0 | −0.04679 | −0.02485 | 3.14 |

`|ext_lp|` monotone-increasing as `r_ramp` falls (−0.047 slow → −0.231 fast, ~5×). The faster the
`Tt4` slam relative to the shaft clock, the deeper the swing toward surge — *this* is the
governing variable.

---

## 4. REPORT THE CROSSING, GATE THE FLIP — `transient_surge_margin`

flow/press, accel 1000→1400, `r_ramp=0.3`, floors `phi_surge_lp=0.76`, `phi_surge_hp=0.55`:

```
margin_min_lp = -0.0192   (transient min phi_lp - phi_surge)   crossed_lp = True
steady_min_lp = +0.0131   (steady   min phi_lp - phi_surge)    -> steady CLEARS
margin_min_hp = +0.3256                                        crossed_hp = False
steady_min_hp = +0.3933
```

The LP transient dips **below** a floor that every steady point clears (0.7408 < 0.76 < 0.7731 in
raw phi) — the flip. The HP never approaches. The crossing DEPTH rides on the imposed floor and the
ramp rate (disclaimed); the load-bearing object is `margin_min_lp < steady_min_lp` on the accel
(and `>` on the decel) — the flip's SIGN.

---

## 5. Non-tautological gate (bare-math)

An independent closed-form CPG two-shaft accel closure (own thermodynamics, own choke bisection, own
forward speed lines, own 2-D running-line root, own Euler shaft march — NO `Gas`/`Component`/
`ComponentMap`/`TwoSpoolTransient` call) reproduces the shipped excursion's SIGN (negative, both
spools), the LP-over-HP ordering, and the schedule-slaving (`rho`-invariance + ramp-rate
monotonicity) on a shaped map. Lives in `tests/test_rung44.py::test_bare_math_*` — the rigorous
anchor, exactly as for rungs 31/33/38/39/40.

---

## Method (literature)

Cohen–Rogers–Saravanamuttoo *Gas Turbine Theory* Ch. 9: during an acceleration the working line
swings **above** the steady equilibrium running line toward the surge line, because the fuel step
outruns the shaft's inertial response; the margin consumed is the gap to the low-flow boundary.
Rung 44 applies this to **two** characteristics: the swing concentrates on the LP spool (rung 41's
`(†)` shielding, now transient) and is governed by the ramp rate, not the inter-spool clock ratio.
