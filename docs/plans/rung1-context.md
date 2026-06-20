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
- Components are pure `apply(state, gas) -> state`. The turbine holds a
  reference to the compressor it drives so the shaft balance stays explicit
  (per spec § Architecture). *How* the work is communicated is left open as a
  deliberate design exercise.
- `FlowState` carries only totals (`Tt, pt, mdot, far`). Static exit quantities
  (`M9, T9, V9`) are surfaced on `EngineResult`, since `FlowState` is totals-only.
- The validation test is dependency-free (`python tests/test_validation.py`) and
  also pytest-discoverable.

## Open design question (left for the implementer)
How the turbine receives the compressor's work / required ΔTt — e.g. store the
compressor's in/out states on the object, or have `build_turbojet` pass the
ΔTt directly. Keep the coupling explicit either way (SPEC.md § The shaft balance).

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
