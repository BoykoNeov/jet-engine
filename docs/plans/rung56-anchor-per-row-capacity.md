# Rung 56 anchor — PER-ROW CAPACITY: the probes, the predictions as written, and their scoring

Rung 55 § The next seam, verbatim:

> *"**Per-row capacity — rung 54's channel, per stage.** `X(v) = m·sqrt(1+v²)` is a face
> quantity; in a stack each row has its own throat and its own `X_k`, and P4 says the REAR rows
> are the ones driven to high `φ`. Rung 54's `capacity_margin` should therefore **bind at the
> back** while its incidence margin binds at the front — the two rungs' objects on the same
> machine, in opposite places. It needs a `C` per row."*

Order of work, as the project requires: **probes first** (they fix the instrument and the sign),
**then** the predictions, **then** the measurement that scores them.

---

## PROBE A — the instrument. Is "a `C` per row" really `K` new constants?

The seam says the channel *"needs a `C` per row"* — which reads as `K` disclosed constants, a
serious dilution of rung 54's single-constant discipline. Probe A asks whether the stack's own
design ladder already fixes the PROFILE, leaving only a LEVEL.

**It does.** At design the stack sizes every annulus so `φ_k` = 1, so `Vx_k` = `U_k`; on a
constant mean radius `U_k` = `U`, hence **every row's design throat velocity is the same** while
`Tt_k` climbs the ladder. With `ν ≡ V/sqrt(γ R Tt) = M/sqrt(1+(γ−1)/2·M²)`:

```
    ν_k = ν_1 / sqrt(θ_k,d)                     [DERIVED off the stack's OWN design ladder]
    M_k = ν_k / sqrt(1 − (γ−1)/2·ν_k²)
    C_k = MFP(M_k)/MFP(1)                       [rung 54's own constant, per row]
```

One disclosed LEVEL (the front row's `C`, i.e. rung 54's constant unchanged — its row was
already *"one row at the compressor face"*), `K−1` rows **derived**. Rung 54's pattern exactly:
shape derived, level disclosed. `θ_1,d` = 1 ⇒ `C_1` = `C` identically, so the K = 1 reduce is
exact and γ-independent.

Measured profiles (`K` = 8, `dT` split, front-row `C` = 0.90 ⇒ `M_th0,front` = 0.678):

```
   LP  M_k = 0.678 0.660 0.643 0.628 0.613 0.600 0.587 0.575    C_K/C_1 = 0.911
   HP  M_k = 0.678 0.646 0.617 0.592 0.570 0.550 0.532 0.516    C_K/C_1 = 0.848
```

**This is the probe's content: the derived profile FIGHTS the seam.** The rear rows are designed
with *more* capacity margin (lower Mach) exactly where the off-design loading drives them
hardest. Against it, the per-row corrected flow `m_k = φ_k·n_k` (an exact identity, the face
relation `m = φ·n` per row):

```
   Tt4=800  LP  m_k = 0.4631 … 0.6728    m_K/m_1 = 1.4527   (φ_K/φ_1 = 1.3597)
   Tt4=800  HP  m_k = 0.7058 … 1.0957    m_K/m_1 = 1.5523   (φ_K/φ_1 = 1.4322)
```

