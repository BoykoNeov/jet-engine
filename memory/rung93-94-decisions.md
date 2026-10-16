---
name: rung93-94-decisions
description: "User scope decisions 2026-10-16 for rung 92's seam: TWO rungs — 93 metal-temperature limit (cooling buys a hotter burner, both framings as a switch, vane + blade metal; page = new 'Cool the turbine' view + opt-in slam readout; limits by material year), 94 load-carrying burner (hold thrust or HP speed, switch)"
metadata:
  node_type: memory
  type: project
  originSessionId: 76bf8353-8e43-4b1e-a44d-8d50c63caa37
  modified: 2026-10-16T06:19:07.993Z
---

Rung 92's seam (metal limit + a burner that carries the generator load) — user answers 2026-10-16:

- **Two rungs.** Rung 93 = the metal-temperature limit and cooling buying a hotter burner. Rung 94 = the
  burner that carries the load (follows 93).
- **Rung 93 framing: BOTH, as a switch** — (B) the sandbox's fixed engine run hotter (cooling + metal
  limit set the allowed `Tt4`), and (A) a fresh design at each setting.
- **Metal: vane AND rotor blade**, each cooled by its own rung-91 stream, each its own limit.
- **Page (rung 93):** a NEW view "Cool the turbine" for the steady trade (A/B switch, charts over `Tt4`),
  plus an OPT-IN metal readout on the Controls slam (off by default — uncooled metal is over the limit
  at every default throttle). **Limits set by material YEAR + part life** (Gauntner's relation; 1985 /
  10 000 h default → 1228 / 1172 K), the derived temperatures shown.
- **Rung 94: each throttle endpoint holds THRUST or HP SPEED (a switch)**; `Tt4` floats. Steady outer
  solve, fuel ramp between endpoints unchanged (advisor: a closed-loop governor in the march is a new
  limiter-family rung — not chosen).

**Rung 93 SHIPPED 2026-10-16** (all six predictions HIT; spec `docs/rung93-spec.md`, page view + readout,
plan § 22) — [[rung93-metal-temperature]]. **Next: rung 94** (the load-carrying burner), as decided above.

Anchor committed `e2a6c43` (`docs/plans/rung93-anchor-metal-temperature.md`). Correlation: Gauntner
1980 NASA TM-81453 (NTRS 19800011581); its "efficiency peaks" power-plant result does NOT transfer to a
turbojet (TSFC rises with `Tt4` even uncooled).

Related: [[rung92-shaft-offtake]], [[rung91-turbine-cooling]], [[sandbox-direction]].
