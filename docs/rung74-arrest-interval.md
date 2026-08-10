# Rung 74's ARREST is an INTERVAL with TWO DERIVED EDGES — and the DEMAND coordinate has no four-loop cell

**Not a rung.** No new effect, no new plant code, no new constant, no new knob — five shipped
readers driven over the wall axis. This is a **CORRECTION to shipped rungs**, in the bucket
`docs/rungs72-77-march-audit.md`, `docs/rung79-gap-margin.md` and `docs/rung29-pi-c-margin.md`
occupy.

**HONESTY, FIRST — AND TWICE.**

1. This was found by **exploration**, not by a pre-registered prediction. Nothing below is
   scored as a prediction met. What *was* fixed in advance is the **discriminator** (§ 1), and
   it is named as such.
2. **Half of § 3's headline relation is DEFINITIONAL and is marked so.** A φ floor's control law
   *is* "act to keep `φ ≥ φ_lim`", so *"`φ` never violated ⇒ the floor commands nothing"* is what
   a floor **is**, not something measured. The measured half is which loop **wins the
   min-select**, and only the informative cells are counted (§ 3.1). This document was written
   after `docs/rung78-spec.md`, rung 79 § 5.5 and `docs/rung79-gap-margin.md` § 4.1 recorded
   **four** vacuity traps in a row; the lesson taken from them — *a counter is only as good as
   the NOUN it counts* — is applied here to this document's own counters.

---

## 0. THE SEAM, IN ITS OWN WORDS — AND IT IS UNBUILDABLE AS WRITTEN

Rung 79 § 9, `docs/rung79-gap-margin.md` § 6 and `docs/rungs72-77-march-audit.md` § 6 each name
the same one:

> **A φ-FLOOR MARCH WITH NON-ZERO INITIAL MARGIN.** … a wall **below** the initial operating
> point and **above** the free excursion minimum — `φ_lim ∈ (0.7731, 0.7884)` at these settings,
> a ~1.5% window. It is a **new rig**, not comparable to §§ 1–4.

**The window is inverted, its two endpoints come from a table that does not mean what they were
read as, and the rig it asks for was shipped three rungs ago.** All three clauses are measured
below.

* `0.7731162133` is the **free operating point** (§ 2), so it is the window's *upper* edge, not
  its lower one: a wall above it is **lifted onto** (§ 2), which is the zero-margin condition the
  seam is trying to escape. **The interval as written is the ANTI-window.**
* **Both endpoints are sourced — to `min φ` rows of rung 68 § 7's single-loop ledger** (carried
  into rungs 69 § 8 and 70 § 5). `0.773116` is that table's **F** row (fuel leg alone);
  `0.788430` is its **S** row (stator alone). They are the φ each lever *holds* when it is the
  only one armed — **not** an operating point and **not** an excursion minimum. § 0.1 measures
  why the F row happens to equal `φ(0)`, and why that coincidence is the whole mis-read.
* The correct window is **(free droop, free operating point) = (0.7430, 0.7731)** in `clip` (§ 5),
  and the cell `(clip, 0.76)` inside it **already has all four loops live with non-zero initial
  margin.** It is rungs 75/76's own wall. **The seam is closed by MEASUREMENT, not by plumbing.**

### 0.1 THE SEAM'S PROSE AND THE SEAM'S NUMBERS NAME **DISJOINT** INTERVALS

This is the stronger refutation, because it kills the *construction* and not just the arithmetic.
Bare plant, nothing armed, 341 points:

| quantity | measured here | rung 68 § 7 / rung 70 § 5 row |
|---|---|---|
| `φ_lp(0)` — the initial operating point | **0.7731162133** | **F** (fuel alone) `min φ` = 0.773116 |
| `min φ_lp` — the free excursion minimum | **0.7354416064** (index 47/341) | **bare** `min φ` = 0.735442 |
| free droop with the redline governor live | **0.7429942633** (§ 5) | **G** (governor alone) `min φ` = 0.7430 |
| the seam's upper endpoint | — | **S** (stator alone) `min φ` = 0.788430 |

