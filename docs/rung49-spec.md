# Rung 49 — The φ / surge-margin FEEDBACK limiter: a limiter acts on a spool through BOTH its edges, and the two edges answer to DIFFERENT clocks

Rungs 46/47/48 built three fuel-side limiters and rung 48 unified them into one law: *a
fuel-side limiter rebates a spool IFF it engages UPSTREAM of that spool's own surge minimum.*
`docs/both-edges-limiter-negative.md` then hardened that law into a **magnitude** —

```
        relief  =  min_{s ≤ s_eng} φ_bare  −  min_s φ_bare                        (rung 48+)
```

— an **edge** condition, and proved the window's *closing* edge could play no part, because
**no `pt3`-filter limiter can close its window inside the ramp**: every proxy signal (`pt3`,
`Wf`, `n`, and every filter of them) rises monotonically through the ramp, so its release is
structurally post-ramp. That negative named its own successor:

> *"the only signals with a turnover UPSTREAM of a surge minimum are the surge variables
> themselves … the live door is a **φ / surge-margin FEEDBACK limiter** — a limiter that
> watches the thing being protected rather than a proxy for it."*

This rung walks through that door, and the door was worth walking through: the instrument
**does** close its window inside the ramp, and the closing edge turns out to be **the opposite
sign to everything the ladder has measured so far.**

