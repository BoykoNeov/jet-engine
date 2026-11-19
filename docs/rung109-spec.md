# Rung 109 — THE TURBOFAN: a fan whose air mostly goes around the core, on the design point

**Code:** `rust/src/turbofan.rs` — `Fan` (the knobs: `alpha`, `pi_f`, `eta_f` / `e_f`, `pi_fn`), `build_turbofan` /
`Turbofan` (`try_run`, `core_to_4`, `try_finish`), `try_cold_nozzle` / `try_cold_sonic_throat` (the bypass air's nozzle),
`score_turbofan` (two jets, per total air), `best_pi_f` (the fan pressure ratio with the lowest fuel per thrust).
**Gates and scoring:** `rust/tests/rung109.rs`. **Panel:** `print_turbofan_table` (born in Rust, held to
`rust/oracle/rust_owned/`; a change detector only). **Anchor:** `docs/plans/rung109-anchor-turbofan.md`, committed as
`19a2b6e` before any rung-109 code. **Sandbox:** the Design view's *Fan (turbofan)* box (`docs/sandbox/README.md`
§ Rung 109; gates `tests/sandbox_fan.rs`, the browser checks `the_turbofan_*`).

Roadmap B1; SPEC.md § Future directions B, *"Turbofan. Add a fan, a bypass stream, and a splitter."* The scope was set
on a measured probe, each answer the user's (memory `rung109-decisions`): the **design point only**, on the Design view;
the fan's nozzle **follows the core nozzle's mode**; **one lumped turbine** drives the compressor and the fan; three page
extras (the best fan pressure ratio, a fuel-versus-bypass chart, two-jet efficiency rows). The afterburner with a fan is
refused in words (my scope call: an afterburning turbofan mixes its streams first).

> **THE HEADLINE — a fan trades jet speed for airflow, and the best fan does NOT make the two jets equally fast.**
> On the same core, fuel per unit of thrust falls with bypass ratio: by bypass ratio 12 (each at its best fan) to
> 0.55–0.65 of the turbojet's at 11 km Mach 0.8, and to 0.33–0.36 standing still, where the fan keeps paying long after
> it has flattened in flight. At the best fan pressure ratio the fan's jet leaves at **0.70–0.81 of the core jet's
> speed**, not at the ideal engine's "equal jets": moving work from the core jet to the fan passes through the turbine
> and the fan, each losing a share, so the best split stops short — near the product of their two efficiencies
> (0.88 × 0.90 ≈ 0.79). A convergent fan nozzle costs almost nothing (0.4–1.3 % at bypass 8, against 8.7–15 % for the
> turbojet), because the fan's low pressure ratio has little to waste. Mattingly's Example 7-6 runs through the shipped
> model to his own equations at 1e−12.

## The device (anchor § 1)

Stations 0 → 2 (fan face) → **13** (fan exit, bypass side) → **19** (fan-nozzle exit) beside the core's 2 → 3 → 4 → 5 → 9.
The core air passes the fan root inside the compressor's ratio (2 → 3 is the whole core compression, the book's
convention); `π_fn` lumps the bypass duct and the fan nozzle (13 → 19 totals).

* **The fan** is the shipped `Compressor` on station 2's cold air at `π_f`, with `α·ṁ_core` of mass. Its efficiency
  follows the compressor's switch (isentropic `η_f` 0.88, or polytropic `e_f`).
* **The shaft:** `Δh_turbine = [h_c(Tt3) − h_c(Tt2) + α·(h_c(Tt13) − h_c(Tt2))] / (η_m·(1+f))` — Mattingly's 7-52o on the
  perfect gas. The shaft-closure check (every run) carries the fan's term.
* **The cold nozzle** (`try_cold_nozzle`): the shipped nozzle's three laws on the cold section, in the core's mode;
  convergent ⇒ a cold sonic throat (closed form on the perfect gas, bisection on the tables).
* **Thrust per total air** `F/ṁ0 = [F_core + α·F_fan]/(1+α)`, `F_fan = V19 − V0 + R_c·T19·(1 − p0/p19)/V19`;
  `TSFC = f/[(1+α)·F/ṁ0]`. Thermal efficiency = kinetic energy added to BOTH jets per fuel energy; overall =
  thrust power per fuel power; propulsive = overall ÷ thermal. Standing still both are exactly 0.
* **The reduce:** α = 0 is `build_turbojet(…).run(…)` by DISPATCH — no fan arithmetic is entered.
* **The wall** (`NO_CORE_PRESSURE`): if even an IDEAL turbine would leave the gas no warmer than the outside air, the
  core jet is below outside pressure — `pr(Tt4)/pr(Tt0) > pr(Tt3)/pr(Tt2) ≥ π_c ≥ π_d·π_c·π_b`, so `pt5 > p0` needs
  `Tt5s > T0`. It is checked BEFORE the gas tables are asked to invert an enthalpy below their range (the first
  scoring run hit exactly that, `gas.rs:511`), and gives the core nozzle's own refusal in words. With a set exit
  pressure BELOW the outside pressure the words are a slight overstatement (the jet may still leave); disclosed.
* **The fan's floor:** a fan whose rise does not cover the inlet and duct losses (standing still, `π_f` near 1)
  is refused in words naming it.
