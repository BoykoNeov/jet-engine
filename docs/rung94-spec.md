# Rung 94 — A BURNER THAT CARRIES THE LOAD: each throttle holds thrust or HP speed

**Code:** `rust/src/burner_hold.rs` (the hold `Hold`, the search `floor_of` / `try_hold_root` and its refusals
`HoldRefusal` / `refusal_words`, the steady hold on rung 92's matcher `try_hold_point`, the loaded metal reading
`metal_of`, and the metal cap `try_metal_cap`); the Controls view's ends (`sandbox_controls::build`) · **Gates:**
`rust/tests/rung94.rs`, the page's `rust/tests/sandbox_hold.rs` · **Panel:** `print_burner_hold_table` (born in Rust,
held to `rust/oracle/rust_owned/`, a change detector only) · **Anchor (the model and every bar, committed `5924f6a`
before any hold code existed):** `docs/plans/rung94-anchor-burner-hold.md` · **Sandbox:** the Controls view, "Throttle
holds burner temperature / thrust / HP speed" and "Stop the burner at the metal limit" (`docs/plans/sandbox-plan.md`
§ 23) · **Scratch scripts and outputs:** `docs/plans/scratch-archive/rung94/`.

**Shipped as KNOBS** (the sandbox direction): predictions were registered first and are scored here; a miss is a
finding.

Rung 92 closed by naming this seam, and rung 93 left it as the second of the user's two rungs (2026-10-16): each
throttle end holds **thrust or HP speed, a switch**, the burner temperature floating; the fuel ramp between the ends
unchanged; the metal limit **reported** by default and **binding** on a switch.

> **THE HEADLINE — a burner that carries the generator relieves the compressors.** Holding thrust, the burner runs
> hotter (about 49 K for 5 % of the design HP compressor power at 1500 K, 84 K at 1000 K) and pushes more air, so the
> LP compressor sits FURTHER from stall than at a fixed burner temperature at every power, and the HP compressor too
> above idle. A generator on the LP shaft, once the burner carries it, leaves BOTH compressors further from stall
> than no generator at all (1000–1500 K, both maps). So rung 92's concession — "a fixed burner temperature
> understates the surge effects at idle" — is WRONG for the LP spool at every power and for the HP spool above idle;
> at idle the HP compressor's correction is a wash. (These signs were seen on the existing model before the freeze —
> reported, not credited.) A linear model built from rung 92's slopes, with a bar set by its own measured curvature,
> predicted every burner rise.
>
> **HOLDING HP SPEED IS NOT HOLDING THRUST.** With the generator on the HP shaft the speed hold burns 47–92 % hotter
> than the thrust hold and gives MORE thrust (+4 % at 1500 K, +15 % at 700 K); with it on the LP shaft the speed hold
> barely moves the burner (flat maps) or runs it COOLER (`flow/press`: −15 K at 1500 K) and loses more thrust than a
> fixed burner temperature. And **an LP-shaft generator lifts the engine's thrust floor**: a fixed power of 5 % cannot
> run below 143 N (from 913 K), against 58 N for the same power on the HP shaft — so at a low throttle no hold exists.

## 1. What it adds

**The target.** A throttle set to `T_set` asks for what THIS engine gives at `T_set` with the generator OFF — its
absolute thrust (not per kg of air: the airflow moves) or its HP shaft speed; the burner temperature `T` solves
`Q(T; load) = Q(T_set; no load)` on the loaded engine.

**The search.** Bisection in `T` (the held quantity rises with `T`). Its low end is the lowest `T` the loaded engine
solves at — a 50 K grid, then bisected between the last failing and first solving points (an HP 0.10 solves from 974
K, 80 N, but not at 950 K: a grid alone would start at 1000 K, 104 N, above an 89 N target — rung 93's knob bug
again). Refused in words, naming the bound, below the least the load allows or above the range's top.

**The metal (rung 93)** is read on the loaded point from its temperatures and shares; on the switch it BINDS: the
burner stops at the metal limit (bisected) and the end falls short — the shortfall shown. The cap is refused in words
without a hold (nothing floats), without the metal read, or without cooling (uncooled, the 1985 metals allow about
990 K, below every opening throttle).

