# Rung 74 — THE DEMAND COORDINATE

Rung 72/73's six states, four clocks, four loops and three actuators — **every one of them
unchanged**. The only thing added is *which variable the two fuel-side legs lag*.

    CLIP    dg/ds = ( required - g ) / τ ,  g ≥ 0 ,   mf = mf_sched − max(gf, gr)
    DEMAND  dw/ds = ( cap      − w ) / τ ,  no floor, mf = min(mf_sched, w_f, w_r)

Rung 73 § 11 named it **its sharpest seam** and *the last place `n_live = 4` could still hide*:
*every leg in this family lags its clip; a real fuel control lags the demand (`w = mf_sched − g`).
On a ramp those are different plants — they differ by `ṁf_sched·τ`.*

> **HEADLINE — A COORDINATE ON THE LAG IS PURE BILL. IT CANNOT TOUCH THE RANK, AND IT MOVES THE
> CUT BY THE SCHEDULE'S OWN SLOPE.** Substituting `w = mf_sched − g` gives
> `dg/ds = (req − g)/τ + ṁf_sched` — a **state-independent forcing**, so it appears in no
> Jacobian. Measured on two Jacobians at one state through two different closures: the
> characteristic polynomial agrees to **1.24e−9 relative**, the non-crossing entries agree
> **EXACTLY (0.0)**, and all four cyclic products to **5.2e−11**. And `min()` is flat in the
> masked **demand** exactly as `max()` was flat in the masked **clip** — `mask_leak = 0.0` in
> BOTH coordinates — so the masked column is still zero, `M` is still block-triangular, and
> **`n_live` is still ≤ 3. The seam closes by REFUTATION, the third rung running.**

> **WHAT THE COORDINATE DECIDES IS WHAT THE LAG IS LAGGING BEHIND.** A first-order lag tracks a
> slow target well and a ramping one poorly, with a steady error of the target's slope times
> `τ`. In clip coordinates the target `required = mf_sched − cap` rides the **SCHEDULE**; in
> demand coordinates the target `cap` rides the **PLANT**. Same leg, same clock, and the
> tracking error goes from `ṁf_sched·τ` to `ċap·τ`. Measured open loop: **0.9969 × the
> predicted `ṁf_sched·τ`**, decaying to `1.4e−14` when the ramp stops.
>
> **SO IT CORRECTS RUNG 47's HEADLINE CONCESSION.** *The cost of realism is that a lagged
> governor breaks the redline hold* is a property of the **COORDINATE**, not of the lag: on
> all six arms the clip plant overshoots `Tt4_max` by **+79 … +83 K** and the demand plant
> sits **UNDER it, by 1.85 … 2.32 K**.

> **AND THE FLOOR CHANGES ADDRESS, WHICH DECIDES WHETHER THE PLANT EXISTS.** The clip law floors
> the **STATE** (`g ≥ 0`); the demand law floors the **COMPOSITION** (`mf ≤ mf_sched`). The two
> admit the same applied fuels and are not the same plant — **rung 52's `max(0,·)`, inherited
> unexamined for 22 rungs, is this family's implicit anti-windup device.** So rung 73 § 0.2's
> *an applied-referenced leg is self-anti-winding under min-select — that is a property of the
> composition* is **CORRECTED**: it is a property of the **coordinate's stop**. Remove the stop
> and the same motion has nothing in its path — **the masked leg has no interior equilibrium at
> all and the plant does not exist** (§ 4).

Pre-registration: `docs/plans/rung74-anchor-demand-coordinate.md`, whose § 0 discloses its order
and reports two measurements taken before it existed. Gates: `tests/test_rung74.py`.

---

## 0. WHAT MOVED, AND WHAT DID NOT

| | rung 72 | rung 73 | **rung 74** |
|---|---|---|---|
| the loops | fuel, gov, valve, stator | the same four | **the same four** |
| states / actuators | 6 / 3, one shared | 6 / 3, one shared | **6 / 3, one shared** |
| what is added | a leg | a REFERENCE | **a COORDINATE** |
| the masked leg's pole | `−1/τ_masked` | the ORIGIN | **whichever the reference sets** |
| `det J` | rung 71's cell only | NOWHERE | **unmoved — the spectrum is invariant** |
| the bill | — | +9 … +71 K | **−81 … −283 K** |

