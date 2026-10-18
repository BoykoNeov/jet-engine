# Rung 100 anchor — BELOW CHOKE: the formula's jet push computed from the reading, a fan-speed formula that rebalances the LP shaft, and two slow tables in the speed grid (pre-registered 2026-10-18 — § 3)

**Seam:** rung 99 § *The next seam*: *"The formula with the unchoked nozzle: recompute the jet's push with the subsonic
exit when `pt9/p0` is below critical — on the pressure ratio a closed form (G6); on the fan speed it also needs the LP
shaft's unchoked work split (rung 97)."* Rung 99's headline: where the nozzle is not choked its formula gives too little
thrust read from the pressure ratio (−1.0…−1.4 %) and too much read from the fan speed (+4.4…+5.1 %).

**The setup (user, 2026-10-18; memory `rung100-decisions`).** The scope moved three times, each time on a measured
probe and the user's answer (the probes are § 0):
* first answers, before any probe: **both readings**, the fan speed **approximate**; the formula **replaced** (rung 99's
  kept only as a hidden reduce setting); the thrust worked out **at the reading**, not on the table's rows — I
  recommended the pressure ratio only;
* after G4 showed the user's fan-speed approximation (A) removes only a quarter to a half of the error, and a second
  device (B) more but overshooting, the user asked **why** — G5 — and then asked to **try a slow table in the grid**;
