# Rung 92 — SHAFT-POWER OFFTAKE: the aircraft's generator load on a shaft

**Code:** `rust/src/shaft_offtake.rs` (the demand `Demand` / `ShaftOfftake`, the power unit `p_ref`,
the loaded steady cascade and matcher `ShaftOfftakeMatcher` / `R92`, and rung 62's transient cells:
the closures MARKED — `r92_try_close`, `r92_try_close_fuel` — the power and tail cells WRAPPED —
`r92_powers`, `r92_try_instant_tail` — and the page's reading `shares`); the load lives on
`TwoSpoolEngine::offtake` and is copied onto rung 38's core; rung 62's cells dispatch to rung 92's
first when it is active (the closures on the engine, the power and tail cells on the closure's own
mark) · **Gates:** `rust/tests/rung92.rs` · **Panel:** `print_shaft_offtake_table` (born in Rust,
held to `rust/oracle/rust_owned/`, a change detector only) · **Anchor (the derivation, the linear
model and every bar, committed `1773b7d` before any offtake code existed):**
`docs/plans/rung92-anchor-shaft-offtake.md` · **Sandbox:** the Controls view, "The aircraft's
generator" (`docs/plans/sandbox-plan.md` § 21) · **Scratch scripts and outputs:**
`docs/plans/scratch-archive/rung92/`.

**Shipped as KNOBS** (the sandbox direction): predictions were registered first and are scored
here; a miss is a finding.

Rung 91 closed by naming this seam: *"Shaft-power offtake — the aircraft's generator load on the HP
shaft: the other half of what an aircraft takes (rung 90 took its air)."* The user chose (2026-10-15)
either shaft as a switch (the HP shaft of today's engines, the LP shaft of the more-electric
proposal), the demand as a share of the turbine's work or a fixed power, and the whole seam in one
rung as rungs 90/91.

> **THE HEADLINE — a bleed is a power offtake plus air thrown away.** On the simple maps this is
> EXACT (gated to 1e−9): an LP-shaft share `s` is rung 42's interstage bleed `b = s` with the air
> kept, and an HP plus an LP share `s` is rung 90's customer bleed `c = s` with the air kept — every
> speed and pressure ratio identical, each face's flow coefficient differing by exactly `(1−s)`.
> So a generator is the half of a bleed that never leaves, and it shows what the bleeds hid: **on
> the HP shaft it pushes BOTH compressors toward surge at every power** (no crossing): the HP
> compressor gets less work, its pressure ratio falls, and that starves the LP face too. Rung 90's
> HP spool was helped only because its air LEFT. **On the LP shaft the same load is nearly free in
> the steady state**: the LP's own loss is offset by the HP's pressure rise (crossing at **1295 K**,
> predicted 1290), and the HP sees rung 42's response exactly (crossing at rung 42's `π*`, **789 K**,
> predicted 783). Per unit power the HP shaft costs more on both spools at every throttle, while
> the thrust and fuel bill is the same on either shaft. A two-line model written before any code
> predicts both shafts to ≤ 0.0030.
>
> **THE VALVE PAYS FOR THE HP SHAFT, AND THE SPOOL SLOWS.** On rung 64's slam the same kilowatts on
> the HP shaft make the LP protection valve bleed more (all 5 riding pairs; 1.6e−4 → 2.7e−2 at the
> shaped `r` 0.5 cell, `sm` 0.34), its ceiling fall (6/6), and cost it more than on the LP shaft (all 24 set
> points); the HP spool's rise is 1.3–1.6 % later (6/6). **The one MISS:** the LP-shaft load's small
> steady HP relief does not survive a fast slam (B2: 4/6, both `r` = 0.25 cells reverse).

## 1. What it adds

A load `P_x` on a shaft, after the bearings. Per unit of that shaft's compressor air
`q = P_x / ṁ_compressor` enters ONE place, the shaft balance:
`h(Tt3) − h(Tt25) = η_m (gas per HPC air) Δh_HPT − q_H`, likewise the LP per face air. **It moves no
gas**: both turbine pins, the face referrals, the burner's air and the thrust bookkeeping are
untouched — gated: on the perfect gas `τ`, `π` of both turbines stay put to 1e−11.

**Two demands per shaft, either or both shafts.** *Share*: `P_x = s · P_turbine`. *Power*:
`P_x = w · P_ref`, `P_ref` the design HP compressor power (307.6 kW per kg/s of design air on rig A)
— ONE unit on both shafts, so an HP and an LP load of the same `w` are the same kilowatts; the share
floats (iterated in the steady joint loop over the compressor flows).

