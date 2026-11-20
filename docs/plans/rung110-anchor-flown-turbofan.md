# Rung 110 anchor — THE TURBOFAN FLOWN: the fixed turbofan off design, on the two-shaft engine (pre-registered 2026-11-20 — § 3)

**Seam:** rung 109's first concession (`docs/rung109-spec.md` § Concessions: *"Design point only. The fixed turbofan
flown off design (a fan map, two nozzles sharing the flow, a fan shaft of its own) is the next rung's"*) and its second
(*"One lumped turbine … arrives with the off-design engine"*); roadmap B1's open line.

**The setup (user, 2026-11-20; memory `rung110-decisions`).** Each answer the user's:
* **The textbook method first:** component efficiencies held at their design values, both high-pressure throats choked,
  no compressor maps. Shaft speeds still come out (Mattingly's 8-52ag/ah); stall margin, the map, the throttle slam and
  the Fly-it afterburner wait for the charts (rung 111).
* **ONE solver for every design.** The user: *"my idea is not to have the need to develop extra modules that are
  practically the same. Also - about a sandbox 'design your own engine' i think a single solver would be better."* So
  the flown turbofan is **the existing two-shaft engine (rungs 38/103/104) given a bypass stream** — not a parallel
  solver. At bypass 0 it IS that engine; the fan knob always sets the first compressor's ratio (π_f = 1 at bypass 0 is
  one shaft).
* **Fly it switches to this engine for every design "when charts arrive"** (rung 111), not in this rung: nothing on the
  page goes dark, the experiments guide is rewritten once. **Rung 110 = the model, its gates, one CLI panel.**
* **My calls, told to the user and not objected to:** the split rule (§ 1 D1) and the LP turbine's law (§ 1 D6).

Shipped as KNOBS on the model: every prediction below is scored HIT/MISS, and a miss is a finding.

**Staging (one rung, three commits, each green and pushed):** this anchor → the model (`rust/src/flown_turbofan.rs`,
`rust/tests/rung110.rs`, the CLI panel) → the spec, CLAUDE.md row, notes and charts-page splice.

