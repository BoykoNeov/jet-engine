# Rung 56 — PER-ROW CAPACITY: two constraints on one machine, at opposite ends

Rung 55 named this seam and predicted its answer:

> *"`X(v) = m·sqrt(1+v²)` is a face quantity; in a stack each row has its own throat and its own
> `X_k`, and P4 says the REAR rows are the ones driven to high `φ`. Rung 54's `capacity_margin`
> should therefore **bind at the back** while its incidence margin binds at the front — the two
> rungs' objects on the same machine, in opposite places. **It needs a `C` per row.**"*

Both halves move. The seam is **right at part power and wrong near design**, for a reason it did
not anticipate; and it does **not** need a `C` per row — the stack already knows all but one.

---

## THE HEADLINE

> **A resolved machine's two binding rows are DIFFERENT rows — different END and different
> SPOOL. The one lever that exists reaches only the wrong one: a front-row stator restores the
> row that STALLS and DEBITS the row that CHOKES, through the shaft speed they share.**

At `Tt4` = 800, `K` = 8, `C` = 0.90, on all five disclosed shapes — these are the **machine-wide**
minima, over all 16 rows:

```
    worst INCIDENCE margin :  LP  row 0  (front)   M_i = 0.349
    worst CAPACITY  margin :  HP  row 7  (rear)    M_c = 0.164
```

*Stated precisely, because two separate facts are stacked here.* **Per spool** the ends are the
same on both: each spool's incidence binds at its own front and its capacity at its own rear
(LP rear `M_c` = 0.448, HP front `M_i` = 0.678 — both are their spool's binding row for their own
currency). The **cross-spool** content is carried by the two machine-wide comparisons: the
incidence exposure is the LP's and the capacity exposure is the HP's. It is the *combination*
that puts the machine's two binding rows at diagonally opposite corners.

Rung 55's seam predicted *front-vs-back on one machine*. Measured, the separation is larger than
that: the two constraints are not merely at opposite ends of a compressor, they are on **different
spools**. Rungs 41/44/45/53 put the surge exposure on the LP; rung 55 P1 found the *stack*
correction landing harder on the HP; this rung shows the **second constraint lives there too**.
A lumped block has one `φ` and one face — it cannot express either statement, let alone their
separation.

And rung 55's law extends. Rung 55: *a positional lever buys its relief from the part it does not
move*. Rung 56: **the part it does not move is where the OTHER constraint lives** — so the
lever's benefit and its debit are not just in different currencies, they are in different
**places**.

---

## The instrument — the seam asked for `K` constants; the ladder supplies `K−1`

### The profile is DERIVED (rung 54's pattern, one level up)

At design the stack sizes every annulus so `φ_k` = 1, hence `Vx_k` = `U_k`; on a constant mean
radius `U_k` = `U`, so **every row has the same design throat velocity** while `Tt_k` climbs the
ladder. It is the *total-referenced* Mach `ν ≡ V/sqrt(γRTt) = M/sqrt(1+(γ−1)/2·M²)` that scales
at a common velocity, so

```
    ν_k = ν_1 / sqrt(θ_k,d)        M_k = ν_k / sqrt(1 − (γ−1)/2·ν_k²)        C_k = MFP(M_k)/MFP(1)
```

**ONE disclosed LEVEL** — the front row's `C`, which is rung 54's constant *unchanged* (its row
was already *"one row at the compressor face"*) — and **`K−1` rows DERIVED** off the stack's own
design temperature ladder. Rung 54's discipline exactly: shape derived, level disclosed, verdicts
as thresholds on the level. `k` = 0 returns the disclosed constant *exactly* rather than
round-tripping it through `design_throat_mach`'s bisection, so the `K` = 1 reduce is bit-for-bit
and independent of `γ`.

Measured (`K` = 8, `dT`, `C` = 0.90 ⇒ `M_th0,front` = 0.678):

