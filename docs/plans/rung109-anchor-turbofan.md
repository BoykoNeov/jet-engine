# Rung 109 anchor — THE TURBOFAN: a fan whose air mostly goes around the core, on the design point (pre-registered 2026-11-19 — § 3)

**Seam:** roadmap B1 (`docs/plans/roadmap.md`, Part 3); SPEC.md § Future directions B: *"Turbofan. Add a fan, a bypass
stream, and a splitter; separate or mixed exhaust; sweep bypass ratio."* The guide's experiment 2 ends on it: a hot,
fast jet wastes energy, so move more air more slowly.

**The setup (user, 2026-11-19; memory `rung109-decisions`).** Chosen after the probe (§ 0), each answer the user's:
* **Design point only**, on the **Design view**. A bypass-ratio knob; **0 is today's turbojet, bit for bit**. Flying
  the fixed turbofan off design (a fan map, two nozzles sharing the flow) is a later rung.
* The fan's nozzle **follows the core nozzle's mode**: fully expanded, a set exit pressure, or the fixed convergent
  nozzle (so a cold-air sonic throat is new code).
* **One lumped turbine** drives the compressor and the fan — the textbook's own method (§ 0 G1).
* Three page extras: the **best fan pressure ratio** at the current bypass ratio (a search), a **fuel-versus-bypass
  chart**, and the **two-jet efficiency rows** (roadmap A6's equivalent-jet split, extended to two jets).
* My scope call, disclosed: **the afterburner with a fan is refused in words.** An afterburning turbofan mixes the two
  streams first; that is a different engine.

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Staging (one rung, three commits, each green and pushed):** this anchor → the model (`rust/src/turbofan.rs`, tests,
CLI panel) → the page (Design view, chart, best-fan readout).

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung109-turbofan\probe`, against the shipped crate at
`7f34d05`; not archived — its one source file is reproduced in the build's tests). **Disclosed:** every § 0 number is a
fan stream bolted on BESIDE the shipped code by the probe (its own cold nozzle, its own scoring). No rung-109 code exists.

---

## 0. GIVENS — measured before anything was written

**G1 — the external anchor: Mattingly, *Elements of Gas Turbine Propulsion* (McGraw-Hill, 1996), Example 7-6,
pp. 398–400** — the separate-exhaust turbofan with losses. **Not** the AIAA *Elements of Propulsion* the rung-2/4/102
anchors cite; the two books' example numbers differ. Inputs: `M0` 0.8, `T0` 390 °R, `γc` 1.4 / `cpc` 0.240 Btu/(lbm·°R),
`γt` 1.33 / `cpt` 0.276, `hPR` 18 400 Btu/lbm, `π_dmax` 0.99, `π_b` 0.96, `π_n` 0.99, `π_fn` 0.99, `e_c` 0.90,
`e_f` 0.89, `e_t` 0.89, `η_b` 0.99, `η_m` 0.99, `P0/P9` = `P0/P19` = 0.9, `Tt4` 3000 °R, `π_c` 36, `π_f` 1.7, `α` 8.
The book's method: the core air passes the fan root inside `π_c` (2 → 3 is the WHOLE core compression); ONE turbine pays
`τc − 1 + α(τf − 1)` (its Eq. 7-52o); thrust per TOTAL air `ṁ0 = (1+α)ṁc`; `S = f/[(1+α)F/ṁ0]`.

Three columns: the book's printed number; the book's own equations re-run from its inputs (`ex76.mjs`); the probe —
the shipped components (dual CPG gas) plus a fan stream (`fan.txt`, part A):

| | book | its equations | the model (probe) |
|---|---|---|---|
| `τc` / `ηc` | 3.119 / 84.2 % | 3.11935 / 84.17 % | 3.11935 |
| `τf` / `ηf` | 1.1857 / 88.2 % | 1.18572 / 88.15 % | 1.18572 |
| `f` | 0.02868 | 0.028678 | 0.028678 |
| `τt` / `πt` / `ηt` | 0.54866 / 0.06599 / 92.0 % | 0.548608 / 0.065958 / 92.00 % | 0.548608 / 0.065958 |
| `Pt9/P9` / `M9` / `T9/T0` / `V9/a0` | 3.066 / 1.394 / 3.196 / 2.427 | 3.06503 / 1.39342 / 3.19613 / 2.42644 | — / 1.39342 / 3.19613 / 2.42644 |
| `Pt19/P19` / `M19` / `T19/T0` / `V19/a0` | 2.286 / 1.154 / 1.0561 / 1.186 | 2.28583 / 1.15421 / 1.05610 / 1.18614 | — / 1.15421 / 1.05610 / 1.18614 |
| `F/ṁ0`, lbf/(lbm/s) | 18.02 | 18.0237 | 18.0238 |
| `S`, (lbm/h)/lbf | 0.6366 | 0.63646 | 0.63645 |
| `FR` (core ÷ fan thrust, each per its own air) | 3.988 | 3.98607 | 3.98607 |
| `η_T` | 40.98 % | 40.98 % | 40.98 % |
| `η_O` / `η_P` | 30.60 % / 74.65 % | see below | 30.60 % / 74.66 % (thrust-based) |

Every row: the model within **0.05 %** of the book — the book's rounding of its own intermediates (it carries 4–5
digits). **The book's one inconsistency:** its printed `η_P` 74.65 % is NOT its printed formula (7-50/7-52ac, which it
states only for `P9 = P19 = P0`): that formula, at its own numbers, gives **66.28 %**. The printed 30.60 % / 74.65 % are
the THRUST-based overall efficiency `F·V0/(ṁf·hPR)` = 30.60 %, and `η_P = η_O/η_T`. The pressure push (`P9 ≠ P0`)
is in the thrust and not in the kinetic energy — exactly the trap roadmap A6 fixed for one jet. The rung follows the
thrust-based reading (it is the one that counts what the engine delivers).

**G2 — bypass ratio 0 IS the turbojet** (`fan.txt`, part B; page defaults: 250 K, 50 kPa, Mach 0.85, `π_c` 10, `Tt4`
1500 K, the real-loss set): specific thrust and TSFC **bit-equal** to `build_turbojet(…).run(…)` on all five gases.
Thermal efficiency bit-equal on four; on the equilibrium gas it differs in the last two digits — traced: the probe
multiplied `(1+f)·V9·V9` where `score` writes `(1+f)·(V9·V9)`. Arithmetic order, not physics; the build reduces by
DISPATCH, so it calls `score` itself.

**G3 — what a fan does on the page's default engine** (`fan.txt`, part C; fan efficiency 0.88, fan duct 0.98, fully
expanded; best fan pressure ratio = the lowest TSFC on a 0.01 grid from 1.05 to 4.0; equilibrium gas):

| | α 0 | 1 | 2 | 4 | 6 | 8 | 10 |
|---|---|---|---|---|---|---|---|
| cruise M0.85, `π_c` 10: TSFC g/(kN·s) | 34.04 | 28.81 | 26.27 | 24.13 | 23.19 | 22.76 | 22.59 |
| best `π_f` | — | ≥ 4.0 (grid edge) | 3.46 | 2.23 | 1.81 | 1.60 | 1.48 |
| thrust per CORE air, N·s/kg | 798 | 943 | 1035 | 1127 | 1172 | 1194 | 1203 |
| standing sea level, `π_c` 10: TSFC | 28.72 | 21.57 | 18.14 | 14.57 | 12.63 | 11.37 | 10.48 |
| thrust per core air | 943 | 1256 | 1493 | 1859 | 2145 | 2382 | 2584 |

(i) Fuel per thrust falls with bypass ratio, steeply at first; at cruise it is nearly flat past α 8 (on the perfect gas
at `π_c` 30 it turns back up: 18.222 at α 8, 18.320 at α 10 — an optimum bypass ratio exists). (ii) The best fan
pressure ratio falls as bypass ratio rises. (iii) Standing still the fan pays most: thrust from the same core rises
2.5× by α 8. (iv) **The fan has walls:** at a fixed `π_f` 1.6, α 8–10 is refused at `π_c` 10 — the turbine cannot pay
for that much fan work and still leave the core jet enough pressure to get out. At small `π_f` standing still the fan
cannot even cover the inlet and duct losses (the fan nozzle's total pressure below ambient).

---

## 1. THE DEVICE — derived before coding

**Stations** (Mattingly Fig. 7-11): 0 → 2 (fan face) → **13** (fan exit, bypass side) → **19** (fan-nozzle exit), and
the core 2 → 3 → 4 → 5 → 9 as today. The duct and fan nozzle are one total-pressure ratio `π_fn` (13 → 19 totals).

**The setting** `Fan { alpha, pi_f, eta_f, e_f, pi_fn }`:
* `alpha = 0` — **the default and the reduce**: no fan stream; the existing path by DISPATCH, bit for bit.
* the fan efficiency follows the compressor's switch: isentropic `η_f` (default 0.88) or polytropic `e_f` (0.90);
  `π_f` default 1.6; `π_fn` default 0.98 (the core nozzle's).
* The page's airflow knob stays the **CORE** airflow (the turbojet's air). Adding a fan keeps the core and adds air
  around it: the bypass flow is `α·ṁ_core`. Both thrust per total air (the book's) and per core air are reported.

**Station 13 — the fan.** The shipped `Compressor` applied to station 2 at `π_f` (cold air; pure). **Physical
justification:** the bypass air is fresh air that meets only the fan, so the cold section's laws are the whole story.

**The shaft (one lumped turbine).** Per unit core air,
`Δh_turbine = [h_c(Tt3) − h_c(Tt2) + α·(h_c(Tt13) − h_c(Tt2))] / (η_m·(1+f))`
— the turbojet's balance plus the fan's work on the bypass air. On the perfect gas this is Mattingly's 7-52o exactly.
The shaft-closure check (every run) gains the same term.

**Station 19 — the cold nozzle.** The shipped nozzle reads the HOT section at the burned fuel/air ratio; the bypass air
needs the COLD one (on Mattingly's dual gas the difference is `γ` 1.4 against 1.33). A cold twin, the same laws:
expanded/exit-pressure — `T19 = T_from_pr_c(pr_c(Tt19)·p19/pt19)`, `V19 = √(2(h_c(Tt19) − h_c(T19)))` (perfect gas:
the closed form, as the hot nozzle); convergent — choked when `pt19/p0` exceeds the sonic ratio, the throat from
`h_c(Tt19) − h_c(T*) = ½γc(T*)R_c T*` (perfect gas: closed form), `p19 = p*`; else `p19 = p0`. Conservation, every run:
`h_c(T19) + V19²/2 = h_c(Tt19)` (the hot nozzle's tolerances).

**Thrust** per total air: `F/ṁ0 = [F_core + α·F_fan]/(1+α)` with
`F_core = (1+f)V9 − V0 + (1+f)R_t(f)T9(1 − p0/p9)/V9` (today's, per core air) and
`F_fan = V19 − V0 + R_c·T19·(1 − p0/p19)/V19`; `TSFC = f/[(1+α)·F/ṁ0]`.
**Efficiencies:** thermal = the kinetic energy added to BOTH jets per fuel energy,
`[(1+f)V9² + αV19² − (1+α)V0²]/(2f·hPR)`; overall = `(1+α)(F/ṁ0)·V0/(f·hPR)` (thrust-based, G1);
propulsive = overall ÷ thermal. Standing still: propulsive and overall exactly 0 (the standing-engine rule).
**Two-jet equivalent split (page):** each jet's equivalent speed from its own thrust — `V_eq9 = (F_core + V0)/(1+f)`,
`V_eq19 = F_fan + V0` — then the same two formulas on them; at α = 0 this is `sandbox::equivalent_jet` itself.

**The best fan pressure ratio** (page): at the current bypass ratio and everything else held, the `π_f` with the lowest
TSFC (standing still: the highest thrust — the same thing, since the fuel is fixed by the core). A bracketing search
over the range that runs; reported with the run's TSFC there, or *"above X"* when the lowest TSFC is at the top of the
runnable range.

**The chart** (page): TSFC and thrust per core air against bypass ratio 0–12 at the current settings, two lines each:
the current `π_f`, and the best `π_f` at each bypass ratio. Points that do not run are left out, and the page says
where the line stops.

**Refusals, in words** (all `Abort`s, never panics):
* *"The fan's pressure rise (X) does not cover the inlet and duct losses here: the bypass air would arrive at the
  nozzle below the outside pressure."*
* *"The turbine cannot pay for this fan: after driving the compressor and the fan, the core jet has no pressure left to
  leave the nozzle. Lower the bypass ratio or the fan pressure ratio."* (the core nozzle's `pt9 < p9`, or `Tt5` not
  above the ambient temperature.)
* *"The afterburner is not modelled on a turbofan (it would need the two streams mixed first)."*

**Where it lives:** `rust/src/turbofan.rs` (the fan stream, the cold nozzle, the builder and runner, scoring, the
search, the refusals); `sandbox.rs` + the page (Design view); a CLI panel for rung 109 (the anchor table, G3's sweep).
`engine.rs`, `components.rs`, `build_turbojet` and every station table: untouched.

## 2. REDUCES and GATES (construction — no credit)

* **α = 0 ⇒ the prior code, bit for bit** — every existing gate, oracle, golden and check request; plus a test on all
  five gases × three nozzle modes × standing/flying that the turbofan entry at α = 0 returns `build_turbojet(…).run(…)`'s
  numbers exactly.
* **Continuity:** α = 1e−9 within 1e−6 relative of α = 0 (specific thrust, TSFC, thermal efficiency).
* **G1 through the shipped model:** every printed row of § 0's table within **0.1 %** of the book (the book's 4–5
  digits); `η_O` thrust-based within 0.1 % of 30.60 %; the probe's values to **1e−12 relative** (the build is the same
  arithmetic in the code).
* **Mattingly's equations, perfect gas, fully expanded** (`P0/P9` = 1): `F/ṁ0`, `S`, `f`, `τt` equal 7-52 to 1e−12
  relative at G1's inputs and at a second point (standing sea level, α 5, `π_f` 1.5, `π_c` 25).
* **Cold nozzle ≡ hot nozzle where they must agree:** on the single perfect gas (cold = hot) the cold twin equals the
  shipped nozzle bit for bit (both modes); the convergent cold throat on the perfect gas equals the closed form to 1e−12.
* **Conservation, every run:** shaft closure with the fan term (1e−6, the existing bar); fan-nozzle energy (the hot
  nozzle's bars); bypass mass `= α·ṁ_core` exactly.
* **Two-jet split:** fully expanded ⇒ equal to the speed-only split to 1e−9 on every gas; α = 0 ⇒ `equivalent_jet` bit
  for bit.
* **The page:** default replies byte-equal (fan keys written only off their defaults); keys round-trip; α outside
  [0, 15], `π_f` outside [1.0, 4.0], fan efficiency outside [0.5, 1], `π_fn` outside [0.85, 1] refused by the parser.

## 3. PREDICTIONS — frozen here, scored after the build

**Cells.** Flights NOT in G3: **standing sea level** (288.15 K, 101 325 Pa, Mach 0), **3 km Mach 0.5** (268.65 K,
70 120 Pa), **11 km Mach 0.8** (216.65 K, 22 632 Pa). `π_c` **15, 40**; `Tt4` **1400, 1700 K**; gases thermally perfect,
reacting, equilibrium (perfect gas reported, not scored). The real-loss set; fan efficiency 0.88 isentropic, `π_fn`
0.98; fully expanded unless stated. Bypass ratios **0, 1, 2, 4, 8, 12**. "Best `π_f`" = the shipped search. **36 cells**
per (flight × `π_c` × `Tt4` × gas). A cell the main cycle refuses is VACUOUS (stated); a § 1 refusal where a number is
expected is a MISS.

**P1 — fuel per thrust falls with bypass ratio, at first steeply.** At the best `π_f`, TSFC falls strictly through
α 0 → 1 → 2 → 4. *Derivation:* G3 (i) at every gas and both flights. **36 cells** (HIT only if all three steps fall).

**P2 — then the return flattens.** The TSFC fall from α 8 to 12 is **less than a third** of the fall from α 2 to 4 (a
rise counts as a HIT: it is less). *Derivation:* G3 — cruise 2→4 fell 2.14, 8→10 fell 0.17. *Risk:* standing still
the fall keeps going longer (G3: 1.25 for 8→10 against 3.57 for 2→4, already over a third per two units). **36 cells.**

**P3 — the best fan pressure ratio falls as bypass ratio rises.** Strictly through α 2 → 4 → 8 → 12 (α 1 left out: its
best can sit above the 4.0 knob limit, G3). *Derivation:* G3 (ii); Mattingly § 5 (ideal: the best `π_f` makes the two
jets equally fast, and more bypass air shares the same fan work). **36 cells.**

**P4 — at the best fan, the fan jet is slower than the core jet, by about the product of two efficiencies.** At the
best `π_f`, `V19/V9` lies in **[0.70, 0.88]**, for α 2, 4, 8, 12. *Derivation:* moving a little work from the core jet
to the fan: the turbine turns jet energy into shaft work at about `η_t`, the fan turns shaft work back into jet energy
at about `η_f`; thrust is best when the last joule gives the same thrust either way, which puts the fan jet at about
`η_f·η_t` = 0.88 × 0.90 ≈ 0.79 of the core jet's speed (Kerrebrock's result; the ideal engine's "equal jets" is the case
both efficiencies are 1). *Risk:* the fuel's mass (`1+f`) and the duct and nozzle losses tilt it; standing still the
argument still holds (it never used `V0`). **36 × 4 = 144 cells.**

**P5 — a convergent fan nozzle costs little.** At 11 km Mach 0.8, α 8, the fully expanded engine's best `π_f`:
switching both nozzles to convergent raises TSFC by **less than the same switch on the turbojet** (α 0) at the same
`π_c`, `Tt4` and gas, and by **less than 3 %**. *Derivation:* the fan's pressure ratio is low, so its under-expansion
wastes little, and the fan carries most of the thrust. *Risk:* the core's own penalty is unchanged and still in the sum.
**12 cells** (11 km only, HIT only if both clauses hold).

**P6 — standing still, a fan multiplies the core's thrust.** Standing sea level, α 8 at its best `π_f`: thrust from the
same core between **2.0× and 3.0×** the turbojet's. *Derivation:* G3 (iii) 2.53× at `π_c` 10, `Tt4` 1500. *Risk:* the
`Tt4` and `π_c` of the cells differ. **12 cells.**

**Scoring:** 36 + 36 + 36 + 144 + 12 + 12 = **276 cells**, printed by the rung's test with every number.

## 4. What would change the plan

* A G1 row outside 0.1 % after the build ⇒ stop; the build is not the probe's arithmetic.
* α = 0 not bit-equal anywhere ⇒ stop; the dispatch is wrong.
* P4 missing wholesale ⇒ the "two efficiencies" argument is wrong for this model; write why before touching the
  prediction.
* The best-`π_f` search slower than **10×** one design run on the page ⇒ make it coarser, say so on the page.
