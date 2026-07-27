---
name: pt3-sensor-lag-negative
description: "rung 48's own named next seam (a lagged pt3 sensor) investigated and NEGATIVE — an effective-margin reparameterisation; confirms rung 48 and corrects its seam's SIGN; NOT shipped, NOT a rung"
metadata: 
  node_type: memory
  type: project
  originSessionId: b0adbb35-2f0c-4a25-90a5-5d8dd44a10a7
  modified: 2026-07-27T10:01:01.336Z
---

**INVESTIGATED, NEGATIVE (2026-07-27)** — the lagged/filtered `pt3` sensor on rung 48's
`Wf/pt3` feedforward leg. `docs/pt3-sensor-lag-negative.md`. **NOT shipped, NOT a rung** —
no `engine.py`/test/spec code. Probes in `M:\claud_projects\temp\rung49-probe\`.

**The deflation was structural and it won.** `cap = (1+m)·κ_ss·pt3·[q/pt3]` — the sensed/true
ratio is a PURE multiplicative scalar on the cap, so a sensor lag is exactly a *time-varying*
margin. The only question was whether the time-varying one does anything a CONSTANT one
cannot. At matched engagement it does not: `Δrelief_lp` = 0.0–0.4 %, stable across two `ds`.

**The measurement that decided it had to be SUB-GRID.** My first matched-`s_eng` control
matched on the recorded grid value (`ds`=0.02) and reported a 12 % gap — an artifact, since
rung 48's own `relief_lp` moves ~33 % between `ds`=0.02 and 0.005, so 12 % sat INSIDE the
discretization band. The advisor blocked the rung reading on exactly this and was right.
Fix: engagement = interpolated zero of `slack(s)=mf_sched−cap`, bisect `m'` on THAT, and
refine in `ds` to check the sign is stable. **Lesson: a difference smaller than your own
discretization band is not evidence — check it against the band before it becomes a claim.**

**Two by-products worth keeping (the reason the negative was worth doing):**
- **Rung 48's seam had the SIGN BACKWARDS.** Its spec asked whether the lag pushes `s_eng`
  PAST `s_lp*`. A lag on a *rising* `pt3` reads LOW ⇒ cap LOWER ⇒ engages **EARLIER**. The
  opposite of [[rung47-lagged-topping-governor]], because that lags the limiter's OUTPUT
  trigger and this lags its INPUT measurement. Spec + CLAUDE.md corrected (rung-28 precedent
  for editing a shipped rung).
- **A real NEGATIVE CONTROL for [[rung48-accel-schedule]]'s rule**, which rung 48 lacked: at
  `m=0.48, τ_p=0.05` the lag removes 15× the fuel and moves engagement 0.400→0.280, yet
  `relief_lp` is EXACTLY 0 — 0.280 is still downstream of `s_lp*`=0.240. And at matched
  engagement the sensor removes **57 % more fuel** for the **same** relief. Fuel removed
  downstream of the minimum purchases NOTHING.

**Why the one new DoF can't matter — structural, not config.** The lag does decouple the
RELEASE edge from engagement (at matched `s_eng`, `s_rel` = 1.06 / 1.12 / 1.42 for
const-`m'` / `τ_p`=0.2 / 0.4). But rung 48's finding 1 (the ratio rises monotonically through
the ramp) means the leg cannot release before the ramp flattens, while the minima are
ramp-driven and thus always INSIDE the ramp (48%/80% at r=0.5, 32%/64% at r=2.0, both 93% at
r=0.15). So the release is always post-ramp, always downstream of both minima, for any `r`.

**What WOULD reopen it:** a limiter whose engagement AND release both land inside the ramp —
a **rate-limited or lead-lag/washout-filtered `pt3`**, not a pure lag. That is the live seam
now. Also untested: the `τ_p`×`τ_gov` 4-state composite (the crossing rule predicts min-select
is the whole interaction — a cheap falsification target).

**Gotcha:** the sensor state is STIFF. `ds/τ_p ≳ 3` breaks RK4 and the truncated march reports
a large SPURIOUS relief (`ν_end` 0.79 vs 0.959). Filter every row to `ds/τ_p ≲ 2` — and check
`ν_H_end` FIRST on every row, before reading any relief number.
