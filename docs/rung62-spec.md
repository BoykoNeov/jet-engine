# Rung 62 — THE BLEED SCHEDULE beside the STATOR SCHEDULE: a schedule's loop has a SIGN

Rung 61 put rung 42's bleed valve and rung 53's variable stator on one **steady** machine
and closed by naming this rung:

> *"A `b(n_L)` schedule beside a `v(n)` stator schedule, on the **transient plant** — the one
> question this rung's steady answer makes sharp, because § 2 says the two devices' **costs**
> do not share and § 1 says their **credits** do not stack."*

Both halves are answered, and the answer to the first is not the one the seam expected.

> **THE HEADLINE — a state-fed schedule closes a FEEDBACK LOOP on itself through the shaft
> speed it reads, and that loop's SIGN is the sign of the lever's own `dn/d(setting)`.
> Rung 57 measured its stator schedule SURRENDERING 17–23 % of its own authority. The same
> instrument, the same plant, the same ramp and the same schedule form give a bleed schedule
> `FULL/RAMP` = **1.09–1.10**: it **AMPLIFIES** itself. Both signs were derivable from
> published tables before either was measured — and the two loops, closing through ONE
> state, do NOT compose: a bleed schedule TRIPLES the stator schedule's surrender
> (0.169–0.229 → 0.633–0.724) while the stator leaves the bleed's amplification alone to
> within 0.7 %. A ONE-WAY arrow, running from the amplifying lever to the cancelling one.**

Four sections. § 0 is the plant, and it is not furniture — it contains the rung's one silent
wrong number.

---

## 0. THE PLANT — and the touch point that converged on the wrong residual

**Rung 61 needed no new solve; this rung does.** Rung 61 composed by MRO alone because both
its parents sat on the steady cascade. Rung 42's valve lives in `_cascade_bleed` — and rung
40 **removed** that shaft balance, deliberately, to make the two power residuals the ODE
right-hand sides. The transient ladder never calls it. So the valve is threaded through the
FORWARD closure, at five sites:

| # | site | what changes |
|---|---|---|
| 1 | `_close` | `m_hp` referral ×(1−b); `m_imp` ÷(1−b) |
| 2 | `_close_fuel` | the same, **plus** `f = ṁ_fuel / CORE air` — the burner never sees the dumped air, so metered fuel makes a **richer** mixture |
| 3 | `_powers` | `Pt_lp` ×(1−b); `Φ_lp` on FACE air, `Φ_hp` on CORE air |
| 4 | `_instant_tail` | the same, plus rung 42's (3) thrust booking |
| 5 | `__init__` | the design capture stays at `b` = 0 — rung 42's shut-at-design discipline, asserted |

**Touch point 3 was missed on the first pass, and it is the exact silent-wrong-number class
rung 61 documented.** Rung 40 factored `(Φ_L, Φ_H)` out of `_instant_tail` into `_powers` so
the equilibrium Newton would not rebuild the nozzle each step. Left bleed-free, the Newton
**converged to 1e-12 on a residual the plant does not use**: it returned `n_L` = 0.8720 where
the true root is 0.8282 — **5.3 % wrong** — with `φ_L` still agreeing to 1e-3 and **no
exception anywhere**. No consistency check internal to the transient ladder can see it.

### The reduce — TWO-AXIS, and the corner with no ancestor

```
    (v = 0, b = 0)   =>  rungs 43-52 bit-for-bit   (both dispatches fall through)
    (v != 0, b = 0)  =>  rung 57 bit-for-bit       (rung 57's own body runs verbatim)
    (v = 0, b != 0)  =>  NO TRANSIENT ANCESTOR
```

The dispatch is **per call** on the *live* `b`, not on a constructor flag, so a `b_max` = 0
schedule returns to the parent at every state rather than multiplying by (1−0.0) — the
machinery is witnessed inert, not merely arithmetically neutral. Rung 57's `_close` /
`_close_fuel` / `_powers` / `_instant_tail` are left **literally unchanged**.

The third corner is validated the way **rung 40 validated itself** — the forward
`equilibrium` must reproduce rung 42's steady `TwoSpoolBleedMatcher.match`, **through the
forward closure only, never by calling that matcher**:

