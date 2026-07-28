# Rung 58 — the COMPOSITE MIN-SELECT: two levers DO NOT SUPERPOSE

Rung 57 put rung 53's floor-moving variable stator on the transient plant and found it has
**no clock**: across a 20× ramp-rate range the share of its rotation that survives moved 1.05
points, and rung 53's *design-point* closed form predicted it. That bounded the whole
rungs-46–52 engagement-timing family — their law is a property of POINT-movers.

Rung 57 armed nothing else, and named the composite as its own next seam:

> *"a real FADEC runs the VSV schedule **and** the accel schedule **and** the topping governor
> together, and rung 57's result makes the composite interesting rather than routine: one
> lever in the min-select is clocked and the other is not, so the pair cannot factorise the
> way rung 52 § 3 already said a two-lag cascade cannot. The obstacle is a currency, not code."*

This rung builds it, and the obstacle turns out to be **two** currencies, one of which is
fatal to a whole class of partners.

---

## THE HEADLINE

**Two levers in one min-select do not superpose — and a pair is composable at all only in
coordinates neither of them moves.**

Four cells on one plant (`neither` / `stator` / `fuel` / `both`) and their mixed second
difference `ΔI = [M(both) − M(fuel)] − [M(stator) − M(neither)]` — *how much the stator's
credit changes when a fuel-side leg is armed beside it*. No ranking of the two levers is
needed, which is exactly why this is measurable where rung 57's Concessions declared a
head-to-head **trapped** (fuel withheld and shaft speed paid have no common currency — rung
48's matched-accel-time trap, rung 43's currency circularity). A second difference in ONE
currency needs no exchange rate.

At `r = 0.5`, `margin = 0.25`, LP spool, `ds = 0.005`:

| cell | `M_i` | `s*` | `v(s*)` | min `φ_L` | `ν_H,end` | `s_eng` |
|---|---|---|---|---|---|---|
| neither | 0.458452 | 0.2336 | 0.0000 | 0.73545 | 0.952714 | — |
| stator  | 0.511179 | 0.2583 | 0.1465 | 0.68547 | 0.950101 | — |
| fuel    | 0.473521 | 0.1246 | 0.0000 | 0.74396 | 0.943163 | 0.12289 |
| both    | 0.531264 | 0.1251 | 0.1636 | 0.68949 | 0.940288 | 0.12269 |

    stator credit   bare +0.052727   fuel armed +0.057743   ΔI = +0.005017   =  +9.51 %
    fuel-leg s_eng  bare  0.1228893  stator armed 0.1226905                  =  −0.162 %

**The influence runs ONE WAY.** The fuel leg moves the stator's credit by 9.5 %; the stator
moves the fuel leg's engagement time by 0.16 % — a factor of 59, and the small number is
measured SUB-GRID, where `mf < mf_sched` could only resolve a whole `ds` cell.

**THE MECHANISM: relocation × state-feed.** The clocked leg RELOCATES the incidence minimum
(rung 48/50's truncated-descent law — the clip ARRESTS the `φ` descent, so the minimum moves
to the engagement edge): `s* 0.2336 → 0.1246`. A **state-fed** stator schedule is read at that
relocated point, and being closed at low speed it is **more closed** there: `v(s*) 0.1465 →
0.1636`, `+11.7 %`.

A CONSTANT setting has no state-feed, and its interaction is an order of magnitude down:
**+0.80 %** at `v = 0.20`, **+0.99 %** at `v = 0.10`, against the schedule's **+9.51 %**. That
floor is **real, not zero** — it survives `ds` halving (`0.92 % → 0.79 %` grid,
`0.86 % → 0.80 %` refined) — so it is published as a floor and never rounded away. It is also
an `r = 0.5` fact: see § The interaction has a clock.

So the non-additivity is **not** a plant coupling and **not** a re-timing: it is the clocked
lever choosing where the clock-free one gets read. The stator does not stop being clock-free —
see the next section, which is where this rung nearly went wrong.

### The interaction is PREDICTED by marches that never saw the fuel leg

