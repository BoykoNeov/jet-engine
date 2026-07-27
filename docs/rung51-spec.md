# Rung 51 — The release RATE: the debit is not a functional of the applied-fuel trajectory

Rung 50 isolated the release edge with a forced release time `s_off` and found that the closing
edge relocates both spools' minima to itself, that the debit is monotone in the DEFICIT at fixed
release, and that rung 48's immunity is TIMING, not clip shape. It named its own next seam in
the Concessions, and named it precisely:

> *"The release is still an instantaneous hand-back. `s_off` moves **when** the fuel is returned,
> never **how fast**. A finite `τ_rel` would separate total deficit from deficit RATE — and
> **nothing measured here separates them**, which is exactly why `τ_rel` is named as the next
> seam rather than folded in."*

This rung builds that axis: the clip is **faded linearly to zero** over `[s_off, s_off+τ_rel]`
instead of being dropped at `s_off`.

**THE HEADLINE.** **The release debit is not a functional of the applied-fuel trajectory.** Take
a faded release and the two HARD releases at the two ends of its own fade interval. The faded
run's applied fuel is **pointwise sandwiched** between them (measured: 0 violations over the
whole march) and its total `fuel_removed` lies **between** theirs — yet its debit lies strictly
**OUTSIDE** both, shallower, on **both spools**:

| `r`=2.0, φ floor 0.7725 | `fuel_removed` | `relief_lp` | `relief_hp` |
|---|---|---|---|
| hard `s_off`=1.10 | 0.003801 | −0.05740 | −0.06900 |
| **faded 1.10, τ_rel=0.20** | **0.004534** | **−0.03121** | **−0.05476** |
| hard `s_off`=1.30 | 0.005249 | −0.06379 | −0.08088 |

No monotone functional of the fuel *level*, and no function of the total deficit, can produce a
value outside a bracket it sits inside. **The debit answers to the RATE of the hand-back** —
so rung 50 § 5's monotone-in-deficit law is **BOUNDED to the instantaneous hand-back**, not
refuted on its own axis.

**The scope, stated here and not buried in Concessions:** the bracket violation appears where
the dive is DEEP (`s_off` ∈ {1.10, 1.56} at `r`=2.0, both `τ_rel`) and does **NOT** appear at
`s_off`=0.30, where the faded point lands *inside* its bracket (−0.00478 / −0.01083 / −0.01975).
**Where it interpolates, rate and deficit are not separable and no claim is made.**

Two further results, one of them a correction to a shipped rung:

1. **Cross-family, the violation is large enough to FLIP THE SIGN.** On rung 48's `Wf/pt3` leg
   the sweep threw up a **naturally-occurring matched-deficit pair** — not solved for, which is
   what keeps it out of the matched-currency trap: `fuel_removed` matched to **0.02 %**,
   `relief_hp` **+0.005321 vs −0.007776**.
2. **Rung 50's precondition (a) is MIS-STATED** — the relocation crossover sits *upstream* of a
   spool's bare minimum, not at it. Rung 50's own § 1 table already violated the stated
   condition. **Its relocation headline is untouched; the boundary was wrong.**

---

## The instrument — a RATE on rung 50's forced release

```
    w(s) = clamp( (s_off + τ_rel − s) / τ_rel , 0, 1 )
    applied cap  =  mf_sched + w · (leg_cap − mf_sched)
```

`w ≡ 1` before `s_off`, fades linearly to 0 across the release interval, 0 after
(`_release_weight`). Everything up to `s_off` is **bit-identical** across a `τ_rel` sweep, so
the trigger, the engagement edge and the entire engaged window are held fixed while the
hand-back rate alone varies. `τ_rel` **requires** `s_off`.

**Why a stateless RAMP and not an asymmetric fast-attack / slow-release LAG.** All three reasons
were written down before any code:

1. **A lag's release edge is EMERGENT.** Sweep its time constant and the release time moves with
   it — reinstating exactly the confound `s_off` was built to kill, which is rung 49's
   within-family hedge one rung later. The ramp pins the trigger by construction.
