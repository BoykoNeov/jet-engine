# Rung 52 — The asymmetric fast-attack / slow-release LAG: a self-releasing limiter pins its own trigger, and cannot debit the spool it watches

Rung 50 built a **forced** release time `s_off`; rung 51 gave that forced release a **rate**
`τ_rel` as a stateless linear fade. Both were explicitly *isolation diagnostics, not control
laws* — instruments in the tradition of rung 34/40's `freeze=` and rung 41's
`surge_margin_channels`. Both existed for one reason, stated in rung 50: **rung 49's family
could not pin a release edge**, so moving one meant moving `φ_lim`/`m`, which drags the
engagement edge, the window length and the clip depth along with it (rung 49 § 3's
within-family hedge).

Rung 51 named the physically-realisable version as its own next seam and **deferred it with
three reasons**. This rung builds it — and the first thing it does is *check those reasons*,
because they are factual claims about a system rung 51 never ran.

**Two of the three fall.** The rung's content follows from the one that falls hardest.

---

## The instrument — `AsymmetricLag`, the clip amount as a state with TWO constants

```
    required(ν, s) = max(0, mf_sched(s) − min(armed leg caps at mf_sched))
    dg/ds = (required − g) / τ_att      if required > g          (fast ATTACK)
            (required − g) / τ_rel      if required < g          (slow RELEASE)
    mf    = mf_sched − g
```

`g` — the clip **amount**, not the valve position — is a **third state**: exactly rung 47's
`_integrate_fuel_lagged` pattern, moved onto rungs 48/49's legs and given **two** constants
instead of one. This is how a real fuel limiter is built, and *why*: cut hard to protect, hand
back gently so the recovery does not re-excite the thing you were protecting.

`required` is computed from the **scheduled** fuel, never the clipped value, so each leg's cap
is solved off the same bracket rungs 48/49 use, and a dormant leg returns `mf_sched` itself.

**No new constant.** `τ_att` and `τ_rel` are swept coordinates, like `s_off` and rung 51's
`τ_rel` before them. `φ_lim` (rungs 36/41/49) and `m` (rung 48) are inherited with their
disclaimers intact.

### Rung 51's three deferral reasons, and what happened to each

**(1) *"A lag's release edge is EMERGENT. Sweep its time constant and the release time moves
with it — reinstating exactly the confound `s_off` was built to kill."* — FALSE.**

`τ_rel` is **never read** while `required > g`. So the entire march up to the first crossing
is **bit-identical** across a `τ_rel` sweep, and the crossing — the leg's own release trigger —
cannot move. **The leg pins its own trigger.** That is the property rung 50 had to *force*, and
a realisable limiter has it for free.

This is a one-line structural argument that needs no knowledge of the plant, and it was
available to rung 51 at the desk. The measurements below **corroborate** it; they do not carry
it, and the spec says so rather than selling a sweep as a discovery.

**(2) *"It needs `max(g, required)` inside the derivative at a state-dependent location —
materially worse than rung 50's fixed, grid-aligned step."* — FORM-DEPENDENT, and rung 51
named the bad form.**

An asymmetric-**rate** lag switches on `sign(required − g)`, and **both branches carry the same
`(required − g)` numerator, which vanishes at the switch**. The RHS is therefore **continuous** —
a **kink, not a jump** — hence Lipschitz in `g`, hence a unique solution and a convergent RK4
with locally reduced order at the crossing cell. Rung 47's latch hazard (a boolean flipping
between k1 and k4, silently destroying the integrator's order) **does not recur**, and it is
the *rate* form, not the *level* form rung 51 sketched, that buys this.

**(3) *"It is finite-time... an exponential never completes, so 'the release edge' stops being
a locatable object."* — STANDS.**

Answered by **declaring** an edge rather than redesigning the instrument: *fractional-of-
schedule*, `(mf_sched − mf)/mf_sched ≥ ε`, the currency `release_relief.deficit_at_release`
already uses. Every debit is reported at **two** ε (0.05 and 0.01) so no verdict rests on a
threshold — the analogue of rungs 50/51's `ds`-convergence discipline.

