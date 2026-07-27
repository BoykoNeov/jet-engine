# Rung 50 anchor — the forced release edge (`s_off`): probe transcripts and verified numbers

Every number here was measured OUTSIDE the repo first (`M:\claud_projects\temp\rung50\probe.py`,
a local `TwoSpoolFuelTransient` subclass), rung 49's own method. Predictions were written
before any measurement (`PREDICTIONS.md`, reproduced in § 0) so the two inversions below are
findings and not retrofits.

Config throughout: rung 49's — CPG gas, `FLIGHT(250 K, 50 kPa, M0 0.85)`, `π_LPC/π_HPC/Tt4`
= 3/6/1500, LP map `(a=0.20, b=0.05, σ=0.1, l=0.7)`, HP map `(a=0.08, b=0.15, σ=0.1, l=1.0)`,
accel 1000 → 1400 K, `ρ = 1`, `ds = 0.02`, `s_settle = 2` unless stated.

Bare references:

| `r` | min φ_lp | `s_lp*` | min φ_hp | `s_hp*` | `nu_hp_end` (settle 2 / 4) |
|---|---|---|---|---|---|
| 0.5 | 0.735466 | 0.240 | 0.861199 | 0.400 | 0.958113 / — |
| 2.0 | 0.760938 | 0.320 | 0.910688 | 0.640 | 0.958787 / 0.959064 |

---

## § 0. The predictions, as written before measuring

- **P1** `|relief_hp|` peaks at `s_off ≈ r` (ramp-clocked, rung 49 § 3 deconfounded); refuted
  if it peaks at `s_off ≈ s_hp*`.
- **P2** forcing an early release should break rung 49's gate-3 identity on the WATCHED spool;
  predicted `relief_lp` **rises monotonically and saturates**.
- **P3** `s_off ≤ s_eng` ⇒ both reliefs exactly 0; `s_off ≥` natural `s_rel` ⇒ inert.
- **P4** a release lag should GROW the debit where the natural release is short of the ramp end.

**Scored: P1 CONFIRMED (but only after the `r`=2.0 rerun — at `r`=0.5 it is confounded).
P2 CONFIRMED IN SUBSTANCE, WRONG IN SHAPE (it does not rise-and-saturate, it goes NEGATIVE
first). P3 CONFIRMED both ends. P4 not run — superseded (see § 7).**

---

## § 1. P1/P2 — the two-sided sweep at `r` = 0.5, LP-watching, `φ_lim` = 0.7450

```
  s_off   s_eng   s_rel     rel_lp     rel_hp  s@minLP  s@minHP    removed  nu_hp_end
  0.100     nan     nan   +0.00000   +0.00000    0.240    0.400   0.000000   0.958113
  0.160   0.120   0.140   -0.00056   -0.00031    0.240    0.400   0.000010   0.958111
  0.200   0.120   0.200   -0.00242   -0.00139    0.240    0.380   0.000064   0.958105
  0.260   0.120   0.260   -0.00490   -0.00384    0.280    0.380   0.000154   0.958092
  0.300   0.120   0.280   -0.00611   -0.00562    0.300    0.380   0.000189   0.958082
  0.360   0.120   0.340   -0.00358   -0.00923    0.360    0.360   0.000295   0.958062
  0.400   0.120   0.380   +0.00095   -0.01085    0.400    0.400   0.000356   0.958051
  0.440   0.120   0.420   +0.00830   -0.01016    0.440    0.440   0.000392   0.958044
  0.500   0.120   0.440   +0.00953   -0.00889    0.300    0.460   0.000396   0.958044
  ... (0.56 … 9.90 all identical to the 0.500 row: the forcing is INERT past the
      natural release, which is rung 49's unforced instrument)
```

**P3 both ends confirmed** (row 0.100 exactly zero; rows ≥ 0.500 float-identical).
**P2 confirmed in substance:** `relief_lp` is **negative** over `s_off ∈ [0.16, 0.36]`,
min −0.00611. The limiter is net-HARMFUL to the very spool it watches. Predicted shape
(monotone rise + saturation) was **wrong**.

