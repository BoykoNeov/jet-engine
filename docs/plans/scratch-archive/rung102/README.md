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
