---
name: gov-lag-step
description: "2026-11-11 plan § 47 — rung 106's governed lag moved to the 0.005 step; a step check must read the points the page DRAWS, over the whole slider"
metadata:
  node_type: memory
  type: project
  originSessionId: 5680c8ab-239f-4159-9ba1-e34491b7f4fc
  modified: 2026-11-11T19:33:30.598Z
---

Plan § 47 (2026-11-11, SHIPPED): rung 106's single lag beside the governor now runs on DS/4 (`march_ds`), the user's
choice "every governed lag run" (one rule with rung 108). Page cost ~4× (default 1.6 s, slowest tried 9.4 s).

Process lessons:
- My first step check compared the fine run only on the 0.02 grid (`gap(x, 4, y, 16)`); the page draws EVERY fine
  point, so the bar must be read there (`gap(x, 1, y, 4)`). The advisor caught it before I offered options.
- Roughness here is scattered across the knob, not a threshold (rung 106 § 4): sweep the WHOLE slider (0.05 grid), not
  four values; print 3 sig figs so a "1.0e-4" can't hide which side of the bar it is.
- A changed step breaks every "reduces to the predecessor's PAGE run" test even where the new effect never fires —
  restate it against the predecessor's march at the same step, and tell the user in plain words.
- Model-level tests that march at DS but compare length to `expected_points()` break when the page's step changes —
  count at the march's own step.
- cmd `start /belownormal` + a manifest path with a space: neither `^ ` nor `\"…\"` survives from Bash; write a `.cmd`
  file under W:\temp\claude and start that. See [[windows-tooling-file-hazards]].

Related: [[sandbox-direction]], [[needless-precision]].
