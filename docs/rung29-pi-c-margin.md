# "Earned at design", hardened — rung 29's one design point, re-checked on the π_c axis

**Status: a CONFIRMATION of rung 29 plus a SHARPENING of its finding — NOT a new rung.** No new
physics, no new constant, no code-path change. It re-measures a verdict rung 29 shipped from a single
`π_c` and finds it **robust with margin**; and it separates rung 29's own currency (`radical
inventory`) from the thing that actually sets the shift, on the one axis where the two come apart.

## What rung 29 left open

> **One design point.** `π_c = 10`, `M0 = 0.85`. Whether "earned at design / bites hot" survives the
> `π_c` axis is unchecked — a seam, flagged not chased.
> — `docs/rung29-spec.md` § Concessions

The precedent it named: rung 28's `β` margin got exactly this treatment (`docs/rung28-beta-margin.md`)
and the worry **inverted**. This one does not invert. It confirms — and the interesting part is the
mechanism underneath, not the verdict.

**Method.** Same counterfactual discipline as the β doc: identical `FLIGHT` and `REAL_LOSSES`, sweep
`(Tt4, π_c)`, mark cells where the cycle does not solve. Every number below came from the shipped
`Gas.shifting_turbine`. The `π_c = 10` column reproduces the rung-29 anchor exactly
(`0.0107 / 0.0765 / 0.3680 / 1.8592 %` at `Tt4 = 1500 / 1800 / 2100 / 2400`), which is the check that
the `delta_h`/`far` wiring is right before any new column is trusted.

## The verdict: robust with margin — and NOT simply "protective"

The discriminating measurement is the **earned/not-earned boundary** `Tt4*(π_c)`, where `|ΔT5/T5|`
crosses `_SHIFT_EARNED_TOL = 1e-3`. It is measured where `ΔT5 ≈ 1 K`, three orders above the solver
floor, so it is the robust way to read the plane.

| π_c | `Tt4*` [K] | runnable ceiling [K] | not-earned span [K] | `ΔT5/T5` at `Tt4`=1500 [%] |
|---|---|---|---|---|
| 2  | **1957.4** | 2269 | 312 | 0.0061 |
| 3  | 1902.6 | 2301 | 398 | 0.0083 |
| 5  | 1867.6 | 2346 | 478 | 0.0099 |
| 8  | 1852.5 | 2391 | 538 | 0.0106 |
| 10 | 1848.9 | 2415 | 566 | **0.0107** |
| 15 | **1846.0** | 2460 | 614 | 0.0106 |
| 20 | 1847.0 | 2495 | 648 | 0.0103 |
| 40 | 1855.2 | 2590 | 735 | 0.0096 |
| 80 | 1868.4 | 2700 | 832 | 0.0090 |

**"Earned at design" is π_c-robust.** At `Tt4 = 1500 K` the bound never exceeds **0.0107%** at any
`π_c` from 2 to 80 — **9.4× under** the declared threshold — and the boundary never comes within
**346 K** of the design point. Neither the low-π_c edge (which the β doc's lesson said to check, and
where the radical inventory is *largest*) nor the high-π_c edge threatens it. The cool end is safe
too: at `Tt4 = 1200` the bound is 0.0005% at every runnable `π_c`.

**But the verdict is not "π_c is protective."** Unlike β — which fell monotonically and had a clean
sign — `π_c` here is **weak and non-monotone**, and it cuts both ways:

- `Tt4*` is **bowl-shaped**: it *falls* 111 K from `π_c`=2 to a minimum ≈ **1846 K near `π_c` ≈ 15–16**,
  then *rises* 22 K out to `π_c`=80. The worst case is **interior**, sitting near where real engines
  and rung 29's own design point live. (The bowl is very flat between `π_c` 11 and 22 — 1846.0 to
  1847.5 K — so the minimum's *location* is soft; its *value* is not.)
- The runnable ceiling **rises** with `π_c` (2269 → 2700 K) **faster than `Tt4*` moves**, so the
  not-earned band **widens 2.7×** across the axis. Higher `π_c` buys a slightly higher boundary and
  considerably more territory above it.

So the honest statement is *robust with margin, weak and non-monotone in `π_c`* — not an inversion,
and not a comfort of the β kind.

### The interior maximum is resolved, not bisection noise

At `Tt4 = 1500` the shift peaks in `π_c` at a value only ~2% above its `π_c`=7 flank, which is close
enough to the solver's declared gate-2 tolerance (`1e-6` relative) to deserve a check. It survives:

| π_c | 9.75 | 10.00 | 10.25 | **10.50** | 10.75 | 11.00 | 11.25 |
|---|---|---|---|---|---|---|---|
| `ΔT5` [K] | 0.13504982 | 0.13470777 | 0.13435353 | 0.13398870 | 0.13361466 | 0.13323274 | 0.13284403 |
| `ΔT5/T5` [%] | 0.01066454 | 0.01066831 | 0.01067060 | **0.01067156** | 0.01067131 | 0.01066996 | 0.01066760 |

Smooth to **8 significant digits**, with consistent second differences at the 1e-8 level. `_work_limited_expand`
bisects to `1e-13` relative in `T` and `1e-12` in `p`; gate 2's `1e-6` is a loose *assertion* threshold,
not the solver's accuracy. The turnover is real.

Two readings follow, and only the second is quoted as a claim:

- **Rung 29 did not sample a favourable `π_c`.** `π_c = 10` (0.010668%) sits essentially **at** the
  design-point maximum over the whole axis (0.010672% at `π_c` ≈ 10.5). The shipped number is the
  worst case, not a lucky one — the direction that matters for a claim of the form "even the maximum
  shift is negligible."
- The *location* `π_c ≈ 10.5` is **not** claimed as physically meaningful. It is where two opposed
  channels happen to cross for this `FLIGHT` and these losses.

Interior structure in the shift is unambiguous well above the noise question at hotter `Tt4` — the
`Tt4 = 1800` row runs 0.0410 (π_c=2) → 0.0776 (π_c=15) → 0.0687 (π_c=80), a 1.9× rise and a clear
turnover. That, not the 1500 K row, is where to point at the shape.

## The sharpening: `INVENTORY` is an incomplete currency — `ENERGY = INVENTORY × COMPLETION`

Rung 29's finding is **RATIO ≠ ENERGY**: `x_frozen/x_eq` measures *kinetic* distance from equilibrium
and anti-correlates with the shift across `Tt4`, while the absolute **radical inventory** tracks it
(ratio ÷33, inventory ×121, shift ×174). The `π_c` axis breaks the second half of that, and this is
the substantive result of this check:

At fixed `Tt4 = 1500`, over `π_c` = 2 → 80:

| π_c | inventory `x_O+x_H+x_OH` at 4 | eq. inventory at 5 | **completion** | `ΔT5` [K] | `ΔT5/T5` [%] |
|---|---|---|---|---|---|
| 2  | **5.048e-05** | 3.204e-05 | **36.5%** | 0.0876 | 0.0061 |
| 4  | 4.160e-05 | 1.429e-05 | 65.6% | 0.1287 | 0.0093 |
| 6  | 3.702e-05 | 8.001e-06 | 78.4% | 0.1366 | 0.0103 |
| 8  | 3.400e-05 | 4.969e-06 | 85.4% | **0.1369** | 0.0106 |
| 10 | 3.178e-05 | 3.281e-06 | 89.7% | 0.1347 | 0.0107 |
| 20 | 2.548e-05 | 6.389e-07 | 97.5% | 0.1189 | 0.0103 |
| 40 | 1.990e-05 | 5.337e-08 | 99.73% | 0.0971 | 0.0096 |
| 80 | **1.477e-05** | 7.689e-10 | **99.995%** | 0.0751 | 0.0090 |

**The inventory falls 3.4× while the shift rises.** Along `π_c`, inventory and shift **anti-correlate**
over most of the axis — exactly the failure rung 29 diagnosed for the *ratio*, now committed by the
*inventory* it proposed as the fix.

The two opposed channels are both visible in the table:

- **Inventory ↓ (protective, monotone).** Raising `π_c` raises `pt4`, and dissociation produces more
  moles, so pressure suppresses it (Le Chatelier). `5.05e-5 → 1.48e-5`.
- **Completion ↑ (inflating, monotone).** Raising `π_c` raises `delta_h` (70 → 799 kJ/kg), so the
  expansion runs deeper and colder (`T5` 1445 → 831 K) at higher `p5`. Equilibrium at the exit wants
  far more of the pool recombined: **36.5% → 99.995%**. The frozen pool only pays out what the
  expansion asks it to.

Their **product** — the *recombined* inventory — is what sets the shift, and it peaks in the interior.
At `Tt4 = 1500` it predicts `ΔT5` to **±4%** across the whole axis (`ΔT5/Δinventory` = 4707…5085 K per
unit mole fraction), including the turnover location.

**Why rung 29's Tt4-axis claim still stands.** Along `Tt4`, both channels move the *same* way
(hotter ⇒ more inventory *and*, since `T5` rises less than the dissociation does, more to give up), and
the inventory swings **two orders**, dominating. Along `π_c` both channels are **comparable and
opposed** — factor 3.4 against factor 2.7 — so the incomplete currency fails. This is a **refinement**
of rung 29, not a correction: its `RATIO ≠ ENERGY` inversion is untouched, and its `inventory` reading
was right on the axis it measured.

### Disclaimed: the product law is scoped to the cool end, and `CO` is why

`x_O+x_H+x_OH` is rung 29's inventory definition and it **omits `CO → CO₂`**, a major energy carrier
once the pool is hot. The consequence is visible:

| | `ΔT5 / Δinventory` [K per unit mole fraction] |
|---|---|
| `Tt4` = 1500 K | 4707 … 5085 — **flat to ±4%** over `π_c` 2→80 |
| `Tt4` = 2100 K | 11096 … 5475 — **varies 2×** over the same span |

So the quantitative "energy = inventory × completion" law is quoted **only at the cool design point**,
where `CO` is negligible. Hot, the mechanism is still the right qualitative decomposition (both
channels are measured, both monotone, and the interior extremum follows), but the missing carrier is
named rather than absorbed by forcing the O/H/OH product to predict `ΔT5`.

## The honest statement, revised

Rung 29 said: *freezing the turbine is EARNED at the design point and bites hot; the shift is set by
the radical inventory, not the super-equilibrium ratio; whether this survives the `π_c` axis is
unchecked.*

It now reads: **"earned at design" holds at every `π_c` from 2 to 80 with a 9.4× margin, and the
earned/not-earned boundary stays above 1846 K everywhere — but `π_c` is a weak, non-monotone, and
double-edged knob (the boundary is bowl-shaped with an interior worst case near `π_c` ≈ 15, and the
not-earned band widens 2.7× with `π_c`). And the currency needs one more factor: the shift is set by
the recombined inventory — `INVENTORY × COMPLETION` — which the `Tt4` axis cannot distinguish from
inventory alone, and the `π_c` axis can, because there the two factors oppose.**

**What is still not claimed.** `η_t` remains 1 by the nature of a reversible bracket, and the whole
plane is still the *bound*, not a rate — the deferred finite-rate turbine march (`docs/rung29-spec.md`
§ Deferred) is unaffected by any of this. The **`M0` / flight axis** — the other axis this doc left
open — is now **CHECKED** (`docs/rung29-M0-margin.md`), and it is the clean **opposite** of `π_c`:
monotone-protective, no turnover. It **confirms the `INVENTORY × COMPLETION` currency directly** (the
recombined inventory tracks the shift across all of `M0`) and pins down what makes an axis turn over vs
stay monotone: not whether completion has *headroom* but whether the axis drives `delta_h` — and hence
completion — steeply enough for it to outpace the inventory suppression. `π_c` does (×11, via `τ_c`);
`M0` does not (×2–3, a ram-datum shift), which the `π_c = 2` control confirms is decisive over headroom.

## Gates

In `tests/test_rung29.py`:

- `test_earned_at_design_is_pi_c_robust` — `Tt4`=1500 stays earned with >5× margin at every scanned
  `π_c`; and the boundary is bracketed `1800 < Tt4* < 2200` at every `π_c` (earned at 1800, not at
  2200), so "far above the design point" cannot drift.
- `test_pi_c_channels_oppose` — inventory strictly **decreasing** in `π_c` and completion strictly
  **increasing**, at fixed `Tt4`: the two opposed channels, each on its own.
- `test_pi_c_is_not_simply_protective` — the shift **rises** from `π_c`=2 to 10 and **falls** from 10
  to 80 (asserted at `Tt4`=1800, where the turnover clears any solver-noise question), forbidding the
  β-style "higher `π_c` is protective" reading.
- `test_inventory_alone_fails_on_the_pi_c_axis` — the sharpening: inventory and shift **anti-correlate**
  from `π_c`=2 to 10, so entry inventory is not sufficient; the recombined inventory is.
