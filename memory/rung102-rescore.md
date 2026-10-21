---
name: rung102-rescore
description: "2026-10-21 confirmation/correction (not a rung): rung 102 Part C's 360 cells re-scored under rung 104's LP-turbine law (rung 104's last seam) — 235–237 vs the pin's 335 reproduced line for line; end-of-run stall margins flip; the Controls page's afterburner note still carried the pin's numbers — lessons: when a rung changes a DEFAULT, grep the page prose for the old numbers; check a page sentence against the page's OWN default request; a control law reproducing the archive line for line is what licenses reading every difference as the law"
metadata:
  node_type: memory
  type: project
  originSessionId: e952446d-b2f0-42cc-90b2-38d0317b3dd8
  modified: 2026-10-21T15:03:08.686Z
---

Confirmation/correction sweep (not a rung), 2026-10-21, after [[rung105-flight-wiring]]. Spec: `docs/rung102-spec.md`
§ Part C re-scored. Archive: `docs/plans/scratch-archive/rung102-rescore/`. User said "what is next" → "go" on my
recommendation (item 1); no scope decisions were needed.

**Lesson 1 — when a rung changes a DEFAULT, grep the page prose for the superseded rung's numbers.** Rung 104 made the
ellipse the default and corrected rung 102's spec and tests, but the Controls page's afterburner note (template.html,
`ctl-ab-note`) and the sandbox README kept "the LP slows by about a third … the HP spins past its design speed" — the
pin's picture — for a day. No test reads prose. Search the template and README for the old claim's words, not only
the spec.

**Lesson 2 — check a page sentence against the page's own default request.** "The LP compressor dips toward stall right
after light-off" was never true of the page's default (light-off 0.5 τ sits inside the slam's own dip: the minimum after
it EQUALS unlit's, on every law, the pin too); it was true of the scored grid's late light-offs. A `page_default.rs`
probe found it in one run.

**Lesson 3 — the control arm makes the differences readable.** Re-running the archived scorer with the law as an argv,
the `pin` arm reproduced `score_c.txt` and `tt4hold.txt` line for line (the advisor had warned rungs 103 / sandbox § 40
might move C10/C12 — they did not). Only then is every other-law difference the law's. Keep a diff script
(`cmp.py`) in the archive beside the outputs.

Also: the PowerShell `Start-Process` launch of a page test ran at NORMAL priority — use the Bash `cmd //v:on //c "start
/belownormal …"` form; a path with a space needs the PowerShell route only with `-PriorityClass` set after launch, which
misses children ([[run-tests-below-normal]]).
