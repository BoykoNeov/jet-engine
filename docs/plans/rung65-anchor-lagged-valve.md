# Rung 65 anchor — the LAGGED BLEED VALVE

Rung 64's named next seam, verbatim from `docs/rung64-spec.md` § "The next seam":

> **THE LAGGED VALVE.** § 3's deletion and § 4's plateau both rest on the valve being
> INSTANTANEOUS — it pins `φ` at every sub-evaluation, which is what makes `dφ/dWf` exactly
> zero and the minimum exactly flat. Give it rung 47's first-order lag and neither can survive
> unchanged: a trailing-edge actuator cannot pin what it has not caught up to, so the plateau
> should break into a genuine minimum and the second limiter should get part of its plant back.

Every valve from rung 42 to 64 is INSTANTANEOUS. This one has a bandwidth.

---

## § 0 — THE PRE-CHECKS, run before any prediction was written

Rung 64's own precedent: two discriminating probes decide whether the rung has content, and
**their results are declared here as the rung's GIVEN, not as scored predictions.** § 2
pre-registers only what they did not touch. Everything in this section was measured at
`ds = 0.01` on rung 64's grid (`φ_surge` = 0.55, `b_max` = 0.10, `φ_lim` = 0.80, `r` = 0.5,
shaped map).

### Probe A — the INITIAL CONDITION. Is `b_cmd(0) = 0`?

The advisor's question, and it was load-bearing. Rungs 47/52 start their third state at `g = 0`
because a clip is zero before engagement. A valve position is not: if the limiter is already
riding on the running line the march starts from, then starting at `b = 0` injects a startup
transient into the **early-ramp LP minimum**, which is the binding one (rungs 41/44) — and
every number in this rung would be measuring that instead of the lag.

Measured: **`b_cmd(0) = 0.036626` — the valve is RIDING at `s = 0`**, `φ_lp(0) = 0.800000`
sitting exactly on the floor. So the initial condition is

    b(0) = b_cmd(0)   —   the EQUILIBRIUM valve position

which is not a convenience but the physics: a lag is a purely transient object, so at
equilibrium the valve has caught up and every steady solve on a lagged machine runs rung 64's
instantaneous law. That is also what makes a lagged march start on the SAME running line as
the instantaneous march it is compared against.

### Probe B — the CREDIT vs `τ`. Does rung 64 § 3's deleted plant come back?

