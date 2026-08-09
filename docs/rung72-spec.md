# Rung 72 — TWO LOOPS ON ONE ACTUATOR

Rung 52's `φ` FUEL leg armed **beside** rung 47's `Tt4` governor, so two limiters drive the
**same actuator**. Six states, four clocks, four loops, **three actuators**:

    dgf/ds = ( F(ν,gr,q,v) − gf ) / τ_f   F = rung 52's clip,  φ_lp ≥ φ_lim   [FUEL   ]
    dgr/ds = ( R(ν,gf,q,v) − gr ) / τ_g   R = rung 47's clip,  Tt4 ≤ Tt4_max  [GOV    ]
    dq/ds  = ( C(ν,gf,gr,v) − q ) / τ_q   C = rung 65's b_cmd, φ_lp ≥ φ_lim   [VALVE  ]
    dv/ds  = ( V(ν,gf,gr,q) − v ) / τ_s   V = rung 68/69's,   φ_lp or M_i     [STATOR ]

`n = 4` — the last unoccupied **shape** after rungs 68–71 filled every `(3, m)` cell, and the
seam rung 70 § 6.1 and rung 71 § 11 both named. Rung 71 § 11 asked one question of it: *does
§ 1's `m` count constraints or actuators?*

> **HEADLINE — A SHARED ACTUATOR ADDS A SWITCH BETWEEN PLANTS, NOT A LOOP.**
> Min-select makes authority **exclusive**, so the masked leg reaches the plant through a
> `max()` that is *flat* in it: its column is `(−1, 0, 0, 0)ᵀ`, the block is triangular, and
> this **one** six-state plant IS rung 68, 69, 70 or 71 at every instant — polynomial for
> polynomial, to **7.1e−17** — plus a free pole at the masked leg's own clock.
>
> | stator watches | fuel leg holds | governor holds |
> |---|---|---|
> | `φ`   | **RUNG 68** (`m_live` 1, zeros **2**) | **RUNG 70** (`m_live` 2, zeros **1**) |
> | `M_i` | **RUNG 69** (`m_live` 2, zeros **1**) | **RUNG 71** (`m_live` 3, zeros **0**) |
>
> So `zeros = n_live − m_live`, counting the loops that hold **authority** — and **the RANK
> CHANGES at the hand-over with no state, no gain and no clock moving.** No earlier rung in this
> family could exhibit that, because none had a quantity that could change without something
> moving.

> **AND RUNG 71 § 11's QUESTION HAS A THIRD ANSWER: NEITHER.** The constraint reading is wrong
> by exactly **one on both arms**; the actuator reading is right on the `φ` arm and wrong on the
> incidence one — right by *coincidence*, because there the masked leg and the missing gradient
> happen to cancel. **Without the second arm this rung would have shipped "`m` counts
> actuators" and been wrong.**

> **AND THE `(4, m)` CELLS ARE A MIRAGE.** `n_live` is 3 at every instant, so a shared actuator
> cannot occupy them at all: min-select collapses `(4, m)` to `(3, m)` plus a pole. Rung 71
> § 11 called `n = 4` "the only unoccupied shape at this size" and named two routes to it —
> **this one is closed by being shown impossible**, and rung 69 § 11's (a fourth LP lever)
> stays open. **This is the first rung to close a seam by refuting its premise.**

**AND IT IS RUNG 66's MIRROR.** Two loops on one **variable** are ONE loop with the rates added
(`pair = 1` exactly, `det J ≡ 0`, rank one) — maximally *redundant*. Two loops on one
**actuator** are ONE loop plus a free pole (`pair_FR = 0` exactly, `det J = −det J_parent/τ_m`)
— maximally *exclusive*. The two corners of one question, six rungs apart.

Pre-registration: `docs/plans/rung72-anchor-shared-actuator.md`, whose § 0 discloses its own
order. Gates: `tests/test_rung72.py`.

---

