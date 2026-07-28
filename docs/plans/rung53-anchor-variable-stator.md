# Rung 53 anchor — the VARIABLE STATOR: predictions, and how they scored

Rung 42 shipped the interstage bleed valve and named this rung in its own header block:

> *"And bleed moves the operating point `phi_op`; it does NOT move the stall floor
> `phi_surge` — that is the variable-stator half of the seam, still open."*

Every surge lever in the project so far — the throttle (36/41), the bleed valve (42), the
ramp (44/45), the topping governor (46/47), the `Wf/pt3` schedule (48), the `φ` floor
(49/50/51/52) — moves the OPERATING POINT against a FIXED floor. The variable stator is the
first lever that **moves the floor itself**. That is why it is a rung and not a seventh
instrument on the fuel side.

## The instrument (fixed before any measurement)

The stator setting is expressed **in the swirl it induces**, `v ≡ tan α₁` at the rotor inlet
(`v > 0` = closed, co-rotating pre-swirl; `v < 0` = opened past axial, counter-swirl). It is a
swept coordinate — a geometry DoF being dialled, like `bleed` / `s_off` / `τ_rel` before it —
**not a fitted constant**. Both channels below are then derived from the map's OWN rung-34
loading slope `l` and rungs 36/41's OWN imposed floor `φ_s0`: **zero new constants.**

**Channel 1 — the WORK (the running line).** Euler with inlet swirl, rotor exit relative
angle `β₂` fixed by the blade metal, `φ = Vx/U`:

```
    Δh = U²[1 − φ(tan β₂ + tan α₁)]
```

Normalised by the design work (`φ_d = 1`, `v = 0`) this is
`ψ(φ,v) = [1 − φ(t₂+v)]/(1 − t₂)`, and matching its design slope to the map's own
`dψ/dφ|₁ = −l` **derives** `t₂ = l/(1+l)`, hence `1/(1−t₂) = 1+l`:

```
    ψ(φ, v) = 1 − σ(φ−1)² − l(φ−1)  −  v(1+l)·φ            [σ term = the map's non-Euler
                └──── rung 34's law, untouched ────┘          loss curvature, stator-inert]
```

**Channel 2 — the FLOOR.** The rotor stalls at a critical relative inlet angle,
`tan β₁ = (1 − φv)/φ ≥ T_c`. Rungs 36/41's imposed `φ_s0` **anchors** `T_c = 1/φ_s0`, so the
floor's *variation* is derived, not imposed:

```
    φ_surge(v) = 1/(T_c + v) = φ_s0 / (1 + v·φ_s0)
```

Both channels ride on the SAME `v`. That is the physics: **a variable stator cannot move the
floor without moving the running line.**

## Where it lives

