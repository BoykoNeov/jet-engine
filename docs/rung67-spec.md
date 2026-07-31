# Rung 67 — CASCADE A: TWO LOOPS ON TWO VARIABLES

**Rung 47's lagged `Tt4` topping governor beside rung 65's lagged φ-referenced bleed valve.**
Four states, two clocks — and, unlike cascade B, two *different* protected variables. Rung 66's
named next seam.

> **HEADLINE — ONE SCALAR DECIDES BOTH FACES, AND ADMISSIBILITY IS NOT OBSERVABILITY.** The
> cross-gain product `P = R_q·C_g` is the whole content of a two-loop actuator block. Two loops
> on ONE variable have `P ≡ +1` by identity (rung 66) ⇒ degenerate, no oscillation at any clock
> ratio. Two loops on TWO variables have `P < 0` ⇒ **non-degenerate, so the pair buys
> AUTHORITY**, and the oscillatory mode rung 66 forbids becomes admissible inside a window
> `ρ + 1/ρ < 2 + 4|P|` in the clock ratio `ρ = τ_v/τ_g` — log-symmetric about matched clocks,
> zero new constants. **But the same scalar damps it:** `ζ = 1/√(1+|P|)` and `T = 2πτ/√|P|`,
> both free of any time constant. Measured `|P| = 2.04e−2` ⇒ `ζ = 0.990`, `T = 44 τ`: the mode
> decays by `e^−44` in one period, **at every clock pair**, so no bandwidth choice can make it
> visible. The mode is real, measured in the spectrum, and unobservable in every trajectory.

**It EXTENDS rung 64 and INVERTS rung 66's ledger.** Rung 66: a second limiter on the same
variable buys bandwidth, not authority (38× credit erosion). Here each loop keeps essentially
all of its standalone credit on its own currency (erosion **0.93× / 1.26×**). **What a second
limiter buys is decided by whether it watches a DIFFERENT VARIABLE** — not by its law (rung
64), not by its actuator, not by its clock.

---

## 0. WHICH cascade, and the PRE-CHECK that had to run first

| | fuel leg | watches | tests |
|---|---|---|---|
| B — rung 66 | rung 52's `AsymmetricLag` over rung 49's `surge` floor | `φ_lp` | can a second clock reach rung 65's marginal mode? |
| **A — TAKEN** | rung 47's `tau_gov` topping governor | `Tt4` | what changes when the two loops are not redundant? |

