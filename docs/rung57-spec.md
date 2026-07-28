# Rung 57 — the STATOR SCHEDULE on the TRANSIENT plant: a wall-moving lever has no CLOCK

Rung 53 opened with the observation that every surge lever the project had built moved the
OPERATING POINT against a fixed wall, and shipped the first one that moves the WALL. But it
shipped it on the **steady** matcher. Rung 56 closed the steady side of that ladder and named
this rung in the open list:

> *"A **stator schedule `v(n)` on the TRANSIENT plant** (the first lever that could move the
> wall *during* an accel — now with a row COUNT as well as a setting)."*

Meanwhile the transient ladder had spent seven rungs (46–52) on fuel-side levers, and every
single one of them turned out to be credited by a **clock**: rung 48's engagement time, rung
49's two edges answering to two different clocks, rung 50's relocation of both minima to the
release edge, rung 51's release rate, rung 52's self-pinned trigger. That is a large, coherent
body of law. This rung asks whether any of it is about *limiters* — or only about levers that
move the point.

---

## THE HEADLINE

> **A lever that moves the OPERATING POINT is credited by WHEN it acts. A lever that moves the
> WALL is credited by WHERE the machine is — and that is a MAP property, with no memory and no
> clock. The engagement-timing family of rungs 46–52 is a property of point-movers, and it does
> not generalise.**

Measured, on the LP spool, at a constant setting `v` = 0.20, over a **20× ramp-rate range**:

| `r` | bare `M_i` | credit | credit/`v` | erosion |
|---|---|---|---|---|
| 0.10 | +0.33150 | 0.07123 | 0.35616 | 0.6438 |
| 0.25 | +0.41323 | 0.06917 | 0.34587 | 0.6541 |
| 0.50 | +0.45847 | 0.06914 | 0.34570 | 0.6543 |
| 1.00 | +0.48692 | 0.06929 | 0.34644 | 0.6536 |
| 2.00 | +0.50400 | 0.06944 | 0.34720 | 0.6528 |

**The margin the lever is credited against swings 52.0 %. The share of the lever's rotation
that survives moves 1.05 points.** The dynamics dominate the margin and are nearly inert to the
lever — which is the whole content, and the reason P2 exists as a separate gate. On the
`tilted` shape the same two numbers are **61.8 %** and **2.56 points**.

*Read against the anchor doc, this table replaces one that said something else.* The
contaminated probe C reported a **monotone 6-point** erosion drift over the same sweep; that
drift was an artifact of marching the scheduled leg from the **bare** equilibrium. Measured
consistently — and off a **constant** setting, which needs no schedule definition at all — the
drift is 1.05 points and **not** monotone. The 6-point number is superseded, not a second
result.

And rung 53's **design-point** closed form predicts the surviving share:

```
    dM_i/dv|design = 1/(2+l)         (rung 53's Jacobian, at phi_op = 1, n = 1)
```

`1/(2+0.7)` = **0.37037** against a measured 0.3457–0.3562 — **within 3.9 %**, at an operating
point that is nowhere near design (`φ_op` ≈ 0.67–0.76, `n` ≈ 0.78) and on a trajectory that is
not a steady state at all. On `tilted` (`l` = 0.85) it is **within 2.2 %**.

---

## The second law — two thirds of the rotation never arrives

The stator lowers the wall and moves the running line **with the same coordinate** — rung 53's
own physics ("a variable stator cannot move the floor without moving the running line"). Split
the credit:

```
    pointwise   the FLOOR channel alone: the BARE trajectory read against the ARMED wall.
                For a constant v this is EXACTLY v, since M_i = T_c - 1/phi + v at frozen phi.
    net         the real credit: ARMED trajectory, ARMED wall.
    erosion     1 - net/pointwise — the share the lever's own WORK channel eats.
```

