# Rung 96 — A GOVERNOR THAT INFERS THRUST: the throttle holds a reading (pressure ratio or fan speed) the control turns into thrust

**Code:** `rust/src/inferred_thrust.rs` — the reading `epr`, the control's table `Table` / `try_table`, the thrust it
believes (`Table::inferred`), the calibrated end's refusal words. `rust/src/governor.rs`'s `Law` gains `Epr`, `LpRate` and
`LpProp`, and `held_by` (rung 95's arms unchanged). The Controls view adds the switches `thrust_by` and `table_clean` and
the calibrated ends (`sandbox_controls::build`).
**Gates:** `rust/tests/rung96.rs`, and the page's `rust/tests/sandbox_inferred.rs`.
**Panel:** `print_inferred_table` (born in Rust, held to `rust/oracle/rust_owned/`; a change detector only).
**Anchor:** `docs/plans/rung96-anchor-inferred-thrust.md`. It holds the device and every bar, committed as `67075f1`
before any rung-96 code existed.
**Sandbox:** the Controls view, *Govern during the move* → *Thrust read from* (`docs/plans/sandbox-plan.md` § 25).
**Scratch scripts and outputs:** `docs/plans/scratch-archive/rung96/`.

**Shipped as KNOBS** (the sandbox direction): predictions were registered first and are scored here; a miss is a
finding.

Rung 95 closed by naming this seam: its governor read thrust directly, and a real engine infers it. The setup was
proposed to the user, who answered "go" (2026-10-16):
* both readings, as a toggle;
* the throttle still asks for a straight line in thrust, which a steady table turns into the reading's demand;
* rung 95's two speed laws moved to the fan's (LP) shaft;
* a switch for whether the table was made on the engine with the generator off or on.

> **THE HEADLINE — an accelerating engine is a loaded engine, so a control that reads pressure or fan speed asks for
> MORE thrust than it believes it is getting.**
>
> Both nozzles are choked, so the pressure ratio reads the jet's gross push EXACTLY (to ~1e−15). What it cannot see is
> the drag of the air the engine swallows. At a given pressure ratio, an engine whose shafts must also find power to
> spare (to speed up, or to drive a generator) burns hotter and passes less air. It therefore gives more thrust than the
> steady table says. Through the default move the engine gives up to **12 % of the thrust change** more than the
> pressure-ratio control believes (16 % less on a slowdown). The run is hotter than rung 95's true-thrust one, and both
> compressors come nearer stall.
>
> The fan speed misses by far more: **26–46 % of the change** accelerating, **32–59 %** slowing.
>
> Calibrated on the engine with the generator off, the pressure-ratio control gives **0.35–1.25 % more** thrust than
> asked at the steady ends, with the generator on either shaft. The fan-speed control misses by **−9 % to +26 %**, its
> sign set by WHICH shaft carries the load.

## 1. What it adds

**The reading and the table.** The control reads either the engine pressure ratio, `EPR = π_LPT·π_HPT·pt4 / pt2` (the
nozzle inlet's total pressure as every transient tail builds it, over the inlet's), or the LP shaft speed `ν_L`. It turns
the reading into thrust through **its table**: 13 steady points (rung 48's count) between the two ends, the ends
themselves as the end rows. Each row holds thrust, `EPR`, `ν_L`, fuel and the swallowed air. Values are read linearly
between rows, and the end segments are carried on beyond them. The rows rise together in thrust, `EPR`, `ν_L` and fuel on every engine probed
(anchor G4); the build refuses in words if they do not. The throttle's demand is rung 95's straight line in thrust, read
through the table into the reading's demand. The thrust the control BELIEVES is the table's thrust at the reading,
recorded at every point beside the true one.

**The laws** are roots in fuel at the instant's frozen shaft speeds, as rung 95's:
* the pressure ratio by instant inversion (anchor G1: it rises strictly with fuel);
* the fan speed by rung 95's commanded rate, `dν_L/ds = (ν_L,dem − ν_L)/τ_N`, or its proportional law,
  `mf = W(ν_L,dem)·[1 + K(ν_L,dem − ν_L)]`, with `W` the table's own fuel.

The clamps, the legs and the min-select are rung 95's unchanged.

**Calibrated on the clean engine** (`table_clean`, default on; inert with the generator off). The table is made on the
generator-off engine between the two set throttles, so its end rows ARE rung 94's targets. Each end is then where THIS
engine's steady reading equals the table's (rung 94's `try_hold_root` with the reading held). Its thrust against the
target is the **steady calibration error**, kept apart from the error during the move. With the switch off, the table is
made on this engine between its own ends (rung 94's hold ends, or the set throttles with no load).

**Scope:**
* rung 95's plain march, refused beside rungs 47/52/88's lagged marchers;
* the metal cap is refused with an inferred reading (a capped end falls below the table's span);
* the table is made at the run's own flight condition, so the corrections real controls make for flight condition
  (`δ`/`θ`) cancel — a concession, since the off-design-altitude seam was open when this rung shipped (its solver
  side was repaired 2026-10-16: sandbox plan § 26).

## 2. Reduces

* **Read directly, the thrust governor is rung 95's, bit for bit** (dispatch: no table is built). Every earlier gate,
  oracle and golden is unmoved, and rung 95's suite passes unchanged. Off the thrust hold the reading is ignored, bit for
  bit.
* **With the generator off, `table_clean` is inert** (bit for bit, four maps × both readings).
* **The loaded table's ends ARE rung 94's** — the same steady instants.
* **The pressure-ratio gap is the ram drag it cannot see:** `F_true − F̂ = V0·(ṁ̂_face − ṁ_face)` at every point of
  eight engines (the generator on either shaft, cooling, customer bleed, the bleed lever, both limiters) on all four
  maps. The worst residual is **1.2e−13** of the thrust, against a frozen bar of 1e−12. This is construction (anchor
  G2/G3), not a finding.

  *The free line:* at standstill (`V0` = 0) the pressure ratio would read net thrust exactly. The whole error is
  flight speed × airflow error, and the Controls view flies at M 0.85.
* **The hold identity:** on the root with no leg cutting, `EPR` equals its demand to ≤ 4e−15, and the fan's commanded
  rate holds its law to ≤ 7e−12. On its root, the pressure-ratio control believes exactly the demanded thrust: the
  piecewise-linear table inverts itself.
* **The fixed point:** run long (8 τ), every inferred run settles on its end — rung 95's, or the calibrated one.
  * Pressure ratio: to 1e−9 in speed.
  * Fan speed: the LP shaft lands at once, but the HP shaft creeps (3e−6 short after 8 τ on flat-LP), so its gate's bar
    is 1e−5 (§ 4).

## 3. Scoring (anchor § 3 — bars frozen at `67075f1`)

The default slam is 1000 → 1400 K over 0.5 τ, settling for 2 τ, with no generator, lever or limiter unless stated, on
each of the four maps. "True" is rung 95's governor at the same settings. The fan-speed law is the commanded rate at
`τ_N` 0.1. Gaps are given as fractions of the end-to-end thrust change.

| # | verdict | measured |
|---|---|---|
| H1 the pressure-ratio control's true thrust above what it believes on an acceleration, below on a slowdown | **HIT** 8/8 | over the ramp's 25 points: +0.021…+0.120, +0.022…+0.123, +0.022…+0.122, +0.024…+0.121 accelerating (largest at `s` 0.26–0.30); −0.155…−0.014, −0.159…−0.015, −0.158…−0.014, −0.154…−0.017 slowing (flow/press, press/flow, tilted, flat-LP) |
| H2 so hotter and nearer stall than rung 95's, both compressors | **HIT** 12/12 (all 8 minima interior) | peak `Tt4` 1936.5 / 1946.4 / 1937.3 / 1908.0 K against 1927.4 / 1940.3 / 1932.5 / 1898.7; lowest `φ_LP` 0.703 / 0.708 / 0.698 / 0.652 against 0.714 / 0.722 / 0.713 / 0.665; lowest `φ_HP` 0.820 / 0.788 / 0.800 / 0.820 against 0.832 / 0.804 / 0.816 / 0.833 |
| H3 the fan-speed control's true thrust above what it believes on an acceleration, below on a slowdown | **HIT** 8/8 | +0.093…+0.397, +0.096…+0.455, +0.092…+0.422, +0.075…+0.259 accelerating; −0.541…−0.082, −0.593…−0.092, −0.566…−0.086, −0.323…−0.047 slowing |
| H4 calibrated on the clean engine, the pressure ratio gives more thrust than asked at both ends, generator on either shaft | **HIT** 16/16 | HP shaft: +1.19…+1.25 % at 1000 K, +0.355…+0.358 % at 1400 K; LP shaft: +1.12…+1.19 %, +0.349…+0.355 % (power 0.02, four maps) |

All four were derived before the build (anchor § 3, derivation (D)). The probe printed no gap, temperature or margin. It
did print clamp counts that pointed H2's way, disclosed in the anchor before freezing. H2's margin is thin in
temperature (5–10 K) but clear in stall margin (0.011–0.016 in flow coefficient on both spools).

## 4. After scoring (reported, not scored)

* **The fan speed's calibration error has no derived sign, and it is large.** With the clean table and the generator at
  power 0.02, the fan-speed control's ends miss by −4.5 % / −1.2 % (HP shaft, flow/press; −9.2 % / −4.0 % flat-LP) and
  by **+19.9 % / +6.8 %** (LP shaft; +26.2 % / +11.1 % flat-LP), at 1000 / 1400 K. The sign follows WHICH shaft carries
  the load:
  * an LP-shaft load must be dragged round at the held fan speed, which takes a faster, hotter core: the HP shaft runs
    0.026–0.054 faster and the burner 57–120 K hotter than on the clean engine at the same fan speed, and the thrust
    rises;
  * an HP-shaft load slows the core under a held fan speed: the HP shaft runs 0.012–0.031 slower (the burner within
    19 K), and the thrust falls.

  These were measured at the calibrated ends after scoring (`claims.txt`, flow/press and flat-LP), not derived.

  The pressure ratio misses by about 1 % either way.
* **The inferred controls overshoot the thrust they were asked for, and get there sooner.** On the default
  acceleration (`report.txt`, settle 4 τ), the true thrust overshoots its final value by the following fractions of the
  change, and reaches 95 % of it at the following times. Rung 95's true-thrust governor makes no overshoot by
  construction and reaches 95 % at 0.475 τ; the open ramp takes 1.29–1.39 τ.

  | control | overshoot | 95 % reached | peak `Tt4` |
  |---|---|---|---|
  | pressure ratio | 0.10–0.11 | 0.416–0.419 τ | 1908–1946 K |
  | fan speed, commanded rate 0.1 τ | 0.19–0.25 (flat-LP: none) | 0.36–0.38 τ (flat-LP: 0.496 τ) | up to 2015 K (flat-LP: 1786 K) |
  | fan speed, proportional `K` 20 | 0.05–0.32 | 0.34–0.44 τ | up to 2092 K |

  The flat-LP map's fan-speed run is the exception: cooler and slower than rung 95's.
* **The proportional law on the clean table droops**, as a proportional law on a wrong table must: after 8 τ the fan
  sits 1e−4 to 5e−3 off its demand with the generator on.
* **Under the fan-speed control the HP shaft settles on its own clock.** The control pins the LP shaft on its demand,
  and the HP shaft then relaxes by itself; its last approach is slow. After 8 τ it is still 3e−6 short of its end on flat-LP,
  against 1e−9 for the pressure ratio. The early approach is not slower: the HP shaft gets within 1e−3 of its end at
  0.76–1.9 τ (3.6 τ flat-LP), against rung 95's 1.0–2.5 τ.
* **The step check set the LP laws' knob limits** (`stepcheck.txt`: 0.02 against 0.005, max flow-coefficient
  difference, four maps):
  * the commanded rate: 1.2–1.6e−3 at 0.02 τ, 1.8–2.5e−4 at 0.03 τ, and 4.9–6.6e−5 at **0.04 τ**;
  * the proportional law: 6.8–8.9e−5 at gain **50**, 1.5–2.0e−3 at 100.

  On accelerations, those bolded values are where the limits sit: rung 95's values, now measured on the LP shaft. On
  slowdowns the governor sits on the lean wall, and the march is rougher (≤ 9.5e−4) and erratic in `τ_N`, as rung 95
  found: that wall is the closure's search bound, not smooth in the state.
* **The crash map** (1 000 requests, seed 13, every reading and law, the table clean or loaded, random limiters, levers,
  generator, cooling and cap, both ends over the slider range — `scratch-archive/rung96/crash.txt`). **No panic.**
  * **594 ran to the end:** 91 on the lean edge somewhere, 116 on the rich.
  * **53 stopped early with a reason:** 51 had the stall floor out of reach, and 2 had a throttle already above its
    redline, rung 95's class.
  * **The rest were refused in words:** the ends (rung 94's words and the calibrated end's), 43 the acceleration
    schedule's table, 25 the cap beside an inferred reading, 60 the cap without cooling, and 5 the control's table.
  * **Time:** the median request takes 0.23 s. The slowest timed 10 s on a loaded machine, and 1.6 s re-timed alone.
    Rung 95's true-thrust governor takes 4.4 s on the same request: the cost is the lean wall beside the bleed valve, not
    the reading.
* **Cost.** An inferred run costs what rung 95's does (0.12 s natively on the default move; 0.38 s with the redline and
  the schedule). A clean-calibrated one costs 0.16 s with the generator on.

## 5. Why — the model

Anchor § 3's derivation (D) has three steps:
1. Both turbines are choked, so their pressure and temperature ratios are nearly fixed. The pressure ratio is then a
   fixed multiple of the compressors' overall pressure ratio.
2. At an equal overall pressure ratio, the compressors' work per unit of air is nearly equal. An engine that must also
   find power to spare, to accelerate a shaft or to carry a load, needs more turbine work per unit of air. With the
   turbine temperature ratio fixed, that means a hotter burner.
3. The choked HP nozzle passes `ṁ ∝ pt4/√Tt4`. So at an equal pressure ratio, the surplus-carrying engine swallows LESS
   air, pays less ram drag, and gives MORE net thrust than the steady table says.

On a slowdown the shafts give power back, and the sign flips. So an acceleration and a generator are the same thing to a
pressure-ratio control. H1 and H4 share one derivation, and both hit, which is consistent with (D) but does not prove it. The
decomposition gate proves the less-air step; nothing here measured the hotter-burner step at an equal pressure ratio.

The fan speed's error has two parts at a given LP speed:
* the accelerating engine burns more fuel (the LP shaft needs a surplus, and its acceleration rises with fuel), which
  raises thrust over the table's;
* its HP speed differs from the steady HP speed at that LP speed. The anchor named this as H3's risk: the sign was safe
  only if the HP shaft leads.

**That condition was measured after scoring, and it did NOT hold** (`report.txt`). Accelerating, the HP shaft LAGGED its
steady speed at every ramp point on flow/press (by up to 0.002) and on flat-LP (by up to 0.026). It led on press/flow (by
up to 0.014), and was mixed on tilted. H3 hit on all four maps anyway: the extra fuel outweighs an HP lag of this size.
The named risk happened, and the sign survived it. Why the fan speed's gap is several times the pressure ratio's is not
separated here.

## What it does to its neighbours

* **Rung 95 — its concession DISCHARGED, and its headline SHARPENED.** "The governor reads thrust directly" is now a
  switch. A real control's reading makes rung 95's hot slam hotter: a pressure-ratio control asks for more thrust than
  the throttle's line through the move, so it burns 4–18 % more fuel than rung 95's at every point of the move (and less
  after it, at most points, as rung 95's catches up — `claims.txt`).
* **Rung 94 / 92 — a generator is an acceleration that never ends.** Calibrated without the load, the pressure-ratio
  control gives slightly more thrust than asked with a generator on either shaft, by the same mechanism (D) as the
  move. Its miss is nearly the same on either shaft (+1.1 % LP against +1.2 % HP at 1000 K). The fan speed's is not:
  the LP load is the worse one there (up to +26 %), and the HP load misses the other way.
* **Rungs 42/62 — the bleed's ram-drag booking** (the dumped air pays full ram drag) is exactly what the pressure ratio
  cannot see. The decomposition gate carries it through the bleed lever and the customer's air to 1e−13.

## Concessions

* **The table is made at the run's own flight condition**, so the corrections for flight condition cancel. The
  off-design-altitude seam was open when this rung shipped; its solver side was repaired 2026-10-16 (sandbox plan § 26).
* **Both nozzles are choked** (inherited). That is what makes the pressure ratio read gross thrust exactly; an unchoked
  nozzle would add a second error. Rung 97 now solves the unchoked nozzle on this engine and measured that error's
  shape: G2 survives there (the fully expanded nozzle's push also depends on `pt9` alone), G3's straight line does not;
  none of this spec's scored runs unchoke (rung 97 § 4).
* The table is 13 rows, read linearly between them. On the pressure ratio this costs nothing in gross thrust (G3). On
  the fan speed it adds a small interpolation error between rows, not separated here.
* No sensor dynamics, noise or lag. A real `EPR` probe and speed pickup have both.
* The metal cap is refused beside an inferred reading.
* Inherited: invented maps, rung 36's imposed stall line, the perfect gas, and rung 95's lean-wall roughness.

## The next seam

**The governor on the lagged marchers** (rungs 47/52/88, carried from rung 95), or **the control's own corrections**:
the table made at one flight condition and flown at another, its readings corrected by `δ`/`θ` as a real schedule's
are. That needed the off-design-altitude seam first — repaired 2026-10-16 (sandbox plan § 26), so it is now buildable.
