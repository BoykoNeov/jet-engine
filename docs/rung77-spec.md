# Rung 77 — THE STIFFNESS LEDGER

Rung 72–76's six states, four clocks, four loops and three actuators — **every one of them
unchanged**. This rung adds **no knob, no state, no constant and no plant code**. It reads three
solvers this family has run since rungs 46, 48 and 49 and never compared.

    accel (48)  G_a(w) = w − cap(w)             G_a′ = 1 − c        DIMENSIONLESS
    gov   (46)  G_g(w) = Tt4(w) − Tt4_max       G_g′ = ∂Tt4/∂w      K per kg/s
    φ     (49)  G_s(w) = φ_lim − φ_lp(w)        G_s′ = −∂φ_lp/∂w    φ per kg/s

Rung 76 § 8 named the seam: ***the `1/(1−c)` gain as a design variable. Every other set-point
solve in this family (`_topping_fuel`, `_surge_fuel`) has one and it has never been read.***

> **HEADLINE — A SET-POINT SOLVE'S SENSITIVITY IS A FORCING OVER A SLOPE, AND `1/(1−c)` IS THE
> SLOPE HALF OF ONE LEG.** The implicit function theorem gives `dw*/dq = −G_q/G_w` for all
> three legs. Substituting the accel leg's residual returns rung 76 § 3's identity exactly, with
> `1/(1−c) = 1/G_a′` — so **`1/(1−c)` is not a gain a solve BUYS; it is that leg's own residual
> slope**, and it takes that form for one reason: **its set point is a formula for its own
> actuator**. `Tt4_max` and `φ_lim` are CONSTANTS, so `G_g′` and `G_s′` have no `1` to subtract
> from, are dimensional, and have no second reading to difference against. **The other two legs
> have a STIFFNESS but can never have a GAIN.** Rung 76 § 8's wording is **REFUTED**; rung 76
> § 0.1's own sentence — *a floor on a STATE is not a formula for a FUEL* — is what predicts it,
> read one step further.

