# Rung 83 — THE CORRECTOR'S OWN BAR

**The seam:** `docs/rung82-spec.md` § 8, first bullet — *"THE FIXED POINT'S OWN COST. It lands to a
few percent but needs a bisection over marches — ~40 marches a threshold here. Whether a **single
Newton step** off the residual `h` (whose slope § 3a already measures at ±0.044 / −1.83) reaches the
same place for one march is untested, and it is the difference between a diagnosis and a usable
predictor."*

**Plant:** `CorrectorLawTransient` (`turbojet/engine.py`) — **no state, no knob, no constant**, the
fourth reader-only rung (77, 81, 82, 83). **Anchor + pre-registration:**
`docs/plans/rung83-anchor-corrector-law.md`. **Gates:** `tests/test_rung83.py`.

**THE SEAM'S QUESTION IS ANSWERED, AND THE ANSWER IS NO** — one step does not reach the fixed
point, and neither does an iteration under a start rule fixed in advance. The reason is not
accuracy, and it is not the slope.

> **HEADLINE — A BRACKETING SOLVE LOCATES A *SIGN CHANGE*; A CORRECTOR NEEDS A *ROOT*, AND ON A
> RESIDUAL BUILT AS A MINIMUM THOSE ARE DIFFERENT OBJECTS.** At `r = 0.25` on the shipped grid
> there is no root at all: `g = F − τ` steps from `+1.65e−3` to `−2.43e−3` across a τ step of
> `1.25e−5`, at an argmin handover. The thirteen marches buy **an answer that exists**.

**CROSS-RUNG.** Rung 78 found *a residual's SLOPE is a GAUGE, its root's UNIQUENESS is not.*
This rung finds **its root's EXISTENCE is not either** — and it is `ds`-dependent.

---

## 0. HONESTY FIRST — WHAT THE PRE-CHECK SETTLED, AND TWO NUMBERS THAT MEASURED NOTHING