**No state, no gain, no clock, no loop and no reference is added.** That is what makes this the
cleanest experiment in the family after rung 73's — and what makes a **bill** change this large
with a spectrum change of **zero** worth a rung.

### 0.1 The three declared knobs

`_lag_coord` joins rung 72's `_share_law` and rung 73's `_ref_law`:

| `_lag_coord` | state | target | floor | is |
|---|---|---|---|---|
| `clip` | `g` | `required` | on the **STATE** | **rung 73/72, bit-for-bit** |
| `demand` | `w` | `cap` | on the **COMPOSITION** | **THE PLANT** |
| `demand-latched` | `w` | `min(mf_sched, cap)` | on the **STATE** | § 3's instrument |

`demand × sum` is **refused** — `min(mf_sched, w_f, w_r)` has no `sum` reading that keeps the
schedule as an input, so marching it would swap two declared laws at once (rung 73's refusal of
`applied × sum`, in its own words).

### 0.2 The cap had to be unfloored first, and it was measured first

Every shipped cap is **floored at the schedule**: `_surge_fuel` and `_sched_fuel` return
`mf_sched` *itself* when the leg is clear, and `required_gov` short-circuits before
`_topping_fuel` runs. **The cap above the schedule had never been computed in this project**,
and the demand target *is* that cap. Measured along rung 73's own trajectory, every point:

| | `s = 0` | `s = 0.1` | from `s = 0.2` | failures |
|---|---|---|---|---|
| `cap_Tt4 / mf_sched` | **1.3032** | 1.0190 | binding (< 1) | **0 of 341** |
| `cap_φ / mf_sched` | **1.0000** | binding | binding | **0 of 341** |

