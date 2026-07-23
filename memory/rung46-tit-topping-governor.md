---
name: rung46-tit-topping-governor
description: "SHIPPED rung 46 = the TIT topping governor (first fuel-side FEEDBACK); enforcing the TIT redline SPLITS the surge relief — rebates the late HP spool but machine-zero the early binding LP one (surge debit paid on early-ramp fuel, upstream of the governor's late window); advisor blocked the tautological peak-diagnostic first, then greenlit the inverted finding"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9631707c-b2ad-471c-8b1c-ebaff114ecae
  modified: 2026-07-23T07:17:34.685Z
---

SHIPPED rung 46 = **the TIT topping governor** on `TwoSpoolFuelTransient`
(`integrate_fuel(Tt4_max=…)` min-select clip + `_topping_fuel` set-point solve +
`topping_relief`). The **first fuel-side FEEDBACK** in the ladder (35→43→45→46) — every prior
fuel rung ended on "no TIT limit is modelled"; this models it. User picked the **control/enforce**
fork over the **diagnose/dwell** fork.

**The finding (INVERTED from the starting hypothesis).** The governor works — pins `Tt4` at the
redline (the effect added). But enforcing the TIT limit does NOT relieve the binding surge margin:
it rebates surge on the **LATE, non-binding HP** spool (`relief_hp>0`) but is **MACHINE-ZERO on the
EARLY, binding LP** one (`relief_lp=0` exactly). A two-shaft **surge-relief SPLIT** no single shaft
can show. Mechanism: the surge debit is paid on the **early-ramp fuel** (LP surge min at `Tt4≈1374`,
DURING the ramp, below any valid redline, then self-recovers); the governor only trims **late** fuel
(`Tt4>redline`, near ramp end), **upstream of which** the LP cost was already incurred. Rung 35's
two accel limits are coupled in CAUSE but **SEQUENCED in time**; the governor acts on the trailing
(TIT) limit and structurally misses the leading (surge) one. This completes rung 45's "fuel enlarges
the surge approach": the enlargement is deposited early, the topping governor is too late to claw it
back. **Lever/caveat:** `relief_lp=0` only at moderate `r`; in the fast-ramp limit (`r≤0.3`) the LP
surge min migrates above the redline and `relief_lp` goes positive — a modest LP-surge lever exactly
where surge is most dangerous. Robust across all 4 shapes incl. **hp-only** (LP flat, no rung-40
complex mode) ⇒ the pure WINDOW mechanism, not a mode artifact.

**Advisor was load-bearing twice.** (1) It **BLOCKED my first scope** — a peak-exceedance diagnostic
(`Tt4_max−Tt4_peak(ρ)`) — as tautological (rung 43's `Tt4_peak(ρ)` minus a constant; the clean
single-shaft `lp_disabled⇒rung 35` reduce was the tell the two-shaft novelty was already spent). It
offered the dwell-diagnostic vs topping-governor fork; I surfaced it to the user, who chose the
governor. (2) After I PROBED and the sign INVERTED, it **retracted its own "enforce TIT → relieve
surge" framing**, told me to center the headline on the **DIFFERENTIAL** (`relief_hp>0` while
`relief_lp=0` — un-attackable as two-shaft) not on "timing" (which exists single-shaft too), and made
the **shape sweep** (all 4 maps) the one must-do before gating. All measured clean.

**Reduce:** dormant (`Tt4_max=None` or ≥ bare peak) ⇒ rung 45/43 bit-for-bit; `lp_disabled` ASSERTS
(the split is inherently two-shaft); decel never fires ⇒ rung 45 bit-for-bit; cycle ⇒ rung 6. No
independent bare-math gate (reads rung 43's `integrate_fuel`, anchored transitively to rung 40).

Idealised **instantaneous** governor (min-select, no `τ_gov`) — a lagged/actuator governor is the new
OPEN seam (a slow-enough clip could smear into the early LP surge point). See [[rung45-transient-fuel-surge]],
[[rung35-fuel-metering]], [[rung43-two-shaft-fuel-metering]].
