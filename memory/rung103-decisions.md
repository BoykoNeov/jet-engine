---
name: rung103-decisions
description: "User scope decisions 2026-10-19 for rung 103 (the STEADY two-shaft matchers' unchoked branch, CLAUDE.md OPEN seam / rung 97's next seam): switch OFF in old code, ON in the sandbox pages; all six matchers (38/39/42/90/91/92); sub-idle (net thrust <= 0) refused in words like rung 33"
metadata:
  node_type: memory
  type: project
  originSessionId: cb6a1eda-ec03-4b47-9ef7-4103a61da000
  modified: 2026-10-19T11:13:31.318Z
---

Picked 2026-10-19 ("5" from my post-102 list: the steady matchers below choke).

**Reach census** (worktree of `2688ce8` with a one-call logger at the six refusal sites, full gate): 1 642 refusals —
r92 894 (rung94 714, cli_golden rust-owned panel 180), r39 501 (slice_l_oracle 318, two_spool_oracle 134, rung41 36),
r42 191, r38 48, r91 7, r90 1. Python-held oracles, CLI goldens, rung 41/42/90/94 claims and the Blades/Cooling page
tests all READ these refusals — hence the switch.

**Probe** (sandbox Controls engine, 4 maps × 4 flights × 550–1150 K): 74/208 refused; rung 97's helper inside rung 39's
joint (f, pt4) loop converges at 70 (8–60 passes, never capped), one root in every residual scan, equal to rung 40's
transient equilibrium to every printed digit where that converges (it fails at 34 cells, incl. flat-lp CHOKED ones).
4 true refusals (3 "nozzle cannot pass", 1 off-map). ~9 new points have net thrust < 0.

**User answers (AskUserQuestion):**
- Default: **OFF in old code, ON in the pages** (Blades, Cooling, Controls' generator check) + this rung's tests;
  re-run old rungs' claims with it ON and REPORT what moves.
- Scope: **all six** steady matchers.
- Below idle: **refuse in words** (rung 33's precedent; shared `score` not built for negative thrust).

**How to apply:** dispatch only where the old guard fires (advisor: in-loop choke tests could re-path choked points);
do not reuse rung 97's `lp_pinned` as this rung's switch. Related: [[rung97-unchoked-lp]], [[rung33-subsonic-matching]].

**Status 2026-10-19:** rung 103 BUILT ([[rung103-steady-unchoked]]; spec `docs/rung103-spec.md`, plan § 35). H1 37/38 (the miss at M9 0.9998), H2 174/174.
