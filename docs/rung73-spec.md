# Rung 73 — THE APPLIED REFERENCE

Rung 72's two fuel-side legs, re-referenced to the fuel the engine is **actually burning**. Six
states, four clocks, four loops, three actuators — every one of them rung 72's. **The only thing
added is which fuel a leg computes its clip from.**

    dgf/ds = ( F(ν,gf,gr,q,v) − gf ) / τ_f   F = rung 52's clip,  φ_lp ≥ φ_lim   [FUEL   ]
    dgr/ds = ( R(ν,gf,gr,q,v) − gr ) / τ_g   R = rung 47's clip,  Tt4 ≤ Tt4_max  [GOV    ]
    dq/ds  = ( C(ν,gf,gr,v)   − q  ) / τ_q   C = rung 65's b_cmd, φ_lp ≥ φ_lim   [VALVE  ]
    dv/ds  = ( V(ν,gf,gr,q)   − v  ) / τ_s   V = rung 68/69's,    φ_lp or M_i    [STATOR ]

Rung 72 § 6 conceded, and § 11 named as **its sharpest seam**: *`F_r = R_f = 0` is a property of
the two inherited laws, not of shared actuators in general. A leg referenced to the applied fuel
would give `F_r ≠ 0`, both fuel rows would couple, and the block would not be triangular — the
spectrum would then be genuinely four-dimensional and `n_live` might reach 4 after all.*

> **HEADLINE — THE COUPLING IS REAL, AND IT LANDS IN THE WRONG COLUMN.**
> `F_r = −1` **exactly**, so the seam's *premise* holds. But triangularity was never a property
> of the masked leg's **ROW**. It is a property of its **COLUMN**, and `F_r` sits in the
> **authoritative** one. Under min-select the masked column is `(0,0,0,0)ᵀ` under **every**
> reference — `max()` is flat in the masked state and nothing downstream can see it — so
>
>     eig(M₄) = { 0 } ∪ eig(M₃)  ,   M₃ = the parent rung's 3×3 block, ENTRY FOR ENTRY
>
> **TRIANGULARITY IS A PROPERTY OF MIN-SELECT ALONE, NOT OF THE REFERENCE.** Rung 72's headline
> survives its own sharpest seam, and the bound its § 6 put on it — *read it as a shared
> actuator under min-select **with schedule-referenced legs*** — is **REFUTED**: the last clause
> comes off.

> **WHAT THE REFERENCE BUYS IS THE POLE.** Rung 72's free pole at `−1/τ_masked` moves to
> **exactly the origin**: a masked leg referenced to the applied fuel is a **pure integrator
> running open loop.** Rung 72 saw min-select windup's *lag*; this is the windup itself. So
> `zeros = n_live − m_live + n_masked` — **every one of rung 72's four per-cell counts gains
> exactly one** — and **`det J` dies in rung 71's cell**, the only full-rank plant in the family
> (`+5.9e4` under rung 72). *A reference is not a gain, not a clock and not a loop, and it
> changes the RANK.*

> **AND THE MASKED INTEGRATOR DOES NOT WIND UP.** Masked means `gr > gf ≈ req_f`, so
> `dgf/ds = (req_f − gr)/τ_f < 0`: the leg integrates the exceedance still **owed**, and the leg
> holding the actuator is already curing it. **An applied-referenced leg is self-anti-winding
> under min-select** — that is a property of the composition, not of these numbers, and it is
> why the pair composes at all without an anti-windup device.

**AND IT IS THE FIRST RUNG IN THIS FAMILY WHOSE FINDING IS INVISIBLE IN THE SPECTRUM AND VISIBLE
IN THE BILL.** The reference reaches two entries of a 4×4 and a state that is coupled to
nothing, yet it moves `max Tt4` by up to **+71 K** — because **authority is a function of `s`**,
and the reference moves the **hand-over**. Rung 72 § 5's own ledger under-reported the fuel
leg's peak debit by **110× / 39×** (§ 5).

Pre-registration: `docs/plans/rung73-anchor-applied-reference.md`, whose § 0 discloses its order
and reports **two** measurements taken before it existed. Gates: `tests/test_rung73.py`.

---

## 0. WHAT MOVED, AND WHAT DID NOT

| | rung 70 | rung 71 | rung 72 | **rung 73** |
|---|---|---|---|---|
| the loops | gov, valve, stator | gov, valve, stator | fuel, gov, valve, stator | **the same four** |
| actuators | 3 | 3 | 3, one SHARED | **3, one SHARED** |
| states | 5 | 5 | 6 | **6** |
| zeros | 1 | 0 | 2 / 1 / 1 / 0, by cell | **3 / 2 / 2 / 1, by cell** |
| the masked leg's pole | — | — | `−1/τ_masked` | **the ORIGIN** |
| `det J` alive in | — | **yes** | rung 71's cell **only** | **NOWHERE** |
| what is added | a sensor | a coordinate | a leg | **a REFERENCE** |

