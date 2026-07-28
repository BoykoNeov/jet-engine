# Rung 59 — the MATCHED SCHEDULE: the ORDINATE cannot see the stator

Rung 58 put rung 53's floor-moving stator beside a fuel-side min-select leg and found the two
do not superpose. It held ONE leg object across its four cells, and refused the matched
variant — the leg re-derived on the machine it actually runs on, which is what a FADEC burns
in — as a confounded experiment, on a stated premise:

> *"`accel_schedule` reads `self.equilibrium`, so a stator-armed machine derives a different
> `κ_ss` table; letting each cell derive its own would make the leg itself differ between
> cells and the second difference would isolate nothing."*

**That premise is false**, and this rung is mostly the proof of how false, and where it is not.

---

## THE HEADLINE

**A derived schedule's ORDINATE cannot see a stator; only its INDEX can. So matching a
schedule to a moved machine is PURE RE-INDEXING — a no-op exactly when the lever leaves the
schedule's own abscissa alone, and worth 100 % of the effect when it does not.**

Two halves, and the split between them is the rung.

### The ordinate — a function of `Tt4` ALONE, on EITHER spool, exactly

    κ_ss  =  f · ṁ/pt3  =  π_b · f(Tt3, Tt4) · MFP_A4 / [(1+f)·√Tt4]

- **(i) `A4` is CHOKED** (rungs 30/31), so the corrected group `ṁ(1+f)√Tt4/pt4` is *hardware*
  — `γ`, `R`, the throat area — and nothing the stators do can reach it. Measured:
  `MFP = 2.962907072632e−05`, **the same number at `Tt4` = 1000, 1200 and 1400 and on every
  stator setting tried**.
- **(ii) `Tt3` is pinned by the TWO SHAFT BALANCES**, which are map-free with every throat
  choked (rung 31's `(★)`, chained twice by rung 38). A stator changes the *speed* at which a
  temperature ratio is bought — `nu_lp` moves **+9.6 %** — not the ratio. Measured: `dTt25`,
  `dTt3`, `df` all `≤ 1e−13` across an LP constant, an LP schedule and an HP constant setting.
- **(iii) `f` is pressure-independent on CPG.** `_solve_f(Tt3, pt4, Tt4)` reduces to
  `(h4−h3)/(η_b·hPR − h4)` there, so an invariant `Tt3` gives an exactly invariant `f` even
  though `pt4` moves 0.373 %.

**And the invariance does NOT ride on rung 57's stator-inert efficiency.** That was the one
way premise (ii) could have been the wrong reason, and it is ruled out structurally rather
than numerically: in `_close`, `eta_lpc`/`eta_hpc` enter **only** the pressure chain
(`pi_lpc`, `pi_hpc` → `pt4`) and have **no path to `Tt25` or `Tt3` at all**. The temperature
side is `τ`-driven and the shaft balance pins `τ`; the efficiency side is where the −0.373 %
went. So the finding would survive a fully stator-*sensitive* efficiency map — which is worth
knowing, because that concession is still standing from rung 57.

Read off the code, the ordinate identity is then two lines: `mdot4 = A4·pt4·MFP(f,Tt4)/√Tt4`
gives `ṁ/pt4 = g(f, Tt4)` exactly (choked `A4`), and `f = f(Tt3, Tt4)` with `Tt3` pinned. The
`κ_ss` invariance is not an emergent numerical coincidence; it is those two relations.

So `κ_ss` is a function of `Tt4` alone. The stator moves the mass-flow **scale** (`ṁ` and
`pt4` both by −0.373 % at `v = 0.20`) and the LP shaft speed (`nu_lp` by **+9.6 %**), and
`Wf/pt3` is homogeneous of degree zero in that scale.

**DOMAIN, stated as premises rather than discovered later:** a fully-choked machine on the CPG
branch. Rung 33's unchoked branch is the named boundary — there `MFP_A4` is no longer the
hardware group — and on a reacting gas `f` picks up composition dependence. Neither is claimed.

### The abscissa — and this is where the two spools SPLIT

The schedule is indexed on `n_H`. Whether `n_H(Tt4)` moves is a per-spool fact:

| stator | `d n_H` at fixed `Tt4` | the table | `Δ_match` |
|---|---|---|---|
| **LP**, const or scheduled | `≤ 1e−13` | **bit-identical up to solver noise** | **machine zero** |
| **HP**, `v = 0.05` | **+3.32 %** | same curve, RE-INDEXED | `+1.05e−02` |
| **HP**, `v = 0.10` | **+6.69 %** | same curve, RE-INDEXED | `+1.48e−02` |

An LP stator cannot move `n_H(Tt4)` for exactly rung 39's reason — **the map opens ONE arrow
HP→LP, and `π_LPC` cancels out of the HP-face corrected flow.** The HP face does not know the
LP stators exist. An HP stator moves the face itself.

---

## The algebra — the whole license for this rung

The matched leg `L_A` is derived on the **armed** machine, so it is a no-op on the two BARE
cells (`neither`, `fuel`) *by construction*. Therefore

    ΔI_matched  −  ΔI_bare-leg   =   M_i(both, L_A) − M_i(both, L_B)   =   Δ_match

with **no residual term**. `Δ_match` is a FIRST difference on ONE machine — same stator, same
grid, same `T_c` off the design map — so rung 58's objection to matching (*the leg differing
across cells*) does not apply to it. That is precisely why the experiment rung 58 refused is
readable here, and it is why this rung is cheap: the plant is untouched.