### Why it asserts against `s_off`/`tau_rel`, and against `tau_gov`

Forcing a release on a leg whose clip is already a **state** would have to *zero that state* —
a third instrument. Rung 50's own assertion text already refuses exactly this for the rung-46/47
governor, and the argument carries verbatim. Two clip-amount states on two legs (`tau_gov` +
`lag`) is a two-lag cascade, not this rung. An **instantaneous** topping governor (`Tt4_max`
alone) composes fine and stays unlagged, exactly as `s_off` leaves the redline alone.

---

## THE FINDINGS

Config: rung 49/50/51's, unchanged — CPG gas, `FLIGHT(250 K, 50 kPa, M0=0.85)`,
`π_LPC/π_HPC/Tt4 = 3/6/1500`, shaped LP/HP maps, accel 1000→1400 K, `ρ`=1, `s_settle`=4,
LP-watching floors `φ_lim` = 0.7450 (`r`=0.5) / 0.7725 (`r`=2.0). Bare minima `s_lp*`/`s_hp*` =
0.24/0.40 (`r`=0.5) and 0.32/0.64 (`r`=2.0). `tests/test_rung52.py` reproduces; every transcript
is in `docs/plans/rung52-anchor-asymmetric-lag.md`.

### 1. THE TRIGGER PINS ITSELF — measured (`r`=2.0, `φ_lim`=0.7725, `τ_rel` ∈ {0.02, 0.10, 0.40})

| τ_att | `s_cross` | `g` at crossing | `s_eng`(ε=.05) | `relief_watched` | **spread over τ_rel** |
|---|---|---|---|---|---|
|0.02|1.14 (×3)|0.00495|0.12|+0.01008762|**0.00e+00**|
|0.05|0.94 (×3)|0.00341|0.16|+0.00837440|**0.00e+00**|
|0.10|0.82 (×3)|0.00237|0.20|+0.00645431|**0.00e+00**|
|0.20|0.74 (×3)|0.00153|0.28|+0.00437907|**0.00e+00**|
|0.40|0.70 (×3)|0.00091|0.42|+0.00264351|**0.00e+00**|

`n_recross = 1` everywhere, so the self-pinning is exact here and not merely
first-crossing-exact. **Rung 51's reason 1 is refuted, and `s_off` is retrospectively confirmed
as an isolation diagnostic rather than the only way to obtain a pinned release.**

**Where the bit-identity actually stops — measured, not assumed.** Strictly, it holds up to the
RK4 step that *straddles* the crossing: that step's later sub-stages already have
`required < g`, so they read `τ_rel`, and the crossing is *recorded* one grid point downstream.
Hence `s_cross`, `s_eng` and `relief_watched` are **exact** (the first two are grid coordinates;
the third because the watched minimum lies strictly upstream of the straddling step), while
`g` at the crossing carries a partial-step residual — **~4e-4 relative**, visible as the
0.00495 / 0.00496 split in the table above. That is the integrator's granularity, not a
weakness in the argument.

The `τ_att` column is rung 48's engagement-time law in realisable clothing: a slower attack
engages **later** (`s_eng` walks 0.12 → 0.42) and credits **less** (+0.0101 → +0.0026).

### 2. A SELF-RELEASING LIMITER CANNOT DEBIT THE SPOOL IT WATCHES — and this is *not* the tautology

The exact zero in the last column has **two** steps, and only the second is content.

1. `τ_rel` is not read before the crossing ⇒ nothing upstream of it can move. *Unconditional,
   structural — and by itself a tautology.*
2. **The watched spool's own minimum is upstream of the crossing.** This is the step that could
   have failed. Note it needs the **actual** `φ_lp` minimum, not `required`'s turnover — they
   are different objects, because under a lag `φ_lp` dips **below** `φ_lim` (§ 5: 0.7395 against
   a 0.7450 floor). It holds because **the lag's undershoot is largest EARLY**: while `g` is
   still climbing the clip has not caught up, so `φ_lp`'s deepest excursion sits near `s_eng` by
   construction — **rung 48's arrest law operating through the lag's own attack transient**.

   **Precondition — this step is a property of a FIRST-ORDER attack.** "Undershoot largest
   early" follows from `ġ = (required − g)/τ_att` starting at `g=0`; a *second-order* or
   rate-limited attack can delay the undershoot and so relocate the watched spool's minimum
   **off** `s_eng`. The CANNOT-debit result is therefore claimed for a first-order attack only —
   a delayed-undershoot shape is **not covered**, and is exactly this rung's named next seam.

