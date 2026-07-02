# Rung 13 — The resolved mixing PDF: a mixture-fraction β-PDF that replaces rung-12's parameterised segregation (composition variance, isolated from dwell)

Rung 12 showed the dilution-jet mixing **optimum** is a **variance** effect, and captured it with
the *minimal* variance model — **two streams** (a mean-field bulk + an under-mixed core), with the
segregated fraction `w(C)` and core dwell `τ_core(C)` **parameterised** by hand. Rung 12 named its
own successor: a **resolved mixing PDF** — more than two streams, a *continuous distribution* of
mixture fraction, so the segregation is one width of a distribution rather than two hand-tuned
lumps. Rung 13 is exactly that distribution: a **mean-preserving β-PDF of mixture fraction** whose
single width — the **segregation `g`** — rides the same Holdeman group `C=(S/H)√J`. It pins the
**emissions optimum AT `C_opt≈2.5`** and makes the underlying lesson exact.

**A MECHANISM SEPARATION (the sharp result — not a "turn-up" reproduction).** Rung 12's NO-vs-`J`
curve *turned back up* on the over-penetration flank. That **climb** came from the **dwell** effect
— an **absolute**, off-optimum-growing `τ_core` that survives `J→∞` (a **TIME** mechanism). Rung 13
isolates the **COMPOSITION** mechanism (the mixture-fraction variance) and **drops the quench
chain**, so it **structurally cannot** climb. What composition variance *alone* delivers: the
emissions minimum **pinned AT `C_opt`** (perfect mixing → uniform lean → ≈0), both **immediate
flanks rising** (the convexity jump); and at *extreme* segregation ⟨EI⟩ **descends again** (the
β-PDF goes bimodal — real physics). So **composition variance pins the optimum location; the dwell
effect (rung 12) makes the over-penetration climb; combining them (the PDF through the quench) is
rung 14.** This replaces rung-12's segregation *parameterisation*, not its dwell mechanism —
complementary, not a superset.

**The lesson, framed correctly (this is NOT "convexity/Jensen").** The NO-vs-φ bell is convex on
its flanks but **concave at the peak** — there is no global convexity to invoke. The honest
statement: **NO production is sharply peaked at stoich, so spreading the local φ around a fixed mean
(unmixedness) RAISES the mean NO whenever the mean is OFF-stoich** (the stoich-ward tail samples the
peak while the mean sits in a low-EI wing), **most strongly the leaner the mean — and it REVERSES
sign at a stoich mean** (spreading then moves mass *off* the peak, lowering NO). Our combustor mean
is **lean** (`φ_overall≈0.40`; the dilution zone conserves mass ⇒ the mean mixture fraction is the
overall value, and the Holdeman group is a dilution-jet quantity), so segregation raises NO and the
optimum is where segregation is least (`C_opt`).

**Honest scope up front:** rung 13 **isolates** composition variance on the *ideal* primary bell
and **drops** the rungs 10–12 finite-quench dwell chain. So the optimum minimum collapses to the
**well-mixed lean value ≈ 10⁻⁵ g/kg** (near zero) — a **modeling boundary, not a physics claim**
(primary-history / bulk NO is out of scope). The **certified** content is the optimum **pinned AT
the Holdeman group `C_opt`** with **both immediate flanks rising**, and the **`(H/S)²` shift** —
recovered from a *continuous* PDF (the over-penetration climb is rung-12's dwell effect, absent
here). Carrying the PDF **through** the rung-12 quench is the **rung-14 seam**.

> **Read `docs/rung12-spec.md` first**, and `docs/plans/rung13-anchor-mixing-pdf.md` (numbers-
> before-code: the β-PDF closure, the **regime-aware quadrature** that makes mean-preservation exact
> for the lean-mean singularity, the machine-checked worked example — the convexity jump, the
> optimum pinned AT `C_opt` (both flanks rising), the humped ⟨EI⟩(g), the `(H/S)²` shift, the
> stoich-mean sign reversal, the two exact reduces, and the mechanism separation from rung-12's
> dwell). This file states only what *changes*; the Zeldovich rates, the equilibrium bell
> primitives (`_primary_aft`, `_equilibrium_composition`, `_thermal_no`), the `a6`/`a7`/`Kp`/
> `_equil_solve` substrate, the `JetMixing`/`Unmixedness` configs and Holdeman group, and the
> "derive before you code" / conservation-assert contract all carry over **unchanged**.

---

## What rung 13 adds (and what it deliberately does not)

