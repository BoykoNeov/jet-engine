# Rung 21 — Super-equilibrium O through the IDEAL-BELL PDF integrals: discharging the last eq-O seam

Rung 20 threaded the Westenberg `m(T)` super-eq-O lift through **everything that flows through
`_quench_no`** (the mean-field bulk `ei_no_quenched`, the rung-12 core, the rung-16 per-pocket, the
rung-17 clamp numerators), but **deliberately left** the three **ideal-bell composition integrals** on
equilibrium O and **forbade** combining them with `super_eq_o`:

- `ei_no_pdf` (rung 13 — the β-PDF over the ideal bell),
- `ei_no_pdf_quench` **term 2** (rung 15 — the same integral × a scalar dwell factor),
- `ei_no_transported` (rung 18 — the same integral at a transported width).

The stated reason (rung 20 §Scope): `ei_no_pdf_quench = term1 + term2` would become a **half-lifted
hybrid** — term1 (`ei_no_quenched`) lifted by rung 20, term2 stuck on eq-O. Rung 21 **discharges that
seam**: it threads the *same* `m(T)` through the ideal bell so **both terms carry the lift**, the hybrid
objection dissolves, and `super_eq_o` combines with **every** closure. It adds **no new species/config**
and touches **no cycle path** — a decoupled diagnostic, so the cycle stays **bit-for-bit rung 6**.

> **Read `docs/rung19-spec.md` (the super-eq-O multiplier `m(T)` and its two concessions),
> `docs/rung20-spec.md` (the same lift through the finite quench — the inversion this generalizes),
> and `docs/rung13-spec.md` (the ideal-bell β-PDF) first.** This file states only what *changes*; the
> numbers-before-code instrumentation lives in `docs/plans/rung21-anchor-ideal-bell-lift.md`.

---

## The load-bearing result: the ideal-bell lift is ≈×1.15 — the rung-20 inversion, generalized

The naive story — "the ideal bell spans deep-lean pockets where flames are cool and `m(T)` is large
(→1.9), so the composition-PDF lift should be *bigger* than the primary's ×1.28" — is **wrong**, and
the instrumentation (measured before this lesson was written) **inverts** it exactly as rung 20 did on
the cooling trajectory. The reason is identical: **NO forms where it is hottest, and `m` is smallest
where it is hottest.**

The ideal bell EI(φ) is sharply **peaked near stoich** (φ≈0.97, T_p≈2424 K), which is the whole rung-13
lesson (segregation *raises* the mean because NO is peaked there). At that peak `m(T_p)≈1.12` — its
**floor**. The lean and rich flanks carry large `m` (→1.5) but orders-of-magnitude less EI. Because the
β-PDF integral is **EI-weighted**, it is dominated by the peak, so:

| reference | lift | why |
|---|---|---|
| deep-lean **point value** (g→0 at ξ̄, φ≈0.40) | **×1.91** | cool lean flame ⇒ `m` large, but EI≈1.3e-5 (negligible) |
| **⟨EI⟩_pdf** (g>0 — the number that matters) | **×1.14–1.17** | EI-weighted onto the near-stoich peak where `m` is minimal |
| rung-19 **primary** (φ_p=1.5) | ×1.28 | `m(2110 K)` — an off-peak point, *cooler* than the bell peak |
| rung-20 **quench** (bulk/pocket) | ×1.17 / ×1.16 | the same peak-concentration on the cooling path |

**The ideal-bell PDF lift (×1.15) sits right alongside the quench lift and BELOW the primary lift** —
because the bell peak (~2424 K) is *hotter* than the φ_p=1.5 flame (2110 K), and `m` is smallest where
hottest. **And it DECREASES with segregation** (×1.173 at g=0.005 → ×1.144 at g=0.30): heavier variance
pulls more PDF mass onto the peak, where the lift is smallest. That measured, slightly counter-intuitive
monotonicity **is** the rung-21 result (surfaced per the working contract's "stop and explain
surprises"). It is the third time this lift has come in *smaller than the intuition* (primary rung 19,
quench rung 20, composition PDF rung 21).

## The hybrid, resolved (the point of the rung)

At J=36 (an over-penetration flank), in the real `zoned_nox` code path:

| field | eq-O | super-eq-O | lift |
|---|---|---|---|
| `ei_no_pdf` (rung 13) | 0.5871 | 0.6717 | ×1.144 |
| `ei_no_pdf_quench` (rung 15 composite) | 1.9838 | 2.2931 | **×1.156** |
| `ei_no_transported` (rung 18) | 0.9939 | 1.1482 | ×1.155 |
| `ei_no_quenched` (rung-20 bulk term1) | 0.8993 | 1.0524 | ×1.170 |

The `pdf_quench` **composite** lift (×1.156) sits **between** its term1 (bulk ×1.17) and term2
(ideal-bell ×1.14) — the *measured* proof that both terms now carry `m(T)`, so nothing is half-lifted.
That is the whole payoff: rung 20's explicit objection is **removed by construction**, not by a caveat.

## Shape is preserved — this is a consistency lift, not a new feature

Rung 21 multiplies the whole rung-13/15/18 family by a peak-concentrated ≈×1.15. It does **NOT** change:
the optimum **LOCATION** (still pinned AT `C_opt`, where `g→0` ⇒ the lift acts on a delta), the `(H/S)²`
shift, the humped ⟨EI⟩(g), or the stoich-mean **sign reversal**. Both the eq-O and super-eq-O J-sweeps
have their minimum AT `J_opt`. The *lesson* is the **measured magnitude and its inversion**, not a
relocated feature.

---

## The equations — a diagnostic layer, no station changes

Every cycle station is **bit-for-bit rung 6**. Inside the ideal bell, at each sampled mixture fraction
ξ ⇒ local `far=ξ/(1−ξ)` ⇒ flame `T_p`:

```
super-eq O:   m(T_p) = (C2/C1)·T_p·exp((θ1−θ2)/T_p)     (rung-19 Westenberg ratio; m≡1 ⇒ rung 13/15/18)
              [O] → m(max(T_p, T_floor))·[O]             (lifts R1 AND R2 alike — as _thermal_no does)
              T_floor = 1500 K                           (the flame-band floor — m diverges as T→0)
⟨EI⟩_pdf  =   ∫ EI_bell(φ(ξ); m)·P_β(ξ; ξ̄, g) dξ         (rung-13 integral over the LIFTED bell)
g→0 limit:    ⟨EI⟩ → EI_bell(φ̄; m)                       (delta ⇒ super-eq-O well-mixed point value)
```

The T-floor and the `1≤m≤2` band are the same as rung 20 (the bell samples flames down to the lean
flammability edge, so `m` can climb toward the band ceiling on the flank — but those flank pockets carry
negligible EI). `_ideal_bell_ei` already asserts nothing new is needed: the per-pocket `_thermal_no`
K-check/trace guard is intrinsic, and a ≈×1.15 lift on a trace base stays trace.

## What is certified vs carried

- **Certified:** the reduce (`super_eq_o=False` ⇒ bit-for-bit rungs 13/15/18 — a defaulted kwarg gating
  a single multiply); the **modest, peak-concentrated** lift (`≈×1.15`, `< primary ×1.28`, decreasing in
  `g`); the **hybrid resolution** (both `pdf_quench` terms lift, the composite sits between them); the
  **shape preservation** (optimum LOCATION / `(H/S)²` shift / sign reversal unchanged); the `g→0`
  consistency (lifted delta = the super-eq-O point value).
- **Carried from rungs 19–20 (still semi-empirical/imposed):** the super-eq **ratio** `m(T)` is
  Westenberg's fitted partial-eq/eq ratio (a full-equilibrium pool cannot self-yield super-eq O), so the
  lifted integrals are **better-justified but not pinned** magnitudes. The **prompt** term stays a
  **primary-only invariant EI** — it is NOT injected into the composition integrals (its magnitude is
  imposed; the rung-20 rationale).

## Scope — the last eq-O seam on the primary/PDF side, closed

