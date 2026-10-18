# Rung 98 anchor — THE CORRECTED TABLES: the control's three tables made ONCE at the design flight, their readings corrected for the air's pressure and temperature when flown elsewhere (pre-registered 2026-10-18 — § 3)

**Seam:** rung 96 § *The next seam*: *"the control's own corrections: the table made at one flight condition and flown
at another, its readings corrected by `δ`/`θ` as a real schedule's are."* Rung 96's concession: *"The table is made at
the run's own flight condition, so the corrections for flight condition cancel."*

**The setup (user, 2026-10-18; memory `rung97-decisions`, `rung98-decisions`):**
* **all three tables** are made once at the design flight (250 K, 50 kPa, Mach 0.85): rung 96's thrust table, rung
  48's acceleration schedule, rung 95's steady table for the proportional HP-speed law;
* **two switches**, each on or off: correct for the air's PRESSURE (`δ`) and for its TEMPERATURE (`θ`);
* the table's **rows cover the run's own range**, translated to the design flight (the corrected-equivalent throttles),
  so at the design flight everything is rung 96 bit for bit;
* **a third setting, asked before freezing:** *tables made at* — the DESIGN flight (**the default**, both corrections
  on) or THIS flight (rung 96/97's behaviour, kept as the reduce setting). The advisor proposed the setting so saved
  off-design runs could still be reproduced; a side note pointed out that defaulting it to "this flight" would hide the
  rung; the user chose "design flight" as the default. Saved off-design runs that use a table therefore change; nothing
  moves at the design flight or in a run that uses no table.

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung98-probe`, archived in
`docs/plans/scratch-archive/rung98/`). It printed steady similarity residuals at equal corrected throttle (`sim.txt`),
whether the design-flight rows solve (`reach.txt`), whether the scoring flights' own ends and tables solve (`cells.txt`,
`cells2.txt` (a)), and the SPREAD of the cross-flight thrust split (`cells2.txt` (b) — the spread only, not its value).
**Disclosed:** an earlier probe (`docs/plans/scratch-archive/rung97/sim.txt`) and G2 below printed the steady
temperature residual of the corrected STATE (hotter air ⇒ every corrected quantity slightly higher at equal corrected
throttle). Neither printed a table READ: no reading turned into thrust, no end's thrust miss, no governed or open-loop
run, no `Tt4`, flow coefficient or speed of any move. No rung-98 code exists. § 1–§ 3 were written after these prints.

---

## 0. GIVENS — measured before anything was written

The Controls engine is rung 97's: rung 43's rig, the default move 1000 → 1400 K over 0.5 τ, settling 2 τ, step 0.02.
The hardware is captured at the design flight (plan § 27); `δ` and `θ` are the INLET-TOTAL ratios, `δ = pt2/pt2_d`,
`θ = Tt2/Tt2_d`, both from the transient core's own `inlet()` at the flight flown and at the design flight (so each is
exactly 1 at the design flight — the same call on the same arguments).

**G1 — the pressure correction is EXACT** (`sim.txt`, G1 rows). At the design `T0` and Mach, with `p0` at six
non-power-of-two values from 5.5 to 101 kPa, the steady engine at the same burner temperature reproduces the design
flight's corrected thrust `F/δ`, `EPR`, `ν_L`, `ν_H`, fuel `/δ`, face airflow `/δ`, `Wf/pt3` and the corrected HP speed
`n_H` to ≤ 3.6e−11 relative (every map; plain, generator by SHARE on either shaft, customer air by share, cooling
0.05/0.05, stator, bleed, both levers). *Why:* the model's air, burner and nozzle carry no Reynolds number or absolute
pressure; every flow scales with `pt2`. (The earlier 0.0 at `p0 × ½` was halving, exact in binary.)
**Two variants break it, as expected:** a generator set by POWER and customer air set by AMOUNT are fixed quantities,
not fractions, so they weigh more in thinner air (corrected thrust off by 5–100 % where both points solve; many
points do not solve at all). They are
reported apart from every score below.

**G2 — the temperature correction is NOT exact** (`sim.txt`, G2 rows). At `p0` 50 kPa, Mach 0.85, `T0` 216.65–320 K
(`θ` 0.87–1.28), the engine at the corrected-equivalent throttle `Tt4·θ` reads every corrected quantity slightly HIGHER
in hotter air and lower in colder: at `θ` 1.28 and 1400 K, `F/δ` +2.0 %, `EPR` +1.4 %, `ν_L/√θ` +0.44 %, fuel
`/(δ√θ)` +1.6 %; about linear in `θ − 1`, the same sign in all eight columns on every map and share/off variant. *Why
(the earlier probe's reading, not separated further here):* the fuel's heating value is fixed while the air's enthalpy
scales with `θ` — `f = (h4−h3)/(η_b·h_PR − h4)` — so the hot engine at the same corrected throttle is not the same
corrected engine.

**G3 — Mach, at fixed inlet totals, moves ONLY the thrust** (`sim.txt`, G4 rows). With `T0` and `p0` chosen so `Tt2`
and `pt2` equal the design's, Mach 0.6–2.0 reproduces `EPR`, both speeds, fuel, face airflow, `Wf/pt3` and `n_H` to
≤ 1.6e−13 (0.0 at most), on every map and share/off variant, wherever the nozzle stays choked. At Mach 0.3 the 800 K
point unchokes and the identity breaks (by up to 3 %). *Why:* with a choked nozzle nothing downstream of the inlet sees
`p0`; only the net thrust does — the nozzle's pressure term `A9·(p9 − p0)` and the ram drag `V0·ṁ`. **No correction by
`δ`/`θ` can remove a speed change**: it lives in the thrust, not in the engine's state.

**G4 — the design-flight rows can be built only within reach** (`reach.txt`). Rows at `Tt4·(Tt2_d/Tt2)` for the moves
1000 → 1400, 1400 → 1000 and 700 → 1400 over 90 flights (0–15 km, ±30 K, Mach 0.3–3.0): every row solves and every
table rises in thrust, `EPR`, `ν_L` and fuel (rung 96's check), EXCEPT where the translated throttle falls below about
500 K at the design flight — `θ ≳ 2` (Mach 2.5–3; 21–23 of 90 flights for the default move on three maps), and for a
700 K start at `θ ≳ 1.4` (Mach 1.6–2.5; 7–9 flights). The flat-LP map has no such failures because its own ends are
refused first there (43 of 90). Cold air at altitude moves the rows UP (1400 K → up to ~1750 K); none failed.

**G5 — the scoring flights run, and their tables can be built** (`cells.txt`, `cells2.txt` (a)). Hot (290 K, 50 kPa,
M 0.85, `θ` 1.16), cold (216.65 K, `θ` 0.867), thin (26.5 kPa, `δ` 0.53), dense (80 kPa, `δ` 1.6), Mach 0.5 and Mach 1.4
(250 K, 50 kPa): both ends of the default move solve, choked, on all four maps; and the 13 design-flight rows of the
default move (the throttles all three tables use, 822–1151 K at Mach 1.4 up to 1154–1616 K cold) solve, choked, and rise
in thrust, `EPR`, `ν_L` and fuel, on all four maps. (Added after the advisor's review: `cells.txt` alone had checked
only the flight's own ends.)

**G6 — the cross-flight thrust split is an identity to 1.65e−13** (`cells2.txt` (b)). For a table made at the design
flight and read at another, at a choked steady point with pressure ratio `e`:
`F_true − δ·F_tab(e) − [δ·V0_d·ṁ_tab(e) − V0·ṁ_face]` takes the same value at every throttle (the 9 points 1000–1400 K of
each of the six scoring flights, every map, all ten variants — the fixed-amount ones included, since the identity needs
only the choked nozzle, not similarity). Its spread over the largest thrust is ≤ 1.65e−13 (cooled, Mach 1.4; ≤ 1e−13
otherwise). Only the spread was printed, not the value. *Why:* the convergent choked nozzle's gross thrust is
`G = A9·[(1+γ)·c·EPR·pt2 − p0]`, so `G(e) − δ·G_d(e) = A9·(δ·p0_d − p0)` whatever `e` is; the rest is ram drag.

## 1. THE DEVICE — derived before coding

**The settings** (Controls view): *Tables made at* `table_at` = `design` (the default) or `flight`; *Correct for air
pressure* `corr_p` (default on) and *for air temperature* `corr_t` (default on), which act under `design` only. All
three are inert when the run uses no table — no inferred-thrust governor, no acceleration schedule, no proportional
HP-speed law. Every saved setting stays readable; a missing key takes the default.

**The rows.** Under `design`, each table is built on the same machine (the generator-off one when rung 96's clean table
is chosen) at the DESIGN flight, at rung 96/48/95's own row throttles multiplied by `Tt2_d/Tt2` — always, whatever
the switches: where the table's rows lie is the table's coverage, not a correction. Its end rows are therefore NOT the
run's ends.

**The corrections — the table carried to the flight flown.** With `δ` (switch on, else 1) and `θ` (switch on, else 1):
* **thrust table** (rung 96): thrust `× δ`, `EPR` unchanged, LP speed `× √θ`, fuel `× δ√θ`, face airflow `× δ/√θ`;
  the `EPR` is read with the flight's own `pt2` (the probe sits in this inlet), and the flight's `V0` is the table's;
* **steady HP-speed table** (rung 95): speed `× √θ`, fuel `× δ√θ`;
* **acceleration schedule** (rung 48): `Wf/pt3 × √θ`. Its abscissa is ALREADY the model's corrected HP speed
  (`ν_H·√(Tt25_d/Tt25)`), and `Wf/pt3` already divides out pressure — so **the pressure switch does nothing to it, by
  construction**: a ratio unit is its own pressure correction.

Carrying the table is the same as correcting each reading the other way, and leaves every downstream read (rung 96's
`Table`, rung 95's `steady_fuel`, rung 48's `cap`) unchanged.

**The page's thrust split.** Rung 96's page splits the pressure-ratio control's gap `F_true − F̂` into ram drag, the
whole of it at the table's own flight. Under `design` with `δ` on, the split has two parts (G6): the RAM-DRAG part
`δ·V0_d·ṁ_tab(e) − V0·ṁ_face` (the design table's airflow at the reading, flown at the design speed, against the air
actually swallowed at this speed), and the NOZZLE part, the rest — a constant through the run, the nozzle's pressure
term at this ambient against the design's. With `δ` off the gap is shown whole, unsplit (the identity needs `δ`). The
LP-speed control's gap is shown whole, as in rung 96. The exponent is the textbook ½; no exponent is fitted
(that would be an unanchored constant). **The `EPR` law has nothing for `θ` to correct** — `EPR` and `F/δ` contain no
temperature — so the temperature switch is inert on it, by construction.

**The ends.** Under `design`, flown away from the design flight, an inferred-thrust run's ends are CALIBRATED as rung
96's clean-table ends are: each end is where this engine's steady reading equals the carried table's reading at the
target thrust (rung 94's `try_hold_root`, 600–2000 K, refused in rung 96's words if out of reach). The target is rung
96's (this engine's thrust at the set throttle at this flight; the generator-off engine's under the clean table). The
end's thrust minus its target is the **steady miss**, scored apart from the move. The proportional laws (HP or LP) are
not calibrated: they start on the set-throttle end and settle where their own law balances; that droop is reported.

**Flown AT the design flight** (`T0`, `p0`, `M0` equal to the design's, bitwise), `design` IS rung 96/95/48's own
table: the code path dispatches to it, whatever the switches.

**Refusals, in words.** A row with no steady point at the design flight: *"The control's tables are made at the design
flight (250 K, 50 kPa, Mach 0.85). In the design flight's terms this run's throttles are X–Y K, and at Z K the engine
has no steady point there: a table made at the design flight cannot cover this flight. Make the tables at this flight
instead."* A calibrated end out of reach: rung 96's `refusal_words`, prefixed by the design-flight table.

**Where it lives:** `rust/src/corrected_tables.rs` — the flight's `δ`/`θ`, the row translation, the three carries, the
words. `sandbox_controls.rs` gains the three settings and routes the three table builds through it.

## 2. REDUCES and GATES (construction — no credit)

* **`table_at` = `flight` ⇒ rung 97, bit for bit** — every existing gate, oracle and golden, and every off-design
  request of the Controls checks.
* **Flown at the design flight ⇒ rung 96/97, bit for bit**, under every switch (dispatch).
* **No table in use ⇒ bit for bit** under every setting.
* **Inert by construction, bit for bit:** the temperature switch on an `EPR` run with no other table; the pressure
  switch on the acceleration schedule.
* **The pressure gate (not self-certifying):** at a pressure-only flight (`T0`, `M0` the design's, so `θ` = 1 exactly)
  with `δ` on, the carried design-flight table matches a table built AT that flight by its own solves — every column of
  all three tables ≤ **1e−9** relative (G1 measured ≤ 3.6e−11 on the share/off variants; over the bar means investigate,
  not relax). The fixed-amount variants are excluded (G1).
* **The pressure gate on a run:** there, a governed `EPR` run with `δ` on matches the `flight` run at every recorded
  point, thrust and `Tt4` ≤ **1e−7** relative. *Bar derived, not measured:* the calibrated ends land by rung 94's
  bisection to 1e−9 K, on a reading off by ≤ 3.6e−11 relative (G1), so each end's `Tt4` moves by ~1e−7 K (~1e−10
  relative); the governor's fuel root is held to 1e−13 absolute (`LEG_TOL`) on fuel ~1e−2. Expected ≤ ~1e−9, the bar
  100× above it.
* **H1 of the first draft, moved here as a SIGN gate (the advisor: it is construction).** Pressure correction OFF, both
  readings, every map, both ends: in thinner air (26.5 kPa) the end's thrust is below its target, in denser air (80 kPa)
  above — or the end is refused as a reading below the least the engine reads (thin) / above what it reads at 2000 K
  (dense). *Why it cannot fail:* at equal corrected state the thrust is `δ` × the design's (G1); uncorrected, the target
  `F` is looked up where the table holds `F/δ`, and both the table and this flight's engine rise in thrust with the
  reading. Any other refusal fails the gate. The SIZES are reported.
* **The decomposition gate** (rung 96's, extended to a table made elsewhere — G6). On the root of an `EPR` run with `δ`
  on, at every CHOKED point: the NOZZLE part of the page's split, `F_true − F̂ − [δ·V0_d·ṁ_tab(e) − V0·ṁ_face]`, is the
  SAME value at every point of the run, and the page shows exactly these two parts. **Bar: spread ≤ 1e−12 × the
  thrust** (rung 96's bar; G6 measured ≤ 1.65e−13 on steady points, the cooled engine worst — a 6× margin).
* **What the default moves — pre-registered (the advisor's item 4).** `table_at` defaults to `design`, so a request that
  never names it changes ONLY if it uses a table AND flies off the design flight. Census after the build: the whole gate
  is run and every changed test, golden and check request is listed. A pre-existing test that moved is pinned to
  `table_at: flight`, so it keeps testing what it was written for; none is re-baselined silently. Already visible in the
  Controls check requests (`sandbox_controls.rs`, plan §§ 27/29): the +40 K sea-level `EPR` run from 700 K (its rows
  translate to ~530 K at the design flight, near G4's edge — it may now be refused in § 1's words), and the proportional
  laws flown off the design flight. Expected: those requests change, every other one is bit for bit.

## 3. PREDICTIONS — frozen here, scored after the build


Cells: the default move, the generator off, no lever, each of the four maps, the other corrections ON unless named.
"Over" means the end's true thrust exceeds its target, "under" the reverse. An end refused for a reading out of
reach scores by the refusal's direction (*below the least the engine reads* ⇒ under; *more than it reads at 2000 K* ⇒
over); a cell whose design-flight table cannot be built scores VACUOUS, stated (G5 found none at these flights); any
other refusal or stop is a MISS, stated. The LP-speed law is scored at the page's default, the commanded rate `τ_N` 0.1
(a calibrated end does not depend on it). (The first draft's H1, the pressure correction off, is construction and is
now a gate in § 2.)

**H1 — temperature correction OFF, the fan-speed control: in cold air it gives too much thrust, in hot air too
little.** Cold (216.65 K) ⇒ over, hot (290 K) ⇒ under, both ends. *Derivation:* uncorrected, the control holds the
PHYSICAL LP speed the table lists. In cold air (`θ` < 1) that speed is a HIGHER corrected speed, `ν_L/√θ`, and the
corrected thrust rises with corrected speed. The state residual of G2 (≤ 0.5 % in corrected speed at these `θ`) is far
smaller than the 7 % that `√θ` moves the speed. 4 maps × 2 ends × 2 flights = **16 cells**.

**H2 — Mach, both corrections on: flying faster than the design Mach the control gives too little thrust; slower,
too much.** Mach 1.4 ⇒ under, Mach 0.5 ⇒ over, both ends, `EPR` and LP speed (`T0`, `p0` the design's). *Derivation:*
by G3 the corrected engine is the same at any Mach; the net thrust at a given corrected state changes with Mach only
through the nozzle's pressure term and the ram drag, at fixed inlet totals. A faster flight lowers `p0/pt2` (gross
thrust up by `A9·pt2·|d(p0/pt2)|`) and raises the flight speed per root inlet temperature (ram drag up by
`ṁ·√(γR·Tt2)·(1+(γ−1)M²/2)^(−3/2)` per unit Mach). **An estimate, not a measurement** (it assumes `Tt9` ≈ 650–900 K
and `EPR` ≈ 1.6–2.5 on the choked nozzle, `A9·pt2 ≈ ṁ·√Tt9/(0.040·EPR)`): at Mach 0.85 the pressure term is
≈ 195–260 m/s × `ṁ` per unit Mach against the ram drag's ≈ 280, and the ram drag's lead widens away from 0.85 in both
directions — so **net thrust at a given corrected state FALLS as Mach rises.** The design-Mach table then over-states the
thrust at Mach 1.4 (the control settles on a lower reading than the target needs ⇒ under) and under-states it at Mach
0.5 (⇒ over). *Risks, named:* (i) at the LOW throttle the two terms nearly balance in this estimate (≈ 260 against
280), so its error could flip the sign — the 1000 K cells are the exposed ones; (ii) these flights also change `θ`
(0.92 and 1.22), whose residual (G2) adds ~0.5–1.5 % with its own sign. 2 readings × 4 maps × 2 ends × 2 flights =
**32 cells**.

**H3 — the acceleration schedule, temperature correction OFF: in hot air the slam is cooler and slower, in cold air
hotter and faster** than the same schedule corrected. Open-loop default move with the schedule on (margin 0.25),
`corr_t` off against on at the same flight: hot ⇒ the peak `Tt4` is LOWER and the thrust reaches 95 % of its final value
LATER; cold ⇒ peak HIGHER and SOONER. A run that never reaches 95 % by the end of the settle counts as the LATER of
the two (both never: the cell is VACUOUS). *Derivation:* uncorrected, the cap is the corrected cap divided by `√θ` at
every state; a leg only cuts, and at frozen speeds `Tt4` and the spool's acceleration rise with fuel (rung 95's G1).
Through the leg's hold the burner sits on the cap line, so a lower cap is a cooler line; the leg releases where the cap
meets the ramp's final fuel, which with a lower cap happens at a higher speed, so `Tt4` at release is lower too.
*Risk:* the peak may lie outside the leg's hold (before engagement, where both runs are identical, or after release),
and the diverged spools could reorder it there. **Vacuity:** a cell counts only if the leg holds the fuel at one point
at least in one of its two runs. 4 maps × 2 flights × 2 quantities = **16 cells**.

**H4 — the proportional HP-speed law on the design table, temperature correction OFF: in hot air it settles at a
HIGHER HP speed, in cold air a LOWER one,** than the same law corrected. Governed, holding HP speed, proportional law,
gain 20; the HP speed at the end of the settle, `corr_t` off against on: hot ⇒ higher, cold ⇒ lower. *Derivation:* the
steady fuel at the flight is `δ√θ·W_d(ν/√θ)`; with `W_d` rising faster than linearly in speed (`W_d ∝ ν^k`, `k` > 1),
the uncorrected `δ·W_d(ν)` exceeds it by `θ^((k−1)/2)` in hot air, so the law pours in too much fuel and the shaft
settles above where the corrected law settles; in cold air the reverse. *Risk:* `k` > 1 is assumed, not measured; a
clamp at either wall pushes the same way. 4 maps × 2 flights = **8 cells**.

**Total: 72 scored cells.**

**Reported, not scored:**
* the temperature residual with BOTH corrections on — each reading's steady miss hot and cold (G2 printed the state,
  not the table read, but the sign of the read is near it);
* the size of every steady miss (§ 2's sign gate, H1, H2), and of the `EPR` control's θ-off run (identical to θ-on by
  construction);
* the fixed-amount variants (generator by power, customer air by amount) at the thin and dense flights;
* the proportional laws' droop on the design-flight table with both corrections on;
* the thrust gap through the move (rung 96's `F_true − F̂`) at the scoring flights.

## 4. What would change the plan

A MISS is reported as a finding (the sandbox rule). A failed root inside the clamp is a reported stop with its reason.
A flight beyond G4's reach is refused in § 1's words — or another refusal, stated. **The crash map runs before the spec
is written**, and is compared back to back with a worktree of the commit before the model (rung 97's review lesson),
request by request. **Cost:** under `design` a table costs 13 steady solves at the design flight (rung 96's thrust table
reused two of the run's own); the clean table with the generator doubles its share. Timed natively against `flight`
before the page is built.