---

## THE MAIN TABLE

`r = 0.5`, `margin = 0.25`, `ds = 0.005`, currency `M_i`. `ΔI` is rung 58's mixed second
difference; `|ΔI| ratio` is unmatched over matched.

| stator leg | spool | `d n_H` | `credit_bare` | `ΔI` (bare leg) | `ΔI` (matched) | `Δ_match` | `|ΔI|` ratio | abscissa |
|---|---|---|---|---|---|---|---|---|
| LP sched `v_max=0.20` | lp | 0.0000 % | +5.272661e−02 | +5.016872e−03 | +5.016872e−03 | −1.8e−15 | 1.0 | — |
| LP sched `v_max=0.20` | hp | 0.0000 % | +6.216026e−04 | −4.975375e−04 | −4.975375e−04 | −1.0e−15 | 1.0 | — |
| LP const `v=0.20` | lp | 0.0000 % | +6.914726e−02 | +5.562911e−04 | +5.562911e−04 | +1.2e−14 | 1.0 | — |
| LP const `v=0.20` | hp | 0.0000 % | −1.453782e−03 | +1.500990e−03 | +1.500990e−03 | +1.3e−14 | 1.0 | — |
| **HP const `v=0.05`** | lp | 3.3209 % | +4.265509e−03 | −1.065927e−02 | **−1.113808e−04** | +1.0548e−02 | **95.7×** | **100.00 %** |
| **HP const `v=0.05`** | hp | 3.3209 % | +1.769690e−02 | −1.559667e−02 | **+1.524192e−03** | +1.7121e−02 | 10.2× | **100.00 %** |
| **HP const `v=0.10`** | lp | 6.6898 % | +7.976999e−03 | −1.513924e−02 | **−3.133029e−04** | +1.4826e−02 | **48.3×** | **100.00 %** |
| **HP const `v=0.10`** | hp | 6.6898 % | +3.439212e−02 | −2.949089e−02 | **+2.934916e−03** | +3.2426e−02 | 10.0× | **100.00 %** |

On the four LP rows every downstream quantity is identical too — `s_eng` 0.122690, fuel
removed 4.117e−03, `s*` 0.12510 — not merely the margin. **The matched leg IS the bare leg.**
The abscissa column is left blank there on purpose: `Δ_match` is machine zero, so its
decomposition is `0/0` and the shares it returns (150 %, 233 %, 76 %) are noise ratios. They
are not reported as anything else.

---

## The second finding — an UNMATCHED schedule MANUFACTURES an interaction

The `|ΔI| ratio` column is the practical result. On an HP-statored machine, running rung 58's
bare-machine leg reports an interaction **48–96× larger on the LP spool** than the leg the
machine would actually be given — and on the HP spool, the spool carrying the stator, it
reports it **with the wrong SIGN** (`−1.56e−02` unmatched vs `+1.52e−03` matched at
`v = 0.05`; `−2.95e−02` vs `+2.93e−03` at `v = 0.10`).

So "the two levers do not superpose" is, on the HP branch, very largely an artifact of
calibration mismatch rather than of physics. Matched, the pair very nearly *does* superpose.

**And the ratio is a LOWER bound, because the matched interaction is at the grid noise
floor.** Halving `ds` (0.005 → 0.0025) moves `Δ_match` by only **−0.299 %** — it is a real,
resolved object — but it moves `ΔI_matched` from `−3.13e−04` to `−8.30e−05`, a factor 3.8
*toward zero*, which is what a quantity indistinguishable from zero does under refinement. The
ratio at the finer grid is 179×. The honest statement is therefore not "48× smaller" but
**"matched, the interaction is not resolvable at this resolution, while unmatched it is 12
orders above the noise"** — and the 48× / 95.7× figures are quoted as the coarse-grid
measurement they are.

