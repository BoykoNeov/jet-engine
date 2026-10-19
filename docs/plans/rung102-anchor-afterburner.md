# Rung 102 anchor — THE AFTERBURNER: a second burner behind the turbine, on the design, on the engine you fly, and lit during the slam (pre-registered 2026-10-19 — § 3)

**Seam:** CLAUDE.md's open list, *"Afterburner"*; SPEC.md § Future directions B: *"A second burner between turbine and
nozzle, with the nozzle re-sized."* Rung 1's plan deferred it by name.

**The setup (user, 2026-10-19; memory `rung102-decisions`).** Picked after `/clear` from a list of open items. The scope
was set on measured probes (§ 0), each answer the user's:
* **ONE rung** covering (A) the design point, (B) the one-shaft engine flown off design (*Fly it*) and (C) the two-shaft
  engine of the *Controls* view (I had recommended splitting; the user chose one).
* The duct's pressure loss **DERIVED**: a fixed flame-holder loss plus the loss of heating a moving gas (Rayleigh), from
  a burning-zone speed knob.
* An afterburner temperature beyond what the remaining oxygen can give is **REFUSED IN WORDS**, naming the hottest one
  reachable.
* The design half on the **Design view**; the built-engine half on **Fly it AND Controls**.
* On a built engine the nozzle is **a slider** (throat area relative to the dry throat) running from *held* past the
  textbook opening, plus a **match** setting (the textbook law: the area that keeps the core unchanged, at every point).
