# Rung 81 — THE AUTHORITY CLOCK

**The seam:** `docs/rung80-spec.md` § 10, first bullet — *"A `demand` FOUR-LOOP CELL WITH THE φ
FUEL LEG AUTHORITATIVE. All 11 measured have the governor holding."*

**Plant:** `AuthorityClockTransient` (`turbojet/engine.py`) — **no state, no knob, no constant**
(rung 77's precedent). **Anchor + pre-registration:** `docs/plans/rung81-anchor-authority-clock.md`.
**Gates:** `tests/test_rung81.py`.

---

## 0. HONESTY, FIRST — WHAT THE PRE-CHECK ALREADY KNEW, AND HOW THE FIVE PREDICTIONS SCORED

**The seam's own question was answered BEFORE the anchor was written.** A § 0 pre-check on rung
80's shipped plant (rung 65's precedent) found the mirror cell — 7 points, all `authority = fuel`,
at clocks `(0.20, 0.01, 0.50, 0.05)`. The anchor's § 0 records this and takes no credit for it.
What it could **not** say is which of the three clocks it moved did it; that is this rung's § 1.

| # | predicted | measured |
|---|---|---|
| **P1** | the § 1 criterion predicts the authority label at **≥ 95 %** of points, misses sitting where the two sides are **within 10 %** | **SPLIT.** Threshold **CONFIRMED** — 1 045 of 1 054 (**99.15 %**), worst single cell **95.24 %**. Tie-locality clause **REFUTED at its own line**: 3 of the 9 misses exceed 10 %, worst **11.77 %** (§ 2) |
| **P2** | `τ_f` is the lever; `τ_q` is not | **CONFIRMED.** With `τ_q` pinned at rung 80's 0.05 the fuel region still opens, from `τ_f = 0.08` (§ 1). The pre-check's valve clock was not the cause |
| **P3** | in `clip` the fuel region lies on the **opposite side of the diagonal** | **WORDING REFUTED, MECHANISM CONFIRMED.** At the split wall `clip` has **no** fuel region at all — 0 of 18 cells. The sign flip appears only on the **control arm** (§ 3): at the shared wall, slowing the fuel leg 10× takes it from 13 fuel-held points to 3 |
| **P4** | the mask is symmetric; `n_live ≤ 3` a seventh time | **CONFIRMED.** `mask_leak == 0` **exactly** on both sides of the switch, one authority per interior point, `ever_two_authorities` False (§ 4) |
| **P5** | `zeros` moves by one at fuel-authority points (rung 72 § 3, in `clip`) | **REFUTED** — `zeros = {1}` in **both** regimes. Reported, never gated, exactly as registered |

**AND THE HEADLINE IS NONE OF THEM.** It is § 3's unpredicted exact invariance.

**Two disclosures that are not softened.** P1's worst cell cleared its registered bar by **0.24
percentage points** — a finer grid could push it under, and the constant was registered, not
tuned. And the 10 % clause is **refuted at 11.77 %**; it is therefore **not gated** — gating a
refuted clause at its measured value is fitting the test to the result.

---

## 1. THE CRITERION, AND THE GRID — § 1 MEASURED

Rung 74's `demand` law is `dw/ds = (c − w)/τ` with `w` the fuel a leg ALLOWS and `c` its cap;
`min` gives the actuator to the smaller `w`. A first-order lag tracking a ramp sits below its
target by `τ·dc/ds`, and `required = mf_sched − c` turns the cap difference into the demand
difference, so — with **nothing added, no constant and no knob**:

> **THE FUEL LEG HOLDS THE ACTUATOR IFF**
>
>     required_gov − required_fuel   <   τ_f · ċ_f  −  τ_gov · ċ_r
>          [the SET-POINT gap]              [the LAG-ERROR gap]