**The transient closure never sees a shaft balance**, so rung 62's closure cells run unchanged and
only MARK the result (`CloseState::offtake`); the power and tail cells WRAP the earlier bodies:
`Φ̇ = (P_turbine − P_compressor − P_x)/(P_ref,shaft · ν)`, the turbine's power rebuilt from the
unloaded rate (`Φ̇ · P_ref,shaft · ν + air · p_c`, `air` that shaft's compressor air on every lower
path — gated by the plant gate's share check).

**Where it lives.** On the design engine, copied onto rung 38's core; the design run ignores it
(hardware sized unloaded). Refused in words by every body that cannot model it: rungs 38, 39, 42, 90
and 91 steady (the new body is a copy of 91's); rung 40's power and tail **only on an UNMARKED
closure** — a machine below rung 62 (rung 57's, rung 43's), whose closures never pass rung 62's
dispatch. **The perfect gas only** (the steady body is rung 91's cooled cascade). Bounds: a share in
`[0, 0.5)`; at a root, half a turbine's work or more is refused in words; a power past the **fold**
— less work, less air, a larger share, until no steady point exists — is refused at the trial that
finds the load above the whole turbine's work ("no steady point").

## 2. Reduces

No load (`None`, zero demands) dispatches to the earlier bodies verbatim: steady → rung 91's
`try_match_cooled` (→ 90 → 42 → 39); transient → rung 62's cells unmarked (→ 91 / 90 / 62 / 57).
Gated bit for bit at three throttles × two valve settings × {plain, customer share, vane cooling} ×
three inactive spellings (steady) and two lever arms × the same three (transient, both closures and
the fuel path); an unmarked closure carries `offtake: None`. The design run is unchanged (gated).
**Plant gate**: the transient equilibrium equals the steady match to 1e−9 (speeds, flow
coefficients, pressure ratios, the HP turbine point, `Tt3`, burner and face flow, thrust, and both
load shares) for HP and LP × share and power × every lower path (rung 57's, rung 62's bled, rung
90's share and amount, rung 91's) at 1400 and 1000 K — 40 points. **The fuel-metered path** (the one
the sandbox marches, which the plant gate's `Tt4`-pinned solve never enters) carries the load too: at the
loaded equilibrium's speeds and fuel, rung 43's fuel closure lands at rest at 1200 K through rung 92's
cells, for both shafts and modes on rung 57's, rung 62's bled and rung 90's paths — and the same fuel
on the unloaded engine is visibly not at rest. Both shaft balances close on the
rebuilt stations with the load booked; the power mode delivers `w · P_ref`. No oracle, no CLI golden
and no earlier rung's gate moved.

## 3. Scoring (anchor § 3 — bars frozen at `1773b7d`)

Rig A: rungs 42/90/91's (CPG, π 3 × 6, 250 K / 50 kPa / M 0.85), `Tt4` 1500 → 650 K, readings
Richardson-extrapolated from 0.01 and 0.02. Rig B: rung 64's (shaped / tilted × `r` ∈ {0.25, 0.5,
1}, `ds` 0.005, 1000 → 1400 K, `φ` valve `b_max` 0.10, `sm` 0.30–0.42); demands none, HP power
`w` 0.02, LP power `w` 0.02; every comparison between equal-length runs (291 / 341 / 441 points).

| # | verdict | measured |
|---|---|---|
| A1 HP shaft magnitude | **HIT** 24/24 | within 0.0030 (LP) / 0.0023 (HP) of the model (bar 0.01): LP −1.399 → −0.891, HP −0.904 → −0.392 |
| A2 HP shaft, no relief | **HIT** 24/24 | both negative at every `Tt4` |
| A3 LP shaft magnitude | **HIT** 24/24 | within 0.0011 on both spools |
| A4 LP shaft, HP crossing | **HIT** | once, positive above, at **789 K** (bar 783 ± 75) — rung 42's `π*`, as the identity requires |
| A5 LP shaft, LP crossing | **HIT** | once, negative above, at **1295 K** (bar 1290 ± 55) |
| A6 equal power | **HIT** 11/11; 650 K **not scorable as written** | the HP shaft lower on both spools at every `Tt4` 1500–700 K (flat; flow/press the same). At 650 K the 0.02 reading unchokes the nozzle (refused, out of scope) |
| A7 shaped signs | **HIT** 8/8 | 1500 K: HP shaft −1.036 / −0.603, LP shaft −0.025 / +0.078; 650 K: −0.825 / −0.296, +0.151 / −0.010 |
| B1 | **HIT** 6/6 | HP load, valve shut: `min φ_LP` falls by 0.046–0.050, `min φ_HP` by 0.029–0.037 |
| B2 | **MISS** 4/6 | LP load raises `min φ_HP` at `r` 0.5 and 1 (tilted `r` 0.5 by only 2e−6) and LOWERS it at `r` 0.25, both maps (by 4e−4 / 1.4e−3) |
| B3 | **HIT** 5/5 riding pairs; ceiling 6/6 | the valve bleeds more (shaped `r` 0.5, `sm` 0.34: 1.6e−4 → 2.7e−2); ceiling falls by 0.051–0.056 |
| B4 | **HIT** 3/3 | HP load > LP load where both ride — and at all 24 set points |
| B5 | **HIT** 6/6 | HP `ν` 90 % rise later by 0.014–0.021 (1.3–1.6 %) |

**B2, the finding.** The LP load's steady HP relief is small (+0.21 to +0.31 per unit `w`, so
+0.004–0.006 at `w` 0.02), and the slam adds a term that grows with ramp speed and opposes it:
shaped `min φ_HP` shift +0.0015 (`r` 1) → +0.0006 (`r` 0.5) → −0.0004 (`r` 0.25). Monotone in the
ramp rate; the mechanism was not read (a fixed load on the LP shaft slows the LP spool's own
acceleration, the likeliest carrier). The anchor named B5 and A5 as the likeliest misses; both hit.

## 4. After scoring (reported, not scored)

* **The same kilowatts cost the same thrust.** At 0.05 of `P_ref` (flat): −1.17 / −1.16 % thrust,
  +2.63 / +2.63 % fuel per unit thrust (HP / LP shaft) at 1500 K; −8.46 / −8.41 % and +14.3 / +14.2 %
  at 1000 K. The power comes out of the same fuel either way; only the surge bill differs.
* **A fixed power has a fold.** At 0.05 of `P_ref` the share grows from 5 % (HP) / 14 % (LP) at
  1500 K to 20 % / past half by 900 K; at 1000 K an HP load of 0.10 settles at 44 % of the HP
  turbine's work and 0.12 has no steady point at all — less work, less air, a larger share.
* **In the slam the LP load HELPS the LP spool** (`min φ_LP` up in 6/6, by 0.0001–0.006) and the
  valve bleeds less (where it rides) — the steady model's +0.4 per unit `w` at the ramp's 1000 K
  start. Its LP sign was left unscored (§ 0's decomposition, exact only on flat maps, predicted the
  opposite on `flow/press`; measured, the linear model was right: −0.025 at 1500 K, not −0.28).

## 5. Why — the model

Rung 91's compressor algebra with `h`, `ℓ` the fractional changes of the HP and LP compressor work
and no referral term (no air leaves): an HP share is `h = −1, ℓ = 0`, so `d ln φ_L = −a_H`,
`d ln φ_H = ½ − a_H` — the HP compressor's pressure ratio falls (`a_H` ≈ 0.9–1.4) and both faces
lose flow, the HP's speed drop (½) recovering only part of it. An LP share is `h = 0, ℓ = −1`:
`d ln φ_L = ½ − a_L + a_H t_L` (its own loading loss against the HP's pressure rise, as `Tt25`
falls) and `d ln φ_H = t_L(a_H − 1)` — rung 42's HP response, `π*` included. Rung 90's customer row
is HP + LP + 1 on each face; rung 42's bleed row is LP + 1 on the LP face: the +1 is the dumped
air, and on the simple maps the decomposition is exact.

## What it does to its neighbours

* **Rung 90 — its HP mechanism CORRECTED, its verdict confirmed.** "The HP turbine drives its
  compressor with less gas than the compressor pumps, so the HP spool gets a freedom away from
  surge": the work deficit pushes the HP spool TOWARD surge (−0.90 per unit at 1500 K); the relief
  (+0.21) is the dumped air's face referral (+1) winning. The sandbox's customer note now says so.
* **Rung 42 — its HP verdict EXPLAINED.** The bleed's HP response (and its `π*`) is pure LP-shaft
  work: the identity holds the HP spool bit-identical with or without the air.
* **Rung 64 — its ceiling moved by a load a third time**, now one that takes no air.
* **Rung 41 — CONFIRMED.** The LP is still the exposed spool.

## Concessions

* **Fixed burner temperature.** A real control holds speed or thrust and burns hotter to carry the
  load; at a fixed `Tt4` the surge effects here are the load's alone and understate what a real
  engine sees at idle.
* **Hardware sized unloaded**; no gearbox or generator inertia (the load is a pure power, so its
  torque `P/ω` rises as the spool slows — included, since `Φ̇` carries `1/ν`).
* The identities are exact only on the simple maps (constant map efficiency and loading).
* The perfect gas only (refused otherwise). Rig B at `ds` 0.005 only.
* Inherited: invented maps, rung 36's imposed `φ_surge`, both nozzles choked, the LP valve only, no
  fuel leg in rig B (the sandbox runs them beside it, replayed bit for bit).

## The next seam

**A metal-temperature limit** that lets cooling buy a hotter burner (rung 91's), and with it **a
burner that carries the load** — a control that holds thrust (or speed) and lets `Tt4` rise to pay
for the generator: the realistic idle case this rung could only bound.
