# Rung 53 — the VARIABLE STATOR: what a margin *is*, when the lever moves the wall

Rung 42 shipped the interstage bleed valve and named this rung in its own header:

> *"And bleed moves the operating point `phi_op`; it does NOT move the stall floor
> `phi_surge` — that is the variable-stator half of the seam, still open."*

Every surge lever in this project so far moves the OPERATING POINT against a FIXED wall: the
throttle (36/41), the bleed valve (42), the ramp (44/45), the TIT topping governor (46/47), the
`Wf/pt3` feedforward schedule (48), the `φ` floor and its release family (49/50/51/52). The
variable stator is the **first lever that moves the wall itself** — and that turns out not to be
a detail of one device but the thing that exposes what the project has been measuring for
seventeen rungs.

---

## THE HEADLINE — a general law, not a device study

> **A surge margin is a DISTANCE to a boundary. When a lever moves the boundary, that distance
> is COORDINATE-DEPENDENT: two reference-free margins that vanish on the same boundary can
> disagree on whether the lever helped. Only the coordinate in which the boundary is FIXED
> measures a margin at all.**

Proved, not observed. For any lever `x`, with `v` the stator setting and `φ_op` the operating
flow coefficient:

```
    M_φ = φ_op − φ_surge(v)                 M_i = T_c − tan β₁(φ_op, v)
    sign(dM_φ/dx) = sign(φ_op′ + v′·φ_surge²)      sign(dM_i/dx) = sign(φ_op′ + v′·φ_op²)
```

At `v′ = 0` these reduce to `sign(φ_op′)` and `sign(φ_op′/φ_op²)` — **identical**, because the
Jacobian `1/φ_op²` is strictly positive. So:

- **A floor-fixed lever can never split the two currencies.** Rungs 36–52 read surge as a
  `φ`-distance and were correct to; that is now a *derived* licence rather than an assumption.
  This is the cross-rung correction, and it is a **BOUNDING**, not a refutation.
- **A floor-moving lever splits them iff `−φ_op′/v′ ∈ (φ_surge², φ_op²)`** — an interval whose
  **width is the open margin itself.** The disagreement exists precisely *because* there is
  margin, and closes as the margin closes.

At the design point (`φ_op = 1`, the stator as the lever so `v′ = 1`) the split condition is
`(1+l)/(2+l) ∈ (φ_s0², 1)` — for the LP map `0.630 ∈ (0.3025, 1)`, so it splits, and

```
    dM_φ/dv = −(1+l)/(2+l) + φ_s0²  = −0.32713   (LP)     the φ-margin SHRINKS on closing
    dM_i/dv = +1/(2+l)              = +0.37037   (LP)     the incidence margin GROWS
```

both measured to five decimals. `T_c` is a property of the blade **metal**, so the incidence
boundary is stator-invariant and `M_i` is the margin; `M_φ` is a mixed measure the moment the
stator moves, conflating *"the point moved toward the wall"* with *"the wall moved."* And read
in the correct currency the model says what engineering practice says: **closing the stators
buys surge margin.**

---

## The instrument

The stator setting is expressed **in the swirl it induces**, `v ≡ tan α₁` at the rotor inlet
(`v > 0` closed / co-rotating pre-swirl, `v < 0` opened past axial). It is a swept geometry
coordinate — like `bleed`, `s_off`, `τ_rel` before it — **not a fitted constant**. Both channels
it drives are derived from constants the maps already carry, so **the rung adds none**.

### Channel 1 — the WORK (`ComponentMap.psi`)

Euler work with inlet swirl, rotor exit relative angle `β₂` fixed by the blade metal, `φ = Vx/U`:

```
    Δh = U²·[1 − φ·(tan β₂ + tan α₁)]
```

Normalising on the design work (`φ_d = 1`, `v = 0`) gives `ψ = [1 − φ(t₂+v)]/(1 − t₂)`. Matching
its design slope to **this map's own** rung-34 slope `dψ/dφ|₁ = −l` **derives** `t₂ = l/(1+l)`,
hence `1/(1−t₂) = 1+l`, and the stator is one extra term:

```
    ψ(φ, v) = 1 − σ(φ−1)² − l(φ−1)  −  v·(1+l)·φ
              └──── rung 34's law, untouched ────┘
```