> **AND THE TWO ROUTES TO A SINGULARITY ARE DIFFERENT ROUTES.** `dw*/dq` diverges iff `G_w → 0`.
> The accel leg reaches that at `c → 1` (rung 76 § 8's fourth seam) and **this family never gets
> there** — `c ≤ 0.2234` over 24 cells. The φ leg reaches it when **another lever pins the
> variable it watches**: rung 64's riding valve, which that rung derived and explicitly marked
> *"DERIVED, not measured."* **Measured here for the first time** — `G_s′` falls from `9.97` open
> to `1.7e−08` closed, and `φ_lp` is `0.8000000000` bit-identical across ±10 % fuel. The
> governor reaches neither: nothing in this family pins `Tt4` at a fixed fuel.

> **AND THE VALVE'S SIGN SPLITS ACROSS THE LEGS.** `dw*/dq` is **negative** for the accel leg
> and the governor and **positive** for the φ leg: opening the bleed valve *tightens* both
> fuel-side caps and *loosens* the one that watches φ. **The single lever that protects the φ
> leg pays for it on both others** — rung 61's *buys back the COORDINATE, not the BILL* and rung
> 64's *a limiter's LAW cannot buy PROTECTION, only its PRICE*, now visible as a sign.

Pre-registration: `docs/plans/rung77-anchor-solve-stiffness.md`. Gates: `tests/test_rung77.py`.

---

## 0. WHAT MOVED, AND WHAT DID NOT

| | rung 75 | rung 76 | **rung 77** |
|---|---|---|---|
| the loops | four | the same four | **the same four** |
| states / actuators | 6 / 3 | 6 / 3 | **6 / 3** |
| what is added | a STOP WITH A RATE | a second READING of a cap | **a READING of all three legs** |
| new constants | `τ_t` | 0 | **0** |
| new knobs | `_windup_law` | `_cap_law` | **none** |
| plant code touched | a leg's law | the plant both legs read | **none at all** |
| the reduce | by dispatch | by dispatch | **by CONSTRUCTION** |

### 0.1 The one override is a CONSTRUCTOR, and it is the trap's fifteenth face

`_shared_rig` builds its machine through `at_lever`, which **names its class literally**. Rungs
61–76 hit the carried-knob trap fifteen times over — a knob (61–75), then a TABLE (rung 76's
`accel_for`). Here the thing that does not travel is the **CLASS**: a subclass that does not
re-declare `at_lever` gets a rig with none of these readers on it, and every number below would
have been taken on a `SensedCapTransient`. It is the only override in the rung.

### 0.2 The settings are rung 76's, taken verbatim

`φ_lim = 0.80` (the inherited Jacobian floor), `margin = 0.10`, `Tt4_max = 1200 K`, all clocks
`0.05`, `ds = 0.005`, read at the `_riding4` points of the `clip | sched | none | solve` march,
every 8th — so the ledger is taken on **rung 76's own plant at rung 76's own settings**, and § 1
can be differenced against its § 3 without rung 63's lesson biting.

---

## 1. MEASURED — THE THREE SLOPES, AND THE ONE THAT IS DIMENSIONLESS

10 riding points, valve and stator **frozen** at the trajectory's states (`_b_state` / `_v_state`)
— the OPEN loop, which is how every reader from rung 64 to rung 76 has read this plant.

| leg | `G_w` raw | unit | normalised by its own set point | `1/‖n‖` |
|---|---|---|---|---|
| accel | `0.80275 … 0.81434` | — | `0.80275 … 0.81434` (`= 1 − c`) | **`1.2280 … 1.2457`** |
| gov | `6.5342e+04 … 7.0341e+04` | K per kg/s | `0.71658 … 0.73210` | `1.3659 … 1.3955` |
| φ | `9.9723 … 11.7196` | φ per kg/s | `0.16637 … 0.17385` | `5.7522 … 6.0105` |

**THE ACCEL COLUMN IS RUNG 76 § 3's MEASURED GAIN, DIGIT FOR DIGIT** — that rung read
`1.22799 … 1.24573` by differencing two *readings* of one cap; this reads `1/G_a′` from one
residual and never touches `solve_gain`. Two independent computations of one number, and it is
what makes this a generalisation of rung 76 rather than a second imposition beside it.

**The instrument:** `‖(1 − G_a′) − c‖ = 7.29e−11` against rung 76's own `_c_at`. Both readings
share a step size so their roundoff cancels — this is a check on the **algebra**, not agreement
to eleven figures, and § 8 scores P1 that way.

### 1.1 The normalisation is the one thing here that could have been an imposition, and it is not

The three raw slopes **carry three different units**, so ordering them is not a comparison. Each
is normalised by **its own set point** — `w`, `Tt4_max`, `φ_lim` — every one that leg's own
already-imposed scalar, and for the accel leg the normalisation is the **identity** (its set
point *is* a fuel). That is why the accel column comes back as `1 − c` exactly rather than as
`1 − c` times something.

**But § 2 does not need it at all**, and that is the stronger answer: `dw*/dq` is a fuel per valve
position for all three legs, so the ledger has a currency that needs no normalisation. § 1's
table survives only because it is what reproduces rung 76.

---

## 2. MEASURED — `dw*/dq`, THE CURRENCY ALL THREE LEGS SHARE

kg/s of set-point fuel per unit valve position, at the same 10 points:

| leg | `dw*/dq` | sign |
|---|---|---|
| accel | `−5.5477e−04 … −2.6428e−04` | **negative** |
| gov | `−1.3781e−03 … −1.1091e−03` | **negative** |
| φ | `+6.1902e−02 … +7.0782e−02` | **positive** |

`‖dw*/dq‖` is ordered `accel < gov < φ` at every point, and the φ leg is **45–64× the governor
and 112–268× the accel leg** — by a wide margin the stiffest thing in this control system.

**D1 HOLDS AS A MEASUREMENT.** `direct` re-solves each leg's whole set point at `q ± dq`; `ift`
differences the two partials separately at the unperturbed set point. Worst relative disagreement
**`7.15e−09`** — see § 2.2 for why that number is arithmetic.

### 2.1 THE SIGN SPLIT, AND ITS MECHANISM

The valve moves the three caps in **two directions**, and each is one station of the same chain:

* **more bleed → less core flow → lower `pt3`** ⇒ the `Wf/pt3` cap **falls**: the accel leg
  tightens.
* **more bleed → less core flow at the same fuel → hotter `Tt4`** ⇒ the governor's set point
  **falls**: it tightens too.
* **more bleed → more surge margin → higher `φ_lp`** ⇒ the φ leg permits **more** fuel: it
  loosens.

**So the one lever that buys the φ leg its protection debits both fuel-side legs, in the
currency they are each written in.** Rung 61 found a compensating lever buys back the coordinate
and not the bill; rung 64 found a limiter's law buys the price and not the protection. This is
the same statement with a **sign** on it, and it is visible only because all three legs are read
in one currency.

### 2.2 The `7.15e−09` is a DIFFERENCING FLOOR, and it was checked rather than asserted

`dq` swept over three decades, worst relative `‖direct − ift‖`:

| `dq` | `1e−6` | `1e−5` | `1e−4` | `1e−3` |
|---|---|---|---|---|
| residual | `9.600e−08` | **`7.152e−09`** | `1.050e−08` | `3.868e−07` |

A textbook central-difference **V**: roundoff-dominated to the left, truncation-dominated to the
right, optimum at `dq = 1e−5`. The gains themselves are stable to six figures across all four.
**The anchor's `< 3e−9` was optimistic by 2.4×** and is scored REFUTED-as-tolerance / HELD-as-law
(§ 8, P2) — rung 76's own P2 precedent, with a fuller justification than that rung had.

---

## 3. MEASURED — THE SINGULAR LIMIT, AND RUNG 64's DERIVATION

`dw*/dq` diverges iff `G_w → 0`. There are two ways to get there and they are **not the same
way**.

| | route | reachable in this family? |
|---|---|---|
| accel | `c → 1` — the set point chases its **own actuator** | **NO**, `c ≤ 0.2234` over 24 cells |
| φ | another lever **pins the variable it watches** | **YES** — rung 64's riding valve |
| gov | neither: nothing here pins `Tt4` at a fixed fuel | **NO** |

**RUNG 64's DERIVATION, MEASURED FOR THE FIRST TIME.** Rung 64 states, in its own words *"DERIVED,
not measured"*, that where the bleed valve rides it re-pins `φ_lp` to `φ_lim` at any fuel, so
`dφ/dW_f = 0` and rung 49's solve is degenerate across its whole bracket. Read at the same states,
OPEN (`_b_state = q`) against CLOSED (`_b_state = None`, the valve re-solving at every trial fuel):

| | measured |
|---|---|
| `‖G_s′‖` OPEN, minimum | **`9.9723`** |
| `‖G_s′‖` CLOSED, maximum | **`1.733e−08`** — `0.0e+00` exactly at 4 of 10 points |
| `φ_lp` offset from `φ_lim` over ±10 % fuel, closed | **`9.99e−16`** |
| `φ_lp` spread over that band | **`1.11e−15`** |

Nearly **nine orders**, and the blunt form is stronger than the derivative: `φ_lp` is
`0.8000000000` at `0.9·w`, at `w` and at `1.1·w`. **This is a CONFIRMATION of a derivation and is
scored as one** — nothing here is a surprise, and § 7 does not report it as one. What is new is
that the degeneracy is **exact**, not approximate.

### 3.1 The governor is the control, and it behaves like one

The same open/closed reading on the governor: `‖G_g′‖` moves by **`2.01e−02` relative**, from a
value never below `6.53e+04`. So closing the valve's loop **does** perturb `∂Tt4/∂w` slightly —
bleed changes `Tt4` at a given fuel — but it does not annihilate it. **2 % against 100 %**: the
φ leg's collapse is its own degeneracy and not an artifact of what closing a loop does to any
residual. Without this control § 3 would not be admissible.

---

## 4. THE ORDER, OVER 24 CELLS — AND THE GUARD THAT MAKES IT LEGAL

Both stator arms × two `φ_lim` (`0.76 / 0.80`) × three `margin` (`0.05 / 0.10 / 0.40`) × two
`Tt4_max` (`1180 / 1200 K`). **24 of 24 cells live.**

**READ RAW, THE ORDER IS NOT INVARIANT** — `('accel','gov','φ')` in 21 cells, `('gov','accel','φ')`
in 3, all at `margin = 0.40`. **P4 as pre-registered is REFUTED.**

**AND THE REFUTATION IS RUNG 76 § 1.3's OWN TRAP, ONE LEVEL OVER.** A margin scan shows why: from
`margin = 0.15` upward the governor's and the φ leg's gains **freeze to identical values**
(`1.10940e−03`, `6.20101e−02`) — the trajectory has stopped depending on `margin` because the
**accel leg has gone dormant** and no longer binds anywhere on it. Above that threshold the raw
ledger is ordering a leg that is *not acting* against two that are. Rung 76 § 1.3 hit exactly
this and fixed it with a switch guard; this is the same guard one level over:

    a leg is LIVE at a point iff  G(mf_sched) > 0        — `_cap_free`'s own test, rung 49's sign

| | raw | **guarded** |
|---|---|---|
| orderings seen | `('accel','gov','φ')`, `('gov','accel','φ')` | `('accel','gov','φ')` ×16, `('gov','φ')` ×8 |
| invariant? | **no** | **yes — the two agree on every pair both contain** |
| φ in top place | — | **24 / 24 cells, and every point within them** |

The `('gov','φ')` cells are the `margin = 0.40` ones with the dormant accel leg **removed**, not
re-ordered. **So the guarded ledger is order-invariant, and the φ leg is the stiffest leg
everywhere.** Reading the UNFLOORED cap is what lets a dormant leg have a slope at all (rung 74's
`_cap_free`, deliberately); ordering that slope against a live one's is the error.

| over the whole sweep | |
|---|---|
| `ift_err`, worst cell | `1.392e−08` |
| normalised-slope separation, worst | `0.0445` |
| governor's normalised slope, minimum | `0.7064` |
| `c`, maximum anywhere | **`0.2234`** |

---

## 5. WHAT THIS DOES TO THE RUNGS BEFORE IT

* **CORRECTS rung 76 § 8's third seam, in its wording and not its spirit.** The other two legs do
  **not** have a `1/(1−c)` and structurally cannot: their set points are constants. What they
  have is `1/‖G_w‖`, of which `1/(1−c)` is the accel leg's instance. Rung 76 § 3's *measurement*
  is untouched and is reproduced here by an independent route (§ 1).
* **CONFIRMS rung 64's derived degeneracy, and gives it its first measurement** (§ 3). Rung 64
  marked it *"DERIVED, not measured"* and declared `removed_together` NOT a result; this measures
  the derivation itself, which is the part that was never in doubt but was never read.
* **UNIFIES rung 64's *deleted plant* with rung 76's *`c → 1`* as ONE object** — `G_w → 0` —
  reached by two different mechanisms, one of which this family reaches and one of which it does
  not (§ 3).
* **BOUNDS rung 76 § 8's fourth seam before it is built.** `c → 1` is not reachable by any
  setting this family already has: `c ≤ 0.2234` over 24 cells, so the divergent-gain rung needs a
  **new** schedule reference and cannot be had by turning `margin` up.
* **SHARPENS rungs 61 and 64 with a SIGN** (§ 2.1): the valve loosens the leg that watches φ and
  tightens both that do not, which is what *buys the coordinate, not the bill* looks like when
  all three legs are read in one currency.
* **CONFIRMS rung 76 § 1.3's `margin ≈ 0.20` threshold from a different instrument** (§ 4) — that
  rung found the knob inert above it, this one finds the leg dormant above it, and the margin
  scan puts the crossing between `0.15` and `0.20` on both readings.

---

## 6. CONCESSIONS (in addition to every one rungs 62–76 list, all inherited)

* **THIS RUNG MEASURES SOLVERS, NOT PHYSICS.** `G_w` is a property of how a limiter is *written*.
  That is the point — rung 76 § 8 asked for it — but no thermodynamic claim is made anywhere
  here, and the ledger would read differently on the same engine with the same limiters coded a
  different way. That is the finding, not a defect, and it is stated as a concession because it
  bounds what § 2's ordering means.
* **§ 1's NORMALISATION IS A CHOICE**, though each scale is the leg's own already-imposed scalar
  and the accel column's is the identity. **§ 2 is the answer to it** and needs no normalisation;
  every ordering claim in § 4 is taken in § 2's currency, never § 1's.
* **THE ORDERING IS READ AT EACH LEG'S OWN SET POINT**, which is the natural point and is not the
  same point for the three legs. A ledger read at one common `w` is a different object and is not
  measured here.
* **`dw*/dq` IS ONE DIRECTION.** `q` is the valve, chosen for rung 76 § 3's reason (the one other
  state the caps reach through the plant). The stator and the two speeds are unmeasured, so
  "stiffest leg" means stiffest **to the valve**.
