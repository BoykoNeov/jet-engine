---
name: solver-audit-rust-era
description: "2026-10-05 Rust-era solver-tolerance re-audit — all post-rung-43 sites safe; found the equilibrium Newton's -80 floor makes the solve unsolvable below ~500-580 K (OPEN, user decision)"
metadata:
  node_type: memory
  type: project
  originSessionId: 68afa7be-e2e8-4518-b7c6-16fc219937d0
  modified: 2026-10-05T19:39:51.644Z
---

Re-audited every `did not converge` site added after rung 43 (plan: `docs/plans/todo-solver-tolerance-audit.md`, "Rust-era re-audit"). Joint-IC solves (66–74) safe by their 1e-12 exit / 1e-9 assert gap; rung 55's stacked secants measured safe on the reacting gas (1.28M solves, 0 raises, wander ≤4.1e-13 vs 1e-11 bar).

**The finding: the rung-6 equilibrium Newton measures its step BEFORE the `y >= -80` floor**, so when an equilibrium species (H, or CO at tiny f) lies below e^-80 (T ≲ 460 K any f, ≲ 580 K as f→0) the step never shrinks and it raises at 200. A REPRESENTATION floor, not a noise floor — and monotone in T, so July's non-monotone discriminator is blind to it. Load-bearing: it drives rung 33's 440/420 K "below idle" rows. Fix would break Python parity at those points → **awaiting the user's decision.** The 600/650 K rung-33 raises are not yet attributed.

**Why:** process lessons — (1) the plan file's header said "open" while its body said CLOSED; I recommended the audit off the header. Read a plan to the end before recommending it. (2) Ask whether a solver CLAMPS between the step and the convergence test — a clamp can make the residual unreachable regardless of precision.

**How to apply:** when auditing a solver, check clamp/floor placement relative to the convergence measure, not only tolerance vs noise. Probes live in `W:\temp\claude\solver_audit\rust` (instrumented crate copy). See [[instrument-fed-by-what-it-certifies]].
