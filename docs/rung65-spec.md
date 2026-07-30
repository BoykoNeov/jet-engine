# Rung 65 — the LAGGED BLEED VALVE: what a finite bandwidth costs, and what it gives back

Rung 64's named next seam. Every valve from rung 42 to 64 is INSTANTANEOUS — a pure function
of the state, re-solved at every sub-evaluation. This one has a bandwidth: the position becomes
a THIRD STATE relaxing toward rung 64's command with a time constant `τ`.

**HEADLINE: a lag repairs the SOLVE without removing the DEGENERACY.**

Two loops on one variable are redundant, and the redundancy is **conserved**. Rung 64's
instantaneous valve hid it inside a solver, where it was a roundoff coin flip and no number
about it was a result. A finite bandwidth moves it into the STATE, where it is a **marginal
mode**: exactly frozen (`db/ds` ≤ 1e−14 over a whole march), `τ`-invariant to 1e−15 across a
20× range, a genuine one-parameter family bounded above by the valve's own minimality law, and
selected by the initial condition alone.

Two corollaries the same sweeps establish: rung 64's ceiling gains **BANDWIDTH** as a second
hardware axis and it is **PURE LOSS** (worse protection *and* more bleed); and rung 64 § 4's
destroyed minimum-location is **RESTORED** at any finite bandwidth — while at the valve's stop,
bandwidth buys exactly nothing, which is rung 64's own headline found in a second place.

Anchor: `docs/plans/rung65-anchor-lagged-valve.md` (six predictions HIT, two REFUTED, and one
§ 0 pre-check RETRACTED as a numerical artifact — all three published in full).

---

## 0. THE INSTRUMENT, and the pre-check that would have counterfeited the rung

Rungs 47/52 start their third state at `g = 0`, because a fuel clip is zero before engagement.
A valve position is not. Measured before any prediction was written: on rung 64's own grid
**`b_cmd(0) = 0.036626`** — the limiter is already RIDING on the running line the march starts
from, with `φ_lp(0)` sitting exactly on the floor. Starting at `b = 0` would have injected a
startup transient straight into the **early-ramp LP minimum**, which is the binding one (rungs
41/44), and every number below would have measured that instead of the lag. So

    b(0) = b_cmd(0)   —   the EQUILIBRIUM valve position

which is physics, not convenience: a lag is a purely transient object, so at equilibrium the
valve has caught up. Every STEADY solve on a lagged machine (`equilibrium`, `fuel_for_Tt4`, the
running line) therefore runs rung 64's instantaneous law — which is also what makes a lagged
march start on the SAME running line as the instantaneous march it is compared against.

### Scope, and a modelling floor found rather than assumed

Steady flight condition, fully-choked branch, CPG working gas, two-spool map-matched plant,
rung 45's accel fuel ramp — rung 64's grid throughout (`φ_surge` = 0.55, `b_max` = 0.10,
`φ_lim` = 0.80, `r` = 0.5, `ds` = 0.005).

**`τ` cannot be swept below ~`ds`/2.** `db/ds = (b_cmd − b)/τ` under an explicit RK4 needs
`z = ds/τ` inside the stability region (|z| ≲ 2.78). A first pre-check ran `τ` = 0.002 at
`ds` = 0.01 — `z = 5` — and returned `∫b ds` = 0.169 against the grid-converged 0.0383, which
briefly looked like the finding *"a fast valve bleeds more"*. It is not a finding; it is
instability. The retraction is published in the anchor's § 3 because the artifact is easy to
reproduce and looks physical. The whole scored sweep sits at `z ≤ 0.5` and is checked against a
5× finer grid.

---

## 1. THE SECOND HARDWARE AXIS — bandwidth, and it is PURE LOSS

Rung 64: the ceiling on the protected coordinate is `min φ` over the fully-open march, a
property of `b_max` — the lever's AUTHORITY, which is hardware. A valve that cannot reach its
command in time does not deliver its set point either, and it fails for a reason no control law
can touch. Six bandwidths of the SAME law, against rung 64's instantaneous valve:

