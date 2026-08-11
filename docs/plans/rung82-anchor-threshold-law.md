# Rung 82 — THE THRESHOLD'S OWN LAW (anchor + pre-registration)

**The seam:** `docs/rung81-spec.md` § 8, first bullet — *"THE THRESHOLD'S OWN LAW. § 1 locates it
between `τ_f = 0.05` and 0.08 on this ramp. The criterion says it should move with `ċ_f/ċ_r`, i.e.
with the schedule's slope and the wall pair — untested, and it is the sweep that would turn the
criterion from a label-predictor into a quantitative one."*

**Plant:** `ThresholdLawTransient` (`turbojet/engine.py`) — **no state, no knob, no constant**
(rungs 77/81's precedent, and here it is forced the same way: the knob that moves the threshold is
the ramp rate `r`, which `_stator_march` has carried since rung 57). **Gates:**
`tests/test_rung82.py`. **Spec:** `docs/rung82-spec.md`.

---

## 0. WHAT THE PRE-CHECK ALREADY ESTABLISHED — NOT PREDICTIONS, AND NOT THIS RUNG'S CREDIT

Rung 81 § 0's precedent, and it applies harder here: **the seam named `ċ_f/ċ_r` as the lever, and a
pre-check on the shipped plant refuted that framing before this anchor was written.** Two scripts,
no new class, `AuthorityClockTransient`'s own `_split_march` / `_criterion_at` / `_riding4`.

The intended headline was a **two-term split**: `τ_f* = gap/ċ_f + τ_gov·(ċ_r/ċ_f)`, with the ramp
rate dividing the first term and leaving the second invariant. That requires `ċ_f/ċ_r` to be
`r`-invariant. **It is not**, and E4 below is the refutation. Everything in this section is
measurement, not prediction, and the rung takes **no credit** for any of it.