## 0. WHAT MOVED, AND WHAT DID NOT

| | rung 69 | rung 70 | rung 71 | **rung 72** |
|---|---|---|---|---|
| the loops | fuel, valve, stator | gov, valve, stator | gov, valve, stator | **fuel, gov, valve, stator** |
| actuators | 3 | 3 | 3 | **3, one of them SHARED** |
| states | 5 | 5 | 5 | **6** |
| `(n, m)` | (3, 2) | (3, 2) | (3, 3) | **(4, ·) — and `n_live` = 3** |
| zeros | 1 | 1 | 0 | **2 / 1 / 1 / 0, by CELL** |
| what selects the plant | the build | the build | the build | **AUTHORITY, at run time** |

The plant, the ramp, the maps, `φ_lim`, `b_max`, `v_max`, `m_lim` and `Tt4_max` are all
inherited. **The only thing added is rung 52's leg**, which is the one lever this ladder has
carried since rung 52 and never armed beside the governor — rungs 68/70/71 each assert against
it in so many words.

### 0.1 The one modelling decision, DECLARED

    mf = mf_sched − max(gf, gr)      MIN-SELECT in clip coordinates.  THE PLANT.
    mf = mf_sched − gf − gr          the SUM law.  AN INSTRUMENT (§ 3), never the plant.

The first is what a fuel control does: every leg computes the fuel it would allow, the lowest
wins, which in clip coordinates is the largest cut. The second double-clips — rung 70's own
integrator refuses it in so many words — and is carried **only** as § 3's isolation instrument,
the role rungs 50/51's forced release edges played. `_applied_clip` is the single place the law
lives, so no reader can compose the two clips differently from the march that produced its base
point.

---

## 1. THE DERIVATION

Rung 69 § 1, re-read unchanged by rungs 70 and 71:

    row_i(M) = −(1/c⁽ⁱ⁾_i)·∇c⁽ⁱ⁾ᵀ      ⇒      rank M = dim span{∇c⁽ⁱ⁾} =: m ,   zeros = n − m

**That formula has a precondition nothing before this rung could violate: the loop must own the
actuator it solves for.** Every plant from rung 62 to rung 71 has one law per actuator, and § 1
says so in passing without ever needing it. Here two laws drive one actuator, and the
precondition becomes the subject.

### 1.1 What a shared actuator does to a row

Both fuel-side laws are **inherited verbatim** and both compute their clip from the **scheduled**
fuel — rung 47's discipline (*`required` is what the clip WOULD have to be, not what the current
clip makes it*) and rung 52's (*solved from the scheduled fuel so arming one leg cannot perturb
another's bracket*). Neither was written with a second clip in mind. Therefore

    F does not depend on gr   and   R does not depend on gf        ⇒   F_r = R_f = 0, EXACTLY

and each fuel row carries a `−1` on its own axis with **zero on the other fuel axis**. A row of
that shape is not `−(1/c_i)∇c` for any constraint the plant has: under min-select `φ` and `Tt4`
depend on the fuel states only through `max(gf, gr)`, so the true gradient is flat in the masked
one and the masked leg's own `c⁽ⁱ⁾_i` is **0**. **Its row formula is UNDEFINED — not parallel to
another's, undefined** — while `J` stays perfectly finite, every row being
`(∂R_i/∂x_j − δ_ij)/τ_i`. The closed form `zeros = n − m` has **no value to take**; `det J`, the
zero count, `c1` and the spectrum all remain measurable. That is the honest shape of the finding,
and it rhymes with rung 71's own *counts gradients, not live loops*.

### 1.2 The masked COLUMN, and why the spectrum splits

Order the states `(gf, gr, q, v)` and take the governor holding authority (`gr > gf`). `max()`
is flat in `gf`, so nothing downstream of the actuator sees the masked state:

    C_f = V_f = 0   and   R_f = 0 (§ 1.1)      ⇒     column_gf(M) = (−1, 0, 0, 0)ᵀ

