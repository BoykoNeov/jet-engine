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

After the build (the SHIPPED device, `rust/src/governor_lag.rs`):
- `score.rs` → `score.txt` — the spec's § 3 table (H1, H1b, H2, H3, H4, H5 — every cell), the reported reliefs, the
  burner-temperature hold to 1600 K, H4 past the grid (where it flips), the thrust's first peak and sag on four maps
  against the instant limiter, the unlimited governor and rung 47's open ramp, and the cost.
- `crash.rs` → `crash_first.txt` — the crash map (1 000 requests through `sandbox::call`, seed 12, every one governed with
  the redline and its lag): no panic; ONE minimum-fuel run with a start below the redline (#896, the metal cap holding
  the start while the governor's request was above the redline) — the user then refused that start too. `crash.txt` is
  its re-run after that refusal.
- `slow.rs` → `slow.txt` — the crash map's five slowest requests timed with the lag on and off (the same: not the lag's).
  The spec's 14.6–17.4 s are a first run on a quiet machine; `slow.txt` is a second one taken while the gate ran
  (22–28 s, the same on/off pattern).
- `one.rs` — prints one request's reply (used on #896).
- `r95time.rs` (+ `r95time_reqs.txt`) → `r95_vs_today.txt` — the five slowest requests, stripped of keys rung 95 did not
  know, timed on a worktree at rung 95's `096367d` and on today's code, back to back: 0.2–1.8 s then, 21–37 s now — a
  regression of rung 95's route, not this rung's (spec § 4).