**The pre-check that could have killed the rung** (`docs/plans/rung67-anchor-cascade-a.md` § 0,
run and recorded before the predictions were written — rung 65's precedent). Both cross-gains
exist only where both laws ride at once. Rung 50's own assert calls the rung-46/47 governor's
window *"post-ramp by construction"*, and the valve rides early ramp (rungs 41/44). Disjoint
windows ⇒ `R ≡ 0` where the valve rides ⇒ `R_q ≡ 0` ⇒ no cascade, and a NEGATIVE doc instead.

**PASS at all fifteen `(φ_lim, Tt4_max)` corners** (86–325 riding points). **And it corrects the
received framing:** the governor's window is post-ramp only at rung 46/47's own redline. The
scheduled fuel drives *instantaneous* `Tt4` to ~1900 K during the accel — rung 35's TIT
overshoot, the reason the governor exists — so any redline below that engages at `s ≈ 0.08…0.20`,
over the valve's own window.

    THE ANCHOR   φ_lim = 0.80 (rung 66's own, so cascade B re-runs at identical settings)
                 Tt4_max = 1200 K   b_max = 0.10   τ_v = 0.05   ds = 0.0025

`Tt4_max = 1200` puts the overlap in the EARLY ramp, where the binding LP minimum lives, and
keeps the valve strictly interior (`b ≤ 0.068`, never at a stop) — which is what riding
requires.

---

## 1. The plant — four states, two clocks, two SET POINTS

    dg/ds = ( R(ν, q) − g ) / τ_g    R = max(0, mf_sched − topping(ν, Tt4_max))   [rung 47]
    dq/ds = ( C(ν, g) − q ) / τ_v    C = b_cmd, smallest b holding φ_lp ≥ φ_lim   [rung 65]

Rung 66's construction with **one substitution** — the fuel leg's *sensor* moves from `φ_lp` to
`Tt4` — and that single change inverts the algebra. `R` runs with `_b_state` set (the governor
senses the machine as the valve actually is); `C` is read at the applied fuel `mf_sched − g`.
Both are rung 66's choices verbatim.

**`Tt4_max` PLACEMENT — the ambiguity rung 66 recorded and dodged now RUNS.** Rung 66's own
docstring: rung 52 min-selects the redline UNLAGGED on top of the already-clipped fuel, rung 65
puts it inside the caps at `mf_sched`, *"the two disagree, nothing would catch a wrong pick"*,
and cascade B never armed it. Here the redline **is** the lagged leg, so it is carried BY the
state exactly as rung 47 carries it. **The detector is a gate, not an argument:** with the valve
disarmed the march reproduces `_integrate_fuel_lagged` bit-for-bit (§ 6, gate 3).

**The `_b_state` boundary is load-bearing in a way it was not on B.** `R_q ≠ 0` *only* because
the governor senses `Tt4` on the machine as the valve actually is. Forget `_b_state = q` around
`required` and `R_q ≡ 0` identically — the rung silently becomes two INDEPENDENT loops,
`det J = 1/(τ_gτ_v)`, no complex branch anywhere, and nothing fails. `cross_identity` reports
`R_q_min_abs = 1.245e−3` as a **gate** for exactly this reason.

**The joint IC cannot inherit rung 66's solver message.** Both starts are coupled, so `(g, q)`
is solved as the two laws' simultaneous fixed point. The iteration contracts at `|P|` — and that
is where the cascades part. On B the identity pins `|P| = 1` wherever both laws ride, so rung 66
can honestly report a stall as THE DEGENERACY. Here `|P|` is pinned by nothing: a stall would
mean `|P| ≥ 1` with the equilibrium still **unique** (`det J ≠ 0`) — a solver failure published
as a degeneracy. The fallback is a damped sweep and the message says so.

---

## 2. THE SCALAR — `P < 0`, and everything follows

    J = [ −1/τ_g    R_q/τ_g ]   tr J  = −(1/τ_g + 1/τ_v)
        [ C_g/τ_v   −1/τ_v  ]   det J = (1 − P)/(τ_gτ_v)
                                disc  = (1/τ_g − 1/τ_v)² + 4P/(τ_gτ_v)

**The signs are derivable before any march, and they DISAGREE:**

* `R_q > 0` — more bleed ⇒ less core flow ⇒ hotter at fixed fuel ⇒ the governor must clip MORE.
* `C_g < 0` — more clip ⇒ less applied fuel ⇒ higher `φ_lp` ⇒ the valve needs LESS bleed.

⇒ `P < 0`, ⇒ `det J = (1 + |P|)/(τ_gτ_v) > 0` **strictly**. The degeneracy is gone.

### Measured on the shipped closures, at riding points of a real march

`R_q` and `C_g` are central-differenced on `_topping_fuel` and `_solve_b`; neither closure knows
the other exists.

| `τ_g` | `ρ_clock` | riding | `P` range | `R_q` | `C_g` | complex |
|---|---|---|---|---|---|---|
| 0.005 | 10 | 270 | [−2.044e−2, −1.910e−2] | +1.25e−3 … +1.45e−3 | −15.3 … −13.7 | **0/13** |
| 0.05 | **1** | 195 | [−2.081e−2, −1.910e−2] | +1.25e−3 … +1.54e−3 | −15.3 … −13.4 | **13/13** |
| 0.5 | 0.1 | 182 | [−2.149e−2, −1.910e−2] | +1.25e−3 … +1.97e−3 | −15.3 … −10.9 | **0/13** |

Both derived signs hold at every sampled point. The individual gains move by 1.2–1.6× across a
march while `P` stays inside a 12 % band — so a near-constant `P` is not a constant plant.

**`P` is dimensionless** (`R` is a fuel clip per bleed FRACTION, `C` a fraction per fuel), so
`|P| ≈ 0.02` is directly comparable to cascade B's `≡ 1`: **two loops on two variables are ~50×
more weakly coupled than two on one variable, and the sign flips.**

### The oscillation window, in one dimensionless coordinate

    disc < 0   ⟺   ρ + 1/ρ  <  2 + 4|P|,        ρ = τ_v/τ_g

an interval **log-symmetric about matched clocks**, its half-width set by one measured plant
scalar and nothing else. At `P = −2.038e−2`:

    ρ ∈ [0.7523, 1.3292]        ρ_lo·ρ_hi − 1 = 0   EXACTLY

**Measured against it:** complex at 13/13 sampled points at `ρ = 1` (inside), real at 0/13 at
both `ρ = 10` and `ρ = 0.1` (outside). This is the mode rung 66 proved impossible at ANY ratio —
recovered as the `P → +1` limit of the same formula rather than asserted separately.

### ADMISSIBILITY IS NOT OBSERVABILITY — and it is the same scalar again

At matched clocks `λ = −1/τ ± i√|P|/τ`, so

    ζ = 1/√(1 + |P|) = 0.98995            T = 2πτ/√|P| = 44.0 τ

**Neither contains a time constant.** The mode decays by `e^−44 ≈ 8e−20` over one period at
*every* clock pair, so no bandwidth choice can make it visible — and at `τ_v = 0.05` the period
is `2.2` in `s`, longer than the whole march (1.70) besides.

**Measured on the plant's own FREE response**, which is the honest test: two marches, natural
and with `b0` offset by 0.005, differenced — the forcing is common and cancels, so what remains
is the homogeneous solution. Across `ρ ∈ {0.25, 0.5, 0.8, 1.0, 1.25, 2.0, 4.0}`:

**at most ONE sign change anywhere, and one is admissible for a REAL pair** — a sum of two
decaying real exponentials has at most one zero, so only a *second* crossing requires a complex
pair. `rings_anywhere = False`.

**A null result is worth nothing until the instrument is shown to fire**, so
`detector_sensitivity` runs the same RK4 and the same counter on the linear block itself:

| `|P|` | ζ | `T/τ` | decay/period | sign changes |
|---|---|---|---|---|
| 0.02 | 0.990 | 44.4 | 5.1e−20 | **0** |
| 0.5 | 0.817 | 8.9 | 1.4e−04 | **3** |
| 3.0 | 0.500 | 3.6 | 2.7e−02 | **7** |
| 10.0 | 0.302 | 2.0 | 1.4e−01 | **13** |

The detector reads 0 at this plant's `|P|` because the mode is dead, **not because it is blind.**
A visibly ringing actuator pair needs `ζ < 0.7`, i.e. `|P| > 1` — a coupling as strong as cascade
B's identity but negative. No lever in this ladder is near it.

---

## 3. THE STABILITY FLOOR — rung 66's is BOUNDED, not corrected

Rung 66 derived `ds·(1/τ_g + 1/τ_v) ≤ 2` from its own identity: `det J ≡ 0` makes the non-zero
eigenvalue exactly `−(1/τ_g + 1/τ_v)`, so **the rates add**. Here `det J ≠ 0` and on the complex
branch the radius is `√det = √((1+|P|)/(τ_gτ_v))`, which at matched clocks is `1.01/τ` against
the sum's `2/τ`.

    measured:  sum/radius = 1.98x at matched clocks,  1.10x at ρ = 10 and ρ = 0.1
               sum_always_safe = True

**A floor derived from an identity is conservative wherever the identity does not hold.** The
sum is kept as the a-priori assert — it is what can be computed *before* a march — and the
measured radius is reported beside it. The sum stops bounding the radius only once `|P| > 3`,
which is 150× this plant's value.

---

## 4. WHAT THE PAIR DELIVERS — the 2×2 cascade B could not build

Two currencies now, both areas (rung 66's argument for an integral over an extremum, verbatim):

    I_T = ∫₀^r max(0, Tt4 − Tt4_max) ds        the governor's
    I_φ = ∫₀^r max(0, φ_lim − φ_lp) ds         the valve's (rung 66's, verbatim)

Both loops lagged in every cell (rung 66's discipline: a lagged loop against an *instantaneous*
one is not a control but a different plant).

| case | `I_T` | credit_T | `I_φ` | credit_φ | max `Tt4` | min `φ_lp` |
|---|---|---|---|---|---|---|
| bare | 1.09950e+2 | — | 2.58877e−2 | — | 1695.4 | 0.73544 |
| **gov** only | 2.74680e+1 | **+75.02 %** | 2.04852e−2 | **+20.87 %** | 1279.2 | 0.74299 |
| **valve** only | 1.17010e+2 | **−6.42 %** | 1.93992e−3 | **+92.51 %** | 1717.5 | 0.78912 |
| both | 2.85653e+1 | +74.02 % | 1.40892e−3 | +94.56 % | 1281.5 | 0.78912 |

### The OFF-DIAGONAL has OPPOSITE SIGNS — the object with no cascade-B analogue

* **The valve DEBITS the temperature: −6.42 %**, and it shows in the trajectory too — arming the
  valve alone raises peak `Tt4` by **+22.1 K** (1695.4 → 1717.5). That is `R_q > 0` in the
  protection currency: bleed makes it hotter.
* **The governor CREDITS the surge margin: +20.87 %.** That is `C_g < 0`: clipping fuel raises
  `φ_lp`.

One loop helps the other; the other hurts it. Both signs were derivable from the two gains
before any march.

### The DIAGONAL: each loop keeps its own currency — the pair buys AUTHORITY

| loop | own currency | standalone | marginal (on top of the other) | erosion |
|---|---|---|---|---|
| governor | `I_T` | 75.02 % | **80.44 %** | **0.93×** |
| valve | `I_φ` | 92.51 % | **73.69 %** | **1.26×** |
| *rung 66, same valve, same `φ_lim`* | *`I_φ`* | *60.46 %* | *1.59 %* | ***38.1×*** |

The governor's erosion is **below 1** — super-additive, and the mechanism is the off-diagonal:
the valve raises `I_T`, so the governor added on top of it removes *more* violation against the
same bare baseline than it does alone.

### The sharpest form of it, and it is a same-settings comparison

Cascade B was **re-run at cascade A's settings** (rung 63's lesson), and the two share their
`bare` and `valve` cells to six figures. Adding a SECOND loop to the same lagged valve:

| second loop | watches | delivers ALONE on `I_φ` | buys at the MARGIN on `I_φ` |
|---|---|---|---|
| cascade B — rung 49's φ floor | `φ_lp` | **60.46 %** | **+1.59 pts** |
| cascade A — rung 47's governor | `Tt4` | **20.87 %** | **+2.05 pts** |

**A loop that does not even watch `φ_lp`, and delivers 3× less on it standalone, buys 29 % MORE
`φ` protection at the margin than a loop that watches it.** That is `det J ≡ 0` priced in the
protected variable's own currency.

---

## 5. RUNG 66 § 8's CONCESSION — discharged for one number, INVERTED for the other

Rung 66 § 8, verbatim:

> The 84 % `b0` sensitivity of § 5 is reported as a MEASUREMENT and **not attributed** to the
> zero eigenvalue. Separating it from ordinary transient sensitivity needs a **non-degenerate
> pair to compare against**, and § 2's scope table shows the set-point offset that would build
> one leaves no riding points on this anchor.

**Cascade A is that pair.** Rung 65/66's `b0` instrument is re-run verbatim — same offset
(±0.01), same grid, same `φ_lim` — and rung 66's own numbers were re-measured here rather than
quoted:

| across ±0.01 in `b0` | rung 66 (degenerate) | rung 67 (non-degenerate) | verdict |
|---|---|---|---|
| spread in the WITHHELD FUEL | **83.96 %** | **0.014 %** | **collapses ~6100×** |
| spread in the VIOLATION INTEGRAL | **40.75 %** | **45.50 %** | **survives, slightly larger** |

**Both branches were pre-registered** (anchor P3), and the answer took one each.

* **The withheld-fuel spread IS the zero eigenvalue.** On B the degeneracy is precisely a trade
  between clipping fuel and bleeding that leaves `φ_lp` at the set point, so the *split* between
  the two is undetermined and the withheld fuel inherits the `b0` offset one-for-one. Remove the
  degeneracy and it collapses to nothing. **That half of the concession is discharged.**
* **The violation-integral spread is NOT.** It survives on a pair with no marginal direction, so
  it was ordinary transient sensitivity all along — the offset physically moves the bleed during
  the early window where the integral is accumulated. **That half INVERTS.**

And the residual difference points the other way from rung 66's ledger: A's 45.50 % against B's
40.75 % says the redundant pair is **12 % less sensitive** to its own initial condition on the
shared variable. **That is the one thing redundancy buys that bandwidth does not** — and rung 66,
having only the degenerate case, could not see it.

**The 12 % is a finding and not noise, and that had to be measured rather than asserted.** Both
spreads are grid-converged *and so is their ratio* — the quantity the claim actually rests on:

| `ds` | A `dI_φ` | B `dI_φ` | **ratio** |
|---|---|---|---|
| 0.01 | 45.413 % | 40.613 % | 1.1182 |
| 0.005 | 45.447 % | 40.696 % | 1.1167 |
| 0.0025 | 45.495 % | 40.748 % | **1.1165** |

The ratio moves 0.15 % across a 4× range against a 12 % effect. `docs/pt3-sensor-lag-negative.md`
is the standing counter-example — a 12 % gap that sat *inside* its own `ds` band and was
therefore not a finding — so the test gates the RATIO with a margin, after a first version of
that gate was written as a tautology (§ 6, gate 9).

**The mechanism has a supporting number.** The proposed reason for A's larger spread is that in
A the valve is the *only* φ-protecting loop, so moving `b0` moves φ unopposed, whereas in B the
fuel leg defends the same variable and partly compensates. The ledger says exactly that: B's
fuel leg credits `I_φ` at **60.46 %** against A's governor at **20.87 %** — three times the
φ-authority available to absorb the offset. It is consistent, not proved (§ 8).

---

## 6. Reduce contract

* `tau_gov=None`, `lag=None` ⇒ **rung 65 bit-for-bit**, by dispatch (gate 1, 341 pts).
* `tau_gov=None`, `lag` set ⇒ **rung 66 bit-for-bit**, by dispatch — cascade B untouched, and
  all three of ITS arms with it (gate 2).
* `bleed_lim=None` (or its `tau=None`) with `tau_gov` set ⇒ **rung 47's
  `_integrate_fuel_lagged` bit-for-bit**, by dispatch. **That arm is also the `Tt4_max`
  placement detector** of § 1 (gate 3).
* `τ_g → 0` converges to rung 65 with an instantaneous governor; `τ_v → 0` converges to rung 47
  on a bleed-limited plant. **Neither is bit-for-bit** — a different code path with a fourth
  state, rung 65/66's two-armed disagreement on two axes. REPORTED, never asserted to zero.

**Grid convergence** across an 8–16× `ds` range: `P` 0.04 %, `I_φ` 0.23 %, `I_T` **0.005 %**,
both `b0` spreads 0.2 %.

**Gate 9 was written as a TAUTOLOGY first, and the correction is recorded because the claim it
guards is the one that leaves this file.** The inversion half of § 5 rests on a 12 % gap, and
the first form of its gate (`a > 0.9 × b`, sitting beside an existing `a > 0.3`) set a threshold
of 0.366 against a measured 0.455 — it would have passed on a spread that had *shrunk* by 10 %,
i.e. on the opposite finding. The gate now watches the RATIO with a margin, and the grid table
in § 5 is what makes the ratio quotable.

**The damped IC fallback is EXERCISED, not merely shipped.** On this plant `|P| ≈ 0.02` and the
undamped sweep converges in 1–2 iterations at every corner (`ever_damped = False`), so the
damped retries are code that never runs here — untested guard code, which is a liability rather
than a safeguard. `_joint_fixed_point` is therefore extracted from the march and driven directly
with synthetic laws of chosen `P`: the composite multiplier is `(1−w) + wP`, so `w = 1` handles
`|P| < 1`, `w = ½` up to 3, `w = ¼` up to 7 — **and the 60-iteration cap participates in the
choice**, so `P = −0.9` (which contracts, at 0.9 per iteration) is damped exactly like a
divergent one.

**One instrument was repaired, and the defect is disclosed rather than hidden.** `_exceed` does
not copy rung 66's `_violation` upper limit. `_violation` breaks on `traj[i]["s"] > s_hi`, which
DROPS the whole final cell whenever the marched `s` lands a float's width past `r`. On rung 66's
currency that is immaterial — the φ violation is an early-ramp object, its integrand ~0 by
`s = r`. On the temperature currency the integrand is at its MAXIMUM there, so a dropped cell is
worth ~`ds·490`: the raw `I_T` drifted **2.8 % monotonically over 8× `ds` with the increments
refusing to halve** — a grid artefact that reads exactly like slow convergence. `_exceed`
interpolates the straddling cell instead, and `I_T` converges to 0.005 %. The credit RATIO was
stable either way (both cells lose the same sliver), which is why **no published number changes**;
the raw integral becomes quotable, which is why it was fixed. **Rung 66's `_violation` is
deliberately NOT touched** — its numbers are gated.

---

## 7. Predictions, scored

The anchor pre-registered seven, two designed to fail.

| | prediction | verdict |
|---|---|---|
| **P1** | measured spectrum matches the closed form, including the window edges | **HIT.** Complex 13/13 inside (`ρ = 1`), real 0/13 outside at both `ρ = 10` and `ρ = 0.1`; `ρ_lo·ρ_hi − 1 = 0` exactly. |
| **P2** | the mode is ADMISSIBLE AND UNOBSERVABLE, both from the same scalar | **HIT.** `ζ = 0.98995` (predicted 0.985–0.995), `T = 44.0 τ`, and at most ONE sign change in the free response anywhere — admissible for a real pair, so zero evidence of ringing. The detector fires at 3/7/13 crossings for `|P| = 0.5/3/10`. |
| **P3** | the `b0` spread COLLAPSES, discharging rung 66 § 8 (both branches registered) | **SPLIT — and it is the richer answer.** The withheld-fuel spread collapses 83.96 % → 0.014 % (discharged); the violation-integral spread survives 40.75 % → 45.50 %, a 12 % gap whose *ratio* is grid-stable to 0.15 % (**inverted**). § 5. |
| **P4** | the cross-credit off-diagonals have OPPOSITE signs | **HIT.** Valve on `I_T`: **−6.42 %** (+22.1 K on peak `Tt4`); governor on `I_φ`: **+20.87 %**. |
| **P5** | near-additivity on the diagonal — erosion ≲ 1.5× against rung 66's 38× | **HIT, and one cell went past it.** 1.26× (valve on `I_φ`) and **0.93×** (governor on `I_T` — *super*-additive, because the valve's debit gives it more to remove). |
| **P6** | *(designed to fail)* rung 66's sum floor is still SAFE but no longer the radius | **HIT, not the failure.** `sum_always_safe = True`, conservative by **1.98×** at matched clocks — the derived 2× — and 1.10× at `ρ = 10` and `ρ = 0.1`. Rung 66's floor is BOUNDED, not corrected. |
| **P7** | *(designed to fail)* the joint IC converges at every corner, including a LIVE start | **HIT, not the failure.** Three of eight corners open with the fuel leg live (`required(0) > 0`) — which rung 66 could exhibit at none — and the solve takes **2 iterations, residual exactly 0, no damping**. |

