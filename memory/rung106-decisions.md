---
name: rung106-decisions
description: "User scope decisions 2026-10-21 for rung 106 (rung 95's seam: the governor on the lagged marchers): rung 47's temperature-limiter LAG only this step (52's release and 88's slow valve stay refused, next steps); rung 101's lagged sensor beside it left for another session (refused for now); ALL governor holds/laws ride it"
metadata:
  node_type: memory
  type: project
  originSessionId: a8f5c403-d6a4-452b-be07-1955f109c760
  modified: 2026-10-21T19:22:17.296Z
---

Rung 95's seam "the governor on the lagged marchers (47/52/88)". User answers (AskUserQuestion, 2026-10-21), after
"what is next" → "1":

- **Which lag: rung 47's temperature-limiter lag ONLY** (the original refusal, smallest change). Rung 52's realistic
  release and rung 88's slow bleed valve stay refused — follow-on steps.
- **Rung 101's lagged exhaust sensor beside it: "for another session"** — refused in words now, a later step.
- **Holds: ALL** — thrust, HP speed (rate and proportional), burner temperature, rung 96's inferred thrust (EPR / fan
  speed).

Advisor framing adopted: build on rung 101's three-state governed march in `governor.rs` (don't widen
`integrate_fuel_lagged`'s schedule signature); record the clip state at every point (a new Route); nothing old moves
(the combination is refused today); likeliest headline = the stateless governor's "nothing winds up" becomes
conditional (the clip is measured off the governor's demand) — derive its sign in the anchor before code; re-ask the
user once a prototype measures cost + effect ([[rung105-hp-vane]] lesson 1). Related: [[rung95-decisions]].

**Lean limit (asked after the probe, 2026-10-21): HOLD AND REPORT** — where the lag's leftover clip would take the
applied fuel below the leanest runnable one, hold it there (a minimum-fuel stop) and mark the point. Probe
(`docs/plans/scratch-archive/rung106/reach2.txt`): 399 of 2 520 runs, ALL slowdowns starting ABOVE the redline (the clip
stored at the start, then the governor dives); the open ramp on the SAME 399 configurations never gets there (re-checked after the advisor caught my first claim
resting on 6 runs: `scratch-archive/rung106/open_lean.txt`, all 399 ran to the end, 0 at the wall); no
acceleration and no start below the redline ever did. Prototype with the open ramp ≡ rung 47 bit for bit (`scratch-archive/rung106/check47.txt`);
cost ≈ rung 95's governed run (0.3–1.5×), so no cost re-ask.

**Hot start (asked next, prompted by a side-agent note): REFUSE IN WORDS** — governor + slow limiter with the START's
burner temperature above the redline is refused ("the limiter would not allow it; raise the limit or start lower"); the
lean-limit hold stays only as a safety net. Applies to the NEW route only — rung 47's open lagged route keeps
accepting such starts. Predictions are scored on starts below the redline.

**Hot REQUEST (asked after the crash map, 2026-10-21): REFUSE IT TOO** — crash #896 (seed 12): the metal cap held the
start at 978 K while the thrust governor's request at s = 0 was above the redline (Tt4 at the request > 1480), so the
clip charged from 0 under the metal leg and the fast slowdown then held the burner at the lean edge ~0.6 τ. Refused in
words when the governor's request at the start is above the limit (the clip's target there > 0) — same principle as
the hot start; the minimum-fuel stop stays a safety net.

**Speed-law edges beside the lag (asked twice after the advisor's floor sweep, 2026-10-21): LIMIT EACH LAW.** First
asked for the HP rate alone ("limit it", 0.1); the sweep then showed every speed law rough at its rung-95/96 edge, so
re-asked — "limit each law": HP rate ≥ 0.1, fan-speed rate ≥ 0.15, HP gain ≤ 10, fan-speed gain ≤ 7 (density-scaled as
rung 95's), refused in words. Thrust and pressure-ratio holds clean, no limit.

**Page defaults past the edges (side-agent note, asked): NUDGE THE KNOB** — the HP gain 20 and fan-speed τ 0.1 defaults
sat past the lag's edges; switching the lag / governor / hold / law / reading moves the knob to the edge and a note
under the lag says so; typed values still meet the refusal. Browser-checked
(`the_lag_beside_the_governor_nudges_the_gain_and_runs_the_lagged_route`).
