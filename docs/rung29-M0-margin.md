# "Earned at design", hardened — rung 29's one flight condition, re-checked on the M0 axis

**Status: a CONFIRMATION of rung 29 plus a CORRECTION to how the `π_c` check's unification was
framed — NOT a new rung.** No new physics, no new constant, no code-path change. It re-measures the
verdict on the last axis rung 29 left untouched (`M0`, flight Mach) and finds it **robust with a wider
margin than `π_c`** — and, unlike `π_c`, **monotone**. The interesting part is that being the *clean
opposite* of `π_c` is exactly what pins down the currency both checks share.

## What rung 29 (and the π_c check) left open

> **One design point.** `π_c = 10`, `M0 = 0.85`. … The `π_c` axis is now CHECKED
> (`docs/rung29-pi-c-margin.md`) … The **`M0` / flight axis is still untouched.**
> — `docs/rung29-spec.md` § Concessions

`π_c` did not invert (it confirmed, weak and non-monotone, with an interior worst case). This axis is
the natural completion: the other knob that moves both channels rung 29's finding rests on.

**Method.** Same counterfactual discipline as the `π_c` doc: identical `REAL_LOSSES` and ambient
(`T0 = 250 K`, `p0 = 50 kPa`), fixed `π_c = 10`, sweep `(Tt4, M0)`, read the shipped
`Gas.shifting_turbine`. The `M0 = 0.85` column reproduces the rung-29 anchor exactly
(`dT5/T5 = 0.01067 %` at `Tt4 = 1500`), the wiring check before any new column is trusted. Two cells
are unrunnable for reasons that are *not* the turbine and were traced to be sure: `M0 = 0` trips
`ram must not cool/depressurize` (the `h_c` inverse returns `T0 − ε` at zero ram — a static-condition
numerical edge, so the sweep starts at `M0 = 0.3`), and the high-`Tt4` ceiling is the **equilibrium
burner-balance** assert (`components.py:300`), the cycle's own limit, not the bracket's.

## The verdict: robust with a wider margin — and MONOTONE, the clean opposite of `π_c`

At `Tt4 = 1500 K`, sweeping `M0` (`π_c = 10`):

| `M0` | `pt4` [MPa] | `delta_h` [kJ/kg] | inventory `x_O+x_H+x_OH` | completion | `ΔT5` [K] | `ΔT5/T5` [%] |
|---|---|---|---|---|---|---|
| 0.3  | 0.50 | 265 | **3.591e-05** | 86.00% | 0.1458 | **0.01130** |
| 0.85 | 0.75 | 299 | 3.178e-05 | 89.68% | 0.1347 | 0.01067 |
| 1.6  | 1.98 | 394 | 2.325e-05 | 96.12% | 0.1065 | 0.00901 |
| 2.5  | 8.02 | 579 | 1.309e-05 | 99.67% | 0.0641 | 0.00628 |
| 3.0  | 17.40 | 712 | **7.569e-06** | 99.97% | 0.0384 | **0.00427** |

**"Earned at design" is `M0`-robust, with more margin than `π_c` gave.** The bound never exceeds
**~0.0114 %** at any runnable `M0` — **8.8× under** the `_SHIFT_EARNED_TOL = 1e-3` threshold. And the
worst case is the **low-`M0`** end (takeoff/climb-out), *not* the design cruise point: the shift **falls
monotonically** from low `M0` to `3.0`, so `M0 = 0.85` (0.01067 %) is already on the protected side of
its own worst runnable case. There is no interior extremum and no turnover. The trend **plateaus**
toward the low edge rather than diverging — `M0 = 0.3` reads 0.01130 %, and spot-checks down to the
climb-out band hold flat (0.01136 % at `M0 = 0.2`, 0.01139 % at `M0 = 0.1`); `M0 = 0` itself is only a
numerical edge (the `h_c` inverse returns `T0 − ε` at zero ram, tripping the ram assert), not a physical
limit. So the worst case is pinned at low `M0` by the physics, not by where the scan happens to start.

**This is the clean opposite of `π_c`.** `π_c` produced a bowl-shaped boundary and an interior maximum
in the shift (`docs/rung29-pi-c-margin.md`); `M0` is monotone-protective — the shift, and the
earned/not-earned boundary, move one direction the whole way. Rung 28's `β` fell monotonically on
`π_c`; `M0` is the shifting-turbine bracket's β-like axis. That difference is not a discrepancy to
reconcile — it is the measurement that identifies *what actually sets the shift*, below.

## The mechanism: the SAME currency (`INVENTORY × COMPLETION`), read where it is lopsided

The `π_c` check sharpened rung 29's currency from the entry inventory to the **recombined** inventory,
`ENERGY = INVENTORY × COMPLETION`. The `M0` axis confirms that currency directly: the recombined
inventory `inv × completion` **falls monotonically** (3.09e-05 → 7.57e-06) and tracks the shift, at a
near-constant **4722 … 5069 K per unit recombined mole fraction** (+7.3 % over the whole axis). The
product law that the `π_c` doc could only quote at one point holds across all of `M0`.

