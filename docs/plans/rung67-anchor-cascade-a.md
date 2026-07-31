# Rung 67 anchor — CASCADE A: TWO LOOPS ON TWO VARIABLES

**Status: § 0 is MEASURED (a feasibility pre-check, run before the predictions were written).
§§ 1–4 are DERIVATION ONLY.** Every number in §§ 2–4 is a prediction, not a measurement.
Predictions are scored HIT / MISS after the sweep, and a MISS is published in place, as in
rungs 51/58/63/64/65/66.

Rung 66's named next seam: **rung 47's lagged `Tt4` topping governor beside rung 65's lagged
φ-referenced bleed valve.** Four states, two clocks — and, unlike cascade B, two *different*
protected variables.

---

## 0. THE PRE-CHECK — measured FIRST, because it decides whether this is a rung at all

Rung 65's precedent: run the one measurement that can kill the rung before writing anything
that depends on it, and publish it as a pre-check rather than as a finding.

### 0.1 Do the two windows OVERLAP?

Both cross-gains exist only where both laws are live at once. Rung 50's own assert calls the
rung-46/47 governor's window *"post-ramp by construction"*, and the valve rides EARLY ramp
(rungs 41/44). Disjoint windows ⇒ `R ≡ 0` wherever the valve rides ⇒ `R_q ≡ 0` ⇒
`det J = 1/(τ_g τ_v)` trivially ⇒ **no cascade, and this is `docs/cascade-a-negative.md`.**