**The pre-check settled the foundation before the anchor was written** (anchor § 0, E1–E5; rungs
81/82's precedent, and the rung takes no credit for it). `h == τ̂_min − κ·τ` holds **to the last
bit** and the raw `τ_eff` set along a trajectory is a **single float**, so rung 82's "fixed point"
and "forward reading" are one object and the whole rung is about solving `F(τ) = τ`.

**Two numbers in earlier drafts measured nothing, and both are recorded rather than deleted.**

1. **An identity round-trip sold as a verification.** § 1.3's error law was first quoted as
   *"verified to 2.7e−14 across 35 rows"*. `c̄` is **defined** as `(F(τ₀) − τ*)/(τ₀ − τ*)`, so
   substituting it into the secant root reproduces the law in two lines **for any `F`, any slope,
   any root**. It cannot fail. The law is kept as a **derivation**; it is never quoted as evidence
   and **is not gated** — such a test would pass on a plant that does not exist. Rung 70's *"a gate
   computing my own formula twice"* and rung 77's *"perfect 1.000e+00"*, third occurrence.
2. **A slope read off an iteration's incidental path.** The first § 3 called `F` at `r = 0.25` a
   **sawtooth**, from a slope of ≈ −9 between two points the secant happened to visit. Measured
   properly (110 marches, §§ 3.1–3.2), `F` is **smooth and monotone on each branch** with **one**
   argmin handover. The wrong finding hid a stronger one.

**The registered predictions, scored:**

| # | predicted | measured |
|---|---|---|
| **P1** | the identity is exact; raw `τ_eff` a single float | **CONFIRMED** — difference exactly `0.0`, `κ = 3.0` exactly |
| **P2** | `sign(h)` gives the side from one march | **CONFIRMED**, both branches at all five ramps — **CORRECTS rung 82 § 6** |
| **P3** | a 2-march secant lands within 10 % of the 13-march fixed point at every ramp | **REFUTED** — no fixed protocol reaches it; best worst-case 19.6 % |
| **P4** | the borrowed slope is worse than the measured one at a majority of ramps | **REFUTED as stated**, and **folded into § 1.4** — the win/loss split tracks the lever arm, not the slope's provenance |
| **P5** | the cheap reading is no worse than the solve at ≥ 3 of 5 ramps | **REFUTED** — the bar was mis-specified (§ 0.1 below) |
| **P6** | the iterated secant reaches 1 % within 8 marches at ≥ 4 of 5 ramps | **REFUTED — 1 of 5** |
| **P7** | starting below the root is strictly faster than above | **REFUTED** — "below" holds both the best and the only total failure |
| **P8** | the non-monotonicity bites ≥ 1 ramp | **CONFIRMED via the overshoot arm only** — `clamps = 0` everywhere, `0 of 5` descend monotonically |

**AND THE HEADLINE IS NONE OF THEM.** It is § 3's, which no prediction anticipated.

### 0.1 P5 died of its own wording — the second bar in this lineage to die that way

P5 said *"the cheap forward reading from a reference far ABOVE"* and never **fixed** the reference.
With hindsight it passes 3 of 5 (2.4 % / 0.3 % / 0.4 % against the solve's 3.7 % / 3.6 % / 2.7 %) —
but the winning reference is a **different one at every ramp** (0.200, 0.200, 0.120, 0.120, 0.160),
and knowing which requires knowing the answer. At any single fixed reference the solve wins:

| `τ_ref` | 0.020 | 0.030 | 0.050 | 0.080 | 0.120 | 0.160 | 0.200 |
|---|---|---|---|---|---|---|---|
| ramps where the cheap reading beats the 13-march solve | 0/5 | 0/5 | 0/5 | 0/5 | **2/5** | 1/5 | 0/5 |

**The twelve extra marches WERE buying accuracy.** Rung 82's P1 died of scoring against a
loop-count width; this one died of a bar that permitted choosing the reference afterwards. Same
shape, second time: **a bar that names a DIRECTION is not a bar until it names a POINT.**

---

## 1. THE IDENTITY, THE STEP, AND WHY THE STEP CANNOT INHERIT THE READING

### 1.1 The two readings are one object

`_threshold_scan` returns `h = min_s [ĥ(s) − τ_eff(s)]` and `τ̂_min = min_s ĥ(s)`. With `τ_eff`
constant along `s` the two minima share an argmin, so

    h(τ) = τ̂_min(τ) − κ·τ = κ·( F(τ) − τ ),        F = τ̂_min/κ = rung 82's FORWARD reading

**The fixed point is the ROOT of the forward reading's own residual**, and rung 82's `fix`
bisection is literally solving `F(τ) = τ`. `CorrectorLawTransient.corrector_read` checks this on
the shipped dict at **zero tolerance**; it is the one part of § 1 that is a measurement.

### 1.2 The Newton step, and `1/(1−c)` in a third role

    h′(τ) = κ·(F′(τ) − 1)   ⇒   τ₁ = τ₀ + ( F(τ₀) − τ₀ ) / ( 1 − c ),     c = F′(τ₀)

**`κ` cancels.** The correction is the forward reading's own miss over rung 77's `1/(1−c)` — that
scalar's **third role**, after 77's stiffness and 78's gauge.

### 1.3 The error law — ALGEBRA, never a measurement (§ 0.1's discipline)

With `c̄ = (F(τ₀) − τ*)/(τ₀ − τ*)` the mean slope from the root out to the reference:

    (τ̂ − τ*)/τ*  =  ( c̄ − ĉ )/( 1 − ĉ )  ×  ( τ₀/τ* − 1 )
                     \_ slope error _/       \_ LEVER ARM _/

Two lines, true for any `F`. **Not gated, not quoted as evidence.** Its content is that a one-step
corrector has exactly two error sources and they **multiply**.

### 1.4 The opposition — and P4 folded in

Rung 82 § 3a's finding is that the forward reading gets **better the further above** the root the
reference sits (98.9 % at `τ_ref = 0.020` → 2.4 % at 0.120). § 1.3 says the correction gets **worse**
in the same direction. Solving for the slope precision a 10 % answer needs:

| ramp | `τ*` | required \|Δc\| at `τ₀ = 0.120` | `0.160` | `0.200` |
|---|---|---|---|---|
| 0.20 | 0.01108 | **0.0097** | **0.0071** | **0.0056** |
| 0.35 | 0.03710 | 0.0428 | 0.0289 | 0.0218 |
| 0.70 | 0.09202 | 0.3144 | 0.1294 | 0.0815 |

Rung 82 § 3a's two quoted slopes are **+0.044** and **−1.83**. **P4 is this law, not a separate
result:** the borrowed slope beats the measured one at the two near starts (29.7 / 35.0 % against
202 / 111 %) and loses at the four far ones, and that split tracks the lever arm, not where the
slope came from.

### 1.5 `c̄` is not what any local reading measures

Worse than a precision problem: along the ladder the **local** chord slope and `c̄` differ in
**SIGN** at six sampled rows — `r = 0.20` at τ₀ = 0.120/0.160/0.200 (+0.0066/+0.0038/+0.0024 against
−0.0032/−0.0013/−0.0005), `r = 0.35` at 0.050 (+0.1200 against −0.3757), `r = 0.50` at 0.080
(+0.1609 against −0.2283), `r = 0.70` at 0.120 (+0.2428 against −0.2666). At `r = 0.25`, τ₀ = 0.020,
they share a sign and differ **89×** (−0.1131 against −10.03).

The reason is structural: **`F` has an interior minimum just above the root** (at `r = 0.20`, `F`
equals `τ*` = 0.01108 at the root, falls to 0.00918 at τ = 0.020, then rises to 0.01098 at 0.200).
Anywhere on that rising far branch the local slope is positive while the chord back across the
minimum is negative. **Measuring `c̄` needs `F(τ*)` — the root — i.e. the solve being avoided.** One
mechanism kills both estimators: they are not estimating the right quantity badly, they are
estimating the **wrong quantity**.

### 1.6 The kink was NOT the mechanism

`F` is a `min`, so it is only piecewise smooth, and V6 registered a chord across an argmin switch as
not-a-slope. **It did not discriminate.** Under a fixed protocol the same-argmin flag neither
predicts nor anti-predicts accuracy, and the two best single estimates in the sweep (1.2 % at
`r = 0.25`, 4.1 % at `r = 0.50`) both straddle a switch. Two lucky rows out of ~30 is V6 failing to
be the mechanism, not inverting. **§ 3 is the mechanism** — and it is the same `min`, read
correctly.

---

## 2. THE SIDE IS FREE — CORRECTING RUNG 82 § 6

`h(τ₀) < 0` iff `F(τ₀) < τ₀`, so **`sign(h(τ₀))`, off the same single march that gives `F(τ₀)`, says
which side of the root `τ₀` sits on.** Rung 82 § 6 says the reader *"cannot know which side it is on
without solving the problem it was trying to avoid"*. **It can.** The side is one march; only the
root is a solve. Confirmed at every ladder point of every ramp, on **both branches at all five
ramps** after three extra below-root scans closed the gap the first ladder left at `r = 0.20/0.25`.

**Scope, stated and not softened.** The free side is the side of the **residual's own root**, not of
the plant's measured threshold. Those differ by 2.7–9.4 %, and there is a live counter-instance —
`r = 0.25`, τ = 0.020: `h = −8.14e−3` (above the root, 0.019754) while `n_fuel = 0` (below the
plant's threshold, 0.020332). Registered as V7 and reported, not filed as a miss.

---

## 3. THE SHAPE — AND `r = 0.25` HAS NO ROOT AT ALL

### 3.1 `F` resolved through the root, at both of rung 82's own steps

| | `ds = 0.005` (shipped) | `ds = 0.0025` (rung 82's `ds_fine`) |
|---|---|---|
| branch behaviour | smooth, monotone, ≈ −1 % per 0.25 % τ step | same |
| argmin handovers on `[0.017, 0.023]` | **one**, `s` 0.0850 → 0.0800 | **one**, same place, `s` 0.0825 → 0.0800 |
| jump in `F` there | **−19.40 %** | **−11.25 %** |
| bisected answer | 0.019754 | 0.019465 — **moved by 2.891e−4 = exactly ONE bracket width** |

### 3.2 The sign change is a JUMP, not a crossing

21 points at `1.25e−5` spacing — 50× finer than § 3.1's ladder, 23× finer than the bisection's own
final bracket, and placed inside it:

    τ = 0.0197750    g = F − τ = +1.6498e−03    s_bind = 0.0850
    τ = 0.0197875    g = F − τ = −2.4264e−03    s_bind = 0.0800   ← argmin HANDOVER

Exactly **one** sign change in the interval, and it coincides with the handover. The smaller of the
two residuals is **132× the τ step**: `g` does not approach zero, it **steps across it**.

**So at `r = 0.25` on the shipped grid the fixed point DOES NOT EXIST.** The 13-march bisection's
answer, 0.019754, is the location of a **discontinuity** — a perfectly good sign-change locator, and
not a root. That is why the secant fails there even started at the bisection's own answer (§ 4):
there is nothing to converge to.

### 3.3 Existence is `ds`-dependent

At `ds = 0.0025` the upper branch **does** cross zero smoothly (`g` = +3.18e−4 at τ = 0.019250,
−1.31e−4 at 0.019500; root ≈ 0.019427, matching the fine bisection's 0.019465 inside its width).
Refining the march step **creates** the root, moves the answer by a full bracket width, and opens a
**new** handover at τ = 0.023. **Root existence is a property of the plant AND its resolution**, and
rung 82's `ds_fine` control — which its fingerprint arm deliberately switches off — is what sees it.

### 3.4 The contrast that closes it

At `r = 0.35`, the one ramp where the secant reached `1.7e−15`, `F` is smooth through the crossing
(`g` = +1.56e−4 at τ = 0.037000, −6.83e−4 at 0.037333) and the nearest handover is far away at
τ = 0.038667. **A genuine root on a smooth branch is exactly where a corrector works — and it is one
ramp of five.**

---

## 4. THE ITERATION, AND THE CAUSAL DISCRIMINATOR

Under a start rule fixed in writing before the run (`τ₀ = √(lo·hi) = 0.034641`, `τ₁ = 1.25 τ₀`,
cap 6, clamps counted, flat pair aborts):

| ramp | root | start/root | 1 %@ | 5 %@ | final \|g\| | final err | bisection's own res. |
|---|---|---|---|---|---|---|---|
| 0.20 | 0.01108 | 3.13× | — | 6 | 8.2e−04 | 5.35 % | ±1.30 % |
| 0.25 | 0.01975 | 1.75× | — | 5 | 3.5e−03 | 4.55 % | ±0.73 % |
| **0.35** | 0.03710 | **0.93×** | **5** | 3 | **1.7e−15** | **0.10 %** | ±0.39 % |
| 0.50 | 0.06109 | 0.57× | — | 7 | 1.5e−02 | 1.58 % | ±0.24 % |
| 0.70 | 0.09202 | 0.38× | — | — | 1.5e−01 | 21.63 % | ±0.16 % |

**P6 refuted, 1 of 5.** Zero clamps at every ramp; **0 of 5** descend monotonically.

**At `r = 0.35` the "0.10 % error" is the BISECTION's.** The secant drove the residual to `1.7e−15`
in 5 marches; the 13-march bisection only located the root to ±0.39 %. Where a root exists on a
smooth branch, the corrector is the *more* accurate of the two.

**THE CONTROL — an intervention, scoring nothing.** Re-running the same secant with the start moved
onto each root converts **3 of the 4 failures**: `r = 0.20` (\|g\| = 8.0e−16, 0.49 % against a
±1.30 % resolution), `r = 0.50` (1.5e−14, 0.08 % / ±0.24 %), `r = 0.70` (6.2e−08, 0.12 % / ±0.16 %)
— all inside the bisection's own resolution. **`r = 0.25` still fails** (\|g\| = 5.3e−3, 9.59 %),
oscillating from the root itself.

So the five outcomes had **two causes**, and the intervention separates them: three were the
**start** (§ 1.3's lever arm, curable only by knowing the answer), one was the **shape** (§ 3, not
curable at all).

---

## 5. WHAT IT COSTS, PAIRED WITH ITS BASELINE (rung 63's recorded over-claim)

- rung 82's fixed-point bisection alone: 2 endpoints + 10 midpoints + 1 re-scan = **13 marches**
- rung 82's `threshold_law` **row**: 13 + 13 + 1 = **27**, or **40** with the `ds_fine` control
- the estimators: **1 march** (borrowed slope) · **2** (measured) · **8** (capped iteration)

The comparison is **13 → 1/2/8** against the **fixed point**. The seam's "~40" is a whole row
including the measured bisection a predictor never has, and is never used as the bar.

---

## 6. THE REDUCE CONTRACT — AN IDENTITY

No state, no knob, no constant. Every march this rung runs is `ThresholdLawTransient._scan`
unchanged; `c` is measured, `1/(1−c)` is rung 77's own scalar, and the ladder, spacing and start
points are reader settings in the sense rung 82's `tau_refs` and `bracket` are. So the reduce is
**bit-for-bit**: at rung 80's walls and rung 81's clocks this class's march is
`ThresholdLawTransient`'s, and `corrector_read(τ)["scan"]` is `_scan(τ)` — the same dict, same
values, every key.

---

## 7. HONEST SCOPE

- **One plant, one coordinate.** Everything here is the `demand` coordinate at rung 80's split
  wall. § 3's mechanism (a residual built as a `min` jumps at handovers) is general to rung 82's
  reader; that it produces a **missing root** at 1 of 5 ramps is measured on this rig only.
- **`ds`-dependence is shown, not mapped.** § 3.3 shows the root at `r = 0.25` appearing between
  `ds = 0.005` and `0.0025`. Which ramps have roots at which steps is a two-parameter sweep this
  rung does not run.
- **The bisection is not vindicated as *correct*.** It returns a well-defined sign-change location
  whether or not a root is there. § 3 says that is a *feature* relative to a corrector; it also
  means rung 82's `r = 0.25` row reports a discontinuity where its text says fixed point.
- **`r = 0.35`'s 5-march success is one sample of one arbitrary start** that happened to land on a
  root. The control (§ 4) shows 3 of 4 other roots also converge when started at them. It is a
  **polisher, not a predictor** — and the bracket that tells it a root is there is the expense it
  was meant to remove.

---

## 8. NEXT SEAMS

* **WHICH RAMPS HAVE ROOTS, AND AT WHICH `ds`.** § 3.3 shows existence flipping between the two
  shipped steps at one ramp. The `(r, ds)` map — where sign changes are crossings and where they
  are handovers — is unmeasured, and it decides whether rung 82's five-row table contains one
  discontinuity or several.
* **THE HANDOVER AS THE OBJECT.** If the bisection is really locating an argmin handover, then the
  quantity rung 82 has been calling a threshold may be better defined as *"the ramp point at which
  the binding trajectory point changes hands"* — which is a plant property, not a solver artifact,
  and would be readable without any bisection at all.
* **A BRACKET FROM ONE MARCH.** § 2 makes `sign(h)` free, and § 4 shows the secant is excellent
  given a tight bracket. Whether the side plus the forward reading bounds the root well enough to
  seed a short bisection — a hybrid costing ~6 marches rather than 13 — is untested, and it is the
  only route left to the seam's original ambition.
* **A FINGERPRINT ARM FOR THIS RUNG.** Rung 82's three arms are bit-exact because no Jacobian is
  built; this rung builds none either, but § 3.2's residuals sit at `1.6e−3` beside a τ step of
  `1.25e−5`, and § 4's traces reach `1.7e−15` — a genuinely small live value, unlike rung 82's,
  where the binding number turned out to belong to the search.
* Everything rungs 72–82 §§ 8–11 leave, unchanged by this rung.
