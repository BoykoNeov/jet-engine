# Rung 51 anchor — the release RATE (`τ_rel`): predictions written BEFORE any measurement

Rung 50 shipped the forced release time `s_off` and named its own next seam in the
Concessions:

> *"The release is still an instantaneous hand-back. `s_off` moves **when** the fuel is
> returned, never **how fast**. A finite `τ_rel` would separate total deficit from deficit
> RATE — and **nothing measured here separates them**, which is exactly why `τ_rel` is named
> as the next seam rather than folded in."*

This rung builds that axis. **The gate, stated before the instrument:**

> **Does a rate-limited hand-back sit ON or OFF rung 50 § 5's fixed-release
> deficit → depth curve?**

- **ON** ⇒ the release debit is a pure *deficit/timing* object; rung 50's relocation law
  survives the rate axis and `τ_rel` is a **CONFIRMATION** rung. *This verdict is publishable
  and will be shipped as one* — it says "rate is timing in disguise", which is a real
  statement about the mechanism, and it is written here so that it cannot be quietly
  discarded in favour of hunting for a regime where the rate does move.
- **OFF, shallower** ⇒ the rate governs; rung 50 § 5's monotone-in-deficit law is **BOUNDED**
  to the instantaneous hand-back, and the deficit was a *proxy* for the step SIZE.
- **OFF, deeper** ⇒ rung 47's cost-of-realism shape, on a new edge.

## The instrument (chosen before measuring, for stated reasons)

**A stateless forced-release RAMP.** The min-select leg's clip AMOUNT is faded linearly to
zero over `[s_off, s_off + τ_rel]` instead of being dropped at `s_off`:

```
    w(s) = clamp( (s_off + τ_rel − s) / τ_rel , 0, 1 )          (a pure function of s)
    applied cap  =  mf_sched + w · (leg_cap − mf_sched)
```

`w ≡ 1` before `s_off`, `w ≡ 0` after `s_off + τ_rel`. `τ_rel = 0` (or `None`) short-circuits
to rung 50's step, reaching the **identical branch** so the reduce is bit-for-bit, not
equal-to-tolerance.

**Why this and not an asymmetric (fast-attack / slow-release) first-order lag:**

