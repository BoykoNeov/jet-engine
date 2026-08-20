# Rung 48 — The Wf/pt3 acceleration schedule: a fuel-side limiter rebates a spool iff it engages UPSTREAM of that spool's own surge minimum

**Scope.** `AccelSchedule` + `TwoSpoolFuelTransient.accel_schedule` /
`_sched_fuel` / `integrate_fuel(…, accel=…)` / `schedule_relief` / `engagement_sweep`, with
`accel` threaded through `_fuel_ramp_march`, `phi_excursion_fuel` and
`transient_surge_margin_fuel` — the **second, FEEDFORWARD min-select leg** on the two-shaft fuel
path. Rungs 46/47 built the **feedback** leg (the TIT topping governor, then its lag) and closed
on this concession:

> *"No lead/anticipation, no rate-limit, no sensor-plus-actuator cascade is modelled (a
> lead-compensated governor COULD reach the LP — the one thing a pure lag cannot; that is the open
> door this rung leaves, having shut the pure-lag one)."*

Rung 48 walks through that door with the instrument a real FADEC actually uses — the classic
`Wf/pt3` acceleration schedule — and finds that the door was never about *lead* at all.

> **The rung-46/47 LP/HP surge-relief SPLIT is not a property of which spool, which limiter, or
> which lag. It is ONE mechanism, and it is a TIMING crossing: a fuel-side limiter rebates a
> spool's surge minimum IF AND ONLY IF it engages UPSTREAM of THAT SPOOL'S OWN minimum.** Both
> crossings are measured on ONE instrument, because the schedule margin `m` maps continuously to
> an engagement start time `s_eng(m)` that sweeps ACROSS both minima: at `r = 0.5`, `relief_lp`
> falls to **EXACTLY 0** as `s_eng` passes `s_lp* = 0.24` (m ≈ 0.42) **while `relief_hp` is still
> +0.0075**, and `relief_hp` dies only when `s_eng` reaches `s_hp* = 0.40` (m ≈ 0.48). **This is
> NOT rung 44's ramp-rate lever in disguise**: the fuel removed varies SMOOTHLY and stays POSITIVE
> through both crossings, and the settled endpoint is unmoved (bare to 5 dp for `m ≥ 0.25`) — at `m = 0.45` the SAME
> clip removing the SAME fuel gives `relief_hp = +0.0034` and `relief_lp = 0.000000`, a per-spool
> split at FIXED fuel-removed that no ramp-rate story can produce. Rung 46's finding is recovered
> as the special case `s_eng > s_lp*` (a redline-triggered governor is late BY CONSTRUCTION), and
> rung 47's refutation as the case where a lag pushes `s_eng` later still. **THE HONEST BOUNDARY:
> at small `m` the limiter DOES degenerate into rung 44's ramp-rate lever — it throttles the whole
> ramp and the accel no longer completes.**

`accel=None` leaves rungs 45/46/47 **bit-for-bit**, and the default `build_turbojet(…).run(…)`
design run is **bit-for-bit rung 6**.

---

## The control law — a FEEDFORWARD leg, its shape DERIVED, one imposed scalar

The limiter caps the fuel by the compressor-delivery total pressure:

```
Wf  ≤  (1 + m) · κ_ss(n_H) · pt3          [the classic Wf/pt3 accel schedule]
```

with `pt3 = pt4/π_b = π_HPC · π_LPC · pt2` the HP-compressor **delivery** total (already carried
by `_close_fuel`; **zero new plant**), `n_H` the corrected HP speed the HP map already runs on,
and

```
κ_ss(n_H)  =  (Wf / pt3)  ON THE STEADY RUNNING LINE at that n_H
```

built by `accel_schedule(…)` from the plant's own `equilibrium(…)` points over the accel band.
**The schedule SHAPE is therefore DERIVED, not imposed** — the entire imposition is the ONE scalar
margin `m`, exactly the discipline rung 41 used for `φ_surge` and rung 46 for `Tt4_max`. `m = 0`
is "never exceed the steady fuel/pressure ratio"; real schedules sit above the steady line and
below the surge line, which is the `m > 0` band this rung sweeps.