The stator's credit is not a scalar but a **profile in `s`**: `M_i(armed, s) − M_i(bare, s)`,
point by point, off the two FUEL-LEG-FREE marches. The fuel leg does not reshape that profile
— it changes **which point of it is read**. Re-reading the profile at the relocated minimum
recovers

| leg | ΔI measured | ΔI predicted from the leg-free marches | recovered |
|---|---|---|---|
| schedule | +0.005017 | +0.004311 | **86 %** |
| constant | +0.000556 | +0.000599 | **108 %** |

The residual is the genuine plant coupling, and it is the **minority** channel. This is the
strongest form of the mechanism claim: the composite's non-additivity is computable from the
single-lever runs plus one number — the relocation distance.

---

## The second finding — the DECOMPOSITION is clocked, the DELIVERED CREDIT is not

This is where the rung nearly went wrong, so the near-miss is published with the result.

The interaction is strongly ramp-rate-dependent. The obvious reading — *"a clock-free lever
inherits its partner's clock"* — is **refuted by the same table**, because the quantity a
designer is handed is not `ΔI`, it is `credit_bare + ΔI`, the credit the stator actually buys
on the composite machine:

| `r` | credit bare | ΔI | **composed** | share | reloc | `v` ratio | `removed` |
|---|---|---|---|---|---|---|---|
| 0.10 | 0.055415 | +0.003157 | 0.058572 | +5.70 % | −0.0727 | 1.11152 | 6.22e−03 |
| 0.15 | 0.052284 | +0.006163 | 0.058447 | +11.79 % | −0.1114 | 1.17570 | 5.96e−03 |
| 0.25 | 0.051059 | +0.007368 | 0.058427 | **+14.43 %** | −0.1447 | 1.19640 | 5.42e−03 |
| 0.35 | 0.051798 | +0.006556 | 0.058354 | +12.66 % | −0.1450 | 1.15844 | 4.89e−03 |
| 0.50 | 0.052727 | +0.005017 | 0.057744 | +9.51 % | −0.1332 | 1.11662 | 4.11e−03 |
| 0.75 | 0.053859 | +0.002639 | 0.056498 | +4.90 % | −0.0901 | 1.06269 | 2.83e−03 |
| 1.00 | 0.054657 | +0.000184 | 0.054841 | +0.34 % | −0.0210 | 1.01276 | 1.61e−03 |
| 2.00 | 0.056380 | +0.000000 | 0.056380 | 0.00 % | +0.0000 | 1.00000 | **0.0 — dormant** |

**ΔI ANTI-CORRELATES with the bare credit** — it is largest exactly where the bare credit is
smallest — so the composed credit is **FLATTER in `r` than the bare one**:

| leg | bare credit spread | **composed credit spread** |
|---|---|---|
| schedule | 8.53 % | **6.80 %** |
| constant `v = 0.20` | 3.11 % | **0.89 %** (3.5× flatter) |

So **rung 57 is CONFIRMED, not bounded, on the thing it measured**: the credit a wall-moving
lever delivers is no more ramp-rate-dependent beside a clocked leg than alone, and for a
constant setting it is markedly *less*. What is clocked is the **decomposition** — how the
credit splits between the lever and the interaction — and a decomposition is not a
deliverable. The distinction between those two is the difference between a finding and a
currency artifact, and this project has been caught by that exact shape three times (rung 43's
circularity, rung 45's referenced excursion, rung 49's confound). It nearly was a fourth.

What survives, and is the rung: the two levers **do not superpose**, the non-additivity runs
one way, and it is predictable. That is rung 52 § 3's non-factorization, given a mechanism and
a predictor.

**P4 is therefore scored HIT on "the interaction has a clock", DIRECTION REFUTED, and its
INTERPRETATION corrected by its own table.** The direction: the share is **non-monotone,
peaking near `r ≈ 0.25`**, with three points on the rising limb — at the fastest ramps the leg
engages at `s = 0.0225`, so early that little `φ` descent is left to arrest, and the
relocation collapses.

