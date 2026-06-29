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
- [ ] Turbine — shaft balance (Tt5, pt5)
- [ ] Nozzle — fully expanded (M9, T9, V9)
- [ ] Engine chains components and solves the shaft balance
- [ ] Performance: specific thrust, TSFC, eta_th, eta_p, eta_o
- [ ] Conservation assertions wired into components (run every time)
- [ ] Validation test green to ~0.1%

## Artifacts
- [ ] Run prints the full station table
- [ ] T–s diagram (stations 0,2,3,4,5,9; isentropic legs vertical, p=const as curves)
- [ ] NOTES.md — plain-language explanation of each station + what the shaft balance buys
