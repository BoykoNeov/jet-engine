# The lagged / filtered `pt3` sensor — INVESTIGATED, **NOT SHIPPED**

> **Status.** This is a **negative-result record**, not a rung spec. It is the attack on rung 48's
> own named next seam (`docs/rung48-spec.md` § Concessions — *"a lagged/filtered `pt3` sensor is the
> obvious next seam"*). It was derived, prototyped and measured, returned a negative verdict, and was
> **deliberately not added to the ladder** — no `*-spec.md`, no `engine.py`/`main.py`/test code, no
> rung-table entry, by design. It sits in the `docs/tau-res-negative.md` /
> `docs/mixing-scale-negative.md` / `docs/mixing-jicf-anchor-negative.md` family.
>
> **Why this file exists:** so the negative is not re-investigated from scratch, so its two positive
> by-products are not lost — rung 48's crossing rule **tested on a second, independent instrument
> with a genuine negative control**, and rung 48's stated seam **corrected in sign** — and so the one
> measurement that decides it is on record. **If you are about to build a `τ_p` sensor lag on the
> `Wf/pt3` leg, read this first — it was done.**

## What this investigation was

Rung 48 shipped the feedforward `Wf/pt3` acceleration schedule and found the unifying rule: **a
fuel-side limiter rebates a spool IFF it engages UPSTREAM of THAT spool's own surge minimum.** Its
limiter reads `pt3` from the same closure the plant runs on — no sensor dynamics — and it closed on:

> *"A lagged/filtered `pt3` sensor is the obvious next seam, and rung 47's result says the interesting
> question is whether the lag pushes `s_eng` past `s_lp*` — i.e. the SAME crossing, now as a sensor
> question."*

This investigation gave the pressure measurement a first-order lag: a third state `q` with

```
dq/ds = (pt3_true − q) / τ_p        q(0) = pt3 at the initial equilibrium (a SETTLED sensor)
cap    = (1 + m) · κ_ss(n_H) · q    [the SENSED pressure, not the true one]
```

marched as `(ν_L, ν_H, q)` in a prototype that leaves `engine.py` untouched. Config throughout is
rung 48's: CPG gas, accel 1000→1400 K, `ρ = 1`, `r = 0.5`, bare `s_lp* = 0.240`, `s_hp* = 0.400`,
`ν_H_end = 0.95906`.

**The verdict is that a sensor lag is an effective-margin reparameterisation on the surge axis, and
rung 48's crossing rule absorbs it entirely.**

## The deflation was structural, and it is the whole story

Write the cap out:

```
cap(s) = (1+m)·κ_ss(n_H)·pt3(s) · [ q(s) / pt3(s) ]
```

The bracket is a **pure multiplicative scalar on the cap at every instant**. A sensor lag is therefore
*exactly* a **time-varying margin** `m'(s) = (1+m)·ρ(s) − 1` with `ρ = q/pt3 ∈ (0,1]` on a rising
accel. There is no new plant, no new signal, no new mechanism — rung 48's instrument already **is** a
scalar multiplier on the cap. So the only question worth asking was never *"does the lag help"* (it
does, and trivially: a lower cap engages earlier, and rung 48 already says engage-earlier ⇒ more
relief — that is a corollary of the shipped rung, not a finding). **The question is whether the
time-VARYING margin does anything a CONSTANT one cannot.** It does not.

## THE DECIDING MEASUREMENT — matched sub-grid engagement, refined in `ds`

Engagement is the zero of `slack(s) = mf_sched(s) − cap(state(s), mf_sched(s))` — exactly the `ghi`
that `_sched_fuel` tests. It is continuous in `s`, so its zero crossing interpolates to a **sub-grid**
`s_eng_exact`. Bisect the constant margin `m'` on **that** (not on the grid value), then compare the
LP relief:

| `m` | `τ_p` | `s_eng_exact` | matched `m'` | `Δrelief_lp` @ `ds=0.02` | @ `ds=0.01` |
|---|---|---|---|---|---|
| 0.35 | 0.20 | 0.13480 | 0.26913 | +0.4 % | +0.4 % |
| 0.35 | 0.40 | 0.12911 | 0.26006 | +3.0 % | **+0.3 %** |
| 0.45 | 0.20 | 0.18440 | 0.33957 | **+0.000000** | **+0.000000** |

**The reliefs collapse.** At matched engagement the sensor and a constant margin give the same LP
relief — exactly, in the `m = 0.45` case, at both step sizes. The one case that looked distinguishable
(+3.0 %) **shrank 10× under `ds` refinement**, which is the signature of discretization, not of a
mechanism.

**This measurement had to be done sub-grid.** A first pass matched on the recorded `s_eng` — quantized
to the `ds = 0.02` grid — and reported a 12 % gap. That gap was an artifact: rung 48's own `relief_lp`
moves ~33 % between `ds = 0.02` and `ds = 0.005`, so a 12 % difference sat **inside the discretization
band** and was evidence of nothing. Two configurations reporting the same grid `s_eng` had engaged at
genuinely different instants inside one cell. *Any future attack on this seam must match sub-grid and
refine in `ds`, or it will re-discover the same artifact.*

## The by-product worth keeping (1) — the rule tested with a real NEGATIVE CONTROL

Because the lag pushes `s_eng` **earlier**, it can be aimed at margins where rung 48 has `relief_lp`
**exactly 0** and asked to re-awaken them. It does — and only when it crosses:

| `m` | `τ_p` | `s_eng` | vs `s_lp*` | `relief_lp` | `relief_hp` | fuel removed |
|---|---|---|---|---|---|---|
| 0.48 | none | 0.400 | downstream | **0.000000** | +0.000016 | 0.00002 |
| 0.48 | **0.05** | **0.280** | **downstream** | **0.000000** | +0.006426 | **0.00030** |
| 0.48 | 0.10 | 0.240 | on the min | +0.000090 | +0.011881 | 0.00067 |
| 0.48 | 0.20 | 0.220 | upstream | +0.000691 | +0.016875 | 0.00136 |
| 0.45 | none | 0.320 | downstream | **0.000000** | +0.003385 | 0.00015 |
| 0.45 | 0.10 | 0.220 | upstream | +0.000691 | +0.015714 | 0.00097 |
| 0.42 | none | 0.280 | downstream | **0.000000** | +0.007493 | 0.00037 |
| 0.42 | 0.05 | 0.220 | upstream | +0.000691 | +0.014680 | 0.00085 |

**The `(m=0.48, τ_p=0.05)` row is the control.** The lag is armed, it has moved engagement a long way
earlier (0.400 → 0.280), and it is removing **15× more fuel** than the no-lag baseline — and
`relief_lp` is still **EXACTLY 0**, because 0.280 is still downstream of `s_lp* = 0.240`. The lag
itself buys the LP nothing; only crossing `s_lp*` does.

The matched-engagement table sharpens the same point from the other side: at `(m=0.35, τ_p=0.4)` the
sensor removes **57 % more fuel** than its matched constant margin (0.00578 vs 0.00369) and buys the
**same** LP relief to 0.3 %. **Fuel removed downstream of the minimum purchases nothing** — an
independent confirmation of rung 48's finding 3 (this is not rung 44's ramp-rate lever), reached on an
instrument rung 48 did not have.