**THE HEADLINE.** A φ-floor on ONE spool **DEBITS the other**. The engagement edge truncates
a descent (a credit, rung 48's term); the **release edge RE-OPENS one** (a debit, new). Both
are real, they compete, and — the part that makes this a rung rather than a footnote — **they
answer to different clocks**: the credit is set by *that spool's own* surge minimum (per-spool,
rung 48), while the debit is set by **the RAMP END** (common-mode, rung 44's clock). Measured
at `r = 2.0`, where those two references sit 3.1× apart, the debit is **8× larger** at
`s_rel ≈ r` than at `s_rel ≈ s_hp*`. Rung 48's law is not refuted — it is **BOUNDED**: it is
the one-shot-arrest special case, exact whenever the release lands well past the ramp, which is
exactly the regime its own leg was structurally confined to.

---

## The control law — FEEDBACK ON THE PROTECTED VARIABLE, one imposed scalar

```
        Wf  ≤  the fuel that holds   φ_spool  ≥  φ_lim         (spool ∈ {lp, hp})
```

min-selected onto whatever fuel the (bare | topped | topped-lagged | scheduled) path would have
applied, so the composite is `min(schedule, topping, accel_cap, surge_cap)`. `φ = ṁ_corr/n` is
the map's own flow coefficient — the variable rungs 36/41/44/45 measure surge against — so this
is the first leg in the ladder whose **sensed** signal is the **protected** one. Rung 46/47's
governor is feedback on TIT; rung 48's schedule is feedforward on pressure. Neither watches φ.

The cap is **implicit** in `Wf` (φ moves with the fuel through `_close_fuel`), so it is a
bracketed Illinois set-point solve — the same structure as rung 46's `_topping_fuel` and rung
48's `_sched_fuel`, and for the same reason. φ falls monotonically with fuel at fixed spool
speeds (more fuel ⇒ hotter `Tt4` ⇒ less choked-NGV corrected capacity ⇒ less flow at the same
`n`), so the bracket is sound.

**The imposition is ONE scalar, `φ_lim`,** and it is the *same* disclaimed constant the ladder
has carried since rung 36: `SurgeLimiter.from_margin(cmap, sm)` sets `φ_lim = (1+sm)·φ_surge`
off the map's own imposed surge line. **Consequence, stated up front: the MAGNITUDE of every
relief number here is disclaimed. The SIGNS, the ORDERING, the CROSSING and the ds-convergence
are the claims** — rung 36/41's discipline, unchanged.

## Sign-space only — inherited from rungs 41/44/45/46/47/48, plus one imposed `φ_lim`

No new physics, no new constant beyond `φ_lim`. The plant is rung 43's, the surge object is
rung 45's reference-free raw `min φ`, the relief differential is rung 46's `topping_relief`
shape and rung 48's `schedule_relief` shape.

---

## THE FINDINGS (config: CPG gas, accel 1000→1400 K, `ρ = 1`, `ds = 0.01`; `tests/test_rung49.py` reproduces)

Bare march at `r = 0.5`: **LP min `φ` = 0.735448 @ `s_lp*` = 0.230; HP min `φ` = 0.861169 @
`s_hp*` = 0.390.** LP start `φ` = 0.773116. `nu_hp` at settle = 0.959060.

### 1. The instrument WORKS — a clean SLIDING MODE, and the FIRST window with BOTH edges inside the ramp (the enabling measurement)

The naive worry is chatter: the clip raises φ (`both-edges` § arrest), which makes the leg
dormant, which restores fuel, which pushes φ back down. **Measured: no chatter.** The
set-point solve rides the floor exactly — φ_LP = `0.750000` to 6 dp at *every* engaged point,
`s` = 0.090 … 0.520, then releases cleanly.

And the window **closes inside the ramp**:

| `φ_lim` (watch LP) | 0.7650 | 0.7550 | 0.7500 | 0.7450 | **0.7400** | **0.7370** |
|---|---|---|---|---|---|---|
| `s_eng` | 0.030 | 0.070 | 0.090 | 0.120 | **0.150** | **0.190** |
| `s_rel` | 0.910 | 0.600 | 0.520 | 0.440 | **0.350** | **0.290** |
| both edges inside `r`=0.5? | no | no | no | **yes** | **yes** | **yes** |

**This is the object `both-edges-limiter-negative.md` proved unreachable for the whole
`pt3`-filter family.** Its Half-1 argument was never wrong — it was a statement about *proxy*
signals, and it correctly named the one signal class that escapes it. (W) — *does the window
LENGTH do anything beyond its opening edge?* — is finally askable.

### 2. THE HEADLINE — the release edge is NOT inert. It is where the damage is made.

One clip, **opposite signs on the two spools**, at `r = 0.5`:

| `φ_lim` (watch LP) | 0.7650 | 0.7550 | 0.7500 | 0.7450 | 0.7400 | 0.7370 | 0.7360 |
|---|---|---|---|---|---|---|---|
| `relief_lp` (watched) | +0.02955 | +0.01955 | +0.01455 | +0.00955 | +0.00455 | +0.00155 | +0.00055 |
| **`relief_hp` (unwatched)** | **−0.00023** | **−0.00798** | **−0.01132** | **−0.00954** | **−0.00274** | **−0.00041** | **−0.00008** |
| `s@min φ_hp` | 0.920 | 0.610 | 0.530 | 0.450 | 0.380 | 0.390 | 0.400 |
| `s_rel` | 0.910 | 0.600 | 0.520 | 0.440 | 0.350 | 0.290 | 0.260 |

`s_eng` = 0.030 … 0.210 — **every one upstream of `s_hp*` = 0.390.** Rung 48's law predicts a
**credit** on the HP in every row. Measured: a **debit** in every row.

**The mechanism, read straight off the trace** (`φ_lim` = 0.745, window [0.120, 0.440]):

| `s` | 0.120 | 0.200 | 0.260 | 0.340 | 0.400 | **0.440** | 0.450 | 0.500 |
|---|---|---|---|---|---|---|---|---|
| `φ_hp` limited − bare | +0.0014 | +0.0134 | **+0.0159** | +0.0108 | +0.0006 | **−0.0094** | −0.0108 | −0.0096 |
| clipped? | CLIP | CLIP | CLIP | CLIP | CLIP | **CLIP (last)** | — | — |

Inside the window the HP is **better off** — the clip really does slow its descent, exactly as
rung 48's arrest says. But it **slows it, it does not arrest it**: `φ_hp` falls right through
the whole window (0.899 → 0.853) while the bare march has already turned around at 0.390. Then
the leg lets go, the withheld fuel is delivered to a plant that is still ramping, and the
descent **re-opens**. **The unwatched spool's minimum sits ONE grid step after `s_rel`, in
6 of 6 rows.** The damage is made at the closing edge.

So the law acquires a second term:

```
   relief  =  min( truncation at s_eng ,  the re-opened dive after s_rel )  −  min_s φ_bare
                    └── rung 48's term, ≥ 0 ──┘   └── new, ≤ 0 ──┘
```

### 3. THE DISCRIMINATOR — the two edges answer to DIFFERENT clocks

The debit could be referenced to the unwatched spool's own minimum `s_hp*` (a per-spool timing
law, rung 48's structure with two edges) **or** to the ramp end `r` (rung 44's clock). At
`r = 0.5` these sit at 0.390 and 0.500 — too close to separate. **At `r = 2.0` they are 3.1×
apart** (`s_hp*` = 0.650, ramp end 2.0), and the answer is unambiguous:

| `φ_lim` (watch LP) | 0.7615 | 0.7630 | 0.7650 | 0.7670 | 0.7690 | 0.7710 | 0.7725 |
|---|---|---|---|---|---|---|---|
| `s_rel` | 0.390 | 0.510 | 0.670 | 0.840 | 1.070 | 1.450 | 2.110 |
| `s_rel / s_hp*` | 0.60 | 0.78 | **1.03** | 1.29 | 1.65 | 2.23 | 3.25 |
| `s_rel / r` | 0.20 | 0.26 | 0.34 | 0.42 | 0.54 | 0.73 | **1.05** |
| **`relief_hp`** | −0.0001 | −0.0014 | **−0.0058** | −0.0114 | −0.0186 | −0.0298 | **−0.0451** |

**The debit is 8× larger at `s_rel ≈ r` than at `s_rel ≈ s_hp*`,** and it grows monotonically
with `s_rel` right through the unwatched spool's own minimum without noticing it. **The credit
is per-spool; the debit is ramp-clocked.** `both-edges-limiter-negative.md`'s unifying fact —
*"the ramp is the only clock in the system"* — turns out to govern the closing edge too, and to
govern it *instead of* the per-spool structure that governs the opening edge.

Far side, same `r`=2.0: `s_rel/r` = 1.05 / 1.21 / 1.37 ⇒ −0.0451 / −0.0442 / −0.0438 — the
debit peaks with the release at the ramp end and decays past it. At `r`=0.5 the decay is
already complete by `s_rel/r` = 1.82 (−0.0002).

### 4. The sign FLIPS with the ramp — rung 48's regime recovered, not contradicted

Push the release well past the ramp end and the debit term vanishes, leaving rung 48's credit
alone. At **`r = 0.15`** (`s_lp*` = `s_hp*` = 0.150, releases at `s_rel` = 0.37 … 0.58, i.e.
`s_rel/r` = 2.5 … 3.9):

| `φ_lim` (watch LP) | 0.7550 | 0.7500 | 0.7450 | 0.7400 |
|---|---|---|---|---|
| `relief_lp` | +0.0666 | +0.0616 | +0.0566 | +0.0516 |
| **`relief_hp`** | **+0.0517** | **+0.0473** | **+0.0437** | **+0.0397** |

**Same instrument, same plant, same watched spool — the unwatched relief changes SIGN with the
ramp rate.** This is the two-term law predicting its own inversion, and it is why rung 48 is
*bounded* rather than *refuted*: its **credit term survives verbatim** (§ 5 reproduces its exact
zero on this instrument), and what fails is only the claim that the credit is the *whole* story.

**And the boundary is drawn honestly.** Rung 48's leg is **empirically immune** to the release
debit — `both-edges-limiter-negative.md` measured φ monotone non-decreasing from `s_eng` to the
end of the march in **32/32 cells**. *Why* it is immune this rung does **not** establish, and
the tempting explanation is measured to be wrong: **the `s_rel/r` clock does not transfer across
instrument types.** Rung 48's `m` = 0.42 releases at `s_rel/r` = 1.16 with no dive at all, while
the φ floor at `φ_lim` = 0.755 releases at `s_rel/r` = 1.20 and debits −0.008 — the same ratio,
the opposite outcome. (The obvious alternative, the size of the fuel hand-back at release, is
also **refuted**: it is *anti*-correlated at `r`=2.0 — 0.344 % ⇒ −0.045 against 0.873 % ⇒ −0.019
— and largest of all, 2.5 %, in the `r`=0.15 rows where the relief is *positive*.) **The clock
result below is a WITHIN-FAMILY result and is claimed only as one.**

### 5. Rung 48's crossing law, reproduced EXACTLY on a different instrument class

Flip the watched spool. An **HP**-watching floor engages late; the LP's minimum is early
(`s_lp*` = 0.230), so rung 48's edge condition applies to it in pure form — and the exact-zero
lands where the law says, with **no fitting and no limited march**:

| `φ_lim` (watch HP) | 0.9200 | 0.9000 | 0.8800 | 0.8700 | 0.8650 | 0.8620 |
|---|---|---|---|---|---|---|
| `s_eng` | 0.060 | 0.120 | 0.200 | 0.250 | 0.300 | 0.350 |
| vs `s_lp*` = 0.230 | up | up | up | **down** | down | down |
| `relief_lp` | +0.02192 | +0.01013 | +0.00121 | **0.000000** | **0.000000** | **0.000000** |
| predicted `min_{s≤s_eng} φ_bare − min φ_bare` | +0.02039 | +0.00852 | +0.00071 | **0** | **0** | **0** |

**A genuine forecast off a bare march landing on a limiter class rung 48 never built.** And no
debit appears on the LP here, because a release edge is structurally *late* (it needs an
accumulated window) while the **LP's basin is EARLY** — within 0.005 of its own minimum only
over `s ∈ [0.150, 0.320]`, against the HP's `[0.290, 0.500]`. Every HP-watching release lands
past the LP basin; every LP-watching release lands inside the HP one. **The early-LP /
late-HP structure that ran through rungs 46/47/48 is what decides WHICH spool is exposed to the
closing edge** — and it is the HP, exactly inverting rungs 41/44/45's "the LP eats more".

### 6. NOT rung 44's ramp-rate lever — the non-tautology gate

The deflation to exclude is "any clip removes fuel and slows the accel". Same discipline as
rung 48, same three exclusions, all measured:

- **The endpoint is unmoved.** `nu_hp` at settle = 0.959048 … 0.959060 against bare 0.959060 —
  ≤ 1.2e-5 across the whole admissible band, and at `r`=2.0, 0.959024 … 0.959064 vs 0.959064.
- **`fuel_removed` varies smoothly and stays positive** through the sign change: 0.005602 /
  0.001879 / 0.000983 / 0.000395 / 0.000095 as `relief_hp` runs −0.0002 → −0.0113 → −0.0027.
  **The largest fuel removal produces the SMALLEST debit** (0.005602 ⇒ −0.00023) — the debit is
  not "how much fuel", it is **when it is given back**.
- **One clip, two signs.** At every row of § 2 the same clip credits the LP and debits the HP.
  A ramp-rate lever cannot do that: it moves both spools the same way.

### 7. THE HONEST BOUNDARY — a floor above the running line destroys the accel

`φ_lim` must sit **below the initial running-line φ**, or the leg binds from `s = 0` and never
releases. Measured on the `hp-only` shape (FLAT LP map, LP start φ below the floors swept):
`s_eng` = 0.000 for every floor, and `nu_hp` at settle collapses to **0.7366 against bare
0.9589** — the accel does not complete. This is structurally rung 48's `m → 0` degeneracy and
is reported, not hidden: **read the crossings only where `nu_hp_end` is unmoved.**

---

## Reduce-to-prior contract (the spine)

- `surge=None` ⇒ the leg is never consulted ⇒ `integrate_fuel` is **bit-for-bit** rungs
  45/46/47/48 (and rung 43 with all legs off). Exact dispatch: the cap list is empty.
- A **dormant floor** (`φ_lim` below the whole bare march) returns `mf_sched` itself, float
  identical — no solve runs, so the trajectory is float-for-float bare.
- Each cap is solved **independently off the scheduled fuel**, so arming one leg cannot perturb
  another's bracket — rung 48's bit-for-bit composite requirement, carried verbatim.
- `lp_disabled` **asserts**: the finding is inherently two-shaft (a split BETWEEN spools).
- A **decel** never fires the leg (φ rises above the running line throughout).
- The design run `build_turbojet(…).run(…)` is untouched — bit-for-bit rung 6.

## Verification gates (`tests/test_rung49.py`)

1. `surge=None` ⇒ bit-for-bit rungs 43/46/47/48 on `(nu_lp, nu_hp, phi_lp, phi_hp, Tt4, f, mf)`.
2. A dormant floor is float-for-float the bare march.
3. **The HOLD** — `φ_watched == φ_lim` to 1e-9 at every engaged point (no chatter), and the
   watched relief equals `φ_lim − min φ_bare` identically. *Stated as an IDENTITY check, not a
   finding: it is definitional under a working set-point solve and gates only the solver.*
