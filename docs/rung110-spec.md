# Rung 110 — THE TURBOFAN FLOWN: rung 109's fixed turbofan off design, on the two-shaft engine

**Code:** `rust/src/flown_turbofan.rs` — `split_rung109` (rung 109's design onto two shafts, D1), `FanComponents`,
`fly_rung109` / `FlownTurbofan` (`try_match`, `match_point`, `set_turbine_law`), `FlownPoint` (both jets, `alpha`,
`n_fan` / `n_hp`, `core_choked` / `fan_choked`). The bypass stream lives INSIDE rung 39's loop in `rust/src/two_spool.rs`
(`try_fan_split`, the `(f, pt4)` fixed point) — one shared loop, the user's decision. **Gates and scoring:**
`rust/tests/rung110.rs`. **Panel:** `print_flown_turbofan_table` (born in Rust, held to `rust/oracle/rust_owned/`; a
change detector only — its engine is the grid's (6, 1.6) equilibrium cell, so the grid checks its numbers).
**Anchor:** `docs/plans/rung110-anchor-flown-turbofan.md`, committed before any rung-110 code. **No page change** (the
user's decision: rung 111 brings the maps and switches Fly it for every design).

Roadmap B1's second half; rung 109's first two concessions (design point only; one lumped turbine). The scope was the
user's (memory `rung110-decisions`): the textbook method (efficiencies held at design, both high-pressure throats
choked, no maps); **ONE solver for every design** — the two-shaft engine of rungs 38/39/103/104 given a bypass stream,
so at bypass 0 it IS that engine; Fly it switches over only when the maps arrive.

> **THE HEADLINE — a fan closes rung 38's triangle, and opens the core nozzle.** Flown off design, the bypass ratio is
> an OUTPUT of two fixed nozzles sharing the fan's air, and it reads the core's flow — which reads the HP compressor. So
> with a fan the LP shaft HEARS the HP compressor (a 0.01 weaker HP compressor lowers the fan's pressure ratio at every
> fan point that runs, 162 of 168), where at bypass 0 rung 38's triangle keeps it deaf (bit for bit). And because the
> fan takes most of the LP turbine's work, the core's own jet weakens: its nozzle unchokes at part throttle, rung 104's
> LP-turbine law takes over, and the HP shaft's speed falls BELOW the textbook's "turbine temperature alone" (by up to
> 0.16 standing at 900 K; 0.005 wherever the core nozzle stays choked). In cruise at high bypass the core's own net
> thrust crosses zero near 1070–1135 K — below it, the fan carries the engine.

## The device (anchor § 1)

Stations: the two-shaft engine's 0 → 2 → 25 → 3 → 4 → 45 → 5 → 9, plus the bypass 2 → 13 → 19. One fan pressure
ratio for both streams.

* **D1 — the split (my call):** rung 109's compressor and lumped turbine converted to POLYTROPIC efficiencies on the
  design, then split at `πf` (fan root + HP compressor) and at the HP shaft's work. Composition is exact, so stations
  3 and 5 are rung 109's and no derived efficiency exceeds 1 (anchor G4). Each part's isentropic efficiency at design
  is then held off design.
* **D2–D3 — the LP shaft pays for two streams:** `W_LPT = Δhs(πf)·(1/η_root + α(πf)/ηf)` per kilogram of core air; the
  bypass ratio is the fan nozzle's flow over the core's (`α = A18·(ρV)19 / ṁ_core`, `ṁ_core` the HP vane's choked
  flow). Both terms rise with `πf`, so the root is unique (bisection from the lowest ratio that still blows air out).
  `ṁ_core` reads `pt4`, so the bypass joins the outer `(f, pt4)` fixed point.
* **D5 / G6 — ONE nozzle rule for every engine** (the user's decision, with a stop-if-anything-moves condition that
  never fired): `A8` and `A18` are sized at the design's ACTUAL exit Mach, so an engine whose nozzle is unchoked at
  design returns to its design point. Where the design nozzle is choked the capture is the old one, bit for bit.
* **D6:** the LP turbine on rung 104's law (the ellipse); the book gate runs the pin.
* **D8 — shaft speeds without a map:** `N/N_design = √(Δhs/Δhs,design)` for the fan (on station 2) and the HP
  compressor (on 25) — Mattingly's similarity assumption, disclosed.
* **D9:** scoring is rung 109's `score_turbofan`; standing still it goes through `score_or_static`.

**Built and found while building** (each a solver fact, not physics):
* **The fan loop's tolerance is 1e−10**, not 1e−13: the table gases' inversion noise (~2e−11) made the tighter bar a
  two-cycle. The loop gets 1 000 passes and a rounding floor (1e−9) for the slow crawl just below choke (gate 4b).
* **Rung 104's choked-nozzle shortcut** is skipped when the design nozzle was unchoked (it assumed the capture's choke).
* **The one-shaft end** (`α` 0, `πf` 1) flies on rung 31's matcher, as the anchor's gate 2 requires.
* **A bypass stream with no fan rise** (`α > 0`, `πf ≤ 1`) is refused at `fly_rung109` in its own words; before, it hit
  an internal check with misleading ones.

## Gates (anchor § 2) — measured

1. **α = 0 is the two-shaft engine, bit for bit:** 90 points (3 gases × 3 flights × 5 throttles × designs), every
   station and number; none refused by one side only.
2. **α = 0, πf = 1 is the one-shaft engine (rung 31), bit for bit:** 90 points.
3. **The design reduce:** (3a) the split design engine is rung 109's design run — 44 designs, worst 2.1e−10; (3b) the
   flown engine returns to its design — 44 designs (core choked 24 / unchoked 20; fan unchoked 14), worst 2.7e−10.
4. **Continuity:** α 1e−9 within 1e−6 of α 0, 90 points. **4b:** 40 designs unchoked at design return home (worst
   5.9e−9), and 227 points show no step at α → 0.
5. **Example 8-8 (Mattingly 1996, pp. 526–529) through the model:** the model against the book's equations re-derived
   with exact `(1+f)` — 1.1e−11; that re-derivation with the book's frozen `(1+f)` against every printed number — 0.117 %
   (Table 8-7's first row ≤ 0.032 %). **MISS, recorded:** the anchor's "model within 0.3 % of the printed numbers" —
   thrust is 0.367 % off. The gap is explained, and the explanation is asserted instead (the user's decision): exact
   `(1+f)` moves `α` +0.031 %, `πcH` −0.142 %; the model's own consistent reference moves the rest.
6. **Perfect-gas closed forms** (single perfect gas, pin law): `τtH, πtH` constant, `N_HP` on its closed form
   `√[(1+f)Tt4/((1+fR)Tt4R)]`, and `N_LP = N_HP` at α 0 with the core choked — worst 3.1e−13 (bar 1e−12 at α 0, 1e−9
   with a fan: the fan loop's own promise).
7. **The triangle at bypass 0:** a 0.01 weaker HP compressor on the FROZEN hardware leaves `πf` bit for bit unmoved —
   on the perfect and thermally perfect gases, 42 choked points. On the equilibrium gas it moves, by 6.4e−7 of
   `πcH`'s relative move at worst — **CONFIRMS rung 38's own caveat** (`docs/rung38-spec.md`, "the one place the spools
   still talk"): the triangle is exact at a fixed `f`, and on the equilibrium gas the burner's `f` reads `pt4` (its
   dissociation), so the shafts still talk through `f`, weakly. Asserted `≤ 1e−3` there; bit for bit elsewhere.
8. **Refusals in words:** a bypass with no fan rise; the core with no pressure left (rung 109's words); the customer- and
   interstage-bleed matchers handed a turbofan. **No gate for "the fan nozzle cannot blow any air"** (`FAN_NOZZLE_DRY`):
   a wide search (`probe_dry`, ignored: 3 gases × 2 loss sets × 7 designs built at cruise and standing × 5 flights, each
   throttle line scanned down to its edge and bisected to 0.1 K) NEVER met it — the core nozzle refuses first (rung 97's
   "cannot pass the LP turbine's flow", rung 103's "below idle"). It stays in the code as a guard; a gate on an invented
   point would test nothing.

## Scored (anchor § 3) — 480 HIT of 632 (P8 vacuous)

| | claim | HIT | MISS |
|---|---|---|---|
| P1 | `N_HP` within 0.010 of `√(Tt4/Tt4,design)` | 93 | 117 |
| P2 | α rises for each 100 K down, fan nozzle choked | 54 | 6 |
| P3 | standing, α peaks inside [900, 1500] K | 4 | 4 |
| P4 | a weaker HP compressor lowers `πf` (fan designs) | 162 | 6 |
| P5 | the core's net thrust crosses zero in cruise in the windows / stays positive | 7 | 3 |
| P6 | `N_fan ≤ N_HP − 0.02` (fan, ≤ 1300 K); `N_LP = N_HP` ± 0.005 at α 0 choked | 150 | 10 |
| P7 | fuel per thrust: standing interior minimum; cruise strictly rising | 10 | 6 |
| P8 | a fan hides the LP turbine's law (under a third of the turbojet's effect) | 0 | 0 |

Frozen at the first run (`assert_eq!`); a moved count is a finding, never a re-freeze. **The misses are findings:**

* **P1's 117 are ALL fan designs: 111 where the core nozzle is open, 6 refused.** The turbojet (0, 1.6) hits all 42 cells.
  Worst `|N_HP − √|`: 0.0052 where the core nozzle is choked, **0.164 where it is open** (standing, (10, 1.48), 900 K);
  on the pin law, 0.0046 everywhere. So the textbook's "HP speed from turbine temperature alone" holds while both HP
  throats AND the core nozzle are choked; a fan unchokes the core nozzle at part throttle (it took the LP turbine's
  work), and rung 104's law then lets the HP turbine's pressure ratio fall — rung 104's headline, now reached through
  the fan. The anchor named P1 on the equilibrium gas as the risk; the cause was the nozzle, on both gases equally.
* **Refusals (counted as misses):** design (2, 3.46) standing at 1000 and 900 K, and (6, 1.81) standing at 900 K, are
  refused (rung 97: the core nozzle cannot pass the LP turbine's flow). They make P1's 6, P2's 6, P3's 4, P4's 6, six of P6's
  and four of P7's misses.
* **P5's 3:** the core goes (slightly) negative at 900 K in cruise where the anchor said it would stay positive —
  (2, 3.46) on both gases (−4.1 / −7.4 N·s/kg) and (6, 1.6) on the equilibrium gas (−2.3). The two windowed designs
  hit: zero crossings at 1070 / 1075 K (6, 1.81) and 1129 / 1134 K (10, 1.48).
* **P6's other 4:** (2, 3.46) in cruise at 1300 and 1200 K — the fan slows only 0.010 / 0.018 faster than the HP shaft
  (a high-pressure-ratio fan behaves more like a booster).
* **P7's other 2:** (2, 3.46) in cruise, fuel per thrust dips 0.04–0.4 % from 1500 to 1400 K before rising — the risk
  the anchor named.
* **P8 is VACUOUS — untested, not passed:** it scores only cells where the turbojet's own thrust moves ≥ 1e−4 when the
  law is switched, and the largest anywhere is **2.7e−5**. The threshold was not moved after the run; the test asserts the vacuity, so a later change that gives it
  cells fails loudly.

## Concessions

* **No maps:** efficiencies held at design; shaft speeds by D8's similarity; no stall margin (rung 111).
* **D1's split is a convention;** the book's (fan efficiency on the core air) lives only in gate 5.
* **Steady only;** the slam, the governor and the limiters on this engine come with rung 111.
* **Separate exhausts only;** no mixer, no afterburner with a fan (rung 109's refusal, kept).
* **OPEN — the edge of running gives a solver message.** On 169 of `probe_dry`'s 420 throttle lines, the point within
  0.1 K of the last that runs is refused with "the turbofan's operating point did not settle" instead of the physical
  reason (below idle / the nozzle cannot pass the flow) a step further down gives. No wrong number comes out; the words
  are unhelpful. Nothing shows it today (no page runs this engine; the panel reads 900–1500 K, far from the edge). Asked
  of the user 2026-11-23, not yet answered; rung 111 (which puts this engine on the page) must settle it.
* **Four designs built standing with the page's losses do not fly** at their own design ((6, 1.81) and (10, 1.48) on
  the perfect gas, (10, 1.48) on the table gases): found by `probe_dry`; § 3 builds every design at cruise.