`M` is block **upper**-triangular, and therefore

    eig(M₄) = { −1 } ∪ eig(M₃) ,   det M₄ = −det M₃ ,   zeros(72) = zeros(parent)

with `M₃` the parent rung's own 3×3 block, entry for entry. In `J = diag(1/τ)·M` the split
survives with its clock attached: **a pole at exactly `−1/τ_masked`**, independent of every gain,
every other clock and the plant, and `c0(72) = −c0(parent)/τ_masked`.

**The masked leg is running open loop** — a first-order lag driven by a reference it cannot act
on. That is min-select windup, seen in the spectrum.

**THE POLE IS A CONSEQUENCE, NOT A SECOND MEASUREMENT, AND IT IS NOT GATED.** `_jac4` puts
`−1/τ_i` on the diagonal *by construction*, so once the masked column's off-diagonal entries are
measured zero, `A e_m = −(1/τ_m) e_m` is algebra. Reporting the eigenvalue as confirmation would
be the shipped instrument agreeing with itself — rung 67 gate 9's retraction and rung 71 § 1.4's
`c1`, in a third shape. **§ 4 gates the mask leak and reports the pole.**

### 1.3 The rank law that replaces `zeros = n − m`

A loop holding authority contributes its constraint's gradient; a loop that does not contributes
its own axis, which no other row occupies. So

    rank M = m_live + n_masked        ⇒        zeros = n_live − m_live

with `n_live` the loops **holding authority** and `m_live` the distinct constraint gradients
among them. `n_live = 3` on both arms and at every instant; `m_live` is 1, 2, 2 or 3 by cell.

### 1.4 The two readings rung 70 § 6.1 / rung 71 § 11 offered, and the third

| arm | constraints | actuators | constraint reading | actuator reading | § 1.3 | **measured** |
|---|---|---|---|---|---|---|
| `φ`, fuel holds | 2 | 3 | 2 | 1 | **2** | **2** |
| `φ`, gov holds | 2 | 3 | 2 | 1 | **1** | **1** |
| `M_i`, fuel holds | 3 | 3 | 1 | 1 | **1** | **1** |
| `M_i`, gov holds | 3 | 3 | 1 | 1 | **0** | **0** |

Neither offered reading survives: the constraint one is wrong on three of four cells, the
actuator one on two. Both are **static** readings of a quantity that turns out to be dynamic.

---

## 2. MEASURED — § 0's WINDOWS, AND THE HAND-OVER

`Tt4_max = 1200 K`, `φ_lim = 0.80`, `b_max = 0.10`, `v_max = 0.20`, `m_lim = T_c − 1/φ_lim`, all
inherited. `ds = 0.005`.

### 2.1 The windows — and they are the WIDEST in the family since rung 68

| leg | `φ` arm | INCIDENCE arm |
|---|---|---|
| fuel leg riding | 0.005 … 1.700 (340) | 0.005 … 1.700 (340) |
| governor riding | 0.110 … 1.700 (319) | 0.105 … 1.700 (320) |
| valve riding | 0.000 … 0.610 (123) | 0.000 … 0.645 (130) |
| stator riding | 0.005 … 0.435 (87) | 0.005 … 0.245 (49) |
| **ALL FOUR** | **0.110 … 0.435 (66) = 19.35 %** | **0.105 … 0.245 (29) = 8.50 %** |

against rung 71's 2.05 %. A rung whose *feasibility* was the open question has the most
comfortable sample since rung 68, because the fourth loop is a fuel-side one and the fuel side is
live over essentially the whole march.

### 2.2 Authority changes hands ONCE, inside the joint window

| arm | fuel holds | governor holds | hand-over |
|---|---|---|---|
| `φ` | `s` ≤ 0.205 | `s` ≥ 0.205 | **0.205** |
| incidence | `s` ≤ 0.245 | `s` ≥ 0.245 | **0.245** |

