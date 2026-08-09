---
name: rung75-antiwindup-device
description: "Rung 75 shipped — an anti-windup device is decisive on the SPECTRUM and inert on the RANK; the blind-instrument trap was caught before it produced a perfect refutation; rung 74's \"divergence\" was a contraction at ratio one"
metadata: 
  node_type: memory
  type: project
  originSessionId: 49a76bcc-4eba-4f33-a026-7c85b09f6e6b
  modified: 2026-08-09T18:37:02.434Z
---

Rung 75 (shipped 2026-08-09) declared the anti-windup device rung 74 § 4 found this family
had **by accident** (rung 52's `max(0,·)`): back-calculation, `dw/ds += (mf_app − w)/τ_t`.

**HEADLINE: decisive on the SPECTRUM, inert on the RANK — the exact inverse of
[[rung74-demand-coordinate]].** The term is state-DEPENDENT, so unlike rung 74's forcing it is
in the Jacobian: it writes `−1/τ_t` onto the masked diagonal [[rung73-applied-reference]] had
cancelled to zero. The pole leaves the ORIGIN, `det J` revives, `zeros` drops by 1 — and
`n_live` ≤ 3 stands a **fourth** time, because the term is in the masked leg's **ROW** while the
COLUMN stays zero. Rung 71 found `zeros` counts gradients, not live loops; this is its converse:
**a pole is not a loop either.**

**The three lessons worth carrying:**

1. **A new term can make every inherited instrument blind, and the blindness reads as a clean
   refutation.** Every gains reader in this ladder differences the leg's *target* and hands it
   to `_jac4`; the tracking term is in neither the target nor `taus`. Run as-is, the shipped
   reader would have reported "diagonal unchanged, `det J` still dead, spectrum invariant" — a
   perfect refutation of the headline, having measured nothing. Caught by the advisor before any
   Jacobian was read. **Before measuring a new term, check the instrument can see it** — ask
   what the reader actually differences, not what it is named. Seventh instance of this
   family's own pattern (see [[rung72-shared-actuator]], [[rung73-applied-reference]]).
2. **"It diverged" can be "it contracted at ratio exactly one."** Rung 74 reported IC residual
   `2.898e−3` after 60 iterations as *no interior equilibrium*. The sweep is a fixed-point
   iteration with slope `σ = τ_t/(τ+τ_t)`; `ceil(ln(tol/res₀)/ln σ)` predicted **185/98/54/32
   against 185/98/54/32**, using rung 74's own residual and zero fitted constants. Its verdict
   stands (`τ_t→∞ ⇒ w*→∞`); its number is explained, and the apparent existence boundary was
   the 60-iteration cap cutting a geometric sequence. **A non-converging solver is a
   measurement — read its rate before calling it a failure.**
3. **The one refutation was a prediction written on the STATE about an equality that lives on
   the OUTPUT.** P8 said dormant ⇒ track ≡ latch. Measured: output agrees `0.0` exactly, state
   never agrees at all (the park law puts the tracker *above* the schedule because the target
   term still pushes toward `cap > mf_sched`). Third shape of the same confusion after rung 74
   P6 and rung 58's *check the SUM, not the term*.

**And the trap bit again:** `_ic_cap` was set on the outer rig, `_shared_rig` returned a fresh
machine without it, and the two slowest arms reported ASSERT instead of 185/98 — thirteenth
instance, first time on a knob added minutes earlier. Gated now.

`τ_t` is the first new constant since rung 65 and is not derivable; every finding is a sweep
property or a threshold on it (redline holds iff `τ_t/τ_f ≲ 1.25`). Its fast end is grid-limited
at `0.00625` by the inherited RK4 floor — disclosed, not loosened.
