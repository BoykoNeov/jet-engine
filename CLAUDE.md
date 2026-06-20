# Turbojet Cycle Simulator

A station-by-station model of an **ideal turbojet** (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram.

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

## Read this first
- **`SPEC.md`** is the contract and the thermodynamics handout. Read it before
  touching code — it has the station equations, the shaft balance, the frozen
  rung-1 assumptions, and the validation case. This file only summarizes.
- `docs/plans/` holds the living plan, context, and task checklist for rung 1.

## Working contract (from SPEC.md — these override convenience)
- **Derive before you code.** For each station, write the governing equation and
  a one-line physical justification (why it holds) *before* implementing it.
- **Show the work.** Every run prints the full station table (Tt, pt, …) so the
  numbers can be watched propagating.
- **Pure components.** Each component is `apply(state, gas) -> state` with no
  hidden state.
- **Conservation checks are assertions**, run on every execution (not as
  separate tests). See SPEC.md § Conservation checks.
- **Stay in rung-1 scope:** ideal cycle only — no component efficiencies,
  pressure losses, variable cp, or off-design. But design the seams so those
  attach later without a rewrite.
- **Stop and explain surprises.** If a number looks off, reason about the
  physics rather than silently moving on.

## Conventions
- **SI units throughout** (K, Pa, kg/s, m/s, J/kg). Convert kPa → Pa internally.
- The cycle runs in **total (stagnation)** quantities `Tt, pt`; convert to
  static only at the nozzle exit (station 9) for exhaust velocity.

## Layout
- `turbojet/gas.py` — `FlowState`, `Gas` (data; given in the spec).
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle`.
- `turbojet/engine.py` — chains components, solves the shaft balance, performance.
- `tests/test_validation.py` — encodes the spec's expected-output table.
- `main.py` — runs the validation case: station table + T–s diagram.

## Commands
- Run the model:  `python main.py`
- Run tests:      `python tests/test_validation.py`  (or `pytest`)
- Install deps:   `pip install -r requirements.txt`  (matplotlib only)

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
