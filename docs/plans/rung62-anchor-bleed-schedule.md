# Rung 62 anchor — the BLEED SCHEDULE beside the STATOR SCHEDULE, on the transient plant

The seam rung 61 named, both halves:

> *"A `b(n_L)` schedule BESIDE a `v(n)` stator schedule on the transient (rung 61's seam);
> **fuel + bleed**."* — CLAUDE.md, still-OPEN list
>
> *"…the one question this rung's steady answer makes sharp, because § 2 says the two
> devices' **costs** do not share and § 1 says their **credits** do not stack."*
> — `docs/rung61-spec.md` § The next seam

This file is the **pre-registration**: the probes that fixed the instrument, the predictions
as written *before* measuring, and their scoring. Nothing here is edited after the fact —
where a prediction missed, the miss is published as a miss.

---

## 0. The framing the advisor blocked, and why it was wrong

The first framing was: *"a bleed schedule is a POINT-mover but is STATELESS, so if it has no
clock either, rung 57's attribution of 'no clock' to WALL-MOVING is corrected."*

**Refused, and correctly.** Rung 57's own § 2 (`docs/rung57-spec.md` lines 79–82) already
states the mechanism:

> *"…both channels are **algebraic in the instantaneous state**. The floor channel has no
> memory by construction; the work channel's erosion is the local Jacobian at the operating
> point. **Neither has anywhere to put a clock.**"*

So "the bleed schedule has no clock" is a **confirmation of rung 57's published mechanism**,
not a discovery. Billing it as the headline would have been the duplicate-identity failure
this project has now made three times (rung 61 twice, rung 60 once). It is carried below as
a **corroborating control**, explicitly labelled, and it is not the rung.

## 1. THE INSTRUMENT — and the five touch points, one of which was a silent wrong number

Rung 61 needed **no new solve**: both its parents sat on the steady cascade, so the pair was
the MRO and nothing else. **That does not repeat here.** Rung 42's valve lives in
`_cascade_bleed`, the steady two-shaft cascade — and rung 40 *removed* the shaft balance to
make the two power residuals the ODE right-hand sides, so the transient ladder never calls
it. The valve has to be threaded through the FORWARD closure:

| # | site | what changes |
|---|---|---|
| 1 | `_close` | `m_hp` referral ×(1−b); `m_imp` ÷(1−b) |
| 2 | `_close_fuel` | the same, **plus** `f = ṁ_fuel / CORE air` (the burner never sees the dumped air) |
| 3 | `_powers` | `Pt_lp` ×(1−b); `Φ_lp` on FACE air, `Φ_hp` on CORE air |
| 4 | `_instant_tail` | the same, plus rung 42's (3) thrust booking |
| 5 | design capture | stays at `b` = 0 — rung 42's shut-at-design discipline, **asserted** |

**Touch point 3 was missed on the first pass and it is exactly the silent-wrong-number class
rung 61 documented.** Rung 40 factored `(Φ_L, Φ_H)` out of `_instant_tail` into `_powers` so
the equilibrium Newton would not rebuild the nozzle each step. With `_powers` left
bleed-free the Newton **converged to 1e-12 on a residual the plant does not use**: it
returned `n_L` = 0.8720 where the true root is 0.8282 — a **5.3 %** error, with `φ_L` still
agreeing to 1e-3 and **no exception anywhere**. Probe A caught it only because it compared
against an independent object.

### The gate that caught it — and the corner with no ancestor

The two-axis reduce has a corner the ladder has never had before:

```
    (v = 0, b = 0)   =>  rungs 43-52 bit-for-bit   (per-call dispatch)
    (v != 0, b = 0)  =>  rung 57 bit-for-bit       (per-call dispatch)
    (v = 0, b != 0)  =>  NO TRANSIENT ANCESTOR
```

The third corner is validated the way **rung 40 validated itself**: the FORWARD
`equilibrium` must reproduce rung 42's steady `TwoSpoolBleedMatcher.match` at the same
`(flight, Tt4)` — **through the forward closure only, never by calling that matcher**.