* **THE GUARD IS NECESSARY AND IS DISCLOSED** (§ 4). Under it P4 holds; without it P4 is refuted.
  Both readings are reported and the guarded one is not presented as the raw one.
* **`margin`, `φ_lim`, `Tt4_max`, `b_max`, `v_max` remain rungs 48/67's imposed values**, taken
  verbatim so § 1 differences against rung 76 § 3 (§ 0.2).
* **RUNG 64's CONTROL IS 2 %, NOT ZERO** (§ 3.1). Closing the valve's loop is not perfectly
  neutral on the governor's residual; the claim rests on the ratio to the φ leg's ~9 orders, not
  on the governor being untouched.
* **§ 3.1's CONTROL IS `2.01e−02` AT ONE SETTING, NOT A BOUND.** The rung's
  admissibility rests on the ratio between it and the φ leg's ~9 orders, and the gate
  on it (`< 0.1`) is the loosest in the file. The mechanism is real — bleed moves `Tt4`
  at fixed fuel — so the 2 % can drift with settings; what cannot drift to 100 % is the
  φ leg's exact zero.
* The STAGE STACK (55/56) is still off the transient ladder, and this does **not** close rung
  63's *fuel + bleed + STATOR*.

---

## 7. THE REDUCE — BY CONSTRUCTION, AND THERE IS NOTHING FOR IT TO MISS

