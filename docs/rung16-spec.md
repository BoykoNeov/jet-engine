# Rung 16 — The PDF through the finite quench, PER POCKET: retiring rung-15's linearised dwell (the cooling-limited excess)

Rung 15 combined the two dilution-mixing mechanisms rungs 12–13 kept isolated — the **composition**
β-PDF (rung 13) **and** the **dwell** (rung 12) — into one additive result. But it did so with **one
acknowledged approximation**, stated loudly in its own spec: term 2 rescaled each pocket's
**constant-temperature** ideal-bell NO by a **scalar dwell ratio** `D(u) = τ_core/τ_ref`, exact only
while `EI ∝ τ` (the dormant clamp). That linearisation **ignores that a lingering pocket COOLS**. Rung
15 named its own successor: *carry the full per-pocket trajectory*. Rung 16 is exactly that.

**THE UPGRADE (only term 2's internals change).** Rung 16 carries **each rich-of-mean β-PDF pocket
through its OWN finite quench** — the same clamp-free `_quench_no` on the pocket's cooling/mixing
trajectory at the dwell `τ_core` — instead of rescaling a constant-`T` bell. The dwell now acts
**inside the chemistry**: a pocket that lingers **cools** as it re-makes NO through the stoichiometric
crossing, so its NO **saturates**. Term 1 (the mean-field bulk-quench floor) is **unchanged**.

**THE ROBUST RESULT (the honest headline — a refinement, not a reversal).** Resolving the dwell
per-pocket makes term 2 **SUBLINEAR** in `τ_core` (far-flank growth `≈×1.29` across `J=144→625` vs
rung-15's **linear** `×1.51` — the dwell ratio *exactly*). That cooling-limited growth **erodes
rung-15's over-penetration secondary basin by ~18–32 %**, dropping it into **near-degeneracy** with the
sharp `C_opt` notch. The composition mechanism survives intact: the excess still **→ 0 AT `C_opt`**
(both immediate flanks up), so the notch is a robust **local** optimum. **What rung 16 does NOT do** is
relocate the global optimum: which of the two now-comparable wells is globally lowest **flips sign**
across the β-PDF quadrature (~5 %), the `φ>2` tail treatment, and the `C_e` regime (2 %→21 % over
`0.20→0.15`) — all comparable to the margin. So rung 16 **quantifies rung-15's linearisation error**
(the linear dwell over-penalised over-penetration); it does **not** claim a new optimum location.

**THE CONSTRUCTION (additive — mirrors rung 15; only term 2's internals change).**

```
⟨EI⟩₁₆(J) = EI_bulk_quench(τ_mean(J))               [term 1: rung-11 MEAN-FIELD floor — UNCHANGED]
          + ⟨EI_pocket_quench(ξ; τ_core(C))⟩_g       [term 2: PER-POCKET quench β-PDF integral]
```

Compare rung 15's `term 2 = D(u(C)) · ⟨EI_bell⟩(g(C))`. The **only** change is inside term 2: the
constant-`T` bell `EI_bell(ξ)` scaled by the scalar `D(u)` becomes the **per-pocket quenched**
`EI_pocket_quench(ξ; τ_core)` — the dwell `τ_core` passed straight into each pocket's `_quench_no`
rather than multiplying a reference-`T` value. **Everything else is rung 15**: the same Holdeman group
`C=(S/H)√J`, the same kinked `g(C)` and `τ_core(C)`, the same mean-preserving β-PDF, the same additive
floor.

> **Honest scope up front (what is and isn't certified).** The **certified** content is the
> **sublinear dwell** (`×1.29` vs `×1.51`), the **far-flank erosion** (18–32 %), the **near-degeneracy**
> (the `C_opt`-vs-far gap collapses to `<½` of rung-15's), the composition excess **vanishing AT
> `C_opt`**, and the reduces. **Explicitly NOT certified — and no test asserts it —** is the
> **global-min LOCATION** (it is within the quadrature/tail/`C_e` ambiguity). The β-PDF is still
> **presumed** and `g(C)`/`τ_core(C)` still **modeled** (not transported). The clamp is **dormant**
> here (`max_a≈0.79 < 1`): the rung-15↔16 difference is **cooling within the dwell**, not
> super-equilibrium rollover — the clamp-free integrator is the *capability* (it would roll a super-eq
> pocket over), exercised but dormant, exactly like rung 10's own clamp (`0.677`).

> **Read `docs/rung15-spec.md` first** (rung 16 is its term 2, resolved), and
> `docs/plans/rung16-anchor-pocket-quench.md` (numbers-before-code: the C_e root-cause, the tail
> treatment, the diagonal that killed the tail-clamp artifact, the convergence study, and the machine-
> checked worked example — the sublinear ratio, the erosion, the near-degeneracy, both reduces). This
> file states only what *changes*; the Zeldovich rates, the clamp-free `_quench_no`, the τ-independent
> trajectory, the β-PDF quadrature (`_beta_pdf_nodes_weights`), the ideal-bell fallback
> (`_ideal_bell_ei`), the `JetMixing`/`QuenchPDF` configs and the Holdeman group, and the "derive
> before you code" / conservation-assert contract all carry over **unchanged**.

---

## What rung 16 adds (and what it deliberately does not)

**Adds:**

- **A per-pocket PDF-through-quench config** (`PocketQuenchPDF`) — the SAME knobs as `QuenchPDF`
  (`S`, `C_opt`, `k_g`, `g_max`, `τ_res`, `b_u`, `n_bell`, `n_quad`), differing only in what it does
  with the dwell: `core_dwell(C)=τ_res·(1+b_u·u)` is the **absolute** `τ_core` passed **into** each
  pocket's quench (not rung-15's `dwell_factor` ratio). It **rides on** a `JetMixing`, and is
  **mutually exclusive** with `pdf`, `pdf_quench` **and** `unmixedness` — a **≤1-of-four** guard (four
  closures of the same variance physics).
- **A module helper** `_pocket_quench_mean_ei(…)` — the rung-16 upgrade of `_pdf_mean_ei`: it builds a
  fixed ξ-grid over the burnable window `[0, ξ(φ=2)]`, carries each **rich-of-mean** pocket through its
  own `_quench_no` at `τ_core` (lean-of-mean / `φ>2` pockets fall back to `_ideal_bell_ei` — 0 above
  `φ2` — since they never re-cross stoich), interpolates, and integrates against the β-PDF quadrature.
  Returns `(⟨EI⟩, max_a)`. **No new chemistry, no new integrator** — it reuses the rung-10 `_quench_no`
  and the rung-13 quadrature.
- **The additive combination** in `zoned_nox`: term 1 = the rung-11 bulk quench already computed
  (`ei_no_quenched`); term 2 = `_pocket_quench_mean_ei(…)`. `⟨EI⟩₁₆ = term1 + term2`, recorded in
  `ei_no_pocket_excess`/`ei_no_pocket_quench`; the pocket `max_a` folds into the dormancy gate.
- **`main.py` panel + `tests/test_rung16.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the layer is opt-in via `pocket_quench`. Every
  cycle station is **bit-for-bit rung 6** (the whole rung 1–15 suite stays green). The reduce is a
  short-circuit: `pocket_quench=None` runs the exact rung-15 path (and its reduces below it).
- **Claim a global-min location.** The two near-degenerate optima are within the numerical/modeling
  ambiguity; **no gate asserts `argmin==J_opt` on the total** (contrast rung-15's GATE 3, valid there
  because rung 15's far basin sits a clear ~20 % above `C_opt`). Rung 16's near-degeneracy gate asserts
  the **gap collapsed**, never which well wins.
- **Reduce bit-for-bit to rung 12/13.** Rung 16 is rung 15 with the dwell resolved; the only legitimate
  reduces are the **`pocket_quench=None` short-circuit** (code-path-identical rung 15) and the **`C_opt`
  (`g→0`) limit** ⇒ the **finite bulk quench NO**.
- **Transport the PDF, or fire on super-equilibrium here.** The β shape is presumed; `g(C)`/`τ_core(C)`
  are modeled. The clamp is dormant at this design point — the correction is cooling, not rollover.
  Held **`φ ≤ 2.0`** (soot bound): rich pockets above `φ2` are out of scope (bell = 0), identical to
  rung 15 — which is what keeps the reduce and the `φ>2` β-PDF tail treatment exact.

---

## The one thing that makes it work (stated loudly — it IS the rung)

**The dwell must enter INSIDE the chemistry — that is the whole difference from rung 15.** Rung 15's
`term 2 = D(u)·⟨EI_bell⟩` multiplies a **constant-`T`** bell by a scalar dwell ratio, so term 2 is
**exactly linear** in `τ_core` (double the dwell, double term 2). That is correct only while the pocket
stays at its flame temperature. In reality a pocket that lingers is a pocket that is **mixing and
cooling**: the longer it dwells, the further down its cooling trajectory it has travelled, so each
extra increment of dwell re-makes **less** NO. Carrying the pocket through its own `_quench_no`
captures exactly this — term 2 becomes **sublinear** in `τ_core`. On the near-`C_opt` flank the two
agree (short dwell, little cooling); on the **far over-penetration flank** (long dwell, wide PDF) the
linearisation **over-shoots**, and the sublinear per-pocket result **erodes** it. That erosion is the
rung: it reveals that rung-15's sharp `C_opt` pin was **partly an artifact of the linear dwell**, and
that strong jets (over-penetration) cost far less NO than the linearisation implied.

---

## The equations — an additive combination, no station changes

Every cycle station is **bit-for-bit rung 6**. `zoned_nox` is the rung-8..15 flow; rung 16 only
replaces term 2's internals when a `PocketQuenchPDF` is passed:

```
POCKET-QUENCH  (pocket_quench=PocketQuenchPDF(…), REQUIRES mixing=JetMixing(J,…)):
   C   = (S/H)·√J                                    Holdeman group (H, J from the jet)
   g   = min(g_max, k_g·|ln(C/C_opt)|)               segregation (KINKED; 0 at C_opt)       — rung 13
   τ_core = τ_res·(1+b_u·|ln(C/C_opt)|)              absolute per-pocket dwell (KINKED)      — rung 12
   term1 = ei_no_quenched                            the rung-11 mean-field bulk quench (the FLOOR)
   term2 = ⟨EI_pocket_quench(ξ; τ_core)⟩_g           β-PDF integral of the PER-POCKET quench:
             per pocket ξ (far_local=ξ/(1−ξ), α=far/far_local):
               rich-of-mean, burnable, φ≤2 → _quench_no(…, τ_core)   (dilutes DOWN through stoich)
               lean-of-mean / φ>2 / too-lean → _ideal_bell_ei         (0 above φ2; never re-crosses)
   ei_no_pocket_quench = term1 + term2               → far flank ERODES into near-degeneracy w/ C_opt
   g → 0 (at C_opt)  ⇒  term2 → single lean pocket at ξ̄ ≈ 0  ⇒  ei_no_pocket_quench = ei_no_quenched
```

- **`pocket_quench` REQUIRES `mixing`** — it needs `J` and `H` to form `C` **and** the derived
  `τ_mean` for term 1 (assert). `pocket_quench=None` keeps the exact rung-15 path.
- **`PocketQuenchPDF`, `pdf_quench`, `pdf`, and `unmixedness` are mutually exclusive** — four closures
  of the same variance physics (assert at most one).
- **Standing asserts (rung-16 deltas):** the rung-7 **K-check** + **trace guard** bind at every
  trajectory `T` in **every pocket's** `_quench_no` (term 2) and in term 1's; the **mean-preservation
  assert** (`⟨ξ⟩≈ξ̄`, variance) on term 2's quadrature (reused from rung 13); `g∈[0,g_max]`, `u≥0`,
  both 0 at `C_opt`; `S,C_opt,τ_res>0`, `k_g,b_u≥0`, `0<g_max<1`; `pocket_quench ⇒ mixing`; at most one
  of `{pocket_quench, pdf_quench, pdf, unmixedness}`; the clamp-dormancy `max_a<1` gate now spans the
  bulk **and** every pocket stream.

---

## Verification gates (priority order)

1. **Reduce (load-bearing, exact by construction).** `pocket_quench=None` short-circuits to the rung-15
   path *before* any rung-16 code — every existing call is **bit-for-bit rung 15** (hence the whole
   rung 1–15 suite stays green). Second reduce: at `C_opt` (`g→0`) ⟨EI⟩₁₆ = the **finite bulk quench
   NO** `ei_no_quenched` to `<0.1%` (the single lean pocket at ξ̄ ≈ 0).
2. **Helper pinned to production.** The fast cached-bank helper matches `zoned_nox`'s
   `ei_no_pocket_quench` at the same resolution — so the sweep gates exercise the production arithmetic.
3. **THE MECHANISM — sublinear dwell.** rung-15 term 2 scales **linearly** with dwell (its far-flank
   ratio equals the `dwell_factor` ratio to `<2%`); rung-16 term 2 is **sublinear** (`ratio_16 <
   0.95·ratio_15`). The cooling made visible.
4. **THE HEADLINE — far-flank erosion.** `EI₁₆(J) < 0.9·EI₁₅(J)` at every over-penetration `J ∈
   {144,225,400,625}` — the sublinear dwell drops the whole far flank vs rung-15's linear climb.
5. **The far-flank CLIMB flattens (NOT a global-min claim).** The resolution-robust face of the
   erosion: rung-15's **linear** dwell makes the far over-penetration flank **CLIMB** (`EI₁₅` rises
   `>10%` across `J=144→625`), while rung-16's **sublinear** per-pocket dwell **FLATTENS** it
   (`climb₁₆ < ½·climb₁₅`). Same two endpoints, opposite far-flank slope — and rung-15's climb is
   exactly what put its basin *above* `C_opt`, so flattening it is what brings the two into
   near-degeneracy (at fine resolution). The gate asserts the **slope contrast**, and deliberately
   makes **no** `argmin` assertion (the global-min location is within the quadrature/tail/`C_e`
   ambiguity — contrast rung-15's GATE 3).
6. **The composition excess vanishes AT `C_opt`, both immediate flanks up.** term 2 `< 0.01·floor` at
   `C_opt`; both immediate flanks lift the total above the notch (the pin survives as a local optimum).
7. **Clamp dormancy over the pockets.** `max_a < 1` across the sweep and every pocket (the difference
   is cooling, not super-eq rollover); the pocket `max_a` folds into `max_a_quench`.
8. **Cycle untouched.** A `pocket_quench` call leaves the cycle `far` bit-identical — rung 6.
9. **Guards.** `pocket_quench ⇒ mixing`; the **≤1-of-four** mutual exclusivity; `PocketQuenchPDF`
   positivity/range (`k_g=0`/`b_u=0` allowed — floor-only / flat-dwell limits).
10. **The Holdeman kinks.** `g(C)`, `u(C)`, `τ_core(C)` are 0 / `τ_res` at `C_opt`, rising on both
    flanks, symmetric in `ln C`, `g` capped at `g_max`.