* after G6 (a Mach-0.05 slice cuts the grid's worst low-speed miss from 33 % to 1.8 %): **all four** — the exact
  pressure-ratio formula, a Mach-0.05 slice, **also** a Mach-0.15 slice (measured, G6), and the fan-speed formula
  **B**.

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung100-probe`, archived in
`docs/plans/scratch-archive/rung100/`) against the shipped crate at `f041d2d`, and — for the grid (G6, G7) — against a
worktree of `f041d2d` whose `GRID` constant alone was edited. **Disclosed:** that worktree IS the grid half of the
device (a constant), so G6/G7 are measurements of it, recorded as givens and NOT scored; § 3's grid predictions are
at flights, throttles and variants `gridlow` never ran. `fan2` ran the fan-speed device B (its probe copy) at Mach 0.1,
0.3, 0.4 and sea level — the scored cells (Mach 0.05, 0.15, 0.5) lie between and below them, so § 3's B sizes are
near-interpolations; its signs and orderings are where the score can still fail. `eprguess` ran the pressure-ratio
formula's arithmetic at the same unscored flights. No rung-100 code exists.

---

## 0. GIVENS — measured before anything was written

The Controls engine, cells and conventions are rung 99's (rung 97's two-shaft transient, design flight 250 K, 50 kPa,
Mach 0.85; "over" = the engine gives more thrust than asked). `push(pt9) = a8·[p9·(1 + γ·M9²) − p0]` is the nozzle's
gross thrust from its total pressure alone, `p9`, `M9` from the convergent nozzle at `pt9/p0`.

**G1 — the hot gas is a perfect gas of constant `γ` 1.30, `R` 285.92, on every Controls variant** (`givens.txt` A, ten
variants, `T` 500/1500 K, `f` 0/0.03). So `Tt9` cancels from the push in BOTH branches: choked `a8·[(1+γ)·p* − p0]`,
unchoked `a8·γ·p0·M9²`.

**G2 — the push identity holds at every steady point, choked or not** (`givens.txt` B, `bars.txt` (a)):
`F_true = push(π_n·EPR·pt2) − V0·ṁ_face`, worst 1.4e−14 of the thrust choked (4.5e−13 cooled), 1.5e−12 unchoked;
**absolute worst 6.5e−11 N per kg/s** (smallest thrust 3.3), ten variants × four maps × eight flights incl. sea level,
600–1500 K. The plant's choke flag and the push's agree at every point. So a pressure-ratio control that knows `a8`,
`π_n`, `γ` and `p0` reads the JET'S PUSH exactly, at and below choke; its only error is the air it believes swallowed.

**G3 — rung 99's formula equals that push only where its design rows are choked and the pressure switch is on**
(`givens.txt` C, its shipped table read at 200 readings): ≤ 3.3e−15 of the table's thrust span at choked readings with
all rows choked; **Mach 1.4 from 700 K** (one design row below choke): 3.9–4.4e−3 of the span at CHOKED readings;
**pressure switch off**: 64 % (the identity needs `δ`). Unchoked readings (Mach 0.05, 700 K): 0.9–1.05e−2.

**G4 — two fan-speed approximations at unscored flights** (`givens.txt` D, `fan2.txt`; plain engine, 700 K, design
air): rung 99's formula +7.1 % (Mach 0.1) / +1.9 % (Mach 0.4); **A** (the HP side read off the table AT the fan speed,
the LP turbine re-solved against `p0`) +5.1 % / +1.4 %; **B** (the table row whose HP side, its LP turbine re-solved,
gives the LP power the table needs at that fan speed) −2.2 % / −0.75 %. Over every variant and flight: A same sign as
rung 99, removing 25–50 %; B the other sign, |B|/|rung 99| 0.18–0.40 on the plain, cooled, customer-air, LP-generator
and bleed-lever engines, both readings' maps. **Exception: the HP generator by POWER** (a fixed amount, rung 98 G1's
pressure-similarity break): rung 99 −5.7 % and B −6.85 % at Mach 0.1, 800 K; B also −7.2 % against A −0.55 % on the
tilted map at Mach 0.4. Deep below choke at sea level (650–700 K, Mach 0.1–0.3) both refuse: the table's engine,
re-solved against the flight's air, has no operating point (rung 97's words) — 9 of 340 `fan2` points.

**G5 — why B cannot be exact** (`why.txt`). The true unchoked engine against the choked table row at the SAME
corrected burner temperature (plain, Mach 0.1, 700 K): fan speed 0.608 / 0.618, LP compressor ratio 1.795 / 1.829,
`pt45/pt2` 2.435 / 2.469. With the TRUE HP side, the LP re-solve reproduces the true thrust to 1e−13. So the whole
miss is the HP side: an unchoked engine leaves the table's one-parameter running line (at a given burner temperature
its fan turns slower; at a given fan speed it burns hotter). B finds the burner temperature (its row within ~1 K of the
truth at mild unchoke) but reads that row's compressors, which belong to the faster choked fan — so it credits too
much core pressure. Only a SECOND parameter can bring the engine back: a table made at this flight, or the flight
speed (the grid).

**G6 — two slow slices in the grid** (`gridlow_*.txt`, `gridlow_join*.txt`; the worktree, measured, NOT scored).
Calibrated start's miss, worst over plain / cooled / customer-air × four maps × both readings, flights 250 K Mach
0.1/0.2/0.25, sea level Mach 0.05/0.15/0.25, 5 km Mach 0.1 (Mach < 0.3, not a slice's own flight):

| start | reading | rung 99 grid | + Mach 0.05 | + Mach 0.05 and 0.15 | rung 99 formula |
|---|---|---|---|---|---|
| 700 K | pressure ratio | 16 % | 3.2 % | **0.88 %** | 34 % |
| 700 K | fan speed | 33 % | 1.8 % | **0.59 %** | 72 % |
| 1000 K | pressure ratio | 5.7 % | 0.76 % | **0.25 %** | 0.02 % |
| 1000 K | fan speed | 5.8 % | 0.73 % | **0.24 %** | 0.11 % |

Refusals: 17–19 of 84 at 700 K in every column (the flat-LP map's own ends; rung 99's grid adds 5 at sea level).
Mach ≥ 0.45: unchanged, bit for bit (the bracket is the same).

**G7 — the slow slices' reach** (`reach_low.txt`, rung 99's 162 flights × 3 moves × 4 maps). Grid slices built for
the default move: 149 (rung 99: 147), 157, 157, 115 by map; no flight newly refused; the two rung-99 failures at 8 km,
−30 K, Mach 0.05/0.15 (the 0.6 slice's 1815 K row) now build. Flights with unchoked slice rows (700 K start): 51 (rung
99: 48).

**G8 — the pressure-ratio push's leftover at unscored flights** (`eprguess.txt`, the push identity applied at rung
99's calibrated starts): the formula's error IS the airflow guess there, to the bit. Design air, 700 K, unchoked:
+0.09…+0.21 % (plain), +0.64…+1.47 % (cooled), +0.42…+0.91 % (customer air), against rung 99's formula −0.16…−1.37 %,
−2.1…−6.3 %, −1.5…−4.0 %. Sea level, 700 K: **+2.1…+5.3 % (plain), +7.2…+15.5 % (cooled)**, against rung 99's −11…−50 %.
Every unchoked airflow guess is POSITIVE: at a given pressure ratio the off-line engine swallows less air than the
table expects, so the control believes too much ram drag and the engine overdelivers. **The push is exact; the airflow
column has G5's one-line problem.**

**G9 — B against rung 99 where choked** (`bars.txt` (b)): ≤ 8.3e−14 of the thrust (cooled; ≤ 1.9e−15 elsewhere), six
variants, Mach 0.5 / 1.4 / sea-level 0.85, 1000–1300 K.

## 1. THE DEVICE — derived before coding

**Two hidden reduce settings** (request keys, not on the page): `low_grid` (default **on**) and `formula_choked`
(default **off**). `low_grid: off` is rung 99's eight-slice grid; `formula_choked: on` is rung 99's formula column.
Both missing ⇒ the defaults.

**The grid.** Ten slices at `(250 K, 50 kPa, M_j)`, `M_j` ∈ {**0.05, 0.15**, 0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0};
everything else is rung 99's (bracket, weight, blend at equal reading, slices carried by their own `δ_j`, `θ_j`, the
design slice IS the design flight, a slice's own flight dispatches to tables made there). The page's slowest Mach is
0.05, so rung 99's "extend the trend" now acts only above Mach 3.0.

**The formula — pressure ratio** (`speed_corr: formula`, `EPR`). At a reading `e`:
`F̂(e) = push(π_n·e·pt2) − V0·ṁ̂(e)`, with `push` the closed form above (`a8`, `π_n`, `γ` the model's; `pt2`, `p0`, `V0`
this flight's), choked iff `pt9·(2/(γ+1))^(γ/(γ−1)) > p0`, and `ṁ̂(e)` rung 98's carried design table's airflow at `e`
(rung 96's interpolation, end segments carried on). The table's thrust column is not read. The reading the control
demands for thrust `F` is the root of `F̂(e) = F`, bracketed from the rows' own `F̂`. Built only if `F̂` rises over the
rows (rung 96's words otherwise). Under this formula the pressure switch acts on the airflow guess alone.

**The formula — fan speed: B, the LP shaft rebalanced on the table's own gas-generator line.** Rung 98's design rows
(the same 13 steady points) carry, per row `x` (its design burner temperature): the LP-turbine inlet `pt45` (× `d`),
`Tt45` and the choked `Tt5` (× `θ` when its switch is on), `f45`, `η_LPT`, the LP turbine's specific power `w(x)`
(rung 62's per-shaft-air booking, the LP load the shaft carries in that steady state), the fan speed (× `t`), the
airflow (× `d/t`); each column linear in `x` between rows, end segments carried on. At a row parameter `x`:
* the LP turbine re-solved against THIS flight's `p0` — rung 97's `try_lp_turbine` at the row's `Tt45`, `f45`,
  `η_LPT`, `pt45` — gives `π_u`, `Tt5u`;
* its power `w_u(x) = w(x)·[h(Tt45) − h(Tt5u)]/[h(Tt45) − h(Tt5)]` (the row's flow, the re-solved work);
* the fan speed it holds, `ν(x)`: the table's fan speed at which the LP load equals `w_u(x)` (rows read `w → ν`);
* the thrust believed, `F̂(x) = push(π_n·π_u·pt45) − V0·ṁ̂(x)`.
The control reads `F̂` at fan speed `ν` by the root of `ν(x) = ν`, and demands for thrust `F` the `ν(x)` at the root of
`F̂(x) = F`. Where the LP turbine at `x` stays choked `w_u = w`, so `ν(x)` is the table's own fan speed and `F̂` rung 99's
formula in algebra (G9). Built only if `w` and the fan speed rise over the rows. The acceleration schedule and the HP
table stay rung 98's under `formula`.

**Refusals, in words.** A row whose LP turbine, against this flight's air, has no operating point (G4's deep cases):
*"The control's fan-speed formula: the table's engine at X K, with its exhaust pushing against this flight's air
(p0 kPa), has no working state — the nozzle cannot pass its flow. The formula cannot cover this throttle; make the
tables at this flight instead."* — at the build, or as a stop mid-run. A root not bracketed within the table's rows
and one row-span beyond: the same advice, naming the asked thrust or fan speed.

**The page.** The Flight panel's toggle keeps three choices; *formula*'s words say what it now knows (the nozzle's area
and the outside air). Under the formula on `EPR` the gap split shows **the airflow guess** and **the rest, which is
now zero at every point** (G2) — the page reads "nothing" off the rest itself (absolute bar 1e−9 N per kg/s), not off
the choke flags. Under the fan-speed formula the results row names the table row B settled on against the run's own
burner temperature.

**Where it lives:** `rust/src/unchoked_formula.rs` — the push, the two formula readers, their roots and words;
`mach_axis.rs` — the slow slices and `low_grid`; `inferred_thrust::Table` gains an optional formula reader consulted by
`inferred` / `reading_at` (which take the core for B); `sandbox_controls.rs` — the two hidden keys.

## 2. REDUCES and GATES (construction — no credit)

* **`low_grid: off` and `formula_choked: on` ⇒ rung 99, bit for bit** — every existing gate, oracle, golden.
* **`speed_corr: none`, `table_at: flight`, no table, the design flight ⇒ unchanged, bit for bit.**
* **The grid at Mach ≥ 0.3 ⇒ rung 99, bit for bit** (same two slices).
* **The slow slices at their own flights ⇒ tables made there (rung 97), bit for bit.**
* **The push gate (not self-certifying — the plant's thrust against the formula's push):** on `EPR` formula runs at
  Mach 0.05 (unchoked start) and 1.4 (choked), every recorded point: `|F_true − F̂ − V0·(ṁ̂ − ṁ_face)|` ≤ **1e−9 N per
  kg/s** (G2: 6.5e−11). Also asserted at build: `γ` constant on the run's gas.
* **The B gate (not self-certifying):** at a choked run (Mach 1.4, default move, both corrections) every recorded
  point's `F̂` under B against rung 99's formula column ≤ **1e−12 × the thrust** (G9: 8.3e−14).
* **B's rebalance gate:** at an unchoked calibrated end, `ν(x)` at B's root equals the end's fan speed ≤ 1e−9, and the
  re-solved LP turbine's `π_u` is above the row's choked `π` (the turbine expands less).
* **What the new defaults move — pre-registered, census by running against a clean worktree of `f041d2d`:**
  (a) `grid` runs off the design flight at Mach < 0.3 that read a table (`table_at: design`) — every other grid run
  bit for bit; (b) every `formula` run that infers thrust — `EPR` by ≤ 1e−12 of the thrust where its reading and rows
  are choked with `δ` on, more where not (G3's two classes: Mach 1.4 from 700 K, pressure switch off), fan speed by
  ≤ 1e−12 where choked; (c) formula runs whose schedule or HP table alone is read: unchanged. A pre-existing test,
  panel or check request that moved is pinned (`low_grid: off` / `formula_choked: on`) and keeps testing rung 99.

## 3. PREDICTIONS — frozen here, scored after the build

Cells: the inferred-thrust governor (rung 99's conventions), hold thrust, generator off unless named, both corrections
on, the default `to` 1400 K, the CALIBRATED START's steady miss ("over" > 0). An end refused for its own steady point
(the flat-LP map's low throttles) scores VACUOUS, stated; a refusal in § 1's formula or slice words is a MISS, stated.

**H1 — the slow slices hold the grid's low-speed miss under 1 %, where `gridlow` never flew.** Flights: sea level
+30 K (318.15 K, 101.325 kPa) at Mach 0.08 and 0.22; 3 km standard at Mach 0.12 and 0.22. Starts 750 K and 800 K.
Variants: plain, the LP generator by share 0.05, the bleed lever. Four maps, both readings: **|miss| < 1.0 %**, and
**|miss| < |rung 99's grid miss|** wherever rung 99's exceeds 0.3 %. *Derivation:* G6's worst 0.88 % / 0.59 % at 700 K
over 3 variants, 0.25 % at 1000 K; a 750–800 K start is shallower below choke. *Risks, named:* the hot day translates a
slow slice's rows down by `Tt2_j/Tt2` ≈ 0.79 (750 K → 590 K at the Mach-0.05 slice), below anything G6 built; the two
new variants were not in G6. 4 flights × 2 starts × 3 variants × 4 maps × 2 readings = **192 cells** (both claims per
cell: a cell HITs only if both hold).

**H2 — the pressure-ratio formula's miss is the airflow guess: over, and smaller than rung 99's.** Design air (250 K,
50 kPa), Mach 0.05 and 0.15, the 700 K start (unchoked on every map, rung 99 G4); plain, cooled (0.05/0.05), customer
air by share 0.05; four maps: miss **> 0** and **|miss| < |rung 99's formula miss|**. *Derivation:* G2 — the rest is
zero; G8 — every unchoked airflow guess positive, +0.09…+1.47 % at Mach 0.1–0.4, against rung 99's −0.16…−6.3 %. A
belief below the truth makes the control settle LOW and overdeliver; settling low goes deeper below choke, which only
enlarges both. *Risk:* rung 99's own miss shrinks toward Mach 0.4 (−0.16 % plain), so the ordering is safe only at the
slow end; Mach 0.5 is therefore **not in H2's ordering** (H3). 2 flights × 3 variants × 4 maps = **24 cells**.

**H3 — the pressure-ratio formula's miss stays small.** Same variants and maps at Mach 0.05, 0.15 and 0.5, 700 K start:
|miss| **< 0.3 %** (plain), **< 2.0 %** (cooled), **< 1.3 %** (customer air), and > 0 at Mach 0.5 too. *Derivation:* G8's
range at Mach 0.1–0.4 (0.21 % / 1.47 % / 0.91 % worst), the airflow guess scaling with `V0` and the unchoked airflow
shift; at Mach 0.05 rung 99's own split measured the guess at +0.045…+0.065 % (plain). *Risk:* Mach 0.5's 700 K start is
barely below choke (rung 99 G6), where the guess could change sign. 3 × 3 × 4 = **36 cells**.

**H4 — the repaired design row: Mach 1.4 from 700 K, `EPR` formula, the start: |miss| < |rung 99's formula miss|.**
Plain, three maps (flat-LP refuses its own end there). *Derivation:* G3 — rung 99's formula carried a design row below
choke as if choked, an error of 3.9–4.4e−3 of the table's span, absent from the exact push; the airflow guess at Mach
1.4 is rung 99 H4's ~0.17 %. *Risk:* G3's error is the worst over readings; at the calibrated start's own reading it
may be smaller than the airflow guess's change. **3 cells.**

**H5 — B overshoots: the fan-speed formula's miss is UNDER, and smaller than rung 99's.** H2's flights plus Mach 0.5,
700 K start, plain / cooled / customer air, four maps: miss **< 0** and **|miss| < |rung 99's formula miss|**.
*Derivation:* G4/G5 — B lands on about the right burner temperature but credits the choked fan's compressors, so it
believes more thrust than there is; the control settles high and underdelivers. Every G4 point except the fixed-amount
generator (not a cell here). *Risk:* at Mach 0.5 the start is barely below choke; G4's mildest points (cooled, 800 K,
Mach 0.3: B −0.07 % vs rung 99 −0.07 %) show the ordering vanishing there. 3 × 3 × 4 = **36 cells**.

**H6 — B removes most of the error: |B| / |rung 99| between 0.15 and 0.6** at H5's cells where rung 99's |miss| exceeds
0.5 %. *Derivation:* G4's 0.18–0.40 at 700 K over these variants and Mach 0.1–0.4. *Risk:* a calibrated end settles at
its own throttle, not G4's fixed one (B's lower settling deepens the unchoke, rung 99's higher one shallows it).
**≤ 36 cells** (counted at scoring; cells below the 0.5 % floor stated).

**Total: ≤ 327 scored cells** (H6's count fixed at scoring by its stated floor).

**Reported, not scored:** every miss's size, rung 99's beside it; the grid's misses at G6's flights under the shipped
code (a re-measure of a given); the 1000 K starts; the end at 1400 K; the fixed-amount variants (G4's exception) at H5's
cells; the formula's misses at sea level (G8's large airflow guess); B's chosen row against the true burner
temperature; the schedule and the HP table under the slow slices (Mach 0.1, 700 K: the one place those tables' speed
axis can matter); cost (governed `EPR` / fan speed under grid and formula at Mach 0.1 and 1.4 against rung 99's
`cost.txt`).

## 4. What would change the plan

A MISS is a finding. The crash map (rung 99's seed-99 generator over the 1 500 requests, plus `low_grid` and
`formula_choked` left at their defaults) runs before the spec is written, compared request by request against the
clean worktree, every differing class read against § 2's census. B nests a root over the row parameter, each step a
rung-97 turbine solve, inside the governor's demand — evaluated once per governor call (it is outside the fuel
residual, `governor.rs`); if a run costs more than 3× rung 99's formula run, the cost is reported before the page is
built.
