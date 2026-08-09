# Rung 77 anchor — THE STIFFNESS LEDGER (rung 76 § 8's third seam)

Scored in `docs/rung77-spec.md` § 8. The rule this project runs under: **a prediction that is
edited after the measurement is not a prediction.**

The seam, in rung 76 § 8's own words: ***THE `1/(1−c)` GAIN AS A DESIGN VARIABLE.** § 3 makes
stiffness a measurable property of how a limiter is written. Every other set-point solve in this
family (`_topping_fuel`, `_surge_fuel`) has one and it has never been read.*

**This rung builds it and expects to REFUTE the seam's own wording.** § 0 below is everything
measured before this document existed (`M:\claud_projects\temp\rung77\probe_gw.py`,
`probe_gw2.py`) and **none of it is scored**. § 1 is derived on paper from the inherited laws and
is listed as derivation, not prediction, except where § 8 finds a derivation measured false (rung
72's D5 precedent). § 2 is scored. **No `dw*/dq` has been read for any leg, in any cell.**

---

## 0. WHAT WAS MEASURED BEFORE THIS DOCUMENT EXISTED — UNSCORED

Settings throughout: `φ_lim = 0.80` (the inherited Jacobian floor), `margin = 0.10`,
`Tt4_max = 1200 K`, all clocks `0.05`, `ds = 0.005`, the `StatorLimiter` arm, read at the 76
`_riding4` points of the `clip | sched | none | solve` march, every 8th.

### 0.1 The three residuals, and their slopes

    accel (48)  G_a(w) = w − cap(w)              G_a′ = 1 − c          dimensionless
    gov  (46)   G_g(w) = Tt4(w) − Tt4_max        G_g′ = ∂Tt4/∂w        K per kg/s
    φ    (49)   G_s(w) = φ_lim − φ_lp(w)         G_s′ = −∂φ_lp/∂w      φ per kg/s

| | measured |
|---|---|
| `G_a′` | `0.80275 … 0.81434` |
| `G_g′` | `6.534e+04 … 7.034e+04` |
| `G_s′` | `9.9723 … 11.7196` |
| `‖(1 − G_a′) − c‖`, `c` from rung 76's own `_c_at` | `7.29e−11` |

`c ∈ 0.1857 … 0.1973` here against rung 76 § 0.3's `0.1790 … 0.1875` — **overlapping bands at
different sample sets, not a disagreement**, and § 2's P1 pins the instrument properly.

### 0.2 The units problem, and the one normalisation that is not an imposition

The three slopes **carry different units**, so a raw ordering of them is not a legal comparison.
Normalising each leg by **its own set point** — `w/w`, `w/Tt4_max`, `w/φ_lim`, every one of them
that leg's own already-imposed scalar and no new constant —

| leg | normalised slope | `1/‖n‖` |
|---|---|---|
| accel | `0.8027 … 0.8143` (`= 1 − c` exactly) | **`1.228 … 1.246`** |
| gov | `0.7166 … 0.7321` | `1.366 … 1.396` |
| φ | `0.1664 … 0.1738` | `5.752 … 6.011` |

**and the accel column is rung 76 § 3's measured gain `1.22799 … 1.24573`, digit for digit.**
That is what makes this a generalisation of rung 76 rather than a second imposition beside it.

### 0.3 Rung 64's DERIVED degeneracy, measured

Rung 64 states, explicitly **"DERIVED, not measured"**, that where the bleed valve rides it
re-pins `φ_lp` to `φ_lim` at any fuel, so `dφ/dW_f = 0` and `_surge_fuel`'s solve is degenerate
across its whole bracket. Read with `_b_state = None` (the valve re-solving) at the same states:

    φ_lp at 0.9·w, w, 1.1·w   =   0.8000000000  0.8000000000  0.8000000000     (φ_lim = 0.80)
    G_s′ closed               =   0.0e+00 exactly at 4 of 10 points, ‖·‖ ≤ 2.1e−08 at the rest

**against `G_s′` open `= 9.97 … 11.72`.** This is a CONFIRMATION of a derivation and is scored as
one — nothing here is a surprise, and § 8 must not report it as one. What is new is that the
degeneracy is **exact**, not approximate.

---

## 1. DERIVED ON PAPER — NOT SCORED AS PREDICTION

**D1. A set-point solve's sensitivity is a FORCING OVER A SLOPE.** For any `G(w, q) = 0` with
root `w*(q)`, the implicit function theorem gives

    dw*/dq  =  − G_q / G_w                                                          [D1]

with `G_w` the residual slope of § 0.1. This is dimensionally identical for all three legs —
kg/s per unit valve — so **`dw*/dq` is the ledger's legal currency and needs no normalisation at
all.** § 0.2's normalised table is retained only because it is what reproduces rung 76's gain.

**D2. `1/(1−c)` IS THE SLOPE HALF OF ONE LEG, NOT A GAIN A SOLVE BUYS.** Substituting
`G_a = w − cap(w, q)` into D1 gives `dw*_a/dq = (∂cap/∂q)/(1 − c)`, which is rung 76 § 3's
identity — and the `1/(1−c)` is `1/G_a′`, i.e. **the same object D1 gives every leg**. Rung 76
read it as an amplification only because the accel leg has a *second reading* (`sensed`) to put
in the numerator; the factorisation itself is generic.

**D3. THE OTHER TWO LEGS CANNOT HAVE A `1/(1−c)`, AND THE REASON IS STRUCTURAL.** `Tt4_max` and
`φ_lim` are **constants**, so `G_g′ = ∂Tt4/∂w` and `G_s′ = −∂φ/∂w` have **no `1` to subtract
from** and are dimensional. There is no `c`, and there is no second reading to difference
against — a floor on a STATE is not a formula for a FUEL, which is rung 76 § 0.1's own sentence
read one step further. **So rung 76 § 8's wording is expected to be REFUTED**, and D3 is the
refutation's content.

**D4. TWO ROUTES TO A SINGULARITY, AND THIS FAMILY REACHES ONLY ONE.** `dw*/dq` diverges iff
`G_w → 0`. For the accel leg that is `c → 1` (rung 76 § 8's fourth seam) and § 0.1 measures
`c ≈ 0.19`, nowhere near. For the φ leg it is `∂φ/∂w → 0`, which rung 64 **derives** happens
exactly where the valve rides — and § 0.3 measures it as an exact zero. **A limiter's stiffness
diverges either because its set point chases its own actuator or because another lever pins the
variable it watches; this family reaches the second and never the first.**

**D5. THE GOVERNOR CANNOT GO SINGULAR AT ALL.** `∂Tt4/∂w > 0` is `_topping_fuel`'s own stated
premise ("Tt4 rises monotonically with fuel at fixed spool speeds") and no lever in this family
pins `Tt4` at a fixed fuel — rung 47's governor moves the fuel, not `Tt4`. So the governor is
predicted to be the one leg with **no** route to `G_w = 0`.

---

## 2. SCORED PREDICTIONS

**P1 — THE INSTRUMENT REPRODUCES RUNG 76, ON RUNG 76's OWN SAMPLE.** Read at the sample
`solve_gain` walks, `1/G_a′` matches `solve_gain`'s `gain` to `< 3e−9` (the inherited
differencing floor `eps/dg ≈ 2.2e−9`, NOT `1e−9` — rung 76's P2 was scored optimistic by exactly
that), and `1 − G_a′` matches its `c` to the same. *If this fails, the instrument is wrong and
the rung is not.*

**P2 — D1 HOLDS, PER LEG, AS A MEASUREMENT.** `dw*/dq` differenced directly in `q` equals
`−G_q/G_w` from separately-differenced parts, for all three legs, to `< 3e−9` relative.

**P3 — THE LEDGER IS ORDERED, AND THE ORDER IS φ ≫ gov > accel.** In the legal currency
`‖dw*/dq‖` (kg/s per unit valve), the φ leg's is the largest and the accel leg's the smallest, at
every riding point. *Predicted from § 0.2's normalised table, which is why P3 is weak evidence
and P4 is the real test.*

**P4 — THE ORDER SURVIVES THE ARMS.** P3's ordering holds on **both** stator arms
(`StatorLimiter` / `StatorIncidenceLimiter`), at three `margin` values (`0.05 / 0.10 / 0.40`),
two `Tt4_max` (`1180 / 1200 K`) and two `φ_lim` (`0.76 / 0.80`) — at least 12 combinations, with
no combination inverting it.

**P5 — THE THREE SLOPES ARE MEASURABLY DIFFERENT, WHICH IS THE NON-VACUITY GATE.** The three
normalised slopes are separated by more than `1e−2` at every point — so the reader is not
reporting one quantity three times. *This is the knob-less reduce's substitute for rung 76 § 6's
"the same machine under `sensed` must differ".*

**P6 — `G_s′` CLOSED IS ZERO TO MACHINE PRECISION AND `G_s′` OPEN IS NOT.** `‖G_s′ closed‖ <
1e−7` and `G_s′ open > 1.0` at every riding point, a separation of `> 7` orders. Confirmation of
rung 64's derivation, and § 8 scores it as a confirmation.

**P7 — THE GOVERNOR'S SLOPE IS BOUNDED AWAY FROM ZERO IN EVERY CELL** (D5): normalised
`G_g′ > 0.5` over every combination of P4. *Refutable: a cell where the governor's normalised
slope collapses would kill D5.*

**P8 — `c < 1` IS NOT APPROACHED.** Over the P4 sweep, `c` stays below `0.35`, so the accel leg's
`1/(1−c)` stays below `1.6`. *This is what makes rung 76 § 8's fourth seam (`c → 1`) unreachable
by any setting this family already has, and it is scored so the claim is measured rather than
assumed.*

**P9 — THE REDUCE IS EXACT AND KNOB-LESS.** `StiffnessLedgerTransient` adds no state, no knob and
no plant code, so **every** march it runs is `SensedCapTransient`'s bit-for-bit, on all five of
rung 76's live cells plus the accel-armed φ arm. *Not a tolerance: the parent's methods are not
overridden.*

**P10 — NOTHING IN THE PARENT MOVES.** No production line outside the new class is edited, so
unlike rung 76 § 6.1 there is **no** parent edit for the reduce spine to be blind to, and no
worktree check is needed. *Scored by inspection of the diff, and it is the reason a knob-less
rung's reduce is defensible.*

---

## 3. WHAT WOULD MAKE THIS NOT A RUNG

If P1 fails, the reader is broken. If P5 fails, the ledger is one quantity in three costumes and
this is a note, not a rung. If **D3 is measured false** — if some reading of `_topping_fuel` or
`_surge_fuel` does produce a `1 − c` form — then rung 76 § 8 was right, the headline inverts, and
the rung ships as a CONFIRMATION of the seam instead. All three outcomes are shippable; the one
that is not is a reader that agrees with rung 76 § 3 **because it is rung 76 § 3** (P1 measured on
a path that calls `solve_gain` internally is refused, and P1's reader is written independently).
