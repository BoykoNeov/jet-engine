# Rung 66 — THE TWO-LAG CASCADE

**A lagged bleed VALVE beside a lagged FUEL leg, both watching `φ_lp`.** Four states, two
clocks. Rung 65's named next seam and rung 52's own standing one, reached from the airflow
side.

> **HEADLINE — TWO LOOPS ON ONE VARIABLE ARE ONE LOOP WITH THE RATES ADDED.** Two control
> laws that hold the same variable to the same set point have `∂R/∂q · ∂C/∂g ≡ 1` — an
> IDENTITY, not a coincidence — so the actuator Jacobian has `det J ≡ 0` at every point, at
> every bandwidth, on every plant. Its eigenvalues are exactly `{0, −(1/τ_g + 1/τ_v)}`: the
> zero is rung 65's degeneracy, CONSERVED and now provably unremovable; the other is the two
> clocks, which **ADD**. So a second limiter on the same variable buys **BANDWIDTH, NOT
> AUTHORITY** — which is why its protection credit is strongly sub-additive, and which extends
> rung 64 (*a limiter's LAW cannot buy PROTECTION, only its PRICE*) from a limiter's law to a
> whole second limiter.

**It also CORRECTS rung 65.** Rung 65 found `b` exactly FROZEN and called it the marginal
mode. The mode is not the freeze: a zero eigenvalue means no restoring force *along* a
direction, not that the state sits still on it. Rung 65's instantaneous fuel leg pinned the
state to the manifold `φ_lp = φ_lim`, where the marginal direction has nothing to drive it.
Give the fuel leg a clock too and the state runs off-manifold and DRIFTS along that same
direction. **Same degeneracy, different observable — the freeze was the MANIFOLD, not the
mode.**

---

## 0. WHICH cascade, and why the other one is the seam

| | fuel leg | watches | tests |
|---|---|---|---|
| **B — TAKEN** | rung 52's `AsymmetricLag` over rung 49's `surge` floor | `φ_lp` | rung 65 § 3: can a second clock reach the marginal mode? |
| A — the seam | rung 47's `tau_gov` topping governor | `Tt4` | rung 52 § 3's non-additivity (two clocks, two *different* variables) |

Rung 65 § 3's marginal mode is **two loops on ONE variable**, so only a `φ`-referenced fuel leg
tests it. A is a legitimate rung and not this one; it is asserted against in
`integrate_fuel`, exactly as rung 65 asserted against this one.

**B needed no new control law.** `_integrate_fuel_asym`'s `required` already min-selects over
`accel` **and** `surge`, so rung 52's lag is a lag on the *composite* requirement with rung
49's `φ` leg inside it. The rung is a MERGE of two shipped integrators
(`_integrate_fuel_asym` + `_integrate_fuel_valve_lag` → `_integrate_fuel_cascade`).

---

## 1. The plant — four states, two clocks

Spools `ν_lp, ν_hp` (rung 40), the fuel-clip amount `g` (rung 52), the valve position `q`
(rung 65):

    dg/ds = ( R(ν, q) − g ) / lag.tau(R, g)      R = max(0, mf_sched − min(armed caps))
    dq/ds = ( C(ν, g) − q ) / τ_v                C = b_cmd, the smallest b holding φ_lp ≥ φ_lim

**The coupling is by construction, and it is an assumption with a physical justification, not
a discovery.** `R` is evaluated with `_b_state` set — the fuel leg solves its cap against the
plant *as the valve actually is*, because a real limiter watches the machine it is on.
Symmetrically `C` is read at the APPLIED fuel `mf_sched − g` (rung 65's own choice, verbatim).
So `R` reads `q` and `C` reads `g`: cross-coupled through the plant though neither law mentions
the other.

**The `_b_state` boundary is the rung-62 `_powers` trap reloaded.** Every closure call that
represents *the plant* (`_instant_fuel`, `_surge_fuel`, `_sched_fuel`, `_topping_fuel`) runs
with `_b_state` set; only `command`, which roots rung 64's valve over TRIAL positions, runs
without it. Get it backwards and a solver converges on a residual the plant never uses, with no
test failing. Gate 2 is the detector.

### The joint initial condition

Rung 52 starts `g = 0` (its march opens dormant); rung 65 starts `b = b_cmd(0)`, because
starting at 0 injects a startup transient into the EARLY-ramp LP minimum — the binding one
(rungs 41/44), and where every number here is taken. On a cascade both are true at once **and
they are each other's arguments**, so `(g, q)` is solved as the two laws' simultaneous fixed
point. THE ITERATION IS ITSELF A DIAGNOSTIC: it contracts at `|R_q C_g|`, so it converges
exactly when `det J > 0`, and a failure to converge is the degeneracy announcing itself at
`s = 0`.

On the anchored case the march opens dormant (`R(0) = 0`, `ic_iters = 1`, residual exactly 0),
so the joint solve is not load-bearing *here*. **The obvious escape was measured and it is not
one.** The natural prediction — a hotter `Tt4_lo` opens with the floor already engaged, the
march sits ON the degeneracy at `s = 0`, and the iteration stalls — is **FALSE** at every
corner tested (`Tt4_lo ∈ {1000, 1200, 1300}` K × `φ_lim ∈ {0.80, 0.82, 0.84}`): `required(0)`
is exactly 0 in all six, `ic_iters = 1`, residual 0. What the `b0` column shows is that the
same outcome arrives by **two different mechanisms** — at 1000 K the valve is open and carries
the floor (`b0` = 0.037 / 0.062 / 0.087), at 1200 and 1300 K it is fully SHUT (`b0` = 0) and
the starting running line clears the floor unaided. Either way the fuel leg is dormant at
`s = 0`.

So the degeneracy's signature at `s = 0` is **not** a stalled solve but **non-uniqueness of the
initial condition**: on the manifold the pair is a one-parameter family, and the iteration lands
on the `g = 0` member because it starts there and that point is a fixed point. That is exactly
the quantity § 5's `b0` instrument measures. The joint solve is therefore the *correct* initial
condition rather than a *needed* one — no start on this grid exercises it, and whether any
admissible start does is untested.

---

## 2. THE IDENTITY — `R_q · C_g ≡ 1`

Both laws are implicit functions of the **same** constraint `φ(w, b) = φ_lim`, where `w` is the
applied fuel and `b` the valve position.

**The fuel law.** `_surge_fuel` returns the cap `w(q)` such that `φ(w(q), q) = φ_lim`, and
`R = mf_sched − w(q)`. Differentiating the constraint in `q`:

    φ_w · w′(q) + φ_b = 0        ⇒     w′(q) = −φ_b/φ_w        ⇒     R_q = +φ_b/φ_w

**The valve law.** `b_cmd = C(g)` is the position such that `φ(mf_sched − g, C(g)) = φ_lim`.
Differentiating in `g`:

    φ_w · (−1) + φ_b · C′(g) = 0 ⇒     C_g = φ_w/φ_b

    ⇒     R_q · C_g  =  (φ_b/φ_w)(φ_w/φ_b)  ≡  1

**The two cross-gains are RECIPROCALS by construction.** The result does not depend on the
plant, on the gains, on the actuators, or on the bandwidths. It needs only that (i) both laws
hold the same variable to the same set point, and (ii) both partials are finite and non-zero.

### Measured — and this is what makes it a result rather than an algebraic curiosity

`R_q` and `C_g` are central-differenced on the **shipped closures** at the riding points of a
real march (`required > 0` and `0 < b_cmd < b_max`). Neither closure knows about the other.

| `τ_att` | riding pts | sampled | eigenvalues REAL | `R_q·C_g` range |
|---|---|---|---|---|
| 0.005 | 437 | 13 | 13 / 13 | [0.9947, 1.0140] |
| 0.05  | 228 | 12 | 12 / 12 | [0.9763, 1.0335] |
| 0.5   | 138 | 13 | 13 / 13 | [0.9644, 1.0068] |

The individual gains move by a factor of ~1.7 across a march (`R_q` from −5.7e−2 to −9.9e−2,
`C_g` from −17.5 to −10.6) **while their product stays within 3.5 % of 1** — which is the
identity showing up in finite differences, not a fit. The residual departure is the transient
itself: `R` is evaluated at the live `q` and `C` at the live `g`, and off-manifold those are
not the same point.

### The consequences, in order

    J = [ −1/τ_g    R_q/τ_g ]     tr J = −(1/τ_g + 1/τ_v)
        [ C_g/τ_v   −1/τ_v  ]     det J = (1 − R_q C_g)/(τ_g τ_v)  ≡  0

1. **`det J ≡ 0`.** The eigenvalues are exactly `{0, tr J}`.
2. **They are REAL**, and for a stronger reason than the anchor's sign argument: the
   discriminant is `tr² − 4·0 = tr²`. There is **no oscillatory actuator mode** at any clock
   ratio — rung 40's map-created inter-spool mode does **not** transfer to the actuator side.
3. **The non-zero eigenvalue is `−(1/τ_g + 1/τ_v)`: THE RATES ADD.** Measured against the
   shipped closures — 39.97 vs 40 at `(τ_v, τ_g) = (0.05, 0.05)`, 220.0 vs 220 at
   `(0.05, 0.005)`, 21.99 vs 22 at `(0.05, 0.5)`.
4. **The zero eigenvalue is rung 65's degeneracy, and it is now provably unremovable.** The
   anchor pre-registered `R_q C_g = 1` as a *locus* the clocks could not move. It is not a
   locus — it is an identity. No bandwidth, no gain, no second actuator can leave it.

---

### THE SCOPE — one SET POINT, not merely one variable

The derivation needs both laws evaluated at the **same** operating point. Two loops on the same
variable with *different* set points define two different manifolds, their partials are taken
at different points, and the product leaves 1. Measured by offsetting the valve's `φ_lim` from
the fuel leg's, everything else held:

| valve `φ_lim` | offset | `R_q` | `C_g` | `R_q·C_g` |
|---|---|---|---|---|
| 0.80 | 0 % | −7.427e−2 | −13.387 | **0.99433** |
| 0.81 | +1.25 % | −6.396e−2 | −15.390 | 0.98435 |
| 0.78 | −2.50 % | −4.831e−2 | −19.687 | 0.95104 |

(±2.5 % and beyond leaves no riding points at all — the two laws stop overlapping.)

**So the headline claims ONE SET POINT.** The departure from 1 is a *distance* between the two
laws' evaluation points: a 2.5 % set-point offset buys a 4.9 % departure, and the residual
±3.5 % seen at zero offset during a transient is the same thing from a different cause — `R` is
read at the live `q` and `C` at the live `g`, and off-manifold those are not the same point.
The identity is **exact on the shared manifold** and degrades smoothly away from it.

### The phase lock — a corollary, not a second finding

Each command is by definition the value that would put `φ_lp` at `φ_lim`, so `b − b_cmd` and
`g − required` must both change sign when `φ_lp` crosses the floor. What is *not* definitional
is that this survives arbitrary bandwidth mismatch. Across nine `(τ_v, τ_att)` pairs spanning
a 100× ratio both ways, the three zero-crossings land in **the same grid cell every time**
(spread 0 cells) while the crossing itself moves from index 98 to 209 — a factor of two. The
two tracking errors are rigidly slaved to one scalar; the pair has one error signal.

---

## 3. THE STABILITY FLOOR — the rates add, so the naive bound is WRONG in the unsafe direction

Rung 65 published a **RETRACTION**: `db/ds = (b_cmd − b)/τ` under an explicit RK4 is unstable
above `z = ds/τ ≈ 2.78`, and at `z = 5` it returned an `∫b ds` **4.4× the converged value**,
looking exactly like a physical finding ("a fast valve bleeds more"). Rung 47's
`_integrate_fuel_lagged` still carries no such assert at all.

The naive transfer to a cascade — bound the fastest clock, `ds/min(τ) ≤ 2` — **is wrong by up
to a factor of 2**, because § 2 item 3 says the rates ADD:

    assert  ds · ( 1/τ_v + 1/min(τ_att, τ_rel) )  ≤  2.0

Where the fuel leg is dormant `R_q` vanishes, `det J = 1/(τ_g τ_v) > 0`, and the radius drops
to `max(1/τ_g, 1/τ_v)` — which the sum still bounds. **The sum is the correct a-priori floor in
both regimes.** At matched clocks it reads `ds/τ ≤ 1.0`, exactly half of rung 65's single-state
constant: a sweep that inherited rung 65's floor would run at twice the step this rung admits.

This is the rung's own identity paying for its own guard, and it is a strict improvement on
rung 65's: **rung 65's floor was a scalar because it had one state; a cascade's floor is a
spectral radius, and on a degenerate pair that radius is the SUM.**

---

## 4. WHAT THE PAIR DELIVERS — bandwidth, not authority

### The currency had to be repaired first

`min φ_lp` is **not usable** here, and the reason is a measurement rather than a preference: on
the fuel-leg-alone control the argmin sits at **`s = 0.0025`**, the first point off the running
line the march starts on. That number is the initial condition, not a protected minimum, and a
credit table built on a clamped extremum is not quotable. The same control's march also
**truncates at `s = 1.08`** of a 1.70 span (the plant walks into a corner: `Tt4` 594 K).

The primary currency is therefore the **violation integral over the ramp**,
`I = ∫₀^r max(0, φ_lim − φ_lp) ds` — an area, which cannot be clamped by its own initial
condition and which the late truncation cannot reach either. It is grid-converged to 0.19 %
across an 8× `ds` range (1.52626e−3 → 1.52922e−3).

### The ledger

Each loop **lagged**, alone and together. The controls are both lagged on purpose: a lagged
loop against an *instantaneous* one is not a control but a different plant — rung 65 already
called the instantaneous limit singular, so such a comparison collapses to "the instantaneous
loop holds the set point" and measures nothing about redundancy.

| case | `I` | credit | min φ (s>0) | argmin `s` |
|---|---|---|---|---|
| bare, no loop | 2.58877e−2 | — | 0.735441 | 0.2325 |
| **F** — lagged FUEL only | 1.02368e−2 | 60.46 % | 0.773558 | 0.0025 † |
| **V** — lagged VALVE only | 1.93992e−3 | 92.51 % | 0.789122 | 0.0875 |
| **LL** — BOTH lagged | **1.52910e−3** | **94.09 %** | 0.793085 | 0.0700 |

† clamped by the initial condition — see above.

    sum of the two standalone credits    152.96 %
    delivered by the PAIR                 94.09 %

* **The pair beats both singles** — so "two protections that each work, together fail" is
  false, and any framing resting on an instantaneous control is dead.
* **It is strongly sub-additive**, and the asymmetry is the finding:

| adding | marginal credit | that loop's standalone credit | erosion |
|---|---|---|---|
| the FUEL leg on top of the valve | **1.59 %** | 60.46 % | **38.1×** |
| the VALVE on top of the fuel leg | 33.64 % | 92.51 % | 2.8× |

**A whole second limiter — its own sensor, its own law, its own actuator, its own clock — buys
1.59 points of protection where it delivers 60.46 alone.** That is § 2 in the protection
currency: `det J ≡ 0` means the pair has ONE effective actuator direction, so the second loop
cannot buy authority. What it buys is the rate, and the rates add.

### Where the deficit lives

The pair still misses the floor: `min φ_lp = 0.793085` against `φ_lim = 0.80`, an undershoot of
**−6.9148e−3**, grid-stable to five figures across `ds` ∈ {0.01, 0.005, 0.0025, 0.00125}. It is
a **RAMP** phenomenon, not a startup artifact: over `s > 3·max(τ)` the minimum is
**+3.137e−3 ABOVE** the floor, i.e. the post-ramp regime is over-protection, both actuators
holding their clip after the law has released. Neither single-loop control reaches the floor
either — the set point is held by the fastest loop, and **two finite-bandwidth loops have no
fastest loop.**

---

## 5. THE CORRECTION TO RUNG 65 — the freeze was the MANIFOLD

Rung 65 § 3 demonstrated its continuum with a `b0` sweep: move the valve's initial position and
the FROZEN value tracks it one-for-one (`db/db0 = 1.0`, drift exactly 0) while both laws stay
exactly satisfied. That instrument is re-run here **verbatim**, with the fuel leg given a clock:

| | rung 65 (instantaneous fuel leg) | rung 66 (both lagged) |
|---|---|---|
| valve drift over the march | **0** (exactly frozen) | 4.23e−2 … 5.22e−2 |
| `d(b_end)/d(b0)` | **1.0** (a continuum) | **−8.24e−10** |
| withheld fuel vs `b0` | moves one-for-one | 84 % relative spread across ±0.01 |

**The frozen state is gone; the degeneracy is not.** § 2 says `det J ≡ 0` at every point and
every clock, including these. A zero eigenvalue means **no restoring force along a direction**,
not a state that sits still. Rung 65's instantaneous fuel leg pinned the state to the manifold
`φ_lp = φ_lim`, and *on* the manifold the marginal direction has nothing to drive it — so it
did not move. Give the fuel leg a clock and the state runs off-manifold and **drifts along that
same direction**. Same degeneracy, different observable.

**And the initial condition is still load-bearing, which is the half that matters
operationally.** `b_end` washes out because the valve reaches its lower stop, but the *path*
does not: a ±0.01 offset in the initial valve position changes the fuel the composite withholds
by **84 %** and the violation integral by ±20 %. A stable mode would have forgotten it. This
rung claims that spread as a MEASUREMENT and does not attribute it to the zero eigenvalue —
separating the two would need a non-degenerate pair to compare against, and § 2's scope table
shows the set-point offset that would build one leaves no riding points.

---

## 6. Reduce contract

* `tau=None` **and** `lag=None` ⇒ **rung 64 bit-for-bit**, by dispatch (inherited).
* `tau` set, `lag=None` ⇒ **rung 65 bit-for-bit**, by dispatch — the merged integrator is never
  entered and the state count is 3.
* `tau=None`, `lag` set ⇒ **rung 52's integrator bit-for-bit**, by dispatch — state count 3,
  the other three.
* `τ_g → 0` converges to rung 65; `τ_v → 0` converges to rung 52 on a bleed-limited plant.
  **Neither is bit-for-bit** — a different code path with a fourth state, exactly rung 65's
  two-armed disagreement now on two axes. Both are REPORTED per clock, never asserted to zero.

---

## 7. Predictions, scored

The anchor (`docs/plans/rung66-anchor-two-lag-cascade.md`) pre-registered eight, two designed
to fail. Scored honestly, including where the anchor's own central derivation was too weak.

| | prediction | verdict |
|---|---|---|
| **P1** | eigenvalues REAL across the whole grid; no oscillatory actuator mode | **HIT** — 13/13, 12/12, 13/13 sampled riding points real. But **for a stronger reason than predicted**: the anchor argued from `R_q C_g > 0` making the discriminant positive; the truth is `det J ≡ 0`, so the discriminant is `tr²`. The sign argument was a weaker route to a correct answer. |
| **P2** | the marginal mode survives with `b` still a constant of the motion | **SPLIT — the rung.** The *mode* survives identically (`det J ≡ 0` at every clock); the *frozen state* does not (`d(b_end)/db0 = −8e−10` against rung 65's 1.0). § 5. |
| **P3** | `τ_g`-invariance of the withheld fuel, ~1e−15 as rung 65's `τ_v`-invariance | **MISS.** That invariance was a property of the manifold, where `τ` multiplied a machine zero. Off-manifold nothing is machine-zero and the withheld fuel moves with both clocks. Same root cause as P2. |
| **P4** | *(designed to fail)* the mode is broken transiently, before both laws ride | **HIT, and stronger than written.** The anchor expected a resolvable *entry event* into an otherwise-degenerate ride. In fact the state is off-manifold for the whole ramp: the floor is genuinely undershot by −6.9e−3 and recovers only after it. |
| **P5** | *(designed to fail)* the pair is non-additive because the loops SUBSTITUTE, delivering ~the better single | **HIT on the mechanism, and the magnitude is far past what was written.** 152.96 % of standalone credit delivers 94.09 %; the second loop's marginal credit is **1.59 % against 60.46 % alone — 38× erosion**, not "a few percent" from the better single. |
| **P6** | rung 52's pre-crossing `tau_rel` bit-identity survives the merge | **HIT.** `first_diff` coincides with the crossing cell at every rate; identical against itself. A free merge-bug detector that found no bug. |
| **P7** | the HP debit is non-monotone in the clock RATIO with an interior worst case | **NOT RUN.** Displaced: § 2's identity made the spectrum the rung, and a debit shape on a plant whose set point is never held is not a clean object. Named as an open seam rather than scored. |
| **P8** | bandwidth remains PURE LOSS on both axes; no interior optimum | **NOT RUN**, same reason, and § 4 partly supersedes it: adding a *loop* is nearly pure loss (1.59 % for a whole limiter), which is the stronger statement the rung actually earned. |

**The anchor's central claim needs correcting in place.** It read:

> THE SECOND CLOCK CANNOT REACH THE MARGINAL MODE — PROVABLY, NOT MEASURABLY. The degeneracy
> locus `R_q C_g = 1` is a property of the two control LAWS through the plant; both time
> constants are a positive prefactor on it.

The conclusion is right and the reasoning is too weak. `R_q C_g = 1` is **not a locus** the
system might sit on or off — it is an **identity** forced by two laws sharing one constraint,
so `det J ≡ 0` everywhere and the clocks are powerless over it for a reason that has nothing to
do with their being a prefactor. The anchor also predicted the observable wrongly: it expected
the frozen state to persist, and § 5 shows the freeze belonged to the manifold.

---

## 8. Concessions

* Every one rungs 62/63/64/65 list, all inherited.
* `τ_g` and `τ_v` are swept coordinates on the march's own `s` — no attempt to anchor a real
  actuator bandwidth or a real limiter loop lag. ORDERING, SIGNS and INVARIANCES are the
  claims; every MAGNITUDE is disclaimed.
* The valve lag stays **SYMMETRIC** (rung 65's concession, verbatim) while the fuel leg is
  asymmetric. Asymmetry on both is a third constant and is not taken.
* **Cascade A is asserted against, not run.**
* `φ_lim` and `b_max` remain **IMPOSED** (rung 64's concession, verbatim).
* The eigenvalue measurement samples finitely many trajectory points, so it is a DIAGNOSTIC
  that can miss a brief excursion — a guard against rung 65's retracted trap, not a proof of
  convergence. Grid convergence is checked independently.
* The 84 % `b0` sensitivity of § 5 is reported as a MEASUREMENT and **not attributed** to the
  zero eigenvalue. Separating it from ordinary transient sensitivity needs a non-degenerate
  pair to compare against, and § 2's scope table shows the set-point offset that would build
  one leaves no riding points on this anchor.
* The fuel-leg-alone control's march **truncates at `s = 1.08`** of 1.70. The ramp-window
  violation integral cannot reach that far, so the ledger is unaffected — but no post-ramp
  number is quoted for that cell.

---

## 9. Next seams

* **CASCADE A** — rung 47's `tau_gov` `Tt4` governor beside this valve. It is the pairing this
  rung's identity does **not** cover: two loops on two DIFFERENT variables, hence no shared
  constraint, hence `R_q C_g ≠ 1` and `det J ≠ 0`. The cross-gains there have **opposite**
  signs (more bleed → less core flow → hotter at fixed fuel, so `∂R/∂q > 0`), so A admits the
  oscillatory actuator mode B provably cannot, with `τ_v/τ_g` as its `ρ`. Asserted against in
  `integrate_fuel`, not run.
* **The two-loop DEBIT shape** (the anchor's P7/P8) — displaced by the identity, and not clean
  on a plant whose set point is never held. It wants a case where the pair *does* reach the
  floor.
* **An ASYMMETRIC valve** — rung 65's standing concession, untouched here. This rung's fuel leg
  is asymmetric and its valve is not; making both asymmetric is a fourth constant.
* **THREE loops on one variable.** The identity is stated for a pair. `det J ≡ 0` for two laws
  on one constraint suggests a rank deficiency that GROWS with the loop count — an `n`-loop
  block on one manifold should have `n − 1` zero eigenvalues, so the third limiter would buy
  even less than the second's 1.59 %. Derivable, unmeasured.
* **Fuel + bleed + STATOR, all three on one plant** — rung 63's seam, still untouched by
  64/65/66.
