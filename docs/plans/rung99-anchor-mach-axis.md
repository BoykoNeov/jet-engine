# Rung 99 anchor — THE SPEED AXIS: the control's tables made at eight flight speeds and blended at the one flown, or the thrust recomputed for it (pre-registered 2026-10-18 — § 3)

**Seam:** rung 98 § *The next seam*: *"A Mach axis: a table made at several flight speeds and read at the one flown —
the correction rung 98 shows the `δ`/`θ` pair cannot make."* Rung 98's headline: a table made at Mach 0.85 gives
8.4–9.1 % too much thrust at Mach 0.5 and 4.6–9.7 % too little at Mach 1.4, both corrections on, because a speed change
moves only the thrust (the jet's push against the outside air and the intake's drag), never the engine's inside.

**The setup (user, 2026-10-18; memory `rung99-decisions`), asked before any probe:**
* the speed axis goes on **all three tables** (rung 96's thrust table, rung 48's acceleration schedule, rung 95's
  steady HP-speed table) — I recommended the thrust table only;
* beyond the grid the control **extends the trend** (the straight line through the two end tables) — I recommended
  holding the end table;
* the speed axis is **on by default** (`none` reproduces rung 98);
* the exact formula (rung 98's split: the nozzle's pressure term and the intake drag recomputed for the flight speed) is
  **also a setting** — I recommended a test-only reference;