The pre-registered mechanism was: under a lag the valve is a CONSTANT inside any one
derivative evaluation, so the fuel leg sees rung 42's imposed-valve plant with `dφ/dWf < 0`
strictly, and rung 64 § 3's degeneracy cannot survive. The discriminating measurement was
`fuel_removed` on the composite (rung 49's φ fuel floor + this valve) against its two
endpoints. Measured:

| plant | `fuel_removed` | `min φ_lp` |
|---|---|---|
| bare (fuel leg alone, no valve) | 8.836327e−3 | 0.800000000 |
| rung 64 (INSTANTANEOUS valve + leg) | 1.924860e−4 | 0.800000000 |
| lagged, `τ` = 0.2 | **2.024341e−2** | 0.800000000 |
| lagged, `τ` = 0.05 | **2.024341e−2** | 0.800000000 |
| lagged, `τ` = 0.01 | **2.024341e−2** | 0.800000000 |
| lagged, `τ` = 0.002 | **2.024341e−2** | 0.800000000 |

**Identical to seven digits across a 100× range in `τ`.** The trace says why:

    max |b(τ=0.2) − b(τ=0.002)|  over the march  =  1.9e−14
    max |b_cmd − b| / τ          (i.e. max |db/ds|) =  2.7e−15
    b(0) = 0.036626363670182814     b(end) = 0.03662636367018285

The valve is **FROZEN at `b(0)` for the whole march**, at every `τ`. The mechanism is not the
one predicted, and it is stronger: a fuel floor and a valve floor on the SAME variable are two
actuators regulating ONE quantity, so wherever both ride, **every `(b, Wf)` pair on the curve
`φ_lp = φ_lim` satisfies BOTH laws at once**. The valve's command is therefore satisfied
wherever the fuel leg has left it, `db/ds ≡ 0`, and `τ` multiplies a machine zero.

> The predicted mechanism (`dφ/dWf < 0` at frozen `b`, so the leg's solve is repaired) is
> CORRECT — the leg returns a definite, reproducible clip of 1.396e−2 where rung 64's was a
> roundoff coin flip. The predicted CONSEQUENCE ("the second limiter gets part of its plant
> back", rung 64's own words) is WRONG: the composite is still under-determined. The
> continuum did not go away. It moved out of the solver and into the STATE.

### Probe C — the valve ALONE, two bandwidths (fell out of probe B's debug trace)

Declared here rather than scored, because it was seen before the predictions were written:

| `τ` | `min φ_lp` | `∫b ds` (rectangle) |
|---|---|---|
| 0.2 | 0.776822 (undershoot −2.32e−2) | 0.0459 |
| 0.002 | 0.800000 (undershoot ~0) | 0.1699 |

A slow valve **protects less AND bleeds less**. The pre-registered "a lag is pure loss on both
axes" is dead on arrival; what § 2 scores is the SHAPE of the trade, not its sign.

### What § 0 fixes — the axis, before predictions

| | rung 64 | rung 65 |
|---|---|---|
| the valve | INSTANTANEOUS — a function of the state | LAGGED — a THIRD STATE |
| the ceiling belongs to | `b_max`, the AUTHORITY | authority **and** BANDWIDTH |
| two loops on one variable | the second's plant is DELETED (roundoff) | the second's SOLVE is repaired, the CONTINUUM survives as a marginal mode |

**Candidate headline, to be confirmed or refuted by § 2:** *a lag repairs the SOLVE without
removing the DEGENERACY — the redundancy of two loops on one variable is CONSERVED, moving out
of the solver and into the state.*

---

## § 1 — The derivation, before any further probe

**(a) The command does not read the live position.** `b_cmd` is rung 64's root over trial
positions at the current `(state, fuel)`; `_solve_b` forces each trial through `_b_forced` and
never consults `b`. So

    db/ds = (b_cmd(state, Wf) − b) / τ

is **affine in `b`** — Lipschitz with constant `1/τ`, no latch, and rung 47's hazard cannot
recur. Its two kinks (the dormant edge at `b_cmd = 0`, the saturation edge at `b_max`) are
kinks and not jumps: rung 52's argument for `AsymmetricLag`, one lever over.

**(b) The command is read at the APPLIED fuel, not the scheduled one.** A real valve watches
the machine it is on. Rung 52 computes its `required` off the SCHEDULED fuel to keep two legs'
brackets bit-identical; that reason does not transfer, because the valve is not min-selected
against anything. With no fuel leg armed the two are the same number — the choice is only
visible on the composite, and it is what makes § 0's probe B a statement about physics rather
than about bookkeeping.

**(c) The plant runs at the STATE and every STEADY solve runs the instantaneous law.** Inside a
march `_b_state` is set and the closure dispatches to rung 63's; outside it — `equilibrium`,
`fuel_for_Tt4`, the running line — the lag is meaningless and rung 64's root runs. See probe A.

**(d) A lagged position is not a function of the state, so it must be RECORDED.** This
CORRECTS a rung-64 code comment: there `b_at_point` re-solves the position exactly, because
the valve is a pure state function. Re-solving a lagged one would silently hand back the
COMMAND — the one number that is not the valve.

**(e) Rung 42's stops are hardware and clamp the STATE, never the command.** The state chases a
bounded command from a bounded start, so the clamp is inert while the command is interior; it
is not a solver tolerance.

---

## § 2 — The predictions, written before the sweeps

Grid: rung 64's, at the finding resolution `ds` = 0.005 unless stated; `τ` swept descending
(0.4, 0.2, 0.1, 0.05, 0.02, 0.01).

**P1 — the TRADE is monotone in both currencies, in OPPOSITE directions.** Across the whole
sweep at `ds` = 0.005: `undershoot` = `min φ_lp − φ_lim` is monotone (worse with larger `τ`)
AND `∫b ds` is monotone the other way. Probe C saw the sign at two points; this scores
monotonicity across six, at a different `ds`.

**P2 — the `τ → 0` arm of the reduce is FIRST ORDER.** `dev` = max |`φ_lp(τ)` − `φ_lp`(rung
64)| on the same grid shrinks with `τ`, and the ratio between consecutive halvings lands in
[1.6, 2.4] — a first-order approach to the instantaneous trajectory. (`tau=None` is the OTHER
arm and is bit-for-bit by dispatch; it is a gate, not a prediction.)

**P3 — rung 64 § 4's destroyed argmin is RESTORED at every `τ` in the sweep.** `plateau_pts`
== 1 at all six, including the smallest. **This is the prediction most likely to fail**: as
`τ → 0` the plateau must return, so if it fails, the `τ` at which it returns is itself the
answer to "how singular is the limit" and will be published as such.

**P4 — and the restored argmin is a RESULT, not a tie.** At `τ` = 0.05, halving `ds` from
0.005 to 0.0025 moves `s_at_min_lp` by ≤ 2 grid cells — against rung 64's factor-**3.3** move
under a 1e−15 perturbation of an untouched `b_max`.

**P5 — the marginal mode is a CONTINUUM, not a coincidence.** On the composite:
`frozen` (max |b − b(0)| over the march) ≤ 1e−9; `db_db0` = 1 to ≤ 1e−9 under a ±0.01
perturbation of the initial position; `laws_held` (max |`φ_lp` − `φ_lim`| where the leg rides)
≤ 1e−9 with the valve strictly interior; `dremoved` ≠ 0 — different members withhold different
fuel; and `tau_span_rel` ≤ 1e−9 across a 20× `τ` range.

**P6 — the DISCRIMINATOR, both halves.** At a state taken off an armed march: the
instantaneous plant's `φ_lp` span across a fuel bracket [0.90, 1.00]·`Wf` is < 1e−9 (rung 64's
DELETED plant, exhibited rather than derived), the lagged plant's is > 1e−3, monotone in fuel
with a sign change in `G` — rung 49's own premise, restored. Ratio > 1e6.

**P7 — the HP debit CHANGES SIGN with bandwidth.** Rung 64 measured that the closed loop
DEBITS the HP (−5.3e−4) while the state-blind constant valve CREDITS it (+2.3e−3), because a
constant valve is still open at the HP's own LATE minimum where the state-fed laws have shut.
A lagged valve is slower to shut, i.e. more constant-like. So `d_min_phi_hp` rises
monotonically with `τ` and **crosses zero inside the sweep**.

**P8 — a SATURATED lagged floor is a different object and its `plateau_pts` proves nothing.**
A floor above the fully-open march's own minimum commands `b_max` throughout, so under a lag it
is a bare exponential approach with no feedback content. Predict: its `min φ_lp` is strictly
WORSE than rung 64's over-set floor and falls monotonically with `τ`, while `plateau_pts` == 1
for a reason unrelated to tracking error. P3 must therefore be read on RIDING cells only, and
the gate must distinguish them or it passes for the wrong reason.

---

## § 3 — Scoring

**Six HIT, two REFUTED, and one § 0 pre-check RETRACTED.** All at `ds` = 0.005 on rung 64's
grid unless stated.

### RETRACTION — § 0 probe C's `τ` = 0.002 row was a NUMERICAL ARTIFACT

`db/ds = (b_cmd − b)/τ` under an explicit RK4 needs `z = ds/τ` inside the stability region
(|z| ≲ 2.78 on the negative real axis). Probe C ran `τ` = 0.002 at `ds` = 0.01, i.e. **z = 5**.
Measured, valve alone:

| `τ` | `ds` | `ds/τ` | `min φ_lp` | `∫b ds` |
|---|---|---|---|---|
| 0.002 | 0.01 | **5.00** | 0.800000000 | **0.169222** ← probe C's number, unstable |
| 0.002 | 0.005 | 2.50 | 0.799433994 | 0.038333 |
| 0.002 | 0.001 | 0.50 | 0.799334631 | 0.038325 |
| 0.2 | 0.01 | 0.05 | 0.776822104 | 0.045686 |
| 0.2 | 0.005 | 0.02 | 0.776806981 | 0.045687 |

Probe C's claim that a fast valve bleeds MORE is dead: the true `∫b` at `τ` = 0.002 is 0.0383,
converging on rung 64's instantaneous 0.038249. The whole scored sweep (`τ` ≥ 0.01 at
`ds` = 0.005, so `z` ≤ 0.5) is grid-converged — checked at `τ` = 0.01 against `ds` = 0.001.
**This is a modelling floor and is disclosed as one**: `τ` cannot be swept below ~`ds`/2.

The artifact is now **unreachable**: `_integrate_fuel_valve_lag` asserts `ds/τ ≤ 2.0` before
the march, and `tests/test_rung65.py` gate 7 pins the raise. The row above therefore cannot be
reproduced on the shipped plant — which is the point. It is recorded here because it looked
like a physical finding for the length of one probe, and a future rung adding a relaxation
state should expect the same trap.

### P1 — HIT, and it restores what probe C appeared to kill

| `τ` | `min φ_lp` | undershoot | `∫b ds` | `b` peak | `dev` |
|---|---|---|---|---|---|
| 0.4 | 0.770917491 | −2.908e−2 | 0.051785 | 0.06481 | 3.810e−2 |
| 0.2 | 0.776806981 | −2.319e−2 | 0.045687 | 0.07620 | 3.681e−2 |
| 0.1 | 0.783291203 | −1.671e−2 | 0.042005 | 0.08520 | 2.816e−2 |
| 0.05 | 0.789122771 | −1.088e−2 | 0.040132 | 0.08995 | 1.746e−2 |
| 0.02 | 0.794571011 | −5.429e−3 | 0.039003 | 0.09172 | 7.829e−3 |
| 0.01 | 0.797007245 | −2.993e−3 | 0.038626 | 0.09197 | 4.109e−3 |
| rung 64 | 0.800000000 | 0 (exact) | 0.038249 | 0.09202 | — |

Both currencies are monotone and they run the SAME way: **a slower valve protects LESS and
bleeds MORE.** It opens late (missing the minimum) and closes late (dumping air after the need
has passed), and the closing tail dominates the integral. So bandwidth is **PURE LOSS**, not a
trade — the prediction the § 0 artifact had appeared to refute.

### P2 — HIT on the shrink, CORRECTED on the order

`dev` shrinks monotonically. Consecutive-halving ratios: 1.035 (0.4→0.2), 1.307 (0.2→0.1),
1.613 (0.1→0.05), 1.905 (0.02→0.01) — approaching 2 from below. The prediction's [1.6, 2.4]
band was written as if first order held across the whole sweep; it does not, because **`dev`
SATURATES at large `τ`**: it is bounded above by the valve-shut march's own deficit
(0.800000 − 0.735442 = 6.46e−2), so it cannot keep doubling. First order is the `τ → 0`
asymptote and the sweep shows the approach to it.

### P3 — HIT, by two orders

`plateau_pts` == 1 at **every** `τ`, `plateau_span` == 0 exactly, against rung 64's **114** on
the same grid. And rung 64's plateau is a genuine interval rather than a tie: it scales with
the grid — 57 / 114 / 227 at `ds` = 0.01 / 0.005 / 0.0025, exactly ∝ 1/`ds` — while the lagged
march stays at 1 on all three.

### P4 — HIT, and the side-by-side is the finding

`s_at_min_lp` at `τ` = 0.05 against rung 64's, over a 4× refinement:

| `ds` | lagged `s_at_min` | lagged `min φ_lp` | rung 64 `s_at_min` | rung 64 `plateau_pts` |
|---|---|---|---|---|
| 0.01 | 0.0900 | 0.789129958 | 0.3800 | 57 |
| 0.005 | 0.0850 | 0.789122771 | 0.3000 | 114 |
| 0.0025 | 0.0875 | 0.789121707 | 0.0675 | 227 |

The lagged argmin moves by 2 cells at the finest grid and its value converges to 6 figures;
rung 64's moves by a factor **5.6**. Rung 64 § 4's destroyed object is back.

### P5 — HIT, and SHARPENED: the continuum has an EDGE, and it is the valve's own law

On the composite at `τ` = 0.05, sweeping the initial position:

| `b0` | `b0`/`b_cmd(0)` | `b_end` | drift | `fuel_removed` | frozen? |
|---|---|---|---|---|---|
| 0.018313 | 0.500 | 0.0183131818 | 2.4e−16 | 1.325704e−2 | **yes** |
| 0.026626 | 0.727 | 0.0266263637 | 3.3e−16 | 1.735602e−2 | **yes** |
| 0.036260 | 0.990 | 0.0362601000 | 6.5e−16 | 2.069827e−2 | **yes** |
| **0.036626** | **1.000** | 0.0366263637 | 6.0e−16 | 2.024341e−2 | **yes** ← the physical IC |
| 0.036993 | 1.010 | 0.0369804185 | 1.2e−05 | 1.976358e−2 | no |
| 0.038458 | 1.050 | 0.0383966379 | 6.1e−05 | 1.721558e−2 | no |
| 0.043952 | 1.200 | 0.0000047934 | 4.4e−02 | 9.325988e−3 | no |

Every member with `b0 ≤ b_cmd(0)` is **exactly frozen** for the whole march, holds
`φ_lp = φ_lim` to 1.3e−15 with the valve strictly interior, and withholds a DIFFERENT amount of
fuel — 1.33e−2 to 2.02e−2, a 53 % spread across THIS probe's endpoints (the natural member is
the top one here). The spec's canonical pair is § 3's tabulated extremes, 1.326e−2 … 2.070e−2,
a 56 % spread; the gate pins neither, only a one-sided ratio on the ±0.01 pair. `τ`-invariance
across a 20× range: 1.03e−15 relative.

