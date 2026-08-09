# Rung 76 — THE FUEL-DEPENDENT CAP

Rung 72/73/74/75's six states, four clocks, four loops and three actuators — **every one of
them unchanged**, and this is the first rung since 71 that adds **no constant at all**. The only
thing added is a second *reading* of a cap this family has read one way since rung 48.

    solve    cap = w*  with  w* = (1+margin)·κ(n_H(w*))·pt3(w*)      — RUNG 48's set-point solve
    sensed   cap(w) = (1+margin)·κ(n_H(w))·pt3(w)   at  w = mf_app   — THE SCHEDULE AS WRITTEN

Rung 73 § 11 named it, and rungs 73, 74 **and** 75 each deferred it: *A CAP THAT DEPENDS ON THE
FUEL IT IS ASKED ABOUT — then `∂required/∂mf ∉ {0,1}`, the three readings become a continuum,
and the hook stops being one line.*

> **HEADLINE — A DEVICE IN A LEG'S LAW REACHES ONLY THE MASKED LEG; A DEVICE IN THE PLANT THE
> LEGS READ REACHES ONLY THE AUTHORITATIVE ONE.** Rung 75's back-calculation sits in a leg's own
> law, and min-select **masks a law** — so everything it did, it did to the masked leg, and the
> authoritative diagonal came back *moved 0.0 relative*, as it had for rungs 73 and 74 too. A
> sensed cap is in **no leg's law**: it is in the plant both legs read through `mf_app`, and
> min-select **cannot mask a plant**, because the plant is shared. So it writes `c/τ_f` on the
> **authoritative** fuel diagonal — the one entry three consecutive rungs each measured as
> unmoved — and leaves the masked one alone. Measured: `−20.0000 → −16.25 … −16.42`, i.e.
> **moved 0.1874 relative**, fitting `(c−1)/τ_f` to `1.1e−9` at every point, **in both
> references and on both stator arms**; masked diagonal **moved 0.0 exactly**.
>
> And `n_live` does not move. `mask_leak = 0.0` **exactly**, as under rungs 73/74/75 — the
> masked **column** stays zero because `min()` is flat in what the masked leg holds. **`n_live`
> is still ≤ 3, the FIFTH running** — and the two rungs together say why: the obstruction is
> neither a law nor a plant but the **composition**.

> **AND `zeros` DOES NOT MOVE EITHER, WHICH IS THE EXACT INVERSE OF RUNG 75.** Rung 75 moved the
> spectrum (`zeros` `1 → 0`, `det J` revived) and not the bill's rank; this moves `det J`'s
> **value** by `1−c` and its **count** not at all. Rung 74 moved the bill and not the spectrum,
> rung 75 the spectrum and not the rank, rung 76 the authoritative row and not the count.

> **AND THE SET-POINT SOLVE WAS NEVER A RELOCATION OF THE CAP — IT IS A GAIN ON IT.** The
> unpredicted finding (§ 3): differentiating the fixed point `cap = cap_sensed(cap, q)` gives
> `d(cap_solve)/dq = (d(cap_sensed)/dq)/(1−c)`, so **writing a limiter as a solve multiplies its
> sensitivity to every other state by `1/(1−c)`**. Measured `1.228 … 1.246` against `1/(1−c)` at
> `< 1e−8`, both stator arms. A limiter written as a solve is a **stiffer** limiter than the
> schedule it claims to implement, and nothing in rungs 48–75 could see it, because none of them
> had a second reading of the same cap to difference against.

Pre-registration: `docs/plans/rung76-anchor-sensed-cap.md`. Gates: `tests/test_rung76.py`.

---

## 0. WHAT MOVED, AND WHAT DID NOT