### The ratio IS clocked, and at a slow ramp it INVERTS — the stronger form

The `|ΔI|` ratio was pre-registered (P4) to stay above 10× across ramp rate. It does not, and
the way it fails is worth more than the prediction was:

| `r` | `ΔI` (bare leg) | `ΔI` (matched) | ratio |
|---|---|---|---|
| 0.25 | −3.2978e−02 | −5.0128e−03 | 6.6× |
| 0.50 | −1.5139e−02 | −3.1330e−04 | 48.3× |
| **1.00** | **+0.0000e+00 — DORMANT** | **+1.5322e−05 — BINDS** | *(envelope edge)* |

At `r = 1.00` the ratio is not 0×; it is **undefined**, because the unmatched leg is
`removed = 0.0` **exactly** — rung 58's `r = 2.0` dormancy, reappearing, and reported as the
ENVELOPE EDGE it is rather than as a measurement.

But the row is the strongest practical statement in the rung: **on a slow accel the
bare-machine schedule never engages at all, while the machine's own schedule does.** An
unmatched schedule does not merely mis-size the limiter's effect — at the slow end it misses
the engagement entirely and reports a limiter that is not there. That follows directly from
the re-indexing: the matched cap is 10–11 % lower at fixed `n_H`, so it binds on ramps where
the bare one never reaches its own.

**P4 is scored MISS as registered** — the 10× floor is `r`-dependent (6.6× at the fast end)
and undefined at the slow one. The claim that survives is the SIGN and the ORDER at the
measured `r`, and the gate is set at 5× to match what is claimed rather than at the weakest
row.

The mechanism is visible in the same rows: the re-indexed cap is **10–11 % lower at fixed
`n_H`** (the matched table maps a given `n_H` to the `κ` of a *lower* `Tt4`), so the leg
engages far earlier and cuts far harder — `s_eng` **0.2469 → 0.1247**, fuel removed
**7.59e−04 → 5.01e−03**, a factor 6.6.

---

## The isolation — 100.00 % abscissa, 0.00 % ordinate

The claim "matching is pure re-indexing" is not an inference from the two `1e−13` columns. It
is measured, by splicing the two tables (`_synthetic_leg`) and re-running the armed cell
against each half:

| leg | abscissa | ordinate | `M_i` | `s*` | `s_eng` | removed | Δ vs `L_B` |
|---|---|---|---|---|---|---|---|
| `L_B` | bare | bare | 0.466358763 | 0.24276 | 0.246883 | 7.5911e−04 | +0.000000e+00 |
| `L_A` | armed | armed | 0.481184700 | 0.12583 | 0.124668 | 5.0148e−03 | +1.482594e−02 |
| **`L_synth`** | **armed** | **bare** | **0.481184700** | 0.12583 | 0.124668 | 5.0148e−03 | **+1.482594e−02** |
| **`L_ctrl`** | **bare** | **armed** | **0.466358763** | 0.24276 | 0.246883 | 7.5911e−04 | **+3.330669e−15** |

    ABSCISSA carries  +1.482594e-02  =  100.00 %
    ORDINATE carries  +3.330669e-15  =    0.00 %

`L_synth` reproduces the real matched leg **to every digit of `M_i`, `s*`, `s_eng` and the
fuel removed**. This is the answer to the skeptic who reads `Δ_match` as *"you swapped in a
tighter schedule, of course the margin moved"*: the schedule is tighter, and the entire reason
it is tighter is that its index moved.

**The split is UNCONDITIONAL over every axis swept**, which is why it is stated without a
scope: `abscissa_share = 100.000 %` at all three HP settings (`v_hp` = 0.05, 0.10, 0.15,
i.e. out to the authority edge), at all three ramp rates (`r` = 0.25, 0.50, 1.00), and it
survives `ds`-halving. Unlike rung 58's interaction, which is strongly clocked, **this
mechanism has no clock at all** — fitting, since it is a property of a *table*, not of a march.

**But the mechanism's SIZE is not monotone in the setting, and the index shift is.** Along the
`v_hp` ladder the abscissa shift rises cleanly — **3.321 % → 6.690 % → 10.096 %** — while
`Δ_match` **turns over**: `+1.0548e−02 → +1.4826e−02 → +1.4693e−02`. There is an **interior
maximum** between `v_hp` = 0.10 and 0.15.

