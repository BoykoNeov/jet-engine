# Rung 90 — the CUSTOMER bleed: air the aircraft takes at station 3

**Code:** `rust/src/customer_bleed.rs` (the demand `CustomerBleed`, the steady matcher
`CustomerBleedMatcher` / `R90`, and rung 62's four transient cells copied with the extraction:
`r90_try_close`, `r90_try_close_fuel`, `r90_powers`, `r90_try_instant_tail`); the demand lives on
`TwoSpoolEngine::customer` and is copied onto rung 38's core; rung 62's cells dispatch to rung 90's
when it is active · **Gates:** `rust/tests/rung90.rs` · **Panel:** `print_customer_bleed_table` (born
in Rust, held to `rust/oracle/rust_owned/`, a change detector only) · **Anchor (the derivation, the
linear model and every bar, committed `504d428` before any customer-bleed code existed):**
`docs/plans/rung90-anchor-customer-bleed.md` · **Sandbox:** the Controls view, "Air the aircraft
takes" (`docs/plans/sandbox-plan.md` § 19) · **Scratch scripts and outputs:**
`docs/plans/scratch-archive/rung90/`.

**Shipped as KNOBS** (the sandbox direction): predictions were registered first and are scored
here; a miss is a finding.

Rung 89 closed by naming this seam (as rungs 42, 61, 63, 86–88 had): *"the station-3 customer bleed
— air taken off the core for the aircraft, not to protect a compressor — a bleed the valves here
would have to work around."* The user chose the whole of it in one rung (2026-10-15): the steady
two-shaft engine, the throttle-slam engine, the protection valve beside it, and a sandbox knob, with
the demand set as a fixed share or a fixed amount of air.

