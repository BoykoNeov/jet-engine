# Rung 75 — THE DECLARED ANTI-WINDUP DEVICE

Rung 72/73/74's six states, four clocks, four loops and three actuators — **every one of them
unchanged**. The only thing added is a **stop with a rate** on the masked leg.

    none    dw/ds = ( target − w ) / τ                             — RUNG 74: nothing in its path
    track   dw/ds = ( target − w ) / τ  +  ( mf_app − w ) / τ_t    — back-calculation

Rung 74 § 10 named it: *§ 4 says the clip coordinate has an anti-windup device by accident. A
declared tracking/reset law on the demand plant would make `demand × applied` exist, and the
comparison would isolate what the accident was buying.*

> **HEADLINE — AN ANTI-WINDUP DEVICE IS DECISIVE ON THE SPECTRUM AND INERT ON THE RANK: THE
> EXACT INVERSE OF RUNG 74's COORDINATE.** The tracking term is **state-dependent**, so unlike
> rung 74's forcing it *is* in the Jacobian — it writes `−1/τ_t` onto the masked leg's own
> diagonal, **the one rung 73's applied reference had cancelled to exactly zero**. Measured
> through an instrument that differences the *derivative*: the masked diagonal is `−1/τ_t` at
> two clocks (residual `< 1e−9`, ratio **4.000000**), `det J` goes from `< 1e−9` to `> 1`,
> `zeros` from **1 to 0**, and `|det J|` scales by the **same 4.000000**. **The masked pole
> leaves the ORIGIN and `det J`, dead since rung 73, revives.**
>
> And `n_live` does not move. `mask_leak = 0.0` **exactly**, as under rung 74 — the term sits in
> the masked leg's **ROW** (it reads the authoritative leg through `mf_app`) while the masked
> **COLUMN** stays zero, because `min()` is still flat in what the masked leg holds. **`n_live`
> is still ≤ 3, the FOURTH running.** Rung 74 moved the bill and not the spectrum; this moves
> the spectrum and not the rank.

> **AND THE DEVICE DISARMS ITSELF ON THE LEG THAT HOLDS THE ACTUATOR.** `mf_app = min(mf_sched,
> w_f, w_r)` equals `w_auth` **in a neighbourhood**, so `(mf_app − w_auth)/τ_t` is the zero
> *function* — not small, zero. Measured `track_leak = 0.0` exactly and the authoritative
> diagonal unmoved to **0.0 relative**. Rung 72's *ONE plant IS rungs 68/69/70/71 by AUTHORITY*
> is untouched: everything this rung does, it does to the **masked** leg.

> **AND RUNG 74's `2.898e−3` WAS NOT A SOLVER FAILING TO FIND A PLANT — IT WAS A CONTRACTION
> WITH RATIO EXACTLY ONE.** The joint IC sweep is a fixed-point iteration whose map has slope
> `σ = τ_t/(τ + τ_t)`, so it converges in `ceil(ln(tol/res₀)/ln σ)` iterations — with **rung
> 74's own reported residual** as `res₀` and the inherited tolerance, **zero fitted constants**.
> Measured **185 / 98 / 54 / 32 against 185 / 98 / 54 / 32**. Rung 74's verdict stands
> (`τ_t → ∞ ⇒ w* → ∞`, no finite equilibrium); its *number* is explained, and the
> `exists / does not exist` boundary is the 60-iteration cap cutting a geometric sequence.

Pre-registration: `docs/plans/rung75-anchor-antiwindup.md`, whose § 0 is larger than rung 74's
and says why. Gates: `tests/test_rung75.py`.

---

## 0. WHAT MOVED, AND WHAT DID NOT

| | rung 73 | rung 74 | **rung 75** |
|---|---|---|---|
| the loops | four | the same four | **the same four** |
| states / actuators | 6 / 3 | 6 / 3 | **6 / 3** |
| what is added | a REFERENCE | a COORDINATE | **a STOP WITH A RATE** |
| the masked leg's pole | the ORIGIN | whichever the reference sets | **`−1/τ_t` — off the origin** |
| `det J` | NOWHERE | unmoved (dead) | **REVIVED** (`applied` only) |
| `zeros` | `n − m` | unmoved | **loses `n_masked`** |
| `n_live` | ≤ 3 | ≤ 3 | **≤ 3** |
| the bill | +9 … +71 K | −81 … −283 K | **−160 K against the accident** |