| | rung 74 | rung 75 | **rung 76** |
|---|---|---|---|
| the loops | four | the same four | **the same four** |
| states / actuators | 6 / 3 | 6 / 3 | **6 / 3** |
| what is added | a COORDINATE | a STOP WITH A RATE | **a second READING of a cap** |
| new constants | 0 | `τ_t` | **0** |
| where it lives | a leg's forcing | a leg's LAW | **the PLANT both legs read** |
| the leg it reaches | masked | masked | **AUTHORITATIVE** |
| masked diagonal | whichever the reference sets | `−1/τ_t` | **unmoved, 0.0 exactly** |
| authoritative diagonal | moved 0.0 | moved 0.0 | **moved 0.1874** |
| `det J` | dead | REVIVED | **scaled by `1−c`** |
| `zeros` | unmoved | `1 → 0` | **unmoved** |
| `n_live` | ≤ 3 | ≤ 3 | **≤ 3** |

### 0.1 The fifth declared knob, and its DOMAIN is declared with it

`_cap_law` joins `_share_law` (72), `_ref_law` (73), `_lag_coord` (74) and `_windup_law` (75):

| `_cap_law` | the accel leg's cap is | is |
|---|---|---|
| `solve` | rung 48's implicit fixed point | **RUNG 75**, by the branch not being taken |
| `sensed` | the schedule evaluated at `mf_app` | **THIS RUNG** |

**It reaches the ACCEL branch of `_cap_fuel` and nothing else, and that is stated here rather
than conceded in § 6.** `_cap_gov` and the φ branch are floors on **states**, and a floor on a
state is not a formula for a fuel — they have no sensed form in any rung, so the governor's row
is bit-identical in every cell (measured `0.0e+00`).

**`clip × sensed` is REFUSED, by assert and by name, at `integrate_fuel`** — `clip` dispatches
out of this ladder before `_cap_fuel` is ever called, so such a march would silently be rung 73
and be reported as this rung. Rung 75 § 0.1's refusal, one knob over.

### 0.2 The accel leg had never been armed here, so nothing is differenced against a quoted number

Rungs 72–75 arm the **φ** leg and never `accel`. The accel leg is a supported argument of
`integrate_fuel`/`_stator_march` all the way down, so this is a plant the ladder has always had
and never marched — but it means rung 73 § 5's ledger, rung 74's bill and rung 75's `−160 K` and
§ 1.3 table are all at `accel=None` and are **not comparable**. Rung 63's lesson. **The whole
`windup × cap_law` 2×2 is therefore re-measured here on one rig, and rung 75's headline is
reproduced rather than cited** — which turns the plant change from a concession into evidence.

**And the joint IC sweep does not notice.** `demand × applied × none` fails with residual
`2.898e−03` on the accel-armed plant — rung 74's own number to four figures — so rung 75 § 2's
derivation of it carries here untouched.

### 0.3 `c` is MEASURED, and it is the one number the rung rests on

    c(w) ≡ ∂cap_sensed/∂w = (1+margin)·d[κ(n_H(w))·pt3(w)]/dw

| | value |
|---|---|
| `c` over the Jacobian base points, `margin = 0.10` | **0.1790 … 0.1875** |
| over five margins `0.05 … 0.80` | **0.174 … 0.306**, positive everywhere |

`c > 0` because more fuel at fixed spool speeds is a hotter `Tt4`, less choked-NGV corrected
capacity and therefore a **higher** `π_c` — rung 49's own docstring chain, read one station
further. **`c < 1` is a measurement and is not implied by the shipped solver working**: a
bracketing root-finder converges on a sign change whether or not `G = w − cap(w)` is monotone,
so `_sched_fuel` bracketing buys *a root exists*, never `G′ > 0`.

---

## 1. MEASURED — THE WHOLE MATRIX, TWO REFERENCES × TWO DEVICES

`φ_lim = 0.80` (the inherited Jacobian floor), `margin = 0.10`, all clocks `0.05`, `ds = 0.005`,
read through `_rhs_laws` and never `_jac4` (rung 75 § 1.1's reason survives one knob over and
gains a second: a sensed cap is not part of any leg's **target** either, so a target-differencing
reader would report both fuel diagonals at `−1/τ_i` by construction and refute this rung having
measured nothing).

