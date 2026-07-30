# Rung 63 anchor — FUEL + BLEED on one plant

The seam rung 62 named:

> *"**FUEL + BLEED** — the half of rung 61's seam this rung does not take. Every fuel-side leg
> (46–52) lives on `integrate_fuel`, and the bleed now sits in the same closure, so the
> composite is one constructor away; § 2 makes it sharp, because a fuel leg is a min-select
> with **edges** while the bleed schedule is continuous and self-amplifying, and § 2 says a
> positive loop does not leave its neighbour's loop alone."*
> — `docs/rung62-spec.md` § The next seam

This file is the **pre-registration**. Everything in § 1 and § 2 was written **before any
probe was run**; § 3 scores it afterwards, and where a prediction missed the miss is
published as a miss.

---

## 0. The blocker that would have counterfeited the whole rung

Rung 62 overrode `at_stator` on purpose, so that a rung-57 reader called on a bleed-armed
machine differences against a sibling **carrying this machine's valve** — otherwise the
valve's whole effect would be booked to the stator (rung 62's gate 3, rung 61's `at_setting`
trap one ladder over).

That override reaches **six** inherited readers, not one:

    stator_credit  credit_decomposition  composite_credit
    engagement_shift  schedule_invariance  matched_credit

`schedule_invariance` is the one that bites. Called on a bleed-armed machine it derives the
`Wf/pt3` table on `self` and on `self.at_stator()` — **which is the same bleed-armed
machine** — and returns `ordinate_identical = True`, `d_ordinate ≈ 0`. That is *numerically
identical to rung 59's headline result*, so it would read as a clean confirmation of rung 59
while measuring nothing at all.

**Every bleed-isolating reader in this rung is therefore built on the
`at_lever(**reference)` / `at_lever(**{**reference, **lever})` pattern that rung 62's
`marginal_loop` already established, and rungs 58/59's own methods are left literally
unchanged.** The mirror of rung 62's gate 3 is a gate here: the bleed-isolating reference
sibling must be asserted valve-SHUT.

## 1. THE DERIVATION — written before any probe, and it is the mechanism

Rung 58 measured a **one-way arrow**: a fuel leg moves the stator's credit by +9.51 %, the
stator moves the fuel leg's engagement time by −0.162 % (a factor of 59, sub-grid). Rung 59
then explained the small number exactly — the stator reaches **neither** of the `Wf/pt3`
leg's two sensed inputs:

```
    ORDINATE   kappa_ss = Wf/pt3 = pi_b * f(Tt3,Tt4) * MFP_A4 / [(1+f) * sqrt(Tt4)]
               A4 is CHOKED, so MFP_A4 is hardware; Tt3 is pinned by two MAP-FREE shaft
               balances.  =>  kappa_ss = kappa_ss(Tt4) alone.        [rung 59 _proof_chain]
    ABSCISSA   n_H(Tt4): the HP-face corrected flow carries pt4 ∝ pi_LPC over pt25 ∝ pi_LPC.
               pi_LPC CANCELS.                                        [rung 39's ONE arrow]
```

