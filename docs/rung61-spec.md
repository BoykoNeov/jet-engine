# Rung 61 — STATOR + BLEED: a compensating lever buys back the COORDINATE, not the BILL

Rungs 36 and 41 closed with the same standing concession — *"no bleed valve / variable
stator, the devices that raise the margin at low speed."* Rung 42 built the valve, rung 53
built the stator, and every spec since has carried the same one-line seam:

> *"**stator + bleed together** — rung 53's saturation says the bleed takes over where the
> stator's authority ends."*

Rung 61 puts both devices on one steady machine. The seam's own sentence is **refuted**, and
so is the more attractive story that replaced it.

> **THE HEADLINE — a second lever can remove another lever's DEBIT exactly and still leave
> its BILL. Rung 42's valve buys back the whole of rung 53's `φ`-debit — machine-exactly —
> the compensated overspeed is still **0.73–1.02×** the stator's own, because its `φ`-drop was
> a partial REBATE on the loading it removed, and restoring `φ` forfeits the rebate. Past
> `v ≈ 0.3` the compensated machine overspeeds the UNcompensated one: undoing the lever is
> strictly worse than leaving it alone. So "undoing a lever" is a statement about a
> COORDINATE, never about a machine — and the pair's credits superpose to ≤ 2.3 % while its
> SPEED cost interacts adversely in every one of 30 rows.**

Four findings, and the order matters because each closes the escape the previous one opens.

---

## 0. The plant — the composition needs NO new solve

Rung 53's stator enters the solve in exactly one way: it replaces the map object
(`with_vsv`), which reaches `psi` and `solve_n`. Rung 42's valve enters in exactly one way:
it changes the cascade (the LP shaft balance and `(‡-b)`). **The two levers are
code-orthogonal**, so the composition is the MRO and nothing else:

```
StatorBleedMatcher -> TwoSpoolBleedMatcher -> VariableStatorMatcher -> TwoSpoolMapMatcher
```

rung 42's cascade run against rung 53's maps. Every method on the class is a **read**.

**Two silent-failure traps, both real, both closed in code** (they would have produced
plausible numbers with no exception):

1. `VariableStatorMatcher.at_setting` hard-constructs its own class, and *every* rung 53/54
   instrument routes through it (`stator_sweep`, `currency_split`, `incidence_schedule`,
   `_scan`, `authority_ceiling`, `schedule_throat`). Un-overridden, every sweep would have
   run **with the valve shut**. Overridden here to carry `self.bleed`.
2. `TwoSpoolBleedMatcher.__init__` forwards a fixed argument list carrying **no vsv**. Under
   the new MRO a co-operative `super()` chain lands on rung 53's `__init__` with `vsv = 0`
   and the **stators silently never move**. So rung 53's `__init__` is called *explicitly*.
   This is the one place the two ladders do not compose, and it is commented as such.

## 1. The seam AS POSED — REFUTED, and in the opposite direction to this rung's own prediction

"Takeover" says the stator works until it saturates and the valve then carries on: the
valve should be **indifferent** to the stator's ceiling. Rung 54's `authority_ceiling` at
four valve positions (LP, `flow/press`):

| `Tt4` | `b`=0 | 0.05 | 0.10 | 0.15 |
|---|---|---|---|---|
| 1500 `v_edge` | 1.2400 | 1.2000 | 1.2000 | 1.1600 |
| 1500 `M_i` span | 0.20856 | 0.20268 | 0.19787 | 0.19079 |
| 1000 `v_edge` | 2.5200 | 2.4400 | 2.3600 | 2.2800 |
| 1000 `M_i` span | 0.38133 | 0.36195 | 0.34206 | 0.32168 |

Opening the valve **shrinks** the stator's remaining authority, monotonically, at every
throttle. The anchor pre-registered the opposite sign (P1a predicted `v_edge` rises ≥ 5 %)
and it is scored a miss.

