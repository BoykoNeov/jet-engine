---
name: rung109-decisions
description: "User scope decisions 2026-11-19 for rung 109 (the TURBOFAN, roadmap B1): design point only on the Design view; fan nozzle follows the core's nozzle mode; one lumped turbine drives fan + compressor; extras = best-fan-pressure-ratio readout, fuel-vs-bypass chart, two-jet efficiency rows"
metadata:
  node_type: memory
  type: project
  originSessionId: 86c1c634-fca3-4bea-9c24-fe8da115d6c2
  modified: 2026-11-19T08:44:56.724Z
---

User said "what is next" → I listed roadmap B1–B5 → "go with your recommendation" (B1, turbofan), 2026-11-19.

**Anchor:** Mattingly, *Elements of Gas Turbine Propulsion* (McGraw-Hill 1996 — NOT the AIAA *Elements of
Propulsion* the earlier anchors cite), Example 7-6, pp. 398–400: separate-exhaust turbofan, M0 0.8, T0 390 °R,
πc 36, πf 1.7, α 8, Tt4 3000 °R, dual CPG (1.4/0.240, 1.33/0.276), e_c 0.90, e_f = e_t 0.89, P0/P9 = P0/P19 = 0.9.
Book F/ṁ0 18.02 lbf/(lbm/s), S 0.6366. The scan came from a site of unknown legitimacy — numbers go in the anchor
doc, the PDF is never committed. **Book quirk:** printed η_P 74.65 % / η_O 30.60 % are THRUST-based (η_O = F·V0 /
fuel power, η_P = η_O/η_T); its own printed η_P formula (7-50, valid only at P9 = P0) gives 66.3 %.

**Probe (2026-11-19, `W:\temp\claude\rung109-turbofan\probe`):** shipped components + a lumped turbine paying
`α·(h_c(Tt13) − h_c(Tt2))` + a cold-section nozzle reproduce every printed number ≤ 0.05 %. α = 0 is bit-equal to
`build_turbojet().run()` on all 5 gases once the scoring uses `score`'s exact expression order (`(1+f)*(v9*v9)`).
Cruise default (πc 10, Tt4 1500, equilibrium): TSFC 34.0 → 22.6 g/(kN·s) for α 0 → 10; best πf falls with α.

**User answers (AskUserQuestion):**
- Scope: **design point only**, Design view, bypass-ratio knob, α = 0 ≡ today's turbojet. Off-design = a later rung.
- Fan nozzle: **follows the core nozzle's mode** (expanded / exit pressure / convergent) — needs a cold sonic throat.
- Turbine: **one lumped turbine** drives compressor + fan (the textbook's method).
- Extras, all three: **best fan pressure ratio** readout (search), **fuel-vs-bypass chart**, **two-jet efficiency rows**.

Related: [[roadmap-and-standing-engine]], [[rung102-decisions]] (same staging: anchor → code → page).
