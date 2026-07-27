# Rung 50 — The release edge, ISOLATED: the closing edge relocates BOTH spools' minima to itself, and a limiter's immunity is TIMING, not clip SHAPE

Rung 49 found that a fuel-side limiter acts on a spool through **both** its edges, and that the
two answer to different clocks: the engagement edge truncates a descent (a credit, rung 48's
term), the release edge re-opens one (a debit, new). But it could only move the release edge by
moving `φ_lim` — which drags the **engagement** edge, the **window length** and the **clip
depth** along with it. So its clock result was hedged, honestly and correctly, as a
**within-family** result:

> *"the ratio that governs the debit within the φ-floor family (`s_rel/r`) does **not** carry
> across instrument types … The clock result below is a WITHIN-FAMILY result and is claimed
> only as one."*

And it left a named open seam:

> *"**WHY rung 48's leg is immune to the release debit is an OPEN SEAM.** … Something about
> the *shape* of the clip … is the obvious suspect, but it is **not measured here**."*

This rung builds the instrument that decides both: a **forced release time `s_off`**, which
slides the closing edge **alone**, two-sided, with everything up to it bit-identical.

**THE HEADLINE.** The release edge is not merely *a* term in the law — it is the **dominant**
one. It **relocates BOTH spools' minima to itself**: whenever the release lands at or after a
spool's own bare minimum, that spool's limited-march minimum sits **at the release point**, for
the watched spool and the unwatched one alike, in both instrument families, grid-independently.
Three consequences follow, and each corrects or bounds a prior rung:

1. **The debit is RAMP-clocked — deconfounded.** At `r`=2.0 it is **2.6× larger** with the
   release at the ramp end than at the unwatched spool's own minimum, walking straight through
   that minimum without noticing it. Rung 49 § 3's within-family hedge can be **lifted**.
2. **A limiter forced to release early DEBITS THE SPOOL IT WATCHES** (−0.064 at `r`=2.0). Rung
   49's watched-side identity is **BOUNDED** to the unforced instrument — the same shape as
   rung 49 bounding rung 48, and it is not broken by this.
3. **THE SEAM CLOSES: rung 48's immunity is TIMING, not SHAPE.** Rung 48's own leg, forced to
   release inside the ramp, debits both spools exactly like the φ floor. Nothing about its clip
   protects it — its natural release is simply post-ramp. Rung 49's named suspect is
   **refuted**, and with it rung 49 § 4's "the magnitude explanation does not transfer": at
   **fixed** release time the debit is monotone in the deficit **across both families**.

---

## The instrument — an ISOLATION DIAGNOSTIC, not a control law

```
        the armed min-select leg (accel | surge) is DISARMED for  s ≥ s_off
```

No engine has a limiter that drops out at a wall-clock time. **`s_off` is a channel-isolation
diagnostic, and the project ships several**: rung 34/40's `freeze='lp'` holds a spool's speed
against its own ODE (physically impossible), rung 41's `surge_margin_channels` and rung 43's
`freeze_channels` are the same move. Each exists to break a coupling that the physical
instrument cannot break. This is one of those, and it is named as one in the docstring.

**Why it is the right isolation.** `φ_lim` and `m` are *window* dials: tighten either and the
window opens at BOTH ends while the clip deepens. `s_off` moves exactly one edge, and moves it
**two-sided** — earlier *and* later than the natural release. A hysteresis or a lag can only
move it later (φ recovers monotonically; a lag postpones), so neither can reach the early-release
regime where finding 2 lives.

**Why it costs no state.** `armed = s < s_off` is a pure function of `s`, and the march is
already non-autonomous through `fuel_schedule(s)`, so `s` threads into the RK4 sub-steps exactly
as the schedule already does. A boolean **latch** would have flipped between k1 and k4 and
silently destroyed the integrator's order — rung 47 hit that and answered it with a continuous
third state; here no state is needed at all. `s_off` must be passed **on the `ds` grid** or the
switch straddles a step.

**No new constant.** `s_off` is a swept coordinate, not an imposed scalar. Rung 49's `φ_lim` and
rung 48's `m` are inherited with their disclaimers intact.

---