| `Tt4` | `b` | `n_L` fwd | `n_L` steady | rel | `φ_L` rel |
|---|---|---|---|---|---|
| 1500 | 0.05 | 0.987777 | 0.987777 | 2.9e-14 | 7.3e-14 |
| 1500 | 0.10 | 0.976004 | 0.976004 | 1.1e-12 | 2.6e-12 |
| 1200 | 0.10 | 0.828181 | 0.828181 | 1.3e-13 | 2.6e-13 |
| 1000 | 0.10 | 0.733632 | 0.733632 | 1.7e-13 | 6.8e-13 |

Worst over 3 throttles × 3 valve positions: **6.1e-12**. Before the `_powers` fix: **5.4e-2**.

### Scope pre-check (rung 42 warns the choked guard "bites SOONER" with the valve open)

A `b(n_L)` schedule is *most* open at idle — exactly `Tt4_lo`. Checked before choosing the
ramp: the nozzle stays **choked at every `b` up to 0.30, at every `Tt4` down to 900**. No
scope hazard, and the ramp `Tt4` = 1000 → 1400 is inherited from rung 57 unchanged.

## 2. THE PREDICTIONS — written before the pair was measured

### P1 — the LOOP-GAIN SIGN (the headline). **Written before probe B ran.**

Rung 57's third law is a **self-cancellation**: `FULL/RAMP-ONLY` = 0.754–0.896, because
closing the stators raises the speed at fixed power, the schedule reads the higher `n` and
opens back up. Its loop gain is

```
    G = (dn_L/d·)(d·/dn_L)          stator:  (+)(−) < 0   =>  negative feedback
```

Both factors for the **bleed** are already published. Rung 61 § 2's factorisation attributes
**−9.77 %** to *"bleed's lower `τ_c`"* ⇒ `dn_L/db < 0`. A handling bleed is OPEN at low `n`
⇒ `db/dn_L < 0`. Product **POSITIVE**.

> **P1: a state-fed bleed schedule AMPLIFIES itself — `FULL/RAMP-ONLY` > 1 — where rung 57's
> stator schedule surrenders 10–25 %. Same instrument, same plant, opposite sign.**
>
> P1a: `dn_L/db` < 0 at **every** throttle 900–1500 (no reversal of the rung-42 `π*` kind).
> P1b: the commanded setting at the surge minimum moves the **opposite way** for the two
> levers between the RAMP and FULL legs — the loop, witnessed directly rather than as a ratio.
> P1c: the effect survives shape (`smooth`/`linear`) and grid refinement.

### P2 — the PAIR's credits. **Written before probe D ran.**

Rung 61 measured the two levers' steady credits as additive to **≤ 2.3 %** ("substitutes
drawing on one incidence budget"). Rung 58 measured a stator schedule beside a **fuel-side**
leg on this same transient plant and found they do **not** superpose (one-way, ~86 %).

> **P2: the two SCHEDULES superpose far better than rung 58's pair but worse than rung 61's
> steady ≤ 2.3 % — predicted four-cell interaction in `M_i` of 5–20 % of the credit sum,
> with the excess over rung 61 attributable to the shared speed state that the steady
> matcher re-solves away.** The reason rung 58's pair is the outlier is that its fuel leg is
> a **min-select** (a switching nonlinearity with edges); both levers here are continuous.

### P3 — the PAIR's LOOP. **The one this rung exists for.**

The two loops close through **the same state** `n_L`. They have opposite signs.

> **P3: the pair's `FULL/RAMP-ONLY` lands strictly BETWEEN the two singles, and specifically
> the bleed's positive loop partially RESTORES the authority the stator schedule surrenders
> to its own negative loop — a cross-lever effect in a quantity that is not the credit.**
> If it holds: **two levers that do not compose in the CREDIT can still compose in the LOOP**,
> because the loop is a property of the shared STATE, not of the wall.
>
> P3a: the stator's surrendered share (1 − FULL/RAMP) **shrinks** when the bleed schedule is
> armed beside it.

### P4 — the COST. Rung 61's steady result, transplanted.

Rung 61 found the pair's **speed** cost interaction positive in all 30 rows (+0.19…+1.81 %):
the pair always costs more shaft speed than the sum of its parts.

> **P4: the adverse speed interaction survives on the transient — the pair's peak `n_L`
> exceeds the sum of the two singles' — and the two devices' costs still do not share a
> currency (the stator is paid in speed, the bleed in thrust), so no scalar "which is
> better" is reported.**