1. **It holds the trigger fixed by construction.** A lag's release edge is *emergent*: change
   `τ_rel` and the release moves, which reintroduces exactly the confound `s_off` was built to
   kill (rung 49's within-family hedge, one rung later). Here `s_off` is pinned and the
   trajectory up to it is bit-identical; only the hand-back rate varies.
2. **No state, and rung 50's RK4 argument carries verbatim** — `w` is a pure function of `s`,
   which threads into the sub-steps exactly as `fuel_schedule(s)` already does. A lag needs
   `applied_clip = max(g, required)`, a non-smooth kink inside the derivative at a
   **state-dependent** location — materially worse than rung 50's fixed, grid-aligned step.
3. **It is finite-time.** An exponential never completes, so "the release edge" stops being a
   locatable object and the relocation headline becomes unmeasurable. A ramp gives a release
   **interval** with a definite end — the thing the relocation law must now be tested against.

**The lag is therefore the NEXT seam, not this rung** — named, with the reason.

## Config (rung 50's, unchanged)

CPG gas, `FLIGHT(250 K, 50 kPa, M0=0.85)`, `π_LPC/π_HPC/Tt4 = 3/6/1500`, shaped LP/HP maps,
accel 1000 → 1400 K, `ρ`=1, `ds`=0.02, `s_settle`=4. LP-watching floors `φ_lim`=0.7450
(`r`=0.5) and 0.7725 (`r`=2.0). Bare minima: `r`=0.5 → `s_lp*`=0.240, `s_hp*`=0.400;
`r`=2.0 → 0.320, 0.640. **Both** `s_off` and `s_off + τ_rel` on the `ds` grid.

---

## THE PREDICTIONS (committed before running anything)

### P1 — WHERE the minimum sits

Rung 50's hard release puts `argmin φ` **at** the release, to one grid cell. The mechanism is
that `φ` is an **algebraic** output of `(ν_L, ν_H, mf)` through `_instant_fuel`, so a *step*
in fuel produces a *step* down in `φ`, and the spool spin-up then carries `φ` back up.

With the clip faded, the fuel returns gradually while `ν` keeps rising, so the minimum sits
wherever the hand-back rate stops beating the spin-up recovery.

> **PREDICT: for small `τ_rel` (a few `ds`) the minimum sits at the FAR end,
> `s_off + τ_rel`, to within a grid cell. For large `τ_rel` it DETACHES and sits strictly
> INTERIOR to `(s_off, s_off + τ_rel)`.**

*Falsifiers:* the minimum stays pinned at `s_off` for all `τ_rel` (⇒ the near end governs and
the fade is inert on location); or it never detaches from the far end however large `τ_rel`
gets (⇒ the far end governs, full stop, and the fade is a pure relabeling of `s_off`).

### P2 — HOW DEEP, at fixed `s_off`

> **PREDICT: `|relief|` falls MONOTONICALLY with `τ_rel` at fixed `s_off`, on both spools,
> and tends toward 0 (or crosses into a credit) for `τ_rel` large enough.**

Note the built-in confound and why it is not the gate: `fuel_removed` *rises* with `τ_rel`
(the clip is held partially on for longer), so a naive sweep moves the deficit and the rate
**together** — the same trap rung 49 § 4 fell into and rung 50 § 5 corrected. The sweep alone
therefore cannot decide the rung; § 5's curve can.

*Falsifier:* `|relief|` rises with `τ_rel` (the extra withheld fuel dominating the gentler
hand-back) ⇒ the OFF-deeper verdict.

### P3 — THE GATE: on or off rung 50 § 5's curve

Rung 50 § 5 measured, at **fixed release time**, `|relief_hp|` monotone increasing in
`fuel_removed`, within the φ-floor family (`r`=2.0, `s_rel`=0.740: 0.000658 → 0.001822 gave
−0.01330 → −0.04079) and across families (`r`=0.5, `s_rel`=0.420).

Reconstruct that curve here by sweeping `φ_lim` at a **hard** release (rung 50's instrument),
then drop the faded points onto the same `(fuel_removed, |relief|)` axes.

> **PREDICT: OFF the curve, and SHALLOWER — a faded release is markedly less damaging than a
> hard release that removed the same total fuel. ⇒ the RATE governs, and rung 50 § 5's
> deficit law is BOUNDED to the instantaneous hand-back (the deficit was a proxy for the
> STEP SIZE, which is `deficit / τ_rel` in the limit).**

Because the curve's abscissa is ambiguous for a faded run (which release time is it to be
compared at?), the comparison is made against curves built at **both** ends, `s_rel = s_off`
and `s_rel = s_off + τ_rel`. That ambiguity is not a nuisance — it is the measurement:

*Falsifier / confirmation branch:* if the faded points lie **ON** the curve built at
`s_rel = s_off + τ_rel`, then the fade is a **relabeling of the release time** — rate is
timing in disguise, rung 50 is CONFIRMED on a new instrument, and that is the shipped result.

### P4 — WHICH END governs rung 50's precondition (a)

Rung 50's gate 3 precondition is stated for a *point* release: "the release lands at or after
that spool's own bare minimum". With an interval, the spec text must say which end.

> **PREDICT: the FAR end governs — relocation requires `s_off + τ_rel ≥ s*`.** Rationale: the
> fuel is still being handed back until the far end, so a dive that starts upstream of `s*`
> can still bottom past it.

*Falsifier:* a case with `s_off < s* ≤ s_off + τ_rel` whose minimum stays in the bare basin
(⇒ the NEAR end governs), or one with `s_off + τ_rel < s*` that relocates anyway.

### P5 — the non-tautology / not-rung-44's-lever exclusions (carried from rung 50 § 7)

> **PREDICT: `nu_hp_end` unmoved at `s_settle`=4 (< 5e-4 against bare) across the whole `τ_rel`
> sweep**, so the fade is not a disguised ramp-rate lever; and `fuel_removed` monotone
> INCREASING in `τ_rel` while `|relief|` is monotone DECREASING — the largest removal giving
> the *smallest* debit, which a ramp-rate lever cannot do.

---

## What was actually measured

*(filled in after the probe — nothing below this line was written before running)*

Bare references, **measured in this session** (not read off rung 50's constants):
`r`=0.5 → `s_lp*`=0.240 (φ 0.735466), `s_hp*`=0.400 (0.861199);
`r`=2.0 → `s_lp*`=0.320 (0.760938), `s_hp*`=0.640 (0.910688). Identical to rung 50's.

### The gate FRAMING was superseded — say so plainly

P3 proposed rung 50 § 5's fixed-release deficit→depth curve as the gate. **It is the wrong
instrument, and the probe showed why:** at matched release-*completion* time a faded run always
removes LESS fuel than the hard one (its clip is fading, not full), so "shallower at matched
completion" is confounded with "removed less" — and rung 50 § 5 already says less deficit ⇒
shallower dive. The curve cannot separate them.

**What replaced it: a TWO-SIDED BRACKET.** For a fade over [`s_off`, `s_off`+`τ_rel`], the two
hard releases at the two ENDS bracket it — *pointwise in applied fuel* (measured, 0 violations)
and in total `fuel_removed`. If the debit were any monotone functional of the fuel level or of
the total deficit, the faded run would have to land BETWEEN them. It does not.

### The bracket, `r`=2.0, φ floor 0.7725 (the load-bearing table)

| placement | `fuel_removed` | `relief_lp` | `relief_hp` |
|---|---|---|---|
| hard `s_off`=1.10 | 0.003801 | −0.05740 | −0.06900 |
| **faded 1.10, τ=0.20** | **0.004534** | **−0.03121** | **−0.05476** |
| hard `s_off`=1.30 | 0.005249 | −0.06379 | −0.08088 |
| hard `s_off`=1.10 | 0.003801 | −0.05740 | −0.06900 |
| **faded 1.10, τ=0.40** | **0.005129** | **−0.01889** | **−0.04224** |
| hard `s_off`=1.50 | 0.006855 | −0.06597 | −0.08904 |
| hard `s_off`=1.56 | 0.007356 | −0.06545 | −0.09049 |
| **faded 1.56, τ=0.20** | **0.008214** | **−0.02638** | **−0.06157** |
| hard `s_off`=1.76 | 0.009029 | −0.05769 | −0.09042 |
| hard `s_off`=1.56 | 0.007356 | −0.06545 | −0.09049 |
| **faded 1.56, τ=0.40** | **0.008779** | **−0.01353** | **−0.04855** |
| hard `s_off`=1.96 | 0.010508 | −0.03400 | −0.07800 |

Every faded row: `fuel_removed` strictly between its two brackets, `relief` strictly OUTSIDE
(shallower than) both, on **both** spools. The 1.56/τ=0.20 pair is the cleanest — its two
brackets **agree** (−0.09049, −0.09042) and the faded value is 1.47× shallower.

**Pointwise sandwich, measured** (`s_off`=1.56, τ=0.20 vs hard@1.56 / hard@1.76): applied-fuel
violations of `hard@1.76 ≤ faded ≤ hard@1.56` = **0** over the whole march.

### The NATURALLY-OCCURRING matched-deficit pair (rung 48's leg, `r`=2.0, m=0.15)

Found in the sweep, not solved for — which is what keeps it out of the matched-currency trap:

| | `fuel_removed` | `relief_hp` |
|---|---|---|
| faded `s_off`=1.10, τ=0.40 | 0.001240742 | **+0.005321** |
| hard `s_off`=1.30 | 0.001241011 | **−0.007776** |

**Matched to 0.02 % in total fuel removed; OPPOSITE SIGN of relief** (a 0.0131 swing in φ).

### The rate sweep at fixed trigger (`r`=2.0, φ 0.7725)

`s_off`=1.56: τ = None/0.04/0.10/0.20/0.40/0.60/0.80 →
`relief_hp` −0.09049 / −0.08564 / −0.07654 / −0.06157 / −0.04855 / −0.04476 / −0.04430;
`relief_lp` −0.06545 / −0.05766 / −0.04349 / −0.02638 / −0.01353 / −0.00772 / −0.00430;
`fuel_removed` 0.007356 → 0.009391 (monotone UP).
`s_off`=1.10: `relief_hp` −0.06900 → −0.04224, `relief_lp` −0.05740 → −0.01889 (τ ≤ 0.40).
**`s_off`=0.30: `relief_hp` −0.00478 → −0.01083 → −0.01483 — DEEPENS.**

### Cross-family (rung 48 accel m=0.15, `r`=2.0, `s_off`=1.10)

τ = None / 0.20 / 0.40 → `relief_hp` −0.00887 / −0.00153 / **+0.00532** (unforced: +0.0056).
`relief_lp` ≡ **0.00000** in every row — rung 48's exact-zero law survives the rate axis.

### Rung 50's precondition (a) — MIS-STATED (the boundary, not the headline)

Hard releases, `r`=2.0, `s_hp*`=**0.640** (measured above): `s_off` 0.30 → min φ_hp @0.560;
0.32 → 0.560; 0.34 → 0.540; 0.36 → 0.520; 0.40 → 0.480; 0.44 → **0.440**; 0.50 → 0.500;
0.56 → 0.560. The minimum approaches the release **monotonically from above** and locks onto it
at `s_off`≈0.44 (release 0.420) — **0.66× `s_hp*`, well upstream of it.**

Rung 50 § 1's own table already violated its stated condition and it went unnoticed: at
`s_off`=0.30 the LP release (0.280) is upstream of `s_lp*`=0.320, yet `s@min φ_lp` = 0.300 —
relocated, and un-italicised.

### Robustness

`ds` 0.02 → 0.01 at (1.56, τ=0.20): `relief_lp` −0.02638 → −0.02640, `relief_hp` −0.06157 →
−0.06162 (**0.08 %**); min locations 1.700/1.740 → 1.700/1.750 (one cell).
`ρ` ∈ {0.25, 4}: signs and the shallower-than-both-brackets ordering both survive
(ρ=0.25: −0.09462 → −0.06082; ρ=4: −0.08700 → −0.05971).
`nu_hp_end` across every row: 0.959059–0.959064 against bare 0.959064 (**≤ 5e-6**, threshold 5e-4).
`s_off` past the natural release ⇒ `τ_rel` **inert** (all τ float-identical, `r`=0.5, `s_off`=0.44).

---

## PREDICTION SCORING

| | verdict |
|---|---|
| **P1** — min at the far end for small τ, DETACHES interior for large τ | **CONFIRMED**, both halves. (1.56: τ=0.04/0.10 → min at completion; τ=0.20 → LP 1.700 interior, HP 1.740 at completion; τ=0.40 → both interior.) |
| **P2** — `|relief|` monotone FALLING in τ at fixed `s_off` | **FALSIFIED.** It falls at `s_off`=1.10/1.56 but **RISES** at `s_off`=0.30 (−0.00478 → −0.01483), and at `r`=0.5 the same fade splits the spools in opposite directions (LP −0.00611 → +0.00409, HP −0.00562 → −0.00808). The postponement-vs-rate decomposition that reconciles them is **post-hoc**, and is labelled as such in the spec. |
| **P3** — OFF rung 50 § 5's curve, shallower | **SUPERSEDED.** The framing was wrong (see above); the two-sided bracket replaced it and answers the same question without the confound. Direction of the answer — the debit is not a deficit functional — was right. |
| **P4** — the FAR end governs the precondition | **CONFIRMED** (every faded row relocates to its completion point, not its trigger), but **entangled with a correction**: the precondition it was asked about is itself mis-stated. |
| **P5** — `nu_hp_end` unmoved; removal UP while debit DOWN | **CONFIRMED** (≤5e-6; and at `s_off`=1.56 removal +28 % while `|relief_hp|` halves). |