The plant, the ramp, the maps, the clocks, `φ_lim`, `b_max`, `v_max`, `m_lim` and `Tt4_max` are
all inherited. **No state, no gain, no clock and no loop is added.** That is what makes this the
cleanest single-variable experiment in the family — and what makes a rank change inside it
worth a rung.

### 0.1 The seam names one plant; the ladder admits THREE

Every cap in this ladder — `_topping_fuel`, `_surge_fuel`, `_sched_fuel` — is a **set-point
solve**: it returns the fuel at which the constraint is exactly met, a function of `(ν, q, v)`
and **not of the fuel it was asked about**. So `∂required/∂mf` is not a gradient; it is the
**branch indicator** `{0, 1}`, and "referenced to the applied fuel" resolves three ways:

| | law | fixed point when the leg HOLDS | verdict |
|---|---|---|---|
| **A** | only the DORMANCY TEST moves to `mf_app` | unchanged | not a plant — the guard half, which B and C both inherit |
| **B** | `req = g_own + (mf_app − cap)` | `mf_app = cap` ✓ | **THE PLANT** |
| **C** | `req = mf_app − cap` | `g = (mf_sched − cap)/2` ✗ | a P-controller with **2× droop**; § 3's instrument |

**C is not refused as broken** — it is a well-posed proportional law. It is refused as
*degenerate for this ladder*: a leg that structurally cannot reach its own floor makes every
currency in the rung-46…72 ledger measure a different object. It is carried as § 3's isolation
instrument, the role rung 72's SUM law and rungs 50/51's forced release edges played. Refusing
it silently is what rung 63's lesson is about.

### 0.2 The hook is ONE LINE and no solver changes

Because `cap` is fuel-independent, `mf_app − cap ≡ req_sched − applied_clip`, so

    req_applied = g_own + req_sched − max(gf, gr)          `_reference`, the ONE seat of the law

with `req_sched` the **shipped** rung-47 / rung-52 `required`. Nothing is re-bracketed and
rungs 46–52's laws are untouched. **When the leg HOLDS, `max(gf,gr) == g_own` and the hook
returns `req_sched` FLOAT-IDENTICALLY** — an explicit branch, not an arithmetic coincidence.
That is rung 48's `_sched_fuel` device, and it is load-bearing: `g_own + req − g_own` is not
`req` in binary floating point, and through a central difference of step `1e−7` the cancellation
shows up as a `4e−11` entry on the *authoritative* leg's own diagonal — turning § 1.2's *`M₃` is
the parent's block entry for entry* from an exact claim into a `1e−11` one.

### 0.3 The state floor moves into the initial-condition sweep

Rung 72's IC sweep needed no floor: both `required` closures return `max(0, ·)`. The hook returns
an **increment**, which can be negative, so a dormant leg beside a riding one has **no interior
fixed point at all** (`dg/ds = −clip/τ < 0`) and its equilibrium is the stop itself. The sweep
now applies the same physical stop the march applies after every step — **a no-op for rung 72,
verified bit-for-bit** (§ 7), and load-bearing here. Without it the ledger's fuel-riding cells
diverge at `gr = −0.126` after 60 iterations, which is how the omission was found.

---

## 1. THE DERIVATION

### 1.1 The premise holds; one line separates it from the conclusion

Governor holding (`gr > gf`), states `(gf, gr, q, v)`. The masked fuel leg's law is
`F = gf + req_f(q,v) − gr`, so

    F_f = +1   and   F_r = −1        EXACTLY — rung 72 § 11's own `F_r ≠ 0`, HELD.

But `F_r` is an entry of the masked leg's **row**, in the **authoritative** leg's column.
Triangularity lives in the *masked* column:

    C, V read mf_app = mf_sched − max(gf, gr)      flat in gf
    R reads its own applied fuel, also via max()    flat in gf
    F reads gf only through its own `+gf` term      → the DIAGONAL, not an off-diagonal
    ⇒  column_gf(M) = (0, 0, 0, 0)ᵀ    — ZERO, where rung 72 had (−1, 0, 0, 0)ᵀ

**`M` is still block upper-triangular.** The coupling the seam anticipated is real, is exactly
`−1/τ_m`, and **points the wrong way to break anything**: the masked leg is *driven by* the
authoritative one and reaches it through nothing. Rung 62's ONE-WAY, in a fourth shape.

### 1.2 What the reference actually buys

    eig(M₄) = { 0 } ∪ eig(M₃)         against rung 72's { −1 } ∪ eig(M₃)
    M₃ = the parent rung's own 3×3 block, ENTRY FOR ENTRY

because the **authoritative** leg's applied reference is the identity (§ 0.2) and `C`, `V` and
the other fuel law are untouched. Hence the pole at the origin, `zeros = n_live − m_live +
n_masked`, and `det J ≡ 0` in all four cells.

