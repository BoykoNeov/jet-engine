---
name: rung48-accel-schedule
description: "SHIPPED rung 48 = the Wf/pt3 acceleration schedule (feedforward min-select leg); UNIFIES rungs 46/47 — a fuel-side limiter rebates a spool IFF it engages upstream of THAT spool's own surge minimum"
metadata: 
  node_type: memory
  type: project
  originSessionId: 1979c54f-103b-47ac-8082-1b7a3f31382e
  modified: 2026-07-27T08:45:56.941Z
---

**SHIPPED rung 48** (2026-07-27) — the `Wf/pt3` **acceleration fuel schedule**:
`AccelSchedule` + `accel_schedule` / `_sched_fuel` / `integrate_fuel(…, accel=…)` /
`schedule_relief` / `engagement_sweep` on `TwoSpoolFuelTransient`. Spec
`docs/rung48-spec.md`, anchor `docs/plans/rung48-anchor-accel-schedule.md`, gates
`tests/test_rung48.py` (14, all passing incl. shape robustness).

**The headline — a UNIFICATION, not just a new leg.** The rung-46/47 LP/HP surge-relief
split is not a spool property nor a limiter property: **a fuel-side limiter rebates a spool
IFF it engages UPSTREAM of THAT spool's OWN surge minimum.** Measured with ONE instrument
because the schedule margin `m` maps continuously to an engagement start time `s_eng(m)`
(the bare `(Wf/pt3)/κ_ss` ratio rises monotonically *through* both minima). At r=0.5:
`relief_lp` → **exactly 0** as `s_eng` passes `s_lp*`=0.24 **while `relief_hp` is still
+0.0075**, and `relief_hp` dies only as `s_eng` reaches `s_hp*`=0.40. Rung 46 is the special
case of a leg that is late by construction; rung 47 the case where a lag makes it later.

**Why feedforward works where rung 47's lag could not:** the door rung 47 left open was
labelled "phase LEAD" — but the answer was **watch the INPUT, not the output**. `Wf` steps
up immediately while `pt3` can only rise as the spools spin up, so the ratio is already 21%
above `κ_ss` at s=0.10, far upstream of the LP min. See [[rung47-lagged-topping-governor]],
[[rung46-tit-topping-governor]].

**Method notes worth keeping:**
- **The advisor BLOCKED the spec twice, and was right both times.** (1) It rejected my
  "match the accel time and compare" control as a re-run of the rung-42/43/45 currency trap
  (a matched-time slower ramp is a different plant-in-time), and demanded one measurement
  first: how far above `κ_ss` the bare ramp sits AT `s_lp*`. That number (40%) is what
  decided the rung existed rather than being rung 44 restated. (2) It then demanded the
  fuel-removed-vs-relief pair before any spec text. It also **withdrew its own earlier
  framing** once the probe landed. Probe-before-spec keeps paying.
- **The non-tautology gate that carries the rung:** fuel removed varies SMOOTHLY and stays
  POSITIVE through the crossing where relief switches EXACTLY off, the endpoint is unmoved,
  and at m=0.45 ONE clip removing ONE quantity of fuel rebates the HP and gives the LP
  exactly nothing. **A ramp-rate story cannot split two spools from the same removed fuel.**
- **The honest boundary is gated, not hidden:** at small `m` the leg DOES degenerate into
  rung 44's ramp-rate lever (accel does not complete). Own gate, so it can't be folded in.
- **Bit-for-bit lesson:** two Illinois solves off different brackets agree only to
  tolerance, so the min-select must compute EACH cap independently from the *scheduled*
  fuel — otherwise arming one leg perturbs the other's root in the last bits and the
  two-leg composite reduce fails at 1e-13. Structure the min-select, don't chain it.
- Two claims of mine needed correcting against the data mid-build: a "5 dp endpoint for
  every m≥0.10" spec sentence its own table contradicted (true only for m≥0.25; 0.012% at
  m=0.15), and a backwards assertion in the composite gate.

**Next seam this leaves open:** a **lagged/filtered `pt3` sensor** — i.e. whether sensor lag
pushes `s_eng` past `s_lp*`. Rung 48's crossing, asked as a *sensor* question. Also still
open: a rate-limit, a sensor+actuator cascade, the variable stator, a bleed schedule
`b(n_L)`, fuel+bleed together.
