# Rung 100 — BELOW CHOKE: the formula's jet push from the reading, a fan-speed formula that rebalances the LP shaft, and two slow tables in the speed grid

**Code:** `rust/src/unchoked_formula.rs` — `Push` (the nozzle's gross thrust from its total pressure alone), `Formula`
(the pressure ratio's reader; the fan speed's B with its rows `GasGen`, `at_row`, `try_fan_root`), their roots and
words. `mach_axis.rs` gains the slow slices (`GRID`, rung 99's kept as `GRID99`; `grid`, `bracket(g, m)`,
`DESIGN_MACH`; a `Slice` carries its Mach) and `Tables::Formula { choked, .. }`. `inferred_thrust::Table` gains an
optional `formula`, read through `try_inferred` / `try_reading_at` (the plain readers refuse a table that has one);
`governor::Governor::try_demand` routes the run through it. `corrected_tables::try_design_table_rows` hands the
design equilibria to the fan formula. `sandbox_controls.rs` gains the two hidden reduce keys and `fan_rows`.
**Gates:** `rust/tests/rung100.rs`; rung 99's tests, panel and its three moved check requests are pinned to the reduce
settings (`low_grid: off`, `formula_choked: on`).
**Panel:** `print_unchoked_formula_table` (born in Rust, held to `rust/oracle/rust_owned/`; a change detector only).
**Anchor:** `docs/plans/rung100-anchor-unchoked-formula.md`, committed as `4b5c769` before any rung-100 code existed.
**Sandbox:** the Controls view's speed toggle ("ten speeds"), its note, the results rows (`docs/plans/sandbox-plan.md`
§ 32). **Scratch scripts and outputs:** `docs/plans/scratch-archive/rung100/`.

Rung 99's seam: *"The formula with the unchoked nozzle: recompute the jet's push with the subsonic exit when `pt9/p0` is
below critical — on the pressure ratio a closed form; on the fan speed it also needs the LP shaft's unchoked work
split."* The scope moved three times on measured probes, each by the user's answer (memory `rung100-decisions`): both
readings, the formula replaced, worked out at the reading; then — after the user's first fan-speed approximation (A)
measured weak and the user asked **why** — a grid slice at low speed; then all four: the exact pressure-ratio push,
slow slices at Mach **0.05 and 0.15**, and the fan-speed approximation **B**.

> **THE HEADLINE — below choke an engine leaves the table's one line, and only a second number brings it back.** A
> table made where the nozzle chokes describes the engine by ONE number (its throttle). Unchoked, the outside air
> pushes back on the LP turbine and the engine settles elsewhere: at a given burner temperature its fan turns slower
> and feeds the core ~1.4 % less pressure; at a given fan speed it burns hotter. **The jet's push itself is exact**
> from the pressure ratio and the outside air (zero rest at every point, choked or not) — but the pressure-ratio
> formula still reads the air swallowed off the table's line, and the fan-speed formula (B) finds the hotter burner
> but reads the choked engine's compressors, so it overshoots (−1…−6 % where rung 99 gave +4…+17 %). **The second
> number is the flight speed:** two slow tables made where the nozzle unchokes take the grid's worst miss BELOW MACH
> 0.3 from 16 % / 33 % to under 1 % on both readings (between Mach 0.3 and 0.6, untouched, the grid still misses by
> −0.95 % / −3.67 % from 700 K at Mach 0.5). And one exact term can remove a COMPENSATING error: at Mach 1.4 from 700 K
> rung 99's push and airflow came from the same unchoked design row and cancelled; the exact push leaves the airflow
> alone, −1.7 % against rung 99's +0.09 %.

## The device (anchor § 1)

* **The grid**: ten slices at `(250 K, 50 kPa, M_j)`, `M_j` ∈ {**0.05, 0.15**, 0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0},
  the rest rung 99's. The page's slowest Mach is 0.05, so "extend the trend" acts only above Mach 3.0.