| | `sched`, `solve` | `sched`, `sensed` | `applied`, `solve` | `applied`, `sensed` |
|---|---|---|---|---|
| **authoritative** diagonal | `−20.0000` | **`−16.42 … −16.25`** | `−20.0000` | **`−16.35 … −16.25`** |
| … moved, relative | — | **0.1874** | — | **0.1875** |
| … against `(c−1)/τ_f` | — | **`1.05e−9`** | — | **`1.11e−9`** |
| masked diagonal | `−20.0` / `−40.0` | **moved `0.0` exactly** | `0.0` / `−20.0` | **moved `0.0` exactly** |
| masked row's cross | `0.0` / `20.0` | **unmoved** | `−20.0` / `0.0` | **unmoved** |
| `mask_leak` | `0.0` exactly | **`0.0` exactly** | `0.0` exactly | **`0.0` exactly** |
| governor's whole row | — | **`0.0e+00`** | — | **`0.0e+00`** |
| `zeros` | 0 … 1 | **unmoved** | 1 (`none`) / 0 | **unmoved** |
| `det J` ratio | — | **0.8152 … 0.8222** | — | **0.8151 … 0.8197** |

**AND BOTH STATOR ARMS CARRY IT.** The table is rung 68's `StatorLimiter`; rung 69's
**INCIDENCE** stator gives `−20.0000 → −16.3563`, moved `0.1822`, residual `6.48e−10`,
`det` ratio `0.81712`, `mask_leak` and `gov_row` both exactly `0.0`. The knob acts on a cap whose
law never mentions the stator's coordinate, and that is now measured rather than argued.

### 1.1 Why the authoritative leg and only it

`mf_app = min(mf_sched, w_f, w_r)`. Where the fuel leg holds, `∂mf_app/∂w_f = 1`, so
`∂cap/∂w_f = c` and

    ∂RHS_f/∂w_f = ( c − 1 ) / τ_f          — in BOTH references

identically, because `_demand_reference` returns `cap` **itself** when `mf_app == w_own`: the
applied reference is the identity on the leg that holds (rung 73 § 0.2), so it cannot change what
a plant-side gain does there. Where the leg is masked, `∂mf_app/∂w_masked = 0` — `min` is flat in
what the masked leg holds — so `∂cap/∂w_masked = 0` and the masked diagonal is untouched. **The
same flatness that gives rungs 72–76 their triangularity is what confines this rung's device to
the authoritative leg.**

### 1.2 The `det J` ratio is `1 − c` to 0.7 %, and the residual is not noise

The masked column is zero, so `det J` = masked diagonal × `det`(live 3×3) — rung 75 § 1.3's own
factorisation. Only the authoritative fuel row moves, and § 3 shows the **whole** row scales by
`1−c` when both laws are read at the same `w`. They are not: the sensed cap is read at `mf_app`
and the solve's fixed point sits at `cap_solve`, and `c` varies between them. Measured ratio
`0.8152 … 0.8222` against `1−c ∈ 0.8126 … 0.8210`, residual `7.25e−3` (`6.96e−4` on the incidence
arm, `5.7e−4 … 5.5e−2` across the robustness sweep). **Anchor P7 asked for `< 1e−9` and is
scored REFUTED**; § 3 is the correction, and it is a sharper statement than the identity it
replaces.

`applied × solve` is excluded, as the anchor excluded it: `det J ≡ 0` there (rung 73's dead
determinant, measured `< 8.4e−9`) and the ratio is `0/0`.

### 1.3 The knob is INERT where the leg it re-reads does not set the cap