**The masked leg is a pure integrator running open loop.** Rung 72 called its own free pole
"min-select windup, seen in the spectrum". It was the windup's *first-order lag*. This is the
integrator.

### 1.3 THE INSTRUMENT HAD TO BE WEAKENED FIRST, AND THAT COSTS FIVE ORDERS OF MAGNITUDE

Rung 72 § 1.2 refuses to gate its free pole because `_jac4` writes `−1/τ_i` on the diagonal *by
construction*. **A pole at the origin sits in exactly that position** — reporting `λ = 0` from a
diagonal this rung also constructed would be the **fourth** instance of the shipped instrument
agreeing with itself (rung 67 gate 9, rung 71 § 1.4's `c1`, rung 72 § 4's matched clocks).

So `_jac4` no longer constructs the fuel-side diagonal: it reads a **measured** `F_f` and `R_r`,
two central differences rung 72 never needed. `(F_f − 1)/τ_f` reproduces rung 72's `−1/τ_f`
when `F_f = 0` is *measured*, so rung 72's readers are bit-unchanged (§ 7) and the assumption
set is strictly smaller.

**THE PRICE IS EXPLICIT AND IT IS THE ANCHOR'S ONE REFUTED TOLERANCE.** A measured diagonal
carries the float cancellation of § 0.2, so the parent-polynomial comparison lands at **5.3e−12**
where rung 72's constructed diagonal reached **7.1e−17**. *Not letting an instrument agree with
itself costs five orders of magnitude in the identity it is checking* — and the identity is
still exact to eleven figures. § 9 scores it as REFUTED-as-stated.

### 1.4 The three readings move DISJOINT halves of the same matrix

|  | masked row's diagonal | masked row's cross | authoritative row | pole | `M₃` | zeros |
|---|---|---|---|---|---|---|
| rung 72 (sched) | `−1/τ_m` | 0 | rung 72's | `−1/τ_m` | parent's | `n` |
| **B (applied)** | **0** | **`−1/τ_m`** | unchanged | **0** | **parent's** | **`n`+1** |
| C (literal) | `−1/τ_m` | `−1/τ_m` | **`−2/τ_live`** | `−1/τ_m` | **NOT parent's** | **`n`−1 or `n`** |

**B moves the pole and keeps the parent; C keeps the pole and moves the parent.** Two readings
of one seam that agree on `F_r ≠ 0` and disagree on everything it was supposed to imply — which
is what makes the headline a *measurement* rather than a choice of law.

---

## 2. MEASURED — § 0's WINDOWS, THE HAND-OVER, AND A THIRD CLOCK ARM

`Tt4_max = 1200 K`, `φ_lim = 0.80`, `b_max = 0.10`, `v_max = 0.20`, all inherited. `ds = 0.005`.

### 2.1 The hand-over is LATE on every arm and at every clock

| arm | clocks `(τ_f,τ_g,τ_q,τ_s)` | hand-over 72 → 73 | `max Tt4` 72 → 73 | `min φ_lp` |
|---|---|---|---|---|
| `φ`   | (0.05, 0.05, 0.05, 0.05) | 0.205 → **0.235** | 1283.36 → **1315.22** (+31.87) | **unmoved** |
| `φ`   | (0.20, 0.01, 0.50, 0.05) | 0.145 → **0.155** | 1217.79 → **1227.21** (+9.42) | unmoved |
| `φ`   | (0.20, 0.005, 0.80, 0.05) | 0.140 → **0.150** | 1208.94 → **1219.14** (+10.20) | unmoved |
| `M_i` | (0.05, 0.05, 0.05, 0.05) | 0.245 → **0.300** | 1282.76 → **1353.74** (+70.98) | **unmoved** |
| `M_i` | (0.20, 0.01, 0.50, 0.05) | 0.195 → **0.250** | 1217.77 → **1254.67** (+36.90) | unmoved |
| `M_i` | (0.20, 0.005, 0.80, 0.05) | 0.200 → **0.245** | 1208.88 → **1235.43** (+26.55) | unmoved |

`min φ_lp` is unmoved **to every printed digit** on all six arms (`worst |Δφ| = 0.0`, exactly).

**AND THE SIGN IS DERIVABLE, WHICH IS THIS RUNG'S FIRST CORRECTION OF RUNG 72.** A masked
governor referenced to the **schedule** races toward `req_sched` — the clip the *schedule* would
need — so it is credited with a cut the fuel leg has **already made**. Referenced to the applied
fuel it integrates `req_sched − gf`, the cut still **owed**. **The physically-correct governor
is the SLOWER one**, takes the actuator later, and lets the redline be approached with less
margin. Rung 72's redline protection was, in part, an artifact of a counterfactual — and only at
rung 72, because with one fuel-side leg `gf = 0` and the two references coincide.

