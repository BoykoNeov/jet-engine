# Rung 84 — THE MARCHED MINIMUM'S STAIRCASE

**The seam:** `docs/rung83-spec.md` § 8, first bullet — *"WHICH RAMPS HAVE ROOTS, AND AT WHICH `ds`.
§ 3.3 shows existence flipping between the two shipped steps at one ramp. The `(r, ds)` map — where
sign changes are crossings and where they are handovers — is unmeasured, and it decides whether rung
82's five-row table contains one discontinuity or several."*

**Plant:** `StaircaseLawTransient` (`turbojet/engine.py`) — **no state, no knob, no constant**, the
**fifth** reader-only rung (77, 81, 82, 83, 84). **Anchor + pre-registration:**
`docs/plans/rung84-anchor-staircase-law.md`. **Gates:** `tests/test_rung84.py`.

**THE MAP IS MEASURED — and the mechanism under it names an object rung 83 did not have.**

> **HEADLINE — A MINIMUM OVER A MARCHED SET IS NOT A MINIMUM AT ALL; IT IS AN EVALUATION ON A
> MOVING GRID BOUNDARY, SO THE RESIDUAL CARRIES THE MARCH'S OWN SAWTOOTH.** A `min` over a
> **fixed** finite set of continuous functions is continuous — it *kinks* at a handover and cannot
> *jump*. Rung 83's jump is the four-loop window **opening one march step earlier**; the entering
> point binds immediately because the argmin **is** the window's leading point (71 of 71). So
>
>     h(τ; ds) = h_true(τ) + ĥ′·δ(τ, ds) + O(ds²),      δ = s_edge − s* ∈ [0, ds)
>
> — the true residual plus a **sawtooth** of amplitude `ĥ′·ds` on a lattice of tread `ds/|ds*/dτ|`.

**CROSS-RUNG.** Rung 83's `argmin_moved` fired **correctly** and reported a **consequence** — an
edge move forces an argmin move, never the reverse. **Verdict CONFIRMED, reason CORRECTED**, rung
28's shape. Rung 82's residual is **BOUNDED** (its `min` never binds away from the window's first
point), and rung 82 § 2's **V5 gets the scale it lacked**.

---

## 0. HONESTY FIRST — THREE OF SEVEN PREDICTIONS REFUTED, TWO FOR THE SAME REASON

