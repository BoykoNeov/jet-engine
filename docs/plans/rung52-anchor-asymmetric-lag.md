# Rung 52 anchor — the asymmetric fast-attack / slow-release LAG: predictions, and how they scored

Rung 51 shipped the release RATE `τ_rel` as a **stateless linear fade** and named its own next
seam in the Concessions:

> *"The asymmetric fast-attack / slow-release LAG is the next seam, deferred for the three
> reasons in § The instrument. It is the physically-realisable version and its release edge is
> **emergent**, which is exactly why it cannot be the instrument that establishes the effect."*

This rung builds that seam — and **the first thing it does is check rung 51's stated reason,
because that reason is a factual claim about a system rung 51 never ran.**

## The three deferral reasons, restated (rung 51 § The instrument), and this rung's answer

| # | Rung 51's reason | Rung 52's answer |
|---|---|---|
| 1 | *"A lag's release edge is EMERGENT. Sweep its time constant and the release time moves with it — reinstating exactly the confound `s_off` was built to kill."* | **FALSE.** See P1 below. `τ_rel` is never *read* while `required > g`, so the leg **pins its own trigger** and the pre-crossing march is bit-identical across a `τ_rel` sweep. |
| 2 | *"A lag needs `max(g, required)` inside the derivative at a **state-dependent** location — materially worse than rung 50's fixed, grid-aligned step."* | **Form-dependent, and rung 51 picked the bad form.** An asymmetric-**rate** lag switches on `sign(required − g)`, and both branches → 0 as `required → g`: the RHS is **continuous** — a KINK, not a jump. Lipschitz in `g` ⇒ unique solution, RK4 converges with locally reduced order. Rung 47's latch hazard does not recur. |
| 3 | *"It is finite-time... an exponential never completes, so 'the release edge' stops being a locatable object."* | **STANDS, and is answered by a declared definition, not by a redesign.** The release edge is defined *fractional-of-schedule*, `(mf_sched − mf)/mf_sched < ε`, matching `release_relief`'s existing `deficit_at_release` currency; every debit is reported at **two** ε (0.05 and 0.01) so no verdict rests on the threshold. |

## The instrument (fixed before any measurement)

```
    required(ν, s) = max(0, mf_sched(s) − surge_cap(ν, mf_sched))
    dg/ds = (required − g) / τ_att      if required > g       (fast ATTACK)
            (required − g) / τ_rel      if required < g       (slow RELEASE)
    mf    = mf_sched − g
```

The clip AMOUNT `g` is a third state, exactly rung 47's `_integrate_fuel_lagged` pattern, moved
onto rungs 48/49's legs and given **two** constants instead of one. `lag=None` never enters this
branch ⇒ **exact-dispatch** reduce to rungs 49/50/51, rung 47's own contract.

**No new constant.** `τ_att` and `τ_rel` are swept coordinates, like `s_off` and `τ_rel` before
them. `φ_lim` (rungs 36/41/49) and `m` (rung 48) are inherited with their disclaimers intact.

## Config (rung 50/51's, unchanged)

CPG gas, `FLIGHT(250 K, 50 kPa, M0=0.85)`, `π_LPC/π_HPC/Tt4 = 3/6/1500`, shaped LP/HP maps,
accel 1000 → 1400 K, `ρ`=1, `s_settle`=4. LP-watching floors `φ_lim` = 0.7450 (`r`=0.5) and
0.7725 (`r`=2.0). Bare minima: `r`=0.5 → `s_lp*`=0.24, `s_hp*`=0.40; `r`=2.0 → 0.32, 0.64.

---

# THE PREDICTIONS, as written before measuring — and how they scored

### P1 — *"the release trigger is set by the plant and the attack transient; `τ_rel` may move it only second-order"* (mine, written before probe 1)