| `Tt4` | `b` | `n_L` fwd | `n_L` steady | rel |
|---|---|---|---|---|
| 1500 | 0.10 | 0.976004 | 0.976004 | 1.1e-12 |
| 1200 | 0.10 | 0.828181 | 0.828181 | 1.3e-13 |
| 1000 | 0.10 | 0.733632 | 0.733632 | 1.7e-13 |

Worst over 3 throttles × 3 valve positions: **6.1e-12**, against **5.4e-2** before the fix.

**Scope, pre-checked** (rung 42 warns its choked guard "bites SOONER" with the valve open,
and a handling schedule is *most* open at idle): choked at every `b` ≤ 0.30 down to
`Tt4` = 900. The ramp is rung 57's own, 1000 → 1400, unchanged.

## 1. THE HEADLINE — the loop gain's sign

Rung 57's third law: a state-fed `v(n)` **self-cancels**, `FULL/RAMP` = 0.754–0.896, because
closing the stators raises the speed at fixed power, the schedule reads the higher `n` and
opens back up. Written as a loop gain through the shared speed state,

```
    G = (dn_L/d·) x (d·/dn_L)          stator:  (+) x (−)  <  0
```

Both factors for a handling bleed were **already published**. Rung 61 § 2's own
factorisation attributes **−9.77 %** to *"bleed's lower `τ_c`"* ⇒ `dn_L/db` < 0; a valve open
at low speed has `db/dn_L` < 0. Product **positive**. Measured on the steady running line,
with no reversal anywhere in the band — which had to be checked, because rung 42's *own*
`dφ_H/db` passes through zero at `π*` = 3.24674 and reverses below:

| `Tt4` | `dn_L/db` (`v`=0) | `dn_L/db` (`v`=0.20) | `dn_L/dv` |
|---|---|---|---|
| 1500 | −0.23996 | −0.13163 | +0.65312 |
| 1300 | −0.23463 | −0.14520 | +0.51994 |
| 1100 | −0.22670 | −0.15355 | +0.40935 |
| 900 | −0.21399 | −0.15435 | +0.31995 |

And the loop itself, same `n_lo`, same ramp, same schedule form:

| lever | `r`=0.25 | `r`=0.50 | `r`=1.00 |
|---|---|---|---|
| stator `v_max` = 0.20 | 0.8313 | 0.7894 | 0.7709 |
| **bleed `b_max` = 0.10** | **1.0989** | **1.0971** | **1.0930** |

### The loop, witnessed directly rather than as a ratio

`FULL/RAMP` is a ratio of two marginal credits. The loop itself is visible without any
normalisation, in **the setting each schedule commands at its own surge minimum**:

| lever | `r` | cmd on RAMP | cmd on FULL | |
|---|---|---|---|---|
| stator | 0.25 | 0.11997 | 0.09797 | commands **LESS** — backs off |
| stator | 1.00 | 0.12733 | 0.10833 | commands **LESS** |
| bleed | 0.25 | 0.06582 | 0.07092 | commands **MORE** — leans in |
| bleed | 1.00 | 0.07310 | 0.07668 | commands **MORE** |

driven by the head start's own sign: the stator raises the armed idle (0.7557 → 0.7993), the
bleed lowers it (0.7557 → 0.7370).

### The PLACEMENT artifact — found, published, fixed

The first measurement used rung 57's own `n_lo` = 0.75574, the *bare* machine's idle speed.
That is right for the stator, whose head start moves `nu0_L` **UP**, *into* the active band.
It is wrong for the bleed, whose head start moves it **DOWN**, *below* `n_lo`, where `S`
clips to 1 and `b ≡ b_max`: **`db/dn` = 0 and there is no loop left to measure.**

| `n_lo` | stator `FULL/RAMP` | bleed `FULL/RAMP` |
|---|---|---|
| 0.75574 (bleed clipped) | 0.8029 / 0.7563 | 1.0480 / 1.0378 |
| 0.70 | 0.8119 / 0.7578 | 1.0916 / 1.0828 |
| **0.65** | **0.8313 / 0.7709** | **1.0989 / 1.0930** |
| 0.60 | 0.8531 / 0.7857 | 1.0946 / 1.0925 |