**Why this leg is EARLY-acting where rungs 46/47's is LATE.** The topping governor is *feedback on
a consequence*: it cannot fire until `Tt4` has already reached the redline, which on an accel
happens near the END of the ramp — that is the whole content of rung 46's "the surge debit is paid
on early-ramp fuel, upstream of the governor's late window" and of rung 47's refutation (a lag
only moves a late window later). The `Wf/pt3` leg is *feedforward on the cause*: `Wf` steps up
immediately while `pt3` — which can only rise as the spools spin up — LAGS, so the ratio is
already 21% above `κ_ss` at `s = 0.10`, far upstream of the LP minimum. **The instrument does not
need phase lead. It needs to be watching the input rather than the output.**

**The cap is IMPLICIT in `Wf`** (`pt3` and `n_H` both move with the fuel through the closure), so
`_sched_fuel` is a bracketed Illinois set-point solve — the same structure as rung 46's
`_topping_fuel`, and for the same reason. It returns the scheduled fuel UNCHANGED (float-
identical) when the schedule is already under the cap, which is what makes the dormant reduce
bit-for-bit rather than merely equal.

**Min-select ordering.** With both legs armed the applied fuel is
`min(schedule, topping(Tt4_max), accel_cap)`, the accel leg applied LAST to whatever the
(bare | topped | topped-lagged) path would have applied. Whenever only one leg binds, the
composite is bit-for-bit that leg's single-leg result — gate 3.

---

## Sign-space only — inherited from rungs 41/44/45/46/47, plus one imposed `m`

`φ_surge` is imposed (rung 41), the maps are representative (rung 32's standing concession,
doubled), the ramp band and rate are chosen. Rung 48 makes **no** claim about the absolute margin
a real schedule carries. It delivers **signs and a crossing**: that `relief_lp` and `relief_hp`
each switch on iff `s_eng` is upstream of that spool's own minimum, that the switch-off is EXACT
(the upstream march is bit-identical, not merely close) rather than gradual, that fuel-removed is smooth and positive through it, and that
the endpoint is unmoved. The **rung-36 discipline holds: report the crossing, gate the flip.**

---

## THE FINDINGS (config: CPG gas, accel 1000→1400 K, `ρ = 1`, `r = 0.5`; `tests/test_rung48.py` reproduces)

### 1. The window EXISTS — `m` is an ENGAGEMENT-TIME instrument (the enabling measurement)

On the **bare** rung-45 accel the ratio `(Wf/pt3)/κ_ss` rises **monotonically** from 1.0000 at
`s = 0` to a max 1.4885 at `s = 0.46`:

| `s` | 0.00 | 0.10 | 0.18 | **0.24** | 0.32 | 0.40 | 0.46 |
|---|---|---|---|---|---|---|---|
| ratio | 1.0000 | 1.2105 | 1.3340 | **1.4004** | 1.4574 | 1.4841 | 1.4885 |
| `φ_LP` | 0.7731 | 0.7473 | 0.7373 | **0.7355 ← min** | 0.7399 | 0.7510 | 0.7629 |

The LP surge minimum sits at `s_lp* = 0.240` (48% of the ramp) where the ratio is **1.40** — and
the ratio is already **1.21 at `s = 0.10`**, far upstream. Because the ratio rises monotonically
*through* `s_lp*`, the margin `m` maps continuously to an engagement start `s_eng(m)` that sweeps
from `s ≈ 0` (m → 0) to `s ≈ 0.46` (m → 0.49). **That is what makes the rung possible**: one
scalar moves the clip across the surge minimum with everything else held fixed. Had the bare ramp
sat only marginally above `κ_ss` at `s_lp*`, any `m` engaging there would have throttled the whole
early ramp and rung 44's "schedule-slaved" verdict would have swallowed this rung.

### 2. THE HEADLINE — the per-spool timing crossing, twice on one instrument

`engagement_sweep`, `r = 0.5` (bare: `min φ_LP = 0.73547` at `s_lp* = 0.240`,
`min φ_HP = 0.86120` at `s_hp* = 0.400`):