That is rung 48/50's truncated-descent law setting the ceiling, not a defect: re-indexing buys
its effect by moving the engagement *upstream of the incidence minimum*, and once the clip
arrests the `φ` descent essentially at the start of the ramp there is no descent left to
arrest, so further re-indexing adds nothing. The same shape as rung 55's interior optimum in
row count and rung 58's non-monotone share peaking near `r ≈ 0.25`. **`Δ_match` measures what
the re-indexing DOES, not how far the index moved** — and only the latter is monotone.

### What did NOT recover — and it is reported, not buried

Rung 58 recovered 86 % of its interaction from the two fuel-leg-free marches, by re-reading
the stator's credit profile at the relocated minimum. **The same recovery here returns 3.6 %**
(predicted `+5.41e−04` against a measured `+1.48e−02`).

That is not a failure of this rung's mechanism, it is a different mechanism, and the reason is
rung 58's own: that channel is the schedule's **state-feed**, and the HP branch is run with a
**constant** setting, which has none. Rung 58 measured the state-feed-free floor at 0.8 %.
So the profile channel is *absent by construction* here and the re-indexing carries all of it.
**No claim is made that rung 58's predictor extends to rung 59; the measurement says it does
not.**

---

## What it does to its neighbours

- **Rung 58 — its CONCESSION DISCHARGED as VACUOUS, not as small.** Rung 58 ran an **LP**
  stator. On an LP stator the matched leg is the bare leg, to machine precision, in the table
  *and* in every downstream quantity. Its four-cell numbers were **never confounded**, and the
  experiment it declined is the experiment it already ran. The false premise is corrected in
  the shipped docstring (`engine.py`) and in `docs/rung58-spec.md` § Concessions — rung 28's
  precedent for editing a shipped rung. **The discipline itself stands**: the leg is still
  ONE object, passed in, so the caller's choice stays visible. Only its stated reason was wrong.
- **Rung 58 — BOUNDED.** *"`Wf/pt3` is a coordinate the stator does not move"* — measured
  there as a 0.16 % engagement shift — is an **LP-stator** statement. An HP stator moves the
  index by 6.7 % and the engagement time by a **factor of two** (0.2469 → 0.1247), four orders
  above rung 58's number. Its law is right; its scope was one spool wider than its evidence.
- **Rung 53 — EXTENDED, and for the first time SPLIT.** Its coordinate-dependence law reaches
  a fifth object: after a margin (53), a constraint's severity (54), a lever's cost (56) and a
  limiter's composability (58), **a derived schedule's CALIBRATION**. Uniquely, this object
  splits in two: the *value* is coordinate-free (exactly), the *index* is not.
- **Rung 39 — CONFIRMED from a new side.** Its one arrow HP→LP (`π_LPC` cancels out of the HP
  face) is *why* an LP stator cannot move `n_H(Tt4)`. Rung 39 proved it for the steady match;
  here it is what makes an entire class of re-calibration a no-op.
- **Rungs 30/31 — LOAD-BEARING.** The choked `A4` and the map-free shaft balance `(★)` are
  the two premises of the ordinate invariance. This is the first rung where `(★)` does work
  outside the matching solve itself.

---

## The instrument

`ScheduledStatorTransient` grows a rung-59 section; the plant is untouched, and **no new
constant enters** — `accel_schedule` already derives on whatever machine it is called on, so
the matched leg needed no new code at all.

- `_proof_chain(…)` — the three factors `κ_ss` is built from at one steady point, so the
  ordinate claim is *checked* rather than asserted.
- `schedule_invariance(…)` — the two tables, tuple-level identity verdicts for **each half
  separately**, and the proof-chain residuals over the band.
- `_synthetic_leg(index, values)` — the splice. Asserts a common `margin`, so the isolation
  can never smuggle in a leg change.
- `_clamp_audit(…)` — the standing BLOCKER check, and the artifact most likely to have
  counterfeited this rung: `AccelSchedule.cap` **clamps** outside its abscissa bracket, and
  rung 59 re-indexes that very abscissa. A leg read outside its bracket runs on `κ[0]`, the
  envelope edge (rung 48's `m → 0`, rung 58's `r = 2.0` dormancy), not the derived shape.
  **Audited on every cell, asserted, never assumed.** Cleared at these settings: all 201 / 86 /
  234 cutting points and all three engagements strictly inside.
- **`matched_credit(…)`** — THE RUNG: the four rung-58 cells plus the matched and the two
  spliced ones, `Δ_match`, `ΔI` both ways, and the abscissa/ordinate shares.

### The reduce

1. **`v = 0` ⇒ tuple identity.** `v_max = 0` or `vsv_lp = 0` gives `L_A.kappa == L_B.kappa`
   and `L_A.n_H == L_B.n_H` as **Python tuple equality**, hence `Δ_match` exactly `0.0`. This
   is the strong identity reduce.