So the seam's **prose** — *below the initial operating point, above the free excursion minimum* —
names `(0.7354, 0.7731)` bare, or `(0.7430, 0.7731)` in the rig rungs 74–79 actually march (the
redline governor is armed there whatever the φ wall does, so its droop is the G row, not the bare
one). **That is § 5's window, exactly.** The seam's **numbers** name `(0.7731, 0.7884)` — on the
*other side* of its own lower endpoint. The two intervals are disjoint and share one endpoint.

**The mis-read is legible.** In rung 68's rig the shared wall is `0.80`, which is **above** `φ(0)`,
so with the fuel leg alone the plant is in breach from `s = 0` and the best the leg does is hold
`φ` at its initial value — which is why that row's `min φ` **is** `φ(0)` to seven digits. Reading
it as a droop turns an operating point into an excursion floor. The same table's **S** row is the
stator holding its own wall (§ 2's mechanism, one loop), not a floor either. Two holding values
were read as two excursion limits, and the interval between them inverted.

And in the coordinate rungs 74–79 actually argue in, the seam is **not satisfiable at any wall** —
which is § 4, and the reason this document exists.

---

## 1. THE DISCRIMINATOR, CHOSEN BEFORE ANY SWEEP

The tempting first move is to sweep the wall and report where things change. That would have
produced § 4's table and left its *cause* — coordinate or tracking error? — unresolved, because
on the wall axis the two are collinear.

So the discriminator was fixed first: **hold the coordinate AND the wall AND the hardware, and
move ONLY the fuel lag's clock `tau_f`.** That changes the authoritative leg's tracking error and
nothing else. It is § 6, and it is run in **both** coordinates — the null result in `demand`
(§ 6.1) is as load-bearing as the monotone one in `clip` (§ 6.2).

---

## 2. A FLOOR WITH AUTHORITY TO SPARE SITS **EXACTLY** ON ITS WALL

Stator armed **alone**, `φ_lp` read at the initial condition:

| wall | `φ_lp(0)` | `φ_lp(0) − wall` | `v(0)` | `\|v\|/v_max` |
|---|---|---|---|---|
| 0.7731162133 | 0.7731162133 | −1.11e−16 | −0.0000000 | **0.000** |
| 0.775 | 0.7750000000 | 0.0 | −0.004917 | 0.025 |
| 0.780 | 0.7800000000 | +4.44e−16 | −0.017825 | 0.089 |
| 0.7884 | 0.7884000000 | −1.11e−16 | −0.039053 | 0.195 |
| 0.790 | 0.7900000000 | −2.22e−16 | −0.043033 | 0.215 |
| 0.800 | 0.8000000000 | +8.88e−16 | −0.067464 | 0.337 |

**Every wall is landed on to the last bit**, with 3× to 40× the authority left over. And the
**free operating point is `0.7731162133`** — the row where the lift goes to zero *is* the row
where the wall meets the free plant, so the number is read off the lever's own dormancy edge and
not asserted.

**In the four-loop rig the lift is the VALVE's, not the stator's.** At `φ_lim = 0.78` with all
four armed: `b(0) = 0.00962`, `v(0) = −0.0000000`. The stator-only table above is therefore a
**probe of the mechanism, not of the rig** — which is why `0.7884` is not stator *saturation*
(19.5% of authority is not a limit) even though it **is** a stator number: it is rung 68 § 7's
stator-alone `min φ`, i.e. this table's own row 4 seen from the other side. A lever holding its
wall and a lever running out of authority are different facts, and only the first one is here.

---

## 3. THE MEASURED RELATION — AND THE HALF OF IT THAT IS DEFINITIONAL

All four loops armed, lever motion measured **from the initial condition** (`|x(s) − x(0)| >
1e−12`; a bare `x ≠ 0` count reports the IC lift, not the march):

| coord | wall | `min φ_lp` | breach | valve moved | stator moved | `max Tt4` |
|---|---|---|---|---|---|---|
| clip | 0.72 | 0.742994 | **+2.30e−02** | 0 | 0 | 1279.2 |
| clip | 0.74 | 0.742994 | **+2.99e−03** | 0 | 0 | 1279.2 |
| clip | **0.76** | 0.756432 | **−3.57e−03** | **275** | **281** | 1281.1 |
| clip | **0.78** | 0.775300 | **−4.70e−03** | **340** | **314** | 1282.0 |
| clip | **0.80** | 0.795155 | **−4.85e−03** | **340** | **315** | 1283.4 |
| demand | 0.72 | 0.746435 | +2.64e−02 | 0 | 0 | 1198.2 |
| demand | 0.74 | 0.746435 | +6.44e−03 | 0 | 0 | 1198.2 |
| demand | **0.76** | 0.760992 | **+9.92e−04** | **0** | **0** | 1197.7 |
| demand | 0.78 | 0.780000 | −5.55e−16 | 0 | 0 | **1000.0** |
| demand | 0.80 | 0.800000 | −2.22e−16 | 0 | 0 | **1000.0** |

### 3.1 THE COUNT, HONESTLY

Of the ten rows, **six carry no information**: the two `demand` rows at 0.78/0.80 are the arrest
(both sides of the relation at float noise, ±1e−16, and § 4's subject anyway), and the four rows
at 0.72/0.74 have the φ leg **below the plant's own droop** — dormant, so nothing is being
selected at all. **The informative subset is FOUR cells**: `clip` 0.76/0.78/0.80 (leg breaches,
both levers move) and `demand` 0.76 (leg holds, both levers 0/341).

**And one direction of the relation is definitional.** "The plant never violates `φ_lim`, so a
`φ_lim` floor commands nothing" restates the floor's law. What is **not** definitional, and is
what these four cells measure:

> **THE MIN-SELECT'S WINNER DECIDES WHETHER THE OTHER LOOPS ON ITS CONSTRAINT EVER SEE A
> VIOLATION AT ALL.** The airflow levers are not parallel defenders of the wall — they act only
> inside the authoritative leg's **tracking error**, and their entire authority is its failure.

**The wall alone flips it, inside one coordinate** (`clip` 0.74 → 0.76): the coordinate is
*sufficient* but not *necessary*, so this is not a restatement of rung 74's coordinate result.

### 3.2 THE COUNTER IS NOT `docs/rungs72-77-march-audit.md`'s

That audit reports valve 52 / stator 53 at `(clip, 0.76)`; the motion counter here reports
275/281 on the same cell. **Different nouns** — the audit counts its own liveness label, this
counts displacement from the IC. Both report **0/341** on `(demand, 0.76)`, so the finding is
counter-independent; the two columns must not be differenced.

---

## 4. THE ARREST IS AN INTERVAL, AND BOTH EDGES ARE DERIVED

Rung 74 § 2.2 discloses the arrest at the **cell** `φ_lim = 0.80`;
`docs/rungs72-77-march-audit.md` § 4 sharpens that to *"the arrest is a CELL, `(demand, 0.80)`,
and rung 74 owns it."* **It is an interval**, and the cell is one point in it. In `demand`:

| wall | `φ_lp(0)` | `b(0)/b_max` | `max Tt4` | verdict |
|---|---|---|---|---|
| 0.7700 | 0.77311621 | 0.000 | 1195.1 | marches |
| 0.7725 | 0.77311621 | 0.000 | 1044.9 | marches |
| **0.7731** | 0.77311621 | 0.000 | **1001.1** | marches — **the last one** |
| **0.7732** | 0.77320000 | 0.001 | **1000.0** | **ARRESTED — the first** |
| 0.7740 … 0.8400 | = wall | 0.012 … 0.869 | **1000.0** | **ARRESTED** (12 walls) |
| **0.8500** | 0.85000000 | **0.987** | **1000.0** | **ARRESTED — the last** |
| **0.8550** | 0.85500000 | **1.000** | **998.7** | **DECELERATES — the first** |
| 0.8600 … 0.9600 | = wall | 1.000 | 996.9 … 911.8 | **DECELERATES** |

* **LOWER EDGE = THE FREE OPERATING POINT.** Bracketed to `[0.7731, 0.7732]`, and § 2's
  independently-read free point `0.7731162133` lies **inside the bracket**. Below it no lift is
  needed, so the leg opens with margin; above it a floor lifts `φ(0)` **onto** the wall, the leg
  opens **on** its own floor with no authority left, and the accel never starts.
* **UPPER EDGE = THE VALVE'S SATURATION.** Bracketed to `[0.8500, 0.8550]` — `b/b_max` is 0.987
  at the last arrested wall and **exactly 1.000** at the first non-arrested one. Nothing is
  fitted: both edges are read off shipped quantities.
* **AND THE PLANT DOES NOT ESCAPE ABOVE THE UPPER EDGE — IT REVERSES.** Past saturation the
  stator keeps closing (`v(0)` −0.009 → −0.200) and `max Tt4` falls **below** `Tt4_lo`, to 911.8 K
  at `φ_lim = 0.96`. The arrest's upper edge is not a return to normal behaviour; it is the onset
  of a worse one.

**In `clip`, at the SAME walls, there is no arrest anywhere** — 0.7731 / 0.775 / 0.78 / 0.7884 /
0.80 / 0.82 all march to 1281.6–1284.7 K. Rung 74 § 2.2's own sentence, *"the whole accel at that
floor is powered by the clip coordinate's own tracking error"*, is therefore **right and
under-stated**: it holds not at that floor but on the **entire interval**.

---

## 5. THE CONSEQUENCE: THE `demand` COORDINATE HAS **NO** FOUR-LOOP CELL

Put § 3 and § 4 on one axis. In `demand`, for **every** wall:

* **above** the free operating point → the plant is **ARRESTED** (§ 4). There is no trajectory.
* **at or below** it → the leg **HOLDS** its wall (breach `+2.84e−03` at 0.750, `+9.92e−04` at
  0.760, `+5.67e−05` at 0.770, `+2.33e−07` at 0.7731 — positive, i.e. *over*-protecting, at every
  wall measured) → the airflow levers are **INERT, 0/341** (§ 3).

**The dichotomy is exhaustive on the wall axis**, and it was checked at **17 walls spanning
0.72–0.96**, not argued. So:

> **THERE IS NO WALL AT WHICH A `demand` MARCH HAS ALL FOUR LOOPS LIVE.** Every trajectory claim
> rungs 74–79 make in that coordinate is read on an **arrested** plant or on a **two-loop** one
> (governor + φ fuel leg).

`clip`, at the same settings, has the cell the seam asked for — and a whole window of them:

| coord | wall | initial margin | breach | valve moved | stator moved | `max Tt4` |
|---|---|---|---|---|---|---|
| clip | 0.750 | +2.31e−02 | −2.56e−03 | 245 | 251 | 1279.7 |
| clip | 0.760 | +1.31e−02 | −3.57e−03 | 275 | 281 | 1281.1 |
| clip | 0.770 | +3.12e−03 | −4.41e−03 | 300 | 307 | 1281.5 |
| clip | 0.7731 | +1.62e−05 | −4.65e−03 | 306 | 314 | 1281.6 |

with the free droop at **0.7429942633** (`clip`) / **0.7464354455** (`demand`), read off the
walls where the leg is provably dormant. **That is the seam's rig**, and it has been shipped
since rung 75.

**PROVENANCE, so no reader takes one table for a subset of another.** § 3's grid and this table
are **separate runs in separate interpreters**, not one sweep sliced twice. They overlap at one
cell — `(clip, 0.760)` — and agree there on every quoted digit (breach −3.57e−03, valve 275,
stator 281), which is the only cross-run check available and is why the overlap was kept rather
than deduplicated. § 4's `demand` sweep and § 6's two clock sweeps are likewise their own runs;
§ 6.1 and § 6.2 differ **only** in the coordinate, and § 6.3 re-runs § 6.2's own points beside
the second reference so the two columns are read at identical settings.

---

## 6. THE DISCRIMINATOR, RUN — AND ONE HALF OF THE MECHANISM SURVIVES

### 6.1 IN `demand`, THE CLOCK BUYS NOTHING — AND THAT IS RUNG 74's OWN RESULT

`(demand, 0.76)` fixed, `tau_f` swept **25×**:

| `tau_f` | 0.02 | 0.05 | 0.10 | 0.20 | 0.35 | 0.50 |
|---|---|---|---|---|---|---|
| breach | +9.32e−04 | +9.92e−04 | +9.92e−04 | +9.92e−04 | +9.92e−04 | +9.92e−04 |
| levers moved | 0 | 0 | 0 | 0 | 0 | 0 |

No breach at any clock, so no lever ever wakes. This is **not** a failed experiment: rung 74's own
table says the demand target rides the **plant**, so a leg that holds has `cap_dot ≈ 0` and a
first-order lag with nothing to chase has no steady error to accumulate. The null is the
prediction.

### 6.2 IN `clip`, THE BREACH IS MONOTONE IN THE CLOCK, AND THE STATOR FOLLOWS IT

`(clip, 0.80)` fixed — same coordinate, same wall, same hardware, **only** `tau_f` moved:

| `tau_f` | breach | stator moved | `max \|Δv\|` |
|---|---|---|---|
| 0.005 | −1.400e−03 | 340 | 2.454e−02 |
| 0.010 | −2.328e−03 | 332 | 3.008e−02 |
| 0.020 | −3.458e−03 | 322 | 3.456e−02 |
| 0.050 | −4.845e−03 | 315 | 3.796e−02 |
| 0.100 | −5.577e−03 | 312 | 3.884e−02 |
| 0.200 | −6.025e−03 | 311 | 3.899e−02 |

**Monotone over 40× in the clock, and the stator's displacement is monotone with it.** The causal
direction — *more tracking error ⇒ deeper breach ⇒ more lever motion* — is demonstrated **inside
one coordinate**, which the wall axis cannot do.

### 6.3 THE FLOOR THE CLOCK DOES NOT EXPLAIN — AND THE CANDIDATE IS **REFUTED**, NOT UNTRIED

The breach does **not** extrapolate to zero: at `tau_f = 0.005` it is still −1.400e−03. The
obvious candidate is structural rather than temporal — in `clip` the leg's target rides the
**scheduled** fuel, so rung 73's `_ref_law` should carry an offset the clock cannot reach. Run at
`(clip, 0.80)`, both references:

| `tau_f` | breach @ `sched` | breach @ `applied` | `max Tt4` @ `sched` | `max Tt4` @ `applied` |
|---|---|---|---|---|
| 0.005 | −1.400e−03 | **−1.400e−03** | 1284.3 | **1370.9** |
| 0.010 | −2.328e−03 | **−2.328e−03** | 1284.2 | **1413.2** |
| 0.050 | −4.845e−03 | **−4.845e−03** | 1283.4 | 1315.2 |
| 0.200 | −6.025e−03 | **−6.025e−03** | 1283.1 | 1283.3 |

**Identical to every digit printed, while the same knob moves `max Tt4` by up to 129 K.** The
reference is **demonstrably live on the plant and cannot touch the φ breach** — so this is a
refutation with its own positive control, not a null from an inert knob.

**The claim is therefore narrowed, deliberately:** the breach is **monotone in the clock, with a
floor that is not the reference.** *"The tracking error IS the other levers' authority"* is
established as a **direction**, not as a decomposition. The floor is a **declared open
concession** (§ 8), not a rounding detail.

---

## 7. WHAT THIS CORRECTS

* **`docs/rung74-spec.md` § 2.2** — the arrest is disclosed at the **cell** `φ_lim = 0.80`. It is
  the **interval** `(0.7731162133, ≈0.852)`, with both edges derived (§ 4), and the section's own
  sentence about the clip coordinate's tracking error powering the accel holds on all of it.
  **Rung 74 found, mechanised and gated the arrest; this only measures how far it reaches.**
* **`docs/rungs72-77-march-audit.md` § 4 / § 6** — *"the arrest is a CELL"* was true of the cells
  **measured**: that audit's two `demand` points, 0.76 and 0.80, sit either side of the free
  operating point, which is exactly the edge. Its § 3 finding *"liveness is a property of the
  `(coordinate, wall)` pair"* is **sharpened to a mechanism and made exhaustive** (§ 5): in
  `demand` the pair has no live cell at any wall.
* **`docs/rung79-spec.md` § 9 and `docs/rung79-gap-margin.md` § 6** — the seam's window is the
  **anti-window**; both its endpoints are `min φ` rows of rung 68 § 7's single-loop ledger read as
  excursion limits when they are **holding values** (§ 0.1), so its prose and its numbers name
  disjoint intervals; and the rig it asks for is shipped (§ 0/§ 5). **CLOSED, by refuting its
  premise.**
* **Nothing in rung 68 § 7 / 69 § 8 / 70 § 5 is wrong** — those tables report `min φ` and that is
  what they contain. What was missing is that the **F** row's `min φ` is its own initial condition,
  because at a wall above `φ(0)` a fuel leg alone can only hold. That is rung 74 § 2.2's mechanism
  visible one rung early, unremarked (§ 0.1).

## 8. WHAT THIS DOES **NOT** SAY, AND WHAT IS STILL OPEN

* **The `clip` breach floor is unexplained** (§ 6.3). The clock and rung 73's reference are both
  measured; the residual is neither.
* **No claim about rung 65.** *"Bandwidth is a 2nd hardware axis and PURE LOSS"* is about a
  lever's lag costing **its own** protection. That a **different** leg's lag is what gives the
  airflow levers anything to do is a juxtaposition, not a correction, and is not scored as one.
* **One rig, one flight condition, one `b_max`/`v_max` pair.** The upper edge is the **valve's**
  saturation *on this hardware*; a larger `b_max` moves it and nothing here says where.
* **`0.7884` is sourced, not derived here.** It is rung 68 § 7's stator-alone `min φ` (§ 0.1).
  *Why* the stator alone holds at that particular φ is rung 68's question, not this one's; nothing
  here re-derives it, and no verdict here depends on it.
* **STILL OPEN, and now sharper: a four-loop live-wall march in `demand`.** § 5 shows it is
  unreachable on the **shared** wall by construction. The one untried route is to **split the
  wall** — arm the airflow levers ABOVE the fuel leg so the queue's order reverses and they
  become authoritative. That is a real knob (`sm_air` beside `sm`), zero new constants, and it is
  **a rung**, not this document.

## 9. THE GATES

In `tests/test_rung74.py`, per the repo's convention that a margin sweep gates inside the rung's
own file (rungs 28/29):

* **the arrest BRACKET** — 0.7731 marches and 0.7732 does not, with the free operating point read
  independently and asserted to lie between them. A threshold with a **derived** edge, so it
  cannot be satisfied by tuning.
* **the coordinate contrast at a FIXED wall** — `clip` marches where `demand` arrests, which is
  what makes the bracket a statement about the coordinate rather than about the wall.

**Deliberately NOT gated: § 3's relation.** One direction of it is definitional (§ 3.1), so a test
would pass forever and guard nothing — this repo's own recorded failure mode.