**AND THE REFERENCE MOVES A SECOND THING, WHICH IS INERT AND IS NAMED HERE RATHER THAN LEFT
IMPLICIT.** `lag.tau(required, g)` picks attack-vs-release from `required`, and under the applied
reference `required` is the *referenced* value — so the masked fuel leg's own clock can sit on a
different branch than it would under rung 72 at the same base point (`req_sched > gr` in place of
`req_sched > gf`). It cannot reach anything: while masked the leg is invisible to the plant, and
§ 2.1 measures that authority never hands back, so the branch never propagates to the trajectory;
and every J-comparison in § 3 holds `τ` **fixed** across the two references, so it cannot enter a
gain either. The delay's sign above is therefore the governor's integrand alone.

### 2.2 No windup — measured, and it was the feasibility gate

341 of 341 points on every arm, `ic_iters = 1`, `ic_res = 0.0` (rung 72's P9 inherited).
`final g_fuel = 0.0` **exactly** on all six arms, and the masked leg's peak clip is *smaller*
than rung 72's on every arm (`1.4e−3` vs `8.9e−3` at matched clocks). Had it run away, the
hand-over would have slammed a wound-up clip onto the actuator and starved the engine — how rung
72 § 4's SUM law died, at 84 points of 341.

### 2.3 A THIRD clock arm, and why rung 72's coverage does not transfer

The delay in § 2.1 moves the hand-over past the incidence stator's own window, so at matched
clocks the **incidence / governor** cell — rung 71's — is **EMPTY** (0 points, against rung 72's
1), and rung 72's WIDE-CELL arm reaches it with 4. A **DEEP-CELL** arm is added:

    (τ_f, τ_g, τ_q, τ_s) = (0.20, 0.005, 0.80, 0.05)

which is rung 72 § 2.3's own device pushed one notch — governor twice as fast, valve 1.6× slower
— and takes the cell to **13 points** at `ds = 0.005`. All four entries are swept march
coordinates; no physical constant enters. The RK4 floor is live: `(0.40, 0.002, 1.00, 0.08)`
**trips it** at `ds·Σ(1/τ) = 2.58`.

---

## 3. MEASURED — THE FOUR CELLS, WHICH ARE THE RUNG

`ds = 0.002`, both arms, all three clock arms, every point interior and regime-checked; **zero
points skipped** for regime or switch proximity, and the parent reader interior at **every** one.

| arm | authority | parent | n | `zeros` | rung 72 | `gap_hi` | `null` | `|det|` |
|---|---|---|---|---|---|---|---|---|
| `φ`   | fuel | **rung 68 + a zero** | 32 | **3** | 2 | 5.1e−13 | 6.8e−13 | 6.6e−11 |
| `φ`   | gov  | **rung 70 + a zero** | 123 | **2** | 1 | 5.1e−13 | 6.8e−13 | 9.7e−09 |
| `M_i` | fuel | **rung 69 + a zero** | 58 | **2** | 1 | 1.1e−12 | 5.3e−12 | 2.9e−10 |
| `M_i` | gov  | **rung 71 + a zero** | 23 | **1** | 0 | 2.3e−13 | 2.3e−13 | 4.5e−07 |

