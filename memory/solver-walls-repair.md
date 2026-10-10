---
name: solver-walls-repair
description: "2026-10-10 repair of three \"model gives up though an answer exists\" walls (rung 34 low flow wall, rung 31 turbine wall, rung 43 schedule check) — process lessons"
metadata:
  node_type: memory
  type: project
  originSessionId: cba02fc6-3b0c-4526-a967-0970652034d7
  modified: 2026-10-10T10:39:17.971Z
---

Repaired 2026-10-10 (sandbox plan § 14; CLAUDE.md's three OPEN lines → one REPAIRED line). Lessons:

- **"Only failing cases change" is false when the failure is CONTROL FLOW.** Marching rung 34's
  closure wall in everywhere moved 1 416 of combustor_oracle's 2 066 keys by ~1e-11: the steady
  speed search walks its bracket in past FAILING trials, so a trial that now succeeds moves the
  bracket and the converged root. Scope such a fix to where a failure ENDS a run (the time march).
  The advisor predicted this before any code; check every caller that treats Err as a signal.
- **A "solver artefact" stop label can hide real physics.** Once the artefact was removed, 8 of
  the remaining stops were genuine flame-outs (commanded Tt4 ≤ compressor exit, f ≈ 1e-6). Measure
  what is LEFT after a repair, then name it.
- **A continuation switched on globally crashed a shipped diagnostic.** Rung 58's leg_residual
  reads the scheduled-fuel instant at every recorded point, so it assumed the march STOPPED where
  that instant fails. Ship such a fix as a default-OFF knob (the reduce spine) and let the sandbox
  turn it on.
- **Licence by a PHYSICAL predicate, then check it equals the moved set.** tests/common/
  turbine_wall.rs names the wall cells by "0.02 trial below the 150 K tables", and the named set
  equals the 12 moved cells exactly. Compare against the UNCHANGED code (a git worktree), not only
  against Python: the brk/ keys that differed from Python were the old eq_floor licence, not mine.

Related: [[sandbox-slice4a]], [[sandbox-slice4b]], [[solver-audit-rust-era]].
