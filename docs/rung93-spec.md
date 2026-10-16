# Rung 93 — METAL TEMPERATURE: what the turbine's metal allows, and what cooling buys

**Code:** `rust/src/metal_temperature.rs` (Gauntner's correlation forward and inverse — `coolant_flow`,
`effectiveness`, `metal`, `try_need`, `allowable`; the reading of a point `MetalInputs` / `read`; framing B
on rung 91's fixed hardware, `FixedEngine::try_max_tt4` and `FixedEngine::try_limited`; framing A, the cooled
design run `try_cooled_design_run` and `try_limited_design`) · **Gates:** `rust/tests/rung93.rs`, the page's
`rust/tests/sandbox_cooling.rs` · **Panel:** `print_metal_temperature_table` (born in Rust, held to
`rust/oracle/rust_owned/`, a change detector only) · **Anchor (the correlation, the model and every bar,
committed `e2a6c43` before any metal code existed):** `docs/plans/rung93-anchor-metal-temperature.md` ·
**Sandbox:** the new *Cool the turbine* view, and a metal readout on the Controls slam
(`docs/plans/sandbox-plan.md` § 22) · **Scratch scripts and outputs:** `docs/plans/scratch-archive/rung93/`.

**Shipped as KNOBS** (the sandbox direction): predictions were registered first and are scored here; a miss
is a finding.

Rung 92 closed by naming this seam: *"A metal-temperature limit that lets cooling buy a hotter burner (rung
91's), and with it a burner that carries the load."* The user chose (2026-10-16) TWO rungs — this one, and
rung 94 for the burner that carries the load — a metal temperature for the vane AND the rotor blade, and both
framings as a switch: *this engine, run hotter* and *redesigned at each temperature*; on the page, a new view
plus an opt-in readout on the slam, the limits set by a material year and a part life.

> **THE HEADLINE — cooling never stops the engine, and on fixed hardware it cools its own coolant.** With
> Gauntner's 1980 NASA correlation (as published, his hot spot, safety margin and 1985 metals), each percent
> of vane air buys about 100 K of burner temperature and costs about half a percent of thrust, so **thrust at
> the metal limit rises with the burner temperature through 2200 K for every cooling technology, on the fixed
> engine and on a fresh design**: there is no best cooling share — the walls are elsewhere (speed, flow). On
> fixed hardware each share of cooling slows the HP spool, so the compressor delivers COLDER air: the engine
> needs 5 % less cooling than a frozen sum at 1500 K and 22 % less at 2000 K (film). So the fixed engine and a
> fresh design TRADE PLACES — below a crossing the fixed engine needs less air, above it more — and the crossing
> **rises as the technology worsens**: 1624 K (transpiration), 1672 K (film), 1875 K (coated), above 2200 K
> (convection) — more coolant cools its own source more. A two-term model written before any metal code
> predicts the fixed engine's cooling to ≤ 5.3e−4.
>
> **On a fresh design, cooling LOWERS the fuel bill at a fixed burner temperature.** The burner gets less air
> at the same fuel ratio while thrust falls only part as fast: TSFC at the metal limit sits 0.18 → 2.9 mg/N·s
> below the uncooled design's (film, 1300 → 2200 K). The metal-limited engine is, in fuel, a cooler engine.

## 1. What it adds

