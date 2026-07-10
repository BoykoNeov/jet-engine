# Rung 20 — Super-equilibrium O *through the quench*: lifting the finite-quench lower bound

Rung 19 lifted the equilibrium-O lower bound only on the **primary** diagnostic (`ei_no`/`x_no_mix`).
Every finite-quench field — `ei_no_quenched` (rung 11 bulk), the rung-12 core, `ei_no_pocket_quench`
(rung 16) — and every rung-17 exhaust-clamp margin `a` still **re-made** NO on the rung-6 **equilibrium**
`[O]`, so all of them were still **lower bounds**. Rung 20 threads the same Westenberg `m(T)` lift
**inside** the `_quench_no` re-making, closing that seam. It adds **no new species** and touches **no
cycle path** — a decoupled diagnostic, so the cycle stays **bit-for-bit rung 6**.

> **Read `docs/rung19-spec.md` (the super-eq-O multiplier and its two concessions),
> `docs/rung10-spec.md` (the finite quench / the dropped clamp) and `docs/rung17-spec.md` (the
> exhaust-NO clamp ladder) first.** This file states only what *changes*, and the numbers-before-code
> instrumentation lives in `docs/plans/rung20-anchor-super-eq-through-quench.md`.

---

## The load-bearing result: the lift is MODEST, PEAK-CONCENTRATED, and *smaller* than the primary's

The intuitive story — "`m(T)` grows as `T` falls, so the super-eq lift bites hardest on the slow,
cooling, lingering pocket" — is **wrong**, and the instrumentation (below) inverts it. Three effects
fight, and the temperature dependence decides:

1. The Zeldovich re-making is dominated by the **hottest stoich crossing** (`k1f` carries a ~38 000 K
   activation exp — the whole reason thermal NO is a *peak* phenomenon). On the quench that peak is the
   **hottest** point (`T_peak≈2448 K` at the rung-17 design point).
2. `m(T)` is **decreasing in T** (rung 19, gate 3), so `m` is at its **minimum** (`≈1.14`) *exactly at
   the peak where the NO mass is made*. The cool tail carries large `m` (`→1.5+`) but **negligible
   formation** (`R1∝[O]` there is ~100× smaller).
3. On the cool tail NO is super-equilibrium (`a=[NO]/[NO]_e>1`), so the rate factor `(1−a²)<0` — the
   chemistry is **destroying** NO, and lifting `[O]` scales `R1` **and** `R2` alike, so it does *not*
   run away.

Net: the effective lift on `ei_no_quenched` is a **formation-weighted average of a T-decreasing `m`**,
hence **floored by `m(T_peak)`** and **modest**. Measured at the design point:

| quantity | eq-O | super-eq-O | factor |
|---|---|---|---|
| `ei_no_quenched` (bulk) | 0.2712 | 0.3176 g/kg | **×1.171** |
| `ei_no_pocket_quench` (rung 16) | 0.917 | 1.067 g/kg | **×1.164** |
| primary `ei_no` (rung 19, ideal quench) | — | — | **×1.276** = `m(T_p=2110)` |
| `m(T_peak=2448 K)` | — | — | 1.142 |