| `m` | `s_eng` | vs `s_lp*` | `relief_lp` | `relief_hp` | fuel removed | `ν_H` end |
|---|---|---|---|---|---|---|
| 0.15 | 0.080 | upstream | **+0.018965** | +0.053719 | 0.01202 | 0.95894 |
| 0.25 | 0.140 | upstream | **+0.008498** | +0.035737 | 0.00411 | 0.95905 |
| 0.35 | 0.200 | upstream | **+0.001818** | +0.018433 | 0.00127 | 0.95906 |
| 0.38 | 0.220 | upstream | **+0.000179** | +0.013399 | 0.00081 | 0.95906 |
| 0.40 | 0.240 | **on the min** | +0.000053 | +0.010228 | 0.00057 | 0.95906 |
| 0.42 | 0.280 | downstream | **0.000000** | +0.007493 | 0.00037 | 0.95906 |
| 0.44 | 0.300 | downstream | **0.000000** | +0.004594 | 0.00022 | 0.95906 |
| 0.46 | 0.340 | downstream | **0.000000** | +0.002051 | 0.00010 | 0.95906 |
| 0.48 | 0.400 | downstream (**on `s_hp*`**) | **0.000000** | **+0.000016** | 0.00002 | 0.95906 |
| 0.50 | — (dormant) | — | 0.000000 | 0.000000 | 0.00000 | 0.95906 |

Read the two columns as two independent crossings **of the same instrument**:

- `relief_lp` decays to EXACTLY zero as `s_eng` passes `s_lp* = 0.24` — and stays exactly zero.
- `relief_hp` is still **+0.0075 there**, and dies only when `s_eng` reaches `s_hp* = 0.40`.

**The "EXACTLY 0" is a MECHANISM statement, not a rounding.** A downstream-engaging march is
**bit-identical to the bare one on every recorded key until its first engagement** — verified at
`m = 0.42, 0.45, 0.48`, where the first divergence lands exactly at `s_eng` (0.280 / 0.320 /
0.400), all downstream of `s_lp* = 0.24`. Nothing upstream of the minimum moved, so the minimum
is the same float. That is why the switch-off is exact rather than merely small (gate 8b).

**Both crossings are demonstrated to the SAME standard — but the HP one needs a slower ramp.** At
`r = 0.5` the ratio peaks at 1.4885, which runs out of dial just as `s_eng` reaches `s_hp* = 0.40`:
the HP side there shows a collapse to +0.000016, not a clean exact zero. At **`r = 2.0`** the minima
separate further (`s_lp* = 0.32`, `s_hp* = 0.64`) and `m = 0.20` engages at `s = 0.700`, strictly
PAST the HP minimum with fuel still being removed (0.00002): `relief_hp` is then **exactly 0**, and
the march is bit-identical through BOTH minima (gate 9b). The rule is not an LP-only result.

**One rule covers both, and covers rungs 46 and 47:** a fuel-side limiter rebates a spool iff it
engages upstream of that spool's own surge minimum. Rung 46's split (HP rebated, LP machine-zero)
is the special case of a leg that is late by construction — a redline-triggered governor cannot
fire before `Tt4` reaches the redline. Rung 47's refutation is the case where a lag pushes an
already-late `s_eng` later still. **Nothing about the LP spool made it unreachable; the
instrument was simply looking at the wrong signal.**

### 3. NOT the ramp-rate lever — the non-tautology gate

The obvious deflation is "any clip removes fuel, and removing fuel slows the accel, and rungs
44/45 already showed the surge excursion is ramp-rate-driven — so this is rung 44 restated." It is
not, on three counts measured together:

- **Fuel removed is SMOOTH and POSITIVE through both crossings** (0.00081 → 0.00057 → 0.00037 →
  0.00022 → 0.00010 → 0.00002). At `m = 0.42`, where `relief_lp` is *exactly* zero, fuel is still
  being removed and the accel is still being slowed. The LP minimum gets **nothing** from it.
