# Rung 66 anchor — THE TWO-LAG CASCADE

**Status: DERIVATION ONLY. Written before any probe.** Every number below is a prediction,
not a measurement. Predictions are scored HIT / MISS after the sweep, and a MISS is published
in place, as in rungs 51/58/63/64/65.

Rung 65's named next seam, and rung 52's own standing one, reached from the airflow side:
**a lagged bleed VALVE beside a lagged FUEL leg.** Four states, two clocks.

---

## 0. WHICH cascade — and why the other one is the seam

Rung 65 § 3 stakes a specific prediction: *"the second clock cannot reach the marginal mode
either."* The marginal mode is **two loops on ONE variable** (`φ_lp`). Only a φ-referenced
fuel leg tests it. That fixes the pairing:

| | fuel leg | watches | tests |
|---|---|---|---|
| **B — TAKEN** | rung 52's `AsymmetricLag` over rung 49's `surge` floor | `φ_lp` | **rung 65 § 3**: can a second clock reach the marginal mode? |
| A — the seam | rung 47's `tau_gov` topping governor | `Tt4` | rung 52 § 3's non-additivity (two clocks, two *different* variables) |

A is a legitimate rung; it is not the one rung 65's spec promised. **One rung, one headline** —
A is named as this rung's next seam and asserted against, exactly as rung 65 asserted against
this one.

**B is reachable with the shipped code and does not need a new leg.** `_integrate_fuel_asym`'s
`required` (engine.py:5129–5135) already min-selects over `accel` **and** `surge`, so rung 52's
lag is a lag on the *composite* min-select requirement, φ leg included. The rung is a MERGE of
two existing integrators (`_integrate_fuel_asym` + `_integrate_fuel_valve_lag`), not a new law.

---

## 1. The plant — four states, two clocks

Spools `ν_lp, ν_hp` (rung 40), the fuel-clip amount `g` (rung 52), the valve position `q`
(rung 65). Holding the spools frozen — they are the SLOW states, which is the entire premise of
calling `g` and `q` actuator lags —

    dg/ds = ( R(ν, q) − g ) / τ_g          R = max(0, mf_sched − min(armed caps))   [rung 52]
    dq/ds = ( C(ν, g) − q ) / τ_v          C = b_cmd, the smallest b holding φ_lp ≥ φ_lim [65]

