# Rung 60 — the MATCHED φ FLOOR: a floor PINS the coordinate it watches

Rung 58 put rung 53's floor-moving stator beside a fuel-side min-select leg and found, as its
third finding, that rung 49's `φ`-referenced limiter **cannot be composed with it at all**:
the admissible set-point bands on the bare and statored machines are **DISJOINT**, so no
single floor is the same instrument on both. Rung 59 then repaired the *other* leg — rung 48's
`Wf/pt3` schedule — by matching it to the machine it runs on, and closed with the obvious
next move:

> *"The obvious repair is the one this rung just validated for `Wf/pt3`: match the set point
> to the machine. … rung 59's result says what to expect: matching should annihilate most of
> rung 58's disjointness the way it annihilated most of the HP interaction."*

**Matching does annihilate the disjointness — and it buys nothing, because the disjointness
was never the disease.** This rung builds the matched floor and finds out why.

---

## THE HEADLINE

**A limiter that FLOORS a variable PINS it. So a floor leg's composite second difference is a
difference of SET POINTS, not of dynamics, and takes a value fixed by the offset between the
leg's coordinate and the currency — exactly `v` for a `φ` floor, exactly `0` for an incidence
floor. Re-referencing the leg MOVES the tautology; it does not remove it. A leg that SETS a
minimum cannot compose with a wall-moving lever, in ANY coordinate.**

Three parts, and the order matters because each kills the repair the previous one proposed.

### 1. "Match the set point" is not a well-posed instruction

Rung 59's matching was canonical because a *schedule* has a definition to re-run: derive
`κ_ss` on the armed machine. **A set point has no definition to re-run.** Two natural rules,
both defensible:

    fixed φ-MARGIN off the moved wall    φ = (1+sm) / (T_c + v)
    fixed INCIDENCE                      φ = 1 / (T_c + v − M_B)

and in the incidence coordinate they are apart by

    1/φ_inc  −  1/φ_rel   =   v · sm / (1 + sm)

**exactly** — zero new constants, and zero exactly when either the lever or the margin is
(there is then nothing to disagree about). Verified at `sm` = 0.02 / 0.05 / 0.10 with residuals
**−2.4e−16 / −7.0e−16 / −1.1e−16**. At `v = 0.20, sm = 0.05` the two rules are `9.52e−03`
apart in incidence — **14 % of the ramp's whole excursion**, so this is not a rounding choice.

Nothing in the problem picks between them. Only rung 58's own currency finding does.

### 2. The canonical repair is not a calibration — it is a CHANGE OF COORDINATE

Rung 58 chose `M_i = T_c − (1/φ − v)` as its currency on the grounds that its wall is the
**metal**: `T_c = 1/φ_surge` off the DESIGN map, one number, bit-identical in all four cells.
Rung 53's own docstring had already named it *"the coordinate in which a stator-moved surge
boundary stands still."* So there is exactly one coordinate in which a set point is a single
number valid on every machine, and the matched `φ` floor **is** the incidence floor:

    M_i  ≥  m_lim        realised at the live setting as      φ_lim(v) = 1/(T_c + v − m_lim)

`IncidenceLimiter` is that leg. It needs no new solve — it hands back a plain rung-49
`SurgeLimiter` at the live `v` — and the conversion is legal rather than circular because
**`v` is a function of the shaft state and not of the fuel** (`_arm` takes `(nu_L, nu_H, Tt2)`),
so within a derivative call the floor is a constant and rung 49's monotonicity bracket
("cutting fuel raises `φ`") carries verbatim.

**And it works, as admissibility.** At `v = 0.20`, `r = 0.5`, LP:

| coordinate | bare band | armed band | gap |
|---|---|---|---|
| `φ` | [0.735442, 0.773116] | [0.670882, 0.702329] | **+0.033113 = 105.3 % of a band** |
| `M_i` | [0.458455, 0.524715] | [0.527606, 0.594347] | **+0.002891 = 4.4 % of a band** |

