# PER-ROW BLADING — investigated, NEGATIVE (not shipped, not a rung)

Rung 56's named next seam:

> *"**PER-ROW BLADING.** Every claim above rides on all stages sharing one map — one `ψ`, one
> island, one `φ_surge`, one `T_c`. … A real stack's rows differ by design (front rows
> transonic and low-turning, rear rows subsonic and high-turning), and the two constraints
> this rung separated are exactly the ones per-row blading would move independently. It needs
> a `ψ_k`/`T_c,k` law, and — unlike the capacity profile — the ladder does **not** supply it."*

**Attacked, and it fails — but not where rung 56 expected.** The ladder supplies *more* of the
law than rung 56 thought (§1–2: the metal angle, the work split, and the critical incidence all
come out with zero new constants). What it does not supply is **the anchor** — what is held
fixed while the blades change — and every sign and magnitude in the result answers to that
choice, not to the blading.

**VERDICT: NEGATIVE, by a decision rule fixed before the measurement was read.** Under the
only well-posed anchor the capacity channel is **inert (< 1.7 %)**, and the surviving incidence
effect is a modest positional one that does not reach the separation rung 56 asked about.

Two results survive independently and are recorded here because they live nowhere else:
**§1's correction of rung 55's work split**, and **§3's blade-speed identity**.

---

## 1. THE LADDER SUPPLIES MORE THAN RUNG 56 THOUGHT — and it corrects rung 55

Rung 53 inverted the map's design slope into one rotor exit angle, `t₂ = tan β₂ = l/(1+l)`.
Resolved into rows, on rung 56's constant mean radius (`U_k` = `U`) and rung 55's design sizing
(`φ_k` = 1), Euler work at design is

```
    Δh_k = U²·(1 − t₂,k)                                                          ...(1)
    l_k  = (1+l)·Δh₀/Δh_k − 1                                    ZERO new constants
```

At design `β₁ = arctan(1/φ − v) = 45°` for **every** row, so turning = `β₁ − β₂` is set by `β₂`
alone: **small `t₂` = small `l` = HIGH turning = HIGH work.** (Stated because the opposite
mapping is the natural guess and it is wrong.)

Run backwards on the **shipped** design ladder (`τ_lpc,d` = 1.409709, `τ_hpc,d` = 1.759671,
read from the code), rung 55's two disclosed work splits are two **hardware** choices:

| rung-55 split | implied blade taper | min `w₂/w₁` (de Haller ≥ 0.72) |
|---|---|---|
| `dT` (the default) | 0.0000 LP / 0.0000 HP — **uniform blades** | 0.7647 / 0.7906 — OK |
| `tau` (the alternative) | 0.5007 LP / **0.6397 HP** | 0.7219 / **0.7185 — VIOLATES** |

**CORRECTION OF RUNG 55 (the rung-28 shape).** Rung 55 disclosed the work split as free
robustness furniture and rung 56 P4 measured it as load-bearing without asking what it *was*.
It is a blading law: **the split axis IS the blading axis.** The default is exactly uniform
blades — which is why route (a) has literally zero content there — and the alternative sits
just outside the published diffusion limit on the HP. Both rungs' *verdicts* survive (each was
asserted across both splits); the *framing* does not.

Consequence for the seam: a per-row `ψ_k` law derived from the work split is **degenerate on
`dT` and non-physical on `tau`** — dead as a primitive. It survives only inverted (§2).

## 2. THE INVERSION — blading as the primitive, and it needs no new constant either

Take the metal as primitive and let (1) *derive* the work split. One taper `t`:

```
    t₂,k = t₂,0·(1 − t·k/(K−1)),   t₂,0 = l/(1+l)          [the map's OWN slope]
    θ_k,d = 1 + (τ_d−1)·S_k/S_K,   S_k = Σ_{j<k}(1 − t₂,j)      U² CANCELS
```

Four channels off one primitive: the work ladder (now an output), the per-row loading slope
`l_k` (which **enters `solve_n`**, so this has a feedback leg unlike rung 56), rung 56's
capacity profile (which rides on the now-derived `θ_k,d`), and the **per-row critical
incidence** — the piece rung 56 said the ladder could not supply:

```
    R = cos β₁,crit,0 / cos β₂,0 held ROW-INVARIANT (a critical DIFFUSION, level inherited
    from rungs 36/41's imposed φ_surge via row 0)   =>   T_c,k = tan(arccos(R·cos β₂,k))
```

Zero new constants; rear rows get **smaller** `T_c,k` (a tighter incidence limit). And `t = 0`
reproduces rung 55's `dT` ladder to 2.2e-16 — **rung 55's default is the uniform-blading limit
of this construction**, so the reduce would have been an identity, not a new flag.

Even the taper's *range* is anchored rather than disclosed: de Haller's published
`w₂/w₁ ≥ 0.72` at `β₁` = 45° gives `t_max` = 0.534 (`l`=0.7) / 0.616 (`l`=1.0), with the
uniform baseline already clear of it at 0.765 / 0.791.

**So the instrument is better than rung 56 predicted. It still fails — for a different reason.**

## 3. WHAT KILLS IT — the taper sets the DESIGN BLADE SPEED, and the anchor is a free knob

`U²` cancels from the *normalised* temperature ladder (so design capture holds — `τ_d`, `π_d`
and the whole cycle are untouched), but it does **not** cancel from the machine. Summing (1):

```
    U²(t)/U²(0) = S(0)/S(t),      S(t) = Σ_k (1 − t₂,k(t))                        ...(2)
```