| `τ` | `min φ_lp` | undershoot | `∫b ds` | `b` peak | `plateau_pts` |
|---|---|---|---|---|---|
| 0.4 | 0.770917491 | −2.908e−2 | 0.051785 | 0.06481 | 1 |
| 0.2 | 0.776806981 | −2.319e−2 | 0.045687 | 0.07620 | 1 |
| 0.1 | 0.783291203 | −1.671e−2 | 0.042005 | 0.08520 | 1 |
| 0.05 | 0.789122771 | −1.088e−2 | 0.040132 | 0.08995 | 1 |
| 0.02 | 0.794571011 | −5.429e−3 | 0.039003 | 0.09172 | 1 |
| 0.01 | 0.797007245 | −2.993e−3 | 0.038626 | 0.09197 | 1 |
| **rung 64** | **0.800000000** | **0 (exact)** | **0.038249** | 0.09202 | **114** |

Both currencies are monotone and they run the **same** way:

> a slower valve protects LESS and bleeds MORE.

It opens late, missing the minimum, and closes late, dumping air after the need has passed —
and the closing tail dominates the integral. There is no trade to buy: bandwidth is pure loss.
Rung 64's floor delivers its set point EXACTLY (rung 60's tautology); no finite bandwidth does,
and the deficit is `τ`-monotone from −3.0e−3 to −2.9e−2.

The `τ → 0` arm of the reduce is measured rather than asserted: `dev` = max |`φ_lp(τ)` −
`φ_lp`(rung 64)| on the same grid shrinks monotonically, with consecutive-halving ratios 1.035,
1.307, 1.613, **1.905** — approaching first order from below. It cannot be first order at large
`τ` because `dev` **saturates**: it is bounded above by the valve-shut march's own deficit
(6.46e−2).

Reader: `LaggedBleedTransient.bandwidth_ceiling`.

---

## 2. RUNG 64 § 4's DESTROYED ARGMIN, RESTORED

Rung 64 § 4: a floor that RIDES pins `φ_lp` to `φ_lim` over an *interval*, so the minimum's
VALUE is a result (rung 60) and its LOCATION is not one — the argmin is a 1-ulp tie, and
doubling an untouched `b_max` moved `s_at_min_lp` by a factor 3.3 while every physical quantity
moved by ≤ 6.6e−16. That bounded every rung-44-to-52 reader that reports WHERE a minimum sits.

A trailing actuator cannot pin what it has not caught up to. Measured: `plateau_pts` == 1 and
`plateau_span` == 0 **exactly, at every `τ`** — against 114 for the instantaneous floor on the
same grid. And rung 64's plateau is a genuine interval rather than a tie: it scales ∝ 1/`ds`
(57 / 114 / 227 at `ds` = 0.01 / 0.005 / 0.0025) while the lagged march stays at 1 on all three.

The restored argmin is a RESULT and not a survivor, over a 4× refinement at `τ` = 0.05:

| `ds` | lagged `s_at_min` | lagged `min φ_lp` | rung 64 `s_at_min` |
|---|---|---|---|
| 0.01 | 0.0900 | 0.789129958 | 0.3800 |
| 0.005 | 0.0850 | 0.789122771 | 0.3000 |
| 0.0025 | 0.0875 | 0.789121707 | 0.0675 |

Two grid cells against a factor of 5.6.

**What this does and does not license.** Rung 64's bound on rungs 44–52 is not lifted — it is
LOCALISED to the idealisation that produced it. A minimum-location reading on a floored plant
is meaningless when the actuator is instantaneous and is a measurement when it is not. Since
every real actuator is, the bound is a property of the model, which is the honest form of the
statement and not a weaker one.

### The one place bandwidth buys nothing

A floor set above the fully-open march's own minimum SATURATES (rung 64 § 1's witness,
`φ_lim` = 0.890467). Under a lag it is a bare exponential approach with no feedback content:

| law | `min φ_lp` | deficit | `∫b ds` |
|---|---|---|---|
| rung 64 | 0.807362922 | −8.310e−2 | 0.075003 |
| `τ` = 0.01 | 0.807362922 | −8.310e−2 | 0.075985 |
| `τ` = 0.05 | 0.807362922 | −8.310e−2 | 0.079937 |
| `τ` = 0.2 | 0.807362922 | −8.310e−2 | 0.094678 |

**`τ`-invariant to nine digits** on the protected coordinate while `∫b` still pays the
pure-loss bill. Where the valve is against its stop, bandwidth is exactly as powerless as law
was — rung 64's *"the ceiling is the AUTHORITY"* found in a second place, on a second axis.

It also carries a methodological warning the gate obeys: a saturated cell has
`plateau_pts` == 1 for the INSTANTANEOUS floor too, for a reason that has nothing to do with
tracking error. § 2's claim must be read on RIDING cells only, or it passes for the wrong
reason.

---

## 3. THE RUNG — the degeneracy is CONSERVED, not removed

Rung 64 § 3: an instantaneous valve re-pins `φ_lp` to `φ_lim` at ANY fuel, so rung 49's
`_surge_fuel` solves `G ≡ 0` across its whole bracket and returns an arbitrary point of a
continuum — *"a closed-loop lever does not DISARM a second limiter on the same variable, it
DELETES that limiter's PLANT"*, and its residual's very existence was a roundoff coin flip.

**The predicted repair happens.** Inside any one derivative evaluation the lagged valve is a
CONSTANT, so the fuel leg sees rung 42's imposed-valve plant. Exhibited directly — the same
fuel bracket swept on both plants at one state off an armed march (`s` = 0.085, `b` = 0.055):

| plant | `φ_lp` across [0.90, 1.00]·`Wf` | span |
|---|---|---|
| INSTANTANEOUS | 0.800000000 (×5) | **1.78e−15** |
| LAGGED | 0.789123 → 0.790477 → 0.791837 → 0.795949 → 0.802915 | **1.379e−2** |

Ratio 7.76e12, monotone in fuel with a strict sign change in `G`. Rung 49's own premise — *"φ
falls monotonically with fuel at fixed spool speeds"* — is restored verbatim, and rung 64's
deletion is EXHIBITED where it could only be derived.

**And the continuum survives anyway.** A fuel floor and a valve floor on the same variable are
two actuators regulating ONE quantity, so wherever both ride, every `(b, Wf)` on the curve
`φ_lp = φ_lim` satisfies BOTH laws at once. The valve's command is therefore satisfied wherever
the fuel leg has left it:

    b_cmd(state, Wf(b))  ==  b        ⇒        db/ds ≡ 0        for every τ

Measured on the composite: `b_cmd − b` ≤ 2.7e−15, `b` drift ≤ 6.5e−16 end-to-end, and
`fuel_removed` = 2.024341e−2 **identical to seven digits at `τ` = 0.2, 0.05 and 0.01**
(`τ`-span 1.03e−15 relative across a 20× range). `τ` multiplies a machine zero and cannot
reach the mode. (A pre-check also saw the same seven digits at `τ` = 0.002 on a grid § 0's
stability floor now forbids — legitimately, because on the composite `db/ds` is machine-zero
so the stiff mode is never excited; it is not reproducible on the shipped plant and is not
quoted as evidence.)

> **A LAG REPAIRS THE SOLVE WITHOUT REMOVING THE DEGENERACY.** The redundancy of two loops on
> one variable is CONSERVED — it moves out of the solver, where it was roundoff, and into the
> STATE, where it is a marginal (zero-eigenvalue) mode fixed by the initial condition alone.

### The continuum has an EDGE, and the edge is the valve's own law

A frozen state could be one initial condition's coincidence. A CONTINUUM means the frozen value
moves with `b0` while both laws stay exactly satisfied:

| `b0` / `b_cmd(0)` | 0.500 | 0.727 | 0.990 | **1.000** | 1.010 | 1.050 | 1.200 |
|---|---|---|---|---|---|---|---|
| drift | 2.4e−16 | 3.3e−16 | 6.5e−16 | 6.0e−16 | 1.2e−5 | 6.1e−5 | 4.4e−2 |
| `fuel_removed` | 1.326e−2 | 1.736e−2 | 2.070e−2 | 2.024e−2 | — | — | — |

