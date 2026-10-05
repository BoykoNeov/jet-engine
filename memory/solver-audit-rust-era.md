---
name: solver-audit-rust-era
description: "2026-10-05 Rust-era solver-tolerance re-audit — post-43 sites safe; the equilibrium Newton's -80 floor found and FIXED (floor → -300), with a rule-based oracle divergence in tests/common/eq_floor.rs"
metadata:
  node_type: memory
  type: project
  originSessionId: 68afa7be-e2e8-4518-b7c6-16fc219937d0
  modified: 2026-10-05T20:35:36.868Z
---

Re-audited every `did not converge` site added after rung 43 (`docs/plans/todo-solver-tolerance-audit.md`, "Rust-era re-audit" (a)–(e)). Joint-IC solves (66–74) safe by their 1e-12 exit / 1e-9 assert gap; rung 55's stacked secants measured safe on the reacting gas.

**Defect found and fixed (user chose "fix"):** the rung-6 equilibrium Newton measured its step BEFORE the `y >= -80` floor, so a species whose equilibrium lay below e^-80 spun the solve to its cap. Floor lowered to -300 in `gas.rs` — the first deliberate divergence from `python-final`. 4023/4023 passing solves bit-identical. It had been HIDING a true refusal (Tt3 > Tt4 at M0 = 2 → no f ≥ 0), which reached a panicking assert; `try_solve_equilibrium` now returns it as an `Abort`. The three affected oracle gates take ONE rule (`rust/tests/common/eq_floor.rs`), oracle files untouched.

**Why:** process lessons — (1) the plan's header said "open" while its body said CLOSED; I recommended off the header — read a plan to the end. (2) Check where a solver CLAMPS relative to its convergence test. (3) My first write-up claimed "the other species converged" without measuring — the damping read the same pre-floor step; the advisor caught it. (4) Removing a false failure can expose a TRUE one on a different (panicking) route — run the real callers, not just the kernel. (5) A drafted test license that nothing uses is a hole: measure it, then delete it.

**How to apply:** when changing a solver that Python-parity gates cover, encode the divergence as a checked RULE with invariants (counts conserved, everything else bit-exact), never by editing `rust/oracle/`. See [[instrument-fed-by-what-it-certifies]].
