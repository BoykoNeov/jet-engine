---
name: rung45-transient-fuel-surge
description: "SHIPPED rung 45 = transient two-spool surge line on the FUEL path; rung 43's rho-monotone Tt4 overshoot NEVER reaches the reference-free surge object (a currency trap); fuel ENLARGES the approach + compresses the excursion ratio"
metadata: 
  node_type: memory
  type: project
  originSessionId: 821e37b8-eaa6-49a6-ba6a-8e418b1c031a
  modified: 2026-07-23T04:16:16.617Z
---

SHIPPED rung 45 = rung 44's transient-surge diagnostic (`phi_excursion_fuel` /
`transient_surge_margin_fuel` on `TwoSpoolFuelTransient`) on rung 43's FUEL-controlled plant,
where `Tt4` is an OUTPUT that overshoots.

**THE HEADLINE (correction, rung-43 currency-circularity echo on the surge axis):** rung 43's
TIT overshoot is strongly rho-MONOTONE (~12% over 25x rho), yet it does NOT reach the
reference-free surge object — the raw transient min phi is rho-INVARIANT (0.63% < rung 44's 2%
bar, weakly monotone SAME direction, an order weaker — "weakly coupled, not decoupled"). The rho
signal is real in the PLANT, absent from the SURGE MARGIN; it surfaces only in reference-dependent
currencies (the output-Tt4-referenced excursion swings ~40% — a moving-reference artifact). So
rung 44's "rho powerless over surge" SURVIVES the control swap on the reference-free object.

**Confirmed-prediction legs (rung 44's concession forecast both, so NOT the finding):** fuel
ENLARGES the surge approach (raw min phi deeper than Tt4-control at every matched r — rung 35 on
two shafts, the Tt4 overshoot amplifies it); the split survives (accel toward surge, decel mirror,
LP leads) but the LP-eats-more DOMINANCE COMPRESSES (excursion ratio 1.24-1.74 vs rung 44's
1.6-2.2 — the strong asymmetry moves to the raw margin where LP crosses / HP clears wide);
ramp-rate still governs.

**Reference discipline (advisor-set):** use the COMMANDED (fuel-equivalent) running line for the
excursion (reduces to rung 44 exactly on the Tt4 path); the output reference is named+rejected. The
SURGE claim is put on the reference-free raw margin so the headline doesn't ride on the reference
debate. Decel only gated on the referenced excursion (ext>0) — the raw margin is DEGENERATE on a
decel (moves away from surge, tr~st~0, measured +0.00006), advisor caught my window-fix wouldn't
work. No independent bare-math gate (rung 43 precedent — anchored transitively via control-invariance
to rung 40's cascade). lp_disabled ASSERTS (split is inherently two-shaft).

**Advisor arc:** probed sign first, then advisor reshaped: lead surge on reference-free min_phi not
the excursion; reframe headline as the currency trap (NOT "rung 44 survives"); my reconcile
measurement (min_phi 0.63% rho-invariant) made the advisor RETRACT its own "re-opens a secondary rho
channel" (it had been reading the ext(out) artifact it told me to keep off the critical path).

Related: [[rung44-transient-surge-line]] [[rung43-two-shaft-fuel-metering]]
[[rung35-fuel-metering]] [[rung41-two-spool-surge-line]]. Files: docs/rung45-spec.md,
docs/plans/rung45-anchor-fuel-surge.md, tests/test_rung45.py (8 gates), methods in engine.py,
main.py panel print_transient_fuel_surge_table.