Searched for a counter-case across **7 floors × 2 ramp rates**, plus all 5 `τ_att` of § 1:

| r | φ_lim | `s_cross` | `s_min_lp` | credit spread |
|---|---|---|---|---|
|2.0|0.7625|0.38|0.24|0.00e+00|
|2.0|0.7650|0.46|0.18|0.00e+00|
|2.0|0.7675|0.58|0.16|0.00e+00|
|2.0|0.7725|1.14|0.10|0.00e+00|
|0.5|0.7450|0.32|0.16|0.00e+00|
|0.5|0.7460|0.34|0.16|0.00e+00|
|0.5|0.7480|0.36|0.16|0.00e+00|

`s_min_lp < s_cross` without exception. Composed with step 1:

> **A self-releasing leg releases only after the watched variable has begun to recover, and its
> own attack transient has already pinned that spool's minimum at the engagement edge. It
> therefore cannot debit the spool it watches.**

**The cross-rung payoff.** Rung 50 found that an **early forced** release *debits the spool it
watches*, and **BOUNDED** rung 49's watched-side identity on that basis. This rung shows that
result is an **artifact of forcing**: no realisable self-releasing leg can produce it. Rung 49's
identity is **RESTORED for every physically-realisable leg**, and rung 50's bound is re-scoped
to the artificial instrument that produced it — which is also *why* rung 50 needed an artificial
instrument to get there.

### 3. THE TWO CLOCKS SEPARATE ONE WAY — the design premise is HALF TRUE

A fast-attack/slow-release limiter is *designed* on the premise that the two constants tune
independently. This is the first instrument on which rung 49's two clocks are **independently
dialable on a single realisable leg**, so the premise becomes testable.

Additive-separability residual on the **debit**,
`D(ta,tr) − D(ta,tr₀) − D(ta₀,tr) + D(ta₀,tr₀)`:

| r=2.0 τ_att \ τ_rel | 0.020 | 0.100 | 0.400 |
|---|---|---|---|
|0.020|+0.000000|+0.000000|+0.000000|
|0.050|+0.000000|−0.002251|−0.005537|
|0.100|+0.000000|−0.004157|−0.009926|
|0.200|+0.000000|−0.005849|−0.013865|
|0.400|+0.000000|−0.007036|−0.016956|

| r=0.5 τ_att \ τ_rel | 0.010 | 0.040 | 0.160 |
|---|---|---|---|
|0.020|+0.000000|+0.000000|+0.000000|
|0.080|+0.000000|−0.000750|−0.002085|
|0.320|+0.000000|−0.001563|−0.004896|

**The same order as the main effects, at both ramp rates.** Against the largest single-axis
main-effect deviation (`max_main_effect`, the quantity the gate compares to): **0.0139 vs
0.0235 → 59 %** on the 2×3 corner at `r`=2.0, **0.0049 vs 0.0070 → 70 %** at `r`=0.5. Against
the deepest debit in the full 5×3 grid, the residual reaches **0.0170 of 0.0274 → 62 %**. Not
multiplicatively separable either: the `τ_rel` ratios drift 0.636 → 0.618 → 0.600 → 0.562 →
0.475 and then change **sign**.

Against that, `credit_spread` is **machine zero** at every `τ_att`, at both ramp rates.

> **`τ_att` owns the credit EXACTLY. The debit is irreducibly JOINT. The premise is half true —
> and the half that fails is the PROTECTIVE one:** a release rate cannot be chosen for the
> unwatched spool's benefit without knowing the attack constant.

### 4. RUNG 51'S RATE VERDICT TRANSFERS, AND STRENGTHENS

