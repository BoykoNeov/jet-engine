# Rung 27 anchor — NO freeze-out: is the frozen-NO ASSUMPTION earned? (numbers before code)

> **Verified anchor data** for rung 27 (a `T,p`-dependent NO clock resolving whether — and where —
> *exhaust NO* freezes, against rung-26's recombination clock). The feasibility **probe** is
> `M:\claud_projects\temp\rung27-anchor\da_no_profile2.py` (outside git — project temp policy),
> reusing the shipped rung-6/7/8 machinery (`_equilibrium_composition`, `_k_zeldovich`,
> `_equilibrium_no_fraction`, `zoned_nox`, `build_turbojet`) read-only. Design point: the standard
> real-loss cycle (`π_d=0.97, η_c=0.88, η_b=0.99, π_b=0.96, η_t=0.90, η_m=0.99, π_n=0.98`, `M0=0.85`,
> `π_c=10`), swept over `Tt4`. **No rung code written yet — this file is the go/no-go.**

## The assumption under test

Every NO number since rung 7 **assumes** the station-4 mixture and its NO freeze through the nozzle.
Rung 14/17's clamp corollary reads `max_a = x_no_frozen/x_no_e(T9) ≫ 1` **off that assumption** — the
numerator is frozen *by fiat*. Rung 26 then anchored a recombination clock and found the **major** pool
does **not** freeze at entry: `s_freeze` walks 0.12→0.38 as `Tt4` climbs. So the assumption is now
visibly unearned for the majors. **Does it leak for NO?** Rung 7's folklore says no ("Zeldovich is
slow"), but folklore is not a derivation.

## 1. The clock is anchored — rung 7's own constants, zero new

The NO clock uses the **extended-Zeldovich rate constants already at `gas.py:750`** (Hanson &
Salimian 1984, as tabulated in Turns) — already **K-checked against the project's own `a6/a7` thermo**
by `_kcheck_ratio` (detailed balance ties the transcribed rates to the thermo; measured 1.035–1.044).
**Zero new constants**, exactly as rung 26 (which reused GRI-Mech 3.0).

| Reaction | A | n | θ (K) |
|---|---|---|---|
| `2r: NO + O → N + O2` | 3.8e9 | 1.0 | **20820** |
| `3r: NO + H → N + OH` | 1.7e14 | 0.0 | **24560** |

Contrast rung 26's clock: `Ea = 0.00` for **every** recombination sink. **These have a large thermal
barrier.** That contrast is the rung.

## 2. The clock — and why it is `[NO]`-independent

From the rung-7/10 reverse-rate form `d[NO]/dt = 2R1(1−a²)/(1+βa)`, `a=[NO]/[NO]_e`,
`β=R1/(R2+R3)`, the exact local linearised relaxation time is

```
  τ_NO = −1/(df/dc) = [NO]_e (1+βa)² / ( 2 R1 (2a + βa² + β) )
```

In the **super-equilibrium** limit the clamp cares about (`a ≫ 1`) this collapses to

```
  τ_NO → 1 / ( 2 ( k2r[O] + k3r[H] ) ) = 1 / ( 2 c_tot ( k2r x_O + k3r x_H ) )
```

**independent of `[NO]_e` AND of `a`** (the `[NO]_e` inside `R2/R3` cancels against the `a` in the
numerator). Two consequences, both load-bearing:

- **The freeze answer does not depend on which `x_no_frozen` we feed it** — a robustness property
  rung 26's clock does not have (its `x_OH` is the whole radical pool).
- **The kill test is clean**: only two factors, `k(T)` and `c_tot`, no `[NO]_e` anywhere — exactly
  parallel to rung 26's `k(T)` vs `c_tot²`.

**The clock depends on `[O]` and `[H]` — the very radicals rung 26's clock destroys.** The two rungs
interlock; §6 records that as the coupled refinement, not a claim here.

**Bound.** Evaluated on the **frozen station-4 pool**: radical-RICH (recombination has not yet removed
`[O],[H]`) and `τ_NO ∝ 1/([O],[H])` ⇒ the **fastest possible** NO relaxation ⇒ `Da_NO` **UPPER** bound.
If NO freezes here it freezes *a fortiori* on the real radical-depleting path. Same bounding logic
rung 26's anchor used for `x_OH`.

## 3. The gate — does exhaust NO freeze? **YES, from entry, at every `Tt4`**

`Da_NO = τ_res/τ_NO` with rung-26's `τ_res = L/(0.6·V9)`, `L=0.5 m`. `Da_recomb` is rung-26's clock on
the same probe (it **reproduces rung-26's anchor table entry values exactly** — 0.31 / 0.735 / 1.446 /
2.846 / 4.501 — an independent cross-check that this probe's cycle scaffolding matches):

| `Tt4` | `Da_NO` entry → exit | ever ≥ 1? | `Da_recomb` entry | **separation** (entry) |
|---|---|---|---|---|
| 1500 | 8.285e-09 → 2.158e-13 | **never** | 0.310 | 3.74e7 |
| 1650 | 3.126e-07 → 1.207e-11 | **never** | 0.735 | 2.35e6 |
| 1800 | 5.994e-06 → 3.354e-10 | **never** | 1.446 | 2.41e5 |
| 2000 | 1.463e-04 → 1.257e-08 | **never** | 2.846 | 1.95e4 |
| 2200 | 2.011e-03 → 2.392e-07 | **never** | 4.501 | 2.24e3 |

**`Da_NO ≪ 1` everywhere, by 3–9 orders of magnitude, at every `Tt4`.** The frozen-NO assumption is
**earned** — and it is earned on the *upper* bound, so the real flow freezes harder still.

**Robust to the clock's regime.** `a` is **not** uniformly ≫1: it runs ≈0.3 (entry, sub-equilibrium)
→ 10–35 (exit) — NO crosses into super-equilibrium *mid-nozzle*, as `x_no_e(T)` collapses (which is
precisely rung 14's clamp firing). So neither asymptotic limit is uniformly valid. **It does not
matter**: the exact local linearisation (probe v1) and the `a≫1` asymptote (v2) agree within **~3×**
at `Tt4=2200` (entry `6.0e-4` vs `2.0e-3`) — against a margin of 3–9 **orders**. The freeze survives
the clock definition, the NO level, and the regime.

## 4. The kill test — the two terms **AGREE**; this **INVERTS rung 26**

On the standalone clock with the frozen pool pinned (a **mechanism** certification, not the marched
path — rung 26's precedent exactly). `τ_NO = 1/(2 c_tot(k2r(T) x_O + k3r(T) x_H))`:

| `Tt4` | kill T (`k` pinned ⇒ density alone) | kill p (`c_tot` pinned ⇒ `T` alone) | verdict |
|---|---|---|---|
| 1500 | τ ×4.11 — **drives** | τ ×9.34e3 — **drives** | AGREE |
| 1800 | τ ×4.66 — **drives** | τ ×3.83e3 — **drives** | AGREE |
| 2200 | τ ×5.21 — **drives** | τ ×1.61e3 — **drives** | AGREE |

**Both legs drive freezing, at every `Tt4` — and Arrhenius dominates density by ~300–2000×.**

**The inversion, stated precisely:**

|  | rung 26 (recombination) | **rung 27 (NO)** |
|---|---|---|
| barrier | `Ea = 0` ⇒ `k` **accelerates** on cooling | `θ ≈ 20820 / 24560 K` ⇒ `k` **craters** |
| molecularity | **ter**molecular ⇒ `c_tot²` | **bi**molecular ⇒ `c_tot¹` |
| the two terms | **OPPOSE** — density wins *despite* `T` | **AGREE** — both drive |
| kill p | Da **rises** 4.50→12.1 (no freeze) | Da **falls** ×1.6e3 (freezes) |
| ⇒ | freezes mid-nozzle, point **moves** | freezes **at entry**, ~10³–10⁹ margin |

**Same nozzle, two anchored clocks, opposite mechanism structure.** Rung 26 refuted the
"unanchored-Arrhenius trap" framing by having *no* Arrhenius term; rung 27 has a *large* one and
freezes for that reason — the density term is along for the ride, and structurally weaker (`c¹`, not
`c²`) besides.

## 5. What is NOT claimed — no moving freeze point

**Rung 26's headline has no analogue here, and we do not manufacture one.** `s_freeze_NO = 0` at every
`Tt4`; nothing moves. What *is* honest and quantified is the **margin trend**: because Zeldovich is
steeply Arrhenius and recombination is `Ea=0`, the separation between the two clocks **narrows
steeply** with `Tt4` — `Da_NO` entry climbs ×2.4e5 across the band while `Da_recomb` entry climbs only
×15, so the entry separation collapses **3.74e7 → 2.24e3** (monotone, ~10× per 200 K). **No crossing
within any physical `Tt4`** — extrapolating one would be an overclaim.

Also not claimed: the **location** (rides on the same `L≈0.5 m` as rung 26 — but here the margin is so
large that no O(1) `L` touches the answer, which is *stronger* than rung 26's disclaimer, not weaker);
the single-representative-reaction surrogate (`2r/3r` only; the full network is not summed).

## 6. The reduce gate (name it before coding — advisor)

Mirror rung 26 exactly:

- **`freeze_out_nozzle` (rung 26) stays literally untouched** — a *new* method beside it.
- **The reduce:** NO relaxation off (`Da_NO ≡ 0`, injected at the `Da_local` level as a literal
  constant) ⇒ marched exit NO **== entry NO == rung-14's `x_no_frozen`, bit-for-bit** ⇒
  `max_a` reproduces rung 14/17's number to the ULP. That is the tripwire.
- The **rung-26-coupled march** (NO riding the relaxing pool, so `[O],[H]` deplete under it) is a
  **secondary refinement**, not the load-bearing claim: it can only *slow* NO further (radical-poorer
  ⇒ larger `τ_NO`), so it moves the answer **deeper into frozen**.

## 7. Verdict — GO

All three of the advisor's stop conditions cleared: `Da_NO` never approaches 1 (3–9 orders clear at
every `Tt4`); **neither** kill leg un-freezes; the margin trend is real and quantified. The rung's
claims are: **(1)** the frozen-NO assumption carried since rung 7 is **derived**, on an upper bound,
robust to clock/level/regime; **(2)** the kill test **inverts rung 26's** — two anchored clocks in the
same nozzle, freezing for structurally opposite reasons; **(3)** the margin **narrows** with `Tt4`,
quantified, with no crossing claimed.

## Sources

- Hanson & Salimian (1984) extended-Zeldovich rate constants, as tabulated in Turns — the same source
  `gas.py:746` already cites, already K-checked in-code against the project's `a6/a7` thermo.
- GRI-Mech 3.0 (`grimech30.dat`) — rung-26's recombination clock, reproduced here for the side-by-side.
