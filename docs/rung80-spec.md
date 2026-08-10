# Rung 80 — THE SPLIT WALL

**The seam:** `docs/rung74-arrest-interval.md` § 8 — *"a four-loop live-wall march in `demand`.
§ 5 shows it is unreachable on the **shared** wall by construction. The one untried route is to
**split the wall** — arm the airflow levers ABOVE the fuel leg … That is a real knob (`sm_air`
beside `sm`), zero new constants, and it is **a rung**."*

**Plant:** `SplitWallTransient` (`turbojet/engine.py`). **Anchor + pre-registration:**
`docs/plans/rung80-anchor-split-wall.md`. **Gates:** `tests/test_rung80.py`.

---

## 0. HONESTY, FIRST — ALL THREE PRE-REGISTERED PREDICTIONS ARE REFUTED

The anchor was written and committed before any sweep ran. Every one of its predictions is
wrong, and none is softened here:

| # | predicted | measured |
|---|---|---|
| **P1** | the levers close the gap, `φ` rises to `φ_air`, **the fuel leg goes dormant** — `n_riding4 = 0`, a total order with one live floor | **REFUTED.** The fuel leg keeps cutting at **215 of 341** points and `n_riding4` reaches **33–39**. The split OPENS the four-loop cell (§ 2) |
| **P2** | four riding loops need the valve **SATURATED**, which the arrest edge forbids ⇒ the cell is shut by a **derived impossibility** | **REFUTED twice.** In the four-loop window the valve rides at **15 %** of its authority; and above the arrest edge saturated-and-marching cells exist at `φ_air` = 0.855/0.86/0.88 (§ 4) |
| **P3** | the arrest **changes owner**: sweeping `sm_air` reproduces `[0.7731, 0.7732]` on the new knob | **REFUTED.** Neither split arm arrests **at any wall**. The arrest needs the two floors to **COINCIDE** (§ 3) |

**P4 was declared unaskable in advance and stays so.** *The rank is blind to a level* is
calculus (`∂(c−φ)/∂x = −∂φ/∂x`), so no test is written on it. The measurable question — rung 69's
`c1` discriminator — is § 5, and it is reported with its own positive control.

**What survives from the anchor is its window arithmetic and its two controls**, both of which
did their job: the `clip` positive control fired on every arm, and the shared-wall control
reproduced rung 74's bracket to the digit (§ 3).

---

## 1. THE KNOB, AND THE DERIVATION THAT SAYS WHY THE FUEL LEG STAYS LIVE

Every floor in this family since rung 49 comes from ONE margin through ONE factory:

    φ_lim = (1 + sm) · cmap.phi_surge        the fuel leg (49), the valve (64), the stator (68)

Rung 80 adds a second margin for the **airflow** legs only — zero new constants, since
`phi_surge` is the map's own and `sm_air` is swept exactly as `sm` has been since rung 49:

    φ_lim = (1 + sm    ) · phi_surge         the FUEL leg's floor
    φ_air = (1 + sm_air) · phi_surge         the VALVE's and the STATOR's

### 1.1 THE TWO FLOORS ARE EVALUATED AT DIFFERENT FUEL FLOWS — AND THAT IS THE WHOLE RUNG

`docs/rung74-arrest-interval.md` § 3 established that the airflow levers act only inside the
authoritative leg's tracking error. The natural reading — which the anchor's P1 took — is that a
higher airflow floor lifts `φ` past the fuel leg's floor and switches it off. **It does not, and
the reason is that the two legs do not read the same fuel.**

The fuel leg's cut is `required_fuel = mf_sched − w*`, with `w*` solving `φ(w*) = φ_lim`. Since
`φ` decreases in fuel, that cut is positive **iff**

    φ(mf_sched) < φ_lim                                        [the COUNTERFACTUAL]

— iff the fuel the schedule *asks for* would breach the fuel wall. The airflow legs, by contrast,
hold their floor at the fuel actually burnt, `mf_app = min(mf_sched, w_f, w_r) ≤ mf_sched`:

    φ(mf_app) ≥ φ_air                                          [the ACTUAL]

and `mf_sched − mf_app` **is the fuel-side legs' own cut**. So both are satisfiable together
exactly when

    φ(mf_sched)  <  φ_lim  <  φ_air  ≤  φ(mf_app)