**CORRECT IN DIRECTION, UNDERSTATED IN KIND — and the understatement is the finding.** It is not
second-order; it is **exactly zero, and structurally so**. The advisor had framed two sub-cases
(`g` still climbing toward a turning `required` ⇒ deconfounded; `g` caught up and tracking ⇒
the separation rate *is* `τ_rel`) and asked which one the plant sits in. Neither is the reason.
The reason needs no plant knowledge at all: **`τ_rel` is not read before the first crossing**, so
the entire pre-crossing trajectory is bit-identical no matter what the plant does.

Measured (r=2.0, φ_lim=0.7725), `s_CROSS` across `τ_rel` ∈ {0.02, 0.10, 0.40}:

| τ_att | s_CROSS | g@cr | s_eng(ε=.05) | credit `rel_lp` | credit spread over τ_rel |
|---|---|---|---|---|---|
|0.02|1.14 (×3)|0.00495|0.12|+0.01008762|**0.00e+00**|
|0.05|0.94 (×3)|0.00341|0.16|+0.00837440|**0.00e+00**|
|0.10|0.82 (×3)|0.00237|0.20|+0.00645431|**0.00e+00**|
|0.20|0.74 (×3)|0.00153|0.28|+0.00437907|**0.00e+00**|
|0.40|0.70 (×3)|0.00091|0.42|+0.00264351|**0.00e+00**|

`n_recross = 1` in every row — a single crossing, so the self-pinning is exact here rather than
merely first-crossing-exact.

**Recorded honestly: the numerics CORROBORATE a one-line structural argument; they do not carry
it.** The 136 s of sweeping was worth it only because it also produced the factorization grid —
P1 itself was decidable at the desk, and rung 51 got it wrong by not doing so.

### P2 — *"one row will break the credit's zero: a shallower floor, or the r=0.5 config, will put the crossing upstream of the watched spool's minimum"* (the advisor's, written before gate B)

**FALSIFIED. No such row exists in this family, and there is a mechanism for why.**

Searched: 7 floors × 2 ramp rates, plus all 5 `τ_att` of P1. `s_min_lp < s_CROSS` in **every**
case; credit spread `0.00e+00` in **every** case.

| r | φ_lim | s_CROSS | s_min_lp | credit spread |
|---|---|---|---|---|
|2.0|0.7625|0.38|0.24|0.00e+00|
|2.0|0.7650|0.46|0.18|0.00e+00|
|2.0|0.7675|0.58|0.16|0.00e+00|
|2.0|0.7725|1.14|0.10|0.00e+00|
|0.5|0.7450|0.32|0.16|0.00e+00|
|0.5|0.7460|0.34|0.16|0.00e+00|
|0.5|0.7480|0.36|0.16|0.00e+00|

**The mechanism — two steps, and the second is the content.**

1. `τ_rel` is never read while `required > g` ⇒ nothing upstream of the crossing can move.
   *Unconditional, structural, one line — and by itself a tautology.*
2. **The watched spool's own minimum is upstream of the crossing.** Two different turnarounds
   are in play and they are *not* the same object: `required`'s, and the actual `φ_lp`'s — under
   a lag `φ_lp` dips **below** `φ_lim` (gate C: 0.7395 against a 0.7450 floor at τ=0.08), so the
   claim needs the actual one. It holds because **the lag's undershoot is largest early**: while
   `g` is still climbing the clip has not caught up, so `φ_lp`'s deepest excursion sits near
   `s_eng` by construction — rung 48's arrest law operating through the lag's own attack
   transient.

Composed: **a self-releasing limiter releases only after the watched variable has begun to
recover, and its own attack transient has already pinned that spool's minimum at the engagement
edge. It therefore CANNOT debit the spool it watches.**

### P3 — the factorization question (framed before the grid, scored after; the *magnitudes* are exploratory)

Framed as: *does rung 49's credit/debit split factor across `(τ_att, τ_rel)`* — the premise a
real fast-attack/slow-release limiter is designed on. **Answer: one way only.**

