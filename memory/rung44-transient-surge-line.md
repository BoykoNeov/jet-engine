---
name: rung44-transient-surge-line
description: "SHIPPED rung 44 = the transient two-spool surge line; the accel drives BOTH spools toward surge, LP eats ~1.6-2.2x (rung 41 survives dynamically), but the excursion is SCHEDULE-slaved (rho-invariant, ramp-rate-driven, mode-INDEPENDENT); report the crossing / gate the flip"
metadata: 
  node_type: memory
  type: project
  originSessionId: 5f9ff7bb-fade-442b-957d-848eb020b7ff
  modified: 2026-07-23T03:05:12.990Z
---

SHIPPED rung 44 — **the transient two-spool surge line**: `phi_excursion` /
`transient_surge_margin` methods on `TwoSpoolTransient` (engine.py), gates in
`tests/test_rung44.py`, spec `docs/rung44-spec.md`, anchor
`docs/plans/rung44-anchor-transient-surge.md`, panel `print_transient_surge_table` in main.py.
Discharges the seam rungs [[rung40-two-shaft-transient]] and [[rung41-two-spool-surge-line]]
both deferred in the same words (march rung 40's trajectory against rung 41's imposed line;
rung 41 was steady-only, no surge-survival claim).

**The finding — a clean SPLIT, sign-space only** (phi_surge stays imposed, so NO survival claim;
rung 36's "report the crossing, gate the flip" discipline enforced):
1. The two-shaft **acceleration drives BOTH spools TOWARD surge** (phi dips below the steady
   running line — measured sign, not assumed; the forward-choke closure could have gone either
   way), and the **LP eats ~1.6-2.2x the HP's excursion** — rung 41's steady exposure split
   SURVIVES dynamically. Decel is the mirror (away from surge). Confirmation + extension.
2. The excursion is **SCHEDULE-slaved**: **rho-INVARIANT** (<2% over a 25x rho range, extremum
   always at the ramp's end), **ramp-rate-driven** (~5x, faster slam => deeper toward surge), and
   **mode-INDEPENDENT**. Rung 40's two exotic objects (rho = which spool leads; the LP-map complex
   mode) are BOTH surge-irrelevant — the surge-axis form of rung 40's own "schedule-slaved, not
   lead-governed" scope-limit. What governs the transient surge hazard is the mundane slam rate
   against the shaft clock.
3. `transient_surge_margin` ALLOWS phi < phi_surge and records it (unlike the steady
   `surge_margin`, which asserts clear): with a floor in the gap the LP transient CROSSES while
   every steady point clears — the first surge-relevant transient object. Flip's SIGN gated,
   crossing DEPTH disclaimed.

**Why this is a real rung, not a restatement:** it CORRECTS the tempting headline "the LP-map
complex mode is the transient surge story" — it isn't. Cross-rung shape identical to rung 40's
scope-limit inverted onto the surge axis.

**Advisor pattern (held again — this is why sign-errors keep getting caught):**
- **Probed the sign BEFORE framing** (temp probe.py/probe2.py), per advisor. The project reads
  signs backwards repeatedly ([[rung37-combustor-dynamics]], rung 40's hp-only, [[rung42-interstage-bleed]]).
- Referenced the excursion to the **instantaneous-Tt4 running line**, NOT the starting point —
  avoided the exact backwards-sign bug rung 40's first slip_excursion probe hit.
- Advisor **confirmed the sign independently** and blocked "done" on PROCESS: run the real green
  gate `pytest --runslow` (bare pytest deselects other rungs' slow finding gates) + `python main.py`
  end-to-end, since I edited a shared class (rungs 42/43 inherit `TwoSpoolTransient`) + added a panel.
- Advisor **framing sharpen**: the mode-irrelevance claim leads on the **damping ratio**
  (|Im/Re|<=0.164 < 0.25, airtight); the hp-only-largest-ratio leg is CORROBORATION only (it
  confounds mode-absence with an LP shaped->flat loading change — a partial artifact, named, per
  rung 41's own discipline). Demoted it in spec/anchor/panel/test so it can't drift into "proof".

**Reduce:** the rung-44 methods only READ (integrate/match); they add no state and never write
phi_surge => arming a surge line leaves rung 40's integrate/equilibrium/jacobian bit-for-bit (the
rung-41 witness, one rung on). Non-tautological gate = an independent bare-math CPG two-shaft accel
closure (own thermodynamics/choke/speed-lines/2-D root/EULER shaft march) reproducing the SIGN +
LP-over-HP + schedule-slaving (discharges rung 42's missing-bare-math-gate ding). Cycle rung-6 exact.

**Fork decided deliberately:** Tt4-control for the core (mirrors rung 36, isolates the surge
question from fuel control); the fuel path ([[rung35-fuel-metering]]/[[rung43-two-shaft-fuel-metering]],
where fuel control ENLARGES the single-spool excursion) is the natural OPEN extension, deferred.
