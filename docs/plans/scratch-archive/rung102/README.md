# Rung 102 pre-anchor probes (scratch crate `W:\temp\claude\rung102-probe`, against `884e60a`)

Each `*.rs` is a `[[bin]]` of `Cargo.toml.txt`; its output is the same-named `*.txt`. All are afterburners bolted on
BESIDE the shipped code by the probe, not rung 102's device.

- `design.rs` → `design.txt` (CPG/TPG/reacting, then `design_eq.txt` after the run was cut to reacting+equilibrium):
  G1/G2 — the design point at three flights, four gases.
- `anchor.rs` → `anchor.txt`: G3 — Mattingly § 8.4's reference point, the book's equations and the model.
- `fixed.rs` → `fixed.txt`: G4's core numbers. **Its "OPENED" line is void** — it runs `run(kd / kd)`, the dry engine
  again (a slip caught by review); the anchor treats the opened claim as a § 2 gate, not a given.
- `held.rs` → `held.txt`: G4's thrust and fuel. Its "OPENED x…" label prints `1/k` of the HELD fixed point, not the
  matched area; the opened thrust is computed at the dry core (correct by the effective-throat argument).
- `rayleigh.rs` → `rayleigh.txt`: G5 — thermal choke and the heat-addition loss, perfect gas.
- `twospool.rs` → `twospool.txt`: G6 — rung 39's STEADY two-shaft matcher (not the Controls march); panics at 2000 K.

Part A (after the anchor):
- `score_a.rs` → `score_a.txt`: H2–H6 at the anchor's § 3 cells (H1 is pinned in `rust/tests/rung102.rs`).
- `newton.rs`: where the rung-6 equilibrium Newton fails hugging stoichiometric (φ 0.999999 at 1500 K, 3.2 bar).
- `crashreq.rs` (seeded random Design-view requests, afterburner off/on) + `replay.rs` (request file → replies, a panic
  caught) → `census_a.txt`. The base side ran `replay.rs` in a second crate on a worktree of `884e60a`.

Part B (after Part A shipped; against the Part B code):
- `dry_b.rs` → `dry_b.txt`: the 16 scored (flight, `Tt4`) points DRY on all three maps, before any Part B code (anchor § 3b).
- `fly_b.rs` → `fly_b.txt`: cost (best of 9) and the two match gates at two flights × two maps × two `Tt7`; the
  instrument (the shipped solver, throat × k, iterated).
- `micro_b.rs` (the parts of a lit instant, µs), `micro_b2.rs` / `micro_b3.rs` (the trials-and-instants census on the
  solve and on the page's path) — what located each cost: the duct's bisections, the low-`π_t` band where the duct's
  entry chokes, the unchoked march, the shipped speed search's march-in.
- `fly_eq.rs` → `fly_eq.txt`: the reacting and equilibrium gases' cost.
- `score_b.rs` → `score_b.txt`: H7–H9 (80 cells) + `tilted` reported + the page-reply cost at the H9 cells.
- `nu_b.rs`: the held lit shaft speed over the unlit one, and the flame holders' thrust cost, over the 32 H7 cells.
- `call_b.rs`: the fly ops through `sandbox::call`.
- `crashfly.rs` (seeded random Fly-it requests, afterburner keys on/off) + `replay.rs` → `census_b.txt`, with the check
  requests' comparison against a worktree of `7de9419`.

Part C PROBE (after Part B shipped, before the addendum; a second crate `Cargo_c.toml.txt` against a throwaway worktree
of `23c5651` carrying the hook in `probe_c.diff` — a thread-local afterburner read by `unchoked_lp::try_lp_turbine`, a
light-off clock ticked in the march derivatives, `lp_exit` in the thrust tails, and `guard()` traps at unreached sites):
- `march_c.rs` → `march_c.txt`: the default slam not fitted / fitted unlit / lit (held ×1, ×1.15, *match*; `Tt7` 1800,
  2000; light-off 0.25 and 3.0) via a schedule wrapper.
- `march_c2.rs` → `march_c2.txt`: the clock in the derivatives; `settle`, `dt`, `redline`, `floor`, `govern`, `hot`,
  `unchoked` (the txt is that run; the source's later modes `gov1` (every point of the governed lit runs), `pm` →
  `pm_c.txt` (both stall margins) and the dry `dt` baseline / on-grid light-off were run after it, printed in the
  session).
- `sweep_c.rs` → `sweep_c.txt`: thrust against MAIN fuel at frozen speeds, unlit and lit (the governor's handle).
- `reach_c.rs` → `reach_c.txt`: the 108 Controls check requests not fitted / fitted unlit / lit held / lit *match*
  (light-off 0.24), with lit-trial counts and the `guard()` traps.
- `grid_dry.rs` → `grid_dry.txt`: the addendum's scored grid run FITTED-UNLIT only (no lit run on those cells).