| # | established by the pre-check |
|---|---|
| **E1** | **The four-loop window CLOSES from above.** At rung 80's clocks `n_riding4` falls 60/59/55/52/46/33/19/8/**0** across `r` = 0.10 … 1.00. At `r ≥ 1.00` there is **no four-loop point to hold a threshold**, so the `r` sweep is admissible only on `r ≤ 0.85` |
| **E2** | **The ramp rate opens the fuel region ON ITS OWN.** At rung 80's own clocks (all 0.05, `τ_f` untouched) `n_fuel` = 0 / 0 / 0 / 6 / 13 / 16 / 19 / 20 at `r` = 0.85 … 0.10. Rung 81 needed `τ_f = 0.08` at `r = 0.5`; `r = 0.35` opens it with **no clock change at all** |
| **E3** | **The threshold in `τ_f` is monotone in `r`,** bracketed: `< 0.010` (`r`=0.15), `(0.010, 0.020]` (0.20), `(0.020, 0.035]` (0.25), `(0.035, 0.050]` (0.35), `(0.050, 0.080]` (0.50 — rung 81's own bracket, reproduced) |
| **E4** | **`ċ_f/ċ_r` IS NOT `r`-INVARIANT** — band `[1.40, 3.33]` at `r` = 0.50 against `[1.40, 8.16]` at 0.25. The two-term split is **dead as a headline** before it was written down |
| **E5** | **EVERY scored cell's binding clock is the RELEASE one.** `_criterion_at` reads `τ_eff = 3·τ_f` at every point of every cell examined — rung 52's asymmetry, which rung 81 handled inside its reader and never surfaced as a finding |
| **E6** | **The FORWARD reading is wrong at `r` = 0.5.** Read off the `τ_f = 0.05` reference march the criterion implies an effective threshold of **0.330**, against a measured bracket of `(0.15, 0.24]` in the same currency — **1.4–2.2× high** |

**What § 0 could NOT say** — and what this rung is therefore for: whether the *fixed-point* reading
lands where the forward one misses, whether the miss is **signed**, how the error **scales** with
the reference's distance, and whether `τ_gov` and the **wall pair** reach the threshold through the
two terms the criterion says they do. E6 is one number at one ramp off one reference; it is not a
law, and it is not scored as one.

---

## 1. THE MECHANISM BEING TESTED, DERIVED — ZERO NEW CONSTANTS

Rung 81 § 1's criterion, verbatim and unmodified:

> **THE FUEL LEG HOLDS THE ACTUATOR IFF**
>
>     required_gov − required_fuel   <   τ_f · ċ_f  −  τ_gov · ċ_r
>          [the SET-POINT gap]              [the LAG-ERROR gap]

Rearranged for the clock, at one trajectory point `s`, this rung adds **nothing** to it:

    THE POINT-WISE IMPLIED THRESHOLD      τ̂(s)  =  ( gap(s) + τ_gov·ċ_r(s) ) / ċ_f(s)

`τ̂` is in the **EFFECTIVE** clock — the one `_criterion_at` actually reads through
`_demand_tau`. Rung 52's lag is asymmetric (`tau_rel = 3·tau_att`, set in `_coord_march`), so the
**swept** knob and the **binding** clock are not the same number, and E5 says every binding point
in this family is in release. The two are therefore carried side by side everywhere, and the map
between them is named, never assumed:

    τ_eff(s)  =  κ(s) · τ_f ,      κ ∈ {1, 3}     — read per point, never imposed

**A march has at least one fuel-predicted point iff**  `h(τ_f) ≡ min_s [ τ̂(s) − τ_eff(s) ] < 0`.
So the criterion's own threshold is the **root of `h`** — and `h`'s inputs (`gap`, `ċ_f`, `ċ_r`,
`κ`) are all read off a trajectory that `τ_f` itself moves (rung 81 § 3: 1 304 of 1 364 floats).
**That is the whole rung.** Three readings of one formula, and they are not the same object:

| reading | definition | what it is |
|---|---|---|
| **FORWARD** | `τ̂_min` off a **fixed** reference march (`τ_f = 0.05`, where the region is closed), divided by `κ` | a genuine **prediction** — the pre-registered primary |
| **FIXED POINT** | the root of `h(τ_f)` — the criterion read off the march **at** the candidate | a **self-consistent** solve, not a forward prediction, and labelled so |
| **MEASURED** | the smallest `τ_f` with `n_fuel > 0`, bisected on the plant | the thing being predicted |

**AND THE FIXED POINT'S CONVERGENCE GAIN IS RUNG 77's `1/(1−c)`.** `h` is a residual whose slope in
`τ_f` carries both the direct term (`−κ`) and the trajectory's own response (`dτ̂/dτ_f`); their
ratio is the transfer rate. **If it is large, rung 81 § 2's 99.15 % label accuracy does not
transfer to the threshold at all** — registered here as the risk, not discovered in § 6a.

---

## 2. THE PREDICTIONS, IN THE ORDER THEY WILL BE SCORED

**P1 — THE FIXED POINT LANDS AND THE FORWARD READING DOES NOT.** At every admissible `r`, the root
of `h` falls **inside** the measured bisection bracket; the forward reading falls **outside** it at
`r = 0.5` and at ≥ 2 of the other admissible ramps. Scored as a pair — a rung whose fixed point
also missed would be a broken reader, and a rung where the forward reading also landed would have
no content.

**P2 — THE MISS IS SIGNED, AND IT IS EARLY.** Rung 81 § 2 measured every one of its 9
disagreements as `predicted = fuel, measured = gov` — the criterion fires **early, never late**.
Transferred to the threshold: **both** the fixed-point and the forward readings sit **BELOW** the
measured threshold, at **every** admissible `r`. This is the free prediction and it is the cheap
falsifier: if it fails, the fault is in this rung's reader, not in the physics.

**P3 — THE FORWARD ERROR GROWS WITH THE REFERENCE'S DISTANCE.** Sweeping the reference march's own
`τ_f` over the admissible range, `|forward − measured|` increases **monotonically** with
`|τ_ref − τ_f*|`, and the forward reading is **never** closer than the fixed point at any ramp.
This is what makes E6 a law rather than one number.

**P4 — `τ_gov` SHIFTS THE THRESHOLD UP, AFFINELY.** `∂τ_f*/∂τ_gov = ċ_r/ċ_f` **at the binding
point** — so the threshold rises with `τ_gov`, and the measured rise agrees with the binding
point's own locally-read slope ratio to **within 25 %**. Rung 81 § 1's table already shows the
sign (11/9/5 fuel-held at `τ_f = 0.08` across `τ_gov` = 0.02/0.05/0.20); the **magnitude** is new.

**P5 — THE WALL PAIR REACHES THE SET-POINT TERM ONLY.** Raising `φ_lim` raises `gap` and therefore
the threshold; the binding point's `ċ_f/ċ_r` moves by **less than a third** of the fractional move
in `gap`. Rung 81 § 4.4 **refused** a wall sweep for exactly this reason — *"it would confound the
clock with the set-point gap"*. This rung is where that confound becomes the measurement, because
the criterion says the two enter through **different terms**, and a wall sweep is the only way to
test that they separate. Two walls only (§ 4.4), and the refusal is discharged by name.

---

## 3. THE VACUITY CONDITIONS — REGISTERED IN ADVANCE, RUNG 79's LESSON

What would make this reader unable to report a threshold **even if one existed**:

| # | condition | consequence |
|---|---|---|
| **V1** | the four-loop window is empty at **either** bracket end — `n_riding4 = 0`, E1's own failure mode at `r ≥ 1.0` | that ramp is **not a data point**: "no fuel-held point" means "no point to hold". Row **void**, counted, and reported |
| **V2** | `riding4_valid` false — the plant never left `Tt4_lo` (rung 80 § 8's 320-point frozen plant) | row **void** |
| **V3** | the measured threshold sits **at** a bracket endpoint rather than strictly inside | row **void** — an endpoint is a censored observation, never a value. E3's `r = 0.15` row (`< 0.010`) is already one |
| **V4** | the binding point is in **ATTACK** (`κ = 1`) at some cell while others are in release | that row is reported **separately** and never averaged into a `κ = 3` row; `κ` is read per point and printed on every row |
| **V5** | the threshold's own `ds`-sensitivity exceeds the bisection bracket width | the threshold is **NOT RESOLVED** and the row is **void** — rung 81 § 2 put scored points `9.3e-04` from a tie, so a finer `ds` can hand the first flip to a different point |
| **V6** | the march arrests (`max_Tt4 ≤ Tt4_lo`) or the plant surges at the fast end | row **void**, reported by name |
| **V7** | `|Δgap|` across the `r` sweep is comparable to the term it is being separated from | **P5's separation is withdrawn** and the coupling is reported as the finding, not buried |

---

## 4. THE CONTROLS, FIXED IN ADVANCE

1. **RUNG 81's OWN CELL IS THE IDENTITY CONTROL.** At `r = 0.5`, `φ = 0.75/0.77`, clocks
   `(0.05, 0.05, 0.05, 0.05)`: **33** four-loop points, **all `gov`**; and at `τ_f = 0.08`,
   **9 fuel of 39**. Both are rung 81 § 1's own table cells. A grid that disagrees is not the
   shipped plant. (The pre-check already reproduced both; it is a **gate** here, not a finding.)
2. **THE `ds` CONTROL — V5's instrument.** Every quoted threshold is re-bisected at
   `ds = 0.0025` and the two brackets reported side by side. A threshold that moves by more than
   its own bracket is void, not rounded.
3. **ONE KNOB PER AXIS.** `τ_q = τ_s = 0.05` throughout, rung 80's values, on **every** arm
   including P5's — rung 81 § 4.3's control, inherited verbatim, and § 0's own confound.
4. **THE WALLS ARE RUNG 80's EXCEPT ON P5's ARM**, where **exactly one** moves (`φ_lim`), with
   `φ_air` held. Two settings, not a grid: enough to test a separation, not enough to fit one.
5. **SWEPT AND EFFECTIVE CLOCK ARE BOTH PRINTED, ALWAYS.** E5 makes `κ` load-bearing, and a table
   quoting one currency silently would be off by 3× — the trap this rung is most exposed to.

## 5. WHAT WOULD REFUTE EACH PREDICTION

| # | refuted by |
|---|---|
| **P1** | the fixed-point root falling outside the measured bracket at any admissible ramp; **or** the forward reading landing inside it at every ramp (which would make the rung contentless, and is reported as such rather than reframed) |
| **P2** | either reading sitting **above** the measured threshold at any admissible ramp |
| **P3** | a non-monotone `|forward − measured|` in `|τ_ref − τ_f*|`, or the forward reading beating the fixed point at any ramp |
| **P4** | a threshold that **falls** with `τ_gov`, or a rise disagreeing with the local slope ratio by more than 25 % |
| **P5** | `ċ_f/ċ_r` at the binding point moving comparably to `gap` under the wall move (V7), i.e. the two terms not separating |

## 6. THE REDUCE CONTRACT

This rung adds **no state, no knob and no constant** — `r` is `_stator_march`'s own parameter since
rung 57, and `κ` is rung 52's `3.0`. The reduce is therefore an **identity**: at `r = 0.5`, rung
80's walls and rung 81's clocks, `ThresholdLawTransient`'s march must be **bit-for-bit**
`AuthorityClockTransient`'s on `φ_lp`, `Tt4`, `b`, `v`, and its four-loop set rung 81's own. A
reader-only rung whose march moved would be a rung-81 regression wearing a new class name.

## 6a. SCORED — APPENDED AFTER THE SWEEPS, WITH NOTHING ABOVE EDITED

Tables and derivations: `docs/rung82-spec.md`. **Three of the five predictions are split or worse,
and one is VOID BY MY OWN BAR** — the registration errors are named here, not smoothed.

| # | verdict |
|---|---|
| **P1** | **VOID — the bar was self-referential.** "Inside the measured bisection bracket" is a width I set by choosing `n_bisect = 10` (2.89e-4, i.e. 0.3–2.4 % of `τ*`). At 20 bisections both readings fail harder; at 3 both pass. **A test whose outcome is set by a loop count measures nothing** (rung 78's vacuity traps, rung 79's lesson). Both readings land outside it, and that is reported rather than re-scored in a currency that works — the substantive comparison is P3's |
| **P2** | **SPLIT, and the refuted half was REPLACED BY A STRONGER LAW.** Fixed point **below** the measured threshold at **5 of 5** ramps — **CONFIRMED**. The forward reading is early at only 3 of 5 — **REFUTED as stated**, because its sign is not a property of the *reading*: § 3a holds the ramp fixed and sweeps the reference across the threshold, and `sign(forward − τ*)` follows **the reference's own side, 5 of 5** |
| **P3** | **SPLIT.** *"Never closer than the fixed point"* — **CONFIRMED, 5 of 5**. The **growth clause is VOID on § 1–3's axis** (it compared distances across five *different* plants, which is not the axis P3 named) and, re-measured at fixed ramp, **SPLIT**: error grows with distance **below** the threshold (51 % → 99 %) and **SHRINKS above** it (16.3 % → 7.0 % → 2.4 %). The further above you start, the *better* the forward reading |
| **P4** | **SIGN CONFIRMED, MAGNITUDE REFUTED at the registered line.** The threshold rises with `τ_gov`. The full-span secant is **0.02569** against the frozen-trajectory coefficient `1/(κ·ratio) = 0.04857` — **47.1 % miss**, registered bar 25 %. Sub-interval by sub-interval it is **1.236× the prediction near the reference** (`τ_gov` 0.02→0.05, i.e. inside the 25 % bar) and **0.397× away from it** (0.05→0.20). **Transfer 0.529** |
| **P5** | **DIRECTION REFUTED, SEPARATION WITHDRAWN (V7 fired).** Raising `φ_lim` **LOWERS** the threshold, monotonically, 0.0978 → 0.0137 over 0.745 → 0.755 — the anchor had the sign of `∂gap/∂φ_lim` backwards (`φ_lim` is the fuel leg's **own** floor, so raising it makes that leg's cap **more** severe). And the terms do not separate: `gap` −64.1 % against the lag term `τ_gov·ċ_r` **+43.9 %** — **and `ċ_f` +144.4 %**, the largest move in the table, on a slope the criterion places in *neither* of the wall's terms |

**THE VACUITY CONDITIONS THAT FIRED, ALL OF THEM.** V1 at `r ≥ 1.0` (§ 0, E1) — no four-loop point
to hold a threshold. V3 twice on the wall axis: `φ_lim = 0.740` censored **above** `τ_f = 0.30`,
`φ_lim = 0.760` censored **below** 0.004 — a **>20× swing in the threshold for 0.02 of wall**,
which is why § 5 sweeps 0.745…0.755 and not the pair the anchor named. V5 as registered trips at
`r = 0.35`; **in its own currency the step-halving moves the threshold by ≤ 0.75 % at every ramp**,
an order of magnitude below the smallest effect scored — the same self-referential-width error as
P1, disclosed the same way and retro-fitted no more than P1 was. V7 fired, and P5 is withdrawn
rather than softened.

**THE CONTROLS ALL HELD.** Rung 81 § 1's own cell reproduced exactly (`n_riding4` **33**, `n_fuel`
**0** at `r = 0.5`, matched clocks) — the identity reduce. `κ = 3.0` on **every** row of **every**
table, `all_kappa_pure` True, so V4 never fired and the swept/effective distinction is a clean
factor throughout. `n_void = 0` on the ramp sweep and on the corrected wall sweep.

**AND ONE ERROR IN THIS RUNG'S OWN READER, CAUGHT AND KEPT NAMED.** The first `p4` dropped `κ` and
reported a 5.7× miss where the honest number is 1.9× — **§ 4.5's own warning firing on the reader
that wrote it**. It is corrected in place with the mistake left in the comment, not tidied away.
