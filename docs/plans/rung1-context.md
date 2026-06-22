# Rung 1 — Context (key files, decisions, validation data)

## Key files
- `SPEC.md` — the contract + thermodynamics handout (authoritative).
- `turbojet/gas.py` — `FlowState`, `Gas` (data; implemented).
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle` (stubs).
- `turbojet/engine.py` — `Engine`, `build_turbojet`, result/performance types (stubs).
- `tests/test_validation.py` — encodes the expected-output table (red until done).
- `main.py` — station-table printer + T–s diagram entry point.

## Decisions made during scaffolding
- Spec kept as `SPEC.md` (not folded into CLAUDE.md — too large; the best
  practice is to point, not embed). CLAUDE.md is a concise pointer.
- Components are pure `apply(state, gas) -> state`. The shaft balance is owned by
  the *engine*, not the turbine: `Engine.run` computes the required ΔTt and passes
  it to `Turbine.apply(state, gas, delta_Tt)`. This keeps every component pure and
  puts the coupling equation where it can be read (per spec § Architecture). See
  the resolved design note below.
- `FlowState` carries only totals (`Tt, pt, mdot, far`). Static exit quantities
  (`M9, T9, V9`) are surfaced on `EngineResult`, since `FlowState` is totals-only.
- The validation test is dependency-free (`python tests/test_validation.py`) and
  also pytest-discoverable.

## Design decision — turbine↔compressor coupling (resolved)
**Chosen: loose coupling, engine-mediated (design A).** `Engine.run` computes
`delta_Tt = (Tt3 - Tt2)/(1 + f)` from the states it already holds and passes it to
`Turbine.apply(state, gas, delta_Tt)`. The turbine takes no constructor load.

Why, over the alternatives:
- *Tight* (`Turbine(compressor)`): the turbine at station 4 doesn't have Tt2, so it
  would need the compressor to remember its in/out states — breaking purity
  (contract #3) — or need Tt2 plumbed in anyway, saving nothing.
- *Loose, config-per-run* (build `Turbine(delta_Tt=…)` inside `run`): keeps a uniform
  `apply(state, gas)` signature, but its only payoff is a generic component loop —
  which we reject because it would bury the keystone. Leaves a half-built turbine.
- *Loose, call-argument* (chosen): one diverging signature, zero indirection, the
  coupling equation visible in `run`. The divergence *documents* that the turbine is
  shaft-locked.

The shaft balance must stay an explicit, named, asserted step in `run` (not a generic
loop) so the coupling is visible (SPEC.md § The shaft balance, line 159).

## Validation case (from SPEC.md — do not re-derive, just match to ~0.1%)
**Inputs:** T0=250 K, p0=50 kPa, M0=0.85, pi_c=10, Tt4=1500 K,
gamma=1.4, cp=1004, R=287, hPR=42.8 MJ/kg.

**Expected:**
Tt0=Tt2=286.1 K; pt0=pt2=80.19 kPa; Tt3=552.4 K; pt3=pt4=801.9 kPa;
f=0.02304; Tt5=Tt9=1239.7 K; pt5=pt9=411.5 kPa; M9=2.033; T9=678.8 K;
V9=1061.6 m/s; V0=269.4 m/s; specific thrust=816.6 N·s/kg;
TSFC=2.821e-5 kg/(N·s); eta_th=0.4821; eta_p=0.4073; eta_o=0.2231.

## Cross-validation references (for later rungs)
NASA EngineSim (dry-turbojet config), pyCycle (`example_cycles`), Mattingly
*Elements of Propulsion* worked examples.