Additive residual on the debit, `D(ta,tr) − D(ta,tr0) − D(ta0,tr) + D(ta0,tr0)`:

| r=2.0, τ_att \ τ_rel | 0.020 | 0.100 | 0.400 |   | r=0.5, τ_att \ τ_rel | 0.010 | 0.040 | 0.160 |
|---|---|---|---|---|---|---|---|---|
|0.020|+0.000000|+0.000000|+0.000000| |0.020|+0.000000|+0.000000|+0.000000|
|0.050|+0.000000|−0.002251|−0.005537| |0.080|+0.000000|−0.000750|−0.002085|
|0.100|+0.000000|−0.004157|−0.009926| |0.320|+0.000000|−0.001563|−0.004896|
|0.200|+0.000000|−0.005849|−0.013865| | | | | |
|0.400|+0.000000|−0.007036|−0.016956| | | | | |

Residual −0.0170 against main effects of −0.0274 at r=2.0 (62%); −0.0049 against −0.0070 at
r=0.5 (70%). **Same order as the main effects, at both ramp rates.** Not multiplicative either:
`D(ta,·)` ratios drift 0.636 → 0.475 and then change sign.

### P4 — rung 51's rate verdict on a realisable leg (a transfer test, not a new prediction)

Slower hand-back ⇒ **shallower** debit, monotone over the whole 20× `τ_rel` range at both ramp
rates, with the anti-deflation pair intact — `fuel_removed` **RISES** (0.00500 → 0.00724) while
the debit **SHRINKS** (−0.0274 → −0.0038). More fuel removed, smaller debit: the debit is not a
function of the total deficit. That is rung 51's headline, reproduced on a physically-realisable
leg, and here it goes **further** — the debit crosses zero into a **CREDIT**.

---

# THE GATES

### Gate A — anti-degeneracy on the sign flip (rungs 49/50's `nu_hp_end` discipline)

The sign flip is the most quotable number in the grid and it sits where the leg engages least.
bare `nu_hp_end` = 0.95906392 (r=2.0):

| τ_att | τ_rel | rel_hp | removed | nu_hp_end | Δ |
|---|---|---|---|---|---|
|0.02|0.02|−0.027354|0.00500|0.95906287|−1.06e-06|
|0.02|0.40|−0.003815|0.00724|0.95905473|−9.19e-06|
|0.20|0.40|+0.000299|0.00146|0.95906327|−6.48e-07|
|0.40|0.02|−0.005552|0.00042|0.95906386|−5.73e-08|
|0.40|0.40|**+0.001031**|0.00078|0.95906362|**−3.05e-07**|

