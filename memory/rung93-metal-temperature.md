---
name: rung93-metal-temperature
description: "Rung 93 (metal temperature, Gauntner 1980; fixed vs fresh engine) process lessons — a screenshot is not a measurement of the configuration you name; derive every registered sign; check a published trend transfers to THIS cycle type"
metadata:
  node_type: memory
  type: project
  originSessionId: 76bf8353-8e43-4b1e-a44d-8d50c63caa37
  modified: 2026-10-16T05:37:16.578Z
---

Rung 93 (2026-10-16): Gauntner's NASA TM-81453 correlation on rung 91's vane/blade streams; framing B (fixed
engine, `FixedEngine`) and A (fresh cooled design), the user's switch; new sandbox view *Cool the turbine* + an
opt-in slam metal readout. Anchor `e2a6c43` before any metal code; M1–M6 all HIT. Scope decisions:
[[rung93-94-decisions]].

**Lesson 1 — a screenshot is not a measurement of the configuration you name.** The browser test's screenshot
showed the slam's blade metal crossing its limit, and I wrote "on the page's default slam … past its limit" into
the spec. The test had left a bleed lever on; natively, the TRUE default slam peaks 9 K INSIDE the limit.
**How to apply:** before quoting a number off a screenshot, re-run exactly the named settings natively (here as a
pinned test) — the page's state is whatever the previous test step left.

**Lesson 2 — derive every registered sign.** M5 ("cooling's fuel bill grows") had no model behind it; the advisor
saw it was backwards (burner air falls by the whole share, thrust by ~0.5 of it). Re-registered before the freeze;
it then HIT. Same family as [[rung90-customer-bleed]]'s guessed bar.

**Lesson 3 — check a published trend transfers to this cycle.** "Efficiency peaks at a moderate Tt4 once cooling
is counted" is a POWER-PLANT result; a turbojet's TSFC rises with Tt4 even uncooled, and the model's KE-based
`η_thermal` is meaningless on a choked nozzle (η_p > 1). Score a jet on F/ṁ and TSFC.

Also: two page notes I drafted generalised past what was measured (maps "do not enter" — true for 2 of 4 shapes);
both softened before shipping. A long Rust file via bash heredoc failed again — use Write ([[windows-tooling-file-hazards]]).
