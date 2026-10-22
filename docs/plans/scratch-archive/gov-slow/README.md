# The slow governed runs with the HP-shaft generator — scratch instruments (2026-10-22)

`docs/plans/sandbox-plan.md` § 43's open item. Run from scratch crates depending on `turbojet` by path
(`W:\temp\claude\gov-slow\probe` on the commit before the fix, `probe2` on a copy with the fix; deleted afterwards; the
manifest is `Cargo.toml.txt`, `debug = true` for the profiler).

- `bytes.rs` — the byte census: 400 random Controls requests (seed 31) weighted to rung 94's held ends (generator on,
  thrust or HP-speed holds), across the governor, metal cap, inferred readings and carried tables, flights (Mach
  0.05–0.9), turbine laws, the HP vane, the afterburner and the levers; through `sandbox::call`, 8 threads, written
  `i<TAB>request<TAB>reply` in order.
- `run_census.sh` — both builds on the same requests, then `cmp`: **IDENTICAL** (15 152 476 bytes each, 264 ran;
  1 598 s before, 883 s after, below normal beside other work).
- `wasm_time.mjs` — times each request of a file through a `sandbox_wasm.wasm` in node (`node wasm_time.mjs WASM
  REQS`). The five slow requests of rung 106's crash map (`../rung106/r95time_reqs.txt`) took 13.5–17.1 s each after
  the fix.

The profile (samply, symbols through `dbghelp`) put ~82% of a slow request in failing equilibria below the loaded
engine's floor, each retried from the low-start repair's six starts.