a band that is **non-empty precisely because the two floors differ**. On a shared wall it
collapses (`φ_lim = φ_air` leaves no room), which is rung 74's 0/341 result; split it and the
band opens.

> **THE FUEL LEG IS LIVE ON THE COUNTERFACTUAL, THE AIRFLOW LEGS ON THE ACTUAL.** A lever that
> raises the achieved `φ` **erodes** the counterfactual breach without extinguishing it — measured
> monotone, `n_cut_fuel` = 242 → 233 → 215 as `φ_air` goes shared → 0.76 → 0.77 (§ 2).

### 1.2 THE IMPLEMENTATION COST IS ONE OVERRIDE, AND IT IS DELIBERATELY NOT A KWARG

`_shared_rig` is overridden **six** times up this ladder (rungs 73/74/75/76/78/79), each calling
its parent with an explicit argument list. A new keyword at the base would be swallowed by every
one of them: the split would never happen, the reduce test would pass **because the knob was
ignored**, and `split_liveness` would honestly report *the levers did not move* — which is the
anchor's own P1. **A plumbing bug would have returned a confirmation.** So the split is applied to
the machine `super()` returns, the ladder's own idiom, and the walls are read **back off the
limiters the rig will march with** and asserted apart by name.

---

## 2. THE FOUR-LOOP CELL OPENS — § 2 MEASURED

`φ_lim = 0.75`, taus `(0.05, 0.05, 0.05, 0.05)`, `ds = 0.005`, 341 points, `Tt4_lo/hi/max =
1000/1400/1200`, `b_max = 0.10`, `v_max = 0.20`, `phi_surge = 0.55`. `φ_air = None` is the
shared wall — each coordinate's own baseline at identical settings.

| coord | `φ_air` | `φ(0)` | `min φ` | `n_cut_fuel` | valve moved | stator moved | **`n_riding4`** | `max Tt4` |
|---|---|---|---|---|---|---|---|---|
| demand | *shared* | 0.773116 | 0.752836 | 242 | **0** | **0** | **0** | 1198.07 |
| demand | 0.76 | 0.773116 | 0.755927 | 233 | 259 | 265 | **31** | 1198.10 |
| demand | 0.77 | 0.773116 | 0.764057 | 215 | 288 | 295 | **33** | 1198.11 |
| clip | *shared* | 0.773116 | 0.747436 | 182 | 245 | 251 | 23 | 1279.67 |
| clip | 0.76 | 0.773116 | 0.755228 | 160 | 273 | 280 | 19 | 1280.97 |
| clip | 0.77 | 0.773116 | 0.764057 | 149 | 296 | 303 | 23 | 1281.26 |

* **THE POSITIVE CONTROL FIRED.** `clip` moves both levers on every arm, including the shared
  baseline (245/251 at `φ_lim = 0.75`), which agrees with `docs/rung74-arrest-interval.md` § 5's
  245/251 at the same cell. A reader returning zeros there would have been broken, and every
  `demand` zero in the table uninterpretable.
* **THE `demand` BASELINE IS RUNG 74's RESULT, REPRODUCED.** Shared wall ⇒ 0 lever motion, 0
  four-loop points. **The split is what created the cells**, not a change of rig.
* **The four-loop window is narrow and interior:** at `φ_air = 0.77` it is `s ∈ [0.190, 0.350]`,
  `φ ∈ [0.771561, 0.772315]`, with the valve at `b ≤ 0.0147` — **15 % of `b_max`**. Nothing is
  saturated; P2's premise never applies.

---

## 3. THE ARREST IS A DEGENERACY OF THE **SHARED** WALL — § 3 MEASURED

`docs/rung74-arrest-interval.md` § 4 gives the mechanism as *"a floor lifts `φ(0)` ONTO the wall,
the leg opens ON its own floor with no authority left, and the accel never starts."* On a shared
wall **"a floor" and "the leg" are the same object**, so that sentence cannot say which of the two
owns the arrest. This is the only rig in the project that can separate them. Three arms, one wall
axis, `coord = demand`:

| wall `w` | **shared** (`φ_lim=φ_air=w`) | **air** (`φ_lim=0.75`, `φ_air=w`) | **fuel** (`φ_lim=w`, `φ_air=0.80`) |
|---|---|---|---|
| 0.7700 | marches — 1195.06 | marches — 1198.11 | marches — 1197.99 |
| 0.7725 | marches — 1044.91 | marches — 1198.11 | marches — 1197.99 |
| 0.7731 | marches — 1001.13 | marches — 1198.11 | marches — 1197.99 |
| **0.7732** | **ARRESTED — 1000.00** | marches — 1198.11 | marches — 1197.99 |
| 0.7740 | **ARRESTED — 1000.00** | marches — 1198.11 | marches — 1197.99 |
| 0.7800 | **ARRESTED — 1000.00** | marches — 1198.09 | marches — 1197.96 |

* **THE CONTROL REPRODUCES RUNG 74's BRACKET EXACTLY** — last march 0.7731, first arrest 0.7732.
  The rig is therefore the same rig, and the two split arms are interpretable.
* **NEITHER SPLIT ARM ARRESTS, AT ANY WALL.** `owner = []`.
* **AND THE LIFT STILL HAPPENS.** In the `air` arm at `w = 0.78`, `φ(0)` is lifted from 0.773116
  onto **0.78** with the valve opening to 9.6 % of `b_max`; in the `fuel` arm `φ(0)` sits on
  **0.80** with the valve at 36.6 % throughout. The lift is present, and there is no arrest.

> **HEADLINE (§ 3) — THE ARREST BELONGS TO NEITHER FLOOR. IT BELONGS TO THEIR COINCIDENCE.**
> The lift that *causes* the arrest is the lift that *cures* it, once the floors differ. At `s=0`
> the top floor puts `φ(0)` on itself; a fuel leg watching that same level opens with **exactly
> zero** margin and cannot let the accel start, while a fuel leg watching a level `φ_air − φ_lim`
> **below** it opens with that margin in hand. Rung 74's arrest interval is a statement about a
> **degenerate** configuration, not about a wall height.

---

## 4. THE SATURATION EDGE, INDEPENDENTLY REPRODUCED — § 4 MEASURED

`φ_lim = 0.75` fixed, `φ_air` swept, `coord = demand`:

| `φ_air` | 0.78 | 0.80 | 0.82 | 0.84 | 0.85 | **0.855** | 0.86 | 0.88 |
|---|---|---|---|---|---|---|---|---|
| `b(0)/b_max` | 0.096 | 0.366 | 0.624 | 0.869 | **0.987** | **1.000** | 1.000 | 1.000 |
| saturated | no | no | no | no | no | **yes** | yes | yes |
| arrested | no | no | no | no | no | no | no | no |

The valve's saturation edge is bracketed at **[0.850, 0.855]** — `docs/rung74-arrest-interval.md`
§ 4's own bracket, to the same two digits, on a **different rig** (split walls, fuel leg at 0.75).
**That edge is a property of the valve and the operating point, not of the shared wall** — which
that document could not have shown, because on a shared wall the two are confounded.

It also kills P2's second half: cells that are **saturated AND marching** exist (0.855, 0.86,
0.88), so "saturation ⇒ arrest" was never true either.

---

## 5. THE FOURTH LOOP IS **RIDING**, NOT **AUTHORITATIVE** — § 5 MEASURED

Rung 72 established the noun that matters for rank: under min-select exactly one of the two
fuel-side legs reaches the actuator, the other is **MASKED**, and `n_live` counts the loops
holding **AUTHORITY**. `_riding4` counts something else — all four *riding and strictly interior*.
On a shared wall the distinction never mattered, because `_riding4` was empty in `demand` anyway.
Here it does. Twelve-gain reader over the `_riding4` points, every 5th:

| coord | `φ_air` | riding | interior | skipped | authority | masked | `max \|mask_leak\|` | zeros | `max \|cyclic\|` |
|---|---|---|---|---|---|---|---|---|---|
| demand | *shared* | 0 | 0 | 0/0 | — | — | — | — | **VACUOUS** |
| demand | 0.77 | 33 | 7 | **0/0** | `gov`×7 | fuel | **0.0** | {1} | **0.0** |
| demand | 0.80 | 19 | 4 | **0/0** | `gov`×4 | fuel | **0.0** | {1} | **0.0** |
| clip | *shared* | 23 | 4 | 0/1 | `fuel`×1, `gov`×3 | both | 0.0 | {1, 2} | **0.99999999986** |
| clip | 0.77 | 23 | 5 | **0/0** | `gov`×5 | fuel | **0.0** | {1} | **0.0** |
| clip | 0.80 | 4 | 1 | **0/0** | `gov`×1 | fuel | **0.0** | {1} | **0.0** |