* **The pressure ratio**: `F̂(e) = push(π_n·e·pt2) − V0·ṁ̂(e)`, `push = a8·[p9·(1 + γ·M9²) − p0]` (choked `p9 = p*`,
  `M9 = 1`; else `p9 = p0`, `M9` from `pt9/p0`), `ṁ̂` the carried design table's airflow. Root of `F̂(e) = F` for the
  demand. The pressure switch acts on the airflow guess alone.
* **The fan speed (B)**: on the design rows (carried; `x` = design burner temperature), the LP turbine re-solved
  against this flight's `p0` (rung 97), its power the row's scaled by the re-solved over the choked work at the same
  inlet; `ν(x)` the table's fan speed at that LP load; `F̂(x) = push(π_n·π_u·pt45) − V0·ṁ̂(x)`. The reading's thrust is
  `F̂` at the root of `ν(x) = ν`; the demand is `ν(x)` at the root of `F̂(x) = F`. Choked, `ν(x)` is the row's own and
  this is rung 99's column in algebra.
* **Reduce**: `low_grid: off` ⇒ rung 99's grid; `formula_choked: on` ⇒ rung 99's column (no reader built). Both are
  hidden request keys, written to a reply only off their defaults.
* **The page**: the toggle says *ten speeds*; under the formula on the pressure ratio the gap's rest reads "nothing"
  off the number itself (≤ 1e−9 N per kg/s); under the fan-speed formula the results name the table row it settles
  on at each end, beside the engine's own burner temperature.
* **Refusals**: a demand whose table row, against this flight's air, has no working state (rung 97's limit) — *"The
  control's fan-speed formula: a thrust of X lies beyond the table's rows, where the table's engine, with its exhaust
  pushing against this flight's air, has no working state …; make the tables at this flight instead."* (Rung 104: its
  driven case — sea level, Mach 0.1, 600 K — is rung 38's choked vane's; under the vane's own flow law the table's
  engine works there and the run starts, and lower down the engine or the speed table refuses first: kept on the pin.)

## Givens (anchor § 0) — the probes that moved the scope

* **G1** the hot gas is perfect, `γ` 1.30, every variant. **G2** `F_true = push(π_n·EPR·pt2) − V0·ṁ_face` at every
  steady point, choked or not: ≤ 1.5e−12 of the thrust, **6.5e−11 N per kg/s absolute**.
* **G3** rung 99's formula equals that push only with its design rows choked and `δ` applied (≤ 3.3e−15); Mach 1.4
  from 700 K (one design row unchoked) 4e−3 of the span; pressure switch off 64 %.
* **G4** the fan-speed approximations at unscored flights: rung 99 +7.1 %, **A** (the HP side at the fan speed) +5.1 %,
  **B** −2.2 % (plain, 700 K, Mach 0.1); A removes a quarter to a half, B 60–80 % but the other way.