`_cap_fuel` is `min(accel, φ)` — **min-select one level down**, rung 74's own phrase — and the φ
leg has no sensed form. Wherever the φ cap is the lower one the knob is inert **by
construction**, and a reader that pooled those points would report a real law as broken. It did:
two cells of the first sweep came back at `1.9e−01` where every other cell was at `1e−9`, and
they were points where the two cap laws sit on **opposite sides of the cap's own switch** — one
accel-bound, one φ-bound. Differencing those measures a **leg change** and reports it as a law
that failed. The fix is rung 72's `switch_guard` one min-select level down: a point counts only
where the accel leg binds under **both** laws. With that guard, `1.3e−9` is the worst residual
in **18** (`φ_lim`, `Tt4_max`, `margin`, stator-arm) combinations.

### 1.4 The masked-leg cell is STRUCTURALLY unreachable here, and rung 48 says why

Anchor P6 predicted the masked row's cross would move to `+c/τ_m` (`sched`) — and the cell has
**no points**: across 24 combinations, `gov`-authoritative and `accel`-binding are **mutually
exclusive, without exception**. Rung 48's own mechanism predicts it: the `Wf/pt3` leg is
**feedforward on the cause** and fires early, the topping governor is **feedback on a
consequence** and fires late — so where the accel leg binds the cap it binds *early*, and the leg
that binds the cap is then also the leg that holds the actuator. **P6 is not a gap, it is a
consequence**, and it is why this rung's whole content is on the authoritative leg.

---

## 2. THE BILL — the path moves, and the destination does not

`φ_lim = 0.76` (the both-legs-ride arm), `margin = 0.10`:

| | `sched × none` | `applied × track` |
|---|---|---|
| peak `Tt4` | 1179.24 → **1168.98 K** (`−10.25`) | 1173.07 → **1162.86 K** (`−10.21`) |
| `min φ_lp` | 0.760992 → **0.762375** (`+1.38e−3`) | 0.766129 → **0.766762** (`+6.33e−4`) |
| fuel integral | **−1.23 %** | **−1.24 %** |
| cuts harder over the whole ramp | **yes** | **yes** |

**The sensed leg is the more protective one, everywhere on the ramp**, and the mechanism is the
droop identity: during the ramp `mf_app < cap_solve`, so `cap_sensed = cap_solve + c·(mf_app −
cap_solve) < cap_solve`. **Rung 48's set-point solve has been quietly granting the engine the
fuel it would be self-consistent with, which is more fuel than the schedule it implements
allows.**

**AND IT IS NOT A GRID ARTIFACT.** Re-read at `ds = 0.002`, a 2.5× refinement: `−10.25 K` and
`+1.387e−3` — **unmoved in every quoted digit** (`docs/pt3-sensor-lag-negative.md`'s discipline,
and unlike rung 73 § 5's marginal-`φ` column this one never needed the band).

### 2.1 The trajectories do NOT converge at the tail, and that refutes the anchor's form of D2

Anchor P10 predicted `w_f` under the two laws would agree to `1e−6` once the schedule stops.
Measured: `1.9e−2` at the tail against `1.5e−2` on the ramp — **larger, not smaller**. The
prediction was the right claim in the wrong coordinate: D2 is a statement about an
**equilibrium**, and the tail of this march is not one. The schedule stops at `s = 0.5` but the
**spools are still spinning up**, so the cap keeps moving and both legs keep chasing it. § 3
measures D2 where it actually lives — as a property of the two **laws** — and finds it exact.

---

## 3. WHAT THE SOLVE WAS BUYING — a GAIN, and this section is derivation AFTER measurement

Written to explain § 1.2's 0.7 % residual, and labelled as such (rung 75 § 0.3's precedent).
Two identities, both with **zero fitted constants**:

**(1) `cap_sensed(cap_solve) = cap_solve` EXACTLY.** `cap_solve` is by construction the fixed
point of `cap_sensed`, so the two laws **agree at the solve's own answer**. Measured
`6.94e−18` — machine zero — at every point on both stator arms. This is D2 where it lives, and
it is why the equilibrium of a leg that holds does not move.

**(2) `d(cap_solve)/dq = ( d(cap_sensed)/dq ) / ( 1 − c )`** at the same `w`, from
differentiating `cap = cap_sensed(cap, q)` in one line.

