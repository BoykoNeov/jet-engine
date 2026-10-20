# Rung 99 — THE SPEED AXIS: the control's tables made at eight flight speeds and blended at the one flown, or the thrust recomputed for it

**Code:** `rust/src/mach_axis.rs` — `SpeedCorr`, the grid (`GRID`, `bracket`, `slice_flight`), `Tables` (rung 98's
design-flight carry, two grid slices, or the formula) with its three builders, the blends `blend_thrust` /
`blend_pairs`, the formula column `formula_table`, the formula's gap split. `sandbox_controls.rs` gains `speed_corr`
and routes rungs 48/95/96's table builds through `Tables`. `inferred_thrust::lin` is now public (the blend reads each
slice with it).
**Gates:** `rust/tests/rung99.rs`; rung 98's tests, panel and check requests that fly off Mach 0.85 are pinned to
`speed_corr: none`.
**Panel:** `print_mach_axis_table` (born in Rust, held to `rust/oracle/rust_owned/`; a change detector only).
**Anchor:** `docs/plans/rung99-anchor-mach-axis.md`, committed as `7113ed3` before any rung-99 code existed.
**Sandbox:** the Controls view's Flight panel (a three-way speed toggle), the results table's two rows
(`docs/plans/sandbox-plan.md` § 31).
**Scratch scripts and outputs:** `docs/plans/scratch-archive/rung99/`.

Rung 98's seam: *"A Mach axis: a table made at several flight speeds and read at the one flown — the correction rung 98
shows the `δ`/`θ` pair cannot make."* The user chose (memory `rung99-decisions`): the axis on **all three** tables;
**extend the trend** beyond the grid; **on by default**; the **exact formula also a setting**; and — after the reach
probe showed a grid ending at Mach 2.0 could not build Mach 2.5–3 flights — the grid **Mach 0.3–3.0**.

> **THE HEADLINE — a table at several speeds leaves the BEND of thrust against speed; recomputing the thrust leaves only
> the guess of the air swallowed, and only while the nozzle is choked.** At a fixed corrected engine the net thrust over
> the inlet pressure depends on Mach alone, through two terms: the intake drag always bends that curve up; the nozzle's
> push against the outside air bends it up below Mach ≈ 0.8 and down above. So blending two tables lands SHORT where the
> curve bends up (Mach 0.5: −0.27…−0.54 %) and OVER where the nozzle wins (Mach 1.4's low throttle: +0.29…+0.32 %),
> nearly exact where the two cancel (Mach 1.4's high throttle: −0.04 %), and carried past the grid it misses more (Mach
> 0.15: +0.8…+1.7 %) — against rung 98's 4.6–9.7 % (and ~20–27 % at Mach 0.05–0.15). The formula misses by
> 0.02 % at Mach 0.5 and 0.18 % at Mach 1.4 (the table's airflow guess, a temperature effect) — but where the nozzle is
> not choked it gives too little thrust read from the pressure ratio (−1.0…−1.4 %) and **too much read from the fan speed
> (+4.4…+5.1 %)**. On the schedule and the speed governor's table the axis does almost nothing while choked (≤ 1.9 K,
> ≤ 0.03 %): nothing but the thrust moves with flight speed.

## The device (anchor § 1)

* **The setting** `speed_corr` = `grid` (default) / `formula` / `none` (rung 98), under `table_at: design`, in a run that
  reads a table, off the design flight. Inert elsewhere.
* **Slices** at `(250 K, 50 kPa, M_j)`, `M_j` ∈ {0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0}; each is rung 98's design-flight
  table with the slice's flight as the design (rows at the run's throttles × `Tt2_slice/Tt2`, carried by its own
  `δ_j`, `θ_j`). The 0.85 slice IS the design-flight object. A flight reads the two slices around it — outside the grid
  the two end ones, the line carried on — with `w = (M − M_a)/(M_b − M_a)`; at a grid speed one slice.
* **The blend, at equal reading:** rows at the union of both slices' abscissae (the control's reading; HP speed;
  corrected HP speed), every other column `(1 − w)·y_a + w·y_b`, each slice read as its own reader reads it (rung 96's
  end segments carried on; rung 95's and rung 48's end values held).
* **The formula (thrust table only):** `F̂ = d·F_d + d·V0_d·ṁ_d − V0·ṁ̂ + a8·(d·p0_d − p0)` row by row on rung 98's carried
  table; the schedule and the HP table stay rung 98's.
* **Reduce / dispatch:** `none` is rung 98; `grid` at Mach 0.85 (any air) is rung 98's own table; `grid` at a slice's own
  flight is tables made there (rung 97); the design flight is rung 96.