Slower hand-back ⇒ **shallower** debit, **monotone over the whole 20× `τ_rel` range at both
ramp rates**, with the anti-deflation pair intact — `fuel_removed` **RISES** (0.00500 → 0.00724)
while the debit **SHRINKS** (−0.027354 → −0.003815). More fuel removed, smaller debit: the debit
is not a function of the total deficit. That is rung 51's headline, on a realisable leg — and
here it goes further, **crossing zero into a CREDIT**.

**The sign flip, with its anti-degeneracy pair** (rungs 49/50's discipline — the flip sits where
the leg engages *least*, so the accel must be shown to complete there). bare
`nu_hp_end` = 0.95906392:

| τ_att | τ_rel | `relief_other` | `fuel_removed` | `nu_hp_end` | Δ |
|---|---|---|---|---|---|
|0.02|0.02|−0.027354|0.00500|0.95906287|−1.06e-06|
|0.02|0.40|−0.003815|0.00724|0.95905473|−9.19e-06|
|0.20|0.40|+0.000299|0.00146|0.95906327|−6.48e-07|
|0.40|0.02|−0.005552|0.00042|0.95906386|−5.73e-08|
|0.40|0.40|**+0.001031**|0.00078|0.95906362|**−3.05e-07**|

Every Δ ≤ **1e-5 relative**, monotone in `fuel_removed`, and the corner rows are the **least**
perturbed of all. No row is near incompletion. **The flip is real.** At `r`=0.5 it is broader
still — it appears at *every* `τ_att` including the fast-attack row.

### 5. `ds`-STABILITY AND THE INSTANTANEOUS LIMIT

**`ds`-stability of the crossing** (`r`=2.0, τ_att=0.02, τ_rel=0.10). This gate underwrites
every invariance number above: if the kink were resolved differently at different resolutions,
all of them would inherit it.

| ds | `s_cross` | `g` at crossing | min φ_lp | min φ_hp |
|---|---|---|---|---|
|0.0400|1.160|0.004964|0.77106881|0.89330383|
|0.0200|1.140|0.004957|0.77102559|0.89328664|
|0.0100|1.130|0.004958|0.77102387|0.89322924|
|0.0050|1.125|0.004958|0.77102376|0.89322930|

`s_cross` moves by **exactly one grid cell per halving** (0.020, 0.010, 0.005) — the resolution
limit of "first recorded point with `required < g`", not motion of the crossing; geometric
convergence to ≈1.120. Reliefs are converged: min φ_lp to ~1e-7 by `ds`=0.01, min φ_hp agreeing
to 6e-8 between the last two.

**The instantaneous limit** — `ds` held **FIXED** at 0.0025 while τ varies **alone** (a first
pass halved both together and therefore measured neither limit). `r`=0.5, `φ_lim`=0.7450;
rung-49 reference min φ_lp = 0.74500000 (= `φ_lim` exactly — the sliding mode), min φ_hp =
0.85141792:

| τ (=τ_att=τ_rel) | min φ_lp | Δ | ratio | min φ_hp | Δ | ratio |
|---|---|---|---|---|---|---|
|0.080|0.73953722|−5.463e-03|—|0.86044099|+9.023e-03|—|
|0.040|0.74118344|−3.817e-03|0.699|0.85754502|+6.127e-03|0.679|
|0.020|0.74261652|−2.383e-03|0.625|0.85503439|+3.616e-03|0.590|
|0.010|0.74363438|−1.366e-03|0.573|0.85336914|+1.951e-03|0.540|

Monotone from **both sides** — undershoot on the watched spool (a lag cannot hold a floor
instantaneously), overshoot on the other. **Observed order ≈ 0.8, i.e. SUB-first-order**, and
that is what is reported, not a claimed order of 1. What this gate exists to rule out — a
structural mismatch between `required` and `_surge_fuel`'s min-select — is ruled out.

---

## Reduce-to-prior contract (the spine)

- **`lag=None` never enters `_integrate_fuel_asym`** ⇒ the march is **bit-for-bit** rungs
  45/46/47/48/49/50/51. Exact dispatch, rung 47's own contract — not equal-to-tolerance.
  Verified on four arming combinations (bare, `surge`, `surge`+`s_off`, `surge`+`s_off`+`τ_rel`).