`StiffnessLedgerTransient` overrides **no plant method** — only `at_lever`, a constructor, and it
constructs the same machine with the same five knobs carried. So every march it runs is
`SensedCapTransient`'s **bit-for-bit**, on all five of rung 76's live cells and on the accel-armed
φ arm, and this is **not a tolerance**: the parent's code is the code that runs.

Gated non-vacuous on rung 73's `charpoly_selftest` discipline, and the gate is § 1's own
separation: **the three normalised slopes differ by more than `1e−2` at every point in every
cell** (worst `0.0445`), so the reader is not reporting one quantity three times. A ledger whose
three columns agreed would pass a bit-for-bit reduce and mean nothing.

### 7.1 AND UNLIKE RUNG 76 THERE IS NO PARENT EDIT, WHICH IS WHY A KNOB-LESS REDUCE IS DEFENSIBLE

Rung 76 § 6.1 had to check its one parent edit against `HEAD~1` in a `git worktree` — 229,152
floats — because every reduce arm ran with both sides post-change and `test_numeric_fingerprint.py`
stops at rung 66. **This rung edits no line outside its own class**, so there is no such
exposure and no worktree check is needed. The fingerprint gap (CLAUDE.md § Open engineering
tasks) is untouched by this rung and **is not closed by it** — it remains open for rungs 67–76.