## THE FINDINGS (config: rung 49's — CPG gas, accel 1000→1400 K, `ρ`=1, `ds`=0.02; `tests/test_rung50.py` reproduces; full transcripts in the anchor)

**Every table below is produced by the shipped `release_sweep`, on the `ds` grid** — the same
call `tests/test_rung50.py` makes, so each row is reproducible verbatim from the public API.

Bare references: `r`=0.5 → min φ_lp 0.735466 @ `s_lp*`=0.240, min φ_hp 0.861199 @ `s_hp*`=0.400.
`r`=2.0 → 0.760938 @ 0.320, 0.910688 @ 0.640.

### 1. THE HEADLINE — the release edge relocates BOTH spools' minima to itself

LP-watching `φ_lim`=0.7725 at `r`=2.0, sweeping the forced release **on the `ds` grid** (this is
the sweep `tests/test_rung50.py` reads; `s_eng` = 0.020 in **every** row — the engagement edge is
held fixed, which is the whole point of the axis):

| `s_off` | 0.30 | 0.66 | 1.10 | **1.56** | 1.80 | 2.06 | 2.20 |
|---|---|---|---|---|---|---|---|
| `s_rel` | 0.280 | 0.640 | 1.080 | **1.540** | 1.780 | 2.040 | 2.100 |
| `s@min φ_lp` | 0.300 | 0.660 | 1.100 | **1.560** | 1.800 | 2.060 | *1.600* |
| `s@min φ_hp` | *0.560* | 0.660 | 1.100 | **1.560** | 1.800 | 2.060 | 2.100 |
| `relief_lp` | −0.0107 | −0.0347 | −0.0574 | **−0.0655** | −0.0546 | −0.0078 | *+0.0116* |
| `relief_hp` | −0.0048 | −0.0345 | −0.0690 | **−0.0905** | −0.0892 | −0.0595 | −0.0427 |

Both minima track the release point cell-for-cell. **The two italicised exceptions are the law's
two preconditions, and both are gated:**

- **(a) the release must land at or after that spool's own bare minimum.** At `s_off`=0.30 the
  release (0.280) is upstream of `s_hp*`=0.640, so the re-opened dive merges into the
  still-ongoing bare descent and bottoms in the bare basin (0.560) instead.
- **(b) the dive branch must actually beat rung 48's truncation branch** — i.e. that spool's
  relief must be negative. At `s_off`=2.20 the LP's relief has gone **positive** (+0.0116): the
  credit branch has won, and its minimum sits back at 1.600, nowhere near the release.

That conjunction *is* § 6's two-branch law, stated as a precondition rather than as a slogan.

**It is not a grid artifact**, checked at **both** ramp rates — including `r`=2.0, whose dives
are ~8× deeper than `r`=0.5's and are where an artifact would most plausibly hide:

| `r` = 0.5, `φ_lim`=0.7450 | `s_off`=0.30 | 0.40 | 0.44 |
|---|---|---|---|
| `s@min φ_lp` at `ds`=0.02 / 0.01 | 0.300 / 0.300 | 0.400 / 0.400 | 0.440 / 0.440 |
| `relief_hp` at `ds`=0.02 / 0.01 | −0.00562 / −0.00568 | −0.01085 / −0.01089 | −0.01016 / −0.01014 |

| `r` = 2.0, `φ_lim`=0.7725 | `s_off`=1.10 | 1.56 |
|---|---|---|
| `s@min φ_lp` / `s@min φ_hp` at `ds`=0.02 | 1.100 / 1.100 | 1.560 / 1.560 |
| `s@min φ_lp` / `s@min φ_hp` at `ds`=0.01 | 1.100 / 1.100 | 1.560 / 1.560 |
| `relief_hp` at `ds`=0.02 / 0.01 | −0.06900 / −0.06945 | −0.09049 / −0.09108 |

Offset **0.000 at both `ds` at both ramp rates**; depths converged to **0.2–2 %** at `r`=0.5 and
**0.7 %** at `r`=2.0 — the deep dives converge *better*, not worse. Far tighter than rung 49's
own ~13 % gate-12 drift, because a forced dive is anchored to an imposed `s_off` rather than to
a solved edge. So the convergent statement is *"the minimum relocates **to the release
point**"*; the one-cell offset in the raw tables is discretization.

