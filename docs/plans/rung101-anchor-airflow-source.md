# Rung 101 anchor — THE AIRFLOW OFF THE LINE: where the thrust formula gets the air the engine swallows — rung 100's table, the speed tables, or an exhaust-temperature sensor (pre-registered 2026-10-19 — § 3)

**Seam:** rung 100 § *The next seam*: *"The airflow off the line: the pressure-ratio formula's whole miss is
`V0·(ṁ̂ − ṁ)`, the table's airflow at the reading; below choke (and at a low throttle flown fast) the engine is off the
table's line, so `ṁ̂` is wrong by 0.5–3 %, magnified by `V0` against a small thrust. B's rebalance … could also fix
the airflow row — or the slow slices could supply the airflow beside the exact push."* Also its third: *"the formula at
a low throttle flown fast (Mach 1.4–1.6 from 700 K: −1.7 % / −21 %)"*.

**The setup (user, 2026-10-19; memory `rung101-decisions`).** Picked after `/clear` from a list of open seams. The
scope was set on measured probes (§ 0), each answer the user's:
* after G1–G3 (the four candidate airflows at the steady start and along the move): **both new sources, as a
  CHOICE** — rung 100's table kept, the speed tables' airflow, and an exhaust-temperature sensor; the sensor with a
  **lag knob, default off**; **both readings** (pressure ratio and fan speed);