Using the *floored* cap instead would have manufactured a dormant-leg cut and let it be reported
as a finding. `_cap_free` walks the inherited bracket in the other direction (`1/0.9` against
`_surge_fuel`'s own `0.9` — a direction, not a new constant), returns the **shipped** number
wherever the leg binds (gated), and asserts by name rather than falling back.

---

## 1. THE DERIVATION, AND THE INSTRUMENT THAT MUST NOT CONSTRUCT IT

### 1.1 One line, and it is not scored

    w = mf_sched − g ,  cap = mf_sched − req   ⇒   dg/ds = (req − g)/τ + ṁf_sched

The added term is a function of `s` alone. **It cannot appear in a Jacobian**, so the spectrum,
`zeros`, `det J` and every cyclic product are invariant. This is listed as derivation and
**not scored** — a `_jac4` handed eigenvalue invariance by construction would be the sixth
instance of the shipped-instrument-agrees-with-itself pattern (rung 67 gate 9, rung 71 § 1.4,
rung 72 §§ 4 and 8, rung 73's `_reference` no-op that returned a *perfect confirmation* having
measured nothing).

**So `w` is marched as a genuine state and the schedule is never differentiated.** That also
handles the ramp's two kinks exactly, where a derivative would have had to pick a branch.

### 1.2 In *demand* coordinates the matrix is `D J D⁻¹`, `D = −I` on the fuel block

Every fuel↔non-fuel off-diagonal flips sign; fuel↔fuel and non-fuel↔non-fuel entries do not;
cyclic products cross the block an even number of times and are invariant. **So the spectrum
cannot be the discriminator and the entries are.** Measured — two Jacobians at the same state,
through `_demand_laws` against `_quad_laws`, 41 interior points (171 off-regime, 1 near-switch):

| quantity | measured | prediction |
|---|---|---|
| charpoly gap, relative | **1.24e−9** | 0 |
| fuel↔non-fuel entries, after the sign flip | **4.53e−10** | 0 |
| the other off-diagonals | **0.0, exactly** | 0 |
| the four cyclic products | **5.17e−11** | 0 |
| `mask_leak`, both coordinates | **0.0, exactly** | 0 |
| entries that genuinely changed sign at O(1) | **6**, largest **562.9** | ≥ 1 |

The last row is the gate that matters: a port that silently changed nothing would pass every
other line.

### 1.3 The states are the CLIP plant's, and that is a disclosure

A Jacobian is a function of the state, not of which trajectory passed through it — but only one
plant has all four legs riding. `_shared_rig` builds the surge leg, the valve and the stator
from **one** margin, so at the lowered floors § 2 needs, the valve is off-regime at every point
and there is no interior cell at all; at the inherited floor the clip plant rides all four and
the demand plant does not accelerate (§ 2.2). Reading both matrices on the clip plant's states
is what lets one window serve both, and it costs nothing this rung claims.

### 1.4 The forcing, isolated — and why § 3 cannot do it

Two plants that differ at all differ **everywhere downstream**, so a closed-loop
`latched − clip` measures the forcing *plus every consequence of having applied it*. Read open
loop along one trajectory, both lag laws against their own targets at the same states:

| | measured | predicted |
|---|---|---|
| `ṁf_sched` | 0.0277351 | — |
| `ṁf_sched·τ_gov` | — | 1.3868e−3 |
| mean `g_demand − g_clip`, late ramp | **1.3825e−3** | ratio **0.9969** |
| worst point, late ramp | — | **1.2%** |
| after the ramp | 1.3865e−3 → **1.45e−14** | → 0 |

**A forcing, not a gain**: it lives while the schedule moves and unwinds to machine zero when it
stops.

---

## 2. MEASURED — THE BILL, AND RUNG 47's CONCESSION

`Tt4_max = 1200 K`, `b_max = 0.10`, `v_max = 0.20`, `ds = 0.005`, clocks all `0.05`, all
inherited. `φ_lim` is swept — see § 2.2.

| arm | `φ_lim` | clip `max Tt4` | latched | demand | Δ(coordinate) | Δ(floor) | `min φ_lp` |
|---|---|---|---|---|---|---|---|
| `φ` | 0.80 | 1283.36 (**+83.4**) | 1000.00 | 1000.00 | **−283.4** | 0.00 | 0.79515 → **0.80000** |
| `φ` | 0.76 | 1281.11 (**+81.1**) | 1197.35 | 1197.68 (**−2.32**) | **−83.4** | 0.33 | 0.75643 → **0.76099** |
| `φ` | 0.70 | 1279.18 (**+79.2**) | 1197.98 | 1198.15 (**−1.85**) | **−81.0** | 0.18 | 0.74299 → **0.74644** |
| `M_i` | 0.80 | 1282.76 (**+82.8**) | 1000.00 | 1000.00 | **−282.8** | 0.00 | 0.79138 → **0.80000** |
| `M_i` | 0.76 | 1279.07 (**+79.1**) | 1197.35 | 1197.68 (**−2.32**) | **−81.4** | 0.33 | 0.75353 → **0.76099** |
| `M_i` | 0.70 | 1279.18 (**+79.2**) | 1197.98 | 1198.15 (**−1.85**) | **−81.0** | 0.18 | 0.74299 → **0.74644** |

**Every arm is read at ONE reference (`sched`), and the first version of this reader was not.**
It read the clip plant under rung 73's APPLIED reference and the two demand plants under
`sched`, so every number was the coordinate PLUS the reference — 32 K and 71 K of the `φ_lim =
0.80` arms, against a reference worth +162 K inside one coordinate (§ 4). **That is § 0.1's own
refusal, broken by this rung's own reader**, and it is recorded rather than quietly fixed: rung
63's *change one law at a time* is easy to state in a spec and easy to lose in a `for` loop.

**The clip plant breaches the redline on all six arms; the demand plant holds it on all six**,
and `min φ_lp` rises on all six too — an extra cut is protective on both currencies.
The `φ_lim = 0.70` arm sits below the clip plant's own droop, so only the **governor** is live
there — which is what makes it a statement about rung 47's leg and not rung 49's.

**AND THE SPLIT IS DECISIVE.** The **coordinate** is worth 81–283 K; the **floor's address** is
worth ≤ 0.33 K. The anti-windup half of this rung matters *structurally* (§ 4: whether the plant
exists), not numerically — **on this currency**. See § 2.3.

### 2.1 The hand-over moves LATER, and the prediction that it would move earlier was wrong

Anchor P5 predicted earlier, reasoning that both legs carry more cut and the governor's grows
faster. Measured (`φ_lim = 0.76`, first governor authority):

| `φ_lim` | | `φ` arm | `M_i` arm |
|---|---|---|---|
| 0.76 | clip | 0.180 | 0.205 |
| 0.76 | **demand** | **0.625** | **0.625** |
| 0.76 | demand-latched | 0.760 | 0.760 |
| 0.70 | clip / demand | **0.110 / 0.110** | **0.110 / 0.110** |

**3.0–3.5× later, and the reason retires the prediction's own logic**: a hand-over is set by
*when the other leg's constraint arrives*, not by which clip grows faster. The φ leg's extra cut
keeps `Tt4` down, and keeping `Tt4` down is exactly what delays the governor.

**AND THE `φ_lim = 0.70` ROW IS THE MECHANISM'S OWN CONTROL.** There the surge leg is below the
clip plant's droop and never arms, so there is no extra cut to delay anything — and the
hand-over is **unmoved to the grid cell**, in a coordinate that moved the peak `Tt4` by 81 K.

### 2.2 THE ARREST — disclosed, not tuned away

At the **inherited** floor (`φ_lim = 0.80`) the surge cap equals the scheduled fuel at `s = 0`
(§ 0.2). A leg that *tracks* its cap therefore pins `φ` on the floor and the acceleration never
starts: `max Tt4 = Tt4_lo` **exactly**, `min φ_lp = 0.800000` **exactly**. This is the strongest
form of the rung's claim — **the whole accel at that floor is powered by the clip coordinate's
own tracking error** — and it is why the comparison arms sit at floors the accel survives.
`φ_lim` has been an imposed, swept coordinate since rungs 36/49 and is one here.

### 2.3 The floor's address is negligible in one currency and not in another

≤ 0.33 K on peak `Tt4`, and **0.135 in hand-over time** (0.625 against 0.760) — a 22% shift in
when the governor takes the actuator. Rung 49's *a limiter acts through both edges on different
clocks* in a fifth shape: **"negligible" is a statement about a currency, never about a law.**

---

## 3. THE ISOLATION INSTRUMENT, AND THE PREDICTION IT REFUTED

`demand-latched` is exactly the clip plant plus the forcing, so the three arms split the rung:
`latched − clip` is the **coordinate**, `demand − latched` is the **floor's address**.

Anchor **P6** predicted `demand − latched` would be *zero to machine precision wherever both
legs are riding*. **REFUTED, and on a confusion the project has hit before**: that is a property
of the **LAW**, and the reader compared **TRAJECTORIES**. Measured `floor_dTt4 = 65.2 K` with
332 of 341 points riding — because by then the two plants are at different states, and every
difference downstream of a difference is one too. Rung 58's *check the SUM, not the term* and
rung 63's *check a quoted number was taken at this rung's settings*, in a third shape. § 1.4's
open-loop reader exists because of it.

---

## 4. THE STOP WAS DOING THE ANTI-WINDUP — rung 73 § 0.2, CORRECTED

Rung 73 § 0.2: *masked means `gr > gf ≈ req_f`, so `dgf/ds = (req_f − gr)/τ_f < 0` … **an
applied-referenced leg is self-anti-winding under min-select — that is a property of the
composition, not of these numbers.*** The **motion** is a property of the composition and
reproduces here exactly (it is the same law, sign-mirrored). **Where it stops is not.**

| cell | plant? | masked leg's `w / mf_sched` | `max Tt4` |
|---|---|---|---|
| `demand` × `sched` | **yes** | **1.1536** | 1197.68 |
| `demand` × `applied` | **NO — no interior equilibrium** | — | — |
| `demand-latched` × `sched` | yes | 0.9864 | 1197.35 |
| `demand-latched` × `applied` | yes | **1.0 — exactly the stop** | 1359.88 |

In clip coordinates the masked applied-referenced leg runs **into** the floor at `g = 0` and
halts; rung 73 § 0.3 had to add that same stop to its IC sweep and said its *equilibrium is the
stop itself*. In demand coordinates the identical motion is `dw/ds = (cap − mf_app)/τ > 0` with
**nothing in its path**: the joint IC sweep diverges (residual 2.898e−3 after 60 iterations) and
the march never starts. The pair is the proof — **with** the stop the leg parks at *exactly*
`w/mf_sched = 1.0`; **without** it there is no plant.

`windup_law` reports this as a **cell table rather than an assertion**, because *the plant does
not exist* is a measurement about one of four cells and not an error.

---

## 5. WHAT THIS DOES TO THE RUNGS BEFORE IT

* **RUNG 47's HEADLINE CONCESSION IS CORRECTED** (§ 2). *The cost of realism is that a lag
  breaks the redline hold* — measured as a property of the coordinate, not of the lag: +79…+83 K
  of overshoot becomes 1.9…2.3 K of undershoot with no clock, gain, loop or state moved.
* **RUNG 73 § 0.2's SELF-ANTI-WINDING IS CORRECTED** (§ 4): a property of the coordinate's stop,
  not of the composition. Its § 0.3 state floor turns out to have been load-bearing for a reason
  it did not state.
* **RUNG 52's `max(0,·)` IS NAMED** for the first time in 22 rungs: an anti-windup device, not
  bookkeeping.
* **RUNGS 49–73's ACCELERATIONS ARE BOUNDED** (§ 2.2): at the inherited floor the accel exists
  only because the clip coordinate's leg lags its own target by `ṁf_sched·τ`.
* **RUNG 69's SPLIT IS SHARPENED FROM THE OTHER SIDE.** Rung 69: *a loop's COORDINATE decides
  whether it adds a zero or a rank* — that is the **CONSTRAINT's** coordinate. This is the
  **STATE's**, and it cannot touch the rank at all. **A constraint's coordinate is geometry; a
  state's is bookkeeping — and only one of them is in the Jacobian.**
* **RUNG 72's `n_live` SURVIVES A THIRD ATTACK** — and by the same mechanism each time, which is
  now stated in its coordinate-free form: *min-select is flat in whatever the masked leg holds.*

---

## 6. CONCESSIONS (in addition to every one rungs 62–73 list, all inherited)

* **`φ_lim` had to be swept** to get a comparable trajectory (§ 2.2). It is an imposed coordinate
  since rungs 36/49, but this is the first rung whose *arms* depend on moving it.
* **The unfloored cap is a solve in a regime the family has never exercised.** Measured available
  at 341/341 points on both arms; outside that it asserts by name rather than falling back.
* **`_shared_rig` gives every leg one margin**, so a lowered floor makes the valve dormant — which
  is why § 1's states are the clip plant's (§ 1.3) and why no cell mixes a lowered floor with a
  gains reading.
* **The demand plant is `sched`-referenced only.** `demand × applied` has no plant (§ 4); the
  applied reference is therefore measured only through `demand-latched`.
* **Nothing here re-derives `cap` for a fuel-dependent constraint** — rung 73 § 11's second seam
  is untouched, and it would make `∂required/∂mf ∉ {0,1}` in *both* coordinates.

---

## 7. THE REDUCE — TWO ARMS, AND THE SECOND IS THE ONE THAT RUNS

1. **`_lag_coord = 'clip'` ⇒ rung 73, BIT-FOR-BIT, by exact dispatch** — the march is not
   entered. Gated non-vacuous: the same machine under `demand` must differ (rung 73's
   `charpoly_selftest` discipline).
2. **`demand-latched` on a FLAT schedule ⇒ the clip plant, BY IDENTITY.** The forcing is
   identically zero and the two floors coincide, so this is rung 71's *inherited identity* form
   — and it is the only reduce in which `_integrate_fuel_demand` actually runs. Gated
   non-vacuous: 241 of 241 points riding, `Tt4` spanning 1162 → 1199 K, off the running line.

   **Anchor P7 said bit-for-bit and is REFUTED-as-stated**: the two marches compute the same
   quantity through different float expressions (`cap − w` against `−(req − g)`), so the
   agreement is `1.59e−12` on `Tt4` (≈1.3e−15 relative) and `3.9e−18` on both clips. The gate
   carries the measured tolerance and the reason — rung 73 § 1.3's precedent, where not letting
   an instrument agree with itself cost five orders of magnitude and was worth it.

---

## 9. THE ANCHOR, SCORED

| | claim | verdict |
|---|---|---|
| D1 | forcing is state-independent ⇒ `J` unmoved | **HELD** (derivation; § 1.2 measures it) |
| D2 | `D = −I` sign pattern | **HELD** — non-crossing entries exactly 0.0 |
| D3 | `min()` flat in the masked demand ⇒ `n_live ≤ 3` | **HELD** — `mask_leak = 0.0` both |
| D4 | steady error `ṁf_sched·τ` | **HELD** — ratio 0.9969 |
| D5 | flat schedule ⇒ the clip plant | **HELD in substance, REFUTED as *exact*** (§ 7) |
| P1 | spectrum agrees ≤ 1e−9 relative | **HELD at 1.24e−9**, gated one decade above |
| P2 | the entries do **not** agree | **HELD** — 6 sign changes, largest 562.9 |
| P3 | `zeros` / `det J` / pairs agree cell for cell | **HELD, one step removed** — the pairs directly (5.2e−11) and the whole characteristic polynomial at 41 states to 1.24e−9 relative, which **pins** the zero counts and `det J = a0`: a count could differ only if a root sat within that margin of the threshold, and rung 73's live determinant is `+5.9e4`. The per-cell table is not separately re-run and does not need to be. |
| P4 | `max Tt4` falls, `min φ` rises | **HELD on both, all six arms** — `Tt4` by 81–283 K, `φ_lp` on every arm (§ 2's last column). An extra cut is protective on both currencies, as an extra cut must be |
| P5 | the hand-over moves **earlier** | **REFUTED** — 3.0–3.5× **LATER**, and the reason retires the prediction's logic; the `φ_lim = 0.70` row is its control (§ 2.1) |
| P6 | `demand − latched` zero wherever both ride | **REFUTED** — a law/trajectory confusion; § 1.4's reader exists because of it (§ 3) |
| P7 | flat-schedule reduce bit-for-bit | **REFUTED as stated** — 1.59e−12 (§ 7) |
| P8 | the lag returns `tau_att` on attack | **HELD**, gated |
| P9 | `clip` reproduces rung 73 bit-for-bit | **HELD**, gated |

**Three refutations and one partial, from thirteen — and all three became content** (P5's
retired logic, P6's law/trajectory confusion and the reader it forced, P7's float tolerance).

**AND TWO FINDINGS WERE NOT PREDICTED AT ALL:** the **arrest** (§ 2.2) and the **missing
equilibrium** (§ 4) — the second of which corrects a shipped rung. Both were found by the first
march that ran, which is the argument for marching before writing the readers.

---

## 10. NEXT SEAMS

* **THE STATE-AS-DEMAND COORDINATE FOR THE VALVE AND THE STATOR.** Only the two *fuel-side* legs
  were re-coordinated here. The valve holds a position and the stator an angle — neither has a
  schedule under it, so the forcing has no analogue; but `b_max` and `v_max` are **state floors**
  in exactly rung 52's sense, and § 4 says a state floor can be the only thing giving a masked
  loop an equilibrium.
* **A FUEL-DEPENDENT CAP** (rung 73 § 11's second seam) — untouched, and now known to be a
  question about *both* coordinates.
* **THE ARREST, AS A PLANT.** At `φ_lim = 0.80` the demand plant has an equilibrium and no
  transient. What accelerates it — a schedule that outruns the leg's own clock — is rung 44's
  ramp-rate lever pointed at a limiter that no longer lags the ramp.
* **AN ANTI-WINDUP DEVICE, EXPLICITLY.** § 4 says the clip coordinate has one by accident. A
  declared tracking/reset law on the demand plant would make `demand × applied` exist, and the
  comparison would isolate what the accident was buying.
* Everything rungs 72/73 § 11 leave: three legs on one actuator, an asymmetric valve/governor,
  fuel + bleed + stator, and the real spatial/transported-CFD PDF.
