---
name: rung97-unchoked-lp
description: "Rung 97 (two-shaft unchoked nozzle, transient) process lessons — a flag the solver sets is not a guard; a boundary-continuity gate is tautological when one helper computes both sides; list 'something else' among expected outcomes"
metadata:
  node_type: memory
  type: project
  originSessionId: f1e86d85-6f2a-4ab4-9593-9fd477e71319
  modified: 2026-10-18T04:37:00.829Z
---

Rung 97 (2026-10-18): `unchoked_lp.rs::try_lp_turbine` (rung 38's pin while the nozzle stays choked, else the subsonic
root `A45 supply = A8·ρ9·V9`), wired into all 6 transient LP-turbine sites (tails + power hooks); `TwoSpoolCore::lp_pinned`
= the old model, scoring only. Anchor `36b4908`. H1/H2 HIT 6/6 (2 vacuous). Steady matchers still refuse. Rung 95/96
outputs re-ran identical. Follows [[rung97-decisions]]; the Mach knob and rung 98 (δ/θ) come next.

**Lesson 1 — a FLAG is not a GUARD.** Rung 38's steady matcher refused an unchoked nozzle; the transient on the same
equations only set `Branch::Subsonic` and carried on with void numbers, and the page ignored the flag. Found only by a
probe for another feature. When a family reuses a sibling's equations, check the sibling's SCOPE GUARD made the trip too.

**Lesson 2 — a continuity gate at a dispatch boundary can be tautological.** With one helper computing both the pin
and the root, the boundary gap was 5e−14 on every variant — but `pt45` cancels from both, so a wrong `pt45` would never
jump. The non-tautological gate used a quantity the helper never reads: nozzle mass flow vs the engine's delivered gas
(≤ 7.6e−14). It was the red check's strongest failure.

**Lesson 3 — an "expected outcome" list needs a third slot.** The anchor said § 27's refusal "now runs or fails with the
new words"; it went off-map instead (the slower LP shaft). Write "or another refusal, stated" when forecasting a
changed failure.

**Tooling:** a bash loop building `cmd //c "start … release\\$b.exe"` left a LITERAL `$b`; `start /wait` then waited
forever on a missing exe (looked like a slow run). Loop over exes in PowerShell `Start-Process`, not bash→cmd. And I
typed `python -` once more (the Store stub hang) — see [[windows-tooling-file-hazards]].

**Why:** these are the traps that let a void number ship or a gate certify itself.
**How to apply:** any new branch in a shared solver — census who reaches it (a logger, one write per record), gate with
a conservation law the new code does not compute, and re-run neighbouring rungs' archived scoring scripts.

**Final review (advisor), 2026-10-18:** (a) compare a crash map against the code JUST BEFORE the change, back to back in
a worktree, request by request — the archived baseline predated an unrelated words commit; (b) the clean compare found
the pin had INVENTED steady points (HP generator, low throttle, warm air): a power-balance scan proved the branch has
none, so the new refusals are the model's, not the Newton's; (c) I wrote "rung 96's G2 now fails" without measuring —
it SURVIVES (subsonic push also depends on pt9 alone); G3's straight line is what breaks. Measure every "now fails".