**PASSES. The flip is real.** Every Δ ≤ 1e-5 *relative*; Δ is monotone in `fuel_removed`; the
corner rows are the *least* perturbed of all. No row is near incompletion. (An earlier probe
label called the deep rows "degenerate" off an arbitrary 1e-6 *absolute* cut — a mislabel. Rung
49/50's discipline is about the accel *failing to complete*, which would be orders larger.)

### Gate B — `ds`-stability of `s_CROSS` (this one underwrites every invariance number above)

r=2.0, φ_lim=0.7725, τ_att=0.02, τ_rel=0.10:

| ds | s_CROSS | g@cr | min φ_lp | min φ_hp |
|---|---|---|---|---|
|0.0400|1.160|0.004964|0.77106881|0.89330383|
|0.0200|1.140|0.004957|0.77102559|0.89328664|
|0.0100|1.130|0.004958|0.77102387|0.89322924|
|0.0050|1.125|0.004958|0.77102376|0.89322930|

**PASSES.** `s_CROSS` moves by **exactly one grid cell per halving** (0.020, 0.010, 0.005) —
the resolution limit of "first recorded point with `required < g`", not motion of the crossing;
geometric convergence to ≈1.120. `g@cr` stable to 6e-6 from ds=0.02 down. Reliefs ds-converged:
min φ_lp to ~1e-7 by ds=0.01, min φ_hp agreeing to 6e-8 between the last two. **The kink is not
being resolved differently at different resolutions.**

### Gate C — the instantaneous limit (`τ → 0` approaches rung 49's min-select)

ds held **FIXED** at 0.0025 and τ varied **alone** (a first pass halved τ and ds together and
therefore measured neither limit — superseded). r=0.5, φ_lim=0.7450; rung-49 reference
min φ_lp = 0.74500000 (= `φ_lim` exactly, the sliding mode), min φ_hp = 0.85141792:

| τ | min φ_lp | Δ_lp | ratio | min φ_hp | Δ_hp | ratio |
|---|---|---|---|---|---|---|
|0.080|0.73953722|−5.463e-03|—|0.86044099|+9.023e-03|—|
|0.040|0.74118344|−3.817e-03|0.699|0.85754502|+6.127e-03|0.679|
|0.020|0.74261652|−2.383e-03|0.625|0.85503439|+3.616e-03|0.590|
|0.010|0.74363438|−1.366e-03|0.573|0.85336914|+1.951e-03|0.540|

**PASSES as an APPROACH, and the order is reported as measured, not as hoped.** Monotone from
both sides (undershoot on the watched spool — a lag cannot hold a floor instantaneously;
overshoot on the other). Ratios trend toward 0.5 but do not reach it over this range: observed
order ≈ **0.8, sub-first-order**. What this gate exists to rule out — a structural mismatch
between `required` and `_surge_fuel`'s min-select — is ruled out.

---

# What this rung claims, in order of strength

1. **Rung 51's deferral reason 1 is FALSE.** A self-releasing leg **pins its own trigger**;
   everything upstream of its crossing is bit-identical across a release-rate sweep — the exact
   property `s_off` was built to *force*. The asymmetric lag is a legitimate rate instrument in
   its own right, and `s_off` is retrospectively confirmed as an *isolation diagnostic* rather
   than the only way to get a pinned release.
2. **A self-releasing limiter CANNOT debit the spool it watches** (P2). This **BOUNDS** rung
   50's "an early release DEBITS the spool it watches" to **forced** releases, and thereby
   **RESTORES** rung 49's watched-side identity — which rung 50 had bounded — for every
   physically-realisable leg.
3. **The two clocks separate ONE WAY.** `τ_att` owns the credit *exactly* (machine-zero over
   5 × 3 × 2 configs); the debit is irreducibly joint (interaction 62–70% of the main effects at
   both ramp rates). The fast-attack/slow-release design premise is **half true — and the half
   that fails is the protective one.**
4. **Rung 51's rate verdict transfers and strengthens** (P4): monotone over 20×, anti-deflation
   pair intact, and the debit **crosses zero into a credit**.

# Honest scope, written with the findings

- **`φ_lim` and `m` inherit rungs 36/41/48/49's imposed constants.** Magnitudes disclaimed;
  signs, orderings, machine-zeros and the ds/τ convergences are the claims.
- **Claim 1's numerics corroborate an argument; they do not carry it.** Stated so a reader is
  not sold a sweep as a discovery.
- **Claim 2 is a negative search plus a mechanism, not a theorem.** 7 floors × 2 ramp rates × 5
  attack constants with no exception, and an argument for why — but no proof that no plant
  admits a counter-case.
- **The self-pinning is exact for the FIRST crossing.** A leg that re-engages (`n_recross > 1`)
  would have later crossings that *are* `τ_rel`-dependent. Not observed in any row here.
- **The lag is LINEAR and first-order in each direction.** No shape sweep; nothing is claimed
  about the hand-back's functional form beyond attack-rate ≠ release-rate.
- **Gate C's observed order is ≈0.8, not 1**, and is reported as such.
- The plant is rung 43's non-equilibrium (CPG/TPG) gas — rung 35's standing concession.