**No state, no loop, no actuator and no reference is added.** One constant is: `τ_t`, the first
new clock since rung 65, and it has no derivation from anything shipped (§ 6).

### 0.1 The fourth declared knob

`_windup_law` joins `_share_law` (72), `_ref_law` (73) and `_lag_coord` (74):

| `_windup_law` | the masked leg's stop | is |
|---|---|---|
| `none` | none at all | **RUNG 74**, by the branch not being taken |
| `track` | back-calculation onto `mf_app`, clock `τ_t` | **THIS RUNG** |

**`clip × track` and `demand-latched × track` are REFUSED, by assert and by name.** Rung 52's
`max(0,·)` is still in the clip coordinate and the latch is still in `demand-latched`, so either
cell would run **two** anti-windup devices at once and attribute the result to this one — rung
63's *change one law at a time*, which rung 74 § 2 records itself breaking in a `for` loop. The
refusal is checked at `integrate_fuel`, not only in the hook, because **`clip` dispatches out of
this ladder before any hook is read** and would otherwise have marched rung 73 silently.

### 0.2 The park law — where a masked leg comes to rest

Setting `dw/ds = 0` for a masked leg under the **applied** reference, where
`target = w + cap − mf_app` so the leg's own `w` cancels from the first term:

    w* = mf_app + ( τ_t / τ ) · ( cap − mf_app )                       [applied]
    w* = ( τ_t·cap + τ·mf_app ) / ( τ + τ_t )                          [sched]

An offset **above** the applied fuel, linear in the clock ratio and in the leg's own slack.
`τ_t → 0` parks it at `mf_app` exactly — textbook perfect tracking, a leg with nothing to
unwind; `τ_t → ∞` recovers rung 74's divergence. **The two references give different park laws,
which is what makes the windup × reference 2×2 non-degenerate** — and § 1.3 measures both faces.

---

## 1. THE INSTRUMENT HAD TO BE REBUILT, AND THAT IS THIS RUNG'S ONE IMPLEMENTATION COST

### 1.1 Every inherited gains reader is BLIND to this rung's subject

`_demand_laws` / `_quad_laws` return each leg's **target**; `_demand_gains_at` central-differences
those; `_jac4` assembles `J[i][j] = (∂cmd_i/∂x_j − δ_ij)/τ_i` from the result plus `taus`. **The
tracking term is in neither.** It is not part of any leg's target, and `τ_t` is not in `taus`.

So `demand_gains` run on the `track` cell would have reported the masked diagonal unchanged,
`det J` still dead and the spectrum invariant — **a perfect refutation of this rung's headline,
having measured nothing.** That is rung 73's `_reference` no-op with the sign flipped, and it
would have passed every gate one would think to write. It was found *before* any Jacobian was
read and is anchor § 0.6, a numbered section rather than a footnote.

