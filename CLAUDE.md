# Turbojet Cycle Simulator

A station-by-station model of a single-spool turbojet (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram. **Rung 1** is the ideal cycle;
**rung 2** adds real components (isentropic efficiencies + pressure losses) and a
dual-section (cold/hot) gas, anchored to a published textbook case.

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

## Read this first
- **`SPEC.md`** — rung-1 contract + thermodynamics handout: station equations,
  shaft balance, the frozen ideal assumptions, the validation case.
- **`docs/rung2-spec.md`** — rung-2 contract + handout: efficiencies, dual gas,
  the reworked asserts, the two thermal efficiencies, the verification gates.
- `docs/plans/` — living plan/tasks (rung 1 and rung 2), and
  `rung2-anchor-mattingly.md` (the verified Mattingly Example 7.1 anchor data).

## Working contract (from SPEC.md — these override convenience)
- **Derive before you code.** For each station, write the governing equation and
  a one-line physical justification (why it holds) *before* implementing it.
- **Show the work.** Every run prints the full station table (Tt, pt, …) so the
  numbers can be watched propagating.
- **Pure components.** Each component is `apply(state, gas) -> state` with no
  hidden state (Turbine and Nozzle diverge their signatures by design).
- **Conservation checks are assertions**, run on every execution (not as
  separate tests). See SPEC.md / docs/rung2-spec.md § Conservation checks.
- **Current scope (rung 2):** ideal + real components (isentropic `η_c/η_t`,
  pressure ratios `π_d/π_b/π_n`, `η_b`, `η_m`, dual cold/hot gas, specified exit
  pressure). Still deferred — keep the seams for them: polytropic efficiency as a
  first-class knob, variable `cp(T)`, off-design / component maps, a *choked*
  convergent nozzle, afterburner.
- **Stop and explain surprises.** If a number looks off, reason about the
  physics rather than silently moving on.

## Conventions
- **SI units throughout** (K, Pa, kg/s, m/s, J/kg). Convert kPa → Pa internally.
- The cycle runs in **total (stagnation)** quantities `Tt, pt`; convert to
  static only at the nozzle exit (station 9) for exhaust velocity.

## Layout
- `turbojet/gas.py` — `FlowState`, dual-section `Gas` (cold/hot, `unified()`).
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle` (+ loss
  params and the `ram_recovery(M0)` helper).
- `turbojet/engine.py` — chains components, solves the dual-cp + `η_m` shaft
  balance, scores performance (two thermal efficiencies + cascade check).
- `tests/test_stations.py` — per-station rung-1 checks.
- `tests/test_validation.py` — rung-1 spec table (doubles as reduce-to-ideal).
- `tests/test_rung2.py` — reduce-to-ideal, directional, Mattingly Ex 7.1 anchor.
- `main.py` — runs ideal vs real at one design point: tables + overlaid T–s diagram.

## Commands
- Run the model:  `python main.py`
- Run tests:      `pytest`  (or `python tests/test_rung2.py`, etc.)
- Install deps:   `pip install -r requirements.txt`  (matplotlib only)

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