### 2. THE DISCRIMINATOR, DECONFOUNDED — rung 49 § 3's hedge lifted

At `r`=2.0 the unwatched spool's own minimum (0.640) and the ramp end (2.000) sit **3.1× apart**.
Reading `relief_hp` off the § 1 table: **−0.0345 with the release AT `s_hp*`**, deepening
monotonically straight through it to **−0.0905 at `s_off`=1.56**, then collapsing past the ramp
end (−0.0595 at 2.06, −0.0427 at 2.20).

**2.6× larger near the ramp end than at the unwatched spool's own minimum**, with the engagement
edge (`s_eng`=0.020 in every row) and the clip depth held fixed. Rung 49 measured this ordering
by sweeping `φ_lim`; it now holds on an axis that moves **only** the release edge, so the
**within-family hedge is lifted**: the credit is per-spool (rung 48), the debit is ramp-clocked.

*The mechanism is a product of two factors, and both are needed:* the dive needs an accumulated
**deficit** (which grows with the window) **and** a still-ramping **plant** (which dies at `r`).
The deficit keeps growing past the peak — `fuel_removed` 0.007356 → 0.011105 from `s_off` 1.56 →
2.20, **51 % more fuel removed** — while the debit **more than halves**. The product peaks just
inside `r`.

### 3. THE WATCHED SPOOL IS NOT SAFE — rung 49's identity BOUNDED

Rung 49's gate 3 asserts `relief_watched = φ_lim − min φ_bare` identically, and calls it
definitional. It is — **under the unforced instrument.** Force the release early and it fails,
in the only direction that matters:

| `s_off` (`r`=0.5, `φ_lim`=0.7450) | 0.16 | 0.20 | 0.26 | **0.30** | 0.36 | 0.44 | 0.60 (past release) |
|---|---|---|---|---|---|---|---|
| `relief_lp` (**watched**) | −0.00056 | −0.00242 | −0.00490 | **−0.00611** | −0.00358 | +0.00830 | +0.00953 |
| `relief_hp` | −0.00031 | −0.00139 | −0.00384 | −0.00562 | −0.00923 | −0.01016 | −0.00889 |

**A surge limiter released too early leaves the spool it is protecting WORSE OFF than no
limiter at all.** The predicted shape (a monotone rise to saturation) was written down before
the measurement and was **wrong** — it dips negative first. Rung 49 saw none of this because an
LP-watching floor's natural release (0.440) always lands past the LP basin `[0.15, 0.32]`; its
"the exposed spool is the LATE one" is therefore a statement about **where the natural release
lands**, not about the spools. Rung 49 is **bounded, not corrected** here — the same shape in
which it bounded rung 48.

### 4. THE SEAM CLOSES — rung 48's immunity is TIMING, not clip SHAPE

Rung 48's own leg (`Wf/pt3`, m=0.25), `r`=0.5, forced to release inside the ramp:

| `s_off` | 0.24 | 0.30 | 0.36 | 0.44 | 0.50 | 0.70 | **unforced (`s_rel`=1.120)** |
|---|---|---|---|---|---|---|---|
| `relief_lp` | −0.00633 | −0.01135 | −0.01528 | −0.01867 | −0.01991 | +0.00850 | **+0.00850** |
| `relief_hp` | −0.00405 | −0.00907 | −0.01811 | −0.03050 | −0.03825 | −0.00901 | **+0.03574** |
| `s@min φ_hp` | 0.380 | 0.360 | 0.360 | 0.440 | 0.500 | 0.700 | 0.140 |

**Left alone it credits both spools; forced inside the ramp it debits both, with the same
relocation signature.** Its clip shape is unchanged throughout. **Rung 49's named suspect is
refuted and the seam closes: the immunity is that its natural release is POST-RAMP.**

Cross-regime, out of rung 49's own `s_hp*`-vs-`r` confound (`r`=2.0, m=0.15, the corrected band
floor — m=0.25 never engages on so slow a ramp): `relief_hp` = 0 → −0.0037 (`s_off`=0.64) →
−0.0089 (1.10) → −0.0035 (1.56) → +0.0032 (1.80) → **+0.0056 unforced**. Same conclusion.