* **the grid, asked after the reach probe:** Mach 0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0. My first default ended at
  2.0; the probe (G1) showed that extending past 2.0 needs the Mach-1.6 table, which cannot be built for Mach 2.5–3
  flights. The page's Mach control runs 0.05–3.5, so the trend is extended below 0.3 and above 3.0.

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung99-probe`, archived in
`docs/plans/scratch-archive/rung99/`): `reach.txt` / `reach_to3.txt` (do the bracketing tables build — counts and
failing flights), `sim.txt` (a) (how far the NON-thrust corrected columns differ between grid tables at equal corrected
throttle — the thrust column was not printed), `sim.txt` (b) and `cells.txt` (where rung 98's split identity holds,
spread and fail counts only; the scoring cells solve, y/n and unchoked counts), and — **after the advisor's pre-freeze
review** found an arithmetic slip in my first draft's bend estimate — `bend.txt`: the curve the grid interpolates and
the formula's unchoked error, both computed ANALYTICALLY from engine state, the model's nozzle area and its inlet
(G5, G6). **Disclosed:** `bend.txt` is close to the predicted quantity — it is the grid's error at a FIXED corrected
engine with the temperature residual neglected — so H1–H5 below are tests of that neglect and of the calibrated ends,
not blind guesses of a size; rung 98's published results (its H2 misses, its temperature residuals) are used too.
No grid or formula TABLE was built or read — no blended reading, no end's miss, no run. No rung-99 code exists.

---

## 0. GIVENS — measured before anything was written

The Controls engine is rung 98's (rung 97's two-shaft transient, the default move 1000 → 1400 K over 0.5 τ, settling
2 τ). The design flight is 250 K, 50 kPa, Mach 0.85. A **grid table** ("slice") at Mach `M_j` is built in the design
altitude's air, at the flight `(250 K, 50 kPa, M_j)`; the Mach-0.85 slice IS the design flight.

**G1 — the steady solver reaches the slow end; the grid must run to Mach 3** (`reach.txt`, `reach_to3.txt`). 162
flights (0–15 km, ±30 K, Mach 0.05–3.0), moves 1000 → 1400, 700 → 1400, 1400 → 1000, four maps. Each flight's two
bracketing slices (below the grid: 0.3 and 0.6; above: the last two) on 13 rows at `Tt4·Tt2_slice/Tt2`:
* rows where the nozzle is not choked **solve** (rung 97's branch is in the steady equilibrium too): 47–48 flights of
  the 700 → 1400 move have such rows (the 0.3 / 0.6 slices at low Mach, warm air); the 1000 → 1400 move has none;
* with the grid to 2.0, 135 of 162 flights build for the default move (rung 98's single table: 130) — the Mach-1.6
  slice fails for most Mach 2.5–3 flights (rows below ~600 K there). **With the grid to 3.0: 147–157 of 162** (rung 98:
  130–138; flat-LP 115 in both, its own ends refused first). The few left fail near the engine's floor (a Mach-2.5/3
  slice's row at a corrected ~350 K) and in cold air at 8 km, Mach 0.05–0.15 (the 0.6 slice's row at ~1815 K, the top).

**G2 — between slices the NON-thrust columns move by the temperature residual only, while choked** (`sim.txt` (a)). At
equal corrected throttle, each slice against the Mach-0.85 one: `EPR`, both corrected speeds, corrected fuel, corrected
airflow, `Wf/pt3` and the corrected HP speed differ by ≤ 3.3 % at Mach 2.0 (`θ` 1.57), ≤ 0.6 % at Mach 0.3 (`θ` 0.89),
about linear in `θ − 1` — rung 98's G2 (the fuel's fixed heating value), on every map and share/off variant. Where the
point is NOT choked (0.3 and 0.6 slices, 700–800 K) they move by up to 8–14 %. The fixed-amount variants (generator by
power, customer air by amount) move more (pressure similarity broken, rung 98 G1).

**G3 — the formula's one ingredient beyond the table** (`cells.txt`). At choked points read on a table whose rows are
all choked, rung 98's split constant `F_true − δ·F_tab(e) − [δ·V0_d·ṁ_tab(e) − V0·ṁ_face]` equals
`a8·(δ·p0_d − p0)` — `a8` the nozzle area the model already carries (`base.a8`, in its own per-design-airflow units)
— to ≤ 1.9e−15 of the thrust, every scoring cell. **Where the point or a row it reads is not choked, the identity
fails** (`sim.txt` (b): every unchoked point by more than 1e−9; at Mach 1.4–1.6 a 700 K start puts one design row below
choke and the spread over the run is 1.6e−3–9.8e−3).

**G4 — the scoring cells run** (`cells.txt`). At 250 K, 50 kPa: Mach 0.15, 0.5 and 1.4, moves 1000 → 1400 and
700 → 1400, every map — the run's own ends solve; both bracketing slices build and rise (rung 96's check); rung 98's
design rows build. Unchoked: the 700 K end at Mach 0.15 and 0.5 on every map; one 0.3-slice row (Mach 0.15) or two
(Mach 0.5) for the 700 K start; no other end or row. The 1000 → 1400 move is choked everywhere. Flat-LP refuses its
own 700 → 1400 ends at Mach 1.4. Mach 0.05 the same as 0.15 (the 700 K end unchoked, one 0.3-slice row).

**G5 — the curve the grid interpolates, from the model's own quantities** (`bend.txt`). At a fixed corrected engine
with a choked nozzle the net thrust over `pt2` depends on Mach alone (rung 98 G1/G3):
`φ(M) = C − a8·g(M) − ṁc·h(M)`, with `g = p0/pt2` and `h = V0/√Tt2` from the core's own `inlet()` at each flight
(the nozzle's push against the outside air; the intake drag), `a8` the nozzle area, `ṁc = ṁ_face·√Tt2/pt2` the
corrected airflow. So the grid's error at equal corrected state, the temperature residual NEGLECTED, is
`pt2·{−a8·[(1−w)g_a + w·g_b − g] − ṁc·[(1−w)h_a + w·h_b − h]}` — the table's belief minus the truth. Over the end's
thrust, all four maps within 0.03 points of each other:

| Flight (250 K, 50 kPa) | slices, `w` | 1000 K end | 1400 K end | parts at 1000 K (nozzle / drag) |
|---|---|---|---|---|
| Mach 0.5 | 0.3/0.6, +0.667 | +0.53 % | +0.26 % | +0.38 / +0.16 |
| Mach 1.4 | 1.2/1.6, +0.5 | −0.31…−0.33 % | **+0.027…+0.030 %** | −1.19 / +0.86 |
| Mach 0.15 (extended) | 0.3/0.6, −0.5 | −1.69 % | −0.83 % | −1.35 / −0.35 |
| Mach 0.05 (extended) | 0.3/0.6, −0.833 | −3.35 % | −1.64 % | −2.76 / −0.61 |

The drag term always bends the curve UP (chord above it: belief > truth). The nozzle term bends it up at low Mach and
DOWN above about Mach 0.8 (`d²/dM²(1+0.2M²)^−3.5 ∝ 1.6M² − 1`, `π_d` aside — my first draft wrote `2.32M² − 1`; the
advisor caught it). **At Mach 1.4 the two nearly cancel at the high throttle** — the nozzle's push is a larger share of
the thrust at low throttle, so there the downward bend wins. Also printed (reported in § 3): Mach 0.9–1.1 (the 0.85/1.2
bracket, spanning the inlet's recovery law turning on at Mach 1) ≤ 0.1 % at 1000/1400 K, up to −0.6 % at 700 K; Mach
3.2 (extended above) parts of ±4 % and ±20 % that nearly cancel — at 1400 K there the corrected throttle is near the
floor and the thrust small.

**G6 — the formula where the nozzle is not choked** (`bend.txt`, H5 ingredients). The formula carries the choked
nozzle's gross thrust `A9·[(1+γ)·p* − p0]`; unchoked, the true one is `A9·p0·γ·M9²`. At the 700 K start the jet's
`pt9/p0` is 1.62–1.66 at Mach 0.05 and 0.15, 1.79–1.82 at Mach 0.5 (choking near ~1.85); the true-minus-formula term over
the thrust (`γ` 1.30 / 1.36, bracketing the jet's) is −0.99…−1.77 % at Mach 0.05, −0.91…−1.70 % at 0.15, −0.01…−0.23 %
at 0.5; at 750 K it nearly vanishes. Against it, the airflow guess's error `V0·Δṁ` — first order, its sign not derived
— bounded with sim (a)'s ~5 % unchoked airflow shift: ≤ 0.22 % at Mach 0.05, ≤ 0.71 % at 0.15, ≤ 3.1 % at 0.5.

## 1. THE DEVICE — derived before coding

**The setting** (Controls view, Flight panel, beside rung 98's): *Correct for flight speed* `speed_corr` =
`grid` (**the default**), `formula`, or `none` (rung 98). It acts only under `table_at: design`, in a run that reads a
table, flown off the design flight — everywhere else it is inert. A missing key takes the default.

**The grid.** Eight slices at `(250 K, 50 kPa, M_j)`, `M_j` ∈ {0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0}. A flight at
Mach `M` reads the two slices around it — or, outside the grid, the two end slices (0.3 and 0.6 below, 2.5 and 3.0
above) — with the weight `w = (M − M_a)/(M_b − M_a)` (below 0 or above 1 outside: the straight line carried on). A
flight exactly at a grid Mach reads that slice alone. **Each slice is rung 98's design-flight table with the slice's
flight in the design's place:** rows at the run's throttles × `Tt2_slice/Tt2`, built there, carried by that slice's own
`δ_j = pt2/pt2_j`, `θ_j = Tt2/Tt2_j` (the two switches as in rung 98).

**The blend — at equal reading, never row by row.** The two slices' rows lie at different throttles, so each table is
blended on its own abscissa: the thrust table on the reading the control uses (`EPR` or LP speed), the steady HP table
on HP speed, the schedule on the corrected HP speed. The blended table's rows sit at the union of both slices'
abscissae; every other column there is `(1 − w)·y_a(x) + w·y_b(x)`, each slice read with rung 96's own interpolation
(end segments carried on). Inside both slices' rows this IS "read both tables at the same reading and blend the
answers"; beyond them the blended table's own end segments carry on. The blended table goes downstream unchanged
(rung 96's `Table`, rung 95's steady table, rung 48's schedule), and the thrust table must still rise (rung 96's
check — refused in words if not).

**The formula — the thrust table only.** Rung 98's carried design table, its thrust column replaced row by row by the
net thrust recomputed for this flight: `F̂ = d·F_d + d·V0_d·ṁ_d − V0·ṁ̂ + a8·(d·p0_d − p0)`, with `d` = `δ` (switch on,
else 1), `ṁ̂ = ṁ_d·d/t` (`t` = `√θ` switch on, else 1) the table's airflow carried to this flight, `V0` and `p0` this
flight's. Every term is linear in the row's own columns, so reading between rows gives the formula at any reading.
**By G3, wherever the point and its rows are choked, `F_true − F̂ = V0·(ṁ̂ − ṁ_face)` exactly** — only the table's guess
of the air swallowed is left. The acceleration schedule and the HP table have no closed form for a speed change; under
`formula` they are rung 98's.

**The ends.** Calibrated as in rung 98 on the blended (or formula) thrust table: each end sits where this engine's
steady reading equals the table's reading at rung 96's target; the end's thrust minus its target is the steady miss.

**Flown at a grid slice's own flight** (`T0`, `p0`, `M0` equal to a slice's, bitwise) under `grid`, the tables ARE
tables made at this flight — the code dispatches to rung 97's path. At the design flight every setting is rung 96
(rung 98's dispatch). **The Mach-0.85 slice is the design-flight object itself** (`design_flight()`, not a rebuilt
flight; a test asserts its Mach is 0.85), so **at Mach 0.85 with any other air** `grid` reads rung 98's own table, bit
for bit.

**The page.** The Flight panel shows the three-way choice under *tables made at the design flight*. The results table
says which slices were read, the weight, and whether the trend is being extended (and how far past the grid). Rung
98's two-part split of the pressure-ratio control's gap: under `grid` the gap is shown whole (no single table speed);
under `formula` it is split into the **airflow guess** `V0·(ṁ̂ − ṁ_face)` and **the rest** — zero while the nozzle is
choked, so a non-zero rest marks an unchoked point. Under `formula` with the pressure switch off the gap is shown whole
(the identity needs `δ`, as in rung 98).

**Refusals, in words.** A slice row with no steady point: *"The control's tables are made at Mach a and b in the design
altitude's air (250 K, 50 kPa). At Mach a this run's throttles are X–Y K, and at Z K the engine has no steady point
there: tables made there cannot cover this flight. Make the tables at this flight instead."* Under `formula`, rung 98's
words. A blended table that does not rise: rung 96's words for a table that cannot be turned back into thrust.

**Where it lives:** `rust/src/mach_axis.rs` — the grid, the bracket and weight, the blend, the formula column, the
words. `corrected_tables.rs` lends its `Carry` and builders with the slice flight as the design. `sandbox_controls.rs`
gains `speed_corr` and routes the three tables through it.

## 2. REDUCES and GATES (construction — no credit)

* **`speed_corr: none` ⇒ rung 98, bit for bit**; **`table_at: flight` ⇒ rung 97**; **no table ⇒ bit for bit**; **the
  design flight ⇒ rung 96** — every existing gate, oracle and golden.
* **`grid` at Mach 0.85, any air ⇒ rung 98, bit for bit** (the one slice is rung 98's table) — so rung 98's hot, cold,
  thin and dense tests do not move.
* **`grid` flown at a slice's own flight ⇒ rung 97, bit for bit** (dispatch), every slice.
* **The pressure gate, per slice (not self-certifying):** at each slice's Mach with `T0` = 250 K and `p0` ≠ 50 kPa
  (`θ_j` = 1 exactly), the carried slice matches a table built AT that flight by its own solves — every column of all
  three tables ≤ **1e−9** relative (rung 98's bar; its G1 measured ≤ 3.6e−11). Fixed-amount variants excluded.
* **The blend gate:** at random readings inside both slices' rows, the blended table's value equals
  `(1 − w)·(slice a) + w·(slice b)` ≤ **1e−12** relative, every column, all three tables; at `w` = 0 it is slice a.
* **The formula gate (not self-certifying — the engine's own thrust against the table):** on an `EPR` run under
  `formula` at a flight where every point and row is choked (Mach 1.4, default move), at every recorded point:
  `F_true − F̂ − V0·(ṁ̂ − ṁ_face)` ≤ **1e−12 × the thrust** (G3 measured ≤ 1.9e−15 on steady points; rung 96/98's bar).
* **What the default moves — pre-registered.** `grid` is the default, so a request changes ONLY if it reads a table,
  has its tables made at the design flight, and flies off Mach 0.85 (and off the design flight). Census after the
  build: the whole gate, and every changed test, golden and check request listed. Already visible: rung 98's Mach 0.5
  / 1.4 tests; its Mach-1.4 `EPR` check request (now on the 1.2/1.6 slices); its Mach-3 sea-level request (default map
  and move, `EPR`) — **still refused**, now in § 1's grid words: it reads the Mach-3 slice alone, whose row at 868 K has
  no steady point (G1, `reach_to3.txt`); any of plan § 29's Mach requests that reads a table and runs. A pre-existing test that
  moved is pinned to `speed_corr: none`, so it keeps testing rung 98; none is re-baselined silently.

## 3. PREDICTIONS — frozen here, scored after the build

Cells: the inferred-thrust governor, both readings (`EPR`, LP speed at the page's `τ_N` 0.1), the default move unless
named, the generator off, no lever, each of the four maps, both corrections on, **flights in the design altitude's air
(250 K, 50 kPa)** so that a slice's air differs from the flight's only by speed. The score is the steady miss of each
calibrated end: "over" = the end's true thrust above its target, "under" the reverse; an end refused for a reading out
of reach scores by the refusal's direction (rung 98's rule); a slice that cannot be built scores VACUOUS, stated (G4
found none); any other refusal or stop is a MISS, stated.

**The bend, and what neglecting the temperature residual costs.** G5 gives the grid's error at a FIXED corrected engine.
The predictions test the two things G5 leaves out: the temperature residual (G2 — the slices' air differs from the
flight's by speed only, so by `θ` too), and the calibrated end (the control holds a reading, so the miss is the belief
error at the reading it settles on). The slices' `θ` against the flight: 1.031 / 0.979 at Mach 0.5 (weights ⅓, ⅔ — net
first-order term −0.34 % of `θ`), 1.081 / 0.921 at Mach 1.4 (weights ½, ½ — cancelling to first order), 0.987 / 0.937 at
Mach 0.15 (weights 1.5, −0.5 — net +1.1 %, the blended table behaving as one made in slightly cooler air than the
flight's, which pushes toward "over"). Rung 98 measured the temperature residual of a whole table read at `θ` 1.16 as
+0.08 % (`EPR`) and +0.06…+0.25 % (fan speed): first order ≈ 0.5 % (`EPR`) and ≤ 1.6 % (fan speed) per unit `θ − 1`.
A belief above the truth makes the control settle short (under), below it overshoot (over).

**H1 — the grid's leftover miss has the sign of the bend.** Mach 0.5, both ends ⇒ **under**; Mach 1.4, the 1000 K
end ⇒ **over**. Both readings. *Derivation:* G5's belief − truth is +0.53 % / +0.26 % at Mach 0.5 and −0.31…−0.33 % at
Mach 1.4's low end, against a temperature term of ≤ 0.005 % (Mach 0.5, net first order × 1.6 %) plus second-order
terms of ~0.01 %. *Risk:* the calibrated end settles on a reading slightly off the end's throttle, where G5's number
moves with throttle — strongly at Mach 1.4, where it changes sign between 1000 and 1400 K. **Mach 1.4's 1400 K end is
NOT scored** (G5: +0.03 %, below what the neglected terms can move); its miss is reported. 2 readings × 4 maps × (2 + 1)
ends = **24 cells**.

**H2 — the grid shrinks rung 98's Mach miss below 1 %.** Mach 0.5 and 1.4, both readings, both ends: |steady miss|
< 1 % (rung 98: 4.6–9.7 %). *Derivation:* G5 ≤ 0.53 %; the temperature terms above add ≤ ~0.05 %. **32 cells.**

**H3 — extending the trend below the grid gives too much thrust: Mach 0.15 ⇒ over.** Both readings, both ends, the
default move (choked throughout, G4). *Derivation:* below Mach ~0.8 both terms bend the curve up, so the straight line
through the 0.3 and 0.6 slices, carried below 0.3, passes UNDER it: G5 −1.69 % / −0.83 %. The temperature term pushes the
same way (+1.1 % of `θ`). *Risk:* none named beyond the calibrated end. 2 × 4 × 2 = **16 cells.**

**H4 — the formula against the grid, where choked: the formula wins where the bend is large, the grid where it
cancels.** `EPR` and fan speed at Mach 0.5 (both ends), `EPR` at Mach 1.4's 1000 K end: |formula miss| < |grid miss|;
`EPR` at Mach 1.4's 1400 K end: |grid miss| < |formula miss|. *Derivation:* by G3 the formula's only error is the
airflow guess `V0·(ṁ_face − ṁ̂)`, the temperature residual of the corrected airflow at equal reading — its table is
rung 98's, carried from the design flight (`θ` 0.918 at Mach 0.5, 1.217 at Mach 1.4). Scaled from rung 98's `EPR`
residual (+0.08 % at `θ` 1.16, `V0` 290 m/s) by `θ − 1` and by `V0` (158 / 443 m/s): ≈ 0.02 % at Mach 0.5, ≈ 0.17 % at
Mach 1.4; on the fan speed the formula also inherits the gross thrust's temperature residual, ≤ ~0.13 % at Mach 0.5.
Against the grid's G5: 0.26–0.53 % at Mach 0.5, 0.31–0.33 % at Mach 1.4's 1000 K end (a 2× margin), 0.03 % at its 1400 K
end (a 6× margin the other way). *Risks, named:* the Mach-1.4 cells rest on a scaled rung-98 number, not a measurement;
the fan-speed cells at Mach 1.4 (formula ≈ 0.35 % against the grid's 0.03–0.33 %) are too close to call and are
**reported, not scored.** (2 readings × 4 maps × 2 ends at Mach 0.5) + (4 maps × 2 ends, `EPR`, Mach 1.4) = **24 cells.**

**H5 — where the nozzle is NOT choked the formula gives too little thrust.** Mach 0.05, the 700 → 1400 move, the
starting end (unchoked on every map, G4), the `EPR` reading: **under**. *Derivation:* the formula carries the choked
nozzle's push, `A9·[(1+γ)·p* − p0]`; unchoked, the true push is `A9·p0·γM9²`, smaller by `(γ+1)/(2γ)·A9·p0·(1 − x)²`
to second order (`x = p*/p0` < 1; zero, with zero slope, at the choke). G6: −0.99…−1.77 % of the thrust at this end,
against an airflow-guess error ≤ 0.22 % (at `V0` ≈ 16 m/s). The table believes more thrust than there is, so the
control settles short — at a LOWER throttle, deeper below choke, which only enlarges the term. *Risk:* the ≤ 0.22 %
bound uses sim (a)'s ~5 % unchoked airflow shift at equal throttle, not at equal pressure ratio. The fan-speed cells
(their error first order in the unchoked fan-speed shift, unsigned) are **reported, not scored.** 4 maps = **4 cells.**

**H6 — on the schedule and the HP table the speed axis is nearly inert while choked.** Mach 0.5 and 1.4, default move:
(a) the open-loop slam with the schedule on (margin 0.25) peaks within **10 K** of the same slam under `none`; (b)
the governed proportional HP-speed law (gain 20) settles within **0.1 %** HP speed of `none`. *Derivation:* those
tables have no thrust column, and at a fixed corrected engine nothing but the thrust moves with speed (rung 98 G3);
the grid only swaps rung 98's temperature residual (the design table at `θ` 0.918 / 1.217) for the slices' (`θ` near
1) — ≤ ~1 % of `Wf/pt3` or steady fuel at these `θ` (G2), against the 7–10 % of rung 98's whole `√θ` that moved the
peak ~60 K (rung 98 H3) and the settling speed 0.33 % (rung 98 H4). Scaled: ≤ ~6 K and ≤ ~0.03 %. *Vacuity:* a schedule
cell counts only if the leg holds the fuel at one point at least in one of its two runs. *Risk:* rung 98's H4 lesson —
a reading past the table's top row is clamped; the bar absorbs it unless the clamp moves by more than the residual.
4 maps × 2 flights × 2 = **16 cells**.

**Total: 116 scored cells.**

**Reported, not scored:**
* every miss's size (H1–H5), and the `none` (rung 98) miss at the same cells beside it;
* the cells taken out of scoring: Mach 1.4's 1400 K end (H1's sign), the fan-speed formula-against-grid cells at Mach
  1.4 (H4), the fan-speed cells of H5 — each with G5/G6's number beside its miss;
* the misses through Mach 0.85–1.2, a bracket that spans the inlet's recovery law turning on at Mach 1 (`π_d` has an
  infinite second derivative just above 1 — the curve's sharpest bend sits inside one interval);
* the misses at Mach 3.2 (extending above the grid) and Mach 0.05;
* the grid's effect on the schedule and the HP law at Mach 0.15 with the 700 K start, where slice rows are unchoked
  (G2's 8–14 %) — the one place those two tables' speed axis should matter;
* the fixed-amount variants at Mach 0.5 / 1.4;
* the formula's "rest" (the unchoked part) through the H5 runs.

## 4. What would change the plan

A MISS is reported as a finding (the sandbox rule). A failed root inside the clamp is a reported stop with its reason.
**The crash map runs before the spec is written** and is compared request by request with a worktree of the commit
before the model, every differing class read (rung 98's review lesson). **Cost:** under `grid` each table the run reads builds
its own two slices on its own rows (the thrust table 2 × 13 steady solves where rung 98 built 13; the schedule and the
HP table likewise), the clean table doubling the thrust table's share; timed natively against `none` before the page is
built.
