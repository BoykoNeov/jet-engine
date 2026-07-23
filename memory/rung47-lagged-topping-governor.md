---
name: rung47-lagged-topping-governor
description: "SHIPPED rung 47 = the lagged/actuator topping governor (tau_gov); a first-order lag REFUTES rung 46's next-seam hope"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9e2e2ace-0744-4dcb-a101-c2e2f6b20e60
  modified: 2026-07-23T08:36:25.792Z
---

SHIPPED rung 47 = **the lagged / actuator topping governor** (`tau_gov`) — the response-lag
realism on rung 46's instantaneous TIT governor. `TwoSpoolFuelTransient.integrate_fuel(…,
tau_gov=…)` / `_integrate_fuel_lagged` (the clip AMOUNT `g` a THIRD state:
`dg/ds=(required−g)/tau_gov`, `mf=schedule−g`), threaded through `_fuel_ramp_march` /
`phi_excursion_fuel` / `transient_surge_margin_fuel` / `topping_relief`, plus
`topping_command_trace`.

**The finding REFUTES rung 46's own concession** ("a slow-enough governor could smear the clip
window and reach EARLIER into the LP surge point"). A first-order lag is a **TRAILING-edge** tool —
it delays, never anticipates ("reach earlier" needs phase LEAD). Formulation-independent:
`relief_lp = 0` EXACTLY for every tau_gov at moderate r (topped-lagged is bit-identical to bare up
to the LATE engagement, upstream of which the EARLY LP surge min at s≈0.24/Tt4≈1374 already passed).
At FAST r (where rung 46's INSTANT governor DID reach the LP, relief_lp>0 — the lever), the lag
**ERODES** that positive relief toward 0, never enhances it. In NO regime does the lag reach the LP
better than the ideal min-select. **The cost of realism:** the lag breaks rung 46's "governor
HOLDS the redline" (gate 3 inverted) — Tt4 OVERSHOOTS growing with tau_gov (~55→190 K in-band at
r=0.5, ~220→390 K at fast r=0.15, the classic topping overshoot) — and erodes the HP rebate toward 0.

**Secondary (where the lag lives):** the overshoot lives in the sensing/limiter-LOOP lag, NOT the
valve. A pure metering-VALVE-position lag is INERT on the accel because the binding topping command
RISES monotonically (`topping_command_trace` monotone) — an instant-up valve tracks it with no lag.
WHERE the lag lives decides whether it even overshoots.

**Model choice (advisor-blocked):** ship the LOOP lag (lag the clip AMOUNT). The **coupled
single-valve lag (Model X — lag the whole fuel path incl. schedule) was REJECTED**: it de-fangs
the accel (bare Tt4 peak collapses 1698→1462 K), BREAKS the rung-45 dormant reduce, and its
apparent relief_lp>0 is the accel being slowed, not the clip reaching earlier (a rung-42/43/45-kind
currency confound). Isolating the governor (bare=rung 45 exactly, tau_gov→0=rung 46 exactly) is
what makes the differential admissible.

**Advisor arc (typical for this project):** (1) BLOCKED Model X on the reduce-break + confound,
insisted the physics settles it (a lag is trailing-edge; "reach earlier" needs lead). (2) OWNED
its own wrong "first-order slew to topping" sketch when I proved it INERT (the valve-slew tracks a
monotone-rising command → 0 overshoot); reframed the loop lag as a lagged TIT feedback. (3)
Corrected its earlier "relief_lp≤0, could go slightly negative" → it's `==0` in-band, positive-but-
eroded at fast r, never negative. (4) BLOCKED the spec until I measured the fast-ramp erosion (the
one regime where the concession's hope had a chance) — it confirmed erosion. Precedent: measure the
SIGN first, bring data, let the advisor reshape the framing.

Reduces: tau_gov=None bit-for-bit rung 46; dormant redline (required≡0⇒g≡0) bit-for-bit rung 45;
lp_disabled ASSERTS (inherently two-shaft) + tau_gov needs a redline; decel never fires; cycle
rung-6 exact. Tests CPG (gas-independent, reacting-gas fuel control still deferred). Spec
`docs/rung47-spec.md`, anchor `docs/plans/rung47-anchor-lagged-governor.md`, gates
`tests/test_rung47.py`, main.py `print_lagged_governor_table`. The OPEN door rung 47 leaves: a
**lead-compensated / anticipatory** governor (the one thing a pure lag cannot do — reach the early
LP min). See [[rung46-tit-topping-governor]], [[rung45-transient-fuel-surge]],
[[rung43-two-shaft-fuel-metering]].