Note `relief_lp` ≡ **exactly 0.00000** in every `r`=2.0 row: rung 48's exact-zero law
(`s_eng`=0.360 is downstream of `s_lp*`=0.320) **survives the forcing untouched**, because every
release there lands past the LP basin. The two laws coexist rather than compete.

### 5. THE DEFICIT FACTOR — rung 49 § 4's refutation was itself CONFOUNDED

Rung 49 § 4 refuted hand-back magnitude as the explanation, measuring it *anti*-correlated. But
it swept magnitude and timing **together**. Fix the release time and the sign reverses:

**(a) Within family, `r`=2.0, all four releasing at `s_rel`=0.740:**

| `φ_lim` | 0.7670 | 0.7690 | 0.7710 | 0.7725 |
|---|---|---|---|---|
| `fuel_removed` | 0.000658 | 0.001084 | 0.001508 | 0.001822 |
| `relief_hp` | −0.01330 | −0.02325 | −0.03328 | −0.04079 |

**(b) Across families, `r`=0.5, all three releasing at `s_rel`=0.420:**

| | φ floor 0.7450 | φ floor 0.7500 | rung 48 m=0.25 |
|---|---|---|---|
| `fuel_removed` | 0.000392 | 0.000783 | 0.000977 |
| `relief_hp` | −0.01016 | −0.02252 | −0.03050 |