```
    LP  M_k = 0.678 0.660 0.643 0.628 0.613 0.600 0.587 0.575    C_K/C_1 = 0.911
    HP  M_k = 0.678 0.646 0.617 0.592 0.570 0.550 0.532 0.516    C_K/C_1 = 0.848
```

The HP profile falls harder because its `τ_d` is larger — a functional of the ladder, gated, not
fitted.

### And the derived profile FIGHTS the seam

The rear rows come out designed with **more** capacity margin (lower Mach) exactly where the
off-design march loads them hardest. Against it, the per-row corrected flow

```
    m_k = φ_k · n_k            EXACTLY -- the face identity m = φ·n, at every station
```

(the march already computes both factors, so no new object) runs **ahead of `φ_k`** because the
rear rows' corrected *speed* rises too. **Which end binds is a contest, not an inspection** —
which is why the instrument was probed before the prediction was written (anchor § PROBE A).

### The disclosed alternative, and it is NOT furniture

`cap_profile = "uniform"` (`C_k` = `C`, rung 54's constant applied per row without the ladder)
is kept as the control. Unlike rung 55's work split it **carries the levels** — see P4.

### Diagnostic-only, inherited BY THEOREM

Rung 54 P1: the throat enters no solver, so it is a post-hoc functional of the already-solved
state and can only remove settings from the feasible set, never relieve. Unchanged here.

---

## The reduce — an INVARIANCE, on a stack that DOES enter the solver

Rung 54 earned an invariance over `C` on a channel that entered no solver at all. Rung 55's stack
**does** enter the solver, and `capacity`/`cap_profile` ride on the very objects
(`ComponentMap`, `StageStack`) the speed-line inversion consumes — so the invariance is no longer
free and must be asserted. Measured: every matched field bit-identical across `C` ∈ {0, 0.30,
0.70, 0.90, 0.99} × both profiles, at a **moved stator**, including a `C` at which the binding row
provably chokes. Plus `K` = 1 reproducing rung 54's own `throat_margin` — `X`, `M_c`, `c_min` — to
the last bit, on both profiles and at a moved stator.

---

## P1 — the binding row MIGRATES: the seam HIT at part power, REFUTED near design

Binding row against throttle (1500 → 800, one digit per throttle, `K` = 8, `C` = 0.90):

| profile | LP | HP |
|---|---|---|
| derived | `0 0 7 7 7 7 7 7` | `0 0 0 7 7 7 7 7` |
| uniform | `7 7 7 7 7 7 7 7` | `7 7 7 7 7 7 7 7` |

**Near design the PROFILE binds (front row, highest Mach); at part power the LOADING binds (rear
row); the migration is one-way**, on all five shapes and both spools. Strip the profile and the
contest disappears — `X_k` alone decides and the rear binds everywhere.

The LP crossover sits between `Tt4` = 1400 and 1300 on every shape; the HP's between 1300 and
1200. **Pre-registered band (LP crossover inside [1200, 1500]): HIT.** The seam's *"binds at the
back"* is **HIT at part power and REFUTED near design** — and the refutation is *derived*, not an
artifact: it is the machine's own design Mach distribution.

*The crossover LOCATION is disclaimed* on both `C` and the profile (it moves with the MFP
curvature); its **existence** is the claim.

---

## P2 — the non-tautology gate is a RESOLUTION gap, not a feedback one

Rung 55's stack changed `τ_c(m,n)` and so had a feedback leg. Rung 56 has none, by rung 54's
theorem. What makes it content is **resolution**: at the *same solved state*, how much of the
throat loading could rung 54's **face** read not see?

```
    amplification = (1 − M_c at the binding ROW) / (1 − M_c at the FACE)
```

| `Tt4` | LP derived | HP derived | LP uniform | HP uniform |
|---|---|---|---|---|
| 1200 | 1.061 | 1.000 | 1.165 | 1.178 |
| 1000 | 1.184 | 1.136 | 1.299 | 1.340 |
| 800  | 1.323 | 1.317 | 1.453 | 1.552 |

**Exactly `1.0` at `K` = 1** (the binding row *is* the face — and it is bit-exact, since
`φ_0` ≡ `m/n` and `n_0` ≡ `n`), growing monotonically with throttle depth, ≥ 1.15 on the HP at
`Tt4` = 800 on **every shape and both splits**. **Pre-registered band: HIT**, including the
predicted ordering `uniform > derived` — the derived profile is *protective*.

`X_{K−1}` sits at an **interior ladder station**: it is not recoverable from `(τ_c, π_c)`, so it
genuinely needs the march.

---

## P3 — `K` is a resolution here too (band MISSED, claim HIT)

Amplification increments at `Tt4` = 800 over `K` = 1→2→4→8→16→32:

```
    LP   0.1887  0.0901  0.0446  0.0223  0.0112      ratios 0.48 0.50 0.50 0.50
    HP   0.1754  0.0906  0.0507  0.0277  0.0147      ratios 0.52 0.56 0.55 0.53
```

First-order convergent on both, so nothing rides on a particular `K`. **Scored as written: the
pre-registered band was "at least halves"; the LP halves, the HP shrinks by ~0.53 — a MISS on the
band, a HIT on what it encoded.** The gate is set at the measured 0.60, not at the prediction.

---

## P4 — the disclosed SPLIT is LOAD-BEARING here, inverting rung 55 P6's pattern

The amplification rides on the internal `θ`/`ϖ` ladder — which is precisely what the work split
moves. Rung 55 P6 measured the split moving `d_φ` by **0.01 %**; here it moves the amplification
by **3.7–4.6 %** (HP, `Tt4` = 800, all five shapes), ~400×.

**Pre-registered and HIT** (band: > 2 % on the HP at 800). The honest half: the prediction also
said *"every binding-row identity unchanged"*, and that **fails in the near-crossover cells**
(`press/flow` HP at `Tt4` = 1200 binds row 2 on `dT` and row 3 on `tau`, where the whole spread
across rows is ~1e-3). So: **the LEVELS are disclaimed on the split; the SIGNS and the part-power
binding row are not.** The relative metric also blows up near the crossover (`x−1` → 0) and is
reported only away from it.

---

## P5 — the HEADLINE measured, and rung 54 CORRECTED

Rung 54 § The exposure split wrote:

> *"The capacity ceiling is a **pure-LP** object. The HP schedule's demand *falls* monotonically
> and **never approaches its throat at any throttle**."*

At the **face** that is true and stays true. Resolved into rows it is nearly false — and the two
read *opposite signs in throttle*:

| `Tt4` | HP **face** `M_c` | HP **rear row** `M_c` | `C*` (row chokes above) |
|---|---|---|---|
| 1200 | +0.2163 | +0.0771 *(uniform)* / front binds *(derived)* | 0.975 |
| 1000 | +0.2925 | +0.1961 | 0.950 |
| 800  | +0.3647 | +0.1636 | **0.913** |

**The face RELAXES with throttle while the rear row TIGHTENS.** Stated as rung 54 requires, a
threshold **on** the constant: *any HP row whose design capacity fraction exceeds 0.913 is choked
at `Tt4` = 800.* On the naive uniform profile that row sits at `M_c` = **+0.0139** — a hair from
the wall; the derived profile is what holds it at +0.164. **A machine pays for its rear rows'
off-design capacity at DESIGN, in the front row's Mach.**

**This is the rung-28 shape:** the face-level reasoning survives *as a face-level statement*, and
the verdict it supported is corrected by resolution. Rung 54's LP claim was about the *stator
schedule's* demand; this one is about the *bare* running line. Two mechanisms, and only a resolved
compressor separates them.

---

## P6 — the positional lever's cost is CURRENCY-DEPENDENT (predicted band REFUTED)

**The sign first, measured before the headline was fixed** (anchor § PROBE B2). The front-row
lever reaches an unmoved rear row only through the solved `(m, n)` — and the sign is a **DEBIT**,
monotone in `v`. Rung 55's honest half (*the shaft speed is the one thing every stage shares*),
now in rung 54's currency.

The prediction was that the rear-row debit ratio (front-only / lumped) would track rung 55's `dN`
ratio within 25 %. `Tt4` = 1000:

| `v` | rear-row debit ratio | `dN` ratio |
|---|---|---|
| 0.20 | 0.133 | 0.122 |
| 0.354 | 0.073 | 0.120 |
| 0.60 | 0.045 | 0.117 |

**MISS — and the miss is the finding.** The **speed** ratio is nearly `v`-invariant; the
**throat** ratio *collapses* with the setting. The lumped lever spends every row's throat
*directly*, by `sqrt(1+v²)`, on top of the speed rise, so the positional lever's advantage is
larger — and **grows** — in exactly the currency rung 54 introduced. **Rung 53's law a fourth
time: not only is a margin coordinate-dependent, and a constraint's severity (rung 54), and a
lever's benefit (rung 55) — so is a LEVER'S COST.**

**And the lever relocates the binding row to itself.** Push the front-row setting far enough
(`v` ≈ 0.6 at `Tt4` = 1200, `v` ≈ 0.9 at 1000/800) and its own `sqrt(1+v²)` overwhelms the rear's
loading, so the binding capacity row moves to the moved row. **Rung 50's shape in a currency rung
50 never saw** — and the threshold sits well above rung 55's own front-row schedule (`v*` ≈ 0.35),
so rung 55's published lever does not trip it.

---

## Scope and concessions

- **DIAGNOSTIC ONLY, by rung 54's theorem, and gated.** A choked row changes nothing solved.
  Making the compressor row the *binding* throat would invert rung 31's `(★)` and restructure the
  matching cascade — a different, larger rung. Inherited explicitly, because the rear row at
  `C` = 0.90 sits close enough to the wall to be tempting.
- **CONSTANT MEAN RADIUS is a disclosed geometric choice**, not a derivation: it is what makes
  `U_k` = `U`, hence a common design throat velocity, hence the profile. A different radius law
  gives a different profile with the same method.
- **`Vx` is constant through the stack**, forced by `φ_k` = 1 everywhere (rung 55's uniform
  annulus sizing). Real practice lets `Vx` fall rearward, which would make `M_k` fall *faster*
  — so **the derived profile here is conservative**, and the crossover would move to lower
  throttle. The claim is bounded in the safe direction.
- **`γ_th` = 1.4 for the throat MFP** — a disclosed CPG placement, rung 41's `(★)` / rung 55's
  `kc` precedent. It cannot touch the `K` = 1 reduce (`θ_d[0]` ≡ 1), gated over three `γ`.
- **The profile carries the LEVELS** (P4's lesson applied to itself): every level is disclaimed
  on both `C` and `cap_profile`; only signs, orderings and thresholds-on-`C` are load-bearing.
- **Inherited**: rung 55's stack and all its concessions (all stages share one map, one `K` per
  spool, `π_c` not re-derived, steady + two-spool only), rung 54's thin-vane cosine throat, no
  vane-row loss, and rungs 36/41's imposed `φ_s0` as the incidence anchor.
- **Still one map per stack**, so the per-row *blading* differs only through the ladder. Rung
  55's standing concession, and it bounds this rung too.

## Verification gates (`tests/test_rung56.py`)

1. **REDUCE — an INVARIANCE over the constant AND the profile**, on a stack that enters the
   solver; plus `K` = 1 reproducing rung 54's `throat_margin` bit-for-bit; plus a hand-built
   one-stage stack carrying the constant exactly for three `γ` and both profiles.
2. **THE DERIVED PROFILE** — `C_0` is the disclosed constant *exactly*, `C_k` falls monotonically,
   `ν_k·sqrt(θ_k,d)` is invariant (the derivation itself), the HP falls harder than the LP, and
   the disclosed alternatives are rejected outside `{derived, uniform}`.
3. **THE PER-ROW CURRENCY** — `m_k` ≡ `φ_k·n_k`, `X_k` is rung 54's law at the row's OWN setting
   (only the front block carries it), and the design tie is a **tolerance** with the drift
   asserted *non-zero* so the noise warning cannot rot.
4. **THE NON-TAUTOLOGY GATE** — amplification exactly `1.0` at `K` = 1, growing with throttle
   depth, ≥ 1.15 on the HP at 800 on every shape × both splits; `uniform` > `derived`.
5. **P1** — front binds near design, rear at part power, migration one-way, all five shapes; and
   the uniform control binding rear at every **off-design** throttle (design excluded by gate 3).
6. **P3** — increments positive, shrinking, and geometric (ratio < 0.60, the measured level).
7. **P4** — the split moves the amplification (> 2 % on the pre-registered HP-800 cell, > 1.7 %
   elsewhere — the measured floor, not the prediction rounded up) without moving a sign.
8. **P5 — THE HEADLINE** — opposite ends *and* opposite spools on all five shapes; rung 54's HP
   claim corrected with the sign flip in throttle and `C*` < 0.92; and the diagnostic-only
   refusal gated at a `C` that provably chokes.
9. **P6** — the debit's sign and monotonicity, its concentration on the moved row (> 10×), the
   currency divergence (throat ratio collapses, speed ratio does not, > 2× apart), and the
   relocation threshold sitting above rung 55's schedule.
10. **CYCLE UNTOUCHED** — the default single-spool design run is bit-for-bit rung 6.

## The next seam

> **⚠ CLOSED, NEGATIVE — `docs/per-row-blading-negative.md`. Do NOT re-open as written.** The
> paragraph below is kept as the record of what was asked, but its last sentence is **false**:
> the ladder *does* supply a `ψ_k`/`T_c,k` law, with zero new constants (at design `β₁` = 45° for
> every row, so `Δh_k = U²(1 − t₂,k)` derives the metal angle, the work split and `l_k`; and a
> row-invariant critical **diffusion** ratio anchored on rung 36's `φ_surge` derives `T_c,k`).
> What it does **not** supply is the **ANCHOR** — what is held fixed while the blades change —
> because a taper *sets the design blade speed*, `U²(t)/U²(0) = S(0)/S(t)`. Three anchors give
> three different verdicts, including opposite signs on the capacity channel. And the request
> itself is **over-determined**: holding the machine size *and* the front blade makes a monotone
> front-low-turning / rear-high-turning stack **impossible** (proof in § 4 there, `K`-independent).
> On the one well-posed anchor the capacity channel is **inert** (< 1.7 %) and the incidence
> channel is +0.6…4.6 % and purely positional — so this rung's headline is *consistent with, and
> not overturned by,* the obvious second lever. **What would revive it is an anchor supplied by
> physics** (a stress / tip-Mach limit pinning `U`, or an annulus law `Vx(k)`), not a better taper.

**PER-ROW BLADING.** Every claim above rides on all stages sharing one map — one `ψ`, one island,
one `φ_surge`, one `T_c`. That is what keeps rung 55's positional claim clean *and* what bounds
both rungs. A real stack's rows differ by design (front rows transonic and low-turning, rear rows
subsonic and high-turning), and the two constraints this rung separated are exactly the ones
per-row blading would move independently. It needs a `ψ_k`/`T_c,k` law, and — unlike the capacity
profile — the ladder does **not** supply it.

Then, unchanged: the **stator schedule `v(n)` on the TRANSIENT plant** (now with a row count as
well as a setting), **stator + bleed together**, and a **bleed schedule** `b(n_L)`.

## Anchor

`docs/plans/rung56-anchor-per-row-capacity.md` — the two probes that fixed the instrument and the
sign *before* the predictions, the six predictions as written, and their scoring:
**P1 HIT, P2 HIT, P3 band MISS / claim HIT, P4 HIT (with its second clause failing near the
crossover), P5 HIT, P6 REFUTED — and P6's refutation is the rung's fourth law.**