Measured on shipped code (a rung-65 lagged valve with the INSTANTANEOUS `Tt4_max` governor,
which rung 65's integrator already accepts), `required` reconstructed per point at the live
valve position; `ds = 0.005`, `r = 0.5`, `s_settle = 1.2`, `b_max = 0.10`, `τ_v = 0.05`:

| `φ_lim` | `Tt4_max` | governor window | valve window | **overlap** | riding pts |
|---|---|---|---|---|---|
| 0.80 | 1150 | [0.080, 1.700] | [0.000, 0.980] | [0.080, 0.980] | 181 |
| 0.80 | **1200** | [0.110, 1.700] | [0.000, 0.800] | **[0.110, 0.800]** | **139** |
| 0.80 | 1250 | [0.135, 1.700] | [0.000, 0.710] | [0.135, 0.710] | 116 |
| 0.80 | 1300 | [0.165, 1.700] | [0.000, 0.655] | [0.165, 0.655] | 99 |
| 0.80 | 1350 | [0.195, 1.700] | [0.000, 0.620] | [0.195, 0.620] | 86 |
| 0.82 | 1150 | [0.080, 1.700] | [0.000, 1.700] | [0.080, 1.700] | 325 |
| 0.84 | 1200 | [0.105, 1.700] | [0.000, 1.700] | [0.315, 1.700] | 278 |

**PASS at all fifteen corners** (86–325 riding points; the table is the `φ_lim = 0.80` column
plus two of the nine others). **And it CORRECTS the received framing.** The governor's window
is post-ramp only at rung 46/47's own redline. The scheduled fuel drives the *instantaneous*
`Tt4` to ~1900 K during the accel — that is rung 35's TIT overshoot, the whole reason the
governor exists — so any redline below that engages EARLY, at `s ≈ 0.08…0.20`.

**THE ANCHOR: `φ_lim = 0.80`, `Tt4_max = 1200 K`, `b_max = 0.10`.** `φ_lim` is rung 66's own
value, so cascade B can be re-run at cascade A's settings for a same-settings comparison (rung
63's lesson). `Tt4_max = 1200` puts the overlap in the EARLY ramp, where the binding LP minimum
lives, and keeps the valve strictly interior (`b ≤ 0.068`, never at its stop) — which is what
"riding" requires. Both are IMPOSED, exactly as at rung 64; moving `Tt4_max` off rung 47's
value is a disclosed anchoring choice, not a new constant.

### 0.2 Do the CROSS-GAINS exist, and what are their signs?

The `_b_state` trap (rung 62's `_powers` trap, third reload): `R_q ≠ 0` only because the
governor senses `Tt4` on the machine *as the valve actually is*. Forget `_b_state = q` around
the governor's `required` and `R_q ≡ 0` identically — the rung silently reduces to two
independent loops, with nothing failing.

Derived before measuring:

* **`R_q > 0`.** More bleed ⇒ less core flow ⇒ hotter at fixed fuel ⇒ the governor must clip
  MORE. (Rung 66's seam § 9 states this sign.)
* **`C_g < 0`.** More clip ⇒ less applied fuel ⇒ higher `φ_lp` (rung 49: the floor works by
  cutting fuel) ⇒ the valve needs LESS bleed.
* ⇒ **`P = R_q·C_g < 0`** — the OPPOSITE of cascade B's `≡ +1`.

Measured at riding points, central differences on the shipped closures (`_topping_fuel` for the
fuel law, `_solve_b` for the valve's), `Tt4_max ∈ {1150, 1200, 1300}`:

| | range over the riding window |
|---|---|
| `R_q` | **+1.13e−3 … +1.77e−3** |
| `C_g` | **−12.6 … −15.9** |
| `P = R_q·C_g` | **−1.79e−2 … −2.28e−2** |

Both derived signs hold. **`P` is dimensionless** (`R` is a fuel clip per bleed FRACTION, `C` a
fraction per fuel), so `|P| ≈ 0.02` is directly comparable to cascade B's `1`: **two loops on
two variables are ~50× more weakly coupled than two on one variable, and the sign flips.**

**ONE PROBE ERROR IS RECORDED HERE BECAUSE IT IS THE IMPLEMENTATION'S TRAP.** The first pass
evaluated `C` at `g = 0` and read `C_g` **exactly 0** at most points — which looks like proof
of decoupling and would have killed the rung. It is a SATURATION artifact: at the unclipped
scheduled fuel (`Tt4 ~ 1900 K`) the valve command sits hard on `b_max`, so both sides of the
difference return the stop. **The base point is the APPLIED fuel `mf_sched − g`,** which is
what the plant reads. Any instrument here that reports `C_g = 0` is at a stop, not decoupled.

---

## 1. The plant — four states, two clocks, two SET POINTS

Spools `ν_lp, ν_hp` (rung 40), the governor's fuel-clip amount `g` (rung 47), the valve
position `q` (rung 65):

    dg/ds = ( R(ν, q) − g ) / τ_g     R = max(0, mf_sched − topping(ν, Tt4_max))   [rung 47]
    dq/ds = ( C(ν, g) − q ) / τ_v     C = b_cmd, smallest b holding φ_lp ≥ φ_lim   [rung 65]

`R` is evaluated with `_b_state` set (§ 0.2); `C` is read at the applied fuel `mf_sched − g`
(rung 65's own choice, verbatim). Cross-coupled through the plant, neither law mentioning the
other — cascade B's construction exactly, with the fuel leg's *sensor* moved from `φ_lp` to
`Tt4`. That single substitution is the whole experiment.

**`Tt4_max` PLACEMENT — the ambiguity rung 66 dodged now RUNS.** Rung 66's own docstring
records it: rung 52's placement min-selects the redline UNLAGGED on top of the already-clipped
fuel, rung 65's puts it inside the caps at `mf_sched`, *"the two disagree, nothing would catch
a wrong pick, and cascade B arms `surge` alone, so every rung-66 diagnostic passes
`Tt4_max=None` and the ambiguity never runs."* **This rung takes rung 47's placement** —
`mf = mf_sched − g` with the redline carried BY the state — because the redline is this
cascade's lagged leg, not a min-select cap beside it. **The detector for a wrong pick is a
gate**, not an argument: with the valve disarmed the march must reproduce rung 47's
`_integrate_fuel_lagged` BIT-FOR-BIT.

**THE JOINT INITIAL CONDITION CANNOT INHERIT RUNG 66's SOLVER.** Rung 66 iterates the two laws
to their simultaneous fixed point by Gauss–Seidel, whose contraction factor is `|R_q C_g|`, and
its assert reads *"this is the DEGENERACY LOCUS … a finding, not a solver failure."* On cascade
A that message is **wrong**: `|P|` is unconstrained by any identity, so a diverging iteration
here would mean `|P| > 1` with the equilibrium still unique (`det J ≠ 0`) — a solver failure
published as a degeneracy. At the measured `|P| ≈ 0.02` the iteration contracts hard, but the
message and the fallback must both be rewritten: **converge, and on failure report `|P|`, not
"the marginal mode".**

---

## 2. The linearised block — the SAME scalar, the OTHER sign

    J = [ −1/τ_g    R_q/τ_g ]     tr J  = −(1/τ_g + 1/τ_v)   < 0 always
        [ C_g/τ_v   −1/τ_v  ]     det J = (1 − P)/(τ_g τ_v),  P = R_q C_g

    disc = tr² − 4 det = (1/τ_g − 1/τ_v)² + 4P/(τ_g τ_v)

With `P < 0`, `det J = (1 + |P|)/(τ_gτ_v) > 0` **strictly** — no zero eigenvalue, no marginal
mode, no continuum of equilibria. **The pair is non-degenerate, so it CAN buy authority.**

### The oscillation window, in ONE dimensionless coordinate

Writing `ρ = τ_v/τ_g` (rung 40's coordinate, moved to the actuator side), `disc < 0` ⟺

    ρ + 1/ρ  <  2 + 4|P|

— an interval in `ρ`, **log-symmetric about matched clocks**, whose half-width is set by a
single measured plant scalar and nothing else. Zero new constants. At `|P| = 0.0192`
(`Tt4_max = 1200`, mid-window) the edges are

    ρ ∈ [0.759, 1.318]        (ρ_lo · ρ_hi = 1, exactly)

**This is the mode rung 66 proved impossible at ANY ratio.** There, `P ≡ +1` forces
`disc = tr² ≥ 0`; here `P < 0` admits complex roots. Same algebra, same scalar, opposite sign —
which is why rung 67 is the INVERSE of rung 66 and not a new topic.

### But ADMISSIBILITY IS NOT OBSERVABILITY — and it is the same scalar again

At matched clocks (`ρ = 1`, the window's centre, where the mode is most available):

    λ = −1/τ ± i √|P| / τ        ζ = |Re λ| / |λ| = 1/√(1 + |P|)        T = 2πτ / √|P|

`|P| = 0.0192` ⇒ **`ζ = 0.9906`** and **`T = 45.3 τ`**. The mode decays by `e^(−45)` ≈ 3e−20
over one period. At the anchor's `τ_v = 0.05` that period is `T ≈ 2.27` in `s` — **longer than
the entire march** (`r + s_settle = 1.70`), so it cannot complete a cycle even undamped.

**The scalar that opens the window is the scalar that damps the mode.** A visibly ringing
actuator pair needs `ζ < 0.7`, i.e. `|P| > 1` — a coupling as strong as cascade B's identity,
but negative. The ladder's levers are nowhere near it.

---

## 3. Reduce contract

* `tau_gov=None` **and** `lag=None` ⇒ **rung 65 bit-for-bit**, by dispatch (the valve alone).
* `tau_gov=None`, `lag` set ⇒ **rung 66 bit-for-bit**, by dispatch — cascade B untouched, all
  three of its own arms with it.
* `bleed_lim=None` (or `tau=None`) with `tau_gov` set ⇒ **rung 47's `_integrate_fuel_lagged`
  bit-for-bit**, by dispatch. **This arm is also the `Tt4_max`-placement detector** (§ 1).
* `τ_g → 0` converges to rung 65 with an instantaneous governor; `τ_v → 0` converges to rung 47
  on a bleed-limited plant. **Neither is bit-for-bit** — a different code path with a fourth
  state, rung 65/66's two-armed disagreement on two axes. Both REPORTED per clock, never
  asserted to zero.

---

## 4. Pre-registered predictions

Scored HIT / MISS after the sweep. Two are designed to fail. § 0 is excluded — it was measured
first, and nothing below re-predicts it.

**P1 — the measured spectrum matches the closed form, INCLUDING the window edges.** Sweeping
`ρ = τ_v/τ_g` across the predicted boundary, the eigenvalues of the block finite-differenced on
the shipped closures go complex INSIDE `[ρ_lo, ρ_hi]` and real outside, with the crossing
located to within the grid's resolution. The window's product `ρ_lo · ρ_hi = 1` to machine
precision. **A MISS means `P` is not the only thing in the discriminant.**

**P2 — the mode is ADMISSIBLE AND UNOBSERVABLE, and the two facts share a cause.** Predicting
`ζ ∈ [0.985, 0.995]` at matched clocks and ZERO completed oscillations in either tracking error
(`b − b_cmd`, `g − required`) anywhere on the grid: sign changes attributable to ringing = 0,
against a predicted period `T ≈ 45τ` versus a riding window of `≈ 0.69` in `s` (`T/window ≈ 3.3`
at `τ_v = 0.05`). **This is the calibration that keeps the rung from over-claiming** — the
seam promised an oscillatory mode, and the honest result is that it exists in the spectrum and
never in the trajectory.

**P3 — the `b0` initial-condition spread COLLAPSES, discharging rung 66 § 8.** Rung 66 concedes
verbatim that its 84 % path-spread cannot be attributed to the zero eigenvalue *"needs a
non-degenerate pair to compare against, and § 2's scope table shows the set-point offset that
would build one leaves no riding points on this anchor."* **Cascade A is that pair.** Rung
65/66's `b0` instrument is re-run VERBATIM. Predicting the withheld-fuel spread falls by
≥ 1 order of magnitude versus rung 66's 84 %, because both eigenvalues are now strictly
negative (`≈ −1/τ` each) and the offset is forgotten in `~3τ ≪ ` the ramp.
**BOTH BRANCHES ARE REGISTERED:** if the spread SURVIVES, rung 66's 84 % was ordinary transient
sensitivity, its § 8 concession INVERTS, and that is the finding instead. Neither outcome may
be narrated as the expected one after the fact.

**P4 — the cross-credit 2×2 has OPPOSITE-SIGN off-diagonals.** Two currencies now:
`I_T = ∫max(0, Tt4 − Tt4_max) ds` and `I_φ = ∫max(0, φ_lim − φ_lp) ds` (rung 66's violation
integral, verbatim). Predicting the valve **DEBITS** the temperature (`R_q > 0`: bleed makes it
hotter — arming the valve alone raises `I_T`) while the governor **CREDITS** the surge margin
(`C_g < 0`: clipping fuel raises `φ_lp` — arming the governor alone lowers `I_φ`). A signed,
asymmetric cross-coupling: one loop helps the other, the other hurts it. **This is the object
cascade B could not have**, having only one currency.

**P5 — near-ADDITIVITY on the diagonal: the pair buys AUTHORITY.** Rung 66 measured 38×
erosion (a whole second limiter bought 1.59 points where it delivered 60.46 alone) and attributed
it to `det J ≡ 0`. With `det J ≠ 0` and `|P| ≈ 0.02`, predicting each loop delivers on its OWN
currency within a few percent of its standalone credit — **erosion ≲ 1.5×, versus rung 66's
38×**. That contrast, taken at the SAME `φ_lim` with cascade B re-run at cascade A's settings,
is the rung's protection-currency headline.

**P6 — DESIGNED TO FAIL: rung 66's `ds·(1/τ_g + 1/τ_v) ≤ 2` floor is still SAFE, but it is no
longer the radius.** On the complex branch `|λ| = √det = √((1+|P|)/(τ_gτ_v))`, which at matched
clocks is `1.01/τ` against the sum's `2/τ` — conservative by ~2×. The sum stops bounding the
radius once `|P| > 3`. Predicting: safe everywhere on this grid, i.e. rung 66's floor is
**BOUNDED, not corrected** — a floor derived from an identity is conservative wherever the
identity does not hold. **A MISS (a `|P| > 3` point, e.g. at a `b_cmd` kink where `C_g` blows
up) is a real finding and a correction to rung 66 § 3.**

**P7 — DESIGNED TO FAIL: the joint IC solve converges at EVERY corner, unlike rung 66's.** Rung
66's iteration converged only because every start it tried opened DORMANT (`required(0) = 0`,
`ic_iters = 1`). Cascade A's contraction is `|P| ≈ 0.02`, so it should converge in 2–3
iterations even where BOTH laws are live at `s = 0` — which § 0.1 shows is reachable
(`Tt4_max = 1150` engages at `s = 0.08`; a hotter `Tt4_lo` engages at 0). Predicting: converges
at every corner INCLUDING a live start, `ic_iters ≤ 4`, residual ≤ 1e−12. **A MISS means the
Gauss–Seidel structure, not the contraction factor, was doing the work at rung 66.**

---

## 5. Concessions (declared before the probe)

* Every one rungs 62/63/64/65/66 list, all inherited.
* **`τ_g` and `τ_v` are swept coordinates on the march's own `s`** — no attempt to anchor a real
  actuator bandwidth or a real limiter loop lag. ORDERING, SIGNS and INVARIANCES are the claims;
  every MAGNITUDE is disclaimed.
* **`Tt4_max = 1200 K` is IMPOSED**, and it is *not* rung 46/47's value. It is chosen in § 0.1
  to put the governor's window over the valve's — a disclosed anchoring choice made for
  measurability, and every number here is conditional on it.
* **`φ_lim` and `b_max` remain IMPOSED** (rung 64's concession, verbatim).
* **Both lags stay SYMMETRIC.** Rung 52's asymmetric fuel leg is NOT used here — cascade A's
  fuel leg is rung 47's governor, which has one constant. Rung 66's asymmetric-valve seam is
  untouched, and so is the asymmetric-governor question.
* The spectrum is evaluated at finitely many trajectory points, so it is a DIAGNOSTIC that can
  miss a brief excursion — a guard against rung 65's retracted trap, not a proof of convergence.
  Grid convergence is checked independently.
* **`P` is measured on a two-spool CPG plant with imposed maps.** Whether `|P| ≪ 1` is a
  property of *this* plant or of fuel-vs-airflow levers generally is NOT established here; the
  claim is about the ALGEBRA (one scalar sets both window and damping), with `|P| ≈ 0.02` as
  this plant's value of it.