* **The page:** the toggle under *tables made at the design flight*; the results row names the slices, the weight and
  whether the line is carried past the grid; under the formula the pressure-ratio gap splits into the airflow guess and
  the rest (zero while choked); under the grid it is shown whole.

## Gates (anchor § 2) — measured margins (`margins.txt`, two maps)

| Gate | Bar | Measured |
|---|---|---|
| `none` / `table_at: flight` / no table / design flight ⇒ old code | bit for bit | bit for bit (whole gate, goldens) |
| `grid` at Mach 0.85, hot / cold / thin air, four laws, two maps | bit for bit | bit for bit |
| `grid` at each slice's own flight ⇒ tables made there | bit for bit | bit for bit, 7 slices × 3 laws |
| Pressure-only flight per slice: carried slice vs tables built there | 1e−9 | 3.5e−12 (8 slices × 4 variants × 2 laws) |
| The blend = the two slices' answers blended, inside both | 1e−12 | 4.8e−16 (Mach 0.5, 1.4, 0.15) |
| The RUN's three tables = the blend of two slices rebuilt by hand (Mach 0.5) | bit for bit | bit for bit; ≠ rung 98's |
| The formula's rest after the airflow guess, choked run (Mach 1.4) | 1e−12 × thrust | 2.4e−15 × thrust |

**Red check** (the final review): a 0.1 % error put into the blend weight failed ONLY the run-tables gate above — the
blend gate reads the weight back from the run, and H1–H6's bars are wider than 0.1 % of a few-tenths miss. Before that
gate (added at the final review) no test could see a wrong weight. Restored.

**What the default moved** (census by running, against a worktree of `7113ed3`): of 83 Controls check requests, 2 —
exactly the two pre-registered (rung 98's Mach-1.4 `EPR` request and its sea-level Mach-3 one, the latter still
refused, now in the grid's words); both pinned to `none`. In the gate: rung 98's Mach-0.5/1.4/3 tests and panel (pinned
to `none`), and the browser page (rebuilt). No other test moved.

## Scored (anchor § 3) — 116 HIT, 0 MISS of 116 (`score.txt`)

* **H1 — the grid's leftover miss has the sign of the bend. 24 / 24.** Mach 0.5: −0.27 / −0.54 % (1400 / 1000 K, every
  map, both readings). Mach 1.4's 1000 K end: +0.29…+0.32 %.
* **H2 — below 1 %. 32 / 32.** Largest 0.54 %.
* **H3 — carried below the grid, too much thrust. 16 / 16.** Mach 0.15: +1.65…+1.71 % / +0.84…+0.86 %.
* **H4 — choked, the formula wins where the bend is large, the grid where it cancels. 24 / 24.** Formula `EPR` −0.023 %
  (Mach 0.5), +0.18 % (Mach 1.4) against the grid's 0.27–0.54 % and 0.30–0.32 %; at Mach 1.4's 1400 K end the grid's
  0.04 % beats the formula's 0.18 %.
* **H5 — unchoked, the formula gives too little thrust on the pressure ratio. 4 / 4.** Mach 0.05, the 700 K start:
  −1.03…−1.43 % (the end lands at ~697 K, deeper below choke); the rest of the gap up to 1.1–1.5 % of the thrust.
* **H6 — the axis nearly inert on the schedule and the HP table. 16 / 16.** Peak `Tt4` within 0.5–1.9 K; settling HP
  speed within 0.005 % (Mach 0.5) and 0.031 % (Mach 1.4).

**What the score is worth.** G5 computed the grid's error at a fixed corrected engine from the model's own state before
the freeze (disclosed in the anchor): H1–H4 test only what G5 neglects (the temperature residual, the calibrated end),
and the measured misses sit within 0.02 points of G5's numbers. H5's size was bounded by G6 (−1.0…−1.8 %); H6 is the
one family scored against a derivation alone. The anchor's first draft (before the advisor's review) had the nozzle's
bend turning at Mach 0.66 and a 2× margin at Mach 1.4; the true margin was 1.1–1.4× at the low throttle and none at the
high one, which is why Mach 1.4's 1400 K end was left unscored — it came out −0.04 %.

## Reported, not scored

