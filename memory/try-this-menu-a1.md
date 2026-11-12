---
name: try-this-menu-a1
description: "Roadmap A1 shipped 2026-11-12: the sandbox's per-view 'Try this' menu (29 experiments). Process lessons: a reset-pin-patch menu must await each run landing (settled()); write an experiment's text only after measuring its run against the pin it is compared with"
metadata:
  node_type: memory
  type: project
  originSessionId: 688f24b4-7dc4-4660-8fee-113193ab630d
  modified: 2026-11-12T15:24:15.607Z
---

**Shipped 2026-11-12** (commits `1f02569`, wording fix `2bc73cc`): a "Try this" menu on Design, Fly it, Controls,
Burner and Cooling (not Blades). Each entry resets the view, runs and pins the opening, applies a patch and runs it.
The 29 entries live in `docs/sandbox/template.html` between `const EXPERIMENTS =` and `// end EXPERIMENTS`.
Tests: `rust/tests/sandbox_page.rs` runs every entry natively; `rust/sandbox-wasm/browser.mjs` step 17c drives the
menu in Chrome.

**Lessons.**
- `change()` queued `runNow` on `requestAnimationFrame`, so `window.sandbox.fly()` resolved before its run had
  landed. A menu that resets, pins and patches in sequence must await each run: hence `settled()`. Look for that
  gap in any new caller that chains page actions.
- The advisor caught an entry whose text described the run against the wrong baseline: rich burning with slow
  dilution, read against the PINNED opening, still shows less NO. Write an entry's text only after measuring the run
  against the pin the page actually compares it with (a wasm probe on the built page is enough).
- Known gate flake: the blades slow-sweep kill race (`a_knob_move_stops_a_slow_sweep_and_the_new_one_finishes`),
  more likely under load. Rerun `-Only sandbox-browser` before calling it a failure.

Related: [[roadmap-and-standing-engine]], [[plain-notes-a4]].
