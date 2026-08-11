# Rung 81 anchor — THE AUTHORITY CLOCK

**Written and committed BEFORE any rung-81 sweep ran.** Its § 0 declares, separately and first,
the four facts a § 0 pre-check had already established — so no prediction below may claim them.

**The seam:** `docs/rung80-spec.md` § 10, first bullet — *"A `demand` FOUR-LOOP CELL WITH THE φ
FUEL LEG AUTHORITATIVE. All 11 measured have the governor holding. Whether the φ leg can hold the
actuator while all four ride is untested, and it is the cell § 5's table cannot reach."*

**Plant:** `AuthorityClockTransient` (`turbojet/engine.py`), a reader-only rung on rung 80's
machine — rung 77's precedent (*"no knob, constant or plant code"*). **Gates:** `tests/test_rung81.py`.

---

## 0. WHAT THE PRE-CHECK ALREADY ESTABLISHED — NOT PREDICTIONS, AND NOT THIS RUNG'S CREDIT

Rung 65 § 0's precedent: orientation on an ALREADY-SHIPPED plant, at its own settings, before the
anchor. It read rung 80's own cells per point. Four facts, and the seam's own question is among
them:

* **E1 — the governor does not win by a hair.** In `demand` at matched clocks `(0.05,0.05,0.05,0.05)`,
  rung 80 § 5's own two rows (`φ_lim = 0.75`, `φ_air = 0.77` and `0.80`) have `required_gov >
  required_fuel` at **every** four-loop point, by 145× falling to 1.55× across the window; the state
  gap `w_fuel − w_gov` is **+2.1 %…+22.7 %** of the scheduled fuel. All 33 / 19 points: `gov`.
* **E2 — THE SEAM'S CELL EXISTS, and the pre-check found it.** At clocks `(0.20, 0.01, 0.50, 0.05)`
  — slow fuel leg, fast governor, slow valve — the **same walls** give a `demand` four-loop cell of
  **7 points, all seven `authority = fuel`**. **The existence question is therefore ANSWERED BEFORE
  THIS ANCHOR, and this rung claims no credit for it.**
* **E3 — the mirror clock setting `(0.01, 0.20, 0.50, 0.05)` gives an EMPTY cell** (0 points), so
  the effect is one-sided in the clocks and not a widening of the window.
* **E4 — AND THE TWO NOUNS DISAGREE IN SIGN, which is why there is a rung here at all.** In that
  fuel-authority cell the **set points** still favour the governor (`required_gov − required_fuel =
  +8.8e-4 … +1.04e-3`, positive throughout) while the **states** hand the actuator to the fuel leg
  (`w_fuel − w_gov = −9.7e-4 … −9.2e-4`, negative throughout). The leg that *wants* the deeper cut
  is **not** the leg that *gets* to set the fuel.

**AND THE PRE-CHECK CANNOT SAY WHICH KNOB DID IT.** Its `(0.20, 0.01, 0.50, 0.05)` arm moved the
fuel clock, the governor clock **and the valve clock** (0.05 → 0.50) in one step. Separating them
is this rung's first job, and until it is done E2 is an existence result and nothing more.

---

## 1. THE MECHANISM BEING TESTED, DERIVED — ZERO NEW CONSTANTS

Every fuel-side leg in `demand` (rung 74) carries `dw/ds = (c − w)/τ`, `w` the fuel it ALLOWS and
`c` its cap. On a ramp with `ċ = dc/ds` the quasi-steady tracking error of a first-order lag is

    w  ≈  c − τ · ċ                                             [the LAG ERROR]

and `_demand_authority` gives the actuator to the leg with the **smaller** `w`. Subtracting the two
legs, and using `required = mf_sched − c` so that `c_f − c_r = required_gov − required_fuel`:

> **THE FUEL LEG HOLDS THE ACTUATOR IFF**
>
>     required_gov − required_fuel   <   τ_f · ċ_f  −  τ_gov · ċ_r
>          [the SET-POINT gap]              [the LAG-ERROR gap]

A race between two differences. Both sides are already in the shipped trajectory dict
(`required_fuel`, `required_gov`, `cap_fuel`, `cap_gov`; `ċ` by finite difference along `s`), so
the criterion introduces **no constant, no knob and no new plant code** — it is rung 74's own law
read as an inequality.

**AND IT IS RUNG 74's BILL.** `τ·ċ` is exactly the ramp-tracking error rung 74 named *the BILL*,
measured there as exceeding the fuel leg's entire clip. Rung 74's headline is that a **state's**
coordinate is *pure bill — no rank, all trajectory*. If the criterion above governs, that bill
**selects which leg is masked**, hence which rows of the Jacobian are identically zero — and a
quantity with no rank of its own decides the membership of the live set.

---

## 2. THE PREDICTIONS, IN THE ORDER THEY WILL BE SCORED

**P1 — THE CRITERION GOVERNS, POINT BY POINT.** Over a clean `(τ_f, τ_gov)` grid with the valve and
stator clocks held FIXED, the sign of `[τ_f·ċ_f − τ_gov·ċ_r] − [required_gov − required_fuel]`
predicts the measured `authority` label at **≥ 95 %** of four-loop points, and the residual
disagreements sit where the two sides are within 10 % of each other (the quasi-steady
approximation's own edge). Predicted **≥ 95 %**, not 100 %: `ċ` is not constant across the window.

**P2 — τ_f IS THE LEVER AND τ_q IS NOT.** With `τ_q` held at 0.05 the fuel-authority cell still
opens at large `τ_f`; the valve clock moves the **width** of the four-loop window (which points
qualify) and not **who holds** the actuator inside it. Refuted if the cell is empty at every `τ_f`
when `τ_q = 0.05`, which would mean the pre-check's E2 was the valve's doing.

**P3 — THE SIGN FLIPS BETWEEN COORDINATES.** In `clip` the state is the CUT `g`, lagging
`required` **from below**, so a slower leg has a **smaller** cut and **less** authority — the
opposite of `demand`. Predicted: on the identical `(τ_f, τ_gov)` grid, the fuel-authority region in
`clip` lies on the **opposite side** of the diagonal from `demand`'s. If it holds:

> **WHICH LOOP HOLDS THE ACTUATOR IS COORDINATE-DEPENDENT** — rung 53's *a margin is a distance*
> and rung 79's *a coordinate is a gauge the plant cannot reach*, now said about **authority**,
> the one noun rungs 72–80 treated as a property of the plant.

**P4 — THE MASK IS SYMMETRIC, AND `n_live ≤ 3` A SEVENTH TIME.** Every cell rungs 72–80 measured
had the **governor** holding and the **fuel leg** masked. This rung supplies the mirror. Predicted:
rung 72's block is indifferent to which leg is masked — exactly one authority per interior point,
`mask_leak == 0` **exactly**, `ever_two_authorities` **False**. Refuted by any non-zero leak or any
two-authority point, either of which would make rung 72's block an artifact of *which* leg was
masked rather than of `min`.

**P5 — `zeros` MOVES BY ONE AT FUEL-AUTHORITY POINTS.** Rung 72 § 3 measured that in `clip` at
unmatched clocks (*"the masked leg regains a rank"*). Predicted to reproduce in `demand`.
**REPORTED, NOT GATED** — rung 80 § 8's discipline for `zeros`, inherited unchanged.

---

## 3. THE VACUITY CONDITIONS — REGISTERED IN ADVANCE, RUNG 79's LESSON

What would make this reader unable to report a fuel authority **even if the cell existed**:

| # | condition | consequence |
|---|---|---|
| V1 | `_quad_gains_at` returns no interior point in the fuel-authority cell (rung 80 attrited 33 riding → 7 interior) | the gain table is **VACUOUS**; P4/P5 are not scored, and the null is not a confirmation |
| V2 | `riding4_valid` false — the plant never left `Tt4_lo` | that row's counts are **void** (rung 80 § 8's 320-point frozen plant) |
| V3 | the `clip` positive control shows no fuel authority **anywhere** | the reader is broken; the whole table is void |
| V4 | fuel authority reported at **every** cell, matched clocks included | the reader is not reading the clock — it would contradict E1, which is measured |
| V5 | the criterion in § 1 scored on points where `required_fuel == 0` or `required_gov == 0` | one side is not a race; those points are **excluded by name**, counted, and reported |

## 4. THE CONTROLS, FIXED IN ADVANCE

1. **THE MATCHED-CLOCK CONTROL.** `(0.05,0.05,0.05,0.05)` at the anchor walls must reproduce E1 —
   all `gov`. It is the same rig, so a disagreement means the grid is not the shipped plant.
2. **THE `clip` POSITIVE CONTROL.** Rung 80 § 5's shared-wall `clip` arm has a fuel-authority
   interior cell at `s = 0.135`; it is carried in the same table so a `demand` null is readable.
3. **ONE KNOB PER AXIS.** `τ_q` and `τ_s` are held FIXED across the `(τ_f, τ_gov)` grid — the
   pre-check's own confound (§ 0), named so it cannot be repeated silently.
4. **THE WALLS ARE RUNG 80's, UNCHANGED.** `φ_lim = 0.75`, `φ_air = 0.77`, on rung 80's rig
   (`FLOOR = 0.55`, `B = 0.10`, `V_MAX = 0.20`, `LO/HI/TT4_MAX = 1000/1400/1200`). A wall sweep is
   **refused** at this rung: it would confound the clock with the set-point gap, which is the very
   difference § 1's criterion is a statement about.

## 5. WHAT WOULD REFUTE EACH PREDICTION

| # | refuted by |
|---|---|
| P1 | criterion agreeing with the measured label at < 95 % of scored points, or disagreements sitting far from the tie |
| P2 | an empty fuel-authority set at every `τ_f` once `τ_q` is held at 0.05 |
| P3 | `clip`'s fuel-authority region lying on the **same** side of the diagonal as `demand`'s |
| P4 | any non-zero `mask_leak`, or any interior point with two authorities |
| P5 | (not gated — reported either way) |

## 6. THE REDUCE CONTRACT

This rung adds **no state, no knob and no constant**. The reduce is therefore an **identity**: at
the matched clocks and rung 80's walls, `AuthorityClockTransient`'s march must be **bit-for-bit**
`SplitWallTransient`'s on `φ_lp`, `Tt4`, `b`, `v` and both legs' states over all 341 points, and
its four-loop set must be rung 80's own. A reader-only rung whose march moved would be a rung-80
regression wearing a new class name.

## 6a. SCORED — APPENDED AFTER THE SWEEPS, WITH NOTHING ABOVE EDITED