and **both legs want a cut over 319 of 341 points**, so the masked leg is *riding and reaching
nothing*, not dormant. The hand-over sits **inside** the joint window on every arm, which is what
lets § 3 measure a rank change on both sides of it **on one trajectory**, with no second plant.

### 2.3 Why rung 71's own cell is nearly empty at matched clocks — and it is derivable

The incidence arm's governor-authority cell holds **1 point of 35** at `τ = (0.05,0.05,0.05,0.05)`.
The two events that end the fuel leg's authority and the incidence stator's window are the **same
event**: `φ_lp` recovering through the floor shrinks rung 52's clip (so the governor overtakes it)
*and* makes the stator dormant (rung 71 § 0.2). So the cell is not thin by accident.

**THE WIDE-CELL CLOCK ARM** `(τ_f, τ_g, τ_q, τ_s) = (0.20, 0.01, 0.50, 0.05)` reaches it: a fast
governor and a slow fuel leg hand over **early**, a slow valve keeps the stator riding **late**.
It takes the cell from 1 point to **21**. All four are swept march coordinates and it is rung
71's own slow-valve device, one clock wider.

---

## 3. MEASURED — THE FOUR CELLS, WHICH ARE THE RUNG

`ds = 0.002`, both arms, both clock arms, every point interior and regime-checked.

| arm | authority | parent | n | `zeros` | § 1.3 | parent-polynomial gap | `det J` |
|---|---|---|---|---|---|---|---|
| `φ` | fuel | **rung 68** | 29 | **2** | 2 | 3.6e−17 | ≈ 0 |
| `φ` | gov | **rung 70** | 132 | **1** | 1 | 7.1e−17 | ≈ 0 |
| `M_i` | fuel | **rung 69** | 54 | **1** | 1 | 7.1e−17 | ≈ 0 |
| `M_i` | gov | **rung 71** | 22 | **0** | 0 | 2.8e−17 | **+5.9e4** |

**`det J` is alive in exactly one cell — rung 71's own**, the only full-rank plant in the family.
That is rung 71 § 1.3's factorisation surviving a rung that adds no factor: the masked leg
multiplies it by `−1/τ_masked` and nothing else.

### 3.1 The test is a POLYNOMIAL IDENTITY, not a root match — and that is not a detail

The claim is `p₄(λ) = (λ + 1/τ_masked)·p₃(λ)`, with `p₃` rebuilt from the **shipped** rung-68/
69/70/71 readers (`_triple_gains_at` → `_invariants`). Compared **root by root** the rung-68 cell
returns a gap of **4.6e−7** while every other cell sits at 1e−13 — and that number is not a
disagreement between two plants. Diagnosed rather than tolerated:

* the two readers land on the **same** manifold base point — `v_base` agrees to **0.0** exactly,
  so a manifold mismatch is ruled out;
* in that cell `pair_RC = pair_CV = pair_RV = 1.000000` — rung 66's identity three times over,
  which is rung 68's own rank-ONE signature — so `c1 ≈ 1e−08` and `c0 ≈ 1e−13` and the parent has
  a **double** zero root, this rung a **triple** one;
* a repeated root is resolved only to the square root of the working precision, and
  4.6e−7 ≈ √1e−14.

Compared **coefficient by coefficient** the same cell returns **3.6e−17**. The instrument had a
floor, not the plant, and the coefficient form states the claim exactly.

### 3.2 The four exact zeros

`F_r`, `R_f`, `pair_FR` and the masked leg's `C`/`V` gains are **`== 0.0`**, not "< tol", at every
interior point on both arms — gated as exact equality, because `max()` is flat and the
scheduled-fuel discipline is structural. Beside them, the **live** gains are checked non-zero
(worst `1.1e−3`), so "exactly zero everywhere" is not being bought with a decoupled instrument.

**`pair_FR = 0` is rung 66's identity at its opposite corner** (§ Headline).

---