A taper anchored on row 0 swings the design blade speed by **+29 % / −11 %** over the range
probed. That is not the same engine — and because `φ_d` = 1 makes `Vx` = `U`, it moves every
row's design Mach, hence rung 56's `C`. **Sweeping the taper at fixed `C` = 0.90 silently
re-picks the throat each time.** Three anchors, and the results are not variations — they are
different verdicts. **The first three rows are ENDPOINT-referenced** (to the sweep's `t` = −0.8)
and are shown to establish that *the verdicts disagree*, which survives; their magnitudes are
not claims — only the last row's are, and it is referenced to uniform blading (§4):

| anchor | what is held | LP-front incidence | HP-rear capacity |
|---|---|---|---|
| **row 0** (`C` fixed) | front blade; `U` floats ±29 % | **−15 %** (debit) | **−17 %** (debit) |
| **row 0** (`C` tracks `U`) | front blade; annulus | −15 % (debit) | **+463 %** (relief) |
| **mean** (`U` pinned) | machine size | **+104 %** (relief) | ±1 % |
| **row 0 + mean** (§4) | machine size AND front blade | **+0.6…4.6 %** (relief) | **< 1.7 %** |

Every sign, every magnitude, and even rung 56's binding-row **migration** flip with the anchor.
The `+463 %` is a near-zero-denominator artifact against a design point (`C` → 0.99) that
`ComponentMap.with_capacity`'s own `C < 1` assertion would refuse; the `+104 %` is the moved
row's own limit moving — near-tautological. Both are why the endpoint-referenced rows are
labelled above rather than quoted below.

**This is the seam's real cost.** Rung 56 said the ladder does not supply a `ψ_k`/`T_c,k` law.
It supplies both. What it does not supply is **what is held fixed**, and that is a *second*
unanchored knob sitting underneath the first — the same disease that killed
`docs/mixing-scale-negative.md` (an unanchored penetration exponent) and
`docs/mixing-jicf-anchor-negative.md` (an unanchored spread exponent).

## 4. THE SEAM'S REQUEST IS OVER-DETERMINED — a proof, and it is the real finding

Both defects of §3 are removable at once, with the same one-parameter family applied to rows
1…K−1 only — so `U` is pinned (machine unchanged, `C` = 0.90 genuinely fixed) **and** the front
row's metal is pinned (any front-row effect is then purely positional, through the solved `n`):

```
    t₂,0 = t₂,ref ;   t₂,k = b·(1 − s·(k−1)/(K−2)), k ≥ 1,   b = t₂,ref/(1 − s/2)
```

**And that family cannot be monotone.** Row 0 pinned at `t₂,ref`, plus the machine pinned
(`Σ_k (1 − t₂,k)` fixed ⇒ `mean_k t₂,k` = `t₂,ref`), forces `mean_{k≥1} t₂,k` = `t₂,ref` too;
any non-constant decreasing sequence with that mean must start **above** it, so
`t₂,1 > t₂,0` — row 1 turns *less* than row 0. Measured at `s` = +0.60, `t₂,k/t₂,ref` =

```
    1.000  1.429  1.286  1.143  1.000  0.857  0.714  0.571        NON-monotone
```

**So: you cannot simultaneously hold the machine size, hold the front blade, and have a
monotone front-low-turning / rear-high-turning stack.** The only monotone member of the family
is the constant one. This is `K`-independent and is what explains §3 — every anchor gave a
different verdict because the seam's specification is over-determined, and each anchor drops a
different one of its three requirements.

**The verdict, on the well-posed (necessarily non-monotone) family.** Decision rule fixed
before the output was read: capacity moves > 2 % somewhere in range ⇒ the rung lives; inert ⇒
negative. Over `s` ∈ [−0.6, +0.6], `K` = 8, three shapes, `Tt4` ∈ {1000, 800}, **referenced to
`s` = 0** (uniform blading — the only point in the family where nothing has moved; referencing
to an endpoint is rung 43's currency-circularity trap, and at `s` ≤ −0.4 the binding incidence
row is `lp1`, a row this family *does* move):

```
    HP-rear capacity margin :  max |Δ| = 1.61 %   -- and NON-MONOTONE (falls, then turns back)
    LP-front incidence      :  +0.56 % .. +4.62 % over s > 0, where the binding row (lp0) is
                               the UNMOVED one, so the effect is purely positional
```

**NEGATIVE — and by a wider margin than the first reading suggested.** With the machine and the
moved row both held, per-row blading **cannot reach the capacity constraint at all**, and what
it does reach — the incidence row — it reaches only through the channel rung 55 already
published (the shared shaft speed), at single-digit percent, a small fraction of rung 53's
stator authority. It does **not** move rung 56's two constraints independently; it barely moves
either.

*Rung 56's headline is therefore **consistent with, and not overturned by, the obvious second
lever**.* Stated no more strongly than that: only one anchor family was tested, and its only
well-posed member has, by §4's proof, a shape the seam did not ask for.

## 5. What would make it a rung

Not a better taper — an **anchor supplied by physics rather than by choice**, which is also the
only way out of §4's over-determination. Concretely: a row-level stress or tip-Mach limit that
pins `U` from outside the stack, or an annulus law `Vx(k)` (which rung 56 already flagged as
forced constant here) that fixes the geometry independently of the blading. Either turns (2)
from a free knob into a constraint, and frees the third requirement so a monotone taper becomes
admissible. Both are new laws with new constants, and neither is in the ladder.

Until then the standing concession is unchanged and should be read as **bounded, not open**:
rungs 55/56's stages share one map, and resolving the blades does not buy back what rung 56
showed the stator cannot reach.

## Reproduction, and the pre-registration scored

**Nothing in the shipped code was changed.** Every number above came from local subclasses of
`StageStack`/`StageStackMatcher` overriding four methods (`_blade_angles` — new; `_ladder_T`;
`psi_at`; a per-row `tan_beta1_crit_at`), built in `M:\claud_projects\temp\rung57-blading\`.
That folder is outside git (project temp policy), so this tracked file is the durable record.
The probe ladder: **A** the work split read as a `ψ_k` law (degenerate on `dT`, non-physical on
`tau`) · **B** the inversion + the de Haller ceiling · **C** rung 55's two splits scored as
blading (§1) · **D** the front anchor · **E** the `U`/`C` coupling (§3) · **F** the mean
anchor · **G** the well-posed anchor and the verdict (§4).

Predictions were written before any in-repo measurement, and are scored as written:

- **P1** taper tightens the capacity minimum, monotonically — sign HIT under the front anchor,
  **void** under the well-posed one, where the channel is inert.
- **P2** the incidence-binding row migrates rearward — **REFUTED**; it HITS only under the
  anchor that moves the very row it measures.
- **P3** the two minima move independently (*the discriminating one*) — **REFUTED**; the ratio
  is near taper-invariant.
- **P4** the front row is reached only through the solved `n` — HIT, and it is what makes §4's
  small number trustworthy rather than merely small.
- **P5** `work_gap` moves (a feedback leg, unlike rung 56) — HIT.
- **P6** `K`-convergence — not reached; the verdict landed first.

**The two predictions that would have made this a rung are exactly the two that failed.**
