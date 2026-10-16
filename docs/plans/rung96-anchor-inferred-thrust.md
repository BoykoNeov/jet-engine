# Rung 96 anchor — A GOVERNOR THAT INFERS THRUST: the throttle holds a READING (pressure ratio or LP speed) the control turns into thrust (pre-registered 2026-10-16 — § 3)

**Seam:** rung 95 § *The next seam*: *"A governor that infers thrust — from fan speed or a pressure ratio, as a real
control does — and the gap between the inferred and the true thrust during the move"*; its concession: *"The governor
reads thrust directly — a real engine infers it (fan speed, pressure ratio)."*

**The setup (2026-10-16):** proposed to the user, who answered "go":
* both readings, a toggle: the engine pressure ratio (turbine-exit over inlet total pressure — `EPR`, the reading of
  the Pratt and Rolls-Royce controls) or the LP shaft speed (`N1`, the fan speed of the General Electric/CFM controls);
* the throttle still asks for a straight line in THRUST (rung 95's demand), which a steady table turns into the
  reading's demand, so the run compares like for like with rung 95's true-thrust governor;
* the LP speed governor reuses rung 95's two speed laws (commanded rate / proportional) on the other shaft;
* a switch: the control's table built on the CLEAN engine (generator off) or on the loaded one.

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung96-probe`, archived in
`docs/plans/scratch-archive/rung96/`) holding a PROTOTYPE governor built from public pieces. It printed monotonicity,
identities, clamp counts and which leg bound. **Disclosed:** its clamp counts were visible before § 3 was written.
Under the redline, the pressure-ratio governor asked for the rich wall at 21–24 points, against the true-thrust
governor's 3–5. On the 0.5 τ slowdown it asked for the lean wall at 5–6 points, against 0. That points the same way as
H2. No temperature, stall margin, time or thrust gap was printed. § 1–§ 3 were written before any governor code existed
in the crate.

---

## 0. GIVENS — measured before anything was written

The Controls engine is rung 95's: rung 43's rig, the default slam 1000 → 1400 K over 0.5 τ, settling for 2 τ, step
0.02, flying at M 0.85.

**G1 — both readings rise with fuel at frozen shaft speeds** (`reach.txt`, `reach2.txt`). The check ran at every 5th
point of the open march, accelerating and slowing, with 41 log-spaced fuel flows across the solvable range. `EPR` and
the LP acceleration `dν_L/ds` rise strictly at all 1 066 samples per shape, on all four shapes. The same holds on six
modified engines:
* the generator on the HP shaft or the LP shaft (power 0.02);
* turbine cooling (0.05 / 0.03);
* the customer bleed (0.05);
* the bleed lever;
* the stator lever.

So the `EPR` inversion has one root, and the LP commanded rate has one root.

**G2 — equal pressure ratio ⇒ equal gross thrust, exactly** (`reach.txt`, `reach2.txt`). Gross thrust here is net
thrust plus `V0` × the air the inlet swallows, in Controls thrust units. The check took two states with different shaft
speeds and solved the fuel at which the second matches the first's `EPR`. Their gross thrusts agree to ≤ 5.5e−15
relative (19–24 pairs per cell, every engine above), and to ≤ 9.5e−14 with turbine cooling.

*Why:* both nozzles are choked and the gas is perfect. The convergent nozzle's gross thrust is
`A9·[(1+γ)·p9 − p0]` with `p9 = pt9·(2/(γ+1))^(γ/(γ−1))`, so it depends on the nozzle pressure alone, and `Tt9`
cancels. **This is construction, not a finding.** A pressure-ratio governor reads gross thrust exactly. The whole gap
between the thrust it infers and the true thrust is the RAM DRAG it cannot see: `V0` × (the airflow the table expects
at that `EPR` − the airflow the engine actually swallows).

**G3 — gross thrust is AFFINE in `EPR`.** The line through the 13-row steady table's end rows reproduces the gross
thrust at every point of the open march to ≤ 1.2e−15 relative (≤ 9.9e−14 cooled). Linear interpolation in the table
is therefore exact for the gross part. This sets the decomposition gate's bar (§ 2).

**G4 — the steady tables rise.** On every engine of G1, on THIS machine and on the generator-off (clean) machine, the
13 rows ordered by thrust rise in `EPR`, in `ν_L` and in fuel. So thrust → reading → thrust inverts uniquely, and the
LP proportional law's steady fuel `W(ν_L)` is single-valued.

**G5 — the `EPR` pt5 is the tail's own.** Every transient thrust tail (rung 40's, 62's bleed, 90's customer, 91's
cooling, 92's offtake through rung 40's hooks) builds its nozzle inlet as `pt = π_LPT·π_HPT·pt4`, from the same
`Instant2` fields. So `EPR = π_LPT·π_HPT·pt4 / pt2`, with `pt2` the inlet's (`TwoSpoolTransientCore::inlet`).

**G6 — reachability of a prototype** (`reach.txt`; flow/press and flat-LP). The `EPR` governor and the LP-speed
governor (rate law at `τ_N` 0.1 and 0.5, proportional at `K` 20) run to the end on every case. The cases: the default
slam, the redline (1480 K), the acceleration schedule, a 2 τ ramp, and the 0.5 τ and 2 τ slowdowns. Every nozzle point
is choked. Rich clamps happen only under a cutting leg, as in rung 95.

## 1. THE DEVICE — derived before coding

**The switch** *Thrust read from* (`thrust_by`): `true` (the default — rung 95's governor, which reads thrust itself),
`epr`, or `n1`. It applies when the move is governed and the throttle holds thrust; otherwise it is ignored, as rung
95's `gov_tau` is ignored off the speed hold. Every saved setting stays valid.

**The demand.** The demand is rung 95's straight line in THRUST from the start end's target to the final end's over the
ramp, constant after. The control turns it into the READING's demand through its **table**: 13 rows (rung 48's count)
of steady points between the two table ends' burner temperatures, the ends exact rows. Each row stores thrust, `EPR`,
`ν_L`, fuel and face airflow, and values are read linearly between rows (G4: invertible).

**Which engine the table is built on** (the switch *Control calibrated on*, `table_clean`, default ON). With the
generator off it has no effect: the clean engine IS the engine.
* **ON:** the table is built on the generator-off machine between the two set throttles, `T_set`. These are rung 94's
  target ends, so the table's thrust at each end is the target exactly.
* **OFF:** the table is built on THIS (loaded) machine between the run's own ends.

**The laws** — each a root in fuel at the instant's frozen shaft speeds:
* **`EPR`** — instant inversion, `EPR(mf; ν) = EPR_dem(s)` (G1). No constant: fuel moves the pressure at once.
* **LP speed** — rung 95's toggle and knobs moved to the LP shaft:
  * the commanded rate, `dν_L/ds = (ν_L,dem(s) − ν_L)/τ_N`, solved as a root (G1);
  * the proportional law, `mf = W(ν_L,dem)·[1 + K·(ν_L,dem − ν_L)]`, with `W` the steady fuel at LP speed read off the
    SAME table.

  With the clean table and the generator on, `W` is the clean engine's fuel. So the proportional law settles with a
  speed error (droop) on the loaded engine. That is a property of a proportional law on a wrong table, and it is
  reported, not hidden.

The clamps, the legs (rung 46's redline, 48's schedule, 49/60's floor) and the min-select are rung 95's unchanged. The
governor's fuel is the demand the legs cut.

**The ends.** With the true-thrust reading the ends are rung 95's. With an inferred reading and the loaded table (or no
generator), the ends are rung 95's too: the table's end rows ARE those ends. **With an inferred reading, the generator
on and the clean table**, each end is where the LOADED engine's steady reading equals the clean table's reading at the
target. That is a root in burner temperature by rung 94's `try_hold_root`, with the reading as the held quantity
(G4: rising), refused in rung 94's words if out of reach. Its thrust differs from the target: that is the **steady
calibration error**, scored apart from the move (the advisor's separation). The run starts on that end's equilibrium,
and the governed law is on its root there.

**Recorded at every point:** the reading, its demand, the INFERRED thrust `F̂` (the table's thrust at the reading) and
the true thrust. For `EPR`, `F̂` on the root equals the demand exactly, because the piecewise-linear table inverts
itself.

**Scope.**
* Refused in words beside rungs 47/52/88's lagged marchers, as rung 95.
* **The metal cap is refused with an inferred reading.** A capped end's thrust falls below the table's span, so the
  table saturates. It can be built later.
* The control reads at the run's own flight condition. The usual corrections for flight condition (corrected speed,
  `δ`/`θ`) cancel because the table is built at the same condition, which is a concession: the off-design-altitude seam
  stays open.

**Where it lives:** `rust/src/inferred_thrust.rs` — the reading, the table, the calibrated end, the inferred thrust.
`governor.rs`'s `Law` gains the readings as its dispatch point; rung 95's arms are unchanged. The Controls view's route
and page are under § 1's two switches.

## 2. REDUCES and GATES (construction — no credit)

* **`thrust_by` = true ⇒ rung 95, bit for bit** (dispatch): every existing gate, oracle and golden is unmoved.
* **Generator off ⇒ `table_clean` inert, bit for bit.**
* **Loaded table ⇒ the ends are rung 94/95's ends exactly** (the same `Instant2`).
* **The decomposition gate** (G2 + G3). For `EPR`, at every recorded point of every run:
  `F_true − F̂ = V0·(ṁ̂_face − ṁ_face)`, where `ṁ̂_face` is the table's airflow read at the reading. **Bar frozen:
  ≤ 1e−12 × the thrust** (measured margin ~10× cooled, ~1 000× otherwise). Over it means investigate, not relax. (A
  free line for the spec, not a gate, since the Controls view flies one condition: at standstill, `V0` = 0, the
  pressure ratio would read net thrust exactly. The whole error is flight speed × airflow error.)
* **The hold identity:** on the root with no leg cutting, `EPR` equals its demand to the root's tolerance, and the
  rate law's LP acceleration equals `(ν_L,dem − ν_L)/τ_N`.
* **The fixed point:** run long, the `EPR` and LP rate-law runs settle on the final end — rung 95's, or the calibrated
  end with the clean table. The proportional law settles on its own fixed point, with its droop under a clean table
  reported.
* **The step check:** the LP laws' knob limits come from their OWN step check (0.02 against 0.005). Rung 95's 0.04 τ
  and gain-50 limits were measured on the HP shaft.

## 3. PREDICTIONS — frozen here, scored after the build

Cells: the default slam, the generator off unless stated, no lever or limiter, each of the four maps. "True-thrust" is
rung 95's governor at the same settings. The LP speed governor is scored at the page's default law, the commanded rate
`τ_N` 0.1.

**The shared derivation (D).** Three steps:
1. With both turbines choked (rung 38's choke chained twice), their pressure ratios and temperature ratios are nearly
   fixed; they move only through the turbine maps' efficiencies. So `EPR ≈ const × pt4/pt2`, with the overall
   compressor pressure ratio times the burner loss in the constant.
2. At an equal overall pressure ratio, the compressors' work per unit air is nearly equal (`Tt2` is fixed by the
   flight). An engine whose shafts must ALSO deliver a surplus, to accelerate or to carry a load, needs more turbine
   work per unit air. With the turbine temperature ratio fixed, that means a higher `Tt4`.
3. The choked HP nozzle passes `ṁ ∝ pt4/√Tt4`. So at equal `EPR` the surplus-carrying engine swallows LESS air, pays
   less ram drag, and gives MORE net thrust than the steady table says.

*Risks:*
* A shaft that decelerates while the other accelerates leaves the total surplus of uncertain sign.
* The two compressors' work at equal overall pressure ratio depends on how the ratio splits between them, and their
  efficiencies move along the maps.

**H1 — the `EPR` governor's true thrust EXCEEDS the thrust it infers all through an acceleration, and falls below it
all through a slowdown.** At every recorded point with `0 < s ≤ ramp`, `F_true − F̂ > 0` on 1000 → 1400 K and `< 0` on
1400 → 1000 K. *Derivation:* (D); on a slowdown both shafts give up power (a deficit), so every sign flips. 8 cells.

**H2 — so the `EPR`-governed acceleration is HOTTER and NEARER STALL than rung 95's true-thrust one.** Its peak `Tt4`
is higher, and its lowest flow coefficient is lower on BOTH compressors. *Derivation:* H1 ⇒ the `EPR` governor
delivers more thrust than the line rung 95 holds exactly. Thrust rises with fuel at frozen speeds (rung 95's G1), so it
burns more, and more fuel means more `Tt4` overshoot and surge approach (rungs 35/45). *Risk:* the states diverge, and
the `EPR` run's faster spools need less `Tt4` for the same thrust later in the move. **Visible before freezing** (§
Status): the probe's rich-wall counts point this way. 4 maps × 3 = 12 cells. **Vacuity rule (rung 95's):** a
flow-coefficient cell counts only if both runs' minima are interior.

**H3 — the LP-speed governor's true thrust also exceeds the thrust it infers through an acceleration, and falls below
it through a slowdown** (`0 < s ≤ ramp`, rate law `τ_N` 0.1). *Derivation:* there are two parts at a given LP speed.
(i) The accelerating engine burns more fuel than the steady engine at the same shaft speeds: the LP shaft must have a
surplus, the LP acceleration rises with fuel (G1), and thrust rises with fuel at frozen speeds. (ii) Its HP speed
differs from the steady HP speed at that LP speed. **The sign is safe if the HP shaft LEADS** (`ν_H > ν_H,steady(ν_L)`
on the acceleration — the HP shaft is driven by the burner directly, and its lighter load reaches the fuel first). If
the HP shaft lags, its deficit lowers thrust and may outweigh (i). That is the risk. 8 cells.

**H4 — calibrated on the CLEAN engine, the `EPR` control gives MORE thrust than asked at both steady ends, with the
generator on either shaft.** At the calibrated end (§ 1), the true thrust exceeds the target, at 1000 K and 1400 K,
with the generator on the HP or the LP shaft (power 0.02). *Derivation:* (D) with a load in place of an acceleration;
at steady state the load is a permanent surplus. *Risk, named (the advisor's):* rungs 92/94 found an LP-shaft load
nearly free, and it becomes a stall-margin GAIN once the burner floats. If the LP load shifts the split of the
compressors' pressure ratio enough, (D)'s "equal work at equal overall pressure ratio" may not hold. 2 shafts × 4 maps
× 2 ends = 16 cells. An end with no steady point scores VACUOUS.

**Reported, not scored:** the size of each gap (the user's question, measured after the build); the LP-speed control's
calibration error under a clean table (no sign derived); the proportional law's droop.

## 4. What would change the plan

A MISS is reported as a finding (the sandbox rule). A failed root inside the clamp is a reported stop with its reason.
**The crash map runs BEFORE the spec is written** (rung 95's lesson 1). **Cost:** the table costs 11 more steady solves
per build (22 with the clean table and the generator on). The governor's solve per derivative is rung 95's. The
default governed run is timed natively before the page is built.
