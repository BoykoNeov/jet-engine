---
name: small-tasks-a2-a7-a8
description: "2026-11-19: roadmap A2 (experiments guide held to the model), A7 (page polish: view buttons, folded explanations, share link, emptied boxes) and A8 (two clippy lines) DONE; process lessons from them"
metadata:
  node_type: memory
  type: project
  originSessionId: c17a0663-6500-4475-9d5b-85b0a7331fb0
  modified: 2026-11-19T07:50:57.887Z
---

**User, 2026-11-19: "do all small tasks"** — A2, A7, A8 of `docs/plans/roadmap.md`, all DONE that day. Part 2 of
the roadmap is now empty; only B1–B5 remain, and those are the user's choice (ask, do not pick).

**Process lessons:**
- **"Read off the page" was not true of every number.** The guide's Controls numbers quoted *time to 90 % of the
  thrust change* (the page shows 95 %) and the *final steady* thrust (the page's "thrust at the end" is the run's
  last point). The model had not moved; the readings were never the page's. When a number disagrees, find the
  page's own reading (`template.html`'s row/tile code) before deciding drift vs mapping vs guide error.
- **A bar at the rounding itself (half a unit) is brittle by construction** — measured: 31 guide numbers used
  >80 % of it, one all of it. The user's side note flagged it; one unit of the last digit shipped, after
  measuring margin use (cf. [[needless-precision]]). Measure the margin distribution before choosing a bar.
- **The guide is the expected side**: the test parses the Markdown tables and sentence templates, so no number is
  copied into the test ([[instrument-fed-by-what-it-certifies]]); a census (every number checked or exempt with
  a reason) is what catches an unmapped row ([[rust-port-oracle-cannot-see-a-missing-gate]]). Red-check each kind.
- **Headless Chrome pages never have focus: blur/focusout never fire.** `Emulation.setFocusEmulationEnabled`
  fixes it. A hidden element cannot take focus either, so a probe that picks inputs must filter to visible ones.
- **`history.replaceState` works on file://** in Chrome; `navigator.clipboard.writeText` is refused headless —
  ship a select-to-copy fallback.
- **A range slider cannot stand in for its number box's value**: it snaps to its step (altitude 5579 → 5600).
- A side effect found while fixing item 4: `+$(box).value` turns an emptied box into 0 — grep for `= +$(` when
  touching number inputs.

Related: [[roadmap-and-standing-engine]], [[try-this-menu-a1]], [[sandbox-direction]].
