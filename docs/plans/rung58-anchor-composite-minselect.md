# Rung 58 anchor — the stator schedule BESIDE a fuel-side limiter, on ONE plant

The probes that fixed the currency and the mechanism **before** the predictions, the
predictions as written, and their scoring. Same discipline as rungs 37/44/46/49/56/57: probe
first, pre-register only once the instrument is trusted, then score honestly.

Everything below was run from `M:\claud_projects\temp\rung58`. Probes A–C ran against the repo as of
`634f8f4` (rung 57 shipped, no rung-58 code); probe D onward and **every § Scoring number**
ran against the rung-58 tree, and the shipped `composite_credit` reproduces probe A exactly.

**RESOLUTION — the probe tables below are the `ds = 0.01` reads; § Scoring and the spec are
`ds = 0.005`.** That is why the stator credit reads `0.052715` here and `0.052727` there, and
why `s*` is `0.23` here and `0.2336` there. Probe D § 2 is the convergence check that licenses
publishing only the refined set.

Common to every run: the CPG gas, `FLIGHT = (250 K, 50 kPa, M0 0.85)`,
`π_LPC/π_HPC/Tt4 = 3/6/1500`, the rung-49 `REAL` losses, the rung-53/57 shapes
`LP = (a .20, b .05, σ .1, l .7)`, `HP = (a .08, b .15, σ .1, l 1.0)`, both with
`φ_surge = 0.55`, and the rung-45 accel ramp `Tt4 1000 → 1400` (`s_settle = 1.2`).

**THE EXPERIMENT.** Four cells on ONE plant — `neither` / `stator` / `fuel` / `both` — and
the mixed second difference

    ΔI  =  [M(both) − M(fuel)]  −  [M(stator) − M(neither)]

i.e. *how much the stator's credit changes when a fuel-side leg is armed beside it*. No
ranking of the two levers is required, which is the whole reason this composite is
measurable where rung 57's Concessions declared a head-to-head trapped (fuel withheld and
shaft speed paid have no common currency — rung 48's matched-accel-time trap, rung 43's
currency circularity).

**THE FUEL LEG IS HELD FIXED ACROSS ALL FOUR CELLS.** `AccelSchedule` is derived ONCE, on the
BARE machine (`margin = 0.25`), and the same object is passed to the `fuel` and `both` cells.
This is an **isolation choice and it is not what a FADEC does**: rung 48's `accel_schedule`
reads `self.equilibrium`, so a stator-armed machine would derive a *different* `κ_ss` table.
Re-deriving per cell would let the leg itself differ between cells and the second difference
would no longer isolate anything. The matched-schedule variant is a **different, confounded
experiment** and is not this rung.

---

## The probes (instrument-fixing — NOT evidence for the predictions)

### A — the REFERENCE WALL: which currency can the composite even be read in?

The blocking question. Rung 53 showed a margin is a DISTANCE, so a floor-moving lever makes
it coordinate-dependent; the four cells would otherwise be differenced across two different
walls and "non-additivity" would be a coordinate artifact. Both rung-53/57 currencies, LP
spool, `r = 0.5`, schedule `v_max = 0.20, n_lo = 0.7557`:

| cell | `M_i` | `M_φ` | min `φ_L` | `s*` | `v(s*)` | `ν0_L` |
|---|---|---|---|---|---|---|
| neither | 0.458467 | +0.185448 | 0.73545 | 0.23 | 0.0000 | 0.7557 |
| stator  | 0.511182 | +0.178411 | 0.68547 | 0.26 | 0.1462 | 0.8166 |
| fuel    | 0.474031 | +0.193964 | 0.74396 | 0.12 | 0.0000 | 0.7557 |
| both    | 0.531787 | +0.184996 | 0.68949 | 0.13 | 0.1632 | 0.8166 |

|  | stator credit, bare | stator credit, fuel armed | ΔI |
|---|---|---|---|
| `M_i`   | **+0.052715** | **+0.057756** | **+0.005041** |
| `M_φ`   | −0.007037 | −0.008968 | −0.001931 |

**The two currencies disagree on the SIGN of the stator's own credit** (`M_i` positive,
`M_φ` negative) and therefore on the sign of the interaction. `M_i = T_c − (1/φ − v)` has its
wall at the **METAL** — `T_c = cmap.tan_beta1_crit()` off the *design* map, one number,
identical in all four cells — while `M_φ`'s wall `φ_surge/(1+v·φ_surge)` moves with `v`.
**Only `M_i` is a common object across the four cells, so it is the only currency in which
this rung has a headline.** `M_φ` is reported throughout and never differenced.