---

## 8. THE ANCHOR, SCORED

| | claim | verdict |
|---|---|---|
| D1 | `dw*/dq = −G_q/G_w`, all three legs | **HELD** — measured, § 2 |
| D2 | `1/(1−c) = 1/G_a′` is the accel leg's instance of D1 | **HELD** — § 1 reproduces rung 76 § 3 digit for digit by an independent route |
| D3 | the other two legs cannot have a `1 − c` form | **HELD** — their set points are constants; § 1's units table is the evidence |
| D4 | two routes to `G_w → 0`; this family reaches one | **HELD** — § 3 |
| D5 | the governor cannot go singular at all | **HELD** — `‖G_g′‖ ≥ 6.53e+04`, and § 3.1's control |
| P1 | the instrument reproduces rung 76 to `< 3e−9` | **HELD** — `7.29e−11` |
| P2 | D1 holds as a measurement to `< 3e−9` | **HELD AS LAW, tolerance REFUTED** — `7.15e−09` at the optimal step, and § 2.2's V-curve shows the residual is arithmetic. The law was not optimistic; the tolerance was, by 2.4× |
| P3 | the order is `φ ≫ gov > accel` | **HELD** — φ ~50× the governor, ~130× the accel leg |
| P4 | the order survives ≥ 12 combinations with none inverting | **REFUTED RAW, HELD GUARDED** — 3 of 24 cells invert, all at `margin = 0.40`, and § 4 shows every one is a **dormant** accel leg. Under rung 76 § 1.3's own guard: invariant, 24/24 |
| P5 | the three slopes are separated by `> 1e−2` | **HELD** — worst `0.0445` over the sweep |
| P6 | `G_s′` closed `< 1e−7`, open `> 1.0` | **HELD** — `1.73e−08` against `9.97`, and `φ_lp` bit-identical over ±10 % fuel. **The SHIPPED GATE is `1e−6`, not the pre-registered `1e−7`**: differencing `φ ≈ 0.8` at `dw ≈ 1e−8` has a roundoff floor of `≈ 1.8e−08`, so the tolerance that HELD has only ~5× headroom and would ship as a flake. P2's lesson applied forward instead of scored afterwards |
| P7 | the governor's normalised slope `> 0.5` everywhere | **HELD** — minimum `0.7064` |
| P8 | `c` stays below `0.35` | **HELD** — `0.2234` |
| P9 | the reduce is exact and knob-less | **HELD** — no plant method overridden |
| P10 | nothing in the parent moves | **HELD** — by diff |