**The coupling is real by CONSTRUCTION, and that is an assumption with a physical
justification, not a discovery.** `R` is evaluated with `_b_state` set — the fuel leg solves
its cap against the plant *as the valve actually is*, because a real limiter watches the
machine it is on, not a machine with an idealised valve. Symmetrically `C` is read at the
APPLIED fuel `mf_sched − g` (rung 65's own choice, verbatim). So `R` reads `q` and `C` reads
`g`: the two actuators are cross-coupled through the plant even though neither law mentions
the other.

**THE `_b_state` BOUNDARY IS THE RUNG-62 `_powers` TRAP, RELOADED.** Every closure call that
represents *the plant* (`_instant_fuel`, `_surge_fuel`, `_sched_fuel`, `_topping_fuel`) must run
with `_b_state` set; only `command`/`_solve_b`, which computes `b_cmd` over TRIAL positions,
must run without it. Get the boundary wrong and a solver converges on a residual the plant never
uses — with no test failing. Rung 65's `der` clears `_b_state` in a `finally` before calling
`command`; the merged `der` must preserve that and extend it over `required`.

---

## 2. The linearised actuator block — and rung 65 § 3 falls out of it

With `R_q = ∂R/∂q` and `C_g = ∂C/∂g`,

    J = [ −1/τ_g    R_q/τ_g ]        tr J = −(1/τ_g + 1/τ_v)  < 0   always
        [ C_g/τ_v   −1/τ_v  ]        det J = (1 − R_q C_g) / (τ_g τ_v)

    discriminant:  tr² − 4 det  =  (1/τ_g − 1/τ_v)²  +  4 R_q C_g / (τ_g τ_v)

### The cross-gain SIGNS are derivable before running anything — and on B they AGREE

* `R_q < 0`. More bleed raises `φ_lp` (rung 64: that is *why* the valve is the protection
  lever), so the fuel floor needs to clip LESS to hold `φ_lim`.
* `C_g < 0`. More clip means less applied fuel, which raises `φ_lp` (rung 49: the floor works
  by cutting fuel), so the valve needs LESS bleed to hold `φ_lim`.

**Both negative ⇒ the product `R_q C_g > 0`.** This is the structural difference between B and
A, and it inverts the naive expectation transferred from rung 40:

* the discriminant is `(1/τ_g − 1/τ_v)² + (positive)` ⇒ **strictly positive** ⇒ eigenvalues are
  **REAL**. There is **NO oscillatory actuator mode** on B. (On A the gains have OPPOSITE signs
  — `∂R/∂q > 0` via more bleed → less core flow → hotter at fixed fuel — so A's product is
  negative and an oscillatory mode is *admissible* there, with `τ_v/τ_g` as its ρ. That is A's
  content, and another reason it is a separate rung.)
* the two loops are **mutually SUBSTITUTING**, not competing: each one's action does the other's
  job. That is rung 65 § 3's redundancy, in linear algebra.

### THE MARGINAL MODE IS `det J = 0`, AND THE CLOCKS ARE NOT IN IT

    det J = 0    ⇔    R_q C_g = 1

A zero eigenvalue is a CONTINUUM of equilibria — precisely rung 65 § 3's "constant of the
motion, selected by the initial condition." And `τ_g, τ_v` enter `det J` **only through the
strictly positive factor `1/(τ_g τ_v)`**, so they cannot move the locus. Hence, before any
probe:

> **THE SECOND CLOCK CANNOT REACH THE MARGINAL MODE — PROVABLY, NOT MEASURABLY.** The
> degeneracy locus `R_q C_g = 1` is a property of the two control LAWS through the plant; both
> time constants are a positive prefactor on it. Rung 65 § 3's prediction is upgraded from a
> conjecture to a derivation, and the rung's job is to EXHIBIT it, not to discover it.

If this survives the sweep it makes rung 66 a *sharpening* rung. **The honest risk is that it
is too clean** — see § 4 P4, which is the prediction designed to break it.

---

## 3. THE STABILITY FLOOR — the #1 fake-finding risk, and it is worse than rung 65's

Rung 65 published a RETRACTION: an RK4 instability at `z = ds/τ = 5` returned `∫b ds` 4.4× the
converged value and *looked exactly like a physical finding* ("a fast valve bleeds more"). The
cascade doubles the exposure and rung 47's `_integrate_fuel_lagged` (engine.py:5028–5101) has
**no `ds/τ` assert at all**. Two facts make a naive floor wrong:

1. There are two `z`'s, and
2. **the stiff eigenvalue is NOT `max(1/τ_g, 1/τ_v)`.** On the complex branch `|λ| = √det`;
   with `R_q C_g < 0` that is `√((1+|R_q C_g|)/(τ_g τ_v))`, which can exceed BOTH diagonal
   rates. (On B, with `R_q C_g > 0`, coupling *lowers* `det` — but the real branch's larger root
   still needs bounding.)

**The floor is therefore DERIVED from the block, and MEASURED rather than assumed:**

    ρ_spec = max |λ(J)|,  from R_q, C_g finite-differenced at the march's initial point
    assert  ds · ρ_spec  ≤  2.0                       (explicit RK4, negative real axis)
    backed a-priori by   ds / min(τ_g, τ_v)  ≤  2.0   (the diagonal-only bound, rung 65's)

Both are asserted. This is a strict improvement on rung 65's constant-only floor and should be
said as one: **rung 65's floor was a scalar because it had one state; a cascade's floor is a
spectral radius.** Re-checked along the trajectory, not only at `s = 0`, because `R_q` and `C_g`
are state-dependent (the valve's kinks at `b_cmd = 0` and `b_max`, and the floor's own crossing).

---

## 4. Pre-registered predictions

Scored HIT / MISS after the sweep. Two are designed to fail.

**P1 — the eigenvalues are REAL across the whole `(τ_g, τ_v)` grid.** From § 2's sign argument.
No oscillation in `g`, `q`, or `φ_lp` on B, at any clock ratio — including matched clocks, where
A would be *most* prone to it (the `(1/τ_g − 1/τ_v)²` term vanishes at `τ_g = τ_v`, so a negative
product needs the least help there; B's positive product means matched clocks are the *safest*
point, not the most dangerous). **Refutes the transfer of rung 40's inter-spool mode to the
actuator side.**

**P2 — the marginal mode SURVIVES the second clock, with `b` still a constant of the motion.**
Wherever both laws ride, `dq/ds` and `dg/ds` are both machine-zero, `φ_lp = φ_lim` holds to
~1e−12, and the withheld fuel still varies with `b0` — rung 65 § 3's `_stator_march(b0=…)`
instrument, now on a 4-state plant. From § 2's `det J = 0` argument.

**P3 — `τ_g`-invariance of the withheld fuel, matching rung 65's `τ_v`-invariance.** Rung 65
measured `fuel_removed` identical to seven digits across a 20× `τ_v` range because `τ` multiplies
a machine zero. The same argument applies to `τ_g` on the composite, so the span should be
~1e−15 relative across a comparable `τ_g` range.

**P4 — DESIGNED TO FAIL: the marginal mode is BROKEN transiently, before both laws ride.**
§ 2's `det J = 0` is an argument about the *riding* equilibrium. Before the fuel floor's
crossing, `R = 0` and only the valve is live, so `R_q ≡ 0`, `det J = 1/(τ_g τ_v) > 0` — no
degeneracy. The prediction is that the plant is therefore **well-posed on approach and
degenerate only on arrival**, so the *entry into* the marginal mode is a resolvable event with a
location that moves with `τ_g`. If P4 hits, § 2's clean derivation is TRUE BUT INCOMPLETE, and
the rung's headline has to carry both halves. **This is the prediction most likely to become the
rung.**

**P5 — DESIGNED TO FAIL: the cascade is NOT additive (rung 52 § 3), and B is where that breaks
down.** Rung 52 predicted non-additivity generally. But if both loops substitute (§ 2), the
*delivered* protection may be almost exactly the better of the two acting alone — i.e.
`min`-like rather than additive, and *closer* to additive-in-the-trivial-sense than rung 52
expected. Predicting: the pair's `min φ_lp` is within a few percent of the single-lever result,
so **"not additive" is right for the wrong reason** — they do not add because they *substitute*,
not because they interfere.

**P6 — the free merge-bug detector.** Rung 52's structural fact — `tau_rel` is never read while
`required > g`, so the pre-crossing march is BIT-IDENTICAL across a `tau_rel` sweep — must
survive the merge. Predicted: bit-for-bit, exactly as at rung 52. A MISS here is not a finding,
it is a **bug**: it means the merged integrator started reading the release constant, or the
`_b_state` boundary of § 1 leaked.

**P7 — the HP debit is non-monotone in the clock RATIO, with an interior worst case.** Rung 65
found the HP debit non-monotone in `τ_v` alone with an interior worst case ("lateness is not
persistence"). Predicting the same shape survives in `τ_v/τ_g` and that the worst case is NOT at
either extreme.

**P8 — bandwidth remains PURE LOSS on both axes.** Rung 65: a slower valve is worse protection
*and* more bleed. Predicting no `(τ_g, τ_v)` pair beats the instantaneous pair on both, i.e. the
two-clock version has no interior optimum either. **A MISS is a real finding** — it would be the
first bandwidth optimum in the ladder.

---

## 5. Reduce contract

* `lag=None` **and** `tau=None` ⇒ rung 64, bit-for-bit, by dispatch (inherited).
* `lag=None`, `tau` set ⇒ **rung 65 bit-for-bit**, by dispatch — the merged integrator is not
  entered and the state count is 3.
* `tau=None`, `lag` set ⇒ **rung 52 bit-for-bit**, by dispatch — state count 3, the other three.
* `τ_g → 0` ⇒ converges to rung 65 (instantaneous fuel leg); `τ_v → 0` ⇒ converges to rung 52 on
  a bleed-limited plant. **Neither is bit-for-bit** — a different code path with a fourth state,
  exactly rung 65's two-armed disagreement, now on two axes. Both deviations are REPORTED per
  clock, never asserted to zero.

## 6. Concessions (declared before the probe)

* Every one rungs 62/63/64/65 list, all inherited.
* **`τ_g` and `τ_v` are swept coordinates on the march's own `s`** — no attempt to anchor a real
  actuator bandwidth or a real limiter loop lag. ORDERING, SIGNS and INVARIANCES are the claims;
  every MAGNITUDE is disclaimed.
* **The valve lag stays SYMMETRIC** (rung 65's concession, verbatim) while the fuel leg is
  asymmetric. Asymmetry on BOTH is a third constant and is not taken.
* **Cascade A is asserted against, not run** (§ 0).
* **`φ_lim` and `b_max` remain imposed** (rung 64's concession, verbatim).
* The spectral floor of § 3 is evaluated at finitely many trajectory points, so it is a
  DIAGNOSTIC that can miss a brief excursion — it is a guard against the rung-65 trap, not a
  proof of convergence. Grid convergence is checked independently at one `(τ_g, τ_v)` pair.