### B — WHERE the interaction lives: the constant-setting control

Same four cells, with the state-fed schedule replaced by a CONSTANT setting (rung 53's lever,
which has no state-feed at all):

| stator leg | stator credit bare | with fuel | ΔI | share |
|---|---|---|---|---|
| SCHEDULE `v_max = .20` | +0.052715 | +0.057756 | +0.005041 | **+9.6 %** |
| CONSTANT `v = .20` | +0.069139 | +0.069773 | +0.000634 | **+0.9 %** |
| CONSTANT `v = .1462` | +0.051488 | +0.051964 | +0.000476 | **+0.9 %** |

and the pointer to the mechanism: the fuel leg RELOCATES the incidence minimum
(`s* 0.23 → 0.12` bare, `0.26 → 0.13` armed — rung 48/50's truncated-descent law), and a
state-fed schedule is **more closed** at the earlier, slower point (`v(s*) 0.1462 → 0.1632`,
**+11.6 %**), which is within two points of the +9.6 % interaction. A constant setting cannot
do that, and its interaction is an order of magnitude smaller.

### C — the CONVERSE direction, sub-grid

Probe B's engagement read was grid-resolution (the first clipped point). The residual
`g(s) = mf_sched − accel.cap(n_H, pt3)` evaluated at the SCHEDULED fuel is continuous and
crosses zero AT the engagement, and the march is bit-identical to the unclipped one up to
that crossing, so interpolating `g` gives a true sub-grid `s_eng`:

| stator leg | `s_eng` (clipped march) | `s_eng` (unclipped) | shift vs bare |
|---|---|---|---|
| bare | 0.1228917 | 0.1229076 | — |
| SCHEDULE `v_max = .20` | 0.1226919 | 0.1227076 | **−0.16 %** |
| CONSTANT `v = .20` | 0.1230448 | 0.1230600 | **+0.12 %** |

**The coupling is ONE-WAY.** The stator moves the fuel leg's engagement time by less than two
parts in a thousand, while the fuel leg moves the stator's credit by ten percent.

### D — the advisor's three blockers, before pre-registration

1. **Is the +0.9 % constant-`v` residual discretization?** No. `ds 0.010 → 0.005` moves it
   `0.92 % → 0.79 %` (grid) and `0.86 % → 0.80 %` (parabolic-refined). It does **not** halve,
   so it is a **real second-order plant coupling** and is disclosed as a FLOOR, never rounded
   to zero. The headline is therefore "an order of magnitude", not "collapses".
2. **`s*` was read on the `ds` grid, but the mechanism IS the relocation.** Parabolic vertex
   fit on `M_i` around the argmin: interaction `+9.90 %` at `ds = .010`, `+9.51 %` at
   `ds = .005`; grid-read `+9.56 % → +9.51 %`. Converged at **≈ 9.5 %**, `v`-ratio
   `1.11648 → 1.11662`. Every published number is the refined one at `ds = 0.005`.
3. **Does the accel COMPLETE at `margin = 0.25`?** (Rung 48's `m → 0` corner degenerates into
   rung 44's ramp-rate lever, readable only where `ν_H,end` is unmoved.) The leg costs
   `Δν_H,end = −9.55e−03` without the stator and `−9.81e−03` with it — the **same** cost to
   2.7 %, while the credit moves 9.5 %. The interaction is not a re-measured ramp rate.

A by-product worth recording: the CONSTANT `v = 0.20` cell ends at `ν_L,end = 1.0433` — a 4 %
LP **overspeed** past design — where the schedule ends at `0.9560`. That is rung 53's "paid in
shaft speed" bill, and containing it is the state-fed schedule's own reason to exist.

---

## The predictions, as written (before any rung-58 code existed)

P1 and P2 restate what the probes already measured, so that the shipped gates pin them; P3–P5
are **open** and were written down before being run.

**P1 (CURRENCY).** The composite is readable in `M_i` and not in `M_φ`: the two disagree on
the sign of the stator's own credit, and `M_i`'s wall `T_c` is bit-identical across all four
cells while `M_φ`'s is not.

**P2 (ONE-WAY).** With a state-fed schedule, `|Δs_eng|/s_eng < 0.5 %` while
`ΔI / credit > 5 %`. The clocked lever hands the clock-free one its evaluation point; the
clock-free lever gives nothing back to the clock.

**P3 (MECHANISM — OPEN).** Sweep the schedule's knee `n_lo` at fixed `v_max` and fixed ramp
endpoints, so that the schedule's local slope `|dv/dn|` at the minimum changes while its
saturation value does not. Then `ΔI / credit` is **monotone** in `v(s*_armed)/v(s*_bare) − 1`,
and **falls to probe D's constant-`v` floor (≈ 0.8 %)** in the saturated corner where the
schedule holds `v_max` across the whole relocation interval. If instead `ΔI` survives at the
floor-crossing corner, the mechanism is NOT the state-feed and P3 is refuted.

**P4 (THE INTERACTION HAS A CLOCK — OPEN).** Rung 57 found the stator's own surviving share
ramp-rate-invariant to 1.05 points across a 20× `r` range — a wall-mover has no clock. The
INTERACTION, by contrast, is a relocation effect, and relocation is a clock property.
Prediction: `ΔI / credit` varies by **more than 2×** across the same `r` range, and it
**GROWS as `r` shrinks** (a faster ramp engages the leg at lower speeds, where the schedule is
more closed, so `Δv` is larger).

**P5 (LEG-INDEPENDENCE — OPEN).** Replace rung 48's FEEDFORWARD `Wf/pt3` leg with rung 49's
`SurgeLimiter` — feedback on the protected variable, a different sensed signal and a different
engaged window. The interaction keeps the **same sign** (positive) and stays at least 5× the
constant-`v` floor, because the mechanism is *relocation × state-feed* and rung 49's leg
relocates the minimum too (rung 50's headline). The MAGNITUDE is not predicted.

**P6 (NOT A RAMP-RATE ARTIFACT).** At the headline margin, the fuel leg's cost in `ν_H,end` is
the same with and without the stator to within 5 %, while `ΔI/credit` is ~10 %.

---

## Scoring

**P1 HIT · P2 HIT · P3 HIT on monotonicity, PARTIAL on the corner · P4 HIT on "it has a
clock", DIRECTION REFUTED, INTERPRETATION CORRECTED · P5 REFUTED, and replaced by a stronger
negative · P6 HIT.**

### P1 — **HIT**

Probe A's table is the gate. `M_i` credit `+0.0527`, `M_φ` credit `−0.0070`; `M_i`
interaction `+0.0050`, `M_φ` interaction `−0.0019`. Opposite signs in both rows.

### P2 — **HIT**

`ΔI/credit = +9.51 %` against `Δs_eng/s_eng = −0.162 %` (identical on the limited and the
dormant read, so no clip is contaminating it). A factor of 59.

### P3 — **HIT on monotonicity, PARTIAL on the saturated corner**

The knee sweep, `v_max = 0.20` and both ramp endpoints held, `r = 0.5`, `ds = 0.005`:

| `n_lo` | credit | ΔI | share | `v(s*)` bare → fuel | ratio |
|---|---|---|---|---|---|
| 0.6000 | +0.032646 | +0.003693 | **+11.31 %** | 0.08875 → 0.10067 | 1.13434 |
| 0.7000 | +0.044081 | +0.004657 | +10.56 % | 0.12114 → 0.13676 | 1.12897 |
| 0.7557 | +0.052727 | +0.005017 | +9.51 % | 0.14649 → 0.16357 | 1.11662 |
| 0.8200 | +0.065083 | +0.003627 | +5.57 % | 0.18583 → 0.19690 | 1.05955 |
| 0.8600 | +0.068784 | +0.000945 | +1.37 % | 0.20000 → 0.20000 | 1.00000 |
| 0.9000 | +0.068795 | +0.000935 | +1.36 % | 0.20000 → 0.20000 | 1.00000 |
| CONST 0.20 | +0.069147 | +0.000556 | **+0.80 %** | 0.20000 → 0.20000 | 1.00000 |
| CONST 0.10 | +0.035779 | +0.000354 | +0.99 % | 0.10000 → 0.10000 | 1.00000 |

**Monotone in the `v`-ratio, over a 8× range in the share.** `n_lo = 0.95` is outside the
envelope (the rung-40 two-shaft equilibrium does not converge at `Tt4 = 1400` with the LP
stators that closed at high speed) and is reported, not hidden.

**PARTIAL:** the saturated corner falls to `1.37 %`, not to the `0.80 %` constant floor — a
**7× collapse**, but a factor 1.7 short of the floor. The reason is checkable and is not a
defect: a schedule saturated AT THE MINIMUM is still not a constant setting, because it opens
again downstream (`v(0.94) = 0.0787` against a constant `0.20`), so the two machines' late
trajectories differ. The prediction should have said "toward", not "to".

### P4 — **HIT on the claim, its DIRECTION REFUTED**

Eight ramp rates, `ds = 0.005`, schedule `n_lo = 0.7557` and a constant `v = 0.20` beside it.
`removed_*` is `∫(schedule − applied) ds` — the DORMANCY check the advisor demanded before any
exact zero could be quoted:

| `r` | leg | credit | ΔI | share | reloc | `v` ratio | removed bare | removed armed | `s_eng` |
|---|---|---|---|---|---|---|---|---|---|
| 0.10 | sched | 0.055415 | +0.003157 | +5.70 % | −0.0727 | 1.11152 | 6.2226e−03 | 6.2342e−03 | 0.0225 |
| 0.10 | const | 0.071251 | −0.002125 | **−2.98 %** | −0.0726 | 1.00000 | 6.2226e−03 | 6.3010e−03 | 0.0225 |
| 0.15 | sched | 0.052284 | +0.006163 | +11.79 % | −0.1114 | 1.17570 | 5.9552e−03 | 5.9667e−03 | 0.0339 |
| 0.15 | const | 0.069840 | −0.000700 | −1.00 % | −0.1113 | 1.00000 | 5.9552e−03 | 6.0357e−03 | 0.0339 |
| 0.25 | sched | 0.051059 | +0.007368 | **+14.43 %** | −0.1447 | 1.19640 | 5.4214e−03 | 5.4332e−03 | 0.0574 |
| 0.25 | const | 0.069128 | +0.000403 | +0.58 % | −0.1331 | 1.00000 | 5.4214e−03 | 5.5066e−03 | 0.0574 |
| 0.35 | sched | 0.051798 | +0.006556 | +12.66 % | −0.1450 | 1.15844 | 4.8923e−03 | 4.9031e−03 | 0.0825 |
| 0.35 | const | 0.069105 | +0.000635 | +0.92 % | −0.1294 | 1.00000 | 4.8923e−03 | 4.9804e−03 | 0.0825 |
| 0.50 | sched | 0.052727 | +0.005017 | +9.51 % | −0.1332 | 1.11662 | 4.1086e−03 | 4.1171e−03 | 0.1229 |
| 0.50 | const | 0.069147 | +0.000556 | +0.80 % | −0.1127 | 1.00000 | 4.1086e−03 | 4.2004e−03 | 0.1229 |
| 0.75 | sched | 0.053859 | +0.002639 | +4.90 % | −0.0901 | 1.06269 | 2.8337e−03 | 2.8384e−03 | 0.1997 |
| 0.75 | const | 0.069232 | +0.000374 | +0.54 % | −0.0635 | 1.00000 | 2.8337e−03 | 2.9298e−03 | 0.1997 |
| 1.00 | sched | 0.054657 | +0.000184 | +0.34 % | −0.0210 | 1.01276 | 1.6089e−03 | 1.6095e−03 | 0.2947 |
| 1.00 | const | 0.069298 | **+0.000000** | +0.00 % | +0.0000 | 1.00000 | 1.6089e−03 | 1.7066e−03 | 0.2947 |
| 2.00 | sched | 0.056380 | +0.000000 | +0.00 % | +0.0000 | 1.00000 | **0.0** | **0.0** | nan |
| 2.00 | const | 0.069437 | +0.000000 | +0.00 % | +0.0000 | 1.00000 | **0.0** | **0.0** | nan |

**HIT on the claim.** Over the range where the leg engages the share runs `0.34 % → 14.43 %`
— a factor of 42 — while the lever's OWN credit runs `0.0511 → 0.0564`, ±5 %: rung 57's
ramp-rate invariance reproduced, with the interaction violating it beside it.

**DIRECTION REFUTED.** The prediction said the share grows monotonically as the ramp gets
faster. It does not: it is **non-monotone, peaking near `r ≈ 0.25`**, with THREE points on the
rising limb (0.10, 0.15, 0.25) and four falling. At the fastest ramp the leg engages at
`s = 0.0225`, so early that little `φ` descent is left to arrest, and the relocation collapses
to `−0.0727`. The share tracks the `v`-ratio row by row.

**The two exact zeros are NOT the same object, and only one of them is admissible.**
- `r = 2.00`: `removed = 0.0` **exactly** — the leg never binds. `fuel` is then bit-identical
  to `neither` and `both` to `stator`, and `ΔI = 0` is the tautology `_one_leg` refuses at the
  door. Reported as the ENVELOPE EDGE (rung 48's `m → 0` corner precedent), never as evidence.
- `r = 1.00`, constant leg: `removed = 1.6089e−03` — the leg **does** bind, and the zero is
  real. It engages at `s = 0.2947`, DOWNSTREAM of the minimum, so it relocates nothing
  (`reloc = +0.0000`) and the interaction is exactly zero. **That is rung 48's engagement law
  reappearing inside the composite**: a leg that engages downstream of a spool's minimum
  rebates nothing — and, it turns out, re-prices nothing either.

**The constant leg's floor has a SIGN.** `+0.80 %` at `r = 0.5` but `−2.98 %` at `r = 0.10`
and `−1.00 %` at `r = 0.15`. The residual plant-coupling channel has its own clock and its own
sign, so "an order-of-magnitude floor" is an `r ≈ 0.25–0.75` statement and is scoped as one.

**AND THE INTERPRETATION WAS WRONG, caught on this table before commit.** The reading this
rung was about to publish — *"a clock-free lever inherits its partner's clock"* — is refuted by
its own rows. `ΔI` ANTI-CORRELATES with `credit_bare`, so the quantity a designer is handed,
`credit_bare + ΔI`, is FLATTER in `r` than the bare credit:

| leg | bare spread | composed spread |
|---|---|---|
| schedule | 8.53 % | **6.80 %** |
| constant `v = 0.20` | 3.11 % | **0.89 %** |

So rung 57 is **CONFIRMED on the delivered credit**, and only its DECOMPOSITION is clocked — a
decomposition is not a deliverable. The advisor caught this by doing arithmetic on the table
above; no new run was needed, and the shipped headline was changed from "inherits a clock" to
"two levers do not superpose". Logged here because it is the fourth time this project has come
near a currency artifact (rungs 43, 45, 49) and the first time it was caught before shipping.

### P5 — **REFUTED as written, and the refutation is stronger than the prediction**

Rung 49's `SurgeLimiter` is **not composable with the stator at a fixed set point at all**,
and not for a quantitative reason. The floor must sit BELOW the machine's `φ` at `s = 0` (or
it binds from the start and the "acceleration" is a deceleration) and ABOVE its minimum `φ`
(or it never binds). Those admissible windows are:

| machine | `φ_L(s=0)` | min `φ_L` | admissible floor |
|---|---|---|---|
| bare | 0.773116 | 0.735442 | **(0.7354, 0.7731)** |
| schedule `v_max=.20` | 0.712823 | 0.685460 | **(0.6855, 0.7128)** |
| constant `v=.20` | 0.702329 | 0.670882 | (0.6709, 0.7023) |

**DISJOINT**, with a gap of 0.0226 — the stator displaces the running line in `φ` by more than
the ramp's own `φ` excursion. This is rung 53's coordinate-dependence reaching a limiter's
SET POINT, and it is a hard bound, not a magnitude.

At the floors that were tried (0.745, 0.750) the leg binds from `s = 0` on the armed machine,
which pins BOTH cells' incidence minima at `φ = φ_lim` exactly — and then

    M_i(both) − M_i(fuel) = [T_c − 1/φ_lim + v] − [T_c − 1/φ_lim + 0] = v

**to machine precision**: `credit_fuel = 0.16890367018595565` against
`v* = 0.16890367018595367` (schedule, |diff| 2.0e−15) and `0.20000000000000084` against
`0.2` (constant, |diff| 8.3e−16), IDENTICALLY for both floors. A limiter that pins the
protected variable forbids the work channel, so rung 57's erosion — two thirds of the
rotation — goes to **exactly zero**. That identity is published; the `+220 %` "share" it
produces is NOT, because its denominator comes from a different regime (rung 43's currency
circularity).

### P6 — **HIT**

`Δν_H,end` from the leg: `−9.551e−03` bare, `−9.812e−03` armed — the same cost to 2.7 %,
`fuel_removed` 4.1086e−03 vs 4.1171e−03 (0.2 %), while `ΔI/credit` is 9.5 %.

### A by-product past the pre-registration — the interaction is PREDICTED

The stator's credit is a **profile in `s`** (armed minus bare, point by point), not a scalar.
The fuel leg does not reshape that profile, it changes which point of it is read. Re-reading
the profile at the relocated minimum — from the two marches that **never saw the leg** —
recovers **86 %** of the schedule's interaction and **108 %** of the constant's. The residual
is the genuine plant coupling, and it is the minority channel.

---

## Where the probes live

`M:\claud_projects\temp\rung58\probe_a.py` (currency + the four cells),
`probe_b.py` (the constant-setting control), `probe_c.py` (sub-grid `s_eng`, both
directions), `probe_d.py` (the three blockers: residual convergence, parabolic-refined
minimum, `ν_H,end`).