**At fixed release time the debit is monotone increasing in the deficit — within the family AND
across two instrument families.** Rung 48's clip is not gentler per unit deficit; it is
**worse**. The functional form is *measured, not derived* (rung 49's concession carried): the
two blocks give 2.8×⇒3.1× and 1.4×⇒1.8×, mildly superlinear and disagreeing on exponent.
**Claimed only as monotone.**

### 6. One inferred mechanism, MEASURED AND KILLED (recorded because it was made)

The two families peak at different `s_off` (rung 48's m=0.15 at ≈1.10, the φ floor at ≈1.56).
The tempting reading — that their deficit-at-release trends run opposite — was measured and is
**wrong**: both peak at the same place (`s_rel`≈1.54, at 12.6 % and 41.5 %). The differing net
peaks are the **credit branch taking over**: rung 48's rising cap arrests the HP descent hard, so
once the dive is shallow enough the credit wins and `s@min φ_hp` snaps back to `s_eng`. The φ
floor's deficit is ~3.4× larger at matched release, so its dive keeps winning even post-ramp.

**So the law is a MINIMUM OVER TWO BRANCHES, not a sum:**

```
   min_s φ_limited  =  min(  min_{s ≤ s_eng} φ_bare ,            <- rung 48's truncation (credit)
                             the dive bottoming AT s_rel  )      <- rungs 49/50 (debit)

   dive depth:  monotone increasing in the DEFICIT at release (§ 5, cross-family)
                peaked in the RAMP REMAINING at release (§ 2, per-spool-blind)
```

Rung 48's law is the case where the first branch wins; rung 49's headline is the case where the
second does. § 5's monotone-in-deficit statement applies to the **dive branch only**.

### 7. NOT rung 44's ramp-rate lever — the non-tautology gate

Rung 48/49's three exclusions, all measured:

- **The endpoint is unmoved.** At rung 49's gate-10 settle (`s_settle`=4, `r`=2.0), `nu_hp_end`
  moves by **≤ 5.75e-6** against bare 0.959064 — two orders under the 5e-4 threshold. (An
  under-settled march at `s_settle`=2 shows 9.6e-4; that is the settle, not a physical shift,
  and is reported here rather than hidden.)
- **Fuel removal is MONOTONE while the debit is PEAKED.** `s_off` 1.56 → 2.20 removes **51 %
  MORE fuel** for **less than half** the debit (−0.0905 → −0.0427). A ramp-rate lever cannot.
- **One clip, two signs, both ways.** § 3's early-release rows debit the watched spool while
  § 1's late rows credit it — from the same leg on the same plant.

---

## Reduce-to-prior contract (the spine)

- `s_off=None` ⇒ the gate is never applied ⇒ `integrate_fuel` is **bit-for-bit** rungs
  43/45/46/47/48/49. Exact dispatch: `armed` short-circuits on `s_off is None`.
- `s_off` beyond the natural release is **inert** — float-for-float the unforced leg
  (rung 49's φ floor, rung 48's schedule).
- `s_off ≤ s_eng` ⇒ the leg never engages ⇒ the march is float-for-float **bare**.
- `s_off` requires an armed `accel` or `surge` leg (**asserts** otherwise). The rung-46/47
  topping governor is out of scope: its window is post-ramp by construction, and the lagged path
  carries the clip amount as a STATE, so forcing its release is a different instrument.
- `lp_disabled` **asserts**: the finding is inherently two-shaft (both spools' minima relocate).
- The design run `build_turbojet(…).run(…)` is untouched — bit-for-bit rung 6.

## Verification gates (`tests/test_rung50.py`)

1. `s_off=None` ⇒ bit-for-bit rungs 43/46/47/48/49 on `(nu_lp, nu_hp, phi_lp, phi_hp, Tt4, f, mf)`,
   and `release_relief(s_off=None)` equals rung 49's `surge_relief` exactly.
2. `s_off` past the natural release is float-for-float the unforced leg; `s_off ≤ s_eng` is
   float-for-float bare.
3. **THE HEADLINE** — for every `s_off` whose release lands at or after a spool's own bare
   minimum, that spool's `argmin φ` sits **at `s_off`** to within one grid cell, for **both**
   spools; and the precondition bites (at `s_off`=0.30, `r`=2.0 the HP min stays at its bare
   basin, not at `s_off`).
4. **THE DISCRIMINATOR** — at `r`=2.0, `|relief_hp|` is monotone in `s_off` straight through
   `s_hp*`, and exceeds its value at `s_hp*` by **> 2.5×** at the peak near the ramp end.
5. **The watched spool is debited** — at `r`=0.5 there exists `s_off` with `relief_lp < 0` on an
   LP-watching floor, and rung 49's identity is recovered as `s_off` → past the natural release.
6. **THE SEAM** — rung 48's `accel` leg, forced to release inside the ramp, gives
   `relief_lp < 0` **and** `relief_hp < 0`; unforced, the same leg gives both **> 0**.
7. **Cross-regime seam** — at `r`=2.0, m=0.15: `relief_hp < 0` forced inside the ramp, `> 0`
   unforced, and `relief_lp` is **exactly 0.0** in every row (rung 48's law survives).
8. **The deficit factor at FIXED release** — three legs matched to the same `s_rel` (two φ
   floors + rung 48's schedule) have `fuel_removed` and `|relief_hp|` in the **same order**.
9. **Non-tautology** — `fuel_removed` monotone in `s_off` while `|relief_hp|` is peaked (the
   largest removal is not the largest debit), and `nu_hp_end` unmoved to 5e-4 at `s_settle`=4.
10. **Robustness** — the relocation offset and the debit survive `ds` ∈ {0.02, 0.01}
    (depth change < 5 %), and the split's sign survives `ρ` ∈ {0.25, 4}.
11. The design run is bit-for-bit rung 6.

## Concessions

- **`s_off` is not a control law.** It is an isolation diagnostic, and every number here is a
  statement about *mechanism*, not about a limiter anyone would build. What it licenses is the
  reading of the two *physical* legs (rungs 48/49), whose release edges it explains.
- **The dive-depth law is measured, not derived** — monotone in the deficit at fixed release,
  peaked in the ramp remaining. Neither functional form is derived, and § 5's two blocks
  disagree on exponent. Rung 49's concession, carried.
- **`φ_lim` and `m` inherit rungs 36/41/48/49's imposed constants.** Magnitudes disclaimed;
  signs, ordering, crossings and ds-convergence are the claims.
- **The release is still an instantaneous hand-back.** `s_off` moves *when* the fuel is returned,
  never *how fast*. A finite `τ_rel` would separate total deficit from deficit RATE — and
  **nothing measured here separates them**, which is exactly why `τ_rel` is named as the next
  seam rather than folded in.
- The plant is rung 43's non-equilibrium (CPG/TPG) gas — rung 35's standing concession.

## Anchor

`docs/plans/rung50-anchor-release-edge.md` — the probe transcripts, the predictions as written
before measuring (two scored wrong on shape), and the verified numbers.
