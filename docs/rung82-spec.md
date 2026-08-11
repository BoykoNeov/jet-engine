# Rung 82 — THE THRESHOLD'S OWN LAW

**The seam:** `docs/rung81-spec.md` § 8, first bullet — *"THE THRESHOLD'S OWN LAW. § 1 locates it
between `τ_f = 0.05` and 0.08 on this ramp. The criterion says it should move with `ċ_f/ċ_r` … it
is the sweep that would turn the criterion from a label-predictor into a quantitative one."*

**Plant:** `ThresholdLawTransient` (`turbojet/engine.py`) — **no state, no knob, no constant**
(rungs 77/81's precedent). **Anchor + pre-registration:**
`docs/plans/rung82-anchor-threshold-law.md`. **Gates:** `tests/test_rung82.py`.

**THE SEAM'S OWN QUESTION IS ANSWERED, AND THE ANSWER IS NO.** The criterion cannot be turned into
a quantitative threshold predictor — and the reason is not accuracy.

---

## 0. HONESTY, FIRST — THE PRE-CHECK KILLED THE INTENDED HEADLINE, AND TWO BARS WERE MY OWN

The seam names `ċ_f/ċ_r` as the lever, so the intended rung was a **two-term split**:
`τ_f* = gap/ċ_f + τ_gov·(ċ_r/ċ_f)`, with the ramp rate dividing the first term and leaving the
second invariant. **A pre-check on the shipped plant refuted that before the anchor was written**
(anchor § 0, E1–E6; rung 81 § 0's precedent, and the rung takes no credit for any of it). The
split needs `ċ_f/ċ_r` to be `r`-invariant, and it is not: band `[1.40, 3.33]` at `r = 0.50`
against `[1.40, 8.16]` at 0.25.

| # | predicted | measured |
|---|---|---|
| **P1** | the fixed point lands inside the measured bracket, the forward reading does not | **VOID — the bar was mine.** The bracket is `2^-10` of the search interval, a width set by a loop count, not by the plant. Both readings sit outside it; neither fact means anything (§ 2) |
| **P2** | both readings sit **below** the measured threshold at every ramp | **SPLIT.** Fixed point **5 of 5**. The forward reading only 3 of 5 — **and the refuted half became § 3a's causal law**: its sign is its **reference's side**, 5 of 5 at fixed ramp |
| **P3** | forward error grows with the reference's distance; never better than the fixed point | **SPLIT.** *Never better* **CONFIRMED 5 of 5**. *Growth* **VOID on § 1–3's axis** (five different plants) and, on its own axis, **SPLIT — grows below the threshold, SHRINKS above it** |
| **P4** | `∂τ_f*/∂τ_gov` = the binding point's own `1/(κ·ratio)`, within 25 % | **SIGN CONFIRMED, MAGNITUDE REFUTED** — 47.1 % on the full secant. Sub-interval: **1.236×** near the reference, **0.397×** away. **Transfer 0.529** |
| **P5** | the wall reaches the set-point term only; raising `φ_lim` raises the threshold | **DIRECTION REFUTED** (raising it **lowers** the threshold) **and SEPARATION WITHDRAWN** (V7): `gap` −64 %, lag term +44 %, **`ċ_f` +144 %** |

**AND THE HEADLINE IS NONE OF THEM.** It is § 3a's — a forward reading that inherits the sign of
its own reference.

**Two disclosures that are not softened.** P1 and V5 both compared a **physical** quantity against
a **bisection tolerance**; that is a category error in the registration and it is reported as one,
not re-scored into a currency that passes. And this rung's own first `p4` **dropped `κ`** and
reported a 5.7× miss where the honest number is 1.9× — anchor § 4.5's warning firing on the reader
that wrote it, corrected in place with the mistake left named in the code.

---

## 1. THE THREE READINGS — WHY A LABEL PREDICTOR IS NOT A THRESHOLD PREDICTOR

Rung 81 § 1's criterion, unmodified, rearranged for the clock at one trajectory point:

    THE POINT-WISE IMPLIED THRESHOLD    τ̂(s) = ( gap(s) + τ_gov·ċ_r(s) ) / ċ_f(s)

`τ̂` is in the **EFFECTIVE** clock. Rung 52's lag is asymmetric (`tau_rel = 3·tau_att`), and
**every binding point in this family is in release**: `κ = 3.0` on every row of every table below,
`all_kappa_pure` True, V4 never fired. A table quoting the swept knob against a threshold derived
in the effective one is off by exactly 3×.

A march has a fuel-predicted point iff `h(τ_f) ≡ min_s [ τ̂(s) − κ·τ_f ] < 0`, so the criterion's
threshold is **the root of `h`** — and `h`'s inputs are all read off a trajectory that `τ_f` moves
(rung 81 § 3: 1 304 of 1 364 floats). Three readings of one formula, and they are **not the same
object**: **FORWARD** (`τ̂_min/κ` off a fixed reference march — a prediction), **FIXED POINT**
(the root of `h` — a self-consistent solve), **MEASURED** (bisected on the plant).

---

## 2. THE RAMP SWEEP — § 1–3 MEASURED

`φ_lim/φ_air = 0.75/0.77`, `τ_gov = τ_q = τ_s = 0.05`, `coord = demand`, `ds = 0.005`, reference
`τ_ref = 0.05`, 10 bisections (bracket 2.89e-4), `n_void = 0`, `monotone_in_r` True.

| `r` | measured `τ*` | eff (`×κ`) | fixed point | forward | err fix | err fwd | `s_bind` | `n4` |
|---|---|---|---|---|---|---|---|---|
| 0.20 | 0.01224 | 0.03671 | 0.01108 | 0.01009 | **9.4 %** | 17.6 % | 0.060 | 52 |
| 0.25 | 0.02033 | 0.06100 | 0.01975 | 0.01805 | 2.8 % | 11.2 % | 0.080 | 49 |
| 0.35 | 0.03854 | 0.11563 | 0.03710 | 0.03225 | 3.7 % | 16.3 % | 0.120 | 44 |
| 0.50 | 0.06340 | 0.19021 | 0.06109 | 0.11008 | 3.6 % | **73.6 %** | 0.185 | 35 |
| 0.70 | 0.09462 | 0.28386 | 0.09202 | 0.26771 | 2.7 % | **182.9 %** | 0.270 | 24 |

* **THE FIXED POINT LANDS TO A FEW PERCENT AND THE FORWARD READING DOES NOT** — 2.7–9.4 % against
  11.2–182.9 %, and the forward reading is **never** the closer of the two (P3, 5 of 5).
* **THE RAMP IS A THRESHOLD LEVER IN ITS OWN RIGHT.** `τ*` rises monotonically 0.0122 → 0.0946
  across the admissible range, and at rung 80's own clocks the fuel region **opens with no clock
  change at all** below `r = 0.5` (anchor § 0, E2). Rung 81 needed `τ_f = 0.08`; `r = 0.35` does it
  free.
* **AND THE WINDOW IS BOUNDED ABOVE, NOT SMALL.** At `r ≥ 1.0` `n_riding4 = 0` — V1, and the
  distinction matters: there is no four-loop point to *have* a threshold, so those ramps are not
  data points with a large value, they are **not data points**.
* **`s_bind` MIGRATES MONOTONICALLY**, 0.060 → 0.270 — rung 56's *"the binding row migrates with
  power"* one rung along, and § 3a's mechanism.

**V5, IN ITS OWN CURRENCY.** Halving the step to `ds = 0.0025` moves `τ*` by **0.00 / 0.00 / −0.75
/ −0.46 / −0.31 %** — an order of magnitude below the smallest effect scored. The thresholds are
resolved; the registered form of V5 compared that move against the *bisection* width and trips at
`r = 0.35`, which is P1's error a second time and is reported, not repaired.

---

## 3. § 3a — THE DISCRIMINATOR, AND THE HEADLINE

§ 2's five rows all use the **same** `τ_ref = 0.05`, so the reference's side of the threshold is
**perfectly collinear with `r`**: the ramp was changed and the side followed. That is a correlation
across a confounded axis — `docs/rung74-arrest-interval.md`'s *"a closed-loop difference cannot
isolate a forcing"*. **So the ramp is held and the reference is swept across the threshold**
(`r = 0.35`, `τ* = 0.03854`):

| `τ_ref` | side | forward | signed error | \|err\| | `s_bind` | `n_fuel` |
|---|---|---|---|---|---|---|
| 0.020 | **below** | 0.07664 | **+0.03810** | 98.9 % | 0.130 | 0 |
| 0.030 | **below** | 0.05837 | **+0.01983** | 51.4 % | 0.130 | 0 |
| 0.050 | above | 0.03225 | **−0.00629** | 16.3 % | 0.115 | 6 |
| 0.080 | above | 0.03585 | **−0.00269** | 7.0 % | 0.110 | 20 |
| 0.120 | above | 0.03762 | **−0.00092** | 2.4 % | 0.105 | 39 |

> **HEADLINE — A FORWARD READING INHERITS THE SIGN OF ITS OWN REFERENCE.** At one ramp, on one
> plant, `sign(forward − τ*)` follows **the side its reference march sits on — 5 of 5**. A
> reference above the threshold under-predicts; below, it over-predicts. So the forward reading is
> not a prediction of the threshold at all: **it is a report on where the reader started.** The
> fixed point, which has no reference, is the only reading with nothing to inherit.

**AND THE MAP IS A CONTRACTION ON ONE SIDE ONLY.** Read as an iteration `τ_ref ↦ forward(τ_ref)`,
the local slope is **≈ +0.044** above the threshold (0.08 → 0.12) and **≈ −1.83** below it
(0.02 → 0.03). Above, it converges in a single step — which is why the error **shrinks** as the
reference moves further away (16.3 → 7.0 → 2.4 %), refuting P3's growth clause on the side nobody
expected. Below, `|slope| > 1`: the iteration **diverges**, and the error grows (51 → 99 %). This
is rung 77's `1/(1−c)` with the sign of `c` deciding whether the reading is usable at all, and
**the threshold is the boundary between the two regimes.**

**THAT MECHANISM EXPLAINS § 2 ENTIRELY.** The two ramps where the forward reading blew up (73.6 %,
182.9 %) are **exactly** the two where `τ_ref = 0.05` sits *below* the threshold. Nothing about
those ramps is special; the reference was on the diverging side.

---

## 4. THE OTHER TWO KNOBS — § 4–5 MEASURED

**§ 4, `τ_gov` (`r = 0.35`, `φ_lim = 0.75`).** The threshold rises — 0.03681 / 0.03854 / 0.04143
at `τ_gov` = 0.02 / 0.05 / 0.20 — so P4's sign holds. The magnitude does not:

| span | measured | `1/(κ·ratio)` | ratio |
|---|---|---|---|
| 0.02 → 0.20 (full secant) | 0.025694 | 0.048570 | **0.529** |
| 0.02 → 0.05 | 0.057812 | 0.046769 | **1.236** |
| 0.05 → 0.20 | 0.019271 | 0.048570 | **0.397** |

**The criterion's coefficient is right near the reference and collapses away from it** — inside
P4's registered 25 % on the near span, 60 % low on the far one. `transfer = 0.529` is the fraction
of a **frozen-trajectory** coefficient that survives the plant's own response, and it is this
rung's headline measured on an independent knob.

**§ 5, THE WALL.** Admissible range only — at `φ_lim = 0.740` the threshold is censored **above**
`τ_f = 0.30` and at 0.760 **below** 0.004 (V3, both reported): a **>20× swing for 0.02 of wall**.

| `φ_lim` | `τ*` | `gap` | `ċ_f` | `ċ_r` | ratio |
|---|---|---|---|---|---|
| 0.7450 | 0.09780 | +2.3110e-03 | +1.0968e-02 | +2.1292e-03 | 5.151 |
| 0.7500 | 0.03854 | +1.5724e-03 | +1.8104e-02 | +2.5402e-03 | 7.127 |
| 0.7550 | 0.01368 | +8.2907e-04 | +2.6806e-02 | +3.0641e-03 | 8.749 |

**P5 FAILS BOTH WAYS, AND THE SECOND FAILURE IS THE INTERESTING ONE.** The direction is backwards
(`φ_lim` is the fuel leg's **own** floor, so raising it makes that leg's cap more severe and
**lowers** `gap`) — and the terms do not separate: `gap` **−64.1 %** against the lag term
`τ_gov·ċ_r` **+43.9 %**. **`ċ_f` moves +144.4 %** — the largest move in the table, on a slope the
wall is supposed to reach through *neither* term. V7 fired as registered, and P5 is **withdrawn**.

---

## 5. WHAT THIS SAYS, AND WHAT IT CORRECTS

> **THE CRITERION'S TERMS ARE NOT INDEPENDENT COORDINATES.** Every knob reaches every term, through
> the trajectory. The ramp moves both slopes non-proportionally (`ċ_f/ċ_r` 1.4–3.3 at `r = 0.5`,
> 1.4–8.2 at 0.25). The governor clock keeps 53 % of its own coefficient. The wall, placed in the
> set-point term, is the largest mover of `ċ_f` in the rung.

**SO RUNG 81's 99.15 % IS BOUNDED, NOT DIMINISHED.** That criterion scores as a **label** predictor
*because* it is evaluated on the very trajectory it labels — every input read at the point being
scored. Asked to predict **across** trajectories it has no separation to exploit, and the
seam's hope — *"turn the criterion from a label-predictor into a quantitative one"* — is answered
**no**, with the reason. **This BOUNDS rung 81 § 1–2 the way rung 53 bounded rungs 36–52's currency:
the result stands exactly where it was measured and does not travel.**

**AND IT CORRECTS RUNG 81 § 8's OWN SEAM TEXT.** *"The criterion says it should move with
`ċ_f/ċ_r`"* — it does move, monotonically and by 8× across the admissible ramps, but **not with
that ratio**, which is itself a function of every knob. The seam named a coefficient where the
plant has a coupling.

**A THIRD READING, FREE.** The ramp rate is a fuel-side authority lever the whole 46–52 family
never used: at rung 80's clocks it opens the fuel region with **no limiter change at all**
(anchor § 0, E2). Rung 44 called the surge excursion *"schedule-slaved, ramp-rate-driven"*; this is
the same lever reaching **which leg holds the actuator**, which rung 44 had no instrument for.

## 6. WHAT THIS DOES **NOT** SAY

* **It does not say the criterion is wrong.** Its labels still score 99.15 % (rung 81 § 2), and the
  fixed-point reading lands to 2.7–9.4 %. What fails is **evaluation off a foreign trajectory**.
* **It does not say the forward reading is useless** — from *above* the threshold it converges in
  one step to 2.4 %. It says the reader cannot know which side it is on without solving the problem
  it was trying to avoid.
* **It does not measure a rank, a mode, or `n_live`.** No Jacobian is built. Rungs 72–81's ledger is
  untouched; this rung reads authority only through `n_fuel`.
* **The ramp window, the wall range and the bracket are DISCLOSED, not derived.** `r ∈ [0.20, 0.70]`
  and `φ_lim ∈ [0.745, 0.755]` are where V1/V3 leave the plant marchable; nothing here claims they
  are physical boundaries.
* **`κ = 3.0` is rung 52's constant, not a finding.** That every binding point sits in release is
  measured; *why* the release branch binds is not asked.

## 7. THE REDUCE CONTRACT, AND THE GATES

**The reduce is an IDENTITY** — no state, no knob, no constant. At `r = 0.5`, rung 80's walls and
rung 81's clocks, `ThresholdLawTransient`'s march is **bit-for-bit** `AuthorityClockTransient`'s,
and its four-loop set rung 81's own: `n_riding4 = 33`, `n_fuel = 0`, reproduced on every run.

Gated in `tests/test_rung82.py`: the identity reduce; the monotone `τ*(r)` and its `n_void = 0`;
the fixed point beating the forward reading at every ramp (P3's surviving half) and sitting below
the measured threshold at every ramp (P2's surviving half); **§ 3a's 5-of-5 sign law** and the
one-sided contraction; `κ = 3.0` pure on every row; V1 at `r ≥ 1.0`; V3's two censored walls; V7's
withdrawal; and the `ds` control in its **own** currency (≤ 1 %).

**Deliberately NOT gated:** P1 (void by its own bar — gating a self-referential width would ship
the error), V5 in its registered form, the forward reading's *values*, P4's full-span secant, and
any count on a row with `riding4_valid` False.

## 8. NEXT SEAMS

* **THE FIXED POINT'S OWN COST.** It lands to a few percent but needs a bisection over marches —
  ~40 marches a threshold here. Whether a **single Newton step** off the residual `h` (whose slope
  § 3a already measures at ±0.044 / −1.83) reaches the same place for one march is untested, and it
  is the difference between a diagnosis and a usable predictor.
* **THE DIVERGING SIDE, RESOLVED.** § 3a measures the map's slope at two points below the threshold
  and one pair above. Where the slope passes through −1, and whether that boundary is the threshold
  itself or merely near it, is unmeasured — and it decides whether "start above" is a rule or a
  heuristic.
* **THE SAME THREE READINGS IN `clip`.** Rung 81 § 3 found the fuel leg's clock an **exact null**
  there at the split wall; a threshold in a coordinate where the knob is inert is either absent or
  somewhere else entirely, and neither has been looked at.
* **A FINGERPRINT ARM FOR THIS RUNG.** Slice 6's lesson applies: this rung's two readers sit in
  different regimes (§ 2's bisection is a *bracket*, § 3a's forward values are floats off a single
  march), so one tolerance pair is unlikely to serve both. The smallest live value to clear is
  § 5's `gap` at `φ_lim = 0.755`, **8.29e-04**.
* Everything rungs 72–81 §§ 8–11 leave, unchanged by this rung.