**The pre-check settled the mechanism before the anchor was written** (anchor § 0, E1–E6; rungs
81/82/83's precedent, and the rung takes no credit for it). It reproduced rung 83 § 3.2's published
`g` exactly and then split its step in two: **99.4 % membership term, 0.6 % smooth term**, and the
sign change **does not survive** restriction to the points the two marches share.

| # | predicted | measured |
|---|---|---|
| **P1** | the argmin is the window's **leading** point, ≥ 95 % | **CONFIRMED — 100 %**, 71 of 71 across five ramps × both shipped steps × both branches, plus 41 of 41 on § 3's ladder |
| **P2** | the classifier is **exact**, no threshold, both directions | **CONFIRMED** — membership term **exactly `0.0`** on every equal-set pair and non-zero on every changed-set pair; 40 ladder pairs and all 10 map cells |
| **P3** | the jump count **doubles** per halving, ±25 % | **REFUTED** — 3.00 / 1.67 / 2.00 / 1.90. **Its content confirmed instead** (§ 3) |
| **P4** | a counter-example exists (argmin moves, set does not) | **REFUTED — 0 of 40**, which the anchor pre-registered as the **stronger** reading (§ 2.2) |
| **P5** | `Λ` is `ds`-invariant within ±30 % | **REFUTED** — 60.9 % spread on the registered estimator. **Same defect as P3** (§ 5) |
| **P6** | root existence is **not** monotone in `ds` | **REFUTED — absent / present / present / present** over four steps at `r = 0.25` (§ 6) |
| **P7** | 1–4 of 10 cells rootless **and** a failing ramp's identity changes | **SPLIT** — 1 of 10 ✓; the identity clause **REFUTED**, no ramp newly fails (§ 6) |

### 0.1 P3 and P5 died of the SAME defect, and it is the third bar in this lineage to die of wording

Both compared **ratios of small integers**. P3's bar is a ratio of jump counts; P5's `Λ` divides by a
tread estimated as *window / count*. The counts are **1, 3, 5** — and a count of grid crossings is
quantized to **±1**, i.e. **±100 %** and **±33 %** before any physics enters. Neither bar could have
passed on a correct plant.

**A SMALL INTEGER COUNT CANNOT CARRY A RATE.** Rung 82's P1 and rung 83's P5 both died of bars that
named a direction or permitted choosing a reference afterwards; this pair names a *quantity* whose
resolution is coarser than the effect. Same family, third occurrence. The repair for each is in its
own section and **neither is re-scored into a pass**.

### 0.2 The Λ estimator that refuted P5 was in the shipped reader, and is corrected in place

`staircase_number` originally derived its tread from `lattice_count`'s quantized `n_jumps`. It now
takes `spacing` as an **argument**, and returns the factors with no `lam` when none is given — so
the choice is visible in the signature rather than hidden in a divide whose output a spec would
then silently correct. That is slice 4's *"characterises an ADJACENT computation"* trap, caught
before it shipped.

---

## 1. THE DERIVATION — zero new constants

### 1.1 The residual is a BOUNDARY reading (P1)

Rung 82 builds `F(τ) = min_{s ∈ S(τ)} ĥ(s;τ)/κ` over `S(τ)` = the interior points riding four loops.
**The minimum is attained at the window's FIRST point at 100 % of every march this rung ran** — 71
placed samples plus a 41-point ladder, five ramps, both shipped steps, both branches. So

    F(τ) = ĥ( s_edge(τ); τ ) / κ

and rung 82's `min` is **decorative**: the object rungs 82 and 83 have been solving is an evaluation
on a moving **boundary**. Every section below rests on this, and it is why the summand profile
matters at all (§ 4).

### 1.2 The boundary is on a GRID, so the residual is a SAWTOOTH

The four loops engage at a **continuous** `s*(τ)`; the march samples at spacing `ds`; so
`s_edge(τ) = ceil(s*(τ)/ds)·ds` — **measured on the grid at 71 of 71** (`edge_on_grid`, V3). Writing
`δ = s_edge − s* ∈ [0, ds)`,

    h(τ; ds) = ĥ(s*(τ); τ) + ĥ′·δ(τ, ds) + O(ds²)

`δ` falls smoothly as `s*` descends and **jumps up by `ds`** at each grid crossing: a sawtooth. Two
consequences follow with no further physics, and each is measured separately below:

* **COUNT** — jumps over `[τ_a, τ_b]` number `(s_edge(τ_a) − s_edge(τ_b))/ds`, an **integer from two
  marches** (§ 3).
* **TREAD** — the τ-spacing between jumps is `ds/|ds*/dτ|`, i.e. **∝ ds**.

### 1.3 The staircase number

Rise `= ĥ′·ds/κ`; tread's drift `= |dg/dτ|·ds/|ds*/dτ|`. So

    Λ = rise / tread = (ĥ′/κ) · |ds*/dτ| / |dg/dτ|

**contains no `ds`** — three plant slopes. Its content is that jumps shadow `Λ/(1+Λ)` of the
residual's own range: where a crossing lands on a jump rather than a tread, there is no root.

---

## 2. WHAT THE JUMP ACTUALLY IS — and the instrument that says so exactly

### 2.1 The classifier is an identity, not a threshold (P2)

On the points two marches **share**, the minimum is a min over a *fixed* set and is continuous by
construction. So splitting `Δh` into a **smooth** term (the common points) and a **membership** term
(the rest) needs no tolerance:

| | `r = 0.25` — rung 83's JUMP | `r = 0.35` — rung 83's CROSSING |
|---|---|---|
| scored set | **gains `s = 0.080`** (48 → 49) | unchanged (43 → 43) |
| `Δh` total | `−1.223e−02` | `−2.515e−03` |
| **membership term** | **`−1.216e−02` (99.4 %)** | **exactly `0.0`** |
| sign change survives restriction to the shared points? | **NO** (`+4.949e−3 → +4.879e−3`) | YES |

**A sign change is a CROSSING iff it survives restriction to the common points.** Measured over 40
ladder pairs and all 10 map cells: the membership term is `0.0` **exactly** whenever the sets agree
and non-zero whenever they differ, in both directions and with no ties. This **replaces rung 83
§ 3's `ratio = smallest_g/step`**, a number its own docstring had to call *"reported and never
thresholded into a verdict here"*.

### 2.2 P4 refuted into the stronger reading — the three flags are ONE event

P4 predicted at least one interior handover (argmin moves, set does not) and found **0 of 40**; the
mirror count is **0** as well. With P1 holding, this is not a coincidence but a **construction**: the
argmin *is* the edge, so it can only move when the edge moves, and the edge only moves when a point
enters. `argmin_moved`, `edge_moved` and `set_changed` are the same event on this plant.

**So rung 83's flag was right and its reason was the consequence.** It fired at exactly the right
place; what it recorded was the *effect* of the set changing. Rung 28's shape — verdict CONFIRMED,
reason CORRECTED — and **not** "rung 83 was wrong about the `min`."

---

## 3. THE COUNT — P3 refuted, its content confirmed over five steps

Each jump moves the leading point by exactly one grid step, so the jump count over a window is the
edge **index** difference: **two marches, an integer, nothing to miss between samples**. Certified
against the expensive instrument — a 41-point ladder at `ds = 0.005` counts **1**, and so does the
two-march arithmetic.

`τ ∈ [0.016, 0.024]`, `r = 0.25`:

| `ds` | 0.005 | 0.0025 | 0.00125 | 0.000625 | 0.0003125 |
|---|---|---|---|---|---|
| jumps | 1 | 3 | 5 | 10 | 19 |
| `n·ds` | 0.00500 | 0.00750 | 0.00625 | 0.00625 | 0.00594 |
| band `[(n−1)ds, (n+1)ds]` | [0, .010] | [.005, .010] | [.005, .0075] | [.005625, .006875] | [.005625, .00625] |

**The five bands INTERSECT at `[0.005625, 0.006250]`** — one plant number `Δs* ≈ 0.0059`, pinned
across a **16× range of `ds`**, with the count exactly `Δs*/ds`. An empty intersection would have
falsified § 1.2 outright.

**P3's ratio bar is refuted and the refutation is instructive**: 3.00 / 1.67 / 2.00 / 1.90 — the two
ratios taken at counts ≥ 5 *are* the doubling; the two at counts of 1 and 3 measure the ±1
quantization. V4 (a monotone edge) is certified on the same ladder: indices `[17×19, 16×22]`.

---

## 4. WHY THE RISE IS FIRST ORDER — the summand profile, read directly

`edge_read` returns every summand keyed by `s`, so the profile near the boundary costs **one march**
and no new machinery. At `r = 0.25`, `τ = 0.0197`, the first cell above the edge:

| `ds` | 0.005 | 0.0025 | 0.00125 |
|---|---|---|---|
| `ĥ′·ds` (first-cell difference) | `1.6025e−02` | `7.1929e−03` | `3.3980e−03` |
| halving ratio | — | 0.4489 | 0.4724 |
| **implied exponent** | — | **1.156** | **1.082** |

**The profile is smooth at the boundary with a finite slope**, and the exponent is converging to 1.
The same reading explains something rung 82 recorded without a cause: `h` at this τ is `+5.37e−3`,
`−1.51e−3`, `−4.85e−3` at the three steps — moving at slopes **2.75** and **2.67**, which is `ĥ′`.
**The residual's LEVEL is `ds`-dependent at first order**, because a boundary reading samples a
sloped profile at a point that moves with the grid.

---

## 5. THE STAIRCASE NUMBER — P5 refuted, and what survives it

**As registered, P5 fails: `Λ` = 0.2672 / 0.5122 / 0.4276, spread 60.9 %.** The defect is § 0.1's:
its tread divides the window by a count of 1, 3, 5.

Recomputed with the spacing taken from § 3's converged `Δs*` — *the same window, so the three values
now share a factor by construction and this is a diagnosis, not a pass*:

| `ds` | rise | `\|dg/dτ\|` | spacing | tread | `Λ` |
|---|---|---|---|---|---|
| 0.005 | 4.0534e−03 | 1.8962 | 6.7368e−03 | 1.2775e−02 | 0.3173 |
| 0.0025 | 2.6054e−03 | 1.9075 | 3.3684e−03 | 6.4254e−03 | 0.4055 |
| 0.00125 | 1.2800e−03 | 1.8710 | 1.6842e−03 | 3.1512e−03 | 0.4062 |

**`Λ`'s `ds`-independence rests on its FACTORS, not on this spread.** Each is measured on its own and
each is first order or flat:

* rise halving ratios **0.643 / 0.491** — the second is § 1.3's 0.5; the first compares events at
  different `s` (the entering point is `0.080` there, `0.0825` at both finer steps) where `ĥ′` differs
* `ĥ′` first order, exponent **1.156 → 1.082** (§ 4)
* `|ds*/dτ|` one plant number over 16× (§ 3)
* `|dg/dτ|` **flat at 1.8710–1.9075** across a 4× step change

Taking the two finest, **`Λ ≈ 0.406`**, so jumps shadow **`Λ/(1+Λ) = 28.9 %`** of the axis.

---

## 6. THE MAP — the seam's question, answered; P6 and P7's second clause refuted

Five ramps × both shipped steps, each cell bisected with **rung 82's own `_bisect`** and then
classified by § 2.1's exact criterion:

| `r` | 0.20 | 0.25 | 0.35 | 0.50 | 0.70 |
|---|---|---|---|---|---|
| `ds = 0.005` | crossing | **JUMP** | crossing | crossing | crossing |
| `ds = 0.0025` | crossing | crossing | crossing | crossing | crossing |

**1 of 10 cells has no root**, and it is rung 83's own `r = 0.25` at the shipped step — the
classifier reproduces that rung's bisected answer `0.019754` and says, by an identity rather than a
judgement, that it is not a root. Every one of the nine crossings carries a membership term of
**exactly `0.0`**. **So rung 82's five-row table contains exactly ONE discontinuity, not several** —
which is what the seam asked.

**P6 IS REFUTED.** At `r = 0.25` over four steps: **absent / present / present / present**
(`ds` = 0.005 / 0.0025 / 0.00125 / 0.000625; the two finest land in the same bisection cell). The
absence does **not** come back. §§ 1–5 derive that the shadow *fraction* is `ds`-invariant and that
refinement re-lays the lattice rather than shrinking the shadow — **but one ramp over four steps
cannot test a 29 % rate, and this rung does not claim it does.** Rung 83 § 3.3's reading (*"refining
the march step creates the root"*) survives its own re-measurement.

**P7 SPLITS.** The count clause holds (1 of 10, inside 1–4); the **identity clause is refuted** — no
ramp newly fails at the finer step, the failing set simply empties.

### 6.1 Rung 82's V5 gets the scale it lacked

Rung 82 § 2 records that V5 *"compared that move against the bisection width and trips at `r = 0.35`,
which is P1's error a second time and is reported, not repaired."* It had no number for how large a
**benign** move should be. § 1.2 supplies one: the root inherits the sawtooth, so it may shift by up
to `(ĥ′/κ)·ds/|dg/dτ| ≈ 0.48·ds`. Across the two shipped steps that bounds the move at **3.6e−3**.

Measured, in bisection widths (`2.891e−4`): **0, 1, 5, 0, 1**. All five are inside the derived bound,
the largest — `r = 0.35`, the very row V5 voids — at **40 %** of it. **A threshold may move many
bisection widths for an entirely benign reason**, so V5's bar is not merely the wrong currency (rung
82's own diagnosis) but *tighter than the mechanism permits*.

---

## 7. THE REDUCE CONTRACT — AN IDENTITY

No state, no knob, no constant. Every march is `ThresholdLawTransient`'s, reached through
`_scan_cells` — that method's own body, extracted so this rung can see the point identities it
reduces to counts. **The extraction is a pure refactor and it is CERTIFIED, not asserted:** rungs 82
and 83's 24 tests pass, and the three `r82`/`r82r`/`r82t` fingerprint arms — 440 values at
`TOL = 0.0` against committed CPython goldens — pass unchanged. `edge_read(τ)["h"] == _scan(τ)["h"]`
is gated at zero tolerance.

---

## 8. HONEST SCOPE

- **One plant, one coordinate, one flight condition.** Everything is the `demand` coordinate at rung
  80's split wall. § 1's derivation is general to any residual built as a `min` over a marched set;
  that the argmin is *always* the boundary is measured **on this rig only** and is what makes the
  sawtooth the whole story here.
- **`Λ` is measured at ONE ramp.** `r = 0.25`, three steps, one lattice event each. `ĥ′` varies with
  `s`, so the three events are not the same point of the profile — which is the honest reading of the
  0.643 rise ratio, and the reason § 5 leans on the factors.
- **The 28.9 % shadow is DERIVED, not tested.** Ten cells with one absence cannot distinguish 29 %
  from 10 %; the two numbers are quoted apart deliberately and no agreement is claimed.
- **The map is 10 cells at 2 steps.** `Δs*` is pinned over five steps at one ramp; whether
  `|ds*/dτ|` is comparable at the other four is not measured.
- **The root is still drifting at the finest step tested.** `r = 0.25` moves 0.019754 → 0.0194648 →
  0.0188867 → 0.0188867, i.e. 4.4 % from coarsest to finest. Inside § 6.1's bound at every step, and
  **not converged** in rung 82's own currency.

---

## 9. NEXT SEAMS

* **THE SHADOW RATE, TESTED.** `Λ/(1+Λ) = 28.9 %` predicts how often a crossing misses a tread. It
  needs ~50 independent cells to separate from 10 %, and the ramp axis has only five useful points —
  so it needs a second axis (the wall pair, `τ_gov`) or a different plant.
* **`Λ` AT THE OTHER FOUR RAMPS.** § 5's three slopes are all measurable in ~20 marches per ramp, and
  whether `Λ` is a property of the plant or of the ramp decides whether § 6's map has one rate or five.
* **THE SAWTOOTH REMOVED.** `δ` is computable if `s*` is: a march that interpolates the four-loop
  boundary instead of sampling it would carry `O(ds²)` and no sawtooth at all. That is a change to
  rung 80's `_riding4`, not a reader — the first non-reader rung in this lineage since 80.
* **RUNG 82's V5, REPLACED.** § 6.1 derives a bar to replace the bisection width. Re-running rung
  82's own table against it is cheap and would say whether any of its five rows is genuinely
  unresolved.
* Everything rungs 72–83 §§ 8–11 leave, unchanged by this rung.