But here the two channels are **lopsided**, and that is why the axis is monotone:

- **Inventory ↓ (protective) dominates.** Raising `M0` raises `pt4` via ram (0.50 → 17.40 MPa, ×35),
  which suppresses dissociation hard: inventory falls **×4.7**.
- **Completion ↑ (inflating) is nearly spent.** It rises only **86 % → 100 %** (×1.16) — it starts
  near saturation and has almost nowhere to go, so it cannot fight the inventory collapse. On `π_c` the
  same factor swung **36.5 % → 99.995 %** (×2.7) and *did* fight it, producing the turnover.

So the product falls monotonically: inventory wins at every `M0` because completion is out of room.

## The correction: it is the `delta_h` SWING, not completion "headroom", that decides turnover

The tempting reading — "`M0` is monotone because completion is already saturated at `π_c = 10`" — is
**wrong**, and the control that refutes it is running the `M0` sweep at **`π_c = 2`**, where `delta_h`
is small and completion starts with plenty of headroom:

| `M0` (at `π_c = 2`) | `delta_h` [kJ/kg] | completion | `ΔT5/T5` [%] |
|---|---|---|---|
| 0.3 | 62 | 33.06% | 0.00615 |
| 0.85 | 70 | 36.52% | 0.00606 |
| 1.6 | 93 | 45.79% | 0.00579 |
| 2.5 | 137 | 61.08% | **0.00517** |

Completion now has room (33 % → 61 %, ×1.85) — **and the shift is still monotone-falling. No turnover.**
Headroom alone does not bring it back. This is a clean isolation, not just a plausibility argument: the
`M0`-at-`π_c=2` sweep **starts** at ~33 % completion — essentially the **same** starting headroom as the
`π_c` sweep's own low end (~36.5 % at `π_c = 2`, `M0 = 0.85`). *Same starting headroom, opposite outcome*
— `π_c` turns over from there, `M0` does not — so the difference cannot be headroom; it is the
**driver**. The turnover requires the completion channel to *outpace* the inventory suppression over
some sub-range, and completion is driven by `delta_h`. The `delta_h` **swing** is the discriminator:

| axis | `delta_h` swing across the axis | completion swing | shift |
|---|---|---|---|
| `π_c` 2→80 (at `M0 = 0.85`) | 70 → 799 kJ/kg — **×11.4** | 36.5 % → 100 % (×2.7) | **turns over** |
| `M0` 0.3→3.0 (at `π_c = 10`) | 265 → 712 — ×2.7 | 86 % → 100 % (×1.16) | monotone |
| `M0` 0.3→2.5 (at `π_c = 2`) | 62 → 137 — ×2.2 | 33 % → 61 % (×1.85) | monotone |