* after G4/G5 (the fan-speed reading; the lag's cost): the speed tables **offered on the fan speed too, even though
  they make it worse there** ("show the result"); the lag **in HP-shaft time constants**;
* after the advisor flagged it: the **default is rung 100's table** — nothing that exists moves; the two new sources are
  opt-in, under the *formula* speed correction only (where the jet's push is exact, rung 100).

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung101-probe`, to be archived in
`docs/plans/scratch-archive/rung101/`) against the shipped crate at `54ee954`. **Disclosed:** every § 0 number is read
on rung 100's OWN governed trajectories — the probe evaluated each candidate belief on the run rung 100's governor
flew, open loop. **Even the START is not the device's own** (corrected 2026-10-19, before any code, from the
committed `604f6a2`, which said it was): under a carried table rung 98 CALIBRATES each end — the steady point where the
belief equals the end's target — so a new belief moves the start. The start's miss is the belief's error at that
moved point; the probe read it at rung 100's, a near-interpolation (the bars below were frozen with that margin and
are not changed by this correction). The MOVE transfers less (the governor will follow the new belief, and a slow
sensor feeds back: a low exhaust reading → too little thrust believed → more fuel → more overshoot). So § 3 freezes
SIZES only on starts, and signs / orderings / wide bars on the move. No rung-101 code exists.

---

## 0. GIVENS — measured before anything was written

The Controls engine, cells and conventions are rung 100's (rung 97's two-shaft transient, design flight 250 K, 50 kPa,
Mach 0.85; the governed move, hold thrust, generator off, both corrections on, `to` 1400 K; "miss" = `(F − F̂)/F`, > 0
when the engine gives more than the control believes). `push(pt9)` is rung 100's nozzle gross thrust;
`ṁ9(pt5, Tt5) = a8·ρ9·V9` the convergent nozzle's flow on the perfect gas (choked: `a8·pt9·√(γ/R)·(2/(γ+1))^((γ+1)/(2(γ−1)))/√Tt9`;
unchoked: `p9 = p0`, `M9` from `pt9/p0`), `Tt9 = Tt5`, `pt9 = π_n·pt5`.

**G1 — the four candidate airflows on the pressure ratio** (`airflow.txt`; plain / cooled 0.05+0.05 / customer
share 0.05 / bleed lever × three maps × 250 K Mach 0.1, 0.5, 1.4, 1.6 and sea level Mach 0.1, 0.3 × starts 700 / 1000 K;
the exact push throughout, so each miss IS that airflow's guess):

| airflow `ṁ̂` | start | along the move |
|---|---|---|
| rung 100: carried design table at the reading | −55…+7.7 % (customer M1.6 700 K: −184 % at F 13; bleed −1123 % at F 1.6) | +0.9…+47 % |
| the table made AT THIS FLIGHT (the floor) | ≤ 0.16 % (Mach ≤ 1.4); −0.4…−1.6 % at M1.6 700 K; −5…−7.5 % where F < 3 | +0.9…+47 % |
| the speed grid's blended table at the reading | ≤ 0.14 % except Mach 0.5 (−0.28…−0.45 %) and M1.6 700 K (= the floor) | ≈ the floor's, to 1 % of itself |
| rung 100's rebalanced row (B's) | −0.4…−3.0 % at M1.4, −6…−18 % at M1.6 (≈ 30 % of rung 100's); NO ROOT at every unchoked start | ≈ rung 100's |
| exhaust sensor `(ṁ9 − Wf)/((1−b)(1−c))` | **≤ 1e−14** | **≤ 1e−14** |

So: (i) the move's error is NOT a table's — a table made at this flight misses as much (rung 96's accelerating engine:
at a given pressure ratio it passes less air); (ii) the grid's airflow beside the exact push takes rung 100's steady
start to the floor except in the 0.3–0.6 gap; (iii) the sensor closes the plant's own mass bookkeeping to the bit, at
every instant. **Without** the overboard term (`b` the valve, `c` the customer share) the sensor misses by up to −80 %
(`airflow.txt`'s first run): the control must know the air it throws away.

**G2 — the sensor's closure** (`airflow_overboard.txt`, customer share / customer amount / bleed lever / both
levers): `|face − (ṁ9 − Wf)/((1−b)(1−c))| ≤ 3.9e−14` kg/s per kg/s of design air, start and every 8th march point.
Cooling air rejoins before the nozzle: no term.

**G3 — the floor's own misses** (rung 100 lesson 3, checked before scoring any variant): at the start the table made at
this flight reads its own end row: ≤ 0.16 % on all four variants at Mach ≤ 1.4; its larger misses (M1.6, 700 K, the
customer and bleed engines: −1.2…−7.5 %) are where `F` is 1.6–14 N per kg/s. **Hence § 3's thrust floor.**

**G4 — the fan-speed reading under B** (`fan.txt`; plain / cooled / customer / bleed × two maps × G1's flights): B's
push is approximate (rung 100 G5), and its push and airflow errors CANCEL in part — the speed tables' airflow (≈ the
truth at the start) makes the start WORSE: M1.4 700 K plain −4.9 → −6.8 %, M0.1 −2.35 → −2.49 %, sea level M0.3
−6.9 → −9.7 %; the exception is near choke (Mach 0.5: −0.17 → +0.04 %, bleed −0.75 → −0.68 %). The sensor, fed B's own
`pt5` guess and the measured `Tt5`, credits LESS air where B credits more pressure (`ṁ9 ∝ pt9`), so its errors cancel
B's: start |EXH|/|R100| 0.40–0.55 at Mach 1.4–1.6, 0.81–1.15 at Mach ≤ 0.5; along the move EXH < R100 at EVERY cell
(M1.4 700 K: +55 → +21 %, M0.1: +38 → +36 %). The sea-level Mach-0.1 700 K starts read 0.000 on every source (the start
IS B's row there).

**G5 — the lag's cost, open loop** (`fan.txt`, the EPR sensor's `Tt5` filtered first-order AFTER the run, `τ` in HP
time constants): worst in-move miss, every cell positive (the probe reads low while the gas heats, so `ṁ9 ∝ 1/√Tt5` is
over-credited): `τ` 0.25 → 2: +2.1 → +3.0 % (M0.1 700 K plain), +11.7 → +16.2 % (M0.5), +36.9 → +47.9 % (M1.4),
+0.73 → +1.19 % (M0.1 1000 K); rising with `τ` in every cell. Against the tables' own in-move miss (G1): at `τ` 1 the
lagged sensor is WORSE in every cell; at 0.25 it is better in the design air at Mach ≤ 0.5 and at sea level from 1000 K, worse at sea level from 700 K (+2.8 vs +2.6 %, +8.6 vs +8.0 %) and at Mach ≥ 1.4.

**G6 — the governor's residual under the sensor** (`shape.txt`; plain / cooled / bleed × two maps × 250 K Mach 0.1,
0.85, 1.6, 2.0 and sea level 0.3, 1.2 × 700 / 1000 K; the state frozen at the start and at three points along the
move, fuel scanned 0.3–3× the run's on 161 log points where the instant runs): on the pressure ratio
`F̂(i(mf)) − F_dem` — unlagged, and with the sensor's temperature frozen at the start's `Tt5`, 0.8× and 1.2× it —
**never falls, ≤ 1 sign change**; on the fan rate law `Φ_L/ρ − (ν_dem(mf) − ν_L)/τ` with `ν_dem` from B fed the
sensor at the trial fuel: **never falls, ≤ 1 sign change, ≤ 1 row root**. Mach 2.5 and 3.0 refuse to build (no
data). **Named risk:** at Mach 1.6–2.0 B-with-sensor has NO row root at 9–99 of up to 324 trial fuels (far from the
run's) — a search that wanders there stops in § 1's words. Derived (not measured): on the pressure ratio with the
sensor frozen at `Tm`, `dF̂/dpt9 = a8·[(1+γ)·(2/(γ+1))^(γ/(γ−1)) − V0·Φ*/√Tm]` (choked) turns negative for
`V0 > 844 m/s · √(Tm/700 K)` — about Mach 2.7 at 250 K: a slow sensor at that speed believes LESS thrust for more fuel.

## 1. THE DEVICE — derived before coding

**The setting** (`airflow`, three values, shown under the *formula* speed correction only; inert elsewhere, including
the design flight, where no formula is built):
* `table` — **the default**: rung 100's carried design table, bit for bit.
* `grid` — the speed grid's thrust table (rung 99/100's slices, bracket and blend, AT THE READING the control uses),
  read for its airflow column only; the push stays rung 100's.
* `exhaust` — the sensor: `ṁ̂ = (ṁ9(pt5̂, T_m) − W_f)/((1 − b)(1 − c))`, with `W_f` the metered (applied) fuel, `b` the
  bleed valve's position and `c` the customer share at the instant (the model's realized values: in *amount* mode the
  control really knows an amount, not a share — disclosed), `T_m` the sensor's reading; on the pressure ratio
  `pt5̂ = EPR·pt2` (measured), on the fan speed `pt5̂` = B's own guess at its row (`π_u·pt45`).

**The lag** (`exhaust_lag` on/off, default **off**; `exhaust_tau` in HP-shaft time constants, default 1.0, range
**0.05–5**; RK4 on `dT_m/ds = (Tt5 − T_m)/τ` is stable for `DS/τ < 2.78`, `DS` 0.02 ⇒ `τ > 0.0072`; 0.05 keeps the
step a seventh of that edge). Off: `T_m = Tt5` of the instant. On: `T_m` is a THIRD STATE of the governed march,
starting at the steady start's `Tt5` (so the start's belief is exact and the whole lag error is in the move), stepped
by the march's own RK4 beside `ν_L`, `ν_H`. Off ⇒ rung 95's two-state march by DISPATCH, not a zero-lag third state.

**The governor under the sensor.** `F̂` now depends on the instant (its `Tt5`, its fuel, its valve), so the reading's
demand cannot be computed from `s` alone. By dispatch, only under `exhaust`:
* pressure ratio: the fuel is the root of `F̂(i(mf)) − F_dem(s)` (thrust units), the same bracket-and-clamp search;
* fan speed, rate law: the root of `Φ_L/ρ(i(mf)) − (ν_dem(mf) − ν_L)/τ`, `ν_dem` B's demand for `F_dem` with the sensor
  fed `i(mf)`;
* fan speed, proportional law: the root of `mf − w(ν_dem(mf))·(1 + k·(ν_dem(mf) − ν_L))` — `ν_dem` falls with fuel
  (more fuel ⇒ more believed thrust at every row), so the residual rises strictly; one root.
The belief reported and gated is evaluated at the APPLIED instant (after the limiters cut). The ends are calibrated on
the same belief where they are calibrated at all (the generator cells; none is scored).

**The speed tables under the formula.** `airflow: grid` builds rung 100's grid slices for the thrust table beside the
design table. Their reach becomes the formula's: a slice that cannot be built refuses the run in rung 99's words. On
the fan speed, B's `F̂(x) = push(π_n·π_u·pt45) − V0·ṁ̂_grid(ν(x))`.

**Refusals, in words.** B-with-sensor at a trial fuel with no row root (G6's risk): *"The control's fan-speed formula,
with the exhaust sensor: a thrust of X lies beyond the table's rows at this exhaust temperature …"* — rung 100's
family, naming the sensor. A lag below 0.05 or above 5 τ is refused by the settings parser.

**The page.** The Flight panel's *formula* choice gains a three-way *airflow from* toggle (*table* / *speed tables* /
*exhaust sensor*) and, under the sensor, the lag checkbox and slider; the note says what each knows (the sensor: the
nozzle's area and loss, the fuel it meters, the air it dumps; a real probe responds in about one to a few shaft time
constants). The gap split keeps its two rows (the airflow guess, the rest); under the lag the results show the
sensor's reading beside the true exhaust temperature. The reply carries `airflow` / `exhaust_lag` / `exhaust_tau` only
off their defaults, so every default reply keeps rung 100's shape.

**Where it lives:** `rust/src/airflow_source.rs` — the sensor's flow, the three sources, the sensor-fed residuals and
the third state; `unchoked_formula.rs`'s readers take an airflow source; `governor.rs`'s march dispatches to a
three-state march when the lag is on; `sandbox_controls.rs` — the three keys.

## 2. REDUCES and GATES (construction — no credit)

* **`airflow: table` (the default) ⇒ rung 100, bit for bit** — every existing gate, oracle, golden, check request; no
  pin needed.
* **`airflow: grid` at a grid slice's own flight ⇒ the airflow column of the table made there, bit for bit**; at the
  design flight no formula is built (inert).
* **The push gate under `grid`** (rung 100's, the new `ṁ̂`): `|F_true − F̂ − V0·(ṁ̂ − ṁ_face)| ≤ 1e−9 N per kg/s` at
  every recorded point, Mach 0.05 (unchoked) and 1.4.
* **The sensor gate (pressure ratio, no lag):** `|F_true − F̂| ≤ 1e−9 N per kg/s` at every recorded point (applied
  instant), plain / cooled / customer / bleed, Mach 0.05 and 1.4 (G1/G2: 1e−14 relative). Not self-certifying: the
  nozzle's closed form against the plant's own face flow.
* **Its closed-loop consequence:** the same run against `thrust_by: true` (rung 95's thrust governor), same settings:
  every point's thrust within **1e−6 of the thrust** (the two root solves stop on different iterates; LEG_TOL-level).
* **The lag at a run with no move** (`from` = `to`): the three-state run's every point equals the two-state run's to
  **1e−12** relative (the sensor never leaves its start).
* **The lag reduces to its start:** `T_m(0) = Tt5(start)` exactly; the start's belief equals the sensor-off belief.
* **The page:** default-key replies byte-equal to rung 100's; the three keys round-trip; `exhaust_tau` 0.04 / 5.1
  refused.
* **What the defaults move — pre-registered, census by running against a clean worktree of `54ee954`:** NOTHING — every
  check request and the crash map (seed 99, 1 500, new keys at defaults) identical. A SECOND crash map with `airflow`,
  `exhaust_lag`, `exhaust_tau` randomized: no panic; every outcome difference against `airflow: table` in one of three
  classes: (A) `grid` — new numbers, or a refusal in the grid's reach words; (B) `exhaust` — new numbers, or § 1's
  sensor refusal; (C) lagged — new numbers.

## 3. PREDICTIONS — frozen here, scored after the build

**Cells.** Flights NOT in § 0: design air (250 K, 50 kPa) at Mach **0.25, 0.7, 1.2, 1.5**; 3 km standard
(268.65 K, 70.12 kPa) at Mach **0.2, 1.3**; sea level +30 K (318.15 K, 101.325 kPa) at Mach **0.2**. Starts **750 K and
900 K**. Variants: plain, cooled 0.05/0.05, customer share 0.05, bleed lever. Maps: flow/press and tilted. 7 × 2 × 4 × 2
= **112 cells per reading**. `airflow: formula` speed correction throughout, the governed move, hold thrust, generator
off, `to` 1400 K. **Thrust floor:** a cell whose steady start thrust is below **20 N per kg/s** is VACUOUS (G3), and so
is a cell refused for its own steady points or its grid slices (stated); a refusal in § 1's sensor words is a MISS.
"Start" = the belief's miss at the steady start; "move" = the worst `|F − F̂|/F` over the recorded points with
`F ≥ 20 N per kg/s`, signed.

**H1 — the speed tables fix the pressure ratio's steady start.** `EPR`, `grid`: **|start miss| < max(0.7 %, 0.5 N per
kg/s / F)**, and **< |rung 100's|** wherever rung 100's exceeds 0.3 %. *Derivation:* G1 — the grid's start ≤ 0.14 %
outside the 0.3–0.6 gap and its M1.6 700 K floor; −0.45 % worst in the gap; Mach 0.7 lies in the 0.6–0.85 gap, which
G1 did not fly. *Risk:* Mach 0.7 (a gap the probe never read), Mach 1.5 at 750 K (near G3's floor-misses). **112 cells.**

**H2 — the speed tables leave the move alone.** `EPR`, `grid` vs `table_at: flight` (the floor), each in its own
closed loop: **|move(grid) − move(floor)| < 0.15 × |move(floor)|**, same sign. *Derivation:* G1 — open loop, within 1 %
of each other; the move's error is rung 96's accelerating engine, which no steady table reaches. *Risk:* closed loop,
the two starts' beliefs differ by H1's sizes and the governors follow different demands. **112 cells.**

**H3 — the unlagged sensor makes the pressure-ratio control rung 95's.** Scored as a cell claim on top of § 2's gate
(the gate is at two flights; this is every cell): `EPR`, `exhaust`, no lag: **every point's thrust within 1e−6 of the
`thrust_by: true` run's.** *Derivation:* G1/G2. *Risk:* none named — a miss would mean the closure fails somewhere G2
did not reach (a valve or customer mode at a new flight). **112 cells.**

**H4 — a slow sensor's move error: positive, and rising with `τ`.** `EPR`, `exhaust`, lag on at `τ` = 0.1, 0.5 and 2:
**move(0.1) < move(0.5) < move(2), all > 0.** *Derivation:* G5 — open loop, every cell positive and rising. Closed
loop the governor adds fuel on a low belief, which can only grow the overshoot it is reading. *Risk:* feedback could
saturate the error (a limiter cuts the fuel): a tie at the redline would fail the strict ordering; the limiters are
off in these cells. **112 cells.**

**H5 — a slow sensor loses to the tables in the move.** `EPR`: **move(exhaust, τ 2) > move(grid)** at every cell, and
**move(exhaust, τ 0.1) < move(grid)** at every cell with Mach ≤ 0.7. *Derivation:* G5 — at 0.25 better at Mach ≤ 0.5 in the design air,
worse at sea level from 700 K and at ≥ 1.4; at 1 worse everywhere; 0.1 and 2 bracket that. *Risk:* closed loop the
lagged sensor's own feedback grows its error (H4) while the grid's does not, which favours the first claim and threatens
the second; the hot sea-level Mach-0.2 cells (G5's sea level at 0.25 already lost) and Mach 0.7 (outside G5) are the
second claim's named risk. **112 cells** (the second claim applies at the 64 with Mach ≤ 0.7; a cell HITs only if
every claim that applies holds).

**H6 — the sensor helps the fan speed fast, and is a wash slow.** `N1`, `exhaust`, no lag, the start: at Mach ≥ 1.2,
**|EXH| < 0.7 × |rung 100's|**; at Mach ≤ 0.7, **|EXH| within ±30 % of |rung 100's|** (cells where |rung 100's| <
0.05 % VACUOUS). *Derivation:* G4 — 0.40–0.55 at Mach 1.4–1.6, 0.81–1.15 at Mach ≤ 0.5. *Risk:* Mach 0.7 near the
start's choke boundary (G4's Mach-0.5 cancellation). **112 cells.**

**H7 — the sensor helps the fan speed in the move.** `N1`, `exhaust`, no lag: **|move(EXH)| < |move(rung 100)|** at
every cell. *Derivation:* G4 — every cell open loop. *Risk:* closed loop. **112 cells.**

**H8 — the speed tables make the fan speed WORSE where B's errors cancelled.** `N1`, `grid`, the start: at Mach ≤ 0.25
and ≥ 1.2 (away from choke), **|grid| > |rung 100's|, same sign** (cells where |rung 100's| < 0.05 % VACUOUS).
*Derivation:* G4 — B's push and airflow errors oppose; the grid's airflow is near the truth, so only the push's error
is left, larger than the sum. *Risk:* Mach 0.25 and 3 km Mach 0.2 lie near G4's choke-boundary exceptions at 900 K.
**6 flights × 2 × 4 × 2 = 96 cells.**

**Total: 880 scored cells** (7 × 112 + 96).

**Reported, not scored:** every size beside rung 100's; the lagged sensor on the fan speed (τ 0.1 / 0.5 / 2); the
calibrated (generator) cells; Mach 0.5 (the 0.3–0.6 gap, a re-measure of G1); Mach 2.0 at 1000 K (G6's high-speed
corner); the proportional fan law under the sensor at one cell per flight; cost (each source against `table`,
governed EPR and fan speed at Mach 0.25 and 1.2, best of 9); the refusals counted by class.

## 4. What would change the plan

A MISS is a finding. The crash maps (§ 2) run before the spec is written. If a sensor-fed governor run costs more than
3× its `table` run, the cost is reported before the page is built. If G6's no-root risk stops more than 5 % of the
crash map's `exhaust` fan-speed runs, the refusal is reported to the user before the spec, with the alternative (the
row search widened, or the fuel term taken from the table) named.