Every member at or below `b_cmd(0)` is exactly frozen, holds `φ_lp = φ_lim` to 1.3e−15 with the
valve strictly interior, and withholds a **different** amount of fuel — 1.326e−2 to 2.070e−2
across the tabulated members, a 56 % spread. Above it, the valve closes. (The gate pins this
as a ONE-SIDED RATIO on the ±0.01 pair — measured 1.166, asserted `> 1.10` — rather than as
the tabulated extremes: the magnitude is a grid measurement, the materiality is the claim.)

**The edge is derivable.** The valve's law is *the SMALLEST position holding the floor*. Above
`b_cmd(0)` the valve is doing more than its own law asks, so its command sits below the live
position and it closes; at or below it, the fuel leg takes up the slack and the pair is exactly
redundant. The family is `b0 ∈ (0, b_cmd(0)]` — and the physical initial condition sits
precisely ON its upper edge, which is why the natural march looks like a unique solution.

**What this does to rung 64 § 3.** It SHARPENS it and corrects its prognosis. Rung 64's own
next-seam text predicted *"the second limiter should get part of its plant back"*. It gets ALL
of its plant back — and the composite is still under-determined, because the under-determinacy
was never the solver's. Rung 64's *"a limiter whose plant has been deleted does not fail, it
grinds"* becomes: **with a real actuator it neither fails nor grinds — it answers, definitely
and reproducibly, and the answer depends on where the valve happened to be.**

Readers: `LaggedBleedTransient.fuel_authority` (the discriminator), `.marginal_mode` (the rung).

---

## Verification gates (`tests/test_rung65.py`)

**Gate 1 — THE REDUCE, both arms.** `tau=None` marches bit-for-bit rung 64 through the new
class (341 points × 7 keys), on the unarmed, constant, schedule and floor arming modes alike;
the dormant floor still dispatches to the rung-63 grandparent; the single-spool design run is
bit-for-bit rung 6. The `τ → 0` arm is the CONVERGENCE (`dev` shrinking monotonically), gated
as a limit and explicitly NOT as equality — a different code path with a third state.

**Gate 2 — THE OBJECT.** `tau = 0` is refused by assertion (the instantaneous valve is
`tau=None`, a different object); a lagged limiter handed to rung 64's `LimitedBleedTransient`
is refused rather than silently unlagged; the two-lag CASCADE (`tau_gov` / `AsymmetricLag`) and
rungs 50/51's forced edges are refused on the lagged path.

**Gate 3 — THE TRAP, fifth instance.** `at_lever` / `at_stator` return THIS class carrying the
lag, and a lagged machine's `b_at_point` refuses a trajectory point that did not record the
position — the direct correction of rung 64's re-solve comment. `b0` is a per-MARCH argument,
so no sibling constructor can drop it.

**Gate 4 — BANDWIDTH IS PURE LOSS.** Undershoot and `∫b ds` both monotone in `τ`, in the same
direction, bracketed by rung 64's exact set point and zero deficit; `dev` monotone; and the
saturated floor's `min φ_lp` `τ`-INVARIANT to 1e−9 while its `∫b` is not.

**Gate 5 — THE ARGMIN RESTORED.** `plateau_pts` == 1 and `plateau_span` == 0 at every `τ` on
RIDING cells, against rung 64's ≥ 100 on the same grid and its ∝1/`ds` growth; `s_at_min_lp`
stable within 2 grid cells over a 4× refinement. The saturated cell is asserted to be EXCLUDED
from this gate, by name.

**Gate 6 — THE DEGENERACY, both halves.** The discriminator: instantaneous span < 1e−9,
lagged span > 1e−3 and monotone in fuel with a sign change. The marginal mode: drift ≤ 1e−12
for `b0` ≤ `b_cmd(0)`, `φ_lp = φ_lim` to 1e−12 with the valve interior, `fuel_removed` genuinely
different between members, `τ`-invariance ≤ 1e−9 relative, and the EDGE — frozen at 0.99× and
1.00× `b_cmd(0)`, not frozen at 1.01×.