The quench lift (**×1.17**) is *smaller* than the **primary** lift (**×1.28**) — because the quench
re-making samples an even **hotter** peak than the primary flame, and `m` is smallest where it is
hottest. **Threading the lower-bound lift through the quench gives *less* lift, not more.** That is the
rung-20 surprise, and it is the honest correction of the naive headline (surfaced per the working
contract's "stop and explain surprises").

## The certified spine: the rung-17 `a` margins rise, the denominator does not

Rung 17's caveat — "every margin `a` is built on equilibrium O, hence a lower bound" — is now
**discharged**. The rung-17 margins `a=[NO]/[NO]_e(T9)`:

| margin | eq-O | super-eq-O |
|---|---|---|
| `a_mixed_out` (rung 8) | 0.0158 | 0.0202 (×1.28, the primary lift — stays **≪1**, dormant) |
| `a_bulk_quench` (rung 11) | 3.27 | 3.83 (×1.17) |
| `a_pocket` (rung 16) | 11.06 | 12.87 (×1.16) |
| denominator `x_no_e_exit` | 2.1147e-06 | **2.1147e-06 (bit-identical)** |

The numerators (kinetic re-made NO) lift; the **denominator** `x_no_e(T9)=Kp_NO·√(x_N2·x_O2)` is a
**thermodynamic ceiling**, set by the a6/a7 thermo, **not by the O-atom closure** — so it is
**untouched**. Every margin **rises by a bounded factor**, and the rung-17 ordering
`a_mixed<a_bulk<a_pocket` and `a_mixed<1` (mixing hides the NO at the rich primary) **survive**. This is
the clean, defensible payoff: the rung-17 lower bounds are lifted by a *bounded, computed* factor.

## The clamp still does NOT fire at the burner

The per-pocket `max_a_quench` rises with the lift (`0.72→0.81`) but stays **<1** — the clamp remains
**dormant at station 4**. Super-equilibrium O speeds **formation** but does not raise the **ceiling**
`[NO]_e` (a thermodynamic quantity), so it is **not** the lever for the "per-pocket clamp fires AT the
burner" seam. That lever is a **slow-enough freeze on a cooling pocket** (a long dwell that freezes NO
high while the local `[NO]_e` collapses), which stays a separate deferred seam.

---

## The equations — a diagnostic layer, no station changes

Every cycle station is **bit-for-bit rung 6**. Inside `_quench_no`'s Zeldovich re-making, at each
trajectory point `(T, [O], …)`:

```
super-eq O:   m(T) = (C2/C1)·T·exp((θ1−θ2)/T)          (rung-19 Westenberg ratio; m≡1 ⇒ rung 10/11)
              [O] → m(max(T, T_floor))·[O]              (lifts R1 AND R2 alike, as _thermal_no does)
              T_floor = 1500 K                          (the flame-band floor — m diverges as T→0)
prompt:       EI_quenched_total = ei_no_quenched + ei_no_prompt   (prompt is per-kg-fuel INVARIANT)
guard:        1 ≤ m(T) ≤ 2 at every trajectory T        (standing assert, mirrors the K-check/trace)
```

**The T-floor is load-bearing.** `m(T)=A·T·exp(B/T)` with `B=θ1−θ2≈3967 K` **diverges** as `T→0`
(`m(1200 K)≈3.0`, past the `[1,2]` bound); the quench cools toward `T_mix≈Tt4`, so a bare lift would
inject an out-of-band multiplier at a colder design point. The Westenberg partial-eq closure is a
**flame model** (`T≳1500 K`) anyway, so `m` is frozen at `m(max(T, 1500 K))` below the band — which also
keeps the standing `1≤m≤2` trajectory assert honest.

**Prompt through the quench is chosen, not trivial.** `EI_prompt` is per-kg-fuel and prompt is a
flame-front phenomenon set at the primary, so dilution lowers its mole fraction but **not** its emission
index — it rides the quench **unchanged**, exposed as `ei_no_quenched_total = ei_no_quenched +
ei_no_prompt`. We **add** it here rather than inject prompt moles into `_quench_no`'s cooling chemistry
**on purpose**: prompt's **magnitude is imposed** (rung-19 concession), so running an un-certified number
through Zeldovich destruction would be false precision. That is also why prompt is kept **out** of the
rung-17 clamp `a` (which stays a certified-*thermal* margin).

## What is certified vs carried

- **Certified:** the reduce (`super_eq_o=False` ⇒ bit-for-bit, a defaulted kwarg gating a single
  multiply); the **modest, peak-concentrated** lift (`≈m(T_peak)`, floored, `< primary`); the **spine**
  (`a` rises because the numerator lifts while the thermodynamic denominator does not); the **bound**
  (`1≤m≤2` via the floor).
- **Carried from rung 19 (still imposed/semi-empirical):** the super-eq **ratio** `m(T)` is Westenberg's
  fitted partial-eq/eq ratio — a full-equilibrium pool cannot self-yield super-eq O — so the lifted `a`
  is a **better-justified but still not pinned** magnitude. The **prompt magnitude** stays imposed
  (kept out of `a`).