The **sign** survives the bad placement; the **magnitude halves**. The gate asserts the
saturation *exactly* — at the bad placement `sched(nu0_armed) == b_max` identically — rather
than through an arithmetic proxy.

Robustness: `shape="linear"` (no C¹ flat spot) gives 0.812–0.887 and 1.073–1.079. Grid
`ds` = 0.02 / 0.01 / 0.005 moves both ratios by **< 0.1 %**.

## 2. THE SECOND FINDING — two loops through one state do NOT compose, and the arrow is ONE-WAY

The pre-registered prediction (P3) was that the bleed's *positive* loop would **restore**
part of what the stator's negative loop surrenders. It is **refuted, with the opposite
sign**, and the refutation is this section.

The test has to carry the neighbour on **both sides** of the difference — comparing a pair's
composite `FULL/RAMP` against the two singles' does not test it, because the composite is a
credit-weighted blend of two different quantities.

**The stator schedule's OWN marginal surrender (1 − `FULL/RAMP`), by neighbour:**

| neighbour | `r`=0.25 | `r`=0.50 | `r`=1.00 |
|---|---|---|---|
| (none) | +0.1687 | +0.2106 | +0.2291 |
| constant `b` = 0.0709 *(level-matched)* | +0.2265 | +0.2546 | +0.2629 |
| constant `b` = 0.10 *(over-matched)* | +0.2471 | +0.2685 | +0.2720 |
| **SCHEDULE `b_max` = 0.10** | **+0.7241** | **+0.6839** | **+0.6333** |

**The mirror is nearly inert.** The bleed schedule's own ratio beside a stator: 1.0989 →
1.0971 (schedule) / 1.0945 (constant), and 1.0930 → 1.0852. **Within 0.7 %.**

### It is the LOOP, not the LEVEL — the control that makes it mean anything

A constant valve position has no loop of its own. The schedule commands only **0.0709 /
0.0740 / 0.0767** at its own surge minimum (trajectory mean 0.034–0.038), so the naive
constant-`b`=0.10 leg was comparing against *strictly more lever*. Matched at the schedule's
own commanded value it reaches +0.2265, and even over-matched at `b_max` only +0.2471:
**the schedule does 2.3–2.9× the damage with 23–29 % less lever.**

### The MECHANISM, measured — and the algebra that is NOT claimed

Armed idle `nu0_L` at `n_lo` = 0.65:

```
    bare 0.755741    stator 0.799323    bleed 0.736987    PAIR 0.790243
    stator +0.043582   bleed -0.018753   ADDITIVE +0.024829   PAIR ACTUAL +0.034503
    the stator's marginal head start is +0.053256 on top of bleed-only,
                                    vs  +0.043582 on top of bare        --  22 % LARGER
```

As the stator raises `n`, the bleed schedule **closes**, which raises `n` further; the stator
schedule reads that enlarged excursion and closes back harder.

> **The small-signal "two loops through one state MULTIPLY rather than add" statement is
> NOT claimed.** The measured gap (pair 0.8626 against an additive 1.0058) is consistent
> with it and with other readings, and this project's own precedent — rung 61's *"a
> 'derived' scaling whose binding constant is mine is not derived"* — applies. What is
> claimed is the one-way arrow, the level-matched control, and the 22 % enlargement.

