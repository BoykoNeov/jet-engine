# Rung 1 — Implementation Plan (Ideal Turbojet)

Status: **scaffolding complete; physics not yet implemented.**
Source of truth: [`../../SPEC.md`](../../SPEC.md) — read it first.

## Goal
Build the ideal (everything-isentropic, cold-air-standard) turbojet cycle so the
gas state at every station, the thrust, and the efficiencies fall out and can be
hand-checked against the spec's validation table. The point is *understanding*,
so **derive each station before coding it.**

## Approach: explore → derive → code → verify, one station at a time
Work the stations in flow order (0 → 2 → 3 → 4 → 5 → 9). For each:
1. Write the governing equation + a one-line physical justification (why it holds).
2. Implement the component body (pure `apply(state, gas) -> state`).
3. Wire the relevant conservation assertion(s) into the body.
4. Re-run the validation test; confirm the new station's numbers match.

### Station order and where each lives
| Step | Station | Where | Key idea |
|------|---------|-------|----------|
| 1 | 0 freestream | `engine.Engine.freestream` | ambient + M0 → Tt0, pt0, V0 |
| 2 | 2 inlet | `components.Inlet` | ideal: totals preserved |
| 3 | 3 compressor | `components.Compressor` | isentropic at pi_c |
| 4 | 4 burner | `components.Burner` | energy balance → fuel-air ratio f |
| 5 | 5 turbine | `components.Turbine` | **shaft balance** (the keystone) |
| 6 | 9 nozzle | `components.Nozzle` | fully expanded → V9 (static) |
| 7 | performance | `engine.Engine.run` | thrust, TSFC, efficiencies |
| 8 | T–s diagram | `main.plot_ts_diagram` | the payoff artifact |
| 9 | NOTES.md | repo root | plain-language explanation |

## Verification gates
- **Primary hand-check:** `1 - Tt2/Tt3 == 1 - 1/pi_c^g == 0.4821`. If these
  disagree, the compression leg is buggy — fix before trusting anything else.
- Full validation table matches to ~0.1% (`tests/test_validation.py`).
- Conservation assertions pass on every run (mass, shaft energy, burner energy,
  isentropic legs).

## Done when
Every box in `rung1-tasks.md` is checked and the validation test is green.

## Explicitly out of scope (rung 2+)
Component efficiencies, pressure losses, variable cp(T), off-design / component
maps, afterburner, CD-nozzle CFD. Design the seams for them; do not build them.