**A bleed breaks both, and the algebra says exactly where.** Of the two shaft balances only
the LP one carries the valve (`_powers`: `Pt_lp = eta_m*(1-b)*(1+f)*dh_LPT`; the HP balance
has core flow on both sides so `(1-b)` cancels — rung 42's bleed-INVARIANT form):

```
    dh_LPC = eta_m * (1-b) * (1+f) * dh_LPT(Tt4, f)
        =>  Tt25 FALLS with b        (the LP turbine drives (1-b) of the core per unit face air)
    dh_HPC = eta_m * (1+f) * dh_HPT(Tt4, f)          -- bleed-free
        =>  Tt3 = Tt25 + const(Tt4)  falls by the SAME enthalpy
        =>  f = f(Tt3, Tt4) RISES     (a colder burner inlet needs more fuel for the same Tt4)
        =>  kappa_ss RISES
    and  m_hp ∝ sqrt(Tt25) * pi_HPC / (1+f)   moves too   =>  n_H(Tt4) MOVES
```

`pi_LPC` still cancels out of `m_hp` — rung 39's arrow is not repealed. What moves the
abscissa is that the bleed moves `Tt25` **itself**, which no stator can do.

Magnitude, by hand, at `b_max` = 0.10: `Tt25 - Tt2` falls ~10 % ≈ 11 K, `Tt3` with it, and
`f ≈ 0.02` rises by `cp*11/(hPR*eta_b) ≈ 2.6e-4` — about **+1.3 %** on `kappa_ss`. Small,
and a thousand times rung 59's machine-precision zero.

**So the a-priori expectation for the seam's own question is a TWO-WAY arrow — the family's
first.** That, and not the rung-59 boundary, is what the seam asks. (Leading with "rung 59's
ordinate-invariance is bounded" would be the duplicate-identity billing rung 62 § 0 records
this project having made three times: it restates rung 59's own derivation with one term
carried. It is the MECHANISM here, not the headline.)

## 2. THE PREDICTIONS — written before measuring

Scored in § 3 as **eleven items**.

### P1 — the two sensed inputs. *The mechanism, stated so it can fail.*

> **P1a** `d_ordinate` ≠ 0 for a bleed schedule, **POSITIVE** (κ rises with b), magnitude
> **1.0–2.0 %** at `b_max` = 0.10 against rung 59's exact zero for a stator.
> **P1b** `d_abscissa` ≠ 0 as well — the bleed moves `Tt25`, which is upstream of rung 39's
> cancellation rather than protected by it.
> **P1c** Therefore `abscissa_share` and `ordinate_share` are **both** non-degenerate for the
> first time in the ladder. Rung 59 had exactly one of them 1 and the other 0 (LP stator:
> abscissa 0; HP stator: ordinate 0). Neither share is within 0.05 of 0 or 1.

### P2 — the ARROW. *The seam's own question.*

> **P2a** The bleed schedule moves the accel leg's sub-grid engagement time `s_eng` by
> **> 1 %** — at least an order above rung 58's stator figure of −0.162 %, because the bleed
> reaches both sensed inputs and the stator reached neither.
> **P2b** The sign is **EARLIER** engagement (`s_eng` falls): the leg cuts when
> `Wf_sched > (1+m)·κ_ss(n_H)·pt3`, and the bleed lowers `pt3` (lower `Tt25` ⇒ lower `pi_LPC`
> work and lower core flow), so the cap falls and the crossing arrives sooner.
> **P2c** The reverse direction is **also** live: the fuel leg moves the bleed's credit by
> more than rung 58's constant-setting floor of +0.80 %. Hence **two-way**, where rung 58's
> ratio of the two directions was 59:1.

### P3 — the LOOP, beside a leg with EDGES. *Rung 62 § 2's own claim, transplanted.*

Rung 62: a stator schedule beside a bleed schedule has its surrender **tripled**, while the
bleed's amplification is left alone to 0.7 % — a one-way arrow from the amplifying lever to
the cancelling one. A fuel leg has **no loop of its own** (no state-feed: it reads the state
but its output is a fuel cap, not a setting that re-enters the state through `dn/d(setting)`).

> **P3** The bleed's own `FULL/RAMP` amplification, measured with the accel leg carried on
> **both** sides of the difference (`marginal_loop`), stays within **2 %** of its leg-free
> 1.093–1.099 — i.e. a legged neighbour perturbs the loop far less than a *scheduled* one
> did, because it has no `dn/d(setting)` to close through. **This is the falsifiable half:**
> if the amplification moves > 5 %, the loop answers to the trajectory and not to the
> neighbour's own loop, and rung 62 § 2's mechanism attribution is bounded.

### P4 — the φ-FLOOR cell. *Two named outcomes, one of them nominated.*

A `SurgeLimiter` pins `phi_lp`, and the bleed's credit runs **entirely** through `phi` (it is
a pure point-mover: `v ≡ 0`, so `M_i = T_c - 1/phi` exactly). Rung 60 found a floor beside a
stator gives `= v` in `phi` and `= 0` in `M_i`, both exact. With `v ≡ 0` **both** currencies
would collapse to the same exact zero. But the bleed *raises* `phi_lp` — that is its credit —
so it may instead push the floor **DORMANT**.