2. **A nonzero LP setting is NOT bit-identical, and the gate says so.** The tables agree to
   `≤ 8e−13`, not to the last bit — `equilibrium`'s Newton converges to a tolerance, not to an
   exact float. The invariance gate is therefore a **tolerance** gate at `1e−12`, and the
   bit-level gate is reserved for `v = 0`. Asserting tuple equality at nonzero `v` would fail,
   and would be claiming more than the solver can deliver.
3. Passing the same leg object to both sides reproduces rung 58's `composite_credit`
   bit-for-bit.
4. Rung 57's three readers pass no leg; rung 58's readers are untouched; the design run is
   bit-for-bit rung 6.

---

## Concessions

- **The HP branch is a CONSTANT setting, and that is deliberate.** Rung 57 conceded that its
  HP *schedule* reads `nu_H` rather than the corrected `n_H` (because `Tt25` is an output of
  the root the schedule must be armed before). A constant HP setting has no such reader, so
  the HP branch sidesteps that concession entirely — which is the only reason it is admissible
  as evidence here. An HP *schedule* would re-open it.
- **The HP stator's authority limit is `v_hp ≈ 0.15`.** At `v_hp = 0.20` the LP compressor
  trial goes off-map (`m_lp = 1.63`, the loading law non-physical). Rung 53's saturation,
  reappearing on the other spool; the ladder is run to 0.15 and no further.
- **`share` is reported and never leaned on.** `credit_bare` is a denominator from another
  regime and on one row it is *negative* (`−1.45e−03`, LP const read on the HP spool). The raw
  second differences in `M_i` carry every claim — rungs 43/45/49's currency-circularity trap.
- **One currency, one gas, one flight point.** `M_i` with its wall at the metal, CPG, as
  rungs 53/57/58. `M_φ` is reported per cell and never differenced (rung 58's finding).
- **`φ_surge` is still rung 36's imposed constant** anchoring `T_c = 1/φ_surge`. Its LEVEL is
  disclaimed exactly as in rungs 36/41/53/57/58; the load-bearing objects here are an exact
  invariance, a 100 %/0 % isolation and a ratio, none of which reads the level.

---

## The next seam

**The matched `φ` floor.** Rung 58 showed a `φ`-referenced leg is not composable at a fixed
set point, because the admissible floor bands on the bare and statored machines are DISJOINT.
The obvious repair is the one this rung just validated for `Wf/pt3`: *match the set point to
the machine*. But a floor matched in the margin coordinate under a **moving** `v` is a
state-fed set point — genuinely new plant, not a leg swap — and rung 58 already showed a floor
that binds from `s = 0` pins `M_i` and makes the credit exactly `v`, whose share is
currency-circular. So it needs its own probe and its own rung, and rung 59's result says what
to expect: matching should annihilate most of rung 58's disjointness the way it annihilated
most of the HP interaction.

> **✔ BUILT — rung 60, and the expectation was RIGHT about the disjointness and WRONG about
> what it would buy.** Matching does annihilate it (24× at a constant setting, entirely on
> rung 58's own schedule), and the canonical matching rule turns out not to be a calibration
> at all — a set point has no definition to re-run, so the two natural rules differ by exactly
> `v·sm/(1+sm)`, and only rung 58's currency finding picks between them. But the repaired leg
> is still not composable: a floor **PINS** the coordinate it watches, so the second difference
> is a difference of set points (`= v` for `φ`, `= 0` for incidence, both exact). **The
> state-fed worry was also avoidable**: the criterion `credit < excursion` leaves the constant
> ladder admissible out to `v = 0.190`, so rung 60's body is a scalar floor and no new plant.
> `docs/rung60-spec.md`.

Then, unchanged: **stator + bleed together** (rung 53's saturation), a **bleed schedule**
`b(n_L)`, and the **lag SHAPE / two-lag cascade** (rung 52's own seam).

## Anchor

`docs/plans/rung59-anchor-matched-schedule.md` — probes A–G (which fixed the mechanism and
cleared the clamp blocker before any prediction was written), the five predictions as
registered, and their scoring:
**P1 HIT · P2 HIT · P3 HIT · P4 MISS (informative — the slow-ramp dormancy inversion) ·
P5 SPLIT: HIT on the mechanism, MISS on monotonicity.**
It also records the three things the advisor got wrong (the bit-level gate, the
rung-vs-negative rule) and the one real risk it caught (whether `Tt3`'s invariance rides on
rung 57's stator-inert efficiency — ruled out structurally).
