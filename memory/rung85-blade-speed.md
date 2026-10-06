---
name: rung85-blade-speed
description: "Rung 85 (blade-speed walls) SHIPPED 2026-10-06 as knobs — process lessons: a self-comparing identity gate slipped in again; an extrapolation across shapes was confounded (the shapes differed in curvature and η, not only slope); the user's 'less rounding' became a knob"
metadata:
  node_type: memory
  type: project
  originSessionId: 4a42dd03-de77-467e-b8fb-896efb20308c
  modified: 2026-10-06T10:45:40.022Z
---

**Shipped 2026-10-06** — `rust/src/blade_speed.rs`, `rust/tests/rung85.rs`, panel
`print_blade_speed_table` (the first RUST-OWNED panel, D6), spec `docs/rung85-spec.md`, anchor
`docs/plans/rung85-anchor-blade-speed.md`. Physical headline is in CLAUDE.md's table.

**User decisions this session:** (1) "make it closer to reality, maybe don't round so much" →
the rounding knob `λ` (0 = blades slow down, 1 = blades at the wall, rows load lighter via a
design pre-swirl, folded into the map as `l' = (1+l)/r − 1`); (2) after scoring, the DROOP switch
(A `σ' = σ/r` default, B `σ' = σ`) — because `λ` reaches a held schedule ONLY through `σ`.

**Process lessons:**
- **A self-comparing gate slipped in AGAIN** ([[instrument-fed-by-what-it-certifies]]): the P8
  design-point identity fed the reading `(n, φ) = (1, 1)` by hand, so it compared one function
  with itself. The advisor caught it; the fix feeds the PLANT's matched design point. Ask "what
  supplies the value" for every identity gate, not just the comparison ones.
- **A cross-shape extrapolation was CONFOUNDED.** Q3 and A4 extrapolated rung 55's
  `tilted`-vs-`flow/press` bill gap "in the slope `l`" — but `tilted` also differs in curvature
  `σ` and η island, and the held speed turned out to carry NO `l` at all (holding incidence
  cancels it exactly). Before extrapolating a measured difference along one parameter, list
  EVERY parameter the two cases differ in.
- **"Settled by published numbers" can be settled for the wrong reason:** old-Q3 was moved to
  no-credit on an estimate of 1.43; it measured 1.287 and crossed by 0.09 %.
- **A crash behind a sandbox knob is unacceptable:** rung 53's schedule panics when unreached, so
  rung 85 drives the lumped plant through rung 55's one-row stack (proven identical) and checks
  it against rung 53's PUBLISHED numbers.
- The scoring grid took 2 s — measure cost before marking a run `#[ignore]`; ignored scoring
  runs leave the HIT/MISS lines unbacked by the gate.

**Next:** the web sandbox ([[sandbox-direction]]) — plan it first.