**P1 NOT decidable here:** the `|rel_hp|` peak is at `s_off` = 0.400 = `s_hp*`, but the ramp
end is 0.500 — 2.5 cells away. This is rung 49 § 3's own confound, refused.

## § 2. P1 DECIDED — `r` = 2.0, `φ_lim` = 0.7725 (`s_hp*` = 0.640 vs ramp end 2.000, 3.1× apart)

```
  s_off   s_eng   s_rel     rel_lp     rel_hp  s@minLP  s@minHP    removed  nu_hp_end
  0.300   0.020   0.280   -0.01070   -0.00478    0.300    0.560   0.000268   0.958784
  0.450   0.020   0.440   -0.02010   -0.01458    0.460    0.460   0.000654   0.958780
  0.550   0.020   0.540   -0.02628   -0.02373    0.560    0.560   0.000980   0.958775
  0.650   0.020   0.640   -0.03213   -0.03247    0.660    0.660   0.001370   0.958769   <- s_hp*
  0.750   0.020   0.740   -0.03762   -0.04079    0.760    0.760   0.001822   0.958760
  0.900   0.020   0.880   -0.04825   -0.05450    0.900    0.900   0.002557   0.958741
  1.100   0.020   1.080   -0.05740   -0.06900    1.100    1.100   0.003801   0.958700
  1.300   0.020   1.280   -0.06379   -0.08088    1.300    1.300   0.005249   0.958630
  1.550   0.020   1.540   -0.05981   -0.08609    1.560    1.560   0.007356   0.958479
  1.800   0.020   1.780   -0.05461   -0.08920    1.800    1.800   0.009352   0.958208   <- PEAK
  1.950   0.020   1.940   -0.02993   -0.07493    1.960    1.960   0.010508   0.957998
  2.050   0.020   2.040   -0.00569   -0.05800    2.060    2.060   0.011001   0.957866   <- past r
  2.200   0.020   2.100   +0.01156   -0.04269    1.600    2.100   0.011105   0.957827
  9.900   0.020   2.100   +0.01156   -0.04269    1.600    2.100   0.011105   0.957827
```

**P1 CONFIRMED.** `|rel_hp|` = 0.03247 at `s_off` = `s_hp*`, rising straight through it to
**0.08920 at `s_off` = 1.80**, then collapsing past the ramp end. **2.75× larger at the ramp
end than at the unwatched spool's own minimum**, on an axis that moves ONLY the release edge.
Rung 49 § 3's within-family clock claim now holds deconfounded.

**The relocation signature:** `s@minLP` and `s@minHP` equal `s_off` (to one grid cell) in
every row from 0.450 on — BOTH spools' minima are relocated to the release point.

### § 2b. The same sweep re-run ON THE ds GRID — the SHIPPED numbers

The § 2 sweep used several off-grid `s_off` (0.45, 0.55, 0.65, 0.75, 0.90, 1.55, 1.95, 2.05),
where the switch straddles a step. Re-run on-grid through the shipped `release_sweep`; **this is
the table `docs/rung50-spec.md` § 1 and `tests/test_rung50.py` read.** `s_eng` = 0.020 in every
row (the engagement edge is held fixed — the point of the axis).

```
s_off  s_eng  s_rel   rel_lp     rel_hp     sminLP  sminHP  removed   nu_hp_end
 0.30  0.020  0.280   -0.01070   -0.00478   0.300   0.560  0.000268  0.958784
 0.66  0.020  0.640   -0.03474   -0.03447   0.660   0.660  0.001370  0.958768
 1.10  0.020  1.080   -0.05740   -0.06900   1.100   1.100  0.003801  0.958700
 1.56  0.020  1.540   -0.06545   -0.09049   1.560   1.560  0.007356  0.958468   <- PEAK
 1.80  0.020  1.780   -0.05461   -0.08920   1.800   1.800  0.009352  0.958208
 2.06  0.020  2.040   -0.00775   -0.05951   2.060   2.060  0.011001  0.957853
 2.20  0.020  2.100   +0.01156   -0.04269   1.600   2.100  0.011105  0.957827
```

