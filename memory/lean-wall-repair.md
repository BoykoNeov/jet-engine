---
name: lean-wall-repair
description: "2026-10-20 solver repair (sandbox plan § 40): the fuel closures' lean wall 0.004 follows the run on Controls (half the leaner steady end, TwoSpoolCore::lean_wall, hidden `lean: old`) — lessons: a stop named from a missing reading; a 'only failures change' proof holds per call, not per caller"
metadata:
  node_type: memory
  type: project
  originSessionId: 4f1b4d4e-247c-4d0e-a03f-b489a896eded
  modified: 2026-10-20T01:41:16.588Z
---

Solver repair (not a rung), closing § 39's open item. User chose, in two plain questions: RELAX the wall in the pages
(not refuse lean starts), and TIE IT TO THE RUN (half the leaner steady end's mixture when under 0.004) rather than
remove it. `TwoSpoolCore::lean_wall: Cell<Option<f64>>`, `None` = old code; read by rungs 43/62/90/91's closures' `hi0`;
set in `sandbox_controls::build` after the ends; copied in `stator.rs::at_setting`. Gate `rust/tests/lean_wall.rs`.

**Lesson 1 — a stop kind named for a MISSING reading is a guess.** § 39 called first-step stops "no burner state"
because, with no recorded point, the classifier had no mixture to read. 1 553 of 1 691 were starts leaner than the
wall. Fall back to the start's own state (`run_mixture`) before naming a cause.

**Lesson 2 — "lowering the wall only changes calls that failed" is true per CALL, not per CALLER.** The scan argument
held (seed 36's 58 changes all bit-equal on their common prefix), but rung 101's map had 18 governed runs that
completed both times and DIFFER: the governor/limiter fuel searches treat a failed trial as information, and 15 of them
had been resting AT the old wall. Run the crash map that draws the callers (governor, limiters), not only the closure.

**Lesson 3 — the advisor's "measure what the fixed step turns into" was right:** at the design flight the low starts
now stop RICH after 11–21 steps (no acceleration schedule); the whole-slam floor did not move there. Measure the
product's whole action after a fix, before writing the question to the user (see [[idle-floor]] lesson 1c).

Also: a `cmd start` on a not-yet-built exe hung the shell (Windows dialog) — chain the run on the build's success.
Related: [[idle-floor]], [[low-start-repair]], [[hp-wall-repair]].
