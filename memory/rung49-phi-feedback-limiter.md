---
name: rung49-phi-feedback-limiter
description: "SHIPPED rung 49 = the phi / surge-margin FEEDBACK limiter; a limiter acts on a spool through BOTH edges and they answer to DIFFERENT clocks (credit per-spool, debit ramp-clocked); an LP floor DEBITS the HP; bounds rung 48 without refuting it"
metadata: 
  node_type: memory
  type: project
  originSessionId: ebd8cf26-eb39-48a1-8bb6-40b64c97a4df
  modified: 2026-07-27T13:20:06.752Z
---

**Rung 49 is SHIPPED** (2026-07-27): the **φ / surge-margin FEEDBACK limiter** —
`SurgeLimiter`/`_surge_fuel`/`integrate_fuel(…,surge=…)`/`surge_relief`/`floor_sweep` on
`TwoSpoolFuelTransient`. It is the door [[both-edges-limiter-negative]] left open: the only
signal with a turnover upstream of a surge minimum is the surge variable itself.

**The findings.** (1) The min-select is a clean **sliding mode**, not chatter — hold error
~1e-15 — and it produces the **first engaged window with BOTH edges inside the ramp**, the
object no `pt3` filter can build. (2) **HEADLINE: a limiter acts on a spool through BOTH its
edges.** The engagement edge truncates a descent (credit — rung 48's term); the **release edge
RE-OPENS one** (debit — new), so an LP-watching floor **DEBITS the HP** while crediting the LP,
with the unwatched minimum relocating to one grid step **after `s_rel`**. (3) **The two edges
answer to DIFFERENT clocks** — credit per-spool (rung 48), debit by the **RAMP END** (rung 44's
clock): at `r`=2.0, where the two references sit 3.1× apart, the debit is **8× larger** at
`s_rel≈r` than at `s_rel≈s_hp*`. (4) Rung 48 is **BOUNDED, not refuted** — at `r`=0.15 the
release lands ≫ ramp and the unwatched relief **FLIPS positive**; and an HP-watching floor
reproduces rung 48's **exact zero** at `s_lp*` off a bare march. (5) The spool exposed to the
closing edge is the **LATE** one (the HP), inverting rungs 41/44/45's "the LP eats more".

**Why:** it completes the fuel-side limiter trilogy — feedback on TIT (46/47), feedforward on
pressure (48), feedback on the protected variable (49) — and it is the first rung where a
limiter is shown to *hurt* a spool.

**How to apply:** the method that worked was the one the advisor forced: **build the leg as a
local subclass in `M:\claud_projects\temp\rung49-probe\`, outside the repo, and measure the
signs BEFORE writing any spec.** Both my prediction and the advisor's (an LP floor rebates both
spools) were **wrong**; the probe inverted them. Also: the watched-spool relief
`φ_lim − min φ_bare` is **definitional** under a working set-point solve — it is gated as an
identity check only, never as evidence. See [[rung48-accel-schedule]], [[rung46-tit-topping-governor]],
[[rung47-lagged-topping-governor]], [[pt3-sensor-lag-negative]].

**The clock result is WITHIN-FAMILY, and I had to cut an over-claim to say so.** I first wrote
(on advisor framing) that rung 48's leg was "structurally confined" to the no-debit regime.
Probe 7 refuted it: rung 48's `m`=0.42 releases at `s_rel/r`=1.16 with 32/32 monotone cells while
the φ floor at 1.20 debits −0.008 — same ratio, opposite outcome. The hand-back-magnitude
hypothesis is refuted too (anti-correlated at `r`=2.0). The advisor retracted its own framing when
I brought the measurement.

**The seams it leaves:** (1) a **lag / hysteresis on the RELEASE edge** — rung 49 hands the fuel
back instantaneously, so its debit is a **lower bound**; (2) **why rung 48's leg escapes the
release debit at all** — the clip SHAPE (its cap rises with `pt3`, keeping the deficit at 0.3–0.7 %
where the φ floor's grows to 2.3 %) is the untested suspect.