On-grid the peak lands at `s_off` = 1.56 (0.09049) rather than 1.80, and the ratio against the
release-at-`s_hp*` row is **2.6×** rather than 2.75. The ORDERING, the walk-through-`s_hp*` and
the collapse past `r` are unchanged; the spec quotes these on-grid numbers.

**The two precondition rows** (italicised in the spec): `s_off`=0.30 has `s@minHP` = 0.560, NOT
at the release, because 0.280 is upstream of `s_hp*`=0.640; `s_off`=2.20 has `s@minLP` = 1.600
because `rel_lp` has gone POSITIVE there — the credit branch won. Both are gated.

Endpoint at rung 49's gate-10 settle (`s_settle` = 4.0), same sweep:

```
  s_off=0.66  nu_hp_end=0.959064  bare=0.959064  d=-1.64e-07  removed=0.001370  rel_hp=-0.03447
  s_off=1.56  nu_hp_end=0.959062  bare=0.959064  d=-1.75e-06  removed=0.007356  rel_hp=-0.09049
  s_off=1.80  nu_hp_end=0.959061  bare=0.959064  d=-3.23e-06  removed=0.009352  rel_hp=-0.08920
  s_off=2.20  nu_hp_end=0.959058  bare=0.959064  d=-5.75e-06  removed=0.011105  rel_hp=-0.04269
```

`s_off` 1.56 → 2.20 removes **51 % more fuel** for **less than half** the debit.

## § 3. ds convergence (blocking check) — `r` = 0.5, `φ_lim` = 0.7450, `s_off` on-grid

```
  ds=0.020  s_off=0.30  s_rel=0.280  s@minLP=0.300 (+0.000)  s@minHP=0.380 (+0.080)  rel_lp=-0.00611  rel_hp=-0.00562
  ds=0.020  s_off=0.40  s_rel=0.380  s@minLP=0.400 (+0.000)  s@minHP=0.400 (+0.000)  rel_lp=+0.00095  rel_hp=-0.01085
  ds=0.020  s_off=0.44  s_rel=0.420  s@minLP=0.440 (+0.000)  s@minHP=0.440 (+0.000)  rel_lp=+0.00830  rel_hp=-0.01016
  ds=0.010  s_off=0.30  s_rel=0.290  s@minLP=0.300 (+0.000)  s@minHP=0.380 (+0.080)  rel_lp=-0.00624  rel_hp=-0.00568
  ds=0.010  s_off=0.40  s_rel=0.390  s@minLP=0.400 (+0.000)  s@minHP=0.400 (+0.000)  rel_lp=+0.00087  rel_hp=-0.01089
  ds=0.010  s_off=0.44  s_rel=0.430  s@minLP=0.440 (+0.000)  s@minHP=0.440 (+0.000)  rel_lp=+0.00831  rel_hp=-0.01014
```

The relocation is **grid-independent** (the minimum lands AT `s_off`, offset 0.000, at both
`ds`) and the depths converge to **0.2–2 %** — far tighter than rung 49's own gate-12 drift
(~13 %), because the forced dive is anchored to an imposed `s_off` rather than to a solved edge.

**The one row that sharpens the law:** at `s_off` = 0.30 the HP minimum stays at 0.380, NOT at
`s_off` — the release is UPSTREAM of the HP's own bare minimum (0.400), so the bare basin is
still ahead and still governs. **The relocation holds once the release lands at or after that
spool's own bare minimum; before it, the bare minimum wins.** Same at `r`=2.0 § 2 row 0.300
(`s@minHP` = 0.560 against `s_off` = 0.300).

## § 4. Endpoint (blocking check) — `r` = 2.0 at rung 49's gate-10 settle, `s_settle` = 4.0