**The two exact zeros are NOT the same object.** At `r = 2.00` the `margin = 0.25` leg is
**DORMANT** (`removed = 0.0` exactly): `fuel` is then bit-identical to `neither` and `both` to
`stator`, so `ΔI = 0` is the tautology `_one_leg` refuses at the door — reported as the
ENVELOPE EDGE (rung 48's `m → 0` corner precedent), never as evidence. At `r = 1.00` with a
constant setting the leg **does** bind (`removed = 1.61e−03`) and the zero is real: it engages
at `s = 0.2947`, DOWNSTREAM of the minimum, so it relocates nothing and re-prices nothing.
**Rung 48's engagement law, reappearing inside the composite.**

**And the constant leg's floor has a SIGN.** `+0.80 %` at `r = 0.5`, but `−2.98 %` at
`r = 0.10` and `−1.00 %` at `r = 0.15`. The residual plant-coupling channel has its own clock
and its own sign, so the order-of-magnitude split is an `r ≈ 0.25–0.75` statement.

---

## The third finding — a `φ`-referenced leg is NOT COMPOSABLE AT ALL

The pre-registered P5 asked whether the interaction survives with rung 49's `SurgeLimiter`
(feedback on the protected variable) in place of rung 48's feedforward `Wf/pt3` leg. The
answer is not a magnitude. **The leg cannot be held fixed across the four cells at all.**

A `φ` floor must sit **below** the machine's `φ` at `s = 0` — or it binds from the start and
the "acceleration" is a deceleration — and **above** its minimum `φ`, or it never binds:

| machine | `φ_L(s=0)` | min `φ_L` | admissible floor |
|---|---|---|---|
| bare | 0.773116 | 0.735442 | **(0.7354, 0.7731)** |
| schedule `v_max = 0.20` | 0.712823 | 0.685460 | **(0.6855, 0.7128)** |
| constant `v = 0.20` | 0.702329 | 0.670882 | (0.6709, 0.7023) |

**DISJOINT, by 0.0226.** The stator displaces the running line in `φ` by more than the ramp's
own `φ` excursion, so no single floor is the same instrument on both machines.

> **⚠ VERDICT CONFIRMED, DIAGNOSIS CORRECTED BY RUNG 60.** The disjointness is real and
> reproduces, but it is a **symptom, not the disease** — and it is repairable. Re-referenced to
> **incidence** (the one currency whose wall the stator does not move — this rung's own
> currency finding) the gap shrinks **24×** at a constant setting and **closes entirely on the
> `v_max = 0.20` schedule above**, so a single set point IS admissible on both machines. The
> leg is *still* not composable, because a floor that binds **PINS its own coordinate**: the
> second difference becomes a difference of SET POINTS with a derived value — exactly `v` for
> a `φ` floor, exactly `0` for an incidence floor. **The by-product below was the disease,
> filed here as a curiosity.** `docs/rung60-spec.md`.

This is rung 53's law reaching a **limiter's set point**. Rung 53 made a MARGIN
coordinate-dependent; rung 54 a CONSTRAINT'S SEVERITY; rung 56 a LEVER'S COST; rung 58 adds
**a LIMITER'S COMPOSABILITY**. Rung 48's leg composes precisely because its cap `Wf/pt3` is
stator-invariant — which is what the 0.16 % engagement shift measures.

### The by-product: a pinned floor annihilates rung 57's erosion, EXACTLY

At the floors that were tried the leg binds from `s = 0` on the armed machine, pinning BOTH
cells' incidence minima at `φ = φ_lim`. Then

    M_i(both) − M_i(fuel) = [T_c − 1/φ_lim + v] − [T_c − 1/φ_lim + 0] = v

— the stator's credit is **exactly the setting it commands there**, the POINTWISE credit, with
rung 57's erosion at **exactly zero**. Measured:

| leg | `credit_fuel` | `v(s*)` | \|diff\| |
|---|---|---|---|
| schedule | 0.16890367018595565 | 0.16890367018595367 | 2.0e−15 |
| constant | 0.20000000000000084 | 0.2 | 8.3e−16 |

and **identical at both floors (0.745, 0.750)**, which is what proves it is the pinning rather
than a coincidence. **Rung 60 promotes this from by-product to mechanism**: it is not a corner
of the `φ` leg but the general law that kills the whole floor family, and its other end — an
incidence floor giving credit exactly **0** — is measured there on the same plant. Rung 57's whole second law — two thirds of the rotation never arrives,
eaten by the lever's own WORK channel pushing the running line down — is **the work channel,
and a limiter that floors `φ` forbids precisely it.**

The `+220 %` "share" this produces is NOT published: its denominator comes from a different
regime, which is rung 43's currency circularity exactly.

---

## The currency is a FINDING, not a convention

Rung 53 proved a margin is a DISTANCE, so a floor-moving lever makes it coordinate-dependent.
A four-cell second difference therefore has to be read in an object whose wall is the same in
all four cells, or "non-additivity" is a coordinate artifact.

    M_i = T_c − (1/φ − v)      wall = the METAL: T_c off the DESIGN map, ONE number, shared
    M_φ = φ − φ_surge/(1+v·φ_surge)             wall MOVES with v

They **disagree on the sign** of the stator's own credit — `M_i` `+0.0527`, `M_φ` `−0.0070` —
and hence on the sign of the interaction (`+0.0050` vs `−0.0019`). Only `M_i` can carry this
rung. `M_φ` is reported per cell throughout and **never differenced**.

---

## The instrument

`ScheduledStatorTransient` (rung 57) grows a rung-58 section; the plant is untouched.

- `_stator_march(…, accel=, surge=, Tt4_max=)` — rung 57's march with ONE fuel-side
  min-select leg threaded through. All three default to `integrate_fuel`'s own defaults.
- `_one_leg(…)` — refuses two fuel legs at the door. Fuel-leg × fuel-leg is min-select
  ALGEBRA: whenever one binds the other contributes exactly zero, so the interaction is
  trivially `−credit(other)` — the tautological-gate failure mode of rungs 40/46.
- `_leg_residual(…)` / `_s_eng(…)` — the leg's engagement residual `g(s)` at the SCHEDULED
  fuel, one sign convention for all three legs, and its interpolated first upward zero. This
  is what makes 0.16 % measurable: `g` is continuous and the march is bit-identical to the
  unclipped one up to the crossing.
- `_refine_min(…)` — the incidence minimum, parabola-refined off the `ds` grid. The mechanism
  IS a relocation of one or two cells, so argmin and `v` there are both quantized without it.
- `_profile_credit(…)` — the credit as a callable profile in `s`.
- **`composite_credit(…)`** — THE RUNG: the four cells, `ΔI`, the mechanism prediction, and
  the deflation exclusion.
- `engagement_shift(…)` — the CONVERSE reading, on the limited AND the dormant march.
- `interaction_sweep(…)` — siblings on the same hardware (rung 53's `at_setting` discipline).

### The reduce

By **dispatch and by identity**, and the second is the strong one:

1. No fuel leg armed ⇒ `_stator_march` reaches the identical `integrate_fuel` call ⇒
   bit-for-bit rung 57 on every recorded key, on a bare, a scheduled and a constant machine.
2. A leg that is ARMED but never binds (a `margin = 0.60` accel schedule, a `φ_lim = 0.50`
   floor) leaves the march bit-identical to no leg at all — the composite machinery is
   witnessed **inert**, not merely skipped. Rung 57's "same map object" move, one ladder on.
3. Rung 57's own three readers never pass a leg, so its published constant-`v` erosion band
   (0.60–0.70) is reproduced unchanged.
4. The design run is bit-for-bit rung 6.

---

## Concessions

- **The fuel leg is ONE OBJECT, derived ONCE on the BARE machine.** Letting each cell derive
  its own would make the leg itself differ between cells and the second difference would
  isolate nothing. Every reader takes the leg as an argument so the choice is the caller's and
  is visible. **The discipline stands; its stated reason was FALSE — see the banner below.**

  > **⚠ CORRECTED BY RUNG 59.** This bullet used to read *"a stator-armed machine derives a
  > different `κ_ss` table … the matched-schedule variant is a different, confounded
  > experiment and is not this rung."* It is not a different experiment. `κ_ss` is a function
  > of `Tt4` **alone** — `A4` is choked so the corrected group is hardware, and `Tt3` is
  > pinned by the map-free shaft balances (rung 31's `(★)`) — so a schedule's **ordinate**
  > cannot see a stator on **either** spool, and its **abscissa** `n_H(Tt4)` is untouched by
  > an **LP** stator (rung 39's one arrow: `π_LPC` cancels out of the HP face). **This rung
  > ran an LP stator**, so the leg above already *is* the matched leg — identical not only in
  > the table but in `s_eng`, the fuel removed and `s*` — and **every number in this spec is
  > unconfounded**. The concession is discharged as VACUOUS, not as small. An **HP** stator
  > does re-index the table (+3.3–6.7 %), and there an unmatched leg manufactures an
  > interaction 48–96× too large and of the wrong sign on the statored spool. See
  > `docs/rung59-spec.md`.
- **The `+0.8 %` constant-setting floor is an `r = 0.5` number.** At the fastest ramp measured
  it goes **negative** (`−2.98 %` at `r = 0.10`): the residual plant-coupling channel has its
  own clock and its own sign. The order-of-magnitude split between a setting and a schedule is
  asserted where it was measured and nowhere else.
- **One spool, one currency, one gas.** LP-side, `M_i`, CPG — as rungs 53/57.
- **`φ_surge` is still rung 36's imposed constant**, and it anchors `T_c = 1/φ_surge`. Its
  LEVEL is disclaimed exactly as in rungs 36/41/53/57; the load-bearing objects here are a
  second difference, a machine-precision identity and a disjointness, none of which reads the
  level.
- **The HP schedule still reads `ν_H` and not its corrected speed** — rung 57's concession,
  inherited unchanged, and every claim here is LP-side.
- **No head-to-head.** Nothing below says which lever is better. The composite is an
  interaction term, which is the whole reason it dodges rung 57's trapped comparison.

---

## What it does to its neighbours

- **Rung 57 — CONFIRMED, on a test it did not run.** "A wall-moving lever has no clock" was
  measured on the lever alone. Put it beside a clocked leg and the DELIVERED credit gets
  *flatter*, not more clocked (8.53 % → 6.80 % for a schedule, 3.11 % → 0.89 % for a constant
  setting): the interaction absorbs the lever's own residual ramp-rate drift. Only the
  DECOMPOSITION is clocked, and that is not a deliverable.
- **Rung 53 — EXTENDED.** Its coordinate-dependence law reaches a third object: after a
  margin (53), a constraint's severity (54) and a lever's cost (56), a **limiter's
  composability**.
- **Rung 49 — BOUNDED.** Its `φ` limiter, the one leg that watches the protected variable, is
  the one leg that cannot be composed with a floor-moving lever at a fixed set point.
- **Rung 48 — CONFIRMED, from the other side.** Its engagement law is untouched by a
  wall-mover (0.16 %), and the reason is that its cap lives in `Wf/pt3`, a coordinate the
  stator does not move.
- **Rung 52 § 3 — CONFIRMED and given a mechanism.** It said a two-instrument cascade should
  not be additive. Here it is not, and the non-additivity has a name and a predictor.

---

## The next seam

**The matched schedule.** Re-derive `κ_ss` on the armed machine — what a FADEC actually burns
in — and ask whether the interaction survives when the leg is matched to the plant it runs on.
This rung refuses it as a confound *for isolating the mechanism*; with the mechanism now
isolated and predictable from the leg-free marches, the confounded experiment becomes
readable, because the prediction says what to subtract.

> **✔ BUILT — rung 59, and it needed no subtraction.** There was nothing to subtract *on this
> rung's machine*: the matched leg is bit-identical to the one used above. The seam was real
> only on the **other** spool. `docs/rung59-spec.md`.

Then, unchanged: **stator + bleed together** (rung 53's saturation), a **bleed schedule**
`b(n_L)`, and the **lag SHAPE / two-lag cascade** (rung 52's own seam).

## Anchor

`docs/plans/rung58-anchor-composite-minselect.md` — the four probes that fixed the currency
and the mechanism before the predictions (including probe D, the three blockers), the six
predictions as written, and their scoring:
**P1 HIT · P2 HIT · P3 HIT on monotonicity / PARTIAL on the corner · P4 HIT on the claim,
DIRECTION REFUTED · P5 REFUTED and replaced by a stronger negative · P6 HIT.**