**The transient:** only the two ends move, each the hold's `T` on the march's own machine (rung 62's equilibrium ≡ the
steady match by rung 92's plant gate); the fuel ramps between their fuel flows as before.

## 2. Reduces

Holding the burner temperature IS rung 92 (dispatched, bit for bit, steady and on the page); any hold with no active
load meets its target at `T_set` itself and never searches (bit for bit). No oracle, no CLI golden, no earlier gate
moved; the design run is rung 6.

## 3. Scoring (anchor § 3 — bars frozen at `5924f6a`)

Rig A (rungs 42/90–93's), flat and `flow/press`, fixed POWER loads (× the design HP compressor power).

| # | verdict | measured |
|---|---|---|
| H1 thrust hold, burner rise | **HIT** 11/11 | every rise inside its curvature bar, all a little above the linear model (as the quadratic said): flat HP 48.89 K (47.7 ± 5.2) at 1500 K, 84.15 (78.3 ± 36.2) at 1000, 39.96 (38.1 ± 9.5) at 800, 44.24 (41.5 ± 17.4) at 700; flat LP 48.66 / 83.77 / 44.31; `flow/press` HP 49.18 / 49.21, LP 49.11 / 42.51 (1500 / 700 K) |
| H2 the LP relief | **HIT — but KNOWN before the freeze, no credit** | the LP compressor further from stall than at the fixed burner temperature at all 11 cells; the HP compressor too at 1500 and 1000 K (8/8) |
| H3 the LP generator a gain | **HIT 8/8 — but KNOWN before the freeze, no credit** | holding thrust, an LP 0.05 lifts both flow coefficients above the unloaded engine's: flat +0.030 / +0.026 (1500 K), +0.096 / +0.028 (1000 K); `flow/press` +0.022 / +0.017, +0.083 / +0.022 |
| H4 the speed hold | **HIT** | HP loads: thrust rises (+0.040, +0.142 flat at 1500 / 700 K; +0.079 `flow/press`) and the burner runs hotter than the thrust hold's; `flow/press` LP 0.05: −14.83 K (bar −12.9 ± 5.8), thrust down 10.9 % against 8.4 % at the fixed `T0`; every `ΔT` in its bar (flat HP 71.68 / 71.61, flat LP −0.57, `flow/press` HP 94.49, LP −14.83 / −11.04) |

**H2 and H3 were not tests.** The pre-freeze check that measured G4 (`check2.rs`, meant to read only the 700 K
HP-compressor cells) also printed the hold-minus-fixed flow-coefficient shifts at 1000 and 1500 K for every row, on the
existing matcher at `T0 + ΔT_lin` — which IS H2, and with G3's fixed-temperature shifts gives H3's signs. They are
reported as true, not credited as predicted; the scored rung is H1 and H4. (Their measured margins above are the device's.)

Construction (no credit): the reduce; every root meets its target to ≤ 2e−12; the zero-load identities (device
Richardson against `−Q_w/Q_T`: 0.1 % at 1500 K, 0.7 % at 1000 K, 3–4 % at 700 K, where the finite-difference error grows
toward the fold); the search reaches G5's brackets (an HP 0.10 at the 700 K thrust settles at 983.84 K, floor 973.99;
an LP 0.05 is refused naming 143 N from 913 K); the cap lands on the limit; every refusal in words.

## 4. After scoring (reported, not scored)

* **The HP compressor at idle.** Holding thrust against a 0.02 load at 700 K, the HP flow coefficient sits 0.0016
  (flat HP shaft) and 0.0013 (flat LP) CLOSER to stall than at the fixed 700 K, and within 0.0001 either way on
  `flow/press` — the
  zero-load sign change (below ~800 K) survives only as a wash.
* **The thrust cost of a load depends on the shaft off flat maps.** On `flow/press` an HP-shaft load costs 11–12 %
  more thrust per unit power than an LP-shaft one at 1000 and 700 K — rung 92's "the same kilowatts cost the same
  thrust" BOUNDED to flat maps and high power. Likewise the cooling air's temperature drop with the load is NEARLY the
  same on either shaft (to 3 % on `flow/press`), so the metal nearly does not care which shaft carries the generator.
* **On the page's default slam** (1000 → 1400 K, 2 % generator, thrust held): the ends run at 1034.9 / 1421.7 K (HP
  shaft) and 1031.2 / 1421.1 K (LP shaft); with the LP generator both compressors sit further from stall than the
  unloaded engine's at both ends (`φ_LP` 0.794 vs 0.773 at the start).
* **What a hold can carry.** At the 700 K thrust a fixed 700 K carries an HP 0.032; holding the thrust carries 0.10 at
  984 K — three times the fold. On `flow/press` an HP 0.10 lands at 1026.6 K (floor 1021.8).

## 5. Why — the model

Anchor § 1. The hold's rise is the implicit-function slope `−Q_w/Q_T` from rung 92's measured partials, and each
flow coefficient moves by its load response plus its running-line slope times the rise. `φ_L` rises with `Tt4` at
every power, so the hotter burner relieves the LP compressor; `φ_H`'s slope changes sign near 800 K, so the HP relief
fades toward idle. An LP-shaft load barely moves the HP speed (it even raises it on `flow/press`), so holding HP speed
against it needs no hotter burner — or a cooler one.

## What it does to its neighbours

* **Rung 92 — its concession CORRECTED.** A fixed burner temperature OVERSTATES the stall penalty of a generator for
  the LP spool at every power and for the HP spool above idle; at idle the HP's is a wash. Its "same kilowatts, same
  thrust" BOUNDED to flat maps.
* **Rung 93 — its metal put to work.** The metal limit now binds a floating burner (the switch), and the loaded
  engine's cooler cooling air moves the limit slightly.
* **Rung 46 — a limit that is a target.** The temperature limiter cuts fuel at a redline; the hold sets the throttle's
  burner temperature from a thrust, and the metal cap stops it — the redline seen from the steady end.

## Concessions

* **Steady ends, unchanged ramp**: the slam between two held ends is rung 43's fuel ramp; no closed-loop governor
  holds thrust or speed during it (a different rung, in the limiter family).
* **The target is the unloaded engine's at the set throttle** — one choice of what a throttle means; a typed thrust
  would be another.
* The held quantity must rise with the burner temperature (true on every line measured); the perfect gas only.
* Inherited: invented maps, rung 36's imposed stall line, both nozzles choked, Gauntner's 1980 correlation (rung 93).

## The next seam

**A governor in the slam**: hold thrust or speed *during* the move — a closed loop on the fuel, beside rungs 46–52's
limiters, where the hold and the metal cap become two more legs of the min-select.