## 4. THE ISOLATION INSTRUMENT — AND ITS OWN CONFOUND

The SUM law is read **at the min-select trajectory's own base points**: one law swapped, nothing
else, which is rung 71's `m70`-at-identical-points device applied to a composition law instead of
a rung. **It is not marched** — double-clipping starves the engine and its own march stops at 84
points of 341 with `Tt4` never reaching the redline, so marching it would confound the law with
the state.

**AND THE FIRST VERSION OF THIS READER AGREED WITH ITSELF.** At `τ_f = τ_g` the SUM law has
`(1, −1, 0, 0)` as an exact eigenvector with eigenvalue `−1/τ` — the two fuel rows are `(−1,0,·)`
and `(0,−1,·)` and the shared columns cancel in the difference direction — so the free-pole test
passes under **both** laws:

| clocks `(τ_f, τ_g)` | min-select pole residual | SUM pole residual |
|---|---|---|
| (0.05, 0.05) — **MATCHED** | 3.6e−16 | **3.6e−16 — the confound** |
| (0.05, 0.08) | 1.6e−15 | **3.6e−01** |
| (0.02, 0.09) | 6.4e−15 | **9.0e−02** |

The matched arm is **carried in the table and gated**, because a discriminator quoted from it
alone is a discriminator that never tested anything. It is rung 66's lesson (*my own stability
floor was unsafe*) and rung 70's (*a gate computing my own formula twice*), for the third time.

**The zero count also moves** — by exactly one, at **fuel-authority** points only (2 → 1 on the
`φ` arm), where min-select was masking a leg. The anchor's D5 derived that it would not; that is
scored in § 9.

**TWO FACTS ABOUT SUM's STABILITY, AND NOT A THEOREM.** On the `φ` arm the SUM Jacobian has a
right-half-plane root (`Re(λ)·τ_min = +3.6e−2`) where min-select has none (`+1.4e−8`, the zero
roots); on the incidence arm SUM is stable (`−9.5e−3`). And its own march terminates at 84/341
points. Both are reported; neither supports "min-select is stabilising" as a theorem, because a
frozen Jacobian on a trajectory the SUM law never marched cannot carry one.

---

## 5. THE LEDGER — WHAT A MASKED LEG BUYS

16 cells, three currencies, `ds = 0.005`. A spectral reading says a masked leg is coupled to
nothing; the ledger disagrees, because **authority is a function of `s`** and a leg masked late
held the actuator early.

| | `φ` arm | incidence arm |
|---|---|---|
| fuel leg's marginal `φ` credit | **+1.6e−5** | **+1.9e−4** |
| its solo credit | 1.6e−2 | 1.6e−2 |
| **kept** | **0.11 %** | **1.2 %** |
| `min φ_lp` with / without it | 0.795155 / 0.793448 | 0.791380 / 0.785043 |
| marginal `Tt4` **integral** | +2.22 | +4.94 |
| `max Tt4` with / without it | 1283.36 / 1283.07 | 1282.76 / 1280.91 |

