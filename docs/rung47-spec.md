# Rung 47 — The lagged / actuator topping governor: realism breaks the redline hold and STILL misses the LP

**Scope.** `TwoSpoolFuelTransient.integrate_fuel(…, Tt4_max=…, tau_gov=…)` /
`_integrate_fuel_lagged`, with `tau_gov` threaded through `_fuel_ramp_march`,
`phi_excursion_fuel`, `transient_surge_margin_fuel`, `topping_relief`, plus
`topping_command_trace` — the **lag** on rung 46's TIT topping governor. Rung 46 modelled the
governor as an idealised **instantaneous** min-select fuel clip and closed on this concession:

> *"Idealised instantaneous governor — no actuator lag. … A lagged governor (a real fuel valve /
> a `τ_gov`) is left open; a slow-enough governor would smear the clip window and could reach
> earlier into the LP surge point — a natural next seam, not modelled here."*

Rung 47 models `τ_gov` and **REFUTES** that hope.

> **A first-order lag is a TRAILING-edge tool — it DELAYS the governor's action, it never
> ANTICIPATES. "Reach earlier into the LP surge point" needs phase LEAD, which no lag provides.
> The finding is formulation-independent and has two halves. (1) THE REFUTATION: `relief_lp = 0`
> EXACTLY for every `τ_gov` at moderate ramp rates — the topped-lagged march is bit-identical to
> the bare one up to the (late) engagement, upstream of which the EARLY LP surge minimum
> (`s ≈ 0.24`, `Tt4 ≈ 1374`) has already passed and self-recovered; the lag only ADDS fuel AFTER
> engagement. At the FAST ramp where rung 46's instantaneous governor DID reach the LP (the lever,
> `relief_lp > 0`), the lag ERODES that positive relief toward zero — strictly WORSE than the
> ideal. In NO regime does the lag reach the LP better than the instantaneous min-select. (2) THE
> COST OF REALISM: the lag DESTROYS rung 46's clean "the governor HOLDS the redline" (its gate 3).
> `Tt4` OVERSHOOTS the redline by an amount growing with `τ_gov` (~55→191 K in-band at `r = 0.5`,
> ~220→390 K at fast `r = 0.15`) — the classic topping overshoot — and the HP surge rebate ERODES
> toward zero. A real (lagged) governor is strictly worse than rung 46's idealisation: it breaks
> the TIT hold, erodes the HP surge rebate, and still misses the LP. THE SECONDARY: the overshoot
> lives in the sensing/limiter-LOOP lag, NOT the valve — a pure metering-VALVE-position lag is
> INERT on the accel because the binding topping command RISES monotonically. WHERE the lag lives
> decides whether it even overshoots.**

Rung 47 is the control-side realism counterpart of rung 46 and the culmination of the fuel-control
ladder (35 → 43 → 45 → 46 → 47). `tau_gov=None` leaves rung 46 **bit-for-bit** (the instantaneous
min-select), and the default `build_turbojet(…).run(…)` design run is **bit-for-bit rung 6**.

---

## The control law — the clip AMOUNT is a third state, lagged (the limiter-loop lag)

`_integrate_fuel_lagged` marches `(ν_L, ν_H, g)` where `g` is the **clip amount** (the fuel
REDUCTION below the schedule) — a THIRD state relaxing toward the instantaneous requirement:

```
required(ν, s) = max(0, schedule(s) − topping(ν, Tt4_max))     [0 unless the scheduled fuel
                  would overshoot the redline at this flow — the same rung-46 engagement guard]
dg/ds = (required − g) / τ_gov
mf_applied = schedule(s) − g
```

`required` is computed by the SAME quasi-steady closure (`_instant_fuel` / `_topping_fuel`) the
plant already uses, so no new physics enters — only a new **control time constant**. Because
`required` GROWS after engagement while `g` TRAILS it, `mf_applied` stays ABOVE `topping` ⇒ `Tt4`
overshoots the redline (the classic topping overshoot). Reduces: governor off / redline above
peak (`required ≡ 0`, `g ≡ 0`) is **rung 45**; `τ_gov → 0` (`g → required`, snapped) is rung 46's
instantaneous min-select; `τ_gov=None` dispatches to the untouched rung-46 2-state loop.