* The throttle keeps holding **turbine-inlet temperature** (no speed hold).
* On Controls the afterburner **lights at a chosen moment** in the slam (an event in time, not a different engine).
* Rungs 96–101's thrust GUESSES (pressure ratio, fan speed, exhaust sensor) with the afterburner lit are **refused in
  words** — they know nothing of a second burner. Direct thrust / speed / temperature controls work.

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Staging (one rung, four commits, each green and pushed):** this anchor → (A) design + Design view → (B) Fly it →
(C) Controls. **Part C's predictions are NOT frozen here**: § 0's two-shaft probe ran the STEADY matcher (rung 39),
not the Controls march, and its naive reading already missed (§ 0 G6). Part C gets an ADDENDUM to this file — a probe
of the actual march, then its predictions frozen — committed before any Part C code. Parts A and B are frozen now.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung102-probe`, archived in
`docs/plans/scratch-archive/rung102/`) against the shipped crate at `884e60a`. **Disclosed:** every § 0 number is an
afterburner bolted on BESIDE the shipped code by the probe (its own fuel balance, a fixed `π_AB`, the nozzle fed station
7), not the device of § 1. On the built engine the probe modelled the afterburner as a SHRUNKEN throat
`A8·k` (G4) — valid only while the nozzle chokes. **A probe slip, caught by review before this was written:** the probe's
"opened nozzle ⇒ core identical" line re-ran the DRY engine (`run(kd/kd)`), so it measured nothing; that claim is the
textbook's (§ 8.4) plus a continuity argument, and is a § 2 GATE through the new code, not a § 0 given. No rung-102
code exists.

---

## 0. GIVENS — measured before anything was written

Conventions: the project's real-loss set (`π_d` 0.97·ram, `η_c` 0.88, `η_b` 0.99, `π_b` 0.96, `η_t` 0.90, `η_m` 0.99,
`π_n` 0.98), `π_c` 10, `Tt4` 1500 K unless stated; probe `π_AB` 0.95, `η_AB` 0.95; "×" ratios are lit / dry of the same
engine.

**G1 — the design point, four gases, three flights** (`design.txt`, `design_eq.txt`; fully expanded):

| flight | gas | Tt7 2000 K: thrust × | TSFC × | A8 × | `√(Tt7/Tt5)/π_AB` |
|---|---|---|---|---|---|
| sea level M0.2 | CPG / TPG / reacting / equilibrium | 1.324 / 1.298 / 1.307 / 1.308 | 1.483 / 1.499 / 1.561 / 1.569 | 1.389 / 1.369 / 1.380 / 1.381 | 1.361 / 1.326 / 1.327 / 1.327 |
| 250 K 50 kPa M0.85 | ″ | 1.413 / 1.379 / 1.392 / 1.393 | 1.378 / 1.403 / 1.457 / 1.465 | 1.387 / 1.367 / 1.378 / 1.378 | 1.358 / 1.324 / 1.325 / 1.325 |
| 11 km M2 | ″ | 1.790 / 1.677 / 1.713 / 1.714 | 1.342 / 1.361 / 1.414 / 1.420 | 1.459 / 1.422 / 1.437 / 1.438 | 1.426 / 1.373 / 1.375 / 1.375 |

(i) The afterburner pays most flown fast: the thrust gain grows and the fuel penalty shrinks with flight speed. (ii) The
nozzle must open MORE than the square-root rule: CPG by `(1+f7)/(1+f5)` (~2 %), the variable gases by 3–5 % (the hotter
gas's lower `γ` lowers its choked flow per area). (iii) A lit duct at `Tt7 = Tt5` (no heat) costs ~1.5 % thrust at
`π_AB` 0.95 — the duct's loss alone.

**G2 — the equilibrium ceiling and the reacting panic** (`design_eq.txt`). On the equilibrium gas `Tt7` 2300 K is
unreachable at all three flights — the bisection pins at the lean-stoichiometric bracket (`f` 0.067676) with the balance
open — while 2200 K runs (φ 0.92–0.97): dissociation soaks up the heat, so the ceiling is ≈ 2250 K. At 2200 K the
equilibrium afterburner needs ~6 % more afterburner fuel than the reacting gas (no dissociation); at 2000 K ~1 %.
The reacting gas **PANICS** at stoichiometric (`gas.rs:264`, "rich mixture"); CPG/TPG have no oxygen bookkeeping and
would run past it. **On the equilibrium gas `Tt7 = Tt5` is REFUSED**: the frozen station-4 mixture, re-equilibrated at
`Tt5`, recombines and releases heat (residual +4–5 J/mol of air) — so *lit with no extra fuel* is warmer than the gas
that arrived, and "afterburner off" must be its own switch, never `Tt7 = Tt5`. `Tt5 + 1 K` runs.

**G3 — the external anchor: Mattingly, *Elements of Propulsion* (2nd ed., AIAA), § 8.4, the afterburning turbojet's
REFERENCE point** (sea-level static, maximum wet; the book's dual-spool engine, whose design point a single spool with
the same overall ratios reproduces exactly — polytropic efficiencies compose, and both shafts carry `η_m` 0.995):
inputs `π_c` 20 (5 × 4), `e_c` = `e_t` 0.9, `π_dmax` 0.98, `π_b` 0.96, `π_n` 0.98, `η_b` 0.995, `η_m` 0.995,
`Tt4` 3200 °R, `Tt7` 3600 °R (2000 K), `π_AB` 0.94, `η_AB` 0.95, `cp_c` 0.24 / `γ_c` 1.4, `cp_t` = `cp_AB` 0.295 / `γ` 1.3,
`h_PR` 18 400 Btu/lbm; the book prints `τ_tH·τ_tL` = 0.8821 × 0.9033, `π_tH·π_tL` = 0.5466 × 0.6127, `f` 0.0358,
`f_AB` 0.0195, `f_o` 0.0554, `M9` 1.85, `F` 25 000 lbf at `ṁ0` 181.57 lbm/s, `S` 1.4473 (lbm/h)/lbf. The probe
(`anchor.txt`, `build_turbojet` on a dual CPG gas + the probe's afterburner, `M0` 1e−3):

| | book | Mattingly's own eqs (7.33/8.45), P0/P9 = 1 | the model |
|---|---|---|---|
| `f` / `f_AB` / `f_o` | 0.0358 / 0.0195 / 0.0554 | 0.03581 / 0.01955 / 0.05536 | 0.03581 / 0.01955 / 0.05536 |
| `τ_t` / `π_t` | 0.79680 / 0.33490 | 0.79676 / 0.33490 | 0.79676 / 0.33490 |
| `F/ṁ0`, N·s/kg | 1350.26 | 1354.85 | 1354.51 |
| `S`, mg/(N·s) | 40.995 | 40.859 | 40.869 |
| `M9` | 1.85 | 1.8268 | 1.8268 |

The model reproduces the book's EQUATIONS to 0.03 % (the gap is `M0` 1e−3 vs 0). The book's PRINTED thrust sits 0.34 %
below its own equations at the same fuel, from a source not found: `P0/P9` from 1.00 to 1.06 moves the equations'
thrust by < 0.03 % (`M9` 1.85 needs `P0/P9` ≈ 1.04). Disclosed, not chased. Its "wet thrust about 20 % greater than
dry" (Fig. 8.43): the model gives 1.194 with the afterburner off taking no duct loss (the book's 8.45q). The same section
states the control law Part B's *match* implements: *"A8 is controlled during afterburner operation such that the
turbine exit conditions remain constant."* Example 7.5 (the § 7.3 afterburning turbojet) prints only a figure, and
uses `cp_AB ≠ cp_t` — the model gives the afterburner the hot section's properties (exact for § 8.4; disclosed).

**G4 — the built one-shaft engine, nozzle HELD** (`fixed.txt`, `held.txt`; the Fly-it solver at its defaults,
thermally-perfect gas, `Tt4` held 1500 K; the afterburner as an effective throat `A8·k`,
`k = π_AB·(MFP7/MFP5)·√(Tt5/Tt7)·(1+f5)/(1+f7)` solved to a fixed point with the matched `Tt5`):

| flight | Tt7 | eff. throat `k` | spool speed | `π_c`, airflow | stall margin (dry) | thrust × dry | fuel × dry |
|---|---|---|---|---|---|---|---|
| 250 K M0.85 | 1600 | 0.849 | −15 % | −31 % | 0.118 (0.395) | 0.781 | 0.976 |
| | 1800 | 0.800 | −21 % | −41 % | 0.058 | 0.714 | 0.988 |
| | 2000 | 0.758 | −26 % | −49 % | 0.020 | 0.649 | 0.986 |
| 11 km M2 | 1600 | 0.849 | −14 % | −27 % | 0.049 (0.176) | 0.932 | 1.060 |
| | 1800 | 0.800 | −19 % | −36 % | 0.021 | 0.923 | 1.115 |
| | 2000 | 0.758 | −24 % | −43 % | 0.003 | 0.905 | 1.156 |

**A lit afterburner behind a held nozzle gives LESS thrust than the dry engine** — about the same fuel, 10–35 % less
thrust — and walks the compressor to its stall line (the airflow collapses, the operating point climbs its speed line).
With the nozzle opened by the rule at the dry point the lit thrust is ×1.39 (M0.85) / ×1.63 (M2) at fuel ×1.94 / ×2.03.

**G5 — the derived duct loss** (`rayleigh.txt`, perfect gas `γ` 1.3, constant-area heat addition). Thermal choking:
the largest `Tt7/Tt6` a duct entered at Mach `M6` can take is 22.3 / 6.0 / 2.97 / **1.94** at `M6` 0.1 / 0.2 / 0.3 /
0.4 — so at 0.4 a doubling is impossible (the knob must be the slow BURNING-ZONE speed after the diffuser, not the
turbine exit's). The heat-addition loss alone at `Tt7/Tt6` = 2: 0.993 / 0.973 / 0.935 at `M6` 0.1 / 0.2 / 0.3. Mattingly
Problem 7.9 gives 0.97 unlit and 0.94 lit at `Tt8/Tt6` = 2.0 (a real engine's numbers): a flame-holder loss of **0.97**
and **`M6` 0.20** give 0.97 × 0.973 = **0.944**. Those are the defaults.

**G6 — the two-shaft engine, STEADY matcher only** (`twospool.txt`; rung 39's map matcher on the Controls design,
`A8` held as G4): `Tt7` 1600 → LP speed −34 %, HP speed **+13 %**, LP margin 0.24 → 0.15, HP margin UP (0.69 → 0.99),
HP pressure ratio 6 → 7.9; `Tt7` 1800 → LP −47 %, HP +18 %; `Tt7` 2000 **PANICS** at `two_spool.rs:936` (the LP
turbine's choke-match bracket — the LP expansion is driven toward nothing). The naive reading — "the HP turbine sits
between two choked throats, so the HP spool is untouched" — **misses**: the HP turbine's work per kg is unchanged, but
the LP compressor delivers colder air, so the same work buys the HP compressor more pressure ratio and corrected speed.
This is NOT the Controls plant (a time-stepped march, rung 97); Part C's addendum probes that.

---

## 1. THE DEVICE — derived before coding

**Stations** (the industry numbering, Mattingly Fig. 7.8): 5 turbine exit → 6 burning-zone entry → 7 afterburner exit
→ (8 throat) → 9 nozzle exit. 5 → 6 is the diffuser, isentropic (Mattingly's own assumption): `Tt6 = Tt5`, `pt6 = pt5`.

**The setting** (`Afterburner { tt7, eta_ab, pi_fh, m6 }` and a mode):
* `None` — **the default and the reduce**: no afterburner; every existing path, bit for bit, by DISPATCH (no `Option`
  threaded into an unchanged arithmetic path).
* `Unlit` — fitted, not lit: `Tt7 = Tt5`, `f7 = f5`, `pt7 = π_fh·pt5`, no re-freeze (the station-5 mixture flows on).
  The flame holders cost pressure lit or not.
* `Lit` — burning to `Tt7`.
Defaults: `η_AB` 0.95, `π_fh` 0.97, `M6` 0.20 (G5). `Tt7` default 2000 K.

**The fuel** (energy, 6 → 7; fuel's own enthalpy and momentum neglected as the main burner's are):
`(1+f5)·h_t(Tt6, f5) + η_AB·(f7 − f5)·h_PR = (1+f7)·h_t(Tt7, f7)` — the main burner's balance with vitiated air in:
the reactant is the turbine-exit gas, not fresh air. On CPG `h_t` is f-independent and this is Mattingly's (7.29)
exactly (`f_AB = f7 − f5`, both per unit AIR). On TPG/reacting: the fixed point `f7 = g(f7)` seeded at `f5`, a
contraction for the reason the main burner's is. On the EQUILIBRIUM gas: the scale-B absolute balance per mol of air,
reactant = the station-4 FROZEN mixture at `Tt6` plus `n_fuel(f7) − n_fuel(f5)` of fuel, product = the equilibrium mixture
at `(f7, Tt7, pt7)`, loss `(1 − η_AB)·Δn_fuel·LHV` — bisection on `f7 ∈ [f5, f_stoich)`. Its composition depends weakly
on `pt7`, which the duct loss sets after the fuel: the two are iterated to `|Δpt7| ≤ 1e−10·pt7` (pass count reported).
**The main burner's dry-air guard (`components.rs:292`) is not touched** — the afterburner is its own component.

**The stoichiometric cap, on every gas.** `f7 < f_stoich·(1 − 1e−6)`, the main burner's own bracket, applied to CPG and
TPG too (which have no oxygen to run out of, but the fuel does): the same refusal means the same thing on every gas.

**The duct loss** (6 → 7): `π_AB = π_fh · π_R(M6, Tt7/Tt6)`, `π_R` the constant-area heat-addition (Rayleigh) ratio.
On CPG the closed forms (Mattingly / any gas-dynamics text):
`Tt/Tt* = (γ+1)M²(2+(γ−1)M²)/(1+γM²)²`, `pt/pt* = (γ+1)/(1+γM²)·((2+(γ−1)M²)/(γ+1))^(γ/(γ−1))`; `M7` the SUBSONIC root
of `Tt7/Tt* = (Tt7/Tt6)·(Tt6/Tt*)`; no root ⇒ thermal choke. On the variable gases the same three laws solved directly —
mass `ρ6V6·(1+f7)/(1+f5) = ρ7V7`, momentum `p6 + ρ6V6² = p7 + ρ7V7²`, energy `h7 + V7²/2 = h_t(Tt7, f7)` — for the
subsonic static state at 7 (a 1-D root), then `pt7` from the static state by the gas's own `pr`. **`M6 = 0` ⇒ `π_R = 1`
by dispatch** (no 0/0): with `π_fh` 0.94 that is Mattingly's fixed `π_AB` — the anchor's path through the shipped model.

**The nozzle** sees `(Tt7, pt7, f7)` and the station-7 gas. On the equilibrium gas the afterburner FREEZES its own
mixture at `(f7, Tt7, pt7)`: the gas's single burn-condition guard (`gas.rs:1062`) becomes **per fuel/air ratio** — each
`far` its own burn condition, which the station-4 and station-7 freezes satisfy (`f7 > f5` whenever lit; `Unlit` does
not freeze). The guard still fires on a genuinely conflicting reuse. Moves no number (it is an assert); proven by the
oracles.

**Thrust** with the TOTAL fuel: `F/ṁ0 = (1+f7)·V9 − V0 + (1+f7)·R_t(f7)·T9·(1 − p0/p9)/V9`, `S = f7/(F/ṁ0)`, the
efficiencies on `f7` (Mattingly 7.30–7.32). `engine::try_score` reads `f` at station "4"; it is NOT changed — the
afterburner path scores through a twin that takes the total `f` explicitly (its callers' station tables keep their
meaning).

**Refusals, in words** (all `Abort`s, never panics):
* too hot: *"The afterburner cannot reach X K: burning all the oxygen left in the gas gives at most Y K here."* (Y by
  bisection on `Tt7` at `f7 = f_stoich·(1 − 1e−6)`, on the gas in use.)
* too cold: *"The afterburner cannot cool the gas: it arrives at X K."* (`Tt7 ≤ Tt5`; on the equilibrium gas below the
  recombination self-heat of G2, in the same words with that temperature.) On a built engine `Tt5` floats, so a `Tt7`
  valid at design can be refused at another throttle.
* thermal choke: *"Heating the gas to X K would choke the afterburner duct (it enters at Mach M); the hottest it can
  take is Y K."*
* duct entry choked (built engines): *"The gas leaving the turbine is too much for the afterburner duct at this point."*
* Controls: a thrust guess (pressure ratio, fan speed, exhaust sensor) with the afterburner lit (user's decision).

**Part A — the design point.** `build_afterburning_turbojet(gas, π_c, Tt4, p_ambient, losses, ab)` in a NEW module
(`rust/src/afterburner.rs`): the five shipped components, then the afterburner, then the nozzle; `build_turbojet` and
its station table untouched (the CLI golden, the T–s diagram, the visuals pages). The result carries stations
0 2 3 4 5 7 9, and the same engine `Unlit` beside it (the dry readout of the Design view). The T–s diagram gains 7 with
the main burner's composition convention (no false entropy jump at the second freeze).

**Part B — Fly it.** The duct area `A6` is HARDWARE, captured from the design run at the `M6` knob; off design `M6`
floats (`ṁ5·√Tt5/(pt5·A6) = MFP(M6)`, subsonic root). The nozzle relation carries the afterburner INSIDE it — rung 31's
choked (★) with the right-hand side evaluated on station 7 (`pt7 = π_AB(M6, Tt7/Tt5)·pt5`, `MFP(Tt7, f7)`, mass
`(1+f7)/(1+f5)`), rung 33's subsonic branch the same, the running line and the stall reading through them. The slider
`s` = throat area / the DRY throat (`s` = 1 is *held*); *match* = the area that leaves the core where the dry engine
sits at THIS throttle and flight (the dry core solved first by the shipped path, then the afterburner's throat
computed). Range `s` ∈ [1, 2]; a *match* above 2 is reported, not clipped.

**Part C — Controls.** Light-off at a chosen time in the slam; before it `Unlit`, after it `Lit`; the nozzle on the
slider or *match* at every step. The LP choke-match bracket's panic (`two_spool.rs:936`) becomes an `Abort` in words.
Device detail, gates and predictions: the ADDENDUM (staging above).

**Where it lives:** `afterburner.rs` (the component, the duct, the builder, the refusals, the scoring twin; Parts B/C's
nozzle relations); `gas.rs` (the per-`far` guard); `sandbox.rs` + the page (Design view; Fly it); `sandbox_controls.rs`
(Part C). A CLI panel for rung 102 (the anchor table and G1's sweep).

## 2. REDUCES and GATES (construction — no credit)

* **`None` ⇒ the prior code, bit for bit** — every existing gate, oracle, golden, check request; no pin needed.
* **The per-`far` guard moves nothing:** every oracle bit-exact; the guard still fires on a conflicting burn condition
  at one `far` (a gate builds one).
* **Mattingly's equations on CPG:** with `M6` 0 and `π_fh` 0.94, `f_AB`, `τ_AB`, `F/ṁ0`, `S` equal (7.29)/(7.33) to
  **1e−12 relative** at G3's inputs, and at a second point (Mach 2, 11 km).
* **The Rayleigh solve on a CPG gas ≡ the closed form** to 1e−10 (the variable-gas solver run on the perfect gas).
* **Conservation, every run:** mass `ṁ7 = ṁ5·(1+f7)/(1+f5)`; the energy balance closes to 1e−9; the equilibrium
  absolute balance closes (the main burner's 1e−6); the duct's momentum and mass to 1e−9.
* **`Unlit` ⇒ `Tt7 = Tt5`, `f7 = f5`, `pt7 = π_fh·pt5`** exactly; with `π_fh` 1 the station-9 state equals the
  no-afterburner run's bit for bit.
* **Part B, *match* ⇒ the core is the dry engine's** — solved with the afterburner INSIDE the nozzle relation at the
  matched area (a fixed `s`), compared to the dry solve: `π_c`, spool speed, airflow, `Tt5`, stall margin within
  **1e−9 relative**. A second, independent instrument: G4's effective-throat fixed point at the same `s` agrees to 1e−8
  (choked points). Not self-certifying: the gate solves through the new nozzle relation; the instrument through the
  shipped one with a scaled throat.
* **Part B, `s` = *match* computed at the dry point ⇒ the core cannot move** — the textbook's law (G3), here a gate.
* **The page:** default replies byte-equal (the afterburner keys written only off their defaults); the keys
  round-trip; `M6` outside [0, 0.35], `π_fh` outside [0.85, 1], `η_AB` outside [0.5, 1], `s` outside [1, 2] refused by
  the parser.
* **What the defaults move — census by running against a clean worktree of `884e60a`:** NOTHING — every check request
  and a crash map (seed 102, 1 500 requests, afterburner off) identical. A SECOND crash map with the afterburner keys
  randomized: no panic; every difference is new numbers or a § 1 refusal.

## 3. PREDICTIONS — frozen here, scored after the build (Parts A and B)

**Part A cells.** Defaults `η_AB` 0.95, `π_fh` 0.97, `M6` 0.20, fully expanded, the real-loss set. Flights NOT in G1:
**sea level static (M0.05)**, **3 km standard Mach 0.5** (268.65 K, 70.12 kPa), **250 K 50 kPa Mach 1.2**, **11 km
Mach 1.6** (216.65 K, 22.632 kPa). `π_c` **6, 15**; `Tt4` **1400, 1600 K**; `Tt7` **1700, 1900, 2100 K**; gases
thermally-perfect, reacting, equilibrium (CPG reported). A cell refused by the main cycle is VACUOUS (stated); a
refusal in § 1's words where the prediction expects a number is a MISS.

**H1 — Mattingly § 8.4 through the shipped model** (`M6` 0, `π_fh` 0.94, dual CPG, `M0` 1e−3): `f`, `f_AB`, `f_o`
within **±0.00005** of 0.0358 / 0.0195 / 0.0554; `τ_t` and `π_t` within **1e−4** of 0.79680 / 0.33490; `F/ṁ0` within
**0.5 %** of 1350.26 N·s/kg and `S` within **0.5 %** of 40.995 mg/(N·s) (the book's printed residual, 0.34 %, inside);
`M9` within **2 %** of 1.85; wet / dry thrust in **[1.15, 1.25]** ("about 20 %"). *Derivation:* G3. *Risk:* none
beyond the disclosed residual. **1 cell** (HIT only if every claim holds).

**H2 — the afterburner pays more flown fast.** At each `(gas, π_c, Tt4, Tt7)` the wet/dry thrust ratio rises
STRICTLY through the four flights in Mach order, and the TSFC ratio FALLS strictly. *Derivation:* G1 — every column, all
gases. *Risk:* the flights differ in altitude too (ambient temperature moves `Tt5` and so the temperature rise); sea level
static against 3 km Mach 0.5 is the nearest pair. **3 × 2 × 2 × 3 = 36 cells.**

**H3 — the nozzle must open MORE than the square-root rule.** `A8_lit/A8_dry` (the dry throat = the same engine
`Unlit`, which sees `π_fh·pt5`) exceeds the rule `√(Tt7/Tt5)·π_fh/π_AB = √(Tt7/Tt5)/π_R` by **1 % to 8 %**, every
cell — the excess is `(1+f7)/(1+f5)·MFP(Tt5)/MFP(Tt7)`. *Derivation:* G1 — 3.4–3.6 % on the variable gases (2.1 % CPG,
the mass term alone); the mass `(1+f7)/(1+f5)` and the hotter gas's lower choked flow per area both push it up.
*Risk:* `Tt7` 2100 on the equilibrium gas near its ceiling (dissociation lowers `γ` further); `Tt7` 1700 at `Tt4` 1600
(a small rise). **3 × 4 × 2 × 2 × 3 = 144 cells.**

**H4 — dissociation sets a ceiling.** On the equilibrium gas, at each `(flight, π_c, Tt4)`: `Tt7` 2100 RUNS, and the
too-hot refusal's named ceiling lies in **[2150, 2350] K**; the reacting gas's ceiling (the same refusal, no
dissociation) is **at least 50 K higher** at every cell. *Derivation:* G2 — ≈ 2250 K at three flights; the reacting gas
reached 2300 K at M2. *Risk:* 11 km Mach 1.6 (the lowest pressure — the most dissociation, the lowest ceiling) and sea
level `π_c` 15 (the highest). **4 × 2 × 2 = 16 cells.**

**H5 — dissociation costs fuel near the ceiling.** At `Tt7` 2100: `f_AB(equilibrium)/f_AB(reacting) − 1` in
**[1.5 %, 8 %]**. *Derivation:* G2 — ~1.3 % at 2000 K, ~5.9 % at 2200 K. *Risk:* the low-pressure flight (more
dissociation at the same temperature) toward the top of the bar. **16 cells.**

**H6 — a fitted, unlit afterburner costs thrust.** `Unlit` vs `None`, same engine: thrust change in
**[−1.5 %, −0.3 %]**. *Derivation:* G1(iii) — −1.5 % at a 0.95 loss; 0.97 is three fifths of that loss. *Risk:* sea
level static (the smallest jet pressure ratio — a fixed pressure loss bites hardest); the low-thrust flight 11 km Mach
1.6 at `Tt4` 1400. **3 × 4 × 2 × 2 = 48 cells.**

**Part B cells.** The Fly-it solver (thermally-perfect gas, its two map shapes), design = the page's opening design.
Flights NOT in G4: **250 K 50 kPa Mach 0.5**, **3 km Mach 0.3**, **11 km Mach 1.4**, **sea level Mach 0.1**. Throttles
`Tt4` **1300, 1500 K**. `Tt7` **1700, 1900 K**. Defaults otherwise. A throttle where the DRY engine does not run is
VACUOUS (stated); a lit point past the stall line counts as margin gone (HIT for H8); a lit point the solver refuses in
§ 1's words is a MISS for H7–H9 and reported.

**H7 — held nozzle: lit thrust BELOW dry.** `s` = 1: lit thrust < dry thrust, same throttle and flight. *Derivation:*
G4 — ×0.65–0.93, every row. *Risk:* fast flight with a small temperature rise (M2 at 1600 K was ×0.932, the closest);
11 km Mach 1.4 at `Tt7` 1700. **4 × 2 × 2 × 2 = 32 cells.**

**H8 — held nozzle: the stall margin collapses.** `s` = 1: margin falls by **more than 60 %** of its dry value at `Tt7`
1900 and **more than 40 %** at 1700. *Derivation:* G4 — −70 % (1600 K) to −98 % (2000 K). *Risk:* `Tt4` 1300 (a lower
`Tt5`, so a LARGER rise at the same `Tt7` — conservative) against the second map shape (its stall line read differently).
**32 cells.**

**H9 — opening the nozzle buys both back, and crosses dry on the way.** Over 11 equal steps of `s` from 1 to *match*:
the margin and the lit thrust both rise STRICTLY, and the lit thrust crosses the dry thrust strictly inside
`(1, s_match)`. *Derivation:* G4's two ends (below dry at held, ×1.39–1.63 at the rule). *Risk:* a non-monotone margin
where the operating point turns on a speed line (rung 36's map curvature). **4 × 2 × 2 = 16 cells** (`Tt7` 1900, both
maps).

**Total: 341 scored cells** (1 + 36 + 144 + 16 + 16 + 48 + 32 + 32 + 16).

**Reported, not scored:** every size beside the bars; CPG on every Part A claim; the derived `π_AB` at every cell (G5's
soft check: near 0.94 at a doubling); the equilibrium `pt7` iteration's pass counts; the *match* area at every Part B
cell beside the square-root rule; the held-nozzle thrust and fuel ratios; cost (§ 4); the refusals counted by class.

**§ 3b — Part B clarifications, 2026-10-19, written after Part A shipped (7de9419) and BEFORE any Part B code, on dry
runs only** (`dry_b.txt` in the scratch archive). No bar moves; three readings the text above left open are fixed:

* **"Its two map shapes"** — the solver has THREE (`flow`, `pressure`, `tilted`). The two scored are **`flow`** (the
  page's opening shape, G4's) and **`pressure`** (the next in the page's order; "its stall line read differently" —
  curvature in pressure). `tilted` is run and REPORTED beside, not scored. Cell counts unchanged.
* **The dry pre-flight**: all 16 scored (flight, `Tt4`) points run DRY on all three shapes, every one CHOKED, every one
  with a positive margin (`flow` 0.222–0.507, `pressure` 0.399–0.883, `tilted` 0.281–0.658). So no cell is VACUOUS, and
  no clause is needed for a dry point unchoked or already past the stall line.
* **"Dry" and "held" (`s` = 1)** — the fitted engine's dry throat is sized for its UNLIT duct, `A8_dry = A8_core/π_fh`,
  so unlit with `s` = 1 sits where the engine without an afterburner sits wherever the nozzle is choked (all 16 dry
  points: equal to rounding, a gate — `π_fh·(A8/π_fh)` is not bit-exact). H7–H9's "dry" is that unlit engine. G4's probe
  held the CORE throat (`A8_core`, no `1/π_fh`) against the engine without an afterburner, so its held throat was 3 %
  smaller than Part B's: Part B's held point is slightly LESS severe than G4's numbers (a direction, stated before
  scoring). *Match* sizes the area from the lit exit at the unlit core: `ṁ7·√Tt7/(π_n·pt7·MFP7)` when choked,
  `ṁ7/(ρ9·V9)` at `p9 = p0` when not (not reached on the scored grid, per the dry pre-flight).

## 4. What would change the plan

* **Cost on the product's path.** Fly it is live at 10–45 ms a point (plan § 10.7). If a lit point (held or *match*)
  costs more than **3×** the dry point, best of 9, the afterburner's nozzle relation is re-planned (e.g. a cached duct
  table) before Part B ships. The Design view on the equilibrium gas: a lit design more than **5×** the dry one ⇒ the
  same.
* **The equilibrium `pt7` iteration** needing more than **10** passes anywhere on Part A's grid ⇒ the coupling is
  re-derived (stated), not a looser bar.
* **The Rayleigh solve on the variable gases** failing to find a subsonic root where the CPG closed form has one ⇒ the
  solver, not the refusal, is wrong; fixed before scoring.
* **A panic anywhere** in the crash maps ⇒ fixed as a refusal before the commit (rung 92's rule).
* **Part C's addendum** may change Part C's device (light-off as a step in `Tt7` vs a ramp; the slider's meaning in
  time) — its own anchor, its own predictions; it cannot move Parts A/B's frozen bars.