Every row above is admissible: `ν_H_end` is 0.95906 / 0.95905 / 0.95904 against bare 0.95906 across
the whole sweep, so none of it is de-fanging.

## The by-product worth keeping (2) — rung 48's seam had the SIGN BACKWARDS

Rung 48 (and the CLAUDE.md seam line) asked whether the lag pushes `s_eng` **past** `s_lp*`. It pushes
it the **other way**. A first-order lag on a **rising** `pt3` reads **low**, so the cap is **lower**
than truth and the leg engages **EARLIER**:

| `m` | none | `τ_p`=0.05 | 0.10 | 0.20 | 0.40 | 0.80 |
|---|---|---|---|---|---|---|
| 0.42 | 0.280 | 0.220 | 0.200 | 0.180 | 0.160 | 0.160 |
| 0.45 | 0.320 | 0.240 | 0.220 | 0.200 | 0.180 | 0.180 |
| 0.48 | 0.400 | 0.280 | 0.240 | 0.220 | 0.200 | 0.180 |

This is the **opposite of rung 47**, where a loop lag pushed an already-late window later still — and
it is the reason the two lags are not the same object. Rung 47 lagged the limiter's **output trigger**;
this lags its **input measurement**. `docs/rung48-spec.md` § Concessions has been corrected.

## Why the one genuinely new degree of freedom cannot matter — a STRUCTURAL argument

