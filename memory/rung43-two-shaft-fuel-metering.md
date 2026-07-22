---
name: rung43-two-shaft-fuel-metering
description: "SHIPPED rung 43 = two-shaft fuel metering; the two spools sit at DIFFERENT points in ONE overshoot loop so NEITHER clock governs it; the referenced currencies turned out CIRCULAR and that trap is the rung's real lesson; also found+fixed a pre-existing rung-40 solver hole"
metadata: 
  node_type: memory
  type: project
  originSessionId: 71bfd339-a351-421e-97da-3e4e1f05452d
  modified: 2026-07-22T19:11:15.590Z
---

Rung 43 (`TwoSpoolFuelTransient`) carries rung 35's **fuel** control onto rung 40's
two-shaft plant: `ṁ_fuel` metered, `Tt4` an OUTPUT floating against the airflow *two*
lagging spools can pump. Seam named by rung 40. User picked it from a candidate list with
the two-word instruction "Two-shaft fuel metering".

**The finding is a MECHANISM, not a number.** `f = ṁ_fuel/ṁ_air` is set at the **LP face**
but the `Tt4` it produces is metered back through the **HP-fed** NGV choke — the two spools
sit at *different points in the one loop*. So "which spool's lag governs the overshoot?" (a
question one shaft structurally cannot ask) answers **NEITHER**: freezing *either* spool
makes the overshoot WORSE (6/6), and the share of the relief trades with `ρ`. The positive
is bounded *structurally*: `ρ` multiplies only the LP ODE, so `ρ→∞` **IS** the LP-frozen
march — `ρ`-independent bit-for-bit, with `X(ρ)` converging up onto it.

**The methodological lesson — CURRENCY CIRCULARITY.** Excursions referenced to a running
line read back **whichever spool sits in the denominator**: the fitted effective-clock
exponent went 0.05 (`ν_H`-referenced) → 0.35 (spool-neutral `X`) → 0.45–0.65
(`ν_L`-referenced). So `q≈0` was never evidence that "the HP clock governs" — it was the
reference reading itself back. **The data selected the instrument, not the answer.** This is
the same family as the [[rung42-interstage-bleed]] confounded-relative-`SM` catch, one rung
on, and worth reaching for whenever an excursion is quoted as a ratio.

**Four claims written, probed, withdrawn** (kept visible per rung 40's convention): the
`√det` composite clock (a true rung-40 identity, *not connected* to the overshoot — 0.35 is
merely the midpoint of the two circular currencies); "`q=1` refuted in every currency"
(FALSE — on `X`, `q=0` fits worse); "irreducibly two-dimensional" (overclaim — only
*power-law* collapses were tested); and, killed **before any code**, "fuel metering breaks
rung 39's `(†)`" (a category error — `(†)` is a steady η-fixed-point artifact absent from
the transient closure).

**Two process notes worth keeping.** (1) The advisor **retracted its own earlier supporting
advice** mid-review when I brought measurements — its predicted separation was wrong and it
had not anticipated the currency flip. Bringing numbers back beat taking the estimate.
(2) A test failure exposed a **pre-existing rung-40 defect**, not a rung-43 one: rung 40's
2-D Newton chased an *absolute* `_EQ_TOL`=1e-12 below the reacting gas's own ~1e-10 residual
noise floor, so it raised at `Tt4`=1300/1400 while 1500/1450/1200 squeaked under
(non-monotone in `Tt4` ⇒ solver artifact, not physics). Fixed with a **best-so-far
acceptance AFTER the loop** — reached only by inputs that previously *raised*, so prior rungs
are untouched **by construction** rather than by testing. Advisor blocked the
stagnation-exit-inside-the-loop I first reached for, which could have changed a returned
iterate mid-descent. Precedent: diagnosing a surprise into a *prior* rung and fixing it there
is correct here, per the project's "stop and explain surprises" contract — see
[[rung28-coupled-no-march]] for editing a shipped rung.

Reduce: control-invariance (machine zero, forward-burner closure) + `lp_disabled` ⇒ rung 35
bit-for-bit + `Tt4`-control untouched ⇒ rung 40 bit-for-bit. No independent bare-math gate —
the second deliberate break in the rung-38/39/40 streak (after [[rung42-interstage-bleed]]),
because unlike those rungs this reduce *does* enter the new code and lands on rung 40's own
bare-math-anchored manifold, so the closure is anchored transitively.
