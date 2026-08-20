---
name: rust-port-slice-u-step4
description: "Slice U step 4 — a gate that reads a key only by comparing it with ITSELF cannot see what the key is, and a suite whose thesis is invariance is entirely that shape"
metadata: 
  node_type: memory
  type: project
  originSessionId: 3688e397-f7fc-4957-8677-b3e66aa2c742
  modified: 2026-08-20T11:37:32.215Z
---

Slice U step 4 (rung 52's asymmetric lag: `LagRelief` + three readers + 15 gates) shipped
2026-08-20. 972 keys bit-exact on the first run; 294 lines added, ZERO deleted; 15/15 in 0.75 s
including all four gates Python marks slow, so no `#[ignore]` — decided on a measured cost.

**A GATE THAT READS A KEY ONLY BY COMPARING IT WITH ITSELF CANNOT SEE WHAT THE KEY IS.** Two
predictions said "caught" and both were wrong, for one reason. Swapping `g_at_cross` with
`required_at_cross` moves 56 keys and nothing notices; swapping `min_phi_hp_lag` with its bare
twin moves 56 keys and nothing notices. Both keys ARE read — one by a gate asserting they are
invariant across a rate sweep, the other by a gate asserting convergence across a step sweep. Both
are comparisons of a key against ITSELF AT ANOTHER CELL, and a defect applied uniformly moves
every cell together and leaves the comparison untouched.

**AND THE SUITE IS THAT SHAPE BY CONSTRUCTION, NOT BY ACCIDENT.** Rung 52's whole subject is that
the release constant moves nothing — the crossing, the credit, the engagement edge. A suite built
to prove nothing moves is maximally blind to everything being wrong by the same amount. This is
[[rust-port-slice-t-step3]]'s scale-invariance hole reached from the suite's THESIS instead of
from its readers, and it is the sharper form: the thesis predicts the blindness in advance.

**A ONE-SIDED BAR CANNOT SEE AN ERROR IN THE DIRECTION IT ALREADY ALLOWS.** Two more uncaught
injections were of that kind: a `> 0.4` ratio bar is cleared MORE comfortably when the denominator
loses a term, so dropping half of a max and dropping an interaction term both pass. Hold the
numerator and denominator as VALUES, never the ratio.

Also: BOTH traps registered in the eight-line crossing loop — the `Option<bool>` seed and the
dormant-point `continue` — move ZERO keys on every one of the 18 cells. Two guard clauses kept
purely under *COPY vs REDERIVATION*, with no evidence behind them until a manufactured trajectory
at step 5.

See `docs/plans/todo-rust-port.md` § 5.18 step 4. Related: [[rust-port-slice-u-step3]],
[[rust-port-slice-u-step5]].