**Why a caveat and not a clean sweep:** the minima do relocate (rung 50's question).
`s_min` on the reference march moves 0.190 → 0.220 (`r`=0.25) and 0.270 → 0.350 (`r`=1.00)
when the bleed schedule is armed, and the RAMP/FULL separation widens from 0.01 to 0.07.
Relocation is present in **every** leg including the constant-`b` controls, whose surrender
stays at 0.25, so it does not track the effect — but it is a consequence of the loop rather
than an independent check, and it is disclosed as one.

## 3. CORRECTION of rung 61 — the steady near-additivity was the SHAFT BALANCE's doing

Rung 61 measured these two devices' credits **additive to ≤ 2.3 %** and called them
*"substitutes drawing on one incidence budget"*. On the transient, over six
`(v_max, b_max, n_lo)` combinations and two map shapes, the same pair is **sub-additive by
9.1–29.1 %** — an order more:

| `v_max` | `b_max` | `c_stator` | `c_bleed` | `c_pair` | sum | `i`/sum |
|---|---|---|---|---|---|---|
| 0.20 | 0.10 | +0.03780 | +0.09177 | +0.10467 | +0.12957 | **−0.192** |
| 0.10 | 0.05 | +0.02212 | +0.04435 | +0.06043 | +0.06647 | −0.091 |
| 0.30 | 0.15 | +0.04921 | +0.14172 | +0.13529 | +0.19093 | −0.291 |

**The mechanism is the one rung 57 already identified, applied to a different claim.** Rung
40 removed the shaft balance that a steady matcher uses to re-solve `n_H` and absorb a
lever's `Tt25` shift; on the transient the speeds are STATES and both levers act on the same
one. This is exactly the shape in which **rung 57 corrected rung 53's two exact zeros** —
*"the zeros were the SHAFT BALANCE's doing, not the map's."* Rung 61's near-additivity was
the same artifact, one currency over. Its claim is corrected in **scope**, not refuted: the
steady numbers still hold on the steady machine, and this rung reproduces its sign.

**Rung 61's adverse SPEED-cost interaction survives** the transplant — positive in all 12
rows measured (`shaped` +0.00223 / +0.00190 / +0.00144, `tilted` +0.00205 / +0.00174 /
+0.00130). It is published **raw**: `cost_bleed` is negative while `cost_stator` is positive,
so a normalised interaction would carry a difference of opposite-signed terms in its
denominator — rung 43's currency-circularity trap.

## 4. The CONTROL that is explicitly not a finding

A constant setting's credit per unit setting, over a 20× ramp-rate range:

| `r` | bleed credit/`b` | stator credit/`v` |
|---|---|---|
| 0.10 | 1.42551 | 0.35616 |
| 0.25 | 1.29335 | 0.34587 |
| 0.50 | 1.24407 | 0.34570 |
| 1.00 | 1.21573 | 0.34644 |
| 2.00 | 1.20043 | 0.34720 |

**The signature is MONOTONICITY, not the size of the swing.** The stator's series is
**non-monotone at the 0.4 % level** (rung 57 says as much in its own words); the bleed's is
**strictly monotone decreasing at every one of the five rates**. So rung 57's ramp-rate
invariance is a **wall-mover** property: the floor channel contributes exactly `v` whatever
the trajectory does, while a point-mover's entire credit runs through `φ` and inherits the
trajectory's own ramp-rate dependence.

> **This CONFIRMS rung 57's published mechanism and is billed as a control.** Rung 57 § 2
> already states it — *"both channels are algebraic in the instantaneous state … neither has
> anywhere to put a clock."* The rung's first framing was to sell "a bleed schedule has no
> clock either" as a correction of rung 57's *attribution*; that was **refused before any
> code was written**, because it is rung 57's own sentence. What is genuinely new is only
> the complementary case — the project's first continuous, edgeless, **non-fuel**
> point-mover, all of rungs 46–52's being min-select fuel limiters with edges.

---

## Verification gates (`tests/test_rung62.py`, 57 assertions)

1. **REDUCE, TWO-AXIS and per CALL** — `b` = 0 is rung 57 bit-for-bit on 15 keys at three
   throttles, for a constant `v`, a scheduled `v` and no `v`; on the `Tt4`-pinned closure
   AND on the fuel closure; and a `b_max` = 0 schedule **dispatches** (asserted by the
   absence of the bled dict's own key) rather than computing unit factors.
2. **THE PLANT GATE** — the forward `equilibrium` reproduces rung 42's steady match to
   1e-9 at 3 throttles × 2 valve positions, plus a direct `_powers` ≡ `_instant_tail`
   witness so a future edit cannot restore one and not the other.
3. **THE `at_stator` TRAP** — rung 61's `at_setting` trap one ladder over: a sibling must
   carry this machine's valve, or every inherited rung-57 reader would difference an armed
   machine against a VALVE-SHUT bare one.
4. **THE HEADLINE** — both loop factors signed and non-reversing at four throttles; the two
   schedules strictly on opposite sides of 1.0 at three ramp rates, in published bands; the
   **commanded setting** moving oppositely; survival of shape and of grid refinement; and
   the **saturation artifact** gated *exactly* (`sched(nu0_armed) == b_max` at the bad
   placement, `< b_max` at the good one).
5. **THE SECOND FINDING** — the bleed schedule more than doubles the stator's surrender and
   drives it past 0.60; the mirror stays within 2 %; **the level-matched control** (matched
   *and* over-matched constants, ordered, with the schedule beating the larger constant by
   > 2.2×); and the 22 % head-start enlargement.
6. **CORRECTS RUNG 61** — sub-additive on two shapes × three rates, clearing rung 61's
   steady 2.3 % by more than 3×; the adverse **raw** speed interaction positive throughout.
7. **THE CONTROL** — the bleed's credit/setting strictly monotone, the stator's not.
8. **CYCLE UNTOUCHED** — the default single-spool design run is bit-for-bit rung 6.

## Concessions

* **The valve is an IMPOSED position, not a controlled one** — rung 42's own disclaimer.
  `b(n_L)` says where it sits; nothing schedules it against a measured margin. A
  **φ-referenced** bleed limiter would have edges, hence a clock, hence is a different rung
  (and rung 60's tautology plus the φ-rate negative bound what it could say).
* **`n_lo` is a placement, and it is load-bearing** — § 1 publishes the sweep rather than
  hiding it. The sign is placement-robust across 0.60–0.756; the magnitude is not.
* **No LP/HP symmetry for the valve.** The bleed schedule reads the LP's true corrected
  speed; there is no HP analogue and none is offered — rung 42 showed the valve is a degree
  of freedom on the LP spool and **not** the HP, and rung 57's HP-schedule concession
  (reading `nu_H` rather than `n_H`) would compound it.
* **The flat-η island is out of scope for the march**, tripping rung 57's own documented
  off-map guard. Rung 57 published no flat-η march either, so the two η-toggle controls that
  rungs 53/57/61 used are unavailable here and no η-mediation claim is made.
* **The dumped air carries full ram drag and returns no exhaust momentum** — rung 42's (3),
  the conservative booking, inherited unchanged; `sp_thrust` stays core-referenced (so `b`=0
  is bit-for-bit) with `sp_thrust_inlet` beside it.
* **No head-to-head against the fuel-side limiters** — rung 57's currency concession,
  inherited and now doubled: fuel withheld, shaft speed paid and air dumped have no common
  currency.
* Inherited unchanged: `φ_surge` is still rung 36's imposed constant, `eta_c_at` is still
  stator-inert, fully-choked branch, both NGVs choked, one `η_m`, no bypass, rung 35's
  forward-burner gas concession, no customer/cooling bleed.

## What it does to its neighbours

* **Rung 61 — CORRECTED in scope.** Its ≤ 2.3 % superposition is a steady-matcher property;
  the shaft balance it rests on was removed by rung 40. Its adverse speed-cost interaction
  survives.
* **Rung 57 — EXTENDED, and explicitly not re-claimed.** Its ramp-rate invariance is shown
  to be a *wall-mover* property by supplying the complementary case; its stated mechanism
  (algebraic-in-state) is confirmed, not corrected. Its `FULL/RAMP` table is reproduced
  exactly as this rung's control.
* **Rung 42 — its steady match becomes a transient VALIDATION ORACLE**, which is what
  caught the `_powers` defect.
* **Rungs 46–52 — untouched.** No clock claim is made in either direction beyond rung 57's.

## The next seam

**FUEL + BLEED** — the half of rung 61's seam this rung does not take. Every fuel-side leg
(46–52) lives on `integrate_fuel`, and the bleed now sits in the same closure, so the
composite is one constructor away; § 2 makes it sharp, because a fuel leg is a min-select
with **edges** while the bleed schedule is continuous and self-amplifying, and § 2 says a
positive loop does not leave its neighbour's loop alone. Beyond that: a **φ-referenced
bleed limiter** (a *controlled* valve, where rung 60's floor tautology and the φ-rate
negative both have something to say), and rung 61's other opening — the stator+bleed pair on
the **station-3 customer bleed**, a different sink with a different arrow.

## Anchor

`docs/plans/rung62-anchor-bleed-schedule.md` — the framing that was blocked before any code
(§ 0), the five predictions as written before measuring, and their scoring:
**three of nine wrong, and the two that produced this rung's content are P3 (refuted with
the opposite sign) and P5 (missed, and the miss restates as a monotonicity).**