A sensor lag does add something a constant margin cannot: `ρ(s) → 1` as the ramp flattens, so the
sensor **catches up** and the leg's **release** is set by `τ_p` independently of its engagement. That
is real and measurable — at matched `s_eng = 0.140`, `s_rel` is **1.060** (constant `m'`), **1.120**
(`τ_p = 0.2`), **1.420** (`τ_p = 0.4`): three releases from one start.

**It is nevertheless surge-irrelevant, and structurally so — not just at this config.** Rung 48's own
finding 1 is that the bare `(Wf/pt3)/κ_ss` ratio rises **monotonically through the ramp**. So once the
leg engages it **cannot** release until the ramp flattens. And both surge minima are **ramp-driven**,
hence always *inside* the ramp — at `r = 0.5` they sit at 48 % and 80 % of it, at `r = 2.0` at 32 %/64 %,
at `r = 0.15` both at 93 %. Therefore **the release edge is always post-ramp and always downstream of
both minima, for any `r`** — the region where, by rung 48's rule, it rebates nothing and only removes
fuel. Measured: `s_rel ≥ 0.76` even at the smallest `τ_p`, against `s_hp* = 0.40`.

The one new DoF is real, and it is aimed where nothing lives.

## Two loose ends, recorded so they are not re-derived

- **`τ_p → 0` converges to rung 48; it is not bit-for-bit.** At `ds = 0.005` (rung 48:
  `relief_lp = +0.001212`), `τ_p` = 0.08 / 0.04 / 0.02 / 0.01 gives `Δ` = +3.40e-3 / +1.83e-3 /
  +9.61e-4 / +3.18e-4 — monotone to zero, roughly first-order in `τ_p`. A shipped version would take
  `τ_p=None` as **exact dispatch** (bit-for-bit) and state `τ_p → 0` as convergence, rung 47's
  precedent for `tau_gov`. **The sensor state is stiff:** `τ_p = 0.002` at `ds = 0.02` is `ds/τ_p = 10`,
  far past RK4 stability — the march breaks early and reports a truncated trajectory that *looks* like
  a large spurious relief (`ν_end = 0.78993`). Filter every row to `ds/τ_p ≲ 2`.
- **The decel reduce holds.** On a falling `pt3` the sensor reads **high**, so the cap is *higher* than
  truth and the leg is even more dormant: at `τ_p` = 0.05 / 0.2 / 0.8 the leg engages at **0** points
  and the march matches the un-limited decel exactly (rung 48's gate 5, on the sensed leg).
- **A coincidence that is NOT a finding.** `relief_lp` repeats to 6 dp across several `(m, τ_p)` pairs
  sharing a grid `s_eng` (+0.000691 at 0.220 for three distinct pairs; +0.001818 at 0.200 for three).
  This is **not** "relief is a function of `s_eng` alone": `(m=0.35, τ_p=0.02)` and `(m=0.42, τ_p=0.2)`
  both report `s_eng = 0.180` and give +0.002155 vs +0.003485, and `relief_hp` does not repeat at all.
  It is the grid quantization again. Do not build a claim on it without a sub-grid measure.

## What WOULD reopen this

Not a bigger `τ_p`. The negative rests on two things, and only a change to one of them matters:

1. **A limiter whose engagement and release are BOTH inside the ramp.** The release edge is inert only
   because it is structurally post-ramp. A limiter that can release mid-ramp — e.g. a **rate-limited**
   or **washout/lead-lag filtered** `pt3` (not a pure lag), or a schedule with a non-monotone ratio —
   would put a second edge where a surge minimum lives, and the crossing rule would have two edges to
   be tested against rather than one. **That is the live version of this seam.**
2. **A `τ_p` × `τ_gov` composite** (the 4-state march, deferred here because no code shipped). Rung 48
   min-selects the feedforward leg with rungs 46/47's feedback one; whether an early-biased sensor lag
   and a late-biased loop lag interact beyond min-select is untested. The crossing rule predicts they
   do not — each leg is judged by its own engagement time — which makes it a cheap falsification target.

## Method note

Prototype and probes: `M:\claud_projects\temp\rung49-probe\` (`probe_pt3_sensor.py` — the 3-state
march and the `τ_p` sweep; `probe_crossing.py` — the re-awakening test and the `τ_p → 0` convergence;
`probe_matched.py` — the sub-grid discriminator and the decel reduce). No project file was modified by
the investigation except this doc, the rung-48 sign correction, and the CLAUDE.md status map.