**Adds:**

- **A resolved-mixing-PDF config** (`MixingPDF`) — the presumed **β-PDF** of mixture fraction, fixed
  by the mean `ξ̄` (= the overall mixture fraction) and a **segregation** width `g(C)`. It reuses the
  rung-12 Holdeman geometry: `g(C) = min(g_max, k_g·|ln(C/C_opt)|)` — the **same KINKED L1 distance**
  from `C_opt` that drove rung 12's `w(C)`. It **rides on** a `JetMixing` (it needs the jet's `J` and
  duct `H` to form `C`), so `pdf` is passed *with* `mixing`. Physical/order-of-magnitude defaults.
- **A PDF quadrature over the ideal bell `EI(ξ)`** — the rung-9 ideal-primary EI_NO, evaluated once
  on a fixed fine mixture-fraction grid and **linearly interpolated** onto the PDF's quadrature
  nodes (the bell is smooth). `⟨EI⟩(g) = ∫ EI(φ(ξ))·P_β(ξ;ξ̄,g) dξ`. Rung 13 adds **no new chemistry
  and no new integrator** — only the PDF integral over the existing bell.
- **A regime-aware, mean-preserving quadrature** — the lean mean gives a β shape parameter `a<1`, so
  `P_β ∝ ξ^{a−1}` has an **integrable singularity at ξ→0** that naïve uniform-in-ξ midpoint
  mis-weights (⟨ξ⟩ drifts, ⟨EI⟩ never converges). The fix: for `a<1` substitute **`u=ξ^a`** (the
  Jacobian cancels the singular factor exactly, leaving a bounded weight `(1−ξ)^{b−1}`, `b≥1`); for
  `a≥1` (near-delta) use a windowed uniform-in-ξ grid. **Mean-preservation `⟨ξ⟩≈ξ̄` is asserted every
  run** — the closure that computes ⟨EI⟩ at the wrong mean is exactly what the discipline forbids.