**Gate 7 — the modelling floor.** Every march stays on the choked branch; and the RK4 stability
constraint is gated as an assertion on `ds/τ`, so a future sweep cannot silently reproduce the
retracted artifact.

---

## Concessions

Every one rungs 62/63/64 list, all inherited, plus:

- **The lag is SYMMETRIC — one constant.** A real bleed valve opens and closes at different
  rates, and rung 52 showed a min-select leg's asymmetry is where its trigger-pinning lives.
  Not taken, on rung 52's own argument: `tau_close` is never read while `b_cmd > b`, so the
  pre-crossing march is bit-identical across a `tau_close` sweep and rung 52's one line already
  says what it would find. A second constant also doubles a sweep over marches that each carry
  an outer root per sub-evaluation. Named as this rung's next seam.
- **`τ` is a swept coordinate on the march's own `s`**, like rungs 47/51/52's constants. No
  attempt is made to anchor a real actuator's bandwidth, so the MAGNITUDE of every deficit is
  disclaimed; the ORDERING, the SIGNS and the invariances are the claims.
- **`τ` is bounded BELOW by the integrator** (`ds`/`τ` ≲ 2.78, § 0). The `τ → 0` limit is
  therefore approached and never reached, which is exactly the right epistemic position for a
  rung whose finding is that the limit is singular in one respect and smooth in another.
- **The two-lag CASCADE is refused, not run.** A lagged valve beside a lagged fuel leg is four
  states and two clocks — rung 52's own standing seam, from the airflow side. Asserted against.
- **`φ_lim` and `b_max` remain imposed** (rung 64's concession, verbatim).
- **The marginal mode is exhibited on ONE composite pair.** That a floor on `φ` plus a valve on
  `φ` are redundant is derived and general; that the edge is `b_cmd(0)` is derived from the
  valve's minimality law; the 56 % spread in withheld fuel across § 3's tabulated members
  (1.326e−2 … 2.070e−2) is a measurement on this grid — the gate pins only that the spread is
  material, as a one-sided ratio.

## What it does to its neighbours

- **SHARPENS rung 64 § 3 and corrects its prognosis** — the second limiter gets ALL of its
  plant back, and the composite is still under-determined (§ 3).
- **LOCALISES rung 64 § 4's bound on rungs 44–52** — a minimum-location reading on a floored
  plant is meaningless under an instantaneous actuator and a measurement under a real one
  (§ 2).
- **EXTENDS rung 64's ceiling to a second hardware axis** — authority AND bandwidth — and
  shows bandwidth is pure loss (§ 1).
- **CONFIRMS rung 64's headline at the stop** — where the valve saturates, bandwidth buys
  nothing, exactly as law bought nothing (§ 2).
- **REFUTES the "a lagged valve is a slow constant valve" reading** — the HP debit is
  non-monotone in `τ` with an interior worst case and never becomes a credit (anchor P7).
  Lateness is not persistence.
- **EXTENDS rung 52's trigger-pinning argument to an airflow lever**, by transfer rather than
  by measurement (Concessions).

## The next seam

**THE ASYMMETRIC VALVE, and the two-lag CASCADE.** Both halves of rung 52's standing seam,
reachable from the airflow side and refused here on purpose. `tau_open ≠ tau_close` is the
shape; and a lagged valve beside rungs 47/52's lagged FUEL leg is the cascade — four states,
two clocks, and the first plant in the ladder where § 3's marginal mode has a second dynamic
object to interact with. Rung 52 § 3 predicts the cascade should not be additive; § 3 here
predicts something sharper, that the second clock cannot reach the marginal mode either.

Still open beside it: **fuel + bleed + STATOR**, all three on one plant (rung 63's, untouched
by 64 and by this).

## Anchor

`docs/plans/rung65-anchor-lagged-valve.md` — the derivation written before any probe, the two
pre-checks declared as the rung's given, eight numbered predictions with six HIT and two
REFUTED published in full, and one pre-check RETRACTED as an integrator artifact.
