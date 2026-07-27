---
name: rung50-release-edge-isolated
description: "SHIPPED rung 50 = the release edge ISOLATED via a forced release time s_off; the closing edge relocates BOTH spools' minima to itself; CLOSES rung 49's open seam — rung 48's immunity is TIMING not clip SHAPE"
metadata: 
  node_type: memory
  type: project
  originSessionId: 6c4b8881-ee10-4449-9e2c-eac08d19e0cc
  modified: 2026-07-27T15:52:49.519Z
---

Rung 50 (shipped 2026-07-27) is **the release edge, ISOLATED** —
`integrate_fuel(…,s_off=…)` / `release_relief` / `release_sweep` on
`TwoSpoolFuelTransient`. Spec `docs/rung50-spec.md`, anchor
`docs/plans/rung50-anchor-release-edge.md`, gates `tests/test_rung50.py` (15).

**The instrument is an ISOLATION DIAGNOSTIC, not a control law** — a forced disarm
time, in the same tradition as rung 34/40's `freeze='lp'` (which holds a spool's
speed against its own ODE). It is a pure function of `s`, so **no state and no
latch**: `s` threads into the RK4 sub-steps exactly as `fuel_schedule(s)` already
does. A boolean latch would flip between k1 and k4 and silently destroy the
integrator's order — [[rung47-lagged-topping-governor]] hit that and paid for it
with a third state.

**Why it had to exist:** [[rung49-phi-feedback-limiter]] could only move a
limiter's release edge by moving `φ_lim`, which drags `s_eng`, the window length
and the clip depth with it — so its clock claim was hedged as *within-family*.
`s_off` slides the closing edge **alone and two-sided** (hysteresis and lags move
it one way only), everything up to it bit-identical.

**HEADLINE:** the release edge **relocates BOTH spools' minima to itself** —
watched and unwatched, both instrument families, ds-independent — under a two-part
precondition that *is* the two-branch law
`min(rung-48 truncation, the dive bottoming at s_rel)`.

Three consequences: rung 49's clock hedge **LIFTS** (debit 2.6× deeper near the
ramp end than at `s_hp*`, walking through it unnoticed); an early release **DEBITS
THE SPOOL IT WATCHES** (rung 49's identity **BOUNDED**, not broken); and **the
standing open seam CLOSES** — rung 48's own leg forced inside the ramp debits both
spools, so its immunity is **TIMING, not clip SHAPE** ([[rung48-accel-schedule]]'s
exact zero survives untouched). Rung 49 §4's "hand-back magnitude does not
transfer" was itself **confounded**: at *fixed* release time the debit is monotone
in the deficit across both families.

**Method lessons worth repeating:**
- Predictions were written to a file **before** measuring. Two scored wrong on
  shape — the watched-spool relief goes *negative* first rather than rising to
  saturation. That inversion is only a finding because the prediction was on file.
- The first `r`=0.5 run *looked* decisive but sat inside rung 49's own
  `s_hp*`-vs-ramp-end confound. Re-running at `r`=2.0 (clocks 3.1× apart) was what
  actually decided it. **Refusing my own first confirming result was the key move.**
- One inferred mechanism ("the two families' deficit trends run opposite") was
  measured and **killed**; it is recorded in the spec §6 and anchor §7 because it
  was made, not hidden.
- The advisor **retracted its own advice** mid-task (it had said to probe with
  `s_off` then ship `τ_rel`; once the probe landed it said ship `s_off` and name
  `τ_rel` as the seam, because nothing measured separates total deficit from
  deficit *rate*).

Next seam: a **finite release-edge RATE** `τ_rel` — rung 50 moves *when* the fuel
is handed back, never *how fast*, and nothing here separates total deficit from
deficit rate.
