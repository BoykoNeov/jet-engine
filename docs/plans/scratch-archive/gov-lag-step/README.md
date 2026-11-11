# Plan § 47 — rung 106's lag beside the governor on the finer step: scratch scripts and outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung106-rough`, deleted after; its manifest
is `Cargo.toml.txt`, the scripts were `src/lib.rs` and `src/bin/*.rs`). Every script drives the SHIPPED
`governor_lag::march` as the Controls view calls it (`lib.rs::lag_march`), with the step as an argument.

- `sweep.rs` → `sweep.txt` — the 63 governed settings × lag {0.1, 0.2, 0.4, 0.8} at steps 0.02/k, k = 1, 2, 4, 8, 16;
  gaps k=1 vs 4, k=2 vs 8, k=4 vs 16 on the 0.02 grid only (times are 8 runs at once — not quoted).
- `lags.rs` → `lags.txt` — the floor and schedule cases × the lag 0.1–1.0 every 0.05, read on EVERY point of the checked
  run: k=2 vs 8, k=4 vs 16 (the advisor's correction to `sweep.rs`, which read only the 0.02 grid).
- `wasmtime.mjs` + `reqs.txt` → `wasmtime.txt` — page cost in WebAssembly under Node, before and after.
  Usage: `node wasmtime.mjs <sandbox_wasm.wasm> reqs.txt`.