**Status:** § 0 measured on two throwaway scripts (`W:\temp\claude\rung110-flown-turbofan\probe\`, Node; not archived:
`ex88.mjs` is re-implemented in the build's Rust gate, `probe2.mjs`'s numbers are quoted below). **Disclosed:** every
§ 0 number is a dual PERFECT gas with the pin law — a textbook engine computed beside the shipped code, not by it. No
rung-110 code exists.

---

## 0. GIVENS — measured before anything was written

**G1 — the external anchor: Mattingly, *Elements of Gas Turbine Propulsion* (McGraw-Hill, 1996), § 8-5 "Turbofan
Engine — Separate Exhausts and Convergent Nozzles", pp. 518–541; Example 8-8, pp. 526–529.** The engine is **Example
7-6's — rung 109's anchor** — referenced at 40 kft, Mach 0.8, and flown to sea-level static at the same `Tt4` = 3000 °R.
Source: the user's own scan (not committed; `.git/info/exclude`). The book's method (Eqs. 8-36 … 8-52): both
high-pressure throats (4, 4.5) choked ⇒ `τtH, πtH` constant; the LP turbine pinned while the core nozzle is choked,
else from `πtL = πtLR·√(τtL/τtLR)·MFP(M9R)/MFP(M9)`; the fan's exit state for the core = the bypass's (13 = 2.5); the
bypass ratio from the two nozzles' flow, `α = αR·(πcHR/πcH)·√[(τλ/(τrτf))/(…)R]·MFP(M19)/MFP(M19R)` (8-46); HP shaft
`τcH = 1 + (τλ/τλR)(τrR τfR)/(τr τf)·(τcHR − 1)` (8-48); LP shaft `τf = 1 + (1−τtL)/(1−τtL)R·(τλ/τr)/(τλ/τr)R·(1+αR)/(1+α)·(τfR − 1)`
(8-50); functional iteration on `τtL` to 1e−4 (Table 8-7).

Printed reference: `T0` 390 °R, `γc` 1.4 / `cpc` 0.24, `γt` 1.33 / `cpt` 0.276, `hPR` 18 400 Btu/lbm, `M0` 0.8, `Tt4`
3000 °R, `πc` 36, `πf` 1.7, `α` 8, `ηf` 0.8815, `ηcH` 0.8512, `τtH` 0.7580, `πtH` 0.2851, `τtL` 0.7262, `πtL` 0.2349,
`ηtL` 0.9068, `ηb` 0.99, `πdmax` 0.99, `πb` 0.96, `πn` 0.99, `πfn` 0.99, `ηmH` 0.9915, `ηmL` 0.997, `P0` 2.730 psia,
`ṁ0` 600 lbm/s; both nozzles choked at the reference. Off design: `T0` 518.7 °R, `P0` 14.696 psia, `M0` 0.

The book's equations re-run from those inputs (`ex88.mjs`):

| | book | its equations, to 1e−4 (book's stop) | to 1e−13 |
|---|---|---|---|
| Table 8-7 row 1 (`τcH πcH πf M19 M9 α τf τtL πtL`) | 2.3875 15.322 1.7000 0.8861 0.8904 10.07 1.1281 0.7262 0.2375 | 2.3873 15.317 1.7000 0.8861 0.8901 10.069 1.1281 0.7262 0.2375 | — |
| `α` | 9.103 | 9.1023 | 9.1050 |
| `πf` | 1.4973 | 1.4968 | 1.4972 |
| `πcH` | 16.555 | 16.554 | 16.551 |
| `ṁ0`, lbm/s | 1638 | 1636.9 | — |
| `f` | 0.02769 | 0.02769 | 0.02769 |
| `F/ṁ0`, lbf/(lbm/s) | 29.04 | 29.026 | 29.035 |
| `S`, (lbm/h)/lbf | 0.3398 | 0.33993 | 0.33976 |
| `F`, lbf | 47 570 | 47 514 | — |
| `N_fan/N_R`, `N_HP/N_R` | 0.938, 1.00 | 0.9379, 1.0001 | — |

Every row within **0.12 %** — the book's rounding (it stops at 1e−4 and carries 4–5 digits; `F` = 1639 × 29.04). The
book's printed intermediates agree as closely: `T9/T0` 2.848, `V9/a0` 1.4165, `T19/T0` 1.0205, `V19/a0` 0.7688
(≤ 0.05 %).

**G2 — the book holds `(1+f)` at its reference value in both shaft balances and in `ṁc`** (Eqs. 8-48/8-50 "equating
the constant to reference values"). Carried exactly, Example 8-8 moves by **+0.031 % in `α`, −0.055 % in `πf`,
−0.142 % in `πcH`** (`f` 0.027703 against 0.027690) — the size of the book's own rounding. So the model (which carries
`f` exactly, as every matcher since rung 31 does) is held to the book through a re-derivation of its equations WITH
exact `(1+f)` (§ 2 gate 5), not through a model switch used only by a gate.

**G3 — the book's engine is NOT rung 109's at station 3.** The book sends the core air through the fan root at the FAN's
efficiency: `τf·τcH` = 1.18571 × 2.63569 = 3.1252 against Example 7-6's single compressor `τc` = 3.1194 (+0.19 %). So the
Example 8-8 gate builds its reference engine from the BOOK's printed component values, and the "flown at design
reproduces rung 109's design run" reduce is a separate gate on the model's own convention (D1).

**G4 — the split rule's wall, found before choosing.** Deriving the HP compressor's efficiency so that station 3
matches, with the fan's efficiency on the core air, gives an impossible 1.043 on the page's lossless set (`ηc` 1, fan
0.88, `πc` 10, `πf` 1.6). D1's rule cannot produce one above 1.

**G5 — what a fan does flown off design** (`probe2.mjs`; page loss set — `πd` 0.97, `ηc` 0.88, `ηb` 0.99, `πb` 0.96,
`ηt` 0.90, `ηm` 0.99, `πn` 0.98, fan 0.88, `πfn` 0.98; design 250 K / 50 kPa / Mach 0.85, `πc` 10, `Tt4` 1500 K; D1's
split; convergent nozzles; dual perfect gas, pin law):

| design (α, πf) | cruise: α at Tt4 1500 → 900 | standing SL: α 1500 → 900 | cruise: core's own thrust ÷ total at 900 | N_HP at 1100 (all designs, all flights) |
|---|---|---|---|---|
| (0, 1.6) — two-shaft turbojet | 0 | 0 | 1 | 0.8524 (and N_LP the same to 4 digits) |
| (2, 3.46) | 2.00 → 2.21 rising | 2.00 → 2.10 (max at 1100) → 2.03 | +0.018 | 0.8525 |
| (6, 1.81) | 6.00 → 7.89 rising | 5.83 → 5.98 (max at 1300) → 5.27 | **−0.213** (crosses ~1050 K) | 0.8525 |
| (10, 1.48) | 10.0 → 14.0 rising | 9.02 → 9.09 (max at 1400) → 8.29 (at 1000) | **−0.366** (crosses ~1150 K) | 0.8525 |
| (6, 1.6) — the knob's default | 6.00 → 8.25 rising | 5.79 → 5.97 (max) → 5.30 | +0.034 | 0.8525 |

(i) In cruise, with the fan nozzle choked, the bypass ratio **rises** as the throttle comes back: the core's flow
(`∝ pt4/√Tt4`) falls faster than the fan's. (ii) Standing still the fan nozzle is unchoked and the bypass ratio rises
and then **falls**. (iii) At high bypass the core's own net thrust turns **negative** at part throttle in cruise — its
jet slower than the flight — and the fan makes all the thrust. (iv) **The HP shaft's speed depends on `Tt4` alone**:
0.8524–0.8525 at 1100 K for every design and flight — Mattingly's 8-52ah with both HP throats choked gives
`N_HP ∝ √[ηcH·ηm·(1+f)·cpt·Tt4·(1−τtH)]`, no flight and no bypass in it. (v) At bypass 0 the LP shaft slows exactly
with the HP; with a fan it slows faster (at 1100 K in cruise: fan 0.73, HP 0.85 at (6, 1.81)). (vi) **The triangle
closes:** at bypass 0 the first compressor's ratio is blind to the HP compressor's efficiency (`dπf/dηcH` 1e−13, the
fixed point's noise — rung 38's triangle); with a fan it is not (`dπf/dηcH` +0.3 … +3.3): the bypass ratio reads the
core's flow, which reads `πcH`, so the LP shaft hears the HP one, map or no map. (vii) Fuel per thrust standing still
has an interior minimum near 1300–1400 K for every fan design; in cruise it rises monotonically as the throttle comes
back. (viii) **At high bypass the core nozzle is unchoked AT THE DESIGN POINT** ((6, 1.81) and (10, 1.48)) — see G6.

**G6 — every shipped matcher sizes the core nozzle as a CHOKED throat** (`matcher.rs:376`, `two_spool.rs:959`:
`A8 = ṁ4·√Tt9/(pt9·MFP*)`; rung 31's doc: *"The design REFERENCE is the choked-CONVERGENT design point"*). A turbofan's
core nozzle is often unchoked at design (G5 viii), and its fan nozzle is unchoked at any standing design. Sized as
choked, such a throat is too small and the engine flown at its own design point would not return to it. So both
throats are captured at the design's ACTUAL exit Mach (`MFP(M9R)`, `MFP(M19R)` — Mattingly's own form); where the
design nozzle is choked that is `MFP*` and the capture is the old one, bit for bit.

**G7 — what rung 111's switch will touch (counted, not estimated; recorded here so the user's decision has its bill).**
Controls, Cooling, Blades and Burner run their OWN engines and are untouched. On Fly it: experiments **9, 10, 11** of
19 (`docs/sandbox/EXPERIMENTS.md`); the **13** Fly-it browser checks (`rust/sandbox-wasm/browser.mjs` lines 166–292:
`fly_it_opens_on_the_design_point_and_matches_native` … `the_slam_is_refused_on_the_equilibrium_gas_in_words`); the
Try-this entries for experiments 9–11. Nothing of this moves in rung 110.

---

## 1. THE DEVICE

Stations: the two-shaft engine's 0 → 2 → **25** (fan root exit, core) → 3 → 4 → **45** → 5 → 9, plus the bypass
2 → **13** (fan exit, bypass) → **19**. One fan pressure ratio `πf` for both streams (Mattingly's 13 = 2.5 in
PRESSURE); their temperatures differ (D2).

* **D1 — the split rule (my call).** The design is rung 109's: one compressor (`πc`, its efficiency knob), one lumped
  turbine (its efficiency knob), a fan (`α`, `πf`, `ηf`/`ef`, `πfn`). Flying it needs two compressors and two
  turbines. The compressor's knob is converted to its POLYTROPIC efficiency on the design (`ec` such that
  `Compressor(πc, ec)` gives rung 109's `Tt3`), and the core's compression is split at `πf`: the fan root
  `Compressor(πf, ec)`, the HP compressor `Compressor(πc/πf, ec)`. The lumped turbine likewise: its `et` from the
  design, split at the HP shaft's work. A polytropic path composes exactly (`φ(T) ` is additive in `ln π` on every gas),
  so stations 3 and 5 are rung 109's, and no derived efficiency can exceed 1 (G4). Each part's ISENTROPIC efficiency
  at design is then held off design (the book's practice; the two-shaft core takes isentropic knobs only).
* **D2 — the fan's two streams.** The core air through the fan root at D1's root efficiency (rung 109's convention:
  no fan-root loss beyond the compressor's own); the bypass air at the fan's `ηf`. Off design both share the LP turbine's
  work through ONE isentropic rise: `Δhs = W_LPT/(1/η_root + α/ηf)` per kilogram of core air.
* **D3 — the bypass ratio is an OUTPUT.** `ṁ_bypass` = the fan nozzle's flow at `pt19 = πfn·πf·pt2`, `Tt13`, through
  its captured throat `A18`; `ṁ_core` = the HP vane's choked flow `A4·pt4·MFP*(Tt4)/√Tt4 / (1+f)`; `α = ṁ_bypass/ṁ_core`.
* **D4 — the fan nozzle** is rung 109's cold nozzle (`try_cold_nozzle` / `try_cold_sonic_throat`), convergent (the
  two-shaft engine's core nozzle is the fixed convergent one; the fan's follows it, rung 109's rule).
* **D5 — the throats** `A4`, `A45` as rung 38 captures them; `A8` and `A18` at the design's actual exit Mach (G6).
* **D6 — the LP turbine's law (my call):** rung 104's (the ellipse by default) — the law every two-shaft engine on the
  page already flies; the HP vane on rung 105's default (pin). Rung 103's below-choke switch ON and the standing engine
  ON, as the two-shaft pages set them. The book gate runs the pin (the book's own assumption).
* **D7 — the solve.** Rung 38's cascade at a fixed `(Tt2, Tt4, f)` — the turbines from their throats (rung 104's joint
  pair, rung 103's free root below choke) — then the LP shaft with D2's bypass term, then the HP shaft as rung 38's.
  The bypass ratio reads `pt4` (D3), so it joins the outer `(f, pt4)` fixed point. **At α = 0 no bypass term is
  entered**: the code path is rung 38's.
* **D8 — shaft speeds without a map** (Mattingly 8-52ag/ah, on the table gases as the isentropic rise):
  `N/N_design = √(Δhs/Δhs,design)` for the fan (on station 2) and the HP compressor (on station 25). An assumption the
  book makes (similar velocity triangles), disclosed; replaced by the maps' speeds in rung 111.
* **D9 — scoring** is rung 109's (`score_turbofan`: two jets, per total air, thrust-based efficiencies), so at the design
  flight and throttle the numbers are rung 109's.
* **D10 — refused in words:** a fan with no pressure rise flown with a bypass stream (`πf ≤ 1`, `α > 0`); the core jet
  with no pressure left (rung 109's `NO_CORE_PRESSURE` words); a fan nozzle that cannot pass its flow; a customer bleed,
  cooling, shaft offtake or afterburner with a fan (the two-shaft core's existing refusals — out of scope).

## 2. GATES (`rust/tests/rung110.rs`)

1. **α = 0 IS the two-shaft engine, bit for bit:** every station, thrust, `πLPC`, `πHPC` against `TwoSpoolCore::match_point`
   built the same way (D6's switches), on the perfect, thermally perfect and equilibrium gases, three flights, five
   throttles — wherever the design nozzle is choked (the captures coincide, G6).
2. **α = 0, πf = 1 IS the one-shaft engine** (`TwoSpoolMatcher::lp_disabled` → rung 31), bit for bit, where both run.
3. **The design reduce:** at the design flight and `Tt4`, the flown engine reproduces rung 109's design run — every
   station, both jets, thrust per air, TSFC — to **1e−9** relative (D1's polytropic inversions are converged to 1e−13),
   over gases × α {2, 6, 10} × `πf` × chosen and lossless loss sets, including designs whose core or fan nozzle is
   unchoked at design (G6).
4. **Continuity:** α = 1e−9 within 1e−6 of α = 0 on every gas, flying and standing.
5. **Example 8-8 through the model:** the book's reference engine (CPG, the book's printed component values, pin law)
   flown to sea-level static matches an independent re-derivation of the book's equations **with exact `(1+f)`** to
   **1e−10**; that re-derivation with the book's frozen `(1+f)` reproduces Table 8-7's first row and every printed
   result to the book's rounding (**≤ 0.15 %**); the model against the printed numbers within **0.3 %** (G2 + rounding).
6. **The perfect-gas closed forms (rung 31's self-consistency, two shafts):** on a single perfect gas with the pin law,
   `τtH, πtH` constant across a throttle sweep to 1e−12; `N_HP/N_R = √[(1+f)Tt4/((1+fR)Tt4R)]` to 1e−12 (G5 iv);
   with both nozzles choked and α = 0, `N_LP/N_HP` = 1 to 1e−12.
7. **The triangle at bypass 0:** perturbing the HP compressor's efficiency leaves `πf` bit-for-bit unchanged at α = 0
   (rung 38's gate 3, kept).
8. **Refusals in words** (D10), each driven by a measured failing point.

## 3. PREDICTIONS (scored HIT/MISS; written before the model)

**Grid:** gases {thermally perfect, equilibrium} × designs (α, πf) {(0, 1.6), (2, 3.46), (6, 1.81), (10, 1.48),
(6, 1.6)} (the fans at rung 109's best for cruise, plus the knob's default) × flights {cruise — the design flight, 250 K
/ 50 kPa / Mach 0.85; standing — 288.15 K / 101 325 Pa; climb — 268.65 K / 70 108 Pa / Mach 0.5} × `Tt4` {1500, 1400,
…, 900} K; the page's loss set; D6's law. 210 points. A point the model refuses is recorded and scored as a MISS
of every claim that needed it.

| | claim | cells |
|---|---|---|
| P1 | **The HP shaft's speed follows the turbine temperature alone:** `|N_HP/N_HP,design − √(Tt4/Tt4,design)| ≤ 0.010` | 210 |
| P2 | **Throttling back with the fan nozzle choked raises the bypass ratio:** α strictly rises for each 100 K step down where the fan nozzle is choked at both ends (fan designs) | the pairs |
| P3 | **Standing still the bypass ratio peaks inside the range:** α has an interior maximum over `Tt4` ∈ [900, 1500] at standing (fan designs) | 8 |
| P4 | **The bypass closes rung 38's triangle:** lowering `ηcH` by 0.01 LOWERS `πf` at every fan point (`dπf/dηcH > 0`) | 168 |
| P5 | **At high bypass the core goes negative in cruise:** the core's own net thrust crosses zero inside (950, 1250) K for (6, 1.81) and inside (1000, 1300) K for (10, 1.48); it stays positive down to 900 K for (0, 1.6), (2, 3.46), (6, 1.6) | 10 |
| P6 | **A fan slows faster than the core:** at every fan point with `Tt4 ≤ 1300`, `N_fan ≤ N_HP − 0.02`; at α = 0 with both nozzles choked, `|N_LP/N_HP − 1| ≤ 0.005` | the points |
| P7 | **Fuel per thrust:** standing, an interior minimum over [900, 1500] K for every fan design; in cruise, strictly rising as `Tt4` falls for every fan design | 16 |
| P8 | **A fan hides the LP turbine's law:** the thrust moved by switching the law (ellipse → pin) at a fan design is under a THIRD of what it moves at (0, 1.6) at the same gas, flight and `Tt4` (cells where the α = 0 effect ≥ 1e−4) | ≤ 168 |

*Reasoning, so a miss can be read:* P1 is G5 (iv) carried onto the table gases and rung 104's law (which unpins
`πtH` slightly); P2–P3 are G5 (i)–(ii); P4 is G5 (vi) — the sign: a weaker HP compressor lowers `πcH`, hence `pt4` and
the core flow, so α rises and each kilogram of air through the fan gets less of the LP turbine's work; P5 is G5 (iii),
its windows set around the probe's crossings; P6 is G5 (v); P7 is G5 (vii); P8: the turbofan's LP turbine expands ~4×
(`πtL` 0.21–0.25), where the ellipse `√(1−x²)` is nearly flat; the booster's expands ~1.15× (`πtL` 0.87), where it is
steep. **Named risks:** P1 on the equilibrium gas (the `(1+f)` and `cp(T)` terms); P5's windows (a perfect-gas probe
places them); P7's cruise claim at (2, 3.46) where the probe's rise is shallow at the top; P8 where the booster's own
effect is small.

## 4. WHAT SHIPS

The model and one CLI panel (`print_flown_turbofan_table`, born in Rust, held to `rust/oracle/rust_owned/`): the default
turbofan flown over the throttle at the design flight and standing — bypass ratio, both pressure ratios, both shaft
speeds, both nozzles' state, each jet's thrust, TSFC. **No page change** (the user's decision; rung 111 brings the
charts and switches Fly it for every design).

## 5. CONCESSIONS (named now)

* **No maps:** efficiencies held at design; shaft speeds by D8's similarity assumption; no stall margin.
* **D1's split is a convention** (polytropic, on the core); the book's own (fan efficiency on the core air) lives only
  in gate 5.
* **Steady only;** the slam, the governor and the limiters on this engine come with rung 111's maps.
* **Separate exhausts only;** no mixer, no afterburner with a fan (rung 109's refusal, kept).