- **The endpoint is unmoved**: `ν_H` at settle is 0.95906 — bare to 5 dp — for every `m ≥ 0.25`,
  and still within **0.012 %** of bare at `m = 0.15` (0.95894, the leg's longest engagement in
  the admissible window). The comparison is same-endpoint — unlike a retuned ramp rate, a different
  trajectory family. (This is also why "match the accel time and compare" was rejected as a
  control: a matched-time slower ramp is a different plant-in-time, the rung-42/43/45 currency
  trap. The instrument controls for itself.)
- **The split at FIXED fuel-removed is the clincher**: at `m = 0.45` ONE clip, ONE trajectory, ONE
  quantity of fuel removed yields `relief_hp = +0.0034` and `relief_lp = 0.000000`. A ramp-rate
  effect cannot rebate one spool and not the other from the same removed fuel. Only a *timing*
  mechanism can.

### 4. THE HONEST BOUNDARY — where it DOES become the ramp-rate lever

At `m = 0.05` the cap sits so close to the steady line that it binds essentially from `s = 0.04`
and never releases inside the window: `ν_H` at settle is **0.87246** (vs 0.95906) — the accel has
not completed — and `Tt4_peak` collapses to 1243.7 K. The apparent relief there (+0.031) is
substantially "the accel was de-fanged," the same confound rung 47 rejected the coupled
single-valve lag for. **The admissible window at `r = 0.5` is `m ∈ [0.15, 0.45]`**, where the
endpoint is untouched; the finding is stated there and the `m → 0` corner is reported, not used.

> **CORRECTED** (`docs/both-edges-limiter-negative.md`): this window was originally stated as
> `m ∈ [0.10, 0.45]`, and its low end was too wide. Measured at `m = 0.10`: `ν_H_end = 0.95714`
> against bare `0.95906` — a shift of **1.9e-3, ~4× gate 10's own 5e-4 tolerance.** `m = 0.10` was
> never in the swept `MARGINS`, so no gate was affected; the honest window is `[0.15, 0.45]`, which
> is what gate 10 actually tests. The headline crossing (`m` = 0.38–0.48) is untouched.

### 5. Fast-ramp corroboration — the minima COINCIDE, so the crossings coincide

At `r = 0.15` the LP and HP minima both sit at `s = 0.140` (the LP min is at 93% of the ramp, vs
48% at `r = 0.5`). The rule then predicts a SINGLE crossing rather than a split — and that is what
happens: `relief_lp` and `relief_hp` both die together as `s_eng` passes 0.14 (both +0.0049/+0.0123
at `m = 0.70`, `s_eng = 0.12`; both **exactly 0** at `m = 0.78`, `s_eng = 0.16`). A degenerate case
that would have broken a "the LP is special" reading, and confirms the timing reading. The crossing
must therefore be stated **per `r`** (`s_lp*` moves with ramp rate) — never as a universal `s`.

### 6. Character contrast with rung 46 — this leg is NOT a TIT limiter

The `Wf/pt3` leg does not hold the redline and does not try to: at `m = 0.40` the peak `Tt4` is
still 1642 K (bare 1695 K). The two legs are complements, not substitutes — one protects the
turbine, the other protects the compressor, they engage in different windows, and the min-select
composite is what a real accel schedule actually is.

---

## Reduce-to-prior contract (the spine)

1. **`accel=None` ⇒ bit-for-bit rungs 45/46/47.** The leg is never consulted — no call to
   `_sched_fuel`, so the march is byte-identical on the recorded keys, with and without a redline
   and with and without `tau_gov`.
2. **A dormant schedule ⇒ bit-for-bit bare.** `m` above the march's max ratio (≥ 0.50 at
   `r = 0.5`) leaves the cap above the schedule everywhere, `_sched_fuel` returns its argument
   float-identically, and the trajectory is the rung-45 one float-for-float.
3. **Two-leg composite ⇒ single-leg, when only one binds.** Armed together, `(Tt4_max, accel)`
   reproduces the `Tt4_max`-only march bit-for-bit when the accel leg is dormant, and the
   `accel`-only march bit-for-bit when the redline is above the resulting peak. This is the
   min-select ordering gate.