`τ_gov` is framed as the **sensing / limiter-loop lag** (the DOMINANT lag in a real TIT limiter,
far larger than valve slew — Cohen–Rogers, Ch. 9). It is NOT the metering-valve position: a
valve-position lag is a DIFFERENT subsystem, and it is **inert** here (see finding 4).

---

## Sign-space only — inherited from rungs 41/44/45/46, plus one imposed `τ_gov`

`Tt4_max` is IMPOSED (rung 41's `phi_surge` discipline) and `τ_gov` is an imposed clock — rung 47
makes **no** claim about the absolute redline or the absolute lag an engine carries. It delivers
**signs**: that the lag overshoots the redline (growing with `τ_gov`), that `relief_lp` stays 0 at
moderate `r` and is ERODED (never enhanced) at fast `r`, that `relief_hp` erodes, and that the
overshoot lives in the loop lag not the valve. The **rung-36 discipline holds: report the
crossing, gate the flip** — here the "flips" are the sign of the overshoot (0 → positive) and the
direction of the LP-relief change under the lag (eroded, not enhanced).

---

## THE FINDINGS (config: CPG gas, accel 1000→1400, `ρ = 1`; `tests/test_rung47.py` reproduces)

The gas is CPG for tractability; the finding is **gas-independent** (rung 35/43/45/46's carried
concession — the forward burner asserts against a non-equilibrium gas, and the reacting reduce is
the `Tt4`-control path). The bare peak here is ~1670 K; the redline 1480 sits in the gap.

### (1) THE COST OF REALISM — the lag overshoots the redline, the HP rebate erodes (`r = 0.5`, redline 1480, flow/press)

| `τ_gov` | overshoot (K) | `relief_lp` | `relief_hp` | held |
|---|---|---|---|---|
| inst (None) | +0.0 | +0.0000000 | +0.005267 | **True** |
| 0.05 | +55.6 | +0.0000000 | +0.003560 | False |
| 0.10 | +95.3 | +0.0000000 | +0.002873 | False |
| 0.20 | +137.5 | +0.0000000 | +0.002135 | False |
| 0.40 | +170.2 | +0.0000000 | +0.001410 | False |
| 0.80 | +190.9 | +0.0000000 | +0.000860 | False |

`overshoot = Tt4_peak_top − Tt4_max`. The instantaneous governor holds (`overshoot = 0`); ANY
finite lag breaks the hold, and the overshoot grows monotonically with `τ_gov` (the classic
topping overshoot). The HP rebate erodes monotonically toward zero (the clip is softer, later).
**This inverts rung 46's gate 3** ("the governor holds `Tt4` at the redline").

### (2) THE REFUTATION — `relief_lp = 0` EXACTLY, every `τ_gov`, every shape (`τ_gov = 0.2`, redline 1480)

| shape | overshoot (K) | `relief_lp` | `relief_hp` | held |
|---|---|---|---|---|
| flow/press | +137.5 | +0.0000000 | +0.002135 | False |
| press/flow | +141.1 | +0.0000000 | +0.002985 | False |
| tilted     | +140.6 | +0.0000000 | +0.002601 | False |
| hp-only    | +134.6 | +0.0000000 | +0.001992 | False |

The LP relief is machine-zero at every shape, incl. **hp-only** (LP map flat, no rung-40 complex
mode) — so the refutation is the **timing/window** mechanism, not a mode artifact. **Why exactly
zero:** the LP surge minimum (`s ≈ 0.24`, `Tt4 ≈ 1374`) falls UPSTREAM of engagement, where the
topped-lagged trajectory is bit-identical to the bare one (`g ≡ 0` until the clip engages, which
is late). A causal first-order lag acts on the trailing edge; it cannot reach a leading-edge
minimum. The concession's "reach earlier" needs phase LEAD — no lag provides it.

### (3) THE LEVER, LAGGED — at fast `r` the lag ERODES rung 46's positive LP relief (`r = 0.15`, redline 1440, flow/press)

| `τ_gov` | overshoot (K) | `relief_lp` | `relief_hp` |
|---|---|---|---|
| inst (None) | +0.0 | **+0.02687** | +0.05088 |
| 0.05 | +218.9 | +0.01512 | +0.02568 |
| 0.10 | +292.3 | +0.01033 | +0.01534 |
| 0.20 | +343.8 | +0.00603 | +0.00843 |
| 0.40 | +388.5 | +0.00317 | +0.00443 |

This is the airtight half. At the fast ramp rung 46's **instantaneous** governor DOES reach the LP
(`relief_lp = +0.027`, the lever — the LP surge minimum has migrated above the redline into the
clip window). If a lag could "reach earlier into the LP surge point," the lagged `relief_lp` would
EXCEED the instantaneous. It does the **opposite**: the lag erodes it monotonically toward zero
(as `τ_gov → ∞`, `g → 0`, the topped march → bare, relief → 0). So in BOTH regimes the lag reaches
the LP no better than the ideal min-select — **neutral at moderate `r`, strictly worse at fast
`r`**. And the overshoot is enormous (~220→390 K) — the fast ramp is where the lag hurts most.

### (4) THE SECONDARY — the overshoot lives in the LOOP lag, not the valve (redline 1480, `r = 0.5`)

`topping_command_trace` marches the rung-46 instantaneous topped accel and reads the applied fuel
(the min-select topping set-point) over the engaged window: **monotone non-decreasing** (45
engaged points, `mf` rising 0.01766 → 0.02334). This is WHY a pure metering-VALVE-position lag is
**inert** on the accel: once the governor engages, the binding topping command RISES monotonically
(`ν ↑ ⇒ airflow ↑ ⇒ more fuel to hold the redline`), so an instant-up / lag-down valve tracks it
with no lag (probe: valve-slew gives zero overshoot / zero change at both `r = 0.5` and
`r = 0.15`). The topping OVERSHOOT therefore lives specifically in the sensing/limiter-LOOP lag
(which lags the clip AMOUNT). **WHERE the lag lives decides whether it even overshoots** — a real
sharpening. (Stated as observed on the tested accels; a decel or a non-monotone command could
differ.)

---

## Reduce-to-prior contract (the spine)

- **`τ_gov=None` ⇒ rung 46 bit-for-bit.** The instantaneous path is the untouched 2-state loop;
  `integrate_fuel` / `topping_relief` return float-for-float the rung-46 result (the new `mf` /
  `overshoot` / `tau_gov` keys are additive; the rung-46 numeric keys are byte-unchanged, and
  `tests/test_rung46.py` still passes).
- **Dormant lag ⇒ rung 45 bit-for-bit.** A redline above the bare peak leaves `required ≡ 0` ⇒
  `g ≡ 0` ⇒ `mf_applied ≡ schedule` ⇒ the lagged march equals the bare rung-45 march at any
  `τ_gov`.
- **`lp_disabled` ASSERTS + `τ_gov` needs a redline.** The split is inherently two-shaft (rung
  46's contract); `integrate_fuel(Tt4_max=…, tau_gov=…)` asserts on the degenerate engine, and
  `τ_gov` without `Tt4_max` asserts (a lag with no governor is meaningless).
- **Decel ⇒ rung 45 bit-for-bit** (finding echo): the clip never fires on a deceleration, so the
  lagged decel march equals the bare one at every shape.
- **Cycle untouched ⇒ rung 6 bit-for-bit.** The governor lives only on the two-spool transient.

### Why rung 47 carries no independent bare-math gate (rung 45/46's precedent)

Rung 47 marches rung 43's `integrate_fuel`, anchored transitively to rung 40's steady manifold.
The genuinely new content is (a) the lagged FEEDBACK, gated by the reduces and the overshoot's
growth with `τ_gov`, and (b) the surge-relief **signs** under the lag — shape-robust directions
(overshoot up, `relief_lp = 0`/eroded, `relief_hp` eroded), not magnitudes a bare-math replica
would constrain (rungs 42/43/45/46 set this precedent).

---

## Verification gates (`tests/test_rung47.py`)

1. **REDUCE — `τ_gov=None` bit-for-bit rung 46.** `integrate_fuel` point tuples and
   `topping_relief` (`relief_*`, `Tt4_peak_top`, `held`) identical to the no-`τ_gov` call; the
   instantaneous governor holds.
2. **REDUCE — dormant lag bit-for-bit rung 45.** Redline above the bare peak ⇒ the lagged march
   equals the bare rung-45 march float-for-float at `τ_gov = 0.3`.
3. **REDUCE — `lp_disabled` asserts + `τ_gov` needs a redline.**
4. **REDUCE — decel bit-for-bit rung 45.** Lagged decel `== ` bare decel at every shape.
5. **CYCLE UNTOUCHED ⇒ rung 6.** Default design run bit-for-bit after exercising the lagged governor.
6. **THE HEADLINE.** Every shape (incl. hp-only) at `τ_gov = 0.2`: `not held` and `overshoot > 1`;
   `relief_lp ≈ 0` (below machine tol); `0 < relief_hp < ` the instantaneous rebate.
7. **MONOTONE COST.** Over a `τ_gov` sweep (flow/press): `overshoot` grows, `relief_hp` erodes,
   `relief_lp` pinned at 0.
8. **THE LEVER, LAGGED.** At `r = 0.15` where the instantaneous `relief_lp > 0`, the lagged
   `relief_lp` is strictly LESS than the instantaneous AND decreasing with `τ_gov` (eroded, never
   enhanced), while it overshoots hugely.
9. **THE SECONDARY.** `topping_command_trace` is monotone non-decreasing over a real engaged
   window (the reason a valve-position lag is inert).

---

## Concessions

- **`τ_gov` imposed — no lag-level claim.** The absolute lag, the overshoot magnitudes, and the
  fast-ramp erosion rate are all disclaimed; load-bearing are the overshoot's SIGN and growth,
  `relief_lp = 0`/eroded, `relief_hp` erosion, and the reduces.
- **The refutation is sign/existence.** `relief_lp = 0` is the machine-zero of "the lag never
  touched the early LP min"; at fast `r` it is "strictly less than the instantaneous and
  decreasing." Neither the overshoot nor the rebate magnitude is gated.
- **Loop lag, not valve lag — a modelling choice, justified by finding 4.** The overshoot is
  produced by lagging the clip AMOUNT (the limiter-loop bandwidth), not the valve position, which
  is inert on the accel. The **coupled single-valve lag** (lagging the whole fuel path, schedule
  included) was rejected: it de-fangs the accel (the bare `Tt4` peak collapses ~1698 → 1462 K
  from sluggishness alone), **breaks the rung-45 dormant reduce**, and its apparent LP relief is
  the accel being slowed, not the clip reaching earlier — a confound of the rung-42/43/45
  currency kind. Isolating the governor (bare = rung 45 exactly, `τ_gov → 0` = rung 46 exactly) is
  what makes the differential admissible.
- **The valve-inert claim is stated as observed** on the tested monotone-command accels, not as a
  universal (a non-monotone command — e.g. a step schedule, or a decel — could make a valve lag
  bite).
- **`ρ` is a DISCLAIMED clock group, doubled** (inherited): the overshoot/rebate carry a weak `ρ`
  trend (they track rung 43's `ρ`-loud overshoot) but are not calibrated.
- **Reacting-gas fuel control deferred** (rung 35/43/45/46, verbatim): the governor runs on the
  non-equilibrium gases; the finding is gas-independent and the reacting reduce is the
  `Tt4`-control path (rung 44, bit-for-bit rung 6 at the cycle).
- **Idealised single first-order lag, both NGVs choked, no bypass, one `eta_m`, isentropic knobs,
  no bleed** — inherited from rungs 38–46. No lead/anticipation, no rate-limit, no sensor-plus-
  actuator cascade is modelled (a lead-compensated governor COULD reach the LP — the one thing a
  pure lag cannot; that is the open door this rung leaves, having shut the pure-lag one).

---

## Anchor

`docs/plans/rung47-anchor-lagged-governor.md`. The **method** is again Cohen–Rogers–Saravanamuttoo
*Gas Turbine Theory* Ch. 9 — the acceleration fuel schedule limited by a maximum turbine
temperature, now with the **limiter's finite response lag** (the topping overshoot). Rung 47's own
contribution is the **refutation** of rung 46's next-seam hope (a trailing-edge lag cannot reach
the leading-edge LP surge minimum: `relief_lp = 0` at moderate `r`, eroded at fast `r`), the
**cost of realism** (the lag breaks the redline hold — overshoot growing with `τ_gov` — and erodes
the HP rebate), and the **valve-vs-loop-lag contrast** (the overshoot lives in the loop lag; a
valve-position lag is inert because the binding topping command is monotone-rising).