2. **No state, and rung 50's RK4 argument carries verbatim.** `w` is a pure function of `s`,
   threaded into the sub-steps exactly as `fuel_schedule(s)` already is. A lag needs
   `max(g, required)` inside the derivative at a **state-dependent** location — materially worse
   than rung 50's fixed, grid-aligned step (rung 47 hit the latch version of this hazard and had
   to answer it with a third continuous state).
3. **It is finite-time.** An exponential never completes, so "the release edge" stops being a
   locatable object and the relocation law has nothing to be tested against. A ramp gives a
   release **interval** with a definite end — and which end governs turns out to be measurable.

**The lag is therefore this rung's own next seam**, named with its reason, exactly as rung 50
named `τ_rel`.

**No new constant.** `τ_rel` is a swept coordinate, like `s_off`. `φ_lim` (rung 36/41/49) and
`m` (rung 48) are inherited with their disclaimers intact.

**It is an isolation diagnostic, not a control law** — it inherits rung 50's disclaimer in full,
and arguably more strongly: it fades a clip that was already forced off at an arbitrary
wall-clock time. Every number below is a statement about *mechanism*.

---

## THE GATE — and why the OBVIOUS gate is confounded (recorded because it was the plan)

The pre-registered gate was rung 50 § 5's fixed-release **deficit → depth curve**: drop the
faded points onto it and see whether they lie on or off. **That instrument is confounded, and
the probe is what showed it:** at matched release-*completion* time a faded run always removes
LESS fuel than the hard one (its clip is fading, not full), and rung 50 § 5 already says less
deficit ⇒ shallower dive. "Shallower at matched completion" therefore proves nothing.

**The two-sided BRACKET replaces it.** For a fade over `[s_off, s_off+τ_rel]`, the two hard
releases at the two ends bracket it:

- **pointwise in applied fuel** — `hard@(s_off+τ_rel) ≤ faded ≤ hard@s_off` at every march
  point. Structural (the fading clip is strictly between "full clip" and "no clip"), but the
  caps are state-dependent, so it is **measured**: `s_off`=1.56, τ_rel=0.20 → **0 violations**.
- **in total `fuel_removed`** — measured, every row.

If the debit were any monotone functional of the fuel level, or any function of the total
deficit, the faded run would have to land BETWEEN its brackets. **It lands outside.**

---

## THE FINDINGS (config: rung 49/50's — CPG gas, accel 1000→1400 K, `ρ`=1, `ds`=0.02, `s_settle`=4; `tests/test_rung51.py` reproduces; transcripts in `docs/plans/rung51-anchor-release-rate.md`)