so `m_k` runs **ahead of `φ_k`** (the rear rows' corrected SPEED rises too, `n_k` = `n·sqrt(θ_kd/θ_k)`).
The contest is real and its outcome is not decidable by inspection. **This is why the instrument
had to be probed before the prediction was written.**

**A second probe finding, unforced:** the naive UNIFORM profile (`C_k` = `C`, rung 54's constant
applied per row without the ladder) puts the HP rear row at `M_c` = **+0.0139** at `Tt4` = 800,
`C` = 0.90 — a hair from choking. The derived profile puts it at **+0.1636**. The disclosed
profile therefore carries the LEVELS, and no level claim may be made robust to it (contrast rung
55 P6, where the split carried nothing).

## PROBE B — the two things that must be measured before a headline is fixed

**B1 — the design-point float drift.** Under a uniform profile every `X_k` ties at design. It
does not tie in floating point: `max|X_k − 1|` = 7.8e-15 … 1.9e-14 over `K` = 2…16 on both
spools. So `X_k`(design) = 1 is a **tolerance, not an identity**, and any binding-row assertion
at or near design under the uniform profile is noise-driven and must not be gated.
(`argmin` is row 0 on all eight cells here, but only by a few ULP.)

**B2 — THE STATOR SIGN, which fixes the headline.** The front-row lever reaches an unmoved rear
row only indirectly, through the solved `(m, n)`. Measured (`K` = 8, `vsv_stages` = 1, derived
profile, `C` = 0.90), rear-row `M_c`:

| `Tt4` | spool | `v`=0 | 0.20 | 0.354 | 0.60 |
|---|---|---|---|---|---|
| 1000 | LP | +0.3773 | +0.3759 | +0.3748 | +0.3730 |
| 800 | HP | +0.1636 | +0.1584 | +0.1541 | +0.1469 |

**The lever DEBITS the row it does not move**, monotonically — and the channel is the shaft
speed, the one thing every stage shares (rung 55's own honest half, now in a second currency).
The debit is small where the lever does not act and large where it does (LP `Tt4`=1000,
`v`=0.354: front loses 0.0313, rear loses 0.0025 — 12.5×), and rung 53's LUMPED lever at the
same setting costs the rear **0.0342, i.e. 13.7× the front-only lever's rear debit.**

**B3, unforced:** at large `v` the binding capacity row **relocates to the moved row itself**
(LP `Tt4` = 1200 flips 7 → 0 between `v` = 0.354 and 0.60). Rung 50's shape — a lever relocating
the extremum to itself — in a currency rung 50 never saw.

---

## THE PREDICTIONS, as written before the measurement that scores them

Everything above is a PROBE and is scored as nothing. The following six are predictions.

### P1 — the seam's own claim, and where it breaks
Under the **derived** profile the binding capacity row is the **REAR at part power** but the
**FRONT at and near design**, with a crossover throttle in between, on **both** spools and on
**all five** disclosed shapes. Under the **uniform** profile it is the rear at every off-design
throttle (design excluded by B1).
*Band:* the LP crossover lies inside `Tt4` ∈ [1200, 1500] at `C` = 0.90; the seam's "binds at the
back" is therefore **HIT at part power and REFUTED near design**, and the refutation is
*derived*, not a modelling artifact.

### P2 — the AMPLIFICATION is the non-tautology gate
There is no feedback leg (rung 54 P1 is inherited: the channel enters no solver), so what makes
this content is RESOLUTION. At the same solved state the **binding row's** margin is materially
tighter than rung 54's **face** margin, **exactly 0.0 at `K` = 1**, growing monotonically with
throttle depth.
*Band:* amplification (face deficit `1−M_c^face` vs row deficit) ≥ 1.15× on the HP at
`Tt4` = 800, `K` = 8, on every shape and both splits. **Predicted to be larger under `uniform`
than `derived`** (the derived profile is protective).

### P3 — `K` is a resolution here too
The amplification grows with `K` but its **increments shrink** (rung 55 P5's first-order
convergence), so no claim rides on a particular `K`.
*Band:* the `K`-increment at least halves over `K` = 2 → 4 → 8 → 16 on the HP at `Tt4` = 800.

### P4 — the SPLIT is load-bearing here, unlike rung 55
The amplification rides on the internal `θ`/`ϖ` ladder, which is precisely what the disclosed
split moves. So — against rung 55 P6's pattern — I predict the `"dT"` / `"tau"` split moves the
amplification **measurably** (> 2 % relative on the HP at `Tt4` = 800, i.e. an order of magnitude
worse than rung 55 P6's 0.01 %), while leaving **every SIGN and every binding-row identity**
unchanged.

### P5 — the two constraints land at OPPOSITE ENDS **and on opposite SPOOLS**
On one machine at part power: the worst **incidence** margin is the **LP FRONT** row (rung 55 P4,
inherited) and the worst **capacity** margin is the **HP REAR** row. Shape-robust on all five.
*This is the strong form of the seam*, which predicted only front-vs-back on one machine.
**And it CORRECTS rung 54**, which wrote *"the HP schedule's demand … never approaches its throat
at any throttle"* — true at the face, false at the rear row. Rung-28 shape: resolution corrects
the verdict, the face-level reasoning survives as a face-level statement.

### P6 — the positional lever's THROAT debit factorises like rung 55's speed cost
Rung 55: `dN_ratio` = `(1/K)·(v*_front/v*_lumped)`. The rear row's throat debit is transmitted by
`n` alone, so I predict the **front-only/lumped rear-debit ratio tracks the same `dN` ratio to
within 25 %**, at matched `v`. Measured seed (B2): 0.0025/0.0342 = **0.073** at `v` = 0.354,
`K` = 8, against rung 55's `dN` ratio 0.035 at its own `v*` — a factor ~2 apart at matched-`v`
rather than matched-target, so this one is genuinely open.

---

## SCORING — as written, four HIT, one band MISS, one REFUTED

| | claim | verdict |
|---|---|---|
| **P1** | migration front→rear, crossover in [1200,1500] on the LP | **HIT** — LP crosses 1400/1300 on all five shapes, HP 1300/1200; uniform binds rear at every off-design throttle. The seam is HIT at part power, REFUTED near design. |
| **P2** | amplification ≥ 1.15 on HP@800 every shape/split, 0 at `K`=1, uniform > derived | **HIT** on all three clauses — 1.31–1.32 derived / 1.55 uniform, exactly `1.0` at `K`=1. |
| **P3** | increments at least halve, HP@800 | **BAND MISS / CLAIM HIT** — LP ratios 0.48/0.50/0.50/0.50 (halves); HP 0.52/0.56/0.55/0.53 (does not). First-order convergent either way. The gate is set at the measured 0.60. |
| **P4** | split moves it > 2 % on HP@800, no sign or binding-row change | **HIT on the band** (3.7–4.6 %, ~400× rung 55 P6). **Second clause FAILS** near the crossover: `press/flow` HP@1200 binds row 2 (`dT`) vs 3 (`tau`), spread ~1e-3. |
| **P5** | opposite ends AND opposite spools, five shapes; rung 54's HP claim corrected | **HIT** — 5/5. Incidence worst LP row 0 (0.349); capacity worst HP row 7 (0.164). Face relaxes with throttle while the rear row tightens; `C*` = 0.913 at `Tt4`=800. |
| **P6** | throat-debit ratio tracks rung 55's `dN` ratio within 25 % | **REFUTED** — within 9 % at `v`=0.20, off by 39 % at 0.354 and 62 % at 0.60. The `dN` ratio is `v`-invariant (0.117–0.122); the throat ratio collapses (0.133→0.045). **The refutation is the rung's fourth law.** |

**One prediction I did not write and should have.** P4's "binding-row identity unchanged" clause
was written without noticing it contradicts P1: a *migration* means there are throttles where two
rows are nearly tied, and near a tie any perturbation moves the argmin. The clause was doomed by
P1's own content. Recorded rather than quietly dropped.

**One gate I wrote that over-reached its prediction.** The first version of the P4 gate asserted
> 2 % on *every* spool × throttle cell, not the HP-800 cell the prediction named; `flat-eta` LP at
`Tt4` = 1000 landed at 1.98 % and failed it. The band was not moved to rescue a claim — the
pre-registered cell is gated at 2 % and the wider sweep at its **measured** floor (1.7 %), with
both disclosed in the gate.