**Erosion is 0.63–0.66 everywhere** — across both shapes, all five ramp rates, and (on rung
53's steady matcher, at the same throttles) 0.652 at `Tt4` = 1000 rising to 0.675 at 1500. A
20° stator rotation buys about a **third** of its nominal incidence, and what sets the fraction
is nothing but the map's loading slope `l`.

This is why the headline is a *map* statement rather than a *dynamics* statement: both channels
are algebraic in the instantaneous state. The floor channel has no memory by construction; the
work channel's erosion is the local Jacobian at the operating point. Neither has anywhere to
put a clock.

---

## The third law — a state-fed schedule SELF-CANCELS

A constant setting is rung 53's lever transplanted. The thing a real engine implements is a
**schedule** `v(n)`: closed at low corrected speed, open at design speed. That is state
feedback, and it is the one thing a constant cannot be. Three legs on one ramp
(`credit_decomposition`), schedule `v_max` = 0.20:

| `r` | FULL | START-ONLY | RAMP-ONLY | share START | FULL/RAMP |
|---|---|---|---|---|---|
| 0.10 | +0.05537 | +0.01498 | +0.06178 | 0.271 | 0.896 |
| 0.25 | +0.05111 | +0.00611 | +0.06366 | 0.120 | 0.803 |
| 0.50 | +0.05272 | +0.00156 | +0.06857 | 0.030 | 0.769 |
| 1.00 | +0.05465 | −0.00163 | +0.07226 | −0.030 | 0.756 |
| 2.00 | +0.05639 | −0.00384 | +0.07478 | −0.068 | 0.754 |

Two readings, and the second is the finding.

**It is not an initial-condition device.** A state-fed schedule is already closed at the low
speed the machine idles at, so it has acted before `s` = 0 — the armed machine starts at
`nu0_L` = 0.8166 against the bare 0.7557, which is rung 53's *"thrust-neutral, paid in SHAFT
SPEED"* showing up as an initial condition. That head start **alone** delivers 27 % of the
credit at the fastest ramp, 3 % at `r` = 0.5, and **goes negative** for slow ramps. The credit
is delivered *during* the ramp, locally, by the live wall.

**And the schedule surrenders 10–25 % of its own authority.** FULL is below RAMP-ONLY at every
ramp rate, and the gap deepens as the ramp lengthens. The mechanism is exactly the head start:
closing the stators raises the speed the machine sits at for the same power, **the schedule
reads that higher speed and opens back up.** A state-fed floor-moving lever is negative
feedback on itself.

Against a **constant setting matched at the schedule's own surge minimum**, the schedule's
residual is under 25 % of the credit. So the honest bound is: **this rung is about the LEVER,
not about scheduling it.** Rung 53's setting does most of the work; the schedule's content is
the self-cancellation, not extra margin.

---

## The cross-rung CORRECTION — rung 53's two exact zeros BREAK

Rung 53 § P5 measured, on the steady cascade and reported with `==`:

- **`vsv_lp` cannot reach the HP spool at all** — `Δφ_HP` and `Δn_HP` are *exactly*
  `+0.000e+00`. Rung 53 called the LP stator **"a pure-LP lever, bit-for-bit."**
- **`vsv_hp` reaches the LP only through `η_HPC`** — on flat-η islands `Δφ_LP` is *exactly*
  `+0.000e+00`; on shaped ones it is `−4.79e-03`.

The same toggle, at a **fixed transient state** (`nu_L` = 0.775960, `nu_H` = 0.806924,
`mf` = 0.015917 — the bare march's own LP surge minimum), `v` = 0.20:

| island | lever | `Δφ_LP` | `Δφ_HP` | `Δn_HP` | `ΔTt25` |
|---|---|---|---|---|---|
| shaped-η | `v_LP` | −6.647e-02 | **−9.608e-03** | +1.531e-02 | −13.05 K |
| shaped-η | `v_HP` | **−1.278e-01** | −1.653e-01 | −6.458e-03 | +5.72 K |
| flat-η | `v_LP` | −6.609e-02 | **−9.289e-03** | +1.573e-02 | −13.37 K |
| flat-η | `v_HP` | **−1.179e-01** | −1.617e-01 | −6.010e-03 | +5.31 K |

**Both zeros break, and neither breakage is η-mediated** — the flat-η island, which was rung
53's *own* control for zeroing the arrow, reproduces the shaped one to within 5 %.

**The zeros were the SHAFT BALANCE's doing, not the map's.** In the steady cascade `n_H` is
re-solved so that the LP stator's `Tt25` shift is absorbed; the arrow that survives is the
residual η one, which is why rung 53 found it switchable off. Rung 40 **removed** that balance
— deliberately, to make the two power residuals the ODE right-hand sides — so on the transient
`n_H = nu_H·√(Tt25_d/Tt25)` is a state read against a *moved* `Tt25`, and the arrow opens
through the energy channel. `ΔTt25` = −13.05 K names it.

It remains a **minor** arrow (14 % of the lever's own LP effect), so rung 53's per-spool
picture survives as an approximation. What does not survive is the word *exactly*.

---

## The instrument

`StatorSchedule` + `ScheduledStatorTransient` in `turbojet/engine.py`.

```
    v(n) = v_max * S( (n_ref - n)/(n_ref - n_lo) )        S clipped to [0, 1]
```

CLOSED at low corrected speed, **exactly 0 at and above the design speed `n_ref`** — asserted
in `__post_init__`, not relied on, because the whole hardware capture (`A4/A45/A8`,
`mcorr_*_d`, `tau_*_d`) is taken at `v` = 0 (rung 53's discipline) and a schedule holding a
nonzero setting there would silently contradict every design reference.

`shape="smooth"` (`x²(3−2x)`, C¹ at both corners) is the default and it is **not cosmetic**:
the schedule's kink lives in **STATE** space, so rung 50's *"pass the switch on the `ds` grid"*
trick is structurally unavailable — you cannot align a state-space corner with a time grid.
`shape="linear"` is carried only as a C⁰ shape-robustness control.

The class arms two ways, mutually exclusive per spool: `vsv_lp/vsv_hp` (a **constant** setting,
rung 53's lever transplanted, applied once at construction) and `vsv_sched_lp/vsv_sched_hp` (a
schedule, read off the live state at every closure). `_arm` is a **pure function of
(`nu_L`, `nu_H`, `Tt2`)** — no history, no latch — so it is RK4-legal exactly as rung 50's
`s`-threading was.

Readers: `stator_transient_margin` (both currencies, per spool), `stator_credit` (the credit
and its erosion), `credit_decomposition` (START / RAMP / FULL), `arrow_toggle` (rung 53's P5,
transplanted). They are **new** readers, deliberately: `phi_excursion_fuel` and
`transient_surge_margin_fuel` read the FIELD `phi_surge`, which rung 53 pinned to the design
setting so rungs 41/44/45's readers stay literally unchanged. Under a moving stator that field
is the wrong wall.

### The reduce — dispatch AND identity

With no schedule armed, `_arm` returns on its first line and both closures run the inherited
rung-40/43 bodies with the maps untouched: **bit-for-bit rungs 43–52** on every recorded key. A
schedule whose `v_max` is 0.0 returns 0.0 at every `n`, at which point `_arm` hands back the
**same map object** (`is`, not `==`) — so the swap machinery is witnessed *inert*, not merely
*skipped*. Both are gated.

---

## The defect this rung found (fixed, scoped, gated)

The rung-40/43 closures bracket the LP flow at `hi = min(2.5, φ_max,LP·n_L)` — **the LP map's
own limit**, with nothing bounding where that puts the **HP face**. At `φ_L` = 2.11 the HP face
sits at `φ_H` > 4, far outside its own map: `ψ_H` = −3.09, `τ_hpc` = −2.18, and **`Tt3` = −649 K**.
`gas.pr_c()` then raises a float to a fractional power on a negative base, and Python returns a
**complex**, which reaches `glo < 0.0 < ghi` as a `TypeError` — while every caller in the ladder
catches `AssertionError` only.

It survives today only by accident: with a *shaped* η island `eta_c_at` collapses at `φ_H` ≈ 4
and cancels the negative enthalpy back to a positive one, so the endpoint stays real (if
physically meaningless) and the bracket works. On a **flat-η** island nothing cancels it.

The fix is one assertion in each closure's `g`, converting a non-real residual into the
ladder's documented off-map `AssertionError`. It **changes no number**: every real-valued
evaluation — including the nonsense high wall the shaped maps rely on — passes straight
through. Gated by `test_offmap_guard_is_an_assertion_not_a_typeerror`.

---

## Concessions

- **The HP schedule reads `nu_H`, the HP SHAFT speed, not its corrected speed.** `Tt25` is an
  OUTPUT of the very root the schedule must be armed before, so a true `n_H` schedule would
  need the closure re-implemented rather than wrapped. They coincide at the design point, and
  **every load-bearing claim above is LP-side** — which is also where rungs 41/44/45 put the
  exposure.
- **`eta_c_at` is stator-INERT.** The efficiency island still peaks at (`φ`, `n`) = (1, 1)
  whatever the stators do. Rung 53 disclosed the `σ` term's stator-inertia; this is a **second**
  one, and it bites harder here because a displaced running line puts `η_c` straight into
  `π_LPC`. Sign: a closed stator pushes `φ_op` **down**, off the island's peak, so the modelled
  `η_c` falls where a real re-staggered row's would partly recover — the model therefore
  **understates** the credit, making the erosion reported above a conservative bound.
- **The schedule's own residual over a matched constant is small** (< 25 % of the credit), so
  no claim is made that scheduling beats setting. That bound is gated, not assumed.
- **No head-to-head against the fuel-side limiters.** Fuel withheld and shaft speed paid have
  no common currency — rung 48's matched-accel-time trap and rung 43's currency circularity.
  The comparison in the headline is about the *structure* of the credit (clock vs map), not
  about which lever is better.
- **`φ_surge` is still rung 36's imposed constant.** It anchors `T_c = 1/φ_surge`, so its LEVEL
  is disclaimed exactly as in rungs 36/41/53; the load-bearing objects here are ratios and
  invariances, which ride on the running line rather than on that constant.

---

## What it does to its neighbours

- **Rungs 46–52 — BOUNDED, not refuted.** Their engagement-timing law is intact for the levers
  it was measured on. What rung 57 shows is that it is a law about *point-movers*, so it must
  not be quoted as a law about *limiters*. This is the same shape as rung 53's bounding of the
  `φ`-currency: rung 53 bounded rungs 36–52's **currency**, rung 57 bounds their **clock**.
- **Rung 53 — CORRECTED (P5) and EXTENDED (the Jacobian).** Its two exact zeros are steady-state
  properties of the shaft balance, and break on the transient without the η channel it named.
  Its design-point Jacobian, on the other hand, turns out to predict the transient credit off
  design and out of equilibrium to a few percent — a considerably stronger claim than it made.
- **Rung 40 — a latent defect closed**, in the closure it introduced.

---

## The next seam

**The stator schedule beside the FUEL-side limiters, on ONE plant.** This rung deliberately
arms nothing else — the constructor is orthogonal to `integrate_fuel`'s eight legs, and the
findings above are all measured on the bare fuel ramp. But a real FADEC runs the VSV schedule
*and* the accel schedule *and* the topping governor together, and rung 57's result makes the
composite interesting rather than routine: one lever in the min-select is clocked and the other
is not, so the pair cannot factorise the way rung 52 § 3 already said a two-lag cascade cannot.
The obstacle is a currency, not code — see the Concessions.

Then, unchanged from rung 56's list: **stator + bleed together** (rung 53's saturation says the
bleed takes over where the stator's authority ends), a **bleed schedule** `b(n_L)`, and the
**lag SHAPE / two-lag cascade** (rung 52's own seam).

## Anchor

`docs/plans/rung57-anchor-stator-schedule.md` — the probes that fixed the instrument and the
sign *before* the predictions (including probe E, which was **wrong** and is published as
such), the six predictions as written, and their scoring:
**P1 claim HIT / band MISSED on one shape, P2 HIT, P3 HIT, P4 HIT, P5 HIT (both halves),
P6 HIT.**