```
  BARE nu_hp_end=0.959064
  s_off= 0.75  nu_hp_end=0.959064  d=-2.17e-07  removed=0.001822  rel_hp=-0.04079
  s_off= 1.30  nu_hp_end=0.959063  d=-9.19e-07  removed=0.005249  rel_hp=-0.08088
  s_off= 1.80  nu_hp_end=0.959061  d=-3.23e-06  removed=0.009352  rel_hp=-0.08920
  s_off= 2.20  nu_hp_end=0.959058  d=-5.75e-06  removed=0.011105  rel_hp=-0.04269
  s_off= 9.90  nu_hp_end=0.959058  d=-5.75e-06  removed=0.011105  rel_hp=-0.04269
```

**≤ 5.75e-6** — two orders under rung 49's 5e-4 gate-10 threshold. (The 9.6e-4 seen in § 2 is
an UNDER-SETTLED march, `s_settle`=2 at `r`=2; not a physical endpoint shift.) And the
anti-deflation card: `s_off` 1.80 → 2.20 removes **19 % MORE fuel** for **less than half** the
debit. Fuel removal is monotone in `s_off`; the debit is peaked. A ramp-rate lever cannot do that.

## § 5. THE SEAM TEST — rung 48's leg with a forced release (`r` = 0.5, m = 0.25)

Rung 49's standing OPEN seam: *"WHY rung 48's leg is immune to the release debit … the clip
SHAPE is the obvious suspect, but it is NOT measured here."*

```
  s_off   s_eng   s_rel     rel_lp     rel_hp  s@minLP  s@minHP    removed
  0.160   0.140   0.140   -0.00050   -0.00028    0.240    0.400   0.000007
  0.240   0.140   0.240   -0.00633   -0.00405    0.260    0.380   0.000170
  0.300   0.140   0.280   -0.01135   -0.00907    0.300    0.360   0.000293
  0.360   0.140   0.340   -0.01528   -0.01811    0.360    0.360   0.000538
  0.440   0.140   0.420   -0.01867   -0.03050    0.440    0.440   0.000977
  0.500   0.140   0.480   -0.01991   -0.03825    0.500    0.500   0.001388
  0.700   0.140   0.680   +0.00850   -0.00901    0.120    0.700   0.002742
  9.900   0.140   1.120   +0.00850   +0.03574    0.120    0.140   0.004110   <- rung 48 itself
```

**The seam CLOSES: the immunity is TIMING, not SHAPE.** Left alone, rung 48's leg releases at
1.120 — post-ramp — and delivers its rung-48 credit (+0.0085 / +0.0357). Forced to release
inside the ramp it **debits both spools** (−0.0199 / −0.0383), with the same
minimum-relocation signature. Nothing about its clip shape protects it.

Cross-regime, `r` = 2.0 at m = 0.15 (the corrected band floor; m=0.25 never engages at `r`=2):

```
  s_off= 0.30  s_eng=nan    s_rel=nan    rel_lp=+0.00000  rel_hp=+0.00000  s@minHP=0.640
  s_off= 0.64  s_eng=0.360  s_rel=0.620  rel_lp=+0.00000  rel_hp=-0.00369  s@minHP=0.640
  s_off= 1.10  s_eng=0.360  s_rel=1.080  rel_lp=+0.00000  rel_hp=-0.00887  s@minHP=1.100
  s_off= 1.56  s_eng=0.360  s_rel=1.540  rel_lp=+0.00000  rel_hp=-0.00351  s@minHP=1.560
  s_off= 1.80  s_eng=0.360  s_rel=1.780  rel_lp=+0.00000  rel_hp=+0.00319  s@minHP=1.800
  s_off= 1.96  s_eng=0.360  s_rel=1.940  rel_lp=+0.00000  rel_hp=+0.00563  s@minHP=0.360
  s_off= 9.90  s_eng=0.360  s_rel=2.240  rel_lp=+0.00000  rel_hp=+0.00563  s@minHP=0.360
```