### P5 — the CONTROL that is NOT a finding (see § 0).

> **P5: a CONSTANT `b`'s credit per unit `b` is ramp-rate-invariant to the same order as
> rung 57's constant `v` (1.05 points over a 20× range) — the bleed has no clock either.**
> This CONFIRMS rung 57's stated mechanism (algebraic-in-state) and is reported as a
> corroborating control. It is **not** billed as a finding and it is **not** the headline.

---

## 3. SCORING

### P1 — **HIT**, and the probe that scored it found a placement artifact first.

P1a, `dn_L/db` on the steady running line — **no reversal**:

| `Tt4` | `dn_L/db` at `v`=0 | at `v`=0.20 |
|---|---|---|
| 1500 | −0.23996 | −0.13163 |
| 1300 | −0.23463 | −0.14520 |
| 1100 | −0.22670 | −0.15355 |
| 1000 | −0.22109 | −0.15505 |
| 900 | −0.21399 | −0.15435 |

and the stator's own factor `dn_L/dv` = +0.320 … +0.653, positive at every throttle. Both
loop-gain factors have the predicted sign with no reversal anywhere in the band.

**THE PLACEMENT ARTIFACT (found, published, fixed).** The first measurement used rung 57's
own `n_lo` = 0.75574 — the *bare* machine's idle speed — for both levers. That is fine for
the stator, whose head start moves `nu0_L` **UP** to 0.8166, *into* the active band. It is
wrong for the bleed, whose head start moves `nu0_L` **DOWN** to 0.7336, *below* `n_lo`, where
`S` clips to 1 and `b ≡ b_max`: `db/dn` = 0, and the loop is measured **saturated**. The
tell was arithmetic — `FULL − RAMP` (+0.00445) was almost exactly the START term (+0.00418),
i.e. an additive head start with no loop at all, while the stator's `FULL − RAMP` was 4.8× its
own START term. Re-placed so both armed idles sit strictly inside the band:

| `n_lo` | stator `FULL/RAMP` (r=0.25 / 1.0) | bleed `FULL/RAMP` | bleed (F−R)/START |
|---|---|---|---|
| 0.75574 (saturated) | 0.8029 / 0.7563 | 1.0480 / 1.0378 | 2.93 / 1.18 |
| 0.70 | 0.8119 / 0.7578 | 1.0916 / 1.0828 | 4.92 / 2.37 |
| 0.65 | 0.8313 / 0.7709 | **1.0989 / 1.0930** | 5.21 / 2.59 |
| 0.60 | 0.8531 / 0.7857 | 1.0946 / 1.0925 | 5.08 / 2.58 |

P1b, the loop **witnessed directly** — the setting each schedule commands at its own surge
minimum, RAMP leg → FULL leg, `n_lo` = 0.65:

| lever | `r` | cmd on RAMP | cmd on FULL | direction |
|---|---|---|---|---|
| stator | 0.25 | 0.11997 | 0.09797 | commands **LESS** — backs off |
| stator | 1.00 | 0.12733 | 0.10833 | commands **LESS** |
| bleed | 0.25 | 0.06582 | 0.07092 | commands **MORE** — leans in |
| bleed | 1.00 | 0.07310 | 0.07668 | commands **MORE** |

P1c — `shape="linear"` (S′ ≠ 0 at both corners, no C¹ flat spot): stator 0.8116–0.8870,
bleed 1.0735–1.0787. Grid: `ds` = 0.02 / 0.01 / 0.005 moves `FULL/RAMP` by **< 0.1 %** for
both levers (stator 0.7900/0.7894/0.7893, bleed 1.0971/1.0971/1.0969).

### P2 — **HIT on sign and mechanism, band slightly too narrow.**

Predicted 5–20 % of the credit sum. Measured **9.1 % – 29.1 %**, negative (sub-additive) in
every row, over six `(v_max, b_max, n_lo)` combinations at `r` = 0.50:

| `v_max` | `b_max` | `n_lo` | `c_stator` | `c_bleed` | `c_pair` | sum | `i`/sum |
|---|---|---|---|---|---|---|---|
| 0.20 | 0.10 | 0.65 | +0.03780 | +0.09177 | +0.10467 | +0.12957 | **−0.192** |
| 0.10 | 0.05 | 0.65 | +0.02212 | +0.04435 | +0.06043 | +0.06647 | −0.091 |
| 0.30 | 0.15 | 0.65 | +0.04921 | +0.14172 | +0.13529 | +0.19093 | −0.291 |
| 0.20 | 0.10 | 0.60 | +0.03263 | +0.07650 | +0.09010 | +0.10913 | −0.174 |
| 0.20 | 0.05 | 0.65 | +0.03780 | +0.04435 | +0.07082 | +0.08215 | −0.138 |
| 0.10 | 0.10 | 0.65 | +0.02212 | +0.09177 | +0.10054 | +0.11389 | −0.117 |

and on the `tilted` maps −0.184 … −0.203. Against rung 61's **steady ≤ 2.3 %** this is an
**8×** loss of superposition, and the predicted reason — the shared speed state the steady
matcher re-solves away — is confirmed by P3's controls below.

### P3 — **REFUTED, with the OPPOSITE SIGN, and the refutation is the second finding.**

P3 predicted the bleed's positive loop would **restore** part of what the stator's negative
loop surrenders. Measured on the composite, the pair's ratio does land between the singles
(0.8626 / 0.8646 / 0.8731 vs stator 0.7709–0.8313 and bleed 1.0930–1.0989) — but that
composite is a credit-weighted blend of two different quantities and does **not** test the
claim. The clean test carries the neighbour on BOTH sides of the difference, so the
difference is the stator schedule alone:

**The stator schedule's OWN marginal surrender (1 − FULL/RAMP), by neighbour:**

| neighbour | `r`=0.25 | `r`=0.50 | `r`=1.00 |
|---|---|---|---|
| (none) | +0.1687 | +0.2106 | +0.2291 |
| constant `b` = 0.05 | +0.2108 | +0.2412 | +0.2521 |
| constant `b` = 0.10 | +0.2471 | +0.2685 | +0.2720 |
| **SCHEDULE `b_max` = 0.10** | **+0.7241** | **+0.6839** | **+0.6333** |

The bleed schedule **triples** the stator schedule's self-cancellation — the opposite of
P3a. And the mirror is nearly inert: the bleed schedule's own ratio moves 1.0989 → 1.0971
(constant `v`) → 1.0945, and 1.0930 → 1.0852, i.e. **within 0.7 %**. A **one-way arrow**,
running from the amplifying lever to the cancelling one.

**Three controls, because this is the rung's most quotable number sitting on its least-checked
measurement** (the advisor's block, and it was right — `FULL` collapses to +0.01080, 30 % of
the no-neighbour value, so the ratio is two small marginal differences):

* **GRID.** `ds` = 0.02 / 0.01 / 0.005 → 0.2730 / 0.2759 / 0.2767 at `r` = 0.25 and
  0.3663 / 0.3667 / 0.3666 at `r` = 1.00. The effect is 3× the singles' own value and the
  grid moves it by ≤ 1.3 %.
* **LEVEL-MATCHED.** The constant-`b` legs above were **not** level-matched: the schedule
  commands only 0.0709 / 0.0740 / 0.0767 at its own surge minimum (trajectory mean 0.034–0.038)
  while constant `b` = 0.10 holds 0.10 everywhere. Matched at the schedule's own commanded
  value the surrender is **+0.2265 / +0.2546 / +0.2629** — and even the *over*-matched constant
  0.10 reaches only +0.2471 / +0.2685 / +0.2720. **The schedule does 2.3–2.9× the damage with
  23–29 % less lever.** The level explanation is dead; it is the LOOP.
