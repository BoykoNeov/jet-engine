# Rung-21 anchor — super-equilibrium O through the IDEAL-BELL PDF integrals

The numbers-before-code record for rung 21: the effective super-eq-O lift on the **ideal-bell
composition integrals** — `ei_no_pdf` (rung 13), `ei_no_pdf_quench` term 2 (rung 15),
`ei_no_transported` (rung 18) — that rung 20 deliberately left on equilibrium O (forbidden to combine).
Instrumented **before** the lesson was written (project discipline; the advisor's explicit gate was
"the ideal bell samples pockets at their FLAME temperatures peaked near stoich — measure whether the
effective lift lands near the primary ×1.28, the quench ×1.17, or elsewhere; that measured number IS
the load-bearing result"). Instrumentation: `M:\claud_projects\temp\rung21\instrument_pdf_lift.py`.

No new external constants — rung 21 reuses the rung-19 Westenberg `m(T)` ratio, now threaded through the
existing `_ideal_bell_ei`/`_bell_interpolator`/`_pdf_mean_ei` chain. What is new is the **EI-weighted**
behaviour of that ratio across the composition bell, which had to be measured, not assumed.

## 1. The design point (the rung-13/17 rich RQL primary)
`M0=0.85, T0=250 K, p0=50 kPa; πc=10, Tt4=1500 K`; equilibrium engine ⇒ `far=0.02718, Tt3=583.5 K,
p=747 kPa` ⇒ `ξ̄=0.02646` (overall φ≈0.40, deep lean). Rich primary `φ_p=1.5`. β-PDF over the ideal
bell EI(φ) on the ξ-grid up to the φ=2 soot bound (`n_bell=n_quad=200`).

## 2. Where the bell EI mass is vs where `m` is large (the SAME inversion as rung 20)
The ideal bell EI(φ), its flame temperature `T_p`, the multiplier `m(T_p)`, and the per-φ lift:

| φ | T_p (K) | m(T_p) | EI_eq | EI_super | lift |
|---|---|---|---|---|---|
| 0.60 | 1896 | 1.418 | 0.0495 | 0.0702 | 1.418 |
| 0.73 | 2109 | 1.277 | 1.208 | 1.541 | 1.275 |
| **0.97** | **2424** | **1.149** | **21.8** | **24.41** | **1.120** ← bell peak, m minimal |
| 1.02 | 2450 | 1.142 | 18.39 | 20.36 | 1.107 |
| 1.22 | 2348 | 1.174 | 0.594 | 0.692 | 1.164 |
| 1.50 | 2111 | 1.276 | 1.34e-3 | 1.71e-3 | 1.275 |
| 1.90 | 1804 | 1.501 | 1.3e-7 | 2.0e-7 | 1.501 |

The EI **mass** lives at the near-stoich **peak** (φ≈0.9–1.05, T≈2360–2450 K), where `m` is at its
**minimum** (`≈1.11–1.15`). The lean and rich flanks carry larger `m` (→1.5+) but orders-of-magnitude
less EI. This is rung 20's inversion generalized from the *cooling trajectory* to the *composition bell*:
NO forms where it is **hottest**, and `m` is **smallest** where it is hottest.

## 3. The measured lift (the headline number)
The effective `⟨EI⟩_pdf` lift, integrating the β-PDF over the bell at a range of segregation widths `g`:

| g (segregation) | ⟨EI⟩_eq | ⟨EI⟩_super | lift |
|---|---|---|---|
| 0.005 | 0.449 | 0.527 | **×1.173** |
| 0.020 | 1.183 | 1.360 | ×1.149 |
| 0.050 | 0.981 | 1.123 | ×1.146 |
| 0.100 | 0.673 | 0.770 | ×1.144 |
| 0.200 | 0.386 | 0.442 | ×1.144 |
| 0.300 | 0.252 | 0.288 | **×1.144** |

**The ideal-bell PDF lift is ×1.14–1.17** — right alongside the rung-20 quench lift (bulk ×1.17,
pocket ×1.16), **below** the rung-19 primary lift (×1.28), and **far below** the deep-lean point value:

| reference | lift | why |
|---|---|---|
| deep-lean **point value** (g→0 at ξ̄, φ≈0.40) | **×1.912** | cool lean flame (T≈1600 K) ⇒ `m` large, but EI≈1.3e-5 (negligible) |
| **⟨EI⟩_pdf** (g>0, the number that matters) | **×1.14–1.17** | β-PDF is EI-weighted ⇒ dominated by the near-stoich peak where `m` is minimal |
| bell **peak** (φ≈0.97) | ×1.12 | the hottest point, `m` at its floor |
| rung-19 **primary** (φ_p=1.5) | ×1.28 | `m(2110 K)` — a cooler, off-peak point than the bell peak |
| rung-20 **quench** (bulk/pocket) | ×1.17 / ×1.16 | the same peak-concentration on the cooling path |

**The lift even DECREASES with segregation** (×1.173 at g=0.005 → ×1.144 at g=0.30): more variance pulls
more PDF mass onto the near-stoich peak, where the lift is smallest. So heavier segregation gives *less*
fractional lift — the honest, measured, slightly counter-intuitive corollary.

### Production `zoned_nox` cross-check (J=36 over-penetration flank, after the guard flip)
Confirming the standalone bell integral in the real code path (`instrument`→`verify_combined.py`):

| field | eq-O | super-eq-O | lift |
|---|---|---|---|
| `ei_no_pdf` (rung 13) | 0.5871 | 0.6717 | **×1.144** |
| `ei_no_pdf_quench` (rung 15 composite) | 1.9838 | 2.2931 | **×1.156** |
| `ei_no_transported` (rung 18) | 0.9939 | 1.1482 | **×1.155** |
| `ei_no_quenched` (rung-20 bulk, reference) | 0.8993 | 1.0524 | ×1.170 |

`super_eq_o=False` is **bit-for-bit identical** to the default for all three fields (a defaulted kwarg
gating a single multiply). The `pdf_quench` **composite** lift (×1.156) sits **between** its term1
(bulk quench ×1.17) and term2 (ideal-bell ×1.14) — the measured proof the hybrid is resolved, both
terms carrying `m(T)`.

## 4. Why this closes the seam cleanly (the pdf_quench hybrid, resolved)
Rung 20 **forbade** `super_eq_o` with `pdf_quench` because `ei_no_pdf_quench = term1 + term2` would
become a **half-lifted hybrid**: term1 (`ei_no_quenched`, the mean-field bulk quench) was already lifted
×1.17 (rung 20), but term2 (the β-PDF integral over the ideal bell) stayed eq-O. Rung 21 lifts term2 by
the SAME peak-concentrated ×1.14–1.17, so both terms carry `m(T)` and the sum is internally consistent —
the hybrid objection is discharged, not worked around. (Because both terms lift by nearly the same
factor, the *composite* `ei_no_pdf_quench` lift is ≈×1.17 too — no new surprise, just consistency.)

## 5. The reduce and the g→0 consistency (the load-bearing gates)
- `super_eq_o=False` ⇒ `m≡1` ⇒ the `_ideal_bell_ei`/`_pdf_mean_ei`/`_bell_interpolator` calls are
  **byte-identical** to the prior rung ⇒ `ei_no_pdf`/`ei_no_pdf_quench`/`ei_no_transported` bit-for-bit
  rungs 13/15/18. Cycle bit-for-bit rung 6.
- `g→0` (at C_opt, the delta): the lifted `_pdf_mean_ei` short-circuits to the **super-eq-O well-mixed
  point value** `_ideal_bell_ei(far, …, super_eq_o=True)` (lift ×1.91 at this lean mean) — consistent
  with the g>0 integral in the limit, mirroring rung 13's `g→0` reduce.

## 6. Honest scope (carried from rungs 19–20)
- The super-eq **ratio** `m(T)` is Westenberg's fitted partial-eq/eq ratio — **semi-empirical** (a
  full-equilibrium pool cannot self-yield super-eq O). The lifted PDF integrals are **better-justified
  but not pinned** magnitudes, exactly as the rung-20 clamp `a`.
- Rung 21 is a **shape-preserving consistency lift**: it does NOT change the rung-13 optimum LOCATION
  (the min stays pinned AT `C_opt`, where `g→0`), the `(H/S)²` shift, or the humped ⟨EI⟩(g) / stoich-mean
  sign reversal. It multiplies the whole family by a peak-concentrated ≈×1.15. The *lesson* is the
  **measured magnitude and its inversion**, not a relocated feature.
- The **prompt** term stays a primary-only invariant EI (rung 19); it is NOT injected into the PDF
  integrals (its magnitude is imposed — the rung-20 rationale).
- The **"per-pocket clamp fires AT the burner"** slow-freeze seam and the **spatial/transported-CFD PDF**
  remain the two deferred seams (rungs 22/23).