**It buys `φ`, and it DOES spend the governor's currency — in opposite directions on the two
readings.** The exceedance *integral* improves (a credit of ~8 % and ~17 % of the governor's own)
while the *peak* gets **worse** (+0.29 K, +1.86 K). The anchor predicted the peak unmoved; § 9
scores it.

**AND ITS SOLO CELL IS DEGENERATE, WHICH IS QUOTED BESIDE THE RATIO.** Rung 52's leg **alone**
holds `max Tt4` at the initial 1000 K — it starves the accel outright, `E = 0` — so the `kept`
denominator is taken on a trajectory no other cell shares. Rung 71 § 4's *quote the absolute
integral beside the ratio*, with the confound larger here than there.

---

## 6. CONCESSIONS (in addition to every one rungs 62–71 list, all inherited)

* **The composition law is DECLARED, not derived.** Min-select is what fuel controls do and the
  SUM law is refused as non-physical, but neither is derived from anything in this model. Every
  structural result in § 1.2/§ 1.3 is *contingent on min-select*, and § 4 measures what changes
  without it.
* **AND ON THE SCHEDULED-FUEL REFERENCE.** § 1.1's `F_r = R_f = 0` is a property of the two
  inherited laws, not of shared actuators in general. A leg referenced to the **applied** fuel
  would give `F_r ≠ 0`, both fuel rows would couple, and the block would not be triangular. That
  is the sharpest seam here (§ 11) and it **bounds the headline**, which should be read as *a
  shared actuator under min-select, with schedule-referenced legs*.
* **The incidence arm's governor-authority cell is 1 point at matched clocks** and is measured on
  the WIDE-CELL clock arm (§ 2.3). Its thinness is derived, not numerical, but the cell's numbers
  are a reading over 21 points and are quoted as such.
* **`Tt4_max = 1200 K` is RUNG 67's imposed value**, taken verbatim so the numbers difference
  against rungs 67/70/71 (rung 63's lesson). `φ_lim`, `b_max` and `v_max = 0.20` remain imposed;
  `m_lim` adds no constant (rung 69 § 10).
* **The free pole is algebra, not a measurement** (§ 1.2), and is reported, never gated.
* **A `tie` is a set of measure zero here** and every reader skips points within `4·dg` of the
  hand-over, reporting the count. A hand-over that *lingers* (a tie of positive measure) is a
  different plant and is not measured — § 11.
* **`τ_f` is the AsymmetricLag's local value** `lag.tau(required, g)`, so the fuel clock is a
  function of the trajectory. Every table records the value used.
* All four clocks are swept coordinates on the march's own `s`. **Orderings, signs and
  invariances are the claims; every magnitude is disclaimed.**
* The spectrum is sampled at finitely many trajectory points — a diagnostic that can miss a brief
  excursion (rung 65's retracted trap), not a proof of convergence.
* The STAGE STACK (rungs 55/56) is still off the transient ladder, and this still does **not**
  close rung 63's *fuel + bleed + STATOR* seam, which wants the stator as an OPEN-loop schedule.

---

## 7. THE REDUCE — FIVE ARMS, ALL BY DISPATCH, ALL BIT-FOR-BIT

| arm | reduces to | measured |
|---|---|---|
| no fuel leg, incidence stator, governor | **rung 71** | 341 pts, worst **0.0** |
| no fuel leg, `φ` stator, governor | **rung 70** | 341 pts, worst **0.0** |
| fuel leg, incidence stator, `tau_gov=None` | **rung 69** | 341 pts, worst **0.0** |
| fuel leg, `φ` stator, `tau_gov=None` | **rung 68** | 341 pts, worst **0.0** |
| no stator, no fuel leg, governor | **rung 67** | 341 pts, worst **0.0** |

**The dispatch asks for BOTH FUEL LEGS, not for a stator** — deliberately. The shared actuator is
the subject, so the plant is owned with a stator (§ 3's four cells) or without (the ledger's `FG`
and `FGV` cells, which have **no inherited home at all**: rung 52's own integrator refuses `lag`
beside `tau_gov` in so many words). Gating entry on a stator would leave those cells unmarchable
and the ledger holed exactly where the fourth loop stands alone.

A **sibling integrator** was required, not rung 71's re-entry: a state is genuinely added, so
rung 71's *reuse, do not copy* argument does not carry and rungs 68/69/70's precedent does.

**THE FIVE REDUCE TESTS ARE DELIBERATELY NOT MARKED `slow`**, and it is recorded because
`conftest.py` says `-m "not slow"` has no backstop. Each runs two 341-point marches, so they are
not free — but the reduce spine is *the project's spine*, and rungs 69/70/71 leave their own
(2, 4 and 4 tests) unmarked for the same reason. A stated choice, not an omission. Every
FINDING sweep here — `authority_law`, `shared_gains`, `shared_cells`, `mask_discriminator`,
`shared_bill` — **is** marked. The refusals cost nothing: each asserts before any march.

**GATE TIMING IS UNMEASURED AT THIS RUNG AND SAYING SO IS THE POINT.** Three runs on this box
gave 15:04, then 8:37, then **9:21 for the 868-test `-m "not slow"` subset** — a subset cannot
take longer than the whole, so the box was not in a steady state and none of the three is a
reading. CLAUDE.md's `~3:18` / `~1:18` are therefore left as the last *trustworthy* numbers and
flagged stale, rather than overwritten with a measurement that looks verified and is not.

---

## 8. THE REFUSALS, AND THE INSTRUMENT GATED AGAINST ITSELF

Refused: an undeclared `_share_law`; `tau_gov` without `Tt4_max`; rungs 50/51's forced release
edges (twice over — `_stator_march` does not plumb them at all); an instantaneous valve beside
lagged legs; `ds` past the four-clock RK4 floor, **re-justified a fifth time on a fourth
argument** (the masked leg's pole is exactly `−1/τ_f` and the other three share the remainder).

**`_charpoly4` SILENTLY RETURNED A WRONG POLYNOMIAL AND NOTHING DOWNSTREAM COULD TELL.** Its
first version had `A` where Faddeev–LeVerrier needs `M_{k−1}`, and it produced an entirely
plausible spectrum: stable-looking roots, `det = 5.9e+05`, a root residual of 1e−09 (the root
finder was faithfully solving the wrong polynomial) and a parent comparison that merely came out
large. So the polynomial is checked against an **independent** trace and cofactor determinant and
against a **triangular** matrix whose spectrum is its own diagonal — and the broken recursion is
**rebuilt in the test and fed to the check**, because a self-test that has never failed on the
bug it was written for is ceremony (`test_charpoly_selftest_catches_the_broken_recursion`).

**AND THE BOUNDARY TRAP HAS TEETH HERE.** Rung 70 measures the governor's cross-gains against a
deliberately blind version, because losing `_b_state`/`_v_state` around `required` decouples the
odd loop and nothing fails. Under a shared actuator: a fuel leg whose `required` lost the
boundary would return `F_q = F_v = 0` and **its row would look exactly like a masked one** — the
rung would confirm its own headline through a bug. Both legs are asserted against both blind
versions, on every sampled point.

---

## 9. THE ANCHOR, SCORED

| | prediction | verdict |
|---|---|---|
| P1 | `zeros` = 1 (`φ`) / 0 (incidence) | **REFUTED AS STATED** — it is **2→1** and **1→0**, switching at the hand-over. The law § 1.3 it came from is **CONFIRMED at all four cells** |
| P2 | the parent's spectrum plus the free pole | **HELD**, and strengthened into a coefficient identity: worst gap **7.1e−17** |
| P3 | `c0 = −c0(parent)/τ_masked` | **HELD** — ≈0 in three cells, live only in rung 71's |
| P4 | the masked column is exactly zero, and swaps | **HELD** — `0.0`, both roles, both arms |
| P5 | SUM: the count holds, the pole goes | **HALF REFUTED** — the pole half holds *only at unmatched clocks* (the matched arm was my own confound, § 4); the **count DOES move**, by one, at fuel authority |
| P6 | fuel leg's marginal `φ` small and positive; `max Tt4` unmoved | **HALF REFUTED** — the credit holds (0.11 %, 1.2 % kept); `Tt4` **moves**, as an integral credit and a peak **debit** |
| P7 | `pair_FR = 0` exactly | **HELD** |
| P8 | five reduce arms bit-for-bit | **HELD** — worst 0.0 |
| P9 | the IC inherits and adds nothing | **HELD** — 1 iteration, residual exactly 0, both legs open dormant |
| P10 | the refusals | **HELD** |
| **D5** | *(derived, "not scored")* the SUM law does not move the zero count | **MEASURED FALSE — AND SCORED ANYWAY** |

**THREE MISSES, ONE ROOT CAUSE, AND IT IS THIS RUNG'S CONTENT.** P1, P5 and D5 all assumed the
**governor** holds authority throughout. It does not — rung 52's leg holds it over the early part
of every joint window, which is exactly why the plant has four cells and not two. The oversight
that broke the predictions *is* the finding: authority is not a static property of a build, and I
reasoned about it as though it were.

**AND D5 IS SCORED THOUGH IT SAT IN THE "DERIVED, NOT SCORED" SECTION.** That section exists so
paper-work done before measurement is not passed off as prediction — **not** so a wrong
derivation escapes correction. A derivation that is measured false and quietly dropped is exactly
what rung 63's lesson is about. The anchor is **not** edited (rung 70's precedent).

---

## 10. WHAT THIS DOES TO THE RUNGS BEFORE IT

* **RUNG 71 § 11's QUESTION: answered, with a third option.** `m` counts neither constraints nor
  actuators. Its `zeros = n − m` is **bounded again**: rung 71 showed `m` counts gradients and
  not live loops; rung 72 shows `n` counts loops with **authority** and not states.
* **RUNG 71's "the only unoccupied shape at this size": one of its two routes is CLOSED BY
  REFUTATION.** A shared actuator cannot occupy `(4, m)`. Rung 69 § 11's route stays open.
* **RUNGS 68/69/70/71 are re-read as FOUR REGIMES OF ONE PLANT.** Nothing they measured changes —
  every one of their spectra is reproduced here to 1e−13 — but the *table* they spent four rungs
  filling turns out to be indexed by a run-time quantity, not by four builds.
* **RUNG 66 acquires its mirror** (§ Headline), and its identity `pair = 1` is now one of two
  corners rather than a fact about cascades.
* **RUNG 67's *a zero cross-gain is saturation, never decoupling* gains a second exception.** It
  can also be **masking** — and saturation-filtering does not catch it, because the masked leg is
  nowhere near a stop.

---

## 11. NEXT SEAMS

* **AN APPLIED-FUEL-REFERENCED LEG.** § 1.1's `F_r = R_f = 0` is the whole triangular structure,
  and it comes from *both inherited laws computing at the scheduled fuel*. A leg that reads the
  applied fuel gives `F_r ≠ 0`, couples the two fuel rows, and destroys the block form — the
  spectrum would then be genuinely four-dimensional and `n_live` might reach 4 after all.
  **This is the sharpest seam here, and it is the direct test of § 6's bound on the headline.**
* **RUNG 69 § 11's `(4, 2)`** — 2-on-`φ` plus 2-on-`M_i`, still needing a fourth LP lever this
  plant does not have. **Not** closed by this rung: that is a different shape (this one is
  3-on-`φ` + 1-on-`Tt4`), and rung 63's lesson applies.
* **THREE legs on one actuator.** § 1.3 predicts two free poles and `n_live` still 3. If it holds,
  a fuel control's whole min-select stack is one loop plus a comb of open-loop poles.
* **A TIE OF POSITIVE MEASURE** — a hand-over that lingers rather than crosses (two legs holding
  equal clips over an interval). Every reader here skips it as a kink; it is a distinct plant and
  the only place the `max()` is not locally smooth.
* **TWO LEGS ON ONE ACTUATOR WATCHING THE SAME CONSTRAINT** (both `φ`). Then both cells have the
  same parent and the switch becomes invisible to the rank — a negative control for § 3.
* An **ASYMMETRIC valve** (rung 65) and an **asymmetric governor** (rung 67) — both still open.
* **Fuel + bleed + STATOR-as-a-SCHEDULE** — rung 63's seam, still open after 64–72.
* Everything rung 68 § 10 left: a plant with `|P| > 1`, and the real spatial/transported-CFD PDF.