The parabolic `σ` term is the map's **non-Euler loss curvature** and is deliberately left
stator-inert (see Concessions).

### Channel 2 — the FLOOR (`ComponentMap.phi_surge_at`)

A rotor stalls at a critical relative inlet angle. With pre-swirl `V_θ1 = φ·U·v`, the relative
tangential velocity is `U(1 − φv)` and the axial is `φU`, so

```
    tan β₁ = (1 − φ·v)/φ = 1/φ − v          stall iff  tan β₁ ≥ T_c
```

Rungs 36/41's **imposed** `φ_s0` is by definition the `φ` at which the design-set stators
(`v = 0`) reach stall, so it **anchors** `T_c = 1/φ_s0` (`ComponentMap.tan_beta1_crit`), and the
floor's *variation* is derived rather than imposed:

```
    φ_surge(v) = 1/(T_c + v) = φ_s0/(1 + v·φ_s0)
```

Closing the stators lowers the floor. **Both channels ride on the same `v`** — that is the
physics: *a variable stator cannot move the floor without moving the running line.*

**Division of duties (so rung 41's readers stay literally unchanged):** the FIELD `phi_surge`
remains the design-setting anchor that rungs 36/41/44/45 read; the METHOD `phi_surge_at()` is
the live floor that rung 53's diagnostics read. They coincide at `v = 0`.

---

## Where it lives, and the reduce

`VariableStatorMatcher(TwoSpoolMapMatcher)` in `turbojet/engine.py`, with per-spool `vsv_lp` /
`vsv_hp`. The stators sit at their **design setting at the design point by construction** (rung
42's valve-shut discipline): the hardware `A4/A45/A8` and both maps' design references are
captured from a `v = 0` design run, and only then are the stators moved. `at_setting()` produces
siblings on the *same* hardware so a swept setting can never be confused with a re-designed
engine.

**THE REDUCE IS AN IDENTITY, stronger than rung 42's dispatch.** At `vsv_lp == vsv_hp == 0` the
stored maps are the **same objects** that were passed in (`m.map_lp is LP`), and `match` is rung
39's own method, **inherited unoverridden** — there is no rung-53 code path to skip. This is
possible because of P1 below: the stator adds **no new closure and no new equation**. Contrast
rung 42, whose bleed needed an entire new cascade.

Rungs 38–52 are untouched: `psi` and `phi_max` return early at `vsv == 0`, and `phi_surge` the
field still means the anchor.

---

## P1 — the stator is a SPEED lever, and adds no closure

`τ_c` comes from rung 38's **map-free ENERGY cascade** (`Tt25/Tt2`, `Tt3/Tt25`), and the
face-referred corrected flow `m` from rung 39's *(dagger)*/*(ddagger)* carries no loading law. So
`v` enters the steady solve through **`solve_n` alone**: closing the stators unloads the
compressor, `n` RISES, `φ_op = m/n` FALLS, and `m` moves only second-hand through the efficiency
island. The general sensitivity is

```
    dφ_op/dv = −(1+l)·φ_op² / D(φ_op) ,   D(φ) = 2 + 2σ(φ−1) + l(2−φ)
    ⇒  at design (φ_op = 1):   dφ_op/dv = −(1+l)/(2+l)
```

**Measured: EXACT at design** (`−0.62963` LP, `−0.66667` HP; 0.00 % error), for a reason the
prediction missed — the efficiency island `η_c = base − a(φ−1)² − b(n−1)² − c(…)` is
**stationary at `(φ,n) = (1,1)`**, so `dη/dv = 0` to first order there and `m` cannot move at
all (`|Δm/m| / |Δn/n|` = 2.0e-8 LP, 1.5e-9 HP). Off design `m` does move, and both
pre-registered bands hold across the choked band: flow-vs-speed ratio ≤ **4.4e-2**, closed form
within **3.4 %** at `Tt4` = 1000.

### The trade: THRUST-NEUTRAL, paid in SHAFT SPEED

A consequence of the same algebra, and the cleanest contrast with rung 42. At fixed `Tt4` the
energy cascade pins `τ_c`, so `π_c` can move only through `η` — second order at the island peak.
Over `v ∈ [−0.1, +0.3]`: specific thrust varies **< 0.13 %** (and *peaks* at `v = 0`), TSFC
< 0.13 %, `π_LPC` 3.000 → 2.975 — while **`N_L` rises +19 %**. Rung 42's bleed costs thrust
monotonically; **the stator buys margin with speed, not with performance.** Shaft speed is a
mechanical/stress limit this project does not model, which is exactly why the schedule numbers
below are scope-bound while the currency result is not.

---

## P7 — the currency finding made operational: the constant-incidence schedule

The correct currency makes one object derivable that `φ`-space cannot even express: the schedule
`v*(Tt4)` holding the rotor incidence at its **design** value — which is what a real VSV
schedule is *for*. One 1-D root per throttle point (`incidence_schedule`), the target incidence
**read** off the matcher rather than assumed, zero new constants, still steady.

| `Tt4` | `v*` | `φ_op` | `M_i` | `M_φ` | `M_φ` bare | `SM_N` | `SM_N` bare |
|---|---|---|---|---|---|---|---|
| 1500 | 0.000 | 1.0000 | 0.8181818 | 0.45000 | 0.45000 | 0.2413 | 0.2413 |
| 1300 | 0.335 | 0.7490 | 0.8181818 | 0.28458 | 0.35590 | 0.3581 | 0.1446 |
| 1100 | 0.877 | 0.5328 | 0.8181818 | 0.16175 | 0.26529 | 0.4851 | 0.0804 |
| 1000 | 1.244 | 0.4457 | 0.8181818 | 0.11911 | 0.22312 | 0.5371 | 0.0580 |

**One trajectory, one boundary, three reference-free distances, three verdicts.** `M_i` is
constant to `1e-13` *by construction*; on the identical trajectory `M_φ` falls **74 %** — to
*below* its own unscheduled value at every point — while `SM_N` rises **2.2×** (9.3× against
bare). The `φ`-currency reports a large monotone margin LOSS along a schedule that changes the
true margin **not at all**.

**Two confidence levels in this section, stated explicitly:**

- **The currency result is coordinate algebra** — the `1/φ_op²` Jacobian, the interval law, the
  two closed forms. It rides on no magnitude and survives any overspeed.
- **The schedule's NUMBERS are model-bound.** Holding design incidence at `Tt4` = 1000 costs
  `N_L` **+26 %** in this model. A real multistage machine does not pay that, because it gets
  the effect from **stage rematching and stator-row flow capacity** — the channels this lumped
  single-stage-equivalent map does not have (see Concessions). Read `v*` as *"the swirl a
  one-stage cartoon needs"*, not as a VSV schedule.

**The schedule's SIZE inherits rung 41's split.** To hold design incidence at `Tt4` = 1000 the
LP needs `v*` = **1.244** and the HP only **0.185** — **6.7×** — because the LP is the spool that
takes the throttle excursion (rungs 41/44/45). *The stator authority a spool needs is a measure
of its exposure.*

---

## The third currency, and why it does NOT vote

Rung 41's constant-speed pressure-ratio margin `SM_N` also rises on closing (`dSM_N/dv` =
**+0.838** LP, **+2.093** HP, positive on every shape). **This is reported as measured and is
NOT evidence for `M_i`.** `SM_N`'s boundary is not fixed either — it is evaluated at the live
floor — so the law makes no prediction about its sign. The available mechanism has nothing to do
with incidence: `SM_N` is a π-gap between two points on ONE speed line, and closing the stator
drops the floor point much further in π than it drops the operating point, so the gap widens.

**No vote-counting.** *"Two of three currencies say it helps"* is exactly the reasoning this
rung's law forbids: `M_i` is privileged because its boundary is stator-invariant, not because it
has company.

---

## P5 — a rung-39 refinement: the stator's inter-spool arrow is η-MEDIATED ONLY

Rung 39 found the map opens exactly **one arrow, HP → LP**, because `π_LPC` cancels out of the
HP-face corrected flow and the energy cascade is map-free. The stator inherits that structure and
sharpens it:

- **`vsv_lp` cannot reach the HP spool at all.** Measured at `vsv_lp = 0.20`: `Δφ_HP` and
  `Δn_HP` are **exactly `+0.000e+00`** (while `Δφ_LP = −0.11894`, so the lever is live). The LP
  stator is a **pure-LP lever, bit-for-bit**.
- **`vsv_hp` reaches the LP only through `π_HPC`, and only because `η_HPC` moved.** On flat-η
  islands (`a = b = c = 0`) `Δφ_LP` is **exactly `+0.000e+00`**; on the shaped islands it is
  `−4.79e-03`.

So the stator is a **cleaner per-spool DoF than rung 42's bleed valve**: bleed reaches the HP
through the shared `Tt25` — an ENERGY channel, alive even on a flat map — whereas the stator's
only inter-spool channel is the efficiency island, and it is switchable off.

---

## The split's boundaries — two-sided, and closed-form in the map's own constants

From the interval law, the split dies at **both** ends, and both boundaries are closed forms with
no new constants:

```
    dM_i/dv > 0   (the stator helps in incidence)   ⟺  1 + (φ_op−1)(2σ − l) > 0
    dM_φ/dv < 0   (the currencies split)            ⟺  φ_s0² < (1+l)φ_op²/D(φ_op)
```

- **Floor tightness.** At `φ_op = 1` the split needs `φ_s0 < √[(1+l)/(2+l)]` = **0.7935** (LP).
  Measured: `dM_φ/dv` crosses zero between `φ_s0` = **0.79** (−0.0055) and **0.82** (+0.0428).
  The disclosed floors (0.55 in rungs 41/44/45, 0.65 in rung 36) sit well below, so the split is
  not an artifact of one floor choice.
- **Part power.** Predicted `φ_op ≈ 0.71`; measured crossing between `Tt4` = **825**
  (`φ_op` = 0.7078) and **800** (`φ_op` = 0.6996) — **inside** the choked envelope, so the flip
  is reachable. The prediction lands **0.3 % above** the bracket rather than inside it, which is
  the closed form's known off-design error (P1) and is scored as a miss on the *level*; the
  claim is the existence and the bracket. Below the crossing both currencies agree that closing
  the stator loses margin.
- **Upper end.** `2σ − l < 0` for every disclosed shape, so `dM_i/dv > 0` out to
  `φ_op = 1 + 1/(l−2σ)` = **3.0** (LP) — outside the physical band. The incidence benefit is
  robust exactly where the `φ` reading is not.

---

## What this rung does NOT do — the rung-46–52 timing law is INAPPLICABLE

The fuel-side family's law is *"a limiter rebates a spool IFF it engages upstream of THAT
spool's own surge minimum"* — a **timing** law over a moving point and a fixed wall. A steady
stator setting has no engagement time, and it moves the wall. The law does not transfer, and the
reason is **structural, not a sign flip**: it presupposes a fixed floor. What transfers is the
*currency discipline* of rungs 42/43/45 — and the headline makes that transfer a **correction**.

---

## Verification gates (`tests/test_rung53.py`)

1. **REDUCE — an IDENTITY.** `vsv_lp == vsv_hp == 0` ⇒ `m.map_lp is LP` and `m.map_hp is HP`
   (object identity, the strong claim), and every matched field `==` rung 39's on **both** the
   fast/CPG and the reacting-equilibrium gas. Plus `psi`/`phi_max`/`phi_surge_at` bit-for-bit at
   `vsv = 0`, and `ComponentMap.flat().with_phi_surge(x).is_flat()` still true while
   `.with_vsv(x)` is **not** flat.
2. **THE CONTROL that could have killed the rung (P3a).** At `v = 0`, across the throttle band
   on both spools, **all three** currencies agree in sign at every step and `dM_i/dM_φ` tracks
   `1/φ_op²` to ≤ 1e-3 relative. A floor-fixed lever cannot split them.
3. **THE HEADLINE (P3).** With the stator as the lever the signs DO split, on both spools and
   across all five disclosed map shapes; the measured derivatives match `−(1+l)/(2+l) + φ_s0²`
   and the closed form `+1/(2+l)`; and the interval test `−φ_op′/v′ ∈ (φ_surge², φ_op²)` holds.
4. **ZERO NEW CONSTANTS.** `tan_beta1_crit() == 1/phi_surge` exactly; `phi_surge_at()` at
   `v = 0` returns the field; the derived `t₂ = l/(1+l)` reproduces `psi`'s slope; and the
   incidence floor and the `ψ` law agree — `phi_surge_at()` is exactly where
   `tan_beta1() == tan_beta1_crit()`.
5. **P1 — SPEED not FLOW.** `n` up, `φ_op` down, `|Δm/m| ≤ 0.1·|Δn/n|` across the band (machine
   zero at design), and the closed form within 10 %. Plus the trade: specific thrust flat to
   < 0.5 % while `N_L` rises > 15 %.
6. **P5's TWO EXACT ZEROS, with `==`.** `vsv_lp` leaves `φ_HP`/`n_HP` bit-identical; `vsv_hp` on
   flat-η islands leaves `φ_LP` bit-identical; and the shaped-island arrow is nonzero (so the
   zeros are not vacuous).
7. **P7 — the schedule.** `M_i` constant to `_INC_TOL` at every point while `M_φ` falls **below
   its own bare value** at the same throttle — one assertion, both halves. Plus `v*` monotone
   rising as power falls, and `v*_LP > 3·v*_HP` (rung 41's split in the schedule's size).
8. **BOTH SPLIT BOUNDARIES AS BRACKETS** — the floor-tightness crossing between `φ_s0` = 0.79
   and 0.82 (the closed form 0.7935 falls inside), and the part-power crossing between
   `Tt4` = 825 and 800. Asserted as brackets, not as point values (the levels ride on the
   disclosed constants); the part-power prediction 0.71 is gated only to within 1 % of the
   bracket, because it misses it by 0.3 %.
9. **RUNG 41's TWO-PATH π GATE SURVIVES the new `ψ` term** — `_pi_c_spool` at the operating
   point reproduces the shipped `π` to ≤ 1e-11 at a MOVED stator, so the swirl term is
   consistent between the two code paths.
10. **CYCLE UNTOUCHED** — the default single-spool design path is bit-for-bit rung 6.

---

## Concessions

- **The SWIRL/incidence channel ONLY.** A real VSV row also changes the compressor's own flow
  CAPACITY (the stator throat) and rematches the stage stack against itself — the dominant
  effect in a real multistage machine, and the reason a real schedule does not need this model's
  overspeed. A lumped single-stage-equivalent map has neither, and the capacity channel needs a
  **new constant** (area per unit setting). **Refused, and named as this rung's seam.**
- **The σ term is stator-inert.** The parabolic loading curvature is an empirical loss term, not
  Euler work, so the stator is not given an arbitrary action on it. A real stator setting does
  change the loss bucket's shape; that too needs a new constant.
- **The incidence benefit SATURATES in `v`** and does not turn back: `dM_i/dv = 1 − (1+l)/D_v`
  with `D_v = D(φ) − v(1+l)φ` approaches zero only asymptotically, and `solve_n`'s speed-line
  bracket (a map-validity edge) is reached first. So a stator has **finite authority** and cannot
  restore design incidence arbitrarily far off design. (The apparent turning point that this
  algebra suggests is *not* reached — see the anchor.)
- **`φ_s0` and the map shapes inherit rungs 36/41's disclaimers.** `φ_s0` is imposed and now
  does double duty as the incidence anchor, so `T_c` is imposed with it. **Magnitudes
  disclaimed; the signs, orderings, machine-zeros, brackets and closed forms are the claims.**
- **Steady only.** A *scheduled* `v(n)` on the rung-40/43 transient plant is a different rung.
- The plant is rung 39's gas — rung 35's standing concession.
- **`phi_max` is generalised but NOT exercised by this rung**: the two-spool steady cascade never
  calls it (only the rung-34/40/43 forward transient closures do).

## The next seam

**The stator-row FLOW CAPACITY channel** — the half this rung refuses, and the one that carries
the real multistage benefit. It needs an anchored area-per-setting law, and with it the model
would no longer have to buy incidence with overspeed. Beyond that: a **stator schedule on the
transient plant** (`v(n)` against rungs 44/45's excursions — the first lever that could move the
wall *during* an accel), and **stator + bleed together** (the two halves of rung 42's seam, now
both built, and P7's saturation says the bleed is what takes over where the stator's authority
runs out).

## Anchor

`docs/plans/rung53-anchor-variable-stator.md` — the predictions as written before measuring (P1
scored *understated*, P7 *stronger than predicted*, one mid-build hypothesis raised and
**refuted**), the probe transcripts and the verified numbers.