4. **Both edges inside the ramp** — exhibit `0 < s_eng < s_rel < r`. (The unreachable object.)
5. **THE HEADLINE** — LP-watching at `r`=0.5: `relief_lp > 0` **and** `relief_hp < 0` from the
   same clip, at every `φ_lim` in the admissible band.
6. **The mechanism** — the unwatched spool's `argmin φ` sits within 3 grid cells **after**
   `s_rel`, and inside the window the unwatched φ is **above** bare.
7. **The sign flip** — at `r`=0.15 (release ≫ ramp) `relief_hp > 0`: rung 48's regime recovered.
8. **THE DISCRIMINATOR** — at `r`=2.0, `|relief_hp|` at `s_rel≈r` exceeds that at `s_rel≈s_hp*`
   by > 5×, and `|relief_hp|` is monotone in `s_rel` straight through `s_hp*`.
9. **Cross-instrument confirmation** — HP-watching: `relief_lp` is **exactly 0.0** once
   `s_eng > s_lp*` and strictly positive before it, and matches the bare-march forecast
   `min_{s≤s_eng} φ_bare − min φ_bare` to **2e-3 absolute** where nonzero. *(Absolute, not
   relative: the forecast is O(ds)-accurate, so near the crossing both sides → 0 and the
   RELATIVE error necessarily blows up — `both-edges-limiter-negative.md` § "the relative
   error blows up near the crossing". The exact-zero side is gated exactly.)*
9b. **The exposed spool is the LATE one** — the LP basin (within 0.005 of its own min) is
   `[0.15, 0.32]` against the HP's `[0.29, 0.50]`; every HP-watching release lands past the LP
   basin (no debit), every LP-watching release inside the HP one (debit).
10. **Non-tautology** — `fuel_removed` positive and monotone in the floor, `nu_hp_end`
    unmoved to 5e-4 (at a full `s_settle`=4 settle), and the largest `fuel_removed` gives a
    **smaller** `|relief_hp|` than a middling one.
11. **The honest boundary** — the FLAT-LP corner binds at `s_eng == 0` and moves `nu_hp_end`
    by > 0.2 (gated as a degeneracy, not read as a crossing).
12. **Robustness** — the debit survives `ds` ∈ {0.02, 0.01} (change < 25 %) and
    `ρ` ∈ {0.25, 4} (sign of the split unmoved). `ds` = 0.005 measured in the anchor
    (−0.01174 against −0.01132 at 0.01, −0.01040 at 0.02), not gated — cost.
13. The design run is bit-for-bit rung 6.

## Concessions

- **`φ_lim` inherits rung 36/41's imposed `φ_surge`.** Magnitudes disclaimed; signs, ordering,
  crossing and convergence claimed. A real surge line would move every number here.
- **WHY rung 48's leg is immune to the release debit is an OPEN SEAM.** It is immune (32/32
  monotone cells, that doc's own measurement), and this rung does not explain it: the ratio that
  governs the debit *within* the φ-floor family (`s_rel/r`) does **not** carry across instrument
  types, and the hand-back magnitude is refuted outright (§ 4). Something about the *shape* of
  the clip — rung 48's cap rises with `pt3` and so keeps its deficit at 0.3–0.7 %, where the φ
  floor's grows to 2.3 % — is the obvious suspect, but it is **not measured here**.
- **The release is a hard min-select hand-back** — a real limiter has actuator dynamics
  (rung 47's `τ_gov` on the *other* leg). A lag on the release edge would smear the dive; the
  debit's *existence* rests on the fuel deficit being returned to a still-ramping plant, which a
  lag postpones but does not remove. **Not measured here — the first open seam.**
- **`φ` is the map's flow coefficient, not a measured stall margin.** The limiter therefore
  "knows" the protected variable perfectly — no sensor model, no noise. That is the *best case*
  for this instrument class, so the debit is a **lower bound** on the real cost.
- The debit is quantified against `s_rel/r`; the **functional form** of that dependence is not
  derived, only measured (peak at ≈1, decay past it).
- The plant is rung 43's non-equilibrium (CPG/TPG) gas — rung 35's standing concession.

## Anchor

`docs/plans/rung49-anchor-phi-limiter.md` — the probe transcripts and the verified numbers.