**The upper edge is exactly `b_cmd(0)`, and it is derivable.** The valve's law is *the SMALLEST
position holding the floor*. Above `b_cmd(0)` the valve is doing more than its own law asks, so
its command sits below the live position and it closes; at or below it, the fuel leg takes up
the slack and the pair is exactly redundant. So the family is `b0 ∈ (0, b_cmd(0)]` — and the
physical initial condition sits precisely ON its upper edge, which is why the natural march
looked like a unique solution.

### P6 — HIT, by twelve orders

At `s` = 0.085, `b` = 0.055045, sweeping the fuel bracket [0.90, 1.00]·`Wf`:

| plant | `φ_lp` across the bracket | span |
|---|---|---|
| INSTANTANEOUS | 0.800000000 ×5 | **1.78e−15** |
| LAGGED | 0.789123 → 0.790477 → 0.791837 → 0.795949 → 0.802915 | **1.379e−2** |

Ratio 7.76e12, the lagged plant monotone in fuel with a strict sign change in `G`. Rung 64's
deletion is EXHIBITED (it was only derived there), and rung 49's own premise — "φ falls
monotonically with fuel at fixed spool speeds" — is restored verbatim.

### P7 — REFUTED. The HP debit is NON-MONOTONE in bandwidth and never becomes a credit