* **RELOCATION** (rung 50's question, and an honest caveat rather than a clean pass). The
  minima do move: `s_min` on the reference march goes 0.190 → 0.220 (`r`=0.25) and 0.270 →
  0.350 (`r`=1.00) when the bleed schedule is armed, and the RAMP/FULL separation widens from
  0.01 to 0.07. Relocation is present in **every** leg including the constant-`b` controls,
  whose surrender stays at 0.25 — so it does not track the effect — but it is a consequence
  of the loop rather than an independent check, and it is disclosed as such.

**The mechanism, MEASURED (F4), not asserted.** Armed idle `nu0_L` at `n_lo` = 0.65:

```
    bare 0.755741   stator only 0.799323   bleed only 0.736987   PAIR 0.790243
    stator +0.043582   bleed -0.018753   additive +0.024829   PAIR ACTUAL +0.034503
    the stator's marginal head start is +0.053256 on top of bleed-only,
                                    vs  +0.043582 on top of bare   -- 22 % LARGER
```

As the stator raises `n`, the bleed schedule **closes**, which raises `n` further; the stator
schedule reads that enlarged excursion and closes back harder. **The small-signal "two loops
through one state MULTIPLY rather than add" algebra is NOT claimed** — E3's gap (pair 0.8626
vs additive 1.0058) is consistent with it and with other readings, and this project's own
precedent (rung 61: *"a 'derived' scaling whose binding constant is mine is not derived"*)
applies. What is claimed is the measured one-way arrow and the measured 22 % head-start
enlargement.

### P4 — **HIT.** The adverse speed-cost interaction survives the transplant.

Raw four-cell interaction in peak `nu_L`, **positive in all 12 rows** measured:
`shaped` +0.00223 / +0.00190 / +0.00144, `tilted` +0.00205 / +0.00174 / +0.00130
(`r` = 0.25 / 0.50 / 1.00). Rung 61's steady sign holds dynamically.

**The RATIO is deliberately not published.** `n_bleed` is *negative* (the valve lowers peak
speed) while `n_stator` is positive, so `i_n`/(sum) has a difference of opposite-signed terms
in its denominator — rung 43's currency-circularity trap exactly. The raw excursion carries
the claim.

### P5 — **MISSED**, and the miss restates as a **monotonicity**, not a swing ratio.

Credit per unit setting, over a 20× ramp-rate range:

| `r` | bleed credit/`b` | stator credit/`v` |
|---|---|---|
| 0.10 | 1.42551 | 0.35616 |
| 0.25 | 1.29335 | 0.34587 |
| 0.50 | 1.24407 | 0.34570 |
| 1.00 | 1.21573 | 0.34644 |
| 2.00 | 1.20043 | 0.34720 |

The first framing ("the bleed swings 6× more") rides on the `r` = 0.10 endpoint, which is an
outlier in **both** series. The honest statement is categorical: **the stator's is
NON-monotone at the 0.4 % level** (0.34570 → 0.34720 — and rung 57 says in as many words that
its drift is not monotone), **the bleed's is STRICTLY monotone decreasing at every one of the
five rates** (−15.8 % overall, −7.2 % excluding the endpoint). Monotone decay against a
non-monotone wobble at the noise floor.

So the ramp-rate invariance rung 57 found is a **wall-mover** property: the floor channel
contributes exactly `v` whatever the trajectory does, while a point-mover's entire credit is
through `φ` and inherits the trajectory's own ramp-rate dependence. This **supplies rung 57's
complementary case** — the project's first continuous, edgeless, non-fuel point-mover — and it
does so without re-claiming rung 57's published mechanism (§ 0).

---

## 4. Summary of scoring

| | prediction | verdict |
|---|---|---|
| P1 | bleed schedule self-AMPLIFIES (`FULL/RAMP` > 1) | **HIT** (1.09–1.10 vs stator 0.77–0.83) |
| P1a | `dn_L/db` < 0 with no reversal | **HIT** |
| P1b | commanded setting moves oppositely | **HIT** |
| P1c | survives shape + grid | **HIT** |
| P2 | interaction 5–20 % sub-additive | **HIT** on sign/mechanism, band too narrow (9.1–29.1 %) |
| P3 | bleed's loop RESTORES the stator's surrender | **REFUTED, opposite sign** — it TRIPLES it |
| P3a | stator's surrendered share shrinks | **REFUTED** (0.169–0.229 → 0.633–0.724) |
| P4 | adverse speed-cost interaction survives | **HIT** (12/12 rows) |
| P5 | bleed's credit/setting ramp-invariant like the stator's | **MISSED** — strictly monotone, not invariant |

**Three of nine scored wrong, and the two that produced the rung's content are P3 and P5.**
This is the same shape as rungs 61 (P5a refuted → the headline), 49 (both predicted signs
wrong) and 42 (the hypothesis refuted by its own probe).
