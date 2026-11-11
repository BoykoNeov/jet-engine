# Sandbox plan § 45 — a fast cut beside the governor: scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\gov-fast-attack`, deleted after the work; its
manifest is `Cargo.toml.txt`, its library `lib.rs`: the shipped `governor_release::march` at any step, and the gap on
the 0.02 grid).

**Before the code (the measurements the user chose from):**
- `sweep.rs` → `sweep.txt` — how fine: 63 settings (seven laws at rung 106's edges × schedule / floor / both + redline
  × design / thin / dense) × attack {0.005, 0.01, 0.02, 0.05, 0.1} × release {0.1, 0.4, 0.8}, steps 0.02/k for
  k = 1…32, each k against 4k. Clean at k = 2 (attack ≥ 0.05), 4 (≥ 0.01), 8 (0.005). The probe's k = 1 march equals
  the view's own in all 63 (`same=true`).
- `sag.rs` → `sag.txt` — the sag the page would show, steps k = 1, 4, 16, 32, read on the 0.02 grid and on every point:
  converged at k = 4; rung 107's fine-step numbers confirmed; a fast cut's sag ~20 % shallow on the 0.02 grid.
- `timing.rs` → `timing.txt` — cost: rung 107's crash-map requests at the old floors, one thread; the slowest re-timed
  at k = 2 / 4 / 8 with the attack at 0.05 / 0.02 / 0.005.

**After the code (the frozen bars):**
- `sweep2.rs` → `sweep2.txt` — B2 on the shipped route: 63 settings × attack {0.02 … 0.19} × release {0.1, 0.4, 1.6},
  the page's run (0.005) against 0.00125: 1 323 runs, every one to the end, worst 4.4e−5 (bar 1e−4).
- `crash.rs` → `crash.txt` — B5: rung 107's crash map (seed 13, attack 0.02–0.5) through `sandbox::call` on the new
  code: 0 panics, 421 admitted (270 before), 60 stops, every one the floor out of reach (0 "other"), 0 re-attacks,
  0 minimum-fuel holds; slowest 5.75 s natively.
- `wasmtime.mjs` → `wasmtime.txt` — the slowest requests (`slow_reqs.txt`) and the page's default governed release
  (`default_reqs.txt`) in WebAssembly (Node's V8, the page's call path): slowest admitted 11.2 s; the default at a
  0.02 cut 1.9 s against 0.35 s at 0.2.
- `stops.rs` → `stops.txt` — B3's second half (added after review): of the crash map's requests, 151 ran a fast cut and
  21 stopped (7 part-way, 14 at the first point); every stop's diagnosis re-ran its last fine step, found the failure
  and reported the last point's time. #470 is pinned in `rust/tests/gov_fast_attack.rs`.