4. **`lp_disabled` ASSERTS** — the finding is a per-spool split, inherently two-shaft (rung 46/47's
   rule, verbatim).
5. **A DECEL never fires the leg** (the ratio stays ≤ 1 below the running line) ⇒ bit-for-bit
   rung 45.
6. **The design run is bit-for-bit rung 6** — the whole structural/dynamic ladder is a separate
   entry point.

---

## Verification gates (`tests/test_rung48.py`)

1. `test_reduce_accel_none_bit_for_bit` — contract 1, over bare / `Tt4_max` / `tau_gov` paths.
2. `test_reduce_dormant_schedule_bit_for_bit_rung45` — contract 2.
3. `test_reduce_two_leg_composite` — contract 3, both directions.
4. `test_reduce_lp_disabled_asserts` / `test_decel_never_fires` — contracts 4, 5.
5. `test_cycle_untouched_rung6` — contract 6.
6. `test_kappa_derived_from_running_line` — `κ_ss` at a steady point reproduces that point's own
   `Wf/pt3`, and `pt3 == π_HPC·π_LPC·pt2` (the identity, checked directly — not by dividing out
   the factors it multiplies back).
7. `test_window_exists` — the ratio rises monotonically through `s_lp*` and is ≥ 15% above `κ_ss`
   at `s_lp*` (finding 1; the enabling measurement, gated as a sign not a level).
8. `test_engagement_crossing_lp` — `relief_lp > 0` for every `m` with `s_eng < s_lp*` and
   **exactly 0** for every `m` with `s_eng > s_lp*` (finding 2, the headline).
8b. `test_downstream_clip_is_bit_identical_through_the_minimum` — the MECHANISM behind gate 8:
   a downstream clip leaves the whole pre-minimum march bit-identical, diverging exactly AT
   engagement. Without it, gate 8 checks only the consequence.
9. `test_engagement_crossing_hp_is_later` — at the `m` where `relief_lp` is exactly 0,
   `relief_hp > 0`; and `relief_hp` survives until `s_eng` reaches `s_hp*` (finding 2, the split).
9b. `test_hp_crossing_demonstrated_on_a_slow_ramp` — at `r = 2.0`, `s_eng = 0.700 > s_hp* = 0.64`
   with fuel still removed ⇒ `relief_hp` EXACTLY 0 and bit-identical through both minima. This is
   what raises the HP side from corroborated to demonstrated.
10. `test_not_ramp_rate_lever` — fuel removed is strictly positive and monotone-decreasing in `m`
    across the LP crossing while `relief_lp` goes exactly to 0, and `ν_H` at settle is unchanged
    to **5e-4** over the swept `MARGINS = (0.15 … 0.48)` (finding 3, the non-tautology). *The
    tolerance is 5e-4, not 1e-4: the longest in-window engagement (`m = 0.15`) moves the endpoint
    by 0.012 %. Corrected from an earlier "1e-4 for `m ≥ 0.10`" here, which matched neither the
    asserted tolerance nor the swept set — see `docs/both-edges-limiter-negative.md`.*
11. `test_degeneracy_boundary` — at `m = 0.05` the accel does NOT complete (`ν_H` end below the
    bare endpoint by > 1e-2) (finding 4, the honest boundary — gated so it cannot be quietly
    folded into the finding).
12. `test_fast_ramp_single_crossing` (slow) — at `r = 0.15` the two minima coincide and the two
    reliefs die together (finding 5).
13. `test_robustness_map_shapes` (slow) — the crossing rule holds on the rung-47 `SHAPES` set,
    including mode-free `hp-only`.

---

## Concessions

- **`m` is IMPOSED and so is `φ_surge`** — no claim about the margin a real accel schedule
  carries, nor about the depth of any crossing. Load-bearing are the two crossings' existence and
  their per-spool ORDER, the exactness of the switch-off, and the reduces.
