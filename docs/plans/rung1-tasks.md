# Rung 1 — Task Checklist

Mirrors the SPEC.md deliverables. Check off as each lands with passing numbers.

## Scaffolding (done)
- [x] git repo initialized
- [x] SPEC.md in place; concise CLAUDE.md pointer
- [x] Python package skeleton (`turbojet/`) with interfaces
- [x] Failing validation test encoding the expected-output table
- [x] Plan / context / tasks docs

## Physics (to do — derive before coding each)
- [ ] `FlowState`, `Gas`, and the five components, each with a one-line physical comment
- [x] Station 0 freestream (Tt0, pt0, V0) — worked example; `Engine.freestream`, green in `tests/test_stations.py`
- [x] Inlet (Tt2, pt2) — ideal full-recovery; green in `tests/test_stations.py`
- [x] Compressor (pt3, Tt3) + primary hand-check passes — green in `tests/test_stations.py`
- [x] Burner (pt4, f) — ideal, no loss; f=0.02304, mass grows by (1+f); green in `tests/test_stations.py`
- [x] Turbine — shaft balance (Tt5=1239.7 K, pt5=411.5 kPa); delta_Tt=(Tt3-Tt2)/(1+f); green in `tests/test_stations.py`
- [x] Nozzle — fully expanded (M9=2.033, T9=678.8 K, V9=1061.5 m/s); diverges its
      return type (NozzleExit: totals + statics); green in `tests/test_stations.py`
- [x] Engine chains components and solves the shaft balance — `Engine.run` owns the
      delta_Tt + shaft-closure assert; `build_turbojet` wires the five components
- [x] Performance: specific thrust=816.6, TSFC=2.821e-5, eta_th=0.4821, eta_p=0.4073,
      eta_o=0.2231 (note: eta_o != eta_th*eta_p — eta_th is Brayton, not propulsion)
- [x] Conservation assertions wired into components (run every time)
- [x] Validation test green to ~0.1% (`tests/test_validation.py`: both cases pass)

## Artifacts
- [x] Run prints the full station table (`main.py` → `print_station_table`)
- [x] T–s diagram (stations 0,2,3,4,5,9; isentropic legs vertical, p=const as curves) —
      `main.py` → `plot_ts_diagram`, written to `ts_diagram.png`. Closed Brayton loop:
      0/9 drawn STATIC so the heat-rejection leg (9→0 at p0) exists as the 2nd const-p curve
- [x] NOTES.md — plain-language explanation of each station + what the shaft balance buys