| `τ` | 0.4 | 0.2 | 0.1 | 0.05 | 0.02 | 0.01 | rung 64 |
|---|---|---|---|---|---|---|---|
| Δ`min φ_hp` | −5.62e−4 | −6.77e−4 | −7.19e−4 | −6.76e−4 | −6.01e−4 | −5.67e−4 | −5.31e−4 |

An **interior worst case near `τ` ≈ 0.1**, no sign change, always a debit. Grid-independent:
≤ 0.32 % across a 2× refinement. The predicted mechanism — "a lagged valve is slower to shut,
so it is more like rung 42's constant valve, which CREDITS the HP" — is wrong, and the reason
is worth keeping: **a lagged valve is not a slow approximation to a constant one.** It is late
at BOTH edges, so it arrives at the HP's own late minimum having opened less far than a
constant valve ever did. Lateness is not persistence.

### P8 — REFUTED in the direction that CONFIRMS RUNG 64

A floor set 10 % above the fully-open march's minimum (`φ_lim` = 0.890467):

| law | `min φ_lp` | deficit | `∫b ds` | `b` peak | `plateau_pts` |
|---|---|---|---|---|---|
| rung 64 | 0.807362922 | −8.310e−2 | 0.075003 | 0.100000 | 1 |
| `τ` = 0.01 | 0.807362922 | −8.310e−2 | 0.075985 | 0.100000 | 1 |
| `τ` = 0.05 | 0.807362922 | −8.310e−2 | 0.079937 | 0.100000 | 1 |
| `τ` = 0.2 | 0.807362922 | −8.310e−2 | 0.094678 | 0.100000 | 1 |

