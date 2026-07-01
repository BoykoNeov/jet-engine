# Turbojet Cycle Simulator

A station-by-station model of a single-spool turbojet (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram. **Rung 1** is the ideal cycle;
**rung 2** adds real components (isentropic efficiencies + pressure losses) and a
dual-section (cold/hot) gas, anchored to a published textbook case; **rung 2b**
adds polytropic efficiency `e_c/e_t` as a first-class knob beside the isentropic one;
**rung 3** upgrades the working fluid from calorically perfect (constant `cp`) to
thermally perfect (`cp = cp(T)`, gas-table `h`/`pr` property functions); **rung 4**
makes the hot-section **composition track the fuel/air ratio** `f` (explicit `(CH₂)ₙ`
lean-combustion stoichiometry → `cp_t/R_t/γ_t = f(T,f)`), which turns the burner into
an implicit `f = g(f)` fixed-point solve.

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

## Read this first
- **`SPEC.md`** — rung-1 contract + thermodynamics handout: station equations,
  shaft balance, the frozen ideal assumptions, the validation case.
- **`docs/rung2-spec.md`** — rung-2 contract + handout: efficiencies, dual gas,
  the reworked asserts, the two thermal efficiencies, the verification gates.
- **`docs/rung2b-polytropic.md`** — rung-2b contract + handout: polytropic `e_c/e_t`
  as a first-class knob, the `η_c < e < η_t` asymmetry, the equivalence gate.
- **`docs/rung3-variable-cp.md`** — rung-3 contract + handout: the `h`/`pr` property
  interface, the closed-form-CPG-branch trap, station equations in enthalpy form.
- **`docs/rung4-reacting-products.md`** — rung-4 contract + handout: `(CH₂)ₙ`
  stoichiometry, the `f`-parameterized properties, the implicit burner, the
  Fork-A/Fork-B datum split.
- `docs/plans/` — living plan/tasks (rungs 1–3), plus `rung2-anchor-mattingly.md`,
  `rung3-anchor-cengel.md`, and `rung4-anchor-mattingly.md` (the verified textbook
  anchor data).

## Working contract (from SPEC.md — these override convenience)
- **Derive before you code.** For each station, write the governing equation and
  a one-line physical justification (why it holds) *before* implementing it.
- **Show the work.** Every run prints the full station table (Tt, pt, …) so the
  numbers can be watched propagating.
- **Pure components.** Each component is `apply(state, gas) -> state` with no
  hidden state (Turbine and Nozzle diverge their signatures by design).
- **Conservation checks are assertions**, run on every execution (not as
  separate tests). See SPEC.md / docs/rung2-spec.md § Conservation checks.
- **Current scope (rung 4):** ideal + real components (isentropic `η_c/η_t` **or**
  polytropic `e_c/e_t`, mutually exclusive; pressure ratios `π_d/π_b/π_n`, `η_b`,
  `η_m`, dual cold/hot gas, specified exit pressure) on a **thermally-perfect** gas
  (`cp = cp(T)`; calorically-perfect sections kept as the closed-form branch) — with
  the hot section now a **reacting** gas whose composition (and `cp_t/R_t/γ_t`) tracks
  `f` via explicit `(CH₂)ₙ` lean-complete-combustion stoichiometry, solved through an
  implicit `f = g(f)` burner. Fork A: fixed `hPR`, `h(0)=0` datum. Still deferred —
  keep the seams: **rich combustion + high-temperature dissociation and
  formation-enthalpy bookkeeping (Fork B) = rung 5**, off-design / component maps, a
  *choked* convergent nozzle, afterburner.
- **Stop and explain surprises.** If a number looks off, reason about the
  physics rather than silently moving on.

## Conventions
- **SI units throughout** (K, Pa, kg/s, m/s, J/kg). Convert kPa → Pa internally.
- The cycle runs in **total (stagnation)** quantities `Tt, pt`; convert to
  static only at the nozzle exit (station 9) for exhaust velocity.

## Layout
- `turbojet/gas.py` — `FlowState`, dual-section `Gas` (cold/hot, `unified()`); the
  CPG closed-form / TPG NASA-integral property interface (`h/pr/T_from_*/cp_*_at`,
  hot methods carry `far`) and the `Gas.thermally_perfect()` (frozen products) and
  `Gas.reacting()` (composition-tracks-`f`) factories; `_products_composition(f)` +
  `_ReactingSection` (memoized per-`f` `_TPGSection`).
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle` in `h`/`pr`
  form (+ loss params, `ram_recovery(M0)`, the polytropic `e_c/e_t` knob; the Nozzle
  branches CPG/TPG — the velocity↔enthalpy trap). The `Burner` runs the implicit
  `f = g(f)` fixed point; `Turbine`/`Nozzle` hot-section calls thread `far`.
- `turbojet/engine.py` — chains components, solves the `Δh` + `η_m` shaft balance,
  scores performance (two thermal efficiencies + cascade check); freestream branches
  CPG/TPG.
- `tests/test_stations.py` — per-station rung-1 checks (turbine takes `delta_h`).
- `tests/test_validation.py` — rung-1 spec table (doubles as reduce-to-ideal).
- `tests/test_rung2.py` — reduce-to-ideal, directional, Mattingly Ex 7.1 anchor.
- `tests/test_polytropic.py` — rung-2b: reduce-to-ideal, polytropic⇄isentropic
  equivalence (1e-9), polytropic-native anchor, the `η_c<e<η_t` asymmetry.
- `tests/test_variable_cp.py` — rung-3: round-trip inverses, dual-section
  discriminating check, air-table + Çengel/Mattingly machinery anchors, gas-table effect.
- `tests/test_reacting.py` — rung-4: stoichiometry hand-check, implicit-solve
  direction + cross-datum burner, Mattingly Ex 6.3 products anchor, test-only McKinney
  cross-check, `f`-sweep directional; the implicit burner is a no-op on frozen gas.
- `main.py` — runs ideal vs real at one design point: tables + overlaid T–s diagram,
  plus the rung-2-frozen-`cp` vs rung-3-`cp(T)` table and the rung-4 frozen-vs-reacting
  + `f`-sweep table.

## Commands
- Run the model:  `python main.py`
- Run tests:      `pytest`  (or `python tests/test_rung2.py`, etc.)
- Install deps:   `pip install -r requirements.txt`  (matplotlib only)

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