`_rhs_laws` / `_rhs_gains_at` difference the **derivative**, so all sixteen entries including
every diagonal are measured end to end. **`τ_t` is deliberately not added to `taus`**: letting
`_jac4` write `−1/τ_t` onto the diagonal would be the **seventh** instance of the
shipped-instrument-agrees-with-itself pattern (rung 67 gate 9, rung 71 § 1.4, rung 72 §§ 4 and
8, rung 73's `_reference`, rung 74 § 1.1). Rung 73 *weakened* `_jac4` to measure two diagonals;
this finishes that move for all four.

`τ_f` is frozen at the base point rather than recomputed per perturbation — rung 72's own
convention, because rung 52's lag is a **step** in the attack/release direction and
central-differencing across it would measure the step.

### 1.2 The states are the CLIP plant's at the INHERITED floor, and that is a disclosure

Rung 74 § 1.3, inherited word for word and for its own reason: `_shared_rig` gives every leg
**one** margin, so at the lowered floor § 4's arms need, the valve is off-regime at every point
and `_riding4` returns **nothing** — measured, **0 of 0**, by the first reader written here,
which is why the paragraph exists. A Jacobian is a function of the **state**, and every claim in
this section is a **difference between two laws at one state**, so the choice of trajectory
cannot manufacture one.

### 1.3 Measured — the whole matrix, at two clocks and two references

`φ_lim = 0.80` (the inherited floor), clocks all `0.05`, `ds = 0.005`, 7 (`applied`) / 9
(`sched`) interior points:

| | `applied`, `none` | `applied`, `track` | `sched`, `none` | `sched`, `track` |
|---|---|---|---|---|
| masked diagonal | **0.0** (rung 73's origin) | **`−1/τ_t`** | `−1/τ` | **`−(1/τ + 1/τ_t)`** |
| … at `τ_t = 0.05` | 0.0 | −20.000 | −20.0 | −40.000 |
| … at `τ_t = 0.0125` | 0.0 | −80.000 | −20.0 | −100.000 |
| authoritative diagonal | −20.0 | **−20.0, moved 0.0** | −20.0 | **−20.0, moved 0.0** |
| `track_leak` | — | **0.0 exactly** | — | **0.0 exactly** |
| `mask_leak` | **0.0 exactly** | **0.0 exactly** | **0.0 exactly** | **0.0 exactly** |
| `det J` | `< 1.3e−13` | **−574.7 … 4.59** | −453.7 … 6.93 | −907.5 … 13.86 |
| `zeros` | **1** | **0** | 0 | 0 |
| ratio (0.05 → 0.0125) | — | **4.000000** on diagonal *and* `det J` | — | **2.500000** on both |

**AND BOTH STATOR ARMS CARRY IT.** The table above is rung 68's `StatorLimiter` (7 / 9 interior
points); rung 69's **INCIDENCE** stator gives the same entries to the same tolerances on 3 / 4
points — `−20 / −80` against `0.0`, `−40 / −100` against `−20`, `zeros` `1 → 0` against `0 → 0`,
both leaks exactly `0.0`, and **both ratios exact**. The device acts on the two *fuel-side* legs,
whose laws never mention the stator's coordinate, and that is now measured rather than argued.

**THE TWO RATIOS ARE THE SAME STATEMENT.** `det J` = masked diagonal × `det`(live 3×3), and the
live block is rung 71's, unmoved — so `det J` must follow the diagonal exactly, and it does, in
*both* references: `4.000000` where the diagonal is `−1/τ_t`, `2.500000` where it is the two
rates **added** (`100/40`). Rung 66's *two loops on one variable are one loop with the rates
added* in a fifth shape, now on a device rather than a loop.

**THE REVIVAL IS `applied`-ONLY, and that is one mechanism with two faces rather than two
findings.** Under `sched` the masked target is `cap`, which contains no `w`, so the diagonal was
already `−1/τ` and nothing was ever dead there.

### 1.4 The masked row's coupling, and where it vanishes

`∂RHS_masked/∂w_auth = 1/τ_t − 1/τ_masked` under `applied` (measured to `< 1e−9` at every point
and both authority cells, with `τ_masked` being `τ_gov = 0.05` or the lag's own `τ_rel = 0.15`),
against rung 74's `−1/τ_masked`. So the entry **changes sign across `τ_t = τ_masked` and is
exactly `0.0` there**: at matched clocks the masked leg stops reading the authoritative one
altogether. Under `sched` it is `+1/τ_t` where rung 74 measured exactly `0.0` — the device
*creates* a coupling in one reference and *cancels* one in the other.

---

## 2. RUNG 74's RESIDUAL, EXPLAINED — AND ITS VERDICT UNTOUCHED

The joint IC sweep is a fixed-point iteration. The device changes its map's slope in the leg's
own state from `1` to `σ = τ_t/(τ + τ_t)`, so the residual falls **geometrically** and the sweep
converges in `ceil(ln(tol/res₀)/ln σ)` iterations, with `res₀` **rung 74 § 4's own reported
residual** `2.898e−3` and `tol` the inherited `1e−12`:

| `τ_t` | `σ` | predicted | measured |
|---|---|---|---|
| 0.4 | 0.888889 | **185** | **185** |
| 0.2 | 0.800000 | **98** | **98** |
| 0.1 | 0.666667 | **54** | **54** |
| 0.05 | 0.500000 | **32** | **32** |

**Four of four, exactly, with no fitted constant.** So:

* **rung 74's verdict stands.** `τ_t → ∞ ⇒ σ = 1 ⇒` no contraction at all, and the park law
  gives `w* → ∞`: there is genuinely no finite equilibrium, which is what rung 74 measured.
* **its number is explained.** `2.898e−3` never moved because a contraction with ratio one has
  nowhere to move it.
* **the `exists / does not exist` boundary this rung's own first probe showed at `τ_t/τ ≈ 2–4`
  is the 60-ITERATION CAP cutting a geometric sequence** — a solver boundary, not a plant
  property. The park law gives a finite equilibrium at **every finite `τ_t`**.

The cap is raised **in a reader and nowhere else** (`_ic_cap`, default `60` on every plant in
this family, gated). Raising it in a plant would make § 2's boundary this rung's choice rather
than the inherited solver's.

---

## 3. THE ACCIDENT AND THE DEVICE — where this rung's own prediction died

Anchor **P8** predicted the two coincide where no leg is cutting: there `mf_app = mf_sched` and
the tracker pulls to exactly where the latch clamps. **REFUTED ON THE STATE, HELD EXACTLY ON THE
OUTPUT** — and the refutation is the park law the same anchor derived two lines earlier. The
tracking term pulls toward `mf_app`, but the **target** term still pushes toward `cap`, and
`cap > mf_sched` (rung 74 § 0.2 measures `1.303×` at `s = 0`), so the balance sits **above** the
schedule while the latch clamps **at** it.

| | dormant points | cutting points |
|---|---|---|
| output (`mf`, `Tt4`, `ν_lp`) | **0.0, exactly** | 164.6 K |
| state (`w_f`, `w_r`) | **2.898e−3** (`τ_t = 0.05`) / **7.246e−4** (`0.0125`) | — |

**The state gap is the park law**: quartering the clock quarters it, `ratio = 4.000000`. And the
dormant-point gap at `τ_t = τ` is `2.898e−3` — **rung 74's own residual again**, because it is
the same quantity: the offset a masked leg would have to travel to reach the stop.

So the honest statement is a **distinction, not an equality**: *while nothing is cutting the two
devices burn identically and their states never agree at all.* That the prediction was written
on the **state** while the equality lives on the **output** is rung 74 P6's law/trajectory
confusion in a **third** shape (rung 58's *check the SUM, not the term*), and it is scored
REFUTED rather than rewritten.

---

## 4. THE BILL — a threshold on the one constant this rung adds

`φ_lim = 0.76`, `applied`, everything else inherited:

| `τ_t` | `τ_t/τ_f` | `max Tt4` | over `Tt4_max` | `min φ_lp` | hand-over |
|---|---|---|---|---|---|
| 0.00625 | 0.125 | 1197.41 | **−2.59** | 0.76590 | 0.695 |
| 0.0125 | 0.25 | 1197.42 | **−2.58** | 0.76590 | 0.695 |
| 0.025 | 0.5 | 1197.45 | **−2.55** | 0.76587 | 0.695 |
| 0.05 | 1.0 | 1197.50 | **−2.50** | 0.76575 | 0.700 |
| 0.0625 | 1.25 | 1198.18 | **−1.82** | 0.76564 | 0.700 |
| 0.075 | 1.5 | 1202.51 | **+2.51** | 0.76545 | 0.700 |
| 0.0875 | 1.75 | 1206.65 | **+6.65** | 0.76511 | 0.705 |
| 0.1 | 2.0 | 1210.49 | **+10.49** | 0.76437 | 0.705 |
| **the ACCIDENT** | — | **1359.88** | **+159.88** | 0.76591 | **1.065** |

**RUNG 47's HEADLINE CONCESSION, THIRD LAYER.** Rung 47: *the cost of realism is that a lagged
governor breaks the redline hold.* Rung 74: that is a property of the **coordinate**, not the
lag. Rung 75: **within the demand coordinate it is a THRESHOLD ON `τ_t`** — bracketed at
`τ_t/τ_f ∈ (1.25, 1.5]`, i.e. *a tracking clock no slower than its own leg's*. Rung 54's shape
(*every verdict is a threshold ON the disclosed constant*), now on a clock rather than a level.

**AND THE COMPARISON THAT CARRIES THE MAGNITUDE IS AGAINST THE ACCIDENT, NOT ACROSS THE SWEEP.**
`τ_t` moves the peak by 13 K and the hand-over by two grid cells; the declared device beats the
inherited stop by **160 K** and hands the actuator over **0.36 earlier**. The whole sweep lives
inside a tenth of the credit the device itself delivers.

**P9's mechanism, registered in advance with both branches named.** The hand-over is **monotone
increasing** in `τ_t` with the fast end earliest — the *windup-dominant* branch (a) — so a
less wound-up leg does take over sooner, and rung 74 § 2.1's *a hand-over is set by when the
other leg's constraint arrives* does not generalise to a device that changes how much unwinding
the arriving leg has to do. The span is **two grid cells**, and that is stated as a bound, not
polished away.

---

## 5. WHAT THIS DOES TO THE RUNGS BEFORE IT

* **RUNG 74 § 4's RESIDUAL IS CORRECTED IN ITS READING, NOT ITS VERDICT** (§ 2). *No interior
  equilibrium* is right; *the sweep failed* is not — it was a contraction at `σ = 1`, and the
  four-of-four derived iteration counts are the proof.
* **RUNG 73's ORIGIN POLE IS SHOWN TO BE REMOVABLE WITHOUT TOUCHING THE RANK** (§ 1.3). Rung 73
  killed `det J` by cancelling a diagonal; a device that writes that diagonal back revives it
  and buys **no** live loop. So *`zeros` counts gradients, not live loops* (rung 71) gains its
  converse: **a pole is not a loop either.**
* **RUNG 72's `n_live ≤ 3` SURVIVES A FOURTH ATTACK**, and by the same mechanism each time:
  *min-select is flat in whatever the masked leg holds* — which no term added to that leg's own
  **row** can change.
* **RUNG 52's `max(0,·)` IS NOW COMPARED, not just named.** Rung 74 named it an anti-windup
  device; § 3/§ 4 measure what it was buying — the right *output* while nothing is cutting, and
  **160 K of redline worse** than a declared device once something is.
* **RUNG 47's CONCESSION IS A THRESHOLD ON A CLOCK** (§ 4), which is its third and narrowest
  reading.
* **RUNG 66's RATE-ADDITION IDENTITY EXTENDS FROM LOOPS TO DEVICES** (§ 1.3): under `sched` the
  device and the leg's own lag appear on one diagonal as `1/τ + 1/τ_t`, and `det J` follows.

---

## 6. CONCESSIONS (in addition to every one rungs 62–74 list, all inherited)

* **`τ_t` IS A NEW CONSTANT AND IS NOT DERIVED.** The first new clock since rung 65. Every
  finding is stated as a property of the sweep or as a threshold on it; no verdict here is
  quoted at a single value.
* **Its fast end is GRID-LIMITED.** The device adds `1/τ_t` to each of two fuel-side diagonals,
  so `_rk4_floor_shared` admits `τ_t ≥ 2·ds/(2 − ds·Σ(1/τ_i)) = 0.00625` at the inherited grid
  (gated, with the assert measured at `0.005`). **Perfect tracking is not reachable here and is
  not claimed**, and the constant is not loosened to reach it (rung 65's lesson).
* **The `clip` coordinate is untested with this device**, by refusal (§ 0.1) — so *what the
  accident was buying* is measured only against the demand coordinate's own latch, never
  against rung 52's stop in the coordinate that actually carries it.
* **The control row is THIN.** Only **one** dormant point exists on the anchor trajectory; the
  accel arms almost immediately. § 3's `0.0` is exact but it is one point.
* **The hand-over's `τ_t`-dependence is two grid cells**, at the resolution limit (§ 4). The
  monotonicity never inverts across five clocks, but the magnitude statement is against the
  accident and not across the sweep.
* **Every Jacobian is read at the INHERITED floor and every trajectory at the lowered one**
  (§ 1.2), so no single cell carries both — rung 74's split, inherited.
* The **BILL and the CONTRACTION** are read on the `inc = False` stator arm only; § 1.3's
  Jacobian carries both (it was going to be conceded here, and measuring it was cheaper than
  writing the concession).
* **Nothing here re-derives `cap` for a fuel-dependent constraint** — rung 73 § 11's second seam
  is untouched by a third rung running.

---

## 7. THE REDUCE — TWO ARMS, BOTH BY DISPATCH, AND THAT IS STRONGER THAN RUNG 74's

1. **`_windup_law = 'none'` ⇒ rung 74, BIT-FOR-BIT, on all three of its live cells**
   (`clip×applied`, `demand×sched`, `demand-latched×applied`). The hook's branch is simply not
   taken — **not a tolerance**, which rung 74's own second arm had to be (`1.59e−12`, two float
   expressions for one quantity). It is available here only because this rung **reuses its
   parent's march** rather than siring one: rung 71's form, four rungs on.
2. **The refusals are refusals** — `clip × track`, `demand-latched × track`, an undeclared law
   and an undeclared `τ_t` all assert by name.

Gated non-vacuous on rung 73's `charpoly_selftest` discipline: **the same machine under `track`
must differ**, and the cell rung 74 has no plant for must march.

---

## 9. THE ANCHOR, SCORED

| | claim | verdict |
|---|---|---|
| D1 | the park law | **HELD** — § 3's `4.000000` state-gap ratio measures it |
| D2 | masked diagonal `−1/τ_t` (`applied`) / `−(1/τ+1/τ_t)` (`sched`) | **HELD**, both, `< 1e−9` |
| D3 | the device is the zero *function* on the leg that holds | **HELD** — `track_leak = 0.0` exactly |
| D4 | the masked column untouched ⇒ `n_live ≤ 3` | **HELD** — `mask_leak = 0.0` exactly |
| D5 | masked-row coupling `1/τ_t − 1/τ_masked` | **HELD** — `< 1e−9`, both authority cells |
| D6 | the contraction | **HELD** — § 2, and it was derivation *after* measurement, so it sits in § 0.3 |
| P1 | masked diagonal `−1/τ_t` at two clocks, ratio 4.000 | **HELD** — `4.000000` |
| P2 | authoritative diagonal unmoved, `track_leak = 0` | **HELD** — moved `0.0` relative, leak `0.0` |
| P3 | `mask_leak = 0.0` exactly under `track` | **HELD** |
| P4 | `det J` revives, `zeros` −1, `|det J|` ∝ `1/τ_t` | **HELD** — `<1.3e−13` → `>1`, `1 → 0`, ratio `4.000000` |
| P5 | under `sched` both alive, no `zeros` drop | **HELD** — and the ratio is `2.500000`, which the block-triangular law predicts and P5 did not state |
| P6 | the coupling changes sign, zero at `τ_t = τ_masked` | **HELD** — exactly `0.0` there |
| P7 | 98 and 185 iterations, derived | **HELD** — 4/4 exact, including both |
| P8 | dormant ⇒ `track ≡ latch` | **REFUTED** — exact on the OUTPUT, never on the STATE (§ 3) |
| P9 | hand-over monotone, branch (a) windup-dominant | **HELD** — monotone increasing, fast end earliest; span two grid cells |
| P10 | a redline threshold on `τ_t`, within 2× of `τ_f` | **HELD** — bracketed at `τ_t/τ_f ∈ (1.25, 1.5]` |
| P11 | `none` reproduces rung 74 bit-for-bit | **HELD**, gated on three cells |
| P12 | and it is not vacuous | **HELD**, gated |

**One refutation from twelve, and it became § 3** — a distinction (output vs state) sharper than
the equality it replaced.

**AND ONE FINDING WAS NOT PREDICTED AT ALL:** that `det J` and the masked diagonal scale by the
**same** ratio in **both** references (`4.000000` / `2.500000`), which is block-triangularity
measured rather than assumed, and is the strongest single number in § 1.3.

**AND ONE TRAP BIT AGAIN DURING THE BUILD** (§ 7's gate now watches it): `_ic_cap` was set on
the outer rig, `_shared_rig` returned a fresh machine without it, and § 2's two slowest arms
reported ASSERT instead of 185 and 98 — the **thirteenth** instance of the trap rungs 61–74 each
hit, and the first one where the missing knob was one this rung had added minutes earlier.

---

## 10. NEXT SEAMS

* **THE DEVICE ON THE VALVE AND THE STATOR.** Both hold a position against a `max`, so both have
  rung 52's stop and neither has a declared rate. Rung 74 § 10's first seam, now with a device
  to put there.
* **`clip × track`, VIA A COORDINATE THAT HAS NO STOP TO BEGIN WITH.** The refusal in § 0.1 is
  about running two devices at once, not about the question — a clip coordinate with `max(0,·)`
  *removed* and `track` added would measure the accident against the device in the coordinate
  that actually carries it.
* **AN ASYMMETRIC `τ_t`** (fast reset, slow release), which is rung 52's own asymmetry pointed
  at the anti-windup rate rather than the lag. § 1.4 says the coupling changes sign across
  `τ_t = τ_masked`; an asymmetric device would sit on both sides of that within one march.
* **A FUEL-DEPENDENT CAP** — rung 73 § 11's second seam, untouched by three rungs running, and
  the only one left that could move `∂required/∂mf ∉ {0,1}`.
* Everything rungs 72/73/74 § 10/11 leave: three legs on one actuator, fuel + bleed + stator,
  and the real spatial/transported-CFD PDF.