Bare references, **measured in this session** (not read off rung 50's constants): `r`=0.5 →
min φ_lp 0.735466 @ `s_lp*`=0.240, min φ_hp 0.861199 @ `s_hp*`=0.400; `r`=2.0 → 0.760938 @
0.320, 0.910688 @ 0.640.

### 1. THE HEADLINE — outside the bracket, both spools, four placements

| `r`=2.0, φ floor 0.7725 | `fuel_removed` | `relief_lp` | `relief_hp` |
|---|---|---|---|
| hard `s_off`=1.10 | 0.003801 | −0.05740 | −0.06900 |
| **faded 1.10, τ_rel=0.20** | **0.004534** | **−0.03121** | **−0.05476** |
| hard `s_off`=1.30 | 0.005249 | −0.06379 | −0.08088 |
| **faded 1.10, τ_rel=0.40** | **0.005129** | **−0.01889** | **−0.04224** |
| hard `s_off`=1.50 | 0.006855 | −0.06597 | −0.08904 |
| hard `s_off`=1.56 | 0.007356 | −0.06545 | −0.09049 |
| **faded 1.56, τ_rel=0.20** | **0.008214** | **−0.02638** | **−0.06157** |
| hard `s_off`=1.76 | 0.009029 | −0.05769 | −0.09042 |
| **faded 1.56, τ_rel=0.40** | **0.008779** | **−0.01353** | **−0.04855** |
| hard `s_off`=1.96 | 0.010508 | −0.03400 | −0.07800 |

The **1.56 / τ_rel=0.20** row is the cleanest instance: its two brackets **agree** (−0.09049 and
−0.09042 — postponing a *hard* release from 1.56 to 1.76 does essentially nothing), and the
faded run over exactly that interval is **1.47× shallower**. There is no timing story left to
tell; what differs is the rate.

### 2. THE SCOPE — where it interpolates, no claim is made

At `s_off`=0.30, `r`=2.0 the same construction gives hard@0.30 = −0.00478, **faded τ_rel=0.20 =
−0.01083**, hard@0.50 = −0.01975: the faded point is **INSIDE** its bracket. The bracket
violation is a **deep-dive** phenomenon, present at `s_off` ∈ {1.10, 1.56} and absent at 0.30.

Two consequences, and both are reported rather than smoothed:

- **P2 was FALSIFIED.** The prediction on file was "`|relief|` monotone falling in `τ_rel` at
  fixed `s_off`". It falls at 1.10 and 1.56 (−0.09049 → −0.04855 at 1.56) but **RISES** at 0.30
  (−0.00478 → −0.01083 → −0.01483). At `r`=0.5, `s_off`=0.30 the same fade even splits the
  spools in **opposite directions** — LP −0.00611 → +0.00409 (a credit), HP −0.00562 → −0.00808.
- **The reconciliation is POST-HOC and is labelled as such.** A fade does two things at once: it
  slows the hand-back (shallower) *and* it postpones the effective release (rung 50's ramp clock
  ⇒ deeper on the clock's rising limb, cheap at/past its peak). Which wins depends on where on
  that clock the release sits. This decomposition was **not** predicted; it was written after
  the measurement to explain it, and it is not gated.

### 3. CROSS-FAMILY — the violation flips the sign (rung 48's leg, m=0.15, `r`=2.0, `s_off`=1.10)

| τ_rel | None | 0.20 | 0.40 |
|---|---|---|---|
| `relief_hp` | −0.00887 | −0.00153 | **+0.00532** |
| `relief_lp` | 0.00000 | 0.00000 | 0.00000 |

Bracketed: hard@1.10 = −0.00887 (`fuel_removed` 0.000807), hard@1.50 = −0.00477 (0.001727);
faded τ_rel=0.40 = **+0.00532** at 0.001241 — removal between, relief outside **both**, and on
the other side of zero.

And the sweep threw up a **naturally-occurring matched-deficit pair** — found, not solved for,
which is what keeps it clear of the matched-currency trap rung 48 was blocked on twice:

| | `fuel_removed` | `relief_hp` |
|---|---|---|
| faded `s_off`=1.10, τ_rel=0.40 | 0.001240742 | **+0.005321** |
| hard `s_off`=1.30 | 0.001241011 | **−0.007776** |

**The same total fuel withheld, to 0.02 %, with opposite-signed relief.**

Note `relief_lp` ≡ **exactly 0.00000** in every row: rung 48's exact-zero law (`s_eng`=0.360 is
downstream of `s_lp*`=0.320) **survives the rate axis untouched**, as it survived rung 50's
forcing. Three rungs now, unmoved.

*What this is NOT.* It is tempting to read finding 3 as "a slow hand-back buys back rung 48's
immunity — so immunity has two routes, release late or release slowly." **That is an engineering
reading of an isolation diagnostic no engine has**, and it is not claimed. `τ_rel` fades a clip
that was already forced off at an arbitrary time. The claim is the bracket violation and its
cross-family reproduction; the sign flip is its evidence, not a design recommendation.

### 4. WHERE THE MINIMUM SITS — the completion point governs (P1, P4)

Every faded row relocates its minima to the **completion** point `s_off+τ_rel`, not to the
trigger — `s_off`=0.56/τ_rel=0.20 → min φ_hp @0.760; 0.60/0.10 → 0.700; 0.44/0.40 → 0.840;
0.30/0.40 → 0.700. At larger `τ_rel` the minimum **DETACHES into the interior**, the spin-up
recovery having overtaken the hand-back: 1.56/τ_rel=0.20 → LP @1.700 (interior), HP @1.740 (at
completion); 1.56/τ_rel=0.40 → LP @1.740, HP @1.820, both interior of [1.56, 1.96]. **P1 was
written with both halves and both scored right.**

### 5. RUNG 50'S PRECONDITION (a) IS MIS-STATED — the boundary, not the headline

Rung 50 stated the relocation law's first precondition as:

> *"(a) the release must land at or AFTER that spool's own bare minimum. Upstream of it the
> re-opened dive merges into the still-ongoing bare descent and bottoms in the bare basin
> instead."*

**Rung 50's own § 1 table already violated it.** In the `s_off`=0.30, `r`=2.0 row the LP release
(0.280) is upstream of `s_lp*`=0.320, yet `s@min φ_lp` = 0.300 — relocated to the release, and
un-italicised. Only the HP column was flagged.

Sampling the interval rung 50 skipped locates the real crossover (hard releases, `r`=2.0,
`s_hp*`=**0.640** measured this session):

| `s_off` | 0.30 | 0.32 | 0.34 | 0.36 | 0.40 | **0.44** | 0.50 | 0.56 |
|---|---|---|---|---|---|---|---|---|
| `s@min φ_hp` | 0.560 | 0.560 | 0.540 | 0.520 | 0.480 | **0.440** | 0.500 | 0.560 |

The minimum does not sit "in the bare basin" — it walks **monotonically toward the release from
above** and locks onto it at `s_off`≈0.44, i.e. at a release of 0.420, **0.66× `s_hp*` and well
upstream of it**. Rung 50 read a single sample (0.560 at `s_off`=0.30) as the bare basin and
generalised it into a necessary condition; it is **sufficient, not necessary**.

**Corrected statement:** relocation holds once the release is late enough that the re-opened
dive bottoms before the residual bare descent would have — a crossover lying **upstream** of
that spool's bare minimum, whose location is measured, not derived. **Rung 50's finding 1
(relocation) and its two-branch law § 6 are untouched.** With a fade, the same corrected
crossover applies to the **completion** point (finding 4).

### 6. NOT rung 44's ramp-rate lever — the non-tautology gate

- **The endpoint is unmoved.** `nu_hp_end` ∈ [0.959059, 0.959064] across the entire sweep
  against bare 0.959064 — **≤ 5e-6**, two orders under rung 49/50's 5e-4 threshold.
- **Fuel removal RISES while the debit FALLS.** At `s_off`=1.56, `τ_rel` 0 → 0.80 removes
  **+28 %** more fuel (0.007356 → 0.009391) for **less than half** the debit (−0.09049 →
  −0.04430). A ramp-rate lever cannot do that.
- **One fade, two signs.** § 2's `r`=0.5 row credits LP (+0.00409) and debits HP (−0.00808) from
  the same leg on the same plant.

### 7. Robustness

`ds` 0.02 → 0.01 at (1.56, τ_rel=0.20): `relief_lp` −0.02638 → −0.02640, `relief_hp` −0.06157 →
−0.06162 — **0.08 %**, and the min locations move by at most one cell.
`ρ` ∈ {0.25, 4}: the sign and the shallower-than-both-brackets ordering both survive
(ρ=0.25: −0.09462 → −0.06082; ρ=4: −0.08700 → −0.05971).

---

## Reduce-to-prior contract (the spine)

- `tau_rel=None` **or** `0.0` ⇒ `_release_weight` returns exactly 1.0 / 0.0 through the
  **identical branch** ⇒ `integrate_fuel` is **bit-for-bit** rungs 43/45/46/47/48/49/50.
- `tau_rel` **requires** `s_off` (**asserts** otherwise): a rate without a pinned trigger is the
  asymmetric lag, whose release edge moves with the rate — a different instrument.
- `s_off` past the leg's natural release ⇒ `tau_rel` is **inert**, float-for-float the unforced
  leg (there is no clip left at `s_off` to fade).
- `lp_disabled` **asserts** — inherited from rung 50: the finding is a split between spools.
- The rung-46/47 topping governor stays out of scope (rung 50's exclusion, unchanged).
- The design run `build_turbojet(…).run(…)` is untouched — **bit-for-bit rung 6**.

## Verification gates (`tests/test_rung51.py`)

1. `tau_rel=None` and `tau_rel=0.0` ⇒ bit-for-bit rungs 49/50 on
   `(nu_lp, nu_hp, phi_lp, phi_hp, Tt4, f, mf)`; `rate_sweep`'s `None` row equals
   `release_relief` exactly.
2. `tau_rel` without `s_off` **asserts**; `lp_disabled` **asserts**; `s_off` past the natural
   release ⇒ every `tau_rel` float-identical.
3. **THE HEADLINE** — at `s_off` ∈ {1.10, 1.56}, `τ_rel` ∈ {0.20, 0.40}: `fuel_removed` strictly
   BETWEEN the two hard brackets while `relief_lp` **and** `relief_hp` are strictly ABOVE
   (shallower than) **both**.
4. **The pointwise sandwich** — `hard@(s_off+τ_rel) ≤ faded ≤ hard@s_off` in applied fuel at
   every march point: **0 violations**.
5. **THE SCOPE (a NEGATIVE gate)** — at `s_off`=0.30 the faded point lies strictly INSIDE its
   bracket. Asserted, so the claim cannot silently widen.
6. **Cross-family** — rung 48's leg at `s_off`=1.10 goes `relief_hp` < 0 at `τ_rel`=None to
   **> 0** at `τ_rel`=0.40, outside both brackets; and `relief_lp` is **exactly 0.0** in every row.
7. **The matched-deficit pair** — `fuel_removed` agreeing within 0.1 % with opposite-signed
   `relief_hp`.
8. **Location** — every faded row's minima sit at the completion point or interior to the fade
   interval, never upstream of it; and the large-`τ_rel` detachment is asserted.
9. **The precondition correction** — the hard-release scan is monotone-approaching and locks
   onto `s_off` at 0.44, upstream of the measured `s_hp*`=0.640.
10. **Non-tautology** — `fuel_removed` monotone UP in `τ_rel` while `|relief|` falls, and
    `nu_hp_end` within 5e-4 of bare everywhere.
11. **Robustness** — `ds` ∈ {0.02, 0.01} within 1 %; `ρ` ∈ {0.25, 4} preserve sign + ordering.
12. The design run is bit-for-bit rung 6.

## Concessions

- **`τ_rel` is not a control law**, and neither is the `s_off` it rides on. Both are isolation
  diagnostics in the project's tradition (rung 34/40's `freeze=`, rung 41's
  `surge_margin_channels`). What they license is the reading of the *mechanism*.
- **The claim is bounded to the deep-dive regime** (§ 2). Where the faded point interpolates,
  rate and deficit are **not separable** and nothing is claimed.
- **The postponement-vs-rate decomposition is post-hoc**, not predicted and not gated (§ 2).
- **The crossover in § 5 is measured, not derived.** The correction to rung 50 is that its
  stated boundary is wrong, not that the right one is now known in closed form.
- **`φ_lim` and `m` inherit rungs 36/41/48/49's imposed constants.** Magnitudes disclaimed;
  signs, ordering, bracket violations and `ds`-convergence are the claims.
- **The fade is LINEAR.** No shape sweep was run, so nothing is claimed about the hand-back's
  functional form — only that a finite-duration one behaves differently from a step.
- **The asymmetric fast-attack / slow-release LAG is the next seam**, deferred for the three
  reasons in § The instrument. It is the physically-realisable version and its release edge is
  *emergent*, which is exactly why it cannot be the instrument that establishes the effect.
- The plant is rung 43's non-equilibrium (CPG/TPG) gas — rung 35's standing concession.

## Anchor

`docs/plans/rung51-anchor-release-rate.md` — the predictions as written before measuring (P2
scored FALSIFIED, P3's framing superseded), the probe transcripts, and the verified numbers.