`π_c` drives `delta_h` **hard** because it raises the compressor *temperature ratio* `τ_c` — the
compression work climbs steeply. `M0` drives `delta_h` **weakly** because it moves the ram temperature
*datum* `Tt2` with `τ_c` fixed, and `delta_h = (h_c(Tt3) − h_c(Tt2))/(η_m(1+f)) ∝ Tt2·(τ_c − 1)` scales
only linearly with the datum. Both axes suppress inventory through `pt4` by a comparable factor; only
`π_c` also drives completion hard enough for it to win first. **That unifies both checks onto one
`delta_h → completion` surface** — turnover where the axis drives `delta_h` steeply (compression work),
monotone where it does not (`M0`'s datum shift) — and the `π_c = 2` control is what rules out the
simpler "completion headroom" story.

## The envelope: protective per point, yet the earned OPERATING BAND shrinks

`M0` has a structural feature `π_c` did not: ram heating raises the burner-entry `Tt3`, which lifts the
**floor** (minimum runnable `Tt4`, where the burner would need negative fuel). Both edges and the
boundary move — measured by bisection:

| `M0` | floor [K] | `Tt4*` [K] | ceiling [K] | earned band [K] | not-earned band [K] |
|---|---|---|---|---|---|
| 0.3 | 633  | 1838.3 | 2375.4 | **1206** | 537 |
| 0.85 | 684 | 1848.5 | 2415.5 | 1164 | 567 |
| 1.6 | 842  | 1875.2 | 2527.2 | 1033 | 652 |
| 2.5 | 1155 | 1923.9 | 2736.4 | 769 | 812 |
| 3.0 | 1377 | 1960.6 | 2883.2 | **583** | **923** |

The operational statement — the one to lead with — is the first row: **`Tt4*` rises (1838 → 1961 K),
so higher `M0` earns the freeze to a hotter burner**, and any fixed `Tt4` shows a smaller shift. That is
the monotone verdict above, and it is where a real engine lives.

The band arithmetic is geometrically real but must be read with its edges in mind:

- **Per operating point, higher `M0` is protective** (the `Tt4*` rise above).
- **Yet the earned band `[floor, Tt4*]` shrinks** — ×2.1 (1206 → 583 K) — because ram heating drives
  the **floor up faster** (633 → 1377 K) than it lifts the boundary; the not-earned band `[Tt4*, ceiling]`
  **widens** ×1.7 (the ceiling rising because higher `pt4` lets the burner close hotter). The earned
  fraction of the runnable window falls 69 % → 39 %.

**But both band edges are non-operational, so do not read "39 % earned" as "a Mach-3 engine loses usable
range."** The floor (633 → 1377 K) is the near-zero-fuel burner-squeeze limit, and the ceiling
(2375 → 2883 K) sits well above any real turbine-inlet limit; the shrink magnitude is inflated by a floor
no engine runs at, and the widening is mostly into `Tt4` no engine reaches. What is genuinely operational
is only the boundary motion — `Tt4*` rising — and that is *protective*. This still parallels the `π_c`
doc's "not-earned band widens" and is sharper (on `M0` the squeeze is on the *floor*, an edge `π_c` lacks),
but "protective" here means *deeper margin at each point*, not *more earned territory*. (The first-pass
"the whole envelope becomes earned at high `M0`" is the opposite of true: the ceiling rises faster than
`Tt4*`, so non-physical hot territory *opens up*, it does not close.)

## Disclaimed

- **Fixed ambient — this is the `M0` axis, not a flight envelope.** `T0 = 250 K`, `p0 = 50 kPa` are
  held while `M0` sweeps, so `pt4` at `M0 = 3` reaches 17 MPa. A real high-Mach point flies higher and
  thinner, with *lower* `p0` pushing `pt4` (and the inventory suppression) back down. The clean monotone
  result is a property of this isolated axis; "supersonic cruise is safe" is **not** claimed. Like the
  `π_c` check, one knob at a time.
- **The currency law is quoted at the cool design `Tt4` only, and `CO` is worse here.** `x_O+x_H+x_OH`
  omits `CO → CO₂`; at `M0 = 3` `pt4 ≈ 17 MPa` so `CO` intrudes even at `Tt4 = 1500` — the +7.3 %
  drift in K-per-recombined-mole (vs `π_c`'s ±4 %) is that carrier showing. The decomposition is the
  right qualitative mechanism; the quantitative law is not pushed hot or to extreme `pt4`.
- **`η_t = 1` and no rate**, unchanged — the whole plane is the reversible *bound*, not a march. The
  deferred finite-rate turbine (`docs/rung29-spec.md` § Deferred) is untouched.

## The honest statement

Rung 29 said: *freezing the turbine is EARNED at design and bites hot; the shift is set by the
recombined radical inventory; the `M0` axis is unchecked.*

It now reads: **"earned at design" holds at every runnable `M0` (0.1 climb-out to 3.0) with an 8.8×
margin, and the worst case is low-`M0` takeoff, not cruise. `M0` is monotone-protective — the clean opposite of
`π_c`'s interior turnover — and the two axes unify on one `INVENTORY × COMPLETION` surface: the shift
turns over only where the axis drives `delta_h` (hence completion) steeply, which `π_c` does through the
compressor temperature ratio and `M0` does not through its datum shift. And the flight axis is
double-edged in a way `π_c` is not: deeper margin at each point, but ram heating shrinks the earned
operating band ×2.1 while the not-earned band widens ×1.7.**

## Gates

In `tests/test_rung29.py`:

- `test_M0_helper_reproduces_the_certified_flight_anchor` — the wiring gate (gate-2 principle): the new
  `_bracket_M0` helper at `M0 = 0.85` reproduces the certified FLIGHT path `_bracket(1500, 10)`
  bit-for-bit and hits the rung-29 anchor 0.01067 %, so the relative monotonicity gates cannot read
  "monotone" on wrong numbers.
- `test_earned_at_design_is_M0_robust` — `Tt4 = 1500` stays earned with >8× margin at every scanned
  `M0`; the boundary is bracketed `1800 < Tt4* < 2200` at every `M0` (earned at 1800, not at 2200), so
  "far above design" cannot drift.
- `test_M0_shift_is_monotone_protective` — the differentiator, sign-flipped from `π_c`'s "not simply
  protective": the shift is strictly **decreasing** in `M0`, asserted at a **hot** `Tt4 = 2100` (a 2.1×
  swing, well clear of the low-`Tt4` noise scale).
- `test_M0_channels_are_lopsided` — inventory falls and completion rises (the currency's two channels),
  but completion's swing is far smaller than inventory's; and the **recombined** inventory falls
  monotonically, tracking the shift.
- `test_delta_h_swing_not_headroom_is_the_discriminator` — the correction: at `π_c = 2` (completion has
  headroom) the `M0` sweep is **still monotone**, so headroom alone does not restore the turnover; the
  weak `delta_h` swing on `M0` is why.
- `test_M0_envelope_band_squeeze` — cheap point-checks of the two-edge structure: a low `Tt4` runnable
  at low `M0` becomes unrunnable at high `M0` (floor rises); a `Tt4` above design is not-earned at low
  `M0` but earned at high `M0` (`Tt4*` rises, protective); a high `Tt4` unrunnable at design becomes
  runnable at high `M0` (ceiling rises).