| | measured | predicted `1/(1−c)` | residual |
|---|---|---|---|
| `StatorLimiter` arm, 10 pts | **1.22799 … 1.24573** | per point | **`7.54e−9`** |
| INCIDENCE arm, 2 pts | **1.23121 … 1.23267** | per point | **`6.73e−10`** |
| at `margin = 0.40` | **1.25700 … 1.28768** | per point | **`3.18e−8`** |

**So a limiter written as a set-point solve is a STIFFER limiter than the schedule it claims to
implement** — its sensitivity to every other state is multiplied by `1/(1−c) ≈ 1.23`, and `q` is
chosen because the valve is the one other state the cap reaches through the **plant** rather than
through any leg's law. Rungs 48–75 could not see this: none of them had a second reading of the
same cap to difference against.

---

## 4. WHAT THIS DOES TO THE RUNGS BEFORE IT

* **CORRECTS rung 73 § 6's third concession**, which is the seam: `∂required/∂mf ∈ {0,1}` is
  indeed a property of this ladder's **solvers** and not of limiters, and the fourth plant it
  predicted exists. What it did **not** predict is that the continuum lands on the
  **authoritative** leg — rung 73's three readings were all readings of the *masked* one.
* **EXTENDS rung 72's *ONE plant IS rungs 68/69/70/71 by AUTHORITY*.** Rungs 73/74/75 each left
  the authoritative leg untouched and said so. This one touches only it — so the statement is now
  bounded from **both** sides, and what decides which side a device lands on is *where it lives*,
  not what it does.
* **SHARPENS rung 66's *two loops on one variable are ONE loop with the rates ADDED*** in its
  sixth shape: here the two readings of one cap are one cap with the **sensitivity divided by
  `1−c`** (§ 3).
* **BOUNDS rung 48.** Its `Wf/pt3` leg has been implemented as a fixed-point solve for 28 rungs,
  and § 2 measures what that cost: `+10.25 K` of peak `Tt4` and `1.2 %` more fuel than the
  schedule as written permits. Rung 48's *verdicts* are untouched — engagement timing, the `m`
  dial, the truncated-descent law all concern **when** the leg fires, and both readings fire at
  the same place.
* **`n_live ≤ 3` a FIFTH time, and the two rungs together finally say why.** Rung 75 could not
  break it from inside a leg's law; this cannot break it from inside the shared plant. The
  obstruction is `min`'s flatness in the masked state — the **composition**, which is neither.
  **So the only routes left to `n = 4` are a composition that is not `min` and rung 69's fourth
  LP lever** — which is what CLAUDE.md's open list already says, now with a mechanism.

---

## 5. CONCESSIONS (in addition to every one rungs 62–75 list, all inherited)

* **`margin = 0.10` IS IMPOSED**, though it is rung 48's own already-imposed scalar and not a new
  one. Every structural entry is checked at three margins and two `Tt4_max` values (§ 1.3); the
  **bill** in § 2 is quoted at one margin and is a magnitude, not a claim.
* **THE KNOB'S DOMAIN IS ONE LEG.** The φ leg and the governor keep the solve in every cell, so
  this rung says nothing about what a sensed reading of a *floor* would do — there is no such
  reading (§ 0.1). The asymmetry is declared, not discovered.
* **THE MASKED-LEG CELL IS UNREACHED** (§ 1.4). It is unreached for a reason this rung can state
  and rung 48 predicts, but it is unreached: nothing here measures a sensed cap on a **masked**
  leg, and the `+c/τ_m` cross the anchor predicted is unmeasured in both references.
* **§ 3 IS DERIVATION AFTER MEASUREMENT** and is scored as such — it explains a residual rather
  than predicting one, and its two identities were written down only once § 1.2 had missed.
* **THE `det J` RATIO IS APPROXIMATE**, `1−c` to 0.06 – 5 % depending on the arm, and § 1.2 names
  the mechanism rather than tightening the tolerance.
