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

## The options measured after the fix (2026-10-22), and the one chosen

Five slow requests (`../rung106/r95time_reqs.txt`), page = WebAssembly in node; 400-request census as above, against
the fixed build's replies (`diff.py A B`: identical / differing replies, outcome changes, worst relative change).

| Option | Page | Census |
|---|---|---|
| fix only (shipped, `2278f6d`) | 12.6–13.9 s | identical |
| B: each end's hold search starts at its own throttle | 0.3–1.7 s (two requests still 13 s) | 117 differ, worst 2.2e-8; one refusal's inner model message (a row 2.6e-10 K away) |
| C: the floor search without the low start's six retries | 2.7–3.8 s | 4 differ: two refusals' thrust at the floor (5th digit), two runs ≤ 5e-10 — a RULE change (partly undoes `c9a0306` at the floor) |
| B + C | 0.3–3.8 s | 119 differ, worst 2.2e-8, three refusals' words |

B's two slow requests fall back to the full floor search: one has the metal cap on (its bracket's low end is the
floor), one has no loaded steady point at its set throttle (670 K). **The user chose B, extended to those two cases**
(where a search starts, never what counts as running). `hold-from-throttle.patch` is that extension, WRITTEN BUT NOT
YET VERIFIED: `bh::try_hold_root_from` (start at the set throttle; step down if already past the target; any refusal
re-runs the full search) and `bh::try_metal_cap_below` (bracket from the hold root downward; floor only on a failed
step), wired at the Controls view's three hold searches and its metal cap, plus a rung 94 unit test (passes). Its
census and page timing were not run.

## Verified and shipped (2026-11-09)

`hold-from-throttle.patch` now holds the SHIPPED version (it is also the commit). Measured on it: B extended = 0.26 /
0.60 / 1.73 / 12.9 / 1.70 s on the page; census 149 differ, worst 3.9e-8, no run ↔ refusal flip, #227's bracketed
model message only (the row 2.6e-10 K away — option B's case); natively 191 s for the 400 (beside nothing else).
The metal-cap request is fixed; the 670 K one is not, and cannot be by any start: timed per end (a temporary
`eprintln` round each `hold_end`), its starting end is ~9.6 of ~10.6 s, and the step-up's first answering point (920 K)
is already past the target, so the root lies just above the floor and needs the floor's own bisection. The first
draft stepped up from 670 K (720, 770, …) — off the full search's grid, so its four failing solves were memo misses
and the fallback paid them again (8.8 s native vs the unpatched 7.7 s); the shipped step-up walks `lo + k·GRID`
(7.8 s). An earlier reading of solver counters blamed the march (42 000 HP-wall rescues): wrong — the counters do not
separate the hold ends from the march; per-end timing does. Potential follow-up: a coarser `T_TOL` (plan § 43).
