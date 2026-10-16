---
name: rung95-decisions
description: "User scope decisions 2026-10-16 for rung 94's seam (a governor holding thrust/speed DURING the slam): speed law BOTH as a toggle (commanded rate τ_N, proportional + steady table gain K); clamp+report at the rich wall and at the lean wall on decels; metal cap cuts fuel all along the move"
metadata:
  node_type: memory
  type: project
  originSessionId: c9380267-43e0-4c29-a2e4-34afc90f063b
  modified: 2026-10-16T08:34:42.619Z
---

Rung 94's seam — a closed-loop governor holding thrust or HP speed DURING the Controls slam. User answers 2026-10-16
(user's prompt: "a controller that holds thrust or speed during the throttle slam … alongside the existing fuel
limiters — work on it"):

- **HP-speed law: BOTH, a toggle** — commanded rate `dν_H/ds = (ν_dem − ν_H)/τ_N` (one knob τ_N), and proportional +
  steady table (fuel = steady fuel at the target speed + gain × speed gap; one gain knob).
- **Governor asks for more fuel than the engine can burn: CLAMP at the richest runnable mixture and REPORT** (hint: the
  temperature limit / accel schedule is what a real control uses).
- **Slowdowns: CLAMP at the lean limit and REPORT** (the min-fuel stop; the model has no decel limiter).
- **Metal cap (rung 94's `hold_cap`): cuts fuel ALL ALONG the move**, a new min-select leg, not only at the ends.

Advisor framing adopted (2026-10-16): the governor is the TOP of the min-select (it replaces the ramp as the demand the
legs cut), not a cut-only leg — corrects rung 94 spec's "two more legs" wording; thrust/Tt4 by instant inversion (no
constant), speed needs a law with a constant; first build = plain route + instant valve, refused beside rungs 47/52/88's
lags. Related: [[rung94-burner-hold]], [[rung93-94-decisions]], [[sandbox-direction]].

**Rung 95 SHIPPED 2026-10-16** — [[rung95-governor]] (H1–H6 all HIT; spec `docs/rung95-spec.md`, plan § 24). Its next
seam: a governor that INFERS thrust (fan speed / pressure ratio), or the governor on the lagged marchers (47/52/88).