* **The residual floor of every Jacobian claim here is `≈ 2e−9`**, which is `eps/dg` at the
  inherited `dg = 1e−7` — so `1.1e−9` is the *differencing floor*, not agreement to nine figures,
  and the anchor's `< 1e−9` was optimistic by exactly that (§ 6, P2).
* **Every Jacobian is read at the CLIP plant's states at the inherited floor** (rung 74 § 1.3 /
  rung 75 § 1.2, inherited word for word), and under `solve`, necessarily — `clip × sensed` is
  refused. So every claim in § 1 is a difference between two laws at **one** state.
* **`Tt4_max = 1200 K`, `φ_lim`, `b_max`, `v_max` remain rung 67's imposed values**, taken
  verbatim so the *structure* differences against rungs 67–75 even though the *numbers* cannot
  (§ 0.2).
* The spectrum is sampled at finitely many trajectory points; the STAGE STACK (55/56) is still
  off the transient ladder; and this still does **not** close rung 63's *fuel + bleed + STATOR*.

---

## 6. THE REDUCE — ONE ARM, BY DISPATCH, ON FIVE CELLS

`_cap_law = 'solve'` is `AntiWindupTransient` **bit-for-bit** on the accel-armed plant, on all
five live cells (`clip×applied×none`, `demand×sched×none`, `demand×sched×track`,
`demand×applied×track`, `demand-latched×applied×none`) — the hook's branch is simply not taken,
so it is **not a tolerance**. Available because this rung reuses its parent's march: rung 71's
form, five rungs on.

Gated non-vacuous on rung 73's `charpoly_selftest` discipline: **the same machine under `sensed`
must differ** — measured, it does. And the refusals are refusals: `clip × sensed` and `sensed`
without an `accel` schedule both assert by name.

### 6.1 THE ARMS ABOVE CANNOT SEE THIS RUNG'S ONE EDIT TO ITS PARENTS, SO IT WAS CHECKED APART

`_cap_fuel` gained an `mf_app` parameter and `ma = _applied_demand(…)` moved **above** the cap
call in `_demand_laws.F` and `_rhs_laws.F` — which are rung 74's and rung 75's code, not this
rung's. Every reduce arm above runs with `accel` **armed on both sides**, so none of them can
see a change on the φ-only plant those rungs actually shipped; and `test_numeric_fingerprint.py`
— the project's only absolute-value gate — has `TwoLagCascadeTransient` (rung 66) as its
most-derived arm, so neither can it. **The reduce spine is blind to anything that moves both
sides together, and here both sides are post-change.**

So it was checked against the previous commit directly, in a `git worktree` at `HEAD~1`: 24 arms
(both classes × two φ floors × four coordinate/reference/device cells × both stator arms), all
with `accel=None`, **229,152 floats compared and bit-for-bit identical**. The edit is inert, and
12 of those arms instantiate and march `DemandCoordinateTransient` itself, so the parent's
`_sensed_cap` **staticmethod** stub — the one that runs when the child is not involved — is
exercised too.

**This is a one-off check and not a gate, which is a real gap and is recorded as one** in
CLAUDE.md § Open engineering tasks: rungs 67–76's plants have no absolute-value gate at all, and
closing it means new fingerprint arms whose goldens must be regenerated under **CPython**.

---

## 7. THE ANCHOR, SCORED

