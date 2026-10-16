# Rung 94 anchor — A BURNER THAT CARRIES THE LOAD: each throttle holds thrust or HP speed (pre-registered 2026-10-16 — § 3)

**Seam:** rung 92 § *The next seam*: *"a burner that carries the load — a control that holds thrust (or speed) and
lets `Tt4` rise to pay for the generator"*; rung 92's concession: *"Fixed burner temperature. A real control holds
speed or thrust and burns hotter to carry the load; at a fixed `Tt4` the surge effects here are the load's alone and
understate what a real engine sees at idle."*

**The user's choices (2026-10-16):** the seam is two rungs (rung 93 shipped the metal limit); here each throttle end
holds **thrust or HP speed, a switch**, and the burner temperature floats; the fuel ramp between the two ends is
unchanged (a closed-loop governor in the march would be a new limiter rung — not chosen). **The metal limit:** REPORTED
by default (the burner floats as hot as the hold needs, the metal's margin shown), with a SWITCH that makes it BIND (the
burner stops at the limit; the thrust or speed shortfall is the result, shown).

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung94-probe`, archived in
`docs/plans/scratch-archive/rung94/`), on EXISTING code only — rung 92's loaded matcher; no hold code existed. § 1–§ 3
written before any hold number was computed by hold code.

---

## 0. GIVENS — measured before anything was written

Rig A (rungs 42/90–93's): CPG, `π` 3 × 6, design 1500 K, 250 K / 50 kPa / M 0.85, flat and `flow/press` maps; loads
are rung 92's fixed POWER `w` (× the design HP compressor power, one unit on both shafts).

**G1 — the unloaded line and its slopes** (`givens.txt`). Thrust 732 N (1500 K) → 89 N (700 K); `d ln F/dTt4` 1.68e−3
(1500) → 5.23e−3 /K (700); `dN_H/dTt4` 3.45e−4 → 4.91e−4 /K (flat; HP speed design = 1); `d ln φ_L/dTt4` +6.35e−4
(1500) → +4.11e−4 (700) — always positive; **`d ln φ_H/dTt4` +1.85e−4 at 1500 K, crossing zero near 800 K (flat; 760 K
on `flow/press`) and negative below** (−9.35e−5 at 700 K).

**G2 — a fixed power at a fixed `Tt4`, per unit `w`** (Richardson 0.01/0.02). Thrust: `d ln F/dw` −1.60 (1500) → −10.6
(700) — the same on either shaft to 0.4 % on FLAT maps, but on `flow/press` the HP shaft costs 11–12 % MORE thrust per
unit load at 1000 and 700 K (−4.62 vs −4.16, −12.18 vs −10.83): rung 92's "the same kilowatts cost the same thrust" was
measured flat and at higher power, and is BOUNDED by this. HP shaft: `dN_H/dw` −0.50
→ −1.71, `d ln φ_L/dw` −1.40 → −4.75, `d ln φ_H/dw` −0.90 → −2.23 (flat). LP shaft: `dN_H/dw` +0.004 (flat), +0.105 → −0.017
(`flow/press`): an LP-shaft load barely moves — or slightly RAISES — the HP speed. **`dTt3/dw` ≈ −303 K at 1500 K for
either shaft** (−713 at 700 K): NEARLY the total load only — the two shafts agree to 6e−5 relative on flat maps, but
differ by up to 3 % on `flow/press` at finite load (a fixed power is a load per kg of air, and the airflow moves
differently for each shaft). So the metal NEARLY does not care which shaft carries the generator.

**G3 — finite loads, curvature, and the fold** (`curv.txt`). The responses curve: at 1000 K an HP 0.05 cuts `ln F` by
0.268 (linear 0.208). The largest fixed power carried at a fixed `Tt4` (rung 92's fold): HP 0.032 (700 K) → 0.105 (1000)
→ 0.230 (1500); LP 0.031 → 0.060 → 0.146 (flat); `flow/press` slightly less. At 1000 K an HP 0.10 still leaves 104 N
(flat), above the 89 N of the unloaded 700 K point.

## 1. THE DEVICE — derived before coding

**The target** (the advisor's trap 1): at a throttle set to `T_set`, the hold's target is the ABSOLUTE thrust (or HP
speed) THIS engine gives at `T_set` with the generator OFF and every other demand as set; the burner temperature `T`
then solves `Q(T; load) = Q(T_set; no load)` on the loaded engine. Absolute thrust, not per kg of air: the airflow moves
with `T`. **Holding the burner temperature is the default and IS rung 92** (dispatched); a hold with no active load IS
the burner-temperature hold (dispatched — the target is met at `T_set` itself).

**The search** (traps 4): bisection in `T` on `Q(T; load) − Q*`. Its LOW end is the lowest `T` the loaded engine solves
at — found on a 50 K grid from the bottom of the range, then BISECTED between the last failing and first solving grid
points (G5: an HP 0.10 solves from 974 K, where thrust is 80 N, but not at 950 K; a grid alone would start at 1000 K,
104 N, above an 89 N target — rung 93's knob bug again). It reaches BELOW `T_set` when the hold needs a cooler burner (an
LP-shaft load holding HP speed on `flow/press`). Refused in words, NAMING the bound, when the target lies below the least
thrust (or speed) the load allows (G5: an LP 0.05 cannot run below 143 N) or above the range's top.

**The metal** (trap 2): rung 93's reading on a LOADED point — the load lowers the cooling air's temperature (G2), so the
loaded limit differs from rung 93's. The steady body is rung 92's (`OfftakeResult`), so the reading is
`MetalInputs::of_shares` on its temperatures and shares, and the binding cap is a bisection of the loaded engine's metal
excess in `T`. **Reported** by default; **binding** on the switch: the burner stops at `min(T_hold, T_metal)` and the
shortfall `Q(T; load)/Q* − 1` is reported. A cap with NO cooling clamps everything (uncooled, the 1985 metals allow
about 989 K — below the page's default throttles), so it is refused in words (trap 3).

**The transient** (the user's choice): only the two ENDPOINTS move — each is the hold's `T` on the march's own machine
(rung 62's equilibrium, ≡ the steady match by rung 92's plant gate); the fuel ramps between their fuel flows as before.

**Where it lives.** A wrapper around rung 92's matcher and the Controls endpoints; the design run, every matcher and
every march untouched. The perfect gas only (rung 92's).

### Exact consequences of § 0 (GATES, no credit — the implicit-function identity)

At vanishing load the hold's temperature rise is `dT/dw = −Q_w/Q_T`, and each flow coefficient moves by
`d ln φ/dw|hold = d ln φ/dw|T + (d ln φ/dT)(dT/dw)`. From G1/G2 (flat, HP shaft): holding thrust, `dT/dw` = +954 K at
1500 K, +2 024 at 700 K; the LP compressor's loss falls −1.40 → −0.79 at 1500 K (`φ_L` rises with `T`), and the HP
compressor's correction changes sign with `d ln φ_H/dT` — RELIEF above ~800 K, an extra BURDEN below. Holding HP speed:
`dT/dw` = +1 437 K (HP shaft, 1500 K); with an LP-shaft load −11 K (flat) to −258 K (`flow/press`) — a COOLER burner.
The device's zero-load readings are gated against these.

### The model (the prediction instrument)

Linear in the hold: `ΔT = −Q_w w/Q_T`; each `Δ ln φ = φ_w w + φ_T ΔT`; every partial from § 0 at the base `T0`. **Its bar
is its own curvature**: the quadratic root with all second partials measured (`Q_TT` along the line, `Q_ww` from three
loads, `Q_Tw` from `Q_w` at `T0 ± 10 K`), `bar = max(3 |ΔT_quad − ΔT_lin|, 1 K)`. Reachability: rung 92's matcher solves at
every predicted `(T0 + ΔT, w)` below (`model.txt`).

Model, holding THRUST (`ΔT` K, the bar; `Δ ln φ_L`, `Δ ln φ_H` under the hold, against FIXED `T`):

| map, shaft | `T0`, `w` | ΔT (bar) | Δln φ_L hold / fixed | Δln φ_H hold / fixed |
|---|---|---|---|---|
| flat, HP | 1500, 0.05 | +47.7 (5.2) | −0.0396 / −0.0699 | −0.0363 / −0.0452 |
| | 1000, 0.05 | +78.3 (36.2) | −0.1000 / −0.1463 | −0.0745 / −0.0830 |
| | 800, 0.02 | +38.1 (9.5) | −0.0622 / −0.0812 | −0.0412 / −0.0413 |
| | 700, 0.02 | +41.5 (17.4) | −0.0790 / −0.0960 | −0.0487 / −0.0448 |
| flat, LP | 1500, 0.05 | +47.6 (4.8) | +0.0257 / −0.0045 | +0.0240 / +0.0152 |
| | 1000, 0.05 | +78.4 (34.3) | +0.0649 / +0.0186 | +0.0188 / +0.0103 |
| | 700, 0.02 | +41.6 (17.1) | +0.0541 / +0.0370 | −0.0070 / −0.0031 |
| flow/press, HP | 1500, 0.05 | +47.7 (6.7) | −0.0293 / −0.0518 | −0.0242 / −0.0301 |
| | 700, 0.02 | +45.6 (24.7) | −0.0741 / −0.0914 | −0.0370 / −0.0349 |
| flow/press, LP | 1500, 0.05 | +47.6 (6.2) | +0.0190 / −0.0034 | +0.0160 / +0.0101 |
| | 700, 0.02 | +40.1 (15.4) | +0.0519 / +0.0366 | −0.0034 / −0.0015 |

Holding HP SPEED (`ΔT` (bar), `Δ ln F`): flat HP 1500/0.05 +71.8 (2.3), +0.041; 700/0.02 +70.6 (11.3), +0.152; flat LP
1500/0.05 −0.5 (1.0), −0.081; `flow/press` HP 1500/0.05 +98.0 (7.3), +0.084; `flow/press` LP 1500/0.05 −12.9 (5.8),
−0.101, 1000/0.05 −7.5 (10.9), −0.229.

**What it says, before any hold code.**
1. **Carrying the load relieves the compressors.** Holding thrust RELIEVES the LP compressor at every power (it sits
   further from stall than at a fixed burner temperature), and the HP compressor too at 1000–1500 K; at idle the HP
   compressor's correction changes sign at zero load (below ~800 K) but is a wash at a real load (G4). So rung 92's "a
   fixed burner temperature understates the surge effects at idle" is WRONG for the LP spool at every power, and for the
   HP spool above idle.
2. **A generator on the LP shaft becomes a stall-margin GAIN** once the burner carries it: holding thrust, both
   compressors move AWAY from stall at 1000–1500 K (the LP at every power).
3. **Holding HP speed is not holding thrust.** With an HP-shaft load the speed hold burns 50–110 % hotter than the
   thrust hold and RAISES thrust (+4 % at 1500 K, +16 % at 700 K for these loads); with an LP-shaft load it barely moves
   the burner (flat) or COOLS it (`flow/press`), and the thrust falls by more than at a fixed burner temperature.
4. **Holding thrust carries loads far past the fixed-temperature fold — on the HP shaft.** At the 700 K thrust demand
   (89 N, flat) a fixed 700 K carries an HP load of 0.032; the burner that carries the load holds that thrust with an HP
   0.10 (G5: between 974 and 1000 K). An LP-shaft load cannot: its thrust floor rises past the demand (143 N at 0.05).

**G4 — the 700 K HP-compressor shift, evaluated** (`check2.txt`; the advisor's pre-freeze check, no credit). On rung
92's matcher at `T0 + ΔT_lin`, holding thrust against a 0.02 load moves the HP flow coefficient by −0.0015 (flat, HP
shaft), −0.0013 (flat, LP), +0.00001 (`flow/press`, HP), −0.0001 (`flow/press`, LP) relative to the fixed `T0`: the
zero-load sign survives only weakly on flat maps and not at all on `flow/press` — the temperature slope and the
load's cross term both pull it to zero. At 1000 and 1500 K the hold's relief of the HP compressor is about twice the
linear estimate (flat HP 1000 K: +0.0197 vs +0.0085).

**G5 — the least thrust a load allows** (`check2.txt`): the lowest `Tt4` that carries a fixed power, and the thrust
there (flat): HP 0.05 → 763 K, 58 N; HP 0.10 → 974 K, 80 N; LP 0.05 → 913 K, 143 N; LP 0.10 → 1275 K, 352 N
(`flow/press` alike within a few %, HP 0.10 → 1022 K, 77 N). **An LP-shaft generator raises the engine's thrust FLOOR
far more than the same power on the HP shaft** (143 against 58 N at 0.05): the LP spool reaches its own fold first.

## 2. THE READINGS

**Steady (rig A):** the device's `T` for each hold at the cells above; `Δ ln φ_L`, `Δ ln φ_H`, `Δ ln F` there; and the
largest load each hold carries at the 700 K demand. **Zero-load gates:** the device's Richardson readings (w 0.01/0.02)
against the identities, flat and `flow/press`, both shafts, both holds, 700–1500 K. **The slam (reported, not
scored):** the Controls default slam with an HP 0.02 generator and its ends holding thrust — the hotter ends, the
overshoot, the closest approach to stall, the metal against its limit.

## 3. PREDICTIONS (frozen here; scored in the spec)

**Construction (gated, no credit):** the reduce (the burner-temperature hold ≡ rung 92 bit for bit; no load ⇒ `T_set`
exactly); the zero-load identities above (to the Richardson reading's own error); every hold root meets its target to
1e−9 relative; the metal cap's endpoint sits at its limit; **the search reaches G5's brackets** — holding the 700 K thrust
(flat), an HP 0.10 settles between 974 and 1000 K, and an LP 0.05 is refused in words naming its least thrust (143 N);
refusals in words (a cap with no cooling).

- **H1 (thrust hold, magnitude).** The device's `ΔT` within the table's bar at each of the 11 thrust-hold cells.
- **H2 (the LP relief survives).** At every thrust-hold cell the LP compressor sits further from stall than at the
  fixed burner temperature (`Δ ln φ_L` hold > fixed); the HP compressor too at 1500 and 1000 K. (At 700 K the HP
  compressor's sign is G4's — too weak to score; reported.)
- **H3 (the LP generator becomes a gain).** Holding thrust, an LP-shaft 0.05 moves BOTH compressors away from stall
  (`Δ ln φ_L`, `Δ ln φ_H` > 0) at 1500 and 1000 K, both maps.
- **H4 (speed hold, signs and size).** Holding HP speed: with the HP-shaft loads thrust RISES and the burner runs hotter
  than the thrust hold's; with the LP-shaft 0.05 on `flow/press` the burner runs COOLER than `T0` at 1500 K
  (−12.9 ± 5.8 K) and thrust falls by more than at the fixed `T0`; the `ΔT`s within the quoted bars.

**The likeliest miss, named in advance:** H1 at 1000 K and at 700 K, where the bar is set by a curvature that grows
toward the fold — the quadratic estimate may itself be off there (G4 showed second order doubling a first-order shift
at 1000 K); then H4's flat LP cell, where the predicted `ΔT` is smaller than its 1 K floor's bar.
