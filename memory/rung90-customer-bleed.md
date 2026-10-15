---
name: rung90-customer-bleed
description: Rung 90 (customer bleed at station 3) process lessons — a guessed bar on a derived claim was the only miss; a physical bound checked at bracket trials stopped good solves
metadata:
  node_type: memory
  type: project
  originSessionId: 1e590251-094c-4fd2-8af6-77f66a23dc7f
  modified: 2026-10-15T15:21:47.514Z
---

Rung 90 (2026-10-15): the station-3 customer bleed, steady + transient + rung 64's valve + sandbox knob in ONE rung (user's choice). Anchor `504d428` committed before any customer-bleed code; 13 HIT, A5 MISS (10/12), A8's 650 K point not scorable (unchoked).

**Lesson 1 — the only miss was the one bar I did not derive.** A1–A4 came from a two-line linear model (validated first on rung 42's published `b` response, an existing-code given) and hit within 0.01. A5's "> 1 % off-line at every Tt4" was a typed guess; it measured 0.55 % at 1500 K.
**Why:** a derived claim with a guessed magnitude attached scores the guess, not the physics.
**How to apply:** for each bar, either derive the number or register only the sign/ordering.

**Lesson 2 — check a physical bound at the ROOT, not at bracket trials.** The first build aborted "more than half the air" inside the Tt4-pinned closure's residual; its high-wall trial imposes a tiny flow, so good solves stopped. The model check moved to after the root (`share_raw` in trials, `share_at` on the answer).
**How to apply:** any refusal inside a bracketed residual must ask whether the TRIAL can legitimately sit there.

**Lesson 3 — a load lives on the design engine, not on a solver.** Putting the demand on `TwoSpoolEngine` made every `at_lever` sibling keep it for free (rung 64's sibling trap), and refusals in rungs 38/39/42/40/43 stop it being silently dropped.

Also: `grep -r` over the repo walks `rust/target` and times out — use the Grep tool with `!**/target/**`. CLAUDE.md's byte budget keeps 20 B slack; every rung raises it in-commit with a history line in `rust/tests/claude_md_reference.rs`.

Related: [[rung89-asymmetric-valve]], [[rung64-phi-bleed-limiter]], [[rung42-interstage-bleed]], [[sandbox-direction]].