- **`κ_ss` is derived from THIS plant's running line** — a real schedule is drawn against a real
  surge line with a real margin; here the shape comes from the model's own steady points. That
  keeps the imposition to one scalar but means the schedule cannot be better than the map
  (rung 32's standing concession, doubled across two spools).
- **The `m → 0` corner IS the ramp-rate lever** (finding 4). The finding is stated only in the
  endpoint-preserving window and the corner is gated, not hidden.
- **Quasi-steady `pt3`** — the limiter reads the same closure the plant runs on; no sensor lag, no
  filtering, no rate limit on the cap. **A lagged `pt3` sensor was investigated and is NEGATIVE**
  — `docs/pt3-sensor-lag-negative.md`. This bullet originally guessed that the interesting question
  was "whether the lag pushes `s_eng` **past** `s_lp*`"; **that had the sign backwards.** A
  first-order lag on a *rising* `pt3` reads LOW, so the cap is LOWER and the leg engages **EARLIER**
  — the opposite of rung 47's loop lag, because this one lags the limiter's *input measurement*
  rather than its *output trigger*. The investigation is a **confirmation of this rung**, not an
  extension of it: at matched sub-grid engagement the sensor and a constant `m'` give the same
  `relief_lp` (to 0.0–0.4 %, at two `ds`), so the lag is an effective-margin reparameterisation;
  and its one genuinely new degree of freedom — a release edge set independently of engagement — is
  **structurally** inert, since finding 1's monotone ratio means the leg cannot release before the
  ramp flattens while both minima are always inside the ramp. It also supplies a negative control
  this rung lacked: at `m = 0.48, τ_p = 0.05` the lag removes 15× the fuel and moves engagement
  0.400 → 0.280 yet leaves `relief_lp` EXACTLY 0, because 0.280 is still downstream of `s_lp*`.
  That remainder — **a limiter whose engagement AND release both land inside the ramp** (a
  rate-limited or lead-lag/washout-filtered `pt3`) — has since ALSO been investigated and is
  **NEGATIVE**: `docs/both-edges-limiter-negative.md`. It closes the whole `pt3`-filter family at
  once, structurally: **the ramp is the only clock in the system** — every candidate second edge is
  manufactured by the fuel ramp flattening, while both surge minima are ramp-driven and hence
  strictly inside the ramp. The live door is now a **φ / surge-margin FEEDBACK limiter** (the only
  signal with a turnover upstream of a surge minimum is the surge variable itself).

- **THE LAW OF THIS RUNG IS SHARPENED BY THAT INVESTIGATION** (same doc, and it is a strict upgrade,
  not a correction): the crossing rule is a consequence of a **mechanism** — a clip **ARRESTS** the φ
  descent immediately and permanently (verified in all 19 armed rows over `r` = 0.15/0.5/2.0), so the
  limited march's minimum sits AT `s_eng`. Hence the relief is not merely signed but **closed-form,
  from ONE bare march**: `relief = min_{s ≤ s_eng} φ_bare − min_s φ_bare` — exact in the `ds → 0`
  limit (first-order convergent), and **identically 0 whenever `s_eng > s*`**, which *derives* this
  rung's exact-zero crossing rather than fitting it. It also settles what this rung could not:
  because the minimum is fixed at the engaged window's OPENING, the rule is an **EDGE** condition
  **necessarily** — the window's length, its fuel removed and its release edge are all causally
  downstream of an already-determined minimum.

- **…AND THAT LAST SENTENCE IS BOUNDED BY RUNG 49** (`docs/rung49-spec.md`). The arrest-⇒-edge
  argument holds **for this leg**, whose release is structurally post-ramp. Build a limiter whose
  window closes INSIDE the ramp — the φ-feedback floor, the door named above — and a **second,
  opposite-signed term appears at the RELEASE edge**: the withheld fuel is handed back to a
  still-ramping plant and the unwatched spool's descent **re-opens**, relocating its minimum to
  just after `s_rel`. So an LP φ-floor **DEBITS** the HP while engaging upstream of `s_hp*`, which
  this rung's law alone would forbid. **What survives verbatim is the credit term** — rung 49
  reproduces this rung's exact-zero crossing on a fresh instrument class, off a bare march. What is
  bounded is the *universality*: this rung is the **one-shot-arrest special case**. THIS leg is
  **empirically immune** to the release debit (φ monotone non-decreasing from `s_eng` to the end in
  32/32 cells, that doc's own measurement) — but **why** is an open seam: rung 49 measures that the
  ratio governing the debit within its own family does **not** transfer to this leg.
- **No claim that the leg protects the redline** (finding 6) — it is a compressor-protection leg;
  the composite with rung 46's is what a real schedule is.
- **Reacting-gas fuel control deferred** (rungs 35/43/45/46/47, verbatim) — the leg runs on the
  non-equilibrium gases; the finding is gas-independent.
- **Both NGVs choked, no bypass, one `eta_m`, isentropic knobs, no bleed** — inherited from
  rungs 38–47.

### What the RUST PORT measured about these gates (slice T, 2026-08-20)

The port ported this suite one-to-one — sixteen test functions for sixteen — and then injected five
defects into the shipped reader. Four were caught by 5 to 11 gates each. The fifth is the finding.

1. **`fuel_removed`'s SCALE is unobservable to the whole project, in either language.** Dropping
   the `0.5` from its trapezoid doubles the integral exactly and is caught by **0 of 16** gates.
   The reason is not a loose bar — there is no bar to loosen. `fuel_removed` is read in exactly
   three places in the project, and every one of those readings is either `> 0.0` or a pairwise
   `<`; both predicates are invariant under multiplication by any positive constant. The trapezoid
   could be a rectangle rule and nothing would know. Only a VALUE gate can hold it, and the port's
   slice-T oracle is now that gate.

2. **The six `relief == 0.0` assertions are ONE claim, not six.** For the downstream margins the
   bare and limited marches are bit-identical for their first 14 / 16 / 20 points and the LP argmin
   sits at index 12, inside every one of those prefixes — so the two `min` calls read the SAME
   float and the difference is `0.0` for the reason `x - x` is. A one-ULP perturbation upstream
   fails all six together. (They do sit on five DIFFERENT marches, so they are not a block in the
   trivial sense — but the mechanism is one.)

3. **Gate 7's `ratio[s_lp]` is an EXACT dictionary lookup, and that is load-bearing.** It raises if
   the LP minimum is not on the ramp prefix's grid. A port that reads "the last point with
   `s <= s_lp`" instead would silently substitute a neighbouring ratio; measured, the exact lookup
   FINDS the key on this cell, so the divergence would have been latent rather than loud.

4. **Gates 8b and 9b can report a TRUNCATED march as an unmoved one.** Both find the first
   differing point by zipping the bare and limited trajectories, so a limited march that broke out
   early yields "no difference" and trips the assertion whose message says the clip genuinely moved
   the march. Python has the hole too; naming it costs one length check.

5. **Two arms of `schedule_relief` are live code that no cell in this suite reaches.** `s_eng`'s
   `float("nan")` fallback needs `n_engaged == 0`, which happens at `m >= 0.55` and which the
   lowest suite cell (gate 12's `m = 0.78` at `r = 0.15`) misses by exactly one engagement point;
   and the `min(traj, key=...)` first-on-tie rule is never exercised, because no `phi` array here
   has a tie (the closest gap is 1.61e-5). Both are gated in the port by cells and a rule test
   ADDED for the purpose, not inherited.

---

## Anchor

`docs/plans/rung48-anchor-accel-schedule.md`. The **method** is again Cohen–Rogers–Saravanamuttoo
*Gas Turbine Theory* Ch. 9 — the acceleration fuel schedule as a `Wf/pt3`-vs-speed law sitting
between the steady running line and the surge line, min-selected with the maximum-turbine-
temperature limiter of rungs 46/47. Rung 48's own contribution is the **unification**: the
rung-46/47 LP/HP surge-relief split is not a spool property or a limiter property but a **timing
crossing per spool**, demonstrated by sweeping ONE scalar that moves the engagement time across
both minima, with the ramp-rate deflation excluded by a per-spool split at fixed fuel-removed.
