# The anchored-δ(J) (jet-in-crossflow trajectory) investigation — INVESTIGATED, **NOT SHIPPED**

> **Status.** This is a **negative-result record**, not a rung spec. It is the attack the
> **SCALE-negative** (`docs/mixing-scale-negative.md`) named as *the one worthwhile next step* on the
> standing **mixing-ceiling** seam: *"a new attempt is only worthwhile if it brings an **anchored**
> penetration exponent (or a real transported/CFD cross-plane field)."* It was derived, prototyped and
> tested, returned a negative verdict on **pinning the emissions optimum**, and was **deliberately not
> added to the ladder**. It is NOT a rung — no `*-spec.md`, no `gas.py`/`main.py`/test code, no entry in
> the rung table, by design.
>
> **Why this file exists:** so the negative is not re-investigated from scratch, and so the two
> *positive* by-products (rung 22 confirmed on its penetration law; the SCALE-negative sharpened) are not
> lost. If you are about to *"anchor rung-22's penetration exponent with a jet-in-crossflow trajectory
> law,"* read this first — it was done.

## What this investigation was

The standing mixing-ceiling seam (`docs/rung22-spec.md` … `docs/rung24-spec.md` § deferred). Rungs 22–24
feed a Gaussian-plume **cartoon** of one dilution cross-plane through the β-PDF closure:

- **penetration** `δ = k_p·√(S·H)·J^(1/4)` — the number `C_opt≈2.5` rides on the semi-empirical `k_p`;
- **spread** a FIXED mixing length `σ_y=k_y·H, σ_z=k_z·S` (J-independent);
- **scale** one global `τ_mix ∝ 1/√J` (rung 11, un-anchored).

The **SCALE-negative** built a penetration-growing plume (`δ∝J^p`) + a finite-`τ_res` dwell cap and
found the emissions turn's *existence and location* ride on the **un-anchored penetration exponent `p`**
(clean interior U at the hand-picked `p=1/4`; monotone at the "more standard" `p≈1/2`). It named an
**anchored `δ(J)` law** as the one worthwhile successor.

**This is that attack.** The published jet-in-crossflow (JICF) trajectory correlation

```
   y/(r·d) = A·(x/(r·d))^m ,   r = velocity ratio (= √J for a density-matched jet),  d = jet diameter
   Pratte & Baines (1967):     A = 2.05,  m = 0.28
   Hasselbrink & Mungal (2001): A = 1.60,  m = 0.33
```

evaluated at a fixed dilution length `x` gives `δ = A·x^m·(rd)^(1−m)`, i.e. an **anchored** penetration
exponent on `J` — `m` from measurement, not hand-picked. Under rung-22's fixed-mass-ratio jet diameter
`d∝√(SH)·J^(−1/4)` (so `rd=√(SH)·J^(1/4)`) this is `δ ∝ J^{(1−m)/4}`.

> **Source scope (disclosed).** The `A`/`m` values above are quoted from a **secondary literature
> summary** (a web search of the JICF trajectory-scaling literature; the primary Pratte-Baines /
> Hasselbrink-Mungal / Margason PDFs were rate-limited or unparsable and were **not** read first-hand).
> This does **not** load-bear: the Part-1 verdict is **insensitive to the exact `m`** — *any* `m>0` (any
> bent trajectory) breaks the `(S/H)√J` collapse (the drift is monotone in `m`; see the table), and the
> Part-2 emissions run holds penetration at `p=1/4` **regardless of `m`**. The correlation is used only to
> establish that a *physically-anchored* penetration exponent is `>1/4`-family (bent) or the momentum-depth
> `m=0`, not for any specific number.

---

## Part 1 — the PENETRATION axis: the Holdeman collapse rules IN `δ∝rd`, rules OUT the bent trajectory

`proto1_collapse.py` re-runs the **shipped** `_spatial_segregation` width with only the penetration
exponent changed, and asks whether the g-width uniformity optimum still **collapses onto the Holdeman
group** `C=(S/H)√J` as `S,H` vary independently.

| penetration | exponent on J | `C=(S/H)√J_opt` spread over 2× indep. `S`/`H` | |
|---|---|---|---|
| `δ∝rd` straight (rung 22, `m=0`) | 0.250 | **0.0%** (`C=2.504` everywhere) | COLLAPSES (Holdeman) |
| Pratte-Baines `m=0.28`, fixed-mass | 0.180 | 26.8% | DRIFTS |
| Hasselbrink-Mungal `m=0.33`, fixed-mass | 0.168 | 30.5% | DRIFTS |
| Pratte-Baines `m=0.28`, fixed-diam | 0.360 | 21.1% | DRIFTS |
| Hasselbrink-Mungal `m=0.33`, fixed-diam | 0.335 | 17.5% | DRIFTS |

