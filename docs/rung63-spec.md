# Rung 63 — FUEL + BLEED: what a min-select leg can FEEL

Rung 62 closed by naming this rung:

> *"**FUEL + BLEED** — the half of rung 61's seam this rung does not take. Every fuel-side
> leg (46–52) lives on `integrate_fuel`, and the bleed now sits in the same closure, so the
> composite is one constructor away; § 2 makes it sharp, because a fuel leg is a min-select
> with **edges** while the bleed schedule is continuous and self-amplifying."*

> **THE HEADLINE — a fuel schedule's TABLE has exactly two protections, and only a lever that
> extracts MASS can reach them. `κ_ss` is guarded by a CHOKED `A4`; its abscissa `n_H(Tt4)` is
> guarded by rung 39's `π_LPC` cancellation. Rung 59 showed a stator satisfies both and read
> that as a fact about schedules. It is not: it is a fact about levers that preserve
> `ṁ_face = ṁ_core`. The bleed is the ladder's only lever that breaks that identity, and the
> identity sits in the LP shaft balance — UPSTREAM of both guards. So a bleed moves BOTH
> halves of the table by ~1e−2 where a stator moves neither (~1e−13), with `MFP_A4` pinned at
> 1e−16 throughout. This is exact, grid-independent, and was derived before it was measured.**
>
> **THE CONSEQUENCE, and it is bounded — the leg's engagement time moves +2.9 to +4.2 %,
> LATER, in all six `(ramp rate × map shape)` cells. But `s_eng` is a TRAJECTORY quantity, not
> a table quantity, so a stator moves it too (up to +1.28 %) with the table bit-identical. The
> bleed's channel is STRUCTURAL and the stator's is TRAJECTORY-MEDIATED; what separates them
> in the data is that the bleed's shift is systematically positive and larger in every cell,
> not that a stator cannot re-time a leg. It can.**

Four sections. § 0 is the instrument, and it is the section that would have counterfeited the
rung.

---

## 0. THE INSTRUMENT — the override that turns six readers into armed-vs-armed

Rung 62 overrode `at_stator` **on purpose**: a rung-57 reader called on a bleed-armed machine
must difference against a sibling *carrying this machine's valve*, or the valve's whole effect
is booked to the stator (rung 62's gate 3, rung 61's `at_setting` trap one ladder over).

That override reaches **six** inherited readers, not one:

    stator_credit   credit_decomposition   composite_credit
    engagement_shift   schedule_invariance   matched_credit

**`schedule_invariance` is the one that bites.** On a bleed-armed machine it derives the
`Wf/pt3` table on `self` and on `self.at_stator()` — *the same bleed-armed machine* — and
returns `ordinate_identical = True`, `d_ordinate ≈ 0`. That is **numerically identical to
rung 59's headline result**, so it would have read as a clean confirmation of rung 59 while
measuring nothing at all, and § 1 below is exactly the claim it would have counterfeited.

Every rung-63 reader is therefore built on `_isolating`, which constructs the pair from
`at_lever` and **asserts the reference sibling is valve-shut**. Rungs 58/59's own methods are
left literally unchanged. `_legs` (rung 62) gains `accel` / `surge` / `Tt4_max`, all
defaulting to `None` — `_stator_march`'s own default — so every rung-62 caller reaches the
identical four marches: **THE REDUCE**.

### Scope, pre-checked rather than discovered

Rung 62's choked pre-check was run with no leg cutting fuel. A rung-49 floor cuts hard, and
the bled `_close_fuel` makes a metered fuel flow **richer** (`f = ṁ_fuel / CORE air`). Re-run
with each leg armed: `choked` at **every point of every march**. `_clamp_audit` reports
`clamped = 0` on every cell but one — the tilted map at `r` = 0.25, where 1 of 207 cutting
points sits outside the derived bracket. That single exception is published with its
robustness check in § 2 (‡) rather than engineered away.

## 1. THE MECHANISM — the leg's two sensed inputs, and what can reach them

Rung 58 measured a one-way arrow: the leg moved the stator's credit by **+9.51 %**, the stator
moved the leg's engagement time by **−0.162 %** *(both at rung 58's own placement, `n_lo` =
0.7557 — see § 2 for why that figure is not a control for this rung's grid)*. Rung 59
explained the small number exactly — the leg senses two things and a stator reaches
**neither**:

```
    ORDINATE   kappa_ss = Wf/pt3 = pi_b * f(Tt3,Tt4) * MFP_A4 / [(1+f) * sqrt(Tt4)]
               A4 CHOKED => MFP_A4 is hardware; Tt3 pinned by two MAP-FREE shaft balances
                                                          [rung 59's _proof_chain]
    ABSCISSA   n_H(Tt4): the HP-face corrected flow carries pt4 ~ pi_LPC over pt25 ~ pi_LPC.
               pi_LPC CANCELS.                            [rung 39's ONE arrow]
```

**A bleed breaks both, and the algebra says where.** Of the two shaft balances only the LP one
carries the valve — the HP has core flow on both sides, so `(1−b)` cancels (rung 42's
bleed-INVARIANT form):

```
    dh_LPC = eta_m*(1-b)*(1+f)*dh_LPT(Tt4,f)    =>  Tt25 FALLS with b
    dh_HPC = eta_m*(1+f)*dh_HPT(Tt4,f)          =>  Tt3 falls by the SAME enthalpy
                                                =>  f RISES     =>  kappa_ss RISES
    m_hp ~ sqrt(Tt25) * pi_HPC / (1+f)          =>  n_H(Tt4) MOVES
```

`π_LPC` still cancels out of `m_hp`: **rung 39's arrow is not repealed.** What moves the
abscissa is that the bleed moves `Tt25` *itself*, which no stator can do.

Measured — the hand estimate was `Tt25 − Tt2` down ~10 %, `f` up ~1.3 %:

| | `Tt25 − Tt2` | `Tt3` | `f` | `MFP_A4` | `κ_ss` | `n_H` |
|---|---|---|---|---|---|---|
| `b` = 0.10, `Tt4` = 1200 | **−9.980 %** | −9.24 K | +1.077 % | **+2.2e−16** | +1.054 % | +1.493 % |

`MFP_A4` at machine zero is the control that must hold for *any* lever — `A4` is choked, so
the corrected group is hardware. It does, to 1.1e−16.

**The two tables, half by half**, against rung 59's own verdicts:

| lever | `d_ordinate` | `d_abscissa` |
|---|---|---|
| LP stator, scheduled *(rung 59's zero, reproduced)* | 1.95e−13 | 7.76e−13 |
| bleed, constant `b` = 0.05 | 5.64e−3 | 8.73e−3 |
| bleed, constant `b` = 0.10 | **1.13e−2** | **1.77e−2** |
| bleed, scheduled `b_max` = 0.10 | 9.54e−3 | 1.02e−2 |

Ten orders of magnitude apart. **Rung 59's "a schedule's ORDINATE cannot see a stator, only
its INDEX can" is therefore BOUNDED, not refuted**: it is a property of levers that preserve
the core-mass identity, and every lever the ladder had carried until rung 42's valve reached
the transient did.

### The splice, and the ratio that is deliberately withdrawn

Rung 59 split the matched leg's effect into an abscissa half and an ordinate half and always
had one of them **exactly** zero, which made the split trivially additive. Here both are live:

| `ds` | `delta_index` | `delta_value` | `delta_match` |
|---|---|---|---|
| 0.005 | +4.626e−3 | −2.103e−3 | +2.367e−3 |
| 0.0025 | +4.648e−3 | −1.848e−3 | +2.633e−3 |

**The two halves carry OPPOSITE SIGNS and they fight**: re-indexing alone would move the
armed cell by +4.63e−3, re-valuing alone by −2.10e−3, and the matched leg nets +2.37e−3.

> **The SHARES are not published.** They would be +1.955 / −0.889 summing to 1.066 against
> rung 59's documented "must sum to 1" — but `delta_match` is then a small difference of two
> larger opposite-signed terms, which is rung 43's currency-circularity shape. Under an `ds`
> halving `delta_index` moves 0.5 % while `delta_value` and `delta_match` move 11–12 % and the
> shares swing 10 %. So the shares are numerical. The claim is carried by the two **raw**
> deltas being large and opposite in sign — grid-stable in both sign and order of magnitude,
> and needing no ratio. Rungs 45/49's precedent, applied to our own new instrument.

## 2. THE CONSEQUENCE — the leg is re-timed, and by how much a stator can do it too

`leg_retiming` is rung 58's `engagement_shift` on a lever the leg can feel. One leg object on
both plants (rung 58's discipline), sub-grid, and read on the **dormant** march — where `g` is
defined everywhere and no clip has yet perturbed the states. The **dormant** reading is this
rung's published convention throughout; measured against the limited one it makes no
difference anywhere — the two agree to **under 1 % of the shift** in all twelve cells, for
both levers.

> **RUNG 58's OWN −0.162 % IS NOT THE CONTROL FOR THIS TABLE, and an earlier draft used it as
> one.** Rung 58 placed its schedule at `n_lo` = 0.7557 (rung 57's knee); every number here is
> at `n_lo` = **0.65**, the placement rung 62 had to adopt because rung 57's leaves a *bleed*
> schedule saturated at `b_max` (rung 62 § 1's published artifact). Two different schedules,
> so the two numbers are not comparable and rung 58's is never used as a control — where § 1
> cites it, it is labelled as rung 58's own measurement at rung 58's own placement. The stator
> control in the table below is measured **on this rung's placement**. The `s_eng` = 0.1228893
> reference value *does* reproduce rung 58's exactly, because the bare machine is the same.

**All six cells, dormant, `b_max` = 0.10 against `v_max` = 0.20.** The last column re-derives
the leg over a **wider** `Tt4` band (950–1500 instead of the ramp's own 1000–1400) and is the
robustness check for the clamp caveat below — not a second result:

| shape | `r` | stator | **bleed** | ratio | bleed, wide band |
|---|---|---|---|---|---|
| shaped | 0.25 | +0.481 % | **+3.324 %** | 6.9 | +3.318 % |
| shaped | 0.50 | −0.026 % | **+3.509 %** | 134 | +3.502 % |
| shaped | 1.00 | +0.012 % | **+4.222 %** | 345 | +4.250 % |
| tilted | 0.25 | +1.276 % | **+2.882 %** | **2.3** | **+3.161 %** ‡ |
| tilted | 0.50 | +0.449 % | **+3.319 %** | 7.4 | +3.313 % |
| tilted | 1.00 | +0.570 % | **+3.943 %** | 6.9 | +3.969 % |

‡ **THE ONE CLAMP, disclosed and kept.** On this single cell, **1 of 207** cutting points on
the *reference* march reads `AccelSchedule.cap` at its clamped endpoint — the envelope edge
rather than the derived shape, which is precisely what rung 59 built `_clamp_audit` to catch.
The derivation band is left at rungs 58/59's ramp band regardless, because widening it
re-derives every `κ` and would move every other number in this rung to rescue one point. The
cell is **not** dropped — it is the cell that refuted this rung's own over-claim, and dropping
it would be removing inconvenient data. The wide-band column shows the conclusion is
insensitive to the caveat: the bleed shift there is +2.88 % clamped and +3.16 % clean, the
ordering and the sign are unchanged either way, and the gate asserts `clamped <= 1`.

> **THE OVER-CLAIM THIS TABLE KILLED, PUBLISHED RATHER THAN QUIETLY DROPPED.** On rung 58's own
> cell (shaped, `r` = 0.5) the ratio is 134:1, and the first draft of this rung read that as
> *"a stator cannot re-time the leg; a bleed moves it twenty times as far."* **The tilted map at
> `r` = 0.25 refutes it**: the stator reaches **+1.28 %** and the ratio falls to 2.3.
>
> The repair is a distinction the first draft had conflated. `κ_ss` and `n_H(Tt4)` are
> properties of the steady TABLE; `s_eng` is a property of the TRAJECTORY through it. A stator
> leaves the table bit-identical (§ 1) and still moves the *path* of `pt3` and `n_H`, so it can
> re-time a leg without touching either guarded quantity. **The bleed's channel is STRUCTURAL,
> the stator's is TRAJECTORY-MEDIATED.** What the data separates is not presence from absence
> but **systematic from incidental**: the bleed's shift is positive and ≥ +2.88 % in all six
> cells and strictly the larger in all six; the stator's spans −0.03 % to +1.28 % and **changes
> sign** across the grid. The sign change is not solver noise — the two near-zero cells hold to
> four significant figures under an `ds` halving (−0.02625 → −0.02633 and +0.01224 → +0.01228).
> It is nonetheless left **out of the gate**, which asserts only the per-cell ordering and a
> nonzero stator shift: the ordering cannot be wrong, and after this rung's other two
> over-claims that is the trade worth making.

**The forward direction has a CONDITION; the return direction does not.**

| `r` | return (`s_eng`) | forward (credit) | leg `removed` | leg `s_eng` |
|---|---|---|---|---|
| 0.25 | +3.32 % | +11.56 % | — | — |
| 0.50 | +3.51 % | +8.35 % | 4.11e−3 | 0.1229 |
| 1.00 | +4.22 % | **+0.66 %** | 1.61e−3 | **0.2947** |

At `r` = 1.00 the forward direction collapses — and *not* because the leg went dormant:
`removed` = 1.61e−3, which reproduces rung 58's published value exactly. It engages at
`s` = 0.2947, **downstream** of the incidence minimum, so it relocates nothing. That is rung
48's engagement law and rung 58's own published explanation of that very number, reappearing
inside a different composite.

### The forward direction is rung 58 CONFIRMED, not new content

Rung 58's mechanism claim was that the interaction is computable from marches that never saw
the leg: re-read the leg-free credit profile at the relocated minimum.

| lever | interaction | predicted from the leg-free marches | recovered |
|---|---|---|---|
| rung 58's stator schedule | +0.005017 | +0.004311 | 86 % |
| **this rung's bleed schedule** | +0.007658 | +0.006513 | **85.0 %** |

So the headline is **not** the 2.4:1 ratio of the two directions — that averages a
confirmation with a discovery, and the ratio itself inverts at `r` = 1.00. Nor, as the table
above establishes, is it the return arrow alone. **It is the TABLE result of § 1**, which owes
nothing to either direction's magnitude: the re-timing is that result's bounded consequence,
and the forward direction is rung 58 reproduced on a new lever.

### The SIGN is not the obvious one — three channels, and the third decides

The pre-registered prediction was **earlier** engagement, reasoning that the bleed lowers
`pt3` so the cap falls. Half right, and **refuted**. At the reference plant's own engagement
time:

```
    pt3       -2.817 %      the pressure channel -- as predicted
    kappa(nH) +2.074 %      the ABSCISSA channel, fighting it  (this rung's own s 1)
    ---------------------
    cap       -0.802 %      the two nearly cancel
    mf_sched  -1.366 %      the COMMANDED ramp, re-derived on the bled plant
    => g = mf_sched - cap goes NEGATIVE   =>  the crossing arrives LATER
```

The third term is not an artifact: `_stator_march` pins both plants to the **same `Tt4`
endpoints** (rung 35's apples-to-apples discipline, carried since rung 43), and a bled machine
burns different fuel to reach them. The prediction failed by forgetting the channel the same
rung had just derived one section earlier.

### The bleed's own LOOP is left alone by a legged neighbour

Rung 62 § 2: a bleed *schedule* triples a stator schedule's surrender, while the stator leaves
the bleed's amplification alone to 0.7 % — a one-way arrow from the amplifying lever to the
cancelling one. A fuel leg has **no loop of its own**: it reads the state but emits a fuel cap,
not a setting that re-enters through `dn/d(setting)`.

| `r` | bleed `FULL/RAMP`, leg-free | with the accel leg on both sides | Δ |
|---|---|---|---|
| 0.25 | 1.098455 | 1.107188 | +0.795 % |
| 0.50 | 1.096873 | 1.102404 | +0.504 % |
| 1.00 | 1.092978 | 1.093589 | +0.056 % |

**Within 0.8 %, i.e. the same order as rung 62's scheduled neighbour.** So the loop answers to
its own `dn/d(setting)` and not to the trajectory a neighbour hands it — rung 62 § 2's
mechanism attribution **CONFIRMED** on a neighbour of a different kind.

## 3. THE SECOND FINDING — a `φ` floor and the valve have NO COMPOSABLE MIDDLE

A bleed's credit runs **entirely** through `φ`: it is a pure point-mover, `v ≡ 0`, so
`M_i = T_c − 1/φ` exactly, with `T_c` the blade metal off the design map. A `SurgeLimiter`
**pins** `φ`. Rung 60 found a floor beside a *stator* gives `= v` in `φ` and `= 0` in
incidence, both exact; with `v ≡ 0` those two collapse onto each other, and the pair has only
**two** regimes — with a boundary that is not fitted, because it is the two plants' own
minimum `φ`:

| `sm` | `φ_lim` | `removed` (fuel) | `removed` (both) | lever's credit |
|---|---|---|---|---|
| 0.30 | 0.71500 | 0.0 | 0.0 | *(leg dormant on both — no composite)* |
| 0.34 | 0.73700 | 1.49e−5 | **0.0 exactly** | DISARMED |
| 0.36 | 0.74800 | 7.11e−4 | **0.0 exactly** | DISARMED |
| 0.40 | 0.77000 | 1.10e−2 | **0.0 exactly** | DISARMED |
| 0.43 | 0.78650 | 1.31e−2 | **0.0 exactly** | DISARMED |
| 0.46 | 0.80300 | 8.23e−3 | 8.56e−4 | **−1.33e−15 — the tautology** |

The disarming band is `sm` ∈ **[0.337167, 0.433944]**, whose edges are exactly
`min φ(reference)` = 0.735442 and `min φ(armed)` = 0.788669 over `φ_surge` = 0.55. Inside it,
the armed cell is **bit-for-bit its own leg-free march** — the strongest available witness, and
one a tolerance would blur. Above it, both plants bind, the floor pins the currency, and the
valve's credit is **exactly zero**.

**There is no middle in which the two compose.** The band's width tracks the valve:

| `b_max` | 0.000 | 0.025 | 0.050 | 0.100 | 0.150 |
|---|---|---|---|---|---|
| width in `sm` | 0.000 | 0.0218 | 0.0451 | 0.0968 | 0.1556 |

> **This EXTENDS rung 60 rather than repeating it.** Rung 60's tautology was a statement about
> what a floor does to a lever's *credit*; the regime below it — where the same floor is
> **disarmed** by the lever, exactly and bit-for-bit — has no analogue there, because a stator
> moves the wall and the floor with it. A relief lever that moves only the *point* can push its
> partner's set point clean out of the trajectory. `fuel_removed` carries every verdict here;
> `s_eng` is deliberately not reported, because a floor above the initial `φ` is violated from
> `s` = 0 and `_s_eng` correctly returns nan.

---

## Verification gates (`tests/test_rung63.py`)

1. **THE REDUCE** — the rung-62 `_legs`/`marginal_loop` path with no leg is bit-for-bit the
   pre-rung-63 one (`loop_decomposition` and `marginal_loop` agree key-for-key with a
   leg-free `marginal_loop`), and every rung-62/57 gate still passes untouched.
2. **THE `_isolating` GATE** — the mirror of rung 62's gate 3: a lever key present in the
   neighbour raises, and the reference sibling is asserted valve-SHUT. Plus the **direct
   witness of the trap**: `at_stator()` on a bleed-armed machine returns a machine that
   still carries the valve, so `schedule_invariance` on it returns `ordinate_identical`
   — the false confirmation, gated as such so no future edit can reintroduce it silently.
3. **THE MECHANISM** — `d_ordinate` and `d_abscissa` both > 1e−3 for a bleed and both
   < 1e−12 for a stator (rung 59's own tolerance), on the same instrument; the sign of the
   ordinate shift POSITIVE; `d_mfp` < 1e−14 for every lever (the choked-`A4` control); and
   the LP-balance chain signed (`Tt25` down, `Tt3` down, `f` up).
4. **THE CONSEQUENCE** — the bleed's `s_eng` shift > +2.5 % at three ramp rates on two map
   shapes, POSITIVE (later) throughout, and **strictly larger than the stator's in every
   cell** (the stator's own shift is asserted NONZERO and < 2 %, because it is real and
   trajectory-mediated — no "a stator cannot re-time a leg" claim is gated); dormant and
   limited readings agreeing to 1e−4 in the ratio; `clamped <= 1` per cell; and the forward
   direction's collapse at `r` = 1.00 shown NOT to be dormancy (`removed` > 0) but downstream
   engagement (`s_eng` > the incidence minimum's `s`).
5. **THE FORWARD ARROW IS RUNG 58** — the leg-free profile predictor recovers 80–95 % of the
   interaction, in rung 58's own band.
6. **THE LOOP** — the bleed's `FULL/RAMP` beside a legged neighbour within 2 % of its
   leg-free value at three rates, and still > 1 (amplifying).
7. **THE SECOND FINDING** — `removed` exactly 0.0 and the armed cell bit-for-bit its leg-free
   march at every `sm` inside the band; the credit within 1e−12 of zero above it with the
   floor still binding on both; the band's edges reproducing the two plants' own min `φ`;
   width monotone in `b_max`.
8. **SCOPE + CYCLE UNTOUCHED** — every march `choked`, `clamped` = 0 on every cell, and the
   default single-spool design run bit-for-bit rung 6.

## Concessions

* **Only the FEEDFORWARD leg composes.** Rung 46/47's TIT governor and rung 49's `φ` floor are
  both feedback on a variable the bleed moves, so § 3's dichotomy is the general answer for the
  `φ` floor and no `Tt4_max` composite is offered. The `Wf/pt3` leg is the one whose sensed
  inputs are *pressures and speeds*, which is exactly why it can be re-timed rather than
  disarmed.
* **The valve is an IMPOSED position, not a controlled one** — rung 42's disclaimer, inherited
  through rung 62. A **φ-referenced bleed limiter** is still open, and § 3 now bounds what it
  could say: a controlled valve watching `φ` would be a floor and a point-mover at once.
* **`delta_match`'s magnitude is not claimed**, only its two raw halves' signs and orders — see
  § 1. The `ds`-sensitivity is published rather than tuned away.
* **`s_eng`'s magnitude rides on `margin` = 0.25**, rung 48's disclaimed scalar, and on the
  schedule placement `n_lo` = 0.65 (rung 62's, not rung 57's — which is why rung 58's own
  figure is not a control here). The claims are the SIGN and the per-cell ORDERING against a
  stator on the SAME placement, not a ratio. No head-to-head against a stator in
  fuel-withheld terms is offered — rung 57/62's currency concession, inherited.
* Inherited unchanged from rung 62: `φ_surge` is rung 36's imposed constant, `eta_c_at` is
  stator-inert, fully-choked branch, both NGVs choked, one `η_m`, no bypass, rung 35's
  forward-burner gas concession, the flat-η island out of scope for the march, rung 42's (3)
  thrust booking, no HP analogue for the valve, and no customer/cooling bleed.

## What it does to its neighbours

* **Rung 58 — BOUNDED, and its mechanism CONFIRMED.** Its one-way arrow is re-derived as a
  consequence of the leg's two protections rather than of the lever's kind; the arrow closes
  for a lever that breaks them. Its relocation × state-feed predictor reproduces this rung's
  forward direction at 85 % against its own 86 %.
* **Rung 59 — BOUNDED.** "A schedule's ordinate cannot see a stator" is a property of
  core-mass-preserving levers. This rung supplies the first counterexample, and the first
  non-degenerate splice — in which the two halves carry opposite signs.
* **Rung 60 — EXTENDED.** Its tautology is reproduced exactly (−1.33e−15) in the binding
  regime, and a second regime is added below it in which the lever *disarms* the floor.
* **Rung 62 — CONFIRMED.** Its § 2 loop-attribution survives a neighbour of a different kind:
  a leg with no `dn/d(setting)` perturbs the bleed's amplification by ≤ 0.8 %.
* **Rung 48 — reappears intact** inside a third composite: the forward arrow's collapse at
  `r` = 1.00 is its engagement law, with `s_eng` = 0.2947 the same witness rung 58 published.

## The next seam

**A `φ`-REFERENCED BLEED LIMITER** — a *controlled* valve, and § 3 is what makes it sharp: a
floor that watches `φ` and a lever whose entire credit runs through `φ` are, in this rung's
pair, two objects that cannot compose; made one object, the floor would move its own set point
as it acted. Rung 60's tautology and the `φ`-rate negative both bound what it can say, and
rung 52's *"a self-releasing limiter cannot debit the spool it watches"* predicts the shape.
Beyond that: **fuel + bleed + stator**, all three on one plant (this rung held to two, since a
third lever breaks the second difference's isolation), and rung 61's other opening — the
**station-3 customer bleed**, a different sink with a different arrow into the same two
sensed inputs.

## Anchor

`docs/plans/rung63-anchor-fuel-bleed.md` — the derivation written before any probe, the five
predictions as written before measuring, scored as eleven items: **seven HIT, one REFUTED
(P2b's sign, by a channel this rung's own § 1 had just derived), and three whose RESULT held
but whose stated REASON did not** — P2c demoted to a rung-58 confirmation, P1c's ratio
withdrawn as grid-sensitive, and P2a's "an order above rung 58" refuted twice over (wrong
schedule placement, then a real trajectory channel the prediction had conflated with the
table). The over-claims are published in § 3 with the tables that killed them.