Predicted "strictly worse, monotone in `τ`". Measured: **`τ`-INVARIANT to nine digits** on the
protected coordinate, while `∫b` still pays the pure-loss bill. Where the valve is against its
stop, bandwidth is exactly as powerless as law was — which is rung 64's headline (*the ceiling
is the AUTHORITY*) found in a second place. The methodological half of P8 stands and matters:
`plateau_pts` == 1 for the instantaneous saturated cell too, so a saturated cell **cannot**
support P3 and the gate must read P3 on riding cells only.

### The headline, after scoring

> **A lag repairs the SOLVE without removing the DEGENERACY.** Two loops on one variable are
> redundant, and the redundancy is CONSERVED: rung 64's instantaneous valve hid it in a
> solver, where it was a roundoff coin flip; a finite bandwidth moves it into the STATE, where
> it is a marginal mode — exactly frozen, `τ`-invariant to 1e−15, a genuine one-parameter
> family bounded above by the valve's own minimality law, and selected by the initial
> condition alone.

with the two corollaries the sweeps establish: rung 64's ceiling gains **BANDWIDTH** as a
second hardware axis and it is **pure loss** (P1); and rung 64 § 4's destroyed argmin is
**RESTORED** at any finite bandwidth (P3/P4) — while at the stop, bandwidth buys nothing at
all (P8).
