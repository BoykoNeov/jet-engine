# Rung 101 — THE AIRFLOW OFF THE LINE: where the thrust formula gets the air swallowed — rung 100's table, the speed tables, or an exhaust-temperature sensor

**Code:** `rust/src/airflow_source.rs` — `AirflowFrom` (the setting), `Air` (a formula's built source), `Sensed` (what the
sensor sees at an instant), `nozzle_flow` / `sensed_air` (the nozzle's closed-form flow and the air it credits),
`SensorLag`, `TAU_MIN`/`TAU_MAX`. `unchoked_formula.rs`'s readers carry an `Air` and gain `_sensed` twins;
`inferred_thrust::Table` gains `sensed`, `try_belief`, `try_demand_at`; `mach_axis::Tables::Formula` carries the source
and, for the grid, its slices; `governor.rs` gains `try_governor_fuel_at` (the sensor's roots), `try_der_at`,
`try_rk_step_sensed` and the three-state march; `fuel_transient::PointExtra::Sensor { tm }` (15 keys).
`sandbox_controls.rs` gains `airflow` / `exhaust_lag` / `exhaust_tau`, the ends calibrated on the sensor's belief
(`sensed_end_root`), the re-run step's third state, and the reply's `exhaust_tt5` / `exhaust_sensor` / `demand`.
**Gates:** `rust/tests/rung101.rs`. **Panel:** `print_airflow_source_table` (born in Rust, held to `rust/oracle/rust_owned/`;
a change detector only). **Anchor:** `docs/plans/rung101-anchor-airflow-source.md`, committed as `604f6a2` before any
rung-101 code (its Status corrected in `eab1563`, still before code). **Sandbox:** the Controls view's *air swallowed*
toggle under the formula, the sensor's lag (`docs/plans/sandbox-plan.md` § 33). **Scratch:** `docs/plans/scratch-archive/rung101/`.

Rung 100's seam: *"The airflow off the line: the pressure-ratio formula's whole miss is `V0·(ṁ̂ − ṁ)`, the table's
airflow at the reading; below choke (and at a low throttle flown fast) the engine is off the table's line …"* The scope
was set on measured probes, each answer the user's (memory `rung101-decisions`): both new sources as a CHOICE beside
rung 100's table (the default); the sensor with a lag knob, off by default, in HP-shaft time constants; both readings,
the speed tables offered on the fan speed even after the probe showed they make it worse there.

> **THE HEADLINE — a steady table cannot see an accelerating engine; a sensor on the nozzle can, until it is slow.**
> Rung 100's whole remaining error was the air it believed swallowed. Read from the speed tables, the pressure ratio's
> steady miss falls to the floor of a table made at this very flight (Mach 1.4 from 700 K: −1.7 % → −0.01 %) — but the
> move's error does not move at all (+39 % either way): at a given pressure ratio an accelerating engine passes less air
> than ANY steady table says (rung 96). An exhaust-temperature sensor on the nozzle's own flow, with the fuel the control
> meters and the air it dumps taken off, closes the plant's mass bookkeeping to 4e−14: read from the pressure ratio the
> control then knows the TRUE thrust at every instant, and governs exactly as rung 95's thrust-reading governor (to
> 4e−14). A probe that lags the gas gives it back: the governor holds the BELIEF on the demand line whatever the lag, so
> the error moves into the thrust delivered — +48 % at Mach 1.4 with a one-shaft-time-constant probe, worse than the
> tables. And on the fan speed, whose formula's push is itself a guess, the speed tables remove an airflow error that had
> cancelled part of it — but only below choke; where everything is choked they help.

## The device (anchor § 1)

* **`airflow`** under `speed_corr: formula` (inert elsewhere, and at the design flight, where no formula is built):
  `table` (rung 100's, the default), `grid` (the speed grid's thrust table at this Mach — rung 99/100's slices,
  blended at the reading — its airflow column only), `exhaust` (the sensor).
* **The sensor:** `ṁ̂ = (ṁ9(pt5̂, T_m) − W_f)/((1 − b)(1 − c))`, `ṁ9 = a8·ρ9·V9` on the perfect gas (choked at `p*`, else at
  `p0`), `W_f` the applied fuel, `b` the valve, `c` the customer share (the model's realized values — in *amount* mode a
  real control knows an amount, not a share: disclosed). `pt5̂ = EPR·pt2` on the pressure ratio (measured); B's own
  guess `π_u·pt45` on the fan speed (so there the sensor is not exact).
* **The lag:** `exhaust_lag` (off), `exhaust_tau` (1.0, 0.05–5 HP τ; RK4's edge at this step is 0.0072). On, `T_m` is a
  third march state, `dT_m/ds = (Tt5 − T_m)/τ`, started at the steady start's `Tt5`; off, rung 95's two-state march by
  dispatch.
* **The governor under the sensor:** the belief depends on the instant, so each law's root is taken at the trial fuel
  — pressure ratio `F̂(i(mf)) − F_dem`, fan rate law `Φ_L/ρ − (ν_dem(mf) − ν_L)/τ`, fan proportional law
  `mf − W(ν_dem)·[1 + K·(ν_dem − ν_L)]` (rises strictly: `ν_dem` falls with fuel). Every other table: rung 95/96's.
* **The ends:** calibrated where the steady BELIEF equals the target, searched from the set throttle outward
  (`sensed_end_root`) — see *The first score*.

## Givens (anchor § 0) — the probes that set the scope

Open loop, on rung 100's own governed runs: **G1** rung 100's table −55…+7.7 % at the start, +0.9…+47 % in the move; a
table made at this flight ≈ 0 at the start, the SAME in the move; the grid ≤ 0.14 % at the start outside the 0.3–0.6 gap;
B's rebalanced row weak (dropped); the sensor ≤ 1e−14 everywhere. **G2** the sensor's closure ≤ 3.9e−14 with the
overboard term, up to −80 % without. **G4** on the fan speed the grid makes the start worse at 700 K (−4.9 → −6.8 % at
Mach 1.4), the sensor better (→ −2.7 %). **G5** a lag of 0.25 τ costs +2…+75 % in the move. **G6** every residual rises
with fuel, one root.

## Gates (anchor § 2) — measured

| Gate | Bar | Measured |
|---|---|---|
| `airflow: table` (default) ⇒ rung 100 | bit for bit | census: 326 / 326 check replies byte-equal; crash map below |
| inert under `grid`/`none` speed correction, at the design flight, lag without the sensor | bit for bit | holds |
| grid airflow at the design slice's Mach (0.85, another altitude) | EPR bit for bit; fan ≤ 1e−9 | holds (the fan reads the column along a different straight line: rounding) |
| push gate under the grid airflow (rest after the guess) | 1e−9 N per kg/s | holds, Mach 0.05 and 1.4 |
| sensor gate, EPR, no lag: `|F − F̂|` every point, four engines | 1e−9 N per kg/s | ≤ 4.6e−12 |
| closed loop against `thrust_by: true` | 1e−6 of the thrust | ≤ 4e−14 |
| lag starts at the steady start's `Tt5`; the re-run step bit for bit | exact | holds |
| a faster sensor comes closer (true thrust's gap, per halving of τ) | 1.2–2.0 | 1.32–1.69 (`lagorder.txt`) |
| lag refused outside 0.05–5 where it acts | — | holds |
| the page's fill-in = the model's defaults | — | holds |

**Two gates replaced, disclosed.** The anchor's *"grid at a slice's own flight ⇒ the table made there, bit for bit"* is
gated at the design slice's Mach at another altitude instead (the same identity, and a run to compare against); its
*"lag at a run with no move"* cannot be built — a design table between equal throttles has no span and is refused — so
the lag's reduce is gated by its trend. That gate's first bar (a ratio of 2, "first order") was typed, not measured, and
failed at 1.59996: the measurement is SUBLINEAR (1.32–1.69 per doubling), because the governor moves the fuel with the
demand and the exhaust temperature jumps at the move's start — even a fast probe misses part of the jump. And the first
draft compared the BELIEFS: on the pressure ratio they are 3e−12 apart whatever the lag, since the governor holds the
belief on the demand line; the lag shows in the thrust delivered.

## Scored (anchor § 3) — 778 HIT, 46 MISS, 56 VACUOUS of 880 (`score.txt`)

* **H1 — the speed tables fix the pressure ratio's steady start. 112 / 112.**
* **H2 — and leave the move alone. 112 / 112** (closed loop, within 15 % of a table made at this flight).
* **H3 — the unlagged sensor IS rung 95's thrust governor. 112 / 112** (≤ 1e−6, every point).
* **H4 — a slow sensor's move error is positive and rises with τ (0.1 < 0.5 < 2). 112 / 112.**
* **H5 — it loses to the tables at τ 2, wins at τ 0.1 at Mach ≤ 0.7. 111 / 112.** The miss: hot sea level, Mach 0.2,
  750 K, cooled, tilted map — τ 2 at +11.16 % against the grid's +11.26 % (a tie; the named risk).
* **H6 — the sensor helps the fan speed fast, a wash slow. 77 HIT, 3 MISS, 32 VACUOUS.** The misses are the bars, both
  ways: Mach 0.25 from 750 K plain, ratio 0.67 (better than "a wash"); 3 km Mach 1.3 from 750 K, 0.76–0.78 (less than
  0.7×). Vacuous: rung 100's own start under 0.05 % (24, Mach 0.7), or rung 100's fan formula refused at hot sea level
  from 750 K (8 — the table's rows have no working state against that air).
* **H7 — the sensor helps the fan speed in the move. 104 / 104** (8 vacuous, as H6's sea-level 8).
* **H8 — the speed tables make the fan speed worse. 38 HIT, 42 MISS — the finding.** The lost cancellation needs B's
  PUSH to be wrong, and it is only below choke: at the starts where the start or its lowest design rows are unchoked
  (Mach 0.25 cooled, hot sea level, 750 K flown at Mach 1.3–1.5) the grid is worse, as G4 measured at 700 K; where the
  start and its rows are all choked, B is rung 99's exact column (rung 100 G9) and its only error the airflow guess,
  which the grid REMOVES (Mach 1.2 from 750 K: +0.134 → +0.042 %; `h8why.txt`). The probe flew only 700 K starts, all
  below choke: the prediction generalized a below-choke measurement to choked cells.

**The first score** (`score_first.txt`, 746 / 84 / 50): 19 cells — fan speed with the sensor, Mach 0.2–0.25 from 750–
900 K — were REFUSED while calibrating their ends: rung 94's root search walks up from 600 K, and the fan-speed formula
with the sensor cannot be read that far below its rows (rung 97's "no working state" at the extrapolated row), where the
bare reading, read without the sensor, always can. An implementation defect, not a device choice: the search now starts
at the set throttle (`sensed_end_root`). Rescored on the shipped code: the two scores differ in exactly those 19 cells'
H6 and H7 lines (diffed); every other line is identical as printed, H3's included (the sensor's pressure-ratio ends are
found by the new search too, to the same tolerance).

## Census — what the defaults moved, and what the new keys do (`census`, `crash101.rs`)

By running, against a clean worktree of `54ee954`. **Check requests: all 326 (every view) byte-equal** — the defaults
move nothing; the 9 new ones run (8) or refuse (the lag of 0.04 τ). **Crash map, rung 99's generator (seed 99, 1 500
requests), the new keys left out: 1 500 / 1 500 identical, no panic.** The same 1 500 with the three keys drawn from a
separate stream: 4 differ, all "ran → ran" (the generator reaches the formula with an inferred thrust in ~1 % of
requests). So a **targeted** map (seed 101, 600, every request governing an inferred thrust under the formula on
design-flight tables): keys left out, 600 / 600 identical to the worktree; with the keys, **no panic, 118 differ, every
one on a non-table source**: 77 new numbers, 14 refusals that now RUN (rung 100's "does not rise" / "not reached"
refusals, which the sensor's belief escapes), 23 refused both ways in new words, and 4 newly refused — 3 in the
sensor's words (its end lands where this loaded engine has no steady point: a customer bleed taking half the air, a
bled engine off its speed lines) and 1 under the speed tables in the formula's own bracket words (a thrust not reached
within the pressure ratios; the anchor's class A named only the grid's reach words — disclosed).

**A second calibration fix, found by the map** (before it, 5 newly refused): the sensor's end search started AT the set
throttle, where a loaded engine (a generator, a valve) can have no steady point — the reason its end is calibrated at
all. It now starts from the nearest grid temperature that answers, upward first; one of the five then runs. Reached
only where the set throttle has no steady point, so no scored cell (generator off, every set throttle answers) moved.

**Cost** (`smoke.txt`, one run each, under the crash maps' load — indicative only): a governed run 0.3–1.6 s on every
source; the sensor on the fan speed up to 0.9 s against rung 100's 0.3 s at Mach 0.5 (each demand a root over the rows
at the trial fuel), lagged 1.5 s at Mach 1.4.

## What it does to its neighbours

* **Rung 100 — its seam answered:** the airflow off the line is the whole steady miss, removable without a new sensor
  (the speed tables) on the pressure ratio; its "the formula at a low throttle flown fast" seam (−1.7 % / −21 %) goes with
  it. Not on the fan speed, where B's push is the larger error.
* **Rung 96 — its headline sharpened:** the loaded-engine gap is no table's — it survives a table made at this flight
  and the speed tables exactly — and an air sensor removes it entirely (the control then reads thrust).
* **Rung 95 — reached from rung 96:** the pressure-ratio governor with an exact sensor IS the thrust governor.

## Concessions

* The sensor is ideal in everything but its lag: the gas's own exhaust temperature (no radial profile, no probe error),
  the nozzle's own area and loss, the model's realized bleed and customer shares.
* On the fan speed the sensor uses B's guessed nozzle pressure; the fan-speed reading never measures it.
* The lag is first order and on the march's step; its floor (0.05 τ) is set by the integrator, not a probe.
* Inherited: invented maps, rung 36's imposed stall line, the perfect gas (both closed forms assert it), rung 95's
  lean-wall roughness, rung 98's fixed-amount loads.

## The next seam

**A realistic sensor:** a probe whose reading is not the gas's (a radial profile, a bias), or a measured pressure on the
fan-speed reading (a second sensor, which would make that reading's push exact as the pressure ratio's is). **The fan
speed's push:** B's error is now the fan-speed formula's whole steady miss where choked rows are absent — the engine off
the table's line (rung 100 G5) — and only a second number brings it back. Or the steady matchers' unchoked branch (rung
97's seam), or the governor on the lagged marchers (rung 95's).