- **`lag_relief` / `lag_sweep` / `factorization_grid` are new entry points.** The design run
  `build_turbojet(…).run(…)` is untouched, as at every rung since 7.
- **`lag` refuses to compose** with `s_off`/`τ_rel` (alternative release instruments), with
  `tau_gov` (a two-lag cascade), with an un-armed leg, and with `lp_disabled` (the finding is a
  split BETWEEN spools).

## Verification gates (`tests/test_rung52.py`)

1. **The reduce spine** — `lag=None` bit-for-bit on four arming combinations.
2. **All four refusals** assert with their stated reasons.
3. **The trigger pins itself** — `s_cross`, `g` at the crossing and `relief_watched` invariant
   across a `τ_rel` sweep; the credit spread is **exactly** `0.0`, not merely small.
4. **The credit's zero is not vacuous** — `s_min_lp < s_cross` on every floor tested, and the
   watched relief is **positive** (there is a real credit for `τ_rel` to fail to move).
5. **Non-factorization** — `max_residual` is the same order as `max_main_effect` (≥ 0.4×) at
   **both** ramp rates. *(`slow` — it is a 3×3 grid of paired marches at two `r`.)*
6. **The rate verdict** — the debit is monotone in `τ_rel` while `fuel_removed` rises
   (anti-deflation), and the sign flip's `nu_hp_end` is within 1e-5 relative of bare.
7. **`ds`-stability** — `s_cross` moves ≤ one grid cell per halving. *(`slow`.)*
8. **The instantaneous limit** — `|min φ_lp − φ_lim|` decreases monotonically as τ falls at
   fixed `ds`. *(`slow`.)*
9. **The cycle is untouched** — a design run before and after a rung-52 call is bit-identical.

## Concessions

- **Claim § 1's numerics CORROBORATE a structural argument; they do not carry it.** Stated so a
  reader is not sold a sweep as a discovery — rung 51 got this reason wrong by not making the
  one-line argument, and this rung should not pretend the argument was hard.
- **§ 2 is a broad negative search plus a mechanism, not a theorem.** 7 floors × 2 ramp rates ×
  5 attack constants with no exception, and an argument for why — but no proof that no plant
  admits a counter-case.
- **The self-pinning is exact for the FIRST crossing.** A leg that re-engages (`n_recross > 1`)
  would have later crossings that *are* `τ_rel`-dependent. Not observed in any row here, and
  `n_recross` is reported on every row so it cannot go unnoticed.
- **The lag is LINEAR and first-order in each direction.** No shape sweep was run; nothing is
  claimed about the hand-back's functional form beyond attack-rate ≠ release-rate. **§ 2's
  CANNOT-debit headline inherits this**: its second step rides on the first-order attack's
  undershoot-largest-early, so a delayed-undershoot attack shape is outside the claim.
- **The instantaneous-limit order is ≈0.8, not 1**, and is reported as measured.
- **`φ_lim` and `m` inherit rungs 36/41/48/49's imposed constants.** Magnitudes disclaimed;
  the signs, orderings, machine-zeros and convergences are the claims.
- **`AsymmetricLag` is a CONTROL LAW, not an isolation diagnostic** — the first release-side
  instrument in this ladder that is. That is what licenses § 2's re-scoping of rung 50. It does
  not license reading rungs 50/51 as *wrong*: they measured what a forced release does, and
  that is still what a forced release does.
- The plant is rung 43's non-equilibrium (CPG/TPG) gas — rung 35's standing concession.

## The next seam

**The lag's own SHAPE**, and the sensor+actuator cascade beyond it: a *second-order* or
rate-limited attack (the valve, not the loop), and the two-lag cascade this rung refuses
(`tau_gov` + `lag` — a redline lag and a surge lag on one plant, which is what a real FADEC
runs). § 3's non-factorization is the reason to expect that cascade to be interesting rather
than additive.

## Anchor

`docs/plans/rung52-anchor-asymmetric-lag.md` — the predictions as written before measuring
(P1 scored *understated*, the advisor's P2 scored **FALSIFIED**), the probe transcripts, and
the verified numbers.
