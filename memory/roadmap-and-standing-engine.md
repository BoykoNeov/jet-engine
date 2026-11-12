---
name: roadmap-and-standing-engine
description: "2026-11-12 carte-blanche review: docs/plans/roadmap.md is now the entry point for future work (tasks A1–A8 written for a lesser model, directions B1–B5 await the user's choice); the standing engine (Mach 0) shipped on the Design view only; process lessons from the session"
metadata:
  node_type: memory
  type: project
  originSessionId: 97dd3700-c972-4de6-8e1c-5ace8f6fcf10
  modified: 2026-11-12T09:31:01.341Z
---

**User, 2026-11-12:** "improve the project — design and concepts, current and future; fix bugs; if too big,
write plans for a lesser model. Carte blanche." What came of it:

- **`docs/plans/roadmap.md` is the entry point for future work.** Part 2 = ready tasks A1–A8 (experiments
  menu in the page, bind the experiments guide to the page, standing engine on Fly it/Controls/afterburner,
  plain-language notes for rungs 25–108, the charts page frozen at rung 29, an efficiency split that survives
  a convergent nozzle, page polish, two linter lines). Part 3 = directions B1–B5 (turbofan, real seconds,
  starting, a measured compressor map, a mission) — **the user has not chosen among them; ask, do not pick.**
  My stated recommendation there: A1 and A4 first, then A3 and B1.
- **Shipped:** root `README.md`; `docs/sandbox/EXPERIMENTS.md` (19 experiments, numbers read off the page,
  NOT yet gated — task A2); the standing engine on the Design view (sandbox plan § 48).

**Process lessons:**
- **A refusal can be another rung's control flow.** Mach 0 failed in `try_freestream` and `try_score`; rung 41
  uses both failures to skip its Mach 0 column. Fixing in place would move recorded results. The safe form is a
  twin (`*_or_static`) that handles the exact-zero case and otherwise calls the original, used only by callers
  that failed there before. Read a failing function's doc comment for "control flow" before touching it.
- **The gate for a new limit case: compare it with a near-limit run through the UNCHANGED path** (Mach 0
  against Mach 1e-6) — an independent check, not the new code agreeing with itself.
- **`window.sandbox.fly(patch)` resolves before its run lands** (entering the view queues a newer run that
  supersedes it). A probe must wait for `window.sandbox.last.fly` to match. Roadmap task A2 carries the fix.
- **In Controls results, `stall_lp`/`stall_hp` are the stall LINE (0.55), not a margin** — a probe that
  takes their minimum measures nothing.
- **A driver script for ad-hoc page probes is cheap** (headless Chrome, own profile and port, PID recorded,
  closed through its port): ~60 lines modelled on `rust/sandbox-wasm/browser.mjs`. Reading numbers off the
  real page beat reasoning about what it should show. See [[windows-tooling-file-hazards]] for the heredoc
  traps hit while writing it.

Related: [[sandbox-direction]], [[sandbox-mach]], [[instrument-fed-by-what-it-certifies]].
