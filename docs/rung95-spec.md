# Rung 95 — A GOVERNOR IN THE SLAM: the throttle holds thrust, HP speed or burner temperature during the move

**Code:** `rust/src/governor.rs` (the governor `Governor` / `Law`, its fuel `try_governor_fuel` with the clamp `Clamp`,
the metal leg `try_metal_fuel`, one derivative `try_der`, the step `try_rk_step`, the march `march`, the proportional
law's table `try_steady_table`); the Controls view's route (`sandbox_controls::{build, Route::Governor}`) · **Gates:**
`rust/tests/rung95.rs`, the page's `rust/tests/sandbox_governor.rs` · **Panel:** `print_governor_table` (born in Rust,
held to `rust/oracle/rust_owned/`, a change detector only) · **Anchor (the device and every bar, committed `bc14046`
before any governor code existed):** `docs/plans/rung95-anchor-governor.md` · **Sandbox:** the Controls view, *Govern
during the move* (`docs/plans/sandbox-plan.md` § 24) · **Scratch scripts and outputs:**
`docs/plans/scratch-archive/rung95/`.

**Shipped as KNOBS** (the sandbox direction): predictions were registered first and are scored here; a miss is a
finding.

Rung 94 closed by naming this seam: a closed loop on the fuel that holds thrust or speed *during* the move, beside rungs
46–52's limiters. The user's choices (2026-10-16): the HP-speed law BOTH, a toggle (a commanded rate, or proportional on
a steady table); a governor that asks for more fuel than the engine can burn — or less than it can run on — is CLAMPED
there and the page says so; the metal cap cuts the fuel all along the move.