`VariableStatorMatcher(TwoSpoolMapMatcher)` with per-spool `vsv_lp`, `vsv_hp` — rung 42's
shape exactly (the stators sit at their DESIGN setting at the design point by construction;
the hardware and both maps' design references are captured from a `v=0` design run).
`vsv_lp == vsv_hp == 0` ⇒ bit-for-bit rung 39. **In the event this came out stronger than the
planned dispatch:** because the stator turned out to add no closure (P1), `match` is *not
overridden at all* and the stored maps are the **same objects** passed in, so the reduce is an
**IDENTITY of code path** rather than a dispatch around one.

## Config

Rung 42's: CPG + reacting gas, `FLIGHT(250 K, 50 kPa, M0=0.85)`, `π_LPC/π_HPC/Tt4 = 3/6/1500`,
LP map `(a=0.20, b=0.05, σ=0.1, l=0.7)`, HP map `(a=0.08, b=0.15, σ=0.1, l=1.0)`,
floors `φ_s0 = 0.55` both spools (rung 41/44/45's value).

---

# THE PREDICTIONS, as written before measuring — and how they scored

## P1 — the stator is a SPEED lever, not a flow lever

`τ_c = Tt3/Tt25` (HP) and `Tt25/Tt2` (LP) come from the **map-free ENERGY cascade**, and the
face-referred corrected flow `m` (rung 39's *(dagger)*/*(ddagger)*) contains no loading law.
So `v` enters the steady solve through `solve_n` ALONE, at first order: closing the stator
unloads the compressor, so **`n` RISES and `φ_op = m/n` FALLS**, with `m` moving only
second-hand through the efficiency island.

> **Quantitative:** at the design point (`φ_op = 1`), `dφ_op/dv = −(1+l)/(2+l)`
> ⇒ **−0.630** (LP, `l=0.7`), **−0.667** (HP, `l=1.0`). And `|Δm/m| ≪ |Δn/n|`.

**Tolerance, pre-registered before measuring** (the closed form assumes `m` EXACTLY fixed;
`m` in fact moves through `η_c(φ,n) → π → m`, and the shaped maps carry `a` = 0.20/0.08, so a
few-percent departure is EXPECTED and is not a falsification):

| What | Counts as CONFIRMED | Counts as FALSIFIED |
|---|---|---|
| **Structure** (the load-bearing part) | `n` rises, `φ_op` falls, and `\|Δm/m\| ≤ 0.1·\|Δn/n\|` | either sign wrong, or `\|Δm/m\|` comparable to `\|Δn/n\|` |
| **Closed form** (corroboration) | measured `dφ_op/dv` within **10 %** of `−(1+l)/(2+l)` | outside 25 % — reported as measured either way |

**SCORE: CONFIRMED, and UNDERSTATED.** The closed form is not an approximation at the design
point, it is EXACT there (0.00 % on both spools) — and for a derived reason the prediction
missed: the efficiency island `eta_c = base - a(phi-1)^2 - b(n-1)^2 - c(..)` is STATIONARY at
`(phi,n) = (1,1)`, so `d(eta)/dv = 0` to first order at design and `m` cannot move at all
(`|dm/m| / |dn/n|` = **2.0e-8** LP, **1.5e-9** HP — machine noise). Off design `m` does move, and
both pre-registered bands hold across the whole choked band: `|dm/m| / |dn/n|` <= **4.4e-2**
(band: <= 0.1) and the closed form to <= **3.4 %** at Tt4 = 1000 (band: 10 %).

## P2 — the floor law is derived and anchored

`φ_surge(0) == φ_s0` exactly; `dφ_surge/dv|₀ = −φ_s0² = −0.3025`. `T_c` is READ OFF rungs
36/41's imposed floor, so the rung adds no constant to the two it inherits.

**SCORE: CONFIRMED** by construction and by test (`phi_surge_at()` at `v = 0` returns the field
itself; `tan_beta1_crit() == 1/phi_surge`).

## P3 — THE HEADLINE: the two reference-free currencies DISAGREE IN SIGN

`φ`-margin `M_φ = φ_op − φ_surge(v)` and incidence margin `M_i = T_c − tan β₁(φ_op, v)` are
**both reference-free** and **vanish on the same boundary** (`M_φ > 0 ⟺ M_i > 0` identically,
since `tan β₁ = 1/φ − v` is monotone in `φ`). Yet:

```
    dM_φ/dv|₀ = −(1+l)/(2+l) + φ_s0²   < 0      the φ-margin SHRINKS on closing
    dM_i/dv|₀ = +1/(2+l)               > 0      the incidence margin GROWS
```

> **Quantitative:** `dM_φ/dv` = **−0.328** (LP), **−0.364** (HP);
> `dM_i/dv` = **+0.370** (LP), **+0.333** (HP) — and the latter is the closed form
> **`1/(2+l)`**, zero new constants (rung 41's `π*` register).

The resolution, and the rung: **margin is a DISTANCE to a boundary, and distance is not
invariant under a lever-dependent coordinate change unless the boundary is FIXED.** In
incidence coordinates the boundary is a constant of the metal (`T_c`); in `φ` coordinates the
lever moves the wall, so a `φ`-distance conflates *"the point moved toward the wall"* with
*"the wall moved."* For a floor-moving lever the incidence currency is the correct one — and
it says what engineering practice says: **closing the stators buys margin.**

### P3a — the sign split PROVABLY requires a moving floor (the advisor's blocking check)

The objection: `M_i` is a monotone *reparameterisation* of `M_φ`, and `φ ↦ 1/φ` is nonlinear,
so the two could disagree under the THROTTLE alone at `v = 0` — in which case the moving
floor is not the mechanism and P3's resolution is wrong. **It cannot.** For any lever `x`:

```
    dM_φ/dx = φ_op′ + v′·φ_surge(v)²           dM_i/dx = φ_op′/φ_op² + v′
    ⇒  sign(dM_φ/dx) = sign(φ_op′ + v′·φ_s²)   sign(dM_i/dx) = sign(φ_op′ + v′·φ_op²)
```

At `v′ = 0` the two reduce to `sign(φ_op′)` and `sign(φ_op′/φ_op²)` — **identical**, because
the Jacobian `1/φ_op²` is strictly positive. Magnitudes differ by exactly that factor; signs
cannot. So **a floor-fixed lever can never split the currencies, and every rung 36–52 was
safe for that reason.** In general the two disagree

```
    IFF   −φ_op′/v′  ∈  (φ_surge², φ_op²)          — an interval whose WIDTH is the OPEN MARGIN
```

so the disagreement window exists precisely *because* there is margin, and closes as the
margin closes. At the design point (`φ_op = 1`, `v′ = 1`) this is
`(1+l)/(2+l) ∈ (φ_s0², 1)`: **0.630 ∈ (0.3025, 1)** ⇒ split (LP).

> **Gate:** at `v = 0`, sweep `Tt4` across the choked band and assert
> `sign(ΔM_φ) == sign(ΔM_i)` at every step on both spools, while the magnitude ratio tracks
> `1/φ_op²`. This gate could FAIL and would kill P3's stated mechanism.

**SCORE (P3 and P3a): BOTH CONFIRMED — P3 to five decimals, P3a as a PROOF.**

P3: LP `dM_phi/dv` = **-0.32713** (predicted -0.32713), `dM_i/dv` = **+0.37037** = 1/2.7
exactly; HP **-0.36417** / **+0.33333** = 1/3 exactly. `split = True` on both spools and on all
FIVE map shapes tried (flow/press, press/flow, tilted, steep, flat-eta).

**A THIRD currency, unregistered, sides with incidence:** rung 41's pressure-ratio margin
`SM_N` gives `dSM_N/dv` = **+0.838** (LP) / **+2.093** (HP), positive on every shape. So of three
reference-free distances to the same boundary, the ODD ONE OUT is exactly the one whose
coordinate carries the moving wall.

**But `SM_N`'s agreement is NOT explained by the law, and is reported as measured.** `SM_N`'s
boundary is not fixed either — it is evaluated at the LIVE floor — so the law makes no
prediction about its sign, and the fact that it agrees with `M_i` is not evidence for `M_i`.
The available mechanism is unrelated to incidence: `SM_N` is a π-gap between two points on ONE
speed line, and closing the stator drops the floor point much further in π than it drops the
operating point, so the gap widens. **No vote-counting.** "Two of three currencies say it
helps" is exactly the reasoning this rung's law forbids: `M_i` is privileged because its
boundary is stator-invariant, not because it has company.

P3a: the algebra needs no run — the run is the CONTROL. At `v = 0`, across Tt4 = 1500 -> 1000
on both spools, all **three** currencies agree in sign at every step, and the ratio
`dM_i/dM_phi` tracks `1/phi_op^2` to <= **7e-4** relative. The gate could have failed and did
not; the sign split is therefore established as requiring a MOVING floor, not merely observed
alongside one.

## P4 — the split has a TWO-SIDED derived boundary in the map's own constants

From P3a's interval, the split dies at **both** ends — and both are closed forms with no new
constants. Writing `D(φ) ≡ 2ψ₀(φ) − φψ₀′(φ) = 2 + 2σ(φ−1) + l(2−φ)`, the general
operating-point sensitivity is `dφ_op/dv = −(1+l)φ_op²/D(φ_op)` (P1's design value is the
`φ_op=1` case), so:

```
  upper end (dM_i/dv > 0, the stator helps in incidence)  ⟺  1 + (φ_op−1)(2σ − l) > 0
  lower end (dM_φ/dv < 0, the currencies split)           ⟺  φ_s0² < (1+l)φ_op²/D(φ_op)
```

- **Floor-tightness boundary** (at `φ_op = 1`): the split needs `φ_s0 < √[(1+l)/(2+l)]` ⇒
  **0.794** (LP), **0.816** (HP). The disclosed floors (0.55 in rungs 41/44/45, 0.65 in rung
  36) sit well below, so the split is not an artifact of one floor choice.
- **Part-power boundary** (at `φ_s0 = 0.55`, LP): the split needs
  `1.7 φ_op²/(2 + 0.2(φ_op−1) + 0.7(2−φ_op)) > 0.3025` ⇒ **`φ_op ≳ 0.71`**. Predicted:
  around `φ_op ≈ 0.71` the `φ`-currency FLIPS to agreement, and below it BOTH currencies say
  closing the stator loses margin — *the stator stops helping at deep part power even in the
  correct currency.*
- **Upper end**: `2σ − l < 0` for every disclosed shape, so `dM_i/dv > 0` holds for
  `φ_op < 1 + 1/(l − 2σ)` = **3.0** (LP) — outside the whole physical band. The incidence
  benefit is therefore robust where the `φ` reading is not.

**SCORE: CONFIRMED, both ends, both bracketed by the closed forms.**
- Floor-tightness: `dM_phi/dv` crosses zero between `phi_s0` = **0.79** (-0.0055) and **0.82**
  (+0.0428) — predicted **0.7935**.
- Part power: crosses between Tt4 = **825** (`phi_op` = 0.7078, -0.0071) and **800**
  (`phi_op` = 0.6996, +0.0005). Predicted **`phi_op` ~ 0.71**, which lands **just ABOVE** the
  measured bracket — a **0.3 % miss**, scored honestly and fully consistent with the closed
  form's known few-percent error off design (P1). The load-bearing claim is the EXISTENCE and
  the bracket, not the level 0.71. It is REACHABLE: the crossing sits INSIDE the choked
  envelope, and below it both currencies agree that closing the stator loses margin.
- Upper end: `dM_i/dv` > 0 at every point measured, as predicted (the closed-form boundary
  `phi_op` = 3.0 is outside the physical band).

## P5 — the stator's inter-spool arrow is η-MEDIATED ONLY (strictly weaker than bleed's)

Rung 39: `π_LPC` **cancels** out of the HP face, and rung 38's ENERGY cascade (`Tt25`, `Tt3`)
is map-free. So:
- **`vsv_lp` cannot reach the HP spool at all** — it moves `n_L`, `φ_L`, `η_LPC`, `π_LPC`, and
  every one of those is invisible to *(dagger)*.
- **`vsv_hp` reaches the LP spool only through `π_HPC`**, and `π_HPC` moves only because
  `η_HPC = η_c(φ_H, n_H)` moved. On a loading-only map (`a = b = c = 0`) that arrow is
  **EXACTLY zero** and the stator is *per-spool to machine precision*.

Contrast rung 42: bleed reaches the HP through the shared `Tt25` — an ENERGY channel, present
even on a flat map. **Prediction: the stator is a cleaner per-spool DoF than the bleed valve
is**, and the LP stator is a pure-LP lever with no HP consequence whatever.

**SCORE: CONFIRMED, with EXACT zeros (`==`, not a tolerance).**
- `vsv_lp = 0.20`: `d phi_HP` = **+0.000e+00** and `d n_HP` = **+0.000e+00** — the LP stator is a
  pure-LP lever, bit-for-bit. (`d phi_LP` = -0.11894, so the lever is live.)
- `vsv_hp = 0.20` on FLAT-eta islands (`a = b = c = 0`): `d phi_LP` = **+0.000e+00 EXACTLY**.
- `vsv_hp = 0.20` on the shaped islands: `d phi_LP` = **-4.79e-03** — the eta arrow, and only
  the eta arrow.

So the stator IS the cleaner per-spool DoF: rung 42's bleed reaches the HP through the shared
`Tt25` (an ENERGY channel, alive even on a flat map); the stator's only inter-spool channel is
the efficiency island, and it is switchable off.

## P6 — the rung-46–52 timing law is INAPPLICABLE, not inverted

The fuel-side family's law is *"a limiter rebates a spool IFF it engages upstream of THAT
spool's own surge minimum"* — a **timing** law over a moving point and a fixed wall. A steady
stator setting has no engagement time at all, and it moves the wall. **Prediction: the law
does not transfer, and the reason is structural (it presupposes a fixed floor), not a sign
flip.** What DOES transfer is the currency discipline of rungs 42/43/45 — and P3 says it
transfers as a *correction*: the `φ`-distance every one of rungs 36–52 scored surge in was
safe ONLY because their levers left the floor fixed.

**SCORE: CONFIRMED structurally** (an argument, not a measurement — there is no engagement time
to measure). What transfers is the currency discipline, and P3/P3a make it a CORRECTION: the
`phi`-distance every one of rungs 36-52 scored surge in was safe ONLY because their levers left
the floor fixed.

## P7 — THE PAYOFF: the constant-incidence schedule (registered before measuring)

The rung's own currency makes one object derivable that `φ`-space cannot even express: the
stator schedule `v*(Tt4)` that holds the rotor incidence AT ITS DESIGN VALUE
(`tan β₁ = 1/φ_op − v = 1` exactly, since `φ_op = 1` and `v = 0` at design) — which is what a
real VSV schedule is *for*. It is a 1-D root per throttle point, zero new constants, and still
steady.

> **Prediction:** along `v*`, `M_i` is **EXACTLY constant** (by construction — the gate is that
> the solve achieves it to solver tolerance), while `M_φ` **FALLS substantially** below its
> `v = 0` value at the same throttle. So the `φ`-currency reports a large, monotone margin
> LOSS along a schedule that by construction changes the true margin **not at all**. The
> schedule itself should be `v* = 1/φ_op(v*) − 1`, i.e. **increasingly closed as power falls**
> — the direction real engines schedule. `SM_N` reported as measured (it sided with incidence
> in P3's sweep, so it should not collapse).

**SCORE: CONFIRMED, and STRONGER than predicted.** Along the LP constant-incidence schedule
(`v*` = 0 -> 1.244 as Tt4 falls 1500 -> 1000):

| Tt4 | `v*` | `phi_op` | `M_i` | `M_phi` | `M_phi` bare | `SM_N` | `SM_N` bare |
|---|---|---|---|---|---|---|---|
| 1500 | 0.000 | 1.0000 | 0.8181818 | 0.45000 | 0.45000 | 0.2413 | 0.2413 |
| 1300 | 0.335 | 0.7490 | 0.8181818 | 0.28458 | 0.35590 | 0.3581 | 0.1446 |
| 1100 | 0.877 | 0.5328 | 0.8181818 | 0.16175 | 0.26529 | 0.4851 | 0.0804 |
| 1000 | 1.244 | 0.4457 | 0.8181818 | 0.11911 | 0.22312 | 0.5371 | 0.0580 |

`M_i` is constant to **1e-13** (solver tolerance) BY CONSTRUCTION, and on the identical
trajectory `M_phi` falls **74 %** — to *below* its own unscheduled value at every point — while
`SM_N` rises **2.2x**, or **9.3x** against bare. One trajectory, one boundary, three
reference-free distances, three different verdicts.

**Two findings on this table that were NOT predicted:**
1. **The stator is THRUST-NEUTRAL; its currency of cost is SHAFT SPEED.** At fixed Tt4 the
   map-free energy cascade pins `tau_c`, so `pi_c` can move only through `eta` — second order at
   the island peak. Measured over `v` in [-0.1, +0.3]: specific thrust varies **< 0.13 %** (and
   *peaks* at `v = 0`), TSFC < 0.13 %, `pi_LPC` 3.000 -> 2.975 — while `N_L` rises **+19 %**, and
   **+26 %** along the schedule at Tt4 = 1000. Contrast rung 42, whose bleed costs thrust
   monotonically. The stator buys margin with speed, not with performance.
2. **The schedule's SIZE inherits rung 41's split.** To hold design incidence at Tt4 = 1000 the
   LP needs `v*` = **1.244** and the HP only **0.185** — a **6.7x** ratio, because the LP is the
   spool that takes the throttle excursion (rungs 41/44/45). The stator authority a spool needs
   is a measure of its exposure.

## A mid-build hypothesis, RAISED AND REFUTED (rung 40's rule: not quietly dropped)

The `v_hi` bracket failure at Tt4 = 1100 looked like a **stator-authority TURNING POINT**: the
general sensitivity is `dphi_op/dv = -(1+l)phi^2 / D_v` with
`D_v = 2 + 2 sigma(phi-1) + l(2-phi) - v(1+l)phi`, so `dM_i/dv = 1 - (1+l)/D_v` changes sign
where `D_v = 1+l` — closing the stators past an optimum should GIVE BACK incidence margin. **It
does not happen.** A far sweep (`v` to 2.1 at Tt4 = 1100) has `M_i` **monotone rising and
saturating** (0.592 -> 0.928) with `D_v` approaching 1.70 only ASYMPTOTICALLY (2.79 -> 1.84); the
speed-line bracket in `solve_n` — a map-validity edge, not a physical one — is reached first. The
bracket failure was a bug in the ladder (it never evaluated `v_hi` itself), now fixed. What is
true and kept: **the incidence benefit saturates in `v`,** so a stator has finite authority and
cannot restore design incidence arbitrarily far off design.

## Honest scope, declared up front

- **Swirl / incidence channel ONLY.** A real VSV row also changes the compressor's own flow
  CAPACITY (the stator throat) and rematches the stages against each other — the dominant
  effect in a real multistage machine. A lumped single-stage-equivalent map has no stator
  throat and no stage stack, and the capacity channel needs a new constant (area per unit
  setting). **Refused, and named as this rung's seam.**
- `φ_s0` and the map shapes inherit rungs 36/41's disclaimers: magnitudes disclaimed, the
  **signs, orderings, machine-zeros and closed forms** are the claims.
- Steady only. The stator on the transient plant (a *scheduled* `v(n)`) is a different rung.
- The plant is rung 39's gas — rung 35's standing concession.