* **The fan-speed formula where the nozzle is not choked: +4.4…+5.1 %** (Mach 0.05, 700 K start) — the opposite sign to
  the pressure ratio's. Split at the calibrated end (`fansplit.txt`): the airflow guess is ±0.04–0.06 % of the thrust
  (the table's airflow ~1 % off, but the flight speed only ~16 m/s); the **jet push** is +4.3…+4.9 % (pressure ratio:
  −1.1…−1.5 %, G6's term). Rung 97's mechanism: unchoked, the back-pressure takes work off the LP shaft, so at a given
  fan speed the jet keeps MORE pressure than the choked table expects — the formula's gross push, read at that fan speed,
  falls short of the truth. (Taken out of scoring before the freeze; the review corrected this spec's first wording,
  which blamed the airflow guess.)
* Fan-speed formula against the grid at Mach 1.4: formula +0.15…+0.29 % / +0.20…+0.40 %, grid +0.29…+0.31 % / −0.04 %.
* Across Mach (flow/press and flat-LP, both readings): Mach 0.05 grid +3.3…+3.4 % / +1.7 % (rung 98: +23…+27 %, the
  formula ±0.2 %); Mach 0.9–1.1 (the bracket spanning the inlet recovery's onset at Mach 1) grid within 0.1 %, rung 98
  −0.8…−4.9 %; Mach 3.2 refused on every setting (the default move's low throttle has no steady point there).
  *Since sandbox plan § 39 (2026-10-20)* the live page keeps a steady point below idle (and warns), so rows that had
  none now do: the Mach-3 sea-level move of G1 runs (from 168 N per kg/s below idle). This rung's tests run with the
  page's hidden `idle` setting at `old` (§ 37's rule), so every number above stands.
* The schedule and the proportional law at Mach 0.15 from 700 K (slow slices with unchoked rows): peak within 0.9 K,
  HP speed within 0.006 % — the schedule only cuts in higher up.
* Fixed amounts (`EPR`, grid / none): generator by power +0.19 / −0.06 % at Mach 0.5, +2.3 / +0.5 % at Mach 1.4 (none
  +9 % / −9 %); customer air by amount −0.8 / −0.3 % and +0.4 / −0.03 %.

## Crash map, compared request by request (`crash_compare.txt`)

1 500 random Controls requests (seed 99: rung 98's map plus the speed correction at random — grid 60 %, formula 20 %,
none 20 %), through this code and a worktree of `7113ed3`. **No panic on either.** 76 outcomes differ — **every one**
reads a table made at the design flight with a speed correction on, off Mach 0.85 for the grid (0 outside; 315 such
requests unchanged, mostly refused earlier for other reasons). 28 that rung 98 refused at Mach 1.5–3.5 (its design
table out of reach) now run (4 of them stopping early, for their own limiters); 31 ran with different numbers (23 grid,
8 formula) and 2 stopped early in both; 10 are refused in the grid's words instead of rung 98's (5 below Mach 1.5, 5
above); 2 formula refusals keep their class with new numbers. **Three that ran are now refused**,
each read: #828 asks for customer air by AMOUNT, more than half the compressor's flow at the Mach-0.3 slice's thinner
air (a fixed amount breaks pressure similarity, rung 98 G1); #1144 and #1235 put a slow slice's low row (~650–740 K) where
the engine has no operating point at all — the nozzle cannot pass the flow against air nearly at the engine's own
pressure (rung 97's limit) — while the flight itself, a little faster, runs. Those are the grid's own reach limits at
low speed and throttle, refused in words that say to make the tables at this flight.

**Cost** (`cost.txt`, best of 9, Mach 1.4): governed `EPR` 0.191 s (none) → 0.195 s (grid) / 0.185 s (formula); the
schedule 0.061 → 0.065 s; the proportional law 0.041 → 0.045 s.

## What it does to its neighbours

* **Rung 98 — its concession "no Mach axis" DISCHARGED**, and its headline sharpened: the speed miss is the bend of a
  curve with two terms, and it can be interpolated (grid) or computed (formula). Its sea-level Mach-3 refusal stands on
  the grid too (the Mach-3 slice cannot reach 868 K).
* **Rung 96 — the gap split extended:** under the formula the ram part becomes the airflow guess, and the constant
  nozzle part becomes zero while choked — so a non-zero rest marks an unchoked point.
* **Rung 97 — the unchoked branch matters to a control:** where the nozzle is not choked, a formula built on the choked
  nozzle mis-reads the thrust, by opposite signs on the two readings.

## Concessions

* The grid's slices are all in the design altitude's air; a flight elsewhere carries the temperature residual of rung 98
  to the nearer slices (smaller than rung 98's, not zero).
* Linear blending in Mach, linear extension past the grid (the user's choice); no curvature correction.
* The formula assumes the choked nozzle and knows the nozzle area; it corrects only the thrust table.
* Inherited: invented maps, rung 36's imposed stall line, the perfect gas, rung 95's lean-wall roughness, rung 98's
  fixed-amount loads (run, not corrected).

## The next seam

**The formula with the unchoked nozzle**: recompute the jet's push with the subsonic exit when `pt9/p0` is below
critical — on the pressure ratio a closed form (G6); on the fan speed it also needs the LP shaft's unchoked work split
(rung 97), since that, not the airflow guess, makes the +5 %. Or the steady matchers'
unchoked branch (rung 97's seam), or the governor on the lagged marchers (rung 95's).