**Both designed-to-fail predictions HIT, exactly as at rung 66.** That is a fact about the
predictions, not a triumph: two rungs running, the ones written to break have been the ones most
firmly confirmed, which means they were not aggressive enough. The genuinely surprising result
here (§ 5's split) came from a prediction whose *both* branches were registered — that is the
device that worked, and it should be used more.

**One caveat on P7, stated because the number invites over-reading.** The three live corners
open with the fuel leg engaged and the valve at its lower stop (`b0 = 0`), so the coupling at
`s = 0` runs one way there. A start with BOTH laws strictly interior was not found on this grid.

---

## 8. Concessions

* Every one rungs 62/63/64/65/66 list, all inherited.
* `τ_g` and `τ_v` are swept coordinates on the march's own `s` — no real actuator bandwidth or
  limiter loop lag is anchored. ORDERINGS, SIGNS and INVARIANCES are the claims; every MAGNITUDE
  is disclaimed.
* **`Tt4_max = 1200 K` is IMPOSED and is NOT rung 46/47's value.** It is chosen in § 0 so the two
  windows overlap at all; every number here is conditional on it. `φ_lim` and `b_max` remain
  imposed (rung 64, verbatim).
* **Both lags are SYMMETRIC.** Rung 52's asymmetric fuel leg is not used — cascade A's fuel leg
  is rung 47's governor, which has one constant. Rung 66's asymmetric-valve seam is untouched,
  and so is the asymmetric-governor question.
* **`P` is measured on a two-spool CPG plant with imposed maps.** Whether `|P| ≪ 1` is a property
  of *this* plant or of fuel-vs-airflow levers generally is NOT established. The claim is about
  the ALGEBRA — one scalar sets both the window and the damping — with `|P| ≈ 0.02` as this
  plant's value of it. **A plant with `|P| > 1` would ring, and nothing here rules one out.**
* The spectrum is sampled at finitely many trajectory points, so it is a DIAGNOSTIC that can miss
  a brief excursion (rung 65's retracted trap), not a proof of convergence.
* § 5's comparison holds the instrument, the offset, the grid and `φ_lim` fixed, but the two
  cascades' fuel legs are different objects (a φ floor vs a `Tt4` governor). The attribution of
  the withheld-fuel collapse to the zero eigenvalue rests on that difference being the *only*
  relevant one, which is argued mechanically in § 5 and not proved.
* The `_exceed` boundary repair (§ 6) is disclosed but its effect on rung 66's own currency was
  checked only by the argument that the φ integrand vanishes by `s = r`, not by a re-run of
  rung 66 under the repaired rule — that would change gated numbers.

---

## 9. Next seams

* **THREE loops on one variable — CLOSED BY RUNG 68** (`docs/rung68-spec.md`). The prediction
  below holds exactly: `J = −D·c·rᵀ` is RANK ONE at every `n`, so an `n`-loop block on one
  manifold carries `n − 1` zeros and one root at `−Σ 1/τ_i`. What it does NOT carry is rung
  66's magnitude guess (the third limiter buys 2.45 %, MORE than the second's 1.59 %), and the
  `n ≥ 3` content turns out to be the CYCLIC product rather than the pairwise identity restated
  — a block can be pairwise-degenerate and still rank 2.
* **THREE loops on TWO variables** — this rung's own: the governor and the φ floor *both* beside
  the valve, which § 2's algebra says superposes a `P ≡ +1` block onto a `P < 0` one. Asserted
  against in `integrate_fuel`, not run.
* **A plant with `|P| > 1`.** Everything about the window is derived and only its *width* is
  measured, so the one experiment that would test the derivation where it bites — an actually
  ringing actuator pair — is missing. It needs a lever pair whose cross-gains are strong, not a
  bandwidth choice: `ζ` has no `τ` in it.
* **An ASYMMETRIC governor** (and rung 65's asymmetric valve) — both still open.
* **Fuel + bleed + STATOR on one plant** — rung 63's seam, still untouched by 64/65/66/67.