**THE ZEROS ARE GUARDED, because an exact zero can mean "nothing was computed"** — rung 78 § 5.1's
logged trap, and in `demand` every interior cell masks the *same* leg, so `mask_leak` and the
cyclic product's `V_f` factor are one zero seen twice. Two discriminators, both required before
the row above is read:

1. **THE POSITIVE CONTROL.** The `clip` shared-wall arm contains a cell where the **governor** is
   the masked leg (`s = 0.135`, authority `fuel`). On the identical code path it returns a cyclic
   product of **0.99999999986** — rung 66's *"two loops on one variable give exactly 1"*. The
   instrument does produce non-zero values; the zeros elsewhere are structural.
2. **EVERY POINT WAS DIFFERENCED.** `skipped = {switch: 0, regime: 0}` on all four split arms, so
   the zeros are computed over 7/4/5/1 points and not over an empty set.

And `ever_two_authorities` is **False** everywhere: no cell has both fuel-side legs holding.

---

## 6. THE HEADLINE — TWO NOUNS, AND THE SEAM NAMED THE WRONG ONE

> **A LEVEL SPLIT SEPARATES LOOPS ON THE CONSTRAINT; IT CANNOT SEPARATE THE TWO THAT SHARE THE
> ACTUATOR.** The split delivers the **four-loop cell** — all four loops riding and strictly
> interior in `demand` for the first time in this family, the seam's own object, at last
> non-empty. It does **not** deliver a **fourth live loop**: the fuel leg and the governor are
> still on one actuator under `min`, one is still masked with an **exactly zero** column, and
> `n_live` is **still ≤ 3 — a SIXTH time.**