**`v_edge` is rung 53's admitted `solve_n` artifact edge, so the refutation does not rest on
it.** Two artifact-free reads carry it. First, `M_i(v=0)` **rises** with `b` (0.818 → 0.927 at
`Tt4` = 1500) by more than the peak rises — **the valve pre-spends what the stator was going
to buy**, which is a plain margin read at a fixed setting. Second, the four-cell superposition
table in `M_i`, 5 shapes × 2 throttles × 3 `(v,b)` pairs:

```
    interaction / (credit_stator + credit_bleed)  ∈  [−2.28 %, +0.45 %]
```

with a **shape-dependent sign** (positive on `flow/press` and `steep`, negative on the other
three). So the two levers are not a sequence and not a synergy: they are **substitutes
drawing on one incidence budget**, additive to ~2 %.

## 2. THE HEADLINE — the debit goes, the bill stays

Define `b*(v)` as the valve setting that restores `φ_op` to its bare value. On the LP spool
it exists and is mild. At the compensated point, by construction and exactly:

```
    ΔM_i  = + v                             ΔM_φ = + v φ_s0²/(1 + v φ_s0)
```

Residuals ≤ 1.2e-11 at every `(Tt4, v)` tried, including the flat-η island, and `ΔM_φ` is
**identical at every throttle**. **This is NOT a finding.** It is rung 60's published
tautology (`docs/rung60-spec.md`: *"leg floors φ ⇒ … = v"*) arriving by a **third route** —
rung 60 reached it by **pinning** `φ`, this reaches it by **restoring** `φ`, which shows the
identity needs no floor at all. It is gated for exactness and demoted to a lemma. Its real
use is as evidence for the headline: *the identity holds no matter what the machine did.*

What the machine did:

| `Tt4` | `v` | `b*` | `Δn` stator → comp | **retained** | `ΔF` stator → comp |
|---|---|---|---|---|---|
| 1500 | 0.10 | 0.09020 | +6.42 % → +4.71 % | 73 % | −0.12 % → −8.91 % |
| 1500 | 0.20 | 0.18636 | +13.06 % → +11.06 % | 85 % | −0.45 % → −18.68 % |
| 1500 | 0.30 | 0.29206 | +19.87 % → **+20.25 %** | **102 %** | −0.96 % → −30.00 % |
| 1300 | 0.20 | 0.15118 | +11.55 % → +9.30 % | 81 % | −0.58 % → −15.91 % |
| 1100 | 0.20 | 0.12254 | +10.19 % → +7.90 % | 78 % | −0.64 % → −14.19 % |
| 1100 | 0.30 | 0.18984 | +15.52 % → +13.36 % | 86 % | −1.05 % → −22.43 % |