Rung 21 lifts the three **ideal-bell composition integrals** and removes the rung-20 forbid guard, so
`super_eq_o` now combines with **every** closure (`mixing`/`unmixedness`/`pdf`/`pdf_quench`/
`pocket_quench`/`transported`). The **two remaining deferred seams are untouched**: the "per-pocket
clamp fires AT the burner" **slow-freeze** lever (super-eq O is not it — rung 20 showed `max_a<1` at
station 4; the lever is a Damköhler-limited NO freeze on a cooling pocket) and the **spatial /
transported-CFD PDF** (rung 18 proved a 0-D transport cannot derive the `C_opt` optimum).

## Design (additive, mirrors rungs 19–20)
- `_bell_interpolator(…, super_eq_o=False)` lifts every bell node's `[O]` by `m(T_p)` via the
  already-existing `_ideal_bell_ei(…, super_eq_o=…)`; `False` ⇒ byte-identical rung 13.
- `_pdf_mean_ei(…, super_eq_o=False)` threads it through **both** the `g→0` delta short-circuit and the
  built bell (so the reduce stays consistent in the limit).
- `zoned_nox(…, super_eq_o=…)` passes it into the three `_pdf_mean_ei` calls (`pdf`, `pdf_quench`
  term 2, `transported`) and the **forbid guard is removed** (replaced by an explanatory comment).
- **For `transported`, only the bell EI is lifted — NOT the width `g`.** The transported segregation
  `g(C)` is a *mixing* quantity (the variance-decay ODE residual from the derived two-stream ceiling);
  super-equilibrium O is an **[O]-atom** closure on the *chemistry*, physically independent of the
  mixing rate. So `_two_stream_ceiling`/`_transport_variance` are untouched — the lift acts purely on
  the EI the PDF integrates, exactly as for `pdf`/`pdf_quench`.
- `exhaust_no_clamp` is unaffected (it does not use the ideal-bell integrals).

## Verification gates (priority order) — `tests/test_rung21.py`
1. **Reduce (load-bearing).** `super_eq_o=False` ⇒ `ei_no_pdf`/`ei_no_pdf_quench`/`ei_no_transported`
   bit-for-bit the prior rung (a defaulted kwarg). Rungs 1–20 suites green (the rung-20 forbid gate is
   **consciously inverted** — see §note); cycle bit-for-bit rung 6.
2. **Modest, peak-concentrated lift.** `ei_no_pdf` lift `∈(1.10,1.20)`, **strictly `<`** the primary
   lift (×1.28) at the same `φ_p`, and **`<`** the deep-lean point-value lift (×1.9).
3. **Lift decreases with segregation.** `⟨EI⟩_pdf` lift at large `g` `<` the lift at small `g` (more
   variance ⇒ more stoich-peak weighting ⇒ smaller fractional lift).
4. **The hybrid resolved.** `ei_no_pdf_quench` composite lift `∈(1.10,1.20)` and **between** its term1
   (bulk) and term2 (ideal-bell) lifts; combining `super_eq_o` with `pdf`/`pdf_quench`/`transported`
   **does not raise**.
5. **`g→0` consistency.** lifted `ei_no_pdf` at `C_opt` (g=0) == `_ideal_bell_ei(far, …, super_eq_o=True)`
   (the super-eq-O well-mixed point value).
6. **Shape preserved.** eq-O and super-eq-O J-sweeps both have their minimum AT `J_opt`; the optimum
   LOCATION / `(H/S)²` shift are unmoved by the lift.

## Note — the consciously inverted rung-20 gate
`tests/test_rung20.py::test_forbid_super_eq_o_with_ideal_bell_closures` asserted that `super_eq_o` +
`{pdf, pdf_quench, transported}` **raises**. Rung 21 makes that combination **valid**, so the test is
**deliberately re-pointed** to assert the combination now succeeds (with a comment naming rung 21). This
is an intentional behavior change, called out in the commit — not an accidental break.

## Done when
Reduce reproduces rungs 1–20 to the digit (the rung-20 forbid gate consciously inverted; all suites
green) and the cycle is bit-for-bit rung 6; gates 2–6 hold. `main.py` gains a rung-21 panel (the
effective ideal-bell lift, the point-value-vs-peak spread, the composite `pdf_quench` sitting between its
terms, and the honest scope). `CLAUDE.md` scope + rung table + deferred seams updated (ideal-bell PDF
lifts **done**; the "clamp fires at the burner" slow-freeze and the spatial/transported-CFD PDF stay the
two next seams).