That is why rungs 72–79 could not find it and why five closed routes all failed the same way:
**they were looking for the rank noun and the seam was written in the riding noun.** Rungs
72/73/74/75/76 each closed by refuting a route to `n_live = 4`; this rung closes the *remaining*
route by showing it was never a route to that quantity at all. `n = 4` on **one** actuator needs a
non-`min` composition (rung 76 named the composition; rung 79 named the solver's short-circuit) —
a level on the constraint cannot buy it, however wide.

---

## 7. WHAT THIS CORRECTS

* **`docs/rung74-arrest-interval.md` § 4 and `docs/rung74-spec.md` § 2.2** — the arrest's
  mechanism is stated as *a floor lifts `φ(0)` onto the wall*. Measured with the floors separated
  (§ 3): **the lift alone does not arrest anything.** `φ(0)` is lifted to 0.78 and to 0.80 in the
  two split arms and both march. The arrest requires the fuel leg to be watching **the level it
  was lifted onto** — a **coincidence of two floors**, not a height. Rung 74 found, mechanised and
  gated the arrest, and its bracket reproduces here to the digit; what moves is *what the bracket
  is a bracket in*.
* **`docs/rung74-arrest-interval.md` § 8** — the seam's premise, *"unreachable on the shared wall
  by construction"*, is **confirmed**; its proposed cure, *"the queue's order reverses and they
  become authoritative"*, is **refuted**. The levers do become live; they never become
  authoritative, because authority is decided on the **actuator** and they are not on it.
* **`docs/rung74-arrest-interval.md` § 4's saturation edge** is confirmed at [0.850, 0.855] on an
  independent rig (§ 4), and **re-attributed**: it is the valve's, not the shared wall's.
* **Nothing in rungs 68 § 7 / 72 / 76 is wrong.** `mask_leak = 0` and one authority per point are
  rung 72's results, and they survive a wall split of any width measured.

---

## 8. WHAT THIS DOES **NOT** SAY

* **No rank claim.** *A gradient is blind to its own level* is calculus and is not tested (§ 0).
  `zeros = {1}` is **reported** in § 5, never gated.
* **`n_riding4` IS MEANINGLESS ON AN ARRESTED PLANT**, and this rung's own table proves it: the
  shared arm at `φ_lim = 0.78` reports **320** four-loop points with `max Tt4 = 1000.0` exactly —
  the plant never moved. That is `docs/rungs72-77-march-audit.md`'s *"a liveness counter on a
  FROZEN plant reports FULL activity"*, reproduced. Every row carries `riding4_valid`, and the
  count must never be quoted without it.
* **One rig, one flight condition, one `b_max`/`v_max` pair, one `phi_surge`.** The four-loop
  window's width, the 15 % valve usage and the saturation edge are all this hardware's.
* **The `demand` four-loop cells all have the GOVERNOR authoritative** (11 of 11). Nothing here
  exhibits a `demand` cell with the φ fuel leg holding the actuator while all four ride, and no
  claim is made about one.
* **The split's own physical meaning is disclosed, not defended.** Real bleed and stator schedules
  are not referenced to a surge-margin floor above the fuel limiter's; this is the rung-68 family's
  `φ`-referenced construction with one number moved, and it exists to answer a structural question.

---

## 9. THE REDUCE CONTRACT, AND THE GATES

**REDUCE — two arms, both exact.** `_sm_air = None` dispatches to rung 79 on an `is None` test, so
the override returns `super()`'s machine untouched: `coord_scan` is **identical**, verified by
serialising both. And `sm_air == sm` — which rebuilds the same floors from the same factory —
agrees with the `None` path **bit-for-bit** over all 341 points on `φ_lp`, `Tt4`, `b` and `v`.

**GATES** (`tests/test_rung80.py`):

1. **the reduce**, both arms above.
2. **the shared-wall CONTROL**: rung 74's bracket `(0.7731 marches, 0.7732 arrests)` reproduced —
   a derived edge, so it cannot be satisfied by tuning.
3. **`owner == []`**: neither split arm arrests at any wall, with the lift asserted **present**
   (`φ(0)` on the airflow wall, valve open) so the null is not a dormant knob.
4. **the cell opens**: `demand` shared ⇒ `n_riding4 == 0`; `demand` split ⇒ `n_riding4 > 0`, with
   the `clip` positive control non-zero on every arm.
5. **the fuel leg stays live** (`n_cut_fuel > 0` on every split arm) **and erodes monotonically**
   in `φ_air` — § 1.1's derivation, scored on the counterfactual noun.
6. **the mask survives the split**: one authority per interior point, `mask_leak == 0` exactly,
   **and the positive control non-zero**, and `all_differenced` — the zero is not an empty set.
7. **the knob is loud**: a rig built with `sm_air` set must carry `φ_air > φ_lim` on the limiters
   it marches with.

**Deliberately NOT gated:** `zeros`, `c1` and the cyclic product's *value* (§ 0's P4 —
definitional or reported-only), and `n_riding4` on arrested rows (§ 8).

---

## 10. NEXT SEAMS

* **A `demand` FOUR-LOOP CELL WITH THE φ FUEL LEG AUTHORITATIVE.** All 11 measured have the
  governor holding. Whether the φ leg can hold the actuator while all four ride is untested, and
  it is the cell § 5's table cannot reach.
* **THE SPLIT ON A NON-`min` COMPOSITION.** § 6 says a level cannot separate two legs on one
  actuator. Rung 76 named the composition as the obstruction; this rung shows a second knob that
  does not reach it. `n_live = 4` still needs either a 4th lever that is not on the fuel actuator
  or a composition that is not `min`.
* **THE ARREST'S COINCIDENCE, MADE CONTINUOUS.** § 3 measures split vs shared. How the arrest
  turns off as `φ_air − φ_lim → 0⁺` — sharply or smoothly — is not measured, and it is the one
  sweep that would say whether the degeneracy is a measure-zero set or a neighbourhood.
* **A LARGER `b_max`.** The saturation edge is hardware (§ 4); rung 74 § 8's concession is
  inherited unchanged.
* **A FINGERPRINT ARM FOR THIS RUNG.** `tests/test_numeric_fingerprint.py` pins absolute values
  for rungs 67–79 (its docstring §§ SLICE 3–4). Rung 80 has none, and the tracking line that used
  to live under CLAUDE.md § Open engineering tasks was deleted in this rung's commit once its
  three entries all closed — so the debt is recorded **here**, deliberately, rather than nowhere.
  A slice 5 would lead with `split_arrest`'s three-arm table, since it is the reader that
  exercises the new knob on both sides of the shared/split boundary.
* Everything rungs 72–79 §§ 8–11 leave, unchanged by this rung.