**The metal of a row** (Gauntner inverted). A row sees `T = T_row + S` (vane: `Tt4`; blade: `0.92·Tt41`, the
relative frame; `S` = 150 °R), its hot spot `T + P(T − Tc)` (`P` 0.30 vane, 0.13 blade; `Tc = Tt3`, the cooling
air), and its stream `w` (over the gas entering the row) buys the effectiveness `φ` of
`¾w = 0.022 F (φ/(1−φ))^1.25` (`F` the technology: 2.0 convection, 1.5 coated, 1.0 film, 0.8 transpiration; ¾:
Gauntner's endwall `4/3`): the metal is `T_hot − φ(T_hot − Tc)`. The limits: `(10·YEAR − 17640) − (100 log₁₀ LIFE
− 400)` °R (vane), 100 °R lower for the blade. **It moves no gas**: a metal reading is algebra on a rung-91 point.

**What the metal decides.** *Framing B* (rung 91's fixed hardware, sized uncooled at 1500 K): the hottest
`Tt4` at given shares (bisected), and the shares that hold both metals at their limits at a given `Tt4` — a
fixed point through the plant (each pass matches the engine at the current shares, reads its temperatures,
solves the 2×2 linear share equations; 8–40 passes, contracting because a share moves its own coolant only
weakly). *Framing A*: the design run with rung 91's two mixers inside (`try_cooled_design_run`); there `Tt3` is
fixed by `π` and `f` by `(Tt3, Tt4)`, so the closed form is exact (3 passes).

**Where it lives.** Beside rung 91's point; nothing runs unless asked. The perfect gas only (the mixers).
Bounds in words: a limit at or below the cooling air is refused ("no amount of that air"); a root needing a
stream past rung 91's quarter is refused at the ROOT ("cannot hold the metal") — trials are held inside the
cap, since `TurbineCooling::new` panics there.

## 2. Reduces

No metal asked for ⇒ nothing changes: no oracle, no CLI golden, no earlier gate moved; the default design run
is rung 6. The cooled design run with no active cooling IS `TwoSpoolEngine::run` (dispatched, counted, gated
bit for bit at three `Tt4`); its cooled body at a 1e−9 share lands on it to 1e−7. Its mixers and both shaft
balances close (asserted). The transient readout's `MetalInputs::of_shares` equals the station reading on a
steady rung-91 point to 1e−12.

## 3. Scoring (anchor § 3 — bars frozen at `e2a6c43`)

Rig A (rungs 42/90–92's: CPG, π 3 × 6, design 1500 K, 250 K / 50 kPa / M 0.85), flat and `flow/press`,
Gauntner's 1985 metals, `Tt4` 1300 → 2200 K by 100, FACTOR 2.0 / 1.5 / 1.0 / 0.8.

| # | verdict | measured |
|---|---|---|
| M1 B magnitude | **HIT** 80/80 | within 5.25e−4 of Model 1 (bar 1.5e−3); worst at FACTOR 2.0, 2200 K — the named likeliest miss, held |
| M2 cooling cools its coolant | **HIT** 80/80 | the fixed engine's total below Model 0's everywhere: film 0.05386 vs 0.05666 (1500 K), 0.1482 vs 0.1905 (2000 K) |
| M3 the crossing | **HIT** 4/4 | 1624 K (bar 1626 ± 70), 1672 K (1676 ± 70), 1875 K (1889 ± 70); FACTOR 2.0: none ≤ 2200 K (B − A = −0.0053 at 2200 K) |
| M4 no best share | **HIT** 12/12 | thrust per inlet air at the limit rises monotonically on B (both maps) and A, every FACTOR |
| M5 cooling lowers the fuel bill | **HIT** 4/4 | A's TSFC below the uncooled design's and the gap widening: film −0.18 → −2.93, convection −0.34 → −4.96 mg/N·s |
| M6 the knob | **HIT** | the binding row switches once, vane → blade, at **3.382 %** vane air (bar 3.38 ± 0.10 %); the blade's limit rises 1464 → 1663 K with vane air 0 → 16 % |

Construction (no credit): Gauntner's own example and worked limits to the °R, Kandiyoor's Fig. 6.5 (0.198 /
0.099) and the Y&W case's FACTOR 2.016; the correlation round-trips to 1e−12; A ≡ the closed form to 1e−9; both
B roots put both metals at their limits to ≤ 5e−9 K, and the knob returns the limited point's `Tt4` to 1e−5 K;
every refusal in words.

## 4. After scoring (reported, not scored)

* **M5's size was the borrowed slopes'.** The model, with the fixed engine's thrust slopes, said −0.35 → −4.66
  mg/N·s (film); a fresh design saves half to two-thirds of that: there cooling costs more thrust per share than on the
  fixed engine. The sign was the claim.
* **Where the fixed engine goes when it runs hotter.** With convection the HP spool stays at 0.90–0.94 of design
  speed all the way to 2200 K — the cooling slows it as fast as the heat speeds it — and the LP face sits at
  flow coefficient 0.84–0.94 (design 1); with film the HP reaches 1.03 and the LP 1.16 of design speed by 2200
  K. Those are the walls this rung does not model (rung 85's redlines, the invented maps' far reach).
* **The uncooled engine is metal-limited at 989 K** (the vanes): the sandbox's default slam (1000 → 1400 K) is
  over the limit at both ends without cooling, which is why the slam readout is off by default.
* **The slam.** The fuel-metered slam's `Tt4` overshoots its end by 230–580 K (§ 0 G4) while `Tt3` — the cooling
  air — lags 33–72 K. The metal is an algebraic function of both at each instant, so the slam has no model
  content beyond arithmetic; the page reads it, peak against limit. It is quasi-steady (no blade thermal lag,
  which rung 37's heat-soak metal is not): an upper bound. Read on the page's default slam with 5 % vane and 3 %
  blade air (film, 1985 metals; `tests/sandbox_cooling.rs`): the end holds at 1062 / 1026 K (vane / blade), and
  the overshoot lifts the metal to 1201 / 1163 K at the peak — 137 K above the blade's end value, within 9 K of
  its 1172 K limit (the vane within 27 K). The slam, not the steady throttle, sets how much cooling is enough.

## 5. Why — the model

Anchor § 1. The inverse gives the hottest gas a row can see, `T_seen,max = Tc + (T_lim − Tc)(1+y)/(1+P)`, with
`y ∝ w^0.8`: nearly linear in the coolant, so returns diminish only weakly, and the thrust a hotter burner buys
beats the ~0.5 %-per-percent the air costs. On fixed hardware `Tt3 = Tt3,line(Tt4)·(1 + a₁ε₁ + a₂ε₂)`, `a` ≈ −0.65
to −0.97 per unit share (measured; linear and additive to 0.2 K at 15 % cooling; map-independent): a fresh
design's `Tt3` is the design value at every `Tt4`, the fixed engine's rises with `Tt4` and falls with its own
cooling — whence the crossing, and its rise with the technology factor.

## What it does to its neighbours

* **Rung 91 — its concession DISCHARGED.** "At a fixed burner temperature cooling can only look like a cost —
  what it buys is the rotor-inlet temperature": it buys burner temperature, about 100 K a percent, and on a
  fresh design it buys fuel too.
* **Rung 91 — its thrust bill READ.** 5 % of vane air costing a third of a customer share's thrust is why
  cooling never stops the engine: the bill is too small against what it buys.
* **Rung 46 — its redline given a source.** A turbine temperature limit is a metal limit seen through the
  cooling: on this engine the same 1985 metals allow 989 K uncooled and ~1500 K with 3.4 % vane and 2 % blade
  air.

## Concessions

* **Gauntner's correlation as published**: a 1980 fit with fixed constants (0.022, 1.25, the 0.92 relative
  frame, the hot-spot and safety allowances); no cooled-turbine efficiency penalty (his DELV/DELN is not
  modelled), no mixing pressure loss, coolant at `Tt3` (no cooler). Read trends and comparisons, not the last
  kelvin.
* **One stage, two rows.** The LP turbine is uncooled; a row's whole stream counts as its coolant (rung 91's
  streams), the endwall quarter included.
* **Constant heat transfer off design**: the coolant-to-gas ratio is the right group only at a fixed Stanton
  number and geometry; framing B uses it across a 900 K band.
* **Hardware sized uncooled at 1500 K** (rung 91's); above ~1600 K framing B runs on the invented maps' far
  reach.
* **Quasi-steady metal** in the slam (an upper bound on the peak).
* The perfect gas only; no customer bleed or shaft load in the cooling searches (the readout carries both).

## The next seam

**Rung 94 — a burner that carries the load** (already chosen): each throttle end holds thrust or HP speed and
the burner temperature floats to carry rung 92's generator — and now the metal says how far it may float.