| | claim | verdict |
|---|---|---|
| D1 | the droop identity | **HELD** — 1.0e−4 measured against 1.02e−4 predicted, § 0.3 |
| D2 | the equilibria coincide where the fuel leg holds | **HELD** — `6.94e−18`, § 3(1) |
| D3 | authoritative diagonal `(c−1)/τ_f`, both references | **HELD** — `1.05e−9` / `1.11e−9` |
| D4–D6 | masked diagonal unmoved, cross moves, `n_live ≤ 3` | **HELD** except the cross (see P6) |
| D7 | `det J` scales by `1 − c` | **HELD in mechanism, not as an identity** — § 1.2 |
| D8 | the governor's row is bit-identical | **HELD** — `0.0e+00` |
| P1 | `c ∈ (0,1)`, both stator arms | **HELD** — `0.174 … 0.306` over five margins |
| P2 | authoritative diagonal `(c−1)/τ_f` to `< 1e−9` | **HELD to `1.3e−9`**, which is the `eps/dg` differencing floor — the *tolerance* was optimistic, the law was not |
| P3 | the move is identical in both references | **HELD** — the same expression fits both to the same floor |
| P4 | masked diagonal moved `0.0` exactly | **HELD** — exactly, every cell |
| P5 | `mask_leak = 0.0` ⇒ `n_live ≤ 3`, fifth | **HELD** — exactly |
| P6 | the masked row's cross moves | **UNREACHED, and § 1.4 says why** — `gov`-authority and accel-binding are mutually exclusive in 24/24 combinations, which rung 48's early/late mechanism predicts |
| P7 | `det J` ratio `= 1−c` to `< 1e−9` | **REFUTED** — `0.7 %`, and the correction became § 3 |
| P8 | `zeros` unmoved in all four cells | **HELD** |
| P9 | governor's row bit-identical | **HELD** — `0.0e+00` |
| P10 | the trajectories converge at the tail | **REFUTED** — `1.9e−2`, *larger* than on the ramp; § 2.1 |
| P11 | the sensed leg cuts harder; `φ` up, `Tt4` down | **HELD** — `−10.25 K`, `+1.38e−3`, and grid-invariant at `ds = 0.002` |
| P12 | `solve` reproduces rung 75 bit-for-bit, non-vacuously | **HELD**, five cells |
| P13 | not an artifact of `margin = 0.10` | **HELD** — `1.3e−9` worst over 18 combinations |

**Two refutations and one unreached from thirteen.** P7's refutation **became § 3**, which is the
strongest result in the rung and was not predicted at all; P10's became § 2.1, a distinction
(equilibrium vs trajectory) sharper than the equality it replaced; and P6's absence turned out to
be a **consequence of rung 48's own mechanism** rather than a hole.

**AND ONE TRAP DID NOT BITE, FOR THE FIRST TIME IN FIFTEEN RUNGS.** Rungs 61–75 each hit the
carried-knob trap (`_shared_rig` returning a fresh machine without the rung's own knob).
`_shared_rig`, `at_lever` and `_cap_march` were written to carry `_cap_law` **before** the first
reader ran, because the pattern is now named in five specs. What bit instead was its **cousin**:
the schedule κ is read off the plant's own equilibria, so an `AccelSchedule` built on `self` and
marched on `_shared_rig`'s machine would be a schedule for a *different engine* — which is why
`accel_for` exists.

---

## 8. NEXT SEAMS

* **A SENSED CAP ON A MASKED LEG.** § 1.4 says the accel leg cannot be masked while it binds. A
  *second* fuel-side leg with an explicit law — or the governor referenced to a sensed `Tt4`
  rather than a set point — would reach the cell P6 was written for.
* **`∂required/∂mf` IN THE CLIP COORDINATE.** This rung answers rung 73's seam in the **demand**
  coordinate, where the cap is a target. The clip coordinate's `required = max(0, mf_sched − cap)`
  with a sensed cap is a different hook and is refused here.
* **THE `1/(1−c)` GAIN AS A DESIGN VARIABLE.** § 3 makes stiffness a measurable property of how a
  limiter is *written*. Every other set-point solve in this family (`_topping_fuel`,
  `_surge_fuel`) has one and it has never been read.
* **A CAP WHOSE `c` APPROACHES 1.** Everything here rests on `c ≈ 0.2`. `c → 1` is a solve whose
  gain diverges — reachable by a schedule referenced to a quantity the fuel moves harder.
* Everything rungs 72/73/74/75 § 10/11 leave: an asymmetric `τ_t`, the device on the valve and
  the stator, three legs on one actuator, fuel + bleed + stator, and the real spatial PDF.