a **24×** shrink. On rung 58's own machine — the `v_max = 0.20` **schedule** — it closes
entirely: gap **−0.013533**, the bands OVERLAP, and a fixed incidence set point is one
instrument on both. (Its credit measures `0.052727`, reproducing rung 58's `credit_bare` to
every digit — the cross-check that both rungs read one object.)

### 3. …and it buys nothing, because a floor PINS

A floor that binds holds its own coordinate **at** the set point. So on every leg-armed cell
the minimum **is** the set point, and

    leg floors φ     M_i(both) − M_i(fuel)  =  [T_c − 1/φ_lim + v] − [… + 0]  =  v
    leg floors M_i   M_i(both) − M_i(fuel)  =  m_lim − m_lim                  =  0

Both derived before measurement, both met at machine precision:

| leg | setting | regime | `M_i(both) − M_i(fuel)` | derived | residual |
|---|---|---|---|---|---|
| incidence `M`=0.500 | `v`=0.05 | both pinned | −1.6e−15 | **0** | −1.6e−15 |
| incidence `M`=0.509 | `v`=0.10 | both pinned | −2.7e−15 | **0** | −2.7e−15 |
| incidence `M`=0.518 | `v`=0.15 | both pinned | −2.2e−16 | **0** | −2.2e−16 |
| `φ` floor 0.750 | `v`=0.15 | both pinned | +0.150000000000000 | **`v`** | +1.4e−16 |
| `φ` floor 0.750 | `v`=0.20 | both pinned | +0.200000000000002 | **`v`** | +1.7e−15 |
| incidence `M`=0.490 | `v`=0.15 | armed clears | +0.021210121922141 | `M_i(stator)−m_set` | +2.7e−15 |

**A number reproduced to 1e−15 by an identity is not evidence about the machine**, and the
gate asserts exactly that — the opposite of the usual gate, which is the point.

The two φ rows reproduce rung 58's own by-product (*"a pinned floor annihilates rung 57's
erosion, EXACTLY"*) at a setting rung 58 never ran, so **both ends of the tautology are
measured on one plant**: a `φ` floor reports the full POINTWISE credit with rung 57's erosion
annihilated; an incidence floor reports NO credit at all. Neither is a measurement.

**And the third regime is no escape.** Put the floor below the armed machine's own minimum and
`both` goes dormant — bit-identical to `stator`, the leg removing exactly `0.0` fuel — so the
difference is `M_i(stator) − m_set`: the floor and ONE leg-free march, with no armed-cell
dynamics in it either. `_pin_audit` names all three degeneracies (`pinned` / `dormant` /
`from_zero`) on every cell and they are asserted, never assumed — rung 59's `_clamp_audit`, one
ladder on.