Same conclusion out of rung 49's `s_hp*`-vs-`r` confound. Note `rel_lp` ≡ **exactly 0.00000**
in every row — rung 48's exact-zero law (`s_eng` = 0.360 > `s_lp*` = 0.320) survives the
forcing untouched, because every release here lands past the LP basin.

## § 6. The DEFICIT factor, deconfounded — and rung 49 § 4's refutation CORRECTED

Rung 49 § 4 refuted hand-back magnitude as the explanation, measuring it *anti*-correlated.
But it measured along a path where timing and magnitude moved TOGETHER. Fix the release time
and sweep the deficit instead.

**(a) Within family, `r` = 2.0, ALL FOUR releasing at `s_rel` = 0.740:**

```
  phi_lim=0.7670  s_eng=0.100  removed=0.000658  rel_lp=-0.00102  rel_hp=-0.01330
  phi_lim=0.7690  s_eng=0.060  removed=0.001084  rel_lp=-0.01445  rel_hp=-0.02325
  phi_lim=0.7710  s_eng=0.040  removed=0.001508  rel_lp=-0.02778  rel_hp=-0.03328
  phi_lim=0.7725  s_eng=0.020  removed=0.001822  rel_lp=-0.03762  rel_hp=-0.04079
```

**(b) ACROSS families, `r` = 0.5, all three releasing at `s_rel` = 0.420:**

```
  phi floor 0.7450   removed=0.000392   rel_hp=-0.01016
  phi floor 0.7500   removed=0.000783   rel_hp=-0.02252
  rung48  m=0.25     removed=0.000977   rel_hp=-0.03050
```

**At fixed release time the debit is MONOTONE INCREASING in the deficit — within the family
AND across two instrument families.** Rung 49 § 4's "the magnitude explanation is refuted /
does not transfer" was a **confound**, and rung 48's clip is not gentler per unit deficit —
it is **worse**. The functional form is measured, not derived (rung 49's own concession
carried): the two blocks give 2.8× ⇒ 3.1× and 1.4× ⇒ 1.8×, mildly superlinear and
disagreeing on exponent. Claimed only as monotone.

## § 7. The mechanism reading — one inferred story MEASURED AND DISCARDED

The two families peak at different `s_off` (rung 48's m=0.15 at ≈1.10, the φ floor at ≈1.80).
The tempting explanation — that their deficit-at-release trends run opposite — was measured
and is **WRONG**. Instantaneous deficit at the last engaged point, `r` = 2.0:

```
  s_off | rung48 m=0.15: s_rel  deficit%  rel_hp | phi floor 0.7725: s_rel  deficit%  rel_hp
   0.64 |  0.620    6.164  -0.00369 |  0.620   29.085  -0.03268
   1.10 |  1.080   11.638  -0.00887 |  1.080   39.393  -0.06900
   1.56 |  1.540   12.572  -0.00351 |  1.540   41.484  -0.09049
   1.80 |  1.780   11.566  +0.00319 |  1.780   36.623  -0.08920
   1.96 |  1.940   10.341  +0.00563 |  1.940   27.605  -0.07800
```

**Both families' deficit-at-release peak at the SAME place (`s_rel` ≈ 1.54).** The differing
net-relief peaks are therefore NOT a deficit-trend effect. They are the **credit branch taking
over**: rung 48's rising cap arrests the HP descent hard, so its credit branch is large, and
once the dive is shallow enough the credit wins and the net relief goes positive
(`s@minHP` snaps back to `s_eng` = 0.360 at `s_off` ≥ 1.96). The φ floor's deficit is ~3.4×
larger at matched release, so its dive branch keeps winning even post-ramp.

**The law is therefore a MINIMUM OVER TWO BRANCHES, not a sum**, and § 6's monotone-in-deficit
statement applies to the DIVE branch only. This paragraph replaces the inferred story; the
inference is recorded here because it was made and then killed by measurement.

## § 8. Superseded prediction P4 (the release lag)

Not run. The `s_off` isolation produced the findings, and a `τ_rel` lag would confound total
deficit against deficit RATE — which none of the data above separates. Named as rung 50's
first open seam with exactly that question attached.
