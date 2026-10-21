# Rung 106 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung106-probe`, deleted after the rung; its
manifest is `Cargo.toml.txt`). `lib.rs` is a PROTOTYPE built from public pieces only: rung 95's governor as the demand of
rung 47's lagged redline clip (a third state), the other legs instant off the demand, an applied fuel below the lean
wall held there; its `OPEN` switch hands in the open ramp instead of the governor. Every script printed REACHABILITY
only (anchor `docs/plans/rung106-anchor-governor-lag.md` § 0).

- `reach.rs` → `reach.txt` — G2/G3/G5: 336 runs (seven laws × six cases × two maps × four lags): run length, stops,
  the governor's clamps, the applied fuel at the lean wall, which leg held, bit-equality with rung 95 before the clip
  engages, cost against rung 95.
- `reach2.rs` → `reach2.txt` — (a) the flat-LP + floor stops are rung 95's own; (b) the fast ramp 0.16; (c) G4's
  2 520-run sweep for the lean wall (399 runs, all slowdowns starting above the redline).
- `check47.rs` → `check47.txt` — G1: the prototype with the open ramp IS rung 47's lagged route, bit for bit (four maps,
  four lags); and two hot-start slowdowns on rung 47's own route.
- `open_lean.rs` → `open_lean.txt` — G4's sweep with the open ramp as the demand (the advisor's check of my first claim,
  which rested on six runs): 0 at the wall; the 679 early stops are the prototype's missing solvable-ceiling fallback,
  none among G4's 399 configurations.