`τ_q = τ_s = 0.05` (rung 80's), `φ_lim = 0.75`, `φ_air = 0.77`, `ds = 0.005`, 341 points. Cells
are **fuel-held / four-loop points**; every row had `riding4_valid` True:

| `τ_f` \ `τ_gov` | **0.02** | **0.05** | **0.20** |
|---|---|---|---|
| 0.02 | 0 / 30 | 0 / 31 | 0 / 30 |
| 0.05 | 0 / 32 | **0 / 33** ← rung 80's own cell | 0 / 32 |
| 0.08 | 11 / 39 | 9 / 39 | 5 / 36 |
| 0.10 | 23 / 43 | 20 / 42 | 14 / 39 |
| 0.12 | 34 / 46 | 31 / 45 | 22 / 42 |
| 0.20 | **47 / 47** | **47 / 47** | **47 / 47** |

* **THE CONTROL REPRODUCES RUNG 80 EXACTLY** — `(0.05, 0.05)` returns **33** four-loop points,
  all `gov`. Same rig, same walls, same number; the grid is the shipped plant.
* **P2 HOLDS: `τ_q` NEVER MOVED.** The fuel region opens at `τ_f = 0.08` with the valve clock
  pinned at 0.05 throughout. The pre-check's three-clock step was not needed.
* **THE THRESHOLD IN `τ_f` IS CROSSED AT EVERY `τ_gov` ON THIS GRID** — so the region spans
  **both sides of the diagonal** (`fast_fuel`, `matched` and `slow_fuel` all present), which
  *"a slower leg wins"* does not describe. **But `τ_gov` still modulates how much of the window
  is fuel-held** — 11/9/5 at `τ_f = 0.08`, 34/31/22 at 0.12, a ~2× spread at fixed `τ_f`. The
  region is a threshold in `τ_f` that `τ_gov` shifts, not a knob `τ_gov` is absent from.
* **AND THE SLOPES SAY WHY.** Measured `ċ_f/ċ_r = 1.7 … 5.0` across the grid: the fuel leg's cap
  moves several times faster than the governor's, so `τ_f·ċ_f` beats `τ_gov·ċ_r` **at equal
  clocks**. The race is between two *products*, and the plant supplies a lopsided pair.

---

## 2. THE CRITERION SCORED, AND WHERE IT MISSES — § 2 MEASURED

1 054 points scored over 36 grid cells (every four-loop point with a central difference; **0**
dropped as trajectory edges, and `_riding4` guarantees both demands positive, so the anchor's V5
exclusion never fired).

| | |
|---|---|
| agreeing | **1 045 / 1 054 = 99.15 %** |
| worst single cell | **95.24 %** (`τ_f = 0.10`, `τ_gov = 0.05`) |
| disagreements | **9**, every one in a transition row (`τ_f` = 0.08 / 0.10 / 0.12) |
| every miss is | `predicted = fuel`, `measured = gov` — the criterion is **early**, never late |
| worst miss margin | **0.1177** — 3 of 9 above the registered 0.10 |
| closest scored point | `\|margin\| = 9.3e-04` (`τ_f = 0.10`, `τ_gov = 0.20`) — and it is a **miss** |

**THE GRID WAS REFINED BECAUSE THE FIRST ONE COULD NOT FAIL.** A `(0.02, 0.05, 0.20)` axis
returned 506 of 506 with the **closest point 25 % from the tie** — a perfect score on a test with
no hard cases in it. Adding 0.08 / 0.10 / 0.12 put points at `9e-04` of the tie and produced the
nine misses. **The 100 % was the weaker measurement**, and it is recorded here so the 99.15 % is
not read as a degradation.

The misses are the quasi-steady form's own edge: `w ≈ c − τ·ċ` is exact only where `ċ` is
constant, and every miss sits inside a transition window where it is not. That is a **disclosed
approximation in the criterion**, not a defect in the plant — and the direction is consistent
(the lag-error gap is over-estimated near the tie, so the criterion calls `fuel` a few points
early).

---

## 3. THE HEADLINE — A MASKED LEG'S CLOCK IS AN EXACT NULL KNOB — § 3 MEASURED

Nothing predicted this, and it is the strongest reading in the rung. Each `(coord, τ_gov)` column
of § 1's grid holds six marches differing **only** in `τ_f`, over a **10× range**. Compared
**bit-for-bit**, all 341 points × `φ_lp`/`Tt4`/`b`/`v` = **1 364 floats**:

| column | differing floats, worst pair | four-loop counts across the column |
|---|---|---|
| `demand` @ `τ_gov` = 0.02 | **1 304 / 1 364** | 30, 32, 39, 43, 46, 47 |
| `demand` @ 0.05 | **1 304 / 1 364** | 31, 33, 39, 42, 45, 47 |
| `demand` @ 0.20 | **1 304 / 1 364** | 30, 32, 36, 39, 42, 47 |
| `clip` @ 0.02 | **0 / 1 364** | 27 (all six) |
| `clip` @ 0.05 | **0 / 1 364** | 23 (all six) |
| `clip` @ 0.20 | **0 / 1 364** | 9 (all six) |

> **HEADLINE — A LEG THAT NEVER HOLDS THE ACTUATOR HAS NO CLOCK.** In `clip` at the split wall
> the fuel leg is masked for the whole ramp, and a **10× sweep of its own time constant moves not
> one bit** of the engine's trajectory. This is rung 72's *"`min` is flat in the masked leg"* —
> until now a zero in a Jacobian, read off a linearisation — **promoted to an exact invariance of
> the plant itself**, at three governor clocks, on 1 364 floats each.

**AND THE CONTROL MAKES IT A STATEMENT ABOUT MASKING, NOT ABOUT `clip`.** At the **shared** wall
`clip` is the one cell in this family where the fuel leg does take the actuator (rung 80 § 5), and
there the identical sweep is **live**:

| `τ_f` | 0.02 | 0.05 | 0.08 | 0.10 | 0.12 | 0.20 |
|---|---|---|---|---|---|---|
| fuel-held points | **13** | 7 | 5 | 5 | 4 | **3** |
| `max Tt4` | 1279.712860 | 1279.674497 | 1279.666919 | 1279.664847 | 1279.663600 | 1279.661553 |

**Inert where the leg is masked, live where it is not — the same knob, the same coordinate, the
same rig.** So the null result above is not `clip` being insensitive; it is masking being total.

**AND THIS IS WHERE P3's MECHANISM SURVIVES ITS OWN WORDING.** That table falls **monotonically**:
in `clip` a **slower** fuel leg has **less** authority (13 → 3), while in `demand` the same
slowing takes it from **0 → 47**. The sign of the `τ_f` term flips with the coordinate exactly as
§ 1's derivation says it must — but it had to be read on a **control arm**, because the split wall
leaves `clip` with no fuel region to compare. The registered prediction was made about the split
wall and **failed there**; the confirmation is the control's, and is reported as such.

---

## 4. THE MIRROR MASK — RUNG 72's BLOCK ON THE OTHER SIDE OF THE SWITCH — § 4 MEASURED

Every mask measurement rungs 72–80 made had the governor holding. The identical twelve-gain
instrument, `coord = demand`, `φ_lim/φ_air = 0.75/0.77`, every four-loop point:

| clocks | riding | interior | authority | `max \|mask_leak\|` | `max \|cyclic\|` | `zeros` |
|---|---|---|---|---|---|---|
| `(0.20, 0.05, 0.05, 0.05)` | 47 | **47** | `fuel` × 47 | **0.0 exactly** | **1.043759340** | {1} |
| `(0.05, 0.05, 0.05, 0.05)` | 33 | **33** | `gov` × 33 | **0.0 exactly** | **0.0 exactly** | {1} |

* **P4 HOLDS, AND THE BLOCK IS SYMMETRIC.** One authority per interior point on both sides,
  `mask_leak` exactly zero on both, `ever_two_authorities` **False**. `n_live ≤ 3` a **seventh**
  time — and for the first time measured with the *other* leg masked.
* **THE ZERO IS FALSIFIABLE ON ONE CODE PATH, which is better than rung 80 could manage.** Rung
  80's control for `cyc = 0` had to be imported from a different coordinate's arm. Here both
  branches sit in one table: the three-φ-loop cycle runs through the fuel leg, so it is **exactly
  0** when that leg is masked and **1.0438** when the governor is. An instrument that produced
  only zeros would be indistinguishable from one that measured nothing; this one produces both.
* **`all_differenced`**: `skipped = {switch: 0, regime: 0}` on both arms — 47 and 33 points, not
  a subset.
* **P5 REFUTED.** `zeros = {1}` in both regimes: it does **not** move by one. Rung 72 § 3's
  reading was taken in `clip` at unmatched clocks and is not contradicted for that cell; it simply
  does not carry to `demand`. **Reported, not gated** (rung 80 § 8's discipline).

---

## 5. WHAT THIS SAYS, AND WHAT IT CORRECTS

> **AUTHORITY IS DECIDED BY THE LAG, NOT BY THE SET POINT.** In every fuel-authority cell here
> the governor's own limit is still the more severe one — `required_gov > required_fuel`
> throughout — and it loses the actuator anyway, because the fuel leg's state sits further below
> its target. **The leg demanding the deeper cut is not the leg setting the fuel.**

**AND THAT SENTENCE IS MEASURED, NOT INFERRED.** Over the whole fuel-authority region — **310 of
310** scored points, every `τ_f ≥ 0.08` row at all three governor clocks — the set-point gap is
**strictly positive** (`+1.08e-03 … +2.08e-03`) while the lag-error gap that beats it runs
`+1.87e-03 … +2.88e-03`. Not one point has the fuel leg winning by *also* demanding more. Had the
two nouns agreed anywhere, this rung would be reporting arithmetic; they disagree everywhere, and
that is gated.

* **CORRECTS `docs/rung74-spec.md`'s headline reading.** Rung 74 established that a state's
  coordinate is *pure bill — no rank, all trajectory*, and that is right: `τ·ċ` is
  state-independent and appears in no Jacobian. What this rung adds is that the same bill
  **selects which leg is masked**, and the masked leg's whole column is the zero one. So a
  quantity with no rank of its own **decides the membership of the live set** — and § 3 shows the
  consequence at its most extreme, a real time constant that is exactly inert.
* **EXTENDS `docs/rung72-spec.md`.** Its block is confirmed on the mirror cell and, in § 3,
  strengthened from a Jacobian zero into a bit-exact plant invariance.
* **BOUNDS `docs/rung80-spec.md` § 5's table.** Its eleven `demand` cells all had the governor
  holding — **not because `demand` cells are like that**, but because every one was measured at
  matched clocks, which § 1 now shows is on the `gov` side of a threshold in a knob rung 80 never
  swept. Rung 80's numbers stand; their generality does not.
* **Nothing in rungs 68 § 7 / 72 / 76 is wrong.** One authority per point and `mask_leak = 0` are
  rung 72's results and they survive the switch being thrown.

---

## 6. WHAT THIS DOES **NOT** SAY

* **No rank claim, and no new one.** `n_live ≤ 3` is rung 72's, by `min`'s flatness in the masked
  leg; this rung tests only that it is symmetric in *which* leg that is. `zeros` is reported.
* **The criterion is QUASI-STEADY and its misses are disclosed** (§ 2). It is a reading of rung
  74's law, not an identity, and it is early near the tie by up to 11.77 % of the larger term.
* **`τ_gov` IS NOT ABSENT.** § 1's threshold is crossed at every `τ_gov` measured, but the size of
  the fuel window varies ~2× with it. "A threshold in `τ_f` alone" would be an over-claim.
* **One rig, one wall pair, one flight condition, one `ds`.** The `τ_f = 0.08` threshold is this
  hardware's and this ramp's; the criterion is what is meant to travel, not the number.
* **The `clip` null is a null at THIS wall.** § 3's control shows the same knob live at the
  shared wall; nothing here says `clip` is insensitive to `τ_f` in general.
* **`τ_gov = 0.20` blows the redline** (`max Tt4` = 1444 K in `clip`) — rung 47's *cost of
  realism*, reproduced incidentally and not pursued.

---

## 7. THE REDUCE CONTRACT, AND THE GATES

**REDUCE — an IDENTITY, because the rung adds nothing.** At rung 80's clocks and walls,
`AuthorityClockTransient`'s march is `SplitWallTransient`'s **bit-for-bit**: 341 points × 8 keys
(`φ_lp`, `Tt4`, `b`, `v`, both legs' states, both demands), **0 differing values**. A reader-only
rung whose march moved would be a rung-80 regression wearing a new class name.

**GATES** (`tests/test_rung81.py`):

1. **the reduce**, above — exact.
2. **the rung-80 control**: `(0.05, 0.05)` gives **33** four-loop points, **all `gov`**.
3. **the seam's cell exists**: `τ_f = 0.20` gives four-loop points with the **fuel leg holding**,
   with `riding4_valid` True — the seam's own object, and the first in this family.
4. **`τ_q` is not the cause** (P2): the fuel region opens with the valve clock pinned at 0.05.
5. **the two nouns DISAGREE** (§ 5): at every fuel-held point the **governor's** demand is still
   the larger — a single non-positive set-point gap would make the headline arithmetic.
6. **THE NULL KNOB, both arms** (§ 3): every `clip` split-wall column **bit-identical** across a
   10× `τ_f` sweep (`n_differing == 0` of 1 364), **and** the shared-wall control **live**
   (`max Tt4` moves, fuel-held count falls 13 → 3). Neither alone is the gate.
7. **the mirror mask** (P4): `mask_leak == 0` **exactly** with both regimes present,
   `ever_two_authorities` False, `all_differenced`, and the cyclic discriminator **split**
   (`0.0` where the fuel leg is masked, `> 0` where the governor is).
8. **the criterion** (P1): worst cell **≥ 95 %**, at the registered bar and not at the measured
   value.

**Deliberately NOT gated:** `zeros` (P5, refuted and reported), the cyclic product's *value*, the
10 % tie-locality clause (**refuted** — gating it at 12 % would fit the test to the result), and
any count on a row with `riding4_valid` False.

---

## 8. NEXT SEAMS

* **THE THRESHOLD'S OWN LAW.** § 1 locates it between `τ_f = 0.05` and 0.08 on this ramp. The
  criterion says it should move with `ċ_f/ċ_r`, i.e. with the schedule's slope and the wall pair
  — untested, and it is the sweep that would turn the criterion from a label-predictor into a
  quantitative one.
* **A CELL WITH THE FUEL LEG HOLDING *AND* THE VALVE SATURATED.** Rung 80 § 4's saturation edge
  was measured with the governor holding throughout; whether it moves with the switch is untested.
* **THE FOURTH LOOP, STILL.** `n_live = 4` is unmoved and unreached: this rung throws the switch,
  it does not remove it. Rung 80 § 10's second bullet stands verbatim — a non-`min` composition or
  a fourth lever off the fuel actuator.
* **A FINGERPRINT ARM FOR THIS RUNG.** `tests/test_numeric_fingerprint.py` has no `r81`. The
  natural lead is § 3's `n_differing`, which is **discrete and structurally zero** — the shape
  slice 4 handled for rung 79, and it needs that slice's two-sided reasoning, not a relative leg.
* Everything rungs 72–80 §§ 8–11 leave, unchanged by this rung.