**One refutation and one split verdict from ten predictions.** P2's refutation is a *tolerance*
and § 2.2 replaces it with a measured differencing floor. **P4's is the substantive one, and it
became § 4**: the raw ordering really is not invariant, and finding out *why* recovered rung 76
§ 1.3's guard in a place nothing had asked for it — the second time in two rungs that this
family's `min`-select has produced a comparison between a leg that acts and one that does not.

**AND THE TRAP DID NOT BITE, FOR THE SECOND TIME IN SIXTEEN RUNGS** — but only because it was
looked for in a new place. `_cap_law`, `_ref_law`, `_lag_coord`, `_windup_law` and `_ic_cap` all
travel through `at_lever`; what would have leaked is the **class**. What bit instead was its
third cousin: a residual **closure** carried out of the `_b_state` block it was built in, so both
`q ± dq` readings landed on the same closed-valve plant and `G_q` came back identically zero. The
tell was the number — a relative error of `1.000e+00` exactly, with no noise in it. It is fixed
structurally (`_residuals` exists so a closure cannot outlive its block), not by moving a line.

---

## 9. NEXT SEAMS

* **THE OTHER THREE DIRECTIONS.** `dw*/dq` is measured for the valve alone (§ 6). The stator and
  the two spool speeds are the rest of the gradient, and "stiffest leg" is only a claim about `q`
  until they are read.
* **A CAP WHOSE `c` APPROACHES 1**, still open and now **bounded**: § 4 shows `margin` cannot get
  there, so it needs a schedule referenced to a quantity the fuel moves harder than `pt3`.
* **THE LEDGER AT ONE COMMON `w`.** Every slope here is read at its own leg's set point (§ 6). A
  ledger read at the applied fuel is a different object and would say what the legs' stiffnesses
  are *where the engine actually is*, not where each leg would put it.
* **A SENSED CAP ON A MASKED LEG** — rung 76 § 8's first seam, untouched here, and this rung
  supplies the reason it needs a **sensed governor**: § 1's D3 says why the governor has no
  sensed form, so building one means changing what its set point *is*, not how it is read.
* **THE STIFFNESS AS A DESIGN VARIABLE, ACTUALLY EXERCISED.** This rung measures stiffness; it
  never changes one and watches the plant respond. A rung that re-writes one leg to a different
  `G_w` at the same set point would separate *stiffness* from *set point* — and § 2.1's sign
  split says the two fuel-side legs would move together and the φ leg against them.
* Everything rungs 72–76 § 8–11 leave, unchanged by this rung.