The mechanism is exact: the field's z-structure is fully geometry-normalized (`σ_z=k_z·S` over a
period-`S` cell), so `g` depends on `δ/H` **by construction** (the `g_min` value is `0.0182` at *every*
geometry — the tell). The optimum group is therefore `(S/H)^{1−1/(4p)}`, which equals Holdeman's
`(S/H)√J` **only at `p=1/4`**, i.e. `m=0`. (Analytic check: `m=0.28` halve-S predicts
`4.295·0.5^{−0.389}=5.62`, matches the measured `5.624`.)

**HONEST SCOPE — this is NOT "data anchors `m=0`.**" Because `g` depends on `δ/H` by construction and
Holdeman's group is exactly what rung 22 *chose* `√(SH)J^{1/4}` to reproduce, "collapse ⟺ `p=1/4`" is
**algebra**, not independent data pulled back out. The correct, deflated claim is a **RULING-OUT**: the
**bent-JICF penetration forms are inconsistent with the standard `(S/H)√J` dilution-jet correlation;
rung-22's `δ∝rd` is the consistent one.** Physically: confined-dilution-jet penetration **depth** scales
as the momentum length `rd`, which is *not* a fixed-`x` slice of a free-JICF bent trajectory `δ∝rd^{1−m}`
— a known distinction. It is **not** a "near-field" claim (the near field is `m≈1/2`; `m=0` is a
depth scaling orthogonal to the trajectory exponent).

So Part 1 **confirms rung 22 on the penetration axis** and defuses the SCALE-negative's `p≈1/2`
pessimism *there* — the "more standard" bent exponent it feared is ruled out by the very correlation
`C_opt` comes from. **But only that axis.**

---

## Part 2 — the EMISSIONS axis: still NOT pinned — a SECOND, un-anchored exponent

`proto2_emissions.py` holds penetration at the collapse-consistent `p=1/4`, reuses the SCALE-negative
harness (growing-σ plume + finite-`τ_res` cap on the **shipped rung-24 chemistry**), and jogs the axes
Part 1 did **not** touch: the **spread-growth exponent `p_σ`** (decoupled from penetration), the cap
`τ_res`, and the rate constant `c_D`. The question the seam asks: is the `⟨EI⟩(J)` min **location**
(in `C`) stable?

`⟨EI_NO⟩` over `C=(S/H)√J = 1.25 … 5.00` (grid 32², n_quad 120), penetration `p=1/4` throughout:

| config | 1.25 | 1.88 | 2.50 | 3.12 | 3.75 | 4.38 | 5.00 | global min | interior dip |
|---|---|---|---|---|---|---|---|---|---|
| BASELINE `p_σ=.25` | 2.792 | 2.923 | 2.889 | **2.817** | 2.869 | 2.921 | 2.938 | J4 endpoint | **C≈3.12** |
| SPREAD-FIXED `p_σ=0` | 2.927 | 2.929 | 2.889 | 2.910 | 2.915 | 2.878 | **2.829** | J64 endpoint | ~none |
| SPREAD-STRONG `p_σ=.5` | 2.668 | 2.856 | 2.889 | 2.449 | 2.361 | 2.382 | **2.213** | J64 endpoint | erased (flank falls) |
| CAP-SHORT `τ=1.0ms` | **1.738** | 1.876 | 1.870 | 1.814 | 1.852 | 1.885 | 1.892 | J4 endpoint | C≈3.12 |
| CAP-LONG `τ=5.0ms` | 4.500 | 4.620 | 4.541 | **4.447** | 4.528 | 4.611 | 4.645 | J25 interior | C≈3.12 |
| CD-HIGH `×2` | **2.769** | 2.900 | 2.868 | 2.798 | 2.853 | 2.907 | 2.924 | J4 endpoint | C≈3.12 |
| CD-LOW `×0.5` | **2.807** | 2.938 | 2.902 | 2.828 | 2.879 | 2.930 | 2.946 | J4 endpoint | C≈3.12 |

**The three-way decomposition (the crisp reading).**

1. **Interior-turn LOCATION — penetration-anchored, STABLE at `C≈3.12`.** Across the cap (×0.4–2) and
   `c_D` (×0.5–2) the interior local min never moves off `C≈3.12`. It is `g`-driven, and `g` is
   penetration-anchored (`p=1/4` held). *When a turn exists, it is at `C≈3.12`* (offset rich of the
   uniformity `C_opt=2.5` by the still-falling mean-field bulk — the rung-12 pull-right).
2. **Interior-turn EXISTENCE — rides on the un-anchored SPREAD exponent `p_σ`.** `p_σ` sets the
   high-`J` flank *slope*: `p_σ=0` flat (no turn), `0.25` rising (interior dip), `0.5` falling
   (turn erased, curve tilts hard to the over-penetration endpoint). Confirmed at the field level
   (`proto3_diag.py`): the spread exponent **moves the `g`-optimum** (`C=2.5 → 3.12 → drift`) and flips
   whether the high-`J` flank **rises** (fixed-σ far-wall pile-up) or **falls** (a growing plume
   re-uniformizes the over-penetrated jet: `g` = 0.0084→0.0064 at `p_σ=0.5`).