The `φ`-debit is gone; the overspeed is not. Stated as an inequality on the **raw**
excursions rather than as a fraction of a collapsing base (rungs 42/43's own confound):
`Δn_comp ≥ 0.70·Δn_stator` at every row, and `Δn_comp > Δn_stator` at (1500, 0.30). Rung
42's own thrust bill (up to −30 %) is now on top of it. The anchor predicted retention < 50 % (P5a) and that
refutation is what produced this rung.

### THE MECHANISM — the `φ`-debit was carrying a rebate

`n` solves `τ_c = 1 + (τ_d−1)·ψ(φ)·n²`. At `Tt4` = 1500, `v` = 0.20:

| cell | `b` | `φ` | `n` | `ψ` | `τ_c` |
|---|---|---|---|---|---|
| bare | 0 | 1.00000 | 1.00000 | 1.00000 | 1.40971 |
| stator | 0 | 0.88106 | 1.13062 | 0.78228 | 1.40971 |
| compensated | 0.18636 | 1.00000 | 1.11060 | 0.66000 | 1.33353 |

The compensated point is **more unloaded than the stator-only point** (0.660 < 0.782), and
that is the whole story. `ψ = base(φ) − v(1+l)φ`: the stator's `φ`-drop raises `base(φ)` by
**+0.0833**, which is the map's own loading law **partly paying back the swirl the stator
removed**. Restoring `φ` **forfeits the rebate**, and simultaneously makes the swirl term
`−v(1+l)φ` larger because `φ` is larger. So `Δn_comp` factorises into

```
    loading term  +23.09 %      fighting      demand term  −9.77 %   (bleed's lower τ_c)
```

and at `v = 0.30` the loading term (+42.86 %) **beats** the demand term (−15.83 %): the
compensated machine overspeeds the bare stator.

**Where the crossover is and is not reached, stated precisely.** From `v` = 0.20 → 0.30 at
`Tt4` = 1500 the loading term grows **1.86×** while the demand relief grows only **1.62×**,
so the gap closes with setting — that trend is the reason, not a coincidence of one point.
But it is a *trend*, not a theorem: at `Tt4` = 1100, `v` = 0.30 retention is still 86 % and
the crossover has **not** been reached. The claim is its **existence and sign** at high power
and large setting, gated there and nowhere else.

> **The arithmetic that `ψ_comp = base(φ_bare) − v(1+l)φ_bare` reproduces to 1e-12 on all
> five shapes is `psi` evaluated at a known argument — an IDENTITY, and it is gated only as
> a plumbing check that `at_point` composes the two levers correctly. The *finding* is the
> `base` slope's sign and size (the rebate), and the two-term factorisation above.**

### The price collapses on the map's own loading slope

`l` spans 0.7 → 1.2 across the five disclosed shapes. At fixed `(v, Tt4)`, `b*/[v(1+l)]`
spreads only **1.25–3.3 %** (6.7 % at the largest setting tried). The price's **entire**
shape-dependence is `(1+l)`; its level rides on `v` and the throttle and is disclaimed.

**A ceiling is NOT claimed.** The walk dies at `b ≥ _B_CAP`, a constant this rung introduces
(rung 42's own bound is `b < 0.5`), and re-running at caps 0.35 / 0.42 / 0.45 moves the
last-compensable `v`, as a ratio to `1/(1+l)`, from 0.55 → 0.68 → 0.70. **The level is where
the walk was truncated**, so only the cap-free price scaling is published. (Rung 42's
envelope guard never binds first: the death reason is `b ≥ cap` at every throttle 1500 → 900
on two shapes.)

## 3. Compensability is SPOOL-dependent — the levers do not span the same space

Rung 53's stator is a lever on **either** spool. Rung 42's valve is a lever on **one**
(its `dφ_H/db` passes through zero at `π* = 3.24674` and reverses below). So a stator debit
is compensable only where the two overlap. On the HP spool, at `v` = 0.20:

| `Tt4` | `π_HPC` | stator spends | **whole** valve range returns | shortfall |
|---|---|---|---|---|
| 1500 | 6.0000 | −0.125684 | +0.026294 | −0.099390 |
| 1300 | 5.1833 | −0.117298 | +0.020348 | −0.096950 |
| 1100 | 4.3892 | −0.109970 | +0.013369 | −0.096601 |
| 900 | 3.6303 | −0.104650 | +0.005944 | −0.098706 |

Short by **4–17×**, and the shortfall is **throttle-invariant to 3 %**. The anchor predicted
a *divergence toward `π*`* (P2b) — mechanism right, shape wrong: it is not a blow-up at one
pressure ratio, it is **uniformly unavailable across the whole choked band**.

**`π*` is NOT billed as a fourth independent appearance.** It is rung 42's own crossing seen
in a new currency, and it turned out not to be the operative boundary at all — the valve is
too weak on the HP everywhere, not just at its sign change.

## 4. CORRECTION of rung 53 — per-spool cleanliness does not survive composition

Rung 53's P5: *"the stator is a **cleaner per-spool DoF than rung 42's bleed valve**"* —
`vsv_lp` leaves `φ_HP`/`n_HP` **bit-identical**, and rung 53's own inter-spool arrow is
η-mediated, so a flat-η island switches it **off**.

| | `Δφ_HP` at `v` = 0.20 |
|---|---|
| stator alone (shaped maps) | **+0.000000e+00** (rung 53's exact zero, reproduced) |
| compensated pair (shaped) | +1.553991e-02 |
| compensated pair, **flat-η island** | **+1.571717e-02** — the arrow **SURVIVES** |

Buying the LP debit back requires the dirty lever, and bleed's arrow is the **energy**
channel (the shared `Tt25`), which no flat map can switch off. So **rung 53's cleanliness is
a property of the lever in ISOLATION and is lost under composition** — a pair inherits the
dirtier lever's arrow. Rung 53's claim is corrected in scope, not refuted: the isolated
zeros still hold, and this rung reproduces them as its control.

### The cost interaction, and a machine-zero that turned out to be RUNG 53's, not this rung's

The four-cell interaction in the **costs**, five shapes:

* **speed** `i_n`: **positive in all 30 rows** (+0.19 % … +1.81 %) — the pair always costs
  more shaft speed than the sum of its parts. This is a real interaction and it is gated.
* **thrust** `i_F`: positive on the four shaped maps (+0.02 % … +0.70 %) and **exactly `0.0`**
  on the **flat-η island** — but that zero is **NOT a claim about the pair**, and the check
  that established it was run before publishing.

**Why the thrust zero is not this rung's finding.** On a flat-η map the stator's *own* thrust
effect is **exactly `0.0`** (raw difference `0.0`, at `v` = 0.10 / 0.20 / 0.30, `Tt4` = 1500
and 1200), so one term of the four-cell difference vanishes identically and the interaction is
zero for free. Framing it as "the pair's thrust interaction is η-mediated" would have been an
overclaim.

**What the check found instead sharpens rung 53's P1.** Rung 53 reported the stator as
*thrust-neutral*, measured as "specific thrust flat to < 0.5 %". Switch the efficiency island
off and it is not approximately neutral — it is **exactly** neutral:

| flat-η, `Tt4` = 1500 | `v` = 0.10 | 0.20 | 0.30 |
|---|---|---|---|
| `ΔF` (stator alone) | **+0.000000e+00** | **+0.000000e+00** | **+0.000000e+00** |
| `Δn` (stator alone) | +6.46 % | +13.22 % | +20.28 % |

So **the whole of the stator's thrust cost is the efficiency island**, and with that island
flat it is a *pure* speed lever — rung 53's P1 ("thrust-neutral, paid in shaft speed") upgraded
from a tolerance to a machine zero. The pair's thrust interaction then vanishes as a corollary,
and is reported as one.

## 5. Two loci — the PRICE is coordinate-dependent too

"Restore the point" (`φ_op`) and "restore the reported margin" (`M_φ`) are different
instructions, because the stator moved the floor between them.

| `Tt4` | `v` | `b*_φ` | `b*_{M_φ}` | gap |
|---|---|---|---|---|
| 1500 | 0.10 | 0.09020 | 0.04952 | +0.04068 |
| 1500 | 0.20 | 0.18636 | 0.10686 | +0.07950 |
| 1500 | 0.30 | 0.29206 | 0.17273 | +0.11933 |
| 1200 | 0.10 | 0.06612 | 0.02509 | +0.04104 |
| 1200 | 0.20 | 0.13606 | 0.05516 | +0.08090 |
| 1200 | 0.30 | 0.21123 | 0.09035 | +0.12088 |

`b*_φ > b*_{M_φ}` strictly, the gap grows with `v`, and — unpredicted — **the gap is
throttle-invariant to 1.5 %** while each price separately moves by 27–37 %. The ladder
extends once more: rung 53 found a **margin** coordinate-dependent, rung 54 a **constraint's
severity**, rung 56 a **lever's cost**, and rung 61 the **price of undoing one lever with
another**.

---

## Reduce-to-prior contract (the spine) — TWO-AXIS

Stronger than either parent's alone, because the machine has two device coordinates:

```
    (v = 0, b = 0)   =>  rung 39 bit-for-bit   (valve dispatches away; the maps are the
                                                SAME OBJECTS; `match` is inherited)
    (v ≠ 0, b = 0)   =>  rung 53 bit-for-bit   (the dispatch lands on rung 53's own
                                                inherited path — an IDENTITY)
    (v = 0, b ≠ 0)   =>  rung 42 bit-for-bit   (the cascade sees the DESIGN maps)
```

Verified on 15 matched fields at `Tt4` = 1500 / 1200 / 950. Rung 42's `_cascade_bleed` and
rung 53's `__init__` capture discipline are left **literally unchanged**, so the rung-39/41/
42/53/54 suites keep witnessing them. The default `build_turbojet(…).run(…)` design path is
untouched ⇒ **bit-for-bit rung 6**.

## Verification gates (`tests/test_rung61.py`)

1. **REDUCE — TWO-AXIS**, all three corners bit-for-bit (`==`), on the fast gas and the
   reacting gas, across throttles.
2. **THE TWO TRAPS, asserted directly** — a moved-stator combined matcher really has
   `map_lp.vsv != 0` under the MRO, and `at_setting` returns a `StatorBleedMatcher` carrying
   `bleed` (both would otherwise be silent wrong numbers).
3. **THE HEADLINE — retention.** At `b*`, `φ_op` is restored to the bare value while
   **≥ 70 %** of the stator's `Δn` survives, at every point of the table; and at `v` = 0.30,
   `Tt4` = 1500 the compensated point's `Δn` **exceeds** the bare stator's (the crossover,
   asserted as a strict inequality — the sign is the claim).
4. **THE MECHANISM.** `ψ_comp < ψ_stator` (more unloaded, the rebate forfeited), `base(φ)`
   rises as `φ` falls, and the two-term factorisation reproduces `Δn_comp` — plus the
   `ψ_comp` closed form to 1e-11, gated **as a plumbing check** and labelled as an identity.
5. **RUNG 60's TAUTOLOGY, third route.** `ΔM_i == v` and `ΔM_φ == v φ_s0²/(1+vφ_s0)` to
   `1e-10`, at ≥ 6 `(Tt4, v)` points and on the flat-η island; `ΔM_φ` throttle-invariant.
   Gated as an identity, explicitly not as a finding.
6. **THE SEAM AS POSED, refuted.** `v_edge` and the `M_i` span are **monotone decreasing** in
   `b` at ≥ 2 throttles; `M_i(v=0)` rises with `b`. Plus the artifact-free version: the
   4-cell credit interaction is `< 3 %` of the credit sum on all five shapes.
7. **SPOOL-DEPENDENCE.** `b*_LP` exists at every throttle; `b*_HP` is `None` at every
   throttle with reason `"valve authority exhausted"`; the HP shortfall is ≥ 3× the whole
   valve's return and is throttle-invariant to 10 %.
8. **THE PRICE COLLAPSE.** `b*/[v(1+l)]` spreads `< 5 %` across the five shapes at fixed
   `(v, Tt4)`, at ≥ 2 throttles. **And the NEGATIVE control that keeps it honest**: the
   cap-ratio test — re-running the ceiling walk at two caps moves the last-compensable `v`,
   so the gate asserts the ceiling is cap-dependent and must not be claimed.
9. **RUNG 53 CORRECTED, and RUNG 53 SHARPENED.** `Δφ_HP == 0.0` exactly for the stator alone
   (rung 53's zero, reproduced — so the correction is not vacuous), `≠ 0` for the pair, and
   **still ≠ 0 on the flat-η island**. Separately, rung 53's P1 upgraded: the stator's own
   `ΔF` is **exactly `0.0`** on a flat-η island while `Δn > 6 %`. The pair's flat-η thrust
   interaction is asserted as a **corollary** of that, not as a finding; the adverse **speed**
   interaction is the gated claim.
10. **CYCLE UNTOUCHED** — the default single-spool design path is bit-for-bit rung 6.

### What the RUST PORT measured about these gates (slice O, 2026-08-17)

The port re-ran this rung's own instruments against a bit-exact oracle and swept past the
suite's grid. Four things it found that this spec did not say:

1. **`_feasible`'s `try/except` swallows NOTHING on the shipped grid.** Over 10 613 calls in
   320 `compensating_bleed` cells, the plant refused **zero** times. Its docstring's claim —
   *"the feasible set is bounded on BOTH axes, by different mechanisms"* — is **CONFIRMED** by a
   1 760-cell wide sweep (756 refusals under exactly two mechanisms: the speed-line bracket
   bounds `v` between 1.2 and 1.3; the choked envelope bounds `b` at 0.49, and only near the
   throttle edge, `Tt4 = 700`) — but **its SCOPE is corrected: both bounds sit entirely outside
   every shipped test.** Two of `compensating_bleed`'s three `None` branches
   (`stator setting infeasible`, `choked envelope closed`) are therefore dead on the whole
   suite; only `valve authority exhausted` is live, at 124 of 320 cells.

2. **Two dead things, of different kinds.** `_B_MAX = 80` is never approached (22–30 passes over
   all 196 solved calls) — a dead CAP. And the bisection's exit is
   `abs(r) <= _B_TOL or hi - lo <= 1e-15`, whose **second disjunct never fires**: 196 of 196
   exits are on the tolerance. That is a dead ARM of a live condition, which is the more easily
   lost of the two.

3. **`compensability`'s `ratio` uses Python truthiness**, `(bh / bl) if (bl and bh)`, which
   treats an exact `0.0` as absent where `is not None` would not. Measured **latent**: no
   `b* == 0.0` on any grid swept (196 values, min 8.54e-3), and on the shipped throttle band
   every row is mixed anyway. Recorded so nobody "simplifies" it into a different function.

4. **The two tolerance-style bars have measured headroom**, recorded here so later erosion reads
   as erosion: gate 3's retention worst is **0.73339** against its `>= 0.70`, and gate 6's
   credit interaction worst is **0.01686** against its `< 0.03`.

Neither this rung's verdicts nor its numbers move. See `docs/plans/todo-rust-port.md` § 5.11.

## Concessions

* **Steady only.** Rung 42's valve is not read by any transient ladder and rung 57 already
  owns the stator on the transient, so a **scheduled** `b(n_L)` beside a `v(n)` schedule is a
  different rung. No surge-*survival* claim is made.
* **`b*` is a constructed operating point, not a control law.** It is the answer to "what
  would it take to undo this?", root-found offline. Nothing schedules it.
* **`_B_CAP` is this rung's own constant** and it is what binds the compensable range. The
  ceiling it induces is explicitly **not** published; only the cap-free price scaling is.
* **Every magnitude rides on the two representative maps, the imposed `φ_surge = 0.55`, and
  `b` as an imposed valve position** — rungs 36/41/42/53's inherited disclaimers, now
  stacked. Load-bearing: the **retention range**, the **crossover's existence and sign**, the
  **rebate's sign**, the **spool asymmetry**, the **`(1+l)` collapse**, the **two exact
  zeros**, and the **two identities**.
* **`v_edge` is rung 53's `solve_n` artifact**, so § 1's table is corroboration and the
  4-cell interaction carries the claim.
* **The incidence peak is not interior on these shapes**, so `v_peak == v_edge` and P1c was
  not scorable.
* Inherited unchanged: fully-choked branch, both NGVs choked, one `η_m`, no bypass,
  isentropic knobs, rung 35's forward-burner gas concession, no customer/cooling bleed.

## The next seam

**A `b(n_L)` schedule beside a `v(n)` schedule, on the transient plant** — the one question
this rung's steady answer makes sharp, because § 2 says the two devices' *costs* do not
share and § 1 says their *credits* do not stack. Beyond that: the stator + bleed pair on the
**HP-exit (station 3) customer bleed**, which is a different sink with a different arrow, and
whether the rebate of § 2 has an analogue for any lever whose debit is in a coordinate the
loading law also reads.

## Anchor

`docs/plans/rung61-anchor-stator-bleed.md` — the six predictions as written before measuring
(**three scored wrong**, and two of those produced the rung), the probe tables, and the cap
check that killed the ceiling before it was published.