* **The best fan** (`best_pi_f`): a 31-point scan of `π_f` ∈ [1, 4]; where the best point's neighbour does not run,
  the runnable edge is found by bisection and becomes the bracket's end; a golden-section refinement to 1e−4. It
  reports whether the best sits at the knob's top (4.0) or against the turbine's wall. *Found while scoring:* the
  first version refined only between RUNNABLE grid neighbours, so where the runnable window was narrower than the
  grid step (high bypass, high pressure ratio, cool turbine) it returned a grid point far from the wall-limited best
  (on the perfect gas at standing, `π_c` 40, `Tt4` 1400, α 12: `π_f` 1.1 with the fan jet at 0.18 of the core's).

## Givens (anchor § 0)

Mattingly, *Elements of Gas Turbine Propulsion* (McGraw-Hill 1996), Example 7-6, re-derived from its inputs and run
through a probe: every printed row within 0.05 % (the book's own rounding). **The book's one inconsistency:** its
printed propulsive efficiency 74.65 % is not its printed formula (which gives 66.28 % and holds only at `P9 = P0`); the
printed 30.60 % / 74.65 % are thrust-based — the trap roadmap A6 fixed for one jet. The rung follows the thrust-based
reading.

## Gates (anchor § 2) — measured (`tests/rung109.rs`)

* **α = 0 ⇒ the turbojet, bit for bit:** five gases × three nozzle modes × flying and standing (30 cases), every
  station and every performance number.
* **Continuity:** α = 1e−9 within 1e−6 of α = 0 on every gas, flying and standing; bypass mass `= α·ṁ_core` exactly.
* **Example 7-6 through the shipped model:** 17 printed rows within 0.1 % (largest 0.049 %, `π_t` and `FR`).
* **Mattingly's equations to 1e−12:** `f`, `τ_t`, `F/ṁ0`, `S`, `η_T` at Example 7-6 (`P0/P9` 0.9), the same fully
  expanded, and a standing point (α 5, `π_f` 1.5, `π_c` 25).
* **The cold nozzle IS the hot one on a single perfect gas,** bit for bit, in all three modes; its sonic throat is the
  closed form `T* = 2Tt/(γ+1)` to 1e−12.
* **Refusals in words:** the turbine's wall (α 10, `π_f` 3) and the fan's floor (standing, `π_f` 1.02).
* **The anchor's** "the probe's values to 1e−12" is met by the equations gate: the probe printed 6–7 digits only.

## Scored (anchor § 3) — 256 HIT of 276

| | claim | HIT | MISS |
|---|---|---|---|
| P1 | fuel per thrust falls strictly through α 0 → 1 → 2 → 4 | 36 | 0 |
| P2 | the α 8 → 12 fall is under a third of the 2 → 4 fall | 24 | 12 |
| P3 | the best fan pressure ratio falls strictly through α 2 → 4 → 8 → 12 | 36 | 0 |
| P4 | at the best fan, `V19/V9` ∈ [0.70, 0.88] (α 2, 4, 8, 12) | 136 | 8 |
| P5 | 11 km: convergent costs less at α 8 than for the turbojet, and < 3 % | 12 | 0 |
| P6 | standing: α 8 multiplies the core's thrust by 2.0–3.0 | 12 | 0 |

**The misses are findings.** **P2's 12 are EVERY standing-still cell**, the risk the anchor named: standing still the
fan keeps paying (fuel per thrust at α 12 is 0.33–0.36 of the turbojet's, against 0.55–0.65 at 11 km). Every flying
cell hit. **P4's 8:** six are α 2 in flight at 1700 K, where the best fan lies above the knob's top (4.0) — the ratio
there (0.58) is read at the knob's edge, not at the best; two are standing, `π_c` 40, `Tt4` 1400, α 12 at 0.6988, just
under the band. Over every scored cell whose best is inside the range, the ratio runs **0.6988 to 0.8085**, falling as
bypass ratio rises — the "two efficiencies" argument sets the level, the fuel's mass and the duct losses tilt it.
P5's penalties: turbojet 8.7–15.2 %, α 8 0.39–1.30 %. P6: 2.43–2.56×.

## The page (anchor § 4)

Bypass ratio 0 is the turbojet's reply byte for byte (every key but the echoed settings, `tests/sandbox_fan.rs`). The
page's thrust is `(F_core + α·F_fan)·ṁ_core` (its airflow knob is the core's); the efficiency split is roadmap A6's on two
jets (`sandbox::equivalent_jets`, which reduces to the one-jet split at α = 0 bit for bit). **The cost bar:** the best-fan
search is 0.4–0.8 ms against a 1.0 ms design run (native release, equilibrium gas), inside the anchor's 10×, so it runs
on every change at full resolution; a whole turbofan run with the chart's twelve searches is 6.6–10 ms. *Found while
writing the page's experiment:* on the opening design (Mach 0.85, 5.6 km) the best-fan fuel per thrust has an interior
minimum near bypass ratio 11 (22.564 g/(kN·s), against 22.575 at 12): past it, a bigger fan at its best costs slightly
more fuel per thrust.

## Concessions

* **Design point only.** The fixed turbofan flown off design (a fan map, two nozzles sharing the flow, a fan shaft
  of its own) is the next rung's.
* **One lumped turbine.** At the design point a split into HP and LP turbines changes how the work is shared, not the
  thrust or fuel; it arrives with the off-design engine.
* **Separate exhausts only.** No mixer; the afterburner with a fan is refused.
* **No fan-root loss on the core air** beyond what the compressor's own efficiency carries (the book's convention).