* **G5 — why** (the user's question): at the same corrected burner temperature the unchoked engine's fan turns slower
  (0.608 / 0.618), its fan pressure ratio is lower (1.795 / 1.829), `pt45/pt2` 2.435 / 2.469; with the TRUE HP side the
  LP re-solve reproduces the thrust to 1e−13. The whole miss is the HP side: the engine off the table's line.
* **G6** the slow slices (measured on the device, not scored): worst low-speed miss 16 % / 33 % (rung 99's grid) →
  3.2 % / 1.8 % (+ Mach 0.05) → **0.88 % / 0.59 %** (+ 0.05 and 0.15), 700 K start, three variants.
* **G7** reach: grid tables build for 149 flights of 162 (rung 99: 147), none newly refused.
* **G8** the exact push's leftover IS the airflow guess, always positive unchoked: +0.1…+1.5 % in the design air,
  **+2…+15 % at sea level from 700 K** (cooled engine worst) — the table's airflow is read off its line too.
* **G9** B against rung 99 where choked: ≤ 8.3e−14 of the thrust.

## Gates (anchor § 2) — measured

| Gate | Bar | Measured |
|---|---|---|
| `low_grid: off` + `formula_choked: on` ⇒ rung 99 | bit for bit | rung 99's suite, panel and check requests pinned; census |
| grid at Mach ≥ 0.3 ⇒ rung 99's | bit for bit | bit for bit (Mach 0.3, 0.5, 1.4, both readings, schedule) |
| slow slices at their own flight ⇒ tables made there | bit for bit | bit for bit, both readings |
| the push gate: rest after the airflow guess, every point | 1e−9 N per kg/s | ≤ 5e−12 (Mach 0.05 unchoked and 1.4, two maps) |
| B against rung 99's column, choked run | 1e−12 × thrust | holds at every point (Mach 1.4) |
| B's rebalance at an unchoked end: `ν(x)` = the end's | 1e−9 | holds; `π_u` above the choked pin, the row hotter |
| the push continuous through choke | 1e−6 | holds |

## Scored (anchor § 3) — 242 HIT, 35 MISS, 40 VACUOUS of 317 (`score.txt`)

* **H1 — the slow slices hold the grid under 1 %, where the probe never flew. 140 / 168 (24 vacuous: the flat-LP
  map's own ends).** Every one of the 28 misses is the **fan-speed control with the LP generator** (by share) — not the
  speed axis: that cell misses by +4…+10 % with tables made AT THIS FLIGHT, and +10.4 % at the design flight itself
  (`genlp.txt`), because rung 96's clean table (made with the generator off) is read on an engine whose fan carries
  the load; with the loaded table it is ~0.1 %. The grid's own share there is under 1 %. **Without that variant,
  112 / 112: worst 0.90 % against rung 99's grid's 24.7 %.** The cell choice, not the slices, missed.
* **H2 — the pressure-ratio formula's miss is over and smaller than rung 99's (Mach 0.05, 0.15). 20 / 20.**
  +0.04…+0.76 % against −0.86…−6.2 %.
* **H3 — it stays small. 27 / 30.** The three misses are Mach 0.5 on the plain engine, where the 700 K start is barely
  below choke: −0.002…−0.02 % — the airflow guess there changes sign (named risk). Sizes within the bars.
* **H4 — the repaired design row. 0 / 3 — the finding.** Mach 1.4 from 700 K: −1.51…−1.75 % against rung 99's
  +0.07…+0.09 %. The calibrated start reads below the design table, whose lowest row is unchoked at the design
  flight; rung 99 took its push AND its airflow from that row and the two errors cancelled (`h4why.txt`): the rest
  is now 1e−13, the airflow guess −0.46 % of the air × 444 m/s against a thrust of 75 N per kg/s. At Mach 1.6 the
  same mechanism gives −21 % against rung 99's −4.5 % (reported).
* **H5 — B overshoots and is smaller than rung 99. 29 / 30.** −0.01…−6.4 % against +0.03…+17 % (Mach 0.05/0.15: −0.88…
  −6.4 % against +4.0…+17 %). The miss: Mach 0.5,
  press/flow map, −0.17 % against rung 99's +0.14 % — barely unchoked, the ordering vanishing (named risk).
* **H6 — B removes most of the error: ratio 0.15–0.6. 26 / 26.** 0.22–0.60.

**What the score is worth.** G4/G6/G8 ran the device's own arithmetic (B's probe copy, the grid's constant, the push)
at flights between and beside the scored ones, so H1–H3, H5, H6 are near-interpolations; disclosed in the anchor. The
two families the probes did not reach both missed: H4 (a design row unchoked — no probe ran one) and H1's LP-generator
cells (a variant the grid probe did not run, whose own miss dwarfs the bar).

## Reported, not scored

* Mach 0.5 from 700 K under rung 99's own grid (0.3/0.6 slices, unchanged): −0.95 % (pressure ratio), −3.67 % (fan
  speed) — the 0.3 slice's 700 K row is unchoked, the 0.6 one's is not.
* Mach 1.4 fan-speed formula from 700 K: −4.66 % against rung 99's −6.57 %.
* Sea level from 700 K, pressure ratio: +2…+5 % (plain), +7…+15 % (cooled) — G8's airflow guess.
* The fan-speed formula refuses 4 of 72 low-throttle sea-level requests (`findref.txt`), in § 1's words.

## Census — what the new defaults moved (`census.txt`, `crash_classified.txt`)

By running, against a clean worktree of `f041d2d`. **Check requests: 3 of 91 moved — exactly the pre-registered
classes:** rung 99's Mach-1.4 pressure-ratio formula request (choked: rounding), its Mach-0.15 fan-speed grid request
(now the slow slices), its Mach-0.05 formula request from 700 K (unchoked); all three pinned to the reduce settings.
**Crash map** (seed 99, 1 500 requests, the new keys at their defaults): **no panic on either build; 13 outcomes
differ, every one in a predicted class** — 4 of the 47 grid requests below Mach 0.3 that carry a table, 9 of the 24
formula requests that infer thrust; none outside. 11 ran on both (new numbers); 2 refused on both, in the same class
(one formula request with new numbers, one slow-slice row out of reach in the grid's words). In the gate: rung 99's
tests and panel (pinned), the browser page (rebuilt), the settings round trip (two keys).

**Cost** (`cost.txt`, best of 9): Mach 0.1 from 700 K — grid 0.507 → 0.283 s (pressure ratio; the slow slices build
nearer the flight), formula 0.29 → 0.30 s and 0.26 → 0.31 s (fan speed); Mach 1.4 — fan-speed formula 0.170 → 0.212 s
(B's root at every reading), the rest unchanged.

## What it does to its neighbours

* **Rung 99 — its seam CLOSED** for the pressure ratio (the push exact below choke) and answered for the fan speed (B,
  and why no single-line formula can be exact); its "extend the trend" now inert below Mach 3; its H5 numbers are
  its reduce setting's.
* **Rung 97 — the unchoked branch is a control's problem twice:** the push (solved) and the engine's position off the
  line (needs a second number).
* **Rung 96 — the airflow guess is the whole pressure-ratio miss** wherever the push is computed; its clean-table
  calibration on an LP-loaded engine is a larger effect than any speed axis (H1's misses).

## Concessions

* B is a stand-in for an on-board engine model; exact needs the full engine solve (= tables made at this flight).
* The airflow column is read off the table's line: no correction is attempted (the next seam).
* The slow slices are in the design altitude's air; the hot-day reach was not probed before the score (it built).
* Inherited: invented maps, rung 36's imposed stall line, the perfect gas (the push's closed form asserts it), rung
  95's lean-wall roughness, rung 98's fixed-amount loads.

## The next seam

**The airflow off the line:** the pressure-ratio formula's whole miss is `V0·(ṁ̂ − ṁ)`, the table's airflow at the
reading; below choke (and at a low throttle flown fast) the engine is off the table's line, so `ṁ̂` is wrong by
0.5–3 %, magnified by `V0` against a small thrust. B's rebalance (the burner temperature found from the LP shaft)
could also fix the airflow row — or the slow slices could supply the airflow beside the exact push. **The grid between
Mach 0.3 and 0.6 at a low throttle:** the same off-the-line engine (the 0.3 table's 700 K row unchoked, the 0.6 one's
not) leaves −3.7 % on the fan speed at Mach 0.5 from 700 K — a table near Mach 0.45, or a finer slow grid. **The
formula at a low throttle flown fast** (Mach 1.4–1.6 from 700 K: −1.7 % / −21 %, worse than rung 99's −0.1 % / −4.5 %):
rung 99's column kept as a page choice, or the airflow fix above. Or the steady
matchers' unchoked branch (rung 97's seam), or the governor on the lagged marchers (rung 95's).