## Scope — deliberately narrow (per-field internal consistency)

Rung 20 lifts **everything that flows through `_quench_no`**: the mean-field bulk (`ei_no_quenched`), the
rung-12 core (`ei_no_unmixed`/`ei_no_core`), the rung-16 per-pocket (`ei_no_pocket_quench`, including
each pocket's initial `_thermal_no` and its lean/tail `_ideal_bell_ei`), and the rung-17 numerators. The
**ideal-bell composition integrals** — `ei_no_pdf` (rung 13), `ei_no_pdf_quench` **term 2** (rung 15),
`ei_no_transported` (rung 18) — **deliberately stay equilibrium-O lower bounds**: combining `super_eq_o`
with `pdf`/`pdf_quench`/`transported` is **forbidden** (an explicit `zoned_nox` guard), because
`ei_no_pdf_quench` would otherwise become a half-lifted **hybrid** (lifted term 1 + eq-O term 2). Since
rung 16 already supersedes rung 15's linearised dwell, `{bulk, core, per-pocket, clamp}` is the honest
cut. `super_eq_o` **combines** with `mixing` / `unmixedness` / `pocket_quench`.

## Design (additive, mirrors rungs 10–19)
- `_quench_no(…, super_eq_o=False)` lifts `[O]` by `m(max(T, T_floor))` inside `dn_dt`; `False` ⇒
  byte-identical rung 10/11/12/16.
- `_ideal_bell_ei(…, super_eq_o=False)` and `_pocket_quench_mean_ei(…, super_eq_o=False)` thread the
  flag so the per-pocket β-PDF integral is internally consistent (no eq-O pocket in a lifted mean).
- `zoned_nox(…, super_eq_o=…)` passes it into the bulk / core / per-pocket quench calls and **forbids**
  the three ideal-bell closures. `ZonedNOxState.ei_no_quenched_total` adds the invariant prompt.
- `exhaust_no_clamp(…, super_eq_o=False)` threads it into all three numerator `zoned_nox` calls; the
  common denominator is untouched.

## Verification gates (priority order) — `tests/test_rung20.py`
1. **Reduce (load-bearing).** `super_eq_o=False` ⇒ bit-for-bit the prior rung (bulk / per-pocket / core
   / clamp identical; a direct `_quench_no` reduce). Rungs 1–19 suites green, untouched; cycle
   bit-for-bit rung 6.
2. **Modest, peak-concentrated lift.** bulk lift `∈(1.10,1.25)`, `≥ m(T_peak)`, and **strictly `<`** the
   primary lift at the same `φ_p` (`T_peak > T_primary`).
3. **Clamp dormant.** `max_a_quench<1` with the lift (super-eq O is not the burner-clamp lever).
4. **The spine.** `a_bulk`/`a_pocket` **rise**; `x_no_e_exit` **bit-identical**; ordering + `a_mixed<1`
   survive; `hides_super_eq`/`ladder_monotone` hold.
5. **Prompt-through invariance.** `ei_no_quenched_total = ei_no_quenched + ei_no_prompt`; ideal quench ⇒
   `None`.
6. **Forbid guard.** `super_eq_o` + `{pdf, pdf_quench, transported}` raises; combines OK with
   `mixing`/`unmixedness`/`pocket_quench`.
7. **The floor.** raw `m(1200 K)>2`; floored `m(1500 K)<2` — the standing `1≤m≤2` trajectory assert holds.

## Done when
Reduce reproduces rungs 1–19 to the digit (existing suites untouched, green) and the cycle is
bit-for-bit rung 6; gates 2–7 hold. `main.py` gains a rung-20 panel (the effective-lift number, the
peak-concentration, the `a`-margins rising with the untouched denominator, and the honest scope).
`CLAUDE.md` scope + rung table + deferred seams updated (super-eq O through the quench **done**; the
finite-quench prompt is invariant-added; the "clamp fires at the burner" and the ideal-bell-PDF lifts
stay the next seams).