> **THE HEADLINE — a different arrow from rung 42's.** Rung 42's interstage bleed was a freedom on
> the LP spool alone (the HP balance cancelled it). Air taken AFTER the HP compressor does not
> cancel: the HP turbine drives its compressor with less gas than the compressor pumps. So the HP
> spool gets its own freedom — pushed AWAY from surge at every power, with no sign change (rung
> 42's response reversed at `π*`; this one would need `π_HPC` ≈ 10). And the LP spool is pushed
> TOWARD surge at high power — the HP's pressure-ratio drop starves the LP face more than the
> missing air feeds it — and helped below a crossing at **910 K** (predicted 900 K). A two-line
> linear model, written before any code, predicts both spools to ≤ 0.009 on the flat maps.
>
> **AND THE VALVE WORKS AROUND IT.** On rung 64's slam the LP's closest approach to surge sits on
> the high-power side of that crossing, so with 5 % of the HP air going to the aircraft the LP
> protection valve bleeds MORE to hold the same margin (all 15 riding pairs, +17–760 %), its
> ceiling falls (all 6 cells), and a fixed AMOUNT — a larger share at the slam's low-power start —
> costs it more again (all 16). The HP spool gains more margin than the LP loses (all 6). The
> named "likeliest miss" (the slam's minimum sitting below the crossing) did not happen.

## 1. What it adds

Per unit inlet air the LPC pumps 1, the HPC `(1−b)`, the burner and turbines `(1−b)(1−c)`. `c`
enters six places (anchor § 1): the HP shaft balance `(1−c)` (NEW — no longer cancels); the HP face
referral (†) `1/(1−c)` (NEW — so rung 39's HP efficiency loop cannot be called verbatim, as rung 42
could; it is copied with (†-c), and the triangle survives); the LP balance `(1−b)(1−c)`; the LP face
referral (‡) `1/[(1−b)(1−c)]`; the rebuild books the extraction at station 3 before the burner; the
thrust charges both extractions full ram drag. Untouched: both turbine pins.

**Two demands.** *Share* `c` fixed; *amount* `W = w × design airflow`, so `c = W/ṁ_HPC` floats
(iterated inside the steady joint loop; explicit per closure trial on the transient). A closure past
half the air is refused (`Abort`, in words) — checked at the ROOT of the Tt4-pinned closure, because
its high-wall bracket trial imposes a tiny flow (found when the first build stopped good solves).

**Where it lives.** On the design engine, which the design run ignores (hardware captured with it
off, as rung 42's valve). Every sibling a reader builds keeps it (gated). Every body that cannot
model it — rungs 38, 39 and 42 steady; 40, 43 (hence 57) transient — REFUSES an active one (gated).
A transient engine carrying one gets rung 90's steady table on its inner core, so rungs 40/44's
steady re-matches see the same demand.

## 2. Reduces

Inactive (`None`, share 0, amount 0) dispatches to the earlier bodies verbatim: steady → rung 42's
`try_match_bleed` (→ rung 39 at `b = 0`), transient → rung 62's existing bodies (→ rung 57 at
`b = 0`), gated bit for bit at three throttles × two valves (steady) and three lever arms
(transient, both closures and the fuel path); the closure carries no `customer` field then. The
design run is unchanged by a demand (gated). **Plant gate** (rung 62's — the only check that
catches a missing factor in the power loop): the transient equilibrium equals the steady match to
1e−9 in speeds, flow coefficients, pressure ratios, share, burner and face flow and thrust, for both
demands, with and without the interstage valve. Mass identities and "the amount mode delivers its
amount" gated. No oracle, no CLI golden and no earlier rung's gate moved.

## 3. Scoring (anchor § 3 — bars frozen at `504d428`)

Rig A: rung 42's (CPG, π 3 × 6, 250 K / 50 kPa / M 0.85), `Tt4` 1500 → 650 K. Rig B: rung 64's
(shaped / tilted × `r` ∈ {0.25, 0.5, 1}, `ds` 0.005, 1000 → 1400 K, `φ` valve `b_max` 0.10, `sm`
0.30–0.42).

| # | verdict | measured |
|---|---|---|
| A1 HP magnitude | **HIT** 12/12 | `d ln φ_H/dc` within 0.0085 of the model (bar 0.03) |
| A2 HP, no `π*` | **HIT** 12/12 | +0.218 at 1500 K rising to +0.602 at 650 K |
| A3 LP magnitude | **HIT** 12/12 | within 0.0093 of the model (bar 0.05); the residual is a near-constant +0.008, as `b`'s +0.015 |
| A4 LP crossing | **HIT** | one sign change, at **910 K** (bar 900 ± 50) |
| A5 HP off its line | **MISS** 10/12 | at the same `x_H` the `c` 0.05 HP point sits **0.55 %** off the `c` = 0 line at 1500 K, 0.77 % at 1400 K (bar > 1 %), rising to 3.24 % at 650 K. Rung 42's interstage bleed: ≤ 0.016 %. The freedom is real (35–200×); the bar was a guess, not a derivation |
| A6 shaped signs | **HIT** | HP + everywhere; LP −0.320 at 1500 K, +0.252 at 650 K |
| A7 the trade | **HIT** 12/12 | at 5 %: thrust −8.3 % (customer) vs −7.2 % (interstage) at 1500 K, −23.5 vs −15.9 at 650 K; TSFC +5.5 vs +2.9, +27.4 vs +13.8 |
| A8 amount mode | **HIT** at 1500–700 K; **not scorable** at 650 K as written | share 0.0525 → 0.1218 as the throttle falls; at 700 K the LP shift +0.0336 vs the share's +0.0117. At 650 K the amount takes 12.8 % and the nozzle unchokes — refused, out of scope (the envelope shrinks, as rung 42's) |
| B1 | **HIT** 6/6 | valve shut, `min φ_LP` falls by 0.0041–0.0058 |
| B2 | **HIT** 6/6 | `min φ_HP` rises by 0.0086–0.0105, more than the LP falls |
| B3 | **HIT** 15/15 riding pairs | the valve's bleed rises (shaped `r` 0.5, `sm` 0.38: 9.29e−3 → 1.27e−2) |
| B4 | **HIT** 6/6 | ceiling falls by 0.0037–0.0057 |
| B5 | **HIT** 16/16 | amount > share (same cell: 1.62e−2) |

Every B comparison is between equal-length runs (same point count in every cell).

## 4. After scoring (reported, not scored)

* **A dormant valve wakes.** Where the valve never opened without the customer bleed (tilted `r` 0.5,
  `sm` 0.34) it opens with it; where it saturated (`r` 0.25, `sm` 0.42) its violation deepens
  (0.7800 → 0.7743 shaped).
* **A fixed amount collapses the operating point.** Taking air lowers the flow, which raises the
  share: on the Controls engine an amount of 0.1 of the design flow is 11 % of the HP air at 1500 K
  and 19 % at 1000 K; 0.3 has no steady point even at 1400 K (50.2 %).

## 5. Why — the model

`d ln φ_L/dc = 1.5 − a_L − a_H(1 − t_L)`, `d ln φ_H/dc = 0.5 + (1 − t_L)(1 − a_H)` (anchor § 1;
`a` each compressor's `d ln π / d ln(τ−1)`, `t_L = (τ_L−1)/τ_L`). The HP: its loading falls by
`(1 − t_L)` per unit `c` and its face flow rises by 1, so it always gains. The LP: its loading falls
by 1 (`−a_L + ½`) and its face sees `π_HPC/(1−c)` — the HP's pressure-ratio drop `a_H(1 − t_L)`
against the missing air's +1. At high power `a_H(1−t_L)` ≈ 1 and the LP's own loss wins; at low power
both `a`'s fall and the LP gains. The same algebra on `b` reproduces rung 42's published response,
including its `π*` (anchor § 0 G2).

## What it does to its neighbours

* **Rung 42 — EXTENDED, its structural claim BOUNDED.** "Bleed is a freedom on the LP spool and NOT
  the HP" holds for the station it was made at; one station downstream the HP spool gets a freedom
  of its own, and rung 42's (†)-carries-no-`b` argument is exactly what fails. Rung 42's `π*` is the
  linear model's `a_H = 1`.
* **Rung 64 — its ceiling moved by a LOAD.** The ceiling is the lever's authority (64's headline);
  a customer bleed lowers it without touching the valve.
* **Rung 41 — CONFIRMED.** The LP is still the exposed spool; the customer bleed widens the split.

## Concessions

* Customer air goes overboard; **cooling air that re-enters at the turbine is not modelled** (it
  changes the turbine flow and work — a different sink, the next seam).
* The hardware is captured with the customer bleed OFF; a real engine is sized with it on.
* Rig B scored at `ds` 0.005 only (the anchor registered no step bar); every B margin is ≥ 0.0037 in
  `φ`, against rung 64's ~1e−4 step sensitivity.
* Inherited: invented maps, rung 36's imposed `φ_surge`, the CPG gas, both nozzles choked, the LP
  valve only, no fuel leg in rig B (the sandbox runs them beside it, replayed bit for bit).
* No shaft-power offtake (the other half of an aircraft's demand).

## The next seam

**Turbine cooling air** — HP-compressor air that skips the HP turbine's nozzle and rejoins
downstream: a bleed that changes the turbines' flow and work, not only the compressors'. Beside it:
**shaft-power offtake** (the generator's load on the HP shaft), the other half of what an aircraft
takes.