**The general statement, and its asymmetry.** The composite reports `∂M_i/∂v` at fixed
leg-coordinate. A leg that sets the minimum's VALUE fixes that derivative by construction; only
a leg that moves the minimum's LOCATION leaves it to the plant. Rung 48's `Wf/pt3` schedule
composes for exactly that reason — it relocates (rung 48/50's truncated-descent law) and never
pins.

**Only one direction is proved here.** *Setting a minimum ⇒ cannot compose* is measured across
the whole floor family: three binding regimes, two coordinates, at machine precision. The
converse — *relocating ⇒ composes* — rests on **rung 48's schedule alone**, the single
positive case rungs 58/59 supply. The headline is stated as the direction that was proved, and
the biconditional is what the next seam exists to test.

---

## What DOES survive — the admissibility criterion, and it has a clock

The one non-tautological object the matched floor delivers is **whether a common set point
exists at all**. Both bands share the bare minimum as their origin, so

    gap  =  M_min(armed) − M_0(bare)  =  CREDIT − EXCURSION

is an **algebraic identity** (measured residual exactly `0.0`, and stated as an identity
rather than a result). A fixed incidence set point is admissible **iff the lever's credit is
smaller than the ramp's own excursion.** That is a criterion a designer can apply, and its two
inputs answer to different things.

**The stator ladder** (`r = 0.5`) — and rung 58's own two legs straddle the threshold at the
SAME setting:

| leg | credit | excursion | criterion | gap / band | `M_i` | `φ` |
|---|---|---|---|---|---|---|
| const `v`=0.05 | 0.018195 | 0.066261 | −0.048065 | −72.5 % | OK | OK |
| const `v`=0.10 | 0.035777 | 0.066261 | −0.030484 | −46.0 % | OK | NO |
| const `v`=0.15 | 0.052758 | 0.066261 | −0.013502 | −20.4 % | OK | NO |
| const `v`=0.19 | 0.065920 | 0.066261 | −0.000341 | −0.5 % | OK | NO |
| **const `v`=0.20** | 0.069152 | 0.066261 | **+0.002891** | +4.4 % | **NO** | NO |
| **sched `v_max`=0.20** | 0.052727 | 0.066261 | **−0.013533** | −20.4 % | **OK** | NO |
| sched `v_max`=0.30 | 0.066264 | 0.066261 | **+0.000004** | +0.0 % | NO | NO |

**Re-referencing MOVES the threshold, it does not abolish it.** The `φ` column has a threshold
too — it is already inadmissible at `v = 0.10`, against `v = 0.190` for incidence, because
`φ`'s effective displacement is the WALL SHIFT as well as the credit. So re-referencing roughly
doubles the setting a common set point survives to. What is gated is the implication —
incidence is admissible wherever `φ` is, never less — and not "`φ` always fails", which is
false at the smallest setting.

**This INVERTS rung 58's ranking.** There the constant setting was the benign partner — its
interaction was an order of magnitude smaller than the schedule's (+0.80 % against +9.51 %).
Here it is the constant setting that cannot be composed and the schedule that can, at the same
`v = 0.20`. The two properties are not the same property: *interaction* answers to the
schedule's state-feed, *composability* answers to the credit's magnitude, and they run
opposite ways. The `v_max = 0.30` row lands on the criterion at `+4e−06`, **0.006 % of a
band** — the sharpest available statement that the threshold is a real boundary and not a
fitted one.

**The ramp-rate ladder** (const `v = 0.20`) is the mechanism:

| `r` | credit | excursion | criterion | gap / band | `M_i` |
|---|---|---|---|---|---|
| 0.15 | 0.069771 | 0.159101 | −0.089330 | −56.2 % | OK |
| 0.25 | 0.069128 | 0.111487 | −0.042358 | −38.0 % | OK |
| 0.35 | 0.069101 | 0.087027 | −0.017926 | −20.6 % | OK |
| 0.50 | 0.069152 | 0.066261 | +0.002891 | +4.4 % | NO |
| 0.75 | 0.069234 | 0.047960 | +0.021274 | +44.4 % | NO |
| 1.00 | 0.069299 | 0.037805 | +0.031494 | +83.3 % | NO |

    credit spread  0.97 %          excursion spread  4.21x  (321 %)      ratio ~330x

**COMPOSABILITY HAS A CLOCK, AND THE CLOCK IS ENTIRELY THE RAMP'S.** The credit is rung 57's
number — *a wall-moving lever has no clock* — and it stands still to under 1 % across a 6.7×
range of ramp rate, while the excursion collapses 4.2×. So the same limiter and the same
stator compose on a slow accel and stop composing on a fast one, with the lever's setting never
touched. Rung 57's law is doing the work, on an axis rung 57 never looked at.

**`ds` refinement is reported on the criterion's INPUTS**, which is where it tests something:
credit `0.069139 / 0.069152 / 0.069146` and excursion `0.066248 / 0.066261 / 0.066262` at
`ds` = 0.01 / 0.005 / 0.0025 — a 0.02 % spread. The criterion itself is far less grid-sensitive
than either input (`+0.002890935` at both coarser grids, `+0.002884267` refined, −0.23 %),
because `credit − excursion` collapses to `M_min(armed) − M_0(bare)` and `s = 0` is a grid
point on every grid.

## The TIMING half — not a margin, so nothing floors it

`s_eng` is a time. It has no wall and is pinned by nothing, and it is where a floor leg's
composite is actually readable — rung 58's converse reading, surviving here for a reason rung
58 did not need:

| leg | `s_eng` bare | `s_eng` armed | Δ | fuel removed |
|---|---|---|---|---|
| incidence `M`=0.500, `v`=0.05 | 0.0489 | 0.0954 | **+95.1 %** | 2.81e−03 → 7.49e−04 |
| incidence `M`=0.509, `v`=0.10 | 0.0299 | 0.1242 | **+315.9 %** | 4.91e−03 → 2.83e−04 |
| incidence `M`=0.518, `v`=0.15 | 0.0123 | 0.1603 | **+1200.1 %** | 9.56e−03 → 6.61e−05 |

against rung 58's **−0.16 %** for the feedforward leg — **594× to 7500×**. So the stator
massively re-times a floor leg *even in the one coordinate whose wall it does not move*, and
the reason is the half re-referencing cannot reach: **it fixes the WALL, not the TRAJECTORY.**
Rung 53's work channel pushes the running line down regardless, and a floor's engagement
answers to the DISTANCE between the two. Rung 48's cap is immune because the stator moves
neither the wall nor the trajectory in `Wf/pt3` (rung 59: the ordinate cannot see a stator at
all).

---

## What it does to its neighbours

- **Rung 58 — its third finding CONFIRMED, its DIAGNOSIS CORRECTED.** *"A `φ`-referenced leg
  is not composable at all"* is right, and the disjointness is real. But disjointness is a
  **symptom**: it can be repaired (24× at a constant setting, entirely on rung 58's own
  schedule) and the leg is *still* not composable, because the disease is the pinning. Its own
  by-product — the pinned floor giving credit `= v` — was the disease, filed as a curiosity.
- **Rung 58 — its ranking INVERTED.** The constant setting, whose interaction was an order of
  magnitude *smaller*, is the leg that cannot be composed; the schedule can. Interaction and
  composability are different properties with opposite orderings.
- **Rung 59 — COMPLETED, as the other half of one law.** A schedule's matching is pure
  RE-INDEXING (abscissa 100 %, ordinate 0 %); a floor has no abscissa at all, so its matching
  is pure RE-VALUING — and the re-valuation is not free calibration but the closed-form
  coordinate map `1/φ ↦ 1/φ − v`. Rung 59 matched a leg and gained a correct measurement;
  rung 60 matches a leg and gains only admissibility. The difference is relocate-vs-pin.
- **Rung 57 — LOAD-BEARING on a new axis.** Its no-clock law is what makes the credit the
  ramp-invariant half of the criterion; without it the threshold would have no clean clock.
- **Rung 53 — EXTENDED to a sixth object, and BOUNDED for the first time.** Its
  coordinate-dependence law reaches a limiter's SET POINT (after a margin, a constraint's
  severity, a lever's cost, a limiter's composability, a schedule's calibration). The bound:
  re-referencing to the stator-invariant coordinate fixes the WALL and *not* the trajectory,
  so coordinate-freedom is not the same as stator-immunity.
- **Rung 49 — its instrument SURVIVES, its diagnostic role does not.** The `φ` floor is intact
  as a limiter; what it cannot do is carry a credit measurement beside a wall-mover.

---

## The instrument

`IncidenceLimiter` beside rung 49's `SurgeLimiter`, and a rung-60 section on
`ScheduledStatorTransient`. **No new constant enters** — `m_lim` is rung 36's same disclaimed
floor read as an incidence (`from_phi` / `from_margin`).

- `IncidenceLimiter(spool, m_lim)` / `.at(T_c, v)` — the floor, and the rung-49 leg it IS at a
  setting. `.at(T_c, 0.0)` is **float-identical** to the hand-built rung-49 floor.
- `_resolve_floor(...)` — the one plumbing point. A `SurgeLimiter` returns **by identity**
  (`is`), so rungs 49–59 reach the same object; only `_surge_fuel` and `_leg_residual` consult it.
- `matching_rules(sm, v)` — the two matching rules and their derived gap, checked against
  `v·sm/(1+sm)` rather than asserted.
- `_band(...)` / `set_point_bands(...)` — the admissible band in BOTH coordinates, the gap, and
  the `credit − excursion` identity.
- `composability_ladder(...)` — the threshold walked to its crossing, over stator legs at fixed
  `r` **or** over `r` at a fixed leg. It refuses both axes at once: they carry different halves.
- `_pin_audit(...)` — the BLOCKER. All three degeneracies named and asserted per cell.
- **`floor_composite(...)`** — THE RUNG: the four cells with a floor leg, the regime, the
  DERIVED tautology value, and the timing half. It refuses a feedforward leg at the door.

### The reduce

1. **`v = 0` ⇒ bit-for-bit rung 49.** `at(T_c, 0.0)` computes `1/(T_c + 0.0 − m_lim)` and
   `x + 0.0 == x` exactly, so the identical float reaches `_surge_fuel` and the whole march is
   bit-identical to the hand-built `SurgeLimiter` — the strong identity reduce, not a tolerance.
2. **A rung-49 floor passes the resolver by IDENTITY** (`is`) on a bare, a constant and a
   scheduled machine, so rungs 49–59 cannot be perturbed by the resolver existing.
3. Leg-free and `accel`-leg marches are bit-for-bit unchanged; rung 58's `composite_credit`
   still runs and still reports its leg.
4. The design run is bit-for-bit rung 6.

---

## Concessions

- **The load-bearing body is a CONSTANT stator setting, and that is why there is no new
  plant.** At constant `v` the resolved floor is a scalar, so the incidence leg *is* a plain
  rung-49 `φ` floor at the matched value and the whole rung is a leg swap. On a **schedule**
  the set point is state-fed — genuinely new plant — and that branch is reported (the schedule
  rows of the ladder) but carries no claim the constant branch does not.
- **`v` is read through `v_of` against the DESIGN `Tt2`**, the convention rungs 57/58 already
  use in `_read` and `_refine_min`. It is exact at the design flight condition, which is where
  every claim is made; off-design it would need the live `Tt2`.
- **The admissible set point is necessarily near-extreme in both bands**, and that is
  structural rather than a bad choice: an overlap 20 % of a band wide leaves nowhere else to
  put it. It is why the timing shifts are so large, and why `ADMISSIBLE` in the gate is three
  measured pairs rather than a formula.
- **The `φ`-floor rows are INADMISSIBLE and are reported as such.** A `φ` floor inside the bare
  band binds from `s = 0` once the stator is armed (`from_zero`), which is rung 58's finding.
  They are shown for the tautology's value, not as a composite.
- **One spool, one currency, one gas, one flight point.** LP-side, `M_i`, CPG — as rungs
  53/57/58/59. `M_φ` is never differenced (rung 58's finding).
- **`φ_surge` is still rung 36's imposed constant**, anchoring `T_c = 1/φ_surge`. Its LEVEL is
  disclaimed exactly as in rungs 36/41/53/57/58/59; the load-bearing objects here are two exact
  identities, a criterion and a ratio, none of which reads the level.

## The next seam

**A limiter that RELOCATES rather than FLOORS the protected variable.** Rung 60 says the
composable legs are the ones that move a minimum's location; rung 49's family all set its
value. The object that would settle it is a `φ`-*rate* limiter — a leg that caps `dφ/ds`
rather than `φ` — which arrests the descent without fixing where it stops, and so should
compose where every floor cannot. It is new plant (a derivative of the protected variable is
not available inside a derivative call without a state), and it is the first leg that would
test the relocate-vs-pin law rather than illustrate it.

Then, unchanged: **stator + bleed together** (rung 53's saturation), a **bleed schedule**
`b(n_L)`, and the **lag SHAPE / two-lag cascade** (rung 52's own seam).

## Anchor

`docs/plans/rung60-anchor-matched-floor.md` — probes A–E (probe A's advisor-forced measurement
that inverted the schedule branch, probe C's blocker that BECAME the rung), the five
predictions as registered, and their scoring: **P1 HIT · P2 HIT · P3 HIT · P4 HIT · P5 HIT.**
It also records the estimate the advisor blocked — a `credit − excursion` gap computed off rung
58's published table by silently mixing two machines — which survived in sign on one branch and
inverted on the other.