One value of `zeros` per cell, all four cells, each exactly one above rung 72's. `worst_v_gap`
is **0.0 exactly** — the two readers land on the same manifold base point, so a manifold
mismatch is ruled out (rung 72 § 3.1's device). `worst |λ|/Σ(1/τ) = 0.909 < 1`, so the RK4 floor
is measured rather than trusted (rung 65's retraction).

**`det J` IS DEAD IN ALL FOUR, INCLUDING RUNG 71's.** Normalised by the `Σ(1/τ)⁴ = 4.1e7` it
scales with, the worst is `1.1e−14` — against rung 72's live `+5.9e4` in that same cell.

### 3.1 `gap` and `null` ARE ONE NUMBER, NOT TWO — and that is gated separately

The masked column's only non-zero entry is its **own diagonal** (`F_f − 1`, zero only up to
§ 0.2's cancellation), and `a₃` is minus the trace — so the `j = 1` term of the parent-polynomial
gap **reproduces the null residual entry for entry**. Quoting both as agreement would be this
family's **sixth** instrument-agrees-with-itself. The independent comparison is `gap_hi`
(`j = 2, 3, 4`), where the two readers genuinely meet, and it is gated as its own quantity.

### 3.2 The exact zeros, and the one quantity that is exactly ONE and cannot be gated as such

At every interior point on both arms:

    self_live  == 0.0    EXACT — the holding leg's applied reference IS the scheduled one
    mask_leak  == 0.0    EXACT — the masked leg still reaches the plant through nothing
    delta_rest == 0.0    EXACT — 14 of the 16 entries of J(73) − J(72), at the SAME base points
    self_masked  ≈ +1    to 3e−12          cross_masked ≈ −1   to 3e−12

**THE TWO EXACT ZEROS ARE GATED AS EQUALITY AND THE TWO ONES ARE NOT, AND THAT IS NOT A DOUBLE
STANDARD.** `self_live` is exact because the hook takes an explicit identity **branch**;
`self_masked` is a central difference of a **sum** (`gf ± dg + raw − gr`) and float addition does
not distribute. *An exact zero survives a difference quotient; an exact one does not.* The
anchor's P7 asked for both and § 9 scores the miss.

Beside them the **live** gains are checked non-zero (worst `1.1e−3`), so "exactly zero
everywhere" is not being bought with a decoupled instrument — and rung 72's `_assert_fuel_boundary`
runs at every sampled point, on both legs, against both blind versions.

---

## 4. THE ISOLATION INSTRUMENT — reading C, and the discriminator that FAILED

C is read at reading B's own base points: one law swapped, nothing else (rung 71's device, rung
72 § 4's, third instance). It is **not marched** — a leg that lands at half its own required
clip holds neither floor, so its trajectory would confound the reference with the state.

| | root at the ORIGIN | root at `−1/τ_masked` | live leg's diagonal | zeros vs rung 72, **per point** |
|---|---|---|---|---|
| rung 72 | (already has one in 3 cells) | **4.4e−15** | 0.0 | — |
| **B** | **1.2e−06** | **0.094 … 1.000** | **0.0** exactly | **+1, everywhere** |
| C | 1.5e−12 … 0.118 | **2.7e−15** | **−1.0** exactly | **−1 or 0, never +1** |

**AND THE OBVIOUS DISCRIMINATOR IS THE ONE THAT FAILS.** *Is there a root at the origin?* cannot
separate B from rung 72 at all: rung 72 already has zero roots in three of its four cells, so
`origin_72` comes back at `1.3e−13`. What separates them is the **count, differenced per point**
— and even that must not be pooled, because this reader spans both authority cells whose counts
already differ by one under rung 72 alone. Pooled, `min(zeros_B) > max(zeros_72)` compares the
`φ` arm's fuel cell against its own governor cell and says nothing; it is the gate that failed
first, and it failed for that reason.

`1.2e−06` is the root finder's resolution on a **multiple** zero root, not a disagreement — rung
72 § 3.1's own diagnosis, one root deeper (`eps^(1/3) ≈ 6e−6`). The gated quantities are the
count and the null **direction**; the pole location is reported.

---

## 5. THE LEDGER — WHAT THE SCHEDULED REFERENCE WAS QUIETLY BUYING

Rung 72's own 16-cell ledger, run under **both** references and differenced. `ds = 0.005`.

| | `φ` arm | incidence arm |
|---|---|---|
| fuel leg's marginal **peak `Tt4` debit**, rung 72 | +0.291 K | +1.855 K |
| the same, **corrected** | **+32.158 K** | **+72.837 K** |
| **ratio** | **110.4×** | **39.3×** |
| its marginal `Tt4` **integral** | +2.222 → **−0.142** | +4.939 → **−0.307** |
| its marginal `φ` credit | +1.65e−5 → **−3.63e−5** | +1.87e−4 → **+7.16e−5** |
| `min φ_lp` (all four loops) | 0.795155 → **0.795155** | 0.791380 → **0.791380** |
| hand-over | 0.205 → **0.235** | 0.245 → **0.300** |

**AND THE SIGN CHANGE IS CONFIRMED ON A FINER GRID, BECAUSE IT IS A SMALL DIFFERENCE OF SMALL
NUMBERS.** The marginal `φ` column is a difference of violation integrals of order `1e−2`, at one
`ds`, while `min φ_lp` does not move at all — exactly the shape `docs/pt3-sensor-lag-negative.md`
was written about (*a 12 % gap inside my own `ds` band*). Re-read at `ds = 0.002`, a 2.5× refinement:

| | `φ` arm, `ds` 0.005 → 0.002 | `M_i` arm, 0.005 → 0.002 |
|---|---|---|
| marginal `φ`, rung 72 | +1.645e−5 → **+1.631e−5** | +1.870e−4 → **+1.866e−4** |
| marginal `φ`, corrected | −3.629e−5 → **−3.612e−5** | +7.163e−5 → **+7.198e−5** |
| peak `Tt4` debit ratio | 110.4× → **107.7×** | 39.3× → **39.5×** |

Every entry moves by under 1 %, and the `φ`-arm sign survives. The claim is quoted with its
band; the `Tt4` half was never in doubt (`+32 K` against a march-noise floor four orders below).

**RUNG 72 UNDER-REPORTED ITS OWN PEAK DEBIT BY TWO ORDERS OF MAGNITUDE**, and the mechanism is
§ 2.1's: the fuel leg's authority window is **early**, where the reference is the identity, while
the governor's is **late**, where it is not — so a masked governor credited with a cut it did not
make takes the actuator too soon and holds the redline that rung 72 reported.

**AND THE `φ` COLUMN MOVES TOO, WHICH THE ANCHOR SAID IT WOULD NOT.** The *delivered* `min φ_lp`
is unmoved to six figures, but the marginal `φ` **integral** falls, and on the `φ` arm it
**changes sign**: under the correct reference rung 52's leg is a net `φ` **debit** on the arm
whose currency it watches. The delayed governor's extra fuel is paid in `φ` after the leg's own
minimum has passed — which is rung 49's *a limiter acts through BOTH edges on DIFFERENT clocks*,
reaching a currency the limiter does own. Reported with the absolute integrals beside the ratios
(rung 71 § 4), and § 9 scores the miss.

---

## 6. CONCESSIONS (in addition to every one rungs 62–72 list, all inherited)

* **The reference is DECLARED, not derived** — as rung 72's composition law is. B is chosen
  because it is the only reading under which the leg reaches its own set point; that is an
  argument, not a derivation, and § 4 measures what C does instead. Everything structural in
  § 1 is contingent on **min-select AND reading B**.
* **AND THE THIRD READING IS REFUSED, NOT MEASURED AS A PLANT.** C is read at B's base points
  and never marched, so nothing here says what a 2×-droop fuel control would *do* — only what
  its Jacobian looks like where B's trajectory goes.
* **`∂required/∂mf ∈ {0,1}` IS A PROPERTY OF THIS LADDER'S SOLVERS**, not of limiters in
  general. Every cap here is a set-point solve, which is what collapses "applied-referenced"
  from a continuum to three readings and makes the hook one line. A leg whose cap depended on
  the fuel it was asked about would be a fourth plant.
* **THE `Tt4` NUMBERS IN § 5 ARE NOT A REDLINE-PROTECTION RESULT.** They are the difference
  between two model references at one imposed `Tt4_max` on one ramp; the **ordering** and the
  **sign** are the claims and every magnitude is disclaimed.
* **`Tt4_max = 1200 K` is RUNG 67's imposed value**, taken verbatim so the numbers difference
  against rungs 67/70/71/72 (rung 63's lesson). `φ_lim`, `b_max`, `v_max` remain imposed.
* **The DEEP-CELL clock arm is new** (§ 2.3) and is disclosed as a swept coordinate; the
  incidence/governor cell is unreachable at matched clocks and is quoted over 23 points pooled
  across the two wider arms.
* **The pole at the origin is REPORTED, never gated** (§ 1.3, § 4) — the count and the null
  direction are the gated quantities.
* **A `tie` is a set of measure zero** and every reader skips points within `4·dg` of the
  hand-over, reporting the count (zero, here, at `ds = 0.002`).
* The spectrum is sampled at finitely many trajectory points — a diagnostic that can miss a
  brief excursion (rung 65's retracted trap), not a proof.
* The STAGE STACK (rungs 55/56) is still off the transient ladder, and this still does **not**
  close rung 63's *fuel + bleed + STATOR* seam.

---

## 7. THE REDUCE — SIX ARMS, FIVE BY DISPATCH AND ONE BY THE DECLARED LAW

| arm | reduces to | measured |
|---|---|---|
| `_ref_law = "sched"` | **rung 72** | 341 pts, bit-for-bit |
| no fuel leg, incidence stator, governor | **rung 71** | 341 pts, bit-for-bit |
| no fuel leg, `φ` stator, governor | **rung 70** | 341 pts, bit-for-bit |
| fuel leg, incidence stator, `tau_gov=None` | **rung 69** | 341 pts, bit-for-bit |
| fuel leg, `φ` stator, `tau_gov=None` | **rung 68** | 341 pts, bit-for-bit |
| no stator, no fuel leg, governor | **rung 67** | 341 pts, bit-for-bit |

**THE FIVE INHERITED ARMS ARE AN IDENTITY HERE, NOT MERELY A DISPATCH.** With one fuel-side leg
armed the sole leg always holds authority, so `max(gf,gr) == g_own` everywhere and the applied
reference **is** the scheduled one — the reduce would hold even if the dispatch were removed.
Rung 71's *inherited identity* form, one rung on.

**AND THE SIXTH ARM IS GUARDED AGAINST BEING VACUOUS.** A `_reference` that ignored `_ref_law`
would pass it by comparing rung 73 with rung 73, so `test_the_scheduled_reduce_is_not_vacuous`
requires the *same two marches* under the applied reference to **differ**, in `Tt4` by more
than 1 K. That is not hypothetical — see § 8.

**A SIBLING INTEGRATOR WAS NOT REQUIRED**, and here rung 71's *reuse, do not copy* argument does
carry: no state is added, so rung 72's march is re-entered through a one-line hook. Rung 72
needed a sibling because it added a state; this rung needs none because it adds a law.

**THE SIX REDUCE TESTS ARE DELIBERATELY NOT MARKED `slow`** — rung 72 § 7's reasoning, verbatim,
and rungs 69/70/71/72's precedent. Every FINDING sweep (`handover_law`, `applied_gains`,
`applied_cells`, `ref_discriminator`, `applied_bill`) **is** marked; the refusals cost nothing.

**GATE TIMING, FINALLY RE-MEASURED — AND NOT ON PURPOSE.** Rung 72 left CLAUDE.md's `~3:18` /
`~1:18` flagged stale because its box was not in a steady state. The two whole-suite runs this
rung needed *anyway* (one after the code landed, one after the docs) came back at **5:31** and
**5:00** for **1182 tests**, and the fast subset at **1:54** for 882 — consistent readings, so
CLAUDE.md is updated from them. The 26 rung-73 tests are 49 s of that; the fast half, 8 s.
**The standing rule is that a gate run is never scheduled for a timing**: a quoted run time is
documentation, not a correctness signal, so it is taken from a run that was happening anyway or
left stale and disclosed. That rule is now in CLAUDE.md § Commands.

---

## 8. THE REFUSALS, AND THE INSTRUMENT THAT PRODUCED A PERFECT CONFIRMATION

Refused: an **applied reference on top of the SUM composition** (two declared laws at once —
under `sum` the hook never takes its identity branch, both fuel rows gain a cross term and the
block form goes; rung 63's lesson in its plainest form); an undeclared `_ref_law`; and every one
of rung 72's five, still armed through this rung's `integrate_fuel`. The RK4 floor is
**re-justified a SIXTH time on a genuinely new argument** — a zero eigenvalue is *neutrally
stable*, so "the dominant root is below the rate sum" is no longer the sentence; `λ = 0` is
interior to every explicit region and the other three share a trace `1/τ_masked` **more**
negative than rung 72's, so the inherited constant is *more* conservative here.

**AND `_reference`'s FIRST VERSION APPLIED READING B UNCONDITIONALLY.** `_with_ref('sched', ·)`
was then a **no-op**, and every A-vs-B reader differenced the plant against itself. It did not
fail. It returned

    worst_delta_rest = 0.0        mask_leak = 0.0        moved_scaled = [0.0]

— the first two being **exactly this rung's headline**, produced by an instrument that had
measured nothing. That is the **fifth** instance of this family's shipped-instrument-agrees-with-
itself pattern (rung 67 gate 9, rung 71 § 1.4's `c1`, rung 72 § 4's matched clocks and § 8's
`_charpoly4`), and the only defence that has ever worked is a gate that **fails when the two
laws are the same one**. So the bug is rebuilt in the test and fed to the gate
(`test_the_reference_dispatch_is_live`), which records that the broken reader still passes
`delta_rest == 0.0` and fails `moved_scaled == ±1` — **the live gate is the one that measures a
non-zero**, and the probe must also re-bless `at_lever`'s hard-coded class or it tests the
shipped one and passes.

---

## 9. THE ANCHOR, SCORED

| | prediction | verdict |
|---|---|---|
| P1 | `zeros` = 3/2 (`φ`) and 2/1 (`M_i`) — rung 72's counts each **+1** | **HELD** — one value per cell, all four cells |
| P2 | the coefficient identity with the pole at the origin, worst gap ≤ **1e−15** | **REFUTED AS STATED** — measured **5.3e−12** (independent half 1.1e−12). The identity holds; the tolerance was rung 72's, and § 1.3 is why it cannot be met |
| P3 | `c0 = 0`, `det J` dead in all four cells including rung 71's | **HELD** — worst `1.1e−14` normalised, against rung 72's live `+5.9e4` |
| P4 | the zero eigenvector lies **on** the masked axis, ≤ 1e−12 | **HELD** — worst null residual `5.3e−12`… and it is **the same number as P2's**, which § 3.1 declares rather than counts twice |
| P5 | C moves the AUTHORITATIVE row and not the pole | **HELD, and strengthened** — live diagonal exactly `−1.0` vs B's exactly `0.0`, pole kept to 2.7e−15; and C moves the **count** the other way (`−1`, never `+1`), which was not predicted |
| P6 | `Tt4` peak debit **> 10×** on both arms; the `φ` column unmoved to 4 s.f. | **HALF HELD, HALF REFUTED** — the debit is **110×/39×**, far past the floor; but the marginal `φ` credit **moves and changes sign on the `φ` arm**. `min φ_lp` is unmoved, so it is the integral, not the peak |
| P7 | `F_f`, `R_r` **exactly** 1 / 0 at masked / holding points | **HALF REFUTED** — the **zeros are exact** (`self_live`, `mask_leak`, `delta_rest` all `== 0.0`); the **ones are not** (3e−12). A difference quotient of a *sum* cannot return an exact one, and § 3.2 is the lesson |
| P8 | six reduce arms bit-for-bit | **HELD** — and the sixth is guarded against vacuity (§ 7) |
| P9 | hand-over LATE on every arm and clock; no hand-back | **HELD** — +0.010 … +0.055, one hand-over, 341/341 points |
| P10 | the refusals | **HELD**, and one more was needed (§ 0.3's IC floor) |
| — | *(already refuted with paper)* `n_live` reaches 4 | **CONFIRMED REFUTED** by measurement |

**THE THREE MISSES SHARE A ROOT CAUSE, AND IT IS DIFFERENT FROM RUNG 72's.** P2, P6's second
half and P7 all assumed that *exact structure survives being measured*. It does not, in two
distinct ways: a measured diagonal carries float cancellation (P2, P7), and a currency that
looks untouched at its **peak** can move in its **integral** (P6). Rung 72's misses came from
treating a run-time quantity as static; this rung's come from treating an instrument as
transparent. **§ 1.3's weakening of `_jac4` is what caused two of the three, and it was the
right trade** — the alternative was a headline the instrument would have written itself.

---

## 10. WHAT THIS DOES TO THE RUNGS BEFORE IT

* **RUNG 72's HEADLINE SURVIVES ITS OWN SHARPEST SEAM, AND ITS BOUND DOES NOT.** § 6 said to
  read *a shared actuator adds a switch between plants, not a loop* as holding *under min-select
  **with schedule-referenced legs***. The last clause is **REFUTED**: triangularity, the parent
  identification and `n_live = 3` are properties of **min-select alone**. Rung 72's own § 11
  expected the opposite, and it is the third rung running whose seam is closed by examining its
  premise rather than its conclusion — worth naming as a pattern, not a novelty.
* **RUNG 72 § 5's LEDGER IS CORRECTED BY TWO ORDERS OF MAGNITUDE** (§ 5), and the correction is
  structural, not numerical: a masked leg referenced to the schedule is credited with a cut
  another leg made. **The error exists only at rung 72** — with one fuel-side leg the two
  references coincide exactly — so rungs 46–71 are untouched.
* **RUNG 72's `zeros = n_live − m_live` GAINS A TERM.** `+ n_masked`, and it is the reference
  that decides whether the masked leg contributes a pole or a zero. Rung 71 bounded `n − m` to
  *gradients, not live loops*; rung 72 to *loops with authority, not states*; rung 73 shows the
  masked loops are not free either — **they contribute, and what they contribute is a choice of
  law.**
* **RUNG 71's FULL-RANK CELL IS NOT A PROPERTY OF THE PLANT.** Its `det J ≠ 0`, the only one in
  the family and the whole content of rung 71 § 1.3's factorisation, is killed by a change that
  adds no loop and no constraint.
* **RUNG 66's MIRROR ACQUIRES A THIRD CORNER.** Two loops on one *variable* are one loop with
  the rates ADDED (`pair = 1`); two on one *actuator* under a scheduled reference are one loop
  plus a **pole** (`pair_FR = 0`); under an applied reference, one loop plus an **integrator**.
  The composition law sets the first; the reference sets the third.
* **RUNG 49's *both edges, different clocks* reaches a new currency** (§ 5): a `φ` leg that
  debits `φ` itself, through a governor it never touches.

---

## 11. NEXT SEAMS

* **THE STATE-AS-DEMAND COORDINATE.** Every leg in this family lags its **clip**; a real fuel
  control lags the **demand** (`w = mf_sched − g`). On a *ramp* those are different plants —
  they differ by `ṁf_sched·τ`, so the two coordinates disagree by exactly the schedule's own
  slope. This rung deliberately did **not** fold that in (it is a coordinate change on the lag,
  not a reference change, and rung 63's lesson says one at a time). **It is the sharpest seam
  here**, and it is the last place `n_live = 4` could still hide.
* **A CAP THAT DEPENDS ON THE FUEL IT IS ASKED ABOUT** — § 6's concession. Then
  `∂required/∂mf ∉ {0,1}`, the three readings become a continuum, and the hook stops being one
  line.
* **READING C, MARCHED** — with the droop accepted and the currencies re-derived on it. § 4 reads
  its Jacobian only; what a 2×-droop limiter *does* is unmeasured.
* **THREE legs on one actuator, under the applied reference.** Rung 72 § 11 predicts two free
  poles; this rung predicts two poles **at the origin** and `zeros = n_live − m_live + 2`.
* **AN ASYMMETRIC valve** (rung 65) and an **asymmetric governor** (rung 67) — both still open.
* **Fuel + bleed + STATOR-as-a-SCHEDULE** — rung 63's seam, still open after 64–73.
* Everything rung 68 § 10 left: a plant with `|P| > 1`, and the real spatial/transported-CFD PDF.