3. **GLOBAL min — at an ENDPOINT in 6/7 configs** (rung-22's documented max-segregation concession:
   high `g` moves mass off the stoich peak, lowering mean NO through the ideal-bell far flank —
   reproduced, not new). *Which* endpoint (under- vs over-penetration) flips with `p_σ`. The
   SCALE-negative only reported a "clean interior U at `p=1/4`" because its `J`-grid `[9…36]` **excluded
   the low-`J` max-segregation endpoint** this wider grid reveals. And *whether the interior turn is ever
   the **global** min* itself rides on the un-anchored cap: only CAP-LONG (`τ_res=5ms`) promotes it to the
   global min, by amplifying the dwell term — one more free knob standing between the turn and a pin.

**THE HEADLINE — two un-anchored mixing exponents.** There are **two** free mixing exponents in the
cartoon: **penetration** and **spread/entrainment**. Part 1 rules **in** the first (`δ∝rd`, via
Holdeman-collapse consistency). The emissions optimum's *existence* rides on the **second** — the
spread/entrainment exponent — which JICF **trajectory** scaling does **not** supply. **Anchoring the
penetration exponent MOVES the free parameter from penetration to spread; it does not eliminate it.**

**Numerical health (checked, not assumed).** Mean-preservation drift is **0.00%** at every `J` and every
`p_σ` (`proto3_diag.py`) — the closure is not strained, so `p_σ=0.5`'s steep descent is physical
re-uniformization (`g` falling), not an artifact. **Grid convergence (32² → 48², baseline): the two
sweeps agree to `<0.15%` at every `J`** (max abs. `⟨EI⟩` diff `0.004`); the interior dip is a converged
`~3.6%` at `C≈3.12` (`2.924→2.820`), and the global min stays at the low-`J` endpoint in both — so the
interior turn is **real, not grid noise**, and the verdict is grid-robust.

**Do NOT claim** the `p_σ=0` flat curve refutes rung 22. Rung 22's U-shaped `g` runs through the
**ideal bell**; proto2 runs `g` through the **capped-dwell per-pocket quench**. Different path — the flat
`⟨EI⟩` at fixed σ is a property of the SCALE-negative-inherited construction, not a refutation.

---

## What this means for the seam (the corrected takeaway)

The emissions optimum needs **BOTH** mixing exponents anchored — penetration **and** spread/entrainment —
**or** the full transported/CFD cross-plane pattern. JICF **trajectory** scaling anchors only penetration.
So this attack:

- **CONFIRMS rung 22** on its penetration law: `δ∝rd` is the collapse-consistent choice; the bent-JICF
  alternatives the SCALE-negative feared are ruled out by the Holdeman correlation itself.
- **SHARPENS the SCALE-negative**: the un-anchored-exponent problem is **not solved** by the JICF
  trajectory — it is **relocated** from the penetration exponent to the spread exponent. And even the
  SCALE-negative's "clean interior U at `p=1/4`" was partly an artifact of a `J`-grid that excluded the
  max-segregation endpoint; on a wider grid the **global** min is at an endpoint (rung-22's concession),
  the interior turn being only *local*.

**Do NOT re-run:** the JICF-trajectory penetration + growing-σ-at-hand-picked-`p_σ` construction. That is
this investigation. **A new attempt is only worthwhile** if it brings an **anchored spread/entrainment
exponent** (a jet-plume spreading law for a *confined* dilution jet — itself murkier than the trajectory)
**or** a real transported/CFD cross-plane **pattern** (predicting the field shape, not just feeding a
β-PDF closure) — the standing ceiling rungs 22–24 already name.

(Precedent: rung 18's "a 0-D transport **cannot derive** `C_opt`", and the SCALE-negative / τ_res-negative
— shipped negatives of the same shape: the seam over-promised on one axis; anchoring it exposes the next.)

## Reproduction

Probes live in `M:\claud_projects\temp\rung-mixing-jicf\` (`proto1_collapse.py` — the collapse table;
`proto2_emissions.py` — the emissions discriminator via a module-level monkeypatch of
`gas._spatial_local_field` with the growing-σ + finite-`τ_res`-cap field and a `CappedMixing` subclass
overriding `tau_q`; `proto3_diag.py` — the field-health / `g(J)` diagnostic). That folder is **outside
git** (project temp policy), so this tracked file is the durable record. The probes reuse the shipped
rung-24 chemistry (`Gas.zoned_nox(..., spatial_local=SpatialLocalPDF(...))`) — **nothing in the shipped
tree was changed.** Sources: Pratte & Baines, *Chem. Eng.* (1967); Hasselbrink & Mungal, *JFM* (2001);
Margason, *Fifty Years of Jet in Cross Flow Research*, AGARD CP-534 (1993).