> **P4** DORMANT is nominated: `fuel_removed` in the `both` cell falls by **> 50 %** against
> the `fuel` cell at a floor that binds on the bare machine, and at a large enough `b_max`
> goes to **exactly 0.0**. That is not rung 60's tautology — it is *a relief lever disarming a
> floor limiter*, which is a different statement and a better one. Scored on `fuel_removed`
> per cell, **never** on the interaction alone (rung 58's dormant-`r`=2.0 precedent: a
> dormant leg's zero is the envelope edge, not evidence).

### P5 — SCOPE, pre-checked rather than discovered.

Rung 62's choked pre-check (choked at every `b` ≤ 0.30 down to `Tt4` = 900) was run with **no
leg cutting fuel**. A rung-49 floor cuts hard, and the bled `_close_fuel` makes a metered fuel
flow **richer** (`f = mdot_fuel / CORE air`).

> **P5** The choked branch survives the ramp with each leg armed at its intended setting.
> Checked BEFORE the ramp is fixed, not after a failure.
>
> Also expected, and **not** a defect: rung 59's `_clamp_audit` asserts `clamped == 0`, and
> the bleed moves `n_H`. If it fires, the derivation band widens; it is a bracket statement
> about where the table was built, not a finding.

---

## 3. SCORING — eleven items: **seven HIT, one REFUTED, three HIT-with-its-reason-corrected**

Plant: rung 57/62's own — `FLIGHT`(250 K, 50 kPa, M0 0.85), `π_LPC/π_HPC` 3/6, `Tt4_d` 1500,
`shaped` maps, `φ_surge` 0.55, ramp `Tt4` 1000→1400, `b_max` 0.10, `n_lo` 0.65,
accel `margin` 0.25, `ds` 0.005.

| # | prediction | verdict | measured |
|---|---|---|---|
| P1a | ordinate moves, POSITIVE, 1.0–2.0 % | **HIT** | `d_ordinate` = +1.05 % at mid, 1.13 % max (const `b`=0.10); stator control **1.95e−13** |
| P1b | abscissa moves too | **HIT** | `d_abscissa` = +1.49 % at mid, 1.77 % max |
| P1c | both splice halves non-degenerate | **HIT, and the ratio withdrawn** | raw `delta_index` +4.63e−3, `delta_value` −2.10e−3 — **opposite signs**; the *shares* (+1.955/−0.889) move ~10 % under an `ds` halving, so they are not published (see below) |
| P2a | `s_eng` moves > 1 % | **HIT** — but its *stated reason* was wrong, see (iv) | **+2.88 to +4.22 %** over six cells (dormant; limited agrees to <1 %) |
| P2b | the sign is EARLIER | **REFUTED** | **LATER.** The `pt3` half of my reasoning was right (−2.82 %) but the abscissa channel I had just predicted in P1b fights it (`κ(n_H)` +2.07 %, so the cap moves only −0.80 %) and the COMMANDED ramp (−1.37 %) decides |
| P2c | leg moves the lever's credit > 0.80 % | **HIT** | **+8.35 %** at `r`=0.5 — but see the correction below: this direction is rung 58 CONFIRMED, not new |
| P2d | hence two-way | **HIT, and re-framed twice** | forward +8.35 %, return +3.51 %; neither the 2.4:1 ratio nor an "asymmetry of conditions" survives as the claim — see (i) and (iv). What stands is the TABLE result (§ 1) with the re-timing as its bounded consequence |
| P3 | the bleed's loop within 2 % beside a leg | **HIT** | +0.80 / +0.50 / +0.06 % at `r` = 0.25/0.50/1.00 |
| P4 | the floor is DISARMED (> 50 %, → 0 at large `b_max`) | **HIT, in its strongest form** | **exactly 0.0**, and the armed cell **bit-for-bit** its own leg-free march, over the whole band `sm` ∈ [0.3372, 0.4344] |
| P4b | *(not predicted)* the regime above the band | **the tautology, exact** | `M_i(both) − M_i(fuel)` = **−1.33e−15** with the floor still binding on both |
| P5 | scope survives with a leg armed | **HIT** | `choked` at every point of every march; `clamped` = 0 on every cell |

### The three corrections this scoring forced

**(i) P2c is a CONFIRMATION, not half of a discovery.** The forward direction (leg → lever)
is rung 58's relocation × state-feed mechanism, and rung 58's own predictor — re-reading the
leg-free credit profile at the relocated minimum — recovers **85.0 %** of it, against rung
58's own 86 % for a stator schedule. So the headline cannot be the 2.4:1 ratio: it averages a
confirmation with a discovery. What is new is the **return** arrow alone.

**(ii) The two directions answer to DIFFERENT conditions, and the `r` sweep earned it.**

| `r` | return: `s_eng` shift | forward: credit shift | leg `removed` | leg `s_eng` |
|---|---|---|---|---|
| 0.25 | +3.32 % | +11.56 % | — | — |
| 0.50 | +3.51 % | +8.35 % | 4.11e−3 | 0.1229 |
| 1.00 | +4.22 % | **+0.66 %** | 1.61e−3 | **0.2947** |

The forward arrow collapses at `r` = 1.00 — and **not** because the leg went dormant
(`removed` = 1.61e−3, which reproduces rung 58's published value exactly). It engages at
`s` = 0.2947, **downstream** of the incidence minimum, so it relocates nothing: rung 48's
engagement law, and rung 58's own published explanation of the same number. The return arrow
has no such condition — +2.9 to +4.2 % at every rate, on both map shapes.

**(iv) P2a's PREMISE — "at least an order above rung 58's stator figure" — is REFUTED, and
the refutation was self-inflicted twice.** The prediction compared against rung 58's published
−0.162 %. Two things are wrong with that, both found after the fact:

1. **It is a different schedule.** Rung 58 placed its stator at `n_lo` = 0.7557; every number
   here is at `n_lo` = **0.65**, which rung 62 had to adopt because rung 57's placement leaves
   a *bleed* schedule saturated at `b_max`. The two are not comparable and rung 58's figure is
   now quoted nowhere as a control. (An intermediate draft mis-diagnosed the gap as a
   limited-vs-dormant difference; probe H killed that too — the two readings agree to under
   1 % of the shift in all twelve cells, for both levers.)
2. **Measured on this rung's own placement, a stator DOES re-time the leg** — up to +1.28 %
   (tilted, `r` = 0.25), where the ratio falls to **2.3** and not "an order".

The repair is a distinction the prediction had conflated: `κ_ss` and `n_H(Tt4)` are properties
of the steady TABLE (§ 1's claim, exact and grid-independent); `s_eng` is a property of the
TRAJECTORY through it, which any lever can move. **The bleed's channel is structural, the
stator's trajectory-mediated**, and the gateable statement is that the bleed's shift is
positive and strictly the larger in every cell — not that a stator cannot re-time a leg.

**(iii) The splice's shares are withdrawn, and this is the advisor's catch.** Rung 59's
`matched_credit` documents "they must sum to 1"; here they sum to 1.066. With both halves
beyond |1| and opposite in sign, `delta_match` is a small difference of two larger terms —
rung 43's currency-circularity shape. Halving `ds` to 0.0025 settles it:

| `ds` | `delta_index` | `delta_value` | `delta_match` | shares | sum |
|---|---|---|---|---|---|
| 0.005 | +4.626e−3 | −2.103e−3 | +2.367e−3 | +1.955 / −0.889 | 1.0662 |
| 0.0025 | +4.648e−3 | −1.848e−3 | +2.633e−3 | +1.765 / −0.702 | 1.0634 |

`delta_index` moves **0.5 %**; `delta_value` and `delta_match` move 11–12 %, and the shares
swing 10 % while their sum barely moves. So the shares are numerical and are **not
published**; P1c's claim is carried by the two raw deltas being large and **opposite in
sign**, which needs no ratio and is grid-stable in both sign and order of magnitude.

### What was measured and is NOT claimed

* The `s_eng` shift is reported on the **dormant** march as well as the limited one (they
  agree to 6 decimals). No claim rides on the clipped-state feedback.
* `sm` = 0.43 and 0.46 return `s_eng` = nan — the floor is violated from `s` = 0, so
  `_s_eng` finds no upward crossing. Every floor verdict is read off `fuel_removed`
  instead, and no gate touches `s_eng` in that section.
* The band edges are the two plants' **own** minimum `φ`, so nothing in P4 is fitted; but
  that also makes the *existence* of a band the content, not its endpoints.