> **THE HEADLINE — a governor needs its limiters: holding thrust along a fast move makes the slam HOTTER, not
> gentler.** To raise thrust in a straight line before the spools have moved, the fuel must rise faster than the open
> ramp — the steady line's thrust gain includes the speed the spools have not yet gained — so the governor burns more
> fuel than the ramp at EVERY point of an acceleration (and less at every point of a slowdown). On the default slam
> (1000 → 1400 K in half an HP spool time) the turbine peaks near **1 930 K** against the ramp's ~1 700 K, over 500 K
> above where it settles, its hottest moment exactly at the end of the move, and both compressors come nearer stall. It
> gets the thrust there by construction (95 % at 0.48 τ against the ramp's 1.39 τ) — and with the 1480 K temperature
> limiter on, which then holds the fuel for most of the move, it STILL gets there about 18 % sooner.
>
> **A SPEED GOVERNOR'S GAIN IS NOT ITS SPEED.** Held to a proportional law, a fixed gain closes the HP speed gap about
> **1.9× more slowly at idle than near full power** (the turbine has less power to spare there, and the shaft's own
> settling is slower too); the commanded-rate law closes it at its own time constant at every power (by construction) —
> one reason a control may schedule an acceleration rather than a gain (not measured here).

## 1. What it adds

**The governor is the TOP of the min-select** (the anchor's correction to rung 94's seam sentence): a leg can only cut,
so the hold could never be one. The governor's fuel replaces the open-loop ramp as the DEMAND every armed leg — rung
46's redline, 48's schedule, 49/60's floor and the new metal leg — is solved from and cuts.

**The demand** — a straight line in the held quantity from the start end's target to the final end's over the ramp,
constant after; the targets are rung 94's (`T_set`, or the unloaded engine's thrust or HP speed at `T_set` with the
generator on, else the end's own). **The laws**, each a root in fuel at the instant's frozen shaft speeds (anchor G1:
the held quantities rise strictly with fuel there): the burner temperature and thrust by INSTANT inversion; the HP speed
by a COMMANDED RATE `dν_H/ds = (ν_dem − ν_H)/τ_N` or PROPORTIONALLY, `mf = W(ν_dem)·[1 + K(ν_dem − ν_H)]` on a 13-row
table `W` of the machine's equilibria (the ends exact rows).

**The clamp** — the root is bracketed by stepping out from the ramp's fuel; a demand beyond the richest runnable fuel
takes it, below the leanest the leanest (the closure's fuel-air walls 0.065 / 0.004, G2), and each point records which.
The governor holds no state, so a leg's release hands it the fuel straight back; it was clamped RICH only while a leg
was cutting in every case checked (anchor G3's eight; `tests/sandbox_governor.rs` holds it on the redline run). **The metal leg** — on rung 94's cap switch, the fuel at which the hotter
metal reaches its limit; with the governor on it needs a floating hold, the metal read and cooling, not the generator
(each end then capped by rung 94's bisection even with no load).

**Scope:** rung 43's plain march with the instant valve and the airflow levers; refused in words beside rung 47's lag,
52's release and 88's slow valve (each its own marcher). On the page: the switch *Govern during the move*, the speed
law and its knob (`τ_N` ≥ 0.04 τ, gain ≤ 50 — both set by the step check, § 4), the demand drawn on the chart of the
held quantity, a strip marking the clamps, and the table's rows.

## 2. Reduces

Off, the Controls run is rung 43's march, bit for bit (dispatch; no earlier gate, oracle or golden moved). **Governing
the burner temperature IS rung 40's temperature-commanded march** on the same machine — two code paths, a fuel root
through `r62_try_close_fuel` against the temperature closure `r62_try_close` — agreeing to **6e−16** in both speeds on
all four maps (bar frozen at 1e−8). Run long, every governed march settles on rung 94's end (speeds to ≤ 3e−7, thrust
to ≤ 1e−7 relative), and where the governor is on its root with no leg cutting it holds its demand to ≤ 8e−15 (true by
construction — no credit). The design run is rung 6.

## 3. Scoring (anchor § 3 — bars frozen at `bc14046`)

The default slam (1000 → 1400 K over 0.5 τ, settle 2 τ, no generator, lever or limiter unless stated), each of the
four maps; "open" is the same run with the governor off.

| # | verdict | measured |
|---|---|---|
| H1 more fuel all through an acceleration, less all through a slowdown | **HIT** 8/8 | governed/open fuel − 1 over the ramp's 25 points: +0.056…+0.361 (flow/press), +0.058…+0.373, +0.059…+0.371, +0.060…+0.365 accelerating; −0.541…−0.027, −0.555…−0.028, −0.550…−0.028, −0.528…−0.031 slowing (flow/press, press/flow, tilted, flat-LP) |
| H2 hotter and nearer stall, both compressors | **HIT** 12/12 (all 8 minima interior — none vacuous) | peak `Tt4` 1927 / 1940 / 1932 / 1899 K against 1695 / 1703 / 1702 / 1691; lowest `φ_LP` 0.714 / 0.722 / 0.713 / 0.665 against 0.735 / 0.747 / 0.737 / 0.687; lowest `φ_HP` 0.832 / 0.804 / 0.816 / 0.833 against 0.861 / 0.841 / 0.849 / 0.862 |
| H3 hottest at the end of the move | **HIT** 4/4 | the peak at point 25 (`s` = 0.5) on every map; no clamp |
| H4 a fixed gain is slower at idle | **HIT** 4/4 (sign) | `K` 20, `1/e` time after a 10 K step at step 0.005: 0.155 / 0.147 / 0.151 / 0.152 τ at 1000 K against 0.082 / 0.077 / 0.079 / 0.080 at 1400 K — ratio **1.90** (all maps); step 0.02 agrees to 1e−5. The anchor's `ν_H²` estimate (1.47, reported, not scored) is short, as it said it would be: it leaves out the shaft's own settling |
| H5 with the redline, still sooner | **HIT** 4/4 | 95 % of the thrust step at 1.323 / 1.330 / 1.328 / 1.226 τ governed against 1.608 / 1.606 / 1.607 / 1.486 open |
| H6 the commanded rate trades time for temperature | **HIT** 4/4 | peak `Tt4` at `τ_N` 0.05 / 0.1 / 0.2 / 0.5: 2047 / 2023 / 1934 / 1652 K (flow/press), the same order on every map |

All six were derived before the build (anchor § 3); none was visible to the reachability probe (anchor § 0 printed
clamp counts, monotonicity and the ends' speeds only).

## 4. After scoring (reported, not scored)

* **The step check set two knobs' limits.** At the page's step 0.02 against 0.005: thrust 8e−8 and the commanded rate at
  0.1 τ 1e−6 in flow coefficient; the proportional law 3.5e−6 at `K` 20, 5e−5 at 50, 1e−3 at 100 and 2e−2 at 200 — so
  the gain stops at **50**; the commanded rate 1.1e−3 at one step (0.02 τ), under 1e−4 from two steps on every
  acceleration — so `τ_N` starts at **0.04 τ**.
* **On the lean edge the run is rougher.** A fast commanded rate on a slowdown sits on the lean wall, and there the march
  at 0.02 agrees with 0.005 only to ~1e−3 in flow coefficient — erratic in `τ_N`, not shrinking with it (0.04: 1.1e−3,
  0.06: 4.9e−4 on one map). That wall is the fuel closure's search bound, which is not smooth in the shaft speeds; the
  page marks every point there and says so. The proportional law there: ≤ 3e−4 at `K` 50.
* **Cost.** A governed run costs about 3.5× the open loop (0.49 s natively with the redline and the schedule armed,
  against 0.14 s); the crash map's median request 0.10 s, slowest 1.35 s.
* **The crash map** (1 000 governed requests, seed 12, every hold and law, random limiters, levers, generator, cooling
  and cap, both ends over the slider range, `scratch-archive/rung95/crash.txt`): **no panic.** 536 ran to the end (60
  of them on the lean edge somewhere, 60 on the rich), 66 stopped early with a reason (64 the stall floor out of reach,
  2 the temperature limiter's own refusal — a throttle already above its redline, which the open loop meets the same
  way), the rest refused in words (135 the cap without cooling, 56 the cap on the burner-temperature hold, 8 the
  schedule's table, the others an end with no steady point or a hold the load cannot meet — rung 94's words).
  **Its first run found 3 stops of the governor's own**: on a deep slowdown (≈1650 → 620 K) the governor's search found
  no runnable fuel to start from — the ramp's too lean for shafts still spinning fast, the start's too rich. A fallback
  scan (two decades around the ramp, nearest first, reached only where the three first tries fail — every scored run is
  unchanged) now finds one; two of the three run to the end, one stops at its stall floor. Gated by
  `a_deep_slowdown_finds_a_runnable_fuel_to_start_the_governors_search_from`.
* **The panel** (flow/press): the commanded rate (0.1 τ) and the proportional law (`K` 20) on HP speed run hotter still
  — 2023 and 2100 K — and reach 95 % thrust at 0.40 and 0.38 τ; with the redline they sit at the rich edge for 24 and 34
  points (the redline cutting) and reach 95 % at 1.29–1.30 τ.

## 5. Why — the model

Anchor § 3's derivations. At frozen speeds thrust rises with fuel (`∂F/∂mf`) and with speed; along the steady line the
thrust gain `ΔF/Δmf` carries the speed too, so it exceeds the frozen gain — a straight line in thrust, asked before the
spools respond, needs fuel rising faster than a straight line in fuel. At the end of the move the governor needs more
than the end's fuel (its spools still lag) while the ramp has exactly that, and after it the held thrust needs less
`Tt4` as the spools gain, so the peak sits at the move's end. More fuel ⇒ the larger `Tt4` overshoot and surge
approach of rungs 35/45, on both spools. The proportional law's loop gain is `K·W·∂(dν_H/ds)/∂mf`, the HP shaft's power
surplus per fractional fuel change over its speed — smaller at idle — plus the shaft's own relaxation at fixed fuel, also
slower there.

## What it does to its neighbours

* **Rung 94 — its seam sentence CORRECTED.** The hold is the TOP of the min-select, not "another leg": a leg can only
  cut. The metal cap IS a leg — and with the governor on it no longer needs the generator (a governed thrust floats the
  burner with no load).
* **Rungs 46–52 — the limiters become the control.** Under a thrust governor the 1480 K temperature limiter holds the
  fuel for about half the default run (67 of 126 points in the anchor's reachability probe, on flow/press); whether rung 46's split — relief on the late HP spool, none on the early LP — survives a demand
  that is itself the hot thing is not measured here.
* **Rung 43/35 — the TIT overshoot belongs to the open RAMP, and a governor can enlarge it.** Governing the burner
  temperature has none — by construction (a commanded `Tt4` cannot overshoot itself; no finding), at the price of the
  slowest thrust; governing thrust enlarges it (H2, the finding).
* **Rung 40 — its temperature-commanded march is now a SPECIAL CASE** of a fuel governor (the burner-temperature hold),
  to 6e−16.

## Concessions

* **The governor reads thrust directly** — a real engine infers it (fan speed, pressure ratio); the stateless laws have
  no integrator (no wind-up to model, and none of a real PI loop's overshoot).
* **Plain march only:** refused beside rungs 47/52/88's lagged marchers.
* **The clamps are the closure's search walls** (fuel-air 0.004 / 0.065), not a flame-out or a pump limit; on the lean
  wall the march is ~10× rougher than elsewhere (§ 4).
* The demand is a straight line in the held quantity — one choice of what a throttle moving means.
* Inherited: invented maps, rung 36's imposed stall line, both nozzles choked, the perfect gas.

## The next seam

**A governor that infers thrust** — from fan speed or a pressure ratio, as a real control does — and the gap between the
inferred and the true thrust during the move; or **the governor on the lagged marchers** (47/52/88), where its demand
meets a lag's state.