- **The payoff** — a sharp emissions minimum **PINNED AT `C_opt`** from a continuous PDF:
  `⟨EI⟩(g(C))` **collapses to the well-mixed lean value AT `C_opt`** (where `g=0` ⇒ delta ⇒
  `EI(φ_overall)`) and **both immediate flanks lift by orders** (segregation), the minimum at the
  Holdeman optimum (`J_min=J_opt`), shifting **exactly as `(H/S)²`**. (Not rung-12's climbing bowl —
  ⟨EI⟩(g) is **humped**, so the far-over-penetration flank *descends*; the climb is rung-12's dwell.)
- **`main.py` panel + `NOTES.md` section + `tests/test_rung13.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the PDF layer is opt-in via `pdf`. Every
  cycle station is **bit-for-bit rung 6** (the whole rung 1–12 suite stays green). The reduce is a
  **short-circuit**: `pdf=None` runs the exact rung-12 path (and its `unmixedness`/`mixing`/`tau_q`
  reduces below it).
- **Reduce bit-for-bit to rung 12.** Rung 13 is an **ALTERNATIVE closure** of the same unmixedness
  physics — a continuous PDF is *not* rung 12's two discrete streams plus one knob. The only
  legitimate reduces are the **`pdf=None` short-circuit** (code-path-identical rung 12) and the
  **`g→0` delta** (well-mixed point value). No knob turns the PDF into the two-stream model.
- **Model the finite-quench bulk.** Rung 13 isolates composition variance on the **ideal** bell; the
  rungs 10–12 dwell chain is dropped, so the optimum minimum is the well-mixed lean value ≈ 0, not a
  finite bulk. Bulk / primary-history NO is **out of scope** — the rung-14 seam.
- **Transport the PDF.** The β shape is **presumed**; its width `g(C)` is still a *modeled* function
  of the Holdeman group, not solved from a PDF-transport / mixing-frequency equation. It resolves the
  mixture-fraction *distribution* (rung 12 resolved only two points) but does not predict its shape.
- **Add super-equilibrium O / prompt (Fenimore) NO** — still deferred (rung-7 seam); the PDF ⟨EI⟩
  stays an equilibrium-O lower bound. Held **φ ≤ 2.0** (soot bound, rung 9): the bell is 0 for φ>2.

---

## The two things that make it work (stated loudly — they ARE the rung)

**(1) The correct lesson is "peaked-at-stoich × off-stoich-mean," not "convexity."** ⟨EI⟩ ≫ EI(⟨φ⟩)
holds for our lean mean because the stoich-ward tail of the PDF reaches the sharp bell peak; at a
**stoich mean the sign reverses**. Stating it as generic Jensen-convexity is false (the bell is
concave at its peak) and would predict the wrong sign at φ=1. The lean-mean choice is what makes
segregation *raise* NO — and it is physically forced (the dilution mean is the overall lean value).

**(2) The quadrature must PRESERVE THE MEAN — that assertion is the deliverable.** A presumed-β-PDF
closure exists precisely to hold the mean fixed while varying the variance. The lean-mean singularity
(`a<1`, `ξ^{a−1}` blows up at ξ→0) makes naïve quadrature compute ⟨EI⟩ at the **wrong mean** (a
35–95 % error in the one quantity that must be exact). The `u=ξ^a` change of variable cancels the
singularity exactly, restoring machine-precision mean-preservation and N-stability. The `g(C)` kink
then pins the emissions optimum at `C_opt` (as in rung 12: at `C_opt`, `g=0` ⇒ the PDF is a delta ⇒
⟨EI⟩ = the well-mixed point value), so `J_min = J_opt` shifts exactly as `(H/S)²`.

---

## The equations — a PDF integral over the bell, no station changes

Every cycle station is **bit-for-bit rung 6**. `zoned_nox` is the rung-8..12 flow; rung 13 only adds
the PDF integral over the ideal bell when a `MixingPDF` is passed:

```
NO PDF   (pdf=None):  the exact rung-12 two-stream (or rung-11/10/9/8 per short-circuits)  → rung 12
BETA-PDF (pdf=MixingPDF(…), REQUIRES mixing=JetMixing(J,…)):
   ξ̄  = far_overall/(1+far_overall)              mean mixture fraction (dilution mean = overall)
   C  = (S/H)·√J                                  Holdeman group (H, J from the jet)
   g  = min(g_max, k_g·|ln(C/C_opt)|)             segregation (KINKED; 0 at C_opt)
   a  = ξ̄·(1/g − 1),   b = (1−ξ̄)·(1/g − 1)       β shape params (mean-preserving; σ²=g·ξ̄(1−ξ̄))
   ⟨EI⟩ = Σ_i  w_i · EI_bell(ξ_i)                 regime-aware quadrature (u=ξ^a for a<1),
                                                   ASSERT Σ w_i ξ_i ≈ ξ̄  and  Σ w_i (ξ_i−ξ̄)² ≈ σ²
   g → 0  ⇒  ⟨EI⟩ = EI_bell(ξ̄) = EI(φ_overall)    delta short-circuit (well-mixed point value)
                                                  → min PINNED AT C_opt (J_min=J_opt); both flanks lift;
                                                    ⟨EI⟩(g) HUMPED ⇒ far-over-penetration flank descends
```

- **`pdf` REQUIRES `mixing`** — it needs the jet's `J` and `H` to form `C` (assert). `pdf=None` keeps
  the exact rung-12 path. `MixingPDF` and `unmixedness` are **mutually exclusive** (two closures of
  the same physics; assert not both).
- **Standing asserts (rung-13 deltas):** the rung-7 **K-check** + **trace guard** still bind at every
  bell node `T` (reused via `_thermal_no` when the bell is built); the **mean-preservation assert**
  (`⟨ξ⟩≈ξ̄`, variance ≈ target) on every quadrature; `g∈[0,g_max]` with `g=0` at `C_opt`; `S>0`,
  `C_opt>0`, `k_g≥0`, `0<g_max<1`; `pdf ⇒ mixing`; `not (pdf and unmixedness)`; β nodes `ξ∈(0,1)`.

---

## Verification gates (priority order)

1. **Reduce (load-bearing, exact by construction).** `pdf=None` short-circuits to the rung-12 path
   *before* any PDF code — every existing call is **bit-for-bit rung 12** (hence 11/10/9/6; the whole
   rung 1–12 suite stays green). Second reduce: `g→0` ⇒ the β-PDF is a delta at `ξ̄` ⇒ ⟨EI⟩ =
   `EI(φ_overall)` exactly (the well-mixed point value). **No** bit-for-bit reduce to the two-stream
   model is claimed (different closure).
2. **Mean-preservation holds (the deliverable).** `⟨ξ⟩ ≈ ξ̄` and variance ≈ `g·ξ̄(1−ξ̄)` for the
   whole `g`-range — machine-precision in the singular `a<1` regime (the `u=ξ^a` transform), <0.2 %
   in the `a≥1` regime. A quadrature that drifts the mean fails it.
3. **The optimum pinned AT `C_opt` (THE lesson).** `⟨EI⟩` collapses to the well-mixed lean value ≈0
   AT `C_opt` (`g=0`), and **both immediate flanks lift by orders** (segregation). A SEPARATE gate
   pins the **humped `⟨EI⟩(g)`** (peak at low `g`, descending toward high `g` as the β-PDF goes
   bimodal) — so the far-over-penetration flank *descends*, NOT rung-12's climbing bowl (that climb
   is the dwell effect, absent here). The mechanism separation is the result, not a defect.
4. **The optimum is AT the Holdeman group `C_opt`.** `J_min == J_opt = (C_opt·H/S)²` — shrink `S` and
   the EI-min moves to higher `J` **exactly as `(H/S)²`** (the kink pins it at `C_opt` for every `S`).
5. **The convexity jump is large and correct-signed** — ⟨EI⟩ ≫ EI(⟨φ⟩) for the lean mean (4–5
   orders), and **REVERSES SIGN at a stoich mean** (spreading lowers ⟨EI⟩ there). The discriminator
   that certifies the *peaked-at-stoich × off-stoich-mean* framing over the loose "convexity" claim.
6. **`g(C)` is the segregation** — 0 at `C_opt`, rising (kinked) on both flanks, symmetric in `ln C`,
   capped at `g_max`.
7. **Cycle untouched.** Re-running the cycle after a `pdf` `zoned_nox` call leaves station 4
   bit-for-bit (pure diagnostic).
8. **K-check + trace guards bind** at every bell node (reused unchanged from rungs 7–12).

## Conservation asserts (rung-13 deltas)
Carry over rung 6/7/8/9/10/11/12's, plus: `pdf ⇒ mixing`; `not (pdf and unmixedness)`; the `MixingPDF`
positivity/range guards (`S,C_opt>0`, `k_g≥0`, `0<g_max<1`); `g∈[0,g_max]` with `g=0` at `C_opt`; the
**mean-preservation assert** (`⟨ξ⟩≈ξ̄`, variance ≈ target) and β-shape validity (`a>0`, `b≥1`,
`ξ∈(0,1)`) on every quadrature.

## Done when
The reduce holds exactly (`pdf=None` short-circuit; rungs 1–12 green, untouched; cycle bit-for-bit
rung 6) and `g→0` gives the well-mixed point value; mean-preservation is asserted and exact in the
singular regime; the emissions minimum is **PINNED AT `C_opt`** (both flanks lifting) with the
**humped `⟨EI⟩(g)`** tested, `J_min = J_opt` **shifting as `(H/S)²`**; the convexity jump is large for
the lean mean and **reverses at a stoich mean**; the K-check/trace gates hold at every bell node.
`main.py` gains a rung-13 PDF panel (the `J`-sweep from the continuous PDF: the minimum at `C_opt`,
both flanks up, the humped far flank; the convexity jump; the `(H/S)²` shift; the mechanism
separation from rung-12's dwell); `NOTES.md` gains a rung-13 section (the resolved PDF vs the two
streams; the correct "peaked-at-stoich" framing; composition vs dwell; the mean-preserving
quadrature); `CLAUDE.md` scope +
rung table + deferred seams updated (resolved mixing PDF **done** — with the PDF-through-quench
integration (rung-14 seam), super-equilibrium O / prompt NO, and the frozen-vs-equilibrium nozzle
still carved out).

## The rung-14+ seam (keep it additive)
Rung 13 resolves the mixing field as a *presumed* PDF on the *ideal* bell. Next seams, all still
additive on this substrate: (a) **the PDF through the finite quench** — carry the resolved mixture-
fraction distribution through the rung-12 `_quench_no` trajectory (bulk + spread both dwelling), so
the optimum minimum is the finite bulk NO rather than the isolated well-mixed ≈ 0 (the immediate
seam); (b) **a transported / CFD PDF** — solve the PDF shape from a mixing-frequency / PDF-transport
equation rather than presuming β and modeling `g(C)`; (c) **super-equilibrium O / prompt (Fenimore)
NO** — the richer radical pool in exactly the near-stoich pockets this PDF now resolves, above the
equilibrium-O lower bound; (d) the still-open **equilibrium-vs-frozen nozzle expansion** (rung-6
seam). Only *how the PDF is obtained*, *what it rides through*, *on what radical pool*, and *how the
nozzle freezes* changes.
