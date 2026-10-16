# Rung 95 anchor — A GOVERNOR IN THE SLAM: the throttle holds thrust, speed or burner temperature DURING the move (pre-registered 2026-10-16 — § 3)

**Seam:** rung 94 § *The next seam*: *"A governor in the slam: hold thrust or speed during the move — a closed loop on
the fuel, beside rungs 46–52's limiters, where the hold and the metal cap become two more legs of the min-select."*;
its concession: *"Steady ends, unchanged ramp: the slam between two held ends is rung 43's fuel ramp; no closed-loop
governor holds thrust or speed during it."*

**The user's choices (2026-10-16):** the HP-speed law is BOTH, a toggle — a commanded rate (one time constant) and a
proportional law on a steady table (one gain); a governor that asks for more fuel than the engine can burn is CLAMPED
at the richest runnable mixture and the page SAYS so; on a slowdown it is clamped at the lean limit and says so; the
metal cap (rung 94's switch) cuts the fuel ALL ALONG the move.

**A correction to rung 94's seam sentence (the advisor's, adopted):** the hold is not "another leg of the min-select".
A leg can only CUT fuel, so a thrust leg could never add fuel on an acceleration that is short of it. The governor is
the TOP of the min-select: its fuel replaces the open-loop ramp as the demand that every leg (rungs 46/48/49/60 and the
new metal leg) is bracketed from and cuts. The metal cap IS a new leg.

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung95-probe`, archived in
`docs/plans/scratch-archive/rung95/`) holding a PROTOTYPE governor built from public pieces; it printed REACHABILITY
only — clamp counts, which leg bound, monotonicity, the ends' speeds — never a temperature, a stall margin, a time or a
fuel comparison a claim below is about. § 1–§ 3 written before any governor code existed in the crate.

---

## 0. GIVENS — measured before anything was written

The Controls engine (rung 43's rig: `π_LPC` 3 × `π_HPC` 6, design 1500 K, 250 K / 50 kPa / M 0.85, the perfect gas,
`ρ` 1); the default slam 1000 → 1400 K over 0.5 τ, settling 2 τ, step 0.02; no generator, no lever, unless stated.

**G1 — the held quantities rise with fuel at frozen shaft speeds** (`reach.txt`). At every 5th point of the existing
open-loop march, 41 fuel flows log-spaced across the solvable range: thrust, `Tt4` and the HP acceleration
`dν_H/ds` rise strictly at all 1 066 samples per shape, all four shapes. So each instant inversion has one root.

**G2 — the walls are the closure's fuel-air walls.** The solvable range at frozen speeds ends at `f` = 0.0040–0.0042
(lean, `F_FLOOR` 0.004) and `f` = 0.0650 (rich, `F_CAP`) at every sampled state.

**G3 — reachability of a prototype governor** (`reach.txt`, `reach2.txt`; flow/press and flat-LP):
* default slam, no limiter: the thrust, `Tt4` and HP-speed (rate law, `τ_N` 0.1, 0.5) governors sit on their root at
  every point — **no clamp**; likewise on a 2 τ ramp and on the decel 1400 → 1000 K, except the rate law at `τ_N` 0.1 on
  the 0.5 τ decel: 22 of 126 points on the LEAN wall (14 of the ramp's 26);
* with the redline (1480 K) or the acceleration schedule (margin 0.25) armed, the thrust and fast speed governors ask
  for the RICH wall at 3–44 points — **every one while a leg is cutting** (0 rich clamps with no leg in authority, 8 of
  8 cases): the clamp is a governor out of authority, never a run on the wall;
* the proportional law (13-row steady table, gain `K` 2–200): no clamp on the accel; on the decel the lean wall from
  `K` 20 (14 points) to 200 (30).

**G4 — the ends' HP speeds** (`ends.txt`): `ν_H` 0.790 → 0.959 (flow/press, flat-LP), 0.795 → 0.960 (press/flow), 0.792
→ 0.960 (tilted); `(ν_H,end/ν_H,start)²` = 1.46–1.47.

## 1. THE DEVICE — derived before coding

**The switch** *Govern during the move* (`govern`, default OFF — every saved setting stays valid). On, the quantity
rung 94's `hold` names — burner temperature, thrust, or HP speed — is governed along the whole run.

**The demand** (the advisor's 5): a straight line in the held quantity from the start end's TARGET to the final end's
over the ramp, constant after. The targets are rung 94's: `T_set` for the burner temperature; with the generator on,
the thrust or HP speed the unloaded engine gives at `T_set`; with it off, the end's own thrust or speed. Thrust is read
with the bleed booking (`ControlsSolver::thrust`, rung 62's per-inlet-air thrust when bleeding).

**The laws** (a root in FUEL at the instant's frozen shaft speeds, G1):
* **burner temperature, thrust** — INSTANT inversion `Q(mf; ν) = Q_dem(s)`; no constant (fuel moves `Tt4` and thrust at
  once).
* **HP speed** — a shaft speed cannot be set instantly (inertia), so a law with a constant, the user's toggle:
  * **commanded rate** — the root of `dν_H/ds (mf; ν) = (ν_dem(s) − ν_H)/τ_N`; knob `τ_N` ≥ one step;
  * **proportional** — `mf = W(ν_dem)·[1 + K·(ν_dem − ν_H)]`, `W` the steady fuel at HP speed `ν` read off a 13-row table
    (rung 48's row count) of THIS machine's equilibria between the ends' burner temperatures, the ends exact rows,
    linear between rows; knob `K` (fuel fraction per unit HP speed short).

**The clamp** (the user's): the root is bracketed by stepping out from the open-loop ramp's fuel at `s` (×1.25 up /
×0.8 down, a failed instant bisected to the wall); a demand above what the richest runnable fuel gives takes that fuel,
below the leanest, the leanest — each point records ROOT / RICH / LEAN. The proportional law's fuel is clamped to the
same walls.

**The min-select:** the governor's fuel is the demand (`mf_sched`) every armed leg is solved from and cuts: rung 46's
redline, 48's schedule, 49/60's floor, and the **metal leg** — on rung 94's cap switch, the fuel at which the hotter
metal (rung 93's reading on the instant) reaches its limit, bisected at frozen speeds, armed where the demand's metal is
over. The applied fuel is the minimum. The governor holds no state (no integrator), so a leg's release hands authority
straight back — nothing winds up (G3: the rich clamps are all under a cutting leg).

**Scope** (the advisor's 7): rung 43's plain march with the instant valve and the airflow levers; refused in words
beside rung 47's lag, rung 52's release and rung 88's slow valve (each its own marcher — a governor on them is another
model).

**The metal cap with the governor on** (the advisor's blocker, adopted; it follows from the user's "all along the move",
not a new choice): rung 94 refuses the cap without the generator because with no load nothing floats — but a governed
thrust or speed DOES float the burner along the move with no load. So with `govern` on the cap needs a floating hold
(thrust or HP speed, not the burner temperature), the metal read and cooling — NOT the generator. Each end is capped by
rung 94's bisection even with no load (the hold point is then `T_set` itself, bisected down from there if its metal is
over), so the run starts on an equilibrium the metal leg agrees with. With `govern` off, rung 94's rule stands.

**Where it lives:** `rust/src/governor.rs` — its own march (the plain march's shape, with the governor in each
derivative evaluation), a public derivative the page re-runs for a stop, and the Controls route; nothing else moves.

## 2. REDUCES and GATES (construction — no credit)

* **Off ⇒ the earlier run, bit for bit** (dispatch): every existing gate; the page with `govern` 0.
* **Governing the burner temperature IS rung 40's temperature-commanded march**: on the plain Controls machine the
  governed `Tt4` run and `TwoSpoolTransient::integrate` on the same linear `Tt4` schedule agree at every point — two
  code paths (a fuel root through the machine's fuel closure `r62_try_close_fuel` vs. the temperature closure
  `r62_try_close`, both on the SAME `TwoSpoolTransientCore` through its own hooks — checked); **bar frozen: ≤ 1e−8 in
  both speeds at every point** — exceeded means investigate, not relax. `integrate`'s 0.2 speed floor is inert here
  (speeds ≥ 0.73) and both marches record `n + 1` points.
* **The fixed point is rung 94's end**: run long, the governed march settles on the final end's equilibrium (thrust
  and HP speed held, generator on, both shafts; and generator off).
* **The hold identity**: where the governor is on its root and no leg cuts, thrust (or `Tt4`) equals the demand to the
  root's tolerance, and the rate law's HP acceleration equals `(ν_dem − ν_H)/τ_N` — after the ramp the rate law's HP
  error decays as `exp(−s/τ_N)` exactly. True by construction.
* **The metal leg lands on the limit** where it cuts; the run starts at equilibrium with a capped end.
* **Step check:** the default governed runs at the step 0.02 against 0.005, the flow coefficients' difference reported
  (§ 12.10's class, ~1e−4); the proportional gain's upper bound set where the march at 0.02 stops agreeing.

## 3. PREDICTIONS — frozen here, scored after the build

Cells: the default slam (no generator, no lever, no limiter unless stated), each of the four maps; "open loop" = the
same settings with `govern` off.

**H1 — the thrust governor burns MORE fuel than the ramp all through an acceleration, LESS all through a slowdown.**
At every recorded point with `0 < s ≤ ramp`, governed fuel > open-loop fuel on 1000 → 1400 K, and < on 1400 → 1000 K.
*Derivation:* the open ramp's fuel and thrust both start at the ends; at frozen speeds thrust rises with fuel (G1) and
with speed, so the steady line's thrust gain `ΔF/Δmf` (speed included) exceeds the frozen gain `∂F/∂mf` — to raise
thrust along a straight line before the spools move, fuel must rise faster than the ramp. At the ramp's end the governed
spools still lag the final end (fuel must exceed the end's to give its thrust) while the ramp's fuel equals it. Between,
the governed spools run ahead of the ramp's (more fuel), which works AGAINST the claim — the risk. 8 cells.

**H2 — so the governed acceleration is hotter and nearer stall:** its peak `Tt4` is above the open loop's, and its
minimum flow coefficient lower on BOTH compressors. *Derivation:* H1 and rungs 35/45 (more fuel ⇒ larger `Tt4`
overshoot and larger surge approach, on both spools — the LP's minimum early in the ramp, the HP's late). 4 maps × 3 = 12.
**Vacuity rule (frozen):** a flow-coefficient cell counts only if BOTH runs' minima are interior (index > 0); a minimum
at the start point is shared and scores VACUOUS, not HIT. The count of interior cells is reported.

**H3 — the thrust governor's hottest moment is the END of the ramp** (the recorded point at `s` = ramp, index 25, ±1).
*Derivation:* at fixed thrust, `Tt4` falls as the spools speed up; after the ramp the demand stops and the spools keep
gaining, so `Tt4` only falls; during the ramp the demand rises at a constant rate while the spools lag. Risk: a spool
accelerating fast enough mid-ramp could peak `Tt4` early. 4 cells.

**H4 — a fixed proportional gain closes the HP speed gap SLOWER at low power than near full power**; the commanded rate
closes it at `τ_N` at every power (identity, § 2). Small steps 990 → 1000 K and 1390 → 1400 K (ramp one step, settle
3 τ), `K` 20: the time after the ramp for the HP error to fall to `1/e` of its ramp-end value is LONGER at 1000 K.
*Derivation:* the loop gain is `K·W·∂(dν_H/ds)/∂mf` — the HP shaft's power surplus per fractional fuel change, over its
speed. Turbine power scales roughly as the cube of speed, so the gain as `ν_H²`, smaller at idle. The closed loop's rate is that gain
PLUS the plant's own relaxation at fixed fuel (the shaft returns to equilibrium by itself), which is also slower at idle —
so the sign holds and the estimate below is incomplete. Estimate (reported, not scored): `τ(1000)/τ(1400)` ≈
`(ν_H,1400/ν_H,1000)²` ≈ 1.47 (G4). **Scored at the step 0.005** (the ramp then four steps), the `1/e` crossing
interpolated in log of the error; the step 0.02 is a cross-check only. 4 cells, sign scored.

**H5 — with the redline on (1480 K), the governor still gets the thrust sooner:** the governed run reaches 95 % of the
thrust step earlier than the open loop with the same redline. *Derivation:* H1 while the governor holds the fuel; while
the redline holds both, the governed spools are ahead, and at the same `Tt4` faster spools give more thrust. 4 cells.
The crossing time is interpolated linearly between recorded points; a run that never reaches 95 % scores the cell
VACUOUS.

**H6 — the commanded rate trades time for temperature:** the governed acceleration's peak `Tt4` FALLS as `τ_N` rises,
over `τ_N` ∈ {0.05, 0.1, 0.2, 0.5}. *Derivation:* a longer `τ_N` asks a smaller HP acceleration at the same state ⇒ less
fuel (G1); the states differ, so monotone is the claim, not a slope. 4 cells.

## 4. What would change the plan

A MISS is reported as a finding (the sandbox rule). A governor that stops a run (a failed root inside the clamp) is a
reported stop with its reason, like every other route's. **Cost** (the advisor's): each derivative now brackets and
solves a root under the legs; one default governed run (redline and schedule armed) is timed natively and in the
browser build BEFORE the page is built — at 5–10× the open loop's slowest request, the bracketing is tightened first.
